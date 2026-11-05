use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use shakmaty::uci::UciMove;
use shakmaty::{CastlingMode, Chess, Position};

use chessai::bdt::{Bdt, BdtParams};
use chessai::classifier::{Classifier, Uniform};
use chessai::cnbp::Cnbp;
use chessai::encoding::{self, CATEGORIES, Encoded};
use chessai::game::Game;
use chessai::games;
use chessai::game::GameResult;
use chessai::player::{Player, Policy};
use chessai::selfplay;

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
    /// `bdt:PATH` (a saved model) or `rand`.
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
}

#[derive(clap::Args, Clone, Debug)]
struct ModelArgs {
    /// Laplace smoothing.
    #[arg(long, default_value_t = 1.0)]
    alpha: f64,
    /// Predict from cached probabilities (faster) instead of raw counts.
    #[arg(long)]
    use_probs: bool,
    /// BDT: children per split.
    #[arg(long, default_value_t = 2)]
    branch_factor: usize,
    /// BDT: examples a leaf sees between split checks.
    #[arg(long, default_value_t = 2000)]
    split_threshold: u32,
    /// BDT: maximum number of splits.
    #[arg(long, default_value_t = 20)]
    split_limit: u32,
    /// BDT: divisor applied to a splitting leaf's counts.
    #[arg(long, default_value_t = 10.0)]
    forget_factor: f64,
}

impl ModelArgs {
    fn bdt_params(&self) -> BdtParams {
        BdtParams {
            class_num: 3,
            branch_factor: self.branch_factor,
            split_threshold: self.split_threshold,
            split_limit: self.split_limit,
            forget_factor: self.forget_factor,
            nb_alpha: self.alpha,
            use_probs: self.use_probs,
        }
    }
}

fn random_seed() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64
}

/// A player's model, loaded from a `cnbp:PATH`, `bdt:PATH` or `rand` spec.
enum LoadedModel {
    Cnbp(Cnbp),
    Bdt(Bdt),
    Uniform(Uniform),
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
            "bdt" => LoadedModel::Bdt(Bdt::load(path, use_probs).with_context(|| format!("loading {path:?}"))?),
            _ => bail!("unknown model kind {kind:?}, expected cnbp or bdt"),
        })
    }

    fn player(&self, policy: Policy, temperature: f64) -> Player<'_> {
        let model: &(dyn Classifier + Sync) = match self {
            LoadedModel::Cnbp(m) => m,
            LoadedModel::Bdt(m) => m,
            LoadedModel::Uniform(m) => m,
        };
        // A random player chooses moves at random whatever the policy.
        let policy = if matches!(self, LoadedModel::Uniform(_)) { Policy::Rand } else { policy };
        Player { model, policy, temperature }
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
) -> Result<()> {
    let (model_a, model_b) = (LoadedModel::load(a, use_probs)?, LoadedModel::load(b, use_probs)?);
    let (player_a, player_b) = (model_a.player(a_policy, temperature), model_b.player(b_policy, temperature));
    println!("arena: A = {a} ({:?}), B = {b} ({:?}), {games} games, seed {seed}", player_a.policy, player_b.policy);

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

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Arena { a, b, games, a_policy, b_policy, temperature, use_probs, seed, out } => {
            arena(&a, &b, games, (a_policy, b_policy), temperature, use_probs, seed.unwrap_or_else(random_seed), out)
        }
        Command::SelfPlay { out, model, rounds, games, policy, temperature, seed, threads, vs_previous, no_games, params } => {
            if let Some(threads) = threads {
                rayon::ThreadPoolBuilder::new().num_threads(threads).build_global()?;
            }
            let seed = seed.unwrap_or_else(random_seed);
            let run = SelfPlayRun { out, rounds, games, policy, temperature, seed, vs_previous, no_games };
            match model {
                ModelKind::Cnbp => run.run("cnbp", Cnbp::new(3, &CATEGORIES, params.alpha, params.use_probs)),
                ModelKind::Bdt => run.run("bdt", Bdt::new(&CATEGORIES, params.bdt_params())),
            }
        }
        Command::Train { games, out, model, rounds, seed, params } => match model {
            ModelKind::Cnbp => {
                train_rounds(&games, &out, rounds, seed, "cnbp", Cnbp::new(3, &CATEGORIES, params.alpha, params.use_probs))
            }
            ModelKind::Bdt => train_rounds(&games, &out, rounds, seed, "bdt", Bdt::new(&CATEGORIES, params.bdt_params())),
        },
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
            let params = BdtParams {
                class_num: 3,
                branch_factor,
                split_threshold,
                split_limit,
                forget_factor,
                nb_alpha: alpha,
                use_probs,
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
}

impl SelfPlayRun {
    fn run<M: Classifier + Sync + Clone>(&self, name: &str, mut model: M) -> Result<()> {
        std::fs::create_dir_all(&self.out)?;
        println!(
            "self-play: {name}, {} rounds of {} games, {}policy {:?}, seed {}, {} threads",
            self.rounds,
            self.games,
            if self.vs_previous { "version N vs N-1, " } else { "" },
            self.policy,
            self.seed,
            rayon::current_num_threads()
        );

        let ladder_path = self.out.join("ladder.csv");
        if self.vs_previous {
            std::fs::write(&ladder_path, "round,version,opponent,wins,losses,draws,score,elo,elo_low,elo_high\n")?;
        }
        let mut previous = model.clone();

        for round in 0..self.rounds {
            let start = Instant::now();
            let player = |model| Player { model, policy: self.policy, temperature: self.temperature };
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

            let start = Instant::now();
            let data = selfplay::training_data(&records, self.seed, round);
            for (pos, label) in &data {
                model.train(pos, label);
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
