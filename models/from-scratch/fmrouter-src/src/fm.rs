//! Factorization machine (Rendle 2010): logistic regression plus every
//! pairwise interaction of the categorical features. Each (feature, value)
//! has a vector of `factors` weights per class, and two features present
//! together add the dot product of their vectors to the class's logit. No
//! pair is chosen by hand, and the low rank keeps a prediction at
//! O(features x factors): with ½(|Σv|² - Σ|v|²) for the sum over pairs.
//! Rust only.

use std::fmt::Write as _;
use std::io;
use std::path::Path;

use crate::logreg::Logreg;
use crate::state::{Reader, Writer};
use crate::textfmt::{Tokens, invalid};

/// Most factors a model can have (the per-class sums live on the stack).
pub const MAX_FACTORS: usize = 32;

#[derive(Clone, Debug)]
pub struct Fm {
    /// The bias and one weight per (class, feature, value).
    linear: Logreg,
    class_num: usize,
    categories: Vec<u8>,
    /// Index of each feature's first value among all the features' values.
    starts: Vec<usize>,
    /// Number of values of all the features together.
    values: usize,
    factors: usize,
    /// The vector of class c and value u starts at `(c * values + u) * factors`.
    vectors: Vec<f64>,
    /// AdaGrad: the sum of squared gradients of each vector weight.
    squares: Vec<f64>,
    rate: f64,
}

impl Fm {
    /// A model whose vectors are uniform in [-scale, scale], from a fixed
    /// generator seeded by `seed`. All-zero vectors would never learn: each
    /// one's gradient is the sum of the others.
    pub fn new(class_num: usize, categories: &[u8], factors: usize, rate: f64, scale: f64, seed: u64) -> Self {
        assert!((1..=MAX_FACTORS).contains(&factors), "factors must be 1 to {MAX_FACTORS}");
        let mut starts = Vec::with_capacity(categories.len());
        let mut values = 0;
        for &cat in categories {
            starts.push(values);
            values += cat as usize;
        }
        let n = class_num * values * factors;
        let mut x = seed;
        let vectors = (0..n)
            .map(|_| {
                x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                scale * (2.0 * (x >> 11) as f64 / (1u64 << 53) as f64 - 1.0)
            })
            .collect();
        Fm {
            linear: Logreg::new(class_num, categories, rate),
            class_num,
            categories: categories.to_vec(),
            starts,
            values,
            factors,
            vectors,
            squares: vec![0.0; n],
            rate,
        }
    }

    /// Class `c`'s interaction term for the position; writes the sum of its
    /// vectors into `sum`.
    fn interactions(&self, c: usize, data: &[u8], sum: &mut [f64]) -> f64 {
        sum.iter_mut().for_each(|s| *s = 0.0);
        let mut squares = 0.0;
        for (&start, &value) in self.starts.iter().zip(data) {
            let at = (c * self.values + start + value as usize) * self.factors;
            for (s, &w) in sum.iter_mut().zip(&self.vectors[at..at + self.factors]) {
                *s += w;
                squares += w * w;
            }
        }
        0.5 * (sum.iter().map(|s| s * s).sum::<f64>() - squares)
    }

    /// Writes each class's logit (log of its unnormalised probability).
    pub fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        self.linear.predict_log(data, out);
        let mut sum = [0.0; MAX_FACTORS];
        for (c, logit) in out.iter_mut().enumerate().take(self.class_num) {
            *logit += self.interactions(c, data, &mut sum[..self.factors]);
        }
    }

    /// Writes the probability of each class.
    pub fn predict(&self, data: &[u8], out: &mut [f64]) {
        self.predict_log(data, out);
        softmax(&mut out[..self.class_num]);
    }

    /// One AdaGrad step on the log-loss of the example. `label` is a
    /// distribution over classes.
    pub fn train(&mut self, data: &[u8], label: &[f64]) {
        let mut p = [0.0; 256];
        let p = &mut p[..self.class_num];
        self.predict(data, p);
        self.linear.step(data, p, label, 1.0, 0.0);
        let k = self.factors;
        let mut sum = [0.0; MAX_FACTORS];
        for c in 0..self.class_num {
            let g = p[c] - label[c];
            if g == 0.0 {
                continue;
            }
            self.interactions(c, data, &mut sum[..k]);
            for (&start, &value) in self.starts.iter().zip(data) {
                let at = (c * self.values + start + value as usize) * k;
                for f in 0..k {
                    // The logit's derivative by this weight: the other vectors' sum.
                    let grad = g * (sum[f] - self.vectors[at + f]);
                    if grad == 0.0 {
                        continue;
                    }
                    self.squares[at + f] += grad * grad;
                    self.vectors[at + f] -= self.rate * grad / self.squares[at + f].sqrt();
                }
            }
        }
    }

    /// Divides the AdaGrad sums by `factor` (see `Logreg::forget`).
    pub fn forget(&mut self, factor: f64) {
        self.linear.forget(factor);
        self.squares.iter_mut().for_each(|s| *s /= factor);
    }

    pub fn class_num(&self) -> usize {
        self.class_num
    }

    /// AdaGrad's base step size (of the vectors and of the linear part).
    pub fn rate(&self) -> f64 {
        self.rate
    }

    /// Length of each feature value's interaction vector (per class).
    pub fn factors(&self) -> usize {
        self.factors
    }

    pub fn to_text(&self) -> String {
        let mut s = String::new();
        writeln!(s, "fm\n{}\n{}\n{}\n{}", self.class_num, self.rate, self.factors, self.categories.len()).unwrap();
        for cat in &self.categories {
            writeln!(s, "{cat}").unwrap();
        }
        // Full precision, so a reloaded model trains on exactly as before.
        for x in self.vectors.iter().chain(&self.squares) {
            writeln!(s, "{x:e}").unwrap();
        }
        s.push_str(&self.linear.to_text());
        s
    }

    pub fn from_text(text: &str) -> io::Result<Self> {
        Self::from_tokens(&mut Tokens::new(text))
    }

    /// Parses a model written by `to_text` from `tokens` (a delegation
    /// tree's file holds its FM routers so).
    pub(crate) fn from_tokens(tokens: &mut Tokens) -> io::Result<Self> {
        if tokens.next::<String>()? != "fm" {
            return Err(invalid("not a factorization machine model"));
        }
        let class_num: usize = tokens.next()?;
        let rate: f64 = tokens.next()?;
        let factors: usize = tokens.next()?;
        let feature_num: usize = tokens.next()?;
        let categories = (0..feature_num).map(|_| tokens.next()).collect::<io::Result<Vec<u8>>>()?;
        if !(1..=MAX_FACTORS).contains(&factors) {
            return Err(invalid("factorization machine factors out of range"));
        }
        let mut model = Fm::new(class_num, &categories, factors, rate, 0.0, 0);
        for x in model.vectors.iter_mut().chain(model.squares.iter_mut()) {
            *x = tokens.next()?;
        }
        model.linear = Logreg::from_tokens(tokens)?;
        Ok(model)
    }

    /// Writes the whole model for a tree's state file (see `state`).
    pub(crate) fn write_state(&self, w: &mut Writer) {
        w.usize(self.class_num);
        w.f64(self.rate);
        w.usize(self.factors);
        w.u8s(&self.categories);
        w.f64s(&self.vectors);
        w.f64s(&self.squares);
        self.linear.write_state(w);
    }

    /// Reads a model written by `write_state`.
    pub(crate) fn read_state(r: &mut Reader) -> io::Result<Self> {
        let class_num = r.usize()?;
        let rate = r.f64()?;
        let factors = r.usize()?;
        let categories = r.u8s()?;
        if !(1..=MAX_FACTORS).contains(&factors) {
            return Err(invalid("factorization machine factors out of range in state file"));
        }
        let mut model = Fm::new(class_num, &categories, factors, rate, 0.0, 0);
        let (vectors, squares) = (r.f64s()?, r.f64s()?);
        if vectors.len() != model.vectors.len() || squares.len() != model.squares.len() {
            return Err(invalid("factorization machine state of the wrong shape"));
        }
        model.vectors = vectors;
        model.squares = squares;
        model.linear = Logreg::read_state(r)?;
        if model.linear.class_num() != class_num {
            return Err(invalid("factorization machine state of the wrong shape"));
        }
        Ok(model)
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        std::fs::write(path, self.to_text())
    }

    pub fn load(path: &Path) -> io::Result<Self> {
        Self::from_text(&std::fs::read_to_string(path)?)
    }
}

fn softmax(x: &mut [f64]) {
    let max = x.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    x.iter_mut().for_each(|v| *v = (*v - max).exp());
    let sum: f64 = x.iter().sum();
    x.iter_mut().for_each(|v| *v /= sum);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The class is the XOR of features 0 and 1 (feature 2 is noise): no
    /// weight per feature value can predict it, a pairwise interaction can.
    fn xor_examples() -> Vec<([u8; 3], [f64; 2])> {
        (0..400u32)
            .map(|i| {
                let (a, b) = ((i % 2) as u8, (i / 2 % 2) as u8);
                ([a, b, (i / 4 % 3) as u8], if a == b { [1.0, 0.0] } else { [0.0, 1.0] })
            })
            .collect()
    }

    fn mean_log_loss(predict: impl Fn(&[u8], &mut [f64]), examples: &[([u8; 3], [f64; 2])]) -> f64 {
        let mut p = [0.0; 2];
        examples
            .iter()
            .map(|(data, label)| {
                predict(data, &mut p);
                -(if label[0] == 1.0 { p[0] } else { p[1] }).ln()
            })
            .sum::<f64>()
            / examples.len() as f64
    }

    #[test]
    fn learns_an_interaction_logistic_regression_cannot() {
        let examples = xor_examples();
        let mut fm = Fm::new(2, &[2, 2, 3], 4, 0.1, 0.1, 7);
        let mut lr = Logreg::new(2, &[2, 2, 3], 0.1);
        for _ in 0..30 {
            for (data, label) in &examples {
                fm.train(data, label);
                lr.train(data, label);
            }
        }
        let fm_loss = mean_log_loss(|d, p| fm.predict(d, p), &examples);
        let lr_loss = mean_log_loss(|d, p| lr.predict(d, p), &examples);
        assert!(lr_loss > 0.65, "logistic regression should stay near chance: {lr_loss}");
        assert!(fm_loss < 0.1, "the factorization machine should learn XOR: {fm_loss}");
    }

    #[test]
    fn text_round_trip_and_forgetting_keep_predictions() {
        let mut fm = Fm::new(3, &[2, 2, 3], 3, 0.05, 0.1, 1);
        for (data, label) in xor_examples() {
            fm.train(&data, &[label[0], label[1], 0.0]);
        }
        let loaded = Fm::from_text(&fm.to_text()).unwrap();
        assert_eq!(loaded.to_text(), fm.to_text());
        let mut forgotten = fm.clone();
        forgotten.forget(10.0);
        let (mut p, mut q, mut r) = ([0.0; 3], [0.0; 3], [0.0; 3]);
        for (data, _) in xor_examples().iter().take(12) {
            fm.predict(data, &mut p);
            loaded.predict(data, &mut q);
            forgotten.predict(data, &mut r);
            assert_eq!(p, q);
            assert_eq!(p, r);
        }
    }
}
