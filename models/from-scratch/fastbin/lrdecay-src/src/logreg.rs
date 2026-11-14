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

use crate::state::{Reader, Writer};
use crate::textfmt::{Tokens, invalid};

/// hybrid6: whether `step` may use AVX2 (a build switch, for timing; results are the same either way).
const STEP_AVX2: bool = true;

/// lrdecay1: a model with a sum memory settles its sums (see `Logreg::age`) before an example would take age times
/// -ln(1 - 1/N) above this, so that the held sums stay below e^32 times the true ones.
const AGE_LIMIT: f64 = 32.0;

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
    /// AdaGrad: the sum of squared gradients of each weight. lrdecay1: with a sum memory, each sum divided by
    /// (1 - 1/N)^age (see `age`).
    squares: Vec<f64>,
    /// AdaGrad's base step size.
    rate: f64,
    /// lrdecay1 (--lr-sum-memory N): the AdaGrad sums' memory, in this model's own examples; 0 (the default) for
    /// none, today's AdaGrad, whose sums never decay. With a memory N, each example first multiplies every sum by
    /// (1 - 1/N), then adds its squared gradients: S <- S (1 - 1/N) + g^2, so a gradient's weight in the sums falls
    /// by e after about N examples, and the steps no longer shrink for good as the model ages.
    memory: u64,
    /// lrdecay1, with a memory: the examples this model has stepped on (each `step` is one) since its sums were
    /// last brought up to date (`settle`). Each held sum is the true sum divided by (1 - 1/N)^age, so an example
    /// touches only its own weights: it adds its squared gradient times (1 - 1/N)^(-age), and the decay of all
    /// the other sums is the growth of that factor. Always 0 without a memory.
    age: u64,
    /// lrdecay1: -ln(1 - 1/N) for a memory N, 0 without one.
    kappa: f64,
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
            memory: 0,
            age: 0,
            kappa: 0.0,
        }
    }

    /// lrdecay1: gives the AdaGrad sums a memory of `n` examples (see `memory`), or none with 0. Settles the sums
    /// first, so they keep their values.
    pub fn set_sum_memory(&mut self, n: u64) {
        assert!(n != 1, "a sum memory of 1 example would forget every sum at once: 0 (none) or at least 2");
        self.settle();
        self.memory = n;
        self.kappa = if n == 0 { 0.0 } else { -(-1.0 / n as f64).ln_1p() };
    }

    /// lrdecay1: the AdaGrad sums' memory in examples, 0 for none.
    pub fn sum_memory(&self) -> u64 {
        self.memory
    }

    /// lrdecay1: brings every held sum up to date (see `age`) and sets the age to 0; nothing at age 0, which a
    /// model without a memory always is. The sums' values stay the same, up to rounding.
    pub fn settle(&mut self) {
        if self.age == 0 {
            return;
        }
        let f = self.settle_factor();
        self.squares.iter_mut().for_each(|s| *s *= f);
        self.age = 0;
    }

    /// lrdecay1: (1 - 1/N)^age, the factor that brings the held sums up to date.
    fn settle_factor(&self) -> f64 {
        (-(self.age as f64) * self.kappa).exp()
    }

    /// lrdecay1: one more example of a model with a memory, and the factor (1 - 1/N)^(-age/2) by which its
    /// gradient is scaled (see `step`), after settling first if the age would pass `AGE_LIMIT`.
    fn age_one(&mut self) -> f64 {
        if (self.age + 1) as f64 * self.kappa > AGE_LIMIT {
            self.settle();
        }
        self.age += 1;
        (0.5 * self.kappa * self.age as f64).exp()
    }

    /// The weights, class by class (for tests).
    #[cfg(test)]
    pub(crate) fn weights_mut(&mut self) -> &mut [f64] {
        &mut self.weights
    }

    /// The model in fixed point (see `fixed`), from its current weights.
    pub fn fixed(&self) -> Option<crate::fixed::FxLinear> {
        crate::fixed::FxLinear::from_weights(self.class_num, &self.starts, self.width, &self.weights)
    }

    /// Writes each class's logit (log of its unnormalised probability).
    pub fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        // hybrid6: three classes' sums side by side, each in its own order (from -0.0, as `sum`), so their
        // additions overlap; the values are those of the loop below.
        if self.class_num == 3 && out.len() >= 3 {
            let w = self.width;
            let (w0, rest) = self.weights.split_at(w);
            let (w1, w2) = rest.split_at(w);
            let w2 = &w2[..w];
            let (mut s0, mut s1, mut s2) = (-0.0f64, -0.0f64, -0.0f64);
            for (&s, &v) in self.starts.iter().zip(data) {
                let k = s + v as usize;
                s0 += w0[k];
                s1 += w1[k];
                s2 += w2[k];
            }
            out[0] = w0[0] + s0;
            out[1] = w1[0] + s1;
            out[2] = w2[0] + s2;
            return;
        }
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
        // hybrid6: a small buffer for the usual few classes (no 2 KB to zero per example).
        let (mut small, mut big) = ([0.0; 8], Vec::new());
        let p: &mut [f64] = if self.class_num <= 8 {
            &mut small[..self.class_num]
        } else {
            big.resize(self.class_num, 0.0);
            &mut big
        };
        self.predict(data, p);
        self.step(data, p, label, 1.0, 0.0);
    }

    /// An AdaGrad step on `weight` times the log-loss of the example, given
    /// the model's current prediction `p` for it, plus an L2 penalty of
    /// `l2` / 2 times the square of each weight the example uses.
    pub(crate) fn step(&mut self, data: &[u8], p: &[f64], label: &[f64], weight: f64, l2: f64) {
        // lrdecay1: with a sum memory, a held sum U is the true sum S times c = (1 - 1/N)^(-age) (see `age`). The
        // decayed step adds g^2 to S and moves the weight by rate g / sqrt(S); in held terms that is U += c g^2 and
        // rate sqrt(c) g / sqrt(U), which is AdaGrad's step below on the gradient sqrt(c) g. So `weight` and `l2`,
        // which make up g, are both scaled by sqrt(c).
        let (weight, l2) = if self.memory == 0 {
            (weight, l2)
        } else {
            let scale = self.age_one();
            (weight * scale, l2 * scale)
        };
        // hybrid6: four weights at a time with AVX2 when the CPU has it and every value is in range (so the
        // weights a step touches are distinct, and their order can't matter). The same operations on each weight:
        // vector division, square root, products and sums round as the scalar ones do.
        #[cfg(target_arch = "x86_64")]
        {
            let nf = self.starts.len().min(data.len());
            if STEP_AVX2 && std::arch::is_x86_feature_detected!("avx2")
                && self.categories[..nf].iter().zip(&data[..nf]).all(|(&c, &v)| v < c)
            {
                // SAFETY: AVX2 is present, and every index is in range (checked above).
                unsafe { self.step_avx2(&data[..nf], p, label, weight, l2) };
                return;
            }
        }
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

    /// `step` four weights at a time (see `step`); `data` holds only the features of `starts`, each in range.
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn step_avx2(&mut self, data: &[u8], p: &[f64], label: &[f64], weight: f64, l2: f64) {
        unsafe { self.step_avx2_body(data, p, label, weight, l2) }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    #[allow(unsafe_op_in_unsafe_fn)]
    unsafe fn step_avx2_body(&mut self, data: &[u8], p: &[f64], label: &[f64], weight: f64, l2: f64) {
        use std::arch::x86_64::*;
        let nf = data.len();
        let rate = self.rate;
        for c in 0..self.class_num {
            let g = weight * (p[c] - label[c]);
            if g == 0.0 && l2 == 0.0 {
                continue;
            }
            let base = c * self.width;
            let (w_ptr, s_ptr) = (self.weights.as_mut_ptr(), self.squares.as_mut_ptr());
            let one = |i: usize| {
                // SAFETY: i is in range (see `step`).
                unsafe {
                    let w = *w_ptr.add(i);
                    let g = g + l2 * w;
                    if g == 0.0 {
                        return;
                    }
                    let sq = *s_ptr.add(i) + g * g;
                    *s_ptr.add(i) = sq;
                    *w_ptr.add(i) = w - rate * g / sq.sqrt();
                }
            };
            one(base);
            let (vg, vl2, vrate, zero) = (_mm256_set1_pd(g), _mm256_set1_pd(l2), _mm256_set1_pd(rate), _mm256_setzero_pd());
            let mut f = 0;
            while f + 4 <= nf {
                let ix = [
                    base + self.starts[f] + data[f] as usize,
                    base + self.starts[f + 1] + data[f + 1] as usize,
                    base + self.starts[f + 2] + data[f + 2] as usize,
                    base + self.starts[f + 3] + data[f + 3] as usize,
                ];
                // Plain loads (gathers are slow on the E-cores).
                let w = _mm256_set_pd(*w_ptr.add(ix[3]), *w_ptr.add(ix[2]), *w_ptr.add(ix[1]), *w_ptr.add(ix[0]));
                let sq = _mm256_set_pd(*s_ptr.add(ix[3]), *s_ptr.add(ix[2]), *s_ptr.add(ix[1]), *s_ptr.add(ix[0]));
                let gi = _mm256_add_pd(vg, _mm256_mul_pd(vl2, w));
                // `if g == 0.0 { return; }` per weight: unordered (NaN) counts as nonzero, as there.
                let live = _mm256_cmp_pd::<_CMP_NEQ_UQ>(gi, zero);
                let sq2 = _mm256_add_pd(sq, _mm256_mul_pd(gi, gi));
                let w2 = _mm256_sub_pd(w, _mm256_div_pd(_mm256_mul_pd(vrate, gi), _mm256_sqrt_pd(sq2)));
                let (mut so, mut wo) = ([0.0f64; 4], [0.0f64; 4]);
                _mm256_storeu_pd(so.as_mut_ptr(), _mm256_blendv_pd(sq, sq2, live));
                _mm256_storeu_pd(wo.as_mut_ptr(), _mm256_blendv_pd(w, w2, live));
                for k in 0..4 {
                    *s_ptr.add(ix[k]) = so[k];
                    *w_ptr.add(ix[k]) = wo[k];
                }
                f += 4;
            }
            while f < nf {
                one(base + self.starts[f] + data[f] as usize);
                f += 1;
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
        for x in &self.weights {
            writeln!(s, "{x:e}").unwrap();
        }
        // lrdecay1: the sums as they are now, `settle`'s values. A model with a memory carries neither the memory
        // nor its age in this format; flat models are settled before they are saved (`Classifier::refresh`), so
        // their files resume exactly with the same --lr-sum-memory, and trees resume from their state files.
        let f = self.settle_factor();
        for &x in &self.squares {
            let x = if self.age == 0 { x } else { x * f };
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

    /// Writes the whole model for a tree's state file (see `state`). lrdecay1: not for a model with a sum memory
    /// (`write_state_memory`).
    pub(crate) fn write_state(&self, w: &mut Writer) {
        assert!(self.memory == 0, "a logistic regression with a sum memory is written by write_state_memory");
        self.write_core(w)
    }

    /// lrdecay1: `write_state` for a model with a sum memory: the same, then the memory and the sums' age, so that
    /// the held sums read back exactly.
    pub(crate) fn write_state_memory(&self, w: &mut Writer) {
        assert!(self.memory != 0, "write_state_memory is for a logistic regression with a sum memory");
        self.write_core(w);
        w.u64(self.memory);
        w.u64(self.age);
    }

    /// lrdecay1: reads a model written by `write_state_memory`.
    pub(crate) fn read_state_memory(r: &mut Reader) -> io::Result<Self> {
        let mut model = Self::read_state(r)?;
        let (memory, age) = (r.u64()?, r.u64()?);
        if memory < 2 {
            return Err(invalid("a logistic regression's sum memory in a state file is at least 2 examples"));
        }
        model.set_sum_memory(memory);
        model.age = age;
        Ok(model)
    }

    fn write_core(&self, w: &mut Writer) {
        w.usize(self.class_num);
        w.f64(self.rate);
        w.u8s(&self.categories);
        w.f64s(&self.weights);
        w.f64s(&self.squares);
    }

    /// Reads a model written by `write_state`.
    pub(crate) fn read_state(r: &mut Reader) -> io::Result<Self> {
        let class_num = r.usize()?;
        let rate = r.f64()?;
        let categories = r.u8s()?;
        let mut model = Logreg::new(class_num, &categories, rate);
        let (weights, squares) = (r.f64s()?, r.f64s()?);
        if weights.len() != model.weights.len() || squares.len() != model.squares.len() {
            return Err(invalid("logistic regression state of the wrong shape"));
        }
        model.weights = weights;
        model.squares = squares;
        Ok(model)
    }

    pub fn class_num(&self) -> usize {
        self.class_num
    }

    /// grow1: the number of parameters (the MDL bar's k): every class's weights, biases included.
    pub fn num_params(&self) -> usize {
        self.class_num * self.width
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

    /// grow5's tests: the AdaGrad sums (lrdecay1: as held, see `age`).
    #[cfg(test)]
    pub(crate) fn squares(&self) -> &[f64] {
        &self.squares
    }

    /// lrdecay1's tests: the AdaGrad sums as they are now (`settle`'s values), and the age.
    #[cfg(test)]
    pub(crate) fn settled_squares(&self) -> Vec<f64> {
        let f = self.settle_factor();
        self.squares.iter().map(|&x| if self.age == 0 { x } else { x * f }).collect()
    }

    #[cfg(test)]
    pub(crate) fn age(&self) -> u64 {
        self.age
    }

    /// A model of this one's shape and step size that knows nothing yet:
    /// every weight and AdaGrad sum 0. lrdecay1: with this one's sum memory.
    pub(crate) fn blank(&self) -> Logreg {
        let mut out = Logreg::new(self.class_num, &self.categories, self.rate);
        (out.memory, out.kappa) = (self.memory, self.kappa);
        out
    }

    /// A model of this one's shape and step size with every weight 0 but
    /// this one's AdaGrad sums: it predicts nothing yet but steps as this
    /// one would.
    pub(crate) fn zeroed(&self) -> Logreg {
        let mut out = self.clone();
        out.weights.iter_mut().for_each(|w| *w = 0.0);
        out
    }

    /// This model with AdaGrad base step size `rate`.
    pub(crate) fn with_rate(mut self, rate: f64) -> Logreg {
        self.rate = rate;
        self
    }

    /// Whether `other` has the same classes and features, so that their
    /// weights line up.
    fn same_shape(&self, other: &Logreg) -> bool {
        self.class_num == other.class_num && self.categories == other.categories
    }

    /// Sets every weight to the sum of `a`'s and `b`'s, models of this one's
    /// shape, and every AdaGrad sum to the sum of theirs (`add_squares`) or
    /// to `b`'s: the effective model of a residual leaf, `a` its delegator's
    /// shadow and `b` its own weights (see `delegation`).
    pub(crate) fn set_sum(&mut self, a: &Logreg, b: &Logreg, add_squares: bool) {
        assert!(self.same_shape(a) && self.same_shape(b), "adding logistic regressions of different shapes");
        assert!(self.memory == 0 && a.memory == 0 && b.memory == 0, "residual leaves have no sum memory (lrdecay1)");
        for (x, (y, z)) in self.weights.iter_mut().zip(a.weights.iter().zip(&b.weights)) {
            *x = y + z;
        }
        for (x, (y, z)) in self.squares.iter_mut().zip(a.squares.iter().zip(&b.squares)) {
            *x = if add_squares { y + z } else { *z };
        }
    }

    /// `set_sum` for just the weights and sums an example uses: every class's
    /// bias and the weight of each feature's value in `data`, the only ones a
    /// training step on it changes (see `step`).
    pub(crate) fn set_sum_at(&mut self, a: &Logreg, b: &Logreg, data: &[u8], add_squares: bool) {
        debug_assert!(self.same_shape(a) && self.same_shape(b), "adding logistic regressions of different shapes");
        debug_assert!(self.memory == 0 && a.memory == 0 && b.memory == 0, "residual leaves have no sum memory (lrdecay1)");
        for c in 0..self.class_num {
            let base = c * self.width;
            let mut set = |i: usize| {
                self.weights[base + i] = a.weights[base + i] + b.weights[base + i];
                self.squares[base + i] = if add_squares { a.squares[base + i] + b.squares[base + i] } else { b.squares[base + i] };
            };
            set(0);
            for (&s, &v) in self.starts.iter().zip(data) {
                set(s + v as usize);
            }
        }
    }

    /// A model of this one's shape and step size whose weights are this
    /// one's minus `other`'s, with this one's AdaGrad sums.
    pub(crate) fn minus(&self, other: &Logreg) -> Logreg {
        assert!(self.same_shape(other), "subtracting logistic regressions of different shapes");
        let mut out = self.clone();
        out.weights.iter_mut().zip(&other.weights).for_each(|(x, y)| *x -= y);
        out
    }

    /// The square root of the sum of the squared weights, and the largest
    /// absolute weight.
    pub(crate) fn weight_norms(&self) -> (f64, f64) {
        let l2 = self.weights.iter().map(|w| w * w).sum::<f64>().sqrt();
        (l2, self.weights.iter().fold(0.0, |m: f64, w| m.max(w.abs())))
    }
}

pub(crate) fn softmax(x: &mut [f64]) {
    let max = x.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    x.iter_mut().for_each(|v| *v = (*v - max).exp());
    let sum: f64 = x.iter().sum();
    x.iter_mut().for_each(|v| *v /= sum);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classifier::Classifier;

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

    /// lrdecay1: a seeded stream over features of skewed frequencies (feature 0 is 0 nine times in ten, so its
    /// value 1 goes untouched for long stretches) whose labels drift: class 0's chance follows feature 1 and flips
    /// its sign every 700 examples.
    fn drifting(seed: u64, n: usize) -> Vec<([u8; 4], [f64; 3])> {
        let mut x = seed;
        let mut next = move |m: u64| {
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (x >> 33) % m
        };
        (0..n)
            .map(|i| {
                let d = [(next(10) == 0) as u8, next(3) as u8, next(4) as u8, next(2) as u8];
                let up = (i / 700) % 2 == 0;
                let p0 = if (d[1] == 0) == up { 0.8 } else { 0.2 };
                let r = next(1000) as f64 / 1000.0;
                let label = if r < p0 { [1.0, 0.0, 0.0] } else if r < p0 + 0.1 { [0.0, 0.0, 1.0] } else { [0.0, 1.0, 0.0] };
                (d, label)
            })
            .collect()
    }

    /// lrdecay1: the decayed AdaGrad written densely, as defined: before each example every sum is multiplied by
    /// (1 - 1/N), then each weight the example uses adds its squared gradient and steps by rate g / sqrt(sum).
    struct Dense {
        model: Logreg,
        rho: f64,
    }

    impl Dense {
        fn step(&mut self, data: &[u8], label: &[f64], weight: f64, l2: f64) {
            let m = &mut self.model;
            m.squares.iter_mut().for_each(|s| *s *= self.rho);
            let mut p = vec![0.0; m.class_num];
            m.predict(data, &mut p);
            for c in 0..m.class_num {
                let g = weight * (p[c] - label[c]);
                let base = c * m.width;
                let mut idx = vec![base];
                idx.extend(m.starts.iter().zip(data).map(|(&s, &v)| base + s + v as usize));
                for i in idx {
                    let gi = g + l2 * m.weights[i];
                    if gi == 0.0 {
                        continue;
                    }
                    m.squares[i] += gi * gi;
                    m.weights[i] -= m.rate * gi / m.squares[i].sqrt();
                }
            }
        }
    }

    fn close(a: &[f64], b: &[f64], tol: f64) -> bool {
        a.iter().zip(b).all(|(x, y)| (x - y).abs() <= tol * x.abs().max(y.abs()).max(1e-300)) && a.len() == b.len()
    }

    /// lrdecay1: the lazy decay (held sums and an age, `step`) is the dense one, across examples of weight 1
    /// (`train`), weighted steps with an L2 penalty, a forget, and settles at the age limit (N 30 settles every
    /// 944 examples) and by hand.
    #[test]
    fn lrdecay1_lazy_decay_is_the_dense_decay() {
        let n = 30;
        let mut lazy = Logreg::new(3, &[2, 3, 4, 2], 0.1);
        lazy.set_sum_memory(n);
        let mut dense = Dense { model: Logreg::new(3, &[2, 3, 4, 2], 0.1), rho: 1.0 - 1.0 / n as f64 };
        let mut settled_by_limit = false;
        for (i, (d, y)) in drifting(3, 5000).into_iter().enumerate() {
            let (weight, l2) = if i % 7 == 3 { (0.6, 0.01) } else { (1.0, 0.0) };
            if weight == 1.0 {
                lazy.train(&d, &y);
            } else {
                let mut p = [0.0; 3];
                lazy.predict(&d, &mut p);
                lazy.step(&d, &p, &y, weight, l2);
            }
            dense.step(&d, &y, weight, l2);
            settled_by_limit |= i > 0 && i < 4000 && lazy.age() == 1;
            if i == 2500 {
                lazy.forget(4.0);
                dense.model.forget(4.0);
            }
            if i == 4000 {
                lazy.settle();
                assert_eq!(lazy.age(), 0);
            }
            if i % 500 == 499 {
                assert!(close(&lazy.weights, &dense.model.weights, 1e-9), "weights at {i}");
                assert!(close(&lazy.settled_squares(), &dense.model.squares, 1e-9), "sums at {i}");
            }
        }
        assert!(settled_by_limit, "the age limit settled the sums");
        // The memory matters here: AdaGrad's sums are far larger.
        let mut plain = Logreg::new(3, &[2, 3, 4, 2], 0.1);
        drifting(3, 5000).iter().for_each(|(d, y)| plain.train(d, y));
        assert!(plain.squares[0] > 20.0 * dense.model.squares[0], "{} {}", plain.squares[0], dense.model.squares[0]);
    }

    /// lrdecay1: a memory far longer than the stream is AdaGrad, up to the decay it still has (here 1 - 1e-8 at
    /// most); memory 0 is AdaGrad exactly.
    #[test]
    fn lrdecay1_a_long_memory_is_adagrad() {
        let data = drifting(5, 20_000);
        let mut plain = Logreg::new(3, &[2, 3, 4, 2], 0.05);
        let mut long = plain.clone();
        long.set_sum_memory(1_000_000_000_000);
        let mut zero = plain.clone();
        zero.set_sum_memory(0);
        for (d, y) in &data {
            plain.train(d, y);
            long.train(d, y);
            zero.train(d, y);
        }
        let dw = plain.weights.iter().zip(&long.weights).map(|(a, b)| (a - b).abs()).fold(0.0, f64::max);
        let ds = (plain.squares.iter().zip(long.settled_squares())).map(|(a, b)| ((a - b) / a.max(1e-300)).abs()).fold(0.0, f64::max);
        let wmax = plain.weights.iter().fold(0.0f64, |m, w| m.max(w.abs()));
        eprintln!("max weight difference {dw:e} (largest weight {wmax:e}); max relative sum difference {ds:e}");
        assert!(dw < 1e-6 * wmax && ds < 1e-7, "{dw:e} {ds:e}");
        assert_ne!(plain.weights, long.weights, "the long memory still decays");
        assert_eq!(zero.to_text(), plain.to_text());
        assert_eq!((zero.sum_memory(), zero.age()), (0, 0));
    }

    /// lrdecay1: shorter memories take larger steps on a drifting stream, and settle at the age limit.
    #[test]
    fn lrdecay1_a_shorter_memory_tracks_drift_faster() {
        let data = drifting(9, 7000);
        let loss = |n: u64| {
            let mut m = Logreg::new(3, &[2, 3, 4, 2], 0.05);
            m.set_sum_memory(n);
            let mut total = 0.0;
            for (i, (d, y)) in data.iter().enumerate() {
                let mut p = [0.0; 3];
                m.predict(d, &mut p);
                if i >= 3500 {
                    total -= y.iter().zip(&p).map(|(a, b)| a * b.ln()).sum::<f64>();
                }
                m.train(d, y);
            }
            total / 3500.0
        };
        let (adagrad, long, short) = (loss(0), loss(5000), loss(500));
        assert!(short < long && long < adagrad, "{short} {long} {adagrad}");
    }

    /// lrdecay1: a flat model settled (as `Classifier::refresh` does before saving) resumes from its text exactly;
    /// unsettled, its text holds the settled sums. A state written with its memory resumes exactly at any age.
    #[test]
    fn lrdecay1_resumes_exactly() {
        let data = drifting(13, 3000);
        let mut model = Logreg::new(3, &[2, 3, 4, 2], 0.05);
        model.set_sum_memory(400);
        data[..1700].iter().for_each(|(d, y)| model.train(d, y));
        assert!(model.age() > 0);
        // The text of an unsettled model holds what settling gives.
        let unsettled = model.to_text();
        let mut settled = model.clone();
        Classifier::refresh(&mut settled);
        assert_eq!(settled.age(), 0);
        assert_eq!(settled.to_text(), unsettled);
        // The state keeps the held sums and the age.
        let mut w = Writer::new("lr");
        model.write_state_memory(&mut w);
        let bytes = w.into_bytes();
        let mut r = Reader::new(&bytes, "lr").unwrap();
        let mut from_state = Logreg::read_state_memory(&mut r).unwrap();
        r.finish().unwrap();
        assert_eq!((from_state.sum_memory(), from_state.age()), (400, model.age()));
        assert_eq!(from_state.squares, model.squares);
        // The settled model's text read back, given the memory again.
        let mut from_text = Logreg::from_text(&settled.to_text()).unwrap();
        from_text.set_sum_memory(400);
        for (d, y) in &data[1700..] {
            for m in [&mut model, &mut from_state, &mut settled, &mut from_text] {
                m.train(d, y);
            }
        }
        assert_eq!(from_state.to_text(), model.to_text());
        assert_eq!(from_state.squares, model.squares);
        assert_eq!(from_text.to_text(), settled.to_text());
        assert_eq!(from_text.squares, settled.squares);
        // Settling changes only the rounding.
        assert!(close(&settled.settled_squares(), &model.settled_squares(), 1e-12));
    }

    /// lrdecay1: copies carry the memory and the age; a blank model the memory at age 0; forgetting divides the
    /// sums as they are now.
    #[test]
    fn lrdecay1_copies_carry_the_memory() {
        let mut model = Logreg::new(3, &[2, 3, 4, 2], 0.05);
        model.set_sum_memory(250);
        drifting(1, 900).iter().for_each(|(d, y)| model.train(d, y));
        let age = model.age();
        assert!(age > 0);
        let blank = model.blank();
        assert_eq!((blank.sum_memory(), blank.age()), (250, 0));
        let zeroed = model.zeroed();
        assert_eq!((zeroed.sum_memory(), zeroed.age(), &zeroed.squares), (250, age, &model.squares));
        let mut forgotten = model.clone();
        forgotten.forget(10.0);
        let tenth: Vec<f64> = model.settled_squares().iter().map(|s| s / 10.0).collect();
        assert!(close(&forgotten.settled_squares(), &tenth, 1e-14));
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
