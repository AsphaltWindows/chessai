//! Exact fast paths (models/from-scratch/movegen, Q9): whether the side to move has a legal move, decided without
//! generating every legal move. The search asks this at every leaf only to tell mates and stalemates from playable
//! positions, and nearly always some piece can move. The answer is always the same as
//! `!pos.legal_moves().is_empty()`.

use shakmaty::{Bitboard, Chess, Color, Position, attacks};

/// Whether the side to move has a legal move (the same answer as `!pos.legal_moves().is_empty()`).
pub fn has_legal_move(pos: &Chess) -> bool {
    match quick_has_move(pos) {
        Some(answer) => answer,
        None => !pos.legal_moves().is_empty(),
    }
}

/// `Some(true)` when a move is certainly legal: not in check, an unpinned knight, pawn (single push or capture) or
/// slider with a move, or (in check or not) a king step to a square no enemy piece attacks once the king has left
/// its own. `None` when it can't tell (castling, en passant, pinned pieces moving along the pin, interpositions and
/// captures of a checker are left to the full generation).
fn quick_has_move(pos: &Chess) -> Option<bool> {
    let board = pos.board();
    let turn = pos.turn();
    let king = board.king_of(turn)?;
    let us = board.by_color(turn);
    let them = board.by_color(!turn);
    let occ = board.occupied();
    let king_can_step = || {
        let without_king = occ ^ Bitboard::from_square(king);
        (attacks::king_attacks(king) & !us).into_iter().any(|t| board.attacks_to(t, !turn, without_king).is_empty())
    };
    if pos.is_check() {
        return if king_can_step() { Some(true) } else { None };
    }
    let snipers = (attacks::rook_attacks(king, Bitboard::EMPTY) & board.rooks_and_queens() & them)
        | (attacks::bishop_attacks(king, Bitboard::EMPTY) & board.bishops_and_queens() & them);
    let mut pinned = Bitboard::EMPTY;
    for s in snipers {
        let between = attacks::between(king, s) & occ;
        if between.count() == 1 {
            pinned |= between & us;
        }
    }
    let free = us & !pinned & !board.kings();
    let not_ours = !us;
    for sq in free & board.knights() {
        if (attacks::knight_attacks(sq) & not_ours).any() {
            return Some(true);
        }
    }
    let push = if turn == Color::White { 8 } else { -8 };
    for sq in free & board.pawns() {
        if let Some(t) = sq.offset(push) {
            if !occ.contains(t) {
                return Some(true);
            }
        }
        if (attacks::pawn_attacks(turn, sq) & them).any() {
            return Some(true);
        }
    }
    for sq in free & board.bishops_and_queens() {
        if (attacks::bishop_attacks(sq, occ) & not_ours).any() {
            return Some(true);
        }
    }
    for sq in free & board.rooks_and_queens() {
        if (attacks::rook_attacks(sq, occ) & not_ours).any() {
            return Some(true);
        }
    }
    if king_can_step() {
        return Some(true);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding::{encode, encode_board_scan};
    use shakmaty::fen::Fen;
    use shakmaty::{CastlingMode, Position};

    fn position(fen: &str) -> Chess {
        fen.parse::<Fen>().unwrap().into_position(CastlingMode::Standard).unwrap()
    }

    #[test]
    fn mates_stalemates_and_forced_cases() {
        for (fen, has) in [
            ("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1", false),                         // stalemate
            ("6k1/5ppp/8/8/8/8/8/4R1K1 w - - 0 1", true),                      // a mate in one to play
            ("4R1k1/5ppp/8/8/8/8/8/6K1 b - - 0 1", false),                     // back-rank mate
            ("rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3", false), // fool's mate
            ("8/8/8/8/8/2k5/1q6/K7 w - - 0 1", false),                         // mated in the corner
            ("8/8/8/8/8/1qk5/8/K7 w - - 0 1", false),                          // stalemated in the corner
            ("3rk3/8/8/8/8/8/3r4/r3K3 w - - 0 1", false),                      // checked, every square covered
            ("4k3/8/8/8/8/8/4r3/4K2R w K - 0 1", true),                        // checked, can step aside
            ("4k3/8/8/8/1b6/8/3P4/4K3 w - - 0 1", true),                       // the pinned pawn can't, the king can
            ("4k3/4r3/8/8/8/8/4N3/4K3 w - - 0 1", true),                       // pinned knight; the king moves
            ("k7/2Q5/1K6/8/8/8/8/8 b - - 0 1", false),                         // stalemate by the queen
            ("4k3/8/8/2pP4/8/8/8/2K5 w - c6 0 2", true),                       // en passant
            ("8/8/8/1k6/3Pp3/8/8/4K2R b K d3 0 1", true),                      // en passant for black
        ] {
            let pos = position(fen);
            assert_eq!(!pos.legal_moves().is_empty(), has, "legal moves of {fen}");
            assert_eq!(has_legal_move(&pos), has, "has_legal_move on {fen}");
        }
    }

    /// Random games (a fixed xorshift sequence): every position's answer and encoding match the originals.
    #[test]
    fn random_games_agree_with_full_generation() {
        let mut x = 0x9e37_79b9_7f4a_7c15u64;
        let mut positions = 0;
        for _ in 0..300 {
            let mut pos = Chess::default();
            for _ in 0..400 {
                assert_eq!(has_legal_move(&pos), !pos.legal_moves().is_empty());
                assert_eq!(encode(&pos), encode_board_scan(&pos));
                positions += 1;
                let moves = pos.legal_moves();
                if moves.is_empty() || pos.is_insufficient_material() {
                    break;
                }
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                pos.play_unchecked(moves[(x % moves.len() as u64) as usize]);
            }
        }
        assert!(positions > 30_000);
    }
}
