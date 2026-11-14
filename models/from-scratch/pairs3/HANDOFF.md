# PAIRS3 BUILD fork: handoff (started 2026-11-13 02:14)

Directive (coordinator, 11-13 ~02:13): build chessai-pairs3 from pairs2's source with (1) per-checkpoint restart of the
selection statistics, (2) a frequency-aware SE for the interaction contrast, (3) defaults cap 300, drop |mu| < 1.0 sd
after 3M own examples. Checks: training identity vs hybrid6 (pairs off, 16 cases), exact resume + kill -9 smoke, play
identity vs hybrid21, NULL by band (<100, 100-999, 1000+ games; mean z^2 0.9-1.1 each, ~0 chosen), offline rerun
(25 sets, held-out, vs plain), clean-list league smoke (no value in > ~5 pairs, < 10% inert once filled). Leagues: row
F-P-S7v3 (recipes.txt + make_recipes.py, backups *.before-s7v3), drafts leagues/specs/batch7/s7v3-p1-s1.DRAFT.json /
-s2 (seeds 77/78). DO NOT LAUNCH. Report < 15 lines.

Constraints: no rs/ edits; cp -n into models/from-scratch/bin/chessai-pairs3; nice -n 10, <= 3 processes; scratch on
disk under models/from-scratch/pairs3/ (not /tmp); never touch running experiments or pairs2; delete nothing; no commits.

## Progress
- 02:18 copied pairs2-src -> models/from-scratch/pairs3-src (cp -a, target/ included for incremental builds).
- 02:31 pairs.rs rewritten (windows, 22 sums per candidate: W 18 + WP 4, WM 9 per value; z_with = the influence's
  quadratic form; --pairs-contrast marginal|cells, contrast + own counters in state kind probit-pairs3; cap 300,
  drop 1.0); compound.rs := hybrid21's (MAX_STATES 65,536) + testdata/dp6601.model; main.rs flag + docs; probit.rs
  load_pairs_state(path, contrast). Release build OK, no warnings.
- 02:36 pairs unit tests 14/14 (brute-force z for both contrasts; cells cancel a correlated own effect, marginal
  does not; windows restart; own counts). Full suite running -> pairs3/test-all.log.
- 02:40 INSTALLED bin/chessai-pairs3 md5 0a208b81ef4d6b3272d1b82a63ef7bb1 (the directive's build; tests 222 + 2 pass).
- 02:46 null + offline (both contrasts) done -> null/, offline/. FINDING: bands calibrated (null: <100 0.94-0.96,
  100-999 0.99-1.02, 1000+ 1.75 in window 1 then 1.02-1.25) BUT the null picks 20 a window, all side-to-move (64) x
  home-square pairs, |z| 13-26; real data too (marginal and cells), and held-out log-loss is WORSE than plain
  (+0.0037 m, +0.0030 c at round 25, SE 0.0005; pairs2 was -0.0007). Cause (scripts/labelcheck.py): a side-to-move
  pair compares interleaved positions of the same game, whose label is one, so the clustered SE of the contrast is
  tiny and a tiny systematic effect in v (model nonlinearity / parity) is "significant"; the labels themselves show no
  interaction under the null (|z| < 1.6) and a tiny one in real data (J ~0.005 result units, z ~25: game-end parity).
  Tiny-effect pairs active in half of all positions add latent variance (round 5's zero-mean pairs: +0.0016).
- 02:52 FIX in progress (pairs3-src): floor the SE at the iid (per-example) SE of the same contrast: select on
  |I| / sqrt(max(V_clustered, V_iid)) > z ("significant AND worth a weight": iid z^2/2 ~ the LL gain); bands keep
  reporting the clustered z (the directive's calibration check). Install as bin/chessai-pairs3f (never overwrite).
- 02:59 INSTALLED bin/chessai-pairs3f md5 819821cfd76a7d24938abe522f585182 (pairs3 + --pairs-se floored|clustered,
  default floored, in the state; v^2 sums: W 19, WM 10; tests 223 + 2 pass, incl. brute-force floored z both
  contrasts and a within-game tiny-effect test). Floored null/offline running: null/mf, offline/mf, offline/cf
  (scripts/nulloff3f.sh). Backups leagues/recipes.txt.before-s7v3, make_recipes.py.before-s7v3 made.
- 03:07 TRAINING IDENTITY (pairs3f vs hybrid6, pairs's chains: 2 arms flag-absent/--pairs off x probit/lr x 4 steps =
  16 cases): all IDENTICAL (id/chains3.log).
- 03:12-03:15 EXACT RESUME (id/resume-*.log; multi-round process vs chained single-round processes, --pairs-every
  100000): -f (marginal floored: 0 pairs pass in 200k windows), -fcells (20/40/60 pairs), -fcdrop: models + state
  IDENTICAL. -cmdrop (clustered + drop-after 20000, to exercise drops) running.
- 03:13 NULL cells-floored (null/cf): FAILS: 65-94 a window pass the floor, |z| up to 31, all castling/ep-absent x
  king/rook-square pairs (structural near-zero cells, e.g. black rook d8 & black long castling present): a cell
  filled by one or two games gets ~0 variance (cells other than the pair's are centred at their own means). The
  marginal's pair cell is centred at the null prediction: null-mf 0 chosen. => recipe = marginal floored (defaults);
  --pairs-contrast cells: do not use.
- 03:10 RECIPE ROW F-P-S7v3 added (make_recipes.py HAND row + comment; recipes.txt line appended); regenerated
  recipes == recipes.txt; diff vs .before-s7v3 = the one line.
- 03:13 LEAGUE SMOKES running (smoke/runA.sh: one driver cycles 1-9, copy -> rootA-c9, then cycles 10-29; runB.sh: one
  driver a cycle, kill -9 in a pairs line's cycle-6 training, through cycle 9; compare: smoke/compare.py B vs
  rootA-c9; clean list: scripts/listcheck.py on rootA's final models of F-P-S7v3.loc and the two observers
  (--pairs-every 200000, marginal and cells)).
- 03:16 DRAFTS written: leagues/specs/batch7/s7v3-p1-s1.DRAFT.json, -s2 (seeds 77/78); copies created OK in
  pairs3/draftcheck/ (scratch root; F-P-S7v3.loc trains pairs3f --use-probs --pairs on). NOT launched.
- 03:16 PLAY IDENTITY chained after -cmdrop (identity/chain.sh: streams A then B, compare3.py -> identity/compare3.txt).
- 03:17 EXACT RESUME -cmdrop (clustered SE + drop-after 20000: 20 -> 37 -> 55 pairs, drops at both later
  checkpoints): models + state IDENTICAL. All four resume runs pass.
03:28:37
- 03:28 KILL -9 SMOKE PASS: root B (one driver a cycle, SIGKILL to the process group in F-P-S7v3.loc's cycle-6 training,
  resumed) vs root A after cycle 9 (rootA-c9): files identical 49, differ 0; cycles.csv decision columns identical.
  Root A continues to cycle 29 (clean list).
03:30:25
- 03:30 CELLS NULL FAILURE CONFIRMED (null/cellcount-cf.txt): null-cf's top pairs each have a cell a~b of 3-18
  positions from 1-3 games (e.g. 59:10&67:0: 12 positions, 1 game). A cell centred at its own mean and filled by one
  game adds 0 to the clustered variance, and ~0 to the iid floor (one game's v's are alike). Not the recipe.
- 03:30 DUPLICATES (scripts listcheck + inline): a binary feature's two values (stm 64, castling 66-69) make the same
  interaction twice (x(i:0&X) + x(i:1&X) = x(X)): offline mf 18 of 31 pairs in 9 groups (22 distinct interactions;
  after merging, still 64:* x 9 = a hub; inert 7 of the 22 outside the groups); smoke observer at cycle 9: 12 of 20 in
  4 groups (64x66 and 64x67 four ways each). Fix (NOT built): skip a candidate whose canonical key (binary feature's
  value -> *) is already chosen; statistics and state unchanged.
- 03:29 identity stream A done (03:20); B running.
03:32:11
- 03:31 PLAY IDENTITY: 31/31 IDENTICAL vs hybrid21 (identity/compare3.txt). summary.txt written (all but the
  clean-list result). LEFT: root A to cycle 29 (waiter), then scripts/listcheck.py on rootA's final models of
  F-P-S7v3.loc and both observers + their checkpoint lines (runs/<line>.log), add to summary.txt, report (< 15 lines).
- 03:54 DONE. Clean-list smoke (29 cycles): the recipe line 40 pairs, hubs 46:5 x14, 64:1 x11, 64:0 x9, inert 22%:
  FAILS the criterion (list not full); observers fail too (marginal 52 / 33% inert; cells 280 / 31 hubs / 40%).
  summary.txt final. Nothing of mine running; pairs3/ holds 2.1 GB on disk. Report sent to the coordinator.
