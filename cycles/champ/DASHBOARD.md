# The dashboard (the champion rules run)

`dashboard.py` serves a live page of this run at http://127.0.0.1:8765, reachable from this machine only. The page
refreshes every 30 s. The first run's page is kept read-only at http://127.0.0.1:8765/first/ (see "The first run's
archive").

The dashboard only reads. It takes no locks, writes nothing but its own `dashboard.log`, parses a file only when the
file changes, and runs at nice 19. It went live on 2026-11-07 at 22:34, during cycle 1, the seeding cycle.

## Starting, stopping, rolling back

- **Start**, from `cycles/champ/`:

      setsid nohup nice -n 19 python3 dashboard.py >> dashboard.log 2>&1 < /dev/null &

  - Each start writes a line to `dashboard.log` with its pid, its config and where the archive comes from.
  - Options:
    - `--port` (8765);
    - `--config` (`./config.json`);
    - `--first` and `--first-config`: the first run's dashboard and config (`../dashboard.py`, `../config.json`);
    - `--once`: renders the page to stdout and exits.
- **Stop:** `kill PID`, with the pid from the last start line in `dashboard.log`. Restarting only the dashboard
  doesn't touch the loop or the league.
- **Rollback** to the first run's own page, which shows only the first run:

      kill PID; cd /home/iv/dev/chessai/cycles && (setsid nohup nice -n 19 python3 dashboard.py >> dashboard.log 2>&1 < /dev/null &)

  The old process, its command and the switch are recorded in
  `models/from-scratch/champion-sim/stage2/dashboard-rollback.txt`. The same `dashboard.py` is staged there.

## Costs (measured 2026-11-07, cycles 1-2)

- **A page render:**
  - The page is about 0.3-0.8 MB and takes 0.05-0.3 s of CPU, once per open tab every 30 s.
  - The head-to-head table is fetched separately, only when `games.csv` changes. It is 0.4-1.3 MB and takes 0.03-1.2 s.
- **Memory:** about 55 MB for this run at cycle 2.
  - Opening the archive adds about 390 MB, kept until the dashboard restarts.
  - The first run's dashboard held 1.38 GB.
- **Disk sizes:** a background thread measures `cycles/runs` and `cycles/champ/owncal` every 10 minutes.
- **The archive's first visit:** about 7 CPU-s; 9 s of wall time, up to about 45 s while a tournament runs (the
  dashboard is at nice 19). After that it is served from a cache.
- **Growth:**
  - `games.csv` grows by about 130,000-170,000 games a cycle. The dashboard reads it incrementally.
  - Standings and head to head grow with the field, about 160-185 entries.
  - The page has no step that grows like the first run's covariance did.

## The page

The header gives the time of the reading and the calibration mode:

- "calibration: own games": each version is calibrated on its own play, in `cycles/champ/owncal`.
- "calibration: held-out sets".

The mode is read from `state.json`, where `init` sets it. If `config.json` disagrees, the header says so, because the
driver stops on a mismatch.

Each panel has an anchor:

- `#now`, `#chart`, `#standings`, `#lineages`, `#global`, `#seeding`;
- `#compare`, `#head-to-head`, `#elo`, `#gates`, `#history`.

### The bar at the top

- **Run:** switches between this run and the first run's archive.
- **Filters:** a row of chips per factor. The factors are read from each lineage's `lineages.txt` flags, with the
  binary's defaults:
  - architecture: F, T, D, M;
  - leaf evaluator: L, N, P, TAN, FM;
  - router;
  - split management;
  - self-play depth;
  - training labels;
  - move selection.
- **How the filters combine:** values of one factor add up (or), and factors narrow down (and). Each chip counts the
  lineages it would show. League play depth (ply 1, ply 2) picks entries, not lineages. The filters apply to every
  panel. The reference players always stay.
- **Classifications new in this run:**
  - The MoE kinds are M:
    - `moe`: leaf L, router "gate (soft)";
    - `moenb`: leaf N, router "gate (soft)";
    - `nbgate`: leaf L, router "gate (NB)";
    - `nbmix`: leaf N, router "gate (NB)".
  - `--router tan` is router Te.
  - The variant names `--nbmix-components` ("4 components"), `--tan-m`, `--leaf-share` and `--moe-experts` ("8
    experts").
- **Colour and group by:** any factor. Every lineage's marks take its value's colour.
- **Saved settings:** the filters, colour, highlight, chart ply, sorts, status filter and head-to-head scope are kept
  in the browser (localStorage). The archive shares them.

### Now

- **Tiles:**
  - Loop: running, with the loop's pid. Only a loop or driver whose working directory is `cycles/champ/` counts, so
    the first run's never does.
  - STOP file.
  - The cycle: its kind (the seeding cycle, a normal cycle or a global tournament), its phase and its progress.
  - Elapsed: from `loop.log`.
  - A rough end time.
  - The next global tournament: every 25th cycle, `global_every`.
- **Self-play:**
  - Each job's CPU against its budget. A job's budget is the cycle's even share plus its carry.
  - Running and finished jobs, then those still waiting.
- **Tournament:**
  - Games so far of the cycle's target, from `tournament.log`'s "tournament k/N" lines.
  - The log's last line, with ratings unsigned.
  - With `--reports-last`, the tool writes its tables once, after the cycle's last round-robin. Until then, Standings
    and the chart keep the previous cycle's fit.
- **A global cycle:** the rejoining champions, each with its rating before the tournament.
- **The own phase:** the own-games steps and their gates so far.
- **Notes:** the driver's notes for the cycle, e.g. own calibration's (versions made, reused and deleted, and its disk).

### Elo by cycle

- **Lines:** one per lineage the filters show, at 2-ply, or 1-ply on its toggle. Each cycle the lineage played gives a
  point, as measured then:
  - a finished cycle at 2-ply: `cycles.csv` at its decision;
  - a finished cycle at 1-ply: the league's `elo_history.csv` at the cycle's last round-robin
    (`state.json`'s `round_robin_ends`);
  - the cycle under way: today's fit, once its round-robins have ended.
- **Marks:**
  - A ring marks a champion, at the cycle it became the champion and its Elo then. "Champions" hides the rings.
  - A frozen lineage's line is dotted from its freeze.
  - × ends a retired lineage's line.
  - A retired champion that rejoins a global tournament gets a point there.
- **Clicking:** a line, its name, or the lineage anywhere on the page highlights it (again to un-highlight).
- **Hovering** a point gives its version, Elo ±, cycle and decision.
- **Under the chart:** a key grouped by the colour factor, with each lineage's latest Elo, and a table view.

### Standings: the field

- **Rows:** one table ranked by Elo across every lineage, with no sections. It holds:
  - the players of the current fit's tournament;
  - the retired champions (muted, rank "–");
  - the entries that left at that fit's decision, e.g. the loser of the seeding or a dethroned champion (muted, "left
    the field").
- **During a tournament:** the fit is still the previous cycle's. The new entries are listed under the table as not
  rated yet.
- **Before the first ratings:** the field is listed instead.
- **Columns:**
  - Rank, entry, ply, lineage, version.
  - **Positions** (added 2026-11-08): how many positions the version has trained on since its lineage began, every
    position once, e.g. "79.8M".
    - It sums the version's chain of steps from v0: each self-play round's positions, from the round's line in the
      lineage's log (`cycles/runs/<lineage>.log`: "round N: +W −L =D, P plies"), and each kept own-games step's,
      from `cycles.csv` (this run's and the first run's).
    - A version copied from an older one (the seeding's old peak, a revival: a hard link) counts that one's chain.
      For example, F-L-c v3147 is a copy of v2860, so v2861–v3146's training is not in it.
    - It is exact: it equals the trees' training-state step counters, 4,892 states of 40 tree lineages, all of
      them. The model files' own counts are something else: a tree's root restarts its count when it splits, and
      with `--error winner` it counts decisive positions only.
    - Hover a count for its parts: self-play positions, games and rounds; own-step positions, league games and steps.
    - Click the heading to sort by it, again to reverse, Elo to go back.
    - "≥" would mark a chain with a step found in neither the log nor `cycles.csv`; there are none.
    - /first/ shows the column too: it is added to that run's page, and its file stays as it is.
  - Games: the entry's league games.
  - **Status:**
    - active;
    - revived cN;
    - finished-frozen cN;
    - frozen at global cN;
    - finished-retired cN;
    - out again cN;
    - left the field;
    - reference.

    Hover a status for the driver's decision and note.
  - **Change:** over the fit's cycle, the row's Elo minus the lineage's rating at that ply at the end of the previous
    cycle, e.g. "▲ +12" or "▼ −9". It shows "new" in a lineage's first cycle at that ply. With the gate results, it is
    the only "+" on the page. Absolute ratings and gaps carry none.
  - Elo and ±95%.
    - **A champion's peak** (added 2026-11-08): a champion row's Elo is "current (peak)", e.g. "1812 (1824)".
    - The peak is the highest Elo that entry has had in any cycle's fit of this run, read from the league's Elo
      history at each cycle's last round-robin. It counts the fits from before the version became the champion, when
      it played as the current version.
    - It is always shown, even when it equals the current Elo. Hover for the cycle of the peak. Ratings drift
      upward across cycles, so the peak is often recent.
    - Sorting by Elo uses the current value. Other rows, and /first/, are unchanged.
  - **z vs champion:**
    - The latest champion test, from the driver's note to 2 decimals: the current version's 2-ply Elo minus the
      champion's, over the standard error of the difference.
    - 2 or more (`champion_se`) makes a new champion, shown in bold green.
    - The champion's own entry says "champion", or "new champion" when it became the champion in the fit's cycle.
    - Clicking the heading sorts by z, again to reverse.
  - **Patience:** self-play CPU since the lineage's last new champion, of 32 CPU-min (`patience_cpu_s` 1920).
  - An interval bar, then each factor's value. The factor the page colours by is in bold.
- **Filter by status:** a checkbox per status. None ticked shows every row.

### Lineages: champions, patience, status

- **Rows:** one per lineage, with these columns:
  - Status.
  - Champion: its version and the cycle it became the champion. Hover for the test.
  - Champion Elo: its 2-ply entry in today's fit.
  - Elo now: the newest version's 2-ply entry.
  - z vs champion, with the test's cycle.
  - Patience used.
  - What is left, with about how many cycles that is at the current share.
  - The seeding outcome.
  - The seven factor letters.
  - The variant: the hyperparameters that differ from the field's usual recipe.
- **Sorting:** click a heading to sort.
- **Milestones:** under the table, the milestones held. Each is a multiple of 100 held by a reference player or
  claimed by a frozen lineage.

### Global tournaments

- **The rule:** the next global cycle, the rule, and the revival test:

  z = (S − E) / sqrt(V + (V · ln10 / 400 · ci0 / 1.96)²)

  - S is its points, and E those expected from its rating before.
  - V = Σ p(1 − p).
  - Revival needs z ≥ `revive_se`, which is 2.
- **Rejoiners:** the champions rejoining now, then those that would rejoin at the next global cycle.
- **Past global cycles:** for each one and each champion:
  - games, score, expected and z;
  - the outcome: revived, frozen at global or out again;
  - its Elo after, and the driver's note.

### Seeding (cycle 1)

- **Per lineage:**
  - the latest version and the old peak, each with its 2-ply Elo at the seeding and its first-run rating;
  - the first champion;
  - what it trains from: a copy of the old peak when that won;
  - both entries in today's fit.
- **Counts:** a summary counts each outcome. Cycle 1 had 31 lineages keep their latest version, 23 take their old
  peak, and 6 new lineages wait for a first version.
- **Before cycle 1 decides:** the panel shows the plan.

### Factor comparison

- **Rows:** pick a factor. Each row is a set of lineages that agree on every other factor.
- **Cells:** each value's lineages, by champion or by Elo now, at 2-ply or 1-ply.
- **The difference column:** the pair with the fewest other differences, with ± from the two ±s combined as if
  independent.

### Head to head

- **Cells:** "+wins −losses =draws" for every pair, shaded by score. Hover a cell for the games, the score and what the
  fit expects.
- **Scopes:**
  - the current field's entries;
  - one cycle's tournament, marked partial while it runs;
  - all cycles by lineage@ply.
- **Clicking** a row gives that player's record per opponent and cycle.

### Elo over cycles, by archetype

Small multiples of the per-cycle points, one per archetype group. A dashed line marks a depth-2 self-play lineage.
Clicking a series highlights the lineage in the chart at the top.

### Gate history

- **Cells:** each own-games step's gate, by lineage and cycle:
  - the score, and the Elo ± against the model from before the step;
  - ✓ kept or ↺ reverted;
  - "frozen", "retired" (finished-retired), "out again" or "revived" where there was no step.
- **Pooled:** a row per cycle, with a chart. The lineages are grouped by the colour factor, retired ones last.
- **The last columns:** kept steps of gates, and the sum of the kept steps' Elo. That sum is biased up, since each
  step was kept for scoring at least 50%.

### Cycle history

- **A row per cycle:**
  - kind; started (from `loop.log`); took; self-play (the job queue's makespan); tournament; own + gates;
  - field, games, trained;
  - self-play CPU against the budget;
  - new champions;
  - finished: frozen · retired;
  - global: revived · frozen · out;
  - after: active · frozen · retired;
  - gates kept.
- **A chart:** the CPU per lineage against each one's budget.
- **Disk:**
  - `cycles/runs` is shared with the first run.
  - Free disk.
  - `cycles/champ/owncal`, with own calibration: about 1.1 GB after cycle 1.
- **Timing caveat:** `loop.log` stamps cycle starts in minutes, so "took" and "own + gates" can be up to about a
  minute off.

## What it reads

- `cycles/champ/`:
  - `config.json`, `state.json`, `cycles.csv`, `totals.csv`, `lineages.txt`;
  - `loop.log`, `STOP`, `cycle-NNN.md`.
- `selfplay/league-champ/`: `players.txt`, `retired.txt`, `ratings.csv`, `elo_history.csv`, `games.csv`, `models.csv`,
  `tournament.log`.
- The disk sizes of `cycles/runs` and `cycles/champ/owncal`.
- `/proc`, for the loop, the driver and the self-play jobs.

The record formats are in `README.md` ("Records").

## The first run's archive (/first/)

- **What it is:** the first run's own page, rendered by its unchanged `cycles/dashboard.py` with `cycles/config.json`.
  - It reads the first run's files, which are no longer written.
  - The first run's own rules apply there: qualification, dormancy, freeze/retire.
  - It adds a run selector and a note that it is the archive.
- **How it is served:** inside this dashboard's process, imported when first visited.
  - The page is cached until one of the first run's files changes.
  - It shares this page's saved settings and its disk measurements.
  - The import writes no bytecode beside the first run's files.
- **The independence caveat:** the archive doesn't compute the fit's covariance.
  - On the first run's league of about 2,900 players, that pure-Python Cholesky takes over 5 CPU-min. Meanwhile it
    holds the process's interpreter lock and stalls this run's page.
  - So the archive's ply-gain ±, and the ± of its wake gaps, combine the two ratings' ±s as if they were independent.
    That ignores the correlation between the two entries' ratings, so a ± can differ from what the live first-run
    page showed. Each such hover says so.
  - The rest of the archive is as the first run's page showed it.
