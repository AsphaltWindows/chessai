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
use crate::logreg::{Logreg, softmax};
use crate::pathctx::{Mark, PathTable};
use crate::probit::{Probit, ProbitParams};
use crate::tan::{Tan, TanOptions};
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
    /// grow1 (the growth rule, phase 1, models/from-scratch/theory-2/growth-spec.md): a leaf is a candidate once it
    /// has STAGNATED (its window means, from 20 blocks each, improved by at most `GrowParams::stall_z` block standard
    /// errors for `stall_windows` closes in a row); splits are rationed by credits, one per `credit` examples
    /// through the root (at most `credit_cap` banked); at a leaf's window close, with a credit and a free trial slot,
    /// the candidate whose window mean is furthest above the model's over the same window splits on trial, and the
    /// split stays only if its block-mean gain over the unsplit leaf clears `trial_z` block standard errors and the
    /// MDL bar (`mdl`); an undone trial refunds its credit and backs the leaf off. BDT only.
    Grow,
}

/// grow2: when a BDD router (delegator) splits (theory-2 6.40-6.41; growth-spec.md section 2 as 6.40 amends it).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum RouterSplitRule {
    /// Today's drift trigger: --router-split-z (or, without it, the stall rule by --split-z). Unchanged.
    #[default]
    Z,
    /// Stall plus headroom: at the router's window close, it splits if its own routing error is stagnant (its block
    /// means improved by at most `stall_z` block standard errors for `stall_windows` closes in a row, out of
    /// back-off), every direct child has seen a window's examples, and the oracle headroom H (per example, the
    /// routed child's loss minus the best child's, before the router trains; 20-block standard error) exceeds
    /// `router_z` standard errors and the MDL bar. No shadows: the split is made from the router as it is.
    StallH,
    /// Stall-h's gate, plus the shadow test: while the last window's H cleared its bars the router's shadows run
    /// (copies of its router, one per part of a partition of its examples: their arrival path, or a label-free
    /// cluster router's class), each scored on an example before it trains on it; the router splits, into the
    /// shadows, only if their prequential gain G also exceeds `router_z` standard errors and the MDL bar.
    StallHg,
    /// growth-spec phase 2 as written: the router splits into its shadows (running from its first close) when its
    /// subtree has stalled (it is stagnant and stalled nodes hold `subtree_share` of its children's examples,
    /// recursively) and G clears its bars.
    Subtree,
}

/// grow2: the partition of a router's shadows (growth-spec 2.2).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum RouterShadow {
    /// By arrival path where the router has several (its parents are the members of one sibling set), else cluster.
    #[default]
    Auto,
    /// By arrival path only (a router with one arrival path never splits).
    Path,
    /// By a label-free cluster router's class, always.
    Cluster,
}

/// grow2: the n of the MDL bar and of the scheduler's tie-break (theory-2 6.41 note).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum MdlN {
    /// As grow1: a leaf trial's split node's examples (an undo resets them to the trial's), a router's examples since
    /// it was made or last split.
    #[default]
    Trial,
    /// The node's lifetime examples (`Node::life`), which nothing resets.
    Lifetime,
}

/// grow4: when a leaf may split on trial under the growth rule (theory-2 6.47 (1)).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum GrowGate {
    /// grow1's: the leaf is stagnant (`stall_windows` closes in a row without improvement) and out of back-off.
    #[default]
    Stall,
    /// No stagnation test (a league field drifts, so window-to-window changes measure the stream): the leaf has
    /// closed `age_windows` of its own windows and is out of back-off.
    Age,
}

/// grow4: which eligible leaf a scheduling point splits on trial (theory-2 6.47 (1)).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum GrowPriority {
    /// grow1's: the largest excess of the leaf's last window mean over the model's, then more examples, then the
    /// lower index.
    #[default]
    Excess,
    /// The smallest paired snapshot gain (the leaf's last window: its frozen copy from the close before minus the
    /// online leaf, on the same examples) first, "most stagnant first"; leaves without a measurement after those
    /// with one; ties as `Excess`.
    Snapshot,
}

/// grow5 (theory-2 6.62 (a)): what a growth-rule trial's shadow (the leaf without the split) starts as.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum GrowShadowForget {
    /// grow1-4's: a copy of the leaf forgotten by `forget_factor`, as the split's new leaves are.
    #[default]
    Split,
    /// The leaf as it is, unforgotten: the alternative to the split is the leaf as it would have continued. The new
    /// leaves still forget. An undone trial then leaves the leaf with its own sums carried through the trial.
    None,
}

/// grow1: the growth rule's options (see `SplitRule::Grow`). Used only with that rule; grow2: and with a router
/// split rule other than `RouterSplitRule::Z`.
#[derive(Clone, Copy, PartialEq)]
pub struct GrowParams {
    /// D: one credit per this many examples through the root.
    pub credit: u64,
    /// C: the most credits banked.
    pub credit_cap: u32,
    /// Z_S: a window improved if its mean fell by more than this many standard errors of the difference.
    pub stall_z: f64,
    /// K_S: a node is stagnant after this many closes in a row without improvement.
    pub stall_windows: u32,
    /// Z_T: a trial's gain must exceed this many block standard errors.
    pub trial_z: f64,
    /// M: the MDL bar's multiplier (0: no bar).
    pub mdl: f64,
    /// S: the share of a router's subtree that must be stalled (phase 2; leaves ignore it).
    pub subtree_share: f64,
    /// T: the most split trials in flight.
    pub trials: u32,
    /// Warn once the graph has this many nodes.
    pub warn_nodes: usize,
    /// grow2: the n of the MDL bar and of the scheduler's tie-break.
    pub mdl_n: MdlN,
    /// grow2: the router split rule (BDD with path context); `Z` is today's.
    pub router_rule: RouterSplitRule,
    /// grow2: Z_R, the standard errors H and G must exceed.
    pub router_z: f64,
    /// grow2: the shadows' partition.
    pub router_shadow: RouterShadow,
    /// grow2: log every router window close (H, G, their bars, the gate, the decision). Logging only.
    pub log_router_stats: bool,
    /// grow3 (theory-2 6.46 (iii) 1): stall-h and stall-hg judge a router once its children holding this share of
    /// their lifetime examples have each closed a window; 1 is grow2's guard (every child).
    pub guard_share: f64,
    /// grow4 (theory-2 6.47 (1)): the leaves' eligibility (stall: grow1's; age: `age_windows` own windows closed).
    pub gate: GrowGate,
    /// grow4: with `GrowGate::Age`, the own windows a leaf must have closed.
    pub age_windows: u32,
    /// grow4: which eligible leaf splits (excess: grow1's; snapshot: the smallest paired snapshot gain first).
    pub priority: GrowPriority,
    /// grow4 (theory-2 6.48): a trial is kept only if its gain also exceeds this many nats per example (0: no
    /// margin, grow1's verdict).
    pub min_gain: f64,
    /// grow5 (theory-2 6.62 (a)): whether a trial's shadow forgets at the split (split: grow1-4's) or starts as the
    /// leaf (none).
    pub shadow_forget: GrowShadowForget,
}

impl GrowParams {
    /// grow4: whether leaves measure their paired snapshot gain (under the age gate, or the snapshot priority).
    pub fn snapshot_on(&self) -> bool {
        self.gate == GrowGate::Age || self.priority == GrowPriority::Snapshot
    }

    /// grow4: whether a leaf with these statistics is eligible to split on trial.
    pub(crate) fn eligible(&self, s: &GrowStats) -> bool {
        match self.gate {
            GrowGate::Stall => s.eligible(self.stall_windows),
            GrowGate::Age => s.prev_exists && s.closes >= self.age_windows as u64 && s.closes >= s.eligible_after_close,
        }
    }
}

impl Default for GrowParams {
    fn default() -> Self {
        GrowParams {
            credit: 5_000_000,
            credit_cap: 2,
            stall_z: 2.0,
            stall_windows: 2,
            trial_z: 2.0,
            mdl: 1.0,
            subtree_share: 0.9,
            trials: 2,
            warn_nodes: 55,
            mdl_n: MdlN::Trial,
            router_rule: RouterSplitRule::Z,
            router_z: 2.0,
            router_shadow: RouterShadow::Auto,
            log_router_stats: false,
            guard_share: 1.0,
            gate: GrowGate::Stall,
            age_windows: 3,
            priority: GrowPriority::Excess,
            min_gain: 0.0,
            shadow_forget: GrowShadowForget::Split,
        }
    }
}

/// As `#[derive(Debug)]` wrote grow1's options, which state files record; grow2's appear only when they are used (not
/// `log_router_stats`, which does not change training), so grow1's options are recorded as before.
impl std::fmt::Debug for GrowParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("GrowParams");
        s.field("credit", &self.credit)
            .field("credit_cap", &self.credit_cap)
            .field("stall_z", &self.stall_z)
            .field("stall_windows", &self.stall_windows)
            .field("trial_z", &self.trial_z)
            .field("mdl", &self.mdl)
            .field("subtree_share", &self.subtree_share)
            .field("trials", &self.trials)
            .field("warn_nodes", &self.warn_nodes);
        if self.mdl_n != MdlN::Trial {
            s.field("mdl_n", &self.mdl_n);
        }
        if self.router_rule != RouterSplitRule::Z {
            s.field("router_rule", &self.router_rule)
                .field("router_z", &self.router_z)
                .field("router_shadow", &self.router_shadow);
        }
        if self.router_rule != RouterSplitRule::Z && self.guard_share != 1.0 {
            s.field("guard_share", &self.guard_share);
        }
        // grow4: its options only where they differ from grow1's behaviour, so grow1-3's states read as before.
        if self.gate != GrowGate::Stall {
            s.field("gate", &self.gate).field("age_windows", &self.age_windows);
        }
        if self.priority != GrowPriority::Excess {
            s.field("priority", &self.priority);
        }
        if self.min_gain != 0.0 {
            s.field("min_gain", &self.min_gain);
        }
        // grow5: only where it differs from grow1-4's behaviour, so their states read as before.
        if self.shadow_forget != GrowShadowForget::Split {
            s.field("shadow_forget", &self.shadow_forget);
        }
        s.finish()
    }
}

/// grow1: block accumulators (spec 0.1): a stream of values in blocks of a fixed length; the mean of each finished
/// block counts as one observation, so correlated neighbours (a game's positions) don't understate the noise.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Blocks {
    /// Sum and count of the current block's values.
    pub partial: f64,
    pub in_block: u32,
    /// Count, sum and sum of squares of the finished blocks' means.
    pub count: u32,
    pub sum: f64,
    pub sum_sq: f64,
}

impl Blocks {
    /// Adds a value, finishing a block every `len` values.
    pub fn add(&mut self, x: f64, len: u32) {
        self.partial += x;
        self.in_block += 1;
        if self.in_block >= len {
            let mean = self.partial / len as f64;
            self.count += 1;
            self.sum += mean;
            self.sum_sq += mean * mean;
            self.partial = 0.0;
            self.in_block = 0;
        }
    }

    /// The mean of the finished blocks' means and its standard error (the partial block is dropped).
    pub fn stats(&self) -> (f64, f64) {
        let k = self.count as f64;
        let mean = self.sum / k;
        let se = ((self.sum_sq / k - mean * mean).max(0.0) / (k - 1.0)).sqrt();
        (mean, se)
    }
}

/// grow1: a node's statistics for the growth rule (spec 0.1-0.2, 0.8): the current window's blocks of its errors,
/// the last closed window's, its stagnation streak and its back-off. Zero (Default) unless the rule is on.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct GrowStats {
    pub blocks: Blocks,
    /// The last closed window's mean and standard error (none before the first close), and the whole model's mean
    /// error over the same window (for a candidate's excess).
    pub prev_exists: bool,
    pub prev_mean: f64,
    pub prev_se: f64,
    pub prev_model_mean: f64,
    /// Closes in a row without improvement.
    pub stall_streak: u32,
    /// Windows closed.
    pub closes: u64,
    /// After an undone trial: its back-off in windows, and the close count from which the node may split again.
    pub backoff_windows: u32,
    pub eligible_after_close: u64,
    /// grow4 (`GrowParams::snapshot_on`): the current window's blocks of the paired snapshot differences (the frozen
    /// copy's loss minus the online leaf's, per scored example, both before the leaf trains on it), and the last
    /// finished window's mean and standard error (none before the first). Zero otherwise; in states only when on.
    pub snap_blocks: Blocks,
    pub snap_exists: bool,
    pub snap_gain: f64,
    pub snap_se: f64,
}

impl GrowStats {
    /// Starts afresh as a node a split made (spec 0.3), keeping the close count and the back-off, which only an undone
    /// trial reads (0.8).
    pub fn restart(&mut self) {
        *self = GrowStats {
            closes: self.closes,
            backoff_windows: self.backoff_windows,
            eligible_after_close: self.eligible_after_close,
            ..GrowStats::default()
        };
    }

    /// Closes the window (spec 0.2): its mean and standard error from the finished blocks, the improvement over the
    /// previous window and the bar `z` standard errors of their difference (None at the first close), and the
    /// streak. `model_mean` is the whole model's mean error over the window.
    pub fn close(&mut self, z: f64, model_mean: f64) -> (f64, f64, Option<(f64, f64)>) {
        let (mean, se) = self.blocks.stats();
        let change = self.prev_exists.then(|| (self.prev_mean - mean, z * (self.prev_se * self.prev_se + se * se).sqrt()));
        if let Some((improvement, bar)) = change {
            self.stall_streak = if improvement <= bar { self.stall_streak + 1 } else { 0 };
        }
        (self.prev_exists, self.prev_mean, self.prev_se, self.prev_model_mean) = (true, mean, se, model_mean);
        self.blocks = Blocks::default();
        self.closes += 1;
        (mean, se, change)
    }

    /// Whether the node is stagnant (`windows` closes in a row without improvement) and out of back-off.
    pub fn eligible(&self, windows: u32) -> bool {
        self.prev_exists && self.stall_streak >= windows && self.closes >= self.eligible_after_close
    }
}

/// grow1: the graph's credits (spec 0.5) and trials in flight.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct GrowGraph {
    pub credits: u32,
    /// The root example count at which the next credit comes.
    pub next_credit_at: u64,
    pub credits_spent: u64,
    pub credits_refunded: u64,
    pub trials_in_flight: u32,
}

/// grow2: a router's statistics under a router split rule (theory-2 6.40): the current window's blocks of its headroom
/// H and of its shadows' gain G, and its shadows. Empty (Default) otherwise.
#[derive(Clone, Debug, Default)]
pub(crate) struct RouterGrow {
    /// Per scored example, before the router trains on it: the loss of the child it routes to minus the least of its
    /// children's losses (the oracle's re-routing gain).
    pub h: Blocks,
    /// While the shadows run, per scored example: the loss of the child the router routes to minus that of the child
    /// the example's shadow picks, each before it trains on the example.
    pub g: Blocks,
    /// grow3, stall-hg (6.46 (iii) 2): the same gain for the null shadows (a hash partition of the same shape), and
    /// per example the real gain minus the null's, paired by block: the split's statistic.
    pub g_null: Blocks,
    pub g_diff: Blocks,
    /// Whether the shadows have run since the window opened, so that G covers the whole window.
    pub g_full: bool,
    pub shadows: Option<Box<Shadows>>,
}

/// grow2: a router's shadows (growth-spec 2.2): copies of its router and path table, one per part of a partition of
/// its examples, each trained, as the router trains on all of them, on its part's.
#[derive(Clone, Debug)]
pub(crate) struct Shadows {
    pub partition: Partition,
    pub routers: Vec<(NodeModel, PathTable)>,
    /// Whether they run (stall-hg pauses them while the router's headroom fails its bars).
    pub running: bool,
    /// grow3, stall-hg: the null shadows, copies of the router and its table like `routers` and as many, each
    /// trained on the examples of its part of a hash partition (as the cluster warm-up's); empty under subtree.
    pub null: Vec<(NodeModel, PathTable)>,
}

/// grow2: a shadows' partition of their router's examples.
#[derive(Clone, Debug)]
pub(crate) enum Partition {
    /// By arrival path: the parent the example came from, by its place in `parents` (the router's, in increasing order).
    Path { parents: Vec<usize> },
    /// By the class of `q`, a router trained label-free by hard EM as a cluster-routed delegator's is (its pick, or
    /// for its first `split_threshold / 5` examples a hash of the position), on `trained` examples so far.
    Cluster { q: NodeModel, trained: u64 },
}

impl Partition {
    pub fn kind(&self) -> SplitKind {
        match self {
            Partition::Path { .. } => SplitKind::Path,
            Partition::Cluster { .. } => SplitKind::Cluster,
        }
    }
}

/// grow2: how a router splits (growth-spec 2.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SplitKind {
    /// Unsharing by arrival path: each of its parents gets its own copy.
    Path,
    /// Into `branch_factor` copies under a new top router, as `Bdd::split_delegator`.
    Cluster,
}

/// grow2: a router split's post-split check (growth-spec 2.5): the nodes it made (an unsharing: the router itself,
/// which stands for its first path, and its copies), each one's first window once it closes (mean routing error, its
/// block standard error, examples), and the router's last window before the split.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PostCheck {
    pub router: usize,
    /// The training step of the split.
    pub at: u64,
    pub made: Vec<usize>,
    pub first: Vec<Option<(f64, f64, u64)>>,
    pub ref_mean: f64,
    pub ref_se: f64,
}

/// grow1: the MDL bar (spec 0.7), nats per example: `mdl` k ln(n) / (2 n) for k parameters judged at n examples.
pub(crate) fn mdl_bar(mdl: f64, k: usize, n: u64) -> f64 {
    if mdl == 0.0 || n == 0 {
        return 0.0;
    }
    let n = n as f64;
    mdl * k as f64 * n.ln() / (2.0 * n)
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

/// BDD with error routing: what a delegator whose children are sub-routers
/// made by one delegator split (sharing their children) learns.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum SubrouterTarget {
    /// Each sub-router's router error at predicting the error target of the
    /// shared children; it picks the sub-router that would route best.
    Routing,
    /// bddhome: the target its router had before the split, the error target
    /// of the shared children's own predictions (through chains of such
    /// splits, of the children the first router of the chain routed among).
    /// Its class i sends the position to sub-router i, whose home is shared
    /// child i. Right after a split the delegator and every sub-router are
    /// copies of the old router, so each sub-router picks its own home and
    /// the routing is the old router's; each sub-router then learns, on its
    /// own region only, where another shared child predicts better. Its
    /// targets never depend on the sub-routers, so it cannot tie between them.
    Home,
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
    /// Ordinal probit regression (see `probit`). Leaves only: it predicts the
    /// three results, not which child to delegate to.
    Probit,
    /// Tree-augmented naive Bayes (see `tan`), with a fixed m (`tan_m`):
    /// generative like CNBP, so it can be a cluster router.
    Tan,
}

/// The classifier at a node: a leaf's prediction of the classes, or a
/// delegator's router, which predicts which child will classify an input
/// best.
#[derive(Clone, Debug)]
pub enum NodeModel {
    Cnbp(Cnbp),
    Lr(Logreg),
    Probit(Probit),
    Tan(Tan),
}

impl NodeModel {
    pub fn kind(&self) -> NodeKind {
        match self {
            NodeModel::Cnbp(_) => NodeKind::Cnbp,
            NodeModel::Lr(_) => NodeKind::Lr,
            NodeModel::Probit(_) => NodeKind::Probit,
            NodeModel::Tan(_) => NodeKind::Tan,
        }
    }

    pub fn class_num(&self) -> usize {
        match self {
            NodeModel::Cnbp(m) => m.class_num(),
            NodeModel::Lr(m) => m.class_num(),
            NodeModel::Probit(_) => 3,
            NodeModel::Tan(m) => m.class_num(),
        }
    }

    /// Adds one data point. `label` is a distribution over classes.
    pub fn train(&mut self, data: &[u8], label: &[f64]) {
        match self {
            NodeModel::Cnbp(m) => m.train(data, label),
            NodeModel::Lr(m) => m.train(data, label),
            NodeModel::Probit(m) => m.train(data, label),
            NodeModel::Tan(m) => m.train(data, label),
        }
    }

    /// Writes the probability of each class.
    pub fn predict(&self, data: &[u8], out: &mut [f64]) {
        match self {
            NodeModel::Cnbp(m) => m.predict(data, out),
            NodeModel::Lr(m) => m.predict(data, out),
            NodeModel::Probit(m) => m.predict(data, out),
            NodeModel::Tan(m) => m.predict(data, out),
        }
    }

    /// Writes the log of each class's unnormalised probability.
    pub fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        match self {
            NodeModel::Cnbp(m) => m.predict_log(data, out),
            NodeModel::Lr(m) => m.predict_log(data, out),
            NodeModel::Probit(m) => m.predict_log(data, out),
            NodeModel::Tan(m) => m.predict_log(data, out),
        }
    }

    /// Prediction during training: a CNBP's is from its raw counts, so one
    /// with `use_probs` need not recalculate its probabilities every step.
    fn predict_training(&self, data: &[u8], out: &mut [f64]) {
        match self {
            NodeModel::Cnbp(m) => m.predict_counts(data, out),
            NodeModel::Lr(m) => m.predict(data, out),
            NodeModel::Probit(m) => m.predict(data, out),
            NodeModel::Tan(m) => m.predict_counts(data, out),
        }
    }

    /// Makes the model ready to predict after training.
    pub fn refresh(&mut self) {
        match self {
            NodeModel::Cnbp(m) => m.refresh(),
            NodeModel::Tan(m) => m.refresh(),
            NodeModel::Lr(_) | NodeModel::Probit(_) => {}
        }
    }

    /// Lets the model adapt faster while predicting the same: divides a
    /// CNBP's counts, or a logistic regression's AdaGrad sums, by `factor`.
    /// A probit's weights' variances are multiplied by it instead, which
    /// also makes it predict less confidently.
    pub fn forget(&mut self, factor: f64) {
        match self {
            NodeModel::Cnbp(m) => m.forget(factor),
            NodeModel::Lr(m) => m.forget(factor),
            NodeModel::Probit(m) => m.forget(factor),
            NodeModel::Tan(m) => m.forget(factor),
        }
    }

    /// Adds `var` to the variance of every weight of a probit (see
    /// `DelegationParams::split_noise`). Other classifiers have no
    /// variances: unchanged.
    fn widen(&mut self, var: f64) {
        if let NodeModel::Probit(m) = self {
            m.widen(var);
        }
    }

    /// grow1: the number of free parameters, for the MDL bar (spec 0.7): a logistic regression's weights (every
    /// class's, biases included), a probit's weights (one per input value and the bias) and its draw margin, a CNBP's
    /// counts and class totals, a TAN's CNBP part and conditional tables.
    pub fn num_params(&self) -> usize {
        match self {
            NodeModel::Cnbp(m) => m.num_params(),
            NodeModel::Lr(m) => m.num_params(),
            NodeModel::Probit(m) => m.num_params(),
            NodeModel::Tan(m) => m.num_params(),
        }
    }

    /// Saves the model on its own, as a CNBP or LR model file.
    pub fn save(&self, path: &std::path::Path) -> io::Result<()> {
        match self {
            NodeModel::Cnbp(m) => m.save(path),
            NodeModel::Lr(m) => m.save(path),
            NodeModel::Probit(m) => m.save(path),
            NodeModel::Tan(m) => m.save(path),
        }
    }

    /// A CNBP as its counts (the C format's node), a logistic regression or
    /// a probit as its own text format, which starts with the word "logreg"
    /// or "probit".
    pub(crate) fn write(&self, s: &mut String) {
        match self {
            NodeModel::Cnbp(m) => m.write_counts(s),
            NodeModel::Lr(m) => s.push_str(&m.to_text()),
            NodeModel::Probit(m) => s.push_str(&m.to_text()),
            NodeModel::Tan(m) if m.is_fresh() => m.write_text(s),
            NodeModel::Tan(m) => {
                let mut m = m.clone();
                m.refresh();
                m.write_text(s)
            }
        }
    }

    /// The kind and settings, which a resumed run's options must match.
    pub(crate) fn options(&self) -> String {
        match self {
            NodeModel::Cnbp(m) => format!("cnbp alpha {} use_probs {}", m.alpha(), m.use_probs()),
            NodeModel::Lr(m) => format!("lr rate {}", m.rate()),
            NodeModel::Probit(m) => format!("probit {:?}", m.params()),
            NodeModel::Tan(m) => format!("tan alpha {} m {:?} use_probs {}", m.alpha(), m.options(), m.use_probs()),
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
            NodeModel::Probit(m) => {
                w.u8(2);
                m.write_state(w)
            }
            NodeModel::Tan(m) => {
                w.u8(3);
                m.write_state_to(w)
            }
        }
    }

    /// Reads a classifier written by `write_state`.
    pub(crate) fn read_state(r: &mut Reader) -> io::Result<Self> {
        match r.u8()? {
            0 => Ok(NodeModel::Cnbp(Cnbp::read_state(r)?)),
            1 => Ok(NodeModel::Lr(Logreg::read_state(r)?)),
            2 => Ok(NodeModel::Probit(Probit::read_state(r)?)),
            3 => Ok(NodeModel::Tan(Tan::read_state_from(r)?)),
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
    /// The options of new probit leaves (`NodeKind::Probit`). Not saved in
    /// model files apart from the leaves' own; when loading, the first
    /// probit leaf's (the default if none).
    pub probit: ProbitParams,
    /// With probit leaves: a split's new leaves, and leaves reset to their
    /// delegator's shadow (`reset_leaves`), start from the posterior of the
    /// leaf they copy (forgotten by `forget_factor`) with this standard
    /// deviation added to every weight's in quadrature (variance plus
    /// noise^2): the hierarchical prior in which a child's weights are its
    /// parent's plus independent N(0, noise^2). The children predict less
    /// confidently at first and can move away from the parent quickly where
    /// their own examples disagree with it. Shadows and trial copies, which
    /// stand for the parent, get none. 0: plain copies. Not saved in model
    /// files; 0 when loading.
    pub split_noise: f64,
    /// With probit leaves: one draw margin for the whole tree. Every step,
    /// the leaf that trains on the example takes its draw margin's gradient
    /// step (see `probit`), from the margin all the tree's probits share, and
    /// every probit of the tree (leaves, shadows, trial copies) then takes
    /// the leaf's new margin: one margin learned from every example, by the
    /// leaf that predicts it, so that the leaves' log-odds of a win against a
    /// loss are on the same scale. Before the first split the tree is a
    /// plain probit. Not saved in model files (whose leaves have one margin
    /// anyway); false when loading.
    pub probit_shared_eps: bool,
    /// m of TAN nodes (`NodeKind::Tan`), fixed. Not saved in model files;
    /// when loading, that of the nodes.
    pub tan_m: f64,
    /// BDD with error routing: what delegators over sub-routers learn (see
    /// `SubrouterTarget`). Only training uses it: not saved in model files
    /// (`Routing` when loading), which therefore load and play as before.
    pub subrouter_target: SubrouterTarget,
    /// BDD: every node also has a path table, inputs that tell it which path delivered the position (see
    /// `pathctx`). Saved in model files, which binaries without it then refuse.
    pub path_context: bool,
    /// BDD (errdag): the router-split threshold. With it, a delegator splits only if its window's mean router
    /// error rose by more than this many standard errors over its previous window's (and is at least the routers'
    /// mean), in place of the stall rule by `split_z`; infinity never splits a delegator. With `split_z` 0, Some(0)
    /// is exactly the stall rule. Leaf splits are unchanged. Training only: not saved in model files (None when
    /// loading); in state files' options only when set.
    pub router_split_z: Option<f64>,
    /// BDD (errdag2): a separate budget for delegator (router) splits. With it, `split_limit` counts leaf splits
    /// only, delegator splits stop at this many, and a delegator may split only while fewer delegator splits than
    /// leaf splits have been made, so the DAG is the tree plus at most this many router splits. Without it, every
    /// split counts towards `split_limit`. Training only: not saved in model files (None when loading); in state
    /// files' options only when set.
    pub router_split_limit: Option<u32>,
    /// grow1: the growth rule's options, used only with `SplitRule::Grow` (and in state files' options only then).
    /// Training only: not saved in model files (the default when loading).
    pub grow: GrowParams,
}

/// As `#[derive(Debug)]` would write it, except that the residual options,
/// the probit leaves' options, the split noise and the sub-router target
/// appear only when they are used, so that the options trees without them
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
        // Only probit leaves use their options.
        if self.leaf == NodeKind::Probit {
            s.field("probit", &self.probit);
        }
        if self.split_noise != 0.0 {
            s.field("split_noise", &self.split_noise);
        }
        if self.probit_shared_eps {
            s.field("probit_shared_eps", &self.probit_shared_eps);
        }
        // Only TAN nodes use it.
        if [self.leaf, self.router, self.error_router].contains(&NodeKind::Tan) {
            s.field("tan_m", &self.tan_m);
        }
        if self.subrouter_target != SubrouterTarget::Routing {
            s.field("subrouter_target", &self.subrouter_target);
        }
        if self.path_context {
            s.field("path_context", &self.path_context);
        }
        if self.router_split_z.is_some() {
            s.field("router_split_z", &self.router_split_z);
        }
        if self.router_split_limit.is_some() {
            s.field("router_split_limit", &self.router_split_limit);
        }
        // grow1: only the growth rule uses its options; grow2: and the router split rules.
        if self.split_rule == SplitRule::Grow || self.grow.router_rule != RouterSplitRule::Z {
            s.field("grow", &self.grow);
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
            NodeKind::Probit => {
                assert_eq!(class_num, 3, "a probit predicts the three results: it cannot route");
                NodeModel::Probit(Probit::new(categories, self.probit))
            }
            NodeKind::Tan => NodeModel::Tan(Tan::new(
                class_num,
                categories,
                self.nb_alpha,
                TanOptions::fixed(self.tan_m),
                self.use_probs,
            )),
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
    /// With path context, for a node whose table has cells: the cells' values the cached output is for.
    pub cache_values: Vec<usize>,
    /// errdag4, with path context: the part of the output with cells that doesn't depend on the path (a CNBP's class
    /// products before the cells' factors, a logistic regression's raw log-scores, a probit's latent mean and
    /// variance), for the position of training step `base_step` (0: none). Dropped wherever `cache_step` is.
    pub base: [f64; 4],
    pub base_step: u64,
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
    /// With `DelegationParams::path_context`: the node's path table (see `pathctx`); None without.
    pub table: Option<Box<PathTable>>,
    /// With `DelegationParams::path_context`: the sibling set the node is a member of, if any, by registry index,
    /// and its number in the set (from 1). Derived from the registry (`Graph::sets`).
    pub member: Option<(usize, usize)>,
    /// grow1: the growth rule's statistics (see `SplitRule::Grow`); zero without it. In state files only with it.
    /// grow2: also a router's (its routing errors) under a router split rule, and a leaf's under `Subtree`.
    pub grow: GrowStats,
    /// grow2: the examples the node recorded in its life (`record`), which neither a split nor an undo resets (theory-2
    /// 6.41 note); the n of `MdlN::Lifetime`. In state files only where that or a router split rule reads it.
    pub life: u64,
    /// grow2: a router's headroom, shadows and gain under a router split rule; empty otherwise.
    pub rgrow: RouterGrow,
    /// grow4 (`GrowParams::snapshot_on`): a leaf's classifier as it was at its last window close, frozen, which
    /// scores the leaf's examples through the next window beside the online leaf. None otherwise (and for a leaf
    /// that has not closed a window since it was made or its trial was undone). In states only when on.
    pub snap: Option<Box<NodeModel>>,
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
    /// grow1: a growth-rule trial's blocks of the same differences (spec 1.2), which judge it; None for others.
    pub grow: Option<Blocks>,
}

impl Trial {
    pub fn new(shadow: NodeModel, children: Option<Vec<usize>>, trained_before: u32, made: Vec<usize>) -> Box<Self> {
        Box::new(Trial { shadow, children, trained_before, made, examples: 0, n: 0, sum: 0.0, sum_sq: 0.0, grow: None })
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
    pub(crate) fn new(kind: Kind, classifier: NodeModel) -> Self {
        Node {
            kind,
            cache: vec![0.0; classifier.class_num()],
            cache_step: 0,
            cache_values: Vec::new(),
            base: [0.0; 4],
            base_step: 0,
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
            table: None,
            member: None,
            grow: GrowStats::default(),
            life: 0,
            rgrow: RouterGrow::default(),
            snap: None,
        }
    }

    pub fn record(&mut self, error: f64) {
        self.num_trained_last += 1;
        self.num_trained_total += 1;
        self.life += 1;
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
    /// With `DelegationParams::path_context`: the registry of sibling sets (see `pathctx`), each its members' node
    /// ids, in the order of the router splits that made them.
    pub sets: Vec<Vec<usize>>,
    /// grow1: the growth rule's credits and trials in flight (see `SplitRule::Grow`); zero without it.
    pub grow: GrowGraph,
    /// grow2: under a router split rule, the post-split checks still waiting for their nodes' first windows, and the
    /// counts of checks made and of their warnings (growth-spec 2.5). In state files only under such a rule.
    pub post_checks: Vec<PostCheck>,
    pub post_checked: u64,
    pub post_warnings: u64,
    /// grow2: the router an unsharing just split this step, its parents then, and the nodes that now stand for it by
    /// parent, so the example goes on to its path's (`Bdd::train_node_ctx`). Not saved.
    pub unshared: Option<(usize, Vec<usize>, Vec<usize>)>,
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
        assert!(
            params.router != NodeKind::Probit && params.error_router != NodeKind::Probit,
            "probits predict the results, not which child predicts best: they cannot route"
        );
        assert!(
            params.split_noise >= 0.0 && (params.split_noise == 0.0 || params.leaf == NodeKind::Probit),
            "split_noise is a standard deviation (at least 0) for probit leaves"
        );
        assert!(!params.probit_shared_eps || params.leaf == NodeKind::Probit, "probit_shared_eps is for probit leaves");
        assert!(
            !params.path_context
                || (matches!(params.leaf, NodeKind::Lr | NodeKind::Cnbp | NodeKind::Probit)
                    && matches!(params.router, NodeKind::Lr | NodeKind::Cnbp)),
            "path context is for logistic-regression and CNBP nodes (and probit leaves)"
        );
        if params.split_rule == SplitRule::Grow {
            assert!(
                params.split_trial.is_none() && params.revise.is_none() && params.reset_leaves.is_none(),
                "the growth rule makes its own trials: no split_trial, revise or reset_leaves"
            );
            assert!(params.split_z == 0.0, "the growth rule has its own thresholds: split_z must be 0");
            assert!(params.trial_error == ErrorMeasure::Log, "the growth rule judges trials by log-loss (trial_error log)");
            assert!(!params.path_context, "the growth rule (phase 1) is for trees: no path_context");
            assert!(params.split_threshold >= 20, "the growth rule's windows have 20 blocks: split_threshold at least 20");
            assert!(params.grow.credit >= 1, "the growth rule's credit interval is at least 1 example");
        }
        // grow2: the router split rules are for error-routed DAGs with path context (theory-2 6.40-6.41).
        if params.grow.router_rule != RouterSplitRule::Z {
            assert!(
                params.path_context && params.route == RouteTarget::Error && params.route_switch.is_none(),
                "the router split rules are for error-routed DAGs with path context (cluster routers are out of scope: their examples do not score every child)"
            );
            assert!(params.subrouter_target == SubrouterTarget::Routing, "the router split rules need subrouter_target routing");
            assert!(params.router_split_z.is_none(), "a router split rule replaces router_split_z");
            assert!(params.error == ErrorMeasure::Log, "the router split rules measure in nats: error log");
            assert!(params.split_threshold >= 20, "the router split rules' windows have 20 blocks: split_threshold at least 20");
            assert!(params.split_rule != SplitRule::Grow, "the router split rules keep the leaf rule as it is: no split_rule grow");
        }
        let root = Node::new(Kind::Leaf, params.new_classifier(params.leaf, params.class_num, categories));
        let grow = match params.split_rule {
            SplitRule::Grow => GrowGraph { next_credit_at: params.grow.credit, ..GrowGraph::default() },
            _ => GrowGraph::default(),
        };
        let mut graph = Graph {
            categories: categories.to_vec(),
            params,
            split_number: 0,
            nodes: vec![root],
            step: 0,
            model_errors: Running::default(),
            router_errors: Running::default(),
            dead: Vec::new(),
            game: None,
            sets: Vec::new(),
            grow,
            post_checks: Vec::new(),
            post_checked: 0,
            post_warnings: 0,
            unshared: None,
        };
        graph.reset_root_table();
        graph
    }

    /// With path context, gives the root a path table for its classifier (no cells: nothing is above it).
    fn reset_root_table(&mut self) {
        if self.params.path_context {
            self.nodes[0].table = Some(Box::new(crate::pathctx::table_for(self, &self.nodes[0].classifier)));
        }
    }

    /// A graph whose root leaf starts as `root`, a classifier over the same
    /// features and classes, instead of an untrained one.
    pub fn starting_from(root: NodeModel, categories: &[u8], params: DelegationParams) -> Self {
        let mut graph = Graph::new(categories, params);
        graph.nodes[0].classifier = root;
        graph.reset_root_table();
        graph
    }

    /// Writes the probability of each class into `out`.
    ///
    /// # Panics
    /// With `use_probs`, if the model was trained since the last `refresh()`.
    pub fn predict(&self, data: &[u8], out: &mut [f64]) {
        if self.params.path_context {
            return self.predict_path(data, out, false);
        }
        self.nodes[self.leaf(data)].classifier.predict(data, out)
    }

    /// Writes the log of each class's unnormalised probability, from the leaf
    /// `predict` would use (see `Cnbp::predict_log`).
    pub fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        if self.params.path_context {
            return self.predict_path(data, out, true);
        }
        self.nodes[self.leaf(data)].classifier.predict_log(data, out)
    }

    /// `predict` (`log` false) or `predict_log` with path context: the leaf's prediction, each node on the way
    /// predicting with its path table for the position's path (see `pathctx`).
    fn predict_path(&self, data: &[u8], out: &mut [f64], log: bool) {
        let mut marks = Vec::with_capacity(16);
        let leaf = self.descend(data, &mut marks, |_, _| ());
        self.node_predict_path(leaf, &marks, data, out, log);
    }

    /// Node `idx`'s own prediction (`predict`, or `predict_log` if `log`) for a position whose path passed through
    /// the sibling sets `marks`: its classifier's, with its path table's cells (see `pathctx`) if it has any.
    pub(crate) fn node_predict_path(&self, idx: usize, marks: &[Mark], data: &[u8], out: &mut [f64], log: bool) {
        let node = &self.nodes[idx];
        let Some(table) = node.table.as_deref().filter(|t| !t.cells().is_empty()) else {
            return match log {
                true => node.classifier.predict_log(data, out),
                false => node.classifier.predict(data, out),
            };
        };
        let mut values = Vec::with_capacity(table.cells().len());
        table.values_for(marks, &mut values);
        let n = node.classifier.class_num();
        let (mut extra_small, mut extra_big) = ([0.0; 8], Vec::new());
        let extra = small_buffer(&mut extra_small, &mut extra_big, n);
        match &node.classifier {
            NodeModel::Lr(lr) => {
                table.lr_offsets(&values, extra);
                lr.predict_log(data, out);
                out[..n].iter_mut().zip(extra.iter()).for_each(|(z, e)| *z += e);
                if !log {
                    softmax(&mut out[..n]);
                }
            }
            NodeModel::Cnbp(nb) => {
                table.cnbp_factors(&values, nb.alpha(), extra);
                if log {
                    nb.predict_log(data, out);
                    out[..n].iter_mut().zip(extra.iter()).for_each(|(z, f)| *z += f.ln());
                } else {
                    nb.predict_scaled(data, extra, out);
                }
            }
            NodeModel::Probit(probit) => {
                let (mu, s2) = table.probit_parts(&values);
                probit.predict_with_cells(data, mu, s2, out, log);
            }
            _ => unreachable!("path context is for logistic-regression, CNBP and probit nodes"),
        }
    }

    /// With path context: routes the position from the root as play does, each router predicting with its path
    /// table. Returns the leaf; leaves in `marks` the sibling sets its path passed through, and calls `on_node` with
    /// each node of the path in turn and the marks so far (the node's own included).
    pub(crate) fn descend(&self, data: &[u8], marks: &mut Vec<Mark>, mut on_node: impl FnMut(usize, &[Mark])) -> usize {
        marks.clear();
        let (mut scores_small, mut scores_big) = ([0.0; 8], Vec::new());
        let scores = small_buffer(&mut scores_small, &mut scores_big, self.params.branch_factor);
        let mut idx = 0;
        loop {
            on_node(idx, marks);
            let Kind::Branch { children } = &self.nodes[idx].kind else { return idx };
            self.node_predict_path(idx, marks, data, scores, false);
            idx = children[argmax(scores)];
            if let Some(mark) = self.nodes[idx].member {
                marks.push(mark);
            }
        }
    }

    /// hybrid15: `descend`'s loop from node `idx`, which the position entered with `marks` (the marks of its path so
    /// far, the node's own included): each router predicts with its path table and the child of its argmax is
    /// entered, its mark pushed. Returns the leaf; leaves the path's marks in `marks`. From the root with no marks it
    /// is `descend`.
    pub(crate) fn descend_from(&self, mut idx: usize, data: &[u8], marks: &mut Vec<Mark>) -> usize {
        let (mut scores_small, mut scores_big) = ([0.0; 8], Vec::new());
        let scores = small_buffer(&mut scores_small, &mut scores_big, self.params.branch_factor);
        loop {
            let Kind::Branch { children } = &self.nodes[idx].kind else { return idx };
            self.node_predict_path(idx, marks, data, scores, false);
            idx = children[argmax(scores)];
            if let Some(mark) = self.nodes[idx].member {
                marks.push(mark);
            }
        }
    }

    /// The leaves' node indices and classifiers.
    pub fn leaves(&self) -> impl Iterator<Item = (usize, &NodeModel)> {
        self.nodes.iter().enumerate().filter(|(_, n)| matches!(n.kind, Kind::Leaf)).map(|(i, n)| (i, &n.classifier))
    }

    /// Writes node `idx`'s own prediction (see `NodeModel::predict_log`). With path context, for the position's own
    /// path, which must reach the node.
    pub fn node_predict_log(&self, idx: usize, data: &[u8], out: &mut [f64]) {
        if self.params.path_context {
            let mut upto = None;
            self.descend(data, &mut Vec::with_capacity(16), |node, marks| {
                if node == idx {
                    upto = Some(marks.to_vec());
                }
            });
            let upto = upto.expect("with path context, a node predicts only the positions whose path reaches it");
            return self.node_predict_path(idx, &upto, data, out, true);
        }
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
        assert!(!self.params.path_context, "with path context a leaf's prediction depends on the path: no blends");
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
        if self.params.path_context {
            return self.descend(data, &mut Vec::with_capacity(16), |_, _| ());
        }
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
            if let Some(table) = &mut node.table {
                table.forget(factor);
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
                node.base_step = 0;
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
        // hybrid6: a small buffer for the usual branch factors (no 2 KB to zero per call).
        let bf = self.params.branch_factor;
        let (mut small, mut big) = ([0.0; 16], Vec::new());
        let scores: &mut [f64] = if bf <= 16 {
            &mut small[..bf]
        } else {
            big.resize(bf, 0.0);
            &mut big
        };
        loop {
            match &self.nodes[idx].kind {
                Kind::Leaf => return self.node_output(idx, data, out),
                Kind::Branch { .. } => {
                    let scores = &mut scores[..bf];
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
            match &mut node.classifier {
                // hybrid6: the same output, reusing the classes a step's training left unchanged.
                NodeModel::Tan(m) => m.predict_counts_cached(data, &mut node.cache),
                model => model.predict_training(data, &mut node.cache),
            }
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
        if self.nodes[idx].num_trained_last < self.params.split_threshold || self.leaf_limit_reached() {
            return None;
        }
        if self.params.split_rule == SplitRule::Grow {
            return Some(self.grow_close(idx) == Some(idx));
        }
        let node = &mut self.nodes[idx];
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
                SplitRule::Grow => unreachable!("the growth rule closes its windows in grow_close"),
            };
        if split {
            self.split_leaf(idx);
        }
        self.open_window(idx);
        Some(split)
    }

    /// Whether the split limit stops leaf splits: every split counts towards it, or with `router_split_limit`
    /// (BDD) only leaf splits.
    fn leaf_limit_reached(&self) -> bool {
        match self.params.router_split_limit {
            None => self.split_number >= self.params.split_limit,
            Some(_) => self.split_counts().0 >= self.params.split_limit,
        }
    }

    /// (leaf splits, delegator splits) made so far. A leaf split turns a leaf into a delegator over
    /// `branch_factor` new leaves; a delegator split adds delegators, no leaves. So without merges, split trials
    /// or shared leaves (which `router_split_limit` refuses) the leaves count the leaf splits, and the rest of
    /// `split_number` are delegator splits.
    pub fn split_counts(&self) -> (u32, u32) {
        let leaves = self.nodes.iter().filter(|n| matches!(n.kind, Kind::Leaf)).count() as u32;
        let per_split = self.params.branch_factor as u32 - 1;
        debug_assert!(leaves >= 1 && (leaves - 1) % per_split == 0, "{leaves} leaves");
        let leaf_splits = (leaves - 1) / per_split;
        (leaf_splits, self.split_number - leaf_splits)
    }

    pub fn train_leaf(&mut self, idx: usize, data: &[u8], label: &[f64], error: Option<f64>) {
        let step = self.step;
        let node = &mut self.nodes[idx];
        match &mut node.classifier {
            // hybrid6: the prediction this step cached is the one `Logreg::train` computes first.
            NodeModel::Lr(lr) if node.cache_step == step && node.cache.len() == lr.class_num() => {
                lr.step(data, &node.cache, label, 1.0, 0.0)
            }
            model => model.train(data, label),
        }
        node.cache_step = 0;
        node.base_step = 0;
        // The leaf trains last in a step, after the shadows on its path.
        if self.params.probit_shared_eps {
            self.share_eps(idx);
        }
        // grow1: the window's blocks of the leaf's errors (spec 0.1).
        let grow_len = self.growing().then(|| self.grow_block_len());
        let node = &mut self.nodes[idx];
        if let Some(error) = error {
            node.record(error);
            if let Some(len) = grow_len {
                node.grow.blocks.add(error, len);
            }
        }
        // grow4: the paired snapshot difference (theory-2 6.47 (1)): the frozen copy's error on the example minus
        // the online leaf's (`error`, from its prediction before this step's training). The copy never trains, so
        // where it is scored doesn't matter. Only leaves with a snapshot (`GrowParams::snapshot_on`).
        if let (Some(error), Some(len)) = (error, grow_len) {
            if let Some(snap) = self.nodes[idx].snap.as_deref() {
                let class_num = snap.class_num();
                let (mut small, mut big) = ([0.0; 16], Vec::new());
                let out: &mut [f64] = if class_num <= 16 {
                    &mut small[..class_num]
                } else {
                    big.resize(class_num, 0.0);
                    &mut big
                };
                snap.predict_training(data, out);
                let d = self.error(out, label) - error;
                self.nodes[idx].grow.snap_blocks.add(d, len);
            }
        }
        if self.params.leaf_share > 0.0 {
            let share = self.params.leaf_share;
            for (i, node) in self.nodes.iter_mut().enumerate() {
                if i != idx && matches!(node.kind, Kind::Leaf) {
                    match &mut node.classifier {
                        NodeModel::Cnbp(cnbp) => cnbp.train_weighted(data, label, share),
                        // An AdaGrad step on `share` times the example's
                        // log-loss, from the leaf's own prediction.
                        NodeModel::Lr(lr) => {
                            let mut p = [0.0; 256];
                            let p = &mut p[..lr.class_num()];
                            lr.predict(data, p);
                            lr.step(data, p, label, share, 0.0);
                        }
                        _ => panic!("leaf_share needs CNBP or LR leaves"),
                    }
                    node.cache_step = 0;
                    node.base_step = 0;
                }
            }
        }
    }

    /// With `probit_shared_eps`: gives every probit of the tree the draw
    /// margin of leaf `idx`, which just trained.
    fn share_eps(&mut self, idx: usize) {
        let NodeModel::Probit(leaf) = &self.nodes[idx].classifier else { return };
        let eps = leaf.eps();
        let set = |model: &mut NodeModel| {
            if let NodeModel::Probit(m) = model {
                m.set_eps(eps);
            }
        };
        for node in &mut self.nodes {
            set(&mut node.classifier);
            if let Some(trial) = &mut node.trial {
                set(&mut trial.shadow);
            }
            if let Some(revision) = &mut node.revision {
                set(&mut revision.shadow);
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
        node.base_step = 0;
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
        let bf = self.params.branch_factor;
        let (mut small, mut big) = ([0.0; 16], Vec::new());
        let router: &mut [f64] = if bf <= 16 {
            &mut small[..bf]
        } else {
            big.resize(bf, 0.0);
            &mut big
        };
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
        let bf = self.params.branch_factor;
        let (mut small, mut big) = ([0.0; 16], Vec::new());
        let router: &mut [f64] = if bf <= 16 {
            &mut small[..bf]
        } else {
            big.resize(bf, 0.0);
            &mut big
        };
        self.node_output(idx, data, router);
        let router_error = self.router_error(router, target);
        match &mut self.nodes[idx].classifier {
            // hybrid6: `router` is the router's prediction this step, the one `Logreg::train` computes first.
            NodeModel::Lr(lr) if router.len() == lr.class_num() => lr.step(data, router, target, 1.0, 0.0),
            model => model.train(data, target),
        }
        self.nodes[idx].cache_step = 0;
        self.nodes[idx].base_step = 0;
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
            node.base_step = 0;
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
        // grow2: a router split rule's blocks of the router's routing errors (spec 0.1).
        if self.params.grow.router_rule != RouterSplitRule::Z {
            let len = self.grow_block_len();
            self.nodes[idx].grow.blocks.add(router_error, len);
        }
    }

    /// Widens a new leaf copied from the leaf it replaces by `split_noise`
    /// (see `DelegationParams::split_noise`).
    fn add_split_noise(&self, classifier: &mut NodeModel) {
        if self.params.split_noise > 0.0 {
            classifier.widen(self.params.split_noise * self.params.split_noise);
        }
    }

    /// Turns leaf `idx` into a delegator over `branch_factor` new leaves,
    /// copies of it that forget by `forget_factor` (see `NodeModel::forget`),
    /// so they predict the same but adapt faster (probit leaves: widened by
    /// `split_noise`). With `split_trial`, one
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
            self.add_split_noise(&mut classifier);
            let child = self.new_node(Kind::Leaf, classifier);
            self.nodes.push(child);
        }
        let made: Vec<usize> = (first..first + self.params.branch_factor).collect();
        let trial = self.params.split_trial.map(|_| {
            let mut shadow = self.nodes[idx].classifier.clone();
            shadow.forget(self.params.forget_factor);
            Trial::new(shadow, None, 0, made.clone())
        });
        // grow1: the growth rule tests every split (spec 1.2): the same shadow, the leaf's examples so far (its n at
        // the verdict, with the trial's), and blocks of the differences.
        let trial = match self.growing() {
            false => trial,
            true => {
                let mut shadow = self.nodes[idx].classifier.clone();
                // grow5: under --grow-shadow-forget none the shadow is the leaf as it is (theory-2 6.62 (a)).
                if self.params.grow.shadow_forget == GrowShadowForget::Split {
                    shadow.forget(self.params.forget_factor);
                }
                let mut trial = Trial::new(shadow, None, self.nodes[idx].num_trained_total, made.clone());
                trial.grow = Some(Blocks::default());
                self.grow.trials_in_flight += 1;
                Some(trial)
            }
        };
        let revision = self.revising().then(|| {
            let mut shadow = self.nodes[idx].classifier.clone();
            shadow.forget(self.params.forget_factor);
            Revision::new(shadow, self.params.branch_factor)
        });

        self.make_delegator(idx, made.clone(), trial, revision);
        if self.params.path_context {
            crate::pathctx::after_leaf_split(self, idx, &made);
        }
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
        node.base_step = 0;
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
        let mut shadow_predicted = false;
        if judged && (stream || self.params.log_fresh_gains) {
            self.predict_training(idx, data, scratch);
            let branch_error = measure(judge, scratch, label);
            revision.shadow.predict_training(data, scratch);
            shadow_predicted = true;
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
            match &mut revision.shadow {
                // hybrid6: `scratch` still holds the shadow's prediction of this example (nothing since wrote
                // it or trained the shadow), the one `Logreg::train` computes first.
                NodeModel::Lr(lr) if shadow_predicted && scratch.len() == lr.class_num() => {
                    lr.step(data, scratch, label, 1.0, 0.0)
                }
                shadow => shadow.train(data, label),
            }
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
                    self.add_split_noise(&mut classifier);
                    (classifier, None)
                }
            };
            let node = &mut self.nodes[leaf];
            node.classifier = classifier;
            node.residual = residual;
            node.cache_step = 0;
            node.base_step = 0;
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
        node.base_step = 0;
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
            if let Some(blocks) = &mut trial.grow {
                blocks.add(d, self.grow_block_len());
            }
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
        let grow = trial.grow.is_some();
        let keep = match grow {
            // grow1: the growth rule's verdict (spec 1.3).
            true => self.grow_verdict(idx, &trial, mean, se),
            false => {
                let keep = mean < -self.params.split_trial.unwrap_or(0.0) * se;
                let kind = if trial.children.is_some() { "delegator" } else { "leaf" };
                let outcome = if keep { "kept" } else { "undone" };
                eprintln!("split trial of {kind} {idx}: {outcome}, error vs unsplit {mean:+.7} (standard error {se:.7}, {} of {} examples)", trial.n, trial.examples);
                keep
            }
        };
        if grow {
            self.grow.trials_in_flight -= 1;
        }
        if keep {
            if grow {
                self.grow_log_nodes();
            }
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
        node.base_step = 0;
        node.error_total = 0.0;
        node.forget_window();
        self.open_window(idx);
        self.split_number -= 1;
        if grow {
            self.grow_undone(idx);
            self.grow_log_nodes();
        }
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
    /// regression and probit nodes in their own formats. Children may be
    /// shared (BDD).
    pub fn to_text(&self) -> String {
        let p = &self.params;
        let mut s = String::new();
        if p.path_context {
            // The marker, then the registry of sibling sets: each set's index and its members' node ids.
            writeln!(s, "{PATH_CONTEXT_MARKER} {PATH_CONTEXT_VERSION} {}", self.sets.len()).unwrap();
            for (set, members) in self.sets.iter().enumerate() {
                write!(s, "{set} {}", members.len()).unwrap();
                members.iter().for_each(|m| write!(s, " {m}").unwrap());
                s.push('\n');
            }
        }
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
            if p.path_context {
                node.table.as_ref().expect("with path context every node has a path table").write_text(&mut s);
            }
        }
        s
    }

    /// Parses the C text format (`bdt_from_file`), with logistic regression
    /// and probit nodes in their own formats. The node ids stored in the file must match
    /// their positions, as the C code always writes them.
    pub fn from_text(text: &str, use_probs: bool) -> io::Result<Self> {
        let mut t = Tokens::new(text);
        // A model with path context starts with a marker, which binaries without it cannot parse.
        let path_context = t.peek() == Some(PATH_CONTEXT_MARKER);
        let mut sets = Vec::new();
        if path_context {
            t.next::<String>()?;
            let version: u32 = t.next()?;
            if version != PATH_CONTEXT_VERSION {
                return Err(invalid(format!("path context format {version}, expected {PATH_CONTEXT_VERSION}")));
            }
            let set_num: usize = t.next()?;
            for set in 0..set_num {
                if t.next::<usize>()? != set {
                    return Err(invalid(format!("sibling set {set} is out of order in the registry")));
                }
                let member_num: usize = t.next()?;
                sets.push((0..member_num).map(|_| t.next()).collect::<io::Result<Vec<usize>>>()?);
            }
        }
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
            } else if t.peek() == Some("tan") {
                NodeModel::Tan(Tan::from_tokens(&mut t, classes, &categories, nb_alpha, use_probs)?)
            } else if t.peek() == Some("probit") {
                let probit = Probit::from_tokens(&mut t)?;
                if !matches!(kind, Kind::Leaf) || class_num != 3 {
                    return Err(invalid(format!("node {n} is a probit, which only leaves of 3 classes can be")));
                }
                NodeModel::Probit(probit)
            } else {
                let mut cnbp = Cnbp::new(classes, &categories, nb_alpha, use_probs);
                cnbp.read_counts(&mut t)?;
                NodeModel::Cnbp(cnbp)
            };
            let table = match path_context {
                false => None,
                true => Some(Box::new(PathTable::read_text(&mut t)?)),
            };
            nodes.push(Node {
                num_trained_total,
                num_trained_last,
                error_total,
                error_last,
                error_prev,
                table,
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
                NodeModel::Cnbp(_) | NodeModel::Probit(_) | NodeModel::Tan(_) => None,
            })
            .unwrap_or(0.05);
        let probit = nodes
            .iter()
            .find_map(|n| match &n.classifier {
                NodeModel::Probit(m) => Some(m.params()),
                NodeModel::Cnbp(_) | NodeModel::Lr(_) | NodeModel::Tan(_) => None,
            })
            .unwrap_or_default();
        let tan_m = nodes
            .iter()
            .find_map(|n| match &n.classifier {
                NodeModel::Tan(m) => Some(m.m()),
                _ => None,
            })
            .unwrap_or(crate::tan::DEFAULT_M);
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
            probit,
            split_noise: 0.0,
            probit_shared_eps: false,
            tan_m,
            subrouter_target: SubrouterTarget::Routing,
            path_context,
            router_split_z: None,
            router_split_limit: None,
            grow: GrowParams::default(),
        };
        let mut graph = Graph {
            categories,
            params,
            split_number,
            nodes,
            step: 0,
            model_errors: Running::default(),
            router_errors: Running::default(),
            dead: Vec::new(),
            game: None,
            sets,
            grow: GrowGraph::default(),
            post_checks: Vec::new(),
            post_checked: 0,
            post_warnings: 0,
            unshared: None,
        };
        if path_context {
            graph.register_members()?;
        }
        Ok(graph)
    }
}

/// The first token of a model file with path context, and the version of its format.
const PATH_CONTEXT_MARKER: &str = "pathcontext";
const PATH_CONTEXT_VERSION: u32 = 1;

#[cfg(test)]
thread_local! {
    /// Test switch: path tables do not train, so they stay neutral and path context must train exactly as without.
    pub(crate) static FROZEN_TABLES: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Path context (see `pathctx`): training with the position's path, given as the sibling sets it passed through
/// (`marks`; a node's own mark, pushed on entering it, never concerns its own table). Without cells anywhere, each
/// function does exactly what its counterpart without path context does.
impl Graph {
    /// Sets each node's membership from the registry and checks the graph (see `pathctx::check`).
    fn register_members(&mut self) -> io::Result<()> {
        for (set, members) in self.sets.iter().enumerate() {
            for (j, &m) in members.iter().enumerate() {
                if m >= self.nodes.len() {
                    return Err(invalid(format!("sibling set {set} has a member out of range")));
                }
                self.nodes[m].member = Some((set, j + 1));
            }
        }
        crate::pathctx::check(self)
    }

    /// On entering node `idx`: pushes its mark, if it is a member of a sibling set.
    pub fn enter(&self, idx: usize, marks: &mut Vec<Mark>) {
        if let Some(mark) = self.nodes[idx].member {
            marks.push(mark);
        }
    }

    /// Node `idx`'s cells' values for a path that passed through `marks`; empty for a node without cells.
    fn cell_values(&self, idx: usize, marks: &[Mark], out: &mut Vec<usize>) {
        match &self.nodes[idx].table {
            Some(table) => table.values_for(marks, out),
            None => out.clear(),
        }
    }

    /// `node_output` with the node's path table. A node without cells uses `node_output`; one with cells caches its
    /// output for the step and the cells' values.
    pub fn node_output_ctx(&mut self, idx: usize, marks: &[Mark], data: &[u8], out: &mut [f64]) {
        let mut values = Vec::new();
        self.cell_values(idx, marks, &mut values);
        if values.is_empty() {
            return self.node_output(idx, data, out);
        }
        let step = self.step;
        let node = &mut self.nodes[idx];
        if node.cache_step != step || node.cache_values != values {
            let table = node.table.as_deref().expect("a node with cells has a table");
            let n = node.classifier.class_num();
            let (mut extra_small, mut extra_big) = ([0.0; 8], Vec::new());
            let extra = small_buffer(&mut extra_small, &mut extra_big, n);
            // errdag4: within a step every call is for the same position, so the part of the output that doesn't
            // depend on the path is computed once per node and step (`base`), by the same code from the same inputs;
            // the cells then enter by the same operations in the same order as before. It is dropped wherever the
            // output cache is (`cache_step = 0`), i.e. wherever the node's classifier changes. Up to 4 classes.
            let cached = n <= 4;
            let fresh = cached && node.base_step == step;
            match &node.classifier {
                NodeModel::Lr(lr) => {
                    table.lr_offsets(&values, extra);
                    if cached {
                        if !fresh {
                            lr.predict_log(data, &mut node.base[..n]);
                            node.base_step = step;
                        }
                        node.cache.copy_from_slice(&node.base[..n]);
                    } else {
                        lr.predict_log(data, &mut node.cache);
                    }
                    node.cache.iter_mut().zip(extra.iter()).for_each(|(z, e)| *z += e);
                    softmax(&mut node.cache);
                }
                NodeModel::Cnbp(nb) => {
                    table.cnbp_factors(&values, nb.alpha(), extra);
                    if cached {
                        if !fresh {
                            nb.class_products(data, &mut node.base[..n]);
                            node.base_step = step;
                        }
                        node.cache.copy_from_slice(&node.base[..n]);
                        nb.scale_normalise(extra, &mut node.cache);
                    } else {
                        nb.predict_counts_scaled(data, extra, &mut node.cache);
                    }
                }
                NodeModel::Probit(probit) => {
                    let (mu, s2) = table.probit_parts(&values);
                    if cached {
                        if !fresh {
                            let (m, var) = probit.latent_base(data);
                            node.base[0] = m;
                            node.base[1] = var;
                            node.base_step = step;
                        }
                        probit.predict_from_base(node.base[0], node.base[1], mu, s2, &mut node.cache);
                    } else {
                        probit.predict_with_cells(data, mu, s2, &mut node.cache, false);
                    }
                }
                _ => unreachable!("path context is for logistic-regression, CNBP and probit nodes"),
            }
            node.cache_step = step;
            node.cache_values = values;
        }
        out.copy_from_slice(&node.cache);
    }

    /// `predict_training` from node `idx`, which the position entered with `marks`; the marks are as they were
    /// on return.
    pub fn predict_training_ctx(&mut self, mut idx: usize, marks: &mut Vec<Mark>, data: &[u8], out: &mut [f64]) {
        let len = marks.len();
        let (mut scores_small, mut scores_big) = ([0.0; 8], Vec::new());
        let bf = self.params.branch_factor;
        loop {
            match &self.nodes[idx].kind {
                Kind::Leaf => break self.node_output_ctx(idx, marks, data, out),
                Kind::Branch { .. } => {
                    let scores = small_buffer(&mut scores_small, &mut scores_big, bf);
                    self.node_output_ctx(idx, marks, data, scores);
                    let Kind::Branch { children } = &self.nodes[idx].kind else { unreachable!() };
                    idx = children[argmax(scores)];
                    self.enter(idx, marks);
                }
            }
        }
        marks.truncate(len);
    }

    /// The error of the prediction of the subtree of node `idx`, which the position entered with `marks`.
    pub fn subtree_error_ctx(&mut self, idx: usize, marks: &mut Vec<Mark>, data: &[u8], label: &[f64], scratch: &mut [f64]) -> f64 {
        self.predict_training_ctx(idx, marks, data, scratch);
        self.error(scratch, label)
    }

    /// `begin_step` with path context.
    pub fn begin_step_ctx(&mut self, data: &[u8], label: &[f64], scratch: &mut [f64]) -> Option<f64> {
        self.remove_dead();
        self.step += 1;
        self.predict_training_ctx(0, &mut Vec::new(), data, scratch);
        let error = self.scored(label).then(|| self.error(scratch, label));
        if let Some(error) = error {
            self.model_errors.add(error);
        }
        error
    }

    /// `route` with the node's path table.
    pub fn route_ctx(&mut self, idx: usize, marks: &[Mark], data: &[u8]) -> usize {
        let (mut router_small, mut router_big) = ([0.0; 8], Vec::new());
        let router = small_buffer(&mut router_small, &mut router_big, self.params.branch_factor);
        self.node_output_ctx(idx, marks, data, router);
        argmax(router)
    }

    /// Trains node `idx`'s classifier on `label` together with its path table's cells, whose values for the
    /// position are `values`: a logistic regression takes one AdaGrad step on the log-loss of its prediction with the
    /// cells, for its own weights and the cells' alike; a CNBP counts the example in both; a probit's ADF update moves
    /// its own weights and the cells' alike (`Probit::train_with_cells`). A node without cells trains as without path
    /// context.
    fn train_with_table(&mut self, idx: usize, values: &[usize], data: &[u8], label: &[f64]) {
        let node = &mut self.nodes[idx];
        let table = node.table.as_deref_mut().expect("with path context every node has a path table");
        #[cfg(test)]
        if FROZEN_TABLES.with(|f| f.get()) {
            return node.classifier.train(data, label);
        }
        if values.is_empty() {
            return node.classifier.train(data, label);
        }
        match &mut node.classifier {
            NodeModel::Lr(lr) => {
                let n = lr.class_num();
                let (mut p_small, mut p_big, mut extra_small, mut extra_big) = ([0.0; 8], Vec::new(), [0.0; 8], Vec::new());
                let (p, extra) = (small_buffer(&mut p_small, &mut p_big, n), small_buffer(&mut extra_small, &mut extra_big, n));
                table.lr_offsets(values, extra);
                lr.predict_log(data, p);
                p.iter_mut().zip(extra.iter()).for_each(|(z, e)| *z += e);
                softmax(p);
                lr.step(data, p, label, 1.0, 0.0);
                table.lr_step(values, p, label, lr.rate());
            }
            NodeModel::Cnbp(nb) => {
                nb.train(data, label);
                table.cnbp_add(values, label);
            }
            NodeModel::Probit(probit) => {
                let mut cells = table.probit_cells(values);
                probit.train_with_cells(data, label, &mut cells);
                table.set_probit_cells(values, &cells);
            }
            _ => unreachable!("path context is for logistic-regression, CNBP and probit nodes"),
        }
    }

    /// `train_leaf` with the leaf's path table (path context refuses leaf sharing and shared probit margins).
    pub fn train_leaf_ctx(&mut self, idx: usize, marks: &[Mark], data: &[u8], label: &[f64], error: Option<f64>) {
        let mut values = Vec::new();
        self.cell_values(idx, marks, &mut values);
        self.train_with_table(idx, &values, data, label);
        let node = &mut self.nodes[idx];
        node.cache_step = 0;
        node.base_step = 0;
        if let Some(error) = error {
            node.record(error);
            // grow2: the subtree rule's leaf statistics.
            if self.params.grow.router_rule == RouterSplitRule::Subtree {
                self.leaf_grow_add(idx, error);
            }
        }
    }

    /// `train_router` (error routing) with the router's path table.
    pub fn train_router_ctx(&mut self, idx: usize, marks: &[Mark], data: &[u8], child_errors: &[f64]) -> (usize, f64) {
        assert!(self.params.route == RouteTarget::Error, "path context is for error routing");
        let target = self.error_target(child_errors);
        let (mut router_small, mut router_big) = ([0.0; 8], Vec::new());
        let router = small_buffer(&mut router_small, &mut router_big, self.params.branch_factor);
        self.node_output_ctx(idx, marks, data, router);
        let router_error = self.router_error(router, &target);
        let mut values = Vec::new();
        self.cell_values(idx, marks, &mut values);
        self.train_with_table(idx, &values, data, &target);
        self.nodes[idx].cache_step = 0;
        self.nodes[idx].base_step = 0;
        (self.route_ctx(idx, marks, data), router_error)
    }

    /// hybrid14: whether any node's path table has cells (path context after a router split). Without, every node
    /// predicts with its classifier alone, exactly as without path context.
    pub fn has_cells(&self) -> bool {
        self.params.path_context && self.nodes.iter().any(|n| n.table.as_ref().is_some_and(|t| !t.cells().is_empty()))
    }

    /// The number of path table parameters (see `PathTable::param_count`) over all nodes.
    pub fn path_param_count(&self) -> usize {
        self.nodes.iter().filter_map(|n| n.table.as_ref()).map(|t| t.param_count()).sum()
    }
}

/// grow1: the growth rule, phase 1 (leaves; models/from-scratch/theory-2/growth-spec.md sections 0 and 1). Every
/// function here runs only with `SplitRule::Grow`.
impl Graph {
    /// Whether the growth rule is on.
    pub(crate) fn growing(&self) -> bool {
        self.params.split_rule == SplitRule::Grow
    }

    /// The length of a window's blocks (spec 0.1): a twentieth of `split_threshold`.
    pub(crate) fn grow_block_len(&self) -> u32 {
        (self.params.split_threshold / 20).max(1)
    }

    /// After every example through the root (spec 0.5): a credit per `credit` examples, at most `credit_cap`.
    /// Nothing without the growth rule.
    pub fn end_step(&mut self) {
        if !self.growing() {
            return;
        }
        let g = self.params.grow;
        while self.step >= self.grow.next_credit_at {
            self.grow.credits = (self.grow.credits + 1).min(g.credit_cap);
            self.grow.next_credit_at += g.credit;
            eprintln!(
                "grow credit at N {}: credits {} (cap {}), next at {}",
                self.step, self.grow.credits, g.credit_cap, self.grow.next_credit_at
            );
        }
    }

    /// A scheduling point (spec 0.6) at the close of leaf `idx`'s window: closes it (0.2), opens the next, and splits
    /// the best candidate on trial if a credit and a trial slot are free. Returns the leaf that split, if any.
    fn grow_close(&mut self, idx: usize) -> Option<usize> {
        let z = self.params.grow.stall_z;
        let windows = self.params.grow.stall_windows;
        let n = self.grow_n(idx);
        let node = &mut self.nodes[idx];
        // The window's per-example statistics as before (`error_prev`, which model files hold).
        node.close_window(0.0);
        let model_mean = self.model_errors.mean_since(node.window_model);
        let blocks = node.grow.blocks.count;
        let (mean, se, change) = node.grow.close(z, model_mean);
        let s = node.grow;
        let change = match change {
            Some((improvement, bar)) => format!("improvement {improvement:+.7} vs bar {bar:.7}"),
            None => "first window".to_string(),
        };
        eprintln!(
            "grow close of leaf {idx} at N {}: window mean {mean:.7} (block se {se:.7} over {blocks} blocks), {change}, streak {}{}, excess {:+.7}, n {}, closes {}{}",
            self.step,
            s.stall_streak,
            if s.prev_exists && s.stall_streak >= windows { " (stagnant)" } else { "" },
            mean - model_mean,
            n,
            s.closes,
            if s.closes < s.eligible_after_close { format!(" (backing off until close {})", s.eligible_after_close) } else { String::new() },
        );
        if self.params.grow.snapshot_on() {
            self.grow_snapshot_close(idx);
        }
        self.open_window(idx);
        self.grow_schedule(idx)
    }

    /// grow4 (theory-2 6.47 (1)): at leaf `idx`'s window close, ends its paired snapshot measurement (the mean and
    /// block standard error of the window's snapshot differences, logged) if a snapshot scored the window, and freezes
    /// a copy of the leaf as it is now, for the next window. Only with `GrowParams::snapshot_on`.
    fn grow_snapshot_close(&mut self, idx: usize) {
        let step = self.step;
        let node = &mut self.nodes[idx];
        let b = node.grow.snap_blocks;
        if node.snap.is_some() && b.count >= 2 {
            let (mean, se) = b.stats();
            (node.grow.snap_exists, node.grow.snap_gain, node.grow.snap_se) = (true, mean, se);
            eprintln!(
                "grow snapshot of leaf {idx} at N {step}: gain {mean:+.7} (block se {se:.7} over {} blocks, z {:+.2})",
                b.count,
                if se > 0.0 { mean / se } else { 0.0 }
            );
        }
        node.grow.snap_blocks = Blocks::default();
        node.snap = Some(Box::new(node.classifier.clone()));
    }

    /// Step 4 of a scheduling point at leaf `at`'s close (spec 0.6, phase 1): with a credit and fewer than `trials`
    /// trials in flight, the stagnant leaf out of back-off and not made by a split still on trial whose last window's
    /// mean is furthest above the model's over the same window (ties: more examples, then the lower index) splits on
    /// trial, spending a credit. One split at most. Returns it.
    pub(crate) fn grow_schedule(&mut self, at: usize) -> Option<usize> {
        let g = self.params.grow;
        let (credits, in_flight) = (self.grow.credits, self.grow.trials_in_flight);
        if credits < 1 || in_flight >= g.trials {
            eprintln!(
                "grow schedule at leaf {at}'s close: credits {credits}, trials in flight {in_flight} of {}: {}",
                g.trials,
                if credits < 1 { "no credit" } else { "no trial slot" }
            );
            return None;
        }
        let on_trial: Vec<usize> = self.nodes.iter().filter_map(|n| n.trial.as_ref()).flat_map(|t| t.made.iter().copied()).collect();
        let by_snapshot = g.priority == GrowPriority::Snapshot;
        let mut best: Option<(usize, f64, u64, Option<f64>)> = None;
        let mut candidates = Vec::new();
        for (i, node) in self.nodes.iter().enumerate() {
            // grow4: the gate (stall: grow1's stagnation; age: own windows closed).
            if !matches!(node.kind, Kind::Leaf) || self.dead.contains(&i) || on_trial.contains(&i) || !g.eligible(&node.grow) {
                continue;
            }
            let excess = node.grow.prev_mean - node.grow.prev_model_mean;
            // grow2: by `MdlN` (trial: as grow1).
            let n = self.grow_n(i);
            // grow4: with the snapshot priority, the leaf's last paired snapshot gain (none before its first).
            let snap = (by_snapshot && node.grow.snap_exists).then_some(node.grow.snap_gain);
            match by_snapshot {
                false => candidates.push(format!("{i} ({excess:+.5}, n {n})")),
                true => candidates.push(format!(
                    "{i} ({excess:+.5}, n {n}, snapshot {})",
                    snap.map_or("none".to_string(), |s| format!("{s:+.5}"))
                )),
            }
            // grow1's order: the larger excess, then more examples (ties: the lower index, the first seen). grow4's
            // snapshot priority: a measured leaf before one without, the smaller gain first, then grow1's order.
            let better = |(_, e, m, s): (usize, f64, u64, Option<f64>)| match (snap, s) {
                (Some(a), Some(b)) if a != b => a < b,
                (Some(_), None) => true,
                (None, Some(_)) => false,
                _ => excess > e || (excess == e && n > m),
            };
            if best.is_none_or(better) {
                best = Some((i, excess, n, snap));
            }
        }
        let Some((v, excess, _, snap)) = best else {
            eprintln!("grow schedule at leaf {at}'s close: credits {credits}, trials in flight {in_flight}: no candidate (the credit waits)");
            return None;
        };
        eprintln!(
            "grow schedule at leaf {at}'s close: credits {credits}, trials in flight {in_flight}, candidates {}: leaf {v} (excess {excess:+.7}{})",
            candidates.join(", "),
            match (by_snapshot, snap) {
                (false, _) => String::new(),
                (true, Some(s)) => format!(", snapshot gain {s:+.7}"),
                (true, None) => ", no snapshot gain yet".to_string(),
            }
        );
        self.split_leaf(v);
        self.grow_restart(v);
        self.grow.credits -= 1;
        self.grow.credits_spent += 1;
        let Kind::Branch { children } = &self.nodes[v].kind else { unreachable!("a split leaf is a delegator") };
        eprintln!(
            "grow trial start: leaf {v} splits into {children:?} at N {} (k {}), credits {}, trials in flight {}",
            self.step,
            self.split_params_added(v),
            self.grow.credits,
            self.grow.trials_in_flight
        );
        self.grow_log_nodes();
        Some(v)
    }

    /// The split node `v` and its windows restart (spec 0.3): `forget_window`, a new window, fresh statistics (its
    /// close count and back-off stay, for an undone trial's back-off, 0.8).
    fn grow_restart(&mut self, v: usize) {
        self.nodes[v].forget_window();
        self.open_window(v);
        self.nodes[v].grow.restart();
        // grow4: a delegator now; an undone trial's leaf starts its snapshots afresh at its next close.
        self.nodes[v].snap = None;
    }

    /// k of the MDL bar for the split of node `v` (spec 0.7): its children's new parameters beyond one, and its
    /// router's: (B - 1) d_leaf + d_router.
    pub(crate) fn split_params_added(&self, v: usize) -> usize {
        let Kind::Branch { children } = &self.nodes[v].kind else { unreachable!("a split node is a delegator") };
        let leaf = self.nodes[children[0]].classifier.num_params();
        (children.len() - 1) * leaf + self.nodes[v].classifier.num_params()
    }

    /// The verdict on node `idx`'s growth trial (spec 1.3): kept iff its gain, minus the mean of the blocks' mean
    /// differences of its errors from the shadow's, is above `trial_z` block standard errors and above the MDL bar at
    /// the node's lifetime examples. Logs it; `mean` and `se` are the per-example statistics, for the log.
    fn grow_verdict(&self, idx: usize, trial: &Trial, mean: f64, se: f64) -> bool {
        let g = self.params.grow;
        let blocks = trial.grow.expect("a growth trial has blocks");
        let (mean_d, se_g) = blocks.stats();
        let gain = -mean_d;
        // grow2: the node's lifetime examples with `MdlN::Lifetime` (theory-2 6.41 note), which an undo leaves as
        // they are; else as grow1, the node's examples before the split (an undo resets them to the trial's) and the
        // trial's.
        let n = match g.mdl_n {
            MdlN::Trial => trial.trained_before as u64 + trial.examples as u64,
            MdlN::Lifetime => self.nodes[idx].life,
        };
        let k = self.split_params_added(idx);
        let (bar_z, bar_mdl) = (g.trial_z * se_g, mdl_bar(g.mdl, k, n));
        // grow4 (theory-2 6.48): and the size margin, when there is one.
        let keep = gain > bar_z && gain > bar_mdl && (g.min_gain == 0.0 || gain > g.min_gain);
        #[cfg(test)]
        LAST_VERDICT.with(|v| v.set(Some((gain, se_g, bar_mdl, keep))));
        #[cfg(test)]
        LAST_VERDICT_N.with(|v| v.set(Some(n)));
        eprintln!(
            "grow trial of leaf {idx} at N {}: {}, gain {gain:+.7} (block se {se_g:.7} over {} blocks, z {:+.2}), bars: z {bar_z:.7}, mdl {bar_mdl:.7} (k {k}, n {n}){}; per example {mean:+.7} (se {se:.7}), {} of {} examples",
            self.step,
            if keep { "kept" } else { "undone" },
            blocks.count,
            if se_g > 0.0 { gain / se_g } else { 0.0 },
            if g.min_gain != 0.0 { format!(", min {:.7}", g.min_gain) } else { String::new() },
            trial.n,
            trial.examples
        );
        keep
    }

    /// After node `idx`'s growth trial was undone (spec 1.3, 0.8): refunds its credit and backs the leaf off.
    fn grow_undone(&mut self, idx: usize) {
        let cap = self.params.grow.credit_cap;
        self.grow.credits = (self.grow.credits + 1).min(cap);
        self.grow.credits_refunded += 1;
        // grow4: no snapshot from before the trial (`grow_restart` dropped it; the next close freezes a new one).
        self.nodes[idx].snap = None;
        let s = &mut self.nodes[idx].grow;
        s.restart();
        s.backoff_windows = (2 * s.backoff_windows).max(1);
        s.eligible_after_close = s.closes + s.backoff_windows as u64;
        s.stall_streak = 0;
        eprintln!(
            "grow refund and back-off: credits {}; leaf {idx} backs off {} windows, eligible from close {} (now {})",
            self.grow.credits, s.backoff_windows, s.eligible_after_close, s.closes
        );
    }

    /// Logs the graph's size after a split (spec 0.9), with a warning from `warn_nodes` nodes.
    fn grow_log_nodes(&self) {
        let live = |i: &usize| !self.dead.contains(i);
        let nodes = (0..self.nodes.len()).filter(live).count();
        let leaves = (0..self.nodes.len()).filter(live).filter(|&i| matches!(self.nodes[i].kind, Kind::Leaf)).count();
        eprintln!(
            "nodes {nodes} leaves {leaves} routers {} splits {} credits {}",
            nodes - leaves,
            self.split_number,
            self.grow.credits
        );
        if nodes >= self.params.grow.warn_nodes {
            eprintln!(
                "grow warning: {nodes} nodes (from {}): hybrid15's fixed-point play is exact up to 64 nodes and plays in float above",
                self.params.grow.warn_nodes
            );
        }
    }
}

#[cfg(test)]
thread_local! {
    /// Test record: the last growth trial's gain, block standard error, MDL bar and verdict.
    pub(crate) static LAST_VERDICT: std::cell::Cell<Option<(f64, f64, f64, bool)>> = const { std::cell::Cell::new(None) };
    /// grow2 test record: the n the last growth trial was judged at.
    pub(crate) static LAST_VERDICT_N: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };
}

/// grow2: the router split rules (theory-2 6.40-6.41; growth-spec section 2 as 6.40 amends it), for error-routed DAGs
/// with path context (see `RouterSplitRule`). Everything here runs only under a rule other than `RouterSplitRule::Z`,
/// except `grow_n`, which grow1's scheduler and verdict read too.
impl Graph {
    /// Whether a router split rule other than today's is on.
    pub(crate) fn router_ruled(&self) -> bool {
        self.params.grow.router_rule != RouterSplitRule::Z
    }

    /// The n of the MDL bar, the scheduler's tie-break and the subtree shares (see `MdlN`): with `Trial` the node's
    /// examples since it was made or last split (`num_trained_total`; an undone leaf trial leaves the trial's), with
    /// `Lifetime` all it ever recorded.
    pub(crate) fn grow_n(&self, idx: usize) -> u64 {
        match self.params.grow.mdl_n {
            MdlN::Trial => self.nodes[idx].num_trained_total as u64,
            MdlN::Lifetime => self.nodes[idx].life,
        }
    }

    /// grow3 (6.46 (iii) 1): whether router `idx`'s children may be judged: those holding `guard_share` of their
    /// lifetime examples (`life`) have each closed a window (`num_trained_total` at least W); at 1, every child
    /// (grow2's guard).
    pub(crate) fn children_ready(&self, idx: usize) -> bool {
        let (w, share) = (self.params.split_threshold, self.params.grow.guard_share);
        let children = self.children(idx);
        if share >= 1.0 {
            return children.iter().all(|&c| self.nodes[c].num_trained_total >= w);
        }
        let total: u64 = children.iter().map(|&c| self.nodes[c].life).sum();
        let held: u64 = children.iter().filter(|&&c| self.nodes[c].num_trained_total >= w).map(|&c| self.nodes[c].life).sum();
        total > 0 && held as f64 / total as f64 >= share - 1e-12
    }

    /// Node `idx`'s parents, in increasing order.
    pub(crate) fn parents_of(&self, idx: usize) -> Vec<usize> {
        (0..self.nodes.len()).filter(|&p| self.children(p).contains(&idx)).collect()
    }

    /// Router `idx`'s parents if it can split by arrival path (growth-spec 2.2, 2.4): it has several, and it is not a
    /// member of a sibling set (whose members stay `branch_factor`, each one node).
    pub(crate) fn path_parents(&self, idx: usize) -> Option<Vec<usize>> {
        let parents = self.parents_of(idx);
        (self.nodes[idx].member.is_none() && parents.len() >= 2).then_some(parents)
    }

    /// The split router `idx` would make now (growth-spec 2.2): by path where it has several arrival paths and the
    /// shadow option allows it, else by cluster where allowed; None for neither (`Path` and a single arrival path).
    pub(crate) fn split_kind(&self, idx: usize) -> Option<SplitKind> {
        let path = self.path_parents(idx).map(|_| SplitKind::Path);
        match self.params.grow.router_shadow {
            RouterShadow::Auto => Some(path.unwrap_or(SplitKind::Cluster)),
            RouterShadow::Path => path,
            RouterShadow::Cluster => Some(SplitKind::Cluster),
        }
    }

    /// k of the MDL bar for a split of router `idx` (spec 0.7): an unsharing by m paths adds m - 1 routers, a cluster
    /// split B copies and a top router; a router's parameters are its classifier's and its path table's.
    pub(crate) fn router_split_params(&self, idx: usize, kind: SplitKind) -> usize {
        let node = &self.nodes[idx];
        let d = node.classifier.num_params() + node.table.as_ref().map_or(0, |t| t.param_count());
        match kind {
            SplitKind::Path => (self.parents_of(idx).len() - 1) * d,
            // grow3 (6.46 (iii) 3): under stall-h and stall-hg the label-free top router does not count: (B - 1) d.
            SplitKind::Cluster => match self.params.grow.router_rule {
                RouterSplitRule::StallH | RouterSplitRule::StallHg => (self.params.branch_factor - 1) * d,
                _ => (self.params.branch_factor + 1) * d,
            },
        }
    }

    /// Whether router `idx`'s shadows, if any, are for the split it would make now: the same kind, and by path for
    /// the same parents. Others are stale (the structure changed under them).
    fn shadows_fit(&self, idx: usize) -> bool {
        match self.nodes[idx].rgrow.shadows.as_deref().map(|s| &s.partition) {
            None => true,
            Some(Partition::Path { parents }) => self.path_parents(idx).as_ref() == Some(parents) && self.split_kind(idx) == Some(SplitKind::Path),
            Some(Partition::Cluster { .. }) => self.split_kind(idx) == Some(SplitKind::Cluster),
        }
    }

    /// Per scored example at router `idx`, which the position entered from `parent` with `marks`, before the router
    /// trains on it (theory-2 6.40 (2)): adds its headroom, the loss of the child it routes to minus the least of
    /// `losses` (each child's loss, through the child); and while its shadows run, picks the example's part (its
    /// arrival path, or the cluster router's class, which then trains on it), adds the gain of the part's shadow's
    /// pick, scored before the shadow trains, and trains the shadow on the router's target (the `error_target` of
    /// `child_errors`).
    pub(crate) fn router_observe(
        &mut self,
        idx: usize,
        parent: Option<usize>,
        marks: &[Mark],
        data: &[u8],
        losses: &[f64],
        child_errors: &[f64],
    ) {
        let (len, bf) = (self.grow_block_len(), self.params.branch_factor);
        let routed = self.route_ctx(idx, marks, data);
        let least = losses.iter().copied().fold(f64::INFINITY, f64::min);
        let mut rg = std::mem::take(&mut self.nodes[idx].rgrow);
        rg.h.add(losses[routed] - least, len);
        if let Some(shadows) = rg.shadows.as_deref_mut().filter(|s| s.running) {
            let part = match &mut shadows.partition {
                Partition::Path { parents } => {
                    let parent = parent.expect("a router with several parents is not the root");
                    parents.iter().position(|&p| p == parent).expect("the example came from one of the router's parents")
                }
                Partition::Cluster { q, trained } => {
                    let pick = match *trained < (self.params.split_threshold / 5) as u64 {
                        true => (position_hash(data) % bf as u64) as usize,
                        false => {
                            let mut scores = vec![0.0; bf];
                            q.predict_training(data, &mut scores);
                            argmax(&scores)
                        }
                    };
                    q.train(data, &one_hot(pick, bf));
                    *trained += 1;
                    pick
                }
            };
            let (router, table) = &mut shadows.routers[part];
            let mut values = Vec::new();
            table.values_for(marks, &mut values);
            let mut out = vec![0.0; bf];
            shadow_output(router, table, &values, data, &mut out);
            let gain = losses[routed] - losses[argmax(&out)];
            rg.g.add(gain, len);
            let target = self.error_target(child_errors);
            shadow_train(router, table, &values, data, &target);
            // grow3, stall-hg: the null shadow of the example's hash part, scored before it trains and trained alike.
            if !shadows.null.is_empty() {
                let part = (position_hash(data) % shadows.null.len() as u64) as usize;
                let (router, table) = &mut shadows.null[part];
                values.clear();
                table.values_for(marks, &mut values);
                shadow_output(router, table, &values, data, &mut out);
                let null_gain = losses[routed] - losses[argmax(&out)];
                rg.g_null.add(null_gain, len);
                rg.g_diff.add(gain - null_gain, len);
                shadow_train(router, table, &values, data, &target);
            }
        }
        self.nodes[idx].rgrow = rg;
    }

    /// `Subtree` only, per scored example at leaf `idx`: the blocks of its errors, closing a window (spec 0.2) every 20
    /// blocks, for its ancestors' subtree stall (0.4).
    pub(crate) fn leaf_grow_add(&mut self, idx: usize, error: f64) {
        let (len, gp, step) = (self.grow_block_len(), self.params.grow, self.step);
        let s = &mut self.nodes[idx].grow;
        s.blocks.add(error, len);
        if s.blocks.count >= 20 {
            let (mean, se, change) = s.close(gp.stall_z, 0.0);
            if gp.log_router_stats {
                eprintln!(
                    "leaf close {idx} at N {step}: mean {mean:.7} (se {se:.7}), {}, streak {}{}",
                    describe_change(change),
                    s.stall_streak,
                    if s.prev_exists && s.stall_streak >= gp.stall_windows { " (stagnant)" } else { "" }
                );
            }
        }
    }

    /// Whether node `v`'s subtree has stalled (spec 0.4): it is stagnant out of back-off, and it is a leaf or its
    /// stalled children hold `subtree_share` of its children's examples (by `grow_n`); each node evaluated once.
    pub(crate) fn subtree_stalled(&self, v: usize) -> bool {
        self.stalled_memo(v, &mut vec![None; self.nodes.len()])
    }

    fn stalled_memo(&self, v: usize, memo: &mut Vec<Option<bool>>) -> bool {
        if let Some(stalled) = memo[v] {
            return stalled;
        }
        let stalled = self.nodes[v].grow.eligible(self.params.grow.stall_windows)
            && match &self.nodes[v].kind {
                Kind::Leaf => true,
                Kind::Branch { children } => {
                    let total: u64 = children.iter().map(|&c| self.grow_n(c)).sum();
                    let held: u64 = children.iter().filter(|&&c| self.stalled_memo(c, memo)).map(|&c| self.grow_n(c)).sum();
                    total > 0 && held as f64 >= self.params.grow.subtree_share * total as f64
                }
            };
        memo[v] = Some(stalled);
        stalled
    }

    /// The close of router `idx`'s window under a router split rule (theory-2 6.40-6.41): its routing error's window
    /// and stagnation (spec 0.2), the post-split check waiting for it (2.5), its headroom H and its shadows' gain G
    /// over the window (block standard errors), the gate, the decision, and its shadows for the next window. Returns
    /// the split to make now, with the window's mean routing error and block standard error (the post-split check's
    /// reference). The caller makes the split and opens the next window.
    pub(crate) fn router_close(&mut self, idx: usize) -> Option<(SplitKind, f64, f64)> {
        let (gp, step) = (self.params.grow, self.step);
        let (leaf_splits, router_splits) = self.split_counts();
        let budget = match self.params.router_split_limit {
            None => self.split_number < self.params.split_limit,
            Some(cap) => router_splits < cap && router_splits < leaf_splits,
        };
        let examples = self.nodes[idx].num_trained_last as u64;
        let kind = self.split_kind(idx);
        if !self.shadows_fit(idx) {
            // Shadows of another partition than the split's are dropped at the structure change that made them so
            // (`conform_shadows`); here for safety, before their gain is read.
            let rg = &mut self.nodes[idx].rgrow;
            (rg.shadows, rg.g_full, rg.g, rg.g_null, rg.g_diff) = (None, false, Blocks::default(), Blocks::default(), Blocks::default());
        }
        let node = &mut self.nodes[idx];
        // The window's per-example statistics as before (`error_prev`, which model files hold).
        node.close_window(0.0);
        let model_mean = self.model_errors.mean_since(node.window_model);
        let (mean, se, change) = node.grow.close(gp.stall_z, model_mean);
        let stats = |b: &Blocks| (b.count >= 2).then(|| (b.stats(), b.count));
        let h = stats(&node.rgrow.h);
        let gain = if node.rgrow.g_full { stats(&node.rgrow.g) } else { None };
        // grow3, stall-hg: the null shadows' gain and the paired difference (6.46 (iii) 2).
        let null_ran = node.rgrow.g_full && node.rgrow.shadows.as_ref().is_some_and(|s| !s.null.is_empty());
        let (gain_null, gain_diff) = match null_ran {
            true => (stats(&node.rgrow.g_null), stats(&node.rgrow.g_diff)),
            false => (None, None),
        };
        node.rgrow.h = Blocks::default();
        node.rgrow.g = Blocks::default();
        node.rgrow.g_null = Blocks::default();
        node.rgrow.g_diff = Blocks::default();
        node.rgrow.g_full = false;
        let (streak, closes) = (node.grow.stall_streak, node.grow.closes);
        self.post_check_report(idx, mean, se, examples);
        let stagnant = self.nodes[idx].grow.eligible(gp.stall_windows);
        let ready = self.children_ready(idx);
        let n = self.grow_n(idx);
        let k = kind.map(|kind| self.router_split_params(idx, kind));
        let bar = k.map(|k| mdl_bar(gp.mdl, k, n));
        let clears = |s: Option<((f64, f64), u32)>| match (s, bar) {
            (Some(((mean, se), _)), Some(bar)) => mean > gp.router_z * se && mean > bar,
            _ => false,
        };
        let h_pass = clears(h);
        // grow3: stall-hg splits on the real shadows' gain over the null shadows' (paired), subtree on the real gain.
        let g_pass = match gp.router_rule {
            RouterSplitRule::StallHg => clears(gain_diff),
            _ => clears(gain),
        };
        let rule = gp.router_rule;
        let stalled = rule == RouterSplitRule::Subtree && kind.is_some() && self.subtree_stalled(idx);
        let gate = match rule {
            RouterSplitRule::StallH | RouterSplitRule::StallHg => stagnant && ready && h_pass,
            RouterSplitRule::Subtree => stalled,
            RouterSplitRule::Z => unreachable!("today's rule closes its windows in check_delegator_split"),
        };
        let fire = gate && (rule == RouterSplitRule::StallH || g_pass);
        let split = fire && budget;
        let shadows = match (split, rule, kind) {
            (true, _, _) => "used by the split",
            (false, RouterSplitRule::StallHg, Some(kind)) if h_pass => self.run_shadows(idx, kind),
            (false, RouterSplitRule::Subtree, Some(kind)) => self.run_shadows(idx, kind),
            _ => self.pause_shadows(idx),
        };
        #[cfg(test)]
        LAST_ROUTER_CLOSE.with(|c| {
            c.borrow_mut().push(RouterCloseRecord {
                idx,
                step,
                kind,
                k,
                n,
                bar,
                h: h.map(|x| x.0),
                g: gain.map(|x| x.0),
                g_null: gain_null.map(|x| x.0),
                g_diff: gain_diff.map(|x| x.0),
                stagnant,
                ready,
                h_pass,
                g_pass,
                gate,
                split,
            })
        });
        if gp.log_router_stats {
            let describe = |s: Option<((f64, f64), u32)>| match s {
                Some(((mean, se), blocks)) => {
                    format!("{mean:+.7} (se {se:.7}, z {:+.2}, {blocks} blocks)", if se > 0.0 { mean / se } else { 0.0 })
                }
                None => "none".to_string(),
            };
            // grow3: stall-hg's split statistic is the paired difference, after the real and the null gains.
            let g_text = match gp.router_rule {
                RouterSplitRule::StallHg => format!(
                    "{}; G null {}; G diff {}{}",
                    describe(gain),
                    describe(gain_null),
                    describe(gain_diff),
                    if g_pass { " passes" } else { "" }
                ),
                _ => format!("{}{}", describe(gain), if g_pass { " passes" } else { "" }),
            };
            let kind_text = match kind {
                Some(SplitKind::Path) => format!("path (parents {:?})", self.parents_of(idx)),
                Some(SplitKind::Cluster) => "cluster".to_string(),
                None => "no split kind".to_string(),
            };
            eprintln!(
                "router close {idx} at N {step}: routing error {mean:.7} (se {se:.7}), {}, streak {streak}{}, closes {closes}; children {}; {kind_text}, k {}, n {n}, bars z {} and mdl {}; H {}{}; G {g_text}{}; gate {}; {}; shadows {shadows}",
                describe_change(change),
                if stagnant { " (stagnant)" } else { "" },
                if ready { "ready" } else { "not ready" },
                k.map_or("-".to_string(), |k| k.to_string()),
                gp.router_z,
                bar.map_or("-".to_string(), |b| format!("{b:.7}")),
                describe(h),
                if h_pass { " passes" } else { "" },
                if rule == RouterSplitRule::Subtree { format!("; subtree {}", if stalled { "stalled" } else { "not stalled" }) } else { String::new() },
                if gate { "open" } else { "shut" },
                match (fire, budget) {
                    (true, true) => "SPLIT",
                    (true, false) => "would split, but the budget is spent",
                    (false, _) => "no split",
                },
            );
        }
        split.then(|| (kind.expect("a split has a kind"), mean, se))
    }

    /// Starts router `idx`'s shadows for partition `kind` (clones of its router and path table, no forgetting:
    /// growth-spec 2.2) if it has none, and runs them from the next window. Returns what happened, for the log.
    pub(crate) fn run_shadows(&mut self, idx: usize, kind: SplitKind) -> &'static str {
        let started = self.nodes[idx].rgrow.shadows.is_none();
        if started {
            let node = &self.nodes[idx];
            let table = node.table.as_deref().expect("with path context every node has a path table");
            let (partition, m) = match kind {
                SplitKind::Path => {
                    let parents = self.path_parents(idx).expect("a path split has several parents");
                    let m = parents.len();
                    (Partition::Path { parents }, m)
                }
                SplitKind::Cluster => (Partition::Cluster { q: self.new_router(), trained: 0 }, self.params.branch_factor),
            };
            let routers = vec![(node.classifier.clone(), table.clone()); m];
            // grow3: stall-hg's null shadows, the same copies over a hash partition of the same shape (6.46 (iii) 2).
            let null = match self.params.grow.router_rule {
                RouterSplitRule::StallHg => routers.clone(),
                _ => Vec::new(),
            };
            self.nodes[idx].rgrow.shadows = Some(Box::new(Shadows { partition, routers, running: false, null }));
        }
        let rg = &mut self.nodes[idx].rgrow;
        let shadows = rg.shadows.as_deref_mut().expect("shadows");
        let was = std::mem::replace(&mut shadows.running, true);
        rg.g_full = true;
        match (started, was) {
            (true, _) => "started",
            (false, true) => "running",
            (false, false) => "resumed",
        }
    }

    /// Pauses router `idx`'s shadows, if any (stall-hg: its headroom failed its bars). Returns what happened.
    fn pause_shadows(&mut self, idx: usize) -> &'static str {
        let rg = &mut self.nodes[idx].rgrow;
        rg.g_full = false;
        match rg.shadows.as_deref_mut() {
            Some(s) => match std::mem::replace(&mut s.running, false) {
                true => "paused",
                false => "paused still",
            },
            None => "none",
        }
    }

    /// After a router split under a router split rule: the shadows whose partition the structure no longer gives
    /// (their router's parents changed) are dropped, to start afresh at their router's next start (growth-spec 2.7 b);
    /// the others get the cells their router's path table gained since they started, at neutral values (as
    /// `pathctx::after_router_split` gives them).
    pub(crate) fn conform_shadows(&mut self) {
        for idx in 0..self.nodes.len() {
            let Some(kind) = self.nodes[idx].rgrow.shadows.as_ref().map(|s| s.partition.kind()) else { continue };
            if !self.shadows_fit(idx) {
                let rg = &mut self.nodes[idx].rgrow;
                (rg.shadows, rg.g_full, rg.g, rg.g_null, rg.g_diff) = (None, false, Blocks::default(), Blocks::default(), Blocks::default());
                if self.params.grow.log_router_stats {
                    eprintln!("router {idx}'s shadows ({kind:?}) dropped at N {}: its arrival paths changed", self.step);
                }
                continue;
            }
            let cells = self.nodes[idx].table.as_ref().expect("with path context every node has a path table").cells().to_vec();
            let shadows = self.nodes[idx].rgrow.shadows.as_deref_mut().expect("shadows");
            for (_, table) in shadows.routers.iter_mut().chain(shadows.null.iter_mut()) {
                for &cell in &cells {
                    if !table.cells().contains(&cell) {
                        table.insert_cell(cell);
                    }
                }
            }
        }
    }

    /// After router `idx`'s split of `kind` made the nodes `made` (an unsharing: `idx` itself first): the router
    /// starts afresh (spec 0.3), the split's post-split check waits for the nodes' first windows (2.5, `ref_mean` and
    /// `ref_se` the router's last window), other routers' shadows follow the structure, and the log.
    pub(crate) fn after_rule_split(&mut self, idx: usize, kind: SplitKind, made: Vec<usize>, ref_mean: f64, ref_se: f64) {
        let node = &mut self.nodes[idx];
        (node.grow, node.rgrow) = (GrowStats::default(), RouterGrow::default());
        let (_, router_splits) = self.split_counts();
        eprintln!(
            "router split of router {idx} at N {}: {}, made {made:?}; router splits {router_splits}{}",
            self.step,
            match kind {
                SplitKind::Path => "unshared by arrival path".to_string(),
                SplitKind::Cluster => format!("cluster split into a new sibling set (set {})", self.sets.len() - 1),
            },
            self.params.router_split_limit.map_or(String::new(), |cap| format!(" of {cap}"))
        );
        self.post_checks.push(PostCheck { router: idx, at: self.step, first: vec![None; made.len()], made, ref_mean, ref_se });
        self.conform_shadows();
        let nodes = self.nodes.len();
        let leaves = self.nodes.iter().filter(|n| matches!(n.kind, Kind::Leaf)).count();
        eprintln!("nodes {nodes} leaves {leaves} routers {} splits {}", nodes - leaves, self.split_number);
        if nodes >= self.params.grow.warn_nodes {
            eprintln!(
                "grow warning: {nodes} nodes (from {}): hybrid15's fixed-point play is exact up to 64 nodes and plays in float above",
                self.params.grow.warn_nodes
            );
        }
    }

    /// At the close of node `idx`'s window (mean routing error `mean`, block standard error `se`, `n` examples): its
    /// first window, for the post-split checks waiting for it; a check whose nodes have all closed one is made
    /// (growth-spec 2.5): their mean, pooled by examples, above the router's last window before the split by more than
    /// `router_z` standard errors of the difference warns and backs the nodes off (spec 0.8). The split stays.
    pub(crate) fn post_check_report(&mut self, idx: usize, mean: f64, se: f64, n: u64) {
        let mut done = Vec::new();
        for (c, check) in self.post_checks.iter_mut().enumerate() {
            for (j, &m) in check.made.iter().enumerate() {
                if m == idx && check.first[j].is_none() {
                    check.first[j] = Some((mean, se, n));
                }
            }
            if check.first.iter().all(|f| f.is_some()) {
                done.push(c);
            }
        }
        let checks: Vec<PostCheck> = done.into_iter().rev().map(|c| self.post_checks.remove(c)).collect();
        for check in checks.into_iter().rev() {
            let first: Vec<(f64, f64, u64)> = check.first.iter().map(|f| f.expect("closed")).collect();
            let total: u64 = first.iter().map(|f| f.2).sum::<u64>().max(1);
            let pooled: f64 = first.iter().map(|f| f.0 * f.2 as f64).sum::<f64>() / total as f64;
            let se_pool = first.iter().map(|f| (f.2 as f64 / total as f64 * f.1).powi(2)).sum::<f64>().sqrt();
            let diff = pooled - check.ref_mean;
            let bar = self.params.grow.router_z * (se_pool * se_pool + check.ref_se * check.ref_se).sqrt();
            let worse = diff > bar;
            self.post_checked += 1;
            if worse {
                self.post_warnings += 1;
                for &m in &check.made {
                    let s = &mut self.nodes[m].grow;
                    s.backoff_windows = (2 * s.backoff_windows).max(1);
                    s.eligible_after_close = s.closes + s.backoff_windows as u64;
                    s.stall_streak = 0;
                }
            }
            eprintln!(
                "router post-split check of router {}'s split at N {}: nodes {:?}, first windows {pooled:.7} (se {se_pool:.7}, {total} examples) vs the router's last {:.7} (se {:.7}): {diff:+.7} vs bar {bar:.7}: {}; checks {} warnings {}",
                check.router,
                check.at,
                check.made,
                check.ref_mean,
                check.ref_se,
                if worse { "WARNING, worse: the nodes back off" } else { "fine" },
                self.post_checked,
                self.post_warnings
            );
        }
    }
}

/// grow2: a window's change for the log: the improvement over the previous window and its bar, or none at the first.
fn describe_change(change: Option<(f64, f64)>) -> String {
    match change {
        Some((improvement, bar)) => format!("improvement {improvement:+.7} vs bar {bar:.7}"),
        None => "first window".to_string(),
    }
}

/// grow2: a shadow router's output for a position whose path gives its cells the values `values`, as
/// `Graph::node_output_ctx` computes a router's (`Graph::node_output` without cells).
pub(crate) fn shadow_output(router: &NodeModel, table: &PathTable, values: &[usize], data: &[u8], out: &mut [f64]) {
    if values.is_empty() {
        return router.predict_training(data, out);
    }
    let mut extra = vec![0.0; out.len()];
    match router {
        NodeModel::Lr(lr) => {
            table.lr_offsets(values, &mut extra);
            lr.predict_log(data, out);
            out.iter_mut().zip(&extra).for_each(|(z, e)| *z += e);
            softmax(out);
        }
        NodeModel::Cnbp(nb) => {
            table.cnbp_factors(values, nb.alpha(), &mut extra);
            nb.predict_counts_scaled(data, &extra, out);
        }
        _ => unreachable!("path context routers are logistic regressions or CNBPs"),
    }
}

/// grow2: trains a shadow router on `target` with its path table, as `Graph::train_with_table` trains a router.
pub(crate) fn shadow_train(router: &mut NodeModel, table: &mut PathTable, values: &[usize], data: &[u8], target: &[f64]) {
    if values.is_empty() {
        return router.train(data, target);
    }
    match router {
        NodeModel::Lr(lr) => {
            let n = lr.class_num();
            let (mut p, mut extra) = (vec![0.0; n], vec![0.0; n]);
            table.lr_offsets(values, &mut extra);
            lr.predict_log(data, &mut p);
            p.iter_mut().zip(&extra).for_each(|(z, e)| *z += e);
            softmax(&mut p);
            lr.step(data, &p, target, 1.0, 0.0);
            table.lr_step(values, &p, target, lr.rate());
        }
        NodeModel::Cnbp(nb) => {
            nb.train(data, target);
            table.cnbp_add(values, target);
        }
        _ => unreachable!("path context routers are logistic regressions or CNBPs"),
    }
}

/// grow2 test record: a router close's statistics and decision (see `Graph::router_close`).
#[cfg(test)]
#[derive(Clone, Debug)]
pub(crate) struct RouterCloseRecord {
    pub idx: usize,
    pub step: u64,
    pub kind: Option<SplitKind>,
    pub k: Option<usize>,
    pub n: u64,
    pub bar: Option<f64>,
    pub h: Option<(f64, f64)>,
    pub g: Option<(f64, f64)>,
    pub g_null: Option<(f64, f64)>,
    pub g_diff: Option<(f64, f64)>,
    pub stagnant: bool,
    pub ready: bool,
    pub h_pass: bool,
    pub g_pass: bool,
    pub gate: bool,
    pub split: bool,
}

#[cfg(test)]
thread_local! {
    /// grow2 test record: every router close's statistics and decision, in order.
    pub(crate) static LAST_ROUTER_CLOSE: std::cell::RefCell<Vec<RouterCloseRecord>> = const { std::cell::RefCell::new(Vec::new()) };
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
        // Only trees with path context have a registry, so others' states are as before.
        if self.params.path_context {
            w.usize(self.sets.len());
            self.sets.iter().for_each(|members| w.usizes(members));
        }
        // grow1: likewise the growth rule's credits (spec 1.5).
        let grow = self.growing();
        if grow {
            let g = self.grow;
            w.u32(g.credits);
            w.u64(g.next_credit_at);
            w.u64(g.credits_spent);
            w.u64(g.credits_refunded);
            w.u32(g.trials_in_flight);
        }
        // grow2: likewise a router split rule's post-split checks.
        let ruled = self.router_ruled();
        if ruled {
            w.u64(self.post_checked);
            w.u64(self.post_warnings);
            w.usize(self.post_checks.len());
            for c in &self.post_checks {
                w.usize(c.router);
                w.u64(c.at);
                w.usizes(&c.made);
                w.f64(c.ref_mean);
                w.f64(c.ref_se);
                for first in &c.first {
                    w.bool(first.is_some());
                    if let Some((mean, se, n)) = first {
                        w.f64(*mean);
                        w.f64(*se);
                        w.u64(*n);
                    }
                }
            }
        }
        let life = (grow && self.params.grow.mdl_n == MdlN::Lifetime) || ruled;
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
                if grow {
                    write_blocks(w, &trial.grow.expect("with the growth rule every trial has blocks"));
                }
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
            // Likewise path tables.
            if self.params.path_context {
                node.table.as_ref().expect("with path context every node has a path table").write_state(w);
            }
            // grow1: and the growth rule's statistics; grow2: a router split rule's too.
            if grow || ruled {
                let s = &node.grow;
                write_blocks(w, &s.blocks);
                w.bool(s.prev_exists);
                w.f64(s.prev_mean);
                w.f64(s.prev_se);
                w.f64(s.prev_model_mean);
                w.u32(s.stall_streak);
                w.u64(s.closes);
                w.u32(s.backoff_windows);
                w.u64(s.eligible_after_close);
            }
            // grow4: the paired snapshot statistics and the frozen copy, only where leaves measure them.
            if grow && self.params.grow.snapshot_on() {
                let s = &node.grow;
                write_blocks(w, &s.snap_blocks);
                w.bool(s.snap_exists);
                w.f64(s.snap_gain);
                w.f64(s.snap_se);
                w.bool(node.snap.is_some());
                if let Some(snap) = &node.snap {
                    snap.write_state(w);
                }
            }
            // grow2: the lifetime count where it is read, and a router split rule's headroom, gain and shadows.
            if life {
                w.u64(node.life);
            }
            if ruled {
                write_router_grow(w, &node.rgrow, self.params.grow.router_rule == RouterSplitRule::StallHg);
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
        let sets = match params.path_context {
            true => (0..r.usize()?).map(|_| r.usizes()).collect::<io::Result<Vec<_>>>()?,
            false => Vec::new(),
        };
        let grow_rule = params.split_rule == SplitRule::Grow;
        let grow = match grow_rule {
            true => GrowGraph {
                credits: r.u32()?,
                next_credit_at: r.u64()?,
                credits_spent: r.u64()?,
                credits_refunded: r.u64()?,
                trials_in_flight: r.u32()?,
            },
            false => GrowGraph::default(),
        };
        let ruled = params.grow.router_rule != RouterSplitRule::Z;
        let (mut post_checked, mut post_warnings, mut post_checks) = (0, 0, Vec::new());
        if ruled {
            post_checked = r.u64()?;
            post_warnings = r.u64()?;
            for _ in 0..r.usize()? {
                let (router, at, made, ref_mean, ref_se) = (r.usize()?, r.u64()?, r.usizes()?, r.f64()?, r.f64()?);
                let first = (0..made.len())
                    .map(|_| Ok(if r.bool()? { Some((r.f64()?, r.f64()?, r.u64()?)) } else { None }))
                    .collect::<io::Result<Vec<_>>>()?;
                post_checks.push(PostCheck { router, at, made, first, ref_mean, ref_se });
            }
        }
        let life = (grow_rule && params.grow.mdl_n == MdlN::Lifetime) || ruled;
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
                if grow_rule {
                    trial.grow = Some(read_blocks(r)?);
                }
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
            if params.path_context {
                node.table = Some(Box::new(PathTable::read_state(r, matches!(node.classifier, NodeModel::Probit(_)))?));
            }
            if grow_rule || ruled {
                node.grow = GrowStats {
                    blocks: read_blocks(r)?,
                    prev_exists: r.bool()?,
                    prev_mean: r.f64()?,
                    prev_se: r.f64()?,
                    prev_model_mean: r.f64()?,
                    stall_streak: r.u32()?,
                    closes: r.u64()?,
                    backoff_windows: r.u32()?,
                    eligible_after_close: r.u64()?,
                    ..GrowStats::default()
                };
            }
            if grow_rule && params.grow.snapshot_on() {
                let s = &mut node.grow;
                s.snap_blocks = read_blocks(r)?;
                s.snap_exists = r.bool()?;
                s.snap_gain = r.f64()?;
                s.snap_se = r.f64()?;
                if r.bool()? {
                    node.snap = Some(Box::new(NodeModel::read_state(r)?));
                }
            }
            if life {
                node.life = r.u64()?;
            }
            if ruled {
                node.rgrow = read_router_grow(r, params.grow.router_rule == RouterSplitRule::StallHg)?;
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
        if !post_checks.iter().flat_map(|c| c.made.iter().chain([&c.router])).all(in_range) {
            return Err(invalid("a post-split check of the state refers to a node out of range"));
        }
        let mut graph = Graph {
            categories: categories.to_vec(),
            params,
            split_number,
            nodes,
            step,
            model_errors,
            router_errors,
            dead,
            game: None,
            sets,
            grow,
            post_checks,
            post_checked,
            post_warnings,
            unshared: None,
        };
        if graph.params.path_context {
            graph.register_members()?;
        }
        Ok(graph)
    }
}

/// grow1: block accumulators in a state file.
fn write_blocks(w: &mut Writer, b: &Blocks) {
    w.f64(b.partial);
    w.u32(b.in_block);
    w.u32(b.count);
    w.f64(b.sum);
    w.f64(b.sum_sq);
}

fn read_blocks(r: &mut Reader) -> io::Result<Blocks> {
    Ok(Blocks { partial: r.f64()?, in_block: r.u32()?, count: r.u32()?, sum: r.f64()?, sum_sq: r.f64()? })
}

/// grow2: a router's headroom and gain blocks and its shadows in a state file.
fn write_router_grow(w: &mut Writer, g: &RouterGrow, null: bool) {
    write_blocks(w, &g.h);
    write_blocks(w, &g.g);
    if null {
        write_blocks(w, &g.g_null);
        write_blocks(w, &g.g_diff);
    }
    w.bool(g.g_full);
    w.bool(g.shadows.is_some());
    if let Some(sh) = &g.shadows {
        match &sh.partition {
            Partition::Path { parents } => {
                w.u8(0);
                w.usizes(parents);
            }
            Partition::Cluster { q, trained } => {
                w.u8(1);
                q.write_state(w);
                w.u64(*trained);
            }
        }
        w.bool(sh.running);
        w.usize(sh.routers.len());
        for (router, table) in &sh.routers {
            router.write_state(w);
            table.write_state(w);
        }
        if null {
            w.usize(sh.null.len());
            for (router, table) in &sh.null {
                router.write_state(w);
                table.write_state(w);
            }
        }
    }
}

fn read_router_grow(r: &mut Reader, null: bool) -> io::Result<RouterGrow> {
    let (h, g) = (read_blocks(r)?, read_blocks(r)?);
    let (g_null, g_diff) = match null {
        true => (read_blocks(r)?, read_blocks(r)?),
        false => (Blocks::default(), Blocks::default()),
    };
    let g_full = r.bool()?;
    let shadows = match r.bool()? {
        false => None,
        true => {
            let partition = match r.u8()? {
                0 => Partition::Path { parents: r.usizes()? },
                1 => Partition::Cluster { q: NodeModel::read_state(r)?, trained: r.u64()? },
                k => return Err(invalid(format!("unknown shadow partition {k} in state file"))),
            };
            let running = r.bool()?;
            let routers = (0..r.usize()?)
                .map(|_| Ok((NodeModel::read_state(r)?, PathTable::read_state(r, false)?)))
                .collect::<io::Result<Vec<_>>>()?;
            let null = match null {
                true => (0..r.usize()?)
                    .map(|_| Ok((NodeModel::read_state(r)?, PathTable::read_state(r, false)?)))
                    .collect::<io::Result<Vec<_>>>()?,
                false => Vec::new(),
            };
            Some(Box::new(Shadows { partition, routers, running, null }))
        }
    };
    Ok(RouterGrow { h, g, g_null, g_diff, g_full, shadows })
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
    let (mut scores_small, mut scores_big) = ([0.0; 8], Vec::new());
    let scores = small_buffer(&mut scores_small, &mut scores_big, router.class_num());
    router.predict(data, scores);
    argmax(scores)
}

/// errdag4: a zeroed scratch slice of length `n`: `small` (zeroed by the caller) up to its 8 entries, otherwise `big`
/// grown to `n` zeros. It replaces 256-wide stack buffers whose zeroing was ~5% of DAG training and play; every user
/// reads only the first `n` entries, as before.
pub(crate) fn small_buffer<'a>(small: &'a mut [f64; 8], big: &'a mut Vec<f64>, n: usize) -> &'a mut [f64] {
    if n <= small.len() {
        &mut small[..n]
    } else {
        big.clear();
        big.resize(n, 0.0);
        &mut big[..]
    }
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
