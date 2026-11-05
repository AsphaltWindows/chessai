//! Sanity checks on shakmaty before we build on it: move generation
//! correctness (perft) and incremental Zobrist hashing.

use shakmaty::fen::Fen;
use shakmaty::zobrist::Zobrist64;
use shakmaty::{CastlingMode, Chess, EnPassantMode, Position, perft};

fn position(fen: &str) -> Chess {
    fen.parse::<Fen>()
        .unwrap()
        .into_position(CastlingMode::Standard)
        .unwrap()
}

/// Standard perft suite (https://www.chessprogramming.org/Perft_Results).
#[test]
fn perft_suite() {
    let cases = [
        ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 5, 4_865_609),
        ("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 4, 4_085_603),
        ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 6, 11_030_083),
        ("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1", 5, 15_833_292),
        ("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 4, 2_103_487),
        ("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10", 4, 3_894_594),
    ];

    for (fen, depth, nodes) in cases {
        assert_eq!(perft(&position(fen), depth), nodes, "perft({depth}) of {fen}");
    }
}

/// Small deterministic RNG so the test needs no extra dependencies.
struct XorShift(u64);

impl XorShift {
    fn next(&mut self, n: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % n as u64) as usize
    }
}

/// Incremental hash updates must match a full recomputation on every move
/// of many random games (covers castling, en passant and promotions).
#[test]
fn incremental_zobrist_matches_full() {
    let mut rng = XorShift(0x9E37_79B9_7F4A_7C15);
    let (mut checked, mut fallbacks) = (0, 0);

    for _ in 0..500 {
        let mut pos = Chess::default();
        let mut hash: Zobrist64 = pos.zobrist_hash(EnPassantMode::Legal);

        while !pos.is_game_over() && pos.halfmoves() < 100 {
            let moves = pos.legal_moves();
            let m = moves[rng.next(moves.len())];

            let incremental = pos.update_zobrist_hash(hash, m, EnPassantMode::Legal);
            pos.play_unchecked(m);
            let full: Zobrist64 = pos.zobrist_hash(EnPassantMode::Legal);

            match incremental {
                Some(h) => {
                    assert_eq!(h, full, "after {m:?}");
                    checked += 1;
                }
                None => fallbacks += 1,
            }
            hash = full;
        }
    }

    println!("incremental updates checked: {checked}, fell back to full recompute: {fallbacks}");
    assert!(checked > 0, "no incremental updates were exercised");
}
