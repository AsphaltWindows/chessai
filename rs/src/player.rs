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

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Policy {
    /// Expected score: win + draw / 2.
    WldScore,
    /// Expected score multiplied by a random factor in [0.9, 1.1].
    WldScoreJitter10,
    /// Expected score multiplied by a random factor in [0.8, 1.2].
    WldScoreJitter20,
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

/// Index into `candidates` of the move to play.
pub fn select<R: RngExt>(
    player: Player,
    turn: Color,
    candidates: &[Candidate],
    rng: &mut R,
) -> usize {
    let Player { model, policy } = player;
    if policy == Policy::Rand {
        return rng.random_range(0..candidates.len());
    }

    let (win, loss) = match turn {
        Color::White => (0, 1),
        Color::Black => (1, 0),
    };
    let mut scores = [0.0; 3];
    let mut best_key = (f64::NEG_INFINITY, f64::NEG_INFINITY);
    let mut best = Vec::new();

    for (i, c) in candidates.iter().enumerate() {
        model.predict(&c.position, &mut scores);
        let expected = scores[win] + scores[2] / 2.0;
        // Moves are compared by (tier, value), larger is better.
        let key = match policy {
            Policy::WldScore => (0.0, expected),
            Policy::WldScoreJitter10 => (0.0, expected * (1.0 + rng.random_range(-0.1..=0.1))),
            Policy::WldScoreJitter20 => (0.0, expected * (1.0 + rng.random_range(-0.2..=0.2))),
            Policy::WldWd if scores[win] > scores[loss] => (1.0, scores[win]),
            Policy::WldWd => (0.0, scores[2]),
            Policy::Rand => unreachable!(),
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
