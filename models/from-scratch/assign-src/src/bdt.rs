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
        Bdt { graph: Graph::new(categories, params) }
    }

    /// A single leaf that starts as `root`, a classifier of the leaves' kind,
    /// instead of an untrained one.
    pub fn starting_from(root: NodeModel, categories: &[u8], params: DelegationParams) -> Self {
        Bdt { graph: Graph::starting_from(root, categories, params) }
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
            let routed = g.assigned_child(idx, routed, children.len());
            g.train_shadow(idx, data, label);
            return self.train_node(children[routed], Some(idx), data, label, None, scratch);
        }

        // Error routing compares every child's error; cluster routing needs
        // only the error of the child it picks. The router trains on its own
        // pick; the example then descends to the assigned child
        // (`assign_epsilon`: usually the same one).
        let (routed, child_error, router_error) = match g.params.route {
            RouteTarget::Error => {
                let child_errors = g.prediction_errors(&children, data, label, scratch);
                let (routed, router_error) = g.train_router(idx, data, &child_errors);
                let routed = g.assigned_child(idx, routed, children.len());
                (routed, child_errors[routed], router_error)
            }
            RouteTarget::Cluster => {
                let (routed, router_error) = g.train_cluster_router(idx, &children, data, label, scratch);
                let routed = g.assigned_child(idx, routed, children.len());
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
            assign_epsilon: 0.0,
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
                split |= bdt.split_count() > 0;
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

    /// On one cluster an error router sends (nearly) every example to the
    /// child that pulls ahead; with `assign_epsilon` 1 each example trains a
    /// child drawn uniformly instead, so the children share the examples
    /// evenly. The router trains either way.
    #[test]
    fn assign_epsilon_spreads_the_training_examples() {
        let shares = |eps: f64| {
            let params = DelegationParams { split_limit: 0, assign_epsilon: eps, ..params(false) };
            let mut bdt = Bdt::new(&[3, 3], params);
            bdt.graph.split_leaf(0);
            let Kind::Branch { children } = bdt.graph.nodes[0].kind.clone() else { panic!("no split") };
            let n = 3000;
            for i in 0..n {
                bdt.train(&[0, (i % 3) as u8], &[1.0, 0.0]);
            }
            let counts: Vec<f64> = children.iter().map(|&c| bdt.graph.nodes[c].num_trained_total as f64).collect();
            let total: f64 = counts.iter().sum();
            assert_eq!(total, n as f64);
            counts.iter().map(|k| k / total).collect::<Vec<f64>>()
        };
        let routed = shares(0.0);
        assert!(routed.iter().any(|&s| s > 0.9), "{routed:?}");
        let spread = shares(1.0);
        assert!(spread.iter().all(|&s| (s - 0.5).abs() < 0.05), "{spread:?}");
    }

    /// The draw moves an example away from the router's pick with
    /// probability epsilon times (k - 1) / k, the same on every run, and
    /// never at 0.
    #[test]
    fn assigned_child_draws_with_the_given_probability() {
        let mut bdt = Bdt::new(&[3, 3], DelegationParams { branch_factor: 4, assign_epsilon: 0.25, ..params(false) });
        let n = 40_000u64;
        let mut moved = 0;
        for step in 1..=n {
            bdt.graph.step = step;
            moved += (bdt.graph.assigned_child(0, 1, 4) != 1) as u64;
        }
        let rate = moved as f64 / n as f64;
        assert!((rate - 0.25 * 0.75).abs() < 0.01, "{rate}");
        let mut off = Bdt::new(&[3, 3], params(false));
        for step in 1..=1000 {
            off.graph.step = step;
            assert_eq!(off.graph.assigned_child(0, 1, 2), 1);
        }
    }

    /// State files record the option only when it is used, so trees without
    /// it record what they did before it.
    #[test]
    fn assign_epsilon_is_recorded_only_when_used() {
        let p = params(false);
        assert!(!format!("{p:?}").contains("assign_epsilon"));
        let on = DelegationParams { assign_epsilon: 0.1, ..p.clone() };
        assert_eq!(format!("{on:?}"), format!("{p:?}").replace(" }", ", assign_epsilon: 0.1 }"));
    }

    #[test]
    #[should_panic(expected = "assign_epsilon is a probability")]
    fn assign_epsilon_must_be_a_probability() {
        Bdt::new(&[3, 3], DelegationParams { assign_epsilon: 1.5, ..params(false) });
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
}
