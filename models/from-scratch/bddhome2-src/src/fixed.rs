//! Fixed-point incremental evaluation of linear models (modelopt prototype).
//!
//! A logistic regression's logits are a bias plus one weight per feature. Here each weight is rounded once to a
//! multiple of 2^-32 and kept as an integer, so a position's sums are exact integers: the same whatever order the
//! terms are added in. A child position's sums are its parent's plus the changes of the few features the move
//! changed (2-4 squares, the side to move, en passant, castling) instead of a fresh sum over all 70 features.
//! Being exact, the sums do not depend on the path that reached the position, so games stay repeatable.
//!
//! The value of a position for the side to move, ln P(win) - ln P(loss), is taken from the logits directly
//! (z_win - z_loss, after the model's calibration), which equals the probabilities' log-odds without the
//! exp and ln.
//!
//! Three model kinds are linear in this way: logistic regression (its logits), naive Bayes with cached
//! probabilities (its log-probabilities) and probit (the mean and variance of its latent variable, from which
//! the normal CDF gives the probabilities).
//!
//! The mode comes from the environment variable CHESSAI_FX: `on` (default: fixed-point sums and direct values),
//! `direct` (float sums as before, direct values from the logits), `off` (hybrid4's arithmetic). CHESSAI_FX_CHECK=1
//! checks every incremental step against a full encoding and sum (slow; for tests).

use std::sync::OnceLock;

use shakmaty::{CastlingSide, Chess, Color, EnPassantMode, Piece, Position, Role};

use crate::calibration::Calibration;
use crate::encoding::{self, Encoded, FEATURES};

/// Weights are held in units of 2^-SHIFT.
pub const SHIFT: i32 = 32;
const SCALE: f64 = (1u64 << SHIFT) as f64;

/// Per-class sums (3 classes, padded to 4 lanes).
pub type Sums = [i64; 4];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Off,
    Direct,
    On,
}

pub fn mode() -> Mode {
    static MODE: OnceLock<Mode> = OnceLock::new();
    *MODE.get_or_init(|| match std::env::var("CHESSAI_FX").as_deref() {
        Ok("off") => Mode::Off,
        Ok("direct") => Mode::Direct,
        Ok("on") | Err(_) => Mode::On,
        Ok(other) => panic!("CHESSAI_FX must be on, direct or off, not {other:?}"),
    })
}

fn checking() -> bool {
    static CHECK: OnceLock<bool> = OnceLock::new();
    *CHECK.get_or_init(|| std::env::var("CHESSAI_FX_CHECK").is_ok_and(|v| v == "1"))
}

/// How a model's sums give its log-probabilities.
#[derive(Clone, Copy, Debug)]
pub enum Head {
    /// The sums are the classes' logits (logistic regression) or unnormalised log-probabilities (naive Bayes).
    Logits,
    /// Lanes 0 and 1 are the latent variable's mean and its weights' variance (probit); `beta2` is added to the
    /// variance and `eps` is the draw margin.
    Probit { eps: f64, beta2: f64 },
}

/// A linear model over the encoded features in fixed point, with an optional calibration of its logits.
#[derive(Clone, Debug)]
pub struct FxLinear {
    /// Row 0: the biases; row starts[f] + v: feature f's value v. Each row holds the classes' weights.
    table: Vec<Sums>,
    starts: [u16; FEATURES],
    head: Head,
    cal: Option<Calibration>,
}

/// The weight `w` in units of 2^-SHIFT, rounded to the nearest (ties away from 0); None if it is out of range.
fn quantize(w: f64) -> Option<i64> {
    (w.is_finite() && w.abs() < (1u64 << 20) as f64).then(|| (w * SCALE).round() as i64)
}

impl FxLinear {
    /// From the weights of up to 4 outputs: `rows[i][c]` is output c's weight of index i (0: the bias; starts[f] + v:
    /// feature f's value v). None if a weight is out of the fixed-point range (|w| >= 2^20, or not finite).
    pub fn from_rows(rows: impl Iterator<Item = [f64; 4]>, starts: impl Fn(usize) -> usize, head: Head) -> Option<FxLinear> {
        let mut table = Vec::new();
        for row in rows {
            let mut q = [0i64; 4];
            for c in 0..4 {
                q[c] = quantize(row[c])?;
            }
            table.push(q);
        }
        let starts = std::array::from_fn(|f| u16::try_from(starts(f)).unwrap());
        Some(FxLinear { table, starts, head, cal: None })
    }

    /// From a logistic regression's weights: `class_num` (at most 4) blocks of `width`, each the bias then feature
    /// f's values from `starts[f]`.
    pub fn from_weights(class_num: usize, starts: &[usize], width: usize, weights: &[f64]) -> Option<FxLinear> {
        assert!(class_num <= 4 && starts.len() == FEATURES && weights.len() == class_num * width);
        let rows = (0..width).map(|i| std::array::from_fn(|c| if c < class_num { weights[c * width + i] } else { 0.0 }));
        FxLinear::from_rows(rows, |f| starts[f], Head::Logits)
    }

    pub fn with_calibration(mut self, cal: Calibration) -> FxLinear {
        self.cal = Some(cal);
        self
    }

    /// The sums of a position: the biases plus each feature's weights.
    pub fn full(&self, enc: &Encoded) -> Sums {
        let mut s = self.table[0];
        for (f, &v) in enc.iter().enumerate() {
            let row = &self.table[self.starts[f] as usize + v as usize];
            for c in 0..4 {
                s[c] += row[c];
            }
        }
        s
    }

    /// Changes feature `f` of `acc` to `new`, updating its sums.
    #[inline]
    fn set(&self, acc: &mut Acc, f: usize, new: u8) {
        let old = acc.enc[f];
        self.moved(&mut acc.sums, f, old, new);
        acc.enc[f] = new;
    }

    /// The model's own log-probabilities of [white win, black win, draw] (as its `predict_log`).
    #[inline]
    fn log_probs(&self, sums: &Sums) -> [f64; 3] {
        match self.head {
            Head::Logits => [sums[0] as f64 / SCALE, sums[1] as f64 / SCALE, sums[2] as f64 / SCALE],
            Head::Probit { eps, beta2 } => crate::probit::log_probs_of_latent(sums[0] as f64 / SCALE, sums[1] as f64 / SCALE + beta2, eps),
        }
    }

    /// The (calibrated) logits of [white win, black win, draw], as the model's `predict_log`.
    #[inline]
    pub fn logits(&self, sums: &Sums) -> [f64; 3] {
        let raw = self.log_probs(sums);
        match &self.cal {
            Some(cal) => cal.logits(&raw),
            None => raw,
        }
    }

    /// ln P(win) - ln P(loss) for the side to move, from the logits. For logits (LR, NB) it is linear in their
    /// difference d = l_white - l_black, taken from the exact sums: d itself, or calibrated
    /// z_white - z_black = slope_winner * d + offset_white - offset_black.
    #[inline]
    pub fn value(&self, sums: &Sums, white_to_move: bool) -> f64 {
        let v = match self.head {
            Head::Logits => {
                let d = (sums[0] - sums[1]) as f64 / SCALE;
                match &self.cal {
                    Some(cal) => cal.slope_winner * d + (cal.offsets[0] - cal.offsets[1]),
                    None => d,
                }
            }
            Head::Probit { .. } => {
                let z = self.logits(sums);
                z[0] - z[1]
            }
        };
        if white_to_move { v } else { -v }
    }

    /// The sums after changing feature `f` from `old` to `new`.
    #[inline]
    fn moved(&self, sums: &mut Sums, f: usize, old: u8, new: u8) {
        if old != new {
            let start = self.starts[f] as usize;
            let (a, b) = (&self.table[start + new as usize], &self.table[start + old as usize]);
            for c in 0..4 {
                sums[c] += a[c] - b[c];
            }
        }
    }

    /// The probabilities of [white win, black win, draw], as the model's `predict`: the softmax of the logits, or
    /// an uncalibrated probit's normal CDFs.
    pub fn probs(&self, sums: &Sums) -> [f64; 3] {
        if let (None, Head::Probit { eps, beta2 }) = (&self.cal, self.head) {
            return crate::probit::probs_of_latent(sums[0] as f64 / SCALE, sums[1] as f64 / SCALE + beta2, eps);
        }
        let z = self.logits(sums);
        let max = z.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let e = z.map(|v| (v - max).exp());
        let sum: f64 = e.iter().sum();
        e.map(|v| v / sum)
    }
}

/// A position's encoding and its sums under an `FxLinear`.
#[derive(Clone, Copy, Debug)]
pub struct Acc {
    pub enc: Encoded,
    pub sums: Sums,
}

/// A square's feature value: 0 empty, white pawn..king 1-6, black 7-12 (as `encoding::encode`).
#[inline]
fn code(piece: Option<Piece>) -> u8 {
    match piece {
        None => 0,
        Some(p) => {
            let role = match p.role {
                Role::Pawn => 1,
                Role::Knight => 2,
                Role::Bishop => 3,
                Role::Rook => 4,
                Role::Queen => 5,
                Role::King => 6,
            };
            if p.color == Color::Black { role + 6 } else { role }
        }
    }
}

impl Acc {
    /// A position's accumulator, summed in full.
    pub fn new(fx: &FxLinear, pos: &Chess) -> Acc {
        let enc = encoding::encode(pos);
        Acc { enc, sums: fx.full(&enc) }
    }

    /// Calls `f(feature, new value)` for the features of `next`, a position one move after `pos` (whose
    /// accumulator this is), that may differ from `pos`'s: the squares whose piece changed, the side to move, the
    /// en passant file and, if the rights changed, castling. Each feature at most once.
    #[inline]
    fn changes(pos: &Chess, next: &Chess, mut f: impl FnMut(usize, u8)) {
        let (a, b) = (pos.board(), next.board());
        let changed = (a.white() ^ b.white())
            | (a.black() ^ b.black())
            | (a.pawns() ^ b.pawns())
            | (a.knights() ^ b.knights())
            | (a.bishops() ^ b.bishops())
            | (a.rooks() ^ b.rooks())
            | (a.queens() ^ b.queens())
            | (a.kings() ^ b.kings());
        for sq in changed {
            f(sq as usize, code(b.piece_at(sq)));
        }
        f(64, next.turn().is_black() as u8);
        f(65, next.ep_square(EnPassantMode::Always).map_or(0, |sq| sq.file() as u8 + 1));
        if pos.castles().castling_rights() != next.castles().castling_rights() {
            let castles = next.castles();
            f(66, castles.has(Color::White, CastlingSide::QueenSide) as u8);
            f(67, castles.has(Color::Black, CastlingSide::QueenSide) as u8);
            f(68, castles.has(Color::White, CastlingSide::KingSide) as u8);
            f(69, castles.has(Color::Black, CastlingSide::KingSide) as u8);
        }
    }

    /// The accumulator of `next`, a position one move after `pos` (whose accumulator this is).
    pub fn child(&self, fx: &FxLinear, pos: &Chess, next: &Chess) -> Acc {
        let mut acc = *self;
        Acc::changes(pos, next, |f, v| fx.set(&mut acc, f, v));
        if checking() {
            let enc = encoding::encode(next);
            assert_eq!(acc.enc, enc, "incremental encoding differs from encode()");
            assert_eq!(acc.sums, fx.full(&enc), "incremental sums differ from the full sums");
        }
        acc
    }

    /// Just the sums of `child` (no encoding), for leaves.
    pub fn child_sums(&self, fx: &FxLinear, pos: &Chess, next: &Chess) -> Sums {
        let mut sums = self.sums;
        Acc::changes(pos, next, |f, v| fx.moved(&mut sums, f, self.enc[f], v));
        if checking() {
            assert_eq!(sums, fx.full(&encoding::encode(next)), "incremental sums differ from the full sums");
        }
        sums
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding::CATEGORIES;
    use crate::logreg::Logreg;
    use rand::{RngExt, SeedableRng};
    use rand::rngs::StdRng;

    #[test]
    fn incremental_sums_equal_full_sums_on_random_games() {
        let mut lr = Logreg::new(3, &CATEGORIES, 0.1);
        lr.randomize(5, 1.0);
        let fx = lr.fixed().unwrap();
        let mut rng = StdRng::seed_from_u64(9);
        for _ in 0..200 {
            let mut pos = Chess::default();
            let mut acc = Acc::new(&fx, &pos);
            for _ in 0..200 {
                let moves = pos.legal_moves();
                if moves.is_empty() {
                    break;
                }
                // Every child, as candidates and the search compute them.
                for &m in &moves {
                    let mut next = pos.clone();
                    next.play_unchecked(m);
                    let child = acc.child(&fx, &pos, &next);
                    let enc = encoding::encode(&next);
                    assert_eq!(child.enc, enc);
                    assert_eq!(child.sums, fx.full(&enc));
                }
                let m = moves[rng.random_range(0..moves.len())];
                let mut next = pos.clone();
                next.play_unchecked(m);
                acc = acc.child(&fx, &pos, &next);
                pos = next;
            }
        }
    }

    /// Positions of a random game, labelled by a coin.
    fn random_positions(seed: u64, n: usize) -> Vec<(Encoded, [f64; 3])> {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut pos = Chess::default();
        let mut out = Vec::new();
        while out.len() < n {
            let moves = pos.legal_moves();
            if moves.is_empty() || pos.halfmoves() >= 100 {
                pos = Chess::default();
                continue;
            }
            pos.play_unchecked(moves[rng.random_range(0..moves.len())]);
            let mut label = [0.0; 3];
            label[rng.random_range(0..3)] = 1.0;
            out.push((encoding::encode(&pos), label));
        }
        out
    }

    #[test]
    fn probit_and_naive_bayes_fixed_logits_match_their_float_ones() {
        let data = random_positions(4, 3000);
        let mut probit = crate::probit::Probit::new(&CATEGORIES, crate::probit::ProbitParams::default());
        let mut nb = crate::cnbp::Cnbp::new(3, &CATEGORIES, 1.0, true);
        for (x, y) in &data {
            probit.train(x, y);
            nb.train(x, y);
        }
        nb.refresh();
        let (fp, fn_) = (probit.fixed().unwrap(), nb.fixed().unwrap());
        for (x, _) in data.iter().step_by(37) {
            let (mut a, mut b) = ([0.0; 3], [0.0; 3]);
            probit.predict_log(x, &mut a);
            nb.predict_log(x, &mut b);
            let (fa, fb) = (fp.logits(&fp.full(x)), fn_.logits(&fn_.full(x)));
            let (mut pa, mut pb) = ([0.0; 3], [0.0; 3]);
            probit.predict(x, &mut pa);
            nb.predict(x, &mut pb);
            let (qa, qb) = (fp.probs(&fp.full(x)), fn_.probs(&fn_.full(x)));
            for c in 0..3 {
                assert!((a[c] - fa[c]).abs() < 1e-6, "probit {a:?} vs {fa:?}");
                assert!((b[c] - fb[c]).abs() < 1e-6, "nb {b:?} vs {fb:?}");
                assert!((pa[c] - qa[c]).abs() < 1e-9, "probit probs {pa:?} vs {qa:?}");
                assert!((pb[c] - qb[c]).abs() < 1e-9, "nb probs {pb:?} vs {qb:?}");
            }
        }
    }

    #[test]
    fn fixed_logits_match_float_logits_closely() {
        let mut lr = Logreg::new(3, &CATEGORIES, 0.1);
        lr.randomize(3, 1.0);
        let fx = lr.fixed().unwrap();
        let pos = Chess::default();
        let enc = encoding::encode(&pos);
        let mut float = [0.0; 3];
        lr.predict_log(&enc, &mut float);
        let fixed = fx.logits(&fx.full(&enc));
        for c in 0..3 {
            assert!((float[c] - fixed[c]).abs() < 1e-8, "{float:?} vs {fixed:?}");
        }
    }
}
