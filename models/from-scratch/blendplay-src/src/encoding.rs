//! The 70-feature categorical position encoding, matching
//! `py_src/chess/categorical_input.py`.
//!
//! | index  | meaning                                   | values |
//! |--------|-------------------------------------------|--------|
//! | 0..64  | squares a1, b1, ..., h1, a2, ..., h8       | 0 empty, 1-6 white P N B R Q K, 7-12 black |
//! | 64     | side to move                              | 0 white, 1 black |
//! | 65     | file of a pawn that just double-pushed    | 0 none, 1-8 a-h |
//! | 66..70 | castling: white long, black long, white short, black short | 0/1 |
//!
//! One deliberate difference from the Python version: castling features are
//! the real castling rights. Python only tracks whether the king and rook
//! have moved, so it keeps the flag set after the rook is captured at home.

use shakmaty::{CastlingSide, Color, EnPassantMode, Position, Role};

pub const FEATURES: usize = 70;

/// Number of values each feature can take.
pub const CATEGORIES: [u8; FEATURES] = {
    let mut cats = [13u8; FEATURES];
    cats[64] = 2;
    cats[65] = 9;
    cats[66] = 2;
    cats[67] = 2;
    cats[68] = 2;
    cats[69] = 2;
    cats
};

pub type Encoded = [u8; FEATURES];

pub fn encode<P: Position>(pos: &P) -> Encoded {
    let mut enc = [0u8; FEATURES];

    for (square, piece) in pos.board().iter() {
        let role = match piece.role {
            Role::Pawn => 1,
            Role::Knight => 2,
            Role::Bishop => 3,
            Role::Rook => 4,
            Role::Queen => 5,
            Role::King => 6,
        };
        enc[square as usize] = role + if piece.color == Color::Black { 6 } else { 0 };
    }

    enc[64] = if pos.turn() == Color::White { 0 } else { 1 };
    // Python records the file after any double push, whether or not a
    // capture is possible, which is what EnPassantMode::Always gives.
    enc[65] = pos
        .ep_square(EnPassantMode::Always)
        .map_or(0, |sq| sq.file() as u8 + 1);

    let castles = pos.castles();
    enc[66] = castles.has(Color::White, CastlingSide::QueenSide) as u8;
    enc[67] = castles.has(Color::Black, CastlingSide::QueenSide) as u8;
    enc[68] = castles.has(Color::White, CastlingSide::KingSide) as u8;
    enc[69] = castles.has(Color::Black, CastlingSide::KingSide) as u8;

    enc
}

#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::Chess;

    #[test]
    fn start_position() {
        let enc = encode(&Chess::default());
        assert_eq!(enc[..8], [4, 2, 3, 5, 6, 3, 2, 4]);
        assert_eq!(enc[8..16], [1; 8]);
        assert_eq!(enc[16..48], [0; 32]);
        assert_eq!(enc[48..56], [7; 8]);
        assert_eq!(enc[56..64], [10, 8, 9, 11, 12, 9, 8, 10]);
        assert_eq!(enc[64..], [0, 0, 1, 1, 1, 1]);
    }

    #[test]
    fn values_within_categories() {
        let enc = encode(&Chess::default());
        assert!(enc.iter().zip(CATEGORIES).all(|(&v, c)| v < c));
    }
}
