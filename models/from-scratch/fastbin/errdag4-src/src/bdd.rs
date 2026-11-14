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

use crate::delegation::{DelegationParams, Graph, Kind, NodeKind, NodeModel, RouteTarget, SubrouterTarget, Trial, at_least};
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
            return self.train_node_ctx(0, &mut Vec::new(), data, label, error, &mut scratch);
        }
        let error = self.graph.begin_step(data, label, &mut scratch);
        self.train_node(0, data, label, error, &mut scratch);
    }

    /// `train_node` with path context (see `pathctx`): the position entered node `idx` with `marks`, which are as
    /// they were on return. Every prediction it asks for is for the path it would take: a router scoring its
    /// children enters each child with the child's mark, and so on down. Without split trials and cluster routing.
    fn train_node_ctx(&mut self, idx: usize, marks: &mut Vec<Mark>, data: &[u8], label: &[f64], error: Option<f64>, scratch: &mut [f64]) {
        let g = &mut self.graph;
        let Kind::Branch { children } = &g.nodes[idx].kind else {
            if let Some(split) = g.check_leaf_split(idx) {
                if split {
                    g.nodes[idx].forget_window();
                }
                return self.train_node_ctx(idx, marks, data, label, error, scratch);
            }
            #[cfg(test)]
            self.check_training_path(idx, marks);
            return self.graph.train_leaf_ctx(idx, marks, data, label, error);
        };
        let children = children.clone();
        if self.check_delegator_split(idx) {
            return self.train_node_ctx(idx, marks, data, label, error, scratch);
        }
        let len = marks.len();
        if error.is_none() {
            let routed = self.graph.route_ctx(idx, marks, data);
            self.graph.enter(children[routed], marks);
            #[cfg(test)]
            ACTUAL_PATH.with(|p| p.borrow_mut().push(children[routed]));
            self.train_node_ctx(children[routed], marks, data, label, None, scratch);
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
                let child_errors = match self.shared_children(&children) {
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
                let (routed, router_error) = self.graph.train_router_ctx(idx, marks, data, &child_errors);
                (routed, child_errors[routed], router_error)
            }
        };
        self.graph.enter(children[routed], marks);
        #[cfg(test)]
        ACTUAL_PATH.with(|p| p.borrow_mut().push(children[routed]));
        self.train_node_ctx(children[routed], marks, data, label, Some(child_error), scratch);
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
}
