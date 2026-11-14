//! Online Bayesian ordinal probit regression with assumed-density filtering
//! (the TrueSkill / AdPredictor family: Herbrich et al. 2007, Graepel et al.
//! 2010), on the logistic regression's features (see `logreg`).
//!
//! The model. Every weight (a bias, and one per feature value, as one class
//! of the logistic regression's weights) is Gaussian, w_j ~ N(mu_j, s2_j),
//! independent of the others; a new model has mu 0 and s2 = prior^2. A
//! position's latent advantage for white is t = sum_j w_j x_j + N(0, beta^2),
//! x_j 1 for the weights the position uses (the bias and its 70 features'
//! values) and 0 for the others. The game's result is ordered: white wins if
//! t > eps, black if t < -eps, and it is drawn if |t| <= eps, eps being the
//! draw margin. So with m = sum mu_j x_j and S^2 = beta^2 + sum s2_j x_j^2,
//! P(white wins) = Phi((m - eps) / S), P(black wins) = Phi((-m - eps) / S)
//! and the draw the rest: the weights' uncertainty makes positions the model
//! knows little about less extreme.
//!
//! Training (assumed-density filtering): each example first adds tau^2 to
//! the variances of the weights it uses (dynamics: self-play's games drift
//! as the model improves, and the step sizes must not die out), then the
//! latent t's distribution N(m, S^2) is truncated to the result's interval,
//! the truncated Gaussian is replaced by the Gaussian of its mean and
//! variance, and that change of t is passed on to the weights it is a sum
//! of: mu_j += s2_j v / S, s2_j *= 1 - s2_j w / S^2, v and w being the
//! standardized shift of t's mean and the fraction of its variance removed
//! (TrueSkill's v and w functions, for one-sided and two-sided truncations).
//!
//! Tempering. Every position of a game shares its result, so counting each
//! as an independent observation shrinks the variances (and so the steps)
//! about as many times too fast as a game has positions, the double counting
//! that makes naive Bayes overconfident. Each update therefore counts the
//! result's likelihood only to the power `alpha` (Power EP / a fractional
//! update): the Gaussian message from the result to t has its precision and
//! precision-mean multiplied by alpha, which turns v and w into
//! alpha v / (1 - (1 - alpha) w) and alpha w / (1 - (1 - alpha) w). With
//! `per_game` C the power is C / L instead, L the number of positions of the
//! position's game, so that every game counts as C observations whatever its
//! length. A label that is a distribution over results (search labels)
//! counts each result k to the power alpha label_k, one after another.
//!
//! The draw margin is fitted online by type-II maximum likelihood: after each
//! example, eps takes a gradient step of `eps_rate` on the log of the
//! probability the model gave the example's result before learning from it.
//!
//! The model file holds everything training needs (the weights' means and
//! variances, eps and the options), so a model trains on from its file
//! exactly as if it had not stopped.

use std::fmt::Write as _;
use std::io;
use std::path::Path;

use crate::encoding::Encoded;
use crate::state::{Reader, Writer};
use crate::textfmt::{Tokens, invalid};

/// The model's options; see the module documentation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProbitParams {
    /// The prior standard deviation of every weight.
    pub prior: f64,
    /// The standard deviation of the latent's noise around the weights' sum.
    pub beta: f64,
    /// The dynamics: the standard deviation added to each weight an example
    /// uses before learning from it.
    pub tau: f64,
    /// The power of each example's likelihood.
    pub alpha: f64,
    /// If set, the power is this divided by the example's game's number of
    /// positions, when training knows its game (else `alpha`).
    pub per_game: Option<f64>,
    /// The draw margin of a new model.
    pub eps: f64,
    /// The draw margin's step size (0: fixed).
    pub eps_rate: f64,
}

impl Default for ProbitParams {
    /// Chosen on an offline benchmark (models/from-scratch/probit/offline:
    /// 4 league round-robins, one shuffled pass, scored on a fifth): neither a
    /// power below 1 nor per-game powers predicted better than alpha 1, and a
    /// little dynamics (tau 1e-5) predicted slightly better than none, while
    /// keeping the steps from dying out over a long self-play run.
    fn default() -> Self {
        ProbitParams { prior: 0.3, beta: 1.0, tau: 1e-5, alpha: 1.0, per_game: None, eps: 0.5, eps_rate: 1e-4 }
    }
}

/// The smallest draw margin: a draw stays possible.
const EPS_MIN: f64 = 1e-3;

#[derive(Clone, Debug)]
pub struct Probit {
    categories: Vec<u8>,
    /// Index of each feature's first value in `weights`, after the bias at
    /// index 0.
    starts: Vec<usize>,
    /// Each weight's posterior mean and variance.
    weights: Vec<[f64; 2]>,
    params: ProbitParams,
    /// The draw margin.
    eps: f64,
    /// The number of positions of each game of the round being trained, by
    /// its index (the low 32 bits of the id `train_in_game` gets).
    lengths: Vec<u32>,
}

impl Probit {
    pub fn new(categories: &[u8], params: ProbitParams) -> Self {
        let mut starts = Vec::with_capacity(categories.len());
        let mut width = 1;
        for &cat in categories {
            starts.push(width);
            width += cat as usize;
        }
        Probit {
            categories: categories.to_vec(),
            starts,
            weights: vec![[0.0, params.prior * params.prior]; width],
            params,
            eps: params.eps,
            lengths: Vec::new(),
        }
    }

    pub fn params(&self) -> ProbitParams {
        self.params
    }

    /// The draw margin.
    pub fn eps(&self) -> f64 {
        self.eps
    }

    /// Sets the draw margin (see `DelegationParams::probit_shared_eps`).
    pub fn set_eps(&mut self, eps: f64) {
        self.eps = eps;
    }

    /// The mean and variance of the latent t of a position (beta^2 included).
    fn latent(&self, data: &[u8]) -> (f64, f64) {
        let [mut m, mut var] = self.weights[0];
        for (&s, &v) in self.starts.iter().zip(data) {
            let [mu, s2] = self.weights[s + v as usize];
            m += mu;
            var += s2;
        }
        (m, var + self.params.beta * self.params.beta)
    }

    /// Calls `f` on each weight the position uses: the bias, then each
    /// feature's value.
    fn for_active(&mut self, data: &[u8], mut f: impl FnMut(&mut [f64; 2])) {
        f(&mut self.weights[0]);
        for (&s, &v) in self.starts.iter().zip(data) {
            f(&mut self.weights[s + v as usize]);
        }
    }

    /// White's latent advantage's mean, and the variance of the weights'
    /// sum (beta's not included).
    pub fn latent_parts(&self, data: &[u8]) -> (f64, f64) {
        let (m, var) = self.latent(data);
        let beta2 = self.params.beta * self.params.beta;
        (m, var - beta2)
    }

    /// The model's mean and standard deviation of white's latent advantage
    /// (beta included).
    pub fn latent_mean_sd(&self, data: &[u8]) -> (f64, f64) {
        let (m, var) = self.latent(data);
        (m, var.sqrt())
    }

    /// Writes the probabilities of [white win, black win, draw].
    pub fn predict(&self, data: &[u8], out: &mut [f64]) {
        let (m, var) = self.latent(data);
        out[..3].copy_from_slice(&probs_of_latent(m, var, self.eps));
    }

    /// Writes the log-probabilities of [white win, black win, draw], exact
    /// far into the tails (no underflow).
    pub fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        let (m, var) = self.latent(data);
        out[..3].copy_from_slice(&log_probs_of_latent(m, var, self.eps));
    }

    /// The model in fixed point (see `fixed`): lanes 0 and 1 are each weight's mean and variance.
    pub fn fixed(&self) -> Option<crate::fixed::FxLinear> {
        let rows = self.weights.iter().map(|&[mu, s2]| [mu, s2, 0.0, 0.0]);
        let head = crate::fixed::Head::Probit { eps: self.eps, beta2: self.params.beta * self.params.beta };
        crate::fixed::FxLinear::from_rows(rows, |f| self.starts[f], head)
    }

    /// Learns from one position whose result's distribution is `label`
    /// ([white win, black win, draw]), with the likelihood to the power
    /// `alpha`.
    pub fn train(&mut self, data: &[u8], label: &[f64]) {
        self.update(data, label, self.params.alpha);
    }

    /// `train` for a position of game `game` of the round (see
    /// `observe_games`): with `per_game`, the power is per_game / L.
    pub fn train_in_game(&mut self, data: &[u8], label: &[f64], game: u64) {
        let power = match self.params.per_game {
            Some(c) => match self.lengths.get((game & 0xffff_ffff) as usize) {
                Some(&len) if len > 0 => c / len as f64,
                _ => self.params.alpha,
            },
            None => self.params.alpha,
        };
        self.update(data, label, power);
    }

    /// Records the length of each of the round's games, in order, for
    /// `train_in_game`.
    pub fn observe_games(&mut self, games: &[&[Encoded]]) {
        self.lengths.clear();
        self.lengths.extend(games.iter().map(|g| g.len() as u32));
    }

    /// The standardized interval of result `class` for a latent N(m, s^2).
    fn interval(&self, class: usize, m: f64, s: f64) -> (f64, f64) {
        match class {
            0 => ((self.eps - m) / s, f64::INFINITY),
            1 => (f64::NEG_INFINITY, (-self.eps - m) / s),
            _ => ((-self.eps - m) / s, (self.eps - m) / s),
        }
    }

    /// The derivative of the log-probability of result `class` with respect
    /// to eps, from its interval's truncation (`truncated`).
    fn eps_gradient(class: usize, t: &Truncation, s: f64) -> f64 {
        match class {
            0 => -t.ga / s,
            1 => -t.gb / s,
            _ => (t.ga + t.gb) / s,
        }
    }

    fn update(&mut self, data: &[u8], label: &[f64], power: f64) {
        if power <= 0.0 {
            return;
        }
        let tau2 = self.params.tau * self.params.tau;
        if tau2 > 0.0 {
            self.for_active(data, |w| w[1] += tau2);
        }
        let (m, var) = self.latent(data);
        let s = var.sqrt();
        // The draw margin's gradient, from the prediction before learning.
        let mut grad = 0.0;
        let mut first = None;
        for (class, &l) in label.iter().enumerate().take(3) {
            if l > 0.0 {
                let (lo, hi) = self.interval(class, m, s);
                let t = truncated(lo, hi);
                grad += l * Self::eps_gradient(class, &t, s);
                first.get_or_insert((class, t));
            }
        }
        let Some((first_class, first_t)) = first else { return };
        for (class, &l) in label.iter().enumerate().take(3) {
            if l <= 0.0 {
                continue;
            }
            let (t, var) = if class == first_class {
                (first_t, var)
            } else {
                // Later results of a mixed label see the updated weights.
                let (m, var) = self.latent(data);
                let (lo, hi) = self.interval(class, m, var.sqrt());
                (truncated(lo, hi), var)
            };
            self.absorb(data, &t, var, power * l);
        }
        if self.params.eps_rate > 0.0 && grad.is_finite() {
            self.eps = (self.eps + self.params.eps_rate * grad).max(EPS_MIN);
        }
    }

    /// Moves the weights the position uses by a truncation `t` of the latent
    /// (variance `var`) taken to the power `power`.
    fn absorb(&mut self, data: &[u8], t: &Truncation, var: f64, power: f64) {
        let (v, w) = t.tempered(power);
        if !(v.is_finite() && w.is_finite()) {
            return;
        }
        let s = var.sqrt();
        let (dv, dw) = (v / s, w / var);
        self.for_active(data, |x| {
            let s2 = x[1];
            x[0] += s2 * dv;
            x[1] = s2 * (1.0 - s2 * dw);
        });
    }

    /// With path cells (`pathctx`, errdag3): the latent's mean and variance
    /// (beta^2 included) with the position's cell weights, whose means sum to
    /// `mu` and variances to `s2`, added to the model's own weights.
    fn latent_with(&self, data: &[u8], mu: f64, s2: f64) -> (f64, f64) {
        let (m, var) = self.latent(data);
        (m + mu, var + s2)
    }

    /// errdag4: the latent's mean and variance without cells (`latent`), which `Graph::node_output_ctx` caches per
    /// node and training step; `latent_with` adds the cells' parts to it.
    pub(crate) fn latent_base(&self, data: &[u8]) -> (f64, f64) {
        self.latent(data)
    }

    /// errdag4: `predict_with_cells` (not log) from `latent_base`'s (m, var) and the cells' parts: the same
    /// additions (`latent_with`) and `probs_of_latent`.
    pub(crate) fn predict_from_base(&self, m: f64, var: f64, mu: f64, s2: f64, out: &mut [f64]) {
        let (m, var) = (m + mu, var + s2);
        out[..3].copy_from_slice(&probs_of_latent(m, var, self.eps));
    }

    /// `predict` (or `predict_log` if `log`) with path cells whose weights for
    /// the position have means summing to `mu` and variances to `s2`.
    pub fn predict_with_cells(&self, data: &[u8], mu: f64, s2: f64, out: &mut [f64], log: bool) {
        let (m, var) = self.latent_with(data, mu, s2);
        let p = match log {
            true => log_probs_of_latent(m, var, self.eps),
            false => probs_of_latent(m, var, self.eps),
        };
        out[..3].copy_from_slice(&p);
    }

    /// `train` with path cells: `cells` holds the position's cell weights
    /// (mean, variance), one per cell, which learn as further weights the
    /// position uses, exactly as the model's own: the dynamics widen them,
    /// they enter the latent, and the truncation moves them (see `update`).
    pub fn train_with_cells(&mut self, data: &[u8], label: &[f64], cells: &mut [[f64; 2]]) {
        let power = self.params.alpha;
        if power <= 0.0 {
            return;
        }
        let tau2 = self.params.tau * self.params.tau;
        if tau2 > 0.0 {
            self.for_active(data, |w| w[1] += tau2);
            cells.iter_mut().for_each(|w| w[1] += tau2);
        }
        let sums = |cells: &[[f64; 2]]| cells.iter().fold((0.0, 0.0), |(m, v), w| (m + w[0], v + w[1]));
        let (mu, s2) = sums(cells);
        let (m, var) = self.latent_with(data, mu, s2);
        let s = var.sqrt();
        let mut grad = 0.0;
        let mut first = None;
        for (class, &l) in label.iter().enumerate().take(3) {
            if l > 0.0 {
                let (lo, hi) = self.interval(class, m, s);
                let t = truncated(lo, hi);
                grad += l * Self::eps_gradient(class, &t, s);
                first.get_or_insert((class, t));
            }
        }
        let Some((first_class, first_t)) = first else { return };
        for (class, &l) in label.iter().enumerate().take(3) {
            if l <= 0.0 {
                continue;
            }
            let (t, var) = if class == first_class {
                (first_t, var)
            } else {
                let (mu, s2) = sums(cells);
                let (m, var) = self.latent_with(data, mu, s2);
                let (lo, hi) = self.interval(class, m, var.sqrt());
                (truncated(lo, hi), var)
            };
            let (v, w) = t.tempered(power * l);
            if !(v.is_finite() && w.is_finite()) {
                continue;
            }
            let (dv, dw) = (v / var.sqrt(), w / var);
            let step = |x: &mut [f64; 2]| {
                let s2 = x[1];
                x[0] += s2 * dv;
                x[1] = s2 * (1.0 - s2 * dw);
            };
            self.for_active(data, step);
            cells.iter_mut().for_each(step);
        }
        if self.params.eps_rate > 0.0 && grad.is_finite() {
            self.eps = (self.eps + self.params.eps_rate * grad).max(EPS_MIN);
        }
    }

    /// Multiplies every weight's variance by `factor` (divides its precision,
    /// as a CNBP's counts): the model learns faster, but unlike a logistic
    /// regression it also predicts less confidently.
    pub fn forget(&mut self, factor: f64) {
        self.weights.iter_mut().for_each(|w| w[1] *= factor);
    }

    /// Adds `var` to every weight's variance: a copy widened by noise^2 is
    /// the prior of a child of the model whose weights are the model's plus
    /// independent N(0, noise^2) (see `DelegationParams::split_noise`). It
    /// predicts less confidently and moves away from the model quickly.
    pub fn widen(&mut self, var: f64) {
        self.weights.iter_mut().for_each(|w| w[1] += var);
    }

    pub fn to_text(&self) -> String {
        let p = &self.params;
        let mut s = String::new();
        let per_game = p.per_game.map_or("none".to_string(), |c| format!("{c:e}"));
        // Full precision, so a reloaded model trains on exactly as before.
        writeln!(s, "probit\n1\n{:e}\n{:e}\n{:e}\n{:e}\n{per_game}", p.prior, p.beta, p.tau, p.alpha).unwrap();
        writeln!(s, "{:e}\n{:e}\n{:e}\n{}", p.eps, p.eps_rate, self.eps, self.categories.len()).unwrap();
        for cat in &self.categories {
            writeln!(s, "{cat}").unwrap();
        }
        for [mu, s2] in &self.weights {
            writeln!(s, "{mu:e} {s2:e}").unwrap();
        }
        s
    }

    pub fn from_text(text: &str) -> io::Result<Self> {
        let mut tokens = Tokens::new(text);
        let model = Self::from_tokens(&mut tokens)?;
        if tokens.peek().is_some() {
            return Err(invalid("probit model file has extra values"));
        }
        Ok(model)
    }

    /// Reads a model in the text format from `tokens`, which may go on after
    /// it (a tree's file holds its leaves one after another).
    pub(crate) fn from_tokens(tokens: &mut Tokens) -> io::Result<Self> {
        if tokens.next::<String>()? != "probit" {
            return Err(invalid("not a probit model"));
        }
        if tokens.next::<u32>()? != 1 {
            return Err(invalid("unknown probit model version"));
        }
        let (prior, beta, tau, alpha) = (tokens.next()?, tokens.next()?, tokens.next()?, tokens.next()?);
        let per_game = match tokens.next::<String>()?.as_str() {
            "none" => None,
            c => Some(c.parse::<f64>().map_err(|e| invalid(format!("{e}: {c:?}")))?),
        };
        let (eps0, eps_rate, eps): (f64, f64, f64) = (tokens.next()?, tokens.next()?, tokens.next()?);
        let feature_num: usize = tokens.next()?;
        let categories = (0..feature_num).map(|_| tokens.next()).collect::<io::Result<Vec<u8>>>()?;
        let params = ProbitParams { prior, beta, tau, alpha, per_game, eps: eps0, eps_rate };
        let mut model = Probit::new(&categories, params);
        model.eps = eps;
        for w in &mut model.weights {
            *w = [tokens.next()?, tokens.next()?];
        }
        Ok(model)
    }

    /// Writes the model, unrounded, for a tree's state file (see `state`):
    /// what `to_text` writes, in binary.
    pub(crate) fn write_state(&self, w: &mut Writer) {
        let p = &self.params;
        for x in [p.prior, p.beta, p.tau, p.alpha] {
            w.f64(x);
        }
        w.bool(p.per_game.is_some());
        w.f64(p.per_game.unwrap_or(0.0));
        for x in [p.eps, p.eps_rate, self.eps] {
            w.f64(x);
        }
        w.u8s(&self.categories);
        w.f64s(&self.weights.iter().flatten().copied().collect::<Vec<f64>>());
    }

    /// Reads a model written by `write_state`.
    pub(crate) fn read_state(r: &mut Reader) -> io::Result<Self> {
        let (prior, beta, tau, alpha) = (r.f64()?, r.f64()?, r.f64()?, r.f64()?);
        let (has_per_game, per_game) = (r.bool()?, r.f64()?);
        let (eps0, eps_rate, eps) = (r.f64()?, r.f64()?, r.f64()?);
        let categories = r.u8s()?;
        let per_game = has_per_game.then_some(per_game);
        let params = ProbitParams { prior, beta, tau, alpha, per_game, eps: eps0, eps_rate };
        let mut model = Probit::new(&categories, params);
        model.eps = eps;
        let weights = r.f64s()?;
        if weights.len() != 2 * model.weights.len() {
            return Err(invalid("probit state of the wrong shape"));
        }
        for (w, pair) in model.weights.iter_mut().zip(weights.chunks_exact(2)) {
            *w = [pair[0], pair[1]];
        }
        Ok(model)
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        std::fs::write(path, self.to_text())
    }

    pub fn load(path: &Path) -> io::Result<Self> {
        Self::from_text(&std::fs::read_to_string(path)?)
    }
}

// ---------------------------------------------------------------- the normal distribution

/// 1 / sqrt(2 pi).
const FRAC_1_SQRT_2PI: f64 = 0.398_942_280_401_432_7;
/// Below this |x|, Phi(x) is 1/2 plus a rational function of x.
const CENTRAL: f64 = 0.674_489_75;
/// sqrt(32): above it, the tail's asymptotic rational function.
const SQRT_32: f64 = 5.656_854_249_492_381;

// W. J. Cody's rational Chebyshev approximations of the normal distribution
// (ACM TOMS Algorithm 715, 1993), as in R's pnorm: relative error about 1e-16
// in both tails.
const A: [f64; 5] = [
    2.2352520354606839287,
    161.02823106855587881,
    1067.6894854603709582,
    18154.981253343561249,
    0.065682337918207449113,
];
const B: [f64; 4] = [47.20258190468824187, 976.09855173777669322, 10260.932208618978205, 45507.789335026729956];
const C: [f64; 9] = [
    0.39894151208813466764,
    8.8831497943883759412,
    93.506656132177855979,
    597.27027639480026226,
    2494.5375852903726711,
    6848.1904505362823326,
    11602.651437647350124,
    9842.7148383839780218,
    1.0765576773720192317e-8,
];
const D: [f64; 8] = [
    22.266688044328115691,
    235.38790178262499861,
    1519.377599407554805,
    6485.558298266760755,
    18615.571640885098091,
    34900.952721145977266,
    38912.003286093271411,
    19685.429676859990727,
];
const P: [f64; 6] = [
    0.21589853405795699,
    0.1274011611602473639,
    0.022235277870649807,
    0.001421619193227893466,
    2.9112874951168792e-5,
    0.02307344176494017303,
];
const Q: [f64; 5] = [
    1.28426009614491121,
    0.468238212480865118,
    0.0659881378689285515,
    0.00378239633202758244,
    7.29751555083966205e-5,
];

/// Phi(x) - 1/2 for |x| <= CENTRAL.
fn central(x: f64) -> f64 {
    let xsq = x * x;
    let (mut num, mut den) = (A[4] * xsq, xsq);
    for i in 0..3 {
        num = (num + A[i]) * xsq;
        den = (den + B[i]) * xsq;
    }
    x * (num + A[3]) / (den + B[3])
}

/// r(y) = Phi(-y) exp(y^2 / 2) for y > CENTRAL (Mills' ratio over sqrt(2 pi)).
fn scaled_tail(y: f64) -> f64 {
    if y <= SQRT_32 {
        let (mut num, mut den) = (C[8] * y, y);
        for i in 0..7 {
            num = (num + C[i]) * y;
            den = (den + D[i]) * y;
        }
        (num + C[7]) / (den + D[7])
    } else {
        let xsq = 1.0 / (y * y);
        let (mut num, mut den) = (P[5] * xsq, xsq);
        for i in 0..4 {
            num = (num + P[i]) * xsq;
            den = (den + Q[i]) * xsq;
        }
        (FRAC_1_SQRT_2PI - xsq * (num + P[4]) / (den + Q[4])) / y
    }
}

/// (Phi(x), Phi(-x)), each accurate to its own size.
pub fn cdf2(x: f64) -> (f64, f64) {
    let y = x.abs();
    if y <= CENTRAL {
        let c = central(x);
        return (0.5 + c, 0.5 - c);
    }
    let tail = (-0.5 * y * y).exp() * scaled_tail(y);
    if x < 0.0 { (tail, 1.0 - tail) } else { (1.0 - tail, tail) }
}

/// Phi(x).
pub fn cdf(x: f64) -> f64 {
    cdf2(x).0
}

/// The probabilities of [white win, black win, draw] for a latent N(m, var) and draw margin `eps` (`predict`).
pub fn probs_of_latent(m: f64, var: f64, eps: f64) -> [f64; 3] {
    let s = var.sqrt();
    let (pw, cw) = cdf2((m - eps) / s);
    let (pb, cb) = cdf2((-m - eps) / s);
    // The draw's probability from the complement that is not close to 1.
    [pw, pb, if m >= 0.0 { cw - pb } else { cb - pw }.max(0.0)]
}

/// The log-probabilities of [white win, black win, draw] for a latent N(m, var) and draw margin `eps`, exact far
/// into the tails (`predict_log`).
pub fn log_probs_of_latent(m: f64, var: f64, eps: f64) -> [f64; 3] {
    let s = var.sqrt();
    let (zw, zb) = ((m - eps) / s, (-m - eps) / s);
    let (lw, lw_c) = ln_cdf2(zw);
    let (lb, lb_c) = ln_cdf2(zb);
    // The draw is t in [zb, -zw] standardized.
    let ld = if zw >= 0.0 {
        // Both ends below 0: Phi(-zw) - Phi(zb).
        lw_c + ln_one_minus_exp(lb - lw_c)
    } else if zb >= 0.0 {
        lb_c + ln_one_minus_exp(lw - lb_c)
    } else {
        (-(lw.exp() + lb.exp())).ln_1p()
    };
    [lw, lb, ld]
}

/// (ln Phi(x), ln Phi(-x)), without underflow.
pub fn ln_cdf2(x: f64) -> (f64, f64) {
    let y = x.abs();
    if y <= CENTRAL {
        let c = central(x);
        return ((0.5 + c).ln(), (0.5 - c).ln());
    }
    let r = scaled_tail(y);
    let ln_tail = -0.5 * y * y + r.ln();
    let ln_body = (-(-0.5 * y * y).exp() * r).ln_1p();
    if x < 0.0 { (ln_tail, ln_body) } else { (ln_body, ln_tail) }
}

/// ln Phi(x).
pub fn ln_cdf(x: f64) -> f64 {
    ln_cdf2(x).0
}

/// The standard normal density.
pub fn pdf(x: f64) -> f64 {
    FRAC_1_SQRT_2PI * (-0.5 * x * x).exp()
}

/// The inverse Mills ratio phi(x) / Phi(x), without underflow (about -x far
/// below 0).
pub fn mills(x: f64) -> f64 {
    if x < -CENTRAL {
        return FRAC_1_SQRT_2PI / scaled_tail(-x);
    }
    pdf(x) / cdf(x)
}

/// ln(1 - e^x) for x <= 0.
fn ln_one_minus_exp(x: f64) -> f64 {
    if x > -std::f64::consts::LN_2 { (-x.exp_m1()).ln() } else { (-x.exp()).ln_1p() }
}

/// The standard normal truncated to [lo, hi] (either may be infinite), with
/// Z = Phi(hi) - Phi(lo): ga = phi(lo) / Z and gb = phi(hi) / Z (0 at an
/// infinite end). Its mean is ga - gb and its variance 1 - w (`v_w`).
#[derive(Clone, Copy, Debug)]
pub struct Truncation {
    pub lo: f64,
    pub hi: f64,
    pub ga: f64,
    pub gb: f64,
}

impl Truncation {
    /// TrueSkill's v (the mean's shift) and w (the share of the variance
    /// removed).
    pub fn v_w(&self) -> (f64, f64) {
        let v = self.ga - self.gb;
        let mut w = v * v;
        if self.ga != 0.0 {
            w -= self.lo * self.ga;
        }
        if self.gb != 0.0 {
            w += self.hi * self.gb;
        }
        (v, w.clamp(0.0, 1.0))
    }

    /// v and w of the likelihood taken to the power `p`: the message to the
    /// latent has its natural parameters multiplied by p.
    pub fn tempered(&self, p: f64) -> (f64, f64) {
        let (v, w) = self.v_w();
        if p == 1.0 {
            return (v, w);
        }
        let d = 1.0 - (1.0 - p) * w;
        (p * v / d, p * w / d)
    }
}

/// The truncation of the standard normal to [lo, hi], lo < hi.
pub fn truncated(lo: f64, hi: f64) -> Truncation {
    let (ga, gb) = if lo == f64::NEG_INFINITY {
        (0.0, mills(hi))
    } else if hi == f64::INFINITY {
        (mills(-lo), 0.0)
    } else if hi <= 0.0 {
        both_below(lo, hi)
    } else if lo >= 0.0 {
        // Mirrored: [-hi, -lo] has its ends' densities swapped.
        let (a, b) = both_below(-hi, -lo);
        (b, a)
    } else {
        // Around 0: no cancellation in Z.
        let z = 1.0 - cdf2(hi).1 - cdf(lo);
        (pdf(lo) / z, pdf(hi) / z)
    };
    Truncation { lo, hi, ga, gb }
}

/// (phi(lo) / Z, phi(hi) / Z) for lo < hi <= 0, from the ratio
/// q = Phi(lo) / Phi(hi): Z = Phi(hi) (1 - q).
fn both_below(lo: f64, hi: f64) -> (f64, f64) {
    let d = ln_cdf(lo) - ln_cdf(hi);
    let one_minus_q = -d.exp_m1();
    (mills(lo) * d.exp() / one_minus_q, mills(hi) / one_minus_q)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Phi(-z) for z >= 0 by references independent of Cody's: the
    /// all-positive series of erf below 2 (1 - erf loses digits above), the
    /// continued fraction of Mills' ratio above.
    fn reference_lower(z: f64) -> f64 {
        assert!(z >= 0.0);
        if z < 2.0 {
            // erf(x) = 2/sqrt(pi) e^{-x^2} sum 2^n x^{2n+1} / (2n+1)!!
            let x = z / std::f64::consts::SQRT_2;
            let (mut term, mut sum, mut n) = (x, x, 0.0);
            while term > 1e-300 && term > sum * 1e-18 {
                n += 1.0;
                term *= 2.0 * x * x / (2.0 * n + 1.0);
                sum += term;
            }
            let erf = 2.0 / std::f64::consts::PI.sqrt() * (-x * x).exp() * sum;
            0.5 * (1.0 - erf)
        } else {
            reference_mills_ratio(z) * pdf(z)
        }
    }

    /// Mills' ratio Phi(-z) / phi(z), by its continued fraction
    /// 1 / (z + 1 / (z + 2 / (z + 3 / (z + ...)))), evaluated backwards.
    fn reference_mills_ratio(z: f64) -> f64 {
        let mut t = z;
        for k in (1..20_000).rev() {
            t = z + k as f64 / t;
        }
        1.0 / t
    }

    fn close(a: f64, b: f64, rel: f64) -> bool {
        (a - b).abs() <= rel * a.abs().max(b.abs()) || a == b
    }

    #[test]
    fn normal_cdf_matches_known_values() {
        // High-precision values of Phi(-z).
        for (z, p) in [
            (1.0, 0.15865525393145705),
            (2.0, 0.022750131948179207),
            (3.0, 0.0013498980316300946),
            (5.0, 2.866515718791939e-7),
            (10.0, 7.619853024160527e-24),
        ] {
            assert!(close(cdf(-z), p, 2e-15), "Phi(-{z}) = {:e}, want {p:e}", cdf(-z));
            assert!(close(cdf(z), 1.0 - p, 1e-15), "Phi({z})");
            assert!(close(ln_cdf(-z), p.ln(), 1e-14), "ln Phi(-{z})");
        }
        assert_eq!(cdf(0.0), 0.5);
    }

    #[test]
    fn normal_cdf_matches_independent_references() {
        // The two references agree where both work.
        for i in 0..=20 {
            let z = 1.5 + i as f64 * 0.025;
            let series = reference_lower(z.min(1.999));
            let fraction = reference_mills_ratio(z.min(1.999)) * pdf(z.min(1.999));
            assert!(close(series, fraction, 1e-13), "z {z}: series {series:e} fraction {fraction:e}");
        }
        let mut worst = 0.0f64;
        for i in 0..=4000 {
            let z = i as f64 * 0.01; // 0 to 40
            let want = reference_lower(z);
            // In logs, and the inverse Mills ratio, without the subnormal
            // numbers Phi(-z) and phi(z) are beyond z = 37.5.
            let (want_ln, want_mills) = if z < 2.0 {
                (want.ln(), pdf(z) / want)
            } else {
                let r = reference_mills_ratio(z);
                (-0.5 * z * z + (r * FRAC_1_SQRT_2PI).ln(), 1.0 / r)
            };
            let (lo, hi) = cdf2(-z);
            let (ln_lo, ln_hi) = ln_cdf2(-z);
            if z <= 37.0 {
                let rel = ((lo - want) / want).abs();
                worst = worst.max(rel);
                assert!(rel < 5e-14, "Phi(-{z}) = {lo:e}, reference {want:e}");
            }
            assert!(close(hi, 1.0 - want, 1e-15), "Phi({z})");
            assert!((ln_lo - want_ln).abs() < 1e-13 * want_ln.abs().max(1.0), "ln Phi(-{z}) = {ln_lo}, want {want_ln}");
            assert!((ln_hi - (-want).ln_1p()).abs() < 1e-15, "ln Phi({z})");
            let m = mills(-z);
            assert!(close(m, want_mills, 1e-13), "mills(-{z}) = {m}, want {want_mills}");
        }
        eprintln!("largest relative error of Phi(-z), z in 0..40: {worst:e}");
        // Far out, in logs only.
        for z in [50.0, 100.0, 1000.0] {
            let want = -0.5 * z * z + (reference_mills_ratio(z) * FRAC_1_SQRT_2PI).ln();
            assert!(close(ln_cdf(-z), want, 1e-14), "ln Phi(-{z})");
            assert!(close(mills(-z), 1.0 / reference_mills_ratio(z), 1e-13), "mills(-{z})");
        }
    }

    /// Mass, mean and variance of the standard normal truncated to [lo, hi]
    /// by Simpson's rule, scaled by the density at the point of the interval
    /// nearest 0 so that nothing underflows: (ln Z, mean, variance).
    fn integrate(lo: f64, hi: f64) -> (f64, f64, f64) {
        let c = if lo > 0.0 { lo } else if hi < 0.0 { hi } else { 0.0 };
        let (a, b) = (lo.max(c - 40.0), hi.min(c + 40.0));
        let n = 400_000;
        let h = (b - a) / n as f64;
        let (mut z, mut m1, mut m2) = (0.0, 0.0, 0.0);
        for i in 0..=n {
            let x = a + i as f64 * h;
            let k = if i == 0 || i == n { 1.0 } else if i % 2 == 1 { 4.0 } else { 2.0 };
            let f = k * (-0.5 * (x - c) * (x + c)).exp();
            z += f;
            m1 += f * x;
            m2 += f * x * x;
        }
        let mean = m1 / z;
        let ln_z = (z * h / 3.0).ln() - 0.5 * c * c + FRAC_1_SQRT_2PI.ln();
        (ln_z, mean, m2 / z - mean * mean)
    }

    #[test]
    fn truncation_matches_numerical_integration() {
        let inf = f64::INFINITY;
        let cases = [
            // One-sided (wins and losses): from deep in the tail to the bulk.
            (-inf, -30.0),
            (-inf, -8.0),
            (-inf, -2.5),
            (-inf, -0.3),
            (-inf, 0.0),
            (-inf, 0.5),
            (-inf, 3.0),
            (-inf, 9.0),
            (-30.0, inf),
            (-2.0, inf),
            (0.2, inf),
            (4.0, inf),
            (25.0, inf),
            // Two-sided (draws): around 0, and both ends in either tail.
            (-0.3, 0.3),
            (-1.5, 0.2),
            (-0.01, 0.01),
            (-3.0, -1.0),
            (-12.0, -11.0),
            (-35.0, -34.2),
            (1.0, 2.5),
            (7.0, 9.0),
            (30.0, 30.5),
            (-0.5, 6.0),
        ];
        for (lo, hi) in cases {
            let t = truncated(lo, hi);
            let (v, w) = t.v_w();
            let (ln_z, mean, var) = integrate(lo, hi);
            assert!((v - mean).abs() < 1e-8 * mean.abs().max(1.0), "[{lo}, {hi}]: v {v}, integrated mean {mean}");
            assert!((1.0 - w - var).abs() < 1e-8, "[{lo}, {hi}]: 1 - w {}, integrated variance {var}", 1.0 - w);
            // The mass, by the same functions predictions use.
            let ln_mass = if lo == f64::NEG_INFINITY {
                ln_cdf(hi)
            } else if hi == inf {
                ln_cdf(-lo)
            } else if hi <= 0.0 {
                ln_cdf(hi) + ln_one_minus_exp(ln_cdf(lo) - ln_cdf(hi))
            } else if lo >= 0.0 {
                ln_cdf(-lo) + ln_one_minus_exp(ln_cdf(-hi) - ln_cdf(-lo))
            } else {
                (-(cdf(lo) + cdf(-hi))).ln_1p()
            };
            assert!((ln_mass - ln_z).abs() < 1e-8 * ln_z.abs().max(1.0), "[{lo}, {hi}]: ln Z {ln_mass}, integrated {ln_z}");
        }
    }

    #[test]
    fn tempered_update_is_the_scaled_message() {
        // The message from a truncation to the latent N(m, s2), as natural
        // parameters, scaled by p and added back: the posterior it gives.
        let (m, s2): (f64, f64) = (0.7, 2.3);
        let s = s2.sqrt();
        for (lo, hi) in [(-0.4, f64::INFINITY), (f64::NEG_INFINITY, 0.9), (-1.2, -0.1)] {
            let t = truncated(lo, hi);
            let (v, w) = t.v_w();
            let (post_m, post_v) = (m + s * v, s2 * (1.0 - w));
            let (pi, tau) = (1.0 / post_v - 1.0 / s2, post_m / post_v - m / s2);
            for p in [1.0, 0.5, 0.05] {
                let prec = 1.0 / s2 + p * pi;
                let mean = (m / s2 + p * tau) / prec;
                let (vp, wp) = t.tempered(p);
                assert!(close(m + s * vp, mean, 1e-12), "p {p}: mean {} vs {mean}", m + s * vp);
                assert!(close(s2 * (1.0 - wp), 1.0 / prec, 1e-12), "p {p}: variance");
            }
        }
    }

    /// One update of a model of two weights, against the exact posterior's
    /// marginal moments by numerical integration: assumed-density filtering
    /// keeps each weight's exact mean and variance after one observation.
    #[test]
    fn one_update_matches_the_exact_marginal_posterior() {
        let params =
            ProbitParams { prior: 0.8, beta: 0.6, tau: 0.0, alpha: 1.0, per_game: None, eps: 0.3, eps_rate: 0.0 };
        // One feature of 2 values: weights are [bias, value 0, value 1].
        for class in 0..3 {
            let mut model = Probit::new(&[2], params);
            model.weights = vec![[0.2, 0.5], [-0.4, 0.3], [0.9, 0.7]];
            let mut label = [0.0; 3];
            label[class] = 1.0;
            let before = model.clone();
            model.train(&[1], &label);
            // Exact: integrate over the bias b and the used weight u (the
            // third, unused, is unchanged); t = b + u + N(0, beta^2).
            let ([mb, vb], [mu, vu]) = (before.weights[0], before.weights[2]);
            let (eps, beta) = (0.3, 0.6);
            let lik = |x: f64| match class {
                0 => cdf((x - eps) / beta),
                1 => cdf((-x - eps) / beta),
                _ => cdf((eps - x) / beta) - cdf((-eps - x) / beta),
            };
            let n = 1200;
            let (sb, su) = (vb.sqrt(), vu.sqrt());
            let (mut z, mut eb, mut eb2, mut eu, mut eu2) = (0.0, 0.0, 0.0, 0.0, 0.0);
            for i in 0..=n {
                let b = mb + sb * (-8.0 + 16.0 * i as f64 / n as f64);
                for j in 0..=n {
                    let u = mu + su * (-8.0 + 16.0 * j as f64 / n as f64);
                    let f = (-0.5 * ((b - mb) * (b - mb) / vb + (u - mu) * (u - mu) / vu)).exp() * lik(b + u);
                    z += f;
                    eb += f * b;
                    eb2 += f * b * b;
                    eu += f * u;
                    eu2 += f * u * u;
                }
            }
            let (eb, eu) = (eb / z, eu / z);
            let (vb2, vu2) = (eb2 / z - eb * eb, eu2 / z - eu * eu);
            let [nb, nvb] = model.weights[0];
            let [nu, nvu] = model.weights[2];
            assert!((nb - eb).abs() < 1e-6 && (nu - eu).abs() < 1e-6, "class {class}: means {nb} {nu} vs {eb} {eu}");
            assert!((nvb - vb2).abs() < 1e-6 && (nvu - vu2).abs() < 1e-6, "class {class}: variances");
            assert_eq!(model.weights[1], before.weights[1], "the unused weight is unchanged");
        }
    }

    /// Positions of 3 features whose results come from a known ordinal
    /// probit; the model learns it (with alpha 1: independent examples).
    #[test]
    fn learns_a_known_ordinal_probit() {
        let truth = |x: &[u8]| [0.0, 0.8, -0.8][x[0] as usize] + [0.5, -0.5][x[1] as usize];
        let (eps, beta) = (0.4, 1.0);
        let params = ProbitParams { prior: 1.0, beta, tau: 0.0, alpha: 1.0, per_game: None, eps: 1.0, eps_rate: 1e-3 };
        let mut model = Probit::new(&[3, 2, 4], params);
        let mut seed = 12345u64;
        let mut uniform = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (seed >> 11) as f64 / (1u64 << 53) as f64
        };
        for _ in 0..200_000 {
            let x = [(uniform() * 3.0) as u8, (uniform() * 2.0) as u8, (uniform() * 4.0) as u8];
            // t = truth + N(0, 1) by inverting Phi with bisection.
            let u = uniform();
            let (mut a, mut b) = (-10.0, 10.0);
            for _ in 0..60 {
                let mid = 0.5 * (a + b);
                if cdf(mid) < u { a = mid } else { b = mid }
            }
            let t = truth(&x) + beta * a;
            let label = if t > eps { [1.0, 0.0, 0.0] } else if t < -eps { [0.0, 1.0, 0.0] } else { [0.0, 0.0, 1.0] };
            model.train(&x, &label);
        }
        assert!((model.eps() - eps).abs() < 0.05, "eps {}", model.eps());
        for x in [[0u8, 0, 0], [1, 0, 3], [2, 1, 2], [1, 1, 1]] {
            let m = truth(&x);
            let want = [cdf((m - eps) / beta), cdf((-m - eps) / beta)];
            let mut got = [0.0; 3];
            model.predict(&x, &mut got);
            assert!((got[0] - want[0]).abs() < 0.02 && (got[1] - want[1]).abs() < 0.02, "{x:?}: {got:?} vs {want:?}");
            let mut logs = [0.0; 3];
            model.predict_log(&x, &mut logs);
            for k in 0..3 {
                assert!((logs[k].exp() - got[k]).abs() < 1e-12, "{x:?}: class {k}");
            }
        }
    }

    #[test]
    fn predictions_are_consistent_far_into_the_tails() {
        let params = ProbitParams { prior: 0.1, beta: 1.0, tau: 0.0, alpha: 1.0, per_game: None, eps: 0.5, eps_rate: 0.0 };
        let mut model = Probit::new(&[2], params);
        for bias in [-60.0, -9.0, -0.6, -0.1, 0.0, 0.2, 0.6, 9.0, 60.0] {
            model.weights[0][0] = bias;
            let (mut p, mut l) = ([0.0; 3], [0.0; 3]);
            model.predict(&[0], &mut p);
            model.predict_log(&[0], &mut l);
            assert!((p.iter().sum::<f64>() - 1.0).abs() < 1e-15, "{bias}: {p:?}");
            for k in 0..3 {
                assert!(l[k].is_finite(), "{bias}: {l:?}");
                if p[k] > 1e-300 {
                    assert!(close(l[k].exp(), p[k], 1e-9), "{bias}: class {k}: {} vs {}", l[k].exp(), p[k]);
                }
            }
            // Symmetry: the mirrored position swaps white and black.
            let mut mirrored = model.clone();
            mirrored.weights[0][0] = -bias;
            let mut q = [0.0; 3];
            mirrored.predict_log(&[0], &mut q);
            assert!(close(l[0], q[1], 1e-12) && close(l[2], q[2], 1e-12), "{bias}: {l:?} vs {q:?}");
        }
    }

    #[test]
    fn tempering_and_per_game_power() {
        let params = ProbitParams { prior: 0.5, beta: 1.0, tau: 0.0, alpha: 0.1, per_game: None, eps: 0.3, eps_rate: 0.0 };
        let mut full = Probit::new(&[3], ProbitParams { alpha: 1.0, ..params });
        let mut tempered = Probit::new(&[3], params);
        full.train(&[1], &[1.0, 0.0, 0.0]);
        tempered.train(&[1], &[1.0, 0.0, 0.0]);
        let gain = |m: &Probit| m.weights[2][0];
        assert!(gain(&tempered) > 0.0 && gain(&tempered) < gain(&full) / 3.0, "{} {}", gain(&tempered), gain(&full));
        assert!(tempered.weights[2][1] > full.weights[2][1]);
        // per_game 2 on a game of 20 positions is power 0.1.
        let mut per_game = Probit::new(&[3], ProbitParams { per_game: Some(2.0), ..params });
        let game = [[1u8; crate::encoding::FEATURES]; 20];
        per_game.observe_games(&[&game[..]]);
        per_game.train_in_game(&[1], &[1.0, 0.0, 0.0], 7 << 32);
        assert_eq!(per_game.weights, tempered.weights);
        // A uniform label of a balanced prediction barely moves the mean.
        let mut soft = Probit::new(&[3], ProbitParams { alpha: 1.0, ..params });
        soft.train(&[1], &[1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0]);
        assert!(soft.weights[2][0].abs() < 0.05, "{}", soft.weights[2][0]);
    }

    #[test]
    fn text_round_trip() {
        let params =
            ProbitParams { prior: 0.3, beta: 1.0, tau: 0.01, alpha: 0.05, per_game: Some(2.5), eps: 0.4, eps_rate: 1e-3 };
        let mut model = Probit::new(&[2, 3], params);
        model.train(&[1, 2], &[0.2, 0.3, 0.5]);
        model.train(&[0, 1], &[1.0, 0.0, 0.0]);
        model.train(&[1, 1], &[0.0, 0.0, 1.0]);
        let back = Probit::from_text(&model.to_text()).unwrap();
        assert_eq!(back.to_text(), model.to_text());
        assert_eq!(back.params(), model.params());
        let (mut a, mut b) = ([0.0; 3], [0.0; 3]);
        model.predict(&[1, 1], &mut a);
        back.predict(&[1, 1], &mut b);
        assert_eq!(a, b);
        // Training on from the file is training on.
        let (mut m1, mut m2) = (model.clone(), back);
        for (x, l) in [([0u8, 2u8], [0.0, 1.0, 0.0]), ([1, 0], [0.0, 0.0, 1.0])] {
            m1.train(&x, &l);
            m2.train(&x, &l);
        }
        assert_eq!(m1.to_text(), m2.to_text());
        let none = Probit::new(&[2], ProbitParams { per_game: None, ..params });
        assert_eq!(Probit::from_text(&none.to_text()).unwrap().params().per_game, None);
    }
}
