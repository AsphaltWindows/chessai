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
//!
//! With CNBP nodes, training order, tie-breaking and file format follow the
//! C version exactly, so both produce identical trees and predictions. The
//! nodes, their statistics and the file format are shared with the BDD
//! (`delegation`).

use std::io;
use std::path::Path;

use crate::delegation::{DelegationParams, Graph, Kind, NodeModel, RouteTarget};
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
        self.train_node(0, data, label, error, &mut scratch);
    }

    /// `error` is the error of the node's prediction for this example, or
    /// None if the example is not scored; only leaves use it, branches record
    /// the error of their own choice. `scratch` holds `class_num` values.
    fn train_node(&mut self, idx: usize, data: &[u8], label: &[f64], error: Option<f64>, scratch: &mut [f64]) {
        let g = &mut self.graph;
        let Kind::Branch { children } = &g.nodes[idx].kind else {
            if g.check_leaf_split(idx).is_some() {
                // The leaf may have become a branch; train it as it is now.
                return self.train_node(idx, data, label, error, scratch);
            }
            return g.train_leaf(idx, data, label, error);
        };
        let children = children.clone();
        if g.trial_step(idx, data, label, scratch) || g.revise_step(idx, data, label, scratch) {
            // The split was undone or merged; train the node as it is now.
            return self.train_node(idx, data, label, error, scratch);
        }
        if error.is_none() {
            let routed = g.route(idx, data);
            return self.train_node(children[routed], data, label, None, scratch);
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
        self.train_node(children[routed], data, label, Some(child_error), scratch);
        self.graph.record_router(idx, router_error);
    }

    /// Sees a round's games, each the positions in the order played, before
    /// training on them: with shadows, measures the seams their test needs
    /// (see `DelegationParams::revise`). Call it with the tree refreshed.
    pub fn measure_seams<P: AsRef<[u8]>>(&mut self, games: &[&[P]]) {
        self.graph.measure_seams(games)
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
    use crate::delegation::{ErrorMeasure, ErrorTarget, NodeKind, RouteTarget, SplitRule, log_loss};

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
            DelegationParams {
                leaf: NodeKind::Lr,
                route: RouteTarget::Cluster,
                route_switch: Some(0.0),
                error_router: NodeKind::Lr,
                ..base.clone()
            },
            DelegationParams { revise: Some(0.0), reset_leaves: Some(0.0), seam_weight: 0.1, ..base.clone() },
            DelegationParams { leaf: NodeKind::Lr, router: NodeKind::Lr, revise: Some(f64::INFINITY), ..base.clone() },
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
                    round.iter().for_each(|(game, label)| game.iter().for_each(|pos| tree.train(pos, label)));
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
}
