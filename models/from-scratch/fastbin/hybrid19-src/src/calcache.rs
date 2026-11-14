//! hybrid16: a content-addressed, cross-process cache for calibration fits (`cal:`, `leafcal:` and `blend:`
//! models, fitted on a league's own games; see `Calibrated::fit` in main.rs).
//!
//! A league fits the same calibration again and again: every tournament and every self-games run of a cycle loads
//! `cal:<league>/owncal/<line>@<v>:KIND:<model>` and fits it from scratch, and the champions' fits repeat every
//! cycle. The fit is a deterministic function of what it reads, so it is kept, once made, in a small file beside
//! the games it was fitted on: `<GAMES>/calfit-<key hash>.bin`. Only own-games directories are used, those whose
//! parent directory is named `owncal` (a league's or an arena's): they are written once and deleted with their
//! directory when their version leaves the field (leagues.py's own_gc), so the cache needs no pruning of its own
//! and nothing else reads them (read_labelled and leagues.py read only the outcome files). Shared held-out sets are
//! never written to.
//!
//! The key is everything the fit reads: this binary's own bytes (/proc/self/exe, standing for the code), the
//! CHESSAI_* environment, the calibration's options (cal, leafcal or blend and its TAU), the inner model's kind,
//! `--use-probs` and file bytes, and the bytes of the three outcome files in the order `read_labelled` reads them.
//! Each part is hashed with two XXH64s (128 bits) with its length; the file holds the whole key, compared in full,
//! and a checksum of its contents. The value is the fit bit for bit: the number of positions, the whole model's
//! calibration and each leaf's (f64 bits), and the leaf lines `fit` printed, so a hit prints and reports the same
//! text (the first line's path is the caller's, as `fit` prints it).
//!
//! Anything unexpected (unreadable files, a file changing while it is read, a corrupt or foreign cache file, a
//! directory that can't be written) means fitting as before, silently. CHESSAI_CALCACHE=off turns the cache off;
//! so does any CHESSAI_*_STATS or CHESSAI_*_CHECK variable, whose diagnostics count work a hit would skip.
//! Writes go to a unique temporary file in the same directory and are renamed into place, so concurrent processes
//! only ever see whole files; two writers of one key write the same bytes.

use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

use crate::encoding::Encoded;
use crate::games::{OUTCOME_FILES, parse_position};

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

/// The two seeds of a 128-bit digest (two XXH64s).
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
        Self::of_file_with(path, &mut Vec::new())
    }

    /// `of_file` reading through `buf` (grown to 1 MiB once, then reused).
    pub fn of_file_with(path: &Path, buf: &mut Vec<u8>) -> io::Result<Self> {
        let mut f = fs::File::open(path)?;
        let mut hs = [Xxh64::new(SEEDS[0]), Xxh64::new(SEEDS[1])];
        buf.resize(1 << 20, 0);
        let mut len = 0u64;
        loop {
            let n = match f.read(buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e),
            };
            hs[0].update(&buf[..n]);
            hs[1].update(&buf[..n]);
            len += n as u64;
        }
        Ok(Digest { len, h: [hs[0].finish(), hs[1].finish()] })
    }

    fn put(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.len.to_le_bytes());
        out.extend_from_slice(&self.h[0].to_le_bytes());
        out.extend_from_slice(&self.h[1].to_le_bytes());
    }
}

// ---------------------------------------------------------------- what applies

/// The inner model kinds whose loading reads exactly the one file at PATH (`KIND::load` is `from_text` of the
/// file, with `--use-probs` where the kind takes it), the kinds `load_report` vouches for.
const KINDS: [&str; 5] = ["cnbp", "lr", "probit", "bdt", "bdd"];

/// Off, or the CHESSAI_* environment's part of the key (sorted, CHESSAI_CALCACHE left out).
fn environment() -> Option<Vec<u8>> {
    use std::os::unix::ffi::OsStrExt;
    let mut vars: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
    for (k, v) in std::env::vars_os() {
        let k = k.as_bytes();
        if !k.starts_with(b"CHESSAI_") {
            continue;
        }
        if k == b"CHESSAI_CALCACHE" {
            if v.as_bytes() == b"off" {
                return None;
            }
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

fn is_own_games(games: &Path) -> bool {
    games.parent().and_then(|p| p.file_name()).is_some_and(|n| n == "owncal")
}

// ---------------------------------------------------------------- the key

/// A fit the cache may serve: begun before the inner model loads (`begin`), keyed after (`Begun::key`).
pub struct Begun {
    head: Vec<u8>,
    model: PathBuf,
    model_before: Digest,
    games: PathBuf,
}

/// Starts a cached fit of `wrapper` (cal, leafcal or blend, with `blend`'s TAU) on `games` of the inner model
/// `inner` (KIND:PATH), before the inner model loads; None if the cache doesn't apply, which means fitting as
/// before: turned off, games not an own-games directory, an inner kind not in KINDS, or anything unreadable.
pub fn begin(wrapper: &str, per_leaf: bool, blend: Option<f64>, inner: &str, use_probs: bool, games: &Path) -> Option<Begun> {
    let env = environment()?;
    if !is_own_games(games) {
        return None;
    }
    let (kind, path) = inner.split_once(':')?;
    if !KINDS.contains(&kind) {
        return None;
    }
    let exe = exe()?;
    let model_before = Digest::of_file(Path::new(path)).ok()?;
    let mut head = Vec::new();
    head.extend_from_slice(b"chessai calfit key 1\n");
    exe.put(&mut head);
    head.extend_from_slice(&env);
    for s in [wrapper, kind] {
        head.extend_from_slice(&(s.len() as u64).to_le_bytes());
        head.extend_from_slice(s.as_bytes());
    }
    head.push(per_leaf as u8);
    match blend {
        Some(tau) => {
            head.push(1);
            head.extend_from_slice(&tau.to_bits().to_le_bytes());
        }
        None => head.push(0),
    }
    head.push(use_probs as u8);
    Some(Begun { head, model: PathBuf::from(path), model_before, games: games.to_path_buf() })
}

/// A fit's key, complete.
pub struct Keyed {
    key: Vec<u8>,
    file: PathBuf,
    games: [Digest; 3],
}

impl Begun {
    /// The key, once the inner model has loaded: None (fit as before) if the model file changed meanwhile or a
    /// file can't be read.
    pub fn key(self) -> Option<Keyed> {
        let mut buf = Vec::new();
        let model = Digest::of_file_with(&self.model, &mut buf).ok()?;
        if model != self.model_before {
            return None;
        }
        let mut key = self.head;
        model.put(&mut key);
        let mut games = [Digest { len: 0, h: [0; 2] }; 3];
        for (g, (file, _)) in games.iter_mut().zip(OUTCOME_FILES) {
            *g = Digest::of_file_with(&self.games.join(file), &mut buf).ok()?;
            g.put(&mut key);
        }
        let name = Digest::of(&key);
        let file = self.games.join(format!("calfit-{:016x}{:016x}.bin", name.h[0], name.h[1]));
        Some(Keyed { key, file, games })
    }
}

// ---------------------------------------------------------------- the value

/// A fit, bit for bit: positions, the whole model's calibration and each leaf's (slope_winner, slope_decisive,
/// offsets[0], offsets[1] as f64 bits; None for a leaf without its own), and the leaf lines `fit` printed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fit {
    pub positions: u64,
    pub whole: [u64; 4],
    pub leaves: Vec<Option<[u64; 4]>>,
    pub leaf_lines: String,
}

const MAGIC: &[u8] = b"CHESSAI-CALFIT-1\n";

fn encode(key: &[u8], fit: &Fit) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&(key.len() as u64).to_le_bytes());
    out.extend_from_slice(key);
    out.extend_from_slice(&fit.positions.to_le_bytes());
    fit.whole.iter().for_each(|x| out.extend_from_slice(&x.to_le_bytes()));
    out.extend_from_slice(&(fit.leaves.len() as u64).to_le_bytes());
    for leaf in &fit.leaves {
        match leaf {
            Some(c) => {
                out.push(1);
                c.iter().for_each(|x| out.extend_from_slice(&x.to_le_bytes()));
            }
            None => out.push(0),
        }
    }
    out.extend_from_slice(&(fit.leaf_lines.len() as u64).to_le_bytes());
    out.extend_from_slice(fit.leaf_lines.as_bytes());
    let sum = Digest::of(&out);
    out.extend_from_slice(&sum.h[0].to_le_bytes());
    out.extend_from_slice(&sum.h[1].to_le_bytes());
    out
}

/// The fit in a cache file's bytes if they are whole, of this format and of exactly this key.
fn decode(bytes: &[u8], key: &[u8]) -> Option<Fit> {
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
    let positions = u64_at(take(8)?);
    let mut whole = [0u64; 4];
    for x in &mut whole {
        *x = u64_at(take(8)?);
    }
    let n_leaves = usize::try_from(u64_at(take(8)?)).ok()?;
    if n_leaves > body.len() {
        return None;
    }
    let mut leaves = Vec::with_capacity(n_leaves);
    for _ in 0..n_leaves {
        match take(1)?[0] {
            0 => leaves.push(None),
            1 => {
                let mut c = [0u64; 4];
                for x in &mut c {
                    *x = u64_at(take(8)?);
                }
                leaves.push(Some(c));
            }
            _ => return None,
        }
    }
    let text_len = usize::try_from(u64_at(take(8)?)).ok()?;
    let leaf_lines = String::from_utf8(take(text_len)?.to_vec()).ok()?;
    if at != body.len() {
        return None;
    }
    Some(Fit { positions, whole, leaves, leaf_lines })
}

static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

impl Keyed {
    /// The cached fit, if there is one for exactly this key.
    pub fn lookup(&self) -> Option<Fit> {
        decode(&fs::read(&self.file).ok()?, &self.key)
    }

    /// The labelled positions of the own games as `read_labelled` reads them, read again in full and parsed from
    /// those very bytes, if they are still the bytes of the key (else None: fit as before, not stored).
    pub fn read_labelled(&self, games: &Path) -> Option<Vec<(Encoded, [f64; 3])>> {
        let mut data = Vec::new();
        for ((file, label), digest) in OUTCOME_FILES.iter().zip(&self.games) {
            let bytes = fs::read(games.join(file)).ok()?;
            if Digest::of(&bytes) != *digest {
                return None;
            }
            let text = String::from_utf8(bytes).ok()?;
            let positions = text
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(parse_position)
                .collect::<io::Result<Vec<Encoded>>>()
                .ok()?;
            data.extend(positions.into_iter().map(|p| (p, *label)));
        }
        Some(data)
    }

    /// Keeps the fit for the next process: a unique temporary file in the same directory, renamed into place.
    /// Failure of any kind leaves things as they were (and removes the temporary file if it was made).
    pub fn store(&self, fit: &Fit) {
        let bytes = encode(&self.key, fit);
        let Some(dir) = self.file.parent() else { return };
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos());
        let tmp = dir.join(format!(
            ".calfit-tmp-{}-{}-{nanos}",
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
                if fs::rename(&tmp, &self.file).is_err() {
                    let _ = fs::remove_file(&tmp);
                }
            }
            Err(e) => {
                if e.kind() != io::ErrorKind::AlreadyExists {
                    let _ = fs::remove_file(&tmp);
                }
            }
        }
    }

    pub fn file(&self) -> &Path {
        &self.file
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn a_fit_round_trips_and_anything_else_misses() {
        let fit = Fit {
            positions: 12345,
            whole: [1.5f64.to_bits(), (-0.25f64).to_bits(), 0.1f64.to_bits(), f64::NAN.to_bits()],
            leaves: vec![None, Some([1, 2, 3, u64::MAX]), None],
            leaf_lines: "  leaf 1: 100 positions, 50.0% decisive, calibration the whole model's\n".to_string(),
        };
        let key = b"some key".to_vec();
        let bytes = encode(&key, &fit);
        assert_eq!(decode(&bytes, &key), Some(fit.clone()));
        assert_eq!(decode(&bytes, b"other key"), None);
        for i in 0..bytes.len() {
            let mut bad = bytes.clone();
            bad[i] ^= 0x10;
            assert_eq!(decode(&bad, &key), None, "flipped byte {i}");
            assert_eq!(decode(&bytes[..i], &key), None, "cut at {i}");
        }
        let mut longer = bytes.clone();
        longer.push(0);
        assert_eq!(decode(&longer, &key), None);
    }
}
