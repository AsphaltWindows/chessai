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
    /// Output of `classifier` for the position of the current training step;
    /// valid while `cache_step` equals the tree's `step`.
    cache: Vec<f64>,
    cache_step: u64,
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
}

#[derive(Clone, Debug)]
pub struct Bdt {
    categories: Vec<u8>,
    params: BdtParams,
    split_number: u32,
    nodes: Vec<Node>,
    /// Counts training steps, starting at 1; a `cache_step` of 0 marks no cache.
    step: u64,
}

impl Bdt {
    pub fn new(categories: &[u8], params: BdtParams) -> Self {
        assert!(params.split_threshold > 0, "split_threshold must be at least 1");
        assert!(params.branch_factor >= 2, "branch_factor must be at least 2");
        let root = Node::new(
            Kind::Leaf,
            Cnbp::new(params.class_num, categories, params.nb_alpha, params.use_probs),
        );
        Bdt { categories: categories.to_vec(), params, split_number: 0, nodes: vec![root], step: 0 }
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
        let mut idx = 0;
        loop {
            let node = &self.nodes[idx];
            match &node.kind {
                Kind::Leaf => return node.classifier.predict(data, out),
                Kind::Branch { children } => idx = children[best_child(&node.classifier, data)],
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
        let error = prediction_error(&out, label);
        self.train_node(0, data, label, error, &mut out);
    }

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
                    let recent = node.error_last / node.num_trained_last as f64;
                    let previous = node.error_prev;
                    node.error_prev = recent;
                    if previous >= 0.0 && recent > previous {
                        self.split(idx);
                    }
                    let node = &mut self.nodes[idx];
                    node.num_trained_last = 0;
                    node.error_last = 0.0;
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
                    child_errors[i] = prediction_error(scratch, label);
                    // Strict comparison: ties go to the lowest index, as in C.
                    if i == 0 || child_errors[i] < min_error {
                        min_error = child_errors[i];
                        best = i;
                    }
                }

                let mut target = vec![0.0; self.params.branch_factor];
                target[best] = 1.0;
                self.nodes[idx].classifier.train(data, &target);
                self.nodes[idx].cache_step = 0;

                // The router is trained on which child predicted the actual
                // outcome best, then the position goes to the child the
                // updated router picks from the position alone, as in
                // prediction, so each child learns about the positions it will
                // be asked about.
                let mut router = [0.0; 256];
                let router = &mut router[..self.params.branch_factor];
                self.node_output(idx, data, router);
                let routed = argmax(router);

                self.train_node(children[routed], data, label, child_errors[routed], scratch);
                self.nodes[idx].record(error);
            }
        }
    }

    fn split(&mut self, idx: usize) {
        let first = self.nodes.len();
        for _ in 0..self.params.branch_factor {
            let mut classifier = self.nodes[idx].classifier.clone();
            classifier.forget(self.params.forget_factor);
            self.nodes.push(Node::new(Kind::Leaf, classifier));
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

        Ok(Bdt { categories, params, split_number, nodes, step: 0 })
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
        }
    }

    fn record(&mut self, error: f64) {
        self.num_trained_last += 1;
        self.num_trained_total += 1;
        self.error_last += error;
        self.error_total += error;
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
