//! Common interface of the trainable position classifiers.

use std::io;
use std::path::Path;

use crate::bdt::Bdt;
use crate::cnbp::Cnbp;

pub trait Classifier {
    /// Adds one data point. `label` is a distribution over classes.
    fn train(&mut self, data: &[u8], label: &[f64]);
    /// Makes the model ready to predict after training.
    fn refresh(&mut self);
    /// Writes the probability of each class into `out`.
    fn predict(&self, data: &[u8], out: &mut [f64]);
    /// Predicts `data` as the model would predict positions like `route_by`;
    /// for models that delegate positions to sub-models (BDT), the sub-model
    /// is chosen by `route_by`. Others ignore it.
    fn predict_routed(&self, _route_by: &[u8], data: &[u8], out: &mut [f64]) {
        self.predict(data, out)
    }
    fn save(&self, path: &Path) -> io::Result<()>;
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
    fn save(&self, path: &Path) -> io::Result<()> {
        Cnbp::save(self, path)
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
    fn predict_routed(&self, route_by: &[u8], data: &[u8], out: &mut [f64]) {
        Bdt::predict_routed(self, route_by, data, out)
    }
    fn save(&self, path: &Path) -> io::Result<()> {
        Bdt::save(self, path)
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
