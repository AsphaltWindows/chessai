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
/// Most nodes a compound model may have. hybrid20: 128 (hybrid5-19: 64). Nothing in the fixed-point path depends on
/// the node count: every node's table, sums, margin and error bound are its own (sums of at most 71 entries, each
/// under 2^20 units of 2^-32, so under 2^59; cell offsets under 10^6 in logs), the route follows child indices, and
/// `Acc` and `Tables` are vectors. The cap is a speed guard: `child_into` updates every node at each interior
/// position while the float path evaluates only the route's nodes, so the gain shrinks with the node count
/// (measured 11-12: x0.37 a game at 81-93 nodes, x0.58 at 121, ply 2). Past 128, measure before raising it.
pub const MAX_NODES: usize = 128;

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
    /// hybrid15: for a DAG whose path tables have cells, its route states (see `Cells`); None otherwise.
    cells: Option<Cells>,
}

/// hybrid15: a DAG with path context whose tables have cells (router splits made sibling sets). A node's prediction
/// then depends on the path that reached it, through its cells' values (`pathctx`), so routing walks states: a node
/// and the marks of the path to it (sorted; at most one per sibling set, so the order doesn't matter to the values).
/// Every state any path reaches is listed up front, from the root down every child of every router.
///
/// At a router state with cells the float router's choice is the argmax of its classifier's output with the cells'
/// part added: a logistic regression's logits plus the cells' offsets (`lr_offsets`, then softmax), a CNBP's class
/// products times the cells' factors (`cnbp_factors`, then normalised). In fixed point the state adds, per class, the
/// offset or ln of the factor, rounded to the sums' unit, to the node's sums, and its margin covers that rounding,
/// ln's own (2u), and the float addition or product the cells add (u). Leaves add the cells' part to their estimate
/// (logs: the offsets' difference; probit: the cells' mean and variance) with the bounds widened likewise. Float
/// values and fallbacks use the float model with the state's marks (`Graph::node_predict_path`,
/// `Graph::descend_from`), so every value is the float model's.
#[derive(Clone, Debug)]
pub struct Cells {
    states: Vec<State>,
    /// The state of each (node, sorted marks).
    index: std::collections::HashMap<(usize, Vec<crate::pathctx::Mark>), u32>,
}

/// One route state (see `Cells`).
#[derive(Clone, Debug)]
struct State {
    node: usize,
    /// The path's marks, sorted.
    marks: Vec<crate::pathctx::Mark>,
    /// Whether the node's table has cells (else the state is its node, as without path context).
    cells: bool,
    /// Routers: per class, the cells' part in sum units (0 without cells), the margin, and each class's child state.
    offs: Sums,
    margin: i64,
    next: Vec<u32>,
    /// Logs leaves with cells: the cells' offsets' difference (class 0 - class 1), and per lane the logs' size bound
    /// and the bound on the distance of the fixed-point logs plus the offsets from the float logs (the node's, widened
    /// by the cells: see `Tables::cells`), for the state's bound under a calibration (`Compound::state_errs`).
    diff: f64,
    abs: [f64; 3],
    err: [f64; 2],
    /// Probit leaves with cells: the cells' latent mean and variance (mu, s2).
    probit: (f64, f64),
}

/// Most route states a DAG with cells may have (else it plays in float). hybrid21 (2026-11-13): 65,536, was 4,096
/// (hybrid15's). The cap only decides eligibility: it bounds the up-front enumeration (`Tables::cells`), nothing is
/// sized by it, states are looked up by index and the (node, marks) map with u32 ids, and every state keeps its own
/// margin and float fallback, so any value is exact (the argument of hybrid20's `MAX_NODES`). The warm ply-2
/// standard's D-P DAGs passed 4,096 at v100 (4,756 and 6,601 states) and played entirely in float.
const MAX_STATES: usize = 65536;

/// Whether DAGs with cells play in fixed point (hybrid15): CHESSAI_FX_CELLS=on (the default) or off (hybrid14: in
/// float). Either way the values are the float model's.
pub fn cells_on() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| match std::env::var("CHESSAI_FX_CELLS").as_deref() {
        Ok("on") | Err(_) => true,
        Ok("off") => false,
        Ok(other) => panic!("CHESSAI_FX_CELLS must be on or off, not {other:?}"),
    })
}

impl Tables {
    /// The graph's nodes in fixed point; None if a node can't be held in fixed point (a CNBP or TAN without
    /// `use_probs`, a TAN of other than 3 classes, more than 4 classes, a weight out of range), or the graph is too
    /// big.
    pub(crate) fn new(graph: &Graph) -> Option<Tables> {
        if graph.nodes.len() > MAX_NODES || graph.categories.as_slice() != CATEGORIES.as_slice() || !graph.dead.is_empty() {
            return None;
        }
        // hybrid14: a DAG with path context whose tables have no cells (no router split made a sibling set yet)
        // plays as without path context: every node predicts with its classifier alone and routes by its argmax
        // (`Graph::node_predict_path`, `Graph::descend`). hybrid15: one with cells walks route states (`Cells`).
        let cells = graph.has_cells();
        if cells && !cells_on() {
            return None;
        }
        let nodes = graph.nodes.iter().map(|n| Self::node(&n.classifier, &n.kind)).collect::<Option<Vec<_>>>()?;
        let cells = match cells {
            true => Some(Self::cells(graph, &nodes)?),
            false => None,
        };
        Some(Tables { graph: (graph as *const Graph as usize, graph.nodes.len()), nodes, cells })
    }

    /// hybrid15: the route states of a DAG with cells (see `Cells`); None past `MAX_STATES` or for a node whose
    /// cells' part can't be bounded (a classifier other than logistic regression and CNBP routers, logistic
    /// regression and probit leaves).
    fn cells(graph: &Graph, nodes: &[Node]) -> Option<Cells> {
        use crate::pathctx::Mark;
        let mut states: Vec<State> = Vec::new();
        let mut index: std::collections::HashMap<(usize, Vec<Mark>), u32> = std::collections::HashMap::new();
        // Every (node, marks) a path reaches: the root with none, then each router's children.
        let mut todo: Vec<(usize, Vec<Mark>)> = vec![(0, Vec::new())];
        while let Some((idx, marks)) = todo.pop() {
            if index.contains_key(&(idx, marks.clone())) {
                continue;
            }
            if states.len() >= MAX_STATES {
                return None;
            }
            index.insert((idx, marks.clone()), states.len() as u32);
            if let Kind::Branch { children } = &graph.nodes[idx].kind {
                for &child in children {
                    let mut m = marks.clone();
                    m.extend(graph.nodes[child].member);
                    m.sort_unstable();
                    todo.push((child, m));
                }
            }
            states.push(State {
                node: idx,
                marks,
                cells: false,
                offs: [0; 4],
                margin: 0,
                next: Vec::new(),
                diff: 0.0,
                abs: [0.0; 3],
                err: [0.0; 2],
                probit: (0.0, 0.0),
            });
        }
        for i in 0..states.len() {
            let (idx, marks) = (states[i].node, states[i].marks.clone());
            let node = &graph.nodes[idx];
            let fx = &nodes[idx];
            let mut values = Vec::new();
            if let Some(table) = &node.table {
                table.values_for(&marks, &mut values);
            }
            let cells = !values.is_empty();
            let table = node.table.as_deref();
            let st = &mut states[i];
            st.cells = cells;
            match (&node.kind, &fx.role) {
                (Kind::Branch { children }, Role::Router { margin, .. }) => {
                    st.next = children
                        .iter()
                        .map(|&child| {
                            let mut m = marks.clone();
                            m.extend(graph.nodes[child].member);
                            m.sort_unstable();
                            index[&(child, m)]
                        })
                        .collect();
                    st.margin = *margin;
                    if !cells {
                        continue;
                    }
                    let n = fx.classes;
                    let mut part = [0.0f64; 8];
                    // The cells' part in logs, and its distance from the float operation's effect, per class.
                    let mut extra_err = [0.0f64; 4];
                    match &node.classifier {
                        NodeModel::Lr(_) => {
                            table?.lr_offsets(&values, &mut part[..n]);
                            for c in 0..n {
                                // The float logit S + e rounds once: u |S + e| <= u (|S| + |e|), |S| <= abs (1 + gamma_70).
                                extra_err[c] = 1.01 * U * (fx.abs[c] * (1.0 + gamma(70)) + part[c].abs());
                            }
                        }
                        NodeModel::Cnbp(nb) => {
                            let mut f = [0.0f64; 8];
                            table?.cnbp_factors(&values, nb.alpha(), &mut f[..n]);
                            for c in 0..n {
                                if !(f[c] > 0.0 && f[c].is_finite()) {
                                    return None;
                                }
                                part[c] = f[c].ln();
                                // The float product times the factor rounds once (1.01 u in logs); ln is within 2u.
                                extra_err[c] = 1.01 * U + 2.0 * U * part[c].abs();
                            }
                        }
                        _ => return None,
                    }
                    let mut worst = 0.0f64;
                    for c in 0..n {
                        if !(part[c].abs() < 1e6) {
                            return None;
                        }
                        st.offs[c] = (part[c] * SCALE).round() as i64;
                        // The offset's rounding to the sums' unit, half a unit.
                        worst = worst.max(fx.err[c] + 0.5 / SCALE + extra_err[c]);
                    }
                    st.margin = ((2.0 * worst + 1e-12) * SCALE * 1.001).ceil() as i64 + 1;
                }
                (Kind::Leaf, Role::Leaf { probit }) => {
                    if !cells {
                        continue;
                    }
                    match (&node.classifier, probit) {
                        (NodeModel::Lr(_), None) => {
                            let mut e = [0.0f64; 8];
                            table?.lr_offsets(&values, &mut e[..3]);
                            if !e[..3].iter().all(|x| x.is_finite()) {
                                return None;
                            }
                            st.diff = e[0] - e[1];
                            // The float logs are S + e, rounded once more: u |S + e| <= u (|S| + |e|), |S| <= abs (1 +
                            // gamma_70). The estimate adds e_0 - e_1, itself rounded (u |e_0 - e_1|, in the 2u |e|).
                            st.abs = [fx.abs[0] + e[0].abs(), fx.abs[1] + e[1].abs(), fx.abs[2] + e[2].abs()];
                            st.err = [0, 1].map(|c| {
                                fx.err[c] + 1.01 * U * (fx.abs[c] * (1.0 + gamma(70)) + e[c].abs()) + 2.0 * U * e[c].abs()
                            });
                        }
                        (NodeModel::Probit(_), Some(_)) => {
                            st.probit = table?.probit_parts(&values);
                        }
                        _ => return None,
                    }
                }
                _ => unreachable!("a node's role follows its kind"),
            }
        }
        Some(Cells { states, index })
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

    /// hybrid18: the value and its bound (0 if exact): the float value is within err of value.
    #[inline]
    pub fn estimate(&self) -> (f64, f64) {
        (self.value, self.err)
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
    /// hybrid15: a DAG with cells' route states (see `Cells`), and per state a logs leaf's bound with its cells
    /// (`state_errs`; NaN for the others). Then a reply's `leaf` is its state.
    cells: Option<Cow<'a, Cells>>,
    state_err: Vec<f64>,
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
        let state_err = tables.cells.as_ref().map_or(Vec::new(), |c| Self::state_errs(c, cal));
        Some(Compound {
            graph,
            cal,
            nodes: Cow::Owned(tables.nodes),
            leaf_err,
            cells: tables.cells.map(Cow::Owned),
            state_err,
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
        let state_err = tables.cells.as_ref().map_or(Vec::new(), |c| Self::state_errs(c, cal));
        Some(Compound {
            graph,
            cal,
            nodes: Cow::Borrowed(&tables.nodes),
            leaf_err,
            cells: tables.cells.as_ref().map(Cow::Borrowed),
            state_err,
            check: fixed::checking(),
            stats: stats_on(),
            scratch: Default::default(),
        })
    }

    /// The constant bounds of logs leaves (see `estimate`): with the float logits' sizes at most their weights'
    /// (|l_c| <= abs_c (1 + gamma_70)), |d| <= abs_0 + abs_1, |m| <= (abs_0 + abs_1) / 2 + abs_2 and |value| <=
    /// |p_0| max |d| + |p_2| + |p_3|.
    fn leaf_errs(nodes: &[Node], cal: Calibration) -> Vec<f64> {
        nodes
            .iter()
            .map(|n| match n.role {
                Role::Leaf { probit: None } => Self::logs_err([n.abs[0], n.abs[1], n.abs[2]], [n.err[0], n.err[1]], cal),
                _ => f64::NAN,
            })
            .collect()
    }

    /// `leaf_errs`'s bound for logs whose sizes are within `abs` and whose lanes 0 and 1 are within `err` of the float
    /// logs.
    fn logs_err(abs: [f64; 3], err: [f64; 2], cal: Calibration) -> f64 {
        let Calibration { slope_winner: p0, slope_decisive: p1, offsets: [p2, p3] } = cal;
        let d = (abs[0] + abs[1]) * (1.0 + 1e-9);
        let m = ((abs[0] + abs[1]) / 2.0 + abs[2]) * (1.0 + 1e-9);
        let v = p0.abs() * d + p2.abs() + p3.abs();
        let rounding = 8.0 * U * (p0.abs() * d + p1.abs() * m + p2.abs() + p3.abs() + v);
        let err = 4.0 * (p0.abs() * (err[0] + err[1] + 2.0 * U * d) + rounding) + 1e-10;
        if err.is_finite() && err < 1e-3 { err } else { f64::INFINITY }
    }

    /// hybrid15: per route state, a logs leaf's bound with its cells (`logs_err` with the state's widened sizes and
    /// distances); NaN for the others (routers, probit leaves, and leaves without cells, which use their node's).
    fn state_errs(cells: &Cells, cal: Calibration) -> Vec<f64> {
        cells
            .states
            .iter()
            .map(|st| match st.next.is_empty() && st.cells && st.abs != [0.0; 3] {
                true => Self::logs_err(st.abs, st.err, cal),
                false => f64::NAN,
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
        // hybrid14: the leaf's classifier, which is what `Graph::node_predict_log` uses for every graph with tables
        // here (none with cells, see `Tables::new`), without path context's descent to find the leaf's path.
        self.graph.nodes[leaf].classifier.predict_log(enc, &mut logs);
        let z = self.cal.logits(&logs);
        if white { z[0] - z[1] } else { z[1] - z[0] }
    }

    /// The float value for the side to move of the position `ch` leads to from `acc`'s.
    pub fn value(&self, acc: &Acc, ch: &Changes, white: bool) -> f64 {
        if let Some(cells) = &self.cells {
            let (st, _) = self.route_cells::<false>(cells, acc, ch);
            return self.exact_state(cells, st, &ch.enc, white);
        }
        let (leaf, _) = self.route::<false>(acc, ch);
        self.exact(leaf, &ch.enc, white)
    }

    /// hybrid15: `route` over route states (a DAG with cells, see `Cells`): the float model's leaf's state for the
    /// position `ch` leads to from `acc`'s, and with `SUMS` the leaf's sums there.
    #[inline]
    fn route_cells<const SUMS: bool>(&self, cells: &Cells, acc: &Acc, ch: &Changes) -> (usize, Sums) {
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
        let mut at = 0;
        loop {
            let st = &cells.states[at];
            let node = &self.nodes[st.node];
            let Role::Router { product, .. } = &node.role else { return (at, leaf_sums(st.node)) };
            let raw = node.table.moved(&acc.sums[st.node], &acc.enc, ch);
            if self.check {
                assert_eq!(raw, node.table.full(&ch.enc), "incremental router sums differ from the full sums");
            }
            self.count(&STATS.decisions);
            let mut s = raw;
            for c in 0..node.classes {
                s[c] += st.offs[c];
            }
            // A product router's float product must stay normal before and after the cells' factor.
            let pick = node.pick(&s, st.margin, *product).filter(|&c| !*product || (raw[c] as f64) >= LOW_LOG * SCALE);
            match pick {
                Some(c) => {
                    if self.check {
                        let mut scores = [0.0; 8];
                        let scores = &mut scores[..node.classes];
                        self.graph.node_predict_path(st.node, &st.marks, &ch.enc, scores, false);
                        assert_eq!(crate::delegation::argmax(scores), c, "router decision with cells");
                    }
                    at = st.next[c] as usize;
                }
                None => {
                    self.count(&STATS.fallbacks);
                    let mut marks = st.marks.clone();
                    let leaf = self.graph.descend_from(st.node, &ch.enc, &mut marks);
                    marks.sort_unstable();
                    let found = cells.index[&(leaf, marks)] as usize;
                    return (found, leaf_sums(leaf));
                }
            }
        }
    }

    /// hybrid15: `exact` for route state `st` (a leaf and its path's marks): the leaf's float log-probabilities with
    /// its cells for that path (`Graph::node_predict_path`), under the whole calibration.
    #[inline]
    fn exact_state(&self, cells: &Cells, st: usize, enc: &Encoded, white: bool) -> f64 {
        let st = &cells.states[st];
        if !st.cells {
            return self.exact(st.node, enc, white);
        }
        let mut logs = [0.0; 3];
        self.graph.node_predict_path(st.node, &st.marks, enc, &mut logs, true);
        let z = self.cal.logits(&logs);
        if white { z[0] - z[1] } else { z[1] - z[0] }
    }

    /// hybrid18: reply `r`'s float value, for the position whose encoding is `enc`: its own if exact.
    #[inline]
    pub fn exact_of(&self, r: &Reply, enc: &Encoded) -> f64 {
        if r.exact { r.value } else { self.exact_reply(r.leaf as usize, enc, r.white) }
    }

    /// A reply's float value: `exact` of its leaf, or of its route state for a DAG with cells.
    #[inline]
    fn exact_reply(&self, leaf: usize, enc: &Encoded, white: bool) -> f64 {
        match &self.cells {
            Some(cells) => self.exact_state(cells, leaf, enc, white),
            None => self.exact(leaf, enc, white),
        }
    }

    /// The position `ch` leads to from `acc`'s as a reply at depth 2: its leaf and estimated value, or its float value
    /// if no estimate is safe.
    #[inline]
    pub fn reply(&self, acc: &Acc, ch: &Changes, white: bool) -> Reply {
        if let Some(cells) = &self.cells {
            // hybrid15: the reply's `leaf` is its route state.
            let (st, s) = self.route_cells::<true>(cells, acc, ch);
            self.count(&STATS.replies);
            return match self.estimate_state(cells, st, &s, white) {
                Some((value, err)) => Reply { value, err, exact: false, white, leaf: st as u32 },
                None => {
                    self.count(&STATS.unestimated);
                    Reply { value: self.exact_state(cells, st, &ch.enc, white), err: 0.0, exact: true, white, leaf: st as u32 }
                }
            };
        }
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
        let Calibration { slope_winner: p0, offsets: [p2, p3], .. } = self.cal;
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
        self.estimate_probit(node, (mean, 0.0), (var, 0.0), eps, white)
    }

    /// hybrid15: `estimate` for route state `st` of a DAG with cells: its node's, with the cells' part for the path.
    #[inline]
    fn estimate_state(&self, cells: &Cells, st: usize, s: &Sums, white: bool) -> Option<(f64, f64)> {
        let state = &cells.states[st];
        if !state.cells {
            return self.estimate(state.node, s, white);
        }
        let Calibration { slope_winner: p0, offsets: [p2, p3], .. } = self.cal;
        let node = &self.nodes[state.node];
        let Role::Leaf { probit } = node.role else { unreachable!("routing ends at a leaf") };
        let Some((eps, beta2)) = probit else {
            // Logs: the state's constant bound (`state_errs`).
            let err = self.state_err[st];
            if !err.is_finite() {
                return None;
            }
            let v = p0 * ((s[0] - s[1]) as f64 / SCALE + state.diff) + (p2 - p3);
            return Some((if white { v } else { -v }, err));
        };
        // The float latent is (m + mu, var + s2): one more rounding of each (u |m + mu|, u (var + s2)); the estimate's
        // own additions round as many times.
        let (mu, s2) = state.probit;
        let mean = s[0] as f64 / SCALE + mu;
        let var = s[1] as f64 / SCALE + beta2 + s2;
        let extra = (2.0 * U * (mean.abs() + mu.abs()), 2.0 * U * (var + s2.abs()));
        self.estimate_probit(node, (mean, extra.0), (var, extra.1), eps, white)
    }

    /// `estimate`'s probit part: the leaf's latent mean and variance (beta^2 included), each with an extra bound
    /// beyond the node's own (`err` and the roundings of `estimate`).
    #[inline]
    fn estimate_probit(&self, node: &Node, (mean, mean_extra): (f64, f64), (var, var_extra): (f64, f64), eps: f64, white: bool) -> Option<(f64, f64)> {
        let Calibration { slope_winner: p0, slope_decisive: p1, offsets: [p2, p3] } = self.cal;
        if !(var > 1e-12 && var.is_finite() && mean.is_finite()) {
            return None;
        }
        let sd = var.sqrt();
        let (zw, zb) = ((mean - eps) / sd, (-mean - eps) / sd);
        if !(zw.abs() < 30.0 && zb.abs() < 30.0 && eps / sd > 1e-6) {
            return None;
        }
        // hybrid9: ln Phi from hybrid8's table (`fastprobit::ln_phi`) where it holds (|z| <= 29.5), unless
        // CHESSAI_FX_CTABLE=off. Its bound ew covers the table value's distance from the float ln Phi at the same z, and
        // so from the true ln Phi there; the float value's own ln Phi, at its own z, is allowed 1e-13 + 16u |ln Phi|
        // as before, with |ln Phi| there under |l| + 1 (the z's differ by less than the derivative terms, which an
        // estimate in use keeps far below 1). Outside the table, hybrid8's rational ln Phi and its allowance.
        let in_table = table_on() && zw.abs() <= crate::fastprobit::Z_MAX && zb.abs() <= crate::fastprobit::Z_MAX;
        let (lw, lb, approx) = if in_table {
            let t = crate::fastprobit::table();
            let ((lw, ew), (lb, eb)) = (crate::fastprobit::ln_phi(t, zw), crate::fastprobit::ln_phi(t, zb));
            (lw, lb, ew + eb + 2e-13 + 16.0 * U * (lw.abs() + lb.abs() + 2.0))
        } else {
            let (lw, lb) = (crate::probit::ln_cdf_first(zw), crate::probit::ln_cdf_first(zb));
            (lw, lb, 2.0 * (2e-13 + 16.0 * U * (lw.abs() + lb.abs())))
        };
        // The inverse Mills ratio phi(z) / Phi(z) is at most |z| + 1: d ln Phi(z) / dz.
        let (rw, rb) = (zw.abs() + 1.0, zb.abs() + 1.0);
        let (mean_err, var_err) = (node.err[0] + U * mean.abs() + mean_extra, node.err[1] + 2.0 * U * var + var_extra);
        let derivs = 2.0 * ((rw + rb) / sd * mean_err + (rw * zw.abs() + rb * zb.abs()) / (2.0 * var) * var_err);
        let z_rounding = 2.0 * 4.0 * U * (rw * zw.abs() + rb * zb.abs());
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
                let exact = self.exact_reply(r.leaf as usize, &enc_of(i), r.white);
                assert!(exact.is_nan() || (exact - r.value).abs() <= r.err, "value {exact} outside {} +- {}", r.value, r.err);
            }
        }
        let mut close = 0u64;
        for (i, r) in replies.iter_mut().enumerate() {
            if !r.exact && r.value - r.err <= upper {
                r.value = self.exact_reply(r.leaf as usize, &enc_of(i), r.white);
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
                r.value = self.exact_reply(r.leaf as usize, &enc_of(i), r.white);
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

/// Whether probit leaves' reply estimates use hybrid8's ln Phi table (CHESSAI_FX_CTABLE, default on; off gives
/// hybrid8's estimate). Either way only the choice of replies valued in float changes, never a value.
pub fn table_on() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| match std::env::var("CHESSAI_FX_CTABLE").as_deref() {
        Ok("off") => false,
        Ok("on") | Err(_) => true,
        Ok(other) => panic!("CHESSAI_FX_CTABLE must be on or off, not {other:?}"),
    })
}

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
            subrouter_target: crate::delegation::SubrouterTarget::Routing,
            path_context: false,
            router_split_z: None,
            router_split_limit: None,
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

    /// hybrid15: a DAG with path context trained on random positions until router splits gave its tables cells.
    fn dag_with_cells(leaf: NodeKind, seed: u64) -> Graph {
        let params = DelegationParams {
            leaf,
            router: NodeKind::Cnbp,
            error: ErrorMeasure::Log,
            split_threshold: 150,
            split_limit: 6,
            forget_factor: 1.0,
            split_noise: if leaf == NodeKind::Probit { 0.05 } else { 0.0 },
            path_context: true,
            router_split_z: Some(0.0),
            router_split_limit: Some(3),
            ..params()
        };
        let mut bdd = crate::bdd::Bdd::new(&CATEGORIES, params);
        let data = positions(seed, 40_000);
        for (i, (x, y)) in data.iter().enumerate() {
            bdd.train(x, y);
            if i % 1000 == 999 {
                bdd.refresh();
                if bdd.path_param_count().1 >= 2 && i >= 20_000 {
                    break;
                }
            }
        }
        bdd.refresh();
        let graph = bdd.graph().clone();
        assert!(graph.has_cells(), "no router split made cells (sets {})", graph.sets.len());
        graph
    }

    #[test]
    fn dags_with_cells_value_exactly() {
        for (leaf, seed) in [(NodeKind::Lr, 41), (NodeKind::Probit, 42)] {
            let model = CalGraph { graph: dag_with_cells(leaf, seed), cal: CAL };
            let c = model.compound_play().expect("a DAG with cells plays in fixed point");
            assert!(c.cells.is_some(), "route states");
            same_values(&model, seed, 3, 40);
        }
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
            let (mut n, mut worst, mut tabled) = (0, 0.0f64, 0);
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
                            // Probit leaves' estimates use the ln Phi table (hybrid9), whose bound allows h^2/16
                            // (about 1e-6) per ln Phi; the other leaves' bounds stay under 1e-6.
                            assert!(err < 1e-4, "bound {err}");
                            tabled += (err >= 1e-6) as usize;
                            worst = worst.max((exact - v).abs() / err);
                            n += 1;
                        }
                    }
                    pos.play_unchecked(moves[rng.random_range(0..moves.len())]);
                }
            }
            assert!(n > 5000, "{n} estimates");
            assert!(worst < 0.5, "an error used {worst} of its bound");
            // Both models have a probit leaf, so the table is in use (unless CHESSAI_FX_CTABLE=off).
            assert!(!table_on() || tabled > 100, "{tabled} estimates from the ln Phi table");
        }
    }

    /// hybrid18: positions labelled mostly by material (so captures refute), for leaves that value something.
    fn material_positions(seed: u64, n: usize) -> Vec<(Encoded, [f64; 3])> {
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
            let white = pos.board().white().count() as i32 - pos.board().black().count() as i32;
            let c = if white > 0 && rng.random_range(0..4) > 0 { 0 } else if white < 0 && rng.random_range(0..4) > 0 { 1 } else { rng.random_range(0..3) };
            label[c] = 1.0;
            out.push((encoding::encode(&pos), label));
        }
        out
    }

    fn material_probit(seed: u64) -> NodeModel {
        let mut m = Probit::new(&CATEGORIES, ProbitParams::default());
        for (x, y) in material_positions(seed, 6000) {
            m.train(&x, &y);
        }
        NodeModel::Probit(m)
    }

    fn material_lr(seed: u64) -> NodeModel {
        let mut m = Logreg::new(3, &CATEGORIES, 0.1);
        for (x, y) in material_positions(seed, 6000) {
            m.train(&x, &y);
        }
        NodeModel::Lr(m)
    }

    /// hybrid18: positions of random games (every `every` plies; some games shuffle pieces, so repetitions are
    /// reachable): at depths 3 to `max_depth` the compound search's root values equal the float negamax's, bit for
    /// bit; with root cut windows (T 0.02's and the argmax's) it cuts the same moves with the same exact values and
    /// valid bounds; `root_value_of` gives the float values. Returns the cases.
    fn same_values_deep(model: &CalGraph, seed: u64, games: usize, plies: usize, every: usize, max_depth: u32) -> u64 {
        let c = model.compound_play().expect("compound model");
        let mut rng = StdRng::seed_from_u64(seed);
        let bits = |v: &[f64]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
        let mut cases = 0;
        for g in 0..games {
            let mut game = Game::default();
            for i in 0..plies {
                if game.result().is_some() {
                    break;
                }
                if i % every == every - 1 {
                    for depth in 3..=(if i % (3 * every) == 3 * every - 1 { max_depth } else { max_depth.min(4) }) {
                        let float = Search::new(model, &game).root_values(&game, depth);
                        let deep = Search::new(model, &game).with_compound(Some(&c)).root_values(&game, depth);
                        assert_eq!(bits(&deep), bits(&float), "game {g} ply {i} depth {depth}: {deep:?} vs {float:?}");
                        let best = float.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                        let windows: [&dyn Fn(f64) -> f64; 2] = [&|b: f64| b - 40.0 * 0.02, &f64::next_down];
                        for below in windows {
                            let (fv, fp) = Search::new(model, &game).root_values_windowed(&game, depth, below);
                            let (cv, cp) = Search::new(model, &game).with_compound(Some(&c)).root_values_windowed(&game, depth, below);
                            assert_eq!(cp, fp, "game {g} ply {i} depth {depth}: cuts");
                            for k in 0..cv.len() {
                                if cp[k] {
                                    assert!(float[k] <= cv[k] && cv[k] < best, "move {k}: bound {} value {}", cv[k], float[k]);
                                } else {
                                    assert_eq!(cv[k].to_bits(), fv[k].to_bits(), "move {k}");
                                    assert_eq!(cv[k].to_bits(), float[k].to_bits(), "move {k}");
                                }
                            }
                        }
                        let k = rng.random_range(0..float.len());
                        let v = Search::new(model, &game).with_compound(Some(&c)).root_value_of(&game, depth, k);
                        assert_eq!(v.to_bits(), float[k].to_bits(), "root_value_of {k}");
                        cases += 1;
                    }
                }
                let moves = game.position().legal_moves();
                let quiet: Vec<_> = moves.iter().copied().filter(|m| !m.is_capture() && m.role() != shakmaty::Role::Pawn).collect();
                let m = if g % 2 == 1 && i > 12 && !quiet.is_empty() { quiet[rng.random_range(0..quiet.len())] } else { moves[rng.random_range(0..moves.len())] };
                game.play(m);
            }
        }
        cases
    }

    #[test]
    fn deep_searches_value_exactly() {
        // hybrid18: trees (LR, CNBP and TAN routers; LR, probit and CNBP leaves, some trained on material so captures
        // refute), a DAG, and DAGs with cells, at depths 3-5.
        let before = crate::search::DEEP_STATS.leaves.load(Relaxed);
        let mut cases = 0;
        let models = [
            CalGraph { graph: tree(lr(51, 0.3), [material_lr(52), material_probit(53), lr(54, 0.2)]), cal: CAL },
            CalGraph { graph: tree(cnbp(55), [material_probit(56), cnbp(57), material_lr(58)]), cal: CAL },
            CalGraph { graph: tree(tan(59), [material_lr(60), probit(61), tan(62)]), cal: Calibration::NONE },
        ];
        for (k, model) in models.iter().enumerate() {
            cases += same_values_deep(model, 60 + k as u64, 4, 70, 9, 5);
        }
        let mut graph = Graph::new(&CATEGORIES, params());
        let mut r2 = Logreg::new(2, &CATEGORIES, 0.1);
        r2.randomize(63, 0.3);
        graph.nodes = vec![
            crate::delegation::Node::new(Kind::Branch { children: vec![1, 2, 3] }, lr(64, 0.3)),
            crate::delegation::Node::new(Kind::Branch { children: vec![3, 4] }, NodeModel::Lr(r2)),
            crate::delegation::Node::new(Kind::Leaf, material_lr(65)),
            crate::delegation::Node::new(Kind::Leaf, material_probit(66)),
            crate::delegation::Node::new(Kind::Leaf, cnbp(67)),
        ];
        cases += same_values_deep(&CalGraph { graph, cal: CAL }, 68, 3, 60, 9, 4);
        for (leaf, seed) in [(NodeKind::Lr, 41), (NodeKind::Probit, 42)] {
            let model = CalGraph { graph: dag_with_cells(leaf, seed), cal: CAL };
            assert!(model.compound_play().unwrap().cells.is_some());
            cases += same_values_deep(&model, seed, 2, 60, 10, 4);
        }
        assert!(cases > 60, "{cases} cases");
        if stats_on() {
            assert!(crate::search::DEEP_STATS.leaves.load(Relaxed) > before);
        }
        eprintln!("deep_searches_value_exactly: {cases} cases");
    }

    /// hybrid20: a full ternary tree breadth-first, `routers` LR routers (node i routes to 3i+1..3i+3) over
    /// 2 * routers + 1 leaves, some of them trained on material (probit and LR) so captures refute.
    fn full_tree(routers: usize, seed: u64, trained: bool) -> Graph {
        let mut graph = Graph::new(&CATEGORIES, params());
        let n = 3 * routers + 1;
        graph.nodes = (0..n)
            .map(|i| {
                let s = seed + i as u64;
                if i < routers {
                    return crate::delegation::Node::new(Kind::Branch { children: vec![3 * i + 1, 3 * i + 2, 3 * i + 3] }, lr(s, 0.3));
                }
                let leaf = match (trained, i % 9) {
                    (true, 0) => material_probit(s),
                    (true, 4) => material_lr(s),
                    _ => lr(s, 0.2),
                };
                crate::delegation::Node::new(Kind::Leaf, leaf)
            })
            .collect();
        graph
    }

    #[test]
    fn graphs_up_to_the_cap_value_exactly() {
        // hybrid20: 121 nodes (40 routers, 81 leaves) is over hybrid19's cap (64) and under hybrid20's (128): it takes
        // the compound path, and its values equal the float search's at depths 1-2 and 3-4, bit for bit.
        let model = CalGraph { graph: full_tree(40, 900, true), cal: CAL };
        assert_eq!(model.graph.nodes.len(), 121);
        assert!(model.graph.nodes.len() > 64 && model.graph.nodes.len() <= MAX_NODES);
        assert!(model.compound_play().is_some(), "121 nodes take the compound path");
        same_values(&model, 901, 2, 40);
        let cases = same_values_deep(&model, 902, 2, 40, 9, 4);
        assert!(cases > 4, "{cases} cases");
        // The cap itself: a tree of 127 nodes (the largest ternary tree under 128) takes the path, one of 130 doesn't.
        assert!(Tables::new(&full_tree(42, 1000, false)).is_some(), "127 nodes");
        assert!(Tables::new(&full_tree(43, 1100, false)).is_none(), "130 nodes");
    }

    #[test]
    fn dags_past_4096_route_states_value_exactly() {
        // hybrid21: a real DAG past hybrid20's state cap (4,096): std-v1-p2-s2's D-P-Ne-ctx-grow v100 (69 nodes, CNBP
        // routers, probit leaves, path cells; testdata/dp6601.model, md5 21183511...). It takes the compound path
        // with its route states, and its values equal the float search's at depths 1-2, bit for bit.
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/dp6601.model");
        let graph = crate::bdd::Bdd::load(&path, true).expect("testdata/dp6601.model").graph().clone();
        assert!(graph.has_cells(), "a DAG with cells");
        let model = CalGraph { graph, cal: CAL };
        let c = model.compound_play().expect("past 4,096 route states takes the compound path");
        let n = c.cells.as_ref().expect("route states").states.len();
        assert!(n > 4096 && n <= MAX_STATES, "{n} route states");
        same_values(&model, 2101, 2, 30);
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
