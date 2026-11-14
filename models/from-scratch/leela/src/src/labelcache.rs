//! treelab2 (2026-11-13, the distill fork; fastbin review-1113c's candidate 1): a content-addressed, cross-process
//! cache for the labels `train --label-depth D --label-model KIND:PATH` computes (`train_rounds_labelled`).
//!
//! The distillation runs train six trees and DAGs a cycle on the same set with the same labeller (the field's F-P
//! line, its current version) and the same depth and result weight: six identical computations of
//! `fit_calibration` + `search_labels`. Those are a deterministic function of what they read (rayon's map/collect
//! keeps the order; the thread count changes nothing), so the first line to need them computes and keeps them, in
//! `<GAMES>/.labelcache/labels-<key hash>.bin`, and the others read them. `<GAMES>` is the train command's games
//! directory: in a league the driver's `trainsets/local`, rebuilt each cycle and removed at its end, so the cache
//! needs no pruning; the round directories beside it are all anything else reads there.
//!
//! The key is everything the computation reads: this binary's own bytes (/proc/self/exe, standing for the code), the
//! CHESSAI_* environment (the process-wide switches such as CHESSAI_FX), the labeller's kind, `--use-probs` and file
//! bytes (digested before it loads and again when keying: a file changed meanwhile means no cache), the depth and the
//! result weight (f64 bits), and the round's game records in read order (each game's result, its movetext, from which
//! its moves are parsed, and its positions). The large parts go in as 128-bit digests (two XXH64s with the length);
//! the file holds the whole key, compared in full, and a checksum of its contents. The value is the calibration and
//! every label, bit for bit.
//!
//! Only labellers whose loading reads exactly the one file at PATH (cnbp, lr, probit, bdt, bdd) are cached; own-search
//! labels (no --label-model) belong to the trainee and are never cached. Anything unexpected (unreadable files, a
//! corrupt, truncated or foreign cache file, a directory that can't be written) means computing as before.
//! CHESSAI_LABELCACHE=off turns the cache off, as does any CHESSAI_*_STATS or CHESSAI_*_CHECK variable (diagnostics
//! that count work a hit would skip); CHESSAI_LABELCACHE=log reports each round's outcome on stderr (stdout is as
//! without the cache: the identity checks compare it).
//!
//! Concurrency: a league trains two lines at once. A miss takes an exclusive lock (flock, std's File::lock) on
//! `labels-<key hash>.lock`, looks again, and only then computes, writes and unlocks, so the second line waits for
//! the first's entry instead of computing it too; the kernel drops the lock if its holder dies. Writes go to a
//! unique temporary file in the same directory and are renamed into place, so readers only ever see whole files.

use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

use crate::calibration::Calibration;
use crate::game::GameResult;
use crate::selfplay::GameRecord;

// ---------------------------------------------------------------- XXH64

const P1: u64 = 0x9E37_79B1_85EB_CA87;
const P2: u64 = 0xC2B2_AE3D_27D4_EB4F;
const P3: u64 = 0x1656_67B1_9E37_79F9;
const P4: u64 = 0x85EB_CA77_C2B2_AE63;
const P5: u64 = 0x27D4_EB2F_1656_67C5;

#[inline(always)]
fn read64(b: &[u8], i: usize) -> u64 {
    u64::from_le_bytes(b[i..i + 8].try_into().unwrap())
}

#[inline(always)]
fn round(acc: u64, input: u64) -> u64 {
    acc.wrapping_add(input.wrapping_mul(P2)).rotate_left(31).wrapping_mul(P1)
}

#[inline(always)]
fn merge(acc: u64, val: u64) -> u64 {
    (acc ^ round(0, val)).wrapping_mul(P1).wrapping_add(P4)
}

/// XXH64 (xxHash, 64 bits) of `b` with `seed`, as specified (checked against twox-hash 1.6.3 on 18,000 random
/// inputs of lengths 0-299 and on own-games files; fastbin/hybrid16/hashbench).
pub fn xxh64(seed: u64, b: &[u8]) -> u64 {
    let mut h = Xxh64::new(seed);
    h.update(b);
    h.finish()
}

/// Streaming XXH64: `update` in any pieces gives `xxh64` of their concatenation.
#[derive(Clone)]
pub struct Xxh64 {
    seed: u64,
    v: [u64; 4],
    buf: [u8; 32],
    buffered: usize,
    total: u64,
}

impl Xxh64 {
    pub fn new(seed: u64) -> Self {
        Xxh64 {
            seed,
            v: [seed.wrapping_add(P1).wrapping_add(P2), seed.wrapping_add(P2), seed, seed.wrapping_sub(P1)],
            buf: [0; 32],
            buffered: 0,
            total: 0,
        }
    }

    #[inline(always)]
    fn stripe(v: &mut [u64; 4], b: &[u8], i: usize) {
        v[0] = round(v[0], read64(b, i));
        v[1] = round(v[1], read64(b, i + 8));
        v[2] = round(v[2], read64(b, i + 16));
        v[3] = round(v[3], read64(b, i + 24));
    }

    pub fn update(&mut self, mut b: &[u8]) {
        self.total += b.len() as u64;
        if self.buffered > 0 {
            let take = (32 - self.buffered).min(b.len());
            self.buf[self.buffered..self.buffered + take].copy_from_slice(&b[..take]);
            self.buffered += take;
            b = &b[take..];
            if self.buffered < 32 {
                return;
            }
            let buf = self.buf;
            Self::stripe(&mut self.v, &buf, 0);
            self.buffered = 0;
        }
        let whole = b.len() / 32 * 32;
        let mut i = 0;
        while i < whole {
            Self::stripe(&mut self.v, b, i);
            i += 32;
        }
        let rest = &b[whole..];
        self.buf[..rest.len()].copy_from_slice(rest);
        self.buffered = rest.len();
    }

    pub fn finish(&self) -> u64 {
        let mut h = if self.total >= 32 {
            let v = self.v;
            let mut h = v[0]
                .rotate_left(1)
                .wrapping_add(v[1].rotate_left(7))
                .wrapping_add(v[2].rotate_left(12))
                .wrapping_add(v[3].rotate_left(18));
            for x in v {
                h = merge(h, x);
            }
            h
        } else {
            self.seed.wrapping_add(P5)
        };
        h = h.wrapping_add(self.total);
        let b = &self.buf[..self.buffered];
        let mut i = 0;
        while i + 8 <= b.len() {
            h = (h ^ round(0, read64(b, i))).rotate_left(27).wrapping_mul(P1).wrapping_add(P4);
            i += 8;
        }
        if i + 4 <= b.len() {
            let k = u32::from_le_bytes(b[i..i + 4].try_into().unwrap()) as u64;
            h = (h ^ k.wrapping_mul(P1)).rotate_left(23).wrapping_mul(P2).wrapping_add(P3);
            i += 4;
        }
        while i < b.len() {
            h = (h ^ (b[i] as u64).wrapping_mul(P5)).rotate_left(11).wrapping_mul(P1);
            i += 1;
        }
        h ^= h >> 33;
        h = h.wrapping_mul(P2);
        h ^= h >> 29;
        h = h.wrapping_mul(P3);
        h ^ (h >> 32)
    }
}

/// The two seeds of a 128-bit digest (two XXH64s); the calibration cache's, so the digests are the same function.
const SEEDS: [u64; 2] = [0x6361_6c66_6974_3136, 0x9d2c_5680_1b87_3ac5];

/// A content digest: the length and two XXH64s of the bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Digest {
    pub len: u64,
    pub h: [u64; 2],
}

impl Digest {
    pub fn of(b: &[u8]) -> Self {
        Digest { len: b.len() as u64, h: [xxh64(SEEDS[0], b), xxh64(SEEDS[1], b)] }
    }

    /// The file's digest, read in pieces (no copy of the whole file).
    pub fn of_file(path: &Path) -> io::Result<Self> {
        let mut f = fs::File::open(path)?;
        let mut hs = Hasher::new();
        let mut buf = vec![0u8; 1 << 20];
        loop {
            let n = match f.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            };
            hs.update(&buf[..n]);
        }
        Ok(hs.finish())
    }

    fn put(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.len.to_le_bytes());
        out.extend_from_slice(&self.h[0].to_le_bytes());
        out.extend_from_slice(&self.h[1].to_le_bytes());
    }
}

/// A streaming `Digest`.
struct Hasher {
    hs: [Xxh64; 2],
    len: u64,
}

impl Hasher {
    fn new() -> Self {
        Hasher { hs: [Xxh64::new(SEEDS[0]), Xxh64::new(SEEDS[1])], len: 0 }
    }

    fn update(&mut self, b: &[u8]) {
        self.hs[0].update(b);
        self.hs[1].update(b);
        self.len += b.len() as u64;
    }

    fn finish(&self) -> Digest {
        Digest { len: self.len, h: [self.hs[0].finish(), self.hs[1].finish()] }
    }
}

// ---------------------------------------------------------------- what applies

/// The labeller kinds whose loading reads exactly the one file at PATH (`LoadedModel::load` of `KIND:PATH`).
const KINDS: [&str; 5] = ["cnbp", "lr", "probit", "bdt", "bdd"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Off,
    On,
    Log,
}

/// CHESSAI_LABELCACHE: off, log (on, with a line on stderr per round), anything else or unset: on.
fn mode() -> Mode {
    match std::env::var_os("CHESSAI_LABELCACHE") {
        Some(v) if v == "off" => Mode::Off,
        Some(v) if v == "log" => Mode::Log,
        _ => Mode::On,
    }
}

/// Off, or the CHESSAI_* environment's part of the key (sorted, CHESSAI_LABELCACHE left out).
fn environment() -> Option<Vec<u8>> {
    use std::os::unix::ffi::OsStrExt;
    let mut vars: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
    for (k, v) in std::env::vars_os() {
        let k = k.as_bytes();
        if !k.starts_with(b"CHESSAI_") || k == b"CHESSAI_LABELCACHE" {
            continue;
        }
        if k.ends_with(b"_STATS") || k.ends_with(b"_CHECK") {
            return None;
        }
        vars.push((k.to_vec(), v.as_bytes().to_vec()));
    }
    vars.sort();
    let mut out = Vec::new();
    out.extend_from_slice(&(vars.len() as u64).to_le_bytes());
    for (k, v) in vars {
        for part in [k, v] {
            out.extend_from_slice(&(part.len() as u64).to_le_bytes());
            out.extend_from_slice(&part);
        }
    }
    Some(out)
}

/// This binary's digest, once per process (None if it can't be read: then no cache).
fn exe() -> Option<Digest> {
    static EXE: OnceLock<Option<Digest>> = OnceLock::new();
    *EXE.get_or_init(|| Digest::of_file(Path::new("/proc/self/exe")).ok())
}

fn note(log: bool, text: &str) {
    if log {
        eprintln!("labelcache: {text}");
    }
}

// ---------------------------------------------------------------- the key

/// A labeller the cache may serve: begun before it loads (`begin`), keyed per round after (`Begun::key`).
pub struct Begun {
    head: Vec<u8>,
    model: PathBuf,
    model_before: Digest,
    dir: PathBuf,
    log: bool,
}

/// Starts caching the labels of the labeller `spec` (KIND:PATH) for the train command's `games`, before the
/// labeller loads; None if the cache doesn't apply (turned off, a kind not in KINDS, anything unreadable), which
/// means computing as before.
pub fn begin(spec: &str, use_probs: bool, games: &Path) -> Option<Begun> {
    let m = mode();
    let log = m == Mode::Log;
    if m == Mode::Off {
        return None;
    }
    let Some(env) = environment() else {
        note(log, "off (a CHESSAI_*_STATS or _CHECK variable)");
        return None;
    };
    let (kind, path) = spec.split_once(':')?;
    if !KINDS.contains(&kind) {
        note(log, &format!("not for labeller kind {kind}"));
        return None;
    }
    let exe = exe()?;
    let model_before = Digest::of_file(Path::new(path)).ok()?;
    let mut head = Vec::new();
    head.extend_from_slice(b"chessai labels key 1\n");
    exe.put(&mut head);
    head.extend_from_slice(&env);
    head.extend_from_slice(&(kind.len() as u64).to_le_bytes());
    head.extend_from_slice(kind.as_bytes());
    head.push(use_probs as u8);
    Some(Begun { head, model: PathBuf::from(path), model_before, dir: games.join(".labelcache"), log })
}

/// The digest of a round's game records in read order: count, then each game's result, movetext and positions.
fn records_digest(records: &[GameRecord]) -> Digest {
    let mut h = Hasher::new();
    h.update(&(records.len() as u64).to_le_bytes());
    for r in records {
        h.update(&[match r.result {
            GameResult::WhiteWin => 0u8,
            GameResult::BlackWin => 1,
            GameResult::Draw => 2,
        }]);
        h.update(&(r.movetext.len() as u64).to_le_bytes());
        h.update(r.movetext.as_bytes());
        h.update(&(r.moves.len() as u64).to_le_bytes());
        h.update(&(r.positions.len() as u64).to_le_bytes());
        for p in &r.positions {
            h.update(p);
        }
    }
    h.finish()
}

/// One round's labels' key, complete.
pub struct Keyed {
    key: Vec<u8>,
    file: PathBuf,
    lock: PathBuf,
    dir: PathBuf,
    log: bool,
}

impl Begun {
    /// The key of the labels of `records` at `depth` and `result_weight`: None (compute as before) if the labeller's
    /// file changed since `begin` or can't be read.
    pub fn key(&self, records: &[GameRecord], depth: u32, result_weight: f64) -> Option<Keyed> {
        let model = Digest::of_file(&self.model).ok()?;
        if model != self.model_before {
            note(self.log, "off (the labeller's file changed)");
            return None;
        }
        let mut key = self.head.clone();
        model.put(&mut key);
        key.extend_from_slice(&depth.to_le_bytes());
        key.extend_from_slice(&result_weight.to_bits().to_le_bytes());
        records_digest(records).put(&mut key);
        let name = Digest::of(&key);
        let stem = format!("labels-{:016x}{:016x}", name.h[0], name.h[1]);
        Some(Keyed {
            key,
            file: self.dir.join(format!("{stem}.bin")),
            lock: self.dir.join(format!("{stem}.lock")),
            dir: self.dir.clone(),
            log: self.log,
        })
    }
}

// ---------------------------------------------------------------- the value

/// The calibration and every label of a round, as `fit_calibration` and `search_labels` return them.
pub type Labels = (Calibration, Vec<Vec<[f64; 3]>>);

const MAGIC: &[u8] = b"CHESSAI-LABELS-1\n";

fn encode(key: &[u8], (cal, labels): &Labels) -> Vec<u8> {
    let n: usize = labels.iter().map(Vec::len).sum();
    let mut out = Vec::with_capacity(MAGIC.len() + 8 + key.len() + 32 + 8 + 8 * labels.len() + 24 * n + 16);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&(key.len() as u64).to_le_bytes());
    out.extend_from_slice(key);
    for x in [cal.slope_winner, cal.slope_decisive, cal.offsets[0], cal.offsets[1]] {
        out.extend_from_slice(&x.to_bits().to_le_bytes());
    }
    out.extend_from_slice(&(labels.len() as u64).to_le_bytes());
    for game in labels {
        out.extend_from_slice(&(game.len() as u64).to_le_bytes());
        for l in game {
            for x in l {
                out.extend_from_slice(&x.to_bits().to_le_bytes());
            }
        }
    }
    let sum = Digest::of(&out);
    out.extend_from_slice(&sum.h[0].to_le_bytes());
    out.extend_from_slice(&sum.h[1].to_le_bytes());
    out
}

/// The labels in a cache file's bytes if they are whole, of this format, of exactly this key and of the round's
/// shape (a label for every position of every game).
fn decode(bytes: &[u8], key: &[u8], records: &[GameRecord]) -> Option<Labels> {
    let body = bytes.get(..bytes.len().checked_sub(16)?)?;
    let sum = Digest::of(body);
    let tail = &bytes[body.len()..];
    if read64(tail, 0) != sum.h[0] || read64(tail, 8) != sum.h[1] {
        return None;
    }
    let mut at = 0usize;
    let mut take = |n: usize| -> Option<&[u8]> {
        let s = body.get(at..at.checked_add(n)?)?;
        at += n;
        Some(s)
    };
    if take(MAGIC.len())? != MAGIC {
        return None;
    }
    let u64_at = |s: &[u8]| u64::from_le_bytes(s.try_into().unwrap());
    let key_len = usize::try_from(u64_at(take(8)?)).ok()?;
    if take(key_len)? != key {
        return None;
    }
    let mut c = [0f64; 4];
    for x in &mut c {
        *x = f64::from_bits(u64_at(take(8)?));
    }
    let cal = Calibration { slope_winner: c[0], slope_decisive: c[1], offsets: [c[2], c[3]] };
    let games = usize::try_from(u64_at(take(8)?)).ok()?;
    if games != records.len() {
        return None;
    }
    let mut labels = Vec::with_capacity(games);
    for r in records {
        let n = usize::try_from(u64_at(take(8)?)).ok()?;
        if n != r.positions.len() {
            return None;
        }
        let raw = take(n.checked_mul(24)?)?;
        labels.push(
            raw.chunks_exact(24)
                .map(|l| std::array::from_fn(|k| f64::from_bits(u64_at(&l[8 * k..8 * k + 8]))))
                .collect(),
        );
    }
    if at != body.len() {
        return None;
    }
    Some((cal, labels))
}

static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

impl Keyed {
    /// The cached labels, if there are some for exactly this key and round.
    fn lookup(&self, records: &[GameRecord]) -> Option<Labels> {
        decode(&fs::read(&self.file).ok()?, &self.key, records)
    }

    /// Keeps the labels for the next process: a unique temporary file in the same directory, renamed into place.
    /// Failure of any kind leaves things as they were (and removes the temporary file if it was made).
    fn store(&self, labels: &Labels) -> bool {
        let bytes = encode(&self.key, labels);
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos());
        let tmp = self.dir.join(format!(
            ".labels-tmp-{}-{}-{nanos}",
            std::process::id(),
            TMP_COUNTER.fetch_add(1, Relaxed)
        ));
        let written = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)
            .and_then(|mut f| f.write_all(&bytes));
        match written {
            Ok(()) => {
                if fs::rename(&tmp, &self.file).is_ok() {
                    return true;
                }
                let _ = fs::remove_file(&tmp);
            }
            Err(e) => {
                if e.kind() != io::ErrorKind::AlreadyExists {
                    let _ = fs::remove_file(&tmp);
                }
            }
        }
        false
    }

    /// The round's labels: the cached ones, or `compute`'s, kept for the next process. A miss takes the key's lock
    /// first and looks again under it, so of processes started together one computes and the rest wait and read.
    pub fn labels(&self, records: &[GameRecord], compute: impl FnOnce() -> Labels) -> Labels {
        if let Some(found) = self.lookup(records) {
            note(self.log, &format!("hit {}", self.file.display()));
            return found;
        }
        let lock = fs::create_dir_all(&self.dir)
            .and_then(|()| fs::OpenOptions::new().create(true).truncate(false).write(true).open(&self.lock))
            .and_then(|f| f.lock().map(|()| f));
        if lock.is_ok() {
            if let Some(found) = self.lookup(records) {
                note(self.log, &format!("hit after waiting {}", self.file.display()));
                return found;
            }
        }
        let made = compute();
        let kept = lock.is_ok() && self.store(&made);
        note(
            self.log,
            &format!(
                "miss: computed{} {}",
                if kept { " and kept" } else { ", not kept (no lock or directory)" },
                self.file.display()
            ),
        );
        made
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoding::FEATURES;

    /// Known XXH64 values (the reference's test vectors) and streaming in any pieces.
    #[test]
    fn xxh64_matches_the_reference_in_any_pieces() {
        assert_eq!(xxh64(0, b""), 0xef46_db37_51d8_e999);
        assert_eq!(xxh64(0, b"a"), 0xd24e_c4f1_a98c_6e5b);
        assert_eq!(xxh64(0, b"abc"), 0x44bc_2cf5_ad77_0999);
        let mut x = 99u64;
        let data: Vec<u8> = (0..1000)
            .map(|_| {
                x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                (x >> 33) as u8
            })
            .collect();
        for len in [0, 1, 3, 4, 7, 8, 31, 32, 33, 63, 64, 65, 100, 1000] {
            let whole = xxh64(7, &data[..len]);
            for piece in [1, 3, 5, 31, 32, 33, 64] {
                let mut h = Xxh64::new(7);
                data[..len].chunks(piece).for_each(|c| h.update(c));
                assert_eq!(h.finish(), whole, "len {len} piece {piece}");
            }
        }
    }

    fn rec(result: GameResult, text: &str, positions: &[u8]) -> GameRecord {
        GameRecord {
            result,
            moves: Vec::new(),
            positions: positions.iter().map(|&b| [b; FEATURES]).collect(),
            movetext: text.to_string(),
        }
    }

    fn round() -> Vec<GameRecord> {
        vec![rec(GameResult::WhiteWin, "1. e4 e5 1-0", &[1, 2]), rec(GameResult::Draw, "1. d4 1/2-1/2", &[3])]
    }

    fn labels_of(records: &[GameRecord], salt: f64) -> Labels {
        let cal = Calibration { slope_winner: 0.9 + salt, slope_decisive: -4.5, offsets: [-10.25, f64::NAN] };
        let labels = records
            .iter()
            .enumerate()
            .map(|(g, r)| (0..r.positions.len()).map(|i| [salt + g as f64, 0.1 * i as f64, -0.0]).collect())
            .collect();
        (cal, labels)
    }

    fn bits((c, l): &Labels) -> Vec<u64> {
        let mut out = vec![c.slope_winner.to_bits(), c.slope_decisive.to_bits(), c.offsets[0].to_bits(), c.offsets[1].to_bits()];
        for g in l {
            out.push(g.len() as u64);
            for x in g.iter().flatten() {
                out.push(x.to_bits());
            }
        }
        out
    }

    fn scratch(name: &str) -> PathBuf {
        // On disk beside the build (target/), not /tmp.
        let d = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target").join(format!("labelcache-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn labels_round_trip_and_anything_else_misses() {
        let records = round();
        let made = labels_of(&records, 0.5);
        let key = b"some key".to_vec();
        let bytes = encode(&key, &made);
        assert_eq!(decode(&bytes, &key, &records).map(|l| bits(&l)), Some(bits(&made)));
        assert!(decode(&bytes, b"other key", &records).is_none());
        let mut fewer = round();
        fewer[1].positions.clear();
        assert!(decode(&bytes, &key, &fewer).is_none(), "another shape");
        assert!(decode(&bytes, &key, &records[..1]).is_none(), "fewer games");
        for i in 0..bytes.len() {
            let mut bad = bytes.clone();
            bad[i] ^= 0x10;
            assert!(decode(&bad, &key, &records).is_none(), "flipped byte {i}");
            assert!(decode(&bytes[..i], &key, &records).is_none(), "cut at {i}");
        }
        let mut longer = bytes.clone();
        longer.push(0);
        assert!(decode(&longer, &key, &records).is_none());
    }

    #[test]
    fn every_part_of_the_key_changes_it() {
        let d = scratch("key");
        let model = d.join("m.model");
        fs::write(&model, b"model bytes").unwrap();
        let spec = format!("probit:{}", model.display());
        let games = d.join("games");
        let base = begin(&spec, true, &games).unwrap().key(&round(), 2, 0.5).unwrap();
        let mut keys = vec![base.key.clone()];
        keys.push(begin(&spec, false, &games).unwrap().key(&round(), 2, 0.5).unwrap().key);
        keys.push(begin(&format!("lr:{}", model.display()), true, &games).unwrap().key(&round(), 2, 0.5).unwrap().key);
        keys.push(begin(&spec, true, &games).unwrap().key(&round(), 3, 0.5).unwrap().key);
        keys.push(begin(&spec, true, &games).unwrap().key(&round(), 2, 0.25).unwrap().key);
        let mut r = round();
        r[0].result = GameResult::BlackWin;
        keys.push(begin(&spec, true, &games).unwrap().key(&r, 2, 0.5).unwrap().key);
        let mut r = round();
        r[1].movetext.push(' ');
        keys.push(begin(&spec, true, &games).unwrap().key(&r, 2, 0.5).unwrap().key);
        let mut r = round();
        r[1].positions[0][FEATURES - 1] ^= 1;
        keys.push(begin(&spec, true, &games).unwrap().key(&r, 2, 0.5).unwrap().key);
        let mut r = round();
        r.swap(0, 1);
        keys.push(begin(&spec, true, &games).unwrap().key(&r, 2, 0.5).unwrap().key);
        keys.push(begin(&spec, true, &games).unwrap().key(&round()[..1], 2, 0.5).unwrap().key);
        let begun = begin(&spec, true, &games).unwrap();
        fs::write(&model, b"model bytez").unwrap();
        assert!(begun.key(&round(), 2, 0.5).is_none(), "the labeller changed after begin");
        keys.push(begin(&spec, true, &games).unwrap().key(&round(), 2, 0.5).unwrap().key);
        fs::write(&model, b"model bytes").unwrap();
        assert_eq!(begin(&spec, true, &games).unwrap().key(&round(), 2, 0.5).unwrap().key, base.key, "same inputs");
        let n = keys.len();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), n, "every change gives another key");
        assert!(begin(&format!("cal:{}", model.display()), true, &games).is_none(), "wrappers aren't cached");
        assert!(begin(&format!("moe:{}", model.display()), true, &games).is_none());
        assert!(begin(&format!("probit:{}", d.join("none").display()), true, &games).is_none());
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn a_miss_computes_and_keeps_a_hit_reads_and_a_bad_file_is_recomputed() {
        let d = scratch("flow");
        let model = d.join("m.model");
        fs::write(&model, b"labeller").unwrap();
        let spec = format!("bdt:{}", model.display());
        let games = d.join("games");
        let records = round();
        let k = begin(&spec, true, &games).unwrap().key(&records, 2, 0.5).unwrap();
        let mut calls = 0;
        let a = k.labels(&records, || {
            calls += 1;
            labels_of(&records, 0.25)
        });
        assert_eq!(calls, 1);
        assert!(k.file.exists() && k.lock.exists());
        let b = k.labels(&records, || {
            calls += 1;
            labels_of(&records, 9.0)
        });
        assert_eq!((calls, bits(&b)), (1, bits(&a)), "a hit, the same bits");
        let mut bytes = fs::read(&k.file).unwrap();
        let mid = bytes.len() / 2;
        bytes[mid] ^= 1;
        fs::write(&k.file, &bytes).unwrap();
        let c = k.labels(&records, || {
            calls += 1;
            labels_of(&records, 0.25)
        });
        assert_eq!((calls, bits(&c)), (2, bits(&a)), "a corrupt file: computed again");
        let leftovers: Vec<_> = fs::read_dir(&k.dir).unwrap().map(|e| e.unwrap().file_name()).collect();
        assert_eq!(leftovers.len(), 2, "the entry and its lock, no temporary files: {leftovers:?}");
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn of_six_threads_started_together_one_computes() {
        use std::sync::atomic::AtomicUsize;
        let d = scratch("six");
        let model = d.join("m.model");
        fs::write(&model, b"labeller").unwrap();
        let spec = format!("probit:{}", model.display());
        let games = d.join("games");
        let records = round();
        let calls = AtomicUsize::new(0);
        let barrier = std::sync::Barrier::new(6);
        let got: Vec<Vec<u64>> = std::thread::scope(|s| {
            let hs: Vec<_> = (0..6)
                .map(|_| {
                    s.spawn(|| {
                        let k = begin(&spec, true, &games).unwrap().key(&records, 2, 0.5).unwrap();
                        barrier.wait();
                        bits(&k.labels(&records, || {
                            calls.fetch_add(1, Relaxed);
                            std::thread::sleep(std::time::Duration::from_millis(200));
                            labels_of(&records, 0.75)
                        }))
                    })
                })
                .collect();
            hs.into_iter().map(|h| h.join().unwrap()).collect()
        });
        assert_eq!(calls.load(Relaxed), 1, "one computed, five read");
        assert!(got.iter().all(|g| *g == got[0]));
        let _ = fs::remove_dir_all(&d);
    }
}
