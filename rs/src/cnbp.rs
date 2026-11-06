//! Categorical Naive Bayes with Partial labels, a port of
//! `c_src/models/cnbp/cnbp.c`.
//!
//! Labels are probability distributions over classes rather than single
//! classes: a label of `[0.7, 0.0, 0.3]` adds 0.7 to class 0's counts and
//! 0.3 to class 2's. Predictions use Laplace smoothing with `alpha`.
//!
//! The arithmetic is kept in the same order as the C version so predictions
//! match it exactly, and models are read and written in its text format.

use std::fmt::Write as _;
use std::io;
use std::path::Path;

use crate::textfmt::Tokens;

#[derive(Clone, Debug)]
pub struct Cnbp {
    categories: Vec<u8>,
    /// Offset of each category's values within one class's block of counts.
    cat_offsets: Vec<usize>,
    total_cat_vals: usize,
    class_num: usize,
    alpha: f64,
    use_probs: bool,
    class_totals: Vec<f64>,
    /// `class_num` blocks of `total_cat_vals` counts.
    class_cat_totals: Vec<f64>,
    /// Probabilities cached from the counts, and their logs, used when
    /// `use_probs` is set.
    class_probs: Vec<f64>,
    class_cat_probs: Vec<f64>,
    class_logs: Vec<f64>,
    class_cat_logs: Vec<f64>,
    fresh: bool,
}

impl Cnbp {
    pub fn new(class_num: usize, categories: &[u8], alpha: f64, use_probs: bool) -> Self {
        let mut cat_offsets = Vec::with_capacity(categories.len());
        let mut total_cat_vals = 0;
        for &cat in categories {
            cat_offsets.push(total_cat_vals);
            total_cat_vals += cat as usize;
        }

        let mut cnbp = Cnbp {
            categories: categories.to_vec(),
            cat_offsets,
            total_cat_vals,
            class_num,
            alpha,
            use_probs,
            class_totals: vec![0.0; class_num],
            class_cat_totals: vec![0.0; class_num * total_cat_vals],
            class_probs: vec![0.0; class_num],
            class_cat_probs: vec![0.0; class_num * total_cat_vals],
            class_logs: vec![0.0; class_num],
            class_cat_logs: vec![0.0; class_num * total_cat_vals],
            fresh: false,
        };
        cnbp.refresh();
        cnbp
    }

    pub fn class_num(&self) -> usize {
        self.class_num
    }

    fn idx(&self, class: usize, cat: usize, val: u8) -> usize {
        class * self.total_cat_vals + self.cat_offsets[cat] + val as usize
    }

    /// Adds one data point. `label` must sum to 1.
    pub fn train(&mut self, data: &[u8], label: &[f64]) {
        self.train_weighted(data, label, 1.0);
    }

    /// Adds one data point counted `weight` times (a fraction for a partial
    /// example). `label` must sum to 1.
    pub fn train_weighted(&mut self, data: &[u8], label: &[f64], weight: f64) {
        let sum: f64 = label.iter().sum();
        assert!(
            (sum - 1.0).abs() <= 0.000001,
            "cnbp label must sum to 1, got {sum}"
        );

        for (cl, block) in self.class_cat_totals.chunks_exact_mut(self.total_cat_vals).enumerate() {
            let l = label[cl] * weight;
            self.class_totals[cl] += l;
            for (&off, &val) in self.cat_offsets.iter().zip(data) {
                block[off + val as usize] += l;
            }
        }
        self.fresh = false;
    }

    /// Divides all counts by `factor`, keeping predictions roughly the same
    /// but letting new training move them faster.
    pub fn forget(&mut self, factor: f64) {
        for t in self.class_totals.iter_mut().chain(self.class_cat_totals.iter_mut()) {
            *t /= factor;
        }
        self.fresh = false;
        self.refresh();
    }

    /// Recomputes the cached probabilities after training. Only needed when
    /// `use_probs` is set; a no-op otherwise.
    pub fn refresh(&mut self) {
        if !self.use_probs || self.fresh {
            return;
        }

        let cur_total: f64 = self.class_totals.iter().sum();
        let class_num = self.class_num as f64;

        for cl in 0..self.class_num {
            self.class_probs[cl] =
                (self.class_totals[cl] + self.alpha) / (cur_total + class_num * self.alpha);
            self.class_logs[cl] = self.class_probs[cl].ln();

            for cat in 0..self.categories.len() {
                let denom = self.class_totals[cl] + self.categories[cat] as f64 * self.alpha;
                for val in 0..self.categories[cat] {
                    let i = self.idx(cl, cat, val);
                    self.class_cat_probs[i] = (self.alpha + self.class_cat_totals[i]) / denom;
                    self.class_cat_logs[i] = self.class_cat_probs[i].ln();
                }
            }
        }
        self.fresh = true;
    }

    /// Writes the probability of each class into `out`.
    ///
    /// # Panics
    /// With `use_probs`, if the model was trained since the last `refresh()`.
    pub fn predict(&self, data: &[u8], out: &mut [f64]) {
        if !self.use_probs {
            return self.predict_counts(data, out);
        }
        assert!(self.fresh, "cnbp trained without refresh() before predict()");

        let mut total = 0.0;
        for (cl, block) in self.class_cat_probs.chunks_exact(self.total_cat_vals).enumerate() {
            let mut p = self.class_probs[cl];
            for (&off, &val) in self.cat_offsets.iter().zip(data) {
                p *= block[off + val as usize];
            }
            out[cl] = p;
            total += p;
        }
        for p in &mut out[..self.class_num] {
            *p /= total;
        }
    }

    /// Predicts from the raw counts, ignoring `use_probs` and the cached
    /// probabilities. Cheaper while a model is still being trained.
    pub fn predict_counts(&self, data: &[u8], out: &mut [f64]) {
        let class_total: f64 = self.class_totals.iter().sum();
        let class_num = self.class_num as f64;

        let mut total = 0.0;
        for (cl, block) in self.class_cat_totals.chunks_exact(self.total_cat_vals).enumerate() {
            let class_count = self.class_totals[cl];
            let mut p = (class_count + self.alpha) / (class_total + class_num * self.alpha);
            for ((&off, &cat), &val) in self.cat_offsets.iter().zip(&self.categories).zip(data) {
                p *= (block[off + val as usize] + self.alpha) / (class_count + cat as f64 * self.alpha);
            }
            out[cl] = p;
            total += p;
        }
        for p in &mut out[..self.class_num] {
            *p /= total;
        }
    }

    /// Writes ln P(class) + sum of ln P(value | class) over the features for
    /// each class: the log of the unnormalised posterior. Differences between
    /// classes are log-odds. Unlike `predict`, it cannot underflow.
    ///
    /// # Panics
    /// With `use_probs`, if the model was trained since the last `refresh()`.
    pub fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        if self.use_probs {
            assert!(self.fresh, "cnbp trained without refresh() before predict_log()");
            for (cl, block) in self.class_cat_logs.chunks_exact(self.total_cat_vals).enumerate() {
                let mut l = self.class_logs[cl];
                for (&off, &val) in self.cat_offsets.iter().zip(data) {
                    l += block[off + val as usize];
                }
                out[cl] = l;
            }
            return;
        }
        let class_total: f64 = self.class_totals.iter().sum();
        let class_num = self.class_num as f64;
        for (cl, block) in self.class_cat_totals.chunks_exact(self.total_cat_vals).enumerate() {
            let class_count = self.class_totals[cl];
            let mut l = ((class_count + self.alpha) / (class_total + class_num * self.alpha)).ln();
            for ((&off, &cat), &val) in self.cat_offsets.iter().zip(&self.categories).zip(data) {
                l += ((block[off + val as usize] + self.alpha) / (class_count + cat as f64 * self.alpha)).ln();
            }
            out[cl] = l;
        }
    }

    /// Serializes in the C text format (`cnbp_to_file`).
    pub fn to_text(&self) -> String {
        let mut s = String::new();
        writeln!(s, "{}", self.class_num).unwrap();
        writeln!(s, "{:.6}", self.alpha).unwrap();
        writeln!(s, "{}", self.categories.len()).unwrap();
        for cat in &self.categories {
            writeln!(s, "{cat}").unwrap();
        }
        self.write_counts(&mut s);
        s
    }

    /// Writes only the counts (`cnbp_to_file_no_params`), as BDT nodes do.
    pub(crate) fn write_counts(&self, s: &mut String) {
        for t in self.class_totals.iter().chain(&self.class_cat_totals) {
            writeln!(s, "{t:.6}").unwrap();
        }
    }

    /// Parses the C text format (`cnbp_from_file`).
    pub fn from_text(text: &str, use_probs: bool) -> io::Result<Self> {
        let mut tokens = Tokens::new(text);
        let class_num: usize = tokens.next()?;
        let alpha: f64 = tokens.next()?;
        let cat_num: usize = tokens.next()?;
        let categories = (0..cat_num)
            .map(|_| tokens.next::<u8>())
            .collect::<io::Result<Vec<_>>>()?;

        let mut cnbp = Cnbp::new(class_num, &categories, alpha, use_probs);
        cnbp.read_counts(&mut tokens)?;
        Ok(cnbp)
    }

    /// Reads counts written by `write_counts` into a model of the right shape
    /// (`cnbp_from_file_with_params`).
    pub(crate) fn read_counts(&mut self, tokens: &mut Tokens) -> io::Result<()> {
        for t in self.class_totals.iter_mut().chain(self.class_cat_totals.iter_mut()) {
            *t = tokens.next()?;
        }
        self.fresh = false;
        self.refresh();
        Ok(())
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        std::fs::write(path, self.to_text())
    }

    pub fn load(path: &Path, use_probs: bool) -> io::Result<Self> {
        Self::from_text(&std::fs::read_to_string(path)?, use_probs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trained(use_probs: bool) -> Cnbp {
        let mut m = Cnbp::new(2, &[3, 2], 1.0, use_probs);
        m.train(&[0, 1], &[1.0, 0.0]);
        m.train(&[2, 1], &[0.0, 1.0]);
        m.train(&[2, 0], &[0.25, 0.75]);
        m.refresh();
        m
    }

    #[test]
    fn hand_computed_prediction() {
        // class 0: total 1.25, P = 2.25/5; P(cat0=2|0) = 1.25/4.25; P(cat1=1|0) = 2/3.25
        // class 1: total 1.75, P = 2.75/5; P(cat0=2|1) = 2.75/4.75; P(cat1=1|1) = 2/3.75
        let a = 2.25 / 5.0 * (1.25 / 4.25) * (2.0 / 3.25);
        let b = 2.75 / 5.0 * (2.75 / 4.75) * (2.0 / 3.75);
        let mut out = [0.0; 2];
        trained(false).predict(&[2, 1], &mut out);
        assert!((out[0] - a / (a + b)).abs() < 1e-12);
        assert!((out[1] - b / (a + b)).abs() < 1e-12);
    }

    #[test]
    fn probs_mode_matches_counts_mode() {
        let (mut p, mut c) = ([0.0; 2], [0.0; 2]);
        trained(true).predict(&[1, 0], &mut p);
        trained(false).predict(&[1, 0], &mut c);
        assert!((p[0] - c[0]).abs() < 1e-12);
    }

    #[test]
    fn log_prediction_matches() {
        for use_probs in [false, true] {
            let (mut p, mut l) = ([0.0; 2], [0.0; 2]);
            trained(use_probs).predict(&[2, 1], &mut p);
            trained(use_probs).predict_log(&[2, 1], &mut l);
            assert!(((l[0] - l[1]) - (p[0] / p[1]).ln()).abs() < 1e-12);
        }
    }

    #[test]
    fn weighted_examples_count_their_weight() {
        let (mut once, mut halves) = (Cnbp::new(2, &[3, 2], 1.0, false), Cnbp::new(2, &[3, 2], 1.0, false));
        once.train(&[2, 1], &[0.25, 0.75]);
        halves.train_weighted(&[2, 1], &[0.25, 0.75], 0.5);
        halves.train_weighted(&[2, 1], &[0.25, 0.75], 0.5);
        assert_eq!(once.to_text(), halves.to_text());
    }

    #[test]
    fn text_round_trip() {
        let m = trained(true);
        let back = Cnbp::from_text(&m.to_text(), true).unwrap();
        assert_eq!(m.to_text(), back.to_text());
    }
}
