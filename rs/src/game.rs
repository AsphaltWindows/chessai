//! A game in progress: position plus the history needed for draw rules
//! (threefold repetition, 50-move rule) that shakmaty does not track.

use shakmaty::zobrist::Zobrist64;
use shakmaty::{Chess, Color, EnPassantMode, KnownOutcome, Move, Position};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameResult {
    WhiteWin,
    BlackWin,
    Draw,
}

impl GameResult {
    pub fn pgn(self) -> &'static str {
        match self {
            GameResult::WhiteWin => "1-0",
            GameResult::BlackWin => "0-1",
            GameResult::Draw => "1/2-1/2",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Game {
    pos: Chess,
    hash: Zobrist64,
    /// Hashes of every position reached since the last irreversible move,
    /// including the current one.
    history: Vec<Zobrist64>,
}

impl Default for Game {
    fn default() -> Self {
        Game::new(Chess::default())
    }
}

impl Game {
    pub fn new(pos: Chess) -> Self {
        let hash = pos.zobrist_hash(EnPassantMode::Legal);
        Game { pos, hash, history: vec![hash] }
    }

    pub fn position(&self) -> &Chess {
        &self.pos
    }

    pub fn play(&mut self, m: Move) {
        let irreversible = self.pos.is_irreversible(m);
        let updated = self.pos.update_zobrist_hash(self.hash, m, EnPassantMode::Legal);
        self.pos.play_unchecked(m);
        self.hash = updated.unwrap_or_else(|| self.pos.zobrist_hash(EnPassantMode::Legal));

        if irreversible {
            self.history.clear();
        }
        self.history.push(self.hash);
    }

    fn repetitions(&self) -> usize {
        self.history.iter().filter(|&&h| h == self.hash).count()
    }

    /// The result if the game is over: checkmate, stalemate, insufficient
    /// material, 50-move rule or threefold repetition.
    pub fn result(&self) -> Option<GameResult> {
        if let Some(outcome) = self.pos.outcome().known() {
            return Some(match outcome {
                KnownOutcome::Decisive { winner: Color::White } => GameResult::WhiteWin,
                KnownOutcome::Decisive { winner: Color::Black } => GameResult::BlackWin,
                KnownOutcome::Draw => GameResult::Draw,
            });
        }
        if self.pos.halfmoves() >= 100 || self.repetitions() >= 3 {
            return Some(GameResult::Draw);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::uci::UciMove;

    fn play(game: &mut Game, uci: &str) {
        let m = uci.parse::<UciMove>().unwrap().to_move(game.position()).unwrap();
        game.play(m);
    }

    #[test]
    fn threefold_repetition_counts_start_position() {
        let mut game = Game::default();
        for _ in 0..2 {
            assert_eq!(game.result(), None);
            for m in ["g1f3", "g8f6", "f3g1", "f6g8"] {
                play(&mut game, m);
            }
        }
        assert_eq!(game.result(), Some(GameResult::Draw));
    }

    #[test]
    fn fools_mate() {
        let mut game = Game::default();
        for m in ["f2f3", "e7e5", "g2g4", "d8h4"] {
            play(&mut game, m);
        }
        assert_eq!(game.result(), Some(GameResult::BlackWin));
    }
}
