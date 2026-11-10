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
//!   unrated.
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
) -> io::Result<Vec<Played>> {
    let order = pairings(entrants.len());
    games
        .par_iter()
        .map(|&(t, white, black)| {
            let g = order.iter().position(|&p| p == (white, black)).expect("no such pairing");
            let record = play_game(entrants[white].player, entrants[black].player, derive_seed(seed, t, g as u64));
            if let Some(dir) = save {
                let dir = game_dir(dir, t, &entrants[white].name, &entrants[black].name);
                write_round(&dir, std::slice::from_ref(&record))?;
            }
            Ok(Played { tournament: t, white, black, white_points: points(record.result), plies: record.moves.len() })
        })
        .collect()
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
/// rewrites the ratings and tables after each one (or once, if none are left
/// to play). The `retired` players play no more, but their games in
/// `games.csv` count; every player there must be an entrant or retired.
/// Prints a line per tournament.
pub fn run(entrants: &[Entrant], retired: &[String], out: &Path, opts: &Options) -> io::Result<()> {
    let invalid_input = |msg: String| io::Error::new(io::ErrorKind::InvalidInput, msg);
    if !(opts.prior_draws >= 0.0) {
        return Err(invalid_input(format!("prior_draws must be 0 or more, not {}", opts.prior_draws)));
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
            let played = play_games(entrants, &missing, opts.seed, save)?;
            append(&games_path, &names, &played)?;
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
        let played = play_tournament(entrants, done, opts.seed, save)?;
        append(&games_path, &names, &played)?;
        games.extend(played);
        done += 1;
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
pub fn read_games(path: &Path, names: &[&str]) -> io::Result<Vec<Played>> {
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
/// virtual draws (`ml_elo`), or pure (`ml_fit`, which starts from `start` if
/// given, ratings as returned). Standard errors only if `errors`.
pub fn ratings(n: usize, games: &[Played], opts: &Options, start: Option<&[f64]>, errors: bool) -> Ratings {
    if opts.prior_draws > 0.0 {
        let raw = ml_elo(n, games, opts.prior_draws);
        let errors = if errors { ml_errors(n, games, opts.prior_draws, &raw) } else { vec![f64::NAN; n] };
        let zero = opts.anchor.map_or(0.0, |a| raw[a]);
        Ratings { elo: raw.iter().map(|r| r - zero).collect(), errors, rated: vec![true; n] }
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
fn write_reports(out: &Path, names: &[&str], active: usize, games: &[Played], opts: &Options) -> io::Result<Ratings> {
    let n = names.len();
    let tournaments = games.iter().map(|g| g.tournament + 1).max().unwrap_or(0);
    let zero = |ratings: &[f64]| opts.anchor.map_or(0.0, |a| ratings[a]);
    let fit = ratings(n, games, opts, None, true);
    let history = running_elo(n, games, opts.k);
    let last = history.last().cloned().unwrap_or_else(|| vec![0.0; n]);
    let running: Vec<f64> = last.iter().map(|r| r - zero(&last)).collect();
    let h2h = head_to_head(n, games);
    let mut plies = vec![0usize; n];
    for g in games {
        plies[g.white] += g.plies;
        plies[g.black] += g.plies;
    }
    // The rated players by rating, then the unrated in the players' order.
    let mut order: Vec<usize> = (0..n).filter(|&i| fit.rated[i]).collect();
    order.sort_by(|&a, &b| fit.elo[b].total_cmp(&fit.elo[a]));
    let unrated: Vec<usize> = (0..n).filter(|&i| !fit.rated[i]).collect();
    let anchor = opts.anchor.map_or("the field's mean".to_string(), |a| names[a].to_string());
    let label = |i: usize| if i < active { names[i].to_string() } else { format!("{} (retired)", names[i]) };
    let record = |i: usize| {
        let [wins, losses, draws] =
            (0..n).fold([0u32; 3], |t, j| [t[0] + h2h[i][j][0], t[1] + h2h[i][j][1], t[2] + h2h[i][j][2]]);
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
            let [w, l, d] = h2h[i][j];
            let games = w + l + d;
            if i == j || games == 0 {
                md.push_str(" · |");
                continue;
            }
            write!(md, " {:.0} |", 100.0 * (w as f64 + d as f64 / 2.0) / games as f64).unwrap();
        }
        md.push('\n');
    }
    let mut csv = String::from("player,opponent,wins,losses,draws,score,elo_difference\n");
    for &i in &everyone {
        for &j in &everyone {
            let [w, l, d] = h2h[i][j];
            let games = w + l + d;
            if i == j || games == 0 {
                continue;
            }
            let score = (w as f64 + d as f64 / 2.0) / games as f64;
            let diff = if score <= 0.0 || score >= 1.0 { f64::NAN } else { -400.0 * (1.0 / score - 1.0).log10() };
            writeln!(csv, "{},{},{w},{l},{d},{score:.4},{diff:.1}", names[i], names[j]).unwrap();
        }
    }
    fs::write(out.join("head_to_head.md"), md)?;
    fs::write(out.join("head_to_head.csv"), csv)?;

    // Both ratings after every tournament, for the players rated by then.
    // Refitting every prefix takes long in a big field, so with pure maximum
    // likelihood the rows of an earlier call are kept for the prefixes whose
    // games are unchanged: elo_history.key records the fit, then each
    // tournament's number of games.
    let mut counts = vec![0usize; tournaments as usize];
    for g in games {
        counts[g.tournament as usize] += 1;
    }
    let fit_key = format!("prior_draws {} anchor {anchor}", opts.prior_draws);
    let mut csv = String::from("tournament,player,elo,running\n");
    let (mut from, mut previous) = (0, None);
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
    for (t, running) in history.iter().enumerate().skip(from) {
        let upto: Vec<Played> = games.iter().filter(|g| g.tournament <= t as u64).copied().collect();
        let then = ratings(n, &upto, opts, previous.as_deref(), false);
        for i in (0..n).filter(|&i| then.rated[i]) {
            writeln!(csv, "{},{},{:.1},{:.1}", t + 1, names[i], then.elo[i], running[i] - zero(running)).unwrap();
        }
        previous = Some(then.elo);
    }
    fs::write(out.join("elo_history.csv"), csv)?;
    if opts.prior_draws == 0.0 {
        let key: String = counts.iter().map(|c| format!("{c}\n")).collect();
        fs::write(out.join("elo_history.key"), format!("{fit_key}\n{key}"))?;
    }
    Ok(fit)
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
        Options { tournaments, seed: 1, k: 8.0, prior_draws: 0.0, anchor: Some(0), catch_up, save_games: None }
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
