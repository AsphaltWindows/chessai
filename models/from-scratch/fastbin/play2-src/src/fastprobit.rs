//! Exact fast depth-2 search for flat probit models (hybrid8).
//!
//! A flat probit plays in fixed point (`fixed`, a probit head): a position's sums are its latent mean and its
//! weights' variance, and its value for the side to move is `FxLinear::value`, the calibration's z_win - z_loss of
//! `probit::log_probs_of_latent` (two log normal CDFs and the draw's log, about 70 ns). At depth 2 a move's value is
//! the smallest of its replies' values (`search`'s negamax with a full window), so most replies need not be valued in
//! full. Each reply gets an estimate from a table of ln Phi with a proven bound on its distance from
//! `FxLinear::value`. Only the replies whose lower bound is at most the smallest upper bound are valued in full, and
//! the loop of `negamax` then runs over them in move order with the same comparisons: the same value, ties and signed
//! zeros included. This is `compound::Compound::best`'s method for trees (hybrid5), applied to a flat probit.
//!
//! The estimate. The z's are computed from the sums with the operations `FxLinear::value` uses, so they are the same
//! bits. ln Phi is concave with -1 < (ln Phi)'' < 0 (1 + (ln Phi)''(z) is the variance of a standard normal truncated
//! above z), so between grid points z_k = -30 + k h (h = 1/256) the chord lies below ln Phi by at most h^2/8; the
//! chord plus h^2/16 is within h^2/16 of it. The table holds `probit::ln_cdf_first` (the bits of the value's own
//! ln Phi) at the grid points; each float ln Phi is allowed 1e-13 + 16u |ln Phi| from the true one, the allowance
//! `compound` uses for the rational approximations. The rest is `compound`'s bound: the calibration's rounding (8u of
//! the terms' sizes, the draw's log under 465 for |z| < 30 and eps / s > 1e-6), times 4, plus 1e-10.
//!
//! CHESSAI_FX_PROBIT2=off turns it off (the hybrid7 search). CHESSAI_FX_CHECK=1 checks every estimate against its
//! bound and every move value against the hybrid7 search, bit for bit. CHESSAI_FX_STATS=1 counts (`report`).

use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

use crate::calibration::Calibration;
use crate::fixed::{self, FxLinear, Head, SHIFT, Sums};

const SCALE: f64 = (1u64 << SHIFT) as f64;
/// The unit roundoff of f64, 2^-53.
const U: f64 = f64::EPSILON / 2.0;
/// The table's grid: z_k = LO + k / PER, k = 0..=N (z from -30 to 30).
const LO: f64 = -30.0;
const PER: f64 = 256.0;
const N: usize = 60 * 256;
/// h^2 / 16: half the chord's largest gap below ln Phi.
const HALF_GAP: f64 = 1.0 / (16.0 * PER * PER);
/// Largest |z| estimated from the table (inside the compound bound's |z| < 30).
pub(crate) const Z_MAX: f64 = 29.5;

/// Whether the fast search is on (CHESSAI_FX_PROBIT2, default on).
pub fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| match std::env::var("CHESSAI_FX_PROBIT2").as_deref() {
        Ok("off") => false,
        Ok("on") | Err(_) => true,
        Ok(other) => panic!("CHESSAI_FX_PROBIT2 must be on or off, not {other:?}"),
    })
}

/// ln Phi at the grid points, as `probit::ln_cdf_first` computes it.
pub(crate) fn table() -> &'static [f64] {
    static T: OnceLock<Box<[f64]>> = OnceLock::new();
    T.get_or_init(|| (0..=N).map(|k| crate::probit::ln_cdf_first(LO + k as f64 / PER)).collect())
}

/// ln Phi(z) for |z| <= Z_MAX from the table, and a bound on its distance from the float ln Phi(z) of
/// `probit::ln_cdf2`: the chord's h^2/16; the table's and the float ln Phi's own approximations (1e-13 + 16u |ln Phi|
/// each, |ln Phi| at most max(|a|, |b|) + h^2/8); the interpolation's rounding (a few u of |a| + |b|); and x's rounding
/// (|z - LO| <= 60, so |dx| <= 60 u PER, at a slope of at most 31 / PER per unit of x).
#[inline]
pub(crate) fn ln_phi(t: &[f64], z: f64) -> (f64, f64) {
    let x = (z - LO) * PER;
    let k = (x.floor() as usize).min(N - 1);
    let frac = x - k as f64;
    let (a, b) = (t[k], t[k + 1]);
    let est = a + frac * (b - a) + HALF_GAP;
    let err = HALF_GAP + 3e-13 + 64.0 * U * (a.abs() + b.abs() + 1.0) + 60.0 * 31.0 * U;
    (est, err)
}

/// A flat probit's value for the side to move from its sums (`FxLinear::value`), estimated, and a bound on its
/// distance from that value. None if `fx` isn't a probit, if either isn't finite, if the bound isn't small, or if the
/// position is outside the table (|z| > Z_MAX, eps / s <= 1e-6, a tiny or non-finite variance).
#[inline]
pub fn estimate(fx: &FxLinear, s: &Sums, white: bool) -> Option<(f64, f64)> {
    let Head::Probit { eps, beta2 } = fx.head() else { return None };
    // As `FxLinear::log_probs` and `probit::log_probs_of_latent` compute them.
    let mean = s[0] as f64 / SCALE;
    let var = s[1] as f64 / SCALE + beta2;
    if !(var > 1e-12 && var.is_finite() && mean.is_finite()) {
        return None;
    }
    let sd = var.sqrt();
    let (zw, zb) = ((mean - eps) / sd, (-mean - eps) / sd);
    if !(zw.abs() <= Z_MAX && zb.abs() <= Z_MAX && eps / sd > 1e-6) {
        return None;
    }
    let t = table();
    let ((lw, ew), (lb, eb)) = (ln_phi(t, zw), ln_phi(t, zb));
    let d = lw - lb;
    if !d.is_finite() {
        return None;
    }
    let d_err = ew + eb + 2.0 * U * d.abs();
    let (v, err) = match fx.calibration() {
        Some(Calibration { slope_winner: p0, slope_decisive: p1, offsets: [p2, p3] }) => {
            let v = p0 * d + (p2 - p3);
            let m_size = (lw.abs() + lb.abs()) / 2.0 + 1000.0;
            let rounding = 8.0 * U * (p0.abs() * d.abs() + p1.abs() * m_size + p2.abs() + p3.abs() + v.abs());
            (v, 4.0 * (p0.abs() * d_err + rounding) + 1e-10)
        }
        None => (d, 4.0 * (d_err + 2.0 * U * d.abs()) + 1e-10),
    };
    if !(v.is_finite() && err < 1e-3) {
        return None;
    }
    Some((if white { v } else { -v }, err))
}

/// A reply at depth 2: its value for the side to move after it, in full (`exact`) or estimated within `err`, and its
/// sums.
#[derive(Clone, Copy, Debug)]
pub struct Reply {
    value: f64,
    err: f64,
    exact: bool,
    white: bool,
    sums: Sums,
}

impl Reply {
    /// A reply whose value is known: the game ends there.
    pub fn known(value: f64) -> Reply {
        Reply { value, err: 0.0, exact: true, white: true, sums: [0; 4] }
    }

    /// hybrid18: the value and its bound (0 if exact): the full value is within err of value.
    #[inline]
    pub fn estimate(&self) -> (f64, f64) {
        (self.value, self.err)
    }

    /// hybrid18: the full value (`FxLinear::value`): its own if exact.
    #[inline]
    pub fn exact_value(&self, fx: &FxLinear) -> f64 {
        if self.exact { self.value } else { fx.value(&self.sums, self.white) }
    }

    /// The position with sums `sums` (side to move white if `white`) as a reply: its estimate, or its full value if no
    /// estimate is safe.
    #[inline]
    pub fn of(fx: &FxLinear, sums: Sums, white: bool) -> Reply {
        count(&STATS.replies);
        match estimate(fx, &sums, white) {
            Some((value, err)) => Reply { value, err, exact: false, white, sums },
            None => {
                count(&STATS.unestimated);
                Reply { value: fx.value(&sums, white), err: 0.0, exact: true, white, sums }
            }
        }
    }
}

/// The value `negamax` finds for the position before the replies (depth 1, a full window): the largest -value, the
/// first on ties, NaN ignored, as its loop computes it, valuing in full only the replies that can matter (see
/// `compound::Compound::best`).
pub fn best(fx: &FxLinear, replies: &mut [Reply]) -> f64 {
    let mut upper = f64::INFINITY;
    for r in replies.iter() {
        let u = r.value + r.err;
        if u < upper {
            upper = u;
        }
    }
    if fixed::checking() {
        for r in replies.iter().filter(|r| !r.exact) {
            let exact = fx.value(&r.sums, r.white);
            assert!(exact.is_nan() || (exact - r.value).abs() <= r.err, "value {exact} outside {} +- {}", r.value, r.err);
        }
    }
    let mut close = 0u64;
    for r in replies.iter_mut() {
        if !r.exact && r.value - r.err <= upper {
            r.value = fx.value(&r.sums, r.white);
            (r.err, r.exact) = (0.0, true);
            close += 1;
        }
    }
    if stats_on() {
        STATS.candidates.fetch_add(close, Relaxed);
        STATS.extra.fetch_add(close.saturating_sub(1), Relaxed);
    }
    if !replies.iter().any(|r| r.exact && r.value <= upper) {
        count(&STATS.everything);
        for r in replies.iter_mut().filter(|r| !r.exact) {
            r.value = fx.value(&r.sums, r.white);
            (r.err, r.exact) = (0.0, true);
        }
    }
    // `negamax`'s loop, over the replies valued in full (the others are larger than one of them).
    let (mut best, mut alpha, beta) = (f64::NEG_INFINITY, f64::NEG_INFINITY, f64::INFINITY);
    for r in replies.iter().filter(|r| r.exact) {
        let value = r.value;
        if -value > best {
            best = -value;
        }
        alpha = alpha.max(-value);
        if alpha >= beta {
            break;
        }
    }
    best
}

/// Counts of the fast search's decisions, for CHESSAI_FX_STATS=1.
pub struct Stats {
    /// Searches and their moves.
    pub searches: AtomicU64,
    pub moves: AtomicU64,
    /// Depth-2 replies (not ending the game), and those valued in full at once (no safe estimate).
    pub replies: AtomicU64,
    pub unestimated: AtomicU64,
    /// Estimated replies valued in full as candidates for the smallest, and those beyond the first of a position.
    pub candidates: AtomicU64,
    pub extra: AtomicU64,
    /// Positions whose replies were all valued in full (no candidate within the bound in float).
    pub everything: AtomicU64,
}

pub static STATS: Stats = Stats {
    searches: AtomicU64::new(0),
    moves: AtomicU64::new(0),
    replies: AtomicU64::new(0),
    unestimated: AtomicU64::new(0),
    candidates: AtomicU64::new(0),
    extra: AtomicU64::new(0),
    everything: AtomicU64::new(0),
};

fn stats_on() -> bool {
    crate::compound::stats_on()
}

#[inline]
pub fn count(counter: &AtomicU64) {
    if stats_on() {
        counter.fetch_add(1, Relaxed);
    }
}

/// Prints the counts to stderr, with CHESSAI_FX_STATS=1.
pub fn report() {
    if !stats_on() {
        return;
    }
    let g = |c: &AtomicU64| c.load(Relaxed);
    eprintln!(
        "probit2 fx: {} searches, {} moves; depth-2 replies {}, valued in full at once {}, candidates valued in full \
         {} ({} beyond one per position), positions valued all in full {}",
        g(&STATS.searches),
        g(&STATS.moves),
        g(&STATS.replies),
        g(&STATS.unestimated),
        g(&STATS.candidates),
        g(&STATS.extra),
        g(&STATS.everything)
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding::{self, CATEGORIES};
    use crate::game::Game;
    use crate::probit::{Probit, ProbitParams};
    use crate::search::{Search, Value};
    use rand::rngs::StdRng;
    use rand::{RngExt, SeedableRng};
    use shakmaty::{Chess, Position};

    #[test]
    fn the_table_is_within_its_bound_of_the_float_ln_phi() {
        let t = table();
        let mut rng = StdRng::seed_from_u64(3);
        let mut worst: f64 = 0.0;
        for i in 0..400_000 {
            let z = if i % 4 == 0 { rng.random_range(-29.5..29.5) } else { rng.random_range(-6.0..6.0) };
            let (est, err) = ln_phi(t, z);
            let float = crate::probit::ln_cdf2(z).0;
            assert!((est - float).abs() <= err, "z {z}: {est} vs {float}, err {err}");
            worst = worst.max((est - float).abs() / err);
        }
        assert!(worst > 0.1, "the bound is far looser than needed ({worst})");
    }

    /// A probit trained on random positions, labelled mostly by material.
    fn probit(seed: u64, n: usize) -> Probit {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut m = Probit::new(&CATEGORIES, ProbitParams::default());
        let mut pos = Chess::default();
        for _ in 0..n {
            let moves = pos.legal_moves();
            if moves.is_empty() || pos.halfmoves() >= 100 {
                pos = Chess::default();
                continue;
            }
            pos.play_unchecked(moves[rng.random_range(0..moves.len())]);
            let mut label = [0.0; 3];
            // Mostly the side with more material wins, so the weights mean something.
            let white = pos.board().white().count() as i32 - pos.board().black().count() as i32;
            let c = if white > 0 && rng.random_range(0..4) > 0 { 0 } else if white < 0 && rng.random_range(0..4) > 0 { 1 } else { rng.random_range(0..3) };
            label[c] = 1.0;
            m.train(&encoding::encode(&pos), &label);
        }
        m
    }

    /// Depth-2 root values of random games' positions with the fast search and with the hybrid7 search: the same
    /// bits.
    fn same_values(fx: &FxLinear, model: &Probit, seed: u64, games: usize, plies: usize) {
        let mut rng = StdRng::seed_from_u64(seed);
        for _ in 0..games {
            let mut game = Game::new(Chess::default());
            for _ in 0..plies {
                let moves = game.position().legal_moves();
                if moves.is_empty() || game.position().halfmoves() >= 100 {
                    break;
                }
                let mut fast = Search::new(model, &game).valuing(Value::LogOdds, crate::calibration::Calibration::NONE).with_fx(Some(fx));
                let a = fast.root_values(&game, 2);
                let mut old = Search::new(model, &game).valuing(Value::LogOdds, crate::calibration::Calibration::NONE).with_fx(Some(fx));
                let b = old.root_values_fx_full(&game, 2);
                let bits = |v: &[f64]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
                assert_eq!(bits(&a), bits(&b), "fast {a:?} vs full {b:?}");
                let m = moves[rng.random_range(0..moves.len())];
                game.play(m);
            }
        }
    }

    #[test]
    fn depth_two_values_are_the_full_searchs_bit_for_bit() {
        for (seed, cal) in [
            (5u64, None),
            (6, Some(Calibration { slope_winner: 0.8, slope_decisive: 0.3, offsets: [0.4, -0.2] })),
            (7, Some(Calibration { slope_winner: 1.7, slope_decisive: -0.4, offsets: [-0.1, 0.3] })),
        ] {
            let model = probit(seed, 20_000);
            let mut fx = model.fixed().unwrap();
            if let Some(c) = cal {
                fx = fx.with_calibration(c);
            }
            same_values(&fx, &model, seed, 6, 60);
        }
    }

    #[test]
    fn estimates_are_within_their_bounds() {
        let model = probit(11, 30_000);
        let fx = model.fixed().unwrap().with_calibration(Calibration { slope_winner: 1.1, slope_decisive: 0.2, offsets: [0.3, 0.1] });
        let mut rng = StdRng::seed_from_u64(12);
        let mut pos = Chess::default();
        let mut n = 0;
        for _ in 0..20_000 {
            let moves = pos.legal_moves();
            if moves.is_empty() || pos.halfmoves() >= 100 {
                pos = Chess::default();
                continue;
            }
            pos.play_unchecked(moves[rng.random_range(0..moves.len())]);
            let s = fx.full(&encoding::encode(&pos));
            for white in [true, false] {
                if let Some((v, err)) = estimate(&fx, &s, white) {
                    let exact = fx.value(&s, white);
                    assert!((exact - v).abs() <= err, "{exact} vs {v} +- {err}");
                    n += 1;
                }
            }
        }
        assert!(n > 30_000, "only {n} estimates");
    }

    #[test]
    fn known_replies_and_ties_follow_the_loop() {
        let model = probit(13, 5_000);
        let fx = model.fixed().unwrap();
        let s = fx.full(&encoding::encode(&Chess::default()));
        let v = fx.value(&s, true);
        // Two equal replies (one estimated), a known one larger, a known one smaller.
        let mut replies = vec![Reply::of(&fx, s, true), Reply::known(v + 1.0), Reply::of(&fx, s, true), Reply::known(v - 0.5)];
        assert_eq!(best(&fx, &mut replies).to_bits(), (-(v - 0.5)).to_bits());
        let mut replies = vec![Reply::of(&fx, s, true), Reply::known(v + 1.0)];
        assert_eq!(best(&fx, &mut replies).to_bits(), (-v).to_bits());
        let mut replies = vec![Reply::known(f64::NAN), Reply::of(&fx, s, true)];
        assert_eq!(best(&fx, &mut replies).to_bits(), (-v).to_bits());
    }
}
