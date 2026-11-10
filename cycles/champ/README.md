# Training/tournament cycles under the champion rules (cycles/champ/)

The user's redesign of the cycles (2026-11-07), run as a restart. Everything here lives in `cycles/champ/` and the
league in `selfplay/league-champ/`. The first run (`cycles/`, `selfplay/league-cycles/`, its `README.md`) stays as it
was when it stopped, as its record: nothing of it is changed. The lineages' run directories, `cycles/runs/<lineage>/`,
are shared: the restart continues each run's version numbers and never rewrites a file there. Where this README says
"as in the first run", `../README.md` has the details (self-play's measuring and chunks, the round-size rule, the
tournament tool).

The dashboard (http://127.0.0.1:8765) is documented in `DASHBOARD.md`, next to this file.

In short:

- The **field** is each active lineage's current version at plies 1 and 2, its **champion** at ply 2 when that is
  another version, the frozen lineages' champions, and the references.
- **Cycle 1 (seeding)** trains nothing. It picks each lineage's first champion: its latest version or its old peak
  from the first run.
- **Every later cycle**:
  - self-play, 60 CPU-min split evenly among the active lineages;
  - each active lineage's new version enters;
  - 5 double round-robins of the field;
  - the champion test (2 standard errors);
  - patience (32 CPU-min of self-play without a new champion);
  - finishing (frozen at a free milestone, else retired);
  - the own-games step and its gate.
- **Every 25th cycle** is a global tournament: the retired champions rejoin and can be revived.

## The references

| player | what | ply |
|---|---|---|
| random | uniformly random moves; the fit's anchor, at 0 | 0 |
| random-1, random-2, random-3 | random moves searched on a uniform evaluation | 1, 2, 3 |
| ref-d4mix43-1 | F-L-d4mix-c's v43, frozen, calibrated on its own games (below; with the default, on the LR held-out set) | 1 |

- ref-d4mix43-1 replaces the first run's random-4. A depth-4 random search costs about 0.7-1 CPU s a game; an LR model
  at ply 1 costs a few CPU ms.
  - It was picked from three candidates, each at ply 1, by a few hundred games against random-3 and random-4.
  - Fitted against them at their first-run ratings: ref-d4mix43-1 422 ± 35 (random-4 418), F-L-d4mix-c v34 331 ± 34,
    nbmix v25 304 ± 35.
  - Head to head against random-4: v43 scored 42 Elo above it, v34 31 below.
  - Its model file is copied to `cycles/champ/refs/F-L-d4mix-c-lr43.model` at `init`, so it never depends on the run
    directory.
- "Elo vs refs" is an entry's Elo minus the mean of random, random-1 and random-2 in the same fit.
- References never train. Each cycle, each one holds the multiple of 100 nearest its rating (within 50) as a
  milestone (see Finishing).

## Cycle 1: the seeding

1. Each lineage with versions enters with its **latest** version (its run's last) at plies 1 and 2, and its **old
   peak** at ply 2.
   - The old peak is its best ply-2 Elo in the first run's final fit (`selfplay/league-cycles`, retired entries
     included).
   - When the latest is the peak, it is the only candidate.
2. The field (these entries and the references) plays 5 double round-robins. Nothing trains: no self-play, own steps
   or gates.
3. The better of the latest and the peak, by its ply-2 Elo in this fit, becomes the lineage's first champion. The
   other candidate's entries leave the field.
4. When the peak wins, its model file becomes the run's next version (a hard link, or a copy), so training resumes
   from it.
   - For a tree, its training state (`{kind}{N}.state`) comes too: from the run directory or from `cycle-states/`.
   - The latest version's state stays in the run directory, because `--resume` reads the last two.
   - Self-play then plays the copy against the latest version, its predecessor by number.
   - The run's version numbers show the copy: in `results.txt` the round that would have made it is missing.
   - Example: F-L-c's latest is v3146 and its peak v2860. If the peak wins, v3147 is a copy of v2860 and training
     goes on from it.
5. The lineages new in the restart have no versions. They sit out the seeding, train from scratch from cycle 2, and
   their first version in the field becomes their champion. They are MoE-N-L-c, MoE-L-N-c, MoE-N-N-c,
   T-L-Te-merge-c, MoE-8-c and T-L-Ne-share-c.

`cycle.py dry-run` shows the seeding field and each lineage's resume points with their files. `init` refuses to
start if any of those files is missing.

## Every later cycle

The phases are saved in `state.json`, so an interrupted cycle resumes where it stopped.

### 1. Self-play

- **The budget.** B = `cycle_budget_cpu_s` = 3600 CPU s (60 CPU-min) a cycle.
  - It is split evenly among the lineages that can self-play this cycle: active, with their binary, and with a cost.
  - Each lineage's budget is the share plus its carry.
- **Skipping.** A lineage whose budget is under half a round at its cost skips the cycle, and its budget carries over
  whole.
- **The carry.** After a job, carry = budget − CPU used. A job plays whole rounds, so it can overshoot a little; an
  overshoot is paid back next cycle. A failed job carries nothing.
- **Patience** grows by the CPU each job used.
- **Jobs** are as in the first run:
  - the round-size rule: 1000 games a round at depth 0, fewer at greater depths;
  - each job measures its CPU per game over its first rounds (at least 5% of its budget), then plays the rest in
    chunks of about 22%, each re-planned from the last;
  - two jobs at a time, four threads each, nice 10.

### 2. The field

- Each active lineage's newest version enters at plies 1 and 2. It replaces the lineage's current entries, except the
  champion's, which stays.
- Entries that leave the field go to `retired.txt`. They play no more, but their games stay in the fit.

### 3. The tournament

- 5 fresh double round-robins of the field (`--no-catch-up`), with chessai-sparsefit and the flags in config.json's
  `tournament.flags`: `--reports-last --sparse-fit`.
  - From cycle 44 the tournament binary is chessai-hybrid (it adds the MoE-LP-L hybrid kind). It was identity-checked
    against chessai-sparsefit with these flags over every player kind in the field, a retirement and late
    joiners: models/from-scratch/lrprobit/hybrid/tourney.out and tourney2.out (1,033 and 2,553 identical files).
  - chessai-sparsefit is chessai-lrshare plus the report fix. chessai-lrshare, like chessai-tanroute2 before it, is a
    superset of every lineage binary (identity-checked against chessai-probit, the first run's tournament binary, on
    every older kind). The fix is a standalone patch, `models/from-scratch/sparsefit-src/sparsefit.patch`, so a later
    binary can carry it (`git apply` it in that binary's source).
  - At the build it loaded every model file of the seeding field, and the new kinds' models: TAN routers, the NB
    mixtures, `--leaf-share` and `--moe-experts 8`.
  - `--reports-last`: the tables (ratings, head-to-head, Elo history) are written once, after the call's last
    round-robin, not after each one.
  - `--sparse-fit`: the fit and its ± come from block elimination over the round-robins' fields, not from dense n×n
    matrices. The ratings and ± are the same: on the first run's league they agree with the dense fit to 5e-13. It
    takes about a second instead of minutes.
  - Without the two flags, the binary's output is byte-identical to chessai-tanroute2's.
- The games are saved for the own-games step.
- The fit is pure maximum likelihood over all of `games.csv` (retired players included), with random at 0.

### 4. The decisions

- **First champion.** A lineage without a champion takes its first rated version as its champion.
- **The champion test.**
  - It compares the current version's ply-2 entry with the champion's.
  - The current version becomes the champion when it is at least `champion_se` = 2 standard errors of their
    difference above it.
  - The standard error comes from the fit's covariance: the inverse of the information matrix of the
    maximum-likelihood fit, with random pinned at 0. It is exact, by sparse elimination, in seconds.
  - The old champion's entry leaves the field.
- **Patience** is the self-play CPU used since the lineage's last new champion. A first champion counts as new.
  - A lineage whose champion was kept this cycle and whose patience reached `patience_cpu_s` = 1920 CPU s (32
    CPU-min) finishes.
  - At an even split among about 60 lineages (about 60 CPU s each a cycle), that is about 32 cycles without a new
    champion. The share grows as lineages finish.
- **Finishing.** A lineage that finishes is **frozen** if its champion's ply-2 Elo is within `milestones.window` = 10
  of a multiple of 100 that nobody holds; it then holds that milestone. Otherwise it is **retired**.
  - A frozen lineage trains no more. Its champion stays in the field as a fixed player.
  - A retired lineage leaves the field. Its champion comes back only for global tournaments.
  - Milestones held: the frozen champions' claims (permanent), and each reference's nearest multiple of 100.
  - Claims made in one cycle go closest first, a tie by name.

### 5. The own-games step and its gate

These run for each active lineage that has entries in the field.

- **The step.** The current version trains once more on its games from this cycle's round-robins against other
  lineages' entries, with the recipe's labels.
  - Those entries include the champions, the lineage's own champion too.
  - Games against the references don't count.
- **The gate.** The trained model plays 100 games at depth 2 against the version it started from, both calibrated as
  in the league (with `calibration` "own", on their own games: see below).
  - Both sides play the league's policy: log-odds softmax at T 0.1. That is `LEAGUE_POLICY`, the tournament
    binary's defaults; `gate_cmd` passes them to the arena.
  - The gate bug, fixed from cycle 7 (the `staged-c6/` deploy, 2026-11-08 01:16). Before the fix, the gates played
    the arena's default, wld-score-jitter10, which is self-play's policy. That covers cycles 2-6 here and every gate
    of the first run. Their records are left as they are.
  - At a score of at least 0.50 it is kept, as the run's next version. Otherwise it is reverted.
  - Some lineages need their own binary for the step: `own_games.binary_by_lineage`.

### 6. The logs

`cycles.csv`, `totals.csv` and `cycle-NNN.md` (see Records).

## The global tournament (every 25th cycle: 25, 50, ...)

1. Every retired champion rejoins the round-robins at ply 2: retired by finishing, or out again after a global
   tournament, but not retired by hand. It is moved back from `retired.txt`.
2. **Revival.** After the fit, each rejoining champion's score in this cycle's games is compared with its expected
   score:
   - z = (S − E) / sqrt(V + (V · ln10 / 400 · ci0 / 1.96)²);
   - S: its points (draws half);
   - E: the points expected from its rating in the previous cycle's fit (elo0, ± ci0) against each opponent's rating
     in this fit;
   - V = Σ p(1 − p), the games' binomial variance. Draws make the true variance smaller, so this errs towards not
     reviving.
3. **Revived** at z ≥ `revive_se` = 2.0. It trains again from its champion, copied as the run's next version (as at
   the seeding), with patience and carry at 0.
4. **Not revived.** It is frozen if its Elo is within 10 of an untaken milestone ("frozen at the global tournament").
   Otherwise it leaves again ("out again") and rejoins at the next global tournament.

## Calibration on own games (config.json's `calibration`: "own" in this run)

This run's config.json has `"calibration": "own"`. That was the coordinator's call for the switch, under the user's
go. With the default, `"heldout"` (or no key), every calibrated player, and both sides of every gate, are calibrated
on the lineage's held-out set, `cal:HELDOUT:KIND:PATH`. The choice is made before `init` (see "Starting it") and
can't change for a league.

With `"calibration": "own"`, each version is calibrated on its own play instead.

- **Why.** The league plays log-odds softmax at T 0.1 on each entry's calibrated probabilities. On their own play,
  the held-out calibrations have slopes from 0.55 to 2.03 (median 1.16), so entries play at different effective
  temperatures. That moves ratings by about 132 Elo per unit ln(slope ratio), 95% 96 to 169
  (`models/from-scratch/leaguecal/summary.txt`).
- **The recipe.** A version's own games are 600 games of its held-out league spec against itself: log-odds softmax
  at T 0.1 on both sides, ply 1, seed 5, `--use-probs`, played by the tournament binary.
  - The version then plays as `cal:OWN:KIND:PATH`, OWN being `cycles/champ/owncal/<lineage>-v<N>`. The tool fits the
    calibration from those games when it loads the model.
  - `own_calibration` in config.json holds the recipe: dir, seed_dirs, games, depth, seed, policy, temperature.
- **Who.** Every calibrated player: the current versions at plies 1 and 2, champions, frozen champions, rejoining
  retired champions, and ref-d4mix43-1 (key `ref-d4mix43`). The random players are uncalibrated, as before. So the
  champion test compares entries calibrated on their own games.
- **The gate.** Both sides are calibrated on their own games, as in the league.
  - The trained model's own games are made just before its gate (key `.post-<lineage>`) and deleted after it.
  - The version it started from is in the field, so its own games exist already.
  - If a kept model ever enters the field itself (usually self-play's newer version does), its own games are made
    again, the same.
- **When.** Before each cycle's round-robins, the driver makes the own games of each field version that lacks them,
  jobs × threads_per_job (8) at a time, on one thread each. The games are seeded and don't depend on the thread
  count, so a version's own games are the same whenever they are made.
- **Disk.** About 11 MB per version.
  - Before the round-robins, the driver deletes the own games of versions no longer in the field.
  - A retired champion's own games are made again when it rejoins a global tournament.
  - `<key>.json` and `<key>.arena` stay as the record. The first holds the spec the games were played with, the model
    file and its md5. The second is the arena's log.
- **Safety.**
  - Own games are used only while the model file's md5 is the recorded one. A changed file stops the driver.
  - `init` records `"calibration": "own"` in state.json, and `run` stops if config.json disagrees, so one league
    never mixes the two.
- **Seeding.** The seeding field reuses `models/from-scratch/leaguecal/games/` (`own_calibration.seed_dirs`).
  - A version's games there are linked, not played, if their log's first line shows the same spec, policy, ply,
    number of games and seed, and the model file is older than the log.
  - leaguecal made them with this recipe, T 0.1 included (its `survey.py`). The log doesn't show the temperature.
- **Cost, when on.** About 2 own-game sets a cycle per active lineage: its new version, and its gate's trained model.
  - At 1.2-3.1 CPU s a set, that is about 4 CPU-min a cycle with 60 active lineages, and under a minute of wall time.
  - Each load also fits the calibration (0.07-0.2 CPU s), about 0.2 CPU-min a cycle.
  - At a global tournament, each rejoining retired champion adds one set.
  - The seeding cycle makes about 9 sets (about 20 CPU s): at the build, 94 of the field's 102 versions were in
    leaguecal.
- **Peak disk.** About 1.3 GB at cycles 10 and 25, and 1.1 GB at cycle 50, in the cost model's synthetic league. That
  is about 120 own-game sets in the field, and about 100 at cycle 50.

## Commands

    python3 cycle.py status            # the next cycle's plan, each lineage's state
    python3 cycle.py dry-run           # the seeding field, the schedule, resume points and their files; writes nothing
    python3 cycle.py init              # cycle 0: copies the references' files, makes the league and state.json,
                                       # writes cycle-000.md (the seeding plan); no games
    python3 cycle.py run [--cycles N]  # what loop.sh runs, one cycle at a time
    python3 cycle.py freeze NAME       # by hand, between cycles: an active lineage stops, its champion stays (no milestone)
    python3 cycle.py unfreeze NAME     # trains again from its latest version, fresh patience; a milestone it held is free
    python3 cycle.py retire NAME       # after taking its line out of lineages.txt: it leaves, never rejoins
    python3 cycle.py unretire NAME     # trains again from its latest version, fresh patience, its champion back
    python3 cycle.py measure|refit|setup-league|verify|selftest   # as in the first run

The hand commands run between cycles only: touch STOP and wait for the loop to stop.

## Starting it (the switch from the first run)

With `calibration` "own" (see above), set it in `config.json` before step 2, so the dry run and `init` see it.
Keep `models/from-scratch/leaguecal/games/` until the seeding cycle has linked its games. The dry run (step 3) then
lists the seeding field's own games: found in leaguecal, or to make.

1. `touch /home/iv/dev/chessai/cycles/STOP`. The first run's loop stops after its current cycle.
2. Copy these files into `cycles/champ/`: `cycle.py`, `config.json`, `lineages.txt`, `costs.csv`, `league.py`,
   `loop.sh` and `README.md`.
   - Rebuild `lineages.txt` and `costs.csv` from the first run's files as they are then, plus this restart's
     additions, in case those changed meanwhile.
3. `cd cycles/champ && python3 cycle.py dry-run` (writes nothing). Check "Files missing: 0" and the seeding field.
4. `python3 cycle.py init`: `refs/`, `selfplay/league-champ/`, `state.json` and `cycle-000.md`.
5. `setsid nohup ./loop.sh >> loop.log 2>&1 < /dev/null &` from `cycles/champ/`.

## Time per cycle (estimates at the build, 2026-11-07)

| part | CPU | wall |
|---|---|---|
| seeding cycle: 162 players, 130,410 games | 1.3 CPU h | about 11 min |
| a crowded cycle's round-robins: about 185 players, 170,200 games | 1.8 CPU h | about 15 min |
| self-play | 60 CPU-min | about 10 min (2 jobs x 4 threads) |
| gates: 100 games at depth 2 for each of about 60 lineages | about 9 CPU-min | |
| own steps | a few CPU-min | |
| the tournament's tables, once a cycle, plus the driver's champion-test SEs (see below) | | 2 + 3 s at cycle 10, 5 + 9 at 25, 11 + 17 at 33, 16 + 26 at 50 |
| a global tournament: each rejoining champion | about 1,850 games, about 1 CPU-min | |
| `calibration` "own" (an option, off): about 2 own-game sets per active lineage | about 4 CPU-min | under 1 min |

- Game costs come from each seeding player's measured CPU per position at its ply and the first run's plies per game
  by pairing type.
- random-4 in a crowded field would have cost about 21 CPU-min a cycle (1,840 games at 0.67 CPU s); ref-d4mix43-1
  costs about 0.5.
- **The tool's tables.** chessai-tanroute2 refit and rewrote its tables after every round-robin, with dense n×n
  matrices; n counts the retired players too.
  - In the first run's league that is about 115 s of each round-robin now.
  - Here about 120 entries leave each cycle, so the tables would take 0.7 min a cycle at cycle 10, 13 at 25, about 29
    at 33 and 83 at 50.
  - These were measured on a synthetic league shaped like this one. Cycles 33 and 50 are scaled as n³ from cycle 25.
  - With chessai-sparsefit, the one call a cycle takes 2 s at cycle 10, 5 at 25, 11 at 33 and 16 at 50. That is
    mostly reading `games.csv` and summing the games.
  - Its peak memory is 0.2 GB at cycle 10, 0.6 at 25, 0.8 at 33 and 1.3 at 50. The head-to-head table, still n×n in
    memory, is 0.4 GB of that at cycle 50.
  - The driver's own champion-test standard errors (sparse, in Python) take 3 s at cycle 10, 9 at 25, 17 at 33 and
    26 at 50.
- **Patience.** At about 60 CPU s of self-play a cycle per lineage, 1920 CPU s of patience means no lineage can finish
  before cycle 33, so the global tournament at cycle 25 has no retired champions to rejoin.

## Records

For the dashboard and anyone reading the run. All files are in `cycles/champ/` unless a path is given.

### cycles.csv

One row per lineage and cycle, for every lineage in the field that cycle, frozen ones included (retired lineages have
no row, except in a global cycle). Empty means not applicable.

| field | meaning |
|---|---|
| cycle, date, lineage | |
| status | after the cycle: active, frozen or retired |
| version | the version whose entries played (its current version); empty when it has none in the field |
| games, rounds, round_size | this cycle's self-play |
| cpu_s, wall_s, cost_ms | this cycle's self-play: CPU, wall, CPU per game |
| segment_rounds | rounds of the measuring segment |
| start_version, end_version | self-play's versions |
| budget_cpu_s | the self-play budget: share plus carry |
| carry_cpu_s | carried to the next cycle (negative: an overshoot to pay back) |
| patience_cpu_s | self-play CPU since the last new champion |
| entries | its entries in the field this cycle, space-separated |
| elo, ci95, rel_elo | the current version's ply-2 entry: Elo, ± (the tool's 95%), Elo vs refs |
| champion_version, champion_elo, champion_ci95 | its champion after the cycle, in this fit |
| diff, diff_se | the champion test: the current version's Elo minus the champion's, and the standard error of that |
| decision | see below |
| note | the decision in words |
| revive_games, revive_score, revive_expected, revive_z | a global cycle's revival statistic, for a rejoining champion |
| own_games, own_positions, own_cpu_s | the own-games step |
| gate_wins, gate_losses, gate_draws, gate_score, gate_elo, gate_ci95, gate_cpu_s | the gate (the trained model's side) |
| gate | kept, reverted, or why there was no gate |
| next_version | the version the next cycle trains from |

Decisions:

- seeding: "first champion: latest", "first champion: old peak";
- new lineages: "waits for a first version", then "first champion";
- the champion test: "new champion", "champion kept";
- finishing: "finished-frozen", "finished-retired";
- a frozen lineage's row: "frozen";
- a global cycle: "revived", "frozen at the global tournament", "out again";
- other: "waiting for binary", a skipped or failed step's reason.

### totals.csv

One row per cycle:

| field | meaning |
|---|---|
| cycle, date | |
| kind | seeding, normal or global |
| active, frozen, retired | lineages by status after the cycle |
| trained | lineages that self-played |
| field, round_robins, games, tournament_min | the tournament |
| budget_cpu_s, share_cpu_s | B and its even share |
| selfplay_cpu_s, own_cpu_s, gate_cpu_s | CPU used |
| new_champions, finished_frozen, finished_retired | decision counts |
| revived, global_frozen, left_again | a global cycle's outcomes |

### state.json

- `cycle`: the last finished cycle.
- `seeded`: false until the seeding cycle ends.
- `references`, `rel_references`: the reference players, and the ones "vs refs" subtracts.
- `calibration`: "own" if the league was set up with that option, else absent.
- `milestones`: {multiple of 100: holders}.
- `current`: the cycle under way, if any, with its phase.
- `lineages`: {name: record}, with these keys:

| key | meaning |
|---|---|
| status | active, frozen or retired |
| version | its run's last version |
| field_version, entries | its current version in the field and that version's entries |
| champion | {version, entry, since (cycle), elo, ci} |
| patience_cpu, carry_cpu | as in cycles.csv |
| elo, ci, rel, elo_cycle | its current version's last rating |
| seeding | its seeding candidates: latest, latest_old (its first-run rating), peak, peak_entry, peak_elo, peak_ci |
| claim | a frozen lineage's milestone |
| frozen_by, frozen_cycle | frozen_by is finish, global or hand |
| retired_by, retired_cycle | retired_by is finish, global or hand; only finish and global rejoin global tournaments |
| revived | the cycles it was revived in |
| cost | its last measured CPU per game |

### Other files

- `cycle-NNN.md`: each cycle's summary, with what happened and the per-lineage table. `cycle-000.md` is the seeding
  plan.
- `selfplay/league-champ/`: the league, with the files as in the first run.
  - `players.txt` and `retired.txt`, with `NAME DEPTH SPEC` lines.
  - `games.csv`, `ratings.csv` (the tool's ci95 is relative to the rated players' mean), `elo_history.csv` and
    `head_to_head.csv`. Their formats are as in the first run.
    - With `--reports-last` they change once per tournament call (once a cycle), not after every round-robin.
  - `tournament.log`: the tool's lines. With `--reports-last`, each round-robin's line times its games only and has
    no top 3: `tournament 3/5: 34040 games in 300s`. The call's last line adds the tables' time and the top 3:
    `tournament 5/5: 34040 games in 300s, tables in 5s; top: ...`.
  - `models.csv` and `league.md`. With `calibration` "own", models.csv's `calibrated` is "own".
- `owncal/`, with `calibration` "own" only: each field version's own games (`<lineage>-v<N>/`), with
  `<lineage>-v<N>.json` and `.arena` as the record (see "Calibration on own games").
- Names:
  - A lineage entry is `<lineage>-v<N>-<ply>`.
  - The references are random, random-1, random-2, random-3 and ref-d4mix43-1. They don't match `-v<N>-<ply>`, and
    `state.json`'s `references` lists them. ref-d4mix43-1 is a model, not a random player.
- New in the restart, for the dashboard's classification:
  - model kinds moenb, nbgate and nbmix (archetypes MoE-N-L, MoE-L-N, MoE-N-N);
  - `--router tan` (T-L-Te);
  - `--leaf-share` on an LR-leaf tree (T-L-Ne-share-c; from cycle 4 also T-L-Le-share3-c and T-L-Nc-share-c; from
    cycle 28 also T-L-Ne-share3-c);
  - `--moe-experts 8` (MoE-8-c).
- Deploys between cycles:
  - `staged-c3/`, at the cycle 3->4 boundary, added T-L-Le-share3-c and T-L-Nc-share-c, both from scratch.
  - `staged-c6/`, at the cycle 6->7 boundary, is the gate-policy fix (cycle.py only; see "The gate" above). Its
    watcher also checks that the tournament binary's defaults are still `LEAGUE_POLICY`'s, and that every gate
    binary's arena takes the policy flags.
  - `staged-c27/`, at the cycle 27->28 boundary (2026-11-08 12:18), added T-L-Ne-share3-c from scratch:
    T-L-Ne-share-c's line with `--leaf-share 0.3` instead of 0.1, on chessai-sparsefit. The loop restarted as pid
    571237; the replaced files are in `backup-20261108-121813/`.
  - `staged-c28/`, at the cycle 28->29 boundary (2026-11-08 12:53), added FM-d2jit-c from scratch: FM-c's line
    plus `--depth 2`, on chessai-cyc5, with a measured cost row "fm d2 jitter10" (0.222 CPU-s a game). The loop
    restarted as pid 581118; the replaced files are in `backup-20261108-125306/`.
  - `staged-c29/`, at the cycle 29->30 boundary (2026-11-08 13:27), added F-L-d4jit-c from scratch (the user's
    request): F-L-c's line plus `--depth 4` (F-L-d2jit-c's with depth 4), on chessai-cyc5, result labels in its own
    step, cost row "lr d4 jitter10" 3.715 CPU-s a game. The loop restarted as pid 588960; the replaced files are in
    `backup-20261108-132746/`.
  - `staged-c33/`, at the cycle 33->34 boundary (armed 2026-11-08 15:44: STOP touched, watcher pid 617950), adds
    three lineages from scratch under the user's "add whichever models you want": T-L-Te-c (T-L-Te-merge-c without
    `--revise 0`, chessai-tanroute2), T-L-Nc-share-merge-c (T-L-Nc-share-c plus `--revise 0`, chessai-lrshare) and
    D-L-Ne-c (T-L-Ne-c with `--model bdd`, without `--reset-leaves inf`, which the BDD refuses; chessai-cyc5). It also
    adds the first two to `own_games.binary_by_lineage` in config.json. The restarted loop's pid goes to
    `staged-c33/loop.pid`; checks in models/from-scratch/bddcheck/.
  - `staged-c34/`, at the cycle 34->35 boundary (2026-11-08 16:27), added T-L-Le-bal-c from scratch: T-L-Le-c's
    line plus `--error-target balanced`, on chessai-bal (own step too). The loop restarted as pid 634290; checks in
    models/from-scratch/bal/.
  - `staged-c35/`, at the cycle 35->36 boundary (armed 2026-11-08 16:27: STOP touched, watcher pid 634509), adds two
    lineages from scratch: T-L-Ne-bal-c (T-L-Ne-c's line plus `--error-target balanced`, chessai-bal) and
    T-L-Nc-share-l10-c (T-L-Nc-share-c's line with `--split-limit 10`, chessai-lrshare), both in
    `own_games.binary_by_lineage`. The restarted loop's pid goes to `staged-c35/loop.pid`; checks in
    models/from-scratch/bal/ (ne/, l10/, rehearsal/).
  - `staged-c43/`, deployed at the cycle 43->44 boundary (2026-11-08 21:52; armed 21:15, the watcher restarted twice
    for restagings, last pid 699409), adds seven lineages from scratch: four one-factor variants of D-L-Ne-c,
    D-L-Ne-lreg-c, D-L-Ne-skip-c and D-L-Ne-rf16-c on chessai-bddfix and D-L-Ne-skill-c on chessai-bddtrig2;
    T-L-Ne-new-c, T-L-Ne-c's exact line, as their same-age BDT control; and the LR+probit hybrids MoE-LP-L-c and
    MoE-LP-L-hard-c on chessai-hybrid (own steps on the same binaries, `own_games.binary_by_lineage`). The
    tournament binary became chessai-hybrid, flags unchanged. Restarted loop: pid 705046. Checks in
    models/from-scratch/dagdeploy/, models/from-scratch/bddtrig/ and models/from-scratch/lrprobit/hybrid/.
  - `staged-cN/` holds the staged files, `live.md5` (the live files they were staged against) and `deploy.py`, the
    watcher. The watcher waits for the loop to stop on STOP, checks, swaps the files in, runs selftest and status,
    and restarts the loop. On any failure it restores the old files and leaves the loop stopped.
  - `staged-cN/loop.pid` is the restarted loop's pid.
  - `deploy.log` is the watchers' log.
  - `backup-<time>/` holds the files a deploy replaced, with `state.json` as it was then.
