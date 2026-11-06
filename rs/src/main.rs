use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use shakmaty::uci::UciMove;
use shakmaty::{CastlingMode, Chess, Position};

use chessai::bdd::Bdd;
use chessai::bdt::Bdt;
use chessai::calibration::Calibration;
use chessai::delegation::{self, DelegationParams, ErrorMeasure, ErrorTarget, NodeKind, NodeModel, RouteTarget, SplitRule};
use chessai::classifier::{Classifier, Uniform};
use chessai::cnbp::Cnbp;
use chessai::logreg::Logreg;
use chessai::moe::Moe;
use chessai::encoding::{self, CATEGORIES, Encoded};
use chessai::game::Game;
use chessai::games;
use chessai::game::GameResult;
use chessai::player::{self, Player, Policy};
use chessai::search::Value;
use chessai::selfplay;
use chessai::tournament;

#[derive(Parser)]
#[command(about = "chessai: categorical Bayesian chess models")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Play a match between two players, alternating colours, and report the
    /// score and an Elo difference estimate. A player is `cnbp:PATH`,
    /// `bdt:PATH`, `bdd:PATH` (a saved model) or `rand`.
    Arena {
        a: String,
        b: String,
        #[arg(long, default_value_t = 1000)]
        games: usize,
        #[arg(long, value_enum, default_value_t = Policy::WldScoreJitter10)]
        a_policy: Policy,
        #[arg(long, value_enum, default_value_t = Policy::WldScoreJitter10)]
        b_policy: Policy,
        /// Softmax temperature for the wld-score-softmax policy.
        #[arg(long, default_value_t = 0.01)]
        temperature: f64,
        /// Predict from cached probabilities (faster) instead of raw counts.
        #[arg(long)]
        use_probs: bool,
        /// Random seed; chosen and printed if not given.
        #[arg(long)]
        seed: Option<u64>,
        /// Also write the games, in the self-play round format.
        #[arg(long)]
        out: Option<PathBuf>,
        /// Search depth of player A in plies (0: score the moves directly;
        /// 1 or more: alpha-beta search, needs a log-odds policy).
        #[arg(long, default_value_t = 0)]
        a_depth: u32,
        #[arg(long, default_value_t = 0)]
        b_depth: u32,
        /// How player A's search values positions (needs --a-depth).
        #[arg(long, value_enum, default_value_t = Value::LogOdds)]
        a_value: Value,
        #[arg(long, value_enum, default_value_t = Value::LogOdds)]
        b_value: Value,
        /// Calibration of player A's probabilities for the score value,
        /// SLOPE_WINNER,SLOPE_DECISIVE,OFFSET_WHITE,OFFSET_BLACK as printed by
        /// `eval-loss --fit-calibration`.
        #[arg(long)]
        a_calibration: Option<String>,
        #[arg(long)]
        b_calibration: Option<String>,
    },

    /// Rate players by repeated double round-robin tournaments: in each,
    /// every player meets every other once with each colour. PLAYERS lists
    /// one player per line as NAME DEPTH SPEC (SPEC as for `arena`; lines
    /// starting with # are comments). After every tournament its games are
    /// appended to OUT/games.csv, and the rating list (ratings.md/.csv), the
    /// head-to-head table (head_to_head.md/.csv) and the ratings after each
    /// tournament (elo_history.csv) are rewritten. Running again with the
    /// same OUT continues where it stopped, or only rewrites the tables;
    /// players added to PLAYERS first play the earlier tournaments' games.
    Tournament {
        players: PathBuf,
        out: PathBuf,
        /// Tournaments to have played in all, counting those already in OUT.
        #[arg(long, default_value_t = 50)]
        tournaments: u64,
        #[arg(long, value_enum, default_value_t = Policy::LogOddsSoftmax)]
        policy: Policy,
        /// Softmax temperature of the policy.
        #[arg(long, default_value_t = 0.1)]
        temperature: f64,
        /// Predict from cached probabilities (faster) instead of raw counts.
        #[arg(long)]
        use_probs: bool,
        /// Random seed; keep it when continuing a run.
        #[arg(long, default_value_t = 1)]
        seed: u64,
        /// K-factor of the running Elo, per game.
        #[arg(long, default_value_t = 8.0)]
        k: f64,
        /// Virtual draws per pairing in the maximum-likelihood ratings, so
        /// perfect scores give finite ratings.
        #[arg(long, default_value_t = 1.0)]
        prior_draws: f64,
        /// The player rated 0 in the tables (default: the field's mean).
        #[arg(long)]
        anchor: Option<String>,
    },

    /// Train a model by self-play: each round plays games in parallel, writes
    /// them in the self_play.py format, and trains the model on them.
    SelfPlay {
        /// Directory for round{N}/ game files, results.txt and models.
        out: PathBuf,
        #[arg(long, value_enum, default_value_t = ModelKind::Cnbp)]
        model: ModelKind,
        #[arg(long, default_value_t = 10)]
        rounds: u64,
        #[arg(long, default_value_t = 1000)]
        games: usize,
        #[arg(long, value_enum, default_value_t = Policy::WldScoreJitter10)]
        policy: Policy,
        /// Softmax temperature for the wld-score-softmax policy.
        #[arg(long, default_value_t = 0.01)]
        temperature: f64,
        /// Random seed; chosen and printed if not given.
        #[arg(long)]
        seed: Option<u64>,
        /// Worker threads for playing; defaults to the number of cores.
        #[arg(long)]
        threads: Option<usize>,
        /// Instead of playing itself, version N plays version N-1 each round
        /// (round 0: the untrained model plays itself). Records version N's
        /// score in ladder.csv.
        #[arg(long)]
        vs_previous: bool,
        /// Don't write each round's games to disk (they take ~50 MB per
        /// 1000 games); models, results.txt and ladder.csv are still written.
        #[arg(long)]
        no_games: bool,
        /// Search depth for choosing moves (0: score the moves directly; 1 or
        /// more: alpha-beta search, needs a log-odds policy).
        #[arg(long, default_value_t = 0)]
        depth: u32,
        /// Label each position with the outcome a search this deep by the
        /// model expects from it, instead of the game's result: the result
        /// if the principal variation ends the game, else the probabilities
        /// the model predicts for its last position (at depth 0, for the
        /// position itself), calibrated on the round's results.
        #[arg(long)]
        label_depth: Option<u32>,
        /// With --label-depth: the weight of the game's result in the label,
        /// the rest being the search's.
        #[arg(long, default_value_t = 0.0)]
        label_result: f64,
        /// With --label-depth: search with this fixed model (`cnbp:PATH`,
        /// `bdt:PATH`, `bdd:PATH`) instead of the model being trained.
        #[arg(long)]
        label_model: Option<String>,
        /// After each round's training, divide the model's counts (CNBP) or
        /// AdaGrad sums (LR, MoE) by this, so older rounds' games count less
        /// (1: all rounds alike).
        #[arg(long, default_value_t = 1.0)]
        round_forget: f64,
        /// Continue the run in OUT from its last saved model, as if it had
        /// not stopped (CNBP and LR only; give the same options and seed).
        #[arg(long)]
        resume: bool,
        /// Start from this saved model instead of an untrained one: a CNBP
        /// file for cnbp, an LR file for lr and moe (as every expert), a file
        /// of the --leaf kind for bdt and bdd (as the root leaf).
        #[arg(long, conflicts_with = "resume")]
        init: Option<PathBuf>,
        #[command(flatten)]
        params: ModelArgs,
    },

    /// Train a model on existing round directories (from self-play, a ladder
    /// or an arena) instead of its own games. Round N's positions are
    /// shuffled and trained on, then the model is saved as {model}{N+1}.model.
    Train {
        /// Directory containing round0/, round1/, ...
        games: PathBuf,
        /// Directory for the saved models.
        out: PathBuf,
        #[arg(long, value_enum, default_value_t = ModelKind::Cnbp)]
        model: ModelKind,
        /// Number of rounds to train on, starting from round0.
        #[arg(long, default_value_t = 10)]
        rounds: u64,
        /// Seed for shuffling each round's positions.
        #[arg(long, default_value_t = 0)]
        seed: u64,
        #[command(flatten)]
        params: ModelArgs,
    },

    /// Play games with a model and report, over the positions reached, how
    /// often jitter10 and softmax at each temperature would play something
    /// other than the top-scored move, and the gap between the top two moves.
    PolicyStats {
        /// `cnbp:PATH`, `bdt:PATH` or `bdd:PATH`.
        model: String,
        #[arg(long, default_value_t = 200)]
        games: usize,
        /// Policy used to play the games that provide the positions.
        #[arg(long, value_enum, default_value_t = Policy::WldScoreJitter10)]
        policy: Policy,
        #[arg(long, default_value_t = 0.01)]
        temperature: f64,
        /// Temperatures to report.
        #[arg(long, value_delimiter = ',', default_value = "0.001,0.003,0.01,0.03,0.1")]
        temperatures: Vec<f64>,
        #[arg(long)]
        use_probs: bool,
        #[arg(long, default_value_t = 1)]
        seed: u64,
    },

    /// Compare a model's predicted probabilities with actual outcomes on
    /// labelled positions (a round directory), bucketed by prediction.
    Calibration {
        /// `cnbp:PATH`, `bdt:PATH` or `bdd:PATH`.
        model: String,
        /// Round directory with white_wins/black_wins/draws.games.
        games: PathBuf,
        #[arg(long)]
        use_probs: bool,
    },

    /// Score a model's predictions on labelled positions (a round directory):
    /// mean log-loss, mean absolute error, and log-loss of who won among
    /// decisive positions.
    EvalLoss {
        /// `cnbp:PATH`, `bdt:PATH` or `bdd:PATH`.
        model: String,
        games: PathBuf,
        #[arg(long)]
        use_probs: bool,
        /// Also fit a calibration of the model's probabilities to these
        /// positions (see `calibration`) and report its log-loss.
        #[arg(long)]
        fit_calibration: bool,
        /// Report the log-loss with this calibration.
        #[arg(long)]
        calibration: Option<String>,
        /// Score only the positions this delegation model (`bdt:PATH` or
        /// `bdd:PATH`) routes to leaf --leaf.
        #[arg(long, requires = "leaf")]
        routed_by: Option<String>,
        /// See --routed-by: the leaf's node index, as `leaves` names it.
        #[arg(long, requires = "routed_by")]
        leaf: Option<usize>,
    },

    /// Write each leaf of a delegation model (`bdt:PATH` or `bdd:PATH`) to
    /// OUT/leaf{N}.model, N being its node index, as a standalone CNBP or LR
    /// model (load it as `cnbp:` or `lr:`).
    Leaves {
        model: String,
        out: PathBuf,
        #[arg(long)]
        use_probs: bool,
    },

    /// Measure how well models rank the legal moves of a position. Positions
    /// are sampled from games played by SOURCE; every legal move is valued by
    /// playing PLAYOUTS games from it (SOURCE plays both sides), and each
    /// model ranks the moves by log-odds for the side to move.
    RankTest {
        /// Model playing the sampling games and the playouts (`cnbp:PATH`, ...).
        source: String,
        /// Models to test.
        models: Vec<String>,
        #[arg(long, default_value_t = 200)]
        positions: usize,
        #[arg(long, default_value_t = 50)]
        playouts: usize,
        #[arg(long, value_enum, default_value_t = Policy::WldScoreJitter10)]
        playout_policy: Policy,
        #[arg(long, default_value_t = 1)]
        seed: u64,
        #[arg(long)]
        use_probs: bool,
    },

    /// Replay games dumped by rs/bench/dump_python_games.py and compare legal
    /// moves, encodings and game results with the Python implementation.
    Xcheck { file: PathBuf },

    /// Play random games, encoding every legal move's resulting position the
    /// way the self-play scripts do, and report half-moves per second.
    BenchMovegen {
        #[arg(long, default_value_t = 300)]
        games: usize,
        #[arg(long, default_value_t = 1)]
        seed: u64,
    },

    /// Train a CNBP on a games directory, save it, and predict every training
    /// position. Same inputs and outputs as rs/bench/cnbp_bench.c.
    BenchCnbp {
        games_dir: PathBuf,
        model_out: PathBuf,
        predictions_out: PathBuf,
        #[arg(long, default_value_t = 1.0)]
        alpha: f64,
        #[arg(long)]
        use_probs: bool,
        /// Times to repeat training and prediction, for more stable timing.
        #[arg(long, default_value_t = 10)]
        reps: usize,
    },

    /// Train a BDT on a games directory, save it, and predict every training
    /// position. Same inputs and outputs as rs/bench/bdt_bench.c.
    BenchBdt {
        games_dir: PathBuf,
        model_out: PathBuf,
        predictions_out: PathBuf,
        #[arg(long, default_value_t = 2)]
        branch_factor: usize,
        #[arg(long, default_value_t = 2000)]
        split_threshold: u32,
        #[arg(long, default_value_t = 20)]
        split_limit: u32,
        #[arg(long, default_value_t = 10.0)]
        forget_factor: f64,
        #[arg(long, default_value_t = 1.0)]
        alpha: f64,
        #[arg(long)]
        use_probs: bool,
        #[arg(long, default_value_t = 3)]
        reps: usize,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum ModelKind {
    Cnbp,
    Bdt,
    /// Bayesian delegation DAG: a BDT whose delegators can split too. Saved
    /// in the BDT file format; load it as `bdd:PATH`.
    Bdd,
    /// Logistic regression over the same features (not naive Bayes); load
    /// it as `lr:PATH`.
    Lr,
    /// Mixture of logistic-regression experts with a logistic-regression
    /// gate; load it as `moe:PATH`.
    Moe,
}

#[derive(clap::Args, Clone, Debug)]
struct ModelArgs {
    /// Laplace smoothing.
    #[arg(long, default_value_t = 1.0)]
    alpha: f64,
    /// Predict from cached probabilities (faster) instead of raw counts.
    #[arg(long)]
    use_probs: bool,
    /// Logistic regression: AdaGrad's base step size.
    #[arg(long, default_value_t = 0.05)]
    lr_rate: f64,
    /// Mixture of experts: number of experts.
    #[arg(long, default_value_t = 2)]
    moe_experts: usize,
    /// Mixture of experts: the gate's initial weights are uniform in
    /// [-scale, scale], so the experts start on different positions.
    #[arg(long, default_value_t = 0.1)]
    moe_gate_scale: f64,
    /// Mixture of experts: seed of the gate's initial weights.
    #[arg(long, default_value_t = 1)]
    moe_seed: u64,
    /// Mixture of experts: L2 penalty on the gate's weights, which keeps it
    /// from switching sharply between experts.
    #[arg(long, default_value_t = 0.0)]
    moe_gate_l2: f64,
    /// A model started from a saved one (--init): divide its counts (CNBP)
    /// or AdaGrad sums (LR, every expert of a mixture) by this, so it adapts
    /// faster.
    #[arg(long, default_value_t = 1.0)]
    init_forget: f64,
    /// BDT and BDD: the leaves' classifier.
    #[arg(long, value_enum, default_value_t = NodeKind::Cnbp)]
    leaf: NodeKind,
    /// BDT and BDD: the delegators' routers' classifier.
    #[arg(long, value_enum, default_value_t = NodeKind::Cnbp)]
    router: NodeKind,
    /// BDT: children per split.
    #[arg(long, default_value_t = 2)]
    branch_factor: usize,
    /// BDT: examples a leaf sees between split checks.
    #[arg(long, default_value_t = 2000)]
    split_threshold: u32,
    /// BDT: maximum number of splits (BDD: of leaves and delegators).
    #[arg(long, default_value_t = 20)]
    split_limit: u32,
    /// BDT: divisor applied to a splitting leaf's counts.
    #[arg(long, default_value_t = 10.0)]
    forget_factor: f64,
    /// BDT: error measure for the split check and the routers' targets.
    #[arg(long, value_enum, default_value_t = ErrorMeasure::Abs)]
    error: ErrorMeasure,
    /// BDT: split only if a window's mean error rose by more than this many
    /// standard errors over the previous window's.
    #[arg(long, default_value_t = 0.0)]
    split_z: f64,
    /// BDT: when a leaf splits. (BDD delegators always use their own rule,
    /// see `--model bdd`.)
    #[arg(long, value_enum, default_value_t = SplitRule::Worse)]
    split_rule: SplitRule,
    /// BDT and BDD: what the routers learn to predict.
    #[arg(long, value_enum, default_value_t = RouteTarget::Error)]
    route: RouteTarget,
    /// BDT and BDD: what error routers learn from an example: the best
    /// child's index, or every child's share of the probability the children
    /// gave the outcome (soft: needs --error, and --trial-error with
    /// --route-switch, to be log or winner).
    #[arg(long, value_enum, default_value_t = ErrorTarget::Best)]
    error_target: ErrorTarget,
    /// BDT and BDD with CNBP leaves: every leaf also trains on the examples
    /// routed to the other leaves, weighted by this (0: only its own).
    #[arg(long, default_value_t = 0.0)]
    leaf_share: f64,
    /// BDT and BDD: test each split on the next --split-threshold examples
    /// through it against the node without the split, and undo the split
    /// unless its mean error is lower by this many standard errors.
    #[arg(long)]
    split_trial: Option<f64>,
    /// BDT and BDD: the error split trials compare, and error routers judge
    /// children by (winner: what log-odds play uses).
    #[arg(long, value_enum, default_value_t = ErrorMeasure::Winner)]
    trial_error: ErrorMeasure,
    /// BDT and BDD with cluster routing: train an error router beside each
    /// router, and switch a delegator to error routing when its picks beat
    /// the cluster router's by this many standard errors over a window of
    /// --split-threshold examples (inf: compare and log only).
    #[arg(long)]
    route_switch: Option<f64>,
}

impl ModelArgs {
    fn delegation_params(&self) -> DelegationParams {
        DelegationParams {
            class_num: 3,
            branch_factor: self.branch_factor,
            split_threshold: self.split_threshold,
            split_limit: self.split_limit,
            forget_factor: self.forget_factor,
            nb_alpha: self.alpha,
            use_probs: self.use_probs,
            leaf: self.leaf,
            router: self.router,
            lr_rate: self.lr_rate,
            error: self.error,
            split_z: self.split_z,
            split_rule: self.split_rule,
            route: self.route,
            error_target: self.error_target,
            leaf_share: self.leaf_share,
            split_trial: self.split_trial,
            trial_error: self.trial_error,
            route_switch: self.route_switch,
        }
    }
}

fn random_seed() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64
}

/// A player's model, loaded from a `cnbp:PATH`, `bdt:PATH`, `bdd:PATH` or `rand` spec.
enum LoadedModel {
    Cnbp(Cnbp),
    Lr(Logreg),
    Moe(Moe),
    Bdt(Bdt),
    Bdd(Bdd),
    HashMix(HashMix),
    Calibrated(Calibrated),
    Uniform(Uniform),
}

/// A model whose log-probabilities are recalibrated (see `Calibration`),
/// fitted on the positions of a round directory labelled by their games'
/// results. Loaded as `cal:GAMES:SPEC`, one calibration for the whole model,
/// or `leafcal:GAMES:SPEC`, one per leaf of a tree fitted on the positions
/// that leaf predicts, so that the leaves' log-odds mean the same; or
/// `blend:TAU:GAMES:SPEC`, which blends all the leaves' calibrated logits
/// with the weights of `Graph::leaf_weights` at TAU (each leaf calibrated on
/// all positions, weighted the same way), so the prediction changes smoothly
/// across the routers' boundaries.
struct Calibrated {
    model: Box<LoadedModel>,
    per_leaf: bool,
    blend: Option<f64>,
    /// Each leaf's calibration, by node index; `None` for leaves with too
    /// few decisive positions to fit one, which use `whole`.
    leaves: Vec<Option<Calibration>>,
    whole: Calibration,
}

impl Calibrated {
    /// Fewest decisive positions a leaf needs for its own calibration.
    const MIN_DECISIVE: f64 = 200.0;

    fn fit(model: LoadedModel, games: &Path, per_leaf: bool, blend: Option<f64>) -> Result<Self> {
        let data = games::read_labelled(games).with_context(|| format!("reading {games:?}"))?;
        let mut all = Vec::with_capacity(data.len());
        // Each leaf's (log-probabilities, class) pairs and their weights.
        let mut by_leaf = Vec::<(Vec<([f64; 3], usize)>, Vec<f64>)>::new();
        let mut weights = Vec::new();
        for (pos, label) in &data {
            let mut logs = [0.0; 3];
            model.classifier().predict_log(pos, &mut logs);
            let class = label.iter().position(|&x| x == 1.0).context("labels must be one-hot")?;
            match blend {
                Some(tau) => model.leaf_weights(pos, tau, &mut weights),
                None if per_leaf => {
                    weights.clear();
                    weights.push((model.leaf(pos), 1.0));
                }
                None => weights.clear(),
            }
            for &(leaf, weight) in &weights {
                if by_leaf.len() <= leaf {
                    by_leaf.resize(leaf + 1, Default::default());
                }
                let mut leaf_logs = [0.0; 3];
                model.leaf_predict_log(leaf, pos, &mut leaf_logs);
                by_leaf[leaf].0.push((leaf_logs, class));
                by_leaf[leaf].1.push(weight);
            }
            all.push((logs, class));
        }
        let whole = Calibration::fit(&all);
        println!("calibration on {}: {whole} ({} positions)", games.display(), all.len());
        let leaves = by_leaf
            .iter()
            .enumerate()
            .map(|(leaf, (d, w))| {
                if d.is_empty() {
                    return None;
                }
                let total: f64 = w.iter().sum();
                let decisive: f64 = d.iter().zip(w).filter(|((_, c), _)| *c < 2).map(|(_, w)| w).sum();
                let cal = (decisive >= Self::MIN_DECISIVE).then(|| Calibration::fit_weighted(d, w));
                let shown = cal.map_or("the whole model's".to_string(), |c| c.to_string());
                let share = 100.0 * decisive / total;
                println!("  leaf {leaf}: {total:.0} positions, {share:.1}% decisive, calibration {shown}");
                cal
            })
            .collect();
        Ok(Calibrated { model: Box::new(model), per_leaf, blend, leaves, whole })
    }

    /// A leaf's calibration, or the whole model's if the leaf has none.
    fn leaf_calibration(&self, leaf: usize) -> Calibration {
        self.leaves.get(leaf).copied().flatten().unwrap_or(self.whole)
    }

    /// The calibrated logits of the 3 classes.
    fn logits(&self, data: &[u8]) -> [f64; 3] {
        let Some(tau) = self.blend else {
            let mut logs = [0.0; 3];
            self.model.classifier().predict_log(data, &mut logs);
            return self.calibration(data).logits(&logs);
        };
        let mut weights = Vec::with_capacity(16);
        self.model.leaf_weights(data, tau, &mut weights);
        let total: f64 = weights.iter().map(|(_, w)| w).sum();
        let mut z = [0.0; 3];
        for &(leaf, weight) in &weights {
            let mut logs = [0.0; 3];
            self.model.leaf_predict_log(leaf, data, &mut logs);
            let leaf_z = self.leaf_calibration(leaf).logits(&logs);
            (0..3).for_each(|k| z[k] += weight / total * leaf_z[k]);
        }
        z
    }

    fn calibration(&self, data: &[u8]) -> Calibration {
        if !self.per_leaf {
            return self.whole;
        }
        self.leaf_calibration(self.model.leaf(data))
    }
}

impl Classifier for Calibrated {
    fn train(&mut self, _data: &[u8], _label: &[f64]) {
        unimplemented!("calibrated models are only for playing and scoring")
    }
    fn refresh(&mut self) {}
    fn predict(&self, data: &[u8], out: &mut [f64]) {
        let z = self.logits(data);
        let max = z.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let e = z.map(|v| (v - max).exp());
        let sum: f64 = e.iter().sum();
        out[..3].copy_from_slice(&e.map(|v| v / sum));
    }
    fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        out[..3].copy_from_slice(&self.logits(data));
    }
    fn save(&self, _path: &Path) -> std::io::Result<()> {
        Ok(())
    }
}

/// CNBPs that each predict the positions a fixed hash of the position
/// assigns to them: a random split of the work, as a baseline for the
/// delegation models. Loaded as `hashmix:PATH,PATH,...`.
struct HashMix(Vec<Cnbp>);

impl HashMix {
    /// The model that predicts `data`, by an FNV-1a hash of it.
    fn pick(&self, data: &[u8]) -> usize {
        let hash = data.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &b| (h ^ b as u64).wrapping_mul(0x0100_0000_01b3));
        (hash % self.0.len() as u64) as usize
    }
}

impl Classifier for HashMix {
    fn train(&mut self, _data: &[u8], _label: &[f64]) {
        unimplemented!("hashmix models are only for playing")
    }
    fn refresh(&mut self) {}
    fn predict(&self, data: &[u8], out: &mut [f64]) {
        self.0[self.pick(data)].predict(data, out)
    }
    fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        self.0[self.pick(data)].predict_log(data, out)
    }
    fn save(&self, _path: &Path) -> std::io::Result<()> {
        Ok(())
    }
}

impl LoadedModel {
    fn load(spec: &str, use_probs: bool) -> Result<Self> {
        if spec == "rand" {
            return Ok(LoadedModel::Uniform(Uniform));
        }
        let (kind, path) = spec.split_once(':').with_context(|| format!("expected KIND:PATH or rand, got {spec}"))?;
        let path = Path::new(path);
        Ok(match kind {
            "cnbp" => LoadedModel::Cnbp(Cnbp::load(path, use_probs).with_context(|| format!("loading {path:?}"))?),
            "lr" => LoadedModel::Lr(Logreg::load(path).with_context(|| format!("loading {path:?}"))?),
            "moe" => LoadedModel::Moe(Moe::load(path).with_context(|| format!("loading {path:?}"))?),
            "bdt" => LoadedModel::Bdt(Bdt::load(path, use_probs).with_context(|| format!("loading {path:?}"))?),
            "bdd" => LoadedModel::Bdd(Bdd::load(path, use_probs).with_context(|| format!("loading {path:?}"))?),
            "hashmix" => LoadedModel::HashMix(HashMix(
                spec[kind.len() + 1..]
                    .split(',')
                    .map(|p| Cnbp::load(Path::new(p), use_probs).with_context(|| format!("loading {p:?}")))
                    .collect::<Result<_>>()?,
            )),
            "cal" | "leafcal" => {
                let (games, inner) = spec[kind.len() + 1..]
                    .split_once(':')
                    .with_context(|| format!("expected {kind}:GAMES:KIND:PATH, got {spec}"))?;
                let inner = LoadedModel::load(inner, use_probs)?;
                LoadedModel::Calibrated(Calibrated::fit(inner, Path::new(games), kind == "leafcal", None)?)
            }
            "blend" => {
                let mut parts = spec[kind.len() + 1..].splitn(3, ':');
                let (Some(tau), Some(games), Some(inner)) = (parts.next(), parts.next(), parts.next()) else {
                    bail!("expected blend:TAU:GAMES:KIND:PATH, got {spec}");
                };
                let tau: f64 = tau.parse().with_context(|| format!("blend TAU {tau:?}"))?;
                let inner = LoadedModel::load(inner, use_probs)?;
                LoadedModel::Calibrated(Calibrated::fit(inner, Path::new(games), true, Some(tau))?)
            }
            _ => bail!("unknown model kind {kind:?}, expected cnbp, lr, moe, bdt, bdd, cal, leafcal or blend"),
        })
    }

    /// Index of the leaf that predicts the position: the tree's leaf, or 0
    /// for models that are not trees.
    fn leaf(&self, data: &[u8]) -> usize {
        match self {
            LoadedModel::Bdt(m) => m.leaf(data),
            LoadedModel::Bdd(m) => m.leaf(data),
            _ => 0,
        }
    }

    /// Each leaf's weight in a blend (`Graph::leaf_weights`, dropping
    /// paths below 0.001); the whole model, as leaf 0, if not a tree.
    fn leaf_weights(&self, data: &[u8], tau: f64, out: &mut Vec<(usize, f64)>) {
        match self {
            LoadedModel::Bdt(m) => m.leaf_weights(data, tau, 1e-3, out),
            LoadedModel::Bdd(m) => m.leaf_weights(data, tau, 1e-3, out),
            _ => {
                out.clear();
                out.push((0, 1.0));
            }
        }
    }

    /// The log-probabilities of leaf `leaf` (see `leaf`).
    fn leaf_predict_log(&self, leaf: usize, data: &[u8], out: &mut [f64]) {
        match self {
            LoadedModel::Bdt(m) => m.node_predict_log(leaf, data, out),
            LoadedModel::Bdd(m) => m.node_predict_log(leaf, data, out),
            _ => self.classifier().predict_log(data, out),
        }
    }

    fn classifier(&self) -> &(dyn Classifier + Sync) {
        match self {
            LoadedModel::Cnbp(m) => m,
            LoadedModel::Lr(m) => m,
            LoadedModel::Moe(m) => m,
            LoadedModel::Bdt(m) => m,
            LoadedModel::Bdd(m) => m,
            LoadedModel::HashMix(m) => m,
            LoadedModel::Calibrated(m) => m,
            LoadedModel::Uniform(m) => m,
        }
    }

    fn player(&self, policy: Policy, temperature: f64) -> Player<'_> {
        let model = self.classifier();
        // A random player chooses moves at random whatever the policy.
        let policy = if matches!(self, LoadedModel::Uniform(_)) { Policy::Rand } else { policy };
        Player::new(model, policy, temperature)
    }
}

fn arena(
    a: &str,
    b: &str,
    games: usize,
    (a_policy, b_policy): (Policy, Policy),
    temperature: f64,
    use_probs: bool,
    seed: u64,
    out: Option<PathBuf>,
    (a_depth, b_depth): (u32, u32),
    (a_value, b_value): (Value, Value),
    (a_calibration, b_calibration): (Option<String>, Option<String>),
) -> Result<()> {
    let (model_a, model_b) = (LoadedModel::load(a, use_probs)?, LoadedModel::load(b, use_probs)?);
    let (mut player_a, mut player_b) = (model_a.player(a_policy, temperature), model_b.player(b_policy, temperature));
    for (player, depth, value, calibration) in
        [(&mut player_a, a_depth, a_value, a_calibration), (&mut player_b, b_depth, b_value, b_calibration)]
    {
        player.depth = depth;
        player.value = value;
        if let Some(c) = calibration {
            player.calibration = Calibration::parse(&c)?;
        }
        check_depth(player)?;
    }
    let depth = |p: &Player| {
        let mut s = String::new();
        if p.depth > 0 {
            s += &format!(", depth {}", p.depth);
        }
        if p.value != Value::LogOdds {
            s += &format!(", value {:?}, calibration {}", p.value, p.calibration);
        }
        s
    };
    println!(
        "arena: A = {a} ({:?}{}), B = {b} ({:?}{}), {games} games, seed {seed}",
        player_a.policy,
        depth(&player_a),
        player_b.policy,
        depth(&player_b)
    );

    let start = Instant::now();
    let records = selfplay::play_match(player_a, player_b, games, seed);
    let secs = start.elapsed().as_secs_f64();
    if let Some(out) = out {
        selfplay::write_round(&out, &records)?;
    }
    if let Some(warning) = selfplay::duplicate_warning(&records) {
        println!("{warning}");
    }

    let m = selfplay::score_match(&records);
    println!("A: +{} -{} ={}, score {:.1}% ({secs:.1}s)", m.wins, m.losses, m.draws, 100.0 * m.score);
    println!("Elo difference A - B: {:+.0} (95% range {:+.0} to {:+.0})", m.elo, m.elo_low, m.elo_high);
    Ok(())
}

/// See `Command::Tournament`. Each distinct SPEC is loaded once.
fn tournament(
    players: &Path,
    out: &Path,
    policy: Policy,
    temperature: f64,
    use_probs: bool,
    anchor: Option<String>,
    mut opts: tournament::Options,
) -> Result<()> {
    let text = std::fs::read_to_string(players).with_context(|| format!("reading {players:?}"))?;
    let mut specs: Vec<&str> = Vec::new();
    let mut entries = Vec::new();
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        let [name, depth, spec] = fields[..] else { bail!("expected NAME DEPTH SPEC, got {line:?}") };
        let depth: u32 = depth.parse().with_context(|| format!("depth in {line:?}"))?;
        if entries.iter().any(|&(n, _, _)| n == name) {
            bail!("player {name} is listed twice");
        }
        let model = match specs.iter().position(|&s| s == spec) {
            Some(m) => m,
            None => {
                specs.push(spec);
                specs.len() - 1
            }
        };
        entries.push((name, depth, model));
    }
    if entries.len() < 2 {
        bail!("a tournament needs at least two players");
    }
    let models = specs.iter().map(|s| LoadedModel::load(s, use_probs)).collect::<Result<Vec<_>>>()?;
    let mut entrants = Vec::new();
    for &(name, depth, model) in &entries {
        let mut player = models[model].player(policy, temperature);
        player.depth = depth;
        check_depth(&player)?;
        entrants.push(tournament::Entrant { name: name.to_string(), player });
    }
    if let Some(anchor) = anchor {
        opts.anchor =
            Some(entries.iter().position(|&(n, _, _)| n == anchor).with_context(|| format!("no player {anchor}"))?);
    }
    std::fs::create_dir_all(out)?;
    std::fs::write(out.join("players.txt"), &text)?;
    let n = entrants.len();
    println!("tournament: {n} players, {} games per tournament, seed {}", n * (n - 1), opts.seed);
    tournament::run(&entrants, out, &opts)?;
    Ok(())
}

/// Search values moves in log-odds, so it needs a log-odds policy.
fn check_depth(player: &Player) -> Result<()> {
    if player.depth > 0 && !matches!(player.policy, Policy::LogOdds | Policy::LogOddsSoftmax | Policy::Rand) {
        bail!("search (depth > 0) needs the log-odds or log-odds-softmax policy, not {:?}", player.policy);
    }
    if player.depth == 0 && player.value != Value::LogOdds {
        bail!("the {:?} value is only used by search (depth > 0)", player.value);
    }
    Ok(())
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Arena {
            a,
            b,
            games,
            a_policy,
            b_policy,
            temperature,
            use_probs,
            seed,
            out,
            a_depth,
            b_depth,
            a_value,
            b_value,
            a_calibration,
            b_calibration,
        } => {
            arena(
                &a,
                &b,
                games,
                (a_policy, b_policy),
                temperature,
                use_probs,
                seed.unwrap_or_else(random_seed),
                out,
                (a_depth, b_depth),
                (a_value, b_value),
                (a_calibration, b_calibration),
            )
        }
        Command::Tournament {
            players,
            out,
            tournaments,
            policy,
            temperature,
            use_probs,
            seed,
            k,
            prior_draws,
            anchor,
        } => tournament(
            &players,
            &out,
            policy,
            temperature,
            use_probs,
            anchor,
            tournament::Options { tournaments, seed, k, prior_draws, anchor: None },
        ),
        Command::SelfPlay {
            out,
            model,
            rounds,
            games,
            policy,
            temperature,
            seed,
            threads,
            vs_previous,
            no_games,
            depth,
            label_depth,
            label_result,
            label_model,
            round_forget,
            resume,
            init,
            params,
        } => {
            if let Some(threads) = threads {
                rayon::ThreadPoolBuilder::new().num_threads(threads).build_global()?;
            }
            let seed = seed.unwrap_or_else(random_seed);
            check_depth(&Player { depth, ..Player::new(&Uniform, policy, temperature) })?;
            let run = SelfPlayRun {
                out,
                rounds,
                games,
                policy,
                temperature,
                seed,
                vs_previous,
                no_games,
                depth,
                label_depth,
                label_result,
                label_model: label_model.map(|spec| LoadedModel::load(&spec, params.use_probs)).transpose()?,
                round_forget,
            };
            let fresh = || Cnbp::new(3, &CATEGORIES, params.alpha, params.use_probs);
            let initial = || -> Result<Cnbp> {
                match &init {
                    Some(path) => {
                        let mut cnbp =
                            Cnbp::load(path, params.use_probs).with_context(|| format!("loading {path:?}"))?;
                        cnbp.forget(params.init_forget);
                        Ok(cnbp)
                    }
                    None => Ok(fresh()),
                }
            };
            // A tree's root leaf: --init's model, of the --leaf kind.
            let tree_root = || -> Result<NodeModel> {
                Ok(match (params.leaf, &init) {
                    (NodeKind::Cnbp, _) => NodeModel::Cnbp(initial()?),
                    (NodeKind::Lr, Some(path)) => {
                        let mut lr = Logreg::load(path).with_context(|| format!("loading {path:?}"))?;
                        lr.forget(params.init_forget);
                        NodeModel::Lr(lr)
                    }
                    (NodeKind::Lr, None) => NodeModel::Lr(Logreg::new(3, &CATEGORIES, params.lr_rate)),
                })
            };
            match model {
                ModelKind::Cnbp if resume => {
                    // Round N plays version N against N-1 and then saves
                    // version N+1, so the last saved version is the next round.
                    let saved = |n: u64| run.out.join(format!("cnbp{n}.model"));
                    let start = (1..).take_while(|&n| saved(n).exists()).last().context("no saved models to resume")?;
                    let load = |n: u64| if n == 0 { Ok(fresh()) } else { Cnbp::load(&saved(n), params.use_probs) };
                    run.run("cnbp", load(start)?, load(start - 1)?, start)
                }
                ModelKind::Cnbp => run.run("cnbp", initial()?, initial()?, 0),
                ModelKind::Lr if resume => {
                    let saved = |n: u64| run.out.join(format!("lr{n}.model"));
                    let start = (1..).take_while(|&n| saved(n).exists()).last().context("no saved models to resume")?;
                    let fresh = || Logreg::new(3, &CATEGORIES, params.lr_rate);
                    let load = |n: u64| if n == 0 { Ok(fresh()) } else { Logreg::load(&saved(n)) };
                    run.run("lr", load(start)?, load(start - 1)?, start)
                }
                ModelKind::Moe if resume => {
                    let saved = |n: u64| run.out.join(format!("moe{n}.model"));
                    let start = (1..).take_while(|&n| saved(n).exists()).last().context("no saved models to resume")?;
                    if start == 1 {
                        bail!("resume a mixture from round 2 on");
                    }
                    run.run("moe", Moe::load(&saved(start))?, Moe::load(&saved(start - 1))?, start)
                }
                _ if resume => bail!("--resume works only for cnbp, lr and moe, whose models hold all their state"),
                ModelKind::Bdt => {
                    let model = Bdt::starting_from(tree_root()?, &CATEGORIES, params.delegation_params());
                    run.run("bdt", model.clone(), model, 0)
                }
                ModelKind::Bdd => {
                    let model = Bdd::starting_from(tree_root()?, &CATEGORIES, params.delegation_params());
                    run.run("bdd", model.clone(), model, 0)
                }
                ModelKind::Moe => {
                    let model = match &init {
                        Some(path) => {
                            let expert = Logreg::load(path).with_context(|| format!("loading {path:?}"))?;
                            let mut moe = Moe::from_expert(
                                expert,
                                &CATEGORIES,
                                params.moe_experts,
                                params.lr_rate,
                                params.moe_gate_scale,
                                params.moe_seed,
                            );
                            moe.forget(params.init_forget);
                            moe.with_gate_l2(params.moe_gate_l2)
                        }
                        None => Moe::new(
                            &CATEGORIES,
                            params.moe_experts,
                            params.lr_rate,
                            params.moe_gate_scale,
                            params.moe_seed,
                        )
                        .with_gate_l2(params.moe_gate_l2),
                    };
                    run.run("moe", model.clone(), model, 0)
                }
                ModelKind::Lr => {
                    let model = match &init {
                        Some(path) => {
                            let mut lr = Logreg::load(path).with_context(|| format!("loading {path:?}"))?;
                            lr.forget(params.init_forget);
                            lr
                        }
                        None => Logreg::new(3, &CATEGORIES, params.lr_rate),
                    };
                    run.run("lr", model.clone(), model, 0)
                }
            }
        }
        Command::Train { games, out, model, rounds, seed, params } => match model {
            ModelKind::Cnbp => {
                train_rounds(&games, &out, rounds, seed, "cnbp", Cnbp::new(3, &CATEGORIES, params.alpha, params.use_probs))
            }
            ModelKind::Bdt => train_rounds(&games, &out, rounds, seed, "bdt", Bdt::new(&CATEGORIES, params.delegation_params())),
            ModelKind::Bdd => train_rounds(&games, &out, rounds, seed, "bdd", Bdd::new(&CATEGORIES, params.delegation_params())),
            ModelKind::Lr => train_rounds(&games, &out, rounds, seed, "lr", Logreg::new(3, &CATEGORIES, params.lr_rate)),
            ModelKind::Moe => train_rounds(
                &games,
                &out,
                rounds,
                seed,
                "moe",
                Moe::new(&CATEGORIES, params.moe_experts, params.lr_rate, params.moe_gate_scale, params.moe_seed)
                    .with_gate_l2(params.moe_gate_l2),
            ),
        },
        Command::PolicyStats { model, games, policy, temperature, temperatures, use_probs, seed } => {
            policy_stats(&model, games, policy, temperature, &temperatures, use_probs, seed)
        }
        Command::Calibration { model, games, use_probs } => calibration(&model, &games, use_probs),
        Command::Leaves { model, out, use_probs } => {
            let model = LoadedModel::load(&model, use_probs)?;
            let leaves: Vec<_> = match &model {
                LoadedModel::Bdt(m) => m.leaves().collect(),
                LoadedModel::Bdd(m) => m.leaves().collect(),
                _ => bail!("only bdt and bdd models have leaves"),
            };
            std::fs::create_dir_all(&out)?;
            for (idx, leaf) in leaves {
                let path = out.join(format!("leaf{idx}.model"));
                leaf.save(&path).with_context(|| format!("writing {path:?}"))?;
                println!("leaf {idx} ({:?}): {}", leaf.kind(), path.display());
            }
            Ok(())
        }
        Command::EvalLoss { model, games, use_probs, fit_calibration, calibration, routed_by, leaf } => {
            let routed = routed_by.as_deref().zip(leaf);
            eval_loss(&model, &games, use_probs, fit_calibration, calibration.as_deref(), routed)
        }
        Command::RankTest { source, models, positions, playouts, playout_policy, seed, use_probs } => {
            rank_test(&source, &models, positions, playouts, playout_policy, seed, use_probs)
        }
        Command::Xcheck { file } => xcheck(file),
        Command::BenchMovegen { games, seed } => bench_movegen(games, seed),
        Command::BenchCnbp { games_dir, model_out, predictions_out, alpha, use_probs, reps } => bench_classifier(
            "cnbp",
            || Cnbp::new(3, &CATEGORIES, alpha, use_probs),
            &games_dir,
            &model_out,
            &predictions_out,
            reps,
        ),
        Command::BenchBdt {
            games_dir,
            model_out,
            predictions_out,
            branch_factor,
            split_threshold,
            split_limit,
            forget_factor,
            alpha,
            use_probs,
            reps,
        } => {
            let params = DelegationParams {
                class_num: 3,
                branch_factor,
                split_threshold,
                split_limit,
                forget_factor,
                nb_alpha: alpha,
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
                trial_error: ErrorMeasure::Winner,
                route_switch: None,
            };
            bench_classifier(
                "bdt",
                || Bdt::new(&CATEGORIES, params.clone()),
                &games_dir,
                &model_out,
                &predictions_out,
                reps,
            )
        }
    }
}

fn train_rounds<M: Classifier>(games: &Path, out: &Path, rounds: u64, seed: u64, name: &str, mut model: M) -> Result<()> {
    use rand::seq::SliceRandom;
    std::fs::create_dir_all(out)?;
    for round in 0..rounds {
        let dir = games.join(format!("round{round}"));
        let mut data = games::read_labelled(&dir).with_context(|| format!("reading {}", dir.display()))?;
        data.shuffle(&mut StdRng::seed_from_u64(selfplay::derive_seed(seed, round, u64::MAX)));
        let start = Instant::now();
        for (pos, label) in &data {
            model.train(pos, label);
        }
        model.refresh();
        model.save(&out.join(format!("{name}{}.model", round + 1)))?;
        println!("round {round}: {} positions, train {:.2}s", data.len(), start.elapsed().as_secs_f64());
    }
    Ok(())
}

fn policy_stats(
    model: &str,
    games: usize,
    policy: Policy,
    temperature: f64,
    temperatures: &[f64],
    use_probs: bool,
    seed: u64,
) -> Result<()> {
    let model = LoadedModel::load(model, use_probs)?;
    let player = model.player(policy, temperature);
    let mut rng = StdRng::seed_from_u64(seed);
    let (mut positions, mut jitter_off) = (0usize, 0usize);
    let mut softmax_off = vec![0.0; temperatures.len()];
    let mut gaps = Vec::new();
    let mut draws = Vec::new();
    let mut underflows = 0usize;
    let mut scores = [0.0; 3];
    // Statistics are about log-odds for the log-odds policies, otherwise
    // about expected score.
    let log_odds = matches!(policy, Policy::LogOdds | Policy::LogOddsSoftmax);

    for _ in 0..games {
        let mut game = Game::default();
        while game.result().is_none() {
            let pos = game.position();
            let candidates = player::candidates(pos);
            let (win, loss) = if pos.turn().is_white() { (0, 1) } else { (1, 0) };
            let expected: Vec<f64> = candidates
                .iter()
                .map(|c| {
                    player.model.predict(&c.position, &mut scores);
                    draws.push(scores[2]);
                    if scores[win] == 0.0 || scores[loss] == 0.0 {
                        underflows += 1;
                    }
                    if log_odds {
                        player::log_odds(scores[win], scores[loss])
                    } else {
                        scores[win] + scores[2] / 2.0
                    }
                })
                .collect();

            if expected.len() > 1 {
                let mut sorted = expected.clone();
                sorted.sort_by(|a, b| b.total_cmp(a));
                let (best, second) = (sorted[0], sorted[1]);
                gaps.push(best - second);
                positions += 1;
                // Off-best probability under softmax: 1 - p(top move); moves
                // tied for the top count as the top move.
                for (off, &t) in softmax_off.iter_mut().zip(temperatures) {
                    let weights = expected.iter().map(|v| ((v - best) / t).exp());
                    let total: f64 = weights.clone().sum();
                    let top: f64 = weights.filter(|&w| w == 1.0).sum();
                    *off += 1.0 - top / total;
                }
                let jittered = expected.iter().map(|v| v * (1.0 + rng.random_range(-0.1..=0.1)));
                let pick = jittered.enumerate().max_by(|a, b| a.1.total_cmp(&b.1)).unwrap().0;
                if expected[pick] < best {
                    jitter_off += 1;
                }
            }

            let chosen = player::select(player, pos.turn(), &candidates, &mut rng);
            game.play(candidates[chosen].mv);
        }
    }

    gaps.sort_by(f64::total_cmp);
    let pct = |q: f64| gaps[((gaps.len() - 1) as f64 * q) as usize];
    println!("{positions} positions with 2+ moves, {games} games played with {policy:?}");
    println!(
        "gap between top two moves: median {:.1e}, 25% {:.1e}, 75% {:.1e}, 90% {:.1e}; exact ties {:.1}%",
        pct(0.5),
        pct(0.25),
        pct(0.75),
        pct(0.9),
        100.0 * gaps.iter().filter(|&&g| g == 0.0).count() as f64 / gaps.len() as f64
    );
    draws.sort_by(f64::total_cmp);
    let draw = |q: f64| draws[((draws.len() - 1) as f64 * q) as usize];
    println!(
        "predicted draw probability over all candidate positions: 10% {:.4}, median {:.6}, 90% {:.6}",
        draw(0.1),
        draw(0.5),
        draw(0.9)
    );
    println!(
        "candidate positions with P(win) or P(loss) exactly 0: {:.2}%",
        100.0 * underflows as f64 / draws.len() as f64
    );
    if log_odds {
        println!("(gaps and softmax figures below are in log-odds, nats)");
    }
    println!("jitter10: plays a non-top move {:.1}% of the time", 100.0 * jitter_off as f64 / positions as f64);
    for (off, t) in softmax_off.iter().zip(temperatures) {
        println!("softmax T={t}: plays a non-top move {:.1}% of the time", 100.0 * off / positions as f64);
    }
    Ok(())
}

fn calibration(model: &str, games: &Path, use_probs: bool) -> Result<()> {
    let model = LoadedModel::load(model, use_probs)?;
    let model = model.classifier();
    let data = games::read_labelled(games)?;
    let mut predictions = Vec::with_capacity(data.len());
    let mut out = [0.0; 3];
    for (pos, label) in &data {
        model.predict(pos, &mut out);
        let class = label.iter().position(|&l| l == 1.0).unwrap();
        predictions.push((out, class));
    }
    println!("{} positions from {}", data.len(), games.display());

    // Probability buckets: by decade of p below 0.5, by decade of 1 - p above.
    let bucket = |p: f64| -> i32 {
        let decade = |x: f64| (x.max(1e-300).log10().floor() as i32).max(-12);
        if p < 0.5 { decade(p) } else { -decade(1.0 - p) }
    };
    for (k, name) in ["white win", "black win", "draw"].iter().enumerate() {
        let mut buckets = std::collections::BTreeMap::<i32, (usize, f64, usize)>::new();
        for (p, class) in &predictions {
            let b = buckets.entry(bucket(p[k])).or_default();
            b.0 += 1;
            b.1 += p[k];
            b.2 += (*class == k) as usize;
        }
        println!("\nP({name}): predicted range | positions | mean predicted | actual");
        for (b, (n, sum, hits)) in buckets {
            let range = if b < 0 { format!("1e{b}..1e{}", b + 1) } else { format!("1-1e{}..1-1e{}", -b + 1, -b) };
            println!("  {range:>18} | {n:>8} | {:>12.3e} | {:.3e}", sum / n as f64, hits as f64 / n as f64);
        }
    }

    // Ratio calibration: predicted ln(P(white win) / P(black win)) against
    // the actual ln(white wins / black wins) in each bucket.
    let mut buckets = std::collections::BTreeMap::<i64, (usize, f64, usize, usize)>::new();
    for (p, class) in &predictions {
        let lo = chessai::player::log_odds(p[0], p[1]);
        let b = buckets.entry((lo / 2.0).floor() as i64).or_default();
        b.0 += 1;
        b.1 += lo;
        b.2 += (*class == 0) as usize;
        b.3 += (*class == 1) as usize;
    }
    println!("\nln(P(white win) / P(black win)): range | positions | mean predicted | actual ln(W/B) (W, B)");
    for (b, (n, sum, w, l)) in buckets {
        let actual = if w > 0 && l > 0 { format!("{:+.2}", (w as f64 / l as f64).ln()) } else { "-".into() };
        println!("  {:+4}..{:+4} | {n:>8} | {:+8.2} | {actual} ({w}, {l})", 2 * b, 2 * b + 2, sum / n as f64);
    }
    Ok(())
}

/// `routed`: a delegation model and one of its leaves; only the positions it
/// routes to that leaf are scored.
fn eval_loss(
    model: &str,
    games: &Path,
    use_probs: bool,
    fit: bool,
    calibration: Option<&str>,
    routed: Option<(&str, usize)>,
) -> Result<()> {
    let model = LoadedModel::load(model, use_probs)?;
    let model = model.classifier();
    let mut data = games::read_labelled(games)?;
    if let Some((spec, leaf)) = routed {
        let router = LoadedModel::load(spec, use_probs)?;
        data.retain(|(pos, _)| router.leaf(pos) == leaf);
    }
    let (mut log, mut abs, mut decisive, mut winner_log) = (0.0, 0.0, 0usize, 0.0);
    let mut out = [0.0; 3];
    for (pos, label) in &data {
        model.predict(pos, &mut out);
        log += delegation::log_loss(&out, label);
        abs += 1.0 - out.iter().zip(label).map(|(p, l)| p * l).sum::<f64>();
        if label[2] == 0.0 {
            let winner = if label[0] == 1.0 { 0 } else { 1 };
            let p = out[winner] / (out[0] + out[1]).max(f64::MIN_POSITIVE);
            winner_log += -p.max(f64::MIN_POSITIVE).ln();
            decisive += 1;
        }
    }
    let n = data.len() as f64;
    println!(
        "{} positions: log-loss {:.4}, abs error {:.4}; {decisive} decisive: winner log-loss {:.4}",
        data.len(),
        log / n,
        abs / n,
        winner_log / decisive.max(1) as f64
    );

    if fit || calibration.is_some() {
        let logs: Vec<([f64; 3], usize)> = data
            .iter()
            .map(|(pos, label)| {
                let mut l = [0.0; 3];
                model.predict_log(pos, &mut l);
                (l, label.iter().position(|&x| x == 1.0).unwrap())
            })
            .collect();
        let report = |name: &str, cal: Calibration| {
            let (mut decisive, mut winner_log) = (0usize, 0.0);
            for (l, class) in &logs {
                if *class < 2 {
                    let p = cal.probs(l);
                    winner_log += -(p[*class] / (p[0] + p[1])).max(f64::MIN_POSITIVE).ln();
                    decisive += 1;
                }
            }
            println!(
                "{name} calibration {cal}: log-loss {:.4}, winner log-loss {:.4}",
                cal.log_loss(&logs),
                winner_log / decisive.max(1) as f64
            );
        };
        if let Some(c) = calibration {
            report("given", Calibration::parse(c)?);
        }
        if fit {
            report("fitted", Calibration::fit(&logs));
        }
    }
    Ok(())
}

/// Plays `game` to the end with `player` on both sides; returns the result.
fn play_out(mut game: Game, player: Player, rng: &mut StdRng) -> GameResult {
    loop {
        if let Some(result) = game.result() {
            return result;
        }
        let pos = game.position();
        let candidates = player::candidates(pos);
        let chosen = player::select(player, pos.turn(), &candidates, rng);
        game.play(candidates[chosen].mv);
    }
}

fn rank_test(
    source: &str,
    models: &[String],
    positions: usize,
    playouts: usize,
    playout_policy: Policy,
    seed: u64,
    use_probs: bool,
) -> Result<()> {
    use rayon::prelude::*;
    let source = LoadedModel::load(source, use_probs)?;
    let player = source.player(playout_policy, 0.1);

    // Sample positions at a random ply of games played by the source.
    let mut rng = StdRng::seed_from_u64(seed);
    let mut sampled = Vec::new();
    while sampled.len() < positions {
        let target = rng.random_range(10..150);
        let mut game = Game::default();
        for _ in 0..target {
            if game.result().is_some() {
                break;
            }
            let pos = game.position();
            let candidates = player::candidates(pos);
            let chosen = player::select(player, pos.turn(), &candidates, &mut rng);
            game.play(candidates[chosen].mv);
        }
        if game.result().is_none() && game.position().legal_moves().len() > 1 {
            sampled.push(game);
        }
    }

    // Value every legal move by playouts, from the point of view of the side
    // to move: win 1, draw 1/2, loss 0. Returns (mean, variance) per move.
    let tasks: Vec<(usize, usize)> = sampled
        .iter()
        .enumerate()
        .flat_map(|(p, g)| (0..g.position().legal_moves().len()).map(move |m| (p, m)))
        .collect();
    let values: Vec<(f64, f64)> = tasks
        .par_iter()
        .map(|&(p, m)| {
            let game = &sampled[p];
            let mover = game.position().turn();
            let mv = game.position().legal_moves()[m];
            let mut rng = StdRng::seed_from_u64(selfplay::derive_seed(seed, p as u64, m as u64));
            let scores: Vec<f64> = (0..playouts)
                .map(|_| {
                    let mut g = game.clone();
                    g.play(mv);
                    match play_out(g, player, &mut rng) {
                        GameResult::Draw => 0.5,
                        GameResult::WhiteWin if mover.is_white() => 1.0,
                        GameResult::BlackWin if mover.is_black() => 1.0,
                        _ => 0.0,
                    }
                })
                .collect();
            let mean = scores.iter().sum::<f64>() / playouts as f64;
            let var = scores.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / playouts as f64;
            (mean, var)
        })
        .collect();

    let mut by_position = vec![Vec::new(); sampled.len()];
    for (&(p, _), &v) in tasks.iter().zip(&values) {
        by_position[p].push(v);
    }
    let decided_pairs: usize = by_position
        .iter()
        .map(|vs| {
            let mut n = 0;
            for i in 0..vs.len() {
                for j in i + 1..vs.len() {
                    if (vs[i].0 - vs[j].0).abs() > 2.0 * ((vs[i].1 + vs[j].1) / playouts as f64).sqrt() {
                        n += 1;
                    }
                }
            }
            n
        })
        .sum();
    println!(
        "{} positions, {} moves, {playouts} playouts each with {playout_policy:?}; {decided_pairs} move pairs whose playout values differ by > 2 standard errors",
        sampled.len(),
        tasks.len()
    );

    for name in models {
        let model = LoadedModel::load(name, use_probs)?;
        let model = model.classifier();
        let (mut agree, mut pairs, mut regret) = (0usize, 0usize, 0.0);
        let mut out = [0.0; 3];
        for (game, vs) in sampled.iter().zip(&by_position) {
            let pos = game.position();
            let (win, loss) = if pos.turn().is_white() { (0, 1) } else { (1, 0) };
            let scores: Vec<f64> = player::candidates(pos)
                .iter()
                .map(|c| {
                    model.predict(&c.position, &mut out);
                    player::log_odds(out[win], out[loss])
                })
                .collect();
            for i in 0..vs.len() {
                for j in i + 1..vs.len() {
                    if (vs[i].0 - vs[j].0).abs() > 2.0 * ((vs[i].1 + vs[j].1) / playouts as f64).sqrt() {
                        pairs += 1;
                        if (scores[i] - scores[j]) * (vs[i].0 - vs[j].0) > 0.0 {
                            agree += 1;
                        }
                    }
                }
            }
            let best = vs.iter().map(|v| v.0).fold(f64::NEG_INFINITY, f64::max);
            let top = scores.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).unwrap().0;
            regret += best - vs[top].0;
        }
        println!(
            "{name}: pairwise agreement {:.1}% of {pairs}, top-move regret {:.4}",
            100.0 * agree as f64 / pairs.max(1) as f64,
            regret / sampled.len() as f64
        );
    }
    Ok(())
}

fn sorted_uci(pos: &Chess) -> String {
    let mut moves: Vec<String> = pos
        .legal_moves()
        .iter()
        .map(|m| m.to_uci(CastlingMode::Standard).to_string())
        .collect();
    moves.sort();
    moves.join(" ")
}

#[derive(Default)]
struct Mismatches {
    count: usize,
    examples: Vec<String>,
}

impl Mismatches {
    fn add(&mut self, example: String) {
        self.count += 1;
        if self.examples.len() < 3 {
            self.examples.push(example);
        }
    }

    fn report(&self, name: &str) {
        println!("{name}: {}", self.count);
        for e in &self.examples {
            println!("    {e}");
        }
    }
}

fn xcheck(file: PathBuf) -> Result<()> {
    let text = std::fs::read_to_string(&file).with_context(|| format!("reading {file:?}"))?;

    let mut game = Game::default();
    let (mut positions, mut games_checked) = (0, 0);
    let mut legal_moves = Mismatches::default();
    let mut encodings = Mismatches::default();
    let mut castling_flags = Mismatches::default();
    let mut rust_ended_early = Mismatches::default();
    let mut results = Mismatches::default();
    let mut illegal_played = Mismatches::default();
    // After python plays a move rust considers illegal the positions no
    // longer correspond, so the rest of that game is skipped.
    let mut desynced = false;

    for (n, line) in text.lines().enumerate() {
        let fen = || shakmaty::fen::Fen::from_position(game.position(), shakmaty::EnPassantMode::Legal);

        if let Some(python_result) = line.strip_prefix("end ") {
            if desynced {
                game = Game::default();
                desynced = false;
                continue;
            }
            let rust_result = game.result().map_or("none", |r| r.pgn());
            if rust_result != python_result {
                results.add(format!("line {}: python {python_result}, rust {rust_result} at {}", n + 1, fen()));
            }
            game = Game::default();
            games_checked += 1;
            desynced = false;
            continue;
        }

        if desynced {
            continue;
        }

        let parts: Vec<&str> = line.split(" | ").collect();
        let [python_legal, python_enc, played] = parts[..] else {
            bail!("line {}: expected 3 fields separated by ' | '", n + 1);
        };
        positions += 1;

        let rust_legal = sorted_uci(game.position());
        if rust_legal != python_legal {
            legal_moves.add(format!(
                "line {}: {}\n      python: {python_legal}\n      rust:   {rust_legal}",
                n + 1,
                fen()
            ));
        }

        let python_enc = games::parse_position(python_enc)?;
        let rust_enc = encoding::encode(game.position());
        if rust_enc != python_enc {
            let differ: Vec<usize> = (0..rust_enc.len()).filter(|&i| rust_enc[i] != python_enc[i]).collect();
            let example = format!("line {}: features {differ:?} differ at {}", n + 1, fen());
            if differ.iter().all(|&i| (66..70).contains(&i)) {
                castling_flags.add(example);
            } else {
                encodings.add(example);
            }
        }

        if let Some(r) = game.result() {
            rust_ended_early.add(format!("line {}: rust says {} at {}", n + 1, r.pgn(), fen()));
        }

        let uci = played
            .parse::<UciMove>()
            .with_context(|| format!("line {}: bad move {played}", n + 1))?;
        match uci.to_move(game.position()) {
            Ok(m) => game.play(m),
            Err(_) => {
                illegal_played.add(format!("line {}: python played {played} at {}", n + 1, fen()));
                desynced = true;
            }
        }
    }

    println!("checked {positions} positions in {games_checked} games\n");
    illegal_played.report("python played a move rust considers illegal (rest of game skipped)");
    legal_moves.report("legal move set differs");
    encodings.report("encoding differs (other than castling flags)");
    castling_flags.report("castling flags differ (known: python keeps the flag after the rook is captured)");
    rust_ended_early.report("rust says game over, python continued");
    results.report("final result differs");

    if legal_moves.count + encodings.count + illegal_played.count > 0 {
        bail!("move generation or encoding mismatches found");
    }
    Ok(())
}

fn bench_movegen(game_num: usize, seed: u64) -> Result<()> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut plies = 0;
    let mut checksum = 0u64;
    let start = Instant::now();

    for _ in 0..game_num {
        let mut game = Game::default();
        while game.result().is_none() {
            let moves = game.position().legal_moves();
            for &m in &moves {
                let mut next = game.position().clone();
                next.play_unchecked(m);
                let enc: Encoded = encoding::encode(&next);
                checksum = checksum.wrapping_add(enc[0] as u64);
            }
            game.play(moves[rng.random_range(0..moves.len())]);
            plies += 1;
        }
    }

    let secs = start.elapsed().as_secs_f64();
    println!(
        "{game_num} games, {plies} plies in {secs:.2}s ({:.0} plies/s) [checksum {checksum}]",
        plies as f64 / secs
    );
    Ok(())
}

fn bench_classifier<C: Classifier>(
    name: &str,
    new_model: impl Fn() -> C,
    games_dir: &Path,
    model_out: &Path,
    predictions_out: &Path,
    reps: usize,
) -> Result<()> {
    let data = games::read_labelled(games_dir).with_context(|| format!("reading {games_dir:?}"))?;

    let mut model = new_model();
    let start = Instant::now();
    for _ in 0..reps {
        model = new_model();
        for (pos, label) in &data {
            model.train(pos, label);
        }
        model.refresh();
    }
    let train_secs = start.elapsed().as_secs_f64() / reps as f64;
    model.save(model_out)?;

    let mut out = vec![[0.0; 3]; data.len()];
    let start = Instant::now();
    for _ in 0..reps {
        for ((pos, _), o) in data.iter().zip(out.iter_mut()) {
            model.predict(pos, o);
        }
    }
    let predict_secs = start.elapsed().as_secs_f64() / reps as f64;

    let mut text = String::new();
    for o in &out {
        writeln!(text, "{:e} {:e} {:e}", o[0], o[1], o[2])?;
    }
    std::fs::write(predictions_out, text)?;

    let n = data.len() as f64;
    println!(
        "rust {name}: {} positions, train {:.2}M pos/s, predict {:.2}M pos/s",
        data.len(),
        n / train_secs / 1e6,
        n / predict_secs / 1e6
    );
    Ok(())
}

struct SelfPlayRun {
    out: PathBuf,
    rounds: u64,
    games: usize,
    policy: Policy,
    temperature: f64,
    seed: u64,
    vs_previous: bool,
    no_games: bool,
    depth: u32,
    label_depth: Option<u32>,
    label_result: f64,
    label_model: Option<LoadedModel>,
    round_forget: f64,
}

impl SelfPlayRun {
    /// Plays and trains rounds `start..rounds`; `previous` is the version
    /// before `model`, the opponent with `vs_previous`.
    fn run<M: Classifier + Sync + Clone>(&self, name: &str, mut model: M, mut previous: M, start: u64) -> Result<()> {
        std::fs::create_dir_all(&self.out)?;
        println!(
            "self-play: {name}, {} rounds of {} games, {}policy {:?}{}, seed {}, {} threads",
            self.rounds,
            self.games,
            if self.vs_previous { "version N vs N-1, " } else { "" },
            self.policy,
            if self.depth > 0 { format!(", search depth {}", self.depth) } else { String::new() }
                + &match self.label_depth {
                    Some(depth) => format!(
                        ", labels from depth-{depth} search by {} with result weight {}",
                        if self.label_model.is_some() { "a fixed model" } else { "the model" },
                        self.label_result
                    ),
                    None => String::new(),
                },
            self.seed,
            rayon::current_num_threads()
        );

        let ladder_path = self.out.join("ladder.csv");
        if self.vs_previous && start == 0 {
            std::fs::write(&ladder_path, "round,version,opponent,wins,losses,draws,score,elo,elo_low,elo_high\n")?;
        }
        if start > 0 {
            println!("resuming at round {start}");
        }

        for round in start..self.rounds {
            let start = Instant::now();
            let player = |model| Player { depth: self.depth, ..Player::new(model, self.policy, self.temperature) };
            let records = if self.vs_previous {
                let (current, opponent) = (player(&model), player(&previous));
                selfplay::play_match(current, opponent, self.games, selfplay::derive_seed(self.seed, round, 0))
            } else {
                selfplay::play_round(player(&model), self.games, self.seed, round)
            };
            let play_secs = start.elapsed().as_secs_f64();

            if self.vs_previous {
                let m = selfplay::score_match(&records);
                let opponent = round.saturating_sub(1);
                let mut ladder = std::fs::OpenOptions::new().append(true).open(&ladder_path)?;
                std::io::Write::write_all(
                    &mut ladder,
                    format!(
                        "{round},{round},{opponent},{},{},{},{:.4},{:.1},{:.1},{:.1}\n",
                        m.wins, m.losses, m.draws, m.score, m.elo, m.elo_low, m.elo_high
                    )
                    .as_bytes(),
                )?;
                println!(
                    "round {round}: v{round} vs v{opponent}: +{} -{} ={}, score {:.1}%, Elo {:+.0} ({:+.0} to {:+.0})",
                    m.wins,
                    m.losses,
                    m.draws,
                    100.0 * m.score,
                    m.elo,
                    m.elo_low,
                    m.elo_high
                );
                previous = model.clone();
            }

            if !self.no_games {
                selfplay::write_round(&self.out.join(format!("round{round}")), &records)?;
            }
            let count = |r: GameResult| records.iter().filter(|g| g.result == r).count();
            let (w, b, d) = (count(GameResult::WhiteWin), count(GameResult::BlackWin), count(GameResult::Draw));
            let mut results = std::fs::OpenOptions::new().create(true).append(true).open(self.out.join("results.txt"))?;
            std::io::Write::write_all(
                &mut results,
                format!("round {round} white wins: {w} black wins: {b} draws: {d}\n").as_bytes(),
            )?;

            let labels = if let Some(depth) = self.label_depth {
                let start = Instant::now();
                let labeller = self.label_model.as_ref().map_or(&model as &(dyn Classifier + Sync), |m| m.classifier());
                let calibration = selfplay::fit_calibration(&records, labeller);
                let labels = selfplay::search_labels(&records, labeller, calibration, depth, self.label_result);
                println!(
                    "round {round}: calibration {calibration}, labels {:.2}s",
                    start.elapsed().as_secs_f64()
                );
                labels
            } else {
                selfplay::result_labels(&records)
            };
            let start = Instant::now();
            let data = selfplay::training_data(&records, &labels, self.seed, round);
            for (pos, label) in &data {
                model.train(pos, label);
            }
            if self.round_forget != 1.0 {
                model.forget(self.round_forget);
            }
            model.refresh();
            let train_secs = start.elapsed().as_secs_f64();
            model.save(&self.out.join(format!("{name}{}.model", round + 1)))?;

            println!(
                "round {round}: +{w} -{b} ={d}, {} plies, play {play_secs:.2}s ({:.0} plies/s), train {train_secs:.2}s",
                data.len(),
                data.len() as f64 / play_secs
            );
            if let Some(warning) = selfplay::duplicate_warning(&records) {
                println!("  {warning}");
            }
        }
        Ok(())
    }
}
