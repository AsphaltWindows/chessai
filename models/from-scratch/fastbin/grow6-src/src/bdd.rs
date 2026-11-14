//! Delegation DAG: a delegation tree (`bdt`) whose delegators can split
//! too. Rust only.
//!
//! A leaf splits as in the BDT, into a delegator over new copies of itself.
//! A delegator splits when its routing stopped improving (by more than
//! `split_z` standard errors since its previous window) while its error is
//! at least the mean error of all routers' choices over the same window. It
//! then becomes a router over `branch_factor` new delegators, copies of its
//! old router that forget by `forget_factor`, which all route to its old
//! children. Children are shared, so the routing can specialise without
//! copying the nodes below it.
//!
//! Delegators made by the same split share their children, so their
//! subtrees predict the same unless they route differently. With error
//! routing their parent compares them by how well each routes the example
//! instead: the router error of predicting the best of the shared children.
//! With `SubrouterTarget::Home` (bddhome) the parent keeps its old router
//! and its old target instead, which shared child predicts best, and its
//! class i sends the example to the sub-router whose home is shared child i.
//!
//! Models are saved in the BDT file format.

use std::io;
use std::path::Path;

use crate::delegation::{
    Blocks, DelegationParams, Graph, GrowStats, Kind, NodeKind, NodeModel, Partition, RouteTarget, RouterGrow, RouterTrial,
    Shadows, SplitKind, SubrouterTarget, Trial, at_least,
};
use crate::pathctx::Mark;
use crate::state::{Reader, Writer};

#[cfg(test)]
thread_local! {
    /// The node sequence training is routing the current position by, and the number of training paths checked.
    static ACTUAL_PATH: std::cell::RefCell<Vec<usize>> = const { std::cell::RefCell::new(Vec::new()) };
    static PATHS_CHECKED: std::cell::RefCell<usize> = const { std::cell::RefCell::new(0) };
}

#[derive(Clone, Debug)]
pub struct Bdd {
    graph: Graph,
    /// With `SubrouterTarget::Home`: each delegator's home children (see
    /// `home_children`), worked out once per structure. Derived: not saved.
    home_cache: HomeCache,
}

/// `Bdd::home_children` of each node (None: not worked out yet), for the
/// structure with `key`, the graph's split count and node count: a BDD's
/// structure changes only by splits, which add nodes (home targets allow
/// no split trials, and BDDs no merges, so no node is ever removed).
#[derive(Clone, Debug, Default)]
struct HomeCache {
    key: (u32, usize),
    homes: Vec<Option<Option<Vec<usize>>>>,
    /// Each node's `home_base` (None: not worked out yet).
    bases: Vec<Option<Vec<usize>>>,
}

impl Bdd {
    pub fn new(categories: &[u8], params: DelegationParams) -> Self {
        Bdd { graph: Graph::new(categories, Self::checked(params)), home_cache: HomeCache::default() }
    }

    /// A single leaf that starts as `root`, a classifier of the leaves' kind,
    /// instead of an untrained one.
    pub fn starting_from(root: NodeModel, categories: &[u8], params: DelegationParams) -> Self {
        Bdd { graph: Graph::starting_from(root, categories, Self::checked(params)), home_cache: HomeCache::default() }
    }

    /// Revisable structure is for trees: a DAG's branches share nodes.
    fn checked(params: DelegationParams) -> DelegationParams {
        // grow6: the growth rule's leaf trials run in DAGs with path context too (the trial's shadow keeps the leaf's
        // path table); without path context it is for trees only, as before.
        assert!(
            params.split_rule != crate::delegation::SplitRule::Grow || params.path_context,
            "the growth rule is for BDTs, and for BDDs with path context"
        );
        assert!(
            params.revise.is_none() && params.reset_leaves.is_none(),
            "revisable structure (revise, reset_leaves) is for BDTs only"
        );
        assert!(
            params.subrouter_target == SubrouterTarget::Routing
                || (params.route == RouteTarget::Error && params.split_trial.is_none()),
            "subrouter_target home is for error routing without split trials"
        );
        if params.path_context {
            // errdag3: probit leaves too, whose cells start at variance split_noise^2 (see `pathctx`).
            let nodes_fit = matches!(params.leaf, NodeKind::Lr | NodeKind::Cnbp | NodeKind::Probit)
                && matches!(params.router, NodeKind::Lr | NodeKind::Cnbp);
            assert!(nodes_fit, "path_context is for logistic-regression and CNBP routers, and logistic-regression, CNBP or probit leaves");
            assert!(
                params.leaf != NodeKind::Probit || (params.split_noise > 0.0 && !params.probit_shared_eps),
                "path_context with probit leaves needs split_noise > 0 (the cells' prior) and no probit_shared_eps"
            );
            assert!(
                params.route == RouteTarget::Error && params.route_switch.is_none(),
                "path_context is for error routing (no cluster routing, no route_switch)"
            );
            assert!(params.split_trial.is_none(), "path_context refuses split trials");
            assert!(params.leaf_share == 0.0, "path_context refuses leaf_share");
        }
        if params.router_split_limit.is_some() {
            // Graph::split_counts counts leaf splits by the leaves.
            assert!(
                params.split_trial.is_none() && params.leaf_share == 0.0,
                "router_split_limit refuses split trials and leaf_share"
            );
        }
        params
    }

    /// The number of path table parameters (see `pathctx`), and the number of sibling sets.
    pub fn path_param_count(&self) -> (usize, usize) {
        (self.graph.path_param_count(), self.graph.sets.len())
    }

    pub fn params(&self) -> &DelegationParams {
        &self.graph.params
    }

    pub fn node_count(&self) -> usize {
        self.graph.nodes.len()
    }

    pub fn split_count(&self) -> u32 {
        self.graph.split_number
    }

    /// Writes the probability of each class into `out`.
    ///
    /// # Panics
    /// With `use_probs`, if the model was trained since the last `refresh()`.
    pub fn predict(&self, data: &[u8], out: &mut [f64]) {
        self.graph.predict(data, out)
    }

    /// See `Cnbp::predict_log`.
    pub fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        self.graph.predict_log(data, out)
    }

    /// Index of the leaf that predicts the position.
    /// The routing graph, for `compound`.
    pub(crate) fn graph(&self) -> &crate::delegation::Graph {
        &self.graph
    }

    pub fn leaf(&self, data: &[u8]) -> usize {
        self.graph.leaf(data)
    }

    /// The leaves' node indices and classifiers.
    pub fn leaves(&self) -> impl Iterator<Item = (usize, &NodeModel)> {
        self.graph.leaves()
    }

    /// See `Graph::leaf_weights`.
    pub fn leaf_weights(&self, data: &[u8], tau: f64, min_weight: f64, out: &mut Vec<(usize, f64)>) {
        self.graph.leaf_weights(data, tau, min_weight, out)
    }

    /// Writes node `idx`'s own prediction (see `Cnbp::predict_log`).
    pub fn node_predict_log(&self, idx: usize, data: &[u8], out: &mut [f64]) {
        self.graph.node_predict_log(idx, data, out)
    }

    /// Recomputes cached probabilities after training (see `Cnbp::refresh`).
    pub fn refresh(&mut self) {
        self.graph.refresh()
    }

    /// Adds one data point. `label` must sum to 1.
    pub fn train(&mut self, data: &[u8], label: &[f64]) {
        let mut scratch = vec![0.0; self.graph.params.class_num];
        if self.graph.params.path_context {
            let error = self.graph.begin_step_ctx(data, label, &mut scratch);
            #[cfg(test)]
            ACTUAL_PATH.with(|p| *p.borrow_mut() = vec![0]);
            self.train_node_ctx(0, None, &mut Vec::new(), data, label, error, &mut scratch);
            // grow6: the growth rule's credits after the step, as `Bdt::train` (nothing without the rule); then the
            // encourage rule's allowance, verdicts and splits, with no example in flight.
            self.graph.end_step();
            if self.graph.encouraging() {
                self.encourage_step_end();
            }
            return;
        }
        let error = self.graph.begin_step(data, label, &mut scratch);
        self.train_node(0, data, label, error, &mut scratch);
    }

    /// `train_node` with path context (see `pathctx`): the position entered node `idx` from `parent` (None at the root)
    /// with `marks`, which are as they were on return. Every prediction it asks for is for the path it would take: a
    /// router scoring its children enters each child with the child's mark, and so on down. Without split trials and
    /// cluster routing.
    #[allow(clippy::too_many_arguments)]
    fn train_node_ctx(
        &mut self,
        idx: usize,
        parent: Option<usize>,
        marks: &mut Vec<Mark>,
        data: &[u8],
        label: &[f64],
        error: Option<f64>,
        scratch: &mut [f64],
    ) {
        let g = &mut self.graph;
        let Kind::Branch { children } = &g.nodes[idx].kind else {
            if let Some(split) = g.check_leaf_split(idx) {
                if split {
                    g.nodes[idx].forget_window();
                    // grow2: the router a leaf split made starts afresh (spec 0.3): its leaf's statistics are not its own.
                    // grow6: under the growth rule `grow_restart` did, keeping what an undone trial's back-off reads,
                    // and the router's own statistics start when the trial keeps the split (`Graph::end_trial`).
                    if g.router_ruled() && !g.growing() {
                        (g.nodes[idx].grow, g.nodes[idx].rgrow) = (GrowStats::default(), RouterGrow::default());
                    }
                }
                return self.train_node_ctx(idx, parent, marks, data, label, error, scratch);
            }
            #[cfg(test)]
            self.check_training_path(idx, marks);
            return self.graph.train_leaf_ctx(idx, marks, data, label, error);
        };
        let children = children.clone();
        // grow6: a growth-rule leaf trial's step, first, as in `Bdt::train_node`; an undone split trains as the leaf.
        if self.graph.trial_step_ctx(idx, marks, data, label, scratch) {
            return self.train_node_ctx(idx, parent, marks, data, label, error, scratch);
        }
        if self.check_delegator_split(idx) {
            // grow2: if the check unshared the router, the position goes on to its arrival path's copy.
            let idx = match self.graph.unshared.take() {
                Some((v, parents, made)) if v == idx => {
                    let from = parent.expect("an unshared router has parents");
                    let to = made[parents.iter().position(|&p| p == from).expect("the position came from one of the router's parents")];
                    #[cfg(test)]
                    ACTUAL_PATH.with(|p| *p.borrow_mut().last_mut().expect("a path") = to);
                    to
                }
                other => {
                    assert!(other.is_none(), "an unsharing of another router than the one checked");
                    idx
                }
            };
            return self.train_node_ctx(idx, parent, marks, data, label, error, scratch);
        }
        let len = marks.len();
        if error.is_none() {
            let routed = self.graph.route_ctx(idx, marks, data);
            self.graph.enter(children[routed], marks);
            #[cfg(test)]
            ACTUAL_PATH.with(|p| p.borrow_mut().push(children[routed]));
            self.train_node_ctx(children[routed], Some(idx), marks, data, label, None, scratch);
            marks.truncate(len);
            return;
        }
        let (routed, child_error, router_error) = match self.home_children(idx, &children) {
            Some(home) => {
                assert_eq!(home.len(), children.len(), "home targets need one sub-router per home child");
                let home_errors = self.home_errors_ctx(idx, &children, &home, marks, data, label, scratch);
                let (routed, router_error) = self.graph.train_router_ctx(idx, marks, data, &home_errors);
                self.graph.enter(children[routed], marks);
                let child_error = self.graph.subtree_error_ctx(children[routed], marks, data, label, scratch);
                marks.truncate(len);
                (routed, child_error, router_error)
            }
            None => {
                let ruled = self.graph.router_ruled();
                let mut losses = None;
                let child_errors = match self.shared_children(&children) {
                    // grow2: the copies' routing errors, and the losses of the children they route to.
                    Some(shared) if ruled => {
                        let (errors, copy_losses) = self.routing_losses_ctx(&children, &shared, marks, data, label, scratch);
                        losses = Some(copy_losses);
                        errors
                    }
                    Some(shared) => self.routing_errors_ctx(&children, &shared, marks, data, label, scratch),
                    None => (0..children.len())
                        .map(|i| {
                            self.graph.enter(children[i], marks);
                            let e = self.graph.subtree_error_ctx(children[i], marks, data, label, scratch);
                            marks.truncate(len);
                            e
                        })
                        .collect(),
                };
                if ruled {
                    // grow2: the router's headroom and its shadows' gain, before it trains (theory-2 6.40).
                    let losses = losses.unwrap_or_else(|| child_errors.clone());
                    self.graph.router_observe(idx, parent, marks, data, &losses, &child_errors);
                    // grow6: a router split's long trial, at the routers that head its region.
                    if self.graph.encouraging() {
                        self.router_trial_observe(idx, marks, data, label, &losses, &child_errors, scratch);
                    }
                }
                let (routed, router_error) = self.graph.train_router_ctx(idx, marks, data, &child_errors);
                (routed, child_errors[routed], router_error)
            }
        };
        self.graph.enter(children[routed], marks);
        #[cfg(test)]
        ACTUAL_PATH.with(|p| p.borrow_mut().push(children[routed]));
        self.train_node_ctx(children[routed], Some(idx), marks, data, label, Some(child_error), scratch);
        marks.truncate(len);
        self.graph.record_router(idx, router_error);
    }

    /// With path context and home targets, for router `idx` over `children` whose home children are `home`: each home
    /// child's error, the position entering it by its home chain (child i of each sub-router in turn, as a router
    /// split leaves the routing, see `split_delegator`), so each copy on the way gives its sibling number.
    fn home_errors_ctx(
        &mut self,
        idx: usize,
        children: &[usize],
        home: &[usize],
        marks: &mut Vec<Mark>,
        data: &[u8],
        label: &[f64],
        scratch: &mut [f64],
    ) -> Vec<f64> {
        let len = marks.len();
        (0..children.len())
            .map(|i| {
                let mut x = children[i];
                self.graph.enter(x, marks);
                while x != home[i] {
                    assert!(marks.len() - len < self.graph.nodes.len(), "router {idx}'s home chain {i} does not reach its home child");
                    x = self.graph.children(x)[i];
                    self.graph.enter(x, marks);
                }
                let e = self.graph.subtree_error_ctx(x, marks, data, label, scratch);
                marks.truncate(len);
                e
            })
            .collect()
    }

    /// `routing_errors` with path context: each copy's router error at predicting the error target of the shared
    /// children as the position would reach them through that copy (the copy's sibling number in their cells).
    fn routing_errors_ctx(
        &mut self,
        children: &[usize],
        shared: &[usize],
        marks: &mut Vec<Mark>,
        data: &[u8],
        label: &[f64],
        scratch: &mut [f64],
    ) -> Vec<f64> {
        let len = marks.len();
        let (mut router_small, mut router_big) = ([0.0; 8], Vec::new());
        let router = crate::delegation::small_buffer(&mut router_small, &mut router_big, self.graph.params.branch_factor);
        children
            .iter()
            .map(|&child| {
                self.graph.enter(child, marks);
                let inner = marks.len();
                let errors: Vec<f64> = shared
                    .iter()
                    .map(|&s| {
                        self.graph.enter(s, marks);
                        let e = self.graph.subtree_error_ctx(s, marks, data, label, scratch);
                        marks.truncate(inner);
                        e
                    })
                    .collect();
                let target = self.graph.error_target(&errors);
                self.graph.node_output_ctx(child, marks, data, router);
                let e = self.graph.router_error(router, &target);
                marks.truncate(len);
                e
            })
            .collect()
    }

    /// grow2: `routing_errors_ctx`, and each copy's loss: the error of the shared child it routes to, through it (the
    /// error of its subtree's prediction, `subtree_error_ctx`).
    fn routing_losses_ctx(
        &mut self,
        children: &[usize],
        shared: &[usize],
        marks: &mut Vec<Mark>,
        data: &[u8],
        label: &[f64],
        scratch: &mut [f64],
    ) -> (Vec<f64>, Vec<f64>) {
        let len = marks.len();
        let (mut router_small, mut router_big) = ([0.0; 8], Vec::new());
        let router = crate::delegation::small_buffer(&mut router_small, &mut router_big, self.graph.params.branch_factor);
        let mut losses = Vec::with_capacity(children.len());
        let errors = children
            .iter()
            .map(|&child| {
                self.graph.enter(child, marks);
                let inner = marks.len();
                let errors: Vec<f64> = shared
                    .iter()
                    .map(|&s| {
                        self.graph.enter(s, marks);
                        let e = self.graph.subtree_error_ctx(s, marks, data, label, scratch);
                        marks.truncate(inner);
                        e
                    })
                    .collect();
                let target = self.graph.error_target(&errors);
                self.graph.node_output_ctx(child, marks, data, router);
                let e = self.graph.router_error(router, &target);
                losses.push(errors[crate::delegation::argmax(router)]);
                marks.truncate(len);
                e
            })
            .collect();
        (errors, losses)
    }

    /// Test check: the values of the trained leaf's cells, for the marks the position brought, decode to exactly the
    /// node sequence training routed it by.
    #[cfg(test)]
    fn check_training_path(&self, leaf: usize, marks: &[Mark]) {
        let mut values = Vec::new();
        self.graph.nodes[leaf].table.as_ref().unwrap().values_for(marks, &mut values);
        let decoded = crate::pathctx::decode(&self.graph, leaf, &values).expect("the path values decode");
        let actual = ACTUAL_PATH.with(|p| p.borrow().clone());
        assert_eq!(decoded, actual, "the leaf's path values decode to another path");
        PATHS_CHECKED.with(|n| *n.borrow_mut() += 1);
    }

    /// As `Bdt::train_node`, plus delegator splits.
    fn train_node(&mut self, idx: usize, data: &[u8], label: &[f64], error: Option<f64>, scratch: &mut [f64]) {
        let g = &mut self.graph;
        let Kind::Branch { children } = &g.nodes[idx].kind else {
            if let Some(split) = g.check_leaf_split(idx) {
                if split {
                    // The new router's error is not comparable with the
                    // leaf's, so its first window cannot split it.
                    g.nodes[idx].forget_window();
                }
                return self.train_node(idx, data, label, error, scratch);
            }
            return g.train_leaf(idx, data, label, error);
        };
        let children = children.clone();
        if self.graph.trial_step(idx, data, label, scratch) {
            // The split was undone; train the node as it is now.
            return self.train_node(idx, data, label, error, scratch);
        }
        if self.check_delegator_split(idx) {
            return self.train_node(idx, data, label, error, scratch);
        }
        let g = &mut self.graph;
        if error.is_none() {
            let routed = g.route(idx, data);
            return self.train_node(children[routed], data, label, None, scratch);
        }

        // Error routing compares every child; cluster routing needs only the
        // error of the child it picks (which a delegator ignores).
        let (routed, child_error, router_error) = match self.graph.params.route {
            RouteTarget::Error => match self.home_children(idx, &children) {
                // bddhome: the old router's target, which home child predicts
                // best; class i sends the example to sub-router i, whose
                // home is home child i. The sub-router's error is its own,
                // that of the leaf it reaches, as for any child.
                Some(home) => {
                    assert_eq!(home.len(), children.len(), "home targets need one sub-router per home child");
                    let home_errors = self.graph.prediction_errors(&home, data, label, scratch);
                    let (routed, router_error) = self.graph.train_router(idx, data, &home_errors);
                    let child_error = self.graph.prediction_errors(&children[routed..=routed], data, label, scratch)[0];
                    (routed, child_error, router_error)
                }
                None => {
                    let child_errors = match self.shared_children(&children) {
                        Some(shared) => self.routing_errors(&children, &shared, data, label, scratch),
                        None => self.graph.prediction_errors(&children, data, label, scratch),
                    };
                    let (routed, router_error) = self.graph.train_router(idx, data, &child_errors);
                    (routed, child_errors[routed], router_error)
                }
            },
            RouteTarget::Cluster => {
                let g = &mut self.graph;
                let (routed, router_error) = g.train_cluster_router(idx, &children, data, label, scratch);
                (routed, g.prediction_errors(&children[routed..=routed], data, label, scratch)[0], router_error)
            }
        };
        self.train_node(children[routed], data, label, Some(child_error), scratch);
        self.graph.record_router(idx, router_error);
    }

    /// Like `Graph::check_leaf_split`, for delegator `idx`. Returns whether
    /// a check was due (the delegator may have changed).
    fn check_delegator_split(&mut self, idx: usize) -> bool {
        // grow2: a router split rule has its own close (theory-2 6.40-6.41).
        if self.graph.router_ruled() {
            return self.rule_close(idx);
        }
        let g = &mut self.graph;
        if g.nodes[idx].num_trained_last < g.params.split_threshold {
            return false;
        }
        // errdag2: with router_split_limit, delegator splits have their own budget, and may only trail the leaf
        // splits (fewer delegator splits than leaf splits so far); the window is checked as usual either way.
        // Without it, every split counts towards split_limit, and a full limit checks no window.
        let allowed = match g.params.router_split_limit {
            None if g.split_number >= g.params.split_limit => return false,
            None => true,
            Some(cap) => {
                let (leaf_splits, router_splits) = g.split_counts();
                router_splits < cap && router_splits < leaf_splits
            }
        };
        let node = &mut g.nodes[idx];
        let (recent, previous, margin) = node.close_window(g.params.router_split_z.unwrap_or(g.params.split_z));
        let routers_mean = g.router_errors.mean_since(node.window_routers);
        // errdag: with router_split_z a delegator needs a rise of its window's mean router error by more than the
        // margin (infinity: never); without it, the stall rule by split_z. The window's bookkeeping is the same.
        let stalled = match g.params.router_split_z {
            Some(z) => z < f64::INFINITY && recent - previous > margin,
            None => previous - recent < margin,
        };
        if allowed && previous >= 0.0 && stalled && at_least(recent, routers_mean) {
            self.split_delegator(idx);
        }
        self.graph.open_window(idx);
        true
    }

    /// grow2: `check_delegator_split` under a router split rule: once router `idx` has a window of examples, closes it
    /// (`Graph::router_close`), makes the split it decides on, if any, and opens the next window. Returns whether a
    /// check was due.
    fn rule_close(&mut self, idx: usize) -> bool {
        if self.graph.nodes[idx].num_trained_last < self.graph.params.split_threshold {
            return false;
        }
        // grow6, encourage: a leaf trial's delegator closes no router window: its windows start when the trial keeps
        // the split (`Graph::end_trial`), and an undone trial leaves the leaf's statistics as the tree's.
        if self.graph.encouraging() && self.graph.nodes[idx].trial.is_some() {
            return false;
        }
        if let Some((kind, ref_mean, ref_se)) = self.graph.router_close(idx) {
            let shadows = self.graph.nodes[idx].rgrow.shadows.take();
            let made = match kind {
                SplitKind::Cluster => self.cluster_split(idx, shadows),
                SplitKind::Path => self.unshare(idx, shadows),
            };
            self.graph.after_rule_split(idx, kind, made, ref_mean, ref_se);
        }
        self.graph.open_window(idx);
        // grow6, encourage: the path cells' size and movement at each of the root's closes (6.80 (2)).
        if idx == 0 && self.graph.encouraging() && self.graph.params.grow.log_router_stats {
            let report = self.graph.cell_report();
            eprintln!("path cells at N {}: {report}", self.graph.step);
        }
        true
    }

    /// grow6: the encourage rule at the end of a training step (theory-2 6.80 (1)), with no example in flight: the
    /// allowance's credits; the verdicts of the router splits whose trials have run their K windows; then, if a router
    /// closed a window this step and a credit is banked, the split of the eligible router the priority puts first.
    fn encourage_step_end(&mut self) {
        self.graph.router_credits();
        let g = &self.graph;
        let length = g.params.grow.router_trial_windows as u64 * g.params.split_threshold as u64;
        let mut i = 0;
        while i < self.graph.router_trials.len() {
            if self.graph.router_trials[i].examples >= length {
                let trial = self.graph.router_trials.remove(i);
                self.router_verdict(trial);
            } else {
                i += 1;
            }
        }
        if !std::mem::take(&mut self.graph.rallow.due) || self.graph.rallow.credits == 0 {
            return;
        }
        match self.graph.encourage_pick() {
            (Some(v), candidates) => self.encourage_split(v, &candidates),
            (None, _) => {
                if self.graph.params.grow.log_router_stats {
                    eprintln!(
                        "router allowance at N {}: router credits {}, no eligible router (the credit waits)",
                        self.graph.step, self.graph.rallow.credits
                    );
                }
            }
        }
    }

    /// grow6: router `v` splits on the allowance (6.80 (1)), whatever its gain: by arrival path where it has several,
    /// else by cluster, into its shadows where they fit (as the router split rules split). No bar. The split's long
    /// trial starts: the router's null shadows (or, without fitting shadows, fresh copies of it) and a copy of the
    /// router as it was, each with its table, go on routing among its old children.
    fn encourage_split(&mut self, v: usize, candidates: &[String]) {
        let g = &mut self.graph;
        let kind = g.split_kind(v).expect("an eligible router has a split kind");
        let node = &g.nodes[v];
        let shadow = node.classifier.clone();
        let table = node.table.as_deref().expect("with path context every node has a path table").clone();
        let (chain, backoff_windows) = (node.chain, node.grow.backoff_windows);
        let chosen = (node.rgrow.prio_g, node.rgrow.prio_h);
        let (ref_mean, ref_se) = (node.grow.prev_mean, node.grow.prev_se);
        let children = g.children(v).to_vec();
        let parents = match kind {
            SplitKind::Path => g.path_parents(v).expect("a path split has several parents"),
            SplitKind::Cluster => Vec::new(),
        };
        let mut shadows = if g.shadows_fit(v) { g.nodes[v].rgrow.shadows.take() } else { None };
        let from_shadows = shadows.is_some();
        // The null shadows go on as the trial's comparison (as many as the split's parts); fresh copies of the router
        // and its table, as the shadows start, where it had none.
        let parts = match kind {
            SplitKind::Path => parents.len(),
            SplitKind::Cluster => g.params.branch_factor,
        };
        let carried = shadows.as_mut().map(|s| std::mem::take(&mut s.null)).filter(|null| null.len() == parts);
        let null_carried = carried.is_some();
        let null = carried.unwrap_or_else(|| vec![(shadow.clone(), table.clone()); parts]);
        let made = match kind {
            SplitKind::Cluster => self.cluster_split(v, shadows),
            SplitKind::Path => self.unshare(v, shadows),
        };
        let g = &mut self.graph;
        // No example is in flight: nothing goes on to a copy.
        g.unshared = None;
        let copies: Vec<usize> = match kind {
            SplitKind::Cluster => made.clone(),
            SplitKind::Path => made[1..].to_vec(),
        };
        for &m in made.iter().chain([&v]) {
            g.nodes[m].chain = chain + 1;
        }
        g.rallow.credits -= 1;
        g.rallow.splits += 1;
        let set = (kind == SplitKind::Cluster).then(|| g.sets.len() - 1);
        let describe = |x: Option<f64>| x.map_or("none".to_string(), |x| format!("{x:+.7}"));
        eprintln!(
            "router split on the allowance at N {}: router {v} ({}), copies {copies:?}{}, children {children:?}; chosen by G-Gnull {}, H {} among {}; router credits {}, router splits {} (kept {}, undone {}, on trial {})",
            g.step,
            match kind {
                SplitKind::Path => format!("by arrival path, parents {parents:?}"),
                SplitKind::Cluster => format!("by cluster, set {}", set.expect("a cluster split's set")),
            },
            match (from_shadows, null_carried) {
                (true, true) => " (its shadows; its null shadows run on as the trial's pair)",
                (true, false) => " (its shadows; fresh null shadows)",
                _ => " (forgotten copies; fresh null shadows)",
            },
            describe(chosen.0),
            describe(chosen.1),
            candidates.join(", "),
            g.rallow.credits,
            g.rallow.splits,
            g.rallow.kept,
            g.rallow.undone,
            g.router_trials.len() + 1
        );
        g.after_rule_split(v, kind, made, ref_mean, ref_se);
        g.router_trials.push(RouterTrial {
            router: v,
            kind,
            at: g.step,
            made: copies,
            set,
            children,
            parents,
            shadow,
            table,
            null,
            chain,
            backoff_windows,
            chosen,
            examples: 0,
            blocks: Blocks::default(),
            window: Blocks::default(),
            window_means: Vec::new(),
            real: Blocks::default(),
            real_window: Blocks::default(),
        });
        g.conform_aux_tables();
        if g.params.grow.log_router_stats {
            let report = g.cell_report();
            eprintln!("path cells after the split at N {}: {report}", g.step);
        }
    }

    /// grow6: at router `idx`, for a scored example, before the router trains: if `idx` heads the region of a router
    /// split on trial (a cluster split's central router; an unsharing's router or one of its copies), the trial's pair
    /// (6.80 (1)): the loss of the split's pick (`losses` at the router's pick before it trains) minus the loss of the
    /// pick of the null shadow of the example's hash part among the old children before it trains (G_real - G_null);
    /// and, logged only, minus the unsplit router's pick's (G_real). An unsharing's old losses are the same `losses`, a
    /// cluster split's its old children's losses through no copy (the new set's cells read 0, as before the split).
    /// Then the null shadow and the unsplit router train on the target. Every `split_threshold` examples the trial's
    /// window is logged.
    #[allow(clippy::too_many_arguments)]
    fn router_trial_observe(
        &mut self,
        idx: usize,
        marks: &mut Vec<Mark>,
        data: &[u8],
        label: &[f64],
        losses: &[f64],
        child_errors: &[f64],
        scratch: &mut [f64],
    ) {
        let g = &mut self.graph;
        let heads = |t: &RouterTrial| t.router == idx || (t.kind == SplitKind::Path && t.made.contains(&idx));
        let Some(t) = g.router_trials.iter().position(heads) else { return };
        let routed = g.route_ctx(idx, marks, data);
        let split_loss = losses[routed];
        let kind = g.router_trials[t].kind;
        let (old_losses, old_errors) = match kind {
            SplitKind::Path => (losses.to_vec(), child_errors.to_vec()),
            SplitKind::Cluster => {
                let old = g.router_trials[t].children.clone();
                self.children_losses(&old, marks, data, label, scratch)
            }
        };
        let g = &mut self.graph;
        let (bf, len, w, k) = (g.params.branch_factor, g.grow_block_len(), g.params.split_threshold as u64, g.params.grow.router_trial_windows);
        let target = g.error_target(&old_errors);
        let step = g.step;
        let trial = &mut g.router_trials[t];
        let (mut values, mut out) = (Vec::new(), vec![0.0; bf]);
        // The pair: the null shadow of the example's hash part (stall-hg's partition), before it trains.
        let part = (crate::delegation::position_hash(data) % trial.null.len() as u64) as usize;
        let (router, table) = &mut trial.null[part];
        table.values_for(marks, &mut values);
        crate::delegation::shadow_output(router, table, &values, data, &mut out);
        let d = split_loss - old_losses[crate::delegation::argmax(&out)];
        crate::delegation::shadow_train(router, table, &values, data, &target);
        trial.blocks.add(d, len);
        trial.window.add(d, len);
        // The unsplit router, before it trains.
        values.clear();
        trial.table.values_for(marks, &mut values);
        crate::delegation::shadow_output(&trial.shadow, &trial.table, &values, data, &mut out);
        let d_real = split_loss - old_losses[crate::delegation::argmax(&out)];
        crate::delegation::shadow_train(&mut trial.shadow, &mut trial.table, &values, data, &target);
        trial.real.add(d_real, len);
        trial.real_window.add(d_real, len);
        trial.examples += 1;
        if trial.examples % w == 0 {
            if trial.window.count >= 1 {
                trial.window_means.push(-trial.window.sum / trial.window.count as f64);
            }
            eprintln!(
                "router trial of router {} ({:?} split at N {}) window {} of {k} at N {step}: G-Gnull {}, over the trial {}; G (against the unsplit router) {}, over the trial {}",
                trial.router,
                trial.kind,
                trial.at,
                trial.examples / w,
                describe_gain(&trial.window),
                describe_gain(&trial.blocks),
                describe_gain(&trial.real_window),
                describe_gain(&trial.real)
            );
            trial.window = Blocks::default();
            trial.real_window = Blocks::default();
        }
    }

    /// grow6: the losses and errors of `children` for a position that entered their router with `marks`, as
    /// `train_node_ctx` takes them for a router over them: over copies that share their children, each copy's loss
    /// (the error of the shared child it routes to) and its routing error; otherwise each child's subtree error.
    fn children_losses(&mut self, children: &[usize], marks: &mut Vec<Mark>, data: &[u8], label: &[f64], scratch: &mut [f64]) -> (Vec<f64>, Vec<f64>) {
        if let Some(shared) = self.shared_children(children) {
            let (errors, losses) = self.routing_losses_ctx(children, &shared, marks, data, label, scratch);
            return (losses, errors);
        }
        let len = marks.len();
        let errors: Vec<f64> = children
            .iter()
            .map(|&c| {
                self.graph.enter(c, marks);
                let e = self.graph.subtree_error_ctx(c, marks, data, label, scratch);
                marks.truncate(len);
                e
            })
            .collect();
        (errors.clone(), errors)
    }

    /// grow6: the verdict on a router split's long trial after its K windows (6.80 (1)): undone only if its paired
    /// gain G_real - G_null (minus the mean of the pairs, from blocks) is below -`router_trial_z` block standard
    /// errors; otherwise it stays, whatever its gain.
    fn router_verdict(&mut self, trial: RouterTrial) {
        let g = &mut self.graph;
        let gp = g.params.grow;
        let (mean, se) = if trial.blocks.count >= 2 { trial.blocks.stats() } else { (0.0, 0.0) };
        let gain = -mean;
        // The bar: -z block standard errors (0 for pairs without spread, so no bar is NaN).
        let bar = if se > 0.0 { -gp.router_trial_z * se } else { 0.0 };
        let undo = trial.blocks.count >= 2 && gain < bar;
        let windows: Vec<String> = trial.window_means.iter().map(|m| format!("{m:+.5}")).collect();
        eprintln!(
            "router trial verdict of router {} ({:?} split at N {}, made {:?}) at N {}: {}, G-Gnull {gain:+.7} (block se {se:.7}, z {:+.2}, {} blocks; bar {:.7}), windows [{}]; G (against the unsplit router) {}",
            trial.router,
            trial.kind,
            trial.at,
            trial.made,
            g.step,
            if undo { "UNDONE" } else { "kept" },
            if se > 0.0 { gain / se } else { 0.0 },
            trial.blocks.count,
            bar,
            windows.join(", "),
            describe_gain(&trial.real)
        );
        match undo {
            true => self.undo_router_split(trial),
            false => self.graph.rallow.kept += 1,
        }
        let g = &self.graph;
        eprintln!(
            "router splits {}: kept {}, undone {}, on trial {}",
            g.rallow.splits,
            g.rallow.kept,
            g.rallow.undone,
            g.router_trials.len()
        );
    }

    /// grow6: undoes a router split whose trial failed: the router routes among its old children again (a cluster
    /// split's central router over them; an unsharing's parents point to it again), as the unsplit router trained
    /// beside the split, with its table; the routers the split made go (removed at the next step), and a cluster
    /// split's sibling set leaves the registry with its cells. The router backs off (its back-off doubled) and leaves
    /// the chain.
    fn undo_router_split(&mut self, trial: RouterTrial) {
        let g = &mut self.graph;
        let v = trial.router;
        match trial.kind {
            SplitKind::Cluster => {
                assert_eq!(g.children(v), &trial.made[..], "a cluster split's central router routes among its copies");
                g.nodes[v].kind = Kind::Branch { children: trial.children.clone() };
            }
            SplitKind::Path => {
                for (j, &p) in trial.parents.iter().enumerate().skip(1) {
                    let copy = trial.made[j - 1];
                    let Kind::Branch { children } = &mut g.nodes[p].kind else { unreachable!("a parent is a router") };
                    let slot = children.iter().position(|&c| c == copy).expect("an unsharing's parent points to its copy");
                    children[slot] = v;
                }
            }
        }
        for &m in &trial.made {
            let node = &mut g.nodes[m];
            // Removed: no one's parent until `remove_dead` drops it at the next step.
            (node.kind, node.member, node.rgrow) = (Kind::Leaf, None, RouterGrow::default());
            g.dead.push(m);
        }
        g.post_checks.retain(|c| c.router != v && !c.made.iter().any(|m| trial.made.contains(m)));
        let examples = trial.examples.min(u32::MAX as u64) as u32;
        let node = &mut g.nodes[v];
        node.classifier = trial.shadow;
        node.table = Some(Box::new(trial.table));
        (node.cache_step, node.base_step) = (0, 0);
        node.num_trained_total = examples;
        node.error_total = 0.0;
        node.forget_window();
        node.chain = trial.chain;
        // Its statistics start afresh, so it ages again (`age_windows` closes), and then backs off.
        let backoff = (2 * trial.backoff_windows).max(1);
        let after = g.params.grow.age_windows as u64 + backoff as u64;
        node.grow = GrowStats { backoff_windows: backoff, eligible_after_close: after, ..GrowStats::default() };
        node.rgrow = RouterGrow::default();
        g.open_window(v);
        if let Some(set) = trial.set {
            g.drop_set(set);
        }
        g.split_number -= 1;
        g.rallow.undone += 1;
        g.conform_shadows();
        let upstream = crate::pathctx::upstream_sets(g);
        g.nodes[v].table.as_mut().expect("with path context every node has a path table").conform_to(&upstream[v]);
        g.conform_aux_tables();
        eprintln!(
            "router split undone at N {}: router {v} routes among {:?} again; removed {:?}{}; router {v} backs off {backoff} windows",
            g.step,
            trial.children,
            trial.made,
            trial.set.map_or(String::new(), |s| format!(", set {s} left the registry"))
        );
        if g.params.grow.log_router_stats {
            let report = g.cell_report();
            eprintln!("path cells after the undo at N {}: {report}", g.step);
        }
    }

    /// grow2: a cluster split of router `idx` (growth-spec 2.4): today's split (`split_delegator`); with shadows, the
    /// top router is their cluster router and the copies are the shadows, as trained (no forgetting), each with its
    /// path table plus the cell the split gives the copies (the set `idx` is a member of, if any). Returns the copies.
    fn cluster_split(&mut self, idx: usize, shadows: Option<Box<Shadows>>) -> Vec<usize> {
        let first = self.graph.nodes.len();
        self.split_delegator(idx);
        let made: Vec<usize> = (first..first + self.graph.params.branch_factor).collect();
        if let Some(shadows) = shadows {
            let Shadows { partition: Partition::Cluster { q, .. }, routers, .. } = *shadows else {
                unreachable!("a cluster split's shadows are by cluster")
            };
            let g = &mut self.graph;
            let own = g.nodes[idx].member.map(|(set, _)| set);
            let node = &mut g.nodes[idx];
            node.classifier = q;
            (node.cache_step, node.base_step) = (0, 0);
            for (&m, (router, mut table)) in made.iter().zip(routers) {
                if let Some(own) = own.filter(|own| !table.cells().contains(own)) {
                    table.insert_cell(own);
                }
                let copy = &mut g.nodes[m];
                copy.classifier = router;
                copy.table = Some(Box::new(table));
                (copy.cache_step, copy.base_step) = (0, 0);
            }
        }
        made
    }

    /// grow2: unshares router `idx` by arrival path (growth-spec 2.4): it stays for its first parent's path, and each
    /// other parent gets a new router over the same children, which that parent now points to instead. The routers
    /// are the shadows, as trained (path j's for parent j); without shadows (stall-h), copies of the router forgotten
    /// by `forget_factor`, as a split's copies are. No sibling set is made. Each router now has one parent, and its
    /// path table keeps the cells of the sets still upstream of it (`pathctx::check`); a set that only other parents'
    /// paths passed reads 0 on its paths, and its cell goes. The router starts afresh (its windows, as
    /// `split_delegator`'s). Returns the router and its copies, by parent.
    fn unshare(&mut self, idx: usize, shadows: Option<Box<Shadows>>) -> Vec<usize> {
        let g = &mut self.graph;
        let parents = g.path_parents(idx).expect("an unsharing has several parents");
        let children = g.children(idx).to_vec();
        let mut routers = match shadows {
            Some(shadows) => {
                assert!(matches!(&shadows.partition, Partition::Path { parents: p } if *p == parents), "an unsharing's shadows are by its parents");
                shadows.routers
            }
            None => {
                let node = &g.nodes[idx];
                let table = node.table.as_deref().expect("with path context every node has a path table");
                (0..parents.len())
                    .map(|_| {
                        let (mut router, mut table) = (node.classifier.clone(), table.clone());
                        router.forget(g.params.forget_factor);
                        table.forget(g.params.forget_factor);
                        (router, table)
                    })
                    .collect()
            }
        };
        assert_eq!(routers.len(), parents.len(), "one router per arrival path");
        let (router, table) = routers.remove(0);
        let node = &mut g.nodes[idx];
        node.classifier = router;
        node.table = Some(Box::new(table));
        (node.cache_step, node.base_step) = (0, 0);
        node.num_trained_total = 0;
        node.error_total = 0.0;
        node.forget_window();
        let mut made = vec![idx];
        for (&parent, (router, table)) in parents[1..].iter().zip(routers) {
            let mut copy = g.new_node(Kind::Branch { children: children.clone() }, router);
            copy.table = Some(Box::new(table));
            g.nodes.push(copy);
            let new = g.nodes.len() - 1;
            made.push(new);
            let Kind::Branch { children: theirs } = &mut g.nodes[parent].kind else { unreachable!("a parent is a router") };
            let slot = theirs.iter().position(|&c| c == idx).expect("a parent points to its child");
            theirs[slot] = new;
        }
        let upstream = crate::pathctx::upstream_sets(g);
        let mut dropped = Vec::new();
        for &m in &made {
            let table = g.nodes[m].table.as_deref_mut().expect("with path context every node has a path table");
            let gone: Vec<usize> = table.cells().iter().copied().filter(|set| !upstream[m].contains(set)).collect();
            gone.iter().for_each(|&set| table.remove_cell(set));
            assert_eq!(table.cells(), upstream[m], "an unsharing's router keeps the cells upstream of it");
            dropped.push(gone);
        }
        if dropped.iter().any(|d| !d.is_empty()) {
            eprintln!("router unsharing of router {idx}: parents {parents:?}, routers {made:?}, cells dropped {dropped:?}");
        }
        g.split_number += 1;
        g.unshared = Some((idx, parents, made.clone()));
        made
    }

    /// Turns delegator `idx` into a router over new delegators, copies of
    /// its router with the counts divided by `forget_factor`, that all route
    /// to its old children. With `split_trial`, one more such copy stands for
    /// the delegator without the split until the split's trial ends. The
    /// delegator's own router starts afresh, except with
    /// `SubrouterTarget::Home`: it keeps its router, which goes on learning
    /// its old target, so with `forget_factor` 1 the routing is unchanged at
    /// the split (it picks sub-router i where it picked child i, and every
    /// sub-router, a copy of it, picks child i there).
    fn split_delegator(&mut self, idx: usize) {
        let g = &mut self.graph;
        let Kind::Branch { children } = &g.nodes[idx].kind else { unreachable!() };
        let children = children.clone();
        let first = g.nodes.len();
        for _ in 0..g.params.branch_factor {
            let old = &g.nodes[idx];
            let mut router = old.classifier.clone();
            router.forget(g.params.forget_factor);
            let mut delegator = g.new_node(Kind::Branch { children: children.clone() }, router);
            delegator.error_router = old.error_router.as_ref().map(|e| e.forgetting_copy(g.params.forget_factor));
            delegator.error_routed = old.error_routed;
            g.nodes.push(delegator);
        }
        let made: Vec<usize> = (first..first + g.params.branch_factor).collect();
        let trial = g.params.split_trial.map(|_| {
            let mut shadow = g.nodes[idx].classifier.clone();
            shadow.forget(g.params.forget_factor);
            Trial::new(shadow, Some(children), g.nodes[idx].num_trained_total, made.clone())
        });

        let router = match g.params.subrouter_target {
            SubrouterTarget::Routing => g.new_router(),
            SubrouterTarget::Home => g.nodes[idx].classifier.clone(),
        };
        let error_router = g.new_error_router();
        let fresh_router = g.params.subrouter_target == SubrouterTarget::Routing;
        let node = &mut g.nodes[idx];
        assert!(node.trial.is_none(), "a delegator split during its own trial");
        node.error_router = error_router;
        node.error_routed = false;
        node.kind = Kind::Branch { children: made.clone() };
        node.classifier = router;
        node.cache_step = 0;
        node.base_step = 0;
        node.num_trained_total = 0;
        node.error_total = 0.0;
        node.forget_window();
        node.trial = trial;
        g.split_number += 1;
        if g.params.path_context {
            crate::pathctx::after_router_split(g, idx, &made, fresh_router);
        }
    }

    /// The children shared by all of `children`, if they are delegators made
    /// by the same delegator split.
    fn shared_children(&self, children: &[usize]) -> Option<Vec<usize>> {
        let grandchildren = |c: usize| match &self.graph.nodes[c].kind {
            Kind::Branch { children } => Some(children),
            Kind::Leaf => None,
        };
        let first = grandchildren(children[0])?;
        children[1..].iter().all(|&c| grandchildren(c) == Some(first)).then(|| first.clone())
    }

    /// With `SubrouterTarget::Home`, for delegator `idx` over `children`:
    /// its home children (see `home_base`) if they are not its own children,
    /// that is if its children are sub-routers made by a delegator split.
    /// None otherwise, and without `Home`.
    fn home_children(&mut self, idx: usize, children: &[usize]) -> Option<Vec<usize>> {
        if self.graph.params.subrouter_target != SubrouterTarget::Home {
            return None;
        }
        let key = (self.graph.split_number, self.graph.nodes.len());
        if self.home_cache.key != key {
            let n = self.graph.nodes.len();
            self.home_cache = HomeCache { key, homes: vec![None; n], bases: vec![None; n] };
        }
        if let Some(home) = &self.home_cache.homes[idx] {
            return home.clone();
        }
        let base = self.home_base_cached(idx);
        let home = (base != children).then_some(base);
        self.home_cache.homes[idx] = Some(home.clone());
        home
    }

    /// The children delegator `idx`'s router routes among in effect: if its
    /// children are delegators whose home bases are all the same (sub-routers
    /// made by one delegator split route among the same children, and go on
    /// doing so through later splits of their own), that base; otherwise its
    /// own children. A tree's delegators never share their children, so
    /// their bases differ. Empty for a leaf.
    #[cfg(test)]
    fn home_base(&self, idx: usize) -> Vec<usize> {
        let Kind::Branch { children } = &self.graph.nodes[idx].kind else { return Vec::new() };
        if children.iter().all(|&c| matches!(self.graph.nodes[c].kind, Kind::Branch { .. })) {
            let bases: Vec<Vec<usize>> = children.iter().map(|&c| self.home_base(c)).collect();
            if bases[1..].iter().all(|b| *b == bases[0]) {
                return bases[0].clone();
            }
        }
        children.clone()
    }

    /// `home_base`, kept per structure in the home cache. The same result: the plain recursion walks every path
    /// below the node, which grows exponentially with the depth of chains of sub-routers; this visits each node once.
    fn home_base_cached(&mut self, idx: usize) -> Vec<usize> {
        if let Some(base) = &self.home_cache.bases[idx] {
            return base.clone();
        }
        let base = match &self.graph.nodes[idx].kind {
            Kind::Leaf => Vec::new(),
            Kind::Branch { children } => {
                let children = children.clone();
                let mut base = children.clone();
                if children.iter().all(|&c| matches!(self.graph.nodes[c].kind, Kind::Branch { .. })) {
                    let bases: Vec<Vec<usize>> = children.iter().map(|&c| self.home_base_cached(c)).collect();
                    if bases[1..].iter().all(|b| *b == bases[0]) {
                        base = bases[0].clone();
                    }
                }
                base
            }
        };
        self.home_cache.bases[idx] = Some(base.clone());
        base
    }

    /// For delegators sharing the children `shared`: each one's router error
    /// at predicting the error target of the shared children (see `ErrorTarget`).
    fn routing_errors(
        &mut self,
        children: &[usize],
        shared: &[usize],
        data: &[u8],
        label: &[f64],
        scratch: &mut [f64],
    ) -> Vec<f64> {
        let g = &mut self.graph;
        let errors = g.prediction_errors(shared, data, label, scratch);
        let target = g.error_target(&errors);
        let mut router = [0.0; 256];
        let router = &mut router[..g.params.branch_factor];
        children
            .iter()
            .map(|&child| {
                g.node_output(child, data, router);
                g.router_error(router, &target)
            })
            .collect()
    }

    /// Path context diagnostics (see `pathctx`) on labelled positions, routed as play routes them: the registry; per
    /// node its cells; per leaf the positions it predicts, the share of them that came through a non-home edge (a
    /// copy routing elsewhere than to the child of its own number), the mean absolute change its cells make to its
    /// white-minus-black log-odds, and the size of its cells' weights beside its own; per router with cells, the
    /// share of the positions through it that it routes elsewhere than it would with its cells neutral.
    pub fn path_report(&self, data: &[(Vec<u8>, [f64; 3])]) -> String {
        use std::fmt::Write as _;
        let g = &self.graph;
        assert!(g.params.path_context, "path-stats is for models with path context");
        let n = g.nodes.len();
        let mut s = String::new();
        let cells: Vec<usize> = g.nodes.iter().map(|x| x.table.as_ref().unwrap().cells().len()).collect();
        let edges: usize = (0..n).map(|i| g.children(i).len()).sum();
        writeln!(s, "nodes {n} edges {edges} sibling-sets {} cells {} path-params {}", g.sets.len(), cells.iter().sum::<usize>(), g.path_param_count()).unwrap();
        for (set, members) in g.sets.iter().enumerate() {
            writeln!(s, "set {set} members {members:?}").unwrap();
        }
        // Per node: positions, non-home arrivals, |change in log-odds|, routed elsewhere.
        let (mut seen, mut nonhome, mut shift, mut rerouted) = (vec![0usize; n], vec![0usize; n], vec![0.0; n], vec![0usize; n]);
        // The cells' change to a leaf's white-minus-black log-odds: sum and sum of squares, for its spread over the
        // leaf's positions (the part that differs by path; a constant part only stands in for a bias).
        let (mut shift_sum, mut shift_squares) = (vec![0.0; n], vec![0.0; n]);
        let (mut marks, mut path) = (Vec::new(), Vec::new());
        let mut scores = [0.0; 256];
        let (mut plain, mut values) = ([0.0; 256], Vec::new());
        let (mut undecoded, mut rerouted_positions) = (0, 0);
        for (pos, _) in data {
            path.clear();
            let leaf = g.descend(pos, &mut marks, |x, _| path.push(x));
            // The leaf's values must give back the path play took.
            g.nodes[leaf].table.as_ref().unwrap().values_for(&marks, &mut values);
            undecoded += (crate::pathctx::decode(g, leaf, &values).as_ref() != Ok(&path)) as usize;
            let mut came_nonhome = false;
            for w in path.windows(2) {
                if let Some((_, m)) = g.nodes[w[0]].member {
                    came_nonhome |= g.children(w[0])[m - 1] != w[1];
                }
            }
            // Routers on the path with cells: would they route elsewhere with their cells neutral?
            let mut at = Vec::new();
            g.descend(pos, &mut Vec::new(), |x, m| at.push((x, m.to_vec())));
            let mut any = false;
            for (x, m) in &at {
                seen[*x] += 1;
                let table = g.nodes[*x].table.as_ref().unwrap();
                if *x == leaf || table.cells().is_empty() {
                    continue;
                }
                let k = g.params.branch_factor;
                g.node_predict_path(*x, m, pos, &mut scores[..k], false);
                g.nodes[*x].classifier.predict(pos, &mut plain[..k]);
                let elsewhere = crate::delegation::argmax(&scores[..k]) != crate::delegation::argmax(&plain[..k]);
                rerouted[*x] += elsewhere as usize;
                any |= elsewhere;
            }
            rerouted_positions += any as usize;
            nonhome[leaf] += came_nonhome as usize;
            let table = g.nodes[leaf].table.as_ref().unwrap();
            if !table.cells().is_empty() {
                table.values_for(&marks, &mut values);
                let mut with = [0.0; 3];
                g.node_predict_path(leaf, &marks, pos, &mut with, true);
                g.nodes[leaf].classifier.predict_log(pos, &mut plain[..3]);
                let d = (with[0] - with[1]) - (plain[0] - plain[1]);
                shift[leaf] += d.abs();
                shift_sum[leaf] += d;
                shift_squares[leaf] += d * d;
            }
        }
        for idx in 0..n {
            let node = &g.nodes[idx];
            let table = node.table.as_ref().unwrap();
            let w = table.weights();
            let l2 = w.iter().map(|x| x * x).sum::<f64>().sqrt();
            let max = w.iter().fold(0.0f64, |m, x| m.max(x.abs()));
            let own = match &node.classifier {
                crate::delegation::NodeModel::Lr(lr) => format!("{:.4}", lr.weight_norms().0),
                _ => "-".to_string(),
            };
            let kind = if g.children(idx).is_empty() { "leaf" } else { "router" };
            let member = node.member.map_or("-".to_string(), |(set, m)| format!("{set}.{m}"));
            let k = seen[idx].max(1) as f64;
            write!(s, "node {idx} {kind} member {member} cells {:?} positions {}", table.cells(), seen[idx]).unwrap();
            match kind {
                "leaf" => write!(
                    s,
                    " nonhome {:.4} logodds-shift {:.5} path-spread {:.5} table-l2 {l2:.4} table-max {max:.4} own-l2 {own}",
                    nonhome[idx] as f64 / k,
                    shift[idx] / k,
                    (shift_squares[idx] / k - (shift_sum[idx] / k).powi(2)).max(0.0).sqrt()
                )
                .unwrap(),
                _ => write!(s, " rerouted {:.4} table-l2 {l2:.4} table-max {max:.4} own-l2 {own}", rerouted[idx] as f64 / k).unwrap(),
            }
            s.push('\n');
        }
        let total = data.len().max(1) as f64;
        let leaves: Vec<usize> = (0..n).filter(|&i| g.children(i).is_empty()).collect();
        writeln!(
            s,
            "overall positions {} paths-not-decoded {undecoded} nonhome {:.4} logodds-shift {:.5} path-spread {:.5} rerouted-at-some-router {:.4}",
            data.len(),
            leaves.iter().map(|&l| nonhome[l]).sum::<usize>() as f64 / total,
            leaves.iter().map(|&l| shift[l]).sum::<f64>() / total,
            // Pooled within-leaf standard deviation.
            (leaves.iter().map(|&l| shift_squares[l] - shift_sum[l] * shift_sum[l] / seen[l].max(1) as f64).sum::<f64>() / total).max(0.0).sqrt(),
            rerouted_positions as f64 / total
        )
        .unwrap();
        s
    }

    /// Serializes in the BDT text format.
    pub fn to_text(&self) -> String {
        self.graph.to_text()
    }

    /// Parses the BDT text format.
    pub fn from_text(text: &str, use_probs: bool) -> io::Result<Self> {
        Ok(Bdd { graph: Graph::from_text(text, use_probs)?, home_cache: HomeCache::default() })
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        std::fs::write(path, self.to_text())
    }

    pub fn load(path: &Path, use_probs: bool) -> io::Result<Self> {
        Self::from_text(&std::fs::read_to_string(path)?, use_probs)
    }

    /// The whole training state, unrounded, for an exact `--resume` (see
    /// `state`).
    pub fn to_state(&self) -> Writer {
        let mut w = Writer::new("bdd");
        self.graph.write_state(&mut w);
        w
    }

    /// Reads a state written by `to_state` by a model trained with `params`
    /// over `categories`.
    pub fn from_state(data: &[u8], categories: &[u8], params: DelegationParams) -> io::Result<Self> {
        let mut r = Reader::new(data, "bdd")?;
        let graph = Graph::read_state(&mut r, categories, params)?;
        r.finish()?;
        Ok(Bdd { graph, home_cache: HomeCache::default() })
    }

    pub fn save_state(&self, path: &Path) -> io::Result<()> {
        self.to_state().save(path)
    }

    pub fn load_state(path: &Path, categories: &[u8], params: DelegationParams) -> io::Result<Self> {
        Self::from_state(&std::fs::read(path)?, categories, params)
    }
}

/// grow6: a router trial's gain for the log: minus the pairs' mean, with its block standard error and z, from blocks.
fn describe_gain(b: &Blocks) -> String {
    match b.count >= 2 {
        true => {
            let (mean, se) = b.stats();
            format!("{:+.7} (block se {se:.7}, z {:+.2}, {} blocks)", -mean, if se > 0.0 { -mean / se } else { 0.0 }, b.count)
        }
        false => "none".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::delegation::{ErrorMeasure, ErrorTarget, NodeKind, RouteTarget, SplitRule, SubrouterTarget};
    use crate::state::Writer;

    /// Noisy labels over 4 features, from a fixed linear congruential generator.
    fn noisy(n: usize) -> Vec<([u8; 4], [f64; 2])> {
        let mut x = 12345u64;
        let mut next = |m: u64| {
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (x >> 33) % m
        };
        (0..n)
            .map(|_| {
                let data = [next(3) as u8, next(3) as u8, next(3) as u8, next(3) as u8];
                let p_first = [0.9, 0.5, 0.2][data[0] as usize] * if data[1] == data[2] { 0.5 } else { 1.0 };
                let label = if (next(1000) as f64) < 1000.0 * p_first { [1.0, 0.0] } else { [0.0, 1.0] };
                (data, label)
            })
            .collect()
    }

    /// As BDT's test: a tree read back from its state trains on exactly as
    /// the original.
    #[test]
    fn state_round_trip_trains_on_identically() {
        let base = DelegationParams {
            class_num: 2,
            branch_factor: 2,
            split_threshold: 40,
            split_limit: 6,
            forget_factor: 1.0,
            nb_alpha: 1.0,
            use_probs: true,
            leaf: NodeKind::Cnbp,
            router: NodeKind::Cnbp,
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
            error_router: NodeKind::Cnbp,
            revise: None,
            seam_weight: 0.0,
            reset_leaves: None,
            leaf_residual: false,
            residual_l2: 0.0,
            residual_inherit_sums: false,
            log_fresh_gains: false,
            revise_on: crate::delegation::ReviseOn::Stream,
            residual_delta_rate: 1.0,
            probit: crate::probit::ProbitParams::default(),
            split_noise: 0.0,
            probit_shared_eps: false,
            tan_m: crate::tan::DEFAULT_M,
            subrouter_target: crate::delegation::SubrouterTarget::Routing,
            path_context: false,
            router_split_z: None,
            router_split_limit: None,
            grow: crate::delegation::GrowParams::default(),
        };
        let variants = [
            base.clone(),
            DelegationParams { leaf: NodeKind::Lr, split_trial: Some(0.0), ..base.clone() },
            DelegationParams { route: RouteTarget::Cluster, route_switch: Some(0.0), ..base.clone() },
            DelegationParams { subrouter_target: SubrouterTarget::Home, ..base.clone() },
            DelegationParams { leaf: NodeKind::Lr, subrouter_target: SubrouterTarget::Home, ..base.clone() },
        ];
        for params in variants {
            let mut bdd = Bdd::new(&[3; 4], params.clone());
            for round in crate::delegation::state_test_rounds() {
                let state = bdd.to_state().into_bytes();
                let mut back = Bdd::from_state(&state, &[3; 4], params.clone()).unwrap();
                assert_eq!(back.to_state().into_bytes(), state);
                assert_eq!(back.to_text(), bdd.to_text());
                for tree in [&mut bdd, &mut back] {
                    round.iter().for_each(|(game, label)| game.iter().for_each(|pos| tree.train(pos, label)));
                    tree.refresh();
                }
                assert!(back.to_state().into_bytes() == bdd.to_state().into_bytes(), "{params:?}");
            }
            assert!(bdd.split_count() > 0, "{params:?}");
            assert!(crate::bdt::Bdt::from_state(&bdd.to_state().into_bytes(), &[3; 4], params).is_err());
        }
    }

    #[test]
    fn undone_delegator_split_restores_the_old_routing() {
        // No splits of their own (limit 0): the test makes them.
        let params = DelegationParams {
            class_num: 2,
            branch_factor: 2,
            split_threshold: 1000,
            split_limit: 0,
            forget_factor: 2.0,
            nb_alpha: 1.0,
            use_probs: false,
            leaf: NodeKind::Cnbp,
            router: NodeKind::Cnbp,
            lr_rate: 0.05,
            error: ErrorMeasure::Abs,
            split_z: 0.0,
            split_rule: SplitRule::StallHigh,
            route: RouteTarget::Cluster,
            error_target: ErrorTarget::Best,
            leaf_share: 0.0,
            split_trial: None,
            trial_error: ErrorMeasure::Abs,
            route_switch: None,
            error_router: NodeKind::Cnbp,
            revise: None,
            seam_weight: 0.0,
            reset_leaves: None,
            leaf_residual: false,
            residual_l2: 0.0,
            residual_inherit_sums: false,
            log_fresh_gains: false,
            revise_on: crate::delegation::ReviseOn::Stream,
            residual_delta_rate: 1.0,
            probit: crate::probit::ProbitParams::default(),
            split_noise: 0.0,
            probit_shared_eps: false,
            tan_m: crate::tan::DEFAULT_M,
            subrouter_target: crate::delegation::SubrouterTarget::Routing,
            path_context: false,
            router_split_z: None,
            router_split_limit: None,
            grow: crate::delegation::GrowParams::default(),
        };
        let data = noisy(4000);
        let mut bdd = Bdd::new(&[3, 3, 3, 3], params);
        bdd.graph.split_leaf(0);
        data[..1000].iter().for_each(|(d, l)| bdd.train(d, l));
        // A delegator split whose trial cannot pass, then a split below it
        // that is not on trial, numbered after the doomed nodes 3 and 4.
        bdd.graph.params.split_trial = Some(f64::INFINITY);
        bdd.split_delegator(0);
        bdd.graph.params.split_trial = None;
        bdd.graph.split_leaf(2);
        let mut before = Bdd::from_text(&bdd.to_text(), false).unwrap();
        data[1000..].iter().for_each(|(d, l)| bdd.train(d, l));
        bdd.refresh();

        let children = |bdd: &Bdd, idx: usize| match &bdd.graph.nodes[idx].kind {
            Kind::Branch { children } => children.clone(),
            Kind::Leaf => vec![],
        };
        assert_eq!(bdd.node_count(), 5);
        assert_eq!(bdd.split_count(), 2);
        assert_eq!((children(&bdd, 0), children(&bdd, 2)), (vec![1, 2], vec![3, 4]));
        assert!(children(&bdd, 3).is_empty() && children(&bdd, 4).is_empty());
        let back = Bdd::from_text(&bdd.to_text(), false).unwrap();
        assert_eq!(bdd.to_text(), back.to_text());
        // Before the trial ended, the delegator split was in place.
        before.refresh();
        assert_eq!((before.node_count(), children(&before, 0)), (7, vec![3, 4]));
    }

    #[test]
    fn delegators_split_and_share_children() {
        let params = DelegationParams {
            class_num: 2,
            branch_factor: 2,
            split_threshold: 50,
            split_limit: 12,
            forget_factor: 10.0,
            nb_alpha: 1.0,
            use_probs: false,
            leaf: NodeKind::Cnbp,
            router: NodeKind::Cnbp,
            lr_rate: 0.05,
            error: ErrorMeasure::Abs,
            split_z: 0.0,
            split_rule: SplitRule::StallHigh,
            route: RouteTarget::Error,
            error_target: ErrorTarget::Best,
            leaf_share: 0.0,
            split_trial: None,
            trial_error: ErrorMeasure::Abs,
            route_switch: None,
            error_router: NodeKind::Cnbp,
            revise: None,
            seam_weight: 0.0,
            reset_leaves: None,
            leaf_residual: false,
            residual_l2: 0.0,
            residual_inherit_sums: false,
            log_fresh_gains: false,
            revise_on: crate::delegation::ReviseOn::Stream,
            residual_delta_rate: 1.0,
            probit: crate::probit::ProbitParams::default(),
            split_noise: 0.0,
            probit_shared_eps: false,
            tan_m: crate::tan::DEFAULT_M,
            subrouter_target: crate::delegation::SubrouterTarget::Routing,
            path_context: false,
            router_split_z: None,
            router_split_limit: None,
            grow: crate::delegation::GrowParams::default(),
        };
        let mut bdd = Bdd::new(&[3, 3, 3, 3], params);
        for (data, label) in noisy(20_000) {
            bdd.train(&data, &label);
        }
        assert_eq!(bdd.node_count(), 1 + 2 * bdd.split_count() as usize);
        let mut parents = vec![0; bdd.node_count()];
        for node in &bdd.graph.nodes {
            if let Kind::Branch { children } = &node.kind {
                children.iter().for_each(|&c| parents[c] += 1);
            }
        }
        assert!(parents.iter().any(|&p| p > 1), "no delegator split: {parents:?}");

        let back = Bdd::from_text(&bdd.to_text(), false).unwrap();
        assert_eq!(bdd.to_text(), back.to_text());
        let (mut a, mut b) = ([0.0; 2], [0.0; 2]);
        bdd.predict(&[1, 2, 2, 0], &mut a);
        back.predict(&[1, 2, 2, 0], &mut b);
        assert_eq!(a, b);
        assert!((a[0] + a[1] - 1.0).abs() < 1e-12);
    }
    fn home_params(leaf: NodeKind, branch_factor: usize, split_threshold: u32, split_limit: u32, forget_factor: f64) -> DelegationParams {
        DelegationParams {
            class_num: 2,
            branch_factor,
            split_threshold,
            split_limit,
            forget_factor,
            nb_alpha: 1.0,
            use_probs: false,
            leaf,
            router: NodeKind::Cnbp,
            lr_rate: 0.05,
            error: ErrorMeasure::Log,
            split_z: 0.0,
            split_rule: SplitRule::Worse,
            route: RouteTarget::Error,
            error_target: ErrorTarget::Best,
            leaf_share: 0.0,
            split_trial: None,
            trial_error: ErrorMeasure::Winner,
            route_switch: None,
            error_router: NodeKind::Cnbp,
            revise: None,
            seam_weight: 0.0,
            reset_leaves: None,
            leaf_residual: false,
            residual_l2: 0.0,
            residual_inherit_sums: false,
            log_fresh_gains: false,
            revise_on: crate::delegation::ReviseOn::Stream,
            residual_delta_rate: 1.0,
            probit: crate::probit::ProbitParams::default(),
            split_noise: 0.0,
            probit_shared_eps: false,
            tan_m: crate::tan::DEFAULT_M,
            subrouter_target: SubrouterTarget::Home,
            path_context: false,
            router_split_z: None,
            router_split_limit: None,
            grow: crate::delegation::GrowParams::default(),
        }
    }

    /// Every position of 4 features with 3 values each.
    fn all_positions() -> Vec<[u8; 4]> {
        (0..81u8).map(|i| [i % 3, i / 3 % 3, i / 9 % 3, i / 27]).collect()
    }

    fn state_of(model: &NodeModel) -> Vec<u8> {
        let mut w = Writer::new("bdd");
        model.write_state(&mut w);
        w.into_bytes()
    }

    /// A root that split as a leaf and trained, so its router and leaves differ.
    fn trained_root(leaf: NodeKind, target: SubrouterTarget) -> Bdd {
        let params = DelegationParams { subrouter_target: target, ..home_params(leaf, 3, 1_000_000, 0, 1.0) };
        let mut bdd = Bdd::new(&[3; 4], params);
        bdd.graph.split_leaf(0);
        noisy(6000).iter().for_each(|(d, l)| bdd.train(d, l));
        bdd.refresh();
        bdd
    }

    /// With home targets and forget factor 1, a delegator split changes
    /// nothing at once: the delegator keeps its router, and each sub-router,
    /// a copy of it, picks the child it would have picked.
    #[test]
    fn home_split_keeps_the_routing() {
        for leaf in [NodeKind::Cnbp, NodeKind::Lr] {
            let mut bdd = trained_root(leaf, SubrouterTarget::Home);
            let positions = all_positions();
            let predict = |bdd: &Bdd| -> Vec<(usize, [f64; 2])> {
                positions
                    .iter()
                    .map(|p| {
                        let mut out = [0.0; 2];
                        bdd.predict(p, &mut out);
                        (bdd.leaf(p), out)
                    })
                    .collect()
            };
            let before = predict(&bdd);
            let router_before = state_of(&bdd.graph.nodes[0].classifier);
            let leaves_used: std::collections::BTreeSet<usize> = before.iter().map(|(l, _)| *l).collect();
            assert!(leaves_used.len() > 1, "the test needs a root that routes: {leaves_used:?}");
            bdd.split_delegator(0);
            bdd.refresh();
            assert_eq!(bdd.node_count(), 7);
            assert_eq!(state_of(&bdd.graph.nodes[0].classifier), router_before, "the delegator keeps its router");
            assert_eq!(predict(&bdd), before, "{leaf:?}");
            // Each sub-router is picked where the old router picked its home.
            let Kind::Branch { children } = &bdd.graph.nodes[0].kind else { unreachable!() };
            let children = children.clone();
            let Kind::Branch { children: home } = &bdd.graph.nodes[children[0]].kind else { unreachable!() };
            let home = home.clone();
            let mut picks = [0usize; 3];
            for p in &positions {
                let mut scores = [0.0; 3];
                bdd.graph.nodes[0].classifier.predict(p, &mut scores);
                let pick = crate::delegation::argmax(&scores);
                picks[pick] += 1;
                assert_eq!(bdd.leaf(p), home[pick]);
                assert_eq!(bdd.graph.leaf_from(children[pick], p), home[pick]);
            }
            assert!(picks.iter().filter(|&&n| n > 0).count() > 1, "{picks:?}");
        }
    }

    /// After a home split, the delegator learns exactly what its router
    /// learned before: the error target of the shared children, not of the
    /// sub-routers.
    #[test]
    fn home_targets_come_from_the_shared_children() {
        for leaf in [NodeKind::Cnbp, NodeKind::Lr] {
            let mut bdd = trained_root(leaf, SubrouterTarget::Home);
            bdd.split_delegator(0);
            let Kind::Branch { children } = &bdd.graph.nodes[0].kind else { unreachable!() };
            let first_copy = children[0];
            let Kind::Branch { children: shared } = &bdd.graph.nodes[first_copy].kind else { unreachable!() };
            let shared = shared.clone();
            let mut scratch = vec![0.0; 2];
            let mut picked = [0usize; 3];
            for (data, label) in noisy(3000) {
                let mut expected = bdd.clone();
                expected.graph.begin_step(&data, &label, &mut scratch);
                let errors = expected.graph.prediction_errors(&shared, &data, &label, &mut scratch);
                let (pick, _) = expected.graph.train_router(0, &data, &errors);
                picked[pick] += 1;
                bdd.train(&data, &label);
                assert_eq!(
                    state_of(&bdd.graph.nodes[0].classifier),
                    state_of(&expected.graph.nodes[0].classifier),
                    "{leaf:?}"
                );
            }
            // It goes on using more than one sub-router (the old target is
            // not a tie between identical copies).
            assert!(picked.iter().filter(|&&n| n > 0).count() > 1, "{picked:?}");
        }
    }

    /// Splits of every kind with home targets, as `delegators_split_and_share_children`.
    #[test]
    fn home_delegators_split_and_share_children() {
        let mut bdd = Bdd::new(&[3, 3, 3, 3], home_params(NodeKind::Cnbp, 2, 50, 12, 10.0));
        for (data, label) in noisy(20_000) {
            bdd.train(&data, &label);
        }
        bdd.refresh();
        assert_eq!(bdd.node_count(), 1 + 2 * bdd.split_count() as usize);
        let mut parents = vec![0; bdd.node_count()];
        for node in &bdd.graph.nodes {
            if let Kind::Branch { children } = &node.kind {
                children.iter().for_each(|&c| parents[c] += 1);
            }
        }
        assert!(parents.iter().any(|&p| p > 1), "no delegator split: {parents:?}");
        // Every delegator over sub-routers has home children, and the cache
        // gives what working them out afresh gives.
        let delegators: Vec<(usize, Vec<usize>)> = (0..bdd.node_count())
            .filter_map(|i| match &bdd.graph.nodes[i].kind {
                Kind::Branch { children } => Some((i, children.clone())),
                Kind::Leaf => None,
            })
            .collect();
        let mut over_subrouters = 0;
        for (i, children) in &delegators {
            let cached = bdd.home_children(*i, children);
            let fresh = bdd.home_base(*i);
            assert_eq!(cached, (fresh != *children).then_some(fresh));
            over_subrouters += cached.is_some() as usize;
        }
        assert!(over_subrouters > 0);
        // Model files keep 6 decimals of the forgotten counts: the same
        // leaves, and predictions equal to rounding.
        let back = Bdd::from_text(&bdd.to_text(), false).unwrap();
        assert_eq!(bdd.to_text(), back.to_text());
        for p in all_positions() {
            let (mut a, mut b) = ([0.0; 2], [0.0; 2]);
            bdd.predict(&p, &mut a);
            back.predict(&p, &mut b);
            assert_eq!(bdd.leaf(&p), back.leaf(&p));
            assert!((a[0] - b[0]).abs() < 1e-9, "{a:?} {b:?}");
        }
    }

    /// The default (`Routing`) still starts the split delegator's router afresh.
    #[test]
    fn routing_split_starts_a_new_router() {
        let mut bdd = trained_root(NodeKind::Lr, SubrouterTarget::Routing);
        let router_before = state_of(&bdd.graph.nodes[0].classifier);
        bdd.split_delegator(0);
        assert_ne!(state_of(&bdd.graph.nodes[0].classifier), router_before);
        let children = bdd.graph.children(0).to_vec();
        assert_eq!(bdd.home_children(0, &children), None);
    }

    #[test]
    #[should_panic(expected = "subrouter_target home is for error routing")]
    fn home_needs_error_routing() {
        let params = DelegationParams { route: RouteTarget::Cluster, ..home_params(NodeKind::Lr, 3, 100, 2, 1.0) };
        Bdd::new(&[3; 4], params);
    }

    #[test]
    #[should_panic(expected = "subrouter_target is for BDDs")]
    fn bdt_refuses_home() {
        crate::bdt::Bdt::new(&[3; 4], home_params(NodeKind::Lr, 3, 100, 2, 1.0));
    }

    // ---- Path context (see `pathctx`).

    /// A fixed generator for the random growth sequences.
    struct Lcg(u64);

    impl Lcg {
        fn next(&mut self, m: u64) -> u64 {
            self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (self.0 >> 33) % m
        }
    }

    /// Path-context parameters: branch factor 3, error routing by log-loss, forget factor 1, no automatic splits
    /// (the tests make them), counts without cached probabilities.
    fn ctx_params(leaf: NodeKind, router: NodeKind, target: SubrouterTarget) -> DelegationParams {
        DelegationParams {
            router,
            subrouter_target: target,
            path_context: true,
            router_split_z: None,
            router_split_limit: None,
            ..home_params(leaf, 3, 1_000_000, 0, 1.0)
        }
    }

    /// Every combination the option supports, for the growth tests.
    fn ctx_variants() -> Vec<DelegationParams> {
        let mut variants = Vec::new();
        for target in [SubrouterTarget::Home, SubrouterTarget::Routing] {
            for (leaf, router) in [(NodeKind::Lr, NodeKind::Cnbp), (NodeKind::Lr, NodeKind::Lr), (NodeKind::Cnbp, NodeKind::Cnbp)] {
                variants.push(ctx_params(leaf, router, target));
            }
        }
        variants
    }

    /// Each position's leaf and the bits of `predict` and `predict_log`, as play computes them.
    fn snapshot(bdd: &Bdd) -> Vec<(usize, Vec<u64>)> {
        all_positions()
            .iter()
            .map(|p| {
                let (mut a, mut b) = ([0.0; 2], [0.0; 2]);
                bdd.predict(p, &mut a);
                bdd.predict_log(p, &mut b);
                (bdd.leaf(p), a.iter().chain(&b).map(|x| x.to_bits()).collect())
            })
            .collect()
    }

    /// The structural checks: every node's cells are the sets upstream of it, worked out afresh from the structure
    /// (in registry order), the registry's member lists are as they were made, and every position's path in play
    /// decodes from its leaf's values to the node sequence play routed it by.
    fn check_structure(bdd: &Bdd, registry: &mut Vec<Vec<usize>>) {
        let g = &bdd.graph;
        let upstream = crate::pathctx::upstream_sets(g);
        for (idx, node) in g.nodes.iter().enumerate() {
            assert_eq!(node.table.as_ref().unwrap().cells(), upstream[idx].as_slice(), "node {idx}'s cells");
        }
        assert!(g.sets.len() >= registry.len() && g.sets[..registry.len()] == registry[..], "registry entries changed");
        *registry = g.sets.clone();
        for (set, members) in g.sets.iter().enumerate() {
            for (j, &m) in members.iter().enumerate() {
                assert_eq!(g.nodes[m].member, Some((set, j + 1)));
            }
        }
        let mut marks = Vec::new();
        for p in all_positions() {
            let mut nodes = Vec::new();
            let leaf = g.descend(&p, &mut marks, |n, _| nodes.push(n));
            let mut values = Vec::new();
            g.nodes[leaf].table.as_ref().unwrap().values_for(&marks, &mut values);
            assert_eq!(crate::pathctx::decode(g, leaf, &values).unwrap(), nodes, "play path of {p:?}");
        }
    }

    /// A random growth sequence: `splits` times, trains on `per_split` positions, then splits a random leaf or
    /// router, and calls `check` with the model before and after the split. Returns the model.
    fn grow(params: DelegationParams, seed: u64, splits: usize, per_split: usize, mut check: impl FnMut(&Bdd, &Bdd)) -> Bdd {
        let mut rng = Lcg(seed);
        let data = noisy(per_split * splits);
        let mut bdd = Bdd::new(&[3; 4], params);
        for batch in data.chunks(per_split) {
            batch.iter().for_each(|(d, l)| bdd.train(d, l));
            bdd.refresh();
            let before = bdd.clone();
            let leaves: Vec<usize> = (0..bdd.node_count()).filter(|&i| bdd.graph.children(i).is_empty()).collect();
            let routers: Vec<usize> = (0..bdd.node_count()).filter(|&i| !bdd.graph.children(i).is_empty()).collect();
            if routers.is_empty() || rng.next(2) == 0 {
                bdd.graph.split_leaf(leaves[rng.next(leaves.len() as u64) as usize]);
            } else {
                bdd.split_delegator(routers[rng.next(routers.len() as u64) as usize]);
            }
            bdd.refresh();
            check(&before, &bdd);
        }
        bdd
    }

    /// No prediction changes at any split, under both sub-router targets and every supported node kind; the cells
    /// are always the sets upstream; registry entries never change; paths decode in play and in training.
    #[test]
    fn path_context_splits_change_no_prediction() {
        let checked_before = PATHS_CHECKED.with(|n| *n.borrow());
        for (v, params) in ctx_variants().into_iter().enumerate() {
            let mut registry = Vec::new();
            let mut tables_moved = false;
            let bdd = grow(params.clone(), 11 + v as u64, 30, 150, |before, after| {
                // The same predictions; a position keeps its leaf, or its leaf split and it went to a new leaf below.
                for ((leaf, bits), (leaf_after, bits_after)) in snapshot(before).into_iter().zip(snapshot(after)) {
                    assert_eq!(bits, bits_after, "{params:?}");
                    assert!(leaf == leaf_after || after.graph.children(leaf).contains(&leaf_after), "{params:?}");
                }
                check_structure(after, &mut registry);
                tables_moved |= after.graph.nodes.iter().any(|n| n.table.as_ref().unwrap().weights().iter().any(|&w| w != 0.0));
            });
            assert!(bdd.graph.sets.len() >= 5, "too few router splits to test: {:?}", bdd.graph.sets);
            assert!(tables_moved, "the path tables never trained: {params:?}");
        }
        assert!(PATHS_CHECKED.with(|n| *n.borrow()) > checked_before + 1000, "training paths were not checked");
    }

    /// The user's nested case: P2, a member of S1 = {P1, P2, P3}, splits. It stays in S1 as the new central router
    /// (same node id, same sibling number), its copies form S2 below it, and a leaf below both reads S1=1, S2=0
    /// through P1; S1=2, S2=2 through P2 and its second copy; S1=3, S2=0 through P3.
    #[test]
    fn path_context_nested_member_split() {
        for target in [SubrouterTarget::Home, SubrouterTarget::Routing] {
            let mut bdd = Bdd::new(&[3; 4], ctx_params(NodeKind::Lr, NodeKind::Cnbp, target));
            bdd.graph.split_leaf(0); // root router over the leaves 1-3
            noisy(3000).iter().for_each(|(d, l)| bdd.train(d, l));
            bdd.split_delegator(0); // S1 = {P1, P2, P3} = nodes 4-6, each over the leaves
            let (p1, p2, p3) = (4, 5, 6);
            assert_eq!(bdd.graph.sets, vec![vec![p1, p2, p3]]);
            noisy(6000)[3000..].iter().for_each(|(d, l)| bdd.train(d, l));
            bdd.refresh();
            let leaf_values = |bdd: &Bdd| -> Vec<(Vec<usize>, Vec<usize>)> {
                let mut marks = Vec::new();
                all_positions()
                    .iter()
                    .map(|p| {
                        let mut nodes = Vec::new();
                        let leaf = bdd.graph.descend(p, &mut marks, |n, _| nodes.push(n));
                        let mut values = Vec::new();
                        bdd.graph.nodes[leaf].table.as_ref().unwrap().values_for(&marks, &mut values);
                        (nodes, values)
                    })
                    .collect()
            };
            let before = leaf_values(&bdd);
            let predictions = snapshot(&bdd);
            bdd.split_delegator(p2); // S2 = {Q1, Q2, Q3} = nodes 7-9 below P2
            bdd.refresh();
            let (q1, q2, q3) = (7, 8, 9);
            assert_eq!(bdd.graph.sets, vec![vec![p1, p2, p3], vec![q1, q2, q3]], "S1 keeps P2 in its place");
            assert_eq!(bdd.graph.nodes[p2].member, Some((0, 2)));
            assert_eq!(snapshot(&bdd), predictions, "{target:?}");
            let cells = |n: usize| bdd.graph.nodes[n].table.as_ref().unwrap().cells().to_vec();
            assert_eq!(cells(p2), Vec::<usize>::new(), "P2 gets no S2 cell, and none for its own set");
            for q in [q1, q2, q3] {
                assert_eq!(cells(q), vec![0], "the copies get S1 (P2 is their parent), not their own set");
            }
            for leaf in 1..=3 {
                assert_eq!(cells(leaf), vec![0, 1], "the leaves below get both");
            }
            // The values the user listed, from the marks of each route.
            let values = |marks: &[(usize, usize)]| {
                let mut v = Vec::new();
                bdd.graph.nodes[1].table.as_ref().unwrap().values_for(marks, &mut v);
                v
            };
            assert_eq!(values(&[(0, 1)]), vec![1, 0]);
            assert_eq!(values(&[(0, 2), (1, 2)]), vec![2, 2]);
            assert_eq!(values(&[(0, 3)]), vec![3, 0]);
            // In play: P2's paths keep S1 = 2, now through a copy (S2 = its number); P1's and P3's read S2 = 0.
            let after = leaf_values(&bdd);
            let mut through = [0; 3];
            for ((nodes_before, v_before), (nodes, v)) in before.iter().zip(&after) {
                assert_eq!(v_before.len(), 1);
                assert_eq!(v[0], v_before[0], "the S1 value is unchanged across the split");
                match nodes[1] {
                    n if n == p2 => {
                        assert_eq!(v[1], nodes[2] - q1 + 1, "S2 names the copy");
                        assert_eq!(&nodes_before[..2], &nodes[..2]);
                        through[1] += 1;
                    }
                    n => {
                        assert_eq!(v[1], 0, "S2 reads 0 through P1 and P3");
                        assert_eq!(nodes_before, nodes);
                        through[if n == p1 { 0 } else { 2 }] += 1;
                    }
                }
            }
            // Home targets keep the old router, which used every leaf; a fresh router (routing targets) may not.
            assert!(target == SubrouterTarget::Routing || (through[1] > 0 && through[0] + through[2] > 0), "{through:?}");
            println!("nested split, {target:?}: positions through P1, P2, P3: {through:?}");
            check_structure(&bdd, &mut Vec::new());
        }
    }

    /// Two runs on the same data, with automatic splits, give byte-identical models and states.
    #[test]
    fn path_context_is_deterministic() {
        for target in [SubrouterTarget::Home, SubrouterTarget::Routing] {
            let params = DelegationParams { split_threshold: 400, split_limit: 12, ..ctx_params(NodeKind::Lr, NodeKind::Cnbp, target) };
            let run = || {
                let mut bdd = Bdd::new(&[3; 4], params.clone());
                noisy(30_000).iter().for_each(|(d, l)| bdd.train(d, l));
                bdd.refresh();
                bdd
            };
            let (a, b) = (run(), run());
            assert!(a.graph.sets.len() >= 2, "{target:?}: too few router splits: {:?}", a.graph.sets);
            assert_eq!(a.to_text(), b.to_text());
            assert_eq!(a.to_state().into_bytes(), b.to_state().into_bytes());
        }
    }

    /// With the path tables frozen at their neutral values, path context trains exactly as without it: the same
    /// splits, routers and leaves, bit for bit. This checks the plumbing (paths in training and scoring) apart from
    /// the tables.
    #[test]
    fn path_context_with_frozen_tables_trains_as_without() {
        for target in [SubrouterTarget::Home, SubrouterTarget::Routing] {
            for (leaf, router) in [(NodeKind::Lr, NodeKind::Cnbp), (NodeKind::Lr, NodeKind::Lr)] {
                let with = DelegationParams { split_threshold: 400, split_limit: 12, ..ctx_params(leaf, router, target) };
                let without = DelegationParams { path_context: false, ..with.clone() };
                let data = noisy(30_000);
                crate::delegation::FROZEN_TABLES.with(|f| f.set(true));
                let mut a = Bdd::new(&[3; 4], with);
                data.iter().for_each(|(d, l)| a.train(d, l));
                crate::delegation::FROZEN_TABLES.with(|f| f.set(false));
                let mut b = Bdd::new(&[3; 4], without);
                data.iter().for_each(|(d, l)| b.train(d, l));
                assert!(a.graph.sets.len() >= 2, "too few router splits: {:?}", a.graph.sets);
                let mut plain = a.clone();
                plain.graph.params.path_context = false;
                assert_eq!(plain.to_text(), b.to_text(), "{target:?} {leaf:?} {router:?}");
                assert_eq!(a.split_count(), b.split_count());
            }
        }
    }

    /// Model and state files round-trip; a state read back trains on exactly as the original; old readers refuse.
    #[test]
    fn path_context_files_round_trip() {
        for params in ctx_variants() {
            let bdd = grow(params.clone(), 5, 12, 150, |_, _| ());
            let text = bdd.to_text();
            assert!(text.starts_with("pathcontext 1 "));
            let back = Bdd::from_text(&text, false).unwrap();
            assert_eq!(back.to_text(), text);
            check_structure(&back, &mut Vec::new());
            if params.leaf == NodeKind::Lr && params.router == NodeKind::Lr {
                // Logistic regressions are saved at full precision: the same predictions.
                assert_eq!(snapshot(&back), snapshot(&bdd));
            }
            let state = bdd.to_state().into_bytes();
            let mut resumed = Bdd::from_state(&state, &[3; 4], params.clone()).unwrap();
            assert_eq!(resumed.to_state().into_bytes(), state);
            let mut original = bdd.clone();
            for model in [&mut original, &mut resumed] {
                noisy(2000).iter().for_each(|(d, l)| model.train(d, l));
                model.refresh();
            }
            assert!(original.to_state().into_bytes() == resumed.to_state().into_bytes(), "{params:?}");
            // A run without the option refuses the state.
            assert!(Bdd::from_state(&state, &[3; 4], DelegationParams { path_context: false, ..params }).is_err());
        }
    }

    /// The parameter count in long random growth: printed for the write-up (run with --nocapture).
    #[test]
    fn path_context_parameter_growth() {
        for target in [SubrouterTarget::Home, SubrouterTarget::Routing] {
            let mut rows = Vec::new();
            let mut n = 0;
            let bdd = grow(ctx_params(NodeKind::Lr, NodeKind::Cnbp, target), 3, 200, 10, |_, after| {
                n += 1;
                if n % 25 == 0 {
                    let g = &after.graph;
                    let cells: usize = g.nodes.iter().map(|x| x.table.as_ref().unwrap().cells().len()).sum();
                    let max = g.nodes.iter().map(|x| x.table.as_ref().unwrap().cells().len()).max().unwrap();
                    let edges: usize = (0..g.nodes.len()).map(|i| g.children(i).len()).sum();
                    rows.push((n, g.nodes.len(), edges, g.sets.len(), cells, max, g.path_param_count()));
                }
            });
            println!("path context growth, {target:?}: splits, nodes, edges, sibling sets, cells, most cells at a node, parameters");
            for r in &rows {
                println!("  {:4} {:5} {:5} {:4} {:6} {:4} {:7}", r.0, r.1, r.2, r.3, r.4, r.5, r.6);
            }
            check_structure(&bdd, &mut Vec::new());
        }
    }

    #[test]
    #[should_panic(expected = "path_context is for BDDs")]
    fn bdt_refuses_path_context() {
        let params = DelegationParams { path_context: true, subrouter_target: SubrouterTarget::Routing, ..home_params(NodeKind::Lr, 3, 100, 2, 1.0) };
        crate::bdt::Bdt::new(&[3; 4], params);
    }

    #[test]
    #[should_panic(expected = "path_context refuses split trials")]
    fn path_context_refuses_split_trials() {
        let params = DelegationParams { split_trial: Some(0.0), ..ctx_params(NodeKind::Lr, NodeKind::Cnbp, SubrouterTarget::Routing) };
        Bdd::new(&[3; 4], params);
    }

    #[test]
    #[should_panic(expected = "path_context is for error routing")]
    fn path_context_refuses_cluster_routing() {
        let params = DelegationParams { route: RouteTarget::Cluster, ..ctx_params(NodeKind::Lr, NodeKind::Cnbp, SubrouterTarget::Routing) };
        Bdd::new(&[3; 4], params);
    }

    #[test]
    #[should_panic(expected = "path_context is for logistic-regression and CNBP")]
    fn path_context_refuses_tan_routers() {
        Bdd::new(&[3; 4], ctx_params(NodeKind::Lr, NodeKind::Tan, SubrouterTarget::Home));
    }

    /// errdag: with split_z 0, router_split_z 0 trains exactly as the stall rule (the same model, bit for bit), with
    /// and without path context and under both sub-router targets; an infinite one makes no router split (no shared
    /// children, no sibling sets) while leaves still split.
    #[test]
    fn router_split_z_zero_is_the_stall_rule_and_inf_never_splits_routers() {
        let shared = |bdd: &Bdd| {
            let mut parents = vec![0; bdd.node_count()];
            for node in &bdd.graph.nodes {
                if let Kind::Branch { children } = &node.kind {
                    children.iter().for_each(|&c| parents[c] += 1);
                }
            }
            parents.iter().filter(|&&p| p > 1).count()
        };
        let bases = [
            DelegationParams {
                subrouter_target: SubrouterTarget::Routing,
                split_rule: SplitRule::StallHigh,
                error: ErrorMeasure::Abs,
                ..home_params(NodeKind::Cnbp, 2, 50, 12, 10.0)
            },
            DelegationParams { split_threshold: 400, split_limit: 12, ..ctx_params(NodeKind::Lr, NodeKind::Cnbp, SubrouterTarget::Routing) },
            DelegationParams { split_threshold: 400, split_limit: 12, ..ctx_params(NodeKind::Lr, NodeKind::Cnbp, SubrouterTarget::Home) },
        ];
        for base in bases {
            let n = if base.path_context { 30_000 } else { 20_000 };
            let train = |params: DelegationParams| {
                let mut bdd = Bdd::new(&[3; 4], params);
                noisy(n).iter().for_each(|(d, l)| bdd.train(d, l));
                bdd.refresh();
                bdd
            };
            let plain = train(base.clone());
            let zero = train(DelegationParams { router_split_z: Some(0.0), ..base.clone() });
            assert_eq!(plain.to_text(), zero.to_text(), "{base:?}");
            assert!(shared(&plain) > 0, "no router split: {base:?}");
            let never = train(DelegationParams { router_split_z: Some(f64::INFINITY), ..base.clone() });
            assert_eq!(shared(&never), 0, "{base:?}");
            assert!(never.split_count() > 0, "{base:?}");
            assert!(never.graph.sets.is_empty(), "{base:?}");
        }
    }

    #[test]
    #[should_panic(expected = "router_split_z is for BDDs")]
    fn bdt_refuses_router_split_z() {
        let params = DelegationParams { router_split_z: Some(2.0), subrouter_target: SubrouterTarget::Routing, ..home_params(NodeKind::Lr, 3, 100, 2, 1.0) };
        crate::bdt::Bdt::new(&[3; 4], params);
    }

    /// errdag2: with router_split_limit, split_limit counts leaf splits only, router splits stop at the cap and
    /// never outnumber the leaf splits made (checked as training goes), and at router_split_z inf the BDD makes the
    /// same splits and predictions as without the option.
    #[test]
    fn router_split_limit_is_the_tree_plus_router_splits() {
        let bases = [
            DelegationParams {
                subrouter_target: SubrouterTarget::Routing,
                split_rule: SplitRule::StallHigh,
                error: ErrorMeasure::Abs,
                ..home_params(NodeKind::Cnbp, 2, 50, 12, 10.0)
            },
            DelegationParams { split_threshold: 400, split_limit: 6, ..ctx_params(NodeKind::Lr, NodeKind::Cnbp, SubrouterTarget::Routing) },
            DelegationParams { split_threshold: 400, split_limit: 6, ..ctx_params(NodeKind::Lr, NodeKind::Cnbp, SubrouterTarget::Home) },
        ];
        for base in bases {
            let data = noisy(if base.path_context { 30_000 } else { 20_000 });
            let train = |params: DelegationParams, cap: Option<u32>| {
                let mut bdd = Bdd::new(&[3; 4], params);
                for (i, (d, l)) in data.iter().enumerate() {
                    bdd.train(d, l);
                    if let (Some(cap), 0) = (cap, i % 500) {
                        let (leaf, router) = bdd.graph.split_counts();
                        assert!(router <= cap && router <= leaf && leaf <= bdd.graph.params.split_limit, "{leaf} {router}");
                    }
                }
                bdd.refresh();
                bdd
            };
            let free = train(base.clone(), None);
            let (free_leaf, free_router) = free.graph.split_counts();
            assert!(free_router > 0 && free_leaf + free_router == base.split_limit, "{free_leaf} {free_router} {base:?}");
            for cap in [1, 2] {
                let capped = train(DelegationParams { router_split_limit: Some(cap), ..base.clone() }, Some(cap));
                let (leaf, router) = capped.graph.split_counts();
                assert_eq!(leaf, base.split_limit, "the leaves keep the whole budget: {base:?}");
                assert!(router >= 1 && router <= cap, "{router} {base:?}");
                assert_eq!(capped.split_count(), leaf + router);
                assert_eq!(capped.graph.sets.len(), if base.path_context { router as usize } else { 0 });
            }
            let inf = |cap| DelegationParams { router_split_z: Some(f64::INFINITY), router_split_limit: cap, ..base.clone() };
            let (tree, with) = (train(inf(None), None), train(inf(Some(4)), Some(4)));
            assert_eq!(with.graph.split_counts(), (tree.split_count(), 0));
            assert_eq!(with.split_count(), tree.split_count());
            let kinds = |b: &Bdd| b.graph.nodes.iter().map(|n| format!("{:?}", n.kind)).collect::<Vec<_>>();
            assert_eq!(kinds(&with), kinds(&tree));
            let (mut a, mut b) = ([0.0; 2], [0.0; 2]);
            for (d, _) in &data {
                tree.predict(d, &mut a);
                with.predict(d, &mut b);
                assert_eq!(a.map(f64::to_bits), b.map(f64::to_bits));
            }
        }
    }

    /// errdag3: `noisy` with every fifth position drawn, for probit leaves (three results).
    fn noisy3(n: usize) -> Vec<([u8; 4], [f64; 3])> {
        let draw = |i: usize, [w, b]: [f64; 2]| if i % 5 == 4 { [0.0, 0.0, 1.0] } else { [w, b, 0.0] };
        noisy(n).into_iter().enumerate().map(|(i, (d, l))| (d, draw(i, l))).collect()
    }

    /// Probit leaves under error routing with CNBP routers and split noise 0.03, as T-P-Ne.
    fn probit_params(path_context: bool, target: SubrouterTarget) -> DelegationParams {
        DelegationParams {
            class_num: 3,
            split_noise: 0.03,
            subrouter_target: target,
            path_context,
            ..home_params(NodeKind::Probit, 3, 300, 6, 1.0)
        }
    }

    /// Each position's leaf and the bits of `predict` and `predict_log` over three results.
    fn snapshot3(predict: impl Fn(&[u8], &mut [f64], bool), leaf: impl Fn(&[u8]) -> usize) -> Vec<(usize, Vec<u64>)> {
        all_positions()
            .iter()
            .map(|p| {
                let (mut a, mut b) = ([0.0; 3], [0.0; 3]);
                predict(p, &mut a, false);
                predict(p, &mut b, true);
                (leaf(p), a.iter().chain(&b).map(|x| x.to_bits()).collect())
            })
            .collect()
    }

    fn bdd_snapshot3(bdd: &Bdd) -> Vec<(usize, Vec<u64>)> {
        snapshot3(|p, out, log| if log { bdd.predict_log(p, out) } else { bdd.predict(p, out) }, |p| bdd.leaf(p))
    }

    /// errdag3: a probit BDD at router_split_z inf (with or without path context and the router budget) makes the
    /// probit BDT's splits and predictions, bit for bit.
    #[test]
    fn probit_bdd_at_router_split_z_inf_is_the_probit_bdt() {
        let data = noisy3(20_000);
        let base = probit_params(false, SubrouterTarget::Routing);
        let mut tree = crate::bdt::Bdt::new(&[3; 4], base.clone());
        data.iter().for_each(|(d, l)| tree.train(d, l));
        tree.refresh();
        assert!(tree.split_count() >= 3, "too few splits to test: {}", tree.split_count());
        let tree_bits = snapshot3(|p, out, log| if log { tree.predict_log(p, out) } else { tree.predict(p, out) }, |_| 0);
        for path_context in [false, true] {
            for cap in [None, Some(4)] {
                let params = DelegationParams { path_context, router_split_z: Some(f64::INFINITY), router_split_limit: cap, ..base.clone() };
                let mut bdd = Bdd::new(&[3; 4], params);
                data.iter().for_each(|(d, l)| bdd.train(d, l));
                bdd.refresh();
                assert_eq!(bdd.split_count(), tree.split_count(), "{path_context} {cap:?}");
                let bits: Vec<_> = bdd_snapshot3(&bdd).into_iter().map(|(_, b)| (0, b)).collect();
                assert_eq!(bits, tree_bits, "{path_context} {cap:?}");
            }
        }
    }

    /// errdag3: probit leaves with path context. At every split a position keeps its leaf or goes to a new one below
    /// it, and its probabilities move only by the split noise's widening (the latent means stay); the cells are the
    /// sets upstream; the cells learn; files and states round-trip, and a resumed model trains on exactly as before.
    #[test]
    fn probit_path_cells_split_quietly_learn_and_round_trip() {
        for (v, target) in [SubrouterTarget::Home, SubrouterTarget::Routing].into_iter().enumerate() {
            let params = probit_params(true, target);
            let mut rng = Lcg(31 + v as u64);
            let data = noisy3(150 * 30);
            let mut bdd = Bdd::new(&[3; 4], params.clone());
            let mut registry = Vec::new();
            let mut cells_moved = false;
            for batch in data.chunks(150) {
                batch.iter().for_each(|(d, l)| bdd.train(d, l));
                bdd.refresh();
                let before = bdd_snapshot3(&bdd);
                let leaves: Vec<usize> = (0..bdd.node_count()).filter(|&i| bdd.graph.children(i).is_empty()).collect();
                let routers: Vec<usize> = (0..bdd.node_count()).filter(|&i| !bdd.graph.children(i).is_empty()).collect();
                if routers.is_empty() || rng.next(2) == 0 {
                    bdd.graph.split_leaf(leaves[rng.next(leaves.len() as u64) as usize]);
                } else {
                    bdd.split_delegator(routers[rng.next(routers.len() as u64) as usize]);
                }
                bdd.refresh();
                for ((leaf, bits), (leaf_after, bits_after)) in before.into_iter().zip(bdd_snapshot3(&bdd)) {
                    assert!(leaf == leaf_after || bdd.graph.children(leaf).contains(&leaf_after), "{target:?}");
                    for (x, y) in bits[..3].iter().zip(&bits_after[..3]) {
                        let (x, y) = (f64::from_bits(*x), f64::from_bits(*y));
                        assert!((x - y).abs() < 0.01, "{x} {y} {target:?}");
                    }
                }
                check_structure(&bdd, &mut registry);
                cells_moved |= bdd.graph.nodes.iter().any(|n| {
                    let t = n.table.as_ref().unwrap();
                    t.is_probit() && t.weights().iter().any(|&w| w != 0.0)
                });
            }
            assert!(bdd.graph.sets.len() >= 5, "too few router splits to test: {:?}", bdd.graph.sets);
            assert!(cells_moved, "the probit cells never trained: {target:?}");
            let text = bdd.to_text();
            let back = Bdd::from_text(&text, false).unwrap();
            assert_eq!(back.to_text(), text);
            check_structure(&back, &mut Vec::new());
            let state = bdd.to_state().into_bytes();
            let mut resumed = Bdd::from_state(&state, &[3; 4], params.clone()).unwrap();
            assert_eq!(resumed.to_state().into_bytes(), state);
            let mut original = bdd.clone();
            for model in [&mut original, &mut resumed] {
                noisy3(2000).iter().for_each(|(d, l)| model.train(d, l));
                model.refresh();
            }
            assert!(original.to_state().into_bytes() == resumed.to_state().into_bytes(), "{target:?}");
            assert_eq!(bdd_snapshot3(&original), bdd_snapshot3(&resumed));
        }
    }

    #[test]
    #[should_panic(expected = "needs split_noise > 0")]
    fn probit_path_context_needs_split_noise() {
        Bdd::new(&[3; 4], DelegationParams { split_noise: 0.0, ..probit_params(true, SubrouterTarget::Routing) });
    }

    #[test]
    #[should_panic(expected = "router_split_limit is for BDDs")]
    fn bdt_refuses_router_split_limit() {
        let params = DelegationParams { router_split_limit: Some(2), subrouter_target: SubrouterTarget::Routing, ..home_params(NodeKind::Lr, 3, 100, 2, 1.0) };
        crate::bdt::Bdt::new(&[3; 4], params);
    }

    // ---- grow2: the router split rules (theory-2 6.40-6.41; growth-spec section 2).

    use crate::delegation::{GrowParams, LAST_ROUTER_CLOSE, MdlN, RouterCloseRecord, RouterShadow, RouterSplitRule};

    /// Path-context parameters (LR leaves, CNBP routers, routing targets) under router split rule `rule`, windows of
    /// `w`: the leaf budget is the 2 leaf splits `path_structure` makes, and router splits may go up to 3, so one more
    /// is left after its root split (fewer router splits than leaf splits).
    fn rule_params(rule: RouterSplitRule, w: u32) -> DelegationParams {
        DelegationParams {
            split_threshold: w,
            split_limit: 2,
            router_split_limit: Some(3),
            grow: GrowParams { router_rule: rule, ..GrowParams::default() },
            ..ctx_params(NodeKind::Lr, NodeKind::Cnbp, SubrouterTarget::Routing)
        }
    }

    /// The structure of the router tests: the root split as a leaf (leaves 1-3) and as a router (copies 4-6, set 0,
    /// each over 1-3), then leaf 1 split (leaves 7-9): router 1's parents are set 0's members, so it splits by path.
    fn path_structure(params: DelegationParams) -> Bdd {
        let mut bdd = Bdd::new(&[3; 4], params);
        bdd.graph.split_leaf(0);
        bdd.split_delegator(0);
        bdd.graph.split_leaf(1);
        assert_eq!((bdd.graph.children(0), bdd.graph.children(1)), (&[4, 5, 6][..], &[7, 8, 9][..]));
        assert_eq!(bdd.graph.split_counts(), (2, 1));
        assert_eq!(bdd.graph.split_kind(1), Some(SplitKind::Path));
        bdd
    }

    /// Router 1 of `path_structure` driven directly, as `train_node_ctx` drives a router: per example (position,
    /// member of set 0 it arrived through, its children's losses), the close when due, then `router_observe` on the
    /// losses (the children's errors too, as for a router over leaves), the router's training and its record.
    /// Returns whether router 1 split by path; stops there.
    fn drive(bdd: &mut Bdd, examples: impl IntoIterator<Item = ([u8; 4], usize, [f64; 3])>) -> bool {
        for (data, j, losses) in examples {
            bdd.graph.step += 1;
            if bdd.check_delegator_split(1) && bdd.graph.unshared.take().is_some() {
                return true;
            }
            let marks = vec![(0, j)];
            bdd.graph.router_observe(1, Some(3 + j), &marks, &data, &losses, &losses);
            let (_, router_error) = bdd.graph.train_router_ctx(1, &marks, &data, &losses);
            bdd.graph.record_router(1, router_error);
        }
        false
    }

    /// Losses with the best child's 0.1 and the others' 1.0.
    fn losses(best: usize) -> [f64; 3] {
        std::array::from_fn(|c| if c == best { 0.1 } else { 1.0 })
    }

    /// An XOR of the arrival path and feature 0: through member 1 child f is best, through member 2 child 1 - f,
    /// through member 3 child 2. A CNBP router with the path's cell cannot learn it (additive); one router per path can.
    fn xor_paths(seed: u64, n: usize) -> Vec<([u8; 4], usize, [f64; 3])> {
        let mut rng = Lcg(seed);
        (0..n)
            .map(|i| {
                let data = [rng.next(2) as u8, rng.next(3) as u8, rng.next(3) as u8, rng.next(3) as u8];
                let j = 1 + i % 3;
                let f = data[0] as usize;
                (data, j, losses([f, 1 - f, 2][j - 1]))
            })
            .collect()
    }

    /// The best child independent of the path: child f (feature 0) 9 times in 10, else child 2.
    fn feature_only(seed: u64, n: usize) -> Vec<([u8; 4], usize, [f64; 3])> {
        let mut rng = Lcg(seed);
        (0..n)
            .map(|i| {
                let data = [rng.next(2) as u8, rng.next(3) as u8, rng.next(3) as u8, rng.next(3) as u8];
                let best = if rng.next(10) < 9 { data[0] as usize } else { 2 };
                (data, 1 + i % 3, losses(best))
            })
            .collect()
    }

    fn take_closes() -> Vec<RouterCloseRecord> {
        LAST_ROUTER_CLOSE.with(|c| std::mem::take(&mut *c.borrow_mut()))
    }

    fn ready_children(bdd: &mut Bdd, w: u32) {
        for c in [7, 8, 9] {
            bdd.graph.nodes[c].num_trained_total = w;
        }
    }

    /// Identity of the options' record: grow1's growth options print as `#[derive(Debug)]` printed them, and grow2's
    /// only when used; the router split rule's are recorded in the state's options, `log_router_stats` never.
    #[test]
    fn grow2_options_print_as_grow1_unless_used() {
        let d = GrowParams::default();
        let grow1 = "GrowParams { credit: 5000000, credit_cap: 2, stall_z: 2.0, stall_windows: 2, trial_z: 2.0, mdl: 1.0, subtree_share: 0.9, trials: 2, warn_nodes: 55 }";
        assert_eq!(format!("{d:?}"), grow1);
        assert_eq!(format!("{:?}", GrowParams { log_router_stats: true, ..d }), grow1);
        assert!(format!("{:?}", GrowParams { mdl_n: MdlN::Lifetime, ..d }).ends_with("warn_nodes: 55, mdl_n: Lifetime }"));
        let ruled = GrowParams { router_rule: RouterSplitRule::StallHg, router_z: 3.0, router_shadow: RouterShadow::Path, ..d };
        assert!(format!("{ruled:?}").ends_with("warn_nodes: 55, router_rule: StallHg, router_z: 3.0, router_shadow: Path }"));
        let guarded = GrowParams { guard_share: 0.9, ..ruled };
        assert!(format!("{guarded:?}").ends_with("router_shadow: Path, guard_share: 0.9 }"));
        assert_eq!(format!("{:?}", GrowParams { guard_share: 1.0, ..ruled }), format!("{ruled:?}"));
        let plain = rule_params(RouterSplitRule::Z, 100);
        assert!(!format!("{plain:?}").contains("GrowParams"));
        assert!(format!("{:?}", rule_params(RouterSplitRule::Subtree, 100)).contains("router_rule: Subtree"));
    }

    /// H is exact: per example the routed child's injected loss minus the least, before the router trains, in blocks.
    #[test]
    fn router_headroom_is_exact_on_injected_losses() {
        let mut bdd = path_structure(rule_params(RouterSplitRule::StallH, 200));
        let stream = xor_paths(3, 25);
        let (mut partial, mut done) = (0.0, Vec::new());
        for (i, (data, j, l)) in stream.iter().enumerate() {
            bdd.graph.step += 1;
            let marks = vec![(0, *j)];
            let routed = bdd.graph.route_ctx(1, &marks, data);
            bdd.graph.router_observe(1, Some(3 + j), &marks, data, l, l);
            partial += l[routed] - l.iter().copied().fold(f64::INFINITY, f64::min);
            if (i + 1) % 10 == 0 {
                done.push(partial / 10.0);
                partial = 0.0;
            }
            let (_, e) = bdd.graph.train_router_ctx(1, &marks, data, l);
            bdd.graph.record_router(1, e);
        }
        let h = bdd.graph.nodes[1].rgrow.h;
        assert_eq!((h.count, h.in_block), (2, 5));
        assert_eq!(h.sum.to_bits(), (done[0] + done[1]).to_bits());
        assert_eq!(h.sum_sq.to_bits(), (done[0] * done[0] + done[1] * done[1]).to_bits());
        assert_eq!(h.partial.to_bits(), partial.to_bits());
        assert!(h.sum > 0.0, "the untrained router misroutes some");
    }

    /// The losses H takes for a router over copies are the losses of the children the copies route to: each copy's
    /// subtree error, as the descent would realise it.
    #[test]
    fn copies_losses_are_their_subtree_errors() {
        let mut bdd = path_structure(rule_params(RouterSplitRule::StallH, 1_000_000));
        noisy(3000).iter().for_each(|(d, l)| bdd.train(d, l));
        let mut scratch = [0.0; 2];
        for p in all_positions() {
            bdd.graph.step += 1;
            let label = [1.0, 0.0];
            let mut marks = Vec::new();
            let (_, losses) = bdd.routing_losses_ctx(&[4, 5, 6], &[1, 2, 3], &mut marks, &p, &label, &mut scratch);
            assert!(marks.is_empty());
            for (k, copy) in [4, 5, 6].into_iter().enumerate() {
                bdd.graph.enter(copy, &mut marks);
                let e = bdd.graph.subtree_error_ctx(copy, &mut marks, &p, &label, &mut scratch);
                marks.clear();
                assert_eq!(losses[k].to_bits(), e.to_bits(), "{p:?} copy {copy}");
            }
        }
    }

    /// G is prequential: an example's gain is the routed child's loss minus that of the child its shadow picked
    /// before training on it; the shadow then trains on the router's target; only the example's path's shadow.
    #[test]
    fn shadow_gain_is_scored_before_the_shadow_trains() {
        let mut bdd = path_structure(rule_params(RouterSplitRule::StallHg, 200));
        assert_eq!(bdd.graph.run_shadows(1, SplitKind::Path), "started");
        let (data, j, l) = ([1, 0, 2, 1], 2, losses(2));
        let marks = vec![(0, j)];
        bdd.graph.step += 1;
        let routed = bdd.graph.route_ctx(1, &marks, &data);
        let before = bdd.graph.nodes[1].rgrow.shadows.clone().unwrap();
        let mut values = Vec::new();
        before.routers[j - 1].1.values_for(&marks, &mut values);
        let mut out = [0.0; 3];
        crate::delegation::shadow_output(&before.routers[j - 1].0, &before.routers[j - 1].1, &values, &data, &mut out);
        let pick = crate::delegation::argmax(&out);
        bdd.graph.router_observe(1, Some(3 + j), &marks, &data, &l, &l);
        let rg = &bdd.graph.nodes[1].rgrow;
        assert_eq!(rg.g.partial.to_bits(), (l[routed] - l[pick]).to_bits());
        let after = rg.shadows.as_ref().unwrap();
        for (k, (r, t)) in after.routers.iter().enumerate() {
            let changed = state_of(r) != state_of(&before.routers[k].0) || t != &before.routers[k].1;
            assert_eq!(changed, k == j - 1, "shadow {k}");
        }
        let mut target_trained = before.routers[j - 1].clone();
        crate::delegation::shadow_train(&mut target_trained.0, &mut target_trained.1, &values, &data, &[0.0, 0.0, 1.0]);
        assert_eq!(state_of(&after.routers[j - 1].0), state_of(&target_trained.0));
        assert_eq!(after.routers[j - 1].1, target_trained.1);
    }

    /// (c) stall-hg fires at the first close whose gate is open and whose G clears its bars, on a stream where the
    /// best child depends on the arrival path; the unsharing gives each member its copy, the copies are the shadows.
    #[test]
    fn stall_hg_fires_on_a_path_dependent_best_child() {
        let w = 2000;
        let mut bdd = path_structure(rule_params(RouterSplitRule::StallHg, w));
        ready_children(&mut bdd, w);
        take_closes();
        let stream = xor_paths(7, 40 * w as usize);
        // The shadows at each close, for the split's check.
        let mut shadows = None;
        let mut split = false;
        for (data, j, l) in stream {
            if bdd.graph.nodes[1].num_trained_last >= w {
                shadows = bdd.graph.nodes[1].rgrow.shadows.clone();
            }
            if drive(&mut bdd, [(data, j, l)]) {
                split = true;
                break;
            }
        }
        let closes = take_closes();
        assert!(split, "{closes:#?}");
        assert!(closes.iter().all(|c| c.idx == 1));
        let first = closes.iter().position(|c| c.gate && c.g_pass).expect("an open gate with G passing");
        assert_eq!(first, closes.len() - 1, "split at the first eligible close: {closes:#?}");
        assert!(closes[..first].iter().all(|c| !c.split));
        let c = &closes[first];
        assert!(c.stagnant && c.ready && c.h_pass && c.split && c.kind == Some(SplitKind::Path));
        let (g, h) = (c.g.unwrap(), c.h.unwrap());
        assert!(g.0 > 0.2 && h.0 > 0.2, "G {g:?} H {h:?}");
        // grow3: the null shadows (hash partition, additive cells like the router's) cannot learn the XOR either, so
        // the paired difference carries the real shadows' gain, and it is what passed.
        let (null, diff) = (c.g_null.unwrap(), c.g_diff.unwrap());
        assert!(diff.0 > 0.2 && diff.0 > 2.0 * diff.1 && null.0.abs() < 0.1, "G null {null:?} diff {diff:?}");
        assert_eq!(c.k, Some(2 * (39 + 12)));
        // The unsharing: member 1 keeps router 1, members 2 and 3 point to new routers 10 and 11 over the same
        // children; the routers are the shadows by path.
        let gr = &bdd.graph;
        assert_eq!(gr.nodes.len(), 12);
        assert_eq!([gr.children(4), gr.children(5), gr.children(6)], [&[1, 2, 3][..], &[10, 2, 3], &[11, 2, 3]]);
        for r in [1, 10, 11] {
            assert_eq!(gr.children(r), [7, 8, 9]);
            assert!(gr.nodes[r].member.is_none());
        }
        let shadows = shadows.unwrap();
        for (k, r) in [1, 10, 11].into_iter().enumerate() {
            assert_eq!(state_of(&gr.nodes[r].classifier), state_of(&shadows.routers[k].0), "router {r}");
            assert_eq!(gr.nodes[r].table.as_deref().unwrap(), &shadows.routers[k].1);
        }
        assert_eq!(gr.sets.len(), 1);
        assert_eq!(gr.split_counts(), (2, 2));
        assert_eq!(gr.post_checks.len(), 1);
        assert_eq!(gr.post_checks[0].made, [1, 10, 11]);
        assert!(gr.nodes[1].rgrow.shadows.is_none() && gr.nodes[1].grow.closes == 0);
        crate::pathctx::check(gr).unwrap();
        check_structure(&bdd, &mut Vec::new());
        // The model file reads back and predicts the same; the state likewise.
        let back = Bdd::from_text(&bdd.to_text(), false).unwrap();
        assert_eq!(back.to_text(), bdd.to_text());
        assert_eq!(snapshot(&back), snapshot(&bdd));
        let state = bdd.to_state().into_bytes();
        assert_eq!(Bdd::from_state(&state, &[3; 4], bdd.params().clone()).unwrap().to_state().into_bytes(), state);
    }

    /// (c) The same with the best child independent of the path: G stays within 1 standard error of 0 over 10
    /// windows and stall-hg never fires.
    #[test]
    fn stall_hg_stays_quiet_without_path_dependence() {
        let w = 2000;
        let mut bdd = path_structure(rule_params(RouterSplitRule::StallHg, w));
        ready_children(&mut bdd, w);
        take_closes();
        assert!(!drive(&mut bdd, feature_only(9, 16 * w as usize)));
        let closes = take_closes();
        let with_g: Vec<_> = closes.iter().filter_map(|c| c.g).collect();
        assert!(with_g.len() >= 10, "{closes:#?}");
        for (mean, se) in with_g {
            assert!(mean.abs() <= se, "G {mean} se {se}");
        }
        assert!(closes.iter().all(|c| !c.split));
        assert!(closes.iter().filter(|c| c.gate).count() >= 8, "the gate opens (H passes, stagnant, ready): {closes:#?}");
    }

    /// stall-h's gate: no split before the router's error is stagnant (three closes), nor while a child has not
    /// seen a window of its own; it splits at the first close where both hold, with H over its bars.
    #[test]
    fn stall_h_waits_for_stagnation_and_its_children() {
        let w = 1000;
        // A low MDL bar (k 102 at n 1000 is 0.35 at M 1, over H ~0.3): only the stall and the children decide.
        let params = || DelegationParams { grow: GrowParams { mdl: 0.25, ..rule_params(RouterSplitRule::StallH, w).grow }, ..rule_params(RouterSplitRule::StallH, w) };
        // Children ready from the start: the first stagnant close splits.
        let mut bdd = path_structure(params());
        ready_children(&mut bdd, w);
        take_closes();
        assert!(drive(&mut bdd, xor_paths(5, 30 * w as usize)));
        let closes = take_closes();
        let first = closes.iter().position(|c| c.stagnant).unwrap();
        assert!(first >= 2, "stagnant only after 3 closes: {closes:#?}");
        assert_eq!(first, closes.len() - 1);
        assert!(closes.iter().all(|c| c.h_pass && c.ready));
        // Children not ready: stagnant closes pass without a split until they are.
        let mut bdd = path_structure(params());
        assert!(!drive(&mut bdd, xor_paths(5, 8 * w as usize)));
        let closes = take_closes();
        assert!(closes.iter().filter(|c| c.stagnant).count() >= 3, "{closes:#?}");
        assert!(closes.iter().all(|c| !c.ready && !c.gate && !c.split));
        ready_children(&mut bdd, w);
        assert!(drive(&mut bdd, xor_paths(6, 2 * w as usize)));
        let closes = take_closes();
        assert_eq!(closes.len(), 1);
        assert!(closes[0].ready && closes[0].stagnant && closes[0].split);
    }

    /// The MDL bar on both sides: at the first stagnant close, a bar just above H refuses the split, one just below
    /// lets it through (the training up to it is the same).
    #[test]
    fn stall_h_mdl_bar_on_both_sides() {
        let w = 1000;
        let run = |mdl: f64| {
            let mut p = rule_params(RouterSplitRule::StallH, w);
            p.grow.mdl = mdl;
            let mut bdd = path_structure(p);
            ready_children(&mut bdd, w);
            take_closes();
            let split = drive(&mut bdd, xor_paths(5, 6 * w as usize));
            (split, take_closes())
        };
        let (_, closes) = run(1.0);
        let c = closes.iter().find(|c| c.stagnant).unwrap().clone();
        let (h, n, k) = (c.h.unwrap().0, c.n, c.k.unwrap());
        let unit = crate::delegation::mdl_bar(1.0, k, n);
        assert!(unit > 0.0 && (c.bar.unwrap() - unit).abs() <= 1e-15);
        for (factor, splits) in [(1.1, false), (0.9, true)] {
            let (_, closes) = run(factor * h / unit);
            let at = closes.iter().find(|x| x.step == c.step).unwrap();
            assert_eq!(at.h.unwrap().0.to_bits(), h.to_bits());
            assert!((at.bar.unwrap() - factor * h).abs() <= 1e-12, "{} {}", at.bar.unwrap(), factor * h);
            assert_eq!((at.h_pass, at.split), (splits, splits), "factor {factor}");
        }
    }

    /// The budget: with no router split left (the root's is the only one router_split_limit 1 allows), an open gate
    /// splits nothing; nor while router splits are as many as leaf splits (a second router split, copy 4's: router 1's
    /// parents are then copies 5 and 6 and copy 4's three copies, still several, so its split would be by path).
    #[test]
    fn router_rules_respect_the_budget() {
        let w = 1000;
        for (cap, second) in [(1, false), (3, true)] {
            let mut p = rule_params(RouterSplitRule::StallH, w);
            (p.router_split_limit, p.grow.mdl) = (Some(cap), 0.25);
            let mut bdd = path_structure(p);
            if second {
                bdd.split_delegator(4);
                assert_eq!(bdd.graph.path_parents(1), Some(vec![5, 6, 10, 11, 12]));
                assert_eq!(bdd.graph.split_kind(1), Some(SplitKind::Path));
            }
            let counts = bdd.graph.split_counts();
            assert_eq!(counts, (2, 1 + second as u32));
            ready_children(&mut bdd, w);
            take_closes();
            assert!(!drive(&mut bdd, xor_paths(5, 12 * w as usize)));
            let closes = take_closes();
            assert!(closes.iter().filter(|c| c.gate).count() >= 5, "{closes:#?}");
            assert!(closes.iter().all(|c| !c.split));
            assert_eq!(bdd.graph.split_counts(), counts);
        }
    }

    /// Real training under each rule, with bars low enough that routers split: the budget holds at every step, the
    /// structure stays valid (cells, registry, decodable paths), and a state saved at any point (shadows running or
    /// not, checks waiting) trains on exactly as the original.
    #[test]
    fn router_rules_train_within_budget_and_resume_exactly() {
        let mut unshared = 0;
        for rule in [RouterSplitRule::StallH, RouterSplitRule::StallHg, RouterSplitRule::Subtree] {
            for router in [NodeKind::Cnbp, NodeKind::Lr] {
                let mut p = DelegationParams {
                    split_threshold: 400,
                    split_limit: 6,
                    router_split_limit: Some(4),
                    router,
                    grow: GrowParams {
                        router_rule: rule,
                        router_z: 0.0,
                        mdl: 0.0,
                        stall_z: 1e9,
                        subtree_share: 0.0,
                        ..GrowParams::default()
                    },
                    ..ctx_params(NodeKind::Lr, router, SubrouterTarget::Routing)
                };
                p.grow.router_shadow = RouterShadow::Auto;
                let data = noisy(60_000);
                let mut bdd = Bdd::new(&[3; 4], p.clone());
                let mut saved_with_shadows = 0;
                for (i, chunk) in data.chunks(2000).enumerate() {
                    let state = bdd.to_state().into_bytes();
                    let mut back = Bdd::from_state(&state, &[3; 4], p.clone()).unwrap();
                    assert_eq!(back.to_state().into_bytes(), state);
                    saved_with_shadows += bdd.graph.nodes.iter().any(|n| n.rgrow.shadows.as_ref().is_some_and(|s| s.running)) as usize;
                    for model in [&mut bdd, &mut back] {
                        for (d, l) in chunk {
                            model.train(d, l);
                            let (leaf, routers) = model.graph.split_counts();
                            assert!(routers <= 4 && routers <= leaf && leaf <= 6, "{leaf} {routers}");
                        }
                        model.refresh();
                    }
                    assert!(back.to_state().into_bytes() == bdd.to_state().into_bytes(), "{rule:?} {router:?} chunk {i}");
                }
                let (leaf, routers) = bdd.graph.split_counts();
                assert_eq!(leaf, 6, "{rule:?}");
                if rule != RouterSplitRule::StallH {
                    assert!(saved_with_shadows > 0, "{rule:?} {router:?}: no state saved with shadows running");
                }
                if rule == RouterSplitRule::StallH {
                    assert_eq!(routers, 4, "the permissive stall-h spends its budget");
                }
                // An unsharing leaves the unshared router's children with parents of no set (its copies).
                let g = &bdd.graph;
                let below_unshared = |c: usize| {
                    let parents = g.parents_of(c);
                    parents.len() >= 2 && parents.iter().all(|&p| g.nodes[p].member.is_none())
                };
                unshared += (0..g.nodes.len()).any(below_unshared) as usize;
                crate::pathctx::check(&bdd.graph).unwrap();
                check_structure(&bdd, &mut Vec::new());
                let back = Bdd::from_text(&bdd.to_text(), false).unwrap();
                assert_eq!(snapshot(&back), snapshot(&bdd));
            }
        }
        assert!(unshared > 0);
    }

    /// Partition choice (growth-spec 2.7 b): a router with one parent shadows by cluster; once a router split above it
    /// makes its parents a sibling set, by path; the shadows of the old partition are dropped at that split, and a
    /// router below the split whose partition stays gets the new cell in its shadows' tables.
    #[test]
    fn partition_follows_the_structure() {
        let mut p = rule_params(RouterSplitRule::StallHg, 1000);
        p.split_limit = 3;
        let mut bdd = Bdd::new(&[3; 4], p.clone());
        bdd.graph.split_leaf(0);
        bdd.graph.split_leaf(1);
        bdd.graph.split_leaf(4);
        assert_eq!((bdd.graph.children(1), bdd.graph.children(4)), (&[4, 5, 6][..], &[7, 8, 9][..]));
        assert_eq!((bdd.graph.split_kind(1), bdd.graph.split_kind(4)), (Some(SplitKind::Cluster), Some(SplitKind::Cluster)));
        bdd.graph.run_shadows(1, SplitKind::Cluster);
        bdd.graph.run_shadows(4, SplitKind::Cluster);
        noisy(2000).iter().for_each(|(d, l)| bdd.train(d, l));
        bdd.split_delegator(0);
        bdd.graph.conform_shadows();
        assert_eq!(bdd.graph.split_kind(1), Some(SplitKind::Path));
        assert!(bdd.graph.nodes[1].rgrow.shadows.is_none(), "router 1's cluster shadows are dropped");
        assert_eq!(bdd.graph.split_kind(4), Some(SplitKind::Cluster));
        let cells = bdd.graph.nodes[4].table.as_ref().unwrap().cells().to_vec();
        assert_eq!(cells, [0]);
        for (_, t) in &bdd.graph.nodes[4].rgrow.shadows.as_ref().unwrap().routers {
            assert_eq!(t.cells(), cells);
        }
        let path = Some(SplitKind::Path);
        for (shadow, want1, want4) in [(RouterShadow::Path, path, None), (RouterShadow::Cluster, Some(SplitKind::Cluster), Some(SplitKind::Cluster))] {
            bdd.graph.params.grow.router_shadow = shadow;
            assert_eq!((bdd.graph.split_kind(1), bdd.graph.split_kind(4)), (want1, want4), "{shadow:?}");
        }
        // Training goes on with node 4's conformed shadows.
        bdd.graph.params.grow.router_shadow = RouterShadow::Auto;
        noisy(2000).iter().for_each(|(d, l)| bdd.train(d, l));
    }

    /// (e) The cluster split with shadows: the top router is the cluster router, the copies are the shadows as
    /// trained, a new sibling set, the structure checks, the file reads back.
    #[test]
    fn cluster_split_uses_the_shadows() {
        let mut bdd = Bdd::new(&[3; 4], rule_params(RouterSplitRule::StallHg, 1_000_000));
        bdd.graph.split_leaf(0);
        bdd.graph.run_shadows(0, SplitKind::Cluster);
        noisy(5000).iter().for_each(|(d, l)| bdd.train(d, l));
        bdd.refresh();
        let shadows = bdd.graph.nodes[0].rgrow.shadows.take().unwrap();
        let crate::delegation::Partition::Cluster { q, trained } = &shadows.partition else { panic!() };
        assert_eq!(*trained, 5000);
        let (q, routers) = (q.clone(), shadows.routers.clone());
        assert!(state_of(&routers[0].0) != state_of(&routers[1].0), "the shadows trained on different parts");
        let made = bdd.cluster_split(0, Some(shadows));
        assert_eq!(made, [4, 5, 6]);
        let g = &bdd.graph;
        assert_eq!(g.sets, [vec![4, 5, 6]]);
        assert_eq!(state_of(&g.nodes[0].classifier), state_of(&q));
        for (k, &m) in made.iter().enumerate() {
            assert_eq!(state_of(&g.nodes[m].classifier), state_of(&routers[k].0));
            assert_eq!(g.nodes[m].table.as_deref().unwrap(), &routers[k].1);
            assert_eq!(g.children(m), [1, 2, 3]);
        }
        crate::pathctx::check(g).unwrap();
        bdd.refresh();
        let back = Bdd::from_text(&bdd.to_text(), false).unwrap();
        assert_eq!(snapshot(&back), snapshot(&bdd));
    }

    /// grow3 (6.46 (iii) 1): the share guard. Router 1's children 7-9: 7 has closed a window, 8 and 9 have not; the
    /// guard opens when 7 holds 90% of the children's lifetime examples, not at 89%; at share 1 (grow2's guard) it
    /// waits for every child; no lifetime examples, no judging.
    #[test]
    fn guard_share_opens_at_90_percent_not_89() {
        let w = 1000;
        let mut p = rule_params(RouterSplitRule::StallH, w);
        p.grow.guard_share = 0.9;
        let mut bdd = path_structure(p);
        let set = |bdd: &mut Bdd, lives: [u64; 3], ready: [bool; 3]| {
            for (k, c) in [7, 8, 9].into_iter().enumerate() {
                bdd.graph.nodes[c].life = lives[k];
                bdd.graph.nodes[c].num_trained_total = if ready[k] { w } else { w - 1 };
            }
        };
        set(&mut bdd, [0, 0, 0], [true, true, true]);
        assert!(!bdd.graph.children_ready(1), "no lifetime examples");
        for (lives, open) in [([9000, 1000, 0], true), ([8900, 1100, 0], false), ([9000, 700, 300], true), ([8999, 1001, 0], false)] {
            set(&mut bdd, lives, [true, false, false]);
            assert_eq!(bdd.graph.children_ready(1), open, "{lives:?}");
        }
        bdd.graph.params.grow.guard_share = 0.89;
        set(&mut bdd, [8900, 1100, 0], [true, false, false]);
        assert!(bdd.graph.children_ready(1));
        bdd.graph.params.grow.guard_share = 1.0;
        set(&mut bdd, [9990, 10, 0], [true, false, true]);
        assert!(!bdd.graph.children_ready(1));
        set(&mut bdd, [9990, 10, 0], [true, true, true]);
        assert!(bdd.graph.children_ready(1));
        // In a close: the starved child (1% of the examples) no longer holds the router at 0.9.
        bdd.graph.params.grow.guard_share = 0.9;
        bdd.graph.params.grow.mdl = 0.25;
        set(&mut bdd, [99_000, 1000, 0], [true, false, false]);
        take_closes();
        assert!(drive(&mut bdd, xor_paths(5, 30 * w as usize)));
        let closes = take_closes();
        assert!(closes.iter().all(|c| c.ready), "{closes:#?}");
        assert!(closes.last().unwrap().split);
    }

    /// A null world that drifts: the best child is feature 0 plus a phase that advances every `period` examples,
    /// whatever the path; the paths go round.
    fn drifting(seed: u64, n: usize, period: usize) -> Vec<([u8; 4], usize, [f64; 3])> {
        let mut rng = Lcg(seed);
        (0..n)
            .map(|i| {
                let data = [rng.next(3) as u8, rng.next(3) as u8, rng.next(3) as u8, rng.next(3) as u8];
                (data, 1 + i % 3, losses((data[0] as usize + i / period) % 3))
            })
            .collect()
    }

    /// grow3 (6.46 (iii) 2): under a null world whose target drifts, the real shadows (by path, each on a third of
    /// the examples) lag the router, which trains on all of them, so G_real drifts below 0 by several standard errors,
    /// as in the P1 smoke; the null shadows (by hash, the same shape) lag alike, so the paired difference stays
    /// within 2 standard errors in every window, and stall-hg does not fire.
    #[test]
    fn null_shadows_cancel_a_drifting_g() {
        let w = 2000;
        let mut bdd = path_structure(rule_params(RouterSplitRule::StallHg, w));
        ready_children(&mut bdd, w);
        take_closes();
        assert!(!drive(&mut bdd, drifting(11, 30 * w as usize, 4 * w as usize)));
        let closes = take_closes();
        let paired: Vec<_> = closes.iter().filter_map(|c| Some((c.g?, c.g_null?, c.g_diff?))).collect();
        assert!(paired.len() >= 10, "{closes:#?}");
        let drifted = paired.iter().filter(|(g, _, _)| g.0 < -2.0 * g.1).count();
        assert!(drifted >= 3, "G_real below -2 SE in {drifted} windows: {paired:#?}");
        for (g, null, diff) in &paired {
            assert!(diff.0.abs() <= 2.0 * diff.1, "G {g:?} null {null:?} diff {diff:?}");
        }
        assert!(closes.iter().all(|c| !c.split));
    }

    /// grow3: stall-hg's states carry the null shadows and their blocks, and resume exactly mid-shadow.
    #[test]
    fn null_shadows_resume_exactly() {
        let w = 2000;
        let p = rule_params(RouterSplitRule::StallHg, w);
        let mut bdd = path_structure(p.clone());
        ready_children(&mut bdd, w);
        let stream = drifting(12, 6 * w as usize, 4 * w as usize);
        let mut checked = 0;
        for chunk in stream.chunks(700) {
            let running = bdd.graph.nodes[1].rgrow.shadows.as_ref().is_some_and(|s| s.running && s.null.len() == 3);
            let state = bdd.to_state().into_bytes();
            let mut back = Bdd::from_state(&state, &[3; 4], p.clone()).unwrap();
            assert_eq!(back.to_state().into_bytes(), state);
            assert!(!drive(&mut bdd, chunk.iter().cloned()));
            assert!(!drive(&mut back, chunk.iter().cloned()));
            assert!(back.to_state().into_bytes() == bdd.to_state().into_bytes());
            checked += running as usize;
        }
        assert!(checked >= 3, "{checked} saves with the null shadows running");
        let g = &bdd.graph.nodes[1].rgrow;
        assert!(g.g_null.count + g.g_null.in_block > 0 && g.g_diff.count + g.g_diff.in_block > 0);
    }

    // grow6 (theory-2 6.80): the leaf growth rule in DAGs with path context, and the encourage rule's allowance,
    // priority, long trials, undo, chain cap and cell counts.

    use crate::delegation::{GrowGate, GrowPriority};

    /// The growth rule as T-P-Ne-grow's (grow4: age gate, snapshot priority, MDL bar `mdl`) over probit leaves under
    /// CNBP error routers with cached probabilities, windows of `w`, a leaf credit per `credit`; with path context.
    fn grow6_params(w: u32, credit: u64, mdl: f64) -> DelegationParams {
        let mut p = probit_params(true, SubrouterTarget::Routing);
        (p.split_threshold, p.split_limit, p.use_probs) = (w, 1_000_000, true);
        (p.split_rule, p.trial_error) = (SplitRule::Grow, ErrorMeasure::Log);
        p.grow = GrowParams { credit, mdl, gate: GrowGate::Age, priority: GrowPriority::Snapshot, ..GrowParams::default() };
        p
    }

    /// `grow6_params` under the encourage rule: a router credit per `router_credit` (0: none).
    fn encourage_params(w: u32, credit: u64, mdl: f64, router_credit: u64) -> DelegationParams {
        let mut p = grow6_params(w, credit, mdl);
        (p.grow.router_rule, p.grow.router_credit, p.grow.guard_share) = (RouterSplitRule::Encourage, router_credit, 0.9);
        p
    }

    /// Three results by the XOR of features 0 and 1 (two values each; white 9 times in 10 when they agree, black
    /// likewise when not), a draw one example in five, features 2 and 3 noise: linear leaves need splits.
    fn xor3(seed: u64, n: usize) -> Vec<([u8; 4], [f64; 3])> {
        let mut rng = Lcg(seed);
        (0..n)
            .map(|i| {
                let x = [rng.next(2) as u8, rng.next(2) as u8, rng.next(3) as u8, rng.next(3) as u8];
                let white = (x[0] == x[1]) == (rng.next(10) < 9);
                (x, if i % 5 == 4 { [0.0, 0.0, 1.0] } else if white { [1.0, 0.0, 0.0] } else { [0.0, 1.0, 0.0] })
            })
            .collect()
    }

    /// 6.80's identity: a DAG growing by the leaf rule without router splits (rule z at router_split_z inf, or
    /// encourage without an allowance) is the grow tree, bit for bit: the same trials and verdicts at the same steps,
    /// the same leaves and predictions, with the MDL bar off and on.
    #[test]
    fn grow6_dag_without_router_splits_is_the_grow_tree() {
        let data = xor3(1, 100_000);
        let (mut kept, mut undone) = (0, 0);
        for mdl in [0.0, 1.0] {
            let p = grow6_params(1000, 3000, mdl);
            let mut tree = crate::bdt::Bdt::new(&[3; 4], DelegationParams { path_context: false, ..p.clone() });
            let dags = [DelegationParams { router_split_z: Some(f64::INFINITY), ..p.clone() }, encourage_params(1000, 3000, mdl, 0)];
            let mut dags: Vec<Bdd> = dags.into_iter().map(|q| Bdd::new(&[3; 4], q)).collect();
            for (d, l) in &data {
                tree.train(d, l);
                for bdd in &mut dags {
                    bdd.train(d, l);
                    assert_eq!(bdd.graph.grow, tree.graph().grow, "mdl {mdl}");
                    assert_eq!(bdd.graph.nodes.len(), tree.graph().nodes.len());
                }
            }
            tree.refresh();
            let tree_bits = snapshot3(|p, out, log| if log { tree.predict_log(p, out) } else { tree.predict(p, out) }, |_| 0);
            for mut bdd in dags {
                bdd.refresh();
                assert_eq!(bdd.split_count(), tree.split_count(), "mdl {mdl}");
                let bits: Vec<_> = bdd_snapshot3(&bdd).into_iter().map(|(_, b)| (0, b)).collect();
                assert_eq!(bits, tree_bits, "mdl {mdl}");
                assert!(bdd.graph.sets.is_empty() && bdd.graph.rallow.splits == 0);
            }
            let g = tree.graph().grow;
            kept += g.credits_spent - g.credits_refunded;
            undone += g.credits_refunded;
            eprintln!("mdl {mdl}: splits {}, trials kept {}, undone {}", tree.split_count(), g.credits_spent - g.credits_refunded, g.credits_refunded);
        }
        assert!(kept >= 2 && undone >= 2, "kept {kept}, undone {undone}: too few verdicts to test");
    }

    use crate::delegation::{RouterTrial, prio_before};

    /// The routers of the router splits on trial, each with its split's step and its examples so far.
    fn on_trial(bdd: &Bdd) -> Vec<(usize, u64, u64)> {
        bdd.graph.router_trials.iter().map(|t| (t.router, t.at, t.examples)).collect()
    }

    /// Every node's path-table cells, summed.
    fn cell_count(bdd: &Bdd) -> usize {
        bdd.graph.nodes.iter().filter_map(|n| n.table.as_ref()).map(|t| t.cells().len()).sum()
    }

    /// What an encourage run did, step by step: each router split (step, router, its chain before), each verdict
    /// (step, router, undone, the trial's examples), and the structure of a refreshed copy every `every` steps.
    #[derive(Debug, Default)]
    struct Run {
        splits: Vec<(u64, usize, u32)>,
        verdicts: Vec<(u64, usize, bool, u64)>,
        checks: usize,
    }

    fn encourage_run(bdd: &mut Bdd, data: &[([u8; 4], [f64; 3])], every: usize, mut each: impl FnMut(&Bdd, &Bdd)) -> Run {
        let mut run = Run::default();
        for (i, (d, l)) in data.iter().enumerate() {
            // As the step starts: the nodes removed at the last step dropped (numbering as `train` sees it).
            let mut before = bdd.clone();
            before.graph.remove_dead();
            bdd.train(d, l);
            let g = &bdd.graph;
            if g.rallow.splits > before.graph.rallow.splits {
                let t = g.router_trials.last().expect("a split goes on trial");
                run.splits.push((g.step, t.router, before.graph.nodes[t.router].chain));
            }
            let now = on_trial(bdd);
            for &(router, _, examples) in on_trial(&before).iter().filter(|(r, at, _)| !now.iter().any(|t| t.0 == *r && t.1 == *at)) {
                let undone = g.rallow.undone > before.graph.rallow.undone;
                // The verdict's step brought the trial's last example (it reached K windows on it).
                run.verdicts.push((g.step, router, undone, examples + 1));
            }
            each(&before, bdd);
            if every > 0 && (i + 1) % every == 0 {
                let mut copy = bdd.clone();
                copy.refresh();
                crate::pathctx::check(&copy.graph).unwrap();
                run.checks += 1;
            }
        }
        run
    }

    /// The allowance: a router credit at every `router_credit` positions through the root, banked up to the cap (the
    /// leaves' `credit_cap`); none with an interval of 0.
    #[test]
    fn grow6_router_credits_accrue_at_the_interval_up_to_the_cap() {
        let mut bdd = Bdd::new(&[3; 4], encourage_params(1000, 3000, 1.0, 100));
        let g = &mut bdd.graph;
        assert_eq!((g.rallow.credits, g.rallow.next_credit_at), (0, 100));
        for step in 1..=450u64 {
            g.step = step;
            g.router_credits();
            assert_eq!(g.rallow.credits as u64, (step / 100).min(2), "step {step}");
        }
        assert_eq!(g.rallow.next_credit_at, 500);
        g.rallow.credits -= 1;
        g.step = 499;
        g.router_credits();
        assert_eq!(g.rallow.credits, 1, "a spent credit comes back at the next interval, not at once");
        g.step = 500;
        g.router_credits();
        assert_eq!((g.rallow.credits, g.rallow.next_credit_at), (2, 600));
        let mut none = Bdd::new(&[3; 4], encourage_params(1000, 3000, 1.0, 0));
        none.graph.step = u64::MAX - 1;
        none.graph.router_credits();
        assert_eq!((none.graph.rallow.credits, none.graph.rallow.next_credit_at), (0, u64::MAX));
        let mut three = encourage_params(1000, 3000, 1.0, 100);
        three.grow.credit_cap = 3;
        let mut bdd = Bdd::new(&[3; 4], three);
        bdd.graph.step = 1000;
        bdd.graph.router_credits();
        assert_eq!(bdd.graph.rallow.credits, 3);
    }

    /// The priority: a measured G_real - G_null before none and the higher first, then likewise H, then the lower
    /// index (a full tie is not "before").
    #[test]
    fn grow6_priority_orders_by_g_then_h_then_index() {
        assert!(prio_before((Some(0.2), Some(0.0)), (Some(0.1), Some(9.0))));
        assert!(!prio_before((Some(0.1), Some(9.0)), (Some(0.2), Some(0.0))));
        assert!(prio_before((Some(0.1), Some(0.5)), (Some(0.1), Some(0.4))));
        assert!(!prio_before((Some(0.1), Some(0.4)), (Some(0.1), Some(0.5))));
        assert!(prio_before((Some(-5.0), None), (None, Some(1.0))));
        assert!(prio_before((None, Some(0.2)), (None, Some(0.1))));
        assert!(prio_before((None, Some(0.0)), (None, None)));
        assert!(!prio_before((Some(0.1), Some(0.1)), (Some(0.1), Some(0.1))));
        assert!(!prio_before((None, None), (None, None)));
    }

    /// Routers 0 (over 1-3), 1 (over 4-6) and 2 (over 7-9), the splits kept, every node trained a window and each
    /// router's own windows closed three times: all three eligible.
    fn three_routers(w: u32) -> Bdd {
        let mut bdd = Bdd::new(&[3; 4], encourage_params(w, 3000, 1.0, 100));
        for v in [0, 1, 2] {
            bdd.graph.split_leaf(v);
            bdd.graph.nodes[v].trial = None;
            bdd.graph.grow.trials_in_flight = 0;
        }
        assert_eq!((bdd.graph.children(0), bdd.graph.children(1), bdd.graph.children(2)), (&[1, 2, 3][..], &[4, 5, 6][..], &[7, 8, 9][..]));
        for node in &mut bdd.graph.nodes {
            (node.num_trained_total, node.life) = (w, 1);
        }
        [0, 1, 2].into_iter().for_each(|r| bdd.graph.nodes[r].grow.closes = 3);
        bdd
    }

    /// Eligibility and the pick (6.80 (1)): the eligible router with the highest G_real - G_null, ties to H, then the
    /// index; a router is eligible at three own closes, out of back-off, with its children ready (90% guard), with
    /// no leaf trial at it, its chain below the cap (not a copy), and outside every split on trial.
    #[test]
    fn grow6_the_allowance_picks_the_best_eligible_router() {
        let mut bdd = three_routers(100);
        let pick = |b: &Bdd| b.graph.encourage_pick().0;
        assert_eq!(pick(&bdd), Some(0), "no statistics yet: all tie, the lowest index");
        let set = |b: &mut Bdd, r: usize, g: Option<f64>, h: Option<f64>| (b.graph.nodes[r].rgrow.prio_g, b.graph.nodes[r].rgrow.prio_h) = (g, h);
        set(&mut bdd, 0, Some(0.1), Some(0.9));
        set(&mut bdd, 1, Some(0.3), Some(0.1));
        set(&mut bdd, 2, Some(0.3), Some(0.2));
        assert_eq!(pick(&bdd), Some(2), "the highest G, the tie to the higher H");
        assert_eq!(bdd.graph.encourage_pick().1.len(), 3);
        bdd.graph.nodes[2].chain = 1;
        assert_eq!(pick(&bdd), Some(1), "a router in a chain at the cap (a copy) does not split");
        let mut two = bdd.clone();
        two.graph.params.grow.router_chain = 2;
        assert_eq!(pick(&two), Some(2), "under a cap of 2 it may");
        bdd.graph.nodes[1].grow.closes = 2;
        assert_eq!(pick(&bdd), Some(0), "two own closes are too young");
        bdd.graph.nodes[1].grow.closes = 3;
        bdd.graph.nodes[1].grow.eligible_after_close = 4;
        assert_eq!(pick(&bdd), Some(0), "in back-off");
        bdd.graph.nodes[1].grow.eligible_after_close = 0;
        // Children ready by the 90% share of their examples: child 4 with 10% of them unready is fine, 11% not.
        bdd.graph.nodes[4].num_trained_total = 0;
        (bdd.graph.nodes[4].life, bdd.graph.nodes[5].life, bdd.graph.nodes[6].life) = (10, 45, 45);
        assert_eq!(pick(&bdd), Some(1));
        (bdd.graph.nodes[4].life, bdd.graph.nodes[5].life, bdd.graph.nodes[6].life) = (11, 45, 44);
        assert_eq!(pick(&bdd), Some(0), "children not ready");
        bdd.graph.nodes[4].num_trained_total = 100;
        assert_eq!(pick(&bdd), Some(1));
        // A leaf trial at the router.
        let mut trialled = bdd.clone();
        let mut other = three_routers(100);
        other.graph.split_leaf(3);
        trialled.graph.nodes[1].trial = other.graph.nodes[3].trial.take();
        assert!(trialled.graph.nodes[1].trial.is_some());
        assert_eq!(pick(&trialled), Some(0), "a leaf trial's delegator does not split");
        // A split on trial: its router, children and parents are busy.
        let g = &bdd.graph;
        let trial = RouterTrial {
            router: 1,
            kind: SplitKind::Cluster,
            at: 0,
            made: Vec::new(),
            set: None,
            children: vec![4, 5, 6],
            parents: Vec::new(),
            shadow: g.nodes[1].classifier.clone(),
            table: (**g.nodes[1].table.as_ref().unwrap()).clone(),
            null: Vec::new(),
            chain: 0,
            backoff_windows: 0,
            chosen: (None, None),
            examples: 0,
            blocks: Blocks::default(),
            window: Blocks::default(),
            window_means: Vec::new(),
            real: Blocks::default(),
            real_window: Blocks::default(),
        };
        bdd.graph.router_trials.push(trial);
        assert_eq!(pick(&bdd), None, "router 1 on trial, its parent 0 busy, 2 a copy");
        bdd.graph.nodes[2].chain = 0;
        assert_eq!(pick(&bdd), Some(2));
        // Under a router split: below a node in a chain (a copy, or a router that split), not the node itself.
        assert!(!(0..10).any(|v| bdd.graph.under_router_split(v)));
        bdd.graph.nodes[1].chain = 1;
        let under: Vec<usize> = (0..10).filter(|&v| bdd.graph.under_router_split(v)).collect();
        assert_eq!(under, [4, 5, 6]);
        bdd.graph.nodes[0].chain = 1;
        let under: Vec<usize> = (0..10).filter(|&v| bdd.graph.under_router_split(v)).collect();
        assert_eq!(under, [1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    /// An unsharing undone (the path case of the K-window undo): its parents point to the router again, the routers it
    /// made go at the next step, the router takes the unsplit router of the trial with the table of its arrival paths
    /// and backs off; the structure checks out and training goes on.
    #[test]
    fn grow6_an_undone_unsharing_restores_the_shared_router() {
        let mut p = encourage_params(1000, 3000, 0.0, 5000);
        p.grow.router_trial_z = f64::INFINITY;
        let data = xor3(2, 40_000);
        let mut bdd = Bdd::new(&[3; 4], p);
        let mut at = 0;
        for (i, (d, l)) in data.iter().enumerate() {
            bdd.train(d, l);
            if bdd.graph.router_trials.iter().any(|t| t.kind == SplitKind::Path && t.examples >= 2500) {
                at = i + 1;
                break;
            }
        }
        assert!(at > 0, "no unsharing on trial");
        let t = bdd.graph.router_trials.iter().position(|t| t.kind == SplitKind::Path).unwrap();
        let trial = bdd.graph.router_trials.remove(t);
        let (v, made, parents, shadow) = (trial.router, trial.made.clone(), trial.parents.clone(), state_of(&trial.shadow));
        assert!(parents.len() >= 2 && made.len() == parents.len() - 1);
        let (cells, sets) = (cell_count(&bdd), bdd.graph.sets.len());
        bdd.undo_router_split(trial);
        let g = &bdd.graph;
        for &p in &parents {
            assert!(g.children(p).contains(&v) && !made.iter().any(|m| g.children(p).contains(m)), "parent {p}");
        }
        assert!(made.iter().all(|m| g.dead.contains(m)));
        assert_eq!(state_of(&g.nodes[v].classifier), shadow, "the unsplit router, as it trained beside the split");
        assert_eq!((g.nodes[v].chain, g.sets.len(), g.rallow.undone), (0, sets, 1));
        assert!(cell_count(&bdd) >= cells, "the router's table has every arrival path's cells again");
        let mut copy = bdd.clone();
        copy.refresh();
        crate::pathctx::check(&copy.graph).unwrap();
        assert_eq!(copy.graph.nodes.len(), bdd.graph.nodes.len() - made.len());
        for (d, l) in &data[at..at + 5000] {
            bdd.train(d, l);
        }
        bdd.refresh();
        crate::pathctx::check(&bdd.graph).unwrap();
    }

    /// Splits, trials and verdicts in a run (6.80 (1)): splits only on credits; each trial judged when its region has
    /// seen exactly K windows of examples; under a bar of +inf every split stays, under a bar of -1e9 every split with
    /// a spread of pairs goes (one without, its pairs all 0, stays: it is not behind); an undone cluster split leaves
    /// its router over its old children, its copies removed (at the next refresh), its set gone with its cells, the
    /// router out of the chain and backing off; the structure checks out throughout; the chain cap holds.
    #[test]
    fn grow6_router_trials_keep_or_undo_after_k_windows() {
        let (w, k, d_r) = (1000u32, 5u64, 5000u64);
        let data = xor3(2, 40_000);
        for z in [f64::INFINITY, -1e9] {
            let mut p = encourage_params(w, 3000, 0.0, d_r);
            p.grow.router_trial_z = z;
            let mut bdd = Bdd::new(&[3; 4], p);
            let mut undos = 0;
            let run = encourage_run(&mut bdd, &data, 2_500, |before, after| {
                let (b, a) = (&before.graph, &after.graph);
                // The verdicts of the step: an undone split's routers are removed, a kept one's stay.
                let gone: Vec<&RouterTrial> = b.router_trials.iter().filter(|t| !a.router_trials.iter().any(|u| u.router == t.router && u.at == t.at)).collect();
                let undone: Vec<&RouterTrial> = gone.iter().copied().filter(|t| t.made.iter().all(|m| a.dead.contains(m))).collect();
                assert_eq!((undone.len() as u64, (gone.len() - undone.len()) as u64), (a.rallow.undone - b.rallow.undone, a.rallow.kept - b.rallow.kept));
                for t in &undone {
                    undos += 1;
                    let v = t.router;
                    assert_eq!(a.children(v), &t.children[..], "the router routes among its old children again");
                    assert_eq!(a.nodes[v].chain, t.chain);
                    assert!(a.nodes[v].grow.eligible_after_close >= a.params.grow.age_windows as u64 + 1);
                    for &p in t.parents.iter().skip(1) {
                        assert!(a.children(p).contains(&v), "an unsharing's parents point to the router again");
                    }
                }
                let split = a.rallow.splits > b.rallow.splits;
                let new_set = split && a.router_trials.last().unwrap().kind == SplitKind::Cluster;
                let dropped = undone.iter().filter(|t| t.set.is_some()).count();
                assert_eq!(a.sets.len() + dropped, b.sets.len() + new_set as usize, "a cluster split's set comes and goes with it");
                if a.sets.is_empty() {
                    assert_eq!(cell_count(after), 0, "no sets, no cells");
                }
                if split {
                    assert!(b.rallow.credits >= 1, "a split spends a credit");
                    let t = a.router_trials.last().unwrap();
                    assert_eq!(t.null.len(), match t.kind {
                        SplitKind::Path => t.parents.len(),
                        SplitKind::Cluster => 3,
                    });
                    if new_set && dropped == 0 {
                        // The new set's cells: one per node below the copies.
                        assert!(cell_count(after) > cell_count(before));
                    }
                    for &m in t.made.iter().chain([&t.router]) {
                        assert_eq!(a.nodes[m].chain, 1);
                    }
                }
                if !undone.is_empty() {
                    let mut copy = after.clone();
                    let (nodes, removed) = (copy.graph.nodes.len(), undone.iter().map(|t| t.made.len()).sum::<usize>());
                    copy.refresh();
                    assert!(copy.graph.nodes.len() <= nodes - removed, "the copies go at the next step");
                    crate::pathctx::check(&copy.graph).unwrap();
                }
            });
            eprintln!("z {z}: {run:?}");
            assert!(run.splits.len() >= 2 && run.checks >= 10, "{run:?}");
            // Credits: one per d_r root positions at most.
            assert!(run.splits.len() as u64 <= data.len() as u64 / d_r);
            // Each verdict exactly K windows of the region's examples after its split; no router in a chain at the cap
            // split.
            for &(_, _, _, examples) in &run.verdicts {
                assert_eq!(examples, k * w as u64);
            }
            assert!(run.splits.iter().all(|&(_, _, chain)| chain == 0), "{run:?}");
            // The kept-rate table counts every leaf verdict once, under a router split or elsewhere.
            let (r, g) = (bdd.graph.rallow, bdd.graph.grow);
            assert_eq!(r.leaf_under.0 + r.leaf_under.1 + r.leaf_elsewhere.0 + r.leaf_elsewhere.1, g.credits_spent - g.trials_in_flight as u64);
            assert_eq!(r.leaf_under.1 + r.leaf_elsewhere.1, g.credits_refunded);
            assert!(r.leaf_under.0 + r.leaf_under.1 >= 3, "{r:?}");
            match z == f64::INFINITY {
                true => assert!(run.verdicts.iter().all(|v| !v.2) && undos == 0 && bdd.graph.rallow.kept == run.verdicts.len() as u64),
                false => assert!(undos >= 1, "{run:?}"),
            }
            assert!(!run.verdicts.is_empty());
        }
    }

    /// The options' record: the encourage options print only under encourage (other rules' records as before).
    #[test]
    fn grow6_options_print_only_under_encourage() {
        let d = GrowParams::default();
        let changed = GrowParams { router_credit: 7, router_trial_windows: 9, router_trial_z: 1.5, router_chain: 3, ..d };
        assert_eq!(format!("{changed:?}"), format!("{d:?}"));
        let hg = GrowParams { router_rule: RouterSplitRule::StallHg, ..d };
        assert_eq!(format!("{:?}", GrowParams { router_credit: 7, ..hg }), format!("{hg:?}"));
        let enc = GrowParams { router_rule: RouterSplitRule::Encourage, ..d };
        let text = format!("{enc:?}");
        let tail = "router_rule: Encourage, router_z: 2.0, router_shadow: Auto, router_credit: 10000000, router_trial_windows: 5, router_trial_z: 2.0, router_chain: 1 }";
        assert!(text.ends_with(tail), "{text}");
        assert_ne!(format!("{:?}", GrowParams { router_credit: 7, ..enc }), text);
    }

    /// States: a run saved and read back every 997 steps (mid-window, mid-trial, with router splits on trial, undone
    /// and kept, leaf trials with their path tables, snapshots) trains on exactly as one process; other encourage
    /// options are refused.
    #[test]
    fn grow6_states_resume_exactly() {
        let mut p = encourage_params(1000, 3000, 0.0, 5000);
        p.grow.router_trial_z = -1e9;
        let data = xor3(2, 30_000);
        let mut one = Bdd::new(&[3; 4], p.clone());
        let mut chain = Bdd::new(&[3; 4], p.clone());
        let (mut mid_trial, mut mid_leaf) = (0, 0);
        for (i, (d, l)) in data.iter().enumerate() {
            one.train(d, l);
            chain.train(d, l);
            if i % 997 == 0 {
                let state = chain.to_state().into_bytes();
                chain = Bdd::from_state(&state, &[3; 4], p.clone()).unwrap();
                assert_eq!(chain.to_state().into_bytes(), state);
                mid_trial += !chain.graph.router_trials.is_empty() as usize;
                mid_leaf += chain.graph.nodes.iter().any(|n| n.trial.as_ref().is_some_and(|t| t.shadow_table.is_some())) as usize;
                let other = DelegationParams { grow: GrowParams { router_credit: 4999, ..p.grow }, ..p.clone() };
                assert!(Bdd::from_state(&state, &[3; 4], other).is_err());
            }
        }
        assert_eq!(one.to_state().into_bytes(), chain.to_state().into_bytes());
        assert!(one.graph.rallow.splits >= 2 && one.graph.rallow.undone >= 1, "{:?}", one.graph.rallow);
        assert!(mid_trial >= 3 && mid_leaf >= 1, "{mid_trial} saves mid router trial, {mid_leaf} mid leaf trial");
    }

    /// The router keep rule (6.80; ruled again in 6.88): after its K windows a router split stays unless its G_real -
    /// G_null is below -`router_trial_z` block standard errors: undone only on clear harm, no positive margin needed
    /// to keep it, and no MDL bar (the MDL bar, with path-table parameters, is the leaf trials' only).
    #[test]
    fn grow6_router_keep_rule_undoes_only_below_minus_z_se() {
        let mut p = encourage_params(1000, 3000, 0.0, 5000);
        p.grow.router_trial_z = 2.0;
        let mut bdd = Bdd::new(&[3; 4], p);
        for (d, l) in &xor3(2, 40_000) {
            bdd.train(d, l);
            if !bdd.graph.router_trials.is_empty() {
                break;
            }
        }
        assert_eq!(bdd.graph.router_trials.len(), 1, "a router split on trial");
        // An MDL bar this high would undo every split if the router verdict charged one.
        bdd.graph.params.grow.mdl = 1e6;
        // 100 blocks of the pairs d = loss(split's pick) - loss(null shadow's pick), mean `mean`, block SE `se`.
        let blocks = |mean: f64, se: f64| {
            let k = 100.0;
            Blocks { partial: 0.0, in_block: 0, count: 100, sum: k * mean, sum_sq: k * (mean * mean + se * se * (k - 1.0)) }
        };
        let se = 0.001;
        for (gain_z, undone) in [(-2.1, true), (-1.9, false), (-0.5, false), (0.0, false), (0.1, false), (1.9, false), (5.0, false)] {
            let mut b = bdd.clone();
            let mut trial = b.graph.router_trials.remove(0);
            trial.blocks = blocks(-gain_z * se, se);
            let (mean, s) = trial.blocks.stats();
            assert!((-mean / s - gain_z).abs() < 1e-9, "G {gain_z} SE: {mean} {s}");
            let made = trial.made.clone();
            let before = b.graph.rallow;
            b.router_verdict(trial);
            let r = b.graph.rallow;
            assert_eq!((r.undone - before.undone, r.kept - before.kept), (undone as u64, !undone as u64), "G {gain_z} SE");
            assert_eq!(made.iter().all(|m| b.graph.dead.contains(m)), undone, "G {gain_z} SE");
        }
        // Pairs without spread (se 0, as collapsed routing gives): kept, not behind.
        let mut b = bdd.clone();
        let mut trial = b.graph.router_trials.remove(0);
        trial.blocks = blocks(0.0, 0.0);
        b.router_verdict(trial);
        assert_eq!((b.graph.rallow.kept, b.graph.rallow.undone), (1, 0));
    }

    /// --router-chain 2 (6.88): a router that split, or a copy, may split once more (chain 1, below the cap), never a
    /// third time; an undone second-level split puts its router back at chain 1 (its first split stays); the
    /// structure holds throughout.
    #[test]
    fn grow6_router_chain_2_lets_a_split_router_split_once_more() {
        let data = xor3(2, 60_000);
        let mut p = encourage_params(1000, 3000, 0.0, 3000);
        p.grow.router_trial_z = f64::INFINITY;
        p.grow.router_chain = 2;
        let fresh = Bdd::new(&[3; 4], p);
        let run = encourage_run(&mut fresh.clone(), &data, 2_500, |_, after| assert!(after.graph.nodes.iter().all(|n| n.chain <= 2)));
        eprintln!("chain 2, every split kept: {run:?}");
        assert!(run.splits.iter().all(|&(_, _, chain)| chain < 2), "a router at chain 2 split: {run:?}");
        assert!(run.splits.iter().any(|&(_, _, chain)| chain == 1), "no second-level split: {run:?}");
        assert!(run.checks >= 20);
        // The same run, but the first second-level split is undone at its verdict.
        let mut bdd = fresh;
        let mut watched: Option<(usize, u64, Vec<usize>)> = None;
        let mut undone = false;
        for (d, l) in &data {
            let before = bdd.graph.rallow;
            bdd.train(d, l);
            if watched.is_none() {
                if let Some(t) = bdd.graph.router_trials.iter().find(|t| t.chain == 1) {
                    watched = Some((t.router, t.at, t.made.clone()));
                    bdd.graph.params.grow.router_trial_z = -1e9;
                }
                continue;
            }
            let (v, at, made) = watched.clone().unwrap();
            if bdd.graph.router_trials.iter().any(|t| t.router == v && t.at == at) {
                continue;
            }
            assert!(bdd.graph.rallow.undone > before.undone, "the second-level split was undone");
            assert_eq!(bdd.graph.nodes[v].chain, 1, "back at chain 1: its first split stays");
            assert!(made.iter().all(|m| bdd.graph.dead.contains(m)));
            let mut copy = bdd.clone();
            copy.refresh();
            crate::pathctx::check(&copy.graph).unwrap();
            undone = true;
            break;
        }
        assert!(undone, "no second-level split reached its verdict");
        for (d, l) in data.iter().take(5000) {
            bdd.train(d, l);
        }
        bdd.refresh();
        crate::pathctx::check(&bdd.graph).unwrap();
    }
}
