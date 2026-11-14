//! Common interface of the trainable position classifiers.

use std::io;
use std::path::Path;

use crate::fm::Fm;
use crate::bdd::Bdd;
use crate::bdt::Bdt;
use crate::cnbp::Cnbp;
use crate::encoding::Encoded;
use crate::ht::Ht;
use crate::logreg::Logreg;
use crate::moe::Moe;
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
