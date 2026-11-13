# Arenas

An arena rates fixed models that are already trained. It has no lines, no training and no versions. The models
play repeated double round-robins with the random references and get rated, in the same way and in the same files
as an experiment's tournament. Use one to compare models from different experiments, or models that no league
holds, such as a run's early round models.

The script is leagues/arena.py. It only reads leagues.py (for its helpers and settings) and never changes a league.

    python3 leagues/arena.py create SPEC.json                 makes leagues/arenas/<name>/ (or the spec's "dir")
    python3 leagues/arena.py run NAME|DIR [--rounds N] [--threads T]
    python3 leagues/arena.py status NAME|DIR [--vs PLAYER]

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
- references: random, random-1, random-2 and random-3 at plies 0-3, with random as the anchor (Elo 0);
- plies: [2];
- calibration: the experiments' settings (leagues.py DEFAULTS);
- owncal_binary: the binary;
- keep_own_games: false;
- stop: ci95 20, chunk_rounds 10, min_rounds 10, max_rounds 200.

Relative paths are from the repository root. The examples are in leagues/specs/arenas/.

Entries:

- `"version": "<experiment>:<line>@<v>"` is an experiment's version: runs/<line>/<kind><v>.model, with the kind,
  held-out games, calibration settings and tournament binary of that experiment. `<experiment>` is a name in
  leagues/experiments/ or a path. The player name is `<line>@<v>` unless the entry gives a name.
- `"file": PATH` is any model file. It needs `kind` and `name`. `heldout` names a set of held-out games in the
  calibration settings (lr or cnbp); without it the model plays uncalibrated before its own games.
- `plies` (default: the spec's) lists the plies the entry plays at; each ply is one player, named `<name>-<ply>`.
- `positions`, `label` and `note` are only for the reader: status shows positions and label.

`create` copies each model into the arena (models/<name>.<kind>.model), checks the copy's md5, and records the
source, md5 and calibration settings in config.json. A later change to the source does not change the arena.

## Calibration

Every entry plays with the calibration the experiments use, fitted on its own games. The model plays itself 600
games at depth 1 with log-odds softmax at T 0.1, seed 5, under its held-out calibration. The command is
leagues.py's make_own, and the games go to owncal/<name>/. The tournament fits the calibration on them when it
loads the player (`cal:<owncal>/<name>:<kind>:<model>`). It is deterministic, so the arena makes the games again
instead of looking for old ones; experiments delete own games of versions that left their field.

- An experiment version keeps its experiment's calibration. Its own games are made with that experiment's binary
  and settings, so they come out as they did there. The arena checks this: after the first chunk it compares the
  calibration its tournament fitted (tour/tournament.log) with the one the experiment's tournament fitted for the
  version (the experiment's tour/tournament.log). It writes both to calibration.csv with identical yes or NO, and
  status reports them.
- A file entry's own games are made with the spec's calibration settings and owncal_binary. Use the binary and
  settings its source used, to calibrate it as its source would have.

## Files

leagues/arenas/<name>/:

- config.json: the resolved spec, with entries, copies, md5s and settings.
- state.json: status (created, calibration, running, done, paused or failed), round-robins played, the stop
  reason, and the CPU and wall time and widest entry ci95 of each chunk.
- arena.log: what the run did; run.out: the detached run's output.
- calibration.csv: entry, source, expected, arena, identical.
- models/: the copies.
- owncal/: each entry's <name>.json (the play spec and md5) and <name>.arena (the own-games command and its output).
  The own games themselves, in owncal/<name>/, are 10-15 MB an entry. A run removes them when it ends, unless
  keep_own_games is set. The next run that has games to play makes them again, and they come out the same.
- tour/: the tournament's own files, as in an experiment. These are players.txt, retired.txt (empty: an arena's field
  is fixed), games.csv, game_cpu.csv, ratings.csv/.md, head_to_head.csv/.md, elo_history.csv and tournament.log.
  Their formats are the experiments' (FORMATS.md), so tools written for an experiment's tour/ work here. The one
  difference is the binary's: ratings.csv has no retired column when nobody is retired.

The tournament command is the experiments' without `--save-games`, because nothing trains on an arena's games:
`tournament players.txt . --tournaments N --use-probs --anchor random --seed S --no-catch-up --retired retired.txt
--prior-draws 0 --reports-last --sparse-fit --game-cpu`. With `--prior-draws 0` a player is unrated until its
results link it both ways to the anchor. A field of strong models therefore needs references close enough to them.

## Stop rule

`run` plays chunk_rounds round-robins at a time. After each chunk it stops if min_rounds are played and every
entry's ci95 is at most stop.ci95; an unrated entry counts as unbounded. It also stops at max_rounds.
References are not counted. The ci95 is the tournament's (ratings.csv): the 95% interval relative to the rated
players' mean. The rule decides only how many round-robins are played; the games are fixed by the
seed. So two arenas with the same spec play the same games, and stopping later only adds round-robins.
