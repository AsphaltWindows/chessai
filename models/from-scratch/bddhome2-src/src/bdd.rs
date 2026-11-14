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

use crate::delegation::{DelegationParams, Graph, Kind, NodeModel, RouteTarget, SubrouterTarget, Trial, at_least};
use crate::state::{Reader, Writer};

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
        params
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
        let error = self.graph.begin_step(data, label, &mut scratch);
        self.train_node(0, data, label, error, &mut scratch);
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
        let node = &mut g.nodes[idx];
        if node.num_trained_last < g.params.split_threshold || g.split_number >= g.params.split_limit {
            return false;
        }
        let (recent, previous, margin) = node.close_window(g.params.split_z);
        let routers_mean = g.router_errors.mean_since(node.window_routers);
        if previous >= 0.0 && previous - recent < margin && at_least(recent, routers_mean) {
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
        let node = &mut g.nodes[idx];
        assert!(node.trial.is_none(), "a delegator split during its own trial");
        node.error_router = error_router;
        node.error_routed = false;
        node.kind = Kind::Branch { children: made };
        node.classifier = router;
        node.cache_step = 0;
        node.num_trained_total = 0;
        node.error_total = 0.0;
        node.forget_window();
        node.trial = trial;
        g.split_number += 1;
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
            self.home_cache = HomeCache { key, homes: vec![None; self.graph.nodes.len()] };
        }
        if let Some(home) = &self.home_cache.homes[idx] {
            return home.clone();
        }
        let base = self.home_base(idx);
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
}
