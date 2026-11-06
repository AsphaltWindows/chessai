//! Mixture of logistic-regression experts. Rust only.
//!
//! A gate, itself a logistic regression over the same features with one
//! class per expert, gives each expert a weight for the position; the
//! prediction is the experts' predictions mixed by those weights. The work
//! is split between specialised classifiers as in the BDT, but softly, so
//! there are no hard boundaries between experts for a search to exploit.
//!
//! Training is online gradient descent on the mixture's log-loss. Each
//! expert learns from an example in proportion to its responsibility for
//! the label (its share of the mixture's probability of it), towards the
//! label reweighted by its share of each class; the gate learns to predict
//! the responsibilities. Experts that start identical stay identical, so the
//! gate starts with random weights: each expert first sees a different
//! random weighting of the positions, and specialises from there.

use std::fmt::Write as _;
use std::io;
use std::path::Path;

use crate::logreg::Logreg;
use crate::textfmt::{Tokens, invalid};

#[derive(Clone, Debug)]
pub struct Moe {
    gate: Logreg,
    experts: Vec<Logreg>,
    /// L2 penalty on the gate's weights: keeps the gate from becoming
    /// steep, so positions blend between experts instead of switching.
    gate_l2: f64,
}

/// Most experts a mixture can have.
const MAX_EXPERTS: usize = 64;

impl Moe {
    /// `expert_num` untrained experts and a gate with random weights in
    /// [-gate_scale, gate_scale].
    pub fn new(categories: &[u8], expert_num: usize, rate: f64, gate_scale: f64, seed: u64) -> Self {
        Self::from_expert(Logreg::new(3, categories, rate), categories, expert_num, rate, gate_scale, seed)
    }

    /// Experts that all start as `expert`, and a random gate as in `new`.
    pub fn from_expert(
        expert: Logreg,
        categories: &[u8],
        expert_num: usize,
        rate: f64,
        gate_scale: f64,
        seed: u64,
    ) -> Self {
        assert!((1..=MAX_EXPERTS).contains(&expert_num), "1 to {MAX_EXPERTS} experts");
        let mut gate = Logreg::new(expert_num, categories, rate);
        gate.randomize(seed, gate_scale);
        Moe { gate, experts: vec![expert; expert_num], gate_l2: 0.0 }
    }

    /// Sets the L2 penalty on the gate's weights.
    pub fn with_gate_l2(mut self, l2: f64) -> Self {
        self.gate_l2 = l2;
        self
    }

    pub fn expert_num(&self) -> usize {
        self.experts.len()
    }

    /// Divides every expert's AdaGrad sums by `factor` (see
    /// `Logreg::forget`), so experts started from a trained model can still
    /// specialise.
    pub fn forget(&mut self, factor: f64) {
        self.experts.iter_mut().for_each(|e| e.forget(factor));
    }

    /// The gate's weights and each expert's prediction for a position.
    fn parts(&self, data: &[u8], gate: &mut [f64], experts: &mut [[f64; 3]]) {
        self.gate.predict(data, gate);
        for (expert, out) in self.experts.iter().zip(experts.iter_mut()) {
            expert.predict(data, out);
        }
    }

    /// Writes the probability of each class.
    pub fn predict(&self, data: &[u8], out: &mut [f64]) {
        let (mut gate, mut experts) = ([0.0; MAX_EXPERTS], [[0.0; 3]; MAX_EXPERTS]);
        let k = self.experts.len();
        self.parts(data, &mut gate[..k], &mut experts[..k]);
        for (c, o) in out.iter_mut().enumerate().take(3) {
            *o = (0..k).map(|e| gate[e] * experts[e][c]).sum();
        }
    }

    /// One gradient step on the mixture's log-loss of the example. `label`
    /// is a distribution over classes.
    pub fn train(&mut self, data: &[u8], label: &[f64]) {
        let (mut gate, mut experts) = ([0.0; MAX_EXPERTS], [[0.0; 3]; MAX_EXPERTS]);
        let k = self.experts.len();
        self.parts(data, &mut gate[..k], &mut experts[..k]);
        let mixture: [f64; 3] = std::array::from_fn(|c| (0..k).map(|e| gate[e] * experts[e][c]).sum());

        // Responsibility of each expert for the label, and the label as
        // that expert accounts for it.
        let mut responsibility = [0.0; MAX_EXPERTS];
        for e in 0..k {
            let mut target = [0.0; 3];
            for c in 0..3 {
                if label[c] > 0.0 {
                    target[c] = label[c] * gate[e] * experts[e][c] / mixture[c].max(f64::MIN_POSITIVE);
                }
            }
            let r: f64 = target.iter().sum();
            responsibility[e] = r;
            if r > 0.0 {
                target.iter_mut().for_each(|t| *t /= r);
                self.experts[e].step(data, &experts[e], &target, r, 0.0);
            }
        }
        self.gate.step(data, &gate[..k], &responsibility[..k], 1.0, self.gate_l2);
    }

    pub fn to_text(&self) -> String {
        let mut s = String::new();
        writeln!(s, "moe\n{}\n{:e}", self.experts.len(), self.gate_l2).unwrap();
        s.push_str(&self.gate.to_text());
        for expert in &self.experts {
            s.push_str(&expert.to_text());
        }
        s
    }

    pub fn from_text(text: &str) -> io::Result<Self> {
        let mut tokens = Tokens::new(text);
        if tokens.next::<String>()? != "moe" {
            return Err(invalid("not a moe model"));
        }
        let k: usize = tokens.next()?;
        if !(1..=MAX_EXPERTS).contains(&k) {
            return Err(invalid(format!("{k} experts")));
        }
        // Files from before the gate's L2 penalty have none (0).
        let gate_l2: f64 = if tokens.peek() == Some("logreg") { 0.0 } else { tokens.next()? };
        let gate = Logreg::from_tokens(&mut tokens)?;
        let experts = (0..k).map(|_| Logreg::from_tokens(&mut tokens)).collect::<io::Result<Vec<_>>>()?;
        if gate.class_num() != k || experts.iter().any(|e| e.class_num() != 3) {
            return Err(invalid("gate or expert class counts don't match"));
        }
        Ok(Moe { gate, experts, gate_l2 })
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
    /// decides the class. One logistic regression cannot fit both (it is
    /// additive), a mixture gated by feature 0 can.
    fn examples() -> Vec<([u8; 3], [f64; 3])> {
        (0..6000u32)
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

    #[test]
    fn fits_what_one_logistic_regression_cannot() {
        let mut lr = Logreg::new(3, &[2, 2, 2], 0.1);
        let mut moe = Moe::new(&[2, 2, 2], 2, 0.1, 0.5, 7);
        for _ in 0..30 {
            for (x, l) in examples() {
                lr.train(&x, &l);
                moe.train(&x, &l);
            }
        }
        let (lr_loss, moe_loss) = (log_loss(|x, p| lr.predict(x, p)), log_loss(|x, p| moe.predict(x, p)));
        assert!(moe_loss < 0.5 * lr_loss, "mixture {moe_loss}, logistic regression {lr_loss}");
    }

    #[test]
    fn text_round_trip() {
        let mut moe = Moe::new(&[2, 3], 3, 0.05, 0.1, 1).with_gate_l2(0.01);
        moe.train(&[1, 2], &[0.2, 0.3, 0.5]);
        let back = Moe::from_text(&moe.to_text()).unwrap();
        assert_eq!(back.to_text(), moe.to_text());
        let (mut a, mut b) = ([0.0; 3], [0.0; 3]);
        moe.predict(&[0, 1], &mut a);
        back.predict(&[0, 1], &mut b);
        assert_eq!(a, b);
        assert!((a.iter().sum::<f64>() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn reads_files_from_before_the_gate_penalty() {
        let moe = Moe::new(&[2, 3], 2, 0.05, 0.1, 1);
        let text = moe.to_text();
        let mut lines: Vec<&str> = text.lines().collect();
        lines.remove(2);
        let old = lines.join("\n") + "\n";
        assert_eq!(Moe::from_text(&old).unwrap().to_text(), text);
    }
}
