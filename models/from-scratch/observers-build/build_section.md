
## Observers (2026-11-12 ~10:10-10:55)

The user, ~10:15: "use the games generated from uniform fields to train models of other types but not have them
participate in the field, so that we can later test their strengths in arenas. Maybe we can call these
'observers'"; "it would be nice to at least be able to define the experiment observers during experiment
definition" (no keep-all-games feature). An experiment spec's `"observers"` (FORMATS.md: config.json, state.json,
lines.csv, totals.csv, observers.csv, index.json, Changes) lists model entries, as `models` does. Each observer
trains every cycle, after the participants, on exactly the local set, from its previous version, with its recipe's
binary and arguments and its own seed offset. It never plays, isn't rated or calibrated, is never crowned and counts
nowhere participants are counted. Versions follow the participants' cycles (v = trained on cycles 1 to v).

Files (backups `*.before-observers` beside each; the patch scripts and tests in models/from-scratch/observers-build/):

| File | Before | After |
|---|---|---|
| leagues/leagues.py | f20818ff... | a5afab4c... (10:23), a5d6c96a... (10:36: the "train-observer" fault point and observers_test), a1801c9e... (10:41:12: the test's field check) |
| leagues/dashboard.py | 7bd7619e... | 48b33c2e... |
| leagues/treeview.py | 5459d854... | f54a0d66... (the report's field without the observers, listed apart) |
| leagues/FORMATS.md | a5ada731... | c5b792d3... |
| leagues/DASHBOARD.md | 07490b63... | f3c75261... |
| leagues/specs/batch6/treeonly-p1.json | (old copy: observers-build/treeonly-p1.json.before-observers) | 9140e086... (observers F-P x2, the 6.78 description) |
| leagues/specs/batch6/fponly-p1.json | new | 9be937ae... |

Every swap of leagues.py was atomic (a temp file beside it, compiled, then renamed): treeonly-p1 and fponly-p1
reload it each cycle. a5afab4c... is kept as observers-build/leagues.py.a5afab4c.

Driver (leagues.py): `observers_option` (make_config, last), `observer_lines`, `observers_in_cycle`,
`new_line_state(lc)` (status `observer`), phase_calibration makes an observer's version 0, phase_train's `one_obs`
after the participants (`current.observed`), apply_decisions versions them, phase_records writes observers.csv,
totals.csv's `observers, observer_train_cpu_s` and the report's Observers line; lines.csv leaves an observer's
champion empty; the index and `status` count observers apart. `needed_prices` skips them. MEASURED gains
`observer_train_cpu_s`; snapshot reads observers.csv.

Tests:
- No key, nothing changes: probit-f10-p1's own spec (created and run by f20818ff...) re-created in a scratch root on
  the new file, pinned to hybrid6 as the real run was, 3 cycles (observers-build/repro_test.py): its 4,608 games (72
  rounds), the cycles, history, rounds, self_games, events and totals rows of cycles 1-3 with their ratings, config.json
  (but created and root) and 24 model and state files of versions 0-3: byte for byte.
- With observers, the participants are untouched: scratch experiment o (T-P-Ne capped x2, ply 1, self-games) with and
  without observers F-P x2 and T-P-Ne capped x2, 3 cycles: the participants' games, ratings, models and records
  equal (lines.csv and state without the observer rows, totals.csv without the observer columns); the T-P-Ne
  observers #1 and #2 hold participants #1 and #2's bdt0-2.model and bdt2.state byte for byte; no observer in
  players.txt, games.csv, self_games.csv, entries.csv, history.csv, cycles.csv or owncal/.
- The selftest's observers_test (run by `selftest` without --quick): the same, plus a run interrupted 8 times
  (cycle 1: tournament, train-observer; cycle 2: tournament-partial, self-games-partial, train-one, train-observer,
  records-half; cycle 3: records-half) equal to the straight run, reports too; observers.csv against cycles.csv;
  totals.csv's field and rounds the participants'; create refuses observers in the global league, without the local
  option, with a warm start, given twice and as an empty list. SELFTEST_RESULT
- The runs, every cycle so far: each observer training on exactly the participants' set (treeonly-p1 226 of 226
  through cycle 113, fponly-p1 282 of 282 through cycle 141), versions on by one, no observer name in players.txt,
  games.csv, self_games.csv, entries.csv, history.csv, cycles.csv or events.csv.
- Arenas: a scratch arena (observers-build/arena/obs-accept; v60 of an F-P observer of treeonly-p1, a T-P-Ne
  observer of fponly-p1 and a participant of each, ply 1, 2 round-robins, no shared own-games cache) created, played
  and checked: "calibrations: 2 identical to their experiment's; 2 without a record to compare" (the observers:
  their experiment never calibrates them). make_split_read.py's tags take observers like any line (`^F-P` in
  treeonly-p1 gives F-P#1.obs and F-P#2.obs as tag60.L1@60, tag60.L2@60); a tag that matches a participant and an
  observer of the same replicate stops on its duplicate-name assert, so give each role its own tag (`\.loc`,
  `\.obs`).
- Dashboard: pages without observers render as before (probit-f10-p1, the global league and the arenas: the same
  HTML body, filter factors, colour-by, items but the new role value, and charts); with observers every panel
  renders; served on a test port, then the dashboard on port 8766 was restarted at 10:48:45 (pid 3209581; the old
  pid 2476120, started 11-11 15:40). treeview.py on an observer tree (fponly-p1) and a participant tree (treeonly-p1)
  says which lines are the field and which observe.

The runs (spec analyst 6.78; launched by repeat.sh each, after train_binary.py pinned chessai-hybrid6 on every
line, observers included):
- treeonly-p1: created 10:12:12 without observers; F-P x2 added before cycle 1 at 10:25:07 by
  observers-build/add_observers.py (refuses a league past cycle 0; checks the spec's participants and other keys
  equal the league's; config/state/lines copies before it in observers-build/treeonly-p1-before/); launched 10:25:17
  (repeat pid 3189520).
- fponly-p1: created with observers T-P-Ne capped x2 at 10:25:32; launched 10:25:54 (repeat pid 3190145).
- Observers get no self-games, and nothing tops that up: the participants' self-games are 25% of cycle 1's set (192
  of 768 games, 4 entries) and 12.5% from cycle 2 (224 of 1,792, 8 entries), in games and in positions alike; a
  line's own are about 3%.
- Cost so far: treeonly-p1's F-P observers 46 CPU-s through cycle 113 (the trees' training 515); fponly-p1's tree
  observers 402 CPU-s through cycle 141 (the F-P lines' training 129).

Not changed: leagues/library/build_index.py (the library fork's) reads config.json's lines, so its `field_of`
counts observers as field members (treeonly-p1 would read "T-P-Ne... x4, F-P x2", mixed); skipping lines with
`"observer": true` there fixes it. Its observer rows have line_status `observer` and no cycles.csv training count
(theirs is in observers.csv).
