//! Self-play: the model plays both sides, and the positions of finished games
//! become its training data, labelled with the game's result or with the
//! outcome a search by the model expects from them.
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
use shakmaty::san::SanPlus;
use shakmaty::{Move, Position};

use crate::calibration::Calibration;
use crate::classifier::Classifier;
use crate::encoding::{self, Encoded};
use crate::game::{Game, GameResult};
use crate::games::OUTCOME_FILES;
use crate::player::{self, Player, Policy};
use crate::search::{Search, Value};

pub struct GameRecord {
    pub result: GameResult,
    /// The moves, in the order played.
    pub moves: Vec<Move>,
    /// The position after each move.
    pub positions: Vec<Encoded>,
    /// The game in standard notation, e.g. "1. e4 e5 2. Nf3 ... 1-0".
    pub movetext: String,
}

impl GameRecord {
    pub fn label(&self) -> [f64; 3] {
        let mut label = [0.0; 3];
        label[self.class()] = 1.0;
        label
    }

    /// The result as a class: 0 white win, 1 black win, 2 draw.
    pub fn class(&self) -> usize {
        match self.result {
            GameResult::WhiteWin => 0,
            GameResult::BlackWin => 1,
            GameResult::Draw => 2,
        }
    }
}

/// Plays one game between two players.
/// A player's model in fixed point, if CHESSAI_FX is on and the model is linear (see `fixed`).
fn fixed_of(player: Player) -> Option<crate::fixed::FxLinear> {
    if crate::fixed::mode() != crate::fixed::Mode::On || player.policy == Policy::Rand {
        return None;
    }
    player.model.fixed_linear()
}

pub fn play_game(white: Player, black: Player, seed: u64) -> GameRecord {
    let (fx_white, fx_black) = (fixed_of(white), fixed_of(black));
    let mut rng = StdRng::seed_from_u64(seed);
    let mut game = Game::default();
    let mut moves = Vec::new();
    let mut positions = Vec::new();
    let mut movetext = String::new();

    let result = loop {
        if let Some(result) = game.result() {
            break result;
        }
        let pos = game.position();
        let player = if pos.turn().is_white() { white } else { black };
        let fx = if pos.turn().is_white() { fx_white.as_ref() } else { fx_black.as_ref() };
        let (mv, position) = if player.depth >= 1 && player.policy != Policy::Rand {
            // A searching player values moves by its search alone (`choose` doesn't read the candidates), so only
            // the chosen move's position is encoded. Its index is into `legal_moves`, the search's order.
            let index = player::choose_with(player, fx, &game, &[], &mut rng);
            let mv = pos.legal_moves()[index];
            let mut next = pos.clone();
            next.play_unchecked(mv);
            (mv, encoding::encode(&next))
        } else if let Some(fx) = fx {
            let candidates = player::candidates_fx(pos, fx);
            let chosen = &candidates[player::select_fx(player, pos.turn(), &candidates, fx, &mut rng)];
            (chosen.0, chosen.1.enc)
        } else {
            let candidates = player::candidates(pos);
            let chosen = &candidates[player::choose(player, &game, &candidates, &mut rng)];
            (chosen.mv, chosen.position)
        };

        if pos.turn().is_white() {
            write!(movetext, "{}. ", pos.fullmoves()).unwrap();
        }
        write!(movetext, "{} ", SanPlus::from_move(pos.clone(), mv)).unwrap();
        moves.push(mv);
        positions.push(position);
        game.play(mv);
    };

    movetext.push_str(result.pgn());
    GameRecord { result, moves, positions, movetext }
}

/// Plays a round of self-play games in parallel. Records are in game order.
pub fn play_round(player: Player, games: usize, seed: u64, round: u64) -> Vec<GameRecord> {
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

/// Each game's result as the label of each of its positions.
pub fn result_labels(records: &[GameRecord]) -> Vec<Vec<[f64; 3]>> {
    records.iter().map(|r| vec![r.label(); r.positions.len()]).collect()
}

/// Fits a calibration of the model's predictions for the games' positions
/// to the games' results.
pub fn fit_calibration(records: &[GameRecord], model: &(dyn Classifier + Sync)) -> Calibration {
    let data: Vec<([f64; 3], usize)> = records
        .par_iter()
        .flat_map_iter(|r| {
            r.positions.iter().map(move |p| {
                let mut logs = [0.0; 3];
                model.predict_log(p, &mut logs);
                (logs, r.class())
            })
        })
        .collect();
    Calibration::fit(&data)
}

/// Labels each position of the games with the outcome that a `depth`-ply
/// search by the model expects from it (`Search::expected_outcome`), mixed
/// with the game's result: `result_weight` of the result, the rest the
/// search's.
pub fn search_labels(
    records: &[GameRecord],
    model: &(dyn Classifier + Sync),
    calibration: Calibration,
    depth: u32,
    result_weight: f64,
) -> Vec<Vec<[f64; 3]>> {
    let fx = if crate::fixed::mode() == crate::fixed::Mode::On { model.fixed_linear() } else { None };
    records
        .par_iter()
        .map(|r| {
            let result = r.label();
            let mut game = Game::default();
            r.moves
                .iter()
                .map(|&m| {
                    game.play(m);
                    let expected = Search::new(model, &game)
                        .valuing(Value::LogOdds, calibration)
                        .with_fx(fx.as_ref())
                        .expected_outcome(&game, depth);
                    std::array::from_fn(|c| result_weight * result[c] + (1.0 - result_weight) * expected[c])
                })
                .collect()
        })
        .collect()
}

/// Every position of the round with its label (one per position, in game
/// order) and its game's id (the round times 2^32 plus the game's index),
/// shuffled so order-sensitive models (BDT) do not see the outcomes in
/// blocks.
pub fn training_data(
    records: &[GameRecord],
    labels: &[Vec<[f64; 3]>],
    seed: u64,
    round: u64,
) -> Vec<(Encoded, [f64; 3], u64)> {
    let mut data: Vec<_> = records
        .iter()
        .zip(labels)
        .enumerate()
        .flat_map(|(g, (r, l))| {
            r.positions.iter().copied().zip(l.iter().copied()).map(move |(p, l)| (p, l, round << 32 | g as u64))
        })
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

pub(crate) fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cnbp::Cnbp;
    use crate::player::Policy;
    use crate::encoding::CATEGORIES;
    use crate::player::Candidate;
    use shakmaty::Chess;

    #[test]
    fn same_seed_same_games_regardless_of_threads() {
        let model = Cnbp::new(3, &CATEGORIES, 1.0, false);
        let jitter = Player::new(&model, Policy::WldScoreJitter10, 0.0);
        let one_thread = rayon::ThreadPoolBuilder::new().num_threads(1).build().unwrap();
        let a = one_thread.install(|| play_round(jitter, 6, 7, 0));
        let b = play_round(jitter, 6, 7, 0);
        let moves = |rs: &[GameRecord]| rs.iter().map(|r| r.movetext.clone()).collect::<Vec<_>>();
        assert_eq!(moves(&a), moves(&b));
        assert_ne!(moves(&a), moves(&play_round(jitter, 6, 8, 0)));
    }

    #[test]
    fn warns_about_repeated_games() {
        let model = Cnbp::new(3, &CATEGORIES, 1.0, false);
        let player = Player::new(&model, Policy::Rand, 0.0);
        let mut records = vec![play_game(player, player, 1), play_game(player, player, 2)];
        assert!(duplicate_warning(&records).is_none());
        records.push(play_game(player, player, 1));
        assert!(duplicate_warning(&records).unwrap().contains("2 distinct games out of 3"));
    }

    #[test]
    fn positions_match_moves() {
        let model = Cnbp::new(3, &CATEGORIES, 1.0, false);
        let player = Player::new(&model, Policy::Rand, 0.0);
        let record = play_game(player, player, 1);
        let plies = record.movetext.split_whitespace().filter(|t| !t.ends_with('.')).count() - 1;
        assert_eq!(record.positions.len(), plies);
        assert_eq!(record.moves.len(), plies);
    }

    #[test]
    fn search_labels_end_with_the_result() {
        let model = Cnbp::new(3, &CATEGORIES, 1.0, false);
        let player = Player::new(&model, Policy::Rand, 0.0);
        let records: Vec<_> = (0..20).map(|g| play_game(player, player, g)).collect();
        let labels = search_labels(&records, &model, Calibration::NONE, 1, 0.0);
        for (r, l) in records.iter().zip(&labels) {
            assert_eq!(l.len(), r.positions.len());
            // The last position is over by the rules, so its label is exact.
            assert_eq!(*l.last().unwrap(), r.label());
            assert!(l.iter().all(|p| (p.iter().sum::<f64>() - 1.0).abs() < 1e-9));
        }
        // With all the weight on the result, every position gets the result.
        assert_eq!(search_labels(&records, &model, Calibration::NONE, 1, 1.0), result_labels(&records));
    }

    /// hybrid3's `play_game`, kept as the reference for the lazy path: every legal move's position is encoded as a
    /// candidate (by `encode_board_scan`, hybrid3's encoder), whoever is to move and whether or not their choice reads
    /// the candidates.
    fn play_game_eager(white: Player, black: Player, seed: u64) -> GameRecord {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut game = Game::default();
        let mut moves = Vec::new();
        let mut positions = Vec::new();
        let mut movetext = String::new();

        let result = loop {
            if let Some(result) = game.result() {
                break result;
            }
            let pos = game.position();
            let candidates = candidates_scan(pos);
            let player = if pos.turn().is_white() { white } else { black };
            let chosen = &candidates[player::choose(player, &game, &candidates, &mut rng)];

            if pos.turn().is_white() {
                write!(movetext, "{}. ", pos.fullmoves()).unwrap();
            }
            write!(movetext, "{} ", SanPlus::from_move(pos.clone(), chosen.mv)).unwrap();
            moves.push(chosen.mv);
            positions.push(chosen.position);
            game.play(chosen.mv);
        };

        movetext.push_str(result.pgn());
        GameRecord { result, moves, positions, movetext }
    }

    /// hybrid3's `player::candidates`: every legal move, in `legal_moves` order, and its position encoded square by
    /// square.
    fn candidates_scan(pos: &Chess) -> Vec<Candidate> {
        pos.legal_moves()
            .into_iter()
            .map(|mv| {
                let mut next = pos.clone();
                next.play_unchecked(mv);
                Candidate { mv, position: encoding::encode_board_scan(&next) }
            })
            .collect()
    }

    /// Naive Bayes counts from random games, so that predictions differ between positions (an untrained model
    /// predicts the same everywhere, and every searched move would tie).
    fn trained_model() -> Cnbp {
        let blank = Cnbp::new(3, &CATEGORIES, 1.0, false);
        let random = Player::new(&blank, Policy::Rand, 0.0);
        let mut model = Cnbp::new(3, &CATEGORIES, 1.0, false);
        for seed in 0..40 {
            let record = play_game(random, random, 1_000 + seed);
            for position in &record.positions {
                model.train(position, &record.label());
            }
        }
        model.refresh();
        model
    }

    fn searcher(model: &(dyn Classifier + Sync), policy: Policy, temperature: f64, depth: u32) -> Player<'_> {
        let mut player = Player::new(model, policy, temperature);
        player.depth = depth;
        player
    }

    #[test]
    fn searching_players_play_the_same_games_without_the_candidates() {
        let trained = trained_model();
        let blank = Cnbp::new(3, &CATEGORIES, 1.0, false);
        let model: &(dyn Classifier + Sync) = &trained;
        // The league's policy (log-odds softmax at 0.1 nats) and the jittered expected score, at depths 1 and 2.
        let softmax = move |depth| searcher(model, Policy::LogOddsSoftmax, 0.1, depth);
        let jitter = move |depth| searcher(model, Policy::WldScoreJitter10, 0.0, depth);
        let log_odds = searcher(model, Policy::LogOdds, 0.0, 2);
        // Valuing the expected score through a calibration, as the league's entries do with their own.
        let mut calibrated = searcher(model, Policy::LogOddsSoftmax, 0.1, 2);
        calibrated.value = Value::Score;
        calibrated.calibration = Calibration { slope_winner: 0.7, slope_decisive: 1.4, offsets: [0.3, -0.2] };
        // Without search (the self-play policy), which still scores every candidate, and a reference player: an
        // untrained model that searches.
        let unsearched = Player::new(model, Policy::WldScoreJitter10, 0.0);
        let reference = searcher(&blank, Policy::LogOddsSoftmax, 0.1, 2);
        // A random mover with a depth set: `choose` still reads its candidates, so it keeps them.
        let random = searcher(model, Policy::Rand, 0.0, 2);

        let mut pairings = Vec::new();
        for depth in [1, 2] {
            for seed in [1, 2, 3] {
                pairings.push((softmax(depth), softmax(depth), seed));
                pairings.push((jitter(depth), jitter(depth), seed));
            }
        }
        for seed in [4, 5] {
            pairings.push((log_odds, log_odds, seed));
            pairings.push((softmax(1), jitter(2), seed));
            pairings.push((jitter(2), softmax(1), seed));
            pairings.push((unsearched, softmax(2), seed));
            pairings.push((softmax(2), unsearched, seed));
            pairings.push((calibrated, jitter(1), seed));
            pairings.push((jitter(1), calibrated, seed));
            pairings.push((reference, softmax(2), seed));
            pairings.push((softmax(2), reference, seed));
            pairings.push((random, softmax(2), seed));
            pairings.push((jitter(1), random, seed));
        }

        let mut games = Vec::new();
        for (white, black, seed) in pairings {
            let lazy = play_game(white, black, seed);
            let eager = play_game_eager(white, black, seed);
            let pairing =
                format!("{:?} d{} v {:?} d{}, seed {seed}", white.policy, white.depth, black.policy, black.depth);
            assert_eq!(lazy.movetext, eager.movetext, "movetext, {pairing}");
            assert_eq!(lazy.moves, eager.moves, "moves, {pairing}");
            assert_eq!(lazy.positions, eager.positions, "positions, {pairing}");
            assert_eq!(lazy.result, eager.result, "result, {pairing}");
            assert_eq!(lazy.positions.len(), lazy.moves.len(), "{pairing}");
            games.push(lazy);
        }
        // Real games: the seeds give different games, and the searches see results.
        let distinct: std::collections::HashSet<&str> = games.iter().map(|g| g.movetext.as_str()).collect();
        let plies: usize = games.iter().map(|g| g.moves.len()).sum();
        let mut results = [0; 3];
        for g in &games {
            results[g.class()] += 1;
        }
        eprintln!("{} games, {} distinct, {plies} plies; white, black, draw {results:?}", games.len(), distinct.len());
        assert!(distinct.len() * 4 >= games.len() * 3, "only {} distinct games of {}", distinct.len(), games.len());
    }
}
