//! Rating tournaments: repeated double round-robins between players (models
//! at given search depths), with an Elo rating list and a head-to-head table
//! updated after every tournament.
//!
//! In each tournament every player meets every other once with each colour.
//! Its games are then appended to `games.csv`, and the ratings and tables
//! are recomputed from all games so far, so a run can be stopped, resumed or
//! extended with more tournaments. Two ratings are kept:
//! - the running Elo, the classic update: all players start at 0, and after
//!   each tournament each gains `k` × (points − expected points) over its
//!   games in it, a game's expected points following from the two players'
//!   ratings before the tournament, so a win against a stronger opponent
//!   gains more;
//! - the maximum-likelihood Elo of all games so far (Bradley-Terry, draws as
//!   half points), which weighs results by the opponents' strengths in the
//!   same way but does not depend on the order of the games, with a 95%
//!   interval. By default every pairing also gets `prior_draws` virtual
//!   draws, so that every rating is finite; as real games accumulate their
//!   pull fades, and the ratings spread out from one tournament to the next.
//!   With `prior_draws` 0 the ratings are the pure maximum-likelihood ones
//!   (`ml_fit`), and players whose rating does not exist are listed as
//!   unrated. In a big league `sparse_fit` gives the same ratings and errors
//!   much faster (`sparse_fit` in the options), and the tables can be
//!   written once at the end of a run instead (`reports_last`).
//!
//! Players can join later: listed with the others, they first play the games
//! of the earlier tournaments they missed, as if they had been there, unless
//! `catch_up` is off: then each tournament is a round-robin of the players
//! listed at the time. Players can also retire: they play no more games, but
//! their games stay in the ratings.
//!
//! The games themselves can also be saved, to train on (`save_games`): each
//! game played in `dir/round{t}/{white}~{black}/`, in the self-play round
//! format (see `selfplay::write_round`), t being its tournament from 0 as in
//! `games.csv`. `train dir` then trains on whole tournaments, one per round.

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::fs;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::time::Instant;

use rayon::prelude::*;

use crate::game::GameResult;
use crate::player::Player;
use crate::selfplay::{derive_seed, play_game, write_round};

/// Elo points per unit of the natural log of the odds of winning.
const ELO: f64 = 400.0 / std::f64::consts::LN_10;

/// A fast deterministic hasher (FxHash's rotate, xor and multiply) for the
/// maps whose order does not matter: those that are only looked up, or
/// whose entries are sorted before use.
#[derive(Default, Clone, Copy)]
struct Fx(u64);

impl Fx {
    #[inline]
    fn add(&mut self, word: u64) {
        self.0 = (self.0.rotate_left(5) ^ word).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95);
    }
}

impl std::hash::Hasher for Fx {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        let mut chunks = bytes.chunks_exact(8);
        for c in &mut chunks {
            self.add(u64::from_le_bytes(c.try_into().unwrap()));
        }
        let rest = chunks.remainder();
        if !rest.is_empty() {
            let mut word = [0u8; 8];
            word[..rest.len()].copy_from_slice(rest);
            self.add(u64::from_le_bytes(word));
        }
    }
    fn write_u8(&mut self, i: u8) {
        self.add(i as u64)
    }
    fn write_u32(&mut self, i: u32) {
        self.add(i as u64)
    }
    fn write_u64(&mut self, i: u64) {
        self.add(i)
    }
    fn write_usize(&mut self, i: usize) {
        self.add(i as u64)
    }
}

type FxBuild = std::hash::BuildHasherDefault<Fx>;

/// A player in the tournament and its name in the tables.
pub struct Entrant<'a> {
    pub name: String,
    pub player: Player<'a>,
}

/// A played game: its tournament, the white and black players (indices of
/// the entrants), white's points and the game's length in plies.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Played {
    pub tournament: u64,
    pub white: usize,
    pub black: usize,
    pub white_points: f64,
    pub plies: usize,
}

pub struct Options {
    /// Tournaments to have played in all, counting those already in `out`.
    pub tournaments: u64,
    pub seed: u64,
    /// K-factor of the running Elo, per game.
    pub k: f64,
    /// Virtual draws added to every pairing for the maximum-likelihood
    /// ratings, so that a perfect score gives a finite rating; 0 for the
    /// pure maximum-likelihood ratings (see `ml_fit`).
    pub prior_draws: f64,
    /// The entrant rated 0 in the tables; the field's mean if None.
    pub anchor: Option<usize>,
    /// First play the games of earlier tournaments that players who joined
    /// later missed (see `missing_games`).
    pub catch_up: bool,
    /// Also write each game played to its directory in this one (see
    /// `game_dir`); games already in `games.csv` are not replayed.
    pub save_games: Option<PathBuf>,
    /// Write the tables only after the last tournament of the run, not
    /// after each one.
    pub reports_last: bool,
    /// Fit the pure maximum-likelihood ratings by `sparse_fit` instead of
    /// `ml_fit` (only with `prior_draws` 0).
    pub sparse_fit: bool,
    /// Also append each game's CPU time, measured on the thread that played
    /// it, to `game_cpu.csv` (tournament,white,black,cpu_s) beside
    /// `games.csv`. The games themselves are the same either way.
    pub game_cpu: bool,
}

/// The CPU time this thread has used, in seconds (CLOCK_THREAD_CPUTIME_ID).
/// A game is played start to end on one thread (`play_game` makes no
/// parallel calls), so the difference around it is the game's CPU time.
pub fn thread_cpu_seconds() -> f64 {
    #[repr(C)]
    struct Timespec {
        tv_sec: i64,
        tv_nsec: i64,
    }
    unsafe extern "C" {
        fn clock_gettime(clock: i32, tp: *mut Timespec) -> i32;
    }
    const CLOCK_THREAD_CPUTIME_ID: i32 = 3;
    let mut t = Timespec { tv_sec: 0, tv_nsec: 0 };
    // SAFETY: clock_gettime writes one timespec into `t`, which outlives the call.
    let r = unsafe { clock_gettime(CLOCK_THREAD_CPUTIME_ID, &mut t) };
    if r != 0 {
        return f64::NAN;
    }
    t.tv_sec as f64 + t.tv_nsec as f64 * 1e-9
}

/// Where `save_games` puts the game of tournament `t` between `white` and
/// `black`: `dir/round{t}/{white}~{black}/`.
pub fn game_dir(dir: &Path, t: u64, white: &str, black: &str) -> PathBuf {
    dir.join(format!("round{t}")).join(format!("{white}~{black}"))
}

/// Every ordered pair of different entrants as (white, black): each pairing
/// once with each colour.
pub fn pairings(n: usize) -> Vec<(usize, usize)> {
    (0..n).flat_map(|white| (0..n).filter(move |&black| black != white).map(move |black| (white, black))).collect()
}

/// Plays tournament `t` in parallel. Game `g` (in the order of `pairings`)
/// uses the seed `derive_seed(seed, t, g)`, so the games do not depend on the
/// number of threads. With `save`, writes each game to its `game_dir` in it.
pub fn play_tournament(entrants: &[Entrant], t: u64, seed: u64, save: Option<&Path>) -> io::Result<Vec<Played>> {
    Ok(play_tournament_timed(entrants, t, seed, save)?.into_iter().map(|(g, _)| g).collect())
}

/// As `play_tournament`, with each game's CPU time in seconds (see
/// `thread_cpu_seconds`).
pub fn play_tournament_timed(
    entrants: &[Entrant],
    t: u64,
    seed: u64,
    save: Option<&Path>,
) -> io::Result<Vec<(Played, f64)>> {
    let games: Vec<(u64, usize, usize)> = pairings(entrants.len()).into_iter().map(|(w, b)| (t, w, b)).collect();
    play_games(entrants, &games, seed, save)
}

/// Plays `games`, as (tournament, white, black), in parallel; each is seeded
/// as in `play_tournament`, by its pairing's place in `pairings`.
fn play_games(
    entrants: &[Entrant],
    games: &[(u64, usize, usize)],
    seed: u64,
    save: Option<&Path>,
) -> io::Result<Vec<(Played, f64)>> {
    let order = pairings(entrants.len());
    games
        .par_iter()
        .map(|&(t, white, black)| {
            let g = order.iter().position(|&p| p == (white, black)).expect("no such pairing");
            let cpu0 = thread_cpu_seconds();
            let record = play_game(entrants[white].player, entrants[black].player, derive_seed(seed, t, g as u64));
            let cpu = thread_cpu_seconds() - cpu0;
            if let Some(dir) = save {
                let dir = game_dir(dir, t, &entrants[white].name, &entrants[black].name);
                write_round(&dir, std::slice::from_ref(&record))?;
            }
            let played =
                Played { tournament: t, white, black, white_points: points(record.result), plies: record.moves.len() };
            Ok((played, cpu))
        })
        .collect()
}

/// Appends each game's CPU time to `game_cpu.csv` at `path`.
fn append_cpu(path: &Path, names: &[&str], played: &[(Played, f64)]) -> io::Result<()> {
    if !path.exists() {
        fs::write(path, "tournament,white,black,cpu_s\n")?;
    }
    let mut csv = String::new();
    for (g, cpu) in played {
        writeln!(csv, "{},{},{},{:.6}", g.tournament, names[g.white], names[g.black], cpu).unwrap();
    }
    fs::OpenOptions::new().append(true).open(path)?.write_all(csv.as_bytes())
}

/// The games of tournaments `0..tournaments` not in `games`, as (tournament,
/// white, black): those of players who joined later. An error if a pairing
/// played twice in a tournament.
pub fn missing_games(n: usize, games: &[Played], tournaments: u64) -> Result<Vec<(u64, usize, usize)>, String> {
    let mut seen = vec![false; tournaments as usize * n * n];
    for g in games {
        let i = (g.tournament as usize * n + g.white) * n + g.black;
        if std::mem::replace(&mut seen[i], true) {
            return Err(format!("tournament {} has {} against {} twice", g.tournament, g.white, g.black));
        }
    }
    let pairs = pairings(n);
    Ok((0..tournaments)
        .flat_map(|t| pairs.iter().map(move |&(w, b)| (t, w, b)))
        .filter(|&(t, w, b)| !seen[(t as usize * n + w) * n + b])
        .collect())
}

/// Appends `played` to `games.csv` at `path`.
fn append(path: &Path, names: &[&str], played: &[Played]) -> io::Result<()> {
    let mut csv = String::new();
    for g in played {
        let result = ["0-1", "1/2-1/2", "1-0"][(2.0 * g.white_points) as usize];
        writeln!(csv, "{},{},{},{result},{}", g.tournament, names[g.white], names[g.black], g.plies).unwrap();
    }
    fs::OpenOptions::new().append(true).open(path)?.write_all(csv.as_bytes())
}

fn points(result: GameResult) -> f64 {
    match result {
        GameResult::WhiteWin => 1.0,
        GameResult::BlackWin => 0.0,
        GameResult::Draw => 0.5,
    }
}

/// Plays tournaments of `entrants` until `opts.tournaments` have been played
/// in `out`, continuing after the games already in `out/games.csv` (first,
/// with `opts.catch_up`, the games players who joined later missed), and
/// rewrites the ratings and tables after each one (with `reports_last`, only
/// after the last; or once, if none are left to play). The `retired`
/// players play no more, but their games in
/// `games.csv` count; every player there must be an entrant or retired.
/// Prints a line per tournament.
pub fn run(entrants: &[Entrant], retired: &[String], out: &Path, opts: &Options) -> io::Result<()> {
    let invalid_input = |msg: String| io::Error::new(io::ErrorKind::InvalidInput, msg);
    if !(opts.prior_draws >= 0.0) {
        return Err(invalid_input(format!("prior_draws must be 0 or more, not {}", opts.prior_draws)));
    }
    if opts.sparse_fit && opts.prior_draws != 0.0 {
        return Err(invalid_input("the sparse fit is for prior_draws 0 only".to_string()));
    }
    let active = entrants.len();
    let names: Vec<&str> = entrants.iter().map(|e| e.name.as_str()).chain(retired.iter().map(String::as_str)).collect();
    let mut seen = HashSet::new();
    if let Some(name) = names.iter().find(|&&n| !seen.insert(n)) {
        return Err(invalid_input(format!("player {name} is listed twice (or both listed and retired)")));
    }
    if let Some(name) =
        names[..active].iter().find(|n| opts.save_games.is_some() && (n.contains(['/', '~']) || n.starts_with('.')))
    {
        return Err(invalid_input(format!(
            "player {name}: to save games, names cannot contain / or ~ or start with a dot"
        )));
    }
    let save = opts.save_games.as_deref();
    fs::create_dir_all(out)?;
    let games_path = out.join("games.csv");
    let mut games = if games_path.exists() {
        read_games(&games_path, &names)?
    } else {
        fs::write(&games_path, "tournament,white,black,result,plies\n")?;
        Vec::new()
    };
    let mut done = games.iter().map(|g| g.tournament + 1).max().unwrap_or(0);
    if opts.catch_up {
        let among_entrants: Vec<Played> =
            games.iter().filter(|g| g.white < active && g.black < active).copied().collect();
        let missing = missing_games(active, &among_entrants, done)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("{}: {e}", games_path.display())))?;
        if !missing.is_empty() {
            let start = Instant::now();
            let timed = play_games(entrants, &missing, opts.seed, save)?;
            let played: Vec<Played> = timed.iter().map(|&(g, _)| g).collect();
            append(&games_path, &names, &played)?;
            if opts.game_cpu {
                append_cpu(&out.join("game_cpu.csv"), &names, &timed)?;
            }
            games.extend(played);
            println!(
                "caught up {} games of tournaments 1-{done} for players who joined later in {:.0}s",
                missing.len(),
                start.elapsed().as_secs_f64()
            );
        }
    }
    if done >= opts.tournaments && !games.is_empty() {
        write_reports(out, &names, active, &games, opts)?;
    }
    while done < opts.tournaments {
        let start = Instant::now();
        let timed = play_tournament_timed(entrants, done, opts.seed, save)?;
        let played: Vec<Played> = timed.iter().map(|&(g, _)| g).collect();
        append(&games_path, &names, &played)?;
        if opts.game_cpu {
            append_cpu(&out.join("game_cpu.csv"), &names, &timed)?;
        }
        games.extend(played);
        done += 1;
        if opts.reports_last {
            // The time of the games alone; the tables' after the last.
            let games_in = format!(
                "tournament {done}/{}: {} games in {:.0}s",
                opts.tournaments,
                active * (active - 1),
                start.elapsed().as_secs_f64()
            );
            if done < opts.tournaments {
                println!("{games_in}");
                continue;
            }
            let start = Instant::now();
            let fit = write_reports(out, &names, active, &games, opts)?;
            let mut order: Vec<usize> = (0..active).filter(|&i| fit.rated[i]).collect();
            order.sort_by(|&a, &b| fit.elo[b].total_cmp(&fit.elo[a]));
            let top: Vec<String> = order.iter().take(3).map(|&i| format!("{} {:+.0}", names[i], fit.elo[i])).collect();
            println!("{games_in}, tables in {:.0}s; top: {}", start.elapsed().as_secs_f64(), top.join(", "));
            continue;
        }
        let fit = write_reports(out, &names, active, &games, opts)?;
        let mut order: Vec<usize> = (0..active).filter(|&i| fit.rated[i]).collect();
        order.sort_by(|&a, &b| fit.elo[b].total_cmp(&fit.elo[a]));
        let top: Vec<String> = order.iter().take(3).map(|&i| format!("{} {:+.0}", names[i], fit.elo[i])).collect();
        println!(
            "tournament {done}/{}: {} games in {:.0}s; top: {}",
            opts.tournaments,
            active * (active - 1),
            start.elapsed().as_secs_f64(),
            top.join(", ")
        );
    }
    Ok(())
}

/// Reads a `games.csv` written by `run` for the players `names` (who may
/// include players with no games yet; see `missing_games`).
///
/// hybrid7: a file as `run` writes it is read line by line, without a vector
/// per line and with a faster name index (`read_games_fast`); anything else
/// goes to the original reader, which gives the same games or the same error.
pub fn read_games(path: &Path, names: &[&str]) -> io::Result<Vec<Played>> {
    if let Some(games) = read_games_fast(path, names) {
        return Ok(games);
    }
    read_games_plain(path, names)
}

/// `read_games` for a file of UTF-8 lines of five fields, each line ending
/// in a newline but perhaps the last, with no carriage return, known
/// results, numbers and players: the same games as `read_games_plain`.
/// None for anything else.
fn read_games_fast(path: &Path, names: &[&str]) -> Option<Vec<Played>> {
    use std::io::BufRead;
    let indices: HashMap<&str, usize, FxBuild> = names.iter().enumerate().map(|(i, &n)| (n, i)).collect();
    let file = fs::File::open(path).ok()?;
    let mut reader = io::BufReader::with_capacity(1 << 20, file);
    let mut games = Vec::new();
    let mut buf = Vec::with_capacity(512);
    let mut first = true;
    loop {
        buf.clear();
        if reader.read_until(b'\n', &mut buf).ok()? == 0 {
            break;
        }
        let mut line = &buf[..];
        if line.last() == Some(&b'\n') {
            line = &line[..line.len() - 1];
        }
        if line.contains(&b'\r') {
            return None;
        }
        let line = std::str::from_utf8(line).ok()?;
        if first {
            first = false;
            continue;
        }
        let mut f = line.split(',');
        let (Some(tournament), Some(white), Some(black), Some(result), Some(plies), None) =
            (f.next(), f.next(), f.next(), f.next(), f.next(), f.next())
        else {
            return None;
        };
        let white_points = match result {
            "1-0" => 1.0,
            "0-1" => 0.0,
            "1/2-1/2" => 0.5,
            _ => return None,
        };
        games.push(Played {
            tournament: tournament.parse().ok()?,
            white: *indices.get(white)?,
            black: *indices.get(black)?,
            white_points,
            plies: plies.parse().ok()?,
        });
    }
    Some(games)
}

/// The original `read_games`.
fn read_games_plain(path: &Path, names: &[&str]) -> io::Result<Vec<Played>> {
    let invalid = |msg: String| io::Error::new(io::ErrorKind::InvalidData, format!("{}: {msg}", path.display()));
    let indices: HashMap<&str, usize> = names.iter().enumerate().map(|(i, &n)| (n, i)).collect();
    let index = |name: &str| {
        indices.get(name).copied().ok_or_else(|| invalid(format!("player {name} is not in the tournament")))
    };
    let mut games = Vec::new();
    for line in fs::read_to_string(path)?.lines().skip(1) {
        let fields: Vec<&str> = line.split(',').collect();
        let [tournament, white, black, result, plies] = fields[..] else {
            return Err(invalid(format!("bad line {line:?}")));
        };
        let white_points = match result {
            "1-0" => 1.0,
            "0-1" => 0.0,
            "1/2-1/2" => 0.5,
            _ => return Err(invalid(format!("bad result {result:?}"))),
        };
        games.push(Played {
            tournament: tournament.parse().map_err(|_| invalid(format!("bad line {line:?}")))?,
            white: index(white)?,
            black: index(black)?,
            white_points,
            plies: plies.parse().map_err(|_| invalid(format!("bad line {line:?}")))?,
        });
    }
    Ok(games)
}

/// The expected points of a player rated `diff` Elo above its opponent.
pub fn expected(diff: f64) -> f64 {
    1.0 / (1.0 + 10f64.powf(-diff / 400.0))
}

/// The running Elo ratings after each tournament of `games` (see the module
/// documentation).
pub fn running_elo(n: usize, games: &[Played], k: f64) -> Vec<Vec<f64>> {
    let tournaments = games.iter().map(|g| g.tournament + 1).max().unwrap_or(0) as usize;
    let mut by_tournament = vec![Vec::new(); tournaments];
    for g in games {
        by_tournament[g.tournament as usize].push(g);
    }
    let mut ratings = vec![0.0; n];
    by_tournament
        .iter()
        .map(|games| {
            let mut change = vec![0.0; n];
            for g in games {
                let surprise = g.white_points - expected(ratings[g.white] - ratings[g.black]);
                change[g.white] += k * surprise;
                change[g.black] -= k * surprise;
            }
            ratings.iter_mut().zip(&change).for_each(|(r, c)| *r += c);
            ratings.clone()
        })
        .collect()
}

/// Each entrant's points, and the games between each pair (both colours),
/// with `prior_draws` virtual draws added to every pair.
fn tallies(n: usize, games: &[Played], prior_draws: f64) -> (Vec<f64>, Vec<Vec<f64>>) {
    let mut points = vec![prior_draws / 2.0 * (n - 1) as f64; n];
    let mut count = vec![vec![prior_draws; n]; n];
    for (i, row) in count.iter_mut().enumerate() {
        row[i] = 0.0;
    }
    for g in games {
        points[g.white] += g.white_points;
        points[g.black] += 1.0 - g.white_points;
        count[g.white][g.black] += 1.0;
        count[g.black][g.white] += 1.0;
    }
    (points, count)
}

/// Maximum-likelihood Elo ratings of `games` (Bradley-Terry, draws as half
/// points) with `prior_draws` virtual draws in every pairing, by the MM
/// algorithm (Hunter 2004). Their mean is 0.
pub fn ml_elo(n: usize, games: &[Played], prior_draws: f64) -> Vec<f64> {
    let (points, count) = tallies(n, games, prior_draws);
    // Strengths: the odds of i beating j are gamma[i] / gamma[j].
    let mut gamma = vec![1.0; n];
    for _ in 0..1_000_000 {
        let mut next: Vec<f64> = (0..n)
            .map(|i| points[i] / (0..n).filter(|&j| j != i).map(|j| count[i][j] / (gamma[i] + gamma[j])).sum::<f64>())
            .collect();
        let mean_log = next.iter().map(|g| g.ln()).sum::<f64>() / n as f64;
        next.iter_mut().for_each(|g| *g /= mean_log.exp());
        let change = gamma.iter().zip(&next).map(|(a, b)| (a.ln() - b.ln()).abs()).fold(0.0, f64::max);
        gamma = next;
        if change < 1e-12 {
            break;
        }
    }
    gamma.iter().map(|g| g.ln() * ELO).collect()
}

/// Standard errors of the `ml_elo` ratings `elo`, relative to the field's
/// mean, from the curvature of the likelihood.
pub fn ml_errors(n: usize, games: &[Played], prior_draws: f64, elo: &[f64]) -> Vec<f64> {
    let (_, count) = tallies(n, games, prior_draws);
    mean_relative_errors(&count, elo)
}

/// Standard errors of the ratings `elo` relative to their mean, from the
/// curvature of the likelihood of `count[i][j]` games between i and j.
fn mean_relative_errors(count: &[Vec<f64>], elo: &[f64]) -> Vec<f64> {
    let n = elo.len();
    // The information matrix, in Elo units, plus 1/n everywhere: its inverse
    // minus 1/n is the pseudo-inverse of the information, the covariance of
    // ratings constrained to a fixed mean.
    let mut info = vec![vec![1.0 / n as f64; n]; n];
    for i in 0..n {
        for j in (0..n).filter(|&j| j != i) {
            let p = expected(elo[i] - elo[j]);
            let w = count[i][j] * p * (1.0 - p) / (ELO * ELO);
            info[i][j] -= w;
            info[i][i] += w;
        }
    }
    let inverse = invert(info);
    (0..n).map(|i| (inverse[i][i] - 1.0 / n as f64).max(0.0).sqrt()).collect()
}

/// Maximum-likelihood ratings as the tables show them: Elo relative to the
/// anchor (or to the rated players' mean without one), standard errors
/// relative to the rated players' mean, and whether each player is rated at
/// all (unrated players' Elo and error are NaN).
pub struct Ratings {
    pub elo: Vec<f64>,
    pub errors: Vec<f64>,
    pub rated: Vec<bool>,
}

/// The maximum-likelihood ratings of `games` as `opts` asks for them: with
/// virtual draws (`ml_elo`), or pure (`ml_fit`, or `sparse_fit` if
/// `opts.sparse_fit`, which start from `start` if given, ratings as
/// returned). Standard errors only if `errors`.
pub fn ratings(n: usize, games: &[Played], opts: &Options, start: Option<&[f64]>, errors: bool) -> Ratings {
    if opts.prior_draws > 0.0 {
        let raw = ml_elo(n, games, opts.prior_draws);
        let errors = if errors { ml_errors(n, games, opts.prior_draws, &raw) } else { vec![f64::NAN; n] };
        let zero = opts.anchor.map_or(0.0, |a| raw[a]);
        Ratings { elo: raw.iter().map(|r| r - zero).collect(), errors, rated: vec![true; n] }
    } else if opts.sparse_fit {
        sparse_fit(n, games, opts.anchor, start, errors)
    } else {
        ml_fit(n, games, opts.anchor, start, errors)
    }
}

/// The pure maximum-likelihood Elo ratings of `games` (Bradley-Terry, draws
/// as half points), by Newton's method. They exist for the root (the
/// anchor, or else the player with the most games) and for the players
/// linked to it by results both ways: each has scored against the root and
/// the root against it, directly or through a chain of such players. The
/// others are unrated: a perfect or a zero score, or no such chain, leaves
/// a rating infinite or undetermined; their games do not move the others'
/// ratings. Ratings are relative to the anchor, or to the rated players'
/// mean without one. `start` (ratings as returned, NaN for none) only
/// speeds up the search. With `errors`, the standard errors relative to the
/// rated players' mean, from the information matrix.
pub fn ml_fit(n: usize, games: &[Played], anchor: Option<usize>, start: Option<&[f64]>, errors: bool) -> Ratings {
    let mut fit = Ratings { elo: vec![f64::NAN; n], errors: vec![f64::NAN; n], rated: vec![false; n] };
    if n == 0 {
        return fit;
    }
    // points[i * n + j]: i's points against j, so that i and j played
    // points[i * n + j] + points[j * n + i] games.
    let mut points = vec![0.0; n * n];
    let mut played = vec![0usize; n];
    for g in games {
        points[g.white * n + g.black] += g.white_points;
        points[g.black * n + g.white] += 1.0 - g.white_points;
        played[g.white] += 1;
        played[g.black] += 1;
    }
    let root = anchor.unwrap_or_else(|| (0..n).rev().max_by_key(|&i| played[i]).unwrap());
    // The rated group: the players the root reaches by "scored against"
    // links, which also reach the root.
    let reach = |forward: bool| {
        let mut seen = vec![false; n];
        seen[root] = true;
        let mut stack = vec![root];
        while let Some(i) = stack.pop() {
            for j in 0..n {
                let scored = if forward { points[i * n + j] } else { points[j * n + i] };
                if scored > 0.0 && !seen[j] {
                    seen[j] = true;
                    stack.push(j);
                }
            }
        }
        seen
    };
    let (forward, backward) = (reach(true), reach(false));
    let group: Vec<usize> = (0..n).filter(|&i| forward[i] && backward[i]).collect();
    let m = group.len();
    let pair = |a: usize, b: usize| points[group[a] * n + group[b]];
    // Each group member's variable in the Newton system: all but the root,
    // which stays at 0.
    let root_at = group.iter().position(|&i| i == root).unwrap();
    let var: Vec<Option<usize>> = (0..m).map(|a| (a != root_at).then(|| a - (a > root_at) as usize)).collect();
    let k = m - 1;
    let log_likelihood = |x: &[f64]| {
        let mut sum = 0.0;
        for a in 0..m {
            for b in 0..m {
                let s = pair(a, b);
                if s > 0.0 {
                    sum += s * log_sigmoid(x[a] - x[b]);
                }
            }
        }
        sum
    };
    // In natural units (Elo / ELO).
    let mut x: Vec<f64> = group
        .iter()
        .map(|&i| start.map(|s| s[i]).filter(|v| v.is_finite()).map_or(0.0, |v| v / ELO))
        .collect();
    let x_root = x[root_at];
    x.iter_mut().for_each(|v| *v -= x_root);
    let mut current = log_likelihood(&x);
    for _ in 0..500 {
        let mut gradient = vec![0.0; k];
        let mut information = vec![0.0; k * k];
        for a in 0..m {
            for b in a + 1..m {
                let games = pair(a, b) + pair(b, a);
                if games == 0.0 {
                    continue;
                }
                let p = sigmoid(x[a] - x[b]);
                let (g, w) = (pair(a, b) - games * p, games * p * (1.0 - p));
                if let Some(i) = var[a] {
                    gradient[i] += g;
                    information[i * k + i] += w;
                }
                if let Some(j) = var[b] {
                    gradient[j] -= g;
                    information[j * k + j] += w;
                }
                if let (Some(i), Some(j)) = (var[a], var[b]) {
                    information[i * k + j] -= w;
                    information[j * k + i] -= w;
                }
            }
        }
        let Some(step) = solve_spd(information, k, gradient) else { break };
        // The Newton step, halved until the likelihood does not fall.
        let mut scale = 1.0;
        let moved = loop {
            let trial: Vec<f64> = (0..m).map(|a| x[a] + var[a].map_or(0.0, |i| scale * step[i])).collect();
            let value = log_likelihood(&trial);
            if value >= current - 1e-12 * current.abs() {
                x = trial;
                current = value;
                break step.iter().fold(0.0f64, |c, s| c.max((scale * s).abs()));
            }
            scale /= 2.0;
            if scale < 1e-12 {
                break 0.0;
            }
        };
        if moved < 1e-10 {
            break;
        }
    }
    let mean = if anchor.is_some() { 0.0 } else { x.iter().sum::<f64>() / m as f64 };
    for (a, &i) in group.iter().enumerate() {
        fit.elo[i] = (x[a] - mean) * ELO;
        fit.rated[i] = true;
    }
    if errors {
        let count: Vec<Vec<f64>> =
            (0..m).map(|a| (0..m).map(|b| if a == b { 0.0 } else { pair(a, b) + pair(b, a) }).collect()).collect();
        let elo: Vec<f64> = x.iter().map(|v| v * ELO).collect();
        for (a, e) in mean_relative_errors(&count, &elo).into_iter().enumerate() {
            fit.errors[group[a]] = e;
        }
    }
    fit
}

/// `ml_fit`'s ratings and standard errors (the same, to rounding) by block
/// elimination instead of dense matrices over all players, for big leagues.
/// The information matrix is a sum of blocks, one per field: the set of
/// players who played a tournament together (a set inside another set
/// joins it). A player in one field only is eliminated within its block;
/// only the players in several fields meet in a dense matrix. The Newton
/// steps solve with this factorization, and the standard errors come from
/// the diagonal of its inverse, block by block, in time that grows with the
/// games and the shared players rather than with the cube of all players.
pub fn sparse_fit(n: usize, games: &[Played], anchor: Option<usize>, start: Option<&[f64]>, errors: bool) -> Ratings {
    let mut fit = Ratings { elo: vec![f64::NAN; n], errors: vec![f64::NAN; n], rated: vec![false; n] };
    if n == 0 {
        return fit;
    }
    // Each pair's points both ways and the first tournament it played in,
    // keyed by (i, j) with i < j; a player's games against itself apart.
    let mut tally: HashMap<(usize, usize), (f64, f64, u64), FxBuild> = HashMap::default();
    let mut alone = vec![0.0; n];
    let mut played = vec![0usize; n];
    for g in games {
        played[g.white] += 1;
        played[g.black] += 1;
        if g.white == g.black {
            alone[g.white] += g.white_points;
            alone[g.white] += 1.0 - g.white_points;
            continue;
        }
        let (i, j, p) =
            if g.white < g.black { (g.white, g.black, g.white_points) } else { (g.black, g.white, 1.0 - g.white_points) };
        let entry = tally.entry((i, j)).or_insert((0.0, 0.0, g.tournament));
        entry.0 += p;
        entry.1 += 1.0 - p;
    }
    let root = anchor.unwrap_or_else(|| (0..n).rev().max_by_key(|&i| played[i]).unwrap());
    // The rated group, as in `ml_fit`: beat[i] are the players i scored
    // against, beaten[i] those who scored against i.
    let (mut beat, mut beaten) = (vec![Vec::new(); n], vec![Vec::new(); n]);
    for (&(i, j), &(p, q, _)) in &tally {
        if p > 0.0 {
            beat[i].push(j);
            beaten[j].push(i);
        }
        if q > 0.0 {
            beat[j].push(i);
            beaten[i].push(j);
        }
    }
    let reach = |links: &[Vec<usize>]| {
        let mut seen = vec![false; n];
        seen[root] = true;
        let mut stack = vec![root];
        while let Some(i) = stack.pop() {
            for &j in &links[i] {
                if !seen[j] {
                    seen[j] = true;
                    stack.push(j);
                }
            }
        }
        seen
    };
    let (forward, backward) = (reach(&beat), reach(&beaten));
    let group: Vec<usize> = (0..n).filter(|&i| forward[i] && backward[i]).collect();
    let m = group.len();
    let mut at = vec![usize::MAX; n];
    for (a, &i) in group.iter().enumerate() {
        at[i] = a;
    }
    let root_at = at[root];
    let var: Vec<Option<usize>> = (0..m).map(|a| (a != root_at).then(|| a - (a > root_at) as usize)).collect();
    let k = m - 1;
    // The pairs of the group that played, in `ml_fit`'s order: (a, b, a's
    // points against b, b's against a, a tournament they played in), a < b.
    let mut pairs: Vec<(usize, usize, f64, f64, u64)> = tally
        .iter()
        .filter(|&(&(i, j), _)| at[i] != usize::MAX && at[j] != usize::MAX)
        .map(|(&(i, j), &(p, q, t))| (at[i], at[j], p, q, t))
        .collect();
    drop(tally);
    pairs.sort_unstable_by_key(|&(a, b, ..)| (a, b));
    // The log-likelihood's terms in `ml_fit`'s order, so it sums the same.
    let mut terms: Vec<(usize, usize, f64)> = Vec::with_capacity(2 * pairs.len());
    for &(a, b, p, q, _) in &pairs {
        if p > 0.0 {
            terms.push((a, b, p));
        }
        if q > 0.0 {
            terms.push((b, a, q));
        }
    }
    for (a, &i) in group.iter().enumerate() {
        if alone[i] > 0.0 {
            terms.push((a, a, alone[i]));
        }
    }
    terms.sort_unstable_by_key(|&(a, b, _)| (a, b));
    let log_likelihood = |x: &[f64]| {
        let mut sum = 0.0;
        for &(a, b, s) in &terms {
            sum += s * log_sigmoid(x[a] - x[b]);
        }
        sum
    };
    let blocks = Blocks::new(games, &at, &var, k, &pairs);
    // In natural units (Elo / ELO).
    let mut x: Vec<f64> = group
        .iter()
        .map(|&i| start.map(|s| s[i]).filter(|v| v.is_finite()).map_or(0.0, |v| v / ELO))
        .collect();
    let x_root = x[root_at];
    x.iter_mut().for_each(|v| *v -= x_root);
    let mut current = log_likelihood(&x);
    let mut weight = vec![0.0; pairs.len()];
    for _ in 0..500 {
        let mut gradient = vec![0.0; k];
        for (w, &(a, b, p, q, _)) in weight.iter_mut().zip(&pairs) {
            let games = p + q;
            let e = sigmoid(x[a] - x[b]);
            let g = p - games * e;
            *w = games * e * (1.0 - e);
            if let Some(i) = var[a] {
                gradient[i] += g;
            }
            if let Some(j) = var[b] {
                gradient[j] -= g;
            }
        }
        let Some(factor) = blocks.factor(&weight) else { break };
        let step = blocks.solve(&factor, &gradient);
        // The Newton step, halved until the likelihood does not fall.
        let mut scale = 1.0;
        let moved = loop {
            let trial: Vec<f64> = (0..m).map(|a| x[a] + var[a].map_or(0.0, |i| scale * step[i])).collect();
            let value = log_likelihood(&trial);
            if value >= current - 1e-12 * current.abs() {
                x = trial;
                current = value;
                break step.iter().fold(0.0f64, |c, s| c.max((scale * s).abs()));
            }
            scale /= 2.0;
            if scale < 1e-12 {
                break 0.0;
            }
        };
        if moved < 1e-10 {
            break;
        }
    }
    let mean = if anchor.is_some() { 0.0 } else { x.iter().sum::<f64>() / m as f64 };
    for (a, &i) in group.iter().enumerate() {
        fit.elo[i] = (x[a] - mean) * ELO;
        fit.rated[i] = true;
    }
    if errors {
        // The covariance of the ratings with the root pinned is the inverse
        // H⁻¹ of the free variables' information H (the root's row and
        // column 0). Relative to the group's mean: Var(x_i - mean) = H⁻¹_ii
        // - 2 u_i / m + s / m², u = H⁻¹ 1 (0 for the root), s = Σ u.
        for (w, &(a, b, p, q, _)) in weight.iter_mut().zip(&pairs) {
            let e = sigmoid(x[a] - x[b]);
            *w = (p + q) * e * (1.0 - e);
        }
        let variances = blocks.factor(&weight).map(|factor| {
            let u = blocks.solve(&factor, &vec![1.0; k]);
            let s = u.iter().sum::<f64>();
            let diagonal = blocks.inverse_diagonal(&factor);
            let mm = m as f64;
            (0..m)
                .map(|a| var[a].map_or(s / (mm * mm), |i| diagonal[i] - 2.0 * u[i] / mm + s / (mm * mm)))
                .collect::<Vec<f64>>()
        });
        match variances {
            Some(v) => {
                for (a, &i) in group.iter().enumerate() {
                    fit.errors[i] = v[a].max(0.0).sqrt() * ELO;
                }
            }
            // Not positive definite in floating point: as `ml_fit` does.
            None => {
                let mut count = vec![vec![0.0; m]; m];
                for &(a, b, p, q, _) in &pairs {
                    count[a][b] = p + q;
                    count[b][a] = p + q;
                }
                let elo: Vec<f64> = x.iter().map(|v| v * ELO).collect();
                for (a, e) in mean_relative_errors(&count, &elo).into_iter().enumerate() {
                    fit.errors[group[a]] = e;
                }
            }
        }
    }
    fit
}

/// `sparse_fit`'s blocks: the information matrix of the free variables
/// (all rated players but the root) as a sum over fields.
struct Blocks {
    blocks: Vec<Block>,
    /// The variables in more than one block, in order, and each variable's
    /// position among them (usize::MAX for the others).
    shared: Vec<usize>,
    shared_at: Vec<usize>,
    k: usize,
}

struct Block {
    /// Its variables: first the `private` ones, in no other block, then
    /// shared ones, each in order.
    vars: Vec<usize>,
    private: usize,
    /// The pairs it holds, each pair in exactly one block: (index in the
    /// pairs, the two players' positions in `vars`, None for the root).
    pairs: Vec<(usize, Option<usize>, Option<usize>)>,
}

/// A factorization of the blocks' sum: per block, its matrix (row by row)
/// with the Cholesky factor of its private part in its first `private`
/// columns (L11 above, L21 below), and the Cholesky factor of the rest, the
/// shared variables' Schur complement (lower, row by row).
struct Factor {
    blocks: Vec<Vec<f64>>,
    rest: Vec<f64>,
}

impl Blocks {
    /// The fields of `games` (each tournament's set of free variables, a set
    /// inside another joining it), and the pairs each holds.
    fn new(games: &[Played], at: &[usize], var: &[Option<usize>], k: usize, pairs: &[(usize, usize, f64, f64, u64)]) -> Blocks {
        let tournaments = games.iter().map(|g| g.tournament as usize + 1).max().unwrap_or(0);
        let mut sets: Vec<Vec<usize>> = vec![Vec::new(); tournaments];
        for g in games {
            let (a, b) = (at[g.white], at[g.black]);
            if a == usize::MAX || b == usize::MAX || a == b {
                continue;
            }
            sets[g.tournament as usize].extend(var[a].into_iter().chain(var[b]));
        }
        for set in &mut sets {
            set.sort_unstable();
            set.dedup();
        }
        // The largest sets first; each joins the first field that holds it.
        let mut order: Vec<usize> = (0..tournaments).filter(|&t| !sets[t].is_empty()).collect();
        order.sort_by(|&s, &t| sets[t].len().cmp(&sets[s].len()).then_with(|| sets[s].cmp(&sets[t])));
        let mut field_of = vec![usize::MAX; tournaments];
        let mut fields: Vec<Vec<usize>> = Vec::new();
        let mut holds: Vec<Vec<bool>> = Vec::new();
        for &t in &order {
            let set = &sets[t];
            field_of[t] = match (0..fields.len()).find(|&f| set.iter().all(|&v| holds[f][v])) {
                Some(f) => f,
                None => {
                    let mut bits = vec![false; k];
                    set.iter().for_each(|&v| bits[v] = true);
                    holds.push(bits);
                    fields.push(set.clone());
                    fields.len() - 1
                }
            };
        }
        drop(holds);
        let mut count = vec![0usize; k];
        for field in &fields {
            field.iter().for_each(|&v| count[v] += 1);
        }
        let shared: Vec<usize> = (0..k).filter(|&v| count[v] > 1).collect();
        let mut shared_at = vec![usize::MAX; k];
        for (r, &v) in shared.iter().enumerate() {
            shared_at[v] = r;
        }
        let mut held: Vec<Vec<usize>> = vec![Vec::new(); fields.len()];
        for (p, &(.., t)) in pairs.iter().enumerate() {
            held[field_of[t as usize]].push(p);
        }
        let mut local = vec![usize::MAX; k];
        let blocks = fields
            .iter()
            .zip(held)
            .map(|(field, held)| {
                let vars: Vec<usize> = field
                    .iter()
                    .copied()
                    .filter(|&v| count[v] == 1)
                    .chain(field.iter().copied().filter(|&v| count[v] > 1))
                    .collect();
                for (l, &v) in vars.iter().enumerate() {
                    local[v] = l;
                }
                let pairs =
                    held.iter().map(|&p| (p, var[pairs[p].0].map(|v| local[v]), var[pairs[p].1].map(|v| local[v]))).collect();
                Block { private: field.iter().filter(|&&v| count[v] == 1).count(), vars, pairs }
            })
            .collect();
        Blocks { blocks, shared, shared_at, k }
    }

    /// Factors the matrix whose pair p has weight `weight[p]` (games × p ×
    /// (1 - p)); None if it is not positive definite.
    fn factor(&self, weight: &[f64]) -> Option<Factor> {
        let s = self.shared.len();
        let mut rest = vec![0.0; s * s];
        let mut blocks = Vec::with_capacity(self.blocks.len());
        for block in &self.blocks {
            let (t, p) = (block.vars.len(), block.private);
            let mut a = vec![0.0; t * t];
            for &(pair, i, j) in &block.pairs {
                let w = weight[pair];
                if let Some(i) = i {
                    a[i * t + i] += w;
                }
                if let Some(j) = j {
                    a[j * t + j] += w;
                }
                if let (Some(i), Some(j)) = (i, j) {
                    a[i * t + j] -= w;
                    a[j * t + i] -= w;
                }
            }
            for j in 0..p {
                let d = a[j * t + j] - (0..j).map(|l| a[j * t + l] * a[j * t + l]).sum::<f64>();
                if !(d > 0.0) {
                    return None;
                }
                let d = d.sqrt();
                a[j * t + j] = d;
                for i in j + 1..t {
                    let v = a[i * t + j] - (0..j).map(|l| a[i * t + l] * a[j * t + l]).sum::<f64>();
                    a[i * t + j] = v / d;
                }
            }
            for i in p..t {
                let ri = self.shared_at[block.vars[i]];
                for j in p..=i {
                    let rj = self.shared_at[block.vars[j]];
                    let v = a[i * t + j] - (0..p).map(|l| a[i * t + l] * a[j * t + l]).sum::<f64>();
                    rest[ri.max(rj) * s + ri.min(rj)] += v;
                }
            }
            blocks.push(a);
        }
        for j in 0..s {
            let d = rest[j * s + j] - (0..j).map(|l| rest[j * s + l] * rest[j * s + l]).sum::<f64>();
            if !(d > 0.0) {
                return None;
            }
            let d = d.sqrt();
            rest[j * s + j] = d;
            for i in j + 1..s {
                let v = rest[i * s + j] - (0..j).map(|l| rest[i * s + l] * rest[j * s + l]).sum::<f64>();
                rest[i * s + j] = v / d;
            }
        }
        Some(Factor { blocks, rest })
    }

    /// Solves `H x = b` with the factorization.
    fn solve(&self, factor: &Factor, b: &[f64]) -> Vec<f64> {
        let s = self.shared.len();
        let mut x = vec![0.0; self.k];
        let mut z: Vec<f64> = self.shared.iter().map(|&v| b[v]).collect();
        let mut ys = Vec::with_capacity(self.blocks.len());
        for (block, a) in self.blocks.iter().zip(&factor.blocks) {
            let (t, p) = (block.vars.len(), block.private);
            let mut y = vec![0.0; p];
            for i in 0..p {
                y[i] = (b[block.vars[i]] - (0..i).map(|l| a[i * t + l] * y[l]).sum::<f64>()) / a[i * t + i];
            }
            for i in p..t {
                z[self.shared_at[block.vars[i]]] -= (0..p).map(|l| a[i * t + l] * y[l]).sum::<f64>();
            }
            ys.push(y);
        }
        let r = &factor.rest;
        for i in 0..s {
            z[i] = (z[i] - (0..i).map(|l| r[i * s + l] * z[l]).sum::<f64>()) / r[i * s + i];
        }
        for i in (0..s).rev() {
            z[i] = (z[i] - (i + 1..s).map(|l| r[l * s + i] * z[l]).sum::<f64>()) / r[i * s + i];
        }
        for (&v, &zv) in self.shared.iter().zip(&z) {
            x[v] = zv;
        }
        for ((block, a), mut y) in self.blocks.iter().zip(&factor.blocks).zip(ys) {
            let (t, p) = (block.vars.len(), block.private);
            for l in 0..p {
                y[l] -= (p..t).map(|i| a[i * t + l] * x[block.vars[i]]).sum::<f64>();
            }
            for i in (0..p).rev() {
                x[block.vars[i]] = (y[i] - (i + 1..p).map(|l| a[l * t + i] * x[block.vars[l]]).sum::<f64>()) / a[i * t + i];
            }
        }
        x
    }

    /// The diagonal of `H⁻¹`. The shared variables' covariance is the rest's
    /// inverse R⁻¹; a block's private variables' is A⁻¹ + Z R⁻¹ Zᵀ, with A
    /// = L11 L11ᵀ its private part and Z = L11⁻ᵀ L21ᵀ.
    fn inverse_diagonal(&self, factor: &Factor) -> Vec<f64> {
        let s = self.shared.len();
        let r = &factor.rest;
        // Column j of R's factor's inverse, from row j on (it is lower).
        let columns: Vec<Vec<f64>> = (0..s)
            .map(|j| {
                let mut c = vec![0.0; s - j];
                for i in j..s {
                    let v = if i == j { 1.0 } else { 0.0 } - (j..i).map(|l| r[i * s + l] * c[l - j]).sum::<f64>();
                    c[i - j] = v / r[i * s + i];
                }
                c
            })
            .collect();
        // R⁻¹ = L⁻ᵀ L⁻¹: entry (a, b) is the dot product of columns a and b of L⁻¹.
        let mut inverse = vec![0.0; s * s];
        for a in 0..s {
            for b in 0..=a {
                let v = columns[a].iter().zip(&columns[b][a - b..]).map(|(x, y)| x * y).sum::<f64>();
                inverse[a * s + b] = v;
                inverse[b * s + a] = v;
            }
        }
        drop(columns);
        let mut diagonal = vec![0.0; self.k];
        for (r, &v) in self.shared.iter().enumerate() {
            diagonal[v] = inverse[r * s + r];
        }
        for (block, a) in self.blocks.iter().zip(&factor.blocks) {
            let (t, p) = (block.vars.len(), block.private);
            let q = t - p;
            let at: Vec<usize> = block.vars[p..].iter().map(|&v| self.shared_at[v]).collect();
            let sigma: Vec<f64> = (0..q * q).map(|c| inverse[at[c / q] * s + at[c % q]]).collect();
            // Z, p × q: column c solves L11ᵀ z = (row p + c of L21).
            let mut z = vec![0.0; p * q];
            for c in 0..q {
                for i in (0..p).rev() {
                    let v = a[(p + c) * t + i] - (i + 1..p).map(|l| a[l * t + i] * z[l * q + c]).sum::<f64>();
                    z[i * q + c] = v / a[i * t + i];
                }
            }
            // diag(A⁻¹)_i: the squares of column i of L11⁻¹.
            let mut own = vec![0.0; p];
            let mut col = vec![0.0; p];
            for j in 0..p {
                for i in j..p {
                    let v = if i == j { 1.0 } else { 0.0 } - (j..i).map(|l| a[i * t + l] * col[l]).sum::<f64>();
                    col[i] = v / a[i * t + i];
                }
                own[j] = col[j..p].iter().map(|v| v * v).sum::<f64>();
            }
            for i in 0..p {
                let zi = &z[i * q..(i + 1) * q];
                let quadratic: f64 =
                    (0..q).map(|c| zi[c] * (0..q).map(|d| sigma[c * q + d] * zi[d]).sum::<f64>()).sum();
                diagonal[block.vars[i]] = own[i] + quadratic;
            }
        }
        diagonal
    }
}

fn sigmoid(z: f64) -> f64 {
    1.0 / (1.0 + (-z).exp())
}

/// ln(sigmoid(z)), without overflow.
fn log_sigmoid(z: f64) -> f64 {
    if z >= 0.0 { -(-z).exp().ln_1p() } else { z - z.exp().ln_1p() }
}

/// Solves `a x = b` for a symmetric positive definite `a` (`k` × `k`, row by
/// row) by its Cholesky decomposition; None if `a` is not positive definite.
fn solve_spd(mut a: Vec<f64>, k: usize, mut b: Vec<f64>) -> Option<Vec<f64>> {
    for j in 0..k {
        let d = a[j * k + j] - (0..j).map(|l| a[j * k + l] * a[j * k + l]).sum::<f64>();
        if !(d > 0.0) {
            return None;
        }
        let d = d.sqrt();
        a[j * k + j] = d;
        for i in j + 1..k {
            let s = a[i * k + j] - (0..j).map(|l| a[i * k + l] * a[j * k + l]).sum::<f64>();
            a[i * k + j] = s / d;
        }
    }
    for i in 0..k {
        let s = b[i] - (0..i).map(|l| a[i * k + l] * b[l]).sum::<f64>();
        b[i] = s / a[i * k + i];
    }
    for i in (0..k).rev() {
        let s = b[i] - (i + 1..k).map(|l| a[l * k + i] * b[l]).sum::<f64>();
        b[i] = s / a[i * k + i];
    }
    Some(b)
}

/// The inverse of a non-singular square matrix (Gauss-Jordan with partial
/// pivoting).
fn invert(mut a: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
    let n = a.len();
    let mut inv: Vec<Vec<f64>> = (0..n).map(|i| (0..n).map(|j| (i == j) as u8 as f64).collect()).collect();
    for col in 0..n {
        let pivot = (col..n).max_by(|&x, &y| a[x][col].abs().total_cmp(&a[y][col].abs())).unwrap();
        a.swap(col, pivot);
        inv.swap(col, pivot);
        let scale = a[col][col];
        a[col].iter_mut().for_each(|x| *x /= scale);
        inv[col].iter_mut().for_each(|x| *x /= scale);
        for row in (0..n).filter(|&r| r != col) {
            let factor = a[row][col];
            if factor != 0.0 {
                for k in 0..n {
                    a[row][k] -= factor * a[col][k];
                    inv[row][k] -= factor * inv[col][k];
                }
            }
        }
    }
    inv
}

/// Wins, losses and draws of each entrant against each other, both colours.
pub fn head_to_head(n: usize, games: &[Played]) -> Vec<Vec<[u32; 3]>> {
    let mut table = vec![vec![[0u32; 3]; n]; n];
    for g in games {
        let (w, b) = (g.white, g.black);
        let (white_result, black_result) = match g.white_points {
            p if p == 1.0 => (0, 1),
            p if p == 0.0 => (1, 0),
            _ => (2, 2),
        };
        table[w][b][white_result] += 1;
        table[b][w][black_result] += 1;
    }
    table
}

/// Writes `ratings.md`, `ratings.csv`, `head_to_head.md`,
/// `head_to_head.csv` and `elo_history.csv` into `out`. Players from
/// `active` on in `names` are retired. Returns the maximum-likelihood
/// ratings as shown (relative to the anchor).
///
/// hybrid7: the same files as before, faster. The full fit runs on a thread
/// of its own while this one fits the history's new prefixes (the two are
/// independent and each is computed as before). The head-to-head counts are
/// kept for the pairs that played only, not for all pairs of players. And
/// `elo_history.csv` is appended to when it is as the last call left it (see
/// `history_tail`), instead of being read and rewritten whole.
fn write_reports(out: &Path, names: &[&str], active: usize, games: &[Played], opts: &Options) -> io::Result<Ratings> {
    let times = std::env::var_os("CHESSAI_REPORT_TIMES").is_some();
    let clock = Instant::now();
    let lap = |what: &str| {
        if times {
            eprintln!("report: {what} at {:.3}s", clock.elapsed().as_secs_f64());
        }
    };
    let n = names.len();
    let tournaments = games.iter().map(|g| g.tournament + 1).max().unwrap_or(0);
    let zero = |ratings: &[f64]| opts.anchor.map_or(0.0, |a| ratings[a]);
    let anchor = opts.anchor.map_or("the field's mean".to_string(), |a| names[a].to_string());
    let (fit, history) = std::thread::scope(|scope| {
        let full = scope.spawn(|| ratings(n, games, opts, None, true));
        let history = rating_history(out, names, games, opts, tournaments, &anchor, &lap);
        lap("history done");
        let fit = full.join().unwrap_or_else(|e| std::panic::resume_unwind(e));
        lap("full fit joined");
        (fit, history)
    });
    let last = &history.last;
    let running: Vec<f64> = last.iter().map(|r| r - zero(last)).collect();
    // Each player's wins, losses and draws (the sums of its head-to-head
    // row), and the head-to-head counts of the pairs that played, both ways.
    let mut totals = vec![[0u32; 3]; n];
    let mut h2h: HashMap<(usize, usize), [u32; 3], FxBuild> = HashMap::default();
    for g in games {
        let (w, b) = (g.white, g.black);
        let (white_result, black_result) = match g.white_points {
            p if p == 1.0 => (0, 1),
            p if p == 0.0 => (1, 0),
            _ => (2, 2),
        };
        totals[w][white_result] += 1;
        totals[b][black_result] += 1;
        h2h.entry((w, b)).or_default()[white_result] += 1;
        h2h.entry((b, w)).or_default()[black_result] += 1;
    }
    let cell = |i: usize, j: usize| h2h.get(&(i, j)).copied().unwrap_or([0; 3]);
    let mut plies = vec![0usize; n];
    for g in games {
        plies[g.white] += g.plies;
        plies[g.black] += g.plies;
    }
    lap("tables counted");
    // The rated players by rating, then the unrated in the players' order.
    let mut order: Vec<usize> = (0..n).filter(|&i| fit.rated[i]).collect();
    order.sort_by(|&a, &b| fit.elo[b].total_cmp(&fit.elo[a]));
    let unrated: Vec<usize> = (0..n).filter(|&i| !fit.rated[i]).collect();
    let label = |i: usize| if i < active { names[i].to_string() } else { format!("{} (retired)", names[i]) };
    let record = |i: usize| {
        let [wins, losses, draws] = totals[i];
        let played = (wins + losses + draws).max(1) as f64;
        let score = 100.0 * (wins as f64 + draws as f64 / 2.0) / played;
        (wins, losses, draws, score, plies[i] as f64 / played)
    };

    // Rating list.
    let fit_text = if opts.prior_draws > 0.0 {
        format!(
            "Elo: maximum likelihood over all games, relative to {anchor}; ± is the 95% interval relative to \
             the field's mean."
        )
    } else {
        format!(
            "Elo: maximum likelihood over all games, with no prior, relative to {anchor}; ± is the 95% \
             interval relative to the rated players' mean."
        )
    };
    let mut md = format!(
        "# Ratings after {tournaments} tournaments ({} games)\n\n\
         {fit_text} Running: classic Elo updated after every tournament (K = {} per game), relative to \
         {anchor}.{}\n\n\
         | # | Player | Elo | ± | Running | Games | Score | Wins | Losses | Draws | Plies |\n\
         |---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n",
        games.len(),
        opts.k,
        if n > active { " Retired players play no more games, but theirs count." } else { "" }
    );
    let mut csv = String::from("rank,player,elo,ci95,running,games,score,wins,losses,draws,plies");
    csv.push_str(if n > active { ",retired\n" } else { "\n" });
    for (rank, &i) in order.iter().enumerate() {
        let (wins, losses, draws, score, plies) = record(i);
        let ci = 1.96 * fit.errors[i];
        writeln!(
            md,
            "| {} | {} | {:+.0} | {ci:.0} | {:+.0} | {} | {score:.1}% | {wins} | {losses} | {draws} | {plies:.0} |",
            rank + 1,
            label(i),
            fit.elo[i],
            running[i],
            wins + losses + draws
        )
        .unwrap();
        write!(
            csv,
            "{},{},{:.1},{ci:.1},{:.1},{},{score:.2},{wins},{losses},{draws},{plies:.1}",
            rank + 1,
            names[i],
            fit.elo[i],
            running[i],
            wins + losses + draws
        )
        .unwrap();
        csv.push_str(if n <= active { "\n" } else if i < active { ",no\n" } else { ",yes\n" });
    }
    if !unrated.is_empty() {
        md.push_str(
            "\n## Unrated\n\nTheir maximum-likelihood rating does not exist (yet): a perfect or a zero score, or \
             no results linking them both ways to the rated players.\n\n\
             | Player | Games | Score | Wins | Losses | Draws | Plies |\n|---|---:|---:|---:|---:|---:|---:|\n",
        );
        for &i in &unrated {
            let (wins, losses, draws, score, plies) = record(i);
            let games = wins + losses + draws;
            writeln!(md, "| {} | {games} | {score:.1}% | {wins} | {losses} | {draws} | {plies:.0} |", label(i)).unwrap();
        }
    }
    fs::write(out.join("ratings.md"), md)?;
    fs::write(out.join("ratings.csv"), csv)?;

    // Head-to-head: row's score against each column, players in rating order.
    let everyone: Vec<usize> = order.iter().chain(&unrated).copied().collect();
    let shown: Vec<usize> = everyone.iter().copied().filter(|&i| i < active).collect();
    let mut md = format!("# Head to head after {tournaments} tournaments\n\n");
    if opts.catch_up && n == active {
        write!(
            md,
            "Each cell is the row player's score (%) against the column player, over {} games per pairing. \
             Players in rating order; columns by rank.",
            2 * tournaments
        )
        .unwrap();
    } else {
        md.push_str(
            "Each cell is the row player's score (%) against the column player, over their games together. \
             Players in rating order, the unrated last; columns by rank.",
        );
        if n > active {
            md.push_str(" Retired players are left out; head_to_head.csv has their games too.");
        }
    }
    md.push_str("\n\n| # | Player |");
    for rank in 1..=shown.len() {
        write!(md, " {rank} |").unwrap();
    }
    md.push_str("\n|---:|---|");
    md.push_str(&"---:|".repeat(shown.len()));
    md.push('\n');
    for (rank, &i) in shown.iter().enumerate() {
        write!(md, "| {} | {} |", rank + 1, names[i]).unwrap();
        for &j in &shown {
            let [w, l, d] = cell(i, j);
            let games = w + l + d;
            if i == j || games == 0 {
                md.push_str(" · |");
                continue;
            }
            write!(md, " {:.0} |", 100.0 * (w as f64 + d as f64 / 2.0) / games as f64).unwrap();
        }
        md.push('\n');
    }
    // The pairs that played, in the order of the rows (by the row player's
    // place in `everyone`, then the column player's).
    let mut place = vec![usize::MAX; n];
    for (r, &i) in everyone.iter().enumerate() {
        place[i] = r;
    }
    let mut met: Vec<(usize, usize, [u32; 3])> =
        h2h.iter().filter(|&(&(i, j), _)| i != j).map(|(&(i, j), &c)| (place[i], place[j], c)).collect();
    drop(h2h);
    met.sort_unstable_by_key(|&(a, b, _)| (a, b));
    let mut csv = String::from("player,opponent,wins,losses,draws,score,elo_difference\n");
    for (a, b, [w, l, d]) in met {
        let (i, j) = (everyone[a], everyone[b]);
        let games = w + l + d;
        if games == 0 {
            continue;
        }
        let score = (w as f64 + d as f64 / 2.0) / games as f64;
        let diff = if score <= 0.0 || score >= 1.0 { f64::NAN } else { -400.0 * (1.0 / score - 1.0).log10() };
        writeln!(csv, "{},{},{w},{l},{d},{score:.4},{diff:.1}", names[i], names[j]).unwrap();
    }
    fs::write(out.join("head_to_head.md"), md)?;
    fs::write(out.join("head_to_head.csv"), csv)?;
    lap("tables written");

    if history.append {
        if !history.rows.is_empty() {
            fs::OpenOptions::new().append(true).open(out.join("elo_history.csv"))?.write_all(history.rows.as_bytes())?;
        }
    } else {
        fs::write(out.join("elo_history.csv"), history.rows)?;
    }
    if let Some(key) = history.key {
        fs::write(out.join("elo_history.key"), key)?;
    }
    lap("history written");
    Ok(fit)
}

/// What `write_reports` writes of the rating history, and the running Elo.
struct History {
    /// The running Elo after the last tournament (0 for all if none).
    last: Vec<f64>,
    /// The rows to append to `elo_history.csv` (`append`), or its whole text.
    rows: String,
    append: bool,
    /// The text of `elo_history.key` (pure maximum likelihood only).
    key: Option<String>,
}

/// Both ratings after every tournament, for the players rated by then.
/// Refitting every prefix takes long in a big field, so with pure maximum
/// likelihood the rows of an earlier call are kept for the prefixes whose
/// games are unchanged: elo_history.key records the fit, then each
/// tournament's number of games. When the file can be appended to (see
/// `history_tail`), only the new rows are made; otherwise the file is read,
/// its rows kept and the whole text made, as before.
fn rating_history(
    out: &Path,
    names: &[&str],
    games: &[Played],
    opts: &Options,
    tournaments: u64,
    anchor: &str,
    lap: &dyn Fn(&str),
) -> History {
    let n = names.len();
    let zero = |ratings: &[f64]| opts.anchor.map_or(0.0, |a| ratings[a]);
    let mut counts = vec![0usize; tournaments as usize];
    for g in games {
        counts[g.tournament as usize] += 1;
    }
    let fit_key = format!("prior_draws {} anchor {anchor}", opts.prior_draws);
    let mut csv = String::new();
    let (mut from, mut previous, mut append) = (0, None, false);
    let tail = if opts.prior_draws == 0.0 { history_tail(out, &fit_key, &counts) } else { None };
    lap("history tail read");
    if let Some((recorded, rows)) = tail {
        let index: HashMap<&str, usize, FxBuild> = names.iter().enumerate().map(|(i, &n)| (n, i)).collect();
        let mut last = vec![f64::NAN; n];
        for (player, elo) in &rows {
            if let (Some(&i), Ok(elo)) = (index.get(player.as_str()), elo.parse()) {
                last[i] = elo;
            }
        }
        (from, previous, append) = (recorded, Some(last), true);
    } else {
        csv.push_str("tournament,player,elo,running\n");
        if opts.prior_draws == 0.0
            && let (Ok(key), Ok(old)) =
                (fs::read_to_string(out.join("elo_history.key")), fs::read_to_string(out.join("elo_history.csv")))
            && key.lines().next() == Some(fit_key.as_str())
        {
            let stored = key.lines().skip(1).map(|l| l.parse::<usize>().ok());
            from = stored.zip(&counts).take_while(|(s, c)| *s == Some(**c)).count();
            let index: HashMap<&str, usize> = names.iter().enumerate().map(|(i, &n)| (n, i)).collect();
            let mut last = vec![f64::NAN; n];
            for row in old.lines().skip(1) {
                let fields: Vec<&str> = row.split(',').collect();
                let t: usize = fields[0].parse().unwrap_or(usize::MAX);
                if t <= from {
                    csv.push_str(row);
                    csv.push('\n');
                }
                if t == from && let (Some(&i), Ok(elo)) = (index.get(fields[1]), fields[2].parse()) {
                    last[i] = elo;
                }
            }
            previous = (from > 0).then_some(last);
        }
        lap("history read whole");
    }
    let (kept, last) = running_elo_from(n, games, opts.k, from);
    for (s, running) in kept.iter().enumerate() {
        let t = from + s;
        let upto: Vec<Played> = games.iter().filter(|g| g.tournament <= t as u64).copied().collect();
        let then = ratings(n, &upto, opts, previous.as_deref(), false);
        for i in (0..n).filter(|&i| then.rated[i]) {
            writeln!(csv, "{},{},{:.1},{:.1}", t + 1, names[i], then.elo[i], running[i] - zero(running)).unwrap();
        }
        previous = Some(then.elo);
        lap("history prefix fitted");
    }
    let key = (opts.prior_draws == 0.0).then(|| {
        let key: String = counts.iter().map(|c| format!("{c}\n")).collect();
        format!("{fit_key}\n{key}")
    });
    History { last, rows: csv, append, key }
}

/// The fast path of the rating history: `elo_history.csv` can be appended to
/// when `elo_history.key` records this fit and was written after it (so the
/// csv is as the last call left it: each call keeps the earlier rows and
/// adds later ones, so its rows are in tournament order, a line each), every
/// tournament the key records still has its number of games (none changed or
/// went), and the csv starts with its header, ends with a newline, holds no
/// carriage return in the part read, and ends with rows of the last
/// tournament the key records. Returns that number of tournaments and that
/// tournament's rows (player, Elo as written), read from the end of the
/// file; None for the slow path.
fn history_tail(out: &Path, fit_key: &str, counts: &[usize]) -> Option<(usize, Vec<(String, String)>)> {
    use std::io::{Read, Seek, SeekFrom};
    const HEADER: &[u8] = b"tournament,player,elo,running\n";
    let (key_path, csv_path) = (out.join("elo_history.key"), out.join("elo_history.csv"));
    let key = fs::read_to_string(&key_path).ok()?;
    let mut lines = key.lines();
    if lines.next() != Some(fit_key) {
        return None;
    }
    let stored: Vec<Option<usize>> = lines.map(|l| l.parse::<usize>().ok()).collect();
    let from = stored.iter().zip(counts).take_while(|(s, c)| **s == Some(**c)).count();
    if from == 0 || from != stored.len() {
        return None;
    }
    let mut file = fs::File::open(&csv_path).ok()?;
    let (key_time, csv_time) = (fs::metadata(&key_path).ok()?.modified().ok()?, file.metadata().ok()?.modified().ok()?);
    if csv_time > key_time {
        return None;
    }
    let size = file.metadata().ok()?.len();
    let header = HEADER.len() as u64;
    if size <= header {
        return None;
    }
    let mut head = vec![0u8; HEADER.len()];
    file.read_exact(&mut head).ok()?;
    if head != HEADER {
        return None;
    }
    let mut span = 1u64 << 22;
    loop {
        let start = size.saturating_sub(span).max(header);
        file.seek(SeekFrom::Start(start)).ok()?;
        let mut buf = Vec::with_capacity((size - start) as usize);
        (&mut file).take(size - start).read_to_end(&mut buf).ok()?;
        if buf.len() as u64 != size - start || buf.last() != Some(&b'\n') || buf.contains(&b'\r') {
            return None;
        }
        // Whole lines only: the first line of the part read may be cut.
        let body = if start == header {
            &buf[..]
        } else {
            match buf.iter().position(|&b| b == b'\n') {
                Some(p) => &buf[p + 1..],
                None => &buf[buf.len()..],
            }
        };
        let mut rows = Vec::new();
        let mut earlier = false;
        if !body.is_empty() {
            for line in body[..body.len() - 1].split(|&b| b == b'\n').rev() {
                let line = std::str::from_utf8(line).ok()?;
                let mut fields = line.split(',');
                let t: usize = fields.next().unwrap_or("").parse().unwrap_or(usize::MAX);
                if t == from {
                    let (Some(player), Some(elo)) = (fields.next(), fields.next()) else { return None };
                    rows.push((player.to_string(), elo.to_string()));
                } else if t < from {
                    earlier = true;
                    break;
                } else {
                    return None;
                }
            }
        }
        if earlier || start == header {
            if rows.is_empty() {
                return None;
            }
            rows.reverse();
            return Some((from, rows));
        }
        span = span.saturating_mul(4);
    }
}

/// `running_elo`, computed the same way, keeping only the ratings after the
/// tournaments from `from` on, and the ratings after the last (0 for all if
/// there are no games).
fn running_elo_from(n: usize, games: &[Played], k: f64, from: usize) -> (Vec<Vec<f64>>, Vec<f64>) {
    let tournaments = games.iter().map(|g| g.tournament + 1).max().unwrap_or(0) as usize;
    let mut by_tournament = vec![Vec::new(); tournaments];
    for g in games {
        by_tournament[g.tournament as usize].push(g);
    }
    let mut ratings = vec![0.0; n];
    let mut change = vec![0.0; n];
    let mut kept = Vec::with_capacity(tournaments.saturating_sub(from));
    for (t, games) in by_tournament.iter().enumerate() {
        change.iter_mut().for_each(|c| *c = 0.0);
        for g in games {
            let surprise = g.white_points - expected(ratings[g.white] - ratings[g.black]);
            change[g.white] += k * surprise;
            change[g.black] -= k * surprise;
        }
        ratings.iter_mut().zip(&change).for_each(|(r, c)| *r += c);
        if t >= from {
            kept.push(ratings.clone());
        }
    }
    (kept, ratings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classifier::Uniform;
    use crate::player::Policy;

    /// `games_per_pair` games between every pair of players with these
    /// ratings, as close to the expected scores as whole games allow.
    fn expected_games(ratings: &[f64], games_per_pair: usize) -> Vec<Played> {
        let mut games = Vec::new();
        for (white, black) in pairings(ratings.len()) {
            let p = expected(ratings[white] - ratings[black]);
            let wins = (p * games_per_pair as f64 / 2.0).round() as usize;
            for g in 0..games_per_pair / 2 {
                let white_points = if g < wins { 1.0 } else { 0.0 };
                games.push(Played { tournament: 0, white, black, white_points, plies: 1 });
            }
        }
        games
    }

    /// `games_per_pair` games between `a` and `b` (half with each colour),
    /// each scoring exactly the expected points of players with these
    /// ratings, so that they are the maximum-likelihood ratings.
    fn exact_games(ratings: &[f64], a: usize, b: usize, games_per_pair: usize) -> Vec<Played> {
        let mut games = Vec::new();
        for (white, black) in [(a, b), (b, a)] {
            let white_points = expected(ratings[white] - ratings[black]);
            games.extend((0..games_per_pair / 2).map(|_| Played { tournament: 0, white, black, white_points, plies: 1 }));
        }
        games
    }

    fn round_robin(ratings: &[f64], games_per_pair: usize) -> Vec<Played> {
        let n = ratings.len();
        (0..n).flat_map(|a| (a + 1..n).flat_map(move |b| exact_games(ratings, a, b, games_per_pair))).collect()
    }

    #[test]
    fn pairings_cover_every_pair_with_both_colours() {
        let pairs = pairings(4);
        assert_eq!(pairs.len(), 12);
        for &(w, b) in &pairs {
            assert_ne!(w, b);
            assert!(pairs.contains(&(b, w)));
        }
    }

    #[test]
    fn maximum_likelihood_recovers_the_ratings() {
        let truth = [-300.0, -100.0, 0.0, 150.0, 250.0];
        let elo = ml_elo(truth.len(), &expected_games(&truth, 2000), 0.01);
        let mean = truth.iter().sum::<f64>() / truth.len() as f64;
        for (e, t) in elo.iter().zip(&truth) {
            assert!((e - (t - mean)).abs() < 3.0, "{elo:?}");
        }
        // Four times the games halve the errors.
        let few = ml_errors(5, &expected_games(&truth, 500), 0.01, &elo);
        let many = ml_errors(5, &expected_games(&truth, 2000), 0.01, &elo);
        for (f, m) in few.iter().zip(&many) {
            assert!((f / m - 2.0).abs() < 0.01, "{few:?} {many:?}");
        }
    }

    #[test]
    fn pure_maximum_likelihood_rates_relative_to_the_anchor() {
        let truth = [0.0, 400.0, 600.0, 1000.0];
        let fit = ml_fit(4, &round_robin(&truth, 100), Some(0), None, true);
        assert_eq!(fit.rated, [true; 4]);
        for (e, t) in fit.elo.iter().zip(&truth) {
            assert!((e - t).abs() < 1e-6, "{:?}", fit.elo);
        }
        // Without an anchor, relative to the mean; a start only saves time.
        let fit = ml_fit(4, &round_robin(&truth, 100), None, Some(&[f64::NAN, 1.0, 2.0, 3.0]), false);
        for (e, t) in fit.elo.iter().zip(&truth) {
            assert!((e - (t - 500.0)).abs() < 1e-6, "{:?}", fit.elo);
        }
        // Four times the games halve the errors, which match ml_errors'.
        let few = ml_fit(4, &round_robin(&truth, 100), Some(0), None, true).errors;
        let many = ml_fit(4, &round_robin(&truth, 400), Some(0), None, true).errors;
        for (f, m) in few.iter().zip(&many) {
            assert!((f / m - 2.0).abs() < 1e-6, "{few:?} {many:?}");
        }
        let elo: Vec<f64> = truth.to_vec();
        for (f, e) in few.iter().zip(ml_errors(4, &round_robin(&truth, 100), 0.0, &elo)) {
            assert!((f - e).abs() < 1e-9);
        }
    }

    #[test]
    fn a_rating_gap_does_not_depend_on_how_many_games_were_played() {
        // Random (the anchor) and two players 400 and 600 Elo above it. Ten
        // times the games between the two leave every rating where it was;
        // virtual draws, whose weight shrinks as games accumulate, would
        // move them.
        let truth = [0.0, 400.0, 600.0];
        let few = round_robin(&truth, 20);
        let mut many = few.clone();
        many.extend(exact_games(&truth, 1, 2, 200));
        for games in [&few, &many] {
            let elo = ml_fit(3, games, Some(0), None, false).elo;
            assert!((elo[1] - 400.0).abs() < 1e-6 && (elo[2] - 600.0).abs() < 1e-6, "{elo:?}");
        }
        let gap = |games: &[Played]| {
            let elo = ml_elo(3, games, 1.0);
            elo[2] - elo[1]
        };
        assert!(gap(&many) - gap(&few) > 10.0, "{} {}", gap(&few), gap(&many));
    }

    #[test]
    fn ratings_that_do_not_exist_are_unrated() {
        // Player 3 won all its games and player 4 played none: both unrated,
        // and player 3's games do not move the others.
        let truth = [0.0, 300.0, 500.0];
        let mut games = round_robin(&truth, 10);
        for opponent in 0..3 {
            games.push(Played { tournament: 0, white: 3, black: opponent, white_points: 1.0, plies: 1 });
            games.push(Played { tournament: 0, white: opponent, black: 3, white_points: 0.0, plies: 1 });
        }
        let fit = ml_fit(5, &games, Some(0), None, true);
        assert_eq!(fit.rated, [true, true, true, false, false]);
        assert!((fit.elo[1] - 300.0).abs() < 1e-6 && (fit.elo[2] - 500.0).abs() < 1e-6, "{:?}", fit.elo);
        assert!(fit.elo[3].is_nan() && fit.errors[4].is_nan());
        // If the anchor never scores, nobody else's rating exists.
        let lost: Vec<Played> = games.iter().filter(|g| g.white != 0 && g.black != 0).copied().chain([
            Played { tournament: 0, white: 0, black: 1, white_points: 0.0, plies: 1 },
            Played { tournament: 0, white: 1, black: 0, white_points: 1.0, plies: 1 },
        ]).collect();
        assert_eq!(ml_fit(5, &lost, Some(0), None, false).rated, [true, false, false, false, false]);
    }

    /// A fresh directory for a test's tournament.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("chessai-tournament-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    fn random_players(names: &[&str]) -> Vec<Entrant<'static>> {
        names.iter().map(|&name| Entrant { name: name.to_string(), player: Player::new(&Uniform, Policy::Rand, 0.1) }).collect()
    }

    fn options(tournaments: u64, catch_up: bool) -> Options {
        Options {
            tournaments,
            seed: 1,
            k: 8.0,
            prior_draws: 0.0,
            anchor: Some(0),
            catch_up,
            save_games: None,
            reports_last: false,
            sparse_fit: false,
            game_cpu: false,
        }
    }

    /// A league whose fields change: each of `n` players plays the
    /// tournaments of a window (some only one), players 0-2 all of them,
    /// with one field inside another and one player who loses every game.
    fn changing_fields(n: usize, seed: u64) -> Vec<Played> {
        let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        let strength: Vec<f64> = (0..n).map(|_| (next() % 1000) as f64 / 300.0).collect();
        let mut games = Vec::new();
        for t in 0..14u64 {
            let field: Vec<usize> = (0..n)
                .filter(|&i| {
                    let first = (i as u64 * 7 + seed) % 12;
                    i < 3 || (t >= first && t < first + 1 + i as u64 % 4) || (t == 13 && i % 5 == 0 && first == 12)
                })
                .filter(|&i| t != 12 || i % 2 == 0)
                .collect();
            for &w in &field {
                for &b in field.iter().filter(|&&b| b != w) {
                    let p = 1.0 / (1.0 + (strength[b] - strength[w]).exp());
                    let white_points = if w == n - 1 {
                        0.0
                    } else if b == n - 1 {
                        1.0
                    } else if next() % 10 == 0 {
                        0.5
                    } else if (next() % 1_000_000) as f64 / 1e6 < p {
                        1.0
                    } else {
                        0.0
                    };
                    games.push(Played { tournament: t, white: w, black: b, white_points, plies: 1 });
                }
            }
        }
        games
    }

    #[test]
    fn the_sparse_fit_is_the_dense_one() {
        for seed in 0..6 {
            let n = 50;
            let games = changing_fields(n, seed);
            for (anchor, start) in [(Some(0), false), (None, false), (Some(1), true)] {
                let dense = ml_fit(n, &games, anchor, None, true);
                let begin = start.then(|| dense.elo.iter().map(|e| e + 3.0).collect::<Vec<f64>>());
                let sparse = sparse_fit(n, &games, anchor, begin.as_deref(), true);
                assert_eq!(dense.rated, sparse.rated);
                assert!(!dense.rated[n - 1]);
                for i in (0..n).filter(|&i| dense.rated[i]) {
                    assert!((dense.elo[i] - sparse.elo[i]).abs() < 1e-9, "elo {i}: {} {}", dense.elo[i], sparse.elo[i]);
                    let (d, s) = (dense.errors[i], sparse.errors[i]);
                    assert!((d - s).abs() < 1e-9 * d.max(1.0), "error {i}: {d} {s}");
                }
            }
        }
    }

    #[test]
    fn tables_written_once_with_the_sparse_fit_are_the_same() {
        let players = random_players(&["a", "b", "c", "d", "e"]);
        let (plain, lean) = (scratch("plain"), scratch("lean"));
        run(&players[..4], &[], &plain, &options(2, false)).unwrap();
        run(&players, &[], &plain, &options(5, false)).unwrap();
        let fast = |tournaments| Options { reports_last: true, sparse_fit: true, ..options(tournaments, false) };
        run(&players[..4], &[], &lean, &fast(2)).unwrap();
        run(&players, &[], &lean, &fast(5)).unwrap();
        // The same to rounding: a rating of 0 may show as -0.0 in one.
        for file in ["games.csv", "ratings.md", "ratings.csv", "head_to_head.md", "head_to_head.csv", "elo_history.csv"] {
            let read = |dir: &PathBuf| fs::read_to_string(dir.join(file)).unwrap().replace("-0.0,", "0.0,");
            assert_eq!(read(&plain), read(&lean), "{file}");
        }
        assert!(run(&players, &[], &lean, &Options { prior_draws: 1.0, ..fast(5) }).is_err());
        fs::remove_dir_all(&plain).unwrap();
        fs::remove_dir_all(&lean).unwrap();
    }

    #[test]
    fn without_catch_up_late_joiners_only_play_new_tournaments() {
        let dir = scratch("late");
        let players = random_players(&["a", "b", "c", "d"]);
        run(&players[..3], &[], &dir, &options(1, false)).unwrap();
        run(&players, &[], &dir, &options(2, false)).unwrap();
        let games = read_games(&dir.join("games.csv"), &["a", "b", "c", "d"]).unwrap();
        assert_eq!(games.iter().filter(|g| g.tournament == 0).count(), 6);
        assert_eq!(games.iter().filter(|g| g.tournament == 1).count(), 12);
        assert!(games.iter().all(|g| g.tournament == 1 || (g.white != 3 && g.black != 3)));
        // With catch-up, d first plays tournament 0's games against a, b and c.
        run(&players, &[], &dir, &options(2, true)).unwrap();
        let games = read_games(&dir.join("games.csv"), &["a", "b", "c", "d"]).unwrap();
        assert_eq!(games.iter().filter(|g| g.tournament == 0).count(), 12);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn retired_players_are_rated_but_play_no_more() {
        let dir = scratch("retired");
        let players = random_players(&["a", "b", "c"]);
        run(&players, &[], &dir, &options(1, false)).unwrap();
        run(&players[..2], &["c".to_string()], &dir, &options(2, false)).unwrap();
        let games = read_games(&dir.join("games.csv"), &["a", "b", "c"]).unwrap();
        assert_eq!(games.len(), 6 + 2);
        assert!(games.iter().filter(|g| g.tournament == 1).all(|g| g.white < 2 && g.black < 2));
        assert!(fs::read_to_string(dir.join("ratings.md")).unwrap().contains("| c (retired) |"));
        assert!(fs::read_to_string(dir.join("ratings.csv")).unwrap().starts_with("rank,player,elo,ci95,running,games,score,wins,losses,draws,plies,retired\n"));
        // A player with games must be listed or retired, and not both.
        assert!(run(&players[..2], &[], &dir, &options(2, false)).is_err());
        assert!(run(&players, &["c".to_string()], &dir, &options(2, false)).is_err());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_rating_history_is_kept_while_its_games_are_the_same() {
        // Three runs of one tournament each reuse the history's earlier
        // prefixes; refitting them all gives the same file.
        let dir = scratch("history");
        let players = random_players(&["a", "b", "c", "d"]);
        for tournaments in 1..=3 {
            run(&players, &[], &dir, &options(tournaments, false)).unwrap();
        }
        let kept = fs::read_to_string(dir.join("elo_history.csv")).unwrap();
        assert_eq!(fs::read_to_string(dir.join("elo_history.key")).unwrap(), "prior_draws 0 anchor a\n12\n12\n12\n");
        fs::remove_file(dir.join("elo_history.key")).unwrap();
        run(&players, &[], &dir, &options(3, false)).unwrap();
        assert_eq!(fs::read_to_string(dir.join("elo_history.csv")).unwrap(), kept);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_history_is_appended_to_only_when_it_is_as_left() {
        // hybrid7: a run appends to elo_history.csv when it is as the last
        // run left it, which gives the file that refitting every prefix
        // gives. A csv changed after its key, or a key that no longer
        // matches, takes the slow path (read, keep and rewrite, as before).
        let players = random_players(&["a", "b", "c", "d"]);
        let fresh = scratch("history-fresh");
        run(&players, &[], &fresh, &options(4, false)).unwrap();
        let whole4 = fs::read_to_string(fresh.join("elo_history.csv")).unwrap();
        run(&players, &[], &fresh, &options(5, false)).unwrap();
        let whole5 = fs::read_to_string(fresh.join("elo_history.csv")).unwrap();
        let dir = scratch("history-append");
        for tournaments in 1..=4 {
            run(&players, &[], &dir, &options(tournaments, false)).unwrap();
        }
        let csv = dir.join("elo_history.csv");
        assert_eq!(fs::read_to_string(&csv).unwrap(), whole4);
        assert!(history_tail(&dir, "prior_draws 0 anchor a", &[12; 5]).is_some());
        run(&players, &[], &dir, &options(5, false)).unwrap();
        assert_eq!(fs::read_to_string(&csv).unwrap(), whole5);
        // A key that recorded fewer tournaments than the csv holds rows of:
        // the slow path drops the later rows and refits them.
        let key = dir.join("elo_history.key");
        fs::write(&key, "prior_draws 0 anchor a\n12\n12\n12\n12\n").unwrap();
        assert!(history_tail(&dir, "prior_draws 0 anchor a", &[12; 5]).is_none());
        run(&players, &[], &dir, &options(5, false)).unwrap();
        assert_eq!(fs::read_to_string(&csv).unwrap(), whole5);
        // A csv changed after its key (a row moved to a later tournament):
        // the slow path drops that row, as the original does.
        let changed = whole5.replacen("\n1,a,", "\n9,a,", 1);
        assert_ne!(changed, whole5);
        fs::write(&csv, &changed).unwrap();
        let later = std::time::SystemTime::now() + std::time::Duration::from_secs(5);
        fs::File::options().write(true).open(&csv).unwrap().set_modified(later).unwrap();
        assert!(history_tail(&dir, "prior_draws 0 anchor a", &[12; 5]).is_none());
        run(&players, &[], &dir, &options(5, false)).unwrap();
        let row = whole5.lines().nth(1).unwrap();
        assert!(row.starts_with("1,a,"));
        assert_eq!(fs::read_to_string(&csv).unwrap(), whole5.replacen(&format!("{row}\n"), "", 1));
        fs::remove_dir_all(&dir).unwrap();
        fs::remove_dir_all(&fresh).unwrap();
    }

    #[test]
    fn the_fast_games_reader_reads_the_same() {
        let dir = scratch("read-games");
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("games.csv");
        let names = ["a", "b", "c"];
        let plain = "tournament,white,black,result,plies\n0,a,b,1-0,10\n0,b,a,1/2-1/2,20\n1,c,a,0-1,30";
        fs::write(&path, plain).unwrap();
        assert_eq!(read_games_fast(&path, &names), Some(read_games_plain(&path, &names).unwrap()));
        assert_eq!(read_games(&path, &names).unwrap().len(), 3);
        // Anything else is left to the original reader: the same games or the same error.
        for odd in [plain.replace('\n', "\r\n"), format!("{plain}\n\n"), plain.replace("1-0", "2-0"), plain.replace(",c,", ",d,")] {
            fs::write(&path, &odd).unwrap();
            assert_eq!(read_games_fast(&path, &names), None);
            match (read_games(&path, &names), read_games_plain(&path, &names)) {
                (Ok(a), Ok(b)) => assert_eq!(a, b),
                (Err(a), Err(b)) => assert_eq!(a.to_string(), b.to_string()),
                _ => panic!("different outcomes"),
            }
        }
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn running_elo_rewards_upsets_more() {
        // Tournament 0: A beats B twice. Tournament 1: B beats A twice; B,
        // now rated lower, gains more than A gained.
        let game = |t, white, black, white_points| Played { tournament: t, white, black, white_points, plies: 1 };
        let games =
            [game(0, 0, 1, 1.0), game(0, 1, 0, 0.0), game(1, 0, 1, 0.0), game(1, 1, 0, 1.0)];
        let history = running_elo(2, &games, 10.0);
        assert_eq!(history[0], vec![10.0, -10.0]);
        assert!(history[1][1] - history[0][1] > 10.0);
        assert!((history[1][0] + history[1][1]).abs() < 1e-9);
    }

    #[test]
    fn players_who_join_later_get_the_games_they_missed() {
        // Two tournaments of players 0-2; then player 3 joins.
        let games: Vec<Played> = (0..2)
            .flat_map(|t| pairings(3).into_iter().map(move |(white, black)| (t, white, black)))
            .map(|(tournament, white, black)| Played { tournament, white, black, white_points: 0.5, plies: 1 })
            .collect();
        assert_eq!(missing_games(3, &games, 2), Ok(vec![]));
        let missing = missing_games(4, &games, 2).unwrap();
        assert_eq!(missing.len(), 2 * (12 - 6));
        assert!(missing.iter().all(|&(t, w, b)| t < 2 && (w == 3 || b == 3)));
        let twice = [games[0], games[0]];
        assert!(missing_games(3, &twice, 1).is_err());
    }

    #[test]
    fn head_to_head_counts_both_sides() {
        let game = |white, black, white_points| Played { tournament: 0, white, black, white_points, plies: 1 };
        let table = head_to_head(2, &[game(0, 1, 1.0), game(1, 0, 0.5)]);
        assert_eq!(table[0][1], [1, 0, 1]);
        assert_eq!(table[1][0], [0, 1, 1]);
    }
}
