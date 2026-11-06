//! Pieces shared by the delegation models, the Delegation Tree (`bdt`) and
//! DAG (`bdd`).
//!
//! Both are graphs of classifiers, each a CNBP or a logistic regression
//! (`NodeModel`; leaves and routers may be of different kinds). Leaves
//! predict the classes. Delegators (branches) predict which of their
//! children will classify an input best and pass it on. This module holds
//! the nodes and their split statistics, prediction, the error measures, the
//! leaf split check and the model file format (that of
//! `c_src/models/bdt/bdt.c` for CNBP nodes); the models differ in how they
//! train delegators and whether delegators split.

use std::fmt::Write as _;
use std::io;

use crate::cnbp::Cnbp;
use crate::logreg::Logreg;
use crate::textfmt::{Tokens, invalid};

/// How the error of a prediction is measured, for the split check and the
/// routers' targets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum ErrorMeasure {
    /// Half the absolute difference: 1 - P(actual class) for a one-hot label.
    Abs,
    /// Log-loss: -ln P(actual class).
    Log,
    /// Log-loss of who won, -ln(P(winner) / (P(white win) + P(black win))),
    /// for classes [white win, black win, draw]: the part of a prediction
    /// that log-odds play uses. Only decisive examples are scored; draws
    /// train the leaf they are routed to, but not the routers, and do not
    /// count towards split checks. Routers' errors are log-losses.
    Winner,
}

/// When a leaf splits, checked every `split_threshold` examples.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum SplitRule {
    /// The window's mean error rose above the previous window's by more than
    /// `split_z` standard errors.
    Worse,
    /// The leaf stalled - its error did not fall by more than `split_z`
    /// standard errors since the previous window - while its mean error is
    /// at least the whole model's over the same window. A leaf that is the
    /// whole model (the root before any split) splits when it stalls.
    StallHigh,
}

/// What a delegator's router learns to predict.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum RouteTarget {
    /// The child whose prediction of the example has the smallest error.
    Error,
    /// The child the router itself picks, without looking at the label:
    /// hard EM, so a CNBP router becomes a mixture of naive Bayes models
    /// that groups positions whose features vary together (game phase, say),
    /// and each child's naive Bayes sees less of the feature dependence it
    /// cannot model. For its first `split_threshold / 5` examples the router
    /// learns pseudo-random targets from a hash of the position instead, to
    /// break the symmetry between its classes.
    Cluster,
}

/// What an error router learns from one example, from the errors of its
/// children's predictions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum ErrorTarget {
    /// All of it on the child with the smallest error. Over many examples
    /// this favours the child most often closest to the outcome, which is
    /// the most confident one in the commoner outcome, even when it is
    /// overconfident.
    Best,
    /// Each child's share of the probability the children gave the label,
    /// P_k(label) / sum_j P_j(label), from errors that are log-losses
    /// (`ErrorMeasure::Log` or `Winner`): the posterior that the child
    /// explains the outcome, as the gate of a mixture of experts learns it.
    /// For children that predict alike, the expected share is above average
    /// exactly when the child's expected log-loss is below average.
    Soft,
}

/// Which classifier a node uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum NodeKind {
    /// Categorical naive Bayes.
    Cnbp,
    /// Logistic regression.
    Lr,
}

/// The classifier at a node: a leaf's prediction of the classes, or a
/// delegator's router, which predicts which child will classify an input
/// best.
#[derive(Clone, Debug)]
pub enum NodeModel {
    Cnbp(Cnbp),
    Lr(Logreg),
}

impl NodeModel {
    pub fn kind(&self) -> NodeKind {
        match self {
            NodeModel::Cnbp(_) => NodeKind::Cnbp,
            NodeModel::Lr(_) => NodeKind::Lr,
        }
    }

    pub fn class_num(&self) -> usize {
        match self {
            NodeModel::Cnbp(m) => m.class_num(),
            NodeModel::Lr(m) => m.class_num(),
        }
    }

    /// Adds one data point. `label` is a distribution over classes.
    pub fn train(&mut self, data: &[u8], label: &[f64]) {
        match self {
            NodeModel::Cnbp(m) => m.train(data, label),
            NodeModel::Lr(m) => m.train(data, label),
        }
    }

    /// Writes the probability of each class.
    pub fn predict(&self, data: &[u8], out: &mut [f64]) {
        match self {
            NodeModel::Cnbp(m) => m.predict(data, out),
            NodeModel::Lr(m) => m.predict(data, out),
        }
    }

    /// Writes the log of each class's unnormalised probability.
    pub fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        match self {
            NodeModel::Cnbp(m) => m.predict_log(data, out),
            NodeModel::Lr(m) => m.predict_log(data, out),
        }
    }

    /// Prediction during training: a CNBP's is from its raw counts, so one
    /// with `use_probs` need not recalculate its probabilities every step.
    fn predict_training(&self, data: &[u8], out: &mut [f64]) {
        match self {
            NodeModel::Cnbp(m) => m.predict_counts(data, out),
            NodeModel::Lr(m) => m.predict(data, out),
        }
    }

    /// Makes the model ready to predict after training.
    pub fn refresh(&mut self) {
        if let NodeModel::Cnbp(m) = self {
            m.refresh();
        }
    }

    /// Lets the model adapt faster while predicting the same: divides a
    /// CNBP's counts, or a logistic regression's AdaGrad sums, by `factor`.
    pub fn forget(&mut self, factor: f64) {
        match self {
            NodeModel::Cnbp(m) => m.forget(factor),
            NodeModel::Lr(m) => m.forget(factor),
        }
    }

    /// Saves the model on its own, as a CNBP or LR model file.
    pub fn save(&self, path: &std::path::Path) -> io::Result<()> {
        match self {
            NodeModel::Cnbp(m) => m.save(path),
            NodeModel::Lr(m) => m.save(path),
        }
    }

    /// A CNBP as its counts (the C format's node), a logistic regression as
    /// its own text format, which starts with the word "logreg".
    fn write(&self, s: &mut String) {
        match self {
            NodeModel::Cnbp(m) => m.write_counts(s),
            NodeModel::Lr(m) => s.push_str(&m.to_text()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DelegationParams {
    pub class_num: usize,
    pub branch_factor: usize,
    pub split_threshold: u32,
    pub split_limit: u32,
    pub forget_factor: f64,
    pub nb_alpha: f64,
    pub use_probs: bool,
    /// Classifiers of the leaves and of the delegators' routers. Not saved
    /// in model files; when loading, those of the nodes.
    pub leaf: NodeKind,
    pub router: NodeKind,
    /// Base step size of logistic-regression nodes. Not saved in model
    /// files; when loading, that of the nodes.
    pub lr_rate: f64,
    /// Not saved in model files; `Abs` when loading.
    pub error: ErrorMeasure,
    /// A split check compares windows with a margin of this many standard
    /// errors of their difference. Not saved in model files; 0 when loading.
    pub split_z: f64,
    /// Not saved in model files; `Worse` when loading.
    pub split_rule: SplitRule,
    /// Not saved in model files; `Error` when loading.
    pub route: RouteTarget,
    /// What error routers learn, error-routed delegators' and those beside
    /// cluster routers (`route_switch`) alike. `Soft` needs their error
    /// measures (`error`, `trial_error`) to be log-losses. Not saved in model
    /// files; `Best` when loading.
    pub error_target: ErrorTarget,
    /// Every leaf also trains on the examples routed to the other leaves,
    /// weighted by this (0: only on its own), so leaves can specialise
    /// without losing the rest of the data. CNBP leaves only. Not saved in
    /// model files; 0 when loading.
    pub leaf_share: f64,
    /// Test each split on the next `split_threshold` examples through the
    /// split node, against the node without the split (the split leaf, or
    /// the split delegator's router over its old children, forgotten like
    /// the nodes the split made and trained on the same examples): keep the
    /// split only if the mean error (`trial_error`) of its predictions is
    /// below the copy's by this many standard errors, otherwise undo it. The
    /// standard error is per example; the positions of a game are
    /// correlated, so it understates the noise. Undone splits do not count
    /// towards `split_limit`. None: no test. Not saved in model files; None
    /// when loading.
    pub split_trial: Option<f64>,
    /// The error split trials compare, and error routers judge children by;
    /// `Winner` judges by what log-odds play uses. Not saved in model files;
    /// `Winner` when loading.
    pub trial_error: ErrorMeasure,
    /// With cluster routing, each delegator also trains an error router: it
    /// learns which child's subtree predicts an example best by
    /// `trial_error` (from the examples that measure scores and the children
    /// predict differently), and its picks are compared with the cluster
    /// router's on the same examples. After every `split_threshold` examples
    /// through the delegator, if the error router's picks had the lower mean
    /// error by this many standard errors, the delegator switches to error
    /// routing: the error router replaces the cluster router and goes on
    /// learning as before. The standard error is that of the means of 20
    /// consecutive blocks of the window, not of single examples, whose
    /// errors are correlated within a game; training shuffles a round's
    /// positions, so blocks of at least a round's examples hold most of a
    /// game's. Each window is logged; infinity compares without switching.
    /// None: no error routers. Not saved in model files (a switched
    /// delegator saves its error router, but a loaded model trains it as a
    /// cluster router); None when loading.
    pub route_switch: Option<f64>,
}

impl DelegationParams {
    /// An untrained classifier of `kind` over `class_num` classes.
    pub fn new_classifier(&self, kind: NodeKind, class_num: usize, categories: &[u8]) -> NodeModel {
        match kind {
            NodeKind::Cnbp => NodeModel::Cnbp(Cnbp::new(class_num, categories, self.nb_alpha, self.use_probs)),
            NodeKind::Lr => NodeModel::Lr(Logreg::new(class_num, categories, self.lr_rate)),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum Kind {
    Leaf,
    Branch { children: Vec<usize> },
}

/// Sum and count of a stream of errors.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Running {
    sum: f64,
    count: u64,
}

impl Running {
    pub fn add(&mut self, error: f64) {
        self.sum += error;
        self.count += 1;
    }

    /// Mean of the errors added since the total was `start`.
    pub fn mean_since(&self, start: Running) -> f64 {
        (self.sum - start.sum) / (self.count - start.count).max(1) as f64
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Node {
    pub kind: Kind,
    /// Predicts classes for a leaf, or the best child for a delegator.
    pub classifier: NodeModel,
    pub num_trained_total: u32,
    pub num_trained_last: u32,
    pub error_total: f64,
    pub error_last: f64,
    /// Mean error of the previous full window, or -1 before the first.
    pub error_prev: f64,
    /// Sum of squared errors in the current window, and the squared standard
    /// error of the previous window's mean; for `split_z`. Not saved.
    pub error_sq_last: f64,
    pub error_prev_se2: f64,
    /// The model's and the routers' running errors when this node's window
    /// started. Not saved.
    pub window_model: Running,
    pub window_routers: Running,
    /// Output of `classifier` for the position of the current training step;
    /// valid while `cache_step` equals the graph's `step`.
    pub cache: Vec<f64>,
    pub cache_step: u64,
    /// The test of this node's split, while it lasts. Not saved: a model
    /// saved during a test keeps the split.
    pub trial: Option<Box<Trial>>,
    /// A cluster-routed delegator's error router, until it switches (see
    /// `DelegationParams::route_switch`). Not saved.
    pub error_router: Option<Box<ErrorRouter>>,
    /// Whether this delegator switched to error routing. Not saved.
    pub error_routed: bool,
}

/// A split on trial (see `DelegationParams::split_trial`).
#[derive(Clone, Debug)]
pub(crate) struct Trial {
    /// The node without the split: the split leaf, or the split delegator's
    /// router over `children`.
    pub shadow: NodeModel,
    /// The split delegator's children; None for a split leaf.
    pub children: Option<Vec<usize>>,
    /// The split delegator's examples before the split.
    pub trained_before: u32,
    /// The nodes the split made, removed if it is undone.
    pub made: Vec<usize>,
    /// Examples through the node since the split.
    pub examples: u32,
    /// Count, sum and sum of squares of the differences between the errors
    /// of the split node's and the shadow's predictions of each example.
    pub n: u32,
    pub sum: f64,
    pub sum_sq: f64,
}

impl Trial {
    pub fn new(shadow: NodeModel, children: Option<Vec<usize>>, trained_before: u32, made: Vec<usize>) -> Box<Self> {
        Box::new(Trial { shadow, children, trained_before, made, examples: 0, n: 0, sum: 0.0, sum_sq: 0.0 })
    }
}

/// An error router trained beside a cluster router (see
/// `DelegationParams::route_switch`).
#[derive(Clone, Debug)]
pub(crate) struct ErrorRouter {
    pub router: NodeModel,
    pub window: GainWindow,
}

impl ErrorRouter {
    /// A copy with a new window, whose router forgets by `factor`.
    pub fn forgetting_copy(&self, factor: f64) -> Box<Self> {
        let mut router = self.router.clone();
        router.forget(factor);
        Box::new(ErrorRouter { router, window: GainWindow::default() })
    }
}

/// An error router's gains over the current window: per example, the error
/// of the cluster router's pick minus that of the error router's (0 for
/// examples that do not train the error router).
#[derive(Clone, Debug, Default)]
pub(crate) struct GainWindow {
    /// Examples through the delegator; of those, the ones that trained the
    /// error router, and the ones of those the two routers picked different
    /// children for.
    pub examples: u32,
    pub judged: u32,
    pub differ: u32,
    /// Sum and sum of squares of the gains.
    pub sum: f64,
    pub sum_sq: f64,
    /// Sum of the gains of the current block; count, sum and sum of squares
    /// of the mean gains of the finished blocks.
    pub block: f64,
    pub blocks: u32,
    pub block_sum: f64,
    pub block_sum_sq: f64,
}

impl GainWindow {
    /// Adds an example's gain, finishing a block every `block_len` examples.
    fn add(&mut self, gain: f64, block_len: u32) {
        self.examples += 1;
        self.sum += gain;
        self.sum_sq += gain * gain;
        self.block += gain;
        if self.examples % block_len == 0 {
            let mean = self.block / block_len as f64;
            self.blocks += 1;
            self.block_sum += mean;
            self.block_sum_sq += mean * mean;
            self.block = 0.0;
        }
    }
}

impl Node {
    fn new(kind: Kind, classifier: NodeModel) -> Self {
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
            window_model: Running::default(),
            window_routers: Running::default(),
            trial: None,
            error_router: None,
            error_routed: false,
        }
    }

    pub fn record(&mut self, error: f64) {
        self.num_trained_last += 1;
        self.num_trained_total += 1;
        self.error_last += error;
        self.error_total += error;
        self.error_sq_last += error * error;
    }

    /// Ends the current window of a split check, making its mean error the
    /// previous window's. Returns this window's mean error, the previous
    /// window's (-1 before the first), and `z` standard errors of their
    /// difference.
    pub fn close_window(&mut self, z: f64) -> (f64, f64, f64) {
        let n = self.num_trained_last as f64;
        let recent = self.error_last / n;
        let se2 = (self.error_sq_last / n - recent * recent).max(0.0) / n;
        let (previous, previous_se2) = (self.error_prev, self.error_prev_se2);
        self.error_prev = recent;
        self.error_prev_se2 = se2;
        (recent, previous, z * (se2 + previous_se2).sqrt())
    }

    /// Forgets the previous window, so the next one cannot split the node.
    pub fn forget_window(&mut self) {
        self.error_prev = -1.0;
        self.error_prev_se2 = 0.0;
    }
}

/// The nodes of a delegation model; node 0 is the root.
#[derive(Clone, Debug)]
pub(crate) struct Graph {
    pub categories: Vec<u8>,
    pub params: DelegationParams,
    pub split_number: u32,
    pub nodes: Vec<Node>,
    /// Counts training steps, starting at 1; a `cache_step` of 0 marks no cache.
    pub step: u64,
    /// Errors of the whole model's predictions, and of every router's
    /// choice, while training; for the split rules. Not saved.
    pub model_errors: Running,
    pub router_errors: Running,
    /// Nodes of undone splits, removed at the start of the next step or by
    /// `refresh` (so node indices stay valid within a step).
    pub dead: Vec<usize>,
}

impl Graph {
    pub fn new(categories: &[u8], params: DelegationParams) -> Self {
        assert!(params.split_threshold > 0, "split_threshold must be at least 1");
        assert!(params.branch_factor >= 2, "branch_factor must be at least 2");
        if params.error_target == ErrorTarget::Soft {
            let log_loss = |m: ErrorMeasure| m != ErrorMeasure::Abs;
            assert!(
                (params.route != RouteTarget::Error || log_loss(params.error))
                    && (params.route_switch.is_none() || log_loss(params.trial_error)),
                "soft error targets need error routers that judge by a log-loss (log or winner)"
            );
        }
        let root = Node::new(Kind::Leaf, params.new_classifier(params.leaf, params.class_num, categories));
        Graph {
            categories: categories.to_vec(),
            params,
            split_number: 0,
            nodes: vec![root],
            step: 0,
            model_errors: Running::default(),
            router_errors: Running::default(),
            dead: Vec::new(),
        }
    }

    /// A graph whose root leaf starts as `root`, a classifier over the same
    /// features and classes, instead of an untrained one.
    pub fn starting_from(root: NodeModel, categories: &[u8], params: DelegationParams) -> Self {
        let mut graph = Graph::new(categories, params);
        graph.nodes[0].classifier = root;
        graph
    }

    /// Writes the probability of each class into `out`.
    ///
    /// # Panics
    /// With `use_probs`, if the model was trained since the last `refresh()`.
    pub fn predict(&self, data: &[u8], out: &mut [f64]) {
        self.nodes[self.leaf(data)].classifier.predict(data, out)
    }

    /// Writes the log of each class's unnormalised probability, from the leaf
    /// `predict` would use (see `Cnbp::predict_log`).
    pub fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        self.nodes[self.leaf(data)].classifier.predict_log(data, out)
    }

    /// The leaves' node indices and classifiers.
    pub fn leaves(&self) -> impl Iterator<Item = (usize, &NodeModel)> {
        self.nodes.iter().enumerate().filter(|(_, n)| matches!(n.kind, Kind::Leaf)).map(|(i, n)| (i, &n.classifier))
    }

    /// Writes node `idx`'s own prediction (see `NodeModel::predict_log`).
    pub fn node_predict_log(&self, idx: usize, data: &[u8], out: &mut [f64]) {
        self.nodes[idx].classifier.predict_log(data, out)
    }

    /// Each leaf's weight for blending the leaves' predictions instead of
    /// picking one: the product along its path of each router's
    /// probabilities, softened by multiplying the router's log-probabilities
    /// by `tau` (1: the router's own, 0: every child alike). Paths whose
    /// weight falls below `min_weight` are dropped, so the weights may sum to
    /// less than 1. A leaf reached by several paths (BDD) appears once per
    /// path.
    pub fn leaf_weights(&self, data: &[u8], tau: f64, min_weight: f64, out: &mut Vec<(usize, f64)>) {
        out.clear();
        self.add_leaf_weights(0, 1.0, data, tau, min_weight, out);
    }

    fn add_leaf_weights(&self, idx: usize, weight: f64, data: &[u8], tau: f64, min_weight: f64, out: &mut Vec<(usize, f64)>) {
        let node = &self.nodes[idx];
        let Kind::Branch { children } = &node.kind else {
            return out.push((idx, weight));
        };
        let mut logs = [0.0; 256];
        let logs = &mut logs[..children.len()];
        node.classifier.predict_log(data, logs);
        let max = logs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        logs.iter_mut().for_each(|l| *l = (tau * (*l - max)).exp());
        let sum: f64 = logs.iter().sum();
        for (&child, &p) in children.iter().zip(logs.iter()) {
            let w = weight * p / sum;
            if w >= min_weight {
                self.add_leaf_weights(child, w, data, tau, min_weight, out);
            }
        }
    }

    /// Index of the leaf that predicts the position.
    pub fn leaf(&self, data: &[u8]) -> usize {
        let mut idx = 0;
        loop {
            let node = &self.nodes[idx];
            match &node.kind {
                Kind::Leaf => return idx,
                Kind::Branch { children } => idx = children[best_child(&node.classifier, data)],
            }
        }
    }

    /// Makes the nodes ready to predict after training (see `Cnbp::refresh`).
    pub fn refresh(&mut self) {
        self.remove_dead();
        for node in &mut self.nodes {
            node.classifier.refresh();
        }
    }

    /// Prediction used during training: CNBP nodes always predict from
    /// their raw counts, so models with `use_probs` do not recalculate
    /// probabilities after every step.
    pub fn predict_training(&mut self, mut idx: usize, data: &[u8], out: &mut [f64]) {
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

    /// Runs a node's own classifier as in training. The output is cached for
    /// the rest of the step: the position does not change within a step, and
    /// a classifier is only trained after the predictions that use it.
    pub fn node_output(&mut self, idx: usize, data: &[u8], out: &mut [f64]) {
        let step = self.step;
        let node = &mut self.nodes[idx];
        if node.cache_step != step {
            node.classifier.predict_training(data, &mut node.cache);
            node.cache_step = step;
        }
        out.copy_from_slice(&node.cache);
    }

    /// Starts a training step. Returns the error of the model's prediction
    /// for the example, or None if it is not scored (see `ErrorMeasure`).
    pub fn begin_step(&mut self, data: &[u8], label: &[f64], scratch: &mut [f64]) -> Option<f64> {
        self.remove_dead();
        self.step += 1;
        self.predict_training(0, data, scratch);
        let error = self.scored(label).then(|| self.error(scratch, label));
        if let Some(error) = error {
            self.model_errors.add(error);
        }
        error
    }

    /// Checks whether leaf `idx` should split once it has seen
    /// `split_threshold` examples since its last check: then splits it if
    /// the split rule says so and starts a new window. Returns None if no
    /// check was due, otherwise whether it split.
    pub fn check_leaf_split(&mut self, idx: usize) -> Option<bool> {
        let node = &mut self.nodes[idx];
        if node.num_trained_last < self.params.split_threshold || self.split_number >= self.params.split_limit {
            return None;
        }
        // Split when the leaf stopped improving: this window's mean error is
        // above the previous window's. Comparing with the lifetime mean
        // instead is skewed by the high error of the first data.
        let (recent, previous, margin) = node.close_window(self.params.split_z);
        let split = previous >= 0.0
            && match self.params.split_rule {
                SplitRule::Worse => recent > previous + margin,
                SplitRule::StallHigh => {
                    previous - recent < margin && at_least(recent, self.model_errors.mean_since(node.window_model))
                }
            };
        if split {
            self.split_leaf(idx);
        }
        self.open_window(idx);
        Some(split)
    }

    pub fn train_leaf(&mut self, idx: usize, data: &[u8], label: &[f64], error: Option<f64>) {
        let node = &mut self.nodes[idx];
        node.classifier.train(data, label);
        node.cache_step = 0;
        if let Some(error) = error {
            node.record(error);
        }
        if self.params.leaf_share > 0.0 {
            for (i, node) in self.nodes.iter_mut().enumerate() {
                if i != idx && matches!(node.kind, Kind::Leaf) {
                    let NodeModel::Cnbp(cnbp) = &mut node.classifier else { panic!("leaf_share needs CNBP leaves") };
                    cnbp.train_weighted(data, label, self.params.leaf_share);
                    node.cache_step = 0;
                }
            }
        }
    }

    /// The child the router of `idx` picks now; for examples that do not
    /// train routers.
    pub fn route(&mut self, idx: usize, data: &[u8]) -> usize {
        let mut router = [0.0; 256];
        let router = &mut router[..self.params.branch_factor];
        self.node_output(idx, data, router);
        argmax(router)
    }

    /// Errors of the predictions of each child's subtree.
    pub fn prediction_errors(&mut self, children: &[usize], data: &[u8], label: &[f64], scratch: &mut [f64]) -> Vec<f64> {
        children
            .iter()
            .map(|&child| {
                self.predict_training(child, data, scratch);
                self.error(scratch, label)
            })
            .collect()
    }

    /// Trains the router of `idx` on its target (see `RouteTarget`; with
    /// `Error`, the `error_target` of `child_errors`; other targets ignore
    /// them). Returns the child the updated router picks from the position
    /// alone, as in prediction, so each child learns about the positions it
    /// will be asked about; and the router's error before training, like a
    /// leaf's. Call `record_router` with it after training the child.
    pub fn train_router(&mut self, idx: usize, data: &[u8], child_errors: &[f64]) -> (usize, f64) {
        let target = match self.params.route {
            RouteTarget::Error => self.error_target(child_errors),
            RouteTarget::Cluster => one_hot(self.cluster_pick(idx, data), self.params.branch_factor),
        };
        self.train_router_on(idx, data, &target)
    }

    /// What an error router learns from an example whose children's
    /// predictions have these errors (see `ErrorTarget`; `Best` takes the
    /// first child on ties, as in C).
    pub fn error_target(&self, errors: &[f64]) -> Vec<f64> {
        match self.params.error_target {
            ErrorTarget::Best => one_hot(argmin(errors), self.params.branch_factor),
            ErrorTarget::Soft => {
                let least = errors.iter().copied().fold(f64::INFINITY, f64::min);
                let mut target: Vec<f64> = errors.iter().map(|e| (least - e).exp()).collect();
                let sum: f64 = target.iter().sum();
                target.iter_mut().for_each(|t| *t /= sum);
                target
            }
        }
    }

    /// A cluster router's target: its own pick, or for its first examples a
    /// hash of the position (see `RouteTarget::Cluster`).
    fn cluster_pick(&mut self, idx: usize, data: &[u8]) -> usize {
        if self.nodes[idx].num_trained_total < self.params.split_threshold / 5 {
            (position_hash(data) % self.params.branch_factor as u64) as usize
        } else {
            self.route(idx, data)
        }
    }

    /// Trains the router of `idx` on `target`; returns as `train_router`.
    fn train_router_on(&mut self, idx: usize, data: &[u8], target: &[f64]) -> (usize, f64) {
        let mut router = [0.0; 256];
        let router = &mut router[..self.params.branch_factor];
        self.node_output(idx, data, router);
        let router_error = self.router_error(router, target);
        self.nodes[idx].classifier.train(data, target);
        self.nodes[idx].cache_step = 0;
        (self.route(idx, data), router_error)
    }

    /// Trains the router of cluster-routed delegator `idx`, whose children
    /// are `children`, on a scored example: as `train_router`, plus its
    /// error router (see `DelegationParams::route_switch`), or as an error
    /// router once it switched. Returns as `train_router`.
    pub fn train_cluster_router(
        &mut self,
        idx: usize,
        children: &[usize],
        data: &[u8],
        label: &[f64],
        scratch: &mut [f64],
    ) -> (usize, f64) {
        let (switched, comparing) = (self.nodes[idx].error_routed, self.nodes[idx].error_router.is_some());
        if !switched && !comparing {
            return self.train_router(idx, data, &[]);
        }
        let errors = self.judged_errors(children, data, label, scratch);
        if switched {
            let Some(errors) = errors else {
                // Nothing to learn; the router's error is that of its own pick.
                let routed = self.route(idx, data);
                let target = one_hot(routed, self.params.branch_factor);
                let mut router = [0.0; 256];
                let router = &mut router[..self.params.branch_factor];
                self.node_output(idx, data, router);
                return (routed, self.router_error(router, &target));
            };
            let target = self.error_target(&errors);
            return self.train_router_on(idx, data, &target);
        }
        let target = one_hot(self.cluster_pick(idx, data), self.params.branch_factor);
        self.compare_routers(idx, data, errors.as_deref());
        let trained = self.train_router_on(idx, data, &target);
        if self.nodes[idx].error_router.as_ref().is_some_and(|e| e.window.examples >= self.params.split_threshold) {
            self.end_router_window(idx);
        }
        trained
    }

    /// Errors by `trial_error` of the predictions of each child's subtree,
    /// if that measure scores the example and the children differ on it.
    fn judged_errors(&mut self, children: &[usize], data: &[u8], label: &[f64], scratch: &mut [f64]) -> Option<Vec<f64>> {
        let judge = self.params.trial_error;
        if judge == ErrorMeasure::Winner && label[0] + label[1] == 0.0 {
            return None;
        }
        let errors: Vec<f64> = children
            .iter()
            .map(|&child| {
                self.predict_training(child, data, scratch);
                measure(judge, scratch, label)
            })
            .collect();
        errors.iter().any(|&e| e != errors[0]).then_some(errors)
    }

    /// Adds an example to the window of delegator `idx`'s comparison of its
    /// routers, before either learns from it, and trains the error router.
    /// `errors` are the children's (see `judged_errors`).
    fn compare_routers(&mut self, idx: usize, data: &[u8], errors: Option<&[f64]>) {
        let mut error_router = self.nodes[idx].error_router.take().expect("no error router to compare");
        let mut gain = 0.0;
        if let Some(errors) = errors {
            let mut scores = [0.0; 256];
            let scores = &mut scores[..self.params.branch_factor];
            error_router.router.predict_training(data, scores);
            let (cluster_pick, error_pick) = (self.route(idx, data), argmax(scores));
            gain = errors[cluster_pick] - errors[error_pick];
            error_router.window.judged += 1;
            error_router.window.differ += (cluster_pick != error_pick) as u32;
            error_router.router.train(data, &self.error_target(errors));
        }
        error_router.window.add(gain, (self.params.split_threshold / 20).max(1));
        self.nodes[idx].error_router = Some(error_router);
    }

    /// Ends the window of delegator `idx`'s comparison of its routers: logs
    /// it, and switches the delegator to error routing if the error router's
    /// mean gain was above `route_switch` standard errors.
    fn end_router_window(&mut self, idx: usize) {
        let error_router = self.nodes[idx].error_router.as_mut().expect("a window without an error router");
        let w = std::mem::take(&mut error_router.window);
        let blocks = w.blocks as f64;
        let mean = w.block_sum / blocks;
        let se = ((w.block_sum_sq / blocks - mean * mean).max(0.0) / (blocks - 1.0)).sqrt();
        let n = w.examples as f64;
        let example_se = ((w.sum_sq / n - (w.sum / n).powi(2)).max(0.0) / n).sqrt();
        let switch = mean > self.params.route_switch.unwrap_or(f64::INFINITY) * se;
        eprintln!(
            "routers of delegator {idx}: error router's gain {mean:+.7} per example (standard error {se:.7} over {} blocks, {example_se:.7} per example), picks differ on {} of {} judged examples{}",
            w.blocks,
            w.differ,
            w.judged,
            if switch { "; switched to error routing" } else { "" }
        );
        if switch {
            let error_router = self.nodes[idx].error_router.take().unwrap();
            let node = &mut self.nodes[idx];
            node.classifier = error_router.router;
            node.cache_step = 0;
            node.error_routed = true;
        }
    }

    /// The error router of a new cluster-routed delegator, if any (see
    /// `DelegationParams::route_switch`).
    pub fn new_error_router(&self) -> Option<Box<ErrorRouter>> {
        let wanted = self.params.route == RouteTarget::Cluster && self.params.route_switch.is_some();
        wanted.then(|| Box::new(ErrorRouter { router: self.new_router(), window: GainWindow::default() }))
    }

    pub fn record_router(&mut self, idx: usize, router_error: f64) {
        self.nodes[idx].record(router_error);
        self.router_errors.add(router_error);
    }

    /// Turns leaf `idx` into a delegator over `branch_factor` new leaves,
    /// copies of it that forget by `forget_factor` (see `NodeModel::forget`),
    /// so they predict the same but adapt faster. With `split_trial`, one
    /// more such copy stands for the leaf without the split until the split's
    /// trial ends (see `trial_step`).
    pub fn split_leaf(&mut self, idx: usize) {
        let first = self.nodes.len();
        for _ in 0..self.params.branch_factor {
            let mut classifier = self.nodes[idx].classifier.clone();
            classifier.forget(self.params.forget_factor);
            let child = self.new_node(Kind::Leaf, classifier);
            self.nodes.push(child);
        }
        let made: Vec<usize> = (first..first + self.params.branch_factor).collect();
        let trial = self.params.split_trial.map(|_| {
            let mut shadow = self.nodes[idx].classifier.clone();
            shadow.forget(self.params.forget_factor);
            Trial::new(shadow, None, 0, made.clone())
        });

        let router = self.new_router();
        let error_router = self.new_error_router();
        let node = &mut self.nodes[idx];
        node.kind = Kind::Branch { children: made };
        node.classifier = router;
        node.cache = vec![0.0; self.params.branch_factor];
        node.cache_step = 0;
        node.num_trained_total = 0;
        node.error_total = 0.0;
        node.trial = trial;
        node.error_router = error_router;
        node.error_routed = false;
        self.split_number += 1;
    }

    /// For a node whose split is on trial (see `DelegationParams::
    /// split_trial`), at the start of its part of a training step: ends the
    /// trial once it has seen `split_threshold` examples, keeping or undoing
    /// the split; otherwise scores the example's prediction against the
    /// shadow's and trains the shadow as the node would have been trained.
    /// Returns whether the split was undone, which changes the node.
    pub fn trial_step(&mut self, idx: usize, data: &[u8], label: &[f64], scratch: &mut [f64]) -> bool {
        let Some(mut trial) = self.nodes[idx].trial.take() else { return false };
        if trial.examples >= self.params.split_threshold {
            return self.end_trial(idx, *trial);
        }
        trial.examples += 1;
        let judge = self.params.trial_error;
        let mut scores = [0.0; 256];
        let scores = &mut scores[..self.params.branch_factor];
        if judge != ErrorMeasure::Winner || label[0] + label[1] > 0.0 {
            self.predict_training(idx, data, scratch);
            let split_error = measure(judge, scratch, label);
            let shadow_error = match &trial.children {
                None => {
                    trial.shadow.predict_training(data, scratch);
                    measure(judge, scratch, label)
                }
                Some(children) => {
                    trial.shadow.predict_training(data, scores);
                    self.predict_training(children[argmax(scores)], data, scratch);
                    measure(judge, scratch, label)
                }
            };
            let d = split_error - shadow_error;
            trial.n += 1;
            trial.sum += d;
            trial.sum_sq += d * d;
        }
        match &trial.children {
            None => trial.shadow.train(data, label),
            Some(children) if self.scored(label) => {
                trial.shadow.predict_training(data, scores);
                // As `train_router`, for a router past its first examples.
                let target = match self.params.route {
                    RouteTarget::Error => {
                        let errors = self.prediction_errors(children, data, label, scratch);
                        self.error_target(&errors)
                    }
                    RouteTarget::Cluster => one_hot(argmax(scores), self.params.branch_factor),
                };
                trial.shadow.train(data, &target);
            }
            Some(_) => {}
        }
        self.nodes[idx].trial = Some(trial);
        false
    }

    /// Ends the trial of node `idx`'s split: keeps the split if its mean
    /// error was below the shadow's by `split_trial` standard errors,
    /// otherwise puts the shadow back in its place. Returns whether the split
    /// was undone.
    fn end_trial(&mut self, idx: usize, trial: Trial) -> bool {
        let n = trial.n as f64;
        let mean = trial.sum / n;
        let se = ((trial.sum_sq / n - mean * mean).max(0.0) / n).sqrt();
        let keep = mean < -self.params.split_trial.unwrap_or(0.0) * se;
        let kind = if trial.children.is_some() { "delegator" } else { "leaf" };
        let outcome = if keep { "kept" } else { "undone" };
        eprintln!("split trial of {kind} {idx}: {outcome}, error vs unsplit {mean:+.7} (standard error {se:.7}, {} of {} examples)", trial.n, trial.examples);
        if keep {
            return false;
        }
        self.dead.extend(&trial.made);
        let (class_num, branch_factor) = (self.params.class_num, self.params.branch_factor);
        // The delegators a delegator split made kept its routing mode and
        // error router.
        let made = &mut self.nodes[trial.made[0]];
        let (error_router, error_routed) = (made.error_router.take(), made.error_routed);
        let node = &mut self.nodes[idx];
        match trial.children {
            None => {
                node.kind = Kind::Leaf;
                node.cache = vec![0.0; class_num];
                node.num_trained_total = trial.examples;
                node.error_router = None;
                node.error_routed = false;
            }
            Some(children) => {
                node.kind = Kind::Branch { children };
                node.cache = vec![0.0; branch_factor];
                node.num_trained_total = trial.trained_before + trial.examples;
                node.error_router = error_router;
                node.error_routed = error_routed;
            }
        }
        node.classifier = trial.shadow;
        node.cache_step = 0;
        node.error_total = 0.0;
        node.forget_window();
        self.open_window(idx);
        self.split_number -= 1;
        true
    }

    /// Removes the nodes of undone splits (`dead`), renumbering the rest.
    fn remove_dead(&mut self) {
        if self.dead.is_empty() {
            return;
        }
        let mut new_index = vec![usize::MAX; self.nodes.len()];
        let mut next = 0;
        for (i, slot) in new_index.iter_mut().enumerate() {
            if !self.dead.contains(&i) {
                *slot = next;
                next += 1;
            }
        }
        let mut i = 0;
        self.nodes.retain(|_| {
            i += 1;
            new_index[i - 1] != usize::MAX
        });
        let renumber = |indices: &mut Vec<usize>| {
            for i in indices.iter_mut() {
                *i = new_index[*i];
                assert!(*i != usize::MAX, "a live node refers to a removed one");
            }
        };
        for node in &mut self.nodes {
            if let Kind::Branch { children } = &mut node.kind {
                renumber(children);
            }
            if let Some(trial) = &mut node.trial {
                renumber(&mut trial.made);
                if let Some(children) = &mut trial.children {
                    renumber(children);
                }
            }
        }
        self.dead.clear();
    }

    /// Starts a new window of examples for the node's split check.
    pub fn open_window(&mut self, idx: usize) {
        let (model, routers) = (self.model_errors, self.router_errors);
        let node = &mut self.nodes[idx];
        node.num_trained_last = 0;
        node.error_last = 0.0;
        node.error_sq_last = 0.0;
        node.window_model = model;
        node.window_routers = routers;
    }

    /// A new node whose first window starts now.
    pub fn new_node(&self, kind: Kind, classifier: NodeModel) -> Node {
        Node { window_model: self.model_errors, window_routers: self.router_errors, ..Node::new(kind, classifier) }
    }

    /// A router over `branch_factor` children that knows nothing yet.
    pub fn new_router(&self) -> NodeModel {
        self.params.new_classifier(self.params.router, self.params.branch_factor, &self.categories)
    }

    pub fn error(&self, predicted: &[f64], actual: &[f64]) -> f64 {
        measure(self.params.error, predicted, actual)
    }

    /// Error of a router's prediction of the best child.
    pub fn router_error(&self, predicted: &[f64], target: &[f64]) -> f64 {
        match self.params.error {
            ErrorMeasure::Abs => prediction_error(predicted, target),
            ErrorMeasure::Log | ErrorMeasure::Winner => log_loss(predicted, target),
        }
    }

    /// Whether an example with this label has an error (see `ErrorMeasure`).
    pub fn scored(&self, label: &[f64]) -> bool {
        self.params.error != ErrorMeasure::Winner || label[0] + label[1] > 0.0
    }

    /// Serializes in the C text format (`bdt_to_file`), with logistic
    /// regression nodes in their own format. Children may be shared (BDD).
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
            node.classifier.write(&mut s);
        }
        s
    }

    /// Parses the C text format (`bdt_from_file`), with logistic regression
    /// nodes in their own format. The node ids stored in the file must match
    /// their positions, as the C code always writes them.
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
            let classifier = if t.peek() == Some("logreg") {
                let lr = Logreg::from_tokens(&mut t)?;
                if lr.class_num() != classes {
                    return Err(invalid(format!("node {n} has {} classes, expected {classes}", lr.class_num())));
                }
                NodeModel::Lr(lr)
            } else {
                let mut cnbp = Cnbp::new(classes, &categories, nb_alpha, use_probs);
                cnbp.read_counts(&mut t)?;
                NodeModel::Cnbp(cnbp)
            };
            nodes.push(Node {
                num_trained_total,
                num_trained_last,
                error_total,
                error_last,
                error_prev,
                ..Node::new(kind, classifier)
            });
        }

        // Routers are of the leaves' kind until the first split.
        let kind_of = |leaf: bool| {
            nodes.iter().find(|n| matches!(n.kind, Kind::Leaf) == leaf).map(|n| n.classifier.kind())
        };
        let leaf = kind_of(true).unwrap_or(NodeKind::Cnbp);
        let router = kind_of(false).unwrap_or(leaf);
        let lr_rate = nodes
            .iter()
            .find_map(|n| match &n.classifier {
                NodeModel::Lr(m) => Some(m.rate()),
                NodeModel::Cnbp(_) => None,
            })
            .unwrap_or(0.05);
        let params = DelegationParams {
            class_num,
            branch_factor,
            split_threshold,
            split_limit,
            forget_factor,
            nb_alpha,
            use_probs,
            leaf,
            router,
            lr_rate,
            error: ErrorMeasure::Abs,
            split_z: 0.0,
            split_rule: SplitRule::Worse,
            route: RouteTarget::Error,
            error_target: ErrorTarget::Best,
            leaf_share: 0.0,
            split_trial: None,
            trial_error: ErrorMeasure::Winner,
            route_switch: None,
        };
        Ok(Graph {
            categories,
            params,
            split_number,
            nodes,
            step: 0,
            model_errors: Running::default(),
            router_errors: Running::default(),
            dead: Vec::new(),
        })
    }
}

/// Whether a window's mean error is at least `mean`, the mean over the same
/// window of a running total. The two are computed differently, so a node
/// that saw every example of the window (the only leaf or router) would
/// otherwise be above or below its own mean by rounding alone.
pub(crate) fn at_least(recent: f64, mean: f64) -> bool {
    recent >= mean - 1e-6 * mean.abs()
}

/// Index of the child a router rates highest.
fn best_child(router: &NodeModel, data: &[u8]) -> usize {
    let mut scores = [0.0; 256];
    let scores = &mut scores[..router.class_num()];
    router.predict(data, scores);
    argmax(scores)
}

/// Index of the largest value, the first one on ties (as in C).
pub(crate) fn argmax(values: &[f64]) -> usize {
    let mut best = 0;
    for (i, &v) in values.iter().enumerate() {
        if v > values[best] {
            best = i;
        }
    }
    best
}

/// Index of the smallest value, the first one on ties (as in C).
pub(crate) fn argmin(values: &[f64]) -> usize {
    let mut best = 0;
    for (i, &v) in values.iter().enumerate() {
        if v < values[best] {
            best = i;
        }
    }
    best
}

/// `len` values, 1 at `index` and 0 elsewhere.
pub(crate) fn one_hot(index: usize, len: usize) -> Vec<f64> {
    let mut v = vec![0.0; len];
    v[index] = 1.0;
    v
}

/// A well-mixed 64-bit hash of an encoded position (FNV-1a, then the
/// splitmix64 finaliser), the same on every run.
fn position_hash(data: &[u8]) -> u64 {
    let mut h = data.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &b| (h ^ b as u64).wrapping_mul(0x0100_0000_01b3));
    h = (h ^ (h >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    h = (h ^ (h >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    h ^ (h >> 31)
}

/// The error of a prediction by `kind`.
fn measure(kind: ErrorMeasure, predicted: &[f64], actual: &[f64]) -> f64 {
    match kind {
        ErrorMeasure::Abs => prediction_error(predicted, actual),
        ErrorMeasure::Log => log_loss(predicted, actual),
        ErrorMeasure::Winner => winner_log_loss(predicted, actual),
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

/// -ln(P(winner) / (P(white win) + P(black win))) for classes [white win,
/// black win, draw], averaged over the decisive part of the label.
pub fn winner_log_loss(predicted: &[f64], actual: &[f64]) -> f64 {
    let decisive = predicted[0] + predicted[1];
    let loss = |p: f64| -(p / decisive.max(f64::MIN_POSITIVE)).max(f64::MIN_POSITIVE).ln();
    (actual[0] * loss(predicted[0]) + actual[1] * loss(predicted[1])) / (actual[0] + actual[1])
}

/// Total variation distance between a prediction and a label.
fn prediction_error(expected: &[f64], actual: &[f64]) -> f64 {
    let mut error = 0.0;
    for (e, a) in expected.iter().zip(actual) {
        error += (e - a).abs();
    }
    error / 2.0
}
