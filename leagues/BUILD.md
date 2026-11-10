# Build notes: the new league engine (engine fork, 2026-11-09)

- 15:10 Started. Spec: models/NEW-DESIGN.md. Formats: leagues/FORMATS.md (written first, 15:15).
- 15:20 New tournament binary models/from-scratch/bin/chessai-hybrid4g (md5 f934bbbc...), from a copy of
  hybrid4-src (models/from-scratch/hybrid4g-src). The only change is `tournament --game-cpu`: each game's thread
  CPU time (clock_gettime CLOCK_THREAD_CPUTIME_ID around play_game; a game is played on one thread) goes to
  OUT/game_cpu.csv. Identity check (leagues-smoke/idcheck): hybrid4 and hybrid4g, flag off and on, give
  byte-identical games.csv, ratings, tables and saved games (8 players, 2 round-robins). cargo test tournament: 13
  passed.
- 15:25 leagues/make_recipes.py writes leagues/recipes.txt from the live registry (read-only): the 70 lineages active
  at cycle 64 give 54 training recipes and 63 experiment models (a recipe plus distinct self-play settings).
  - Three d2jit merge trees train with a smaller --split-threshold, so they are distinct recipes:
    T-L-NLs-merge-st100k, T-N-NNs-merge-st300k and T-N-Nc-merge-st200k.
- 15:30 Untrained models (version 0) come from `train` on a round holding one empty game directory (0 positions;
  `train` refuses an empty round, and --rounds 0 saves nothing).
  - Checked for all 54 recipes (leagues-smoke/untrained/t.py): each loads and plays calibrated in hybrid4g. Own
    calibration of a constant model gives slope 0, and it plays uniformly.
  - F-L-mix fails, because --label-depth needs positions. Its version 0 is made without the label flags, which only
    change how positions are labelled, not the model.
- leagues/costs.csv is a copy of cycles/champ/costs.csv, used for self-play round sizes and first estimates.
- 15:40 make_recipes.py: recipes now group by train binary too (no change in the 54), and a solo lineage takes its
  non-solo twin's train binary (F-P-tau0-solo-c had defaulted to chessai-cyc5-labels, which can't train probit). New
  last column `heldout` (the live lineages.txt's held-out set), used to play own calibration games as the live league
  does.
- 15:45 leagues/leagues.py written (create, add, run, status, stop, index, selftest) and leagues/repeat.sh (one cycle
  per driver process; stops on STOP, failure or an experiment's end).
- 15:50 selftest (leagues-smoke/selftest.log): 33 rule checks pass (names, ply-2 filter, champion test and its SE
  against a dense inverse, anchors: lock-in, ties, margin, 5-cycle release, replacement, one interval per entry;
  global-round selection and CPU equalisation; waiting on a fake global batch). Resume: a 2-recipe global league
  (F-L, T-L-Ne) interrupted at 10 points (every phase, mid-tournament, mid-training, mid-records) reproduces the
  uninterrupted run's games, ratings, models and records exactly; an experiment's self-play interrupted after a chunk
  finishes with each self-play round once. 31 CPU s.
- 15:47-15:50 Smoke part 1 (leagues-smoke/run-smoke-1.sh): the global league (F-L, T-L-Ne, M-LP-L; 2 rounds a
  cycle, 60 calibration games) ran 4 cycles while the experiment three-options (F-L x self-play, local, global;
  min_rounds 2) ran 3. The experiment waited 25 s for the first batch, then copied rounds.
  - Found: sizing a cycle from last cycle's CPU per round gave the local lines 2.4 times the global lines' data in
    cycle 2, because the field grew from 2 to 3 entries a line when champions split from current versions. Now each
    pair is priced by its class from last cycle's games.
  - Found: the last cycle copied global rounds although it trains nothing. It now copies none.
  - Found: events carried the records' time; each now keeps its own.
- 15:51 `add` fixed (it took the league lock twice). Smoke part 2 (run-smoke-2.sh):
  - F-P-tau0 joins the global league at cycle 5;
  - three-options is made again and runs while the global league plays cycles 5-7;
  - sp-vs-local (F-L-d2jit, T-L-Ne x self-play, local) runs under repeat.sh, is stopped with STOP after its first
    cycle, and is restarted.
- 15:55 Smoke part 2 results (leagues-smoke/run-smoke-2.log, global.out, three-options.out, sp-vs-local.out;
  leagues-smoke/equalisation.py prints what each line trained on). All three leagues ran to the end without errors.
  - Late entry: F-P-tau0 entered the global league untrained at cycle 5 as G.0.0 and was crowned at cycle 6 (G.1.1).
  - Anchors after 7 cycles (global/anchors.csv): [100, 200) F-L@1F-1, [200, 300) M-LP-L@2F-1, [300, 400)
    M-LP-L@3F-1, [400, 500) F-P-tau0@1F-1. Each now rates below its lock-in rating (152 to 135, 235 to 179, 361 to
    326, 463 to 434), as the design expects. The smoke's ratings are too noisy (95% CI 48 to 96) to say by how much.
  - Data equalisation: CPU s of the qualifying games each line trained on, and the ratio to the local lines.

    | Experiment, cycle | local | self-play | global |
    |---|---|---|---|
    | sp-vs-local, 1 | 5.3, 5.3 s | 1.03, 1.00 | |
    | sp-vs-local, 2 | 11.3, 11.3 s | 0.97, 1.01 | |
    | three-options, 1 | 5.0 s | 1.11 | 0.99 |
    | three-options, 2 | 10.0 s | 0.96 | 1.28 |

    - Self-play's over- or underspending carries to the next cycle. In three-options' cycle 2 the budget was 9.98 s
      of this cycle's games less the 0.52 s overspent in cycle 1. The job used 17.83 s: training 8.27 s and
      generation 9.56 s, so 0.10 s overspent carries on.
    - The global line's 1.28: the estimate priced an experiment round at 4.3 s from cycle 1's games, and the 3 rounds
      played cost 3.3 s each. Rounding wasn't the cause: the copied 12.8 s came to 2.98 estimated rounds, and 3 were
      played. Games got cheaper as the models trained: ply-2 model against ply-2 model cost 58, 40 and 28 ms a game
      in cycles 1-3 (310, 298 and 229 plies). Untrained models' long games are the design's "early cycles, not
      priced" item.
  - STOP during sp-vs-local's cycle 2: the driver finished the cycle, deleted STOP and exited; repeat.sh ended with
    "STOP found: stopped" (status stopped, phase idle). The restart ran cycle 3 and ended with "experiment done".
  - Both experiments' last cycle copied nothing and trained nothing. three-options played 5 rounds in it.
  - The live league stayed untouched throughout (its loop and dashboard kept running).
- 16:00 `status ROOT` prints "1 model", not "1 models". Quick selftest after it: 33 rule checks pass.
- 16:10-16:58 Follow-ups (the user, via the coordinator): prices instead of measured CPU, the global carry,
  experiment names E.<S|L|G>, a fixed self-play budget for self-play-only experiments, re-runnable price
  calibration, and model variants, sweeps and replicates. All in leagues.py and leagues/calibrate.py; FORMATS.md has
  the changelog (16:40 to 16:58).
  - Prices: every budget and sizing decision now uses a game's price, a fixed formula over its plies and the
    constants of a versioned table, leagues/prices.csv. Measured CPU is still recorded beside it, for information.
    - A game's price: for each side, `us_per_game + moves x us_per_ply` of its (context, kind, search depth). A
      self-play round of G games and P plies: `2 G us_per_game + P us_per_ply`.
    - The per-game term is mine, beyond the suggested per-ply form: per ply alone, short games came out at 0.58 of
      their measured CPU and long ones at 1.27 (the opening's plies have more moves to search than the endgame's).
      With it: 1.11 and 1.05.
    - A league copies its version's constants into its config at create and keeps them. An experiment prices the
      global rounds it copies with its own constants.
    - `leagues.py prices SPEC.json` (calibrate.py) measures, fits and appends a new version with a report,
      leagues/prices/<version>.md. `--fit-only` refits from the measurements already made; `--check` writes the
      report to the work dir without adding a version; `--measure-only` measures and stops.
  - P0 (provisional; the procedure proven on a small sample, as asked, on the old binaries): 5 players at plies 1
    and 2 (3 copied from the live run dirs, 2 untrained), 182 games, plus the old smoke leagues' 6074 games with
    measured CPU; 5 self-play measurements (lr at depths 0, 2 and 4; a mature tree at depth 0). Calibration CPU
    about 14 s. Results: leagues/prices/P0.md; summary in the CPU section below.
  - Fixed in the fit: a constant could be stuck at the floor (probit at ply 1 came out 0) because a negative
    first estimate was held for good. Each iteration now starts with every term free and starts from weights
    1/measured; probit's per-ply constant at ply 1 is 4.4 us (8.0 per ply alone).
  - The global option's mismatch carries: `global_carry_s` (state, totals) adds each cycle's copied qualifying
    price minus the local lines' qualifying price; the next copy aims at the local data minus the carry.
  - Self-play-only experiments: `selfplay_budget_s` priced s per line per cycle, default 120. Experiments with
    local or global lines keep the budget tied to the local data's price; `create` refuses the field for them.
  - Self-play rounds are priced from the output's `round N: +W -B =D, P plies` lines, kept in
    runs/<line>/sp/plies.csv. A round saved without its line (a kill between the two) is rolled back and replayed.
  - Games per self-play round are fixed in the config at create (the live league's round-size rule from
    leagues/costs.csv), so a later edit of costs.csv can't change a running experiment.
  - Variants: an experiment model may be {"model", "set", "sweep", "to", "replicates"}; flags are checked against
    the binaries' --help at create. Example in "How to create" below.
- 16:58 selftest: 66 rule checks pass, among them the price formula, versions, the copy plan with a carry, the
  round estimate, the self-play plan in priced s, the budget rule, variants and their seeds, and recovering
  self-play counts from a log. The replay test (it replaces the resume test): two roots, each a global league
  (F-L, T-L-Ne) and an experiment (F-L under self-play, local and global), run in step for 3 cycles; root b is
  interrupted 18 times (every phase of both leagues, mid-tournament, mid-training, mid-self-play, mid-records). Both
  give the same games, ratings, models (md5), self-play rounds (plies.csv, ladder, results) and records (160
  items). 66 + 124 CPU s.
- 16:58-17:00 Smoke 3 (leagues-smoke/run-smoke-3.sh, .log), for the dashboard: the old smoke leagues moved to
  leagues-smoke/old-1609/ (the P0 spec's include paths follow them); new ones in leagues-smoke/ on prices P0:
  global (F-L, T-L-Ne, F-P-tau0; 2 rounds a cycle; 4 cycles), three-options (F-L x self-play, local, global; 3
  cycles) and lr-sweep (self-play only, budget 2 priced s: F-L x --lr-rate 0.05, 0.1 x 2 replicates; 3 cycles),
  global cycle k then the experiments' cycle k. All ran to the end.
  - three-options' data per line (priced s, self-play / local / global): cycle 1 2.88 / 2.86 / 2.87, cycle 2
    6.84 / 6.87 / 7.07; global carry 0.0 then 0.2 s.
  - lr-sweep: each line played 44-46 rounds of 20 games a cycle for its 2.00 s; carries within 0.03 s. Replicates
    differ only in `--seed` (21, 22) and the training seed; the two rates' models differ.

- 17:11 recipes.txt: models F-N and F-P added (the coordinator's request for the planned experiments: experiment 4
  has F-L, F-P and F-N under self-play and local; the per-family sweeps of probit tau and NB smoothing build on F-P
  and F-N). make_recipes.py takes a new EXTRA list of lineages beyond the cycle-64 snapshot: F-N-c (retired live at
  cycle 57) and F-P-c (frozen live since cycle 37), with their binaries and arguments as they are. F-N joins recipe
  F-N (F-N-d2jit's, the same training); F-P is a new recipe. The regenerated file differs from the old one only by
  the 2 rows and 2 comment lines. Checked end to end (leagues-smoke/extra-check: F-L, F-P, F-N under self-play and
  local, 2 cycles): version 0, own calibration, tournament, training and self-play all ran for both. 40 CPU s.
- 17:20 Fast binaries (the coordinator's go, after models/from-scratch/fastbin/ proved them byte-identical):
  make_recipes.py maps every row's binaries through leagues/binaries-fast.txt (its old -> new table; the BY ROW list
  checks the result: all 65 rows match). recipes.txt now names 5 binaries: chessai-hybrid4g (114 of the 130 binary
  fields), chessai-bddtrig2-q9 (8), chessai-bal-q9 (4), chessai-dag-q9 (2), chessai-presplit-q9 (2). No other field
  changed; comment lines at the top name each live binary and its replacement. Leagues made before keep theirs
  (config.json).
  - Replay on the new binaries (leagues-smoke/replay-fast): the replay test passes again (root b interrupted 18
    times = root a), and root a equals the old binaries' root a (leagues-smoke/selftest-full/replay/a) in all 25 +
    135 items: 641 + 1027 game lines, ratings, 9 + 119 model and state files (md5), 110 self-play rounds (plies.csv,
    ladder, results) and every record, leaving out only the binary paths in lines.csv (the measured columns are left
    out as in the replay test). lines.csv confirms the runs used chessai-hybrid4g. CPU 121 user + 4.5 sys s.
- 17:25-17:40 Prices P1 (leagues/prices/P1.md, its spec P1.spec.json, measurements in leagues-smoke/pricecal-P1):
  now the last version in prices.csv, so new leagues take it.
  - Tournament (chessai-hybrid4g `--game-cpu`): 33 players at plies 1 and 2 plus the four references, one round
    robin per ply. Players: version 0 of 12 recipes, and 21 models copied from the live run dirs (2 early, 19
    mature), covering every recipe kind; trees and DAGs from 21 KB to 2.4 MB.
  - Each round robin was played three times with the same seed, interleaved with the other ply and the self-play
    runs: 7992 games, the same games each time.
  - Noise shows as shifts of the whole machine: the same games cost 0.91 to 1.28 times as much in another repeat
    (load 7 to 13 while the live league ran), and every kind moved together. The constants from one repeat alone
    are 0.83 to 1.22 of the three repeats'. P1 averages the three.
  - Self-play: 25 runs at the live round sizes (depth 0: 1000 games, 1: 860, 2: 40, 4: 10) on the recipe binaries,
    1 to 24 CPU s each. A first pass of 2 to 100 games a run measured 0.04 to 0.5 s each, too little against the
    output's 0.01 s timings, and was replaced (kept in sp-small/).
  - calibrate.py: tournament `repeats` and a report section on them; the report's calibration CPU now comes from
    the work dir's records (a refit had said 0 s); a copied player's model keeps a `.source` file naming the
    version copied (the report had named the run dir's newest version at report time); self-play records keep each
    round's model size (`round_model_bytes`), for a size term later.
  - P1's errors, priced / measured CPU (tournament):
    - all 1.00;
    - by stage: untrained 1.09, early 1.07, mature 0.84;
    - by length: short 1.02, mid 0.87, long 1.15; early long games 1.14 (6 games), untrained long 1.14;
    - by repeat: 1.02, 1.11, 0.89.
  - Self-play runs, priced / measured: flat and mixture models 0.60 to 1.34. Scratch trees are 1.77 (bdt) and 2.33
    (bdd); mature trees are 0.62 and 0.56.
  - The full selftest on P1 and the fast binaries passes: 66 rule checks, and the replay test with 25 + 201 items
    and 176 self-play rounds. CPU 113 user + 4 sys s.
- 18:05-18:21 Fixed-point binaries and prices P2 (the coordinator's go; nothing launched).
  - Switch: make_recipes.py maps every row's binaries through leagues/binaries-fast.txt and then
    leagues/binaries-fx.txt. Its map reader takes md5-checked table rows (`old | md5 | new | md5 | ...`; both files'
    md5s must match) or `old -> new` lines, and skips everything else. In recipes.txt the 57 rows on hybrid4g (114
    fields) now name chessai-hybrid4gfx (md5 a3e6d4e6...). The 8 side-branch rows keep their q9 builds, as
    binaries-fx.txt's BY ROW and UNCHANGED lists say. No other field changed. The driver's default tournament
    binary is chessai-hybrid4gfx.
  - Replay (leagues-smoke/replay-fx): the replay test passes on hybrid4gfx (root b, interrupted 18 times, equals
    root a). Root a also equals a hybrid4g reference (leagues-smoke/replay-h4g) in all 25 + 201 items: 641 + 1027
    game lines, ratings, 9 + 185 model and state files, 176 self-play rounds and every record. Only the binary paths
    in lines.csv and the measured columns are left out. CPU 58 + 3 s (fx) and 106 + 4 s (reference); the straight
    run took 30 s on hybrid4gfx and 53 s on hybrid4g.
  - The reference had to be re-run. Between 17:42 and 18:09, someone other than this fork deleted most of
    leagues-smoke: the 17:20-17:40 references (selftest-full, replay-fast, selftest-P1), the smoke 3 leagues and
    old-1609. pricecal-P0 and pricecal-P1 are intact. The reference ran on P1 from a temporary copy of the code
    whose recipes still named hybrid4g; the copy was removed after.
  - Prices P2 (leagues/prices/P2.md, its spec P2.spec.json, measurements in leagues-smoke/pricecal-P2) is the last
    version in prices.csv, so new leagues take it.
    - Tournament: P1's 33 players (the same model files) at plies 1 and 2, three interleaved repeats. These are the
      same 7992 games as P1 (games.csv identical). CPU P2 / P1 is 0.99 at ply 1 and 0.93 at ply 2 (median game
      0.92). Games with a flat model gain most: lr 0.90 and 0.74 (plies 1 and 2), cnbp 0.89 and 0.75, probit 0.94
      and 0.86. In the fitted constants, a 60-move flat side at ply 2 costs 0.29 (lr), 0.33 (cnbp) and 0.70
      (probit) of P1's. Trees, DAGs, mixtures and FM stay within 0.82-1.06.
    - Self-play: 43 runs at the live round sizes on the recipe binaries. lr, probit and cnbp ran at depths 0, 1, 2
      and 4, from scratch and from the latest live versions (F-P and F-N at depths 1 and 4 by an item's `depth`).
      FM ran at depth 2. Trees, the DAG, HT and the mixtures ran at depth 0; bdt and hybrid also at depth 2.
    - Each kind and depth's self-play ratio is fitted on its mature runs (`selfplay_fit_stages`), the stage
      experiments spend most cycles in. Scratch runs are only checked, priced / measured:
      - lr 0.74-1.31, probit 1.03-1.52, cnbp 0.74-1.10;
      - mixtures 0.88-1.03, HT 1.09, FM 1.27;
      - scratch trees 2.35 (bdt) and 4.99 (bdd), the size effect of open question 4;
      - all 20 untrained runs together, 1.23.
    - Tournament errors: all 1.00; by stage untrained 1.10, early 1.06, mature 0.83. The same games cost 1.36 and
      0.84 of repeat 1 at ply 1 and 1.05 and 1.07 at ply 2: the machine's load moves them.
    - costs.csv: five rows `lr d0 jitter10` to `lr d4 jitter10` (`refresh_costs`): P2's lr self-play prices at the
      mature runs' plies per game, which are those runs' measured generation CPU (depth 3 derived). Self-play
      rounds at anchor 1000 are now 1000, 750, 70, 10 and 10 games at depths 0-4 (were 1000, 860, 40, 10, 10). The
      live league reads cycles/champ/costs.csv, not this file.
    - Calibration fixes: a tree version is copied only with its own and its previous version's states. The live run
      keeps states sparsely; T-L-Ne's newest version had none, and its self-play failed. The copy also checks that
      size and mtime didn't change while it copied.
    - Calibration CPU 446 s (tournaments 239 s, self-play 208 s).
  - Quick selftest on P2: 66 rule checks pass.
- 18:22-18:45 The first batch's five specs (nothing launched): leagues-smoke/batch1/specs/, with root leagues/, so
  `leagues.py create leagues-smoke/batch1/specs/<spec>.json` makes them there once the user decides.
  - 1a lr-settings: F-L with train --lr-rate 0.0125, 0.025, 0.05 (the default) and 0.1, and F-L with train
    --label-depth 2 --label-result 0.5. Local, 500 cycles of 5 rounds; 5 lines.
  - 1b probit-settings: F-P with train --probit-tau 0, 1e-6, 1e-5 (the default) and 1e-4. Local, 500 cycles of
    5 rounds; 4 lines.
  - 1c nb-settings: F-N with train --alpha 0.1, 1 (the default) and 10. Local, 500 cycles of 5 rounds; 3 lines.
  - 1a-1c become 100 cycles of 25 rounds by setting `cycles` 100 and `rounds_per_cycle` 25.
  - 3 selfplay-depth: F-L, F-P and F-N with self-play --depth 0, 1, 2 and 4, and a second F-L at depth 2 (#2:
    self-play seed 22). Self-play only, 60 priced s a line a cycle, 100 cycles; 13 lines. Rounds of 1000, 750, 70
    and 10 games. F-L's depth lines have the live F-L-d1jit, d2jit and d4jit's self-play arguments.
  - 4 leaf-sources: F-L twice (#2: seeds moved), F-P and F-N, each under self-play and local, 100 cycles; 8 lines.
  - Checks: all five were created in a scratch root (leagues-smoke/batch1/check) and give the lines above. Each
    also ran 2 cycles in another scratch root (leagues-smoke/batch1/mini) at nice 19, 2 threads and 1 job. Spec 3
    ran at 4 priced s a line a cycle and without its depth-4 lines, which P2 ran.
  - In the checks every line made version 0, calibrated, played, and trained or self-played; nothing failed.
    The label variant's training labels by a depth-2 search: 5.8 s a cycle, against 0.6 s for plain training.
  - CPU a cycle in the checks' cycle 2, where the fields have their full size (3 entries a line and the 4
    references), and over a whole run at that rate:

    | Spec | Lines | Tournament | Training | Self-play | Calibration | A cycle | Run |
    |---|---:|---:|---:|---:|---:|---:|---:|
    | 1a lr-settings | 5 | 16.0 s | 8.1 s | | 4.8 s | 29 s | 500 cycles: 4.0 h |
    | 1b probit-settings | 4 | 20.9 s | 0.8 s | | 5.2 s | 27 s | 500 cycles: 3.7 h |
    | 1c nb-settings | 3 | 10.5 s | 0.4 s | | 3.2 s | 14 s | 500 cycles: 2.0 h |
    | 3 selfplay-depth, estimated | 13 | 115 s | | 760-880 s | 16 s | 890-1010 s | 100 cycles: 25-28 h |
    | 4 leaf-sources | 8 | 40.7 s | 3.1 s | 136.7 s | 9.2 s | 190 s | 100 cycles: 5.3 h |

    - Spec 3 is scaled from its reduced check: 9030 games a cycle (43 entries) at the check's 12.9 ms a game, and
      13 x 60 priced s of self-play, which cost 0.98-1.13 times their price in the check, with the training inside
      self-play. Its depth-4 lines will likely cost less than their price early on (scratch runs in P2: 0.66-0.91).
    - Spec 4's self-play lines each get the price of the cycle's local data, 28 priced s in cycle 2.
    - At 100 cycles of 25 rounds, 1a-1c would take 3.5, 3.2 and 1.6 h (calibrating 100 versions a line instead of
      500), against 4.0, 3.7 and 2.0 h.
    - These are early-cycle rates. Later cycles change with game lengths. The leagues record their own CPU in
      totals.csv.
  - CPU of the checks: 636 s (1c 24 s, 1b 45 s, 1a 51 s, 4 295 s, 3 221 s).

## How to create and run a league (nothing is launched; the user decides)

Files: `leagues/leagues.py` (the driver), `leagues/repeat.sh` (the loop), `leagues/recipes.txt` (models and recipes,
from `leagues/make_recipes.py`), `leagues/prices.csv` (the price constants, by version; `leagues/calibrate.py`
writes them), `leagues/costs.csv` (self-play round sizes), `leagues/FORMATS.md` (the files).

1. Write a spec. The global league takes recipe names (recipes.txt's 2nd column):

       {"kind": "global", "root": "/home/iv/dev/chessai/leagues", "models": ["F-L", "T-L-Ne", "M-LP-L"],
        "threads": 4, "jobs": 2, "nice": 10}

   An experiment takes model names (1st column) and options:

       {"kind": "experiment", "root": "/home/iv/dev/chessai/leagues", "name": "sp-vs-local",
        "models": ["F-L-d2jit", "T-L-Ne"], "options": ["self-play", "local"], "cycles": 20,
        "threads": 2, "jobs": 2, "nice": 10}

   Any key of DEFAULTS in leagues.py can be overridden (rounds_per_cycle 5, champion_se 2, patience_cycles 20,
   anchors, calibration games 600, selfplay round_anchor_games 1000, min_rounds 5, wait_poll_s 60, seed).
   With "global" in options, global_source defaults to ROOT/global. `prices_version` picks a prices version
   (default: the last in prices.csv).

   A self-play-only experiment with a sweep and replicates (4 lines: two learning rates, two seeds each):

       {"kind": "experiment", "root": "/home/iv/dev/chessai/leagues", "name": "lr-sweep",
        "models": [{"model": "F-L", "sweep": {"--lr-rate": [0.025, 0.05]}, "replicates": 2}],
        "options": ["self-play"], "cycles": 20, "selfplay_budget_s": 120}

   - A model item is a name, or {"model": NAME, "set": {flag: value}, "sweep": {flag: [values]} (one flag, or two
     for a grid), "to": "auto" | "train" | "selfplay" | "both" (or per flag {flag: ...}), "replicates": N}.
     `"auto"` (the default) puts a flag into each argument list whose binary takes it. A value of true is a bare
     flag; null (or false) removes the flag from the recipe.
   - Every flag must be accepted by the binary it goes to (checked against its --help at create); the driver's own
     flags (--seed, --rounds, --games, --threads, --resume, --init, --model, --start) can't be set.
   - Model ids: `F-L+lr-rate=0.025#2`; labels `F-L lr-rate=0.025 #2`.
   - Replicate r moves the self-play `--seed` (and `--moe-seed`/`--fm-seed` for kinds that have them) by r - 1,
     and the training seed by 100003 x (r - 1). Replicate 1, and models without replicates, keep the recipe's.
   - `selfplay_budget_s` (priced s per line per cycle, default 120) is for self-play-only experiments; with local
     or global lines the self-play budget is the local data's price, and `create` refuses the field.
2. `python3 leagues/leagues.py create SPEC.json` makes ROOT/global/ or ROOT/experiments/NAME/ and updates
   ROOT/index.json. Nothing runs yet.
3. Run it detached in its own loop (one driver process per cycle):

       setsid nohup /home/iv/dev/chessai/leagues/repeat.sh LEAGUE_DIR >> LEAGUE_DIR/repeat.log 2>&1 < /dev/null &

   or a set number of cycles in the foreground: `python3 leagues/leagues.py run LEAGUE_DIR --cycles N`.
4. `python3 leagues/leagues.py stop LEAGUE_DIR` stops it after the current cycle (or while it waits). Killing the
   driver (SIGTERM) is also safe at any time: it terminates its children, and the next `run` resumes the cycle at
   its phase. The global league's cycles replay exactly.
5. `python3 leagues/leagues.py status ROOT` (every league) or `status LEAGUE_DIR` (one).
6. `python3 leagues/leagues.py add ROOT/global RECIPE...` (no driver running): new recipes enter untrained next cycle.
7. `python3 leagues/leagues.py selftest --dir DIR` (about 2 CPU min, mostly the replay test; `--quick` skips the
   games: 1 s).
8. `python3 leagues/leagues.py prices SPEC.json` calibrates a new prices version (see leagues/prices/*.spec.json
   and calibrate.py's docstring). A league keeps the version it was created with.

Several leagues can run at once, each with its own threads/jobs/nice. An experiment with the global option reads
the global league's batch under a shared lock and waits when every round of it is used.

## Deviations from models/NEW-DESIGN.md and choices it leaves open (for the user)

1. Experiment version names are `E.<S|L|G>.<major>.<v>` by option (the user's choice, 16:10).
2. A version number counts the line's training steps. That is the line's cycle number since it entered, unless a step
   fails or a self-play line's budget is under half a round; then the same version plays again.
3. An experiment's last cycle trains nothing and copies no global rounds: no later cycle would rate the result. It
   plays `rounds_per_cycle` rounds.
4. Version 0 is made by `train` on one empty game. F-L-mix's is made without its label flags, which need positions; an
   untrained model doesn't depend on its labels. The first step (v0 to v1) trains from scratch, which is the same model.
   A self-play line's run also starts from scratch, since `--init` is refused by TAN and FM.
5. Own calibration games are played as the live league plays them: with the recipe's held-out calibration (the live
   lineages.txt's heldout set: lr or cnbp), 600 games at ply 1, T 0.1, seed 5. The held-out sets are old offline
   games, used only to make those games' play reasonable. Open question below.
6. Training order: every model's `train --seed` is the league's train seed plus the cycle, so all models shuffle the
   same positions the same way. Today each recipe passes its own `--seed 21`.
7. Anchors start at the [100, 200) interval. There is none for [0, 100) or below 0; the references cover the bottom.
   - An anchor is a new entry, `<line>@<v>F-1`, rated from the next cycle on its own games. Its lock-in uses the
     version's ply-1 rating, as decided.
   - A cycle counts as outside when the rating is below lo - 25 or above lo + 125. An unrated cycle leaves the streak
     as it is.
   - Once given up, an interval stays given up, even if the anchor drifts back, until a new entry lands in it. That
     can happen in the same cycle.
   - A retiring line's current ply-1 entry can still lock an interval.
   - A version anchors one interval at most.
8. Global-round selection compares the copied rounds' qualifying price (what the global lines train on) with the
   experiment rounds' qualifying price (what the local lines train on), both priced by the experiment.
   - An experiment round is priced over this cycle's field, each pair at the mean moves of its pair class (model or
     random, by ply) in last cycle's games (in the first cycle, the global batch's), so a field that grows when
     champions split from current versions is priced right.
   - The mismatch carries (`global_carry_s`), as the user chose: copy need = min_rounds x estimate - carry (at least
     one round), rounds played = round((copied + carry) / estimate), at least min_rounds when covered.
9. Edge case: a batch that is entirely unused but too small to cover `min_rounds` experiment rounds is taken whole,
   and the experiment plays the rounds it covers (at least 1). A partly used batch that can't cover makes the
   experiment wait. Without this, an experiment bigger than the global league would wait forever.
10. Self-play budgets and chunks are priced: a chunk is about 25% of the budget at the mean price of this cycle's
    rounds so far (last cycle's for the first chunk; one round if none is known). Over- and underspending (in
    priced s) carries to the next cycle's budget, as in the live league. The measured generation CPU recorded
    beside it is the job's CPU split by the wall time of its phases (playing and labelling on the job's threads,
    training on one); subtracting wall times went to 0 under load.
11. Repeatability: both leagues replay bit for bit, interrupted or not, which the replay test checks. Nothing
    measured enters a decision.
12. STOP is honoured between cycles and while an experiment waits; the driver deletes it when it stops on it. SIGTERM
    at any point is safe: the driver kills its children, and the next run resumes the cycle at its phase.
13. Patience counts a cycle whose decision isn't a new champion, including `unrated`. A line retires when the count
    reaches 20, with decision `retired`, and doesn't train in that cycle.
14. When the current version is the champion (after a failed step), there is no test: `champion-kept`.
15. Added beyond the directive:
    - `leagues.py add` brings recipes into the global league; they enter untrained at the next cycle.
    - New columns `selfplay_carry_s` (cycles.csv) and `tournament_proc_cpu_s` (totals.csv).
    - A `batch` phase. It publishes the batch right after the tournament, so waiting experiments go on before the
      global league trains.
16. Prices (16:10 follow-up):
    - Constants are per context: the tournament binary's games and each self-play binary's generation differ
      (self-play cost 0.76 of the tournament's lr ply-1 constants at depth 0 and 2.2 times its ply-2 ones at depth
      2 in P0).
    - The formula has a per-game term besides the per-ply one (progress note 16:10-16:58).
    - No model-size term yet. P1 shows trees need one (open question 4): their games cost more per move as the tree
      grows: per move, about 4 to 6 times at ply 2 from 21 KB to 1-2 MB.
    - Unmeasured constants are derived: a tournament kind from the mean of the measured kinds at that ply; a
      self-play (kind, depth) from the kind's tournament constants times the self-play/tournament ratio (the
      kind's own, else pooled over kinds at that depth, else interpolated geometrically between measured depths).
      Each row's `source` says which.
    - An experiment prices the global rounds it copies with its own version, so leagues on different versions
      still compare like with like.
17. Variants (16:30 follow-up): `"to": "auto"` (the default) puts a flag where the binaries take it, refusing a
    flag that would only change version 0 (train-only flags in a self-play-only experiment). Replicate seeds as in
    "How to create". Games per self-play round are fixed in the config at create.

## CPU used by this build (all test runs at nice 19)

| Item | CPU |
|---|---|
| Rust build of hybrid4g and its cargo test | about 5 min |
| Identity check, hybrid4 against hybrid4g | about 5 s |
| Untrained models of all 54 recipes, loaded and played | about 1-2 min |
| selftest runs | about 1 min |
| Smoke part 1: global cycles 1-4; three-options' first run | 56 s; about 70 s |
| Smoke part 2: global cycles 5-7; three-options; sp-vs-local | 56 s; 67 s; 96 s |
| Follow-ups: P0 calibration (182 games, 5 self-play runs) and refits | about 15 s |
| Follow-ups: price-shape exploration (fits only) | under 5 s |
| Follow-ups: selftest with the replay test (2 roots x 2 leagues x 3 cycles) | 124 s |
| Follow-ups: smoke 3 (global 4 cycles, two experiments 3 cycles each) | 147 s |
| Follow-ups: F-N and F-P checked in a 2-cycle experiment | 40 s |
| Fast binaries: the replay test re-run on them (leagues-smoke/replay-fast) | 125 s |
| P1: measurements (3 x 2 round robins, 25 self-play runs, a first small self-play pass) | 429 s |
| P1: fits, reports and the size-term study (fits only) | about 2.5 min |
| P1: the full selftest on P1 | 117 s |
| Fixed point: the replay test on hybrid4gfx (leagues-smoke/replay-fx) | 62 s |
| Fixed point: the hybrid4g reference, re-run (leagues-smoke/replay-h4g) | 110 s |
| P2: measurements (3 x 2 round robins, 43 self-play runs) | 447 s |
| P2: fits, checks, comparisons, quick selftest | about 25 s |
| Batch 1: the five specs' 2-cycle checks (leagues-smoke/batch1/mini) | 636 s |
| Total | about 55 min |

The leagues' own records (totals.csv) agree with the processes' measured CPU within 2%:

| League | Cycles | Tournament | Training | Self-play | Calibration | Total |
|---|---|---|---|---|---|---|
| global | 7 | 91.3 s | 3.9 s | 0 | 14.2 s | 109.4 s |
| three-options | 3 | 34.7 s | 0.4 s | 27.1 s | 4.4 s | 66.6 s |
| sp-vs-local | 3 | 32.3 s | 0.6 s | 56.1 s | 6.0 s | 95.0 s |

## Open questions for the user

Settled at 16:10: anchors start at 100; the global option's mismatch carries; an experiment's last cycle trains
nothing; leagues-smoke/ stays until launch; experiment names E.<S|L|G>.

1. Own-calibration games are played with the old offline held-out sets (games/heldout-lr-d2-s555 and
   games/heldout-cnbp-d2-s555), as the live league plays them. Keep that (my pick: it is the live rule), or play them
   with the version's previous own calibration, so no outside data enters?
2. Anchors shared with the global league (open in the design): not built. Experiments have only the four random
   references. If wanted, an experiment would add the global league's current anchors as fixed players.
3. The CPU split between the global league and experiments (open in the design): each league's spec sets its
   threads, jobs and nice (defaults 4, 2 and 10).
4. A model-size term in the price. P1 says trees need one; not built (a format change).
   - Evidence: P1 fits each player its own constants. Against those, the kind's constants misprice a tree's
     games by 0.50 to 2.14 times (log-rms 0.41). Flat models and mixtures are mispriced 0.67 to 1.61 times
     (log-rms 0.18).
   - By model size (report): at ply 2, trees under 50 KB are priced at 1.27 (bdt) and 1.37 (bdd) of their
     measured CPU. Trees over 400 KB are priced at 0.78 to 0.79 (bdt) and 0.76 (bdd).
   - Self-play is mispriced the same way: scratch trees at 1.77 to 2.33, mature ones at 0.56 to 0.62.
   - Hoeffding trees barely move: 1.09 small, 0.89 large.
   - Proposed form: one more constant, `us_per_ply_doubling`, per (context, kind, depth). A side's per-move price
     becomes `us_per_ply + us_per_ply_doubling x log2(max(1, model bytes / 20 KB))`.
     - Fitted on P1's games, it brings the trees to 0.60-1.75 (log-rms 0.23), level with the other kinds.
     - The ply-2 terms come out at 62 us per doubling for bdt (base 132), 112 for bdd (base 121) and 13 for ht
       (base 136).
     - A linear term in bytes does worse (log-rms 0.28).
     - Fitted for every kind, it also catches the 8-expert MoE (priced 0.72), taking all players from log-rms
       0.30 to 0.18.
   - What it needs:
     - the model bytes of every entry, recorded when the entry is made (entries.csv, the global batch), so prices
       never read model files later;
     - the model bytes each self-play round starts from (plies.csv);
     - the fit and the report (calibrate.py). P1's tournament games already carry their models' sizes; the
       self-play records keep them from now on, so a P2 needs only the self-play runs again (about 3 CPU min).
   - Still off after it, by the report's measure: the DAG D-L-Nc-c (0.75; its routing, a recipe effect, not size)
     and the early T-L-Ne-c version 20 at ply 2 (1.28).
