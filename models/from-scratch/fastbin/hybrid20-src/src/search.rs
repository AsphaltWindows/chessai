//! Alpha-beta search over a classifier's evaluation.
//!
//! Positions are valued for the side to move by the model's log-odds,
//! ln P(side to move wins) - ln P(it loses), in nats. Positions where the
//! game is over by the rules are valued exactly: checkmate is a loss,
//! stalemate, insufficient material and the 50-move rule are draws (0). As
//! engines usually do, a position that repeats one reached since the last
//! irreversible move (in the game or along the search path) counts as a
//! draw.
//!
//! Move order (hybrid10). Values only (`root_values`, which gives every root
//! move a full window, so each value is exact in any move order): a node one
//! ply above the leaves tries captures and promotions first (MVV-LVA) when a
//! cutoff is possible, and the fixed-point search keys deeper nodes' moves from
//! the children's sums alone. Only the nodes visited change, not the values.
//! The outcome search (`root_outcomes`, `expected_outcome`) keeps hybrid9's
//! order: its principal variation's leaf can depend on it when values tie.
//! CHESSAI_ORDER1=off gives hybrid9's order everywhere (for checks).
//!
//! Sums from the move (hybrid11). In the values-only fixed-point search, the ordering keys and the leaves take a
//! child's sums from the move itself (`Acc::move_sums`) instead of from the played child (`Acc::child_sums`): the
//! same integer sums, so the same keys, order, nodes and values; the keys no longer copy the position and play the
//! move, and they reuse one buffer per depth. CHESSAI_MOVESUMS=off gives hybrid10's code (for checks).
//!
//! Staged first move (hybrid12). In the values-only fixed-point search, a node one ply above the leaves with a cut
//! window searches one likely refutation before generating its moves: a legal capture with the smallest MVV-LVA key,
//! found from the bitboards (`best_capture`; not in check, no pawn about to promote). All legal moves are generated,
//! in `captures_first`'s order without that capture, only if it doesn't cut off. A legal capture exists, so the node
//! is neither mate nor stalemate, and the draw checks run as before. Otherwise (in check, no capture found, a pawn
//! about to promote) one scan of the generated moves finds the move `captures_first` would put first, and the full
//! partition and sort happen only if it doesn't cut off. Every root move still gets a full window, so the root values
//! are the same in any move order; only the nodes visited can change (when keys tie). The float negamax (trees past
//! ply 2, FM) and the outcome search are unchanged. CHESSAI_STAGED=off gives hybrid11's code (for checks);
//! CHESSAI_FX_CHECK=1 asserts each staged move legal and of the smallest key.
//!
//! Root cut windows (hybrid13). A log-odds softmax at a low temperature, or the log-odds argmax, needs the exact value
//! only of root moves that can be picked. `root_values_windowed` searches the root moves in static-key order: the
//! first with a full window, every other with the window (below(best so far), +inf). A move whose value is above the
//! window's bottom gets its exact value, the same bits as `root_values`; any other gets an upper bound (the search is
//! fail-soft), at or below the bottom, so below the best value. `player` then decides the pick from the bounds where
//! they decide it exactly, with the same draw from the generator, and searches a move again with a full window
//! (`root_value_of`) where they don't. Only the root moves' windows change, not any value or pick. The compound path
//! (trees and DAGs at depths 1-2), fastprobit (a flat probit at depth 2) and depth 1 keep their full searches
//! (`windowable`). CHESSAI_WINDOWS=off gives hybrid12's code (for checks).
//!
//! Bounds past ply 2 (hybrid18). Values only, at depth 3 or more: trees and DAGs (a compound model, see `compound`)
//! search with compound accumulators (one per depth, from the move's changes; no encoding, routers in fixed point)
//! instead of the float negamax, and a flat probit's leaves and keys use hybrid8's ln Phi table. Ordering keys come
//! from the estimates with their bounds (`order_by_bounds`): where two keys' intervals overlap, both are valued in
//! full and sorted by those values, so the moves are in the float order exactly (the stable sort by the full keys). A
//! node one ply above the leaves estimates its leaves in the float search's order and fails high at the first leaf
//! whose bound proves a value at or above beta (the float search fails high too, at that leaf or an earlier one); if
//! none does, it fails low with the largest upper bound if every bound is at or below alpha, else its value is the
//! largest of the leaves', valued in full only where a leaf can be the largest, in move order with the float loop's
//! comparisons (`Compound::best`, `fastprobit::best`): the same value, ties and signed zeros included. So every node
//! fails high, fails low or is exact exactly as in the float search, with the same exact values; alpha moves only on
//! exact values, so every window and every node above the leaves is the float search's; only the bounds a node
//! returns when it fails can differ (both valid). Root values (full windows) are bit-equal; the root cut windows
//! (hybrid13) get valid bounds and decide the same picks. The outcome search, depths 1-2 and models without bounds
//! (flat LR and NB, FM, MoEs, graphs over compound's caps) are unchanged. CHESSAI_FX_DEEP=off gives hybrid17's code;
//! CHESSAI_FX_CHECK=1 asserts every leaf bound and every key order against the float values and every root value
//! against the float search (hybrid17's code), bit for bit.

use shakmaty::zobrist::{Zobrist64, ZobristValue};
use shakmaty::{Bitboard, Chess, Color, EnPassantMode, Move, MoveList, Position, Rank, Role, attacks};

use crate::calibration::Calibration;
use crate::classifier::Classifier;
use crate::compound::{self, Compound};
use crate::encoding;
use crate::fastpath;
use crate::fastprobit;
use crate::fixed::{self, Acc, FxLinear};
use crate::game::Game;
use crate::player::log_odds;

/// How a position is valued for the side to move.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Value {
    /// ln P(win) - ln P(loss), which ignores draws.
    LogOdds,
    /// ln(score / (1 - score)) for the expected score P(win) + P(draw) / 2,
    /// from calibrated probabilities: a draw counts half. Equal to log-odds
    /// when draws are impossible.
    Score,
}

/// Value of being checkmated now, for the side to move: -MATE. Mates
/// further away are worth `MATE - plies`, so quicker mates are preferred.
pub const MATE: f64 = 1e6;

/// The outcome at the end of a principal variation in the fixed-point search: known (the game ended), or the
/// encoding of the leaf, whose outcome the float model gives once the search is done (as `outcome` would), so
/// outcomes are computed once per search instead of at every leaf, and exactly as before.
#[derive(Clone, Copy, Debug)]
enum Pv {
    Known([f64; 3]),
    Leaf(encoding::Encoded),
}

/// Outcomes as probabilities of [white win, black win, draw].
const WHITE_WINS: [f64; 3] = [1.0, 0.0, 0.0];
const BLACK_WINS: [f64; 3] = [0.0, 1.0, 0.0];
const DRAW: [f64; 3] = [0.0, 0.0, 1.0];

/// Whether the values-only search uses hybrid10's move order (see the module's doc): not with CHESSAI_ORDER1=off.
/// Read once.
fn order1() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("CHESSAI_ORDER1").map_or(true, |v| v != "off"))
}

/// Whether the values-only fixed-point search takes children's sums from the moves (hybrid11, see the module's doc):
/// not with CHESSAI_MOVESUMS=off. Read once.
fn movesums() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("CHESSAI_MOVESUMS").map_or(true, |v| v != "off"))
}

/// The Zobrist hash (en passant mode `Legal`) of the position after `m`, a reversible move of `pos` (not
/// `pos.is_irreversible(m)`), from `pos`'s hash `hash` (hybrid11). A reversible move is a quiet move of a piece other
/// than a pawn that leaves the castling rights alone, from a position without a legal en passant capture: the hash
/// changes by the side to move and the piece's two squares. shakmaty's `update_zobrist_hash` gives the same where it
/// gives an answer and declines after any double pawn push and for every king move, where the search then hashed the
/// whole position. Checked against the full hash by a test.
#[inline]
fn reversible_hash(pos: &Chess, hash: Zobrist64, m: Move) -> Zobrist64 {
    let Move::Normal { role, from, to, .. } = m else { unreachable!("a reversible move is a normal move") };
    let piece = role.of(pos.turn());
    let mut h = hash;
    h ^= Zobrist64::zobrist_for_white_turn();
    h ^= Zobrist64::zobrist_for_piece(from, piece);
    h ^= Zobrist64::zobrist_for_piece(to, piece);
    h
}

/// Whether the values-only fixed-point search stages its first move at depth-1 cut nodes (hybrid12, see the module's
/// doc): not with CHESSAI_STAGED=off. Read once.
fn staged() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("CHESSAI_STAGED").map_or(true, |v| v != "off"))
}

/// hybrid18: whether the values-only searches past ply 2 use bounds (see the module's doc): not with
/// CHESSAI_FX_DEEP=off. Read once.
fn deep() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| match std::env::var("CHESSAI_FX_DEEP").as_deref() {
        Ok("off") => false,
        Ok("on") | Err(_) => true,
        Ok(other) => panic!("CHESSAI_FX_DEEP must be on or off, not {other:?}"),
    })
}

/// hybrid18: counts for CHESSAI_FX_STATS=1 (`report`).
pub struct DeepStats {
    /// Root searches past ply 2 with bounds (compound, flat probit).
    pub searches: std::sync::atomic::AtomicU64,
    /// Nodes one ply above the leaves; those failing high or low on bounds alone; leaves estimated.
    pub nodes1: std::sync::atomic::AtomicU64,
    pub fail_high: std::sync::atomic::AtomicU64,
    pub fail_low: std::sync::atomic::AtomicU64,
    pub leaves: std::sync::atomic::AtomicU64,
    /// Ordering keys, and those valued in full because intervals overlapped.
    pub keys: std::sync::atomic::AtomicU64,
    pub keys_full: std::sync::atomic::AtomicU64,
}

pub static DEEP_STATS: DeepStats = DeepStats {
    searches: std::sync::atomic::AtomicU64::new(0),
    nodes1: std::sync::atomic::AtomicU64::new(0),
    fail_high: std::sync::atomic::AtomicU64::new(0),
    fail_low: std::sync::atomic::AtomicU64::new(0),
    leaves: std::sync::atomic::AtomicU64::new(0),
    keys: std::sync::atomic::AtomicU64::new(0),
    keys_full: std::sync::atomic::AtomicU64::new(0),
};

#[inline]
fn deep_count(counter: &std::sync::atomic::AtomicU64, n: u64) {
    if compound::stats_on() {
        counter.fetch_add(n, std::sync::atomic::Ordering::Relaxed);
    }
}

/// hybrid18: prints the counts to stderr, with CHESSAI_FX_STATS=1.
pub fn report() {
    if !compound::stats_on() {
        return;
    }
    let g = |c: &std::sync::atomic::AtomicU64| c.load(std::sync::atomic::Ordering::Relaxed);
    let s = &DEEP_STATS;
    eprintln!(
        "deep fx: {} searches; nodes above the leaves {}, failed high on a bound {}, failed low on bounds {}; leaves \
         estimated {}; keys {}, valued in full {}",
        g(&s.searches),
        g(&s.nodes1),
        g(&s.fail_high),
        g(&s.fail_low),
        g(&s.leaves),
        g(&s.keys),
        g(&s.keys_full)
    );
}

/// hybrid18: `order` becomes the order of a stable sort of keys 0..n descending by `total_cmp` (ties in index order),
/// as `negamax` and `negamax_fx` sort their moves, from estimates: key i is within `est[i].1` of `est[i].0` (a bound of
/// 0: the key itself). Keys are sorted by their intervals' tops; a key whose top is below the lowest bottom of the
/// keys before it starts a new group, all of whose keys are below all of the earlier ones. Within a group of two or
/// more, every key is valued in full (`exact(i)`) and the group is sorted by those values, ties in index order. If an
/// estimate isn't finite, every key is valued in full. `ex` is a buffer.
fn order_by_bounds(est: &[(f64, f64)], mut exact: impl FnMut(usize) -> f64, order: &mut Vec<usize>, ex: &mut Vec<f64>) {
    let n = est.len();
    order.clear();
    order.extend(0..n);
    ex.clear();
    ex.resize(n, f64::NAN);
    deep_count(&DEEP_STATS.keys, n as u64);
    if est.iter().any(|&(k, e)| !(k.is_finite() && e.is_finite())) {
        for i in 0..n {
            ex[i] = if est[i].1 == 0.0 { est[i].0 } else { exact(i) };
        }
        deep_count(&DEEP_STATS.keys_full, n as u64);
        order.sort_by(|&a, &b| ex[b].total_cmp(&ex[a]));
        return;
    }
    let hi = |i: usize| est[i].0 + est[i].1;
    let lo = |i: usize| est[i].0 - est[i].1;
    order.sort_by(|&a, &b| hi(b).total_cmp(&hi(a)));
    let mut start = 0;
    while start < n {
        let mut end = start + 1;
        let mut low = lo(order[start]);
        while end < n && !(hi(order[end]) < low) {
            low = low.min(lo(order[end]));
            end += 1;
        }
        if end - start > 1 {
            for &i in &order[start..end] {
                ex[i] = if est[i].1 == 0.0 { est[i].0 } else { exact(i) };
            }
            deep_count(&DEEP_STATS.keys_full, (end - start) as u64);
            order[start..end].sort_by(|&a, &b| ex[b].total_cmp(&ex[a]).then(a.cmp(&b)));
        }
        start = end;
    }
}

/// hybrid18: the stable sort's order of `keys` descending by `total_cmp`, for checks.
fn float_order(keys: &[f64]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..keys.len()).collect();
    order.sort_by(|&a, &b| keys[b].total_cmp(&keys[a]));
    order
}

/// hybrid18: a leaf whose value (for its side to move) is within `e` of `v` makes its parent, searched with `beta`,
/// fail high if v + e <= -beta: the parent's value is then at least -(v + e) >= beta, a valid lower bound.
#[inline]
fn fails_high((v, e): (f64, f64), beta: f64) -> Option<f64> {
    let top = v + e;
    (top <= -beta).then_some(-top)
}

/// hybrid18: leaves (values within e of v) all at or below the parent's `alpha` from its side: the parent fails low,
/// its value at most the largest -(v - e), a valid upper bound. A value that is not a number is ignored, as the
/// float loop ignores it.
#[inline]
fn fails_low(leaves: impl Iterator<Item = (f64, f64)>, alpha: f64) -> Option<f64> {
    if alpha == f64::NEG_INFINITY {
        return None;
    }
    let mut lowest = f64::INFINITY;
    for (v, e) in leaves {
        let bottom = v - e;
        if bottom < lowest {
            lowest = bottom;
        }
    }
    (lowest >= -alpha).then_some(-lowest)
}

/// hybrid18: one depth's buffers for the searches past ply 2 (`Search::levels`), reused.
#[derive(Default)]
struct Level {
    /// Compound: each move's key (see `CKey`).
    ckeys: Vec<CKey>,
    /// Flat probit: each move and its child's sums.
    pkeys: Vec<(Move, fixed::Sums)>,
    /// The keys' estimates and bounds, their order and full values (`order_by_bounds`).
    est: Vec<(f64, f64)>,
    order: Vec<usize>,
    ex: Vec<f64>,
}

/// hybrid18: a move of a compound search's node at depth 2 or more: its changes, its key's reply (the child's value,
/// estimated) and the position after it.
struct CKey {
    m: Move,
    ch: compound::Changes,
    r: compound::Reply,
    next: Chess,
}

/// A piece's worth for MVV-LVA, as `captures_first` counts it.
#[inline]
fn mvv_worth(r: Role) -> i32 {
    match r {
        Role::Pawn => 1,
        Role::Knight | Role::Bishop => 3,
        Role::Rook => 5,
        Role::Queen => 9,
        Role::King => 0,
    }
}

/// `captures_first`'s sort key of a capture or promotion (smaller first).
#[inline]
fn mvv_lva_key(m: &Move) -> i32 {
    let promo = m.promotion().map_or(0, |r| mvv_worth(r) - 1);
    match m.capture() {
        Some(victim) => -(10 * (mvv_worth(victim) + promo) - mvv_worth(m.role())),
        None => -(10 * promo),
    }
}

/// A legal capture of `pos` with the smallest `mvv_lva_key` among its normal (not en passant) captures, found from
/// the bitboards without generating moves (hybrid12); None if there is none. Only for a position that is not in check
/// and has no pawn of the side to move about to promote (None then, to be safe): every capture is then a normal move
/// without promotion, legal unless a pinned piece leaves its pin's line or the king captures onto an attacked square.
fn best_capture(pos: &Chess) -> Option<Move> {
    let board = pos.board();
    let turn = pos.turn();
    let king = board.king_of(turn)?;
    let us = board.by_color(turn);
    let them = board.by_color(!turn);
    let occ = board.occupied();
    let seventh = Bitboard::from_rank(if turn == Color::White { Rank::Seventh } else { Rank::Second });
    if (board.pawns() & us & seventh).any() {
        return None;
    }
    // Our pieces pinned to our king: the only piece between the king and an enemy slider on the slider's line.
    let snipers = (attacks::rook_attacks(king, Bitboard::EMPTY) & board.rooks_and_queens() & them)
        | (attacks::bishop_attacks(king, Bitboard::EMPTY) & board.bishops_and_queens() & them);
    let mut pinned = Bitboard::EMPTY;
    for sniper in snipers {
        let between = attacks::between(king, sniper) & occ;
        if between.count() == 1 {
            pinned |= between & us;
        }
    }
    // Victims by class, most valuable first: the keys of different classes don't overlap (-10 * victim + attacker,
    // attacker 0-9), so the first class with a legal capture holds the smallest key.
    let classes: [Bitboard; 4] = [
        board.by_role(Role::Queen),
        board.by_role(Role::Rook),
        board.by_role(Role::Bishop) | board.by_role(Role::Knight),
        board.by_role(Role::Pawn),
    ];
    for class in classes {
        let mut best: Option<(i32, Move)> = None;
        for to in class & them {
            let victim = board.role_at(to).expect("a piece on the square");
            for from in board.attacks_to(to, turn, occ) {
                let role = board.role_at(from).expect("a piece on the square");
                if role == Role::King {
                    // The king may capture only onto a square the enemy doesn't attack once the king has left its own.
                    if board.attacks_to(to, !turn, occ ^ Bitboard::from_square(king)).any() {
                        continue;
                    }
                } else if pinned.contains(from) && !attacks::aligned(from, to, king) {
                    continue;
                }
                let key = -(10 * mvv_worth(victim) - mvv_worth(role));
                if best.is_none_or(|b| key < b.0) {
                    best = Some((key, Move::Normal { role, from, capture: Some(victim), to, promotion: None }));
                }
            }
        }
        if let Some((_, m)) = best {
            return Some(m);
        }
    }
    None
}

/// CHESSAI_FX_CHECK=1 (hybrid12): `m0`, from `best_capture`, is a legal move of `pos` with the smallest key among its
/// legal normal captures.
fn check_best_capture(pos: &Chess, m0: Move) {
    let moves = pos.legal_moves();
    assert!(moves.contains(&m0), "best_capture gave an illegal move {m0:?} in {:?}", pos.board());
    let min = moves.iter().filter(|m| matches!(m, Move::Normal { capture: Some(_), .. })).map(mvv_lva_key).min();
    assert_eq!(min, Some(mvv_lva_key(&m0)), "best_capture's {m0:?} hasn't the smallest key in {:?}", pos.board());
}

/// Moves the captures and promotions of `moves` to the front, in place, the most valuable victim first and among
/// equal victims the least valuable attacker (MVV-LVA); the other moves follow in no particular order.
fn captures_first(moves: &mut MoveList) {
    fn worth(r: Role) -> i32 {
        match r {
            Role::Pawn => 1,
            Role::Knight | Role::Bishop => 3,
            Role::Rook => 5,
            Role::Queen => 9,
            Role::King => 0,
        }
    }
    let mut k = 0;
    for i in 0..moves.len() {
        if moves[i].is_capture() || moves[i].is_promotion() {
            moves.swap(i, k);
            k += 1;
        }
    }
    moves[..k].sort_unstable_by_key(|m| {
        let promo = m.promotion().map_or(0, |r| worth(r) - 1);
        match m.capture() {
            Some(victim) => -(10 * (worth(victim) + promo) - worth(m.role())),
            None => -(10 * promo),
        }
    });
}

pub struct Search<'a> {
    model: &'a (dyn Classifier + Sync),
    /// Hashes of the positions since the last irreversible move, of the
    /// game and then of the search path; the current position's is last.
    hashes: Vec<Zobrist64>,
    /// Index into `hashes` of the first position after the last
    /// irreversible move.
    since: usize,
    value: Value,
    calibration: Calibration,
    /// Whether outcomes are the model's own probabilities, as `predict`
    /// gives them, instead of calibrated ones (see `root_outcomes`).
    raw_outcomes: bool,
    /// Positions visited, for statistics.
    pub nodes: u64,
    /// The model in fixed point: positions are then valued from sums kept incrementally along the search.
    fx: Option<&'a FxLinear>,
    /// Whether log-odds values come from the model's logits directly (`Classifier::log_odds_direct`).
    direct: bool,
    /// The model as a compound model in fixed point (see `compound`): `root_values` at depth 1 and 2 then values
    /// positions from sums kept incrementally, with the float model's values exactly.
    cfx: Option<&'a Compound<'a>>,
    /// Whether the values-only search uses hybrid10's move order (`order1`).
    order1: bool,
    /// Whether the values-only fixed-point search takes children's sums from the moves (`movesums`).
    movesums: bool,
    /// The ordering keys' buffers, one per depth, reused (hybrid11).
    keybufs: Vec<Vec<(f64, Move)>>,
    /// Whether the values-only fixed-point search stages its first move at depth-1 cut nodes (`staged`, hybrid12).
    staged: bool,
    /// Depth-1 nodes searched from `best_capture`'s move without generating moves first (hybrid12), for statistics.
    pub staged_nodes: u64,
    /// hybrid18: whether this search's flat probit uses the ln Phi table's bounds (set per root search, see
    /// `probit_bounds`), and the buffers of the searches past ply 2.
    pbounds: bool,
    /// hybrid18: whether the bounds may be used at all (CHESSAI_FX_DEEP's per-search switch, for tests).
    allow_bounds: bool,
    levels: Vec<Level>,
    creplies: Vec<compound::Reply>,
    preplies: Vec<fastprobit::Reply>,
    scanned: Vec<Move>,
    cstack: Vec<compound::Acc>,
}

impl<'a> Search<'a> {
    pub fn new(model: &'a (dyn Classifier + Sync), game: &Game) -> Self {
        Search {
            model,
            hashes: game.history().to_vec(),
            since: 0,
            value: Value::LogOdds,
            calibration: Calibration::NONE,
            raw_outcomes: false,
            nodes: 0,
            fx: None,
            direct: fixed::mode() != fixed::Mode::Off,
            cfx: None,
            order1: order1(),
            movesums: movesums(),
            keybufs: Vec::new(),
            staged: staged(),
            staged_nodes: 0,
            pbounds: false,
            allow_bounds: deep(),
            levels: Vec::new(),
            creplies: Vec::new(),
            preplies: Vec::new(),
            scanned: Vec::new(),
            cstack: Vec::new(),
        }
    }

    /// hybrid18: whether `root_values` at `depth` takes the compound search with bounds (see the module's doc).
    fn compound_deep(&self, depth: u32) -> bool {
        depth >= 3 && self.cfx.is_some() && self.value == Value::LogOdds && self.direct && self.fx.is_none() && self.allow_bounds
    }

    /// hybrid18: whether a fixed-point search at `depth` of `fx` uses the ln Phi table's bounds (see the module's
    /// doc): a flat probit, log-odds values, depth 3 or more, on the values-only order with sums from the moves.
    fn probit_bounds(&self, fx: &FxLinear, depth: u32) -> bool {
        depth >= 3
            && self.value == Value::LogOdds
            && matches!(fx.head(), fixed::Head::Probit { .. })
            && self.order1
            && self.movesums
            && self.allow_bounds
    }

    /// hybrid18: depth `d`'s buffers, taken (give them back with `put_level`).
    fn level(&mut self, d: usize) -> Level {
        if self.levels.len() <= d {
            self.levels.resize_with(d + 1, Level::default);
        }
        std::mem::take(&mut self.levels[d])
    }

    fn put_level(&mut self, d: usize, level: Level) {
        self.levels[d] = level;
    }

    /// Values positions with `cfx`, the model as a compound model in fixed point, if given (see `compound`).
    pub fn with_compound(mut self, cfx: Option<&'a Compound<'a>>) -> Self {
        self.cfx = cfx;
        self
    }

    /// Values positions with `fx`, the model in fixed point, if given.
    pub fn with_fx(mut self, fx: Option<&'a FxLinear>) -> Self {
        self.fx = fx;
        self
    }

    /// Values positions by `value`, with probabilities calibrated by
    /// `calibration` (which `Value::Score` and `expected_outcome` use).
    pub fn valuing(mut self, value: Value, calibration: Calibration) -> Self {
        self.value = value;
        self.calibration = calibration;
        self
    }

    /// The value of each legal move of the game's position for the side to
    /// move, in `legal_moves()` order, searching `depth` plies deep: at
    /// depth 1 the model's value of the position after the move.
    pub fn root_values(&mut self, game: &Game, depth: u32) -> Vec<f64> {
        assert!(depth >= 1, "search depth must be at least 1");
        if let Some(c) = self.cfx {
            if depth <= 2 && self.value == Value::LogOdds && self.direct && self.fx.is_none() {
                return self.root_values_compound(c, game, depth);
            }
            if self.compound_deep(depth) {
                return self.root_values_compound_deep(c, game, depth);
            }
        }
        let pos = game.position();
        if let Some(fx) = self.fx {
            if depth == 2
                && self.value == Value::LogOdds
                && matches!(fx.head(), fixed::Head::Probit { .. })
                && fastprobit::enabled()
            {
                return self.root_values_probit(fx, game);
            }
            return self.root_values_fx_full(game, depth);
        }
        pos.legal_moves()
            .iter()
            .map(|&m| -self.child::<false>(pos, game.hash(), m, depth - 1, 1, f64::NEG_INFINITY, f64::INFINITY).0)
            .collect()
    }

    /// `root_values` with the model in fixed point, every leaf valued in full (hybrid7's search).
    pub(crate) fn root_values_fx_full(&mut self, game: &Game, depth: u32) -> Vec<f64> {
        let fx = self.fx.expect("a model in fixed point");
        self.pbounds = self.probit_bounds(fx, depth);
        if self.pbounds {
            deep_count(&DEEP_STATS.searches, 1);
        }
        let pos = game.position();
        let acc = Acc::new(fx, pos);
        let values: Vec<f64> = pos
            .legal_moves()
            .iter()
            .map(|&m| -self.child_fx::<false>(fx, pos, &acc, game.hash(), m, depth - 1, 1, f64::NEG_INFINITY, f64::INFINITY).0)
            .collect();
        if self.pbounds && fixed::checking() {
            // hybrid18: against hybrid17's search (no bounds), bit for bit.
            self.pbounds = false;
            let full: Vec<f64> = pos
                .legal_moves()
                .iter()
                .map(|&m| -self.child_fx::<false>(fx, pos, &acc, game.hash(), m, depth - 1, 1, f64::NEG_INFINITY, f64::INFINITY).0)
                .collect();
            self.pbounds = true;
            let bits = |v: &[f64]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
            assert_eq!(bits(&values), bits(&full), "probit bounds' values {values:?} differ from the full search's {full:?}");
        }
        values
    }

    /// hybrid13: whether `root_values` at `depth` takes a path `root_values_windowed` follows: the fixed-point search
    /// (`root_values_fx_full`) or the float negamax, at depth 2 or more. Not the compound path or fastprobit, which
    /// `root_values` takes as it decides here, nor depth 1, where nothing can be cut, nor naive Bayes (`FxLinear::root_windows`).
    pub(crate) fn windowable(&self, depth: u32) -> bool {
        if depth < 2 {
            return false;
        }
        if self.cfx.is_some() && depth <= 2 && self.value == Value::LogOdds && self.direct && self.fx.is_none() {
            return false;
        }
        if let Some(fx) = self.fx {
            if depth == 2
                && self.value == Value::LogOdds
                && matches!(fx.head(), fixed::Head::Probit { .. })
                && fastprobit::enabled()
            {
                return false;
            }
            if !fx.root_windows() {
                return false;
            }
        }
        true
    }

    /// hybrid13: move `i`'s value (in `legal_moves()` order) as `root_values` gives it on a `windowable` path: its
    /// subtree searched with a full window.
    pub(crate) fn root_value_of(&mut self, game: &Game, depth: u32, i: usize) -> f64 {
        let pos = game.position();
        let m = pos.legal_moves()[i];
        if let Some(fx) = self.fx {
            self.pbounds = self.probit_bounds(fx, depth);
            let acc = Acc::new(fx, pos);
            let v = -self.child_fx::<false>(fx, pos, &acc, game.hash(), m, depth - 1, 1, f64::NEG_INFINITY, f64::INFINITY).0;
            if self.pbounds && fixed::checking() {
                // hybrid18: against hybrid17's search (no bounds), bit for bit.
                self.pbounds = false;
                let full = -self.child_fx::<false>(fx, pos, &acc, game.hash(), m, depth - 1, 1, f64::NEG_INFINITY, f64::INFINITY).0;
                self.pbounds = true;
                assert_eq!(v.to_bits(), full.to_bits(), "probit bounds' value {v} of root move {i} differs from {full}");
            }
            return v;
        }
        if self.compound_deep(depth) {
            // hybrid18: the compound search, a full window.
            let c = self.cfx.unwrap();
            let mut stack = self.take_cstack(c, depth);
            c.root_into(pos, &mut stack[0]);
            let (root, rest) = stack.split_first_mut().unwrap();
            let mut next = pos.clone();
            next.play_unchecked(m);
            let mut ch = compound::Changes::default();
            ch.fill(pos, &next, &root.enc, c.check);
            let v = -self.child_c(c, pos, root, &ch, &next, rest, game.hash(), m, depth - 1, 1, f64::NEG_INFINITY, f64::INFINITY);
            self.cstack = stack;
            if c.check {
                let float = -self.child::<false>(pos, game.hash(), m, depth - 1, 1, f64::NEG_INFINITY, f64::INFINITY).0;
                assert_eq!(v.to_bits(), float.to_bits(), "compound value {v} of root move {i} differs from float {float}");
            }
            return v;
        }
        -self.child::<false>(pos, game.hash(), m, depth - 1, 1, f64::NEG_INFINITY, f64::INFINITY).0
    }

    /// hybrid13: the root moves' values with cut windows, on a `windowable` path (see the module's doc). The moves are
    /// searched in the order of their static keys (best first; ties in `legal_moves()` order): the first with a full
    /// window, each other with the window (below(best), +inf), `best` the largest exact value so far and below(best)
    /// smaller than it. Returns, in `legal_moves()` order, each move's value and whether it is only an upper bound
    /// (`pruned`, a fail-low at or below its window's bottom). An exact value has the same bits as `root_values`'s
    /// (up to the sign of a zero); a bound is at least the move's value and below the largest exact value.
    pub(crate) fn root_values_windowed(&mut self, game: &Game, depth: u32, below: impl Fn(f64) -> f64) -> (Vec<f64>, Vec<bool>) {
        debug_assert!(self.windowable(depth));
        if self.compound_deep(depth) {
            // hybrid18: the compound search.
            let (values, pruned) = self.root_values_windowed_compound(game, depth, &below);
            if self.cfx.unwrap().check {
                let (fv, fp) = self.root_values_windowed_plain(game, depth, &below, false);
                self.check_windowed(&values, &pruned, &fv, &fp);
            }
            return (values, pruned);
        }
        if let Some(fx) = self.fx {
            if self.probit_bounds(fx, depth) {
                // hybrid18: the ln Phi table's bounds.
                let (values, pruned) = self.root_values_windowed_plain(game, depth, &below, true);
                if fixed::checking() {
                    let (fv, fp) = self.root_values_windowed_plain(game, depth, &below, false);
                    self.check_windowed(&values, &pruned, &fv, &fp);
                }
                return (values, pruned);
            }
        }
        self.root_values_windowed_plain(game, depth, &below, false)
    }

    /// hybrid18: CHESSAI_FX_CHECK's check of a windowed root search with bounds (`values`, `pruned`) against hybrid17's
    /// (`fv`, `fp`): the same moves cut, the same exact values, bit for bit.
    fn check_windowed(&self, values: &[f64], pruned: &[bool], fv: &[f64], fp: &[bool]) {
        assert_eq!(pruned, fp, "windowed cuts with bounds differ (values {values:?}, hybrid17 {fv:?})");
        for i in 0..values.len() {
            if !pruned[i] {
                assert_eq!(values[i].to_bits(), fv[i].to_bits(), "windowed value of move {i}: {} against {}", values[i], fv[i]);
            }
        }
    }

    /// hybrid18: hybrid17's `root_values_windowed` (the fixed-point search or the float negamax), the flat probit's
    /// with the ln Phi table's bounds if `bounds`.
    fn root_values_windowed_plain(&mut self, game: &Game, depth: u32, below: &dyn Fn(f64) -> f64, bounds: bool) -> (Vec<f64>, Vec<bool>) {
        let pos = game.position();
        let moves = pos.legal_moves();
        let fx = self.fx;
        self.pbounds = bounds;
        if bounds {
            deep_count(&DEEP_STATS.searches, 1);
        }
        let acc = fx.map(|fx| Acc::new(fx, pos));
        if bounds {
            // The root's keys from the table's estimates: the same order (`order_by_bounds`).
            let (fx, acc) = (fx.unwrap(), acc.as_ref().unwrap());
            let white_next = !pos.turn().is_white();
            let mut lv = self.level(0);
            lv.pkeys.clear();
            lv.est.clear();
            for &m in &moves {
                let s = acc.move_sums(fx, pos, m);
                let (v, e) = fastprobit::estimate(fx, &s, white_next).unwrap_or_else(|| (fx.value(&s, white_next), 0.0));
                lv.pkeys.push((m, s));
                lv.est.push((-v, e));
            }
            {
                let Level { pkeys, est, order, ex, .. } = &mut lv;
                order_by_bounds(est, |i| -fx.value(&pkeys[i].1, white_next), order, ex);
            }
            if fixed::checking() {
                self.check_order_p(fx, white_next, &lv.pkeys, &lv.order);
            }
            let mut values = vec![0.0; moves.len()];
            let mut pruned = vec![false; moves.len()];
            let mut best = f64::NEG_INFINITY;
            for k in 0..lv.order.len() {
                let i = lv.order[k];
                let alpha = if best == f64::NEG_INFINITY { f64::NEG_INFINITY } else { below(best) };
                let v = -self.child_fx::<false>(fx, pos, acc, game.hash(), moves[i], depth - 1, 1, f64::NEG_INFINITY, -alpha).0;
                values[i] = v;
                if v > alpha {
                    if v > best {
                        best = v;
                    }
                } else {
                    pruned[i] = true;
                }
            }
            self.put_level(0, lv);
            return (values, pruned);
        }
        let mut order: Vec<(f64, usize)> = match (fx, &acc) {
            (Some(fx), Some(acc)) => {
                let white_next = !pos.turn().is_white();
                moves
                    .iter()
                    .enumerate()
                    .map(|(i, &m)| (-self.eval_sums_turn(fx, white_next, &acc.move_sums(fx, pos, m)), i))
                    .collect()
            }
            _ => moves
                .iter()
                .enumerate()
                .map(|(i, &m)| {
                    let mut next = pos.clone();
                    next.play_unchecked(m);
                    (-self.eval(&next), i)
                })
                .collect(),
        };
        order.sort_by(|a, b| b.0.total_cmp(&a.0));
        let mut values = vec![0.0; moves.len()];
        let mut pruned = vec![false; moves.len()];
        let mut best = f64::NEG_INFINITY;
        for &(_, i) in &order {
            let alpha = if best == f64::NEG_INFINITY { f64::NEG_INFINITY } else { below(best) };
            let v = match (fx, &acc) {
                (Some(fx), Some(acc)) => {
                    -self.child_fx::<false>(fx, pos, acc, game.hash(), moves[i], depth - 1, 1, f64::NEG_INFINITY, -alpha).0
                }
                _ => -self.child::<false>(pos, game.hash(), moves[i], depth - 1, 1, f64::NEG_INFINITY, -alpha).0,
            };
            values[i] = v;
            if v > alpha {
                if v > best {
                    best = v;
                }
            } else {
                pruned[i] = true;
            }
        }
        (values, pruned)
    }

    /// hybrid18: the compound search's accumulators, one per depth (give them back to `cstack`).
    fn take_cstack(&mut self, c: &Compound, depth: u32) -> Vec<compound::Acc> {
        let mut stack = std::mem::take(&mut self.cstack);
        while stack.len() < depth as usize + 1 {
            stack.push(c.empty());
        }
        stack
    }

    /// hybrid18: `root_values` past depth 2 with a compound model (see the module's doc): the float negamax's values,
    /// bit for bit.
    fn root_values_compound_deep(&mut self, c: &Compound, game: &Game, depth: u32) -> Vec<f64> {
        let pos = game.position();
        let moves = pos.legal_moves();
        c.count_search(moves.len());
        deep_count(&DEEP_STATS.searches, 1);
        let mut stack = self.take_cstack(c, depth);
        c.root_into(pos, &mut stack[0]);
        let (root, rest) = stack.split_first_mut().unwrap();
        let root = &*root;
        let mut ch = compound::Changes::default();
        let mut values = Vec::with_capacity(moves.len());
        for &m in &moves {
            let mut next = pos.clone();
            next.play_unchecked(m);
            ch.fill(pos, &next, &root.enc, c.check);
            values.push(-self.child_c(c, pos, root, &ch, &next, rest, game.hash(), m, depth - 1, 1, f64::NEG_INFINITY, f64::INFINITY));
        }
        self.cstack = stack;
        if c.check {
            let float: Vec<f64> = moves
                .iter()
                .map(|&m| -self.child::<false>(pos, game.hash(), m, depth - 1, 1, f64::NEG_INFINITY, f64::INFINITY).0)
                .collect();
            let bits = |v: &[f64]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
            assert_eq!(bits(&values), bits(&float), "compound values at depth {depth} {values:?} differ from float {float:?}");
        }
        values
    }

    /// hybrid18: `root_values_windowed` with the compound search: the root moves in the float order of their static
    /// keys (`-self.eval(&next)`, from estimates by `order_by_bounds`), the windows as hybrid13's.
    fn root_values_windowed_compound(&mut self, game: &Game, depth: u32, below: &dyn Fn(f64) -> f64) -> (Vec<f64>, Vec<bool>) {
        let c = self.cfx.unwrap();
        let pos = game.position();
        let moves = pos.legal_moves();
        c.count_search(moves.len());
        deep_count(&DEEP_STATS.searches, 1);
        let mut stack = self.take_cstack(c, depth);
        c.root_into(pos, &mut stack[0]);
        let (root, rest) = stack.split_first_mut().unwrap();
        let root = &*root;
        let mut lv = self.level(0);
        lv.ckeys.clear();
        lv.est.clear();
        for &m in &moves {
            let mut next = pos.clone();
            next.play_unchecked(m);
            let mut ch = compound::Changes::default();
            ch.fill(pos, &next, &root.enc, c.check);
            let r = c.reply(root, &ch, next.turn().is_white());
            let (v, e) = r.estimate();
            lv.est.push((-v, e));
            lv.ckeys.push(CKey { m, ch, r, next });
        }
        {
            let Level { ckeys, est, order, ex, .. } = &mut lv;
            order_by_bounds(est, |i| -c.exact_of(&ckeys[i].r, &ckeys[i].ch.enc), order, ex);
        }
        if c.check {
            self.check_order_c(&lv.ckeys, &lv.order);
        }
        let mut values = vec![0.0; moves.len()];
        let mut pruned = vec![false; moves.len()];
        let mut best = f64::NEG_INFINITY;
        for k in 0..lv.order.len() {
            let i = lv.order[k];
            let alpha = if best == f64::NEG_INFINITY { f64::NEG_INFINITY } else { below(best) };
            let key = &lv.ckeys[i];
            let v = -self.child_c(c, pos, root, &key.ch, &key.next, rest, game.hash(), key.m, depth - 1, 1, f64::NEG_INFINITY, -alpha);
            values[i] = v;
            if v > alpha {
                if v > best {
                    best = v;
                }
            } else {
                pruned[i] = true;
            }
        }
        self.put_level(0, lv);
        self.cstack = stack;
        (values, pruned)
    }

    /// hybrid18: CHESSAI_FX_CHECK: a compound node's key order against the float negamax's (`-self.eval(&next)`, the
    /// stable sort).
    fn check_order_c(&self, keys: &[CKey], order: &[usize]) {
        let float: Vec<f64> = keys.iter().map(|k| -self.eval(&k.next)).collect();
        assert_eq!(float_order(&float), order, "compound key order (float keys {float:?})");
    }

    /// hybrid18: CHESSAI_FX_CHECK: a flat probit node's key order against hybrid17's (`-eval_sums_turn`, the stable
    /// sort), and each key's sums against the encoding's.
    fn check_order_p(&self, fx: &FxLinear, white_next: bool, keys: &[(Move, fixed::Sums)], order: &[usize]) {
        let float: Vec<f64> = keys.iter().map(|(_, s)| -self.eval_sums_turn(fx, white_next, s)).collect();
        assert_eq!(float_order(&float), order, "probit key order (keys {float:?})");
    }

    /// hybrid18: CHESSAI_FX_CHECK: a leaf's estimate (v, e) and its compound float value `exact` against the float
    /// search's value of the leaf (`eval`): the same bits, within the bound.
    fn check_leaf(&self, (v, e): (f64, f64), exact: f64, leaf: &Chess) {
        let float = self.eval(leaf);
        assert_eq!(exact.to_bits(), float.to_bits(), "compound leaf value {exact} differs from float {float}");
        assert!(float >= v - e && float <= v + e, "leaf value {float} outside {v} +- {e}");
    }

    /// hybrid18: `child` in the compound search: `acc` is `pos`'s accumulator, `ch` the move's changes, `next` the
    /// position after it; `accs` the accumulators of `next` and the depths below.
    #[allow(clippy::too_many_arguments)]
    fn child_c(
        &mut self,
        c: &Compound,
        pos: &Chess,
        acc: &compound::Acc,
        ch: &compound::Changes,
        next: &Chess,
        accs: &mut [compound::Acc],
        hash: Zobrist64,
        m: Move,
        depth: u32,
        ply: u32,
        alpha: f64,
        beta: f64,
    ) -> f64 {
        let next_hash = pos.update_zobrist_hash(hash, m, EnPassantMode::Legal).unwrap_or_else(|| next.zobrist_hash(EnPassantMode::Legal));
        let since = self.since;
        if pos.is_irreversible(m) {
            self.since = self.hashes.len();
        }
        self.hashes.push(next_hash);
        let value = self.negamax_c(c, next, acc, ch, accs, next_hash, depth, ply, alpha, beta);
        self.hashes.pop();
        self.since = since;
        value
    }

    /// hybrid18: `negamax` (values only, depth 1 or more) in the compound search: `parent` is the accumulator of the
    /// position before, `ch` the changes to `pos`, `accs[0]` becomes `pos`'s accumulator (see the module's doc).
    #[allow(clippy::too_many_arguments)]
    fn negamax_c(
        &mut self,
        c: &Compound,
        pos: &Chess,
        parent: &compound::Acc,
        ch: &compound::Changes,
        accs: &mut [compound::Acc],
        hash: Zobrist64,
        depth: u32,
        ply: u32,
        mut alpha: f64,
        beta: f64,
    ) -> f64 {
        debug_assert!(depth >= 1);
        self.nodes += 1;
        let mut moves = pos.legal_moves();
        if moves.is_empty() {
            if !pos.is_check() {
                return 0.0;
            }
            return -(MATE - ply as f64);
        }
        if pos.is_insufficient_material() || pos.halfmoves() >= 100 || self.repeated() {
            return 0.0;
        }
        let (acc, rest) = accs.split_first_mut().expect("an accumulator per depth");
        c.child_into(parent, ch, acc);
        let acc = &*acc;
        if depth == 1 {
            if beta < f64::INFINITY && self.order1 {
                // Values only, and a cutoff is possible: likely refutations first (as `negamax`).
                captures_first(&mut moves);
            }
            return self.leaves_c(c, pos, acc, hash, ply, alpha, beta, &moves);
        }
        // `negamax`'s keys (`-self.eval(&next)`, a stable sort), in their order from estimates (`order_by_bounds`).
        let d = depth as usize;
        let mut lv = self.level(d);
        lv.ckeys.clear();
        lv.est.clear();
        for &m in &moves {
            let mut next = pos.clone();
            next.play_unchecked(m);
            let mut ch = compound::Changes::default();
            ch.fill(pos, &next, &acc.enc, c.check);
            let r = c.reply(acc, &ch, next.turn().is_white());
            let (v, e) = r.estimate();
            lv.est.push((-v, e));
            lv.ckeys.push(CKey { m, ch, r, next });
        }
        {
            let Level { ckeys, est, order, ex, .. } = &mut lv;
            order_by_bounds(est, |i| -c.exact_of(&ckeys[i].r, &ckeys[i].ch.enc), order, ex);
        }
        if c.check {
            self.check_order_c(&lv.ckeys, &lv.order);
        }
        let mut best = f64::NEG_INFINITY;
        for k in 0..lv.order.len() {
            let key = &lv.ckeys[lv.order[k]];
            let value = self.child_c(c, pos, acc, &key.ch, &key.next, rest, hash, key.m, depth - 1, ply + 1, -beta, -alpha);
            if -value > best {
                best = -value;
            }
            alpha = alpha.max(-value);
            if alpha >= beta {
                break;
            }
        }
        self.put_level(d, lv);
        best
    }

    /// hybrid18: a node one ply above the leaves in the compound search, past its end-of-game and draw checks, whose
    /// accumulator is `acc`: its leaves, in `negamax`'s order (`moves`), each estimated (`Compound::reply`): it fails
    /// high on the first bound that proves it, else fails low if every bound is at or below alpha, else takes the
    /// largest leaf value (`Compound::best`). See the module's doc.
    #[allow(clippy::too_many_arguments)]
    fn leaves_c(
        &mut self,
        c: &Compound,
        pos: &Chess,
        acc: &compound::Acc,
        hash: Zobrist64,
        ply: u32,
        alpha: f64,
        beta: f64,
        moves: &MoveList,
    ) -> f64 {
        deep_count(&DEEP_STATS.nodes1, 1);
        let mut replies = std::mem::take(&mut self.creplies);
        let mut scanned = std::mem::take(&mut self.scanned);
        replies.clear();
        scanned.clear();
        let mut ch = compound::Changes::default();
        let mut high = None;
        for &m in moves {
            // `child` and `negamax` at depth 0, with hybrid11's hashes: none for an irreversible move (it can't repeat a
            // position), from the move for a reversible one.
            let irreversible = pos.is_irreversible(m);
            let mut leaf = pos.clone();
            leaf.play_unchecked(m);
            let repeated = !irreversible && self.hashes[self.since..].contains(&reversible_hash(pos, hash, m));
            self.nodes += 1;
            let r = match self.terminal_given(&leaf, ply + 1, repeated) {
                Some(end) => compound::Reply::known(end.0),
                None => {
                    ch.fill(pos, &leaf, &acc.enc, c.check);
                    let r = c.reply(acc, &ch, leaf.turn().is_white());
                    deep_count(&DEEP_STATS.leaves, 1);
                    if c.check {
                        self.check_leaf(r.estimate(), c.exact_of(&r, &ch.enc), &leaf);
                    }
                    r
                }
            };
            if let Some(v) = fails_high(r.estimate(), beta) {
                high = Some(v);
                break;
            }
            replies.push(r);
            scanned.push(m);
        }
        let value = if let Some(v) = high {
            deep_count(&DEEP_STATS.fail_high, 1);
            v
        } else if let Some(v) = fails_low(replies.iter().map(|r| r.estimate()), alpha) {
            deep_count(&DEEP_STATS.fail_low, 1);
            v
        } else {
            c.best(&mut replies, |i| {
                let mut leaf = pos.clone();
                leaf.play_unchecked(scanned[i]);
                compound::Changes::of(pos, &leaf, &acc.enc).enc
            })
        };
        self.creplies = replies;
        self.scanned = scanned;
        value
    }

    /// hybrid18: a leaf of the fixed-point search of a flat probit (`child_fx` at depth 0, values only, log-odds), as a
    /// reply estimated from the ln Phi table: the same end-of-game checks and sums.
    #[allow(clippy::too_many_arguments)]
    fn leaf_p(&mut self, fx: &FxLinear, pos: &Chess, acc: &Acc, hash: Zobrist64, ply: u32, m: Move) -> fastprobit::Reply {
        let irreversible = pos.is_irreversible(m);
        let mut leaf = pos.clone();
        leaf.play_unchecked(m);
        let repeated = !irreversible && self.hashes[self.since..].contains(&reversible_hash(pos, hash, m));
        self.nodes += 1;
        match self.terminal_given(&leaf, ply, repeated) {
            Some(end) => fastprobit::Reply::known(end.0),
            None => {
                let sums = acc.move_sums(fx, pos, m);
                let white = leaf.turn().is_white();
                let r = fastprobit::Reply::of(fx, sums, white);
                deep_count(&DEEP_STATS.leaves, 1);
                if fixed::checking() {
                    assert_eq!(sums, fx.full(&encoding::encode(&leaf)), "move sums differ from the full sums");
                    let (v, e) = r.estimate();
                    let full = fx.value(&sums, white);
                    assert!(full >= v - e && full <= v + e, "probit leaf value {full} outside {v} +- {e}");
                }
                r
            }
        }
    }

    /// hybrid18: a node one ply above the leaves in the fixed-point search of a flat probit, past its end-of-game and
    /// draw checks: its leaves in `negamax_fx`'s order (`m0` first if given, then `moves` or the moves generated now,
    /// sorted by `captures_first` if `sort`, skipping `m0` once), each estimated from the ln Phi table: `leaves_c`'s
    /// method, with `fastprobit::best`. The moves are generated only if `m0` doesn't prove a fail high.
    #[allow(clippy::too_many_arguments)]
    fn leaves_p(
        &mut self,
        fx: &FxLinear,
        pos: &Chess,
        acc: &Acc,
        hash: Zobrist64,
        ply: u32,
        alpha: f64,
        beta: f64,
        m0: Option<Move>,
        moves: Option<MoveList>,
        sort: bool,
    ) -> f64 {
        deep_count(&DEEP_STATS.nodes1, 1);
        let mut replies = std::mem::take(&mut self.preplies);
        replies.clear();
        let high = 'scan: {
            if let Some(m0) = m0 {
                let r = self.leaf_p(fx, pos, acc, hash, ply + 1, m0);
                if let Some(v) = fails_high(r.estimate(), beta) {
                    break 'scan Some(v);
                }
                replies.push(r);
            }
            let mut moves = moves.unwrap_or_else(|| pos.legal_moves());
            if sort {
                captures_first(&mut moves);
            }
            let mut skipped = m0.is_none();
            for m in moves {
                if !skipped && Some(m) == m0 {
                    skipped = true;
                    continue;
                }
                let r = self.leaf_p(fx, pos, acc, hash, ply + 1, m);
                if let Some(v) = fails_high(r.estimate(), beta) {
                    break 'scan Some(v);
                }
                replies.push(r);
            }
            None
        };
        let value = if let Some(v) = high {
            deep_count(&DEEP_STATS.fail_high, 1);
            v
        } else if let Some(v) = fails_low(replies.iter().map(|r| r.estimate()), alpha) {
            deep_count(&DEEP_STATS.fail_low, 1);
            v
        } else {
            fastprobit::best(fx, &mut replies)
        };
        self.preplies = replies;
        value
    }

    /// `root_values` at depth 2 with a flat probit in fixed point (hybrid8, see `fastprobit`): the same values, with
    /// only the replies that can be a move's smallest valued in full. Moves, hashes, repetitions and the ends of games
    /// are handled as `child_fx` and `negamax_fx` handle them.
    fn root_values_probit(&mut self, fx: &FxLinear, game: &Game) -> Vec<f64> {
        let pos = game.position();
        let acc = Acc::new(fx, pos);
        let moves = pos.legal_moves();
        fastprobit::count(&fastprobit::STATS.searches);
        if crate::compound::stats_on() {
            fastprobit::STATS.moves.fetch_add(moves.len() as u64, std::sync::atomic::Ordering::Relaxed);
        }
        let mut replies: Vec<fastprobit::Reply> = Vec::with_capacity(64);
        let values: Vec<f64> = moves
            .iter()
            .map(|&m| {
                // `child_fx` at depth 1, ply 1.
                let updated = pos.update_zobrist_hash(game.hash(), m, EnPassantMode::Legal);
                let mut next = pos.clone();
                next.play_unchecked(m);
                let next_hash = updated.unwrap_or_else(|| next.zobrist_hash(EnPassantMode::Legal));
                let since = self.since;
                if pos.is_irreversible(m) {
                    self.since = self.hashes.len();
                }
                self.hashes.push(next_hash);
                let next_acc = acc.child(fx, pos, &next);
                let value = self.ply1_probit(fx, &next, &next_acc, next_hash, &mut replies);
                self.hashes.pop();
                self.since = since;
                -value
            })
            .collect();
        if fixed::checking() {
            let full = self.root_values_fx_full(game, 2);
            let bits = |v: &[f64]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
            assert_eq!(bits(&values), bits(&full), "probit2 values {values:?} differ from the full search's {full:?}");
        }
        values
    }

    /// `negamax_fx` at depth 1 and ply 1 (a full window, so no cutoffs but at an infinite value) for `next`, whose
    /// accumulator is `acc`; `next`'s hash is the last of `hashes`.
    fn ply1_probit(
        &mut self,
        fx: &FxLinear,
        next: &Chess,
        acc: &Acc,
        hash: Zobrist64,
        replies: &mut Vec<fastprobit::Reply>,
    ) -> f64 {
        self.nodes += 1;
        let moves = next.legal_moves();
        if moves.is_empty() {
            if !next.is_check() {
                return 0.0;
            }
            return -(MATE - 1.0);
        }
        if next.is_insufficient_material() || next.halfmoves() >= 100 || self.repeated() {
            return 0.0;
        }
        replies.clear();
        for &m in &moves {
            // `child_fx` and then its leaf at depth 0, ply 2.
            let updated = next.update_zobrist_hash(hash, m, EnPassantMode::Legal);
            let mut leaf = next.clone();
            leaf.play_unchecked(m);
            let leaf_hash = updated.unwrap_or_else(|| leaf.zobrist_hash(EnPassantMode::Legal));
            let since = self.since;
            if next.is_irreversible(m) {
                self.since = self.hashes.len();
            }
            self.hashes.push(leaf_hash);
            self.nodes += 1;
            let reply = match self.terminal(&leaf, 2) {
                Some(end) => fastprobit::Reply::known(end.0),
                None => fastprobit::Reply::of(fx, acc.child_sums(fx, next, &leaf), leaf.turn().is_white()),
            };
            self.hashes.pop();
            self.since = since;
            replies.push(reply);
        }
        fastprobit::best(fx, replies)
    }

    /// `root_values` at depth 1 or 2 with a compound model: the same values (see `compound`). Moves, hashes,
    /// repetitions and the ends of games are handled as `child` and `negamax` handle them.
    fn root_values_compound(&mut self, c: &Compound, game: &Game, depth: u32) -> Vec<f64> {
        let pos = game.position();
        let moves = pos.legal_moves();
        c.count_search(moves.len());
        let mut buffers = c.buffers();
        let compound::Scratch { root, acc, ch, replies } = &mut *buffers;
        c.root_into(pos, root);
        let root = &*root;
        let values: Vec<f64> = moves
            .iter()
            .map(|&m| {
                let updated = pos.update_zobrist_hash(game.hash(), m, EnPassantMode::Legal);
                let mut next = pos.clone();
                next.play_unchecked(m);
                let next_hash = updated.unwrap_or_else(|| next.zobrist_hash(EnPassantMode::Legal));
                let since = self.since;
                if pos.is_irreversible(m) {
                    self.since = self.hashes.len();
                }
                self.hashes.push(next_hash);
                let value = if depth == 1 {
                    // `negamax` at depth 0, ply 1.
                    self.nodes += 1;
                    match self.terminal(&next, 1) {
                        Some(end) => end.0,
                        None => {
                            ch.fill(pos, &next, &root.enc, c.check);
                            c.value(root, ch, next.turn().is_white())
                        }
                    }
                } else {
                    self.ply1_compound(c, pos, root, &next, next_hash, (&mut *ch, &mut *acc, &mut *replies))
                };
                self.hashes.pop();
                self.since = since;
                -value
            })
            .collect();
        drop(buffers);
        if c.check {
            let float: Vec<f64> = moves
                .iter()
                .map(|&m| -self.child::<false>(pos, game.hash(), m, depth - 1, 1, f64::NEG_INFINITY, f64::INFINITY).0)
                .collect();
            let bits = |v: &[f64]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
            assert_eq!(bits(&values), bits(&float), "compound values {values:?} differ from float {float:?}");
        }
        values
    }

    /// `negamax` at depth 1 and ply 1 (a full window, so no cutoffs) for `next`, one move after `pos`, whose
    /// accumulator is `root`; `next`'s hash is the last of `hashes`. `scratch`: buffers.
    fn ply1_compound(
        &mut self,
        c: &Compound,
        pos: &Chess,
        root: &compound::Acc,
        next: &Chess,
        hash: Zobrist64,
        scratch: (&mut compound::Changes, &mut compound::Acc, &mut Vec<compound::Reply>),
    ) -> f64 {
        let (ch, acc, replies) = scratch;
        self.nodes += 1;
        let moves = next.legal_moves();
        if moves.is_empty() {
            if !next.is_check() {
                return 0.0;
            }
            return -(MATE - 1.0);
        }
        if next.is_insufficient_material() || next.halfmoves() >= 100 || self.repeated() {
            return 0.0;
        }
        ch.fill(pos, next, &root.enc, c.check);
        c.child_into(root, ch, acc);
        replies.clear();
        for &m in &moves {
            // `child` and then `negamax` at depth 0, ply 2.
            let updated = next.update_zobrist_hash(hash, m, EnPassantMode::Legal);
            let mut leaf = next.clone();
            leaf.play_unchecked(m);
            let leaf_hash = updated.unwrap_or_else(|| leaf.zobrist_hash(EnPassantMode::Legal));
            let since = self.since;
            if next.is_irreversible(m) {
                self.since = self.hashes.len();
            }
            self.hashes.push(leaf_hash);
            self.nodes += 1;
            let reply = match self.terminal(&leaf, 2) {
                Some(end) => compound::Reply::known(end.0),
                None => {
                    ch.fill(next, &leaf, &acc.enc, c.check);
                    c.reply(acc, ch, leaf.turn().is_white())
                }
            };
            self.hashes.pop();
            self.since = since;
            replies.push(reply);
        }
        // A reply's encoding, again, for those valued in float.
        let enc_of = |i: usize| {
            let mut leaf = next.clone();
            leaf.play_unchecked(moves[i]);
            compound::Changes::of(next, &leaf, &acc.enc).enc
        };
        c.best(replies, enc_of)
    }

    /// The outcome at the end of each legal move's principal variation, in
    /// `legal_moves()` order, searching `depth` plies deep as `root_values`
    /// does: the result if the line ends the game, else the model's own
    /// probabilities (as `predict` gives them, uncalibrated) for the
    /// position it ends in, as [white win, black win, draw]. At depth 1
    /// that is the prediction for the position after the move, unless the
    /// move ends the game.
    pub fn root_outcomes(&mut self, game: &Game, depth: u32) -> Vec<[f64; 3]> {
        assert!(depth >= 1, "search depth must be at least 1");
        let raw = std::mem::replace(&mut self.raw_outcomes, true);
        let pos = game.position();
        let outcomes = if let Some(fx) = self.fx {
            let acc = Acc::new(fx, pos);
            pos.legal_moves()
                .iter()
                .map(|&m| {
                    let pv = self.child_fx::<true>(fx, pos, &acc, game.hash(), m, depth - 1, 1, f64::NEG_INFINITY, f64::INFINITY).1;
                    self.resolve(pv)
                })
                .collect()
        } else {
            pos.legal_moves()
                .iter()
                .map(|&m| self.child::<true>(pos, game.hash(), m, depth - 1, 1, f64::NEG_INFINITY, f64::INFINITY).1)
                .collect()
        };
        self.raw_outcomes = raw;
        outcomes
    }

    /// The outcome expected from the game's position by searching `depth`
    /// plies deep: the result if the principal variation ends the game,
    /// else the calibrated probabilities of the model's prediction for the
    /// position it ends in, as [white win, black win, draw].
    pub fn expected_outcome(&mut self, game: &Game, depth: u32) -> [f64; 3] {
        if let Some(fx) = self.fx {
            let acc = Acc::new(fx, game.position());
            let pv = self.negamax_fx::<true>(fx, game.position(), &acc, game.hash(), depth, 0, f64::NEG_INFINITY, f64::INFINITY).1;
            return self.resolve(pv);
        }
        self.negamax::<true>(game.position(), game.hash(), depth, 0, f64::NEG_INFINITY, f64::INFINITY).1
    }

    /// Plays `m` and searches the resulting position; its value for the side
    /// to move after `m`, and with `OUTCOME` the outcome expected from it
    /// (see `negamax`).
    #[allow(clippy::too_many_arguments)]
    fn child<const OUTCOME: bool>(
        &mut self,
        pos: &Chess,
        hash: Zobrist64,
        m: Move,
        depth: u32,
        ply: u32,
        alpha: f64,
        beta: f64,
    ) -> (f64, [f64; 3]) {
        let updated = pos.update_zobrist_hash(hash, m, EnPassantMode::Legal);
        let mut next = pos.clone();
        next.play_unchecked(m);
        let next_hash = updated.unwrap_or_else(|| next.zobrist_hash(EnPassantMode::Legal));

        let since = self.since;
        if pos.is_irreversible(m) {
            self.since = self.hashes.len();
        }
        self.hashes.push(next_hash);
        let result = self.negamax::<OUTCOME>(&next, next_hash, depth, ply, alpha, beta);
        self.hashes.pop();
        self.since = since;
        result
    }

    /// The position's value for the side to move and, with `OUTCOME`, the
    /// outcome expected at the end of the principal variation (otherwise a
    /// draw, unused).
    fn negamax<const OUTCOME: bool>(
        &mut self,
        pos: &Chess,
        hash: Zobrist64,
        depth: u32,
        ply: u32,
        mut alpha: f64,
        beta: f64,
    ) -> (f64, [f64; 3]) {
        self.nodes += 1;
        // A leaf needs only to know whether the side to move can move (mate and stalemate), not the moves.
        let mut moves = if depth == 0 { MoveList::new() } else { pos.legal_moves() };
        if if depth == 0 { !fastpath::has_legal_move(pos) } else { moves.is_empty() } {
            if !pos.is_check() {
                return (0.0, DRAW);
            }
            return (-(MATE - ply as f64), if pos.turn().is_white() { BLACK_WINS } else { WHITE_WINS });
        }
        if pos.is_insufficient_material() || pos.halfmoves() >= 100 || self.repeated() {
            return (0.0, DRAW);
        }
        if depth == 0 {
            if OUTCOME && self.raw_outcomes && self.value == Value::LogOdds {
                // One prediction gives both the value and the outcome.
                let mut probs = [0.0; 3];
                self.model.predict(&encoding::encode(pos), &mut probs);
                let (win, loss) = if pos.turn().is_white() { (0, 1) } else { (1, 0) };
                return (log_odds(probs[win], probs[loss]), probs);
            }
            return (self.eval(pos), if OUTCOME { self.outcome(pos) } else { DRAW });
        }

        if !OUTCOME && depth == 1 && beta < f64::INFINITY && self.order1 {
            // Values only, and a cutoff is possible: likely refutations first (see the module's doc).
            captures_first(&mut moves);
        }
        if depth >= 2 {
            // Try the moves the model likes best first, for more cutoffs.
            let mut keyed: Vec<(f64, Move)> = moves
                .iter()
                .map(|&m| {
                    let mut next = pos.clone();
                    next.play_unchecked(m);
                    (-self.eval(&next), m)
                })
                .collect();
            keyed.sort_by(|a, b| b.0.total_cmp(&a.0));
            moves.clear();
            moves.extend(keyed.into_iter().map(|(_, m)| m));
        }

        let mut best = (f64::NEG_INFINITY, DRAW);
        for m in moves {
            let (value, outcome) = self.child::<OUTCOME>(pos, hash, m, depth - 1, ply + 1, -beta, -alpha);
            if -value > best.0 {
                best = (-value, outcome);
            }
            alpha = alpha.max(-value);
            if alpha >= beta {
                break;
            }
        }
        best
    }

    /// Whether the current position (the last hash) occurred before since
    /// the last irreversible move.
    fn repeated(&self) -> bool {
        let (current, earlier) = self.hashes[self.since..].split_last().unwrap();
        earlier.contains(current)
    }

    /// The probabilities of [white win, black win, draw] that the model
    /// predicts for a position: calibrated, or with `raw_outcomes` its own.
    fn outcome(&self, pos: &Chess) -> [f64; 3] {
        let mut out = [0.0; 3];
        if self.raw_outcomes {
            self.model.predict(&encoding::encode(pos), &mut out);
            return out;
        }
        self.model.predict_log(&encoding::encode(pos), &mut out);
        self.calibration.probs(&out)
    }

    /// `child` with the model in fixed point: `acc` is `pos`'s accumulator.
    #[allow(clippy::too_many_arguments)]
    fn child_fx<const OUTCOME: bool>(
        &mut self,
        fx: &FxLinear,
        pos: &Chess,
        acc: &Acc,
        hash: Zobrist64,
        m: Move,
        depth: u32,
        ply: u32,
        alpha: f64,
        beta: f64,
    ) -> (f64, Pv) {
        if !OUTCOME && self.movesums {
            // hybrid11 (values only): a reversible move's hash from the move, and a leaf without a hash where its
            // move is irreversible (it can't repeat an earlier position), its repetition checked without pushing it.
            let irreversible = pos.is_irreversible(m);
            if depth == 0 && self.value == Value::LogOdds {
                let mut next = pos.clone();
                next.play_unchecked(m);
                let repeated = !irreversible && self.hashes[self.since..].contains(&reversible_hash(pos, hash, m));
                self.nodes += 1;
                return match self.terminal_given(&next, ply, repeated) {
                    Some(end) => (end.0, Pv::Known(end.1)),
                    None => (fx.value(&acc.move_sums(fx, pos, m), next.turn().is_white()), Pv::Known(DRAW)),
                };
            }
            let next_hash = if irreversible {
                pos.update_zobrist_hash(hash, m, EnPassantMode::Legal)
            } else {
                Some(reversible_hash(pos, hash, m))
            };
            let mut next = pos.clone();
            next.play_unchecked(m);
            let next_hash = next_hash.unwrap_or_else(|| next.zobrist_hash(EnPassantMode::Legal));
            let since = self.since;
            if irreversible {
                self.since = self.hashes.len();
            }
            self.hashes.push(next_hash);
            let next_acc = acc.child(fx, pos, &next);
            let result = self.negamax_fx::<OUTCOME>(fx, &next, &next_acc, next_hash, depth, ply, alpha, beta);
            self.hashes.pop();
            self.since = since;
            return result;
        }
        let updated = pos.update_zobrist_hash(hash, m, EnPassantMode::Legal);
        let mut next = pos.clone();
        next.play_unchecked(m);
        let next_hash = updated.unwrap_or_else(|| next.zobrist_hash(EnPassantMode::Legal));

        let since = self.since;
        if pos.is_irreversible(m) {
            self.since = self.hashes.len();
        }
        self.hashes.push(next_hash);
        let result = if depth == 0 && !OUTCOME && self.value == Value::LogOdds {
            // A leaf whose outcome is not wanted needs only its sums (hybrid11: from the move).
            self.nodes += 1;
            match self.terminal(&next, ply) {
                Some(end) => (end.0, Pv::Known(end.1)),
                None => {
                    let sums = if self.movesums { acc.move_sums(fx, pos, m) } else { acc.child_sums(fx, pos, &next) };
                    (fx.value(&sums, next.turn().is_white()), Pv::Known(DRAW))
                }
            }
        } else {
            let next_acc = acc.child(fx, pos, &next);
            self.negamax_fx::<OUTCOME>(fx, &next, &next_acc, next_hash, depth, ply, alpha, beta)
        };
        self.hashes.pop();
        self.since = since;
        result
    }

    /// `negamax` with the model in fixed point: `acc` is `pos`'s accumulator.
    #[allow(clippy::too_many_arguments)]
    fn negamax_fx<const OUTCOME: bool>(
        &mut self,
        fx: &FxLinear,
        pos: &Chess,
        acc: &Acc,
        hash: Zobrist64,
        depth: u32,
        ply: u32,
        mut alpha: f64,
        beta: f64,
    ) -> (f64, Pv) {
        self.nodes += 1;
        if !OUTCOME && depth == 1 && beta < f64::INFINITY && self.order1 && self.staged && !pos.is_check() {
            // hybrid12: a legal capture first, before generating the moves (see the module's doc).
            if let Some(m0) = best_capture(pos) {
                if fixed::checking() {
                    check_best_capture(pos, m0);
                }
                // A legal move exists, so neither mate nor stalemate: the draw checks below, in the same order.
                if pos.is_insufficient_material() || pos.halfmoves() >= 100 || self.repeated() {
                    return (0.0, Pv::Known(DRAW));
                }
                self.staged_nodes += 1;
                return self.depth1_from::<OUTCOME>(fx, pos, acc, hash, ply, alpha, beta, m0, None, true);
            }
        }
        let mut moves = if depth == 0 { MoveList::new() } else { pos.legal_moves() };
        if if depth == 0 { !fastpath::has_legal_move(pos) } else { moves.is_empty() } {
            if !pos.is_check() {
                return (0.0, Pv::Known(DRAW));
            }
            return (-(MATE - ply as f64), Pv::Known(if pos.turn().is_white() { BLACK_WINS } else { WHITE_WINS }));
        }
        if pos.is_insufficient_material() || pos.halfmoves() >= 100 || self.repeated() {
            return (0.0, Pv::Known(DRAW));
        }
        if depth == 0 {
            return (self.eval_fx(fx, pos, acc), if OUTCOME { Pv::Leaf(acc.enc) } else { Pv::Known(DRAW) });
        }

        if !OUTCOME && depth == 1 && beta < f64::INFINITY && self.order1 {
            if self.staged {
                // hybrid12: the move captures_first would put first (the first of the smallest keys here), by one
                // scan; the full partition and sort only if it doesn't cut off. Without captures and promotions, the
                // first generated move, then the rest in their order: hybrid11's order exactly.
                let mut first = 0;
                let mut first_key = i32::MAX;
                let mut any = false;
                for (i, m) in moves.iter().enumerate() {
                    if m.is_capture() || m.is_promotion() {
                        let k = mvv_lva_key(m);
                        if k < first_key {
                            first_key = k;
                            first = i;
                            any = true;
                        }
                    }
                }
                let m0 = moves[first];
                return self.depth1_from::<OUTCOME>(fx, pos, acc, hash, ply, alpha, beta, m0, Some(moves), any);
            }
            // Values only, and a cutoff is possible: likely refutations first (see the module's doc).
            captures_first(&mut moves);
        }
        if !OUTCOME && depth == 1 && self.pbounds {
            // hybrid18: the leaves from the ln Phi table's bounds (`leaves_p`), in the order above.
            return (self.leaves_p(fx, pos, acc, hash, ply, alpha, beta, None, Some(moves), false), Pv::Known(DRAW));
        }
        if !OUTCOME && depth >= 2 && self.pbounds {
            // hybrid18: the keys below (`-eval_sums_turn` of the moves' sums, a stable sort) in their order from the ln
            // Phi table's estimates (`order_by_bounds`).
            let d = depth as usize;
            let mut lv = self.level(d);
            let white_next = !pos.turn().is_white();
            lv.pkeys.clear();
            lv.est.clear();
            for &m in &moves {
                let s = acc.move_sums(fx, pos, m);
                let (v, e) = fastprobit::estimate(fx, &s, white_next).unwrap_or_else(|| (fx.value(&s, white_next), 0.0));
                lv.pkeys.push((m, s));
                lv.est.push((-v, e));
            }
            {
                let Level { pkeys, est, order, ex, .. } = &mut lv;
                order_by_bounds(est, |i| -fx.value(&pkeys[i].1, white_next), order, ex);
            }
            if fixed::checking() {
                self.check_order_p(fx, white_next, &lv.pkeys, &lv.order);
            }
            let mut best = (f64::NEG_INFINITY, Pv::Known(DRAW));
            for k in 0..lv.order.len() {
                let m = lv.pkeys[lv.order[k]].0;
                let (value, outcome) = self.child_fx::<OUTCOME>(fx, pos, acc, hash, m, depth - 1, ply + 1, -beta, -alpha);
                if -value > best.0 {
                    best = (-value, outcome);
                }
                alpha = alpha.max(-value);
                if alpha >= beta {
                    break;
                }
            }
            self.put_level(d, lv);
            return best;
        }
        if depth >= 2 {
            // The keys read only the children's sums: with the values-only order, the sums alone (no encoding).
            let sums_only = !OUTCOME && self.order1;
            if sums_only && self.movesums {
                // hybrid11: the same keys from the moves' sums, without playing them, into a reused buffer. The
                // stable sort's first move is the first of the largest keys; the rest are sorted only if it doesn't
                // cut off. The same order, so the same nodes.
                let d = depth as usize;
                if self.keybufs.len() <= d {
                    self.keybufs.resize_with(d + 1, Vec::new);
                }
                let mut keyed = std::mem::take(&mut self.keybufs[d]);
                keyed.clear();
                let white_next = !pos.turn().is_white();
                keyed.extend(moves.iter().map(|&m| (-self.eval_sums_turn(fx, white_next, &acc.move_sums(fx, pos, m)), m)));
                let mut first = 0;
                for i in 1..keyed.len() {
                    if keyed[i].0.total_cmp(&keyed[first].0) == std::cmp::Ordering::Greater {
                        first = i;
                    }
                }
                let mut best = (f64::NEG_INFINITY, Pv::Known(DRAW));
                let (value, outcome) =
                    self.child_fx::<OUTCOME>(fx, pos, acc, hash, keyed[first].1, depth - 1, ply + 1, -beta, -alpha);
                if -value > best.0 {
                    best = (-value, outcome);
                }
                alpha = alpha.max(-value);
                if alpha < beta {
                    keyed.remove(first);
                    keyed.sort_by(|a, b| b.0.total_cmp(&a.0));
                    for &(_, m) in keyed.iter() {
                        let (value, outcome) = self.child_fx::<OUTCOME>(fx, pos, acc, hash, m, depth - 1, ply + 1, -beta, -alpha);
                        if -value > best.0 {
                            best = (-value, outcome);
                        }
                        alpha = alpha.max(-value);
                        if alpha >= beta {
                            break;
                        }
                    }
                }
                self.keybufs[d] = keyed;
                return best;
            } else {
                let mut keyed: Vec<(f64, Move)> = moves
                    .iter()
                    .map(|&m| {
                        let mut next = pos.clone();
                        next.play_unchecked(m);
                        if sums_only {
                            return (-self.eval_sums(fx, &next, &acc.child_sums(fx, pos, &next)), m);
                        }
                        let next_acc = acc.child(fx, pos, &next);
                        (-self.eval_fx(fx, &next, &next_acc), m)
                    })
                    .collect();
                keyed.sort_by(|a, b| b.0.total_cmp(&a.0));
                moves.clear();
                moves.extend(keyed.into_iter().map(|(_, m)| m));
            }
        }

        let mut best = (f64::NEG_INFINITY, Pv::Known(DRAW));
        for m in moves {
            let (value, outcome) = self.child_fx::<OUTCOME>(fx, pos, acc, hash, m, depth - 1, ply + 1, -beta, -alpha);
            if -value > best.0 {
                best = (-value, outcome);
            }
            alpha = alpha.max(-value);
            if alpha >= beta {
                break;
            }
        }
        best
    }

    /// hybrid12: a depth-1 node of the values-only fixed-point search with a cut window, past its end-of-game and draw
    /// checks, searched from the legal move `m0`: `m0` first, and only if it doesn't cut off, the other legal moves
    /// (`moves`, or generated now) in `captures_first`'s order (sorted only if `sort`), skipping `m0`. The loop is
    /// `negamax_fx`'s.
    #[allow(clippy::too_many_arguments)]
    fn depth1_from<const OUTCOME: bool>(
        &mut self,
        fx: &FxLinear,
        pos: &Chess,
        acc: &Acc,
        hash: Zobrist64,
        ply: u32,
        mut alpha: f64,
        beta: f64,
        m0: Move,
        moves: Option<MoveList>,
        sort: bool,
    ) -> (f64, Pv) {
        if !OUTCOME && self.pbounds {
            // hybrid18: the leaves from the ln Phi table's bounds, in the same order (`leaves_p`).
            return (self.leaves_p(fx, pos, acc, hash, ply, alpha, beta, Some(m0), moves, sort), Pv::Known(DRAW));
        }
        let mut best = (f64::NEG_INFINITY, Pv::Known(DRAW));
        let (value, outcome) = self.child_fx::<OUTCOME>(fx, pos, acc, hash, m0, 0, ply + 1, -beta, -alpha);
        if -value > best.0 {
            best = (-value, outcome);
        }
        alpha = alpha.max(-value);
        if alpha >= beta {
            return best;
        }
        let mut moves = moves.unwrap_or_else(|| pos.legal_moves());
        if sort {
            captures_first(&mut moves);
        }
        let mut skipped = false;
        for m in moves {
            if !skipped && m == m0 {
                skipped = true;
                continue;
            }
            let (value, outcome) = self.child_fx::<OUTCOME>(fx, pos, acc, hash, m, 0, ply + 1, -beta, -alpha);
            if -value > best.0 {
                best = (-value, outcome);
            }
            alpha = alpha.max(-value);
            if alpha >= beta {
                break;
            }
        }
        best
    }

    /// The value and outcome of a leaf (depth 0) if the game is over there, as `negamax` finds them.
    #[inline]
    fn terminal(&self, pos: &Chess, ply: u32) -> Option<(f64, [f64; 3])> {
        if !fastpath::has_legal_move(pos) {
            if !pos.is_check() {
                return Some((0.0, DRAW));
            }
            return Some((-(MATE - ply as f64), if pos.turn().is_white() { BLACK_WINS } else { WHITE_WINS }));
        }
        if pos.is_insufficient_material() || pos.halfmoves() >= 100 || self.repeated() {
            return Some((0.0, DRAW));
        }
        None
    }

    /// `terminal` with the repetition already known (hybrid11's leaves): the same checks in the same order.
    #[inline]
    fn terminal_given(&self, pos: &Chess, ply: u32, repeated: bool) -> Option<(f64, [f64; 3])> {
        if !fastpath::has_legal_move(pos) {
            if !pos.is_check() {
                return Some((0.0, DRAW));
            }
            return Some((-(MATE - ply as f64), if pos.turn().is_white() { BLACK_WINS } else { WHITE_WINS }));
        }
        if pos.is_insufficient_material() || pos.halfmoves() >= 100 || repeated {
            return Some((0.0, DRAW));
        }
        None
    }

    /// `eval` from a fixed-point accumulator.
    fn eval_fx(&self, fx: &FxLinear, pos: &Chess, acc: &Acc) -> f64 {
        self.eval_sums(fx, pos, &acc.sums)
    }

    /// `eval_fx` from the accumulator's sums, all it reads.
    #[inline]
    fn eval_sums(&self, fx: &FxLinear, pos: &Chess, sums: &fixed::Sums) -> f64 {
        self.eval_sums_turn(fx, pos.turn().is_white(), sums)
    }

    /// `eval_sums` for a position with `white_to_move`, all it reads of the position.
    #[inline]
    fn eval_sums_turn(&self, fx: &FxLinear, white_to_move: bool, sums: &fixed::Sums) -> f64 {
        match self.value {
            Value::LogOdds => fx.value(sums, white_to_move),
            Value::Score => {
                let (win, loss) = if white_to_move { (0, 1) } else { (1, 0) };
                let p = self.calibration.probs(&fx.logits(sums));
                log_odds(p[win] + p[2] / 2.0, p[loss] + p[2] / 2.0)
            }
        }
    }

    /// The outcome of a principal variation found by the fixed-point search: a leaf's is the float model's, as
    /// `outcome` and the depth-0 case of `negamax` give it.
    fn resolve(&self, pv: Pv) -> [f64; 3] {
        match pv {
            Pv::Known(outcome) => outcome,
            Pv::Leaf(enc) => {
                let mut out = [0.0; 3];
                if self.raw_outcomes {
                    self.model.predict(&enc, &mut out);
                    return out;
                }
                self.model.predict_log(&enc, &mut out);
                self.calibration.probs(&out)
            }
        }
    }

    /// The model's value of a position for the side to move.
    fn eval(&self, pos: &Chess) -> f64 {
        let mut out = [0.0; 3];
        let (win, loss) = if pos.turn().is_white() { (0, 1) } else { (1, 0) };
        match self.value {
            Value::LogOdds => {
                let enc = encoding::encode(pos);
                if self.direct {
                    if let Some(v) = self.model.log_odds_direct(&enc, pos.turn().is_white()) {
                        return v;
                    }
                }
                self.model.predict(&enc, &mut out);
                log_odds(out[win], out[loss])
            }
            Value::Score => {
                self.model.predict_log(&encoding::encode(pos), &mut out);
                let p = self.calibration.probs(&out);
                log_odds(p[win] + p[2] / 2.0, p[loss] + p[2] / 2.0)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classifier::Uniform;
    use shakmaty::uci::UciMove;

    fn game(moves: &[&str]) -> Game {
        let mut game = Game::default();
        for uci in moves {
            let m = uci.parse::<UciMove>().unwrap().to_move(game.position()).unwrap();
            game.play(m);
        }
        game
    }

    fn index_of(game: &Game, uci: &str) -> usize {
        let m = uci.parse::<UciMove>().unwrap().to_move(game.position()).unwrap();
        game.position().legal_moves().iter().position(|&l| l == m).unwrap()
    }

    fn value_of(game: &Game, values: &[f64], uci: &str) -> f64 {
        values[index_of(game, uci)]
    }

    #[test]
    fn finds_mate_in_one_and_sees_it_coming() {
        // Black to move can mate with Qh4#.
        let g = game(&["f2f3", "e7e5", "g2g4"]);
        let values = Search::new(&Uniform, &g).root_values(&g, 1);
        assert_eq!(value_of(&g, &values, "d8h4"), MATE - 1.0);
        assert!(values.iter().filter(|&&v| v > 1.0).count() == 1);

        // One move earlier, White should not allow it: g2g4 loses at depth 2.
        let g = game(&["f2f3", "e7e5"]);
        let values = Search::new(&Uniform, &g).root_values(&g, 2);
        assert_eq!(value_of(&g, &values, "g2g4"), -(MATE - 2.0));
        assert_eq!(value_of(&g, &values, "e2e4"), 0.0);
    }

    /// Predicts the same probabilities for every position.
    struct Constant([f64; 3]);

    impl Classifier for Constant {
        fn train(&mut self, _data: &[u8], _label: &[f64]) {}
        fn refresh(&mut self) {}
        fn predict(&self, _data: &[u8], out: &mut [f64]) {
            out.copy_from_slice(&self.0);
        }
        fn save(&self, _path: &std::path::Path) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn expects_the_outcome_of_the_principal_variation() {
        // Black to move mates with Qh4#: a black win, whatever the model says.
        let g = game(&["f2f3", "e7e5", "g2g4"]);
        assert_eq!(Search::new(&Uniform, &g).expected_outcome(&g, 1), BLACK_WINS);
        assert_eq!(Search::new(&Uniform, &g).expected_outcome(&g, 2), BLACK_WINS);

        // Otherwise the model's prediction at the end of the line, here
        // uncalibrated.
        let model = Constant([0.6, 0.2, 0.2]);
        let g = game(&["e2e4"]);
        let expected = Search::new(&model, &g).expected_outcome(&g, 2);
        assert!(expected.iter().zip([0.6, 0.2, 0.2]).all(|(a, b)| (a - b).abs() < 1e-12), "{expected:?}");
    }

    #[test]
    fn root_outcomes_end_in_results_or_raw_predictions() {
        // Black to move mates with Qh4#; every other move leads to the
        // model's own prediction, whatever the calibration.
        let model = Constant([0.6, 0.2, 0.2]);
        let g = game(&["f2f3", "e7e5", "g2g4"]);
        let calibration = Calibration { slope_winner: 2.0, ..Calibration::NONE };
        let outcomes = Search::new(&model, &g).valuing(Value::LogOdds, calibration).root_outcomes(&g, 1);
        let mate = index_of(&g, "d8h4");
        for (i, outcome) in outcomes.iter().enumerate() {
            assert_eq!(*outcome, if i == mate { BLACK_WINS } else { [0.6, 0.2, 0.2] });
        }

        // One move earlier, g2g4 lets Black mate: at depth 2 its line ends in
        // a black win, a quiet move's in the prediction two plies on.
        let g = game(&["f2f3", "e7e5"]);
        let outcomes = Search::new(&model, &g).root_outcomes(&g, 2);
        assert_eq!(outcomes[index_of(&g, "g2g4")], BLACK_WINS);
        assert_eq!(outcomes[index_of(&g, "e2e4")], [0.6, 0.2, 0.2]);
    }

    #[test]
    fn repetition_is_a_draw() {
        // The model thinks White is winning everywhere. After Nf3 Nf6 Ng1,
        // Black's Ng8 repeats the start position, which saves Black.
        let model = Constant([0.6, 0.2, 0.2]);
        let g = game(&["g1f3", "g8f6", "f3g1"]);
        let values = Search::new(&model, &g).root_values(&g, 1);
        assert_eq!(value_of(&g, &values, "f6g8"), 0.0);
        assert!((value_of(&g, &values, "e7e5") + 3f64.ln()).abs() < 1e-12);
    }

    #[test]
    fn captures_first_puts_captures_and_promotions_first_by_mvv_lva() {
        // White: Ra1, Nc3, pawn b7 (can promote, or capture on a8/c8); Black: queen a8, rook c8, pawn d5 (Nxd5).
        let pos: Chess = "q1r1k3/1P6/8/3p4/8/2N5/8/R3K3 w - - 0 1"
            .parse::<shakmaty::fen::Fen>()
            .unwrap()
            .into_position(shakmaty::CastlingMode::Standard)
            .unwrap();
        let mut moves = pos.legal_moves();
        let before: Vec<Move> = moves.to_vec();
        captures_first(&mut moves);
        let mut sorted = moves.to_vec();
        sorted.sort_by_key(|m| format!("{m:?}"));
        let mut expected = before.clone();
        expected.sort_by_key(|m| format!("{m:?}"));
        assert_eq!(sorted, expected, "the same moves");
        let k = moves.iter().take_while(|m| m.is_capture() || m.is_promotion()).count();
        assert_eq!(k, before.iter().filter(|m| m.is_capture() || m.is_promotion()).count());
        // bxa8=Q (queen victim plus a queen) first, then the other captures of the queen, ... and Nxd5 last.
        assert_eq!(moves[0].capture(), Some(Role::Queen));
        assert_eq!(moves[0].promotion(), Some(Role::Queen));
        assert!(moves[..k].iter().rev().find(|m| m.capture().is_some()).unwrap().capture() == Some(Role::Pawn));
    }

    /// hybrid10's values-only move order changes the nodes visited, not the values: every root value bit-equal to
    /// hybrid9's order's, float and fixed point, log-odds and score, at depths 1-4, in positions of random games;
    /// and the outcome search is untouched.
    #[test]
    fn move_order_keeps_root_values() {
        use crate::encoding::CATEGORIES;
        use crate::logreg::Logreg;
        use rand::rngs::StdRng;
        use rand::{RngExt, SeedableRng};
        let mut lr = Logreg::new(3, &CATEGORIES, 0.1);
        lr.randomize(5, 1.0);
        let fx = lr.fixed().unwrap();
        let calibration = Calibration { slope_winner: 1.3, ..Calibration::NONE };
        let mut rng = StdRng::seed_from_u64(11);
        let (mut nodes9, mut nodes10, mut cases) = (0u64, 0u64, 0);
        for g in 0..16 {
            let mut game = Game::default();
            for _ in 0..rng.random_range(4..70) {
                let moves = game.position().legal_moves();
                if moves.is_empty() {
                    break;
                }
                game.play(moves[rng.random_range(0..moves.len())]);
            }
            if game.position().legal_moves().is_empty() {
                continue;
            }
            for depth in 1..=(if g % 4 == 0 { 4 } else { 3 }) {
                for value in [Value::LogOdds, Value::Score] {
                    for fixed in [false, true] {
                        let search = |order1: bool| {
                            let mut s = Search::new(&lr, &game).valuing(value, calibration).with_fx(fixed.then_some(&fx));
                            s.order1 = order1;
                            s
                        };
                        let (mut a, mut b) = (search(false), search(true));
                        let (va, vb) = (a.root_values(&game, depth), b.root_values(&game, depth));
                        let bits = |v: &[f64]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
                        assert_eq!(bits(&va), bits(&vb), "game {g}, depth {depth}, {value:?}, fixed {fixed}");
                        if depth >= 3 {
                            nodes9 += a.nodes;
                            nodes10 += b.nodes;
                        }
                        let (mut a, mut b) = (search(false), search(true));
                        let (oa, ob) = (a.root_outcomes(&game, depth), b.root_outcomes(&game, depth));
                        assert_eq!(oa, ob);
                        assert_eq!(a.nodes, b.nodes, "the outcome search visits the same nodes");
                        cases += 1;
                    }
                }
            }
        }
        assert!(cases > 150, "{cases}");
        // A random model doesn't value material, so captures needn't refute here (real models' savings are measured
        // in fastbin/hybrid10/); this only checks that the order was in effect.
        assert_ne!(nodes9, nodes10, "the values-only order changed no node count");
    }

    #[test]
    fn move_sums_keep_root_values_and_nodes() {
        // hybrid11: with the children's sums from the moves (keys, leaves, lazy sort, reversible hashes), every root
        // value is bit-equal to hybrid10's code (movesums off) and the search visits the same nodes, at depths 1-4,
        // for log-odds and score values, a calibrated LR and a probit, from random-game positions (including ones
        // with repetitions available: the games shuffle pieces back and forth).
        use crate::encoding::CATEGORIES;
        use crate::logreg::Logreg;
        use crate::probit::{Probit, ProbitParams};
        use rand::rngs::StdRng;
        use rand::{RngExt, SeedableRng};
        let mut lr = Logreg::new(3, &CATEGORIES, 0.1);
        lr.randomize(5, 1.0);
        let fx_lr = lr.fixed().unwrap();
        // A probit trained on random positions labelled mostly by material (as fastprobit's tests make one).
        let mut probit = Probit::new(&CATEGORIES, ProbitParams::default());
        {
            let mut rng = StdRng::seed_from_u64(8);
            let mut pos = Chess::default();
            for _ in 0..20_000 {
                let moves = pos.legal_moves();
                if moves.is_empty() || pos.halfmoves() >= 100 {
                    pos = Chess::default();
                    continue;
                }
                pos.play_unchecked(moves[rng.random_range(0..moves.len())]);
                let mut label = [0.0; 3];
                let white = pos.board().white().count() as i32 - pos.board().black().count() as i32;
                let c = if white > 0 && rng.random_range(0..4) > 0 { 0 } else if white < 0 && rng.random_range(0..4) > 0 { 1 } else { rng.random_range(0..3) };
                label[c] = 1.0;
                probit.train(&encoding::encode(&pos), &label);
            }
        }
        let fx_probit = probit.fixed().unwrap().with_calibration(Calibration { slope_winner: 0.8, slope_decisive: 0.3, offsets: [0.4, -0.2] });
        let models: [(&(dyn Classifier + Sync), &FxLinear); 2] = [(&lr, &fx_lr), (&probit, &fx_probit)];
        let calibration = Calibration { slope_winner: 1.3, ..Calibration::NONE };
        let mut rng = StdRng::seed_from_u64(23);
        let (mut cases, mut nodes) = (0, 0u64);
        for g in 0..40 {
          for (model, fx) in models {
            let mut game = Game::default();
            let plies = rng.random_range(2..90);
            for i in 0..plies {
                let moves = game.position().legal_moves();
                if moves.is_empty() {
                    break;
                }
                // Every fourth game shuffles: quiet non-pawn moves where it can, so repetitions are reachable.
                let quiet: Vec<_> = moves.iter().copied().filter(|m| !m.is_capture() && m.role() != shakmaty::Role::Pawn).collect();
                let m = if g % 4 == 3 && i > 10 && !quiet.is_empty() { quiet[rng.random_range(0..quiet.len())] } else { moves[rng.random_range(0..moves.len())] };
                game.play(m);
            }
            if game.position().legal_moves().is_empty() {
                continue;
            }
            for depth in 1..=(if g % 5 == 0 { 4 } else { 3 }) {
                for value in [Value::LogOdds, Value::Score] {
                    let search = |movesums: bool| {
                        let mut s = Search::new(model, &game).valuing(value, calibration).with_fx(Some(fx));
                        s.movesums = movesums;
                        s
                    };
                    let (mut a, mut b) = (search(false), search(true));
                    let (va, vb) = (a.root_values(&game, depth), b.root_values(&game, depth));
                    let bits = |v: &[f64]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
                    assert_eq!(bits(&va), bits(&vb), "game {g}, depth {depth}, {value:?}");
                    assert_eq!(a.nodes, b.nodes, "game {g}, depth {depth}, {value:?}: the same nodes");
                    nodes += b.nodes;
                    let (mut a, mut b) = (search(false), search(true));
                    assert_eq!(a.root_outcomes(&game, depth), b.root_outcomes(&game, depth));
                    assert_eq!(a.nodes, b.nodes);
                    cases += 1;
                }
            }
          }
        }
        assert!(cases > 300 && nodes > 100_000, "{cases} {nodes}");
    }

    #[test]
    fn reversible_hash_equals_full_hash() {
        // hybrid11: for every reversible legal move of every position of 300 random games, the hash from the move
        // equals the hash of the played position; and shakmaty's incremental update, where it answers, agrees.
        use rand::rngs::StdRng;
        use rand::{RngExt, SeedableRng};
        let mut rng = StdRng::seed_from_u64(5);
        let (mut checked, mut declined) = (0u64, 0u64);
        for g in 0..300 {
            let mut pos = Chess::default();
            for i in 0..250 {
                let moves = pos.legal_moves();
                if moves.is_empty() {
                    break;
                }
                let hash: Zobrist64 = pos.zobrist_hash(EnPassantMode::Legal);
                for &m in &moves {
                    if pos.is_irreversible(m) {
                        continue;
                    }
                    let mut next = pos.clone();
                    next.play_unchecked(m);
                    let full: Zobrist64 = next.zobrist_hash(EnPassantMode::Legal);
                    assert_eq!(reversible_hash(&pos, hash, m), full, "{m:?}");
                    match pos.update_zobrist_hash(hash, m, EnPassantMode::Legal) {
                        Some(h) => assert_eq!(h, full),
                        None => declined += 1,
                    }
                    checked += 1;
                }
                let quiet: Vec<_> = moves.iter().copied().filter(|m| !m.is_capture()).collect();
                let m = if g % 2 == 1 && i > 20 && !quiet.is_empty() { quiet[rng.random_range(0..quiet.len())] } else { moves[rng.random_range(0..moves.len())] };
                pos.play_unchecked(m);
            }
        }
        assert!(checked > 500_000 && declined > 10_000, "{checked} {declined}");
    }

    fn fen(f: &str) -> Chess {
        f.parse::<shakmaty::fen::Fen>().unwrap().into_position(shakmaty::CastlingMode::Standard).unwrap()
    }

    /// What `best_capture` must give for a position not in check: None with a pawn of the side to move on its
    /// seventh rank or without legal normal captures; otherwise a legal normal capture with the smallest key among
    /// them.
    fn expect_best_capture(pos: &Chess) {
        let got = best_capture(pos);
        let us = pos.board().by_color(pos.turn());
        let seventh = Bitboard::from_rank(if pos.turn() == Color::White { Rank::Seventh } else { Rank::Second });
        let legal = pos.legal_moves();
        let caps: Vec<Move> = legal.iter().copied().filter(|m| matches!(m, Move::Normal { capture: Some(_), .. })).collect();
        if (pos.board().pawns() & us & seventh).any() || caps.is_empty() {
            assert_eq!(got, None, "{:?}", pos.board());
            return;
        }
        let m0 = got.unwrap_or_else(|| panic!("no capture found in {:?}", pos.board()));
        assert!(legal.contains(&m0), "illegal {m0:?} in {:?}", pos.board());
        let min = caps.iter().map(mvv_lva_key).min().unwrap();
        assert_eq!(mvv_lva_key(&m0), min, "{m0:?} in {:?}", pos.board());
    }

    #[test]
    fn best_capture_finds_a_legal_capture_of_the_smallest_key() {
        // hybrid12: on every position not in check of 400 random games, best_capture against shakmaty's legal moves.
        // The edge cases are counted so the test shows they occur: captures a pinned piece may not make, king
        // captures onto attacked squares (both pseudo-legal, illegal), king captures chosen, en passant available,
        // a pawn about to promote, and positions in check (skipped by the search).
        use rand::rngs::StdRng;
        use rand::{RngExt, SeedableRng};
        let mut rng = StdRng::seed_from_u64(31);
        let (mut checked, mut found, mut pinned_illegal, mut king_illegal, mut king_chosen, mut ep, mut seventh, mut in_check) =
            (0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64);
        for _ in 0..400 {
            let mut pos = Chess::default();
            for _ in 0..300 {
                let moves = pos.legal_moves();
                if moves.is_empty() || pos.halfmoves() >= 100 {
                    break;
                }
                if pos.is_check() {
                    in_check += 1;
                } else {
                    expect_best_capture(&pos);
                    checked += 1;
                    let board = pos.board();
                    let turn = pos.turn();
                    let us = board.by_color(turn);
                    if (board.pawns() & us & Bitboard::from_rank(if turn == Color::White { Rank::Seventh } else { Rank::Second })).any() {
                        seventh += 1;
                    } else if let Some(m0) = best_capture(&pos) {
                        found += 1;
                        king_chosen += (m0.role() == Role::King) as u64;
                    }
                    if moves.iter().any(|m| m.is_en_passant()) {
                        ep += 1;
                    }
                    // Pseudo-legal normal captures (any of our pieces attacking an enemy piece other than the king)
                    // that the legal moves lack.
                    for to in board.by_color(!turn) & !board.kings() {
                        for from in board.attacks_to(to, turn, board.occupied()) {
                            let role = board.role_at(from).unwrap();
                            let promotion = (role == Role::Pawn && to.rank() == if turn == Color::White { Rank::Eighth } else { Rank::First }).then_some(Role::Queen);
                            let m = Move::Normal { role, from, capture: board.role_at(to), to, promotion };
                            if !moves.contains(&m) {
                                if role == Role::King { king_illegal += 1 } else { pinned_illegal += 1 }
                            }
                        }
                    }
                }
                pos.play_unchecked(moves[rng.random_range(0..moves.len())]);
            }
        }
        assert!(checked > 50_000 && found > 10_000, "{checked} {found}");
        assert!(pinned_illegal > 100 && king_illegal > 100 && king_chosen > 100, "{pinned_illegal} {king_illegal} {king_chosen}");
        assert!(ep > 50 && seventh > 1_000 && in_check > 1_000, "{ep} {seventh} {in_check}");
        eprintln!("best_capture: {checked} positions checked, {found} with a capture found, {king_chosen} king captures \
            chosen; illegal pseudo-captures skipped: {pinned_illegal} by pinned pieces, {king_illegal} by the king; \
            {ep} with en passant, {seventh} with a pawn on the seventh, {in_check} in check");
    }

    #[test]
    fn best_capture_in_set_positions() {
        // A pinned bishop (Bd2, pinned by Bb4) may take the pinner, not the knight on e3: Bxb4.
        let pos = fen("4k3/8/8/8/1b6/4n3/3B4/4K3 w - - 0 1");
        expect_best_capture(&pos);
        assert_eq!(format!("{}", best_capture(&pos).unwrap().to_uci(shakmaty::CastlingMode::Standard)), "d2b4");
        // A pinned knight (Nd2, pinned by Bb4) may not take the pawn on f3; the king may not take the rook on f2
        // (Bh4 and Nd1 defend it) nor the knight on d1 (Nb2 defends it): no legal capture, though three are
        // pseudo-legal.
        let pos = fen("4k3/8/8/8/1b5b/5p2/Pn1N1r2/3nK3 w - - 0 1");
        assert!(!pos.is_check() && !pos.legal_moves().is_empty());
        expect_best_capture(&pos);
        assert_eq!(best_capture(&pos), None);
        // The same without the knight on b2: Kxd1 is the only legal capture.
        let pos = fen("4k3/8/8/8/1b5b/5p2/P2N1r2/3nK3 w - - 0 1");
        assert!(!pos.is_check());
        expect_best_capture(&pos);
        assert_eq!(format!("{}", best_capture(&pos).unwrap().to_uci(shakmaty::CastlingMode::Standard)), "e1d1");
        // Only an en passant capture: none found (left to the full generator).
        let pos = fen("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1");
        assert!(pos.legal_moves().iter().any(|m| m.is_en_passant()));
        assert_eq!(best_capture(&pos), None);
        // A pawn about to promote: none (left to the full generator), though Nxd5 is legal.
        let pos = fen("4k3/1P6/8/3p4/8/2N5/8/4K3 w - - 0 1");
        assert_eq!(best_capture(&pos), None);
        // MVV-LVA: the queen first, by the least valuable attacker (the pawn), though the rook can take it too.
        let pos = fen("4k3/8/8/2q5/1P6/8/8/2R1K3 w - - 0 1");
        expect_best_capture(&pos);
        assert_eq!(format!("{}", best_capture(&pos).unwrap().to_uci(shakmaty::CastlingMode::Standard)), "b4c5");
    }

    #[test]
    fn staged_first_move_keeps_root_values() {
        // hybrid12: with the staged first move (and the lazy first move), every root value is bit-equal to hybrid11's
        // code (staged off), at depths 1-5, for log-odds and score values, a random LR and a probit trained mostly on
        // material (so captures refute), from random-game positions (some shuffling pieces, so repetitions are
        // reachable); the outcome search is untouched (equal outcomes and nodes).
        use crate::encoding::CATEGORIES;
        use crate::logreg::Logreg;
        use crate::probit::{Probit, ProbitParams};
        use rand::rngs::StdRng;
        use rand::{RngExt, SeedableRng};
        let mut lr = Logreg::new(3, &CATEGORIES, 0.1);
        lr.randomize(5, 1.0);
        let fx_lr = lr.fixed().unwrap();
        let mut probit = Probit::new(&CATEGORIES, ProbitParams::default());
        {
            let mut rng = StdRng::seed_from_u64(8);
            let mut pos = Chess::default();
            for _ in 0..20_000 {
                let moves = pos.legal_moves();
                if moves.is_empty() || pos.halfmoves() >= 100 {
                    pos = Chess::default();
                    continue;
                }
                pos.play_unchecked(moves[rng.random_range(0..moves.len())]);
                let mut label = [0.0; 3];
                let white = pos.board().white().count() as i32 - pos.board().black().count() as i32;
                let c = if white > 0 && rng.random_range(0..4) > 0 { 0 } else if white < 0 && rng.random_range(0..4) > 0 { 1 } else { rng.random_range(0..3) };
                label[c] = 1.0;
                probit.train(&encoding::encode(&pos), &label);
            }
        }
        let fx_probit = probit.fixed().unwrap().with_calibration(Calibration { slope_winner: 0.8, slope_decisive: 0.3, offsets: [0.4, -0.2] });
        let models: [(&(dyn Classifier + Sync), &FxLinear); 2] = [(&lr, &fx_lr), (&probit, &fx_probit)];
        let calibration = Calibration { slope_winner: 1.3, ..Calibration::NONE };
        let mut rng = StdRng::seed_from_u64(29);
        let (mut cases, mut staged_nodes, mut nodes_off, mut nodes_on) = (0, 0u64, 0u64, 0u64);
        for g in 0..48 {
          for (model, fx) in models {
            let mut game = Game::default();
            let plies = rng.random_range(2..110);
            for i in 0..plies {
                let moves = game.position().legal_moves();
                if moves.is_empty() {
                    break;
                }
                let quiet: Vec<_> = moves.iter().copied().filter(|m| !m.is_capture() && m.role() != shakmaty::Role::Pawn).collect();
                let m = if g % 4 == 3 && i > 10 && !quiet.is_empty() { quiet[rng.random_range(0..quiet.len())] } else { moves[rng.random_range(0..moves.len())] };
                game.play(m);
            }
            if game.position().legal_moves().is_empty() {
                continue;
            }
            let max_depth = if g % 8 == 0 { 5 } else if g % 3 == 0 { 4 } else { 3 };
            for depth in 1..=max_depth {
                for value in [Value::LogOdds, Value::Score] {
                    let search = |staged: bool| {
                        let mut s = Search::new(model, &game).valuing(value, calibration).with_fx(Some(fx));
                        s.staged = staged;
                        s
                    };
                    let (mut a, mut b) = (search(false), search(true));
                    let (va, vb) = (a.root_values(&game, depth), b.root_values(&game, depth));
                    let bits = |v: &[f64]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
                    assert_eq!(bits(&va), bits(&vb), "game {g}, depth {depth}, {value:?}");
                    assert_eq!(a.staged_nodes, 0);
                    if depth <= 2 {
                        assert_eq!(b.staged_nodes, 0, "no cut window one ply above the leaves at depths 1-2");
                        assert_eq!(a.nodes, b.nodes);
                    }
                    staged_nodes += b.staged_nodes;
                    nodes_off += a.nodes;
                    nodes_on += b.nodes;
                    let (mut a, mut b) = (search(false), search(true));
                    assert_eq!(a.root_outcomes(&game, depth), b.root_outcomes(&game, depth));
                    assert_eq!(a.nodes, b.nodes, "the outcome search is untouched");
                    assert_eq!(b.staged_nodes, 0);
                    cases += 1;
                }
            }
          }
        }
        assert!(cases > 300 && staged_nodes > 10_000, "{cases} {staged_nodes}");
        eprintln!("staged_first_move_keeps_root_values: {cases} cases, {staged_nodes} staged nodes, nodes {nodes_off} off, {nodes_on} on");
    }

    #[test]
    fn windowed_root_values_are_exact_or_bounds() {
        // hybrid13: on both windowable paths (fixed point and the float negamax), at depths 2-5, for softmax windows
        // (K T below the best: K 45 and 1 at T 0.02, 0.05 and 0.001) and the argmax window (just below the best):
        // an exact move's value is bit-equal to `root_values`'s (up to the sign of a zero), a pruned move's bound is
        // at least its value and below the best value, and `root_value_of` gives `root_values`'s value. Positions
        // from random games (some shuffling pieces, so repetitions are reachable); a random LR and an LR trained
        // mostly on material, so captures refute.
        use crate::encoding::CATEGORIES;
        use crate::logreg::Logreg;
        use rand::rngs::StdRng;
        use rand::{RngExt, SeedableRng};
        let mut random_lr = Logreg::new(3, &CATEGORIES, 0.1);
        random_lr.randomize(5, 1.0);
        let mut material = Logreg::new(3, &CATEGORIES, 0.1);
        {
            let mut rng = StdRng::seed_from_u64(8);
            let mut pos = Chess::default();
            for _ in 0..20_000 {
                let moves = pos.legal_moves();
                if moves.is_empty() || pos.halfmoves() >= 100 {
                    pos = Chess::default();
                    continue;
                }
                pos.play_unchecked(moves[rng.random_range(0..moves.len())]);
                let mut label = [0.0; 3];
                let white = pos.board().white().count() as i32 - pos.board().black().count() as i32;
                let c = if white > 0 && rng.random_range(0..4) > 0 { 0 } else if white < 0 && rng.random_range(0..4) > 0 { 1 } else { rng.random_range(0..3) };
                label[c] = 1.0;
                material.train(&encoding::encode(&pos), &label);
            }
        }
        let fx_random = random_lr.fixed().unwrap();
        let fx_material = material.fixed().unwrap();
        let models: [(&(dyn Classifier + Sync), &FxLinear); 2] = [(&random_lr, &fx_random), (&material, &fx_material)];
        let mut rng = StdRng::seed_from_u64(31);
        let (mut cases, mut cut, mut moves_seen) = (0u64, 0u64, 0u64);
        for g in 0..40 {
            for (model, fx) in models {
                let mut game = Game::default();
                let plies = rng.random_range(2..110);
                for i in 0..plies {
                    let moves = game.position().legal_moves();
                    if moves.is_empty() {
                        break;
                    }
                    let quiet: Vec<_> = moves.iter().copied().filter(|m| !m.is_capture() && m.role() != shakmaty::Role::Pawn).collect();
                    let m = if g % 4 == 3 && i > 10 && !quiet.is_empty() { quiet[rng.random_range(0..quiet.len())] } else { moves[rng.random_range(0..moves.len())] };
                    game.play(m);
                }
                if game.position().legal_moves().is_empty() {
                    continue;
                }
                let max_depth = if g % 8 == 0 { 5 } else if g % 3 == 0 { 4 } else { 3 };
                for depth in 2..=max_depth {
                    for float in [false, true] {
                        let search = || {
                            let s = Search::new(model, &game);
                            if float { s } else { s.with_fx(Some(fx)) }
                        };
                        let full = search().root_values(&game, depth);
                        assert!(search().windowable(depth));
                        let windows: [(&str, Box<dyn Fn(f64) -> f64>); 7] = [
                            ("K45 T0.02", Box::new(|b: f64| b - 45.0 * 0.02)),
                            ("K45 T0.05", Box::new(|b: f64| b - 45.0 * 0.05)),
                            ("K45 T0.001", Box::new(|b: f64| b - 45.0 * 0.001)),
                            ("K1 T0.02", Box::new(|b: f64| b - 0.02)),
                            ("K1 T0.001", Box::new(|b: f64| b - 0.001)),
                            ("argmax", Box::new(f64::next_down)),
                            ("K40 T0.02", Box::new(|b: f64| b - 40.0 * 0.02)),
                        ];
                        for (name, below) in windows.iter() {
                            let (values, pruned) = search().root_values_windowed(&game, depth, below);
                            let best = (0..values.len()).filter(|&i| !pruned[i]).map(|i| values[i]).fold(f64::NEG_INFINITY, f64::max);
                            let fbest = full.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                            assert_eq!(best, fbest, "game {g}, depth {depth}, float {float}, {name}");
                            for i in 0..values.len() {
                                if pruned[i] {
                                    assert!(full[i] <= values[i] && values[i] < best, "game {g} depth {depth} {name} move {i}: bound {} value {}", values[i], full[i]);
                                    cut += 1;
                                } else {
                                    assert!(values[i].to_bits() == full[i].to_bits() || (values[i] == 0.0 && full[i] == 0.0), "game {g} depth {depth} {name} move {i}: {} against {}", values[i], full[i]);
                                }
                            }
                            moves_seen += values.len() as u64;
                            cases += 1;
                        }
                        for i in 0..full.len() {
                            let v = search().root_value_of(&game, depth, i);
                            assert_eq!(v.to_bits(), full[i].to_bits(), "game {g}, depth {depth}, move {i}");
                        }
                    }
                }
            }
        }
        assert!(cases > 1000 && cut > 10_000, "{cases} {cut}");
        eprintln!("windowed_root_values_are_exact_or_bounds: {cases} cases, {cut} of {moves_seen} root moves cut");
    }

    #[test]
    fn order_by_bounds_is_the_stable_sort_of_the_exact_keys() {
        // hybrid18: random exact keys with estimates within random bounds (many ties, wide and narrow bounds, exact
        // keys among them, signed zeros), against the stable sort by the exact keys.
        use rand::rngs::StdRng;
        use rand::{RngExt, SeedableRng};
        let mut rng = StdRng::seed_from_u64(77);
        let (mut order, mut ex) = (Vec::new(), Vec::new());
        let mut valued = 0usize;
        for case in 0..20_000 {
            let n = rng.random_range(1..40);
            let exact: Vec<f64> = (0..n)
                .map(|_| match rng.random_range(0..6) {
                    0 => 0.0,
                    1 => -0.0,
                    2 => (rng.random_range(-3..4) as f64) * 0.25,
                    _ => rng.random_range(-5.0..5.0),
                })
                .collect();
            let est: Vec<(f64, f64)> = exact
                .iter()
                .map(|&x| match rng.random_range(0..4) {
                    0 => (x, 0.0),
                    1 => {
                        let e = 10f64.powi(-rng.random_range(1..12));
                        (x + rng.random_range(-0.9..0.9) * e, e)
                    }
                    2 => (x + rng.random_range(-0.5..0.5) * 1e-3, 1e-3),
                    _ => (x, 1e-10),
                })
                .collect();
            let mut count = 0usize;
            order_by_bounds(&est, |i| {
                count += 1;
                exact[i]
            }, &mut order, &mut ex);
            assert_eq!(order, float_order(&exact), "case {case}: exact {exact:?} est {est:?}");
            valued += count;
            if case % 1000 == 0 {
                // Not finite: every key valued in full.
                let mut est2 = est.clone();
                est2[0] = (f64::NAN, 1.0);
                order_by_bounds(&est2, |i| exact[i], &mut order, &mut ex);
                assert_eq!(order, float_order(&exact));
            }
        }
        eprintln!("order_by_bounds: {valued} keys valued in full");
    }

    #[test]
    fn probit_bounds_keep_root_values_past_ply_2() {
        // hybrid18: a calibrated flat probit trained mostly on material (so captures refute), at depths 3-5, from
        // random-game positions (some shuffling pieces, so repetitions are reachable): the root values with the ln Phi
        // table's bounds are bit-equal to hybrid17's search (no bounds); with root cut windows (T 0.02's and the
        // argmax's) the same moves are cut, the exact values are the same and the bounds valid; `root_value_of`
        // gives the same values; the outcome search is untouched.
        use crate::encoding::CATEGORIES;
        use crate::probit::{Probit, ProbitParams};
        use rand::rngs::StdRng;
        use rand::{RngExt, SeedableRng};
        let mut probit = Probit::new(&CATEGORIES, ProbitParams::default());
        {
            let mut rng = StdRng::seed_from_u64(9);
            let mut pos = Chess::default();
            for _ in 0..20_000 {
                let moves = pos.legal_moves();
                if moves.is_empty() || pos.halfmoves() >= 100 {
                    pos = Chess::default();
                    continue;
                }
                pos.play_unchecked(moves[rng.random_range(0..moves.len())]);
                let mut label = [0.0; 3];
                let white = pos.board().white().count() as i32 - pos.board().black().count() as i32;
                let c = if white > 0 && rng.random_range(0..4) > 0 { 0 } else if white < 0 && rng.random_range(0..4) > 0 { 1 } else { rng.random_range(0..3) };
                label[c] = 1.0;
                probit.train(&encoding::encode(&pos), &label);
            }
        }
        let fxs = [
            probit.fixed().unwrap().with_calibration(Calibration { slope_winner: 0.8, slope_decisive: 0.3, offsets: [0.4, -0.2] }),
            probit.fixed().unwrap(),
        ];
        let bits = |v: &[f64]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
        let mut rng = StdRng::seed_from_u64(33);
        let (mut cases, mut cut) = (0u64, 0u64);
        for g in 0..36 {
            let fx = &fxs[g % 2];
            let mut game = Game::default();
            let plies = rng.random_range(2..110);
            for i in 0..plies {
                let moves = game.position().legal_moves();
                if moves.is_empty() {
                    break;
                }
                let quiet: Vec<_> = moves.iter().copied().filter(|m| !m.is_capture() && m.role() != shakmaty::Role::Pawn).collect();
                let m = if g % 4 == 3 && i > 10 && !quiet.is_empty() { quiet[rng.random_range(0..quiet.len())] } else { moves[rng.random_range(0..moves.len())] };
                game.play(m);
            }
            if game.position().legal_moves().is_empty() {
                continue;
            }
            let max_depth = if g % 6 == 0 { 5 } else { 4 };
            for depth in 3..=max_depth {
                let search = |bounds: bool| {
                    let mut s = Search::new(&probit, &game).with_fx(Some(fx));
                    s.allow_bounds = bounds;
                    s
                };
                let (mut a, mut b) = (search(true), search(false));
                let (va, vb) = (a.root_values(&game, depth), b.root_values(&game, depth));
                assert_eq!(bits(&va), bits(&vb), "game {g}, depth {depth}");
                let best = vb.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                let windows: [&dyn Fn(f64) -> f64; 2] = [&|b: f64| b - 40.0 * 0.02, &f64::next_down];
                for below in windows {
                    let (cv, cp) = search(true).root_values_windowed(&game, depth, below);
                    let (fv, fp) = search(false).root_values_windowed(&game, depth, below);
                    assert_eq!(cp, fp, "game {g}, depth {depth}: cuts");
                    for k in 0..cv.len() {
                        if cp[k] {
                            assert!(vb[k] <= cv[k] && cv[k] < best, "move {k}: bound {} value {}", cv[k], vb[k]);
                            cut += 1;
                        } else {
                            assert_eq!(cv[k].to_bits(), fv[k].to_bits(), "move {k}");
                        }
                    }
                }
                for k in 0..va.len() {
                    assert_eq!(search(true).root_value_of(&game, depth, k).to_bits(), vb[k].to_bits(), "root_value_of {k}");
                }
                let (mut a, mut b) = (search(true), search(false));
                assert_eq!(a.root_outcomes(&game, depth), b.root_outcomes(&game, depth));
                assert_eq!(a.nodes, b.nodes, "the outcome search is untouched");
                cases += 1;
            }
        }
        assert!(cases > 60 && cut > 500, "{cases} {cut}");
        eprintln!("probit_bounds_keep_root_values_past_ply_2: {cases} cases, {cut} cut root moves");
    }

    #[test]
    fn windowable_follows_root_values_paths() {
        // hybrid13: the compound path and fastprobit keep their full searches, depth 1 too.
        use crate::encoding::CATEGORIES;
        use crate::logreg::Logreg;
        use crate::probit::{Probit, ProbitParams};
        let lr = Logreg::new(3, &CATEGORIES, 0.1);
        let fx_lr = lr.fixed().unwrap();
        let probit = Probit::new(&CATEGORIES, ProbitParams::default());
        let fx_probit = probit.fixed().unwrap();
        let game = Game::default();
        assert!(!Search::new(&lr, &game).with_fx(Some(&fx_lr)).windowable(1));
        assert!(Search::new(&lr, &game).with_fx(Some(&fx_lr)).windowable(2));
        assert!(Search::new(&lr, &game).windowable(3));
        assert_eq!(Search::new(&probit, &game).with_fx(Some(&fx_probit)).windowable(2), !fastprobit::enabled());
        assert!(Search::new(&probit, &game).with_fx(Some(&fx_probit)).windowable(3));
        // Naive Bayes keeps the full root search, calibrated too (the flag survives `with_calibration`).
        let mut nb = crate::cnbp::Cnbp::new(3, &CATEGORIES, 1.0, true);
        nb.train(&encoding::encode(&Chess::default()), &[1.0, 0.0, 0.0]);
        nb.refresh();
        let fx_nb = nb.fixed().expect("a fresh NB with use_probs in fixed point");
        assert!(!Search::new(&nb, &game).with_fx(Some(&fx_nb)).windowable(3));
        let fx_nb_cal = fx_nb.clone().with_calibration(Calibration { slope_winner: 0.8, ..Calibration::NONE });
        assert!(!Search::new(&nb, &game).with_fx(Some(&fx_nb_cal)).windowable(4));
        assert!(lr.fixed().unwrap().with_calibration(Calibration::NONE).root_windows());
    }
}
