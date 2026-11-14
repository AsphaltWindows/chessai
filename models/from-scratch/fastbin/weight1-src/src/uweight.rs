//! Weighting training positions by the model's own uncertainty (weight1, 2026-11-12; analyst 6.73).
//!
//! A position the model already calls decided teaches it little about ranking moves, so its example counts less and
//! an undecided one more, with the mean weight kept at 1 so that the learning rate is unchanged on average:
//!
//! - s, the model's expected score for white, P(white wins) + P(draw) / 2, from the prediction the learner makes of
//!   the position before learning from it (the one its update already computes: online, no snapshot);
//! - u = 4 s (1 - s), 1 at s = 0.5 and 0 at a decided position, clipped to [floor, 1] so that a confidently wrong
//!   model still sees the example that corrects it;
//! - m, the running mean of u over the model's own examples with memory N: n += 1, m += (u - m) / min(n, N) (the
//!   example's own u is in the mean it is divided by, so the first example's weight is exactly 1);
//! - the weight w = u / m, at most cap.
//!
//! The learners take w through their own paths (see `UTrain`): a probit raises the example's likelihood to the power
//! alpha w (`--probit-alpha`'s fractional update; its draw margin's gradient step and dynamics are unchanged); a
//! logistic regression steps by w times the gradient while AdaGrad's sums take the unweighted squared gradient (else
//! AdaGrad would normalise a steady weight away). Flat probit and lr only; trees refuse it.
//!
//! m and n live in the training state, `{kind}N.state` beside the model (`state`'s format, kind "uweight"), which
//! `train` writes after its last round and reads from beside --start's model file; model files are unchanged, so
//! play is too. Without the option (`--weight-by-uncertainty none`) nothing here runs and training is unchanged.

use std::io;
use std::path::Path;

use crate::classifier::Classifier;
use crate::encoding::Encoded;
use crate::fixed::FxLinear;
use crate::logreg::Logreg;
use crate::probit::Probit;
use crate::state::{Reader, Writer};

/// The weighting's options (`--weight-floor`, `--weight-cap`, `--weight-memory`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UWeightParams {
    /// The smallest u (raw u below it is raised to it).
    pub floor: f64,
    /// The largest weight.
    pub cap: f64,
    /// The running mean's memory N, in examples.
    pub memory: u64,
}

impl UWeightParams {
    /// Checks the options: 0 < floor <= 1, cap >= 1, memory >= 1.
    pub fn check(&self) -> Result<(), String> {
        if !(self.floor > 0.0 && self.floor <= 1.0) {
            return Err(format!("--weight-floor must be in (0, 1], not {}", self.floor));
        }
        if !(self.cap >= 1.0) {
            return Err(format!("--weight-cap must be at least 1, not {}", self.cap));
        }
        if self.memory == 0 {
            return Err("--weight-memory must be at least 1".into());
        }
        Ok(())
    }

    /// The options as the state file records them (checked on load).
    fn describe(&self) -> String {
        format!("score floor {:e} cap {:e} memory {}", self.floor, self.cap, self.memory)
    }
}

/// One round's weights, for the round's report line.
#[derive(Clone, Copy, Debug, Default)]
struct RoundStats {
    examples: u64,
    sum_w: f64,
    sum_u: f64,
    at_floor: u64,
    at_cap: u64,
}

/// The weighting's running state (see the module documentation).
#[derive(Clone, Debug)]
pub struct UWeight {
    params: UWeightParams,
    /// The running mean of u.
    pub(crate) m: f64,
    /// The examples seen (all rounds, all runs of `train` that carried the state on).
    pub(crate) n: u64,
    round: RoundStats,
}

const KIND: &str = "uweight";

impl UWeight {
    pub fn new(params: UWeightParams) -> Self {
        UWeight { params, m: 0.0, n: 0, round: RoundStats::default() }
    }

    pub fn params(&self) -> UWeightParams {
        self.params
    }

    /// The weight of an example whose expected score for white the model puts at `s`, and the running mean updated
    /// with it.
    pub fn weight(&mut self, s: f64) -> f64 {
        let p = self.params;
        let raw = 4.0 * s * (1.0 - s);
        // A NaN prediction (never seen) counts as undecided rather than poisoning the mean.
        let u = if raw.is_nan() { 1.0 } else { raw.clamp(p.floor, 1.0) };
        self.n += 1;
        self.m += (u - self.m) / self.n.min(p.memory) as f64;
        let ratio = u / self.m;
        let w = ratio.min(p.cap);
        let r = &mut self.round;
        r.examples += 1;
        r.sum_w += w;
        r.sum_u += u;
        r.at_floor += (raw < p.floor) as u64;
        r.at_cap += (ratio > p.cap) as u64;
        w
    }

    /// The round's report line (then a new round's counts start).
    pub fn round_report(&mut self) -> String {
        let r = std::mem::take(&mut self.round);
        let k = r.examples.max(1) as f64;
        format!(
            "weights by uncertainty (score): {} examples, mean w {:.4}, mean u {:.4}, at floor {:.2}%, at cap {:.2}%, \
             running mean m {:.4} (n {})",
            r.examples,
            r.sum_w / k,
            r.sum_u / k,
            100.0 * r.at_floor as f64 / k,
            100.0 * r.at_cap as f64 / k,
            self.m,
            self.n
        )
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        let mut w = Writer::new(KIND);
        w.str(&self.params.describe());
        w.u64(self.n);
        w.f64(self.m);
        w.save(path)
    }

    /// A state saved by `save` with the same options.
    pub fn load(path: &Path, params: UWeightParams) -> io::Result<Self> {
        let data = std::fs::read(path)?;
        let mut r = Reader::new(&data, KIND)?;
        r.expect_str("weighting options", &params.describe())?;
        let n = r.u64()?;
        let m = r.f64()?;
        r.finish()?;
        Ok(UWeight { params, m, n, round: RoundStats::default() })
    }

    /// The state to start `train` from: the one saved beside the --start model `start` (its path with the extension
    /// .state, as `train` and the leagues name them), if there is one; else a fresh one, with a note.
    pub fn start(params: UWeightParams, start: Option<&Path>) -> io::Result<Self> {
        let Some(model) = start else { return Ok(UWeight::new(params)) };
        let path = model.with_extension("state");
        if path.exists() {
            let uw = UWeight::load(&path, params)
                .map_err(|e| io::Error::new(e.kind(), format!("{}: {e}", path.display())))?;
            eprintln!("weights by uncertainty: running mean {} over {} examples from {}", uw.m, uw.n, path.display());
            Ok(uw)
        } else {
            eprintln!("weights by uncertainty: no state beside {}: the running mean starts afresh", model.display());
            Ok(UWeight::new(params))
        }
    }
}

/// A learner that can train with `UWeight`'s weights through its own path.
pub trait UTrain: Classifier {
    /// Learns from one position as `Classifier::train`, weighted by `uw`.
    fn train_uw(&mut self, data: &[u8], label: &[f64], uw: &mut UWeight);
    /// `Classifier::train_in_game`, weighted by `uw`.
    fn train_in_game_uw(&mut self, data: &[u8], label: &[f64], game: u64, uw: &mut UWeight) {
        let _ = game;
        self.train_uw(data, label, uw)
    }
}

impl UTrain for Probit {
    fn train_uw(&mut self, data: &[u8], label: &[f64], uw: &mut UWeight) {
        Probit::train_uw(self, data, label, uw)
    }
    fn train_in_game_uw(&mut self, data: &[u8], label: &[f64], game: u64, uw: &mut UWeight) {
        Probit::train_in_game_uw(self, data, label, game, uw)
    }
}

impl UTrain for Logreg {
    fn train_uw(&mut self, data: &[u8], label: &[f64], uw: &mut UWeight) {
        Logreg::train_uw(self, data, label, uw)
    }
}

/// A flat model trained with weights by uncertainty: the model, which predicts and saves as itself, and the
/// weighting's state, which `save_state` writes.
pub struct Weighted<M> {
    pub model: M,
    pub uw: UWeight,
}

impl<M: UTrain> Weighted<M> {
    pub fn new(model: M, uw: UWeight) -> Self {
        Weighted { model, uw }
    }
}

impl<M: UTrain> Classifier for Weighted<M> {
    fn train(&mut self, data: &[u8], label: &[f64]) {
        self.model.train_uw(data, label, &mut self.uw)
    }
    fn refresh(&mut self) {
        self.model.refresh()
    }
    fn predict(&self, data: &[u8], out: &mut [f64]) {
        self.model.predict(data, out)
    }
    fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        self.model.predict_log(data, out)
    }
    fn save(&self, path: &Path) -> io::Result<()> {
        self.model.save(path)
    }
    fn forget(&mut self, factor: f64) {
        self.model.forget(factor)
    }
    fn observe_games(&mut self, games: &[&[Encoded]]) {
        self.model.observe_games(games)
    }
    fn train_in_game(&mut self, data: &[u8], label: &[f64], game: u64) {
        self.model.train_in_game_uw(data, label, game, &mut self.uw)
    }
    fn save_state(&self, path: &Path) -> io::Result<bool> {
        self.uw.save(path)?;
        Ok(true)
    }
    fn needs_games(&self) -> bool {
        self.model.needs_games()
    }
    fn fixed_linear(&self) -> Option<FxLinear> {
        self.model.fixed_linear()
    }
    fn log_odds_direct(&self, data: &[u8], white_to_move: bool) -> Option<f64> {
        self.model.log_odds_direct(data, white_to_move)
    }
    fn round_report(&mut self) -> Option<String> {
        Some(self.uw.round_report())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const P: UWeightParams = UWeightParams { floor: 0.2, cap: 3.0, memory: 1_000_000 };

    #[test]
    fn weights_follow_the_formula_with_floor_cap_and_running_mean() {
        let mut uw = UWeight::new(P);
        // First example: m = u, so w = 1 exactly.
        assert_eq!(uw.weight(0.5), 1.0);
        assert_eq!((uw.n, uw.m), (1, 1.0));
        // Decided (raw 4 * 0.99 * 0.01 = 0.0396): u at the floor 0.2; m = 1 + (0.2 - 1) / 2 = 0.6.
        let w = uw.weight(0.99);
        assert_eq!(uw.m, 1.0 + (0.2 - 1.0) / 2.0);
        assert_eq!(w, 0.2 / uw.m);
        // s = 0.75: u = 0.75; m = 0.6 + (0.75 - 0.6) / 3.
        let m = uw.m;
        let w = uw.weight(0.75);
        let u = 4.0 * 0.75 * (1.0 - 0.75);
        assert_eq!(u, 0.75);
        assert_eq!(uw.m, m + (u - m) / 3.0);
        assert_eq!(w, u / uw.m);
        // Many decided examples drag m towards the floor, so an undecided one's ratio passes the cap.
        for _ in 0..1000 {
            uw.weight(1.0);
        }
        assert!(uw.m < 0.21);
        assert_eq!(uw.weight(0.5), 3.0);
        let line = uw.round_report();
        assert!(line.contains("1004 examples"), "{line}");
        assert!(line.contains("at cap 0.10%"), "{line}");
        // The report starts a new round.
        assert!(uw.round_report().contains(" 0 examples"));
    }

    #[test]
    fn the_running_mean_forgets_after_its_memory() {
        let mut uw = UWeight::new(UWeightParams { memory: 2, ..P });
        uw.weight(0.5); // m 1
        uw.weight(1.0); // u 0.2, m = 1 + (0.2 - 1) / 2 = 0.6
        let m = uw.m;
        uw.weight(1.0); // n 3, divided by min(3, 2) = 2
        assert_eq!(uw.m, m + (0.2 - m) / 2.0);
    }

    #[test]
    fn floor_one_gives_weight_one_always() {
        let mut uw = UWeight::new(UWeightParams { floor: 1.0, ..P });
        for s in [0.0, 0.01, 0.3, 0.5, 0.97, 1.0, f64::NAN] {
            assert_eq!(uw.weight(s), 1.0);
        }
    }

    #[test]
    fn the_state_round_trips_exactly_and_checks_its_options() {
        let dir = std::env::temp_dir().join(format!("uweight-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut uw = UWeight::new(P);
        for s in [0.5, 0.93, 0.61, 0.12, 0.999] {
            uw.weight(s);
        }
        let path = dir.join("probit7.state");
        uw.save(&path).unwrap();
        let back = UWeight::load(&path, P).unwrap();
        assert_eq!((back.n, back.m.to_bits()), (uw.n, uw.m.to_bits()));
        // From beside a --start model.
        let started = UWeight::start(P, Some(&dir.join("probit7.model"))).unwrap();
        assert_eq!((started.n, started.m.to_bits()), (uw.n, uw.m.to_bits()));
        // None beside: fresh.
        let fresh = UWeight::start(P, Some(&dir.join("probit8.model"))).unwrap();
        assert_eq!((fresh.n, fresh.m), (0, 0.0));
        // Other options: refused.
        let err = UWeight::load(&path, UWeightParams { floor: 0.3, ..P }).unwrap_err();
        assert!(err.to_string().contains("weighting options"), "{err}");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// Random games' positions with results that mostly follow material, as fastprobit's tests make them.
    fn examples(seed: u64, n: usize) -> Vec<(Encoded, [f64; 3])> {
        use crate::encoding;
        use rand::rngs::StdRng;
        use rand::{RngExt, SeedableRng};
        use shakmaty::{Chess, Position};
        let mut rng = StdRng::seed_from_u64(seed);
        let (mut pos, mut out) = (Chess::default(), Vec::with_capacity(n));
        while out.len() < n {
            let moves = pos.legal_moves();
            if moves.is_empty() || pos.halfmoves() >= 100 {
                pos = Chess::default();
                continue;
            }
            pos.play_unchecked(moves[rng.random_range(0..moves.len())]);
            let white = pos.board().white().count() as i32 - pos.board().black().count() as i32;
            let c = if white > 0 && rng.random_range(0..4) > 0 {
                0
            } else if white < 0 && rng.random_range(0..4) > 0 {
                1
            } else {
                rng.random_range(0..3)
            };
            let mut label = [0.0; 3];
            label[c] = 1.0;
            out.push((encoding::encode(&pos), label));
        }
        out
    }

    const ONE: UWeightParams = UWeightParams { floor: 1.0, cap: 3.0, memory: 1_000_000 };

    #[test]
    fn probit_at_weight_one_trains_as_before() {
        use crate::encoding::CATEGORIES;
        use crate::probit::ProbitParams;
        let data = examples(3, 20_000);
        let mut a = Probit::new(&CATEGORIES, ProbitParams::default());
        let mut b = Weighted::new(Probit::new(&CATEGORIES, ProbitParams::default()), UWeight::new(ONE));
        for (pos, label) in &data {
            a.train(pos, label);
            Classifier::train(&mut b, pos, label);
        }
        assert_eq!(a.to_text(), b.model.to_text());
        assert_eq!(b.uw.n, 20_000);
    }

    #[test]
    fn lr_at_weight_one_trains_as_before() {
        use crate::encoding::CATEGORIES;
        let data = examples(4, 20_000);
        let mut a = Logreg::new(3, &CATEGORIES, 0.05);
        let mut b = Weighted::new(Logreg::new(3, &CATEGORIES, 0.05), UWeight::new(ONE));
        for (pos, label) in &data {
            a.train(pos, label);
            Classifier::train(&mut b, pos, label);
        }
        let state = |m: &Logreg| {
            let mut w = Writer::new("lr");
            m.write_state(&mut w);
            w.into_bytes()
        };
        assert_eq!(a.to_text(), b.model.to_text());
        assert_eq!(state(&a), state(&b.model));
    }

    #[test]
    fn weighted_models_save_their_weights_as_state_and_their_model_as_is() {
        use crate::encoding::CATEGORIES;
        use crate::probit::ProbitParams;
        let dir = std::env::temp_dir().join(format!("uweight-save-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let data = examples(5, 5_000);
        let mut b = Weighted::new(Probit::new(&CATEGORIES, ProbitParams::default()), UWeight::new(P));
        for (pos, label) in &data {
            Classifier::train(&mut b, pos, label);
        }
        Classifier::save(&b, &dir.join("probit1.model")).unwrap();
        assert!(Classifier::save_state(&b, &dir.join("probit1.state")).unwrap());
        let model = Probit::load(&dir.join("probit1.model")).unwrap();
        assert_eq!(model.to_text(), b.model.to_text());
        let uw = UWeight::start(P, Some(&dir.join("probit1.model"))).unwrap();
        assert_eq!((uw.n, uw.m.to_bits()), (b.uw.n, b.uw.m.to_bits()));
        let line = b.uw.round_report();
        assert!(line.contains("5000 examples"), "{line}");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn options_are_checked() {
        assert!(P.check().is_ok());
        assert!(UWeightParams { floor: 0.0, ..P }.check().is_err());
        assert!(UWeightParams { floor: 1.5, ..P }.check().is_err());
        assert!(UWeightParams { cap: 0.5, ..P }.check().is_err());
        assert!(UWeightParams { memory: 0, ..P }.check().is_err());
    }
}
