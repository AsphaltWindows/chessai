# New league system: file formats

The reference comes first; the changelog is at the end ("Changes (newest first)"). A few details are written only in
the changelog: the training-seed rule for replicates (2026-11-09 entries: the league's train seed plus the cycle plus
100003 × seed_offset), the calibration spec's own fields (`depth` on a self-play item, `selfplay_fit_stages`,
`refresh_costs`, `tournament.repeats`, `round_model_bytes`), a copied player's `.source` file, and the rule that a
copied tree version is taken only with states older than 30 s. Arenas have their own doc: leagues/ARENA.md.

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
  arenas/<name>/              one arena dir per arena (fixed models, no training): leagues/ARENA.md
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
- An experiment with observers (config.json below) adds `observers` (their number) and `observer_models` (their
  model ids); its `models`, `labels`, `options` and `lines` are the participants' only.
- An experiment with cold_games (config.json below) adds `cold_games`, its config value.
- An experiment with train_filter (config.json below) adds `train_filter`, its config value (`{"margin": M,
  "smooth": K}`, M "se" or a number), and with self_games_topup `"self_games_topup": true`.
- An experiment with openings (config.json below) adds `openings`, its config value.
- An experiment with rate_every (config.json below) adds `rate_every`, its config value (N, from 2).
- An experiment with champion_se_kind "draw-aware" (config.json below) adds `champion_se_kind`.
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
  anchors.csv        11-09 global leagues only: the anchor table by 100-Elo interval (the 11-10 design has none)
  copied.csv         experiments with the global option: global rounds copied (appended)
  self_games.csv     leagues with self_games: one row per self-game (appended)
  self_games.log     leagues with self_games: each cycle's replay-games command and output
  topup_games.log    experiments with self_games_topup: each cycle's top-up replay-games command and output
  cycle-NNN.md       a readable report of cycle NNN
  driver.log         the driver's progress lines
  tour/              the tournament tool's directory: players.txt, retired.txt, games.csv, game_cpu.csv,
                     ratings.csv/.md, elo_history.csv, head_to_head.csv/.md, tournament.log
  runs/<line>/       a line's model files: <kind><v>.model (and <kind><v>.state for trees), v = version number
  runs/<line>/sp/    a self-play line's self-play run directory; runs/<line>/sp/plies.csv prices its rounds
  runs/<line>.log    the line's train and self-play commands and their output
  owncal/            each version's own calibration games, owncal/<line>@<v>/ with <line>@<v>.json and .arena
  frozenref/         experiments with frozen_references: each reference's model copy <name>.<kind>.model (made at
                     create) and own calibration games <name>/ (made once in cycle 1, never deleted), with
                     <name>.json (how they were made) and <name>.arena (the arena's log)
  pool/              this cycle's saved games while the cycle runs (deleted after training)
  batch/             global only: the latest complete batch, kept until the next replaces it (see below)
  batch.new/         global only: the next batch while it is being linked (renamed to batch/ under the lock)
  copied/            experiments with the global option: the rounds copied for this cycle (deleted after training)
  selfpool/          leagues with self_games: this cycle's self-games while the cycle runs (deleted after the cycle)
  topuppool/         experiments with self_games_topup: this cycle's top-up games while the cycle runs (deleted
                     after the cycle; see below)
  replaypool/        experiments with replay: each cycle's own local games, kept for the later cycles' replay
                     (replaypool/c<cycle>/, see below; a cycle the next cycle's window no longer reaches is deleted
                     after each cycle)
  trainsets/         this cycle's training sets while training runs: trainsets/local/round0/ and
                     trainsets/global/round0/ (the global league: trainsets/<line>/round0/, one set per model), one
                     link per game named <t>~<white>~<black> (deleted after the cycle); with reference_window or
                     train_filter also trainsets/<line>/round0/ for each line it cuts games from (a top-up game's
                     link is <t>~<white>~<black>~<j>)
  .nogames/          an empty game, which `train` turns into an untrained version 0
  .lock              the driver's fcntl lock (one driver per league)
  batch.lock         global only: the batch's fcntl lock
  STOP               if present, the driver stops at the next safe point: between cycles, or while an experiment
                     waits for global rounds; the driver deletes it when it stops on it
  repeat.log         the loop script's output, if it was started as shown in leagues/repeat.sh
  scan-cache/        the driver's summaries of tour/games.csv and tour/game_cpu.csv (games.csv.json,
                     games.csv.pairs.json, game_cpu.csv.json), so each cycle reads only the new lines (see Changes,
                     2026-11-10 10:56); a missing, stale or damaged cache means a full read with the original code
  extend/cap-<C>/    experiments extended past their cap C (`leagues.py extend`, config.json "Cap history"): the
                     originals of the cap cycle: config.json, state.json, lines.csv, entries.csv, cycle-<C>.md, the
                     per-cycle CSVs' rows of cycle C (cycles, history, rounds, events, totals, cold_rounds,
                     self_games, observers), the rows cut from the games files (tour-games.csv, tour-game_cpu.csv,
                     cold-games.csv, cold-game_cpu.csv), redo.json (cycle, t0, rounds, cold_t0, cold_rounds, field,
                     finals, models, states) and the finals' models and training states, runs/<line>/<kind><v>.model
                     and .state (hard links: the run's cleanup deletes its own states once it trains on). With its
                     config.json the dir is a warm start's source for the old finals: "start": {"experiment":
                     "<league dir>/extend/cap-C", "version": V}. Kept for good
```

- Model files are kept for every version (anchors and champions point at them). A tree's training state
  (`<kind><v>.state`) is kept for the current version only.
- `owncal/<line>@<v>/` (the games) is deleted once the version leaves the field; `owncal/<line>@<v>.json` (how they
  were made) and `owncal/<line>@<v>.arena` (the arena's log) stay.

### Names

- A league has **lines**. A global-league line is one model, a recipe at a ply, named `<recipe>.p<ply>` (`F-L.p1`,
  `T-L-Ne.p2`; the 11-10 design, 2026-11-11; before it a global line was the recipe alone, `F-L`). The same recipe
  at another ply (an upgrade) is another model and another line. An
  experiment line is a (model, option) pair: `<model>.sp` (self-play), `<model>.loc` (local training) or
  `<model>.glb` (global training).
- An experiment's **model id** is recipes.txt's model name, then `+<flag>=<value>` for each flag override (the flag
  without its dashes; `+<flag>` for a bare flag, `+<flag>=off` for a removed one), then `#<r>` for replicate r when
  the spec asks for several: `F-L`, `F-L+lr-rate=0.025`, `F-L+lr-rate=0.025#2`. So a line is for example
  `F-L+lr-rate=0.025#2.loc`. Ids use letters, digits and `_ . + = # -` only. Its **label** (for display) has spaces
  instead: `F-L lr-rate=0.025 #2`. Line, entry and directory names carry the id (`runs/F-L+lr-rate=0.025#2.loc/`,
  `F-L+lr-rate=0.025#2.loc@3-2`), so a URL or query string holding one must be percent-encoded (`#` starts a
  fragment, `+` reads as a space in a query).
- A line's **version number** `v` counts its training steps: 0 is the model it enters with (untrained, or a copy of
  its warm start's model, a tree's with its training state: config.json `lines[].start`), and v goes up
  by 1 every cycle it trains. It never resets. The version number alone identifies a version, and every file keys
  on it. A training step that fails, or a self-play line whose budget is under half a round, leaves v as it was, and
  the same version plays again. An experiment's last cycle trains nothing, since no cycle would rate the result.
- **Version names** for display:
  - global league `G.<major>.<v>`; a frozen ply-1 anchor `G.<major>.<v>F` (11-09 design only);
  - experiments `E.<S|L|G>.<major>.<v>`, the letter the line's option (S self-play, L local, G global), for example
    `E.S.2.17`; experiments have no anchors, so no F.
  - `major` is the number of champions crowned so far, not counting the first, untrained one. The crowned version
    takes the new major: if v=5 is crowned as the line's first new champion, it is `G.1.5` from then on, and v=6 is
    `G.1.6`. A version that played before its crowning shows its old name (`G.0.5`) in that cycle's row.
  - history.csv and cycles.csv's `version_name` show the name as it was while the version played (before the
    cycle's decision). lines.csv, entries.csv and anchors.csv show current names.
- **Entry names** (players in `tour/players.txt`) key on the version number, so a crowning renames nothing:
  - `<line>@<v>-1` and `<line>@<v>-2`: version v at ply 1 and ply 2 (`<line>@<v>-P` at ply P, in an experiment
    with `entry_ply` P: 3 or 4 so far; in the global league each model's entries are at its own ply only:
    `F-L.p1@3-1`, `T-L-Ne.p2@3-2`);
  - `<line>@<v>F-1`: the frozen ply-1 anchor copy of version v (11-09 global leagues only);
  - `<name>@<v>F-<P>`: a frozen external reference (an experiment's `frozen_references`, 2026-11-12): version v of
    another experiment's line, at the entry ply P (`xFP1@314F-2`). It parses as a model at ply P, so its games
    qualify for training; `<name>` is the spec's plain name (no `#` or `@`), never a line's or a random reference's;
  - `random`, `random-1`, `random-2`, `random-3`: the references (random moves, searching at ply 0-3). `random` is
    pinned at Elo 0. An experiment without `random` (batch 4: `references` []) has a rating zero instead
    (config.json `rating_zero`).
- A line's champion entry is its champion version's entry at the league's entry ply (config.json `entry_ply`, 2
  unless set; a global-league line: its own ply): `<line>@<c>-2`, or `<line>@<c>-<P>` with `entry_ply` P. It stays
  in the field while that version is champion. When the current version is the champion, one entry serves both
  roles.

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
- `patience_cycles` and `anchors`: 11-09 global leagues only (null in experiments and in the 11-10 design).
  `anchors.lowest` (100): no anchor below it.
- The global league of the 11-10 design (models/NEW-DESIGN.md "The global league"; built 2026-11-11; `create` makes
  only these, and `run` and `add` refuse a global league without `global_design`):
  - spec `"models": [{"recipe": "F-L", "ply": 1}, {"recipe": "T-P-Ne", "ply": 2}, ...]`: each model is a recipe at a
    ply (a whole number from 1; the prices version needs the recipe's kind at that ply), each (recipe, ply) once. A
    plain recipe name is refused. Optional `"train": {"margin": M}` (Elo, a number from 0, default 30),
    `"self_games": false` (default true) and `"patience_rule": {"positions": P, "floor_cycles": K}` (whole numbers,
    P from 1, K from 0; `"rule": "positions"` may be given, no other rule exists). Refused: `anchors`,
    `patience_cycles`, `entry_ply`, `ply1_entries` true, `plateau`, references without `random`; `patience_rule`
    and `train.margin` in an experiment.
  - Optional `"games_per_entry": G` (spec or config.json; a whole number from 1; refused in experiments; absent
    from the config unless given): variable-length cycles (2026-11-11 ~20:30). Each cycle plays
    ceil(G / (2 (N - 1))) double round-robins instead of `rounds_per_cycle`, N the cycle's field (references,
    each live model's current and champion), so every entry plays at least G games. It is read when the cycle
    plans its rounds (the calibration phase, after the field is known): set in a running league's config.json, it
    takes effect from the next cycle, and a resumed cycle keeps the rounds it planned. `run` refuses a value that
    is not a whole number from 1. The cycle report's Field line gives the rule ("games_per_entry 600 over 16
    entries: ceil(600 / 30) rounds, 600 games each"); totals.csv's `field` and `rounds` record N and the rounds.
  - `"global_design": "11-10"`, `"patience_cycles": null`, `"anchors": null`, `"ply1_entries": false`,
    `"patience_rule": {"rule": "positions", "positions": 20000000, "floor_cycles": 5}` (rule 6, below under
    state.json `patience`), `"self_games": true` (unless the spec says false; the tournament binary needs
    `replay-games`), `"train": {"seed": 1, "qualify": "both players rated at or above the model's current Elo
    minus the margin (unrated players count)", "margin": 30}`.
  - Each line: `{"line": "F-L.p1", "model": "F-L", "recipe": "F-L", "option": "league", "kind": "lr", ...,
    "entered_cycle": 1, "ply": 1}` (no `label`: reports and the dashboard show the line id). `add LEAGUE --ply P
    RECIPE...` appends lines the same way (`entered_cycle` the next cycle); a line already in the league, retired
    ones too, can't be added again (no revivals).
  - The experiments' global option is refused against it (it copies games with a model at ply 2, the 11-09 rule; an
    Elo-cutoff G line is NEW-DESIGN's open proposal).
- `options` of a line: `league` (global league), `self-play`, `local`, `global`.
- `global_source`: an experiment's path to the global league dir it copies rounds from (null without the global
  option).
- `min_rounds` (5): with the global option, the copied rounds' ply-2 price must cover this many experiment rounds.
- Optional `"games_per_cycle": T` (spec or config.json; experiments and the global league; a whole number from 1;
  absent from the config unless given; 2026-11-12, the user: "equilibrate the amount of games towards some
  standard range/amount by changing the number of rounds per cycle based on the number of active participants",
  for steadier Elos and a steadier training batch; the analyst's standard is 1584, range 1200-2000). Each cycle
  plays R = round(T / (N (N - 1))) double round-robins (halves up), at least 1 and at most 48, N the cycle's field:
  every entry of its round-robins (references, frozen references, each line's current and champion entries, ply-1
  entries, anchors). R is recomputed every cycle once the field is known (the calibration phase; with the global
  option, the copy phase, where R takes `min_rounds`' place: copy the global rounds that cover R experiment rounds,
  play round((copied + carry) / estimate) rounds, at least R). It replaces `rounds_per_cycle` and `min_rounds`:
  create leaves both out of config.json and refuses a spec that gives either with it, or `games_per_entry` with it
  (run refuses a config with both games_per_cycle and games_per_entry, or a T that is not a whole number from 1).
  `gpc_bound` "under T": the cap binds, 48 rounds are fewer than T games (N up to 6 at T 1584); "over T": the
  floor binds, one round is more than 2 T games (N from 57 at T 1584). Self-games stay one per qualifying entry
  per round, outside T. The random references' games count toward T; only games with a model at the entry ply
  train, as before. A resumed cycle keeps the rounds it planned; a changed T applies from the next cycle. Records:
  state.json `current.games_per_cycle` ({t, n, rounds, bound}); totals.csv's `field` (N), `rounds` (R) and `games`,
  and its last columns `games_per_cycle`, `gpc_bound`, `train_games`, `train_positions` (below); driver.log's
  "cycle C: games_per_cycle T over N entries: R rounds, G games"; the report's Field line gives the rule
  ("games_per_cycle 1584 over 8 entries: round(1584 / 56) = 28 rounds"; "round(1584 / 12) = 132 rounds, capped at
  48: 576 games, under T") and a Training batch line. Without the key nothing changes. At T 1584: 8 entries play
  28 rounds (1,568 games), 12 play 12 (1,584), 20 play 4 (1,520).
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
- Experiment options of 2026-11-10 (batch 4). They are not in the defaults: a spec without them makes a config
  without them, and the league runs exactly as before (shown byte-identical on a copy of small-field).
  - `references` without `random` (batch 4: `[]`): create adds `"rating_zero"`, the first line's version 0 at ply 2
    (`<line>@0-2`, in every experiment's first field). The tournament runs without `--anchor`, so tour/ratings.csv
    is relative to the rated players' mean. The driver re-centres every rating it records (cycles.csv, history.csv,
    the cycle report) on `rating_zero`, which reads 0.0; ci95 stays the binary's (relative to the rated players'
    mean, as always). The champion test's SE pins `rating_zero` (or any rated player while it is unrated): a
    contrast's SE doesn't depend on which rated player is pinned. Refused for the global league.
  - `ply1_entries` (default true): false drops the current@1 entries. A line's entries are then its current
    version and its champion at ply 2 (one entry when they are the same version); cycles.csv's `entry1`, `elo1` and
    `ci1` are empty. Refused (false) for the global league.
  - `plateau`: `{"cycles": K, "min_cycles": M}`. The experiment ends after the first cycle c >= M in which no line
    has crowned a champion for K cycles: c - (the last cycle with a new champion; the first champions count, in
    cycle 1) >= K. That cycle is the last one: it trains nothing (as the last of `cycles`), records the `done` event
    with detail `plateau: no new champion since cycle X (N cycles)`, sets status `done`, and a later `run` exits
    with 4. `cycles` stays the cap. Batch 4 uses `{"cycles": 50, "min_cycles": 100}` and `cycles` 1000. Experiments
    only.
- Options of 2026-11-10 ~22:15-22:40 (the user: experiments screen at ply 1, with self-games, and the plateau's wait
  scales with the number of lines). Not in the defaults either: a spec without them makes a config without them,
  and the league runs exactly as before (shown byte-identical on copies of three experiments; see Changes).
  - `plateau` `{"scaled": true, "min_cycles": M}` or `{"scaled": {"k2": K2}, "min_cycles": M}`: create computes
    K = round(K2 x sqrt(2 / n)), n the number of lines, K2 50 unless given (halves round up: 50 for 2 lines, 41 for
    3, 35 for 4, 29 for 6, 25 for 8, 19 for 14), and writes `"plateau": {"cycles": K, "min_cycles": M, "scaled":
    {"k2": K2, "lines": n}}`. The driver runs the rule above on `cycles`; `scaled` only records the inputs.
    Experiments only. With rate_every above 1 the spec's scaled plateau also takes `rate_scale` (Rate every N
    below): K is then multiplied once more, and `scaled` records `rate_every` and `rate_scale` too.
  - `entry_ply` (a whole number from 1; absent means 2; 3 from 2026-11-11 01:24, any from 03:27): the ply of the
    main entries. Each line's current version and its champion play at it (`<line>@<v>-P`), and so are the rating zero
    (`rating_zero` is `<first line>@0-<entry_ply>`), the champion test, the plateau rule and the training rule: a
    game qualifies (trains the local lines; counts in the `ply2_*` columns, which keep their names) if a model at
    the entry ply plays in it, and `train.qualify` reads `a model at ply P`. With P other than 2, create sets
    `ply1_entries` false (true is refused: the main entries are the only model entries) and refuses the global
    option (it trains on the global league's games with a model at ply 2). References are allowed: they are rated
    with the entries, and a game of a reference against a model at the entry ply qualifies. A self-play line whose
    depth neither the spec nor its recipe sets (the binary's default, 0) self-plays at the entry ply: create puts
    `--depth P` in its `selfplay_args` and records `"selfplay_depth_default": P` on the line (only when the spec
    gives `entry_ply`); a `--depth` the spec or the recipe sets stays (F-L-d2jit self-plays at depth 2). Games are
    priced at their ply, so create refuses a prices version without the lines' kinds at the entry ply: P2 has
    tournament constants at plies 3 and 4 from 2026-11-11 (at ply 4 lr measured, the other kinds estimates;
    prices.csv below) and self-play constants to depth 4; P0 and P1 have none above ply 2. Experiments only.
  - `self_games` (true or false): in each cycle that trains, every round, each entry whose game against itself
    qualifies (a model at the entry ply: the main entries, not references or other-ply entries) plays one game
    against itself, the same version at the same ply (self_games.csv below). They are training games for the local
    lines only, never in tour/games.csv, so outside the ratings, the champion test, the head-to-head tables and the
    plateau rule. They are priced like tournament games, and their price joins the cycle's local data price
    (`data_price_target_s`), which sets the self-play lines' budgets and the global carry. Needs a line that trains
    on the league's games (local, or the global league's) and a tournament binary with `replay-games`
    (chessai-hybrid4gfx, 5, 7, 8 and 9 have it). An experiment's last cycle plays none: nothing trains after it. The
    global league may have them too: its lines train on them, and its batch doesn't include them.
  - `replay`: `{"share": S, "window": W}` (2026-11-11 ~22:00, for the replay-p2 test, analyst 6.46 (i); S a number
    above 0 and below 1, W a whole number from 1; absent from the config unless the spec gives it, and then the
    league runs exactly as before). Each cycle c's local training set is the cycle's own games, as without it (the
    round-robins' qualifying games and, with self_games, the self-games), plus games replayed from the own games of
    cycles max(1, c - W) to c - 1: round(S / (1 - S) x the own games) (rounded half up; S 0.5 replays as many as
    its own), fewer when fewer were kept, sampled uniformly without replacement and deterministically from the
    league's `seed` and c (the first n of the kept games ordered by sha256 of "replay <seed> <c> <cycle> <t>
    <white> <black>"). The replayed games are the kinds the local set holds: round-robin games and self-games.
    Each cycle that trains keeps its own games in replaypool/ before it samples; cycle 1 has nothing to replay;
    the last cycle trains nothing, so it neither keeps nor replays. The sample is recorded in state.json
    (`current.replay`) before training, so a resumed cycle trains on the same set. `train` shuffles a round's
    positions (train_rounds, a seeded shuffle of the whole round), so the replayed games' links keep the usual
    names `<t>~<white>~<black>` (t is unique across cycles) and still interleave with the cycle's own games. The
    local lines' `train_games`, `data_price_s` and `data_cpu_s` (cycles.csv) cover the own and the replayed games
    (the replayed games' price and CPU are those of their own cycles); the cycle's data budget
    `data_price_target_s` (totals.csv: the self-play budget and the global carry) stays the own games'. Experiments
    with local lines only (refused otherwise); the global option's set is not replayed. Switched on in a running
    experiment's config.json, it replays the games kept from then on; `run` refuses a malformed value.
  - `frozen_references` (2026-11-12, for diversity-p2: theory-2 6.45 (2) and 6.53, the training data's coverage from
    other players above a quality floor; absent from the config unless the spec gives it, and then the league runs
    exactly as before). The spec gives `[{"name": N, "version": "<experiment>:<line>@<v>"}, ...]`, the experiment a
    name under the root's experiments/ or a league dir's absolute path. Each is a fixed model: every cycle it joins
    the field after the random references as entry `<N>@<v>F-<P>` at the entry ply P (role `frozen-reference`, line
    empty, version v) and plays the round-robins like an entry. It never trains, is never crowned or retired, plays
    no self-games, and the champion test and the plateau rule don't see it (they look at lines only). A game with it
    counts as a game with a model at the entry ply, so the local lines train on all three kinds: line-line,
    line-reference and reference-reference games (plus the self-games); the round-robins' prices, and so
    `data_price_target_s`, include them at the reference's kind and the entry ply (P2 has probit and bdt at ply 2).
    Own calibration: as arena.py does for an experiment version, its own games are its experiment's (the source's
    calibration settings, held-out calibration and tournament binary: hybrid13 for error-routing-p1), made once in
    cycle 1's calibration phase into frozenref/<N>/ (outside owncal/, so never collected); the arena's result line
    must equal the one the source recorded (owncal/<line>@<v>.arena), and each tournament's fit on them must equal
    the calibration the source's tournament log printed for the version (else the cycle stops: status `failed`).
    create resolves each into config.json: `name`, `version`, `experiment` (as given), `dir`, `line`, `v`, `kind`,
    `source_model`, `md5` (the model's bytes at create), `heldout`, `calibration` (games, depth, seed, policy,
    temperature), `owncal_binary`, `expected_result` and `expected_calibration`, and copies the model into
    frozenref/<N>.<kind>.model; the copy's md5 is checked every cycle (a changed copy stops the cycle in
    calibration). The local set's games by kind are in totals.csv's `local_*` columns and the report's Local set
    line. Refused: a name that isn't plain (NAME_RE), a random reference's or a line's name, a name twice, an empty
    list; a source that isn't an experiment, a missing line or model, a model whose md5 isn't the one its own games
    were made from, a version without one own-games result and one calibration in its source (a version that never
    played there); a missing binary or price; the global league, the global option, replay. `run` refuses a
    malformed value.
  - `reference_window` (2026-11-12, for diversity-p2c: theory-2 6.57, scope (a); absent from the config unless the
    spec gives it, and then the league runs exactly as before): `{"margin": M}`, M a number from 0 (Elo), only with
    `frozen_references` (so not in the global league, nor with the global option or replay). The global league's
    Elo window (`window_ok`, NEW-DESIGN rule 4) applied to the frozen references' games alone: in each cycle a
    line's floor is its current version's Elo in the cycle's fit minus M (decide's `ref_floor`; none while the
    version is unrated, and then nothing is cut), and a round-robin game with a frozen reference trains the line only
    if every frozen reference in it is rated at or above that floor (an unrated reference counts): line-reference
    and reference-reference games alike. Line-line games and self-games always train. A line the window cuts
    nothing from trains on the cycle's shared local set, as without the key (a margin no reference falls below gives
    the league it would have without it: shown byte for byte); a line it cuts games from trains on its own set,
    trainsets/<line> (the local set less the cut games, in the same order), and its cycles.csv `train_games`,
    `data_price_s` and `data_cpu_s` are that set's (the local set's less the cut games' price and measured CPU).
    Each cycle's cut is decided once (a resumed cycle reuses it), logged in driver.log ("cycle C: reference window
    (margin M): <line> (floor F) N games cut, <entry> <Elo>, ... below; ..."; nothing when it cuts nothing), kept in
    state.json (`current.ref_window`) and recorded in totals.csv's `ref_window_*` columns and the report's Reference
    window line (and the Training batch line, with games_per_cycle, names the cut). `run` refuses a malformed value.
- Warm starts (2026-11-11 03:27, theory-2 6.17's twins; tree kinds since 2026-11-12 16:56:27, for the ply-4 mixed field).
  Not in the defaults: a spec without them makes a config without them, and the league runs exactly as before (shown
  byte-identical on copies of four experiments, and for tree kinds on three running ones; see Changes). A spec's
  model entry may have `"start": {"experiment": E, "version": V}` and `"start_forget": F` (a number above 0, default
  1). Each line of the entry (every replicate, under each option) starts from version V of the line of its own name
  (the same model, replicate and option) in experiment E: a name under the root's experiments/, or a league dir's
  absolute path. create records on the line `"start": {"experiment", "dir", "line", "version", "path", "md5",
  "forget"}` (path: the start model's file; md5: its bytes at create), and for a tree kind (bdt, bdd, ht, tan) also
  `"state"` and `"state_md5"` (the start version's training state, `<kind>V.state`, and its bytes). The line's version
  0 is a byte copy of that file (a tree's: of the state, as `<kind>0.state`, then of the model), made in cycle 1's
  calibration phase and checked against the md5 (a changed file stops the cycle: status `failed`); cycle 1's
  tournament plays it. The line's first training runs `train --start <version 0> --init-forget F`, which divides the
  start model's AdaGrad sums (LR) or counts (CNBP) by F, or multiplies a probit's weight variances by F (so it also
  predicts less confidently) (no flag when F is 1); a tree's runs `train --start <version 0's state>`, so the tree
  trains on from its start version's state, as every later training of a tree runs from the previous version's.
  Every later one trains as any line does (`--start` the previous version, no forget). The line keeps its own seeds
  (the league's). A run keeps only its current version's state, so a tree's V must be a finished run's last version
  or a frozen copy (a league-shaped dir: config.json and runs/<line>/<kind>V.model and .state; e.g.
  leagues/starts/grow-dag-p1@395/). Refused: a self-play line (its versions come from self-play), a missing league,
  line or model, a start line of another kind, `--init-forget` among the line's train arguments, and with F not 1 a
  flat line's train binary without `--init-forget`; for a tree a missing state, and a start line of another recipe or
  other train arguments (a state continues the tree under its own options). Experiments only.
  A tree's leaf forget and warm reset (2026-11-12, the ply-2 rung of training standard v1, analyst 6.95; before, a
  tree's F had to be 1). For bdt and bdd, F not 1 runs the first training with `--start-leaf-forget F` (every live
  leaf forgotten once with its path table: LR AdaGrad sums / F, probit variances x F, CNBP counts / F; routers
  untouched), and `"start_reset": true` (default false) with `--warm-reset`: a growing tree or DAG (`--split-rule
  grow`) keeps its structure and growth state (credits, closes, back-off, router chain), closes its leaf trials in
  flight on their statistic so far (kept when fewer than 2 blocks have finished) and a DAG's router trials under their
  own keep rule (kept without a finished window), and restarts every node's windows, its snapshots, the routers'
  shadows and null shadows and pending post-split checks; path tables carry over. Both come after `--start <version
  0's state>` (reset first); later trainings as usual. `start` records `"reset": true` only when true. Both need a
  train binary with the flags (chessai-grow7: the entry's `"train_binary"`). Refused besides: either for another kind
  (a flat line's reset too), a reset without `--split-rule grow`, F not 1 under the grow rule without the reset (the
  reset closes the trials and drops the snapshots first), a binary without the flag, start_reset without start or not
  true/false, an observer with start_reset.
- A model entry's `"train_binary"` (2026-11-12): the entry's lines train with this binary instead of their recipe's
  (recipes.txt): a name in models/from-scratch/bin, a path from the repository or an absolute path, which must exist,
  be executable and accept every training argument of the lines (create refuses otherwise). config.json's
  `lines[].train_binary` is its path; nothing else changes. As specs/ply1/train_binary.py after create, but at create,
  so a warm start's flags are checked against the binary that runs them.
- Observers (2026-11-12; the user's idea and name, ~10:15: "use the games generated from uniform fields to train
  models of other types but not have them participate in the field, so that we can later test their strengths in
  arenas"). Not in the defaults: a spec without the key makes a config without observers, and the league runs exactly
  as before (shown byte for byte: probit-f10-p1 re-run 3 cycles; see Changes). A spec's `"observers"` is a list of
  model entries with the syntax of `models` (model, set, sweep, to, replicates). Each becomes a line appended to
  `lines` after the participants', with `"observer": true`, option `local`, named `<model id>.obs` (a participant's
  is `<model id>.loc`, so an observer of a participant's own model never shares its name), and its recipe's
  binaries, arguments and replicate seeds as any line's. In each cycle that trains, after the participants, every
  observer trains on exactly the cycle's local set (the qualifying round-robin games plus the self-games, plus
  replayed games with replay; reference_window's cuts are per participant and don't apply), from its own previous
  version, with its own training seed (the league's train seed plus the cycle plus 100003 x seed_offset): an
  observer given a participant's model and replicate trains byte for byte as that participant. Its version 0 is made
  in cycle 1's calibration phase. An observer never plays: it is in no field, round robin, self-game or reference;
  it isn't rated or calibrated, never crowned, and counts nowhere participants are counted (games_per_cycle's N, the
  scaled plateau's lines, the field). It gets no self-games of its own, and nothing tops that up: the self-games
  in its set are the participants' (at ply 1 with games_per_cycle 1584, 192 of cycle 1's 768 games with 4 entries,
  then 224 of 1,792 with 8 entries; each participant's own are about 3% of the set). Versions follow the
  participants' cycles: version v has trained on cycles 1 to v. The participants' games, models and records are the
  same with and without observers. Arenas read an observer's versions as any line's
  (`<experiment>:<model id>.obs@<v>`). Records: observers.csv, totals.csv's `observers` and `observer_train_cpu_s`,
  lines.csv and state.json (status `observer`), the index's `observers` and `observer_models`, the cycle report's
  Observers line. Refused: observers in the global league, in an experiment without the local option, a warm start
  (start, start_forget), a model given twice, an empty list. Experiments only.
- Cold games (2026-11-12; the user's decision at 10:54: "let's see how the addition of the T 0.02 games affects the
  model strengths (we can run some uniform ply1 tests) if it doesn't affect the strength negatively let's add this
  T 0.02 games into training if not, fine let's run a small number of additional games to compute the actual
  experiment ratings"). Not in the defaults: a spec without the key makes a config without it, and the league runs
  exactly as before (shown byte for byte; see Changes). `"cold_games": {"temperature": T, "share": S, "train": true
  or false}`, T a number above 0 (the tournament's `--temperature`, nats of log-odds with the log-odds-softmax
  policy), S a number above 0. Each cycle, after the round-robins (the same command, games and saved games as without
  the key), the field plays cold round-robins in a `cold-games` phase: the same entries, specs (own calibrations) and
  policy, the tournament's flags with `--temperature T` in place of any it gives, seed the league's plus 2000003,
  in their own directory `cold/` (players.txt, retired.txt, games.csv, ratings.csv, game_cpu.csv, tournament.log and
  the tables, as tour/'s). Their rounds: R = round(S x games_per_cycle / (N (N - 1))) double round-robins for the
  field's N entries, halves up, from 1 to 48 (S read as its decimal, so 0.25 x 1584 = 396 games exactly: 33 rounds
  of 4 entries, 7 of 8). The in-run ratings are the cold games' fit alone: `cold/ratings.csv`, re-centred on the
  rating zero, and the champion test's SE from `cold/games.csv`; so every decision runs on them (the champion test
  and crownings, the plateau rule, and with them the field), and history.csv's and cycles.csv's Elo columns are the
  cold fit's. `tour/` keeps the round-robins and their own fit, which nothing decides on. With `"train": true` the
  cycle's qualifying cold games join the shared local set after its round-robin games and self-games (their links
  are named `<1000000 + cold t>~<white>~<black>`, so they sort last), and their measured CPU and price join the set's
  data CPU and price (cycles.csv `data_cpu_s`, `data_price_s`); `data_cpu_target_s` and `data_price_target_s` stay
  the round-robins' and self-games'. Every local line and every observer trains on them. With false they rate only
  (not saved). An experiment's last cycle plays them too (it rates its last versions) and trains nothing.
  Self-games stay one per entry per round-robin round, at the tournament's temperature. Records: `cold_rounds.csv`,
  totals.csv's cold columns, the cycle report's Cold games line and its CPU lines, the index's `cold_games`.
  Refused: the global league, an experiment without games_per_cycle, with self-play or global lines, with replay,
  with frozen_references, a malformed value. Experiments only.
- Train filter (2026-11-12; the user's Elo-filtered multi-kind experiment, 11-12 ~11:11; analyst 6.85 (b); the rule
  as the user set it at 13:22-13:24, the global league's qualify rule: "both players need to be at least the current
  model's strength - standard error based on last round"). Not in the defaults: a spec without the key makes a
  config without it, and the league runs exactly as before (shown byte for byte; see Changes). `"train_filter":
  {"margin": M, "smooth": K}`, M `"se"` or a number from 0 (Elo), K a whole number from 1 (cycles; default 1, written
  into config.json; `"se"` takes 1 only). Only in an experiment with cold_games, whose cold ratings make the floors.
  In each cycle that trains, line L's floor is R_L - SE_L with `"se"`, R_L - M with a number: R_L its current
  version's cold Elo in the cycle's fit, SE_L that rating's standard error in the fit (the fit's 95% interval over
  1.96: its games this cycle) (decide's `filter_rating`, `filter_se` and `filter_floor`; no floor while the current
  version is unrated, and then nothing is cut). A game of the local set trains L only if every player in it is rated
  at or above the floor in the cycle's fit (an unrated player counts): L's own games too, with no exemption, so its
  games against weaker opponents are cut (its current version is never below its own floor, so its self-games always
  pass; its champion's pass only while the champion is at or above the floor); the round-robins' qualifying games,
  the self-games and, with cold_games' train, the cold games alike. With K > 1 (numeric margins only; the 6.87 rule's
  smoothing) every rating is a mean over the last K cycles: R_L the mean of L's last K current versions' cold Elo,
  each as rated in its own cycle's fit (cycles.csv's `elo2`), another line's current version at its line's R, any
  other entry (a champion) at the mean of its own cold Elo over those cycles (history.csv's `elo`); ratings are taken
  at the records' 0.1 Elo, so the cut can be recomputed from them. A line the filter cuts nothing from (and that has
  no top-up games) trains on the cycle's shared local set, as without the key; a line it cuts games from trains on
  its own set, trainsets/<line> (the kept games in the local set's order, plus its top-up games with
  self_games_topup), and its cycles.csv `train_games`, `train_positions`, `data_price_s` and `data_cpu_s` are that
  set's (the local set's price and measured CPU less the cut games', plus the top-up games'). totals.csv's
  `train_games` and `train_positions` are then the lines' mean (halves up; the report's Training batch line gives
  the range). Each cycle's cut is decided once, after the self-games (a resumed cycle reuses it), logged in
  driver.log ("cycle C: train filter (floor: each line's current cold Elo minus its standard error) on the local set
  of N games (P positions): <line> (rating R, SE S, floor F) kept G (a%; b% of the positions; o of its O own),
  top-up k against <opponent>; ..."; with a number, "(margin M below each line's current cold Elo)"), kept in
  state.json (`current.train_filter`) and recorded in cycles.csv's and totals.csv's filter columns and the report's
  Train filter line. The ratings, the decisions and every game played are those of the league without the key; only
  the training sets differ.
  Refused: the global league (it has its own training window), an experiment without cold_games, with observers
  (they train on the whole local set) or reference_window, a malformed value. Experiments only.
- Self-games top-up (2026-11-12, with train_filter; NEW-DESIGN's "topped up to the standard with self-games",
  analyst 6.66 (2) and 6.85 (b)). Not in the defaults (false is the same as leaving it out). `"self_games_topup":
  true`, only with train_filter and self_games. In each cycle that trains, after the self-games (phase
  `topup-games`), each line the filter cut games from plays k = min(games_per_cycle, U - K) top-up games, K the
  games it kept of the local set's U (both counts with the self-games and, with train, the cold games), so that its
  set has the local set's size again, at most games_per_cycle more (the cap binds when K < U - games_per_cycle).
  Its current version plays its champion as decided this cycle, colours alternating (the current version white
  first), or itself (its twin `<entry>.twin`, as a self-game) when that champion is the current version (crowned
  this cycle, or the line's first cycle) or is rated below the line's floor, so every top-up game passes the line's
  filter. One replay-games run plays every line's games (config order, then game j) at the tournament's policy and
  temperature: game i of cycle c has replay-games' t = c x 10,000,000 + i and seed the league's plus 3000017
  (derive_seed(seed, t, its pairing)), so no seed stream is the round-robins' or the self-games', and a cycle cut
  short replays them exactly. Training games only, never rated (not in tour/ or cold/): line L's game j belongs to
  round t0 + floor(j R / k) of the cycle's R rounds and is linked into L's set as `<t>~<white>~<black>~<j>`, after
  its round's game of the same players. Priced as tournament games (the entries' kind at their ply); their price,
  and the replay-games process's CPU shared by price, join the line's `data_price_s` and `data_cpu_s`;
  `data_price_target_s` and `data_cpu_target_s` stay the local set's. An experiment's last cycle trains nothing and
  plays none. Records: cycles.csv's `topup_games`, `topup_positions` and `topup_opponent`, totals.csv's `topup_*`
  columns, topup_games.log, topuppool/ (below), the report's Train filter line and its CPU lines, the index's
  `self_games_topup`. Refused: without train_filter, without self_games, a value other than true. Experiments only.
- Openings (2026-11-12; the user, ~15:03: "I'm not sure different opening moves will help but we can test it";
  analyst 6.91: K 6, uniform). Not in the defaults. `"openings": {"plies": K}` or `{"plies": K, "temperature": T}`
  (T null is the same as leaving it out). Every training game the tournament binary plays opens with K plies of
  uniformly random legal moves, each drawn from the game's own seeded random numbers (an index into the position's
  legal moves), then goes on at the tournament's policy and temperature; with T, the opening's moves are each
  mover's log-odds softmax at T instead (its model, ply and calibration). The games that open: the round-robins
  (tour/), the self-games and self_games_topup's games (the binary's `--opening-plies K`, and `--opening-temperature
  T`, on `tournament` and `replay-games`). The cold games (cold_games, the rating set) and the own calibration games
  play as without the key. The opening plies are the game's like any other: in games.csv's and self_games.csv's
  plies, the prices, the saved games and so the training positions (labelled with the game's result). A cycle cut
  short replays its games exactly (the same flags). Needs a tournament binary whose `tournament` and
  `replay-games` take the flags (chessai-hybrid19 or later; without the flags it plays as chessai-hybrid18). Records:
  config.json and the index's `openings`; the logged commands (tour/tournament.log, self_games.log,
  topup_games.log). Refused: the global league, K not a whole number from 1, T not null or a number above 0,
  another key, a binary without the flags, and the flags given in tournament.flags (the cold games would open with
  them too). Experiments only.
- Rate every N (2026-11-12; the user, ~15:10: "We can split cycles into sub-cycles. each sub-cycle is like today's
  cycle but we don't update champions (we still play and train). That way the ratio of training to testing games is
  increased"; analyst 6.92; the plateau rule changed as the user agreed). Not in the defaults: without the key, or
  with 1 (not written into config.json), the league runs exactly as before (byte for byte; see Changes).
  `"rate_every": N`, a whole number from 1. Only the rated cycles, 1, N + 1, 2N + 1, ... (c with (c - 1) mod N =
  0), play the cold games and decide: the champion tests and crownings, the plateau rule and, with train_filter, the
  floors. Every other cycle (unrated) plays its round-robins, self-games and top-up games exactly as configured
  (openings too; the round-robins' own fit in tour/ runs and decides nothing, as always with cold games) and trains
  every line as before, but plays no cold games, tests no champion, crowns nothing, checks no plateau and computes
  no floors: each line's decision is `unrated-cycle` (`first-champion` for a line entering then), its champion
  stays, and history.csv's and cycles.csv's Elo columns are empty. The cap `cycles` must be a rated cycle (1 + a
  multiple of N), so an experiment always ends on a rated cycle: create and run refuse any other, naming the nearest
  two. `plateau.min_cycles` may be any number: the stop comes on the first rated cycle at or after it that meets the
  rule. The plateau rule is checked on rated cycles only and still counts training cycles: the experiment stops on
  rated cycle c >= M when c - (the last crowning's cycle) >= K, every cycle counting (crownings happen on rated
  cycles only). A scaled plateau needs `"rate_scale"`: `"sqrt"`, `"linear"` or `"none"` in the spec's plateau, with
  no default (the user hasn't chosen): create multiplies K = round(K2 x sqrt(2 / n)) by sqrt(N), N or 1 and rounds
  half up (4 lines at N 4: 35 becomes 70, 140 or 35), writes it as `plateau.cycles` and records `rate_every` and
  `rate_scale` in `plateau.scaled`. An explicit `plateau.cycles` counts training cycles as given (rate_scale refused
  there). The experiment's last cycle (rated) trains nothing, as always. With train_filter, an unrated cycle keeps
  the last rated cycle's floors (state.json `rate_carry`): each line's rating, SE and floor are that cycle's, and a
  player is rated as then: an entry rated in that cycle keeps its rating (the champions, which change only on rated
  cycles), a line's newer current version takes the rating its line's current version had then (the line's
  carried `filter_rating`), any other player is unrated; the top-up's "champion at or above the floor" check uses
  the same carried ratings. Records: cycles.csv's `rated` and `champion_age`, totals.csv's `rated`, state.json's
  `champion_cycle` and `rate_carry`, the index's `rate_every`, driver.log's "cycle C: not rated (rate_every N: cycle
  C doesn't rate, cycle C' does): no champion test, no crowning; the train filter keeps cycle R's floors; the last
  crowning was in cycle X", and the cycle report's Rated cycle / Unrated cycle line (an unrated cycle's Cold games
  line says "none" and its CPU lines have no cold games). Refused: the global league; an experiment without
  cold_games, or whose cold games train (`"train": true`: an unrated cycle would have none; not built); with
  reference_window; with a smoothed train_filter (smooth above 1); a malformed value; a cap that isn't a rated cycle;
  a scaled plateau without a valid rate_scale; rate_scale with an explicit plateau.cycles, or without rate_every
  above 1. Experiments only.
- Champion SE kind (2026-11-12, the training standard v1, models/STANDARD.md; analyst 6.93 after the split-half
  check in models/from-scratch/t01check/NOTES.md). Not in the defaults: without the key, or with `"nominal"` (not
  written into config.json), the league runs exactly as before. `"champion_se_kind": "draw-aware"`: the champion
  test's SE is the HC0 sandwich e' A^-1 B A^-1 e (leagues.py contrast_ses_draws), A the fit's information as before
  (n p(1-p) per pair) and B the games' observed score variance, (s - p)^2 per game (s 1, 1/2 or 0), so a draw counts
  for less than p(1-p) assumes. In-run cold fits the nominal SE was ~1.7x the split-half noise (0.60 of it pooled),
  which this SE predicted within 0.02-0.04. The test crowns at `champion_se` of these SEs (the standard: 3, about 1.8
  nominal SEs). Records: cycles.csv's `se_diff` and `z` are the draw-aware SE and its z (they decide), and its last
  columns `se_nominal, z_nominal` keep the nominal ones; the cycle report's "Champion test" line; the index's
  `champion_se_kind`. Refused: another value; the global league. Also for every spec since then: `champion_se`, when
  given, must be a number above 0 (a placeholder such as "TODO" is refused). Experiments only.
- Plateau guard (2026-11-12, the training standard v1; the coordinator after analyst 6.94: "never stop on plateau
  while the last rated test's current-minus-champion is above G draw-aware SEs"). Not in the defaults: without it the
  league runs exactly as before. `plateau.guard_z`: G, a number from 0, kept in config.json's plateau. When the
  plateau rule would end the experiment in a cycle that decides (with rate_every, a rated cycle), the stop is held
  while any line's champion test in that cycle has z above G, z the test's own (current minus champion over the
  test's SE: the draw-aware SE with champion_se_kind "draw-aware", else the nominal one); the next cycle that
  decides checks again, and the cap `cycles` still ends the experiment. A held cycle: driver.log's "cycle C:
  plateau: ... held by plateau.guard_z G: <line> z Z, ...", the cycle report's "Champion test" line, state.json's
  `current.plateau_held` ([line, z] pairs). Refused: a value that isn't a number from 0. Experiments with a plateau.
- Cap history (2026-11-12, `leagues.py extend`; the user, ~21:18: "For the experiments which didn't meet the plateau
  rule for termination can we keep running them?"). Not in a league never extended. `python3 leagues/leagues.py extend
  LEAGUE --cycles N` takes an experiment that ended at its cap C (state done, events.csv's done event "C cycles") on
  to cap N, as if N had been its cap from the start: config.json's `cycles` becomes N and `cap_history` gets {"date",
  "old": C, "new": N, "redo_cycle": C, "finals": {line: its version at the old cap}, "backup": "extend/cap-C"}, one
  entry per extension. Cycle C, which rated and decided but trained nothing and played no self-games (an experiment's
  last cycle), runs again as an ordinary cycle: state.json goes back to the end of cycle C - 1 (no version changed in
  cycle C; a crowning in it is undone: `crowned`, `champion`, `major`, with rate_every `champion_cycle` from
  cycles.csv; `last_crown_cycle` the last crowning before C; `last_rounds` cycle C - 1's; status stopped), cycle C's
  rows leave tour/games.csv and game_cpu.csv (and cold/'s), and the next `run` (repeat.sh) replays them: each round is
  seeded by its number, so the same games, ratings, decisions, then self-games, training and the next cycles; the
  plateau rule, its guard and the cap N decide the end. scan-cache/ is deleted (rebuilt); own calibration games are
  made again where missing (fixed seed: the same games; totals.csv's `cal_made` of cycle C counts them, the one record
  that differs from a run made with cap N). The cycle that runs again checks itself against extend/cap-C/: its field
  after calibration, its games after the tournament and the cold games, each decision (Elo, SEs, z, champion age) and
  the field's ratings after decide, and the plateau rule's last crowning; a difference stops it with an error (status
  failed, repeat.sh stops) and changes nothing more. Its report has the line "- Extended (DATE, leagues.py extend):
  the cap went from C to N; ... are the original's (checked; the originals in extend/cap-C/)"; driver.log the line
  "extend: cap C -> N (leagues.py extend) ..."; status prints "cap raised from C to N on DATE". `--dry-run`: every
  check, then what it would do, changing nothing (it reads the games files itself, so not even scan-cache/ is
  written). Refused, changing nothing: a league that isn't an experiment, one a driver holds (its .lock), one not done
  (running, stopped, failed), one that ended on its plateau rule (before or at its cap), a cap of 1, N not above C, N
  not a rated cycle under rate_every, an option extend_test doesn't cover (self-play or global lines, replay,
  train_filter, frozen_references, self_games_topup, reference_window), records that don't line up, and extend/cap-C/
  already there. The extension writes the backup first, then cuts the games files, then state.json, then config.json:
  a crash in between leaves extend/cap-C/ in place (extend refuses until it is looked at; the originals are all in
  it). Start repeat.sh again after it: an experiment's repeat.sh exits when it is done.

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

- `phase`: `idle` (between cycles), `copy`, `waiting`, `calibration`, `tournament`, `cold-games` (experiments with
  cold_games), `batch` (global), `decide`, `self-games` (leagues with self_games), `topup-games` (experiments with
  self_games_topup), `train`, `self-play` (experiments), `records`. A global cycle runs calibration, tournament,
  batch, decide, train, records; an experiment's runs copy (with the global option), calibration, tournament,
  decide, train, self-play, records. With self_games, self-games comes after decide; with cold_games, cold-games
  after the tournament; with self_games_topup, topup-games after the self-games.
- `status`: as in index.json (`created`, `running`, `waiting`, `stopped`, `done`, `failed`).
- `current`: the running cycle (null between cycles), what a resumed run continues from: `field` (this cycle's
  entries: entry, ply, role, line, version, frozen, spec), `rounds_planned`, `t0` (its first tournament index),
  `rounds` (per-round stats once played), `cal` (calibration CPU by line), `decisions` (by line), `trained` and
  `selfplay` (per line, as they finish), `copied` (experiments with the global option), `events`, `wait_s`,
  `data_cpu_target_s` (measured) and `data_price_target_s` (priced: what the decisions use). With self_games,
  `self_games` (`entries`, `rounds`, `games`, `plies`, `price_s`, `proc_cpu_s` and `wall_s` of the replay-games
  process, `done`), and the two data targets include the self-games (their measured process CPU and their price).
  With replay, `replay`: `share`, `window`, `own` (the cycle's own local games), `target` (the replayed games
  wanted), `available` (the games kept in the window), `window_from` and `window_to`, `games` (the sample:
  `[cycle, t, white, black]` each), `per_cycle` (`[cycle, count]` pairs), `price_s` and `cpu_s` (the sample's).
  With frozen_references, `frozen_cal` (each reference's calibration as the cycle's tournament fitted it, checked
  against its source's) and `local_mix` (the local set's games by kind: `line_line`, `self_games`, `line_ref`,
  `ref_ref`, `random`, `games`), recorded before training; `cal` also has each reference's own-games CPU (cycle 1).
  With reference_window, each line's `decisions` entry has `ref_floor` (its current version's Elo minus the margin,
  null while unrated), and `ref_window` is the cycle's cut, decided before training: `margin`, `floors` (line ->
  floor) and `lines` (only the lines it cut games from: `excluded`, the games cut; `below`, `[entry, Elo]` of the
  references below the line's floor; `price_s` and `cpu_s`, the cut games' price and measured CPU).
- `lines.<line>.version`: the version that plays in the next (or running) cycle. `champion` is the champion's
  version number. `crowned` lists every champion version, the first one included.
- An observer's line state (experiments with observers): `status` is `observer` (never `active`, so no field, rating
  or lines_in_cycle includes it); `version` is its latest (the cycles it has trained on); `major`, `champion` and
  `crowned` stay 0, 0 and [0]. The running cycle's `current.observed` holds the observers' training records (as
  `trained` the participants'), present only with observers.
- With cold_games, the running cycle's `current.cold`: `t0` (cold/games.csv's round-robins before the cycle),
  `planned` (R), `n` (the field's entries), `bound` ("under T", "over T" or empty), `games_target` (S x T as text),
  `proc_cpu_s` and `wall_s` (the cold tournament process, summed over resumes), `round_walls`, `in_local` (the cold
  games that joined the local set; 0 without train), `done`, and when done `rounds` (one record per cold round, as
  `rounds` of the round-robins).
- With train_filter, each line's `decisions` entry has `filter_rating` (R_L), `filter_se` (its current version's
  standard error in the cycle's fit) and `filter_floor` (R_L minus the margin or the SE; null while unrated),
  `current.filter_ratings` is the field's ratings the cut uses (entry -> Elo, null when unrated; means with smooth >
  1), and in a cycle that trains `current.train_filter` is the cut, decided once after the self-games: `margin`,
  `smooth`, `set` (`games` and `positions` of the local set) and `lines` (every line that trains, in config order:
  `rating`, `se` and `floor`; `own`, its entries this cycle; `own_games` and `own_kept`, the set's games they play and
  how many of those it kept; `below`, `[entry, Elo]` of the set's players under its floor, its own champion included;
  `kept` and `kept_positions`; `cut_price_s` and `cut_cpu_s`, the cut games' price and measured CPU; `topup`, its
  top-up games, 0 without self_games_topup; `opponent`, the top-up games' opponent: its champion's entry, `self` for
  its twin, empty without top-up games). With self_games_topup,
  `current.topup`: `lines` (line -> `games`, `plies`, `price_s`, `opponent`), `games`, `plies`, `price_s`,
  `proc_cpu_s` and `wall_s` (the replay-games process), `done`.
- `patience`: cycles since the line's last new champion (11-09 global leagues). In the 11-10 design a level from 0
  (rule 6, the user 2026-11-11 ~13:50): `patience_total / patience_rule.positions`. The 11-10 design's lines also
  keep `patience_total`, the training positions counted since the last crowning: 0 after a new or first champion;
  any other cycle adds the positions the current version trained on (the training that made it, in the cycle
  before; 0 if the line didn't train then), every model alike, the top one included. `patience_from`: the cycle of
  the line's entry or last crowning. `trained_positions`: the positions of the line's last training (0 when it
  didn't train), what the next cycle counts. The line retires (no revivals) when `patience_total` reaches
  `patience_rule.positions` (20,000,000) and at least `patience_rule.floor_cycles` (5) cycles have passed since
  `patience_from` (crowned or entered in cycle c0: never before cycle c0 + 5).
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
- `last_crown_cycle` (experiments with `plateau`): the last cycle in which a line crowned a champion (1 for the first
  champions). The running cycle's `current` also has `last_crown_cycle` and `plateau_stop` (true: this is the last
  cycle) once it has decided. With rate_every an unrated cycle carries `last_crown_cycle` over and its
  `plateau_stop` is false.
- With rate_every: `lines.<line>.champion_cycle`, the cycle its champion was crowned in (1 for the first champion);
  each line's `decisions` entry in an unrated cycle has decision `unrated-cycle` and no ratings (`diff`, `se_diff`,
  `z` null). With train_filter too, `rate_carry` is the last rated cycle's cut, which the unrated cycles after it
  use: `cycle`, `ratings` (that cycle's `current.filter_ratings`, entry -> Elo) and `lines` (line -> `version`,
  `rating`, `se`, `floor`; full precision); in an unrated cycle `current.filter_carry_cycle` is that cycle,
  `current.filter_ratings` the carried ratings (the entries rated then at their rating, each line's current version
  at its line's carried rating, others null) and each line's `decisions` entry has the carried `filter_rating`,
  `filter_se` and `filter_floor`.

### lines.csv (rewritten each cycle)

`line, model, option, kind, archetype, status, version, version_name, champion, champion_name, patience,
entered_cycle, retired_cycle, train_binary, train_args, selfplay_binary, selfplay_args, label, base_model,
train_overrides, selfplay_overrides, replicate, seed_offset`

- The last six as in config.json; overrides are written `flag=value`, separated by spaces.
- The global league (11-10 design) adds last columns `ply, patience_total, patience_from` (the model's ply; rule 6's
  positions since the last crowning and the cycle of the entry or last crowning, as in state.json), and its
  `patience` is the level in % (one decimal; 100 retires, the floor permitting).

- `status`: `active` or `retired` (global, after patience). Experiment lines are always `active` until the
  experiment is done; an experiment's observers are `observer`, with `champion` and `champion_name` empty and
  `version_name` their latest version's (`E.L.0.<v>`: an observer never crowns, so its major stays 0).

### cycles.csv (one row per line per cycle)

`cycle, date, line, model, option, status, version, version_name, entry1, elo1, ci1, entry2, elo2, ci2,
champion, champion_name, champion_entry, champion_elo, champion_ci, diff, se_diff, z, decision, new_name, patience,
train_games, train_positions, train_cpu_s, data_cpu_s, selfplay_budget_s, selfplay_cpu_s, selfplay_gen_cpu_s,
selfplay_rounds, selfplay_games, cal_cpu_s, next_version, note, selfplay_carry_s, data_price_s, selfplay_price_s,
selfplay_plies, label`

- `version` and `version_name`: the version that played this cycle, as named while it played.
- `entry1`/`entry2` with `elo1`/`ci1`, `elo2`/`ci2`: its ply-1 and ply-2 entries' ratings in this cycle's fit (`ci`
  is the tournament's 95% ±). `entry2`, `elo2` and `ci2` are the main entry, at the entry ply: with `entry_ply` 1
  or 3 they hold `<line>@<v>-1` or `<line>@<v>-3`, and `entry1`, `elo1`, `ci1` are empty.
- `champion*`: the champion before this cycle's decision.
- `diff`, `se_diff`, `z`: current minus champion at the entry ply (current@2 minus champion@2 by default), the
  standard error of that difference from the fit's covariance, and `diff / se_diff`.
- `decision`:
  - `first-champion`: the line's first cycle; its version is its first champion;
  - `new-champion`: z >= champion_se, and `new_name` is its new name;
  - `champion-kept`;
  - `unrated`: a rating is missing;
  - `retired`: the line's patience ran out (global);
  - `unrated-cycle` (experiments with rate_every): a cycle that doesn't rate: no champion test, the champion kept;
    the Elo columns, `champion_elo`, `champion_ci`, `diff`, `se_diff` and `z` are empty.
- `patience`: after the decision (the 11-10 global league: the level in %, one decimal).
- `train_games`, `train_positions`, `train_cpu_s`: the training step after the tournament (global, local and global
  lines). With self_games, a local line's games include the cycle's self-games. With replay, they include the
  replayed games (totals.csv `replay_games`), and `data_price_s` and `data_cpu_s` their price and CPU. In the
  global league (11-10 design) each version's own count: the cycle's games and self-games whose players are all
  rated at or above its floor.
- The global league (11-10 design) adds last columns `ply, patience_positions, patience_total, train_floor,
  train_self_games`: the model's ply; rule 6's positions counted this cycle (the positions this version trained on,
  `train_positions` of the line's row in the cycle before when its training made a version; 0 when it crowned) and
  since the last crowning (`patience` is that over `patience_rule.positions`, in %); the training floor (its
  current version's Elo in this cycle's fit minus `train.margin`; empty when it didn't train or its version was
  unrated, in which case every game counted); how many of its training games are self-games.
- An experiment with train_filter adds last columns `filter_rating, filter_se, filter_floor, filter_own_games,
  filter_own_kept, filter_kept_games, filter_kept_positions, filter_kept_share, filter_kept_positions_share,
  topup_games, topup_positions, topup_opponent`: the line's rating, its current version's SE and its floor (its
  current version's cold Elo in the cycle's fit, or with smooth > 1 its last K versions' mean; the floor that minus
  the margin or the SE; empty while unrated, when nothing is cut), the local set's games its entries play and how
  many of them it kept, the local set's games and positions it kept and their shares of the whole set (4 decimals),
  and its top-up games
  (self_games_topup; 0 without it or when nothing was cut), their positions and opponent (its champion's entry,
  `self` for its twin, empty without top-up games). `train_games`, `train_positions`, `data_price_s` and
  `data_cpu_s` are then the line's own set's: the kept games plus the top-up games. All empty in a cycle that trains
  nothing (an experiment's last).
- An experiment with rate_every adds last columns `rated, champion_age` (after train_filter's): 1 in a rated cycle
  and 0 in the others; the age of the line's champion in the cycle (the one its current version was tested against,
  or kept in an unrated cycle): the cycle minus the cycle it was crowned in, so 0 in cycle 1 and, for a line
  crowning in cycle c, the old champion's age in c and 1 in c + 1. With train_filter, an unrated cycle's
  `filter_rating`, `filter_se` and `filter_floor` are the last rated cycle's (state.json's `rate_carry`), and its
  kept games and top-up games are cut on them.
- An experiment with champion_se_kind "draw-aware" adds last columns `se_nominal, z_nominal` (after rate_every's):
  the champion test's nominal SE (p(1-p) per game, as `se_diff` is without the key) and `diff / se_nominal`; empty
  where `se_diff` is. `se_diff` and `z` are then the draw-aware SE and `diff / se_diff`, and decide.
- `data_price_s`: the priced CPU of the games this line trains on in this cycle (global and local lines: the
  qualifying games, and the self-games with self_games; self-play lines: their own rounds). `data_cpu_s`: the same
  games' measured CPU (information only).
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
global_carry_s, data_price_target_s, selfplay_price_s, prices_version`, and in a league with self_games also
`self_games, self_games_plies, self_games_price_s, self_games_proc_cpu_s, self_games_wall_s`, and in an
experiment with replay last `replay_games, replay_target, replay_available, replay_from, replay_to, replay_price_s,
replay_cpu_s`, and in an experiment with frozen_references last `local_games, local_line_line, local_self_games,
local_line_ref, local_ref_ref, local_random, local_line_line_share, local_self_games_share, local_line_ref_share,
local_ref_ref_share, local_random_share`, and in an experiment with reference_window last `ref_window_margin,
ref_window_lines, ref_window_excluded, ref_window_below`, and in a league with games_per_cycle last `games_per_cycle,
gpc_bound, train_games, train_positions`, and in an experiment with observers last `observers, observer_train_cpu_s`,
and in an experiment with cold_games last `cold_temperature, cold_share, cold_train, cold_rounds, cold_games,
cold_cpu_s, cold_proc_cpu_s, cold_wall_s, cold_price_s, cold_in_local`, and in an experiment with train_filter last
`filter_margin, filter_smooth, filter_set_games, filter_set_positions, filter_lines_cut, filter_cut_games,
topup_games, topup_positions, topup_price_s, topup_proc_cpu_s, topup_wall_s`, and in an experiment with rate_every
last `rated` (these groups in this order)

- `rated` (experiments with rate_every): 1 in a rated cycle, 0 in an unrated one. An unrated cycle plays no cold
  games: its `cold_rounds`, `cold_games`, `cold_cpu_s` and `cold_price_s` are 0, `cold_proc_cpu_s` and
  `cold_wall_s` empty, and `new_champions` 0.

- `filter_*`, `topup_*` (experiments with train_filter): the margin (`se` or Elo) and the smoothing (cycles); the
  local set before the filter (its games and positions); the lines the filter cut games from and the games it cut,
  summed over them (a line's own set is the local set less its cut, plus its top-up games: cycles.csv); the top-up
  games summed over the lines, their positions and price, and the replay-games process's measured CPU and wall time
  (0 without self_games_topup). All but the margin and the smoothing empty in a cycle that trains nothing (an
  experiment's last). The top-up games are not in `games`, `price_s`, the `tournament_*` columns,
  `data_price_target_s` or `data_cpu_target_s`.

- `cold_*` (experiments with cold_games): its temperature and share (as given) and train (1 or 0); the cycle's
  cold round-robins (`cold_rounds.csv`): their number and games, the games' measured CPU (`cold/game_cpu.csv`), the
  cold tournament process's CPU (its games, rating fit and tables) and wall time, the games' price; and
  `cold_in_local`, the cold games that joined the local set (0 without train, and in the last cycle, which trains
  nothing). The round-robins' columns (`rounds`, `games`, `tournament_cpu_s`, `price_s`, ...) stay the round-robins'.

- `observers`, `observer_train_cpu_s` (experiments with observers): their number, and their training's measured CPU
  in the cycle (0 in the last cycle, which trains nothing). `train_cpu_s` stays the participants'; `field`,
  `rounds`, `train_games` and `train_positions` are the participants' (the observers train on the same set).

- `games_per_cycle`, `gpc_bound` (leagues with games_per_cycle): the T that planned the cycle's rounds and its bound
  ("under T", "over T" or empty; config.json above); `field` is N and `rounds` R. Empty T in a cycle planned
  before T was set (it keeps the rounds it planned).
- `train_games`, `train_positions` (leagues with games_per_cycle): the cycle's training batch, the games and
  positions each line that trains on the league's games (local or global-league lines) trained on, from
  cycles.csv; where they differ (the global league's training windows) their mean, halves up (the report gives
  the range). An experiment's local lines all train on the same set: the qualifying games plus self-games (plus
  replayed games with replay). Empty in a cycle in which no such line trained (an experiment's last cycle).
- `tournament_cpu_s` and `ply2_cpu_s`: the sums of per-game CPU over all games, and over the qualifying ones.
- The `ply2_*` columns (here, in rounds.csv, copied.csv and state.json's `last_round_ply2_cpu_s`) are the qualifying
  games': a model at the entry ply plays in them (ply 2 unless `entry_ply` says otherwise). The names are kept: with
  `entry_ply` 3, `ply2_games`, `ply2_cpu_s` and `ply2_price_s` count, measure and price the games with a model at
  ply 3 (every round-robin game of an experiment without references), and the cycle report says "ply 3". In the
  global league of the 11-10 design they are the games with a model at any ply (not reference against reference;
  each model's own training set is in cycles.csv), and so is the batch's `ply2` column.
- `self_games*` (leagues with self_games): the cycle's self-games, their plies and price, and the replay-games
  process's measured CPU and wall time. They are not in `games`, `price_s` or the `tournament_*` columns.
- `global_rounds`, `global_cpu_s`, `global_ply2_cpu_s`: the global rounds copied for this cycle (experiments).
- `replay_*` (experiments with replay): the games replayed into the cycle's local set, the count wanted
  (round(S / (1 - S) x own)), the games kept in the window, the window's first and last cycle, and the replayed
  games' price and generation CPU (in their own cycles; information only). Empty in a cycle that built no local set
  (the last cycle). They are not in `data_price_target_s` or `data_cpu_target_s`.
- `local_*` (experiments with frozen_references): the cycle's local training set by kind of game: `line_line` (a
  round-robin game between two of the league's line entries), `self_games`, `line_ref` (a line entry against a
  frozen reference), `ref_ref` (two frozen references) and `random` (a game with a random reference), their sum
  `local_games` (the local lines' `train_games`), and each kind's share of it (4 decimals). Empty in a cycle that
  built no local set (the last cycle). diversity-p2 from cycle 2: 672, 96, 768, 144, 0 of 1,680.
- `ref_window_*` (experiments with reference_window): `ref_window_margin` the margin M; `ref_window_lines` the
  lines the window cut games from this cycle (0: inert, every line trained on the whole local set);
  `ref_window_excluded` the games it cut, summed over those lines (a line's own set is `local_games` less its cut,
  its cycles.csv `train_games`); `ref_window_below` the frozen reference entries below some line's floor,
  ";"-joined. All but the margin empty in a cycle that built no local set (the last cycle). Each line's cut, floor
  and the references below it are in driver.log and the report's Reference window line.
- `data_cpu_target_s`: the measured CPU of this cycle's qualifying games (experiments; information only: the
  decisions use `data_price_target_s`). With self_games, plus the replay-games process's CPU.
- `tournament_proc_cpu_s`: the tournament process's own CPU (games plus the rating fit and tables).
- The `*_price_s` columns are the priced counterparts: `price_s` and `ply2_price_s` of this cycle's games;
  `global_price_s` and `global_ply2_price_s` of the copied rounds (priced by the experiment's own prices);
  `data_price_target_s` what each line should train on (the qualifying games' price, plus the self-games' with
  self_games); `selfplay_price_s` the
  self-play lines' priced generation, summed. `global_carry_s` is the state's after the cycle. `prices_version` is
  the league's.
- Columns ending in `_cpu_s`, `_wall_s` and `wait_s` are measured (information only); decisions use prices.

### observers.csv (experiments with observers; one row per observer per cycle)

`cycle, date, line, model, version, version_name, train_games, train_positions, train_cpu_s, data_cpu_s,
data_price_s, next_version, note, label`

- `version`, `version_name`: the observer's version before the cycle; `next_version` after it (version + 1 when it
  trained; the same in the last cycle, which trains nothing, and after a failed training, whose reason is in
  `note`).
- `train_games`, `train_positions`, `train_cpu_s`, `data_cpu_s`, `data_price_s`: as cycles.csv's for a local line
  (the local set's games, the positions the training binary reported, the measured training CPU, and the set's
  measured CPU and price). They equal the participants' cycles.csv values every cycle (the same set); empty in the
  last cycle.
- An observer has no cycles.csv, history.csv, entries.csv, events.csv or self_games.csv row and no tour/ game.

### cold_rounds.csv and cold/ (experiments with cold_games)

`cold_rounds.csv`: rounds.csv's columns, one row per cold round; `t` is the tournament index in `cold/games.csv`
(from 0, its own numbering). `cold/` has tour/'s files for the cold round-robins (players.txt and retired.txt as
tour/'s, the same field; games.csv, game_cpu.csv, ratings.csv, head_to_head.*, elo_history.*, tournament.log).
`coldpool/` (with train, while the cycle runs) holds the cycle's saved cold games, as `pool/` the round-robins';
removed with it at the cycle's end.

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
  - `entered`: a line joins with its version 0 (entry `<line>@0-2`, or `<line>@0-<P>` with `entry_ply` P or a
    global-league model's ply P); detail
    `untrained`, or with a warm start `warm start: <path> (md5 <md5>; the first training --init-forget F)` (no
    `--init-forget` part when F is 1; a tree's adds `; training state <path> (md5 <md5>)` inside the parentheses,
    and its first-training part is `the first training [--warm-reset] [--start-leaf-forget F]`, none without them);
  - `first-champion` (detail `the untrained version`, or `the warm start (version 0)`);
  - `new-champion` (value z);
  - `retired` (global, patience; in the 11-10 design entry the current version's entry, value the level in %,
    detail `patience reached 100% (rule 6: N positions without a new champion since cycle C, M this cycle;
    20,000,000 retire)`);
  - `anchor-locked` (value: the interval's lower bound);
  - `anchor-available`: an anchor was out of its interval by more than the margin for out_cycles straight cycles;
  - `anchor-replaced`: the old anchor leaves at the moment a new one takes the interval;
  - `copied` (experiments: global rounds copied; value the number of rounds; detail "global cycle C rounds t1 t2
    ...; ply-2 price X s and carry Z s cover N rounds at Y s (how the round was estimated)");
  - `wait-start` and `wait-end` (value: the seconds waited);
  - `done` (experiment finished; value the cycle; detail `N cycles`, or with the plateau rule `plateau: no new
    champion since cycle X (N cycles)`).
- With rate_every, `new-champion` and the plateau's `done` come on rated cycles only.

### history.csv (every field entry, every cycle)

`cycle, entry, line, version, version_name, ply, role, elo, ci95, games`

- `role`: `current`, `champion`, `current+champion`, `anchor`, `reference` or `frozen-reference` (an experiment's
  frozen_references: line empty, version the source version, version_name empty).
- With rate_every an unrated cycle's rows list its field with `elo`, `ci95` and `games` empty (no cold games, no
  fit that decides).

### entries.csv (rewritten each cycle)

`entry, line, version, version_name, ply, role, in_field, joined_cycle, left_cycle, model_file, spec`

- Every entry that was ever in the field, and the next cycle's new ones, with its latest role.
- `in_field` is 1 if it plays in the next cycle. `joined_cycle` is its first cycle; `left_cycle` the last cycle it
  played (empty while it stays).
- `model_file`: the version's model; a frozen reference's copy in frozenref/; empty for the random references.

### anchors.csv (11-09 global leagues only, rewritten each cycle; the 11-10 design has no anchors)

`interval_lo, interval_hi, entry, line, version, version_name, locked_cycle, locked_elo, elo, ci95, outside_streak,
available`

- Intervals are 100 Elo wide on the ply-1 scale, from 100 up: [100,200), [200,300), and so on. `available` is 1
  once the anchor has given its interval up. It stays in the field until a new anchor takes the interval.

### copied.csv (experiments with the global option)

`cycle, global_cycle, t, games, ply2_games, cpu_s, ply2_cpu_s, copied_at, price_s, ply2_price_s`

- `price_s`, `ply2_price_s`: the round's games priced by the experiment's prices (the global league's kinds).

### self_games.csv (leagues with self_games; one row per self-game)

`cycle, t, entry, white, black, result, plies, price_s`

- `t`: the round (tournament index) whose field played it; `entry` plays both sides, so `white` and `black` are
  `entry` too (the analysis tools never see a game of a player against itself in tour/games.csv). `result` as the
  tournament writes it (`1-0`, `0-1`, `1/2-1/2`); `price_s` the game's price (the entry's tournament prices at its
  ply, both sides; 6 decimals).
- How they are played: the tournament binary's `replay-games`, which replays a tournament's games exactly (the same
  players, policy, temperature and move choice), with each entry paired against a twin (its spec and depth under
  the name `<entry>.twin`). Round t's game of the i-th self-game entry (field order) is seeded
  `derive_seed(seed + 1000003, t, g)`, g its pairing's index in replay-games' players list (the entries, then the
  twins), so the games replay exactly and share no seed stream with the round-robins.

### selfpool/ (leagues with self_games, while the cycle runs)

`players.txt` (the self-game entries, then their twins, `NAME DEPTH SPEC`), `games.txt` (`t white black`, one line
per game asked for), `games.csv` (the cycle's self_games.csv rows) and `round<t>/<entry>~<entry>/` (the games, as
`tournament --save-games` writes them; renamed from replay-games' `<entry>~<entry>.twin`). Training links them with
the round-robins' games, ordered by round and names. Deleted at the end of the cycle.

### topuppool/ (experiments with self_games_topup, while the cycle runs)

`players.txt` (for each line with top-up games, its current version then its opponent, `NAME DEPTH SPEC`; a twin as
`<entry>.twin`), `games.txt` (`t white black`, replay-games' t, one line per game asked for), `games.csv` (`t, j,
line, white, black, result, plies, price_s, replay_t`: t the round the game belongs to, j its index among the
line's top-up games, replay_t replay-games' t; a twin named as its entry; price 6 decimals) and
`round<t>/<white>~<black>~<j>/` (the games, as `tournament --save-games` writes them). Only the line it was played
for trains on a top-up game. Deleted at the end of the cycle (a cycle cut short replays them).

### replaypool/ (experiments with replay)

`c<cycle>/round<t>/<white>~<black>/`: the cycle's own local games (hard links to the pool's and selfpool's game
files, so pool/ and selfpool/ can go at the cycle's end), and `c<cycle>/games.csv` (`t, white, black, price_s,
cpu_s`: each game's price and measured CPU, 6 decimals; a self-game's CPU is the self-games process's CPU shared by
price; a game without a CPU row gets the cycle's mean, as rounds.csv). games.csv is written last: a cycle is kept
when it exists, and a keep interrupted before it is redone. After each cycle, the cycles before the next cycle's
window (c + 1 - W) are deleted; a finished experiment keeps its last window (W cycles of games: about 40 KB a game
on disk, ~1.5 GB for 50 cycles of 768 games) until deleted by hand. The cycle report's Replay line gives the count replayed (and how many are
self-games), the count wanted and kept, the cycles they came from with the count from each, their price, and the
local set's own + replayed games; its Measured CPU line adds the replayed games' generation CPU.

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
batch/games.csv      t, white, black, result, plies, cpu_s, ply2 (1 if a model played at ply 2; 11-10 design: a
                     model at any ply), price_s
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
- `source`: `measured: ...`, `derived: ...` or `estimate: ...` (how); `games` and `plies`: the measurements behind a
  measured row.
- A version's rows never change. A version may gain rows for keys it lacked, which no league made before it needs:
  P2's tournament rows at ply 3 and at ply 4 (2026-11-11, for `entry_ply` 3 and 4; leagues/prices/P2.md, sections
  Ply 3 and Ply 4; at ply 4 only lr is measured, the other kinds' rows are estimates). A league
  made before keeps its copy without them. `leagues.py prices SPEC.json` (leagues/calibrate.py) measures, fits and
  appends a new version, with its report `leagues/prices/<version>.md` (the constants, the calibration players, and
  priced / measured CPU by kind, depth, stage, model size, game length and source) next to its spec
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

- Rows added by hand: make_recipes.py's HAND list, models that never ran live (sources `-`), written after the
  generated rows under a `# Added by hand` comment and kept when make_recipes.py is re-run. Each is a generated row
  with one change. Add new ones there, not to recipes.txt alone.
- T-L-Fe (HAND) is T-L-Le with an FM error router (`--router fm`). Its binaries are chessai-fmrouter, the only build
  with FM routers (models/from-scratch/fmrouter/summary.txt). That build writes the same files as chessai-hybrid4gfx
  for every other model. chessai-hybrid4gfx can't load FM routers, so an experiment with T-L-Fe also needs
  `"tournament": {"binary": "<abs path of chessai-fmrouter>"}` in its spec. Its calibration games use that binary
  too.
- D-L-Ne-home (HAND, not run yet) is D-L-Ne with `--subrouter-target home`: after a router splits, the new router
  keeps the old one and its target (which shared child predicts best), and its class i sends the position to
  sub-router i (models/from-scratch/bddhome/summary.txt). Its binaries are chessai-bddhome2, a copy of
  chessai-hybrid4gfx with that option, which trains every other model identically (chessai-bddhome is the first
  build: the same results, slower with the option). The option only changes training and is not in model files, so
  chessai-hybrid4gfx plays its models and no tournament override is needed. Its state files record the option: they
  resume only with it.
- T-L-Nc-merge-anchor (HAND, drafted, not run) is T-L-Nc-merge with the LR anchor `--split-anchor-l2 L`: a split's
  new LR leaves (and reset leaves) keep a frozen copy of the parent's weights and are pulled towards it by an L2
  penalty of L per example (models/from-scratch/lranchor/summary.txt). L scales as 1/split threshold; the row's
  8e-5 at 5M is 4e-4 at 1M, a third of the match to the probit trees' split noise 0.03. Its binaries are
  chessai-hybrid6an, a copy of chessai-hybrid6 with that option, which trains every other model byte-identically to
  chessai-hybrid6 (models, states and logs checked). Model files are plain LR trees (chessai-hybrid4gfx and
  hybrid13 load and score them identically), so no tournament override is needed. State files hold the anchors
  only when L != 0, and resume only with the same option. Its self-play against chessai-hybrid4gfx's is unchecked:
  use it in local experiments.

## Changes (newest first)

- 2026-11-12 22:56:00: `leagues.py extend LEAGUE --cycles N [--dry-run]` (config.json above, "Cap history"; the league
  dir's extend/cap-<C>/), for experiments that ended at their cap before their plateau rule (the user, ~21:18): the
  cap cycle runs again as an ordinary cycle and the run goes on as if N had been its cap from the start. Scratch proof
  (extend_test, in the full selftest): experiments run to cap C, extended and run on gave the games, cold games,
  ratings, models and training states, records, reports, config and state of the same experiments run fresh with the
  final cap, byte for byte (184 items), but totals.csv's `cal_made` of the cycles that ran again: cap 5 -> 13 (5 lines
  with a growing tree and DAG, rate_every 4, the draw-aware test and the guard; owncal/ and scan-cache/ deleted, the
  cycle that runs again interrupted 6 times), 2 -> 5 -> 6 (an observer, cold games that train, openings) and 2 -> 4 (a
  warm start at ply 3); 7 crownings of the old last cycles undone and made again; a changed original's games, then its
  records, stopped the cycle that runs again. Without `cap_history` nothing changes: the running std-v1-p2-s1, -x3 and
  -s2, explore-t015-p1, probit-field-p1 and residual-p2 (boolean set flags, an entry's hybrid6rp), re-created from
  their own specs and run on the new driver, gave their games (std-v1-p2-s1 9,792 in 5 cycles, std-v1-p2-x3 9,792 in 5
  cycles, std-v1-p2-s2 5,712 in 3 cycles, explore-t015-p1 9,792 in 5 cycles, probit-field-p1 9,792 in 5 cycles,
  residual-p2 9,348 in 5 cycles), records, reports and 260 model and state files byte for byte
  (models/from-scratch/extend/repro_extend.py). The full selftest passed (245 rule checks, 19 runs), and the quick one
  after the swap. Swapped in at 22:56:00 (md5 b89c6a4c...; the previous file kept as leagues/leagues.py.before-extend,
  md5 1315b8ef...). Also leagues/library/build_index.py (end reason `extending`; a final at an old cap keeps its
  snapshot) and the dashboard (a "cap raised from C to N" line).

- 2026-11-12 18:56:30: a tree's warm start may forget its leaves (`start_forget` F not 1: `--start-leaf-forget F`)
  and reset a growing tree's or DAG's trials and windows (`"start_reset": true`: `--warm-reset`), on a train binary
  with the flags (chessai-grow7), and a model entry may give its lines' `"train_binary"` ("Warm starts" above), for
  the ply-2 rung of training standard v1 (models/STANDARD.md; leagues/specs/standard/std-v1-p2-*.DRAFT.json). Without
  the keys nothing changes: std-v1-p1-s1 and -s2 re-created from their own specs, pinned as their runs were, and run 5
  cycles on the new logic gave their games (9,792 each), records and 48 version 0-5 model files byte for byte
  (models/from-scratch/std-v1-p2/prep/repro_warm2.py). Scratch proof (models/from-scratch/std-v1-p2/proof): the s1
  draft warm from a frozen std-v1-p1-s1@340 at ply 2, 5 cycles; every version 0 equals its source, cycle 1 carries
  exactly each line's flags and cycle 2 none, every version 1 equals its cycle-1 training replayed from the source,
  and every control without the flags differs (35 checks; replicate_w2.py). The full selftest passed (245 rule checks, 18 runs;
  models/from-scratch/std-v1-p2/prep/selftest-full.out), and the quick one after the swap. Swapped in at 18:56:30 (md5
  1315b8ef...; the previous file kept as leagues/leagues.py.before-warm2, md5 dfba7832...).

- 2026-11-12 17:22:24: experiment options `champion_se_kind` and `plateau.guard_z` (config.json above, "Champion SE kind" and
  "Plateau guard"; the index's `champion_se_kind`): the champion test on the draw-aware (HC0) SE, its nominal SE and
  z kept in cycles.csv's new last columns `se_nominal, z_nominal`; the plateau's stop held while a line's test has z
  above G. create now refuses a `champion_se` that isn't a number above 0. Without the keys nothing changes:
  grow-dag-p1's, filtered-p1's and treeonly-p1-open's own specs re-created and run 3 cycles on the new logic gave
  their games (4,608, 8,400 and 4,892), records and 24, 20 and 16 version 0-3 model and state files byte for byte
  (models/from-scratch/std-v1/leagues/repro_ds.py). The selftest's rule checks compare the draw-aware SE with a dense
  HC0 sandwich on games with draws and check that it doesn't depend on the pinned player; drawse_test refuses 9 bad
  specs and runs scratch experiments with champion_se_kind draw-aware (the last cycle's SEs, z's and decisions
  recomputed from its cold fit), guard_z 0 (the stop held while a test's z is above 0) and guard_z 1000 (ends as
  without the guard). The full selftest passed (234 rule checks, 17 runs; std-v1/selftest-full.out), and the quick
  one after the swap.
  Swapped in at 17:22:24 (md5 dfba7832...; the previous file kept as leagues/leagues.py.before-drawse, md5
  6fb93c92...).

- 2026-11-12 16:56:27: warm starts for tree kinds (bdt, bdd, ht, tan; "Warm starts" above), for the ply-4 mixed field
  (leagues/specs/ply4/probit-p4w.DRAFT.json): version 0 copies the start version's training state and model, `start`
  records `state` and `state_md5`, the first training is `train --start <version 0's state>`, and create refuses
  start_forget other than 1, a missing state, and a start line of another recipe or other train arguments. Without a
  tree warm start nothing changes: grow-dag-p1's, filtered-p1's and treeonly-p1-open's own specs re-created and run 3
  cycles on the new logic gave their games (4,608, 8,400 and 4,892), records and 24, 20 and 16 version 0-3 model and
  state files byte for byte (models/from-scratch/ply4-prep/leagues/repro_tw.py). Scratch proof (ply4-prep/proof2):
  the draft's six lines at ply 1: version 0's files and own games equal the sources', and each version 1 equals its
  cycle-1 training replayed with `--start` the source's state (or model). The full selftest passed on it (232 rule checks, 16 runs; ply4-prep/selftest-full.out), and the quick one after the swap. Swapped in at 16:56:27 (md5 6fb93c92...;
  the previous file kept as leagues/leagues.py.before-treewarm, md5 1ade33c3...).

- 2026-11-12 16:25:27: experiment option `rate_every` (config.json above, "Rate every N"; the index's `rate_every`):
  only cycles 1, N + 1, 2N + 1, ... play the cold games and decide (champion tests, crownings, the plateau rule,
  train_filter's floors); the others play and train, keeping the last rated cycle's champions and floors. The cap
  must be a rated cycle; the plateau is checked on rated cycles only and counts training cycles; a scaled plateau
  needs `rate_scale` (sqrt, linear or none; no default). New: cycles.csv `rated` and `champion_age`, totals.csv
  `rated`, the decision `unrated-cycle`, state.json `champion_cycle` and `rate_carry`. Without the key nothing
  changes: grow-dag-p1's, filtered-p1's and filtered-p1-twin's own specs re-created and run 3 cycles on the new
  logic gave their games (4,608, 8,400 and 8,400), records with their ratings and 24, 20 and 20 version 0-3 model and
  state files, byte for byte (models/from-scratch/rateevery-build/leagues/repro_rate.py). The selftest's rule checks
  cover the schedule and K by rate_scale; rate_every_test runs a scratch experiment without the key and with
  rate_every 4 (straight, interrupted 9 times, and with a 1-cycle plateau): cycles 1, 5 and 9 rate, every unrated
  cycle's floors, kept sets and top-up games come from the last rated cycle's records, cycles 1-2 equal the run
  without the key, straight equals interrupted, the plateau run ends on rated cycle 5, 14 bad specs refused; the
  full selftest passed (227 rule checks, 16 runs). Swapped in at 16:25:27 (md5 1ade33c3...; its last change after
  those checks, status's next-rated-cycle text, outside every run; the quick selftest and rate_every_test passed on
  it; the previous file kept as leagues/leagues.py.before-rateevery, md5 f7177b40...). The dashboard (md5
  f3bf796d..., swapped 16:25:27, the server restarted 16:25:36; before: 67b32e47...) takes ratings and decisions from
  the rated cycles only, marks the unrated ones and shows each line's champion age (DASHBOARD.md).

- 2026-11-12 15:39:00: experiment option `openings` (config.json above; the index's `openings`): every round-robin
  game, self-game and top-up game opens with K uniformly random plies (or the movers' log-odds softmax at T), drawn
  from the game's own random numbers; the cold games and own calibration games don't. The binary is
  chessai-hybrid19 (`--opening-plies`, `--opening-temperature` on `tournament` and `replay-games`; without them
  byte-identical to chessai-hybrid18). Without the key nothing changes: treeonly-p1-t002's, filtered-p1's and
  grow-dag-p1's own specs re-created and run 3 cycles on the new file gave their games (4,892, 8,400 and 4,608),
  records with their ratings and 16, 20 and 24 version 0-3 model and state files, byte for byte
  (models/from-scratch/openings-build/leagues/repro_open.py). The selftest's rule checks cover the flags and the
  refusals; openings_test runs a scratch experiment without and with 4 random plies (straight, interrupted 5 times,
  and with train_filter's top-up games): every round-robin, self-game and top-up command opens and no cold-games or
  own-calibration command does, cycle 1's cold games and version 0's own calibration are the run without openings',
  every line's training positions are its set's plies (the opening plies train), straight equals interrupted, 8 bad
  specs refused; the full selftest on the final file passed (225 rule checks, 15 runs). Swapped in at 15:39:00 (md5
  f7177b40...; the previous file kept as leagues/leagues.py.before-openings, md5 8748a886...).
- 2026-11-12 14:02:33: train_filter's rule as the user set it (13:22-13:24, superseding analyst 6.87; nothing had run
  with the filter): a line's floor is its current version's cold Elo minus that rating's standard error in the
  cycle's fit (`"train_filter": {"margin": "se"}`; numeric margins stay a fixed Elo), no averaging (`smooth`
  defaults to 1; the option stays, numeric margins only), and no exemption for own games: every game of the local set
  trains the line only if both players are rated at or above its floor; the top-up plays the champion if it is at or
  above the floor, else the line's twin (config.json above). Decide's `filter_se`, state.json's
  `current.filter_ratings` (was `filter_smoothed`), the plan's `se` and `own_kept`, cycles.csv's `filter_se` and
  `filter_own_kept`, and the log and report lines say so. Without the key nothing changes: treeonly-p1-coldtrain's,
  treeonly-p1-coldrate's and probit-f10-p1's own specs re-created and run 3 cycles on the new file gave their games
  (4,892, 4,892 and 4,608), records with their ratings and 24 version 0-3 model and state files each, byte for byte
  (models/from-scratch/filter-build/r3/repro_test3.py). The selftest's rule checks cover the rule (no exemption),
  the floor (filter_floor) and the margin's refusals; train_filter_test recomputes every line's rating, SE, floor,
  kept set, own games kept, top-up games and opponent from the records (on the SE, at margin 0 straight and
  interrupted, on the SE with cold games that rate only, at margin 0 smoothed over 2 cycles; 12 bad specs refused);
  the full selftest on the final file passed (223 rule checks, 14 runs). Swapped in at 14:02:33 (the previous
  file kept as leagues/leagues.py.before-filter3, md5 05e0ec76...).
- 2026-11-12 13:15:12: train_filter's rule revised (analyst 6.87; nothing had run with the first one, so the new rule
  is the default): a line's own games (one of its entries plays them) always train it, whoever the opponent; another
  game only if both players' ratings, smoothed over the last K cycles, are at or above the line's smoothed rating
  minus the margin (config.json above). config.json's `train_filter` gains `smooth` (default 5, written for specs
  with `{"margin": M}`), decide's `filter_rating`, state.json's `current.filter_smoothed`, the plan's `smooth`,
  `rating` and `own_games`, cycles.csv's `filter_rating` and `filter_own_games`, totals.csv's `filter_smooth`, and
  the log and report lines say so; the top-up's opponent no longer depends on the floor (its games are the line's
  own). Without the key nothing changes: treeonly-p1-coldtrain's, treeonly-p1-coldrate's and probit-f10-p1's own
  specs re-created and run 3 cycles on the new file gave their games (4,892, 4,892 and 4,608), the cycles, history,
  rounds, self_games, events and totals rows of cycles 1-3 with their ratings (observers and cold_rounds too),
  config.json (but created and root) and 24 version 0-3 model and state files each, byte for byte
  (models/from-scratch/filter-build/repro_test2.py). The selftest's rule checks cover the new keep rule, the
  smoothing and train_filter.smooth's refusals; train_filter_test recomputes every line's smoothed rating and floor,
  own games, kept set, top-up games and opponent from the records (filtered at margin 0 smoothed over 5 and over 2
  cycles; 10 bad specs refused); the full selftest on the final file passed (222 rule checks, 14 runs). Swapped
  in at 13:15:12 (leagues.py md5 05e0ec76...; the previous file kept as leagues/leagues.py.before-filter2, md5
  6f801ac3...).
- 2026-11-12 12:26:58: experiment options `train_filter` and `self_games_topup` (config.json above; state.json's
  `current.train_filter` and `current.topup`, the decisions' `filter_floor`, the `topup-games` phase; cycles.csv's
  and totals.csv's filter and top-up columns, topuppool/ and topup_games.log, only with the keys; the index's
  `train_filter` and `self_games_topup`; the report's Train filter line and its CPU lines). The user's Elo-filtered
  multi-kind experiment (~11:11; analyst 6.85 (b)). Without the keys nothing changes: treeonly-p1-coldtrain's,
  treeonly-p1-coldrate's and probit-f10-p1's own specs re-created and run 3 cycles on the new file gave their games
  (4,892, 4,892 and 4,608), the cycles, history, rounds, self_games, events and totals rows of cycles 1-3 with their
  ratings (observers and cold_rounds too), config.json (but created and root) and 24 version 0-3 model and state
  files each, byte for byte (models/from-scratch/filter-build/repro_test.py). The selftest adds 4 rule checks and
  train_filter_test (experiment k, 3 lines, 4 cycles: without the keys, filtered at margin 0, filtered and topped up
  straight and interrupted 6 times (faults at the top-up phase, after its games ("topup-partial", new), after a
  line's training, in the records), and topped up with cold games that rate only; 9 bad specs refused); the full
  selftest on the final file passed (221 rule checks, 14 runs). Swapped in at 12:26:58 (leagues.py md5
  6f801ac3...; the previous file kept as leagues/leagues.py.before-filter, md5 8f5cb216...). The state.json phase
  list now also names `cold-games` (missing since 11:29).
- 2026-11-12 11:29:15: experiment option `cold_games` (config.json above; state.json's `current.cold`; totals.csv's
  `cold_*` columns, cold_rounds.csv and cold/, only with the key; the index's `cold_games`; the report's Cold games
  line and its CPU lines). The user's decision at 10:54. Without the key nothing changes: probit-f10-p1's own spec
  re-created and run 3 cycles on the new file gave its 4,608 games (72 rounds), the cycles, history, rounds,
  self_games, events and totals rows of cycles 1-3 with their ratings, config.json (but created and root) and 24
  version 0-3 model and state files byte for byte (models/from-scratch/coldgames-build/repro_test.py). The selftest
  adds 5 rule checks and cold_games_test (a scratch experiment, 2 lines and an observer, 3 cycles: rating only plays
  and trains as without the key through cycle 2, versions 0-2 byte for byte; training on them, cycle 1's round-robins
  and cold games are the rating-only run's and the lines and the observer train on its set plus the cold games;
  interrupted 7 times (faults at the cold-games phase, in its round-robins, after a line's and the observer's
  training, in the records) equals the straight run, reports too; the in-run ratings are cold/'s fit). Swapped in at
  11:29:15 (leagues.py md5 8f5cb216...; the previous file kept as leagues/leagues.py.before-coldgames, md5
  c385ee34...). wait_not_busy's second look (10:58:57, md5 a1801c9e... to c385ee34...) changed timing only.
- 2026-11-12 10:23: experiment option `observers` (config.json above; state.json's observer status and
  `current.observed`; lines.csv's status `observer`; observers.csv; totals.csv's `observers, observer_train_cpu_s`,
  only with the key; the index's `observers` and `observer_models`; the report's Observers line and the Measured CPU
  line's observers' training). The user, 11-12 ~10:15. Without the key nothing changes: probit-f10-p1's own spec
  re-run 3 cycles in a scratch root on the new file gave its 4,608 games (72 rounds), its cycles, history, rounds,
  self_games, events and totals rows of cycles 1-3 with their ratings, its config (but created and root) and its
  24 version 0-3 model and state files byte for byte. With observers: a scratch experiment (T-P-Ne capped x2 at
  ply 1, self-games) with and without observers F-P x2 and T-P-Ne capped x2 gave the participants the same games,
  ratings, models and records for 3 cycles, and the T-P-Ne observers held the participants' models byte for byte.
  The selftest adds observers_test (no key; observers straight; observers interrupted 8 times, faults at the
  tournament, the self-games, after a participant's and an observer's training ("train-observer", new) and in the
  records; 5 bad specs refused). leagues.py was swapped atomically at 10:23 (md5 a5afab4c...; the old one is
  leagues/leagues.py.before-observers, md5 f20818ff...), then at 10:36 and 10:41 for the selftest's test (md5
  a1801c9e...). treeonly-p1 (observers F-P x2, added before cycle 1 by
  models/from-scratch/observers-build/add_observers.py) and fponly-p1 (observers T-P-Ne capped x2) run on it.
- 2026-11-12 02:43: experiment option `reference_window` {"margin": M} (config.json above, with frozen_references
  only; decide's `ref_floor`; state.json `current.ref_window`; per-line sets trainsets/<line> for the lines it cuts;
  totals.csv's `ref_window_margin, ref_window_lines, ref_window_excluded, ref_window_below`, only with the key; the
  report's Reference window line; the Training batch line names a cut), for diversity-p2c (theory-2 6.57). Without
  the key nothing changes: diversity-p2's own spec run 3 cycles in a scratch root on the new file gave its games,
  records and version 0-3 models byte for byte (4,176 rows, 16 models; the reports differ only in the references'
  version strings, given as absolute dirs). With it: diversity-p2c's config at margin 30 and at 1e9 gave the same
  league for 3 cycles (the window inert: games, records, 16 models; only the window's margin column and line
  differ); diversity-p2's config with margin 30 (references the lines pass) cut from cycle 6, every cut agreeing
  with the cycle's ratings (176 checks: the logged floors are history.csv's Elo minus 30, every cut reference below
  its line's floor and every kept one at or above it) and with the records (totals, driver.log, cycles.csv
  train_games: the local set less the cut), its games of cycles 1-6 diversity-p2's (8,592) and its models up to
  version 6 diversity-p2's but the one line cut in cycle 6. leagues.py's selftest adds 4 rule checks (the cut, the
  config's refusals, the batch line) and a run (refwindow_test: no key, margin 1e9 and margin 0 over a source
  experiment's versions 0 and 2; 6 bad specs refused). The full selftest passed on the new file (213 rule checks
  and every run). leagues.py was swapped atomically at 02:43:00 (md5 f20818ff...; the old one is
  leagues/leagues.py.before-refwindow, md5 6a9c22ce...); grow-l-p1's cycle 280 and the global league's cycle 3441
  ran on it as before.
- 2026-11-12 01:48: `games_per_cycle` T for experiments and the global league (config.json above: R =
  round(T / (N (N - 1))) rounds a cycle, from 1 to 48, in place of rounds_per_cycle and min_rounds; state.json
  `current.games_per_cycle`; totals.csv's `games_per_cycle, gpc_bound, train_games, train_positions`, only with the
  key; the report's rule text and Training batch line). `games_per_entry` stays (the global league's, unset) and
  the two are exclusive. Without the key nothing changes: the old and new files gave the same games, ratings,
  models, records, reports, config and listing on a probit-late-p2-shaped experiment (3 cycles); probit-late-p2's
  own spec run 3 cycles in a scratch root on the new file gave its games, self-games and version 1-3 models byte for
  byte; all 62 configs (every experiment and the global league) give the same pricing kinds, needed prices,
  phases, current and next field and self-game entries under both files. leagues.py's selftest adds rule checks
  (R's rounding, cap and floor; configs; 8 refused specs; the global option's copy plan with R) and a run (gpc_test:
  an experiment straight and interrupted 5 times, identical; R, games, self-games and the training batch from the
  records; a global league; run refusing bad configs); the full selftest passed (210 rule checks and every run).
  Pilots at T 1584 (models/from-scratch/gpc/): probit-late-p2's spec (4 lines, ply 2) played N 4: 48 rounds (under
  T), 576 games, then N 8: 28 rounds, 1,568 games, 224 self-games, training 1,792 games and ~340k positions a cycle;
  router-p1's spec (10 lines, ply 1) N 10: 18 rounds, 1,620 games, then N 20: 4 rounds, 1,520 games, 80
  self-games, training 1,600 games and ~180k positions, straight and interrupted 4 times identical; a global league
  (F-P and F-L at ply 1, F-P at ply 2) N 7: 38 rounds, 1,596 games, then N 10: 18 rounds, 1,620 games. leagues.py
  was swapped atomically at 01:48:03 (md5 6a9c22ce...; the old one is leagues/leagues.py.before-gpc, md5
  9a0ff25d...); diversity-p2's cycle 124 and the global league's cycle 3048 ran on it as before.
- 2026-11-12 01:20: experiment option `frozen_references` (config.json above; entry names `<name>@<v>F-<P>`,
  role `frozen-reference`; frozenref/; state.json `current.frozen_cal` and `current.local_mix`; totals.csv's
  `local_*` columns, only with the option; the report's Frozen references and Local set lines), for diversity-p2.
  Without it nothing changes: the old and new files gave the same games, ratings, models, records, reports, config
  and league listing on a probit-late-p2-shaped experiment (3 cycles); probit-late-p2's own spec run 3 cycles in a
  scratch root on the new file gave its games, self-games and version 1-3 models byte for byte; and all 61 live
  configs give the same pricing kinds, needed prices, phases, next field and self-game entries under both files.
  leagues.py's selftest adds a run (frozenref_test: a source experiment, then an experiment with two of its
  versions as references, straight and interrupted 5 times: the same games, models, records and reports; the
  counts by kind from the records; the own games and calibrations as the source's; a changed copy stops the
  cycle; 8 bad specs refused). The full selftest passed on the new file (198 rule checks and every run). A pilot of
  leagues/specs/ply2/diversity-p2.json (scratch root, 3 cycles, straight and interrupted 4 times, identical): the
  four error-routing-p1 v314 references' own games and fits equal error-routing-p1's; from cycle 2, 1,584 games a
  cycle (line-line 672, line-ref 768, ref-ref 144) plus 96 self-games. leagues.py was swapped atomically at
  01:20:14 (md5 9a0ff25d...; the old one is leagues/leagues.py.before-frozenref, md5 f5bdba23...); the global
  league's cycle 2843 and router-p1's cycle 274 ran on it as before.
- 2026-11-11 ~22:00: experiment option `replay` (config.json above; state.json `current.replay`; totals.csv's
  `replay_*` columns and replaypool/, only with replay; the report's Replay line). Without it nothing changes: the
  old and new files gave the same games, ratings, models, records, reports, config and league listing on a
  probit-late-p2-shaped experiment (F-P x2, T-P-Ne x2, self-games, ply 2, 3 cycles), and none of the 57 live
  configs has the key. leagues.py's selftest adds the option's checks (config, counts, sample, window, prune) and
  a run (straight and interrupted 4 times: the same games, models, records and reports; the counts, prices and
  report lines from the records; the recorded sample recomputed). The full selftest passed on the new file (198
  rule checks and every run); leagues.py was swapped atomically at 21:55:07 (md5 f5bdba23...; the old one is
  leagues/leagues.py.before-replay, md5 8ef0f1f0...).
- 2026-11-11 13:50: the global league of the 11-10 design (models/NEW-DESIGN.md "The global league"; config.json
  above; lines.csv, cycles.csv, state.json `patience`, events `retired`). Spec models are `{"recipe", "ply"}`
  objects. Each model is a line `<recipe>.p<ply>` with entries at its ply only. There are no anchors (no
  anchors.csv). Each model trains on the cycle's games and self-games whose players are rated at or above its Elo
  minus `train.margin` (30). Rule 6 retires a model after 20,000,000 training positions without a new champion,
  not within 5 cycles of its entry or last crowning. `add` takes `--ply`.
  - Refusals: run and add refuse a global league of the 11-09 design; the experiments' global option is refused
    against the new design.
  - No experiment's files change: mirror runs of error-dag-p1 and error-dag-probit-p1 gave identical records
    under the old and new files, and all 52 live configs give the same fields, prices, phases and lines.
  - leagues.py was swapped atomically at 13:50:45 (md5 4c4a1194...; the old one is
    leagues/leagues.py.before-global11, md5 e7fd28b7...) after the full selftest passed on the new file (174 rule
    checks and the replay, options, self-games at plies 1-3 and warm-start runs).
  - dashboard.py renders the new design: no anchors, and patience as a level in %.
- 2026-11-11 11:02: a flag value that starts with "-" (a spec's `set` or `sweep`, e.g. `"--revise": "-2"`)
  now reaches the binary as one token, `--revise=-2` (clap read a separate `-2` as a flag and refused it), and
  config.json's `train_args` / `selfplay_args` hold it that way; parse_flags reads `--flag=value` tokens as the
  flag and its value. No other value's form changed: no league's config held a negative value or a
  `--flag=value` token. leagues.py swapped atomically at 11:02 (md5 e7fd28b7...; the old one is
  leagues/leagues.py.before-negflags, md5 92aa7bbb...) after the full selftest (145 rule checks and the
  replay, options, self-games at plies 1-3 and warm-start runs) passed on the new file; three experiments were
  running and went on cycling.

- 2026-11-11 09:15: leagues/recipes.txt and make_recipes.py's HAND: row T-L-Nc-merge-anchor (above) on
  bin/chessai-hybrid6an; make_recipes.py re-run into a scratch file gives recipes.txt byte for byte. Draft spec
  leagues/specs/batch5/lr-anchor-p1.DRAFT.json (not created). No other row, file or league changed.

- 2026-11-11 03:27: `entry_ply` any whole number from 1 (4 for theory-2 6.17's W4; config.json above) and warm
  starts (`start` and `start_forget` in a spec's model entry; config.json `lines[].start`; the details of the
  `entered` and `first-champion` events). leagues/prices.csv: P2 gained 11 tournament rows at depth 4 (lr measured
  on chessai-hybrid10 in arenas/diag/flat-ply4-interim-cold, the other kinds estimates; leagues/prices/P2.md, section
  Ply 4), its other rows unchanged. No file or column changed for existing leagues.
  - Leagues without them run as before: leagues.py was swapped atomically at 03:27:07, no league running (md5
    92aa7bbb...; the old one is leagues/leagues.py.before-ply4, md5 e7f1e7d6...), after two cycles of copies of four
    finished experiments gave the same files with the old and the new driver: flat-p3-b (`entry_ply` 3,
    self-games), flat-p2-b (`entry_ply` 2), lr-trees-p1 (`entry_ply` 1, trees) and leaf-sources-2 (the defaults,
    self-play lines). Each copy's cycles cap was raised by 2 (its second cycle the last) and its plateau's
    min_cycles put out of reach, the same for all three runs. The files that differed between the old and the new
    driver are exactly the ones that differ between two runs of the old driver (measured CPU in the cycle reports
    and `last_round_ply2_cpu_s`, the scan cache's records, self-play logs' rates); the engine snapshots are equal:
    818, 1358, 1228 and 3465 items. Harness: leagues-smoke/ident-ply4 (ident4.py, cmp.py, cmp.log).
  - selftest: 144 rule checks (28 new: the field, the training rule, needed prices and create at ply 4; six more
    `entry_ply` refusals; the warm start's records and train arguments and 15 refusals), and a warm-start test on
    hybrid10: an experiment at `entry_ply` 4 starts from another's version 1 with start_forget 8 (byte copies, the
    events, `--init-forget 8` in the first training only, every game and self-game at ply 4; a changed start model
    stops the first cycle). The full selftest: 6 min CPU.
  - The trainer: `train --start M --init-forget F` divides M's AdaGrad sums. Checked with chessai-hybrid4gfx (F-L's
    trainer) on flat-p3's F-L#1 v199 and 200 depth-1 games: no flag and F 1 give the same file; the saved sums are
    the start's divided by F plus the new squares; the weights move 0.48, 0.80 and 1.67 (L2) at F 1, 8 and 64.
  - A dry run of leagues/specs/ply4/flat-p3-w4.json in a scratch root, 2 cycles at nice 19: cycle 1's entries
    F-L#1.loc@0-4 to F-L#4.loc@0-4 (their version 0 byte copies of flat-p3's v199, md5 equal), 72 games and 24
    self-games, 62 CPU s (28 s wall); cycle 2 with 8 entries (v1 current, v0 champion), 336 games and 48 self-games,
    249 CPU s (106 s wall); the first training with `--init-forget 8`, the second without. Deleted after.

- 2026-11-11 01:24: `entry_ply` 3 (config.json above): the main entries play at ply 3 (`<line>@<v>-3`) under the
  rules of `entry_ply` 1 (no other ply-1 entries, no global option, a self-play line's default depth is the entry
  ply). create refuses a prices version without the lines' kinds at ply 3. leagues/prices.csv: P2 gained 11
  tournament rows at depth 3 (lr, probit and fm measured in the ply-3 arenas; the other kinds derived), its other
  rows unchanged (leagues/prices/P2.md, section Ply 3). No file or column changed; at `entry_ply` 3 the `ply2_*`
  columns count, measure and price the games with a model at ply 3.
  - Leagues without it run as before: leagues.py was swapped atomically at 01:24:04 (md5 e7f1e7d6...; the old one is
    leagues/leagues.py.before-ply3, md5 9fec34fd...) after two cycles of copies of four experiments gave the same
    files with the old and the new driver: probit-widen-p1c and lr-trees-p1 (`entry_ply` 1, self-games),
    probit-trees-p2 (`entry_ply` 2, no ply-1 entries, self-games) and leaf-sources-2 (the defaults: references,
    ply-1 entries, local and self-play lines; its second cycle its last). A running one was copied at a cycle's
    start, and the old driver finished that cycle before the comparison. The files that differed between the old
    and the new driver are the ones that also differ between two runs of the old driver: measured CPU in the cycle
    reports and `last_round_ply2_cpu_s`, the scan cache's hash of game_cpu.csv, self-play logs' rates, and one 30-s
    busy wait in a driver log. The engine snapshots (games, ratings, models, records without measured columns) are
    equal: 780, 712, 504 and 3465 items. Harness: leagues-smoke/ident-ply3 (ident3.py, cmp.py).
  - selftest: 116 rule checks (10 new: the field, the training rule, needed prices and create at ply 3, and four
    refusals), and a self-games test at ply 3 (straight and interrupted 8 times, 87 items equal; 40 self-games over
    4 cycles; the self-play lines at depth 3).
  - A dry run of flat-p3-cold's spec (models/from-scratch/theory-2/plans/) in a scratch root, 3 cycles at nice 19:
    only `F-L#k.loc@v-3` entries; the rating zero `F-L#1.loc@0-3`; 60, then 120 self-games a cycle; training on 240,
    then 960 games; every line crowned in cycles 2 and 3; the plateau's K 35 (4 lines), from cycle 200. 123 CPU s:
    about 50 CPU s and 20-29 s wall a cycle with 8 entries. Deleted after.
  - leagues/dashboard.py (the old one: leagues/dashboard.py.before-ply3): on a ply-3 page the champion-gap tooltips
    say current@3 and champion@3. Every existing page renders byte-identically (554 paths, time frozen); ply-1 pages
    keep their "current@2" in those tooltips (until 01:38, when they changed to current@1; BUILD.md, Entry ply 3).

- 2026-11-10 22:40: options `plateau.scaled`, `entry_ply` and `self_games` (config.json above); state.json
  `current.self_games` and the `self-games` phase; self_games.csv, self_games.log, selfpool/; totals.csv's
  `self_games*` columns (only with self_games); lines' `selfplay_depth_default` (only with `entry_ply`).
  - Absent keys keep every league as it was: leagues.py was swapped (atomically; the old one is
    leagues/leagues.py.before-selfgames) after one cycle of copies of three experiments (nb-settings-no-random3: old
    style, references and ply-1 entries; nb-settings-new: the batch-4 options; leaf-sources-2: local and self-play
    lines) gave the same files with the old and the new driver. The files that differed (logs' timings, the cycle
    report's measured CPU, scan-cache stats, `last_round_ply2_cpu_s`) are the ones that also differ between two runs
    of the old driver, and are equal once dates and measured CPU are masked; the engine snapshot (games, ratings,
    models, records without measured columns) is equal. Also the
    selftest: new rule checks, and a self-games test at ply 1 and at ply 2 (run straight and interrupted inside
    the new phase; the same games, ratings and decisions as without self-games in cycle 1; budgets higher by
    exactly the self-games' price).
  - The ply-1 specs: leagues/specs/ply1/ (small-field-x4-p1, growth-limits-p1; train_binary.py sets
    chessai-hybrid6 on a created experiment before cycle 1).
- 2026-11-10 17:39: experiment options for batch 4 (the user's new experiment defaults: no random players, no ply-1
  entries, run until the champions plateau). config.json `ply1_entries`, `plateau` and (made by create when
  `references` lacks `random`) `rating_zero`; state.json `last_crown_cycle`; the `done` event's plateau detail. See
  config.json above.
  - Absent keys keep every league as it was: leagues.py was swapped (atomically; the old one is
    leagues/leagues.py.before-batch4) only after one cycle of a copy of small-field gave the same files with the old
    and the new driver: 953 of 967 files byte-identical, and the other 14 (logs, the cycle report, CPU columns,
    scan-cache stats) are the same 14 that differ between two runs of the old driver, equal once dates and measured
    CPU are masked. Also the selftest (new checks, and an options test run straight and interrupted).
  - Batch 4's specs and its creation wrapper: leagues/specs/batch4/ (batch4.py, queue.sh). From 18:39 the queue
    also runs extras after the reruns (small-field-x4: replicates through the existing "replicates" model key); its
    state file experiments/batch4-queue.txt then lists 18 positions. No engine or format change.
- 2026-11-10 15:34: tournament binary chessai-hybrid8 (models/from-scratch/bin/chessai-hybrid8). No format change.
  - It is chessai-hybrid7 with exact faster depth-2 play for flat probit models: each reply gets an estimate from a
    ln Phi table with a proven bound, and only the replies that can be a move's smallest are valued in full. Every
    tour/ file, saved game, own-calibration game and report is byte-identical to chessai-hybrid7's
    (models/from-scratch/fastbin/hybrid8/summary.txt).
  - `tournament.binary` of cluster-error from cycle 614, split-growth-probit from 288, error-targets-probit from 203,
    error-targets from 215 and residual-leaves from 247. The old configs are in
    models/from-scratch/fastbin/hybrid8/config-before/. Train binaries and recipes are unchanged.
- 2026-11-10 13:14: error-targets and error-targets-probit were created through a wrapper. No format change.
  - `leagues.py create` checks every flag against the --help of the recipe's train binary, so it refuses flags that
    only a newer binary takes (chessai-target's `--error-target`, `--route-before-train`, `--split-bootstrap`).
  - models/from-scratch/target/league/create_et.py gives the recipe row chessai-target in memory and runs the
    engine's cmd_create; swap_et.py then set each line's `train_binary` before cycle 1 (old configs in
    config-before/ there). leagues.py and recipes.txt are unchanged. See leagues/BUILD.md, "How to create and run a
    league".
- 2026-11-10 ~13:00: arenas (leagues/arena.py, leagues/ARENA.md). An arena lives in `<root>/arenas/<name>/` with
  config.json, state.json, arena.log, calibration.csv, models/, owncal/ and tour/; its tour/ files have the
  experiments' formats. The first one is leagues/arenas/matched-depth (spec leagues/specs/arenas/matched-depth.json).
- 2026-11-10 10:56: driver scan cache in leagues/leagues.py (not committed; `git checkout leagues/leagues.py` undoes
  it). A league dir gets `scan-cache/` with summaries of tour/games.csv (games.csv.json, games.csv.pairs.json) and
  tour/game_cpu.csv (game_cpu.csv.json). They are trusted only when the file is unchanged (device, inode, size,
  mtime, ctime) or the SHA-256 of the cached prefix matches; anything else falls back to the original code. Results
  are byte-identical (models/from-scratch/fastbin/drivercache/summary.txt). In use from cluster-error's cycle 417
  and in every experiment created since.
- 2026-11-10 09:16: tournament binary chessai-hybrid7 (models/from-scratch/bin/chessai-hybrid7). No format change.
  - It is chessai-hybrid6 with a faster report step: elo_history.csv appended when the file is as the last run left
    it, the full fit beside the prefix fits, head-to-head counts only for pairs that met, a faster games.csv read.
    Every tour/ file is byte-identical to chessai-hybrid5's (models/from-scratch/fastbin/hybrid7/summary.txt).
  - cluster-error's `tournament.binary` from cycle 342; every experiment created since; the arenas' default.
- 2026-11-10 05:20: train binary chessai-hybrid6 in the three tree experiments. No format change.
  - models/from-scratch/bin/chessai-hybrid6 is chessai-hybrid5 with exact faster training: TAN pair counts in an
    integer batch, TAN training predictions reused, LR nodes stepping from the prediction they just made, an AVX2 LR
    step and a fast parser (models/from-scratch/fastbin/hybrid6/summary.txt). Models, states and train output are
    byte-identical to chessai-hybrid4gfx's; its play is chessai-hybrid5's.
  - single-split, single-split-probit and cluster-error use it as every line's `train_binary` from cycles 503, 512
    and 175. Other leagues keep their binaries (selfplay-depth: chessai-hybrid4gfx). Recipes are unchanged.
- 2026-11-10 00:53: tournament binary chessai-hybrid5 for leagues with trees or DAGs. No format change.
  - models/from-scratch/bin/chessai-hybrid5 is chessai-hybrid4gfx with exact fixed-point play for calibrated trees
    and DAGs at ply 1 and 2 (models/from-scratch/fastbin/hybrid5/summary.txt). Games, ratings, calibration games,
    models and states are byte-identical to chessai-hybrid4gfx's. Mixtures, FM and flat models play as before.
  - single-split, single-split-probit and cluster-error use it as `tournament.binary` from cycles 112, 123 and 40.
    Other leagues keep chessai-hybrid4gfx, which stays the default for new leagues. A spec names it with
    `"tournament": {"binary": "/home/iv/dev/chessai/models/from-scratch/bin/chessai-hybrid5"}` (the draft
    leagues/specs/batch3/5-dag-home.json does). Recipes are unchanged.
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
