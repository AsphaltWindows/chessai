//! Bayesian Delegation Tree, a port of `c_src/models/bdt/bdt.c`.
//!
//! A tree of CNBP classifiers. Leaves predict the classes. Branches predict
//! which of their children will classify an input best and delegate to it.
//!
//! Training a branch asks every child for a prediction, trains the branch's
//! own classifier with the index of the child with the smallest error as the
//! label, and then trains only that child. A leaf splits into a branch with
//! `branch_factor` children once its error over the last `split_threshold`
//! examples is no better than its lifetime average. The children start as
//! copies of the leaf with their counts divided by `forget_factor`, so they
//! predict the same but adapt faster.
//!
//! Training order, tie-breaking and file format follow the C version
//! exactly, so both produce identical trees and predictions.

use std::fmt::Write as _;
use std::io;
use std::path::Path;

use crate::cnbp::Cnbp;
use crate::textfmt::{Tokens, invalid};

#[derive(Clone, Debug)]
enum Kind {
    Leaf,
    Branch { children: Vec<usize> },
}

#[derive(Clone, Debug)]
struct Node {
    kind: Kind,
    /// Predicts classes for a leaf, or the best child for a branch.
    classifier: Cnbp,
    num_trained_total: u32,
    num_trained_last: u32,
    error_total: f64,
    error_last: f64,
    /// Mean error of the previous full window, or -1 before the first.
    error_prev: f64,
    /// Sum of squared errors in the current window, and the squared standard
    /// error of the previous window's mean; for `split_z`. Not saved.
    error_sq_last: f64,
    error_prev_se2: f64,
    /// The tree's error sum and count when this node's window started.
    window_tree_sum: f64,
    window_tree_count: u64,
    /// Output of `classifier` for the position of the current training step;
    /// valid while `cache_step` equals the tree's `step`.
    cache: Vec<f64>,
    cache_step: u64,
}

/// How the error of a prediction is measured, for the split check and the
/// routers' targets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum ErrorMeasure {
    /// Half the absolute difference: 1 - P(actual class) for a one-hot label.
    Abs,
    /// Log-loss: -ln P(actual class).
    Log,
}

/// When a leaf splits, checked every `split_threshold` examples.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum SplitRule {
    /// The window's mean error rose above the previous window's by more than
    /// `split_z` standard errors.
    Worse,
    /// The leaf stalled - its error did not fall by more than `split_z`
    /// standard errors since the previous window - while its mean error is
    /// above the whole tree's over the same window.
    StallHigh,
}

#[derive(Clone, Debug)]
pub struct BdtParams {
    pub class_num: usize,
    pub branch_factor: usize,
    pub split_threshold: u32,
    pub split_limit: u32,
    pub forget_factor: f64,
    pub nb_alpha: f64,
    pub use_probs: bool,
    /// Not saved in model files; `Abs` when loading.
    pub error: ErrorMeasure,
    /// A leaf splits when its window's mean error exceeds the previous
    /// window's by more than this many standard errors of the difference.
    /// Not saved in model files; 0 when loading.
    pub split_z: f64,
    /// Not saved in model files; `Worse` when loading.
    pub split_rule: SplitRule,
}

#[derive(Clone, Debug)]
pub struct Bdt {
    categories: Vec<u8>,
    params: BdtParams,
    split_number: u32,
    nodes: Vec<Node>,
    /// Counts training steps, starting at 1; a `cache_step` of 0 marks no cache.
    step: u64,
    /// Sum and count of the whole tree's prediction errors while training,
    /// for `SplitRule::StallHigh`. Not saved.
    tree_error_sum: f64,
    tree_error_count: u64,
}

impl Bdt {
    pub fn new(categories: &[u8], params: BdtParams) -> Self {
        assert!(params.split_threshold > 0, "split_threshold must be at least 1");
        assert!(params.branch_factor >= 2, "branch_factor must be at least 2");
        let root = Node::new(
            Kind::Leaf,
            Cnbp::new(params.class_num, categories, params.nb_alpha, params.use_probs),
        );
        Bdt {
            categories: categories.to_vec(),
            params,
            split_number: 0,
            nodes: vec![root],
            step: 0,
            tree_error_sum: 0.0,
            tree_error_count: 0,
        }
    }

    pub fn params(&self) -> &BdtParams {
        &self.params
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn split_count(&self) -> u32 {
        self.split_number
    }

    /// Writes the probability of each class into `out`.
    ///
    /// # Panics
    /// With `use_probs`, if the tree was trained since the last `refresh()`.
    pub fn predict(&self, data: &[u8], out: &mut [f64]) {
        self.predict_routed(data, data, out)
    }

    /// Predicts `data` with the leaf that `route_by` is routed to, so that
    /// several positions can be compared by the same leaf.
    pub fn predict_routed(&self, route_by: &[u8], data: &[u8], out: &mut [f64]) {
        let mut idx = 0;
        loop {
            let node = &self.nodes[idx];
            match &node.kind {
                Kind::Leaf => return node.classifier.predict(data, out),
                Kind::Branch { children } => idx = children[best_child(&node.classifier, route_by)],
            }
        }
    }

    /// Divides the counts of every node's classifier by `factor`. The split
    /// statistics are left alone.
    pub fn forget(&mut self, factor: f64) {
        for node in &mut self.nodes {
            node.classifier.forget(factor);
            node.cache_step = 0;
        }
    }

    /// Recomputes cached probabilities after training (see `Cnbp::refresh`).
    pub fn refresh(&mut self) {
        for node in &mut self.nodes {
            node.classifier.refresh();
        }
    }

    /// Prediction used during training: always from the raw counts, so trees
    /// with `use_probs` do not recalculate probabilities after every step.
    fn predict_training(&mut self, mut idx: usize, data: &[u8], out: &mut [f64]) {
        let mut scores = [0.0; 256];
        loop {
            match &self.nodes[idx].kind {
                Kind::Leaf => return self.node_output(idx, data, out),
                Kind::Branch { .. } => {
                    let scores = &mut scores[..self.params.branch_factor];
                    self.node_output(idx, data, scores);
                    let Kind::Branch { children } = &self.nodes[idx].kind else { unreachable!() };
                    idx = children[argmax(scores)];
                }
            }
        }
    }

    /// Runs a node's own classifier from counts. The output is cached for the
    /// rest of the step: the position does not change within a step, and a
    /// classifier is only trained after the predictions that use it.
    fn node_output(&mut self, idx: usize, data: &[u8], out: &mut [f64]) {
        let step = self.step;
        let node = &mut self.nodes[idx];
        if node.cache_step != step {
            node.classifier.predict_counts(data, &mut node.cache);
            node.cache_step = step;
        }
        out.copy_from_slice(&node.cache);
    }

    /// Adds one data point. `label` must sum to 1.
    pub fn train(&mut self, data: &[u8], label: &[f64]) {
        self.step += 1;
        let mut out = vec![0.0; self.params.class_num];
        self.predict_training(0, data, &mut out);
        let error = self.error(&out, label);
        self.tree_error_sum += error;
        self.tree_error_count += 1;
        self.train_node(0, data, label, error, &mut out);
    }

    /// `error` is the error of the node's prediction for this example; only
    /// leaves use it, routers record the error of their own choice.
    /// `scratch` holds `class_num` values for the children's predictions.
    fn train_node(&mut self, idx: usize, data: &[u8], label: &[f64], error: f64, scratch: &mut [f64]) {
        match &self.nodes[idx].kind {
            Kind::Leaf => {
                let node = &mut self.nodes[idx];
                if node.num_trained_last >= self.params.split_threshold
                    && self.split_number < self.params.split_limit
                {
                    // Split when the leaf stopped improving: this window's mean
                    // error is above the previous window's. Comparing with the
                    // lifetime mean instead is skewed by the high error of the
                    // first data.
                    let n = node.num_trained_last as f64;
                    let recent = node.error_last / n;
                    let se2 = (node.error_sq_last / n - recent * recent).max(0.0) / n;
                    let (previous, previous_se2) = (node.error_prev, node.error_prev_se2);
                    node.error_prev = recent;
                    node.error_prev_se2 = se2;
                    let margin = self.params.split_z * (se2 + previous_se2).sqrt();
                    let split = previous >= 0.0
                        && match self.params.split_rule {
                            SplitRule::Worse => recent > previous + margin,
                            SplitRule::StallHigh => {
                                let tree_mean = (self.tree_error_sum - node.window_tree_sum)
                                    / (self.tree_error_count - node.window_tree_count).max(1) as f64;
                                previous - recent < margin && recent > tree_mean
                            }
                        };
                    if split {
                        self.split(idx);
                    }
                    let (sum, count) = (self.tree_error_sum, self.tree_error_count);
                    let node = &mut self.nodes[idx];
                    node.num_trained_last = 0;
                    node.error_last = 0.0;
                    node.error_sq_last = 0.0;
                    node.window_tree_sum = sum;
                    node.window_tree_count = count;
                    return self.train_node(idx, data, label, error, scratch);
                }

                let node = &mut self.nodes[idx];
                node.classifier.train(data, label);
                node.cache_step = 0;
                node.record(error);
            }
            Kind::Branch { children } => {
                let children = children.clone();
                let mut child_errors = vec![0.0; children.len()];
                let (mut best, mut min_error) = (0, f64::INFINITY);
                for (i, &child) in children.iter().enumerate() {
                    self.predict_training(child, data, scratch);
                    child_errors[i] = self.error(scratch, label);
                    // Strict comparison: ties go to the lowest index, as in C.
                    if i == 0 || child_errors[i] < min_error {
                        min_error = child_errors[i];
                        best = i;
                    }
                }

                let mut target = vec![0.0; self.params.branch_factor];
                target[best] = 1.0;
                // Like a leaf, the router records the error of its own
                // prediction (which child will do best) before training on it.
                let mut router = [0.0; 256];
                let router = &mut router[..self.params.branch_factor];
                self.node_output(idx, data, router);
                let router_error = self.error(router, &target);
                self.nodes[idx].classifier.train(data, &target);
                self.nodes[idx].cache_step = 0;

                // The router is trained on which child predicted the actual
                // outcome best, then the position goes to the child the
                // updated router picks from the position alone, as in
                // prediction, so each child learns about the positions it will
                // be asked about.
                self.node_output(idx, data, router);
                let routed = argmax(router);

                self.train_node(children[routed], data, label, child_errors[routed], scratch);
                self.nodes[idx].record(router_error);
            }
        }
    }

    fn split(&mut self, idx: usize) {
        let first = self.nodes.len();
        for _ in 0..self.params.branch_factor {
            let mut classifier = self.nodes[idx].classifier.clone();
            classifier.forget(self.params.forget_factor);
            let mut child = Node::new(Kind::Leaf, classifier);
            child.window_tree_sum = self.tree_error_sum;
            child.window_tree_count = self.tree_error_count;
            self.nodes.push(child);
        }

        let node = &mut self.nodes[idx];
        node.kind = Kind::Branch { children: (first..first + self.params.branch_factor).collect() };
        node.classifier = Cnbp::new(
            self.params.branch_factor,
            &self.categories,
            self.params.nb_alpha,
            self.params.use_probs,
        );
        node.cache = vec![0.0; self.params.branch_factor];
        node.cache_step = 0;
        node.num_trained_total = 0;
        node.error_total = 0.0;
        self.split_number += 1;
    }

    /// Serializes in the C text format (`bdt_to_file`).
    pub fn to_text(&self) -> String {
        let p = &self.params;
        let mut s = String::new();
        writeln!(s, "{}", self.categories.len()).unwrap();
        for cat in &self.categories {
            writeln!(s, "{cat}").unwrap();
        }
        writeln!(s, "{}", p.class_num).unwrap();
        writeln!(s, "{}", p.branch_factor).unwrap();
        writeln!(s, "{}", p.split_threshold).unwrap();
        writeln!(s, "{}", p.split_limit).unwrap();
        writeln!(s, "{}", self.split_number).unwrap();
        writeln!(s, "{}", self.nodes.len()).unwrap();
        writeln!(s, "{:.6}", p.forget_factor).unwrap();
        writeln!(s, "{:.6}", p.nb_alpha).unwrap();

        for (id, node) in self.nodes.iter().enumerate() {
            let kind = match node.kind {
                Kind::Leaf => 0,
                Kind::Branch { .. } => 1,
            };
            writeln!(s, "{kind}").unwrap();
            writeln!(s, "{id}").unwrap();
            writeln!(s, "{}", node.num_trained_total).unwrap();
            writeln!(s, "{:.6}", node.error_total).unwrap();
            writeln!(s, "{}", node.num_trained_last).unwrap();
            writeln!(s, "{:.6}", node.error_last).unwrap();
            writeln!(s, "{:.6}", node.error_prev).unwrap();
            if let Kind::Branch { children } = &node.kind {
                for child in children {
                    writeln!(s, "{child}").unwrap();
                }
            }
            node.classifier.write_counts(&mut s);
        }
        s
    }

    /// Parses the C text format (`bdt_from_file`). The node ids stored in
    /// the file must match their positions, as the C code always writes them.
    pub fn from_text(text: &str, use_probs: bool) -> io::Result<Self> {
        let mut t = Tokens::new(text);
        let cat_num: usize = t.next()?;
        let categories = (0..cat_num).map(|_| t.next::<u8>()).collect::<io::Result<Vec<_>>>()?;
        let class_num: usize = t.next()?;
        let branch_factor: usize = t.next()?;
        let split_threshold: u32 = t.next()?;
        let split_limit: u32 = t.next()?;
        let split_number: u32 = t.next()?;
        let nodes_num: usize = t.next()?;
        let forget_factor: f64 = t.next()?;
        let nb_alpha: f64 = t.next()?;
        let params = BdtParams {
            class_num,
            branch_factor,
            split_threshold,
            split_limit,
            forget_factor,
            nb_alpha,
            use_probs,
            error: ErrorMeasure::Abs,
            split_z: 0.0,
            split_rule: SplitRule::Worse,
        };

        let mut nodes = Vec::with_capacity(nodes_num);
        for n in 0..nodes_num {
            let kind: u32 = t.next()?;
            let id: usize = t.next()?;
            if id != n {
                return Err(invalid(format!("node {n} has id {id}")));
            }
            let num_trained_total = t.next()?;
            let error_total = t.next()?;
            let num_trained_last = t.next()?;
            let error_last = t.next()?;
            let error_prev = t.next()?;
            let (kind, classes) = match kind {
                0 => (Kind::Leaf, class_num),
                1 => {
                    let children = (0..branch_factor).map(|_| t.next()).collect::<io::Result<Vec<usize>>>()?;
                    if children.iter().any(|&c| c >= nodes_num) {
                        return Err(invalid(format!("node {n} has a child out of range")));
                    }
                    (Kind::Branch { children }, branch_factor)
                }
                _ => return Err(invalid(format!("node {n} has unknown type {kind}"))),
            };
            let mut classifier = Cnbp::new(classes, &categories, nb_alpha, use_probs);
            classifier.read_counts(&mut t)?;
            nodes.push(Node {
                num_trained_total,
                num_trained_last,
                error_total,
                error_last,
                error_prev,
                ..Node::new(kind, classifier) });
        }

        Ok(Bdt { categories, params, split_number, nodes, step: 0, tree_error_sum: 0.0, tree_error_count: 0 })
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        std::fs::write(path, self.to_text())
    }

    pub fn load(path: &Path, use_probs: bool) -> io::Result<Self> {
        Self::from_text(&std::fs::read_to_string(path)?, use_probs)
    }
}

impl Node {
    fn new(kind: Kind, classifier: Cnbp) -> Self {
        Node {
            kind,
            cache: vec![0.0; classifier.class_num()],
            cache_step: 0,
            classifier,
            num_trained_total: 0,
            num_trained_last: 0,
            error_total: 0.0,
            error_last: 0.0,
            error_prev: -1.0,
            error_sq_last: 0.0,
            error_prev_se2: 0.0,
            window_tree_sum: 0.0,
            window_tree_count: 0,
        }
    }

    fn record(&mut self, error: f64) {
        self.num_trained_last += 1;
        self.num_trained_total += 1;
        self.error_last += error;
        self.error_total += error;
        self.error_sq_last += error * error;
    }
}

/// Index of the child a branch classifier rates highest.
fn best_child(classifier: &Cnbp, data: &[u8]) -> usize {
    let mut scores = [0.0; 256];
    let scores = &mut scores[..classifier.class_num()];
    classifier.predict(data, scores);
    argmax(scores)
}

/// Index of the largest value, the first one on ties (as in C).
fn argmax(values: &[f64]) -> usize {
    let mut best = 0;
    for (i, &v) in values.iter().enumerate() {
        if v > values[best] {
            best = i;
        }
    }
    best
}

/// Total variation distance between a prediction and a label.
impl Bdt {
    fn error(&self, predicted: &[f64], actual: &[f64]) -> f64 {
        match self.params.error {
            ErrorMeasure::Abs => prediction_error(predicted, actual),
            ErrorMeasure::Log => log_loss(predicted, actual),
        }
    }
}

/// -sum(actual * ln predicted); -ln P(actual class) for a one-hot label.
pub fn log_loss(predicted: &[f64], actual: &[f64]) -> f64 {
    predicted
        .iter()
        .zip(actual)
        .filter(|(_, a)| **a != 0.0)
        .map(|(p, a)| -a * p.max(f64::MIN_POSITIVE).ln())
        .sum()
}

fn prediction_error(expected: &[f64], actual: &[f64]) -> f64 {
    let mut error = 0.0;
    for (e, a) in expected.iter().zip(actual) {
        error += (e - a).abs();
    }
    error / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(use_probs: bool) -> BdtParams {
        BdtParams {
            class_num: 2,
            branch_factor: 2,
            split_threshold: 4,
            split_limit: 3,
            forget_factor: 10.0,
            nb_alpha: 1.0,
            use_probs,
            error: ErrorMeasure::Abs,
            split_z: 0.0,
            split_rule: SplitRule::Worse,
        }
    }

    /// Two clusters with opposite labels, fed in blocks so leaves stall and split.
    fn trained(use_probs: bool) -> Bdt {
        let mut bdt = Bdt::new(&[3, 3], params(use_probs));
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
}
