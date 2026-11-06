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
//!   interval.
//!
//! Players can join later: listed with the others, they first play the games
//! of the earlier tournaments they missed, as if they had been there.

use std::fmt::Write as _;
use std::fs;
use std::io::{self, Write as _};
use std::path::Path;
use std::time::Instant;

use rayon::prelude::*;

use crate::game::GameResult;
use crate::player::Player;
use crate::selfplay::{derive_seed, play_game};

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
    /// ratings, so that a perfect score gives a finite rating; above 0.
    pub prior_draws: f64,
    /// The entrant rated 0 in the tables; the field's mean if None.
    pub anchor: Option<usize>,
}

/// Every ordered pair of different entrants as (white, black): each pairing
/// once with each colour.
pub fn pairings(n: usize) -> Vec<(usize, usize)> {
    (0..n).flat_map(|white| (0..n).filter(move |&black| black != white).map(move |black| (white, black))).collect()
}

/// Plays tournament `t` in parallel. Game `g` (in the order of `pairings`)
/// uses the seed `derive_seed(seed, t, g)`, so the games do not depend on the
/// number of threads.
pub fn play_tournament(entrants: &[Entrant], t: u64, seed: u64) -> Vec<Played> {
    let games: Vec<(u64, usize, usize)> = pairings(entrants.len()).into_iter().map(|(w, b)| (t, w, b)).collect();
    play_games(entrants, &games, seed)
}

/// Plays `games`, as (tournament, white, black), in parallel; each is seeded
/// as in `play_tournament`, by its pairing's place in `pairings`.
fn play_games(entrants: &[Entrant], games: &[(u64, usize, usize)], seed: u64) -> Vec<Played> {
    let order = pairings(entrants.len());
    games
        .par_iter()
        .map(|&(t, white, black)| {
            let g = order.iter().position(|&p| p == (white, black)).expect("no such pairing");
            let record = play_game(entrants[white].player, entrants[black].player, derive_seed(seed, t, g as u64));
            Played { tournament: t, white, black, white_points: points(record.result), plies: record.moves.len() }
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

/// Plays tournaments until `opts.tournaments` have been played in `out`,
/// continuing after the games already in `out/games.csv` (first the games
/// players who joined later missed), and rewrites the ratings and tables
/// after each one (or once, if none are left to play). Prints a line per
/// tournament.
pub fn run(entrants: &[Entrant], out: &Path, opts: &Options) -> io::Result<()> {
    assert!(opts.prior_draws > 0.0, "prior_draws must be above 0");
    let names: Vec<&str> = entrants.iter().map(|e| e.name.as_str()).collect();
    fs::create_dir_all(out)?;
    let games_path = out.join("games.csv");
    let mut games = if games_path.exists() {
        read_games(&games_path, &names)?
    } else {
        fs::write(&games_path, "tournament,white,black,result,plies\n")?;
        Vec::new()
    };
    let mut done = games.iter().map(|g| g.tournament + 1).max().unwrap_or(0);
    let missing = missing_games(names.len(), &games, done)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("{}: {e}", games_path.display())))?;
    if !missing.is_empty() {
        let start = Instant::now();
        let played = play_games(entrants, &missing, opts.seed);
        append(&games_path, &names, &played)?;
        games.extend(played);
        println!(
            "caught up {} games of tournaments 1-{done} for players who joined later in {:.0}s",
            missing.len(),
            start.elapsed().as_secs_f64()
        );
    }
    if done >= opts.tournaments && !games.is_empty() {
        write_reports(out, &names, &games, opts)?;
    }
    while done < opts.tournaments {
        let start = Instant::now();
        let played = play_tournament(entrants, done, opts.seed);
        append(&games_path, &names, &played)?;
        games.extend(played);
        done += 1;
        let elo = write_reports(out, &names, &games, opts)?;
        let mut order: Vec<usize> = (0..names.len()).collect();
        order.sort_by(|&a, &b| elo[b].total_cmp(&elo[a]));
        let top: Vec<String> = order.iter().take(3).map(|&i| format!("{} {:+.0}", names[i], elo[i])).collect();
        println!(
            "tournament {done}/{}: {} games in {:.0}s; top: {}",
            opts.tournaments,
            names.len() * (names.len() - 1),
            start.elapsed().as_secs_f64(),
            top.join(", ")
        );
    }
    Ok(())
}

/// Reads a `games.csv` written by `run` for the entrants `names` (who may
/// include players with no games yet; see `missing_games`).
pub fn read_games(path: &Path, names: &[&str]) -> io::Result<Vec<Played>> {
    let invalid = |msg: String| io::Error::new(io::ErrorKind::InvalidData, format!("{}: {msg}", path.display()));
    let index = |name: &str| {
        names.iter().position(|&n| n == name).ok_or_else(|| invalid(format!("player {name} is not in the tournament")))
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
/// `head_to_head.csv` and `elo_history.csv` into `out`. Returns the
/// maximum-likelihood ratings as shown (relative to the anchor).
fn write_reports(out: &Path, names: &[&str], games: &[Played], opts: &Options) -> io::Result<Vec<f64>> {
    let n = names.len();
    let tournaments = games.iter().map(|g| g.tournament + 1).max().unwrap_or(0);
    let shift = |ratings: &[f64]| -> Vec<f64> {
        let zero = opts.anchor.map_or(0.0, |a| ratings[a]);
        ratings.iter().map(|r| r - zero).collect()
    };
    let raw = ml_elo(n, games, opts.prior_draws);
    let errors = ml_errors(n, games, opts.prior_draws, &raw);
    let elo = shift(&raw);
    let history = running_elo(n, games, opts.k);
    let running = shift(history.last().map_or(&vec![0.0; n][..], |r| &r[..]));
    let h2h = head_to_head(n, games);
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| elo[b].total_cmp(&elo[a]));
    let anchor = opts.anchor.map_or("the field's mean".to_string(), |a| names[a].to_string());

    // Rating list.
    let mut md = format!(
        "# Ratings after {tournaments} tournaments ({} games)\n\n\
         Elo: maximum likelihood over all games, relative to {anchor}; ± is the 95% interval relative to \
         the field's mean. Running: classic Elo updated after every tournament (K = {} per game), \
         relative to {anchor}.\n\n\
         | # | Player | Elo | ± | Running | Games | Score | Wins | Losses | Draws | Plies |\n\
         |---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n",
        games.len(),
        opts.k
    );
    let mut csv = String::from("rank,player,elo,ci95,running,games,score,wins,losses,draws,plies\n");
    for (rank, &i) in order.iter().enumerate() {
        let [wins, losses, draws] = (0..n).fold([0u32; 3], |t, j| [t[0] + h2h[i][j][0], t[1] + h2h[i][j][1], t[2] + h2h[i][j][2]]);
        let played = (wins + losses + draws).max(1) as f64;
        let score = 100.0 * (wins as f64 + draws as f64 / 2.0) / played;
        let mine = games.iter().filter(|g| g.white == i || g.black == i);
        let plies = mine.map(|g| g.plies).sum::<usize>() as f64 / played;
        let ci = 1.96 * errors[i];
        writeln!(
            md,
            "| {} | {} | {:+.0} | {ci:.0} | {:+.0} | {} | {score:.1}% | {wins} | {losses} | {draws} | {plies:.0} |",
            rank + 1,
            names[i],
            elo[i],
            running[i],
            wins + losses + draws
        )
        .unwrap();
        writeln!(
            csv,
            "{},{},{:.1},{ci:.1},{:.1},{},{score:.2},{wins},{losses},{draws},{plies:.1}",
            rank + 1,
            names[i],
            elo[i],
            running[i],
            wins + losses + draws
        )
        .unwrap();
    }
    fs::write(out.join("ratings.md"), md)?;
    fs::write(out.join("ratings.csv"), csv)?;

    // Head-to-head: row's score against each column, players in rating order.
    let mut md = format!(
        "# Head to head after {tournaments} tournaments\n\n\
         Each cell is the row player's score (%) against the column player, \
         over {} games per pairing. Players in rating order; columns by rank.\n\n| # | Player |",
        2 * tournaments
    );
    for rank in 1..=n {
        write!(md, " {rank} |").unwrap();
    }
    md.push_str("\n|---:|---|");
    md.push_str(&"---:|".repeat(n));
    md.push('\n');
    let mut csv = String::from("player,opponent,wins,losses,draws,score,elo_difference\n");
    for (rank, &i) in order.iter().enumerate() {
        write!(md, "| {} | {} |", rank + 1, names[i]).unwrap();
        for &j in &order {
            let [w, l, d] = h2h[i][j];
            let games = w + l + d;
            if i == j || games == 0 {
                md.push_str(" · |");
                continue;
            }
            let score = (w as f64 + d as f64 / 2.0) / games as f64;
            write!(md, " {:.0} |", 100.0 * score).unwrap();
            let diff = if score <= 0.0 || score >= 1.0 { f64::NAN } else { -400.0 * (1.0 / score - 1.0).log10() };
            writeln!(csv, "{},{},{w},{l},{d},{score:.4},{diff:.1}", names[i], names[j]).unwrap();
        }
        md.push('\n');
    }
    fs::write(out.join("head_to_head.md"), md)?;
    fs::write(out.join("head_to_head.csv"), csv)?;

    // Both ratings after every tournament.
    let mut csv = String::from("tournament,player,elo,running\n");
    for (t, running) in history.iter().enumerate() {
        let upto: Vec<Played> = games.iter().filter(|g| g.tournament <= t as u64).copied().collect();
        let elo = shift(&ml_elo(n, &upto, opts.prior_draws));
        let running = shift(running);
        for i in 0..n {
            writeln!(csv, "{},{},{:.1},{:.1}", t + 1, names[i], elo[i], running[i]).unwrap();
        }
    }
    fs::write(out.join("elo_history.csv"), csv)?;
    Ok(elo)
}

#[cfg(test)]
mod tests {
    use super::*;

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
