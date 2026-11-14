//! Mixture of logistic-regression experts with a tree-augmented naive-Bayes
//! gate (`tangate`). Rust only.
//!
//! `nbmix::NbGateMoe` with a TAN of fixed m (`tan`) as its gate in place of
//! its CNBP, all else the same: the gate's posterior over the experts weighs
//! their predictions; the first `warm` examples each go to a random expert
//! (by hash), which the gate counts; after that each expert takes a gradient
//! step weighted by its responsibility for the label, towards the label
//! reweighted by its share of each class, and the gate counts the
//! responsibilities (fractional counts, its pair counts included).
//!
//! While training, the gate scores examples from its current counts with the
//! tree of its last refresh (`Tan::predict_counts`), so it needs no periodic
//! refresh of cached probabilities; `refresh` (after each round) relearns its
//! tree and tables. A TAN that has seen nothing has no parents, so until its
//! first refresh the gate scores as NbGateMoe's CNBP would from the same
//! counts.
//!
//! The model file holds the gate's prediction tables and the experts at full
//! precision: enough to predict, not to train. The state file (`save_state`)
//! holds every count, to train on exactly as if the run had not stopped.

use std::fmt::Write as _;
use std::io;
use std::path::Path;

use crate::logreg::Logreg;
use crate::nbmix::{MAX_COMPONENTS, mix64, normalize_logs, read_categories, write_categories};
use crate::state::{Reader, Writer};
use crate::tan::{Tan, TanOptions};
use crate::textfmt::{Tokens, invalid};

/// The state file's kind (see `state`).
const STATE_KIND: &str = "tangate";

#[derive(Clone, Debug)]
pub struct TanGateMoe {
    categories: Vec<u8>,
    gate: Tan,
    experts: Vec<Logreg>,
    alpha: f64,
    warm: u64,
    seed: u64,
    seen: u64,
}

impl TanGateMoe {
    /// `expert_num` LR experts (learning rate `rate`) and a TAN gate with
    /// Laplace `alpha` and fixed m `tan_m`.
    pub fn new(categories: &[u8], expert_num: usize, alpha: f64, rate: f64, tan_m: f64, warm: u64, seed: u64) -> Self {
        assert!((1..=MAX_COMPONENTS).contains(&expert_num), "1 to {MAX_COMPONENTS} experts");
        TanGateMoe {
            categories: categories.to_vec(),
            gate: Tan::new(expert_num, categories, alpha, TanOptions::fixed(tan_m), true),
            experts: vec![Logreg::new(3, categories, rate); expert_num],
            alpha,
            warm,
            seed,
            seen: 0,
        }
    }

    pub fn component_num(&self) -> usize {
        self.experts.len()
    }

    /// The gate's weights and each expert's prediction: from the gate's
    /// counts while it trains (`training`), else from its tables.
    fn parts(&self, data: &[u8], gate: &mut [f64], experts: &mut [[f64; 3]], training: bool) {
        if training {
            self.gate.predict_counts(data, gate);
        } else {
            self.gate.predict_log(data, gate);
            normalize_logs(gate);
        }
        for (expert, out) in self.experts.iter().zip(experts.iter_mut()) {
            expert.predict(data, out);
        }
    }

    pub fn predict(&self, data: &[u8], out: &mut [f64]) {
        let (mut gate, mut experts) = ([0.0; MAX_COMPONENTS], [[0.0; 3]; MAX_COMPONENTS]);
        let k = self.experts.len();
        self.parts(data, &mut gate[..k], &mut experts[..k], false);
        for (c, o) in out.iter_mut().enumerate().take(3) {
            *o = (0..k).map(|e| gate[e] * experts[e][c]).sum();
        }
    }

    /// As `NbGateMoe::responsibilities`: the gate's weights (no label), or
    /// each expert's responsibility for the label, its target in `targets`
    /// and its prediction in `experts`.
    fn responsibilities(
        &self,
        data: &[u8],
        label: Option<&[f64]>,
        out: &mut [f64],
        targets: &mut [[f64; 3]],
        experts: &mut [[f64; 3]],
        training: bool,
    ) {
        let mut gate = [0.0; MAX_COMPONENTS];
        let k = self.experts.len();
        self.parts(data, &mut gate[..k], &mut experts[..k], training);
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

    /// The gate's weights for the position, or with a label each expert's
    /// responsibility for it (mix-stats), from the gate's tables: refreshed,
    /// or read from a model file.
    pub fn component_weights(&self, data: &[u8], label: Option<&[f64]>, out: &mut [f64]) {
        let (mut targets, mut experts) = ([[0.0; 3]; MAX_COMPONENTS], [[0.0; 3]; MAX_COMPONENTS]);
        self.responsibilities(data, label, out, &mut targets, &mut experts, false);
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
            self.responsibilities(data, Some(label), &mut r[..k], &mut targets[..k], &mut experts[..k], true);
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
    }

    /// Relearns the gate's tree and tables from its counts.
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
        writeln!(s, "tangate\n{k}\n{:e}\n{}\n{}\n{}", self.alpha, self.warm, self.seed, self.seen).unwrap();
        write_categories(&mut s, &self.categories);
        if self.gate.is_fresh() {
            self.gate.write_text(&mut s);
        } else {
            let mut gate = self.gate.clone();
            gate.refresh();
            gate.write_text(&mut s);
        }
        self.experts.iter().for_each(|e| s.push_str(&e.to_text()));
        s
    }

    /// Reads `to_text`'s format, to predict with: it can't train (see
    /// `load_state`).
    pub fn from_text(text: &str) -> io::Result<Self> {
        let mut tokens = Tokens::new(text);
        if tokens.next::<String>()? != "tangate" {
            return Err(invalid("not a tangate model"));
        }
        let k: usize = tokens.next()?;
        if !(1..=MAX_COMPONENTS).contains(&k) {
            return Err(invalid(format!("{k} experts")));
        }
        let alpha: f64 = tokens.next()?;
        let (warm, seed, seen): (u64, u64, u64) = (tokens.next()?, tokens.next()?, tokens.next()?);
        let categories = read_categories(&mut tokens)?;
        let gate = Tan::from_tokens(&mut tokens, k, &categories, alpha, true)?;
        let experts = (0..k).map(|_| Logreg::from_tokens(&mut tokens)).collect::<io::Result<Vec<_>>>()?;
        if experts.iter().any(|e| e.class_num() != 3) {
            return Err(invalid("experts must have 3 classes"));
        }
        if tokens.peek().is_some() {
            return Err(invalid("tangate model file has extra values"));
        }
        Ok(TanGateMoe { categories, gate, experts, alpha, warm, seed, seen })
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        std::fs::write(path, self.to_text())
    }

    pub fn load(path: &Path) -> io::Result<Self> {
        Self::from_text(&std::fs::read_to_string(path)?)
    }

    /// Every count and weight, to train on exactly as if the run had not
    /// stopped (`load_state`).
    pub fn save_state(&self, path: &Path) -> io::Result<()> {
        if !self.gate.is_trainable() {
            return Err(invalid("a tangate loaded from a model file has no training state"));
        }
        let mut w = Writer::new(STATE_KIND);
        w.usize(self.experts.len());
        w.f64(self.alpha);
        w.u64(self.warm);
        w.u64(self.seed);
        w.u64(self.seen);
        w.u8s(&self.categories);
        self.gate.write_state_to(&mut w);
        self.experts.iter().for_each(|e| e.write_state(&mut w));
        w.save(path)
    }

    /// Loads a state saved by `save_state`. The options must be those it was
    /// trained with.
    pub fn load_state(path: &Path, expert_num: usize, alpha: f64, rate: f64, tan_m: f64, warm: u64, seed: u64) -> io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let mut r = Reader::new(&bytes, STATE_KIND)?;
        let (k, saved_alpha, saved_warm, saved_seed, seen) = (r.usize()?, r.f64()?, r.u64()?, r.u64()?, r.u64()?);
        let categories = r.u8s()?;
        let gate = Tan::read_state_from(&mut r)?;
        let experts = (0..k).map(|_| Logreg::read_state(&mut r)).collect::<io::Result<Vec<_>>>()?;
        r.finish()?;
        let saved_m = gate.options().m;
        let rates_differ = experts.iter().any(|e| e.rate().to_bits() != rate.to_bits());
        if k != expert_num
            || saved_alpha.to_bits() != alpha.to_bits()
            || saved_m.to_bits() != tan_m.to_bits()
            || saved_warm != warm
            || saved_seed != seed
            || rates_differ
        {
            return Err(invalid(format!(
                "the state's options differ from this run's (give the options it was trained with): saved {k} experts, alpha {saved_alpha}, m {saved_m}, warm {saved_warm}, seed {saved_seed}; now {expert_num} experts, alpha {alpha}, m {tan_m}, warm {warm}, seed {seed}, rate {rate}"
            )));
        }
        if gate.class_num() != k || experts.iter().any(|e| e.class_num() != 3) {
            return Err(invalid("tangate state of the wrong shape"));
        }
        Ok(TanGateMoe { categories, gate, experts, alpha, warm, seed, seen })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nbmix::NbGateMoe;

    /// As nbmix's tests: feature 0 picks one of two regimes, and in each a
    /// different feature decides the class.
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

    fn trained(k: usize, m: f64, rounds: usize) -> TanGateMoe {
        let mut mix = TanGateMoe::new(&[2, 2, 2], k, 1.0, 0.1, m, 200, 7);
        for _ in 0..rounds {
            for (x, l) in examples() {
                mix.train(&x, &l);
            }
            mix.refresh();
        }
        mix
    }

    #[test]
    fn tangate_fits_what_one_expert_cannot() {
        let (one, two) = (trained(1, 5.0, 10), trained(2, 5.0, 10));
        let (one_loss, two_loss) = (log_loss(|x, p| one.predict(x, p)), log_loss(|x, p| two.predict(x, p)));
        assert!(two_loss < 0.5 * one_loss, "two experts {two_loss}, one {one_loss}");
    }

    /// One expert: the gate's weight is 1, so the model is that LR, trained
    /// as `nbgate`'s single expert is.
    #[test]
    fn one_expert_is_nbgates_one_expert() {
        let mut tan = TanGateMoe::new(&[2, 2, 2], 1, 1.0, 0.1, 5.0, 200, 7);
        let mut nb = NbGateMoe::new(&[2, 2, 2], 1, 1.0, 0.1, 200, 64, 7);
        for (x, l) in examples().iter().take(3000) {
            tan.train(x, l);
            nb.train(x, l);
        }
        tan.refresh();
        nb.refresh();
        for (x, _) in examples().iter().take(64) {
            let (mut a, mut b) = ([0.0; 3], [0.0; 3]);
            tan.predict(x, &mut a);
            nb.predict(x, &mut b);
            assert_eq!(a, b);
        }
    }

    /// Before any refresh with counts the gate has no parents, so it scores
    /// as NbGateMoe's CNBP does from the same counts (refreshed every
    /// example): the same experts, weights within rounding.
    #[test]
    fn before_its_first_refresh_the_gate_is_nbgates() {
        let mut tan = TanGateMoe::new(&[2, 2, 2], 3, 1.0, 0.1, 5.0, 100, 3);
        let mut nb = NbGateMoe::new(&[2, 2, 2], 3, 1.0, 0.1, 100, 1, 3);
        for (x, l) in examples().iter().take(2000) {
            tan.train(x, l);
            nb.train(x, l);
        }
        let (mut a, mut b) = ([0.0; 3], [0.0; 3]);
        let (mut targets, mut experts) = ([[0.0; 3]; 3], [[0.0; 3]; 3]);
        for (x, l) in examples().iter().take(64) {
            tan.responsibilities(x, Some(l), &mut a, &mut targets, &mut experts, true);
            nb.component_weights(x, Some(l), &mut b);
            for e in 0..3 {
                assert!((a[e] - b[e]).abs() < 1e-6, "{a:?} {b:?}");
            }
        }
    }

    /// The model file predicts as the model; the state resumes exactly.
    #[test]
    fn text_and_state_round_trips() {
        let dir = std::env::temp_dir().join(format!("tangate-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let data = examples();
        let (first, rest) = data.split_at(3001);
        let mut a = TanGateMoe::new(&[2, 2, 2], 3, 1.0, 0.05, 50.0, 500, 9);
        for (x, l) in first {
            a.train(x, l);
        }
        a.refresh();
        let back = TanGateMoe::from_text(&a.to_text()).unwrap();
        for (x, _) in data.iter().take(64) {
            let (mut p, mut q) = ([0.0; 3], [0.0; 3]);
            a.predict(x, &mut p);
            back.predict(x, &mut q);
            for c in 0..3 {
                assert!((p[c] - q[c]).abs() < 1e-5, "{p:?} {q:?}");
            }
        }
        let path = dir.join("a.state");
        a.save_state(&path).unwrap();
        let mut b = TanGateMoe::load_state(&path, 3, 1.0, 0.05, 50.0, 500, 9).unwrap();
        assert!(TanGateMoe::load_state(&path, 3, 1.0, 0.05, 51.0, 500, 9).is_err());
        assert!(back.save_state(&dir.join("back.state")).is_err());
        for (x, l) in rest {
            a.train(x, l);
            b.train(x, l);
        }
        a.refresh();
        b.refresh();
        assert_eq!(a.to_text(), b.to_text());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
