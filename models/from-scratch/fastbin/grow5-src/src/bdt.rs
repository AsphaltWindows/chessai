//! Delegation Tree, a port of `c_src/models/bdt/bdt.c` (the Bayesian
//! Delegation Tree) generalised to other classifiers.
//!
//! A tree of classifiers, CNBPs or logistic regressions (`NodeModel`).
//! Leaves predict the classes. Branches predict which of their children will
//! classify an input best and delegate to it.
//!
//! Training a branch asks every child for a prediction, trains the branch's
//! own classifier with the index of the child with the smallest error as the
//! label, and then trains the child the updated classifier picks (with
//! `RouteTarget::Cluster` the label is the branch's own pick, so only the
//! picked child is asked). A leaf
//! splits into a branch with `branch_factor` children when its split rule
//! says so, checked every `split_threshold` examples (by default: when its
//! error over the last window is above the previous window's). The children
//! start as copies of the leaf that forget by `forget_factor` (a CNBP's
//! counts are divided by it), so they predict the same but adapt faster.
//! With residual leaves (`DelegationParams::leaf_residual`) they start as
//! corrections of 0 on top of the split node's shadow instead, which goes on
//! learning from all of the node's examples.
//!
//! With CNBP nodes, training order, tie-breaking and file format follow the
//! C version exactly, so both produce identical trees and predictions. The
//! nodes, their statistics and the file format are shared with the BDD
//! (`delegation`).

use std::io;
use std::path::Path;

use crate::delegation::{DelegationParams, Graph, Kind, NodeModel, ReviseOn, RouteTarget};
use crate::state::{Reader, Writer};

#[derive(Clone, Debug)]
pub struct Bdt {
    graph: Graph,
}

impl Bdt {
    pub fn new(categories: &[u8], params: DelegationParams) -> Self {
        Bdt { graph: Graph::new(categories, Self::checked(params)) }
    }

    /// A single leaf that starts as `root`, a classifier of the leaves' kind,
    /// instead of an untrained one.
    pub fn starting_from(root: NodeModel, categories: &[u8], params: DelegationParams) -> Self {
        Bdt { graph: Graph::starting_from(root, categories, Self::checked(params)) }
    }

    /// A tree's delegators never share children: sub-router targets are for BDDs.
    fn checked(params: DelegationParams) -> DelegationParams {
        assert!(
            params.subrouter_target == crate::delegation::SubrouterTarget::Routing,
            "subrouter_target is for BDDs"
        );
        // Only router splits make paths that meet again.
        assert!(!params.path_context, "path_context is for BDDs");
        assert!(params.router_split_z.is_none(), "router_split_z is for BDDs");
        assert!(params.router_split_limit.is_none(), "router_split_limit is for BDDs");
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
    /// With `use_probs`, if the tree was trained since the last `refresh()`.
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

    /// Index of the leaf that predicts the position when routed from node
    /// `idx`.
    pub fn leaf_from(&self, idx: usize, data: &[u8]) -> usize {
        self.graph.leaf_from(idx, data)
    }

    /// Node `idx`'s children; empty for a leaf.
    pub fn children(&self, idx: usize) -> &[usize] {
        self.graph.children(idx)
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

    /// Lets every model in the tree adapt faster (see `Graph::forget`).
    pub fn forget(&mut self, factor: f64) {
        self.graph.forget(factor)
    }

    /// Adds one data point. `label` must sum to 1.
    pub fn train(&mut self, data: &[u8], label: &[f64]) {
        let mut scratch = vec![0.0; self.graph.params.class_num];
        let error = self.graph.begin_step(data, label, &mut scratch);
        self.train_node(0, None, data, label, error, &mut scratch);
        // grow1: the growth rule's credits (nothing without it).
        self.graph.end_step();
    }

    /// `parent` is the delegator above node `idx` (None for the root).
    /// `error` is the error of the node's prediction for this example, or
    /// None if the example is not scored; only leaves use it, branches record
    /// the error of their own choice. `scratch` holds `class_num` values.
    fn train_node(
        &mut self,
        idx: usize,
        parent: Option<usize>,
        data: &[u8],
        label: &[f64],
        error: Option<f64>,
        scratch: &mut [f64],
    ) {
        let g = &mut self.graph;
        let Kind::Branch { children } = &g.nodes[idx].kind else {
            if g.check_leaf_split(idx).is_some() {
                // The leaf may have become a branch; train it as it is now.
                return self.train_node(idx, parent, data, label, error, scratch);
            }
            return g.train_leaf_below(idx, parent, data, label, error);
        };
        let children = children.clone();
        if g.trial_step(idx, data, label, scratch) || g.revise_step(idx, data, label, scratch) {
            // The split was undone or merged; train the node as it is now.
            return self.train_node(idx, parent, data, label, error, scratch);
        }
        if error.is_none() {
            let routed = g.route(idx, data);
            g.train_shadow(idx, data, label);
            return self.train_node(children[routed], Some(idx), data, label, None, scratch);
        }

        // Error routing compares every child's error; cluster routing needs
        // only the error of the child it picks.
        let (routed, child_error, router_error) = match g.params.route {
            RouteTarget::Error => {
                let child_errors = g.prediction_errors(&children, data, label, scratch);
                let (routed, router_error) = g.train_router(idx, data, &child_errors);
                (routed, child_errors[routed], router_error)
            }
            RouteTarget::Cluster => {
                let (routed, router_error) = g.train_cluster_router(idx, &children, data, label, scratch);
                (routed, g.prediction_errors(&children[routed..=routed], data, label, scratch)[0], router_error)
            }
        };
        // Residual leaves' shadows train here, once the children predicted.
        g.train_shadow(idx, data, label);
        self.train_node(children[routed], Some(idx), data, label, Some(child_error), scratch);
        self.graph.record_router(idx, router_error);
    }

    /// Sees a round's games, each the positions in the order played, before
    /// training on them: with shadows, measures the seams their test needs
    /// (see `DelegationParams::revise`). Call it with the tree refreshed.
    pub fn measure_seams<P: AsRef<[u8]>>(&mut self, games: &[&[P]]) {
        self.graph.measure_seams(games)
    }

    /// Sees a self-play round's positions, in training order with their
    /// labels and games, before training on them: scores the shadow test on
    /// them (see `Graph::observe_round`).
    pub fn observe_round<P: AsRef<[u8]>, L: AsRef<[f64]>>(&mut self, data: &[(P, L, u64)]) {
        self.graph.observe_round(data)
    }

    /// After training on a self-play round: logs its fresh and stream gains
    /// (see `Graph::end_round`).
    pub fn end_round(&self) {
        self.graph.end_round()
    }

    /// Whether it scores rounds before training on them
    /// (`DelegationParams::log_fresh_gains`, `revise_on` fresh).
    pub fn scores_rounds(&self) -> bool {
        let p = &self.graph.params;
        p.log_fresh_gains || p.revise_on != ReviseOn::Stream
    }

    /// `train` for a position of game `game`, which `DelegationParams::
    /// log_fresh_gains` counts gains by.
    pub fn train_in_game(&mut self, data: &[u8], label: &[f64], game: u64) {
        self.graph.game = Some(game);
        self.train(data, label);
    }

    /// Serializes in the C text format (`bdt_to_file`).
    pub fn to_text(&self) -> String {
        self.graph.to_text()
    }

    /// Parses the C text format (`bdt_from_file`).
    pub fn from_text(text: &str, use_probs: bool) -> io::Result<Self> {
        Ok(Bdt { graph: Graph::from_text(text, use_probs)? })
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
        let mut w = Writer::new("bdt");
        self.graph.write_state(&mut w);
        w
    }

    /// Reads a state written by `to_state` by a model trained with `params`
    /// over `categories`.
    pub fn from_state(data: &[u8], categories: &[u8], params: DelegationParams) -> io::Result<Self> {
        let mut r = Reader::new(data, "bdt")?;
        let graph = Graph::read_state(&mut r, categories, params)?;
        r.finish()?;
        Ok(Bdt { graph })
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
    use crate::cnbp::Cnbp;
    use crate::delegation::{ErrorMeasure, ErrorTarget, NodeKind, ReviseOn, RouteTarget, SplitRule, log_loss};
    use crate::logreg::Logreg;
    use crate::probit::ProbitParams;

    fn params(use_probs: bool) -> DelegationParams {
        DelegationParams {
            class_num: 2,
            branch_factor: 2,
            split_threshold: 4,
            split_limit: 3,
            forget_factor: 10.0,
            nb_alpha: 1.0,
            use_probs,
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
            grow: crate::delegation::GrowParams::default(),
        }
    }

    /// Two clusters with opposite labels, fed in blocks so leaves stall and split.
    fn trained(use_probs: bool) -> Bdt {
        trained_with(params(use_probs))
    }

    fn trained_with(params: DelegationParams) -> Bdt {
        let mut bdt = Bdt::new(&[3, 3], params);
        for i in 0..40u8 {
            let (data, label) = if (i / 8) % 2 == 0 { ([0, i % 3], [1.0, 0.0]) } else { ([2, i % 3], [0.0, 1.0]) };
            bdt.train(&data, &label);
        }
        bdt.refresh();
        bdt
    }

    #[test]
    fn splits_and_predicts_distribution() {
        let bdt = trained(false);
        assert!(bdt.split_count() > 0);
        assert_eq!(bdt.node_count(), 1 + 2 * bdt.split_count() as usize);
        let mut out = [0.0; 2];
        bdt.predict(&[0, 1], &mut out);
        assert!((out[0] + out[1] - 1.0).abs() < 1e-12);
    }

    /// A tree read back from its state trains on exactly as the original
    /// (`--resume`), whatever it was doing: splits, trials, error routers,
    /// shadows and their windows. Other options are refused.
    #[test]
    fn state_round_trip_trains_on_identically() {
        let base = DelegationParams { split_threshold: 40, split_limit: 6, forget_factor: 1.0, ..params(true) };
        let variants = [
            base.clone(),
            DelegationParams { leaf: NodeKind::Lr, error: ErrorMeasure::Log, ..base.clone() },
            DelegationParams { split_trial: Some(0.0), ..base.clone() },
            DelegationParams { route: RouteTarget::Cluster, route_switch: Some(0.0), ..base.clone() },
            DelegationParams { leaf: NodeKind::Lr, router: NodeKind::Tan, error: ErrorMeasure::Log, ..base.clone() },
            DelegationParams {
                leaf: NodeKind::Lr,
                router: NodeKind::Tan,
                route: RouteTarget::Cluster,
                revise: Some(0.0),
                ..base.clone()
            },
            DelegationParams {
                leaf: NodeKind::Lr,
                route: RouteTarget::Cluster,
                route_switch: Some(0.0),
                error_router: NodeKind::Tan,
                ..base.clone()
            },
            DelegationParams {
                leaf: NodeKind::Lr,
                route: RouteTarget::Cluster,
                route_switch: Some(0.0),
                error_router: NodeKind::Lr,
                ..base.clone()
            },
            DelegationParams { revise: Some(0.0), reset_leaves: Some(0.0), seam_weight: 0.1, ..base.clone() },
            DelegationParams { leaf: NodeKind::Lr, router: NodeKind::Lr, revise: Some(f64::INFINITY), ..base.clone() },
            DelegationParams {
                leaf: NodeKind::Lr,
                error: ErrorMeasure::Log,
                revise: Some(0.0),
                leaf_residual: true,
                ..base.clone()
            },
            DelegationParams {
                leaf: NodeKind::Lr,
                router: NodeKind::Lr,
                route: RouteTarget::Cluster,
                reset_leaves: Some(0.0),
                leaf_residual: true,
                residual_l2: 0.01,
                ..base.clone()
            },
            DelegationParams {
                leaf: NodeKind::Lr,
                error: ErrorMeasure::Log,
                revise: Some(0.0),
                reset_leaves: Some(0.0),
                leaf_residual: true,
                residual_inherit_sums: true,
                ..base.clone()
            },
            DelegationParams {
                leaf: NodeKind::Lr,
                error: ErrorMeasure::Log,
                revise: Some(0.0),
                revise_on: ReviseOn::Fresh,
                log_fresh_gains: true,
                ..base.clone()
            },
            DelegationParams {
                leaf: NodeKind::Lr,
                error: ErrorMeasure::Log,
                revise: Some(0.0),
                reset_leaves: Some(0.0),
                leaf_residual: true,
                residual_inherit_sums: true,
                residual_delta_rate: 0.5,
                revise_on: ReviseOn::Fresh,
                ..base.clone()
            },
            // grow1: the growth rule, its trials kept (bars below any gain) or undone (above), with credits to spare
            // and states saved in the middle of windows, blocks and trials.
            DelegationParams {
                leaf: NodeKind::Lr,
                error: ErrorMeasure::Log,
                trial_error: ErrorMeasure::Log,
                split_rule: SplitRule::Grow,
                split_limit: 1_000_000,
                grow: crate::delegation::GrowParams { credit: 60, trial_z: -1e30, mdl: -1e30, ..Default::default() },
                ..base.clone()
            },
            DelegationParams {
                leaf: NodeKind::Lr,
                error: ErrorMeasure::Log,
                trial_error: ErrorMeasure::Log,
                split_rule: SplitRule::Grow,
                split_limit: 1_000_000,
                grow: crate::delegation::GrowParams { credit: 60, trial_z: f64::INFINITY, stall_windows: 1, ..Default::default() },
                ..base.clone()
            },
        ];
        for params in variants {
            let mut bdt = Bdt::new(&[3; 4], params.clone());
            let mut split = false;
            for round in crate::delegation::state_test_rounds() {
                let state = bdt.to_state().into_bytes();
                let mut back = Bdt::from_state(&state, &[3; 4], params.clone()).unwrap();
                assert_eq!(back.to_state().into_bytes(), state);
                assert_eq!(back.to_text(), bdt.to_text());
                for tree in [&mut bdt, &mut back] {
                    let games: Vec<&[[u8; 4]]> = round.iter().map(|(game, _)| game.as_slice()).collect();
                    tree.measure_seams(&games);
                    let data: Vec<([u8; 4], [f64; 2], u64)> = (round.iter().enumerate())
                        .flat_map(|(g, (game, label))| game.iter().map(move |&pos| (pos, *label, g as u64)))
                        .collect();
                    tree.observe_round(&data);
                    data.iter().for_each(|(pos, label, game)| tree.train_in_game(pos, label, *game));
                    tree.end_round();
                    tree.refresh();
                }
                assert!(back.to_state().into_bytes() == bdt.to_state().into_bytes(), "{params:?}");
                // grow1: or a growth trial was made (undone ones leave no split).
                split |= bdt.split_count() > 0 || bdt.graph.grow.credits_spent > 0;
            }
            // Shadows may have replaced the branches again.
            assert!(split, "{params:?}");
            let other = DelegationParams { split_limit: 7, ..params };
            assert!(Bdt::from_state(&bdt.to_state().into_bytes(), &[3; 4], other).is_err());
        }
    }

    #[test]
    fn text_round_trip() {
        for use_probs in [false, true] {
            let bdt = trained(use_probs);
            let back = Bdt::from_text(&bdt.to_text(), use_probs).unwrap();
            assert_eq!(bdt.to_text(), back.to_text());
            let (mut a, mut b) = ([0.0; 2], [0.0; 2]);
            bdt.predict(&[2, 0], &mut a);
            back.predict(&[2, 0], &mut b);
            assert_eq!(a, b);
        }
    }

    #[test]
    fn leaf_weights_blend_the_leaves() {
        let bdt = trained(false);
        let leaves: Vec<usize> = bdt.leaves().map(|(i, _)| i).collect();
        let mut weights = Vec::new();
        for data in [[0, 0], [0, 2], [2, 1], [1, 1]] {
            for tau in [1.0, 0.3, 0.0] {
                bdt.leaf_weights(&data, tau, 0.0, &mut weights);
                let total: f64 = weights.iter().map(|(_, w)| w).sum();
                assert!((total - 1.0).abs() < 1e-12, "{weights:?}");
                assert_eq!(weights.len(), leaves.len());
                assert!(weights.iter().all(|(leaf, _)| leaves.contains(leaf)));
            }
            // At tau 1 the heaviest leaf is the one hard routing picks.
            bdt.leaf_weights(&data, 1.0, 0.0, &mut weights);
            let heaviest = weights.iter().max_by(|a, b| a.1.total_cmp(&b.1)).unwrap().0;
            assert_eq!(heaviest, bdt.leaf(&data));
            // At tau 0 every router splits its weight evenly.
            bdt.leaf_weights(&data, 0.0, 0.0, &mut weights);
            assert!(weights.iter().all(|(_, w)| [0.5, 0.25, 0.125, 0.0625].iter().any(|x| (w - x).abs() < 1e-12)));
        }
    }

    #[test]
    fn cluster_routing_separates_groups_without_labels() {
        // Two groups of inputs, values 0-1 or 2-3 on every feature, with
        // coin-flip labels: only the inputs tell them apart.
        let mut x = 99u64;
        let mut next = |m: u64| {
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (x >> 33) % m
        };
        let mut example = || {
            let group = next(2) as u8;
            let data: [u8; 6] = std::array::from_fn(|_| 2 * group + next(2) as u8);
            let label = if next(2) == 0 { [1.0, 0.0] } else { [0.0, 1.0] };
            (group, data, label)
        };
        let params = DelegationParams {
            split_threshold: 500,
            split_limit: 1,
            split_rule: SplitRule::StallHigh,
            route: RouteTarget::Cluster,
            ..params(false)
        };
        let mut bdt = Bdt::new(&[4; 6], params);
        for _ in 0..20_000 {
            let (_, data, label) = example();
            bdt.train(&data, &label);
        }
        bdt.refresh();
        assert_eq!(bdt.split_count(), 1);
        let mut leaves = [[0; 2]; 2];
        for _ in 0..1000 {
            let (group, data, _) = example();
            leaves[group as usize][bdt.leaf(&data) - 1] += 1;
        }
        let pure = |[a, b]: [i32; 2]| a.max(b) as f64 / (a + b) as f64;
        assert!(pure(leaves[0]) > 0.95 && pure(leaves[1]) > 0.95, "{leaves:?}");
        assert_ne!(leaves[0][0] > leaves[0][1], leaves[1][0] > leaves[1][1], "{leaves:?}");
    }

    /// Inputs in two groups (feature 0 and four noisy copies of it) and a
    /// label that follows feature 5 with 10% noise; if `flipped`, the second
    /// group's label is the opposite, which naive Bayes can only learn
    /// within each group.
    fn grouped_example(next: &mut impl FnMut(u64) -> u64, flipped: bool) -> ([u8; 6], [f64; 2]) {
        let group = next(2) as u8;
        let mut data = [group; 6];
        for value in &mut data[1..5] {
            if next(10) == 0 {
                *value = 1 - group;
            }
        }
        data[5] = next(2) as u8;
        let mut class = data[5] ^ (group & flipped as u8);
        if next(10) == 0 {
            class = 1 - class;
        }
        (data, if class == 0 { [1.0, 0.0] } else { [0.0, 1.0] })
    }

    #[test]
    fn split_trial_keeps_only_splits_that_help() {
        for flipped in [true, false] {
            let mut x = 5u64;
            let mut next = |m: u64| {
                x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                (x >> 33) % m
            };
            // No splits of its own (limit 0): the test splits the root.
            let params = DelegationParams {
                split_threshold: 4000,
                split_limit: 0,
                route: RouteTarget::Cluster,
                split_trial: Some(0.0),
                ..params(false)
            };
            let mut bdt = Bdt::new(&[2; 6], params);
            bdt.graph.split_leaf(0);
            for _ in 0..=4000 {
                let (data, label) = grouped_example(&mut next, flipped);
                bdt.train(&data, &label);
            }
            bdt.refresh();
            // The groups need leaves of their own only when the label flips.
            assert_eq!(bdt.node_count(), if flipped { 3 } else { 1 }, "flipped {flipped}");
            assert_eq!(bdt.split_count(), flipped as u32);
            assert!(bdt.graph.nodes.iter().all(|n| n.trial.is_none()));
        }
    }

    #[test]
    fn revision_merges_branches_that_do_not_help() {
        for flipped in [true, false] {
            let mut x = 5u64;
            let mut next = |m: u64| {
                x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                (x >> 33) % m
            };
            // No splits of its own (limit 0): the test splits the root.
            let params = DelegationParams {
                split_threshold: 4000,
                split_limit: 0,
                route: RouteTarget::Cluster,
                revise: Some(0.0),
                ..params(false)
            };
            let mut bdt = Bdt::new(&[2; 6], params);
            bdt.graph.split_leaf(0);
            // Three windows.
            for _ in 0..=12_000 {
                let (data, label) = grouped_example(&mut next, flipped);
                bdt.train(&data, &label);
            }
            bdt.refresh();
            // The groups need leaves of their own only when the label flips.
            assert_eq!(bdt.node_count(), if flipped { 3 } else { 1 }, "flipped {flipped}");
            assert_eq!(bdt.split_count(), flipped as u32);
            assert_eq!(bdt.graph.nodes[0].revision.is_some(), flipped);
        }
    }

    /// A cluster-routed merge tree whose root has split (the test splits it)
    /// and whose thresholds always switch to error routing and always merge
    /// once a window ends, so their order shows; and `train`, which feeds it
    /// `n` examples.
    fn switching_merge_tree(route_switch: f64) -> (Bdt, impl FnMut(&mut Bdt, u32)) {
        let mut x = 5u64;
        let mut next = move |m: u64| {
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (x >> 33) % m
        };
        let params = DelegationParams {
            split_threshold: 4000,
            split_limit: 0,
            route: RouteTarget::Cluster,
            route_switch: Some(route_switch),
            revise: Some(-1e9),
            ..params(false)
        };
        let mut bdt = Bdt::new(&[2; 6], params);
        bdt.graph.split_leaf(0);
        let train = move |bdt: &mut Bdt, n: u32| {
            for _ in 0..n {
                let (data, label) = grouped_example(&mut next, true);
                bdt.train(&data, &label);
            }
        };
        (bdt, train)
    }

    fn merged(bdt: &Bdt) -> bool {
        matches!(bdt.graph.nodes[0].kind, Kind::Leaf)
    }

    fn revision_examples(bdt: &Bdt) -> u32 {
        bdt.graph.nodes[0].revision.as_ref().expect("a shadow").window.examples
    }

    #[test]
    fn a_switch_to_error_routing_gets_a_full_window_before_any_merge_check() {
        // The comparison of the routers and the shadow's window start
        // together at the split. The routers' ends first and switches; the
        // branch has only been cluster-routed, so the shadow's window starts
        // afresh, and the merge check comes a full window later.
        let (mut bdt, mut train) = switching_merge_tree(-1e9);
        train(&mut bdt, 4000);
        assert!(bdt.graph.nodes[0].error_routed, "the first window switches");
        assert_eq!(revision_examples(&bdt), 0);
        train(&mut bdt, 4000);
        assert!(!merged(&bdt), "no merge check within a full window of error-routed examples");
        assert_eq!(revision_examples(&bdt), 4000);
        train(&mut bdt, 1);
        assert!(merged(&bdt), "the next example ends the window, which merges");
    }

    #[test]
    fn the_switch_is_decided_before_a_merge_check_on_the_same_example() {
        for switch in [true, false] {
            // The comparison of the routers misses the first example, so the
            // shadow's window is full when the comparison's last example
            // comes: the switch is decided first.
            let (mut bdt, mut train) = switching_merge_tree(if switch { -1e9 } else { 1e9 });
            let error_router = bdt.graph.nodes[0].error_router.take();
            train(&mut bdt, 1);
            bdt.graph.nodes[0].error_router = error_router;
            train(&mut bdt, 4000);
            assert!(!merged(&bdt), "switch {switch}");
            assert_eq!(bdt.graph.nodes[0].error_routed, switch);
            if switch {
                // The window starts afresh.
                assert_eq!(revision_examples(&bdt), 0);
            } else {
                // Without a switch, the window ends with the next example.
                assert_eq!(revision_examples(&bdt), 4001);
                train(&mut bdt, 1);
                assert!(merged(&bdt));
            }
        }
    }

    #[test]
    fn revision_merges_whole_branches_whose_seams_cost_more_than_they_gain() {
        for seam_weight in [0.0, 100.0] {
            let mut x = 5u64;
            let mut next = |m: u64| {
                x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                (x >> 33) % m
            };
            let params = DelegationParams {
                split_threshold: 4000,
                split_limit: 0,
                route: RouteTarget::Cluster,
                revise: Some(0.0),
                seam_weight,
                ..params(false)
            };
            let mut bdt = Bdt::new(&[2; 6], params);
            bdt.graph.split_leaf(0);
            // A branch within the branch.
            bdt.graph.split_leaf(1);
            for i in 0..=4000 {
                if i == 2000 {
                    // Games that alternate between the groups, which the
                    // root's leaves predict oppositely.
                    let games: Vec<Vec<[u8; 6]>> = (0..10)
                        .map(|_| {
                            (0..20u8)
                                .map(|j| loop {
                                    let (data, _) = grouped_example(&mut next, true);
                                    if data[0] == j % 2 {
                                        break data;
                                    }
                                })
                                .collect()
                        })
                        .collect();
                    bdt.refresh();
                    bdt.measure_seams(&games.iter().map(|g| g.as_slice()).collect::<Vec<_>>());
                }
                let (data, label) = grouped_example(&mut next, true);
                bdt.train(&data, &label);
            }
            bdt.refresh();
            let merged = seam_weight > 0.0;
            assert_eq!(bdt.node_count(), if merged { 1 } else { 5 }, "seam weight {seam_weight}");
            assert_eq!(bdt.split_count(), if merged { 0 } else { 2 });
        }
    }

    #[test]
    fn reset_leaves_replaces_a_leaf_worse_than_the_shadow() {
        for reset_leaves in [None, Some(0.0)] {
            let mut x = 11u64;
            let mut next = |m: u64| {
                x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                (x >> 33) % m
            };
            let params = DelegationParams {
                split_threshold: 1000,
                split_limit: 0,
                route: RouteTarget::Cluster,
                reset_leaves,
                ..params(false)
            };
            let mut bdt = Bdt::new(&[2; 6], params);
            for _ in 0..2000 {
                let (data, label) = grouped_example(&mut next, false);
                bdt.train(&data, &label);
            }
            bdt.graph.split_leaf(0);
            // Leaf 2 learns the opposite labels.
            for _ in 0..5000 {
                let (data, label) = grouped_example(&mut next, false);
                bdt.graph.nodes[2].classifier.train(&data, &[label[1], label[0]]);
            }
            for _ in 0..4000 {
                let (data, label) = grouped_example(&mut next, false);
                bdt.train(&data, &label);
            }
            bdt.refresh();
            let mut right = 0;
            for _ in 0..1000 {
                let (data, label) = grouped_example(&mut next, false);
                let mut p = [0.0; 2];
                bdt.graph.nodes[2].classifier.predict(&data, &mut p);
                right += ((p[1] > p[0]) == (label[1] > 0.5)) as u32;
            }
            assert_eq!(right > 800, reset_leaves.is_some(), "{reset_leaves:?}: {right} of 1000 right");
            assert_eq!(bdt.node_count(), 3);
        }
    }

    #[test]
    fn seams_are_measured_where_games_cross_between_leaves() {
        let params = DelegationParams { split_limit: 0, revise: Some(f64::INFINITY), ..params(false) };
        let mut bdt = Bdt::new(&[2, 2], params);
        bdt.graph.split_leaf(0);
        let (a, b) = ([0u8, 0], [1u8, 1]);
        // Route a to leaf 1 and b to leaf 2, which predict differently.
        let mut scratch = [0.0; 2];
        for _ in 0..50 {
            for (data, errors) in [(a, [0.0, 1.0]), (b, [1.0, 0.0])] {
                bdt.graph.begin_step(&data, &[1.0, 0.0], &mut scratch);
                bdt.graph.train_router(0, &data, &errors);
            }
        }
        bdt.graph.nodes[1].classifier.train(&a, &[1.0, 0.0]);
        bdt.graph.nodes[2].classifier.train(&b, &[0.0, 1.0]);
        bdt.refresh();
        assert_eq!((bdt.leaf(&a), bdt.leaf(&b)), (1, 2));
        bdt.measure_seams(&[&[a, b, b, a][..]]);
        let v = |leaf, data: &[u8; 2]| bdt.graph.white_log_odds(leaf, data);
        let expected = (v(2, &b) - v(1, &b)).abs() + (v(1, &a) - v(2, &a)).abs();
        let revision = bdt.graph.nodes[0].revision.as_ref().unwrap();
        assert_eq!((revision.positions, revision.crossings), (4, 2));
        assert!((revision.seams - expected).abs() < 1e-12 && expected > 0.1, "{} vs {expected}", revision.seams);
    }

    #[test]
    fn routing_from_a_child_stays_in_its_subtree() {
        let tree = trained(false);
        assert!(!tree.children(0).is_empty());
        for data in [[0, 0], [0, 2], [1, 1], [2, 0], [2, 2]] {
            assert_eq!(tree.leaf_from(0, &data), tree.leaf(&data));
            for &child in tree.children(0) {
                let mut subtree = vec![child];
                let mut i = 0;
                while i < subtree.len() {
                    subtree.extend_from_slice(tree.children(subtree[i]));
                    i += 1;
                }
                let leaf = tree.leaf_from(child, &data);
                assert!(subtree.contains(&leaf) && tree.children(leaf).is_empty());
            }
        }
    }

    #[test]
    fn a_tree_that_has_not_split_forgets_like_its_leaf_alone() {
        let mut flat = Cnbp::new(2, &[3, 3], 1.0, true);
        let mut tree = Bdt::new(&[3, 3], DelegationParams { split_limit: 0, ..params(true) });
        for round in 0..3u8 {
            for i in 0..30u8 {
                let (data, label) = if (i + round) % 3 == 0 { ([0, i % 3], [1.0, 0.0]) } else { ([2, i % 3], [0.3, 0.7]) };
                flat.train(&data, &label);
                tree.train(&data, &label);
            }
            flat.forget(1.5);
            tree.forget(1.5);
            flat.refresh();
            tree.refresh();
        }
        assert_eq!(tree.split_count(), 0);
        for data in [[0, 0], [0, 2], [1, 1], [2, 0], [2, 2]] {
            let (mut p, mut q) = ([0.0; 2], [0.0; 2]);
            flat.predict(&data, &mut p);
            tree.predict(&data, &mut q);
            assert_eq!(p, q);
        }
    }

    #[test]
    fn forgetting_reaches_error_routers_and_shadows() {
        let p = DelegationParams {
            route: RouteTarget::Cluster,
            route_switch: Some(f64::INFINITY),
            revise: Some(f64::INFINITY),
            ..params(false)
        };
        let tree = trained_with(p);
        let mut forgotten = tree.clone();
        forgotten.forget(4.0);
        let (before, after) = (&tree.graph.nodes[0], &forgotten.graph.nodes[0]);
        assert!(before.error_router.is_some() && before.revision.is_some());
        assert_ne!(format!("{:?}", before.error_router), format!("{:?}", after.error_router));
        assert_ne!(format!("{:?}", before.revision), format!("{:?}", after.revision));
    }

    #[test]
    fn shadows_that_only_compare_do_not_change_training() {
        let logging = DelegationParams { revise: Some(f64::INFINITY), reset_leaves: Some(f64::INFINITY), ..params(false) };
        assert_eq!(trained_with(logging).to_text(), trained(false).to_text());
    }

    /// Inputs in two groups (feature 0 and a noisy copy of it) and features
    /// 2 and 3; the label follows feature 3 with 10% noise, except in group 1
    /// when feature 2 is 1, where it is the opposite. Cluster routing splits
    /// by group, and the group 1 leaf, half of whose labels are flipped,
    /// learns little; the group 0 leaf predicts the rest of group 1 better.
    fn partly_flipped_example(next: &mut impl FnMut(u64) -> u64) -> ([u8; 4], [f64; 2]) {
        let group = next(2) as u8;
        let mut data = [group; 4];
        if next(10) == 0 {
            data[1] = 1 - group;
        }
        data[2] = next(2) as u8;
        data[3] = next(2) as u8;
        let mut class = data[3] ^ (group & data[2]);
        if next(10) == 0 {
            class = 1 - class;
        }
        (data, if class == 0 { [1.0, 0.0] } else { [0.0, 1.0] })
    }

    #[test]
    fn leaf_share_keeps_the_leaves_closer() {
        // Mean |difference| of the two leaves' log-odds on the same examples.
        let disagreement = |share: f64| {
            let mut x = 11u64;
            let mut next = |m: u64| {
                x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                (x >> 33) % m
            };
            let params =
                DelegationParams { split_threshold: 4000, split_limit: 0, route: RouteTarget::Cluster, leaf_share: share, ..params(false) };
            let mut bdt = Bdt::new(&[2; 4], params);
            bdt.graph.split_leaf(0);
            for _ in 0..20_000 {
                let (data, label) = partly_flipped_example(&mut next);
                bdt.train(&data, &label);
            }
            bdt.refresh();
            let mut sum = 0.0;
            for _ in 0..2000 {
                let (data, _) = partly_flipped_example(&mut next);
                let (mut a, mut b) = ([0.0; 2], [0.0; 2]);
                bdt.node_predict_log(1, &data, &mut a);
                bdt.node_predict_log(2, &data, &mut b);
                sum += ((a[0] - a[1]) - (b[0] - b[1])).abs();
            }
            sum / 2000.0
        };
        let (alone, shared) = (disagreement(0.0), disagreement(0.5));
        assert!(shared < 0.7 * alone, "{shared} vs {alone}");
    }

    /// As `leaf_share_keeps_the_leaves_closer`, with LR leaves.
    #[test]
    fn leaf_share_keeps_lr_leaves_closer() {
        let disagreement = |share: f64| {
            let mut x = 11u64;
            let mut next = |m: u64| {
                x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                (x >> 33) % m
            };
            let params = DelegationParams {
                split_threshold: 4000,
                split_limit: 0,
                route: RouteTarget::Cluster,
                leaf: NodeKind::Lr,
                leaf_share: share,
                ..params(false)
            };
            let mut bdt = Bdt::new(&[2; 4], params);
            bdt.graph.split_leaf(0);
            for _ in 0..20_000 {
                let (data, label) = partly_flipped_example(&mut next);
                bdt.train(&data, &label);
            }
            bdt.refresh();
            let mut sum = 0.0;
            for _ in 0..2000 {
                let (data, _) = partly_flipped_example(&mut next);
                let (mut a, mut b) = ([0.0; 2], [0.0; 2]);
                bdt.node_predict_log(1, &data, &mut a);
                bdt.node_predict_log(2, &data, &mut b);
                sum += ((a[0] - a[1]) - (b[0] - b[1])).abs();
            }
            sum / 2000.0
        };
        let (alone, shared) = (disagreement(0.0), disagreement(0.5));
        assert!(shared < 0.7 * alone, "{shared} vs {alone}");
    }

    #[test]
    fn route_switch_moves_the_examples_a_cluster_mixes() {
        for (switch, error_router) in [(None, NodeKind::Cnbp), (Some(3.0), NodeKind::Cnbp), (Some(3.0), NodeKind::Lr)] {
            let mut x = 7u64;
            let mut next = |m: u64| {
                x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                (x >> 33) % m
            };
            // No splits of its own (limit 0): the test splits the root.
            let params = DelegationParams {
                split_threshold: 4000,
                split_limit: 0,
                route: RouteTarget::Cluster,
                route_switch: switch,
                error_router,
                ..params(false)
            };
            let mut bdt = Bdt::new(&[2; 4], params);
            bdt.graph.split_leaf(0);
            for _ in 0..20_000 {
                let (data, label) = partly_flipped_example(&mut next);
                bdt.train(&data, &label);
            }
            bdt.refresh();
            assert_eq!(bdt.graph.nodes[0].error_routed, switch.is_some());
            // A switched delegator's router is its error router.
            assert_eq!(bdt.graph.nodes[0].classifier.kind(), if switch.is_some() { error_router } else { NodeKind::Cnbp });
            let (mut right, mut group_1) = (0, 0);
            while group_1 < 2000 {
                let (data, label) = partly_flipped_example(&mut next);
                if data[0] == 1 {
                    let mut p = [0.0; 2];
                    bdt.predict(&data, &mut p);
                    group_1 += 1;
                    right += ((p[1] > p[0]) == (label[1] > 0.5)) as u32;
                }
            }
            // A leaf of its own for the flipped examples predicts all of group 1.
            let accuracy = right as f64 / group_1 as f64;
            assert_eq!(accuracy > 0.8, switch.is_some(), "switch {switch:?}, {error_router:?}: accuracy {accuracy}");
        }
    }

    #[test]
    fn logistic_regression_nodes() {
        for (leaf, router) in [(NodeKind::Lr, NodeKind::Lr), (NodeKind::Cnbp, NodeKind::Lr), (NodeKind::Lr, NodeKind::Cnbp)] {
            let bdt = trained_with(DelegationParams { leaf, router, lr_rate: 0.1, ..params(false) });
            assert!(bdt.split_count() > 0, "{leaf:?} leaves, {router:?} routers");
            let back = Bdt::from_text(&bdt.to_text(), false).unwrap();
            assert_eq!(bdt.to_text(), back.to_text());
            assert_eq!((back.params().leaf, back.params().router, back.params().lr_rate), (leaf, router, 0.1));
            let (mut a, mut b) = ([0.0; 2], [0.0; 2]);
            bdt.predict(&[2, 0], &mut a);
            back.predict(&[2, 0], &mut b);
            assert_eq!(a, b);
            assert!((a[0] + a[1] - 1.0).abs() < 1e-12);
        }
    }

    /// TAN routers (and leaves) split, and save and load as text predicting
    /// the same, with or without cached probabilities.
    #[test]
    fn tan_nodes() {
        for use_probs in [false, true] {
            for (leaf, router) in [(NodeKind::Lr, NodeKind::Tan), (NodeKind::Tan, NodeKind::Tan), (NodeKind::Cnbp, NodeKind::Tan)] {
                let bdt = trained_with(DelegationParams { leaf, router, lr_rate: 0.1, tan_m: 3.0, ..params(use_probs) });
                assert!(bdt.split_count() > 0, "{leaf:?} leaves, {router:?} routers");
                let back = Bdt::from_text(&bdt.to_text(), use_probs).unwrap();
                assert_eq!(bdt.to_text(), back.to_text());
                assert_eq!((back.params().leaf, back.params().router, back.params().tan_m), (leaf, router, 3.0));
                for data in [[2, 0], [0, 1], [1, 2]] {
                    let (mut a, mut b) = ([0.0; 2], [0.0; 2]);
                    bdt.predict(&data, &mut a);
                    back.predict(&data, &mut b);
                    assert!((a[0] - b[0]).abs() < 1e-12 && (a[1] - b[1]).abs() < 1e-12, "{a:?} {b:?}");
                    assert!((a[0] + a[1] - 1.0).abs() < 1e-12);
                    assert_eq!(bdt.leaf(&data), back.leaf(&data));
                }
            }
        }
    }

    /// As `cluster_routing_separates_groups_without_labels`, with a TAN
    /// router: generative like CNBP, it clusters rather than collapsing. With
    /// a small m (5 here) one child's TAN can learn both groups through the
    /// features' parents and takes some of the other's (406/75 against
    /// 0/519); an m nearer the counts (1000) keeps CNBP's clean split.
    #[test]
    fn tan_cluster_routing_separates_groups_without_labels() {
        let mut x = 99u64;
        let mut next = |m: u64| {
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (x >> 33) % m
        };
        let mut example = || {
            let group = next(2) as u8;
            let data: [u8; 6] = std::array::from_fn(|_| 2 * group + next(2) as u8);
            let label = if next(2) == 0 { [1.0, 0.0] } else { [0.0, 1.0] };
            (group, data, label)
        };
        let params = DelegationParams {
            split_threshold: 500,
            split_limit: 1,
            split_rule: SplitRule::StallHigh,
            route: RouteTarget::Cluster,
            router: NodeKind::Tan,
            leaf: NodeKind::Lr,
            tan_m: 1000.0,
            ..params(true)
        };
        let mut bdt = Bdt::new(&[4; 6], params);
        for i in 0..20_000 {
            let (_, data, label) = example();
            bdt.train(&data, &label);
            // Refresh now and then, as rounds do: the tree is relearned there.
            if i % 1000 == 999 {
                bdt.refresh();
            }
        }
        bdt.refresh();
        assert_eq!(bdt.split_count(), 1);
        let mut leaves = [[0; 2]; 2];
        for _ in 0..1000 {
            let (group, data, _) = example();
            leaves[group as usize][bdt.leaf(&data) - 1] += 1;
        }
        let pure = |[a, b]: [i32; 2]| a.max(b) as f64 / (a + b) as f64;
        assert!(pure(leaves[0]) > 0.95 && pure(leaves[1]) > 0.95, "{leaves:?}");
        assert_ne!(leaves[0][0] > leaves[0][1], leaves[1][0] > leaves[1][1], "{leaves:?}");
    }

    #[test]
    fn soft_error_targets_prefer_the_calibrated_child() {
        // The first class is the outcome 60% of the time. Child 0 gives it
        // 0.9 (overconfident), child 1 gives it 0.6 (calibrated: the lower
        // expected log-loss). Child 0 is closer to the outcome 60% of the
        // time, so the best-child target favours it; the soft target's
        // expected share for it is .6 * .9 / 1.5 + .4 * .1 / .5 = .44.
        let (overconfident, calibrated) = ([0.9, 0.1], [0.6, 0.4]);
        for (error_target, expected) in [(ErrorTarget::Best, 0), (ErrorTarget::Soft, 1)] {
            let params = DelegationParams { split_limit: 0, error: ErrorMeasure::Log, error_target, ..params(false) };
            let mut bdt = Bdt::new(&[3, 3], params);
            bdt.graph.split_leaf(0);
            let (data, mut scratch) = ([1, 2], [0.0; 2]);
            for i in 0..1000 {
                let label = if i % 5 < 3 { [1.0, 0.0] } else { [0.0, 1.0] };
                let errors = [log_loss(&overconfident, &label), log_loss(&calibrated, &label)];
                // A new step, so the router's cached output is renewed.
                bdt.graph.begin_step(&data, &label, &mut scratch);
                bdt.graph.train_router(0, &data, &errors);
            }
            assert_eq!(bdt.graph.route(0, &data), expected, "{error_target:?}");
        }
    }

    #[test]
    fn soft_error_targets_train_cnbp_and_lr_routers() {
        for router in [NodeKind::Cnbp, NodeKind::Lr] {
            let params =
                DelegationParams { router, lr_rate: 0.1, error: ErrorMeasure::Log, error_target: ErrorTarget::Soft, ..params(false) };
            let bdt = trained_with(params);
            assert!(bdt.split_count() > 0, "{router:?} routers");
            let mut p = [0.0; 2];
            bdt.predict(&[2, 0], &mut p);
            assert!((p[0] + p[1] - 1.0).abs() < 1e-12);
        }
    }

    #[test]
    #[should_panic(expected = "soft error targets need")]
    fn soft_error_targets_need_a_log_loss() {
        Bdt::new(&[3, 3], DelegationParams { error_target: ErrorTarget::Soft, ..params(false) });
    }

    /// A seeded generator: `next(m)` is uniform in 0..m.
    fn lcg(seed: u64) -> impl FnMut(u64) -> u64 {
        let mut x = seed;
        move |m| {
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (x >> 33) % m
        }
    }

    /// Every input of `grouped_example`'s six binary features.
    fn all_inputs() -> impl Iterator<Item = [u8; 6]> {
        (0..64u8).map(|i| std::array::from_fn(|f| (i >> f) & 1))
    }

    /// Residual leaves in an error-routed tree of logistic regressions whose
    /// shadows only log, with no splits of its own (the tests split it).
    fn residual_params() -> DelegationParams {
        DelegationParams {
            branch_factor: 3,
            split_threshold: 4000,
            split_limit: 0,
            forget_factor: 1.0,
            leaf: NodeKind::Lr,
            router: NodeKind::Lr,
            lr_rate: 0.1,
            error: ErrorMeasure::Log,
            revise: Some(f64::INFINITY),
            leaf_residual: true,
            ..params(false)
        }
    }

    /// A residual tree trained on `n` of `grouped_example`'s examples, whose
    /// root the test splits after `split` of them.
    fn residual_tree(params: DelegationParams, split: u32, n: u32, next: &mut impl FnMut(u64) -> u64) -> Bdt {
        let mut bdt = Bdt::new(&[2; 6], params);
        for i in 0..n {
            if i == split {
                bdt.graph.split_leaf(0);
            }
            let (data, label) = grouped_example(next, true);
            bdt.train(&data, &label);
        }
        bdt
    }

    fn lr(model: &NodeModel) -> &Logreg {
        let NodeModel::Lr(m) = model else { panic!("not a logistic regression") };
        m
    }

    /// Delegator `idx`'s shadow.
    fn shadow(bdt: &Bdt, idx: usize) -> &Logreg {
        lr(&bdt.graph.nodes[idx].revision.as_ref().expect("a shadow").shadow)
    }

    /// Residual leaf `idx`'s delta.
    fn delta(bdt: &Bdt, idx: usize) -> &Logreg {
        bdt.graph.nodes[idx].residual.as_deref().expect("a residual leaf")
    }

    /// `a` plus `b`, as a residual leaf's classifier holds them: AdaGrad sums
    /// added, or with `inherit` (`residual_inherit_sums`) `b`'s.
    fn sum_as(a: &Logreg, b: &Logreg, inherit: bool) -> Logreg {
        let mut s = a.clone();
        s.set_sum(a, b, !inherit);
        s
    }

    fn sum(a: &Logreg, b: &Logreg) -> Logreg {
        sum_as(a, b, false)
    }

    /// Whether every residual leaf's classifier is exactly its delegator's
    /// shadow plus its delta, and every leaf below a delegator is residual.
    fn sums_hold(bdt: &Bdt) -> bool {
        let inherit = bdt.params().residual_inherit_sums;
        (0..bdt.node_count()).all(|idx| {
            bdt.children(idx).iter().all(|&child| match &bdt.graph.nodes[child].residual {
                Some(d) => {
                    lr(&bdt.graph.nodes[child].classifier).to_text() == sum_as(shadow(bdt, idx), d, inherit).to_text()
                }
                None => !bdt.children(child).is_empty(),
            })
        })
    }

    fn predictions(model: &Logreg) -> Vec<[f64; 2]> {
        all_inputs()
            .map(|data| {
                let mut p = [0.0; 2];
                model.predict(&data, &mut p);
                p
            })
            .collect()
    }

    #[test]
    fn residual_leaves_start_as_their_shadow_and_follow_it() {
        for inherit in [false, true] {
            let mut next = lcg(5);
            let params = DelegationParams { residual_inherit_sums: inherit, ..residual_params() };
            let mut bdt = residual_tree(params, u32::MAX, 2000, &mut next);
            assert!(bdt.graph.nodes[0].residual.is_none(), "the root is a plain leaf");
            let root = lr(&bdt.graph.nodes[0].classifier).clone();
            bdt.graph.split_leaf(0);
            // The shadow is the root as it was; the new leaves' deltas are 0,
            // so they are exactly the shadow, AdaGrad sums too.
            assert_eq!(shadow(&bdt, 0).to_text(), root.to_text());
            for &child in bdt.children(0) {
                assert_eq!(delta(&bdt, child).weight_norms(), (0.0, 0.0));
                assert_eq!(lr(&bdt.graph.nodes[child].classifier).to_text(), root.to_text());
            }
            // The leaves tie, and ties go to the first child, as with copies
            // of the leaf: the first example trains its delta only. The
            // others still predict as the shadow, which learned from the
            // example.
            let (data, label) = grouped_example(&mut next, true);
            bdt.train(&data, &label);
            assert_ne!(shadow(&bdt, 0).to_text(), root.to_text());
            let children = bdt.children(0).to_vec();
            assert!(delta(&bdt, children[0]).weight_norms().0 > 0.0);
            for &child in &children[1..] {
                assert_eq!(delta(&bdt, child).weight_norms(), (0.0, 0.0));
                if !inherit {
                    assert_eq!(lr(&bdt.graph.nodes[child].classifier).to_text(), shadow(&bdt, 0).to_text());
                }
                assert_eq!(predictions(lr(&bdt.graph.nodes[child].classifier)), predictions(shadow(&bdt, 0)));
            }
            assert!(sums_hold(&bdt));
            for _ in 0..3000 {
                let (data, label) = grouped_example(&mut next, true);
                bdt.train(&data, &label);
            }
            assert!(sums_hold(&bdt));
            // Forgetting keeps the sums.
            bdt.forget(3.0);
            assert!(sums_hold(&bdt));
        }
    }

    #[test]
    fn inheriting_sums_a_split_makes_the_copies_a_split_without_residual_leaves_makes() {
        for forget_factor in [1.0, 4.0] {
            let copies = DelegationParams { forget_factor, leaf_residual: false, ..residual_params() };
            let inherit = DelegationParams { leaf_residual: true, residual_inherit_sums: true, ..copies.clone() };
            let mut a = residual_tree(copies, 2000, 2000, &mut lcg(7));
            let mut b = residual_tree(inherit, 2000, 2000, &mut lcg(7));
            a.graph.split_leaf(0);
            b.graph.split_leaf(0);
            // The same shadow, and leaves with the same weights and AdaGrad
            // sums: the residual leaves' deltas are 0 with the shadow's sums.
            assert_eq!(shadow(&a, 0).to_text(), shadow(&b, 0).to_text());
            for (&x, &y) in a.children(0).iter().zip(b.children(0)) {
                assert_eq!(lr(&a.graph.nodes[x].classifier).to_text(), lr(&b.graph.nodes[y].classifier).to_text());
                assert_eq!(delta(&b, y).to_text(), shadow(&b, 0).zeroed().to_text());
                assert_ne!(delta(&b, y).to_text(), shadow(&b, 0).blank().to_text());
            }
            // So the first step of the leaf the example goes to is a copy's
            // step size: the delta's from the shadow's sums.
            let (data, label) = grouped_example(&mut lcg(8), true);
            let before = b.clone();
            b.train(&data, &label);
            let changed: Vec<usize> =
                b.children(0).iter().copied().filter(|&c| delta(&b, c).to_text() != delta(&before, c).to_text()).collect();
            let [leaf] = changed[..] else { panic!("{changed:?}") };
            let mut p = [0.0; 2];
            sum(shadow(&b, 0), delta(&before, leaf)).predict(&data, &mut p);
            let mut expected = shadow(&before, 0).zeroed();
            expected.step(&data, &p, &label, 1.0, 0.0);
            assert_eq!(delta(&b, leaf).to_text(), expected.to_text());
            assert!(sums_hold(&b));
        }
    }

    #[test]
    fn a_residual_step_trains_the_delta_on_the_summed_prediction_and_the_shadow_on_its_own() {
        for (l2, inherit) in [(0.0, false), (0.05, false), (0.0, true), (0.05, true)] {
            let mut next = lcg(9);
            let params = DelegationParams { residual_l2: l2, residual_inherit_sums: inherit, ..residual_params() };
            let mut bdt = residual_tree(params, 1000, 3000, &mut next);
            let mut routed = vec![0; bdt.node_count()];
            for _ in 0..300 {
                let (data, label) = grouped_example(&mut next, true);
                let before = bdt.clone();
                bdt.train(&data, &label);
                // The shadow takes its own step on the example.
                let mut expected = shadow(&before, 0).clone();
                expected.train(&data, &label);
                assert_eq!(shadow(&bdt, 0).to_text(), expected.to_text());
                // So does exactly one delta, that of the leaf the example is
                // routed to: an AdaGrad step on the log-loss of the shadow
                // (trained) plus the delta.
                let children = bdt.children(0).to_vec();
                let changed: Vec<usize> =
                    children.iter().copied().filter(|&c| delta(&bdt, c).to_text() != delta(&before, c).to_text()).collect();
                assert_eq!(changed, [bdt.leaf(&data)], "l2 {l2}");
                let leaf = changed[0];
                let mut p = [0.0; 2];
                sum(shadow(&bdt, 0), delta(&before, leaf)).predict(&data, &mut p);
                let mut expected = delta(&before, leaf).clone();
                expected.step(&data, &p, &label, 1.0, l2);
                assert_eq!(delta(&bdt, leaf).to_text(), expected.to_text(), "l2 {l2}");
                assert!(sums_hold(&bdt));
                routed[leaf] += 1;
            }
            assert!(routed.iter().filter(|&&n| n > 0).count() > 1, "{routed:?}");
        }
    }

    #[test]
    fn residual_l2_shrinks_the_deltas() {
        let norm = |l2: f64| {
            let bdt = residual_tree(DelegationParams { residual_l2: l2, ..residual_params() }, 1000, 6000, &mut lcg(21));
            bdt.children(0).iter().map(|&c| delta(&bdt, c).weight_norms().0).sum::<f64>()
        };
        let (free, shrunk) = (norm(0.0), norm(1.0));
        assert!(shrunk < 0.5 * free, "{shrunk} vs {free}");
    }

    #[test]
    fn a_residual_leaf_that_splits_materialises_its_sum_as_the_shadow() {
        for (forget_factor, inherit) in [(1.0, false), (4.0, false), (1.0, true), (4.0, true)] {
            let mut next = lcg(13);
            let params = DelegationParams { forget_factor, residual_inherit_sums: inherit, ..residual_params() };
            let mut bdt = residual_tree(params, 1000, 3000, &mut next);
            let leaf = *bdt.children(0).iter().max_by(|&&a, &&b| delta(&bdt, a).weight_norms().0.total_cmp(&delta(&bdt, b).weight_norms().0)).unwrap();
            let before = lr(&bdt.graph.nodes[leaf].classifier).clone();
            assert!(delta(&bdt, leaf).weight_norms().0 > 0.0);
            assert_eq!(before.to_text(), sum_as(shadow(&bdt, 0), delta(&bdt, leaf), inherit).to_text());
            bdt.graph.split_leaf(leaf);
            // The leaf's summed weights, with the sums of both parts' AdaGrad
            // sums (inheriting: the delta's), forgotten as a shadow is, are
            // one model now: its shadow.
            assert!(bdt.graph.nodes[leaf].residual.is_none());
            let mut expected = before.clone();
            expected.forget(forget_factor);
            assert_eq!(shadow(&bdt, leaf).to_text(), expected.to_text());
            assert_eq!(predictions(shadow(&bdt, leaf)), predictions(&before));
            for &child in bdt.children(leaf) {
                assert_eq!(delta(&bdt, child).weight_norms(), (0.0, 0.0));
                assert_eq!(predictions(lr(&bdt.graph.nodes[child].classifier)), predictions(&before));
            }
            assert!(sums_hold(&bdt));
            for _ in 0..3000 {
                let (data, label) = grouped_example(&mut next, true);
                bdt.train(&data, &label);
            }
            assert!(sums_hold(&bdt));
        }
    }

    #[test]
    fn a_merged_branch_below_a_delegator_becomes_a_residual_leaf_again() {
        for inherit in [false, true] {
            merged_branch_becomes_a_residual_leaf(inherit);
        }
    }

    fn merged_branch_becomes_a_residual_leaf(inherit: bool) {
        let mut next = lcg(17);
        let params = DelegationParams { residual_inherit_sums: inherit, ..residual_params() };
        let mut bdt = residual_tree(params, 1000, 3000, &mut next);
        let leaf = bdt.children(0)[0];
        bdt.graph.split_leaf(leaf);
        for _ in 0..3000 {
            let (data, label) = grouped_example(&mut next, true);
            bdt.train(&data, &label);
        }
        let (own, above) = (shadow(&bdt, leaf).clone(), shadow(&bdt, 0).clone());
        let revision = bdt.graph.nodes[leaf].revision.take().unwrap();
        bdt.graph.merge(leaf, *revision);
        bdt.refresh();
        assert!(bdt.children(leaf).is_empty());
        assert_eq!(bdt.node_count(), 4);
        // Its delta is its shadow minus the shadow above, so it predicts as
        // its shadow did, up to rounding.
        assert_eq!(delta(&bdt, leaf).to_text(), own.minus(&above).to_text());
        assert!(sums_hold(&bdt));
        for (p, q) in predictions(lr(&bdt.graph.nodes[leaf].classifier)).iter().zip(predictions(&own)) {
            assert!((p[0] - q[0]).abs() < 1e-12, "{p:?} vs {q:?}");
        }
        for _ in 0..1000 {
            let (data, label) = grouped_example(&mut next, true);
            bdt.train(&data, &label);
        }
        assert!(sums_hold(&bdt));
        // The root's shadow becomes a plain leaf.
        let root = shadow(&bdt, 0).clone();
        let revision = bdt.graph.nodes[0].revision.take().unwrap();
        bdt.graph.merge(0, *revision);
        bdt.refresh();
        assert_eq!(bdt.node_count(), 1);
        assert!(bdt.graph.nodes[0].residual.is_none());
        assert_eq!(lr(&bdt.graph.nodes[0].classifier).to_text(), root.to_text());
    }

    #[test]
    fn residual_trees_save_their_summed_leaves_as_plain_trees() {
        let mut next = lcg(3);
        let mut bdt = residual_tree(residual_params(), 1000, 3000, &mut next);
        bdt.graph.split_leaf(bdt.children(0)[1]);
        for _ in 0..3000 {
            let (data, label) = grouped_example(&mut next, true);
            bdt.train(&data, &label);
        }
        bdt.refresh();
        let text = bdt.to_text();
        let back = Bdt::from_text(&text, false).unwrap();
        assert!(!back.params().leaf_residual && back.graph.nodes.iter().all(|n| n.residual.is_none()));
        assert_eq!(back.to_text(), text);
        for data in all_inputs() {
            let (mut a, mut b) = ([0.0; 2], [0.0; 2]);
            bdt.predict(&data, &mut a);
            back.predict(&data, &mut b);
            assert_eq!(a, b);
            assert_eq!(bdt.leaf(&data), back.leaf(&data));
        }
    }

    /// `DelegationParams` as it was before residual leaves, with the derived
    /// `Debug`, which state files record.
    mod before_residual {
        use crate::delegation::{ErrorMeasure, ErrorTarget, NodeKind, RouteTarget, SplitRule};

        #[allow(dead_code)]
        #[derive(Debug)]
        pub struct DelegationParams {
            pub class_num: usize,
            pub branch_factor: usize,
            pub split_threshold: u32,
            pub split_limit: u32,
            pub forget_factor: f64,
            pub nb_alpha: f64,
            pub use_probs: bool,
            pub leaf: NodeKind,
            pub router: NodeKind,
            pub lr_rate: f64,
            pub error: ErrorMeasure,
            pub split_z: f64,
            pub split_rule: SplitRule,
            pub route: RouteTarget,
            pub error_target: ErrorTarget,
            pub leaf_share: f64,
            pub split_trial: Option<f64>,
            pub trial_error: ErrorMeasure,
            pub route_switch: Option<f64>,
            pub error_router: NodeKind,
            pub revise: Option<f64>,
            pub seam_weight: f64,
            pub reset_leaves: Option<f64>,
        }
    }

    #[test]
    fn options_without_residual_leaves_are_recorded_as_before() {
        let variants = [
            params(false),
            DelegationParams {
                leaf: NodeKind::Lr,
                route: RouteTarget::Cluster,
                route_switch: Some(3.0),
                error_router: NodeKind::Lr,
                revise: Some(0.0),
                reset_leaves: Some(f64::INFINITY),
                seam_weight: 0.25,
                split_z: 1.5,
                ..params(true)
            },
        ];
        for p in variants {
            let old = before_residual::DelegationParams {
                class_num: p.class_num,
                branch_factor: p.branch_factor,
                split_threshold: p.split_threshold,
                split_limit: p.split_limit,
                forget_factor: p.forget_factor,
                nb_alpha: p.nb_alpha,
                use_probs: p.use_probs,
                leaf: p.leaf,
                router: p.router,
                lr_rate: p.lr_rate,
                error: p.error,
                split_z: p.split_z,
                split_rule: p.split_rule,
                route: p.route,
                error_target: p.error_target,
                leaf_share: p.leaf_share,
                split_trial: p.split_trial,
                trial_error: p.trial_error,
                route_switch: p.route_switch,
                error_router: p.error_router,
                revise: p.revise,
                seam_weight: p.seam_weight,
                reset_leaves: p.reset_leaves,
            };
            assert_eq!(format!("{p:?}"), format!("{old:?}"));
            // Residual trees' options name the new ones.
            let on = format!("{:?}", DelegationParams { leaf_residual: true, residual_l2: 0.5, ..p.clone() });
            assert_eq!(on, format!("{old:?}").replace(" }", ", leaf_residual: true, residual_l2: 0.5 }"));
            let inherit = DelegationParams { leaf_residual: true, residual_inherit_sums: true, ..p.clone() };
            assert_eq!(
                format!("{inherit:?}"),
                format!("{old:?}").replace(" }", ", leaf_residual: true, residual_l2: 0.0, residual_inherit_sums: true }")
            );
            // Logging fresh gains changes no training, so it is not recorded.
            let logged = DelegationParams { log_fresh_gains: true, ..p.clone() };
            assert_eq!(format!("{logged:?}"), format!("{old:?}"));
            let fresh = DelegationParams { revise_on: ReviseOn::Fresh, log_fresh_gains: true, ..p.clone() };
            assert_eq!(format!("{fresh:?}"), format!("{old:?}").replace(" }", ", revise_on: Fresh }"));
            let rate = DelegationParams { leaf_residual: true, residual_delta_rate: 0.25, ..p };
            assert_eq!(
                format!("{rate:?}"),
                format!("{old:?}").replace(" }", ", leaf_residual: true, residual_l2: 0.0, residual_delta_rate: 0.25 }")
            );
        }
    }

    #[test]
    #[should_panic(expected = "residual leaves need")]
    fn residual_leaves_need_logistic_regression_leaves() {
        Bdt::new(&[3, 3], DelegationParams { leaf: NodeKind::Cnbp, ..residual_params() });
    }

    #[test]
    #[should_panic(expected = "residual leaves need")]
    fn residual_leaves_need_shadows() {
        Bdt::new(&[3, 3], DelegationParams { revise: None, reset_leaves: None, ..residual_params() });
    }

    #[test]
    #[should_panic(expected = "residual_inherit_sums is for residual leaves")]
    fn inheriting_sums_needs_residual_leaves() {
        Bdt::new(&[3, 3], DelegationParams { leaf_residual: false, residual_inherit_sums: true, ..residual_params() });
    }

    #[test]
    #[should_panic(expected = "residual_delta_rate is a positive multiple for residual leaves")]
    fn a_delta_rate_needs_residual_leaves() {
        Bdt::new(&[3, 3], DelegationParams { leaf_residual: false, residual_delta_rate: 0.5, ..residual_params() });
    }

    #[test]
    #[should_panic(expected = "log_fresh_gains and revise_on fresh score shadows")]
    fn fresh_rounds_need_shadows() {
        Bdt::new(&[3, 3], DelegationParams { revise_on: ReviseOn::Fresh, ..params(false) });
    }

    /// A round of `n` of `grouped_example`'s examples from `games` games,
    /// which take turns, each with its game's id.
    fn round_data(next: &mut impl FnMut(u64) -> u64, round: u64, games: u64, n: u64) -> Vec<([u8; 6], [f64; 2], u64)> {
        (0..n)
            .map(|i| {
                let (data, label) = grouped_example(next, true);
                (data, label, round << 32 | i % games)
            })
            .collect()
    }

    /// The nodes from the root to the leaf that predicts `data`, routed by
    /// logistic-regression routers.
    fn path_of(bdt: &Bdt, data: &[u8]) -> Vec<usize> {
        let mut path = vec![0];
        loop {
            let idx = *path.last().unwrap();
            let children = bdt.children(idx);
            if children.is_empty() {
                return path;
            }
            let mut scores = vec![0.0; children.len()];
            bdt.graph.nodes[idx].classifier.predict(data, &mut scores);
            path.push(children[crate::delegation::argmax(&scores)]);
        }
    }

    #[test]
    fn logging_fresh_gains_leaves_training_unchanged() {
        for leaf_residual in [false, true] {
            let params = DelegationParams { leaf_residual, split_threshold: 500, revise: Some(0.0), ..residual_params() };
            let mut plain = Bdt::new(&[2; 6], params.clone());
            let mut logged = Bdt::new(&[2; 6], DelegationParams { log_fresh_gains: true, ..params });
            let mut next = lcg(11);
            let mut gains = 0;
            for round in 0..6 {
                if round == 1 {
                    plain.graph.split_leaf(0);
                    logged.graph.split_leaf(0);
                }
                if let (3, Some(&child)) = (round, plain.children(0).first()) {
                    if plain.children(child).is_empty() {
                        plain.graph.split_leaf(child);
                        logged.graph.split_leaf(child);
                    }
                }
                let data = round_data(&mut next, round, 10, 700);
                logged.observe_round(&data);
                gains += logged.graph.nodes.iter().filter(|n| n.revision.as_ref().is_some_and(|r| r.round_fresh.stats().is_some())).count();
                for (data, label, game) in &data {
                    plain.train(data, label);
                    logged.train_in_game(data, label, *game);
                }
                logged.end_round();
                plain.refresh();
                logged.refresh();
                assert_eq!(plain.to_state().into_bytes(), logged.to_state().into_bytes(), "round {round}");
            }
            assert!(gains >= 5, "{gains}");
        }
    }

    #[test]
    fn fresh_gains_score_the_round_by_the_model_as_it_starts() {
        use std::collections::BTreeMap;
        let params = DelegationParams { trial_error: ErrorMeasure::Log, log_fresh_gains: true, ..residual_params() };
        let mut next = lcg(19);
        let mut bdt = residual_tree(params, 1000, 3000, &mut next);
        let inner = bdt.children(0)[1];
        bdt.graph.split_leaf(inner);
        for (data, label, game) in round_data(&mut next, 0, 7, 2000) {
            bdt.train_in_game(&data, &label, game);
        }
        bdt.refresh();
        let before = bdt.to_state().into_bytes();
        let data = round_data(&mut next, 1, 7, 1500);
        bdt.observe_round(&data);
        assert_eq!(bdt.to_state().into_bytes(), before, "scoring trains nothing");
        // By hand: per delegator on the path, the shadow's log-loss minus the
        // leaf's, summed by game.
        let mut expected: Vec<BTreeMap<u64, (f64, u32)>> = vec![BTreeMap::new(); bdt.node_count()];
        for (data, label, game) in &data {
            let path = path_of(&bdt, data);
            let mut p = [0.0; 2];
            lr(&bdt.graph.nodes[*path.last().unwrap()].classifier).predict(data, &mut p);
            let leaf_loss = log_loss(&p, label);
            for &idx in &path[..path.len() - 1] {
                shadow(&bdt, idx).predict(data, &mut p);
                let entry = expected[idx].entry(*game).or_insert((0.0, 0));
                entry.0 += log_loss(&p, label) - leaf_loss;
                entry.1 += 1;
            }
        }
        for idx in [0, inner] {
            let got = &bdt.graph.nodes[idx].revision.as_ref().unwrap().round_fresh.games;
            assert_eq!(got.keys().collect::<Vec<_>>(), expected[idx].keys().collect::<Vec<_>>());
            for (game, &(sum, n)) in &expected[idx] {
                assert_eq!(got[game].1, n);
                assert!((got[game].0 - sum).abs() < 1e-9, "{} vs {sum}", got[game].0);
            }
        }
        assert!(expected[inner].values().map(|&(_, n)| n).sum::<u32>() > 100);
        // Training on the round scores every example through the root.
        for (data, label, game) in &data {
            bdt.train_in_game(data, label, *game);
        }
        let revision = bdt.graph.nodes[0].revision.as_ref().unwrap();
        assert_eq!(revision.round_stream.stats().unwrap().2, data.len() as u64);
        assert_eq!(revision.round_stream.games.len(), 7);
    }

    #[test]
    fn round_gains_have_a_standard_error_clustered_by_game() {
        let mut gains = crate::delegation::RoundGains::default();
        for (game, gain) in [(1, 0.75), (1, 0.25), (2, -0.5), (2, -0.5), (3, 0.0)] {
            gains.add(game, gain);
        }
        // Mean 0; the games' sums are 1, -1 and 0: variance 3/2 * 2 / 5^2.
        let (mean, se, n, k) = gains.stats().unwrap();
        assert_eq!((mean, n, k), (0.0, 5, 3));
        assert!((se - (3.0f64 / 2.0 * 2.0).sqrt() / 5.0).abs() < 1e-15, "{se}");
    }

    #[test]
    fn on_fresh_rounds_the_windows_score_the_round_before_training() {
        for revise_on in [ReviseOn::Stream, ReviseOn::Fresh] {
            let params = DelegationParams { split_threshold: 100, revise_on, ..residual_params() };
            let mut next = lcg(23);
            let mut bdt = residual_tree(params, 0, 50, &mut next);
            let examples = |bdt: &Bdt| bdt.graph.nodes[0].revision.as_ref().unwrap().window.examples;
            let data = round_data(&mut next, 0, 5, 250);
            bdt.observe_round(&data);
            for (data, label, game) in &data {
                bdt.train_in_game(data, label, *game);
            }
            match revise_on {
                // Two windows of 100 ended in the round's scoring (comparing
                // only), and training added nothing.
                ReviseOn::Fresh => assert_eq!(examples(&bdt), 50),
                // 50 + 250 examples in training: windows ended after 100 and 200.
                ReviseOn::Stream => assert_eq!(examples(&bdt), 100),
            }
        }
    }

    #[test]
    fn a_fresh_window_that_fails_merges_the_branch_before_training() {
        let params = DelegationParams {
            split_threshold: 100,
            revise: Some(0.0),
            revise_on: ReviseOn::Fresh,
            leaf_residual: false,
            ..residual_params()
        };
        let mut next = lcg(29);
        let mut bdt = residual_tree(params, u32::MAX, 2000, &mut next);
        bdt.graph.split_leaf(0);
        // Leaves that learned the opposite: worse than the shadow.
        let mut wrong = Logreg::new(2, &[2; 6], 0.1);
        for _ in 0..2000 {
            let (data, label) = grouped_example(&mut next, true);
            wrong.train(&data, &[label[1], label[0]]);
        }
        for child in bdt.children(0).to_vec() {
            bdt.graph.nodes[child].classifier = NodeModel::Lr(wrong.clone());
        }
        bdt.observe_round(&round_data(&mut next, 0, 5, 99));
        assert_eq!(bdt.node_count(), 4, "the window is not over yet");
        bdt.observe_round(&round_data(&mut next, 1, 5, 2));
        bdt.refresh();
        assert_eq!(bdt.node_count(), 1, "the root merged back into its shadow");
    }

    #[test]
    fn residual_delta_rate_scales_the_deltas_steps() {
        for inherit in [false, true] {
            let tree = |rate: f64| {
                let params = DelegationParams { residual_delta_rate: rate, residual_inherit_sums: inherit, ..residual_params() };
                residual_tree(params, 500, 501, &mut lcg(31))
            };
            let (one, quarter) = (tree(1.0), tree(0.25));
            assert_eq!(shadow(&one, 0).to_text(), shadow(&quarter, 0).to_text());
            for (&a, &b) in one.children(0).iter().zip(quarter.children(0)) {
                assert_eq!(delta(&quarter, b).rate(), 0.25 * delta(&one, a).rate());
                let (na, ma) = delta(&one, a).weight_norms();
                let (nb, mb) = delta(&quarter, b).weight_norms();
                assert_eq!((nb, mb), (0.25 * na, 0.25 * ma));
            }
            assert!(delta(&one, one.children(0)[0]).weight_norms().0 > 0.0);
            assert!(sums_hold(&quarter));
        }
    }

    /// A tree of probit leaves (T-P) over `state_test_rounds`' features,
    /// cluster-routed by CNBPs.
    fn probit_params() -> DelegationParams {
        DelegationParams {
            class_num: 3,
            branch_factor: 3,
            split_threshold: 40,
            split_limit: 6,
            forget_factor: 1.0,
            leaf: NodeKind::Probit,
            route: RouteTarget::Cluster,
            ..params(true)
        }
    }

    /// `state_test_rounds` with three results: every fifth game is drawn.
    fn probit_rounds() -> Vec<Vec<(Vec<[u8; 4]>, [f64; 3])>> {
        let draw = |g: usize, [w, b]: [f64; 2]| if g % 5 == 4 { [0.0, 0.0, 1.0] } else { [w, b, 0.0] };
        let rounds = crate::delegation::state_test_rounds().into_iter();
        rounds.map(|round| round.into_iter().enumerate().map(|(g, (game, l))| (game, draw(g, l))).collect()).collect()
    }

    fn probit(model: &NodeModel) -> &crate::probit::Probit {
        let NodeModel::Probit(m) = model else { panic!("not a probit") };
        m
    }

    /// A split's children start from the leaf's posterior, widened by the
    /// split noise; the shadow, which stands for the leaf, is not widened.
    #[test]
    fn probit_children_copy_the_leaf_widened_by_the_split_noise() {
        for noise in [0.0, 0.1] {
            let params = DelegationParams { split_limit: 0, revise: Some(f64::INFINITY), split_noise: noise, ..probit_params() };
            let mut bdt = Bdt::new(&[3; 4], params);
            for (game, label) in probit_rounds().concat() {
                game.iter().for_each(|pos| bdt.train(pos, &label));
            }
            assert_eq!(bdt.split_count(), 0);
            let leaf = probit(&bdt.graph.nodes[0].classifier).clone();
            bdt.graph.split_leaf(0);
            let mut widened = leaf.clone();
            widened.widen(noise * noise);
            assert_eq!(widened.to_text() == leaf.to_text(), noise == 0.0);
            for &child in bdt.children(0) {
                assert_eq!(probit(&bdt.graph.nodes[child].classifier).to_text(), widened.to_text());
            }
            let shadow = &bdt.graph.nodes[0].revision.as_ref().unwrap().shadow;
            assert_eq!(probit(shadow).to_text(), leaf.to_text());
        }
    }

    /// Probit trees save their leaves in the tree's text format, load as
    /// probit trees, and record their options (the probit's and the split
    /// noise) in state files only when they have them.
    #[test]
    fn probit_leaves_save_load_and_record_their_options() {
        let params = DelegationParams { split_noise: 0.05, ..probit_params() };
        let mut bdt = Bdt::new(&[3; 4], params.clone());
        for (game, label) in probit_rounds().concat() {
            game.iter().for_each(|pos| bdt.train(pos, &label));
        }
        bdt.refresh();
        assert!(bdt.split_count() > 0);
        let back = Bdt::from_text(&bdt.to_text(), true).unwrap();
        assert_eq!(back.to_text(), bdt.to_text());
        assert_eq!((back.params().leaf, back.params().router), (NodeKind::Probit, NodeKind::Cnbp));
        assert_eq!(back.params().probit, ProbitParams::default());
        for pos in [[0, 0, 1, 2], [2, 1, 0, 0], [1, 1, 1, 1]] {
            let (mut a, mut b) = ([0.0; 3], [0.0; 3]);
            bdt.predict_log(&pos, &mut a);
            back.predict_log(&pos, &mut b);
            assert_eq!(a, b);
            bdt.predict(&pos, &mut a);
            assert!((a.iter().sum::<f64>() - 1.0).abs() < 1e-12);
        }
        // Options: as a CNBP-leaf tree's, plus the probit's and the noise.
        let cnbp = format!("{:?}", DelegationParams { leaf: NodeKind::Cnbp, split_noise: 0.0, ..params.clone() });
        let plain = DelegationParams { split_noise: 0.0, ..params.clone() };
        let probit_options = format!(", probit: {:?} }}", params.probit);
        let expected = cnbp.replace("leaf: Cnbp", "leaf: Probit").replace(" }", &probit_options);
        assert_eq!(format!("{plain:?}"), expected);
        assert_eq!(format!("{params:?}"), expected.replace(" } }", " }, split_noise: 0.05 }"));
    }

    /// A probit tree read back from its state trains on exactly as the
    /// original, with shadows (fresh or stream tests, reset leaves), trials
    /// and split noise.
    #[test]
    fn probit_state_round_trip_trains_on_identically() {
        let base = probit_params();
        let variants = [
            base.clone(),
            DelegationParams { split_noise: 0.1, ..base.clone() },
            DelegationParams { split_trial: Some(0.0), split_noise: 0.1, ..base.clone() },
            DelegationParams { revise: Some(0.0), reset_leaves: Some(0.0), split_noise: 0.1, ..base.clone() },
            DelegationParams {
                revise: Some(2.0),
                revise_on: ReviseOn::Fresh,
                reset_leaves: Some(f64::INFINITY),
                ..base.clone()
            },
        ];
        for params in variants {
            let mut bdt = Bdt::new(&[3; 4], params.clone());
            let mut split = false;
            for round in probit_rounds() {
                let state = bdt.to_state().into_bytes();
                let mut back = Bdt::from_state(&state, &[3; 4], params.clone()).unwrap();
                assert_eq!(back.to_state().into_bytes(), state);
                assert_eq!(back.to_text(), bdt.to_text());
                for tree in [&mut bdt, &mut back] {
                    let data: Vec<([u8; 4], [f64; 3], u64)> = (round.iter().enumerate())
                        .flat_map(|(g, (game, label))| game.iter().map(move |&pos| (pos, *label, g as u64)))
                        .collect();
                    tree.observe_round(&data);
                    data.iter().for_each(|(pos, label, game)| tree.train_in_game(pos, label, *game));
                    tree.end_round();
                    tree.refresh();
                }
                assert!(back.to_state().into_bytes() == bdt.to_state().into_bytes(), "{params:?}");
                split |= bdt.split_count() > 0;
            }
            assert!(split, "{params:?}");
            let other = DelegationParams { split_noise: 0.2, ..params.clone() };
            assert!(Bdt::from_state(&bdt.to_state().into_bytes(), &[3; 4], other).is_err());
            let other = DelegationParams { probit: ProbitParams { tau: 0.0, ..params.probit }, ..params };
            assert!(Bdt::from_state(&bdt.to_state().into_bytes(), &[3; 4], other).is_err());
        }
    }

    /// With a shared draw margin, every probit of the tree (leaves, shadows)
    /// has the margin of the leaf that trained last, where the leaves' own
    /// margins differ without it; before the first split the tree is the
    /// plain one. The options record it, and the state resumes.
    #[test]
    fn probit_shared_eps_ties_every_probit_of_the_tree() {
        let own = DelegationParams { revise: Some(f64::INFINITY), ..probit_params() };
        let shared = DelegationParams { probit_shared_eps: true, ..own.clone() };
        let (mut a, mut b) = (Bdt::new(&[3; 4], own.clone()), Bdt::new(&[3; 4], shared.clone()));
        let mut compared = 0;
        for (i, (pos, label)) in probit_rounds().concat().iter().flat_map(|(g, l)| g.iter().map(move |p| (p, l))).enumerate() {
            a.train(pos, label);
            b.train(pos, label);
            if b.split_count() == 0 && i % 25 == 0 {
                assert_eq!(a.to_text(), b.to_text());
                compared += 1;
            }
        }
        assert!(compared > 0 && b.split_count() > 0 && a.split_count() > 0);
        let margins = |bdt: &Bdt| -> Vec<f64> {
            let nodes = &bdt.graph.nodes;
            let shadows = nodes.iter().filter_map(|n| n.revision.as_ref().map(|r| &r.shadow));
            let leaves = nodes.iter().filter(|n| matches!(n.kind, Kind::Leaf)).map(|n| &n.classifier);
            leaves.chain(shadows).map(|m| probit(m).eps()).collect()
        };
        let (own_eps, shared_eps) = (margins(&a), margins(&b));
        assert!(shared_eps.len() > 2 && shared_eps.windows(2).all(|w| w[0] == w[1]), "{shared_eps:?}");
        assert!(own_eps.windows(2).any(|w| w[0] != w[1]), "{own_eps:?}");
        assert_eq!(format!("{shared:?}"), format!("{own:?}").replace(" } }", " }, probit_shared_eps: true }"));
        let state = b.to_state().into_bytes();
        let back = Bdt::from_state(&state, &[3; 4], shared).unwrap();
        assert_eq!(back.to_state().into_bytes(), state);
        assert!(Bdt::from_state(&state, &[3; 4], own).is_err());
    }

    // grow1: the growth rule's unit tests (models/from-scratch/theory-2/growth-spec.md 1.6 a-j). (a), identity with
    // the rule off, is every test above unchanged, the options test below and the binaries' identity runs.

    use crate::delegation::{Blocks, GrowParams, GrowStats, LAST_VERDICT};

    /// The growth rule over 2-class logistic-regression leaves under CNBP error routers (as T-L-Ne), branch factor
    /// 3: windows of `w` examples, a credit per `credit`.
    fn grow_params(w: u32, credit: u64) -> DelegationParams {
        DelegationParams {
            branch_factor: 3,
            split_threshold: w,
            split_limit: 1_000_000,
            forget_factor: 1.0,
            leaf: NodeKind::Lr,
            error: ErrorMeasure::Log,
            trial_error: ErrorMeasure::Log,
            split_rule: SplitRule::Grow,
            grow: GrowParams { credit, ..GrowParams::default() },
            ..params(true)
        }
    }

    /// `n` examples of 4 features of 3 values whose labels are fair coins: nothing to learn, so leaves stagnate.
    fn coin_stream(seed: u64, n: usize) -> Vec<([u8; 4], [f64; 2])> {
        let mut next = lcg(seed);
        (0..n)
            .map(|_| {
                let x: [u8; 4] = std::array::from_fn(|_| next(3) as u8);
                (x, if next(2) == 0 { [1.0, 0.0] } else { [0.0, 1.0] })
            })
            .collect()
    }

    /// `n` examples labelled by the XOR of two binary features (9 times in 10), with two noise features of 3
    /// values: a linear leaf cannot learn it, linear leaves under a router on either feature can.
    fn xor_stream(seed: u64, n: usize) -> Vec<([u8; 4], [f64; 2])> {
        let mut next = lcg(seed);
        (0..n)
            .map(|_| {
                let x = [next(2) as u8, next(2) as u8, next(3) as u8, next(3) as u8];
                let white = (x[0] == x[1]) == (next(10) < 9);
                (x, if white { [1.0, 0.0] } else { [0.0, 1.0] })
            })
            .collect()
    }

    const XOR_CATS: [u8; 4] = [2, 2, 3, 3];

    fn model_state(m: &NodeModel) -> Vec<u8> {
        let mut w = Writer::new("model");
        m.write_state(&mut w);
        w.into_bytes()
    }

    /// Statistics of a leaf that is stagnant and out of back-off, with this excess over the model.
    fn eligible(excess: f64) -> GrowStats {
        GrowStats { prev_exists: true, prev_mean: 0.5 + excess, prev_model_mean: 0.5, stall_streak: 2, closes: 3, ..GrowStats::default() }
    }

    /// A tree split once at the root, the split kept (no trial), its three leaves 1-3.
    fn three_leaves(params: DelegationParams) -> Bdt {
        let mut bdt = Bdt::new(&[3; 4], params);
        bdt.graph.split_leaf(0);
        bdt.graph.nodes[0].trial = None;
        bdt.graph.grow.trials_in_flight = 0;
        bdt
    }

    /// (b) A window's mean and standard error come from its 20 blocks' means; a partial block is dropped.
    #[test]
    fn grow_window_statistics_come_from_the_blocks() {
        let means: Vec<f64> = (0..20).map(|i| 0.5 + 0.01 * (i % 7) as f64 - 0.003 * i as f64).collect();
        let fill = |stats: &mut GrowStats, means: &[f64]| {
            for &m in means {
                for d in [-0.003, -0.001, 0.001, 0.003] {
                    stats.blocks.add(m + d, 4);
                }
            }
        };
        let expected = |means: &[f64]| {
            let k = means.len() as f64;
            let mean = means.iter().sum::<f64>() / k;
            let var = means.iter().map(|m| m * m).sum::<f64>() / k - mean * mean;
            (mean, (var / (k - 1.0)).sqrt())
        };
        let mut full = GrowStats::default();
        fill(&mut full, &means);
        let (mean, se) = expected(&means);
        let (w_mean, w_se, change) = full.close(2.0, 0.0);
        assert!((w_mean - mean).abs() < 1e-12 && (w_se - se).abs() < 1e-12 && change.is_none(), "{w_mean} {w_se}");
        assert_eq!((full.closes, full.blocks), (1, Blocks::default()));
        // 19.5 blocks: the half block, however large its values, is dropped.
        let mut part = GrowStats::default();
        fill(&mut part, &means[..19]);
        part.blocks.add(100.0, 4);
        part.blocks.add(100.0, 4);
        assert_eq!(part.blocks.count, 19);
        let (mean, se) = expected(&means[..19]);
        let (w_mean, w_se, _) = part.close(2.0, 0.0);
        assert!((w_mean - mean).abs() < 1e-12 && (w_se - se).abs() < 1e-12, "{w_mean} {w_se}");
    }

    /// Blocks whose 20 means have mean `mean` and standard error `se`.
    fn window(mean: f64, se: f64) -> Blocks {
        let k = 20.0;
        Blocks { count: 20, sum: k * mean, sum_sq: k * (mean * mean + se * se * (k - 1.0)), ..Blocks::default() }
    }

    /// (c) Stagnation: closes without an improvement beyond 2 standard errors of the difference count up; a rise
    /// counts too; a fall of 3 standard errors resets the count.
    #[test]
    fn grow_stagnation_counts_windows_without_improvement() {
        let mut s = GrowStats::default();
        let mut streaks = Vec::new();
        for (i, mean) in [1.0, 0.8, 0.6, 0.59, 0.585, 0.58].into_iter().enumerate() {
            s.blocks = window(mean, 0.01);
            s.close(2.0, 0.5);
            streaks.push(s.stall_streak);
            assert_eq!(s.eligible(2), i >= 4, "stagnant from the fifth close");
        }
        assert_eq!(streaks, [0, 0, 0, 1, 2, 3]);
        s.blocks = window(0.60, 0.01);
        s.close(2.0, 0.5);
        assert_eq!(s.stall_streak, 4);
        s.blocks = window(0.57, 0.01);
        s.close(2.0, 0.5);
        assert_eq!(s.stall_streak, 0);
        assert!(!s.eligible(2));
    }

    /// (d) Credits: one per `credit` examples through the root, at most the cap; a spend and a refund; no spend
    /// without a credit.
    #[test]
    fn grow_credits_accrue_up_to_the_cap() {
        let mut bdt = Bdt::new(&[3; 4], grow_params(10_000, 1000));
        for (x, y) in coin_stream(3, 5000) {
            bdt.train(&x, &y);
        }
        // Windows of 10,000: nothing closed, nothing eligible.
        assert_eq!((bdt.graph.grow.credits, bdt.graph.grow.next_credit_at), (2, 6000));
        assert_eq!(bdt.graph.nodes[0].grow.closes, 0);
        // A spend: the root, made eligible, splits on trial at a scheduling point.
        bdt.graph.nodes[0].grow = eligible(0.0);
        bdt.graph.params.grow.trial_z = f64::INFINITY;
        assert_eq!(bdt.graph.grow_schedule(0), Some(0));
        assert_eq!((bdt.graph.grow.credits, bdt.graph.grow.credits_spent, bdt.graph.grow.trials_in_flight), (1, 1, 1));
        // No more credits come; the trial is undone and refunds its credit.
        bdt.graph.grow.next_credit_at = u64::MAX;
        for (x, y) in coin_stream(4, 10_001) {
            bdt.train(&x, &y);
        }
        assert!(matches!(bdt.graph.nodes[0].kind, Kind::Leaf));
        assert_eq!((bdt.graph.grow.credits, bdt.graph.grow.credits_refunded, bdt.graph.grow.trials_in_flight), (2, 1, 0));
        // A refund at the cap is lost.
        bdt.graph.nodes[0].grow = eligible(0.0);
        assert_eq!(bdt.graph.grow_schedule(0), Some(0));
        bdt.graph.grow.credits = 2;
        for (x, y) in coin_stream(5, 10_001) {
            bdt.train(&x, &y);
        }
        assert_eq!((bdt.graph.grow.credits, bdt.graph.grow.credits_refunded), (2, 2));
        // Without a credit the scheduler splits nothing.
        bdt.graph.grow.credits = 0;
        bdt.graph.nodes[0].grow = eligible(0.0);
        assert_eq!(bdt.graph.grow_schedule(0), None);
        assert!(matches!(bdt.graph.nodes[0].kind, Kind::Leaf) && bdt.graph.split_number == 0);
    }

    /// (e) The scheduler picks the candidate with the largest excess, then the most examples, then the lowest index.
    #[test]
    fn grow_schedule_picks_the_largest_excess_then_more_examples_then_the_lower_index() {
        for (cases, want) in [([(0.02, 10), (0.05, 20), (0.05, 30)], 3), ([(0.02, 10), (0.05, 30), (0.05, 30)], 2)] {
            let mut bdt = three_leaves(grow_params(100, 1000));
            bdt.graph.grow.credits = 1;
            for (leaf, (excess, n)) in (1..=3).zip(cases) {
                bdt.graph.nodes[leaf].grow = eligible(excess);
                bdt.graph.nodes[leaf].num_trained_total = n;
            }
            assert_eq!(bdt.graph.grow_schedule(1), Some(want));
            assert!(bdt.graph.nodes[want].trial.is_some());
        }
    }

    /// Trains a tree on the XOR stream until its first growth trial ends: the verdict's gain, block standard
    /// error, MDL bar and outcome.
    fn first_verdict(trial_z: f64, mdl: f64) -> (f64, f64, f64, bool) {
        LAST_VERDICT.with(|v| v.set(None));
        let mut p = grow_params(2000, 1000);
        (p.grow.trial_z, p.grow.mdl) = (trial_z, mdl);
        let mut bdt = Bdt::new(&XOR_CATS, p);
        for (x, y) in xor_stream(5, 30_000) {
            bdt.train(&x, &y);
            if let Some(v) = LAST_VERDICT.with(|v| v.get()) {
                return v;
            }
        }
        panic!("no growth trial ended");
    }

    /// (f) A split that lowers the error (a router on one feature of an XOR the leaf alone cannot learn) is kept
    /// iff its gain clears both bars: each bar tested on both sides, the other off.
    #[test]
    fn grow_trials_are_kept_only_above_both_bars() {
        let (gain, se, _, keep) = first_verdict(0.0, 0.0);
        // A real gain: about 0.013 nats per example at z 3.2 over the trial's 2,000 (the router learns as it goes).
        assert!(keep && gain > 0.005 && gain > 2.0 * se && se > 0.0, "gain {gain} se {se}");
        let z = gain / se;
        for (trial_z, kept) in [(0.9 * z, true), (1.1 * z, false)] {
            let v = first_verdict(trial_z, 0.0);
            assert_eq!((v.0, v.1, v.3), (gain, se, kept), "trial_z {trial_z}");
        }
        let bar1 = first_verdict(0.0, 1.0).2;
        assert!(bar1 > 0.0, "bar {bar1}");
        for (mdl, kept) in [(0.9 * gain / bar1, true), (1.1 * gain / bar1, false)] {
            let v = first_verdict(0.0, mdl);
            assert_eq!(v.0, gain);
            assert!((v.2 - mdl * bar1).abs() <= 1e-12 * v.2, "{} {}", v.2, mdl * bar1);
            assert_eq!(v.3, kept, "mdl {mdl}");
        }
    }

    /// (g) An undone trial: the node is a leaf again with the shadow's parameters, the made nodes are dead, the
    /// split number is back, the credit is refunded, and the leaf backs off 1, 2 and 4 windows over three failures,
    /// no candidate before its back-off ends.
    #[test]
    fn grow_undone_trials_restore_the_leaf_refund_and_back_off() {
        let w = 200;
        let mut p = grow_params(w, 100);
        p.grow.trial_z = f64::INFINITY;
        let mut bdt = Bdt::new(&[3; 4], p);
        let (mut starts, mut undos) = (Vec::new(), Vec::new());
        for (x, y) in coin_stream(7, 400_000) {
            let ending = (bdt.graph.nodes[0].trial.as_ref())
                .filter(|t| t.examples >= w)
                .map(|t| (t.shadow.clone(), t.made.clone(), bdt.graph.grow.credits_refunded));
            let on_trial = bdt.graph.nodes[0].trial.is_some();
            bdt.train(&x, &y);
            let g = &bdt.graph;
            if !on_trial && g.nodes[0].trial.is_some() {
                starts.push(g.nodes[0].grow.closes);
            }
            if let Some((mut shadow, made, refunded)) = ending {
                assert!(matches!(g.nodes[0].kind, Kind::Leaf) && g.nodes[0].trial.is_none());
                shadow.train(&x, &y);
                assert_eq!(model_state(&g.nodes[0].classifier), model_state(&shadow));
                assert_eq!(g.dead, made);
                assert_eq!((g.split_number, g.grow.credits_refunded, g.grow.trials_in_flight), (0, refunded + 1, 0));
                let s = g.nodes[0].grow;
                assert_eq!(s.eligible_after_close, s.closes + s.backoff_windows as u64);
                assert_eq!((s.stall_streak, s.prev_exists), (0, false));
                undos.push((s.backoff_windows, s.eligible_after_close));
            }
            if starts.len() == 4 {
                break;
            }
        }
        assert_eq!(undos.iter().map(|u| u.0).collect::<Vec<_>>(), [1, 2, 4]);
        for (i, &(_, eligible_from)) in undos.iter().enumerate() {
            assert!(starts[i + 1] >= eligible_from, "start at close {} before {eligible_from}", starts[i + 1]);
        }
        // With back-off 4 the stagnation's 3 closes are not enough.
        assert!(starts[3] >= undos[2].1 && undos[2].1 > starts[2] + 3, "{starts:?} {undos:?}");
    }

    /// grow2 (theory-2 6.41 note): with --grow-mdl-n lifetime, a retry after an undone trial is judged at the node's
    /// lifetime examples, which the undo did not reset; with trial (grow1's), at the examples since the undo plus the
    /// trial's. The first verdict is at the same n either way, training is the same, and the MDL bar is at the n used.
    #[test]
    fn grow_lifetime_n_judges_a_retry() {
        use crate::delegation::{LAST_VERDICT_N, MdlN, mdl_bar};
        let w = 200;
        let mut runs = Vec::new();
        for mdl_n in [MdlN::Trial, MdlN::Lifetime] {
            let mut p = grow_params(w, 100);
            (p.grow.trial_z, p.grow.mdl_n) = (f64::INFINITY, mdl_n);
            let mut bdt = Bdt::new(&[3; 4], p);
            // Per verdict: its n, the trial's (examples before the split plus the trial's), the root's life, the
            // example's index, and the bar against the bar at n.
            let mut verdicts = Vec::new();
            for (i, (x, y)) in coin_stream(7, 400_000).into_iter().enumerate() {
                let g = &bdt.graph;
                let ending = (g.nodes[0].trial.as_ref())
                    .filter(|t| t.examples >= w)
                    .map(|t| (t.trained_before as u64 + t.examples as u64, g.nodes[0].life, g.split_params_added(0)));
                LAST_VERDICT_N.with(|v| v.set(None));
                bdt.train(&x, &y);
                if let Some(n) = LAST_VERDICT_N.with(|v| v.get()) {
                    let (trial_n, life, k) = ending.expect("a verdict ends a trial");
                    let bar = LAST_VERDICT.with(|v| v.get()).unwrap().2;
                    assert_eq!(bar, mdl_bar(1.0, k, n));
                    verdicts.push((n, trial_n, life, i as u64));
                    if verdicts.len() == 3 {
                        break;
                    }
                }
            }
            assert_eq!(verdicts.len(), 3);
            runs.push(verdicts);
        }
        let (trial, lifetime) = (&runs[0], &runs[1]);
        for (t, l) in trial.iter().zip(lifetime) {
            // The same training: the same trials end at the same examples.
            assert_eq!((t.1, t.2, t.3), (l.1, l.2, l.3));
            assert_eq!((t.0, l.0), (t.1, l.2), "trial mode judges at the trial's n, lifetime mode at the life");
            // Every example passed the root and was scored (log-loss): its life is the stream so far.
            assert_eq!(l.2, l.3);
        }
        assert_eq!(trial[0].0, lifetime[0].0, "the first verdict is at the same n");
        for v in 1..3 {
            assert!(lifetime[v].0 > trial[v].0 + w as u64, "retry {v}: {} vs {}", lifetime[v].0, trial[v].0);
        }
    }

    /// (h) One split per scheduling point; trials in flight never exceed the slots; a node made by a split on trial
    /// is never a candidate.
    #[test]
    fn grow_makes_one_split_per_scheduling_point_within_its_slots() {
        let mut bdt = three_leaves(grow_params(100, 1000));
        bdt.graph.grow.credits = 2;
        for leaf in 1..=3 {
            bdt.graph.nodes[leaf].grow = eligible(0.01 * leaf as f64);
        }
        assert_eq!(bdt.graph.grow_schedule(1), Some(3));
        let g = &bdt.graph;
        assert_eq!((g.split_number, g.grow.credits, g.grow.trials_in_flight), (2, 1, 1));
        // The made nodes, however stagnant and bad, are not candidates while the trial lasts.
        let made = bdt.graph.nodes[3].trial.as_ref().unwrap().made.clone();
        for &m in &made {
            bdt.graph.nodes[m].grow = eligible(1.0);
        }
        for leaf in 1..=2 {
            bdt.graph.nodes[leaf].grow = GrowStats::default();
        }
        assert_eq!(bdt.graph.grow_schedule(1), None);
        assert_eq!((bdt.graph.split_number, bdt.graph.grow.credits), (2, 1));
        // A long run: as many trials in flight as the slots allow, and no more.
        for slots in [1, 2] {
            let mut p = grow_params(100, 500);
            (p.grow.trials, p.grow.trial_z, p.grow.mdl) = (slots, -1e30, -1e30);
            let mut bdt = Bdt::new(&[3; 4], p);
            let mut most = 0;
            for (x, y) in coin_stream(11, 40_000) {
                bdt.train(&x, &y);
                let on = bdt.graph.nodes.iter().filter(|n| n.trial.is_some()).count() as u32;
                assert_eq!(on, bdt.graph.grow.trials_in_flight);
                most = most.max(on);
            }
            assert_eq!(most, slots);
            assert!(bdt.graph.split_number >= 10, "{}", bdt.graph.split_number);
        }
    }

    /// (i) The rate: a stationary stream, every trial passing, makes exactly one split per credit interval after the
    /// first credit, never two.
    #[test]
    fn grow_keeps_one_split_per_credit_interval() {
        let d = 5000u64;
        let mut p = grow_params(100, d);
        (p.grow.trials, p.grow.trial_z, p.grow.mdl) = (1000, -1e30, -1e30);
        let mut bdt = Bdt::new(&[3; 4], p);
        let mut starts = Vec::new();
        for (i, (x, y)) in coin_stream(13, 12 * d as usize + 2500).into_iter().enumerate() {
            let spent = bdt.graph.grow.credits_spent;
            bdt.train(&x, &y);
            if bdt.graph.grow.credits_spent > spent {
                starts.push(i as u64 + 1);
            }
        }
        // Interval k: examples k d + 1 to (k + 1) d; credit k comes after example k d.
        let per: Vec<usize> = (0..13).map(|k| starts.iter().filter(|&&n| n > k * d && n <= (k + 1) * d).count()).collect();
        assert_eq!(per, [0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], "{starts:?}");
        assert_eq!((bdt.graph.grow.credits_refunded, bdt.graph.split_number as usize), (0, starts.len()));
    }

    /// (j) A model file written during a trial holds the split as if kept and loads and predicts in the loader
    /// (whose format is the old one); the state resumes the trial exactly.
    #[test]
    fn grow_mid_trial_files_load_as_kept_and_states_resume_the_trial() {
        let p = grow_params(2000, 1000);
        let data = xor_stream(5, 30_000);
        let mut bdt = Bdt::new(&XOR_CATS, p.clone());
        let mut at = 0;
        for (i, (x, y)) in data.iter().enumerate() {
            bdt.train(x, y);
            if bdt.graph.nodes[0].trial.as_ref().is_some_and(|t| t.examples == 1000) {
                at = i + 1;
                break;
            }
        }
        assert!(at > 0, "no trial in flight");
        bdt.refresh();
        let text = bdt.to_text();
        let back = Bdt::from_text(&text, true).unwrap();
        assert_eq!(back.to_text(), text);
        assert_eq!((back.split_count(), back.node_count()), (1, 4));
        for i in 0..36u8 {
            let x = [i % 2, (i / 2) % 2, (i / 4) % 3, (i / 12) % 3];
            let (mut a, mut b) = ([0.0; 2], [0.0; 2]);
            bdt.predict(&x, &mut a);
            back.predict(&x, &mut b);
            assert_eq!(a, b);
        }
        let state = bdt.to_state().into_bytes();
        let mut resumed = Bdt::from_state(&state, &XOR_CATS, p.clone()).unwrap();
        assert_eq!(resumed.to_state().into_bytes(), state);
        assert!(resumed.graph.nodes[0].trial.as_ref().is_some_and(|t| t.examples == 1000 && t.grow.is_some()));
        for (x, y) in &data[at..] {
            bdt.train(x, y);
            resumed.train(x, y);
        }
        assert_eq!(resumed.to_state().into_bytes(), bdt.to_state().into_bytes());
        assert!(bdt.graph.grow.credits_spent >= 1 && bdt.graph.nodes[0].trial.is_none());
        // Other growth options are refused.
        let other = DelegationParams { grow: GrowParams { credit: 999, ..p.grow }, ..p };
        assert!(Bdt::from_state(&state, &XOR_CATS, other).is_err());
    }

    /// (a) Without the rule its options are not recorded (states as before); with it, the rule and its options are.
    #[test]
    fn grow_options_are_recorded_only_with_the_rule() {
        let p = params(true);
        let off = DelegationParams { grow: GrowParams { credit: 7, ..GrowParams::default() }, ..p.clone() };
        assert_eq!(format!("{off:?}"), format!("{p:?}"));
        assert!(!format!("{p:?}").contains("grow"));
        let on = DelegationParams { trial_error: ErrorMeasure::Log, split_rule: SplitRule::Grow, ..p.clone() };
        let text = format!("{on:?}");
        assert!(text.contains("split_rule: Grow") && text.ends_with(&format!(", grow: {:?} }}", GrowParams::default())), "{text}");
    }

    #[test]
    #[should_panic(expected = "the growth rule makes its own trials")]
    fn grow_refuses_split_trials() {
        Bdt::new(&[3; 4], DelegationParams { split_trial: Some(0.0), ..grow_params(100, 100) });
    }

    #[test]
    #[should_panic(expected = "trial_error log")]
    fn grow_needs_log_loss_trials() {
        Bdt::new(&[3; 4], DelegationParams { trial_error: ErrorMeasure::Winner, ..grow_params(100, 100) });
    }

    // grow4 (theory-2 6.47 (1), 6.48): the age gate, the paired snapshot gain, the snapshot priority and the size
    // margin. Without them every test above is unchanged (grow1-3's behaviour).

    use crate::delegation::{GrowGate, GrowPriority};

    /// `grow_params` with grow4's leaf options.
    fn grow4_params(w: u32, credit: u64, gate: GrowGate, priority: GrowPriority) -> DelegationParams {
        let mut p = grow_params(w, credit);
        (p.grow.gate, p.grow.priority) = (gate, priority);
        p
    }

    /// The age gate: a leaf is eligible once it has closed `age_windows` windows and is out of back-off, stagnant or
    /// not; the stall gate still needs the streak.
    #[test]
    fn grow4_age_gate_needs_own_windows_not_stagnation() {
        let age = grow4_params(100, 1000, GrowGate::Age, GrowPriority::Excess).grow;
        let stall = grow_params(100, 1000).grow;
        let s = |closes: u64, streak: u32, after: u64| GrowStats {
            prev_exists: true,
            stall_streak: streak,
            closes,
            eligible_after_close: after,
            ..GrowStats::default()
        };
        assert!(!age.eligible(&s(2, 0, 0)) && age.eligible(&s(3, 0, 0)) && age.eligible(&s(9, 0, 0)));
        assert!(!stall.eligible(&s(3, 0, 0)) && stall.eligible(&s(3, 2, 0)));
        assert!(!age.eligible(&s(4, 5, 5)) && age.eligible(&s(5, 0, 5)), "back-off holds under the age gate");
        assert!(!age.eligible(&GrowStats { prev_exists: false, ..s(3, 0, 0) }));
        let five = GrowParams { age_windows: 5, ..age };
        assert!(!five.eligible(&s(4, 0, 0)) && five.eligible(&s(5, 0, 0)));
        // In a run: with credits waiting, the root's first trial starts at its third close (example 3w + 1), and
        // after each undone trial when its back-off (1, 2, 4 windows) ends, never earlier.
        let w = 200;
        let mut p = grow4_params(w, 100, GrowGate::Age, GrowPriority::Excess);
        p.grow.trial_z = f64::INFINITY;
        let mut bdt = Bdt::new(&[3; 4], p);
        let (mut starts, mut undos) = (Vec::new(), Vec::new());
        for (i, (x, y)) in coin_stream(7, 400_000).into_iter().enumerate() {
            let (on_trial, refunded) = (bdt.graph.nodes[0].trial.is_some(), bdt.graph.grow.credits_refunded);
            bdt.train(&x, &y);
            let g = &bdt.graph;
            if !on_trial && g.nodes[0].trial.is_some() {
                starts.push((g.nodes[0].grow.closes, i as u64 + 1));
            }
            if g.grow.credits_refunded > refunded {
                let s = g.nodes[0].grow;
                undos.push(s.eligible_after_close);
            }
            if starts.len() == 4 {
                break;
            }
        }
        assert_eq!(starts[0], (3, 3 * w as u64 + 1), "{starts:?}");
        for (k, &from) in undos.iter().take(3).enumerate() {
            assert_eq!(starts[k + 1].0, from, "start {} at close {} (eligible from {from})", k + 1, starts[k + 1].0);
        }
    }

    /// The paired snapshot gain is exact: per scored example, the frozen copy's log-loss minus the online leaf's,
    /// both before the leaf trains on it, in 20 blocks per window; the first example after a close scores 0 (the copy
    /// is the leaf); logged at the next close. Recomputed here from the leaf and the copy themselves.
    #[test]
    fn grow4_snapshot_gain_is_the_paired_difference_on_the_same_examples() {
        let w = 400u32;
        // No credits: the root stays the only leaf.
        let p = grow4_params(w, u64::MAX / 2, GrowGate::Stall, GrowPriority::Snapshot);
        let len = (w / 20).max(1);
        let mut bdt = Bdt::new(&XOR_CATS, p);
        let (mut mine, mut checked, mut firsts) = (Blocks::default(), 0, 0);
        let mut copy: Option<NodeModel> = None;
        for (x, y) in xor_stream(17, 12 * w as usize + 7) {
            let closes = bdt.graph.nodes[0].grow.closes;
            // Before the step: the online leaf's and the copy's losses on the example.
            let (mut a, mut b) = ([0.0; 2], [0.0; 2]);
            bdt.graph.nodes[0].classifier.predict(&x, &mut a);
            let online = bdt.graph.error(&a, &y);
            let snapped = copy.as_ref().map(|c| {
                c.predict(&x, &mut b);
                bdt.graph.error(&b, &y)
            });
            let pre_close_copy = bdt.graph.nodes[0].classifier.clone();
            bdt.train(&x, &y);
            let s = bdt.graph.nodes[0].grow;
            if s.closes > closes {
                // This example came after a close: the leaf's copy is the leaf as it was before it, so this
                // example's difference is 0 and opens the new window's blocks.
                if copy.is_some() && mine.count >= 2 {
                    let (mean, se) = mine.stats();
                    assert_eq!((s.snap_exists, s.snap_gain, s.snap_se), (true, mean, se), "close {}", s.closes);
                    checked += 1;
                }
                assert_eq!(model_state(bdt.graph.nodes[0].snap.as_deref().unwrap()), model_state(&pre_close_copy));
                copy = Some(pre_close_copy.clone());
                mine = Blocks::default();
                let d = {
                    let mut c = [0.0; 2];
                    pre_close_copy.predict(&x, &mut c);
                    bdt.graph.error(&c, &y) - online
                };
                assert_eq!(d, 0.0);
                firsts += 1;
                mine.add(d, len);
            } else if let Some(sn) = snapped {
                mine.add(sn - online, len);
            }
            assert_eq!(bdt.graph.nodes[0].grow.snap_blocks, mine);
        }
        assert!(checked >= 10 && firsts >= 11, "{checked} {firsts}");
    }

    /// The snapshot priority: a measured leaf before one without, the smallest gain first; ties and unmeasured
    /// leaves by grow1's order (excess, examples, index). The excess priority ignores the snapshots.
    #[test]
    fn grow4_snapshot_priority_picks_the_smallest_learning_gain() {
        type Case = [(f64, Option<f64>); 3];
        let cases: [(Case, usize, usize); 5] = [
            ([(0.02, Some(0.03)), (0.05, Some(0.01)), (0.01, Some(0.02))], 2, 2),
            ([(0.02, Some(0.03)), (0.05, None), (0.01, Some(0.02))], 3, 2),
            ([(0.02, None), (0.05, None), (0.01, None)], 2, 2),
            ([(0.02, Some(0.01)), (0.05, Some(0.01)), (0.0, Some(0.02))], 2, 2),
            ([(0.02, Some(-0.001)), (0.01, Some(0.04)), (0.05, None)], 1, 3),
        ];
        for (case, want_snapshot, want_excess) in cases {
            for (priority, want) in [(GrowPriority::Snapshot, want_snapshot), (GrowPriority::Excess, want_excess)] {
                let mut bdt = three_leaves(grow4_params(100, 1000, GrowGate::Stall, priority));
                bdt.graph.grow.credits = 1;
                for (leaf, (excess, snap)) in (1..=3).zip(case) {
                    let mut s = eligible(excess);
                    if let Some(g) = snap {
                        (s.snap_exists, s.snap_gain) = (true, g);
                    }
                    bdt.graph.nodes[leaf].grow = s;
                }
                assert_eq!(bdt.graph.grow_schedule(1), Some(want), "{case:?} {priority:?}");
            }
        }
    }

    /// The size margin: with the z and MDL bars off, a trial is kept iff its gain exceeds --grow-min-gain; the gain
    /// itself is the same.
    #[test]
    fn grow4_min_gain_is_a_third_bar() {
        let verdict = |min_gain: f64| {
            LAST_VERDICT.with(|v| v.set(None));
            let mut p = grow_params(2000, 1000);
            (p.grow.trial_z, p.grow.mdl, p.grow.min_gain) = (0.0, 0.0, min_gain);
            let mut bdt = Bdt::new(&XOR_CATS, p);
            for (x, y) in xor_stream(5, 30_000) {
                bdt.train(&x, &y);
                if let Some(v) = LAST_VERDICT.with(|v| v.get()) {
                    return v;
                }
            }
            panic!("no growth trial ended");
        };
        let (gain, se, _, keep) = verdict(0.0);
        assert!(keep && gain > 0.005, "{gain} {se}");
        for (m, kept) in [(0.9 * gain, true), (1.1 * gain, false)] {
            let v = verdict(m);
            assert_eq!((v.0, v.1, v.3), (gain, se, kept), "min gain {m}");
        }
    }

    /// States: the snapshot statistics and the frozen copies resume exactly mid-window (and mid-trial), and are
    /// recorded only under grow4's options, which a state's reader must share.
    #[test]
    fn grow4_snapshots_resume_exactly() {
        let mut p = grow4_params(500, 2000, GrowGate::Age, GrowPriority::Snapshot);
        (p.grow.mdl, p.grow.min_gain) = (0.0, 0.001);
        let data = xor_stream(23, 40_000);
        let mut bdt = Bdt::new(&XOR_CATS, p.clone());
        let mut saves = 0;
        for (i, (x, y)) in data.iter().enumerate() {
            bdt.train(x, y);
            // Mid-window, with snapshots and partial blocks: a state, read back and trained alike.
            let snaps = bdt.graph.nodes.iter().filter(|n| n.snap.is_some()).count();
            if i % 3_701 == 1_850 && snaps > 0 && bdt.graph.nodes.iter().any(|n| n.grow.snap_blocks.in_block > 0) {
                let state = bdt.to_state().into_bytes();
                let back = Bdt::from_state(&state, &XOR_CATS, p.clone()).unwrap();
                assert_eq!(back.to_state().into_bytes(), state);
                let n = |b: &Bdt| b.graph.nodes.iter().filter(|n| n.snap.is_some()).count();
                assert_eq!(n(&back), snaps);
                // Without grow4's options the reader refuses the state.
                let plain = DelegationParams { grow: GrowParams { gate: GrowGate::Stall, ..p.grow }, ..p.clone() };
                assert!(Bdt::from_state(&state, &XOR_CATS, plain).is_err());
                saves += 1;
            }
        }
        assert!(saves >= 3, "{saves}");
        // The resumed run is the run: one process against a chain of saves.
        let mut one = Bdt::new(&XOR_CATS, p.clone());
        let mut chain = Bdt::new(&XOR_CATS, p.clone());
        for (i, (x, y)) in data.iter().enumerate() {
            one.train(x, y);
            chain.train(x, y);
            if i % 997 == 0 {
                chain = Bdt::from_state(&chain.to_state().into_bytes(), &XOR_CATS, p.clone()).unwrap();
            }
        }
        assert_eq!(one.to_state().into_bytes(), chain.to_state().into_bytes());
        assert!(one.graph.grow.credits_spent >= 3, "{}", one.graph.grow.credits_spent);
    }

    /// grow4's options are recorded only when they differ from grow1's behaviour.
    #[test]
    fn grow4_options_are_recorded_only_when_set() {
        let d = GrowParams::default();
        let text = format!("{d:?}");
        assert!(!text.contains("gate") && !text.contains("priority") && !text.contains("min_gain"), "{text}");
        let age = format!("{:?}", GrowParams { gate: GrowGate::Age, ..d });
        assert!(age.ends_with("warn_nodes: 55, gate: Age, age_windows: 3 }"), "{age}");
        let snap = format!("{:?}", GrowParams { priority: GrowPriority::Snapshot, min_gain: 0.001, ..d });
        assert!(snap.ends_with("warn_nodes: 55, priority: Snapshot, min_gain: 0.001 }"), "{snap}");
        assert_eq!(format!("{:?}", GrowParams { age_windows: 7, ..d }), text, "the age is recorded with the age gate only");
        assert!(!d.snapshot_on() && GrowParams { gate: GrowGate::Age, ..d }.snapshot_on());
    }

    /// grow5's tests: a leaf's AdaGrad sums (LR leaves).
    fn lr_sums(m: &NodeModel) -> Vec<f64> {
        match m {
            NodeModel::Lr(l) => l.squares().to_vec(),
            _ => panic!("LR leaves"),
        }
    }

    /// grow5 (theory-2 6.62 (a)): at a growth-rule split under --forget-factor 10 the new leaves' AdaGrad sums are the
    /// leaf's divided by 10 in both modes; the trial's shadow's are the leaf's divided by 10 under split (grow1-4's) and
    /// the leaf's own under none. The weights are the leaf's everywhere (forgetting changes only the step).
    #[test]
    fn grow5_the_shadow_keeps_the_leafs_sums_under_none() {
        use crate::delegation::GrowShadowForget;
        for mode in [GrowShadowForget::Split, GrowShadowForget::None] {
            let mut p = grow_params(10_000, 1000);
            p.forget_factor = 10.0;
            p.grow.shadow_forget = mode;
            let mut bdt = Bdt::new(&[3; 4], p);
            for (x, y) in xor_stream(5, 3000) {
                bdt.train(&x, &y);
            }
            assert!(bdt.graph.nodes[0].trial.is_none() && matches!(bdt.graph.nodes[0].kind, Kind::Leaf));
            let before = bdt.graph.nodes[0].classifier.clone();
            let leaf = lr_sums(&before);
            assert!(leaf.iter().filter(|&&s| s > 0.0).count() > 3, "{leaf:?}");
            let tenth: Vec<f64> = leaf.iter().map(|s| s / 10.0).collect();
            bdt.graph.split_leaf(0);
            let g = &bdt.graph;
            let trial = g.nodes[0].trial.as_ref().expect("a growth trial");
            assert!(trial.grow.is_some() && trial.made.len() == 3);
            let mut forgotten = before.clone();
            forgotten.forget(10.0);
            for &c in &trial.made {
                assert_eq!(lr_sums(&g.nodes[c].classifier), tenth, "the new leaves forget in both modes");
                assert_eq!(model_state(&g.nodes[c].classifier), model_state(&forgotten));
            }
            match mode {
                GrowShadowForget::Split => {
                    assert_eq!(lr_sums(&trial.shadow), tenth);
                    assert_eq!(model_state(&trial.shadow), model_state(&forgotten));
                }
                GrowShadowForget::None => {
                    assert_eq!(lr_sums(&trial.shadow), leaf);
                    assert_eq!(model_state(&trial.shadow), model_state(&before), "the shadow is the leaf as it was");
                }
            }
        }
    }

    /// grow5 (theory-2 6.62 (a); the coordinator's undo check): an undone trial leaves the leaf as the shadow carried
    /// it through the trial. A leaf splits at the start of its step (`check_leaf_split`), so the trial's first example
    /// is that step's: the shadow starts as the leaf before the step (as it is under none; forgotten by 10 under split,
    /// grow1-4's), trains on every example of the trial, and at the verdict's step the restored leaf is that shadow
    /// trained on the step's example. So under none the restored leaf's sums trace back to the leaf's own at the split
    /// (each at least as large), under split to a tenth of them.
    #[test]
    fn grow5_an_undone_trial_leaves_the_carried_shadow() {
        use crate::delegation::GrowShadowForget;
        let w = 200;
        for mode in [GrowShadowForget::Split, GrowShadowForget::None] {
            let mut p = grow_params(w, 100);
            p.forget_factor = 10.0;
            p.grow.trial_z = f64::INFINITY;
            p.grow.shadow_forget = mode;
            let mut bdt = Bdt::new(&[3; 4], p);
            // The shadow's start trained on the trial's examples so far; the leaf's sums at the split.
            let mut replay: Option<NodeModel> = None;
            let mut parent: Vec<f64> = Vec::new();
            let mut undone = 0;
            for (x, y) in coin_stream(11, 400_000) {
                let ending = (bdt.graph.nodes[0].trial.as_ref()).filter(|t| t.examples >= w).map(|t| t.shadow.clone());
                let was_on_trial = bdt.graph.nodes[0].trial.is_some();
                let leaf_before = matches!(bdt.graph.nodes[0].kind, Kind::Leaf).then(|| bdt.graph.nodes[0].classifier.clone());
                bdt.train(&x, &y);
                let g = &bdt.graph;
                if let Some(shadow) = ending {
                    let mut carried = replay.take().expect("the trial was followed from its start");
                    assert_eq!(model_state(&carried), model_state(&shadow), "the shadow = its start + the trial's examples");
                    assert!(matches!(g.nodes[0].kind, Kind::Leaf) && g.nodes[0].trial.is_none(), "undone");
                    carried.train(&x, &y);
                    assert_eq!(model_state(&g.nodes[0].classifier), model_state(&carried), "the leaf is the carried shadow");
                    let restored = lr_sums(&g.nodes[0].classifier);
                    match mode {
                        GrowShadowForget::None => {
                            assert!(restored.iter().zip(&parent).all(|(r, p)| r >= p), "under none from the leaf's own sums")
                        }
                        GrowShadowForget::Split => {
                            assert!(restored.iter().zip(&parent).any(|(r, p)| *r < 0.5 * p), "under split from a tenth of them")
                        }
                    }
                    undone += 1;
                    if undone == 2 {
                        break;
                    }
                    continue;
                }
                if was_on_trial {
                    replay.as_mut().expect("followed").train(&x, &y);
                } else if let Some(trial) = g.nodes[0].trial.as_ref() {
                    // The split's step: the shadow started as the leaf before it (forgotten under split) and trained on
                    // this step's example.
                    let before = leaf_before.expect("the root was a leaf");
                    parent = lr_sums(&before);
                    let mut start = before;
                    if mode == GrowShadowForget::Split {
                        start.forget(10.0);
                    }
                    assert_eq!(trial.examples, 1);
                    start.train(&x, &y);
                    assert_eq!(model_state(&start), model_state(&trial.shadow), "the shadow's start");
                    replay = Some(start);
                }
            }
            assert_eq!(undone, 2, "two trials undone");
        }
    }

    /// grow5's option is recorded only when it differs from grow1-4's behaviour.
    #[test]
    fn grow5_shadow_forget_is_recorded_only_when_set() {
        use crate::delegation::GrowShadowForget;
        let d = GrowParams::default();
        assert_eq!(d.shadow_forget, GrowShadowForget::Split);
        let text = format!("{d:?}");
        assert!(!text.contains("shadow_forget"), "{text}");
        let none = format!("{:?}", GrowParams { shadow_forget: GrowShadowForget::None, ..d });
        assert!(none.ends_with("warn_nodes: 55, shadow_forget: None }"), "{none}");
    }
}
