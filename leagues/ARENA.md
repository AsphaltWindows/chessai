# Arenas

An arena rates fixed models that are already trained. It has no lines, no training and no versions. The models
play repeated double round-robins with the random references and get rated, in the same way and in the same files
as an experiment's tournament. Since 11-12 a spec may instead pin the mean of a fixed reference set at Elo 0, with no
random player at all (Pinned anchor), and give each entry its own ply (Mixed-ply arenas). Use one to compare models
from different experiments, or models that no league holds, such as a run's early round models.

The script is leagues/arena.py. It only reads leagues.py (for its helpers and settings) and never changes a league.

    python3 leagues/arena.py create SPEC.json                 makes leagues/arenas/<name>/ (or the spec's "dir")
    python3 leagues/arena.py run NAME|DIR [--rounds N] [--threads T]
    python3 leagues/arena.py status NAME|DIR [--vs PLAYER]
    python3 leagues/arena.py fit NAME|DIR [--write] [--top N]   the fit report (Fit report)

`run` first makes the own calibration games that are missing. It then plays round-robins in chunks until the stop
rule holds, and prints the status. Running it again continues where it stopped. After a run that was cut short,
the next one first brings the tables up to date with the games played. `--rounds N` plays N more
round-robins whatever the stop rule says. `--threads` overrides the config's threads; the games are the same at
any thread count. `status --vs PLAYER` lists that player's direct results against every other player: wins,
losses, draws, score and Elo. The Elo comes with the 95% Wilson interval of the score, counting a draw as half a
win, which makes the interval a little wide. A perfect score still gets a finite lower bound.

Start a long run detached, for example:

    setsid nohup python3 -u leagues/arena.py run NAME >> leagues/arenas/NAME/run.out 2>&1 < /dev/null &

## Spec

```json
{
 "name": "matched-depth",
 "description": "what it asks",
 "seed": 1, "nice": 15, "threads": 2, "jobs": 2,
 "binary": "models/from-scratch/bin/chessai-hybrid7",
 "owncal_binary": "models/from-scratch/bin/chessai-hybrid4gfx",
 "stop": {"ci95": 10, "chunk_rounds": 10, "min_rounds": 10, "max_rounds": 400},
 "entries": [
  {"version": "selfplay-depth:F-L+depth=4.sp@99", "positions": 1649603, "label": "LR d4"},
  {"file": "models/from-scratch/matched-depth/F-L+depth=0.sp.round@5.lr.model", "kind": "lr", "heldout": "lr",
   "name": "F-L+depth=0.sp.round@5", "positions": 1511085, "label": "LR d0 matched"}
 ]
}
```

Defaults, for keys a spec leaves out:

- seed 1; nice 15; threads 2; jobs 2 (own-games jobs at once, threads/jobs threads each);
- binary: chessai-hybrid7;
- flags: the experiments' `--reports-last --sparse-fit`;
- references: random, random-1, random-2 and random-3 at plies 0-3, with random as the anchor (Elo 0); none with an
  "anchor" (Pinned anchor);
- plies: [2];
- calibration: the experiments' settings (leagues.py DEFAULTS);
- owncal_binary: the binary;
- keep_own_games: false;
- owncal_cache: true (own games from and into the shared cache; see Calibration);
- stop: ci95 20, chunk_rounds 10, min_rounds 10, max_rounds 200.

For arenas at plies 2-5, use `"binary": "models/from-scratch/bin/chessai-hybrid12"` (from 11-11 07:34). It plays the
same games as chessai-hybrid11, hybrid10 and hybrid9, byte for byte.
- Against hybrid11 (models/from-scratch/fastbin/hybrid12/summary.txt):
  - flat LR moves cost 31-35% less at ply 3, 25-29% less at ply 5 and 3-5% less at ply 4;
  - flat probit moves cost 16% less at ply 3, 15% at ply 5 and 1% at ply 4;
  - NB moves cost 11% less at ply 3 and 18% at ply 5;
  - plies 1-2, trees, DAGs, MoEs and FM cost the same.
- hybrid11 (before 07:34 the binary for plies 3-4, and for ply 2 with flat LR or NB entries) had cut, against hybrid10
  (models/from-scratch/fastbin/hybrid11/summary.txt):
  - flat LR moves cost 25% less at ply 4, 12% at ply 3 and 15% at ply 2;
  - NB moves cost 28% less at ply 4 and 12-13% at plies 2-3;
  - flat probit moves cost 13% less at ply 4 and 8% at ply 3;
  - trees and FM cost the same.
- hybrid10 had already cut ply-3 moves by 13-41% against hybrid9 (LR 21%, probit 38%, trees 22-32%, FM 41%;
  models/from-scratch/fastbin/hybrid10/summary.txt).

Relative paths are from the repository root. The examples are in leagues/specs/arenas/.

Entries:

- `"version": "<experiment>:<line>@<v>"` is an experiment's version: runs/<line>/<kind><v>.model, with the kind,
  held-out games, calibration settings and tournament binary of that experiment. `<experiment>` is a name in
  leagues/experiments/ or a path. The player name is `<line>@<v>` unless the entry gives a name.
- `"file": PATH` is any model file. It needs `kind` and `name`. `heldout` names a set of held-out games in the
  calibration settings (lr or cnbp); without it the model plays uncalibrated before its own games.
- `plies` (default: the spec's) lists the plies the entry plays at; each ply is one player, named `<name>-<ply>`.
  `"ply": P` (11-12) is the same as `"plies": [P]`; give one or the other. Entries may differ: a mixed-ply arena.
- `positions`, `label` and `note` are only for the reader: status shows positions and label.

`create` copies each model into the arena (models/<name>.<kind>.model), checks the copy's md5, and records the
source, md5 and calibration settings in config.json. A later change to the source does not change the arena.

## Pinned anchor

(11-12, the global ladder; analyst 6.85.) With one random reference as the anchor, a field that beats random in every
game is never rated. No entry is linked both ways to the anchor, so every interval stays infinite and a ci stop runs
on to max_rounds (archparity-p3: 0 of 28 rated; cohort-fp-p2). A spec can pin a reference set instead:

    "references": [],
    "anchor": {"mean_of": ["<entry name>-<ply>", ...], "note": "what the set is"}

`"mean_of": "field"` takes every player. `references` must be left out or empty: the anchor replaces them. Then:

- the tournament runs without `--anchor`, so ratings.csv is relative to the rated players' mean;
- arena.py re-centres every rating on the reference players' mean. Each player gets the 95% interval of that
  difference, from the fit's information matrix, computed as the binary's own errors are: with the field as the
  reference set it reproduces ratings.csv's ci95 within 0.05 Elo (flatply-p1, 41 players). These are pinned.csv,
  which `status` shows (a * marks the reference players);
- the stop rule reads those intervals, so ciX stops (a 6-player ci70 test stopped after 20 round-robins);
- if a reference player is unrated (too few games), the anchor is undefined. Then no player gets an Elo, every
  interval counts as unbounded, and status and fit.md say so.

A fixed reference set (the standing panels, by name and version) gives every arena that holds it the same zero, so
separate runs share a scale. The zero is the same only within the panel's own matchups against each new field (kind
pairs carry 40-70 Elo of matchup, analyst 6.85), so compare across runs with that in mind.

Since 11-12 ~11:30, make_split_read.py and `library.py spec` write `"mean_of": "field"` by default.
make_split_read.py's `--anchor TAG[,TAG]` pins those tags' entries instead. `--random-anchor` (make_split_read.py)
and `--anchor random` (`library.py spec`) write the spec as before and refuse ciX.

## Mixed-ply arenas

Each entry may play at its own plies (`"ply"` or `"plies"`), and players are still named `<name>-<ply>`. The
tournament's players.txt gives each player its depth (`NAME DEPTH SPEC`), so one tournament plays every depth. An
arena at a single ply writes the same files as before. Calibration is per model, not per ply: the own games are at
depth 1, so a model's players at different plies share one calibration.

CPU per game between two players of one class (ms; 11-12, chessai-hybrid16, T 0.02; flat = lr, probit, cnbp; tree =
bdt, bdd):

| ply | flat | tree |
|---|---:|---:|
| 1 | 0.5 | 1.2 |
| 2 | 7.6 | 19.5 |
| 3 | 41.6 | 162.7 |
| 4 | 438 | 2084 |

A game between classes or plies costs about the mean of the two players' figures, so a double round-robin costs about
(players - 1) x the sum of the players' figures.

A mixed-ply rating ranks deployed players: evaluator plus depth. It is not a transitive scale across plies, because
models swap places between plies and kinds carry matchups (analyst 6.85). Read fit.md: within a ply the ratings are
a ranking, across plies a map with known folds. A ply that never scores against another stays unrated until it does.
A 1-round-robin smoke of 8 players at plies 1-2 rated only the ply-1 group; after 4 round-robins all 8 were rated.

## Calibration

Every entry plays with the calibration the experiments use, fitted on its own games. The model plays itself 600
games at depth 1 with log-odds softmax at T 0.1, seed 5, under its held-out calibration. The command is
leagues.py's make_own, and the games go to owncal/<name>/. The tournament fits the calibration on them when it
loads the player (`cal:<owncal>/<name>:<kind>:<model>`).

- An experiment version keeps its experiment's calibration. Its own games are made with that experiment's binary
  and settings, so they come out as they did there. The arena checks this: after the first chunk it compares the
  calibration its tournament fitted (tour/tournament.log) with the one the experiment's tournament fitted for the
  version (the experiment's tour/tournament.log). It writes both to calibration.csv with identical yes or NO, and
  status reports them.
- A file entry's own games are made with the spec's calibration settings and owncal_binary. Use the binary and
  settings its source used, to calibrate it as its source would have.

The games are deterministic. They depend only on the model, its kind, the calibration settings, the held-out games
and the binary that makes them, not on any path. So arenas share them in one cache, leagues/arenas/.owncal-cache/
(since 2026-11-11):

- An entry's key is a hash of four things: the own-games command without its paths (kind, settings and flags), the
  md5 of the model (the arena's copy), the md5 of the held-out games, and the md5 of the binary. Each cache entry,
  <key>/, holds the four game files, read-only, and key.json: the key's parts, each file's size and md5, and when
  the games were made, in which arena, for which entry, and at what CPU cost.
- When an entry's key is cached, the arena copies the games into owncal/<name>/ (a reflink: no data is copied on
  this filesystem). It checks each copy's size and md5 against key.json. owncal/<name>.arena then says "from the
  cache" instead of holding the command's output, and arena.log counts the entries taken from the cache and made.
  A missing or damaged entry is made again.
- Games an arena makes go into the cache. Two runs can make the same key at once. The second then compares its
  games with the cached ones byte for byte, and they must be equal. If they differ, it logs a WARNING and removes
  the cache entry.
- The cache keeps at most 20 GB. Past that, the entries used longest ago go first.
- `"owncal_cache": false` in a spec turns the cache off: every run that needs the own games makes them, as before.
  Arenas created before 2026-11-11 have no such key and use the cache. CHESSAI_OWNCAL_CACHE moves the cache (for
  tests).
- The proof that cached games are the ones a run would make is in models/from-scratch/fastbin/owncache/. Four finished
  arenas were rerun from copies: lr, probit, fm, bdt with LR and probit leaves, and cnbp, calibrated at depth 1 and
  2, on binaries hybrid4gfx, hybrid8 and hybrid9. Each fitted the same calibrations as its original run, from games
  made fresh and from the cache. A sample of 16 entries was also remade independently and matched the cache byte
  for byte.

Experiments don't use the cache; they delete the own games of versions that leave their field.

## Files

leagues/arenas/<name>/:

- config.json: the resolved spec, with entries, copies, md5s and settings.
- state.json: status (created, calibration, running, done, paused or failed), round-robins played, the stop
  reason, and the CPU and wall time and widest entry ci95 of each chunk.
- arena.log: what the run did; run.out: the detached run's output.
- calibration.csv: entry, source, expected, arena, identical.
- models/: the copies.
- owncal/: each entry's <name>.json and <name>.arena. The .json has the play spec, the md5 and, with the cache, the
  cache key. The .arena has the own-games command and its output, or a "from the cache" line. The own games themselves,
  in owncal/<name>/, are 10-15 MB an entry. A run removes them when it ends, unless keep_own_games is set. The next
  run that has games to play takes them from the shared cache, or makes them again, and they come out the same.
- fit.md, residuals.csv, and pinned.csv with a pinned anchor: the fit report (Fit report), written after every chunk
  of an arena with a pinned anchor, or by `fit --write`.
- tour/: the tournament's own files, as in an experiment. These are players.txt, retired.txt (empty: an arena's field
  is fixed), games.csv, game_cpu.csv, ratings.csv/.md, head_to_head.csv/.md, elo_history.csv and tournament.log.
  Their formats are the experiments' (FORMATS.md), so tools written for an experiment's tour/ work here. The one
  difference is the binary's: ratings.csv has no retired column when nobody is retired.

The tournament command is the experiments' without `--save-games`, because nothing trains on an arena's games:
`tournament players.txt . --tournaments N --use-probs --anchor random --seed S --no-catch-up --retired retired.txt
--prior-draws 0 --reports-last --sparse-fit --game-cpu`. With `--prior-draws 0` a player is unrated until its
results link it both ways to the anchor. A field of strong models therefore needs references close enough to them.
With a pinned anchor the command has no `--anchor`: the binary rates the group linked both ways to its player with the
most games (Pinned anchor).

## Stop rule

`run` plays chunk_rounds round-robins at a time. After each chunk it stops if min_rounds are played and every
entry's ci95 is at most stop.ci95; an unrated entry counts as unbounded. It also stops at max_rounds.
References are not counted. The ci95 is the tournament's (ratings.csv): the 95% interval relative to the rated
players' mean. With a pinned anchor it is pinned.csv's: the interval of the player's Elo minus the reference set's
mean. Every player counts there, since every player is an entry. The rule decides only how many round-robins are
played; the games are fixed by the seed. So two arenas with the same spec play the same games, and stopping later
only adds round-robins.

## Fit report

`arena.py fit NAME|DIR` prints the Bradley-Terry fit report of any arena. `--write` also writes it to fit.md and
residuals.csv, and pinned.csv with a pinned anchor. An arena with a pinned anchor writes the three after every chunk.

- residuals.csv has one row per pair of rated players, random references left out, lower ply first (then by name):
  - games, wins, losses and draws;
  - observed and predicted score;
  - z = (s - p) / sqrt(p (1 - p) / n);
  - the pair's Elo deviation from the fit, dev = ELO (s - p) / (p (1 - p)), with se = ELO / sqrt(n p (1 - p))
    (ELO = 400 / ln 10; dev / se = z).
- The summary:
  - chi2/dof of the z, with dof = pairs - (rated players - 1). It is about 1 when the fit holds within noise, and
    less with many draws.
  - rms z.
  - The excess SD: the spread of the deviations beyond their noise (DerSimonian-Laird, weights 1/se^2). This is the
    non-transitive spread in Elo; lopsided pairs with few games weigh next to nothing.
- By ply pair, when the arena has several plies: the same per group. Cross-ply groups also get the weighted mean
  deviation of the lower-ply player. With two plies that mean is 0 by construction (the fit's score equations); with
  more plies, it shows the plies' offsets failing to add up.
- The largest deviations by |z| (`--top`, default 15).

When the binary rated nobody (a random anchor that never scores, as in archparity-p2 to p4), the report refits the
entries' games among themselves the binary's way: pure maximum likelihood over the group linked both ways to the
player with the most games. On flatply-p1 and archparity-p1 the refit matches the binary within about 0.05 Elo.

First readings (11-12):

| arena | chi2/dof | excess SD |
|---|---:|---:|
| flatply-p1 | 0.81 | 0 |
| archparity-p1 | 0.80 | 0 |
| archparity-p2 | 1.35 | 33.5 |
| archparity-p3 | 1.5 | 37.0 |

Ply 1 is transitive within noise. Plies 2 and 3 carry about 35 Elo of non-transitive spread.

## The ladder builder

leagues/specs/arenas/make_ladder.py writes the global ladder's spec from the model library (the user, 11-12 ~11:11;
analyst 6.85; models/NEW-DESIGN.md). There is no training. Each promising library final plays at its specified ply
(its training ply), one entry per model, pinned on the standing panels.

    python3 leagues/specs/arenas/make_ladder.py NAME [--dry-run] [--rounds 10|ciX] [--plies 1,2,3,4]
        [--max-per-ply 16] [--depth-anchors K] [--anchor-plies 1,2] [--include ID[@V][:PLY]] [--exclude REGEX]

- The reference set (Elo 0):
  - ply 1: error-routing-p1, error-dag-probit-p1, grow-p1 and grow-p1b at v300;
  - ply 2: replay-p2, diversity-p2 and probit-late-p2 at v299;
  - an F-P and a tree from each, first replicates.
- Plies 3 and 4 have no standing panel. Provisional sets are labelled, and join the anchor only with
  `--anchor-plies`: flat-p3 and flat-p3-b's F-L at v199 at ply 3; flat-p3-w4's F-L at v99 at ply 4.
- Entries:
  - at plies 1 and 2, one per family (F-P, F-L, T-P-Ne, T-P-Ne-grow, T-L-Ne, T-L-Ne-grow, D-P-Ne-ctx, D-L-Ne-ctx),
    from its latest plateaued run, at the final version, in the variant the record kept;
  - at plies 3 and 4, one per (run, family) of every finished run;
  - new-system runs only;
  - at most 16 a ply.
  The full rule is in the script's docstring and in each spec's description.
- `--depth-anchors K`: the first K references of each anchor ply also play at the plies next to it, outside the
  anchor, to calibrate the depth axis.
- It prints the entries per ply and the CPU estimate. It writes leagues/specs/arenas/NAME.json, with the arena in
  leagues/arenas/ladder/NAME. It runs nothing: `arena.py create` and `run` do that.
- Runs enter the library's index only when build_index.py is run again; only then can they be candidates.
