//! Self-play: the model plays both sides, and the positions of finished games
//! become its training data, labelled with the game's result.
//!
//! Games within a round are independent, so they are played in parallel.
//! Each game has its own seed derived from the run seed, round and game
//! number, so results do not depend on the number of threads.

use std::fmt::Write as _;
use std::io;
use std::path::Path;

use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rayon::prelude::*;
use shakmaty::Position;
use shakmaty::san::SanPlus;

use crate::classifier::Classifier;
use crate::encoding::Encoded;
use crate::game::{Game, GameResult};
use crate::games::OUTCOME_FILES;
use crate::player::{self, Player, Policy};

pub struct GameRecord {
    pub result: GameResult,
    /// The position after each move, in the order played.
    pub positions: Vec<Encoded>,
    /// The game in standard notation, e.g. "1. e4 e5 2. Nf3 ... 1-0".
    pub movetext: String,
}

impl GameRecord {
    pub fn label(&self) -> [f64; 3] {
        match self.result {
            GameResult::WhiteWin => [1.0, 0.0, 0.0],
            GameResult::BlackWin => [0.0, 1.0, 0.0],
            GameResult::Draw => [0.0, 0.0, 1.0],
        }
    }
}

/// Plays one game between two players.
pub fn play_game(white: Player, black: Player, seed: u64) -> GameRecord {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut game = Game::default();
    let mut positions = Vec::new();
    let mut movetext = String::new();

    let result = loop {
        if let Some(result) = game.result() {
            break result;
        }
        let pos = game.position();
        let candidates = player::candidates(pos);
        let player = if pos.turn().is_white() { white } else { black };
        let chosen = &candidates[player::select(player, pos.turn(), &candidates, &mut rng)];

        if pos.turn().is_white() {
            write!(movetext, "{}. ", pos.fullmoves()).unwrap();
        }
        write!(movetext, "{} ", SanPlus::from_move(pos.clone(), chosen.mv)).unwrap();
        positions.push(chosen.position);
        game.play(chosen.mv);
    };

    movetext.push_str(result.pgn());
    GameRecord { result, positions, movetext }
}

/// Plays a round of self-play games in parallel. Records are in game order.
pub fn play_round<M: Classifier + Sync>(
    model: &M,
    policy: Policy,
    games: usize,
    seed: u64,
    round: u64,
) -> Vec<GameRecord> {
    let player = Player { model, policy };
    (0..games as u64)
        .into_par_iter()
        .map(|g| play_game(player, player, derive_seed(seed, round, g)))
        .collect()
}

/// Plays a match in parallel. `a` is white in even games, `b` in odd ones.
/// Records are in game order.
pub fn play_match(a: Player, b: Player, games: usize, seed: u64) -> Vec<GameRecord> {
    (0..games as u64)
        .into_par_iter()
        .map(|g| {
            let (white, black) = if g % 2 == 0 { (a, b) } else { (b, a) };
            play_game(white, black, derive_seed(seed, u64::MAX, g))
        })
        .collect()
}

/// Every position of the round labelled by its game's result, shuffled so
/// order-sensitive models (BDT) do not see the outcomes in blocks.
pub fn training_data(records: &[GameRecord], seed: u64, round: u64) -> Vec<(Encoded, [f64; 3])> {
    let mut data: Vec<_> = records
        .iter()
        .flat_map(|r| r.positions.iter().map(move |&p| (p, r.label())))
        .collect();
    data.shuffle(&mut StdRng::seed_from_u64(derive_seed(seed, round, u64::MAX)));
    data
}

/// Writes the round in the format of self_play.py: one `.games` file of
/// positions per outcome, plus `human.games` with the moves.
pub fn write_round(dir: &Path, records: &[GameRecord]) -> io::Result<()> {
    std::fs::create_dir_all(dir)?;

    for (file, label) in OUTCOME_FILES {
        let mut text = String::new();
        for record in records.iter().filter(|r| r.label() == label) {
            for pos in &record.positions {
                let line: Vec<String> = pos.iter().map(u8::to_string).collect();
                writeln!(text, "{}", line.join(" ")).unwrap();
            }
        }
        std::fs::write(dir.join(file), text)?;
    }

    let human: String = records.iter().map(|r| format!("{}\n\n", r.movetext)).collect();
    std::fs::write(dir.join("human.games"), human)
}

/// Result of a match from the point of view of player A.
pub struct MatchScore {
    pub wins: usize,
    pub losses: usize,
    pub draws: usize,
    /// Fraction of points won, draws counting half.
    pub score: f64,
    /// Elo difference A - B implied by the score, with a 95% range.
    pub elo: f64,
    pub elo_low: f64,
    pub elo_high: f64,
}

/// Scores a match played by `play_match` (A white in even-numbered games).
pub fn score_match(records: &[GameRecord]) -> MatchScore {
    let scores: Vec<f64> = records
        .iter()
        .enumerate()
        .map(|(g, r)| {
            let a_white = g % 2 == 0;
            match r.result {
                GameResult::Draw => 0.5,
                GameResult::WhiteWin if a_white => 1.0,
                GameResult::BlackWin if !a_white => 1.0,
                _ => 0.0,
            }
        })
        .collect();

    let n = scores.len() as f64;
    let wins = scores.iter().filter(|&&s| s == 1.0).count();
    let losses = scores.iter().filter(|&&s| s == 0.0).count();
    let score = scores.iter().sum::<f64>() / n;
    let variance = scores.iter().map(|s| (s - score).powi(2)).sum::<f64>() / n;
    let margin = 1.96 * (variance / n).sqrt();
    let elo = |s: f64| {
        let s = s.clamp(1e-6, 1.0 - 1e-6);
        -400.0 * (1.0 / s - 1.0).log10()
    };

    MatchScore {
        wins,
        losses,
        draws: scores.len() - wins - losses,
        score,
        elo: elo(score),
        elo_low: elo(score - margin),
        elo_high: elo(score + margin),
    }
}

/// A warning if more than 5% of the games are repeats of other games, which
/// makes results look more precise than they are. Happens with policies
/// without jitter, where only exact score ties introduce randomness.
pub fn duplicate_warning(records: &[GameRecord]) -> Option<String> {
    let mut counts = std::collections::HashMap::new();
    for r in records {
        *counts.entry(r.movetext.as_str()).or_insert(0usize) += 1;
    }
    let repeats = records.len() - counts.len();
    if repeats * 20 <= records.len() {
        return None;
    }
    let most = counts.values().max().copied().unwrap_or(0);
    Some(format!(
        "warning: only {} distinct games out of {} (one game was played {most} times); \
         results rest on fewer independent games than it seems. Use a jittered policy for variety.",
        counts.len(),
        records.len()
    ))
}

/// Mixes the run seed, round and game number into an independent seed.
pub fn derive_seed(seed: u64, round: u64, game: u64) -> u64 {
    splitmix64(splitmix64(seed ^ splitmix64(round)) ^ game)
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cnbp::Cnbp;
    use crate::encoding::CATEGORIES;

    #[test]
    fn same_seed_same_games_regardless_of_threads() {
        let model = Cnbp::new(3, &CATEGORIES, 1.0, false);
        let one_thread = rayon::ThreadPoolBuilder::new().num_threads(1).build().unwrap();
        let a = one_thread.install(|| play_round(&model, Policy::WldScoreJitter10, 6, 7, 0));
        let b = play_round(&model, Policy::WldScoreJitter10, 6, 7, 0);
        let moves = |rs: &[GameRecord]| rs.iter().map(|r| r.movetext.clone()).collect::<Vec<_>>();
        assert_eq!(moves(&a), moves(&b));
        assert_ne!(moves(&a), moves(&play_round(&model, Policy::WldScoreJitter10, 6, 8, 0)));
    }

    #[test]
    fn warns_about_repeated_games() {
        let model = Cnbp::new(3, &CATEGORIES, 1.0, false);
        let player = Player { model: &model, policy: Policy::Rand };
        let mut records = vec![play_game(player, player, 1), play_game(player, player, 2)];
        assert!(duplicate_warning(&records).is_none());
        records.push(play_game(player, player, 1));
        assert!(duplicate_warning(&records).unwrap().contains("2 distinct games out of 3"));
    }

    #[test]
    fn positions_match_moves() {
        let model = Cnbp::new(3, &CATEGORIES, 1.0, false);
        let player = Player { model: &model, policy: Policy::Rand };
        let record = play_game(player, player, 1);
        let plies = record.movetext.split_whitespace().filter(|t| !t.ends_with('.')).count() - 1;
        assert_eq!(record.positions.len(), plies);
    }
}
