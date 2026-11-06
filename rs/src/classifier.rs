//! Common interface of the trainable position classifiers.

use std::io;
use std::path::Path;

use crate::bdd::Bdd;
use crate::bdt::Bdt;
use crate::cnbp::Cnbp;
use crate::logreg::Logreg;
use crate::moe::Moe;

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
