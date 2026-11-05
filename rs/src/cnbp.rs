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
    /// Probabilities cached from the counts, used when `use_probs` is set.
    class_probs: Vec<f64>,
    class_cat_probs: Vec<f64>,
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
        let sum: f64 = label.iter().sum();
        assert!(
            (sum - 1.0).abs() <= 0.000001,
            "cnbp label must sum to 1, got {sum}"
        );

        for (cl, block) in self.class_cat_totals.chunks_exact_mut(self.total_cat_vals).enumerate() {
            let l = label[cl];
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

            for cat in 0..self.categories.len() {
                let denom = self.class_totals[cl] + self.categories[cat] as f64 * self.alpha;
                for val in 0..self.categories[cat] {
                    let i = self.idx(cl, cat, val);
                    self.class_cat_probs[i] = (self.alpha + self.class_cat_totals[i]) / denom;
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
        let mut total = 0.0;

        if self.use_probs {
            assert!(self.fresh, "cnbp trained without refresh() before predict()");

            for (cl, block) in self.class_cat_probs.chunks_exact(self.total_cat_vals).enumerate() {
                let mut p = self.class_probs[cl];
                for (&off, &val) in self.cat_offsets.iter().zip(data) {
                    p *= block[off + val as usize];
                }
                out[cl] = p;
                total += p;
            }
        } else {
            let class_total: f64 = self.class_totals.iter().sum();
            let class_num = self.class_num as f64;

            for (cl, block) in self.class_cat_totals.chunks_exact(self.total_cat_vals).enumerate() {
                let class_count = self.class_totals[cl];
                let mut p = (class_count + self.alpha) / (class_total + class_num * self.alpha);
                for ((&off, &cat), &val) in self.cat_offsets.iter().zip(&self.categories).zip(data) {
                    p *= (block[off + val as usize] + self.alpha)
                        / (class_count + cat as f64 * self.alpha);
                }
                out[cl] = p;
                total += p;
            }
        }

        for p in &mut out[..self.class_num] {
            *p /= total;
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
        for t in self.class_totals.iter().chain(&self.class_cat_totals) {
            writeln!(s, "{t:.6}").unwrap();
        }
        s
    }

    /// Parses the C text format (`cnbp_from_file`).
    pub fn from_text(text: &str, use_probs: bool) -> io::Result<Self> {
        let mut tokens = text.split_whitespace();
        let mut next = || {
            tokens
                .next()
                .ok_or_else(|| invalid("cnbp model file ended early"))
        };

        let class_num: usize = next()?.parse().map_err(invalid)?;
        let alpha: f64 = next()?.parse().map_err(invalid)?;
        let cat_num: usize = next()?.parse().map_err(invalid)?;
        let categories = (0..cat_num)
            .map(|_| next()?.parse::<u8>().map_err(invalid))
            .collect::<io::Result<Vec<_>>>()?;

        let mut cnbp = Cnbp::new(class_num, &categories, alpha, use_probs);
        for t in cnbp.class_totals.iter_mut().chain(cnbp.class_cat_totals.iter_mut()) {
            *t = next()?.parse().map_err(invalid)?;
        }
        cnbp.fresh = false;
        cnbp.refresh();
        Ok(cnbp)
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        std::fs::write(path, self.to_text())
    }

    pub fn load(path: &Path, use_probs: bool) -> io::Result<Self> {
        Self::from_text(&std::fs::read_to_string(path)?, use_probs)
    }
}

fn invalid<E: ToString>(e: E) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, e.to_string())
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
    fn text_round_trip() {
        let m = trained(true);
        let back = Cnbp::from_text(&m.to_text(), true).unwrap();
        assert_eq!(m.to_text(), back.to_text());
    }
}
