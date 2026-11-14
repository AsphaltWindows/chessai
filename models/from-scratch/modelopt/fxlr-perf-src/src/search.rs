//! Alpha-beta search over a classifier's evaluation.
//!
//! Positions are valued for the side to move by the model's log-odds,
//! ln P(side to move wins) - ln P(it loses), in nats. Positions where the
//! game is over by the rules are valued exactly: checkmate is a loss,
//! stalemate, insufficient material and the 50-move rule are draws (0). As
//! engines usually do, a position that repeats one reached since the last
//! irreversible move (in the game or along the search path) counts as a
//! draw.

use shakmaty::zobrist::Zobrist64;
use shakmaty::{Chess, EnPassantMode, Move, MoveList, Position};

use crate::calibration::Calibration;
use crate::classifier::Classifier;
use crate::encoding;
use crate::fastpath;
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
        }
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
        let pos = game.position();
        if let Some(fx) = self.fx {
            let acc = Acc::new(fx, pos);
            return pos
                .legal_moves()
                .iter()
                .map(|&m| {
                    -self.child_fx::<false>(fx, pos, &acc, game.hash(), m, depth - 1, 1, f64::NEG_INFINITY, f64::INFINITY).0
                })
                .collect();
        }
        pos.legal_moves()
            .iter()
            .map(|&m| -self.child::<false>(pos, game.hash(), m, depth - 1, 1, f64::NEG_INFINITY, f64::INFINITY).0)
            .collect()
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
        let updated = pos.update_zobrist_hash(hash, m, EnPassantMode::Legal);
        let mut next = pos.clone();
        next.play_unchecked(m);
        let next_hash = updated.unwrap_or_else(|| next.zobrist_hash(EnPassantMode::Legal));
        let next_acc = acc.child(fx, pos, &next);

        let since = self.since;
        if pos.is_irreversible(m) {
            self.since = self.hashes.len();
        }
        self.hashes.push(next_hash);
        let result = self.negamax_fx::<OUTCOME>(fx, &next, &next_acc, next_hash, depth, ply, alpha, beta);
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

        if depth >= 2 {
            let mut keyed: Vec<(f64, Move)> = moves
                .iter()
                .map(|&m| {
                    let mut next = pos.clone();
                    next.play_unchecked(m);
                    let next_acc = acc.child(fx, pos, &next);
                    (-self.eval_fx(fx, &next, &next_acc), m)
                })
                .collect();
            keyed.sort_by(|a, b| b.0.total_cmp(&a.0));
            moves.clear();
            moves.extend(keyed.into_iter().map(|(_, m)| m));
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

    /// `eval` from a fixed-point accumulator.
    #[inline(never)]
    fn eval_fx(&self, fx: &FxLinear, pos: &Chess, acc: &Acc) -> f64 {
        match self.value {
            Value::LogOdds => fx.value(&acc.sums, pos.turn().is_white()),
            Value::Score => {
                let (win, loss) = if pos.turn().is_white() { (0, 1) } else { (1, 0) };
                let p = self.calibration.probs(&fx.logits(&acc.sums));
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
}
