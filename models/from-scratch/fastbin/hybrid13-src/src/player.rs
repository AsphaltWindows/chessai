//! Choosing moves with a classifier, ported from `c_src/harness/player.c`.
//!
//! Every legal move is played on a copy of the position, the resulting
//! position is encoded and scored by the model as [white win, black win,
//! draw], and the move with the best score for the side to move is chosen.
//! Ties are broken at random.

use clap::ValueEnum;
use rand::RngExt;
use shakmaty::{Chess, Color, Move, Position};

use crate::classifier::Classifier;
use crate::compound::Compound;
use crate::encoding::{self, Encoded};
use crate::game::Game;
use crate::calibration::Calibration;
use crate::fixed::{Acc, FxLinear, Sums};
use crate::search::{Search, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Policy {
    /// Expected score: win + draw / 2.
    WldScore,
    /// Expected score multiplied by a random factor in [0.9, 1.1].
    WldScoreJitter10,
    /// Expected score multiplied by a random factor in [0.8, 1.2].
    WldScoreJitter20,
    /// Sample a move with probability proportional to
    /// exp(expected score / temperature).
    WldScoreSoftmax,
    /// Highest log-odds of winning, ln P(win) - ln P(loss). Unlike expected
    /// score, it still separates moves when P(draw) is close to 1.
    LogOdds,
    /// Sample a move with probability proportional to
    /// exp(log-odds / temperature); the temperature is in nats.
    LogOddsSoftmax,
    /// Prefer positions where win > loss, by win probability; otherwise
    /// the highest draw probability.
    WldWd,
    /// A uniformly random legal move.
    Rand,
}

/// A model and the policy it chooses moves with.
#[derive(Clone, Copy)]
pub struct Player<'a> {
    pub model: &'a (dyn Classifier + Sync),
    pub policy: Policy,
    /// Softmax temperature, in units of expected score (0..1). Only used by
    /// `WldScoreSoftmax`.
    pub temperature: f64,
    /// If above 0, value moves by an alpha-beta search this many plies deep
    /// (see `search`) instead of scoring the candidates directly. Needs a
    /// log-odds policy, which uses the searched values, or an expected-score
    /// one (`WldScore` and its jittered versions), which scores each move by
    /// the expected score at the end of its principal variation.
    pub depth: u32,
    /// How the search values positions, and the calibration of the model's
    /// probabilities it uses.
    pub value: Value,
    pub calibration: Calibration,
}

impl<'a> Player<'a> {
    /// A player that scores the candidate moves directly, without search.
    pub fn new(model: &'a (dyn Classifier + Sync), policy: Policy, temperature: f64) -> Self {
        Player { model, policy, temperature, depth: 0, value: Value::LogOdds, calibration: Calibration::NONE }
    }
}

/// A legal move and the encoding of the position it leads to.
pub struct Candidate {
    pub mv: Move,
    pub position: Encoded,
}

pub fn candidates(pos: &Chess) -> Vec<Candidate> {
    pos.legal_moves()
        .into_iter()
        .map(|mv| {
            let mut next = pos.clone();
            next.play_unchecked(mv);
            Candidate { mv, position: encoding::encode(&next) }
        })
        .collect()
}

/// Every legal move, in `legal_moves()` order, and the fixed-point sums of the position it leads to (see `fixed`).
pub fn candidates_fx(pos: &Chess, fx: &FxLinear) -> Vec<(Move, Sums)> {
    let acc = Acc::new(fx, pos);
    pos.legal_moves()
        .into_iter()
        .map(|mv| {
            let mut next = pos.clone();
            next.play_unchecked(mv);
            (mv, acc.child_sums(fx, pos, &next))
        })
        .collect()
}

/// Index into `candidates`, the legal moves of the game's position in
/// `legal_moves()` order, of the move to play.
pub fn choose<R: RngExt>(player: Player, game: &Game, candidates: &[Candidate], rng: &mut R) -> usize {
    choose_with(player, None, game, candidates, rng)
}

/// `choose`, a searching player valuing positions with `fx`, its model in fixed point, if given.
pub fn choose_with<R: RngExt>(
    player: Player,
    fx: Option<&FxLinear>,
    game: &Game,
    candidates: &[Candidate],
    rng: &mut R,
) -> usize {
    choose_with_compound(player, fx, None, game, candidates, rng)
}

/// `choose_with`, also valuing positions with `cfx`, its model as a compound model in fixed point, if given.
pub fn choose_with_compound<R: RngExt>(
    player: Player,
    fx: Option<&FxLinear>,
    cfx: Option<&Compound>,
    game: &Game,
    candidates: &[Candidate],
    rng: &mut R,
) -> usize {
    let pos = game.position();
    if player.depth == 0 || player.policy == Policy::Rand {
        return select(player, pos.turn(), candidates, rng);
    }
    let mut search =
        Search::new(player.model, game).valuing(player.value, player.calibration).with_fx(fx).with_compound(cfx);
    match player.policy {
        Policy::LogOddsSoftmax
            if windows() && player.temperature > 0.0 && player.temperature <= WINDOW_T_MAX && search.windowable(player.depth) =>
        {
            windowed_softmax(&mut search, game, player.depth, player.temperature, rng)
        }
        Policy::LogOddsSoftmax => softmax_sample(&search.root_values(game, player.depth), player.temperature, rng),
        Policy::LogOdds if windows() && search.windowable(player.depth) => windowed_argmax(&mut search, game, player.depth, rng),
        Policy::LogOdds => {
            let values = search.root_values(game, player.depth);
            let best = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let tied: Vec<usize> = (0..values.len()).filter(|&i| values[i] == best).collect();
            tied[rng.random_range(0..tied.len())]
        }
        Policy::WldScore | Policy::WldScoreJitter10 | Policy::WldScoreJitter20 => {
            // As without search, but a move is worth the expected score at
            // the end of its principal variation: at depth 1 the same
            // score, unless the move ends the game.
            let win = if pos.turn().is_white() { 0 } else { 1 };
            let keys: Vec<(f64, f64)> = search
                .root_outcomes(game, player.depth)
                .iter()
                .map(|outcome| score_key(player.policy, outcome[win] + outcome[2] / 2.0, rng))
                .collect();
            pick_best(&keys, rng)
        }
        _ => panic!("search needs a log-odds or expected-score policy, not {:?}", player.policy),
    }
}

/// Index into `candidates` of the move to play.
pub fn select<R: RngExt>(
    player: Player,
    turn: Color,
    candidates: &[Candidate],
    rng: &mut R,
) -> usize {
    let model = player.model;
    let (win, loss) = match turn {
        Color::White => (0, 1),
        Color::Black => (1, 0),
    };
    let probs = |i: usize| {
        let mut scores = [0.0; 3];
        model.predict(&candidates[i].position, &mut scores);
        scores
    };
    let value = |i: usize| {
        let scores = probs(i);
        log_odds(scores[win], scores[loss])
    };
    select_by(player, turn, candidates.len(), probs, value, rng)
}

/// `select` from fixed-point accumulators (see `candidates_fx`).
pub fn select_fx<R: RngExt>(player: Player, turn: Color, candidates: &[(Move, Sums)], fx: &FxLinear, rng: &mut R) -> usize {
    let probs = |i: usize| fx.probs(&candidates[i].1);
    let value = |i: usize| fx.value(&candidates[i].1, turn.is_white());
    select_by(player, turn, candidates.len(), probs, value, rng)
}

/// Index of the move to play among `n` candidates, given each candidate's probabilities of [white win, black win,
/// draw] (`probs`) and its log-odds for the side to move (`value`).
fn select_by<R: RngExt>(
    player: Player,
    turn: Color,
    n: usize,
    probs: impl Fn(usize) -> [f64; 3],
    value: impl Fn(usize) -> f64,
    rng: &mut R,
) -> usize {
    let Player { policy, temperature, .. } = player;
    if policy == Policy::Rand {
        return rng.random_range(0..n);
    }

    let (win, loss) = match turn {
        Color::White => (0, 1),
        Color::Black => (1, 0),
    };

    if matches!(policy, Policy::WldScoreSoftmax | Policy::LogOddsSoftmax) {
        let values: Vec<f64> = (0..n)
            .map(|i| {
                if policy == Policy::LogOddsSoftmax {
                    value(i)
                } else {
                    let scores = probs(i);
                    scores[win] + scores[2] / 2.0
                }
            })
            .collect();
        return softmax_sample(&values, temperature, rng);
    }

    let keys: Vec<(f64, f64)> = (0..n)
        .map(|i| match policy {
            Policy::WldScore | Policy::WldScoreJitter10 | Policy::WldScoreJitter20 => {
                let scores = probs(i);
                score_key(policy, scores[win] + scores[2] / 2.0, rng)
            }
            Policy::LogOdds => (0.0, value(i)),
            Policy::WldWd => {
                let scores = probs(i);
                if scores[win] > scores[loss] { (1.0, scores[win]) } else { (0.0, scores[2]) }
            }
            Policy::Rand | Policy::WldScoreSoftmax | Policy::LogOddsSoftmax => unreachable!(),
        })
        .collect();
    pick_best(&keys, rng)
}

/// The key of a move whose expected score for the side to move is
/// `expected`, under an expected-score policy; the jittered ones draw a
/// random factor.
fn score_key<R: RngExt>(policy: Policy, expected: f64, rng: &mut R) -> (f64, f64) {
    match policy {
        Policy::WldScore => (0.0, expected),
        Policy::WldScoreJitter10 => (0.0, expected * (1.0 + rng.random_range(-0.1..=0.1))),
        Policy::WldScoreJitter20 => (0.0, expected * (1.0 + rng.random_range(-0.2..=0.2))),
        _ => unreachable!("not an expected-score policy: {policy:?}"),
    }
}

/// Index of the move with the largest key, ties broken at random. Moves are
/// compared by (tier, value), larger is better.
fn pick_best<R: RngExt>(keys: &[(f64, f64)], rng: &mut R) -> usize {
    let mut best_key = (f64::NEG_INFINITY, f64::NEG_INFINITY);
    let mut best = Vec::new();
    for (i, &key) in keys.iter().enumerate() {
        if key > best_key {
            best_key = key;
            best.clear();
        }
        if key == best_key {
            best.push(i);
        }
    }
    best[rng.random_range(0..best.len())]
}

/// ln P(win) - ln P(loss). Probabilities that underflowed to 0 are treated
/// as the smallest positive double, so the result is always finite.
pub fn log_odds(win: f64, loss: f64) -> f64 {
    win.max(f64::MIN_POSITIVE).ln() - loss.max(f64::MIN_POSITIVE).ln()
}

/// hybrid13: the largest softmax temperature that searches the root with cut windows (see `search`'s module doc). At
/// T 0.1 the windows cut 3-4% of the root moves and save nothing (the performance review of 11-11 ~06:50).
const WINDOW_T_MAX: f64 = 0.05;

/// hybrid13: whether a low-temperature log-odds softmax and the log-odds argmax search the root with cut windows: not
/// with CHESSAI_WINDOWS=off (hybrid12's code, for checks). Read once.
fn windows() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("CHESSAI_WINDOWS").map_or(true, |v| v != "off"))
}

/// hybrid13: the softmax window's depth below the best value, in temperatures, K: a move whose value is at least K T
/// below the best has a weight under e^-K, so it can't be picked and vanishes from the weights' sum unless the draw
/// lands within a few units of the last place of it, when it is searched again in full. CHESSAI_WINDOW_K changes it
/// (any K > 0 gives the same picks; for measurements). 40 was the fastest of 38-50 on W4 at plies 3 and 4 (11-11):
/// 1-3% of the picks then search some cut moves again. Read once.
fn window_k() -> f64 {
    static K: std::sync::OnceLock<f64> = std::sync::OnceLock::new();
    *K.get_or_init(|| {
        let k = std::env::var("CHESSAI_WINDOW_K").map_or(40.0, |v| v.parse::<f64>().expect("CHESSAI_WINDOW_K: a number"));
        assert!(k > 0.0, "CHESSAI_WINDOW_K must be positive");
        k
    })
}

/// hybrid13: CHESSAI_WINDOW_CHECK=1 also searches every root move in full and asserts the windowed values, bounds and
/// pick against the full search's (for checks; the games are the same). Read once.
fn window_check() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("CHESSAI_WINDOW_CHECK").map_or(false, |v| v == "1"))
}

/// hybrid13: CHESSAI_WINDOW_STATS=1 prints a WINDOW line to stderr for each windowed search (for measurements). Read
/// once.
fn window_stats() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("CHESSAI_WINDOW_STATS").map_or(false, |v| v == "1"))
}

/// hybrid13: `softmax_sample(&search.root_values(game, depth), temperature, rng)` from the root searched with cut
/// windows (`Search::root_values_windowed`, the window's bottom K T below the best value): the same index and the same
/// draw from `rng`.
fn windowed_softmax<R: RngExt>(search: &mut Search, game: &Game, depth: u32, temperature: f64, rng: &mut R) -> usize {
    let full = if window_check() { Some(search.root_values(game, depth)) } else { None };
    let delta = window_k() * temperature;
    let (mut values, mut pruned) = search.root_values_windowed(game, depth, |best| best - delta);
    let cut = pruned.iter().filter(|&&p| p).count();
    let mut searched = 0usize;
    let (index, sum, target, how) = softmax_sample_bounded(&mut values, &mut pruned, temperature, rng, &mut |i| {
        searched += 1;
        search.root_value_of(game, depth, i)
    });
    if window_stats() {
        eprintln!("WINDOW softmax depth {depth} moves {} cut {cut} searched-again {searched} decided {how}", values.len());
    }
    if let Some(full) = full {
        check_softmax(&full, &values, &pruned, temperature, index, sum, target);
    }
    index
}

/// hybrid13: `softmax_sample` over the root moves' `values`, some only upper bounds (`pruned`), each bound below the
/// largest exact value: the same index and the same draw from `rng` as `softmax_sample` over the exact values.
/// `exact(i)` gives move i's exact value; it is called only where the bounds don't decide the pick, and `values` and
/// `pruned` take what it gives. Returns the index, the draw's range (the weights' sum) and the draw, and how the
/// pick was decided: 0 by the bounds alone, 1 after exact values for some moves the walk reached, 2 after exact
/// values for every pruned move (the bounds left the sum undecided, or a value is not a finite number).
///
/// Why it is exact. The largest value is an exact one, so each exact move's weight is `softmax_sample`'s, bit for
/// bit (up to the sign of a zero, which no weight depends on). A pruned move's true weight lies in [0, hi], hi
/// twice its bound's weight (exp needn't be monotone) and at least the smallest normal number. Rounded addition and
/// subtraction are monotone in each operand, so (1) if the weights' sum in `softmax_sample`'s order is the same with
/// every pruned weight at 0 and at hi, it is that sum for any true weights, and the draw is the same; (2) at a pruned
/// move the walk reaches with target >= hi and target - hi == target, the true weight can't be picked and leaves the
/// target unchanged, as with every weight in [0, hi].
fn softmax_sample_bounded<R: RngExt>(
    values: &mut [f64],
    pruned: &mut [bool],
    temperature: f64,
    rng: &mut R,
    exact: &mut dyn FnMut(usize) -> f64,
) -> (usize, f64, f64, u8) {
    let n = values.len();
    let max = (0..n).filter(|&i| !pruned[i]).map(|i| values[i]).fold(f64::NEG_INFINITY, f64::max);
    let weight = |v: f64| ((v - max) / temperature).exp();
    let lo: Vec<f64> = (0..n).map(|i| if pruned[i] { 0.0 } else { weight(values[i]) }).collect();
    let hi: Vec<f64> =
        (0..n).map(|i| if pruned[i] { (2.0 * weight(values[i])).max(f64::MIN_POSITIVE) } else { lo[i] }).collect();
    let sum_lo = lo.iter().sum::<f64>();
    let sum_hi = hi.iter().sum::<f64>();
    let finite = max.is_finite() && values.iter().all(|v| v.is_finite()) && sum_lo.is_finite();
    if !finite || sum_lo.to_bits() != sum_hi.to_bits() {
        for i in 0..n {
            if pruned[i] {
                values[i] = exact(i);
                pruned[i] = false;
            }
        }
        let (index, sum, target) = softmax_sample_traced(values, temperature, rng);
        return (index, sum, target, 2);
    }
    let mut target = rng.random_range(0.0..sum_lo);
    let target0 = target;
    let mut how = 0;
    for i in 0..n {
        let w = if pruned[i] {
            if target >= hi[i] && (target - hi[i]).to_bits() == target.to_bits() {
                continue;
            }
            values[i] = exact(i);
            pruned[i] = false;
            // `softmax_sample` would have had a finite sum with it (the sum is decided): a finite value.
            assert!(values[i].is_finite(), "a root move's value {} after a finite bound", values[i]);
            how = 1;
            weight(values[i])
        } else {
            lo[i]
        };
        if target < w {
            return (i, sum_lo, target0, how);
        }
        target -= w;
    }
    (n - 1, sum_lo, target0, how)
}

/// `softmax_sample`, also returning the weights' sum and the draw (hybrid13): the same arithmetic.
fn softmax_sample_traced<R: RngExt>(values: &[f64], temperature: f64, rng: &mut R) -> (usize, f64, f64) {
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let weights: Vec<f64> = values.iter().map(|v| ((v - max) / temperature).exp()).collect();
    let sum = weights.iter().sum::<f64>();
    let mut target = rng.random_range(0.0..sum);
    let target0 = target;
    for (i, w) in weights.iter().enumerate() {
        if target < *w {
            return (i, sum, target0);
        }
        target -= w;
    }
    (weights.len() - 1, sum, target0)
}

/// hybrid13, CHESSAI_WINDOW_CHECK: the windowed values and bounds against the full search's `full`, and the pick
/// against `softmax_sample`'s over `full` for the same draw.
fn check_softmax(full: &[f64], values: &[f64], pruned: &[bool], temperature: f64, index: usize, sum: f64, target: f64) {
    let best = full.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    for i in 0..full.len() {
        if pruned[i] {
            assert!(full[i] <= values[i] && values[i] < best, "move {i}: bound {} for value {} (best {best})", values[i], full[i]);
        } else {
            assert!(
                values[i].to_bits() == full[i].to_bits() || (values[i] == 0.0 && full[i] == 0.0),
                "move {i}: windowed value {} against {}",
                values[i],
                full[i]
            );
        }
    }
    let weights: Vec<f64> = full.iter().map(|v| ((v - best) / temperature).exp()).collect();
    let s = weights.iter().sum::<f64>();
    assert_eq!(s.to_bits(), sum.to_bits(), "weights' sum {sum} against the full search's {s}");
    let mut t = target;
    let mut pick = weights.len() - 1;
    for (i, w) in weights.iter().enumerate() {
        if t < *w {
            pick = i;
            break;
        }
        t -= w;
    }
    assert_eq!(pick, index, "windowed pick {index} against the full search's {pick} (draw {target} of {sum})");
}

/// hybrid13: `Policy::LogOdds`'s pick from the root searched with cut windows (the window's bottom just below the
/// best value, so a move that ties with it is exact): the same tied moves, so the same index and the same draw.
fn windowed_argmax<R: RngExt>(search: &mut Search, game: &Game, depth: u32, rng: &mut R) -> usize {
    let full = if window_check() { Some(search.root_values(game, depth)) } else { None };
    let (values, pruned) = search.root_values_windowed(game, depth, f64::next_down);
    let n = values.len();
    // Every bound is below the largest exact value, so the best value and the moves that tie with it are exact; a
    // value that is not a number ties with nothing, as in `Policy::LogOdds`.
    let best = (0..n).filter(|&i| !pruned[i]).map(|i| values[i]).fold(f64::NEG_INFINITY, f64::max);
    let tied: Vec<usize> = (0..n).filter(|&i| !pruned[i] && values[i] == best).collect();
    if window_stats() {
        eprintln!("WINDOW argmax depth {depth} moves {n} cut {} tied {}", pruned.iter().filter(|&&p| p).count(), tied.len());
    }
    if let Some(full) = full {
        let fbest = full.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let ftied: Vec<usize> = (0..n).filter(|&i| full[i] == fbest).collect();
        assert_eq!(tied, ftied, "windowed ties against the full search's (values {values:?}, full {full:?})");
        for i in 0..n {
            if pruned[i] {
                assert!(full[i] <= values[i] && values[i] < fbest, "move {i}: bound {} for value {}", values[i], full[i]);
            }
        }
    }
    tied[rng.random_range(0..tied.len())]
}

/// Samples an index with probability proportional to exp(value / temperature).
fn softmax_sample<R: RngExt>(values: &[f64], temperature: f64, rng: &mut R) -> usize {
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    // Subtracting the maximum keeps exp() from overflowing at low temperatures.
    let weights: Vec<f64> = values.iter().map(|v| ((v - max) / temperature).exp()).collect();
    let mut target = rng.random_range(0.0..weights.iter().sum::<f64>());
    for (i, w) in weights.iter().enumerate() {
        if target < *w {
            return i;
        }
        target -= w;
    }
    weights.len() - 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding::CATEGORIES;
    use crate::logreg::Logreg;
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use shakmaty::EnPassantMode;
    use shakmaty::zobrist::Zobrist64;

    /// Whether the search takes the position after `m` as the end of the
    /// game: mate, stalemate, insufficient material, the 50-move rule, or a
    /// repetition of a position since the last irreversible move.
    fn ends_search(game: &Game, m: Move) -> bool {
        let pos = game.position();
        let mut next = pos.clone();
        next.play_unchecked(m);
        let repeats =
            !pos.is_irreversible(m) && game.history().contains(&next.zobrist_hash::<Zobrist64>(EnPassantMode::Legal));
        next.legal_moves().is_empty() || next.is_insufficient_material() || next.halfmoves() >= 100 || repeats
    }

    #[test]
    fn jitter_at_depth_1_chooses_as_at_depth_0_unless_a_move_ends_the_game() {
        // Random weights, so that moves score differently.
        let mut model = Logreg::new(3, &CATEGORIES, 0.1);
        model.randomize(7, 0.3);
        let depth0 = Player::new(&model, Policy::WldScoreJitter10, 0.0);
        let depth1 = Player { depth: 1, ..depth0 };
        let exact = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        let (mut same, mut ending) = (0, 0);
        for g in 0..10 {
            // The positions of a depth-0 jitter game, each chosen from by
            // both players with the same random numbers.
            let record = crate::selfplay::play_game(depth0, depth0, g);
            let mut game = Game::default();
            for (ply, &m) in record.moves.iter().enumerate() {
                let pos = game.position();
                let candidates = candidates(pos);
                let seed = g * 10_000 + ply as u64;
                let (mut rng0, mut rng1) = (StdRng::seed_from_u64(seed), StdRng::seed_from_u64(seed));
                let chosen0 = select(depth0, pos.turn(), &candidates, &mut rng0);
                let chosen1 = choose(depth1, &game, &candidates, &mut rng1);

                // Each move's outcome is the result if it ends the game,
                // else the model's prediction for the position after it.
                let outcomes = Search::new(&model, &game).root_outcomes(&game, 1);
                let mut ends = false;
                for (c, outcome) in candidates.iter().zip(&outcomes) {
                    if ends_search(&game, c.mv) {
                        ends = true;
                        assert!(exact.contains(outcome), "{outcome:?}");
                    } else {
                        let mut predicted = [0.0; 3];
                        model.predict(&c.position, &mut predicted);
                        assert_eq!(*outcome, predicted);
                    }
                }
                if ends {
                    ending += 1;
                } else {
                    // The same move, after the same random draws.
                    assert_eq!(chosen0, chosen1, "game {g} ply {ply}");
                    assert_eq!(rng0.random::<u64>(), rng1.random::<u64>(), "game {g} ply {ply}");
                    same += 1;
                }
                game.play(m);
            }
        }
        assert!(same > 1000 && ending > 10, "{same} positions without a game-ending move, {ending} with");
    }

    #[test]
    fn softmax_follows_temperature() {
        let mut rng = StdRng::seed_from_u64(1);
        let mut count = |t: f64| {
            let mut picks = [0; 2];
            for _ in 0..10_000 {
                picks[softmax_sample(&[0.5, 0.5 + t * 2f64.ln()], t, &mut rng)] += 1;
            }
            picks
        };
        // A gap of T*ln(2) makes the better move twice as likely, at any T.
        for t in [0.001, 0.1] {
            let [worse, better] = count(t);
            assert!((better as f64 / worse as f64 - 2.0).abs() < 0.15, "{worse} {better}");
        }
        // Far below the gap, the best move is always chosen.
        assert_eq!(softmax_sample(&[0.4, 0.6], 1e-6, &mut rng), 1);
    }

    #[test]
    fn bounded_softmax_picks_as_softmax_sample() {
        // hybrid13: `softmax_sample_bounded` against `softmax_sample` with the same generator: the same index and the
        // same generator state after, over adversarial root values (near ties, values on and around the window's
        // bottom, mates, zeros of both signs, duplicates of the best) cut by a simulated windowed search in a random
        // order with bounds anywhere in [value, bottom], at temperatures 1e-6 to 0.05 and window depths K 1 to 100.
        let mut rg = StdRng::seed_from_u64(77);
        let mut how = [0u64; 3];
        let mut cut_moves = 0u64;
        for case in 0..300_000u64 {
            let n = rg.random_range(1..45usize);
            let t = [0.02, 0.05, 0.001, 1e-6, 0.03, 0.02][(case % 6) as usize];
            let k = [45.0, 40.0, 37.0, 50.0, 1.0, 10.0, 100.0, 45.0][(case % 8) as usize];
            let delta = k * t;
            let top = rg.random_range(-3.0..3.0);
            let values: Vec<f64> = (0..n)
                .map(|_| match rg.random_range(0..10) {
                    0 => top,
                    1 => top - delta,
                    2 => top - delta * (1.0 + rg.random_range(-1e-9..1e-9)),
                    3 => top - rg.random_range(0.0..3.0 * delta),
                    4 => top - rg.random_range(0.0..0.2 * delta),
                    5 => -(1e6 - rg.random_range(1..9) as f64),
                    6 => if rg.random_range(0..2) == 0 { 0.0 } else { -0.0 },
                    7 => top - delta * rg.random_range(1.0..1.2),
                    8 => top - 1e-12 * rg.random_range(0.0..10.0),
                    _ => top - rg.random_range(0.0..20.0) * t,
                })
                .collect();
            // A windowed search in a random order: the first exact, then exact above (best - delta), else a bound.
            let mut order: Vec<usize> = (0..n).collect();
            for i in (1..n).rev() {
                order.swap(i, rg.random_range(0..=i));
            }
            let mut bounded = values.clone();
            let mut pruned = vec![false; n];
            let mut best = f64::NEG_INFINITY;
            for &i in &order {
                let alpha = if best == f64::NEG_INFINITY { f64::NEG_INFINITY } else { best - delta };
                if values[i] > alpha {
                    best = best.max(values[i]);
                } else {
                    pruned[i] = true;
                    cut_moves += 1;
                    bounded[i] = match rg.random_range(0..3) {
                        0 => values[i],
                        1 => alpha,
                        _ => values[i] + (alpha - values[i]) * rg.random_range(0.0..1.0),
                    };
                    assert!(values[i] <= bounded[i] && bounded[i] <= alpha);
                }
            }
            let seed = rg.random::<u64>();
            let mut full_rng = StdRng::seed_from_u64(seed);
            let want = softmax_sample(&values, t, &mut full_rng);
            let mut rng = StdRng::seed_from_u64(seed);
            let mut asked = Vec::new();
            let (got, _, _, h) = softmax_sample_bounded(&mut bounded, &mut pruned, t, &mut rng, &mut |i| {
                asked.push(i);
                values[i]
            });
            assert_eq!(got, want, "case {case}: values {values:?} t {t} k {k}");
            assert_eq!(rng.random::<u64>(), full_rng.random::<u64>(), "case {case}: the generator's state");
            how[h as usize] += 1;
        }
        assert!(how[0] > 100_000 && how[1] > 1_000 && how[2] > 1_000, "{how:?}");
        eprintln!("bounded_softmax_picks_as_softmax_sample: decided by the bounds {}, after some exact values {}, after all {}; {cut_moves} moves cut", how[0], how[1], how[2]);
    }

    #[test]
    fn windowed_picks_are_the_full_searchs() {
        // hybrid13: `windowed_softmax` and `windowed_argmax` against the full search's picks with the same generator,
        // from random-game positions, at depths 2-4, on the fixed-point and float paths, at T 0.02, 0.05 and 0.001.
        use crate::fixed::FxLinear;
        let mut lr = Logreg::new(3, &CATEGORIES, 0.1);
        lr.randomize(9, 1.0);
        let fx: FxLinear = lr.fixed().unwrap();
        let mut rg = StdRng::seed_from_u64(5);
        let mut cases = 0;
        for g in 0..60 {
            let mut game = Game::default();
            for _ in 0..rg.random_range(0..80) {
                let moves = game.position().legal_moves();
                if moves.is_empty() {
                    break;
                }
                game.play(moves[rg.random_range(0..moves.len())]);
            }
            if game.position().legal_moves().is_empty() {
                continue;
            }
            for depth in 2..=(if g % 5 == 0 { 4 } else { 3 }) {
                for float in [false, true] {
                    let search = || {
                        let s = Search::new(&lr, &game);
                        if float { s } else { s.with_fx(Some(&fx)) }
                    };
                    for t in [0.02, 0.05, 0.001] {
                        for rep in 0..3u64 {
                            let seed = (g * 1000 + depth as u64 * 10 + rep) ^ 0x5eed;
                            let mut full_rng = StdRng::seed_from_u64(seed);
                            let want = softmax_sample(&search().root_values(&game, depth), t, &mut full_rng);
                            let mut rng = StdRng::seed_from_u64(seed);
                            let got = windowed_softmax(&mut search(), &game, depth, t, &mut rng);
                            assert_eq!(got, want, "game {g}, depth {depth}, float {float}, t {t}");
                            assert_eq!(rng.random::<u64>(), full_rng.random::<u64>());
                            cases += 1;
                        }
                    }
                    let seed = g ^ 0xa11;
                    let mut full_rng = StdRng::seed_from_u64(seed);
                    let values = search().root_values(&game, depth);
                    let best = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                    let tied: Vec<usize> = (0..values.len()).filter(|&i| values[i] == best).collect();
                    let want = tied[full_rng.random_range(0..tied.len())];
                    let mut rng = StdRng::seed_from_u64(seed);
                    let got = windowed_argmax(&mut search(), &game, depth, &mut rng);
                    assert_eq!(got, want, "argmax: game {g}, depth {depth}, float {float}");
                    assert_eq!(rng.random::<u64>(), full_rng.random::<u64>());
                    cases += 1;
                }
            }
        }
        assert!(cases > 500, "{cases}");
        eprintln!("windowed_picks_are_the_full_searchs: {cases} cases");
    }
}
