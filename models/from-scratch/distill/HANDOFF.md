# distill fork (started 2026-11-13 ~13:00; coordinator directive 11-13 ~12:55, the user's go: "We can test
# distillatoin inside stronger field, let's go next")

PART A: distill-p2-s1 (seed 86) / distill-p2-s2 (seed 87) = std-v1-p2-x3 from scratch, except the six tree/DAG lines
train on chessai-treelab with --label-depth 2 --label-result 0.5 --label-model F-P (the field's F-P, moving labeller).
PART B: shared label cache (models/from-scratch/labelcache/src -> bin/chessai-treelab2, cp -n), byte-identity
proofs, swap between cycles (.before-labelcache backups), PERF-LEDGER Done entry.

Constraints: no rs/ edits; never overwrite models/from-scratch/bin/*; nice; <= 3 heavy processes besides the runs;
scratch on disk (here), not /tmp; no driver strings in command lines; no pkill -f; delete nothing; no commits; `date`
before timestamps.

## Log
- 14:02 identity: x3's six tree/DAG train_args == std-v1-p1-s1's token for token (checked from both config.json);
  treelab (e7d31b0e...) vs grow7 SAME on all six arms (tree-labels/id_p1_tl.txt: tlne-capped-std, tpne-capped-std,
  tlneg-std, tpneg-bar = the standard T-P-Ne-grow, dl-std, dp-std), and grow7 == hybrid6 on the capped T-P
  (fastbin/grow7/id_p1_g7.txt), the one tree x3 trains on hybrid6. So treelab without labels = x3's binaries+flags.
- 14:05 machine idle (no experiment running; all batch-7 runs done; x3 done at cycle 437).
- Line names: x3's names + the 47-char label overrides run 99-128 chars (D-L 128: breaks self-games). Plan: five
  recipe rows (make_recipes.py HAND) = x3's exact train args on chessai-treelab: T-L-Ne-cap, T-L-Ne-grow-f10-std,
  T-P-Ne-grow-std, D-L-Ne-ctx-grow-std, D-P-Ne-ctx-grow-std (+ the existing T-P-Ne-cap); names then <= 70 chars.
- 14:08-14:09:55 smoke (smoke/, scratch root, drv.py with the temp recipes): 3 cycles OK; the six trees' commands carry
  --label-model probit:<F-P's current version> (cycle 3: probit2.model = the version that played); same calibration
  in all six (cache will hit); labels ~3.8 s wall / ~7.7 CPU-s a line at 340k positions; self-games fine (names <= 70).
- 14:11:06 recipes swapped (swap_recipes.sh): make_recipes.py 8aab75d4... (backup .before-distill), recipes.txt
  c5441f53... == the smoke's temp (backup recipes.txt.before-distill); only the five rows added.
- 14:11:31 LAUNCHED distill-p2-s1 (seed 86) and distill-p2-s2 (seed 87): leagues/specs/batch7/distill-p2-s{1,2}.json
  (DRAFTs beside). Config check: every line's train_args == x3's + the label flags, binaries treelab (6) / hybrid6
  (flats), no other config key differs from x3 but name/seed/created/tournament (hybrid21).
- 14:12:58 read waiters started: batch7-reads/distill-read.sh mid|final -> distill-read-{mid,final}.out (arenas
  diag/distill-p{2,3}-W, 48 judged + panel; x3 cross arenas diag/distill-x3-p{2,3}-W; xpool.py = cross-arena
  arm_panel, checked equal to arm_panel on a same-tour pair).
- NEXT: Part B (labelcache/src -> bin/chessai-treelab2).
## Part B (labelcache)
- 14:14 labelcache/src = cp -a tree-labels/src (its target/release/chessai == e7d31b0e treelab). New
  src/src/labelcache.rs (hybrid18 calcache's XXH64/Digest; key = exe + CHESSAI_* env less CHESSAI_LABELCACHE +
  labeller kind/use_probs/bytes (digest before load, re-checked) + depth + result-weight bits + the round's records
  (result, movetext, move count, positions); value = calibration + all labels as f64 bits + checksum; entry
  <games>/.labelcache/labels-<128-bit>.bin, flock on .lock, tmp+rename; CHESSAI_LABELCACHE=off|log). main.rs:
  Command::Train begins the cache before the labeller loads (only --label-model with --label-depth);
  train_rounds_labelled(.., cache, ..) computes via Keyed::labels. Own-search labels never cached.
- 14:16 build OK (no warnings); unit tests 253 lib (248 old + 5 new) + 1 bin pass (unit-tests.log).
- 14:18 installed bin/chessai-treelab2 (cp -n) md5 f676d842566aabf720ccc14b572a753c (also labelcache/cand/).
- 14:19 replay #1 (replay/root) FAILED in cycle 1's cleanup: my capture hook copied trainsets/ after pool/ (its link
  targets) was gone; moved aside to replay/failed-1420/. Cycle 1 there already showed the cache in a league: the first
  tree line 13.2 CPU-s, the other five ~1 s (hits). Fixed (capture when pool/ is removed); replay #2 in replay/root2.
- Proof scripts: idtest.py (6 kinds x 3 sets x old/off/cold/warm + SIX at once), compare_replay.py (replay vs live),
  swap_labelcache.py (atomic config.json replace of the six train_binary lines; backups swap/RUN/*.before-labelcache).
- 14:25:05 replay #2 done (5 cycles, replay/root2, treelab2 silent mode, sets captured in replay/sets/cycle-00{1..5}).
  compare_replay.txt: vs the live distill-p2-s1's cycles 1-5: 48 model files (8 lines x v0-v5) byte-identical;
  tour/games.csv 8,288 rows, elo_history 1,008 rows, self-games 672, cold rounds, rounds, cycles.csv, history.csv,
  events.csv, entries (as of cycle 5), 40 trainings' stdout (timings masked): all SAME, failures 0. Training CPU a
  cycle live vs replay: 78.5/19.5, 58.2/13.7, 68.6/16.2, 67.6/17.3, 67.1/17.2 (one tree line computes ~10-13 CPU-s,
  five read in ~0.5-1.2).
- 14:25 idtest.py running (-> labelcache/idtest.txt).
- 14:32:45 idtest done (labelcache/idtest.txt, idtest-summary.txt): 6 kinds x 3 sets x off/cold/warm = 54 arms SAME
  as treelab (model+state bytes, stdout masked, stderr less cache lines), notes as expected (off: none; cold: miss,
  computed and kept; warm: hit); old/off arms made no .labelcache; SIX at once on S5: 1 computed, 5 hit after
  waiting, all SAME. failures 0.
- 14:33:17 SWAPPED (labelcache/swap_labelcache.py; backups labelcache/swap/RUN/{config.json,lines.csv}.before-labelcache;
  one driver.log line each): distill-p2-s1 after cycle 26 (cycle 27 was training on treelab), distill-p2-s2 after 27.
- First cycles on treelab2: s1 28, s2 29 (one tree line computes ~10-11 CPU-s, five read ~1 s; all exit 0).
- 14:37:49 MEASURED: training CPU a cycle s1 62.3 -> 16.2, s2 58.7 -> 15.5; whole measured cycle 96.3 -> 50.1 and
  92.0 -> 49.2 (10 cycles before vs 10 after). PERF-LEDGER.md updated (Done row + queued entry marked done; backup
  labelcache/PERF-LEDGER.md.before-labelcache). labelcache/summary.txt complete.
- DONE. Running: distill-p2-s1/-s2 (repeat.sh), waiters distill-read.sh mid|final. Nothing else of mine running.
