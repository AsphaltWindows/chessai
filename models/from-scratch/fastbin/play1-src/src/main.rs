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
use chessai::calcache;
use chessai::calibration::Calibration;
use chessai::delegation::{
    self, DelegationParams, ErrorMeasure, ErrorTarget, NodeKind, NodeModel, ReviseOn, RouteTarget, SplitRule,
    SubrouterTarget,
};
use chessai::classifier::{Classifier, Uniform};
use chessai::cnbp::Cnbp;
use chessai::logreg::Logreg;
use chessai::moe::Moe;
use chessai::hybrid::{self, Expert, Hybrid};
use chessai::nbmix::{MoeNb, NbGateMoe, NbMix};
use chessai::tangate::TanGateMoe;
use chessai::probit::{Probit, ProbitParams};
use chessai::encoding::{self, CATEGORIES, Encoded};
use chessai::fm::Fm;
use chessai::ht::{Criterion, Ht, HtParams};
use chessai::game::Game;
use chessai::games;
use chessai::game::GameResult;
use chessai::player::{self, Player, Policy};
use chessai::search::Value;
use chessai::selfplay;
use chessai::tan::{self, Tan, TanOptions};
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
        /// 1 or more: alpha-beta search, needs a log-odds policy or an
        /// expected-score one: wld-score and its jittered versions).
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
    /// players added to PLAYERS first play the earlier tournaments' games
    /// (unless --no-catch-up). Every player with games in OUT/games.csv must
    /// be in PLAYERS or in --retired.
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
        /// perfect scores give finite ratings. 0 for pure maximum likelihood:
        /// players whose rating does not exist (a perfect or a zero score, or
        /// no results linking them both ways to the anchor) are listed as
        /// unrated.
        #[arg(long, default_value_t = 1.0)]
        prior_draws: f64,
        /// The player rated 0 in the tables (default: the field's mean).
        #[arg(long)]
        anchor: Option<String>,
        /// Don't play the earlier tournaments' games for the players added
        /// to PLAYERS since: each tournament is a round-robin of the players
        /// listed at the time.
        #[arg(long)]
        no_catch_up: bool,
        /// Retired players, listed as in PLAYERS: they play no more games,
        /// but their games in OUT/games.csv stay in the ratings, where they
        /// are marked as retired. Their models are not loaded.
        #[arg(long)]
        retired: Option<PathBuf>,
        /// Also write every game played, to train on: the game of tournament
        /// T (from 0, as in games.csv) between WHITE and BLACK (the players'
        /// names) goes to DIR/round{T}/{WHITE}~{BLACK}/, in the self-play
        /// round format (its positions in white_wins.games, black_wins.games
        /// or draws.games by result, its moves in human.games). `train DIR`
        /// trains on whole tournaments, one per round; for a chosen subset,
        /// make round directories of links to the chosen games' directories.
        /// Games already in OUT/games.csv are not replayed, so not saved.
        #[arg(long)]
        save_games: Option<PathBuf>,
        /// Rewrite the tables (ratings, head to head, Elo history) only after
        /// the last tournament of this run, not after each one; each
        /// tournament's line then times its games alone, and the last one
        /// also the tables ("tables in").
        #[arg(long)]
        reports_last: bool,
        /// With --prior-draws 0: fit the ratings and their errors by block
        /// elimination over the tournaments' fields instead of with dense
        /// matrices over all players: the same ratings and errors (to
        /// rounding), much faster in a big league.
        #[arg(long)]
        sparse_fit: bool,
        /// Also append each game's CPU time, measured on the thread that
        /// played it, to OUT/game_cpu.csv (tournament,white,black,cpu_s).
        /// The games are the same with or without it.
        #[arg(long)]
        game_cpu: bool,
        /// Open every game with this many plies of uniformly random legal
        /// moves, drawn from the game's random numbers (with
        /// --opening-temperature: the mover's log-odds softmax at it), then
        /// the policy. The opening plies are the game's like any other: in
        /// games.csv's plies and in the saved games, positions included. 0:
        /// no opening (the games as without it).
        #[arg(long, default_value_t = 0)]
        opening_plies: usize,
        /// With --opening-plies: the opening's moves sampled from the mover's
        /// log-odds softmax at this temperature (its model, depth and
        /// calibration), instead of uniformly at random.
        #[arg(long)]
        opening_temperature: Option<f64>,
    },

    /// Fits the pure maximum-likelihood ratings of a games.csv (as
    /// --prior-draws 0) both densely and by block elimination (as
    /// --sparse-fit), and prints the largest differences between their
    /// ratings and errors, and their times: a check of --sparse-fit.
    FitCheck {
        /// A tournament's games.csv.
        games: PathBuf,
        /// The player rated 0 (default: the rated players' mean).
        #[arg(long)]
        anchor: Option<String>,
        /// Also refit the ratings after each of the last N tournaments both
        /// ways, as elo_history.csv does (each from the one before).
        #[arg(long, default_value_t = 0)]
        prefixes: usize,
        /// Only time the sparse fit.
        #[arg(long)]
        sparse_only: bool,
        /// Write both fits' ratings and errors in full to this CSV.
        #[arg(long)]
        dump: Option<PathBuf>,
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
        /// more: alpha-beta search, needs a log-odds policy or an
        /// expected-score one: wld-score and its jittered versions, which
        /// score each move by the expected score at the end of its principal
        /// variation).
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
        /// After round N's training, also divide the counts or sums by
        /// ((N+2)/(N+1))^this, so the games of round s weigh ((s+1)/(N+1))^this
        /// at round N: rounds fade with the run's age, with no time scale to
        /// choose. A start model's games count as the round before the first
        /// (0: off).
        #[arg(long, default_value_t = 0.0)]
        age_forget: f64,
        /// Continue the run in OUT from its last saved model, as if it had
        /// not stopped (give the same options and seed; --rounds counts from
        /// the run's start). Trees resume from the state files of their last
        /// two versions (see --keep-states); a mixture from round 2 on.
        #[arg(long)]
        resume: bool,
        /// Trees also save their whole training state beside each model, as
        /// {model}N.state, for --resume (see `state`); keep the last this
        /// many (0: all). --resume needs the last 2.
        #[arg(long, default_value_t = 2)]
        keep_states: u64,
        /// Start from this saved model instead of an untrained one: a CNBP
        /// file for cnbp, a probit file for probit (its variances multiplied
        /// by --init-forget), an LR file for lr and moe (as every expert), a file
        /// of the --leaf kind for bdt and bdd (as the root leaf).
        #[arg(long, conflicts_with = "resume")]
        init: Option<PathBuf>,
        #[command(flatten)]
        params: ModelArgs,
    },

    /// Train a model on existing round directories (from self-play, a ladder,
    /// an arena or `tournament --save-games`) instead of its own games. Round
    /// N's positions are shuffled and trained on, then the model is saved as
    /// {model}{N+1}.model (a tree's last also as {model}{N+1}.state, for
    /// --start).
    Train {
        /// Directory containing round0/, round1/, ..., each holding positions
        /// in the round format or game directories (`tournament
        /// --save-games`), whose positions are trained on together.
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
        /// Train this saved model further instead of an untrained one: a
        /// model file of the --model kind for cnbp, lr, probit, fm and moe (forgotten
        /// by --init-forget), the state file saved beside a tree's model for
        /// bdt, bdd and ht ({model}N.state, by self-play or train; give the
        /// options it was trained with), for tan its state file or a model
        /// file with its state file beside it (forgotten by --init-forget). --rounds still counts GAMES' rounds
        /// from round0, and the model after round N is still saved as
        /// {model}{N+1}.model, whatever the start model's number.
        #[arg(long)]
        start: Option<PathBuf>,
        /// Label each position with the outcome a search this deep by the
        /// model, as it starts the round, expects from it (as self-play's
        /// --label-depth), calibrated on the round's results, instead of the
        /// game's result. lr only; the round must be game directories with
        /// their moves (human.games), as `tournament --save-games` writes.
        #[arg(long)]
        label_depth: Option<u32>,
        /// With --label-depth: the weight of the game's result in the label,
        /// the rest being the search's.
        #[arg(long, default_value_t = 0.0)]
        label_result: f64,
        /// With --label-depth: search with this fixed model (a spec, as
        /// self-play's --label-model) instead of the model being trained.
        #[arg(long)]
        label_model: Option<String>,
        #[command(flatten)]
        params: ModelArgs,
    },

    /// Replay chosen games of a tournament exactly. PLAYERS is the
    /// tournament's players in their order (NAME DEPTH SPEC, as for
    /// `tournament`); GAMES lists one game per line as T WHITE BLACK. Each is
    /// played with the tournament's seed for it, derive_seed(SEED, T, g), g
    /// its pairing's index, saved to OUT/round{T}/{WHITE}~{BLACK}/, and
    /// printed as T,WHITE,BLACK,RESULT,PLIES (as in games.csv). To replay a
    /// tournament's games, give its policy, temperature and opening.
    ReplayGames {
        players: PathBuf,
        games: PathBuf,
        out: PathBuf,
        #[arg(long, value_enum, default_value_t = Policy::LogOddsSoftmax)]
        policy: Policy,
        #[arg(long, default_value_t = 0.1)]
        temperature: f64,
        #[arg(long)]
        use_probs: bool,
        #[arg(long, default_value_t = 1)]
        seed: u64,
        /// Open every game with this many plies of uniformly random legal
        /// moves, drawn from the game's random numbers (with
        /// --opening-temperature: the mover's log-odds softmax at it), then
        /// the policy. The opening plies are the game's like any other: in
        /// games.csv's plies and in the saved games, positions included. 0:
        /// no opening (the games as without it).
        #[arg(long, default_value_t = 0)]
        opening_plies: usize,
        /// With --opening-plies: the opening's moves sampled from the mover's
        /// log-odds softmax at this temperature (its model, depth and
        /// calibration), instead of uniformly at random.
        #[arg(long)]
        opening_temperature: Option<f64>,
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

    /// Diagnose a probit's self-play move choice: play games with its
    /// self-play policy (jitter10 on the expected score, depth 0) and score
    /// every position's candidate moves by variants of its prediction: as
    /// it is, from the weights' means alone (no predictive variance), with
    /// the draw margin calibrated to the games' share of positions in drawn
    /// games, and with other draw margins.
    ProbitDiag {
        /// A probit model file.
        model: PathBuf,
        #[arg(long, default_value_t = 100)]
        games: usize,
        #[arg(long, default_value_t = 3)]
        seed: u64,
        /// More draw margins to score with.
        #[arg(long, value_delimiter = ',', default_value = "0,0.5,1")]
        eps: Vec<f64>,
    },

    /// Naive-Bayes mixtures: how many components a model uses on a round
    /// directory's positions. For the components' weights given the
    /// position (the gate, or p(component | position)) and given the
    /// position and its label (the training responsibilities): the
    /// effective number of components (exp of the entropy of their mean
    /// weights), the mean weights, exp of the weights' mean entropy per
    /// position (1: hard), and the share of positions whose largest weight
    /// is over 0.99.
    MixStats {
        /// `moe:PATH`, `moenb:PATH`, `nbmix:PATH` or `nbgate:PATH`.
        model: String,
        /// A round directory of labelled positions.
        games: PathBuf,
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
        /// Also write every scored position to this file, one line each in
        /// the order scored: its class (0 white win, 1 black win, 2 draw),
        /// the model's three log-probabilities (`predict_log`, what the
        /// calibration reads) and the 70 encoded features.
        #[arg(long)]
        dump: Option<PathBuf>,
    },

    /// Write each leaf of a tree (`bdt:PATH`, `bdd:PATH` or `ht:PATH`) to
    /// OUT/leaf{N}.model, N being its node index, as a standalone CNBP or LR
    /// model (load it as `cnbp:` or `lr:`).
    Leaves {
        model: String,
        out: PathBuf,
        #[arg(long)]
        use_probs: bool,
    },

    /// How well a tree's routers route (`bdt:PATH`), on the decisive positions
    /// of labelled games (a round directory). For each delegator and each
    /// child it sends positions to: the winner log-loss of every child's
    /// subtree on those positions, with the tree's log-odds recalibrated by
    /// one scale and offset fitted here. A row whose smallest loss is not its
    /// own child's sends positions to a child that predicts them worse than a
    /// sibling; the regret is what sending each row to its best child would
    /// save. Then the pairs of leaves whose log-odds differ least over all the
    /// positions (near-duplicates).
    Routing {
        model: String,
        games: PathBuf,
        #[arg(long)]
        use_probs: bool,
    },

    /// Path context diagnostics of a BDD (`bdd:PATH`, trained with --path-context) on the labelled positions of
    /// round directories: its sibling sets, each node's cells, non-home arrivals at the leaves, and how much the
    /// path tables move the leaves' log-odds and the routers' choices.
    PathStats {
        model: String,
        games: Vec<PathBuf>,
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

    /// play1: serve the dashboard's play page. Reads one request per stdin line, tab-separated KEY=VALUE fields, and
    /// answers each with one stdout line "@@play " and a JSON object (other stdout lines are what loading a model
    /// prints, e.g. its calibration). op=move: SPEC (as for `tournament`), DEPTH (the ply, at least 1), POLICY
    /// (argmax: the log-odds argmax; softmax: the log-odds softmax at TEMPERATURE, default 0.02), SEED, MOVES (UCI,
    /// space-separated, from the start position), VALUES (1: every legal move's search value too): the move the
    /// player chooses there exactly as a tournament player does (selfplay::choose_in; ties and samples drawn from
    /// derive_seed(SEED, plies played, 0)). op=eval: SPEC and MOVES: the model's probabilities of the position.
    /// op=legal: MOVES: the legal moves (UCI), the FEN, the game's end by this engine's rules and the moves' SAN.
    /// op=load: SPEC.
    PlayServe {
        /// Predict from cached probabilities, as the leagues' tournaments do.
        #[arg(long)]
        use_probs: bool,
        /// Models kept loaded; the least recently used goes first.
        #[arg(long, default_value_t = 4)]
        keep: usize,
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
    /// Factorization machine: logistic regression plus learned pairwise
    /// interactions of the features; load it as `fm:PATH`.
    Fm,
    /// Hoeffding tree: one feature-value test per internal node, CNBP or LR
    /// leaves (--leaf), split when the Hoeffding bound allows; load it as
    /// `ht:PATH`.
    Ht,
    /// Tree-augmented naive Bayes: CNBP whose features each also condition
    /// on a parent feature, the Chow-Liu tree of their pair counts (see
    /// `tan`); load it as `tan:PATH`. It trains on from its state file.
    Tan,
    /// Online Bayesian ordinal probit regression over the logistic
    /// regression's features, trained by assumed-density filtering (see
    /// `probit`); load it as `probit:PATH`. Its file holds all its training
    /// state, and its options (--probit-*) when it was made.
    Probit,
    /// Mixture of naive-Bayes (CNBP) experts with a logistic-regression gate
    /// (--moe-experts, --moe-gate-scale, --moe-seed, --moe-gate-l2,
    /// --lr-rate for the gate, --alpha, --nbmix-refresh; see `nbmix`); load
    /// it as `moenb:PATH`. Its file holds all its training state.
    Moenb,
    /// Generative mixture of naive Bayes models (latent class), trained by
    /// online EM (--nbmix-components, --nbmix-warm, --nbmix-seed,
    /// --nbmix-refresh, --alpha; see `nbmix`); load it as `nbmix:PATH`. Its
    /// file holds all its training state.
    Nbmix,
    /// Mixture of logistic-regression experts with a naive-Bayes (CNBP)
    /// gate that counts the responsibilities (--moe-experts, --lr-rate for
    /// the experts, --alpha for the gate, --nbmix-warm, --nbmix-seed,
    /// --nbmix-refresh; see `nbmix`); load it as `nbgate:PATH`. Its file
    /// holds all its training state.
    Nbgate,
    /// As nbgate with a tree-augmented naive-Bayes gate of fixed m
    /// (--tan-m) in place of the CNBP (--moe-experts, --lr-rate, --alpha,
    /// --nbmix-warm, --nbmix-seed; see `tangate`); load it as
    /// `tangate:PATH`. Its model file holds the gate's tables; it resumes
    /// from its state files.
    Tangate,
    /// Mixture of experts of different kinds (--hybrid-experts: lr,
    /// probit, cnbp), each trained on every position exactly as it would be
    /// alone, with a logistic-regression gate over the position that learns
    /// their responsibilities (--hybrid-gate-rate); it predicts with their
    /// mixture, or the gate's choice alone (--hybrid-hard), or one expert
    /// alone (--hybrid-pin, for identity checks). See `hybrid`; load it as
    /// `hybrid:PATH`. Its file holds all its training state, and its options
    /// when it was made.
    Hybrid,
}

/// A hybrid's gate target (see --hybrid-gate-target).
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum HybridGateTarget {
    Full,
    Win,
}

/// A hybrid's expert kind (see --hybrid-experts).
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum HybridExpert {
    Lr,
    Probit,
    Cnbp,
}

#[derive(clap::Args, Clone, Debug)]
struct ModelArgs {
    /// Laplace smoothing.
    #[arg(long, default_value_t = 1.0)]
    alpha: f64,
    /// TAN: weight m of naive Bayes's estimate in each feature's
    /// m-estimate given its parent's value (in games with --tan-games).
    #[arg(long, default_value_t = tan::DEFAULT_M)]
    tan_m: f64,
    /// TAN: m = λ N instead of --tan-m, N the positions counted so far, so
    /// that a parent value's own evidence keeps its weight as the counts
    /// grow.
    #[arg(long)]
    tan_lambda: Option<f64>,
    /// TAN: count the pairs in games (each game once) and keep only the
    /// significant edges (an experiment; see `tan`).
    #[arg(long)]
    tan_games: bool,
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
    /// Mixture of experts (moe): predict with the expert the gate weighs
    /// most, alone (ties to the first), instead of the experts' mixture.
    /// Training is unchanged (soft responsibilities, which the gate learns).
    /// Kept in the model file (only when set).
    #[arg(long)]
    moe_hard: bool,
    /// Naive-Bayes mixture (nbmix): number of components.
    #[arg(long, default_value_t = 4)]
    nbmix_components: usize,
    /// Naive-Bayes mixture (nbmix): the first this many examples each go to
    /// a random component (symmetry breaking), before EM.
    #[arg(long, default_value_t = 10000)]
    nbmix_warm: u64,
    /// Naive-Bayes mixture (nbmix): seed of the warm-up's components.
    #[arg(long, default_value_t = 1)]
    nbmix_seed: u64,
    /// Naive-Bayes mixtures (nbmix, moenb): examples between refreshes of
    /// the components' cached probabilities, which score the examples.
    #[arg(long, default_value_t = 1024)]
    nbmix_refresh: u64,
    /// Factorization machine: length of each feature value's interaction
    /// vector (per class).
    #[arg(long, default_value_t = 8)]
    fm_factors: usize,
    /// Factorization machine: the vectors start uniform in [-scale, scale].
    #[arg(long, default_value_t = 0.01)]
    fm_scale: f64,
    /// Factorization machine: seed of the vectors' starting values.
    #[arg(long, default_value_t = 1)]
    fm_seed: u64,
    /// Probit: the prior standard deviation of every weight. A model's
    /// --probit-* options are kept in its file: a model trained on from its
    /// file (--resume, --start, --init) keeps them.
    #[arg(long, default_value_t = ProbitParams::default().prior)]
    probit_prior: f64,
    /// Probit: the standard deviation of the latent's noise.
    #[arg(long, default_value_t = ProbitParams::default().beta)]
    probit_beta: f64,
    /// Probit: dynamics, the standard deviation added to each weight a
    /// position uses before learning from it.
    #[arg(long, default_value_t = ProbitParams::default().tau)]
    probit_tau: f64,
    /// Probit: the power of each position's likelihood (a game's positions
    /// share its result).
    #[arg(long, default_value_t = ProbitParams::default().alpha)]
    probit_alpha: f64,
    /// Probit: instead of --probit-alpha, the power is this divided by the
    /// number of positions of the position's game, so that each game counts
    /// as this many observations (train needs rounds of game directories).
    #[arg(long)]
    probit_per_game: Option<f64>,
    /// Probit: the draw margin of a new model.
    #[arg(long, default_value_t = ProbitParams::default().eps)]
    probit_eps: f64,
    /// Probit: the draw margin's step size (0: fixed).
    #[arg(long, default_value_t = ProbitParams::default().eps_rate)]
    probit_eps_rate: f64,
    /// Probit, self-play: the players choose their moves with this draw
    /// margin instead of the learned one (0: by the probability that the
    /// mover's latent advantage is positive, with no draws), so that the
    /// expected scores of the moves are not squeezed together by a learned
    /// draw probability near 1. Training and the saved models are
    /// unchanged, so other play (arena, tournament) is too. Train accepts
    /// and ignores it (the cycle's own step passes the recipe's options).
    #[arg(long)]
    probit_play_eps: Option<f64>,
    /// Hybrid: its experts' kinds, in order, 2 to 4 of lr (with --lr-rate),
    /// probit (with the --probit-* options) and cnbp (with --alpha and
    /// --use-probs). Kept in the model file.
    #[arg(long, value_enum, value_delimiter = ',', default_value = "lr,probit")]
    hybrid_experts: Vec<HybridExpert>,
    /// Hybrid: the gate's AdaGrad base step size. Kept in the model file.
    #[arg(long, default_value_t = 0.05)]
    hybrid_gate_rate: f64,
    /// Hybrid: predict with the expert the gate weighs most, alone, instead
    /// of the experts' mixture. Kept in the model file.
    #[arg(long)]
    hybrid_hard: bool,
    /// Hybrid: predict with this expert alone (0 the first), whatever the
    /// gate says, which still learns: the hybrid then plays and trains its
    /// experts as that expert alone would (an identity check). Kept in the
    /// model file.
    #[arg(long)]
    hybrid_pin: Option<usize>,
    /// Hybrid: what the gate learns from: full (each expert's share of the
    /// mixture's probability of the result, draws included) or win (decisive
    /// results only, from each expert's probability of the winner given a
    /// decisive result: what log-odds play uses). Kept in the model file
    /// (only when win).
    #[arg(long, value_enum, default_value = "full")]
    hybrid_gate_target: HybridGateTarget,
    /// A model started from a saved one (--init): divide its counts (CNBP)
    /// or AdaGrad sums (LR, every expert of a mixture) by this, so it adapts
    /// faster.
    #[arg(long, default_value_t = 1.0)]
    init_forget: f64,
    /// BDT, BDD and Hoeffding tree: the leaves' classifier (probit: BDT and
    /// BDD, with the --probit-* options but --probit-per-game).
    #[arg(long, value_enum, default_value_t = NodeKind::Cnbp)]
    leaf: NodeKind,
    /// BDT and BDD: the delegators' routers' classifier (not probit).
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
    /// BDD with error routing: what a delegator over the sub-routers of a
    /// delegator split learns. routing: which sub-router routes best (their
    /// router errors). home: its old router's target, which shared child
    /// predicts best; class i sends the position to sub-router i (bddhome).
    /// Training only: model files are unchanged.
    #[arg(long, value_enum, default_value_t = SubrouterTarget::Routing)]
    subrouter_target: SubrouterTarget,
    /// BDD: path context. Every node also learns from which path delivered the position: one input per sibling set
    /// (the copies of one router split) above it, whose value says which copy the path passed through, or none
    /// (see `pathctx`). Logistic-regression and CNBP nodes, error routing, no split trials. Saved in model files,
    /// which binaries without it refuse.
    #[arg(long)]
    path_context: bool,
    /// BDD (errdag): the router-split threshold. A delegator splits only if
    /// its window's mean router error rose by more than this many standard
    /// errors over its previous window's (and is at least the mean of all
    /// routers' errors over the window), in place of the stall rule by
    /// --split-z. With --split-z 0 (the default), 0 is exactly that rule; a
    /// larger value splits routers less often; inf never splits a router, so
    /// the BDD makes only leaf splits. Leaf splits keep --split-z. Whatever
    /// the leaves. Not saved in model files.
    #[arg(long)]
    router_split_z: Option<f64>,
    /// BDD (errdag2): a separate budget for router (delegator) splits. With
    /// it, --split-limit counts leaf splits only, router splits stop at this
    /// many, and a router may split only while fewer router splits than leaf
    /// splits have been made: the DAG is the tree plus at most this many
    /// router splits. Without it, every split counts towards --split-limit.
    /// Refuses split trials and --leaf-share. Not saved in model files.
    #[arg(long)]
    router_split_limit: Option<u32>,
    /// BDT and BDD with CNBP or LR leaves: every leaf also trains on the
    /// examples routed to the other leaves, weighted by this (0: only its
    /// own): CNBP counts them so, LR takes a gradient step on this times
    /// their log-loss.
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
    /// BDT and BDD with --route-switch: the error routers' classifier
    /// (default: --router's).
    #[arg(long, value_enum)]
    error_router: Option<NodeKind>,
    /// BDT: revisable structure. Every delegator keeps a shadow, the leaf it
    /// was before it split, trained since on every example through it. After
    /// every --split-threshold examples through the delegator, its branch is
    /// merged back into the shadow (as a leaf, which may split again) if the
    /// branch's mean gain over the shadow by --trial-error, less
    /// --seam-weight times its seams, is below minus this many standard
    /// errors (inf: compare and log only). Seams are measured on self-play's
    /// games.
    #[arg(long)]
    revise: Option<f64>,
    /// BDT with --revise: the weight of a branch's seams per position through
    /// its delegator, a seam being the difference in white's log-odds (nats)
    /// of a position between the leaf it is in and the leaf of the position
    /// before it, for moves between two of the branch's leaves.
    #[arg(long, default_value_t = 0.0)]
    seam_weight: f64,
    /// BDT: reset a leaf to a copy of its delegator's shadow (see --revise)
    /// when, over --split-threshold examples routed to it, its mean gain over
    /// the shadow by --trial-error is below minus this many standard errors
    /// (inf: compare and log only).
    #[arg(long)]
    reset_leaves: Option<f64>,
    /// BDT with --leaf lr and shadows (--revise or --reset-leaves): residual
    /// leaves. A leaf below a delegator holds only a correction (its delta)
    /// on top of the delegator's shadow, which goes on learning from every
    /// example through the delegator; it predicts softmax(shadow logits +
    /// delta logits), one logistic regression whose weights are the sums. A
    /// split's children start with deltas of 0 and no AdaGrad history (see
    /// --residual-inherit-sums). Model files hold the summed leaves, so they
    /// play as plain trees; state files hold the deltas.
    #[arg(long)]
    leaf_residual: bool,
    /// BDT with --leaf-residual: an L2 penalty of this / 2 times the square of
    /// each delta weight an example uses, which shrinks a leaf towards its
    /// delegator's shadow.
    #[arg(long, default_value_t = 0.0)]
    residual_l2: f64,
    /// BDT with --leaf-residual: new deltas start with their shadow's AdaGrad
    /// sums instead of none, and a leaf's sums are its delta's, so a split's
    /// new leaves step as the copies of the leaf without residual leaves
    /// would, rather than at AdaGrad's full first-step rate.
    #[arg(long)]
    residual_inherit_sums: bool,
    /// BDT with shadows (--revise or --reset-leaves), in self-play or train
    /// (rounds of game directories): log per round and delegator the shadow test's gain on the round's positions
    /// scored by the model as it starts the round (fresh), beside the gain
    /// training scores on the same round (stream), each with a standard
    /// error clustered by game. Log only: training and files are unchanged.
    #[arg(long)]
    log_fresh_gains: bool,
    /// BDT with shadows, in self-play or train (rounds of game directories):
    /// what the shadow test scores. stream:
    /// each example as training reaches the delegator. fresh: each round's
    /// positions scored by the model as it starts the round, before it
    /// trains on them (the same windows and thresholds).
    #[arg(long, value_enum, default_value_t = ReviseOn::Stream)]
    revise_on: ReviseOn,
    /// BDT with --leaf-residual: the deltas' AdaGrad step size, as a multiple
    /// of --lr-rate (the shadows' and plain leaves').
    #[arg(long, default_value_t = 1.0)]
    residual_delta_rate: f64,
    /// BDT or BDD (errdag3) with --leaf probit: a split's new leaves (and
    /// leaves reset to their delegator's shadow) start from the posterior of
    /// the leaf they copy with this standard deviation added to every
    /// weight's in quadrature (child = parent + N(0, noise^2)), so they can
    /// move away from it quickly. Shadows get none. 0: plain copies. With
    /// --path-context, also the prior of the leaves' path cells: a new cell's
    /// weights start at mean 0 and variance noise^2.
    #[arg(long, default_value_t = 0.0)]
    split_noise: f64,
    /// BDT or BDD (not with --path-context) with --leaf probit: one draw
    /// margin for the whole tree, learned
    /// from every example by the leaf that predicts it and shared by all the
    /// tree's probits (leaves and shadows), instead of one per leaf.
    #[arg(long)]
    probit_shared_eps: bool,
    /// Hoeffding tree: a split is wrong with probability at most this.
    #[arg(long, default_value_t = HtParams::default().delta)]
    ht_delta: f64,
    /// Hoeffding tree: split on the best test once the Hoeffding bound
    /// (bits) is below this, however close the runner-up.
    #[arg(long, default_value_t = HtParams::default().tie)]
    ht_tie: f64,
    /// Hoeffding tree: positions a leaf sees between split checks.
    #[arg(long, default_value_t = HtParams::default().grace)]
    ht_grace: f64,
    /// Hoeffding tree: positions the bound counts as one independent sample
    /// (the positions of a game share its result).
    #[arg(long, default_value_t = HtParams::default().positions_per_sample)]
    ht_positions_per_sample: f64,
    /// Hoeffding tree: most leaves (default: no limit).
    #[arg(long)]
    ht_max_leaves: Option<usize>,
    /// Hoeffding tree: split on a tie only if the best test's gain is above
    /// the bound too (beats not splitting with confidence 1 - delta).
    #[arg(long)]
    ht_pre_prune: bool,
    /// Hoeffding tree: how a leaf chooses its test and decides to split:
    /// information gain about the result by the Hoeffding bound, or (with
    /// --leaf lr) the gain of refitting the leaf's weights on each side of
    /// the test, chosen on half of the games and checked on the other half.
    #[arg(long, value_enum, default_value_t = Criterion::InfoGain)]
    ht_criterion: Criterion,
    /// Hoeffding tree, model gain: split when the best test gains more than
    /// this many nats per position on the held-out half of the games.
    #[arg(long, default_value_t = HtParams::default().gain_threshold)]
    ht_gain_threshold: f64,
    /// Hoeffding tree, model gain: the L2 penalty of the Newton steps that
    /// estimate the gain (a position adds p(1 - p) per class to a weight's
    /// second derivative).
    #[arg(long, default_value_t = HtParams::default().lambda)]
    ht_lambda: f64,
    /// Hoeffding tree, model gain: the fraction of games whose positions the
    /// split statistics keep, chosen by a hash of the game.
    #[arg(long, default_value_t = HtParams::default().sample)]
    ht_sample: f64,
}

impl ModelArgs {
    fn tan_options(&self) -> TanOptions {
        TanOptions { m: self.tan_lambda.unwrap_or(self.tan_m), relative: self.tan_lambda.is_some(), games: self.tan_games }
    }

    /// The --probit-* options.
    fn probit_params(&self) -> ProbitParams {
        ProbitParams {
            prior: self.probit_prior,
            beta: self.probit_beta,
            tau: self.probit_tau,
            alpha: self.probit_alpha,
            per_game: self.probit_per_game,
            eps: self.probit_eps,
            eps_rate: self.probit_eps_rate,
        }
    }

    /// A new ordinal probit regression with these options.
    fn fresh_probit(&self) -> Probit {
        Probit::new(&CATEGORIES, self.probit_params())
    }

    /// A new hybrid of --hybrid-experts, each new as `self-play` makes it
    /// alone, with a uniform gate.
    fn fresh_hybrid(&self) -> Result<Hybrid> {
        let k = self.hybrid_experts.len();
        if !(2..=hybrid::MAX_EXPERTS).contains(&k) {
            bail!("--hybrid-experts takes 2 to {} experts, got {k}", hybrid::MAX_EXPERTS);
        }
        if let Some(e) = self.hybrid_pin
            && e >= k
        {
            bail!("--hybrid-pin {e}: the hybrid has {k} experts, 0 to {}", k - 1);
        }
        let experts = (self.hybrid_experts.iter())
            .map(|kind| match kind {
                HybridExpert::Lr => Expert::Lr(Logreg::new(3, &CATEGORIES, self.lr_rate)),
                HybridExpert::Probit => Expert::Probit(self.fresh_probit()),
                HybridExpert::Cnbp => Expert::Cnbp(Cnbp::new(3, &CATEGORIES, self.alpha, self.use_probs)),
            })
            .collect();
        let target = match self.hybrid_gate_target {
            HybridGateTarget::Full => hybrid::GateTarget::Full,
            HybridGateTarget::Win => hybrid::GateTarget::Win,
        };
        Ok(Hybrid::new(experts, &CATEGORIES, self.hybrid_gate_rate, self.hybrid_hard, self.hybrid_pin)
            .with_gate_target(target))
    }

    fn fresh_moenb(&self) -> MoeNb {
        MoeNb::new(
            &CATEGORIES,
            self.moe_experts,
            self.alpha,
            self.lr_rate,
            self.moe_gate_scale,
            self.moe_seed,
            self.nbmix_refresh,
        )
        .with_gate_l2(self.moe_gate_l2)
    }

    fn fresh_nbmix(&self) -> NbMix {
        NbMix::new(&CATEGORIES, self.nbmix_components, self.alpha, self.nbmix_warm, self.nbmix_refresh, self.nbmix_seed)
    }

    fn fresh_tangate(&self) -> TanGateMoe {
        TanGateMoe::new(&CATEGORIES, self.moe_experts, self.alpha, self.lr_rate, self.tan_m, self.nbmix_warm, self.nbmix_seed)
    }

    /// A tangate's state saved by `save_state`, with these options.
    fn load_tangate(&self, path: &Path) -> std::io::Result<TanGateMoe> {
        TanGateMoe::load_state(path, self.moe_experts, self.alpha, self.lr_rate, self.tan_m, self.nbmix_warm, self.nbmix_seed)
    }

    fn fresh_nbgate(&self) -> NbGateMoe {
        NbGateMoe::new(
            &CATEGORIES,
            self.moe_experts,
            self.alpha,
            self.lr_rate,
            self.nbmix_warm,
            self.nbmix_refresh,
            self.nbmix_seed,
        )
    }

    /// The naive-Bayes mixtures' options, checked as the model is made (their
    /// constructors assert them).
    fn check_nbmix(&self, model: ModelKind) -> Result<()> {
        let k = match model {
            ModelKind::Moenb | ModelKind::Nbgate | ModelKind::Tangate => self.moe_experts,
            ModelKind::Nbmix => self.nbmix_components,
            _ => return Ok(()),
        };
        if !(1..=64).contains(&k) || self.nbmix_refresh == 0 {
            bail!("a naive-Bayes mixture has 1 to 64 components and refreshes every 1 or more examples");
        }
        Ok(())
    }

    /// Probit nodes are BDT leaves only, and the split noise is theirs: the
    /// options of a `model` that trains.
    fn check_probit_nodes(&self, model: ModelKind) -> Result<()> {
        if self.path_context && model != ModelKind::Bdd {
            bail!("--path-context is for --model bdd");
        }
        let tree = matches!(model, ModelKind::Bdt | ModelKind::Bdd | ModelKind::Ht);
        if tree && (self.router == NodeKind::Probit || self.error_router == Some(NodeKind::Probit)) {
            bail!("probits predict the results, not which child predicts best: --router and --error-router cannot be probit");
        }
        // errdag3: BDDs take probit leaves too.
        if tree && self.leaf == NodeKind::Probit && !matches!(model, ModelKind::Bdt | ModelKind::Bdd) {
            bail!("--leaf probit is only implemented for --model bdt and bdd");
        }
        let probit_tree = matches!(model, ModelKind::Bdt | ModelKind::Bdd) && self.leaf == NodeKind::Probit;
        if probit_tree && self.probit_per_game.is_some() {
            bail!("tree leaves train without their games' lengths: no --probit-per-game with --leaf probit");
        }
        if !(self.split_noise >= 0.0) || (self.split_noise != 0.0 && !probit_tree) {
            bail!("--split-noise is a standard deviation (at least 0) for --model bdt or bdd --leaf probit");
        }
        if self.probit_shared_eps && !probit_tree {
            bail!("--probit-shared-eps is for --model bdt or bdd --leaf probit");
        }
        if probit_tree && self.path_context && (self.split_noise <= 0.0 || self.probit_shared_eps) {
            bail!("--path-context with --leaf probit needs --split-noise above 0 (its cells' prior) and no --probit-shared-eps");
        }
        let tan_nodes = tree && [Some(self.leaf), Some(self.router), self.error_router].contains(&Some(NodeKind::Tan));
        if tan_nodes && (self.tan_lambda.is_some() || self.tan_games) {
            bail!("TAN nodes of a tree use a fixed --tan-m: no --tan-lambda or --tan-games");
        }
        if tan_nodes && model != ModelKind::Bdt {
            bail!("TAN nodes are only implemented for --model bdt");
        }
        Ok(())
    }

    /// A new factorization machine with these options.
    fn fresh_fm(&self) -> Fm {
        Fm::new(3, &CATEGORIES, self.fm_factors, self.lr_rate, self.fm_scale, self.fm_seed)
    }

    /// A new Hoeffding tree of one leaf, `root`, with these options.
    fn ht(&self, root: NodeModel) -> Result<Ht> {
        if self.ht_criterion == Criterion::ModelGain && !matches!(root, NodeModel::Lr(_)) {
            bail!("--ht-criterion model-gain needs --leaf lr");
        }
        Ok(Ht::new(root, &CATEGORIES, self.ht_params()))
    }

    fn ht_params(&self) -> HtParams {
        HtParams {
            delta: self.ht_delta,
            tie: self.ht_tie,
            grace: self.ht_grace,
            positions_per_sample: self.ht_positions_per_sample,
            max_leaves: self.ht_max_leaves.unwrap_or(usize::MAX),
            pre_prune: self.ht_pre_prune,
            criterion: self.ht_criterion,
            gain_threshold: self.ht_gain_threshold,
            lambda: self.ht_lambda,
            sample: self.ht_sample,
        }
    }

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
            error_router: self.error_router.unwrap_or(self.router),
            revise: self.revise,
            seam_weight: self.seam_weight,
            reset_leaves: self.reset_leaves,
            leaf_residual: self.leaf_residual,
            residual_l2: self.residual_l2,
            residual_inherit_sums: self.residual_inherit_sums,
            log_fresh_gains: self.log_fresh_gains,
            revise_on: self.revise_on,
            residual_delta_rate: self.residual_delta_rate,
            probit: self.probit_params(),
            split_noise: self.split_noise,
            probit_shared_eps: self.probit_shared_eps,
            tan_m: self.tan_m,
            subrouter_target: self.subrouter_target,
            path_context: self.path_context,
            router_split_z: self.router_split_z,
            router_split_limit: self.router_split_limit,
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
    Probit(Probit),
    Moe(Moe),
    Moenb(MoeNb),
    Nbmix(NbMix),
    Nbgate(NbGateMoe),
    Tangate(TanGateMoe),
    Hybrid(Hybrid),
    Fm(Fm),
    Ht(Ht),
    Tan(Tan),
    Bdt(Bdt),
    Bdd(Bdd),
    HashMix(HashMix),
    Calibrated(Calibrated),
    Uniform(Uniform),
    Float(FloatOnly),
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
    /// The tree's or DAG's nodes in fixed point, built at its first game (see `compound_play`).
    compound: std::sync::OnceLock<Option<chessai::compound::Tables>>,
    /// The lines `fit` printed, for `arena` to print again when A and B share this model (see `load_report`).
    report: String,
}

impl Calibrated {
    /// Fewest decisive positions a leaf needs for its own calibration.
    const MIN_DECISIVE: f64 = 200.0;

    fn fit(model: LoadedModel, games: &Path, per_leaf: bool, blend: Option<f64>) -> Result<Self> {
        let data = games::read_labelled(games).with_context(|| format!("reading {games:?}"))?;
        Self::fit_data(model, &data, games, per_leaf, blend)
    }

    /// `fit` on the labelled positions of `games`, read (hybrid16: the rest of `fit`, unchanged).
    fn fit_data(
        model: LoadedModel,
        data: &[(Encoded, [f64; 3])],
        games: &Path,
        per_leaf: bool,
        blend: Option<f64>,
    ) -> Result<Self> {
        let mut all = Vec::with_capacity(data.len());
        // Each leaf's (log-probabilities, class) pairs and their weights.
        let mut by_leaf = Vec::<(Vec<([f64; 3], usize)>, Vec<f64>)>::new();
        let mut weights = Vec::new();
        for (pos, label) in data {
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
        // Each line is printed when it is made, as before, and kept for `report`.
        let mut report = format!("calibration on {}: {whole} ({} positions)\n", games.display(), all.len());
        print!("{report}");
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
                let line = format!("  leaf {leaf}: {total:.0} positions, {share:.1}% decisive, calibration {shown}\n");
                print!("{line}");
                report.push_str(&line);
                cal
            })
            .collect();
        Ok(Calibrated { model: Box::new(model), per_leaf, blend, leaves, whole, compound: std::sync::OnceLock::new(), report })
    }

    /// hybrid16: `fit` through the calibration cache (`chessai::calcache`): the same fit, printed and reported the
    /// same, taken from the cache when this exact fit was made before. `begun` comes from `calcache::begin`, called
    /// before the inner model loaded; without it, or on anything unexpected, this is `fit` itself.
    fn fit_cached(
        model: LoadedModel,
        begun: Option<calcache::Begun>,
        games: &Path,
        per_leaf: bool,
        blend: Option<f64>,
    ) -> Result<Self> {
        let Some(keyed) = begun.and_then(calcache::Begun::key) else {
            return Self::fit(model, games, per_leaf, blend);
        };
        if let Some(hit) = keyed.lookup() {
            let cal = |c: [u64; 4]| Calibration {
                slope_winner: f64::from_bits(c[0]),
                slope_decisive: f64::from_bits(c[1]),
                offsets: [f64::from_bits(c[2]), f64::from_bits(c[3])],
            };
            let whole = cal(hit.whole);
            // As `fit` prints them: the first line (with this spec's path), then each leaf's line.
            let mut report = format!("calibration on {}: {whole} ({} positions)\n", games.display(), hit.positions);
            print!("{report}");
            for line in hit.leaf_lines.split_inclusive('\n') {
                print!("{line}");
            }
            report.push_str(&hit.leaf_lines);
            let leaves = hit.leaves.into_iter().map(|c| c.map(cal)).collect();
            return Ok(Calibrated {
                model: Box::new(model),
                per_leaf,
                blend,
                leaves,
                whole,
                compound: std::sync::OnceLock::new(),
                report,
            });
        }
        let Some(data) = keyed.read_labelled(games) else {
            return Self::fit(model, games, per_leaf, blend);
        };
        let fitted = Self::fit_data(model, &data, games, per_leaf, blend)?;
        let first = format!("calibration on {}: {} ({} positions)\n", games.display(), fitted.whole, data.len());
        if let Some(leaf_lines) = fitted.report.strip_prefix(&first) {
            let bits = |c: &Calibration| {
                [c.slope_winner.to_bits(), c.slope_decisive.to_bits(), c.offsets[0].to_bits(), c.offsets[1].to_bits()]
            };
            keyed.store(&calcache::Fit {
                positions: data.len() as u64,
                whole: bits(&fitted.whole),
                leaves: fitted.leaves.iter().map(|c| c.as_ref().map(bits)).collect(),
                leaf_lines: leaf_lines.to_string(),
            });
        }
        Ok(fitted)
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
    fn fixed_linear(&self) -> Option<chessai::fixed::FxLinear> {
        if self.per_leaf || self.blend.is_some() {
            return None;
        }
        self.model.classifier().fixed_linear().map(|f| f.with_calibration(self.whole))
    }
    fn log_odds_direct(&self, data: &[u8], white_to_move: bool) -> Option<f64> {
        let z = self.logits(data);
        Some(if white_to_move { z[0] - z[1] } else { z[1] - z[0] })
    }
    /// Its values are `whole`'s z_win - z_loss of the tree's leaf (`logits` without a blend or leaf calibrations).
    /// The model never changes (a calibrated model doesn't train), so its tables are built once.
    fn compound_play(&self) -> Option<chessai::compound::Compound<'_>> {
        if self.per_leaf || self.blend.is_some() {
            return None;
        }
        let inner = self.model.classifier();
        let tables = self.compound.get_or_init(|| inner.compound_tables()).as_ref()?;
        inner.compound_with(self.whole, tables)
    }
}

/// A model that plays with hybrid4's float arithmetic whatever CHESSAI_FX says: no fixed-point sums, no direct
/// values. Loaded as `float:SPEC` (modelopt prototype, to play the float version in the same arena).
struct FloatOnly(Box<LoadedModel>);

impl Classifier for FloatOnly {
    fn train(&mut self, _data: &[u8], _label: &[f64]) {
        unimplemented!("float: models are only for playing")
    }
    fn refresh(&mut self) {}
    fn predict(&self, data: &[u8], out: &mut [f64]) {
        self.0.classifier().predict(data, out)
    }
    fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        self.0.classifier().predict_log(data, out)
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
            "probit" => LoadedModel::Probit(Probit::load(path).with_context(|| format!("loading {path:?}"))?),
            "moe" => LoadedModel::Moe(Moe::load(path).with_context(|| format!("loading {path:?}"))?),
            "moenb" => LoadedModel::Moenb(MoeNb::load(path).with_context(|| format!("loading {path:?}"))?),
            "nbmix" => LoadedModel::Nbmix(NbMix::load(path).with_context(|| format!("loading {path:?}"))?),
            "nbgate" => LoadedModel::Nbgate(NbGateMoe::load(path).with_context(|| format!("loading {path:?}"))?),
            "tangate" => LoadedModel::Tangate(TanGateMoe::load(path).with_context(|| format!("loading {path:?}"))?),
            "hybrid" => LoadedModel::Hybrid(Hybrid::load(path, use_probs).with_context(|| format!("loading {path:?}"))?),
            "fm" => LoadedModel::Fm(Fm::load(path).with_context(|| format!("loading {path:?}"))?),
            "ht" => LoadedModel::Ht(Ht::load(path, use_probs).with_context(|| format!("loading {path:?}"))?),
            "tan" => LoadedModel::Tan(Tan::load(path, use_probs).with_context(|| format!("loading {path:?}"))?),
            "bdt" => LoadedModel::Bdt(Bdt::load(path, use_probs).with_context(|| format!("loading {path:?}"))?),
            "bdd" => LoadedModel::Bdd(Bdd::load(path, use_probs).with_context(|| format!("loading {path:?}"))?),
            "hashmix" => LoadedModel::HashMix(HashMix(
                spec[kind.len() + 1..]
                    .split(',')
                    .map(|p| Cnbp::load(Path::new(p), use_probs).with_context(|| format!("loading {p:?}")))
                    .collect::<Result<_>>()?,
            )),
            "float" => LoadedModel::Float(FloatOnly(Box::new(LoadedModel::load(&spec[kind.len() + 1..], use_probs)?))),
            "cal" | "leafcal" => {
                let (games, inner) = spec[kind.len() + 1..]
                    .split_once(':')
                    .with_context(|| format!("expected {kind}:GAMES:KIND:PATH, got {spec}"))?;
                let begun = calcache::begin(kind, kind == "leafcal", None, inner, use_probs, Path::new(games));
                let inner = LoadedModel::load(inner, use_probs)?;
                LoadedModel::Calibrated(Calibrated::fit_cached(inner, begun, Path::new(games), kind == "leafcal", None)?)
            }
            "blend" => {
                let mut parts = spec[kind.len() + 1..].splitn(3, ':');
                let (Some(tau), Some(games), Some(inner)) = (parts.next(), parts.next(), parts.next()) else {
                    bail!("expected blend:TAU:GAMES:KIND:PATH, got {spec}");
                };
                let tau: f64 = tau.parse().with_context(|| format!("blend TAU {tau:?}"))?;
                let begun = calcache::begin(kind, true, Some(tau), inner, use_probs, Path::new(games));
                let inner = LoadedModel::load(inner, use_probs)?;
                LoadedModel::Calibrated(Calibrated::fit_cached(inner, begun, Path::new(games), true, Some(tau))?)
            }
            _ => bail!("unknown model kind {kind:?}, expected cnbp, lr, probit, moe, moenb, nbmix, nbgate, tangate, hybrid, fm, ht, tan, bdt, bdd, cal, leafcal or blend"),
        })
    }

    /// Everything loading this model printed: each calibration's lines, inner models first, in the order `load`
    /// printed them. `None` unless every part is a kind whose loading prints nothing else (checked for cnbp, lr,
    /// probit, bdt, bdd and rand, and the cal, leafcal, blend and float wrappers), so that `arena` can share one
    /// load between A and B and still print what two loads print.
    fn load_report(&self) -> Option<String> {
        match self {
            LoadedModel::Calibrated(c) => Some(c.model.load_report()? + &c.report),
            LoadedModel::Float(FloatOnly(inner)) => inner.load_report(),
            LoadedModel::Cnbp(_)
            | LoadedModel::Lr(_)
            | LoadedModel::Probit(_)
            | LoadedModel::Bdt(_)
            | LoadedModel::Bdd(_)
            | LoadedModel::Uniform(_) => Some(String::new()),
            _ => None,
        }
    }

    /// Index of the leaf that predicts the position: the tree's leaf, or 0
    /// for models that are not trees.
    fn leaf(&self, data: &[u8]) -> usize {
        match self {
            LoadedModel::Bdt(m) => m.leaf(data),
            LoadedModel::Bdd(m) => m.leaf(data),
            LoadedModel::Ht(m) => m.leaf(data),
            _ => 0,
        }
    }

    /// Each leaf's weight in a blend (`Graph::leaf_weights`, dropping
    /// paths below 0.001); the whole model, as leaf 0, if not a tree.
    fn leaf_weights(&self, data: &[u8], tau: f64, out: &mut Vec<(usize, f64)>) {
        match self {
            LoadedModel::Bdt(m) => m.leaf_weights(data, tau, 1e-3, out),
            LoadedModel::Bdd(m) => m.leaf_weights(data, tau, 1e-3, out),
            // Its tests are hard: all weight on the position's leaf.
            LoadedModel::Ht(m) => {
                out.clear();
                out.push((m.leaf(data), 1.0));
            }
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
            LoadedModel::Ht(m) => m.node_predict_log(leaf, data, out),
            _ => self.classifier().predict_log(data, out),
        }
    }

    fn classifier(&self) -> &(dyn Classifier + Sync) {
        match self {
            LoadedModel::Cnbp(m) => m,
            LoadedModel::Lr(m) => m,
            LoadedModel::Probit(m) => m,
            LoadedModel::Moe(m) => m,
            LoadedModel::Moenb(m) => m,
            LoadedModel::Nbmix(m) => m,
            LoadedModel::Nbgate(m) => m,
            LoadedModel::Tangate(m) => m,
            LoadedModel::Hybrid(m) => m,
            LoadedModel::Fm(m) => m,
            LoadedModel::Ht(m) => m,
            LoadedModel::Tan(m) => m,
            LoadedModel::Bdt(m) => m,
            LoadedModel::Bdd(m) => m,
            LoadedModel::HashMix(m) => m,
            LoadedModel::Calibrated(m) => m,
            LoadedModel::Uniform(m) => m,
            LoadedModel::Float(m) => m,
        }
    }

    /// The model's player at a search depth. Without search, a random model
    /// (`rand`) chooses moves at random whatever the policy. With search it
    /// plays like any other model, valuing every position the same unless
    /// the game is over: at depth 1 it takes a mate in one, at depth 2 it
    /// also avoids allowing one.
    fn player(&self, policy: Policy, temperature: f64, depth: u32) -> Player<'_> {
        let random = matches!(self, LoadedModel::Uniform(_)) && depth == 0;
        let mut player = Player::new(self.classifier(), if random { Policy::Rand } else { policy }, temperature);
        player.depth = depth;
        player
    }
}

/// Whether `arena` shares one load between A and B when their specs are the same: CHESSAI_ARENA_SHARE=on (the
/// default) or off.
fn arena_share_on() -> bool {
    match std::env::var("CHESSAI_ARENA_SHARE").as_deref() {
        Ok("on") | Err(_) => true,
        Ok("off") => false,
        Ok(other) => panic!("CHESSAI_ARENA_SHARE must be on or off, not {other:?}"),
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
    // A and B with the same spec share one load (a model never changes during play, and every game builds each
    // side's fixed-point state afresh), so own calibration fits the held-out calibration once. What the second load
    // would have printed is printed again, so the output is the same. CHESSAI_ARENA_SHARE=off loads twice.
    let model_a = LoadedModel::load(a, use_probs)?;
    let shared = (a == b && arena_share_on()).then(|| model_a.load_report()).flatten();
    let model_b = match shared {
        Some(report) => {
            print!("{report}");
            None
        }
        None => Some(LoadedModel::load(b, use_probs)?),
    };
    let model_b = model_b.as_ref().unwrap_or(&model_a);
    let (mut player_a, mut player_b) =
        (model_a.player(a_policy, temperature, a_depth), model_b.player(b_policy, temperature, b_depth));
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

/// See `Command::FitCheck`.
fn fit_check(path: &Path, anchor: Option<String>, prefixes: usize, sparse_only: bool, dump: Option<&Path>) -> Result<()> {
    let text = std::fs::read_to_string(path).with_context(|| format!("reading {path:?}"))?;
    let mut names: Vec<&str> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for line in text.lines().skip(1) {
        for name in line.split(',').skip(1).take(2) {
            if seen.insert(name) {
                names.push(name);
            }
        }
    }
    let games = tournament::read_games(path, &names)?;
    let n = names.len();
    let anchor = match anchor {
        Some(a) => Some(names.iter().position(|&x| x == a).with_context(|| format!("{a} has no games"))?),
        None => None,
    };
    let timed = |f: &dyn Fn() -> tournament::Ratings| {
        let start = Instant::now();
        let fit = f();
        (fit, start.elapsed().as_secs_f64())
    };
    // The largest differences in Elo and in errors (absolute, and relative to the dense error).
    let compare = |a: &tournament::Ratings, b: &tournament::Ratings| {
        let both: Vec<usize> = (0..n).filter(|&i| a.rated[i] && b.rated[i]).collect();
        let same = (0..n).all(|i| a.rated[i] == b.rated[i]);
        let elo = both.iter().map(|&i| (a.elo[i] - b.elo[i]).abs()).fold(0.0, f64::max);
        let error = both.iter().map(|&i| (a.errors[i] - b.errors[i]).abs()).filter(|d| !d.is_nan()).fold(0.0, f64::max);
        let relative = both
            .iter()
            .map(|&i| (a.errors[i] - b.errors[i]).abs() / a.errors[i].max(1e-300))
            .filter(|d| !d.is_nan())
            .fold(0.0, f64::max);
        format!("same rated players: {same} ({}); max |elo difference| {elo:.3e}, max |error difference| {error:.3e} (relative {relative:.3e})", both.len())
    };
    println!("{} games, {n} players", games.len());
    let (sparse, ts) = timed(&|| tournament::sparse_fit(n, &games, anchor, None, true));
    println!("sparse fit with errors: {ts:.2}s");
    if !sparse_only {
        let (dense_fit, tf) = timed(&|| tournament::ml_fit(n, &games, anchor, None, false));
        println!("dense fit: {tf:.2}s");
        let (dense, td) = timed(&|| tournament::ml_fit(n, &games, anchor, None, true));
        println!("dense fit with errors: {td:.2}s");
        println!("dense fit vs dense fit with errors: {}", compare(&dense_fit, &dense));
        println!("sparse vs dense: {}", compare(&dense, &sparse));
        if let Some(dump) = dump {
            let mut csv = String::from("player,dense_elo,sparse_elo,dense_error,sparse_error\n");
            for i in (0..n).filter(|&i| dense.rated[i] || sparse.rated[i]) {
                writeln!(csv, "{},{:e},{:e},{:e},{:e}", names[i], dense.elo[i], sparse.elo[i], dense.errors[i], sparse.errors[i])
                    .unwrap();
            }
            std::fs::write(dump, csv)?;
        }
    }
    let last = games.iter().map(|g| g.tournament + 1).max().unwrap_or(0);
    let (mut dense_start, mut sparse_start): (Option<Vec<f64>>, Option<Vec<f64>>) = (None, None);
    for t in last.saturating_sub(prefixes as u64)..last {
        let upto: Vec<tournament::Played> = games.iter().filter(|g| g.tournament <= t).copied().collect();
        let (sparse, ts) = timed(&|| tournament::sparse_fit(n, &upto, anchor, sparse_start.as_deref(), false));
        if sparse_only {
            println!("prefix {}: sparse {ts:.2}s", t + 1);
        } else {
            let (dense, td) = timed(&|| tournament::ml_fit(n, &upto, anchor, dense_start.as_deref(), false));
            println!("prefix {}: dense {td:.2}s, sparse {ts:.2}s; {}", t + 1, compare(&dense, &sparse));
            dense_start = Some(dense.elo);
        }
        sparse_start = Some(sparse.elo);
    }
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
    retired: Option<&Path>,
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
    let mut retired_names: Vec<String> = Vec::new();
    if let Some(path) = retired {
        let text = std::fs::read_to_string(path).with_context(|| format!("reading {path:?}"))?;
        // hybrid7: the retired names so far in a set, not searched one by one (the same check).
        let mut retired_seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for line in text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
            let fields: Vec<&str> = line.split_whitespace().collect();
            let [name, depth, _] = fields[..] else { bail!("{path:?}: expected NAME DEPTH SPEC, got {line:?}") };
            depth.parse::<u32>().with_context(|| format!("depth in {line:?}"))?;
            if entries.iter().any(|&(n, _, _)| n == name) || retired_seen.contains(name) {
                bail!("player {name} is listed twice (or both listed and retired)");
            }
            retired_seen.insert(name);
            retired_names.push(name.to_string());
        }
    }
    let models = specs.iter().map(|s| LoadedModel::load(s, use_probs)).collect::<Result<Vec<_>>>()?;
    let mut entrants = Vec::new();
    for &(name, depth, model) in &entries {
        let player = models[model].player(policy, temperature, depth);
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
    let also = if retired_names.is_empty() { String::new() } else { format!(" ({} retired)", retired_names.len()) };
    println!("tournament: {n} players{also}, {} games per tournament, seed {}", n * (n - 1), opts.seed);
    tournament::run(&entrants, &retired_names, out, &opts)?;
    Ok(())
}

/// Search values moves in log-odds, which the log-odds policies use, or by
/// the outcome at the end of their principal variation, which the
/// expected-score ones use.
fn check_depth(player: &Player) -> Result<()> {
    let searchable = matches!(
        player.policy,
        Policy::LogOdds
            | Policy::LogOddsSoftmax
            | Policy::WldScore
            | Policy::WldScoreJitter10
            | Policy::WldScoreJitter20
            | Policy::Rand
    );
    if player.depth > 0 && !searchable {
        bail!("search (depth > 0) needs a log-odds or expected-score policy, not {:?}", player.policy);
    }
    if player.depth == 0 && player.value != Value::LogOdds {
        bail!("the {:?} value is only used by search (depth > 0)", player.value);
    }
    Ok(())
}

fn main() -> Result<()> {
    let result = run();
    chessai::compound::report();
    chessai::fastprobit::report();
    chessai::search::report();
    result
}

fn run() -> Result<()> {
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
            no_catch_up,
            retired,
            save_games,
            reports_last,
            sparse_fit,
            game_cpu,
            opening_plies,
            opening_temperature,
        } => tournament(
            &players,
            &out,
            policy,
            temperature,
            use_probs,
            anchor,
            retired.as_deref(),
            tournament::Options {
                tournaments,
                seed,
                k,
                prior_draws,
                anchor: None,
                catch_up: !no_catch_up,
                save_games,
                reports_last,
                sparse_fit,
                game_cpu,
                opening: opening_of(opening_plies, opening_temperature)?,
            },
        ),
        Command::FitCheck { games, anchor, prefixes, sparse_only, dump } => {
            fit_check(&games, anchor, prefixes, sparse_only, dump.as_deref())
        }
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
            age_forget,
            resume,
            keep_states,
            init,
            params,
        } => {
            if keep_states == 1 {
                bail!("--keep-states 1 would delete the state of the version before the last, which --resume needs");
            }
            params.check_probit_nodes(model)?;
            if let Some(eps) = params.probit_play_eps
                && (model != ModelKind::Probit || !(eps >= 0.0))
            {
                bail!("--probit-play-eps is a draw margin (at least 0) for --model probit");
            }
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
                age_forget,
                keep_states,
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
                    (NodeKind::Probit, Some(path)) => {
                        let mut model = Probit::load(path).with_context(|| format!("loading {path:?}"))?;
                        if params.init_forget != 1.0 {
                            model.forget(params.init_forget);
                        }
                        NodeModel::Probit(model)
                    }
                    (NodeKind::Probit, None) => NodeModel::Probit(params.fresh_probit()),
                    (NodeKind::Tan, None) => NodeModel::Tan(Tan::new(
                        3,
                        &CATEGORIES,
                        params.alpha,
                        TanOptions::fixed(params.tan_m),
                        params.use_probs,
                    )),
                    (NodeKind::Tan, Some(_)) => bail!("--init is not implemented for --leaf tan"),
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
                ModelKind::Probit if resume => {
                    let saved = |n: u64| run.out.join(format!("probit{n}.model"));
                    let start = (1..).take_while(|&n| saved(n).exists()).last().context("no saved models to resume")?;
                    let load = |n: u64| if n == 0 { Ok(params.fresh_probit()) } else { Probit::load(&saved(n)) };
                    let play = params.probit_play_eps.map(play_with_eps);
                    run.run_playing("probit", load(start)?, load(start - 1)?, start, play.as_ref().map(|f| f as _))
                }
                ModelKind::Probit => {
                    let model = match &init {
                        Some(path) => {
                            let mut model = Probit::load(path).with_context(|| format!("loading {path:?}"))?;
                            if params.init_forget != 1.0 {
                                model.forget(params.init_forget);
                            }
                            model
                        }
                        None => params.fresh_probit(),
                    };
                    let play = params.probit_play_eps.map(play_with_eps);
                    run.run_playing("probit", model.clone(), model, 0, play.as_ref().map(|f| f as _))
                }
                ModelKind::Moe if resume => {
                    let saved = |n: u64| run.out.join(format!("moe{n}.model"));
                    let start = (1..).take_while(|&n| saved(n).exists()).last().context("no saved models to resume")?;
                    if start == 1 {
                        bail!("resume a mixture from round 2 on");
                    }
                    run.run("moe", Moe::load(&saved(start))?, Moe::load(&saved(start - 1))?, start)
                }
                ModelKind::Moenb | ModelKind::Nbmix | ModelKind::Nbgate | ModelKind::Tangate if init.is_some() => {
                    bail!("--init is not implemented for the naive-Bayes mixtures")
                }
                ModelKind::Hybrid if init.is_some() => bail!("--init is not implemented for --model hybrid"),
                ModelKind::Hybrid if resume => {
                    let saved = |n: u64| run.out.join(format!("hybrid{n}.model"));
                    let start = (1..).take_while(|&n| saved(n).exists()).last().context("no saved models to resume")?;
                    let load = |n: u64| -> Result<Hybrid> {
                        if n == 0 {
                            params.fresh_hybrid()
                        } else {
                            Hybrid::load(&saved(n), params.use_probs).with_context(|| format!("loading hybrid{n}"))
                        }
                    };
                    run.run("hybrid", load(start)?, load(start - 1)?, start)
                }
                ModelKind::Hybrid => {
                    let model = params.fresh_hybrid()?;
                    run.run("hybrid", model.clone(), model, 0)
                }
                ModelKind::Moenb if resume => {
                    params.check_nbmix(model)?;
                    let saved = |n: u64| run.out.join(format!("moenb{n}.model"));
                    let start = (1..).take_while(|&n| saved(n).exists()).last().context("no saved models to resume")?;
                    let load = |n: u64| if n == 0 { Ok(params.fresh_moenb()) } else { MoeNb::load(&saved(n)) };
                    run.run("moenb", load(start)?, load(start - 1)?, start)
                }
                ModelKind::Moenb => {
                    params.check_nbmix(model)?;
                    let model = params.fresh_moenb();
                    run.run("moenb", model.clone(), model, 0)
                }
                ModelKind::Nbmix if resume => {
                    params.check_nbmix(model)?;
                    let saved = |n: u64| run.out.join(format!("nbmix{n}.model"));
                    let start = (1..).take_while(|&n| saved(n).exists()).last().context("no saved models to resume")?;
                    let load = |n: u64| if n == 0 { Ok(params.fresh_nbmix()) } else { NbMix::load(&saved(n)) };
                    run.run("nbmix", load(start)?, load(start - 1)?, start)
                }
                ModelKind::Nbmix => {
                    params.check_nbmix(model)?;
                    let model = params.fresh_nbmix();
                    run.run("nbmix", model.clone(), model, 0)
                }
                ModelKind::Nbgate if resume => {
                    params.check_nbmix(model)?;
                    let saved = |n: u64| run.out.join(format!("nbgate{n}.model"));
                    let start = (1..).take_while(|&n| saved(n).exists()).last().context("no saved models to resume")?;
                    let load = |n: u64| if n == 0 { Ok(params.fresh_nbgate()) } else { NbGateMoe::load(&saved(n)) };
                    run.run("nbgate", load(start)?, load(start - 1)?, start)
                }
                ModelKind::Nbgate => {
                    params.check_nbmix(model)?;
                    let model = params.fresh_nbgate();
                    run.run("nbgate", model.clone(), model, 0)
                }
                // Its model files lack the gate's pair counts: it resumes
                // from its state files, like TAN.
                ModelKind::Tangate if resume => {
                    params.check_nbmix(model)?;
                    let saved = |n: u64| run.out.join(format!("tangate{n}.model"));
                    let start = (1..).take_while(|&n| saved(n).exists()).last().context("no saved models to resume")?;
                    let load = |n: u64| -> Result<TanGateMoe> {
                        if n == 0 {
                            return Ok(params.fresh_tangate());
                        }
                        let path = run.out.join(format!("tangate{n}.state"));
                        if !path.exists() {
                            bail!("{path:?} is missing: a tangate resumes from the states of its last two versions");
                        }
                        params.load_tangate(&path).with_context(|| format!("loading the state of tangate{n}"))
                    };
                    run.run("tangate", load(start)?, load(start - 1)?, start)
                }
                ModelKind::Tangate => {
                    params.check_nbmix(model)?;
                    let model = params.fresh_tangate();
                    run.run("tangate", model.clone(), model, 0)
                }
                ModelKind::Fm if resume => {
                    let saved = |n: u64| run.out.join(format!("fm{n}.model"));
                    let start = (1..).take_while(|&n| saved(n).exists()).last().context("no saved models to resume")?;
                    let fresh = || params.fresh_fm();
                    let load = |n: u64| if n == 0 { Ok(fresh()) } else { Fm::load(&saved(n)) };
                    run.run("fm", load(start)?, load(start - 1)?, start)
                }
                // TAN's model files lack its pair counts: it resumes from its
                // state files, like the trees.
                ModelKind::Tan if resume => {
                    let saved = |n: u64| run.out.join(format!("tan{n}.model"));
                    let start = (1..).take_while(|&n| saved(n).exists()).last().context("no saved models to resume")?;
                    let load = |n: u64| -> Result<Tan> {
                        let path = run.out.join(format!("tan{n}.state"));
                        if !path.exists() {
                            bail!("{path:?} is missing: a TAN resumes from the states of its last two versions");
                        }
                        Tan::load_state(&path, params.alpha, params.tan_options(), params.use_probs)
                            .with_context(|| format!("loading the state of tan{n}"))
                    };
                    run.run("tan", load(start)?, load(start - 1)?, start)
                }
                ModelKind::Tan => {
                    if init.is_some() {
                        bail!("a TAN starts from scratch (no --init)");
                    }
                    let model = Tan::new(3, &CATEGORIES, params.alpha, params.tan_options(), params.use_probs);
                    run.run("tan", model.clone(), model, 0)
                }
                // Trees resume from their state files, which the model files
                // lack (see `state`); version 0's is saved at the start.
                ModelKind::Bdt | ModelKind::Bdd | ModelKind::Ht if resume => {
                    let name = format!("{model:?}").to_lowercase();
                    let saved = |n: u64| run.out.join(format!("{name}{n}.model"));
                    let start = (1..).take_while(|&n| saved(n).exists()).last().context("no saved models to resume")?;
                    let state = |n: u64| -> Result<PathBuf> {
                        let path = run.out.join(format!("{name}{n}.state"));
                        if !path.exists() {
                            bail!("{path:?} is missing: a tree resumes from the states of its last two versions");
                        }
                        Ok(path)
                    };
                    let context = |n: u64| format!("loading the state of {name}{n}");
                    match model {
                        ModelKind::Bdt => {
                            let load = |n: u64| -> Result<Bdt> {
                                Bdt::load_state(&state(n)?, &CATEGORIES, params.delegation_params())
                                    .with_context(|| context(n))
                            };
                            run.run("bdt", load(start)?, load(start - 1)?, start)
                        }
                        ModelKind::Bdd => {
                            let load = |n: u64| -> Result<Bdd> {
                                Bdd::load_state(&state(n)?, &CATEGORIES, params.delegation_params())
                                    .with_context(|| context(n))
                            };
                            run.run("bdd", load(start)?, load(start - 1)?, start)
                        }
                        _ => {
                            let root = tree_root()?;
                            let load = |n: u64| -> Result<Ht> {
                                Ht::load_state(&state(n)?, &root, &CATEGORIES, params.ht_params())
                                    .with_context(|| context(n))
                            };
                            run.run("ht", load(start)?, load(start - 1)?, start)
                        }
                    }
                }
                ModelKind::Fm => {
                    if init.is_some() {
                        bail!("a factorization machine starts from scratch (no --init)");
                    }
                    let model = params.fresh_fm();
                    run.run("fm", model.clone(), model, 0)
                }
                ModelKind::Ht => {
                    let model = params.ht(tree_root()?)?;
                    run.run("ht", model.clone(), model, 0)
                }
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
                            moe.with_gate_l2(params.moe_gate_l2).with_hard(params.moe_hard)
                        }
                        None => Moe::new(
                            &CATEGORIES,
                            params.moe_experts,
                            params.lr_rate,
                            params.moe_gate_scale,
                            params.moe_seed,
                        )
                        .with_gate_l2(params.moe_gate_l2)
                        .with_hard(params.moe_hard),
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
        Command::Train { games, out, model, rounds, seed, start, label_depth, label_result, label_model, params } => {
            if label_depth.is_some() && !matches!(model, ModelKind::Lr | ModelKind::Probit) {
                bail!("--label-depth is only implemented for lr and probit");
            }
            params.check_probit_nodes(model)?;
            if label_model.is_some() && label_depth.is_none() {
                bail!("--label-model needs --label-depth");
            }
            let label_model = label_model.map(|spec| LoadedModel::load(&spec, params.use_probs)).transpose()?;
            // A flat model's --start: its model file, forgotten by --init-forget.
            fn flat<M: Classifier>(
                start: Option<&Path>,
                forget: f64,
                load: impl FnOnce(&Path) -> std::io::Result<M>,
            ) -> Result<Option<M>> {
                let Some(path) = start else { return Ok(None) };
                let mut model = load(path).with_context(|| format!("loading {path:?}"))?;
                if forget != 1.0 {
                    model.forget(forget);
                }
                Ok(Some(model))
            }
            let (forget, start) = (params.init_forget, start.as_deref());
            let tree_context = || format!("loading the state {:?}", start.unwrap());
            if start.is_some() && forget != 1.0 && matches!(model, ModelKind::Bdt | ModelKind::Bdd | ModelKind::Ht) {
                bail!("--init-forget applies to the --start of cnbp, lr, fm and moe, not of trees");
            }
            match model {
                ModelKind::Cnbp => {
                    let model = flat(start, forget, |p| Cnbp::load(p, params.use_probs))?
                        .unwrap_or_else(|| Cnbp::new(3, &CATEGORIES, params.alpha, params.use_probs));
                    train_rounds(&games, &out, rounds, seed, "cnbp", model)
                }
                ModelKind::Lr => {
                    let model = flat(start, forget, Logreg::load)?
                        .unwrap_or_else(|| Logreg::new(3, &CATEGORIES, params.lr_rate));
                    match label_depth {
                        Some(depth) => {
                            let fixed = label_model.as_ref().map(|m| m.classifier());
                            train_rounds_labelled(&games, &out, rounds, seed, "lr", model, fixed, depth, label_result)
                        }
                        None => train_rounds(&games, &out, rounds, seed, "lr", model),
                    }
                }
                ModelKind::Probit => {
                    let model = flat(start, forget, Probit::load)?.unwrap_or_else(|| params.fresh_probit());
                    match label_depth {
                        // Labelled training gives positions without their games.
                        Some(_) if model.params().per_game.is_some() => {
                            bail!("--label-depth trains a probit without its positions' games: no --probit-per-game")
                        }
                        Some(depth) => {
                            let fixed = label_model.as_ref().map(|m| m.classifier());
                            train_rounds_labelled(&games, &out, rounds, seed, "probit", model, fixed, depth, label_result)
                        }
                        None => train_rounds(&games, &out, rounds, seed, "probit", model),
                    }
                }
                // Its state file: given, or the one beside the given model file.
                ModelKind::Tan => {
                    let model = flat(start, forget, |p| Tan::load_start(p, params.alpha, params.tan_options(), params.use_probs))?
                        .unwrap_or_else(|| Tan::new(3, &CATEGORIES, params.alpha, params.tan_options(), params.use_probs));
                    train_rounds(&games, &out, rounds, seed, "tan", model)
                }
                ModelKind::Fm => {
                    let model = flat(start, forget, Fm::load)?.unwrap_or_else(|| params.fresh_fm());
                    train_rounds(&games, &out, rounds, seed, "fm", model)
                }
                ModelKind::Moe => {
                    let model = flat(start, forget, Moe::load)?.unwrap_or_else(|| {
                        Moe::new(
                            &CATEGORIES,
                            params.moe_experts,
                            params.lr_rate,
                            params.moe_gate_scale,
                            params.moe_seed,
                        )
                        .with_gate_l2(params.moe_gate_l2)
                        .with_hard(params.moe_hard)
                    });
                    train_rounds(&games, &out, rounds, seed, "moe", model)
                }
                ModelKind::Moenb => {
                    params.check_nbmix(ModelKind::Moenb)?;
                    let model = flat(start, forget, MoeNb::load)?.unwrap_or_else(|| params.fresh_moenb());
                    train_rounds(&games, &out, rounds, seed, "moenb", model)
                }
                ModelKind::Nbmix => {
                    params.check_nbmix(ModelKind::Nbmix)?;
                    let model = flat(start, forget, NbMix::load)?.unwrap_or_else(|| params.fresh_nbmix());
                    train_rounds(&games, &out, rounds, seed, "nbmix", model)
                }
                ModelKind::Nbgate => {
                    params.check_nbmix(ModelKind::Nbgate)?;
                    let model = flat(start, forget, NbGateMoe::load)?.unwrap_or_else(|| params.fresh_nbgate());
                    train_rounds(&games, &out, rounds, seed, "nbgate", model)
                }
                // --start a model file, which keeps its own options.
                ModelKind::Hybrid => {
                    let model = match flat(start, forget, |p| Hybrid::load(p, params.use_probs))? {
                        Some(model) => model,
                        None => params.fresh_hybrid()?,
                    };
                    train_rounds(&games, &out, rounds, seed, "hybrid", model)
                }
                // From scratch, or --start a state file (tangateN.state).
                ModelKind::Tangate => {
                    params.check_nbmix(ModelKind::Tangate)?;
                    let model = flat(start, forget, |p| params.load_tangate(p))?.unwrap_or_else(|| params.fresh_tangate());
                    train_rounds(&games, &out, rounds, seed, "tangate", model)
                }
                ModelKind::Bdt => {
                    let model = match start {
                        Some(path) => {
                            Bdt::load_state(path, &CATEGORIES, params.delegation_params()).with_context(tree_context)?
                        }
                        None => Bdt::new(&CATEGORIES, params.delegation_params()),
                    };
                    train_rounds(&games, &out, rounds, seed, "bdt", model)
                }
                ModelKind::Bdd => {
                    let model = match start {
                        Some(path) => {
                            Bdd::load_state(path, &CATEGORIES, params.delegation_params()).with_context(tree_context)?
                        }
                        None => Bdd::new(&CATEGORIES, params.delegation_params()),
                    };
                    train_rounds(&games, &out, rounds, seed, "bdd", model)
                }
                ModelKind::Ht => {
                    let root = params.delegation_params().new_classifier(params.leaf, 3, &CATEGORIES);
                    let model = match start {
                        Some(path) => {
                            Ht::load_state(path, &root, &CATEGORIES, params.ht_params()).with_context(tree_context)?
                        }
                        None => params.ht(root)?,
                    };
                    train_rounds(&games, &out, rounds, seed, "ht", model)
                }
            }
        }
        Command::ReplayGames {
            players,
            games,
            out,
            policy,
            temperature,
            use_probs,
            seed,
            opening_plies,
            opening_temperature,
        } => {
            let opening = opening_of(opening_plies, opening_temperature)?;
            replay_games(&players, &games, &out, policy, temperature, use_probs, seed, opening)
        }
        Command::PolicyStats { model, games, policy, temperature, temperatures, use_probs, seed } => {
            policy_stats(&model, games, policy, temperature, &temperatures, use_probs, seed)
        }
        Command::ProbitDiag { model, games, seed, eps } => probit_diag(&model, games, seed, &eps),
        Command::MixStats { model, games } => mix_stats(&model, &games),
        Command::Calibration { model, games, use_probs } => calibration(&model, &games, use_probs),
        Command::Routing { model, games, use_probs } => routing(&model, &games, use_probs),
        Command::PathStats { model, games, use_probs } => {
            let LoadedModel::Bdd(bdd) = LoadedModel::load(&model, use_probs)? else { bail!("path-stats needs a bdd:PATH model") };
            let mut data = Vec::new();
            for dir in &games {
                data.extend(games::read_labelled(dir).with_context(|| format!("reading {dir:?}"))?.into_iter().map(|(p, l)| (p.to_vec(), l)));
            }
            print!("{}", bdd.path_report(&data));
            Ok(())
        }
        Command::Leaves { model, out, use_probs } => {
            let model = LoadedModel::load(&model, use_probs)?;
            let leaves: Vec<_> = match &model {
                LoadedModel::Bdt(m) => m.leaves().collect(),
                LoadedModel::Bdd(m) => m.leaves().collect(),
                LoadedModel::Ht(m) => m.leaves().collect(),
                _ => bail!("only bdt, bdd and ht models have leaves"),
            };
            std::fs::create_dir_all(&out)?;
            for (idx, leaf) in leaves {
                let path = out.join(format!("leaf{idx}.model"));
                leaf.save(&path).with_context(|| format!("writing {path:?}"))?;
                println!("leaf {idx} ({:?}): {}", leaf.kind(), path.display());
            }
            Ok(())
        }
        Command::EvalLoss { model, games, use_probs, fit_calibration, calibration, routed_by, leaf, dump } => {
            let routed = routed_by.as_deref().zip(leaf);
            eval_loss(&model, &games, use_probs, fit_calibration, calibration.as_deref(), routed)?;
            match dump {
                Some(path) => dump_predictions(&model, &games, use_probs, routed, &path),
                None => Ok(()),
            }
        }
        Command::RankTest { source, models, positions, playouts, playout_policy, seed, use_probs } => {
            rank_test(&source, &models, positions, playouts, playout_policy, seed, use_probs)
        }
        Command::Xcheck { file } => xcheck(file),
        Command::PlayServe { use_probs, keep } => play_serve(use_probs, keep),
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
                tan_m: tan::DEFAULT_M,
                subrouter_target: SubrouterTarget::Routing,
                path_context: false,
                router_split_z: None,
                router_split_limit: None,
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
        let seed = selfplay::derive_seed(seed, round, u64::MAX);
        let start;
        let positions;
        if model.needs_games() {
            // As self-play trains it: the round's games, then their positions
            // shuffled, each with its game's id.
            let round_games = games::read_round_games(&dir).with_context(|| format!("reading {}", dir.display()))?;
            let mut data: Vec<_> = (round_games.iter().enumerate())
                .flat_map(|(g, game)| game.iter().map(move |&(p, l)| (p, l, round << 32 | g as u64)))
                .collect();
            data.shuffle(&mut StdRng::seed_from_u64(seed));
            start = Instant::now();
            let played: Vec<Vec<Encoded>> = round_games.iter().map(|g| g.iter().map(|&(p, _)| p).collect()).collect();
            model.observe_games(&played.iter().map(Vec::as_slice).collect::<Vec<_>>());
            for (pos, label, game) in &data {
                model.train_in_game(pos, label, *game);
            }
            positions = data.len();
        } else if model.scores_rounds() {
            // A tree that scores each round before training on it (see
            // `Bdt::observe_round`): the same positions in the same order as
            // below (the shuffle depends only on their number), each with its
            // game's id; a round of game directories, as tournament
            // --save-games writes.
            let round_games = games::read_round_games(&dir).with_context(|| format!("reading {}", dir.display()))?;
            let mut data: Vec<(Encoded, [f64; 3], u64)> = (round_games.iter().enumerate())
                .flat_map(|(g, game)| game.iter().map(move |&(p, l)| (p, l, round << 32 | g as u64)))
                .collect();
            data.shuffle(&mut StdRng::seed_from_u64(seed));
            start = Instant::now();
            model.observe_round(&data);
            for (pos, label, game) in &data {
                model.train_in_game(pos, label, *game);
            }
            model.end_round();
            positions = data.len();
        } else {
            let mut data = games::read_round(&dir).with_context(|| format!("reading {}", dir.display()))?;
            data.shuffle(&mut StdRng::seed_from_u64(seed));
            start = Instant::now();
            for (pos, label) in &data {
                model.train(pos, label);
            }
            positions = data.len();
        }
        model.refresh();
        model.save(&out.join(format!("{name}{}.model", round + 1)))?;
        println!("round {round}: {positions} positions, train {:.2}s", start.elapsed().as_secs_f64());
    }
    // A tree's state, to train it further (--start).
    if rounds > 0 {
        model.save_state(&out.join(format!("{name}{rounds}.state")))?;
    }
    Ok(())
}

/// As `train_rounds` for a model that trains on positions, but each
/// position's label is the outcome a `depth`-ply search by the model, as it
/// starts the round (or by `fixed`, as self-play's --label-model), expects
/// from it, mixed with the game's result (`selfplay::search_labels`, as
/// self-play's --label-depth), calibrated on the round's games
/// (`selfplay::fit_calibration`). The positions are the same and trained on
/// in the same order as by `train_rounds`.
#[allow(clippy::too_many_arguments)]
fn train_rounds_labelled<M: Classifier + Sync>(
    games: &Path,
    out: &Path,
    rounds: u64,
    seed: u64,
    name: &str,
    mut model: M,
    fixed: Option<&(dyn Classifier + Sync)>,
    depth: u32,
    result_weight: f64,
) -> Result<()> {
    use rand::seq::SliceRandom;
    std::fs::create_dir_all(out)?;
    println!(
        "labels from depth-{depth} search by {} with result weight {result_weight}",
        if fixed.is_some() { "a fixed model" } else { "the model as it starts each round" }
    );
    for round in 0..rounds {
        let dir = games.join(format!("round{round}"));
        let seed = selfplay::derive_seed(seed, round, u64::MAX);
        let records = read_round_records(&dir).with_context(|| format!("reading {}", dir.display()))?;
        let start = Instant::now();
        let labeller: &(dyn Classifier + Sync) = fixed.unwrap_or(&model);
        let calibration = selfplay::fit_calibration(&records, labeller);
        let labels = selfplay::search_labels(&records, labeller, calibration, depth, result_weight);
        let label_secs = start.elapsed().as_secs_f64();
        let mut data: Vec<(Encoded, [f64; 3])> = (records.iter().zip(&labels))
            .flat_map(|(r, l)| r.positions.iter().copied().zip(l.iter().copied()))
            .collect();
        data.shuffle(&mut StdRng::seed_from_u64(seed));
        let start = Instant::now();
        for (pos, label) in &data {
            model.train(pos, label);
        }
        model.refresh();
        model.save(&out.join(format!("{name}{}.model", round + 1)))?;
        println!(
            "round {round}: {} positions, {} games, calibration {calibration}, labels {label_secs:.2}s, train {:.2}s",
            data.len(),
            records.len(),
            start.elapsed().as_secs_f64()
        );
    }
    if rounds > 0 {
        model.save_state(&out.join(format!("{name}{rounds}.state")))?;
    }
    Ok(())
}

/// The games of a round directory of game directories (links will do), in
/// the order of their names (as `games::read_round`), each rebuilt from its
/// moves (human.games) and checked against its saved positions.
fn read_round_records(dir: &Path) -> Result<Vec<selfplay::GameRecord>> {
    let mut paths = std::fs::read_dir(dir)?.map(|e| Ok(e?.path())).collect::<std::io::Result<Vec<_>>>()?;
    paths.retain(|p| p.join(games::OUTCOME_FILES[0].0).exists());
    paths.sort();
    if paths.is_empty() {
        bail!("{}: no game directories", dir.display());
    }
    paths.iter().map(|p| read_game_record(p).with_context(|| format!("game {}", p.display()))).collect()
}

fn read_game_record(dir: &Path) -> Result<selfplay::GameRecord> {
    use shakmaty::san::SanPlus;
    let saved = games::read_labelled(dir)?;
    let text = std::fs::read_to_string(dir.join("human.games"))?;
    let mut game = Game::default();
    let (mut moves, mut positions) = (Vec::new(), Vec::new());
    for tok in text.split_whitespace() {
        if matches!(tok, "1-0" | "0-1" | "1/2-1/2" | "*") {
            break;
        }
        if tok.ends_with('.') && tok[..tok.len() - 1].bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let san: SanPlus = tok.parse().map_err(|e| anyhow::anyhow!("{tok}: {e}"))?;
        let mv = san.san.to_move(game.position()).map_err(|e| anyhow::anyhow!("{tok}: {e}"))?;
        game.play(mv);
        moves.push(mv);
        positions.push(encoding::encode(game.position()));
    }
    if positions.len() != saved.len() || positions.iter().zip(&saved).any(|(a, (b, _))| a != b) {
        bail!("its moves do not give its saved positions ({} vs {})", positions.len(), saved.len());
    }
    let Some(&(_, label)) = saved.first() else { bail!("no positions") };
    let result = match label {
        [1.0, 0.0, 0.0] => GameResult::WhiteWin,
        [0.0, 1.0, 0.0] => GameResult::BlackWin,
        _ => GameResult::Draw,
    };
    if game.result() != Some(result) && result != GameResult::Draw {
        bail!("its moves do not end in its result");
    }
    Ok(selfplay::GameRecord { result, moves, positions, movetext: text.trim().to_string() })
}

/// The opening of --opening-plies and --opening-temperature (see `selfplay::Opening`).
fn opening_of(plies: usize, temperature: Option<f64>) -> Result<selfplay::Opening> {
    if let Some(t) = temperature {
        if plies == 0 {
            bail!("--opening-temperature is the opening's temperature: it needs --opening-plies (1 or more)");
        }
        if !(t > 0.0 && t.is_finite()) {
            bail!("--opening-temperature must be a number above 0");
        }
    }
    Ok(selfplay::Opening { plies, temperature })
}

#[allow(clippy::too_many_arguments)]
fn replay_games(
    players: &Path,
    games_list: &Path,
    out: &Path,
    policy: Policy,
    temperature: f64,
    use_probs: bool,
    seed: u64,
    opening: selfplay::Opening,
) -> Result<()> {
    use rayon::prelude::*;
    let text = std::fs::read_to_string(players).with_context(|| format!("reading {players:?}"))?;
    let mut entries: Vec<(&str, u32, &str)> = Vec::new();
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        let [name, depth, spec] = fields[..] else { bail!("expected NAME DEPTH SPEC, got {line:?}") };
        entries.push((name, depth.parse()?, spec));
    }
    let n = entries.len();
    let index = |name: &str| entries.iter().position(|&(e, _, _)| e == name).with_context(|| format!("no player {name}"));
    let list = std::fs::read_to_string(games_list)?;
    let mut games: Vec<(u64, usize, usize)> = Vec::new();
    for line in list.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let f: Vec<&str> = line.split_whitespace().collect();
        let [t, w, b] = f[..] else { bail!("expected T WHITE BLACK, got {line:?}") };
        games.push((t.parse()?, index(w)?, index(b)?));
    }
    // Only the players in these games are loaded; specs shared by entries are loaded once.
    let mut specs: Vec<&str> = Vec::new();
    for &(_, w, b) in &games {
        for i in [w, b] {
            if !specs.contains(&entries[i].2) {
                specs.push(entries[i].2);
            }
        }
    }
    let models = specs.iter().map(|s| LoadedModel::load(s, use_probs)).collect::<Result<Vec<_>>>()?;
    let player = |i: usize| -> Result<Player<'_>> {
        let m = specs.iter().position(|&s| s == entries[i].2).unwrap();
        let p = models[m].player(policy, temperature, entries[i].1);
        check_depth(&p)?;
        Ok(p)
    };
    let mut players_of = vec![None; n];
    for &(_, w, b) in &games {
        for i in [w, b] {
            if players_of[i].is_none() {
                players_of[i] = Some(player(i)?);
            }
        }
    }
    let order = tournament::pairings(n);
    let played = games
        .par_iter()
        .map(|&(t, w, b)| {
            let g = order.iter().position(|&p| p == (w, b)).expect("no such pairing");
            let record = selfplay::play_game_opening(
                players_of[w].unwrap(),
                players_of[b].unwrap(),
                selfplay::derive_seed(seed, t, g as u64),
                opening,
            );
            let dir = tournament::game_dir(out, t, entries[w].0, entries[b].0);
            selfplay::write_round(&dir, std::slice::from_ref(&record))?;
            Ok((t, w, b, record.result, record.moves.len()))
        })
        .collect::<Result<Vec<_>>>()?;
    for (t, w, b, result, plies) in played {
        let r = match result {
            GameResult::WhiteWin => "1-0",
            GameResult::BlackWin => "0-1",
            GameResult::Draw => "1/2-1/2",
        };
        println!("{t},{},{},{r},{plies}", entries[w].0, entries[b].0);
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
    let player = model.player(policy, temperature, 0);
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

/// A copy of a probit that predicts with draw margin `eps`, for
/// `--probit-play-eps`.
fn play_with_eps(eps: f64) -> impl Fn(&Probit) -> Probit {
    move |model| {
        let mut copy = model.clone();
        copy.set_eps(eps);
        copy
    }
}

/// See `Command::MixStats`.
fn mix_stats(spec: &str, games: &Path) -> Result<()> {
    let loaded = LoadedModel::load(spec, true)?;
    let k = match &loaded {
        LoadedModel::Moe(m) => m.expert_num(),
        LoadedModel::Moenb(m) => m.component_num(),
        LoadedModel::Nbmix(m) => m.component_num(),
        LoadedModel::Nbgate(m) => m.component_num(),
        LoadedModel::Tangate(m) => m.component_num(),
        LoadedModel::Hybrid(m) => m.expert_num(),
        _ => bail!("mix-stats needs a moe, moenb, nbmix, nbgate, tangate or hybrid model"),
    };
    let weights = |data: &[u8], label: Option<&[f64]>, out: &mut [f64]| match &loaded {
        LoadedModel::Moe(m) => m.component_weights(data, label, out),
        LoadedModel::Moenb(m) => m.component_weights(data, label, out),
        LoadedModel::Nbmix(m) => m.component_weights(data, label, out),
        LoadedModel::Nbgate(m) => m.component_weights(data, label, out),
        LoadedModel::Tangate(m) => m.component_weights(data, label, out),
        LoadedModel::Hybrid(m) => m.component_weights(data, label, out),
        _ => unreachable!(),
    };
    let data = games::read_labelled(games).with_context(|| format!("reading {games:?}"))?;
    let entropy = |w: &[f64]| w.iter().filter(|&&x| x > 0.0).map(|&x| -x * x.ln()).sum::<f64>();
    for (name, with_label) in [("given the position", false), ("given the position and its label", true)] {
        let (mut mean, mut w) = (vec![0.0; k], vec![0.0; k]);
        let (mut h, mut hard) = (0.0, 0usize);
        for (pos, label) in &data {
            weights(pos, with_label.then_some(&label[..]), &mut w);
            h += entropy(&w);
            hard += (w.iter().copied().fold(0.0, f64::max) > 0.99) as usize;
            mean.iter_mut().zip(&w).for_each(|(m, x)| *m += x);
        }
        let n = data.len() as f64;
        mean.iter_mut().for_each(|m| *m /= n);
        println!(
            "{name}: effective components {:.2} of {k} (mean weights {}); per position exp(entropy) {:.2}, largest weight over 0.99 in {:.1}% of {} positions",
            entropy(&mean).exp(),
            mean.iter().map(|m| format!("{m:.3}")).collect::<Vec<_>>().join(" "),
            (h / n).exp(),
            100.0 * hard as f64 / n,
            data.len()
        );
    }
    Ok(())
}

/// See `Command::ProbitDiag`. Each candidate move is kept as the mover's
/// latent advantage's mean after it and the variance of the weights' sum, from
/// which every variant's prediction follows.
fn probit_diag(path: &Path, games: usize, seed: u64, extra_eps: &[f64]) -> Result<()> {
    use chessai::probit::{cdf2, ln_cdf};
    let model = Probit::load(path).with_context(|| format!("loading {path:?}"))?;
    let beta2 = model.params().beta * model.params().beta;
    let player = Player::new(&model, Policy::WldScoreJitter10, 0.0);
    let mut rng = StdRng::seed_from_u64(seed);
    // Candidates (mover's mean, weights' variance), by position; the played
    // positions (white's mean, variance) and whether their game was drawn.
    let (mut cands, mut starts) = (Vec::<(f64, f64)>::new(), vec![0usize]);
    let mut played = Vec::<(f64, f64, bool)>::new();
    let (mut drawn_games, mut plies) = (0usize, 0usize);
    for _ in 0..games {
        let mut game = Game::default();
        let first = played.len();
        while game.result().is_none() {
            let pos = game.position();
            let candidates = player::candidates(pos);
            let sign = if pos.turn().is_white() { 1.0 } else { -1.0 };
            if candidates.len() > 1 {
                for c in &candidates {
                    let (m, v) = model.latent_parts(&c.position);
                    cands.push((sign * m, v));
                }
                starts.push(cands.len());
            }
            let chosen = player::select(player, pos.turn(), &candidates, &mut rng);
            let (m, v) = model.latent_parts(&candidates[chosen].position);
            played.push((m, v, false));
            game.play(candidates[chosen].mv);
            plies += 1;
        }
        if game.result() == Some(GameResult::Draw) {
            drawn_games += 1;
            played[first..].iter_mut().for_each(|p| p.2 = true);
        }
    }
    let draw_share = played.iter().filter(|p| p.2).count() as f64 / played.len() as f64;
    // [P(mover wins), P(mover loses), P(draw)] for a mover's mean m.
    let probs = |m: f64, s2: f64, eps: f64| -> [f64; 3] {
        let s = s2.sqrt();
        let (pw, cw) = cdf2((m - eps) / s);
        let (pl, cl) = cdf2((-m - eps) / s);
        [pw, pl, if m >= 0.0 { cw - pl } else { cl - pw }.max(0.0)]
    };
    // The draw margin whose mean P(draw) over the played positions is the
    // share of positions in drawn games (bisection; P(draw) grows with eps).
    let calibrated = |inflate: bool| {
        let mean_draw = |eps: f64| {
            let s2 = |v: f64| beta2 + if inflate { v } else { 0.0 };
            played.iter().map(|&(m, v, _)| probs(m, s2(v), eps)[2]).sum::<f64>() / played.len() as f64
        };
        let (mut lo, mut hi) = (0.0, 20.0);
        for _ in 0..100 {
            let mid = 0.5 * (lo + hi);
            if mean_draw(mid) < draw_share { lo = mid } else { hi = mid }
        }
        0.5 * (lo + hi)
    };
    let eps = model.eps();
    let (cal, cal_mean) = (calibrated(true), calibrated(false));
    println!(
        "{}: {games} games, {plies} plies, {:.1}% drawn; {:.1}% of the positions are in drawn games; {} positions with 2+ moves, {} candidates",
        path.display(),
        100.0 * drawn_games as f64 / games as f64,
        100.0 * draw_share,
        starts.len() - 1,
        cands.len()
    );
    let mut vs: Vec<f64> = cands.iter().map(|c| c.1).collect();
    vs.sort_by(f64::total_cmp);
    let q = |xs: &[f64], p: f64| xs[((xs.len() - 1) as f64 * p) as usize];
    println!(
        "draw margin {eps:.4}, beta {:.3}; weights' variance over candidates: median {:.2e}, 90% {:.2e}, 99% {:.2e}, max {:.2e} (S / beta at the median {:.5})",
        beta2.sqrt(),
        q(&vs, 0.5),
        q(&vs, 0.9),
        q(&vs, 0.99),
        vs[vs.len() - 1],
        ((beta2 + q(&vs, 0.5)) / beta2).sqrt()
    );
    // (name, inflated, draw margin)
    let mut variants = vec![
        ("as is".to_string(), true, eps),
        ("mean weights".to_string(), false, eps),
        (format!("calibrated eps {cal:.4}"), true, cal),
        (format!("mean weights, calibrated eps {cal_mean:.4}"), false, cal_mean),
    ];
    variants.extend(extra_eps.iter().map(|&e| (format!("eps {e}"), true, e)));
    // The log-odds top move as is, to compare the variants' with.
    let mut top_as_is = Vec::new();
    for (k, (name, inflate, eps)) in variants.iter().enumerate() {
        let mut jitter = StdRng::seed_from_u64(seed ^ 0x5eed);
        let (mut gaps, mut odds_gaps, mut draws) = (Vec::new(), Vec::new(), Vec::new());
        let (mut off, mut top_changed) = (0usize, 0usize);
        for (p, w) in starts.windows(2).enumerate() {
            let c = &cands[w[0]..w[1]];
            let mut expected = Vec::with_capacity(c.len());
            let mut odds = Vec::with_capacity(c.len());
            for &(m, v) in c {
                let s2 = beta2 + if *inflate { v } else { 0.0 };
                let pr = probs(m, s2, *eps);
                draws.push(pr[2]);
                expected.push(pr[0] + pr[2] / 2.0);
                let s = s2.sqrt();
                odds.push(ln_cdf((m - eps) / s) - ln_cdf((-m - eps) / s));
            }
            let top2 = |xs: &[f64]| {
                let mut s = xs.to_vec();
                s.sort_by(|a, b| b.total_cmp(a));
                (s[0], s[0] - s[1])
            };
            let (best, gap) = top2(&expected);
            gaps.push(gap);
            odds_gaps.push(top2(&odds).1);
            let jittered = expected.iter().map(|v| v * (1.0 + jitter.random_range(-0.1..=0.1)));
            let pick = jittered.enumerate().max_by(|a, b| a.1.total_cmp(&b.1)).unwrap().0;
            off += (expected[pick] < best) as usize;
            let top = odds.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).unwrap().0;
            if k == 0 {
                top_as_is.push(top);
            } else if top != top_as_is[p] && odds[top] > odds[top_as_is[p]] {
                top_changed += 1;
            }
        }
        for xs in [&mut gaps, &mut odds_gaps, &mut draws] {
            xs.sort_by(f64::total_cmp);
        }
        let n = starts.len() - 1;
        println!("== {name}");
        println!(
            "   P(draw) over candidates: 10% {:.4}, median {:.4}, 90% {:.4}; expected-score gap of the top two: median {:.1e}, 75% {:.1e}, 90% {:.1e}; jitter10 plays a non-top move {:.1}%",
            q(&draws, 0.1),
            q(&draws, 0.5),
            q(&draws, 0.9),
            q(&gaps, 0.5),
            q(&gaps, 0.75),
            q(&gaps, 0.9),
            100.0 * off as f64 / n as f64
        );
        println!(
            "   log-odds gap of the top two: median {:.3}, 90% {:.3}{}",
            q(&odds_gaps, 0.5),
            q(&odds_gaps, 0.9),
            if k == 0 {
                String::new()
            } else {
                format!("; top log-odds move differs from as is: {:.2}%", 100.0 * top_changed as f64 / n as f64)
            }
        );
    }
    Ok(())
}

/// Winner log-loss of white log-odds `v` for a game white won (`y` 1) or lost
/// (`y` -1).
fn winner_loss(v: f64, y: f64) -> f64 {
    let z = y * v;
    if z > 0.0 { (-z).exp().ln_1p() } else { -z + z.exp().ln_1p() }
}

/// See `Command::Routing`.
fn routing(spec: &str, games: &Path, use_probs: bool) -> Result<()> {
    let LoadedModel::Bdt(bdt) = LoadedModel::load(spec, use_probs)? else { bail!("routing needs a bdt: model") };
    let data = games::read_labelled(games)?;
    let raw_odds = |leaf: usize, pos: &[u8]| {
        let mut out = [0.0; 3];
        bdt.node_predict_log(leaf, pos, &mut out);
        out[0] - out[1]
    };
    let decisive: Vec<(&[u8], f64)> = data
        .iter()
        .filter(|(_, label)| label[2] == 0.0)
        .map(|(pos, label)| (&pos[..], if label[0] == 1.0 { 1.0 } else { -1.0 }))
        .collect();
    if decisive.is_empty() {
        bail!("no decisive positions in {games:?}");
    }

    // One scale and offset for the whole tree's log-odds, by Newton's method.
    let tree_odds: Vec<f64> = decisive.iter().map(|(pos, _)| raw_odds(bdt.leaf(pos), pos)).collect();
    let (mut a, mut b) = (1.0, 0.0);
    for _ in 0..50 {
        let (mut ga, mut gb, mut haa, mut hab, mut hbb) = (0.0, 0.0, 0.0, 0.0, 0.0);
        for ((_, y), v) in decisive.iter().zip(&tree_odds) {
            let s = 1.0 / (1.0 + (y * (a * v + b)).exp());
            ga -= s * y * v;
            gb -= s * y;
            let w = s * (1.0 - s);
            haa += w * v * v;
            hab += w * v;
            hbb += w;
        }
        let det = haa * hbb - hab * hab;
        if det.abs() < 1e-300 {
            break;
        }
        let (da, db) = ((hbb * ga - hab * gb) / det, (haa * gb - hab * ga) / det);
        a -= da;
        b -= db;
        if da.abs() + db.abs() < 1e-10 {
            break;
        }
    }
    let odds = |leaf: usize, pos: &[u8]| a * raw_odds(leaf, pos) + b;
    let tree_loss = decisive.iter().zip(&tree_odds).map(|((_, y), v)| winner_loss(a * v + b, *y)).sum::<f64>();
    println!(
        "{} decisive positions; the tree's log-odds recalibrated by scale {a:.4}, offset {b:+.4}: winner log-loss {:.4}",
        decisive.len(),
        tree_loss / decisive.len() as f64
    );

    // Per delegator, per child routed to: positions, and each child's summed loss.
    let mut rows: Vec<Vec<(f64, Vec<f64>)>> = (0..bdt.node_count())
        .map(|idx| vec![(0.0, vec![0.0; bdt.children(idx).len()]); bdt.children(idx).len()])
        .collect();
    let mut oracle = vec![0.0; bdt.node_count()];
    for (pos, y) in &decisive {
        let mut idx = 0;
        while !bdt.children(idx).is_empty() {
            let children = bdt.children(idx);
            let leaves: Vec<usize> = children.iter().map(|&child| bdt.leaf_from(child, pos)).collect();
            let losses: Vec<f64> = leaves.iter().map(|&leaf| winner_loss(odds(leaf, pos), *y)).collect();
            let routed = bdt.leaf_from(idx, pos);
            let picked = leaves.iter().position(|&leaf| leaf == routed).expect("the routed leaf is under a child");
            let row = &mut rows[idx][picked];
            row.0 += 1.0;
            row.1.iter_mut().zip(&losses).for_each(|(sum, loss)| *sum += loss);
            oracle[idx] += losses[picked] - losses.iter().cloned().fold(f64::INFINITY, f64::min);
            idx = children[picked];
        }
    }
    for (idx, rows) in rows.iter().enumerate() {
        let n: f64 = rows.iter().map(|row| row.0).sum();
        if n == 0.0 {
            continue;
        }
        let regret: f64 = rows
            .iter()
            .enumerate()
            .filter(|(_, row)| row.0 > 0.0)
            .map(|(k, row)| row.1[k] - row.1.iter().cloned().fold(f64::INFINITY, f64::min))
            .sum();
        let children = bdt.children(idx).iter().map(|c| c.to_string()).collect::<Vec<_>>().join(" ");
        println!(
            "delegator {idx} (children {children}): {n} positions; regret {:.4} per position ({:.4} against each position's best child)",
            regret / n,
            oracle[idx] / n
        );
        for (k, row) in rows.iter().enumerate() {
            if row.0 == 0.0 {
                continue;
            }
            let cells: Vec<String> = row
                .1
                .iter()
                .enumerate()
                .map(|(j, sum)| if j == k { format!("[{:.4}]", sum / row.0) } else { format!("{:.4}", sum / row.0) })
                .collect();
            println!("  to {} ({:.1}%): {}", bdt.children(idx)[k], 100.0 * row.0 / n, cells.join(" "));
        }
    }

    // Near-duplicate leaves: mean absolute difference of their log-odds.
    let leaves: Vec<usize> = bdt.leaves().map(|(idx, _)| idx).collect();
    let mut sums = vec![0.0; leaves.len() * leaves.len()];
    let mut values = vec![0.0; leaves.len()];
    for (pos, _) in &data {
        values.iter_mut().zip(&leaves).for_each(|(v, &leaf)| *v = odds(leaf, pos));
        for i in 0..leaves.len() {
            for j in i + 1..leaves.len() {
                sums[i * leaves.len() + j] += (values[i] - values[j]).abs();
            }
        }
    }
    let mut pairs: Vec<(f64, usize, usize)> = (0..leaves.len())
        .flat_map(|i| (i + 1..leaves.len()).map(move |j| (i, j)))
        .map(|(i, j)| (sums[i * leaves.len() + j] / data.len() as f64, leaves[i], leaves[j]))
        .collect();
    pairs.sort_by(|x, y| x.0.total_cmp(&y.0));
    let shown: Vec<String> = pairs.iter().take(8).map(|(d, i, j)| format!("{i}-{j} {d:.3}")).collect();
    println!(
        "{} leaves, mean |log-odds difference| of a pair over {} positions {:.3}; closest: {}",
        leaves.len(),
        data.len(),
        pairs.iter().map(|p| p.0).sum::<f64>() / pairs.len().max(1) as f64,
        shown.join(", ")
    );
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

/// `eval-loss --dump`: every position `eval_loss` scores, in its order, as
/// "class l0 l1 l2 f0 ... f69" (the log-probabilities `predict_log` gives,
/// which the calibration reads; the encoded features).
fn dump_predictions(model: &str, games: &Path, use_probs: bool, routed: Option<(&str, usize)>, path: &Path) -> Result<()> {
    let model = LoadedModel::load(model, use_probs)?;
    let model = model.classifier();
    let mut data = games::read_labelled(games)?;
    if let Some((spec, leaf)) = routed {
        let router = LoadedModel::load(spec, use_probs)?;
        data.retain(|(pos, _)| router.leaf(pos) == leaf);
    }
    let mut text = String::with_capacity(data.len() * 200);
    for (pos, label) in &data {
        let mut l = [0.0; 3];
        model.predict_log(pos, &mut l);
        let class = label.iter().position(|&x| x == 1.0).unwrap();
        write!(text, "{class} {:e} {:e} {:e}", l[0], l[1], l[2])?;
        for f in pos.iter() {
            write!(text, " {f}")?;
        }
        text.push('\n');
    }
    std::fs::write(path, text).with_context(|| format!("writing {path:?}"))?;
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
    let player = source.player(playout_policy, 0.1, 0);

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
    age_forget: f64,
    keep_states: u64,
}

impl SelfPlayRun {
    /// Plays and trains rounds `start..rounds`; `previous` is the version
    /// before `model`, the opponent with `vs_previous`.
    fn run<M: Classifier + Sync + Clone>(&self, name: &str, model: M, previous: M, start: u64) -> Result<()> {
        self.run_playing(name, model, previous, start, None)
    }

    /// `run`, the players choosing their moves with `play`'s copies of the
    /// models, if given (see `--probit-play-eps`); training is unchanged.
    fn run_playing<M: Classifier + Sync + Clone>(
        &self,
        name: &str,
        mut model: M,
        mut previous: M,
        start: u64,
        play: Option<&dyn Fn(&M) -> M>,
    ) -> Result<()> {
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

        if play.is_some() {
            println!("the players choose their moves with copies of the models (--probit-play-eps); training is unchanged");
        }
        let ladder_path = self.out.join("ladder.csv");
        if self.vs_previous && start == 0 {
            std::fs::write(&ladder_path, "round,version,opponent,wins,losses,draws,score,elo,elo_low,elo_high\n")?;
        }
        if start > 0 {
            println!("resuming at round {start}");
        } else {
            self.save_state(&model, name, 0)?;
        }

        for round in start..self.rounds {
            let start = Instant::now();
            let player = |model| Player { depth: self.depth, ..Player::new(model, self.policy, self.temperature) };
            let copies = play.map(|f| (f(&model), f(&previous)));
            let (playing, playing_previous) = copies.as_ref().map_or((&model, &previous), |(a, b)| (a, b));
            let records = if self.vs_previous {
                let (current, opponent) = (player(playing), player(playing_previous));
                selfplay::play_match(current, opponent, self.games, selfplay::derive_seed(self.seed, round, 0))
            } else {
                selfplay::play_round(player(playing), self.games, self.seed, round)
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
            model.observe_games(&records.iter().map(|r| r.positions.as_slice()).collect::<Vec<_>>());
            let data = selfplay::training_data(&records, &labels, self.seed, round);
            model.observe_round(&data);
            for (pos, label, game) in &data {
                model.train_in_game(pos, label, *game);
            }
            model.end_round();
            if self.round_forget != 1.0 {
                model.forget(self.round_forget);
            }
            if self.age_forget != 0.0 {
                model.forget(((round + 2) as f64 / (round + 1) as f64).powf(self.age_forget));
            }
            model.refresh();
            let train_secs = start.elapsed().as_secs_f64();
            model.save(&self.out.join(format!("{name}{}.model", round + 1)))?;
            self.save_state(&model, name, round + 1)?;

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

    /// Saves version `v`'s training state, if the model keeps any beyond its
    /// model file (see `state`), and deletes the state --keep-states drops.
    fn save_state<M: Classifier>(&self, model: &M, name: &str, v: u64) -> Result<()> {
        if model.save_state(&self.out.join(format!("{name}{v}.state")))?
            && self.keep_states > 0
            && v >= self.keep_states
        {
            match std::fs::remove_file(self.out.join(format!("{name}{}.state", v - self.keep_states))) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(e.into()),
                _ => {}
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------- play1: the dashboard's play page

/// A JSON string literal of `s`.
fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => write!(out, "\\u{:04x}", c as u32).unwrap(),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// A number as JSON: non-finite values as the strings "+inf", "-inf" and "nan".
fn json_num(v: f64) -> String {
    if v.is_nan() {
        "\"nan\"".into()
    } else if v == f64::INFINITY {
        "\"+inf\"".into()
    } else if v == f64::NEG_INFINITY {
        "\"-inf\"".into()
    } else {
        format!("{v}")
    }
}

/// The game after `moves` (UCI, space-separated) from the start position, and each move's SAN.
fn play_replay(moves: &str) -> Result<(Game, Vec<String>)> {
    let mut game = Game::default();
    let mut sans = Vec::new();
    for (i, text) in moves.split_whitespace().enumerate() {
        let uci: UciMove = text.parse().map_err(|_| anyhow::anyhow!("move {} {text:?} is not UCI", i + 1))?;
        let m = uci.to_move(game.position()).map_err(|_| {
            anyhow::anyhow!("move {} {text} is illegal in {}", i + 1, play_fen(game.position()))
        })?;
        sans.push(shakmaty::san::SanPlus::from_move(game.position().clone(), m).to_string());
        game.play(m);
    }
    Ok((game, sans))
}

fn play_fen(pos: &Chess) -> String {
    shakmaty::fen::Fen::from_position(pos, shakmaty::EnPassantMode::Legal).to_string()
}

/// The game's end by this engine's rules (`Game::result`) as JSON: null, or {"result", "reason"}.
fn play_result(game: &Game) -> String {
    let Some(result) = game.result() else { return "null".into() };
    let pos = game.position();
    let reason = if pos.is_checkmate() {
        "checkmate"
    } else if pos.is_stalemate() {
        "stalemate"
    } else if pos.is_insufficient_material() {
        "insufficient material"
    } else if pos.halfmoves() >= 100 {
        "fifty-move rule"
    } else if game.history().iter().filter(|&&h| h == game.hash()).count() >= 3 {
        "threefold repetition"
    } else {
        "other"
    };
    format!("{{\"result\":{},\"reason\":{}}}", json_str(result.pgn()), json_str(reason))
}

/// The model's probabilities of the game's position as JSON: {"white", "black", "draw"}.
fn play_probs(model: &(dyn Classifier + Sync), game: &Game) -> String {
    let mut p = [0.0; 3];
    model.predict(&encoding::encode(game.position()), &mut p);
    format!("{{\"white\":{},\"black\":{},\"draw\":{}}}", json_num(p[0]), json_num(p[1]), json_num(p[2]))
}

/// The loaded models, least recently used first dropped past `keep`.
struct PlayModels {
    keep: usize,
    use_probs: bool,
    models: Vec<(String, LoadedModel, u64)>,
    tick: u64,
}

impl PlayModels {
    /// The model of `spec` (loaded now if it isn't), and the milliseconds its loading took (0 if it was loaded).
    fn get(&mut self, spec: &str) -> Result<(&LoadedModel, f64)> {
        self.tick += 1;
        let mut load_ms = 0.0;
        let i = match self.models.iter().position(|(s, _, _)| s == spec) {
            Some(i) => i,
            None => {
                let t0 = Instant::now();
                let model = LoadedModel::load(spec, self.use_probs)?;
                load_ms = t0.elapsed().as_secs_f64() * 1000.0;
                self.models.push((spec.to_string(), model, 0));
                if self.models.len() > self.keep.max(1) {
                    let last = self.models.len() - 1;
                    let oldest = (0..last).min_by_key(|&j| self.models[j].2).unwrap();
                    self.models.remove(oldest);
                }
                self.models.iter().position(|(s, _, _)| s == spec).unwrap()
            }
        };
        self.models[i].2 = self.tick;
        Ok((&self.models[i].1, load_ms))
    }
}

/// One request (see `Command::PlayServe`): the JSON fields of the answer, without braces.
fn play_request(line: &str, models: &mut PlayModels) -> Result<String> {
    let mut f: std::collections::HashMap<&str, &str> = std::collections::HashMap::new();
    for field in line.split('\t') {
        let (k, v) = field.split_once('=').with_context(|| format!("expected KEY=VALUE, got {field:?}"))?;
        f.insert(k.trim(), v);
    }
    let get = |k: &str| f.get(k).copied().with_context(|| format!("no {k}"));
    let op = get("op")?;
    let moves = f.get("moves").copied().unwrap_or("");
    let (game, sans) = play_replay(moves)?;
    let plies = sans.len();
    match op {
        "legal" => {
            let mut legal: Vec<String> = game
                .position()
                .legal_moves()
                .iter()
                .map(|m| m.to_uci(CastlingMode::Standard).to_string())
                .collect();
            legal.sort();
            let list = legal.iter().map(|u| json_str(u)).collect::<Vec<_>>().join(",");
            let san = sans.iter().map(|s| json_str(s)).collect::<Vec<_>>().join(",");
            Ok(format!(
                "\"op\":\"legal\",\"plies\":{plies},\"fen\":{},\"legal\":[{list}],\"end\":{},\"san\":[{san}]",
                json_str(&play_fen(game.position())),
                play_result(&game)
            ))
        }
        "load" => {
            let (model, load_ms) = models.get(get("spec")?)?;
            let report = model.load_report().unwrap_or_default();
            Ok(format!("\"op\":\"load\",\"load_ms\":{load_ms:.3},\"report\":{}", json_str(&report)))
        }
        "eval" => {
            let (model, load_ms) = models.get(get("spec")?)?;
            Ok(format!(
                "\"op\":\"eval\",\"plies\":{plies},\"load_ms\":{load_ms:.3},\"probs\":{},\"end\":{}",
                play_probs(model.classifier(), &game),
                play_result(&game)
            ))
        }
        "move" => {
            let depth: u32 = get("depth")?.parse().context("depth")?;
            if depth < 1 {
                bail!("depth must be at least 1");
            }
            let policy_name = f.get("policy").copied().unwrap_or("argmax");
            let (policy, temperature) = match policy_name {
                "argmax" => (Policy::LogOdds, 0.0),
                "softmax" => {
                    let t: f64 = f.get("temperature").copied().unwrap_or("0.02").parse().context("temperature")?;
                    if !(t > 0.0) {
                        bail!("temperature must be above 0");
                    }
                    (Policy::LogOddsSoftmax, t)
                }
                other => bail!("policy must be argmax or softmax, not {other:?}"),
            };
            let seed: u64 = f.get("seed").copied().unwrap_or("1").parse().context("seed")?;
            let values = f.get("values").copied() == Some("1");
            let spec = get("spec")?;
            let (model, load_ms) = models.get(spec)?;
            let player = model.player(policy, temperature, depth);
            check_depth(&player)?;
            let legal = game.position().legal_moves();
            if legal.is_empty() {
                bail!("no legal moves: the game is over ({})", play_result(&game));
            }
            let end_before = play_result(&game);
            let before = play_probs(player.model, &game);
            let mut rng = StdRng::seed_from_u64(selfplay::derive_seed(seed, plies as u64, 0));
            let t0 = Instant::now();
            let mv = selfplay::choose_in(player, &game, &mut rng);
            let think_ms = t0.elapsed().as_secs_f64() * 1000.0;
            let uci = mv.to_uci(CastlingMode::Standard).to_string();
            let san = shakmaty::san::SanPlus::from_move(game.position().clone(), mv).to_string();
            let mut vals = String::new();
            if values {
                let t1 = Instant::now();
                let v = selfplay::root_values_in(player, &game);
                let parts: Vec<String> = legal
                    .iter()
                    .zip(&v)
                    .map(|(m, x)| format!("{}:{}", json_str(&m.to_uci(CastlingMode::Standard).to_string()), json_num(*x)))
                    .collect();
                let chosen = legal.iter().position(|m| *m == mv).map(|i| v[i]).unwrap_or(f64::NAN);
                write!(
                    vals,
                    ",\"value\":{},\"values\":{{{}}},\"values_ms\":{:.3}",
                    json_num(chosen),
                    parts.join(","),
                    t1.elapsed().as_secs_f64() * 1000.0
                )
                .unwrap();
            }
            let mut after = game.clone();
            after.play(mv);
            Ok(format!(
                "\"op\":\"move\",\"plies\":{plies},\"move\":{},\"san\":{},\"think_ms\":{think_ms:.3},\"load_ms\":{load_ms:.3},\
                 \"legal\":{},\"probs_before\":{before},\"probs_after\":{},\"end_before\":{end_before},\"end_after\":{},\
                 \"fen_after\":{}{vals}",
                json_str(&uci),
                json_str(&san),
                legal.len(),
                play_probs(player.model, &after),
                play_result(&after),
                json_str(&play_fen(after.position()))
            ))
        }
        other => bail!("unknown op {other:?} (move, eval, legal or load)"),
    }
}

/// See `Command::PlayServe`.
fn play_serve(use_probs: bool, keep: usize) -> Result<()> {
    use std::io::{BufRead, Write as _};
    let mut models = PlayModels { keep, use_probs, models: Vec::new(), tick: 0 };
    let stdin = std::io::stdin();
    let mut out = std::io::stdout();
    writeln!(out, "@@play {{\"ok\":true,\"ready\":\"play1\",\"use_probs\":{use_probs},\"keep\":{keep}}}")?;
    out.flush()?;
    for line in stdin.lock().lines() {
        let line = line?;
        let line = line.trim_end_matches(['\r', '\n']);
        if line.trim().is_empty() {
            continue;
        }
        let t0 = Instant::now();
        let reply = match play_request(line, &mut models) {
            Ok(body) => format!("{{\"ok\":true,{body},\"ms\":{:.3}}}", t0.elapsed().as_secs_f64() * 1000.0),
            Err(e) => format!("{{\"ok\":false,\"error\":{}}}", json_str(&format!("{e:#}"))),
        };
        let mut out = std::io::stdout();
        writeln!(out, "@@play {reply}")?;
        out.flush()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game(moves: &[&str]) -> Game {
        let mut game = Game::default();
        for uci in moves {
            let m = uci.parse::<UciMove>().unwrap().to_move(game.position()).unwrap();
            game.play(m);
        }
        game
    }

    /// The moves `player` chooses in the game's position over 200 seeds, by
    /// their index in `legal_moves()`.
    fn choices(player: Player, game: &Game) -> Vec<usize> {
        let candidates = player::candidates(game.position());
        (0..200).map(|seed| player::choose(player, game, &candidates, &mut StdRng::seed_from_u64(seed))).collect()
    }

    fn index_of(game: &Game, uci: &str) -> usize {
        let m = uci.parse::<UciMove>().unwrap().to_move(game.position()).unwrap();
        game.position().legal_moves().iter().position(|&l| l == m).unwrap()
    }

    #[test]
    fn random_players_search_like_the_others() {
        let random = LoadedModel::Uniform(Uniform);
        // Black to move can mate with Qh4#: at depth 1 it always does, with
        // the tournament's policy or the arena's.
        let g = game(&["f2f3", "e7e5", "g2g4"]);
        let mate = index_of(&g, "d8h4");
        for policy in [Policy::LogOddsSoftmax, Policy::WldScoreJitter10] {
            assert!(choices(random.player(policy, 0.1, 1), &g).iter().all(|&c| c == mate));
        }
        // Without search it plays at random, as before.
        assert!(choices(random.player(Policy::LogOddsSoftmax, 0.1, 0), &g).iter().any(|&c| c != mate));
        // At depth 2 it never lets Black mate (g2g4), but otherwise picks
        // moves at random.
        let g = game(&["f2f3", "e7e5"]);
        let picked = choices(random.player(Policy::LogOddsSoftmax, 0.1, 2), &g);
        assert!(!picked.contains(&index_of(&g, "g2g4")));
        assert!(picked.iter().collect::<std::collections::HashSet<_>>().len() > 5);
    }

    /// hybrid16: `n` positions of random games, labelled mostly by material (as fastprobit's test), written into
    /// `dir` as an own-games directory's three outcome files; returned in `read_labelled`'s order.
    fn own_games(dir: &Path, seed: u64, n: usize) -> Vec<(Encoded, [f64; 3])> {
        std::fs::create_dir_all(dir).unwrap();
        let mut rng = StdRng::seed_from_u64(seed);
        let mut by_class: [Vec<Encoded>; 3] = Default::default();
        let mut pos = Chess::default();
        while by_class.iter().map(Vec::len).sum::<usize>() < n {
            let moves = pos.legal_moves();
            if moves.is_empty() || pos.halfmoves() >= 100 {
                pos = Chess::default();
                continue;
            }
            pos.play_unchecked(moves[rng.random_range(0..moves.len())]);
            let white = pos.board().white().count() as i32 - pos.board().black().count() as i32;
            let c = if white > 0 && rng.random_range(0..4) > 0 {
                0
            } else if white < 0 && rng.random_range(0..4) > 0 {
                1
            } else {
                rng.random_range(0..3)
            };
            by_class[c].push(encoding::encode(&pos));
        }
        let mut data = Vec::new();
        for ((file, label), positions) in games::OUTCOME_FILES.iter().zip(&by_class) {
            let mut text = String::new();
            for p in positions {
                let values: Vec<String> = p.iter().map(|v| v.to_string()).collect();
                writeln!(text, "{}", values.join(" ")).unwrap();
            }
            std::fs::write(dir.join(file), text).unwrap();
            data.extend(positions.iter().map(|p| (*p, *label)));
        }
        data
    }

    /// Trees and DAGs that split early on a few thousand positions (the league's DAG flags otherwise).
    fn small_delegation(leaf: NodeKind, path_context: bool) -> DelegationParams {
        DelegationParams {
            class_num: 3,
            branch_factor: 3,
            split_threshold: 300,
            split_limit: 4,
            forget_factor: 1.0,
            nb_alpha: 1.0,
            use_probs: true,
            leaf,
            router: NodeKind::Cnbp,
            lr_rate: 0.05,
            error: ErrorMeasure::Log,
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
            split_noise: if path_context { 0.03 } else { 0.0 },
            probit_shared_eps: false,
            tan_m: tan::DEFAULT_M,
            subrouter_target: SubrouterTarget::Routing,
            path_context,
            router_split_z: path_context.then_some(0.0),
            router_split_limit: path_context.then_some(3),
        }
    }

    /// The fit's state bit for bit, and its report.
    fn fit_bits(c: &Calibrated) -> (Vec<Option<[u64; 4]>>, String, bool, Option<u64>) {
        let bits = |c: &Calibration| {
            [c.slope_winner.to_bits(), c.slope_decisive.to_bits(), c.offsets[0].to_bits(), c.offsets[1].to_bits()]
        };
        let mut all = vec![Some(bits(&c.whole))];
        all.extend(c.leaves.iter().map(|l| l.as_ref().map(bits)));
        (all, c.report.clone(), c.per_leaf, c.blend.map(f64::to_bits))
    }

    /// hybrid16: a calibration fit through the cache, cold (fitted, then stored) and warm (read back), equals a
    /// fresh fit bit for bit, with the same report, for cal, leafcal and blend over flat LR, probit and CNBP, a tree
    /// and DAGs with and without path context; and a changed model, changed games or a damaged cache file are fitted
    /// afresh, while games outside an own-games directory are never cached.
    #[test]
    fn cached_calibration_fits_equal_fresh_fits_bit_for_bit() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("target").join(format!("calcache-test-{}", std::process::id()));
        let games = root.join("owncal").join("x@1");
        let data = own_games(&games, 5, 6000);
        let models = root.join("models");
        std::fs::create_dir_all(&models).unwrap();
        let mut lr = Logreg::new(3, &CATEGORIES, 0.05);
        let mut probit = Probit::new(&CATEGORIES, ProbitParams::default());
        let mut cnbp = Cnbp::new(3, &CATEGORIES, 1.0, true);
        let mut bdt = Bdt::new(&CATEGORIES, small_delegation(NodeKind::Lr, false));
        let mut bdd = Bdd::new(&CATEGORIES, small_delegation(NodeKind::Lr, false));
        let mut ctx = Bdd::new(&CATEGORIES, small_delegation(NodeKind::Probit, true));
        for (pos, label) in data.iter().cycle().take(3 * data.len()) {
            lr.train(pos, label);
            probit.train(pos, label);
            cnbp.train(pos, label);
            bdt.train(pos, label);
            bdd.train(pos, label);
            ctx.train(pos, label);
        }
        let file = |name: &str| models.join(name).display().to_string();
        lr.save(Path::new(&file("lr.model"))).unwrap();
        probit.save(Path::new(&file("probit.model"))).unwrap();
        cnbp.save(Path::new(&file("cnbp.model"))).unwrap();
        bdt.save(Path::new(&file("bdt.model"))).unwrap();
        bdd.save(Path::new(&file("bdd.model"))).unwrap();
        ctx.save(Path::new(&file("ctx.model"))).unwrap();
        let specs = [
            (format!("lr:{}", file("lr.model")), true),
            (format!("probit:{}", file("probit.model")), true),
            (format!("cnbp:{}", file("cnbp.model")), true),
            (format!("cnbp:{}", file("cnbp.model")), false),
            (format!("bdt:{}", file("bdt.model")), true),
            (format!("bdd:{}", file("bdd.model")), true),
            (format!("bdd:{}", file("ctx.model")), true),
        ];
        let wrappers: [(&str, bool, Option<f64>); 3] = [("cal", false, None), ("leafcal", true, None), ("blend", true, Some(0.5))];
        let fresh = |spec: &str, use_probs: bool, games: &Path, per_leaf: bool, blend: Option<f64>| {
            Calibrated::fit(LoadedModel::load(spec, use_probs).unwrap(), games, per_leaf, blend).unwrap()
        };
        let cached = |wrapper: &str, spec: &str, use_probs: bool, games: &Path, per_leaf: bool, blend: Option<f64>| {
            let begun = calcache::begin(wrapper, per_leaf, blend, spec, use_probs, games);
            assert!(begun.is_some(), "{wrapper}:{spec}: the cache applies");
            Calibrated::fit_cached(LoadedModel::load(spec, use_probs).unwrap(), begun, games, per_leaf, blend).unwrap()
        };
        let key_file = |wrapper: &str, spec: &str, use_probs: bool, games: &Path, per_leaf: bool, blend: Option<f64>| {
            calcache::begin(wrapper, per_leaf, blend, spec, use_probs, games).unwrap().key().unwrap().file().to_path_buf()
        };
        let mut files = std::collections::HashSet::new();
        let mut split = 0;
        for (spec, use_probs) in &specs {
            for &(wrapper, per_leaf, blend) in &wrappers {
                if blend.is_some() && spec.ends_with("ctx.model") {
                    continue; // path context: no blends
                }
                let want = fit_bits(&fresh(spec, *use_probs, &games, per_leaf, blend));
                let path = key_file(wrapper, spec, *use_probs, &games, per_leaf, blend);
                assert!(!path.exists() && files.insert(path.clone()), "{wrapper}:{spec}: a key of its own");
                let cold = fit_bits(&cached(wrapper, spec, *use_probs, &games, per_leaf, blend));
                assert!(path.exists(), "{wrapper}:{spec}: stored");
                let warm = fit_bits(&cached(wrapper, spec, *use_probs, &games, per_leaf, blend));
                assert_eq!(cold, want, "{wrapper}:{spec} use_probs {use_probs}: cold");
                assert_eq!(warm, want, "{wrapper}:{spec} use_probs {use_probs}: warm");
                if per_leaf && want.0.len() > 2 {
                    split += 1;
                }
            }
        }
        assert_eq!(split, 5, "every tree and DAG split (per-leaf fits with several leaves)");

        // The warm fit is the stored one: another value stored under the key is what a fit then returns.
        let (spec, use_probs) = (&specs[4].0, true);
        let begun = || calcache::begin("leafcal", true, None, spec, use_probs, &games);
        let keyed = begun().unwrap().key().unwrap();
        let mut planted = keyed.lookup().expect("stored");
        planted.whole[0] = 2.5f64.to_bits();
        keyed.store(&planted);
        let got = Calibrated::fit_cached(LoadedModel::load(spec, use_probs).unwrap(), begun(), &games, true, None).unwrap();
        assert_eq!(got.whole.slope_winner, 2.5);
        // A damaged file is fitted afresh, and stored again whole.
        let want = fit_bits(&fresh(spec, use_probs, &games, true, None));
        let mut bytes = std::fs::read(keyed.file()).unwrap();
        let mid = bytes.len() / 2;
        bytes[mid] ^= 1;
        std::fs::write(keyed.file(), &bytes).unwrap();
        assert_eq!(keyed.lookup(), None);
        let got = Calibrated::fit_cached(LoadedModel::load(spec, use_probs).unwrap(), begun(), &games, true, None).unwrap();
        assert_eq!(fit_bits(&got), want);
        assert_eq!(keyed.lookup().map(|f| f.whole[0]), Some(want.0[0].unwrap()[0]));
        std::fs::write(keyed.file(), b"").unwrap();
        let got = Calibrated::fit_cached(LoadedModel::load(spec, use_probs).unwrap(), begun(), &games, true, None).unwrap();
        assert_eq!(fit_bits(&got), want);

        // A changed model (same path) is another key, fitted afresh.
        let lr_spec = &specs[0].0;
        let before = key_file("cal", lr_spec, true, &games, false, None);
        let mut lr2 = Logreg::new(3, &CATEGORIES, 0.05);
        data.iter().take(1000).for_each(|(pos, label)| lr2.train(pos, label));
        lr2.save(Path::new(&file("lr.model"))).unwrap();
        let after = key_file("cal", lr_spec, true, &games, false, None);
        assert_ne!(before, after);
        let want = fit_bits(&fresh(lr_spec, true, &games, false, None));
        assert_eq!(fit_bits(&cached("cal", lr_spec, true, &games, false, None)), want);
        assert_eq!(fit_bits(&cached("cal", lr_spec, true, &games, false, None)), want);

        // Changed games (one position fewer) are another key, fitted afresh.
        let games2 = root.join("owncal").join("x@2");
        std::fs::create_dir_all(&games2).unwrap();
        for (f, _) in games::OUTCOME_FILES {
            std::fs::copy(games.join(f), games2.join(f)).unwrap();
        }
        let draws = std::fs::read_to_string(games2.join("draws.games")).unwrap();
        let cut = draws.trim_end().rfind('\n').unwrap() + 1;
        std::fs::write(games2.join("draws.games"), &draws[..cut]).unwrap();
        let bdd_spec = &specs[5].0;
        assert_ne!(key_file("blend", bdd_spec, true, &games, true, Some(0.5)).file_name(),
                   key_file("blend", bdd_spec, true, &games2, true, Some(0.5)).file_name());
        let want = fit_bits(&fresh(bdd_spec, true, &games2, true, Some(0.5)));
        assert_ne!(want, fit_bits(&fresh(bdd_spec, true, &games, true, Some(0.5))));
        assert_eq!(fit_bits(&cached("blend", bdd_spec, true, &games2, true, Some(0.5))), want);
        assert_eq!(fit_bits(&cached("blend", bdd_spec, true, &games2, true, Some(0.5))), want);

        // Games outside an own-games directory (a held-out set), nested wrappers and other kinds: never cached.
        let held = root.join("heldout").join("round0");
        std::fs::create_dir_all(&held).unwrap();
        for (f, _) in games::OUTCOME_FILES {
            std::fs::copy(games.join(f), held.join(f)).unwrap();
        }
        assert!(calcache::begin("cal", false, None, lr_spec, true, &held).is_none());
        let nested = format!("cal:{}:{lr_spec}", games.display());
        assert!(calcache::begin("cal", false, None, &nested, true, &games).is_none());
        assert!(calcache::begin("cal", false, None, &format!("float:{lr_spec}"), true, &games).is_none());
        assert!(std::fs::read_dir(&held).unwrap().count() == 3);
        std::fs::remove_dir_all(&root).unwrap();
    }
}
