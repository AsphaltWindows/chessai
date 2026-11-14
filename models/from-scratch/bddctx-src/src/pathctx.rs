//! Path context for delegation DAGs (`DelegationParams::path_context`, BDD only): inputs that tell a node which path
//! delivered a position, so that a node shared by several paths can treat them differently.
//!
//! - A sibling set is the `branch_factor` copies that one router split makes. The graph keeps a registry of them:
//!   each has a permanent index (the order of the splits) and its members' node ids. Leaf splits make none, since
//!   whatever is below a leaf split is reached through one child only.
//! - Every node has a path table with one cell per sibling set upstream of it, that is, a set with a member that is
//!   one of the node's ancestors (not the node itself). The cells are in registry order. A cell is a categorical
//!   input with `branch_factor + 1` values: 0 if the position's path did not pass through the set, otherwise the
//!   number (from 1) of the member it passed through. A path passes through at most one member of a set, since the
//!   members are siblings. The values at a node give the whole path from the root to it (see `decode`).
//! - When a member splits (P2 of S1 = {P1, P2, P3}, say), it stays in its place in S1 as the new central router: same
//!   node id, same sibling number. Registry entries never change after they are made. Its copies form a new set S2
//!   below it. The nodes below the copies get an S2 cell; P2 gets none (S2's members are its children) and neither
//!   do the copies (a set's members are not their own ancestors). The copies do have an S1 cell, since P2 is their
//!   parent; it always reads 2 for them. A leaf below both sets reads S1=1, S2=0 through P1; S1=2, S2=2 through P2
//!   and its second copy; S1=3, S2=0 through P3.
//! - The parameters are per (cell, value, class). A logistic regression has weights with AdaGrad sums, added to its
//!   logits like any categorical feature's. A CNBP has counts, smoothed like the board features' (alpha); each cell
//!   is normalised by its own per-class totals, which start when the cell does.
//! - A router split appends its set to the registry. Every node below the copies appends a cell at neutral values:
//!   weights 0, or no counts. The copies start from the splitting router's table, plus a neutral cell for the set
//!   the router is a member of, if it is one. The central router keeps its own table with home targets; with
//!   routing targets its router starts afresh, and so does its table, with the same cells. A leaf split gives each
//!   new leaf the leaf's table, and the new router the same cells, empty. So no prediction changes at a split.
//!
//! A CNBP cell's factor P(value | class) is used divided by 1 / (branch_factor + 1), which is the same for every
//! class and so leaves the posterior as it is; a cell without counts then multiplies every class by exactly 1, and
//! its appearance changes no bit of any prediction.

use std::fmt::Write as _;
use std::io;

use crate::delegation::{Graph, Kind, NodeModel};
use crate::state::{Reader, Writer};
use crate::textfmt::{Tokens, invalid};

/// A sibling set a position's path passed through: the set's registry index and the member's number (from 1).
pub type Mark = (usize, usize);

/// The parameters of a path table, by (cell, value, class).
#[derive(Clone, Debug, PartialEq)]
pub enum Params {
    /// A logistic regression's weights and their AdaGrad sums.
    Lr { weights: Vec<f64>, squares: Vec<f64> },
    /// A CNBP's counts, and each cell's per-class totals (by cell, class).
    Cnbp { counts: Vec<f64>, totals: Vec<f64> },
}

/// A node's path table (see the module's notes).
#[derive(Clone, Debug, PartialEq)]
pub struct PathTable {
    /// The cells: the registry indices of the sibling sets upstream of the node, in registry order.
    cells: Vec<usize>,
    /// Values per cell: `branch_factor + 1`.
    values: usize,
    class_num: usize,
    params: Params,
}

impl PathTable {
    /// A table without cells, for a node whose classifier is a logistic regression (`lr`) or a CNBP.
    pub fn new(lr: bool, class_num: usize, values: usize) -> Self {
        let params = match lr {
            true => Params::Lr { weights: Vec::new(), squares: Vec::new() },
            false => Params::Cnbp { counts: Vec::new(), totals: Vec::new() },
        };
        PathTable { cells: Vec::new(), values, class_num, params }
    }

    /// A table with this one's cells and no counts or weights, for a node with a new classifier.
    pub fn blank(&self, lr: bool, class_num: usize) -> Self {
        let mut table = PathTable::new(lr, class_num, self.values);
        self.cells.iter().for_each(|&set| table.append_cell(set));
        table
    }

    pub fn cells(&self) -> &[usize] {
        &self.cells
    }

    pub fn is_lr(&self) -> bool {
        matches!(self.params, Params::Lr { .. })
    }

    pub fn class_num(&self) -> usize {
        self.class_num
    }

    /// Appends a cell for sibling set `set`, newer than all the table's, at neutral values.
    pub fn append_cell(&mut self, set: usize) {
        assert!(self.cells.last().is_none_or(|&last| last < set), "cells go in registry order");
        self.insert_cell(set);
    }

    /// Adds a cell for sibling set `set` in its place in registry order, at neutral values: weights 0, or no counts.
    pub fn insert_cell(&mut self, set: usize) {
        let cell = self.cells.partition_point(|&s| s < set);
        assert!(self.cells.get(cell) != Some(&set), "a table has one cell per set");
        self.cells.insert(cell, set);
        let (n, at) = (self.values * self.class_num, cell * self.values * self.class_num);
        let zeros = |v: &mut Vec<f64>, at: usize, n: usize| drop(v.splice(at..at, std::iter::repeat_n(0.0, n)));
        match &mut self.params {
            Params::Lr { weights, squares } => {
                zeros(weights, at, n);
                zeros(squares, at, n);
            }
            Params::Cnbp { counts, totals } => {
                zeros(counts, at, n);
                zeros(totals, cell * self.class_num, self.class_num);
            }
        }
    }

    /// Each cell's value for a path that passed through the sibling sets `marks`.
    pub fn values_for(&self, marks: &[Mark], out: &mut Vec<usize>) {
        out.clear();
        out.extend(self.cells.iter().map(|&set| marks.iter().find(|m| m.0 == set).map_or(0, |m| m.1)));
    }

    fn index(&self, cell: usize, value: usize, class: usize) -> usize {
        (cell * self.values + value) * self.class_num + class
    }

    /// Logistic regression: the sum of the cells' weights for the values `values`, per class, into `out`.
    pub fn lr_offsets(&self, values: &[usize], out: &mut [f64]) {
        let Params::Lr { weights, .. } = &self.params else { unreachable!("a CNBP table has no weights") };
        let out = &mut out[..self.class_num];
        out.iter_mut().for_each(|x| *x = 0.0);
        for (cell, &value) in values.iter().enumerate() {
            let i = self.index(cell, value, 0);
            out.iter_mut().zip(&weights[i..i + self.class_num]).for_each(|(o, w)| *o += w);
        }
    }

    /// CNBP: per class, the product over the cells of P(value | class) / (1 / values), smoothed by `alpha`.
    pub fn cnbp_factors(&self, values: &[usize], alpha: f64, out: &mut [f64]) {
        let Params::Cnbp { counts, totals } = &self.params else { unreachable!("a logistic regression's table has no counts") };
        let out = &mut out[..self.class_num];
        out.iter_mut().for_each(|x| *x = 1.0);
        let v = self.values as f64;
        for (cell, &value) in values.iter().enumerate() {
            let i = self.index(cell, value, 0);
            for (c, o) in out.iter_mut().enumerate() {
                *o *= (counts[i + c] + alpha) * v / (totals[cell * self.class_num + c] + v * alpha);
            }
        }
    }

    /// One AdaGrad step at step size `rate` on the log-loss, for the cells' weights of the values `values`, given the
    /// node's prediction `p` with them (as `Logreg::step` for the node's own weights).
    pub fn lr_step(&mut self, values: &[usize], p: &[f64], label: &[f64], rate: f64) {
        let (n, v) = (self.class_num, self.values);
        let Params::Lr { weights, squares } = &mut self.params else { unreachable!("a CNBP table has no weights") };
        for c in 0..n {
            let g = p[c] - label[c];
            if g == 0.0 {
                continue;
            }
            for (cell, &value) in values.iter().enumerate() {
                let i = (cell * v + value) * n + c;
                squares[i] += g * g;
                weights[i] -= rate * g / squares[i].sqrt();
            }
        }
    }

    /// CNBP: counts an example with the values `values`, as `Cnbp::train` counts its features.
    pub fn cnbp_add(&mut self, values: &[usize], label: &[f64]) {
        let (n, v) = (self.class_num, self.values);
        let Params::Cnbp { counts, totals } = &mut self.params else { unreachable!("a logistic regression's table has no counts") };
        for (cell, &value) in values.iter().enumerate() {
            for (c, &l) in label.iter().enumerate().take(n) {
                counts[(cell * v + value) * n + c] += l;
                totals[cell * n + c] += l;
            }
        }
    }

    /// As the node's classifier forgets (see `NodeModel::forget`): a logistic regression's AdaGrad sums, or a CNBP's
    /// counts and totals, divided by `factor`.
    pub fn forget(&mut self, factor: f64) {
        match &mut self.params {
            Params::Lr { squares, .. } => squares.iter_mut().for_each(|s| *s /= factor),
            Params::Cnbp { counts, totals } => counts.iter_mut().chain(totals.iter_mut()).for_each(|s| *s /= factor),
        }
    }

    /// Weights or counts, one per (cell, value, class); AdaGrad sums and totals not counted.
    pub fn param_count(&self) -> usize {
        self.cells.len() * self.values * self.class_num
    }

    /// The weights or counts, by (cell, value, class).
    pub fn weights(&self) -> &[f64] {
        match &self.params {
            Params::Lr { weights, .. } => weights,
            Params::Cnbp { counts, .. } => counts,
        }
    }

    /// Model file format: a header with the cells' set indices, then the parameters at full precision (weights and
    /// AdaGrad sums, or counts and totals), so a reloaded model trains on exactly as before.
    pub fn write_text(&self, s: &mut String) {
        let kind = if self.is_lr() { "lr" } else { "cnbp" };
        write!(s, "pathtable {kind} {} {} {}", self.class_num, self.values, self.cells.len()).unwrap();
        self.cells.iter().for_each(|set| write!(s, " {set}").unwrap());
        s.push('\n');
        let (a, b) = match &self.params {
            Params::Lr { weights, squares } => (weights, squares),
            Params::Cnbp { counts, totals } => (counts, totals),
        };
        for x in a.iter().chain(b) {
            writeln!(s, "{x:e}").unwrap();
        }
    }

    pub fn read_text(t: &mut Tokens) -> io::Result<Self> {
        if t.next::<String>()? != "pathtable" {
            return Err(invalid("expected a node's path table"));
        }
        let lr = match t.next::<String>()?.as_str() {
            "lr" => true,
            "cnbp" => false,
            other => return Err(invalid(format!("unknown path table kind {other:?}"))),
        };
        let (class_num, values, cell_num): (usize, usize, usize) = (t.next()?, t.next()?, t.next()?);
        let cells = (0..cell_num).map(|_| t.next()).collect::<io::Result<Vec<usize>>>()?;
        let mut table = PathTable::new(lr, class_num, values);
        for &set in &cells {
            if table.cells.last().is_some_and(|&last| last >= set) {
                return Err(invalid("a path table's cells are not in registry order"));
            }
            table.append_cell(set);
        }
        match &mut table.params {
            Params::Lr { weights: a, squares: b } | Params::Cnbp { counts: a, totals: b } => {
                for x in a.iter_mut().chain(b.iter_mut()) {
                    *x = t.next()?;
                }
            }
        }
        Ok(table)
    }

    pub fn write_state(&self, w: &mut Writer) {
        w.bool(self.is_lr());
        w.usize(self.class_num);
        w.usize(self.values);
        w.usizes(&self.cells);
        match &self.params {
            Params::Lr { weights: a, squares: b } | Params::Cnbp { counts: a, totals: b } => {
                w.f64s(a);
                w.f64s(b);
            }
        }
    }

    pub fn read_state(r: &mut Reader) -> io::Result<Self> {
        let lr = r.bool()?;
        let (class_num, values) = (r.usize()?, r.usize()?);
        let cells = r.usizes()?;
        if cells.windows(2).any(|w| w[0] >= w[1]) {
            return Err(invalid("a path table's cells are not in registry order"));
        }
        let mut table = PathTable::new(lr, class_num, values);
        cells.iter().for_each(|&set| table.append_cell(set));
        let (a, b) = (r.f64s()?, r.f64s()?);
        match &mut table.params {
            Params::Lr { weights: x, squares: y } | Params::Cnbp { counts: x, totals: y } => {
                if a.len() != x.len() || b.len() != y.len() {
                    return Err(invalid("a path table of the wrong shape in the state"));
                }
                (*x, *y) = (a, b);
            }
        }
        Ok(table)
    }
}

/// A new node's path table for its classifier: no cells yet.
pub(crate) fn table_for(g: &Graph, classifier: &NodeModel) -> PathTable {
    PathTable::new(matches!(classifier, NodeModel::Lr(_)), classifier.class_num(), g.params.branch_factor + 1)
}

/// After leaf `idx` split into the new leaves `made`: each new leaf starts with the leaf's table, forgotten as the
/// leaf's classifier is (see `Graph::split_leaf`), and the new router gets the same cells, empty.
pub(crate) fn after_leaf_split(g: &mut Graph, idx: usize, made: &[usize]) {
    let table = *g.nodes[idx].table.take().expect("with path context every node has a path table");
    for &m in made {
        let mut copy = table.clone();
        copy.forget(g.params.forget_factor);
        g.nodes[m].table = Some(Box::new(copy));
    }
    let router = &g.nodes[idx].classifier;
    g.nodes[idx].table = Some(Box::new(table.blank(matches!(router, NodeModel::Lr(_)), router.class_num())));
}

/// After router `idx` split into the copies `made` (see `Bdd::split_delegator`), whose classifiers are copies of its
/// old router forgotten by the forget factor: registers their sibling set; gives the copies the router's table,
/// forgotten likewise, plus a neutral cell for the set the router is a member of (the router is their parent); starts
/// the central router's table afresh if its router did (routing targets), with the same cells; and gives every node
/// below the copies a neutral cell for the new set. The router keeps its node id, and its place in its own set.
pub(crate) fn after_router_split(g: &mut Graph, idx: usize, made: &[usize], fresh_router: bool) {
    let set = g.sets.len();
    g.sets.push(made.to_vec());
    let table = *g.nodes[idx].table.take().expect("with path context every node has a path table");
    let mut copy = table.clone();
    if let Some((own, _)) = g.nodes[idx].member {
        copy.insert_cell(own);
    }
    copy.forget(g.params.forget_factor);
    for (j, &m) in made.iter().enumerate() {
        g.nodes[m].table = Some(Box::new(copy.clone()));
        g.nodes[m].member = Some((set, j + 1));
    }
    let central = &g.nodes[idx].classifier;
    g.nodes[idx].table = Some(Box::new(match fresh_router {
        true => table.blank(matches!(central, NodeModel::Lr(_)), central.class_num()),
        false => table,
    }));
    let children: Vec<usize> = made.iter().flat_map(|&m| g.children(m).to_vec()).collect();
    for node in below(g, &children) {
        let n = &mut g.nodes[node];
        n.table.as_mut().expect("with path context every node has a path table").append_cell(set);
        n.cache_step = 0;
    }
}

/// The nodes `start` and every node below them, each once, in increasing order.
fn below(g: &Graph, start: &[usize]) -> Vec<usize> {
    let mut seen = vec![false; g.nodes.len()];
    let mut stack = start.to_vec();
    while let Some(x) = stack.pop() {
        if !std::mem::replace(&mut seen[x], true) {
            stack.extend(g.children(x));
        }
    }
    (0..g.nodes.len()).filter(|&x| seen[x]).collect()
}

/// For each node, the registry indices of the sibling sets upstream of it (with a member that is one of its
/// ancestors), in registry order, worked out from the structure alone.
pub(crate) fn upstream_sets(g: &Graph) -> Vec<Vec<usize>> {
    let mut upstream = vec![Vec::new(); g.nodes.len()];
    for (set, members) in g.sets.iter().enumerate() {
        let children: Vec<usize> = members.iter().flat_map(|&m| g.children(m).to_vec()).collect();
        below(g, &children).into_iter().for_each(|node| upstream[node].push(set));
    }
    upstream
}

/// The node sequence from the root to node `idx` that the values `values` of its cells describe: from the node up,
/// a node with one parent came from it, and a node with several (the members of sibling sets that share it) came
/// from the one whose set's value names it. Errors if the values do not describe a path.
pub(crate) fn decode(g: &Graph, idx: usize, values: &[usize]) -> Result<Vec<usize>, String> {
    let table = g.nodes[idx].table.as_ref().ok_or("the node has no path table")?;
    let value_of = |set: usize| table.cells.iter().position(|&s| s == set).map(|cell| values[cell]);
    let mut parents = vec![Vec::new(); g.nodes.len()];
    for (p, node) in g.nodes.iter().enumerate() {
        if let Kind::Branch { children } = &node.kind {
            children.iter().for_each(|&c| parents[c].push(p));
        }
    }
    let mut sequence = vec![idx];
    let mut x = idx;
    while x != 0 {
        let from: Vec<usize> = match parents[x].as_slice() {
            [single] => vec![*single],
            several => several
                .iter()
                .copied()
                .filter(|&p| g.nodes[p].member.is_some_and(|(set, member)| value_of(set) == Some(member)))
                .collect(),
        };
        let [p] = from.as_slice() else {
            return Err(format!("node {x} has {} parents that the values name", from.len()));
        };
        sequence.push(*p);
        x = *p;
        if sequence.len() > g.nodes.len() {
            return Err("the parents form a cycle".to_string());
        }
    }
    sequence.reverse();
    Ok(sequence)
}

/// Checks a graph read from a file: the registry's sets have `branch_factor` distinct members each, which are routers
/// and members of no other set; every node has a table of its classifier's kind and class count; and its cells are
/// exactly the sets upstream of it.
pub(crate) fn check(g: &Graph) -> io::Result<()> {
    let mut member = vec![false; g.nodes.len()];
    for (set, members) in g.sets.iter().enumerate() {
        if members.len() != g.params.branch_factor {
            return Err(invalid(format!("sibling set {set} has {} members", members.len())));
        }
        for &m in members {
            if m >= g.nodes.len() || std::mem::replace(&mut member[m], true) {
                return Err(invalid(format!("sibling set {set} has a member out of range or in another set")));
            }
            if !matches!(g.nodes[m].kind, Kind::Branch { .. }) {
                return Err(invalid(format!("sibling set {set} has a member that is not a router")));
            }
        }
    }
    let upstream = upstream_sets(g);
    for (idx, node) in g.nodes.iter().enumerate() {
        let Some(table) = &node.table else { return Err(invalid(format!("node {idx} has no path table"))) };
        let lr = matches!(node.classifier, NodeModel::Lr(_));
        if table.is_lr() != lr || table.class_num != node.classifier.class_num() || table.values != g.params.branch_factor + 1 {
            return Err(invalid(format!("node {idx}'s path table does not fit its classifier")));
        }
        if table.cells != upstream[idx] {
            return Err(invalid(format!("node {idx}'s path table has the cells {:?}, not the sets upstream of it {:?}", table.cells, upstream[idx])));
        }
    }
    Ok(())
}
