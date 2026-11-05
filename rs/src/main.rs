use std::fmt::Write as _;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use shakmaty::uci::UciMove;
use shakmaty::{CastlingMode, Chess, Position};

use chessai::cnbp::Cnbp;
use chessai::encoding::{self, CATEGORIES, Encoded};
use chessai::game::Game;
use chessai::games;

#[derive(Parser)]
#[command(about = "chessai: categorical Bayesian chess models")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
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
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Xcheck { file } => xcheck(file),
        Command::BenchMovegen { games, seed } => bench_movegen(games, seed),
        Command::BenchCnbp { games_dir, model_out, predictions_out, alpha, use_probs, reps } => {
            bench_cnbp(games_dir, model_out, predictions_out, alpha, use_probs, reps)
        }
    }
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

fn bench_cnbp(
    games_dir: PathBuf,
    model_out: PathBuf,
    predictions_out: PathBuf,
    alpha: f64,
    use_probs: bool,
    reps: usize,
) -> Result<()> {
    let data = games::read_labelled(&games_dir).with_context(|| format!("reading {games_dir:?}"))?;
    let mut model = Cnbp::new(3, &CATEGORIES, alpha, use_probs);

    let start = Instant::now();
    for _ in 0..reps {
        model = Cnbp::new(3, &CATEGORIES, alpha, use_probs);
        for (pos, label) in &data {
            model.train(pos, label);
        }
        model.refresh();
    }
    let train_secs = start.elapsed().as_secs_f64() / reps as f64;
    model.save(&model_out)?;

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
    std::fs::write(&predictions_out, text)?;

    let n = data.len() as f64;
    println!(
        "rust cnbp: {} positions, train {:.1}M pos/s, predict {:.2}M pos/s",
        data.len(),
        n / train_secs / 1e6,
        n / predict_secs / 1e6
    );
    Ok(())
}
