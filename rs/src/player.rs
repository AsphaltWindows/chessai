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
use crate::encoding::{self, Encoded};
use crate::game::Game;
use crate::calibration::Calibration;
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
    /// log-odds policy.
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

/// Index into `candidates`, the legal moves of the game's position in
/// `legal_moves()` order, of the move to play.
pub fn choose<R: RngExt>(player: Player, game: &Game, candidates: &[Candidate], rng: &mut R) -> usize {
    let pos = game.position();
    if player.depth == 0 || player.policy == Policy::Rand {
        return select(player, pos.turn(), candidates, rng);
    }
    let values =
        Search::new(player.model, game).valuing(player.value, player.calibration).root_values(game, player.depth);
    match player.policy {
        Policy::LogOddsSoftmax => softmax_sample(&values, player.temperature, rng),
        Policy::LogOdds => {
            let best = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let tied: Vec<usize> = (0..values.len()).filter(|&i| values[i] == best).collect();
            tied[rng.random_range(0..tied.len())]
        }
        _ => panic!("search needs the log-odds or log-odds-softmax policy, not {:?}", player.policy),
    }
}

/// Index into `candidates` of the move to play.
pub fn select<R: RngExt>(
    player: Player,
    turn: Color,
    candidates: &[Candidate],
    rng: &mut R,
) -> usize {
    let Player { model, policy, temperature, .. } = player;
    let predict = |c: &Candidate, out: &mut [f64]| model.predict(&c.position, out);
    if policy == Policy::Rand {
        return rng.random_range(0..candidates.len());
    }

    let (win, loss) = match turn {
        Color::White => (0, 1),
        Color::Black => (1, 0),
    };
    let mut scores = [0.0; 3];

    if matches!(policy, Policy::WldScoreSoftmax | Policy::LogOddsSoftmax) {
        let values: Vec<f64> = candidates
            .iter()
            .map(|c| {
                predict(c, &mut scores);
                if policy == Policy::LogOddsSoftmax {
                    log_odds(scores[win], scores[loss])
                } else {
                    scores[win] + scores[2] / 2.0
                }
            })
            .collect();
        return softmax_sample(&values, temperature, rng);
    }

    let mut best_key = (f64::NEG_INFINITY, f64::NEG_INFINITY);
    let mut best = Vec::new();

    for (i, c) in candidates.iter().enumerate() {
        predict(c, &mut scores);
        let expected = scores[win] + scores[2] / 2.0;
        // Moves are compared by (tier, value), larger is better.
        let key = match policy {
            Policy::WldScore => (0.0, expected),
            Policy::WldScoreJitter10 => (0.0, expected * (1.0 + rng.random_range(-0.1..=0.1))),
            Policy::WldScoreJitter20 => (0.0, expected * (1.0 + rng.random_range(-0.2..=0.2))),
            Policy::LogOdds => (0.0, log_odds(scores[win], scores[loss])),
            Policy::WldWd if scores[win] > scores[loss] => (1.0, scores[win]),
            Policy::WldWd => (0.0, scores[2]),
            Policy::Rand | Policy::WldScoreSoftmax | Policy::LogOddsSoftmax => unreachable!(),
        };

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
    use rand::SeedableRng;
    use rand::rngs::StdRng;

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
}
