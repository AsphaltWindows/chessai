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
//!
//! With `DelegatorSplitOn::Regret` a delegator watches its routing regret
//! instead (the routed child's loss minus the best child's) and splits as a
//! leaf does. With `SubrouterTarget::Leaf` the parent of sub-routers learns
//! from the loss of the leaf each one routes to, as other error routers
//! learn from their children's predictions.
//!
//! bddtrig: with `DelegatorSplitOn::Skill` a delegator watches its routed
//! child's loss minus that of its single best child so far, and with
//! `router_split_z` a delegator splits only on a rise of its window
//! statistic by more than that many standard errors.
//!
//! Models are saved in the BDT file format.

use std::io;
use std::path::Path;

use crate::delegation::{
    DelegationParams, DelegatorSplitOn, Graph, Kind, NodeModel, RouteTarget, SplitRule, SubrouterTarget, Trial, at_least,
};
use crate::state::{Reader, Writer};

#[derive(Clone, Debug)]
pub struct Bdd {
    graph: Graph,
}

impl Bdd {
    pub fn new(categories: &[u8], params: DelegationParams) -> Self {
        Bdd { graph: Graph::new(categories, Self::checked(params)) }
    }

    /// A single leaf that starts as `root`, a classifier of the leaves' kind,
    /// instead of an untrained one.
    pub fn starting_from(root: NodeModel, categories: &[u8], params: DelegationParams) -> Self {
        Bdd { graph: Graph::starting_from(root, categories, Self::checked(params)) }
    }

    /// Revisable structure is for trees: a DAG's branches share nodes.
    fn checked(params: DelegationParams) -> DelegationParams {
        assert!(
            params.revise.is_none() && params.reset_leaves.is_none(),
            "revisable structure (revise, reset_leaves) is for BDTs only"
        );
        assert!(
            params.route == RouteTarget::Error
                || (params.delegator_split_on == DelegatorSplitOn::RouterError
                    && params.subrouter_target == SubrouterTarget::Routing),
            "delegator_split_on regret and subrouter_target leaf are for error routing"
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
            RouteTarget::Error => {
                let shared = self.shared_children(&children);
                let by_routing = self.graph.params.subrouter_target == SubrouterTarget::Routing;
                let child_errors = match &shared {
                    Some(shared) if by_routing => self.routing_errors(&children, shared, data, label, scratch),
                    _ => self.graph.prediction_errors(&children, data, label, scratch),
                };
                // The regret of the router's pick before it trains on the
                // example, over the children's predictions (cached this step).
                let regret = match self.graph.params.delegator_split_on {
                    DelegatorSplitOn::RouterError => None,
                    DelegatorSplitOn::Regret => Some({
                        let pick = self.graph.route(idx, data);
                        let losses = match &shared {
                            Some(_) if by_routing => self.graph.prediction_errors(&children, data, label, scratch),
                            _ => child_errors.clone(),
                        };
                        losses[pick] - losses.iter().copied().fold(f64::INFINITY, f64::min)
                    }),
                    // A': against every leaf reachable below, before any
                    // node below trains on the example.
                    DelegatorSplitOn::LeafRegret => Some(self.leaf_regret(idx, data, label, scratch)),
                    // 2a: against the single best child so far, over the
                    // same losses as `Regret`.
                    DelegatorSplitOn::Skill => Some({
                        let pick = self.graph.route(idx, data);
                        let losses = match &shared {
                            Some(_) if by_routing => self.graph.prediction_errors(&children, data, label, scratch),
                            _ => child_errors.clone(),
                        };
                        self.graph.nodes[idx].skill_error(pick, &losses)
                    }),
                };
                // B': where every sub-router reaches the same leaf, their
                // losses tie exactly; the delegator does not learn from that.
                let skip = shared.is_some() && self.graph.params.subrouter_target == SubrouterTarget::LeafSkip && {
                    let first = self.graph.reach_leaf(children[0], data);
                    children[1..].iter().all(|&c| self.graph.reach_leaf(c, data) == first)
                };
                let (routed, router_error) = if skip {
                    self.graph.router_pick_untrained(idx, data, &child_errors)
                } else {
                    self.graph.train_router(idx, data, &child_errors)
                };
                (routed, child_errors[routed], regret.unwrap_or(router_error))
            }
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
        let (recent, previous, margin) = node.close_window(g.params.router_split_z.unwrap_or(g.params.split_z));
        // The routers' mean error, or with Regret their mean regret.
        let routers_mean = g.router_errors.mean_since(node.window_routers);
        // 2b: with router_split_z every rule needs a rise of more than the
        // margin; otherwise the stall rules need a fall of less than it.
        let stalled = match g.params.router_split_z {
            Some(_) => recent - previous > margin,
            None => previous - recent < margin,
        };
        // A previous window exists unless forgotten (-1). Skill's means can be
        // negative, so it tests for the mark itself.
        let has_previous = match g.params.delegator_split_on {
            DelegatorSplitOn::Skill => previous != -1.0,
            _ => previous >= 0.0,
        };
        let split = has_previous
            && match (g.params.delegator_split_on, g.params.split_rule) {
                (DelegatorSplitOn::RouterError, _)
                | (
                    DelegatorSplitOn::Regret | DelegatorSplitOn::LeafRegret | DelegatorSplitOn::Skill,
                    SplitRule::StallHigh,
                ) => stalled && at_least(recent, routers_mean),
                (DelegatorSplitOn::Regret | DelegatorSplitOn::LeafRegret | DelegatorSplitOn::Skill, SplitRule::Worse) => {
                    recent > previous + margin
                }
            };
        if split {
            self.split_delegator(idx);
        }
        self.graph.open_window(idx);
        true
    }

    /// Turns delegator `idx` into a router over new delegators, copies of
    /// its router with the counts divided by `forget_factor`, that all route
    /// to its old children. With `split_trial`, one more such copy stands for
    /// the delegator without the split until the split's trial ends.
    fn split_delegator(&mut self, idx: usize) {
        let g = &mut self.graph;
        let Kind::Branch { children } = &g.nodes[idx].kind else { unreachable!() };
        let children = children.clone();
        let first = g.nodes.len();
        // C: the copies may forget more than leaf-split copies do.
        let forget = g.params.router_forget_factor.unwrap_or(g.params.forget_factor);
        for _ in 0..g.params.branch_factor {
            let old = &g.nodes[idx];
            let mut router = old.classifier.clone();
            router.forget(forget);
            let mut delegator = g.new_node(Kind::Branch { children: children.clone() }, router);
            delegator.error_router = old.error_router.as_ref().map(|e| e.forgetting_copy(forget));
            delegator.error_routed = old.error_routed;
            // Skill: the copies route among the old children, as it did.
            delegator.child_loss = old.child_loss.clone();
            g.nodes.push(delegator);
        }
        let made: Vec<usize> = (first..first + g.params.branch_factor).collect();
        let trial = g.params.split_trial.map(|_| {
            let mut shadow = g.nodes[idx].classifier.clone();
            shadow.forget(g.params.forget_factor);
            Trial::new(shadow, Some(children), g.nodes[idx].num_trained_total, made.clone())
        });

        let router = g.new_router();
        let error_router = g.new_error_router();
        let node = &mut g.nodes[idx];
        assert!(node.trial.is_none(), "a delegator split during its own trial");
        node.error_router = error_router;
        node.error_routed = false;
        node.child_loss.clear();
        node.kind = Kind::Branch { children: made };
        node.classifier = router;
        node.cache_step = 0;
        node.num_trained_total = 0;
        node.error_total = 0.0;
        node.forget_window();
        node.trial = trial;
        g.split_number += 1;
    }

    /// bddfix (A'): the loss of the leaf the position reaches from `idx`
    /// (every router's argmax, from this step's cached outputs, so before any
    /// node below trains on it), minus the best loss among all the leaves
    /// reachable below `idx`.
    fn leaf_regret(&mut self, idx: usize, data: &[u8], label: &[f64], scratch: &mut [f64]) -> f64 {
        let reached = self.graph.reach_leaf(idx, data);
        let leaves = self.reachable_leaves(idx);
        let losses = self.graph.prediction_errors(&leaves, data, label, scratch);
        let at = leaves.iter().position(|&l| l == reached).expect("the reached leaf is reachable");
        losses[at] - losses.iter().copied().fold(f64::INFINITY, f64::min)
    }

    /// The leaves reachable from node `idx`, in index order.
    fn reachable_leaves(&self, idx: usize) -> Vec<usize> {
        let mut seen = vec![false; self.graph.nodes.len()];
        let (mut stack, mut leaves) = (vec![idx], Vec::new());
        while let Some(n) = stack.pop() {
            if std::mem::replace(&mut seen[n], true) {
                continue;
            }
            match &self.graph.nodes[n].kind {
                Kind::Leaf => leaves.push(n),
                Kind::Branch { children } => stack.extend(children.iter().copied()),
            }
        }
        leaves.sort_unstable();
        leaves
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
        Ok(Bdd { graph: Graph::from_text(text, use_probs)? })
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
        Ok(Bdd { graph })
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
    use crate::delegation::{ErrorMeasure, ErrorTarget, NodeKind, RouteTarget, SplitRule};

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
            delegator_split_on: crate::delegation::DelegatorSplitOn::RouterError,
            subrouter_target: crate::delegation::SubrouterTarget::Routing,
            router_forget_factor: None,
            router_split_z: None,
        };
        let variants = [
            base.clone(),
            DelegationParams { leaf: NodeKind::Lr, split_trial: Some(0.0), ..base.clone() },
            DelegationParams { route: RouteTarget::Cluster, route_switch: Some(0.0), ..base.clone() },
            DelegationParams {
                error: ErrorMeasure::Log,
                delegator_split_on: DelegatorSplitOn::Regret,
                subrouter_target: SubrouterTarget::Leaf,
                ..base.clone()
            },
            DelegationParams {
                leaf: NodeKind::Lr,
                error: ErrorMeasure::Log,
                delegator_split_on: DelegatorSplitOn::Regret,
                ..base.clone()
            },
            DelegationParams {
                error: ErrorMeasure::Log,
                delegator_split_on: DelegatorSplitOn::LeafRegret,
                subrouter_target: SubrouterTarget::LeafSkip,
                router_forget_factor: Some(16.0),
                ..base.clone()
            },
            DelegationParams { error: ErrorMeasure::Log, delegator_split_on: DelegatorSplitOn::Skill, ..base.clone() },
            DelegationParams { router_split_z: Some(2.0), ..base.clone() },
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
            delegator_split_on: crate::delegation::DelegatorSplitOn::RouterError,
            subrouter_target: crate::delegation::SubrouterTarget::Routing,
            router_forget_factor: None,
            router_split_z: None,
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
            delegator_split_on: crate::delegation::DelegatorSplitOn::RouterError,
            subrouter_target: crate::delegation::SubrouterTarget::Routing,
            router_forget_factor: None,
            router_split_z: None,
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

    /// The tests' error-routed BDD with CNBP nodes, branch factor 2.
    fn regret_params(split_threshold: u32, split_limit: u32) -> DelegationParams {
        DelegationParams {
            class_num: 2,
            branch_factor: 2,
            split_threshold,
            split_limit,
            forget_factor: 1.0,
            nb_alpha: 1.0,
            use_probs: false,
            leaf: NodeKind::Cnbp,
            router: NodeKind::Cnbp,
            lr_rate: 0.05,
            error: ErrorMeasure::Log,
            split_z: 0.0,
            split_rule: SplitRule::Worse,
            route: RouteTarget::Error,
            error_target: ErrorTarget::Best,
            leaf_share: 0.0,
            split_trial: None,
            trial_error: ErrorMeasure::Log,
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
            delegator_split_on: DelegatorSplitOn::Regret,
            subrouter_target: SubrouterTarget::Routing,
                router_forget_factor: None,
                router_split_z: None,
        }
    }

    /// With `Regret`, a delegator's window sums the regret of its router's
    /// pick before each example trains it: the picked child's loss minus the
    /// best child's.
    #[test]
    fn regret_windows_sum_the_routers_regret() {
        // No splits of their own (limit 0): the test makes the one delegator.
        let mut bdd = Bdd::new(&[3, 3, 3, 3], regret_params(1_000_000, 0));
        bdd.graph.split_leaf(0);
        let (mut sum, mut positive) = (0.0, 0);
        for (data, label) in noisy(3000) {
            let mut probe = bdd.clone();
            let mut scratch = [0.0; 2];
            assert!(probe.graph.begin_step(&data, &label, &mut scratch).is_some());
            let errors = probe.graph.prediction_errors(&[1, 2], &data, &label, &mut scratch);
            let pick = probe.graph.route(0, &data);
            let regret = errors[pick] - errors.iter().copied().fold(f64::INFINITY, f64::min);
            assert!(regret >= 0.0);
            positive += (regret > 0.0) as u32;
            sum += regret;
            bdd.train(&data, &label);
        }
        let root = &bdd.graph.nodes[0];
        assert_eq!(root.num_trained_last, 3000);
        assert_eq!(root.error_last, sum);
        // The leaves diverged, so the router sometimes picked the worse one.
        assert!(positive > 0);
    }

    /// With `SubrouterTarget::Leaf`, a delegator over sub-routers trains on
    /// the error target of the losses of the leaves they route to.
    #[test]
    fn leaf_target_trains_on_the_sub_routers_leaves() {
        let params = DelegationParams { subrouter_target: SubrouterTarget::Leaf, ..regret_params(1_000_000, 0) };
        let mut bdd = Bdd::new(&[3, 3, 3, 3], params);
        bdd.graph.split_leaf(0);
        let data = noisy(6000);
        data[..2000].iter().for_each(|(d, l)| bdd.train(d, l));
        bdd.split_delegator(0);
        let children = bdd.graph.children(0).to_vec();
        assert!(bdd.shared_children(&children).is_some());
        let mut differs = 0;
        for (d, l) in &data[2000..] {
            let mut probe = bdd.clone();
            let mut scratch = [0.0; 2];
            probe.graph.begin_step(d, l, &mut scratch);
            let errors = probe.graph.prediction_errors(&children, d, l, &mut scratch);
            let target = probe.graph.error_target(&errors);
            let shared = probe.shared_children(&children).unwrap();
            let routing = probe.routing_errors(&children, &shared, d, l, &mut scratch);
            differs += (probe.graph.error_target(&routing) != target) as u32;
            let mut expected = bdd.graph.nodes[0].classifier.clone();
            expected.train(d, &target);
            bdd.train(d, l);
            let (mut a, mut b) = (String::new(), String::new());
            expected.write(&mut a);
            bdd.graph.nodes[0].classifier.write(&mut b);
            assert_eq!(a, b);
        }
        // The two targets disagree on some examples, so the test tells them apart.
        assert!(differs > 0, "the sub-routers never diverged");
    }

    /// Delegators split on their regret, and their children stay shared.
    #[test]
    fn delegators_split_on_regret() {
        let params = DelegationParams { subrouter_target: SubrouterTarget::Leaf, ..regret_params(50, 12) };
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
    }


    /// A' over leaves is `Regret`; over sub-routers sharing leaves it is the
    /// reached leaf's loss minus the best shared leaf's, so a new R' over
    /// fresh copies starts at its old router's regret.
    #[test]
    fn leaf_regret_measures_against_the_leaves_below() {
        let params = DelegationParams { delegator_split_on: DelegatorSplitOn::LeafRegret, ..regret_params(1_000_000, 0) };
        let mut bdd = Bdd::new(&[3, 3, 3, 3], params);
        bdd.graph.split_leaf(0);
        let data = noisy(6000);
        let mut old_sum = 0.0;
        for (d, l) in &data[..3000] {
            let mut probe = bdd.clone();
            let mut scratch = [0.0; 2];
            probe.graph.begin_step(d, l, &mut scratch);
            let errors = probe.graph.prediction_errors(&[1, 2], d, l, &mut scratch);
            let pick = probe.graph.route(0, d);
            old_sum += errors[pick] - errors.iter().copied().fold(f64::INFINITY, f64::min);
            bdd.train(d, l);
        }
        assert_eq!(bdd.graph.nodes[0].error_last, old_sum, "over leaves, A' is Regret");
        // Right after a router split every copy reaches the leaf the old router
        // picked, so R''s regret is the old router's, not zero.
        let before = bdd.clone();
        bdd.split_delegator(0);
        assert_eq!(bdd.reachable_leaves(0), vec![1, 2]);
        for (d, l) in &data[3000..3200] {
            let mut probe = bdd.clone();
            let mut old = before.clone();
            let mut scratch = [0.0; 2];
            probe.graph.begin_step(d, l, &mut scratch);
            old.graph.begin_step(d, l, &mut scratch);
            let new_regret = probe.leaf_regret(0, d, l, &mut scratch);
            let errors = old.graph.prediction_errors(&[1, 2], d, l, &mut scratch);
            let old_regret = errors[old.graph.route(0, d)] - errors.iter().copied().fold(f64::INFINITY, f64::min);
            assert!((new_regret - old_regret).abs() < 1e-12, "{new_regret} vs {old_regret}");
        }
    }

    /// B': R' does not learn from positions where its sub-routers reach the
    /// same leaf, and learns as `Leaf` does from the others.
    #[test]
    fn leaf_skip_skips_agreeing_sub_routers() {
        let params = DelegationParams { subrouter_target: SubrouterTarget::LeafSkip, ..regret_params(1_000_000, 0) };
        let mut bdd = Bdd::new(&[3, 3, 3, 3], params);
        bdd.graph.split_leaf(0);
        let data = noisy(8000);
        data[..2000].iter().for_each(|(d, l)| bdd.train(d, l));
        bdd.split_delegator(0);
        let children = bdd.graph.children(0).to_vec();
        let (mut skipped, mut trained) = (0, 0);
        for (d, l) in &data[2000..] {
            let mut probe = bdd.clone();
            let mut scratch = [0.0; 2];
            probe.graph.begin_step(d, l, &mut scratch);
            let reached: Vec<usize> = children.iter().map(|&c| probe.graph.reach_leaf(c, d)).collect();
            let mut expected = bdd.graph.nodes[0].classifier.clone();
            if reached.iter().all(|&r| r == reached[0]) {
                skipped += 1;
            } else {
                trained += 1;
                let errors = probe.graph.prediction_errors(&children, d, l, &mut scratch);
                expected.train(d, &probe.graph.error_target(&errors));
            }
            bdd.train(d, l);
            let (mut a, mut b) = (String::new(), String::new());
            expected.write(&mut a);
            bdd.graph.nodes[0].classifier.write(&mut b);
            assert_eq!(a, b);
        }
        assert!(skipped > 0 && trained > 0, "skipped {skipped}, trained {trained}");
    }

    /// C: a router split's copies forget by --router-forget-factor, a leaf
    /// split's copies by --forget-factor.
    #[test]
    fn router_forget_factor_divides_the_copies() {
        let params = DelegationParams { router_forget_factor: Some(16.0), ..regret_params(1_000_000, 0) };
        let mut bdd = Bdd::new(&[3, 3, 3, 3], params);
        bdd.graph.split_leaf(0);
        noisy(2000).iter().for_each(|(d, l)| bdd.train(d, l));
        let mut expected = bdd.graph.nodes[0].classifier.clone();
        expected.forget(16.0);
        let leaf = bdd.graph.nodes[1].classifier.clone();
        bdd.split_delegator(0);
        let (mut a, mut b) = (String::new(), String::new());
        expected.write(&mut a);
        let copy = bdd.graph.children(0)[1];
        bdd.graph.nodes[copy].classifier.write(&mut b);
        assert_eq!(a, b);
        // The leaves are shared, not copied, so they keep their counts.
        let (mut c, mut e) = (String::new(), String::new());
        leaf.write(&mut c);
        bdd.graph.nodes[1].classifier.write(&mut e);
        assert_eq!(c, e);
    }

    /// 2a: a Skill delegator's window sums its pick's loss minus that of
    /// the child with the smallest summed loss over the earlier examples.
    #[test]
    fn skill_windows_sum_the_loss_against_the_best_child_so_far() {
        let params = DelegationParams { delegator_split_on: DelegatorSplitOn::Skill, ..regret_params(1_000_000, 0) };
        let mut bdd = Bdd::new(&[3, 3, 3, 3], params);
        bdd.graph.split_leaf(0);
        let (mut sum, mut sums, mut nonzero) = (0.0, [0.0f64; 2], 0);
        for (data, label) in noisy(3000) {
            let mut probe = bdd.clone();
            let mut scratch = [0.0; 2];
            assert!(probe.graph.begin_step(&data, &label, &mut scratch).is_some());
            let errors = probe.graph.prediction_errors(&[1, 2], &data, &label, &mut scratch);
            let pick = probe.graph.route(0, &data);
            let best = if sums[1] < sums[0] { 1 } else { 0 };
            let e = errors[pick] - errors[best];
            nonzero += (e != 0.0) as u32;
            sum += e;
            sums[0] += errors[0];
            sums[1] += errors[1];
            bdd.train(&data, &label);
        }
        let root = &bdd.graph.nodes[0];
        assert_eq!(root.num_trained_last, 3000);
        assert!((root.error_last - sum).abs() < 1e-9, "{} vs {sum}", root.error_last);
        assert_eq!(root.child_loss, sums.to_vec());
        assert!(nonzero > 0);
        // A router split: R' starts afresh, its copies keep the sums.
        bdd.split_delegator(0);
        assert!(bdd.graph.nodes[0].child_loss.is_empty());
        let copy = bdd.graph.children(0)[1];
        assert_eq!(bdd.graph.nodes[copy].child_loss, sums.to_vec());
    }

    /// 2a: Skill delegators split too, although their windows' means are
    /// mostly negative (the router beats its single best child).
    #[test]
    fn delegators_split_on_skill() {
        let params = DelegationParams { delegator_split_on: DelegatorSplitOn::Skill, ..regret_params(50, 12) };
        let mut bdd = Bdd::new(&[3, 3, 3, 3], params);
        let mut negative = 0;
        for (data, label) in noisy(20_000) {
            bdd.train(&data, &label);
            negative += bdd.graph.nodes.iter().any(|n| matches!(n.kind, Kind::Branch { .. }) && n.error_prev < 0.0 && n.error_prev != -1.0) as u32;
        }
        assert!(negative > 0, "no negative skill window to test the mark on");
        let mut parents = vec![0; bdd.node_count()];
        for node in &bdd.graph.nodes {
            if let Kind::Branch { children } = &node.kind {
                children.iter().for_each(|&c| parents[c] += 1);
            }
        }
        assert!(parents.iter().any(|&p| p > 1), "no delegator split: {parents:?}");
    }

    /// 2b: with router_split_z 0 the delegators split exactly as without it,
    /// and a large one stops router splits but not leaf splits.
    #[test]
    fn router_split_z_needs_a_rise() {
        let base = DelegationParams { delegator_split_on: DelegatorSplitOn::RouterError, ..regret_params(50, 12) };
        let train = |params: DelegationParams| {
            let mut bdd = Bdd::new(&[3, 3, 3, 3], params);
            noisy(20_000).iter().for_each(|(d, l)| bdd.train(d, l));
            bdd
        };
        let plain = train(base.clone());
        let zero = train(DelegationParams { router_split_z: Some(0.0), ..base.clone() });
        assert_eq!(plain.to_text(), zero.to_text());
        let shared = |bdd: &Bdd| {
            let mut parents = vec![0; bdd.node_count()];
            for node in &bdd.graph.nodes {
                if let Kind::Branch { children } = &node.kind {
                    children.iter().for_each(|&c| parents[c] += 1);
                }
            }
            parents.iter().filter(|&&p| p > 1).count()
        };
        assert!(shared(&plain) > 0, "no router split without the margin");
        let high = train(DelegationParams { router_split_z: Some(1e6), ..base });
        assert_eq!(shared(&high), 0);
        assert!(high.split_count() > 0);
    }

    #[test]
    #[should_panic(expected = "are for error routing")]
    fn regret_needs_error_routing() {
        Bdd::new(&[3, 3, 3, 3], DelegationParams { route: RouteTarget::Cluster, ..regret_params(50, 12) });
    }
}
