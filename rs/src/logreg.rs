//! Multinomial logistic regression over the categorical features. Rust only.
//!
//! Like the CNBP it has one parameter per (class, feature, value), plus a
//! bias per class, so its log-odds are a piece-square table too. But the
//! weights are fitted to predict the label (stochastic gradient descent on
//! log-loss, with AdaGrad step sizes) instead of each being one feature's
//! own evidence. Naive Bayes adds up the evidence of correlated features as
//! if it were independent, and when it learns from a model's predictions
//! that error compounds; a fitted model shares the evidence between them.

use std::fmt::Write as _;
use std::io;
use std::path::Path;

use crate::textfmt::{Tokens, invalid};

#[derive(Clone, Debug)]
pub struct Logreg {
    class_num: usize,
    categories: Vec<u8>,
    /// Index of each feature's first value in a class's weights, after the
    /// bias at index 0.
    starts: Vec<usize>,
    /// Number of weights per class.
    width: usize,
    /// Each class's weights in turn.
    weights: Vec<f64>,
    /// AdaGrad: the sum of squared gradients of each weight.
    squares: Vec<f64>,
    /// AdaGrad's base step size.
    rate: f64,
}

impl Logreg {
    pub fn new(class_num: usize, categories: &[u8], rate: f64) -> Self {
        let mut starts = Vec::with_capacity(categories.len());
        let mut width = 1;
        for &cat in categories {
            starts.push(width);
            width += cat as usize;
        }
        Logreg {
            class_num,
            categories: categories.to_vec(),
            starts,
            width,
            weights: vec![0.0; class_num * width],
            squares: vec![0.0; class_num * width],
            rate,
        }
    }

    /// Writes each class's logit (log of its unnormalised probability).
    pub fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        for (c, logit) in out.iter_mut().enumerate().take(self.class_num) {
            let w = &self.weights[c * self.width..(c + 1) * self.width];
            *logit = w[0] + self.starts.iter().zip(data).map(|(&s, &v)| w[s + v as usize]).sum::<f64>();
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
        self.step(data, p, label, 1.0, 0.0);
    }

    /// An AdaGrad step on `weight` times the log-loss of the example, given
    /// the model's current prediction `p` for it, plus an L2 penalty of
    /// `l2` / 2 times the square of each weight the example uses.
    pub(crate) fn step(&mut self, data: &[u8], p: &[f64], label: &[f64], weight: f64, l2: f64) {
        for c in 0..self.class_num {
            let g = weight * (p[c] - label[c]);
            if g == 0.0 && l2 == 0.0 {
                continue;
            }
            let base = c * self.width;
            let mut step = |i: usize| {
                let g = g + l2 * self.weights[base + i];
                if g == 0.0 {
                    return;
                }
                self.squares[base + i] += g * g;
                self.weights[base + i] -= self.rate * g / self.squares[base + i].sqrt();
            };
            step(0);
            for (&s, &v) in self.starts.iter().zip(data) {
                step(s + v as usize);
            }
        }
    }

    pub fn to_text(&self) -> String {
        let mut s = String::new();
        writeln!(s, "logreg\n{}\n{}\n{}", self.class_num, self.rate, self.categories.len()).unwrap();
        for cat in &self.categories {
            writeln!(s, "{cat}").unwrap();
        }
        // Full precision, so a reloaded model trains on exactly as before.
        for x in self.weights.iter().chain(&self.squares) {
            writeln!(s, "{x:e}").unwrap();
        }
        s
    }

    pub fn from_text(text: &str) -> io::Result<Self> {
        Self::from_tokens(&mut Tokens::new(text))
    }

    pub(crate) fn from_tokens(tokens: &mut Tokens) -> io::Result<Self> {
        if tokens.next::<String>()? != "logreg" {
            return Err(invalid("not a logreg model"));
        }
        let class_num: usize = tokens.next()?;
        let rate: f64 = tokens.next()?;
        let feature_num: usize = tokens.next()?;
        let categories = (0..feature_num).map(|_| tokens.next()).collect::<io::Result<Vec<u8>>>()?;
        let mut model = Logreg::new(class_num, &categories, rate);
        for x in model.weights.iter_mut().chain(model.squares.iter_mut()) {
            *x = tokens.next()?;
        }
        Ok(model)
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        std::fs::write(path, self.to_text())
    }

    pub fn load(path: &Path) -> io::Result<Self> {
        Self::from_text(&std::fs::read_to_string(path)?)
    }

    pub fn class_num(&self) -> usize {
        self.class_num
    }

    /// AdaGrad's base step size.
    pub fn rate(&self) -> f64 {
        self.rate
    }

    /// Sets every weight to a uniform random value in [-scale, scale], from
    /// a fixed generator seeded by `seed`.
    pub fn randomize(&mut self, seed: u64, scale: f64) {
        let mut x = seed;
        for w in &mut self.weights {
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            *w = scale * (2.0 * (x >> 11) as f64 / (1u64 << 53) as f64 - 1.0);
        }
    }

    /// Divides the AdaGrad sums by `factor`, so the next steps are larger by
    /// its square root: the model adapts faster, like a CNBP whose counts
    /// are divided by a forget factor.
    pub fn forget(&mut self, factor: f64) {
        self.squares.iter_mut().for_each(|s| *s /= factor);
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

    /// Feature 0 decides the class; features 1 and 2 are copies of it, which
    /// naive Bayes would count three times.
    fn examples() -> Vec<([u8; 4], [f64; 2])> {
        (0..4000u32)
            .map(|i| {
                let x = (i % 2) as u8;
                let noise = (i / 2 % 3) as u8;
                // Class 0 with probability 0.8 when x is 0, 0.2 when x is 1.
                let first = if x == 0 { i % 5 != 0 } else { i % 5 == 0 };
                ([x, x, x, noise], if first { [1.0, 0.0] } else { [0.0, 1.0] })
            })
            .collect()
    }

    #[test]
    fn learns_calibrated_probabilities_from_correlated_features() {
        let mut model = Logreg::new(2, &[2, 2, 2, 3], 0.1);
        for _ in 0..20 {
            for (data, label) in examples() {
                model.train(&data, &label);
            }
        }
        let mut out = [0.0; 2];
        model.predict(&[0, 0, 0, 1], &mut out);
        assert!((out[0] - 0.8).abs() < 0.03, "{out:?}");
        model.predict(&[1, 1, 1, 2], &mut out);
        assert!((out[0] - 0.2).abs() < 0.03, "{out:?}");
    }

    #[test]
    fn text_round_trip() {
        let mut model = Logreg::new(3, &[2, 3], 0.05);
        model.train(&[1, 2], &[0.2, 0.3, 0.5]);
        model.train(&[0, 1], &[1.0, 0.0, 0.0]);
        let back = Logreg::from_text(&model.to_text()).unwrap();
        assert_eq!(back.to_text(), model.to_text());
        let (mut a, mut b) = ([0.0; 3], [0.0; 3]);
        model.predict(&[1, 1], &mut a);
        back.predict(&[1, 1], &mut b);
        assert_eq!(a, b);
    }
}
