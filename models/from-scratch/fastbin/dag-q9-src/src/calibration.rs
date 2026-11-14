//! Calibrated class probabilities.
//!
//! Naive Bayes multiplies the evidence of 70 correlated features as if they
//! were independent, so its probabilities are far too confident (a median
//! P(draw) of 0.9996). A calibration rescales the model's log-probabilities
//! (`Classifier::predict_log`) by `slope` and adds a per-class `offset`, then
//! normalises; both are fitted to minimise log-loss on held-out positions.
//! It needs no chess knowledge, only labelled self-play positions.

use anyhow::{Result, bail};

/// Rescales a model's log-probabilities of [white win, black win, draw]:
/// with d = ln P(W) - ln P(B) and m = (ln P(W) + ln P(B)) / 2 - ln P(D),
/// p(W) ~ exp(offset_W + slope_winner d/2 + slope_decisive m),
/// p(B) ~ exp(offset_B - slope_winner d/2 + slope_decisive m), p(D) ~ 1.
/// Separate slopes, because naive Bayes is far more overconfident about
/// draws than about who wins.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Calibration {
    pub slope_winner: f64,
    pub slope_decisive: f64,
    pub offsets: [f64; 2],
}

impl Calibration {
    /// The model's own probabilities.
    pub const NONE: Calibration = Calibration { slope_winner: 1.0, slope_decisive: 1.0, offsets: [0.0; 2] };

    fn params(&self) -> [f64; 4] {
        [self.slope_winner, self.slope_decisive, self.offsets[0], self.offsets[1]]
    }

    fn from_params(p: [f64; 4]) -> Self {
        Calibration { slope_winner: p[0], slope_decisive: p[1], offsets: [p[2], p[3]] }
    }

    /// Each class's features; its logit is their dot product with the
    /// parameters.
    fn features(logs: &[f64]) -> [[f64; 4]; 3] {
        let d = logs[0] - logs[1];
        let m = (logs[0] + logs[1]) / 2.0 - logs[2];
        [[d / 2.0, m, 1.0, 0.0], [-d / 2.0, m, 0.0, 1.0], [0.0; 4]]
    }

    /// Calibrated logits of the 3 classes (logs of unnormalised
    /// probabilities, the draw's being 0) from log-probabilities.
    pub fn logits(&self, logs: &[f64]) -> [f64; 3] {
        let p = self.params();
        Self::features(logs).map(|f| (0..4).map(|i| p[i] * f[i]).sum::<f64>())
    }

    /// Calibrated probabilities of the 3 classes from log-probabilities.
    pub fn probs(&self, logs: &[f64]) -> [f64; 3] {
        let z = self.logits(logs);
        let max = z.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let e = z.map(|v| (v - max).exp());
        let sum: f64 = e.iter().sum();
        e.map(|v| v / sum)
    }

    /// Mean log-loss on (log-probabilities, actual class) pairs.
    pub fn log_loss(&self, data: &[([f64; 3], usize)]) -> f64 {
        self.weighted_log_loss(data, &vec![1.0; data.len()])
    }

    /// Mean log-loss with each pair weighted by `weights`.
    pub fn weighted_log_loss(&self, data: &[([f64; 3], usize)], weights: &[f64]) -> f64 {
        let loss = |(logs, class): &([f64; 3], usize)| -self.probs(logs)[*class].max(f64::MIN_POSITIVE).ln();
        data.iter().zip(weights).map(|(d, w)| w * loss(d)).sum::<f64>() / weights.iter().sum::<f64>()
    }

    /// Fits the slopes and offsets by Newton's method (the log-loss is
    /// convex in them).
    pub fn fit(data: &[([f64; 3], usize)]) -> Calibration {
        Self::fit_weighted(data, &vec![1.0; data.len()])
    }

    /// Like `fit`, minimising the log-loss with each pair weighted by `weights`.
    pub fn fit_weighted(data: &[([f64; 3], usize)], weights: &[f64]) -> Calibration {
        // Start from the class frequencies alone.
        let mut counts = [1.0f64; 3];
        data.iter().zip(weights).for_each(|((_, c), w)| counts[*c] += w);
        let mut cal = Calibration {
            slope_winner: 0.0,
            slope_decisive: 0.0,
            offsets: [(counts[0] / counts[2]).ln(), (counts[1] / counts[2]).ln()],
        };
        let mut loss = cal.weighted_log_loss(data, weights);

        for _ in 0..100 {
            let (mut grad, mut hess) = ([0.0f64; 4], [[0.0f64; 4]; 4]);
            for ((logs, class), &w) in data.iter().zip(weights) {
                let p = cal.probs(logs);
                let f = Self::features(logs);
                let mean: [f64; 4] = std::array::from_fn(|i| (0..3).map(|c| p[c] * f[c][i]).sum());
                for i in 0..4 {
                    grad[i] += w * (mean[i] - f[*class][i]);
                    for j in 0..4 {
                        hess[i][j] += w * ((0..3).map(|c| p[c] * f[c][i] * f[c][j]).sum::<f64>() - mean[i] * mean[j]);
                    }
                }
            }
            let Some(step) = solve(hess, grad) else { break };
            // Backtracking line search on the Newton direction.
            let mut t = 1.0;
            let improved = loop {
                let p = cal.params();
                let next = Calibration::from_params(std::array::from_fn(|i| p[i] - t * step[i]));
                let next_loss = next.weighted_log_loss(data, weights);
                if next_loss < loss {
                    break Some((next, next_loss));
                }
                t /= 2.0;
                if t < 1e-6 {
                    break None;
                }
            };
            let Some((next, next_loss)) = improved else { break };
            let done = loss - next_loss < 1e-10;
            (cal, loss) = (next, next_loss);
            if done {
                break;
            }
        }
        cal
    }

    /// Parses "SLOPE_WINNER,SLOPE_DECISIVE,OFFSET_WHITE,OFFSET_BLACK", as
    /// printed by `eval-loss --fit-calibration`.
    pub fn parse(s: &str) -> Result<Calibration> {
        let v: Vec<f64> = s.split(',').map(|x| x.trim().parse::<f64>()).collect::<Result<_, _>>()?;
        let Ok(p) = <[f64; 4]>::try_from(v) else {
            bail!("expected SLOPE_WINNER,SLOPE_DECISIVE,OFFSET_WHITE,OFFSET_BLACK, got {s:?}");
        };
        Ok(Calibration::from_params(p))
    }
}

impl std::fmt::Display for Calibration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let p = self.params();
        write!(f, "{:.6},{:.6},{:.6},{:.6}", p[0], p[1], p[2], p[3])
    }
}

/// Solves a x = b by Gaussian elimination with partial pivoting.
fn solve<const N: usize>(mut a: [[f64; N]; N], mut b: [f64; N]) -> Option<[f64; N]> {
    for col in 0..N {
        let pivot = (col..N).max_by(|&i, &j| a[i][col].abs().total_cmp(&a[j][col].abs()))?;
        if a[pivot][col].abs() < 1e-300 {
            return None;
        }
        a.swap(col, pivot);
        b.swap(col, pivot);
        for row in col + 1..N {
            let f = a[row][col] / a[col][col];
            for k in col..N {
                a[row][k] -= f * a[col][k];
            }
            b[row] -= f * b[col];
        }
    }
    let mut x = [0.0; N];
    for row in (0..N).rev() {
        x[row] = (b[row] - (row + 1..N).map(|k| a[row][k] * x[k]).sum::<f64>()) / a[row][row];
    }
    Some(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovers_a_known_calibration() {
        // Labels drawn from calibrated probabilities of overconfident
        // log-probabilities.
        let truth = Calibration { slope_winner: 0.6, slope_decisive: 0.1, offsets: [0.5, -0.3] };
        let mut x = 7u64;
        let mut uniform = || {
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (x >> 11) as f64 / (1u64 << 53) as f64
        };
        let data: Vec<([f64; 3], usize)> = (0..20_000)
            .map(|_| {
                let logs = [-30.0 * uniform(), -30.0 * uniform(), -30.0 * uniform()];
                let p = truth.probs(&logs);
                let u = uniform();
                let class = if u < p[0] { 0 } else if u < p[0] + p[1] { 1 } else { 2 };
                (logs, class)
            })
            .collect();
        let fit = Calibration::fit(&data);
        assert!((fit.slope_winner - 0.6).abs() < 0.06 && (fit.slope_decisive - 0.1).abs() < 0.02, "{fit:?}");
        assert!((fit.offsets[0] - 0.5).abs() < 0.2 && (fit.offsets[1] + 0.3).abs() < 0.2, "{fit:?}");
        assert!(fit.log_loss(&data) <= truth.log_loss(&data) + 1e-3);

        // Weight 2 is the same as a duplicate, weight 0 as leaving it out.
        let doubled: Vec<_> = data[..2000].iter().chain(&data[..1000]).copied().collect();
        let weights: Vec<f64> = (0..3000).map(|i| if i < 1000 { 2.0 } else if i < 2000 { 1.0 } else { 0.0 }).collect();
        let a = Calibration::fit(&doubled);
        let b = Calibration::fit_weighted(&data[..3000], &weights);
        let close = |x: f64, y: f64| (x - y).abs() < 1e-6;
        assert!(close(a.slope_winner, b.slope_winner) && close(a.slope_decisive, b.slope_decisive), "{a:?} {b:?}");
        assert!(close(a.offsets[0], b.offsets[0]) && close(a.offsets[1], b.offsets[1]), "{a:?} {b:?}");
    }
}
