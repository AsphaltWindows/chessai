//! Exact fixed-point play for compound models (hybrid5): delegation trees and DAGs (`Bdt`, `Bdd`) under one
//! calibration of the whole model (`cal:` specs), searched 1 or 2 plies deep with log-odds values.
//!
//! Every node of the graph is held in fixed point as in `fixed`: its classes' sums are exact integers, kept
//! incrementally from a position to the next. A router picks its child from its sums only when the top class leads
//! every other by more than a proven bound on the distance between the fixed-point sums and the float arithmetic of
//! the router's `predict` (the float sum's or product's rounding, the weights' quantization, the logs' own rounding);
//! otherwise the float router decides (a fallback). Either way the leaf is the float model's leaf.
//!
//! Values that reach the move policy are the float model's own. At depth 1 each move's value is its position's leaf
//! value in float (`exact`, the arithmetic of `Calibrated::log_odds_direct` without the routing). At depth 2 a move's
//! value is the smallest of its replies' values (`search`'s negamax with a full window has no cutoffs). Each reply
//! gets a value from the fixed-point sums with a bound on its distance from the float value (`estimate`). Only
//! replies whose lower bound is at most the smallest upper bound can be the smallest, so only those are valued in
//! float, and the float loop runs over them in move order: the same comparisons, so the same value, ties and signed
//! zeros included. If none of them is at most that bound in float (a float NaN), every reply is valued in float.
//!
//! CHESSAI_FX_CHECK=1 checks every incremental step, every bound and every move value against the float model.
//! CHESSAI_FX_STATS=1 counts the fallbacks (`report`, printed to stderr at exit by the binary).

use std::borrow::Cow;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

use shakmaty::Chess;

use crate::calibration::Calibration;
use crate::delegation::{Graph, Kind, NodeModel};
use crate::encoding::{self, CATEGORIES, Encoded, FEATURES};
use crate::fixed::{self, Head, SHIFT, Sums};

const SCALE: f64 = (1u64 << SHIFT) as f64;
/// The unit roundoff of f64, 2^-53.
const U: f64 = f64::EPSILON / 2.0;
/// The quantization error of a sum of 71 terms (a bias and 70 features), each rounded to a multiple of 2^-32.
const QUANT: f64 = 71.0 / (2.0 * SCALE);
/// Below this log, a product router's float product could leave the normal range: fall back.
const LOW_LOG: f64 = -690.0;
/// Most nodes a compound model may have.
pub const MAX_NODES: usize = 64;

/// gamma_n = n u / (1 - n u): the relative error bound of n float operations in sequence.
fn gamma(n: usize) -> f64 {
    let nu = n as f64 * U;
    nu / (1.0 - nu)
}

#[inline]
fn add(s: &mut Sums, a: &Sums, b: &Sums) {
    for c in 0..4 {
        s[c] += a[c] - b[c];
    }
}

/// A node's weights in fixed point.
#[derive(Clone, Debug)]
enum Table {
    /// Row 0 the bias, row starts[f] + v feature f's value v (as `FxLinear`): logistic regression, naive Bayes's logs,
    /// a probit's means and variances.
    Linear { rows: Vec<Sums>, starts: [u16; FEATURES] },
    /// TAN's logs: feature f's entry for its value a, given its parent's value b, is row off + b * step + a (step 0:
    /// no parent). `kids[f]`: the features whose parent is f, as a bit set.
    Tan { bias: Sums, rows: Vec<Sums>, off: [u32; FEATURES], step: [u8; FEATURES], par: [u8; FEATURES], kids: [u128; FEATURES] },
}

impl Table {
    #[inline]
    fn tan_row(off: &[u32; FEATURES], step: &[u8; FEATURES], par: &[u8; FEATURES], enc: &Encoded, f: usize) -> usize {
        off[f] as usize + enc[par[f] as usize] as usize * step[f] as usize + enc[f] as usize
    }

    /// The sums of a position, in full.
    fn full(&self, enc: &Encoded) -> Sums {
        match self {
            Table::Linear { rows, starts } => {
                let mut s = rows[0];
                for f in 0..FEATURES {
                    add(&mut s, &rows[starts[f] as usize + enc[f] as usize], &[0; 4]);
                }
                s
            }
            Table::Tan { bias, rows, off, step, par, .. } => {
                let mut s = *bias;
                for f in 0..FEATURES {
                    add(&mut s, &rows[Self::tan_row(off, step, par, enc, f)], &[0; 4]);
                }
                s
            }
        }
    }

    /// The sums of the position `ch` leads to, from `sums`, those of the position before it, whose encoding is `old`.
    #[inline]
    fn moved(&self, sums: &Sums, old: &Encoded, ch: &Changes) -> Sums {
        let mut s = *sums;
        match self {
            Table::Linear { rows, starts } => {
                for &(f, v) in ch.list() {
                    let start = starts[f as usize] as usize;
                    add(&mut s, &rows[start + v as usize], &rows[start + old[f as usize] as usize]);
                }
            }
            Table::Tan { rows, off, step, par, kids, .. } => {
                // The changed features' entries and their children's, each once.
                let mut set = ch.mask;
                for &(f, _) in ch.list() {
                    set |= kids[f as usize];
                }
                while set != 0 {
                    let f = set.trailing_zeros() as usize;
                    set &= set - 1;
                    add(&mut s, &rows[Self::tan_row(off, step, par, &ch.enc, f)], &rows[Self::tan_row(off, step, par, old, f)]);
                }
            }
        }
        s
    }

    /// Per class, the sum over the features of the largest |entry| (and the bias's), in the weights' own units.
    fn abs_sums(&self, classes: usize) -> [f64; 4] {
        let mut out = [0.0; 4];
        let rows_max = |rows: &[Sums], c: usize| rows.iter().map(|r| r[c].unsigned_abs()).max().unwrap_or(0) as f64;
        for (c, o) in out.iter_mut().enumerate().take(classes) {
            *o = match self {
                Table::Linear { rows, starts } => {
                    rows[0][c].unsigned_abs() as f64
                        + (0..FEATURES)
                            .map(|f| rows_max(&rows[starts[f] as usize..starts[f] as usize + CATEGORIES[f] as usize], c))
                            .sum::<f64>()
                }
                Table::Tan { bias, rows, off, step, par, .. } => {
                    bias[c].unsigned_abs() as f64
                        + (0..FEATURES)
                            .map(|f| {
                                let n = if step[f] == 0 { 1 } else { CATEGORIES[par[f] as usize] as usize };
                                rows_max(&rows[off[f] as usize..off[f] as usize + n * CATEGORIES[f] as usize], c)
                            })
                            .sum::<f64>()
                }
            } / SCALE
                + QUANT;
        }
        out
    }
}

/// What a node does with its sums.
#[derive(Clone, Debug)]
enum Role {
    /// It picks the child of its most probable class. `product`: its probabilities are products of probabilities
    /// whose logs are the sums (naive Bayes, TAN), else a softmax of the sums (logistic regression). `margin`: the
    /// smallest lead, in sum units, that proves the float argmax.
    Router { children: Vec<usize>, product: bool, margin: i64 },
    /// It predicts: its sums are its log-probabilities, or with `probit` (eps, beta^2) a probit's latent mean and
    /// variance (without beta^2).
    Leaf { probit: Option<(f64, f64)> },
}

#[derive(Clone, Debug)]
pub struct Node {
    table: Table,
    classes: usize,
    role: Role,
    /// Per lane, a bound on |sum / 2^32 - the float value it stands for| (see `Tables::node`).
    err: [f64; 4],
    /// Per lane, the sum over the features of the largest |weight| (and the bias's): a bound on the sum's size.
    abs: [f64; 4],
}

impl Node {
    /// The index of the child the float router picks, if the sums prove it; None if too close to call.
    #[inline]
    fn pick(&self, s: &Sums, margin: i64, product: bool) -> Option<usize> {
        let mut top = 0;
        for c in 1..self.classes {
            if s[c] > s[top] {
                top = c;
            }
        }
        for c in 0..self.classes {
            if c != top && s[top] - s[c] <= margin {
                return None;
            }
        }
        if product && (s[top] as f64) < LOW_LOG * SCALE {
            return None;
        }
        Some(top)
    }
}

/// A graph's nodes in fixed point, built once per model (a calibrated wrapper keeps them; see `Compound::with_tables`).
#[derive(Clone, Debug)]
pub struct Tables {
    /// The graph they were built from (its address and size), checked on use.
    graph: (usize, usize),
    nodes: Vec<Node>,
}

impl Tables {
    /// The graph's nodes in fixed point; None if a node can't be held in fixed point (a CNBP or TAN without
    /// `use_probs`, a TAN of other than 3 classes, more than 4 classes, a weight out of range), or the graph is too
    /// big.
    pub(crate) fn new(graph: &Graph) -> Option<Tables> {
        if graph.nodes.len() > MAX_NODES || graph.categories.as_slice() != CATEGORIES.as_slice() || !graph.dead.is_empty() {
            return None;
        }
        let nodes = graph.nodes.iter().map(|n| Self::node(&n.classifier, &n.kind)).collect::<Option<Vec<_>>>()?;
        Some(Tables { graph: (graph as *const Graph as usize, graph.nodes.len()), nodes })
    }

    fn node(model: &NodeModel, kind: &Kind) -> Option<Node> {
        let classes = model.class_num();
        if !(2..=4).contains(&classes) {
            return None;
        }
        let linear = |fx: fixed::FxLinear| (Table::Linear { rows: fx.table().to_vec(), starts: *fx.starts() }, fx.head());
        let (table, head) = match model {
            NodeModel::Lr(m) => linear(m.fixed()?),
            NodeModel::Cnbp(m) => linear(m.fixed()?),
            NodeModel::Probit(m) => linear(m.fixed()?),
            NodeModel::Tan(m) => {
                let (bias, rows, look) = m.fixed_tables()?;
                if look.len() != FEATURES {
                    return None;
                }
                let (mut off, mut step, mut par, mut kids) = ([0u32; FEATURES], [0u8; FEATURES], [0u8; FEATURES], [0u128; FEATURES]);
                for (f, &(o, s, p)) in look.iter().enumerate() {
                    (off[f], step[f], par[f]) = (u32::try_from(o).ok()?, u8::try_from(s).ok()?, u8::try_from(p).ok()?);
                    if s != 0 {
                        if p >= FEATURES || s != CATEGORIES[f] as usize {
                            return None;
                        }
                        kids[p] |= 1u128 << f;
                    }
                }
                (Table::Tan { bias, rows, off, step, par, kids }, Head::Logits)
            }
        };
        let abs = table.abs_sums(classes);
        let product = matches!(model, NodeModel::Cnbp(_) | NodeModel::Tan(_));
        let (role, err) = match kind {
            Kind::Leaf => {
                if classes != 3 {
                    return None;
                }
                match head {
                    // A float sum of 71 terms: gamma_70 times the sum of their sizes.
                    Head::Logits => (Role::Leaf { probit: None }, abs.map(|a| QUANT + gamma(70) * a)),
                    // The mean as above; the variance's sum adds beta^2 last.
                    Head::Probit { eps, beta2 } => {
                        let err = [QUANT + gamma(70) * abs[0], QUANT + gamma(71) * (abs[1] + beta2), 0.0, 0.0];
                        (Role::Leaf { probit: Some((eps, beta2)) }, err)
                    }
                }
            }
            Kind::Branch { children } => {
                if children.len() != classes || matches!(model, NodeModel::Probit(_)) {
                    return None;
                }
                let err = abs.map(|a| {
                    if product {
                        // Logs of the cached probabilities, each within 1 ulp of the exact log (2u |log|), and a float
                        // product of 71 factors, within gamma_70 relatively (1.01 gamma_70 in logs).
                        QUANT + 2.01 * U * a + 1.01 * gamma(70)
                    } else {
                        QUANT + gamma(70) * a
                    }
                });
                let worst = err[..classes].iter().copied().fold(0.0, f64::max);
                // The float sums of the top class and any other are then more than 1e-12 apart, so the softmax or
                // the products order them the same way (strictly).
                let margin = ((2.0 * worst + 1e-12) * SCALE * 1.001).ceil() as i64 + 1;
                (Role::Router { children: children.clone(), product, margin }, err)
            }
        };
        Some(Node { table, classes, role, err, abs })
    }
}

/// The features a move changed: each (feature, new value) once, as a bit set, and the new position's encoding.
#[derive(Clone, Copy)]
pub struct Changes {
    list: [(u8, u8); 16],
    n: usize,
    mask: u128,
    pub enc: Encoded,
}

impl Default for Acc {
    fn default() -> Self {
        Acc { enc: [0; FEATURES], sums: Vec::new() }
    }
}

impl Default for Changes {
    fn default() -> Self {
        Changes { list: [(0, 0); 16], n: 0, mask: 0, enc: [0; FEATURES] }
    }
}

impl Changes {
    /// The changes from `pos`, whose encoding is `old`, to `next`, one move later.
    pub fn of(pos: &Chess, next: &Chess, old: &Encoded) -> Changes {
        let mut ch = Changes::default();
        ch.fill(pos, next, old, fixed::checking());
        ch
    }

    /// `of`, in place; with `check`, against `encode`.
    #[inline]
    pub fn fill(&mut self, pos: &Chess, next: &Chess, old: &Encoded, check: bool) {
        let (mut n, mut mask) = (0usize, 0u128);
        let (enc, list) = (&mut self.enc, &mut self.list);
        *enc = *old;
        fixed::Acc::changes(pos, next, |f, v| {
            if enc[f] != v {
                enc[f] = v;
                if n < list.len() {
                    list[n] = (f as u8, v);
                }
                n += 1;
                mask |= 1u128 << f;
            }
        });
        // At most 4 squares, the side to move, en passant and 4 castling rights.
        assert!(n <= list.len(), "{n} changed features");
        (self.n, self.mask) = (n, mask);
        if check {
            assert_eq!(self.enc, encoding::encode(next), "incremental encoding differs from encode()");
        }
    }

    #[inline]
    fn list(&self) -> &[(u8, u8)] {
        &self.list[..self.n]
    }
}

/// A position's encoding and every node's sums.
#[derive(Clone, Debug)]
pub struct Acc {
    pub enc: Encoded,
    sums: Vec<Sums>,
}

/// A reply at depth 2: its value for the side to move after it, in float (`exact`) or estimated within `err`, and the
/// leaf it routes to.
#[derive(Clone, Copy, Debug)]
pub struct Reply {
    value: f64,
    err: f64,
    exact: bool,
    white: bool,
    leaf: u32,
}

impl Reply {
    /// A reply whose value is known: the game ends there.
    pub fn known(value: f64) -> Reply {
        Reply { value, err: 0.0, exact: true, white: true, leaf: 0 }
    }
}

/// A tree or DAG in fixed point, with the calibration its values use.
pub struct Compound<'a> {
    graph: &'a Graph,
    cal: Calibration,
    nodes: Cow<'a, [Node]>,
    /// Per node, a leaf's bound on |estimate - float value| for every position (logs leaves, see `estimate`); NaN for
    /// probit leaves (a bound per position) and routers, infinite if too large to use.
    leaf_err: Vec<f64>,
    /// CHESSAI_FX_CHECK and CHESSAI_FX_STATS, read once.
    pub check: bool,
    stats: bool,
    /// Buffers reused by every search of the game (see `buffers`).
    scratch: std::cell::RefCell<Scratch>,
}

/// A search's buffers: the root's and a depth-1 position's accumulators, a move's changes and the replies.
#[derive(Default)]
pub struct Scratch {
    pub root: Acc,
    pub acc: Acc,
    pub ch: Changes,
    pub replies: Vec<Reply>,
}

impl<'a> Compound<'a> {
    /// The graph in fixed point under the whole-model calibration `cal`, its tables built afresh (see `Tables::new`).
    #[cfg(test)]
    pub(crate) fn new(graph: &'a Graph, cal: Calibration) -> Option<Compound<'a>> {
        let tables = Tables::new(graph)?;
        let leaf_err = Self::leaf_errs(&tables.nodes, cal);
        Some(Compound {
            graph,
            cal,
            nodes: Cow::Owned(tables.nodes),
            leaf_err,
            check: fixed::checking(),
            stats: stats_on(),
            scratch: Default::default(),
        })
    }

    /// The graph under `cal` with tables built before from this same graph (unchanged since: a model that is only
    /// played); None if `tables` are another graph's.
    pub(crate) fn with_tables(graph: &'a Graph, cal: Calibration, tables: &'a Tables) -> Option<Compound<'a>> {
        if tables.graph != (graph as *const Graph as usize, graph.nodes.len()) {
            return None;
        }
        let leaf_err = Self::leaf_errs(&tables.nodes, cal);
        Some(Compound {
            graph,
            cal,
            nodes: Cow::Borrowed(&tables.nodes),
            leaf_err,
            check: fixed::checking(),
            stats: stats_on(),
            scratch: Default::default(),
        })
    }

    /// The constant bounds of logs leaves (see `estimate`): with the float logits' sizes at most their weights'
    /// (|l_c| <= abs_c (1 + gamma_70)), |d| <= abs_0 + abs_1, |m| <= (abs_0 + abs_1) / 2 + abs_2 and |value| <=
    /// |p_0| max |d| + |p_2| + |p_3|.
    fn leaf_errs(nodes: &[Node], cal: Calibration) -> Vec<f64> {
        let Calibration { slope_winner: p0, slope_decisive: p1, offsets: [p2, p3] } = cal;
        nodes
            .iter()
            .map(|n| match n.role {
                Role::Leaf { probit: None } => {
                    let d = (n.abs[0] + n.abs[1]) * (1.0 + 1e-9);
                    let m = ((n.abs[0] + n.abs[1]) / 2.0 + n.abs[2]) * (1.0 + 1e-9);
                    let v = p0.abs() * d + p2.abs() + p3.abs();
                    let rounding = 8.0 * U * (p0.abs() * d + p1.abs() * m + p2.abs() + p3.abs() + v);
                    let err = 4.0 * (p0.abs() * (n.err[0] + n.err[1] + 2.0 * U * d) + rounding) + 1e-10;
                    if err.is_finite() && err < 1e-3 { err } else { f64::INFINITY }
                }
                _ => f64::NAN,
            })
            .collect()
    }

    #[inline]
    fn count(&self, counter: &AtomicU64) {
        if self.stats {
            counter.fetch_add(1, Relaxed);
        }
    }

    /// The accumulator of a position, summed in full.
    pub fn root(&self, pos: &Chess) -> Acc {
        let mut acc = self.empty();
        self.root_into(pos, &mut acc);
        acc
    }

    /// Makes `out` the accumulator of a position, summed in full.
    pub fn root_into(&self, pos: &Chess, out: &mut Acc) {
        out.enc = encoding::encode(pos);
        out.sums.clear();
        out.sums.extend(self.nodes.iter().map(|n| n.table.full(&out.enc)));
    }

    /// The buffers of a search (see `Scratch`), borrowed for its duration.
    pub fn buffers(&self) -> std::cell::RefMut<'_, Scratch> {
        self.scratch.borrow_mut()
    }

    /// Makes `out` the accumulator of the position `ch` leads to from `acc`'s.
    pub fn child_into(&self, acc: &Acc, ch: &Changes, out: &mut Acc) {
        out.enc = ch.enc;
        out.sums.clear();
        out.sums.extend(self.nodes.iter().zip(&acc.sums).map(|(n, s)| n.table.moved(s, &acc.enc, ch)));
        if self.check {
            for (n, s) in self.nodes.iter().zip(&out.sums) {
                assert_eq!(*s, n.table.full(&ch.enc), "incremental compound sums differ from the full sums");
            }
        }
    }

    /// An empty accumulator, for `root_into` and `child_into`.
    pub fn empty(&self) -> Acc {
        Acc { enc: [0; FEATURES], sums: Vec::with_capacity(self.nodes.len()) }
    }

    /// The float model's leaf for the position `ch` leads to from `acc`'s, and with `SUMS` the leaf's sums there.
    #[inline]
    fn route<const SUMS: bool>(&self, acc: &Acc, ch: &Changes) -> (usize, Sums) {
        let leaf_sums = |leaf: usize| {
            if !SUMS {
                return [0; 4];
            }
            let s = self.nodes[leaf].table.moved(&acc.sums[leaf], &acc.enc, ch);
            if self.check {
                assert_eq!(s, self.nodes[leaf].table.full(&ch.enc), "incremental leaf sums differ from the full sums");
            }
            s
        };
        let mut idx = 0;
        loop {
            let node = &self.nodes[idx];
            let Role::Router { children, product, margin } = &node.role else { return (idx, leaf_sums(idx)) };
            let s = node.table.moved(&acc.sums[idx], &acc.enc, ch);
            if self.check {
                assert_eq!(s, node.table.full(&ch.enc), "incremental router sums differ from the full sums");
            }
            self.count(&STATS.decisions);
            match node.pick(&s, *margin, *product) {
                Some(c) => {
                    if self.check {
                        assert_eq!(self.graph.leaf_from(idx, &ch.enc), self.graph.leaf_from(children[c], &ch.enc), "router decision");
                    }
                    idx = children[c];
                }
                None => {
                    self.count(&STATS.fallbacks);
                    let leaf = self.graph.leaf_from(idx, &ch.enc);
                    return (leaf, leaf_sums(leaf));
                }
            }
        }
    }

    /// The float value of leaf `leaf` for a position (`enc`) whose side to move is white if `white`: the whole
    /// calibration's z_win - z_loss of the leaf's log-probabilities, as `Calibrated::log_odds_direct` computes it for
    /// a position routed to that leaf.
    #[inline]
    pub fn exact(&self, leaf: usize, enc: &Encoded, white: bool) -> f64 {
        let mut logs = [0.0; 3];
        self.graph.node_predict_log(leaf, enc, &mut logs);
        let z = self.cal.logits(&logs);
        if white { z[0] - z[1] } else { z[1] - z[0] }
    }

    /// The float value for the side to move of the position `ch` leads to from `acc`'s.
    pub fn value(&self, acc: &Acc, ch: &Changes, white: bool) -> f64 {
        let (leaf, _) = self.route::<false>(acc, ch);
        self.exact(leaf, &ch.enc, white)
    }

    /// The position `ch` leads to from `acc`'s as a reply at depth 2: its leaf and estimated value, or its float value
    /// if no estimate is safe.
    #[inline]
    pub fn reply(&self, acc: &Acc, ch: &Changes, white: bool) -> Reply {
        let (leaf, s) = self.route::<true>(acc, ch);
        self.count(&STATS.replies);
        match self.estimate(leaf, &s, white) {
            Some((value, err)) => Reply { value, err, exact: false, white, leaf: leaf as u32 },
            None => {
                self.count(&STATS.unestimated);
                Reply { value: self.exact(leaf, &ch.enc, white), err: 0.0, exact: true, white, leaf: leaf as u32 }
            }
        }
    }

    /// A leaf's value for the side to move from its sums, and a bound on the distance to its float value (`exact`).
    /// None when either is not finite or the bound is not small.
    ///
    /// The float value is z_0 - z_1 with z_c the calibration's sum p_0 f_c0 + p_1 m + p_2 [c=0] + p_3 [c=1], f_00 =
    /// d / 2 = -f_10, d = l_0 - l_1, m = (l_0 + l_1) / 2 - l_2, l the leaf's float log-probabilities: in exact
    /// arithmetic p_0 d + p_2 - p_3. Its rounding is within 4u (|p_0 d| + |p_1 m|) + u (|p_2| + |p_3| + |value|), the
    /// estimate's within 3u of the same sizes: 8u covers both. d itself is within the leaf's sum bounds (logs: err_0 +
    /// err_1; probit: the derivatives of ln Phi times the mean's and variance's bounds, the rounding of the z's and a
    /// generous allowance for the normal CDF's rational approximations, 1e-13 + 16u |ln Phi|). The total is then
    /// multiplied by 4 and 1e-10 added.
    ///
    /// A probit's draw log l_2 is not computed: with |z_w|, |z_b| < 30 and eps / s > 1e-6 the draw's probability is
    /// at least 2e-6 phi(30), so |l_2| < 465 (and |m| < (|l_0| + |l_1|) / 2 + 465), and its float value is finite, so
    /// the float value is finite. Outside those limits the reply is valued in float.
    #[inline]
    fn estimate(&self, leaf: usize, s: &Sums, white: bool) -> Option<(f64, f64)> {
        let Calibration { slope_winner: p0, slope_decisive: p1, offsets: [p2, p3] } = self.cal;
        let node = &self.nodes[leaf];
        let Role::Leaf { probit } = node.role else { unreachable!("routing ends at a leaf") };
        let Some((eps, beta2)) = probit else {
            // Logs: the bound is the leaf's constant one (`leaf_errs`).
            let err = self.leaf_err[leaf];
            if err.is_infinite() {
                return None;
            }
            let v = p0 * ((s[0] - s[1]) as f64 / SCALE) + (p2 - p3);
            return Some((if white { v } else { -v }, err));
        };
        let mean = s[0] as f64 / SCALE;
        let var = s[1] as f64 / SCALE + beta2;
        if !(var > 1e-12 && var.is_finite() && mean.is_finite()) {
            return None;
        }
        let sd = var.sqrt();
        let (zw, zb) = ((mean - eps) / sd, (-mean - eps) / sd);
        if !(zw.abs() < 30.0 && zb.abs() < 30.0 && eps / sd > 1e-6) {
            return None;
        }
        let (lw, lb) = (crate::probit::ln_cdf_first(zw), crate::probit::ln_cdf_first(zb));
        // The inverse Mills ratio phi(z) / Phi(z) is at most |z| + 1: d ln Phi(z) / dz.
        let (rw, rb) = (zw.abs() + 1.0, zb.abs() + 1.0);
        let (mean_err, var_err) = (node.err[0] + U * mean.abs(), node.err[1] + 2.0 * U * var);
        let derivs = 2.0 * ((rw + rb) / sd * mean_err + (rw * zw.abs() + rb * zb.abs()) / (2.0 * var) * var_err);
        let z_rounding = 2.0 * 4.0 * U * (rw * zw.abs() + rb * zb.abs());
        let approx = 2.0 * (2e-13 + 16.0 * U * (lw.abs() + lb.abs()));
        let d = lw - lb;
        let d_err = derivs + z_rounding + approx + 2.0 * U * d.abs();
        let m_size = (lw.abs() + lb.abs()) / 2.0 + 1000.0;
        if !d.is_finite() {
            return None;
        }
        let v = p0 * d + (p2 - p3);
        let rounding = 8.0 * U * (p0.abs() * d.abs() + p1.abs() * m_size + p2.abs() + p3.abs() + v.abs());
        let err = 4.0 * (p0.abs() * d_err + rounding) + 1e-10;
        if !(v.is_finite() && err < 1e-3) {
            return None;
        }
        Some((if white { v } else { -v }, err))
    }

    /// The value `negamax` finds for the position before the replies (depth 1, full window): the largest -value,
    /// the first on ties, NaN ignored, as its loop computes it, valuing in float only the replies that can matter.
    /// `enc_of(i)`: reply i's encoding.
    pub fn best(&self, replies: &mut [Reply], mut enc_of: impl FnMut(usize) -> Encoded) -> f64 {
        let mut upper = f64::INFINITY;
        for r in replies.iter() {
            let u = r.value + r.err;
            if u < upper {
                upper = u;
            }
        }
        if self.check {
            for (i, r) in replies.iter().enumerate().filter(|(_, r)| !r.exact) {
                let exact = self.exact(r.leaf as usize, &enc_of(i), r.white);
                assert!(exact.is_nan() || (exact - r.value).abs() <= r.err, "value {exact} outside {} +- {}", r.value, r.err);
            }
        }
        let mut close = 0u64;
        for (i, r) in replies.iter_mut().enumerate() {
            if !r.exact && r.value - r.err <= upper {
                r.value = self.exact(r.leaf as usize, &enc_of(i), r.white);
                (r.err, r.exact) = (0.0, true);
                close += 1;
            }
        }
        if self.stats {
            STATS.candidates.fetch_add(close, Relaxed);
            STATS.extra.fetch_add(close.saturating_sub(1), Relaxed);
        }
        if !replies.iter().any(|r| r.exact && r.value <= upper) {
            self.count(&STATS.everything);
            for (i, r) in replies.iter_mut().enumerate().filter(|(_, r)| !r.exact) {
                r.value = self.exact(r.leaf as usize, &enc_of(i), r.white);
                (r.err, r.exact) = (0.0, true);
            }
        }
        // `negamax`'s loop, over the replies valued in float (the others are larger than one of them).
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

    /// Counts a search of `moves` moves, for CHESSAI_FX_STATS.
    pub fn count_search(&self, moves: usize) {
        if self.stats {
            STATS.searches.fetch_add(1, Relaxed);
            STATS.moves.fetch_add(moves as u64, Relaxed);
        }
    }
}

/// Counts of the fixed-point decisions, for CHESSAI_FX_STATS=1.
pub struct Stats {
    /// Router decisions, and those left to the float router.
    pub decisions: AtomicU64,
    pub fallbacks: AtomicU64,
    /// Depth-2 replies routed to a leaf, and those valued in float at once (no safe estimate).
    pub replies: AtomicU64,
    pub unestimated: AtomicU64,
    /// Estimated replies valued in float as candidates for the smallest, and those beyond the first of a position.
    pub candidates: AtomicU64,
    pub extra: AtomicU64,
    /// Positions whose replies were all valued in float (no candidate was within the bound in float).
    pub everything: AtomicU64,
    /// Searches and their moves.
    pub searches: AtomicU64,
    pub moves: AtomicU64,
}

pub static STATS: Stats = Stats {
    decisions: AtomicU64::new(0),
    fallbacks: AtomicU64::new(0),
    replies: AtomicU64::new(0),
    unestimated: AtomicU64::new(0),
    candidates: AtomicU64::new(0),
    extra: AtomicU64::new(0),
    everything: AtomicU64::new(0),
    searches: AtomicU64::new(0),
    moves: AtomicU64::new(0),
};

pub fn stats_on() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var("CHESSAI_FX_STATS").is_ok_and(|v| v == "1"))
}

/// Prints the counts to stderr, with CHESSAI_FX_STATS=1.
pub fn report() {
    if !stats_on() {
        return;
    }
    let g = |c: &AtomicU64| c.load(Relaxed);
    eprintln!(
        "compound fx: {} searches, {} moves; router decisions {}, float fallbacks {}; depth-2 replies {}, valued in float \
         at once {}, candidates valued in float {} ({} beyond one per position), positions valued all in float {}",
        g(&STATS.searches),
        g(&STATS.moves),
        g(&STATS.decisions),
        g(&STATS.fallbacks),
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
    use crate::classifier::Classifier;
    use crate::cnbp::Cnbp;
    use crate::delegation::{DelegationParams, ErrorMeasure, ErrorTarget, NodeKind, ReviseOn, RouteTarget, SplitRule};
    use crate::game::Game;
    use crate::logreg::Logreg;
    use crate::probit::{Probit, ProbitParams};
    use crate::search::Search;
    use crate::tan::{Tan, TanOptions};
    use rand::rngs::StdRng;
    use rand::{RngExt, SeedableRng};
    use shakmaty::Position;

    fn params() -> DelegationParams {
        DelegationParams {
            class_num: 3,
            branch_factor: 3,
            split_threshold: 4,
            split_limit: 3,
            forget_factor: 10.0,
            nb_alpha: 1.0,
            use_probs: true,
            leaf: NodeKind::Lr,
            router: NodeKind::Lr,
            lr_rate: 0.05,
            error: ErrorMeasure::Abs,
            split_z: 0.0,
            split_rule: SplitRule::Worse,
            route: RouteTarget::Error,
            error_target: ErrorTarget::Best,
            leaf_share: 0.0,
            split_trial: None,
            trial_error: ErrorMeasure::Abs,
            route_switch: None,
            error_router: NodeKind::Lr,
            revise: None,
            seam_weight: 0.0,
            reset_leaves: None,
            leaf_residual: false,
            residual_l2: 0.0,
            residual_inherit_sums: false,
            log_fresh_gains: false,
            revise_on: ReviseOn::Stream,
            residual_delta_rate: 1.0,
            probit: ProbitParams::default(),
            split_noise: 0.0,
            probit_shared_eps: false,
            tan_m: crate::tan::DEFAULT_M,
        }
    }

    /// A graph under a calibration of the whole model, as `Calibrated` in the binary plays it.
    struct CalGraph {
        graph: Graph,
        cal: Calibration,
    }

    impl Classifier for CalGraph {
        fn train(&mut self, _data: &[u8], _label: &[f64]) {}
        fn refresh(&mut self) {}
        fn predict(&self, data: &[u8], out: &mut [f64]) {
            self.graph.predict(data, out)
        }
        fn predict_log(&self, data: &[u8], out: &mut [f64]) {
            self.graph.predict_log(data, out)
        }
        fn save(&self, _path: &std::path::Path) -> std::io::Result<()> {
            Ok(())
        }
        fn log_odds_direct(&self, data: &[u8], white_to_move: bool) -> Option<f64> {
            let mut logs = [0.0; 3];
            self.graph.predict_log(data, &mut logs);
            let z = self.cal.logits(&logs);
            Some(if white_to_move { z[0] - z[1] } else { z[1] - z[0] })
        }
        fn compound_play(&self) -> Option<Compound<'_>> {
            Compound::new(&self.graph, self.cal)
        }
    }

    /// Positions of random games, each with a random label.
    fn positions(seed: u64, n: usize) -> Vec<(Encoded, [f64; 3])> {
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

    fn lr(seed: u64, scale: f64) -> NodeModel {
        let mut m = Logreg::new(3, &CATEGORIES, 0.1);
        m.randomize(seed, scale);
        NodeModel::Lr(m)
    }

    fn cnbp(seed: u64) -> NodeModel {
        let mut m = Cnbp::new(3, &CATEGORIES, 1.0, true);
        for (x, y) in positions(seed, 2000) {
            m.train(&x, &y);
        }
        m.refresh();
        NodeModel::Cnbp(m)
    }

    fn tan(seed: u64) -> NodeModel {
        let mut m = Tan::new(3, &CATEGORIES, 1.0, TanOptions::fixed(crate::tan::DEFAULT_M), true);
        for (x, y) in positions(seed, 2000) {
            m.train(&x, &y);
        }
        m.refresh();
        NodeModel::Tan(m)
    }

    fn probit(seed: u64) -> NodeModel {
        let mut m = Probit::new(&CATEGORIES, ProbitParams::default());
        for (x, y) in positions(seed, 3000) {
            m.train(&x, &y);
        }
        NodeModel::Probit(m)
    }

    /// A router (node 0) over three leaves.
    fn tree(router: NodeModel, leaves: [NodeModel; 3]) -> Graph {
        let mut graph = Graph::new(&CATEGORIES, params());
        let mut nodes = vec![crate::delegation::Node::new(Kind::Branch { children: vec![1, 2, 3] }, router)];
        nodes.extend(leaves.into_iter().map(|l| crate::delegation::Node::new(Kind::Leaf, l)));
        graph.nodes = nodes;
        graph
    }

    const CAL: Calibration = Calibration { slope_winner: 0.7, slope_decisive: 0.3, offsets: [0.2, -0.1] };

    /// Every position of a few random games: the compound search's values equal the float search's, bit for bit,
    /// at depth 1 and 2.
    fn same_values(model: &CalGraph, seed: u64, games: usize, plies: usize) {
        let c = model.compound_play().expect("compound model");
        let mut rng = StdRng::seed_from_u64(seed);
        for _ in 0..games {
            let mut game = Game::default();
            for _ in 0..plies {
                if game.result().is_some() {
                    break;
                }
                for depth in [1, 2] {
                    let float = Search::new(model, &game).root_values(&game, depth);
                    let fx = Search::new(model, &game).with_compound(Some(&c)).root_values(&game, depth);
                    let bits = |v: &[f64]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
                    assert_eq!(bits(&fx), bits(&float), "depth {depth}: {fx:?} vs {float:?}");
                }
                let moves = game.position().legal_moves();
                game.play(moves[rng.random_range(0..moves.len())]);
            }
        }
    }

    #[test]
    fn lr_router_and_leaves_value_exactly() {
        let model = CalGraph { graph: tree(lr(1, 0.3), [lr(2, 0.2), lr(3, 0.2), lr(4, 0.2)]), cal: CAL };
        same_values(&model, 1, 3, 60);
    }

    #[test]
    fn naive_bayes_and_tan_routers_and_probit_and_naive_bayes_leaves_value_exactly() {
        let model = CalGraph { graph: tree(cnbp(5), [probit(6), cnbp(7), lr(8, 0.2)]), cal: CAL };
        same_values(&model, 2, 2, 50);
        let model = CalGraph { graph: tree(tan(9), [lr(10, 0.2), probit(11), tan(12)]), cal: CAL };
        same_values(&model, 3, 2, 50);
    }

    #[test]
    fn ties_fall_back_and_stay_exact() {
        // A router whose classes are equal: every decision is a tie, left to the float router (its first class).
        // Leaves with no weights: every reply ties, so every reply is a candidate.
        let mut same = Logreg::new(3, &CATEGORIES, 0.1);
        same.randomize(13, 0.3);
        let w = same.weights_mut();
        let width = w.len() / 3;
        let (first, rest) = w.split_at_mut(width);
        rest[..width].copy_from_slice(first);
        rest[width..].copy_from_slice(first);
        let zero = NodeModel::Lr(Logreg::new(3, &CATEGORIES, 0.1));
        let model = CalGraph { graph: tree(NodeModel::Lr(same), [zero.clone(), lr(14, 0.2), zero]), cal: CAL };
        let before = STATS.fallbacks.load(Relaxed);
        same_values(&model, 4, 2, 40);
        if stats_on() {
            assert!(STATS.fallbacks.load(Relaxed) > before);
        }
        // Every decision of that router is a fallback.
        let c = model.compound_play().unwrap();
        let Role::Router { margin, product, .. } = c.nodes[0].role else { panic!() };
        let acc = c.root(&Chess::default());
        assert_eq!(c.nodes[0].pick(&acc.sums[0], margin, product), None);
    }

    #[test]
    fn a_dag_values_exactly() {
        // Node 0 routes to 1, 2 or 3; node 1 routes to 3 or 4: leaf 3 has two parents.
        let mut graph = Graph::new(&CATEGORIES, params());
        let mut r2 = Logreg::new(2, &CATEGORIES, 0.1);
        r2.randomize(15, 0.3);
        graph.nodes = vec![
            crate::delegation::Node::new(Kind::Branch { children: vec![1, 2, 3] }, lr(16, 0.3)),
            crate::delegation::Node::new(Kind::Branch { children: vec![3, 4] }, NodeModel::Lr(r2)),
            crate::delegation::Node::new(Kind::Leaf, lr(17, 0.2)),
            crate::delegation::Node::new(Kind::Leaf, probit(18)),
            crate::delegation::Node::new(Kind::Leaf, cnbp(19)),
        ];
        same_values(&CalGraph { graph, cal: CAL }, 5, 2, 50);
    }

    #[test]
    fn estimates_are_within_their_bounds_and_routes_are_the_float_leaves() {
        let models = [
            CalGraph { graph: tree(lr(20, 0.5), [lr(21, 0.5), probit(22), cnbp(23)]), cal: CAL },
            CalGraph { graph: tree(tan(24), [tan(25), probit(26), lr(27, 1.0)]), cal: Calibration::NONE },
        ];
        for model in &models {
            let c = model.compound_play().unwrap();
            let mut rng = StdRng::seed_from_u64(28);
            let (mut n, mut worst) = (0, 0.0f64);
            for _ in 0..4 {
                let mut pos = Chess::default();
                for _ in 0..80 {
                    let moves = pos.legal_moves();
                    if moves.is_empty() {
                        break;
                    }
                    let acc = c.root(&pos);
                    for &m in &moves {
                        let mut next = pos.clone();
                        next.play_unchecked(m);
                        let ch = Changes::of(&pos, &next, &acc.enc);
                        assert_eq!(ch.enc, encoding::encode(&next));
                        let mut child = c.empty();
                        c.child_into(&acc, &ch, &mut child);
                        for (node, s) in c.nodes.iter().zip(&child.sums) {
                            assert_eq!(*s, node.table.full(&ch.enc));
                        }
                        let (leaf, s) = c.route::<true>(&acc, &ch);
                        assert_eq!(leaf, model.graph.leaf(&ch.enc));
                        let white = next.turn().is_white();
                        let exact = c.exact(leaf, &ch.enc, white);
                        assert_eq!(exact.to_bits(), model.log_odds_direct(&ch.enc, white).unwrap().to_bits());
                        if let Some((v, err)) = c.estimate(leaf, &s, white) {
                            assert!((exact - v).abs() <= err, "{exact} vs {v} +- {err}");
                            assert!(err < 1e-6, "bound {err}");
                            worst = worst.max((exact - v).abs() / err);
                            n += 1;
                        }
                    }
                    pos.play_unchecked(moves[rng.random_range(0..moves.len())]);
                }
            }
            assert!(n > 5000, "{n} estimates");
            assert!(worst < 0.5, "an error used {worst} of its bound");
        }
    }

    #[test]
    fn the_best_reply_follows_the_float_loop() {
        // Hand-made replies: the smallest value wins, the first of equal ones, NaN ignored, signed zeros kept.
        let model = CalGraph { graph: tree(lr(29, 0.3), [lr(30, 0.2), lr(31, 0.2), lr(32, 0.2)]), cal: CAL };
        let c = model.compound_play().unwrap();
        let best = |values: &[f64]| c.best(&mut values.iter().map(|&v| Reply::known(v)).collect::<Vec<_>>(), |_| [0; FEATURES]);
        assert_eq!(best(&[0.5, -0.25, 0.75]), 0.25);
        assert_eq!(best(&[f64::NAN, 0.25]), -0.25);
        assert_eq!(best(&[f64::NAN]), f64::NEG_INFINITY);
        assert_eq!(best(&[0.0, -0.0]).to_bits(), (-0.0f64).to_bits());
        assert_eq!(best(&[-0.0, 0.0]).to_bits(), 0.0f64.to_bits());
        assert_eq!(best(&[1.0, f64::NEG_INFINITY, f64::NAN]), f64::INFINITY);
    }
}
