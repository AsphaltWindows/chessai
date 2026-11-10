# New league system: file formats

Changes (newest first):
- 2026-11-09 18:30: the fixed-point binaries and prices `P2`. No format change for leagues.
  - leagues/recipes.txt: make_recipes.py maps every row's binaries through leagues/binaries-fast.txt and then
    leagues/binaries-fx.txt (its table rows, checked by md5). The 57 rows on chessai-hybrid4g (114 binary fields)
    now name models/from-scratch/bin/chessai-hybrid4gfx; the 8 side-branch rows keep their q9 builds. No other
    field changed. The comment lines name each live binary and its rebuilds in turn.
  - New leagues' tournament binary (config.json `tournament.binary`) is chessai-hybrid4gfx. Leagues made before keep
    their binaries. The replay test's leagues give the same games, ratings, models, self-play rounds and records on
    chessai-hybrid4gfx as on chessai-hybrid4g.
  - Prices `P2` is in leagues/prices.csv. It is the last version, so new leagues take it; leagues made on P0 or P1
    keep theirs. Its tournament is P1's (the same 33 players and games) on chessai-hybrid4gfx. Its self-play runs
    start from scratch and from the latest live versions, at the live round sizes. Each kind and depth's self-play
    ratio is fitted on its mature runs; the scratch runs are only checked. Report: leagues/prices/P2.md.
  - leagues/costs.csv has five more rows, `lr d0 jitter10` to `lr d4 jitter10`, priced by P2 (not measured; their
    `models` field says so). The last row of a signature wins, so new leagues' self-play rounds are 1000, 750, 70,
    10 and 10 games at depths 0 to 4 (were 1000, 860, 40, 10 and 10). A league fixes its sizes at create (its
    self-play lines' `selfplay_games`).
  - In the calibration spec only:
    - a self-play item may set `depth` (its self-play `--depth`);
    - `selfplay_fit_stages` (a list of stages) fits each kind and depth's self-play ratio on its runs of those
      stages, where it has some;
    - `refresh_costs` appends the costs.csv rows.
  - A copied tree version (a kind that keeps states) is taken only with its own and its previous version's states,
    all four files older than 30 s.
- 2026-11-09 17:40: prices `P1` is in leagues/prices.csv. It is the last version, so new leagues take it; leagues
  made on P0 keep P0. Calibrated on the fast binaries: the tournament binary's per-game CPU (chessai-hybrid4g
  `--game-cpu`) in one round robin per ply (1 and 2) of 33 players (version 0 of 12 recipes, and 21 models copied
  from the live run dirs: 2 early, 19 mature; model files of 8 KB to 2.4 MB) and the four random references,
  played three times with the same seed, interleaved (7992 games); and 25 self-play runs on the recipe binaries at
  the live round sizes. Report: leagues/prices/P1.md. No format change for leagues. In the calibration only:
  specs may set `tournament.repeats`; a self-play measurement record has `round_model_bytes`; a copied player's
  model has a `.source` file naming the version copied. The price still has no model-size term (an open question
  in BUILD.md): at ply 2, trees under 50 KB are priced at about 1.3 times their measured CPU and trees over 400 KB at
  about 0.8.
- 2026-11-09 17:25: leagues/recipes.txt runs on the fast binaries. make_recipes.py maps every row's
  `train_binary` and `selfplay_binary` through leagues/binaries-fast.txt: 114 of the 130 fields become
  models/from-scratch/bin/chessai-hybrid4g, the rest chessai-bddtrig2-q9 (8), chessai-bal-q9 (4), chessai-dag-q9
  (2) and chessai-presplit-q9 (2). Comment lines at the top name each old binary and its replacement. No other field
  changed; the format is unchanged. The replay test's leagues, re-run on the new binaries, give the same games,
  ratings, models, self-play rounds and records as on the old ones (leaving out only the binary paths in lines.csv,
  and the measured columns, as the replay test does). Leagues made before the switch keep their old binaries
  (config.json).
- 2026-11-09 17:11: leagues/recipes.txt has two more rows (65 models of 55 recipes), added by make_recipes.py's new
  `EXTRA` list of lineages taken though not active at cycle 64: model `F-N` (recipe `F-N`, kind cnbp, from F-N-c,
  retired live at cycle 57) and model `F-P` (a new recipe `F-P`, kind probit, from F-P-c, frozen live since cycle
  37), the default flat NB and probit leaves, with their lineages' binaries and arguments as they are (F-N: self-play
  chessai-cyc5, no --depth; F-P: chessai-probit, default tau). Every other row is unchanged; two comment lines name
  the extras. The format is unchanged.
- 2026-11-09 16:58: LANDED: the 16:40, 16:45 and 16:50 entries are in the driver, and the sections below
  describe them. The selftest checks the rules, and a replay test runs a global league and an experiment with
  self-play, local and global lines twice (straight, and interrupted 18 times): the same games, ratings, models,
  self-play rounds and records. One difference from the plan: cycles.csv gets no new `selfplay_games` column (it
  had one; it now counts the games the self-play output reports). Leagues made before this have no `prices` in
  their config: they can be read (status, index) but not run. The prices version now in leagues/prices.csv is
  `P0`, provisional (a small sample on the old recipe binaries); `P1` follows the switch to the fast binaries.
  The smoke root has new leagues in these formats (see Roots); the old ones moved to `leagues-smoke/old-1609/`.
- 2026-11-09 16:50: PLANNED, amends the 16:40 entry before it lands: the price gets a per-game term.
  - A game's price: each side's `us_per_game` plus its moves times its `us_per_ply`, summed over the two sides. A
    self-play round of G games and P plies: 2 G `us_per_game` plus P `us_per_ply`. (Per ply alone, short games
    were priced at 0.58 of their measured CPU and long ones at 1.27; with the per-game term 1.11 and 1.05.)
  - leagues/prices.csv: new column `us_per_game` before `us_per_ply`:
    `version, context, kind, depth, us_per_game, us_per_ply, source, games, plies, note`.
  - config.json: `prices` = {"version": "P1", "us_per_game": {"<context> <kind> <depth>": number}, "us_per_ply":
    {same keys}}.
  - runs/<line>/sp/plies.csv: `round, games, plies, price_s`.
  - config.json: each self-play line has `selfplay_games` (games per self-play round, fixed at create from the
    round-size rule).
  - cycles.csv: no new column; `selfplay_games` counts the games the self-play output reports.
- 2026-11-09 16:45: PLANNED (names fixed now): experiment model variants, sweeps and replicates.
  - An experiment spec's `models` may hold objects besides plain model names: a base model (recipes.txt's model
    column) with flag overrides (`set`), a sweep of one flag or a grid of two (`sweep`), applied to `train`,
    `selfplay` or `both` arguments (`to`), and `replicates` (the same model with different seeds).
  - Model ids: the base name, then `+<flag>=<value>` per override (the flag without its dashes), then `#<r>` for
    replicate r when there are several. Lines are `<model id>.<sp|loc|glb>` as before, for example
    `F-L+lr-rate=0.025#2.loc`. Ids use only letters, digits and `_ . + = # -`.
  - Labels (for display): the base name, then ` <flag>=<value>` per override, then ` #<r>`, for example
    `F-L lr-rate=0.025 #2`.
  - config.json lines: new keys `label`, `base_model`, `train_overrides`, `selfplay_overrides` (lists of
    [flag, value]; value true for a bare flag, null for a removed one), `replicate`, `seed_offset`. `model` is the
    model id. `train_args` and `selfplay_args` are the effective arguments.
  - lines.csv: new last columns `label`, `base_model`, `train_overrides`, `selfplay_overrides` (each override as
    `flag=value`, separated by spaces), `replicate`, `seed_offset`.
  - cycles.csv: new last column `label`.
  - index.json: `models` lists model ids; new `labels` (the same order).
  - Seeds: replicate r has `seed_offset` r - 1. Its training seed is the league's train seed plus the cycle plus
    100003 x seed_offset; its self-play `--seed` and any `--moe-seed` or `--fm-seed` are the recipe's (or the
    binary's default) plus seed_offset. Lines without replicates keep the recipe's seeds.
- 2026-11-09 16:40: PLANNED (names fixed now; the driver follows, and a later entry says when it lands). Priced CPU
  replaces measured CPU in every budget and sizing decision; experiment version names; fixed self-play budgets.
  - A game's price is the sum, over its plies, of the moving side's `us_per_ply` for its kind and search depth, from
    a versioned table (leagues/prices.csv). Prices come only from games' plies and fixed constants, so they replay
    exactly. Measured CPU stays recorded beside them, for information only.
  - leagues/prices.csv (new): `version, context, kind, depth, us_per_ply, source, games, plies, note`. `context` is
    `tournament` or `self-play`; `kind` is a recipe kind (lr, bdt, ...) or `random` (the references, depth = their
    ply). A version's rows never change; a recalibration (`leagues.py prices`) adds a new version.
  - config.json: `prices` = {"version": "P1", "us_per_ply": {"<context> <kind> <depth>": number}} (the constants
    the league uses, frozen at create). `selfplay_budget_s`: a self-play-only experiment's fixed priced budget per
    line per cycle (default 120); null for other leagues.
  - index.json: each league also has `prices_version` and `selfplay_budget_s`.
  - state.json: new `global_carry_s` (experiments with the global option: priced s by which the global lines' data
    leads the local lines' so far; negative when it trails). Lines' `carry_s` is priced s; `sp_cost` becomes
    `sp_round_price_s`.
  - rounds.csv: new last columns `price_s`, `ply2_price_s`.
  - totals.csv: new last columns `price_s`, `ply2_price_s`, `global_price_s`, `global_ply2_price_s`,
    `global_carry_s`, `data_price_target_s`, `selfplay_price_s`, `prices_version`.
  - cycles.csv: new last columns `data_price_s` (the priced CPU of the games the line trained on), `selfplay_price_s`
    (the priced generation of its self-play this cycle), `selfplay_plies`. `selfplay_budget_s` and
    `selfplay_carry_s` are now priced s. `data_cpu_s`, `selfplay_cpu_s` and `selfplay_gen_cpu_s` stay measured.
  - copied.csv: new last columns `price_s`, `ply2_price_s`.
  - batch/games.csv: new last column `price_s`. batch.json: each round also has `price_s` and `ply2_price_s`; new
    top-level `prices_version`.
  - runs/<line>/sp/plies.csv (new, self-play lines): `round, plies, price_s`, one row per self-play round.
  - Version names: experiment versions are `E.<S|L|G>.<major>.<v>` (S self-play, L local, G global), for example
    `E.S.2.17`; no F. The global league keeps `G.<major>.<v>[F]`. This changes `version_name`, `champion_name` and
    `new_name` in cycles.csv, and `version_name` in history.csv, entries.csv, events.csv and lines.csv.
  - Self-play-only experiments (options exactly ["self-play"]): each line's budget per cycle is `selfplay_budget_s`
    plus its carry. Experiments with local or global lines keep the budget tied to the price of the local data;
    `create` rejects a spec that gives `selfplay_budget_s` for them.
- 2026-11-09 16:00: matched to the driver as built. Additions only; no column was renamed or moved.
  - index.json: each league also has `phase`.
  - Phases: a new `batch` phase (global, right after the tournament). `waiting` is also a status.
  - config.json: new keys `min_rounds`, `wait_poll_s`, `anchors.lowest`, `calibration.heldout`; each line has
    `recipe` and `heldout`. The global league's config also changes through `leagues.py add`.
  - state.json: new keys `status`, `anchor_entries`, `last_round_ply2_cpu_s`, `last_rounds`; lines have
    `sp_version`, `carry_s` and `sp_cost`; `current` is described.
  - events.csv: `date` is when the event happened (copy and wait events come before the cycle's end).
  - cycles.csv: new last column `selfplay_carry_s`. An experiment's last cycle trains nothing.
  - totals.csv: new last column `tournament_proc_cpu_s`.
  - entries.csv: the next cycle's new entries are listed too; `in_field` and `left_cycle` are defined below.
  - New files and dirs: `trainsets/`, `.nogames/`, `batch.new/`, `.lock`, `batch.lock`; `repeat.log` (if the loop
    script runs).
  - The tournament binary is `models/from-scratch/bin/chessai-hybrid4g` (hybrid4 plus `--game-cpu`).
  - `leagues/recipes.txt` is described (11 fields).
- 2026-11-09 15:15: first version, written before the driver; the engine fork keeps this file current.

These are the files the engine (`leagues/leagues.py`) writes and the dashboard reads. Everything is plain CSV, JSON
or Markdown. CSVs have a header row. Times are local, `YYYY-MM-DD HH:MM:SS`. Elo is a plain number, never written with
a "+" sign. A field that doesn't apply is empty.

## Roots

A root holds one global league and any number of experiments:

```
<root>/
  index.json                  every league in this root (see below)
  global/                     the global league (a league dir)
  experiments/<name>/         one league dir per experiment
```

- Real root: `/home/iv/dev/chessai/leagues/` (the code lives there too). It stays empty until the user creates a
  league.
- Smoke root: `/home/iv/dev/chessai/leagues-smoke/`, same layout, made by the end-to-end smoke. It is left in place
  for the dashboard. Remade at 16:58 in the current formats (prices P0): `global` (F-L, T-L-Ne, F-P-tau0; 4
  cycles), `experiments/three-options` (F-L under self-play, local and global; 3 cycles) and
  `experiments/lr-sweep` (self-play only, `selfplay_budget_s` 2: F-L at --lr-rate 0.05 and 0.1, 2 replicates each;
  3 cycles). The earlier smoke leagues (formats before 16:40, no prices) are in `leagues-smoke/old-1609/`.

### index.json

```json
{"updated": "2026-11-09 16:05:12",
 "leagues": [
  {"name": "global", "kind": "global", "dir": "global", "status": "running", "cycle": 12,
   "cycles_planned": null, "models": ["F-L", "T-L-Ne"], "options": ["league"], "lines": 2,
   "created": "...", "updated": "...", "pid": 12345, "phase": "tournament", "labels": ["F-L", "T-L-Ne"],
   "prices_version": "P1", "selfplay_budget_s": null},
  {"name": "lr-sweep", "kind": "experiment", "dir": "experiments/lr-sweep", "status": "running",
   "cycle": 3, "cycles_planned": 20, "models": ["F-L+lr-rate=0.025#1", "F-L+lr-rate=0.025#2"],
   "options": ["self-play"], "lines": 2, "created": "...", "updated": "...", "pid": 23456, "phase": "self-play",
   "labels": ["F-L lr-rate=0.025 #1", "F-L lr-rate=0.025 #2"], "prices_version": "P1",
   "selfplay_budget_s": 120.0}
 ]}
```

- `models` are model ids (see Names), `labels` their display labels in the same order.
- `prices_version`: the prices the league runs on (frozen in its config). `selfplay_budget_s`: a self-play-only
  experiment's priced budget per line per cycle; null otherwise.

- `status`:
  - `created`: no cycle run yet;
  - `running`: a driver is inside a cycle;
  - `waiting`: an experiment waits for unused global rounds;
  - `stopped`: not running (STOP, or no driver);
  - `done`: an experiment finished its planned cycles;
  - `failed`: the last cycle failed (see the league's driver.log).
- `pid` is the driver's pid while one runs, else null. The index writer already reports `stopped` for a `running`
  or `waiting` league whose pid is gone.
- `cycle` is the last completed cycle (0 before the first).
- `phase` is the league's state.json phase (`idle` between cycles).
- Rewritten atomically, under an fcntl lock on `<root>/.index.lock`, by every driver at each phase change; also by
  `leagues.py index ROOT` and `leagues.py status ROOT`. Each league's entry comes from its own config.json and
  state.json, so any writer produces the same file.

## A league dir (both kinds)

```
<league>/
  config.json        how the league was created; changes only when `leagues.py add` adds a recipe (global)
  state.json         where it is: last cycle, current phase, per-line state, anchors, used global rounds
  lines.csv          one row per line: its definition and its latest status (rewritten each cycle)
  cycles.csv         one row per line per cycle (appended)
  totals.csv         one row per cycle (appended)
  rounds.csv         one row per round played (appended)
  events.csv         champion, retirement, anchor, copy and wait events (appended)
  history.csv        every field entry's rating at the end of every cycle (appended)
  entries.csv        every entry ever in the field, with its current role (rewritten each cycle)
  anchors.csv        global only: the anchor table by 100-Elo interval (rewritten each cycle)
  copied.csv         experiments with the global option: global rounds copied (appended)
  cycle-NNN.md       a readable report of cycle NNN
  driver.log         the driver's progress lines
  tour/              the tournament tool's directory: players.txt, retired.txt, games.csv, game_cpu.csv,
                     ratings.csv/.md, elo_history.csv, head_to_head.csv/.md, tournament.log
  runs/<line>/       a line's model files: <kind><v>.model (and <kind><v>.state for trees), v = version number
  runs/<line>/sp/    a self-play line's self-play run directory; runs/<line>/sp/plies.csv prices its rounds
  runs/<line>.log    the line's train and self-play commands and their output
  owncal/            each version's own calibration games, owncal/<line>@<v>/ with <line>@<v>.json and .arena
  pool/              this cycle's saved games while the cycle runs (deleted after training)
  batch/             global only: the latest complete batch, kept until the next replaces it (see below)
  batch.new/         global only: the next batch while it is being linked (renamed to batch/ under the lock)
  copied/            experiments with the global option: the rounds copied for this cycle (deleted after training)
  trainsets/         this cycle's training sets while training runs: trainsets/local/round0/ and
                     trainsets/global/round0/, one link per game named <t>~<white>~<black> (deleted after the cycle)
  .nogames/          an empty game, which `train` turns into an untrained version 0
  .lock              the driver's fcntl lock (one driver per league)
  batch.lock         global only: the batch's fcntl lock
  STOP               if present, the driver stops at the next safe point: between cycles, or while an experiment
                     waits for global rounds; the driver deletes it when it stops on it
  repeat.log         the loop script's output, if it was started as shown in leagues/repeat.sh
```

- Model files are kept for every version (anchors and champions point at them). A tree's training state
  (`<kind><v>.state`) is kept for the current version only.
- `owncal/<line>@<v>/` (the games) is deleted once the version leaves the field; `owncal/<line>@<v>.json` (how they
  were made) and `owncal/<line>@<v>.arena` (the arena's log) stay.

### Names

- A league has **lines**. A global-league line is one model (recipe), named as the recipe (`F-L`, `T-L-Ne`). An
  experiment line is a (model, option) pair: `<model>.sp` (self-play), `<model>.loc` (local training) or
  `<model>.glb` (global training).
- An experiment's **model id** is recipes.txt's model name, then `+<flag>=<value>` for each flag override (the flag
  without its dashes; `+<flag>` for a bare flag, `+<flag>=off` for a removed one), then `#<r>` for replicate r when
  the spec asks for several: `F-L`, `F-L+lr-rate=0.025`, `F-L+lr-rate=0.025#2`. So a line is for example
  `F-L+lr-rate=0.025#2.loc`. Ids use letters, digits and `_ . + = # -` only. Its **label** (for display) has spaces
  instead: `F-L lr-rate=0.025 #2`. Line, entry and directory names carry the id (`runs/F-L+lr-rate=0.025#2.loc/`,
  `F-L+lr-rate=0.025#2.loc@3-2`), so a URL or query string holding one must be percent-encoded (`#` starts a
  fragment, `+` reads as a space in a query).
- A line's **version number** `v` counts its training steps: 0 is the untrained model it enters with, and v goes up
  by 1 every cycle it trains. It never resets. The version number alone identifies a version, and every file keys
  on it. A training step that fails, or a self-play line whose budget is under half a round, leaves v as it was, and
  the same version plays again. An experiment's last cycle trains nothing, since no cycle would rate the result.
- **Version names** for display:
  - global league `G.<major>.<v>`; a frozen ply-1 anchor `G.<major>.<v>F`;
  - experiments `E.<S|L|G>.<major>.<v>`, the letter the line's option (S self-play, L local, G global), for example
    `E.S.2.17`; experiments have no anchors, so no F.
  - `major` is the number of champions crowned so far, not counting the first, untrained one. The crowned version
    takes the new major: if v=5 is crowned as the line's first new champion, it is `G.1.5` from then on, and v=6 is
    `G.1.6`. A version that played before its crowning shows its old name (`G.0.5`) in that cycle's row.
  - history.csv and cycles.csv's `version_name` show the name as it was while the version played (before the
    cycle's decision). lines.csv, entries.csv and anchors.csv show current names.
- **Entry names** (players in `tour/players.txt`) key on the version number, so a crowning renames nothing:
  - `<line>@<v>-1` and `<line>@<v>-2`: version v at ply 1 and ply 2;
  - `<line>@<v>F-1`: the frozen ply-1 anchor copy of version v (global only);
  - `random`, `random-1`, `random-2`, `random-3`: the references (random moves, searching at ply 0-3). `random` is
    pinned at Elo 0.
- A line's champion entry is the ply-2 entry of its champion version, `<line>@<c>-2`. It stays in the field while
  that version is champion. When the current version is the champion, one ply-2 entry serves both roles.

### config.json

```json
{"name": "global", "kind": "global", "created": "...", "root": "/home/iv/dev/chessai/leagues-smoke",
 "seed": 1, "nice": 19, "threads": 4, "jobs": 2,
 "tournament": {"binary": "/abs/chessai-hybrid4gfx", "flags": ["--reports-last", "--sparse-fit"], "game_cpu": true},
 "rounds_per_cycle": 5,
 "cycles": null,
 "champion_se": 2.0,
 "patience_cycles": 20,
 "anchors": {"step": 100, "margin": 25, "out_cycles": 5},
 "references": ["random", "random-1", "random-2", "random-3"],
 "calibration": {"games": 600, "depth": 1, "seed": 5, "policy": "log-odds-softmax", "temperature": 0.1},
 "train": {"seed": 1, "qualify": "a model at ply 2"},
 "selfplay": {"round_anchor_games": 1000, "chunk": 0.25},
 "global_source": null,
 "min_rounds": 5, "wait_poll_s": 60,
 "selfplay_budget_s": null,
 "prices": {"version": "P1", "us_per_game": {"tournament lr 2": 6870.6, "...": 0},
            "us_per_ply": {"tournament lr 2": 93.93, "...": 0}},
 "lines": [{"line": "F-L", "model": "F-L", "option": "league", "kind": "lr", "archetype": "F-L",
            "recipe": "F-L", "heldout": "lr",
            "train_binary": "/abs/chessai-cyc5-labels", "train_args": ["--use-probs", "--seed", "21"],
            "selfplay_binary": null, "selfplay_args": null, "entered_cycle": 1}]}
```

An experiment line (here a replicate of a variant, self-play):

```json
{"line": "F-L+lr-rate=0.025#2.sp", "model": "F-L+lr-rate=0.025#2", "label": "F-L lr-rate=0.025 #2",
 "base_model": "F-L", "recipe": "F-L", "option": "self-play", "kind": "lr", "archetype": "F-L",
 "train_binary": "...", "train_args": ["--use-probs", "--lr-rate", "0.025"],
 "selfplay_binary": "...", "selfplay_args": ["--model", "lr", "--use-probs", "--seed", "22", "...",
                                             "--lr-rate", "0.025"],
 "selfplay_games": 1000, "heldout": "lr", "entered_cycle": 1,
 "train_overrides": [["--lr-rate", "0.025"]], "selfplay_overrides": [["--lr-rate", "0.025"]],
 "replicate": 2, "seed_offset": 1}
```

- `kind`: `global` or `experiment`.
- `cycles`: an experiment's planned number of cycles; null for the global league.
- `patience_cycles` and `anchors`: global only (null in experiments). `anchors.lowest` (100): no anchor below it.
- `options` of a line: `league` (global league), `self-play`, `local`, `global`.
- `global_source`: an experiment's path to the global league dir it copies rounds from (null without the global
  option).
- `min_rounds` (5): with the global option, the copied rounds' ply-2 price must cover this many experiment rounds.
- `wait_poll_s` (60): how often a waiting experiment looks for a new batch.
- `calibration.heldout`: the held-out sets (by name) own calibration games are played with.
- `lines[]` also has `recipe` (the training recipe; equal to `model` in the global league) and `heldout`.
- `lines[].entered_cycle`: the league cycle in which the line first played (global: `add` makes later ones).
- `prices`: the prices version the league runs on, its constants copied at create (`<context> <kind> <depth>` keys,
  CPU microseconds; see leagues/prices.csv). The league keeps them whatever later versions say.
- `selfplay_budget_s`: a self-play-only experiment's (options exactly `["self-play"]`) budget per line per cycle,
  in priced s of generation (default 120); null in other leagues, whose self-play lines get the price of the cycle's
  local data. `create` refuses it for them.
- Experiment lines also have `label`, `base_model` (recipes.txt's model), `train_overrides` and
  `selfplay_overrides` (lists of [flag, value]; value true for a bare flag, null for a removed one), `replicate`
  (1, 2, ...) and `seed_offset` (replicate - 1). `train_args` and `selfplay_args` are the effective arguments, the
  overrides and the replicate's seeds applied.
- `lines[].selfplay_games` (self-play lines): games per self-play round, fixed at create (the round-size rule:
  `selfplay.round_anchor_games` at depth 0, fewer at deeper search).

### state.json

```json
{"cycle": 12, "phase": "tournament", "status": "running", "pid": 12345, "updated": "...",
 "current": {"cycle": 13, "started": "...", "phase": "tournament", "rounds_planned": 5, "t0": 60, "...": "..."},
 "lines": {"F-L": {"status": "active", "version": 12, "major": 2, "champion": 9, "patience": 3,
                   "crowned": [0, 4, 9], "retired_cycle": null, "sp_version": 0, "carry_s": 0.0,
                   "sp_round_price_s": null}},
 "anchors": {"1500": {"entry": "F-L@7F-1", "line": "F-L", "version": 7, "source": "F-L@7-1", "locked_cycle": 8,
                      "locked_elo": 1531.2, "outside_streak": 0, "available": false, "elo": 1522.0, "ci95": 21.3}},
 "anchor_entries": ["F-L@7F-1"],
 "used_global_rounds": [[12, 57], [12, 58]],
 "global_wait": null,
 "last_round_ply2_cpu_s": 80.4,
 "last_rounds": [55, 5],
 "global_carry_s": 0.0}
```

- `phase`: `idle` (between cycles), `copy`, `waiting`, `calibration`, `tournament`, `batch` (global), `decide`,
  `train`, `self-play` (experiments), `records`. A global cycle runs calibration, tournament, batch, decide, train,
  records; an experiment's runs copy (with the global option), calibration, tournament, decide, train, self-play,
  records.
- `status`: as in index.json (`created`, `running`, `waiting`, `stopped`, `done`, `failed`).
- `current`: the running cycle (null between cycles), what a resumed run continues from: `field` (this cycle's
  entries: entry, ply, role, line, version, frozen, spec), `rounds_planned`, `t0` (its first tournament index),
  `rounds` (per-round stats once played), `cal` (calibration CPU by line), `decisions` (by line), `trained` and
  `selfplay` (per line, as they finish), `copied` (experiments with the global option), `events`, `wait_s`,
  `data_cpu_target_s` (measured) and `data_price_target_s` (priced: what the decisions use).
- `lines.<line>.version`: the version that plays in the next (or running) cycle. `champion` is the champion's
  version number. `crowned` lists every champion version, the first one included.
- `patience`: cycles since the line's last new champion (global only).
- `used_global_rounds`: (global cycle, tournament index t) pairs an experiment has copied. They are never reused.
- `anchor_entries`: every anchor entry ever made (a version anchors one interval at most).
- `lines.<line>.sp_version` (self-play lines): the self-play run's last model (`runs/<line>/sp/<kind><n>.model`);
  `carry_s`: the priced s the line under- (positive) or over-spent (negative) against its budget, added to next
  cycle's budget; `sp_round_price_s`: the mean price of its last cycle's self-play rounds (sizes its first chunk).
- `global_carry_s` (experiments with the global option): the priced s by which the global lines' qualifying data
  has led the local lines' over the cycles so far (negative: trailed). The copy step aims at this cycle's local
  data minus it.
- `last_round_ply2_cpu_s`: last cycle's mean ply-2 CPU per round; `last_rounds`: [first tournament index, rounds] of
  last cycle (an experiment prices its next round at those games' mean moves by pair class).
- `global_wait`: while an experiment waits, `{"since", "since_ts", "cycle"}`.

### lines.csv (rewritten each cycle)

`line, model, option, kind, archetype, status, version, version_name, champion, champion_name, patience,
entered_cycle, retired_cycle, train_binary, train_args, selfplay_binary, selfplay_args, label, base_model,
train_overrides, selfplay_overrides, replicate, seed_offset`

- The last six as in config.json; overrides are written `flag=value`, separated by spaces.

- `status`: `active` or `retired` (global, after patience). Experiment lines are always `active` until the
  experiment is done.

### cycles.csv (one row per line per cycle)

`cycle, date, line, model, option, status, version, version_name, entry1, elo1, ci1, entry2, elo2, ci2,
champion, champion_name, champion_entry, champion_elo, champion_ci, diff, se_diff, z, decision, new_name, patience,
train_games, train_positions, train_cpu_s, data_cpu_s, selfplay_budget_s, selfplay_cpu_s, selfplay_gen_cpu_s,
selfplay_rounds, selfplay_games, cal_cpu_s, next_version, note, selfplay_carry_s, data_price_s, selfplay_price_s,
selfplay_plies, label`

- `version` and `version_name`: the version that played this cycle, as named while it played.
- `entry1`/`entry2` with `elo1`/`ci1`, `elo2`/`ci2`: its ply-1 and ply-2 entries' ratings in this cycle's fit (`ci`
  is the tournament's 95% ±).
- `champion*`: the champion before this cycle's decision.
- `diff`, `se_diff`, `z`: current@2 minus champion@2, the standard error of that difference from the fit's
  covariance, and `diff / se_diff`.
- `decision`:
  - `first-champion`: the line's first cycle; its version is its first champion;
  - `new-champion`: z >= champion_se, and `new_name` is its new name;
  - `champion-kept`;
  - `unrated`: a rating is missing;
  - `retired`: the line's patience ran out (global).
- `patience`: after the decision.
- `train_games`, `train_positions`, `train_cpu_s`: the training step after the tournament (global, local and global
  lines).
- `data_price_s`: the priced CPU of the games this line trains on in this cycle (global and local lines: the
  qualifying games; self-play lines: their own rounds). `data_cpu_s`: the same games' measured CPU (information
  only).
- `selfplay_*` (self-play lines):
  - `budget_s`: the priced budget (this cycle's target plus the carry);
  - `price_s`: the priced generation of the rounds it played; `plies`: their plies;
  - `cpu_s`: the job's total CPU (measured);
  - `gen_cpu_s`: the job's CPU split by the wall time of its phases, the playing part (measured);
  - `rounds` and `games`: what it played;
  - `carry_s`: the carry after this cycle (priced).
- `label`: the line's model label.
- `cal_cpu_s`: CPU for this line's own calibration games in this cycle.
- `next_version`: the version that plays next cycle (empty once retired).
- `note`: why a step did nothing or failed.

### totals.csv (one row per cycle)

`cycle, start, end, wall_s, kind, lines_active, lines_retired, field, anchors, rounds, games, ply2_games,
tournament_cpu_s, ply2_cpu_s, tournament_wall_s, train_cpu_s, selfplay_cpu_s, selfplay_gen_cpu_s, cal_cpu_s,
cal_made, new_champions, retired, anchors_locked, anchors_replaced, global_rounds, global_cpu_s, global_ply2_cpu_s,
wait_s, data_cpu_target_s, tournament_proc_cpu_s, price_s, ply2_price_s, global_price_s, global_ply2_price_s,
global_carry_s, data_price_target_s, selfplay_price_s, prices_version`

- `tournament_cpu_s` and `ply2_cpu_s`: the sums of per-game CPU over all games, and over the qualifying ones.
- `global_rounds`, `global_cpu_s`, `global_ply2_cpu_s`: the global rounds copied for this cycle (experiments).
- `data_cpu_target_s`: the measured CPU of this cycle's qualifying games (experiments; information only: the
  decisions use `data_price_target_s`).
- `tournament_proc_cpu_s`: the tournament process's own CPU (games plus the rating fit and tables).
- The `*_price_s` columns are the priced counterparts: `price_s` and `ply2_price_s` of this cycle's games;
  `global_price_s` and `global_ply2_price_s` of the copied rounds (priced by the experiment's own prices);
  `data_price_target_s` what each line should train on (the qualifying games' price); `selfplay_price_s` the
  self-play lines' priced generation, summed. `global_carry_s` is the state's after the cycle. `prices_version` is
  the league's.
- Columns ending in `_cpu_s`, `_wall_s` and `wait_s` are measured (information only); decisions use prices.

### rounds.csv (one row per round)

`cycle, t, round, players, games, ply2_games, cpu_s, ply2_cpu_s, wall_s, price_s, ply2_price_s`

- `t` is the tournament index in `tour/games.csv` (from 0, unique within the league). `round` counts from 1 within
  the cycle.
- `cpu_s` is the sum of per-game CPU (`tour/game_cpu.csv`). It excludes the rating fit and the tables. `wall_s` is
  the tournament's own timing of the round's games (whole seconds). `price_s` and `ply2_price_s`: the round's
  games' prices, all and qualifying.

### events.csv

`cycle, date, line, event, entry, version_name, value, detail`

- `event` is one of:
  - `entered`: a line joins with its untrained version (entry `<line>@0-2`);
  - `first-champion`;
  - `new-champion` (value z);
  - `retired` (global, patience);
  - `anchor-locked` (value: the interval's lower bound);
  - `anchor-available`: an anchor was out of its interval by more than the margin for out_cycles straight cycles;
  - `anchor-replaced`: the old anchor leaves at the moment a new one takes the interval;
  - `copied` (experiments: global rounds copied; value the number of rounds; detail "global cycle C rounds t1 t2
    ...; ply-2 price X s and carry Z s cover N rounds at Y s (how the round was estimated)");
  - `wait-start` and `wait-end` (value: the seconds waited);
  - `done` (experiment finished).

### history.csv (every field entry, every cycle)

`cycle, entry, line, version, version_name, ply, role, elo, ci95, games`

- `role`: `current`, `champion`, `current+champion`, `anchor` or `reference`.

### entries.csv (rewritten each cycle)

`entry, line, version, version_name, ply, role, in_field, joined_cycle, left_cycle, model_file, spec`

- Every entry that was ever in the field, and the next cycle's new ones, with its latest role.
- `in_field` is 1 if it plays in the next cycle. `joined_cycle` is its first cycle; `left_cycle` the last cycle it
  played (empty while it stays).

### anchors.csv (global only, rewritten each cycle)

`interval_lo, interval_hi, entry, line, version, version_name, locked_cycle, locked_elo, elo, ci95, outside_streak,
available`

- Intervals are 100 Elo wide on the ply-1 scale, from 100 up: [100,200), [200,300), and so on. `available` is 1
  once the anchor has given its interval up. It stays in the field until a new anchor takes the interval.

### copied.csv (experiments with the global option)

`cycle, global_cycle, t, games, ply2_games, cpu_s, ply2_cpu_s, copied_at, price_s, ply2_price_s`

- `price_s`, `ply2_price_s`: the round's games priced by the experiment's prices (the global league's kinds).

### tour/players.txt and tour/retired.txt

`NAME DEPTH SPEC` per line (`#` lines are comments). A model's spec is
`cal:<league>/owncal/<line>@<v>:<kind>:<league>/runs/<line>/<kind><v>.model`; a reference's is `rand`.

### tour/game_cpu.csv

`tournament, white, black, cpu_s`: the CPU time of each game, measured on the thread that played it. The tournament
binary writes it with `--game-cpu`.

### The global batch: <global>/batch/

```
batch/batch.json     {"cycle": 12, "made": "...", "prices_version": "P1", "rounds": [{"t": 55, "games": 2652,
                       "ply2_games": 2310, "cpu_s": 80.2, "ply2_cpu_s": 74.1, "price_s": 79.0,
                       "ply2_price_s": 72.8}, ...]}
batch/games.csv      t, white, black, result, plies, cpu_s, ply2 (1 if a model played at ply 2), price_s
batch/round<t>/      the games, as tournament --save-games writes them (<white>~<black>/ game dirs)
```

- It is replaced as a whole right after each global cycle's tournament (phase `batch`), under an fcntl lock on
  `batch.lock`. Experiments copy with hard links while holding the lock shared.
- `ply2` is 1 or 0. `cpu_s` and `price_s` have 6 decimals. Prices here are the global league's; an experiment
  prices what it copies with its own.

### runs/<line>/sp/plies.csv (self-play lines)

`round, games, plies, price_s`: one row per self-play round (from 0), from the self-play output (`round N: +W -B
=D, P plies, ...`): its games, its plies and its price. Rewritten whenever the line's self-play runs.

### copied/ (experiments with the global option, while the cycle runs)

`copied/round<t>/` are hard links to the batch's rounds; `copied/games.csv` is the batch's rows of those rounds.

## leagues/prices.csv (the code dir, not a league)

`version, context, kind, depth, us_per_game, us_per_ply, source, games, plies, note`

- The price formula's constants, in CPU microseconds. A game's price is the sum over its two sides of
  `us_per_game + moves x us_per_ply` for the side's (context, kind, depth); white has ceil(plies / 2) moves, black
  floor(plies / 2). A self-play round of G games and P plies costs `2 G us_per_game + P us_per_ply`.
- `context`: `tournament` (the tournament binary's games; `kind` `random` with depth = the reference's ply) or
  `self-play` (a line's self-play generation at its self-play depth, `--depth`, default 0).
- `source`: `measured: ...` or `derived: ...` (how); `games` and `plies`: the measurements behind a measured row.
- A version's rows never change. `leagues.py prices SPEC.json` (leagues/calibrate.py) measures, fits and appends a
  new version, with its report `leagues/prices/<version>.md` (the constants, the calibration players, and priced /
  measured CPU by kind, depth, stage, model size, game length and source) next to its spec
  `leagues/prices/<version>.spec.json`. A league copies its version's constants into its config at create.
- A spec's `tournament` may set `repeats` (default 1): the same round robins (same seed, same games) played that
  many times, interleaved, in `tour-p<ply>-r<k>` for k > 1. The fit uses every repeat; the report adds the CPU of
  the same games between repeats, and each repeat's constants alone.

## leagues/recipes.txt (the code dir, not a league)

`model | recipe | archetype | kind | seed | train_binary | selfplay_binary | train_args | selfplay_args | sources |
heldout` (fields separated by ` | `; `#` lines are comments), written by leagues/make_recipes.py from the live
registry (the lineages active at cycle 64, plus make_recipes.py's EXTRA: F-N-c and F-P-c). 65 models of 55
recipes. The binaries are the rebuilds that leagues/binaries-fast.txt and then leagues/binaries-fx.txt map the
live ones to (the same games and models): chessai-hybrid4gfx for 57 rows, the q9 builds for the 8 side-branch rows.
Comment lines name each live binary and its rebuilds in turn. The global league takes recipe names; experiments
take model names.
