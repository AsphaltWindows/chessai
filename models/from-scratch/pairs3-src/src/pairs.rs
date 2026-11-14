//! S7: data-chosen pairwise features for the flat probit (analyst 6.97, 2026-11-12; the user approved it ~21:50:
//! "Let's try pairwise features also"). chessai-pairs2: pairs1 with the corrected statistic (analyst 6.100, ~22:56).
//! chessai-pairs3 (2026-11-13, analyst 6.111): selection windows and the contrast's own standard error (below).
//!
//! A pair is the conjunction of two active indicator values: feature i has value a AND feature j has value b (i < j).
//! A chosen pair is one more binary input of the probit, with its own Gaussian weight N(mu, s2): it enters the
//! latent's mean and variance when both values are present, the dynamics (tau) widen it, and the truncation moves it,
//! exactly as the model's own weights (see `probit`).
//!
//! Which pairs. Every pair-value is a candidate (356,642 over the 70 features; a position has 2,415 active). After
//! each example the probit knows the standardized shift v of its latent's mean (`Truncation::tempered`'s v, summed
//! over the label's results): the score of a new weight at zero. pairs1 ranked the candidates by S / sqrt(n), S the
//! sum of v over the examples a candidate was active in, and that z was inflated (mean z^2 4.8 where independent
//! examples give 1): a game's positions share a candidate over many plies with correlated v, and a candidate's S
//! carries both its values' own (main-effect) residuals. pairs2 (6.100) tested the interaction contrast
//! I = S_ab/n_ab - S_a/n_a - S_b/n_b + S/n, with (S, n) of v per indicator value and over all examples, clustered by
//! game, but over the SE of its first term alone, sqrt(sum over the pair's games of (G - m mu0)^2) / n_ab (G the sum
//! of v over a game's positions with the pair active, m their number, mu0 = S_a/n_a + S_b/n_b - S/n): right for a
//! rare pair, too large for a frequent one, whose examples are much of its values' (null mean z^2 1.0 under 100
//! games, 0.78 at 100-999, 0.25 at 1,000+), and its sums ran from the model's start while the model moved.
//! pairs3 (6.111):
//!   windows: the statistics restart at every checkpoint, so each selection sees only the examples since the last;
//!   the contrast's SE, clustered by game: I is a sum over games of each game's influence
//!     Psi_g = (G_ab - m_ab mu0)/n_ab - (G_a - m_a Y_a)/n_a - (G_b - m_b Y_b)/n_b + (G - m Y)/n
//!   (G, m the game's sums for the pair, for each of its two values and for the whole game; Y = S/n the means; the
//!   pair's own term centred at the null's prediction mu0, as pairs2's), and z = I / sqrt(sum over games of Psi_g^2).
//!   Psi_g is linear in the game's eight sums u = (G_ab, m_ab, G_a, m_a, G_b, m_b, G, m), so z needs only the
//!   window's second moments of u, which the statistics keep: per candidate over its games (u_ab with itself and
//!   with the other six), per candidate over all games (its two values' with each other: a game can hold both
//!   values in different positions) and per value over all games (with itself and with the game's);
//!   --pairs-contrast cells: the 2x2 contrast of the four disjoint cells (both values, a not b, b not a, neither),
//!   J = Y_ab - Y_a~b - Y_~ab + Y_~a~b, from the same sums (each cell's game sums are differences of u's): zero under
//!   any additive effect of the two values however correlated they are, where I is zero only when they are
//!   independent (I = beta (P(a) - P(a|b)) + gamma (P(b) - P(b|a)) under v = beta a + gamma b), and undefined when a
//!   cell is empty (one value implies the other: the pair would duplicate a value). The contrast is training state.
//!   Defaults cap 300 (was 500), drop 1.0 sd (was 0.5); a pair's own examples since chosen are counted apart from
//!   the windows' statistics, for the drop rule.
//! pairs3f (the same day, from pairs3's null check): the selection divides the contrast by the larger of its SE
//! clustered by game and its per-example SE (the same influence summed in squares over examples instead of games;
//! the cells' sums of v^2 give it). A side-to-move pair compares interleaved positions of the same games, whose label
//! is one, so the game's own variation cancels from its clustered SE, and effects of a few thousandths of v became
//! "significant" (null: 20 such pairs a window at |z| 13-26, each in half the positions, held-out log-loss 0.003
//! worse than plain); per example such an effect is ~1 SE. So a pair needs both: significance (clustered) and an
//! effect worth a weight (the per-example z^2 / 2 is about the log-likelihood it adds). The checkpoint lines' bands
//! stay the clustered z's (the SE's calibration); "over the bar" counts both (--pairs-se clustered: pairs3's rule).
//! Training order is untouched: train shuffles a round's positions as always, and the probit keeps each example's v
//! (8 bytes) until the round's end, when it sums them by game (the positions come with their games' ids, as trees get
//! them: `Classifier::scores_rounds`) and runs the checkpoints the round crossed: one per `every` examples counted
//! from the model's start, at the end of the round that reaches it. A checkpoint first drops each pair with at least
//! `drop_after` (3M) examples of its own since it was chosen and |mu| below `drop` (1.0) x its sd, then ranks the
//! candidates with |z| above `z` (5) by |z| (ties by index) and adds the first `k` (20), at most `cap` (300) pairs in
//! all, with mu 0 and sd `shrink` (0.5) x the probit's prior; then the window's statistics restart. Its log line
//! gives the window's mean z^2 by the candidates' games (under 100, 100-999, 1,000+: each 1 when nothing interacts).
//! Nothing names a pair: the data choose (zero knowledge). With `sample` M above 1, only the candidates whose index
//! hashes to 0 mod M are tracked (a fixed, knowledge-free subset, for speed).
//!
//! Files. The model file holds the options, the examples trained and the chosen pairs with their weights (probit
//! model version 2, as pairs1's and pairs2's, byte for byte the same format; a model without pairs stays version 1).
//! The statistics are training state: `save_state` writes them beside the model (`probitN.state`, kind
//! probit-pairs3, ~63 MB: the contrast, the pairs' own examples, the window's sums), and training on from a model
//! file reads the state beside it, so a run resumes exactly. Play needs only the model file.
//!
//! Order. A position's chosen pairs are summed after the model's own weights, by their first feature i ascending,
//! then in the order they were chosen: `add_active` (the latent) and `for_active` (training) visit them the same way.

use std::fmt::Write as _;
use std::io;
use std::path::Path;

use crate::encoding::Encoded;
use crate::state::{Reader, Writer};
use crate::textfmt::{Tokens, invalid};

/// Which contrast a candidate's z tests (--pairs-contrast; see the module documentation). Training state: kept in
/// the state file, not the model file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Contrast {
    /// 6.100's I = Y_ab - Y_a - Y_b + Y (the pair's mean less its two values' plus the overall mean).
    Marginal,
    /// The four disjoint cells' J = Y_ab - Y_a~b - Y_~ab + Y_~a~b.
    Cells,
}

/// The options of the pairs (--pairs-*); see the module documentation. The model file keeps all but `contrast` and
/// `floor` (training state).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PairOptions {
    pub k: usize,
    pub z: f64,
    pub cap: usize,
    pub shrink: f64,
    pub drop: f64,
    pub every: u64,
    pub drop_after: u64,
    pub sample: u32,
    pub contrast: Contrast,
    /// pairs3f: select on the contrast over the larger of its clustered and its per-example (iid) SE (see the
    /// module documentation); false: the clustered SE alone (pairs3).
    pub floor: bool,
}

impl Default for PairOptions {
    fn default() -> Self {
        PairOptions {
            k: 20,
            z: 5.0,
            cap: 300,
            shrink: 0.5,
            drop: 1.0,
            every: 1_000_000,
            drop_after: 3_000_000,
            sample: 1,
            contrast: Contrast::Marginal,
            floor: true,
        }
    }
}

impl PairOptions {
    pub fn check(&self) -> Result<(), String> {
        if !(self.z.is_finite() && self.z >= 0.0) {
            return Err(format!("--pairs-z must be a number from 0, not {}", self.z));
        }
        if !(self.shrink.is_finite() && self.shrink > 0.0) {
            return Err(format!("--pairs-shrink must be above 0, not {}", self.shrink));
        }
        if !(self.drop.is_finite() && self.drop >= 0.0) {
            return Err(format!("--pairs-drop must be a number from 0, not {}", self.drop));
        }
        if self.every == 0 {
            return Err("--pairs-every must be at least 1".into());
        }
        if self.sample == 0 {
            return Err("--pairs-sample must be at least 1".into());
        }
        Ok(())
    }
}

/// The current round's counts, for `report`.
#[derive(Clone, Debug, Default)]
struct Round {
    examples: u64,
    active: u64,
    checkpoints: u64,
    added: Vec<f64>,
    dropped: u64,
    /// One line per checkpoint: the pairs dropped (mean and sd) and added (z), as i:a&j:b.
    lines: Vec<String>,
}

/// A candidate's sums over the window's games it was active in, from its game sums (G, m) and those of its first
/// value (Ga, ma), its second (Gb, mb) and the whole game (G0, m0): G, m, G^2, m G, m^2, 1 (pairs2's six), then
/// G Ga, G ma, m Ga, m ma, G Gb, G mb, m Gb, m mb, G G0, G m0, m G0, m m0 (see the module documentation), then
/// (pairs3f) the sum of v^2 over its examples.
const W: usize = 19;
/// A candidate's two values' game sums over all the window's games: Ga Gb, Ga mb, ma Gb, ma mb.
const WP: usize = 4;
/// A value's (or, at weight index 0, every example's) game sums over all the window's games: S = sum of G, n = sum
/// of m, G^2, G m, m^2, G G0, G m0, m G0, m m0, then (pairs3f) the sum of v^2 over its examples.
const WM: usize = 10;

/// The training state: the window's sums (see `W`, `WP` and `WM`), per candidate, per candidate's two values and per
/// weight index (0: every example; starts[i] + a: the examples with feature i at value a), and its games.
#[derive(Clone, Debug, PartialEq)]
struct Stats {
    cand: Vec<[f64; W]>,
    pair: Vec<[f64; WP]>,
    marg: Vec<[f64; WM]>,
    games: u64,
}

#[derive(Clone, Debug)]
pub struct Pairs {
    pub opts: PairOptions,
    cats: Vec<u8>,
    /// Each feature's first weight index in the probit (after the bias at 0), as `Probit::starts`.
    starts: Vec<usize>,
    /// For features i < j: the index of candidate (i, 0, j, 0); candidate (i, a, j, b) is base + a cats[j] + b.
    base: Vec<u32>,
    /// The blocks (i, j) by their first candidate, ascending, to decode a candidate.
    blocks: Vec<(u32, u8, u8)>,
    candidates: usize,
    /// The chosen pairs [i, a, j, b] in the order chosen, and their weights [mu, s2].
    list: Vec<[u8; 4]>,
    weights: Vec<[f64; 2]>,
    /// Per weight index of a value (starts[i] + a): the chosen pairs whose first value it is, (j, b, slot).
    index: Vec<Vec<(u8, u8, u32)>>,
    /// Per weight index of a value: its (feature, value); index 0 (every example) (u8::MAX, 0).
    owner: Vec<(u8, u8)>,
    /// Examples trained with pairs on, from the start.
    trained: u64,
    /// Training only (in the state, not the model file): each chosen pair's examples since it was chosen, for the
    /// drop rule (the windows' statistics restart).
    own: Vec<u64>,
    /// Training only; None in a model loaded to play.
    stats: Option<Box<Stats>>,
    /// The current round's v, in training order, until `end_round` sums them by game.
    pending: Vec<f64>,
    round: Round,
}

fn hash_keep(c: usize, sample: u32) -> bool {
    sample <= 1 || ((c as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 33) % sample as u64 == 0
}

/// The contrast's name in the state file and on the command line.
pub fn contrast_name(c: Contrast) -> &'static str {
    match c {
        Contrast::Marginal => "marginal",
        Contrast::Cells => "cells",
    }
}

/// `contrast_name`'s inverse.
pub fn parse_contrast(s: &str) -> Option<Contrast> {
    [Contrast::Marginal, Contrast::Cells].into_iter().find(|&c| contrast_name(c) == s)
}

/// The selection's SE (`PairOptions::floor`) by name, in the state file and on the command line.
pub fn se_name(floor: bool) -> &'static str {
    if floor { "floored" } else { "clustered" }
}

/// `se_name`'s inverse.
pub fn parse_se(s: &str) -> Option<bool> {
    [true, false].into_iter().find(|&f| se_name(f) == s)
}

/// Adds a game's sums (g, m, and q the sum of v^2) of a value (or of every example) to the value's window sums `w`
/// (see `WM`), with the game's (g0, m0).
fn add_marg(w: &mut [f64; WM], [g, m, q]: [f64; 3], g0: f64, m0: f64) {
    w[0] += g;
    w[1] += m;
    w[2] += g * g;
    w[3] += g * m;
    w[4] += m * m;
    w[5] += g * g0;
    w[6] += g * m0;
    w[7] += m * g0;
    w[8] += m * m0;
    w[9] += q;
}

impl Pairs {
    /// No pairs yet, training from scratch.
    pub fn new(categories: &[u8], opts: PairOptions) -> Self {
        let mut p = Self::shape(categories, opts);
        p.stats = Some(p.zero_stats());
        p
    }

    fn shape(categories: &[u8], opts: PairOptions) -> Self {
        let f = categories.len();
        let mut starts = Vec::with_capacity(f);
        let mut width = 1;
        for &c in categories {
            starts.push(width);
            width += c as usize;
        }
        let mut owner = vec![(u8::MAX, 0u8); width];
        for (i, &c) in categories.iter().enumerate() {
            for a in 0..c {
                owner[starts[i] + a as usize] = (i as u8, a);
            }
        }
        let mut base = vec![u32::MAX; f * f];
        let mut blocks = Vec::new();
        let mut next = 0usize;
        for i in 0..f {
            for j in i + 1..f {
                base[i * f + j] = next as u32;
                blocks.push((next as u32, i as u8, j as u8));
                next += categories[i] as usize * categories[j] as usize;
            }
        }
        Pairs {
            opts,
            cats: categories.to_vec(),
            starts,
            base,
            blocks,
            candidates: next,
            list: Vec::new(),
            weights: Vec::new(),
            index: vec![Vec::new(); width],
            owner,
            trained: 0,
            own: Vec::new(),
            stats: None,
            pending: Vec::new(),
            round: Round::default(),
        }
    }

    fn zero_stats(&self) -> Box<Stats> {
        Box::new(Stats {
            cand: vec![[0.0; W]; self.candidates],
            pair: vec![[0.0; WP]; self.candidates],
            marg: vec![[0.0; WM]; self.index.len()],
            games: 0,
        })
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn trained(&self) -> u64 {
        self.trained
    }

    /// The number of candidates (pair-values).
    pub fn candidates(&self) -> usize {
        self.candidates
    }

    /// The chosen pairs and their weights.
    pub fn chosen(&self) -> impl Iterator<Item = ([u8; 4], [f64; 2])> + '_ {
        self.list.iter().copied().zip(self.weights.iter().copied())
    }

    fn candidate(&self, [i, a, j, b]: [u8; 4]) -> usize {
        let f = self.cats.len();
        self.base[i as usize * f + j as usize] as usize + a as usize * self.cats[j as usize] as usize + b as usize
    }

    fn decode(&self, c: usize) -> [u8; 4] {
        let k = self.blocks.partition_point(|&(b, _, _)| b as usize <= c) - 1;
        let (b, i, j) = self.blocks[k];
        let off = c - b as usize;
        let cj = self.cats[j as usize] as usize;
        [i, (off / cj) as u8, j, (off % cj) as u8]
    }

    fn rebuild_index(&mut self) {
        self.index.iter_mut().for_each(Vec::clear);
        for (slot, &[i, a, j, b]) in self.list.iter().enumerate() {
            self.index[self.starts[i as usize] + a as usize].push((j, b, slot as u32));
        }
    }

    /// Adds the position's chosen pairs' means and variances to `m` and `var`, one by one.
    #[inline]
    pub fn add_active(&self, data: &[u8], m: &mut f64, var: &mut f64) {
        if self.list.is_empty() {
            return;
        }
        for (i, &v) in data.iter().enumerate() {
            for &(j, b, slot) in &self.index[self.starts[i] + v as usize] {
                if data[j as usize] == b {
                    let [mu, s2] = self.weights[slot as usize];
                    *m += mu;
                    *var += s2;
                }
            }
        }
    }

    /// Calls `f` on each of the position's chosen pairs' weights, in `add_active`'s order.
    #[inline]
    pub fn for_active(&mut self, data: &[u8], mut f: impl FnMut(&mut [f64; 2])) {
        if self.list.is_empty() {
            return;
        }
        for (i, &v) in data.iter().enumerate() {
            for &(j, b, slot) in &self.index[self.starts[i] + v as usize] {
                if data[j as usize] == b {
                    f(&mut self.weights[slot as usize]);
                }
            }
        }
    }

    #[cfg(test)]
    fn count_active(&self, data: &[u8]) -> u64 {
        let mut n = 0;
        for (i, &v) in data.iter().enumerate() {
            for &(j, b, _) in &self.index[self.starts[i] + v as usize] {
                n += (data[j as usize] == b) as u64;
            }
        }
        n
    }

    /// Multiplies the pairs' variances by `factor` (`Probit::forget`).
    pub fn forget(&mut self, factor: f64) {
        self.weights.iter_mut().for_each(|w| w[1] *= factor);
    }

    /// After an example whose latent's standardized mean shift was `v`: kept until the round's end (`end_round`).
    pub fn observe(&mut self, data: &[u8], v: f64) {
        assert!(self.stats.is_some(), "pairs: training a model loaded without its training state");
        self.pending.push(v);
        self.round.examples += 1;
        let mut n = 0;
        for (i, &x) in data.iter().enumerate() {
            for &(j, b, slot) in &self.index[self.starts[i] + x as usize] {
                if data[j as usize] == b {
                    self.own[slot as usize] += 1;
                    n += 1;
                }
            }
        }
        self.round.active += n;
        self.trained += 1;
    }

    /// At a round's end, given its examples in training order with their games' ids (as `observe` saw them): sums
    /// their v by game into the window's statistics, then runs the checkpoints the round crossed (new pairs get sd
    /// shrink x `prior`).
    pub fn end_round(&mut self, data: &[(Encoded, [f64; 3], u64)], prior: f64) {
        let pending = std::mem::take(&mut self.pending);
        assert_eq!(pending.len(), data.len(), "pairs: the round's examples and the v's kept for them differ in number");
        let before = self.trained - data.len() as u64;
        let mut order: Vec<u32> = (0..data.len() as u32).collect();
        order.sort_by_key(|&e| data[e as usize].2); // stable: a game's examples in training order
        let f = self.cats.len();
        let keep = |c: usize| self.opts.sample <= 1 || hash_keep(c, self.opts.sample);
        let st = self.stats.as_mut().expect("training state");
        // A game's sums: per candidate (sum, count; touched: the candidate and its two values' weight indices, in
        // the order first seen) and per weight index (gv; values: the game's values, index 0 apart).
        let mut sum = vec![[0.0f64; 2]; self.candidates]; // sum of v, of v^2
        let mut count = vec![0u32; self.candidates];
        let mut touched: Vec<(u32, u32, u32)> = Vec::new();
        let mut gv = vec![[0.0f64; 3]; self.index.len()]; // sum of v, count, sum of v^2
        let mut values: Vec<u32> = Vec::new();
        let mut k = 0;
        while k < order.len() {
            let game = data[order[k] as usize].2;
            let mut end = k;
            while end < order.len() && data[order[end] as usize].2 == game {
                end += 1;
            }
            for &e in &order[k..end] {
                let x = &data[e as usize].0;
                let v = pending[e as usize];
                let v2 = v * v;
                gv[0][0] += v;
                gv[0][1] += 1.0;
                gv[0][2] += v2;
                for i in 0..f {
                    let w = self.starts[i] + x[i] as usize;
                    if gv[w][1] == 0.0 {
                        values.push(w as u32);
                    }
                    gv[w][0] += v;
                    gv[w][1] += 1.0;
                    gv[w][2] += v2;
                }
                for i in 0..f {
                    let ai = x[i] as usize;
                    let wa = (self.starts[i] + ai) as u32;
                    let row = &self.base[i * f..i * f + f];
                    for j in i + 1..f {
                        let c = row[j] as usize + ai * self.cats[j] as usize + x[j] as usize;
                        if !keep(c) {
                            continue;
                        }
                        if count[c] == 0 {
                            touched.push((c as u32, wa, (self.starts[j] + x[j] as usize) as u32));
                        }
                        sum[c][0] += v;
                        sum[c][1] += v2;
                        count[c] += 1;
                    }
                }
            }
            let [g0, m0, _] = gv[0];
            for &(c, wa, wb) in &touched {
                let c = c as usize;
                let ([g, q], m) = (sum[c], count[c] as f64);
                let [ga, ma, _] = gv[wa as usize];
                let [gb, mb, _] = gv[wb as usize];
                let s = &mut st.cand[c];
                s[0] += g;
                s[1] += m;
                s[2] += g * g;
                s[3] += m * g;
                s[4] += m * m;
                s[5] += 1.0;
                s[6] += g * ga;
                s[7] += g * ma;
                s[8] += m * ga;
                s[9] += m * ma;
                s[10] += g * gb;
                s[11] += g * mb;
                s[12] += m * gb;
                s[13] += m * mb;
                s[14] += g * g0;
                s[15] += g * m0;
                s[16] += m * g0;
                s[17] += m * m0;
                s[18] += q;
                sum[c] = [0.0; 2];
                count[c] = 0;
            }
            touched.clear();
            // Per value over all games (and every example's at 0), then per two values of different features.
            values.sort_unstable();
            add_marg(&mut st.marg[0], gv[0], g0, m0);
            for &w in &values {
                add_marg(&mut st.marg[w as usize], gv[w as usize], g0, m0);
            }
            for (p, &w1) in values.iter().enumerate() {
                let (i, a) = self.owner[w1 as usize];
                let [g1, m1, _] = gv[w1 as usize];
                let row = &self.base[i as usize * f..i as usize * f + f];
                for &w2 in &values[p + 1..] {
                    let (j, b) = self.owner[w2 as usize];
                    if j == i {
                        continue;
                    }
                    let c = row[j as usize] as usize + a as usize * self.cats[j as usize] as usize + b as usize;
                    if !keep(c) {
                        continue;
                    }
                    let [g2, m2, _] = gv[w2 as usize];
                    let q = &mut st.pair[c];
                    q[0] += g1 * g2;
                    q[1] += g1 * m2;
                    q[2] += m1 * g2;
                    q[3] += m1 * m2;
                }
            }
            for &w in &values {
                gv[w as usize] = [0.0; 3];
            }
            gv[0] = [0.0; 3];
            values.clear();
            st.games += 1;
            k = end;
        }
        for _ in before / self.opts.every..self.trained / self.opts.every {
            self.checkpoint(prior);
        }
    }

    /// A candidate's z's from its window sums `s` and `q`, its two values' `a` and `b` and every example's `o` (see the
    /// module documentation): the contrast over the square root of the sum over games of its influence squared, a
    /// quadratic form in the second moments of the games' sums u = (G_ab, m_ab, G_a, m_a, G_b, m_b, G, m) (the
    /// clustered z), and (pairs3f) the contrast over the larger of that and the sum over examples of the influence
    /// squared (the floored z: each example's u has one of four patterns, by the disjoint cell it is in, so this sum
    /// needs only the cells' sums of v, v^2 and their counts). None without examples, with an empty cell (cells), or
    /// when the clustered form is no larger than its rounding (a pair that cannot differ from its values: the contrast
    /// is then 0 up to rounding too).
    fn z_with(contrast: Contrast, s: &[f64; W], q: &[f64; WP], a: &[f64; WM], b: &[f64; WM], o: &[f64; WM]) -> Option<(f64, f64)> {
        let (nc, na, nb, n) = (s[1], a[1], b[1], o[1]);
        if nc <= 0.0 {
            return None;
        }
        // The contrast and its influence's coefficients on u.
        let (num, k) = match contrast {
            Contrast::Marginal => {
                let (ya, yb, y) = (a[0] / na, b[0] / nb, o[0] / n);
                let mu0 = ya + yb - y;
                (s[0] / nc - mu0, [1.0 / nc, -mu0 / nc, -1.0 / na, ya / na, -1.0 / nb, yb / nb, 1.0 / n, -y / n])
            }
            Contrast::Cells => {
                // The cells a not b, b not a, neither: their sums are differences of u's.
                let (n1, n2, n3) = (na - nc, nb - nc, n - na - nb + nc);
                if n1 <= 0.0 || n2 <= 0.0 || n3 <= 0.0 {
                    return None;
                }
                let (y1, y2, y3) = ((a[0] - s[0]) / n1, (b[0] - s[0]) / n2, (o[0] - a[0] - b[0] + s[0]) / n3);
                let nu0 = y1 + y2 - y3;
                let k = [
                    1.0 / nc + 1.0 / n1 + 1.0 / n2 + 1.0 / n3,
                    -nu0 / nc - y1 / n1 - y2 / n2 - y3 / n3,
                    -1.0 / n1 - 1.0 / n3,
                    y1 / n1 + y3 / n3,
                    -1.0 / n2 - 1.0 / n3,
                    y2 / n2 + y3 / n3,
                    1.0 / n3,
                    -y3 / n3,
                ];
                (s[0] / nc - nu0, k)
            }
        };
        // The second moments of u over the window's games, upper triangle by rows.
        let m: [&[f64]; 8] = [
            &[s[2], s[3], s[6], s[7], s[10], s[11], s[14], s[15]],
            &[s[4], s[8], s[9], s[12], s[13], s[16], s[17]],
            &[a[2], a[3], q[0], q[1], a[5], a[6]],
            &[a[4], q[2], q[3], a[7], a[8]],
            &[b[2], b[3], b[5], b[6]],
            &[b[4], b[7], b[8]],
            &[o[2], o[3]],
            &[o[4]],
        ];
        let (mut var, mut scale) = (0.0f64, 0.0f64);
        for (r, row) in m.iter().enumerate() {
            for (d, &x) in row.iter().enumerate() {
                let t = if d == 0 { k[r] * k[r] * x } else { 2.0 * k[r] * k[r + d] * x };
                var += t;
                scale += t.abs();
            }
        }
        if !(var.is_finite() && num.is_finite() && var > 1e-10 * scale) {
            return None;
        }
        // Per example: in cell ab, u = (v, 1, v, 1, v, 1, v, 1); a not b (0, 0, v, 1, 0, 0, v, 1); b not a
        // (0, 0, 0, 0, v, 1, v, 1); neither (0, 0, 0, 0, 0, 0, v, 1). Its influence is alpha v + beta, so the cell adds
        // alpha^2 (sum of v^2) + 2 alpha beta (sum of v) + beta^2 (count).
        let cells = [
            ([s[0], s[1], s[18]], k[0] + k[2] + k[4] + k[6], k[1] + k[3] + k[5] + k[7]),
            ([a[0] - s[0], a[1] - s[1], a[9] - s[18]], k[2] + k[6], k[3] + k[7]),
            ([b[0] - s[0], b[1] - s[1], b[9] - s[18]], k[4] + k[6], k[5] + k[7]),
            ([o[0] - a[0] - b[0] + s[0], o[1] - a[1] - b[1] + s[1], o[9] - a[9] - b[9] + s[18]], k[6], k[7]),
        ];
        let iid: f64 = cells.iter().map(|&([sv, n, sq], al, be)| al * al * sq + 2.0 * al * be * sv + be * be * n).sum();
        Some((num / var.sqrt(), num / var.max(iid).sqrt()))
    }

    /// Calls `f(c, z, zf)` for every candidate with a z (clustered; floored), block by block.
    fn each_z(&self, st: &Stats, mut f: impl FnMut(usize, f64, f64)) {
        let o = &st.marg[0];
        for &(b0, i, j) in &self.blocks {
            let (i, j) = (i as usize, j as usize);
            let cj = self.cats[j] as usize;
            for a in 0..self.cats[i] as usize {
                let ma = &st.marg[self.starts[i] + a];
                for b in 0..cj {
                    let c = b0 as usize + a * cj + b;
                    let mb = &st.marg[self.starts[j] + b];
                    if let Some((z, zf)) = Self::z_with(self.opts.contrast, &st.cand[c], &st.pair[c], ma, mb, o) {
                        f(c, z, zf);
                    }
                }
            }
        }
    }

    /// Candidate c's clustered z.
    #[cfg(test)]
    fn z_of(&self, c: usize) -> Option<f64> {
        self.zs_of(c).map(|z| z.0)
    }

    /// Candidate c's clustered and floored z's.
    #[cfg(test)]
    fn zs_of(&self, c: usize) -> Option<(f64, f64)> {
        let mut out = None;
        self.each_z(self.stats.as_ref().unwrap(), |d, z, zf| {
            if d == c {
                out = Some((z, zf));
            }
        });
        out
    }

    /// A checkpoint: the window's z's (`select`), then the window restarts.
    fn checkpoint(&mut self, prior: f64) {
        let mut st = self.stats.take().expect("training state");
        let mut zs = Vec::new();
        self.each_z(&st, |c, z, zf| zs.push((c, z, zf, st.cand[c][5])));
        let window = (st.marg[0][1], st.games);
        st.cand.fill([0.0; W]);
        st.pair.fill([0.0; WP]);
        st.marg.fill([0.0; WM]);
        st.games = 0;
        self.stats = Some(st);
        self.select(prior, &zs, window);
    }

    /// A checkpoint's drops and additions given the window's z's `zs` (candidate, clustered z, floored z, games), and
    /// its examples and games for the log line. The bands are the clustered z's; the selection's z is the floored one
    /// (`floor`) or the clustered.
    fn select(&mut self, prior: f64, zs: &[(usize, f64, f64, f64)], (examples, games): (f64, u64)) {
        self.round.checkpoints += 1;
        let o = self.opts;
        let name = |[i, a, j, b]: [u8; 4]| format!("{i}:{a}&{j}:{b}");
        // Drops first: a pair with drop_after examples of its own since chosen whose mean is within drop sd of 0.
        let mut keep = Vec::with_capacity(self.list.len());
        let mut dropped = Vec::new();
        for slot in 0..self.list.len() {
            let [mu, s2] = self.weights[slot];
            if self.own[slot] >= o.drop_after && mu.abs() < o.drop * s2.sqrt() {
                self.round.dropped += 1;
                dropped.push(format!("{} mu {mu:.4} sd {:.4}", name(self.list[slot]), s2.sqrt()));
            } else {
                keep.push(slot);
            }
        }
        if keep.len() < self.list.len() {
            self.list = keep.iter().map(|&s| self.list[s]).collect();
            self.weights = keep.iter().map(|&s| self.weights[s]).collect();
            self.own = keep.iter().map(|&s| self.own[s]).collect();
        }
        // The window's z's (the chosen pairs' left out): those above the bar, and the mean z^2 by games.
        let mut chosen = vec![false; self.candidates];
        for &p in &self.list {
            chosen[self.candidate(p)] = true;
        }
        let mut top: Vec<(f64, usize)> = Vec::new();
        let mut bands = [(0.0f64, 0u64); 3];
        let (mut max, mut over) = (0.0f64, 0u64);
        for &(c, z, zf, g) in zs {
            if chosen[c] {
                continue;
            }
            let band = &mut bands[if g < 100.0 { 0 } else if g < 1000.0 { 1 } else { 2 }];
            band.0 += z * z;
            band.1 += 1;
            max = max.max(z.abs());
            over += (z.abs() > o.z) as u64;
            let zs = if o.floor { zf } else { z };
            if zs.abs() > o.z {
                top.push((zs, c));
            }
        }
        let band = |(s, n): (f64, u64)| if n > 0 { format!("{:.3} ({n})", s / n as f64) } else { "- (0)".into() };
        let mut line = format!(
            "pairs checkpoint at {} examples: window {examples} examples, {games} games, mean z^2 by games under 100 {}, \
             100-999 {}, 1000+ {}, max |z| {max:.2}, over the bar {over}{};",
            self.trained,
            band(bands[0]),
            band(bands[1]),
            band(bands[2]),
            if o.floor { format!(" (floored: {})", top.len()) } else { String::new() }
        );
        write!(line, " dropped {}{}{};", dropped.len(), if dropped.is_empty() { "" } else { ": " }, dropped.join(", "))
            .unwrap();
        // Then adds: the candidates above the bar, by |z| (ties by index), up to k and the cap.
        let want = o.k.min(o.cap.saturating_sub(self.list.len()));
        if want > 0 {
            top.sort_by(|a, b| b.0.abs().total_cmp(&a.0.abs()).then(a.1.cmp(&b.1)));
            top.truncate(want);
            let s2 = (o.shrink * prior) * (o.shrink * prior);
            let mut added = Vec::with_capacity(top.len());
            for &(z, c) in &top {
                let p = self.decode(c);
                self.list.push(p);
                self.weights.push([0.0, s2]);
                self.own.push(0);
                self.round.added.push(z.abs());
                added.push(format!("{} {z:+.2}", name(p)));
            }
            write!(line, " added {}{}{}", added.len(), if added.is_empty() { "" } else { " (z): " }, added.join(", "))
                .unwrap();
        } else {
            write!(line, " added 0 (at the cap)").unwrap();
        }
        write!(line, "; {} chosen", self.list.len()).unwrap();
        self.round.lines.push(line);
        self.rebuild_index();
    }

    /// The round's lines for the train log (then the round's counts restart): first pairs added (their |z|) and
    /// dropped, the chosen pairs, and the mean number of chosen pairs active per position; then one line per
    /// checkpoint, naming the pairs dropped and added (feature i value a & feature j value b as i:a&j:b, by the
    /// encoding's feature order) with their z.
    pub fn report(&mut self) -> String {
        let r = std::mem::take(&mut self.round);
        let mut s = format!(
            "pairs: {} chosen of cap {}; this round: {} examples, {} checkpoint{}, added {}",
            self.list.len(),
            self.opts.cap,
            r.examples,
            r.checkpoints,
            if r.checkpoints == 1 { "" } else { "s" },
            r.added.len()
        );
        if !r.added.is_empty() {
            let mut z = r.added.clone();
            z.sort_by(f64::total_cmp);
            let q = |p: f64| z[((z.len() - 1) as f64 * p).round() as usize];
            write!(s, " (|z| min {:.2}, median {:.2}, max {:.2})", z[0], q(0.5), z[z.len() - 1]).unwrap();
        }
        write!(
            s,
            ", dropped {}; active per position {:.3}; {} examples since the start",
            r.dropped,
            if r.examples > 0 { r.active as f64 / r.examples as f64 } else { 0.0 },
            self.trained
        )
        .unwrap();
        for line in &r.lines {
            write!(s, "\n{line}").unwrap();
        }
        s
    }

    // ------------------------------------------------------------ files

    /// The model file's pairs section (after the probit's weights).
    pub fn write_text(&self, s: &mut String) {
        let o = &self.opts;
        writeln!(s, "pairs\n{} {:e} {} {:e} {:e} {} {} {}", o.k, o.z, o.cap, o.shrink, o.drop, o.every, o.drop_after, o.sample)
            .unwrap();
        writeln!(s, "{}\n{}", self.trained, self.list.len()).unwrap();
        for (&[i, a, j, b], &[mu, s2]) in self.list.iter().zip(&self.weights) {
            writeln!(s, "{i} {a} {j} {b} {mu:e} {s2:e}").unwrap();
        }
    }

    /// Reads `write_text`'s section; the model plays (no training state until `load_state`).
    pub fn read_text(tokens: &mut Tokens, categories: &[u8]) -> io::Result<Self> {
        if tokens.next::<String>()? != "pairs" {
            return Err(invalid("probit model version 2 without its pairs section"));
        }
        let opts = PairOptions {
            k: tokens.next()?,
            z: tokens.next()?,
            cap: tokens.next()?,
            shrink: tokens.next()?,
            drop: tokens.next()?,
            every: tokens.next()?,
            drop_after: tokens.next()?,
            sample: tokens.next()?,
            // Not in the model file: the state's (`load_state`), or the trainer's (`fresh_state`).
            contrast: Contrast::Marginal,
            floor: true,
        };
        opts.check().map_err(invalid)?;
        let mut p = Self::shape(categories, opts);
        p.trained = tokens.next()?;
        let n: usize = tokens.next()?;
        for _ in 0..n {
            let q: [u8; 4] = [tokens.next()?, tokens.next()?, tokens.next()?, tokens.next()?];
            let [i, a, j, b] = q;
            let f = categories.len();
            if !((i as usize) < f && (j as usize) < f && i < j && a < categories[i as usize] && b < categories[j as usize]) {
                return Err(invalid(format!("bad pair {q:?} in probit model file")));
            }
            p.list.push(q);
            p.weights.push([tokens.next()?, tokens.next()?]);
        }
        p.rebuild_index();
        Ok(p)
    }

    /// Writes the training state to `path` (the contrast, the pairs' own examples and the window's statistics):
    /// after a round's end, never inside one.
    pub fn save_state(&self, path: &Path) -> io::Result<()> {
        let st = self.stats.as_ref().ok_or_else(|| invalid("pairs: no training state to save"))?;
        if !self.pending.is_empty() {
            return Err(invalid("pairs: a round's examples are not yet summed by game (no end of round)"));
        }
        let mut w = Writer::new("probit-pairs3");
        w.u64(self.trained);
        w.u8s(&self.list.iter().flatten().copied().collect::<Vec<u8>>());
        w.str(contrast_name(self.opts.contrast));
        w.str(se_name(self.opts.floor));
        w.usizes(&self.own.iter().map(|&n| n as usize).collect::<Vec<usize>>());
        w.u64(st.games);
        w.f64s(&st.cand.iter().flatten().copied().collect::<Vec<f64>>());
        w.f64s(&st.pair.iter().flatten().copied().collect::<Vec<f64>>());
        w.f64s(&st.marg.iter().flatten().copied().collect::<Vec<f64>>());
        w.save(path)
    }

    /// Reads `save_state`'s file for the model it was saved with (the same examples trained and the same pairs); the
    /// contrast is the state's.
    pub fn load_state(&mut self, path: &Path) -> io::Result<()> {
        let data = std::fs::read(path)?;
        let mut r = Reader::new(&data, "probit-pairs3")?;
        let trained = r.u64()?;
        if trained != self.trained {
            return Err(invalid(format!(
                "{path:?}: a state after {trained} examples, but the model has trained {}",
                self.trained
            )));
        }
        let list = r.u8s()?;
        if list != self.list.iter().flatten().copied().collect::<Vec<u8>>() {
            return Err(invalid(format!("{path:?}: the state's pairs differ from the model's")));
        }
        let contrast = r.str()?;
        let contrast = parse_contrast(&contrast).ok_or_else(|| invalid(format!("{path:?}: contrast {contrast:?}")))?;
        let se = r.str()?;
        let floor = parse_se(&se).ok_or_else(|| invalid(format!("{path:?}: se {se:?}")))?;
        let own = r.usizes()?;
        let games = r.u64()?;
        let cand = r.f64s()?;
        let pair = r.f64s()?;
        let marg = r.f64s()?;
        if own.len() != self.list.len()
            || cand.len() != W * self.candidates
            || pair.len() != WP * self.candidates
            || marg.len() != WM * self.index.len()
        {
            return Err(invalid(format!("{path:?}: a state of the wrong shape")));
        }
        r.finish()?;
        self.opts.contrast = contrast;
        self.opts.floor = floor;
        self.own = own.into_iter().map(|n| n as u64).collect();
        self.stats = Some(Box::new(Stats {
            cand: cand.chunks_exact(W).map(|x| x.try_into().unwrap()).collect(),
            pair: pair.chunks_exact(WP).map(|x| x.try_into().unwrap()).collect(),
            marg: marg.chunks_exact(WM).map(|x| x.try_into().unwrap()).collect(),
            games,
        }));
        Ok(())
    }

    /// Fresh statistics for a model that has trained nothing yet (a driver's untrained version 0 has no state), with
    /// this contrast and SE (the model file keeps neither).
    pub fn fresh_state(&mut self, contrast: Contrast, floor: bool) -> io::Result<()> {
        if self.trained != 0 || !self.list.is_empty() {
            return Err(invalid("pairs: a trained model needs its state file to train on"));
        }
        self.opts.contrast = contrast;
        self.opts.floor = floor;
        self.own.clear();
        self.stats = Some(self.zero_stats());
        Ok(())
    }

    pub fn has_state(&self) -> bool {
        self.stats.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding::{CATEGORIES, FEATURES};

    fn h(mut x: u64) -> u64 {
        x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
        x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        x ^ (x >> 31)
    }

    /// `n` positions, values 0 or 1 at random, in games of `per_game`.
    fn round(seed: u64, n: usize, per_game: usize) -> Vec<(Encoded, [f64; 3], u64)> {
        (0..n)
            .map(|t| {
                let mut x = [0u8; FEATURES];
                for (i, xi) in x.iter_mut().enumerate() {
                    *xi = (h(seed << 32 ^ (t * FEATURES + i) as u64) % 2) as u8;
                }
                (x, [1.0, 0.0, 0.0], (t / per_game) as u64)
            })
            .collect()
    }

    #[test]
    fn candidates_cover_every_pair_value_once() {
        let p = Pairs::new(&CATEGORIES, PairOptions::default());
        assert_eq!(p.candidates(), 356_642);
        // Every (i, a, j, b) maps to a distinct index that decodes back.
        let mut seen = vec![false; p.candidates()];
        let f = CATEGORIES.len();
        for i in 0..f {
            for j in i + 1..f {
                for a in 0..CATEGORIES[i] {
                    for b in 0..CATEGORIES[j] {
                        let q = [i as u8, a, j as u8, b];
                        let c = p.candidate(q);
                        assert!(!seen[c]);
                        seen[c] = true;
                        assert_eq!(p.decode(c), q);
                    }
                }
            }
        }
        assert!(seen.iter().all(|&s| s));
    }

    #[test]
    fn text_and_state_round_trip() {
        let mut p = Pairs::new(&CATEGORIES, PairOptions { every: 20, z: 0.5, k: 4, ..Default::default() });
        let data = round(1, 60, 5);
        for (t, (x, _, _)) in data.iter().enumerate() {
            p.observe(x, (h(t as u64) % 1000) as f64 / 500.0 - 1.0);
        }
        // No state inside a round.
        assert!(p.save_state(&std::env::temp_dir().join("never.state")).is_err());
        p.end_round(&data, 0.3);
        assert_eq!(p.report().lines().filter(|l| l.starts_with("pairs checkpoint")).count(), 3);
        assert!(!p.is_empty());
        let mut s = String::new();
        p.write_text(&mut s);
        let mut q = Pairs::read_text(&mut Tokens::new(&s), &CATEGORIES).unwrap();
        assert_eq!(q.list, p.list);
        assert_eq!(q.weights, p.weights);
        assert_eq!(q.trained, 60);
        assert!(q.fresh_state(Contrast::Marginal, true).is_err());
        let dir = std::env::temp_dir().join(format!("pairs-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("probit1.state");
        // pairs3: the contrast and the pairs' own examples are in the state, not the model file.
        p.opts.contrast = Contrast::Cells;
        p.opts.floor = false;
        p.own = (0..p.list.len() as u64).map(|n| 7 * n + 1).collect();
        p.save_state(&path).unwrap();
        assert_eq!((q.opts.contrast, q.opts.floor), (Contrast::Marginal, true));
        q.load_state(&path).unwrap();
        assert_eq!(q.stats, p.stats);
        assert_eq!((q.opts, &q.own), (p.opts, &p.own));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// Per game (in order of their ids) and per example: u = (G_ab, m_ab, G_a, m_a, G_b, m_b, G, m) of candidate `q`,
    /// straight from the examples.
    fn sums(data: &[(Encoded, [f64; 3], u64)], v: &[f64], [i, a, j, b]: [u8; 4]) -> (Vec<[f64; 8]>, Vec<[f64; 8]>) {
        let one = |t: usize| {
            let x = &data[t].0;
            let (ina, inb) = (x[i as usize] == a, x[j as usize] == b);
            let mut u = [0.0; 8];
            for (k, on) in [(0, ina && inb), (2, ina), (4, inb), (6, true)] {
                if on {
                    u[k] += v[t];
                    u[k + 1] += 1.0;
                }
            }
            u
        };
        let mut games: Vec<u64> = data.iter().map(|d| d.2).collect();
        games.sort_unstable();
        games.dedup();
        let by_game = games
            .iter()
            .map(|&g| {
                let mut u = [0.0; 8];
                for t in (0..data.len()).filter(|&t| data[t].2 == g) {
                    let e = one(t);
                    (0..8).for_each(|k| u[k] += e[k]);
                }
                u
            })
            .collect();
        (by_game, (0..data.len()).map(one).collect())
    }

    /// The contrast and its clustered and floored z's straight from the sums, by the contrast's own definition: each
    /// game's (and each example's) influence from the cells (cells) or the marginal means (marginal).
    fn brute_z(contrast: Contrast, (games, examples): &(Vec<[f64; 8]>, Vec<[f64; 8]>)) -> (f64, f64, f64) {
        let tot: Vec<f64> = (0..8).map(|k| games.iter().map(|g| g[k]).sum()).collect();
        let cell = |g: &[f64]| {
            [(g[2] - g[0], g[3] - g[1]), (g[4] - g[0], g[5] - g[1]), (g[6] - g[2] - g[4] + g[0], g[7] - g[3] - g[5] + g[1])]
        };
        let infl: Box<dyn Fn(&[f64; 8]) -> f64> = match contrast {
            Contrast::Marginal => {
                let (ya, yb, y) = (tot[2] / tot[3], tot[4] / tot[5], tot[6] / tot[7]);
                let mu0 = ya + yb - y;
                let t = tot.clone();
                Box::new(move |g: &[f64; 8]| {
                    (g[0] - g[1] * mu0) / t[1] - (g[2] - g[3] * ya) / t[3] - (g[4] - g[5] * yb) / t[5] + (g[6] - g[7] * y) / t[7]
                })
            }
            Contrast::Cells => {
                let c = cell(&tot);
                let y: Vec<f64> = c.iter().map(|&(s, n)| s / n).collect();
                let nu0 = y[0] + y[1] - y[2];
                let n1 = tot[1];
                Box::new(move |g: &[f64; 8]| {
                    let d = cell(g);
                    (g[0] - g[1] * nu0) / n1 - (d[0].0 - d[0].1 * y[0]) / c[0].1 - (d[1].0 - d[1].1 * y[1]) / c[1].1
                        + (d[2].0 - d[2].1 * y[2]) / c[2].1
                })
            }
        };
        let num: f64 = games.iter().map(|g| infl(g)).sum();
        let vg: f64 = games.iter().map(|g| infl(g).powi(2)).sum();
        let ve: f64 = examples.iter().map(|g| infl(g).powi(2)).sum();
        (num, num / vg.sqrt(), num / vg.max(ve).sqrt())
    }

    #[test]
    fn statistics_are_sums_by_game_whatever_the_training_order() {
        // Games 0 and 2 have feature 0 at 0, game 1 at 1; every other feature 0. Candidate (0:0 & 1:0) is active in
        // games 0 (v 0.5, 0.25) and 2 (v 0.125, -0.5, 1): G 0.75 and 0.625, m 2 and 3 (dyadic: sums exact in any order).
        let mut data = Vec::new();
        let v = [0.5, 0.25, -1.0, 0.75, 0.125, -0.5, 1.0];
        let game = [0u64, 0, 1, 1, 2, 2, 2];
        for t in 0..7 {
            let mut x = [0u8; FEATURES];
            x[0] = (game[t] == 1) as u8;
            data.push((x, [1.0, 0.0, 0.0], game[t]));
        }
        let o = PairOptions { every: 1 << 40, ..Default::default() };
        let mut a = Pairs::new(&CATEGORIES, o);
        for t in 0..7 {
            a.observe(&data[t].0, v[t]);
        }
        a.end_round(&data, 0.3);
        let st = a.stats.as_ref().unwrap();
        let c = a.candidate([0, 0, 1, 0]);
        // G, m of the pair, of 0:0 (the same here), of 1:0 (every example) and of the game, per game: (0.75, 2) in
        // game 0 (whole game 0.75, 2), (0.625, 3) in game 2 (0.625, 3); game 1 (-0.25, 2) has 1:0 but not 0:0.
        assert_eq!(st.cand[c][..6], [1.375, 5.0, 0.953125, 3.375, 13.0, 2.0]);
        assert_eq!(st.cand[c][6..18], [0.953125, 3.375, 3.375, 13.0, 0.953125, 3.375, 3.375, 13.0, 0.953125, 3.375, 3.375, 13.0]);
        assert_eq!(st.cand[c][18], 1.578125); // v^2: 0.25 + 0.0625 + 0.015625 + 0.25 + 1
        assert_eq!(st.games, 3);
        // Every example: S, n, G^2, G m, m^2, then with the game's own (the same), then the sum of v^2.
        assert_eq!(st.marg[0], [1.125, 7.0, 1.015625, 2.875, 17.0, 1.015625, 2.875, 2.875, 17.0, 3.140625]);
        assert_eq!(st.marg[a.starts[0]], [1.375, 5.0, 0.953125, 3.375, 13.0, 0.953125, 3.375, 3.375, 13.0, 1.578125]);
        assert_eq!(st.marg[a.starts[0] + 1], [-0.25, 2.0, 0.0625, -0.5, 4.0, 0.0625, -0.5, -0.5, 4.0, 1.5625]);
        // 0:0 and 1:0 over all games (the games without the pair count too: none here); 0:1 and 1:0 in game 1.
        assert_eq!(st.pair[c], [0.953125, 3.375, 3.375, 13.0]);
        assert_eq!(st.pair[a.candidate([0, 1, 1, 0])], [0.0625, -0.5, -0.5, 4.0]);
        // The same examples in another order: the same statistics.
        let perm = [5, 2, 0, 6, 3, 1, 4];
        let mut b = Pairs::new(&CATEGORIES, o);
        let shuffled: Vec<_> = perm.iter().map(|&t| data[t]).collect();
        for &t in &perm {
            b.observe(&data[t].0, v[t]);
        }
        b.end_round(&shuffled, 0.3);
        assert_eq!(a.stats, b.stats);
    }

    /// `n` positions in games of `per_game`, each feature's value drawn with P(1) = `p1` (0 otherwise) except
    /// feature 5, a copy of feature 3 nine times in ten; v = `own` x feature 3 + a game effect + noise.
    fn correlated(seed: u64, n: usize, per_game: usize, p1: f64, own: f64) -> (Vec<(Encoded, [f64; 3], u64)>, Vec<f64>) {
        let u = |k: u64| (h(seed << 40 ^ k) % 1_000_000) as f64 / 1_000_000.0;
        let mut data = Vec::new();
        let mut v = Vec::new();
        for t in 0..n {
            let mut x = [0u8; FEATURES];
            for (i, xi) in x.iter_mut().enumerate() {
                *xi = (u((t * FEATURES + i) as u64) < p1) as u8;
            }
            if u(1 << 30 | t as u64) < 0.9 {
                x[5] = x[3];
            }
            let game = (t / per_game) as u64;
            v.push(own * x[3] as f64 + 0.5 * (u(1 << 31 | game) - 0.5) + (u(1 << 32 | t as u64) - 0.5));
            data.push((x, [1.0, 0.0, 0.0], game));
        }
        (data, v)
    }

    #[test]
    fn z_is_the_contrast_over_its_influence_s_sum_of_squares_by_game() {
        // Both contrasts, against their definitions computed straight from the examples (`brute_z`), for pairs of
        // rare and common values and the correlated pair (3:1 & 5:1).
        let (data, v) = correlated(3, 3000, 12, 0.3, 0.2);
        for contrast in [Contrast::Marginal, Contrast::Cells] {
            let mut p = Pairs::new(&CATEGORIES, PairOptions { every: 1 << 40, contrast, ..Default::default() });
            for (t, (x, _, _)) in data.iter().enumerate() {
                p.observe(x, v[t]);
            }
            p.end_round(&data, 0.3);
            for q in [[3, 1, 5, 1], [3, 1, 5, 0], [0, 0, 1, 0], [0, 1, 1, 1], [2, 1, 69, 0], [10, 0, 40, 1]] {
                let (num, want, wantf) = brute_z(contrast, &sums(&data, &v, q));
                let (got, gotf) = p.zs_of(p.candidate(q)).unwrap();
                assert!((got - want).abs() < 1e-9 * want.abs().max(1.0), "{contrast:?} {q:?}: {got} vs {want} (contrast {num})");
                assert!((gotf - wantf).abs() < 1e-9 * wantf.abs().max(1.0), "{contrast:?} {q:?} floored: {gotf} vs {wantf}");
                assert!(gotf.abs() <= got.abs());
            }
        }
    }

    #[test]
    fn cells_cancel_a_correlated_value_s_own_effect_where_the_marginal_contrast_does_not() {
        // v moves with feature 3 alone (no interaction), and feature 5 copies feature 3 nine times in ten: the
        // marginal contrast of (3:1 & 5:1) is beta (P(3:1) - P(3:1 | 5:1)) far from 0, the cells' 0.
        let (data, v) = correlated(5, 20_000, 10, 0.5, 0.5);
        let z = |contrast| {
            let mut p = Pairs::new(&CATEGORIES, PairOptions { every: 1 << 40, contrast, ..Default::default() });
            for (t, (x, _, _)) in data.iter().enumerate() {
                p.observe(x, v[t]);
            }
            p.end_round(&data, 0.3);
            p.z_of(p.candidate([3, 1, 5, 1])).unwrap()
        };
        let (marginal, cells) = (z(Contrast::Marginal), z(Contrast::Cells));
        assert!(marginal.abs() > 8.0, "marginal {marginal}");
        assert!(cells.abs() < 3.0, "cells {cells}");
    }

    #[test]
    fn a_tiny_within_game_effect_is_significant_but_not_selected_with_the_floor() {
        // 2,000 games of 20 positions: side to move (64) alternates, feature 12 is 1 in each game's first half, and v
        // is the game's own +-1 (its result) plus 0.02 when 12:1 and 64:1 both hold, plus noise 0.05. The cells of
        // (12:1 & 64:1) are compared within games, so the game's +-1 cancels from the clustered SE and the 0.02 is
        // ~20 SEs; per example it is ~1 SE: the floored z leaves it below the bar.
        let u = |k: u64| (h(9 << 40 ^ k) % 1_000_000) as f64 / 1_000_000.0;
        let (mut data, mut v) = (Vec::new(), Vec::new());
        for t in 0..40_000usize {
            let (g, k) = (t / 20, t % 20);
            let mut x = [0u8; FEATURES];
            x[64] = (k % 2) as u8;
            x[12] = (k < 10) as u8;
            x[30] = (u(1 << 33 | t as u64) < 0.5) as u8;
            let y = if u(1 << 34 | g as u64) < 0.5 { 1.0 } else { -1.0 };
            v.push(y + 0.02 * (x[12] == 1 && x[64] == 1) as u8 as f64 + 0.05 * (u(1 << 35 | t as u64) - 0.5));
            data.push((x, [1.0, 0.0, 0.0], g as u64));
        }
        for contrast in [Contrast::Marginal, Contrast::Cells] {
            let mut p = Pairs::new(&CATEGORIES, PairOptions { every: 1 << 40, contrast, ..Default::default() });
            for (t, (x, _, _)) in data.iter().enumerate() {
                p.observe(x, v[t]);
            }
            p.end_round(&data, 0.3);
            let (z, zf) = p.zs_of(p.candidate([12, 1, 64, 1])).unwrap();
            assert!(z.abs() > 8.0 && zf.abs() < 3.0, "{contrast:?}: clustered {z}, floored {zf}");
        }
    }

    #[test]
    fn a_pair_that_cannot_differ_from_its_value_has_no_z() {
        // Feature 1 is always 0: (0:1 & 1:0) is 0:1 itself. The marginal contrast is 0 up to rounding (no z); the
        // cells have an empty cell (0:1 without 1:0).
        let data = round(9, 400, 8);
        let v: Vec<f64> = (0..400).map(|t| (h(77 << 20 | t) % 1000) as f64 / 500.0 - 1.0).collect();
        let data: Vec<_> = data.into_iter().map(|(mut x, l, g)| { x[1] = 0; (x, l, g) }).collect();
        for contrast in [Contrast::Marginal, Contrast::Cells] {
            let mut p = Pairs::new(&CATEGORIES, PairOptions { every: 1 << 40, contrast, ..Default::default() });
            for (t, (x, _, _)) in data.iter().enumerate() {
                p.observe(x, v[t]);
            }
            p.end_round(&data, 0.3);
            assert_eq!(p.z_of(p.candidate([0, 1, 1, 0])), None, "{contrast:?}");
            assert!(p.z_of(p.candidate([0, 1, 2, 0])).is_some(), "{contrast:?}");
        }
    }

    #[test]
    fn windows_restart_at_each_checkpoint_and_own_examples_count_on() {
        // every 100: the round of 250 crosses two checkpoints; the first chooses from the round, then the window is
        // empty (the second chooses nothing), and the next round's examples alone fill the next window.
        let o = PairOptions { every: 100, z: 1.0, k: 3, ..Default::default() };
        let mut p = Pairs::new(&CATEGORIES, o);
        let data = round(11, 250, 5);
        for (t, (x, _, _)) in data.iter().enumerate() {
            p.observe(x, (h(5 << 20 | t as u64) % 1000) as f64 / 500.0 - 1.0);
        }
        p.end_round(&data, 0.3);
        let r = p.report();
        let lines: Vec<&str> = r.lines().filter(|l| l.starts_with("pairs checkpoint")).collect();
        assert_eq!(lines.len(), 2, "{r}");
        assert!(lines[0].contains("window 250 examples, 50 games") && lines[0].contains("added 3 (z)"), "{r}");
        assert!(lines[1].contains("window 0 examples, 0 games") && lines[1].contains("added 0;"), "{r}");
        let st = p.stats.as_ref().unwrap();
        assert!(st.cand.iter().all(|c| *c == [0.0; W]) && st.marg.iter().all(|m| *m == [0.0; WM]) && st.games == 0);
        // The chosen pairs' own examples: those of the next rounds, across windows.
        let next = round(12, 120, 6);
        for (t, (x, _, _)) in next.iter().enumerate() {
            p.observe(x, (h(6 << 20 | t as u64) % 1000) as f64 / 500.0 - 1.0);
        }
        let want: Vec<u64> = p
            .list
            .iter()
            .map(|&[i, a, j, b]| next.iter().filter(|(x, _, _)| x[i as usize] == a && x[j as usize] == b).count() as u64)
            .collect();
        assert_eq!(p.own, want);
        p.end_round(&next, 0.3);
        assert!(p.report().contains("window 120 examples, 20 games"));
    }

    #[test]
    fn a_value_s_own_effect_does_not_make_its_pairs_interact() {
        // v = 0.5 when feature 3 is 1, less 0.25, plus noise: a main effect, no interaction. pairs1's S / sqrt(n) puts
        // the pairs with 3:1 far above the bar; the contrast, clustered by game, keeps them near 0.
        let data = round(7, 4000, 10);
        let mut p = Pairs::new(&CATEGORIES, PairOptions { every: 1 << 40, ..Default::default() });
        for (t, (x, _, _)) in data.iter().enumerate() {
            let noise = (h(1 << 40 | t as u64) % 10_000) as f64 / 10_000.0 - 0.5;
            p.observe(x, 0.5 * x[3] as f64 - 0.25 + noise);
        }
        p.end_round(&data, 0.3);
        let st = p.stats.as_ref().unwrap();
        let (mut raw_over, mut n, mut worst) = (0, 0, 0.0f64);
        for j in 0..FEATURES {
            if j == 3 {
                continue;
            }
            for b in 0..2u8 {
                let q = if j < 3 { [j as u8, b, 3, 1] } else { [3, 1, j as u8, b] };
                let c = p.candidate(q);
                let s = st.cand[c];
                raw_over += ((s[0] / s[1].sqrt()).abs() > 5.0) as usize;
                worst = worst.max(p.z_of(c).unwrap().abs());
                n += 1;
            }
        }
        assert!(raw_over * 2 > n, "pairs1's statistic: {raw_over} of {n} over 5");
        assert!(worst < 4.5, "the contrast: max |z| {worst} over {n}");
    }

    #[test]
    fn checkpoints_drop_then_add_by_z_up_to_k_and_the_cap() {
        let o = PairOptions { k: 3, z: 5.0, cap: 4, shrink: 0.5, drop: 0.5, every: 10, drop_after: 100, ..Default::default() };
        let mut p = Pairs::new(&CATEGORIES, o);
        // z 6, -7, 7 (a tie with -7: by index), 5 (not above the bar), 9; games 10, 500, 5000, 10, 10.
        // (The floored z is the selection's: here 1 below the clustered, which the bands report.)
        let zs = [(10, 7.0, 6.0, 10.0), (20, -8.0, -7.0, 500.0), (30, 8.0, 7.0, 5000.0), (40, 6.0, 5.0, 10.0), (50, 10.0, 9.0, 10.0)];
        p.select(0.3, &zs, (1000.0, 70));
        let chosen: Vec<usize> = p.list.iter().map(|&q| p.candidate(q)).collect();
        assert_eq!(chosen, [50, 20, 30]);
        assert!(p.weights.iter().all(|&w| w == [0.0, 0.15 * 0.15]));
        assert_eq!(p.own, [0, 0, 0]);
        // The next checkpoint adds one: the cap is 4 (the chosen are left out of the window's z's).
        p.select(0.3, &zs, (1000.0, 70));
        let chosen: Vec<usize> = p.list.iter().map(|&q| p.candidate(q)).collect();
        assert_eq!(chosen, [50, 20, 30, 10]);
        // A pair is dropped once it has drop_after examples of its own and |mu| < drop sd; the others stay.
        p.weights[1][0] = 0.01; // candidate 20: |mu| 0.01 < 0.5 x 0.15
        p.weights[2][0] = 0.2; // candidate 30: kept, |mu| above
        p.own = vec![99, 100, 100, 0]; // candidate 50: too few examples to judge
        p.select(0.3, &[(60, 8.0, 8.0, 10.0), (20, 9.0, 9.0, 10.0)], (1000.0, 70));
        let chosen: Vec<usize> = p.list.iter().map(|&q| p.candidate(q)).collect();
        assert_eq!(chosen, [50, 30, 10, 20]);
        assert_eq!(p.own, [99, 100, 0, 0]);
        let r = p.report();
        assert!(r.contains("added 5 (|z| min 6.00, median 7.00, max 9.00), dropped 1"), "{r}");
        assert!(r.starts_with("pairs: 4 chosen of cap 4; this round: 0 examples, 3 checkpoints"), "{r}");
        let lines: Vec<&str> = r.lines().collect();
        assert_eq!(lines.len(), 4, "{r}");
        assert!(
            lines[1].starts_with(
                "pairs checkpoint at 0 examples: window 1000 examples, 70 games, mean z^2 by games under 100 61.667 (3), \
                 100-999 64.000 (1), 1000+ 64.000 (1), max |z| 10.00, over the bar 5 (floored: 4); dropped 0; added 3 (z): "
            ),
            "{r}"
        );
        assert!(lines[1].contains(" -7.00"), "{r}");
        assert!(lines[2].contains("under 100 42.500 (2), 100-999 - (0), 1000+ - (0)"), "{r}");
        assert!(lines[3].contains(" dropped 1: ") && lines[3].ends_with("; 4 chosen"), "{r}");
    }

    #[test]
    fn active_pairs_follow_the_index() {
        let mut p = Pairs::new(&CATEGORIES, PairOptions::default());
        p.list = vec![[0, 1, 5, 2], [0, 1, 6, 3], [4, 0, 69, 1]];
        p.weights = vec![[1.0, 0.5], [10.0, 0.25], [100.0, 0.125]];
        p.rebuild_index();
        let mut data = vec![0u8; 70];
        data[0] = 1;
        data[5] = 2;
        data[6] = 4;
        data[69] = 1;
        let (mut m, mut v) = (0.0, 0.0);
        p.add_active(&data, &mut m, &mut v);
        assert_eq!((m, v), (101.0, 0.625));
        assert_eq!(p.count_active(&data), 2);
        let mut seen = Vec::new();
        p.for_active(&data, |w| seen.push(w[0]));
        assert_eq!(seen, [1.0, 100.0]);
    }
}
