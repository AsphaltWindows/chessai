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

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io;

use crate::cnbp::Cnbp;
use crate::logreg::Logreg;
use crate::state::{Reader, Writer};
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

/// What the shadow test (see `DelegationParams::revise`) scores the branch
/// and the shadow on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum ReviseOn {
    /// The training stream: each example as training reaches the delegator,
    /// before the branch and the shadow train on it.
    #[default]
    Stream,
    /// Fresh rounds: each self-play round's positions, in training order,
    /// scored by the model as it starts the round, before it trains on any
    /// of them (see `Graph::observe_round`).
    Fresh,
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
    pub(crate) fn write(&self, s: &mut String) {
        match self {
            NodeModel::Cnbp(m) => m.write_counts(s),
            NodeModel::Lr(m) => s.push_str(&m.to_text()),
        }
    }

    /// The kind and settings, which a resumed run's options must match.
    pub(crate) fn options(&self) -> String {
        match self {
            NodeModel::Cnbp(m) => format!("cnbp alpha {} use_probs {}", m.alpha(), m.use_probs()),
            NodeModel::Lr(m) => format!("lr rate {}", m.rate()),
        }
    }

    /// Writes the classifier, unrounded, for a state file (see `state`).
    pub(crate) fn write_state(&self, w: &mut Writer) {
        match self {
            NodeModel::Cnbp(m) => {
                w.u8(0);
                m.write_state(w)
            }
            NodeModel::Lr(m) => {
                w.u8(1);
                m.write_state(w)
            }
        }
    }

    /// Reads a classifier written by `write_state`.
    pub(crate) fn read_state(r: &mut Reader) -> io::Result<Self> {
        match r.u8()? {
            0 => Ok(NodeModel::Cnbp(Cnbp::read_state(r)?)),
            1 => Ok(NodeModel::Lr(Logreg::read_state(r)?)),
            k => Err(invalid(format!("unknown classifier {k} in state file"))),
        }
    }
}

/// `Debug` (written by hand, see below) is how a state file records the
/// options it was trained with, which a resumed run must match.
#[derive(Clone)]
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
    /// Classifier of the error routers beside cluster routers
    /// (`route_switch`); a delegator that switches keeps its error router as
    /// its router. Not saved in model files; when loading, the routers'.
    pub error_router: NodeKind,
    /// Revisable structure: every delegator keeps a shadow, the leaf it was
    /// before it split (forgotten like its children), which trains on every
    /// example through the delegator and predicts it beside the branch.
    /// After every `split_threshold` examples through the delegator, the
    /// branch's mean gain over the shadow (in `trial_error`; with the
    /// standard error of 20 blocks, as for `route_switch`), less
    /// `seam_weight` times its seams, is compared with this many standard
    /// errors below 0: if lower, the shadow takes the branch's place as a
    /// leaf, which may split again, and the branch's splits no longer count
    /// towards `split_limit`. A delegator that switches to error routing
    /// (`route_switch`) starts its windows afresh, so the branch is judged
    /// only on error-routed examples; the shadow goes on as before.
    /// Infinity compares and logs only; None: no shadows unless
    /// `reset_leaves` needs them. BDT only. Neither this nor the shadows are
    /// saved in model files; None when loading.
    pub revise: Option<f64>,
    /// The weight of a branch's seams in `revise`'s test. A seam is the
    /// difference in white's log-odds (nats) of the position after a move
    /// between the leaf it is in and the leaf of the position before, for
    /// moves between two of the branch's leaves; their absolute values are
    /// summed over the games of the window's rounds (measured before
    /// training on them, see `Graph::measure_seams`) and divided by the
    /// positions through the delegator. Not saved in model files; 0 when
    /// loading.
    pub seam_weight: f64,
    /// With shadows (see `revise`): reset a leaf to a copy of its
    /// delegator's shadow (forgotten like a split's children) when, over a
    /// window of `split_threshold` examples the delegator routes to it, its
    /// mean gain over the shadow is below this many standard errors below 0.
    /// Infinity compares and logs only. None: no resets. Not saved in model
    /// files; None when loading.
    pub reset_leaves: Option<f64>,
    /// Residual leaves (hierarchical shrinkage): a leaf below a delegator
    /// holds only a correction, its own weights (the delta), on top of the
    /// delegator's shadow, which goes on learning from every example through
    /// the delegator. It predicts softmax(shadow logits + delta logits), one
    /// logistic regression whose weights are the sums, so each position is
    /// still predicted by one leaf; the leaf's classifier holds the sums,
    /// kept up to date as either part trains. A split's children start with
    /// every weight of the delta 0 and no AdaGrad history (but see
    /// `residual_inherit_sums`); a leaf trains its
    /// delta by an AdaGrad step on the log-loss of its own (summed)
    /// prediction, the shadow's logits held fixed; the shadow trains on its
    /// own loss, before the leaf in the step and after the children's
    /// predictions of the example. A residual leaf that splits becomes a
    /// delegator whose shadow is its summed model (AdaGrad sums: the shadow's
    /// plus the delta's), forgotten as usual. A merged branch's delegator
    /// becomes a residual leaf again if it has a delegator above it: its
    /// delta is its shadow minus that delegator's, with its shadow's AdaGrad
    /// sums. A reset leaf (`reset_leaves`) gets a new delta. Needs
    /// logistic-regression leaves and shadows (`revise` or `reset_leaves`).
    /// Model files hold the summed leaves, so they load and play as plain
    /// trees; state files hold the deltas. BDT only. False when loading.
    pub leaf_residual: bool,
    /// With `leaf_residual`: an L2 penalty of this / 2 times the square of
    /// each delta weight an example uses, which shrinks the deltas towards 0
    /// (the leaf towards its delegator's shadow). 0 when loading.
    pub residual_l2: f64,
    /// With `leaf_residual`: a new delta (a split's children, a reset leaf)
    /// starts with its shadow's AdaGrad sums instead of none, so the leaf
    /// steps as a copy of the leaf it replaces would (see `split_leaf`),
    /// rather than at the full rate of a model that has seen nothing; and a
    /// residual leaf's AdaGrad sums are its delta's alone, as a copy's would
    /// be (the shadow's sums when the delta started, plus its own steps).
    /// Without it, the deltas' first steps are AdaGrad's largest. False
    /// when loading.
    pub residual_inherit_sums: bool,
    /// With shadows, in self-play or train: logs per round and delegator the shadow
    /// test's gain on the round's positions scored by the model as it starts
    /// the round (fresh), beside the gain training scores on the same round
    /// (the stream), each with a standard error clustered by game (see
    /// `Graph::observe_round`, `Graph::end_round`). Log only: training is
    /// unchanged, and it is not recorded in state files. False when loading.
    pub log_fresh_gains: bool,
    /// What the shadow test scores (see `ReviseOn`); its windows and
    /// thresholds are the same either way. Fresh needs rounds with game ids:
    /// self-play's, or train's rounds of game directories. Stream when
    /// loading.
    pub revise_on: ReviseOn,
    /// With `leaf_residual`: the deltas' AdaGrad step size, as a multiple of
    /// `lr_rate` (that of the shadows and of plain leaves). 1 when loading.
    pub residual_delta_rate: f64,
}

/// As `#[derive(Debug)]` would write it, except that the residual options
/// appear only when they are on, so that the options trees without them
/// record in their state files are what they were before those options.
impl std::fmt::Debug for DelegationParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("DelegationParams");
        s.field("class_num", &self.class_num)
            .field("branch_factor", &self.branch_factor)
            .field("split_threshold", &self.split_threshold)
            .field("split_limit", &self.split_limit)
            .field("forget_factor", &self.forget_factor)
            .field("nb_alpha", &self.nb_alpha)
            .field("use_probs", &self.use_probs)
            .field("leaf", &self.leaf)
            .field("router", &self.router)
            .field("lr_rate", &self.lr_rate)
            .field("error", &self.error)
            .field("split_z", &self.split_z)
            .field("split_rule", &self.split_rule)
            .field("route", &self.route)
            .field("error_target", &self.error_target)
            .field("leaf_share", &self.leaf_share)
            .field("split_trial", &self.split_trial)
            .field("trial_error", &self.trial_error)
            .field("route_switch", &self.route_switch)
            .field("error_router", &self.error_router)
            .field("revise", &self.revise)
            .field("seam_weight", &self.seam_weight)
            .field("reset_leaves", &self.reset_leaves);
        if self.leaf_residual || self.residual_l2 != 0.0 {
            s.field("leaf_residual", &self.leaf_residual).field("residual_l2", &self.residual_l2);
        }
        if self.residual_inherit_sums {
            s.field("residual_inherit_sums", &self.residual_inherit_sums);
        }
        // Not log_fresh_gains: it does not change training.
        if self.revise_on != ReviseOn::Stream {
            s.field("revise_on", &self.revise_on);
        }
        if self.residual_delta_rate != 1.0 {
            s.field("residual_delta_rate", &self.residual_delta_rate);
        }
        s.finish()
    }
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
    /// A delegator's shadow (see `DelegationParams::revise`). Not saved.
    pub revision: Option<Box<Revision>>,
    /// A residual leaf's own weights, the delta on its delegator's shadow;
    /// its `classifier` holds the shadow's plus these (see
    /// `DelegationParams::leaf_residual`). None for other nodes. Not saved in
    /// model files.
    pub residual: Option<Box<Logreg>>,
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

    /// The mean gain of the finished blocks and its standard error from
    /// theirs, and the standard error from single examples, which
    /// understates the noise of correlated examples.
    fn stats(&self) -> (f64, f64, f64) {
        let blocks = self.blocks as f64;
        let mean = self.block_sum / blocks;
        let se = ((self.block_sum_sq / blocks - mean * mean).max(0.0) / (blocks - 1.0)).sqrt();
        let n = self.examples as f64;
        let example_se = ((self.sum_sq / n - (self.sum / n).powi(2)).max(0.0) / n).sqrt();
        (mean, se, example_se)
    }
}

/// A round's gains of the branch over the shadow (see `DelegationParams::
/// log_fresh_gains`), summed by game: per game, the sum of its examples'
/// gains and their number.
#[derive(Clone, Debug, Default)]
pub(crate) struct RoundGains {
    pub games: BTreeMap<u64, (f64, u32)>,
}

impl RoundGains {
    pub(crate) fn add(&mut self, game: u64, gain: f64) {
        let entry = self.games.entry(game).or_insert((0.0, 0));
        entry.0 += gain;
        entry.1 += 1;
    }

    /// The mean gain per example, its standard error clustered by game (a
    /// game's positions share its result, so their gains are correlated),
    /// and the numbers of examples and games; None without examples.
    pub(crate) fn stats(&self) -> Option<(f64, f64, u64, usize)> {
        let n: u64 = self.games.values().map(|&(_, count)| count as u64).sum();
        if n == 0 {
            return None;
        }
        let mean = self.games.values().map(|&(sum, _)| sum).sum::<f64>() / n as f64;
        let k = self.games.len();
        let se = if k > 1 {
            let squares: f64 = self.games.values().map(|&(sum, count)| (sum - mean * count as f64).powi(2)).sum();
            (k as f64 / (k - 1) as f64 * squares).sqrt() / n as f64
        } else {
            0.0
        };
        Some((mean, se, n, k))
    }

    fn describe(&self) -> String {
        match self.stats() {
            None => "none".to_string(),
            Some((mean, se, n, k)) => format!(
                "gain {mean:+.7} per example (standard error by game {se:.7}, z {:+.2}; {n} positions in {k} games)",
                if se > 0.0 { mean / se } else { 0.0 }
            ),
        }
    }
}

/// A delegator's shadow, and the windows that compare its branch and its
/// leaf children with it (see `DelegationParams::revise`).
#[derive(Clone, Debug)]
pub(crate) struct Revision {
    /// The leaf the delegator was before it split, forgotten like the
    /// children it split into, and trained since on every example through
    /// the delegator.
    pub shadow: NodeModel,
    /// Examples the shadow trained on.
    pub trained: u32,
    /// Per example through the delegator, the error (`trial_error`) of the
    /// shadow's prediction minus that of the branch's.
    pub window: GainWindow,
    /// The same per child, over the examples routed to it while it is a
    /// leaf.
    pub leaf_windows: Vec<GainWindow>,
    /// The branch's seams in the window's rounds (see `Graph::
    /// measure_seams`): positions through the delegator, moves between two
    /// of its leaves, and the sum of their absolute seams.
    pub positions: u64,
    pub crossings: u64,
    pub seams: f64,
    /// Seams per position of the last window that measured any, for windows
    /// shorter than a round that measure none.
    pub last_seams: f64,
    /// The current round's gains by game, as `Graph::observe_round` scored
    /// them at the round's start and as training does (see
    /// `DelegationParams::log_fresh_gains`). Not saved.
    pub round_fresh: RoundGains,
    pub round_stream: RoundGains,
}

impl Revision {
    /// Starts the windows and the seams afresh, as at the split, keeping the
    /// shadow and its training: after a switch to error routing, the branch
    /// and its leaves are judged on error-routed examples only.
    fn restart(&mut self) {
        let branch_factor = self.leaf_windows.len();
        self.window = GainWindow::default();
        self.leaf_windows = vec![GainWindow::default(); branch_factor];
        (self.positions, self.crossings, self.seams, self.last_seams) = (0, 0, 0.0, 0.0);
    }

    fn new(shadow: NodeModel, branch_factor: usize) -> Box<Self> {
        Box::new(Revision {
            shadow,
            trained: 0,
            window: GainWindow::default(),
            leaf_windows: vec![GainWindow::default(); branch_factor],
            positions: 0,
            crossings: 0,
            seams: 0.0,
            last_seams: 0.0,
            round_fresh: RoundGains::default(),
            round_stream: RoundGains::default(),
        })
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
            revision: None,
            residual: None,
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
    /// The game of the example training (`Bdt::train_in_game`), for
    /// `DelegationParams::log_fresh_gains`. Not saved.
    pub game: Option<u64>,
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
        assert!(
            params.split_trial.is_none() || (params.revise.is_none() && params.reset_leaves.is_none()),
            "split trials and revisable structure (revise, reset_leaves) are exclusive"
        );
        assert!(
            !params.leaf_residual
                || (params.leaf == NodeKind::Lr && (params.revise.is_some() || params.reset_leaves.is_some())),
            "residual leaves need logistic-regression leaves (leaf lr) and shadows (revise or reset_leaves)"
        );
        assert!(
            params.residual_l2 >= 0.0 && (params.residual_l2 == 0.0 || params.leaf_residual),
            "residual_l2 is a penalty (at least 0) for residual leaves (leaf_residual)"
        );
        assert!(
            !params.residual_inherit_sums || params.leaf_residual,
            "residual_inherit_sums is for residual leaves (leaf_residual)"
        );
        let shadows = params.revise.is_some() || params.reset_leaves.is_some();
        assert!(
            shadows || (!params.log_fresh_gains && params.revise_on == ReviseOn::Stream),
            "log_fresh_gains and revise_on fresh score shadows (revise or reset_leaves)"
        );
        assert!(
            params.residual_delta_rate > 0.0 && (params.residual_delta_rate == 1.0 || params.leaf_residual),
            "residual_delta_rate is a positive multiple for residual leaves (leaf_residual)"
        );
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
            game: None,
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
        self.leaf_from(0, data)
    }

    /// Index of the leaf that predicts the position when routed from node
    /// `idx`.
    pub fn leaf_from(&self, mut idx: usize, data: &[u8]) -> usize {
        loop {
            let node = &self.nodes[idx];
            match &node.kind {
                Kind::Leaf => return idx,
                Kind::Branch { children } => idx = children[best_child(&node.classifier, data)],
            }
        }
    }

    /// Node `idx`'s children; empty for a leaf.
    pub fn children(&self, idx: usize) -> &[usize] {
        match &self.nodes[idx].kind {
            Kind::Leaf => &[],
            Kind::Branch { children } => children,
        }
    }

    /// Divides every model in the graph by `factor` (see `NodeModel::forget`):
    /// the nodes' classifiers, error routers and the shadows of trials and
    /// revisions, and residual leaves' deltas, so the whole graph adapts
    /// faster while predicting the same.
    pub fn forget(&mut self, factor: f64) {
        for node in &mut self.nodes {
            node.classifier.forget(factor);
            if let Some(trial) = &mut node.trial {
                trial.shadow.forget(factor);
            }
            if let Some(error_router) = &mut node.error_router {
                error_router.router.forget(factor);
            }
            if let Some(revision) = &mut node.revision {
                revision.shadow.forget(factor);
            }
            if let Some(delta) = &mut node.residual {
                delta.forget(factor);
            }
        }
        // A residual leaf's sums are its shadow's plus its delta's.
        if self.params.leaf_residual {
            for idx in 0..self.nodes.len() {
                self.sync_residual_children(idx, None);
            }
        }
    }

    /// The delegator whose children include node `idx`; None for the root.
    fn parent_of(&self, idx: usize) -> Option<usize> {
        self.nodes.iter().position(|n| matches!(&n.kind, Kind::Branch { children } if children.contains(&idx)))
    }

    /// Makes the classifier of every residual leaf among delegator `idx`'s
    /// children its delegator's shadow plus its delta again, after either
    /// changed: every weight and AdaGrad sum, or with `data` just those the
    /// example uses (the only ones a training step on it changes). The shadow
    /// must be in place (`revision` not taken). Nothing for a leaf.
    fn sync_residual_children(&mut self, idx: usize, data: Option<&[u8]>) {
        let Kind::Branch { children } = &self.nodes[idx].kind else { return };
        let children = children.clone();
        let revision = self.nodes[idx].revision.take().expect("a delegator over residual leaves keeps a shadow");
        let NodeModel::Lr(shadow) = &revision.shadow else { unreachable!("residual leaves are logistic regressions") };
        let add_squares = !self.params.residual_inherit_sums;
        for child in children {
            let node = &mut self.nodes[child];
            if let (Some(delta), NodeModel::Lr(sum)) = (&node.residual, &mut node.classifier) {
                match data {
                    Some(data) => sum.set_sum_at(shadow, delta, data, add_squares),
                    None => sum.set_sum(shadow, delta, add_squares),
                }
                node.cache_step = 0;
            }
        }
        self.nodes[idx].revision = Some(revision);
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

    /// `train_leaf` for leaf `idx` below delegator `parent`: a residual leaf
    /// (see `DelegationParams::leaf_residual`) trains its delta by an AdaGrad
    /// step on the log-loss of its summed prediction, the shadow's logits
    /// held fixed, and its sum follows; other leaves train as usual.
    pub fn train_leaf_below(&mut self, idx: usize, parent: Option<usize>, data: &[u8], label: &[f64], error: Option<f64>) {
        if self.nodes[idx].residual.is_none() {
            return self.train_leaf(idx, data, label, error);
        }
        let parent = parent.expect("a residual leaf has a delegator");
        let revision = self.nodes[parent].revision.take().expect("a delegator over residual leaves keeps a shadow");
        let NodeModel::Lr(shadow) = &revision.shadow else { unreachable!("residual leaves are logistic regressions") };
        let (l2, add_squares) = (self.params.residual_l2, !self.params.residual_inherit_sums);
        let node = &mut self.nodes[idx];
        let (NodeModel::Lr(sum), Some(delta)) = (&mut node.classifier, &mut node.residual) else { unreachable!() };
        let mut p = [0.0; 256];
        let p = &mut p[..sum.class_num()];
        sum.predict(data, p);
        delta.step(data, p, label, 1.0, l2);
        sum.set_sum_at(shadow, delta, data, add_squares);
        node.cache_step = 0;
        if let Some(error) = error {
            node.record(error);
        }
        self.nodes[parent].revision = Some(revision);
    }

    /// With residual leaves (see `DelegationParams::leaf_residual`), trains
    /// delegator `idx`'s shadow on the example, which `revise_step` leaves to
    /// this, and brings its residual leaves' sums up to date. Call it once its
    /// children have predicted the example (routing), before the child it
    /// routes the example to trains. Nothing without residual leaves.
    pub fn train_shadow(&mut self, idx: usize, data: &[u8], label: &[f64]) {
        if !self.params.leaf_residual {
            return;
        }
        let revision = self.nodes[idx].revision.as_mut().expect("a delegator over residual leaves keeps a shadow");
        revision.shadow.train(data, label);
        revision.trained += 1;
        self.sync_residual_children(idx, Some(data));
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
        let (mean, se, example_se) = w.stats();
        let switch = mean > self.params.route_switch.unwrap_or(f64::INFINITY) * se;
        let restart = switch && self.nodes[idx].revision.is_some();
        eprintln!(
            "routers of delegator {idx}: error router's gain {mean:+.7} per example (standard error {se:.7} over {} blocks, {example_se:.7} per example), picks differ on {} of {} judged examples{}{}",
            w.blocks,
            w.differ,
            w.judged,
            if switch { "; switched to error routing" } else { "" },
            if restart { "; its shadow's windows start afresh" } else { "" }
        );
        if switch {
            let error_router = self.nodes[idx].error_router.take().unwrap();
            let node = &mut self.nodes[idx];
            node.classifier = error_router.router;
            node.cache_step = 0;
            node.error_routed = true;
            // The branch has been cluster-routed so far: judge it against its
            // shadow on a full window of error-routed examples (see
            // `DelegationParams::revise`).
            if let Some(revision) = &mut node.revision {
                revision.restart();
            }
        }
    }

    /// Whether this example ends the window of delegator `idx`'s comparison
    /// of its routers (see `train_cluster_router`, which adds the examples
    /// that are scored).
    fn router_window_ends(&self, idx: usize, label: &[f64]) -> bool {
        self.params.route == RouteTarget::Cluster
            && self.scored(label)
            && self.nodes[idx].error_router.as_ref().is_some_and(|e| e.window.examples + 1 >= self.params.split_threshold)
    }

    /// The error router of a new cluster-routed delegator, if any (see
    /// `DelegationParams::route_switch`).
    pub fn new_error_router(&self) -> Option<Box<ErrorRouter>> {
        let wanted = self.params.route == RouteTarget::Cluster && self.params.route_switch.is_some();
        wanted.then(|| {
            let router = self.params.new_classifier(self.params.error_router, self.params.branch_factor, &self.categories);
            Box::new(ErrorRouter { router, window: GainWindow::default() })
        })
    }

    pub fn record_router(&mut self, idx: usize, router_error: f64) {
        self.nodes[idx].record(router_error);
        self.router_errors.add(router_error);
    }

    /// Turns leaf `idx` into a delegator over `branch_factor` new leaves,
    /// copies of it that forget by `forget_factor` (see `NodeModel::forget`),
    /// so they predict the same but adapt faster. With `split_trial`, one
    /// more such copy stands for the leaf without the split until the split's
    /// trial ends (see `trial_step`). With residual leaves, the new leaves
    /// are deltas of 0 on the shadow (see `split_residual_leaf`).
    pub fn split_leaf(&mut self, idx: usize) {
        if self.params.leaf_residual {
            return self.split_residual_leaf(idx);
        }
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
        let revision = self.revising().then(|| {
            let mut shadow = self.nodes[idx].classifier.clone();
            shadow.forget(self.params.forget_factor);
            Revision::new(shadow, self.params.branch_factor)
        });

        self.make_delegator(idx, made, trial, revision);
    }

    /// `split_leaf` with residual leaves (see `DelegationParams::
    /// leaf_residual`): the shadow is the leaf's model, forgotten by
    /// `forget_factor` as usual; a residual leaf's classifier holds the sums
    /// of its delegator's shadow and its delta, AdaGrad sums too, so its
    /// summed model is materialised as one logistic regression. The new
    /// leaves' deltas are all 0, with no AdaGrad history, so the new leaves
    /// predict as the shadow. With `residual_inherit_sums`, the leaf's
    /// AdaGrad sums are its delta's, and the new deltas start with the
    /// shadow's: the shadow and the new leaves are then exactly the copies
    /// of the leaf that a split without residual leaves makes.
    fn split_residual_leaf(&mut self, idx: usize) {
        let mut shadow = self.nodes[idx].classifier.clone();
        shadow.forget(self.params.forget_factor);
        let NodeModel::Lr(base) = &shadow else { unreachable!("residual leaves are logistic regressions") };
        let first = self.nodes.len();
        for _ in 0..self.params.branch_factor {
            let (classifier, residual) = residual_leaf(base, self.new_delta(base), !self.params.residual_inherit_sums);
            let child = Node { residual, ..self.new_node(Kind::Leaf, classifier) };
            self.nodes.push(child);
        }
        let made = (first..first + self.params.branch_factor).collect();
        self.nodes[idx].residual = None;
        self.make_delegator(idx, made, None, Some(Revision::new(shadow, self.params.branch_factor)));
    }

    /// A new residual leaf's delta over `shadow` (forgotten already, at a
    /// split): every weight 0, and no AdaGrad sums or with
    /// `residual_inherit_sums` the shadow's; its step size `residual_delta_rate`
    /// times the shadow's.
    fn new_delta(&self, shadow: &Logreg) -> Logreg {
        let delta = if self.params.residual_inherit_sums { shadow.zeroed() } else { shadow.blank() };
        delta.with_rate(self.params.residual_delta_rate * shadow.rate())
    }

    /// The rest of a split of leaf `idx`: makes it a delegator over the new
    /// leaves `made`, with a new router (and error router), and the split's
    /// trial and shadow if any.
    fn make_delegator(&mut self, idx: usize, made: Vec<usize>, trial: Option<Box<Trial>>, revision: Option<Box<Revision>>) {
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
        node.revision = revision;
        self.split_number += 1;
    }

    /// Whether delegators keep shadows (see `DelegationParams::revise`).
    fn revising(&self) -> bool {
        self.params.revise.is_some() || self.params.reset_leaves.is_some()
    }

    /// For a delegator with a shadow (see `DelegationParams::revise`), at the
    /// start of its part of a training step: ends the window once it has
    /// seen `split_threshold` examples, merging the branch into the shadow
    /// if it failed; otherwise scores the example's prediction by the branch
    /// against the shadow's, as the routed child's too if it is a leaf (see
    /// `DelegationParams::reset_leaves`), and trains the shadow. Returns
    /// whether the branch was merged, which changes the node. If this
    /// example also ends the comparison of the delegator's routers, that
    /// decision comes first: a switch restarts the window, and otherwise it
    /// ends with the next example.
    pub fn revise_step(&mut self, idx: usize, data: &[u8], label: &[f64], scratch: &mut [f64]) -> bool {
        let Some(mut revision) = self.nodes[idx].revision.take() else { return false };
        // On fresh rounds, `observe_round` feeds and ends the windows instead.
        let stream = self.params.revise_on == ReviseOn::Stream;
        let due = stream
            && revision.window.examples >= self.params.split_threshold
            && !self.router_window_ends(idx, label);
        if due && self.end_revision_window(idx, &mut revision) {
            self.merge(idx, *revision);
            return true;
        }
        let block_len = (self.params.split_threshold / 20).max(1);
        let judge = self.params.trial_error;
        let judged = judge != ErrorMeasure::Winner || label[0] + label[1] > 0.0;
        let mut gain = 0.0;
        if judged && (stream || self.params.log_fresh_gains) {
            self.predict_training(idx, data, scratch);
            let branch_error = measure(judge, scratch, label);
            revision.shadow.predict_training(data, scratch);
            gain = measure(judge, scratch, label) - branch_error;
        }
        if let (true, Some(game)) = (self.params.log_fresh_gains, self.game) {
            revision.round_stream.add(game, gain);
        }
        if stream {
            revision.window.judged += judged as u32;
            revision.window.add(gain, block_len);

            // The child that predicts the example, as in `predict_training`.
            let slot = self.route(idx, data);
            let Kind::Branch { children } = &self.nodes[idx].kind else { unreachable!() };
            if matches!(self.nodes[children[slot]].kind, Kind::Leaf) {
                let window = &mut revision.leaf_windows[slot];
                window.judged += judged as u32;
                window.add(gain, block_len);
                if window.examples >= self.params.split_threshold {
                    self.end_leaf_window(idx, slot, &mut revision);
                }
            } else if revision.leaf_windows[slot].examples > 0 {
                // The leaf split; its window is no longer the child's.
                revision.leaf_windows[slot] = GainWindow::default();
            }
        }
        // Residual leaves predict with the shadow: it trains once they all
        // have (`train_shadow`), so they predict the example as at the start
        // of the step.
        if !self.params.leaf_residual {
            revision.shadow.train(data, label);
            revision.trained += 1;
        }
        self.nodes[idx].revision = Some(revision);
        false
    }

    /// Ends the window of delegator `idx`'s comparison with its shadow: logs
    /// it and returns whether the branch failed (see `DelegationParams::
    /// revise`); if not, starts a new window.
    fn end_revision_window(&mut self, idx: usize, revision: &mut Revision) -> bool {
        let (mean, se, example_se) = revision.window.stats();
        let seams = if revision.positions > 0 { revision.seams / revision.positions as f64 } else { revision.last_seams };
        let net = mean - self.params.seam_weight * seams;
        let merge = self.params.revise.is_some_and(|z| net < -z * se);
        eprintln!(
            "revision of delegator {idx}: gain {mean:+.7} per example (standard error {se:.7} over {} blocks, {example_se:.7} per example), seams {seams:.5} per position ({} moves across leaves in {} positions), net {net:+.7}{}",
            revision.window.blocks,
            revision.crossings,
            revision.positions,
            if merge { "; merging" } else { "" }
        );
        if !merge {
            revision.window = GainWindow::default();
            (revision.positions, revision.crossings, revision.seams, revision.last_seams) = (0, 0, 0.0, seams);
        }
        merge
    }

    /// Ends the window of the leaf child in `slot` of delegator `idx`: logs
    /// it, and resets the leaf to a copy of the shadow if it failed (see
    /// `DelegationParams::reset_leaves`).
    fn end_leaf_window(&mut self, idx: usize, slot: usize, revision: &mut Revision) {
        let window = std::mem::take(&mut revision.leaf_windows[slot]);
        let (mean, se, example_se) = window.stats();
        let reset = self.params.reset_leaves.is_some_and(|z| mean < -z * se);
        let Kind::Branch { children } = &self.nodes[idx].kind else { unreachable!() };
        let leaf = children[slot];
        let delta = match &self.nodes[leaf].residual {
            Some(delta) => {
                let (l2, max) = delta.weight_norms();
                format!(", delta norm {l2:.4}, largest weight {max:.4}")
            }
            None => String::new(),
        };
        eprintln!(
            "leaf {leaf} of delegator {idx}: gain over the shadow {mean:+.7} per example (standard error {se:.7} over {} blocks, {example_se:.7} per example){delta}{}",
            window.blocks,
            if reset { "; reset to the shadow" } else { "" }
        );
        if reset {
            let (classifier, residual) = match &revision.shadow {
                // A new delta, as a split's children get; inheriting sums,
                // the shadow's forgotten, as a copy's would be.
                NodeModel::Lr(shadow) if self.params.leaf_residual => {
                    let mut delta = self.new_delta(shadow);
                    delta.forget(self.params.forget_factor);
                    residual_leaf(shadow, delta, !self.params.residual_inherit_sums)
                }
                shadow => {
                    let mut classifier = shadow.clone();
                    classifier.forget(self.params.forget_factor);
                    (classifier, None)
                }
            };
            let node = &mut self.nodes[leaf];
            node.classifier = classifier;
            node.residual = residual;
            node.cache_step = 0;
            node.num_trained_total = 0;
            node.error_total = 0.0;
            node.forget_window();
            self.open_window(leaf);
        }
    }

    /// Puts delegator `idx`'s shadow in its branch's place, as a leaf; the
    /// branch's nodes are removed (see `dead`) and its splits no longer
    /// count.
    pub(crate) fn merge(&mut self, idx: usize, revision: Revision) {
        let Kind::Branch { children } = &self.nodes[idx].kind else { unreachable!("merging a leaf") };
        let mut stack = children.clone();
        let mut splits = 1;
        while let Some(n) = stack.pop() {
            self.dead.push(n);
            if let Kind::Branch { children } = &self.nodes[n].kind {
                splits += 1;
                stack.extend(children);
            }
        }
        eprintln!("merged delegator {idx}'s branch into its shadow, undoing {splits} splits");
        let (classifier, residual) = match self.params.leaf_residual.then(|| self.parent_of(idx)).flatten() {
            // With residual leaves, a leaf below a delegator is residual: the
            // shadow becomes a delta on the shadow above, keeping its own
            // AdaGrad sums, so the leaf predicts as the shadow did.
            Some(parent) => {
                let above = &self.nodes[parent].revision.as_ref().expect("a delegator over residual leaves keeps a shadow");
                let (NodeModel::Lr(own), NodeModel::Lr(above)) = (&revision.shadow, &above.shadow) else {
                    unreachable!("residual leaves are logistic regressions")
                };
                let delta = own.minus(above).with_rate(self.params.residual_delta_rate * own.rate());
                residual_leaf(above, delta, !self.params.residual_inherit_sums)
            }
            None => (revision.shadow, None),
        };
        let class_num = self.params.class_num;
        let node = &mut self.nodes[idx];
        node.kind = Kind::Leaf;
        node.classifier = classifier;
        node.residual = residual;
        node.cache = vec![0.0; class_num];
        node.cache_step = 0;
        node.num_trained_total = revision.trained;
        node.error_total = 0.0;
        node.trial = None;
        node.error_router = None;
        node.error_routed = false;
        node.forget_window();
        self.open_window(idx);
        self.split_number -= splits;
    }

    /// With shadows (see `DelegationParams::revise`), measures the seams of
    /// a round's games, each the positions in the order played, for every
    /// delegator: the positions through it, and for each move between two
    /// of its leaves the seam, the difference in white's log-odds of the
    /// position after the move between the leaf it is in and the leaf of
    /// the position before. Call it with the model refreshed, before
    /// training on the games.
    pub fn measure_seams<P: AsRef<[u8]>>(&mut self, games: &[&[P]]) {
        if !self.revising() {
            return;
        }
        self.remove_dead();
        let n = self.nodes.len();
        let (mut positions, mut crossings, mut seams) = (vec![0u64; n], vec![0u64; n], vec![0.0; n]);
        let (mut path, mut previous) = (Vec::new(), Vec::new());
        for game in games {
            for (i, position) in game.iter().enumerate() {
                let data = position.as_ref();
                self.path(data, &mut path);
                let (&leaf, delegators) = path.split_last().expect("a path has a leaf");
                delegators.iter().for_each(|&d| positions[d] += 1);
                let before = previous.last().copied().filter(|_| i > 0);
                if let Some(before) = before.filter(|&b| b != leaf) {
                    let seam = (self.white_log_odds(leaf, data) - self.white_log_odds(before, data)).abs();
                    // The delegators above both leaves: the paths' common start.
                    for (&d, _) in path.iter().zip(&previous).take_while(|(a, b)| a == b) {
                        crossings[d] += 1;
                        seams[d] += seam;
                    }
                }
                std::mem::swap(&mut path, &mut previous);
            }
        }
        for (i, node) in self.nodes.iter_mut().enumerate() {
            if let Some(revision) = &mut node.revision {
                revision.positions += positions[i];
                revision.crossings += crossings[i];
                revision.seams += seams[i];
            }
        }
    }

    /// The nodes from the root to the leaf that predicts the position.
    fn path(&self, data: &[u8], out: &mut Vec<usize>) {
        out.clear();
        let mut idx = 0;
        loop {
            out.push(idx);
            let node = &self.nodes[idx];
            match &node.kind {
                Kind::Leaf => return,
                Kind::Branch { children } => idx = children[best_child(&node.classifier, data)],
            }
        }
    }

    /// `path` as training routes positions: by the routers' outputs in
    /// training (see `predict_training`).
    fn training_path(&self, data: &[u8], out: &mut Vec<usize>) {
        out.clear();
        let mut scores = [0.0; 256];
        let scores = &mut scores[..self.params.branch_factor];
        let mut idx = 0;
        loop {
            out.push(idx);
            let node = &self.nodes[idx];
            match &node.kind {
                Kind::Leaf => return,
                Kind::Branch { children } => {
                    node.classifier.predict_training(data, scores);
                    idx = children[argmax(scores)];
                }
            }
        }
    }

    /// Before training on a self-play round, with `log_fresh_gains` or
    /// `revise_on` fresh: scores each of the round's positions (in training
    /// order, with labels and game ids) by the model as it starts the round,
    /// without training it. Per delegator on the position's path, the gain is
    /// the shadow test's: the error (`trial_error`) of the shadow's
    /// prediction minus that of the branch's, which is the leaf's (0 for
    /// examples the test does not judge), as `revise_step` scores examples in
    /// training. With `log_fresh_gains` it sums them by game for `end_round`.
    /// On fresh rounds they feed the windows (and the leaves' windows)
    /// instead of training's gains, and a window that ends may merge its
    /// branch at once: later positions see the merged tree. Router windows
    /// (`route_switch`) do not delay fresh windows.
    pub fn observe_round<P: AsRef<[u8]>, L: AsRef<[f64]>>(&mut self, data: &[(P, L, u64)]) {
        let fresh = self.params.revise_on == ReviseOn::Fresh;
        if !fresh && !self.params.log_fresh_gains {
            return;
        }
        self.remove_dead();
        for node in &mut self.nodes {
            if let Some(revision) = &mut node.revision {
                revision.round_fresh = RoundGains::default();
                revision.round_stream = RoundGains::default();
            }
        }
        let judge = self.params.trial_error;
        let block_len = (self.params.split_threshold / 20).max(1);
        let class_num = self.params.class_num;
        let (mut path, mut leaf_p, mut shadow_p) = (Vec::new(), vec![0.0; class_num], vec![0.0; class_num]);
        for (position, label, game) in data {
            let (position, label) = (position.as_ref(), label.as_ref());
            self.training_path(position, &mut path);
            let judged = judge != ErrorMeasure::Winner || label[0] + label[1] > 0.0;
            let leaf = *path.last().expect("a path has a leaf");
            let leaf_error = if judged {
                self.nodes[leaf].classifier.predict_training(position, &mut leaf_p);
                measure(judge, &leaf_p, label)
            } else {
                0.0
            };
            for depth in 0..path.len() - 1 {
                let idx = path[depth];
                let Some(mut revision) = self.nodes[idx].revision.take() else { continue };
                if fresh
                    && revision.window.examples >= self.params.split_threshold
                    && self.end_revision_window(idx, &mut revision)
                {
                    // The rest of the path is gone.
                    self.merge(idx, *revision);
                    break;
                }
                let mut gain = 0.0;
                if judged {
                    revision.shadow.predict_training(position, &mut shadow_p);
                    gain = measure(judge, &shadow_p, label) - leaf_error;
                }
                if self.params.log_fresh_gains {
                    revision.round_fresh.add(*game, gain);
                }
                if fresh {
                    revision.window.judged += judged as u32;
                    revision.window.add(gain, block_len);
                    let child = path[depth + 1];
                    let Kind::Branch { children } = &self.nodes[idx].kind else { unreachable!() };
                    let slot = children.iter().position(|&c| c == child).expect("a path goes from a node to a child");
                    if matches!(self.nodes[child].kind, Kind::Leaf) {
                        let window = &mut revision.leaf_windows[slot];
                        window.judged += judged as u32;
                        window.add(gain, block_len);
                        if window.examples >= self.params.split_threshold {
                            self.end_leaf_window(idx, slot, &mut revision);
                        }
                    } else if revision.leaf_windows[slot].examples > 0 {
                        revision.leaf_windows[slot] = GainWindow::default();
                    }
                }
                self.nodes[idx].revision = Some(revision);
            }
        }
    }

    /// After training on a self-play round, with `log_fresh_gains`: logs per
    /// delegator the round's mean gain of the branch over the shadow as
    /// `observe_round` scored the round's positions (fresh) and as training
    /// scored them (stream), each with its standard error clustered by game.
    /// A delegator that split during the round has no fresh gains; one that
    /// merged is not logged.
    pub fn end_round(&self) {
        if !self.params.log_fresh_gains {
            return;
        }
        for (idx, node) in self.nodes.iter().enumerate() {
            let Some(revision) = &node.revision else { continue };
            if !self.dead.contains(&idx) {
                eprintln!(
                    "round gains of delegator {idx}: fresh {}; stream {}",
                    revision.round_fresh.describe(),
                    revision.round_stream.describe()
                );
            }
        }
    }

    /// White's log-odds by node `idx`'s classifier: ln P(white wins) - ln
    /// P(black wins), for classes [white win, black win, ...].
    pub(crate) fn white_log_odds(&self, idx: usize, data: &[u8]) -> f64 {
        let mut logs = [0.0; 256];
        let logs = &mut logs[..self.params.class_num];
        self.nodes[idx].classifier.predict_log(data, logs);
        logs[0] - logs[1]
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
            error_router: router,
            revise: None,
            seam_weight: 0.0,
            reset_leaves: None,
            leaf_residual: false,
            residual_l2: 0.0,
            residual_inherit_sums: false,
            log_fresh_gains: false,
            revise_on: ReviseOn::Stream,
            residual_delta_rate: 1.0,
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
            game: None,
        })
    }
}

/// The whole training state of a delegation model, for `--resume` (see
/// `state`): everything `to_text` saves, unrounded, and what it does not:
/// the windows' sums of squares and running totals, trials, error routers,
/// which delegators switched to them, shadows and their windows, the step
/// count and the model's and routers' running errors. Each node's
/// prediction cache is not saved: it is only valid within a training step,
/// and the state is saved between steps.
impl Graph {
    pub fn write_state(&self, w: &mut Writer) {
        w.str(&format!("{:?}", self.params));
        w.u8s(&self.categories);
        w.u32(self.split_number);
        w.u64(self.step);
        write_running(w, self.model_errors);
        write_running(w, self.router_errors);
        w.usizes(&self.dead);
        w.usize(self.nodes.len());
        for node in &self.nodes {
            match &node.kind {
                Kind::Leaf => w.u8(0),
                Kind::Branch { children } => {
                    w.u8(1);
                    w.usizes(children);
                }
            }
            node.classifier.write_state(w);
            w.u32(node.num_trained_total);
            w.u32(node.num_trained_last);
            for x in [node.error_total, node.error_last, node.error_prev, node.error_sq_last, node.error_prev_se2] {
                w.f64(x);
            }
            write_running(w, node.window_model);
            write_running(w, node.window_routers);
            w.bool(node.trial.is_some());
            if let Some(trial) = &node.trial {
                trial.shadow.write_state(w);
                w.bool(trial.children.is_some());
                if let Some(children) = &trial.children {
                    w.usizes(children);
                }
                w.u32(trial.trained_before);
                w.usizes(&trial.made);
                w.u32(trial.examples);
                w.u32(trial.n);
                w.f64(trial.sum);
                w.f64(trial.sum_sq);
            }
            w.bool(node.error_router.is_some());
            if let Some(error_router) = &node.error_router {
                error_router.router.write_state(w);
                write_window(w, &error_router.window);
            }
            w.bool(node.error_routed);
            w.bool(node.revision.is_some());
            if let Some(revision) = &node.revision {
                revision.shadow.write_state(w);
                w.u32(revision.trained);
                write_window(w, &revision.window);
                w.usize(revision.leaf_windows.len());
                revision.leaf_windows.iter().for_each(|window| write_window(w, window));
                w.u64(revision.positions);
                w.u64(revision.crossings);
                w.f64(revision.seams);
                w.f64(revision.last_seams);
            }
            // Only residual trees have deltas, so others' states are as before.
            if self.params.leaf_residual {
                w.bool(node.residual.is_some());
                if let Some(delta) = &node.residual {
                    delta.write_state(w);
                }
            }
        }
    }

    /// Reads a state written by `write_state` by a model trained with
    /// `params` (checked) over `categories`.
    pub fn read_state(r: &mut Reader, categories: &[u8], params: DelegationParams) -> io::Result<Self> {
        r.expect_str("options", &format!("{params:?}"))?;
        if r.u8s()? != categories {
            return Err(invalid("the state's features differ from this run's"));
        }
        let split_number = r.u32()?;
        let step = r.u64()?;
        let model_errors = read_running(r)?;
        let router_errors = read_running(r)?;
        let dead = r.usizes()?;
        let node_num = r.usize()?;
        let mut nodes = Vec::with_capacity(node_num.min(1 << 16));
        for _ in 0..node_num {
            let kind = match r.u8()? {
                0 => Kind::Leaf,
                1 => Kind::Branch { children: r.usizes()? },
                k => return Err(invalid(format!("unknown node kind {k} in state file"))),
            };
            let classifier = NodeModel::read_state(r)?;
            let mut node = Node::new(kind, classifier);
            node.num_trained_total = r.u32()?;
            node.num_trained_last = r.u32()?;
            node.error_total = r.f64()?;
            node.error_last = r.f64()?;
            node.error_prev = r.f64()?;
            node.error_sq_last = r.f64()?;
            node.error_prev_se2 = r.f64()?;
            node.window_model = read_running(r)?;
            node.window_routers = read_running(r)?;
            if r.bool()? {
                let shadow = NodeModel::read_state(r)?;
                let children = if r.bool()? { Some(r.usizes()?) } else { None };
                let trained_before = r.u32()?;
                let made = r.usizes()?;
                let mut trial = Trial::new(shadow, children, trained_before, made);
                trial.examples = r.u32()?;
                trial.n = r.u32()?;
                trial.sum = r.f64()?;
                trial.sum_sq = r.f64()?;
                node.trial = Some(trial);
            }
            if r.bool()? {
                let router = NodeModel::read_state(r)?;
                node.error_router = Some(Box::new(ErrorRouter { router, window: read_window(r)? }));
            }
            node.error_routed = r.bool()?;
            if r.bool()? {
                let shadow = NodeModel::read_state(r)?;
                let mut revision = Revision::new(shadow, 0);
                revision.trained = r.u32()?;
                revision.window = read_window(r)?;
                let windows = r.usize()?;
                revision.leaf_windows = (0..windows).map(|_| read_window(r)).collect::<io::Result<_>>()?;
                revision.positions = r.u64()?;
                revision.crossings = r.u64()?;
                revision.seams = r.f64()?;
                revision.last_seams = r.f64()?;
                node.revision = Some(revision);
            }
            if params.leaf_residual && r.bool()? {
                let delta = Logreg::read_state(r)?;
                let fits = matches!(&node.classifier, NodeModel::Lr(sum) if sum.class_num() == delta.class_num());
                if !matches!(node.kind, Kind::Leaf) || !fits {
                    return Err(invalid("a residual delta in the state belongs to no logistic-regression leaf"));
                }
                node.residual = Some(Box::new(delta));
            }
            nodes.push(node);
        }
        let in_range = |&i: &usize| i < node_num;
        for node in &nodes {
            let children = match &node.kind {
                Kind::Branch { children } => children.as_slice(),
                Kind::Leaf => &[],
            };
            let trial_nodes = node.trial.iter().flat_map(|t| t.made.iter().chain(t.children.iter().flatten()));
            if !children.iter().chain(trial_nodes).chain(&dead).all(in_range) {
                return Err(invalid("a node of the state refers to a node out of range"));
            }
        }
        Ok(Graph {
            categories: categories.to_vec(),
            params,
            split_number,
            nodes,
            step,
            model_errors,
            router_errors,
            dead,
            game: None,
        })
    }
}

/// Six rounds of 12 games for the state tests: walks over 4 features of 3
/// values that change one at a time, labelled by a noisy function of where
/// they start.
#[cfg(test)]
pub(crate) fn state_test_rounds() -> Vec<Vec<(Vec<[u8; 4]>, [f64; 2])>> {
    let mut x = 7u64;
    let mut next = |m: u64| {
        x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (x >> 33) % m
    };
    let mut game = || {
        let mut pos: [u8; 4] = std::array::from_fn(|_| next(3) as u8);
        let p = if pos[0] == pos[1] { 0.85 } else { 0.25 };
        let label = if (next(1000) as f64) < 1000.0 * p { [1.0, 0.0] } else { [0.0, 1.0] };
        let mut positions = Vec::with_capacity(15);
        for _ in 0..15 {
            positions.push(pos);
            let feature = next(4) as usize;
            pos[feature] = next(3) as u8;
        }
        (positions, label)
    };
    (0..6).map(|_| (0..12).map(|_| game()).collect()).collect()
}

fn write_running(w: &mut Writer, running: Running) {
    w.f64(running.sum);
    w.u64(running.count);
}

fn read_running(r: &mut Reader) -> io::Result<Running> {
    Ok(Running { sum: r.f64()?, count: r.u64()? })
}

fn write_window(w: &mut Writer, window: &GainWindow) {
    for x in [window.examples, window.judged, window.differ] {
        w.u32(x);
    }
    for x in [window.sum, window.sum_sq, window.block] {
        w.f64(x);
    }
    w.u32(window.blocks);
    w.f64(window.block_sum);
    w.f64(window.block_sum_sq);
}

fn read_window(r: &mut Reader) -> io::Result<GainWindow> {
    Ok(GainWindow {
        examples: r.u32()?,
        judged: r.u32()?,
        differ: r.u32()?,
        sum: r.f64()?,
        sum_sq: r.f64()?,
        block: r.f64()?,
        blocks: r.u32()?,
        block_sum: r.f64()?,
        block_sum_sq: r.f64()?,
    })
}

/// Whether a window's mean error is at least `mean`, the mean over the same
/// window of a running total. The two are computed differently, so a node
/// that saw every example of the window (the only leaf or router) would
/// otherwise be above or below its own mean by rounding alone.
pub(crate) fn at_least(recent: f64, mean: f64) -> bool {
    recent >= mean - 1e-6 * mean.abs()
}

/// The classifier and delta of a residual leaf over `shadow` whose own
/// weights are `delta` (see `DelegationParams::leaf_residual`): the
/// classifier holds their sums (AdaGrad sums too if `add_squares`,
/// otherwise the delta's; see `DelegationParams::residual_inherit_sums`).
fn residual_leaf(shadow: &Logreg, delta: Logreg, add_squares: bool) -> (NodeModel, Option<Box<Logreg>>) {
    let mut sum = shadow.clone();
    sum.set_sum(shadow, &delta, add_squares);
    (NodeModel::Lr(sum), Some(Box::new(delta)))
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
