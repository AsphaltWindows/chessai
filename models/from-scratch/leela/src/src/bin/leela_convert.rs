//! leela_convert (the Leela-data experiment, 2026-11-13): Lc0 V6 training records (input format 1, classical) to
//! the chessai records the `train --records` mode reads.
//!
//! `leela_convert convert OUT.part STATS.json < stream`: the stream is what extract.py writes, per game a u32 record
//! count then that many 916-byte records (bytes 7440..8356 of each 8356-byte V6 record: the 104 history planes, then
//! castling, side to move, rule 50, invariance info, result, the Q/D/M floats, visits, played and best move indices).
//! Each record's position is rebuilt from history slot 0 (planes 0-11: the side to move's P N B R Q K, then the
//! opponent's; side-to-move relative, so ranks are flipped when black is to move; within a byte the files run h..a),
//! the castling bytes and the side-to-move byte. CHECKS: consecutive records of a game must be one legal move apart (our
//! move generator: shakmaty, perft-verified), and then the position IS the previous one with that move played (so the en
//! passant square and the castling rights are exactly what our self-play would have; the record's castling bytes are
//! compared with them); record i+1's history slot 1 must be record i's board. A game's first record (or one after a
//! broken link) is set up from the record, its last move from history slot 1 when there is one (for a double push's
//! en passant file, feature 65). FILTER: records the rescorer marked for deletion (invariance bit 6); then as Stockfish's
//! trainer: the side to move in check, or the best move (best_idx through Lc0's policy index) a capture or a promotion.
//! LABEL: Leela's best_q / best_d as a WDL for the side to move, W = (1 - d + q) / 2, L = (1 - d - q) / 2, D = d, put in
//! chessai's absolute order [white wins, black wins, draw]; the game's result (result_q / result_d) the same way, kept
//! beside it. OUT: 98-byte records: the 70 features, the Leela label (3 x f32), the result label (3 x f32), the game's
//! index in this part (u32), little-endian.
//!
//! `leela_convert chunk OUTDIR CHUNK PART...`: the parts' games in order into OUTDIR/round{r}.rec, whole games, a round
//! closing once it holds at least CHUNK records; the game field becomes the game's index in its round.
//!
//! `leela_convert take OUT.rec N PART...`: whole games from the parts in order until at least N records (held-out set).

use std::io::{self, BufReader, BufWriter, Read, Write};

use chessai::encoding;
use shakmaty::{Bitboard, Board, CastlingMode, Chess, Color, EnPassantMode, File, FromSetup, Move, Piece, Position, Rank, Role, Setup, Square};

const IN_REC: usize = 916; // bytes 7440..8356 of a V6 record
const OUT_REC: usize = 98;
const ROLES: [Role; 6] = [Role::Pawn, Role::Knight, Role::Bishop, Role::Rook, Role::Queen, Role::King];

struct Raw {
    planes: [u64; 104],
    castling: [u8; 4], // us ooo, us oo, them ooo, them oo
    stm: u8,
    inv: u8,
    best_q: f32,
    best_d: f32,
    result_q: f32,
    result_d: f32,
    best_idx: u16,
}

fn f32_at(b: &[u8], o: usize) -> f32 {
    f32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}

fn parse(b: &[u8]) -> Raw {
    let mut planes = [0u64; 104];
    for (i, p) in planes.iter_mut().enumerate() {
        *p = u64::from_le_bytes(b[i * 8..i * 8 + 8].try_into().unwrap());
    }
    // offsets relative to 7440: castling 832..836, stm 836, rule50 837, inv 838, result 839, root_q 840, best_q 844,
    // root_d 848, best_d 852, root_m 856, best_m 860, plies_left 864, result_q 868, result_d 872, played_q 876 ..,
    // visits 900, played_idx 904, best_idx 906.
    Raw {
        planes,
        castling: [b[832], b[833], b[834], b[835]],
        stm: b[836],
        inv: b[838],
        best_q: f32_at(b, 844),
        best_d: f32_at(b, 852),
        result_q: f32_at(b, 868),
        result_d: f32_at(b, 872),
        best_idx: u16::from_le_bytes([b[906], b[907]]),
    }
}

/// History slot `h`'s board in absolute colours (`stm` the record's side to move).
fn slot_board(raw: &Raw, h: usize, stm: Color) -> Board {
    let mut board = Board::empty();
    for p in 0..12 {
        let bb = raw.planes[h * 13 + p];
        let (color, role) = if p < 6 { (stm, ROLES[p]) } else { (!stm, ROLES[p - 6]) };
        let mut bits = bb;
        while bits != 0 {
            let bit = bits.trailing_zeros();
            bits &= bits - 1;
            let file = 7 - (bit % 8);
            let rank = if stm == Color::White { bit / 8 } else { 7 - bit / 8 };
            board.set_piece_at(Square::from_coords(File::new(file), Rank::new(rank)), Piece { color, role });
        }
    }
    board
}

fn slot_empty(raw: &Raw, h: usize) -> bool {
    raw.planes[h * 13..h * 13 + 12].iter().all(|&p| p == 0)
}

/// The record's castling rights as rook squares.
fn castling(raw: &Raw, stm: Color) -> Bitboard {
    let (us_long, us_short, them_long, them_short) = match stm {
        Color::White => (Square::A1, Square::H1, Square::A8, Square::H8),
        Color::Black => (Square::A8, Square::H8, Square::A1, Square::H1),
    };
    let mut bb = Bitboard::EMPTY;
    for (flag, sq) in raw.castling.iter().zip([us_long, us_short, them_long, them_short]) {
        if *flag != 0 {
            bb |= Bitboard::from(sq);
        }
    }
    bb
}

/// Rights a board allows (king and rook at home), for the position before a record's last move.
fn plausible_rights(board: &Board) -> Bitboard {
    let mut bb = Bitboard::EMPTY;
    for (color, king, rooks) in [(Color::White, Square::E1, [Square::A1, Square::H1]), (Color::Black, Square::E8, [Square::A8, Square::H8])] {
        if board.piece_at(king) == Some(Piece { color, role: Role::King }) {
            for r in rooks {
                if board.piece_at(r) == Some(Piece { color, role: Role::Rook }) {
                    bb |= Bitboard::from(r);
                }
            }
        }
    }
    bb
}

fn position(board: Board, turn: Color, rights: Bitboard, ep: Option<Square>) -> Option<Chess> {
    let setup = Setup { board, turn, castling_rights: rights, ep_square: ep, ..Setup::empty() };
    match Chess::from_setup(setup.clone(), CastlingMode::Standard).or_else(|e| e.ignore_invalid_ep_square()) {
        Ok(p) => Some(p),
        Err(e) => {
            if std::env::var_os("LEELA_DEBUG").is_some() {
                eprintln!("invalid: {} ({e})", shakmaty::fen::Fen::try_from_setup(setup).unwrap_or_else(|e| e.ignore()));
            }
            None
        }
    }
}

/// The legal move of `pos` that gives `board` (and the other side to move), with the position it leads to.
fn move_to(pos: &Chess, board: &Board) -> Option<(Move, Chess)> {
    pos.legal_moves().into_iter().find_map(|m| {
        let mut p = pos.clone();
        p.play_unchecked(m);
        (p.board() == board).then_some((m, p))
    })
}

/// The double push's skipped square, the file feature 65 records.
fn double_push(m: Move, turn: Color) -> Option<Square> {
    match m {
        Move::Normal { role: Role::Pawn, from, to, .. } if from.rank().distance(to.rank()) == 2 => {
            let r = if turn == Color::White { Rank::Third } else { Rank::Sixth };
            Some(Square::from_coords(from.file(), r))
        }
        _ => None,
    }
}

fn sq(name: &[u8]) -> Square {
    Square::from_coords(File::new((name[0] - b'a') as u32), Rank::new((name[1] - b'1') as u32))
}

/// The best move (Lc0 policy index, side-to-move relative: ranks flipped for black) among `pos`'s legal moves.
fn best_move(pos: &Chess, uci: &str, turn: Color) -> Option<Move> {
    let b = uci.as_bytes();
    let (mut from, mut to) = (sq(&b[0..2]), sq(&b[2..4]));
    if turn == Color::Black {
        from = from.flip_vertical();
        to = to.flip_vertical();
    }
    let promo = match b.get(4) {
        Some(b'q') => Some(Role::Queen),
        Some(b'r') => Some(Role::Rook),
        Some(b'b') => Some(Role::Bishop),
        Some(b'n') => Some(Role::Knight),
        _ => None,
    };
    let last = if turn == Color::White { Rank::Eighth } else { Rank::First };
    pos.legal_moves().into_iter().find(|&m| match m {
        Move::Castle { king, rook } => {
            king == from && (rook == to || (to.rank() == king.rank() && (to.file() == File::G || to.file() == File::C) && ((to.file() == File::G) == (rook.file() > king.file()))))
        }
        Move::Normal { from: f, to: t, promotion, role, .. } => {
            f == from && t == to && {
                if role == Role::Pawn && t.rank() == last {
                    // plain 7th-to-8th = knight in Lc0's index; q, r, b explicit
                    promotion == Some(promo.unwrap_or(Role::Knight))
                } else {
                    promo.is_none()
                }
            }
        }
        Move::EnPassant { from: f, to: t } => f == from && t == to,
        Move::Put { .. } => false,
    })
}

fn wdl_abs(q: f32, d: f32, stm: Color) -> [f32; 3] {
    let (q, d) = (q.clamp(-1.0, 1.0) as f64, d.clamp(0.0, 1.0) as f64);
    let w = (0.5 * (1.0 - d + q)).max(0.0);
    let l = (0.5 * (1.0 - d - q)).max(0.0);
    let s = w + l + d;
    let (w, l, d) = (w / s, l / s, d / s);
    match stm {
        Color::White => [w as f32, l as f32, d as f32],
        Color::Black => [l as f32, w as f32, d as f32],
    }
}

#[derive(Default)]
struct Stats {
    games: u64,
    records: u64,
    kept: u64,
    invalid_position: u64,
    link_ok: u64,
    link_fail: u64,
    history_ok: u64,
    history_fail: u64,
    games_nonstandard: u64,
    records_nonstandard: u64,
    setup_with_history: u64,
    setup_history_move_found: u64,
    rights_mismatch: u64,
    first_is_start: u64,
    in_check: u64,
    best_capture: u64,
    best_promotion: u64,
    best_unmatched: u64,
    marked_deleted: u64,
    ep_set: u64,
    label_draw_sum: f64,
    label_white_sum: f64,
}

fn convert(out: &str, stats_path: &str, policy_path: &str) -> io::Result<()> {
    let policy: Vec<String> = std::fs::read_to_string(policy_path)?.lines().map(str::to_string).collect();
    assert_eq!(policy.len(), 1858, "policy index: 1858 moves");
    let mut input = BufReader::with_capacity(1 << 24, io::stdin().lock());
    let mut output = BufWriter::with_capacity(1 << 22, std::fs::File::create(out)?);
    let mut st = Stats::default();
    let mut count = [0u8; 4];
    let mut buf = Vec::new();
    loop {
        match input.read_exact(&mut count) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e),
        }
        let n = u32::from_le_bytes(count) as usize;
        buf.resize(n * IN_REC, 0);
        input.read_exact(&mut buf)?;
        let game = st.games as u32;
        st.games += 1;
        // standard chess only: T60 also played Chess960 games (and our encoding, move generator and models are
        // standard chess); a game whose first record isn't the standard start position is skipped whole
        if n > 0 {
            let raw = parse(&buf[..IN_REC]);
            let stm = if raw.stm == 0 { Color::White } else { Color::Black };
            let start = Chess::default();
            if stm != Color::White || slot_board(&raw, 0, stm) != *start.board() || castling(&raw, stm) != start.castles().castling_rights() {
                st.games_nonstandard += 1;
                st.records_nonstandard += n as u64;
                st.records += n as u64;
                if std::env::var_os("LEELA_DEBUG").is_some() {
                    eprintln!("non-standard start: {} {:?}", slot_board(&raw, 0, stm), stm);
                }
                continue;
            }
        }
        let mut prev: Option<Chess> = None; // the previous record's position
        for i in 0..n {
            st.records += 1;
            let raw = parse(&buf[i * IN_REC..(i + 1) * IN_REC]);
            let stm = if raw.stm == 0 { Color::White } else { Color::Black };
            let board = slot_board(&raw, 0, stm);
            let rights = castling(&raw, stm);
            // consecutive records must be one legal move apart: the position is then the previous one with that move
            // played (en passant square and castling rights as our self-play gets them)
            let mut linked: Option<Chess> = None;
            if let Some(pp) = &prev {
                if !slot_empty(&raw, 1) {
                    if slot_board(&raw, 1, stm) == *pp.board() {
                        st.history_ok += 1;
                    } else {
                        st.history_fail += 1;
                    }
                }
                match move_to(pp, &board) {
                    Some((_, p)) if p.turn() == stm => {
                        st.link_ok += 1;
                        if p.castles().castling_rights() != rights {
                            st.rights_mismatch += 1;
                        }
                        linked = Some(p);
                    }
                    _ => st.link_fail += 1,
                }
            }
            let pos = match linked {
                Some(p) => p,
                None => {
                    // a game's first record (or after a broken link): from the record, the last move (for a double
                    // push's en passant square) from history slot 1 when there is one
                    let mut ep = None;
                    if !slot_empty(&raw, 1) {
                        st.setup_with_history += 1;
                        let pb = slot_board(&raw, 1, stm);
                        let pr = plausible_rights(&pb);
                        if let Some(pp) = position(pb, !stm, pr, None) {
                            if let Some((m, _)) = move_to(&pp, &board) {
                                st.setup_history_move_found += 1;
                                ep = double_push(m, !stm);
                            }
                        }
                    }
                    match position(board, stm, rights, ep) {
                        Some(p) => p,
                        None => {
                            st.invalid_position += 1;
                            prev = None;
                            continue;
                        }
                    }
                }
            };
            if i == 0 && pos == Chess::default() {
                st.first_is_start += 1;
            }
            if pos.ep_square(EnPassantMode::Always).is_some() {
                st.ep_set += 1;
            }
            prev = Some(pos.clone());
            if raw.inv & 0x40 != 0 {
                st.marked_deleted += 1;
                continue;
            }
            if pos.is_check() {
                st.in_check += 1;
                continue;
            }
            let uci = &policy[raw.best_idx as usize];
            match best_move(&pos, uci, stm) {
                None => st.best_unmatched += 1,
                Some(m) if m.is_capture() => {
                    st.best_capture += 1;
                    continue;
                }
                Some(m) if m.is_promotion() => {
                    st.best_promotion += 1;
                    continue;
                }
                Some(_) => {}
            }
            let enc = encoding::encode(&pos);
            let leela = wdl_abs(raw.best_q, raw.best_d, stm);
            let result = wdl_abs(raw.result_q, raw.result_d, stm);
            let mut rec = [0u8; OUT_REC];
            rec[..70].copy_from_slice(&enc);
            for (k, v) in leela.iter().chain(result.iter()).enumerate() {
                rec[70 + 4 * k..74 + 4 * k].copy_from_slice(&v.to_le_bytes());
            }
            rec[94..98].copy_from_slice(&game.to_le_bytes());
            output.write_all(&rec)?;
            st.kept += 1;
            st.label_draw_sum += leela[2] as f64;
            st.label_white_sum += leela[0] as f64;
        }
    }
    output.flush()?;
    let s = &st;
    let kept = s.kept.max(1) as f64;
    let fields: Vec<(&str, String)> = vec![
        ("games", s.games.to_string()),
        ("records", s.records.to_string()),
        ("games_nonstandard", s.games_nonstandard.to_string()),
        ("records_nonstandard", s.records_nonstandard.to_string()),
        ("invalid_position", s.invalid_position.to_string()),
        ("link_ok", s.link_ok.to_string()),
        ("link_fail", s.link_fail.to_string()),
        ("history_ok", s.history_ok.to_string()),
        ("history_fail", s.history_fail.to_string()),
        ("rights_mismatch", s.rights_mismatch.to_string()),
        ("setup_with_history", s.setup_with_history.to_string()),
        ("setup_history_move_found", s.setup_history_move_found.to_string()),
        ("first_is_start", s.first_is_start.to_string()),
        ("ep_set", s.ep_set.to_string()),
        ("marked_deleted", s.marked_deleted.to_string()),
        ("in_check", s.in_check.to_string()),
        ("best_capture", s.best_capture.to_string()),
        ("best_promotion", s.best_promotion.to_string()),
        ("best_unmatched", s.best_unmatched.to_string()),
        ("kept", s.kept.to_string()),
        ("mean_draw", format!("{:.4}", s.label_draw_sum / kept)),
        ("mean_white", format!("{:.4}", s.label_white_sum / kept)),
    ];
    let json = format!("{{{}}}\n", fields.iter().map(|(k, v)| format!("\"{k}\": {v}")).collect::<Vec<_>>().join(", "));
    std::fs::write(stats_path, json)?;
    Ok(())
}

/// Reads a part's records game by game: (game id, its records).
fn games_of(path: &str) -> io::Result<Vec<(u32, Vec<u8>)>> {
    let data = std::fs::read(path)?;
    assert_eq!(data.len() % OUT_REC, 0, "{path}: not whole records");
    let mut games: Vec<(u32, Vec<u8>)> = Vec::new();
    for rec in data.chunks_exact(OUT_REC) {
        let g = u32::from_le_bytes(rec[94..98].try_into().unwrap());
        match games.last_mut() {
            Some((id, recs)) if *id == g => recs.extend_from_slice(rec),
            _ => games.push((g, rec.to_vec())),
        }
    }
    Ok(games)
}

fn chunk(outdir: &str, size: usize, parts: &[String]) -> io::Result<()> {
    std::fs::create_dir_all(outdir)?;
    let (mut round, mut in_round, mut games_in_round) = (0usize, 0usize, 0u32);
    let mut out = BufWriter::new(std::fs::File::create(format!("{outdir}/round{round}.rec"))?);
    let mut sizes = Vec::new();
    for part in parts {
        for (_, recs) in games_of(part)? {
            for rec in recs.chunks_exact(OUT_REC) {
                let mut r = rec.to_vec();
                r[94..98].copy_from_slice(&games_in_round.to_le_bytes());
                out.write_all(&r)?;
            }
            in_round += recs.len() / OUT_REC;
            games_in_round += 1;
            if in_round >= size {
                out.flush()?;
                sizes.push((in_round, games_in_round));
                round += 1;
                in_round = 0;
                games_in_round = 0;
                out = BufWriter::new(std::fs::File::create(format!("{outdir}/round{round}.rec"))?);
            }
        }
    }
    out.flush()?;
    if in_round > 0 {
        sizes.push((in_round, games_in_round));
    } else {
        std::fs::remove_file(format!("{outdir}/round{round}.rec"))?;
    }
    let mut text = String::from("round,positions,games,cumulative\n");
    let mut cum = 0;
    for (r, (n, g)) in sizes.iter().enumerate() {
        cum += n;
        text += &format!("{r},{n},{g},{cum}\n");
    }
    std::fs::write(format!("{outdir}/rounds.csv"), text)?;
    println!("{} rounds, {} positions", sizes.len(), cum);
    Ok(())
}

fn take(out: &str, n: usize, parts: &[String]) -> io::Result<()> {
    let mut w = BufWriter::new(std::fs::File::create(out)?);
    let (mut count, mut games) = (0usize, 0u32);
    'outer: for part in parts {
        for (_, recs) in games_of(part)? {
            for rec in recs.chunks_exact(OUT_REC) {
                let mut r = rec.to_vec();
                r[94..98].copy_from_slice(&games.to_le_bytes());
                w.write_all(&r)?;
            }
            count += recs.len() / OUT_REC;
            games += 1;
            if count >= n {
                break 'outer;
            }
        }
    }
    w.flush()?;
    println!("{count} positions, {games} games");
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("convert") if args.len() == 5 => convert(&args[2], &args[3], &args[4]),
        Some("chunk") if args.len() >= 5 => chunk(&args[2], args[3].parse().expect("CHUNK"), &args[4..]),
        Some("take") if args.len() >= 5 => take(&args[2], args[3].parse().expect("N"), &args[4..]),
        _ => {
            eprintln!("usage: leela_convert convert OUT.part STATS.json POLICY.txt < stream | chunk OUTDIR CHUNK PART... | take OUT.rec N PART...");
            std::process::exit(2)
        }
    }
}
