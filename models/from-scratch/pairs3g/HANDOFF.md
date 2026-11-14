# PAIRS FINAL TEST fork: handoff (started 2026-11-13 14:05)

Directive (coordinator, 11-13 ~14:05; the user: "Let's do one final test of pairs"):
1. BUILD chessai-pairs3g = pairs3f + the duplicate fix (skip a candidate pair whose interaction duplicates an already
   chosen pair) in a copy of the pairs3 source (here: src/). Install cp -n as models/from-scratch/bin/chessai-pairs3g.
   Checks: pairs off = hybrid6 training and hybrid21 play byte-identical; unit tests; exact resume with pairs on; the
   null test briefly (bands as before); the 29-cycle clean-list smoke in a scratch root here (hubs / inert /
   duplicates). Run the test whatever the list looks like; report the list honestly.
2. LAUNCH s7v3-p1-s1/-s2 (seeds 77/78: F-P plain vs F-P-S7v3) from the drafts with the binary switched to
   chessai-pairs3g (spec-level train_binary + tournament binary; recipes.txt / make_recipes.py untouched). Description:
   the user's call, the build, the PRE-REGISTERED RULE (two seeds, windows final-10/-5/final; pairs - plain F-P below
   +20 at every ply read 1, 2, 3 closes pairwise features for good; -30 or worse at ply 3 closes them for the goal
   whatever plies 1-2 say; +20 or more at plies 2 AND 3 reopens them, a ply-2 test next). Smoke 2 cycles, create,
   launch detached.
3. READS: models/from-scratch/batch7-reads/pairs-final-read.sh mid|final; both waiters started (nohup).

Constraints: no rs/ edits; never overwrite bin/*; nice, <= 3 heavy processes; scratch on disk (here), not /tmp; no
driver strings in command lines; no pkill -f; delete nothing; no commits; `date` before timestamps.

## Progress
- 14:10 copied models/from-scratch/pairs3-src (the pairs3f source: its target/release/chessai has pairs3f's md5
  819821cf...) -> pairs3g/src (cp -a, 96 MB).
- 14:11 the copy rebuilt (touch, cargo build --release) = 819821cf... byte for byte (path-independent, reproducible).
- 14:12 FIX in src/src/pairs.rs: `interaction` (a two-valued feature's value -> u8::MAX: 64, 66-69), select() goes down
  the ranking skipping a candidate whose interaction is chosen (the post-drop list + the ones added before it), until k
  added; the checkpoint line gains "; duplicates skipped N[: pair z, ...]" before "; M chosen". Statistics, files,
  state kind (probit-pairs3) unchanged. Unit test a_candidate_whose_interaction_is_chosen_is_skipped_for_the_next.
- 14:13 INSTALLED bin/chessai-pairs3g (cp -n) md5 f42fdaff9687e5a802e4aaca631e14d9 (rebuilt twice: same md5).
- 14:15 full suite: 224 + 2 pass (test-all.log).
- 14:16 TRAINING IDENTITY (scripts/chains3g.sh, compare_chains3g.py: flag absent / --pairs off x probit / lr x 4 chained
  steps vs pairs's h6 chains): 16/16 IDENTICAL (id/chains3g.log).
- 14:17 EXACT RESUME (scripts/resume3g.sh; multi-round vs chained single rounds, --pairs-every 100000):
  -cmdrop (--pairs-se clustered --pairs-drop-after 20000: exercises the skip, 19 / 31 / 40 skipped at the three real
  checkpoints, drops 1 / 5): models after sets 1-3 + state IDENTICAL; list 20 -> 39 -> 54, DUPLICATES 0 of 54 (pairs3f's
  run: 53 of 55 in 25 groups). -fcells (no duplicate occurs): IDENTICAL, and byte-identical to pairs3f's own run
  (models 1-3 + state): without a duplicate pairs3g = pairs3f.
- 14:18 launch smoke: launch/make_specs.py -> launch/smoke/s7v3-p1-s1.json (root launch/smoke/root), created (lines
  F-P.loc and F-P-S7v3.loc both on chessai-pairs3g, tournament chessai-pairs3g); running 2 cycles.
- 14:19 NULL (null/mf, 4 min): checkpoint lines = pairs3f's (bar "; duplicates skipped 0"), 25 models + state
  byte-identical to pairs3f's null; 0 chosen. Launch smoke: 2 cycles OK (both lines on pairs3g; F-P-S7v3.loc keeps
  probit2.state; "pairs: 0 chosen" lines). CLEAN-LIST SMOKE started 14:19:42 (smoke/run.sh: root smoke/root, seed 79,
  29 cycles, one driver, observer --pairs-every 200000; then scripts/listcheck_g.py -> smoke/listcheck.txt).
- 14:23 PLAY IDENTITY (identity/identity3g.sh A + B, compare3g.py vs hybrid21's own halves): 31/31 IDENTICAL, 0 stderr
  differences (identity/compare3g.txt).
- 14:23:35 CREATED leagues/specs/batch7/s7v3-p1-s1.json / -s2.json (launch/make_specs.py: from the drafts, tournament
  binary + both entries' train_binary chessai-pairs3g, the new description with the pre-registered rule; recipes.txt
  and make_recipes.py untouched) -> leagues/experiments/s7v3-p1-s1 (seed 77), -s2 (seed 78).
- 14:23:41 LAUNCHED both: setsid nohup leagues/repeat.sh DIR >> DIR/repeat.log (pids 152947, 152955).
- 14:23:47 WAITERS: models/from-scratch/batch7-reads/pairs-final-read.sh mid -> pairs-final-mid.out (v150, plies 1-2)
  and final -> pairs-final-final.out (windows, plies 1-3, prints the rule's verdict).
- 14:27 smoke vs pairs3's root A (same seed 79, same settings): F-P-S7v3.loc's models v1-v10 and F-P.loc's v1-v11
  byte-identical; they part at v11 (the first checkpoint, in v11's training: pairs3f added 20 with 64x66 and 64x67 four
  ways each; pairs3g skipped 23 duplicates and added 20 distinct interactions, all side to move x a value).
- 14:34 CLEAN-LIST SMOKE done: recipe line 40 pairs, 0 duplicates (pairs3f 22/40), hubs 64:0 x12, 64:1 x9, 68:0 x6,
  7:0 x6 (64:* x21 by interaction), inert 22%; observer 51 pairs, 0 duplicates, hubs 65:0 x8, 59:11 x7, inert 2%.
  summary.txt complete. DONE: nothing of mine running but the two experiments' repeat.sh and the two read waiters.
