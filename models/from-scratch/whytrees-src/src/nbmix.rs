//! Mixtures of naive Bayes models (CNBPs). Rust only.
//!
//! Two ways to mix K CNBPs over the same features. Both learn by counting:
//! each component adds an example's counts weighted by its responsibility
//! for it, so a component is a CNBP of the share of the positions it takes.
//!
//! `MoeNb`, a mixture of naive-Bayes experts: `moe`'s structure with CNBP
//! experts. A logistic-regression gate gives each expert a weight for the
//! position; the prediction is the experts' posteriors mixed by those
//! weights. Each expert adds the example's counts weighted by its
//! responsibility for the label (its share of the mixture's probability of
//! it), towards the label reweighted by its share of each class, instead of
//! a gradient step; the gate takes a gradient step towards the
//! responsibilities. The gate starts random, so the experts first see
//! different random weightings of the positions and specialise from there.
//!
//! `NbMix`, a generative mixture (latent class model):
//! p(x, y) = sum_k pi_k p(y | k) prod_i p(x_i | y, k), each component a CNBP
//! with its own class prior and class-conditional counts, and the
//! prediction p(y | x) ~ sum_k pi_k p(y | k) prod_i p(x_i | y, k). Training
//! is online EM: the responsibility of component k for (x, y) is
//! proportional to pi_k p(y | k) prod_i p(x_i | y, k), each component adds
//! the example's counts weighted by it, and pi_k's count grows by it. The
//! first `warm` examples each go to a random component: components that all
//! start empty would otherwise share every example equally for ever.
//!
//! Both score examples with the components' cached probabilities, refreshed
//! every `refresh` examples (stepwise EM in small batches; a refresh is one
//! pass over each component's counts) and whenever the model is refreshed
//! (after each round). Their files hold every count at full precision and
//! the examples seen, so a model trains on from its file exactly as if it
//! had not stopped.

use std::fmt::Write as _;
use std::io;
use std::path::Path;

use crate::cnbp::Cnbp;
use crate::logreg::Logreg;
use crate::textfmt::{Tokens, invalid};

/// Most components a mixture can have.
pub(crate) const MAX_COMPONENTS: usize = 64;

/// log(sum(exp(xs))), stable.
fn log_sum_exp(xs: &[f64]) -> f64 {
    let max = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if max == f64::NEG_INFINITY {
        return max;
    }
    max + xs.iter().map(|x| (x - max).exp()).sum::<f64>().ln()
}

/// Turns logs into probabilities, in place.
pub(crate) fn normalize_logs(xs: &mut [f64]) {
    let z = log_sum_exp(xs);
    xs.iter_mut().for_each(|x| *x = (*x - z).exp());
}

/// A component's counts at full precision.
fn write_counts(s: &mut String, c: &Cnbp) {
    for t in c.class_counts().iter().chain(c.cat_counts()) {
        writeln!(s, "{t:e}").unwrap();
    }
}

fn read_component(tokens: &mut Tokens, categories: &[u8], alpha: f64) -> io::Result<Cnbp> {
    read_cnbp(tokens, 3, categories, alpha)
}

/// A CNBP of `class_num` classes from counts written by `write_counts`.
fn read_cnbp(tokens: &mut Tokens, class_num: usize, categories: &[u8], alpha: f64) -> io::Result<Cnbp> {
    let blank = Cnbp::new(class_num, categories, alpha, true);
    let class_totals = (0..class_num).map(|_| tokens.next()).collect::<io::Result<Vec<f64>>>()?;
    let cat_totals =
        (0..class_num * blank.total_cat_vals()).map(|_| tokens.next()).collect::<io::Result<Vec<f64>>>()?;
    Cnbp::with_counts(class_num, categories, alpha, true, class_totals, cat_totals)
}

pub(crate) fn read_categories(tokens: &mut Tokens) -> io::Result<Vec<u8>> {
    let n: usize = tokens.next()?;
    (0..n).map(|_| tokens.next()).collect()
}

pub(crate) fn write_categories(s: &mut String, categories: &[u8]) {
    writeln!(s, "{}", categories.len()).unwrap();
    for c in categories {
        writeln!(s, "{c}").unwrap();
    }
}

/// splitmix64: the warm-up's random component for example `n`.
pub(crate) fn mix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9e3779b97f4a7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^ (z >> 31)
}

/// Mixture of naive-Bayes experts with a logistic-regression gate.
#[derive(Clone, Debug)]
pub struct MoeNb {
    categories: Vec<u8>,
    gate: Logreg,
    experts: Vec<Cnbp>,
    /// L2 penalty on the gate's weights (as `Moe`'s).
    gate_l2: f64,
    alpha: f64,
    /// Examples between refreshes of the experts' cached probabilities.
    refresh: u64,
    /// Examples trained on.
    seen: u64,
}

impl MoeNb {
    /// `expert_num` empty experts with Laplace smoothing `alpha`, and a gate
    /// with AdaGrad step `rate` and random weights in [-gate_scale, gate_scale].
    pub fn new(
        categories: &[u8],
        expert_num: usize,
        alpha: f64,
        rate: f64,
        gate_scale: f64,
        seed: u64,
        refresh: u64,
    ) -> Self {
        assert!((1..=MAX_COMPONENTS).contains(&expert_num), "1 to {MAX_COMPONENTS} experts");
        assert!(refresh >= 1, "refresh every 1 or more examples");
        let mut gate = Logreg::new(expert_num, categories, rate);
        gate.randomize(seed, gate_scale);
        let expert = Cnbp::new(3, categories, alpha, true);
        MoeNb {
            categories: categories.to_vec(),
            gate,
            experts: vec![expert; expert_num],
            gate_l2: 0.0,
            alpha,
            refresh,
            seen: 0,
        }
    }

    pub fn with_gate_l2(mut self, l2: f64) -> Self {
        self.gate_l2 = l2;
        self
    }

    pub fn component_num(&self) -> usize {
        self.experts.len()
    }

    /// The gate's log-weights and each expert's log-posterior of each class.
    fn parts(&self, data: &[u8], gate: &mut [f64], experts: &mut [[f64; 3]]) {
        self.gate.predict_log(data, gate);
        let z = log_sum_exp(gate);
        gate.iter_mut().for_each(|g| *g -= z);
        for (expert, out) in self.experts.iter().zip(experts.iter_mut()) {
            expert.predict_log_cached(data, out);
            let z = log_sum_exp(out);
            out.iter_mut().for_each(|l| *l -= z);
        }
    }

    /// The log-probability of each class: log sum_k g_k p_k(class).
    pub fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        let (mut gate, mut experts) = ([0.0; MAX_COMPONENTS], [[0.0; 3]; MAX_COMPONENTS]);
        let k = self.experts.len();
        self.parts(data, &mut gate[..k], &mut experts[..k]);
        let mut terms = [0.0; MAX_COMPONENTS];
        for (c, o) in out.iter_mut().enumerate().take(3) {
            (0..k).for_each(|e| terms[e] = gate[e] + experts[e][c]);
            *o = log_sum_exp(&terms[..k]);
        }
    }

    pub fn predict(&self, data: &[u8], out: &mut [f64]) {
        self.predict_log(data, out);
        normalize_logs(&mut out[..3]);
    }

    /// Each expert's responsibility for the label (or the gate's weights,
    /// without a label), into `out`; and, with a label, each expert's share
    /// of each class of it into `targets`. `gate` gets the gate's
    /// log-weights.
    fn responsibilities(
        &self,
        data: &[u8],
        label: Option<&[f64]>,
        out: &mut [f64],
        targets: &mut [[f64; 3]],
        gate: &mut [f64],
    ) {
        let mut experts = [[0.0; 3]; MAX_COMPONENTS];
        let k = self.experts.len();
        self.parts(data, &mut gate[..k], &mut experts[..k]);
        let Some(label) = label else {
            (0..k).for_each(|e| out[e] = gate[e].exp());
            return;
        };
        out[..k].iter_mut().for_each(|r| *r = 0.0);
        targets[..k].iter_mut().for_each(|t| *t = [0.0; 3]);
        let mut post = [0.0; MAX_COMPONENTS];
        for c in 0..3 {
            if label[c] <= 0.0 {
                continue;
            }
            // Each expert's share of the mixture's probability of class c.
            (0..k).for_each(|e| post[e] = gate[e] + experts[e][c]);
            normalize_logs(&mut post[..k]);
            for e in 0..k {
                targets[e][c] = label[c] * post[e];
                out[e] += targets[e][c];
            }
        }
    }

    /// p(expert | position) (the gate), or the responsibilities for `label`.
    pub fn component_weights(&self, data: &[u8], label: Option<&[f64]>, out: &mut [f64]) {
        let (mut targets, mut gate) = ([[0.0; 3]; MAX_COMPONENTS], [0.0; MAX_COMPONENTS]);
        self.responsibilities(data, label, out, &mut targets, &mut gate);
    }

    /// Each expert counts the example weighted by its responsibility; the
    /// gate steps towards the responsibilities.
    pub fn train(&mut self, data: &[u8], label: &[f64]) {
        let k = self.experts.len();
        let (mut r, mut targets, mut gate) = ([0.0; MAX_COMPONENTS], [[0.0; 3]; MAX_COMPONENTS], [0.0; MAX_COMPONENTS]);
        self.responsibilities(data, Some(label), &mut r[..k], &mut targets[..k], &mut gate[..k]);
        let total: f64 = r[..k].iter().sum();
        for e in 0..k {
            if r[e] > 0.0 {
                let target = targets[e].map(|t| t / r[e]);
                self.experts[e].train_weighted(data, &target, r[e]);
            }
            r[e] /= total;
            gate[e] = gate[e].exp();
        }
        self.gate.step(data, &gate[..k], &r[..k], 1.0, self.gate_l2);
        self.seen += 1;
        if self.seen % self.refresh == 0 {
            self.refresh();
        }
    }

    pub fn refresh(&mut self) {
        self.experts.iter_mut().for_each(Cnbp::refresh);
    }

    /// Divides the experts' counts by `factor`.
    pub fn forget(&mut self, factor: f64) {
        self.experts.iter_mut().for_each(|e| e.forget(factor));
    }

    pub fn to_text(&self) -> String {
        let mut s = String::new();
        let k = self.experts.len();
        writeln!(s, "moenb\n{k}\n{:e}\n{:e}\n{}\n{}", self.gate_l2, self.alpha, self.refresh, self.seen).unwrap();
        write_categories(&mut s, &self.categories);
        s.push_str(&self.gate.to_text());
        self.experts.iter().for_each(|e| write_counts(&mut s, e));
        s
    }

    pub fn from_text(text: &str) -> io::Result<Self> {
        let mut tokens = Tokens::new(text);
        if tokens.next::<String>()? != "moenb" {
            return Err(invalid("not a moenb model"));
        }
        let k: usize = tokens.next()?;
        if !(1..=MAX_COMPONENTS).contains(&k) {
            return Err(invalid(format!("{k} experts")));
        }
        let (gate_l2, alpha): (f64, f64) = (tokens.next()?, tokens.next()?);
        let (refresh, seen): (u64, u64) = (tokens.next()?, tokens.next()?);
        if refresh == 0 {
            return Err(invalid("refresh every 0 examples"));
        }
        let categories = read_categories(&mut tokens)?;
        let gate = Logreg::from_tokens(&mut tokens)?;
        if gate.class_num() != k {
            return Err(invalid("the gate's classes don't match the experts"));
        }
        let experts = (0..k).map(|_| read_component(&mut tokens, &categories, alpha)).collect::<io::Result<Vec<_>>>()?;
        if tokens.peek().is_some() {
            return Err(invalid("moenb model file has extra values"));
        }
        Ok(MoeNb { categories, gate, experts, gate_l2, alpha, refresh, seen })
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        std::fs::write(path, self.to_text())
    }

    pub fn load(path: &Path) -> io::Result<Self> {
        Self::from_text(&std::fs::read_to_string(path)?)
    }
}

/// Generative mixture of naive Bayes models (latent class), trained by online
/// EM.
#[derive(Clone, Debug)]
pub struct NbMix {
    categories: Vec<u8>,
    components: Vec<Cnbp>,
    /// Each component's count: its responsibilities summed.
    counts: Vec<f64>,
    /// Cached log of each component's smoothed share, pi_k.
    log_pi: Vec<f64>,
    alpha: f64,
    /// Examples that go to a random component before EM starts.
    warm: u64,
    /// Examples between refreshes of the cached probabilities.
    refresh: u64,
    seed: u64,
    /// Examples trained on.
    seen: u64,
}

impl NbMix {
    pub fn new(categories: &[u8], component_num: usize, alpha: f64, warm: u64, refresh: u64, seed: u64) -> Self {
        assert!((1..=MAX_COMPONENTS).contains(&component_num), "1 to {MAX_COMPONENTS} components");
        assert!(refresh >= 1, "refresh every 1 or more examples");
        let component = Cnbp::new(3, categories, alpha, true);
        let mut model = NbMix {
            categories: categories.to_vec(),
            components: vec![component; component_num],
            counts: vec![0.0; component_num],
            log_pi: vec![0.0; component_num],
            alpha,
            warm,
            refresh,
            seed,
            seen: 0,
        };
        model.refresh_pi();
        model
    }

    pub fn component_num(&self) -> usize {
        self.components.len()
    }

    fn refresh_pi(&mut self) {
        let k = self.counts.len() as f64;
        let total: f64 = self.counts.iter().sum();
        for (l, &c) in self.log_pi.iter_mut().zip(&self.counts) {
            *l = ((c + self.alpha) / (total + k * self.alpha)).ln();
        }
    }

    /// log pi_k + log p(class | k) + sum_i log p(x_i | class, k), for each
    /// component and class.
    fn joint_logs(&self, data: &[u8], out: &mut [[f64; 3]]) {
        for ((component, l), o) in self.components.iter().zip(&self.log_pi).zip(out.iter_mut()) {
            component.predict_log_cached(data, o);
            o.iter_mut().for_each(|x| *x += l);
        }
    }

    /// Log of each class's unnormalised probability: log p(x, class).
    pub fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        let k = self.components.len();
        let mut joint = [[0.0; 3]; MAX_COMPONENTS];
        self.joint_logs(data, &mut joint[..k]);
        let mut terms = [0.0; MAX_COMPONENTS];
        for (c, o) in out.iter_mut().enumerate().take(3) {
            (0..k).for_each(|e| terms[e] = joint[e][c]);
            *o = log_sum_exp(&terms[..k]);
        }
    }

    pub fn predict(&self, data: &[u8], out: &mut [f64]) {
        self.predict_log(data, out);
        normalize_logs(&mut out[..3]);
    }

    /// p(component | position), or with a label the EM responsibilities
    /// p(component | position, label) (label-weighted over its classes);
    /// with a label, also each component's share of each class into
    /// `targets`.
    fn responsibilities(&self, data: &[u8], label: Option<&[f64]>, out: &mut [f64], targets: &mut [[f64; 3]]) {
        let k = self.components.len();
        let mut joint = [[0.0; 3]; MAX_COMPONENTS];
        self.joint_logs(data, &mut joint[..k]);
        let Some(label) = label else {
            (0..k).for_each(|e| out[e] = log_sum_exp(&joint[e]));
            normalize_logs(&mut out[..k]);
            return;
        };
        out[..k].iter_mut().for_each(|r| *r = 0.0);
        targets[..k].iter_mut().for_each(|t| *t = [0.0; 3]);
        let mut post = [0.0; MAX_COMPONENTS];
        for c in 0..3 {
            if label[c] <= 0.0 {
                continue;
            }
            (0..k).for_each(|e| post[e] = joint[e][c]);
            normalize_logs(&mut post[..k]);
            for e in 0..k {
                targets[e][c] = label[c] * post[e];
                out[e] += targets[e][c];
            }
        }
    }

    pub fn component_weights(&self, data: &[u8], label: Option<&[f64]>, out: &mut [f64]) {
        let mut targets = [[0.0; 3]; MAX_COMPONENTS];
        self.responsibilities(data, label, out, &mut targets);
    }

    /// One online EM step (a random component during the warm-up).
    pub fn train(&mut self, data: &[u8], label: &[f64]) {
        let k = self.components.len();
        if self.seen < self.warm {
            let e = (mix64(self.seed ^ mix64(self.seen)) % k as u64) as usize;
            self.components[e].train_weighted(data, label, 1.0);
            self.counts[e] += 1.0;
        } else {
            let (mut r, mut targets) = ([0.0; MAX_COMPONENTS], [[0.0; 3]; MAX_COMPONENTS]);
            self.responsibilities(data, Some(label), &mut r[..k], &mut targets[..k]);
            for e in 0..k {
                if r[e] > 0.0 {
                    let target = targets[e].map(|t| t / r[e]);
                    self.components[e].train_weighted(data, &target, r[e]);
                    self.counts[e] += r[e];
                }
            }
        }
        self.seen += 1;
        if self.seen % self.refresh == 0 {
            self.refresh();
        }
    }

    pub fn refresh(&mut self) {
        self.components.iter_mut().for_each(Cnbp::refresh);
        self.refresh_pi();
    }

    /// Divides every count by `factor`.
    pub fn forget(&mut self, factor: f64) {
        self.components.iter_mut().for_each(|c| c.forget(factor));
        self.counts.iter_mut().for_each(|c| *c /= factor);
        self.refresh_pi();
    }

    pub fn to_text(&self) -> String {
        let mut s = String::new();
        let k = self.components.len();
        writeln!(s, "nbmix\n{k}\n{:e}\n{}\n{}\n{}\n{}", self.alpha, self.warm, self.refresh, self.seed, self.seen).unwrap();
        write_categories(&mut s, &self.categories);
        for c in &self.counts {
            writeln!(s, "{c:e}").unwrap();
        }
        self.components.iter().for_each(|c| write_counts(&mut s, c));
        s
    }

    pub fn from_text(text: &str) -> io::Result<Self> {
        let mut tokens = Tokens::new(text);
        if tokens.next::<String>()? != "nbmix" {
            return Err(invalid("not an nbmix model"));
        }
        let k: usize = tokens.next()?;
        if !(1..=MAX_COMPONENTS).contains(&k) {
            return Err(invalid(format!("{k} components")));
        }
        let alpha: f64 = tokens.next()?;
        let (warm, refresh, seed, seen): (u64, u64, u64, u64) = (tokens.next()?, tokens.next()?, tokens.next()?, tokens.next()?);
        if refresh == 0 {
            return Err(invalid("refresh every 0 examples"));
        }
        let categories = read_categories(&mut tokens)?;
        let counts = (0..k).map(|_| tokens.next()).collect::<io::Result<Vec<f64>>>()?;
        let components = (0..k).map(|_| read_component(&mut tokens, &categories, alpha)).collect::<io::Result<Vec<_>>>()?;
        if tokens.peek().is_some() {
            return Err(invalid("nbmix model file has extra values"));
        }
        let mut model = NbMix { categories, components, counts, log_pi: vec![0.0; k], alpha, warm, refresh, seed, seen };
        model.refresh_pi();
        Ok(model)
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        std::fs::write(path, self.to_text())
    }

    pub fn load(path: &Path) -> io::Result<Self> {
        Self::from_text(&std::fs::read_to_string(path)?)
    }
}

/// Mixture of logistic-regression experts with a naive-Bayes gate: `moe`'s
/// experts and training, but the gate is a CNBP with a class per expert
/// that counts the responsibilities (fractional counts) instead of taking
/// gradient steps towards them. The soft analogue of a tree with a CNBP
/// router and LR leaves. The first `warm` examples each go to a random
/// expert (the gate counts that expert), which breaks the symmetry between
/// experts that start identical; the gate's cached probabilities are
/// refreshed every `refresh` examples.
#[derive(Clone, Debug)]
pub struct NbGateMoe {
    categories: Vec<u8>,
    gate: Cnbp,
    experts: Vec<Logreg>,
    alpha: f64,
    warm: u64,
    refresh: u64,
    seed: u64,
    seen: u64,
}

impl NbGateMoe {
    pub fn new(
        categories: &[u8],
        expert_num: usize,
        alpha: f64,
        rate: f64,
        warm: u64,
        refresh: u64,
        seed: u64,
    ) -> Self {
        assert!((1..=MAX_COMPONENTS).contains(&expert_num), "1 to {MAX_COMPONENTS} experts");
        assert!(refresh >= 1, "refresh every 1 or more examples");
        NbGateMoe {
            categories: categories.to_vec(),
            gate: Cnbp::new(expert_num, categories, alpha, true),
            experts: vec![Logreg::new(3, categories, rate); expert_num],
            alpha,
            warm,
            refresh,
            seed,
            seen: 0,
        }
    }

    pub fn component_num(&self) -> usize {
        self.experts.len()
    }

    /// l4ncmp: the gate's weights and each expert's prediction (`parts`).
    pub fn gate_and_experts(&self, data: &[u8], gate: &mut [f64], experts: &mut [[f64; 3]]) {
        self.parts(data, gate, experts)
    }

    /// l4ncmp: expert `e`.
    pub fn expert(&self, e: usize) -> &Logreg {
        &self.experts[e]
    }

    /// The gate's weights and each expert's prediction.
    fn parts(&self, data: &[u8], gate: &mut [f64], experts: &mut [[f64; 3]]) {
        self.gate.predict_log_cached(data, gate);
        normalize_logs(gate);
        for (expert, out) in self.experts.iter().zip(experts.iter_mut()) {
            expert.predict(data, out);
        }
    }

    pub fn predict(&self, data: &[u8], out: &mut [f64]) {
        let (mut gate, mut experts) = ([0.0; MAX_COMPONENTS], [[0.0; 3]; MAX_COMPONENTS]);
        let k = self.experts.len();
        self.parts(data, &mut gate[..k], &mut experts[..k]);
        for (c, o) in out.iter_mut().enumerate().take(3) {
            *o = (0..k).map(|e| gate[e] * experts[e][c]).sum();
        }
    }

    /// As `MoeNb::responsibilities`, with each expert's prediction into
    /// `experts`.
    fn responsibilities(
        &self,
        data: &[u8],
        label: Option<&[f64]>,
        out: &mut [f64],
        targets: &mut [[f64; 3]],
        experts: &mut [[f64; 3]],
    ) {
        let mut gate = [0.0; MAX_COMPONENTS];
        let k = self.experts.len();
        self.parts(data, &mut gate[..k], &mut experts[..k]);
        let Some(label) = label else {
            out[..k].copy_from_slice(&gate[..k]);
            return;
        };
        let mixture: [f64; 3] = std::array::from_fn(|c| (0..k).map(|e| gate[e] * experts[e][c]).sum());
        for e in 0..k {
            targets[e] = [0.0; 3];
            for c in 0..3 {
                if label[c] > 0.0 {
                    targets[e][c] = label[c] * gate[e] * experts[e][c] / mixture[c].max(f64::MIN_POSITIVE);
                }
            }
            out[e] = targets[e].iter().sum();
        }
    }

    pub fn component_weights(&self, data: &[u8], label: Option<&[f64]>, out: &mut [f64]) {
        let (mut targets, mut experts) = ([[0.0; 3]; MAX_COMPONENTS], [[0.0; 3]; MAX_COMPONENTS]);
        self.responsibilities(data, label, out, &mut targets, &mut experts);
    }

    /// Each expert takes `moe`'s gradient step weighted by its
    /// responsibility; the gate counts the responsibilities.
    pub fn train(&mut self, data: &[u8], label: &[f64]) {
        let k = self.experts.len();
        if self.seen < self.warm {
            let e = (mix64(self.seed ^ mix64(self.seen)) % k as u64) as usize;
            self.experts[e].train(data, label);
            let mut onehot = [0.0; MAX_COMPONENTS];
            onehot[e] = 1.0;
            self.gate.train(data, &onehot[..k]);
        } else {
            let (mut r, mut targets, mut experts) =
                ([0.0; MAX_COMPONENTS], [[0.0; 3]; MAX_COMPONENTS], [[0.0; 3]; MAX_COMPONENTS]);
            self.responsibilities(data, Some(label), &mut r[..k], &mut targets[..k], &mut experts[..k]);
            let total: f64 = r[..k].iter().sum();
            for e in 0..k {
                if r[e] > 0.0 {
                    let target = targets[e].map(|t| t / r[e]);
                    self.experts[e].step(data, &experts[e], &target, r[e], 0.0);
                }
            }
            if total > 0.0 {
                r[..k].iter_mut().for_each(|x| *x /= total);
                self.gate.train(data, &r[..k]);
            }
        }
        self.seen += 1;
        if self.seen % self.refresh == 0 {
            self.refresh();
        }
    }

    pub fn refresh(&mut self) {
        self.gate.refresh();
    }

    /// Divides the experts' AdaGrad sums and the gate's counts by `factor`.
    pub fn forget(&mut self, factor: f64) {
        self.experts.iter_mut().for_each(|e| e.forget(factor));
        self.gate.forget(factor);
    }

    pub fn to_text(&self) -> String {
        let mut s = String::new();
        let k = self.experts.len();
        writeln!(s, "nbgate\n{k}\n{:e}\n{}\n{}\n{}\n{}", self.alpha, self.warm, self.refresh, self.seed, self.seen).unwrap();
        write_categories(&mut s, &self.categories);
        write_counts(&mut s, &self.gate);
        self.experts.iter().for_each(|e| s.push_str(&e.to_text()));
        s
    }

    pub fn from_text(text: &str) -> io::Result<Self> {
        let mut tokens = Tokens::new(text);
        if tokens.next::<String>()? != "nbgate" {
            return Err(invalid("not an nbgate model"));
        }
        let k: usize = tokens.next()?;
        if !(1..=MAX_COMPONENTS).contains(&k) {
            return Err(invalid(format!("{k} experts")));
        }
        let alpha: f64 = tokens.next()?;
        let (warm, refresh, seed, seen): (u64, u64, u64, u64) = (tokens.next()?, tokens.next()?, tokens.next()?, tokens.next()?);
        if refresh == 0 {
            return Err(invalid("refresh every 0 examples"));
        }
        let categories = read_categories(&mut tokens)?;
        let gate = read_cnbp(&mut tokens, k, &categories, alpha)?;
        let experts = (0..k).map(|_| Logreg::from_tokens(&mut tokens)).collect::<io::Result<Vec<_>>>()?;
        if experts.iter().any(|e| e.class_num() != 3) {
            return Err(invalid("experts must have 3 classes"));
        }
        if tokens.peek().is_some() {
            return Err(invalid("nbgate model file has extra values"));
        }
        Ok(NbGateMoe { categories, gate, experts, alpha, warm, refresh, seed, seen })
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        std::fs::write(path, self.to_text())
    }

    pub fn load(path: &Path) -> io::Result<Self> {
        Self::from_text(&std::fs::read_to_string(path)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Feature 0 picks one of two regimes; in each, a different feature
    /// decides the class, and the other is noise. Naive Bayes, which adds
    /// both features' evidence whatever the regime, cannot fit it; a mixture
    /// with a component per regime can.
    fn examples() -> Vec<([u8; 3], [f64; 3])> {
        (0..8000u32)
            .map(|i| {
                let regime = (i % 2) as u8;
                let (a, b) = ((i / 2 % 2) as u8, (i / 4 % 2) as u8);
                let decider = if regime == 0 { a } else { b };
                let label = if decider == 0 { [1.0, 0.0, 0.0] } else { [0.0, 1.0, 0.0] };
                ([regime, a, b], label)
            })
            .collect()
    }

    fn log_loss(predict: impl Fn(&[u8], &mut [f64])) -> f64 {
        let data = examples();
        data.iter()
            .map(|(x, l)| {
                let mut p = [0.0; 3];
                predict(x, &mut p);
                -(0..3).map(|c| l[c] * p[c].max(1e-12).ln()).sum::<f64>()
            })
            .sum::<f64>()
            / data.len() as f64
    }

    fn trained_nb() -> Cnbp {
        let mut nb = Cnbp::new(3, &[2, 2, 2], 1.0, true);
        for _ in 0..10 {
            for (x, l) in examples() {
                nb.train(&x, &l);
            }
        }
        nb.refresh();
        nb
    }

    #[test]
    fn moenb_fits_what_naive_bayes_cannot() {
        let nb = trained_nb();
        let mut mix = MoeNb::new(&[2, 2, 2], 2, 1.0, 0.1, 0.5, 7, 64);
        for _ in 0..10 {
            for (x, l) in examples() {
                mix.train(&x, &l);
            }
        }
        mix.refresh();
        let (nb_loss, mix_loss) = (log_loss(|x, p| nb.predict(x, p)), log_loss(|x, p| mix.predict(x, p)));
        assert!(mix_loss < 0.5 * nb_loss, "mixture {mix_loss}, naive Bayes {nb_loss}");
    }

    #[test]
    fn nbmix_fits_what_naive_bayes_cannot() {
        let nb = trained_nb();
        let mut mix = NbMix::new(&[2, 2, 2], 2, 1.0, 200, 64, 3);
        for _ in 0..10 {
            for (x, l) in examples() {
                mix.train(&x, &l);
            }
        }
        mix.refresh();
        let (nb_loss, mix_loss) = (log_loss(|x, p| nb.predict(x, p)), log_loss(|x, p| mix.predict(x, p)));
        assert!(mix_loss < 0.5 * nb_loss, "mixture {mix_loss}, naive Bayes {nb_loss}");
    }

    /// A model saved and reloaded predicts the same and trains on exactly as
    /// one that did not stop.
    #[test]
    fn text_round_trip_and_resume() {
        let data = examples();
        let (first, rest) = data.split_at(3001);
        let mut a = MoeNb::new(&[2, 2, 2], 3, 1.0, 0.05, 0.2, 1, 100).with_gate_l2(0.01);
        let mut b = NbMix::new(&[2, 2, 2], 3, 1.0, 500, 100, 9);
        for (x, l) in first {
            a.train(x, l);
            b.train(x, l);
        }
        a.refresh();
        b.refresh();
        let (mut a2, mut b2) = (MoeNb::from_text(&a.to_text()).unwrap(), NbMix::from_text(&b.to_text()).unwrap());
        assert_eq!(a2.to_text(), a.to_text());
        assert_eq!(b2.to_text(), b.to_text());
        for (x, l) in rest {
            a.train(x, l);
            a2.train(x, l);
            b.train(x, l);
            b2.train(x, l);
        }
        assert_eq!(a2.to_text(), a.to_text());
        assert_eq!(b2.to_text(), b.to_text());
        let (mut p, mut q) = ([0.0; 3], [0.0; 3]);
        b.predict(&[1, 0, 1], &mut p);
        b2.predict(&[1, 0, 1], &mut q);
        assert_eq!(p, q);
        assert!((p.iter().sum::<f64>() - 1.0).abs() < 1e-12);
    }

    /// One expert under a naive-Bayes gate is logistic regression.
    #[test]
    fn one_nbgate_expert_is_logistic_regression() {
        let mut lr = Logreg::new(3, &[2, 2, 2], 0.05);
        let mut mix = NbGateMoe::new(&[2, 2, 2], 1, 1.0, 0.05, 100, 7, 1);
        for (x, l) in examples().iter().take(500) {
            lr.train(x, l);
            mix.train(x, l);
        }
        mix.refresh();
        let (mut p, mut q) = ([0.0; 3], [0.0; 3]);
        lr.predict(&[0, 1, 1], &mut p);
        mix.predict(&[0, 1, 1], &mut q);
        (0..3).for_each(|c| assert!((p[c] - q[c]).abs() < 1e-12, "{p:?} {q:?}"));
        let back = NbGateMoe::from_text(&mix.to_text()).unwrap();
        assert_eq!(back.to_text(), mix.to_text());
    }

    #[test]
    fn nbgate_fits_what_logistic_regression_cannot() {
        let mut lr = Logreg::new(3, &[2, 2, 2], 0.1);
        let mut mix = NbGateMoe::new(&[2, 2, 2], 2, 1.0, 0.1, 400, 64, 5);
        for _ in 0..30 {
            for (x, l) in examples() {
                lr.train(&x, &l);
                mix.train(&x, &l);
            }
        }
        mix.refresh();
        let (lr_loss, mix_loss) = (log_loss(|x, p| lr.predict(x, p)), log_loss(|x, p| mix.predict(x, p)));
        assert!(mix_loss < 0.5 * lr_loss, "mixture {mix_loss}, logistic regression {lr_loss}");
    }

    /// Mixing one component is the component.
    #[test]
    fn one_component_is_naive_bayes() {
        let mut nb = Cnbp::new(3, &[2, 2, 2], 1.0, true);
        let mut mix = NbMix::new(&[2, 2, 2], 1, 1.0, 0, 1, 1);
        for (x, l) in examples().iter().take(500) {
            nb.train(x, l);
            mix.train(x, l);
        }
        nb.refresh();
        mix.refresh();
        let (mut p, mut q) = ([0.0; 3], [0.0; 3]);
        nb.predict(&[0, 1, 1], &mut p);
        mix.predict(&[0, 1, 1], &mut q);
        (0..3).for_each(|c| assert!((p[c] - q[c]).abs() < 1e-12, "{p:?} {q:?}"));
    }
}
