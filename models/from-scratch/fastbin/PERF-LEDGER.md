# Performance ledger: exact, deterministic speedups

Kept by the recurring performance review (the user asked for it on 2026-11-10) and by the implementation agents. Read
it before proposing anything: don't repeat what is done, queued or ruled out.

## Rules (the user's standing blessing, 2026-11-09 ~23:50)

- Any speedup that keeps everything correct and deterministic may be built and deployed without asking.
- Build in new source dirs and new binaries. Never edit rs/, never overwrite models/from-scratch/bin/.
- Prove exactness with identity checks against the current code: games, models, states and reports byte-identical.
- Swap into a running experiment only when byte-identical, between cycles (stop after the cycle, change, restart),
  or atomically during a phase that doesn't read the changed file (as the driver scan cache was).
- A deterministic speedup that could change a result (e.g. a near-tie) is fine for new experiments only, until it is
  made exact (e.g. a float re-check within the rounding bound).
- Report what was deployed and the measured saving.

## Done

| Speedup | What | Deployed | Saving | Write-up |
|---|---|---|---|---|
| chessai-hybrid4gfx | fixed-point flat LR and NB evaluation in search | before 11-09 | LR/NB search ~3× at depths 2-4 (new/old 0.32-0.38) | fastbin/fx/tables.txt |
| chessai-hybrid5 | fixed-point play for trees and DAGs at ply 1 and 2, float fallback inside the bound | 11-10 00:53 | tournament CPU per game roughly halved | fastbin/hybrid5/summary.txt |
| chessai-hybrid6 | faster training: TAN integer count batches, reused predictions, LR step from its prediction, AVX2 LR step, fast parser | 11-10 05:20 | training CPU ×0.60-0.63 | fastbin/hybrid6/summary.txt |
| chessai-hybrid7 | faster tournament reports: append-only elo_history, full fit beside prefix fits, sparse head-to-head, faster games.csv read | 11-10 09:16 (cluster-error; new experiments) | report step 12-17 s → 7-9.5 s, 2.3 GB → 0.44 GB | fastbin/hybrid7/summary.txt |
| driver scan cache | leagues/leagues.py: prefix-hash cache for the per-cycle scans of games.csv and game_cpu.csv | 11-10 10:56 (all experiments started since) | driver CPU 8.8 → 2.5 s a cycle | fastbin/drivercache/summary.txt |
| chessai-hybrid8 | flat probit (F-P) at ply 2: each reply estimated from a ln Φ table with a proven bound; only replies that can be a move's minimum valued in full (compound::best's method) | 11-10 15:34-15:35, all five running experiments (tournament binary) | F-P's own moves at ply 2 ×0.645 (157 → 101 µs/ply); F-P@2 games ×0.63-0.66 (against F-P@2) and ×0.78-0.80 (against others); ~2-3 s of game CPU a cycle (~5% in split-growth-probit and error-targets-probit, ~3% in cluster-error) | fastbin/hybrid8/summary.txt |
| chessai-hybrid9 | probit-leaf trees at ply 2: each reply's estimate takes ln Φ from hybrid8's table (bound extended by the table's error); own calibration (`arena` with A = B) loads and fits the held-out calibration once, printing the second load's lines again | 11-10 19:39-19:40: batch-4 reruns not yet created (batch4.py and their specs), pairs.py (the running pair watcher keeps hybrid8 until restarted), split-growth-probit from cycle 717, error-targets-probit from 667, nb-settings-new from 33 (config.json swapped atomically; the driver reads it when a cycle starts) | T-P@2 moves ~3-5% faster (pinned cycles ×0.957 P-core, ×0.978 E-core; the review guessed 15-25%, but a reply costs ~3,100 instructions and ln Φ is a small part); own calibration 0.17-0.40 s less a version (12-19%); ~2.5 s of ~45 s a cycle in split-growth-probit and error-targets-probit | fastbin/hybrid9/summary.txt |

| chessai-hybrid10 | values-only search (root_values: tournaments, arenas, replay-games): captures and promotions first (MVV-LVA, in place) at depth-1 nodes when a cutoff is possible, in negamax_fx and the float negamax; depth ≥ 2 ordering keys from Acc::child_sums. Outcome search (self-play's wld-score, search labels), compound and fastprobit paths unchanged; CHESSAI_ORDER1=off gives hybrid9's order | 11-11 ~01:28, installed as models/from-scratch/bin/chessai-hybrid10: the tournament/arena binary for new ply-3 work (ARENA.md says so). Not swapped into anything running: probit-widen-p1c, lr-trees-p1 and probit-trees-p2 play at ply 1-2, where it gains nothing | ply 3 game CPU a ply ×0.59-0.87 (LR 0.79, probit 0.62, NB 0.87, LR-leaf tree 0.78, probit-leaf tree 0.68, FM 0.59); plies 1-2 ×0.97-1.01 (noise; unchanged by construction); ply 4 ×0.98. A mixed ply 1-3 field ×0.72-0.77. Identity: tournaments at plies 1-4 for lr, probit, both bdt kinds, cnbp, fm and random at T 0.1 and 0.02, replay-games, own games, training (incl. search labels) and self-play all byte-identical; 163 unit tests | fastbin/hybrid10/summary.txt |
| arena own-games cache | leagues/arena.py: own calibration games shared across arenas in leagues/arenas/.owncal-cache/<key>/. The key: the own-games command without paths, plus the md5s of the model, the held-out games and the binary. Restored by reflink with each file's size and md5 checked; stored read-only. Concurrent stores are compared byte for byte; 20 GB cap, least recently used first; "owncal_cache": false gives the old behaviour | 11-11 ~01:30 (leagues/arena.py replaced by an atomic rename; backup arena.py.before-owncache); seeded with 31 verified entries | a cached entry costs ~0.1 s instead of ~1 s (calibration depth 1) or ~9.5 s (depth 2), on every repeat of an entry: extensions, and entries in several arenas (the review counted ~416 CPU s of remakes by 00:50, 13% of arena CPU). Exactness: 4 finished arenas (lr, probit, fm, both bdt kinds, cnbp; calibration depths 1 and 2; hybrid4gfx/8/9) refitted their original calibrations from fresh and from cached games; 16 entries remade independently, byte-identical | fastbin/owncache/summary.txt |
| chessai-hybrid11 | values-only fixed-point search (flat LR, probit, NB under root_values): a child's i64 sums from the move itself (Acc::move_sums) for the depth ≥ 2 ordering keys (no copy and play per key; reused key buffers; the first move by a max scan, the rest sorted only if it doesn't cut off: the stable sort's order) and for the leaves (an irreversible leaf skips the hash; repetition checked without a push); a reversible move's hash from the move where shakmaty's update declines (after a double push, king moves). Outcome search, compound, fastprobit and float paths unchanged; CHESSAI_MOVESUMS=off gives hybrid10's code | 11-11 ~03:46, installed as models/from-scratch/bin/chessai-hybrid11 (md5 8b8f1a39…): the tournament/arena binary for new work at plies 3-4, and at ply 2 with flat LR or NB entries (ARENA.md says so). Not swapped into anything running | game CPU a ply, ABBA pinned to the E-cores (unpinned runs agree): LR ply 2 ×0.85, ply 3 ×0.88, ply 4 ×0.75; NB ×0.88, ×0.87, ×0.72; probit ply 2 ×1.00 (fastprobit, unchanged), ply 3 ×0.92, ply 4 ×0.87; trees and FM ×0.92-1.07 (noise; code unchanged). Mixed field B at ply 4 ×0.94-0.96 (trees and FM dominate); lr self-play at ply 4 under softmax ×0.75. Identity: tournaments at plies 1-4 for lr, probit, both bdt kinds, cnbp, fm and random at T 0.1 and 0.02, replay-games, own games, training (incl. search labels), self-play and CHESSAI_MOVESUMS=off all byte-identical against hybrid10; check-mode games; 166 unit tests (3 new) | fastbin/hybrid11/summary.txt |
| chessai-hybrid12 | values-only fixed-point search (flat LR, probit, NB under root_values), a node one ply above the leaves with a cut window: not in check and without a pawn on its seventh rank, a legal capture of the smallest MVV-LVA key found from the bitboards (best_capture: pins along their line, king captures onto unattacked squares) is searched before any move generation; the moves are generated (captures_first's order, without it) only if it doesn't cut. Otherwise one scan finds captures_first's first move; the partition and sort only without a cut. Outcome search, compound, fastprobit and float paths unchanged; plies 1-2 never reach it; CHESSAI_STAGED=off gives hybrid11's code | 11-11 07:34, installed as models/from-scratch/bin/chessai-hybrid12 (md5 9cb4d4d1…): the tournament/arena binary for new work at plies 2-5 (ARENA.md says so). Not swapped into anything (nothing runs) | game CPU a ply, ABBA pinned to the E-cores: W4 (flat LR) ply 3 ×0.650, ply 4 ×0.952, ply 5 ×0.711; field L ×0.686, ×0.966, ×0.752 (ply 2 ×1.006); probit ×0.839, ×0.988, ×0.850; NB ×0.889, ×0.994, ×0.821; trees ×0.998 (unchanged). So ~30% of a ply-3 run, a ply-5 rung ~29 → ~21 CPU-h, a warm ply-4 rung ~0.2 CPU-h. Identity: tournaments at plies 1-5 for lr, probit, both bdt kinds, cnbp, fm, a DAG (bdd), a MoE and random at T 0.1, T 0.02 and argmax, replay-games, own games, training (incl. search labels), 14 self-play runs and CHESSAI_STAGED=off all byte-identical against hybrid11; check-mode games (each staged move asserted legal and of the smallest key) at plies 3-5; 169 unit tests (3 new: best_capture against shakmaty's legal moves on 102,957 positions, set positions, root values bit-equal staged on/off at depths 1-5) | fastbin/hybrid12/summary.txt |
| chessai-hybrid13 | root cut windows for the log-odds softmax at 0 < T ≤ 0.05 and the log-odds argmax, at depth ≥ 2 on the fixed-point full search (flat LR; probit past ply 2) and the float negamax (trees and DAGs past ply 2, FM, MoE): root moves by static key, the first with a full window, the rest with (best − 40T, ∞) (argmax: the next float below the best). Fail-soft, so a value above the bottom is exact (root_values's bits) and one below is an upper bound. The pick is softmax_sample's: decided when the weight sum is bit-equal with each cut weight at 0 and at hi = max(2·exp((bound − max)/T), MIN_POSITIVE), and at each cut move the walk passes, target ≥ hi and target − hi == target; otherwise that move is searched in full (all of them before the draw if the sum is open). One draw, as before. T 0.1, ply 1, the outcome policies, compound, fastprobit and NB (excluded: 3-6% of moves cut, keys cost more) keep hybrid12's code; CHESSAI_WINDOWS=off gives it everywhere, CHESSAI_WINDOW_CHECK=1 runs both and asserts | 11-11 08:11, installed as models/from-scratch/bin/chessai-hybrid13 (md5 804710df…). Not swapped into anything (nothing runs) | game CPU a ply at T 0.02, ABBA pinned to the E-cores: W4 (flat LR) plies 2-5 ×0.843, ×0.765, ×0.738, ×0.721; field L ×0.774, ×0.771, ×0.765, ×0.763; probit ply 3 ×0.865, ply 4 ×0.657, ply 5 ×0.755 (ply 2 ×1.000, fastprobit); tree TL ply 3 ×0.880, ply 4 ×0.758 (ply 2 ×0.978, compound); NB ×1.000-1.006. Argmax W4 ply 3 ×0.436, ply 4 ×0.244. T 0.1 ×0.990-1.002 (unchanged code). Unpinned mixed fields at T 0.02 ×0.69-0.87, argmax ×0.21-0.60. The bounds decided 98.5% of T 0.02 picks; 0.98% of cut moves (0.38% of root moves) searched again. Identity: tournaments at plies 1-5 for lr, probit, both bdt kinds, cnbp, fm, a DAG, a MoE and random at T 0.1, T 0.02, T 0.001, T 0.05 and argmax, replay-games, own games, training (incl. search labels), 21 self-play runs (T 0.02 too) and CHESSAI_WINDOWS=off all byte-identical against hybrid12 (55 of 55, twice); check mode asserted 51,999 windowed softmax and 7,023 argmax searches at plies 2-5 (55 cases, games the same); 173 unit tests (4 new: 300,000 adversarial bounded picks, 2,856 windowed root-value cases, 2,640 windowed picks, the path rules) | fastbin/hybrid13/summary.txt |
| chessai-errdag4 | errdag3 (fastbin/errdag-src) plus: in DAG training each node's path-independent output (CNBP class products before the cells' factors, LR log-odds, probit latent mean and variance) computed once per training step and kept in the node (dropped wherever the per-(step, cells) cache is: 14 sites), the path's cell terms added per call by the same operations; CNBP's three class chains in one pass (training and play); class-sized buffers instead of zeroed 256-wide ones. Byte-identical to errdag3, to errdag2 for LR, to hybrid13 on non-DAG kinds | 11-11 13:58:30 error-dag-probit-p1 (errdag3 → errdag4 from cycle 248), 14:01:27 error-dag-p1 (errdag2 → errdag4 from cycle 140): tournament, own calibration, self-games and the 4 DAG lines' training; config.json replaced atomically during a tournament phase, drivers not stopped | live, per cycle: CPU ×0.62/×0.63 (90 → 56 s, 102 → 65 s; DAG training ×0.45, games ×0.70-0.75, own calibration ×0.75-0.78), wall ×0.65/×0.63 | fastbin/errdag4/summary.txt |
| chessai-hybrid14 | errdag4's model code (path context, router splits and their knobs, probit leaves and cells, errdag4's training cache) merged into hybrid13-src, so every kind gets hybrid5-13's play and hybrid6's training; a ctx DAG whose path tables have no cells (no router split yet) takes hybrid5's fixed-point path (Graph::has_cells), one with cells errdag4's float path | 11-11 ~16:22, installed as models/from-scratch/bin/chessai-hybrid14 (md5 3976e92d…). Not swapped into anything: the global league and the ply-2 fields were to be created on it | play, ABBA pinned, identical games: trees and flats ×0.93-0.99 of hybrid13; cell-less DAGs ×0.74/×0.44 (D-P, plies 1/2) and ×0.63/×0.32 (D-L) of errdag4, the T-*-Ne tree's cost exactly; DAGs with cells ×0.89 (ply 1), ×0.63-0.73 (ply 2); training (unpinned, indicative) ctx DAGs ×0.43-0.55 of errdag4. Identity: errdag4's 36 training comparisons, live-state rounds and v0, 161 play comparisons (depths 1-3, T 0.1 and 0.02, own calibration) and whole-cycle dry runs of both DAG experiments byte-identical against errdag4 / hybrid13 / hybrid6 | fastbin/hybrid14/summary.txt |
| chessai-hybrid15 | hybrid14 plus fixed-point play for DAGs with path cells: every (node, path marks) state a path reaches (≤ 4,096) gets its cell terms in fixed point (LR router offsets, ln CNBP factors, LR leaf logit shifts, probit leaf (mu, s2)) and a margin widened by their error; routers decide in fixed point outside the margin, else descend in float (Graph::descend_from); leaf estimates shifted with bounds, best()'s float valuation as before. CHESSAI_FX_CELLS=off gives hybrid14 | 11-11 16:32, installed as models/from-scratch/bin/chessai-hybrid15 (md5 58717c38…). Not swapped into anything (the global league and the ply-2 fields not created yet) | play, ABBA pinned, identical games, DAGs with cells: ply 1 ×0.70-0.74 (against trees), self-games ×0.52-0.54; ply 2 ×0.40-0.44 against trees, ×0.31 DAG against DAG, self-games ×0.27-0.31; against errdag4 ×0.25-0.32 at ply 2. A cell DAG now costs ~1.3× (D-P) / ~1.8× (D-L) its tree at ply 2 (hybrid14: ~4× / ~6.5×). Cell-less kinds ×0.95-1.00 (same code). Identity: FX_CHECK arenas on real cell DAGs (every decision asserted, 0 float fallbacks), 161 play comparisons against hybrid14, the 36 training comparisons, whole-cycle dry runs of both DAG experiments against errdag4: all byte-identical; 196 unit tests (1 new) | fastbin/hybrid15/summary.txt |
| chessai-hybrid16 | hybrid15 plus a content-addressed, cross-process cache of calibration fits (cal:, leafcal:, blend: of cnbp, lr, probit, bdt, bdd): <GAMES>/calfit-<key>.bin inside the own-games directory (only when its parent is owncal/), key = XXH64×2 of the binary's bytes, the CHESSAI_* environment, the wrapper and its options, the inner kind, --use-probs, the model file and the three outcome files; value = the fit's f64 bits and its leaf lines; temp file + rename; anything unexpected fits as before. CHESSAI_CALCACHE=off gives hybrid15 | 11-11 20:05, installed as models/from-scratch/bin/chessai-hybrid16 (md5 0cb3d238…); tournament binary of leagues/global from cycle 720 and base-probit-p2 from cycle 411 (not base-lr-p2: ~50-60 cycles left to its floor at 548) | a hit saves ~0.12 s (ply 1) / ~0.14 s (ply 2) a fit, a miss costs ~0.01 s: global ~2.3-2.5 CPU-s a cycle of ~26-30 (~8-9%; live cycles 721-722: self-games 1.7-1.9 → 0.24-0.25, tournament −0.8), base-probit-p2 ~2.4-2.7 of ~65 (~4%) | fastbin/hybrid16/summary.txt |
| chessai-hybrid17 | hybrid16 plus (1) the rating fits' field search over only the fields of the set's rarest variable, in creation order (same first field by construction; no k-wide bitmaps) and (2) write_round's digits straight into one buffer (no String per feature value) | built 11-12 07:26 for new work; not swapped | global tournament step x0.74 (9.2 -> 6.8 CPU-s, -2.4 a cycle, growing with history); own calibration x0.83 (1.30 -> 1.07 s a version); a six-line ply-1 cycle ~11-12% | fastbin/hybrid17/summary.txt |
| chessai-hybrid18 | values-only search past ply 2 (root_values at depth ≥ 3: tournaments, arenas, replay-games): (1) trees and DAGs of ≤ 64 nodes keep the compound fixed-point state (a compound::Acc per depth, Changes per move, no encode()); a depth-1 node scans its replies' estimates in the float order, fails high when a top ≤ −β, fails low when α > −∞ and every bottom ≥ −α, else values in full only the replies that can be the minimum (Compound::best); keys at depth ≥ 2 from the estimates in the exact float order (order_by_bounds: the exact key only where intervals overlap); (2) flat probit the same with hybrid8's ln Φ table (fastprobit::estimate/best). Fail-soft: same classifications and exact values, root values bit-equal. Outcome search, plies 1-2, flat LR/NB, FM, MoE, graphs over 64 nodes, own calibration and training unchanged; CHESSAI_FX_DEEP=off gives hybrid17 | 11-12 14:17, installed as models/from-scratch/bin/chessai-hybrid18 (md5 392eab6a…); probit-p3's tournament binary from cycle 235 (config.json swapped 14:17:10; cycle 235 byte-identical to a hybrid17 replay) | game CPU a ply, ABBA pinned: probit-p3's kinds ply 3 ×0.62; T-P-Ne ×0.55 (ply 3), ×0.48 (ply 4); T-L-Ne, LR-router tree and cell DAG ×0.36 (ply 3); F-P ×0.80 (ply 3, T 0.02); T-P-Ne v F-P ×0.61 (ply 5). probit-p3 live: tournament 100.8 → 63.3 CPU-s a cycle, self-games 14.9 → 9.2 (~43 CPU-s a cycle; ~0.8 CPU-h to the floor, ~2.0 to the cap). Identity: tournaments at plies 3-5 (T-P-Ne, T-L-Ne, LR- and TAN-router trees, DAGs with and without cells, a 121-node tree over the cap, F-P, F-L, random) at T 0.1, T 0.02 and argmax, check mode at plies 3-4, self-games: all byte-identical against hybrid17; 205 unit tests (3 new); outcome search unchanged by diff | fastbin/hybrid18/summary.txt |
| chessai-hybrid20 | hybrid19 with compound::MAX_NODES 64 → 128: trees and DAGs of 65-128 nodes take the compound fixed-point path instead of float (plies 1-2, and hybrid18's deep search past ply 2). No structure or bound depends on the node count: per-node tables, sums < 2^59, margins and error bounds; route states keep their own 4,096 cap. So 64 was hybrid5's speed guard | built 11-12 19:09 for new work; the ply-2 rung's and probit-p4w's drafts point at it (19:30); not swapped (std-v1-p1 plays ply 1, its DAGs under 64) | games with a tree or DAG over 64 nodes ×0.39-0.41 at ply 2 (T 0.1, T 0.02), ×0.53-0.76 at ply 3, ×0.39-0.58 at ply 4, ×0.51 at ply 5; ply 1 ~1; argmax at ply 2 ×1.44 (compound's depth-2 path has no root windows); probit-p4w ~25%, ~6-19 CPU-h | fastbin/hybrid20/summary.txt |
| chessai-hybrid21 | hybrid20 with compound::MAX_STATES 4,096 -> 65,536: DAGs with cells past 4,096 route states (the warm ply-2 standard's D-P lines at v100: 4,756 and 6,601) take the compound fixed-point path instead of float. The cap only decides eligibility (states in a Vec + (node, marks) index, u32 ids, per-state margins and fallbacks) | built 11-13 01:07; swapped into std-v1-p2-s1 01:18:09 and -s2 01:23:09 (before their extension); make_split_read.py's plies 1-2 tournament binary 01:18:26 | games with such a DAG x0.40 (ply 2, T 0.1), x0.49 (T 0.02), x0.40-0.52 (ply 3), x0.40-0.48 (ply 4); s1's cycle 101: ~25 CPU-s a cycle (~2.2-2.5 CPU-h over the extension) | fastbin/hybrid21/summary.txt |
| chessai-treelab2 | chessai-treelab plus the shared label cache: the labels of `train --label-depth D --label-model KIND:PATH` (calibration fit + search labels) content-addressed in `<games>/.labelcache/labels-<key>.bin`; key = the binary's bytes, the CHESSAI_* environment, the labeller's kind, --use-probs and bytes, depth, result-weight bits and the round's records in read order (128-bit digests; whole key compared, checksum); a miss takes flock on the key, looks again, computes, writes (temp + rename), so of lines started together one computes; labellers cnbp/lr/probit/bdt/bdd only, own-search labels never; CHESSAI_LABELCACHE=off / log. Proofs: 6 kinds x 3 sets x off/cold/warm and six processes at once byte-identical to treelab (models, states, stdout), a 5-cycle league replay identical to the live run (models v0-v5, games, ratings, decisions) | built 11-13 14:16; swapped into distill-p2-s1 and -s2 at 14:33:17 (their first cycles on it: 28 and 29) | training CPU a cycle 62.3 → 16.2 (s1) and 58.7 → 15.5 (s2), the whole measured cycle 96.3 → 50.1 and 92.0 → 49.2 (10 cycles before vs 10 after); same cycles in the 5-cycle replay 58-79 → 14-20; one label computation a cycle instead of six: ~45 CPU-s a cycle a run, ~10 CPU-h over two seeds to cycle 400 (~15 to the cap) | labelcache/summary.txt |

## Queued

- **Done (11-13, the distill fork; Done table: chessai-treelab2): the shared label cache** (was: conditional, 11-13 ~12:59 review), built into a new treelab binary before the
  distillation run if the user approves that run. Content-addressed and flock-guarded, in <games>/.labelcache/. Saves
  5 of 6 label computations: ~5-13 CPU-h over two seeds, 26-43%. Design and proof plan are in that review's section.
  The lazy PV outcome (review-1113b's candidate 1) stays conditional on own-search tree labels.
- **Done (11-13 01:27): candidates 1 and 2 of the 11-13 ~01:00 review**, as chessai-hybrid21 (Done table;
  fastbin/hybrid21/summary.txt; section at the end): compound::MAX_STATES 4,096 -> 65,536 and nothing else, and
  make_split_read.py's tournament binary at plies 1-2 hybrid16 -> hybrid21. 31 identity cases byte-identical
  (~5,070 games, plies 1-5, check mode 178 M router decisions), both warm runs' cycle 101 replayed under
  extend_check, and the trained v101 models byte-identical to a hybrid20 replay. Swapped into std-v1-p2-s1 (01:18:09)
  and -s2 (01:23:09) before their extension; make_split_read.py installed 01:18:26.
- **Done (11-12 ~19:30): candidate 1 of the ~18:50 review**, as chessai-hybrid20 (Done table;
  fastbin/hybrid20/summary.txt; section at the end): compound::MAX_NODES 64 -> 128 and nothing else. 128 is safe for
  all inputs: nothing in the fixed-point path depends on the node count, and 64 was hybrid5's undocumented speed
  guard. Byte-identical to hybrid19 in 28 cases at plies 1-5, 3,467 games:
  - T 0.1, T 0.02 and argmax, openings on and off;
  - tournaments with saved games, self-games, own calibration, and check mode at plies 1-4.
  Unit tests: 207 + 2, one new (121 nodes). Test-only extras (cell DAGs of 67-70 nodes, a 121-node tree of mixed
  kinds) are in hybrid20/over64-dag-test.patch. Games with a player over 64 nodes: x0.39-0.41 at ply 2 (T 0.1,
  T 0.02), x0.39-0.58 at ply 4; argmax at ply 2 x1.44. The ply-2 rung's and probit-p4w's drafts point at it (19:30).
  Nothing running was swapped.
- **Done (11-12 14:17): candidates 1 and 2 of the ~12:50 review**, as chessai-hybrid18 (Done table;
  fastbin/hybrid18/summary.txt; section at the end): past ply 2, (1) trees and DAGs (≤ 64 nodes) search in the
  compound fixed-point state with their leaves' bounds and estimate-ordered keys, (2) flat probit the same with
  hybrid8's ln Φ table. Byte-identical to hybrid17 at plies 3-5 (T 0.1, T 0.02, argmax; every kind of the directive),
  in check mode (plies 3-4), in self-games; 205 unit tests; the outcome search unchanged by diff. Game CPU a ply,
  pinned: probit-p3's kinds at ply 3 ×0.62, T-P-Ne ×0.55 (ply 3) / ×0.48 (ply 4), T-L-Ne and router trees with a cell
  DAG ×0.36, F-P ×0.80. Swapped into probit-p3 from cycle 235 (14:17:10); cycle 235's tournament, self-games and own
  games byte-identical to a hybrid17 replay of that cycle. Live (cycles 236-247 against 220-234): tournament 100.8 →
  63.3 CPU-s a cycle, self-games 14.9 → 9.2 (~43 CPU-s a cycle saved).
- **Done (11-12 07:26): candidates 1 and 2 of the ~06:45 review**, as chessai-hybrid17 (Done table;
  fastbin/hybrid17/summary.txt; section at the end): (1) Blocks::new searches only the fields holding the set's rarest
  variable, in creation order; (2) write_round writes digits straight into the buffer. Byte-identical to hybrid16 on
  a global-league tournament step (cycle 5060 snapshot, 25,305 round-robins), a grow-p1b cycle, 13 own calibrations,
  self-games and self-play. Global tournament step x0.74, own calibration x0.83. For NEW work (the next global league,
  new ply-1 runs). NOT swapped into the running global league: faster cycles would fill the disk faster (~4-5 days to
  full at the review's measured rates), and its future is the user's call.
- **Done (11-11 20:05): candidate 1 of the ~18:50 review**, the calibration-fit cache, as chessai-hybrid16 (Done table; fastbin/hybrid16/summary.txt; section at the end). Byte-identical to hybrid15 with the cache off, cold and warm (whole-cycle dry runs of the global league and base-probit-p2); swapped into leagues/global (cycle 720) and base-probit-p2 (cycle 411) as the tournament binary. Saving ~2.3-2.5 CPU-s a global cycle (~8-9%, ~2.6 CPU-h a day at ~4,000 cycles of 21 s), ~4% of base-probit-p2.
- **Done (11-11 07:34): candidate 1 of the 06:50 review**, the staged first move at depth-1 cut nodes: installed as
  chessai-hybrid12 (Done table; fastbin/hybrid12/summary.txt). Byte-identical to hybrid11; W4 at plies 3/4/5
  ×0.650/×0.952/×0.711 (the review measured ×0.71/×0.95/×0.70).
- **Done (11-11 08:11): candidate 2 of the 06:50 review**, root cut windows for the log-odds softmax at low
  temperature (and the argmax): installed as chessai-hybrid13 (Done table; fastbin/hybrid13/summary.txt).
  Byte-identical to hybrid12; flat LR at T 0.02 ×0.72-0.84 at plies 2-5 (the review measured ×0.70-0.77), probit and
  trees ×0.66-0.88 at plies 3-5, argmax ×0.24-0.44. Worth ~25% of T 0.02 reads and of any T 0.02 training; nothing
  at T 0.1.
- **Candidate 3**, lazy leaf terminal checks at even plies: only if a ply-4 rung or a ply-4 field goes ahead.
- **Done (11-11 16:22 and 16:32): candidates 3 and 4 of the ~12:50 review**, as chessai-hybrid14 (errdag4's
  model code in hybrid13-src; cell-less ctx DAGs on the fixed-point path) and chessai-hybrid15 (fixed-point play
  for DAGs with cells) (Done table; fastbin/hybrid14/summary.txt, fastbin/hybrid15/summary.txt). The review
  guessed a ply-2 DAG field's game CPU down by about two thirds with both: measured against errdag4 at ply 2,
  cell-less DAGs ×0.32-0.44, DAGs with cells ×0.25-0.32.
- Noted for the next review (11-11 ~01:35, hybrid10):
  - At ply 4, hybrid10 saves only ~2%: the static-evaluation keys at depth-2 and depth-3 nodes dominate (a key per
    child: a full float evaluation for trees and FM). Cheaper ordering at depth-2 nodes, e.g. captures first instead
    of keys, would be exact on the values-only path. Worth measuring only if ply-4 play is wanted.
    - 11-11 ~03:50: hybrid11 made the fixed-point keys cheap (LR at ply 4 ×0.75). Trees and FM still pay a float
      evaluation per key.
  - A nearly untrained model gains nothing from captures-first. A league's first versions at ply 3 may cost 10-20%
    more (self-play's first rounds did).
- Noted for the next review (11-11 ~03:50, from hybrid11). All are exact on the values-only path, where any move
  order gives the same root values, so only node counts would change:
  - Probit keys. Each ordering key still evaluates the probit link, and doubling the keys added ~40% to a probit
    ply-4 move (hybrid10). A cheaper key, such as the linear predictor if the value is monotone in it, would keep
    every value but may change the order and the node count. Measure the nodes.
  - Depth-1 cut nodes. 87% cut on their first move, after full move generation, and generation is ~10% of ply-4
    CPU. Generating captures first and quiet moves only if no capture cuts would save part of that.
  - Probit leaves under a cut window. A bound from hybrid8's ln Φ table could settle most leaves, as hybrid8 does
    at ply 2. Fail-soft bounds must stay valid.
- Benchmark method (11-11, hybrid11): this CPU has P-cores 0-3 at 4.7 GHz and E-cores 4-7 at 3.7 GHz. Unpinned runs
  land on either type, and ABBA pairs swung ±10% (TP at ply 3 read x1.00-1.09 though its code is unchanged). Pin
  comparisons with taskset -c 4-7: pairs then agree within a few percent.
- Noted for the next review (11-11 ~01:40, from the entry_ply 3 build): when a cycle trains with 2 jobs, the
  driver's busy check sometimes waits 30 s on its own sibling training process ("waiting for pid ..., still working
  in this league" in repeat.log). It is wall time only, and old behaviour visible in the live logs. Check how often
  it fires and whether the check can exclude the cycle's own children (results unchanged).
- Use: chessai-hybrid10 for every new ply-3 tournament and arena (flat-p3 from 01:34 is the first).
  - From 11-11 ~03:46: chessai-hybrid11 for new tournaments and arenas at plies 3-4, and at ply 2 with flat LR or NB
    entries. It plays the same games as hybrid10. For fields of trees and FM alone, the two cost the same.
  - From 11-11 07:34: chessai-hybrid12 instead of hybrid11, for new tournaments, arenas and self-games at plies 2-5,
    any kind. It plays the same games as hybrid11. Flat LR, probit and NB moves cost 11-35% less at plies 3 and 5,
    1-5% less at ply 4. Plies 1-2, trees, DAGs, MoEs and FM cost the same.
  - From 11-11 08:11: chessai-hybrid13 instead of hybrid12, for new tournaments, arenas, self-games and self-play at
    any ply and kind. It plays the same games as hybrid12. It saves only with the log-odds softmax at T ≤ 0.05 (T 0.02
    reads, a T 0.02 training arm) and the log-odds argmax, at ply 2 and up: flat LR ×0.72-0.84 (plies 2-5), probit
    ×0.66-0.87 (plies 3-5), trees ×0.76-0.88 (plies 3-4), FM, DAGs and MoEs past ply 2 likewise. T 0.1, ply 1, NB,
    probit at ply 2 and trees or DAGs at plies 1-2 cost the same as hybrid12.
  - From 11-11 14:01: chessai-errdag4 instead of errdag2 and errdag3 wherever those run (fields with path-context
    DAGs: tournaments, arenas, own calibration, self-games, DAG training). Same games, models and states as errdag3
    (and errdag2 for LR). DAG training ×0.45, DAG play ×0.6-0.75. Fields without ctx DAGs stay on hybrid13 (×0.54
    of errdag4's CPU on F-L, F-P and trees, identical games).
  - From 11-11 20:05: chessai-hybrid16 instead of hybrid15 for tournaments, self-games and own calibration of any
    league or arena whose players are cal:/leafcal:/blend: of own games (owncal/ dirs). Same games, ratings and
    fits as hybrid15; a fit repeated by a later process (champions every cycle, self-games after the tournament)
    costs ~0.01 s instead of ~0.12-0.14 s. Training is the same code as hybrid15 (no reason to swap it).
    Arenas: own games made with hybrid16 get new keys in arena.py's own-games cache (it keys on the binary's md5),
    so entries cached under hybrid15 are made once more.
  - From 11-12 14:17: chessai-hybrid18 instead of hybrid17 for new specs, arenas and ladder runs with players at ply
    3 or more (tournaments, self-games, own calibration). It plays the same games as hybrid17. It saves on trees and
    DAGs of ≤ 64 nodes and on flat probit, at plies 3-5: game CPU a ply ×0.36-0.62 for trees and DAGs, ×0.80 for
    F-P (T 0.02), probit-p3's mix ×0.62. Plies 1-2, flat LR and NB, FM, MoEs, graphs over 64 nodes, the outcome
    search, own calibration and training run hybrid17's code. Running arenas keep their binary. As with any switch,
    each calibration fit is redone once (the fit cache keys on the binary's bytes) and arena.py remakes own games
    once (it keys on the binary's md5). specs/arenas/make_ladder.py (HYBRID17) and existing specs still name
    hybrid17. CHESSAI_FX_DEEP=off gives hybrid17's search.
    11-12 ~14:27 (coordinator): specs/arenas/make_ladder.py now defaults to hybrid18 (HYBRID18; backup
    .before-hybrid18), and make_split_read.py gives arenas at ply >= 3 the hybrid18 tournament binary (own
    calibration stays hybrid16; plies 1-2 unchanged; backup .before-hybrid18).
- **Done (11-11 14:01): candidates 1 and 2 of the ~12:50 review** as one binary, chessai-errdag4 (Done table;
  fastbin/errdag4/summary.txt; section at the end). Swapped into error-dag-probit-p1 and error-dag-p1.

## Ruled out or left (with reasons)

- A transposition table at ply 4, flat LR (hybrid11's counters): ~10.5% of leaves repeat a position, ~4% net after
  the table's own cost. Left.
- Killer moves first at ply 4 (hybrid11's counters): the killer is the cut move in only 45% of cut nodes. Captures
  first and the static keys already cut on the first move in 87% of depth-1 and 78% of depth-2 cut nodes, so killers
  are a net loss.

- contrast_ses' sparse elimination and dense Cholesky in leagues.py (~1 s a cycle, grows with the players): no exact
  speedup found that keeps the same floating-point operations in the same order.
- The records phase's CSV rewrites (~0.4 s a cycle): small.
- The scan cache hashes each file once a cycle (~0.3 s per 500 MB). A cheaper proof would trust appends: not exact.
- Mixtures (MoE) and FM still play the float code (hybrid5 skipped them). Worth doing only if they return to
  experiments.
- Flat probit at ply 1 (hybrid8 left it): the log-odds softmax needs every root value exactly, so a bound saves
  nothing; F-P at depth 1 is cheap (the own-calibration arenas).
- Flat probit self-play search at depths 2-4 (hybrid8 left it). Self-play's default policy, wld-score-jitter10,
  searches with root_outcomes and Value::Score (calibrated expected score as log-odds, a different leaf function to
  bound) and needs the principal variation's leaf; at depth 3 and more, negamax_fx also orders moves by their static
  values, so exact play needs exact ordering keys and cutoff decisions. Possible (value in full whenever a bound
  straddles alpha, beta or an ordering neighbour), not straightforward. No running experiment self-plays probit;
  worth doing if P4 (probit self-play at depth 2) starts.

## Reviews

(Dated sections appended by each review: what was measured, ranked candidates with current cost, estimated saving,
exactness approach and risk, and what was checked and dropped.)

### 2026-11-10 ~19:10 review (scheduled; first since the ledger began)

**What runs and what changed.**
- Batch 4 (from 17:41) reruns every old experiment one at a time. Its fields have no random players and no ply-1
  entries, so every game is ply 2 against ply 2, at 12-34 rounds a cycle, with the plateau stop.
  - lr-settings-new is done (194 cycles). probit-settings-new is running.
  - About 100 CPU-h are left in the queue: NEXT-EXPERIMENTS "Batch 4" estimates, plus small-field-x4.
  - The old-design runs end tonight (small-field ~19:35; split-growth-probit and error-targets-probit ~22:00) and
    are not targeted.
- Binaries: tournament chessai-hybrid8, training hybrid6 (chessai-target and chessai-trigger on their lines). No new
  binary since hybrid8.
- leagues.py gained the batch-4 options (references [], ply1_entries false, plateau).

**Measured, from records (cycle-NNN.md over the last 30 cycles; tour/game_cpu.csv joined to games.csv over the
last 20 cycles).**
- CPU-s a cycle:

  | Run | games | tournament process | training | own calibration | wall |
  |---|---|---|---|---|---|
  | lr-settings-new | 11.1 | 13.1 | 5.7 | 5.0 (5 versions) | 17.2 |
  | probit-settings-new | 10.0 | 11.5 | 0.4 | 4.7 (4 versions) | 9.6 |

  - So own calibration is 21-28% of these flat runs' CPU.
  - Old tree runs, for the reruns' mix: tournament 15-47, training 2-7 (cluster-error 43), own calibration 4-11
    (cluster-error 20).
- Game CPU at ply 2 by pairing, µs a ply:

  | Pairing | µs a ply |
  |---|---|
  | probit-leaf tree against probit-leaf tree | 155-164 |
  | probit-leaf tree against F-P | 130 |
  | F-P against F-P | 100-106 |
  | LR-leaf tree against LR-leaf tree | 126 |
  | LR-leaf tree against F-L | 103 |
  | F-L against F-L | 81 |

- Training µs a position, by router: TAN-router trees 6.5-7.6, NB 2.8-4.4, LR cluster 1.5-1.9. In cluster-error the
  four TAN trees take ~22 of 43 training s a cycle.

**Profiles** (snapshots in the session scratchpad perf/snap; nice 19, 1 thread).
- perf is blocked here (SELinux, perf_event_paranoid 2). I used gdb stack sampling instead: 80 samples, 49 on the
  busy thread.
- Probit-leaf tree at ply 2 (split-growth-probit's T-P-Nc-merge limit-1 version), playing itself (arena, depth 2):
  - Search::ply1_compound 18 samples (its inlined estimate and routing);
  - ln Phi: 11 (ln_cdf_first 4, plus the ln and ln_1p it inlines 7), about 23%;
  - Changes::fill 6, has_legal_move 5, Table::moved 3.
- Own calibration (`arena cal:H:K:M cal:H:K:M`, 600 games at depth 1), CPU s:

  | Model | whole step | one held-out fit | the second fit |
  |---|---|---|---|
  | LR | 0.66 | 0.19 | 29% of the step |
  | probit | 0.92 | 0.23 | 25% |
  | probit-leaf tree | 1.21 | 0.24 | 20% |
  | LR-leaf tree | 1.14 | 0.21 | 18% |

**Ranked candidates.**
1. **Probit-leaf trees at ply 2: hybrid8's ln Phi table inside compound's reply estimate** (compound.rs `estimate`,
   the two `ln_cdf_first` calls per reply).
   - Cost now: probit-leaf tree moves at ply 2 are about 22 CPU-h of the queue. They come from single-split-probit-new
     (7.4 CPU-h), split-growth-probit-new (11.8), error-targets-probit-new (6.8) and cluster-error-new (29.4; 12 of
     its 28 entries).
   - Estimated saving: 15-25% of those moves, ~3-5 CPU-h. That is ~8-13% of split-growth-probit-new's and
     single-split-probit-new's time and ~4-7% of cluster-error-new's. A table lookup costs a few ns against ~20-40
     ns for the rational ln Phi; slightly wider bounds send a few more replies to full valuation.
   - Exactness: the estimate only chooses which replies are valued in full, and values stay the exact path's.
     - Use hybrid8's table and bound (chord + h^2/16 within h^2/16, plus the float allowance 1e-13 + 16u|ln Phi|),
       composed into compound's derivative terms.
     - Check mode asserts every estimate is within its bound.
     - Identity: tournaments byte-identical on recorded fields of split-growth-probit, error-targets-probit,
       single-split-probit and cluster-error, as hybrid8 did.
   - Risk: low. The table and bound are proven in hybrid8, and the code has the same structure as compound's
     existing estimate.
   - Timing: the first probit-leaf rerun is single-split-probit-new at queue position 9, ~11-11 13:00 at today's
     pace.
2. **Own calibration: fit the held-out calibration once when arena's A and B specs are the same** (main.rs `arena`:
   LoadedModel::load(a) and load(b) each run Calibrated::fit on the 163,219 held-out positions, and own calibration
   passes the same spec twice).
   - Cost now: 0.19-0.24 CPU-s a version for the second fit. About 28,600 version calibrations are left in the
     queue (estimated cycles × lines).
   - Estimated saving: ~1.6 CPU-h. That is ~5-6% of the small flat runs' CPU (probit-settings-new now,
     lr-settings-2-new, nb-settings*) and 2-3% of tree fields'.
   - Exactness: the same deterministic computation runs once and both players share it (compound's OnceLock tables
     are built once either way).
     - Print the calibration line twice so the .arena log stays the same.
     - Identity: owncal game files byte-identical for LR, probit, cnbp and tree versions.
   - Risk: very low. Best folded into the same new tournament binary as candidate 1.
3. **Cache the parsed held-out set** (games::read_labelled re-reads 24 MB in every own-calibration process).
   - Of the remaining ~0.2 s fit, ~0.12-0.15 s is model-independent: LR's fit costs nearly as much as probit's.
   - Estimated saving: ~0.1 s a version, ~0.8 CPU-h. Below the bar on its own.
   - Exactness: the cache must give the same Encoded bytes and labels, keyed by the held-out files' md5.
   - Risk: low, but it adds a cache file outside the experiments. Do it only alongside candidate 2.

**Checked and dropped, or left for the next review.**
- Running own calibration wider than its jobs=2: wall time only, not CPU.
- Tree play at ply 2: Changes::fill (12%), has_legal_move (10%) and Table::moved (6%) are the incremental features
  and terminal checks, already fast paths. No exact change seen.
- The tournament process outside games (1.5-2 s a cycle in the batch-4 flat runs, ~10%): contrast_ses was ruled out
  before; nothing new.
- **Next review's first item:** TAN-router training (6.5-7.6 µs a position, about 2× NB routers; ~2.7 CPU-h of
  cluster-error-new, ~4-5 CPU-h across the queue). It needs a replayed training step to profile. Do it before
  cluster-error-new starts (queue position 10).

### 2026-11-11 ~00:50 review (scheduled; the user's reminder of ~00:05: "don't forget to have the scheduled optimization/performance expert/reviewer take a look")

**What runs and what changed.**
- Batch 4 ended at 23:48 (the user paused ply-2 experiments). The ply-1 runs are done. probit-widen-p1b finished at
  00:53 (plateau at cycle 200). Nothing runs now except arenas, which the coordinator starts one or two at a time.
- Tonight's measurement work moved to small arenas: 16 since 00:00. Many are near argmax (T 0.02), and since ~00:40
  some are at ply 3 (diag/ply3-cold, ply3-labels-cold, selfplay-depth-ply3-cold). The bottom of RESULTS section 8
  explains why: every routing gain measured at T 0.1 was the calibration and temperature confound. The analyst
  recommends ply-3 cold arenas as the goal's own measurement. Pending the user: ply-2-only flat-leaf experiments rated
  cold, and possibly ply-3 play.
- No new binary since hybrid9 (11-10 19:22). New code: leagues.py self-games (twins replayed through the tournament
  binary), entry_ply, scaled plateau; specs/ply1/train_binary.py (hybrid6 training); dashboard.py training-count
  columns; specs/arenas/make_cold_read.py.
- The last review's first item for this one (TAN-router training, for cluster-error-new) is moot: batch 4 is ended and
  routing is likely to be parked. Dropped unless a TAN-router line is queued again.

**Measured, from records.**
- probit-widen-p1b, last 20 cycles, CPU-s a cycle (ply 1, 8 probit lines):

  | games | tournament process | training | own calibration | self-games process (games 0.22) | wall |
  |---|---|---|---|---|---|
  | 2.2 | 5.3 | 5.5 | 6.8 | 1.3 | 11.2 |

  At ply 1, work outside games is most of the cycle. The tournament process spends 3.1 s besides games: loading 16
  players with their own-games fits (~0.07 s each for a probit tree, measured), plus reports. The self-games process
  spends 1.1 s besides games.
- Arenas since 00:00: 2,733 CPU-s of games, plus ~416 CPU-s remaking own-calibration games (13%). In the 15-round
  diag arenas that's 18-35% (65% where entries calibrate at depth 2). Each run remakes them and deletes them after
  (arena.py drop_own_games), and the same model often enters several arenas: lr-settings-new's F-L 0.0125@136 was in
  5 tonight.
- Ply-3 arenas: 1,269 of the 2,733 CPU-s (46%).
- diag/ply3-cold game CPU by pairing (tour/game_cpu.csv joined to games.csv), µs a ply:

  | Pairing | µs a ply |
  |---|---|
  | LR-2 against LR-2 | 64 |
  | LR-3 against LR-3 | 447 |
  | LR-3 against probit-3 | 680 |
  | probit-3 against random | 704 (a probit-3 move ~1.4 ms) |
  | FM-3 against random | 4,204 (an FM-3 move ~8.4 ms, float code) |

  FM-3's pairings took 52% of that arena's game CPU, for one entry of eight.
- Old champion dashboard (pid 893240): 2.9 CPU-h over 46 h, all while the champion run ran; idle now (0 ticks in
  20 s). Leagues dashboard: ~1% of a core with a page open.

**Profiles** (scratchpad perf3/; nice 19, 1 thread; gdb sampling, since perf is blocked).
- Flat LR self-play at depth 3 (lr-settings-new F-L 0.0125@136, T 0.02), 49 busy samples:

  | Where | Samples |
  |---|---|
  | child_fx leaf work (inlined eval, repetition) | 16 |
  | Acc::child_sums and Acc::child | 14 |
  | has_legal_move | 6 |
  | move generation | 6 |
  | play_unchecked | 4 |
  | hashing | 2 |

  The cost is per leaf, so the lever is fewer leaves.
- At depth 3, root_values_fx_full gives every root move a full window, so each root value is exact. Inside,
  negamax_fx orders moves by static evaluation only at depth ≥ 2. Depth-1 nodes take moves in generation order, so a
  reply's refutation is often found late.
- Instrumented copy (search.rs: a node counter, and an env switch that partitions captures and promotions to the front
  in place, ordered by MVV-LVA among themselves, at depth-1 nodes), 60 depth-3 games, seed 1:
  - nodes 41.2M → 19.1M (-54%);
  - all game files byte-identical to chessai-hybrid9 (black_wins, draws, white_wins, human), with and without the
    switch;
  - CPU in alternating pairs, hybrid9 against the copy with the switch: 3.66/2.85, 3.69/2.60, 2.57/1.98, a ratio of
    0.70-0.78 for the whole process (~0.35 s of it is the held-out fit).
  - A first version that sorted a heap Vec at every depth-1 node was slower (7.8 s against 5.8): the ordering must
    stay in place.
  - Depth 4, 10 games: 2.65 → 2.52 s (×0.95), games identical. At depth 4 the static-evaluation keys at depth-2 and
    depth-3 nodes seem to dominate; not profiled further.

**Ranked candidates.**
1. **Ply-3 search: captures-first ordering at depth-1 nodes** (negamax_fx and the float negamax), with Acc::child_sums
   instead of Acc::child for the depth ≥ 2 ordering keys (eval_fx reads only the sums, so the keys are the same).
   - Current cost: depth-3 moves are most of the ply-3 arenas' game CPU, 1,269 CPU-s tonight, 46% of all arena CPU.
     Ply-3 cold arenas are the analyst's recommended measurement, and ply-3 play may follow.
   - Estimated saving: 22-30% of LR and probit depth-3 move CPU (measured on LR), likely more on the float path
     (trees, FM), where a leaf costs more. Tonight that would have been ~0.1 CPU-h, ~10% of the arena workload. It
     grows with every ply-3 arena or ply-3 league: one ply-3 league would cost ~7× a ply-2 league in games.
   - Exactness: alpha-beta returns the same exact root values in any move order; only the nodes visited change.
     - root_values uses only the values. Restrict the reorder to OUTCOME = false, because root_outcomes takes a PV
       leaf's outcome, and a tie between equal values could pick another leaf and change self-play's Score policy
       games.
     - Identity: depth-3 tournaments byte-identical (games.csv and game files) for F-L, F-P, an LR-leaf tree, a
       probit-leaf tree and FM; self-play at depths 2-4 byte-identical (unchanged path); depth-2 play is unaffected
       (no depth-1 node there searches under a cutoff window).
   - Risk: low. A new source dir and a new binary (chessai-hybrid10).
2. **Arena own-games reuse** (leagues/arena.py, Python only): keep own-calibration games in a shared content-addressed
   cache, leagues/arenas/.owncal-cache/<key>/. Key: model md5, kind, calibration settings, held-out files' md5,
   owncal binary md5. Reuse instead of remaking; copy, or hard-link read-only.
   - Current cost: ~416 CPU-s tonight (13% of arena CPU; 18-35% of the 15-round diag arenas).
   - Estimated saving: most of that for repeated entries and extensions, ~0.05-0.1 CPU-h a night of arenas like
     tonight's.
   - Exactness: the games are a deterministic function of the key. Prove it by remaking a sample (LR, probit, tree,
     FM; depth 1 and depth 2) and comparing bytes. Keep arena.py's existing calibration-line check.
   - Risk: low. Disk is 10-15 MB an entry; cap the cache (e.g. 20 GB, oldest first). Above the 5% bar for the arena
     workload, below 1 CPU-h.
3. **Calibration-fit cache in the tournament and self-games processes** (a champion's own-games fit is redone every
   cycle).
   - Current cost: ~0.07 s per player load (probit tree, 75k positions, measured). About 0.5 s a cycle in the
     tournament and 0.5 s in self-games (twins).
   - Estimated saving: ~5% of a ply-1 probit cycle, 2-3% of a ply-2 cycle.
   - Exactness: store the fitted parameters bit-exactly (hex floats), keyed by own-games md5, model md5 and the
     binary. Or, for self-games, share the entry's loaded player with its twin, as hybrid9's arena does for A = B.
   - Risk: low-moderate (a cache file). Borderline: do it only if ply-1 experiments resume.

**Checked and dropped, or left.**
- FM float play (52% of ply3-cold's game CPU): FM rated 337 below the flat LR at ply 3 and has no queued use. Ledger
  rule unchanged: only if FM returns. Candidate 1 also covers FM's float negamax.
- has_legal_move at leaves (12% at depth 3): skipping it when neither eval nor the stalemate value 0 can reach alpha
  is possible, but the proof is fiddly and the gain small. Left.
- Held-out set parse cache (last review's candidate 3): still ~0.1 s a version; below the bar.
- The probit-widen-p1b phases: the run has ended; nothing queued at ply 1.
- Dashboards: negligible now.

**Outcome (11-11 ~01:35, the implementation agent).** Candidates 1 and 2 are done (Done table:
chessai-hybrid10, arena own-games cache). Candidate 1 measured ×0.79 for LR at ply 3, as estimated, and more
on probit (×0.62), trees (×0.68-0.78) and FM (×0.59). Candidate 3 stays left (ply-1 experiments only).

### 2026-11-11 ~06:50 review (scheduled)

**What runs and what changed.**
- The night's experiments ended by 05:20. At 06:51 the coordinator started flat-p3-w4-w3f1 (theory-2 6.21, the
  reverse rung: W4's finals warm-started into a ply-3 field, forget 1, ~1 CPU-h). Its spec names chessai-hybrid10, to
  copy flat-p3-w3f1 exactly. hybrid11 plays the same games, x0.88 at ply 3 for flat LR.
- No new binary since hybrid11 (03:46). Queued or likely next (NEXT-EXPERIMENTS item 5):
  - train-temp-p2 (ply 2, a T 0.02 arm against a T 0.1 twin, ~2.2 CPU-h);
  - a warm ply-4 rung from W4 (4-5 CPU-h, T 0.1);
  - or a ply-5 rung (30-40 CPU-h) or a from-scratch ply-4 field (~24 CPU-h);
  - T 0.02 arena reads at plies 2-5.
- All of it is flat LR in the values-only fixed-point search (root_values_fx_full, log-odds softmax).

**Measured, from records** (totals.csv; arena tour/game_cpu.csv joined to games.csv):

| Ply (runs) | Games: tournament + self-games | Own calibration | Training | Tournament process besides games | CPU-h a run |
|---|---|---|---|---|---|
| 2 (flat-p2, flat-p2-b) | 51% + 12% | 21% | 6.5% | 9% | 0.98, 1.24 |
| 3 (flat-p3, flat-p3-b) | 80% + 12% | 4.6% | 1.4% | 2% | 3.24, 3.29 |
| 3, warm rungs (W3, ladder, W3f1) | 73% + 13% | 11% | 1.6% | 3.5% | 0.58-0.71 |
| 4 (flat-p3-w4) | 86% + 12% | 1.4% | 0.2% | 0.4% | 4.05 |

- Game CPU a ply (hybrid11, flat LR against flat LR, T 0.02 reads):
  - ply 4: 2.24 ms (diag/warm-reads-p4);
  - ply 5: 23.6 ms, x10.5 a ply (diag/parity-p5), 2.7 CPU-s a game.
  So a ply-5 rung of W4's size (38k games) is ~29 CPU-h, ~99% search.
- Own calibration: 0.5-0.76 CPU-s a version in the runs. Isolated on an E-core it is 0.46 s: 0.15 fixed (load, held-out
  fit) and 0.31 for the 600 depth-1 games. At ply 2 it is 21% of the run.
- Wall time: the driver's 30-s wait on its own training sibling ("waiting for pid ..., still working in this league")
  fired 8 and 15 times in the ply-2 runs (12-18% of their wall time) and 0-6 times in ply-3/4 runs (1-5%). Wall time
  only.

**Profiles** (scratchpad perf4, nice 19, 1 thread, eu-stack sampling; W4's F-L v99 against itself, own calibration
remade, T 0.1, hybrid11):
- Ply 5 (190 samples):
  - 61% at depth-1 nodes: legal move generation 26%, captures_first 22% (15% self, the rest its sort), Acc::child 8%;
  - leaves 15%; depth-2 nodes 17%.
- Ply 4 (219 samples):
  - leaves 40%: has_legal_move 14%, child_fx's clone, play and hash 16%, move_sums and the value ~15%;
  - depth-1 nodes 30% (generation 15%, captures_first 6%);
  - depth-2 keys and sorts 29%.
- Counters (scratch build, CHESSAI_D1STATS): depth-1 nodes with a cut window are 25% of all nodes at plies 3 and 5
  (6% at ply 4). 91% of them cut on their first move, and 85-88% (70% at ply 4) on a capture.
  - At odd plies, every root move gets a full window, so the depth-1 nodes are CUT nodes. Each pays a full legal move
    generation and captures_first to search about one leaf.

**Ranked candidates** (prototypes in fastbin/review-1111: proto-measurement.diff, README.txt; env switches, hybrid11
otherwise).
1. **Staged first move at depth-1 cut nodes** (negamax_fx, values only, beta < inf, not in check, no pawn about to
   promote).
   - The idea:
     - Find a legal capture with the smallest MVV-LVA key from the bitboards. Pins allow a capture along the ray;
       the king may capture only onto an unattacked square.
     - Search it. Generate all legal moves only if it doesn't cut.
     - Otherwise (in check, no capture, a promotion possible), one scan finds captures_first's first move, and the
       full partition and sort happen only without a cut.
   - Measured (ABBA, pinned to E-cores, 1 thread): ply 3 x0.71, ply 4 x0.95, ply 5 x0.70.
     - Lazy ordering alone: x0.86-0.92 at plies 3 and 5, x1.00 at ply 4.
   - Current cost: ply-3 and ply-5 games are 86-99% of their runs and reads. Estimated saving:
     - ~25% of any ply-3 run or read;
     - 9-12 CPU-h of a ply-5 rung;
     - ~30% of ply-3 and ply-5 arena games;
     - ~0.2 CPU-h of the warm ply-4 rung (x0.95).
   - Exactness: alpha-beta gives the same root values in any move order. A legal capture exists, so there is no mate
     or stalemate, and the draw checks run in the same order.
     - Ties between equal keys may pick another first capture, so node counts can change (as hybrid10's did). The
       games can't.
     - Checked: games byte-identical to the installed hybrid11 for L at plies 3-5 (T 0.1 and 0.02), probit and NB
       at ply 3, probit against L at ply 4, and NB against L at ply 5.
     - Check mode (CHESSAI_FX_CHECK=1, each staged move asserted legal) at plies 3 and 5: the same games.
     - To add: a unit test comparing best_capture with legal_moves() filtered to captures (smallest key, legality)
       on random games, and the hybrid11 suite.
   - Risk: low-moderate. The capture finder must match shakmaty's legality, so promotions and en passant are left to
     the full generator.
2. **Root cut windows for the log-odds softmax at low temperature** (root_values_fx_full and softmax_sample).
   - The idea:
     - The first root move, by static key, gets a full window.
     - Every other move gets (best so far - 40T, inf). Above the bound its value is exact, the same bits.
     - Below, only an upper bound is known, and its weight is under e^-40.
     - The sample is exact when the weight sum is bit-equal with the pruned weights at 0 and at their bounds, and at
       each pruned step before the pick, target >= bound and target - bound == target.
     - Otherwise, re-search the pruned moves with a full window, before the draw if the sum differs, and finish the
       walk exactly. The RNG draws the same.
   - Measured (in-process A/B, same positions):

     | T | Root moves pruned | Time (with fallbacks) | Checks passing |
     |---|---|---|---|
     | 0.02, plies 2, 4, 5 | 34-36% | x0.77, x0.75-0.77, x0.70 | 99.0-99.5% |
     | 0.1, plies 2, 4 | 3-4% | x0.98-1.01 | |

     0 disagreements with the full values' pick. Unpruned values bit-equal.
   - Current cost: T 0.02 reads (the next read at plies 2-5: 0.5-7 CPU-h by size) and train-temp-p2's cold arm.
     - Estimated saving: 25-33% of their game CPU, ~0.3-2 CPU-h.
     - If training moves to T 0.02 (the rating-policy decision), the same share of every rung.
     - Nothing at T 0.1.
   - Exactness: monotone float sums. exp's bound needs a margin (libm isn't guaranteed monotone), e.g. 2x the bound.
     rand's random_range(0..S) sees the same S bits.
   - Risk: moderate (a new sampling path). A check mode can run both and compare.
3. **Lazy leaf terminal checks at even plies** (not prototyped).
   - The idea: at a leaf whose static value fails low (x <= alpha) with alpha >= 0, only a mate could change the
     decision. Mate needs check. So a gives-check test from the move can replace the clone, play, has_legal_move and
     draw checks.
   - Exactness: a fail-soft bound on the same side of the window changes no alpha, no cutoff and no root value.
   - Estimated ~10% at ply 4 (leaves 40%, has_legal_move 14%); less at odd plies.
   - The proof is fiddly. Prototype only if the warm ply-4 rung or a ply-4 field is approved.

**No-build items for the coordinator:**
- train-temp-p2's plan names chessai-hybrid9. Name chessai-hybrid11: the same games, ply-2 LR x0.845, ~10% of the run.
- New specs train with hybrid4gfx. specs/ply1/train_binary.py gives chessai-hybrid6: the same models, x0.6 of
  training (2.6% at ply 2, <1% at plies 3-4).
- flat-p3-w4-w3f1 could swap to hybrid11 between cycles (BUILD.md): ~0.1 CPU-h.

**Checked and dropped, or left:**
- Root windows at T 0.1: x0.98-1.01, since only 3-4% of root moves lie 4 nats below the best.
- Own calibration at ply 2 (21%): the held-out parse cache stays ~3% of a ply-2 run. The run-vs-isolated gap
  (0.5-0.76 against 0.46 s a version) looks like 2-job contention. Left.
- The driver's self-wait: wall only. A busy() check that excludes the cycle's own children would be exact. Left for a
  ply-2 batch.
- Acc::child at depth-1 nodes (8% at ply 5; more after candidate 1): an accumulator update from the move would halve
  it. Next review.
- The capture extension (theory-2 6.20) is a search change, not a speedup, and isn't costed here. Counters show 82-95%
  of depth-1 cut nodes have a capture, so it would extend most horizon nodes. best_capture is a building block.

**Outcome (11-11 ~07:40, the hybrid12 implementation fork).** Candidate 1 is done (Done table: chessai-hybrid12,
installed 07:34). Measured on W4 as estimated: ply 3 ×0.650, ply 4 ×0.952, ply 5 ×0.711, games byte-identical.
Its lazy first move (the in-check, no-capture and promotion cases) is included. Candidates 2 and 3 are not started.

**Outcome (11-11 ~08:15, the hybrid13 implementation fork).** Candidate 2 is done (Done table: chessai-hybrid13,
installed 08:11), with K 40 (the fastest of 38-45 on W4) and the argmax too. Measured as estimated: flat LR at T 0.02
×0.72-0.84 at plies 2-5; probit and trees gain past ply 2. The estimate's "99.0-99.5% of checks passing" became 98.5%
of picks decided by the bounds; the rest are searched again exactly. NB is left on hybrid12's path: it cuts too few
moves to pay for the ordering keys (ply 2 ×1.15). Candidate 3 is not started.

### 2026-11-11 ~12:50 review (scheduled)

**What runs and what changed.**
- Running: error-dag-p1 (from 12:12, cycle ~90; tournament and the DAG lines' training on chessai-errdag2) and
  error-dag-probit-p1 (from 11:20, cycle ~195; errdag3). 8 lines, 16 entries, ply 1, floor 300, cap 600.
  probit-late-p2 ended at 12:48 (plateau, cycle 400). error-growth-p1 ended at 12:12.
- New since 06:50: hybrid12, hybrid13, routecells, and chessai-errdag/errdag2/errdag3 (fastbin/errdag-src).
  errdag-src is bddctx-src (hybrid4gfx plus path context) plus the errdag patches. **It has none of hybrid5-13:**
  no fixed-point tree play, no hybrid6 training, hybrid7 reports, hybrid8/9 fast probit, single held-out load
  (hybrid9) or hybrid10-13 search.
- Queued: the global league at ply 2 (six models untrained, 12 entries plus 4 random references, ~1,200 games a
  cycle, indefinitely; errdag3 plays every game); ply-2 experiments for the six; the two DAG cold reads
  (dag_read.sh waiters, errdag2/errdag3, plies 1-2, 75 round-robins).

**Measured, from records** (totals.csv over the last 20 cycles; game_cpu.csv over the last 240 rounds).

| Experiment | CPU-s a cycle | Training | Own calibration | Tournament (games) | Self-games | Wall |
|---|---|---|---|---|---|---|
| error-dag-p1 (errdag2) | 112 | 69.5 (62%) | 24.9 (22%) | 14.4 (9.5) | 3.3 | 57.6 s |
| error-dag-probit-p1 (errdag3) | 89 | 48.7 (55%) | 23.4 (26%) | 14.1 (8.9) | 2.8 | 48.3 s |
| error-routing-p1, error-growth-p1 (hybrid13, 12 entries) | 17-20 | 4.5-5.8 | 6.7-7.2 | 4.3-5.0 (1.6-1.8) | 1.3-1.5 | 12 s |

- Training by line: DAG lines carrying path cells (router splits made) cost 22-30 s a cycle (LR z0), 10-17 s
  (probit z0) and 11 s (probit z30 #2, since ~cycle 110). DAG lines without router splits cost 2-3 s, the trees on
  hybrid6 2-3.3 s, the flats 0.3-0.5 s. So the cell-carrying lines are ~80% of training and ~45% of each run.
- Own calibration: 2.9-3.0 s a version on errdag, against 1.0-1.2 s on hybrid13.
- Game CPU a ply at ply 1, errdag3 against hybrid13 (other runs): T-P-Ne vs T-P-Ne 23.9 against 11.1-12.9 µs;
  F-P vs T-P-Ne 14.5 against 8.3-9.3 µs. Cell-carrying DAG pairs cost 37-52 µs.
- CPU left to the floor (cap): error-dag-p1 211 (511) cycles, 6.6 (15.9) CPU-h; error-dag-probit-p1 106 (406)
  cycles, 2.6 (10.0) CPU-h.
- The driver's self-wait fired 6 and 9 times (~4% of wall). Wall only; left as before.

**Profiles** (review-1111b: README.txt, proto.diff; E-cores, nice 19, 1 thread).
- DAG training, errdag2, LR z0 line v63 on its live trainset (415,663 positions): 45.1 s.
  - Cnbp::predict_counts_scaled 60%, Logreg::predict_log 8%, memset 5% ([0.0; 256] buffers).
  - eu-stack: 83% under routing_errors_ctx. For a router whose children share grandchildren, each of 3 children ×
    3 shared grandchildren gets a subtree prediction with its own path's cell values. node_output_ctx's
    single-entry cache (step, values) then misses: every router and leaf below is evaluated again for each path.
  - Within a step, the expensive part is the same for every path: a CNBP's class products before the cells'
    factors, an LR's raw log-scores, a probit's latent mean and variance. The cells enter only after it.
- DAG play, errdag3, D-P-Ne-ctx z0 against itself at ply 2: Cnbp::predict_scaled 36%, Cnbp::predict 22%, cnbp_factors
  6%, memset 6%. These are float 64-term product chains, one class after another.
- errdag3 against hybrid13 on identical games (ABBA, playcmp.py), hybrid13's CPU as a share of errdag3's:

  | Pairing | Ply 1 (600 games) | Ply 2 (100-150 games) |
  |---|---|---|
  | F-P vs T-P-Ne | ×0.69 | ×0.41 (×0.40 at T 0.02) |
  | T-P-Ne vs T-P-Ne | ×0.50 | ×0.30 |
  | F-L vs T-L-Ne | ×0.60 | ×0.30 |
  | F-P vs F-P | | ×0.67 |
  | F-L vs F-L | ×0.81 | ×0.83 |
  | T-P-Nc-merge vs F-P | | ×0.54 |
  | T-L-Nc-merge vs F-L | | ×0.32 |

  - The merge-tree rows include held-out calibration fits for both binaries.
  - errdag3 loads and plays the merge trees exactly as hybrid13.
- Own calibration (600 depth-1 games plus the held-out fit), errdag3 against hybrid13:
  - T-P-Ne: 2.20 against 1.09 s.
  - F-P: 0.98 against 0.72 s.
  - D-P-Ne-ctx z0: 4.55 s on errdag3 only. 1.00 s of it is fixed: the held-out calibration is loaded twice.
- Ply-2 game cost on errdag3 (ms a game): T-P-Ne vs T-P-Ne 100 (30 on hybrid13); D-P z0 vs T-P-Ne 164; D-P z0 vs
  itself 248; D-L z0 vs T-L-Ne 171.
  - A cell-carrying DAG's half of a game is ~115-125 ms; a probit tree's is ~50 on errdag3 and ~15 on hybrid13.
  - Scaled up, an 8-line field shaped like error-dag-probit-p1 at ply 2 would spend ~350 CPU-s a cycle on games
    (rough). That is ~15× probit-confirm-p2's 23 s, or ~30 CPU-h over 300 cycles.

**Prototypes** (proto.diff, env switches; errdag3 otherwise).
- CHESSAI_BASE_CACHE=1, the path-independent part cached per node and step:
  - LR z0: 45.1 → 17.2-19.4 s (×0.38-0.43).
  - Probit z0 (v164, 326,003 positions): 25.7 → 11.75 s (×0.46).
  - Models and states byte-identical to errdag2's and errdag3's.
- With CHESSAI_BASE_ILP=1 as well (the three class chains in one pass, each in its own order): LR z0 15.8 s
  (×0.35), byte-identical.
- The same interleave in Cnbp::predict and predict_scaled (play), identical games:
  - D-P-Ne-ctx z0 against itself at ply 2: ×0.76.
  - T-P-Ne against itself at ply 2: ×0.76.

**Ranked candidates.**
1. **errdag4, training: cache the path-independent part of each node's output per step** (errdag-src: delegation.rs
   node_output_ctx, cnbp.rs class_products, probit.rs latent). Also run the class chains in one pass and size the
   buffers to the classes instead of [0.0; 256].
   - Saving: cell-carrying DAG training ×0.35 (LR) and ×0.46 (probit, before the interleave).
     - ~32% of error-dag-p1 (36 of 112 s a cycle) and ~25% of error-dag-probit-p1 (21 of 89 s).
     - **2.7 CPU-h to the floors, 7.7 to the caps.**
     - In the global league: ~20 s a cycle once its DAGs carry cells.
   - Exactness:
     - The cached part is computed by the same code from the same inputs.
     - It is dropped at every point the existing cache is (cache_step = 0: 14 sites). Nodes change only there.
     - The cells are applied after it by the same operations in the same order.
   - Identity checks, done: model and state byte-identical on two live lines' trainsets.
   - Identity checks, to add:
     - multi-round training with leaf and router splits (the errdag sweeps' setup: 30 fixed rounds, 100k windows,
       z 0/5/30, LR and probit, with and without path context);
     - the 171 tests;
     - a 3-cycle dry run byte-identical to errdag3's.
   - Risk: low.
2. **errdag4, play (the same binary)**: CNBP router predictions with the class chains in one pass and right-sized
   buffers. Optionally hybrid9's single held-out load for own calibration (A = B), worth ~0.4 s a DAG version.
   - Saving: ×0.76 on tree and DAG play on errdag (tournaments, own-calibration games, self-games, arenas).
     - ~5% of the running pair (0.5-1.4 CPU-h).
     - ~0.1 CPU-h of each queued DAG cold read.
     - ~23% of the global league's game CPU on errdag3, and as much of any ply-2 DAG experiment's.
   - Exactness: each class's product keeps its order. Identical games checked. To add: errdag4 against errdag3 and
     errdag2 on every kind with ctx DAGs, plies 1-2, T 0.1 and 0.02.
   - Risk: low.
3. **hybrid14: errdag4's model code in hybrid13-src** (path context, BDD probit leaves and cells, the router-split
   knobs). Non-ctx kinds get hybrid5-13; cell-less ctx DAGs may take the fixed-point path.
   - Measured: ×0.30-0.54 (trees at ply 2) and ×0.67/×0.83 (F-P/F-L); own calibration ×0.50-0.73.
   - Saving:
     - ~8% of the running pair (a swap needs byte-identical reports as well);
     - ~20% of the global league's games at growth;
     - ~16% of a ply-2 DAG experiment's games.
   - Risk: moderate-high. It is a large port. The identity matrix is every kind × plies 1-2 × T 0.1 and 0.02,
     against hybrid13 and errdag4.
4. **Fixed-point play for cell-carrying DAGs** (on 3): hybrid5's fixed-point DAG path plus per-(node, cell values)
   offsets (CNBP ln factors, LR offsets, probit leaves through hybrid9's table on the shifted latent), with the float
   fallback inside the bound.
   - A DAG's half of a ply-2 game is ~8× a tree's on hybrid13. With 3, this would cut a ply-2 DAG field's game CPU
     by about two thirds (~350 → ~110 s a cycle, rough).
   - Only once ply-2 DAG work is approved. Risk: moderate.

**Global league at ply 2, rough, at full growth.**
- On errdag3: ~165 CPU-s a cycle (games ~110, the two DAGs' training ~32, own calibration ~15, reports ~5), ~46
  CPU-h per 1,000 cycles. Early cycles cost far less: untrained models are single leaves.
- With 1 and 2: ~117 s. With 3 as well: ~95 s. With 4 as well: ~60 s.

**Checked and dropped, or left:**
- probit-late-p2 is over, so there is nothing to swap there.
- The report step on errdag (4.5-4.8 s a cycle with 16 entries, against ~3 on hybrid13): ~1.4% alone. It goes with 3.
- Self-games (3%) gain from 2 automatically.
- The driver's self-wait (~4% of wall): wall only, as before.
- Training of DAG lines without cells: 3.0 s on errdag2 against T-L-Ne's 2.6 on hybrid6. That is hybrid6's ~15%,
  which goes with 3.

- 2026-11-11 13:22 (coordinator): implementation fork started for candidates 1 and 2 as one new binary,
  chessai-errdag4 (DAG training in one pass per node; router class products in one pass in play). Swap into
  error-dag-probit-p1 and error-dag-p1 (tournament and DAG lines' training) only after byte-identity against errdag3
  (probit), errdag2 (LR DAG training) and hybrid13 (play). Candidates 3 (hybrid14) and 4 (fixed-point DAG play) wait
  for the user's call on ply-2 DAG fields.

### 2026-11-11 ~14:15 chessai-errdag4 (implementation fork; candidates 1 and 2 of the ~12:50 review)

- **Build.** fastbin/errdag4-src = a copy of fastbin/errdag-src (chessai-errdag3, md5 261fe1991fab361bc5d75eb428660457)
  plus fastbin/errdag4/errdag4.patch (src/ only: cnbp.rs class_products/scale_normalise/probs_products3, probit.rs
  latent_base/predict_from_base, delegation.rs Node.base/base_step and node_output_ctx, small_buffer; bdd.rs and
  pathctx.rs: base_step reset beside cache_step). Installed as **models/from-scratch/bin/chessai-errdag4, md5
  7d6ce9a7bbe53d58eed1652bbe6ce145** (new file; errdag2 2f963aa0215a9e85bb3d0b9ee07599a0 and errdag3 untouched).
- **Identity** (fastbin/errdag4/*.txt):
  - cargo test --release: 171 passed.
  - train_id.txt: 30-round training with leaf and router splits (sweep data, 100k windows), seeds 11 and 12, LR and
    probit, bdt / ctx z0, z5, z30, inf / bdd z0 / flat: models, states and logs SAME against errdag3, and against
    errdag2 for every LR config. Splits happened (sibling sets 3-4, cells 36-66).
  - live_train_id.txt: one round from the ~12:50 live states of D-L z0 ×2, D-P z0 ×2, T-L-Ne, T-P-Ne and two
    cluster-merge trees: SAME against errdag2/errdag3/hybrid6.
  - play_id.txt / play_id_h13.txt: arenas at depths 1-2, T 0.1 and 0.02, for D-P z0, D-L z0, T-P-Ne, T-L-Ne, F-P,
    F-L and the merge trees, and the leagues' own-calibration command: SAME against errdag3, errdag2 (LR-only pairs)
    and hybrid13 (non-DAG pairs).
  - Dry runs of both live experiments (dry/: a mid-cycle reflink snapshot with paths rewritten, then 2 cycles with
    the old binaries and with errdag4): every model, state, rating, Elo history, head-to-head, games.csv and
    self_games.csv byte-identical. The second cycle's 12,291 game files were captured before the engine's cleanup and
    were identical too. Only timings and the binary's name differ.
- **Swap**, without stopping the drivers (swap_errdag4.py). The engine reads config.json once per cycle process and
  `run` never writes it, so config.json was os.replace()d just after a cycle entered its tournament phase. Backups
  are in fastbin/errdag4/config-before/, with one line in each driver.log.
  - error-dag-probit-p1: 13:58:30 in cycle 247; errdag4 from cycle 248.
  - error-dag-p1: 14:01:27 in cycle 139; errdag4 from cycle 140.
  - The logs show errdag4 for every tournament, own calibration, self-game and DAG-line training from those cycles.
  - The dag_read.sh waiters were left alone (errdag3/errdag2 arguments).
- **Measured live**, 15 cycles before against 22 (probit) and 17 (LR) after, CPU-s a cycle:
  - probit: 90.3 → 56.4 (×0.62). DAG training 43.2 → 19.3, games 9.2 → 6.4, own calibration 24.1 → 18.1. Wall 49.7 →
    32.4 s.
  - LR: 102.3 → 64.6 (×0.63). DAG training 54.5 → 25.1, games 9.2 → 6.9, own calibration 24.1 → 18.7. Wall 52.1 →
    32.7 s.
  - That is ~1.8 CPU-h saved to both floors and ~7.8 to both caps.
- **Left:**
  - hybrid9's single held-out load in own calibration (optional);
  - ply 3+ and outcome search were not compared on errdag (nothing runs them);
  - candidates 3 (hybrid14: errdag4's model code in hybrid13-src) and 4 wait for the user's call on ply-2 DAG fields.

### 2026-11-11 ~16:35 chessai-hybrid14 and chessai-hybrid15 (implementation fork; candidates 3 and 4 of the ~12:50 review)

- **Why now.** The user wants the global league (8 models at ply 1: F-L, F-P, T-P-Nc-merge-fresh,
  T-L-Nc-merge-fresh, D-P-Ne-ctx at z 30 and z 20, D-L-Ne-ctx at z 160 and z 120) and the ply-2 fields launched
  only after these optimizations are in (coordinator, 16:20). The interim note on hybrid14 went to main at ~16:25.
- **hybrid14** (fastbin/hybrid14-src; hybrid14/hybrid14.patch against hybrid13-src: bdd, bdt, cnbp, compound,
  delegation, lib, logreg, main, pathctx (new), probit). Installed as **bin/chessai-hybrid14, md5
  3976e92d941fc125fb77ce332edb4f64**.
  - Identity: unit tests; train_id14.txt 36/36 SAME (errdag4's 30-round configs with leaf and router splits, LR and
    probit, ctx z0/z5/z30/inf, bdd, bdt, flat; trees and flats against hybrid6 too); live_train_id14.txt SAME;
    version 0 of D-P and D-L SAME; play_id14.txt 161/161 SAME (15 own calibrations; 23 pairs at depths 1-3 and T 0.1
    and 0.02; 4 mixed-depth pairs; errdag4 the reference for any pairing with a DAG, hybrid13 otherwise); dry14.txt:
    both DAG experiments, 2 cycles from errdag4's snapshots, 0 key files differ (3,724 and 2,076), the second
    cycle's 12,291 game files identical.
  - CPU (cpu14.txt): trees and flats ×0.93-0.99 of hybrid13; cell-less DAGs ×0.74/×0.44 (D-P z30, plies 1/2) and
    ×0.63/×0.32 (D-L z160) of errdag4; DAGs with cells ×0.89 (ply 1), ×0.63-0.73 (ply 2), self ×0.85.
  - Finding: a ctx DAG without router splits is the T-*-Ne tree of its recipe and seed: identical own games and
    self-games (D-P z30#1 = T-P-Ne#1 @366, D-L z160#1 = T-L-Ne#1 @361), and on hybrid14 the same cost per position
    (11.9/137.9 against 11.9/137.5 us; 82.7 against 82.9). Train seeds don't depend on the recipe, so z 30 / z 20
    and z 160 / z 120 begin identical in the global league.
- **hybrid15** (fastbin/hybrid15-src; hybrid15/hybrid15.patch against hybrid14-src: compound.rs, delegation.rs).
  Installed as **bin/chessai-hybrid15, md5 58717c38118eebc7e0748a67b8d98c88**.
  - Identity: 196 unit tests (new: dags_with_cells_value_exactly, LR and probit leaves) and the compound tests with
    CHESSAI_FX_CHECK=1; FX_CHECK arenas on the real cell DAGs at depths 1-2 (every router decision asserted against
    node_predict_path; 1.35-3.06M decisions an arena at depth 2, 0 float fallbacks); play_id15.txt 161/161 SAME
    against hybrid14's outputs; dry15.txt: both DAG experiments' dry runs with hybrid15, 0 key files differ against
    errdag4's runs, 12,291 game files identical; train_id15.txt 36/36 SAME against errdag4 and hybrid6.
  - CPU (cpu15.txt, hybrid14 the base): DAGs with cells at ply 1 ×0.70-0.74 against trees, self ×0.52-0.54; at ply
    2 ×0.40-0.44 against trees, ×0.31 DAG against DAG, self ×0.27-0.31; against errdag4 at ply 2 ×0.25-0.32.
    Controls without cells ×0.95-1.00. Own calibration of a cell DAG 1.4-1.5 → 0.8 s (a tree's 0.6).
- **Swaps:** none. The global league and the ply-2 fields don't exist yet (specs/global/eight-p1.DRAFT.json still
  names errdag4/hybrid14). hybrid15 can be their tournament binary and the DAG lines' training binary from the
  start: its training is hybrid14's code, byte-identical against errdag4.
- **Left / not verified:**
  - Ply 3+ plays the float path for trees and DAGs (identical at depth 3 in both play_id runs); outcome search
    (wld-score self-play, search labels) not compared on DAGs.
  - compound's caps: graphs over 64 nodes (hybrid5's) or over 4,096 route states (hybrid15's) play in float, exact
    but at hybrid14's speed. The final cell DAGs of both experiments have 25 nodes and 445-607 route states
    (hybrid15/probe-src, a test-only copy): full size at split limit 4 and router limit 4, so the global league's
    DAGs stay inside. Nodes = 1 + 3 × (splits + router splits): with growth the goal, the 64-node cap binds past
    21 splits in all, and states grow faster than nodes.
  - errdag4's left item, hybrid9's single held-out load in own calibration (A = B): hybrid13-src's main.rs has it,
    and hybrid14/15 keep it (checked), so DAGs get it too.

### 2026-11-11 ~18:50 review (scheduled)

**What runs and what changed since ~13:20.**
- errdag4 went into error-dag-probit-p1 (13:58) and error-dag-p1 (14:01); both ended (15:02, 15:50). hybrid14 and
  hybrid15 were built at ~16:35.
- Running:
  - leagues/global since 16:40:57: ply 1, hybrid15 for games, own calibration, self-games and DAG training, hybrid6
    for flat and tree training. Cycle ~546 at 19:03, ~20 s wall a cycle. 8 lines until T-P-Nc-merge-fresh.p1
    retired at cycle 532 (patience), 7 since (18 players with the 4 random references).
  - base-probit-p2 (cycle 303) and base-lr-p2 (cycle 401) since 17:03: ply 2, 6 lines, 12 entries, floor 600, cap
    900, hybrid15.
  - The offline discriminator's ply-2 pools (models/from-scratch/offline-ply/, a hybrid15 tournament at nice 15),
    ~2 CPU-h.
- Queued: specs/ply2/limit-p2.DRAFT.json (F-P x2, T-P-Ne at split limits 1 and 4 x2; ply 2, floor 600, cap 900;
  ~6.7-10 CPU-h).

**Measured, from records** (cycle-NNN.md "Measured CPU", tour/tournament.log, cycles.csv; repeat.sh's cutime sampled
at global cycles 505 and 518).

Global league, CPU-s a cycle:

| Cycles | Tournament (games) | Training | Own calibration | Self-games | Driver | Total | Wall |
|---|---|---|---|---|---|---|---|
| 444-493 (8 lines) | 9.64 (5.62) | 7.04 | 9.95 | 2.04 | ~2.2 | ~30.9 | 19.9 s |
| 494-543 (8, then 7 lines) | 9.67 (5.56) | 6.61 | 9.71 | 1.99 | ~2.2 | ~30.2 | 20.5 s |

- Cutime over cycles 506-518 gives 30.79 s a cycle. The parts sum to 28.58, so the driver is 2.22 s.
- That is ~4,200-4,300 cycles and ~36-37 CPU-h a day with 8 lines (~33-35 with 7).
- Training by line, last 20 cycles:

  | Line | CPU-s a cycle |
  |---|---|
  | D-P-Ne-ctx-z20 | 3.14 (46%) |
  | T-L-Nc-merge-fresh | 0.91 |
  | D-P z30 | 0.65 |
  | T-P-Nc-merge-fresh (retired) | 0.63 |
  | D-L z160 and z120 | 0.55-0.56 each |
  | F-L and F-P | 0.11 each |

- Tournament process, on a reflink snapshot at cycle ~508 (5 tournaments, 16 entries):
  - 10.0-10.4 CPU-s in all: games ~5.6, and the 16 calibration fits at load ~1.9 (~0.12 s each).
  - A zero-game run costs 2.38 s (the fits, the history read and startup).
  - Report laps (CHESSAI_REPORT_TIMES=1): 5 sequential history prefix fits of ~0.11 s each, with the full fit in
    its own thread beside them; the tables take 0.13 s. The report is 0.75 s wall in all.
- Self-games (snapshot; 16 entries plus twins, 32 players):
  - 2.07 CPU-s with calibration and 0.23 raw.
  - The log has 16 "calibration on" lines, so twins share their entry's fit: 16 fits, 1.84 s.
- Own calibration of one version (snapshot):
  - bdt 1.35 s, bdd without cells 1.36, D-P z20 with cells 1.93.
  - bdt costs 0.25 s at 2 games (held-out fit and load) and 1.19 s at 600, so the 600 ply-1 games are ~85%.
- Growth, from linear fits:
  - Tournament: 0.25 s per 100 cycles over cycles 300-544 (0.14 over the last 100).
  - Driver: ~0.27, inferred from its run average of 1.52 s against 2.22 now.
  - Training: 0.18, noisy; this is the models' own growth.
  - Own calibration: flat.
  - All of it is whole-history work: the games.csv read; 5 prefix fits plus a full fit over every version ever
    entered (4,056 retired by cycle ~508); head-to-head over all games; contrast_ses over all players.
  - So ~0.4-0.5 s a cycle per 100 cycles: ~15-20 s a cycle more after each further day at today's pace, if linear.
- Disk:
  - runs/: 2.1 GB after 545 cycles (D-P z20 keeps a 0.86 MB model and a 0.46 MB state a version).
  - tour/: 323 MB (games.csv 52 MB). owncal/: 182 MB (live entries only).
  - ~4-5 MB a cycle, ~20 GB a day; 607 GB free.

Ply-2 runs, last 30 cycles, CPU-s a cycle (parts, without the driver):

| Run | Cycle | Tournament (games) | Training | Own calibration | Self-games | Parts | Left to floor / cap |
|---|---|---|---|---|---|---|---|
| base-probit-p2 | 303 | 31.9 (28.7) | 12.0 | 7.7 | 4.2 | 55.9 | 4.6 / 9.3 CPU-h |
| base-lr-p2 | 401 | 21.9 (18.6) | 5.2 | 6.0 | 3.0 | 36.1 | 2.0 / 5.0 CPU-h |

- base-probit-p2's games by pairing (review-1111b/pair_cpu.py): the z20 DAGs against the cluster trees 29%, the
  DAGs against F-P 25%. The DAG lines' training is ~9.6 s of the 12.0.
- Router splits: both probit z20 DAGs have them (header "pathcontext 1 4" at v310). base-lr-p2's z120 DAGs have none
  ("pathcontext 1 0" at v411).

**Ranked candidates.**
1. **Calibration-fit cache across processes** (hybrid15-src/src/main.rs, Calibrated::fit ~1215-1290). Every
   `cal:GAMES:KIND:MODEL` player in tournaments, replay-games and arenas runs it.
   - This is the ~00:50 review's candidate 3, left then "only if ply-1 experiments resume". Its condition is now
     met: the global league is ply 1 and runs without end. The fits also cost more now (~0.115-0.12 s each, against
     0.07).
   - Repeats:
     - The tournament refits the 7-8 champions every cycle, though their own games and model haven't changed since
       their first fit.
     - The self-games refit all 14-16 entries the tournament has just fitted.
     - So 21-24 of a cycle's 28-32 fits are repeats. Only the new versions' fits are new.
   - Cost now:
     - Global league: ~2.4-2.8 CPU-s a cycle, 8.5-9%. Less 0.1-0.35 s of hashing, that saves ~2.6-3.2 CPU-h a day.
     - Ply-2 fields: 18 of 24 fits, ~1.8 s a cycle: 3.2% of base-probit-p2, 5.0% of base-lr-p2. That is ~0.25-0.55
       CPU-h to their floors and caps together.
     - limit-p2: about the same, ~4.5%.
     - Arenas gain when an entry recurs.
   - Design:
     - Opt-in, by an env var such as CHESSAI_CAL_CACHE=<dir>; leagues.py sets it per league from config.json.
     - Key: a hash over:
       - a code-version id (a source hash at build time, or /proc/self/exe hashed once per process);
       - the spec's kind and calibration options (blend, leafcal);
       - the model file's bytes;
       - the labelled games files' bytes, in read order.
     - Value: the fitted state's f64 bits (the whole fit and the per-leaf fits), and the path-free parts of the
       "calibration on ..." line. On a hit, print the same line with the current path.
     - Writes go to a temp file, then rename. The file stores the full key, and a reader compares it.
     - A hit hashes ~5-10 MB of own games and ≤1 MB of model: a few ms with a fast hash, ~15 ms with md5.
   - Exactness: the fitted state is a deterministic function of the key's inputs. Prove it three ways:
     - A unit test: cold and cached Calibrated give identical bits for lr, probit, bdt, and bdd with and without
       cells.
     - The review-1111c/snap tournament (5 tournaments, 16 entries) and the sg/ self-games replay, with the cache
       off, cold and warm. games.csv, the game files, ratings, history, tables, and stdout except timings must be
       byte-identical to hybrid15.
     - An own-calibration arena (A = B), checked the same way.
   - Risk: low-moderate.
     - A cache directory outside the experiments gets ~8 new keys a cycle (~34,000 a day, a few KB each). Prune it
       with the owncal dirs, or keep each cache file beside its write-once owncal dir.
     - The global league and the ply-2 fields read and write at once; atomic renames make that safe.
   - **Clears the bar:** ≥5% of the global league, and ≥1 CPU-h within a day.
2. **Cell-DAG training: path-context lookups and CNBP cell factors** (hybrid15-src: delegation.rs
   Graph::node_output_ctx, pathctx.rs PathTable::cnbp_factors, cnbp.rs class_products).
   - Profile: review-1111c/train, one D-P-Ne-ctx-z20.p1 step from global cycle 528 (start.state plus the
     1,280-game set; 4.17 CPU-s; perf cycles:u).

     | Symbol | Share |
     |---|---|
     | class_products | 29.2% |
     | node_output_ctx | 17.2% |
     | cnbp_factors | 8.3% |
     | train_weighted | 6.1% |
     | memcmp | 5.3% |
     | predict_training_ctx | 4.4% |
     | probs_of_latent | 4.2% |
     | log | 3.6% |
     | malloc, free, memmove, finish_grow | ~6.3% |

   - Idea:
     - Intern each position's path marks to small integer ids once per step, so node_output_ctx's lookups don't
       hash and compare byte keys (the memcmp).
     - Memoize cnbp_factors per (node, marks) within a step.
     - Reuse the per-call buffers.
     - class_products is the per-position part that errdag4 already computes once per step; no exact cut seen.
   - Estimated saving: 15-25% of a cell DAG's training, unproven.
     - Global league: 0.5-0.8 s of D-P z20's 3.1 s a cycle; that is 1.6-2.6%, ~0.6-0.95 CPU-h a day.
     - base-probit-p2: 1.4-2.4 s a cycle; that is 2.6-4.3%, ~0.1-0.4 CPU-h of what's left.
     - That is below 5% for every workload now, but it grows with the DAGs.
   - Exactness: the memoized and interned values are the same f64s in the same order. Prove it with byte-identical
     models and states on review-1111c/train and on 5-10 consecutive live steps of the cell DAGs (global D-P z20,
     base-probit-p2's z20 DAGs).
   - Risk: low-moderate. The memo must be invalidated when a router split adds cells inside a step.
   - Borderline: prototype it first. It is worth doing if more cell-DAG lines are queued.
3. **The global league's whole-history work.** No exact fix of size; these are design options for the user.
   - What grows:
     - the tournament's games.csv read;
     - 5 prefix fits plus a full fit over every version;
     - head-to-head over all games;
     - the driver's contrast_ses over all players.
     - Together that is ~0.4-0.5 s a cycle per 100 cycles. At today's pace a cycle costs ~45-50 CPU-s after another
       day, and ~60-70 after two (linear extrapolation).
   - The exact pieces are small:
     - A prefix-hash-keyed games.csv parse cache, like the driver's scan cache: ~0.2-0.4 s now.
     - Taking the full fit's point estimates from the last prefix fit, only if the two compute the same thing (not
       checked).
     - An incrementally built tally: sparse_fit's FxHash map would iterate in a different order, so the map would
       have to be rebuilt in the original insertion order.
     - Parallel prefix fits save wall time only.
   - Options. They change history.csv or the ratings, so they're the user's call:
     - one history fit a cycle instead of five;
     - fits over the active entries plus the retired versions of the last N cycles, with older ones frozen as fixed
       anchors or dropped;
     - fewer old versions' model and state files on disk (champions and every Nth).

**Checked and dropped, or left.**
- Own calibration (32% of the global league):
  - ~85% of it is the 600 ply-1 games, already fixed-point (hybrid15).
  - The held-out fit is ~0.2 s a version. Every version is new, so a fit cache doesn't help there.
  - Fewer own-calibration games would be a design change.
- Ply-1 games (18%): fixed-point search since hybrid10-15; nothing new.
- The held-out set parse cache (~0.1 s a version): still below the bar.
- The driver's 2.2 s a cycle is mostly contrast_ses and the scans. The scan cache and contrast_ses are in the ledger
  already; their growth is item 3.
- data_cpu_s in the global league's cycles.csv attributes game CPU to the training sets (global_sets). It is not
  extra CPU.
- compound's caps (64 nodes, 4,096 route states): limit-p2's trees (split limits 1 and 4) and the global league's
  DAGs (split limit 4, router limit 4) stay inside.
- Scratch: models/from-scratch/fastbin/review-1111c/ (README.txt lists it). The ledger as it was before this section
  is kept there as PERF-LEDGER.md.before.

### 2026-11-11 ~20:10 chessai-hybrid16 (implementation fork; candidate 1 of the ~18:50 review)

- **What** (fastbin/hybrid16-src; hybrid16/hybrid16.patch against hybrid15-src: new src/calcache.rs, lib.rs, main.rs;
  Cargo files unchanged). Installed as **bin/chessai-hybrid16, md5 0cb3d238b0c12ac58894fa03b9e0f2d0**.
  - A fit of `cal:`/`leafcal:`/`blend:GAMES:KIND:PATH` (KIND cnbp, lr, probit, bdt or bdd: loaders that read just
    PATH) is kept in GAMES/calfit-<128-bit key>.bin when GAMES's parent directory is named `owncal`.
  - Key: XXH64 ×2 with lengths over the binary's bytes (/proc/self/exe, once a process), the CHESSAI_* environment,
    the wrapper and TAU, the inner kind, --use-probs, the model file and white_wins/black_wins/draws.games in read
    order. The file holds the whole key (compared in full) and a checksum. Value: positions, the whole and per-leaf
    calibrations as f64 bits, and the leaf lines; a hit prints and reports the same text.
  - Guards: the model file is hashed before and after it loads; on a miss the games are read in full again, must
    hash to the key and are parsed from those bytes (read_labelled's steps); the fit itself is hybrid15's code.
    Anything unexpected fits as before, silently. CHESSAI_CALCACHE=off (and any CHESSAI_*_STATS or *_CHECK) turns
    it off. Writes: unique temp file, then rename.
- **Why the own-games directory** (the directive: no driver restart): leagues.py's specs already name it, so only
  config.json's tournament binary changes; it is write-once and per version, and own_gc / arena.py's drop_own_games
  delete it with the version, cache included (no pruning; bounded by live entries); nothing else reads extra files
  there (read_labelled reads the outcome files, own_done and arena.py's shared cache handle GAME_FILES only). Held-out
  sets are never written. A cache beside the binary or the league would need pruning in step with own_gc.
- **Growth:** ~270 bytes a flat fit (~33 bytes and a line more per leaf for per-leaf fits), one per fitted own-games
  dir and binary: ~6-7 made and deleted per global cycle; steady state 12-14 files (~4 KB) a league.
- **Identity:**
  - 200 unit tests (hybrid15's 196; calcache: XXH64 reference vectors and streaming in any pieces, file round trip
    and rejection of every flipped byte and cut; main: cold and warm fits equal fresh fits bit for bit with the same
    report for cal, leafcal and blend over LR, probit, CNBP (both --use-probs), a BDT, a BDD and a path-context DAG;
    planted values are served, damaged/empty files and changed model or games are refitted, held-out dirs and
    nested or other kinds are never cached). Compound tests pass with CHESSAI_FX_CHECK=1. XXH64 checked against
    twox-hash 1.6.3 on 18,000 random inputs and own-games files (hashbench/).
  - Whole-cycle dry runs (dry16.py; snapshots by snap16.py: rsync --link-dest into a reflink pre-copy during a train
    phase, since the global league's 4-s train phase outlives a plain cp of its 28k files; consistent: the phase
    was unchanged after the copy). Each runs the snapshot cycle's training and one full cycle with the live engine,
    pinned to the E-cores: h15, h16 (cold), h16off (CHESSAI_CALCACHE=off), h16warm (h16's cache files seeded).
    Against h15 (cmp16.py):
    - global (cycle 648 snapshot): 22,085 files, 16,326 key files, 0 differ in all three; second cycle's 6,403 game
      files identical. Differences: scan-cache JSONs, totals.csv, and batch/games.csv's cpu_s column only.
    - base-probit-p2 (cycle 390 snapshot): 7,507 files, 4,740 key files, 0 differ in all three; 6,915 game files
      identical; the same three timing files differ.
  - Live check: the global league's cache file names for F-L@538, T-L@581 and D-L z120@638 equal the dry run's (the
    key is content-only).
- **Saving** (meas16.py on the snapshots: each entry's fit loaded as the league does, no games; 3 fresh copies,
  ABBA, pinned, nice 15):

  | Snapshot | Process | hybrid15 | hybrid16 cold | hybrid16 warm |
  |---|---|---|---|---|
  | global (14 entries) | tournament, no games | 2.44-2.53 | 2.67 | 0.83 |
  | global | self-games, one round | 1.85 | 2.00 | 0.15 |
  | base-probit-p2 (12 entries) | tournament, no games | 2.14 | | 0.49 |
  | base-probit-p2 | self-games, one round | 2.02 | 2.14 | 0.35 |

  - A hit saves ~0.12 s (ply 1) / ~0.14 s (ply 2); a miss costs ~0.01 s more.
  - A cycle in steady state: the tournament refits the champions (hits) and fits the new versions (misses); the
    self-games hit every entry. Global (6 lines now): ~2.3-2.5 CPU-s of ~26-30 (~8-9%), ~2.6 CPU-h a day at
    ~4,000 cycles of 21 s. base-probit-p2: ~2.4-2.7 of ~65 (~4%), ~0.15-0.4 CPU-h to its floor or cap.
  - Live, global: self-games 1.72-1.90 (cycles 717-719) → 0.24-0.27 (720-722); the tournament's non-game CPU 4.0-4.4
    → 4.41 (720, champions still cold) → 3.38-3.46 (721-722). base-probit-p2: self-games 4.95-5.07 → 2.94-3.14,
    tournament non-game 4.04-4.06 → 4.34 (411, cold) → 3.28 (412).
- **Swaps** (swap16.py, errdag4's between-cycles procedure: config.json replaced atomically during a cycle's
  tournament phase, backups in hybrid16/config-before/, one driver.log line):
  - leagues/global: 20:05:39, during cycle 719; hybrid16 from cycle 720.
  - base-probit-p2: 20:05:52, during cycle 410; hybrid16 from cycle 411 (≥190 cycles left: floor 600, cap 900).
  - Not base-lr-p2: cycle 548 at 20:05 with its last crowning at 538, so it may stop at 600 (~50 cycles); the
    directive's bar is ~100 cycles left. It saves ~1.5-2 s a cycle there; ~2 CPU-min in all.
  - Training binaries unchanged (hybrid16's training is hybrid15's code; nothing trains through calibrated specs).
- **Left / not verified:**
  - Arenas weren't run with hybrid16 (no arena was swapped). Their own-games dirs are owncal/<name>, so the cache
    applies there when an arena's binary is hybrid16; an own-calibration arena (A = B) fits on the held-out set and
    is never cached, by design.
  - Arena own-games cache: keys include the binary's md5, so arena entries from hybrid16 leagues miss entries cached
    under hybrid15 once.
  - If two processes fit one key at once, both fit and both rename the same bytes into place (not exercised live;
    the leagues fit sequentially).

### 2026-11-12 ~01:00 review (scheduled)

**Running and changed since ~18:50.**
- Running:
  - router-p1: ply 1, 10 lines trained by chessai-grow3, played by hybrid16. D-P-Ne-ctx-r4 with router rules C
    (z 1e9), Z (z 20), H (stall-h) and HG (stall-hg), two of each, plus F-P x2. Cycle 236 at 01:00; plateau stop
    from 300, cap 400.
  - The global league: 2 DAG lines (D-P z30, D-L z160), cycle ~2,650.
- Done tonight: grow-p1, grow-p1b, grow-p1c, replay-p2, base-probit-p2.
- Waiting: stage_read.sh router-p1 (versions 150 and final, plies 1 and 2 in one arena).
- Queued:
  - diversity-p2, approved ~01:05: F-P x2 + T-P-Ne x2 at ply 2 with 4 frozen references, 1,584 games a cycle,
    floor 300, cap 400. The driver key is being built.
  - The next global league: T-P-Ne-grow and T-L-Ne-grow on grow4, plus ctx DAGs with router-p1's rule.
  - The games-per-cycle standard.
- New code:
  - grow3 and grow4: training only (router-split rules, the leaf-growth rule).
  - Replay in leagues.py (replay-p2 only).
  - Per-ply reads (make_cross_read.py, cross_read.sh).

**Measured.** Sources: records; repeat.sh cutime; the driver's own CPU from /proc; one snapshot tournament; three
training steps under perf on a reflink snapshot (nice 19). Scratch: review-1112a/ (README.txt).

| Workload | CPU-s a cycle | Wall a cycle | Left | CPU-h left |
|---|---|---|---|---|
| global league | 9.27 (cutime, cycles 2564-2632) | 6.3 s | ~1,500 cycles (D-P z30's patience) | ~3.9 |
| router-p1 | 66-71 (cutime 224-236: 65.9; parts, last 10: 70.8) | 36 s | 65-165 cycles | 1.2-3.3 |
| diversity-p2 (queued) | ~36 (probit-late-p2's 14.5 ms a game x 1,584) | ~20 s | 300-400 cycles | 3-4 (analyst: 6-8) |
| router-p1 stage read (queued) | | | 1 arena, 44 players, 75 rounds | ~0.35 |

- Global league, CPU-s a cycle:
  - Tournament process 4.98. Of that: games 1.98 (280 a cycle); reports ~2.1, of which the 5 prefix fits take
    0.48/0.33/0.29/0.27/0.27 s, the full fit ~0.3 s in its thread, the tables 0.17 s; startup ~0.5.
  - Own calibration 2.23 (2 versions).
  - The driver's own CPU: 1.9, 21% of the league (sampled 1.76-2.09 over 8 cycles).
  - Training 0.08; self-games 0.06.
  - Games are 21% of the league. At today's pace it burns ~35 CPU-h a day.
- router-p1, CPU-s a cycle (last 10 cycles):
  - Training 46.3. The ctx-DAG lines take 44.8 of it: H 7.4 and 10.2, Z 7.3 and 8.1, HG 4.1 and 3.6, C 2.1 and 2.1.
    F-P takes 0.45 per line.
  - Own calibration 12.5.
  - Tournament process 11.5, of which games 6.2.
  - Self-games 0.5.
- H#2 training step:
  - Setup: stall-h #2 on cycle 227's set (4,800 games, 524,922 positions), 11.0 CPU-s; perf cycles:u, cpu_core
    listing, 21k samples.

    | Symbol | Share |
    |---|---|
    | class_products | 21.8% |
    | node_output_ctx | 15.8% |
    | cnbp_factors | 9.5% |
    | bcmp | 8.3% |
    | train_weighted | 7.4% |
    | memmove | 4.9% |
    | predict_training_ctx | 4.5% |
    | log | 4.4% |
    | probs_of_latent | 3.3% |
    | malloc, realloc, reserve, free | 5.7% |

  - The bcmp is node_output_ctx's `cache_values != values`. objdump shows it is the function's only bcmp call.
  - Reading the set costs ~0.12-0.13 CPU-s a line: user time minus the logged train time (C 1.73 vs 1.61, F-P 0.26
    vs 0.13).
- **Profile trap.** This CPU has two PMUs (cpu_atom, cpu_core), and perf report prints a listing for each.
  - A short run that starts on an E-core shows a small cpu_atom listing (52-107 samples, the file read).
  - In that listing games::parse_position looks like 35-60% of the run. It is ~4.5% on the P-cores.
  - Read the cpu_core listing, or pin with taskset.
- Read arenas:
  - Tonight's cost ~4.65 CPU-h.
  - ms a game:

    | Ply | Pairing | ms a game |
    |---|---|---|
    | 3 | p3-p3 | 146-235 |
    | 2 | | 16-23 |
    | 1 | | ~1 |

  - Mixed-ply arenas spend 30% (plies 1-2) to 56% (plies 1-3) of their CPU on cross-ply games.

**Candidates, ranked.**
1. **Cell-DAG training overhead.** This is the ~18:50 candidate 2, re-measured on grow3.
   - What:
     - node_output_ctx's per-call Vec allocation;
     - values_for's linear search over the marks;
     - the bcmp of cache_values;
     - the copies;
     - cnbp_factors, recomputed when one node alternates between path values within a step (the single-entry
       cache thrashes).
   - Cost:
     - The ctx-DAG lines take 44.8 of router-p1's ~66-71 CPU-s a cycle (~65%).
     - The addressable part is ~40-45% of such a line's cycles (table above).
   - Saving:
     - 15-25% of a ctx-DAG line's training (the 18:50 estimate stands): 6.7-11 CPU-s a cycle.
     - That is 10-16% of router-p1, but only 0.12-0.5 CPU-h of what's left.
     - In the global league, ctx-DAG training is <1% (0.08 CPU-s a cycle).
   - Exactness:
     - Interned path ids and a reused buffer give the same cache hits.
     - A per-node, per-step memo of factors yields the same f64s from the same operations.
     - Proof: byte-identical states, models and logs on review-1112a/train (H#2, C) and review-1111c/train, then 5-10
       live router-p1 steps of each rule.
   - Risk: low-moderate. The memo must be invalidated by a router split or a table update inside a step.
   - **Don't build now.**
     - It clears 5% of router-p1 on paper. But router-p1 ends within ~40-100 min, before a build and its identity
       checks could land.
     - No queued run trains ctx DAGs on large local sets: diversity-p2 has none, and the next global league's DAGs
       train ~8k positions a cycle.
     - Build it first if a router follow-up with 4 or more ctx-DAG lines on 4,800-game sets is queued. There it is
       ~10-16% of the run.
2. **compound's 64-node cap for grown trees.** Conditional; it is "left" in the hybrid15 section.
   - When it binds:
     - The next global league's T-P-Ne-grow and T-L-Ne-grow trees (D 10M) pass 64 nodes at ~21 splits, ~200M
       positions.
     - At ~8k positions a cycle and ~13.7k cycles a day, that is ~2 days.
   - Cost: past the cap a tree plays float, at hybrid14's speed (about 2x a game). Games are ~21% of the global
     league, so this is up to ~10-20% of a long league.
   - Change: MAX_NODES appears only in compound's eligibility check (hybrid16-src compound.rs:40 and :263), so the
     change is a constant.
   - Measure first: fixed-point play at 64-256 nodes may stop paying.
   - Exactness: compound's own float fallback inside the bound, plus hybrid5's identity tests at the raised cap.
   - Do it only if the user sets no 64-node cap for the next global league and it runs past ~2 days.
3. **Fixed-point trees and DAGs past ply 2.** Conditional.
   - Ply-3 tree and DAG games cost 146-476 ms. Only reads use them (diversity-p2's ply-3 read, ~1 CPU-h estimated).
   - It is a large build: alpha-beta with exact float re-checks.
   - Do it only if a ply-3 league or a series of ply-3 reads is queued.

**Design notes for the user (not exact speedups).**
- The global league's per-cycle overhead is ~7.3 CPU-s (driver 1.9, reports 2.1-3.0, own calibration 2.2). Its 280
  games take 1.98. More games a cycle (the games-per-cycle standard) amortise the overhead almost linearly.
- diversity-p2: check that the frozen references' own calibration runs once, not every cycle. Otherwise it costs
  ~4 x 1.2 CPU-s a cycle (~12%).

**Checked and dropped.**
- Parsing the local set once for all lines: 0.12-0.13 CPU-s a line a cycle. For router-p1 that is 1.2 of ~70 (1.7%),
  0.02-0.06 CPU-h left. The large parse_position shares came from the cpu_atom listing.
- The global full fit taken from the last prefix fit is not exact.
  - The prefix fits warm-start from the previous prefix: `ratings(n, &upto, opts, previous, false)`.
  - The full fit starts cold: `ratings(n, games, opts, None, true)`.
- The prefix fits' `upto` copies and tally rebuild: ~0.45 CPU-s a cycle, ~0.19 CPU-h over the tail. Item 3 of the
  ~18:50 review already covers them.
- Splitting the queued router-p1 stage read by ply: the cross-ply games are ~30% of ~22 CPU-min (~0.1 CPU-h). It is
  not exact (another schedule) and small.
- partial_shuffle: 8% only in the cpu_atom listing, under 1.5% on the P-cores.
- router-p1's tournament reports (~5.3 CPU-s a cycle, 7.5%): whole-history work (item 3 of ~18:50), 0.1-0.24 CPU-h
  left.
- router-p1's own calibration (12.5 CPU-s a cycle, 18%): the 600 own games a version dominate. Fewer games would be
  a design change (ledger).
- The global driver's 1.9 CPU-s a cycle (21%):
  - contrast_ses was ruled out before;
  - the scans are cached;
  - the records are small.
  - It was not profiled this time; that needs a full snapshot cycle.

### 2026-11-12 ~06:45 review (scheduled)

**Running and changed since ~01:00.**
- Running (06:45-07:10):
  - fponly-p2b: ply 2, F-P x4, hybrid6 training, hybrid16 games; ends ~07:40.
  - probit-f10-p1: ply 1, F-P x2 + capped T-P-Ne x4 (forget 1 / 10), hybrid6 training; floor 300, cap 400.
  - grow-p1b2: ply 1, F-P x2 + T-P-Ne-grow x4, chessai-grow1 training.
  - The global league: 2 DAG lines, cycle ~4,935, open-ended (its endgame is the user's call).
  - Panel-read arenas from the waiters in models/from-scratch/global11-drafts/*-read.sh.
- New code: grow5 (--grow-shadow-forget), lrdecay1 (failed offline, unused); leagues.py games_per_cycle and
  reference_window; arena scripts make_split_read.py and arm_panel.py.
- Queued (not approved): the next global league (analyst 6.71's six at ply 1, games_per_cycle 1584).

**Measured.** Sources: totals.csv; tournament.log; the global driver's own CPU from /proc. Profiles (nice 19, pinned to
P-cores) on reflink snapshots: one global tournament at cycle 4929 (CHESSAI_REPORT_TIMES=1, perf cycles:u); a ply-1
own calibration (probit-f10-p1's capped T-P-Ne v99, 600 games); training steps on a copied local set (cycle ~180).
Scratch: review-1112b/ (README.txt).

| Workload | CPU-s a cycle | Wall a cycle | Left |
|---|---|---|---|
| global league | 11.55 (tournament process 8.8, of which games 2.3; own cal 2.6) + driver 2.7-3.7 = ~15 | 9.9 s | open-ended: ~36 CPU-h a day |
| fponly-p2b | 28.8-31.0 (tournament 21.5-23.1, own cal 3.4-3.6, self-games 3.0-3.5, training 0.8-1.0) | 10.7 s | ~1.4 CPU-h |
| probit-f10-p1 | 13.5 early → 16.8 (own cal 6.7-7.3, training 3.3-5.8, tournament 3.2-4.0 with games 1.9, self-games 0.3) | 8.7-10.1 s | ~0.6-1.1 CPU-h |
| grow-p1b2 | 11.6 early → ~18.5 late (grow-p1b's last 50 cycles) | 8-10 s | ~1.2-1.5 CPU-h |

- Global tournament process (8.8 CPU-s user, 0.9 sys):
  - The report is 5.7 s wall. The 5 sequential prefix fits take 0.83-1.31 s each (0.11 s at cycle ~508, 0.27-0.48 s at
    ~2,650); the tables take 0.25 s.
  - perf: sparse_fit's own code is 40% of the process. 88% of that is one loop, Blocks::new's field search
    (hybrid16-src tournament.rs, `(0..fields.len()).find(|&f| set.iter().all(|&v| holds[f][v]))`).
  - The loop scans 24,655 tournaments against ~5k fields. Each field is a `vec![false; k]` bitmap with k ≈ 15.2k, about
    75 MB per fit, allocated and zeroed six times a cycle. That zeroing is the 0.9 s sys.
  - Blocks::factor is 1%.
  - So the field search costs ~3.1 CPU-s user plus ~0.5-0.8 s sys a cycle, and ~3.5-4 s of the 9.9 s wall. It grows
    quadratically with history (the earlier reviews put the growth in "fits", item 3 of ~18:50).
  - Experiments' tables take 0-1 s a cycle (~4k tournaments, ~350 fields), so it is negligible there.
- Global disk:
  - elo_history.csv is 9.45 GB and grows 4.8 MB a cycle (5 tournaments × every player ever, 15.2k retired). The rate
    rises ~0.45 KB a cycle every cycle.
  - runs/ adds ~1.5 MB a cycle (a D-L model 1.08 MB, a D-P model 0.46 MB a version).
  - Together ~55 GB a day now, ~4 GB a day more each day. 585 GB free: full in ~4-5 days if the league keeps running.
- Own calibration at ply 1 (one version, 600 games, depth 1, T 0.1, pinned P-core):
  - 1.30-1.35 s user. With 2 games (held-out load and fit only) 0.26-0.27 s.
  - perf, the 600 games minus the 2-game run:

    | Part | Share |
    |---|---|
    | malloc/free | 20.4% |
    | write_round + join + u8 fmt | ~2% |
    | exact float probit leaf values (predict_log and its log/exp) | ~24% |
    | compound play | ~17% |
    | move generation | ~5% |

  - Call graphs are refused here (perf_event_paranoid 2 blocks DWARF, LBR and FP chains). A sampling LD_PRELOAD malloc
    tracer (review-1112b/mtrace: backtrace on 1 malloc in 64, symbolized offline) attributes 95.9% of the run's 4.97M
    mallocs to selfplay::write_round.
  - write_round does `pos.iter().map(u8::to_string).collect::<Vec<String>>()`, then `join(" ")`, per position: 72
    allocations × 66,135 positions (70 features a position).
  - So write_round is ~22% of an own calibration (~0.29 s a version). It is also ~0.4 ms per saved game in tournaments
    (--save-games, once per game, outside the per-game CPU timer: in tournament_proc, not tournament_cpu) and in
    self-games.
- Training: one step each on the same copied local set (~193k positions), pinned P-core, ABBA.
  - Capped T-P-Ne: hybrid6 1.35-1.37 s user, hybrid16 1.19-1.23 (×0.89).
  - Capped T-L-Ne at forget 10: 1.58-1.63 s, against 1.45-1.49 (×0.92).
  - F-P: 0.13-0.14 s on both.
  - Models and states were byte-identical (md5) for all three.
  - hybrid6's profile:

    | Symbol | Share |
    |---|---|
    | Cnbp::predict_counts | 36% |
    | probit update / predict | ~27% |
    | Cnbp::train_weighted | 8% |
    | parse_position | 3% |

  - errdag4's one-pass class_products is in hybrid14 and later, not in hybrid6.
- Global driver's own CPU: 2.73-3.74 CPU-s a cycle over 6 cycles (1.9 at 01:00). Not profiled.

**Candidates, ranked.**
1. **Indexed field search in the rating fits** (hybrid16-src tournament.rs, Blocks::new ~959-1000).
   - Change:
     - Per variable, keep the list of fields that contain it, appended in creation order.
     - A set scans only the fields of its rarest variable, in increasing order.
     - Containment is tested against the field's sorted variable list. No k-wide bitmaps.
   - Exact:
     - Any field that holds the set holds its rarest variable, so the first containing field in creation order is the
       same field.
     - New fields are made in the same order with the same content.
     - So fields, field_of, shared and the blocks are identical, and the floating-point work after them is the same.
   - Cost now: ~3.1 CPU-s user plus ~0.5-0.8 s sys a global cycle (~25% of ~15 CPU-s) and ~40% of its wall. It grows
     quadratically.
   - Saving:
     - ~25% of a global cycle's CPU, cycles ~1.6x as fast.
     - For an open-ended league that buys more cycles an hour on the same ~1.5 cores, not fewer CPU-hours. The CPU-hours
       saved are real only for a finite run.
     - The next global league would reach this regime within ~2-3 days.
   - Proof:
     - A unit test, old against new, on random set families.
     - Snapshot tournaments of leagues/global and of two experiments' tours, hybrid16 against the new binary.
       ratings.csv/.md, head_to_head.*, the bytes elo_history.csv appends, elo_history.key and games.csv must be
       byte-identical. game_cpu.csv holds timings, so it is excluded.
   - Risk: low.
2. **write_round without per-value Strings** (hybrid16-src selfplay.rs).
   - Change: write each u8's decimal digits straight into the output String, with single spaces and '\n'. That is what
     `to_string` + `join(" ")` + `writeln!` produce, so the bytes are the same by construction.
   - Cost now:
     - ~22% of every own calibration and ~0.4 ms a saved game.
     - A 6-line ply-1 cycle: ~2.1 CPU-s (own cal ~1.5, tournament ~0.65), ~12%.
     - fponly-p2b: ~1.5 of ~29 CPU-s (~5%). The global league: ~0.5 of ~15 (~3%).
   - Saving: ~90% of that.
     - The running ply-1 runs end within ~0.5-1.5 h, so under 0.25 CPU-h there.
     - It is worth ~12% of every new ply-1 experiment or league.
   - Proof:
     - Byte-identical .games and human.games from own calibration (lr, probit, bdt, bdd, cnbp), tournaments with
       --save-games, self-games and self-play rounds.
     - Training on them identical.
   - Risk: very low.
3. **No build: train trees with hybrid16 instead of hybrid6.**
   - ×0.89 on the capped probit error tree and ×0.92 on the capped LR tree; F-P equal. Byte-identical on one step each.
   - Worth ~3% of a 6-line ply-1 cycle, below the bar alone.
   - Make it train_binary.py's default for new runs after a wider identity check (cluster trees; grow lines keep their
     grow binaries).

**Design notes for the user (not exact speedups).**
- Disk: the global league writes ~55 GB a day (elo_history.csv quadratic, models a version) against 585 GB free, so the
  disk is full in ~4-5 days if it keeps running. Options:
  - stop the league (its endgame is already flagged);
  - cap or rotate elo_history (a format change).
- The global league: games are 2.3 of ~15 CPU-s a cycle. The rest is overhead that grows with history (fits, driver).

**Checked and dropped.**
- Exact float probit leaf values in ply-1 play (~24% of an own calibration): inherent, because the log-odds softmax
  needs every root value exactly (hybrid8's note).
- The held-out fit (0.26-0.27 s a version, ~20% of an own calibration): every version is new (ruled out at ~18:50).
- Parsing the local set once for all lines: ~0.1 s a line, ~4% of a 6-line cycle, and it needs one process for all
  lines.
- fponly-p2b's ply-2 F-P games (~65% of its CPU): fastprobit since hybrid8, and the run ends ~07:40.
- Experiments' tournament tables (0-1 s a cycle): small.
- The global driver (2.7-3.7 CPU-s a cycle): contrast_ses was ruled out before as inexact. Profile it at the next review
  if the league continues.

### 2026-11-12 ~07:15 chessai-hybrid17 (implementation fork; candidates 1 and 2 of the ~06:45 review)

- **What** (fastbin/hybrid17-src; hybrid17/hybrid17.patch against hybrid16-src: src/tournament.rs, src/selfplay.rs;
  Cargo files unchanged). Installed as **bin/chessai-hybrid17, md5 e4a4fbd4bb104579c942fb654cbf3e11** (cp -n).
  - (1) `first_fields` (Blocks::new's field search): per variable, the list of fields holding it, in creation order; a
    set scans only the fields of its variable in the fewest fields so far, testing containment against the field's
    sorted variable list (`holds_all`). Any field holding the set holds that variable, so the first match is
    hybrid16's first match; fields, field_of, shared and the blocks are the same, and so is the arithmetic after them.
  - (2) `outcome_text` / `push_decimal` (write_round): each u8 value's decimal digits pushed into one byte buffer,
    single spaces, a newline a position: the bytes of `to_string` + `join(" ")` + `writeln!`, without 72 allocations a
    position.
- **Identity** (against bin/chessai-hybrid16; nice 19, cores 4-7; scripts in fastbin/hybrid17/scripts/):
  - 202 unit tests pass, two of them new: first_fields against hybrid16's bitmap search on 400 random set families
    (copies, subsets, empty sets; over 1,000 fields), and outcome_text against hybrid16's formatting over every byte
    value, each result, an empty game and write_round's files.
  - Global league: a consistent snapshot of the running league inside cycle 5060's train phase (snapg.py: tour/ plus
    the four entries' own games and models; one copy of 1.43 s; three earlier attempts failed or retried), 25,305
    round-robins of history (elo_history.csv 10.39 GB). The driver's tournament command replays 5 round-robins (280
    games) from a pristine reflink copy per run (tourrun.py). Six runs: fit cache on as live, and off. Four
    comparisons (cmpout.py), 1,152 files identical each time: 1,120 pool game files (5.56 MB), ratings, head-to-head,
    games.csv, elo_history.key, players/retired, the own games. elo_history.csv: the same size, the 3.27 MB appended
    identical, and its 81,761 prefix extents still the snapshot's (filefrag: untouched). game_cpu.csv differs only in
    cpu_s, tournament.log only in "tables in Ns".
  - grow-p1b (finished; 4,236 round-robins): own games regenerated from owncal/<key>.json, then a 12-round-robin replay
    (1,584 games): 6,407 files identical (6,336 game files, 25.9 MB; elo_history's 1.04 MB appended identical).
  - Own calibration (make_own's arena command): 13 versions (grow-p1b's 4 F-P and 8 T-P-Ne-grow, error-routing-p1's
    capped T-P-Ne v314): 52 files (135 MB) identical.
  - Self-games (replay-games as phase_selfgames, 144 games): the game lines and 576 files identical. self-play (probit,
    2 rounds x 40 games): 11 files identical.
- **Saving** (user + sys of the process):

  | Workload | hybrid16 | hybrid17 | Ratio |
  |---|---|---|---|
  | global tournament step, cache off, ABBA (2 each) | 9.12, 9.29 | 6.71, 6.84 | x0.74 (-2.4 CPU-s) |
  | own calibration at ply 1 (13 versions, alternating) | 1.295 mean | 1.073 mean | x0.83 |
  | grow-p1b step (12 round-robins, 1,584 saved games) | 5.79 | 5.03 | x0.87 |

  - The global league: ~2.8 of ~15 CPU-s a cycle (~19%), the fit part growing with history; but faster cycles only
    buy more cycles while it runs open-ended (and fill the disk faster). A six-line ply-1 experiment: ~2 of ~16-18
    CPU-s a cycle (~11-12%).
  - sys fell only 0.92 -> 0.75 in the global step: the zeroing of the bitmaps was not most of the sys time.
- **Swaps:** none (the directive). For a swap: config.json's tournament binary between cycles (swap16.py's procedure);
  the fit cache refits every entry once (its key holds the binary's bytes) and arena.py remakes own games once (it
  keys on the binary's md5). Training binaries unaffected.
- **Scratch** fastbin/hybrid17/ (global/, exp/, own/, selfplay/; ~535 MB exclusive, the rest shared extents) and
  hybrid17-src/target: deletable, the user's call.

### 2026-11-12 10:58:57 the driver's self-wait fixed (observers fork; the 11-11 ~01:40 note above)
- **What:** leagues.py's wait_not_busy now looks again 1 s later before waiting 30 s. run_measured registers a child
  only after Popen returns, so a parallel training thread's league_busy scan could take a sibling's training process
  for a foreign one ("waiting for pid N, still working in this league" right after "training N lines").
- **How often:** 656 times across the experiments' driver logs, ~8% of cycles, 30 s each; ~20% of treeonly-p1's wall
  time before the fix.
- **Exactness:** timing only; games, ratings, models and records unchanged (no result path touched). Tested with a
  fake chessai process: a short-lived one gives no wait, a lasting one is still waited for. Genuine waits (another
  process with the league's arguments, e.g. treeview) still happen.
- **Swap:** leagues.py f20818ff (pre-observers) -> c385ee34, atomic, at 10:58:57; treeonly-p1 and fponly-p1 picked it up
  at their next cycle with no waits since. Backups leagues/leagues.py.before-observers.

### 2026-11-12 ~12:50 review (scheduled)

**Running and changed since ~06:45.**
- Running (12:50-13:05):
  - probit-p3: ply 3, F-P x2 + capped T-P-Ne x2 (split limit 4), hybrid17 games, hybrid6 training, floor 300, cap
    400. Cycle 142 at 13:00.
  - The fponly cold twins (-coldtrain, -coldrate): ply 1, cycles 44-47 of their 300 floor. The treeonly pair is done.
  - archparity-p4: a ply-4 arena, 29 players, hybrid16. 5 of 20 round-robins in 6,343 s; it is to be cut at 10.
  - Also: the cold-read arenas (plies 1-2), another fork's filter-2 tests, dashboards.
- New code: hybrid17 (07:25); the busy-wait fix (10:58); leagues.py observers, cold_games and train_filter
  (filter-2 in progress); arena.py's pinned anchor, per-entry ply and fit report; chessai-grow6.
- Queued:
  - grow-dag-p1 and filtered-p1 with its twin, all at ply 1;
  - ladder re-runs;
  - more ply-3/4 work if probit-p3 opens it. The 6.84 rule: a tree lead of 30 or more at the final means a ply-4
    mixed field next.

**Measured.**
- Sources: totals.csv, self_games.csv, tour/game_cpu.csv, the ladder's and archparity-p4's game_cpu.csv,
  grow6/summary.txt.
- Profiles: perf cycles:u, flat (call graphs are refused here), nice 19, one thread, pinned to the E-cores, on
  reflink snapshots of probit-p3's v135 models. Scratch: review-1112c/ (README.txt).

| Workload | CPU-s a cycle | Wall a cycle | Left |
|---|---|---|---|
| probit-p3 | ~123: tournament ~103, self-games ~15 (trees 62%), own cal ~3.5, training ~1.4 | 48 s (tournament 39 s) | floor: 158 cycles, ~5.4 CPU-h, ~2 h; cap: 258 cycles, ~8.8 CPU-h, ~3.2 h |
| archparity-p4 | 1,225 CPU-s a round-robin (812 games, 1.51 s a game) | ~1,270 s a round-robin | 5 round-robins: ~1.7 CPU-h, ~1.8 h |
| fponly cold twins (each) | ~10: own cal 3.1, observer training 2.5-2.9, tournament 2.3, training 1.0-1.2, cold 0.4-0.5, self-games 0.3 | 12-13 s | ~255 cycles: ~1.4 CPU-h for both |
| ladder-1 (done) | 346 CPU-s for 18,920 games; trees at ply 3 41 CPU-s (12%) | 524 s | ~0.1 CPU-h a re-run |
| grow-dag-p1 (queued) | 17-25 (grow6/summary.txt) | | 3.3-5.5 CPU-h |
| filtered-p1 and twin (queued) | | | ~3-7 CPU-h |

- probit-p3's games: over the last 8k games, TT 0.181 s, FT 0.132 s and FF 0.089 s a game.
  - So a tree side costs ~2x a flat side at ply 3.
  - Tree sides are ~70 of the tournament's ~103 CPU-s; with their self-games, ~79 of ~123 CPU-s a cycle (65%).
  - F-P sides are ~40 (33%).
- Trees at ply 3 (T v T, 60 games, 0.22 s a game), model evaluation ~73%:

  | Part | Share |
  |---|---|
  | Cnbp::probs_products3 | 27.0% |
  | Cnbp::predict | 5.0% |
  | best_child | 2.6% |
  | Probit::predict_log | 11.9% |
  | log_probs_of_latent | 5.1% |
  | scaled_tail | 1.5% |
  | libm log, exp, log1p, expm1 | ~12% |
  | calibration logits | 2.8% |
  | encode | 3.1% |

  The rest: move generation ~8.5%, child (clone, play, hashes) ~8%, has_legal_move 3.0%, captures_first 2.4%,
  sorts ~2.2%.
- Trees at ply 4 (4 games, 3.58 s a game, 16x ply 3): model evaluation ~85%.
  - Routers 40.7%; probit leaf and libm ~35%; calibration 4.5%; encode 3.8%.
- The same trees at ply 2, ABBA, 60 games a run:
  - The float negamax (CHESSAI_FX=direct, the path trees take past ply 2): 50.8 and 50.8 ms a game.
  - Compound (the default): 22.1 and 22.2 ms a game.
  - So ×0.435; hybrid5 measured ×0.44 on probit-leaf trees.
  - games.csv was identical across the modes.
  - Evaluation is ~81% of the float run's profile, so compound play costs ~30% of the float evaluation it replaces.
- F-P at ply 3 (50 games, 93 ms a game): the float probit link is ~49%.
  - The link: log_probs_of_latent 12.3%, log 12.3%, log1p 7.7%, exp and expm1 ~7.5%, calibration logits 3.7%,
    scaled_tail 3.3%, eval_sums_turn 1.4%.
  - The rest: best_capture 7.7%, child_fx 7.5%, Acc::move_sums 5.6%, has_legal_move 5.0%, negamax_fx 4.3%,
    Acc::child 3.6%, depth1_from 2.7%.

**Candidates, ranked.** Both are items the ledger already holds as conditional. Their condition is now met, because
probit-p3 is the first ply-3 field with trees and F-P.

1. **Fixed-point trees and DAGs past ply 2** (item 3 of the ~01:00 review).
   - Where: search.rs's float negamax (child, negamax, eval) on root_values at depth 3 and more. That covers
     tournaments, arenas and the self-games (replay-games).
   - Change: a compound alpha-beta.
     - A compound::Acc per depth (child_into) and Changes per child, so no encode() calls at all.
     - Interior keys (depth ≥ 2 below the root) from compound's estimate, with no float. Root keys stay float (cheap,
       and the T ≤ 0.05 root order stays hybrid17's).
     - At a leaf, the estimate's bound:
       - hi ≤ alpha: return hi.
       - lo ≥ beta: return lo.
       - Otherwise the float leaf value (compound's exact arithmetic), as now.
     - Kept as now: terminal and draw checks; compound's own router fallback inside the bound.
     - Kept float: the outcome search, Value::Score, and graphs over compound's caps.
   - Cost now: tree sides ~79 of ~123 CPU-s a probit-p3 cycle.
   - Saving:
     - Tree sides ×~0.5 at ply 3 (0.27 + 0.73 × 0.30) and ×~0.4 at ply 4 (0.15 + 0.85 × 0.30).
     - probit-p3: ~39 CPU-s a cycle (~32%), and the cycle wall ~25% shorter.
     - What probit-p3 actually gets depends on when the build lands:
       - from cycle 250 (~14:25): ~0.5 CPU-h to the floor, ~1.6 CPU-h to the cap;
       - after cycle 300: 0-1.1 CPU-h.
     - A ply-4 mixed field like probit-p3:
       - ~1,400-1,700 CPU-s a cycle at today's code, tree sides ~75%, so ~120-140 CPU-h for 300 cycles;
       - ×~0.55 with this.
   - Exactness:
     - Every root move has a full window, so its value is the subtree's minimax value in any move order. The only
       exception is ±0, which exp() and the comparisons can't tell apart; hybrid10 changed this order and stayed
       byte-identical.
     - A leaf's bound on the window's far side keeps each node's classification (fail low, exact, fail high) and
       every exact value.
     - Alpha moves only on exact values, so the windows and the nodes visited match float keys in the same order.
     - root_values_windowed still gets valid bounds, and its pick is exact given valid bounds.
   - Identity checks:
     - Tournaments at plies 3-5 with --save-games, at T 0.1, T 0.02 and argmax, against hybrid17. games.csv and every
       game file must be byte-identical. Entries:
       - T-P-Ne, T-L-Ne, an LR-router tree and a TAN-router tree;
       - a DAG with cells and a cell-less ctx DAG;
       - a grown tree over the 64-node cap (float);
       - F-P, F-L and random as controls.
     - Self-games (replay-games) byte-identical.
     - CHESSAI_FX_CHECK=1 asserting each leaf bound against the float value, and every root value bit-equal to the
       float negamax (as root_values_compound does at ply 2).
     - A unit test: root values bit-equal, compound against float, at depths 3-5 on random positions.
     - The outcome search unchanged, by diff.
   - Risk: moderate. It is a large build, and the bound must hold through the calibration and the fail-soft returns.
     The checks above catch errors.
2. **Probit link bounds past ply 2, flat probit** (the 11-11 ~03:50 notes "probit leaves under a cut window" and
   "probit keys").
   - Where: negamax_fx's leaves and its depth ≥ 2 keys for a probit head. hybrid8's ln Φ table bound settles a leaf
     on the window's far side; keys come from the estimate.
   - Cost now: F-P sides ~40 CPU-s a probit-p3 cycle, ~49% of it the float link.
   - Saving:
     - F-P side ×~0.65 (hybrid8 got ×0.645 at ply 2): ~14 CPU-s a cycle (~11%).
     - From cycle 250: ~0.2 CPU-h to the floor, ~0.6 CPU-h to the cap.
   - Exactness: the same argument as 1; hybrid8's bound proof extends to it.
   - Identity checks: as 1, for F-P (with F-L and NB controls) at plies 3-5.
   - Risk: low to moderate. It shares the bound code with 1, so one fork could build both: ~53 CPU-s a probit-p3
     cycle (~43%).

**Design notes for the user (not exact speedups).**
- Whether 1 and 2 pay off depends on what follows probit-p3.
  - On probit-p3 alone, a 2-3 h build lands near its floor (~15:00).
  - They matter for ply-3/4 reads with trees and for any ply-4 field.
  - A ply-4 mixed field would need a smaller design anyway: ~55-75 CPU-h for 300 cycles even with both builds.
- The queued work at ply 1: own calibration is the largest single cost (grow-dag-p1: 6 × ~1.3 s, 30-45% of a
  cycle). Its exact parts were covered at 06:45 and by hybrid17; fewer own games is a design change (noted before).

**Checked and dropped.**
- DAG shadows in grow6 training.
  - They are ~30% of a DAG line's training: 154.8 s against 119.4 s with the allowance off and 72.9 s for the tree,
    per 112 P1 rounds.
  - In grow-dag-p1 that is 1.0-2.1 CPU-s of a 17-25 CPU-s cycle (6-8%), or 0.26-0.39 CPU-h over the run.
  - Each shadow is its own model with its own counts (real copies, and null copies on a hash partition), so no work is
    shared. An exact change could trim per-example overhead only: under 0.1 CPU-h. Below the bar; not profiled.
- archparity-p4 on hybrid16 rather than hybrid17: the game code is the same (hybrid17 changed only the rating fit and
  write_round), so the per-game cost is the same. It ends (~1.7 CPU-h, ~1.8 h) before any build.
- Cold games and top-up self-games at ply 1: design costs, small (cold 0.3-0.5 CPU-s a cycle in the twins).
  - Root windows can't cut at depth 1.
  - Leaf bounds at T 0.02, depth 1, would save ~1% of a filtered-p1 cycle.
- Ladder re-runs: ~0.1 CPU-h each, with ply-3 tree sides 12% of that; no trees at ply 4.
- has_legal_move at leaves (3% for trees, 5% for F-P at ply 3): left, as before.
- probit-p3's training (hybrid6, ~1.4 CPU-s a cycle): the 06:45 candidate 3 (×0.89 on trees) is ~0.1% here.

### 2026-11-12 ~14:20 chessai-hybrid18 (implementation fork; candidates 1 and 2 of the ~12:50 review)

- **What** (fastbin/hybrid18-src; hybrid18/hybrid18.patch against hybrid17-src: src/search.rs, src/compound.rs,
  src/fastprobit.rs, src/main.rs; Cargo files unchanged). Installed as **bin/chessai-hybrid18, md5
  392eab6a628c580afa3ffa1c78706fa5** (cp -n, 14:17:05). The values-only search (root_values: tournaments, arenas,
  replay-games) at depth 3 and more:
  - (1) Trees and DAGs on the compound fixed-point path (≤ 64 nodes, log-odds values, the direct path) keep the
    compound state through the search: a compound::Acc per depth (child_into) and compound::Changes per move, no
    encode(). A depth-1 node scans its replies' estimates (value, error bound) in the float search's order: it fails
    high as soon as a top v+e ≤ −β (returning −(v+e)); after the scan it fails low when α > −∞ and every bottom
    v−e ≥ −α (returning −min(v−e)); otherwise Compound::best values in full only the replies that can be the minimum
    and runs the float loop in order. Keys at depth ≥ 2 from the estimates in the exact float order (order_by_bounds:
    by interval top; the exact key, Compound::exact_of, only inside groups whose intervals overlap; ties in index
    order; anything non-finite valued in full). Leaf hashes as hybrid11 (reversible moves only).
  - (2) Flat probit at ply 3+: the same scan and key order from hybrid8's ln Φ table (fastprobit::estimate / best),
    the same code (leaves_p), in hybrid12's staged move order.
  - Exact: fail-soft alpha-beta; every node fails low, is exact or fails high as in the float search, with the same
    exact values, so α moves on the same values and the same nodes above the leaves are visited; only a failing
    node's returned bound can differ (both valid). Full root windows give bit-equal root values (same move order,
    so even ±0); hybrid13's windowed roots cut the same moves with the same exact values.
  - Unchanged: the outcome search (root_outcomes, expected_outcome, outcome, resolve, child, negamax and child_fx
    verbatim; the blocks added to the generic negamax_fx and depth1_from are gated by `!OUTCOME`: hybrid18/fndiff.txt,
    hybrid18/outcome-gating.txt), depths 1-2, flat LR and NB, FM, MoEs, graphs over 64 nodes (float), Value::Score,
    own calibration (depth 1), training.
  - CHESSAI_FX_DEEP=off gives hybrid17's search; CHESSAI_FX_CHECK=1 asserts every decision (each bound against the
    float value, each key order against the stable sort of the exact keys, every root value bit-equal to the float /
    hybrid17 search); CHESSAI_FX_STATS=1 prints "deep fx:" counters.
- **Identity** (against bin/chessai-hybrid17; byte-identical: games.csv, every tour file but tournament.log
  (game_cpu.csv without its CPU column), every saved game, stdout with times masked; scripts hybrid18/tour18.py,
  tour18b.py, replay18.py, replay18b.py, suite-*.sh, logs suite-*.log):
  - Tournaments, one round-robin each, at T 0.1, T 0.02 and argmax:
    - ply 3, 12 players, 132 games: T-P-Ne ×2 (probit-p3's capped v135), F-P ×2, T-L-Ne (library capped-l-f10-p1),
      T-L-Lc (LR routers), T-L-Tc (TAN routers), a DAG with cells (error-dag-p1, z 0), a cell-less ctx DAG (z 160),
      a 121-node tree (over the 64-node cap), F-L (base-lr-p2), random;
    - ply 4, the same minus the second T-P-Ne and F-P, 90 games;
    - ply 5: T-P-Ne, F-P, the cell DAG and random (12 games), and T-L-Ne, the LR- and TAN-router trees, the cell-less
      DAG and random (20 games).
  - Check mode (CHESSAI_FX_CHECK=1) at ply 3, T 0.1 and T 0.02 (132 games each); and at ply 4, T 0.02 (90 games, 608 CPU-s in check mode).
  - Self-games (replay-games as phase_selfgames runs them, seed 1000064), ply 3: 22 games at T 0.1, 11 at T 0.02.
  - 205 unit tests pass, 3 new: order_by_bounds is the stable sort of the exact keys; compound deep searches value
    exactly at depths 3-5 (trees with LR, CNBP and TAN routers, a DAG, DAGs with cells, LR and probit; windowed
    searches cut the same moves); probit bounds keep root values at depths 3-5 (windowed, root_value_of, the outcome
    search untouched).
  - The outcome search unchanged, by diff (above).
  - Two builds: the first candidate (577a99b9…, 13:26) read CHESSAI_FX_DEEP per search; the final one reads it once per
    Search (allow_bounds, so the unit tests can toggle it), 400 bytes less code. The first ran suites A-C; every
    comparison was repeated on the final build against the kept hybrid17 runs (suite E: 14 of 14 SAME); suite D's
    ply-5 runs after 13:55 ran the final build.
- **Saving.** Game CPU a ply, ABBA (h17 h18 h18 h17), RAYON_NUM_THREADS=1, nice 19, taskset -c 4-7 (bench18.py;
  games.csv identical in all four runs):

  | Workload | hybrid17 µs a ply | hybrid18 | Ratio (pairs) |
  |---|---|---|---|
  | probit-p3's kinds (T-P-Ne ×2, F-P ×2), ply 3, T 0.1, 180 games | 1,371 / 1,334 | 849 / 839 | ×0.624 (0.619, 0.629) |
  | T-P-Ne v T-P-Ne, ply 3, T 0.02, 60 games | 1,818 / 1,810 | 1,010 / 997 | ×0.553 (0.555, 0.551) |
  | F-P v F-P, ply 3, T 0.02, 50 games | 736 / 705 | 582 / 570 | ×0.799 (0.790, 0.808) |
  | T-L-Ne, LR-router tree, cell DAG, ply 3, T 0.02, 60 games | 2,571 / 2,502 | 901 / 899 | ×0.355 (0.351, 0.359) |
  | T-P-Ne v T-P-Ne, ply 4, T 0.02, 8 games | 18,070 / 18,362 | 8,675 / 8,650 | ×0.476 (0.480, 0.471) |
  | T-P-Ne v F-P, ply 5, T 0.02, 2 games | 87,322 / 86,201 | 53,045 / 52,267 | ×0.607 (0.607, 0.606) |

  - Unpinned whole fields (process CPU, 2 threads, the identity runs): ply 3 ×0.57-0.62, ply 4 ×0.48-0.53, ply 5
    ×0.37-0.53 (with the controls, whose code is unchanged).
  - The review guessed tree sides ×~0.5 at ply 3 and ×~0.4 at ply 4, F-P ×~0.65: trees came out at or better than
    that (T-P-Ne ×0.55 / ×0.48, T-L-Ne and the router kinds ×0.36), F-P worse (×0.80 at T 0.02).
  - The bounds at work (CHESSAI_FX_STATS=1, probit-p3's kinds, ply 3, T 0.1, 3 round-robins): 94% of the 2.95 M
    nodes above the leaves failed high on a bound; 8.86 M leaves estimated; 0.16% of 2.95 M keys valued in full;
    1.5% of the trees' 5.56 M and 2.0% of the probits' 4.69 M depth-1 replies valued in full.
- **Swap: probit-p3** (swap18.py; swap16.py's procedure): config.json's tournament binary replaced atomically at
  **14:17:10**, while cycle 234 was in its tournament (right after its calibration), without stopping the driver;
  **cycle 235 is the first on hybrid18** (own calibration games, tournament, self-games; training stays hybrid6).
  One line in driver.log; the old config.json and lines.csv in hybrid18/p3/config-before/.
  **Cycle 235 against a hybrid17 replay of it** (hybrid18/swap18.log; copies taken inside the phase windows,
  0.06-0.55 s each): the tournament replayed from tour/
  as it stood after cycle 234 with cycle 235's players and the logged command: 8 tour files and 2,912 saved-game
  files identical (728 games, 80,746 plies), stdout the same; the self-games: 104 game lines and 416 files
  identical; the 4 own calibrations cycle 235 made: identical. **SAME** (14:20:05).
  - A control first (swap18.py --check, 13:50-13:53, before the swap): the same snapshots and replays of cycle 203,
    hybrid17 against hybrid17: tournament (8 tour files, 2,912 saved-game files, 728 games), self-games (104 game
    lines, 416 files) and 4 own calibrations all SAME, so the check itself compares what it should.
  - **Measured live** (measure18.py on run_measured's footers; hybrid18/measure18.txt), 15 cycles before (220-234)
    against 12 after (236-247; cycle 235 also redid every calibration fit once, 67.2 CPU-s):

    | probit-p3, a cycle | hybrid17 | hybrid18 | Ratio |
    |---|---|---|---|
    | tournament, CPU-s | 100.8 (SD 2.1) | 63.3 (SD 1.4) | ×0.63 |
    | self-games, CPU-s | 14.9 | 9.2 | ×0.62 |
    | own calibration, CPU-s | 3.6 | 3.5 | depth 1, same code |
    | tournament wall, s | 37.7 | 21.4 | (the machine also got lighter as this fork's runs ended) |

    ~43 CPU-s a cycle saved (the review guessed ~53), the binary's ~119 → ~76 CPU-s; ~0.8 CPU-h from cycle 235 to
    the floor (300), ~2.0 CPU-h to the cap (400).
- **Use:** in the Queued section's list (from 14:17).
- **Left:** specs/arenas/make_ladder.py (HYBRID17) and the existing specs still name hybrid17; running arenas keep
  their binary. Switching an arena or league to hybrid18 refits each calibration once (the fit cache keys on the
  binary's bytes) and arena.py remakes own games once (it keys on the binary's md5).
- **Scratch** fastbin/hybrid18/ (field/, st/, p3/, p3-rehearsal/, bench/) and hybrid18-src/target: deletable, the
  user's call.

### 2026-11-12 ~16:40 chessai-nocells (a TOOL, not a speedup; worker fork for the user's "turn off path parameter passing during play" question)

- **What** (fastbin/nocells-src, a copy of grow6-src): `chessai-nocells neutral-cells IN OUT [--use-probs] [--identity]`
  writes a copy of a BDD model file with every path cell play-neutral, exactly treeview's "without" (each node's
  classifier alone): `PathTable::neutralize` (pathctx.rs: LR weights and AdaGrad sums 0, so offsets are exactly 0;
  CNBP counts and totals 0, so factors are exactly 1; probit means and variances 0, so 0 is added to mu and s2),
  `Bdd::neutralize_cells` (bdd.rs: every node's table and snapshot table), and the subcommand (main.rs). The cells
  stay because `pathctx::check` requires them on load; a probit's init_var stays. Model files only (play needs no
  state). It refuses to overwrite OUT. `--identity` only loads and saves. Installed as **bin/chessai-nocells, md5
  b39fb3f8cb62f81ff69fbda5e7dfc037** (cp -n). rs/ is untouched.
- **Checks** (models/from-scratch/nocells/survey.py; checks/survey.json): 27 ctx DAGs, made up of the library's 25
  ctx rows (errdag4, grow3, hybrid15 formats) and grow-dag-p1's two live lines at v414 (grow6).
  - `--identity` is byte-identical on all 27.
  - chessai-treeview2 path-detail on each neutral copy: the log-loss sum is bit-equal to the original's ll0. No leaf
    moves and no routing step mismatches.
  - The originals' md5s are unchanged.
  - The 8 arena DAGs' neutral copies load and play in chessai-hybrid16 at depths 1 and 2.
- **Use:** put the copy in experiment layout: models/from-scratch/nocells/EXP/ with config.json and lines.csv copied
  from the experiment, and runs/LINE/bddV.model. An arena then takes it as `/abs/.../nocells/EXP:LINE@V`, with the
  experiment's own calibration settings and own-games binary.
- **Scratch:** models/from-scratch/nocells/ and nocells-src/target. Both are deletable, but that is the user's call.

### 2026-11-12 ~18:50 review (scheduled)

**Running and changed since ~12:50.**
- Running (18:50-19:05): std-v1-p1-s1 and -s2 (ply 1, 8 lines each incl. two grow6 DAGs, rate_every 4, draw-aware
  crowning), cycles 407 and 413; hybrid19 games, hybrid6/grow6 training. Latest crownings s1 312, s2 340.
- New binaries: hybrid18 (14:17), hybrid19 (= hybrid18 + openings, 15:21), nocells (a tool, 16:40), play1 (17:26,
  hybrid19 + a serve loop), grow7 (18:40, the warm rung's trainer), play2 (deep mode, another fork's; not touched).
- Queued:
  - the ply-2 rung: std-v1-p2-s1/-s2 warm + -x3 scratch (drafts), hybrid19 games, ~8-18 CPU-h by its draft;
  - probit-p4w (ply 4, on hold): F-P x2 and T-P-Ne x2 from probit-p3 v301, D-P-Ne-ctx-grow x2 from
    leagues/starts/grow-dag-p1@395; hybrid19 games; ~0.68 CPU-h a cycle unrated, 0.94 rated; 41 cycles ~31 CPU-h,
    101 ~76 (ply4-prep's numbers).

**Measured.** Records (totals.csv, cycles.csv, runs/*.log, arenas' tour/game_cpu.csv), then short runs on reflinked
copies (nice 19, one thread, E-cores). Scratch: review-1112d/ (README.txt).

| Workload | CPU-s a cycle | Left |
|---|---|---|
| std-v1-p1 (each; last 24 cycles) | ~33: training 16.6-17.2 (the two DAG lines ~11.6), own cal 8.3-8.5, tournament process 4.4-4.5 (games 2.1-2.2), self-games 0.3, cold 0.3; wall ~15.5 s | ≤ ~190 cycles to the cap: ≤ ~1.8 CPU-h each |
| ply-2 rung (draft) | warm ~80 (tournament ~41, ~0.024 CPU-s a game) | ~8-18 CPU-h |
| probit-p4w (draft) | ~2,450 unrated; games ~2,340, of which games with a DAG ~80% | 31-76 CPU-h |

- grow6 DAG training (a std D-P line from v360, flat profile): Cnbp::class_products 37%, node_output_ctx 14%,
  train_weighted 7.6%, cnbp_factors 6.9%. errdag4's one-call-per-node-per-step cache is in place.
- The std DAGs' growth (the training logs' "path cells" lines): s1 D-L 46 / D-P 42, s2 D-L 48 / D-P 37 nodes at 71-83M
  positions (cycles ~400); D-L ~+0.12 a cycle, D-P ~+0.08. D-L reaches 64 nodes near cycle ~540 if the runs go that
  far, so the ply-2 rung's warm D-L lines start at ~50-63 and can cross 64 during the rung. grow-dag-p1's DAGs (looser
  router-split values) were 81-93 nodes at v599. Of the two grow-dag-p1@395 DAGs that probit-p4w starts from, #1 is
  over 64 now (hybrid19 plays it in float) and #2 under (it was 81 at v599 in grow-dag-p1).
- **compound's 64-node cap, measured.** chessai-hybrid19 against review-1112d/cap-src (hybrid19-src with
  `MAX_NODES = 256`, the only change). Game CPU a game, both sides:

  | Game | Ply | Games | hybrid19 | cap 256 | Ratio |
  |---|---|---|---|---|---|
  | gd300 (grow-dag-p1 D-P #1 @300, under 64) v F-P: CHESSAI_FX=direct against the default | 2 | 20 | 0.0471 (float) | 0.0243 | ×0.52 |
  | gd599a (#1 @599, 93 nodes) v F-P | 2 | 20 | 0.0640 | 0.0236 | ×0.37 |
  | gd599b (#2 @599, 81 nodes) v F-P | 2 | 20 | 0.0591 | 0.0219 | ×0.37 |
  | TBIG (121-node tree, hybrid18's field) v F-P | 2 | 20 | 0.0266 | 0.0153 | ×0.58 |
  | @395 #1 v F-P (T 0.1) | 4 | 4 | 4.25 | 1.65 | ×0.39 |
  | @395 #1 v #2 (T 0.1) | 4 | 2 | 4.60 | 2.32 | ×0.50 |
  | @395 #1 v #2 (T 0.1) | 1 | 4 | 0.0044 | 0.0047 | ~1 |

  - games.csv and ratings.csv byte-identical in every pair. CHESSAI_FX_CHECK=1 (incremental sums asserted equal to the
    full sums) passed on gd599a at ply 2. FX_STATS: float fallbacks 0 for every over-64 model, and the path-context
    DAGs stay inside the 4,096 route states.
  - Ply 1 gains nothing (the std ply-1 runs are unaffected either way).
  - The gain shrinks with size (the tree at 121 nodes ×0.58, both sides): child_into updates every node at each
    interior position, so the fixed-point cost grows with the node count.
- Ply-4 profile of the cap build (@395 #1 v F-P, 4 games): Compound::reply 17.1%, Map::fold 13.3% (probably
  child_into's all-node update, inlined; flat profile, unverified), Table::moved 12.8%, estimate_probit 9.6%,
  fastprobit::estimate 9.1%, Changes::fill 3.9%, has_legal_move 3.4%.

**Candidates, ranked.**
1. **Raise compound::MAX_NODES from 64 to 128.** This is item 2 of the ~01:00 review, held as conditional
   ("measure first: fixed-point play at 64-256 nodes may stop paying"). It is now measured, and its condition is
   met: probit-p4w starts with one DAG over the cap, the ply-2 rung's DAG lines may cross it, and the library's
   grown DAGs are over it.
   - Where: hybrid19-src/src/compound.rs:40 (the constant). It is used only in Tables::new's eligibility check
     (:263). Nothing else is sized by it: the u128 masks are over the 70 features, node indices are usize, and route
     states have their own cap.
   - Saving:
     - probit-p4w: @395 #1's 240 games a cycle at ×0.39-0.50, ~550-670 CPU-s a cycle (~25%). That is ~6-8 CPU-h
       to the floor (41 cycles) and ~15-19 to the cap (101). It is more once #2 passes 64, plus #1's self-games and
       its rated-cycle cold games.
     - Ply-2 rung: each DAG line over 64 costs ~0.040 CPU-s more a side, over ~400 sides a cycle. That is ~16 CPU-s
       a cycle, ~20% of a warm cycle. It only applies if the D-L lines cross 64: ~0-1.5 CPU-h.
     - Arenas and reads on library DAGs over 64 nodes: DAG games ×0.37 at ply 2, ×0.4-0.5 at ply 4.
     - The Play page: play1 is hybrid19 plus a serve loop, so a DAG over 64 nodes thinks ×~0.4 at plies 2+. That is
       latency, not CPU-h. play2's owner should carry the constant.
   - Why 128 rather than 256: 128 covers everything in sight (the std DAGs stay under ~100 through the ply-2 rung;
     the grow-dag-p1 DAGs are 93 at v599; TBIG is 121). The gain at 121 nodes is already smaller. Go past 128 only
     after a measurement at ~200+ nodes.
   - Exactness: the fixed-point bounds and their float fallback are per decision and don't depend on the node count.
     Build hybrid20-src as hybrid19-src with the one constant. Prove it with:
     - byte-identical tournaments against hybrid19 at plies 1-5 (T 0.1, T 0.02, argmax, openings on and off), on a
       field with the over-64 DAGs (gd599a/b, @395 #1), TBIG, cell DAGs and F-P;
     - CHESSAI_FX_CHECK=1 at plies 2-4;
     - self-games at plies 1-4;
     - the unit tests, plus one with a 65-128-node graph: Tables::new returns Some, and its values are bit-equal to
       float.
   - Deploy: install bin/chessai-hybrid20 (cp -n). Point the ply-2 drafts' and probit-p4w's tournament binary at it
     before they launch. No running experiment needs a swap.
   - Cost and risk: the build took 1.5 CPU-min here; identity runs ~10-20 CPU-min. Memory grows by 32 B a node per
     search depth. Risk is low.
   - **Exact; worth it:** ~25% of probit-p4w and 6-19 CPU-h if it launches; ~20% of a ply-2 warm cycle per DAG line
     over the cap.
2. **Lazy compound updates at interior search nodes.** Conditional; ply 3+ with DAGs.
   - Map::fold is 13% of the cap build's ply-4 DAG profile, if it is child_into's update of every node: leaf routes
     read only one path's nodes.
   - Updating a node's sums only when a descendant's route reads them might save up to half of that, ~6% of DAG
     games. On probit-p4w that is ~1.5-4 CPU-h, if it runs to its cap.
   - Exactness: the same integer sums, so the games are identical.
   - It is a moderate build (per-node staleness, recompute along the path), and the gain is uncertain because the
     profile is flat.
   - Do it only if probit-p4w launches and passes its floor. Measure first: count the node updates whose sums are
     never read.
3. **Vectorise Cnbp::class_products across the 3 classes (DAG training).**
   - It is ~13% of a std ply-1 cycle (37% of the DAG lines' ~11.6 CPU-s).
   - A ×1.5 would save ~4%: ~0.15 CPU-h left at ply 1, ~0.15-0.3 on the ply-2 rung.
   - Exact (lane-wise, with the same operations in the same order per class), but below the bar. Not built.

**Checked and dropped.**
- Own calibration (8.3-8.5 CPU-s a std cycle, ~25%): own games at depth 1. Its exact parts were done at 06:45 and by
  hybrid17; fewer own games is a design change (noted at 12:50).
- Tournament process overhead (~2.3 CPU-s a cycle: startup, calibration loads, rating fits, tables; 2.38 s for a
  zero-game run on 11-11): ≤ ~0.25 CPU-h over what's left of std-p1, ~3% of a ply-2 warm cycle. Known, small.
- Self-games and cold games at ply 1 (~0.3 CPU-s a cycle each): small.
- DAG shadows in grow6 (dropped at 12:50): still ~30% of a DAG line's training, and under the bar for the same reason.
- The std ply-1 runs: nothing exact reaches 5% except class_products (~4%). The 64-node cap doesn't touch ply 1.

### 2026-11-12 ~19:05 chessai-hybrid20 (implementation fork; candidate 1 of the ~18:50 review)

- **What:** fastbin/hybrid20-src is a reflink copy of hybrid19-src. hybrid20/hybrid20.patch is the whole diff:
  compound.rs's MAX_NODES 64 -> 128, its doc comment, and one unit test. Installed as **bin/chessai-hybrid20, md5
  cda3d7c59dd98feb431fea23a29f867c** (cp -n, 19:09:25). `cargo build --release` in hybrid20-src rebuilds it byte
  for byte. rs/ is untouched.
- **Why the cap was 64:** no note gives a reason.
  - hybrid5 set it without comment. hybrid15's summary and this ledger call it "hybrid5's cap: exact but at
    hybrid14's speed", and the ~01:00 review said "measure first". It was a speed guard, sized when the models had
    at most 25 nodes.
  - In the code, MAX_NODES appears only in Tables::new's eligibility check, and nothing depends on the node count:
    - Each node has its own table, Sums, margin and error bound.
    - A sum has at most 71 entries, each |w| < 2^20 at 2^-32 units, so it stays < 2^59 and can't overflow an i64.
    - Cell offsets are bounded per state. Route states have their own cap (4,096, u32 ids).
    - Acc and Tables are Vecs, child indices are usize, and Changes is per feature.
    - Fallbacks are decided per router decision.
  - So 128 is safe for all inputs, as any count would be. It stops at 128 because the gain shrinks with size: x0.58
    at 121 nodes. Measure before going past it.
  - Erratum: the doc comment says "each under 2^20 units of 2^-32"; read "each under 2^20 (2^52 units of 2^-32)". It
    is left as is, because a test-only edit to compound.rs already changed the build's md5 (checked).
- **Identity** (hybrid20/identity/: identity.sh, compare.py, compare.txt): 28 cases, all byte-identical to
  hybrid19. That is 3,467 games and 3,796 files; only game_cpu.csv's CPU and stdout's times differ.
  - Players over 64 nodes: grow-dag-p1's finals gd599a/b (93, 81), probit-p4w's start DAG @395 #1 (76), the
    library's largest DAG (84) and TBIG (a 121-node tree). The D-P DAGs have CNBP routers, probit leaves and cells;
    TBIG has CNBP routers and CNBP leaves.
  - Players under 64: @395 #2 (51), gd300 (56), the library's largest tree (40), a cell DAG with LR leaves (25), an
    LR-leaf tree. Flat: F-P and F-L.
  - Tournaments at plies 1-5:
    - T 0.1, T 0.02 and argmax with 8 random opening plies;
    - T 0.1 with sampled openings;
    - games saved at plies 1-4.
  - Self-games at plies 1-4, at T 0.1 and T 0.02.
  - Own calibration games at depths 1 and 2.
  - Check mode: h20 under CHESSAI_FX_CHECK=1 at plies 1-4, 74 games, no assertion failed. The counters show 0.47-24.5
    M router decisions a field, with float fallbacks 0, 0, 0 and 2.
  - Unit tests: 207 lib + 2 main. The new one, graphs_up_to_the_cap_value_exactly, uses a 121-node tree with LR
    routers, bit-equal at depths 1-4; Tables::new takes 127 nodes and refuses 130.
  - Test-only extras, in hybrid20/over64-dag-test.patch (applied, run, reverted), cover every router and leaf kind
    past 64 nodes:
    - cell DAGs with CNBP routers and LR or probit leaves, at 70 and 67 nodes;
    - a 121-node tree with LR, CNBP and TAN routers and LR, probit and CNBP leaves.
    They also pass under check mode.
- **Saving** (the identity runs' paired game CPU of games with a player over 64 nodes; RAYON 2, nice 19, E-cores):

  | Ply | T 0.1 | T 0.02 | argmax |
  |---|---|---|---|
  | 1 | x0.94 | x0.79 | x1.07 (~1, noise at ~3 ms a game) |
  | 2 | x0.39 (180 games) | x0.41 (180) | x1.44 (180) |
  | 3 | x0.76 (30) | x0.61 (30) | x0.53 (30) |
  | 4 | x0.39 (18) | x0.41 (18) | x0.58 (18) |
  | 5 | x0.51 (6) | | |

  - Self-games: x0.69-0.72 at ply 2, x0.65 at ply 3, x0.34 at ply 4.
  - Own calibration: x0.37 at depth 2. Depth 1 is ~unchanged.
  - The review's one-thread pinned measurements (x0.37 at ply 2 for 81-93 nodes, x0.39-0.50 at ply 4) agree.
  - Argmax at ply 2 is slower because compound's depth-2 path values every root move (Search::windowable is false
    for it), while hybrid13's float root windows cut most of them at argmax. Under-64 models already pay this in
    hybrid19: the default costs x1.22 against CHESSAI_FX=direct on an under-64 field (hybrid20/identity/extra/).
    Nothing queued plays argmax at ply 2.
  - Expected: probit-p4w ~550-670 CPU-s a cycle (~25%), ~6-19 CPU-h if it launches, more once @395 #2 passes 64;
    the ply-2 rung ~16 CPU-s a cycle for each DAG line past 64.
- **Use:** new specs and arenas whose trees or DAGs may pass 64 nodes should name bin/chessai-hybrid20 as the
  tournament binary. It is hybrid19 for everything else.
  - Done for leagues/specs/standard/std-v1-p2-{s1,s2,x3}.DRAFT.json and leagues/specs/ply4/probit-p4w.DRAFT.json
    (backups *.before-hybrid20; only the binary and one description clause changed).
  - std-v1-p2/prep/make_drafts.py copies the ply-1 rung's hybrid19: re-point the drafts if it regenerates them.
  - Switching an arena refits each calibration once and remakes own games once (both caches key on the binary).
- **Left / next review:**
  - (a) Root windows in compound's and fastprobit's depth-2 path at argmax: x1.2-1.4 on argmax games at ply 2. Not
    queued, because nothing plays argmax at ply 2.
  - (b) Past 128 nodes, measure first.
  - (c) No real model over 64 nodes has LR leaves yet (the std D-L lines are at 46-48), so the unit tests cover them.
- **Scratch:** hybrid20/identity/ (64 MB of runs; models and calibration sets reflinked), hybrid20/sizes/ and
  hybrid20-src/target. All deletable, the user's call.

### 2026-11-13 ~01:00 review (scheduled)

**Running and changed since 11-12 ~18:50.**
- Running (00:48): explore-t015-p1 (243/301) and probit-field-p1 (256/301), ply 1, hybrid19 games, hybrid6/grow6
  training; residual-p2 (134/401, ply 2, hybrid20 games, hybrid6 + hybrid6rp residual leaves); std-v1-p2-x3
  (192/601, ply 2 scratch, hybrid20, grow7); std-v1-p1-s2 (675/801, extended); flat-p3-w4 (153/200, four F-L at
  ply 4, hybrid11, extended); labels-p1-s1/-s2 (~295/401, two-line ply-1 fields, hybrid20, F-P with
  --label-depth 2 labels on hybrid6); s7-p1-s1/-s2 (~195/401, chessai-pairs2: pair models play the float path).
- Queued: std-v1-p2-s1/-s2 extension 101 -> 301 (models/from-scratch/batch7-reads/extend-when-free.sh, fires when
  at most 8 runs are live, ~when explore/probit-field end); read arenas (batch7-reads/*.sh, plies 1-2, 20-52
  entries, 20-30 round-robins; stdp2 final).
- New binaries: chessai-hybrid6rp (residual leaves, training only; plays as plain probit trees), chessai-pairs1/-2
  (S7; pair models float-only), chessai-play2 (another fork's). New recipe F-P-S7v2.

**Measured** (records: totals.csv last 40 cycles, tour/game_cpu.csv last 60 tournaments, runs/*.log; then short
runs on reflinked copies, nice 19, one thread, E-cores; scratch review-1113a/, README.txt).

| Workload | CPU-s a cycle | Left |
|---|---|---|
| std-v1-p2-s1 / -s2 extension (queued) | ~97 / ~87: games 54 / 44, training 21 (DAG lines 15), own cal 12, cold 5, self 4 | 200 cycles each: ~5.4 + 4.8 CPU-h |
| std-v1-p2-x3 | ~62: games 33, training 15 (DAG lines 9.6), own cal 8.4 | 108 (floor) - 409 (cap): 1.9-7.1 CPU-h |
| residual-p2 | ~54: games 33 (capped trees 71%, F-P 29%; nothing slow), training 7.3 | 166-267: 2.5-4.0 CPU-h |
| flat-p3-w4 | ~186: games 159 (F-L at ply 4, hybrid11), self-games 24 | <= 47: <= 2.4 CPU-h |
| std-v1-p1-s2 | ~51: training 30, own cal 13 | <= 126: <= 1.8 CPU-h |
| explore / probit-field | ~36 / ~34 | ~58 / ~45: ~1 CPU-h together |
| labels-p1 (each) | ~7.4 (training incl. the depth-2 labels 3.6 for both lines) | ~105: ~0.2 CPU-h |
| s7-p1 (each) | ~6.4 (games 0.7) | ~205: ~0.36 CPU-h |
| read arenas | stdp2 early: 53,040 ply-2 games, 0.35 CPU-h of game CPU | ~6-8 more reads: ~2-4 CPU-h |

- **The warm runs' D-P lines play ENTIRELY IN FLOAT.** In std-v1-p2-s1, games with its D-P are 55% of game CPU
  (s2: 47%): DP v F-P 59-80 ms a game against ~15-30 for every other pairing. FX_STATS on DP v FP: 0 compound
  searches. A diagnostic copy (review-1113a/diag-src: hybrid20-src + CHESSAI_FX_WHY prints) shows why: Tables::cells
  refuses a DAG past MAX_STATES = 4,096 route states. s1's D-P v100 (82 nodes, 6 sets, 312 cells) has 4,756; s2's
  (69 nodes, 9 sets, 319 cells) 6,601. The D-L lines (71 / 77 nodes) have 1,633 / 1,741 and play fixed point.
  Other live DAGs: std-v1-p1-s1 D-P v604 4,021 (just under), std-v1-p1-s2 D-P v681 3,442, x3 148 / 409,
  explore 304 / 436, probit-field 82 / 409.
  - perf, hybrid20, DP v FP at ply 2: Cnbp::probs_products3 39.5%, PathTable::cnbp_factors 6.5%,
    Graph::node_predict_path 5.2%, Probit::predict_with_cells 4.9%, Cnbp::predict_scaled 3.5% (the float DAG path);
    DL v FP: compound's fixed-point path (Table::moved 9.5%, Compound::reply 8.6%, probs_products3 10.6% for the one
    float valuation per position).
  - MAX_STATES only bounds the up-front enumeration (Tables::cells); states are looked up by index and the
    (node, marks) map, nothing is sized by the cap, ids are u32, and each state's margin is computed per state.
- **The read arenas at plies 1-2 still play chessai-hybrid16** (make_split_read.py's default; std-v1-p2-read-early,
  t015pf, labels: hybrid16), so DAGs over 64 nodes play in float there too (hybrid16 predates hybrid20's 128). In the
  early ply-2 read the four warm DAG entries (69-82 nodes) cost 53-67 ms a game played against 20-25 for the rest:
  ~20% of its game CPU.

**Candidates, ranked.**
1. **Raise compound::MAX_STATES from 4,096 to 65,536** (hybrid21 = hybrid20 + this constant, its doc comment and a
   unit test with a DAG past 4,096 states: s2's D-P v100 is a real one).
   - Measured with the diag build (review-1113a/diag-src, MAX_STATES 65,536; not installed): the field
     {s1 D-P, s1 F-P, s2 D-P, s2 F-P} at ply 2, 6 tournaments (72 games): games.csv and ratings.csv byte-identical to
     hybrid20; game CPU DP v FP x0.32-0.37, DP v DP x0.30, FP v FP x0.96 (noise), all x0.35. 200 depth-1 own games of
     s2's D-P: identical files, 1.84 -> 1.26 CPU-s. CHESSAI_FX_CHECK=1 (2 tournaments): 5.19M router decisions
     asserted against float, 0 fallbacks, no assertion. Both D-P load as "ok: 82 nodes, 4756 route states" and
     "ok: 69 nodes, 6601 route states".
   - Saving: the queued extension, ~23 CPU-s a cycle in s1 (~24%: 36% of games, plus cold games, self-games, own
     calibration) and ~17 in s2 (~20%): ~2.2 CPU-h over its 200 + 200 cycles, more as the states grow (they only
     grow). Plus every read and ladder arena with these D-P entries, and std-v1-p1-s1's D-P (4,021) once anything
     grows it. Latency on the Play page for such a DAG at plies 2+ (play1/play2 carry their own constant).
   - Exactness: the cap decides only eligibility; every decision keeps its per-state margin and float fallback, as
     hybrid20's MAX_NODES change (proved byte-identical in 28 cases). Prove as hybrid20: tournaments at plies 1-4
     (T 0.1, T 0.02, argmax; openings on and off) on a field with both over-cap D-P, under-cap DAGs, trees and flats;
     self-games; own calibration at depths 1-2; check mode at plies 1-4; unit tests; and a whole-cycle replay of
     std-v1-p2-s1's cycle 101 (the extension re-runs it against its saved original).
   - Deploy: install bin/chessai-hybrid21 (cp -n). Point std-v1-p2-s1/-s2's tournament binary at it before the
     extension fires (both runs are done; their config.json is read when a cycle starts), else between cycles
     (atomic config.json replace during a tournament phase, as errdag4 was swapped). x3 optional (no gain now).
   - Risk: low. Memory ~150 B a state (65,536 states ~10 MB a model), enumeration at load in milliseconds; a cap
     still bounds pathological DAGs.
   - **Exact; worth it (>= 1 CPU-h and ~20-24% of the extension's cycles).**
2. **make_split_read.py's default tournament binary at plies 1-2: hybrid16 -> hybrid21** (or hybrid20). The same
   games by the ledger's identity chain (hybrid16 -> 17 -> 18 -> 19 -> 20, each byte-identical); own-calibration games
   unchanged (they are each experiment's own binary's). DAGs over 64 nodes and over 4,096 states play fixed point
   in reads: ~13% of an stdp2-type read's game CPU (the warm DAG entries at x0.35-0.40), plus hybrid17's rating-fit
   step. ~0.05-0.1 CPU-h a read, ~0.2-0.4 over the queued reads: under the bar alone, a one-line change to bundle
   with 1 (calibration fits are redone once: the fit cache keys on the binary's bytes).
3. **flat-p3-w4 hybrid11 -> hybrid13+**: x0.95 at ply 4, T 0.1 (hybrid12's staged move; hybrid13's windows need
   T <= 0.05): ~0.1 CPU-h over <= 47 cycles. Under the bar (as the 11-11 06:50 review noted).

**Checked and dropped.**
- Cnbp::class_products vectorisation (DAG training, the 18:50 review's candidate 3): DAG training is ~15 CPU-s of a
  warm ply-2 cycle and ~9.6 of x3's, ~2.8 CPU-h left in all; x1.5 on its ~37% saves ~0.35 CPU-h. Still under the bar.
- Lazy leaf terminal checks at ply 4 (Queued, candidate 3 of 11-11 06:50): ~10% of flat-p3-w4's <= 2.4 CPU-h. Under.
- Search labels (labels-p1): the depth-2 labelling runs inside training, 3.6 CPU-s a cycle for both lines. Small.
- S7 pair models' float evaluation: ~0.7 CPU-s of games a ply-1 cycle; ~0.36 CPU-h left a run. Small unless S7
  passes its v300 read and moves to a ply-2 field (then: a fixed-point pairs term, measure first).
- residual-p2: games spread evenly (capped trees on compound, F-P on fastprobit), 20 ms a game; training 7.3 CPU-s.
  Nothing slow.
- Own calibration (8-12 CPU-s a ply-2 cycle): known; the over-cap D-P's depth-1 own games gain with candidate 1.

### 2026-11-13 01:28 chessai-hybrid21 (implementation fork; candidates 1 and 2 of the ~01:00 review)

- **Built** 01:05-01:07: hybrid21-src = hybrid20-src + compound::MAX_STATES 4,096 -> 65,536 (its doc comment, one
  unit test on the real 6,601-state DAG: testdata/dp6601.model). bin/chessai-hybrid21 md5 b52b1bac... (cp -n).
  208 + 2 unit tests pass.
- **Identity vs hybrid20** (hybrid21/identity/, hybrid20's suite + the two over-cap D-P DAGs): 31 cases, all
  byte-identical, ~5,070 games: tournaments at plies 1-5 (T 0.1, T 0.02, argmax; openings on and off), self-games at
  plies 1-4, own calibration at depths 1-2, check mode at plies 1-4 (178 M router decisions, 4 float fallbacks, no
  assertion). cal/heldout and cal/x re-made as reflinks of games/heldout-lr-d2-s555/round0 (hybrid20's were deleted by
  the 11-12 tier-1 cleanup).
- **Whole cycles**: std-v1-p2-s1's and -s2's cycle 101 replayed on hybrid21 from reflink snapshots under extend_check
  (field, games and cold-games rows, decisions, ratings = the hybrid20 originals): both PASSED; s1's replay on hybrid20
  as a control: all 8 trained v101 models and 6 states byte-identical between the two.
- **Read arena**: hybrid16 (make_split_read.py's old default) vs hybrid21, 6 entries at ply 2, 90 games: identical.
- **Speed**: games with an over-cap D-P x0.40 (ply 2 T 0.1), x0.49 (T 0.02), x0.40-0.52 (ply 3), x0.40-0.48 (ply 4),
  ply 1 x0.8-1.0, ply-2 argmax x1.09 (hybrid20's known compound-depth-2 effect; nothing queued plays argmax).
  s1's cycle 101: tournament 56.2 -> 38.7 CPU-s, cold 22.3 -> 15.8, self-games 4.4 -> 3.0: ~25 CPU-s a cycle.
- **Deployed**: std-v1-p2-s1 01:18:09 and -s2 01:23:09 (config.json's tournament binary, while done and idle, the
  extension waiter mid-sleep; swap21.py; backups hybrid21/swap/); their extension's cycle-101 re-run will check hybrid21
  against the originals live. make_split_read.py: plies 1-2 tournament binary hybrid21 (01:18:26, atomic; own
  calibration stays hybrid16; ply 3+ stays hybrid18 -- could take hybrid21 too, not done).
- Expected saving: ~2.2-2.5 CPU-h over the extension's 200 + 200 cycles, growing with the D-P state counts, plus
  ~13% of every read arena with these DAGs.

### 2026-11-13 ~06:53 review (scheduled)

**Running and changed since 11-13 ~01:00.** hybrid21 shipped (01:27). New binary chessai-treelab (grow7 +
--label-depth/--label-result/--label-model for trees and DAGs; tree-labels/src), moving labeller in leagues.py
(04:08), new recipes F-P-lab, T-P-Ne-cap, F-P-S7v3; chessai-pairs3/-pairs3f built, not launched. Running at 06:53:
labels-tree-p2-s2 (cycle 266; floor 300, cap 401; its seed-79 twin plateaued at 333), levers2-p2 (cycle 85/401, ply-2
F-P result vs depth-2 labels). labeldepth-p1-s1/-s2 ended (313 plateau, 401 cap). Queued reads (batch7-reads):
labels-tree-read2 v300 (20 entries, plies 2-3), labels-tree-read3 final (36 entries, version windows, plies 2-3),
levers2-read mid/final (24 entries, plies 2-3), labeldepth-read final (30-34 entries, plies 1-3).

**Measured** (totals.csv last 40 cycles; runs/*.log; tour/game_cpu.csv of tonight's reads; short runs on snapshots in
review-1113b/, nice 19, E-cores 4-7, one thread; perf user-space only here: perf_event_paranoid 2, no call graphs).

| Workload | CPU-s a cycle | Left |
|---|---|---|
| labels-tree-p2-s2 | ~71: training 37.5 (own-search tree labels ~25, F-P-lab 7.1, flat-labelled tree 7.7, result tree 0.7), tournament 15.7, cold 6.9, own cal 3.6, self 1.6 | 34-135 cycles: 0.7-2.7 CPU-h |
| levers2-p2 | ~27: tournament 11, training 6.1 (F-P depth-2 labels on the fx path), cold 6.0, self 2.5, own cal 1.3 | ~316: ~2.4 CPU-h |
| read arenas | ply 2 ~15-17 ms a game; ply 3: flats 46-69 ms, trees + flats 83 ms | ~2.3 CPU-h at ply 3, ~0.5 at plies 1-2 |

- **Tree search labels run the FLOAT negamax with OUTCOME, and every leaf pays two model calls.** selfplay::search_labels
  -> Search::expected_outcome: flats take negamax_fx and resolve(pv) (the PV leaf's outcome once), but a tree has no
  fixed_linear, so it takes the float negamax<OUTCOME=true>, whose depth-0 branch returns (self.eval(pos),
  self.outcome(pos)): eval = predict (log-odds), outcome = predict_log + Calibration::probs, each with its own encode
  and its own routing through the tree. Only the principal variation's leaf outcome is ever used.
  - Snapshot (labels-tree-p2-s2's own-search line, bdt255.state, cycle 256's 1,536-game set, 118,940 positions): labels
    23.34 s, train 0.81 s, one thread. perf: Cnbp::probs_products3 39.0%, Cnbp::predict 7.2%, Probit::predict 6.8%,
    Probit::predict_log 5.3%, log 5.2%, best_child 3.6%, probs_of_latent 3.2%, has_legal_move 2.8%, exp 2.6%,
    log_probs_of_latent 2.6%, encode 2.6%, Calibration::probs 1.3%, negamax 1.3%: ~85% model calls. The leaf-model
    symbols put outcome calls at ~0.78x the eval calls (eval also orders the depth-2 root's ~30 children), so outcome()
    is ~37% of the label search.
- **The flat-labelled tree and F-P-lab compute the same labels.** Both print the same calibration each cycle (cycle 330:
  3.360715,1.527212,-0.824041,-0.591740) on the same set with the same labeller version: 3.71 + 3.95 s of labels, and
  ~3 s each of load, parse and calibration fit.
- **Ply-3 flat probit play** (hybrid18, four levers-window-p3 entries with their cached own games, 144 games, 60.9 ms a
  game): fastprobit::estimate 32.8%, negamax_fx 11.2%, Acc::move_sums 8.7%, fastpath::has_legal_move 7.1%,
  Acc::child 4.0%, leaves_p 3.4%, gen_non_king 2.8%, Reply::of 2.8%. Nothing new beyond the Queued notes.

**Candidates, ranked.**
1. **Lazy principal-variation outcome in the float OUTCOME negamax** (tree and DAG search labels; any float self-play
   with outcomes): carry the PV (as negamax_fx does) and resolve the outcome once at its end, instead of outcome() at
   every leaf. Exact by construction (the search's control flow uses values only; the PV leaf's outcome is the same
   function of the same position; terminal results unchanged). Saving ~35% of a tree's label search: ~8 CPU-s a cycle
   of labels-tree-p2-s2 (~11%), but only 0.1-0.3 CPU-h over its 34-135 cycles left, which end (~15-60 min wall) before a
   build could land. Proof: training identity (models and states byte-identical) with labels on, for a capped tree, a
   grow tree, a cell DAG and a router-split DAG, own search and --label-model; flats untouched (fx path); unit tests.
   Risk: low. **Not worth it now; implement if the labels-tree final adopts search labels for trees**: then ~9 of ~26
   CPU-s a capped-tree line a ply-2 cycle and ~30 of ~90 per 200k positions for a big D-P DAG, on every labelled line.
2. **Share labels between lines with the same labeller, set and settings** (a content-addressed label cache, as
   hybrid16's calibration-fit cache: key = labeller bytes, set, depth, result weight, binary): saves one line's labels
   when a tree is labelled by a field line (~4-7 CPU-s a cycle here, ~7%). Exact (same computation). The lines train
   in parallel, so the second must wait for or find the first's entry. 0.1-0.25 CPU-h left: not worth it now;
   conditional on a design that keeps a flat-labelled tree.
3. **Ply-3 reads**: has_legal_move 7.1% at ply 3 (the Queued "lazy leaf terminal checks", noted for ply 4). At most
   ~5% of ~2.3 CPU-h queued: ~0.1 CPU-h. Under the bar.

**Checked and dropped.**
- Transposition reuse between consecutive positions' label searches: a depth-2 search's leaves are 2 plies down,
  the next position's 3 plies down (other side to move): no shared leaves. The next search's root-child ordering
  evals are this search's leaves, but only where alpha-beta reached them: small, and a cache costs per lookup.
- levers2-p2: cold games 6 of 27 CPU-s a cycle are the design (rating-only cold games), not overhead; nothing slow.
- Calibration fit inside labels (one evaluation a position): ~1% of the tree's label CPU.
- Recommendation: **nothing to start now.** Re-check candidate 1 when the labels-tree verdict is in (pooled final read).

### 2026-11-13 ~12:59 review (scheduled; scratch fastbin/review-1113c)

**Running.** Nothing: every experiment has ended. Workload under review (queued, pending the user): the ply-2 standard
from scratch (8 recipes, as std-v1-p2-x3) with all six tree/DAG lines (T-L-Ne and T-P-Ne capped, T-L-Ne-grow,
T-P-Ne-grow, D-L-Ne-ctx-grow, D-P-Ne-ctx-grow) trained by chessai-treelab on F-P labels (--label-model F-P, the moving
labeller; --label-depth 2 --label-result 0.5); flats unchanged; two seeds of ~400 cycles.

**Measured.**
- std-v1-p2-x3 (jobs 2, threads 4; totals.csv cycles 1-400, blanks as 0): 63.6 CPU-s a cycle, 7.06 CPU-h per 400
  cycles. Split: tournament process 33.5, training 15.9 (the two DAGs 8.0 + 5.9 of 19.7 at cycle 430), calibration 8.5
  (8 versions), cold games 3.4 (rate_every 4: ~13.5 a rated cycle), self-games 2.4. Training sets average 197,743
  positions (242k in cycles 1-100, ~182k after).
  - Correction to the notes behind this review: averaging cold_proc_cpu_s over its rated rows only had put x3 at
    74-80 CPU-s a cycle (9.0 CPU-h); 63.6 is right.
- The design's one new cost is labels, per labelled line a cycle:
  - Live, labels-tree-p2 cycles 300-332 (~118k positions; two runs at once on the 8-CPU box). F-P-lab took 6.91 CPU-s,
    nearly all labels. The flat-labelled capped tree took 7.65 against the result tree's 0.75. That is ~6.8-6.9
    CPU-s, ~57 us a position.
  - Idle machine (nice 19). Setup: review-1113b's snapshot (cycle 256's 1,536 games, 118,940 positions), its
    fplab.model, a probit trainee. Results:
    - labelled train: 2.96 CPU-s, of which labels 2.86 s at 1 thread; 1.42 s wall (2.85 CPU-s) at 2 threads;
    - plain train: 0.05 s;
    - ~25 us a position; parsing and replaying the games ~0.1 s.
    The live runs paid ~2.3x for contention.
  - At x3's 197.7k positions: 4.9 (idle) to 11.5 (busy) CPU-s a line a cycle.
- The labels are a pure function of a fixed set of inputs:
  - the binary and the CHESSAI_* environment (the process-wide switches are CHESSAI_* OnceLocks: FX, ORDER1,
    MOVESUMS, STAGED, fastprobit);
  - the labeller's kind, use_probs and bytes;
  - the round's games in read order;
  - depth and result weight.
  The computation is train_rounds_labelled → fit_calibration, then search_labels(records, labeller, calibration,
  depth, result_weight), and rayon's map/collect keeps the order. Thread count changes nothing: the labelled probit
  trained at 1 and 2 threads is byte-identical (sha256 2259667593e5f219...) and prints the same calibration.
- The six lines compute identical labels every cycle:
  - the moving labeller resolves to the version that played the cycle's games (label_line_args);
  - every line trains on the same trainsets/local (build_set), and trainsets/ is removed at cycle end.
- With an F-P labeller, search_labels takes fixed_linear() (Some for lr and probit) under CHESSAI_FX=on.
  expected_outcome is then negamax_fx + resolve(pv): the principal variation's outcome is already resolved once.

**Workload estimate (400 cycles a seed).**
- Per cycle: 63.6 + 6 × (4.9 to 11.5) = 93 to 133 CPU-s.
- That is 10.3 to 14.7 CPU-h a seed, 21 to 29 for two seeds. The busy end is likelier with two seeds running at once.
- Cost centres over two seeds, busy end:
  - labels: 15.3 CPU-h (52%);
  - tournament: 7.4 (25%);
  - training: 3.5 (12%);
  - calibration: 1.9 (6%);
  - cold games: 0.75 (2.5%);
  - self-games: 0.5 (2%).

**Candidates, ranked.**
1. **Shared label cache**, review-1113b's candidate 2, now worth it.
   - Where: chessai-treelab's train_rounds_labelled, in a new binary built from tree-labels/src. Only with
     --label-model; own-search labels belong to the trainee and are never shared.
   - Key: 2×XXH64, porting hybrid18-src/src/calcache.rs (xxh64, Digest, environment(), exe()). It covers:
     - the binary, and the CHESSAI_* environment less the cache's own switch;
     - the labeller's kind, use_probs and file bytes;
     - every game record of the round in read order (moves, positions, result);
     - depth and result-weight bits.
   - Entry: the full key, the calibration, the labels (f64, ~4.7 MB at 197.7k positions) and a checksum. It is
     written to a unique temp file, then renamed into place.
   - Location: <games>/.labelcache/, i.e. the driver's trainsets/local. That directory is rebuilt each cycle and
     removed at its end, so pruning is free and the driver needs no change. read_round_records reads round0/ only.
     CHESSAI_LABELCACHE=off disables the cache.
   - Locking: the driver trains two lines at once (jobs 2). The miss path takes an exclusive std File::lock (Rust 1.94)
     on <key>.lock, re-checks, computes, writes and unlocks; the waiting lines then read the entry. The kernel drops
     the lock if its holder dies. Without the lock, the first two lines would both compute (4 of 6 saved, not 5).
   - Saving: 5 of the 6 label computations. That is 24-58 CPU-s a cycle: 2.7-6.4 CPU-h a seed, 5.4-12.8 CPU-h over
     two seeds, 26-43% of the workload. The busy end is likelier, since the one computation left meets less
     contention. Wall: about 10 s of a 60-70 s cycle, since six label computations (~4.4 s each, two at a time)
     become one.
   - Exactness by construction: the same deterministic function of the hashed inputs, with the full key compared.
     Proof:
     - training identity (model and state bytes) for all six kinds with --label-model probit:<F-P>: cache off vs a
       cold miss vs a warm hit. Run it on review-1113b's snapshot and 2-3 x3-sized sets;
     - six processes started together: one computes, five hit, and all are byte-identical to cache-off;
     - unit tests: each key part changes the key; truncated, corrupt or wrong-key entries are recomputed;
     - a 5-cycle driver run of the planned spec on both binaries, with identical lines.csv and model sha256s.
   - Risk: low. The one way to get it wrong is a label input left out of the key. The six-kind identity test and the
     binary and environment in the key cover it. Disk: ~5 MB a cycle per experiment, removed with trainsets/.
   - **Implement if the user approves the distillation run**, before its launch (a ~1-2 h fork).
2. **Lazy PV outcome in the float OUTCOME negamax**, review-1113b's candidate 1. It does not apply here: the F-P
   labeller already takes the fx path. Only own-search tree and DAG labels call outcome() at every leaf, so it stays
   conditional on a design with own-search tree labels.

**Checked and dropped.**
- Game parsing and replay in the labelled path (read_round_records): ~0.1 of 2.96 s. Nothing to share beyond the
  labels.
- Sharing across cycles: none. The labeller moves every cycle and the set is new.
- The labeller line: F-P trains on results in this design, so it computes no labels.
- Calibration fits (8.5 CPU-s a cycle): one per new version on its own games. Nothing is shared, and hybrid16's fit
  cache already skips repeats.
- Cold games (3.4 a cycle, 3.3 of it in-game) and the tournament's process overhead (2.3 of 33.5 a cycle, ~0.5 CPU-h
  over two seeds): design cost, or under the bar.
- DAG training (~14 CPU-s a cycle late in x3): covered by errdag4, hybrid15 and grow3; not re-profiled.


### 2026-11-13 ~18:53 review (scheduled; scratch fastbin/review-1113d, 1 MB)

**Running and queued (18:47).**
- std-v1-p2-s1, the extension to cap 601: at cycle 446. Late cycles (380+) cost 84.3 CPU-s each: tournament process
  42.3, training 22.7, calibration 12.1, cold games 4.3, self-games 2.9. That leaves at most 3.6 CPU-h (155
  cycles), less if the plateau rule ends it (last crown 437, window 50). std-v1-p2-s2 ended at cycle 397 on its
  plateau (18:19).
- Distillation reads at ply 3 (hybrid18, T 0.02, 30 RR):
  - distill-p3-mid: done (110k games, 2.7 CPU-h, 88 ms a game).
  - distill-p3-final: 60,180 of 109,800 games, ~100 ms a game, ~1.4 CPU-h left.
  - distill-x3-p3-mid: started 18:26; distill-x3-p3-final still to come.
  - About 3.5 CPU-h of ply-3 games left. Process CPU is game CPU + ~3% (fits and reports).
- Queued, the main workload: the Leela-data experiment (fork under models/from-scratch/leela/; not touched). Its
  inputs are 42 test60 tars (~17 GB) and 3 held-out tars, downloading now, with a Rust converter (src/). Its
  training mode is not built yet. My estimate:
  - conversion 0.5-1 CPU-h;
  - training 1-3 CPU-h (the DAG trained at ~20 us a position in distill-p2, 3.4 s for 174k positions; more if the
    unbounded DAG grows large);
  - reads at plies 1-3 plus the library arenas: ~3-6 CPU-h, mostly ply-3 games at ~0.1 s each.
  The cleanup is deletion only.

**Measured / checked.**
- Sizes of the biggest current grow DAGs (`leaves`): x3 D-P v436 has 17 leaves (max node index 36); distill s2 D-P
  v448 has 11 (30); distill s1 D-P v312 has 13 (28). All are under hybrid18's 64-node cap.
- hybrid18 vs hybrid21 at ply 3 (read flags, RAYON 2, nice 19, loaded machine). The field was the five biggest
  DAG/tree finals of distill-p3-final (dpb448, dlb448, dpa312, gpb448, fpb448).
  - Pairs: one pair of 20 games and three alternating pairs of 40.
  - games.csv was byte-identical in all four pairs.
  - CHESSAI_FX_STATS: every search compound, 0 float fallbacks.
  - Game CPU, the three alternating pairs: 6.12/5.82/5.56 against 6.28/5.54/5.42 CPU-s. That is equal within noise:
    for models under 64 nodes and 4,096 route states the two binaries play the same games at the same speed.

**Candidates, ranked.**
1. **Ply-3 reads on hybrid21 instead of hybrid18, only for models past 64 nodes or 4,096 route states.**
   - Exactness: the identity chain hybrid18 -> 19 -> 20 -> 21 at plies 1-5, plus today's four identical pairs.
   - Gain: x0.61 at ply 3, T 0.02 (hybrid20's table) on games with a 65-128-node model. Today's DAGs are smaller,
     so they gain nothing.
   - Where it applies: the Leela DAG if it grows past 64 nodes, and library DAGs/trees of 76-121 nodes if the
     library-ranking arenas pick them (grow-dag-p1 finals 81/93, the library's largest DAG 84, probit-p4w's start
     DAG 76).
   - Estimate: 0.1-0.5 CPU-h over the Leela reads. That is UNDER THE BAR.
   - It needs no build: make_split_read.py's --binary option for the Leela fork's ply-3 arenas, or the ply->=3
     default changed atomically between arena creations. Cost: one calibration-fit refit per model (the fit cache
     keys on the binary).
   - Not started.
2. **Conditional: the Leela DAG past 128 nodes or 65,536 route states.** Both binaries would then play it in float.
   - Measure first: when the 5M/20M snapshots exist, play a mini ply-3 tournament with CHESSAI_FX_STATS=1, which
     prints the fallbacks.
   - If it falls back: raise compound::MAX_NODES / MAX_STATES. That is exact by hybrid20's argument: nothing in the
     fixed-point path depends on the counts. Measure the speed ratio at the real size first, since the gain shrinks
     with size (x0.58 at 121 nodes).
3. **Conditional: Leela DAG training cost at scale.**
   - Measure positions a second from the 1M/5M snapshot runs.
   - Profile only if what's left in DAG training exceeds ~1 CPU-h and the growth machinery (router trials, split
     windows) dominates. Training is already covered by errdag4, hybrid15 and grow3.

**Checked and dropped.**
- The extension's cost centres: all covered by earlier reviews (hybrid21's states cap, calibration-fit cache, label
  cache n/a). At most 3.6 CPU-h left and it may plateau sooner.
- Tournament fits and reports in the ply-3 arenas: ~3% of process CPU.
- The Leela conversion: Rust, ~0.5-1 CPU-h, the Leela fork's design.
- Swapping binaries inside the running distill arenas: not possible mid-tournament, and no gain at these model sizes.

**Verdict: nothing worth an implementation fork now.** Candidate 1 is free to take for the Leela reads; 2 and 3 wait
on the first snapshots.
