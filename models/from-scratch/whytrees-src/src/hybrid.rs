//! Hybrid: a mixture of experts of different kinds (logistic regression,
//! ordinal probit, naive Bayes) with a learned gate. Rust only.
//!
//! Unlike `moe`, whose experts are alike and learn in proportion to their
//! responsibility (so that they specialise), a hybrid's experts are
//! different models of the same thing, and each learns from every example
//! exactly as it would alone: same positions, same order, same labels. So
//! with the gate pinned to one expert (`pin`) the hybrid is that model, game
//! for game and byte for byte, which is its identity check.
//!
//! The gate is a logistic regression over the position's features with one
//! class per expert. It starts uniform (the experts differ already, so no
//! symmetry needs breaking) and learns each expert's responsibility for the
//! label, its share of the mixture's probability of it, from the experts'
//! predictions before they learn from the example; as `moe`'s gate, that is
//! a gradient step on the mixture's log-loss. The prediction mixes the
//! experts' probabilities by the gate's weights or, with `hard`, is the
//! prediction of the expert the gate weighs most, alone (ties to the first).
//!
//! With the gate target `Win` (--hybrid-gate-target win) the gate learns
//! from decisive results only, and each expert's responsibility comes from
//! its probability of the side that won given that the game was decisive,
//! P(that result) / (P(white wins) + P(black wins)): the part of a prediction
//! that log-odds play uses (ln P(win) - ln P(loss)). A draw teaches the gate
//! nothing (a label's decisive mass weighs its step). The experts learn and
//! the hybrid predicts as with the default target `Full` (each expert's share
//! of the mixture's probability of the label, draws included). The model
//! file names the target only when it is `Win` (format version 2), so a
//! `Full` hybrid's file is as before (version 1).

use std::fmt::Write as _;
use std::io;
use std::path::Path;

use crate::classifier::Classifier;
use crate::cnbp::Cnbp;
use crate::encoding::Encoded;
use crate::logreg::Logreg;
use crate::probit::Probit;
use crate::textfmt::{Tokens, invalid};

/// Most experts a hybrid can have.
pub const MAX_EXPERTS: usize = 4;

/// What the gate learns from (see the module's notes).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GateTarget {
    /// Each expert's share of the mixture's probability of the label.
    #[default]
    Full,
    /// The same for decisive results only, from each expert's probability of
    /// the winner given a decisive result; draws are skipped.
    Win,
}

/// One expert: a flat model of any of the hybrid's kinds.
#[derive(Clone, Debug)]
pub enum Expert {
    Lr(Logreg),
    Probit(Probit),
    Cnbp(Cnbp),
}

impl Expert {
    /// The kind's name, as in the model file and `--hybrid-experts`.
    pub fn kind(&self) -> &'static str {
        match self {
            Expert::Lr(_) => "lr",
            Expert::Probit(_) => "probit",
            Expert::Cnbp(_) => "cnbp",
        }
    }

    /// whytrees: the expert's own log-probabilities.
    pub fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        self.model().predict_log(data, out)
    }

    fn model(&self) -> &dyn Classifier {
        match self {
            Expert::Lr(m) => m,
            Expert::Probit(m) => m,
            Expert::Cnbp(m) => m,
        }
    }

    fn model_mut(&mut self) -> &mut dyn Classifier {
        match self {
            Expert::Lr(m) => m,
            Expert::Probit(m) => m,
            Expert::Cnbp(m) => m,
        }
    }

    /// Its probabilities while it trains: a CNBP's from its counts, as its
    /// cached ones are refreshed only after a round (`Cnbp::predict_counts`).
    fn predict_training(&self, data: &[u8], out: &mut [f64]) {
        match self {
            Expert::Cnbp(m) => m.predict_counts(data, out),
            _ => self.model().predict(data, out),
        }
    }

    /// Its text, as its own model file holds it.
    pub fn to_text(&self) -> String {
        match self {
            Expert::Lr(m) => m.to_text(),
            Expert::Probit(m) => m.to_text(),
            Expert::Cnbp(m) => m.to_text(),
        }
    }

    /// Reads an expert of kind `kind` written by `to_text`; a CNBP predicts
    /// from cached probabilities with `use_probs`, as when loaded alone.
    fn from_tokens(kind: &str, tokens: &mut Tokens, use_probs: bool) -> io::Result<Self> {
        Ok(match kind {
            "lr" => Expert::Lr(Logreg::from_tokens(tokens)?),
            "probit" => Expert::Probit(Probit::from_tokens(tokens)?),
            "cnbp" => {
                let class_num: usize = tokens.next()?;
                let alpha: f64 = tokens.next()?;
                let cat_num: usize = tokens.next()?;
                let categories = (0..cat_num).map(|_| tokens.next::<u8>()).collect::<io::Result<Vec<_>>>()?;
                let mut cnbp = Cnbp::new(class_num, &categories, alpha, use_probs);
                cnbp.read_counts(tokens)?;
                Expert::Cnbp(cnbp)
            }
            _ => return Err(invalid(format!("unknown hybrid expert kind {kind:?}"))),
        })
    }
}

#[derive(Clone, Debug)]
pub struct Hybrid {
    experts: Vec<Expert>,
    gate: Logreg,
    /// Predict with the most-weighted expert alone.
    hard: bool,
    /// Predict with this expert alone, whatever the gate says (identity
    /// checks); the gate still learns.
    pin: Option<usize>,
    /// What the gate learns from.
    target: GateTarget,
}

impl Hybrid {
    /// A hybrid of `experts` (2 to `MAX_EXPERTS`, untrained or not) with a
    /// new uniform gate over `categories` of AdaGrad step `gate_rate`.
    pub fn new(experts: Vec<Expert>, categories: &[u8], gate_rate: f64, hard: bool, pin: Option<usize>) -> Self {
        assert!((2..=MAX_EXPERTS).contains(&experts.len()), "2 to {MAX_EXPERTS} experts");
        assert!(pin.is_none_or(|e| e < experts.len()), "pin an expert the hybrid has");
        let gate = Logreg::new(experts.len(), categories, gate_rate);
        Hybrid { experts, gate, hard, pin, target: GateTarget::Full }
    }

    /// The same hybrid with its gate learning from `target`.
    pub fn with_gate_target(mut self, target: GateTarget) -> Self {
        self.target = target;
        self
    }

    pub fn gate_target(&self) -> GateTarget {
        self.target
    }

    pub fn expert_num(&self) -> usize {
        self.experts.len()
    }

    pub fn experts(&self) -> &[Expert] {
        &self.experts
    }

    /// The gate's weights and each expert's probabilities for a position.
    fn parts(&self, data: &[u8], gate: &mut [f64], experts: &mut [[f64; 3]]) {
        self.gate.predict(data, gate);
        for (expert, out) in self.experts.iter().zip(experts.iter_mut()) {
            expert.model().predict(data, out);
        }
    }

    /// The expert that predicts alone (pinned, or the gate's choice when
    /// hard), if any.
    fn alone(&self, data: &[u8]) -> Option<usize> {
        if self.pin.is_some() {
            return self.pin;
        }
        if !self.hard {
            return None;
        }
        let k = self.experts.len();
        let mut gate = [0.0; MAX_EXPERTS];
        self.gate.predict(data, &mut gate[..k]);
        Some((1..k).fold(0, |best, e| if gate[e] > gate[best] { e } else { best }))
    }

    pub fn predict(&self, data: &[u8], out: &mut [f64]) {
        if let Some(e) = self.alone(data) {
            return self.experts[e].model().predict(data, out);
        }
        let (mut gate, mut experts) = ([0.0; MAX_EXPERTS], [[0.0; 3]; MAX_EXPERTS]);
        let k = self.experts.len();
        self.parts(data, &mut gate[..k], &mut experts[..k]);
        for (c, o) in out.iter_mut().enumerate().take(3) {
            *o = (0..k).map(|e| gate[e] * experts[e][c]).sum();
        }
    }

    /// Normalised log-probabilities: each expert's `predict_log` normalised
    /// and mixed in log space, or the lone expert's own `predict_log`.
    pub fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        if let Some(e) = self.alone(data) {
            return self.experts[e].model().predict_log(data, out);
        }
        let k = self.experts.len();
        let mut gate = [0.0; MAX_EXPERTS];
        self.gate.predict(data, &mut gate[..k]);
        let mut logs = [[0.0; 3]; MAX_EXPERTS];
        for (expert, l) in self.experts.iter().zip(logs.iter_mut()) {
            expert.model().predict_log(data, l);
            let z = log_sum_exp(l);
            l.iter_mut().for_each(|x| *x -= z);
        }
        for (c, o) in out.iter_mut().enumerate().take(3) {
            let terms: [f64; MAX_EXPERTS] =
                std::array::from_fn(|e| if e < k { gate[e].max(f64::MIN_POSITIVE).ln() + logs[e][c] } else { f64::NEG_INFINITY });
            *o = log_sum_exp(&terms[..k]);
        }
    }

    /// The gate's weights for a position, or with a label each expert's
    /// responsibility for it, as training computes them (for diagnostics).
    pub fn component_weights(&self, data: &[u8], label: Option<&[f64]>, out: &mut [f64]) {
        let (mut gate, mut experts) = ([0.0; MAX_EXPERTS], [[0.0; 3]; MAX_EXPERTS]);
        let k = self.experts.len();
        self.parts(data, &mut gate[..k], &mut experts[..k]);
        match label {
            None => out[..k].copy_from_slice(&gate[..k]),
            Some(label) => {
                if !self.targets(&gate[..k], &experts[..k], label, &mut out[..k]) {
                    out[..k].iter_mut().for_each(|o| *o = 0.0);
                }
            }
        }
    }

    /// The gate's target for a label, into `out`, and its step's weight: the
    /// responsibilities (weight 1), or with `Win` those for the label's
    /// decisive part (weight its decisive mass; 0, no step, for a draw).
    fn targets(&self, gate: &[f64], experts: &[[f64; 3]], label: &[f64], out: &mut [f64]) -> bool {
        self.target_weight(gate, experts, label, out) > 0.0
    }

    fn target_weight(&self, gate: &[f64], experts: &[[f64; 3]], label: &[f64], out: &mut [f64]) -> f64 {
        match self.target {
            GateTarget::Full => {
                responsibilities(gate, experts, label, out);
                1.0
            }
            GateTarget::Win => {
                let decisive = label[0] + label[1];
                if decisive <= 0.0 {
                    return 0.0;
                }
                let winner = [label[0] / decisive, label[1] / decisive, 0.0];
                let given: [[f64; 3]; MAX_EXPERTS] = std::array::from_fn(|e| {
                    if e >= experts.len() {
                        return [0.0; 3];
                    }
                    let p = &experts[e];
                    let s = (p[0] + p[1]).max(f64::MIN_POSITIVE);
                    [p[0] / s, p[1] / s, 0.0]
                });
                responsibilities(gate, &given[..experts.len()], &winner, out);
                decisive
            }
        }
    }

    /// The gate's step on an example, from the experts' predictions before
    /// they learn from it.
    fn learn_gate(&mut self, data: &[u8], label: &[f64]) {
        let (mut gate, mut experts) = ([0.0; MAX_EXPERTS], [[0.0; 3]; MAX_EXPERTS]);
        let k = self.experts.len();
        self.gate.predict(data, &mut gate[..k]);
        for (expert, out) in self.experts.iter().zip(experts.iter_mut()) {
            expert.predict_training(data, out);
        }
        let mut target = [0.0; MAX_EXPERTS];
        let weight = self.target_weight(&gate[..k], &experts[..k], label, &mut target[..k]);
        if weight > 0.0 {
            self.gate.step(data, &gate[..k], &target[..k], weight, 0.0);
        }
    }

    pub fn train(&mut self, data: &[u8], label: &[f64]) {
        self.learn_gate(data, label);
        self.experts.iter_mut().for_each(|e| e.model_mut().train(data, label));
    }

    pub fn train_in_game(&mut self, data: &[u8], label: &[f64], game: u64) {
        self.learn_gate(data, label);
        self.experts.iter_mut().for_each(|e| e.model_mut().train_in_game(data, label, game));
    }

    pub fn observe_games(&mut self, games: &[&[Encoded]]) {
        self.experts.iter_mut().for_each(|e| e.model_mut().observe_games(games));
    }

    pub fn needs_games(&self) -> bool {
        self.experts.iter().any(|e| e.model().needs_games())
    }

    pub fn refresh(&mut self) {
        self.experts.iter_mut().for_each(|e| e.model_mut().refresh());
    }

    /// Every expert forgets as it would alone (see `Classifier::forget`);
    /// the gate keeps its sums.
    pub fn forget(&mut self, factor: f64) {
        self.experts.iter_mut().for_each(|e| e.model_mut().forget(factor));
    }

    pub fn to_text(&self) -> String {
        let mut s = String::new();
        let pin = self.pin.map_or("none".to_string(), |e| e.to_string());
        match self.target {
            GateTarget::Full => writeln!(s, "hybrid\n1\n{}\n{}\n{pin}", self.experts.len(), self.hard as u8).unwrap(),
            GateTarget::Win => writeln!(s, "hybrid\n2\n{}\n{}\n{pin}\nwin", self.experts.len(), self.hard as u8).unwrap(),
        }
        for expert in &self.experts {
            writeln!(s, "{}", expert.kind()).unwrap();
        }
        s.push_str(&self.gate.to_text());
        for expert in &self.experts {
            s.push_str(&expert.to_text());
        }
        s
    }

    pub fn from_text(text: &str, use_probs: bool) -> io::Result<Self> {
        let mut tokens = Tokens::new(text);
        if tokens.next::<String>()? != "hybrid" {
            return Err(invalid("not a hybrid model"));
        }
        let version = tokens.next::<u32>()?;
        if version != 1 && version != 2 {
            return Err(invalid("unknown hybrid model version"));
        }
        let k: usize = tokens.next()?;
        if !(2..=MAX_EXPERTS).contains(&k) {
            return Err(invalid(format!("{k} experts")));
        }
        let hard = match tokens.next::<u8>()? {
            0 => false,
            1 => true,
            h => return Err(invalid(format!("hard {h}"))),
        };
        let pin = match tokens.next::<String>()?.as_str() {
            "none" => None,
            e => Some(e.parse::<usize>().map_err(|err| invalid(format!("{err}: {e:?}")))?),
        };
        let target = match version {
            1 => GateTarget::Full,
            _ => match tokens.next::<String>()?.as_str() {
                "win" => GateTarget::Win,
                t => return Err(invalid(format!("unknown gate target {t:?}"))),
            },
        };
        let kinds = (0..k).map(|_| tokens.next::<String>()).collect::<io::Result<Vec<_>>>()?;
        let gate = Logreg::from_tokens(&mut tokens)?;
        let experts =
            kinds.iter().map(|kind| Expert::from_tokens(kind, &mut tokens, use_probs)).collect::<io::Result<Vec<_>>>()?;
        if tokens.peek().is_some() {
            return Err(invalid("hybrid model file has extra values"));
        }
        if gate.class_num() != k || pin.is_some_and(|e| e >= k) {
            return Err(invalid("the gate's classes or the pin don't match the experts"));
        }
        Ok(Hybrid { experts, gate, hard, pin, target })
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        std::fs::write(path, self.to_text())
    }

    pub fn load(path: &Path, use_probs: bool) -> io::Result<Self> {
        Self::from_text(&std::fs::read_to_string(path)?, use_probs)
    }
}

/// Each expert's share of the mixture's probability of the label (the label
/// a distribution over classes).
fn responsibilities(gate: &[f64], experts: &[[f64; 3]], label: &[f64], out: &mut [f64]) {
    let mixture: [f64; 3] = std::array::from_fn(|c| gate.iter().zip(experts).map(|(g, p)| g * p[c]).sum());
    for (e, o) in out.iter_mut().enumerate() {
        *o = (0..3)
            .filter(|&c| label[c] > 0.0)
            .map(|c| label[c] * gate[e] * experts[e][c] / mixture[c].max(f64::MIN_POSITIVE))
            .sum();
    }
}

fn log_sum_exp(x: &[f64]) -> f64 {
    let max = x.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if max == f64::NEG_INFINITY {
        return max;
    }
    max + x.iter().map(|v| (v - max).exp()).sum::<f64>().ln()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probit::ProbitParams;

    const CATS: [u8; 3] = [2, 2, 2];

    /// Feature 0 picks one of two regimes; a different feature decides the
    /// result in each (as `moe`'s tests), with some draws.
    fn examples() -> Vec<([u8; 3], [f64; 3])> {
        (0..3000u32)
            .map(|i| {
                let regime = (i % 2) as u8;
                let (a, b) = ((i / 2 % 2) as u8, (i / 4 % 2) as u8);
                let decider = if regime == 0 { a } else { b };
                let label = match (decider, i % 7) {
                    (_, 0) => [0.0, 0.0, 1.0],
                    (0, _) => [1.0, 0.0, 0.0],
                    _ => [0.0, 1.0, 0.0],
                };
                ([regime, a, b], label)
            })
            .collect()
    }

    fn lr() -> Expert {
        Expert::Lr(Logreg::new(3, &CATS, 0.1))
    }

    fn probit() -> Expert {
        Expert::Probit(Probit::new(&CATS, ProbitParams { tau: 0.0, ..ProbitParams::default() }))
    }

    fn cnbp() -> Expert {
        Expert::Cnbp(Cnbp::new(3, &CATS, 1.0, true))
    }

    fn trained(mut h: Hybrid) -> Hybrid {
        for _ in 0..3 {
            for (x, l) in examples() {
                h.train(&x, &l);
            }
            h.refresh();
        }
        h
    }

    /// Each expert ends as the same model trained alone on the same stream.
    #[test]
    fn experts_learn_as_alone() {
        let h = trained(Hybrid::new(vec![lr(), probit(), cnbp()], &CATS, 0.05, false, None));
        let (mut a, mut b, mut c) = (Logreg::new(3, &CATS, 0.1), Probit::new(&CATS, ProbitParams { tau: 0.0, ..ProbitParams::default() }), Cnbp::new(3, &CATS, 1.0, true));
        for _ in 0..3 {
            for (x, l) in examples() {
                a.train(&x, &l);
                b.train(&x, &l);
                c.train(&x, &l);
            }
            c.refresh();
        }
        assert_eq!(h.experts[0].to_text(), a.to_text());
        assert_eq!(h.experts[1].to_text(), b.to_text());
        assert_eq!(h.experts[2].to_text(), c.to_text());
    }

    /// Pinned, it predicts as the pinned expert, bit for bit.
    #[test]
    fn pinned_is_the_expert() {
        for pin in 0..2 {
            let h = trained(Hybrid::new(vec![lr(), probit()], &CATS, 0.05, false, Some(pin)));
            for (x, _) in examples().iter().take(8) {
                let (mut p, mut q, mut lp, mut lq) = ([0.0; 3], [0.0; 3], [0.0; 3], [0.0; 3]);
                h.predict(x, &mut p);
                h.experts[pin].model().predict(x, &mut q);
                h.predict_log(x, &mut lp);
                h.experts[pin].model().predict_log(x, &mut lq);
                assert_eq!(p, q);
                assert_eq!(lp, lq);
            }
        }
    }

    /// The mixture's probabilities sum to one, its log-probabilities agree
    /// with them, and hard picks one expert's prediction.
    #[test]
    fn mixture_and_hard() {
        let h = trained(Hybrid::new(vec![lr(), probit()], &CATS, 0.05, false, None));
        let mut hard = h.clone();
        hard.hard = true;
        for (x, _) in examples().iter().take(8) {
            let (mut p, mut l) = ([0.0; 3], [0.0; 3]);
            h.predict(x, &mut p);
            h.predict_log(x, &mut l);
            assert!((p.iter().sum::<f64>() - 1.0).abs() < 1e-12);
            for c in 0..3 {
                assert!((l[c].exp() - p[c]).abs() < 1e-9, "{l:?} {p:?}");
            }
            let (mut q, mut e0, mut e1) = ([0.0; 3], [0.0; 3], [0.0; 3]);
            hard.predict(x, &mut q);
            h.experts[0].model().predict(x, &mut e0);
            h.experts[1].model().predict(x, &mut e1);
            assert!(q == e0 || q == e1);
        }
    }

    /// The gate learns to prefer the better expert: a logistic regression
    /// that learns, against one that barely does (rate 1e-6).
    #[test]
    fn gate_prefers_the_better_expert() {
        let weak = Expert::Lr(Logreg::new(3, &CATS, 1e-6));
        let h = trained(Hybrid::new(vec![weak, lr()], &CATS, 0.1, false, None));
        let mut w = [0.0; 2];
        let mean: f64 = examples().iter().map(|(x, _)| {
            h.component_weights(x, None, &mut w);
            w[1]
        }).sum::<f64>() / 3000.0;
        assert!(mean > 0.8, "weight on the learning expert {mean}");
    }

    /// With the win target a draw leaves the gate as it was, the experts
    /// learn as with the default, and the file round-trips with its target;
    /// the default's file is version 1, as before.
    #[test]
    fn win_target() {
        let full = Hybrid::new(vec![lr(), probit()], &CATS, 0.05, false, None);
        let win = full.clone().with_gate_target(GateTarget::Win);
        let mut probe = win.clone();
        let before = probe.gate.to_text();
        probe.train(&[0, 1, 0], &[0.0, 0.0, 1.0]);
        assert_eq!(probe.gate.to_text(), before, "a draw moved the gate");
        let (f, w) = (trained(full), trained(win));
        for e in 0..2 {
            assert_eq!(f.experts[e].to_text(), w.experts[e].to_text());
        }
        assert_ne!(f.gate.to_text(), w.gate.to_text());
        assert!(f.to_text().starts_with("hybrid\n1\n"));
        assert!(w.to_text().starts_with("hybrid\n2\n2\n0\nnone\nwin\n"));
        let back = Hybrid::from_text(&w.to_text(), true).unwrap();
        assert_eq!(back.gate_target(), GateTarget::Win);
        assert_eq!(back.to_text(), w.to_text());
        assert_eq!(Hybrid::from_text(&f.to_text(), true).unwrap().gate_target(), GateTarget::Full);
        // the responsibilities for a decisive label: the expert surer of the winner, given decisive, gets more
        let mut out = [0.0; 2];
        let n = w.target_weight(&[0.5, 0.5], &[[0.2, 0.1, 0.7], [0.3, 0.3, 0.4]], &[1.0, 0.0, 0.0], &mut out);
        assert_eq!(n, 1.0);
        assert!((out[0] - (2.0 / 3.0) / (2.0 / 3.0 + 0.5)).abs() < 1e-12, "{out:?}");
    }

    #[test]
    fn text_round_trip() {
        for (experts, hard, pin) in [(vec![lr(), probit()], false, None), (vec![lr(), cnbp()], true, Some(1)), (vec![probit(), cnbp(), lr()], false, Some(2))] {
            let h = trained(Hybrid::new(experts, &CATS, 0.05, hard, pin));
            let back = Hybrid::from_text(&h.to_text(), true).unwrap();
            assert_eq!(back.to_text(), h.to_text());
            let (mut a, mut b) = ([0.0; 3], [0.0; 3]);
            for (x, _) in examples().iter().take(8) {
                h.predict_log(x, &mut a);
                back.predict_log(x, &mut b);
                // A CNBP's text rounds its counts to 6 decimals, as alone.
                for c in 0..3 {
                    assert!((a[c] - b[c]).abs() < 1e-6, "{a:?} {b:?}");
                }
            }
        }
    }
}
