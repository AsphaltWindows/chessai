//! S7: data-chosen pairwise features for the flat probit (analyst 6.97, 2026-11-12; the user approved it ~21:50:
//! "Let's try pairwise features also"). chessai-pairs1.
//!
//! A pair is the conjunction of two active indicator values: feature i has value a AND feature j has value b (i < j).
//! A chosen pair is one more binary input of the probit, with its own Gaussian weight N(mu, s2): it enters the
//! latent's mean and variance when both values are present, the dynamics (tau) widen it, and the truncation moves it,
//! exactly as the model's own weights (see `probit`).
//!
//! Which pairs. Every pair-value is a candidate (356,642 over the 70 features; a position has 2,415 active). After
//! each example the probit knows the standardized shift v of its latent's mean (`Truncation::tempered`'s v, summed
//! over the label's results). Every active candidate accumulates S += v and n += 1: the first-order score of a new
//! weight at zero. Every `every` examples (a checkpoint) the candidates with |z| = |S| / sqrt(n) above `z` (5: the
//! Bonferroni bar for ~400k candidates) are ranked by |z| (ties by index) and the first `k` (50), at most `cap` (2,000)
//! chosen pairs in all, become pairs with mu 0 and sd `shrink` (0.5) x the probit's prior; a new pair's statistics
//! restart. At the same checkpoint, first, a pair with at least `drop_after` (3M) examples of its own since it was
//! chosen whose |mu| is below `drop` (0.5) x its sd is dropped, and its statistics restart. Nothing names a pair:
//! the data choose (zero knowledge). With `sample` M above 1, only the candidates whose index hashes to 0 mod M are
//! tracked (a fixed, knowledge-free subset, for speed).
//!
//! Files. The model file holds the options, the examples trained and the chosen pairs with their weights (probit
//! model version 2; a model without pairs stays version 1, byte for byte). The candidates' statistics are training
//! state: `save_state` writes them beside the model (`probitN.state`, ~5.7 MB), and training on from a model file
//! reads the state beside it, so a run resumes exactly. Play needs only the model file.
//!
//! Order. A position's chosen pairs are summed after the model's own weights, by their first feature i ascending,
//! then in the order they were chosen: `add_active` (the latent) and `for_active` (training) visit them the same way.

use std::fmt::Write as _;
use std::io;
use std::path::Path;

use crate::state::{Reader, Writer};
use crate::textfmt::{Tokens, invalid};

/// The options of the pairs (--pairs-*); see the module documentation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PairOptions {
    pub k: usize,
    pub z: f64,
    pub cap: usize,
    pub shrink: f64,
    pub drop: f64,
    pub every: u64,
    pub drop_after: u64,
    pub sample: u32,
}

impl Default for PairOptions {
    fn default() -> Self {
        PairOptions { k: 50, z: 5.0, cap: 2000, shrink: 0.5, drop: 0.5, every: 1_000_000, drop_after: 3_000_000, sample: 1 }
    }
}

impl PairOptions {
    pub fn check(&self) -> Result<(), String> {
        if !(self.z.is_finite() && self.z >= 0.0) {
            return Err(format!("--pairs-z must be a number from 0, not {}", self.z));
        }
        if !(self.shrink.is_finite() && self.shrink > 0.0) {
            return Err(format!("--pairs-shrink must be above 0, not {}", self.shrink));
        }
        if !(self.drop.is_finite() && self.drop >= 0.0) {
            return Err(format!("--pairs-drop must be a number from 0, not {}", self.drop));
        }
        if self.every == 0 {
            return Err("--pairs-every must be at least 1".into());
        }
        if self.sample == 0 {
            return Err("--pairs-sample must be at least 1".into());
        }
        Ok(())
    }
}

/// The current round's counts, for `report`.
#[derive(Clone, Debug, Default)]
struct Round {
    examples: u64,
    active: u64,
    checkpoints: u64,
    added: Vec<f64>,
    dropped: u64,
    /// One line per checkpoint: the pairs dropped (mean and sd) and added (|z|), as i:a&j:b.
    lines: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Pairs {
    pub opts: PairOptions,
    cats: Vec<u8>,
    /// Each feature's first weight index in the probit (after the bias at 0), as `Probit::starts`.
    starts: Vec<usize>,
    /// For features i < j: the index of candidate (i, 0, j, 0); candidate (i, a, j, b) is base + a cats[j] + b.
    base: Vec<u32>,
    /// The blocks (i, j) by their first candidate, ascending, to decode a candidate.
    blocks: Vec<(u32, u8, u8)>,
    candidates: usize,
    /// The chosen pairs [i, a, j, b] in the order chosen, and their weights [mu, s2].
    list: Vec<[u8; 4]>,
    weights: Vec<[f64; 2]>,
    /// Per weight index of a value (starts[i] + a): the chosen pairs whose first value it is, (j, b, slot).
    index: Vec<Vec<(u8, u8, u32)>>,
    /// Examples trained with pairs on, from the start.
    trained: u64,
    /// Training only: (S, n) per candidate; None in a model loaded to play.
    stats: Option<Vec<[f64; 2]>>,
    round: Round,
}

fn hash_keep(c: usize, sample: u32) -> bool {
    sample <= 1 || ((c as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 33) % sample as u64 == 0
}

impl Pairs {
    /// No pairs yet, training from scratch.
    pub fn new(categories: &[u8], opts: PairOptions) -> Self {
        let mut p = Self::shape(categories, opts);
        p.stats = Some(vec![[0.0, 0.0]; p.candidates]);
        p
    }

    fn shape(categories: &[u8], opts: PairOptions) -> Self {
        let f = categories.len();
        let mut starts = Vec::with_capacity(f);
        let mut width = 1;
        for &c in categories {
            starts.push(width);
            width += c as usize;
        }
        let mut base = vec![u32::MAX; f * f];
        let mut blocks = Vec::new();
        let mut next = 0usize;
        for i in 0..f {
            for j in i + 1..f {
                base[i * f + j] = next as u32;
                blocks.push((next as u32, i as u8, j as u8));
                next += categories[i] as usize * categories[j] as usize;
            }
        }
        Pairs {
            opts,
            cats: categories.to_vec(),
            starts,
            base,
            blocks,
            candidates: next,
            list: Vec::new(),
            weights: Vec::new(),
            index: vec![Vec::new(); width],
            trained: 0,
            stats: None,
            round: Round::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn trained(&self) -> u64 {
        self.trained
    }

    /// The number of candidates (pair-values).
    pub fn candidates(&self) -> usize {
        self.candidates
    }

    /// The chosen pairs and their weights.
    pub fn chosen(&self) -> impl Iterator<Item = ([u8; 4], [f64; 2])> + '_ {
        self.list.iter().copied().zip(self.weights.iter().copied())
    }

    fn candidate(&self, [i, a, j, b]: [u8; 4]) -> usize {
        let f = self.cats.len();
        self.base[i as usize * f + j as usize] as usize + a as usize * self.cats[j as usize] as usize + b as usize
    }

    fn decode(&self, c: usize) -> [u8; 4] {
        let k = self.blocks.partition_point(|&(b, _, _)| b as usize <= c) - 1;
        let (b, i, j) = self.blocks[k];
        let off = c - b as usize;
        let cj = self.cats[j as usize] as usize;
        [i, (off / cj) as u8, j, (off % cj) as u8]
    }

    fn rebuild_index(&mut self) {
        self.index.iter_mut().for_each(Vec::clear);
        for (slot, &[i, a, j, b]) in self.list.iter().enumerate() {
            self.index[self.starts[i as usize] + a as usize].push((j, b, slot as u32));
        }
    }

    /// Adds the position's chosen pairs' means and variances to `m` and `var`, one by one.
    #[inline]
    pub fn add_active(&self, data: &[u8], m: &mut f64, var: &mut f64) {
        if self.list.is_empty() {
            return;
        }
        for (i, &v) in data.iter().enumerate() {
            for &(j, b, slot) in &self.index[self.starts[i] + v as usize] {
                if data[j as usize] == b {
                    let [mu, s2] = self.weights[slot as usize];
                    *m += mu;
                    *var += s2;
                }
            }
        }
    }

    /// Calls `f` on each of the position's chosen pairs' weights, in `add_active`'s order.
    #[inline]
    pub fn for_active(&mut self, data: &[u8], mut f: impl FnMut(&mut [f64; 2])) {
        if self.list.is_empty() {
            return;
        }
        for (i, &v) in data.iter().enumerate() {
            for &(j, b, slot) in &self.index[self.starts[i] + v as usize] {
                if data[j as usize] == b {
                    f(&mut self.weights[slot as usize]);
                }
            }
        }
    }

    fn count_active(&self, data: &[u8]) -> u64 {
        let mut n = 0;
        for (i, &v) in data.iter().enumerate() {
            for &(j, b, _) in &self.index[self.starts[i] + v as usize] {
                n += (data[j as usize] == b) as u64;
            }
        }
        n
    }

    /// Multiplies the pairs' variances by `factor` (`Probit::forget`).
    pub fn forget(&mut self, factor: f64) {
        self.weights.iter_mut().for_each(|w| w[1] *= factor);
    }

    /// After an example whose latent's standardized mean shift was `v`: the candidates' statistics, the count, and a
    /// checkpoint every `every` examples (new pairs get sd shrink x `prior`).
    pub fn observe(&mut self, data: &[u8], v: f64, prior: f64) {
        let Some(stats) = self.stats.as_mut() else {
            panic!("pairs: training a model loaded without its training state");
        };
        let f = self.cats.len();
        if self.opts.sample <= 1 {
            for i in 0..f {
                let ai = data[i] as usize;
                let row = &self.base[i * f..i * f + f];
                for j in i + 1..f {
                    let c = row[j] as usize + ai * self.cats[j] as usize + data[j] as usize;
                    let s = &mut stats[c];
                    s[0] += v;
                    s[1] += 1.0;
                }
            }
        } else {
            for i in 0..f {
                let ai = data[i] as usize;
                let row = &self.base[i * f..i * f + f];
                for j in i + 1..f {
                    let c = row[j] as usize + ai * self.cats[j] as usize + data[j] as usize;
                    if hash_keep(c, self.opts.sample) {
                        let s = &mut stats[c];
                        s[0] += v;
                        s[1] += 1.0;
                    }
                }
            }
        }
        self.round.examples += 1;
        self.round.active += self.count_active(data);
        self.trained += 1;
        if self.trained % self.opts.every == 0 {
            self.checkpoint(prior);
        }
    }

    fn checkpoint(&mut self, prior: f64) {
        self.round.checkpoints += 1;
        let o = self.opts;
        let cands: Vec<usize> = self.list.iter().map(|&p| self.candidate(p)).collect();
        let stats = self.stats.as_mut().expect("training state");
        let name = |[i, a, j, b]: [u8; 4]| format!("{i}:{a}&{j}:{b}");
        let mut line = format!("pairs checkpoint at {} examples:", self.trained);
        // Drops first: a pair with drop_after examples of its own whose mean is within drop sd of 0.
        let mut keep = Vec::with_capacity(self.list.len());
        let mut dropped = Vec::new();
        for (slot, &c) in cands.iter().enumerate() {
            let [mu, s2] = self.weights[slot];
            if stats[c][1] >= o.drop_after as f64 && mu.abs() < o.drop * s2.sqrt() {
                stats[c] = [0.0, 0.0];
                self.round.dropped += 1;
                dropped.push(format!("{} mu {mu:.4} sd {:.4}", name(self.list[slot]), s2.sqrt()));
            } else {
                keep.push(slot);
            }
        }
        write!(line, " dropped {}{}{};", dropped.len(), if dropped.is_empty() { "" } else { ": " }, dropped.join(", "))
            .unwrap();
        if keep.len() < self.list.len() {
            self.list = keep.iter().map(|&s| self.list[s]).collect();
            self.weights = keep.iter().map(|&s| self.weights[s]).collect();
        }
        // Then adds: the candidates above the bar, by |z| (ties by index), up to k and the cap.
        let want = o.k.min(o.cap.saturating_sub(self.list.len()));
        if want > 0 {
            let mut chosen = vec![false; self.candidates];
            for &p in &self.list {
                chosen[self.candidate(p)] = true;
            }
            let stats = self.stats.as_ref().expect("training state");
            let mut top: Vec<(f64, usize)> = Vec::new();
            for (c, &[s, n]) in stats.iter().enumerate() {
                if n > 0.0 && !chosen[c] {
                    let z = (s / n.sqrt()).abs();
                    if z > o.z {
                        top.push((z, c));
                    }
                }
            }
            top.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
            top.truncate(want);
            let s2 = (o.shrink * prior) * (o.shrink * prior);
            let mut added = Vec::with_capacity(top.len());
            for &(z, c) in &top {
                let p = self.decode(c);
                self.list.push(p);
                self.weights.push([0.0, s2]);
                self.round.added.push(z);
                added.push(format!("{} {z:.2}", name(p)));
            }
            let stats = self.stats.as_mut().expect("training state");
            for &(_, c) in &top {
                stats[c] = [0.0, 0.0];
            }
            write!(line, " added {}{}{}", added.len(), if added.is_empty() { "" } else { " (|z|): " }, added.join(", "))
                .unwrap();
        } else {
            write!(line, " added 0 (at the cap)").unwrap();
        }
        write!(line, "; {} chosen", self.list.len()).unwrap();
        self.round.lines.push(line);
        self.rebuild_index();
    }

    /// The round's lines for the train log (then the round's counts restart): first pairs added (their |z|) and
    /// dropped, the chosen pairs, and the mean number of chosen pairs active per position; then one line per
    /// checkpoint, naming the pairs dropped and added (feature i value a & feature j value b as i:a&j:b, by the
    /// encoding's feature order) with their |z|.
    pub fn report(&mut self) -> String {
        let r = std::mem::take(&mut self.round);
        let mut s = format!(
            "pairs: {} chosen of cap {}; this round: {} examples, {} checkpoint{}, added {}",
            self.list.len(),
            self.opts.cap,
            r.examples,
            r.checkpoints,
            if r.checkpoints == 1 { "" } else { "s" },
            r.added.len()
        );
        if !r.added.is_empty() {
            let mut z = r.added.clone();
            z.sort_by(f64::total_cmp);
            let q = |p: f64| z[((z.len() - 1) as f64 * p).round() as usize];
            write!(s, " (|z| min {:.2}, median {:.2}, max {:.2})", z[0], q(0.5), z[z.len() - 1]).unwrap();
        }
        write!(
            s,
            ", dropped {}; active per position {:.3}; {} examples since the start",
            r.dropped,
            if r.examples > 0 { r.active as f64 / r.examples as f64 } else { 0.0 },
            self.trained
        )
        .unwrap();
        for line in &r.lines {
            write!(s, "\n{line}").unwrap();
        }
        s
    }

    // ------------------------------------------------------------ files

    /// The model file's pairs section (after the probit's weights).
    pub fn write_text(&self, s: &mut String) {
        let o = &self.opts;
        writeln!(s, "pairs\n{} {:e} {} {:e} {:e} {} {} {}", o.k, o.z, o.cap, o.shrink, o.drop, o.every, o.drop_after, o.sample)
            .unwrap();
        writeln!(s, "{}\n{}", self.trained, self.list.len()).unwrap();
        for (&[i, a, j, b], &[mu, s2]) in self.list.iter().zip(&self.weights) {
            writeln!(s, "{i} {a} {j} {b} {mu:e} {s2:e}").unwrap();
        }
    }

    /// Reads `write_text`'s section; the model plays (no training state until `load_state`).
    pub fn read_text(tokens: &mut Tokens, categories: &[u8]) -> io::Result<Self> {
        if tokens.next::<String>()? != "pairs" {
            return Err(invalid("probit model version 2 without its pairs section"));
        }
        let opts = PairOptions {
            k: tokens.next()?,
            z: tokens.next()?,
            cap: tokens.next()?,
            shrink: tokens.next()?,
            drop: tokens.next()?,
            every: tokens.next()?,
            drop_after: tokens.next()?,
            sample: tokens.next()?,
        };
        opts.check().map_err(invalid)?;
        let mut p = Self::shape(categories, opts);
        p.trained = tokens.next()?;
        let n: usize = tokens.next()?;
        for _ in 0..n {
            let q: [u8; 4] = [tokens.next()?, tokens.next()?, tokens.next()?, tokens.next()?];
            let [i, a, j, b] = q;
            let f = categories.len();
            if !((i as usize) < f && (j as usize) < f && i < j && a < categories[i as usize] && b < categories[j as usize]) {
                return Err(invalid(format!("bad pair {q:?} in probit model file")));
            }
            p.list.push(q);
            p.weights.push([tokens.next()?, tokens.next()?]);
        }
        p.rebuild_index();
        Ok(p)
    }

    /// Writes the candidates' statistics (training state) to `path`.
    pub fn save_state(&self, path: &Path) -> io::Result<()> {
        let stats = self.stats.as_ref().ok_or_else(|| invalid("pairs: no training state to save"))?;
        let mut w = Writer::new("probit-pairs");
        w.u64(self.trained);
        w.u8s(&self.list.iter().flatten().copied().collect::<Vec<u8>>());
        w.f64s(&stats.iter().flatten().copied().collect::<Vec<f64>>());
        w.save(path)
    }

    /// Reads `save_state`'s file for the model it was saved with (the same examples trained and the same pairs).
    pub fn load_state(&mut self, path: &Path) -> io::Result<()> {
        let data = std::fs::read(path)?;
        let mut r = Reader::new(&data, "probit-pairs")?;
        let trained = r.u64()?;
        if trained != self.trained {
            return Err(invalid(format!(
                "{path:?}: a state after {trained} examples, but the model has trained {}",
                self.trained
            )));
        }
        let list = r.u8s()?;
        if list != self.list.iter().flatten().copied().collect::<Vec<u8>>() {
            return Err(invalid(format!("{path:?}: the state's pairs differ from the model's")));
        }
        let flat = r.f64s()?;
        if flat.len() != 2 * self.candidates {
            return Err(invalid(format!("{path:?}: a state of the wrong shape")));
        }
        r.finish()?;
        self.stats = Some(flat.chunks_exact(2).map(|x| [x[0], x[1]]).collect());
        Ok(())
    }

    /// Fresh statistics for a model that has trained nothing yet (a driver's untrained version 0 has no state).
    pub fn fresh_state(&mut self) -> io::Result<()> {
        if self.trained != 0 || !self.list.is_empty() {
            return Err(invalid("pairs: a trained model needs its state file to train on"));
        }
        self.stats = Some(vec![[0.0, 0.0]; self.candidates]);
        Ok(())
    }

    pub fn has_state(&self) -> bool {
        self.stats.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding::CATEGORIES;

    #[test]
    fn candidates_cover_every_pair_value_once() {
        let p = Pairs::new(&CATEGORIES, PairOptions::default());
        assert_eq!(p.candidates(), 356_642);
        // Every (i, a, j, b) maps to a distinct index that decodes back.
        let mut seen = vec![false; p.candidates()];
        let f = CATEGORIES.len();
        for i in 0..f {
            for j in i + 1..f {
                for a in 0..CATEGORIES[i] {
                    for b in 0..CATEGORIES[j] {
                        let q = [i as u8, a, j as u8, b];
                        let c = p.candidate(q);
                        assert!(!seen[c]);
                        seen[c] = true;
                        assert_eq!(p.decode(c), q);
                    }
                }
            }
        }
        assert!(seen.iter().all(|&s| s));
    }

    #[test]
    fn text_and_state_round_trip() {
        let mut p = Pairs::new(&CATEGORIES, PairOptions { every: 3, z: 0.5, k: 4, ..Default::default() });
        let data: Vec<u8> = (0..70).map(|i| (i * 7 % CATEGORIES[i] as usize) as u8).collect();
        for t in 0..7 {
            p.observe(&data, if t % 2 == 0 { 1.0 } else { 0.7 }, 0.3);
        }
        assert!(!p.is_empty());
        let mut s = String::new();
        p.write_text(&mut s);
        let mut q = Pairs::read_text(&mut Tokens::new(&s), &CATEGORIES).unwrap();
        assert_eq!(q.list, p.list);
        assert_eq!(q.weights, p.weights);
        assert_eq!(q.trained, 7);
        assert!(q.fresh_state().is_err());
        let dir = std::env::temp_dir().join(format!("pairs-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("probit1.state");
        p.save_state(&path).unwrap();
        q.load_state(&path).unwrap();
        assert_eq!(q.stats, p.stats);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn checkpoints_drop_then_add_by_z_up_to_k_and_the_cap() {
        let o = PairOptions { k: 3, z: 5.0, cap: 4, shrink: 0.5, drop: 0.5, every: 10, drop_after: 100, sample: 1 };
        let mut p = Pairs::new(&CATEGORIES, o);
        let stats = p.stats.as_mut().unwrap();
        // z = S / sqrt(n): 6, -7, 7 (a tie with -7: by index), 5 (not above the bar), 9.
        stats[10] = [60.0, 100.0];
        stats[20] = [-70.0, 100.0];
        stats[30] = [70.0, 100.0];
        stats[40] = [50.0, 100.0];
        stats[50] = [90.0, 100.0];
        p.checkpoint(0.3);
        let chosen: Vec<usize> = p.list.iter().map(|&q| p.candidate(q)).collect();
        assert_eq!(chosen, [50, 20, 30]);
        assert!(p.weights.iter().all(|&w| w == [0.0, 0.15 * 0.15]));
        // The chosen restart their statistics; the rest keep theirs.
        assert_eq!(p.stats.as_ref().unwrap()[50], [0.0, 0.0]);
        assert_eq!(p.stats.as_ref().unwrap()[10], [60.0, 100.0]);
        // The next checkpoint adds one: the cap is 4.
        p.checkpoint(0.3);
        let chosen: Vec<usize> = p.list.iter().map(|&q| p.candidate(q)).collect();
        assert_eq!(chosen, [50, 20, 30, 10]);
        // A pair is dropped once it has drop_after examples of its own and |mu| < drop sd; the others stay.
        p.weights[1][0] = 0.01; // candidate 20: |mu| 0.01 < 0.5 x 0.15
        p.weights[2][0] = 0.2; // candidate 30: kept, |mu| above
        p.stats.as_mut().unwrap()[20] = [3.0, 100.0];
        p.stats.as_mut().unwrap()[30] = [3.0, 100.0];
        p.stats.as_mut().unwrap()[50] = [3.0, 99.0]; // too few examples to judge
        p.stats.as_mut().unwrap()[60] = [80.0, 100.0];
        p.checkpoint(0.3);
        let chosen: Vec<usize> = p.list.iter().map(|&q| p.candidate(q)).collect();
        assert_eq!(chosen, [50, 30, 10, 60]);
        assert_eq!(p.stats.as_ref().unwrap()[20], [0.0, 0.0]);
        let r = p.report();
        assert!(r.contains("added 5 (|z| min 6.00, median 7.00, max 9.00), dropped 1"), "{r}");
        assert!(r.starts_with("pairs: 4 chosen of cap 4; this round: 0 examples, 3 checkpoints"), "{r}");
        let lines: Vec<&str> = r.lines().collect();
        assert_eq!(lines.len(), 4, "{r}");
        assert!(lines[1].starts_with("pairs checkpoint at 0 examples: dropped 0; added 3 (|z|): "), "{r}");
        assert!(lines[3].contains(" dropped 1: ") && lines[3].ends_with("; 4 chosen"), "{r}");
    }

    #[test]
    fn active_pairs_follow_the_index() {
        let mut p = Pairs::new(&CATEGORIES, PairOptions::default());
        p.list = vec![[0, 1, 5, 2], [0, 1, 6, 3], [4, 0, 69, 1]];
        p.weights = vec![[1.0, 0.5], [10.0, 0.25], [100.0, 0.125]];
        p.rebuild_index();
        let mut data = vec![0u8; 70];
        data[0] = 1;
        data[5] = 2;
        data[6] = 4;
        data[69] = 1;
        let (mut m, mut v) = (0.0, 0.0);
        p.add_active(&data, &mut m, &mut v);
        assert_eq!((m, v), (101.0, 0.625));
        assert_eq!(p.count_active(&data), 2);
        let mut seen = Vec::new();
        p.for_active(&data, |w| seen.push(w[0]));
        assert_eq!(seen, [1.0, 100.0]);
    }
}
