# The leagues' dashboard

`leagues/dashboard.py` is a read-only web page over the new league system's files, as `FORMATS.md` defines them.
It uses only the standard library, reads files and `/proc`, and writes nothing. Pages refresh in place every 30 s.

## Running it

```
cd /home/iv/dev/chessai/leagues
setsid nohup nice -n 19 python3 dashboard.py >> dashboard.log 2>&1 < /dev/null &
```

- `--root DIR`: the root that holds `index.json`, `global/` and `experiments/`. The default is this file's directory
  (the real root). For the smoke data, use `--root /home/iv/dev/chessai/leagues-smoke`.
- `--port N`: the default is 8766. It listens on 127.0.0.1 only.
- `--old-dashboard URL`: the champion run's dashboard, linked for the champion run and the first run. The default
  is `http://127.0.0.1:8765/`; `""` drops the links.
- `--once PATH`: renders one page (`/`, `/global/`, `/x/NAME/`) to stdout and exits. Use it for checks.

The server lowers itself to nice 19. Stop it by its pid; the start line in `dashboard.log` prints it.

## Pages

The sticky bar at the top holds the navigation and the filters.

- **Navigation**:
  - links: Leagues (Home), Global, and the six most recent experiments, running ones first. Each link has a status
    dot: green running, amber waiting, blue done, red failed, grey otherwise.
  - with more than three experiments, a list of all of them.
  - links to the champion run and the first run on the champion run's own dashboard (port 8765).
- **Filters**: chips per factor, colour-by, the highlighted line, and how many lines show.
  - They drive every panel on the page: table rows, chart series, legends and the grid's cells.
  - Factors with only one value on the page are left out.
  - Experiments add a factor per overridden flag (a sweep's `lr-rate`, say), the replicate and the base model.
    Colour-by starts on the swept flag when there is one, else on the option (several), else on the model.
  - Each kind of page (home, global, experiment) keeps its own filter state in the browser.
  - Click a row, a cell, a legend entry or a chart line to highlight that line everywhere on the page. Click it
    again to clear it.
- **Tables** sort when you click a heading, which cycles through descending, ascending and off. Sort order survives
  the refresh.
- **Arrows**: an Elo change (since the previous cycle, since an anchor locked) or a gap to the champion has a green
  ▲ when up and a red ▼ when down, none when it rounds to 0. The numbers are unchanged. Gaps between options get
  none, since neither direction is better.

### Home `/`

- **Leagues**: one row per league, each name a link. Columns:
  - kind;
  - status: checked against `/proc`, so a `running` or `waiting` league whose driver is gone shows as `stopped`,
    with the reason in the tooltip; a STOP file is noted there too;
  - now: the running cycle's phase, and during the tournament its rounds played of those planned;
  - cycle n of N, with a meter;
  - lines, as models × options;
  - started: cycle 1's start;
  - finished: done experiments' finish time; for running ones, a projected time;
  - prices: the prices version the league runs on (a self-play-only experiment's budget in the tooltip);
  - CPU spent, measured, broken down in the tooltip (with the tournament games' price beside it);
  - the leading line: the line whose current version rates highest at ply 2.
- **Recent events**: the latest 40 events across every league.
- **Earlier runs**: the links below.
- Filters: kind, status, model, option.

Leagues on disk that `index.json` doesn't list are shown too, with "not in index.json" in the status tooltip.

### A league: `/global/` and `/x/<name>/`

The heading names the models (by label), the options, the cycle, the prices version and, for a self-play-only
experiment, its fixed self-play budget.

**Priced and measured CPU.** Since FORMATS.md's 16:40 entry every budget is priced CPU: a game's price comes from
its plies and the constants of `leagues/prices.csv`, frozen in the league's config, so a league replays exactly.
Wherever a budget or a data amount is shown (training data, data targets and budgets, copied global rounds, the
batch, rounds), the priced figure leads and says "priced"; the measured CPU stands beside it or in the tooltip, for
information. What a league spent (the CPU tile, the Cycles chart and its CPU columns) is measured CPU. A league made
before prices shows measured CPU throughout.

Panels, top to bottom. The global league has no "By model and option", "Training data" or "Global rounds
copied" panels; experiments have no "Anchors" or "Latest batch" panels.

- **Now**: tiles for:
  - status and phase, with the driver's pid and uptime;
  - cycles done (n of N, with a meter), and the running cycle's start and round;
  - lines;
  - the field: entries, anchors, references;
  - CPU spent (measured);
  - experiments: the data budget, priced: a self-play-only experiment's fixed budget per line per cycle, else
    the last training cycle's target (the price of its qualifying games);
  - the last cycle's wall time;
  - experiments: time left;
  - experiments with global data: global rounds copied (priced), time waited, and the global carry: how far the
    global lines' data leads or trails the local lines' (`state.json` `global_carry_s`).

  While a cycle runs, a phase strip shows the cycle's phases in the order the driver runs them:
  - global: calibration, tournament, batch, decide, train, records;
  - experiments: copy (only with the global option; it reads "copy: waiting for global rounds" while the
    experiment waits), calibration, tournament, decide, train, self-play (only with a self-play line), records.

  Earlier phases have a tick and the current one is highlighted; the tournament shows its rounds played of those
  planned. In an experiment's last planned cycle, copy, train and self-play are struck through: that cycle copies
  and trains nothing. During the tournament, a note below the strip gives the cycle's rounds, games and CPU so far
  (`rounds.csv`). The last 14 lines of `driver.log` sit in a fold.
- **By model and option** (experiments): a grid of models (rows) by options (columns). Each cell shows:
  - the line's current version at ply 2, with its ±;
  - the change since the previous cycle;
  - how many new champions it has had;
  - a "best" mark on the model's best option.

  Cells the filters exclude are dimmed rather than hidden, so the columns stay aligned.

  Models show by label (`F-L lr-rate=0.025 #2` for the id `F-L+lr-rate=0.025#2`), and lines as the label plus
  the option suffix.

  With flag overrides or replicates, **By setting** groups the lines by setting (the base model and its overrides)
  and option: the replicates' mean ply-2 rating in the last cycle, their spread, the mean over the last 5 cycles,
  and each replicate's rating.

  Below the grid, **Gaps between options**: for each model and each pair of options:
  - the last cycle's ply-2 gap in words, with the two ±s combined as if independent;
  - the mean gap over the last 5 cycles;
  - how many cycles each option was ahead.
- **Elo by cycle**: each line's current version per cycle, from `cycles.csv`.
  - Toggles: ply 2 or ply 1; the champions (dotted, the champion before each decision); the references; the rings
    where a version was crowned. The rings are on by default up to 12 lines; the browser remembers your choices.
  - In experiments, lines that share a colour get dashes from the first of replicate, model and option that isn't
    the colour: a sweep coloured by its flag dashes its replicates. The legend shows each line's colour and dash.
- **Standings**: every entry in the last completed cycle's field (`history.csv`), by Elo. Columns:
  - line;
  - version name (`G.x.y`, `G.x.yF`; experiments `E.S.x.y`, `E.L.x.y`, `E.G.x.y` by option), with the entry id
    in the tooltip;
  - ply and role;
  - Elo and ±;
  - change since the previous cycle. For a current entry, it is measured against the line's previous version at
    the same ply. For any other entry, it is the entry's own move;
  - games;
  - for current ply-2 entries, the gap to the champion in words, with z.
- **Lines**: one row per line. Columns:
  - status (retired, with its cycle);
  - the version that plays next (— once retired);
  - the last played version with its ply-2 Elo;
  - the champion with its Elo;
  - the gap and z;
  - the last decision;
  - global: patience n of 20;
  - new champions crowned;
  - the cycle it entered;
  - experiments: the priced data of its last training cycle (measured in the tooltip);
  - the recipe: training arguments. The tooltip has the recipe, the binaries, the held-out set, a self-play line's
    carry and mean round price (priced) and games per round, and a variant's model id and replicate.
- **Anchors** (global): one row per 100-Elo interval, highest first, including empty intervals up to the best
  current ply-1 entry. Columns:
  - the anchor's name and line;
  - when it locked and at what Elo;
  - its Elo now and how far it has moved;
  - a mini chart of its rating against its interval and margins;
  - its streak outside the interval;
  - locked or available.
- **Events**: `events.csv`, newest first (the latest 250), with counts per event type. Covers champions, retirements,
  anchors, copies, waits and done; new champions are in green.
- **Training data, priced CPU** (experiments):
  - a chart of each line's training data per cycle (`cycles.csv` `data_price_s`) against the cycle's target
    (dashed, `totals.csv` `data_price_target_s`), or against the fixed budget in a self-play-only experiment
    (`config.json` `selfplay_budget_s`);
  - the average per option, relative to the budget;
  - a table of cycles × lines, each cell a meter with its % of the target. A self-play line is measured against its
    own budget (`selfplay_budget_s`: the target or fixed budget plus its carry). Its tooltip has the priced
    generation, the rounds, games and plies, the carry after the cycle, and the measured generation and job CPU.
    The last cycle of an experiment trains nothing and shows "—".
- **Global rounds copied** (experiments with the global option):
  - where the rounds come from (linked to the global league when it is this root's);
  - rounds used, from `state.json`;
  - the global carry (priced), and per cycle the copied qualifying data against the local data, both priced;
  - waits, from the wait-start and wait-end events, including an open wait;
  - `copied.csv`, newest first: priced (the experiment's prices) and measured CPU of each round.
- **Cycles**: a stacked bar chart of measured CPU per cycle (tournament games, rating fit, training, self-play,
  calibration), and `totals.csv` newest first (the latest 120 cycles). Columns:
  - wall time, field, rounds, games;
  - tournament CPU (the games' own, measured), the games' price, and measured CPU per round;
  - rating fit: the tournament process's own CPU beyond its games (`tournament_proc_cpu_s` minus
    `tournament_cpu_s`), the column shown once a cycle has recorded it;
  - training, self-play and calibration CPU, and the total (the rating fit included);
  - new champions;
  - global: retired lines and anchors;
  - experiments: the data target (priced; not in self-play-only experiments, whose budget is fixed), global
    rounds (priced and measured in the tooltip) and time waited.
- **Rounds**: a bar chart of priced CPU per round over the latest 250 rounds, split into qualifying games and the
  rest; the measured CPU is in the tooltips. Below it, the last three cycles' rounds from `rounds.csv`: priced and
  qualifying priced, measured and qualifying measured, and measured CPU per game.
- **Latest batch** (global): `batch/batch.json`'s cycle, when it was made, its prices version and its rounds,
  priced (the global league's prices) and measured. Each round lists the experiments whose `state.json` used it.
- **Reports and log**: the latest `cycle-NNN.md` in a fold, every report as text (`report/NNN`), the last 256 KB of
  `driver.log` (`log`), and `config.json` and `state.json` (`file/...`).

### Earlier runs

The champion run and the first run are only linked; this server never loads their dashboard:

- the champion run: `http://127.0.0.1:8765/`;
- the first run: `http://127.0.0.1:8765/first/`.

Home says when nothing listens on that port (read from `/proc/net/tcp`; it never connects).

## What it reads

The files of `FORMATS.md`:

- `index.json`;
- per league: `config.json`, `state.json`, `lines.csv`, `cycles.csv`, `totals.csv`, `rounds.csv`, `events.csv`,
  `history.csv`, `anchors.csv`, `copied.csv`, `batch/batch.json`, `cycle-NNN.md`, `driver.log` and `STOP`;
- `/proc/<pid>` for the drivers;
- `repeat.log` when it exists (linked from Reports and log as `repeat-log`).

Reading is cached:

- Each file is re-read only when its mtime, size or inode changes.
- A CSV's trailing partial line is ignored, since the file may be caught mid-append.
- A file that fails to parse keeps its last good reading. The page names it.

It does not read `tour/`, `runs/`, `owncal/`, `pool/` or the batch's games.

## Format assumptions

These are things `FORMATS.md` leaves open, or that the dashboard takes as given:

- **Standings** come from `history.csv`, so they are always the last completed cycle's. While a cycle runs, they
  are not updated round by round: `tour/ratings.csv` is not read.
- **Line definitions**:
  - `config.json`'s `lines` give the order and the argument lists;
  - `lines.csv` gives the latest status and names. When its `train_args`/`selfplay_args` cells are the only
    source, they are read as a JSON list or as shell words;
  - `state.json`'s `lines` fill in `crowned` and `patience`.
- **Names and labels**: model ids and line names come from `config.json` and `lines.csv`; labels from their
  `label` (else `index.json`'s `labels`). A line shows as its model's label plus the line's option suffix. Version
  names come from the CSVs; only a line with no `version_name` yet gets one made from `state.json`
  (`E.<S|L|G>.<major>.<v>` in experiments, `G.<major>.<v>` in the global league).
- **Overrides**: `config.json`'s `train_overrides` and `selfplay_overrides` (lists of [flag, value]) give the
  sweep factors, one per flag, named without dashes; a line without the flag shows "recipe's". `true` shows as
  "on", `null` as "off".
- **Prices**: a league is priced when its `totals.csv` or `rounds.csv` have `price_s`. A self-play-only
  experiment is one with `selfplay_budget_s` in its config; its lines' target is that fixed budget.
- **Factors**, the classification the champion run's dashboard uses:
  - a line's model comes from its `kind` taken as `--model`, plus `train_args`. `bdt` with `--dag-leaves` and
    `bdd` are DAGs (D);
  - its labels come from `--label-depth`/`--label-result`;
  - a self-play line's depth and move selection come from `selfplay_args`;
  - where the flags don't settle it, the archetype's name is used; failing that, "unclassified".
- **Status** comes from `state.json` (`index.json` when `state.json` has none; the driver writes both, `state.json`
  more often). `running` or `waiting` with no live pid shows as `stopped`. A league on disk that `index.json`
  doesn't list gets its status from its `state.json`, or from whether its pid is alive.
- **Rounds played** in the running cycle: the length of `state.json`'s `current.rounds`, else the rows of
  `rounds.csv` for that cycle.
- **CPU**: totals count the games' own CPU (`tournament_cpu_s`), the rating fit and tables
  (`tournament_proc_cpu_s` minus `tournament_cpu_s`, when the column is there), training, self-play and
  calibration, plus the running cycle's rounds from `rounds.csv`.
- **The phase strip** follows the driver's phase lists (`PHASES_GLOBAL`, `PHASES_EXPERIMENT` in `leagues.py`), with
  `waiting` shown inside copy. "An experiment's last cycle" is a cycle number at or above `config.json`'s `cycles`.
- **Anchors**: intervals are `step` wide from `anchors.lowest` (100 if missing) up, with the margin and
  `out_cycles` from `config.json`'s `anchors`. `available` counts as true for 1/true/True.
- **The batch's "used by"** matches experiments whose `global_source` is this global league's directory, comparing
  their `used_global_rounds` pairs `[global cycle, t]` with `batch.json`'s cycle and each round's `t`.
- **Time left and projected finish**: the remaining cycles times the median wall time of the last five cycles.
- **Events**' values: z for new-champion; the interval's lower bound for anchor-locked (shown as the interval).
  Other values are shown as written.

## Development

It was developed against a synthetic root made from FORMATS.md, then against the engine's smoke root. Each page was
rendered with `--once` and fetched with curl on port 8766. A headless Chromium run (a temporary profile, a
same-origin proxy) drove the filters, highlighting, sorting, chart toggles and the 30 s refresh.
