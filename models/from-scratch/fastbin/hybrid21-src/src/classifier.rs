//! Common interface of the trainable position classifiers.

use std::io;
use std::path::Path;

use crate::fm::Fm;
use crate::bdd::Bdd;
use crate::bdt::Bdt;
use crate::cnbp::Cnbp;
use crate::encoding::Encoded;
use crate::fixed::FxLinear;
use crate::ht::Ht;
use crate::hybrid::Hybrid;
use crate::logreg::Logreg;
use crate::moe::Moe;
use crate::nbmix::{MoeNb, NbGateMoe, NbMix};
use crate::tangate::TanGateMoe;
use crate::probit::Probit;
use crate::tan::Tan;

pub trait Classifier {
    /// Adds one data point. `label` is a distribution over classes.
    fn train(&mut self, data: &[u8], label: &[f64]);
    /// Makes the model ready to predict after training.
    fn refresh(&mut self);
    /// Writes the probability of each class into `out`.
    fn predict(&self, data: &[u8], out: &mut [f64]);
    /// Writes the log of each class's unnormalised probability; differences
    /// between classes are log-odds. Unlike `predict`, need not underflow.
    fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        self.predict(data, out);
        out.iter_mut().for_each(|p| *p = p.max(f64::MIN_POSITIVE).ln());
    }
    fn save(&self, path: &Path) -> io::Result<()>;
    /// Lets the model adapt faster while predicting the same: divides its
    /// counts (CNBP) or AdaGrad sums (LR, a mixture's experts) by `factor`.
    fn forget(&mut self, _factor: f64) {
        unimplemented!("this model cannot forget")
    }
    /// Sees a round's games, each the positions in the order played, before
    /// training on them; delegation trees measure their seams (see
    /// `Bdt::measure_seams`), other models ignore them.
    fn observe_games(&mut self, _games: &[&[Encoded]]) {}
    /// `train` for a position of game `game`, an id the game's positions
    /// share; models that sample or hold out whole games (the Hoeffding
    /// tree's model gain) use it.
    fn train_in_game(&mut self, data: &[u8], label: &[f64], _game: u64) {
        self.train(data, label)
    }
    /// Sees a self-play round's positions, in training order with their
    /// labels and games, before training on them; delegation trees score
    /// their shadow test on them (see `Bdt::observe_round`), other models
    /// ignore them.
    fn observe_round(&mut self, _data: &[(Encoded, [f64; 3], u64)]) {}
    /// After training on a self-play round (see `Bdt::end_round`).
    fn end_round(&mut self) {}
    /// Whether it scores rounds (`observe_round`), so that `train` must give
    /// it each round's positions with their games' ids.
    fn scores_rounds(&self) -> bool {
        false
    }
    /// Writes what training needs beyond the model file to `path`, for an
    /// exact `--resume` (see `state`), and returns true; models whose files
    /// hold all their state write nothing and return false.
    fn save_state(&self, _path: &Path) -> io::Result<bool> {
        Ok(false)
    }
    /// Whether it trains only on whole games (`observe_games`, then
    /// `train_in_game`), so that `train` must give it a round's games, not
    /// just their positions (TAN, whose pair counts count games).
    fn needs_games(&self) -> bool {
        false
    }
    /// The model as a fixed-point linear model over the features (see `fixed`), if it is one: built afresh from the
    /// current weights on each call.
    fn fixed_linear(&self) -> Option<FxLinear> {
        None
    }
    /// ln P(win) - ln P(loss) for the side to move, taken from the model's logits directly (no exp and ln), if the
    /// model has logits.
    fn log_odds_direct(&self, _data: &[u8], _white_to_move: bool) -> Option<f64> {
        None
    }
    /// The model as a compound model in fixed point for exact fast play (see `compound`), if its search values are
    /// a tree's or DAG's leaf's log-odds under one calibration of the whole model (`cal:` over `bdt` or `bdd`).
    fn compound_play(&self) -> Option<crate::compound::Compound<'_>> {
        None
    }
    /// A tree's or DAG's nodes in fixed point, for a calibrated wrapper to keep (see `compound_with`).
    fn compound_tables(&self) -> Option<crate::compound::Tables> {
        None
    }
    /// A tree or DAG as a compound model whose values are its leaves' log-odds under the calibration `cal`, with
    /// `tables` from its `compound_tables` (for a calibrated wrapper's `compound_play`; the model must not have changed
    /// since).
    fn compound_with<'a>(
        &'a self,
        _cal: crate::calibration::Calibration,
        _tables: &'a crate::compound::Tables,
    ) -> Option<crate::compound::Compound<'a>> {
        None
    }
}

impl Classifier for Cnbp {
    fn train(&mut self, data: &[u8], label: &[f64]) {
        Cnbp::train(self, data, label)
    }
    fn refresh(&mut self) {
        Cnbp::refresh(self)
    }
    fn predict(&self, data: &[u8], out: &mut [f64]) {
        Cnbp::predict(self, data, out)
    }
    fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        Cnbp::predict_log(self, data, out)
    }
    fn save(&self, path: &Path) -> io::Result<()> {
        Cnbp::save(self, path)
    }
    fn forget(&mut self, factor: f64) {
        Cnbp::forget(self, factor)
    }
    fn fixed_linear(&self) -> Option<FxLinear> {
        if self.class_num() != 3 {
            return None;
        }
        self.fixed()
    }
    fn log_odds_direct(&self, data: &[u8], white_to_move: bool) -> Option<f64> {
        let mut z = [0.0; 3];
        Cnbp::predict_log(self, data, &mut z);
        Some(if white_to_move { z[0] - z[1] } else { z[1] - z[0] })
    }
}

impl Classifier for Tan {
    fn train(&mut self, data: &[u8], label: &[f64]) {
        Tan::train(self, data, label)
    }
    fn refresh(&mut self) {
        Tan::refresh(self)
    }
    fn predict(&self, data: &[u8], out: &mut [f64]) {
        Tan::predict(self, data, out)
    }
    fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        Tan::predict_log(self, data, out)
    }
    fn save(&self, path: &Path) -> io::Result<()> {
        Tan::save(self, path)
    }
    /// Every pair count, which the model file lacks (see `tan`).
    fn save_state(&self, path: &Path) -> io::Result<bool> {
        Tan::save_state(self, path)?;
        Ok(true)
    }
    fn forget(&mut self, factor: f64) {
        Tan::forget(self, factor)
    }
    fn observe_games(&mut self, games: &[&[Encoded]]) {
        Tan::observe_games(self, games)
    }
    fn train_in_game(&mut self, data: &[u8], label: &[f64], game: u64) {
        Tan::train_in_game(self, data, label, game)
    }
    fn needs_games(&self) -> bool {
        self.options().games
    }
}

impl Classifier for Bdt {
    fn train(&mut self, data: &[u8], label: &[f64]) {
        Bdt::train(self, data, label)
    }
    fn refresh(&mut self) {
        Bdt::refresh(self)
    }
    fn predict(&self, data: &[u8], out: &mut [f64]) {
        Bdt::predict(self, data, out)
    }
    fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        Bdt::predict_log(self, data, out)
    }
    fn save(&self, path: &Path) -> io::Result<()> {
        Bdt::save(self, path)
    }
    fn save_state(&self, path: &Path) -> io::Result<bool> {
        Bdt::save_state(self, path)?;
        Ok(true)
    }
    fn forget(&mut self, factor: f64) {
        Bdt::forget(self, factor)
    }
    fn observe_games(&mut self, games: &[&[Encoded]]) {
        Bdt::measure_seams(self, games)
    }
    fn train_in_game(&mut self, data: &[u8], label: &[f64], game: u64) {
        Bdt::train_in_game(self, data, label, game)
    }
    fn observe_round(&mut self, data: &[(Encoded, [f64; 3], u64)]) {
        Bdt::observe_round(self, data)
    }
    fn end_round(&mut self) {
        Bdt::end_round(self)
    }
    fn scores_rounds(&self) -> bool {
        Bdt::scores_rounds(self)
    }
    fn compound_tables(&self) -> Option<crate::compound::Tables> {
        crate::compound::Tables::new(self.graph())
    }
    fn compound_with<'a>(
        &'a self,
        cal: crate::calibration::Calibration,
        tables: &'a crate::compound::Tables,
    ) -> Option<crate::compound::Compound<'a>> {
        crate::compound::Compound::with_tables(self.graph(), cal, tables)
    }
}

impl Classifier for Bdd {
    fn train(&mut self, data: &[u8], label: &[f64]) {
        Bdd::train(self, data, label)
    }
    fn refresh(&mut self) {
        Bdd::refresh(self)
    }
    fn predict(&self, data: &[u8], out: &mut [f64]) {
        Bdd::predict(self, data, out)
    }
    fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        Bdd::predict_log(self, data, out)
    }
    fn save(&self, path: &Path) -> io::Result<()> {
        Bdd::save(self, path)
    }
    fn save_state(&self, path: &Path) -> io::Result<bool> {
        Bdd::save_state(self, path)?;
        Ok(true)
    }
    fn compound_tables(&self) -> Option<crate::compound::Tables> {
        crate::compound::Tables::new(self.graph())
    }
    fn compound_with<'a>(
        &'a self,
        cal: crate::calibration::Calibration,
        tables: &'a crate::compound::Tables,
    ) -> Option<crate::compound::Compound<'a>> {
        crate::compound::Compound::with_tables(self.graph(), cal, tables)
    }
}

impl Classifier for Logreg {
    fn train(&mut self, data: &[u8], label: &[f64]) {
        Logreg::train(self, data, label)
    }
    fn refresh(&mut self) {}
    fn predict(&self, data: &[u8], out: &mut [f64]) {
        Logreg::predict(self, data, out)
    }
    fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        Logreg::predict_log(self, data, out)
    }
    fn save(&self, path: &Path) -> io::Result<()> {
        Logreg::save(self, path)
    }
    fn forget(&mut self, factor: f64) {
        Logreg::forget(self, factor)
    }
    fn fixed_linear(&self) -> Option<FxLinear> {
        if self.class_num() != 3 {
            return None;
        }
        self.fixed()
    }
    fn log_odds_direct(&self, data: &[u8], white_to_move: bool) -> Option<f64> {
        let mut z = [0.0; 3];
        Logreg::predict_log(self, data, &mut z);
        Some(if white_to_move { z[0] - z[1] } else { z[1] - z[0] })
    }
}

impl Classifier for Probit {
    fn train(&mut self, data: &[u8], label: &[f64]) {
        Probit::train(self, data, label)
    }
    fn refresh(&mut self) {}
    fn predict(&self, data: &[u8], out: &mut [f64]) {
        Probit::predict(self, data, out)
    }
    fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        Probit::predict_log(self, data, out)
    }
    fn save(&self, path: &Path) -> io::Result<()> {
        Probit::save(self, path)
    }
    /// Multiplies the weights' variances (see `Probit::forget`).
    fn forget(&mut self, factor: f64) {
        Probit::forget(self, factor)
    }
    /// Records the games' lengths, for --probit-per-game.
    fn observe_games(&mut self, games: &[&[Encoded]]) {
        Probit::observe_games(self, games)
    }
    fn train_in_game(&mut self, data: &[u8], label: &[f64], game: u64) {
        Probit::train_in_game(self, data, label, game)
    }
    /// With --probit-per-game it needs each position's game.
    fn needs_games(&self) -> bool {
        self.params().per_game.is_some()
    }
    fn fixed_linear(&self) -> Option<FxLinear> {
        self.fixed()
    }
    fn log_odds_direct(&self, data: &[u8], white_to_move: bool) -> Option<f64> {
        let mut z = [0.0; 3];
        Probit::predict_log(self, data, &mut z);
        Some(if white_to_move { z[0] - z[1] } else { z[1] - z[0] })
    }
}

impl Classifier for Fm {
    fn train(&mut self, data: &[u8], label: &[f64]) {
        Fm::train(self, data, label)
    }
    fn refresh(&mut self) {}
    fn predict(&self, data: &[u8], out: &mut [f64]) {
        Fm::predict(self, data, out)
    }
    fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        Fm::predict_log(self, data, out)
    }
    fn save(&self, path: &Path) -> io::Result<()> {
        Fm::save(self, path)
    }
    fn forget(&mut self, factor: f64) {
        Fm::forget(self, factor)
    }
}

impl Classifier for Ht {
    fn train(&mut self, data: &[u8], label: &[f64]) {
        Ht::train(self, data, label)
    }
    fn refresh(&mut self) {
        Ht::refresh(self)
    }
    fn predict(&self, data: &[u8], out: &mut [f64]) {
        Ht::predict(self, data, out)
    }
    fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        Ht::predict_log(self, data, out)
    }
    fn save(&self, path: &Path) -> io::Result<()> {
        Ht::save(self, path)
    }
    fn save_state(&self, path: &Path) -> io::Result<bool> {
        Ht::save_state(self, path)?;
        Ok(true)
    }
    fn forget(&mut self, factor: f64) {
        Ht::forget(self, factor)
    }
    fn observe_games(&mut self, games: &[&[Encoded]]) {
        Ht::observe_games(self, games)
    }
    fn train_in_game(&mut self, data: &[u8], label: &[f64], game: u64) {
        Ht::train_in_game(self, data, label, game)
    }
}

impl Classifier for Moe {
    fn train(&mut self, data: &[u8], label: &[f64]) {
        Moe::train(self, data, label)
    }
    fn refresh(&mut self) {}
    fn predict(&self, data: &[u8], out: &mut [f64]) {
        Moe::predict(self, data, out)
    }
    fn save(&self, path: &Path) -> io::Result<()> {
        Moe::save(self, path)
    }
    fn forget(&mut self, factor: f64) {
        Moe::forget(self, factor)
    }
}

/// Predicts every class as equally likely; the model of a random player.
pub struct Uniform;

impl Classifier for Uniform {
    fn train(&mut self, _data: &[u8], _label: &[f64]) {}
    fn refresh(&mut self) {}
    fn predict(&self, _data: &[u8], out: &mut [f64]) {
        let n = out.len() as f64;
        out.iter_mut().for_each(|p| *p = 1.0 / n);
    }
    fn save(&self, _path: &Path) -> io::Result<()> {
        Ok(())
    }
}

impl Classifier for MoeNb {
    fn train(&mut self, data: &[u8], label: &[f64]) {
        MoeNb::train(self, data, label)
    }
    fn refresh(&mut self) {
        MoeNb::refresh(self)
    }
    fn predict(&self, data: &[u8], out: &mut [f64]) {
        MoeNb::predict(self, data, out)
    }
    fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        MoeNb::predict_log(self, data, out)
    }
    fn save(&self, path: &Path) -> io::Result<()> {
        MoeNb::save(self, path)
    }
    fn forget(&mut self, factor: f64) {
        MoeNb::forget(self, factor)
    }
}

impl Classifier for NbMix {
    fn train(&mut self, data: &[u8], label: &[f64]) {
        NbMix::train(self, data, label)
    }
    fn refresh(&mut self) {
        NbMix::refresh(self)
    }
    fn predict(&self, data: &[u8], out: &mut [f64]) {
        NbMix::predict(self, data, out)
    }
    fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        NbMix::predict_log(self, data, out)
    }
    fn save(&self, path: &Path) -> io::Result<()> {
        NbMix::save(self, path)
    }
    fn forget(&mut self, factor: f64) {
        NbMix::forget(self, factor)
    }
}

impl Classifier for TanGateMoe {
    fn train(&mut self, data: &[u8], label: &[f64]) {
        TanGateMoe::train(self, data, label)
    }
    fn refresh(&mut self) {
        TanGateMoe::refresh(self)
    }
    fn predict(&self, data: &[u8], out: &mut [f64]) {
        TanGateMoe::predict(self, data, out)
    }
    fn save(&self, path: &Path) -> io::Result<()> {
        TanGateMoe::save(self, path)
    }
    /// Every count, which the model file lacks (see `tangate`).
    fn save_state(&self, path: &Path) -> io::Result<bool> {
        TanGateMoe::save_state(self, path)?;
        Ok(true)
    }
    fn forget(&mut self, factor: f64) {
        TanGateMoe::forget(self, factor)
    }
}

impl Classifier for NbGateMoe {
    fn train(&mut self, data: &[u8], label: &[f64]) {
        NbGateMoe::train(self, data, label)
    }
    fn refresh(&mut self) {
        NbGateMoe::refresh(self)
    }
    fn predict(&self, data: &[u8], out: &mut [f64]) {
        NbGateMoe::predict(self, data, out)
    }
    fn save(&self, path: &Path) -> io::Result<()> {
        NbGateMoe::save(self, path)
    }
    fn forget(&mut self, factor: f64) {
        NbGateMoe::forget(self, factor)
    }
}

impl Classifier for Hybrid {
    fn train(&mut self, data: &[u8], label: &[f64]) {
        Hybrid::train(self, data, label)
    }
    fn refresh(&mut self) {
        Hybrid::refresh(self)
    }
    fn predict(&self, data: &[u8], out: &mut [f64]) {
        Hybrid::predict(self, data, out)
    }
    fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        Hybrid::predict_log(self, data, out)
    }
    fn save(&self, path: &Path) -> io::Result<()> {
        Hybrid::save(self, path)
    }
    /// Every expert forgets as it would alone.
    fn forget(&mut self, factor: f64) {
        Hybrid::forget(self, factor)
    }
    fn observe_games(&mut self, games: &[&[Encoded]]) {
        Hybrid::observe_games(self, games)
    }
    fn train_in_game(&mut self, data: &[u8], label: &[f64], game: u64) {
        Hybrid::train_in_game(self, data, label, game)
    }
    fn needs_games(&self) -> bool {
        Hybrid::needs_games(self)
    }
}
