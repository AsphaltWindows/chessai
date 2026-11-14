//! Hoeffding tree (Domingos and Hulten 2000, "VFDT"), the textbook
//! incremental decision tree, as a baseline for the delegation trees. Each
//! internal node tests one feature for one value (`x[f] == v` or not), and
//! each leaf holds a CNBP or logistic regression that predicts the
//! positions reaching it. No model routes: the tests are the routing. A
//! leaf splits on the test with the highest information gain once the
//! Hoeffding bound says, with confidence 1 - δ, that the test beats the
//! runner-up, or that the two are too close to matter (the bound is below
//! the tie threshold τ), as in VFDT and MOA. Rust only.
//!
//! Adapted to self-play data:
//! - The positions of a game share its result, so they are far from
//!   independent: the bound counts `positions_per_sample` positions as one
//!   sample (by default a jitter game's ~250).
//! - A splitting leaf's model is copied to both children, as FIMT-DD does
//!   with its leaves' linear models and our BDT does with forget factor 1,
//!   so predictions do not jump at a split. The children's split statistics
//!   start empty, as in VFDT.
//! - Splits are checked in `refresh`, once per self-play round.
//!
//! Model gain (`Criterion::ModelGain`, LR leaves only) instead chooses the
//! test that would most improve the leaf's predictions if each side refit
//! the leaf's weights, rather than the one that says most about the result.
//! A test that only shifts the odds gains little, since the leaf has a
//! weight for the test's value already; a test gains where the other weights
//! should differ between its sides (if a king's square changed what the other
//! pieces' squares are worth, say). A diagonal Newton step on each weight w
//! (each class's bias and value weights) estimates the gain from the sums G
//! and H of the first and second derivatives of the examples' log-loss with
//! respect to w's class's logit, under the leaf's predictions as it trains:
//! ½ Σ_w [G_yes²/(H_yes + λ) + G_no²/(H_no + λ) − G²/(H + λ)]. A leaf sums
//! the derivatives over its examples having each pair of values, which gives
//! every weight's sums on every test's yes side, the no side's being the
//! rest. The choice is honest: a hash of each game puts it in half A or B,
//! the best test on A is checked on B, and the leaf splits when B's gain per
//! position is above a threshold. B's gain is how much more B's loss would
//! fall (to second order) under the Newton steps fitted on A's two sides than
//! under the steps fitted on all of A: negative on average for a test that
//! does not matter, however many weights there are and however alike a
//! game's positions are, whereas the gain of B's own steps on B would be
//! about ½ per weight, times the positions of a game that a weight sees.
//!
//! Model files hold the tests and the leaves' models but not the split
//! statistics (self-play saves one file per round), so a loaded tree
//! predicts the same but gathers its statistics afresh. Self-play also saves
//! the whole tree with its statistics as a state file (see `state`), from
//! which `--resume` continues exactly.

use std::fmt::Write as _;
use std::io;
use std::path::Path;

use rayon::prelude::*;

use crate::cnbp::Cnbp;
use crate::delegation::NodeModel;
use crate::encoding;
use crate::logreg::Logreg;
use crate::selfplay::splitmix64;
use crate::state::{Reader, Writer};
use crate::textfmt::{Tokens, invalid};

/// How a leaf chooses its test and decides to split (see the module
/// documentation).
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Criterion {
    /// Information gain about the class, by the Hoeffding bound.
    InfoGain,
    /// The gain of refitting the leaf's logistic regression on each side,
    /// checked on held-out games.
    ModelGain,
}

/// When leaves split (see the module documentation). Not saved in model
/// files; the defaults when loading.
#[derive(Clone, Copy, Debug)]
pub struct HtParams {
    /// A split is wrong with probability at most this.
    pub delta: f64,
    /// Split on the best test once the bound (in bits) is below this, however
    /// close the runner-up.
    pub tie: f64,
    /// Positions a leaf adds to its statistics between split checks.
    pub grace: f64,
    /// Positions the bound counts as one independent sample.
    pub positions_per_sample: f64,
    /// Most leaves the tree may have.
    pub max_leaves: usize,
    /// Split on a tie only if the best test's gain is above the bound too, so
    /// that it beats not splitting with confidence 1 - δ.
    pub pre_prune: bool,
    /// How leaves choose their tests and decide to split.
    pub criterion: Criterion,
    /// Model gain: split when the best test's gain on half B is above this
    /// many nats per position of B.
    pub gain_threshold: f64,
    /// Model gain: the L2 penalty λ of the Newton steps, in the units of H
    /// (each position adds p(1 - p) for each class of probability p).
    pub lambda: f64,
    /// Model gain: the fraction of games whose positions the statistics
    /// keep, chosen by a hash of the game.
    pub sample: f64,
}

impl Default for HtParams {
    /// VFDT's δ and grace period (MOA's defaults: 1e-7, and 200 samples,
    /// here 200 games' worth of positions), with a far smaller τ than its
    /// 0.05, because our gains are tiny. At the root of a self-play run from
    /// scratch (CNBP leaves, seed 11) the best test gained 0.0088 bits in
    /// round 0, 0.0056 in round 20 and 0.0044 in round 59, with a runner-up
    /// at most 0.001 behind; on the v1000 F-L's games against itself, 0.0046
    /// and 0.0042 after 10 rounds. The runner-up never falls behind by the
    /// bound, so ties decide: a leaf splits once it has R² ln(1/δ)
    /// positions_per_sample / (2τ²) positions in its statistics, R being log2
    /// of the number of classes, which with τ = 0.013 is 30M positions, about
    /// round 100 for the root, near our BDTs' first splits. τ = 0.05 would
    /// split the root in round 6, and `pre_prune` would wait for the bound to
    /// fall below the gain, 250M positions or more (round 900 or later).
    ///
    /// Model gain's defaults: see `model_gain_ignores_noise`.
    fn default() -> Self {
        HtParams {
            delta: 1e-7,
            tie: 0.013,
            grace: 50_000.0,
            positions_per_sample: 250.0,
            max_leaves: usize::MAX,
            pre_prune: false,
            criterion: Criterion::InfoGain,
            gain_threshold: 1e-4,
            lambda: 10.0,
            sample: 1.0,
        }
    }
}

/// A value model gain does not track.
const UNTRACKED: u16 = u16::MAX;

/// Blocks of rows of model gain's pair sums, added in parallel.
const BLOCKS: usize = 32;

/// A leaf's statistics for choosing its split, gathered since it was made.
#[derive(Clone, Debug)]
enum Stats {
    Counts(Counts),
    Gains(Box<Gains>),
}

/// Information gain's statistics: the label-weighted counts of the examples.
#[derive(Clone, Debug)]
struct Counts {
    /// Each class's total weight.
    totals: Vec<f64>,
    /// `class_num` blocks of one count per (feature, value).
    counts: Vec<f64>,
    /// The total weight at the last split check.
    checked: f64,
}

impl Counts {
    fn new(class_num: usize, values: usize) -> Self {
        Counts { totals: vec![0.0; class_num], counts: vec![0.0; class_num * values], checked: 0.0 }
    }
}

/// Model gain's statistics: each half's sums, and the sampled examples not
/// yet added to them (`refresh` adds them, in parallel).
#[derive(Clone, Debug)]
struct Gains {
    halves: [Half; 2],
    pending: Pending,
    /// The examples in the halves at the last split check.
    checked: f64,
}

/// One half's sums for model gain. With g each class's derivative of an
/// example's log-loss with respect to the class's logit, and h the second
/// derivative, a sum is `2 * class_num` numbers: each class's g, then each
/// class's h.
#[derive(Clone, Debug)]
struct Half {
    /// The examples' total label weight.
    n: f64,
    /// The weight of the examples having each tracked value.
    counts: Vec<f64>,
    /// The sum over all the examples.
    total: Vec<f64>,
    /// The sums over the examples having tracked values i and j, for each
    /// i <= j, in the order (0, 0), (0, 1), ..., (1, 1), (1, 2), ...
    pairs: Vec<f64>,
}

impl Half {
    fn new(stride: usize, layout: &Layout) -> Self {
        Half {
            n: 0.0,
            counts: vec![0.0; layout.size],
            total: vec![0.0; stride],
            pairs: vec![0.0; layout.row(layout.size) * stride],
        }
    }
}

/// Sampled examples waiting to be added to a leaf's halves.
#[derive(Clone, Debug, Default)]
struct Pending {
    /// Each example's tracked values, one per feature (`UNTRACKED` if not).
    values: Vec<u16>,
    halves: Vec<u8>,
    /// Each example's label weight.
    weights: Vec<f64>,
    /// Each example's terms of the sums.
    terms: Vec<f64>,
}

impl Gains {
    fn new(stride: usize, layout: &Layout) -> Self {
        Gains {
            halves: [Half::new(stride, layout), Half::new(stride, layout)],
            pending: Pending::default(),
            checked: 0.0,
        }
    }

    /// Adds the pending examples to their halves, each block of rows of the
    /// pair sums in parallel.
    fn flush(&mut self, layout: &Layout, stride: usize) {
        let pending = std::mem::take(&mut self.pending);
        let features = layout.blocks.last().map_or(0, |&(f, _)| f);
        let example = |e: usize| (&pending.values[e * features..][..features], &pending.terms[e * stride..][..stride]);
        for (h, half) in self.halves.iter_mut().enumerate() {
            let examples: Vec<usize> = (0..pending.halves.len()).filter(|&e| pending.halves[e] as usize == h).collect();
            for &e in &examples {
                let (values, terms) = example(e);
                half.n += pending.weights[e];
                half.total.iter_mut().zip(terms).for_each(|(s, t)| *s += t);
                for &i in values.iter().filter(|&&i| i != UNTRACKED) {
                    half.counts[i as usize] += pending.weights[e];
                }
            }
            let mut parts = Vec::with_capacity(layout.blocks.len());
            let mut rest = half.pairs.as_mut_slice();
            for w in layout.blocks.windows(2) {
                let ((first, row), (end, end_row)) = (w[0], w[1]);
                let (part, tail) = rest.split_at_mut((layout.row(end_row) - layout.row(row)) * stride);
                parts.push((first..end, layout.row(row), part));
                rest = tail;
            }
            // A block's rows are the tracked values of its features, so an
            // example's pairs in it are those of its values of those features
            // with its values of the same and later features.
            parts.into_par_iter().for_each(|(block, base, part)| {
                for &e in &examples {
                    let (values, terms) = example(e);
                    for f in block.clone() {
                        let i = values[f];
                        if i == UNTRACKED {
                            continue;
                        }
                        let row = layout.row(i as usize) - base;
                        for &j in values[f..].iter().filter(|&&j| j != UNTRACKED) {
                            let at = (row + (j - i) as usize) * stride;
                            part[at..at + stride].iter_mut().zip(terms).for_each(|(s, t)| *s += t);
                        }
                    }
                }
            });
        }
    }
}

/// Where model gain's pair sums are (see `Half`).
#[derive(Clone, Debug)]
struct Layout {
    /// Each value's index among the tracked values, all but those that
    /// cannot occur (pawns on the first or last rank), or `UNTRACKED`.
    tracked: Vec<u16>,
    /// Number of tracked values.
    size: usize,
    /// The first feature and first row (tracked value) of each block of rows
    /// of the pair sums, then the number of features and `size`.
    blocks: Vec<(usize, usize)>,
}

impl Layout {
    fn new(categories: &[u8]) -> Self {
        let chess = categories == encoding::CATEGORIES;
        let (mut tracked, mut firsts, mut size) = (Vec::new(), Vec::with_capacity(categories.len() + 1), 0);
        for (feature, &cat) in categories.iter().enumerate() {
            firsts.push(size);
            for value in 0..cat {
                let first_or_last_rank = feature < 8 || (56..64).contains(&feature);
                if chess && first_or_last_rank && (value == 1 || value == 7) {
                    tracked.push(UNTRACKED);
                } else {
                    tracked.push(size as u16);
                    size += 1;
                }
            }
        }
        assert!(size < UNTRACKED as usize, "too many values to track");
        firsts.push(size);
        let mut layout = Layout { tracked, size, blocks: vec![(0, 0)] };
        let pairs = layout.row(size);
        for feature in 1..=categories.len() {
            let (_, row) = *layout.blocks.last().unwrap();
            if feature == categories.len() || (layout.row(firsts[feature]) - layout.row(row)) * BLOCKS >= pairs {
                layout.blocks.push((feature, firsts[feature]));
            }
        }
        layout
    }

    /// Index of the pair (i, i): the number of pairs in the rows before i.
    fn row(&self, i: usize) -> usize {
        i * (2 * self.size + 1 - i) / 2
    }

    /// Index of the pair (i, j), i <= j.
    fn pair(&self, i: usize, j: usize) -> usize {
        self.row(i) + j - i
    }
}

#[derive(Clone, Debug)]
enum Node {
    /// Positions with `feature == value` go to `yes`, the others to `no`.
    Test {
        feature: usize,
        value: u8,
        yes: usize,
        no: usize,
    },
    Leaf {
        model: NodeModel,
        stats: Stats,
    },
}

#[derive(Clone, Debug)]
pub struct Ht {
    class_num: usize,
    categories: Vec<u8>,
    /// Index of each feature's first value among all the features' values.
    starts: Vec<usize>,
    /// Number of values of all the features together.
    values: usize,
    /// The CNBP leaves' Laplace smoothing, which their counts need to predict.
    alpha: f64,
    params: HtParams,
    layout: Layout,
    /// Node 0 is the root; children always come after their parent.
    nodes: Vec<Node>,
}

impl Ht {
    /// A tree of one leaf, `root`, which must be a logistic regression for
    /// model gain.
    pub fn new(root: NodeModel, categories: &[u8], params: HtParams) -> Self {
        let class_num = root.class_num();
        let alpha = match &root {
            NodeModel::Cnbp(m) => m.alpha(),
            NodeModel::Tan(m) => m.alpha(),
            NodeModel::Lr(_) | NodeModel::Probit(_) => 1.0,
            NodeModel::Fm(_) => panic!("FM nodes are routers only: a Hoeffding tree's leaves cannot be FMs"),
        };
        assert!(
            params.criterion == Criterion::InfoGain || matches!(root, NodeModel::Lr(_)),
            "model gain needs logistic regression leaves"
        );
        let mut ht = Ht::empty(class_num, categories, alpha, params);
        let stats = ht.fresh_stats();
        ht.nodes.push(Node::Leaf { model: root, stats });
        ht
    }

    /// A tree with no nodes yet.
    fn empty(class_num: usize, categories: &[u8], alpha: f64, params: HtParams) -> Self {
        let mut starts = Vec::with_capacity(categories.len());
        let mut values = 0;
        for &cat in categories {
            starts.push(values);
            values += cat as usize;
        }
        Ht {
            class_num,
            categories: categories.to_vec(),
            starts,
            values,
            alpha,
            params,
            layout: Layout::new(categories),
            nodes: Vec::new(),
        }
    }

    /// Statistics for a new leaf.
    fn fresh_stats(&self) -> Stats {
        match self.params.criterion {
            Criterion::InfoGain => Stats::Counts(Counts::new(self.class_num, self.values)),
            Criterion::ModelGain => Stats::Gains(Box::new(Gains::new(2 * self.class_num, &self.layout))),
        }
    }

    /// Index of the leaf that predicts `data`.
    pub fn leaf(&self, data: &[u8]) -> usize {
        let mut idx = 0;
        loop {
            match &self.nodes[idx] {
                Node::Test { feature, value, yes, no } => idx = if data[*feature] == *value { *yes } else { *no },
                Node::Leaf { .. } => return idx,
            }
        }
    }

    /// Each leaf's index and model.
    pub fn leaves(&self) -> impl Iterator<Item = (usize, &NodeModel)> {
        self.nodes.iter().enumerate().filter_map(|(i, n)| match n {
            Node::Leaf { model, .. } => Some((i, model)),
            Node::Test { .. } => None,
        })
    }

    pub fn leaf_count(&self) -> usize {
        self.leaves().count()
    }

    fn model(&self, idx: usize) -> &NodeModel {
        match &self.nodes[idx] {
            Node::Leaf { model, .. } => model,
            Node::Test { .. } => panic!("node {idx} is not a leaf"),
        }
    }

    /// Adds one data point to its leaf's statistics and model. `label` is a
    /// distribution over classes. For model gain, a position trained without
    /// its game (see `train_in_game`) counts as a game of its own.
    pub fn train(&mut self, data: &[u8], label: &[f64]) {
        let game = match self.params.criterion {
            Criterion::InfoGain => 0,
            Criterion::ModelGain => data.iter().fold(0, |h, &v| splitmix64(h ^ v as u64)),
        };
        self.train_in_game(data, label, game)
    }

    /// `train` for a position of game `game`, an id the game's positions
    /// share, by whose hash model gain samples games and puts them in halves.
    pub fn train_in_game(&mut self, data: &[u8], label: &[f64], game: u64) {
        let idx = self.leaf(data);
        let Node::Leaf { model, stats } = &mut self.nodes[idx] else { unreachable!() };
        match stats {
            Stats::Counts(stats) => {
                for (c, &w) in label.iter().enumerate().filter(|(_, w)| **w != 0.0) {
                    stats.totals[c] += w;
                    let block = &mut stats.counts[c * self.values..(c + 1) * self.values];
                    for (&start, &v) in self.starts.iter().zip(data) {
                        block[start + v as usize] += w;
                    }
                }
                model.train(data, label);
            }
            Stats::Gains(gains) => {
                let NodeModel::Lr(lr) = model else { unreachable!() };
                // As `Logreg::train`, keeping the prediction for the sums.
                let mut p = [0.0; 256];
                let p = &mut p[..self.class_num];
                lr.predict(data, p);
                let hash = splitmix64(game);
                if (hash >> 11) as f64 / ((1u64 << 53) as f64) < self.params.sample {
                    let pending = &mut gains.pending;
                    let weight: f64 = label.iter().sum();
                    pending.halves.push((hash & 1) as u8);
                    pending.weights.push(weight);
                    let tracked = &self.layout.tracked;
                    pending.values.extend(self.starts.iter().zip(data).map(|(&s, &v)| tracked[s + v as usize]));
                    pending.terms.extend(p.iter().zip(label).map(|(&p, &y)| weight * p - y));
                    pending.terms.extend(p.iter().map(|&p| weight * p * (1.0 - p)));
                }
                lr.step(data, p, label, 1.0, 0.0);
            }
        }
    }

    /// Checks the leaves for splits (until the tree has `max_leaves`), then
    /// makes the leaves ready to predict.
    pub fn refresh(&mut self) {
        self.flush();
        let leaves: Vec<usize> =
            (0..self.nodes.len()).filter(|&i| matches!(self.nodes[i], Node::Leaf { .. })).collect();
        for idx in leaves {
            if self.leaf_count() >= self.params.max_leaves {
                break;
            }
            match &self.nodes[idx] {
                Node::Leaf { stats: Stats::Counts(_), .. } => self.check(idx),
                _ => self.check_gains(idx),
            }
        }
        for node in &mut self.nodes {
            if let Node::Leaf { model, .. } = node {
                model.refresh();
            }
        }
    }

    /// Adds model gain's pending examples to the leaves' statistics.
    fn flush(&mut self) {
        for node in &mut self.nodes {
            if let Node::Leaf { stats: Stats::Gains(gains), .. } = node {
                gains.flush(&self.layout, 2 * self.class_num);
            }
        }
    }

    /// Splits leaf `idx` if it has added `grace` positions since its last
    /// check and the bound allows a split; logs the check.
    fn check(&mut self, idx: usize) {
        let Node::Leaf { stats: Stats::Counts(stats), .. } = &self.nodes[idx] else { return };
        let n: f64 = stats.totals.iter().sum();
        if n <= 0.0 || n - stats.checked < self.params.grace {
            return;
        }
        let found = self.info_gain_tests(stats, n);
        if let Node::Leaf { stats: Stats::Counts(stats), .. } = &mut self.nodes[idx] {
            stats.checked = n;
        }
        let Some(((feature, value, gain), second)) = found else { return };
        let p = &self.params;
        let range = (self.class_num as f64).log2();
        let bound = (range * range * (1.0 / p.delta).ln() / (2.0 * n / p.positions_per_sample)).sqrt();
        let test = self.describe(feature, value);
        let numbers = format!("gain {gain:.6} bits (second {second:.6}, bound {bound:.6}, {n:.0} positions)");
        if gain - second > bound || (bound < p.tie && (gain > bound || !p.pre_prune)) {
            let (yes, no) = self.split(idx, feature, value);
            eprintln!("split leaf {idx} on {test} into leaves {yes} (yes) and {no} (no): {numbers}");
        } else {
            eprintln!("leaf {idx} not split: best {test}, {numbers}");
        }
    }

    /// Model gain's `check`: splits leaf `idx` if it has added `grace`
    /// positions since its last check and the best test on half A gains
    /// more than `gain_threshold` per position on half B; logs the check.
    fn check_gains(&mut self, idx: usize) {
        let Node::Leaf { stats: Stats::Gains(gains), .. } = &self.nodes[idx] else { return };
        let [a, b] = &gains.halves;
        let n = a.n + b.n;
        if a.n <= 0.0 || b.n <= 0.0 || n - gains.checked < self.params.grace {
            return;
        }
        let tracked = |feature: usize, value: u8| self.layout.tracked[self.starts[feature] + value as usize];
        let found = self.best_tests(|feature, value| {
            let i = tracked(feature, value);
            if i == UNTRACKED {
                return None;
            }
            let yes = a.counts[i as usize];
            (yes > 1e-9 * a.n && a.n - yes > 1e-9 * a.n).then(|| self.gain(a, i as usize))
        });
        // Per position: A's gains for choosing, B's for deciding.
        let found = found.map(|((feature, value, gain), second)| {
            let i = tracked(feature, value) as usize;
            let second = second.map(|(f, v, g)| (f, v, g / a.n));
            (feature, value, gain / a.n, second, self.cross_gain(a, b, i) / b.n, self.gain(b, i) / b.n)
        });
        let (n_a, n_b) = (a.n, b.n);
        if let Node::Leaf { stats: Stats::Gains(gains), .. } = &mut self.nodes[idx] {
            gains.checked = n;
        }
        let Some((feature, value, gain, second, on_b, own)) = found else { return };
        let test = self.describe(feature, value);
        let second = second.map_or("none".to_string(), |(f, v, g)| format!("{} {g:.3e}", self.describe(f, v)));
        let numbers = format!(
            "gain on A {gain:.3e} nats per position (second {second}), on B {on_b:.3e} (B's own steps {own:.3e}), \
             {n_a:.0} + {n_b:.0} positions"
        );
        if on_b > self.params.gain_threshold {
            let (yes, no) = self.split(idx, feature, value);
            eprintln!("split leaf {idx} on {test} into leaves {yes} (yes) and {no} (no): {numbers}");
        } else {
            eprintln!("leaf {idx} not split: best {test}, {numbers}");
        }
    }

    /// The test with the highest information gain (bits) on `stats`, which
    /// hold `n` positions, and the highest gain of a test on another feature.
    fn info_gain_tests(&self, stats: &Counts, n: f64) -> Option<((usize, u8, f64), f64)> {
        let whole = entropy(&stats.totals, n);
        let (mut yes, mut no) = (vec![0.0; self.class_num], vec![0.0; self.class_num]);
        let found = self.best_tests(|feature, value| {
            let start = self.starts[feature];
            for c in 0..self.class_num {
                yes[c] = stats.counts[c * self.values + start + value as usize];
                no[c] = (stats.totals[c] - yes[c]).max(0.0);
            }
            let n_yes: f64 = yes.iter().sum();
            let n_no: f64 = no.iter().sum();
            if n_yes <= 1e-9 * n || n_no <= 1e-9 * n {
                return None;
            }
            Some(whole - n_yes / n * entropy(&yes, n_yes) - n_no / n * entropy(&no, n_no))
        });
        found.map(|(best, second)| (best, second.map_or(0.0, |(_, _, g)| g.max(0.0))))
    }

    /// The test with the highest `gain` (None for a test that does not send
    /// positions both ways) and the best test on another feature; None if no
    /// test has a gain. As in VFDT, the runner-up is another feature (MOA's
    /// binary tests of nominal attributes do the same): two values of one
    /// square, empty or a white pawn say, split almost alike when the square
    /// rarely holds anything else, so they would always tie.
    fn best_tests(
        &self,
        mut gain: impl FnMut(usize, u8) -> Option<f64>,
    ) -> Option<((usize, u8, f64), Option<(usize, u8, f64)>)> {
        let (mut best, mut second) = (None::<(usize, u8, f64)>, None::<(usize, u8, f64)>);
        let mut runner_up = |test: (usize, u8, f64)| {
            if second.is_none_or(|(_, _, g)| test.2 > g) {
                second = Some(test);
            }
        };
        for (feature, &cat) in self.categories.iter().enumerate() {
            let mut feature_best = None::<(u8, f64)>;
            // Either value of a two-valued feature splits the same way.
            for value in 0..if cat == 2 { 1 } else { cat } {
                let Some(gain) = gain(feature, value) else { continue };
                if feature_best.is_none_or(|(_, g)| gain > g) {
                    feature_best = Some((value, gain));
                }
            }
            let Some((value, gain)) = feature_best else { continue };
            match best {
                Some((_, _, g)) if gain <= g => runner_up((feature, value, gain)),
                _ => {
                    if let Some(old) = best {
                        runner_up(old);
                    }
                    best = Some((feature, value, gain));
                }
            }
        }
        best.map(|b| (b, second))
    }

    /// `half`'s sums for weight `w` (0 the bias, else tracked value w - 1)
    /// over its examples on test `i`'s yes side (those having tracked value
    /// i), and over all its examples.
    fn sums<'a>(&self, half: &'a Half, i: usize, w: usize) -> (&'a [f64], &'a [f64]) {
        let stride = 2 * self.class_num;
        let pair = |a: usize, b: usize| &half.pairs[self.layout.pair(a.min(b), a.max(b)) * stride..][..stride];
        match w {
            0 => (pair(i, i), &half.total),
            _ => (pair(i, w - 1), pair(w - 1, w - 1)),
        }
    }

    /// The gain (nats) on `half`'s examples of a Newton step on each weight
    /// on each side of test `i` (a tracked value) over one on all of them.
    fn gain(&self, half: &Half, i: usize) -> f64 {
        let (c, lambda) = (self.class_num, self.params.lambda);
        let newton = |g: f64, h: f64| if h.max(0.0) + lambda > 0.0 { g * g / (h.max(0.0) + lambda) } else { 0.0 };
        let mut sum = 0.0;
        for w in 0..=self.layout.size {
            let (yes, all) = self.sums(half, i, w);
            for k in 0..c {
                let (g, h, g_yes, h_yes) = (all[k], all[c + k], yes[k], yes[c + k]);
                sum += newton(g_yes, h_yes) + newton(g - g_yes, h - h_yes) - newton(g, h);
            }
        }
        sum / 2.0
    }

    /// How much more the loss on `b`'s examples would fall (nats, to second
    /// order) if each side of test `i` took the Newton steps fitted on its
    /// examples in `a` than if all took the steps fitted on all of `a`.
    fn cross_gain(&self, a: &Half, b: &Half, i: usize) -> f64 {
        let (c, lambda) = (self.class_num, self.params.lambda);
        // B's loss change under the step fitted on A's sums.
        let change = |g_a: f64, h_a: f64, g_b: f64, h_b: f64| {
            let h = h_a.max(0.0) + lambda;
            let step = if h > 0.0 { -g_a / h } else { 0.0 };
            g_b * step + h_b.max(0.0) * step * step / 2.0
        };
        let mut sum = 0.0;
        for w in 0..=self.layout.size {
            let ((yes_a, all_a), (yes_b, all_b)) = (self.sums(a, i, w), self.sums(b, i, w));
            for k in 0..c {
                let (g, h) = (k, c + k);
                sum += change(all_a[g], all_a[h], all_b[g], all_b[h])
                    - change(yes_a[g], yes_a[h], yes_b[g], yes_b[h])
                    - change(all_a[g] - yes_a[g], all_a[h] - yes_a[h], all_b[g] - yes_b[g], all_b[h] - yes_b[h]);
            }
        }
        sum
    }

    /// Turns leaf `idx` into a test; both children start with copies of its
    /// model and empty statistics. Returns the children.
    fn split(&mut self, idx: usize, feature: usize, value: u8) -> (usize, usize) {
        let (yes, no) = (self.nodes.len(), self.nodes.len() + 1);
        let Node::Leaf { model, .. } = std::mem::replace(&mut self.nodes[idx], Node::Test { feature, value, yes, no })
        else {
            unreachable!()
        };
        let stats = self.fresh_stats();
        self.nodes.push(Node::Leaf { model: model.clone(), stats });
        let stats = self.fresh_stats();
        self.nodes.push(Node::Leaf { model, stats });
        (yes, no)
    }

    /// `feature == value`, in words too for the chess encoding.
    fn describe(&self, feature: usize, value: u8) -> String {
        if self.categories == encoding::CATEGORIES {
            format!("feature {feature} == {value} ({})", chess_test(feature, value))
        } else {
            format!("feature {feature} == {value}")
        }
    }

    /// For model gain, once the tree has split, logs the share of a round's
    /// moves whose position is in another leaf than the position before it
    /// (the seams search can exploit), and each leaf's share of the
    /// positions. `games` are the round's games, each its positions in order.
    pub fn observe_games<P: AsRef<[u8]>>(&self, games: &[&[P]]) {
        if self.params.criterion != Criterion::ModelGain || self.leaf_count() < 2 {
            return;
        }
        let (mut positions, mut moves, mut seams) = (vec![0usize; self.nodes.len()], 0, 0);
        for game in games {
            let leaves: Vec<usize> = game.iter().map(|p| self.leaf(p.as_ref())).collect();
            leaves.iter().for_each(|&l| positions[l] += 1);
            moves += leaves.len().saturating_sub(1);
            seams += leaves.windows(2).filter(|w| w[0] != w[1]).count();
        }
        let total = positions.iter().sum::<usize>().max(1) as f64;
        let shares: Vec<String> =
            self.leaves().map(|(i, _)| format!("{i} {:.1}%", 100.0 * positions[i] as f64 / total)).collect();
        eprintln!(
            "seams: {:.2}% of {moves} moves change leaf; positions by leaf: {}",
            100.0 * seams as f64 / moves.max(1) as f64,
            shares.join(", ")
        );
    }

    /// Writes the probability of each class.
    pub fn predict(&self, data: &[u8], out: &mut [f64]) {
        self.model(self.leaf(data)).predict(data, out)
    }

    /// Writes the log of each class's unnormalised probability.
    pub fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        self.model(self.leaf(data)).predict_log(data, out)
    }

    /// Leaf `idx`'s log-probabilities for `data`, wherever `data` belongs.
    pub fn node_predict_log(&self, idx: usize, data: &[u8], out: &mut [f64]) {
        self.model(idx).predict_log(data, out)
    }

    /// Lets the leaves adapt faster while predicting the same (see
    /// `NodeModel::forget`), and scales their statistics down alike.
    pub fn forget(&mut self, factor: f64) {
        self.flush();
        for node in &mut self.nodes {
            if let Node::Leaf { model, stats } = node {
                model.forget(factor);
                match stats {
                    Stats::Counts(stats) => {
                        stats.totals.iter_mut().chain(stats.counts.iter_mut()).for_each(|x| *x /= factor);
                        stats.checked /= factor;
                    }
                    Stats::Gains(gains) => {
                        for half in &mut gains.halves {
                            half.n /= factor;
                            let sums = half.counts.iter_mut().chain(&mut half.total).chain(&mut half.pairs);
                            sums.for_each(|x| *x /= factor);
                        }
                        gains.checked /= factor;
                    }
                }
            }
        }
    }

    /// The tests and the leaves' models (CNBP counts, or logistic regressions
    /// in their own format), without the split statistics.
    pub fn to_text(&self) -> String {
        let mut s = String::new();
        writeln!(s, "hoeffding\n{}\n{}", self.class_num, self.categories.len()).unwrap();
        for cat in &self.categories {
            writeln!(s, "{cat}").unwrap();
        }
        writeln!(s, "{:.6}\n{}", self.alpha, self.nodes.len()).unwrap();
        for node in &self.nodes {
            match node {
                Node::Test { feature, value, yes, no } => writeln!(s, "1\n{feature}\n{value}\n{yes}\n{no}").unwrap(),
                Node::Leaf { model, .. } => {
                    writeln!(s, "0").unwrap();
                    model.write(&mut s);
                }
            }
        }
        s
    }

    pub fn from_text(text: &str, use_probs: bool) -> io::Result<Self> {
        let mut t = Tokens::new(text);
        if t.next::<String>()? != "hoeffding" {
            return Err(invalid("not a Hoeffding tree model"));
        }
        let class_num: usize = t.next()?;
        let cat_num: usize = t.next()?;
        let categories = (0..cat_num).map(|_| t.next::<u8>()).collect::<io::Result<Vec<_>>>()?;
        let alpha: f64 = t.next()?;
        let node_num: usize = t.next()?;
        let root = NodeModel::Cnbp(Cnbp::new(class_num, &categories, alpha, use_probs));
        let mut ht = Ht::new(root, &categories, HtParams::default());
        ht.nodes.clear();
        for n in 0..node_num {
            let node = match t.next::<u32>()? {
                0 => {
                    let model = if t.peek() == Some("logreg") {
                        let lr = Logreg::from_tokens(&mut t)?;
                        if lr.class_num() != class_num {
                            return Err(invalid(format!(
                                "leaf {n} has {} classes, expected {class_num}",
                                lr.class_num()
                            )));
                        }
                        NodeModel::Lr(lr)
                    } else {
                        let mut cnbp = Cnbp::new(class_num, &categories, alpha, use_probs);
                        cnbp.read_counts(&mut t)?;
                        NodeModel::Cnbp(cnbp)
                    };
                    Node::Leaf { model, stats: ht.fresh_stats() }
                }
                1 => {
                    let (feature, value, yes, no): (usize, u8, usize, usize) =
                        (t.next()?, t.next()?, t.next()?, t.next()?);
                    if feature >= cat_num || value >= categories[feature] {
                        return Err(invalid(format!("node {n} tests feature {feature} for value {value}")));
                    }
                    // Children come after their parent, so routing ends.
                    if yes <= n || no <= n || yes >= node_num || no >= node_num {
                        return Err(invalid(format!("node {n} has a child out of order")));
                    }
                    Node::Test { feature, value, yes, no }
                }
                kind => return Err(invalid(format!("node {n} has unknown type {kind}"))),
            };
            ht.nodes.push(node);
        }
        if ht.nodes.is_empty() {
            return Err(invalid("a tree with no nodes"));
        }
        Ok(ht)
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        std::fs::write(path, self.to_text())
    }

    pub fn load(path: &Path, use_probs: bool) -> io::Result<Self> {
        Self::from_text(&std::fs::read_to_string(path)?, use_probs)
    }

    /// The whole tree with its split statistics, unrounded, for an exact
    /// `--resume` (see `state`).
    pub fn to_state(&self) -> Writer {
        let mut w = Writer::new("ht");
        let (_, leaf) = self.leaves().next().expect("a tree has a leaf");
        w.str(&state_options(&self.params, leaf));
        w.usize(self.class_num);
        w.u8s(&self.categories);
        w.f64(self.alpha);
        w.usize(self.nodes.len());
        for node in &self.nodes {
            match node {
                Node::Test { feature, value, yes, no } => {
                    w.u8(1);
                    w.usize(*feature);
                    w.u8(*value);
                    w.usize(*yes);
                    w.usize(*no);
                }
                Node::Leaf { model, stats } => {
                    w.u8(0);
                    model.write_state(&mut w);
                    match stats {
                        Stats::Counts(counts) => {
                            w.u8(0);
                            w.f64s(&counts.totals);
                            w.f64s(&counts.counts);
                            w.f64(counts.checked);
                        }
                        Stats::Gains(gains) => {
                            w.u8(1);
                            for half in &gains.halves {
                                w.f64(half.n);
                                w.f64s(&half.counts);
                                w.f64s(&half.total);
                                w.f64s(&half.pairs);
                            }
                            let pending = &gains.pending;
                            w.u16s(&pending.values);
                            w.u8s(&pending.halves);
                            w.f64s(&pending.weights);
                            w.f64s(&pending.terms);
                            w.f64(gains.checked);
                        }
                    }
                }
            }
        }
        w
    }

    /// Reads a state written by `to_state` by a tree trained with `params`
    /// over `categories` from `root` (checked: the leaves' kind and settings).
    pub fn from_state(data: &[u8], root: &NodeModel, categories: &[u8], params: HtParams) -> io::Result<Self> {
        if params.criterion == Criterion::ModelGain && !matches!(root, NodeModel::Lr(_)) {
            return Err(invalid("model gain needs logistic regression leaves"));
        }
        let mut r = Reader::new(data, "ht")?;
        r.expect_str("options", &state_options(&params, root))?;
        let class_num = r.usize()?;
        if r.u8s()? != categories || class_num != root.class_num() {
            return Err(invalid("the state's features or classes differ from this run's"));
        }
        let alpha = r.f64()?;
        let mut ht = Ht::empty(class_num, categories, alpha, params);
        let node_num = r.usize()?;
        for n in 0..node_num {
            let node = match r.u8()? {
                0 => {
                    let model = NodeModel::read_state(&mut r)?;
                    let stats = match (r.u8()?, ht.fresh_stats()) {
                        (0, Stats::Counts(mut counts)) => {
                            read_sums(&mut r, &mut counts.totals)?;
                            read_sums(&mut r, &mut counts.counts)?;
                            counts.checked = r.f64()?;
                            Stats::Counts(counts)
                        }
                        (1, Stats::Gains(mut gains)) => {
                            for half in &mut gains.halves {
                                half.n = r.f64()?;
                                read_sums(&mut r, &mut half.counts)?;
                                read_sums(&mut r, &mut half.total)?;
                                read_sums(&mut r, &mut half.pairs)?;
                            }
                            gains.pending =
                                Pending { values: r.u16s()?, halves: r.u8s()?, weights: r.f64s()?, terms: r.f64s()? };
                            gains.checked = r.f64()?;
                            Stats::Gains(gains)
                        }
                        _ => return Err(invalid(format!("leaf {n} has statistics of another criterion"))),
                    };
                    if model.options() != root.options() {
                        return Err(invalid(format!("leaf {n} is not of the root's kind")));
                    }
                    Node::Leaf { model, stats }
                }
                1 => {
                    let (feature, value, yes, no) = (r.usize()?, r.u8()?, r.usize()?, r.usize()?);
                    if feature >= categories.len() || value >= categories[feature] {
                        return Err(invalid(format!("node {n} tests feature {feature} for value {value}")));
                    }
                    if yes <= n || no <= n || yes >= node_num || no >= node_num {
                        return Err(invalid(format!("node {n} has a child out of order")));
                    }
                    Node::Test { feature, value, yes, no }
                }
                kind => return Err(invalid(format!("node {n} has unknown type {kind}"))),
            };
            ht.nodes.push(node);
        }
        r.finish()?;
        if ht.leaf_count() == 0 {
            return Err(invalid("a tree with no leaves"));
        }
        Ok(ht)
    }

    pub fn save_state(&self, path: &Path) -> io::Result<()> {
        self.to_state().save(path)
    }

    pub fn load_state(path: &Path, root: &NodeModel, categories: &[u8], params: HtParams) -> io::Result<Self> {
        Self::from_state(&std::fs::read(path)?, root, categories, params)
    }
}

/// What a state must have been trained with to resume: the split options
/// and the leaves' kind and settings.
fn state_options(params: &HtParams, leaf: &NodeModel) -> String {
    format!("{params:?}, leaves {}", leaf.options())
}

/// Reads sums of the shape of `sums` into it.
fn read_sums(r: &mut Reader, sums: &mut Vec<f64>) -> io::Result<()> {
    let saved = r.f64s()?;
    if saved.len() != sums.len() {
        return Err(invalid("statistics of the wrong shape in state file"));
    }
    *sums = saved;
    Ok(())
}

/// `feature == value` in words, for the chess encoding (see `encoding`).
fn chess_test(feature: usize, value: u8) -> String {
    const ROLES: [&str; 6] = ["pawn", "knight", "bishop", "rook", "queen", "king"];
    const CASTLING: [&str; 4] = ["white long", "black long", "white short", "black short"];
    let (v, colours) = (value as usize, ["white", "black"]);
    match feature {
        0..64 => {
            let square = format!("{}{}", (b'a' + feature as u8 % 8) as char, feature / 8 + 1);
            match v {
                0 => format!("{square} empty"),
                _ => format!("{square} {} {}", colours[(v - 1) / 6], ROLES[(v - 1) % 6]),
            }
        }
        64 => format!("{} to move", colours[v]),
        65 if v == 0 => "no double push".to_string(),
        65 => format!("double push on file {}", (b'a' + value - 1) as char),
        _ => format!("{} castling {}", CASTLING[feature - 66], ["lost", "kept"][v]),
    }
}

/// Entropy in bits of the class distribution `counts`, which sum to `n`.
fn entropy(counts: &[f64], n: f64) -> f64 {
    -counts.iter().filter(|&&c| c > 0.0).map(|&c| c / n * (c / n).log2()).sum::<f64>()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CATS: [u8; 3] = [3, 4, 2];

    fn params() -> HtParams {
        HtParams { tie: 0.05, grace: 0.0, positions_per_sample: 1.0, ..HtParams::default() }
    }

    fn tree(lr: bool, use_probs: bool) -> Ht {
        let root = if lr {
            NodeModel::Lr(Logreg::new(2, &CATS, 0.1))
        } else {
            NodeModel::Cnbp(Cnbp::new(2, &CATS, 1.0, use_probs))
        };
        Ht::new(root, &CATS, params())
    }

    /// A model gain tree with LR leaves and the default model gain options.
    fn gain_tree() -> Ht {
        let params = HtParams { criterion: Criterion::ModelGain, ..params() };
        Ht::new(NodeModel::Lr(Logreg::new(2, &CATS, 0.1)), &CATS, params)
    }

    /// A fixed stream of pseudo-random numbers in [0, 2^31).
    fn next(x: &mut u64) -> u64 {
        *x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *x >> 33
    }

    /// Uniform random features; if `decides`, class 0 with probability 0.9
    /// when feature 1 is 2 and 0.1 otherwise, else always 0.5.
    fn examples(n: usize, decides: bool, seed: u64) -> Vec<([u8; 3], [f64; 2])> {
        let mut x = seed;
        (0..n)
            .map(|_| {
                let data = [(next(&mut x) % 3) as u8, (next(&mut x) % 4) as u8, (next(&mut x) % 2) as u8];
                let p = match (decides, data[1]) {
                    (false, _) => 0.5,
                    (true, 2) => 0.9,
                    _ => 0.1,
                };
                let first = (next(&mut x) % 1000) as f64 / 1000.0 < p;
                (data, if first { [1.0, 0.0] } else { [0.0, 1.0] })
            })
            .collect()
    }

    /// Games of `length` positions, which share the game's uniform random
    /// features 0 and 1 (as a game's positions share a king's square for a
    /// while) and its result, feature 2 being each position's own: class 0
    /// with probability 0.5, or if `interacts`, when feature 0 is 0 with
    /// probability [0.9, 0.1, 0.7, 0.3][feature 1]. Feature 1 then matters
    /// only when feature 0 is 0, which an LR cannot fit, and feature 0 says
    /// nothing by itself. The positions come with their game's id, in game
    /// order.
    fn games(n: usize, length: usize, interacts: bool, seed: u64) -> Vec<([u8; 3], [f64; 2], u64)> {
        let mut x = seed;
        let mut out = Vec::with_capacity(n * length);
        for game in 0..n as u64 {
            let (a, b) = ((next(&mut x) % 3) as u8, (next(&mut x) % 4) as u8);
            let p = if interacts && a == 0 { [0.9, 0.1, 0.7, 0.3][b as usize] } else { 0.5 };
            let label = if ((next(&mut x) % 1000) as f64) < 1000.0 * p { [1.0, 0.0] } else { [0.0, 1.0] };
            out.extend((0..length).map(|_| ([a, b, (next(&mut x) % 2) as u8], label, seed << 32 | game)));
        }
        out
    }

    fn all_inputs() -> Vec<[u8; 3]> {
        (0..3).flat_map(|a| (0..4).flat_map(move |b| (0..2).map(move |c| [a, b, c]))).collect()
    }

    /// The tree's predictions for every input.
    fn predictions(ht: &Ht) -> Vec<[f64; 2]> {
        let predict = |data: &[u8; 3]| {
            let mut out = [0.0; 2];
            ht.predict(data, &mut out);
            out
        };
        all_inputs().iter().map(predict).collect()
    }

    /// A tree read back from its state, statistics and all, trains on
    /// exactly as the original (`--resume`), even mid-round with model
    /// gain's examples pending. Other options or leaves are refused.
    #[test]
    fn state_round_trip_trains_on_identically() {
        let data = games(4000, 5, true, 5);
        let gain = HtParams { criterion: Criterion::ModelGain, ..params() };
        let lr = || NodeModel::Lr(Logreg::new(2, &CATS, 0.1));
        let trees = [
            (tree(false, true), NodeModel::Cnbp(Cnbp::new(2, &CATS, 1.0, true)), params()),
            (tree(true, false), lr(), params()),
            (gain_tree(), lr(), gain),
        ];
        for (mut ht, root, params) in trees {
            for round in data.chunks(2500) {
                let (early, late) = round.split_at(1000);
                early.iter().for_each(|(data, label, game)| ht.train_in_game(data, label, *game));
                let state = ht.to_state().into_bytes();
                let mut back = Ht::from_state(&state, &root, &CATS, params).unwrap();
                assert_eq!(back.to_state().into_bytes(), state);
                for tree in [&mut ht, &mut back] {
                    late.iter().for_each(|(data, label, game)| tree.train_in_game(data, label, *game));
                    tree.refresh();
                }
                assert!(back.to_state().into_bytes() == ht.to_state().into_bytes(), "{params:?}");
                assert_eq!(back.to_text(), ht.to_text());
            }
            assert!(ht.leaf_count() > 1, "{params:?}");
            let state = ht.to_state().into_bytes();
            assert!(Ht::from_state(&state, &root, &CATS, HtParams { grace: 1.0, ..params }).is_err());
            let other = NodeModel::Lr(Logreg::new(2, &CATS, 0.2));
            assert!(Ht::from_state(&state, &other, &CATS, params).is_err());
        }
    }

    #[test]
    fn splits_on_the_feature_that_decides() {
        for lr in [false, true] {
            let mut ht = tree(lr, false);
            for (data, label) in examples(5000, true, 1) {
                ht.train(&data, &label);
            }
            ht.refresh();
            assert!(matches!(ht.nodes[0], Node::Test { feature: 1, value: 2, .. }), "{:?}", ht.nodes[0]);
            assert_eq!(ht.leaf_count(), 2);
        }
    }

    /// With labels independent of the features, the bound never separates
    /// the best test from the runner-up, nor (`pre_prune`) from not
    /// splitting; only a tie without pre-pruning splits, as in VFDT.
    #[test]
    fn no_split_without_dependence() {
        let leaves = |params: HtParams| {
            let mut ht = Ht::new(NodeModel::Cnbp(Cnbp::new(2, &CATS, 1.0, false)), &CATS, params);
            for round in 0..50 {
                for (data, label) in examples(2000, false, round) {
                    ht.train(&data, &label);
                }
                ht.refresh();
            }
            ht.leaf_count()
        };
        assert_eq!(leaves(HtParams { tie: 0.0, ..params() }), 1);
        assert_eq!(leaves(HtParams { pre_prune: true, ..params() }), 1);
        assert!(leaves(params()) > 1);
    }

    /// Model gain splits on the test the other feature's effect depends on,
    /// where information gain splits on that other feature, which says more
    /// about the result.
    #[test]
    fn model_gain_splits_where_the_weights_differ() {
        let data = games(4000, 5, true, 5);
        let mut ht = gain_tree();
        let mut info = tree(true, false);
        for &(data, label, game) in &data {
            ht.train_in_game(&data, &label, game);
            info.train(&data, &label);
        }
        ht.refresh();
        info.refresh();
        assert!(matches!(ht.nodes[0], Node::Test { feature: 0, value: 0, .. }), "{:?}", ht.nodes[0]);
        assert!(matches!(info.nodes[0], Node::Test { feature: 1, .. }), "{:?}", info.nodes[0]);
    }

    /// The default threshold and λ with checks after every round: no split
    /// on labels independent of the features, whether a game has one
    /// position or 50 that share its result, where the gain of B's own steps
    /// (the in-sample gain on held-out games) would often be above the
    /// threshold.
    #[test]
    fn model_gain_ignores_noise() {
        let mut own_above = 0;
        for (length, seed) in [(1, 0), (1, 1), (50, 2), (50, 3)] {
            let mut ht = gain_tree();
            for round in 0..50 {
                for (data, label, game) in games(2000 / length, length, false, seed * 1000 + round) {
                    ht.train_in_game(&data, &label, game);
                }
                ht.refresh();
                assert_eq!(ht.leaf_count(), 1, "games of {length}, seed {seed}, round {round}");
                let Node::Leaf { stats: Stats::Gains(gains), .. } = &ht.nodes[0] else { unreachable!() };
                let [a, b] = &gains.halves;
                let best = (0..ht.layout.size).max_by(|&i, &j| ht.gain(a, i).total_cmp(&ht.gain(a, j))).unwrap();
                if ht.gain(b, best) / b.n > ht.params.gain_threshold {
                    own_above += 1;
                }
            }
        }
        assert!(own_above >= 20, "{own_above}");
    }

    /// Until it splits, a model gain tree is the logistic regression it
    /// started with, trained alike.
    #[test]
    fn model_gain_trains_its_leaf_as_lr() {
        let mut ht = gain_tree();
        let mut lr = Logreg::new(2, &CATS, 0.1);
        for (data, label, game) in games(1000, 5, false, 7) {
            ht.train_in_game(&data, &label, game);
            lr.train(&data, &label);
        }
        ht.refresh();
        assert_eq!(ht.leaf_count(), 1);
        let NodeModel::Lr(leaf) = ht.model(0) else { unreachable!() };
        assert_eq!(leaf.to_text(), lr.to_text());
    }

    #[test]
    fn a_split_keeps_the_predictions() {
        let mut ht = tree(false, false);
        for (data, label) in examples(5000, true, 2) {
            ht.train(&data, &label);
        }
        let before = predictions(&ht);
        ht.refresh();
        assert_eq!(ht.leaf_count(), 2);
        assert_eq!(predictions(&ht), before);
    }

    #[test]
    fn text_round_trip() {
        for (lr, use_probs) in [(false, false), (false, true), (true, false)] {
            let mut ht = tree(lr, use_probs);
            for (data, label) in examples(5000, true, 3) {
                ht.train(&data, &label);
            }
            ht.refresh();
            for (data, label) in examples(500, true, 4) {
                ht.train(&data, &label);
            }
            ht.refresh();
            let back = Ht::from_text(&ht.to_text(), use_probs).unwrap();
            assert_eq!(back.to_text(), ht.to_text());
            for data in all_inputs() {
                let (mut a, mut b) = ([0.0; 2], [0.0; 2]);
                ht.predict_log(&data, &mut a);
                back.predict_log(&data, &mut b);
                assert_eq!(a, b, "{data:?}");
            }
        }
    }
}
