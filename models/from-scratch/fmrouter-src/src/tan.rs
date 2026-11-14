//! Tree-augmented naive Bayes (TAN; Friedman, Geiger and Goldszmidt 1997):
//! a CNBP whose features each also condition on one other feature, their
//! parent, chosen from the data.
//!
//! Like CNBP it learns by counting: CNBP's counts, and the count of every
//! pair of feature values with each class, N(x_i, x_j, c). At each `refresh`
//! the parents become the Chow-Liu tree of those counts: the maximum spanning
//! tree of the features' conditional mutual information given the class,
//! I(X_i; X_j | C), rooted at the feature most informative about the class,
//! I(X; C). Ties go to the lowest feature index, so the tree is a function of
//! the counts. Any tree's tables are sums of the same counts, so a new tree
//! needs no retraining.
//!
//! P(c | x) is proportional to P(c) times, for each feature, P(x_i | x_p, c)
//! given its parent p: the m-estimate (N(x_i, x_p, c) + m P(x_i | c)) /
//! (N(x_p, c) + m), which backs off to CNBP's own Laplace estimate P(x_i | c)
//! where the parent's value is rare (and is exactly it where the parent's
//! value was never seen with the class). The root has no parent and uses
//! P(x_i | c). Without parents the predictions are CNBP's, bit for bit.
//!
//! m is fixed, or with `relative` it is λ N, N the positions counted so far
//! (the sum of CNBP's class counts), recomputed with the tables: a parent
//! value's own evidence then weighs N(x_p, c) / (N(x_p, c) + λ N), which
//! stays put as the counts grow, where a fixed m's drifts to 1.
//!
//! With `games` (an experiment that did worse), the pair counts count games,
//! not positions: a game of L positions adds 1/L for each (see
//! `train_in_game`), m is in games, and the tree keeps only the edges whose
//! dependence is significant (a chi-squared test of independence with the
//! number of games as the sample size, at 5% over all the pairs), the rest
//! being naive Bayes's: a forest.
//!
//! The model file (binary, in the `state` format) holds what predicting
//! needs: CNBP's counts, the tree, and each feature's counts with its parent.
//! Training on needs every pair count (~8.6 MB for the 70 chess features),
//! which only the state file holds.

use std::io;
use std::path::Path;

use crate::cnbp::Cnbp;
use crate::state::{Reader, Writer};
use crate::textfmt::invalid;

/// No parent: the root, or every feature of a model without a tree.
const NONE: usize = usize::MAX;

/// The m-estimate's default weight: Friedman, Geiger and Goldszmidt's
/// smoothed TAN uses 5.
pub const DEFAULT_M: f64 = 5.0;

/// How a TAN counts its pairs and weighs its m-estimate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TanOptions {
    /// m; with `relative`, λ: m = λ N, N the positions counted so far.
    pub m: f64,
    pub relative: bool,
    /// Count the pairs in games and keep only significant edges (see the
    /// module's comment).
    pub games: bool,
}

impl TanOptions {
    /// Positions, a fixed m.
    pub fn fixed(m: f64) -> Self {
        TanOptions { m, relative: false, games: false }
    }

    /// The state file's kind, which tells the options apart.
    fn state_kind(&self) -> &'static str {
        match (self.games, self.relative) {
            (false, false) => "tan",
            (false, true) => "tan-rel",
            (true, false) => "tan-games",
            (true, true) => "tan-games-rel",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Tan {
    nb: Cnbp,
    opts: TanOptions,
    /// The weight of CNBP's estimate in each feature's m-estimate in the
    /// tables: `opts.m`, or λ N.
    m: f64,
    n: usize,
    cats: Vec<usize>,
    /// For i < j, `pair_off[i * n + j]`: the start of pair (i, j)'s counts in
    /// a class's block of the state file, `x_i * cats[j] + x_j` from there.
    pair_off: Vec<usize>,
    pair_len: usize,
    /// `cum[j]`: the number of values of the features before j.
    cum: Vec<usize>,
    /// The number of values of the features after i.
    stride: Vec<usize>,
    /// Where feature i's row of counts with the features after it starts
    /// (see `cell`).
    row_off: Vec<usize>,
    /// `class_num` blocks of `pair_len` counts, laid out by `cell`. Empty in
    /// a model loaded from a model file, which can't train.
    pairs: Vec<f64>,
    trainable: bool,
    /// Whether `refresh` learns the tree; false: no parents, as CNBP.
    learn: bool,
    parents: Vec<usize>,
    /// Start of each feature's table: `cats[parent] * cats[i]` entries,
    /// `x_parent * cats[i] + x_i` from there (no entries for features without
    /// a parent, which use CNBP's).
    tab_off: Vec<usize>,
    tab_len: usize,
    /// `class_num` blocks of `tab_len`: each feature's counts with its parent.
    cond: Vec<f64>,
    /// Start of each feature's parent-value counts: `cats[parent]` entries.
    ctx_off: Vec<usize>,
    ctx_len: usize,
    /// `class_num` blocks of `ctx_len`: N(x_parent, c), summed from `cond`.
    ctx: Vec<f64>,
    /// With `use_probs`, what predicting reads (see `Look`): each feature's
    /// P(x_i | x_parent, c), or CNBP's P(x_i | c) without a parent, the
    /// classes side by side, so that one cache line serves every class.
    probs: Vec<f64>,
    /// Their logs, laid out as `probs`.
    logs: Vec<f64>,
    look: Vec<Look>,
    /// The number of positions of each game of the last `observe_games`.
    game_len: Vec<usize>,
    fresh: bool,
}

/// Where feature i's entries are in `Tan::probs` and `Tan::logs`: value a
/// given the value b of feature `par` at `(off + b * step + a) * class_num +
/// c`. A feature without a parent has one row: `step` 0, any `par`.
#[derive(Clone, Copy, Debug)]
struct Look {
    off: usize,
    step: usize,
    par: usize,
}

impl Tan {
    pub fn new(class_num: usize, categories: &[u8], alpha: f64, opts: TanOptions, use_probs: bool) -> Self {
        let mut tan = Tan::blank(Cnbp::new(class_num, categories, alpha, use_probs), opts);
        tan.pairs = vec![0.0; class_num * tan.pair_len];
        tan.trainable = true;
        tan.refresh();
        tan
    }

    /// A model around `nb` with no pair counts, no parents and no tables.
    fn blank(nb: Cnbp, opts: TanOptions) -> Self {
        let cats: Vec<usize> = nb.categories().iter().map(|&c| c as usize).collect();
        let n = cats.len();
        let mut pair_off = vec![0; n * n];
        let mut pair_len = 0;
        for i in 0..n {
            for j in i + 1..n {
                pair_off[i * n + j] = pair_len;
                pair_len += cats[i] * cats[j];
            }
        }
        let mut cum = vec![0; n + 1];
        for i in 0..n {
            cum[i + 1] = cum[i] + cats[i];
        }
        let stride: Vec<usize> = (0..n).map(|i| cum[n] - cum[i + 1]).collect();
        let mut row_off = vec![0; n];
        for i in 1..n {
            row_off[i] = row_off[i - 1] + cats[i - 1] * stride[i - 1];
        }
        Tan {
            nb,
            opts,
            m: opts.m,
            n,
            cats,
            pair_off,
            pair_len,
            cum,
            stride,
            row_off,
            pairs: Vec::new(),
            trainable: false,
            learn: true,
            parents: vec![NONE; n],
            tab_off: vec![0; n],
            tab_len: 0,
            cond: Vec::new(),
            ctx_off: vec![0; n],
            ctx_len: 0,
            ctx: Vec::new(),
            probs: Vec::new(),
            logs: Vec::new(),
            look: Vec::new(),
            game_len: Vec::new(),
            fresh: false,
        }
    }

    pub fn class_num(&self) -> usize {
        self.nb.class_num()
    }

    pub fn alpha(&self) -> f64 {
        self.nb.alpha()
    }

    /// The m of the tables (see `TanOptions`).
    pub fn m(&self) -> f64 {
        self.m
    }

    pub fn options(&self) -> TanOptions {
        self.opts
    }

    pub fn use_probs(&self) -> bool {
        self.nb.use_probs()
    }

    /// Each feature's parent (None: the root).
    pub fn parents(&self) -> Vec<Option<usize>> {
        self.parents.iter().map(|&p| (p != NONE).then_some(p)).collect()
    }

    /// Adds one position as a game of its own. `label` must sum to 1.
    pub fn train(&mut self, data: &[u8], label: &[f64]) {
        self.add(data, label, 1.0);
    }

    /// Notes the games to be trained on next, each its positions: the game
    /// of `train_in_game` is its index here (with `games`).
    pub fn observe_games<P>(&mut self, games: &[&[P]]) {
        if self.opts.games {
            self.game_len = games.iter().map(|g| g.len()).collect();
        }
    }

    /// Adds one position of a game: as `train`, or with `games`, of a game
    /// of the last `observe_games`, its index in the low 32 bits of `game`
    /// (self-play's game ids): CNBP's counts count the position once, the
    /// pair counts 1/L for a game of L positions.
    pub fn train_in_game(&mut self, data: &[u8], label: &[f64], game: u64) {
        if !self.opts.games {
            return self.add(data, label, 1.0);
        }
        let len = self.game_len.get((game & 0xFFFF_FFFF) as usize).copied().unwrap_or(0);
        assert!(len > 0, "a TAN trains on the positions of the games it observed (observe_games)");
        self.add(data, label, 1.0 / len as f64);
    }

    /// Adds one position to CNBP's counts, and `weight` times to the pair
    /// counts.
    fn add(&mut self, data: &[u8], label: &[f64], weight: f64) {
        assert!(self.trainable, "a TAN loaded from a model file can't train: load its state file");
        self.nb.train_weighted(data, label, 1.0);
        let Tan { n, cum, stride, row_off, pair_len, pairs, .. } = self;
        let (n, pair_len) = (*n, *pair_len);
        for (cl, block) in pairs.chunks_exact_mut(pair_len.max(1)).enumerate() {
            let l = label[cl] * weight;
            // Adding 0 would change no count.
            if l == 0.0 || pair_len == 0 {
                continue;
            }
            // One row for each feature: its value's counts with the values
            // of the features after it, added to in order.
            for i in 0..n {
                let row = &mut block[row_off[i] + data[i] as usize * stride[i]..][..stride[i]];
                let start = cum[i + 1];
                for (&c, &x) in cum[i + 1..n].iter().zip(&data[i + 1..n]) {
                    row[c - start + x as usize] += l;
                }
            }
        }
        self.fresh = false;
    }

    /// Where N(x_i = a, x_j = b) is in a class's block of `pairs`, for i < j.
    fn cell(&self, i: usize, j: usize, a: usize, b: usize) -> usize {
        self.row_off[i] + a * self.stride[i] + self.cum[j] - self.cum[i + 1] + b
    }

    /// The pair counts laid out as the state file holds them (see
    /// `pair_off`), from `pairs` (`to_file`) or back.
    fn convert_pairs(&self, from: &[f64], to_file: bool) -> Vec<f64> {
        let (n, len) = (self.n, self.pair_len);
        let mut to = vec![0.0; from.len()];
        for (src, dst) in from.chunks_exact(len.max(1)).zip(to.chunks_exact_mut(len.max(1))) {
            for i in 0..n {
                for j in i + 1..n {
                    let cj = self.cats[j];
                    for a in 0..self.cats[i] {
                        let (mem, file) = (self.cell(i, j, a, 0), self.pair_off[i * n + j] + a * cj);
                        let (s, d) = if to_file { (mem, file) } else { (file, mem) };
                        dst[d..d + cj].copy_from_slice(&src[s..s + cj]);
                    }
                }
            }
        }
        to
    }

    /// Divides all counts by `factor`, keeping predictions roughly the same
    /// but letting new training move them faster.
    pub fn forget(&mut self, factor: f64) {
        assert!(self.trainable, "a TAN loaded from a model file can't forget: load its state file");
        self.nb.forget(factor);
        for t in &mut self.pairs {
            *t /= factor;
        }
        self.fresh = false;
        self.refresh();
    }

    /// After training: learns the tree from the counts and recomputes the
    /// tables. Needed before predicting, with or without `use_probs`.
    pub fn refresh(&mut self) {
        if self.fresh {
            return;
        }
        self.nb.refresh();
        if self.trainable {
            if self.learn {
                self.parents = self.chow_liu();
            }
            self.layout();
            self.extract();
        }
        self.tables();
        self.fresh = true;
    }

    fn parent(&self, i: usize) -> Option<usize> {
        (self.parents[i] != NONE).then_some(self.parents[i])
    }

    /// The tables' layout for the parents.
    fn layout(&mut self) {
        let (mut tab, mut ctx) = (0, 0);
        for i in 0..self.n {
            self.tab_off[i] = tab;
            self.ctx_off[i] = ctx;
            if let Some(p) = self.parent(i) {
                tab += self.cats[p] * self.cats[i];
                ctx += self.cats[p];
            }
        }
        (self.tab_len, self.ctx_len) = (tab, ctx);
    }

    /// Each feature's counts with its parent, from the pair counts.
    fn extract(&mut self) {
        let (n, class_num) = (self.n, self.nb.class_num());
        let mut cond = vec![0.0; class_num * self.tab_len];
        for cl in 0..class_num {
            let pairs = &self.pairs[cl * self.pair_len..(cl + 1) * self.pair_len];
            let block = &mut cond[cl * self.tab_len..(cl + 1) * self.tab_len];
            for i in 0..n {
                let Some(p) = self.parent(i) else { continue };
                let (ci, cp, t) = (self.cats[i], self.cats[p], self.tab_off[i]);
                for b in 0..cp {
                    for a in 0..ci {
                        block[t + b * ci + a] =
                            if i < p { pairs[self.cell(i, p, a, b)] } else { pairs[self.cell(p, i, b, a)] };
                    }
                }
            }
        }
        self.cond = cond;
    }

    /// The parent-value counts, and with `use_probs` the probabilities and
    /// their logs, from `cond`; and the m they use.
    fn tables(&mut self) {
        let class_num = self.nb.class_num();
        self.m = if self.opts.relative {
            self.opts.m * self.nb.class_counts().iter().sum::<f64>()
        } else {
            self.opts.m
        };
        let mut ctx = vec![0.0; class_num * self.ctx_len];
        for cl in 0..class_num {
            for i in 0..self.n {
                let Some(p) = self.parent(i) else { continue };
                let (ci, t, o) = (self.cats[i], cl * self.tab_len + self.tab_off[i], cl * self.ctx_len + self.ctx_off[i]);
                for b in 0..self.cats[p] {
                    ctx[o + b] = self.cond[t + b * ci..t + (b + 1) * ci].iter().sum();
                }
            }
        }
        self.ctx = ctx;
        if !self.nb.use_probs() {
            (self.probs, self.logs, self.look) = (Vec::new(), Vec::new(), Vec::new());
            return;
        }
        let mut look = Vec::with_capacity(self.n);
        let mut len = 0;
        for i in 0..self.n {
            let (step, par, rows) = match self.parent(i) {
                Some(p) => (self.cats[i], p, self.cats[p]),
                None => (0, 0, 1),
            };
            look.push(Look { off: len, step, par });
            len += rows * self.cats[i];
        }
        let mut probs = vec![0.0; class_num * len];
        let mut logs = vec![0.0; class_num * len];
        for cl in 0..class_num {
            for i in 0..self.n {
                let (ci, l) = (self.cats[i], look[i]);
                let Some(p) = self.parent(i) else {
                    for a in 0..ci {
                        let k = (l.off + a) * class_num + cl;
                        (probs[k], logs[k]) = (self.nb.cat_prob(cl, i, a as u8), self.nb.cat_log(cl, i, a as u8));
                    }
                    continue;
                };
                let (t, o) = (cl * self.tab_len + self.tab_off[i], cl * self.ctx_len + self.ctx_off[i]);
                for b in 0..self.cats[p] {
                    let nbc = self.ctx[o + b];
                    for a in 0..ci {
                        let k = (l.off + b * l.step + a) * class_num + cl;
                        let nb = self.nb.cat_prob(cl, i, a as u8);
                        if nbc == 0.0 {
                            (probs[k], logs[k]) = (nb, self.nb.cat_log(cl, i, a as u8));
                        } else {
                            probs[k] = (self.cond[t + b * ci + a] + self.m * nb) / (nbc + self.m);
                            logs[k] = probs[k].ln();
                        }
                    }
                }
            }
        }
        (self.probs, self.logs, self.look) = (probs, logs, look);
    }

    /// For each class c, `acc[c]` combined by `f` (times, or plus for the
    /// logs) with each feature's entry of `table` (`probs` or `logs`) in
    /// feature order: class by class, the arithmetic of a loop over the
    /// features for one class at a time, as CNBP's.
    #[inline(always)]
    fn fold<const C: usize>(&self, data: &[u8], table: &[f64], acc: &mut [f64; C], f: impl Fn(f64, f64) -> f64) {
        for (&a, l) in data.iter().zip(&self.look) {
            let k = (l.off + data[l.par] as usize * l.step + a as usize) * C;
            let row = &table[k..k + C];
            for c in 0..C {
                acc[c] = f(acc[c], row[c]);
            }
        }
    }

    /// The Chow-Liu forest of the counts (see the module's comment). With no
    /// counts, no parents: a model that has seen nothing predicts as CNBP
    /// (a tree's TAN router predicts from its counts before its first
    /// refresh, with this forest).
    fn chow_liu(&self) -> Vec<usize> {
        let (n, class_num) = (self.n, self.nb.class_num());
        let class_counts = self.nb.class_counts();
        let total: f64 = class_counts.iter().sum();
        if total == 0.0 {
            return vec![NONE; n];
        }
        let mut w = vec![0.0; n * n];
        let mut root = 0;
        if total > 0.0 {
            let (counts, stride) = (self.nb.cat_counts(), self.nb.total_cat_vals());
            let mut root_mi = f64::NEG_INFINITY;
            for i in 0..n {
                let off = self.nb.cat_offset(i);
                let mut mi = 0.0;
                for a in 0..self.cats[i] {
                    let na: f64 = (0..class_num).map(|c| counts[c * stride + off + a]).sum();
                    for (c, &nc) in class_counts.iter().enumerate() {
                        let nac = counts[c * stride + off + a];
                        if nac > 0.0 {
                            mi += nac / total * (nac * total / (na * nc)).ln();
                        }
                    }
                }
                if mi > root_mi {
                    (root, root_mi) = (i, mi);
                }
            }
            let max_cat = self.cats.iter().copied().max().unwrap_or(0);
            let (mut rows, mut cols) = (vec![0.0; max_cat], vec![0.0; max_cat]);
            // With games: significant at 5% over all the pairs together
            // (Bonferroni).
            let z = self.opts.games.then(|| normal_quantile(SIGNIFICANCE / (n * n.saturating_sub(1) / 2).max(1) as f64));
            let mut counts = Vec::with_capacity(class_num * max_cat * max_cat);
            for i in 0..n {
                for j in i + 1..n {
                    let v = self.pair_weight(i, j, z, &mut counts, &mut rows, &mut cols);
                    (w[i * n + j], w[j * n + i]) = (v, v);
                }
            }
        }
        prim(n, root, &w, self.opts.games)
    }

    /// The weight of the edge between features i < j: I(X_i; X_j | C) in
    /// nats, from the pair counts. With `z`, the test's normal quantile (see
    /// `chi2_critical`), less the most that independence gives it but
    /// rarely, and at least 0: the critical value of the test of
    /// independence, 2 N I against a chi-squared with sum over the classes
    /// seen of (k_i - 1) (k_j - 1) degrees of freedom (k: the values seen),
    /// N the total count, the number of games; positive only if the
    /// dependence is significant. `counts`, `rows` and `cols` are scratch
    /// space.
    fn pair_weight(&self, i: usize, j: usize, z: Option<f64>, counts: &mut Vec<f64>, rows: &mut [f64], cols: &mut [f64]) -> f64 {
        let (ci, cj) = (self.cats[i], self.cats[j]);
        // Each class's counts of the pair, N(x_i = a, x_j = b) at a * cj + b.
        counts.clear();
        for block in self.pairs.chunks_exact(self.pair_len.max(1)) {
            for a in 0..ci {
                let s = self.cell(i, j, a, 0);
                counts.extend_from_slice(&block[s..s + cj]);
            }
        }
        let len = (ci * cj).max(1);
        let total: f64 = counts.chunks_exact(len).map(|block| block.iter().sum::<f64>()).sum();
        if total <= 0.0 {
            return 0.0;
        }
        let (mut cmi, mut df) = (0.0, 0usize);
        for block in counts.chunks_exact(len) {
            rows[..ci].fill(0.0);
            cols[..cj].fill(0.0);
            for a in 0..ci {
                for b in 0..cj {
                    rows[a] += block[a * cj + b];
                    cols[b] += block[a * cj + b];
                }
            }
            let nc: f64 = rows[..ci].iter().sum();
            if nc <= 0.0 {
                continue;
            }
            let seen = |xs: &[f64]| xs.iter().filter(|&&x| x > 0.0).count();
            df += (seen(&rows[..ci]) - 1) * (seen(&cols[..cj]) - 1);
            for a in 0..ci {
                for b in 0..cj {
                    let nabc = block[a * cj + b];
                    if nabc > 0.0 {
                        cmi += nabc / total * (nabc * nc / (rows[a] * cols[b])).ln();
                    }
                }
            }
        }
        match z {
            None => cmi,
            Some(_) if df == 0 => 0.0,
            Some(z) => (cmi - chi2_critical(df as f64, z) / (2.0 * total)).max(0.0),
        }
    }

    /// Writes the probability of each class into `out`.
    ///
    /// # Panics
    /// If the model was trained since the last `refresh()`.
    pub fn predict(&self, data: &[u8], out: &mut [f64]) {
        assert!(self.fresh, "tan trained without refresh() before predict()");
        let class_num = self.nb.class_num();
        if self.nb.use_probs() && class_num == 3 {
            let mut p = [self.nb.class_prob(0), self.nb.class_prob(1), self.nb.class_prob(2)];
            self.fold(data, &self.probs, &mut p, |x, y| x * y);
            let mut total = 0.0;
            for &x in &p {
                total += x;
            }
            for (o, x) in out.iter_mut().zip(p) {
                *o = x / total;
            }
            return;
        }
        let mut total = 0.0;
        for (cl, o) in out.iter_mut().enumerate().take(class_num) {
            let mut p = if self.nb.use_probs() { self.nb.class_prob(cl) } else { self.prior(cl) };
            for (i, &a) in data.iter().enumerate().take(self.n) {
                p *= self.factor(cl, i, a, data);
            }
            *o = p;
            total += p;
        }
        for p in &mut out[..class_num] {
            *p /= total;
        }
    }

    /// Writes ln P(class) + the sum of each feature's ln P(value | parent's
    /// value, class): the log of the unnormalised posterior.
    ///
    /// # Panics
    /// If the model was trained since the last `refresh()`.
    pub fn predict_log(&self, data: &[u8], out: &mut [f64]) {
        assert!(self.fresh, "tan trained without refresh() before predict_log()");
        let use_probs = self.nb.use_probs();
        if use_probs && self.nb.class_num() == 3 {
            let mut l = [self.nb.class_log(0), self.nb.class_log(1), self.nb.class_log(2)];
            self.fold(data, &self.logs, &mut l, |x, y| x + y);
            for (o, x) in out.iter_mut().zip(l) {
                *o = x;
            }
            return;
        }
        for (cl, o) in out.iter_mut().enumerate().take(self.nb.class_num()) {
            let mut l = if use_probs { self.nb.class_log(cl) } else { self.prior(cl).ln() };
            for (i, &a) in data.iter().enumerate().take(self.n) {
                l += if use_probs { self.log_factor(cl, i, a, data) } else { self.factor(cl, i, a, data).ln() };
            }
            *o = l;
        }
    }

    /// Whether the tables are up to date with the counts (see `refresh`).
    pub fn is_fresh(&self) -> bool {
        self.fresh
    }

    /// Whether it has its counts and can train: made by `new` or read from a
    /// state, not from a model file.
    pub fn is_trainable(&self) -> bool {
        self.trainable
    }

    /// The m the counts give now: `opts.m`, or λ N (see `TanOptions`).
    fn current_m(&self) -> f64 {
        if self.opts.relative { self.opts.m * self.nb.class_counts().iter().sum::<f64>() } else { self.opts.m }
    }

    /// Prediction while training (as a delegation tree's router or leaf):
    /// from the current counts, with the tree of the last `refresh`, which
    /// is only relearned there. Equal to `predict` right after a `refresh`
    /// (up to rounding), without recomputing the tables after every step,
    /// as `Cnbp::predict_counts`. Pair counts in positions only (no
    /// `games`): N(x_parent, c) is then CNBP's count of the parent's value.
    pub fn predict_counts(&self, data: &[u8], out: &mut [f64]) {
        assert!(self.trainable, "a TAN loaded from a model file has no counts: predict, not predict_counts");
        assert!(!self.opts.games, "predict_counts needs pair counts in positions, not games");
        let class_num = self.nb.class_num();
        let (counts, cat, stride) = (self.nb.class_counts(), self.nb.cat_counts(), self.nb.total_cat_vals());
        let total: f64 = counts.iter().sum();
        let (alpha, m) = (self.nb.alpha(), self.current_m());
        let mut best = f64::NEG_INFINITY;
        for cl in 0..class_num {
            let cc = counts[cl];
            let block = &cat[cl * stride..(cl + 1) * stride];
            let pairs = &self.pairs[cl * self.pair_len..(cl + 1) * self.pair_len];
            let mut l = ((cc + alpha) / (total + class_num as f64 * alpha)).ln();
            let mut p = 1.0f64;
            for i in 0..self.n {
                let a = data[i] as usize;
                let nb = (block[self.nb.cat_offset(i) + a] + alpha) / (cc + self.cats[i] as f64 * alpha);
                let f = match self.parent(i) {
                    None => nb,
                    Some(par) => {
                        let b = data[par] as usize;
                        let nbc = block[self.nb.cat_offset(par) + b];
                        if nbc == 0.0 {
                            nb
                        } else {
                            let joint =
                                if i < par { pairs[self.cell(i, par, a, b)] } else { pairs[self.cell(par, i, b, a)] };
                            (joint + m * nb) / (nbc + m)
                        }
                    }
                };
                p *= f;
                // Products of 70 probabilities stay far from underflow, but
                // move them into the log now and then all the same.
                if p < 1e-150 {
                    l += p.ln();
                    p = 1.0;
                }
            }
            out[cl] = l + p.ln();
            best = best.max(out[cl]);
        }
        let mut sum = 0.0;
        for o in &mut out[..class_num] {
            *o = (*o - best).exp();
            sum += *o;
        }
        for o in &mut out[..class_num] {
            *o /= sum;
        }
    }

    /// Writes what predicting needs as text, for a delegation tree's model
    /// file: the word "tan", the tables' m, CNBP's counts as CNBP nodes
    /// write them, each feature's parent (-1: none) and each feature's
    /// counts with its parent. The tables must be fresh.
    pub(crate) fn write_text(&self, s: &mut String) {
        use std::fmt::Write;
        assert!(self.fresh, "write a TAN after refresh()");
        writeln!(s, "tan").unwrap();
        writeln!(s, "{}", self.m).unwrap();
        self.nb.write_counts(s);
        for &p in &self.parents {
            writeln!(s, "{}", if p == NONE { -1 } else { p as i64 }).unwrap();
        }
        for c in &self.cond {
            writeln!(s, "{c:.6}").unwrap();
        }
    }

    /// Reads `write_text`'s format, to predict with: it can't train.
    pub(crate) fn from_tokens(
        t: &mut crate::textfmt::Tokens,
        class_num: usize,
        categories: &[u8],
        alpha: f64,
        use_probs: bool,
    ) -> io::Result<Self> {
        let word: String = t.next()?;
        if word != "tan" {
            return Err(invalid(format!("expected a TAN node, found {word}")));
        }
        let m: f64 = t.next()?;
        let mut nb = Cnbp::new(class_num, categories, alpha, use_probs);
        nb.read_counts(t)?;
        let mut tan = Tan::blank(nb, TanOptions::fixed(m));
        let parents = (0..tan.n)
            .map(|_| t.next::<i64>().map(|p| if p < 0 { NONE } else { p as usize }))
            .collect::<io::Result<Vec<usize>>>()?;
        tan.set_parents(parents)?;
        tan.layout();
        tan.cond = (0..class_num * tan.tab_len).map(|_| t.next::<f64>()).collect::<io::Result<Vec<f64>>>()?;
        tan.tables();
        tan.fresh = true;
        Ok(tan)
    }

    /// Writes the whole model, every pair count included, into a delegation
    /// tree's state file (see `save_state`).
    pub(crate) fn write_state_to(&self, w: &mut Writer) {
        assert!(self.trainable, "a TAN loaded from a model file has no training state");
        w.f64(self.opts.m);
        w.bool(self.opts.relative);
        w.bool(self.opts.games);
        w.bool(self.learn);
        self.nb.write_state(w);
        w.f64s(&self.convert_pairs(&self.pairs, true));
        w.usizes(&self.parents);
        w.bool(self.fresh);
    }

    /// Reads a model written by `write_state_to`, to train on exactly as if
    /// the run had not stopped.
    pub(crate) fn read_state_from(r: &mut Reader) -> io::Result<Self> {
        let opts = TanOptions { m: r.f64()?, relative: r.bool()?, games: r.bool()? };
        let learn = r.bool()?;
        let nb = Cnbp::read_state(r)?;
        let (pairs, parents, fresh) = (r.f64s()?, r.usizes()?, r.bool()?);
        let class_num = nb.class_num();
        let mut tan = Tan::blank(nb, opts);
        if pairs.len() != class_num * tan.pair_len {
            return Err(invalid("TAN state of the wrong shape"));
        }
        (tan.pairs, tan.trainable, tan.learn) = (tan.convert_pairs(&pairs, false), true, learn);
        tan.set_parents(parents)?;
        tan.layout();
        tan.extract();
        if fresh {
            tan.tables();
            tan.fresh = true;
        }
        Ok(tan)
    }

    /// P(class) from the counts, as CNBP computes it without `use_probs`.
    fn prior(&self, cl: usize) -> f64 {
        let counts = self.nb.class_counts();
        let class_total: f64 = counts.iter().sum();
        let alpha = self.nb.alpha();
        (counts[cl] + alpha) / (class_total + self.nb.class_num() as f64 * alpha)
    }

    /// P(x_i = a | the parent's value in `data`, class `cl`).
    fn factor(&self, cl: usize, i: usize, a: u8, data: &[u8]) -> f64 {
        if self.nb.use_probs() {
            return self.probs[self.entry(cl, i, a, data)];
        }
        // As CNBP computes it without `use_probs`.
        let alpha = self.nb.alpha();
        let count = self.nb.cat_counts()[cl * self.nb.total_cat_vals() + self.nb.cat_offset(i) + a as usize];
        let nb = (count + alpha) / (self.nb.class_counts()[cl] + self.cats[i] as f64 * alpha);
        match self.parents[i] {
            NONE => nb,
            p => {
                let b = data[p] as usize;
                let nbc = self.ctx[cl * self.ctx_len + self.ctx_off[i] + b];
                if nbc == 0.0 {
                    nb
                } else {
                    let k = cl * self.tab_len + self.tab_off[i] + b * self.cats[i] + a as usize;
                    (self.cond[k] + self.m * nb) / (nbc + self.m)
                }
            }
        }
    }

    /// `factor`'s log, cached (`use_probs`).
    fn log_factor(&self, cl: usize, i: usize, a: u8, data: &[u8]) -> f64 {
        self.logs[self.entry(cl, i, a, data)]
    }

    /// Where `probs` and `logs` hold feature i's entry for value `a` and its
    /// parent's value in `data`, class `cl` (`use_probs`).
    fn entry(&self, cl: usize, i: usize, a: u8, data: &[u8]) -> usize {
        let l = self.look[i];
        (l.off + data[l.par] as usize * l.step + a as usize) * self.nb.class_num() + cl
    }

    /// Saves what predicting needs (see the module's comment).
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let mut w = Writer::new("tan-model");
        w.usize(self.nb.class_num());
        w.f64(self.nb.alpha());
        w.f64(self.m);
        w.u8s(self.nb.categories());
        w.f64s(self.nb.class_counts());
        w.f64s(self.nb.cat_counts());
        w.usizes(&self.parents);
        w.f64s(&self.cond);
        w.save(path)
    }

    /// Loads a model file, to predict with: it can't train.
    pub fn load(path: &Path, use_probs: bool) -> io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let mut r = Reader::new(&bytes, "tan-model")?;
        let (class_num, alpha, m, categories) = (r.usize()?, r.f64()?, r.f64()?, r.u8s()?);
        let (totals, cat_totals) = (r.f64s()?, r.f64s()?);
        let (parents, cond) = (r.usizes()?, r.f64s()?);
        r.finish()?;
        let nb = Cnbp::with_counts(class_num, &categories, alpha, use_probs, totals, cat_totals)?;
        // The tables' m, whatever the options it was trained with.
        let mut tan = Tan::blank(nb, TanOptions::fixed(m));
        tan.set_parents(parents)?;
        tan.layout();
        if cond.len() != class_num * tan.tab_len {
            return Err(invalid("TAN model of the wrong shape"));
        }
        tan.cond = cond;
        tan.tables();
        tan.fresh = true;
        Ok(tan)
    }

    fn set_parents(&mut self, parents: Vec<usize>) -> io::Result<()> {
        if parents.len() != self.n || parents.iter().enumerate().any(|(i, &p)| p != NONE && (p >= self.n || p == i)) {
            return Err(invalid("TAN parents of the wrong shape"));
        }
        self.parents = parents;
        Ok(())
    }

    /// Saves the whole model, every pair count included, for `load_state`.
    pub fn save_state(&self, path: &Path) -> io::Result<()> {
        if !self.trainable {
            return Err(invalid("a TAN loaded from a model file has no training state"));
        }
        let mut w = Writer::new(self.opts.state_kind());
        w.f64(self.opts.m);
        w.bool(self.learn);
        self.nb.write_state(&mut w);
        w.f64s(&self.convert_pairs(&self.pairs, true));
        w.usizes(&self.parents);
        w.bool(self.fresh);
        w.save(path)
    }

    /// Loads a state saved by `save_state`, to train on exactly as if the run
    /// had not stopped. The options must be those it was trained with.
    pub fn load_state(path: &Path, alpha: f64, opts: TanOptions, use_probs: bool) -> io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let mut r = Reader::new(&bytes, opts.state_kind())?;
        let (saved_m, learn) = (r.f64()?, r.bool()?);
        let nb = Cnbp::read_state(&mut r)?;
        let (pairs, parents, fresh) = (r.f64s()?, r.usizes()?, r.bool()?);
        r.finish()?;
        let saved = (nb.alpha(), saved_m, nb.use_probs());
        let m = opts.m;
        if saved.0.to_bits() != alpha.to_bits() || saved.1.to_bits() != m.to_bits() || saved.2 != use_probs {
            return Err(invalid(format!(
                "the state's options differ from this run's (give the options it was trained with): saved alpha {}, m {}, use_probs {}; now alpha {alpha}, m {m}, use_probs {use_probs}",
                saved.0, saved.1, saved.2
            )));
        }
        let class_num = nb.class_num();
        let mut tan = Tan::blank(nb, opts);
        if pairs.len() != class_num * tan.pair_len {
            return Err(invalid("TAN state of the wrong shape"));
        }
        (tan.pairs, tan.trainable, tan.learn) = (tan.convert_pairs(&pairs, false), true, learn);
        tan.set_parents(parents)?;
        tan.layout();
        tan.extract();
        if fresh {
            tan.tables();
            tan.fresh = true;
        }
        Ok(tan)
    }

    /// The model to train on from `path`: a state file, or a model file with
    /// its state file (the same name, `.state`) beside it.
    pub fn load_start(path: &Path, alpha: f64, opts: TanOptions, use_probs: bool) -> io::Result<Self> {
        if path.extension().is_some_and(|e| e == "state") {
            return Self::load_state(path, alpha, opts, use_probs);
        }
        let state = path.with_extension("state");
        if !state.exists() {
            return Err(invalid(format!("a TAN trains on from its state file, and {state:?} is missing")));
        }
        Self::load_state(&state, alpha, opts, use_probs)
    }

    /// No tree: every feature without a parent, as CNBP.
    #[cfg(test)]
    fn without_tree(mut self) -> Self {
        self.learn = false;
        self.parents = vec![NONE; self.n];
        self.fresh = false;
        self.refresh();
        self
    }
}

/// The chance that independent features show an edge anywhere in the forest.
const SIGNIFICANCE: f64 = 0.05;

/// The upper `p` quantile of the standard normal distribution, for p <= 0.5
/// (Abramowitz and Stegun 26.2.23, error below 4.5e-4).
fn normal_quantile(p: f64) -> f64 {
    let t = (-2.0 * p.ln()).sqrt();
    t - (2.515517 + 0.802853 * t + 0.010328 * t * t) / (1.0 + 1.432788 * t + 0.189269 * t * t + 0.001308 * t * t * t)
}

/// The chi-squared distribution's quantile with `df` degrees of freedom at
/// the standard normal quantile `z` (Wilson and Hilferty).
fn chi2_critical(df: f64, z: f64) -> f64 {
    let v = 2.0 / (9.0 * df);
    df * (1.0 - v + z * v.sqrt()).powi(3)
}

/// The maximum spanning tree of the weights `w` (n by n) by Prim's
/// algorithm, as each node's parent from `root`; with `cut` (no weight
/// negative), the maximum spanning forest: the tree without its edges of
/// weight 0, below which a node roots a tree of its own. Ties go to the
/// lowest index, both for the node added and for its parent (the first
/// found).
fn prim(n: usize, root: usize, w: &[f64], cut: bool) -> Vec<usize> {
    let mut parents = vec![NONE; n];
    if n == 0 {
        return parents;
    }
    let mut in_tree = vec![false; n];
    in_tree[root] = true;
    let mut best: Vec<f64> = (0..n).map(|j| w[root * n + j]).collect();
    let mut from = vec![root; n];
    for _ in 1..n {
        let mut pick = NONE;
        for j in 0..n {
            if !in_tree[j] && (pick == NONE || best[j] > best[pick]) {
                pick = j;
            }
        }
        in_tree[pick] = true;
        if !cut || best[pick] > 0.0 {
            parents[pick] = from[pick];
        }
        for k in 0..n {
            if !in_tree[k] && w[pick * n + k] > best[k] {
                (best[k], from[k]) = (w[pick * n + k], pick);
            }
        }
    }
    parents
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::{RngExt, SeedableRng};

    const CATS: [u8; 5] = [3, 4, 2, 5, 3];
    const FIXED: TanOptions = TanOptions { m: DEFAULT_M, relative: false, games: false };
    const RELATIVE: TanOptions = TanOptions { m: 0.07, relative: true, games: false };
    const GAMES: TanOptions = TanOptions { m: DEFAULT_M, relative: false, games: true };

    /// Training-time prediction from the counts equals `predict` right after
    /// a refresh, fixed or relative m, with or without cached probabilities;
    /// and the text and state formats a delegation tree writes read back.
    #[test]
    fn predict_counts_text_and_state_for_trees() {
        let mut rng = StdRng::seed_from_u64(5);
        for opts in [FIXED, RELATIVE] {
            for use_probs in [false, true] {
                let mut tan = Tan::new(3, &CATS, 1.0, opts, use_probs);
                for (x, label) in data(5, 3000) {
                    tan.train(&x, &label);
                }
                tan.refresh();
                let mut x = [0u8; 5];
                for _ in 0..200 {
                    for (i, v) in x.iter_mut().enumerate() {
                        *v = rng.random_range(0..CATS[i]);
                    }
                    let (mut a, mut b) = ([0.0; 3], [0.0; 3]);
                    tan.predict(&x, &mut a);
                    tan.predict_counts(&x, &mut b);
                    for c in 0..3 {
                        assert!((a[c] - b[c]).abs() < 1e-9, "{a:?} {b:?}");
                    }
                }
                // Text (what predicting needs) and state (everything).
                let mut s = String::new();
                tan.write_text(&mut s);
                let mut t = crate::textfmt::Tokens::new(&s);
                let back = Tan::from_tokens(&mut t, 3, &CATS, 1.0, use_probs).unwrap();
                let mut w = Writer::new("test");
                tan.write_state_to(&mut w);
                let bytes = w.into_bytes();
                let mut r = Reader::new(&bytes, "test").unwrap();
                let state = Tan::read_state_from(&mut r).unwrap();
                r.finish().unwrap();
                for _ in 0..50 {
                    for (i, v) in x.iter_mut().enumerate() {
                        *v = rng.random_range(0..CATS[i]);
                    }
                    let (mut a, mut b, mut c) = ([0.0; 3], [0.0; 3], [0.0; 3]);
                    tan.predict(&x, &mut a);
                    back.predict(&x, &mut b);
                    state.predict(&x, &mut c);
                    for k in 0..3 {
                        assert!((a[k] - b[k]).abs() < 1e-6, "{a:?} {b:?}");
                        assert_eq!(a[k].to_bits(), c[k].to_bits());
                    }
                }
                let mut w2 = Writer::new("test");
                state.write_state_to(&mut w2);
                assert!(w2.into_bytes() == bytes);
            }
        }
    }


    /// Random positions and labels: one-hot mostly, some partial. Each
    /// feature copies the one before it (mod its values) 70% of the time, so
    /// that every edge of that chain is plain to see.
    fn data(seed: u64, len: usize) -> Vec<(Vec<u8>, Vec<f64>)> {
        let mut rng = StdRng::seed_from_u64(seed);
        (0..len)
            .map(|k| {
                let mut x = vec![0u8; CATS.len()];
                for i in 0..CATS.len() {
                    x[i] = if i > 0 && rng.random_range(0..10) < 7 {
                        x[i - 1] % CATS[i]
                    } else {
                        rng.random_range(0..CATS[i])
                    };
                }
                let label = if k % 7 == 0 {
                    vec![0.25, 0.5, 0.25]
                } else {
                    let mut l = vec![0.0; 3];
                    l[rng.random_range(0..3usize)] = 1.0;
                    l
                };
                (x, label)
            })
            .collect()
    }

    fn trained(use_probs: bool, len: usize) -> Tan {
        trained_with(FIXED, use_probs, len)
    }

    fn trained_with(opts: TanOptions, use_probs: bool, len: usize) -> Tan {
        let mut tan = Tan::new(3, &CATS, 1.0, opts, use_probs);
        for (x, l) in data(1, len) {
            tan.train(&x, &l);
        }
        tan.refresh();
        tan
    }

    fn bits(xs: &[f64]) -> Vec<u64> {
        xs.iter().map(|x| x.to_bits()).collect()
    }

    fn temp(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("chessai-tan-{}-{name}", std::process::id()))
    }

    #[test]
    fn without_a_tree_it_is_cnbp_exactly() {
        for (use_probs, opts) in [false, true].into_iter().flat_map(|u| [FIXED, RELATIVE, GAMES].map(|o| (u, o))) {
            let mut cnbp = Cnbp::new(3, &CATS, 1.0, use_probs);
            let mut tan = Tan::new(3, &CATS, 1.0, opts, use_probs);
            // An untrained TAN already predicts as an untrained CNBP.
            let mut fresh_cnbp = Cnbp::new(3, &CATS, 1.0, use_probs);
            fresh_cnbp.refresh();
            for (x, l) in data(2, 400) {
                cnbp.train(&x, &l);
                tan.train(&x, &l);
            }
            cnbp.refresh();
            let tan = tan.without_tree();
            for (x, _) in data(3, 200) {
                let (mut a, mut b, mut c, mut d) = ([0.0; 3], [0.0; 3], [0.0; 3], [0.0; 3]);
                cnbp.predict(&x, &mut a);
                tan.predict(&x, &mut b);
                cnbp.predict_log(&x, &mut c);
                tan.predict_log(&x, &mut d);
                assert_eq!(bits(&a), bits(&b), "use_probs {use_probs}, {opts:?}");
                assert_eq!(bits(&c), bits(&d), "use_probs {use_probs}, {opts:?}");
            }
            let untrained = Tan::new(3, &CATS, 1.0, opts, use_probs);
            let (mut a, mut b) = ([0.0; 3], [0.0; 3]);
            fresh_cnbp.predict(&[1, 2, 0, 4, 1], &mut a);
            untrained.predict(&[1, 2, 0, 4, 1], &mut b);
            assert_eq!(bits(&a), bits(&b));
        }
    }

    /// `predict` and `predict_log` with `use_probs` one class at a time,
    /// through `factor` and `log_factor`.
    fn predict_by_class(tan: &Tan, x: &[u8]) -> ([f64; 3], [f64; 3]) {
        let (mut p, mut l, mut total) = ([0.0; 3], [0.0; 3], 0.0);
        for cl in 0..3 {
            (p[cl], l[cl]) = (tan.nb.class_prob(cl), tan.nb.class_log(cl));
            for (i, &a) in x.iter().enumerate() {
                p[cl] *= tan.factor(cl, i, a, x);
                l[cl] += tan.log_factor(cl, i, a, x);
            }
            total += p[cl];
        }
        p.iter_mut().for_each(|q| *q /= total);
        (p, l)
    }

    #[test]
    fn pair_counts_are_the_data_s_counts() {
        let examples = data(8, 500);
        let mut tan = Tan::new(3, &CATS, 1.0, FIXED, true);
        for (x, l) in &examples {
            tan.train(x, l);
        }
        let file = tan.convert_pairs(&tan.pairs, true);
        assert_eq!(bits(&tan.convert_pairs(&file, false)), bits(&tan.pairs));
        let n = CATS.len();
        for cl in 0..3 {
            let k = cl * tan.pair_len;
            for i in 0..n {
                for j in i + 1..n {
                    for a in 0..CATS[i] as usize {
                        for b in 0..CATS[j] as usize {
                            let mut count = 0.0;
                            for (x, l) in &examples {
                                if (x[i] as usize, x[j] as usize) == (a, b) && l[cl] != 0.0 {
                                    count += l[cl];
                                }
                            }
                            assert_eq!(tan.pairs[k + tan.cell(i, j, a, b)].to_bits(), count.to_bits());
                            let f = k + tan.pair_off[i * n + j] + a * CATS[j] as usize + b;
                            assert_eq!(file[f].to_bits(), count.to_bits());
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn predicting_all_classes_at_once_is_exact() {
        let tan = trained(true, 3000);
        assert!(tan.parents().iter().filter(|p| p.is_some()).count() == CATS.len() - 1);
        let path = temp("at-once");
        tan.save(&path).unwrap();
        let loaded = Tan::load(&path, true).unwrap();
        std::fs::remove_file(&path).unwrap();
        for model in [&tan, &loaded] {
            for (x, _) in data(6, 300) {
                let (mut a, mut b) = ([0.0; 3], [0.0; 3]);
                model.predict(&x, &mut a);
                model.predict_log(&x, &mut b);
                let (p, l) = predict_by_class(model, &x);
                assert_eq!(bits(&a), bits(&p));
                assert_eq!(bits(&b), bits(&l));
            }
        }
    }

    #[test]
    fn conditionals_and_predictions_are_distributions() {
        let tan = trained(true, 2000);
        assert!(tan.parents().iter().filter(|p| p.is_none()).count() == 1, "one root");
        for cl in 0..3 {
            for i in 0..CATS.len() {
                let Some(p) = tan.parent(i) else { continue };
                for b in 0..tan.cats[p] {
                    let mut x = vec![0; CATS.len()];
                    x[p] = b as u8;
                    let sum: f64 = (0..CATS[i]).map(|a| tan.factor(cl, i, a, &x)).sum();
                    assert!((sum - 1.0).abs() < 1e-12, "feature {i} given {p}={b}, class {cl}: {sum}");
                }
            }
        }
        let counts = trained(false, 2000);
        for (x, _) in data(4, 100) {
            let (mut a, mut b) = ([0.0; 3], [0.0; 3]);
            tan.predict(&x, &mut a);
            counts.predict(&x, &mut b);
            assert!((a.iter().sum::<f64>() - 1.0).abs() < 1e-12);
            assert!(a.iter().all(|&p| p > 0.0 && p < 1.0));
            // With and without use_probs: the same model.
            assert!(a.iter().zip(&b).all(|(p, q)| (p - q).abs() < 1e-12));
        }
    }

    #[test]
    fn recovers_a_planted_dependency() {
        // Feature 0 tells the class best; feature 2 tells it less, and
        // feature 4 copies feature 2 95% of the time, which naive Bayes
        // counts as independent evidence; 1 and 3 are noise.
        let mut rng = StdRng::seed_from_u64(5);
        let mut tan = Tan::new(3, &CATS, 1.0, FIXED, true);
        let mut cnbp = Cnbp::new(3, &CATS, 1.0, true);
        let mut examples = Vec::new();
        for _ in 0..20000 {
            let c = rng.random_range(0..2usize);
            let x0 = if rng.random_range(0..10) < 9 { c as u8 } else { rng.random_range(0..3) };
            let x2 = if rng.random_range(0..10) < 7 { c as u8 } else { rng.random_range(0..2) };
            let x4 = if rng.random_range(0..20) < 19 { x2 } else { rng.random_range(0..3) };
            let x = vec![x0, rng.random_range(0..4), x2, rng.random_range(0..5), x4];
            let mut l = vec![0.0; 3];
            l[c] = 1.0;
            examples.push((x, l));
        }
        for (x, l) in &examples {
            tan.train(x, l);
            cnbp.train(x, l);
        }
        tan.refresh();
        cnbp.refresh();
        let parents = tan.parents();
        assert_eq!(parents[0], None, "the root is the feature that tells the class");
        assert!(parents[4] == Some(2) || parents[2] == Some(4), "{parents:?}");
        // It predicts the training data better than naive Bayes.
        let loss = |m: &dyn Fn(&[u8], &mut [f64])| -> f64 {
            examples
                .iter()
                .map(|(x, l)| {
                    let mut p = [0.0; 3];
                    m(x, &mut p);
                    -(p[l.iter().position(|&v| v == 1.0).unwrap()]).ln()
                })
                .sum()
        };
        assert!(loss(&|x, p| tan.predict(x, p)) < loss(&|x, p| cnbp.predict(x, p)));
    }

    #[test]
    fn the_tree_is_a_function_of_the_counts() {
        let chain = trained(true, 3000).parents();
        assert_eq!(chain, trained(true, 3000).parents());
        assert_eq!(chain, trained(false, 3000).parents());
        assert_eq!(chain.iter().filter(|p| p.is_none()).count(), 1, "the chain, one tree: {chain:?}");
        // No data: no parents, whatever the options (a tree's TAN router
        // predicts as CNBP until its first refresh with counts).
        for opts in [FIXED, RELATIVE, GAMES] {
            assert_eq!(Tan::new(3, &CATS, 1.0, opts, true).parents(), vec![None; CATS.len()]);
        }
        // A path: 0-1 heaviest, then 1-2, then 2-3.
        let mut w = vec![0.0; 16];
        for (i, j, v) in [(0, 1, 3.0), (1, 2, 2.0), (2, 3, 1.0), (0, 3, 0.5)] {
            (w[i * 4 + j], w[j * 4 + i]) = (v, v);
        }
        for cut in [false, true] {
            assert_eq!(prim(4, 0, &w, cut), vec![NONE, 0, 1, 2]);
            assert_eq!(prim(4, 3, &w, cut), vec![1, 2, 3, NONE]);
        }
        // Without the edge 1-2, 0-3 joins 2 and 3 to 0's tree.
        (w[6], w[9]) = (0.0, 0.0);
        for cut in [false, true] {
            assert_eq!(prim(4, 0, &w, cut), vec![NONE, 0, 3, 0]);
        }
        // Without 0-3 too: a tree through an edge of weight 0, or cut there,
        // a forest: 2 roots a tree of its own, which 3 joins.
        (w[3], w[12]) = (0.0, 0.0);
        assert_eq!(prim(4, 0, &w, false), vec![NONE, 0, 0, 2]);
        assert_eq!(prim(4, 0, &w, true), vec![NONE, 0, NONE, 2]);
    }

    #[test]
    fn games_count_once() {
        // Two games, of 2 positions and of 3, each the same position over
        // and over: each game adds 1 to its cell of every pair, and CNBP's
        // counts count positions.
        let (x, y) = ([1u8, 2, 0, 4, 1], [0u8, 3, 1, 2, 2]);
        let games: Vec<Vec<[u8; 5]>> = vec![vec![x, x], vec![y, y, y]];
        let (win, draw) = ([1.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        let mut tan = Tan::new(3, &CATS, 1.0, GAMES, true);
        let mut cnbp = Cnbp::new(3, &CATS, 1.0, true);
        tan.observe_games(&games.iter().map(Vec::as_slice).collect::<Vec<_>>());
        // In an order that mixes the games, as training does.
        for (pos, label, game) in [(y, draw, 1), (x, win, 0), (y, draw, 1), (y, draw, 1), (x, win, 0)] {
            tan.train_in_game(&pos, &label, 7 << 32 | game);
            cnbp.train(&pos, &label);
        }
        assert_eq!(tan.nb.class_counts(), &[2.0, 0.0, 3.0]);
        for (pos, cl) in [(x, 0), (y, 2)] {
            for i in 0..CATS.len() {
                for j in i + 1..CATS.len() {
                    let count = tan.pairs[cl * tan.pair_len + tan.cell(i, j, pos[i] as usize, pos[j] as usize)];
                    assert!((count - 1.0).abs() < 1e-12, "{count}");
                }
            }
        }
        let total: f64 = tan.pairs.iter().sum();
        assert!((total - 2.0 * 10.0).abs() < 1e-12, "two games in each of the 10 pairs: {total}");
        // Without a tree: CNBP's predictions, bit for bit.
        let (tan, mut c) = (tan.without_tree(), cnbp);
        c.refresh();
        let (mut a, mut b) = ([0.0; 3], [0.0; 3]);
        tan.predict(&x, &mut a);
        c.predict(&x, &mut b);
        assert_eq!(bits(&a), bits(&b));
    }

    #[test]
    #[should_panic(expected = "observed")]
    fn games_must_be_observed() {
        let mut tan = Tan::new(3, &CATS, 1.0, GAMES, true);
        tan.train_in_game(&[0, 0, 0, 0, 0], &[1.0, 0.0, 0.0], 3);
    }

    #[test]
    fn insignificant_edges_are_cut() {
        // Independent features: what dependence the counts show is too
        // little to be significant; what's left is naive Bayes.
        let mut rng = StdRng::seed_from_u64(9);
        let mut tan = Tan::new(3, &CATS, 1.0, GAMES, true);
        let mut cnbp = Cnbp::new(3, &CATS, 1.0, true);
        let examples: Vec<(Vec<u8>, Vec<f64>)> = (0..300)
            .map(|_| {
                let mut l = vec![0.0; 3];
                l[rng.random_range(0..3usize)] = 1.0;
                (CATS.iter().map(|&c| rng.random_range(0..c)).collect(), l)
            })
            .collect();
        for (x, l) in &examples {
            tan.train(x, l);
            cnbp.train(x, l);
        }
        tan.refresh();
        cnbp.refresh();
        assert_eq!(tan.parents(), vec![None; CATS.len()]);
        for (x, _) in &examples {
            let (mut a, mut b) = ([0.0; 3], [0.0; 3]);
            tan.predict(x, &mut a);
            cnbp.predict(x, &mut b);
            assert_eq!(bits(&a), bits(&b));
        }
        // The same 300 positions as 3 games of 100: still fewer games.
        let games: Vec<&[(Vec<u8>, Vec<f64>)]> = examples.chunks(100).collect();
        let mut by_game = Tan::new(3, &CATS, 1.0, GAMES, true);
        by_game.observe_games(&games);
        for (g, game) in games.iter().enumerate() {
            for (x, l) in game.iter() {
                by_game.train_in_game(x, l, g as u64);
            }
        }
        by_game.refresh();
        assert_eq!(by_game.parents(), vec![None; CATS.len()]);
    }

    #[test]
    fn model_file_predicts_the_same() {
        for use_probs in [false, true] {
            let tan = trained(use_probs, 1500);
            let path = temp(&format!("model-{use_probs}"));
            tan.save(&path).unwrap();
            let back = Tan::load(&path, use_probs).unwrap();
            std::fs::remove_file(&path).unwrap();
            assert_eq!(tan.parents(), back.parents());
            for (x, _) in data(6, 100) {
                let (mut a, mut b, mut c, mut d) = ([0.0; 3], [0.0; 3], [0.0; 3], [0.0; 3]);
                tan.predict(&x, &mut a);
                back.predict(&x, &mut b);
                tan.predict_log(&x, &mut c);
                back.predict_log(&x, &mut d);
                assert_eq!(bits(&a), bits(&b));
                assert_eq!(bits(&c), bits(&d));
            }
        }
    }

    #[test]
    #[should_panic(expected = "can't train")]
    fn model_file_cannot_train() {
        let path = temp("cannot-train");
        trained(true, 100).save(&path).unwrap();
        let mut back = Tan::load(&path, true).unwrap();
        std::fs::remove_file(&path).unwrap();
        back.train(&[0, 0, 0, 0, 0], &[1.0, 0.0, 0.0]);
    }

    #[test]
    fn state_resumes_exactly() {
        for opts in [FIXED, RELATIVE] {
            let state = temp("resume.state");
            let (mut a, more) = (trained_with(opts, true, 1000), data(7, 800));
            a.save_state(&state).unwrap();
            let mut b = Tan::load_state(&state, 1.0, opts, true).unwrap();
            // Also mid-round, trained since the last refresh.
            a.train(&more[0].0, &more[0].1);
            a.save_state(&state).unwrap();
            let mut c = Tan::load_state(&state, 1.0, opts, true).unwrap();
            b.train(&more[0].0, &more[0].1);
            for (x, l) in &more[1..] {
                for m in [&mut a, &mut b, &mut c] {
                    m.train(x, l);
                }
            }
            let saved: Vec<Vec<u8>> = [&mut a, &mut b, &mut c]
                .into_iter()
                .map(|m| {
                    m.refresh();
                    m.save_state(&state).unwrap();
                    std::fs::read(&state).unwrap()
                })
                .collect();
            assert_eq!(saved[0], saved[1]);
            assert_eq!(saved[0], saved[2]);
            for other in [TanOptions::fixed(4.0), TanOptions { m: 4.0, ..opts }, FIXED, RELATIVE, GAMES] {
                if other != opts {
                    assert!(Tan::load_state(&state, 1.0, other, true).is_err(), "{opts:?} as {other:?}");
                }
            }
            assert!(Tan::load_state(&state, 1.0, opts, false).is_err(), "other options");
            // A model file's start: the state beside it.
            let model = state.with_extension("model");
            a.save(&model).unwrap();
            let d = Tan::load_start(&model, 1.0, opts, true).unwrap();
            assert_eq!(d.parents(), a.parents());
            std::fs::remove_file(&model).unwrap();
            std::fs::remove_file(&state).unwrap();
            assert!(Tan::load_start(&model, 1.0, opts, true).is_err(), "no state beside it");
        }
    }

    #[test]
    fn relative_m_is_lambda_times_the_positions() {
        // λ N is the m of the tables, N the positions so far: the same model
        // as with that m fixed, file and all.
        let (lambda, path) = (0.07, temp("relative"));
        let mut rel = trained_with(TanOptions { m: lambda, ..RELATIVE }, true, 1500);
        for len in [1500, 2700] {
            if len > 1500 {
                for (x, l) in data(9, len - 1500) {
                    rel.train(&x, &l);
                }
                rel.refresh();
            }
            let n: f64 = rel.nb.class_counts().iter().sum();
            assert!((n - len as f64).abs() < 1e-9, "{n}");
            assert_eq!(rel.m().to_bits(), (lambda * n).to_bits());
            let mut fixed = trained_with(TanOptions::fixed(lambda * n), true, 1500);
            if len > 1500 {
                for (x, l) in data(9, len - 1500) {
                    fixed.train(&x, &l);
                }
                fixed.refresh();
            }
            assert_eq!(rel.parents(), fixed.parents());
            for (x, _) in data(6, 200) {
                let (mut a, mut b, mut c, mut d) = ([0.0; 3], [0.0; 3], [0.0; 3], [0.0; 3]);
                rel.predict(&x, &mut a);
                fixed.predict(&x, &mut b);
                rel.predict_log(&x, &mut c);
                fixed.predict_log(&x, &mut d);
                assert_eq!(bits(&a), bits(&b));
                assert_eq!(bits(&c), bits(&d));
            }
            rel.save(&path).unwrap();
            let file = std::fs::read(&path).unwrap();
            fixed.save(&path).unwrap();
            assert_eq!(file, std::fs::read(&path).unwrap());
        }
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn forgetting_keeps_the_tree() {
        let mut tan = trained(true, 2000);
        let parents = tan.parents();
        tan.forget(4.0);
        assert_eq!(tan.parents(), parents);
    }
}
