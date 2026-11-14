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
//!   is normalised by its own per-class totals, which start when the cell does. A probit (errdag3) has one latent for
//!   all three results, so its parameters are per (cell, value): Gaussian weights (mean, variance) added to its
//!   latent like any feature's, learning with its own weights (`Probit::train_with_cells`). A new cell's weights
//!   have mean 0 and variance split_noise^2 (the analyst's 6.25 addendum), so a split changes no latent mean; the
//!   variance makes the leaf's predictions very slightly less confident.
//! - A router split appends its set to the registry. Every node below the copies appends a cell at neutral values:
//!   weights 0, or no counts (probit: means 0, variances split_noise^2). The copies start from the splitting router's table, plus a neutral cell for the set
//!   the router is a member of, if it is one. The central router keeps its own table with home targets; with
//!   routing targets its router starts afresh, and so does its table, with the same cells. A leaf split gives each
//!   new leaf the leaf's table, and the new router the same cells, empty. So no prediction changes at a split. (A
//!   probit leaf's copies are widened by split_noise^2, their cells too, as `Graph::split_leaf` widens their weights.)
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
    /// A probit's cell weights' means and variances (by cell, value: the table's class count is 1), and the variance
    /// a new cell's weights start with.
    Probit { means: Vec<f64>, vars: Vec<f64>, init_var: f64 },
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

    /// A table without cells for a probit node, whose new cells' weights start at mean 0 and variance `init_var`.
    pub fn new_probit(values: usize, init_var: f64) -> Self {
        PathTable { cells: Vec::new(), values, class_num: 1, params: Params::Probit { means: Vec::new(), vars: Vec::new(), init_var } }
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

    pub fn is_probit(&self) -> bool {
        matches!(self.params, Params::Probit { .. })
    }

    pub fn class_num(&self) -> usize {
        self.class_num
    }

    /// grow6: the values per cell (`branch_factor + 1`).
    pub fn value_count(&self) -> usize {
        self.values
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
            Params::Probit { means, vars, init_var } => {
                zeros(means, at, n);
                drop(vars.splice(at..at, std::iter::repeat_n(*init_var, n)));
            }
        }
    }

    /// grow2: removes the cell of sibling set `set` with its parameters: for a router an unsharing gave a single
    /// parent, below which the set is no longer upstream (see `Bdd::unshare`); its paths read 0 there.
    pub fn remove_cell(&mut self, set: usize) {
        let cell = self.cells.iter().position(|&s| s == set).expect("the table has a cell for the set");
        self.cells.remove(cell);
        let (n, at) = (self.values * self.class_num, cell * self.values * self.class_num);
        match &mut self.params {
            Params::Lr { weights: a, squares: b } | Params::Probit { means: a, vars: b, .. } => {
                a.drain(at..at + n);
                b.drain(at..at + n);
            }
            Params::Cnbp { counts, totals } => {
                counts.drain(at..at + n);
                totals.drain(cell * self.class_num..(cell + 1) * self.class_num);
            }
        }
    }

    /// grow6: makes the cells exactly `cells` (in registry order): the cells of other sets go with their parameters,
    /// missing ones come at neutral values. For a table kept beside a node (a trial's shadow, a snapshot), after a
    /// router split or its undo changed the node's.
    pub fn conform_to(&mut self, cells: &[usize]) {
        let gone: Vec<usize> = self.cells.iter().copied().filter(|set| !cells.contains(set)).collect();
        gone.into_iter().for_each(|set| self.remove_cell(set));
        for &set in cells {
            if !self.cells.contains(&set) {
                self.insert_cell(set);
            }
        }
    }

    /// grow6: sibling set `set` left the registry (an undone router split): its cell goes, if the table has one, and
    /// the later sets' cells move down one place in the registry's numbering (their order stays).
    pub fn drop_set(&mut self, set: usize) {
        if self.cells.contains(&set) {
            self.remove_cell(set);
        }
        self.cells.iter_mut().filter(|s| **s > set).for_each(|s| *s -= 1);
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

    /// Probit: the sums of the means and of the variances of the cells' weights for the values `values`, cell by cell
    /// in order (as `Probit::train_with_cells` sums them).
    pub fn probit_parts(&self, values: &[usize]) -> (f64, f64) {
        let Params::Probit { means, vars, .. } = &self.params else { unreachable!("not a probit's table") };
        let (mut mu, mut s2) = (0.0, 0.0);
        for (cell, &value) in values.iter().enumerate() {
            let i = self.index(cell, value, 0);
            mu += means[i];
            s2 += vars[i];
        }
        (mu, s2)
    }

    /// Probit: the cells' weights (mean, variance) for the values `values`, one per cell, to train with
    /// (`set_probit_cells` puts them back).
    pub fn probit_cells(&self, values: &[usize]) -> Vec<[f64; 2]> {
        let Params::Probit { means, vars, .. } = &self.params else { unreachable!("not a probit's table") };
        values.iter().enumerate().map(|(cell, &value)| cell * self.values + value).map(|i| [means[i], vars[i]]).collect()
    }

    pub fn set_probit_cells(&mut self, values: &[usize], cells: &[[f64; 2]]) {
        let v = self.values;
        let Params::Probit { means, vars, .. } = &mut self.params else { unreachable!("not a probit's table") };
        for ((cell, &value), w) in values.iter().enumerate().zip(cells) {
            let i = cell * v + value;
            (means[i], vars[i]) = (w[0], w[1]);
        }
    }

    /// Probit: adds `var` to every cell weight's variance, as `Probit::widen` does to the node's own weights.
    pub fn widen(&mut self, var: f64) {
        if let Params::Probit { vars, .. } = &mut self.params {
            vars.iter_mut().for_each(|s2| *s2 += var);
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
    /// counts and totals, divided by `factor`; a probit's variances multiplied by it.
    pub fn forget(&mut self, factor: f64) {
        match &mut self.params {
            Params::Lr { squares, .. } => squares.iter_mut().for_each(|s| *s /= factor),
            Params::Cnbp { counts, totals } => counts.iter_mut().chain(totals.iter_mut()).for_each(|s| *s /= factor),
            Params::Probit { vars, .. } => vars.iter_mut().for_each(|s| *s *= factor),
        }
    }

    /// nocells: makes every cell play-neutral, as treeview's "without" (the node's classifier alone): a logistic
    /// regression's weights and AdaGrad sums 0 (offsets exactly 0), a CNBP's counts and totals 0 (factors exactly 1),
    /// a probit's means and variances 0 (adding exactly 0 to the latent's mean and variance). The cells stay, as
    /// loading checks them against the upstream sets (`check`); a probit's init_var stays.
    pub fn neutralize(&mut self) {
        match &mut self.params {
            Params::Lr { weights: a, squares: b } | Params::Cnbp { counts: a, totals: b } | Params::Probit { means: a, vars: b, .. } => {
                a.iter_mut().chain(b.iter_mut()).for_each(|x| *x = 0.0)
            }
        }
    }

    /// Weights or counts, one per (cell, value, class); AdaGrad sums and totals not counted.
    pub fn param_count(&self) -> usize {
        self.cells.len() * self.values * self.class_num
    }

    /// The weights or counts, by (cell, value, class) (probit: the means, by (cell, value)).
    pub fn weights(&self) -> &[f64] {
        match &self.params {
            Params::Lr { weights, .. } => weights,
            Params::Cnbp { counts, .. } => counts,
            Params::Probit { means, .. } => means,
        }
    }

    /// Model file format: a header with the cells' set indices, then the parameters at full precision (weights and
    /// AdaGrad sums, or counts and totals), so a reloaded model trains on exactly as before.
    pub fn write_text(&self, s: &mut String) {
        let kind = match &self.params {
            Params::Lr { .. } => "lr",
            Params::Cnbp { .. } => "cnbp",
            Params::Probit { .. } => "probit",
        };
        write!(s, "pathtable {kind} {} {} {}", self.class_num, self.values, self.cells.len()).unwrap();
        self.cells.iter().for_each(|set| write!(s, " {set}").unwrap());
        s.push('\n');
        let (a, b) = match &self.params {
            Params::Lr { weights, squares } => (weights, squares),
            Params::Cnbp { counts, totals } => (counts, totals),
            Params::Probit { means, vars, init_var } => {
                writeln!(s, "{init_var:e}").unwrap();
                (means, vars)
            }
        };
        for x in a.iter().chain(b) {
            writeln!(s, "{x:e}").unwrap();
        }
    }

    pub fn read_text(t: &mut Tokens) -> io::Result<Self> {
        if t.next::<String>()? != "pathtable" {
            return Err(invalid("expected a node's path table"));
        }
        let kind = t.next::<String>()?;
        let (class_num, values, cell_num): (usize, usize, usize) = (t.next()?, t.next()?, t.next()?);
        let cells = (0..cell_num).map(|_| t.next()).collect::<io::Result<Vec<usize>>>()?;
        let mut table = match kind.as_str() {
            "lr" => PathTable::new(true, class_num, values),
            "cnbp" => PathTable::new(false, class_num, values),
            "probit" if class_num == 1 => PathTable::new_probit(values, t.next()?),
            other => return Err(invalid(format!("unknown path table kind {other:?} ({class_num} classes)"))),
        };
        for &set in &cells {
            if table.cells.last().is_some_and(|&last| last >= set) {
                return Err(invalid("a path table's cells are not in registry order"));
            }
            table.append_cell(set);
        }
        match &mut table.params {
            Params::Lr { weights: a, squares: b } | Params::Cnbp { counts: a, totals: b } | Params::Probit { means: a, vars: b, .. } => {
                for x in a.iter_mut().chain(b.iter_mut()) {
                    *x = t.next()?;
                }
            }
        }
        Ok(table)
    }

    /// State format (a probit's table, which only a probit node has, also writes its new cells' variance).
    pub fn write_state(&self, w: &mut Writer) {
        w.bool(self.is_lr());
        w.usize(self.class_num);
        w.usize(self.values);
        w.usizes(&self.cells);
        if let Params::Probit { init_var, .. } = &self.params {
            w.f64(*init_var);
        }
        match &self.params {
            Params::Lr { weights: a, squares: b } | Params::Cnbp { counts: a, totals: b } | Params::Probit { means: a, vars: b, .. } => {
                w.f64s(a);
                w.f64s(b);
            }
        }
    }

    /// Reads a state written by `write_state`, for a probit node's table if `probit`.
    pub fn read_state(r: &mut Reader, probit: bool) -> io::Result<Self> {
        let lr = r.bool()?;
        let (class_num, values) = (r.usize()?, r.usize()?);
        let cells = r.usizes()?;
        if cells.windows(2).any(|w| w[0] >= w[1]) {
            return Err(invalid("a path table's cells are not in registry order"));
        }
        if probit && (lr || class_num != 1) {
            return Err(invalid("a probit node's path table in the state is not a probit's"));
        }
        let mut table = match probit {
            true => PathTable::new_probit(values, r.f64()?),
            false => PathTable::new(lr, class_num, values),
        };
        cells.iter().for_each(|&set| table.append_cell(set));
        let (a, b) = (r.f64s()?, r.f64s()?);
        match &mut table.params {
            Params::Lr { weights: x, squares: y } | Params::Cnbp { counts: x, totals: y } | Params::Probit { means: x, vars: y, .. } => {
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
    match classifier {
        NodeModel::Probit(_) => PathTable::new_probit(g.params.branch_factor + 1, g.params.split_noise * g.params.split_noise),
        _ => PathTable::new(matches!(classifier, NodeModel::Lr(_)), classifier.class_num(), g.params.branch_factor + 1),
    }
}

/// After leaf `idx` split into the new leaves `made`: each new leaf starts with the leaf's table, forgotten as the
/// leaf's classifier is (see `Graph::split_leaf`), and the new router gets the same cells, empty.
pub(crate) fn after_leaf_split(g: &mut Graph, idx: usize, made: &[usize]) {
    let table = *g.nodes[idx].table.take().expect("with path context every node has a path table");
    for &m in made {
        let mut copy = table.clone();
        copy.forget(g.params.forget_factor);
        if copy.is_probit() {
            // As Graph::add_split_noise widens the copy's own weights.
            copy.widen(g.params.split_noise * g.params.split_noise);
        }
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
        n.base_step = 0;
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
/// from the one whose set's value names it. Errors if the values do not describe a path. grow2: below a router that a
/// router split rule unshared, a node's parents may be routers of no set (the unshared router's copies); there the
/// path is the one path from the root whose sets give exactly these values (`decode_paths`).
pub(crate) fn decode(g: &Graph, idx: usize, values: &[usize]) -> Result<Vec<usize>, String> {
    decode_by_parents(g, idx, values).or_else(|e| decode_paths(g, idx, values).map_err(|_| e))
}

/// grow2: `decode` by search: the paths from the root to node `idx` whose marks give its cells exactly `values`;
/// errors unless there is one. Paths through a member of a set whose cell names another member are not followed.
fn decode_paths(g: &Graph, idx: usize, values: &[usize]) -> Result<Vec<usize>, String> {
    let table = g.nodes[idx].table.as_ref().ok_or("the node has no path table")?;
    let value_of = |set: usize| table.cells.iter().position(|&s| s == set).map(|cell| values[cell]);
    let (mut found, mut path, mut marks, mut visits) = (Vec::new(), vec![0], Vec::new(), 0usize);
    fn walk(
        g: &Graph,
        idx: usize,
        values: &[usize],
        value_of: &dyn Fn(usize) -> Option<usize>,
        path: &mut Vec<usize>,
        marks: &mut Vec<Mark>,
        found: &mut Vec<Vec<usize>>,
        visits: &mut usize,
    ) {
        *visits += 1;
        let x = *path.last().unwrap();
        if x == idx {
            let mut v = Vec::new();
            g.nodes[idx].table.as_ref().unwrap().values_for(marks, &mut v);
            if v == values {
                found.push(path.clone());
            }
            return;
        }
        if found.len() > 1 || *visits > 1_000_000 {
            return;
        }
        for &c in g.children(x) {
            let member = g.nodes[c].member;
            if let Some((set, j)) = member {
                if c != idx && value_of(set) != Some(j) {
                    continue;
                }
            }
            path.push(c);
            marks.extend(member);
            walk(g, idx, values, value_of, path, marks, found, visits);
            path.pop();
            if member.is_some() {
                marks.pop();
            }
        }
    }
    walk(g, idx, values, &value_of, &mut path, &mut marks, &mut found, &mut visits);
    match found.len() {
        1 => Ok(found.pop().unwrap()),
        n => Err(format!("{n} paths to node {idx} give these values")),
    }
}

fn decode_by_parents(g: &Graph, idx: usize, values: &[usize]) -> Result<Vec<usize>, String> {
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
        let probit = matches!(node.classifier, NodeModel::Probit(_));
        let class_num = if probit { 1 } else { node.classifier.class_num() };
        if table.is_lr() != lr || table.is_probit() != probit || table.class_num != class_num || table.values != g.params.branch_factor + 1 {
            return Err(invalid(format!("node {idx}'s path table does not fit its classifier")));
        }
        if table.cells != upstream[idx] {
            return Err(invalid(format!("node {idx}'s path table has the cells {:?}, not the sets upstream of it {:?}", table.cells, upstream[idx])));
        }
    }
    Ok(())
}
