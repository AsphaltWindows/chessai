//! play2: the play page's deep mode (`play-serve`'s op=deep): iterative deepening over the tournament's search
//! (`Search::deep_iteration`) with a transposition table (`DeepTable`), a time limit and a stop from outside. Each
//! completed iteration chooses as a tournament player of its depth chooses: the log-odds argmax, ties broken by the
//! same draw (derive_seed(seed, plies played, 0)). The move played is the deepest completed iteration's; an iteration
//! the limit cuts off is dropped. No quiescence or extensions: the search is the tournament's at every depth.

use std::time::{Duration, Instant};

use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use shakmaty::{Move, Position};

use crate::fixed::{self, FxLinear};
use crate::game::Game;
use crate::player::{Player, Policy};
use crate::search::{DeepCtx, TtStats, DeepTable, MATE, Search, TtMode};
use crate::selfplay;

/// The deepest iteration a deep search runs (the user, 11-12 ~18:14: "let's say up to 10").
pub const MAX_DEPTH: u32 = 10;

/// A deep search's settings.
#[derive(Clone, Debug)]
pub struct DeepParams {
    pub max_depth: u32,
    /// When to stop (None: no limit, every iteration up to `max_depth`).
    pub deadline: Option<Instant>,
    /// The game's seed and the plies played: the tie-break's draw, as in the tournament.
    pub seed: u64,
    pub plies: u64,
    pub tt: TtMode,
    /// Iterative deepening; false: only `max_depth` (for measurements).
    pub iterate: bool,
    /// Whether an iteration predicted to end past the deadline is not started (`next_ms`).
    pub predict: bool,
    /// Whether the first iteration may be stopped too (the ponder's).
    pub first_abortable: bool,
}

/// One completed iteration.
#[derive(Clone, Debug)]
pub struct DeepIter {
    pub depth: u32,
    /// The chosen move, its index in `legal_moves()`, its exact value, and how many moves tied with it.
    pub index: usize,
    pub mv: Move,
    pub value: f64,
    pub tied: usize,
    pub nodes: u64,
    /// The iteration's time, and the time from its search's start to its end (ms).
    pub ms: f64,
    pub at_ms: f64,
    /// The opponent's best reply after the move (a `search::mv_code`; 0 if none).
    pub reply: u16,
    /// The moves whose values are exact (the best and those searched before it was found): index and value.
    pub exact: Vec<(usize, f64)>,
    /// Whether the iteration was searched while pondering (on the opponent's time).
    pub pondered: bool,
}

/// What a deep search leaves: its completed iterations, the root order for the next, and the iteration it cut off with
/// the time it spent in it (ms): a ponder's, whose work the table keeps for a hit.
#[derive(Clone, Debug, Default)]
pub struct DeepState {
    pub iters: Vec<DeepIter>,
    pub order: Vec<usize>,
    pub partial: Option<(u32, f64)>,
}

/// A deep search's result.
pub struct DeepOutcome {
    pub state: DeepState,
    /// Why it stopped: max depth, mate found, one legal move, time, time (predicted), stopped.
    pub stop: &'static str,
    /// This search's nodes and time (ms; a resumed search's counts only its own).
    pub nodes: u64,
    pub ms: f64,
    /// The iteration it cut off, if one.
    pub cut_depth: Option<u32>,
    pub stats: TtStats,
    /// Whether it searched with the fixed-point model (a flat model with CHESSAI_FX on, as the tournament).
    pub fx: bool,
}

/// The player's model in fixed point, as the tournament's search takes it (selfplay::fixed_of).
fn fx_of(player: Player) -> Option<FxLinear> {
    if fixed::mode() != fixed::Mode::On || player.policy == Policy::Rand {
        return None;
    }
    player.model.fixed_linear()
}

/// The predicted time of the next iteration (ms): the last one's times its growth over the one before (between 1.5
/// and 30; 6 if there is no earlier one or it took under a millisecond).
pub fn next_ms(iters: &[DeepIter]) -> f64 {
    let Some(last) = iters.last() else { return 0.0 };
    let growth = match iters.len() {
        n if n >= 2 && iters[n - 2].ms >= 1.0 => (last.ms / iters[n - 2].ms).clamp(1.5, 30.0),
        _ => 6.0,
    };
    last.ms.max(0.1) * growth
}

/// The deep search of the game's position (which has a legal move) for `player` (its model, value and calibration;
/// the log-odds argmax), from `resume`'s iterations if given (a ponder's, same position), reporting each completed
/// iteration to `progress`. `stop` is polled every 1,024 nodes once a move is known.
pub fn deep_search(
    player: Player,
    game: &Game,
    p: &DeepParams,
    mut table: Option<&mut DeepTable>,
    stop: &dyn Fn() -> bool,
    resume: Option<DeepState>,
    progress: &mut dyn FnMut(&DeepIter),
) -> DeepOutcome {
    let fx = fx_of(player);
    let mut search = Search::new(player.model, game).valuing(player.value, player.calibration).with_fx(fx.as_ref());
    let t0 = Instant::now();
    let moves: Vec<Move> = game.position().legal_moves().to_vec();
    assert!(!moves.is_empty(), "the deep search needs a legal move");
    let mut state = match resume {
        Some(s) if s.order.len() == moves.len() => s,
        _ => DeepState { iters: Vec::new(), order: search.deep_root_order(game, &moves), partial: None },
    };
    if let Some(t) = table.as_deref_mut() {
        t.next_generation();
    }
    let mut dc = DeepCtx::new(table, p.tt, p.deadline, stop);
    let first = state.iters.last().map_or(1, |it| it.depth + 1);
    let depths: Vec<u32> = if p.iterate {
        (first..=p.max_depth).collect()
    } else if first <= p.max_depth {
        vec![p.max_depth]
    } else {
        Vec::new()
    };
    let mut why = "max depth";
    let mut cut_depth = None;
    for d in depths {
        if let Some(last) = state.iters.last() {
            if moves.len() == 1 {
                why = "one legal move";
                break;
            }
            if last.value.abs() > MATE / 2.0 {
                why = "mate found";
                break;
            }
            if stop() {
                why = "stopped";
                break;
            }
            if let Some(deadline) = p.deadline {
                let now = Instant::now();
                if now >= deadline {
                    why = "time";
                    break;
                }
                // A ponder's cut-off iteration (resumed on a hit) has done part of the next one's work already.
                let done = state.partial.filter(|&(pd, _)| pd == d).map_or(0.0, |(_, ms)| ms);
                let next = (next_ms(&state.iters) - done).max(0.0);
                if p.predict && now + Duration::from_secs_f64(next / 1000.0) > deadline {
                    why = "time (the next iteration would not end in it)";
                    break;
                }
            }
        }
        dc.abortable = !state.iters.is_empty() || p.first_abortable;
        let t1 = Instant::now();
        let n1 = search.nodes;
        let Some(root) = search.deep_iteration(&mut dc, game, &moves, &state.order, d) else {
            why = if stop() { "stopped" } else { "time" };
            cut_depth = Some(d);
            let before = state.partial.filter(|&(pd, _)| pd == d).map_or(0.0, |(_, ms)| ms);
            state.partial = Some((d, before + t1.elapsed().as_secs_f64() * 1000.0));
            break;
        };
        state.partial = None;
        let n = moves.len();
        let best = (0..n).filter(|&i| !root.pruned[i]).map(|i| root.values[i]).fold(f64::NEG_INFINITY, f64::max);
        let tied: Vec<usize> = (0..n).filter(|&i| !root.pruned[i] && root.values[i] == best).collect();
        let mut rng = StdRng::seed_from_u64(selfplay::derive_seed(p.seed, p.plies, 0));
        let index = tied[rng.random_range(0..tied.len())];
        // The next iteration's order: by this one's values (bounds are below the best), the chosen move first.
        let mut order = state.order.clone();
        order.sort_by(|&a, &b| root.values[b].total_cmp(&root.values[a]));
        if let Some(k) = order.iter().position(|&i| i == index) {
            order.remove(k);
            order.insert(0, index);
        }
        state.order = order;
        let it = DeepIter {
            depth: d,
            index,
            mv: moves[index],
            value: root.values[index],
            tied: tied.len(),
            nodes: search.nodes - n1,
            ms: t1.elapsed().as_secs_f64() * 1000.0,
            at_ms: t0.elapsed().as_secs_f64() * 1000.0,
            reply: root.replies[index],
            exact: (0..n).filter(|&i| !root.pruned[i]).map(|i| (i, root.values[i])).collect(),
            pondered: false,
        };
        progress(&it);
        state.iters.push(it);
    }
    DeepOutcome {
        state,
        stop: why,
        nodes: search.nodes,
        ms: t0.elapsed().as_secs_f64() * 1000.0,
        cut_depth,
        stats: dc.stats,
        fx: fx.is_some(),
    }
}

/// The opponent's predicted reply after `game`'s position (the position after the deep search's move): `reply` (the
/// search's) if legal there, else the table's best move for the position, else the reply the model values best at
/// depth 1. None if the game has no legal move there.
pub fn predict_reply(player: Player, game: &Game, reply: u16, table: Option<&DeepTable>) -> Option<Move> {
    let moves: Vec<Move> = game.position().legal_moves().to_vec();
    if moves.is_empty() {
        return None;
    }
    if let Some(m) = crate::search::find_code(&moves, reply) {
        return Some(m);
    }
    if let Some(code) = table.and_then(|t| t.best_move(game.hash().0)) {
        if let Some(m) = crate::search::find_code(&moves, code) {
            return Some(m);
        }
    }
    let fx = fx_of(player);
    let search = Search::new(player.model, game).valuing(player.value, player.calibration).with_fx(fx.as_ref());
    search.deep_root_order(game, &moves).first().map(|&i| moves[i])
}
