
## Cold games (2026-11-12 ~11:00-SWAPTIME)

The user's decision at 10:54, on the in-run rating bias (T 0.1 in-run ratings under-rate trees by 30-150): "let's
see how the addition of the T 0.02 games affects the model strengths (we can run some uniform ply1 tests) if it
doesn't affect the strength negatively let's add this T 0.02 games into training if not, fine let's run a small
number of additional games to compute the actual experiment ratings". An experiment spec's `"cold_games":
{"temperature": T, "share": S, "train": true|false}` (FORMATS.md: config.json, state.json, totals.csv, cold_rounds.csv
and cold/, index.json, Changes) adds a cold-games phase after the tournament: the field plays R = round(S x
games_per_cycle / (N (N - 1))) extra double round-robins (halves up, 1 to 48) at temperature T in cold/, seed the
league's plus 2000003, the tournament's flags and policy otherwise. The in-run ratings are the cold games' fit alone
(league_ratings and the champion test's SE read cold/: rating_tour), so every decision (champion test, crownings,
plateau rule, the field) runs on them; tour/ keeps the round-robins and their fit, which decide nothing. With train
true the cycle's cold games join the shared local set after the round-robin games and self-games, so every line and
every observer trains on them; with false they rate only. Not built with self-play or global lines, replay, frozen
references, or without games_per_cycle (create and run refuse them).

Files (backups `*.before-coldgames` beside each; the patch scripts and tests in models/from-scratch/coldgames-build/):

| File | Before | After |
|---|---|---|
FILETABLE

Driver (leagues.py): COLD_* constants and COLD_TOTALS_FIELDS; `cold_games_option` (make_config, after observers),
`cold_games_of`, `cold_rounds_for`, `cold_flags`, `rating_tour`; `phase_cold` (phase `cold-games`, after the
tournament; fault point `cold-games-partial`); `write_players` and `clean_partial` take a directory; `league_ratings`
and phase_decide's `contrast_ses` read `rating_tour`; phase_train appends the cold games to the local set with train
(links `1000000 + t`, after the rest; the set's data CPU and price include them; `current.cold.in_local`);
phase_records writes cold_rounds.csv and the cold totals and removes coldpool/; the report's Cold games line and CPU
lines; the index's `cold_games`; `run` checks the key; `snapshot` takes the cold files; MEASURED gains `cold_cpu_s`,
`cold_proc_cpu_s`, `cold_wall_s`.

Tests:
- No key, nothing changes: probit-f10-p1's own spec re-created on the new file, pinned to hybrid6 as the real run,
  3 cycles (coldgames-build/repro_test.py): 57 items byte for byte (its 4,608 games in 72 rounds, the cycles,
  history, rounds, self_games, events and totals rows of cycles 1-3 with their ratings, config.json but created and
  root, 24 model and state files of versions 0-3).
- The selftest's 5 new rule checks (cold rounds incl. halves up, the cap and the floor, the share read as its decimal;
  the cold flags; the phase order; the rating directory) and cold_games_test (`selftest` without --quick): a scratch
  experiment (T-P-Ne capped x2 at ply 1, an F-P observer, self-games, games_per_cycle 45, 3 cycles) without the key,
  rating only, and training on them straight and interrupted 7 times. Rating only plays and trains as without the key
  through cycle 2 (round-robins, self-games, versions 0-2 of both lines and the observer byte for byte); training on
  them, cycle 1's round-robins and cold games are the rating-only run's, and the lines and the observer train on its
  set plus the cold games (cycle 1: 92 + 12 games); straight and interrupted equal (31 items, reports too); the
  in-run Elo (history.csv) is cold/'s fit with the key and tour/'s without; cold rounds per cycle as planned
  (6, 1, 1); coldpool/ gone after every cycle; create refuses 8 bad specs (global, no games_per_cycle, self-play
  lines, replay, a key missing, share 0, temperature as text, train as text). SELFTESTRESULT
- Dashboard: leagues without cold games render byte for byte as before (the cold test's root without the key, and
  live pages differing only in live data); with them the heading, the Elo and Standings notes, Home's lines cell,
  the CPU tile and the Cycles panel's Cold games and Cold CPU columns and stack show which games rate. DASHRESTART
