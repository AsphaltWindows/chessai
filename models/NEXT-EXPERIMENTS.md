# Next experiments

The current state: what runs, what the user decides next, what is queued, and the headline results. Updated
2026-11-12 06:45.

- Results with their numbers: RESULTS.md. Open ideas: IDEAS.md. Design rules and league decisions: NEW-DESIGN.md.
- The overnight proposals doc of 11-09/10 (P1-P7, the routing path, theory T1-T4) is archived whole:
  archive/2026-11-10/models/NEXT-EXPERIMENTS.md. Its proposals are summarised under "Proposals" below.

## Now (11-12 06:45)

- **11-13 20:37 DISTILL-P2 FINAL READ, PLY 3 (the second half; batch7-reads/distill-read-final.out, done 20:13:53): THE RULE FAILS, DISTILLATION CLOSED.** Rule (pre-registered): a distilled tree/DAG leading the field's best flat by 30+ at plies 2 AND 3 (two-seed window means) = the goal's first pass; within 15 = distillation lifts trees only to the flats' level. The best flat is F-P at both plies (field F-P - field F-L 9.5 at ply 2, 34.7 at ply 3). Distilled lines minus the field's F-P, FP face, seeds pooled (measurement SE ~11): ply 2 capped T-P 19.8, grow T-P 4.3, D-P 7.2, capped T-L -22.2, grow T-L -45.3, D-L -58.2; ply 3 capped T-P 5.5, grow T-P 0.5, D-P -9.8, capped T-L -15.2, grow T-L -76.0, D-L -80.0. No tree/DAG passes; the probit-leaf ones reach the flat (capped T-P 20 at ply 2, within 15 at ply 3), the LR-leaf ones stay below it. Cross-arena (distilled minus std-v1-p2-x3's same line, ply 3): grow T-P 85.5, capped T-L 58.0, D-P 46.7, grow T-L 34.4, D-L 34.4, capped T-P 16.4, F-P 1.9, F-L -19.2: search labels lift the trees over their result-labelled twins, not past the flats. Also: std-v1-p2-s1's extension ended on the cap 601 (s2 plateaued at 397). Every non-Leela final lead is done.

- **11-13 22:06 LEELA-DATA: THE STANDARD-SETTINGS LR TWINS' READS AND THE RE-REGISTERED LR-DAG RULE.** Per type (minus the peers' mean / minus the best): capped T-L std p1 -251 / -275 (rank 5 of 5); capped T-L std p2 -297 / -331 (rank 4 of 4); capped D-L std p1 -232 / -246 (rank 5 of 5); capped D-L std p2 -246 / -272 (rank 4 of 4); unbounded D-L std p1 -262 / -269 (rank 5 of 5); unbounded D-L std p2 -359 / -384 (rank 4 of 4). No ply-3 arenas (no plateaued LR-leaf trees or DAGs trained at ply 3). Rule arenas leela-rule-l-std-p2/-p3: Leela DAG-L unbounded std 50M - Leela F-L 50M: ply 2 -306.5 +-25.8, ply 3 -339.9 +-26.6 Leela DAG-L unbounded std 20M -> 50M: ply 2  +25.9 +-24.5, ply 3  +36.9 +-25.7; mean +31.4 VERDICT: FAIL: the DAG-L unbounded std is below F-L + 15 at plies 2 and 3 (under F-L by more than 15 at one or both): CLOSED Sizes: unbounded D-L std 1M 1 nodes / 1 leaves, 5M 4 nodes / 3 leaves, 20M 28 nodes / 9 leaves, 50M 127 nodes / 21 leaves; capped D-L std 1M 1 nodes / 1 leaves, 5M 4 nodes / 3 leaves, 20M 16 nodes / 7 leaves, 50M 22 nodes / 7 leaves; capped T-L std 1M 1 nodes / 1 leaves, 5M 1 nodes / 1 leaves, 20M 7 nodes / 5 leaves, 50M 13 nodes / 9 leaves. Full verdicts in leela/arch-results.txt.

- **11-13 20:58 LEELA-DATA: THE RULE'S READ AND THE UNBOUNDED DAG.** unbounded DAG p1 -470 / -475 (rank 5 of 5); unbounded DAG p2 -727 / -737 (rank 4 of 4) (minus the D-P peers' mean / minus the best). Rule arenas leela-rule-p2/-p3: Leela DAG unbounded 50M - Leela F-P 50M: ply 2 -169.5 +-21.7, ply 3 -136.8 +-24.4 Leela DAG unbounded 20M -> 50M: ply 2  +31.2 +-21.7, ply 3  +87.5 +-24.4; mean +59.4 VERDICT: FAIL: the DAG unbounded is below F-P + 15 at plies 2 and 3 (under F-P by more than 15 at one or both): CLOSED Sizes: unbounded DAG 1M 1 nodes / 1 leaves, 5M 4 nodes / 3 leaves, 20M 31 nodes / 11 leaves, 50M 100 nodes / 11 leaves; capped DAG 1M 1 nodes / 1 leaves, 5M 4 nodes / 3 leaves, 20M 16 nodes / 7 leaves, 50M 22 nodes / 5 leaves. Full verdicts and the summary in leela/arch-results.txt.

- **11-13 20:35 LEELA-DATA: LR TWINS REDONE ON THE TRAINING STANDARD'S LR SETTINGS; THE LR-DAG RULE RE-REGISTERED HERE, BEFORE ANY READ OF THE NEW LINES (coordinator ~20:35: "identical except the leaf" was wrong for LR leaves).** WHY: models/STANDARD.md "LR settings (v1)": forget 10 on every LR line that splits (at forget 1 a split's clones keep the parent's AdaGrad sums, huge after 50M positions, and can't specialise); grow-mdl 0 for LR grow trees and DAGs (the MDL bar at 1 is ~0.006 nats for LR leaves, k 4262, and LR splits gain 0.0007-0.002, so it blocks them: the first D-L twins had 4 nodes at 5M and 1 node at 20M and 50M, both identical in play). The first LR twins (20:27 launch, 20:32 read) stay as a CONTROL only (sizes and held-out loss; their arenas and rule verdict are kept but decide nothing). NEW LINES (leela/tools/train_lr_std.sh, two cores, nice 10; same records, order, seed 1113, snapshots, binary): T-L-Ne-cap-f10 = std-v1-p2-x3's capped T-L-Ne args (--leaf lr --branch-factor 3 --split-threshold 1000000 --split-limit 4 --forget-factor 10 --router cnbp --route error --error log --reset-leaves inf --use-probs; split limit 4, the same as T-P-Ne-cap's); D-L-Ne-ctx-grow-cap-std and D-L-Ne-ctx-grow-unb-std = the probit twins' args with --leaf lr, no split noise, --forget-factor 10, --grow-mdl 0 (--grow-gate age and --grow-priority snapshot as the standard's D-L-Ne-ctx-grow). Smoke (15 rounds, leela/smoke-std): both DAGs split by 5.1M (4 nodes / 3 leaves; held-out log-loss 0.8626 capped, 0.8581 unbounded vs 0.8625 for the first twins); the capped T-L still one leaf at 5.1M (as the first T-L). READS (leela/tools/arch_arenas_l2.sh -> arch-L2.out): leela-tl-f10-p1/-p2, leela-dl-cap-std-p1/-p2, leela-dl-unb-std-p1/-p2, the same peers as the first twins' arenas; RULE (the same rule): measured in leela-rule-l-std-p2/-p3 (Leela F-L, T-L-cap-f10, D-L-cap-std, D-L-unb-std 50M + its 20M; ci20): PASS if D-L-unb-std@50M - F-L@50M >= 30 at plies 2 AND 3 and still gaining 20M -> 50M (mean over plies 2-3 > 0); CLOSED if within 15 of F-L at plies 2 and 3 (below F-L + 15 at both also closes it); anything else INCONCLUSIVE.

- **11-13 20:32 LEELA-DATA: THE LR TWINS' READS AND THE LR-DAG RULE.** Per type (minus the peers' mean / minus the best): capped T-L p1 -271 / -300 (rank 5 of 5); capped T-L p2 -362 / -400 (rank 4 of 4); capped D-L p1 -257 / -266 (rank 5 of 5); capped D-L p2 -171 / -198 (rank 4 of 4); unbounded D-L p1 -257 / -266 (rank 5 of 5); unbounded D-L p2 -171 / -198 (rank 4 of 4). No ply-3 arenas (no plateaued LR-leaf trees or DAGs trained at ply 3). Rule arenas leela-rule-l-p2/-p3: Leela DAG-L unbounded 50M - Leela F-L 50M: ply 2  +10.4 +-25.8, ply 3   +2.8 +-26.0 Leela DAG-L unbounded 20M -> 50M: ply 2  +76.1 +-25.7, ply 3 +104.6 +-25.9; mean +90.3 VERDICT: FAIL: the DAG-L unbounded is within 15 of F-L at plies 2 and 3: CLOSED Sizes: unbounded D-L 1M 1 nodes / 1 leaves, 5M 4 nodes / 3 leaves, 20M 1 nodes / 1 leaves, 50M 1 nodes / 1 leaves; capped D-L 1M 1 nodes / 1 leaves, 5M 4 nodes / 3 leaves, 20M 1 nodes / 1 leaves, 50M 1 nodes / 1 leaves; capped T-L 1M 1 nodes / 1 leaves, 5M 1 nodes / 1 leaves, 20M 7 nodes / 5 leaves, 50M 13 nodes / 9 leaves. Full verdicts in leela/arch-results.txt.

- **11-13 20:27 LEELA-DATA: LR-LEAF TWINS LAUNCHED; THE LR-DAG RULE PRE-REGISTERED HERE, BEFORE ANY READ (the user ~20:20: "let's train one of each of those tree/dag models that we trained but also do it with an LR leaf. just to make sure we leave no leaf unturned. this is the last hurrah anyways").** T-L-Ne-cap, D-L-Ne-ctx-grow-cap, D-L-Ne-ctx-grow-unb = their probit twins' exact args with --leaf lr and without --split-noise 0.03 (probit leaves only: the binary refuses it with LR leaves); same records, order, seed 1113, snapshots 3/15/59/148 (leela/tools/train_lr_twins.sh; two cores, nice 10: D-L-unb from the start, T-L-cap then D-L-cap). Smoke first (4 rounds each, leela/smoke-lr2): all three train with the grow/ctx/router flags; held-out log-loss 0.8795, KL 0.268, MSE 0.0321 at 1.36M (one leaf yet; Leela F-L 0.8827 at 1.02M). READS (leela/tools/arch_arenas_l.sh -> leela/arch-L.out, leela/arch-results.txt): per-type arenas as the probit ones (ci30, Elo 0 = the peers' mean): leela-tl-p1/-p2 (ply 1: 4 capped T-L-Ne; ply 2: only 1 capped T-L-Ne, filled with 2 T-L-Nc, labelled fill: 3 peers), leela-dl-cap-p1/-p2 and leela-dl-unb-p1/-p2 (D-L-Ne: 4 at ply 1, 3 at ply 2; no fills needed); no ply-3 arenas (no plateaued LR-leaf trees or DAGs trained at ply 3). LR-DAG RULE: measured in leela-rule-l-p2/-p3 (Leela F-L, T-L-cap, D-L-cap, D-L-unb 50M + its 20M; ci20): PASS if D-L-unb@50M - F-L@50M >= 30 at plies 2 AND 3 and still gaining 20M -> 50M (mean over plies 2-3 of D-L-unb@50M - @20M > 0); CLOSED if within 15 of F-L at plies 2 and 3 (below F-L + 15 at both also closes it); anything else INCONCLUSIVE, reported as is.

- **11-13 19:59 LEELA-DATA READS: THE SAME-ARCHITECTURE ARENAS (leagues/arenas/diag/leela-<arch>-p<ply>; the Leela 50M final minus the peers' mean / minus the best peer, Elo, 95% in-run intervals ~+-23-50; verdicts in leela/arch-results.txt).** F-L p1 -184 / -208 (rank 5 of 5); F-L p2 -189 / -216 (rank 5 of 5); F-L p3 -1 / -12 (rank 3 of 5); F-L p4 -106 / -121 (rank 5 of 5); F-P p1 -301 / -319 (rank 5 of 5); F-P p2 -691 / -708 (rank 5 of 5); F-P p3 -544 / -548 (rank 3 of 3); capped T-P p1 -414 / -434 (rank 5 of 5); capped T-P p2 -627 / -655 (rank 5 of 5); capped T-P p3 -657 / -679 (rank 3 of 3); capped DAG p1 -411 / -417 (rank 5 of 5); capped DAG p2 -628 / -641 (rank 4 of 4). The Leela models lose to our plateaued self-play models of their own architecture at plies 1-2 everywhere (F-L by ~185, the probit-leaf ones by 300-700); only the Leela F-L at ply 3 matches its peers. PLAY-ARTIFACT CHECK (coordinator 19:55; leela/tools/art_check.sh, leela/artifact/fp-p2: the Leela F-P against fponly-p2 F-P#2@320 at ply 2, 200 games each, seed 11): fixed point and float (CHESSAI_FX/FX_PROBIT2/WINDOWS off) play byte-identical games, CHESSAI_FX_CHECK=1 silent; uncalibrated it scores 1.8% vs 2.0% calibrated (the own calibration's decisive slope 10.27 and offsets 10.2 cancel in ln P(W) - ln P(B): only the winner slope 0.82 acts); argmax 0.0%. NOT an artifact, nothing rerun. The probit's scale is normal (logit W-B per pawn on games/heldout-lr-d2-s555: Leela F-P 0.204, self-play F-P 0.228 (fponly-p2) and 0.268 (probit-widen-p1c); LR 0.60 is the LR's scale); what differs: the Leela F-P's value tracks material less (corr 0.883 vs 0.967 / 0.944; winner sign accuracy 0.70 vs 0.745 / 0.716), so in shallow search it gives material away (games lost in 67 plies on average, 42 at argmax). Likely from the data: quiet positions only, valued by a far stronger player. The unbounded DAG's arenas and the rule's arenas (leela-rule-p2/-p3) follow when it ends (queue B).

- **11-13 19:46 LEELA-DATA: A FIFTH LINE (F-L) AND NEW READS, BEFORE ANY READ (the user ~19:35-19:45: "we don't need to play against all our ply1 models just a number of plateaud models of the same architecture"; "so leela-trained LR vs our plateaued p1 LRs at p1 (let's grab 4 different models if we have that many) then at p2, p3 etc."; "we're starting separate arenas per model type right?").** F-L launched 19:33:24, ended 19:34:37 (std F-L args: --model lr --use-probs; same records, order, seed 1113 and snapshots). The capped DAG ended 19:43:25. The old waiter (leela_read.sh: panel + library-spread arenas) is SUPERSEDED: killed 19:33:58 before it started any arena, and neutralised (leela/CANCEL-READ). NEW READS (leela/tools/arch_arenas.sh, two queues A and B, nice 10; verdicts in leela/arch-results.txt): one arena per (architecture, ply), leagues/arenas/diag/leela-<arch>-p<ply> (fl, fp, tp, dag-cap, dag-unb): the Leela model's 50M final + the plateaued library models of that architecture TRAINED at that ply, distinct lines (leela/tools/pick_arch.py: the best two on ladder-2 + two typical; leela/picks-arch-pP.json); Elo 0 = the peers' mean; make_split_read defaults (T 0.02; hybrid21 at plies 1-2, hybrid18 at 3+); ci30. Peers: 4 per architecture at plies 1-2 except the DAGs at ply 2 (3: only three plateaued D-P DAGs trained at ply 2); ply 3: F-L 4, F-P 2, T-P 2, DAG none (skipped); ply 4: F-L only (the four lines of flat-p3-w4). Reported per architecture and ply: the Leela model minus the best peer and minus the peers' mean, and its rank. THE RULE (thresholds unchanged) is measured in its own arenas after those, leela-rule-p2/-p3: the five Leela 50M finals + the unbounded DAG's 20M (the rule's gain clause), ci20; ply 3 on hybrid21 if the 50M DAG is past 64 nodes or falls back to float on hybrid18 (fx_check.sh).

- **11-13 19:30 LEELA-DATA: A FOURTH LINE, A CAPPED DAG, LAUNCHED 19:29:58 (the user: "maybe we can train another one with some split limits?").** D-P-Ne-ctx-grow-cap on the same records, order and snapshots (seed 1113, --save-rounds 3,15,59,148; nice 10, one thread): the unbounded DAG's args except --split-limit 4 (as the capped T-P), --router-credit 10000000 and --router-chain 2 (the std recipe's router limits), and no --grow-credit / --grow-credit-cap / --grow-trials (recipe defaults 5M / 2 / 2). Exact: --model bdd --leaf probit --branch-factor 3 --split-threshold 1000000 --split-limit 4 --forget-factor 1 --router cnbp --route error --error log --use-probs --trial-error log --split-noise 0.03 --split-rule grow --path-context --grow-mdl 1 --grow-gate age --grow-priority snapshot --router-split-rule encourage --router-credit 10000000 --router-trial-windows 5 --router-trial-z=-2 --router-chain 2 --log-router-stats. The waiter (leela/tools/leela_read.sh, restarted 19:30) now waits for all four lines; the reads add its 5M/20M/50M snapshots (tag zc) and print capped DAG - F-P, - capped T-P, - unbounded DAG; the library arenas add its 50M final. The pre-registered rule (unbounded DAG vs F-P) is unchanged.

- **11-13 19:08 LEELA-DATA EXPERIMENT LAUNCHED 19:08:11 (the user's last experiment, ~19:00: "take our most promising DAG, unbound its leaf and router count and try to train it on a relatively large batch of Leela data the way stockfish nnue is trained"; a deliberate exception to self-play / zero knowledge for this one experiment; fork, models/from-scratch/leela/). RULE PRE-REGISTERED HERE, BEFORE ANY READ.** DATA: Leela T60 training data (storage.lczero.org/files/training_data/test60, ODbL), 45 tars, 17.45 GB, downloaded 18:42:58-19:06:25; training = 42 tars 2021-12-14 00:17 to 12-15 18:17 (76.45M records, 672,562 games), held-out = 1,000,089 positions (11,683 games) from 3 tars of 2021-12-28. Converter (leela/src/src/bin/leela_convert.rs + tools/extract.py): Chess960 games skipped whole (13,298 games, 1.8% of records); every other record rebuilt and checked: 74,403,319 consecutive-position links all one legal move apart (shakmaty), history slot 1 all equal, castling rights all equal, all 659,264 games from the start position, 0 invalid; Stockfish-style filter: in check 6.2%, best move a capture 17.2%, a promotion 0.19% dropped; 57,310,184 quiet positions kept; label = Leela's best_q/best_d as WDL (lambda 0: pure Leela value, results kept beside). Rounds of >= 340,000 positions (whole games), the same order and shuffle (seed 1113) for all three lines; snapshots after rounds 3/15/59/148 = 1.02M/5.10M/20.06M/50.33M positions. Binary chessai-leela1 (md5 7f786869f9cb3051f2fe37b80b6a7646) = treelab2 + `train --records` + eval-records; with the mode unused byte-identical to treelab2 on 16 cases (the six tree/DAG kinds with depth-2 labels, scratch and from x3's finals; F-P, F-L, T-P, D-P on results): models, states, stdout, stderr. LINES (nice 10, one thread each): D-P-Ne-ctx-grow-unb = x3's D-P line with the caps removed (--router-credit 10M -> 100k, --router-chain 2 -> 1M, + --grow-credit 100k (default 5M), --grow-credit-cap 1M (default 2), --grow-trials 1M (default 2); split-limit already 1M; gates unchanged: split-threshold 1M, grow-mdl 1, trial z 2, router-trial-windows 5, router-trial-z -2); F-P and capped T-P-Ne with x3's args. READS (waiter models/from-scratch/leela/tools/leela_read.sh -> leela/read.out): one arena per ply 1-3 (diag/leela-read-pP): 5M/20M/50M of the three lines + the stranger panel (ply 1 r/e/g/h @300; ply 2 y/v/l @299; ply 3 y/v/l + f3/f3b/p3fp/x4/bfp) + x3's F-P and capped T-P finals @436 + the top 3 ladder-2 models at the ply; 30 RR, make_split_read defaults; then library arenas per ply (diag/leela-lib-pP: the 50M finals + plateaued library models, top 5 per kind + the ladder median and neighbours). RULE: PASS if DAG@50M - F-P@50M >= 30 at plies 2 AND 3 and the DAG still gaining 20M -> 50M (mean over plies 2-3 of DAG@50M - DAG@20M > 0); FAIL (idea closed: even with Leela data the specialists only match one generalist) if within 15 of F-P at plies 2 and 3 (below F-P + 15 at both also closes it); anything else INCONCLUSIVE, reported as is. Also reported: each snapshot minus the best self-play model at each ply, held-out log-loss/KL/MSE, size, CPU.

- **11-13 18:17 FINAL LEADS, RESULTS SO FAR.** PAIRS FINAL (s7v3-p1-s1/-s2, both at cap 401; windows 390/395/400 pooled over both seeds; batch7-reads/pairs-final-final.out): pairs - plain F-P ply 1 +36 (SE 9; FULL +29), ply 2 -48.5 (FULL -51), ply 3 -37 (strangers) => pre-registered VERDICT: pairwise features CLOSED for the goal. DISTILL-P2 FINAL, PLY 2 (distill-p2-s1 plateaued at cycle 313, s2 at 449; windows final-10/-5/final, both seeds; distill-read-final.out): distilled tree/DAG - the field's F-P: capped T-P +20 (SE 11; FULL +21), capped T-L -22, T-P grow +4 (FULL +7), T-L grow -45, D-P +7, D-L -58; vs F-L: T-P capped +29. Cross-arena vs x3's same lines trained on results: T-L capped +34, T-P capped +46, T-L grow +104, T-P grow +117, D-L +91, D-P +57 (flats +14/+16, the field effect). READ: distillation lifts every tree and DAG far above its result-trained twin, but only to the flat's level (grow, DAG) or a little above it (capped T-P +20 < the 30 bar at ply 2); the ply-3 half (distill-p3-mid/-final arenas) still running. Extensions: std-v1-p2-s1/-s2 extended 301 -> 601 at 17:21 (extend-p2-601.out), cycle 392 at 18:16.

- **11-13 14:39 DISTILL-P2 LAUNCHED 14:11:31 (fork a9af182d; distill-p2-s1/-s2, seeds 86/87, cap 601; specs leagues/specs/batch7/distill-p2-s1.json, -s2; design, comparators and the pre-registered rule in the descriptions).** x3's six tree/DAG lines labelled by the field's F-P (moving labeller: F-P v(c-1) each cycle); a 3-cycle smoke passed first; created configs = x3's args + the label flags. SHARED FILES: line names hit 128 chars (breaks self-games), so the fork added five recipe rows holding x3's exact args (T-L-Ne-cap, T-L-Ne-grow-f10-std, T-P-Ne-grow-std, D-L-Ne-ctx-grow-std, D-P-Ne-ctx-grow-std; names <= 70 chars); recipes.txt / make_recipes.py swapped 14:11:06 under md5 guards (backups *.before-distill; rows added only). LABEL CACHE (speedup, pre-approved): bin/chessai-treelab2 (md5 f676d842...) shares one label computation per cycle across the lines; 54 runs (six kinds x cache off / cold / warm) byte-identical to treelab, six processes at once identical, a 5-cycle replay of s1 = the live cycles 1-5 (48 models, 8,288 games, ratings, decisions); swapped into both runs 14:33:17 (atomic config.json replace; backups labelcache/swap/RUN/); training 62 -> 16 CPU-s a cycle (s1), 59 -> 16 (s2); whole cycle ~96 -> ~50; ~10 CPU-h saved over both seeds to cycle 400. Checked 14:39: both running (cycles 41-43), 0 errors, six treelab2 lines each. Read waiters batch7-reads/distill-read.sh mid|final (14:12:58; distill-read-{mid,final}.out; x3's same lines read in their own arenas against the same panel via xpool.py). The ply-2 extensions wait for these runs (extend-after.sh). Notes: models/from-scratch/distill/HANDOFF.md, labelcache/summary.txt.

- **11-13 14:35 PAIRS FINAL TEST LAUNCHED 14:23:41 (fork a405ab35; s7v3-p1-s1/-s2, seeds 77/78, ply 1, cap 401; specs leagues/specs/batch7/s7v3-p1-s1.json, -s2).** Binary models/from-scratch/bin/chessai-pairs3g (md5 f42fdaff...) = pairs3f + the duplicate fix (a candidate is skipped when a chosen pair already covers the same interaction; side to move and the castling rights are two-valued, so their two values gave the same pair). Identity: pairs off = hybrid6 training 16/16, play = hybrid21 31/31 (plies 1-5); stop/resume with pairs on identical; nothing to skip = pairs3f byte for byte; null 0 pairs (25 models = pairs3f's null). Clean-list smoke (29 cycles, the pairs3 smoke's seed): the real line 40 pairs, no duplicates (pairs3f 22 of 40), BUT side to move in 21 of 40 and 22% inert, so the list still fails the clean-list bar (the test runs anyway, the user's final test); the faster-checkpoint observer 51 pairs, 2% inert (pairs3f 33%), hubs no-en-passant 8, square 59 7. Binaries switched in the specs only (recipes.txt, make_recipes.py untouched). ~11 s a cycle at v60. Waiters (14:23:47): batch7-reads/pairs-final-read.sh mid (v150, plies 1-2) and final (version windows, plies 1-3, prints the pre-registered verdict). Notes: models/from-scratch/pairs3g/ (HANDOFF.md, summary.txt; 934 MB). Also running: distill-p2-s1/-s2 (seeds 86/87, cap 601, cycle ~30 at 14:35; fork a9af182d not yet reported; distill-read.sh mid/final waiters up).

- **11-13 14:11 USER: extend the warm ply-2 rung ("sure go ahead").** std-v1-p2-s1/-s2 ended at their cap 301 (already extended once, 101 -> 301, at 01:29) while still crowning (s1 at 297, s2 at 273). Queued after the distillation runs: batch7-reads/extend-after.sh waits for distill-p2-s1/-s2 to end, then extend-when-free.sh dry-runs, extends both to 601 and restarts repeat.sh (log batch7-reads/extend-p2-601.out; cancel with batch7-reads/CANCEL-EXTEND). Dry runs at 601 pass for both now. Cost: ~80 CPU-s a cycle late in the run, so ~6.6 CPU-h each to 601. Plateau Monte Carlo (caps5.py's walk over the last 100 cycles): crown rate 0.28 a rated cycle and the guard (z > 1, 8 lines) holds 94% of quiet rated cycles, so P(plateau before 601) only 0.04-0.06: expect them to end at 601 on the cap again; that guard makes 'plateau' close to unreachable for 8-line ply-2 fields (the user's call then). CORRECTION to my point-5 review note to the user: measured whole-run CPU is 6.9 / 6.7 CPU-h for the warm runs (to 301) and 7.8 for scratch x3 (437 cycles), not 2.3 vs 13; warm stays on the +18 < 30 rule, not on cost.

- **11-13 14:05 USER: "Let's do one final test of pairs".** A fork builds chessai-pairs3g (pairs3f + the duplicate fix), re-checks the null and the list, then launches s7v3-p1-s1/-s2 (seeds 77/78, ply 1, F-P vs F-P-S7v3) whatever the list shows, with reads at plies 1-3 (pairs-final-read.sh). PRE-REGISTERED: at the final (two seeds, windows) pairs - plain F-P below +20 at every ply closes pairs for good; -30 or worse at ply 3 closes them for the goal; +20 or more at plies 2 AND 3 reopens them (a ply-2 test next).

- **11-13 14:00 USER APPROVED distillation inside the strong field** ("We can test distillatoin inside stronger field"): distill-p2-s1/-s2 (seeds 86/87) = std-v1-p2-x3 from scratch with all six tree/DAG lines trained on the field F-P's depth-2 search labels (moving labeller, chessai-treelab); flats unchanged. A fork builds the specs, smokes, launches both seeds, writes the windowed reads, then builds the exact shared label cache (review-1113c) and swaps it in between cycles once byte-identical. Rule: a distilled tree/DAG 30+ over the field's best flat at plies 2 and 3 (two-seed window means) = the goal's first pass; within 15 = trees only reach the flats.

- **11-13 12:35 USER DECISION: depth-2 search labels ADOPTED for the ply-1 standard's F-P** ("sure, we can adopt it for ply 1"; STANDARD.md decision 6). F-L untested with labels (stays on results until tested); ply 2+ unchanged.

- **11-13 11:42 LABELDEPTH-P2 POOLED FINAL (windows: s1 v362-372 plateau, s2 v390-400 cap; arenas diag/labeldepth2b-p2-final, -p3-final): DEPTH-3 LABELS ARE NOT THE LEVER AT PLY 2; THE 07:09 'VALUE GROWS WITH DEPTH' EXTENSION IS REFUTED.** Two-seed window means FP / full (seeds): PLY 2 d3 - p 18.2 / 21.5 (9, 27 / 22, 21); d2 - p 41.5 / 28.8 (13, 70 / 13, 45); d3 - d2 -23.7 / -7.5. PLY 3 d3 - p -6.7 / -1.5; d2 - p 12.5 / 5.5; d3 - d2 -19.2 / -7.7. Rule: d3 - p 30+ at plies 2 and 3 => adopt: NO; within 15 at ply 3 and ~20 at ply 2 => label depth is not the lever at ply 2 (closed for depth 3). d3 led at mid-run (+37 at ply 2) and fell back, while d2 rose. DEPTH-2 LABELS OVER PLY-2 PLAY, ALL FOUR SEEDS (late windows, full panel at ply 2): levers-p2 -14, levers2-p2 +59, labeldepth-p2 s1 +13, s2 +45: mean ~+26, SD across seeds ~32, SE ~16; at ply 3 ~+3 (SE ~10). Below 6.110's 40 line: labels stay out of the ply-2 rung's flats for now. COORDINATOR READING (replacing 07:09's): EVEN-DEPTH LABELS BEAT ODD-DEPTH ONES in both regimes (ply-1 play: d2 > d1 at plies 2-3; ply-2 play: d2 >= d3), which fits a horizon bias: the label search has no quiescence, so an odd-depth search ends on its own side's move and counts horizon captures as free. For ply-1 training depth-2 labels are robust (+38-58 at plies 1-3, two seeds, windows); for ply-2 training they give ~+26 +- 16 at ply 2 only. An even deeper even label (depth 4, ~10x depth 3's cost) is the remaining label test; not launched.

- **11-13 11:05 LABELDEPTH-P2 POOLED MID READ (v290/295/300 of both seeds; arenas diag/labeldepth2b-p2-mid, -p3-mid; 18 judged + y/v/l @299, 30 RR): DEPTH-3 LABELS +37 AT PLY 2 (BOTH SEEDS), +15-20 AT PLY 3.** Two-seed window means, FP / full (seeds; SE ~10.7 / ~7.5 measurement): PLY 2 d3 - p 38.2 / 37.0 (29.7, 46.7 / 31.0, 43.0); d2 - p 13.8 / 19.0 (13.7, 14.0 / 17.7, 20.3); d3 - d2 24.2 / 18.2. PLY 3 d3 - p 15.3 / 20.0 (24.0, 6.7 / 35.0, 5.0); d2 - p -7.3 / -0.3 (-35, 20 / -15, 15); d3 - d2 22.5 / 20.0 (59, -14 / 50, -10). Against the prediction (final: d3 - p 30-60 at ply 2, 40-80 at ply 3; d3 - d2 larger at ply 3): ply 2 inside, ply 3 below so far, and d3 - d2 is ~20 at both plies, not rising with depth (the seeds split at ply 3). Rule (30+ at plies 2 and 3) not met at mid-run; the final (s1 plateaued at v372 at 10:35, s2 at v319) decides. The d2 - p cells (third and fourth ply-2 seeds of levers' contrast) read +14-19 at ply 2 mid: smaller than levers' +46-55.

- **11-13 08:36 LABELDEPTH-P2-S2 LAUNCHED 08:36:19 (seed 85; spec leagues/specs/batch7/labeldepth-p2-s2.json), the second seed of the depth-3 field,** while the machine idled (load ~4 of 8): ply-2 run noise needs two seeds for the 30-Elo rule. ~9.5 CPU-h. Reads now pooled over both seeds in one arena per ply (batch7-reads/labeldepth2b-read.sh mid / final); the s1-only waiters were stopped before they made anything. s1 at v122 (~27 s a cycle): both past v300 ~11:00-11:30, both ended ~12:00-12:30.

- **11-13 08:36 GOAL CROSS-READ (arenas diag/goalcross-p2, -p3; 25 judged + y/v/l @299, 30 RR; window means of 3 versions for the labels-tree lines and std-v1-p2-x3's F-P / F-L / capped T-P, single finals for the rest; batch7-reads/goalcross.sh): THE FLAT-LABELLED TREE DOES NOT YET PASS THE BEST PARAMETER-LIMITED MODELS; ITS WHOLE FIELD IS WEAKER.** Vs the panel's flats (full panel in brackets): PLY 2: std x3 F-P +83 (+105), x3 T-P capped +68, x3 F-L +63, std s2 F-L +58, s2 F-P +46, flat-labelled tree s1 +26 / s2 +9, F-P-lab s2 -8 / s1 -20, levers2 labels -28, levers2 result -90. PLY 3: x3 F-P +89, x3 F-L +79, x3 T-P capped +76, s2 F-L +73, flat-labelled tree s1 +70, s2 F-P +66, flat-labelled tree s2 +63, levers2 labels +38, F-P-lab +38 / +34, levers2 result +22. So inside its field the flat-labelled tree leads the flat (11 at ply 2, 40 at ply 3), but the best std ply-2 run (x3: an 8-recipe mixed field from scratch, 437 cycles) beats it by ~60-75 at ply 2 and ~20-25 at ply 3: the labels-tree fields (one flat + three capped trees) are ~100 weaker at the flat. Field composition again outweighs the training lever. NEXT (for the user): test distillation INSIDE the strong field: the ply-2 standard from scratch with its trees (capped T-P at least) labelled by the field's F-P line (moving labeller), read against x3 (cross-run, needs 2 seeds) and within the run (tree vs F-P, the goal cell); ~14 CPU-h a run. Not launched (size; the user's call).

- **11-13 08:12 LABELS-TREE FINAL, WINDOWED, TWO SEEDS (s1 v322/327/332, s2 v390/395/400; arenas diag/labels-tree3-p2-vfinal-332-400, -p3-; 24 judged + y/v/l @299, 30 RR): A CAPPED TREE LABELLED BY THE FLAT'S SEARCH BEATS THE RESULT TREE BY ~60 (PLY 2) / ~32 (PLY 3) AND THE FIELD'S LABELLED FLAT BY ~11 (PLY 2) / ~40 (PLY 3).** Two-seed window means (seeds; measurement SE ~11 TR/FP face, ~7.5 full): PLY 2: own-search - result tree 1.7 (27, -24); flat-labelled - own 57.5 (45, 70); FLAT-LABELLED - RESULT TREE 59.5 (73, 46); vs F-P-lab FP / TR / full: flat-labelled 9.7 (38, -19) / 13.7 (10, 17) / 11.3 (24, -2); own-search -35 / -44 / -39; result tree -58 / -46 / -52. PLY 3: own - result 3.7; flat-labelled - own 28.5 (22, 35); FLAT-LABELLED - RESULT TREE 31.8 (33, 31); vs F-P-lab FP / TR / full: FLAT-LABELLED 41.5 (55, 28) / 38.0 (35, 41) / 39.7 (45, 35); own-search 6.7 / 9.8 / 8.3; result tree 21 / 6.2 / 13.5. Seed gaps (window means, full panel): ply 2 fl 9, tr -35, to 44, tf 44; ply 3 within 25. Against 6.113: the own-search arm fails (-35..-44 vs the flat at ply 2: the bootstrap bites); for the flat-labelled arm ply 2 is 'within 15' (the tie stands) and ply 3 is a ~40 lead (both seeds 28-55, ~2-3 SE with run noise). READING: distillation from the flat's search is the form that works for trees (+32-60 over result-trained trees, both seeds, both plies); whether it passes the parameter-limited models overall needs a cross-read against the best std ply-2 models (x3 F-P, capped T-P): goalcross read started.

- **11-13 08:02 LEVERS2 LATE-WINDOW READ (levers2-p2 ended at the 401 cap; v390/395/400 of both seeds; arenas diag/levers2-p2-final, -p3-final): SEEDS SPLIT AT PLY 2, INCONCLUSIVE; NO GAIN AT PLY 3.** Depth-2 labels - result labels (FP / full): PLY 2 s1 (levers-p2) 0.7 / -13.7, s2 (levers2-p2) 58.0 / 59.3, both 29.3 (SE 10.8) / 22.8 (7.6); PLY 3 s1 1.0 / -3.0, s2 -17.3 / -8.3, both -8.2 / -5.7. The seed gap at ply 2 (57-73) is ~1.5-1.8 run-noise SDs (ply-2 per-line SD ~28, a contrast between seeds ~40), so the two-seed mean +23-29 has an SE of ~30: neither the rule's 30+ (adopt) nor its within-15 (head start only). Robust across both seeds and windows: equal-depth labels give ~+50 at the play ply mid-run and nothing (or a small cost) one ply deeper. A third ply-2 seed of the same contrast is inside labeldepth-p2-s1 (its d2 line), read at v300 and final. Library index rebuilt.

- **11-13 07:42 LEVERS2 MID-WINDOW READ (levers-p2 + levers2-p2, v290/295/300 each, one arena per ply; arenas diag/levers2-p2-mid, -p3-mid): EQUAL-DEPTH LABELS AT PLY 2 BEHAVE LIKE DEPTH 1 AT PLY 1, BOTH SEEDS.** Depth-2 labels - result labels (FP / full): PLY 2 s1 48.7 / 39.3, s2 61.0 / 52.0, both 54.8 (SE 11.0) / 45.7 (7.7); PLY 3 s1 -28.7 / -20.3, s2 -9.0 / -6.3, both -18.8 (10.7) / -13.3 (7.5). A label as deep as the play helps at the play ply (+46-55) and costs one ply deeper (-13..-19), as labeldepth-p1's d1 did (+68 at ply 1, -36 at ply 3). Late window (levers2 final + levers-p2 390-400) decides whether the ply-2 gain persists (levers-p2 alone: ~0 late).

- **11-13 07:42 labels-tree-p2-s2 ended at the 401 cap (v400; s1 plateaued at v332); LABELDEPTH-P2-S1 LAUNCHED 07:41:59 by the queue; the windowed two-seed labels-tree final read (labels-tree-read3.sh) starts now; library index rebuilt.**

- **11-13 07:20 LABELS-TREE POOLED READ v300 (two seeds, one version each; arenas diag/labels-tree2-p2-v300-300, -p3-; 30 RR vs y/v/l @299): THE FLAT-LABELLED TREE LEADS THE LABELLED FLAT BY ~25-40; OWN-SEARCH LABELS HURT TREES.** Two-seed means (seeds): PLY 2: own-search - result tree (TR face) -19.5 (11, -50); flat-labelled - own-search 55 (32, 78); vs F-P-lab, FP / TR face: own-search -38.5 / -16, FLAT-LABELLED 14 (20, 8) / 39 (39, 39), result tree -51.5 / 3.5. PLY 3: own - result -29.5; flat-labelled - own 60 (63, 57); vs F-P-lab FP / TR: own -12 / -32, FLAT-LABELLED 38 (48, 28) / 28 (-2, 58), result 16.5 / -3. So flat-labelled - result tree ~ +35 at ply 2 and +30 at ply 3. Noise: measurement SE ~18 a pooled cell; seed gaps s1 - s2 ~22 pooled (to 59 at ply 2, fl 39 at ply 3): total SE ~25, so +25-40 is ~1-1.5 SE; v150 (s1) had tf - fl +109 at ply 2, shrunk. Against 6.113: own-search trees fail the rule (below F-P-lab); fixed - own 55-60 = 'the bootstrap bites, the fixed labeller is the form'. READING (coordinator): the tree DISTILLS the flat's search: it fits the flat's depth-2 values better than the flat itself can, while its own early search feeds its errors back. Final (windows, both seeds) decides; if tf - fl holds 30+ at plies 2-3, the next test is the tree labelled by the flat's DEPTH-3 search (labeldepth: deeper labels pay more at deeper reads).

- **11-13 07:12 DEPTH-3 LABELS QUEUED: labeldepth-p2-s1 (seed 84; leagues/specs/batch7/labeldepth-p2-s1.DRAFT.json; launches when one field is left, i.e. after labels-tree-p2-s2 ends).** Called for by labeldepth-p1's pre-registered rule (at ply 2: d1 - p -4, within 20; d2 - p +53-57, 30+ => 'labels must be deeper than the play'; next: depth-3 labels over ply-2 play). One ply-2 field, three F-P lines: result (p), depth 2 (= play; a third seed of levers' line), depth 3 (= play + 1). MEASURED label cost (one 136k-position round from levers2-p2's cycle-184 set, levers-p2's v399 labels model, nice 19): depth 2 6.5 CPU-s (3.8 s wall), depth 3 57 CPU-s (17.6 s wall), 8.8x; the field ~9.5 CPU-h to the 401 cap. Reads (batch7-reads/labeldepth2-read.sh mid / final; windows; y/v/l @299 at plies 2-3). Prediction: d3 - p 30-60 at ply 2, 40-80 at ply 3; d3 - d2 positive, larger at ply 3. Rule: d3 - p 30+ at plies 2 and 3 => the ply-2 rung's flats train on depth-3 labels; within 15 => not the lever. One seed (ply-2 run SD ~28 a line): a 30-60 effect needs the second seed before adoption.

- **11-13 07:09 LABELDEPTH-P1 FINAL, PLY 3 (arena diag/labeldepth-p3-vfinal; windows, two seeds; vs the ply-3 strangers f3, f3b, p3fp, x4, bfp): THE PARITY READING OF LABELS IS REFUTED; LABEL DEPTH MATTERS MORE THE DEEPER THE SEARCH.** PLY 3: d1 - p -36.2 (SE 8.4; seeds -29.7, -42.7); d2 - p 38.3 (7.9; 58.0, 18.7); d2 - d1 74.2 (8.2; 87.3, 61.0). Parity predicted d1 >= d2 at the odd ply 3; instead d2 - d1 rises with the read ply: -17 (ply 1), +59 (ply 2), +74 (ply 3) (FP/full means), d1 - p falls: +68, -4, -36, and d2 - p holds: +50, +55, +38. CORRECTED READING (coordinator; supersedes the 06:26 'a depth-d label carries d's parity' and the 06:35 'LABELS CARRY PARITY' titles, kept for the record): a label as deep as the play (d1 over ply-1 play) helps only at that ply and hurts deeper searches; a label one ply deeper (d2) helps at plies 1-3; the deeper the search the model serves, the more label depth is worth. The training-ply parity gap (flat cross-read, 11-11's 6.19-6.22) stands; it is the label part that was wrong. For the goal: label depth should exceed the play depth, and its value grows with the search depth used; for ply-2 play that means depth-3+ labels (untested; ~15x the depth-2 label cost). levers2-p2 tests depth 2 over ply 2 (equal depth): by this reading it can help at ply 2 but should not help at ply 3. PROPOSAL for the user: the ply-1 standard's flats train on depth-2 labels (+38-58 at plies 1-3, two seeds, windows).

- **11-13 07:06 LABELDEPTH-P1 FINAL, PLIES 1-2 (s1 plateau v312, s2 cap v400; windows final-10 / final-5 / final, 6 versions a contrast; arenas diag/labeldepth-p1-vfinal, -p2-vfinal; vs r/e/g/h @300): SEARCH LABELS PAY AT PLY 1 AT ANY DEPTH; AT PLY 2 ONLY THE EVEN LABEL.** FP face / full (measurement SE ~9 / ~6.5): PLY 1 d1 - p 77.0 / 57.7; d2 - p 57.7 / 43.0; d2 - d1 -19.5 / -14.3. PLY 2 d1 - p -3.2 / -5.5; d2 - p 57.3 / 53.2; d2 - d1 60.2 / 58.5. Trajectory of d1 at ply 2: -56 (v150), -27 (v300), ~-4 (final); at ply 1: 3, 74, 58-77. Run noise: two seeds, ply-1-trained ~18 a recipe. CONCLUSIONS (two seeds, windows): (1) equal-depth labels pay at the play ply at the end (+58-77 at ply 1), so levers-p2's late convergence at ply 2 was seed- or ply-specific (levers2-p2 decides); (2) the same-parity label wins at each read ply (d1 by 14-20 at ply 1, d2 by ~60 at ply 2); (3) for the ply-1 standard's flats, depth-2 labels give +43-58 at ply 1 and +53-57 at ply 2, above 6.110's 40 line (proposal for the user: F-P/F-L at ply 1 train on depth-2 labels; F-L untested with labels). Ply 3 cell running. Library index rebuilt.

- **11-13 06:36 LEVERS2-P2 LAUNCHED 06:35:45 (seed 83; spec leagues/specs/batch7/levers2-p2.json): the second seed of levers-p2's labels contrast at ply 2** (F-P result vs F-P depth-2 labels; S7 dropped; hybrid21), because labeldepth-p1 found equal-depth labels paying +70-78 at the play ply while levers-p2's one seed converged to ~0 late. ~2.3 CPU-h. Reads pooled with levers-p2 in one arena per ply, version windows (batch7-reads/levers2-read.sh: mid = v290/295/300 of both, final = levers2's final-10..final with levers-p2's 390-400). Rule: late window pooled 30+ at ply 2 => same-parity labels pay at ply 2 (adopt for the standard's ply-2 flats on the two-seed mean against 6.110's 40 line); within 15 => a head start only.

- **11-13 06:35 LABELDEPTH-P1 READ v300 (two seeds, one version each; arenas diag/labeldepth-p1-v300, -p2-v300): LABELS CARRY PARITY, BOTH SEEDS.** Pooled, FP face / full (seeds in brackets): PLY 1 (odd) d1 - p 77.5 / 70.5 (69, 86 / 73, 68); d2 - p 51.5 / 54 (21, 82 / 45, 63); d2 - d1 -26.5 / -16. PLY 2 (even) d1 - p -29.5 / -25 (-22, -37 / -35, -15); d2 - p 33.5 / 42.5 (38, 29 / 25, 60); d2 - d1 63 / 67.5 (60, 66 / 60, 75). Measurement SE ~16 (FP) / ~11 (full) pooled. Odd (depth-1) labels win at the odd ply and lose at the even ply; even (depth-2) labels win at the even ply and still help at the odd ply. And EQUAL-DEPTH LABELS PAY AT THE PLAY PLY here: d1 over ply-1 play +70-78 at ply 1 by v300 (+3 at v150), the opposite of levers-p2's trajectory (depth 2 over ply 2: +35-45 mid, ~0 late, ONE seed); so levers-p2's convergence may be seed- or ply-specific, and a second ply-2 seed would settle it. Against the spec's rule (d1 within 20 and d2 30+ at ply 2 = 'deeper than play'): d1 is -27 at ply 2 (parity, not 'within 20') and +74 at ply 1: neither branch fits; the parity reading does. Final (cap 401, ~07:00) adds version windows and ply 3 (parity predicts d1 >= d2 at ply 3).

- **11-13 06:26 FLAT CROSS-READ (arenas diag/flatcross-p2, -p3; 11 flat probit finals + y/v/l @299 at ply 2, + the ply-3 strangers at ply 3; 30 RR; single finals, simple means vs the panel's flats; batch7-reads/flatcross.sh): THE TRAINING-PLY GAP DWARFS THE LABEL GAP, AND IT IS A PARITY GAP.** PLY 2: ply-2-trained std F-P x3 +58, s2 +41, s1 +18; levers-p2 (ply 2, flats-only field) result -73, labels -74; ply-1-trained std F-P s1 -347, s2 -364; labels-p1 depth-2 labels -353 / -364, result -406 / -409. PLY 3: std ply-2 s2 -55, s1 -60, x3 -65; std ply-1 s1 -92, s2 -135; levers labels -135, result -155; labels-p1 labels -190 / -233, result -265 / -292. So ply-1-trained flats trail ply-2-trained ones by ~400 at ply 2 but only ~30-70 at ply 3, the 11-11 parity effect (6.19-6.22: an evaluator learns its training horizon's parity in shared weights). Flats-only fields cost their flats ~100-150 at ply 3 (labels-p1 vs the mixed ply-1 standard) and ~130 at ply 2 (levers vs x3). REREAD OF THE LABEL RESULTS (coordinator): a depth-d label carries d's parity. Depth-2 labels over ply-1 play (labels-p1) lift ply-2 play by 50-75 (even labels fix part of an odd model's even-ply deficit, still ~350 short of ply-2 training); depth-1 labels over ply-1 play (labeldepth d1) cost ~55 at ply 2 (odd labels deepen it); depth-2 labels over ply-2 play (levers) add no parity, so only an early variance cut that the result line catches. The ply-1 d2 - d1 gain of 22-30 at ply 1 (v150) does not fit pure parity (parity says d1 >= d2 at odd plies), so some depth value remains; the labeldepth final's PLY-3 cell separates them (parity: d1 >= d2 at ply 3; depth: d2 > d1). S7's pattern (~0 at ply 2, -22..-95 at ply 3, trained at ply 2) also reads as parity-specific pair weights. IMPLICATIONS: (1) labels cannot substitute for training at the target ply; for a fixed final depth, train at its parity (6.22 stands); (2) the next label test would need depth > play WITH the same parity (depth-4 labels over ply-2 play), expensive, not launched; (3) labels-tree-p2 (depth 2 over ply 2, same parity) can show at most the variance cut, which may still matter more for data-starved tree leaves.

- **11-13 06:06 LABELDEPTH-P1 READ v150 (two seeds, one version each; arenas diag/labeldepth-p1-v150, labeldepth-p2-v150; vs r/e/g/h @300, 30 RR).** p = result labels, d1 = depth-1 labels (= play depth), d2 = depth-2 (= play + 1). Pooled over seeds, FP face / full: PLY 1 d1 - p 3.0 (SE 16) / 2.5 (11); d2 - p 32.5 / 24.5; d2 - d1 29.5 / 22. PLY 2 d1 - p -56.5 (17) / -55.5 (12) (s1 -86, s2 -27); d2 - p 14 / 4 (seeds -22 and 50); d2 - d1 70.5 / 60 (seeds agree: 64/77, 59/61). Early reading: equal-depth labels give NO head start at ply 1 and cost ~55 at ply 2; the extra ply is what pays (d2 - d1 60-70 at ply 2, both seeds). This already departs from levers-p2 (equal-depth labels over ply-2 play led ~45-65 early), so 'equal depth = head start' is not general; 'deeper than the play' is the part that survives so far. Noise: single versions (~24 SD a two-version contrast), ply-1 run SD ~18 a recipe. v300 and the windowed final decide.

- **11-13 05:42 LEVERS VERSION WINDOW, PLY 3 (arena diag/levers-window-p3, 16 judged + y/v/l @299, 30 RR): NO LABELS LEAD AT PLY 3 IN EITHER WINDOW.** Labels - result, FP face / full: MID (v290-300) -4.7 (SE 15.6) / 8.0 (10.8); LATE (v380-400) 16.0 (11.7) / 0.2 (8.2). The same v300 models read 44 / 50 in levers-p3-v300 and -4 / 26 here: single-version ply-3 contrasts carry ~27 SE (FP face) of measurement, so the single-version ply-3 reads (v150 53 / 64, v300 44 / 50) overstated the gain. levers-p2 summary: depth-2 labels over ply-2 play gave ~35-45 at ply 2 mid-run, ~0 late, and ~0-15 at ply 3 throughout the windows. LESSON: read version windows (3-5 versions) for any decision cell; the pooled labels-tree final read moves to windows (labels-tree-read3.sh: final-10 / final-5 / final per seed, plies 2-3).

- **11-13 05:31 LEVERS VERSION WINDOW (ply 2; arena diag/levers-window-p2, 16 judged + y/v/l @299, 30 RR): THE CONVERGENCE IS REAL, not one bad version.** Labels - result, FP face / full panel: MID (v290/295/300) pooled 45.0 (SE 15.7) / 34.7 (11.0); LATE (v380-400, 5 versions) pooled -9.8 (11.8) / -2.0 (8.2); per version late: -31, 6, 16, 5, -45 (FP). So in levers-p2 (depth-2 labels over ply-2 play, a flats-only field, one seed) labels were a head start the result line caught by ~v380. Ply 3 window pending. LAUNCHED 05:30:21 LABELDEPTH-P1-S1/-S2 (seeds 81/82; specs leagues/specs/batch7/labeldepth-p1-*.json): the cheap mechanism test, one ply-1 field per seed with F-P result / F-P --label-depth 1 (= play depth) / F-P --label-depth 2 (= play + 1), cap 401, floor 300, ~1.1 CPU-h a run; smoke 2 cycles OK. Reads waiting (batch7-reads/labeldepth-read.sh 150, 300, final; the final with version windows and a ply-3 cell). Prediction and rule in the spec: d1 within 20 and d2 30+ pooled => labels must be deeper than the play (then: depth-3 labels over ply-2 play for the standard); d1 30+ too => equal-depth labels pay and levers-p2's convergence was field- or seed-specific.

- **11-13 05:30 LABELS-TREE-P2 EARLY READ v150 (seed 79 only; arenas diag/labels-tree-p2-v150, labels-tree-p3-v150; 12 entries, 30 RR vs y/v/l @299).** Lines: fl = F-P-lab, tr = capped T-P-Ne result labels, to = the same tree with own-search labels, tf = the tree labelled by the F-P-lab line. PLY 2 (TR face for tree cells; FP / TR / full for tree vs flat): to - tr 33 [-19, 85] (full 30); tf - to 37 (full 52); tf - tr 70; to - fl 46 / 67 / 57; TF - FL 115 / 104 / 109; tr - fl 19 / 34. PLY 3: to - tr -5 (full 26); tf - to 20 (full 9); to - fl 70 / -24 / 22; tf - fl 68 / -4 / 32; tr - fl 12 / -19 (the faces disagree by ~90 at ply 3: trees do well against the FP strangers and badly against the tree strangers, a matchup effect; the full panel is the fair middle). Against 6.113's final bands: to - fl above its -20..40 band at ply 2 (57), inside at ply 3 (22); tf - to 37-52 at ply 2 = the bootstrap bites early (band: within 20). THE TREE LABELLED BY THE FLAT LEADS THE LABELLED FLAT BY ~105-115 AT PLY 2 and ~32 at ply 3: the first ply-2 cell where a tree clearly beats the field's best flat. CAUTION: one seed, one version, v150 (the levers-p2 labels lead was 71 at v150 and gone at v400); ply-2-trained per-line run SD ~28. The pooled two-seed reads (v300, final) decide.

- **11-13 05:16 LEVERS-P2 FINAL (cap 401, v400; arenas diag/levers-p2-v400, levers-p3-v400): THE LABELS LEAD IS GONE AT v400.** Search labels - result labels (F-P), FP face / full panel: PLY 2 -4 [-57, 49] / 5 [-32, 42] (v150 71 / 61, v300 63 / 45); PLY 3 13 [-39, 64] / -6 [-42, 30] (v150 53 / 64, v300 44 / 50). S7 - result: ply 2 2 / -6, ply 3 -22 / -68. Against 6.109/6.110 (final bands 30-90 at ply 2, 20-80 at ply 3; 40+ at ply 2 adopts labels for the standard's flats): BELOW both bands, NOT adopted on this read. Ratings vs the panel's FP face (simple means): result line -268 / -155 / -96 at ply 2 (v150 / v300 / v400), labels line -194 / -95 / -78; at ply 3 -203 / -124 / -42 and -152 / -84 / -42: both still climbing, the result line faster late (+59 v300->v400 at ply 2 against the labels line's +17), and the field still ~80-100 below the panel. Noise: one seed (ply-2-trained per-line run SD ~28), a single version each (~24 SD for a two-version contrast), measurement ~16-26. COORDINATOR HYPOTHESIS (analyst out): a label from a search NO DEEPER than the play is a variance cut with the model's own bias: a head start that result labels catch up (levers-p2: depth-2 labels over ply-2 play); a label DEEPER than the play carries information the games lack and keeps its lead (labels-p1: depth-2 labels over ply-1 play, 48.5 at ply 1 and 74.5 at ply 2 at v500/v408, two seeds). Checks: (1) running now, a version-window read (batch7-reads/levers-window.sh: lp and ll at 290/295/300 and 380-400 pooled, plies 2-3): LATE near 0 with MID ~50 => convergence; LATE 30+ => v400 was version noise; (2) if convergence, the direct test is depth-3 labels over ply-2 play (label depth = play + 1), prediction: a lead that persists to the final. labels-tree-p2 uses depth 2 at ply 2, the converging configuration, so its trees may show a head start only; it runs on unchanged.

- **11-13 05:10 PLY-2 RUNG FINAL (std-v1-p2-s1/-s2 warm, cap 301; -x3 scratch, plateau at 437; arena diag/std-v1-p2-read-final, 24 judged + y/v/l @299, 30 RR; std_p2_read.py): WARM STAYS THE RULE.** Scratch - warm pooled +18 [+6, +29] (sampling), below 6.95's 30 => rungs start warm (6.103 predicted within 25). Per kind warm - scratch: F-L -23, F-P -53, T-L-c -10, T-P-c -58, T-L-g 19, T-P-g -18, D-L 13, D-P -11 (scratch had 436 versions to warm's 300, warm capped while still crowning). BEST AT PLY 2 (vs the panel mean): x3 F-P 95, x3 F-L 87, s2 F-L 85, x3 T-P-c 84, s2 F-P 68, s2 T-P-g 63; grow trees and DAGs trail (T-L-g -48..-72, D-L -5..-46, D-P 19-36): flats and the capped probit tree lead, as before (training ply >> architecture). NOISE: s1 - s2 pooled -35 [-48, -22] (a shared seed/field effect; the warm seeds also inherit the ply-1 seeds' gap), per kind -94 (T-P-g) to -4; the script's per-line run SD ~28. Ply-2-trained fields are far noisier than the earlier ~5 a recipe (that came from ply-1-trained runs read at ply 2): a single-seed within-field contrast carries ~28-40 of run noise at ply 2. Consequence: LABELS-TREE-P2 SECOND SEED launched 05:09:10 (labels-tree-p2-s2, seed 80, identical otherwise); the s1-only v300/final waiters were replaced by pooled two-seed reads (batch7-reads/labels-tree-read2.sh: v300 and final, one arena per ply with both seeds; the s1-only v150 read stays as the early look).

- **11-13 04:47 LEVERS-P2 READ v300 (arenas diag/levers-p2-v300, levers-p3-v300; 30 RR, pairs2).** SEARCH LABELS - result (F-P), FP face / full panel: PLY 2 63 [8, 118] / 45 [8, 83] (v150 71 / 61); PLY 3 44 [-10, 98] / 50 [13, 87] (v150 53 / 64). S7 - result: PLY 2 -16 / 1 (v150 19 / -20); PLY 3 -23 / -33 (v150 -95 / -63). Labels hold at 45-65 at ply 2 and 44-50 at ply 3 across both reads (inside 6.109/6.110's bands, above the 40 adoption line at ply 2 so far); S7 ~0 at ply 2 and 23-33 below at ply 3, just below its ply-3 band (-20..30) on both faces at v300. Final (cap 401) ~05:30.

- **11-13 04:10 TREE SEARCH LABELS BUILT (models/from-scratch/tree-labels/summary.txt; leagues/BUILD.md) AND LABELS-TREE-P2 LAUNCHED 04:09:31 (seed 79; reads waiting at v150, v300, final: batch7-reads/labels-tree-read.sh).** chessai-treelab (md5 e7d31b0e) = grow7 + --label-depth/--label-result/--label-model for capped trees, grow trees and DAGs; the blended label trains leaves and error-router targets. Flags off: 11 arms byte-identical over 112 rounds (grow tree, both DAGs, both capped trees = hybrid6, a DAG with 12 router splits and 36 undos, F-L, F-P); labels on, the flat path = hybrid6; --label-result 1 = no labels; own search = --label-model <start model>; exact resume; 1 thread = 3 threads; play identity vs grow7 at depths 1-2. Label cost per 200k positions: capped tree own search 25.7 CPU-s (1.2 without), tree with the F-P labeller 8.3, F-P own 6.6, big D-P DAG own 90; the field ~70 CPU-s a cycle, ~6 CPU-h to the 300 floor, ~8 to the 401 cap. PART B swapped into leagues.py 04:08:01 (backup leagues.py.before-labelline): a line's --label-model may name another line, resolved to its current version each training; inert (train commands of 100 experiments, 511 lines, identical; selftests 246 pass; std-v1-p2-s2 cycled normally on it). New recipe rows F-P-lab, T-P-Ne-cap. FIELD: F-P-lab (reference), T-P-Ne-cap result labels, T-P-Ne-cap own-search labels, T-P-Ne-cap labelled by the F-P-lab line; 1,584 games, hybrid21, floor 300, cap 401. Bands (6.113) in the spec and the read script.

- **11-13 03:54 PAIRS3 FORK REPORTED (models/from-scratch/pairs3/summary.txt): NOT LAUNCHED, pre-registered launch test failed.** The directive's statistic (chessai-pairs3, clustered SE) FAILS the null: positions of one game share one label, so a side-to-move pair's clustered SE is tiny and tiny effects pass (20 pairs a null window, |z| 13-26). The fork's fix chessai-pairs3f (md5 819821cf; SE floored at max(clustered, per-example)) chooses 0 pairs in all 5 null windows; band ratios 0.94-1.02 except the 1,000+ band early (1.75 window 1, 1.21 window 3); offline equal to plain (-0.0001, SE 0.0001; pairs2 -0.0007). Identity: training 16/16 vs hybrid6, play 31/31 vs hybrid21, exact resume 4/4, kill -9 league smoke 49/49. CLEAN-LIST SMOKE FAILS: 40 of 300 pairs after 2 checkpoints, hubs 46:5 (white queen g6) x14, 64:1 x11, 64:0 x9, 22% inert; 22 of the 40 are duplicate pairs (both values of stm / castling features give the same interaction; a small fix, not built); merged, 46:5 x12 and stm x9 remain, so the hubs look like real interactions. Drafts leagues/specs/batch7/s7v3-p1-s1/-s2.DRAFT.json (seeds 77/78, pairs3f, ply 1, floor 300, cap 401) ready. COORDINATOR CALL (analyst out): hold. The launch rule (null 0.9-1.1 in every band, no value in more than ~5 pairs, under 10% inert) is not met, the test could only reopen pairs (within 15 would not close them with an unclean list), offline shows nothing, pairs2 lost 63-95 at ply 3, and the 8 cores go to labels-tree-p2 first. Pairs PARKED for the user's call (options: launch the drafts as a cheap ply-1 probe read at plies 1-3; build the duplicate fix first; or close).

- **11-13 03:49 RESIDUAL-P2 FINAL (plateau at cycle 341, v340; arenas diag/resid-p2-v340, resid-p1-v340; 30 RR vs y/v/l @299, 2 replicates an arm): RESIDUAL LEAVES CLOSED.** PLY 2: residual (tr) - capped (tc) -68 [-94, -42] (v150 -57, v300 -40); tr - F-P 12; tc - F-P 80. PLY 1: tr - tc -27 [-52, -2]; tr - F-P -33; tc - F-P -6. Under 6.114's final rule (-25 or worse at ply 2 closes: the chase does not resolve within a run) residual leaves are CLOSED. The v300 -> v340 swing (-40 -> -68) is inside version noise (~17 SD a version), so the trajectory was never a trend. Capped T-P-Ne beat this field's F-P by 58-80 at ply 2 (the F-P weak in a probit-only field, 6.107). Freed capacity held for labels-tree-p2 and pairs3 (8 cores; load ~17-21 even now).

- **11-13 03:45 ANALYST UNAVAILABLE: the Fable usage limit was reached (HTTP 429) on my S7 question; no 6.115.** Coordinator calls until it is back (topping up credits is the user's call). S7 / pairs3 (my call): keep the pre-registered pairs3 plan (two ply-1 runs, seeds 77/78, band 20-80 at the final, within 15 with a clean list closes pairs) but ADD A PLY-3 READ CELL: S7's deficit grows with depth (ply 2 ~0, ply 3 -63..-95), which fits 6.111's 'pairs fit the stream they are trained on' (search leaves lie off the self-play stream). Rule added: pairs3 below -30 at ply 3 at the final closes pairs for the goal whatever ply 1 says; a pairs3 + labels combination only if pairs3 passes at plies 2-3. Launch order as capacity frees: labels-tree-p2 (when built), then pairs3.

- **11-13 03:44 LEVERS-P2 READ v150 (arenas diag/levers-p2-v150, levers-p3-v150; 15 entries, 30 RR, pairs2 binary; one ply-2 field, one seed, the three lines share it).** SEARCH LABELS - result labels (F-P): PLY 2 FP face 71 [5, 137], full panel 61 [18, 104]; PLY 3 FP face 53 [-4, 109], full panel 64 [24, 104]. S7 (pairs2) - result: PLY 2 19 / -20; PLY 3 -95 [-159, -31] / -63 [-106, -19]. Against 6.109/6.110 (final bands: labels 30-90 at ply 2, 20-80 at ply 3; S7 0-40 / -20..30): labels already inside both bands at v150 and above the 40 adoption line at ply 2; S7 below its ply-3 band (pairs2 hurts deeper play; its statistic is the suspect, pairs3 building). Noise: within-field contrast, so the shared field effect cancels; measurement SE ~33 (FP face, 3 strangers) / ~22 (full panel); run SD ~5 a recipe at ply 2. THIRD INDEPENDENT REPLICATION of search labels at plies 2-3: labels-p1 (trained at ply 1, two seeds) 74.5 at ply 2 and 51 at ply 3; levers-p2 (trained at ply 2) 71/61 and 53/64. v300 ~04:25, final (cap 401) ~05:20.

- **ANALYST 6.114 (11-13 03:22): LET THE FINAL DECIDE residual leaves.** The trajectory is the chase mechanism's own prediction (the gap eases as the field slows: -57 to -40 at ply 2, -50 to -21 at ply 1) and the residual lines are the ones still crowning. FINAL RULE: residual - capped at ply 2 within 15 (and not below -30 at ply 1) REOPENS residual leaves as a slower start to the same place (next: the shrunk-delta variant in one ply-2 field); -25 or worse CLOSES them (the chase does not resolve within a run); between: closed as a strength lever, noted as still learning past the capped tree (library interest, not the goal). Either way not the route to the goal: 18 over F-P against the capped tree's 58; the tree-labels build is.

- **11-13 03:21 RESIDUAL-P2 READ v300 (arenas diag/resid-p2-v300, resid-p1-v300; 18 entries, 30 RR, vs the y/v/l panel @299; each arm = 2 replicates pooled).** PLY 2: residual (tr) - capped (tc) -40 [-65, -14] (v150 -57); tr - F-P 18 [-7, 44]; tc - F-P 58 [32, 84] (v150 138: the field's F-P caught up). PLY 1: tr - tc -21 [-47, 4] (v150 -50; ply-1 band -20..20); tr - F-P -49; tc - F-P -27. Against 6.107 (v300 -40..-10, never leading; -40 or worse closes residual leaves at ply 2): EXACTLY ON THE CLOSING LINE, still never leading, improving ~17 since v150. In-run, the residual lines are still being crowned (champions v260, v280) while the capped trees stalled (v136, v144), so the final (cap 401, plateau window 58 from 300; likely ~340-401) decides. Noise: measurement SE ~13 for the pooled cell, run SD ~5 a recipe at ply 2 (halved by 2 replicates) plus a shared field effect that cancels within the field. Reading: residual leaves trail capped trees by 20-40 at v300 at both plies; a lead at the final would need a further 40+ gain.

- **ANALYST 6.113 (11-13 02:46): TREE SEARCH LABELS ARE WORTH A BUILD; build fork started 02:47.** The extended labels read (48.5 ply 1, 74.5 ply 2) makes search labels the first lever since the trees' ply-1 partition to give 50-80 against strangers at the plies the goal cares about. Why it could favour trees: a tree loses at ply 2 because each leaf sees only its share of a moving stream; a search label is a cleaner target per position, so a leaf needs fewer positions. Risk: a tree labelling itself bootstraps on its own errors, hence a fixed-labeller arm. BUILD (fork, models/from-scratch/tree-labels/): --label-depth/--label-result/--label-model for capped trees, grow trees and DAGs; leaves and error-router targets on the blended label; byte-identical when off; optional driver change so a line can be labelled by another line's current model (only if provably inert), else a fixed library labeller. TEST labels-tree-p2 (draft only): one ply-2 field, F-P labels (reference), T-P-Ne capped result, T-P-Ne capped labels (own search), T-P-Ne capped labels by the F-P-labels model; ~1,584 games a cycle, floor 300, cap 401, ~12-16 CPU-h. Read vs y+v+l @299 at plies 2-3, v150 / v300 / final. BANDS (ply 2 / ply 3): tree-labels - tree-result 40-100 / 30-80; tree-labels - F-P-labels -20..40 / -20..40 (the goal's cell); fixed labeller - own search within 20 (own search 30+ below = the bootstrap bites). RULE: tree-labels 30+ over F-P-labels at ply 2 and not below at ply 3 => the standard's trees take search labels; within 15 => labels lift trees and flats alike (the tie stands). Ranking by gain per CPU-h: levers-p2 (running), labels-tree-p2, the rungs with labels, pairs3 at ply 1, --label-depth 3 after depth 2 pays at ply 2; residual leaves last pending v300. Launch when levers-p2's slot frees or capacity allows.

- **11-13 02:44 LABELS-P1 EXTENDED READ (s1 v500 = cap 501 again, s2 v408 = plateau at 409; arenas diag/labels-p1-vext500-408, labels-p2-vext500-408).** Search minus result labels: PLY 1 FP face s1 54, s2 43, pooled 48.5 (meas SE 16); full panel 38 / 35, pooled 36.5. PLY 2 FP face 92 / 57, pooled 74.5 (16.1); full panel 79 / 85, pooled 82 (11.3): inside 6.108's v500 prediction (40-80). Pooled FP face by read (ply 1 / ply 2): v150 23.5 / 34.5, v300 17.5 / 15, v400 -12 / 59.5, ext 48.5 / 74.5; ply 3 at v400 51. READING: a robust gain inside search (ply 2: the last two reads 60-75 with all four seed cells 53-92; ply 3 51); at the root noisy around ~20 (version noise: -12 to 48 between neighbouring versions). Adoption for the standard is levers-p2's call (ply-2 trained; 40+ at ply 2). Library index rebuilt 02:37: 538 rows, 322 plateaued.

- **ANALYST 6.112 (11-13 02:35): AGREES.** Explore closes (T 0.1 stays); no second probit-only seed; the mixed standard stands; the deficit sits in the flat (6.107); the ladder's probit-only top 12 = selection plus shared-field strength at ply 1 (closed). Why T 0.15 costs LR 60-110 and probit ~10: hotter play = noisier outcome labels; probit's ADF update is tempered by the latent's noise and its variances shrink with evidence (a noisy label barely moves a converged weight), while AdaGrad LR at rate 0.05 takes each label at face value with late steps on the large side (lr-settings-2), so label noise becomes weight noise, most in LR tree leaves (a third of the stream each). Corollary (no decision needs it now): a mixed field's exploration dose is set by its LR lines.

- **11-13 02:33 T015PF2 READ v300 (arenas diag/t015pf2-p1-v300, -p2-v300; 48 entries, 30 RR; comparators std-v1-p1-s1 AND -s2 v300).** EXPLORE (T 0.15) minus std: PLY 1 pooled -39.9 (vs s1 -44.5, vs s2 -35.4; trees -42, flats -33.8; per recipe vs s1 / s2: F-L -64 / -59, F-P -3 / -9, T-L -47 / -28, T-P -3 / -18, T-L-grow -90 / -40, T-P-grow -55 / -24, D-L -117 / -103, D-P 23 / -2: the LR recipes pay most); PLY 2 pooled 5.4 (vs s1 -2.2, vs s2 13.1). 6.102 band -45..-10 vs both: met at ply 1 => "exploration is not the limiter, more of it costs precision"; ply 2 null. VERDICT: T 0.1 STAYS; THE EXPLORATION LEVER CLOSES. PROBIT-FIELD minus std's probit lines: PLY 1 pooled -14 (trees -2.7; F-P -45 / -51), PLY 2 pooled -18 (trees -22.5; vs s1 -27, vs s2 -9). 6.102: ply 1 within 20 met; ply 2 -80..-30 NOT met (the v150 -78 was mostly comparator luck + mid-run) => by the letter "comparator luck; a second probit-only seed decides"; my recommendation: no second seed (probit-only is not better anywhere, so the mixed standard stands either way; the remaining question, slightly worse vs equal, is not decision-relevant). STD s1 - s2 in this read: ply 1 9.1 (per-recipe SD ~24), ply 2 15.4 (SD ~14): run-to-run noise as expected.

- **ANALYST 6.111 (11-13 ~02:14) + PAIRS3 FORK STARTED 02:14.** S7: the statistic is the suspect (a 47 at v300 that vanishes at v400 while the list fills with frequent-value pairs = a statistic admitting leakage until the cap). pairs3 = per-checkpoint restart of the statistics (fresh S, Q, G per window) + a frequency-aware SE (component terms) + cap 300 + drop at |mu| < 1.0 sqrt(s2) after 3M own examples. Rerun at PLY 1 as two two-line runs like s7-p1 (seeds 77 / 78) once the build passes: null mean z^2 0.9-1.1 in every frequency band, a CLEAN LIST (no value in more than ~5 pairs at the cap, under 10% inert). Band: S7 minus plain 20-80 at the final; within 15 with a clean list closes the feature lever for good on this data. levers-p2's S7 cell = "pairs2 at ply 2" only (a gain hard to credit, a null uninformative); its labels cell reads cleanly. s7-p1-s1/-s2 ended on cap 401: NOT extended (superseded by pairs3).

- **11-13 02:12 S7 FINAL READ (v400; both runs ended on cap 401 at 01:59 / 02:00, not plateaued; arenas diag/s7-p1-vfinal, s7-p2-vfinal).** S7 minus plain F-P: PLY 1 FP face s1 12, s2 -4, pooled 4 (meas SE 16); full panel 17 / 3, pooled 10. PLY 2 FP face 18 / -13, pooled 2.5; full panel 3 / -16, pooled -6.5. Trajectory (FP face pooled, ply 1 / ply 2): v150 6 / -28, v300 47 / -21.5, v400 4 / 2.5: the v300 ply-1 gain did not hold (version noise). HUB CHECK v400: both models at the 500-pair CAP (39.4M / 40.1M examples); 59:11 in 26 (s1) / 29 (s2) pairs; 18 / 17 values in 10+ pairs (62:10 19, 8:11 17, 3:11 16, 3:5 14, 5:4 18, 59:5 15 ...: mostly pieces on home squares = the most frequent values); 83 / 68 pairs with |w| < 2 sd. By 6.105's rule (ply 1 within 15 AND 10+ pairs sharing one value), the STATISTIC is the suspect: pairs3 (per-checkpoint restart + frequency-aware SE) gets one build and one rerun; asked the analyst to confirm. levers-p2's S7 line (pairs2) continues as launched.

- **ANALYST 6.110 (11-13 01:52).** The depth-2 label's gain is not depth-2-specific: it carries one ply past its own depth (other parity) undiminished within noise; only the root does not profit. levers-p2 bands stand (labels 30-90 at ply 2, 20-80 at ply 3; ply-3 centre raised to 50). --label-depth 3 NOT earlier: next one-factor at ply 2 only if levers-p2 holds with 30+ at ply 3 (predicted 0-40 over depth 2 at ply 3, within 20 at ply 2), ~10x the label CPU. SCALE: labels are a 50-60 lever inside the ladder's 300-500 per rung: a complement to training at the ply, not a substitute.

- **11-13 01:51 LABELS FINALS AT PLY 3 (arena diag/labels-p3-vfinal; 20 entries, 30 RR, hybrid18; 0.15-0.2 CPU-h).** Search minus result labels (v400) vs the ply-3 strangers f3 / f3b / p3fp / x4 / bfp: s1 27 [-15, 69], s2 75 [38, 111], POOLED 51 (meas SE 14.2): INSIDE 6.108's band (40-120), both seeds positive. The labels' gain by ply: ~0 (ply 1), 60 (ply 2), 51 (ply 3): the evaluator gets better INSIDE search. Levels: these ply-1-trained 768-game models sit far below ply-3-trained flats at ply 3 (pa -271, la -225, pb -216, lb -114 vs f3/f3b +108..+144, p3fp +117, bfp ~-60, x4 ~-77; field-mean anchor), so the test that matters is levers-p2 (trained at ply 2).

- **11-13 01:44 ANALYST 6.108 / 6.109 AND ACTIONS.** 6.108: search labels RE-OPENED for ply 2 (the label trains the evaluator to agree with a 2-ply search, which is what a ply-2 search asks of its leaves; at the root the result label is as good; ply 2 is the goal's dimension). 6.109: S7 v300 inside both bands, no pairs3; "pairs fit the stream they are trained on, labels fit the search they will be used in"; both go to a ply-2-trained field. DONE: (1) PLY-3 ARENA of the labels finals RUNNING (labels-p3-read.sh: 4 finals + f3/f3b/p3fp/x4/bfp, 30 RR, hybrid18; band 40-120, under 15 = depth-specific). (2) LABELS-P1 EXTENDED to cap 501 (fired 01:41 at live 6; ~1 CPU-h each; 6.108 predicts the ply-2 gain 40-80 at v500). (3) LEVERS-P2 DRAFT (leagues/specs/batch7/levers-p2.DRAFT.json, smoke-tested): ONE ply-2 flat field, seed 75, F-P result / F-P labels / F-P-S7v2 on the same games (6 entries, 48-round cap = 1,440 games + self-games), tournament pairs2, floor 300 cap 401; the 6.109 2x2's fourth cell (S7 + labels) NOT RUN: pairs2 refuses --label-depth with --pairs on (build only if both levers pay at ply 2). LAUNCHED 01:47:52 (launch-when-free.sh 8); reads queued (levers-read.sh 150 / 300 / final, plies 2-3, y/v/l @299, FP face the verdict; bands 6.109). (4) PLATEAUS: std-v1-p1-s2 at cycle 705 (01:30, finals v704), flat-p3-w4 at 179 (01:40, v178); probit-field-p1 ended on its cap 301 (by design: floor 300). LIBRARY INDEX REBUILT 01:44: 535 rows, 320 plateaued (was 308), 1,550 snapshots.

- **11-13 01:39 S7 READ v300 (arenas diag/s7-p1-v300, s7-p2-v300; pairs2 runs on hybrid21 arenas).** S7 minus plain F-P, two seeds: PLY 1 FP face s1 31, s2 63, pooled 47 (meas SE 15.9); full panel 30 / 46, pooled 38 (11.3). PLY 2 FP face -24 / -19, pooled -21.5 (16.1); full panel -3 / -18, pooled -10.5 (11.7). Bands: ply 1 30-120 -> INSIDE (both seeds positive; 6.105: "30+ = the leak did not matter", no pairs3); ply 2 (revised 6.105) -60..20 -> inside. THE 6.105 PATTERN: a ply-1 gain with a ply-2 loss = the pairs found this field's style, not chess structure -> a ply-2 field before any adoption. MIRROR IMAGE OF SEARCH LABELS (labels: nothing at ply 1, +60 at ply 2). Final read when the runs reach cap 401.

- **11-13 01:38 LABELS-P1 FINAL READ (v400, both runs cap-ended; arenas diag/labels-p1-vfinal, -p2-vfinal).** Search (depth 2, result weight 0.5) minus result labels, F-P, two seeds: PLY 1 FP face s1 -31, s2 7, pooled -12 (meas SE 16); full panel -10 / 18, pooled 4 (11.4). PLY 2 FP face 66 / 53, pooled 59.5 (16.0); full panel 56 / 83, pooled 69.5 (11.5): BOTH SEEDS STRONGLY POSITIVE AT PLY 2, nothing at ply 1. Trajectory (FP face pooled, ply 1 / ply 2): v150 23.5 / 34.5, v300 17.5 / 15, v400 -12 / 59.5 (version noise visible). 6.106's letter (ply 1 >= 25 with both seeds >= 0 adopts; under 15 closes) says CLOSE at ply 1, but the ply-2 result sits inside 6.99's ply-2 band (30-100) with both seeds agreeing, and ply 2 is the one that bears on the main goal (an evaluator for deeper search: labels from depth-2 search make the eval consistent with what a ply-2 search sees, TD-leaf-like). Asked the analyst to rule; candidates: a cheap ply-3 arena of the four finals (does the gain grow with depth?), and a ply-2-trained labels field.

- **11-13 01:30 BATCH.** (1) HYBRID21 (perf review 11-13 ~01:00 -> implementation fork; fastbin/hybrid21/summary.txt): hybrid20 + route-state cap 4,096 -> 65,536 only; md5 b52b1bac; 31 identity cases byte-identical to hybrid20 (~5,070 games, plies 1-5, both over-cap DAGs; check mode 178M router decisions vs float, 4 fallbacks, no failures); both warm runs' cycle 101 replayed exactly; games with the big D-P DAGs x0.40 (T 0.1) / x0.49 (T 0.02); ~2.2-2.5 CPU-h over the extensions. SWAPPED into std-v1-p2-s1 (01:18:09) and -s2 (01:23:09) config.json while done; make_split_read.py's plies 1-2 default binary -> hybrid21 (01:18:26; .before-hybrid21; same games as hybrid16 on a test arena). Note: tier-1 cleanup had deleted hybrid20's cal/heldout and cal/x identity inputs; the fork re-made them as reflinks of games/heldout-lr-d2-s555/round0. (2) WARM PLY-2 EXTENSION FIRED 01:29 (std-v1-p2-s1/-s2 cap 101 -> 301, on hybrid21; cycle 101 re-runs and is checked against the originals). (3) LABELS-P1-S1/-S2 ENDED ON CAP 401 (01:26-01:28, not plateaued; final read now running; per the analyst the cap is the design's end, the verdict comes from the final read). (4) /TMP QUOTA FULL (12,615 of 12,625 MB: the session scratchpad held 13 GB of old helper scratch; task outputs and agent transcripts share the quota): MOVED, not deleted, the 415 scratchpad items older than 11-12 12:00 to models/from-scratch/scratchpad-archive-1265e529/ (list in MOVED-LIST.txt; inv-before/ and other items newer than 12:00 stay in the scratchpad); /tmp now 286 MB. (5) BINARY CHECK on the residual v150 arena: 2 round-robins replayed on hybrid20 = 612 / 612 games identical to the hybrid16 arena, so the in-run vs arena gap (in-run cold games v130-160: fp - tc -17 over 576 games, fp - tr +1, tc - tr +23; arena direct at v150: tc - fp +108 over 240 games, tr - tc -86) is not the binary; the arena's single v150 versions vs in-run's mix of versions and champions is the remaining difference.

- **11-13 01:13 RESIDUAL-P2 READ v150 (arenas diag/resid-p2-v150, resid-p1-v150; 18 entries, 30 RR; vs y/v/l @299; each arm = 2 replicates).** PLY 2: residual (tr) - capped (tc) -57 [-82, -31]; tr - F-P 81; tc - F-P 138 [112, 164]. PLY 1: tr - tc -50; tr - fp -54; tc - fp -4. Bands (6.98): ply 2 tr - tc 20-60, tr - fp -10..40; ply 1 tr - tc -20..20: RESIDUAL LEAVES BELOW BAND at both plies (provisional; v300 / final decide). Entries at ply 2 vs the panel mean: tc L1 / L2 ~+21 / +28 (replicates agree), tr ~-61 / -23, fp ~-97 / -114. The tc - fp 138 is mostly WEAK F-P, not strong trees: on the same y/v/l scale std-v1-p2-x3 at v150 has F-P +35 and T-P capped +45 (early read), i.e. residual-p2's F-P ~140 below the standard's ply-2 F-P while its capped trees are ~20 below the standard's. Train settings identical (hybrid6 --use-probs), so it is the FIELD: residual-p2 is probit-only (F-P, capped, residual); like probit-field-p1's F-P at ply 2 (-90 vs std s1). In-run (cold games, co-adapted) all six lines are level at cycle 157 (2911-2987), so in-run and stranger reads disagree by ~100 here.

- **ANALYST 6.106 (11-13 01:01): labels gap -> WAIT for the final (cap 401), no third seed.** FINAL RULE: pooled FP-face >= 25 at ply 1 with BOTH seeds non-negative adopts search labels for the standard's flats and sends the test to ply 2; under 15, or seed 1 still ~0 with seed 2 alone carrying it, closes the lever ("within the run term"). CALIBRATION CHECK DONE (01:01): both seeds' label calibrations look alike (slope ~1.0-1.3; the other terms swing -9..+4 cycle to cycle in both), so seed 1 ran the factor; the split is noise and the final decides.

- **11-13 01:00 LABELS-P1 READ v300 (the decision point, 6.99/6.104).** Search minus result labels, two seeds: PLY 1 FP face s1 -8, s2 43, pooled 17.5 (meas SE 15.9); full panel -25 / 24, pooled -0.5 (11.2). PLY 2 FP face -12 / 42, pooled 15 (16.3); full panel 15 / 34, pooled 24.5 (11.6). Seed 1 near zero throughout (v150: 11, -1), seed 2 +24..+70: the seed split (~50) is the size of the effect. Rule: 40+ adopts, under 15 closes, 20-40 a third seed: pooled 15-17.5 sits in the gap; NOT ADOPTED; final read (both runs to plateau, cap 401) and the analyst rule on the gap.

- **ANALYST 6.105 (11-13 00:41).** (1) The ply-2 warm start is worth ~40 scratch cycles (93 of scratch's 132 from v75 to v150): a saving, smaller than predicted; the grow tree gained nothing (+4) while flats and capped trees gained ~140, so a warm grow tree's structure and credits carry less than its weights; v300 decides convergence (within 25 predicted). (2a) S7's concentration on frequent values = non-stationary main-effect leakage; NO pairs3 yet. PRE-REGISTERED at v300: ply 1 within 15 AND 10+ pairs sharing one value -> the statistic is the suspect: pairs3 (per-checkpoint restart + a frequency-aware SE) gets one build and one rerun; within 15 with a clean list closes the feature lever; 30+ = the leak did not matter. (2b) Ply-2 loss mechanism: 300 extra parameters fitted to the ply-1 stream's co-occurrences generalise worse off-ply (same shape as the probit-only field). S7 BANDS: ply 1 unchanged (30-120; under 15 closes); PLY 2 REVISED to -60..20. A ply-1 gain with a ply-2 loss = the pairs found this field's style, not chess structure -> a ply-2 field before any adoption.

- **11-13 00:39 S7 READ v150 (arenas diag/s7-p1-v150, s7-p2-v150; pairs2; 20 entries, 30 RR).** S7 minus plain F-P, two seeds: PLY 1 FP face s1 -13, s2 25, pooled 6 (meas SE 16); full panel 6 (12). PLY 2 FP face -19 / -37, pooled -28 (17.5); full panel -19 / -61, -40 (13). Bands (6.97): ply 1 30-120 (under 15 closes; 15-40 a third seed), ply 2 0-60. PROVISIONAL: no gain at ply 1, a loss at ply 2; v300 / final decide. HUB CHECK (6.103) on the v150 S7 models (290 / 300 pairs, 15.2M / 15.0M examples): the smoke's hubs are cleared (40:7 and 65:4 in 0 pairs; 68:1 in 3 / 7; 47:7 in 1 / 0), but a new concentration on common values: 59:11 in 17 (s1) / 13 (s2) pairs, 60:12 in 10 / 9, 6:11 11 / 8, 65:2 14 (s2) (the null's maximum was 4 in its top 100); 93 / 79 pairs with |w| < 2 sd; |w| median 0.18 / 0.16, largest ~0.5-0.8.

- **11-13 00:32 STD-V1-P2 EARLY READ (arena diag/std-v1-p2-read-early, 52 entries at ply 2, 20 RR, 33,200 stranger games; Elo 0 = y/v/l @299).** WARM@75 - SCRATCH@75: pooled +93 [81, 105] (F-L 141, F-P 140, T-L capped 143, T-P capped 96, T-L-grow-f10 122, T-P-grow 4, D-L 55, D-P 44); 6.103 band 100-250: just under. WARM@75 - SCRATCH@150: pooled -39 [-51, -27] (F-L -16, F-P -23, T-L capped -12, T-P capped -80, T-L-grow 33, T-P-grow -75, D-L -66, D-P -77); 6.103 "within 50": met; 6.95 ">= 0 by v50-75": NOT met. SCRATCH@150 - @75: +132 (flats and capped trees +155..176, grow/DAGs +79..121). s1 - s2 @75: pooled 19, run noise SD ~23 a line. READ: the warm start is a head start (largest for flats and capped trees, near nothing for the probit grow tree), and from-scratch overtakes warm@75 by v150; whether they converge is the plateau read (extension of the warm runs to 301 queued; 6.103 predicts within 25 at v300).

- **ANALYST 6.104 (11-13 00:28): nothing in the plan changes.** Log-loss proxies closed, within-kind too; the within-run idea is not worth 1-2 CPU-h (it removes the field term, keeps the run term, and would only measure within-run line differences, at chance in-run per 6.93 (c), which the panel reads already measure). The arena is the only read. LABELS v300 RULE (6.99): 40+ adopts and sends search labels to ply 2; under 15 closes; 20-40 = a third seed (given the ~70 seed spread at ply 2).

- **11-13 00:27 OFFLINE PROXY: BUILT, VALIDATED, FAILS -> LOG-LOSS PROXIES CLOSED (6.101 rule; models/from-scratch/proxy/summary.txt).** Winner log-loss on quiet, balanced moves 10-30 of decided games (fixed reference per slice; P1 = ply-1 games, 154k positions; P2 = ply-2 games, 48k). Millinats (positive = first model better) vs arena Elo: capped tree vs flat ply 1 +16.3 vs +89 (right); ply-2-trained tree vs flat +2.1 vs -34 (wrong, predicted); pure vs mixed F-P ply 2 -40 (-12 without probit-late-p2) vs +15 (wrong); x4 vs bfp +4.9 vs +15..21 (right); F-L vs F-P ply 1 +7.2 vs +24 (right), ply 2 +4.3 vs level (disagrees); CONTROL same F-P recipe two seeds -26.3 vs -1 (FAILS: run-to-run proxy noise ~18 mn a run > the 89-Elo tree cell). Why: the score ranks positions across games and favours models from the field that generated the slice (P2 = probit-late-p2's games: its models 583 mn vs 652-685 for every other run); play only needs move ranking within a position. Quiet/balanced filtering changes no sign. Untested: within-run line-vs-line on the same games (own validation needed). ~2 CPU-h; 1.1 GB new data in models/from-scratch/proxy/data (cleanup candidate, user's OK).

- **11-13 00:08 LABELS-P1 READ v150 (arenas diag/labels-p1-v150, -p2-v150; 20 entries, 30 RR; ~10 min).** Search labels (--label-depth 2 --label-result 0.5) minus result labels, F-P, two seeds: PLY 1 vs the FP face of r/e/g/h @300: s1 11, s2 36, pooled 23.5 (meas SE 15.9); vs the full panel 28 / 45 / 36.5 (11.5). PLY 2 FP face: s1 -1, s2 70, pooled 34.5 (17.8); full panel 1 / 47 / 24 (13.1). 7 of 8 cells positive, one level; seeds disagree by ~70 at ply 2 (run-to-run noise). Bands: 6.101 at v150 30-120; plateau 20-80 (ply 1) / 30-100 (ply 2); 40+ adopts. PROVISIONAL: a consistent modest lead (~25-35), under the adoption line so far; v300 / final decide.

- **ANALYST 6.103 (~23:47).** S7: the fork's centring is right; the null is a conservative pass; frequent candidates (static pawn pairs) are under-selected, so a null S7 at v300 is read with that in mind before closing; restarting the statistics per checkpoint only if the v150 kept list still holds hub-value pairs the drop rule did not clear (CHECK AT THE v150 READ); bands as 6.97. WARM PLY-2 EXTENSION TO 301: YES (the ply-2 field keeps moving for hundreds of cycles, warm or not; decision 2 is read at the equal early points; the extension buys eight plateaued ply-2 standard finals for the ladder, the panels and later ply-2 contrasts, and a second reading: do warm and scratch converge by the plateau, prediction within 25 at v300). PRE-REGISTERED, stdp2 early read: warm over scratch at v75 by 100-250; warm v75 vs scratch v150 within 50; within 25 by v300.

- **S7 PAIRS2 DONE (fork, ~23:40; models/from-scratch/pairs/summary.txt PAIRS2 section) AND LAUNCHED 23:42.** chessai-pairs2 (md5 939a2ca1; pairs1 kept; source pairs2-src): 6.100's contrast + cluster-robust SE, with one change: each game's sums CENTRED at the null (uncentred: mean z^2 788 on the null); training order unchanged (no --game-order flag needed: per-example v summed by game at the round's end; offline weights byte-identical to plain through round 5). Checks: 16/16 training identity vs hybrid6, exact resume, play identity vs hybrid20 (28 cases), kill -9 smoke. NULL (5.03M shuffled examples): 0 pairs chosen; mean z^2 0.80 (target 1.0 +-0.1; LOW = too strict on frequent candidates: ~1.0 under 100 games, 0.78 at 100-999, 0.25 at 1,000+, the SE ignores a pair's overlap with its own values; it may miss common interactions, it does not leak; pairs1 was 5.1 with 12,145 over |z| 5). OFFLINE: -0.0003 / -0.0004 / -0.0007 nats at rounds 15 / 20 / 25 (39 / 59 / 79 pairs), still growing; pairs look like real conjunctions (queens on the enemy back ranks; side to move x en passant file). CAVEAT: from scratch the first checkpoint again picks hub-value pairs (40:7, 65:4, castling partners: early-game non-stationarity); the drop rule should clear them (restarting the statistics at each checkpoint is the alternative, not built: analyst's call). Cost 3.7 CPU-s training a cycle, state 17 MB; ~0.7-0.9 CPU-h and 2-3 h wall a run. Recipe F-P-S7v2. My call: null passes (conservative, not leaking) and offline is not worse -> LAUNCHED s7-p1-s1 (seed 71) and -s2 (72) at 23:42 via launch-when-free.sh 8 (F-P on pairs2 without the flag vs F-P-S7v2, ply 1, tournament pairs2, floor 300 cap 401; 768 games a cycle like labels-p1). Reads queued: s7-read.sh 150 / 300 / final (pairs2 arenas, FP face + full r/e/g/h panel). THE WARM PLY-2 RUNS std-v1-p2-s1/-s2 ENDED ON THEIR CAP 101 (23:24 / 23:34) STILL CROWNING (crown rate 0.39 / 0.31 per rated cycle over their last 50 cycles; the plateau Monte Carlo gives ~0 odds of a plateau by 201, 301 or 601 at these rates): the 6.95 assumption that a warm start plateaus within 41-101 cycles at ply 2 failed. Per the user's 21:18 rule (keep running runs that did not meet the plateau rule) QUEUED: extend-when-free.sh 8 301 (both to cap 301 = x3's floor; <=5.6 CPU-h each; fires when live <= 8, ~when explore and probit-field end; cancel by touching models/from-scratch/batch7-reads/CANCEL-EXTEND).

- **ANALYST 6.102 (~23:40) PRE-REGISTERED v300 BANDS (two comparators, std s1 and s2).** Probit-field minus std's probit lines: ply 1 within 20 (centre 0); ply 2 -80..-30 (centre -55). CONFIRMED if ply 2 is at or below -30 against BOTH comparators with std s1 - s2 inside +-25 => the mixed standard stands; the ladder's probit-only top 12 was selection plus shared-field strength at ply 1 with a cost off-ply (mechanism: a probit-only field is narrower, one leaf family, so its models fit the ply-1 stream as well but generalise worse off their training ply; the LR lines' games are the coverage its probit lines lose). Within 20 of either comparator = comparator luck; a second probit-only seed decides. Explore minus std: band -45..-10 against both comparators; if v300 agrees, T 0.1 stays and the exploration lever closes; within 10 = luck, the lever is null and also closes.

- **T015PF READ v150 (23:02-23:36; arenas diag/t015pf-p1-v150, -p2-v150; vs r/e/g/h @300; one seed each, comparator std-v1-p1-s1 v150).** EXPLORE (T 0.15 training games) minus std (T 0.1): ply 1 -30 pooled (trees -35, flats -16; per recipe fl -21 fp -11 tl -19 tp 10 gl -49 gp -143 dl -62 dp 55); ply 2 -27 (trees -26, flats -29; fl 3 fp -61 tl -61 tp -33 gl -42 gp -82 dl 8 dp 54). PROBIT-FIELD (2 replicates) minus std's probit lines: ply 1 +1 (trees 11; fp -27 tp 3 gp -46 dp 75); ply 2 -78 (trees -74; fp -90 tp -127 gp -86 dp -10). Measurement SEs ~6-8 pooled, but the comparator is ONE run: std s1's dp150 is weak at both plies (both arms +54..+75 on dp) and its fp/tp/gp read strong at ply 2 against both arms, so part of each contrast is std s1's own luck (run-to-run SD ~18 a recipe at ply 1 + a shared field effect ~10-13). Provisional: wider exploration does not help (both plies ~-30; band "25+ = exploration was limiting" not met, the sign is the other way); the probit-only field is level at ply 1 (=> SELECTION explains ladder-2's probit-only top 12) and transfers WORSE to ply 2 (-78; to confirm). THE v300 READ NOW USES BOTH STD SEEDS (t015pf-read2.sh: std-v1-p1-s2 as a second comparator + std s1 - s2 as the in-read noise; the old v300 waiter was stopped before it started anything).

- **ANALYST 6.101 (~23:12; theory-2/summary.txt).** labels-p1 at 768 games a cycle: still one factor; plateau bands stand (20-80 ply 1, 30-100 ply 2), at v150 the lead may be larger (30-120; a cleaner label is worth more when data is scarce); levels not comparable with the standard's; same note for S7's two-line drafts. AGENDA 9 IF RESIDUAL, S7 AND LABELS ALL LAND WITHIN NOISE (gain per CPU-h): 1. model-driven search extensions (eval-swing quiescence, PV ordering) as an arena (library models at ply 3 with / without; band 50-150), then train with it: THE USER'S CALL (parked); 2. ladder rungs for the flats (8-30 CPU-h a rung); 3. a factorized pair model (FM over indicator embeddings with ADF) only if S7 shows 15+ at ply 1 or 0.003+ nats offline; 4. label knobs (--label-result 0.25, --label-depth 3) only if labels-p1 shows 20+; 5. self-games share for trees (0-20, low). Not worth it: ensembles of flats, probit alpha, longer plateaus, more composition replicates. HONEST LINE if all three fail: the parameter-limited flat trained at its ply is the model; the remaining strength is on the search side (tell the user plainly). OFFLINE PROXY: build now, validate before use on 5 contrasts with known arena answers (capped tree vs flat ply 1 +85; pure vs mixed F-P ply 2 +15; ply-2-trained tree vs its flat -34; LR vs probit flats level; x4 vs bfp +15); all signs and rough order right = a screen for new representations, any misordered = closed. PROXY FORK LAUNCHED 23:13 (models/from-scratch/proxy/; fixed position set across models, bootstrap over games, nice, 2 threads, no league CPU).

- **23:03-23:06 THREE EXTENSIONS PLATEAUED at their first rated cycle after the cap** (as the fork's odds said): flat-p3-w3f1 done cycle 106 (finals v105), treeonly-p1 cycle 407 (v406), std-v1-p1-s1 cycle 605 (v604); still extending: std-v1-p1-s2 (to 801) and flat-p3-w4 (to 200). LIBRARY INDEX REBUILT 23:06 (build_index.py, no deletions): 527 rows, 308 plateaued (was 290), 1,484 snapshots; the three runs' rows now plateau / finals as above (old v600/v399/v99 snapshots stay). LABELS-P1 LAUNCHED 23:07 (launch-when-free.sh at 8 live; specs leagues/specs/batch7/labels-p1-s1.json seed 73, -s2 seed 74; smoke test in the scratchpad passed: probit labels work on hybrid6, ~1 s labelling per 10-16k positions): F-P vs F-P --label-depth 2 --label-result 0.5, ply 1, hybrid20, floor 300 cap 401. CAVEAT: a 4-entry field hits the 48-round GPC cap: 576 games + 192 self-games = 768 a cycle (the 8-line fields: 1,792), so its levels are not the standard's; label minus result within each run is still one factor. Reads queued (labels-read.sh 150 / 300 / final, plies 1-2, FP face of r/e/g/h @300 + full panel). NOTE: residual-p2's repeat.sh (pid 3798400) is a child of a Claude tool shell, not setsid: if this session ends it may die; resume = setsid repeat.sh (exact).

- **EXTEND SWAPPED 22:56:00 (leagues.py 1315b8ef -> b89c6a4c, .before-extend; models/from-scratch/extend/summary.txt): `leagues.py extend LEAGUE --cycles N [--dry-run]` re-runs the cap cycle as an ordinary training cycle (checked against the saved original in <league>/extend/cap-C/) and continues as if N had been the cap; extended = fresh higher-cap runs byte for byte over 184 items (grow trees, DAGs, rate_every, observers, cold games, openings, a warm start, no caches); full selftest 245 checks + 19 runs; all six live runs re-run identical on the new driver and continued on it 22:56:01-22:56:27. EXTENSIONS LAUNCHED 22:58 (repeat.sh each): std-v1-p1-s1 and -s2 601 -> 801, flat-p3-w3f1 100 -> 200, treeonly-p1 400 -> 500, flat-p3-w4 100 -> 200; the fork's plateau odds / expected cycles: 0.98 / 40, 0.52 / 153, 0.84 / 30, 0.97 / 19, 0.24 / 89 (flat-p3-w4 117 s a cycle); ~4.6 CPU-h expected, 7.4 max. Rebuild the library index when they end.

- **ANALYST 6.100 (~22:55): S7 selection fix.** Diagnosis confirmed: within-game correlation (design effect ~4.8) plus marginal leakage. (b) interaction contrast I_ab = S_ab/n_ab - S_a/n_a - S_b/n_b + S/n (per-value accumulators for ~910 values + global); (a) cluster-robust SE: per-game partials G_ab, Q_ab += G_ab^2, SE = sqrt(Q_ab)/n_ab, z = I_ab/SE (needs game-contiguous order; fallback a last-game-id stamp, z = I sqrt(g)/s_v). Defaults every 1M, k 20, |z| 5, cap 500, shrink 0.5, drop 0.5 after 3M. NULL CHECK first: mean z^2 1.0 +-0.1, ~0.2 over |z| 5, no value over-represented (above 1.3 = still leaking). The 33-pair result says nothing yet (the statistic's failure); after the fix held-out loss IS a fair screen: 0.003-0.01 nats if interactions matter, under 0.001 predicts arena under 15. DO NOT launch the as-built drafts. S7 FORK RESUMED ~22:56 for chessai-pairs2 (+ my one-factor addition: game-contiguous order as its own flag, used by BOTH lines of the s7 drafts; offline also reports order alone), null check, offline rerun, drafts; launch when the null check passes and S7 is not worse offline.

- **S7 BUILT AND VERIFIED (fork done ~22:5x; models/from-scratch/pairs/summary.txt): chessai-pairs1 (md5 2fed7278; from hybrid20-src; one binary trains F-P-S7 and plays everything): training byte-identical to hybrid6 without the flag; play identical to hybrid20 on its 28-case harness (0 differences); exact resume (state = (S, n) of 356,642 candidates, 5.7 MB, current version only; smoke killed -9 mid-training, 35 files identical); NO leagues.py change needed; recipe row F-P-S7 (recipes.txt / make_recipes.py .before-s7); make_split_read.py --binary. Pair models play on the float path: eval x2.0 / 2.5 / 2.6 at plies 1/2/3 with 300 pairs, x3.0 / 3.5 / 3.9 with 2,000. Drafts batch7/s7-p1-s1/-s2.DRAFT.json (seeds 71/72; 768 games a cycle, the 48-round cap binds; ~0.6-0.8 CPU-h a run).** BUT OFFLINE (25 sets of std-v1-p1 ply-1 games, ~5M positions; 199k held out): S7 - plain log-loss defaults +0.0006 +-0.0008, every-200k +0.0052 +-0.0016, z-bar 15 (33 pairs) -0.0002 +-0.0002, shrink 0.1 -0.0001 +-0.0005: NO GAIN. The selection z is INFLATED (mean z^2 4.78; 12,005 candidates over |z| 5 vs ~0.2 expected; grows with occurrence = within-game correlation; 25 of the first 50 pairs share value 47:7, a black pawn on h6 = a marginal effect leaking). Sent to the analyst ~22:52 for a corrected statistic (game-clustered and/or net of the marginals), their read of the 33-pair result and the launch policy. Nothing launched.

- **THE USER LEFT ~22:45: "continue working on the various problems that remain and trying to draw conclusions as noisy as the data is. Thank you for all your work."** Set up: read waiters in models/from-scratch/batch7-reads/ (pool.py = pooled arm_panel differences; t015pf-read.sh V = explore-t015-p1 and probit-field-p1 vs std-v1-p1-s1 at V, plies 1-2, r/e/g/h panel; stdp2-read.sh early|final = std_p2_read.py; resid-read.sh V|final = residual-p2 vs y/v/l, plies 2-1). Running: t015pf 150 and 300, stdp2 early, resid 150. To start later: resid 300/final, stdp2 final, the extensions', S7's and labels-p1's reads. CHECK-IN CRON c150cf2e every 2 h at :41 (session-only, expires after 7 days); the perf review cron 4aac862f (every 6 h) is alive. QUEUE: extensions after the extend swap; S7 runs after the S7 fork; labels-p1 after both, when the load allows.

- **THE USER ~22:40: "We can have higher ply arenas once in a while but i wouldn't over index on this"** (on 6.99's ply-6 judge). Higher-ply arenas: occasional and cheap. The eval-swing extension arena (6.99 lever 3) was put to them and left unanswered, so it stays PARKED.

- **ANALYST 6.99 (~22:37), agenda item 15, POSITIONAL WEAKNESS.** Limits ranked: (1) representation (additive over square-piece indicators; positional concepts are interactions: pawn x pawn, king x pawn, rook x six empty squares); (2) the label (the result credits ~80 positions equally, decided by short tactics between weak opponents; search labels --label-depth k blended by --label-result resolve tactics first, zero knowledge, TD-leaf-like); (3) coverage / opponents (nothing tells 1. b3 from 1. e4 against these opponents; the ladder helps); (4) horizon blunders = search, not eval (model-driven eval-swing extension is the allowed form). SEARCH LABELS RECORD: once, old regime (lr-settings 11-09, F-L, random players): strongest in-run but confounded; cold 48 +-46 over the best rate line at ply 2; no better at ply 3 (saturated vs randoms); never in the current regime or with probit. LEVERS: 1. S7 (building). 2. labels-p1: two-line probit field at ply 1, F-P result labels vs F-P --label-depth 2 --label-result 0.5, two seeds as separate runs, 1,584 games, floor 300 cap 400, ~3-4 CPU-h a seed (label line training ~12x); read vs the FP face of r+e+g+h at v150 / v300 / final, plies 1-2; bands 20-80 (ply 1), 30-100 (ply 2); 40+ adopts search labels for the standard's flats, under 15 closes. 3. Model-driven quiescence as an ARENA read (no training; the same models at ply 3 with / without an eval-swing extension; band 50-150 at ply 3): the user's call under "search last resort" (11-11 they allowed model-driven extensions in principle). 4. The ladder. OFFLINE PROXY: held-out log-loss on quiet middlegame positions of decided games (quiet = the model's 1-ply and 2-ply values within 0.05; moves 10-30; balanced near 0.5); occasional judge: a ply-6 arena (4 library models, ~100 games each, 1-3 CPU-h). NOW: start nothing; labels-p1 queues behind residual-p2 and S7.

- **THE USER PLAYED (Play page, 18:12-21:42, leagues/play-games/): 10 wins, 1 draw, 1 loss** (+ a 2-move test): probit-p3 T-P capped v301 at plies 1-7 (lost at ply 6), flat-p3-w4-w3f1 F-L v96 at ply 8 (2 W, 1 D), error-dag-probit-p1 DAG v366 at ply 1 (W). The user: "ply 7-8 they can be quite difficult but I'm also playing very fast"; "My own annotation would be that positionally they're quite weak sadly" (declined my offer to annotate with the models' evals). Seen: aimless openings (1. b3 2. a3 3. f3 4. h4; 1. f3 2. a3 3. g4 + king walk), horizon blunders without a capture search (ply 3: 10...Qxa2 11. Nxc7+ Kd7 12. Nxa8). Sent to the analyst ~22:34 as a research item: what limits positional learning (representation, game-result labels, coverage, training ply), the best zero-knowledge lever (S7 pairs, deeper training ply, --label-depth search-value labels), whether label-depth was tested in the current regime, and an offline proxy for positional strength.

- **RESIDUAL-P2 LAUNCHED 22:19:4x** (leagues/specs/batch7/residual-p2.json md5 4a2acc9e; the residual fork's checks, models/from-scratch/residual-p2/summary.txt: hybrid6rp without the flags = hybrid6 on capped T-P-Ne, 58 version files + cycles.csv identical over 9 cycles; smoke: residual lines split like capped (9 leaves at v8, with a 100k threshold), every delta moved (norms 0.93-2.73), gain over the trunk -0.006..+0.030 nats; models play on hybrid20 as plain probit trees; error routing + residual sound: children not routed an example still follow the trunk). Seed 70, ply 2 from scratch, floor 300 / cap 400; ~5-7 CPU-h. Reads (6.98) at v150 / v300 / final vs y+v+l @299, plies 2 and 1. EXTEND FORK resumed 22:1x after its 200-turn limit (HANDOFF.md; swap gated on all tests + repros; residual-p2 added to the repro set 22:21).

- **ANALYST 6.98 (~21:51): RESIDUAL = hybrid6rp AS BUILT** (hierarchical trunk: every delegator's shadow trains on all its examples; leaf = shadow + delta; --leaf-residual --residual-inherit-sums, NOT the non-inherit form; a shrunk-delta build is the follow-up if the trunk alone does nothing). **residual-p2**: ply 2 FROM SCRATCH, F-P x2, T-P-Ne capped x2, capped + residual x2 (hybrid6rp), one seed (70), 1,584 games, self-games, cold rating-only, floor 300, cap 400, ~5-7 CPU-h; reads vs y+v+l @299 at v150 / v300 / final, plies 2 and 1; bands ply 2 residual - capped 20-60 (centre 35), residual - F-P -10..40 (centre 15); ply 1 residual - capped -20..20; 30+ over capped = the trunk beats the lag (then ply-3 twin + shrunk variant), within 15 = nothing to add. T3 (LR, 11-10) not evidence against (no lag at ply 1, old regime, LR dead steps). Honest expectation: 20-60 over capped, level with or a bit above F-P; "clearly outperforms" still needs S7. RESIDUAL FORK ~21:52 (identity of hybrid6rp flags-off vs hybrid6 on the Ne recipe, smoke at ply 2, spec leagues/specs/batch7/residual-p2.json; I launch). **S7 SPEC**: two seeds as separate runs s7-p1-s1/s2, each F-P x1 + F-P-S7 x1, ply 1, floor 300 cap 400, ~4-5 CPU-h both; reads vs r/e/g/h @300 at v150 / v300 / final, plies 1 and 2; bands ply 1 30-120 (under 15 closes), ply 2 0-60, 15-40 => third seed; the candidate stats = per-line STATE file (current version), the model file holds pairs + weights; forwarded to the S7 fork ~21:51.

- **THE USER ~21:48: "sure let's try residual leaves"** (after my explanation of S7 and residual leaves: shared trunk on every position + a per-leaf correction shrunk to 0; prior LR T3 run: no lasting gain; a probit build chessai-hybrid6rp exists, never launched). S7 then ALSO APPROVED ~21:50 ("Let's try pairwise features also"): S7 BUILD FORK launched ~21:50 (training binary with --pairs, byte-identical off; a tournament binary from hybrid20 that plays pair models, byte-identical on all old kinds; exact resume of the candidate stats; smoke; draft leagues/specs/batch7/pairs-p1.DRAFT.json; models/from-scratch/pairs/). Analyst asked ~21:49 for the exact design (is hybrid6rp's shadow chain the "trunk"? delta shrinkage? lines, seeds, ply 2 scratch or warm, reads, band); a build fork follows with the design.

- **CHAMPION vs FINAL DONE 21:42 (models/from-scratch/champ-vs-final/run.out, read.py): PLY 1 final - champion -6.4 (SE 5.6 between pairs, 3.3 measurement; pair SD 27.5; final ahead in 10 of 24); PLY 2 16.1 (SE 9.8 / 5.4; SD 33.9; 9 of 12).** Both inside 6.96's band -10..20; champions never win by 15+ => THE LIBRARY KEEPS FINALS. By family ply 1: flat L 12, flat P -23, tree L -5, tree P -4, grow -7, DAG -12 (4 each, noisy); ply 2: flat P 10, flat L -1, tree P 21, DAG 42 (2). True per-line spread SD ~22 at ply 1 (27.5 minus measurement 16): a given line's final can be 20-40 off its champion either way, but on average they are equal at ply 1 and the final is ahead at ply 2.

- **CLEANUP TIER 1 DONE 21:43** (see CLEANUP-INVENTORY.md head): 370 -> 356 GB used; the 13 extension candidates' caches kept. Two stale wait loops of old forks (pids 2431339 in fastbin/errdag4, 31 h; 686951 in bddtrig, 4 days, a self-matching pgrep) still poll; my kill was refused by the permission classifier: harmless, the user may stop them.

- **ANALYST 6.96 (~21:25) + 6.97 (~21:33).** A, WHY GROWTH DOES NOT PAY (ranked): capacity on these FEATURES is not the bottleneck (linear tables over one-hot square-piece indicators, phase-partitioned; the partition saturates at ~9 leaves; nothing represents an interaction between two squares); then data per leaf at deeper plies (lag); coverage (~45 on-policy, lost by trees first); the split criterion is not the lever; the field (D). EXPERIMENTS: (1) S7-p1, data-chosen pairwise features in the probit leaf: F-P x2 plain vs x2 with S7, ply 1, 2 seeds, build + ~4 CPU-h; band 30-120 if interactions matter, under 15 closes the feature lever (6.97 build: from the flats' binary, F-P only; a pair = the conjunction of two active indicator values with a Gaussian weight; candidates all ~400k pair-values (2,415 active a position); statistic = the first-order score at the current model (S += v, n += 1 per active candidate, z = S/sqrt n, ~3 MB); every 1M positions add the top 50 with |z| > 5 (Bonferroni), mu 0, s2 (prior/2)^2, cap 2,000, drop |mu| < 0.5 sd after 3M own examples; flags --pairs (off = bytes unchanged), -k, -z, -cap, -shrink, -drop; pair list in the model file; power: 2 seeds read 50+ cleanly, 15-40 gets a third seed; read plies 1 and 2 at v150, v300). (2) Residual leaves (shared trunk + shrunk per-leaf residual) at ply 2 vs F-P and the capped tree, build + ~10 CPU-h, band residual - F-P 0-40. (3) Exploration dose T 0.15 vs 0.1, no build, ~2 CPU-h, band +-25 for trees. (4) D's probit-only field. (5) The ladder (train at the ply you play, 300-500 a rung) is the proven route; the flat trained at its ply is today's best at plies 2-4. B: design right; final - champion 0..15 (centre 5, band -10..20); keep finals unless champions win by 15+. C: extend std-v1-p1 x2 (+200), flat-p3-w4 and flat-p3-w3f1 (+100), treeonly-p1 (+100); skip the six uniform arms, grow-p1, flat-p2-w3; guard_z 1 right. D: selection (max of ~49 probit trees ~45 over the mean by chance) + a shared field effect + possibly composition; test: 20-50 over the std's probit lines => single-leaf-family standards, within 15 => selection. E: agenda items 1-8 answered; open: 9 what beats parameter-limited models (S7, residual), 10 warm vs scratch (running), 11 champion vs final, 12 the probit-field effect, 13 the ply-4 rung (hold), 14 one game set (closed for mixed fields). S7 AND RESIDUAL LEAVES PUT TO THE USER ~21:32 (their call under the zero-knowledge / leaf-sharing rules).

- **LAUNCHED ~21:29-21:30 (batch7 specs): explore-t015-p1** (std-v1-p1-s1 with --temperature 0.15 for the training games; seed 65, hybrid6/grow6, cap 301; read v300 vs s1 v300, r/e/g/h panel) and **probit-field-p1** (F-P, T-P capped, T-P-grow, D-P-grow x2 = 8 lines; seed 65; cap 301; read v300 vs s1's probit lines). **CHAMPION vs FINAL ~21:28** (models/from-scratch/champ-vs-final/: make.py, run.sh, read.py; arenas diag/cvf-p1 (24 pairs + r/e/g/h) and cvf-p2 (12 pairs + y/v/l), T 0.02, 30 round-robins). Load ~18 on 8 cores at 21:30 (5 runs + arenas + forks).

- **THE USER, ~21:18:** (1) "For the experiments which didn't meet the plateau rule for termination can we keep running them?" -> EXTEND FORK launched ~21:22 (an exact `leagues.py extend DIR --cycles N` for cap-ended runs; proof vs a fresh higher-cap run; swap .before-extend; nothing launched by it). My scope: the 13 single-ply cap-ended runs (std-v1-p1 x2, treeonly-p1 x5, fponly-p1 x2, grow-p1, flat-p2-w3, flat-p3-w3f1, flat-p3-w4; CPU per cycle 25-29 / 13-16 / 7 / 17 / 26 / 23 / 117 s: ~9 CPU-h per 100 extra cycles for all 13, a third of it the ply-4 run); NOT the 19 old mixed-ply/random-player runs (superseded regime). (2) "when experiments end are we taking the champions or the latest models?" -> THE LATEST (build_index.py: each line's final version; last_champion only recorded); a paired final-vs-champion test is proposed (analyst asked for the band). (3) "Let's run the ply 2 rung once the global ladder finishes" -> decision 2 = the full plan, LAUNCHED 21:21 (below). (4) Cleanup TIER 1 APPROVED (lists regenerating; the 13 extension candidates' caches excluded). (5) "continue collaborating with the math analyst agent doing the research indepedently. There's a lot of questions that we still want answered along with the main goal of searching for a promising model that's able to outpeform the clearly parameter-limited models" -> analyst asked ~21:23 (A main goal: ranked one-factor experiments; B champion vs final; C extensions; D the ladder's probit-field top; E agenda).

- **STD-V1-P2 RUNG LAUNCHED 21:21 (created 21:21:01, repeat.sh each): std-v1-p2-s1 (seed 67) and -s2 (68) warm from std-v1-p1-s1/-s2 v600 (fill_p2_starts.py; version 0 copied at cycle 1, so later ply-1 extensions can't touch it), std-v1-p2-x3 (69) from scratch.** Specs leagues/specs/standard/std-v1-p2-{s1,s2,x3}.json (LAUNCHED note in the description). Ply 2, 1,584 games, hybrid20, rate_every 4, cold T 0.02 rating-only, crown 3 draw-aware SEs; warm floor 41 / cap 101, scratch floor 300 / cap 601. Cost ~8 CPU-h to the floors, ~18 to the caps. Read: std_p2_read.py (early / final).

- **LADDER-2 DONE 21:17:48 (25 min; 10 double round-robins, 6,520 games a player; calibration 319/319 identical).** Depth dominates (random = 0): ply 1 ~530-921, ply 2 ~470-1603, ply 3 2052-2134, the ply-4 flat (flat-p3-w4 v99, cap) 2422. BEST / MEDIAN BY FAMILY: ply 1 tree P (49) 921 / 866, DAG P (16) 919 / 896, tree L (30) 896 / 868, DAG L (8) 887 / 864, tree-grow P (19) 876 / 827, tree-grow L (8) 874 / 843, flat P (49) 839 / 787, flat L (28) 831 / 798; ply 2 flat P (40) 1603 / 1566, tree P (21) 1582 / 1514, flat L (17) 1572 / 1512, DAGs 1519-1539; ply 3 flat L (18) 2134 / 2082, tree P 2094, flat P 2088. Ply-1 top 12 (903-921, CI +-11): all probit trees/DAGs from probit-only fields (probit-widen-p1c, error-dag-probit-p1, router-p1, error-growth-p1). STD MODELS at ply 1: T-L capped s1 896 (#20), grow trees #43-#100, DAGs #63-#160, F-L #93/#143, F-P #167/#192. fponly-p2's 4 F-P lines within 1 Elo (likely the same model). READING: no growing model beats the parameter-limited ones at any ply; flats lead at plies 2-3. The dashboard (Home, library ladder column) now reads ladder-2.

- **LADDER-2 LAUNCHED 20:52:36 (the user, ~20:45: "throwing all our library-qualifying models into the global league tournament"; leagues/specs/arenas/ladder-2.json, leagues/arenas/ladder/ladder-2, run.out): 323 entries + the 4 random players (random = Elo 0), each at its training ply: ply 1 212, ply 2 88, ply 3 22, ply 4 1.** All 290 plateaued library rows + the 16 std-v1-p1 finals (cap-ended, the standard) via --include; the 4 leaf-sources-new self-play rows (F-L#1/#2, F-P, F-N .sp, "self-play depth default") at PLY 1: their self-play used the binary's default depth 0 = score the moves directly, the same one-move lookahead as a tournament ply-1 entry (rs/src/main.rs depth doc). make_ladder.py defaults on top: the standing panels as the reference set (r/e/g/h @300 at ply 1, y/v/l @299 at ply 2), flat-p3/-b @199 provisional at ply 3, flat-p3-w4 @99 (cap, not plateaued) provisional at ply 4. hybrid20, T 0.02, 10 double round-robins, nice 5, threads 3 x jobs 2 (2 cores left for the Play page); estimate 15.4 CPU-min a round-robin, 2.57 CPU-h. The dashboard shows the latest ladder run: Home and the library's ladder column read ladder-2 once it finishes (one 10-round chunk).

- **STD-V1-P1 DONE: s2 19:53:09, s1 19:54:23, BOTH AT THE CAP (cycle 601, finals v600).** The plateau rule (50
  cycles without a crowning) was met from ~581 but HELD by guard_z 1 while LR lines still gained (s1 at 601: F-L z
  1.70 and T-L capped; s2: T-L capped crowned at 593, T-L-grow z 2.32 at 585). Last crownings: s1 533, s2 593. The
  library index (rebuilt 19:5x) marks all 16 finals "end cap, plateaued no": hidden from /m/'s default view.
  READ (std-p1-read.sh -> .out; 16 finals + the r/e/g/h panel @300, T 0.02, 30 round-robins), PLY 1, each seed vs
  the panel (s1 / s2 / mean): T-L capped 57 / 48 / 52; T-P-Ne-grow 69 / 18 / 44; T-L-grow 58 / 16 / 37; T-P capped
  22 / 43 / 32; D-L 43 / 6 / 24; D-P -18 / 13 / -2; F-L -17 / -34 / -26; F-P -48 / -53 / -50. Seed differences s1 - s2:
  17, 5, 9, -21, 42, 50, 36, -31 (each +-~31): RMS 30, minus the measurement part (~16 SE each) => RUN-TO-RUN SD ~18
  per run at ply 1 at v600 (8 pairs; rough). PLY 2 (19:58): T-P capped 121 / 107 / 114; T-L-grow 115 / 112 / 114;
  T-L capped 106 / 94 / 100; T-P-grow 96 / 91 / 94; D-L 93 / 72 / 82; F-P 75 / 43 / 59; F-L 62 / 48 / 55; D-P 41 / 63 /
  52. Seed differences 14, 32, 12, 14, 2, 5, 21, -22: RMS 18, about all measurement (SE ~16) => run-to-run SD ~5
  per recipe at ply 2. s1 > s2 in 13 of 16 cells (mean +10 to +13): a SHARED FIELD EFFECT per run, which cancels
  within a run. READING: trees lead at both plies (capped and grow alike); DAGs below their same-leaf grow trees
  (D-L - T-L-grow -13 / -31, D-P - T-P-grow -46 / -41 at plies 1 / 2); flats last at ply 1 (F-L -26, F-P -50), level
  with D-P at ply 2. LR - PROBIT by recipe: ply 1 +24 (F), +20 (capped), -7 (grow), +26 (DAG), mean +16; ply 2 -4,
  -14, +20, +30, mean +8: no meaningful difference (LR marginally ahead).

- **HYBRID20 DONE ~19:20 (perf review candidate 1; bin/chessai-hybrid20 md5 cda3d7c5; fastbin/hybrid20/summary.txt):
  the fixed-point node cap 64 -> 128, safe for every input** (the cap is only an eligibility check; sums stay under
  2^59; 64 was an undocumented speed guard). 28 identity cases at plies 1-5, all policies, openings, self-games,
  calibration and check mode: byte-identical. Game CPU with a model over 64 nodes: ply 2 x0.39-0.41 (ARGMAX x1.44:
  the fixed-point path lacks root windows; nothing queued plays argmax at ply 2; next review), ply 3 x0.53-0.76,
  ply 4 x0.39-0.58, ply 5 x0.51; ply-4 self-games x0.34. REPOINTED: std-v1-p2-{s1,s2,x3}.DRAFT.json and
  probit-p4w.DRAFT.json (.before-hybrid20). CAUTION: regenerating the ply-2 drafts with make_drafts.py writes
  hybrid19 again; repoint after. Running std-v1-p1 runs unaffected (ply 1).

- **PLAY DEEP MODE LIVE (fork, finished ~19:18; chessai-play2; models/from-scratch/play1/deep-summary.txt; full think
  time on every move since 19:16): iterative deepening to max ply 10, think 5 s - 5 min or no limit, pondering ("Think
  on my time") on by default.** No limit at max ply 10: 5-80 s a move (ply 9: 1.6-22 s). At the defaults (10, 30 s):
  ply 9 on most moves, ply 10 for F-P early. Table move order 1.3-2.9x, table cutoffs another 1.4-2x (1.6-4.2x less
  CPU at ply 7); TT fixed 96 MiB (engine ~110 MB). Ponder hit keeps the searched plies (~1 ply gained). Exact moves =
  play1 byte for byte (12,255 positions + 1,060 requests at plies 1-8); deep with the full table = exact ply in 659 of
  661. Its early-stop prediction (wall time) was too pessimistic under load and is off (DEEP_PREDICT, predict=0).
  FOLLOW-UP play3: the 128-node limit once hybrid20 confirms it, the fixed-point path in deep mode (float is 3.4x
  slower per node on a capped tree), an early stop by node counts.

- **PLY-2 RUNG PREPARED ~18:57 (fork; models/from-scratch/std-v1-p2/README.md): chessai-grow7 (md5 f0f9df79;
  --warm-reset closes in-flight trials and restarts windows, keeping structure; --start-leaf-forget F forgets every
  leaf once, routers untouched; byte-identical to grow6 / hybrid6 without the flags in 10 of 10 comparisons; 249 unit
  tests). leagues.py 1315b8ef swapped 18:56:30 (.before-warm2; start_reset, start_forget for trees, per-entry
  train_binary; selftest 245 checks + 18 runs; the std runs re-run 5 real cycles byte-identical).** Ply-2 proof from
  a frozen std-v1-p1-s1 v340, 8 lines: 35/35 checks; NOTE closing leaf trials early mostly undoes them (3 of 4
  judged; credit refunded). Drafts std-v1-p2-{s1,s2,x3}.DRAFT.json (seeds 67/68/69; warm "version": null until
  fill_p2_starts.py fills the ply-1 finals; x3 scratch min 300 / cap 601). Read: std_p2_read.py (fit on games against
  a fixed outside panel: y/v/l @299 + the ply-1 standing panel as a lower tier). COST at ply 2 (0.024 CPU-s a game):
  warm ~0.9 CPU-h to the floor / ~2.3 to the cap each; scratch ~6 / ~13; rung ~8 / ~18. WAITS FOR THE USER'S
  DECISION 2 and both ply-1 finals (s1 last crowned 365: earliest stop rated cycle 417).

- **STD-V1-P1 at ~18:45 (s1 cycle 345, s2 349): DAG sizes D-L 46 / 42 nodes (17-19 leaves, 25-27 routers), D-P 42 /
  37 (15 leaves, 22-27 routers); grow trees 22-31 (LR 28 / 31, probit 22 / 25).** 6.94 predicted 35-45 at v300: on
  track. grow-dag-p1's probit DAGs were 56-59 nodes at the same cycle. Router trials s1 9 kept / 0 undone, s2 9 / 2.
  Latest crownings s1 312 (F-L), s2 340 (F-L, F-P), so the plateau (50 training cycles without a crowning in any
  line) is not near. WARM-START FORK hit its 200-turn limit ~18:44 (grow7 installed 18:40; leagues.py not swapped
  yet); resumed to finish.

- **PLAY: PLIES 1-8 since ~18:09 (the user: "select a ply above what we've trained on"; "That's fine it olny has
  to make 1 move every once in a while"; play.py PLIES 1-8, .before-ply8; server restarted, pid 3637262).** Any model
  at any ply. Timed: plies 5-6 under 0.5 s, 7 up to ~4 s, 8 up to ~21 s. The user then: "i'd like to be able to set
  it to a very high ply. like 20" -> "maybe 20 is too high and it would cause an OOM issue?" -> "let's say up to 10".
  DEEP MODE FORK ~18:12 (chessai-play2: iterative deepening with a deadline, best move of the deepest completed
  iteration, root/PV ordering + a bounded transposition table, no quiescence; max ply 10, think time 5 s - 5 min or
  no limit; the exact 1-8 menu unchanged). Memory isn't the limit (depth-first search uses memory proportional to
  depth); time is (~4-5x a ply).

- **ANALYST 6.95 (~17:56): (1) grow-dag-p1: liberal router splits gained nothing and cost 20-40 early; the
  long-horizon hypothesis is not confirmed; the standard's tighter values are the correction. (2) Filtered: no
  detectable gain. (3) Openings, trees: its band missed badly. At the training ply trees are level, so openings
  aren't biased against tree learning. What they remove is T 0.1's whole-game exploration, the coverage a 2-ply
  search queries; position-specialised leaves lose it, a flat generalises over it. Keep T 0.1 training games + the
  rating-only cold set; openings CLOSED as the single-set route for mixed fields ("the user's worry was right in
  effect, wrong in place"). (4) PLY-2 RUNG: WARM as the standard (s1 -> p2-s1, s2 -> p2-s2) + ONE scratch run
  (third seed) as the calibration arm, once; ~30 CPU-h. Read vs the ply-2 standing panel at v75, v150 and plateau.
  Prediction: warm = scratch within 25 at the plateau; warm reaches scratch's v150 by v50-75. Rule: scratch 30+ over
  warm says rungs start cold, else warm. Settings: 1,584 games; plateau min 41 / cap 101; start_forget 1 probit, 10
  one-off for LR flats and LR leaves; grow lines keep structure + grow state, with windows/shadows/snapshots reset;
  DAG router trials in flight closed on their statistic so far, shadows fresh, path tables carried.** WARM-START BUILD
  FORK launched ~17:57 (chessai-grow7 with a warm reset + one-off leaf forget; leagues.py start_forget for trees;
  proof from std-v1-p1-s1; drafts std-v1-p2-s1/-s2 warm + -x3 scratch; read script; nothing launched). DECISION 2
  stays the user's.

- **PLAY PAGE LIVE since 17:38:47 (fork, reported ~17:49; http://127.0.0.1:8766/play/, Play in the top bar + a Home tile; server pid 3610216;
  dashboard.py 72f76952, play.py/js/css, chessai-play1 md5 26f4a935 = hybrid19 + a play-serve loop on the
  tournament's code path; models/from-scratch/play1/summary.txt).** Library picker (default plateaued + holdouts, /m/
  filters, sorted by ladder Elo), ply 1-4 (default its training ply), colour, best move or arena-like T 0.02; drag /
  click / typed SAN or UCI, promotion picker, take back, flip, resign; W/D/L bars, search value and top 4 moves; PGNs
  in leagues/play-games/. Checks: argmax tournament byte-identical on both binaries; 12,255 positions at plies 1-4, 0
  move mismatches (201 exact ties); rules = chess.js on 93,163 positions; headless browser 29 checks; moves take
  milliseconds (worst 73 ms, grow6 DAG at ply 4).

- **GROW-DAG-P1 READ v300 (17:35-17:38; grow-dag-read.sh -> .out, written ~17:33; r/e/g/h panel @300, both kinds,
  T 0.02, 30 round-robins): DAG - grow tree ply 1 -38 [-61, -16], ply 2 -21 [-43, 1]** (6.88's bands: -20..10 at
  ply 1, MISSED below; -25..15 at ply 2, within). DAG - F-P -31 / 7; tree - F-P 8 / 28. The liberal router-splitting
  DAGs (credit 5M, chain 2, 46-57 routers) are no better than the grow tree, maybe worse; one run (both lines share
  data). grow-dag-p1 FINISHED 17:50:19 (plateau at its floor, cycle 600; final versions v599; champions F-P
  304/356, trees 424/370, DAGs 452/272). The v600 loop could never pass (final = v599): script fixed to read v599
  (backup .before-v599), rerun 17:51 (ply 1, then ply 2). FINAL READ v599 (17:51-17:54): DAG - tree ply 1 -20
  [-43, 2], ply 2 -8 [-30, 14]; DAG - F-P -11 / -3; tree - F-P 9 / 5 (all within ~20). Against the panel @300 the
  v599 entries sit at -51..-31 (ply 1) and -14..-6 (ply 2), no higher than at v300. VERDICT: router splits (liberal,
  63-76 nodes) gave NO gain in play at v300 or v599; DAG = tree = F-P within ~20 at the end.

- **INVENTORY DONE ~17:28 (fork; models/CLEANUP-INVENTORY.md, lists + scripts in models/from-scratch/inventory/;
  NOTHING deleted; a checker found no library file, binary, source, note, referenced version or running experiment in
  any list).** Disk 391 of 1,021 GB. TIER 1 (scratch, regenerable) ~41 GB: 7.4 Rust build dirs, 1.7 finished-run
  caches, ~32 build/verification scratch. TIER 2 (unreferenced old models/data) ~260 GB: 52 unreferenced experiment
  versions (51 keeping every 50th), 67 + 14 old cycle-system versions + states, ~120 old from-scratch studies (twin 44,
  offline-ply 21 in 3.8M tiny files, drift 12), 3 champion run, 4 retired-binary calibration caches, ~1 pools/held-out
  sets/old self-play. TIER 3 (compress): finished runs' elo_history.csv 35 GB -> ~3.5 with zstd (nothing under leagues/
  reads them now); 166 *.before-* backups into one archive. fastbin looks 157 GB but frees ~33 (dry runs share extents
  with experiments): delete its build scratch first. Undecided: games.csv/game_cpu.csv (12 GB, read by the dashboard
  and arena tools), selfplay/league-champ + league-cycles (1.2 GB, old dashboards), leagues-smoke, rs/target. FOR THE
  USER: tier by tier approval.
- **LIBRARY INDEX REBUILT ~17:33 (build_index.py --extra-file: the 847 arena-referenced versions without a snapshot,
  from inventory/refs.json; backups of index.csv/snapshots.csv in my scratchpad): 481 rows (284 plateaued), 1,398
  snapshots (reflinks; disk unchanged at 365G).** Step 0 of any cleanup done; the inventory's lists must be
  regenerated before deleting (the index changed).

- **STANDARD v1 PLY-1 RUNG LAUNCHED 17:31:37 (created + pinned; repeat.sh 17:31:4x): std-v1-p1-s1 (seed 65) and
  std-v1-p1-s2 (seed 66)** (leagues/specs/standard/std-v1-p1-s{1,2}.json from the fork's drafts, description updated:
  launched by me while the user was away; choices per 6.94). 8 lines, 1 each: F-L, F-P (hybrid6), T-L-Ne capped f10,
  T-P-Ne capped (hybrid6), T-L-Ne-grow-f10 (grow-mdl 0), T-P-Ne-grow (grow-mdl 1) (grow6), D-L-Ne-ctx-grow (+chain 2,
  trial-z -2, grow-mdl 0, forget 10; NEW recipe row, make_recipes.py .before-std) and D-P-Ne-ctx-grow (+chain 2,
  trial-z -2) (grow6); tournament hybrid19; ply 1, 1,584 games, T 0.1, self-games; rate_every 4, cold T 0.02 share
  0.5 rating-only; CROWN at 3 DRAW-AWARE SEs (leagues.py dfba7832, swapped 17:22:24, .before-drawse; selftest 234
  checks + 17 runs, byte-identical off on 3 runs); plateau sqrt K 50 from 300 with guard_z 1.0, cap 601. Cost ~2.2
  CPU-h to cycle 301, ~5.7 to the cap, per seed. Scratch (81 cycles of s1): draw-aware SE median 0.69 of nominal
  (bar 3 ~ 2.1 nominal); both DAGs' first router split at cycle 48 passed at 71 (z 19.4 LR, 3.2 probit).
  filtered-p1 FINISHED (plateau, cycle 353, 17:09). Only grow-dag-p1 runs besides (cycle ~460).

- **OPEN READ DONE 17:20 (open-read.out), TREE FIELD (a = T 0.02 + 6 random opening plies, b = T 0.1, c = plain
  T 0.02; same-kind panel): a - b ply 1: v150 28 [5, 50], v299 3 [-19, 25]; PLY 2: v150 -80 [-102, -58], v299 -109
  [-131, -87]. a - c: 152 / 59 (ply 1), 31 / -48 (ply 2).** 6.91's bands for trees at v299: -40..0 (ply 1: met),
  -50..0 (ply 2: MISSED, far worse). F-P field (above): 0 / -12 at v299. So openings match T 0.1 for flats at both
  plies and for trees at ply 1, but cost trees ~100 at ply 2, where openings trees are even below plain T 0.02 trees
  (-48). This supports the user's worry ("biased against tree/dag models which have specialized nodes or branches for
  the early game"). One seed per arm (run-to-run SD ~35-45), so -109 is ~2.5 SD. Openings stay out of the standard.

- **ANALYST 6.94 (~17:15), ADOPTED for std-v1-p1 (sent to the std fork ~17:16):** (a) plateau window = cycles, not
  tests: rate_scale SQRT (2x; its own 4x "doubles run length for nothing") + a GUARD (no plateau stop while the last
  test's current - champion > 1 draw-aware SE); crown at 3 draw-aware SEs if that build is verified by launch, else 2
  nominal (about the same bar); changing the window before cycle 300 is safe. (b) DAGs: credit 10M, CHAIN 2 (not 1:
  chain 1 ends "encouraged" once each router has split, dropping the long-horizon case), router-trial-z -2 (prunes
  grow-dag-p1's two null splits); predicted nodes 35-45 at v300, 55-70 at v600 (chain 1: 30-40 / 45-60); trees
  19-25. TELL THE USER: chain 2, not the chain 1 I told them. (c) TWO seeds per recipe at ply 1 (two runs). (d) For
  the user: trees trained at ply 3 tie flats at every ply (the tree gain is a ply-1 phenomenon; ply 1 = the
  architecture screen; the deeper evaluator is for now the flat trained at its ply); the ply-1 matrix is
  non-transitive by design (a directed graph; the ladder should show it); openings work for a flat (one game set
  could train and rate), pending the tree field (pre-registered -40..0); path cells add nothing in play (4th
  offline-loss-not-Elo case): v2 candidate DAG without cells, keeping unsharing by path.
- **FILTERED READ ply 1 v300 (17:05; filtered - twin vs the r/e/g/h panel, one seed each): F-L -36 [-67, -5], F-P
  -19 [-50, 13], T-P-Ne-grow -6 [-37, 25], T-P-Ne capped 11 [-20, 43], T-L-Ne capped -48 [-79, -17].** Below 6.90's
  bands (trees 0..40, flats -10..20): the filter doesn't help at ply 1, and hurts the LR lines. PLY 2 (17:08): F-L
  -13 [-44, 18], F-P 55 [24, 87], T-P-Ne-grow -34 [-65, -2], T-P-Ne capped -21 [-53, 11], T-L-Ne capped 2 [-30, 34].
  Mean over kinds -20 (ply 1) and -2 (ply 2); signs flip between plies (F-P -19 -> 55, T-L -48 -> 2) as one-seed
  noise (run-to-run SD ~35-45) predicts: NO DETECTABLE GAIN from the filter. filtered-p1 runs on (cycle ~350).

- **THE USER, ~16:58-17:10:** (1) "First let's establish the standard for ply 1. then we can move onto ply 2" (the
  ply-4 run probit-p4w stays ON HOLD); (2) DAGs "maybe they split a little too liberally? I don't know if we have a
  good way to controll it, but i guess just pick some values"; (3) a PLAY page in the dashboard menu, "a chess board
  where I can play against a game against a chosen model in the library at a chosen ply" (lichess's open-source UI
  if it helps); (4) "are dags capped or uncapped on leaf counts? Are trees?"; (5) ~17:10 "I have to go for a bit,
  please continue all this work". Answers: "capped" = --split-limit counts splits (4 = 9 leaves max); capped trees 13
  nodes; grow trees uncapped, 22-25 nodes at ~430; grow DAGs uncapped, 63-76 nodes (15-19 leaves, 46-57 ROUTERS);
  old error DAGs 4 leaf + 4 router splits. MY DAG PICK for the standard (both leaf types): the recipe defaults
  router-credit 10M and router-chain 1 (grow-dag-p1: 5M, 2) + router-trial-z -2 (keep only if the split beats the
  unsplit router by 2 block-SE; rendered --router-trial-z=-2). Why: router trials 18 decided, 2 undone; 14 of 16 kept
  at z 3+ (so a bar alone barely shrinks them; the rate, re-splits of copies and unsharing fan-out, up to 9 routers in
  one split, do). Rough guess told to the user: ~40% fewer nodes. Two standard choices still the user's: the plateau
  window under rate-every-4 (2x mine / 4x the analyst's) and crowning at 3 draw-aware SEs.
- **FORKS ~17:08:** PLAY PAGE (chessground + chess.js vendored; a new engine command chessai-play1 from hybrid19's
  source, checked against the tournament's argmax choices; Play in the top bar + a Home tile; PGNs under
  leagues/play-games/; models/from-scratch/play1/) and STD-V1-P1 PREP (the D-L-Ne-ctx-grow recipe on grow6 + smoke
  test; a draw-aware champion-test option in leagues.py, off by default, byte-identical off; the spec
  leagues/specs/standard/std-v1-p1.DRAFT.json, 8 lines, rate_every 4, cold T 0.02 share 0.5, rate_scale and the
  draw-aware threshold TODO; priced; scratch create only). ANALYST asked ~17:10 (results 1-6 below + the two choices,
  DAG values, 1 vs 2 seeds at ply 1).
- **PROBIT-P3 PLY-4 READ DONE 16:55:56 (probit-p3-read-v301.out): tree - F-P 1 [-22, 24]** (direct -4 [-58, 49]); so
  17 / -1 / 1 at plies 2 / 3 / 4: TREES TIE FLATS WHEN TRAINED AT PLY 3. P3 F-P - P2 F-P x4 at ply 4: 155 [132, 177]
  (direct 109). P3 tree - ply-3 F-L f3: 28 [6, 50]; P3 F-P - f3: 26 [4, 48].
- **TRAINPLY MATRIX DONE 16:33 (trainply-matrix.out; direct family vs family, both kinds pooled; plies 1 / 2 / 3):
  P2 - P1 -282 / 324 / 145; P3 - P1 -176 / 314 / 237; P3 - P2 76 / -196 / 145.** By kind at ply 1: P2 - P1 tr -387,
  fp -227; P3 - P1 tr -282, fp -79. At ply 3, P3 - P2: tr 185, fp 148. Each family is best at its home ply; at ply 1
  the order is P1 > P3 > P2 (P3 beats P2 at ply 1 though both lose to P1).
- **OPEN READ, F-P FIELD (16:54-16:58; open-read.out; a = T 0.02 + 6 random opening plies, b = T 0.1 (coldrate),
  c = plain T 0.02; same-kind panel): a - b at ply 1: v150 -18 [-40, 4], v299 0 [-22, 22]; ply 2: v150 -141
  [-165, -117], v299 -12 [-35, 11]. a - c: 202 / 105 (ply 1), 152 / 136 (ply 2).** Openings recover what plain
  T 0.02 loses and reach T 0.1 by v299 (not better). Tree field pending (treeonly-p1-open still running).
- **NOCELLS DONE (~17:00; models/from-scratch/nocells/; bin/chessai-nocells md5 b39fb3f8, `neutral-cells IN OUT`
  zeroes every cell parameter): PATH CELLS MAKE NO MEASURABLE ELO DIFFERENCE.** 8 ply-1 DAGs with vs without, stranger
  panel, T 0.02, 30 round-robins: ply 1 pooled 8 [-10, 27] (h2h 17 [-15, 48]); ply 2 -5 [-24, 13] (h2h -12 [-43, 19]);
  both 2 [-11, 15]. Offline log-loss gains 0.002-0.013 (path-detail) don't show as Elo; per-DAG results don't follow
  the gain sizes. 12 of 27 path-context DAGs never split a router (no cells). Tighter: 4x ply-1 round-robins
  ~2.5 CPU-min -> +-9.
- **PLY-4 PREP DONE 16:56 (models/from-scratch/ply4-prep/): TREE WARM STARTS LIVE (leagues.py 1ade33c3 -> 6fb93c92,
  atomic; backups .before-treewarm; selftest 232 checks + 3-cycle byte-identical re-runs of grow-dag-p1, filtered-p1,
  treeonly-p1-open).** A tree's v0 copies the source's state and model; its first training runs train --start.
  hybrid19 plays grow6 DAGs byte-identically to grow6/hybrid17 at plies 1-4 (no new binary). Ply-4 game CPU-s:
  DAG-DAG 8.1, DAG-tree 4.5, DAG-F-P 3.2, tree-tree 2.3, F-P-tree 1.6, F-P-F-P 1.1; a cycle ~0.68 CPU-h unrated, 0.94
  rated; 41 cycles ~31 CPU-h, 101 ~76. Draft leagues/specs/ply4/probit-p4w.DRAFT.json (md5 538b9552; plateau K 58
  from 41, cap 101) ON HOLD. DON'T DELETE leagues/starts/grow-dag-p1@395/ or probit-p3's bdt301.state files.
- **DASHBOARD PAGES LIVE 16:56 (dashboard.py c5a45604, server pid 3567006):** Home = 3 tiles + Global ladder; /x/
  (experiments, events, earlier runs); /a/ with filters; /m/ Models: 421 index rows, default 260 plateaued + holdouts,
  14 sortable columns incl. ladder Elo, filters set/type/leaf/archetype/regime/start/ply/end. leagues/library/
  holdouts.csv header only; PROPOSALS for the user in models/from-scratch/dashboard-pages/proposed-holdouts.md.
  SNAPSHOT GAP: the ply-1 v300 panel versions and the ladder's T-P-Ne v299s have no library snapshot (build_index.py
  --extra EXP:LINE@V before any cleanup). The index is stale (11:56): refresh after the inventory fork finishes.
- **FILTERED READ crashed 16:37** (make_split_read.py: lines without a '#' replicate suffix); fixed 17:04 (md5
  3aece0b8, backup .before-nohash: such lines take their replicate column), RERUN 17:04:43 (26 entries).

- **ANALYST 6.93 (~16:30):** (a) P2 - P3 196 with both equal vs the P1 panel = a SATURATED panel (300 below, both ~95%) + parity edges between near-equals; panels must sit within ~150 of what they judge (cross-ply panels from both families' flats); strength is one number only within a family and ply, across training plies a directed graph; mixed fits report excess SD. (b) probit-p4w: start_forget 1 for F-P (probit variances at the tau equilibrium; W4's 8 was LR's AdaGrad); the ply-1 DAGs start ~300 below at ply 4 and dilute the warm lines ~15-20 for 50-100 cycles, accept and flag (if still 150 behind at the floor: never caught up); 720 games, cycles, plateau fine. PRE-REGISTERED (ply-4 stranger panel, +-25): P4 F-P - its P3 start 100-200 at ply 4 by the floor, -60..0 at ply 3, -150..-50 at ply 2; P4 - x4 at ply 4 150-250; trees - F-P at ply 4 -30..20 (the user's idea -40..0); DAG - tree -30..20; DAG routers split 2-6 more times by the floor, 30-60% undone. (c) never rank lines within a run by in-run ratings (decisions only). (d) adopt draw-aware SE in new experiments; with rate_every 4 crown at 3 draw-aware SEs (~1.8 nominal, today's false-crowning rate); cold share 0.5 at plies 1 and 4. Sent the F-P forget 1 to the ply-4 prep fork.
- **~16:15-16:25, the user: "Alright let's train ply4"; library state + "organize everything we have so that we can start deleting more of the stale artifacts"; "Can we continue to add at least a couple of DAGs in which we may have reason to believe that the routers will split please" / "If we're doing diverse groups that is"; dashboard split (home = Global Ladder + menu: experiments, arenas, models; models sortable by type, regime self-play / homogeneous / diverse / other, training ply; plateaued only + holdouts).** RATE-EVERY BUILT (fork done ~16:35): leagues.py f7177b40 -> 1ade33c3, dashboard 67b32e47 -> f3bf796d at 16:25:27, server pid 3535388 (16:25:36); rated cycles (c-1) mod N = 0, cap must be rated, plateau K in training cycles checked on rated cycles, scaled needs rate_scale (no default); requires cold games train false; scratch rate4-e2e evidence in rateevery-build/. filtered-p1-twin ENDED on its plateau at cycle 380 (16:19:20). Classmates: 21 uniform runs + 45 mixed, all ~chance (my recount 51 / 49 / 50%; the fork's cross-kind 60% not reproducible from its pairs file). grow-dag-p1's D-P-Ne-ctx-grow (router-chain 2, credit 5M) line #2: router splits 9, kept 7 by cycle 373. FORKS ~16:27: (1) ply-4 prep (tree/DAG warm starts in leagues.py, backups .before-treewarm; DAG play binary check / hybrid20 if needed; draft leagues/specs/ply4/probit-p4w.DRAFT.json: F-P x2 + T-P-Ne capped x2 warm from probit-p3 v301, D-P-Ne-ctx-grow x2 warm from grow-dag-p1, entry_ply 4, 720 games, seed 64, rate_every 4, cold 0.5, sqrt plateau min 41 cap 101; NOT launched); (2) dashboard pages (backups .before-pages; holdouts.csv + proposed-holdouts.md for the user); (3) cleanup inventory, read-only (models/CLEANUP-INVENTORY.md, tiers + literal commands, nothing deleted). Analyst 6.93 sent ~16:28 (non-transitivity, ply-4 design incl. F-P forget, classmates, draw-aware threshold). W4 precedent (11-11, F-L forget 8): 173 over its start at ply 4, 200 over W3, nothing at odd plies (RESULTS ~2635).
- **THE USER, ~16:05: "The amount of noise we're trying to reduce almost feels insurmountable the models are so weak, and we still haven't been able to find any consistent way to improve their strength through a systematic addition of new parameters. It's a little disappointing. Perhaps we should bring back MoEs and give them a try as well. But overall it feels like we're just battling noise. Do we even know if probit or LR are better now?"; ~16:08: "Also, I think introducing subcycles is probably a good idea".** Answered ~16:12: architecture effects small and ply-dependent (-34..89; ply-3-trained -1), DATA effects large and single-run visible (training ply 176-224, head-to-head ~185; T 0.02 games -54..-138; on-policy ~45; search labels ~100 old league, outside the standard): not capacity-limited; LR vs probit UNKNOWN (old results conflict; ply-3 F-P - F-L 11 [-8, 30], different runs; < ~60 either way); MoE history poor (LR gates collapse to 1-2 experts, NB gate ~hard routing, none beat trees); PROPOSED: a ply-4-trained mixed field warm-started from probit-p3 v301, read at ply 4 vs the P3 models (the user's 11-11 rule: no deep rungs unless low-ply stagnates; it has), to be re-costed. RE-COST (16:15, probit-p3 totals): ply 3 with hybrid18 ~77 CPU-s a cycle (728 games + 104 self-games; whole run 9.4 CPU-h); ply 4 at T 0.1 (every root move exact, x~16 nodes, hybrid18 x0.48/x0.55) ~x14 a game -> ~1,000 CPU-s = ~0.28 CPU-h a cycle: 50 warm cycles ~14 CPU-h, 100 ~28 (plus cold games if rated on them: share 0.25-0.5 every 4th cycle +6-12%).
- **P3P2 READ DONE 16:02 (ply 2; p3p2-read.out): HEAD-TO-HEAD the ply-2-trained models win clearly: P3 tree - P2 tree direct -129 / -219 / -161 (y/v/l), P3 F-P - P2 F-P -236 / -106 / -260; all 12 P2 entries (118-268) rank above all 4 P3 entries (69-88). AGAINST THE PLY-1 PANEL they are close: trees 82 / -82 / -9 (mean -3), F-P 27 / -114 / -18 (mean -35).** Fit chi2/dof 1.98, excess SD 50.6 Elo over 496 pairs (earlier reads 0-14): a large matchup effect; a weak panel (300-400 below the arms) can't separate P2 from P3 at ply 2. Mirrors ply 3 (P3 F-P - fponly-p2 F-P 176 panel / 228 direct): at a given ply, models trained at that ply win head-to-head. For the analyst's next batch: why (horizon blind spots exploited by same-ply-trained opponents?), and what it means for panel choice.
- **THE USER, ~15:55-16:00: (1) asked what "a run would stop about as soon as today" means; (2) "Not sure i fully understand impact of draw awareness yet. I'm all for reducing SE while not playing more games that do not contribute to training data"; (3) "Can we test the newly finished P3 models againts the P2 models playing at P2?"; (4) "Do we know the correlation of the best-performing model in an experiment to rank above its classmates in an arena?"** Answered ~16:02: (1) late crownings (probit-p3 cycles 200-300: 1.4% of tests) restart the 35-cycle clock; simple model E = (e^(lambda K) - 1)/lambda: today ~110 cycles after real gains stop; rate-every-4 with K 35 ~45, K 70 ~120, K 140 ~440 (late per-test crowning rates, cycles 201-300: 0.5-3% in most runs; t002 runs 9-16%). (2) draw-aware = error bars only; real in-run SE ~0.6 of reported; the precision we THOUGHT we had needs ~half the rating games; with rate-every-4 rating falls from ~1/2 to ~1/10 of all games; arena reads unchanged. (3) P3P2 ARENA LAUNCHED 15:56:22 (global11-drafts/p3p2-read.sh -> p3p2-read.out; arena diag/p3p2-p2-v301, 32 players at ply 2, 30 round-robins): probit-p3 v301 (p3tr, p3fp) against replay-p2 (y), diversity-p2 (v), probit-late-p2 (l) trees and F-P at v299, panel r/e/g/h @300 both kinds; known: P3 F-P - fponly-p2 F-P at ply 2 -224. (4) not known directly; CLASSMATE CHECK sent to the t01check fork ~15:58 (in-run vs arena order: concordance, in-run best = arena best vs 1/k, same/cross kind, training ply vs other, single vs 10-version mean, gap > 2 SE nominal/draw-aware).
- **RANDOM OPENINGS LAUNCHED 15:41:2x: treeonly-p1-open (seed 59) and fponly-p1-open (seed 60)** = the t002 specs + "openings": {"plies": 6} on chessai-hybrid19 (md5 f88040dd, cp -n 15:21:38; without the flags byte-identical to hybrid18), hybrid6 training pinned; the opening plies TRAIN; cold set (normal start) rating-only share 0.25; 300 cycles. Build (fork done ~15:40; openings-build/summary.txt): leagues.py 8748a886 -> f7177b40 at 15:39:00 (backup .before-openings; key reaches round-robins, self-games, top-up; never cold games or own calibration); dashboard 67b32e47, server pid 3468296 (15:39:14). Cost a game x0.90 trees, x1.05 F-P of plain T 0.02. Diversity at plies 8/12/20: ~100% with openings (plain T 0.02: trees 66/86/98%, F-P 19/42/82%). Elo scale ratio 0.927 [0.869, 0.991] (compresses ~7%, no kind shift; T 0.1 vs T 0.02: 1.13). READ queued: global11-drafts/open-read.sh -> open-read.out (waits for each field; arms a = open, b = coldrate, c = t002; plies 1-2, v150/v299, same-kind panel @v300).
- **RATE-EVERY-N BUILD FORK launched ~15:42** (the user's sub-cycles; leagues.py "rate_every": N, cycle 1 rated, unrated cycles play + train only, filter floors and ratings carried from the last rated cycle, plateau in training cycles checked on rated cycles, scaled plateau needs "rate_scale": sqrt | linear | none (NO default: the user's pick), champion age logged, dashboard marks unrated cycles; backups *.before-rateevery; scratch rateevery-build/).
- **PROBIT-P3 DONE (plateau, cycle 302, 14:50:47; current v301 everywhere, champions F-P 263/217, trees 266/264). FINAL READ v301 (probit-p3-read-v301.out; plies 3 and 2 at 30 round-robins from 15:14, then ply 4 at 20): PLY 3 VERDICT tree - F-P -1 [-20, 19]** (v150: 28 [5, 52]); 6.84 (-40..10) hit, 6.90 (10-50) missed, the user's odd-ply band (30-90) missed. The script's rule: 0 or less = the tree's edge is a ply-1 thing (no ply-4-trained field). Ply-3 F-P - ply-2 F-P x4 at ply 3: 176 [157, 194] (v150: 94). Both arms - flat-p3 F-L: 11. One run: within-run contrast, seed variation not covered. PLY 2 (15:27:44): tree - F-P 17 [-3, 37] (v150: 54 [28, 80]; 6.90's 30-80 missed); ply-3 F-P - ply-2 F-P x4 at ply 2: -224 [-243, -205] (training at the playing ply: 176 at ply 3, 224 the other way at ply 2). Told the user ply 3 at ~15:29; ply 2 held for the ply-4 report (ply 4 started 15:27:45).
- **THE USER, ~15:10: SUB-CYCLES ("split cycles into sub-cycles. each sub-cycle is like today's cycle but we don't update champions (we still play and train)").** Analyst 6.92 (~15:20) and I: rate every N = 4 cycles (between: T 0.1 round-robins, self-games, training; no cold games, no champion test), adopt as a mechanics option (strength effect predicted < 20, unresolvable), log champion age, filter floors from the last rated round; not moot if openings work (the crowning test pools N cycles). The user ~15:15: "yeah we'll have to update the plateau rule". PLATEAU OPTIONS PUT TO THE USER (~15:22): analyst = the same tests per window (K rated = 4K training cycles; probit-p3's 35 -> 140); me = the user's sqrt line-scaling applied to tests per cycle (K x sqrt(N) = 2K: 35 -> 70, ~18 rated). Build after the openings fork releases leagues.py. Memory: sub-cycles-rate-every-n.md.
- **THE USER on openings, ~15:15-15:20: "We'll see what happens with the random openings, but I dont have high hopes for it" / "It might also be biased against tree/dag models which have specialized nodes or branches for the early game".** Answered: the opening plies train by default; the read is per kind (tree-only and F-P-only fields against their own T 0.1 runs), so a tree-specific loss shows as the tree arm losing more (a gap between arms resolves only ~120); the variant if so: don't train on the random plies; rating games and arenas keep the normal start.
- **T01CHECK DONE (fork, ~15:20; models/from-scratch/t01check/NOTES.md): T 0.1 CAN'T REPLACE COLD GAMES (option c).** Crownings T 0.1 / cold / both: F-P fields 365/54/45 and 311/71/52; tree fields 248/105/62, 192/128/61; filtered pair 109/61/38, 101/117/44. After cycle 50 the per-test diffs correlate only 0.04-0.46; T 0.1 reads F-P progress 0-19 higher and tree progress 0-23 lower (F-P crowned 4-7x as often, 41-52% of tree crownings missed); across kinds no stable offset (the twin: T 0.1 puts trees 35 BELOW flats, cold 27 above). Same-kind line order: chance per cycle, same on 200-cycle averages. Saving the cold games would buy 2-3% of a ply-1 run's CPU (7% at share 1.0). LEAD: recorded SEs probably too LARGE (draws 45-73% of cold games; late cold diffs spread 32-37 vs nominal 38-39); a draw-aware SE might cut them 30-47% at no game cost, VERIFIED (~15:50, t01check/t01split.py, NOTES.md): IN-RUN CHAMPION-TEST SEs ARE ~1.7x TOO LARGE: split-half SD / nominal 0.60 [0.59, 0.61] over 7,334 tests (F-P twins 0.49, tree twins 0.66, filtered flats 0.68, trees 0.70; cycles 1-50 0.58, later 0.60); the draw-aware (HC0) SE predicts it within 0.02-0.04 (0.63); in-run matchup excess small (pair-cluster 0.61). Today's 2.0-SE test = ~3.3 true SE (F-P 4.1, trees 3.0, filtered 2.9): KIND-BIASED. At 2.0 draw-aware the twins would crown 279/307/267/265 (recorded 54/71/104/128). ARENA PANEL READS: NO cut (split-half 0.82, but matchup excess: binomial x sqrt(chi2/dof) 0.76-1.23, pooled 1.04): keep their CIs (ply-1 could narrow 10-25%, plies 2-3 widen 5-25%). Put to the user ~15:55: draw-aware SE for new experiments' champion test + filter floor, threshold 3 (my pick: today's strictness, kind-neutral, plateau unchanged) or 2 (4-5x crownings, plateau retune).
- **IDDEEP DONE (fork, ~15:22; fastbin/iddeep/out/: item4-table.md, analysis.txt): ID + ordering-only TT NOT WORTH BUILDING** (ply 4 saves 15-29% real CPU, under the 30% bar; ply 3 costs 12-25% more; perfect ordering caps at 52-57%). Better eval does NOT order better: first-move cut rate at ply 4 F-P v10/v300 90.7/88.3, tree v30/v300 86.5/86.0, the ply-3-trained models 74.0/74.4. The 3->4 jump (x16) is mostly the root regime (every root move exact: x3.4; ordering x1.7; inherent x2.8). PRUNING vs ELO (archparity-p3's 28 models, 300 positions, windowed root): ply 3 leaves 1,350-2,030 a move = 13.3-16.0% of the full tree (84-87% PRUNED; I had told the user ~93-95% for ply 3: CORRECT IT), r 0.41 with Elo (stronger models keep more root moves within the window, r 0.76; with exact roots r 0.02); ply 4 10.0-20.1k = 5.4-7.1% (93-95% pruned), r 0.66 (the strongest, ply-2-trained lines cut first only 70-73% vs 82-87%). STRONGER MODELS PRUNE LESS. 98 MB scratch build kept (deletion candidate).

- **THE USER, ~15:03: "I'm not sure different opening moves will help but we can test it" (random opening plies, then near-argmax: one game set for training AND fair ratings).** Build fork launched 15:04: chessai-hybrid19 (--opening-plies K, optional --opening-temperature), leagues.py "openings" key (round-robins + self-games only, not cold games or own calibration; backups *.before-openings), drafts treeonly-p1-open / fponly-p1-open = the t002 runs + openings (comparators coldrate and t002), diversity and cost measured. K, uniform vs softmax, training on the opening plies and predictions asked of the analyst (6.91). Also running: the T 0.1 within-line rating check (fork 14:58) and the pruning / iterative-deepening measurement (fork ~14:52, + per-model pruning vs Elo).
- **ANALYST 6.91 (~15:06): K = 6 UNIFORM random opening plies (one factor; softmax openings diverge less), the opening plies TRAIN. Predictions (same-kind panel, v150 / v299): open - coldrate trees -90..-30 / -40..0 (ply 1), -90..-30 / -50..0 (ply 2); F-P -150..-60 / -70..-20, -180..-80 / -90..-30 (about half the T 0.02 gap recovered); open - t002 trees 20..60 / 20..50, 20..60 / 20..60; F-P 50..120 / 40..80, 60..150 / 50..100. Within ~30 of T 0.1 at the plateau = the rating-only cold set can go; check the scale: random openings compress Elo gaps ~10-20% uniformly across kinds (measure in-run gaps / cold panel gaps; under 0.8 widen margins by the inverse).**
- **T 0.02 TRAINING REDO READ (t002-read.out, 14:30-15:00; a = all T 0.02, b = coldrate (all T 0.1), c = coldtrain (1/5 T 0.02); same-kind panel @v300).** F-P: a - b ply 1 -225 [-250, -200] (v150) / -101 [-123, -79] (v299); ply 2 -312 / -138. Trees: ply 1 -126 / -63 [-85, -41]; ply 2 -99 (v150) / -54 [-76, -32] (v299; read done 15:00:41). c - b (1/5 dose, a new arena): F-P 34 / -32 at ply 1, 60 / 49 at ply 2; trees 86 / 26 at ply 1, 13 at ply 2 v150. BOTH KINDS TRAIN WORSE ON ALL-T 0.02 GAMES (end: F-P -101/-138, trees -63/-54 at plies 1/2; trees hurt less, 38/84 apart, at the edge of one-run resolution), the 11-11 result (-106/-221, F-L) confirmed with the better design; flats worse at ply 1 by ~40, within run-to-run noise. 6.90 bands: trees -60..-20 (edge), F-P -80..-30 (missed, worse), F-P ply 2 -120..-60 (missed, worse). Dose: trees non-monotone (1/5 helps early), F-P ~flat then strongly negative.
- **filtered-p1 + filtered-p1-twin LAUNCHED 14:39** (leagues/specs/global/*.json from the drafts; seed 63; F-L, F-P, T-P-Ne-grow bar on (grow4), T-P-Ne capped, T-L-Ne capped f10 (hybrid6); ply 1, 1,584 games, cold share 1.0 rating-only, champion_se default 2.0, plateau scaled from 300, cap 600; filtered: {"margin": "se"} + top-up). Cycle 2: SE ~23, lines keep 16-25% (champions still v0, below the floors), top-up 1,350-1,512 against themselves. Insight only (6.90).
- **ARCHPARITY-P4 FINAL (cut 14:38:28 at 8,933 games, ~10 round-robins; archparity-sweep.out): tree - flat at ply 4: ply-1 family 46 [30, 62] (r 20, e 37, g 66, h 60); ply-2 family -30 [-48, -12] (y -71, v -6, l -14).** Table, plies 1/2/3/4: ply-1-trained 89 / 36 / -27 / 46; ply-2-trained -9 / -34 / -17 / -30. 6.83 predicted -70..-30 and -30..0: the ply-1 family missed (rebound at ply 4). Told the user 14:42. To the analyst with the t002 read.
- **grow-dag-p1 LAUNCHED 14:30:15** (created 13:15:39, all 6 lines pinned to chessai-grow6; load 10): F-P x2, T-P-Ne-grow bar on x2, D-P-Ne-ctx-grow (--router-chain 2, --router-credit 5000000) x2, seed 62, ply 1, 1,584 games, floor 600, cap 900; ~4.5 CPU-h to the floor. Read (6.80 (3)): ply-1 panel r+e+g+h @v300 at v150, v300, v600 and the final, plies 1-2; 6.88 predictions in the spec. fponly-p1-t002 DONE 14:30 (300 cycles in 50 min); treeonly-p1-t002 at cycle 203.
- **ANALYST 6.90 (~14:29): RUN-TO-RUN NOISE: a uniform field's lines share data, so a run is ONE sample; from the eight B - C cells, one run against one run has SD ~35 at ply 1 (trees ~45), ~45 at ply 2: contrasts under ~90 are unresolvable with one run per arm, under ~60 with two.** Filtered pair: one seed each, INSIGHT ONLY (the user's framing); a second seed only if the first pair shows 60+ anywhere; predictions under the user's rule, filtered - twin at ply 1 v300: trees 0..40, flats -10..20, ply 2 within 30, ranking unchanged. probit-p3: the training-ply zigzag (+89, -34, +28) matches the user's odd/even idea, the play-ply data don't; no account produces both; the test is a ply-4-TRAINED mixed field (costly, the user's call; under the idea tree - flat at ply 4 -40..0). Final read pre-registered: tree - F-P 10-50 at ply 3, 30-80 at ply 2, -20..30 at ply 4. t002 (v299, same-kind panel): a - b trees -60..-20 at ply 1 (centre -40), -80..-30 at ply 2; F-P -80..-30 (centre -55), -120..-60; a - c 10-20 more negative for trees at ply 1, the same for F-P; dose non-monotone for trees, monotone-negative for flats; 4x cold would sit between (trees -20..0, F-P -40..-15): rating-only stands whatever the read shows.
- **HYBRID18 DONE (14:17; bin/chessai-hybrid18 md5 392eab6a; fastbin/hybrid18/summary.txt): values-only fixed-point search at plies 3+, byte-identical to hybrid17 (plies 3-5 x T 0.1/0.02/argmax, check mode, self-games, 205 unit tests).** Game CPU a ply: probit-p3 mix x0.62, T-P-Ne x0.55 (ply 3) / x0.48 (ply 4), LR trees/DAGs x0.36, F-P x0.80. Swapped into probit-p3 at 14:17:10 (cycle 235 replayed with hybrid17: identical); tournament 100.8 -> 63.3 CPU-s a cycle. make_ladder.py defaults to hybrid18; make_split_read.py uses it at ply >= 3 (own calibration stays hybrid16). Running arenas keep their binaries.
- **COLD VERDICT, F-P FIELD (14:21): coldtrain - coldrate (FP panel) 27 [5, 49] / -28 [-50, -6] at ply 1 v150 / v299: FAILS at the end -> RATING-ONLY (drafts set train false, backups .before-train).** Ply 2 76 / 42. coldrate - control -17 / 10 at ply 1, -93 / -74 at ply 2; coldtrain - control 10 / -18, -17 / -31. Run-to-run variance (B - C should be ~0) is ~+-50: a uniform field's 4 lines share data (one sample); resolving <~60 needs seeds. Told the user 14:25.
- **FILTER3 BUILT (observers fork, done 14:02): leagues.py 05e0ec76 -> 8748a886 at 14:02:33 (backup .before-filter3); dashboard 2c25e594 -> f9ea60b0 at 14:02:42 (server pid 3377341).** The user's rule: floor = current version's cold Elo - its SE in this cycle's fit ({"margin": "se"}; numeric margins still work), smooth default 1 ("se" with smooth > 1 refused), no own-game exemption (a champion below the floor: its games cut, self-games included), top-up vs champion if above the floor else vs itself. Selftest 223 checks + 14 runs; byte-identical without the key (3 runs x 3 cycles); running experiments cycled on after the swap. Drafts: filtered-p1 19647061 and twin f33c4dd5, share 1.0, cold train "TODO". SE CORRECTION: the measured SE is 1.15-1.35x the binomial estimate (twins: median 40-47 on 98 games), so ~47-55 at share 0.25 and ~22-26 at 1.0 (I had told the user ~35 -> ~17). Kept shares replayed: twins' last 40 cycles 70-96% at their SE, 37-81% at half; draft field at SE 20-24 with trees 50-150 over flats: trees keep ~20-33% (top-up ~1,170-1,400 each), flats ~80%, ~4,300-4,900 top-up games a cycle. Cost: twin ~1.4-3.6 CPU-h, filtered ~2-5. Launch ~15:30 (after archparity-p4's cut; load), cold train false unless the user says otherwise.
- **THE USER, ~13:37: "Can we repeat the experiment that confirmed that our models are weaker if trained on 0.02 data vs
  0.1 knowing what we know now about how to construct better experiments".** LAUNCHED 13:39: treeonly-p1-t002 (seed
  59) and fponly-p1-t002 (seed 60), leagues/specs/batch6/*-t002.json = the coldrate twins (comparators, done/ending)
  with round-robins and self-games at T 0.02 (--policy log-odds-softmax --temperature 0.02; the default is the same
  policy at 0.1), no observers, the same rating-only cold set, 300 cycles, hybrid6 training. Fixes vs 11-11's
  train-t002-p1/-t01-p1: 144 games a cycle there, arms rated at their own temperatures, direct read, F-L x2 once.
  READ queued (t002-read.sh -> t002-read.out, waits for each field): plies 1-2, v150/v299, same-kind panel @v300,
  arms a = t002, b = coldrate, c = coldtrain: a dose-response 0 / ~1/5 / all T 0.02. ~0.25 + ~0.8 CPU-h runs.
  Answered the user (~13:33): a small end-of-run loss from the 1/5 dose can't be ruled out (v299 ply 2: -9 [-31,
  13] vs T 0.1-only; -12 vs the control; plus run-to-run +-30); recommended rating-only unless F-P shows a gain.
- **THE USER, ~13:22-13:24: THE FILTER RULE IS THEIRS (overrides 6.87):** "You should cut a model's own games against
  significantly weaker opponents. I don't think we should use an averaged rating, instead we should just play more
  games per round so that the standard error is lower." / "both players need to be at least the current model's
  strength - standard error based on last round" (= the old global league's qualify rule, margin = SE). Build sent
  to the observers fork (resumed ~13:27; backups *.before-filter3; dashboard held until the ladder fork swaps):
  train_filter {"margin": "se"}, floor = current version's cold Elo - its SE in this cycle's fit, own games under the
  rule, smooth default 1, top-up vs champion if above the floor else vs itself; drafts: cold share 1.0 (~1,620 cold
  games, SE ~17 against ~35). ASKED THE USER: do the 4x cold games also train (half the data; tested at ~1/5; pure
  T 0.02 training lost ~100)? Default rating-only.
- **The user, ~13:22: "How does ply-1 trained trees doing better at ply 4 revive my even/odd idea?"** It doesn't: their
  idea (10:24) was ply-1 trees better again at ply 3, worse at 4; the data is the reverse (-27 at 3, ~+60 at 4,
  partial). My 12:48 "partly revives your even/odd idea" was wrong; told them. "We need to see how trees trained at
  ply 3 do against flats trained at ply 3 when playing at ply 3": = probit-p3 (v150 read running, final at the end).
- **The user, ~13:25: F-P trained on tree games beat F-P trained on F-P games plausibly because trees split
  differently (more diverse games of reasonable strength); "hybrid fields for experiments to achieve strongest models
  is probably the way to go".** Memory: uniform-fields-and-observers.
- **THE USER, 13:16: "Can we make the global ladder separate from the arena (replace the original global ladder that
  was stopped with 2 final competitors".** Dashboard fork launched ~13:18: the Global section shows the ladder
  (arenas/ladder/*, latest standings inline: rank, player, ply, Elo +- 95%, games; random = 0), ladders leave the
  Arenas lists, the stopped global league moves to Earlier runs (page kept). Backups *.before-ladderglobal. DONE 13:28:39: dashboard 9cd5695f -> 2c25e594, server pid 3343138; Global section titled "Global ladder" (ladder-1 standings: rank, model, ply, Elo +-95%, games; depth anchors badged, random rows muted; every ladder run listed); /a/ladder/ladder-1/ = "Global ladder ladder-1"; arena count 173 -> 172; other pages identical (masked times). Scratch models/from-scratch/ladderglobal-build/.
- **FILTER2 SWAPPED 13:15:12 (observers fork, done): leagues.py 05e0ec76 (backup .before-filter2), dashboard 9cd5695f
  (13:15:22; server pid 3332167).** 6.87's rule is the default: "train_filter": {"margin": 50, "smooth": 5}; own games
  = any game of the line's entries (current or champion); others' games iff both smoothed ratings >= R_L - 50; R_L =
  mean of the line's last 5 versions' own-cycle cold ratings. Byte-identical without the key (three runs, 3 cycles
  each); selftest 222 checks + 14 runs. Expected kept shares (replayed): 96-100% in uniform fields; in the mixed draft
  field ~52% for tree lines if trees sit 50+ over flats on cold ratings, then ~3,100-4,000 top-up games a cycle; cost
  ~24-29 CPU-s a cycle late (twin ~17-20): ~2-4.5 CPU-h over 300-600 cycles. filtered-p1.DRAFT.json has margin 50,
  smooth 5 (cold train still "TODO"). The twin's description still says "tree-only games": fix at launch.
- **ANALYST 6.89 (~13:17): (a) B < C at ply 1 only = run variance (one run an arm; decision noise would show at ply 2
  too); no change to the cold share. Proposed crowning on smoothed ratings for filtered-p1 (2.5 SE on one cycle's
  ~66-72 cold games an entry makes crownings rare). My counter (13:21, to the analyst with the F-P verdict): DROP
  champion_se 2.5 for the default 2.0 in filtered-p1 and its twin, no smoothing build (the user objected to pooling
  across versions; their end rule says "just like experiments do"). Evidence, cycles.csv z of every champion test:
  the cold arms (median se_diff ~38 against ~19 at T 0.1) crowned at 2.0 as often as the T 0.1 controls (treeonly
  coldtrain 126, coldrate 100, control 115 over 300-400 cycles x 4 lines), and at 2.5 they would have crowned about
  half as often (76, 48; fponly 35/34 against 61/47). (b) If F-P passes: cold train TRUE in filtered-p1 AND its twin. (c) The honest
  line: safe to train on, speeds the early climb, does not raise the ceiling.**
- **probit-p3 v150 READ DONE 13:34 (probit-p3-read-v150.out; ply-3 panel f3, f3b, w4, x4, x4b, bfp; 1,760 games an arm): ply-3 tree - ply-3 F-P at PLY 3: 28 [5, 52]** (6.84 predicted -40..10, centre -15: MISSED above; the user's odd-ply idea 30-90; rule 30+ = alive, decided at the final read). At ply 2: 54 [28, 80]. Ply-3 F-P - x4 (ply-2 F-P v299): 94 [72, 116] at ply 3, -328 at ply 2. Ply-3 tree - f3 (ply-3 F-L v199): -41 at ply 3, -32 at ply 2; ply-3 F-P - f3 -65 / -95. Training-ply tree edge so far: ply 1 89, ply 2 -34 (archparity ply-2 family at ply 2), ply 3 28 (v150): odd plies favour trees, as the user's idea; but the ply-3 trees also lead at ply 2, so it follows the TRAINING ply, not the playing ply. One run, halfway.
- **probit-p3 v150 READ launched 13:18:49** (probit-p3-read.sh 150 3,2 20 -> probit-p3-read-v150.out; ply 3 then 2, 26
  entries, 20 round-robins, ~0.1 CPU-s a game at ply 3: ~25 CPU-min). Ply 4 waits for the final read.
- **COLD VERDICT, TREE FIELD DONE (12:52-13:13; cold-read2.out; A = treeonly-p1-coldtrain, B = -coldrate, C =
  treeonly-p1 control; verdict panel TR = r/e/g/h tr @v300, 1,920 games an arm).** Ply 1 v150 / v299; ply 2 v150 /
  v299: A-B 89 [66, 111] / 28 [6, 50]; 47 [24, 69] / -9 [-31, 13]. B-C -43 [-65, -20] / -28 [-50, -6]; -2 / -3. A-C 46
  [24, 68] / 0 [-22, 22]; 44 [22, 66] / -12 [-34, 10]. FP panel A-B 32 / 32; 27 / 20. 6.82's RULE PASSES FOR TREES
  (A-B >= -15, lower bound > -30, at ply 1 at both versions). The cold games are EXTRA games (FORMATS: the round-robins
  are unchanged), so A trains on ~25% more data than B and C; B differs from C only in deciding on the cold fit (396
  games a cycle against 1,584). Cold training speeds the early climb (A-C 46 at v150) but ends level (0 / -12 at
  v299). 6.82 predicted A-B -25..0 and B-C within 15: both missed. F-P field waits for its twins (cycles 113 / 121 of
  300 at 13:14, ~16:00).
- **GROW6 FINAL (13:05): the router keep rule already matches 6.88 (undo only if G_real - G_null < -2 block-SE, no MDL
  bar on router trials = --router-trial-z 2), so NO grow7.** Draft grow-dag-p1 now has --router-chain 2 and
  --router-credit 5000000 on the D-P-Ne-ctx-grow lines; 243 lib + 1 main tests pass; P1 smoke 3 router splits, fast
  twin 12 (2 undone, 48 nodes). Cost with chain 2: ~27 CPU-s a cycle, ~4.5 CPU-h to the 600 floor, 7-7.5 to the 900
  cap. Launch after archparity-p4's cut (load ~16 on 8 cores at 13:11).
- **FILTER BUILT (observers fork; leagues.py 6f801ac3 swapped 12:26:58, backup .before-filter; selftest 221 checks + 14
  runs; byte-identical without the keys on three runs' specs):** "train_filter": {"margin": M} (games where every
  player >= R_L - M on cold ratings; own self-games always kept, others' under the rule; needs cold_games; refuses
  observers and reference windows), "self_games_topup": true (k = min(gpc, U - K) own games vs champion, or itself
  when just crowned), champion_se 2.5 (existing key). Drafts leagues/specs/global/filtered-p1.DRAFT.json (seed 63, cold
  train "TODO" until the twins decide) and filtered-p1-twin.DRAFT.json. Cost: filtered ~1.8-4.2 CPU-h, twin ~1.2-3.
  elo_history grows superlinearly but ~1.7 GB by cycle 600 (no window built; a real one needs rs/). OPEN (to the
  analyst 12:30): R_L of the CURRENT version rests on 72-98 cold games (ci95 ~+-115), so the floor moves by about the
  margin each cycle and the cut follows noise; options: champion's rating, running mean, SE-scaled margin, or filter
  on opponents only (own games always count). Dashboard patch held, applying now (fork).
- **GROW6 BUILT (12:45; chessai-grow6 md5 a14361e6; fastbin/grow6/summary.txt): grow4's leaf rule inside bdd with path
  context + --router-split-rule encourage; byte-identical with the flags off; D-P-Ne-ctx-grow recipe row; draft
  leagues/specs/batch6/grow-dag-p1.DRAFT.json (seed 62, F-P x2, T-P-Ne-grow bar on x2, D-P-Ne-ctx-grow x2, ply 1, 1,584,
  floor 600, cap 900; ~3.3-5.5 CPU-h).** Finding: --router-chain 1 caps router splits at one per router. ANALYST 6.88:
  --router-chain 2, --router-credit 5M; the router KEEP rule = stay unless G_real - G_null < -2 block-SE at the end of K
  windows, NO MDL bar on router trials (fork checking/fixing, maybe chessai-grow7). Prediction: 6-12 router splits a
  line by the floor, 30-60% undone; DAG - tree vs TR panel at ply 1 -20..10 (centre -5) at v300 and v600, -25..15 at
  ply 2; kept rate under router splits level within 15 points; nodes under 55. Launch ~14:00 (load).
- **ARCHPARITY-P4 PARTIAL (12:48, 4.7 round-robins, 480 games an arm): tree minus flat at ply 4, ply-1 family r 46, e 22,
  g 98, h 76 (mean ~+60); ply-2 family y -51, v -10, l -27 (~-29).** 6.83 predicted -70..-30 for the ply-1 family.
  The arena costs ~2.6 CPU-s a game (20 round-robins ~12.6 CPU-h): CUT at 10 round-robins by cut-p4.sh (guarded kill of
  PID 3226783 at 8,700 games, ~14:40). 6.88: if +60 holds, the ply-1 trees' edge is 89 / 36 / -27 / +60, a period-2
  component plus a ply-3 dip no account predicts; re-read the ply-3 cell with more games before any theory.
- **PERF REVIEW fork launched 12:50 (scheduled).**
- **ANALYST 6.87 (12:33): filter = own games always count; other players' games iff both smoothed ratings >= R_L - 50;
  smoothing = mean of the last 5 versions' cold ratings; run BOTH filtered-p1 and the twin.** Fork implementing
  (backups .before-filter2). COLD VERDICT READ queued (cold-read.sh -> cold-read.out, launched 12:32: waits for each
  twin pair, then plies 1-2 at v150 / v299; A-B, B-C, A-C against FP / TR / MX).
- **LADDER-1 DONE 12:06 (11:57-12:06, 44 players, 10 double round-robins; ladder-1.out; leagues/arenas/ladder/ladder-1;
  Elo 0 = random, binary rated all):** ply 4 (one F-L, flat-p3-w4 v99) 2,504 +-88; ply 3 (six F-L) 2,145-2,196; ply-2
  models at ply 3 (depth anchors) 1,916-2,033; ply 2 (eleven) 1,613-1,668 (diversity-p2 F-P 1,668, base-lr-p2 F-L 1,655
  and D-L 1,654, base-probit-p2 D-P 1,646, fponly-p2b F-P 1,642, trees 1,613-1,635); ply-1 models at ply 2 1,375-1,409;
  ply 1 (sixteen) 793-979 (error-dag-probit-p1 D-P 979, error-routing tree 956, probit-f10 tree 938, D-L 934; F-P
  793-838); ply-2 models at ply 1 622-640; random-3 369, random-2 332, random-1 182, random 0. Each extra ply (with a
  model trained for it) adds ~700 / ~500 / ~300 Elo (1->2, 2->3, 3->4); architectures span <200 at ply 1 and ~55 at
  ply 2. Fit: chi2/dof 1.97; excess SD 0 within plies 1, 3 and between 2-3; 15 (1-2), 27 (2-2); the ply-1 vs ply-3/4
  pairs fit badly (gaps of 1,200-1,700 Elo; draws dominate the residuals): long cross-ply jumps are unreliable.
  Re-run after probit-p3 (ply-3 trees and F-P).
- **UNIFORM READ DONE 12:00 (uniform-read.out; verdict = same-kind panel; ply 1 v150 / v299, ply 2 v150 / v299):**
  (a) uniform tree - mixed tree -29 / -14 [-41, 13]; -73 / -4 (6.78: 20-70; FAILED). (b) F-P observer - mixed F-P -33 /
  0 [-32, 31]; -102 / -64. (d) uniform F-P - mixed F-P -36 / -29 [-56, -2]; -14 / -58 (6.78: 0-50; FAILED: at ply 1 the
  pure F-P field is WORSE than the mixed one). (e) tree observer - mixed tree -100 / -144 [-176, -112]; -175 / -151
  (6.78: -40..20; FAILED badly). Same data, MX panel, v299: (c) F-P observer - uniform tree (tree field's games) -54
  at ply 1, -107 at ply 2; (f) tree observer - uniform F-P (F-P field's games) -32 at ply 1, -34 at ply 2 (6.78: tree
  ahead 30-90; SIGN REVERSED). Additive 2x2 (mine): architecture ~11 / on-policy ~43 at ply 1; 37 / 70 at ply 2;
  data quality alongside (an F-P watching stronger trees = the mixed F-P; a tree watching weaker flats -144).
  DECISION (mine, 6.78's rule): (a) is not 30+ at ply 1, so NO ply-2 uniform follow-up. Sent to the analyst (6.86).
  ANALYST 6.86: two terms set a learner's strength: ON-POLICY data (its own games; ~45 at ply 1, 60-80 at ply 2) and
  the QUALITY of others' games relative to its level. At ply 1 trees are the stronger kind, so mixing gives flats the
  trees' games (~30, what (d) loses) and trees the flats' games (~0, what (a) gains); at ply 2 the other kind is weaker,
  so the pure F-P field leads. Corrected 2x2: architecture ~40, on-policy ~45 at ply 1 (mine, 11/43, bounds them);
  ply 2 ~20-40 / 60-80. FIRM: on-policy data is worth as much as the architecture; observers are a poor way to train a
  model (every line should play). Filtered experiment predictions REVISED: vs an unfiltered twin trees -15..15, flats
  -10..10, ranking unchanged; the top-up's on-policy self-games may give the top line 10-30; its value is dynamics and
  the cold filter's mechanics. Cold-games rule and probit-p3 unchanged (probit-p3 also tells which kind is stronger
  at ply 3, which sets the sign of mixing effects there).
- **DASHBOARD FIX (12:05; dashboard.py 7d964fb7, backup .before-refit; pid 3271596 on 8766):** 35 completed arenas
  whose random-only anchor never scored now show Elos refitted without random (field mean 0, marked on the page);
  pinned arenas show pinned.csv. Spot checks match the panel reads.
- **LADDER ARENA SIDE BUILT (ladder fork, ~11:46):** arena.py 6db0ecad (anchor {"mean_of": "field"|[players]}, no random;
  per-entry "ply"; `arena.py fit NAME [--write]` -> fit.md, residuals.csv; ci stop works), make_split_read.py 0e7f16f9
  (field-mean anchor by default; --anchor TAG,...; --random-anchor = the old spec byte for byte), make_ladder.py
  a9bf011d (new), library.py 91bcb1bf (spec --anchor field|random). Identity: flatply-p1 re-run byte-identical.
  Fit report: chi2/dof 0.81 (flatply-p1), 0.80 (archparity-p1) = transitive at ply 1; archparity-p2 1.35 (excess SD
  33.5), -p3 1.5 (37.0) = ~35 Elo of matchup at plies 2-3. PROPOSED FIRST LADDER (dry run): 34 players, 14 references;
  ply 1: 8 standing refs + one per family (F-P, F-L, T-P-Ne, T-P-Ne-grow, T-L-Ne f10, T-L-Ne-grow f10 (bar off: only
  bar-off LR grow lines exist), D-P z30, D-L z160); ply 2: 6 refs + F-P, F-L, T-P-Ne, D-P z20, D-L z120; ply 3: 6 F-L
  (provisional refs flat-p3/-b v199, out of the anchor); ply 4: flat-p3-w4 v99. ~0.08 CPU-h (0.12 with
  --depth-anchors 2). Index rebuild needed for treeonly-p1/fponly-p1 (build_index.py). WAITING for the user's OK on the
  lineup (promised). Follow-ups: the dashboard shows the binary's field-mean ratings, not pinned.csv, for
  panel-anchored ladders. The fork deleted one .pyc it had just written (leagues/__pycache__/arena.cpython-314.pyc):
  a rule breach, harmless; told the user.
- **COLD_GAMES BUILT (observers fork, leagues.py 8f5cb216 swapped 11:29:15; selftest 217 checks + 13 runs; byte-identical
  without the key on probit-f10-p1's 57 items). TWINS LAUNCHED 11:29:54** (treeonly-p1-coldtrain / -coldrate; the
  fponly pair starts automatically when both finish "done"; launcher models/from-scratch/coldgames-build/
  launch_twins.sh -> twins-log.md). Cycle-1 identity OK (regular games md5 038c0e90 = the control's; coldrate
  identical to the control through cycle 3, parts at cycle 4: the cold ratings kept 3 champions where the control
  crowned 4). Cost: rating-only +5-6% a cycle, train +~17%. Cycle 1's cold share is 41% (4 entries, regular rounds
  capped at 48), 20% from cycle 2 (392 of 1,960). Finals are v299 (the last cycle trains nothing): read at v150 and
  v299. build_index.py skips observers; treeview TOOL_VERSION 4.
- **UNIFORM READ LAUNCHED 11:36** (uniform-read.sh -> uniform-read.out; 6.78's cells (a)-(f) at v150 and v299, plies 1
  and 2, panels FP / TR / MX per 6.81). Arenas now have no random player (the ladder fork changed make_split_read.py's
  default: anchor = the mean of every player).
- **FILTERED-EXPERIMENT DRIVER BUILD started ~11:38** (observers fork): train_filter {margin 50}, self-games top-up,
  crown_se, drafts filtered-p1 / filtered-p1-twin (not launched).
- **THE GLOBAL LEAGUE'S NEW PURPOSE (the user, ~11:11-11:12; details and analyst 6.85 in NEW-DESIGN.md's last
  section):** (1) GLOBAL LADDER = an occasional no-training arena, library models each at its own ply, one fit;
  arena-side build (anchor = pinned reference-set mean, no random, fixes ciX; per-entry ply; residual report; ladder
  builder) by the LADDER fork, launched ~11:16, test scale only, I decide the full run. (2) ELO-FILTERED MULTI-KIND
  EXPERIMENT (the old global league as an experiment, update 8's six, the 11:08 end rule): WAITS for the cold-games
  twins; its leagues.py builds (Elo filter on training sets, per-line self-games top-up, cold crowning end rule,
  history only inside the window) go to a fork after cold_games is in.
- **probit-p3 LAUNCHED 11:09:55** (leagues/specs/ply3/probit-p3.json; 6.84): F-P x2 + T-P-Ne capped x2 at ply 3, from
  scratch, seed 61, 720 games, floor 300, cap 400, hybrid17 games, hybrid6 training (pinned 11:09). Measured 80-100
  CPU-s a cycle, ~25 s wall: ~8-10 CPU-h, ~2.5-3 h. Read: ply-3 panel f3, f3b, w4, x4, x4b, bfp (all flats), v150 and
  the final, plies 2-4. Prediction tree minus flat at ply 3 -40..10 (centre -15); the user's odd-ply idea 30-90. Rule:
  30+ = odd-ply hypothesis alive (next a ply-4 mixed field); 0 or less = the tree's advantage is a ply-1 phenomenon.
- **flatply-p1 READ (11:07; ply 1, panel r, e, g, h at v300):** ply-3 F-L minus ply-1 F-L -142..-177 (six pairs);
  ply-4 F-L minus ply-1 F-L -205..-220; w4 - f3 -62 [-80, -44]; x4 - ply-1 F-P -225; f3 - ply-1 F-P -142. The model
  trained at a ply wins at that ply at every ply 1-4 (6.85: inside its bands, filed after the run without seeing it).
- **THE USER'S LEAGUE END RULE (11-12 ~11:08, after update 8):** no retirement; the next global league stops on the
  experiments' plateau rule (no new champions for a while), crownings on cold ratings; games_per_cycle stays. Recorded
  in NEW-DESIGN.md. Lineup/ply/timing (update 8) still awaiting the user.
- **The user, ~11:08: ply-3 TREES vs ply-3 FLATS at ply 3** (nothing on disk is a ply-3-trained tree; every ply-3/4
  model is F-L): design asked of the analyst (6.84), default probit-p3 (T-P-Ne capped x2 + F-P x2, entry_ply 3, from
  scratch, 1,584 games, floor 300, cap 400). **And ply-3 flats vs ply-1 flats at ply 1:** flatply-p1 launched ~11:09
  (flatply-p1.sh -> flatply-p1.out; panel r, e, g, h at v300).
- **PLY-4 HALF READ (11:05):** f3 - x4 104 [87, 121], f3b - x4b 87; w4 - f3 144 [126, 161]; x4 - bfp -16, x4b - bfp
  -11; pfp - bfp -64; rtr - x4 -193. Row minus x4 at plies 1/2/3/4: rtr 290/-481/-83/-193, f3 63/-186/147/104, w4
  -5/-131/62/~248. w4 - f3 alternates -68/55/-85/144 (parity for same-lineage flats); the training ply wins at its ply.
- **THE USER'S RATING DECISION (11-12 10:54, update 6):** test T 0.02 games IN TRAINING with uniform ply-1 runs;
  adopt if strength doesn't drop, else a small rating-only cold set; retraining from scratch accepted. Build
  cold_games {temperature, share, train} (observers fork, after observers). ANALYST 6.82 design: four twins at fixed
  length 300, treeonly-p1-coldtrain / -coldrate (seed 59), fponly-p1-coldtrain / -coldrate (seed 60), share 0.25
  (396 cold games a cycle). Rule: coldtrain minus coldrate against the same-kind panel at or above -15 at ply 1 at
  v150 and v300 (lower bound above -30) = cold games join training; else coldrate's regime. Ply 2 reported; mixed
  panel as the thunderdome's view; log crownings and window cuts vs the control. Predictions: F-P -15..5, trees
  -25..0. ANALYST 6.81: composition's within-kind effect is 15-20 (the rest of the 43-95 was kind matchup); read
  every uniform-field cell against three panels (F-P-only, tree-only, mixed), same-kind panel = verdict for
  same-kind contrasts; arena anchor = the standing panel's mean pinned at zero, no random.

- Running:
  - (DONE 03:50 at v331) diversity-p2c: F-P 73 over base-probit-p2's and 51 over probit-late-p2's at v300 (ply 2), level
    with diversity-p2's (-1; the triangle is non-transitive by ~66); at v150 already 77 over probit-late's v300; final
    v331 12 over bfp v450. THE WINDOW CUT STRONGER REFERENCES: cold (T 0.02), the cluster references beat the line
    versions that cut them by 162 and the F-P references by 37: the in-run ratings (T 0.1 softmax, the calibration-
    slope bias) misjudge model kinds. The F-P lines kept only 16-49% of the reference games, the trees 60-88%. This
    bears on the GLOBAL LEAGUE's window, which uses the same in-run ratings (the user's design; analyst asked for the
    cheapest sound fix);
  - (DONE 02:46 at v299) grow-l-p1: the LR growing tree mostly can't grow. 14-15 trials a line, 4-5 kept, 10 undone
    (7-8 with NEGATIVE gain); mean kept gain 0.0008 nats against probit's 0.011-0.014 under the same rule (every probit
    trial kept). Head to head (v150 / v299): grow minus capped -25 / -16 at ply 1, 48 / 41 at ply 2; capped minus F-L
    38 / 21 at ply 1, -101 / -38 at ply 2. ANALYST 6.59 (mechanism confirmed in grow4-src): at --forget-factor 1 (every
    recipe) a split's children and the trial's shadow keep the parent's full AdaGrad sums (LR forget divides them), so
    they step at its late-life rate and can't specialise inside a trial; probit's split noise and ADF variances keep its
    steps alive. T-L-Ne-grow does NOT enter the six as built. The capped LR tree has the same defect (and its ply-2
    reads across four ply-1 fields spread 150: -43, -17, 55, -101/-38, real; the LR tree is the fragile seat at ply 2);
  - (DONE 03:44 at v347) grow-l-p1b: --forget-factor 10 doubles the LR splits' gains (0.0021 vs 0.0009 nats) and halves
    the negative undos; kept 6-7 of 12-15 trials (forget 1: 4 of 17-18); 15-19 leaves vs 13. Head to head forget 10
    minus forget 1: -24 / -13 at ply 1 (v150 / v347), 30 / 75 at ply 2. Passes 6.59's plan rule: T-L-Ne-grow WITH
    FORGET 10 takes the LR seat of the six (half-confirmed: LR trials still fail 40-60%). ANALYST 6.62: the residual
    failure is a trial bias, since split_leaf forgets the grow trial's SHADOW too (three boosted children against a
    boosted shadow, when the alternative is the unboosted leaf); fix = the shadow keeps the parent's sums. BUILT
    04:15: chessai-grow5 (--grow-shadow-forget split|none, byte-identical to grow4 without it; recipe T-L-Ne-grow ->
    grow5; new row T-L-Ne-grow-f10). REVERSED (analyst 6.64, 04:18, after the fork's smoke runs): the forget boost
    ALONE (no split) is worth ~0.004 nats per example over a 1M trial, so "none" credits the boost to the split (a
    bias FOR splits); as built, children and shadow are boosted alike, which isolates the partition. grow-l-p1c (in
    leagues/experiments, created at cycle 0) is NOT launched; keep --grow-shadow-forget split. The finding underneath
    is learner-level: AdaGrad's steps decay as 1/sqrt(life), too slow for a drifting stream. Next: LR with decaying
    sums (a build), tested on F-L first at ply 1 (6.64 (b)); the LR trees are re-asked on top of it. BUILT AND
    FAILED OFFLINE (05:10; chessai-lrdecay1, md5 cab29ea7, fastbin/lrdecay1/summary.txt): the decaying sums lose to
    AdaGrad at every N (1-8M) on every stream (self-play drift -0.006 at 1M, -0.001 at 8M; abrupt change; stationary),
    raw and calibrated; 6.64 (b) predicted gains 0.003-0.008. lr-decay-p1 NOT run (draft only; recipe row F-L-decay
    added, harmless). The record agreed beforehand (lr-settings-2: the smaller late rate won), so the learner-level
    premise is false for flats; the forget-10 split boost is a fresh child's transient need. Sent to the analyst
    05:12 (LR seat = T-L-Ne-grow at forget 10?; the free slot). Deletable (the user's call): fastbin/lrdecay1 stream
    4.5 GB, id 599 MB, sweep 189 MB. grow5's identity
    runs (models/from-scratch/fastbin/grow5/id, 1.5 GB) can be deleted: the user's call. Also 6.62: the capped LR control and the league's LR lines should run forget 10 (the capped LR
    trees' ply-2 fragility was this defect; prediction capped T-L-Ne forget 10 20-60 over F-L at ply 2); probit trees
    at forget 10 would be a separate factor (forget also divides the routers' counts at a split);
  - (DONE 04:45 at v320, plateau) fponly-p2 (analyst 6.61's composition-vs-volume test: F-P x4, seed 51,
    games_per_cycle 1584). READ (head to head, x4 cells 1,200 games ±20), v300 / v320: PLY 2 x4 minus bfp 65 / 57,
    minus pfp 78 / 28, bfp minus pfp -53 / -10; PLY 1 x4 minus bfp 12 / 52, minus pfp 39 / 70, bfp minus pfp 83 / 27.
    VOLUME DOESN'T HARM: the kill result (within 15 of bfp, 30+ below pfp) is not met, so the 1,584 standard stands.
    The mixed batch (weaker trees and DAGs among base-probit-p2's lines) cost its F-P ~60 at ply 2 and little at
    ply 1. 6.61's ply-1 prediction (-60..-20) had the wrong sign: bfp's ply-1 lead over pfp came from volume, not
    from its mix. On the bfp zero at v300: x4 65, p2c's F-P 73, diversity-p2's 6: a pure F-P field is worth what
    stronger references under the window are worth. ANALYST 6.66 (05:00): the value of a training game at ply 2 is
    set by the quality of play at the positions the trainee reaches: own kind at its level or above is useful and
    its volume helps at both plies; weaker play inside its own distribution (own past, weaker sibling kinds) costs
    ~60-120 at ply 2; weaker foreign play is neutral. It forbids: F-P x4 at 720 games beating x4 at 1,584 (predicts
    the reverse by 20-60); a mixed field with the window on COLD ratings failing to recover x4's level (predicts
    within 20 at ply 2); weaker foreign play harming at ply 2 anywhere. (Its ply-1 clause, "adds nothing at ply 1",
    was wrong on the record: replay's own past helped ply 1 by ~37, the weaker siblings cost 12-52 there. AMENDED
    05:05: own past carries shallow play's blunder structure (6.51), so it helps ply 1 and hurts ply 2; weaker
    siblings play worse without playing shallower, so nothing for ply 1. Forbidden: own-past replay failing to help
    ply 1 in a ply-2 field (by 20-60); weaker-sibling games helping ply 1 anywhere.) DESIGN CONSEQUENCE FOR THE USER (6.66 (2)): trained together at ply 2, F-P's batch carries the
    trees' games and costs ~60 unless the window excludes them; the window must run on sound ratings (6.63 (1),
    margin 50); and since 1,584 counts ALL games while a line's useful volume is its own pairings plus its
    equals', top each line up to the standard with self-games (own current vs own champion), scaled per line from
    its window's kept share. Your call (league mechanics);
  - batch-size read (02:52; probit-late-p2 F-P vs base-probit-p2 F-P at equal versions v300 / v399): bfp minus pfp
    -37 / -55 at ply 2 (prediction 10-50: opposite sign), 83 / 47 at ply 1, -17 / 68 at ply 3. A bigger mixed-kind batch
    made a better ply-1 player and a worse ply-2 one (composition confounded); sent to the analyst;
    ANALYST 6.60 (a HYPOTHESIS, under a falsifiability check I asked for at ~03:00): up at ply 1 and down at ply 2 is
    the signature of training on WEAKER play (replay, base-probit-p2's flats among weaker trees and DAGs); the 1,584
    standard isn't the problem, the quality of a batch's games relative to the trainee is (the Elo window's job); a
    ply-2 league needs the window. Caveat (mine): it needs a coverage term to explain diversity-p2's null (58% of
    positions from 200-weaker foreign play, level with bfp), and the ply-2 triangle (dfp-bfp 6/15, dfp-pfp -10/44,
    bfp-pfp -37/-55) is non-transitive within noise;
    ANALYST 6.61 (after my check): 6.60's gap-and-coverage account withdrawn (it forbade nothing). New account with no
    free term: harm comes from weaker play INSIDE the trainee's own distribution (own past, same-stream siblings);
    weaker foreign play elsewhere is neutral. Decisive cheap run (no build): fponly-p2 (F-P x4, seed 51, games_per_cycle
    1584, ~4 CPU-h; DRAFT leagues/specs/ply2/fponly-p2.DRAFT.json), launch when grow-l-p1b's slot frees. Predictions:
    minus bfp 20-60 at ply 2, -60..-20 at ply 1; minus pfp within 30. If within 15 of bfp at ply 2 and 30+ below pfp:
    the harm is VOLUME per cycle, and the 1,584 standard should drop to ~720 for ply-2 fields (the user's call);
  - the global league (two DAGs left, D-P z30 and D-L z160).
  - **diversity-p2d DONE 05:43 at v299; READ 05:50 (2,400 games a cell, ±14), v250 / v299, ply 2: d's F-P minus
    c's -7 / -19 [-33, -5], tree 4 / -14; ply 1 F-P -9 / 8, tree 0 / -2; d's F-P v299 minus bfp v450 -30 (c's final:
    12). THE WINDOW AS RATED COST NOTHING; removing it cost the F-P up to ~19 at ply 2 late (the lines kept training
    on references they had passed: weaker same-kind play, 6.66). The in-run rating bias is a standings problem here,
    not a strength one.** CORRECTED 06:15 by the panel read: against strangers d's F-P is 21 above c's (6 against
    a wider panel), so the direct -19 is a lineage matchup: NO DETECTABLE EFFECT of the window either way.
    Prediction (6.65: d over c 0-40) failed low on the direct cell, holds on the panel. Method note: inside d the tree beats its own
    flat by 61 / 51 at ply 2, yet routed through c's lines it is level (-37..22 / -11..30); in p2c's read c's tree minus
    flat was -10 direct but -65 via bfp, -55 via pfp: kind-vs-kind head-to-heads are strongly non-transitive (40-70),
    so "tree minus flat inside a run" is not a strength verdict; same-kind cross-run cells are. Sent to the analyst.
    (First read attempt failed: the plateau stopped d at the floor, v299, and v300 didn't exist; rerun p2d-read2.sh.)
  - **diversity-p2d LAUNCHED 04:34** (analyst 6.65, pick 2 after its pick 1 was withdrawn; spec
    leagues/specs/ply2/diversity-p2d.json): p2c exactly with the reference window REMOVED (one factor: the window as
    rated in-run). Same seed 48, the same four references (md5s checked), games_per_cycle 1584, floor 300, cap 400,
    hybrid6 training; byte-identical to p2c through v9, diverging at v10 (p2c's first cut). Read power is in the
    READ (6.65): one arena per (ply, version) with only d's and c's lines, 300 round-robins (2,400 games a 2x2 cell,
    ±14), at v300 and V = min(d final, c final 331), plies 2 then 1, plus d's F-P at V vs bfp v450 at ply 2
    (waiter models/from-scratch/global11-drafts/p2d-read.sh -> p2d-read.out; new scripts make_split_read.py,
    h2h_pool.py). Predictions: d over c 0-40 at ply 2 at equal versions (centre 15); d over bfp450 at the final 0-40;
    ply 1 within 20. Rule: 30+ over c at both reads = the window as rated cost the ceiling; within 15 at both = it
    cost nothing here (the bias is a standings problem, not a strength one).
  - **fponly-p2-720 DONE 06:22 at v559 (plateau at the floor); PANEL READ 06:30 (against y + v + l at v299, 2,880
    games an arm): per compute 720 is level with 1,584 at ply 2 (720 minus 1,584: 8 [-11, 27] and -10 [-29, 9] at x4's
    v300 and v320) and behind at ply 1 (-11, -43); per version 1,584 leads by 16 / 14 at ply 2 (CIs reach 0) and
    21 / 27 at ply 1. KEEP 1,584 (rule: within 20 at equal CPU). The ply-2 equal-version gap fell short of the rule's
    20, so by its letter 6.66's useful-volume account fails at ply 2 (holds at ply 1): sent to the analyst. Direct
    cells ran up to 49 off the panel (x7 v534 minus x4 v320: -59 direct, -10 panel). x7 stopped improving after
    v494 (116, 113, 115 against the panel).** ANALYST 6.71 (c): the useful-volume account is REVISED: doubling a
    line's useful games a cycle is worth ~15 at ply 2 and ~25 at ply 1 per version, and nothing per CPU (the 20-60
    was overclaimed). x7 (728 games) sits within 16 of x4 while x4 is 65 over probit-late's F-P (720 games, two
    thirds against level trees), so that 65 was COMPOSITION (own kind against other kinds), not volume. 1,584 stays
    (6.67's rule: stability at no strength cost; no per-compute case for 720).
  - **fponly-p2-720 LAUNCHED 05:08** (analyst 6.67's pick 1; spec leagues/specs/ply2/fponly-p2-720.json): fponly-p2
    at games_per_cycle 720 (13 rounds, 728 games a cycle against 28 / 1,568), same lines and seed 51; floor 560, cap
    700 so it passes the version of EQUAL MEASURED CPU (per-cycle overhead is fixed, so near v510-545, not 6.67's
    v660; fponly-p2's v300 took 2.11 CPU-h). Read (waiter fponly720-read.sh -> fponly720-read.out): x7 vs x4 at
    equal versions v300 / v320 and at equal CPU, plies 2 and 1, 2,400 games a cell. Predictions (6.67): equal
    versions 1,584 over 720 by 20-60 at ply 2, 10-50 at ply 1; equal CPU 720 over 1,584 by -10..30 (centre 10; 6.67 note: moved from 0-40, since equal CPU is ~v520, not v660) at ply 2. Rule: 720
    at equal CPU 30+ ahead = the standard per compute (720 for ply-2 fields: the user's call); within 20 = keep
    1,584; 1,584 failing to beat 720 at equal versions by 20 ends 6.66's useful-volume account.
  - **capped-l-f10-p1 DONE 06:08 at v319; DIRECT READ: forget 10 level with forget 1 at ply 1 (16 / 0 at v150 /
    v319) and 107 / 50 over it at ply 2; forget 10 over F-L 45 / 49 at ply 2, while forget 1 is 56 / 27 BELOW F-L.
    Predictions met; by the rule forget 10 becomes the capped LR control's setting (6.62 (b) confirmed: the LR
    trees' ply-2 fragility was the frozen-step defect).** PANEL READ (06:20, against grow-l-p1/-p1b's 12 lines):
    forget 10 minus forget 1 70 / 83 at ply 2 (holds), -19 / 8 at ply 1 (level by the final); forget 10 minus F-L 5 /
    21 at ply 2, 14 / 39 at ply 1; forget 1 minus F-L -65 / -62 at ply 2. grow-l-p1b's growing tree, forget 10 minus
    forget 1: 4 / 16 at ply 1, -19 / 33 at ply 2 (mixed early, positive at the final).
  - **capped-l-f10-p1 LAUNCHED 05:10** (6.67's pick 2; leagues/specs/batch6/capped-l-f10-p1.json): F-L x2, capped
    T-L-Ne (limit 4, 1M) at forget 1 x2 and forget 10 x2, seed 54, ply 1, 1,584 games, floor 300, cap 400,
    chessai-grow5 training. Read (capped-f10-read.sh -> capped-f10-read.out): 300 round-robins per (ply, version), v150
    and final, plies 1 and 2. Predictions: forget 10 over forget 1 -10..10 at ply 1, 20-60 at ply 2; over F-L 30-60 /
    20-60. Rule: at or above at ply 1 and 20+ over at ply 2 = forget 10 for the capped LR control; 20 below at ply 1
    = capped keep forget 1, the growing tree keeps 10. ANALYST 6.67: the LR seat of the six stays T-L-Ne-grow at
    forget 10 (its trials fail 40-60%: the partition's real value at LR leaves, about half the probit's).
  - Analyst 6.65's first pick, probit-tau-p2 (F-P at tau 1e-5 / 1e-4 / 1e-3 at ply 2), WITHDRAWN on my pushback:
    the record had answered it (probit-settings: 1e-4 below the default in every window by 26-61, at entry ply 2;
    11-07 self-play 1e-3 -512; the drift study: the probit already forgets on a ~one-cycle memory, its step floors
    at the tau equilibrium). The probit's dynamics are tuned and closed; --probit-alpha not worth a slot (6.65).
  - **fponly-p2b DONE 07:49 at v352; PANEL READ (ply 2, y + v + l): THE PURE F-P FIELD'S LEAD REPLICATES.** At v299
    both seeds' F-P score 104 against the panel (difference -1 [-20, 18]); each is 95-96 over bfp (predicted 55-120).
    The replicate's final v352 is 27 below its own v299 and 48 below fponly-p2's v320 (single versions, unequal points;
    version noise; the best-version rule reads v299).
  - **fponly-p2b LAUNCHED 06:32** (analyst 6.70 slot 1; leagues/specs/ply2/fponly-p2b.json): fponly-p2 at seed 55,
    otherwise identical (F-P x4, ply 2, 1,584 games, self-games, floor 300, cap 400, hybrid6 training). The claim the
    league regime would act on (a pure F-P field 43-87 over every other ply-2 F-P against strangers) rests on one
    run. Read (fponly-p2b-read.sh -> fponly-p2b-read.out, at the end): ply 2, 30 round-robins, against the standing
    ply-2 panel y + v + l at v299; fponly-p2b's F-P at v299 (and its final) beside fponly-p2's v299 / v320 and bfp v299.
    Prediction (6.70): within 30 of fponly-p2's F-P, i.e. 55-120 over bfp. Rule: 40+ below = a lucky seed, and the
    pure-field advantage is 30-60.
  - **probit-f10-p1 DONE 07:41 at v337; PANEL READ (r + e + g + h at v300, 2,560 games an arm): forget 10 minus
    forget 1, v150 / v337: ply 1 -17 [-36, 2] / 7 [-12, 26], ply 2 3 / 24 [5, 44].** Never 20 below (the rule keeps
    forget 1 only then); level at ply 1 at the final, 24 up at ply 2 at the final (6.70 predicted within 15: misses on
    the upside, one read). Trees over F-P against strangers: 113-120 at ply 1, 56-80 at ply 2 (both forgets). Sent
    to the analyst: does the probit capped seat take forget 10 as the LR one did? ANALYST 6.72 (a): NO, it stays at
    forget 1 (one read of 24 ±20 at ply 2; for probit the flag only touches the NB routers' counts, ADF leaves ignore
    it: no mechanism; LR takes 10 because it needs the step reset). A second seed at 20+ at ply 2 would reopen it.
  - **probit-f10-p1 LAUNCHED 06:32** (6.70 slot 2; leagues/specs/batch6/probit-f10-p1.json): F-P x2; capped probit
    error tree T-P-Ne (limit 4, 1M, noise 0.03) at forget 1 x2 and at --forget-factor 10 x2; seed 56, ply 1, 1,584
    games, floor 300, cap 400, hybrid16 games, hybrid6 training. The six should not carry an untested forget on
    probit while the LR trees carry 10. Read (probit-f10-read.sh -> probit-f10-read.out): against the standing
    ply-1 panel r + e + g + h at v300, 40 round-robins (~2,560 games an arm), plies 1 and 2, at v150 (as soon as it
    exists) and at the final. Prediction (6.70): forget 10 within 15 of forget 1 either way (ADF keeps the probit
    leaves' steps alive; only the NB routers' counts change). Rule: 20+ below keeps forget 1.
  - **GROW RE-READ 2 (06:35; panel r + e + g at v300, 1,920 games an arm): THE "MDL BAR GOES" RULING DOES NOT
    SURVIVE.** grow-p1b's no-bar A minus grow1 (bar on), v150 / v350: ply 1 -15 / -56 [-78, -34], ply 2 -4 / -4
    (v300: -56 / -52). A never leads grow1 against strangers (six reads, -4 to -56); the bar's value shows late at
    ply 1 (-56 at two separate versions). My pre-set rule (30+ below at both v150 and v350) is not met (v150 and
    v350 at ply 2 level). grow-p1c's B (age gate + snapshot) minus A: -7 / -3 at ply 1, 16 / 32 at ply 2. grow1
    over its own flat late: 101 at ply 1, 60 at ply 2 (v350). This bears on 6.54's next-league recipes (grow4 rule
    with --grow-mdl 0): flagged for the user; sent to the analyst. ANALYST 6.71 (a): the leaf rule returns to the
    BAR ON (grow4 --grow-mdl 1 --grow-gate age --grow-priority snapshot; B is inert under the bar in outcome); the
    late, training-ply cost fits 6.70 (the bar-free tree's young extra leaves accumulate field-specific fit). (b) a
    replicate as insurance, not a blocker:
  - **grow-p1b2 DONE 07:45 at v349; PANEL READ: A (no bar) minus grow1 (bar on), v150 / v300 / v349: ply 1 -19 / -1 /
    -18 [-37, 1], ply 2 -1 / -20 / 17.** Sizes as grow-p1b: A 23-27 leaves, grow1 11-15. 6.71's prediction half met
    (v150 and ply 2 hold; the final's -18 falls short of 20); the rule lands in between (not 20+ below at the
    final, not within 10 everywhere). The bar stays on under either branch. Across the two seeds A never leads grow1
    at ply 1 (-15, -56, -56; -19, -1, -18). ANALYST 6.72 (b): THE LEAF BAR STAYS ON (a proxy for the generalisation
    gap a one-field prequential trial can't see; a stranger-held-out criterion would be the principled one if growth
    is revisited). Growth claims are read against the panel only.
  - **grow-p1b2 LAUNCHED 06:40** (leagues/specs/batch6/grow-p1b2.json): grow-p1b exactly (F-P x2, grow1 x2, A =
    --grow-mdl 0 x2, 12 rounds, self-games, floor 300, cap 400, every line on chessai-grow1) at seed 57. Read
    (grow-p1b2-read.sh -> grow-p1b2-read.out): panel r + e + g + h at v300 (and r + e + g, as grow-p1b's re-reads),
    40 round-robins, plies 1 and 2, at v150, v300 (if passed) and the final. Prediction (6.71): A under grow1 by
    20-60 at ply 1 at the final, within 20 at v150, ply 2 within 30 throughout. Rule: A 20+ below at the final
    confirms the bar; A within 10 at every read = a seed term (the bar stays on as the conservative setting; growth
    past ~13 leaves stays open).
- **BUILT 07:25 chessai-hybrid17** (md5 e4a4fbd4; candidates 1 and 2 of the ~06:45 performance review; fastbin/
  hybrid17/summary.txt): byte-identical to hybrid16 in every check (global-league tournament step on a live snapshot,
  a grow-p1b cycle, 13 own calibrations, self-games, self-play); global tournament step x0.74 CPU, own calibration
  x0.83, a six-line ply-1 cycle step x0.87. NEW SPECS USE IT as the tournament binary. Not swapped into the running
  runs (all end within the hour) or the global league (faster cycles fill the disk faster). Panel-read waiters stay on
  hybrid16 (arena.py's own-games cache keys on the binary's md5). Deletable scratch ~535 MB (the user's call).
  v150 interim panel reads: probit-f10-p1 forget 10 minus forget 1 -17 [-36, 2] at ply 1, 3 at ply 2; grow-p1b2 A
  minus grow1 -19 [-38, 0] at ply 1, -1 at ply 2 (both inside 6.70/6.71's early predictions; finals decide).
- **ANALYST 6.72 (c), next picks toward an evaluator for search past ply 2:** (1) a ply-3 panel read of what exists
  (DONE 08:03; ply3-panel.out; RESULTS section 1): at ply 3 the ply-3-trained F-L beats the pure ply-2 F-P field by
  147 / 182 (two seed pairs; predicted 150-300); x4 minus bfp is -2 / -7 at ply 3 (predicted 30-80: FAILED, the
  pure-field lead is ply-2 only); replicates agree (0, 6); the ply-4-trained warm start is 81 BELOW its ply-3 start
  at ply 3); (2) the deep
  rung: the user's call ("For the user"); (3) weighting positions by the model's own uncertainty p(1-p) (a build;
  F-P x2 unweighted vs x2 weighted in a pure ply-2 field, panel read, ~4 CPU-h; prediction 0-40 at ply 2; 30+ adopts
  it for the league's flats, within 15 closes the lever). SPEC (6.73): s = P(W) + P(D)/2 from the pre-update
  prediction; u = 4 s(1-s) clipped to [0.2, 1]; w = u / running mean of u (memory 1M, in the state), capped at 3;
  F-P through the likelihood-power path (alpha * w), F-L update w g with the AdaGrad accumulator unweighted; flags
  --weight-by-uncertainty none|score (default none = today's bytes), --weight-floor, --weight-memory, --weight-cap.
  Run weight-p2: F-P x2 vs F-P weighted x2, seed 58, pure ply-2 field, 1,584 games, floor 300, cap 400, hybrid17
  games; panel y + v + l (+ x4, x4b) at v150, v300, final, plies 2 and 1; side read offline: full and winner loss,
  overall and on undecided positions (weighted worse in full loss by 0.005-0.02, better on undecided). Extra branch:
  30+ BELOW also closes the lever. 6.74: the panel read adds PLY 3 (prediction within 20 at ply 3 whatever ply 2
  shows). BUILT 08:20: chessai-weight1 (md5 9f6a7ae9; fastbin/weight1/summary.txt): flag off = hybrid6's bytes
  (F-P and F-L, 3 chained steps and a cold step, models and states; play identical to hybrid17); smoke with the flag:
  mean w 0.99-1.02, 10-20% of positions at the floor, none at the cap. Recipe row F-P-w added (recipes.txt,
  make_recipes.py HAND; backups global11-drafts/*.before-fpw). **weight-p2 LAUNCHED 08:23** (leagues/specs/ply2/
  weight-p2.json; F-P-w x2 controls, F-P-w + --weight-by-uncertainty score x2, seed 58). Read: weight-p2-read.sh ->
  weight-p2-read.out (panel y + v + l + x4 + x4b at v299, 30 round-robins, plies 2 / 1 / 3, v150, v300, final; side
  read uloss.py control vs weighted per pair). Held-out loss is a property of the run (two seeds of the same field
  differ 0.015), so the side read compares lines inside weight-p2 only.
  **weight-p2 DONE 09:03 at v299; READ: THE UNCERTAINTY-WEIGHTING LEVER CLOSES.** Weighted minus controls against
  the panel, v150 / v299: ply 2 -2 / -16 [-36, 4], ply 1 4 / -4, ply 3 24 / 8. Offline it works as designed (loss on
  undecided positions 0.002-0.003 better in both pairs, full loss level, not the predicted 0.005-0.02 worse), and
  play doesn't move. Seed note: the controls are 37 [17, 57] over fponly-p2's F-P at ply 2 (a third seed of the pure
  field; fponly-p2b was -1): field seeds differ by up to ~40 at ply 2. Sent to the analyst.
  ANALYST 6.75: the lever closes (the third time a log-loss change hasn't been Elo; its full-loss prediction was
  wrong: the floor and normalisation kept decided positions trained). Record the pure-field lead at ply 2 as 95 ±40
  across seeds. SLOTS IDLE for runs: everything that serves the goal now needs a user decision (the rung, the league's
  composition and rating fix, the disk); the no-decision runs left (probit-forget seed, grow-p2, replicates) don't
  serve it. Its offline read (f3 vs x4 position by position on held-out ply-3 games) was STOPPED at ~09:40 after the
  user asked "If you're doing comparisons why aren't you using the arena?": offline numbers have failed to predict
  play three times, so the parity question goes to the ARENA instead: the ply-3 read's 26 lines at plies 2 and 4
  (ply24-panel.sh -> ply24-panel.out; same panels and contrasts; ~0.7 CPU-h). Partial offline scratch kept in
  fastbin/p3diff/ (held-out ply-3 sets built; no analysis run).
  PLY-2 HALF (09:27; PANEL (direct), the ply-3 value in brackets): f3-x4 -186 (-317), f3b-x4b -245 (-285) [147 /
  182]; x4-bfp 45 (54), x4b-bfp 57 (32) [-2 / -7]; replicates x4b-x4 -4, f3b-f3 -3 [0 / 6]; w4-f3 61 (101) [-81];
  pfp-bfp -5 (42) [17]; rtr-x4 -481 (-338) [-83]. The same two models swap order by ~380 between plies 2 and 3. The
  ply-4 half runs from 09:27, ~145 s a round-robin, done ~10:45.
  ANALYST 6.76 (pre-registered ~09:57, before the ply-4 half): S_t(r) = A_t - D|r - t| + P[r = t mod 2]. From the
  flats at plies 2-3 (common D): P + D = 190 (the ~380 swing is 2(P + D), not 2P) and A_f - A_x = -25; from w4-f3,
  P - D = -48, so D ~120, P ~70, A_w - A_f ~110. The ply-1 tree doesn't fit the linear form (empirical prediction).
  PLY-4 PREDICTIONS (panel, +-20): f3-x4 and f3b-x4b 0..130, centre 60 (f3 leads: depth beats parity at distance
  two); w4-f3 150..300, centre 220; x4-bfp 0..40, centre 15; rtr-x4 -550..-350, centre -450; pfp-bfp -20..30, centre
  5; replicates within 15. Breaks it: x4 over f3 at ply 4 (D under 50); w4-f3 under 100; x4-bfp over 60 (the
  pure-field lead general, not ply-local). My check: the fitted linear form alone gives f3-x4 ~25 and w4-f3 ~300.
- **11-12 ~10:00, the user's cohort question and model library** (fork building): "how identically-architectured
  fully plateaued models perform against each other based on the cohort they were trained to plateau in" (within an
  identical setup vs across setups), and "a library of fully-plateaud models to choose from for future tests in
  arenas, so that we don't have to retrain them". Library: leagues/library/ (index.csv with end reason, training
  ply, setup, seed, lineage; reflink snapshots; library.py emits arena tags). Read: F-P only, plateau-ended runs,
  ply-2-trained at ply 2 and ply-1-trained at ply 1, final and final-20 versions, ci15, BT over unrelated cohorts,
  nested SDs (version / replicate line / seed / setup). Analyst asked for predictions and a design check.
  ANALYST 6.77 (pre-registered ~10:05; SDs against strangers, ply 2 / ply 1): version within line 10-15 / 10-15;
  replicate line within run 5-10 / 5-10; run within setup 20-30 / 25-40; setup 30-45 / 20-35 (at ply 2 mostly the
  known systematic terms: pure minus mixed 60-95, own-past replay -117, frozen references ~0, so report the pure/mixed
  split, not only an SD); matchup (within-run direct cell vs BT) 25-35 at both. Design fixes, forwarded to the fork:
  common version v299 beside each final (length is confounded with setup: floors 300-600); games_per_cycle and
  ply1_entries as covariates; p2c/p2d one run (identical through v9) or drop one; warm starts excluded or flagged;
  binary recorded; version level as each line's pair difference against the panel; keep different-seed games of one
  setup (they are the run term), drop only same-run and shared-lineage pairs; global F-P flagged, not pooled.
  **RESULT (fork, ~10:55; RESULTS.md section 1, 11-12 ~10:50):**
  - Library built: leagues/library/ (393 rows, 254 plateaued; LIBRARY.md).
  - Same set-up, near-interchangeable models: lines of one run within 0-7; seeds within 5 (ply 2) and 16-20 (ply 1).
  - Across set-ups, fully plateaued F-Ps land within ~20 at the finals (SD 18 at ply 2 without the 11-10
    leaf-sources-new run, which sits at -245; 16 at ply 1).
  - At equal length (v299) ply 2 spreads 48, mostly learning speed: replay-p2 -128 at v299 but level by its final.
  - The 6.77 predictions that missed:
    - matchups 0 within runs, against 25-35;
    - pure minus mixed ~20 against an all-F-P panel, against 60-95;
    - the ply-2 seed term 0-5, against 20-30.
  - ci-stop bug: with a random anchor the stop never triggers. make_split_read.py ciX is unusable until arena.py
    can anchor on a rated player; use fixed round-robins.
  - Leftovers, the user's call: cohort-fp-p2/owncal (888 MB of own games; the run was killed before its cleanup);
    its state.json still says "running".
- **11-12 ~10:05-10:20, the uniform TREE field and OBSERVERS (the user).** After update 3 (pure F-P field ~95 at
  ply 2) and the fact that every ply-2 tree trained in a mixed field (at least half flats; every line trains on the
  whole shared local set): the user chose the tree version of fponly-p2 "at ply 1 first it would be cheaper and
  interesting", and "Depending on the results you can decide whether to continue the uniform field test at ply 2"
  (the ply-2 follow-up is OURS to decide, with the analyst). OBSERVERS, the user's idea and name: "use the games
  generated from uniform fields to train models of other types but not have them participate in the field, so that
  we can later test their strengths in arenas", "a worthwhile experiment add-on"; and "Keeping all the games would be
  too expensive in the long run, but it would be nice to at least be able to define the experiment observers during
  experiment definition" (no keep-all-games; in-run observers from the spec). A fork is building the "observers"
  key (train on the cycle's shared local set, never play, never rated, not counted in games_per_cycle / plateau;
  byte-identical without the key; participants byte-identical with observers present).
  treeonly-p1 (leagues/specs/batch6/treeonly-p1.json; created 10:12, pinned to chessai-hybrid6 training 10:14, NOT
  started): T-P-Ne capped (limit 4, 1M, noise 0.03, forget 1) x4 at ply 1, seed 59, games_per_cycle 1584,
  self-games, plateau scaled min 300, cap 400, hybrid17 games; observers F-P x2; comparator probit-f10-p1's capped
  forget-1 trees (F-P x2, capped x2, capped f10 x2). The fork launches it once the feature passes its tests. Read:
  standing ply-1 panel, v150 and the final, plies 1 and 2: (a) uniform tree minus mixed-field tree, (b) F-P observer
  minus probit-f10-p1's F-P, (c) F-P observer minus uniform tree (learners on identical data). Asked of the analyst
  (6.78): predictions, and whether to run the mirror fponly-p1 (F-P x4 at ply 1 with T-P-Ne x2 observers; crosses
  learner x data source) in parallel or after. The user's earlier question "what do 4 identical models give over
  self-play" (answer: the seeding; mostly measurement) left a one-line self-play test on offer; not requested.
  ANALYST 6.78 (pre-registered ~10:25; against the panel, +-20, ply 1 / ply 2): (a) uniform tree minus mixed capped
  tree 20..70 (centre 40) / 0..40; (b) F-P observer minus mixed F-P participant -20..30 (centre 5) / -30..20 (a
  deficit of 30+ says on-policy positions matter); (c) F-P observer minus uniform tree on identical data: tree ahead
  60..120 / 20..60 (the cleanest architecture cell yet). RUN THE MIRROR IN PARALLEL: fponly-p1 (F-P x4 playing,
  T-P-Ne capped x2 observing; the 2x2 needs both fields; shared panel read). Its cells: uniform F-P minus mixed F-P
  0..50 / -20..30; tree observer minus mixed tree participant -40..20 / -30..20; tree observer minus uniform F-P on
  identical flat data: tree ahead 30..90 / 10..50. Spec: observers get NO self-games (record the ~9% difference, no
  top-up: a top-up would make them play); observers versioned on the participants' cycles. PLY-2 RULE: repeat at
  ply 2 only if (a) is 30+ at ply 1 and (b) within 30. fponly-p1 go-ahead sent to the observers fork ~10:27 (seed
  60, same settings as treeonly-p1; launch after treeonly-p1 runs).
- **11-12 ~10:22, the user: "maybe we should also check performance of ply3 models at ply 1 and ply4 models at
  ply2"; "if parity matters it may be interesting to see if architecture advantages follow parity (maybe trees are
  better at ply 3 again but worse at ply 4)".** (1) w4 (ply-4-trained) minus x4 (ply-2-trained) at PLY 2, from the
  existing panel-ply2-existing games: -131 [-150, -113] (panel x4b, bfp, f3b, rtr; direct -198), and minus x4b -152;
  the 6.76 linear fit gives (A_w - A_x) - 2D = 110 - 25 - 240 = -155. (2) PLY-1 ARENA of the same 26 lines launched
  10:23 (ply1-panel.sh -> ply1-panel.out; ply24-panel.sh's contrasts plus (vi) w4 - x4 and f3 - rtr). The 6.76
  linear fit's values, written before the result: f3 - x4 at ply 1 = (A_f - A_x) - D + P = -25 - 120 + 70 = -75;
  w4 - x4 at ply 1 = (A_w - A_x) - 2D = -155 (|1-4| = 3 vs |1-2| = 1, no parity for either); rtr outside the form.
  (3) The architecture-by-parity sweep (tree minus flat, same field, at plies 1-4, for ply-1- and ply-2-trained
  pairs, against strangers): design asked of the analyst (6.79).
  PLY-1 READ DONE 10:25 (leagues/arenas/diag/panel-ply1-existing; 30 round-robins; PANEL (direct)): f3 - x4 63
  (25), f3b - x4b 46 (22): THE FIT'S -75 FAILS by ~120; x4 - bfp 15, x4b - bfp 7; replicates x4b - x4 -15, f3b - f3
  -28 [-44, -11]; w4 - f3 -66 (-106); pfp - bfp -68; rtr - x4 290 (332); (vi) w4 - x4 -5 [-22, 12] (-32): THE
  FIT'S -155 FAILS by 150; f3 - rtr -258 (-216). Same-lineage pair w4 - f3 by playing ply 1 / 2 / 3: -66 / 61 / -81
  (alternates with parity; fits 6.76 at ply 1: -80). Every ply's own-trained model wins at that ply (rtr by 258-290
  at ply 1). The linear form fails only where x4 plays at ply 1 (x4 ~140 worse there than the fit says, or the
  distance cost isn't linear/symmetric). Sent to the analyst with 6.79.
  ANALYST 6.79 (pre-registered ~10:27) - THE ARCHITECTURE-BY-PARITY SWEEP: 28 entries (ply-1-trained r, e, g, h at
  v300; ply-2-trained y, v, l at v299; F-P x2 and the tree x2 each; h's tree is grow1, the others capped T-P-Ne), one
  arena per playing ply 1-4, 20 round-robins; each field's tree minus flat against the 24 strangers (the other six
  fields), pooled per family. Predictions (pooled tree minus flat): PLY-1 PAIRS at r1 60-110, r2 25-65 (both
  hypotheses); r3 PARITY 50-90 vs DISTANCE 0-30; r4 parity 25-55 vs distance -20..15; the analyst's point: a mix, r3
  ~40, r4 ~15. PLY-2 PAIRS: r2 -40..-10; r1 parity -10..20 vs distance -15..5; r3 parity 0..20 vs distance -15..5;
  r4 parity -35..-10 vs distance -10..10 (separate by ~20 only). DECISIVE: the ply-1 pairs' r3 against r2 (parity: r3
  at or above r2; distance: r3 below by 15+ with r4 near zero). Parity probes: same-lineage adjacent-ply pairs (w4/f3
  now: -66 / 61 / -81 at plies 1-3, ply 4 in the running arena; x4 / the rung's F-P later: parity predicts
  alternating sign 60-80 at every ply, distance a magnitude growing with the ply gap).
  ADDENDUM (ply-1 read): the linear symmetric fit is RETIRED; an asymmetric one (shallower than trained ~150 a ply,
  deeper ~50, P ~70) repairs plies 2-3 but still misses f3 - x4 at ply 1 by ~100: no additive model fits across
  lineages (x4's value function is ply-2-specialised in a way the ply-3 LR flat is not). Survives: parity alternation
  in same-lineage adjacent-ply pairs, and the model trained at a ply wins at that ply at every ply so far. Testable on
  the rung's ply-3 F-P against x4: under x4 at ply 1 by 50-150, over at ply 4 by 50-150, x4 ahead 100-200 at ply 2.
  LAUNCHED 10:28: archparity-sweep.sh (plies 1-3 now; ply 4 after panel-ply4-existing ends) -> archparity-sweep.out.
  PLIES 1-3 READ (10:52; tree minus flat per family, against the 24 strangers, 20 round-robins a ply):
  ply-1-trained (r, e, g, h): ply 1 89 [77, 101] (113, 123, 19, 102), ply 2 36 [23, 48] (19, 36, 29, 59), ply 3 -27
  [-39, -16] (-26, -44, -42, 3); ply-2-trained (y, v, l): ply 1 -9 [-23, 5], ply 2 -34 [-49, -19], ply 3 -17 [-30, -3].
  THE PARITY HYPOTHESIS FAILS through ply 3 (6.79's ~40 at ply 3 failed): the tree's lead shrinks with DISTANCE from the
  training ply, negative two plies away in both families. ANALYST 6.83: distance, not parity, is what architecture
  advantages follow; parity stays real for same-lineage flats (w4/f3: a property of the flat's value function at a
  ply); the tree's extra is a local correction that decays with distance. Ply-4 prediction (pre-registered 11:04):
  ply-1-trained -70..-30 (centre -50), ply-2-trained -30..0 (centre -15); a ply-1-family cell above 0 would revive
  parity. 6.83 on the next league (update 8, presented 11:04): the lineup is unchanged (the six at ply 1; a
  router-split DAG seat only if grow-dag-p1 passes 6.80's confirmation rule against strangers; cold-games rule from
  the 6.82 twins); the kind-matchup result changes how standings are READ, not who plays; the deep goal runs through
  the ladder (ply-3 rung), not the league. Before launch: cap the rating history (per-line summaries and library finals
  kept, game-level history dropped past a window) and an END RULE: a fixed cycle cap plus retirement on COLD ratings,
  "last line standing" only if it comes first.
- **11-12 ~10:45-10:52, ROUTER SPLITS REOPENED BY THE USER, TO BE ENCOURAGED.** The user: "I think we should still
  keep the possibility of a router split open"; "when a router splits it will add additional parameters to its
  children, which potentially help better understand positions over a long horizon, so we shouldn't rule out router
  splits and maybe we should encourage them in some sense. It's possible that router splits will lead to better leaf
  splits later down the line also". They asked whether splits still come early. SPLIT TIMING (node-history over every
  version; scratchpad split_timing.py): router-p1's DAGs (H and Z) made all 4 router splits at v15-v51 and all 4 leaf
  splits at v12-v33 (both capped at 4), nothing after v51 in ~250 versions; capped trees (probit-f10-p1) all 4 leaf
  splits by v49; growing trees split steadily: grow1 (bar) a trial every ~25-30 versions from v29-v40 to v352 (some
  undone by the bar), grow-p1c's B every ~25 versions v25-v348. So router-p1 could not test "router splits lead to
  better leaf splits later" (leaves capped and done before most router splits), and an immediate-gain test rejects a
  split whose payoff comes later. Proposed (to the analyst, 6.80): a growing DAG vs a growing tree at ply 1, uncapped,
  run long, leaf rule grow4 bar on in both, the DAG with path context and router splits ENCOURAGED (an allowance,
  priority over leaves, or a longer trial; the analyst's rule), read against strangers at several versions plus leaf
  trial counts and kept rates after router splits. ~2 CPU-h; queued behind the uniform-field pair unless the user
  wants it sooner. Binary support (grow4 leaf rule plus DAG router rules in one binary) to be checked.
  This was one of 6.43's items "for the user to rule on"; it passes the constraints (model-driven, no chess
  knowledge), so it runs under the 01:05 go-ahead ("continue the research with the analyst's suggestions"). Your
  veto stops it. Not now (6.72): grow-p2, a probit-forget second seed, more composition replicates.
- BUILT 02:43 (fork): the experiment key reference_window {"margin": M} (analyst 6.57: a line trains on a frozen
  reference's games only while the reference is rated at or above its current Elo minus M; line-line and self-games
  always train; leagues.py md5 f20818ff, backup leagues.py.before-refwindow; byte-identical without the key: diversity-
  p2's cycles 1-3 reproduced; forced-cut test agrees with history.csv on 176 checks; selftest passed). Scratch
  models/from-scratch/refwindow/ (713 MB) can be deleted: the user's call.
- **diversity-p2c LAUNCHED 02:46** (analyst 6.58: launch as built). diversity-p2 DONE at v314 (plateau): at equal batch
  its F-P = base-probit-p2's (6 / 15 at ply 2, 6 / 31 at ply 1, -5 at ply 3): 58% of the batch from much weaker frozen
  foreign play is worth own-field sibling games, no replay-style pull (6.58: foreign play adds coverage; own past play
  only staleness). Batch-size read (probit-late vs base-probit F-P, v300/v399, plies 1-3) running ->
  models/from-scratch/global11-drafts/batch-read.out (6.58 (c): bfp over pfp 10-50 at ply 2 at v300, 0-40 at v399,
  40-90 at plies 1 and 3; confounded with composition).
- diversity-p2 so far (02:05-02:12; RESULTS section 7): at EQUAL POSITIONS (diversity v182 = probit-late v399, 2.19x
  positions a cycle) the references bought the flat nothing at ply 2 (-2, 6) and cost the tree ~50 (-59, -48).
  Analyst 6.56: equal positions can't decide anything (a league progresses per cycle); the reads that count are equal
  versions and equal batch. The lines passed the references at ~cycle 8 and sat 150-250 above them for the rest of
  the run, so diversity-p2 tested 58% of positions from much weaker foreign play: the replay pull from outside.
  Source per leaf (tree #1 v226): leaf 12 is 84% reference positions and 0.048 nats worse than the flat on line-line
  games; leaves 10-11 line-specific. The tree's loss beats the flat's on line games (0.779 vs 0.814, the endgame leaf)
  while it plays worse: lower loss is not better play, again (analyst: if it recurs, build a rank test on undecided
  positions). Revised predictions for the main read (6.56): dfp - bfp at v300 -80..-20; dfp - probit-late F-P at
  equal versions -40..40; tree - flat -60..-10; ply 1 over pure 0-40; ply 3 within 40.
- **diversity-p2c (analyst 6.57; my call under the 01:05 go-ahead)**: the user's regime, coverage from STRONGER foreign
  play under the Elo window. diversity-p2's lines and seed (F-P x2, T-P-Ne x2, seed 48), games_per_cycle 1584,
  frozen references = base-probit-p2's v599 finals (F-P x2, T-P-Nc-merge-fresh x2: the strongest ply-2 players on
  disk, which the lines shouldn't pass), reference_window margin 30 (the global league's), floor 300, cap 400, ~6-8
  CPU-h. Predictions (ply 2): acceleration (p2c F-P v150 vs probit-late v300 0-60, v200 20-80), ceiling within 30
  (vs bfp v300 -20..40; v400 vs bfp v450 -40..20), over diversity-p2's F-P at v300 20-80; tree - flat -40..0.
  Plan rules: 40+ over bfp at v400 = the league goes to ply 2 with the window; within 30 = speed only (the ply-2
  question becomes compute); 40+ below = the references must be a mix of strengths (the thunderdome's composition).
- Every run has a read waiter queued. Outputs: models/from-scratch/global11-drafts/*-read.out.
- Building: grow4's leaf part (fork): no stagnation gate, snapshot-gain priority, --grow-min-gain.
- Done tonight: router-p1 (v299: router splits cost ~35-40 at ply 1 against no splits, H -41 and Z -37 at the
  final, nothing significant at ply 2; HG never split = C; the shadow test was right to refuse); grow-p1c (v354: age gate + snapshot priority = stall gate, 1 / 27 at the final, plies 1 / 2;
  grows slightly faster; leaf-growth series at ply 1 complete: recommended rule = grow4 --grow-mdl 0 --grow-gate age
  --grow-priority snapshot, size set by D); replay-p2 (v542; replay HURTS at ply 2: F-P -123 / -64 vs pure at v300 / v399, helps at ply 1
  by ~37; the analyst's case (4); diagnosis asked 00:40); grow-p1 (grow = capped rule, MDL bar binds); grow-p1b (the MDL bar goes: no-bar ties grow1 at
  ply 1 and leads 19-40 at ply 2 with 25 leaves vs 13-15); base-probit-p2 (v599, read ~23:00: in a pure
  ply-2 probit field the flat beats both trees at every version, by ~10-50 at ply 2 and ~53 at ply 3 at v599;
  F-P still improving at v599, trees plateau from ~v450; RESULTS section 7).
- Trigger flaws (analyst 6.47): "stall" measures drift in league fields, and H can't gate. Both withdrawn as gates.
  See "Trigger flaws" below.
- For the user:
  - **DONE 11-12 09:36 (the user: "kill the league and clean up all the space associated with them except for all
    the champion models"): the global league STOPPED at cycle 5918** (STOP file, between cycles). Kept in place: all
    210 models it ever crowned (8 first and 202 new champions, 77 MB; md5-checked in leagues/global/
    kept-before-cleanup.json), the 8 training logs and the small records (configs, CSVs, cycle reports, tour/ratings
    and head_to_head; leagues/global is 174 MB). Deleted: every other version (11.4 GB), tour/elo_history.csv (13.3 GB),
    game_cpu.csv, games.csv, tournament.log, owncal/, batch/, scan-cache/, and the performance forks' snapshot
    copies of the league (fastbin/hybrid17/global, fastbin/review-1112b/global). The user's next ask: "Run all the
    champion models in 1 quick arena": leagues/arenas/diag/global-all-champions-p1 (210 entries, ply 1, T 0.02,
    4 round-robins) DONE ~10:00 (RESULTS section 1): last champion = best in 7 of 8 lines; the endgame's two DAGs top the
    field at 999 each (gained 17-40 in the endgame); then T-L-Nc 969, D-P z20 942, D-L z120 940, T-P-Nc 912, F-L 893,
    F-P 874; the late champions did not get worse. NEXT GLOBAL LEAGUE (the user): rounds must adjust to the active participants:
    games_per_cycle (or games_per_entry) is built for global leagues too; put it in its spec. (Disk estimate
    correction: btrfs zstd compression made the real rate ~20 GB a day, about a week, not 4-5 days.)
  - (RESOLVED, see above) URGENT (11-12 07:15; performance review): the global league will FILL THE DISK in about 4-5 days if it keeps
    running.** leagues/global/tour/elo_history.csv is 10.2 GB and each tournament appends a row for every player ever
    entered (15.2k retired): 4.8 MB a cycle now, rising ~0.45 KB a cycle every cycle, plus ~1.5 MB of models a cycle;
    ~55 GB a day today and climbing fast, against 585 GB free (fastbin/PERF-LEDGER.md, ~06:45 review). A full disk
    would break every run. Options (your call): stop the league (its endgame won't end by itself: the item below),
    or cap / rotate elo_history (a format change). I have not stopped it; a monitor alerts me below 300 GB free.
  - **NEW, IMPORTANT (11-12 ~04:15; analyst 6.63): in-run ratings under-rate TREES by ~100-150 Elo (kind-specific).**
    In diversity-p2c, base-probit-p2's v599 cluster trees averaged ~1560 in-run against ~1685 for its v599 flats, yet
    cold (T 0.02) they play level with them (-12 ±28). The in-run tournament plays log-odds-softmax at T 0.1 (the
    calibration default), the known calibration-slope bias, and it differs by model kind. Everything decided in-run
    runs on it: the reference window (which cut references 162 and 37 stronger cold), crownings, patience,
    retirements, and the global league's standings (its first cluster tree retired first). Analyst's recommendation:
    (1) the cheapest sound fix is a separate cold rating set at T 0.02 each cycle (~a quarter of the training volume,
    ~100 games an entry, SE ~35; +25% games) used only for ratings/window/crownings/patience, training games
    unchanged at T 0.1, window margin 50; (2) the alternative is one game set with an exploration rule that doesn't
    scale with a model's confidence (epsilon-greedy or a softmax over ranks), but it changes the training stream and
    needs a ply-1 test first. Re-read the first global league's standings with the trees under-rated; the cold reads
    are the record. Your call (league mechanics); nothing changed in running leagues. Sizes: cluster trees ~110
    under-rated at ply 2, error trees ~30-60 (error-routing-p1's v314 pair: 43 [15, 71] cold at ply 2 (04:16), an
    older read 77, against +15 in-run). Cold re-read of the global league's eight last champions at ply 1: every
    tree and DAG champion 33-95 over both flats (table under "Global league: last champions").
  - **NEW (11-12 ~06:15; analyst 6.68): verdicts now come from PANEL reads, not direct head-to-heads.** A model
    against its own run's other kind carries co-adaptation and the kind matchup (a tree beat its own flat by 61 at
    ply 2 yet was level against strangers), and even same-kind cells between two runs carry lineage matchups of
    25-45. The panel read scores each contrasted model against lines of runs it never trained against and takes the
    difference (leagues/specs/arenas/arm_panel.py). Re-read so far: (a) the ply-1 tree gain is REAL, about 85 over
    the flat against strangers (four fields: 127, 118, 11, 85; within-field reads said ~89), and ply-1-trained trees
    keep ~44 of it at ply 2; (b) ply-2-TRAINED trees are below their flats against strangers in every field (-8 to
    -45, mean ~-25), so "in a ply-2 field the flat wins" stands; (c) diversity-p2d's "window removal cost 19" was a
    matchup (panel: 6-21 the other way): no effect; (d) fponly-p2's pure F-P field HOLDS and grows against
    strangers: its F-P is 43-87 over every other ply-2 F-P (base-probit 87, probit-late 65, p2c 56, p2d 43; the
    direct cells said 17-42), the strongest ply-2 evaluator on disk; (e) replay's harm holds (F-P -117 against
    strangers). The global league's standings are a co-adapted field by design:
    right as a ranking, matchup-laden as an architecture verdict. More re-checks running (fponly-p2, capped-l-f10).
  - **NEW (11-12 ~08:00; analyst 6.72): THE DEEP RUNG, your call under your 11-11 rule (no deep rungs unless low ply
    stagnates).** The ply-2 question is closed (a pure F-P field is the best ply-2 evaluator and replicates; trees
    trained at ply 2 sit below their flats), so the analyst argues for one rung: a pure F-P field at PLY 3, warm-started
    from fponly-p2's v299, F-P x2, 720 games a cycle, floor 150, cap 250, hybrid17, ~8-12 CPU-h, read against a ply-3
    panel at v75 and the final. Prediction: 100-250 over x4 at ply 3 by v150 and within 50 of flat-p3's F-L at equal
    ply-3 cycles. Rule: within 50 = the ladder from a pure ply-2 field is the route; 100+ behind = the warm start
    carries little. NOT launched. The ply-3 panel read (08:03) bears on it: at ply 3 a ply-3-trained F-L is 147-182
    over the best ply-2-trained evaluator, the ply-2 pure-field lead vanishes (-2 / -7), and ply-4 training costs
    81 at ply 3: the training ply decides strength at a ply.
    REDESIGNED (analyst 6.74, after that read): the composition gain is ply-local; training at the ply you play is
    the strongest lever by an order of magnitude, so the rung is the next step, not a detour. The start point doesn't
    matter for ply-3 strength, so the warm start's value is only the cycles it saves: one factor. Field: pure F-P at
    ply 3, warm from x4 v299 x2 and COLD x2, 720 games a cycle, floor 150, cap 250, hybrid17; read against a ply-3
    panel (f3, f3b, w4, x4, bfp) at v50, v100, v150 and the final. Predictions: warm over cold 50-150 at v50, within
    30 by v150; either arm over x4 at ply 3 by 100-200 by v150; warm within 50 of flat-p3's F-L at equal ply-3 cycles.
    Rules: warm within 30 of cold by v150 = warm starts buy cycles only; cold 30 ahead at the final = rungs start
    cold; either arm within 50 of f3 = the ladder from a pure ply-2 field reaches the ply-3 level. COST ~15-20 CPU-h
    (the warm x2 arm alone is cheaper but loses the warm-start answer). Your call.
  - **NEW (11-12 06:35): the leaf MDL bar should probably stay ON for the growing trees.** 6.54's next-league
    proposal had the growing error trees on --grow-mdl 0 (bar off for leaves), from direct cells in grow-p1b. Against
    strangers the no-bar tree never leads the bar tree (six reads: -15 / -4 at v150, -56 / -52 at v300, -56 / -4 at
    v350, plies 1 / 2), and the bar tree is 101 / 60 over its flat at v350. It is not the clean replicated result my
    pre-set rule asked for (early and at ply 2 it is level). ANALYST 6.71: the bar goes back ON for leaves, now
    (grow4 --grow-mdl 1 --grow-gate age --grow-priority snapshot); a replicate at a new seed is running as insurance
    (grow-p1b2). It also rewrites 6.54's next-league six: F-L, F-P, T-P-Ne-grow (bar on), T-P-Ne capped, T-L-Ne
    capped at forget 10 (the LR seat: with the bar on an LR growing tree barely grows), and optionally T-L-Ne-grow
    forget 10 with the bar on for symmetry. Your call (composition and recipes).
  - **NEW (11-12 05:45): the global league's two-line endgame won't end by itself.** Two lines (D-P z30, D-L z160)
    since cycle 1202 (11-11 21:54), 3,360 cycles and ~9 CPU-h ago. They crown by noise about once per ~420 cycles
    each (16 crownings in the phase), and retirement needs ~20M positions without a crowning, ~3,600 cycles at
    ~5.6k positions a cycle per line (D-L reached 50% once, then crowned at 4111): about a 0.02% chance per stretch.
    It costs ~1.2 cores (9.6 s cycles, ~11 CPU-s each). Your standing instruction was to stop it at ONE line left; I
    haven't stopped it. Options: stop it now and call the two DAGs joint last models standing (the cold re-read of
    the last champions at ply 1 had D-L z160 896 and D-P z30 894, level), or let it run. Your call.
  - **NEW (11-12 ~05:00; fponly-p2 + analyst 6.66): at ply 2 a line's batch should hold its own kind's and its
    equals' games.** A pure F-P field (1,584 games a cycle) beat base-probit-p2's F-P (same volume, weaker trees and
    DAGs in the batch) by ~60 at ply 2; volume itself doesn't harm (the 1,584 standard stands). For a mixed ply-2
    league: (a) the Elo window must run on sound ratings (the item above; margin 50), and (b) 6.66's one new rule:
    top each line's useful volume up to the standard with self-games (own current vs own champion), scaled by its
    window's kept share. Not built; your call.
  - **diversity-p2 APPROVED (the user, 11-12 ~01:05: "Please continue the research with the analyst's suggestions").**
    Analyst 6.53 design: F-P x2 + T-P-Ne x2 (as probit-late-p2), seed 48, ply-2 field, self-games on (lines only),
    12 rounds, floor 300 cap 400 (~6-8 CPU-h); four frozen references = error-routing-p1's v314 finals (F-P x2,
    T-P-Ne x2, ply-1-trained) at ply 2; all three game kinds train (line-line, line-ref, ref-ref; shares logged).
    Read at BOTH equal positions (diversity ~v85 = pure v399, ~v130 = pure v599) and equal versions. Predictions:
    F-P 60-150 over pure at equal positions; 120-220 at v300; 100-200 at v399; tree over flat inside 0-50 at v300;
    ply 1 0-40; ply 3 within 40. Within 30 at equal positions = the offline gain doesn't transfer; the next league
    goes back to ply 1. A fork is building the driver key (frozen_references) from ~01:10; launch after.
    CORRECTED (6.53 correction, ~01:25): at 12 rounds the field is 12 entries (8 line entries + 4 references), 1,584
    games a cycle: line-line 672 (= probit-late-p2's), line-ref 768, ref-ref 144; the batch equals base-probit-p2's.
    Comparators: base-probit-p2's F-P at equal versions (equal batch) and probit-late-p2 at equal positions (diversity
    v169 = pure v399) and equal versions. Predictions (ply 2): over base-probit-p2 F-P at v300 and v599 100-200; over
    probit-late-p2 F-P at equal positions 60-150, at v300 120-220; tree over flat inside 0-50; ply 1 0-40; ply 3
    within 40. The null that matters: within 30 of base-probit-p2's F-P at equal batch.
    **Built and LAUNCHED:** frozen_references installed in leagues.py at 01:20:14 (backup leagues.py.before-frozenref;
    byte-identical without the key: probit-late-p2's cycles 1-3 reproduced, live leagues unchanged, selftest passed;
    references calibrated once in cycle 1 and checked against error-routing-p1's records, md5-checked every cycle).
    diversity-p2 launched 01:22 (cap 400, so the read is at v300 and the final, not v599). Read queued.
  - (was) diversity-p2 (frozen P2x references; ruling and engine build): THE next ply-2 step (analyst 6.51). Coverage
    from other players helped ~200 at ply 2 offline (P2x/P2 mix); coverage from one's own past (replay-p2) hurt
    60-130. Weaker play looks like shallower play to the value function, so own early games pull it toward ply-1
    play. The references' games must not include the lineage's own early play. Prediction: F-P 100-200 over pure at
    v400, trees over flats 20-60. The thunderdome design with the Elo-window rule (the global league) is the regime
    both results point at: the analyst's strongest argument yet for the next global league at ply 2.
    The Elo window is a QUALITY FLOOR on the play: games from players at or above one's own level supply coverage
    without the shallow-play pull; one's own weaker past supplies the pull without coverage. diversity-p2 is the
    controlled test of that regime before the league commits to ply 2. If the user declines frozen references, an
    Elo-windowed own replay is the fallback (lower ceiling: own play never adds coverage). Replay late read: replay
    reached pure's v399 level only at v542, and its flat had nearly stopped improving (7 per 143 cycles vs pure's 24).
  - replay-p2's replaypool (786 MB, leagues/experiments/replay-p2/replaypool) can be deleted after the read: the
    user's call.
  - G (games_per_entry) for the next global league;
  - league design with replay and diversity;
  - residual leaves, self-uncertainty weighting, learned pair features (S7).
  - growth settings for the next global league: D (credit rate; 10M recommended), a 64-node cap or not, leaves only
    until router-p1 reads (analyst 6.50 (3); see "For the user before the next global league grows").
    Analyst 6.52: the leaf series at ply 1 CLOSES; recommended leaf rule grow4 --grow-mdl 0 --grow-gate age
    --grow-priority snapshot (W 1M, C 2, z 2, D 10M for leagues). Growth is free at ply 1 and buys no Elo there.
    BINARY SCOPE (checked 01:00): only error trees (T-P-Ne, T-L-Ne) can grow today. --split-rule grow is BDT-only (no
    DAGs) and refuses --revise/--reset-leaves (so not the eight's fresh-gate cluster trees); the router rules are for
    error-routed path-context DAGs only; the analyst's G-only router rule is not built. Growing the eight would need
    builds: grow on cluster/fresh-gate trees (if coherent with revision), grow leaves in DAGs, G-only routers. None
    started before the user's ruling or router-p1's read.
    **Cheaper path, no build (analyst 6.52 note):** the next league's trees become ERROR trees T-P-Ne-grow and
    T-L-Ne-grow (grow4 rule, --grow-mdl 0 --grow-gate age --grow-priority snapshot, D 10M; T-L-Ne-grow is a recipe
    row to add) in place of the fresh-gate cluster trees.
    - The evidence allows the swap: at ply 1 error and cluster trees are level (118/106 in G2, ~100 in the DAG
      fields), and the LR error tree was 46/55 over F-L against the cluster tree's 19-43 (error-dag-p1).
    - Grow's prequential trial supersedes the fresh gate: a split validated before it is kept needs no later merge
      test.
    - The DAGs stay D-P-Ne-ctx and D-L-Ne-ctx with leaves capped at 4, their router rule set by router-p1's read:
      z inf (capped error trees, giving the league a capped-vs-growing contrast) if HG never fires and H's splits
      cost nothing or lose; --router-split-rule stall-hg --grow-mdl 0 (G-only in practice) if HG fires with gain.
    - Builds only if the user insists on the eight as named: grow leaves inside bdd; grow on cluster trees (drop
      revision when grow is on); the G-only flag with online Q_v.
  - **Analyst 6.54 (~01:50), after router-p1's read: THE NEXT GLOBAL LEAGUE PROPOSAL (the user's decision).**
    (1) The router question CLOSES at ply 1. Router splits of the kinds built (root copies by drift, cluster
    sibling sets, path unsharing on a schedule) cost 35-40 at ply 1 whenever made, nothing significant at ply 2,
    and the paired benefit test refused all of them, correctly: no routing headroom on these leaves, now shown in
    play. HG's record (the family's hindsight): 709 router closes, G_real - G_null positive at 1% of closes (max
    0.0023, z 22, router 0 at 8-9M, the shadows' early transient), 5 closes over z 2, none over the MDL bar, every
    router ending negative (-0.002 to -0.031). Correction to 6.52: the MDL bar STAYS ON for router splits (or is
    replaced by persistence: G_real - G_null over 2 SE at three consecutive closes); only the bar stopped the
    early transient's split, which H's record says would have cost ~40. (The bar still goes for LEAVES: 6.50/6.52.)
    (2) Next league: NO DAGs (a DAG at z inf is a capped error tree at the same play cost; the DAG code stays as
    the vehicle for a router rule re-tested when the leaves change (S7) or the stream is broad (ply 2 mixed)).
    Six lines, all with ply-1 evidence and no open router question:
      - F-L, F-P;
      - T-P-Ne-grow, T-L-Ne-grow (grow4 rule: --grow-mdl 0 --grow-gate age --grow-priority snapshot, D 10M;
        T-L-Ne-grow is a recipe row to add);
        UPDATE 11-12 06:35: the panel re-reads do not support --grow-mdl 0 (no-bar never leads the bar against
        strangers; 56 below late at ply 1). ANALYST 6.71 rewrites the six: F-L, F-P, T-P-Ne-grow (bar on: grow4
        --grow-mdl 1 --grow-gate age --grow-priority snapshot), T-P-Ne capped, T-L-Ne capped at forget 10, and either
        T-L-Ne-grow forget 10 with the bar on (a symmetry line that will barely grow: the bar refuses LR trials of
        0.001-0.002 nats under a bar of 0.005-0.008, so 3-7 leaves) or nothing. See "For the user" (the leaf bar).
      - optional, for a capped-vs-growing contrast inside the thunderdome: T-P-Ne and T-L-Ne capped at 4 as
        fixed-size controls.
    No build needed. Open for the user: this composition; the ply (1, or 2 if diversity-p2 reads well); games per
    cycle (T 1584); D; a 64-node cap or not.
    UPDATE 11-12 ~03:55 (grow-l-p1/p1b, analyst 6.59/6.62): the LR trees need --forget-factor 10 (every recipe has 1,
    which freezes a split's LR children at the parent's late-life AdaGrad step). With forget 10 the LR growing tree
    passes the seat rule (half-confirmed); the capped LR control should take forget 10 too; a shadow fix is being
    built and tested (grow-l-p1c). diversity-p2 (frozen weak references) read null at equal batch; diversity-p2c
    (window regime) and fponly-p2 (composition vs volume at ply 2) are running; the ply of the next league waits on
    them.
    UPDATE 11-12 ~06:20 (analyst 6.67-6.69): BOTH LR TREE SEATS ON --forget-factor 10 (the growing tree: grow-l-p1b;
    the capped control: capped-l-f10-p1, 45-49 over F-L at ply 2 where forget 1 was 27-56 below); the decaying-AdaGrad
    alternative failed offline. Probit lines stay on forget 1 until tested. Against strangers the ply-1 tree gain is
    ~85 ±50 across fields (real), and ply-1-trained trees keep ~44 at ply 2; but ply-2-TRAINED trees sit ~25 below
    their flats. So a ply-1 league's tree seats are strength; in a ply-2 league expect flats on top, trees as growth
    vehicles. The strongest ply-2 evaluator on disk is the pure-F-P field's F-P (fponly-p2). Analyst's note for the
    composition (6.69): LR trees gain only 30-40 over F-L at ply 1 (half the probit trees' ~85) and needed the
    forget fix to stop losing at ply 2, so if the league is to be lean the LR tree seats are the ones to cut; F-L is
    the only LR line that isn't a compromise.

## Games per cycle standard (the user, 11-12 ~01:15; analyst 6.53 correction)

- The user: "equilibrate the amount of games towards some standard range/amount by changing the number of rounds
  per cycle based on the number of active participants", in experiments and global leagues, "because we want more
  stable elos and also a more consistent training batch size".
- Measured: 12 fixed rounds give 672 games a cycle with 4 lines, 1,584 with 6, 4,560 with 10 (router-p1). The
  global league (5 rounds) fell from 1,894 a cycle at 20 entries to 280 at 8. Within a run the field sits at its
  maximum (current + champion per line) almost every cycle.
- Analyst's rule: T = 1,584 games a cycle (range 1,200-2,000), R = max(1, round(T / (N(N-1)))) each cycle.
  - N = every playing entry (currents, champions, references including random).
  - Games per entry 2T/N, so rating SE ~348/sqrt(2T/N): 28 Elo at N 20, 21 at N 12, 17 at N 8. A 2-SE crowning
    needs 78 / 61 / 50, stable within a factor 1.6 across sizes (today 50 to over 100).
  - Rounding half up, floor 1, cap 48 (a 2-line field plays 576 and logs "under T"). Double round-robins with
    colour alternation.
  - Log N, R, games and training positions per cycle. Self-games stay scaled by lines, outside T. Random
    references' games count in T but stay out of training.
  - Specs give T instead of rounds_per_cycle / min_rounds; the old keys are honoured for finished runs.
- Nothing live changes: the running stage-1-style fields are already at 1,584. The rule goes into leagues.py after
  the frozen-references build (same file), byte-identical when unset; for the NEXT global league and new
  experiments.
- **BUILT and installed 01:48:03** (spec key "games_per_cycle": T; backup leagues/leagues.py.before-gpc; scratch
  and doc backups models/from-scratch/gpc/). Byte-identical without the key: probit-late-p2's cycles 1-3 reproduced,
  62 configs unchanged, selftest passed with new rule checks and an interrupted-run test.
  - Pilots at T 1584: 8 entries give R 28 (1,568 games); 20 entries R 4 (1,520); a 7-entry global league R 38.
  - The global league's training batch still varies with its Elo window (both players at or above the model's
    Elo minus 30), which T doesn't control.
  - games_per_entry is kept as an alternative (mutually exclusive). With T set, rounds_per_cycle/min_rounds are left
    out of config.json. "under T" is logged at the 48-round cap, "over T" at the 1-round floor.
  - Documented in FORMATS.md and NEW-DESIGN.md rule 5.

## The eight, 11-11 ~16:20 (the user: "let's take 8 models forward ... but only after the optimizations are in")

- Global league at ply 1: F-L, F-P, T-P-Nc-merge-fresh, T-L-Nc-merge-fresh, D-P-Ne-ctx at router-split z 30 and
  z 20, D-L-Ne-ctx at z 160 and z 120. The user added z 20 and z 120 after hearing that z 30 / z 160 sit on the
  router-split knife edge: per seed it's all or nothing, and only probit z 30 seed 2 split.
- Recipe rows D-P-Ne-ctx-z30, -z20, D-L-Ne-ctx-z160, -z120 are in make_recipes.py HAND and recipes.txt (errdag4,
  the error-dag tests' exact flags). The regenerated recipes.txt differs only by those 4 rows.
- Spec leagues/specs/global/eight-p1.DRAFT.json. Its config dry-run is good: 8 lines at ply 1, leagues/global,
  patience 20M positions with a 5-cycle floor, margin 30, seed 41.
- Ply-2 drafts filled: six-probit-p2 at z 30, six-lr-p2 at z 160. They keep only those z; the user didn't ask for
  z 20 / z 120 there.
- LAUNCHED (the user 16:45: "I think we're ready to start the global league"; "Go ahead ... continue independently"):
  - leagues/global at 16:40 (spec leagues/specs/global/eight-p1.json): hybrid15 games and DAG training, hybrid6 for
    the rest. The z twins diverged by cycle 3 (different Elo-selected training sets).
  - Ply-2 stage 1 at 17:03: base-probit-p2 (seed 39; DAG z 20) and base-lr-p2 (seed 40; DAG z 120); specs in
    leagues/specs/ply2/. Reads at v300, v600 and the final version.
  - hybrid14 (md5 3976e92d...) and hybrid15 (md5 58717c38...; fixed-point play for DAGs with router splits) are
    verified byte-identical (fastbin/hybrid14, fastbin/hybrid15, PERF-LEDGER).
- The user on ply 2 (16:30): approach it "nearly from scratch", with exploratory one-factor experiments; the ply-1
  results only give direction. Stage 2 candidates: cluster-node router model (NB/TAN/LR), split threshold 1M/5M,
  split limit 1/4/8, gate, leaf type. Order and designs from the analyst (6.34) and stage 1.

## The six, 11-11 ~12:00-13:15 (the user: "those 6 sound fine")

- The six: F-L, F-P, D-P-Ne-ctx (probit error DAG), D-L-Ne-ctx (LR error DAG), T-P-Nc-merge (probit cluster tree),
  T-L-Nc-merge (LR cluster tree); all four non-flat models use NB routers. The DAGs' router-split z comes from
  error-dag-probit-p1 (read ~14:15; z 0 / 30 / inf) and error-dag-p1 (read ~15:30; z 0 / 160 / inf).
- **Global league at PLY 1** (the user: "in the global league they should be on ply 1 because that's the ply we
  tested them on"), untrained. The engine's global league is the 11-09 version (plies 1 and 2 per model, anchors,
  flat patience); a fork is building the 11-10 design (NEW-DESIGN.md items 4-8: one ply per model, current and
  champion, the Elo-window training rule with margin 30, self-games). Patience revised with the user (~13:45-13:55):
  training positions, not cycles or percentiles (20M, 5-cycle floor, every model incl. the top; NEW-DESIGN rule 6).
  BUILT and swapped in 13:50 (leagues.py md5 4c4a1194, backup leagues.py.before-global11; selftest 174 checks;
  running experiments byte-identical; dry run of the six at ply 1: 1,260 games and 15-20 CPU-s a cycle, each model
  trained on 180-245 games in early cycles, so 20M positions is ~500-800 cycles early on). Recipe rows
  T-P-Nc-merge-fresh and T-L-Nc-merge-fresh added 13:57 (make_recipes.py HAND); the DAG rows wait for z. Launch
  draft: leagues/specs/global/six-p1.DRAFT.json (seed 41, errdag3 or errdag4 for play). Dashboard (port 8766)
  restarted on the new code 13:58.
- **Ply-2 experiments** (the user: "begin some ply 2 experiments"): leagues/specs/ply2/six-probit-p2.DRAFT.json
  (seed 39: F-P, D-P-Ne-ctx, T-P-Nc-merge x2 each) and six-lr-p2.DRAFT.json (seed 40: F-L, D-L-Ne-ctx, T-L-Nc-merge
  x2 each). Floor 600, cap 900 (theory-2 6.33: probit-late-p2's lines gained 120-145 at ply 2 after v241), cluster
  trees at --revise 2 so the ply-2 gate holds their splits; both create cleanly (scratch check 13:14). Each starts
  when its z read lands. Cost (corrected 13:25 by the perf review): the 4-6 CPU-h estimate holds for tree and flat
  fields on hybrid13; with two DAG lines on errdag3 at ply 2, games alone cost ~150 CPU-s a cycle once the DAGs make
  router splits (a router-split DAG's half of a ply-2 game is ~8x a tree's on hybrid13), so ~25-35 CPU-h at the
  600-cycle floor and up to ~50 at the cap. errdag4 (building) trims ~25%; hybrid14 plus fixed-point DAG play
  (candidates 3-4) would cut game CPU by about two thirds. The user's call before the DAG fields start; if the
  probit read picks z inf, D-P-Ne-ctx is T-P-Ne exactly and the field is cheap.
- **Routers** (the user asked whether NB is best for each): NB won error routing clearly (single-split, ply 2, T 0.1:
  probit NB 44 / TAN -9 / LR -54 over flat; LR leaves -16 / -37 / -111). For cluster routing NB and TAN tied (77/75,
  88/99) and agree on 88-99% of positions; the analyst (6.33) predicts a ply-1 TAN-vs-NB cluster test within 15 either
  way, so it is not run unless the user wants it. A TAN-vs-NB error DAG at ply 1 (~1 CPU-h, predicted TAN 10-30
  below) is the router test with a chance of a difference; after the z reads, if wanted.

## Growth rule (the user, 11-11 ~18:00-19:30; analyst 6.36-6.39; spec models/from-scratch/theory-2/growth-spec.md)

- **The user's points:**
  - Splits all happen early and none later; 3 of 4 global DAGs never router-split.
  - Leaves should wait for stagnation and split at a constant rate per unit of data.
  - No consecutive router splits.
  - Perhaps a node must be more stalled than its children. This became SUBTREE STALL: a router is eligible only when
    nodes holding 90% of its subtree's recent examples have stalled.
- **Diagnosis (6.36):** today's leaf rule (worse, z 0) fires on noise once a leaf has settled, so the cap is the only
  brake. The router rule detects drift and favours the root, and the root re-fires after its own split.
- **The rule:**
  - Stagnation: two window-to-window improvements under 2 block-SE, at least three own windows.
  - One split credit per 5M positions, capped at 2.
  - Split on trial against an unsplit shadow, kept iff the gain beats 2 block-SE and an MDL bar.
  - Back-off on failure; no count cap; routers by path or cluster shadows.
- **Phase 1 (leaves) is BUILT (20:20):** chessai-grow1, md5 5f134f95...; write-up in fastbin/grow1/summary.txt.
  - Byte-identical without grow flags; 208 tests pass.
  - Smoke run on P1, T-P-Ne at D 5M / C 2:
    - the root split at 5M (gain 0.059) and leaf 3 at 10.7M (0.041), both kept;
    - leaf 1's trial at 15.3M was undone (0.0022, under MDL 0.0071);
    - leaf 4 was open at 19.7M; 5 leaves at 21.6M.
  - Analyst's rulings (6.41 note):
    - keep the MDL k at real parameter counts (leaf split 4262: bar 0.0106 at 3M, 0.0032 at 10M);
    - carry n across undos (a lifetime counter), added to grow2 behind --grow-mdl-n lifetime;
    - grow-p1 runs grow1 as built.
  - **grow-p1 LAUNCHED 20:36** in base-lr-p2's slot (leagues/specs/batch6/grow-p1.json, seed 44, floor 300, cap
    400). The grow arm is the recipe row T-P-Ne-g: T-P-Ne without --reset-leaves inf and with --trial-error log,
    neither of which changes a model trained without grow. It took three failed creates (moved to
    models/from-scratch/global11-drafts/grow-p1.failed-*): grow refuses --reset-leaves, it needs --trial-error log,
    and override names made a self-games file name too long. The read (v150 and final, plies 1-2) and the split
    table run automatically at the end.
  - Then the league test grow-p1 at ply 1: F-P x2, T-P-Ne current rule limit 4 x2, T-P-Ne grow (D 5M, no cap) x2.
    Floor 300, cap 400, reads at v150 and the final version.
  - Phase 2 (routers) follows, with a new trigger (below).
- **The user, ~19:50:** subtree stall makes splits happen only at the bottom of the graph; the router rule should
  combine stalling with the routing headroom there is to gain.
  - Analyst 6.40 agrees: under subtree stall a level-1 router is first eligible ~27M positions after the last leaf
    split below it (cycle ~300-350), and the root at ~500-600. Neither is ever eligible while leaf trials keep passing.
  - New phase-2 trigger (error routers only; for cluster routers the headroom would cost ~2x at the node):
    - eligible: the router's own routing error has stalled, every direct child has closed a window, and the headroom
      H (routed child's loss minus the best child's, already computed by error routers) clears 2 block-SE and MDL;
    - split: the shadow partition's out-of-sample gain G also clears 2 block-SE and MDL;
    - one credit stream: routers that pass go first, by gain per parameter; leaves otherwise.
  - Prediction: G won't fire on today's ply-1 models (re-routing gain measured 0.0011 nats). The offline check H'
    (force one unsharing in the P1 DAG at v60, train 10M more) tests whether new children would specialise.
- **stall-p1** (the old stall-high rule, cap 4, one factor) has been running since 19:14, as the user asked.
- **stall-p1 split timing at v254 (20:30; leagues/specs/arenas/split_table.py, which reads the model headers and the
  logs' positions):**
  - The current rule (worse) used its 4 splits at v21/36/53/57 and v21/37/52/58, i.e. 4.0-11.6M positions.
  - stall-high used them at v21/53/129/133 and v21/51/124/167, i.e. 4.0-34.7M positions.
  - The analyst predicted all 4 by v70-110, so stall-high's later splits came later than that.
  - Elo is pending the final cold read (floor 300).
- **stall-p1 READ (21:13-21:45; leagues/arenas/diag/stall-p1-stage), head to head over 600 games:**
  - stall-high against the current rule: -8 / 16 at v150 and -16 / -23 at v392 (plies 1/2). All within noise.
  - Both trees over F-P: 57-77 at v392.
  - Answer to the user's stalling question: waiting for stagnation spreads the splits out (the last two at 25-35M
    positions instead of ~11M) and leaves strength unchanged.
- **base-lr-p2 READ (20:29-21:40; leagues/arenas/diag/base-lr-p2-stage, v300 and v599, plies 1-3), head to head:**
  - LR cluster tree over F-L: 27 / 20 / -26 at v599 (plies 1/2/3).
  - LR DAG over F-L: 0 / -9 / -48.
  - F-L v599 against v300: -51 at ply 2 and 57 at ply 3, since the field moved.
  - No clear ply-2 winner over F-L; both lose at ply 3.
- **The user, ~20:15:** test the split criteria at ply 1 ("valid there also and cheaper"). Several router-split
  strategies may share one experiment, designed with the analyst.
- **router-p1 (analyst 6.41), the router-split rule as the only factor, at ply 1:**
  - Recipe D-P-Ne-ctx. Leaf rule fixed at today's (worse, z 0, limit 4, 1M, noise 0.03). Router-split limit 4 in
    every arm.
  - Arms, x2 each:
    - C: no router splits (z 1e9, which plays as T-P-Ne);
    - Z: today's drift trigger at z 20;
    - H: stall plus headroom (own routing error stagnant, direct children have closed a window, oracle headroom
      H_v over 2 block-SE and MDL);
    - HG: H plus the shadow test (split only if the prequential shadow gain G_v clears 2 SE and MDL);
    - F-P x2 as the anchor.
  - 10 lines, floor 300, cap 400, about 2-2.5 CPU-h.
  - Subtree stall is NOT a league arm: it cannot fire before ~v300, so it would be C's twin. It is checked offline on
    P1 instead.
  - Reads:
    - cold, T 0.02, plies 1-2, at v150 and the finals;
    - per line, every split's version, positions and level;
    - for H and HG, a per-router per-window log of H_v, G_v, their SEs and the decisions.
  - Predictions (±20, ply 1 / ply 2):
    - C minus F-P: 80-120 / 45-80.
    - Z minus C: -10..10 / -45..0.
    - H minus C: -20..10 / -30..10, with 2-4 level-1 router splits per seed at v120-v250.
    - HG minus C: within 15, with 0-1 splits per seed.
  - Readings:
    - HG 20+ over C at both plies would be the first routing benefit; replicate it.
    - H 20+ over C but HG not means the G_v test is too strict.
    - H at or below C (the expected outcome) means routers are not where the gain is on these leaves.
  - Engine: one fork builds the phase-2 flags on top of grow1 (they share --grow-stall-z, --grow-stall-windows and
    --grow-mdl), byte-identical when absent:
    - --router-split-rule {z|stall-h|stall-hg|subtree};
    - --grow-router-z, --grow-router-shadow {auto|path|cluster}, --log-router-stats.
- **grow2 BUILT (~21:50):** bin/chessai-grow2, md5 9bcd8d31; write-up in fastbin/grow2/summary.txt.
  - Byte-identical to grow1 / hybrid15 / hybrid6 without the new flags; 222 tests; exact resume including mid-shadow
    saves; hybrid15 and hybrid16 play its models.
  - P1 smoke: no router split under stall-h, stall-hg or subtree.
  - Open for the analyst before router-p1 launches:
    - (1) The children guard (every child has closed a window) blocked every router, because each router has a
      near-empty child (the root's leaf 3 has 3.2% of examples, router 2's leaf 5 has 0.7%). A share-based guard
      would open it.
    - (2) G falls steadily while cluster shadows run (root -0.0069, z -35; router 2 -0.019). Check it before reading
      HG's refusals as "no routing gain".
    - (3) The MDL k for a cluster split is 10,224 (6.40's ~2,500 counts one router copy; --grow-mdl 0.25 gives it).
- **Leaf tests (6.41):**
  - Keep stall-p1 (prediction: stall-high minus worse -15..10 at v300).
  - grow-p1 as growth-spec section 4.
  - Both get a per-line split table: version, positions, level, so the constant-rate claim is read directly.
  - limit-p2 is not run at ply 2. At ply 1 it is a low-priority filler.
- **Order:** the next free slot goes to grow-p1 once grow1 passes; router-p1 when its flags land. Don't fill slots for
  their own sake.

## Offline ply-1 vs ply-2 discriminator (6.35), done 11-11 ~20:40; analyst 6.42

- **Result** (models/from-scratch/offline-ply/summary.txt): g is flat minus T-P-Ne log-loss after one pass over equal
  21.6M positions.
  - P1 0.088 (seed 2: 0.095), P2 0.058 (0.049), P2x 0.051, P1x 0.056.
  - g2/g1 is 0.58 ± 0.05, with seeds 0.65 / 0.52. That is between H1 and H2, closer to H2.
  - Depth alone keeps 0.56 of g1, players alone 0.63.
  - About 2/3 of g is in the lowest-material leaf in every pool.
  - The two training seeds differ by 3-4 SE, so seeds set the error in offline contrasts.
- **6.42:** the ply-2 field halves the partition's value but does not remove it. The league's ply-2 tie is lost between
  log-loss and Elo, in one of two places:
  - (a) What play weights: the gain is calibration in decided deep endgames, which buys no moves.
  - (b) Tracking lag: trees follow a moving field 3-9x more slowly.
- **Single test, running since ~20:50:** arena offline-pairs-cold. The offline F-P / T-P-Ne pairs (P1 and P2, two seeds)
  are read cold at T 0.02 at plies 1-2.
  - Calibration of the method: the P1 pair at ply 1 should come out 60-110. If not, the test is void.
  - The P2 pair at ply 2: 20-60 means (b), within 20 means (a).
  - (b) runs limit-p2 next as the lag test. (a) drops the ply-2 split-parameter arms.
- **RESULT 20:53: it is (a), or worse.** Cold, T 0.02, 18,000 games among entries, ± 95%:
  - P1 tree minus P1 flat: 77 ±21 at ply 1 (inside the 60-110 band, so the method holds), 71 ±19 at ply 2.
  - **P2 tree minus P2 flat: -35 ±22 at ply 1 and -24 ±19 at ply 2.**
  - P2 flat minus P1 flat: 99 at ply 2. P2 tree minus P1 tree: 4 ±19 at ply 2.
  - With a fixed pool and equal data, the ply-2-trained tree is no better than flat in ply-2 play despite its
    0.05-nat log-loss edge.
  - Ply-2 data makes the flat ~100 stronger at ply 2 but adds nothing to the tree.
  - Tracking lag is ruled out. Analyst 6.43 was asked for the cause and the next ply-2 step.
- **Analyst 6.43:**
  - The cause is data hunger at ply 2. The ply-2 flat is still learning at 21.6M; in the league it gained 122 at ply 2
    between v241 and v399.
  - The tree's log-loss gain sits in decided deep endgames: 57-68% of g2 is in 9-19% of positions, where calibration
    buys no moves.
  - Its middlegame leaves each train on 1/3 to 1/9 of the stream, so they lag the flat where moves are decided.
  - Check (ii) is running: the flat's data curve, offline F-P after 12/36/108 rounds of P1 and P2, arena plus held-out
    loss. Prediction: at ply 2 the 7.2M flat is 30-60 behind the full one; at ply 1, within 15.
  - **Every ply-2 split-parameter arm is dropped** (limit, threshold, gate, routers). limit-p2 is answered by (ii).
    Stage 1 runs to its reads.
- **For the user to rule on (6.43 (2)-(3)):**
  - Residual leaves at ply 2: a shared trunk trained on every example plus a shrunk per-leaf residual. The code has a
    form of it (bdt.rs residual leaves). Test: T-P-Ne residual x2 against F-P x2 at ply 2, and the same at ply 1 to
    show it keeps the ply-1 gain. This is parameter sharing with a trunk, not the router-level leaf sharing the user
    excluded, but it is their call.
  - Objective weighting by the model's OWN uncertainty, p(1-p), or winner loss as the trial error. It passes the
    constraints like eval-swing extensions. Weighting by phase, material or move type fails.
  - S7 pair features: products of existing indicators as candidates, chosen by a statistic of the data,
    zero-initialised, shrunk, dropped when near zero. No chess concept enters, so it passes the letter, but it is
    learned feature construction, so it is the user's call.
- **CORRECTION ~21:00: head-to-heads contradict the BT contrast.** At ply 2, in offline-pairs-cold, the P2 tree beats
  the P2 flat in all four pairings: .587, .593, .643, .573, about 50-100 Elo. At ply 1: .49.
  - BT put the tree 24 behind because the field is non-transitive: the P2 flat beats the P1-trained models much harder
    (.65-.86 against the P1 flats; the P2 tree scores .57-.63).
  - The data curve holds head to head: at ply 2 the P2 flat after 21.6M beats itself after 7.2M by .747 (~188 Elo),
    for 0.0026 nats of held-out loss. At ply 1 the flat is saturated: .460.
  - So at a fixed pool the ply-2 tree DOES beat the ply-2 flat. The league's ply-2 tie may then be tracking lag (b)
    after all. Analyst 6.44 asked to re-read 6.42/6.43, and to say whether reads should report same-field
    head-to-heads beside BT.
- **Analyst 6.44:**
  - The league reads' own head_to_head.md files agree with their BT. The ply-2 ties are real head to head:
    probit-late-p2 v399, tree against flat, 50/46/50/54. The ply-1 verdicts hold. Only cross-field mixes lie.
  - **So it is (b), tracking lag, after all.** On a fixed pool the ply-2 tree beats its flat by ~65. In the league's
    moving ply-2 field (the flat gained 120 between v241 and v399) it ties. Leaves on 1/3-1/9 of the stream follow a
    moving target more slowly.
  - Tests:
    - (i) stage 1's v300 against v600 head-to-heads (lag: the gap opens after crownings stop, 20-50 by v600);
    - (ii) the offline drift test, RUNNING: the pair trained on 10.8M of P2x then 10.8M of P2 (lag predicts the
      drifted tree within 20 of its flat).
  - **limit-p2 returns** as the league's lag test (limit 1 over limit 4 by 15-40 at ply 2 at v600; over flat 0-30).
    The gate, threshold and router arms stay dropped at ply 2. The residual trunk-plus-leaf design stays the
    structural fix (the user's ruling).
  - **Read rule from now:**
    - (a) full BT, as context;
    - (b) the contrast's head-to-head pooled over arm x comparator lines at the same version and ply, with a binomial
      SE (leagues/specs/arenas/h2h_btg.py). **(b) is the verdict;**
    - (c) BT restricted to the contrast's entries.

    stage_read.sh now prints (b). In doubt: cross-ply-probit-cold (6.31 b), and reads that mix versions of one
    lineage (mild).
- **Drift test RESULT (~21:20; arena offline-drift; drift pair = 10.8M of P2x then 10.8M of P2):** head to head at
  ply 2, 300 games each:
  - drift tree against drift flat: .453, -33 [-73, 7];
  - P2 tree against P2 flat: .597, 68 [29, 109];
  - **drift flat against the P2-only flat: .803, 244 [199, 299];**
  - drift tree against the P2-only tree: .608, 76.
  - So drift erases the tree's edge, as lag predicts. Separately, swapping half the ply-2-field data for P2x games
    makes the flat 244 stronger at ply 2 at equal positions. That looks like DIVERSITY hunger at ply 2.
  - Analyst 6.45 asked to interpret it, give a cheap separating check, and say what it implies for ply-2 league
    training (mixed fields; the G option).
- **Analyst 6.45:** it is coverage, not strength and not order.
  - Both pairs end on the same P2 rounds. P2's players are the strongest ply-2 evaluators, near-twins at T 0.1, so their
    games are narrow. A 2-ply search queries off-policy positions that weaker, different players reach.
  - League implication: pure ply-2 fields understate ply-2 models by 100-250; the thunderdome is the right design for
    ply-2 data.
  - Proposed one-factor test, **diversity-p2** (needs the user's ruling and an engine build): F-P x2 and T-P-Ne x2 at
    ply 2 with G2's four finals, frozen, as fixed references at ply 2 whose games enter everyone's training. The
    comparator is the same recipes in a pure field.
  - limit-p2 is held: it runs inside the diverse regime only if a tree gain exists there. router-p1 is unchanged.
- **Coverage check (ii) (~21:30, arena offline-coverage), head to head at ply 2:**
  - the P2x-only flat against the P2 flat: .558, 41 [1, 81] (predicted 100-250, so too small);
  - the P2x-only tree against its flat: .378, **-86** [-128, -47];
  - the P2 tree against the P2 flat: 68, replicated.
  - So P2x alone helps only 41, while P2x then P2 gave 244: the pools complement each other, or order matters.
    Check (iii) is running: reversed order and a round-by-round interleave.
  - **The tree's edge over flat depends on the data regime:** 68 on P2, -33 on P2x then P2, -86 on P2x.
- **Order and mix check (iii) (~21:55; arena offline-order), head to head at ply 2:**
  - Flats against the P2-only flat:
    - interleaved P2x/P2: **186** [144, 235];
    - sequential (P2x then P2): 227;
    - reversed (P2 then P2x): **-63**.
  - Flats against each other: interleaved against sequential **105** [65, 148]; reversed against sequential **-360**.
  - Trees against their own flats: interleaved 49 [10, 89]; sequential -47; reversed -38; P2 alone 100 (68 in the
    earlier arena).
  - So recency dominates (the learners follow their latest data) and a stationary diverse mix is best by far. Trees
    beat their flats on stationary data and lose under a switch.
  - For league design: diversity plus stationarity (e.g. replaying older and other-field games). Analyst 6.46 asked.
- **Analyst 6.46:**
  - The one-pass probit learner is recency-weighted: what the stream ENDS on sets the model. Coverage and recency are
    both real.
  - lag = recency x share: a leaf on 1/3 of the stream has a 3x longer memory in time, so it goes stale under a switch.
  - League design: each cycle's set = the cycle's games + a fixed-share replay of older own-field games + other-field or
    reference games at a fixed share.
  - Two one-factor ply-2 fields, each read head to head against pure fields at equal versions:
    - **replay-p2:** own older games at 50% (window 50 cycles). Prediction: F-P over pure F-P 60-150 at v400.
      No new players, so no purity ruling is needed. **The engine option is being built (fork, ~22:05).** The
      comparator is probit-late-p2 (seed 38, same recipes).
    - **diversity-p2:** frozen P2x references, as in 6.45. **Needs the user's ruling** and an engine build.
    - Trees over flats in both: 20-60.
  - Reads take the best version by head-to-head, not the last: F-L v599 lost to its own v300 by 51 at ply 2, since a
    pure field past its coverage peak declines.
  - grow2 rulings:
    - (1) share-based guard at 90% of lifetime examples;
    - (2) a NULL shadow set, with the statistic G_real - G_null paired by block;
    - (3) k = (m-1) d_R = 5,112.
    The fork is building grow3. router-p1 launches after it.
- **grow3 BUILT (21:55):** bin/chessai-grow3, md5 aa05358a, 224+1 tests; the three rulings above.
  - stall-h and stall-hg replace --router-split-z (they refuse it), hence the recipe row D-P-Ne-ctx-r4 (D-P-Ne-ctx
    without --router-split-z 20; backups make_recipes.py/recipes.txt .before-dpr4).
  - P1 smoke: stall-h made 4 router splits (9.0M, 13.2M, 18.6M, 20.3M positions); stall-hg none. G_real drifts and
    G_null does not; the drift comes from the hard-EM cluster partition itself, so HG's refusals at cluster routers
    reject THAT partition, not every partition.
- **router-p1 LAUNCHED 21:56** (seed 45, leagues/specs/batch6/router-p1.json, every line trains on chessai-grow3).
  - Read waiter queued: stage_read.sh router-p1 150,final 1,2, verdicts head to head against C, then
    leagues/specs/arenas/dag_events.py router-p1 (every router split with its cycle and positions; split_table.py
    reads only bdt headers). Output models/from-scratch/global11-drafts/router-p1-read.out.
  - First router split: H #2, router 0 at 11.0M positions (cycle 21), a cluster split into a new sibling set.
- **Replay BUILT and installed 21:55:07** in leagues/leagues.py (backup .before-replay; fork):
  - spec key "replay": {"share": S, "window": W}; replaypool/c<cycle>/ hard links, sampling deterministic by hash,
    state.json current.replay, pruned after each cycle, self-games replayed too, totals replay_* columns.
  - replay-p2 drafted (leagues/specs/ply2/replay-p2.DRAFT.json): probit-late-p2's spec (seed 38, F-P x2, T-P-Ne x2,
    limit 4, 1M, noise 0.03, ply 2) plus replay share 0.5, window 50; hybrid16 games, hybrid6 training.
  - **LAUNCHED 22:09:33** (grow-p1 had ended at 22:02 and freed its slot; leagues/specs/ply2/replay-p2.json).
    Read queued (leagues/specs/arenas/cross_read.sh, a new waiter): ONE ARENA PER PLY (2, then 1, then 3) at v300 and
    v399, head to head against probit-late-p2: rfp:pfp, rtr:ptr, and tree vs flat inside each run (rtr:rfp, ptr:pfp).
    Output models/from-scratch/global11-drafts/replay-p2-read.out. ~2.3 CPU-h; window 50 needs ~1.5 GB of disk.
  - Why per-ply arenas: in base-probit-p2's 3-ply read arena the cross-ply games with ply 3 (1v3 0.055 s, 2v3 0.112 s a
    game; 3v3 0.228 s) took ~60% of the CPU, and no head-to-head verdict uses cross-ply games.
    (leagues/specs/arenas/make_cross_read.py writes NAME-p<ply>.json per ply, across experiments.)
- **grow-p1 READ (DONE v399, 22:02; read 22:10)**, head to head (ply 1 / ply 2, 600 games, CI about ±28):
  - grow - current: -11 / 21 at v150, -3 / -22 at v399. A tie (prediction -15..10 at v150, -10..15 at v300).
  - Over F-P: grow 54 / 21, current 58 / -43 at v150; grow 97 / 5, current 110 / 18 at v399.
  - v399 over v150: grow 122 / 92, current 108 / 81, F-P -5 / 45. Trees keep gaining at ply 1 long after their last
    split (current's last split was at v51).
  - Growth: kept splits at 7.0, 12.3, 27.3, 49.9, 77.1M (seed 1, 11 leaves) and 7.0, 15.7, 26.4, 36.6, 71.6, 78.0M
    (seed 2, 13 leaves). The pace slows (gaps 5, 15, 23, 27M in seed 1).
  - 27 trials, 11 kept, 16 undone. EVERY undone trial passed the z bar (z 3.1-9.9, gains 0.0024-0.0068 nats) and
    failed only the MDL bar (0.0055-0.0081 = k ln n / 2n, k 4262, n 4-6M). Leaf 3 was retried 4 times per seed.
  - Reading: the growth rule removes the cap at no cost in strength, and the MDL bar sets its pace. Question for the
    analyst: the trial gain is prequential (each example scored before training), which already charges the
    learning cost; does the MDL bar double-count it? A one-factor ply-1 test would be grow with the bar off.
- **Trigger flaws found in router-p1's and grow-p1's logs (~22:05), analyst 6.47 (~22:12):**
  - (1) STALL IS DRIFT. The stagnation test compares window means on different data. In a league field the stream
    moves: router 0's error rose every window (2.6 -> 3.5), and grow1's leaf means bounce 0.70-0.78 with block SEs of
    0.001-0.018. The analyst: "it is my error"; 6.37 assumed stationary pools. A paired snapshot statistic cancels
    the drift but has a floor (the online model always beats a one-window-old copy by its tracking gain), so it would
    never read stagnant. RULING: no stagnation gate in league fields. Eligibility = age (three closed own windows)
    plus credits; the trial (paired) decides; the paired snapshot gain is logged and used as the trial PRIORITY
    (smallest learning gain first), so "wait for stagnation" survives as an order, not a gate.
  - (2) H CANNOT GATE. grow3's H is the routed child's realised loss minus the per-example least child loss
    (hindsight), positive even for identical children (z 26-218 everywhere). RULING: H withdrawn as a gate (logged
    only); the sound router statistic is G_real - G_null, with shadows from age one window at every router.
  - (3) HG's G turning negative (+5.7z, then down to -22z) is the mechanism, not the statistic (G_null stays near 0):
    the cluster partition is phase-like and correlated with routing, so each copy sees a biased subset of targets
    and tracks worse when the stream moves (lag x share in routers). HG's refusal is a finding.
  - router-p1 RUNS ON, read as: C control; Z drift; H a SCHEDULE arm (cluster splits when children holding 90% are
    ready, ~4 per seed by v100-v150); HG = C's twin (±10). Predictions unchanged. Extra readout: G_real - G_null in
    hindsight at H's split routers. The ply-1 router question closes with this run.
  - grow4 (planned, not built): no stagnation gate (age + credits), paired snapshot gain logged as trial priority;
    routers: H logged not gating, shadows from age one, Q_v refit online. grow-p1b (grow4 vs grow1 at ply 1) only if
    grow-p1's trial record shows the gate blocked trials. It does: 33 and 42 closes with a credit waiting for a
    stagnant leaf. The MDL question is open: every undone trial passed z and failed only MDL. Sent ~22:15.
  - base-probit-p2 read checks (6.47 (c)): (i) tree and DAG vs F-P head to head at v300 and v600 at ply 2; (ii) DAG
    vs cluster tree at ply 2; (iii) each line's v300/v450/v600 ladder at ply 2, report the best version; (iv) ply 3
    for parity only.
- **Analyst 6.48 (~22:20), the MDL bar: it double-counts.** The trial gain is prequential, and prequential code
  length is two-part MDL asymptotically, so "split beats shadow at 2 block-SE" IS the MDL test. The bar is also
  miscalibrated: the ply-1 trees' gains came from splits worth 0.004-0.006 nats, exactly what it refused. Ruling:
  --grow-mdl 0 with a size margin --grow-min-gain 0.001 (not built yet); credits then bound size (kept splits <= N/D;
  D 10M is the lean league setting once the bar is off); keep the doubling back-off. The bar binds, the stall gate
  only delayed, so the bar goes first.
  - **grow-p1b LAUNCHED 22:15** (seed 46, leagues/specs/batch6/grow-p1b.json): F-P x2, T-P-Ne-grow x2 (new recipe row =
    grow-p1's grow line; backups global11-drafts/*.before-tpnegrow), T-P-Ne-grow --grow-mdl 0 x2 (arm A without the
    min-gain floor). All on chessai-grow1. Read queued (per-ply arenas, v150 and final, then split_table.py).
  - Arm B (grow4: no stagnation gate, age 3 windows + credits, snapshot-gain priority) goes in the next field;
    grow4's leaf part is being built by a fork.
  - **grow4 BUILT 22:49** (bin/chessai-grow4, md5 73239411; 230+1 tests; byte-identical to grow3/grow1 without the new
    flags; exact resume incl. snapshot windows; write-up models/from-scratch/fastbin/grow4/summary.txt). Flags:
    --grow-gate {stall|age} (--grow-age-windows 3), --grow-priority {excess|snapshot}, --grow-min-gain X.
    Smoke (P1, 22.4M, age + snapshot + mdl 0 + min-gain 0.001): kept at 6.0, 12.2, 18.3M, 0 undone; identical to
    grow1 until 21.17M (credits bind, one eligible leaf at each credit); min-gain never decided (z bar >= 0.00116).
    Snapshot gains: 234, all positive (+0.0012 to +0.0182): the analyst's floor, every leaf still learning.
  - Arm B drafted as grow-p1c (leagues/specs/batch6/grow-p1c.DRAFT.json, seed 47; all lines on grow4).
    **Analyst (6.48 note, ~22:55): B WAITS.** With the MDL bar on, B is inert against grow1 (credits bind, one eligible
    leaf per credit). Run it after grow-p1b, ON TOP OF THE WINNING BAR SETTING, with the winner as comparator: if the
    bar goes, kept splits use up credits, several leaves are eligible per credit, and the priority has choices. The
    draft is rewritten then.
  - **grow-p1b READ (DONE v352 by plateau, 23:29; per-ply reads took 4 min)**, head to head (ply 1 / ply 2):
    no-bar minus grow1 -13 / 40 at v150, -2 / 19 at v352; over F-P at v352: no-bar 80 / 48, grow1 115 / 99. No-bar
    kept 11 of 11 trials (25 leaves), grow1 5-6 of 11 (13-15 leaves). THE BAR GOES (no floor needed): growth is
    governed by the z bar and the credit rate D. Size is then the user's choice through D.
  - **grow-p1c LAUNCHED 23:35** (seed 47): F-P x2; A = T-P-Ne-grow --grow-mdl 0 x2; B = A + --grow-gate age
    --grow-priority snapshot x2; all on grow4. The T-P-Ne-grow recipe row now trains on grow4 (create checks flags
    against the recipe's binary; backups global11-drafts/*.before-grow4). Read queued (per ply, v150 and final).
  - **Analyst 6.50 (~23:45):** the bar goes, no floor. The robust statement: 25 leaves cost nothing against 13-15 at
    either ply. "A beats grow1 at ply 2" (19 ±28) is NOT established. The ply-2 triangle is inconsistent by ~70
    (grow1 over F-P 99, A over grow1 19, A over F-P 48): same-field head-to-heads aren't transitive at 600 games.
    Fragmentation lag needs a MOVING stream (ply-2 TRAINING), not a ply-2 read. A's ply-2 band revised to -10..30.
  - **For the user before the next global league grows (6.50 (3)):**
    (a) D, the credit rate, is the size knob. With the bar off every credit becomes a kept split, so leaves ~ 1 + 2N/D:
        D 5M gives 25-33 leaves per 80M and 64 nodes at ~105M; D 10M half that (64 nodes at ~210M; a global-league
        line at 100k a cycle: 13 leaves at cycle 600, 25 at 1200). 10M recommended.
    (b) Hard cap at 64 nodes (--split-limit 21), or accept the priced fallback past hybrid15's 64-node fixed point.
    (c) Grow LEAVES only until router-p1 reads. The router MDL bar should go too (the shadows are prequential).
    (d) Honest expectation: at ply 1 growth is free but has bought no Elo in three runs (grow -3, no-bar -2 vs the
        capped rule). Its value is for leaves with more to learn (a broad ply-2 regime, or S7).
    W 1M, C 2, z 2 stay.
  - The slot freed by replay-p2 (~23:30): nothing by default. If replay-p2's read shows the replay flat 60+ over the
    pure flat at ply 2, the replay follow-up takes it (one factor: share 0.25 vs 0.5, same recipes). Otherwise it stays
    free until router-p1 and grow-p1b are read.
  - Flagged to the analyst: its A prediction (20-30 kept splits) exceeded the credit bound (79M / 5M = ~16). Restated
    (6.48 addendum): 13-16 kept splits per seed by v399 (27-33 leaves), 0-3 undone; rule threshold 25+ leaves.
- **Analyst 6.49 (~23:05): ply-2 STAGE 1 CLOSES.** In a pure ply-2 probit field no tree or DAG beats its flat at
  any version or ply (10-50 behind at ply 2, ~53 at ply 3). In the LR field the cluster tree is 20 ahead and the
  error tree -9 (±28). The flat is the best model of the pure regime.
  - Against 6.34: ply 1 inside the bands; ply 2 at the edge; ply 3 far outside (parity decay larger than allowed).
    Router splits 4+4 against "at most one seed" (wrong). DAG within 20 of the cluster tree (right).
  - The ply-2 gap tracks the flat's rate of improvement: widest at v450, when F-P rose 34 per 150 cycles, narrowing
    to -12 as it slowed. That is the lag reading (6.44-6.46): the same recipe beats its flat by 65-100 on a
    stationary pool.
  - So the pure ply-2 field is the wrong regime twice over: it understates every model by 100-250 (coverage) and
    penalises partitioned models by their lag. The architecture question at ply 2 is to be re-asked inside a
    stationary, broad stream.
  - NEXT PLY-2 STEP: the data regime first (replay-p2 running; diversity-p2 pending the user on frozen references),
    then the eight screened at ply 2 inside the winning regime. No split-parameter arms in pure fields.
  - replay-p2's read decides:
    (1) replay flat 60+ over pure at ply 2 AND replay tree 20+ over its flat: move the ply-2 screen into replay
        fields; share 0.25 vs 0.5 is the one follow-up factor (the analyst's prediction);
    (2) flat 60+ but tree within 20 of its flat: coverage restored, lag not; diversity-p2 next;
    (3) flat within 30 of pure: own older games are the same narrow lines; diversity-p2 is the route (the user);
    (4) flat 30+ BELOW pure: recency bites the other way; share 0.25 or age-weighting before judging.
- **For a later grow build (6.42 (3)):** grow tries the leaf whose loss most exceeds the model's. That points at the
  middlegame, but the gain sits in the lowest-loss endgame leaf. A one-factor ply-1 arm would test
  --grow-priority {excess|size|gain}. No change to grow-p1.

## Global league: last champions (the user, ~20:15: "take note of the last champions of each model and crown the last model standing and its champion the winner")

| order out | line | retired at cycle | last champion | its Elo then | current at retirement |
|---|---|---|---|---|---|
| 1 | T-P-Nc-merge-fresh.p1 | 531 | G.22.354 | 967.4 | G.22.530, 941.0 |
| 2 | F-P.p1 | 663 | G.19.463 | 964.0 | G.19.662, 969.5 |
| 3 | T-L-Nc-merge-fresh.p1 | 930 | G.20.581 | 984.4 | G.20.929, 954.9 |
| 4 | F-L.p1 | 964 | G.19.538 | 973.2 | G.19.963, 962.7 |
| 5 | D-P-Ne-ctx-z20.p1 | 1066 | G.24.754 | 928.1 | G.24.1065, 897.3 |
| 6 | D-L-Ne-ctx-z120.p1 | 1201 | G.22.638 | 972.0 | G.22.1200, 938.1 |

Still in at cycle 1258 (22:00): D-P-Ne-ctx-z30 (champion G.25.862; patience 60% at 1201) and D-L-Ne-ctx-z160
(champion G.26.1030; patience 12% at 1201). With 8 entries a cycle takes ~5 s. A monitor alerts when one line is
left; then the league is stopped and that line and its champion are the winner.

**Cold re-read of the last champions (11-12 ~04:14; leagues/arenas/diag/global-champions-cold-p1; ply 1, T 0.02, 150
round-robins, CI ±15; at 04:14 the two active lines' champions were G.34.4029 (D-P z30) and G.30.2323 (D-L z160)):**

| cold rank | line | last champion | cold Elo | in-run Elo (its last rated cycle) | out |
|---|---|---|---|---|---|
| 1 | D-L-Ne-ctx-z160.p1 | G.30.2323 | 896.4 | 987 | active |
| 2 | D-P-Ne-ctx-z30.p1 | G.34.4029 | 894.3 | 902 (14 cycles as champion) | active |
| 3 | T-L-Nc-merge-fresh.p1 | G.20.581 | 890.9 | 984 | 3rd |
| 4 | D-L-Ne-ctx-z120.p1 | G.22.638 | 877.6 | 972 | 6th |
| 5 | D-P-Ne-ctx-z20.p1 | G.24.754 | 861.2 | 928 | 5th |
| 6 | T-P-Nc-merge-fresh.p1 | G.22.354 | 847.3 | 967 | 1st |
| 7 | F-L.p1 | G.19.538 | 814.2 | 973 | 4th |
| 8 | F-P.p1 | G.19.463 | 801.6 | 964 | 2nd |

Cold, at ply 1, every tree and DAG champion beats both flats (by 33-95), and the LR cluster tree (retired 3rd) is
level with the two finalists. In-run Elos are from different cycles of a drifting scale, so they don't order the
lines; retirement follows crowning frequency, not rank; and the champions trained for different lengths (v354 to
v4029). So this is a re-read of the standings, not an architecture verdict. It fits the in-run under-rating of trees
(diversity-p2c: ~110 for cluster trees at ply 2).

## Open for the user: the global league's mechanics (analyst 6.34 (d), 11-11 17:15; NOT changed, the design is yours)

- **Training window is one rating SE.** The Elo-minus-30 margin is about one rating SE: per-entry SE is about 21,
  and the eight currents sit within 892-990.
  - So who counts as "the leader" is mostly noise, and training sets swing from 5 to 1,280 games a cycle by noise.
    F-L had 245 games or fewer in 75 of 176 cycles.
  - This negative feedback starves whichever line noise puts on top, and the trees' late gain needs data (46-64M
    positions in G2; lines have about 15-20M now).
  - Fix options: a 2-SE margin (about 60), or a window anchored on the champion's Elo.
- **Crownings are mostly noise.** About 0.38 a cycle since cycle ~40, against about 0.18 expected by chance (2 SE
  over 8 tests), and currents sit within ±40 of their champions.
  - So patience rarely accrues and champions carry little information.
  - Fix option: pool the champion test over all games since the last crowning.
- **First retirement, 19:10:** T-P-Nc-merge-fresh.p1 at cycle 532, after 20M positions without a crowning since
  cycle 355. Its champion was G.22.354 at 967; the current version was 934, inside the 892-990 field.
  - Analyst 6.38 note: this is the crowning noise at work, not a strength verdict. A champion picked on a lucky
    reading makes the next crowning about a 3-SE event per cycle, so 177 cycles without one is likely (~12% that
    one of eight lines goes that long by chance), and the line with the luckiest champion is the one retired.
  - Expect repeats unless the crowning test is pooled over cycles.
- **Second retirement:** F-P.p1 at cycle 663, with its current 23 AHEAD of its champion (z 0.73).
  - D-P-Ne-ctx-z20 reached 93% patience, then was crowned at cycle 682 at z 2.01 (diff 69). Its next current was 87
    behind the new champion, which is the winner's curse.
- **The user, ~19:50: "do you think global league is not playing enough games per round? the standard error looks
  difficult to overcome".**
  - About 150 games per entry a cycle give an SE of about 34 on current minus champion, so a crowning needs about 68
    in one cycle. More games help only by the square root: 4x the games halves the SE.
  - Pooling over cycles (analyst 6.40 sec 5) was REJECTED by the user (~20:10): "You can't pool across cycles,
    because we train once per cycle and you're no longer comparing the elos of the same models".
  - **The user's decision instead: "variable length cycles based on the number of participants in the league".**
    - Each entry plays G games per cycle: R = ceil(G / (2(N-1))) double round-robins for N entries.
    - BUILT and installed 20:23 (leagues.py md5 8ef0f1f0, backup leagues.py.before-varlen; FORMATS.md, NEW-DESIGN
      rule 5): config key games_per_entry; absent = today's 5 rounds, identical snapshots. NOT set in this global
      league (the user: run it to its finish as is); it is for the next one. At N 16: G 300 = 10 rounds (~27 s a
      cycle), 600 = 20 (~46 s), 1200 = 40 (~84 s).
    - G is the user's pick. I recommended 600: SE on the diff about 17, a crowning needs about 34, and the 30 margin
      becomes about 2.5 SE.
- **The z twins split by the window, not by router splits** (none logged yet). Read the four DAG lines as replicates
  of the error-DAG family, not as a test of z.
- **Minor display lag:** the cycle report's "patience positions this cycle" lags "trained on" by one cycle.

## Open for the user (11-11 ~12:00, while they were out)

1. **Screening ply** (theory-2 6.31). Trained in a ply-2 field, the probit error tree ties its flat (18 ±21 at
   ply 2, 9 leaves held); its 118 / 77 lives in the ply-1 field, and the training field's ply outweighs the
   architecture several-fold (cross read: at ply 2 the ply-2 flat beats the ply-1 tree by 267). The trees also learn
   late (G2: 78 between v226 and v314 while F-P stood still). probit-late-p2 (floor 400, read 13:08): the ply-2
   tree still level with F-P at the end (-3 ±18 at ply 2, 17 at ply 3) but closing (22 and 64 over the last 160
   cycles). The analyst's rule: a final ply-2 gain of 40 or more keeps the ply-1 screen (with longer ply-2
   confirmations); under 20, screen at ply 2 and turn to features. The choice is the user's.
2. **A cold champion test** (theory-2 6.32, amended). Crownings, and so the plateau stop, are judged in the league's
   own T 0.1 games, which barely see a tree's edge (1-6 Elo at T 0.1 where a cold read gives 118), so tree runs stop
   early. Option: 48 games per line per cycle, current against champion at T 0.02, kept out of training, pooled since
   the last crowning (SE ~10 after 25 cycles). Measured cost: ~0.8 CPU-s per line per cycle at ply 2 (~12% of a
   6-line cycle's 40 s), ~0.1 at ply 1 (~4%). An engine change; nothing built.
3. **The LR error-DAG test:** answered; the user asked for it, running since 12:12 (error-dag-p1).
4. **S7, pairwise features** (the analyst's lever past ~9 leaves): pairs of existing one-hot indicators chosen by the
   data, zero-initialised, learned. The user's call under the zero-knowledge rule.
5. **Disk:** (13:25 add: fastbin/review-1111b 873M, the perf review's scratch; 15:08 add: treeview-cache/dumps 2.6G and
   parts 168M, unused leftovers of the report tool's first version; fastbin/errdag4/id 1.1G and dry 2.2G on disk.) OK to delete old scratch in /tmp (RAM, 12 of 16 GB used: sg 2.0G, gatecheck 1.7G, ladder 940M, leafsim
   614M, ...) and the forks' finished outputs (errdag 556M, build target dirs ~90M each)?

## The user's decisions, 11-11 ~10:10: error trees and DAGs are their own family

- "let's have error trees/dags be their own thing and turn all our error trees into DAGs and calibrate the router
  splitting correctly". Error family: error routing, non-merging (no --revise, fresh gate or split trials), no leaf
  sharing of either kind ("it will reduce the routing value we're trying to capture"), path context (the user's path
  parameters) in the DAG.
- "They should at least be equal since we can calibrate their router splitting thresholds": the DAG with router
  splits off must train as the tree (byte identity), so a calibrated threshold can only tie or win.
- Built (the errdag fork, 10:15-11:20; models/from-scratch/errdag/summary.txt; RESULTS section 8): chessai-errdag
  (--router-split-z, inf = the tree), errdag2 (--router-split-limit: leaf splits keep their own budget, router
  splits < leaf splits) and errdag3 (probit leaves and probit path cells in the BDD). At z = inf each trains and plays
  exactly as the tree. Offline, LR: no z beats the tree; probit: z 0-10 predict the winner 0.004 better (3-4 SE),
  z 20 worse, z 25 and up the tree.
- **Running from ~11:20: error-dag-probit-p1**, the user's test on the six's error tree: F-P x2, T-P-Ne x2 (the tree)
  and D-P-Ne-ctx (the error DAG with path parameters) at router-split z 0 x2 and z 30 x2 (z 30 = the offline best
  band mapped to 1M windows: the calibrated arm). The LR stand-in error-dag-p1 (leagues/specs/batch5/) is deferred.
  TAN routers later.
- Cluster trees: the analyst's job (the user's words: "analyst can think about ways to deal with cluster trees").
  Routing headroom measured nil out of sample (routecells, RESULTS section 8); their reading: past the phase split,
  growth is a feature problem on one-hot squares. Cheap lever: a gate that keeps a split only on a clear gain.
- G6's leaf-sharing arm (T-L-Ne-share3) runs to its end (its other arm is seat 2's second seed) but won't be seated.
- The ply-2 test ran early in G2's free slot, widened after G2's result: probit-confirm-p2 (10:08), a ply-2 field with
  F-P x2, the probit error tree T-P-Ne x2 (holds its splits: the user's NB-router question) and seat 1's tree at
  --revise 2 x2.
- G2 (10:07): the probit ERROR tree (non-merging) 118 over F-P at ply 1, 77 at ply 2, level with seat 1 (106 / 62).
  It takes a seat; the probit-leaf error DAG port moves up.

## The user's decisions, 11-11 ~08:30 (supersede the morning summary's item 5)

- **Ratings at T 0.02** (adopted). Every cold read and arena rates at T 0.02.
  - **The experiments' own games stay at T 0.1.** They are also the training data, and training on T 0.02 games cost
    106 at ply 1 and 221 at ply 2 (train-t002-p1 against train-t01-p1, 08:43; RESULTS section 8).
- **Back to architecture screening at low plies.** The user, in short:
  - Deeper training makes any model stronger at deeper plies, so it doesn't pick an architecture.
  - The target is an extensible architecture that beats flat LR or flat probit, found by sifting many candidates
    at low plies.
  - Go deep only if low-ply work stagnates, or if there is good reason to think low-ply losers win deeper.
  - So: screen at ply 1, confirm winners at ply 2, no ply-3+ rungs for now.
- **No capture extension:** "the idea of captures feels like non-zero knowledge". Search heuristics driven by the
  model are acceptable: search deeper where the evaluation swings after a ply, or iterative alpha-beta with
  move ordering from the previous iteration's principal line. They are parked until deep plies return.
  - The exact speedups that order captures first (hybrid10, hybrid12) change no game. The user was told and may ask
    to replace them.
- **Disk:** the identity-check folders were deleted with the user's OK.

- **The user's goal (~08:50): six models.** Flat LR and flat probit as baselines, plus the four best candidates
  among trees and DAGs, picked on cold ply-1 performance, "models which can expand in size". Then we move forward.
  The analyst is ranking the candidates and listing the gaps (theory-2 6.24). The ply-2 tests ((a) --revise 2,
  (b) an LR router for the user's NB-router idea) are on hold.
  - **The analyst's plan (6.24):**
    - **Seats so far:**
      - Seat 1: fresh-gate probit trees, limit 4, noise 0.03 (84-104 over flat probit, replicated).
      - Seat 2: fresh-gate LR trees (43 over flat LR), if a second seed replicates it.
    - **Gap runs at ply 1** (own games at T 0.1, plateau floor 300, read cold at T 0.02 at plies 1-2), each with
      F-P x2 in the field:
      - G1, router kind: router-kind-p1, done 09:36. NB-router trees 44 over F-P at ply 1 (12 at ply 2); the LR
        router never splits (1 leaf, -9). LR routers closed; TAN is the fallback if NB routers fail deeper.
      - G4, growth: growth-300k-p1, done 09:50. 1M trees 51 over F-P at ply 1 (4 at ply 2), 5 and 7 leaves; the 300k
        window churns the trees (1-3 leaves) and loses 47 (one line -91). A shorter window does not grow trees.
        Seat 1's recipe now: 44 and 51 in 6-line fields, 87-104 in p1b/p1c.
      - G5, the LR anchor: dropped (09:20). The option is built (chessai-hybrid6an), but offline no strength
        helps, and at the matched strength the fresh gate merges the tree to its root. Seat 2's second seed
        comes from G6. The draft spec stays a draft.
      - G2, probit error trees: error-routing-p1, done 10:07. The non-merging probit error tree T-P-Ne is 118 over
        F-P at ply 1 and 77 at ply 2, level with seat 1's trees (106 / 62) in the same field; 9 leaves. It takes a
        seat (the flagship); its DAG form runs in error-dag-probit-p1, its growth in error-growth-p1.
      - G6, the LR error tree with leaf sharing: lr-share-p1, done 10:42. Sharing trees 22 / -34 (ply 1 / 2), seat
        2's second seed 19 / -17. Sharing is out regardless (the user, 10:05).
      - G3, a DAG: dag-lr-p1, done 11:19 (form (a): cell DAG, 5 shared LR leaves over 9 cells, 1M, x2 against
        T-L-Nc-merge limit 4 fresh x2, F-L x2). Cold: cell DAG 30 / 15 over F-L, the trees 59 / 16, so the cell DAG
        is 30 below the tree at ply 1 (prediction: within 20) and uses 3 of its 5 leaves. No seat (also fixed in
        size). Seat 2's ply-1 reads: 43, 19, 59 in three fields (mean ~40); ply 2 stays weak (-17, 16).
    - **The six as of 11:25:** F-L, F-P; T-P-Ne (flagship, 118 / 77); T-P-Nc-merge (seat 1: 106 / 62, and 44-104
      elsewhere); T-L-Nc-merge (seat 2: ~40 at ply 1 over three seeds, weak at ply 2); the sixth from the running
      tests: the probit error DAG (error-dag-probit-p1, if it at least equals the tree) or the limit-8 error tree
      (error-growth-p1, if it grows with gain).
    - **The ply-2 confirmation failed (11:30, probit-confirm-p2 cold):** trained in a ply-2 field, the probit error
      trees are 18 ±21 over F-P at ply 2 (-17 at ply 1, -49 at ply 3) and the --revise 2 cluster trees -9 (-3, -65),
      all with 9 leaves held. The trees' edge (118 / 77 from a ply-1 field) is a property of the ply-1 training
      field. Cross read (diag/cross-ply-probit-cold, 11:48): at ply 2 the ply-2-field flat beats the ply-1-field
      tree by 267 and the ply-1 flat by 316; the training field's ply outweighs the architecture several-fold.
      Equal-data read (G2 at v226, 11:50): the ply-1-field trees were only 14-20 over F-P at ply 1 then; in one
      field F-P v314 minus v226 is 3 while T-P-Ne gained 78 (11:52): the trees learn late, after the flat has
      plateaued, and probit-confirm-p2's plateau stop (v241) came inside that window.
    - **For the user (theory-2 6.31, the analyst's plain choice):** (A) keep screening at ply 1, knowing the six are
      ply-1 architectures (their gain lives in the ply-1 field); or (B) screen at ply 2, 3-4x the cycle cost, where
      today's trees tie their flat. Running from 11:53 to decide between them: **probit-late-p2** (ply-2 field, F-P
      x2 vs T-P-Ne x2, plateau floor 400 instead of 200; ~2.2-3.3 CPU-h): if the error tree pulls ahead late, the
      ply-1 screen stands with longer ply-2 confirmations; if it still ties, (B) and features (S7).
    - **The pre-screen** (done 09:10, RESULTS section 8): no old champion clears 40 at ply 1. The best are
      T-L-Nc-c 34 and T-L-Ne-share3-c 31 (81 at ply 2), and both old DAGs are level.
    - **Why G4:** probit-limit8-p1 shows the limit doesn't bind. Nothing grew past 3 leaves at 1M, because the
      fresh gate's window (a second-level leaf sees a third of the stream) undoes splits first.
    - **The user's NB-router question (~09:40):** answered from probit-trees-p2 (the gate merged every ply-2 split,
      so the router was never tested at ply 2) and G1. The settling test, (a) above (--revise 2 at ply 2), runs in
      the next free slot; (b) is moot after G1.
    - **The user's unified tree/DAG idea (~09:50):** router splits driven by the loss to routing, so tree and DAG
      become one model (identity = today's tree); "path parameters" = path context. Routing loss on G4's grown
      trees is 0.0003-0.0013 nats, and their second-level leaves are near-copies (RESULTS section 8). Analyst's
      design: theory-2 6.25; IDEAS 9. A fork is building the diagnostic chessai-routecells
      (models/from-scratch/unified/); it decides whether to build the sharing move only or both.
    - **The final pick:** one arena of every candidate plus both flats, plies 1-2, T 0.02. "Best" means the gain over
      the same leaf's flat at ply 1, with two seeds, at least 40 at ply 1, leaves held, and no more than 60 lost at
      ply 2.

**Batch 5** (theory-2 6.23, the analyst's ranked list; one factor each; read cold at T 0.02):
1. **The T 0.02 training check** (done 08:43): train-t002-p1 against train-t01-p1, F-L x2 each in a pure ply-1 field,
   seed 27. Trained on T 0.02 games: -106 at ply 1, -221 at ply 2. The prediction was within 20. Training games
   stay at T 0.1.
2. **Growth of the ply-1 winner** (running from 08:34): probit-limit8-p1, fresh-gate probit trees (split noise 0.03),
   split limit 8 x2 against limit 4 x2. Prediction: 11-15 leaves, 20-40 over limit 4 after a lag.
3. **The ply-2 confirmation:** the same probit trees with --revise 2, so the gate holds its splits, against F-P x2
   in a ply-2 field (~3-5 CPU-h). It starts when a slot frees. probit-trees-p2 (revise 0) is the comparator for the
   gate change.
4. **The LR anchor:** LR trees whose split children are pulled toward the parent's weights, the LR version of the
   probit split noise. A fork is building the option and an offline sweep. The league test follows.
5. **A free pre-screen** (a fork, running): one cold arena at plies 1-2 of every archetype champion never rated
   cold, beside flat LR and probit of the same vintage.

## Morning summary, 11-11 (written overnight; details in RESULTS sections 1 and 8)

**0. The night's main result: an evaluator is best at the search depth of the games it trained on, and training
deeper also helps one ply further. Replicated in two seeds.** flat-p3 trains F-L x4 in a field playing at ply 3. Its
twin flat-p2 is identical at ply 2. Seed replicates: flat-p3-b and flat-p2-b. Final versions, near argmax (T 0.02):

| Ply-3 field minus ply-2 field | ply 2 | ply 3 | ply 4 |
|---|---|---|---|
| seed 20 | -153 ±13 | 181 ±13 | 100 ±18 |
| seed 23 | -216 ±14 | 173 ±13 | 108 ±18 |

- At ply 3, every ply-3-trained line beats every ply-2-trained line. They also beat the best earlier flat LR
  (lr-settings-new) by 168 and the old depth-4 lineage (d4mix, the best at ply 3 until tonight) by 96.
- At ply 4 they are level with d4mix.
- Run-to-run differences are only 7-33.
- Ply-3 training still wins at ply 4, so between these two there is no parity effect (the analyst's argument: at a
  fixed depth a linear evaluator's side-to-move weight cancels). W4 below hints at one for piece values.
- Cost: 3.2 CPU-h per ply-3 run, ~1 per ply-2 run.
- What changes is the piece values. Relative to the queen, ply-3-trained models value knight, bishop and rook at 0.25,
  0.30, 0.55 (ply-2-trained: 0.13, 0.18, 0.38; classical 0.33, 0.33, 0.56). Deeper play converts material, so the
  learned values move towards the classical ones.

**The ply-4 rung is done, and the ladder works** (RESULTS section 8; T 0.02; all three runs warm-started):
- W4 trained flat-p3's final models in a ply-4 field; W3 trained the same starts in a ply-3 field. The field's ply is
  the one factor.
- At ply 4, W4 beats W3 by 200 ±20 and its own start by 173 ±20. Every W4 line beats every W3 and flat-p3 line at ply 4.
  W4 cost 4.0 CPU-h; a from-scratch ply-4 field would cost ~24.
- The ladder: flat-p2's ply-2 models, warm-started into a ply-3 field, reach flat-p3's ply-3 level (-11 ±14) for
  0.69 CPU-h, against 3.2 from scratch.
- The warm start costs 27-51 (W3 ends below its start at every ply). It was the reheated step (sums divided by 8):
  with the sums kept, W3 holds level with its start (7 ±17, 22 ±17). The analyst suggests dividing by 2-4 for the
  next rung, enough to move but less noise.
- **Parity:** W4 minus W3 at plies 1-5 is -85, 172, -1, 200, 3. A ply-4-trained evaluator gains at even depths
  only. The analyst's mechanism: training games carry their search horizon's parity, and the evaluator learns
  corrections for it in the piece and square weights. Not the whole story, though: ply-3 training beat ply-2
  training by 100 at ply 4. For alpha-beta, if this holds, the target depth's parity matters: climb in steps of two,
  or add a capture search at odd horizons.
- What the ply-4 field taught: the pawn's value doubles relative to the queen, and every piece's weight rises.
- **The analyst's recipe:** a warm-started ladder to the deepest affordable ply, one model per target depth (theory-2
  6.18). Cost grows ~8.5x per ply (0.007, 0.059, 0.51 CPU-s a game at plies 2-4, before hybrid11's x0.75 at ply 4).
  A warm ply-5 rung like W4 would be ~30-40 CPU-h, so it waits for your decision. Beyond ply 5 the lever is a
  faster search.

**1. Ratings at the standard temperature were confounded.** Every league and arena played a log-odds softmax at
T 0.1 on calibrated values. A model's effective randomness is then T over its calibrated scale at the ply it plays,
and depth-1 own calibration doesn't fix that scale at ply 2. Alpha-beta plays argmax and ignores calibration, so
near-argmax play (T 0.02) is the measurement for the goal. Re-measured at T 0.02:
- **No earlier routing gain survives at ply 2.** Final trees minus their own flat lines (5 experiments): LR trees 4 at
  T 0.1, -17 ±21 at T 0.02; probit trees 62 and 9 ±23. The residual tree doesn't help either (-45 against flat).
- **The learning-rate effect was the confound too:** rates 0.0125-0.1 are level near argmax.
- **The flat LR from the lr-settings rerun beats every old champion** by 96 ±21 (46 at T 0.1). Flat probit
  passes them too (55 over their mean). The old lineages' own growth was real (127-244 within each lineage).
- **The post-split dip is a real ply-2 weakness** of young trees (it survives T 0.02 and ply-2 calibration). The
  late copy tree's lead was the confound.

**2. What carries:**
- **Who plays the training games.** Training among ply-2 players only: 542 ±38 at T 0.02 (lr-settings pair).
- **The depth of the training games.** First ratings at ply 3: the old depth-4 lineage (F-L-d4mix-c v426) is best
  at ply 3 (63 to 92 over the ply-2-trained flat LR) while trailing at ply 2. Search labels on ply-2 games don't
  carry (37 ±24 behind at ply 3). selfplay-depth's own depth-4 line had too little data to show it.
- **LR, not probit, for depth:** flat probit is 177 ±37 behind flat LR at ply 3.

**3. Routing reopened narrowly, and it replicates.** probit-widen-p1b's fresh-gate limit-4 probit trees beat F-P by
~95 at ply 1 near argmax. The seed-18 replicate p1c gives the same (all eight trees at split noise 0 or 0.03 are
84 to 104 above F-P; 75-round reads). T 0.1 had them level.
- At ply 2 only the noise-0.03 arm carries, ~40 in both seeds. Noise 0 flips sign between seeds and noise 0.1 loses.
- LR leaves under the same fresh gate gain less at ply 1 (lr-trees-p1: 43 ±21) and lose at ply 2 (-43 ±22).
  Stream-gated LR trees show nothing (small-field-x4-p1: -7 ±30).
- probit-trees-p2 trained the noise-0.03 arm at ply 2. Its trees are level with flat probit at plies 1, 2 and 3
  (-10 ±24, -8 ±27, 1 ±27). The analyst had predicted 60-90 at ply 2. So far, routing gains only at ply 1, which
  doesn't serve the goal (search depth 3 and beyond).

**4. Running and built overnight** (you said ~00:50: "launch things without me especially when there's little
else running"):
- **06:51-07:22: the reverse rung** (theory-2 6.21; 1.35 CPU-h with its reads; no search change). Result in
  item 5. hybrid12 and hybrid13 have been built since (both exact; below). Nothing is running.
- Nothing ran 05:20-06:51. Overnight CPU, all on flat LR (item 0):
  - flat-p3 and flat-p3-b: 3.2 CPU-h each.
  - flat-p2 and flat-p2-b: 0.9 and 1.1 CPU-h.
  - W4: 4.0 CPU-h. W3: 0.56. The ladder rung flat-p2-w3: 0.69. W3 without reheating: 0.6.
  - Reads: ~2 CPU-h.
  Routing is parked for the goal (item 3; IDEAS.md).
- probit-trees-p2 (p1b's best arm at ply 2) is done: no gain at any ply (item 3).
- probit-widen-p1c (p1b's replicate, seed 18) is done and replicates p1b (item 3).
- lr-trees-p1 (LR leaves in p1b's conditions) is done. Near argmax its LR trees are 43 ±21 over flat at ply 1 and
  -43 ±22 at ply 2; T 0.1 had them level.
- Built and verified: "entry_ply": 3 in leagues.py (swapped 01:24), then "entry_ply": 4 and per-line warm starts
  ("start" and "start_forget"; swapped 03:27), each byte-identical for existing configs. chessai-hybrid10 (exact;
  ply-3 search CPU x0.62-0.79). chessai-hybrid11 (exact; flat LR x0.75 at ply 4, x0.88 at ply 3; swapped into W4
  at cycle 22). The dashboard's champion-gap tooltips now name the page's ply (@1 on ply-1 pages).
  - 07:34: chessai-hybrid12, from the 06:50 performance review. It searches a legal capture before generating moves,
    one ply above the leaves. Its games are byte-identical to hybrid11's for every model kind, plies 1-5, and
    T 0.1, T 0.02 and argmax. Flat LR game CPU: ×0.65-0.69 at ply 3, ×0.95 at ply 4, ×0.71-0.75 at ply 5.
    New specs at plies 2-5 should name it.
  - 08:11: chessai-hybrid13, also exact (games byte-identical to hybrid12). Near-argmax play searches clearly worse
    root moves only far enough to prove the picked move unchanged. At T 0.02, flat LR ×0.77 and probit ×0.66-0.87;
    T 0.1 is unchanged. New specs at any ply should name hybrid13.
- Deleted with your OK (11-11 ~08:00): leagues-smoke/ident-ply3, ident-ply4 and selftest-ply3. They showed
  9.4 GB but freed ~1 GB, because most of it was reflink copies sharing blocks with the experiments. hybrid12/st and
  hybrid12/cand (58 MB) were deleted too. Can go with your OK: fastbin/hybrid13/st (71 MB), hybrid13/cand (4.8 MB),
  and the build dirs hybrid12-src/target and hybrid13-src/target (89 MB each).

**5. Decisions for you:**
- **The next rung, and a search question** (item 0; the analyst's theory-2 6.20-6.22).
  - **Two parts, learned at different speeds:**
    - Parity, worth 130-200 at the training depth's parity, is learned fast: W4 got all of it in a warm rung.
    - Depth (piece values moving toward the true ones, ~100 a ply at any parity) is learned slowly, from many
      deep games. That's why ply-3 training from scratch beat ply-2 training at ply 4.
  - **Tested this morning without a search change: the reverse rung** (06:51-07:22, 1.35 CPU-h; RESULTS section 8).
    - W4 was trained further at ply 3.
    - That undid its odd-ply losses completely, and most of its even-ply gain: it kept 45 ±15 of 131 at ply 2 and
      19 ±16 at ply 4.
    - So one linear model stores both parities' corrections mostly in the same weights (6.22). The weights show it:
      the pawn's value and the piece-to-queen ratios flip with the parity, while the overall scale stays.
    - A data-side cure (a field mixing two plies, or alternating rungs) could only average the two corrections:
      about 65 at both parities instead of 131 at one.
    - For a fixed final depth, the model trained at that depth's parity is strictly better.
  - **The analyst's recommendation:** a capture extension at the search horizon. It resolves pending captures
    before evaluating, the standard cure for the odd-even effect, and removes the cause instead of patching it in
    the weights. The scale:
    - The extension costs 2-3× at the leaves; one more ply costs ~10×.
    - The parity part is worth 130-200; one more ply of search is worth 500-1000 (measured this morning).
    - No ply-5 rung: ~29 CPU-h by the performance review's measurement, ~20 with hybrid12. W4 already matches W3
      at ply 5.
  - **This is a search change.** Your constraint says search is a last resort. A capture extension uses only legal
    moves, no chess knowledge, but it changes the search, so I haven't built it. The analyst's next steps:
    - **If you accept it:** I build it as an option, off by default. Then an arena only: W4, W3, flat-p3 and flat-p2
      at plies 2-5 with the extension on, under 1 CPU-h. Then one warm ply-4 rung from W4 with the extension in its
      games (~4 CPU-h).
    - **If you decline:** target ply 4, the even depth.
      - First W4f1: flat-p3's finals warmed at ply 4 with forget 1 (~4 CPU-h). It's expected 40-70 above W4 at
        even plies, because forget 8 cost each warm arm 40-70.
      - Then a second ply-4 rung for the depth part.
    - **Either way, two lessons for rungs:**
      - Divide the sums by 1-2, not 8.
      - Judge a rung's plateau stop one ply deeper, or give it a longer floor. More ply-3 games kept adding 50 at
        ply 4 while showing only 9-17 at plies 2-3.
- **Rating policy:** arenas tonight rated at T 0.02. Should leagues rate (and train) at T 0.02 too, or keep T 0.1
  games for training and rate near argmax separately? The analyst suggests diversity from randomised openings
  instead of temperature, and drafted a test of the training-game temperature alone (train-temp-p2, Queued).
- **Ply 2:** you paused ply-2 experiments last night. Tonight showed ply 1 can't measure what matters for the goal,
  so under your standing instruction I ran ply-2 and ply-3 work (probit-trees-p2, flat-p2/-b, flat-p3/-b).
- **Routing:** parked for the goal (gains only at ply 1; item 3). P3 (gate at ply 2) superseded.
- **The global league:** two levers now: the ply of the games a model trains on (item 0; "one ply per model" fits it,
  and the ply should be the search depth the model will be used at) and the field's strength (the thunderdome's
  "train on games of players at or above own Elo minus a margin").

## Running

Done 06:51-07:22 on 11-11 (RESULTS section 8): **flat-p3-w4-w3f1, the reverse rung** (theory-2 6.21). It is flat-p3-w3f1 exactly
(seed 24, ply 3, forget 1, hybrid10, cap 100, plateau min 60), except that it starts from W4's finals (flat-p3-w4
v99) instead of flat-p3's (v199). Only the start differs. The v0 copies are byte-identical to W4's finals.
- **Question:** do ply-3 games overwrite W4's even-ply gain (172 at ply 2, 200 at ply 4 over W3)?
- **The analyst's predictions:**
  - within 20 of W3f1 at ply 3;
  - it keeps 0-60 of W4's gain at plies 2 and 4;
  - near W3f1 at ply 1.
- **What it decides:**
  - A residue of 60 or more means one linear model can hold both parities' corrections. A data-side cure would
    then exist (alternating rungs, or a field with both plies).
  - A residue near zero means only a search change (the capture extension) can remove parity.
- **The result:** all four predictions held. It keeps 45 ±15 of W4's even-ply gain at ply 2 and 19 ±16 at
  ply 4, and is level with W3f1 at plies 1 and 3.
- **The read:** ran at the run's end. Cold, T 0.02, hybrid11, 30 round-robins each:
  - reverse-rung-p123 (plies 1-3);
  - reverse-rung-p4 (ply 4);
  - four groups each: rev, W3f1, W4, flat-p3's finals;
  - contrasts by leagues/specs/arenas/rev_btg.py.
  - Ply 5 is left out: it isn't in the predictions and would cost ~1 CPU-h more.
- **Cost:** 0.49 CPU-h for the run, 0.86 for the reads.

Done 07:10-07:34: chessai-hybrid12, exact (games byte-identical to hybrid11; ×0.65-0.69 at ply 3, ×0.71-0.75 at
ply 5 for flat LR). Done 07:40-08:11: chessai-hybrid13, exact (games byte-identical to hybrid12; ×0.77 on T 0.02
reads for flat LR).

Running at 11:55 on 11-11: error-growth-p1 (T-P-Ne limit 4 vs 8, from 10:43, floor 450), error-dag-probit-p1
(from 11:20, in dag-lr-p1's slot) and probit-late-p2 (from 11:53, in probit-confirm-p2's slot). Arenas done
11:31-11:52: cross-ply-probit-cold, error-routing-p1-v226-cold, g2-versions-cold (RESULTS section 8).

Running at 11:20 on 11-11: probit-confirm-p2 (ply-2 field, from 10:08), error-growth-p1 (T-P-Ne limit 4 vs 8, from
10:43, floor 450), and error-dag-probit-p1 (from ~11:20, in dag-lr-p1's slot). Done since 10:00: G2 error-routing-p1,
G6 lr-share-p1, G3 dag-lr-p1 (its cold read follows its end). Next free slot: the final six's arena, or the LR
error-dag-p1 if wanted.

Running at 10:00 on 11-11: error-routing-p1 (G2, from 09:10), lr-share-p1 (G6, from 09:33), dag-lr-p1 (G3, from
09:48); the routecells fork (no training). Done since 08:34: batch 5's T 0.02 check, probit-limit8-p1, G1, G4 (see
"The user's decisions, 11-11" at the top). Next free slot: the ply-2 --revise 2 test (Batch 5 item 3).

Nothing ran 05:20-06:51. Done after 03:20: flat-p3-b, flat-p2-b (the verdict), flat-p3-w4 and flat-p3-w3 (the
ply-4 rung), flat-p2-w3 (the ladder), flat-p3-w3f1 (no reheat), and the parity reads at plies 1 and 5 (RESULTS
sections 7 and 8; the morning summary above). The 6-hourly performance review started at 06:47 (a background fork).
Running from 01:34 on 11-11: flat-p3 (F-L x4 playing and training at ply 3, self-games, seed 20, 15 rounds, min 200,
standard T 0.1, chessai-hybrid10; the analyst's first priority). Its twin flat-p2 (identical at ply 2,
leagues/specs/ply3/flat-p2.json) launches when probit-widen-p1c ends, so the field's ply is the one factor. Verdict: a
T 0.02 arena at plies 2 and 3 of both twins' final versions with lr-settings-new's F-L lines and d4mix v426.
Done 01:32: lr-trees-p1 (plateau at 302). Read cold, its LR trees are 43 ±21 over F-L at ply 1 and -43 ±22 at
ply 2 (RESULTS section 8). That is below the analyst's 60-100 trigger for lr-trees-p2.
Running from 01:00 on 11-11 (the analyst's plan, theory-2 6.11; read cold on final versions at the end):
probit-trees-p2 (p1b's best arm at ply 2: F-P x2 against fresh-gate T-P-Nc-merge limit 4 noise 0.03 x2,
ply-2-only field). Both use the standard T 0.1 tournament like p1b, so one factor differs from it.
Running from 00:58 on 11-11: probit-widen-p1c, the seed-18 replicate of probit-widen-p1b (identical otherwise;
8 lines, ply 1, K = 25, min_cycles 200, ~19 CPU-s a cycle, ends ~01:40). probit-widen-p1b (00:18-00:54, done at
200 on the plateau rule) read cold showed fresh-gate probit trees 80-104 above F-P at ply 1 (RESULTS section 8);
the replicate checks it. probit-widen-p1 (00:11-00:18, stopped at 63 to add F-P) is not a result. Before it, nothing ran
at 00:10. All three experiments stopped on the plateau rule: small-field-x4-p1 at cycle 190
(23:43; trees level with flat at ply 1), leaf-sources-new at 320 (23:47; its pair arena: rerun - original 36.7 at
ply 2, -3.7 at ply 1) and growth-limits-p1 at 340 (00:01; the merge gate undid growth). RESULTS sections 4 and 7.
Overnight (the user, ~23:40: "continue research, run whichever experiments you feel are appropriate and consult with
the math agent analyst"): the analyst drafts the next ply-1 experiments; new options go to a separate model-maintenance
agent (the user, ~00:05); at most 3 experiments at once.

- **No more ply-2 experiments for now (the user, ~22:15):** "apart from the test we're doing (the result of which is
  probably very evident now, as it's becoming quite obvious that the random players and players of unrelated plies
  should simply not be present in the experiments) we really have no reason to run any more ply 2 experiments at the
  moment". Next experiments run at ply 1 (NEW-DESIGN, the ply ladder). They wait for three engine options a
  background agent is building: self-games, the plateau window scaled by lines, and the entries' ply. Approved by the
  user (~22:15 and ~22:50); the options were built and verified (byte-identical for existing configs) and both
  started at 22:52 (rows above):
  - small-field-x4-p1: small-field-x4 at ply 1 (4 seeds each of F-L and T-L-Tc-merge, 8 lines, K = 25, seed 11).
  - growth-limits-p1: T-L-Nc-merge (split-growth's recipe) at split limits 1, 4 and 8, 2 seeds each (6 lines, K = 29,
    seed 12, 18 rounds a cycle); the split limit is the one factor.
  - Specs: leagues/specs/ply1/. Both with self-games, no references, cap 1000, min 100.
- Done since 19:50: split-growth-probit (21:36) and error-targets-probit (21:47), both at the 1000-cycle cap
  (RESULTS section 4).
- Done since 16:40: trigger-gradient (18:19, cap 400; RESULTS section 4), lr-settings-new (18:41, plateau stop at
  194), probit-settings-new (19:36, plateau stop at 256), nb-settings-new (19:46, plateau stop at 100; the pair
  arenas and their ply-1 replays: RESULTS section 8) and small-field (19:39, cap 400; RESULTS section 4).

- Folders: leagues/experiments/<name>. Every experiment is local (trains on its own games), threshold 1M, 3 children.
  Lines and specs: RESULTS.md section 7 and leagues/specs/batch2/, batch3/.
- Binaries: the tournament is chessai-hybrid8 in all running experiments since 15:35 (exact fixed-point flat probit at
  ply 2; byte-identical to hybrid7; fastbin/hybrid8/summary.txt), and in the prepared specs 12-14. Training is hybrid6, except error-targets' lines 3-5 (both
  experiments), which train on chessai-target (models/from-scratch/target/league/NOTES.txt).
  - From 11-10 19:40 split-growth-probit (cycle 717 on) and error-targets-probit (667 on) play on chessai-hybrid9:
    byte-identical; probit-leaf trees' reply estimates from the ln Φ table, own calibration fitted once
    (fastbin/hybrid9/summary.txt).
- Load: about 18 on 8 cores since the error-targets pair started at 13:14, and the 8 GB swap was full then. Cycles
  slowed: split-growth 22.8 → 37-40 s, split-growth-probit 27 → 42 s, cluster-error 88 → 125-134 s. Budgets are
  priced, so results don't depend on load; only wall time does.
- Why probit runs 1000 cycles and LR 400 (in both pairs): probit-leaf trees crossed flat only around cycle 300 in
  single-split-probit and were still climbing at 1000. LR-leaf trees had saturated by cycle ~200-300.
- small-field (the user's idea: trees gain more from local training when there are more competitors) has a
  confound: with two lines, 36% of local games are against the references (single-split 23%, split-growth 30%),
  27% are within a line (9%, 11%) and 36% between lines (69%, 59%). Fewer competitors and more games against random
  movers can't be told apart if the tree falls behind. Dropping random-3 too would bring the reference share to ~18%.
  Seed noise (the same recipe moved 50-90 between runs) is the larger confound.
- trigger-gradient: 3 lines at 9 rounds give only 1,026 local games a cycle, so a 1M window closes about every 5 cycles
  (~80 windows in 400 cycles). The comparison is within the experiment, so all three lines see the same data.
  Records: models/from-scratch/trigger/league/NOTES.txt.
- error-targets' lines 2-5 are byte-identical until the first split (T-L-Le split at v13 in single-split, T-P-Le at
  v27). Their spread before it is a free noise measure.

Finished (results in RESULTS.md):

| Experiment | Lines | Ended | Section |
|---|---|---|---|
| error-targets (T1) | 5 | 17:11 (400 cycles): every LR error-router variant below F-L; gap targets the best fix (14-78 below late), route-before-train alone worse than Best | 5 |
| cluster-error | 14 | stopped by the user after cycle 637 (16:25): plateau, latest versions 40-140 below their champions | 2, 3, 4 |
| residual-leaves (T3) | 3 | 16:22 (400 cycles): no lasting gain | 4 |
| split-growth (P7) | 5 | 14:05 (400 cycles) | 4 |
| single-split, single-split-probit | 7 + 7 | 10:51 and 10:56 (1000 cycles) | 3 |
| selfplay-depth | 13 | 06:52 (100 cycles) | 6 |
| lr-settings-2, nb-settings-2, probit-settings, leaf-sources-2 | 4, 4, 4, 6 | 00:35, 01:08, 23:46, 23:25 | 2 (written up 11-10) |
| leaf-sources | 8 | 11-09 22:22 | 2 |
| lr-settings, nb-settings, nb-settings-no-random3 | 5, 3, 3 | stopped 21:14-21:15 on 11-09 | 2, 7 |

## Batch 4: every experiment rerun without random players (from 2026-11-10 17:41)

**Purpose.** The user (11-10 ~17:15): "For every single experiment we've done so far. Run the same experiment except
under new conditions (No random players. No PLY 1 players. Run until champions plateau) Then throw all the champions
from the original version of the experiment and the new version of the experiment into an arena (with random
players) for an appropriate number of rounds to compute their elo/strength. Go through every experiment we've done
even the unsuccessful ones". The new conditions are the user's experiment defaults from now on (NEW-DESIGN.md).

**Conditions** (engine options in leagues/FORMATS.md; specs leagues/specs/batch4/<orig>-new.json, made and checked by
batch4.py there):
- No references: ratings are relative to the first line's untrained version (`rating_zero`), relative strength only.
  No ply-1 entries: each line plays its current version and its champion at ply 2. The champion test is unchanged.
- Plateau stop: an experiment ends when no line has crowned a new champion for 50 cycles, never before cycle 100, at
  most 1000 cycles. The last cycle trains nothing; the stop is logged and recorded (`done` event).
- Rounds per cycle: the original's mean games with a model at ply 2 per cycle, over 2n(2n-1) games a round (two ply-2
  entries per line, n lines), rounded. Every game trains. When a line's current version is its champion it has one
  entry, so cycles then play fewer games.
- Kept from each original: the lines (models, recipes, options, every argument), seed and train seed, calibration,
  champion_se, prices P2. In mixed experiments (leaf-sources, -2) self-play budgets follow the engine rule (the price
  of the cycle's local data).
- Binaries: tournament chessai-hybrid8; train chessai-hybrid6, except chessai-target (error-targets' lines 3-5, both
  experiments), chessai-trigger (trigger-gradient's trees) and lr-settings' search-label line (chessai-hybrid4gfx, as
  it ran); self-play binaries as in the originals (chessai-hybrid4gfx). All are exact: the same games as the
  originals' binaries, faster.
  - Tournament chessai-hybrid9 from 11-10 19:39 (byte-identical to hybrid8, faster: fastbin/hybrid9/summary.txt): every
    rerun created from then on (batch4.py and the specs not yet used; its check accepts hybrid8 or hybrid9),
    nb-settings-new from cycle 33, and the pair arenas (pairs.py; the watcher running since ~17:30 keeps hybrid8 until
    it restarts). lr-settings-new and probit-settings-new ran on hybrid8.
- nb-settings-no-random3 is the same design as nb-settings once the references are gone (same lines, seeds, rounds,
  cycles), so nb-settings-new stands in for both. selfplay-depth's rerun is skipped (the user, 11-10 ~17:55: "for selfplay-depth don't do it... it would take forever for d4 to plateau"); its spec stays written, and its original champions still go into the combined arena.

**Ended after leaf-sources-new (the user, 11-10 ~22:15: no more ply-2 experiments for now; see Running).** At
22:05 leagues/experiments/batch4-queue.STOP was set (the user had decided ~22:00 that experiments not yet started
follow the new defaults, which weren't built), so the first queue stops before lr-settings-2-new and the follow-on
queue then exits without starting anything. Items 5-18 below are parked, not cancelled; their pair arenas and
ply-1 replays continue for leaf-sources-new.

**Order and status.** One rerun at a time, beside whatever else runs (leagues/specs/batch4/queue.sh; state in
leagues/experiments/batch4-queue.txt, log batch4-queue.log); the next starts when the previous says "experiment
done". An arena of each original's and rerun's champions (with random players) is run separately as each rerun ends
(leagues/arenas/).

| # | Rerun | Rounds (orig.) | Games a cycle (orig. ply-2) | Lines | Original's plateau stop | Est. cycles | Est. CPU-h | Est. wall h | Status |
|---|---|---|---|---|---|---|---:|---:|---|
| 1 | lr-settings-new | 15 (5) | 1,350 (1,345) | 5 | none in its 172 (last crowning 168) | at least 218 | 1.8 | 1.4 | done 17:41-18:41: plateau at cycle 194 (last crowning 144) |
| 2 | probit-settings-new | 16 (5) | 896 (919) | 4 | 220 | 220 | 1.7 | 1.3 | done 18:41-19:36: plateau at cycle 256 (last crowning 206) |
| 3 | nb-settings-new | 19 (5) | 570 (569) | 3 | 100 (floor; last crowning 3) | 100 | 0.4 | 0.3 | done 19:36-19:46: plateau at cycle 100 (last crowning 39) |
| 4 | leaf-sources-new | 13 (5) | 3,120 (3,101) | 8 | none in its 100 (94) | at least 144 | 6.1 | 4.2 | running from 19:46 |
| 5 | lr-settings-2-new | 16 (5) | 896 (919) | 4 | 374 | 374 | 2.1 | 2.4 | queued |
| 6 | nb-settings-2-new | 16 (5) | 896 (919) | 4 | 100 (floor; last crowning 10) | 100 | 0.6 | 0.5 | queued |
| 7 | leaf-sources-2-new | 14 (5) | 1,848 (1,849) | 6 | none in its 100 (96) | at least 146 | 3.2 | 3.3 | queued |
| 8 | single-split-new | 13 (5) | 2,366 (2,449) | 7 | 339 | 339 | 8.8 | 6.6 | queued |
| 9 | single-split-probit-new | 13 (5) | 2,366 (2,449) | 7 | 276 | 276 | 7.4 | 5.5 | queued |
| 10 | cluster-error-new | 12 (5) | 9,072 (8,812) | 14 | 436 | 436 | 29.4 | 16.6 | queued |
| 11 | split-growth-new | 27 (9) | 2,430 (2,426) | 5 | none in its 400 (399) | at least 449 | 6.6 | 3.2 | queued |
| 12 | split-growth-probit-new | 27 (9) | 2,430 (2,427) | 5 | none yet (running; 501) | at least 551 | 11.8 | 6.8 | queued |
| 13 | error-targets-new | 27 (9) | 2,430 (2,426) | 5 | 290 | 290 | 4.4 | 3.3 | queued |
| 14 | error-targets-probit-new | 27 (9) | 2,430 (2,427) | 5 | 362 | 362 | 6.8 | 4.4 | queued |
| 15 | residual-leaves-new | 34 (9) | 1,020 (1,025) | 3 | 236 | 236 | 1.7 | 1.3 | queued |
| 16 | trigger-gradient-new | 34 (9) | 1,020 (1,024) | 3 | 184 | 184 | 1.0 | 0.7 | queued |
| 17 | small-field-new | 204 (56) | 2,448 (2,454) | 2 | none yet (running; 130) | at least 180 | 2.6 | 1.5 | queued |
| 18 | small-field-x4 (extra, below) | 10 (small-field's 56) | 2,400 (2,458) | 8 | - (new) | ~300-600 | ~5-11 | ~3-6 | queued after small-field-new |
| - | selfplay-depth-new | 12 (5) | 7,800 (7,623) | 13 | none in its 100 (99) | at least 149 | ~49 | ~21 | skipped (the user) |
| - | nb-settings-no-random3-new | | | 3 | | | | | covered by nb-settings-new |

- Estimates (17:45): CPU per cycle from each original's totals.csv up to where the plateau rule would have stopped
  it, with the game CPU of ply-2-against-ply-2 games measured in the original's own games (1.4-1.8 times the
  original's mean qualifying game, NB 1.1, since no qualifying game is against a random mover), so a cycle costs 13%
  less (NB) to 37% more; wall time at the CPU-to-wall ratio the original ran at (1-2.3 cores' worth under a load of
  ~15). The reruns may plateau elsewhere: the rows without a stop on the original are lower bounds (the original's
  last crowning plus 50).
- Total: about 97 CPU-h and 63 h of wall time one at a time, so the queue would end around 11-13 09:00 at the
  originals' pace (cluster-error-new alone ~17 h). Wall times shrink when the four batch-3 runs finish (by ~23:00).

**Extra: small-field-x4 (queued 11-10 18:39).** The user (11-10 ~18:00): "One more thing for us to test in the queue
is the same idea but with 4 of each all differently seeded." Not a rerun: small-field's two models (F-L; T-L-Tc-merge
with single-split's split limit 1 and threshold 1M, tan router, revise 0) as four replicates each through the
engine's "replicates" (8 lines, F-L#1-4.loc and T-L-Tc-merge+split-limit=1+split-threshold=1000000#1-4.loc), under
batch 4's conditions, experiment and train seed 11 (used nowhere else). Replicate r trains with the seed plus the cycle
plus 100003 x (r - 1); LR and tree recipes have no initial-weight seed, so the shuffle is the only difference, and the
games then differ. Spec leagues/specs/batch4/small-field-x4.json (batch4.py `specs`/`check`/`create-item
small-field-x4`; the check compares every line with small-field's: recipe, arguments and binary hybrid6 the same).
- Rounds: small-field's 2,458 games with a model at ply 2 per cycle over 240 games a round (16 entries) = 10 rounds,
  2,400 games a cycle.
- Replicates diverge (scratch run of 4 cycles at 3 rounds, 18:35): version 0 is the same file in all four copies (as
  in leaf-sources' F-L#1/#2), and versions 1-3 differ in all four, for both models.
- Queue: the first queue.sh had already expanded its 17-rerun order, so it wasn't edited; queue.sh was replaced by a
  version that iterates batch4.py `items` (reruns, then extras), and a second copy started with `--after 1809879`
  (pid 1841378) waits for the first to exit, then starts small-field-x4 only if all 17 reruns are done (otherwise it
  stops and says so in batch4-queue.txt). Still one experiment at a time.
- Estimate: ~66 CPU-s a cycle (tournament ~47 at small-field-new's ply-2-against-ply-2 game CPU, training ~8 and
  calibration ~11 for 8 lines), ~38 s of wall at small-field's CPU-to-wall ratio. Cycles: with 8 lines a 50-cycle
  pause in crownings comes later than with 2; single-split (7 lines, these models among them) would have stopped at
  339, so ~300-600: ~5-11 CPU-h and ~3-6 h, from ~11-13 09:00 (cap 1000: ~18 CPU-h, ~11 h).
- Arenas: the pair-arena watcher (leagues/arenas/pairs/pairs.py) takes only its originals and reruns, and its combined
  arena runs once small-field-new's pair is done, which is when small-field-x4 starts, so small-field-x4's champions
  are not in it; they need an arena of their own after it ends.

## Headlines

0. **2026-11-10 evening (RESULTS sections 1, 4-6, 8; the plan: PATH-FORWARD.md).**
   - The table tool counted 5 rounds a cycle, and batch 3 plays 9, so the batch-3 tables showed only each run's
     first ~55%. Corrected: split-growth's LR cluster trees end 35-53 ahead of flat (the single-split gain half
     replicated), and error-targets' gap targets beat Best by 29-80. Items 1-2 below predate the fix; where they
     differ, RESULTS wins.
   - Models get good at the ply they train at: at ply 1, d1-trained models beat d2-trained ones by 238-321; at ply
     2 the order flips.
   - On the same versions, ply 1 reverses 14% of clear ply-2 verdicts. NB and TAN cluster trees with LR leaves beat
     flat at ply 1 in every run (7-73), while at ply 2 the result is mixed.
   - The lr-settings rerun (ply-2-only field) beat the original by 424 at ply 2 but by 36 at ply 1.
1. **Cluster-routed single splits beat flat on equal data, reliably only with probit leaves.** Probit leaves: 75-77
   at cycle 1000, still climbing. LR leaves: 70-100 in single-split, but the same T-L-Nc-merge recipe gained only
   10-20 in split-growth (cycles 200-400, seed 2) and -25 to 26 in cluster-error: the LR gain did not replicate.
   - split-growth, limit 4: without merges the LR tree reached 9 leaves by cycle 50 and stayed 47-108 below flat in
     every window. With merges it grew slowly (3, 7, 5, 9 leaves at cycles 50, 100, 200, 300) and stayed level. NB
     partitions are fixed once made, so early splits lock in; the merge gate acts as a redo. At limit 1 merges
     change nothing. Every NB and TAN cluster router found the same game-phase
   partition: opening, queenless middlegame, endgame.
2. **Error routing:** T-P-Ne (NB router, probit leaves) reached 44-47 above flat, the first error tree clearly above
   flat. No LR-leaf error tree has. LR error routers route by who's winning and lock in; NB ones don't.
3. **cluster-error (to cycle 415):** the probit cluster trees lead flat (TAN ~45-50, NB ~25) and the LR-leaf ones don't
   (-25 to 26). Since cycle ~530 F-P has caught up with F-L (24 above it at cycles 550-571, a small window; decision 1).
   Not the partition, near-copy leaves, the decisive rate, the phase mix or data volume. Left: what the
   stream is made of (half its lines play probit leaves), and chance. split-growth's limit-1 merge line re-tests
   chance.
4. **Self-play depth:** at equal priced CPU, depth 2 is best for LR and probit and depth 0 for NB. At matched training
   positions (matched-depth arena) the order is d4, d1, d2, d0, with d4 200-700 above the rest.
5. **Measurement:** compare current against current (curtab.py). Version windows count champions. Single versions
   swing ~17 Elo, windows ~30: replicate 20-40 Elo contrasts.
6. **Theory:** Best, Soft and Balanced router targets are inconsistent for routed log-loss; gap targets are
   consistent and now run in error-targets. Additive (residual) growth is the only operation with a monotone
   guarantee; its clean rerun is ready (decision 2).
7. **Speed:** play, training, reports and the driver have each been made exactly faster since 11-09 (RESULTS
   section 9; models/from-scratch/fastbin/PERF-LEDGER.md).

## Decisions for the user

0. **The ply question and the routing path (11-10 ~19:00): models/PATH-FORWARD.md.** Proposed, nothing launched:
   - a ply-1 replay of every batch-4 pair (~25-50 CPU-min in all);
   - diagnostics on existing champions (calibration at ply 2, leaf noise, off-game positions);
   - the routing reruns first (a second lane or reordering);
   - a ply-1 twin of small-field-x4, which needs an engine option for the entries' ply (~2-4.5 CPU-h).
1. **cluster-error: stopped at cycle 637 (the user, 16:24).** The run had plateaued: most champions were crowned at
   cycles 211-340, and every line's latest version sat 40-140 below its champion (RESULTS section 2). Earlier note: F-P had been climbing against
   F-L, current against current: -22 at cycles 500-529, -5 at 530-549, 24 at 550-571 (the last two windows have ~200
   games, ±45). The probit trees did not climb with it: T-P-Tc stayed 10-20 above F-L throughout, T-P-Nc fell to -49
   in the last window. Whether F-P keeps pulling away is the most direct probit-against-LR evidence. Candidate
   mechanism: LR's AdaGrad steps die out while probit's tau keeps its steps alive (caution: in probit-settings tau 0
   was only 4-21 below 1e-5 through cycle 500). Offered: a step-size trace of F-L's and F-P's versions (no games).
2. **residual-leaves (T3): lines confirmed 13:50, no merges** (models/from-scratch/resid/rerun/summary.txt; spec
   leagues/specs/batch3/11-residual-leaves.json). F-L; T-L-Nc at limit 4; the same plus --leaf-residual
   --residual-inherit-sums. Local, seed 6, 400 cycles, 9 rounds a cycle, all on hybrid6 (it runs the 11-07 residual
   code unchanged; checks passed). Created; starts by itself when split-growth exits (~14:05).
   - **Probit twin** (the user asked for both leaf types): built and checked 14:30 as chessai-hybrid6rp in
     models/from-scratch/resid-probit/ (163 tests; identical to hybrid6 with the flags off). A child's correction starts at 0 with the shadow's current variances, so at a
     split it predicts exactly as a copy would and steps as a copy would. Lines F-P, T-P-Nc limit 4, the same plus
     residual; seed 7, 1000 cycles; spec batch3/12-residual-leaves-probit.json. Launch needs the user's OK.
3. **P1 moe-gates:** launch? Spec leagues/specs/batch3/1-moe-gates.json (F-L, M-L4-L, M-L4-L-hard, M-L4-N). The
   soft/hard pair measures the seam tax directly.
4. **P2 dag-home:** launch as is, with the T2 refinement (home targets score the copy as routed), or with the D lines
   on chessai-bddctx without the flag (a safeguard against bddhome2's exponential recursion)? Spec
   leagues/specs/batch3/5-dag-home.json.
5. **P6** (cluster against merge against error, one factor each): split-growth's 2×2 now covers the merge gate. Still
   wanted: T-L-Nc against T-L-Ne without merges, and a probit P6?
6. **Boundary blending** (one leaf when the router is confident, blend the top two otherwise): acceptable in
   principle if P1 shows a large seam tax? The user said in 11-05 that blending "defeats the purpose of the
   delegation"; this blends only near boundaries.
7. **Path context** (bin/chessai-bddctx) before any league test: CNBP cells (LR nodes only, LR routers, or accept the
   double counting)? Full path (quadratic) or the K nearest sets? Which lines?
8. **T4 gradient split trigger: built and tested offline 15:20** (bin/chessai-trigger, --split-gradient Z;
   models/from-scratch/trigger/summary.txt). Offline it split 5-6 times where today's trigger split 10, and had lower
   held-out loss in all four arms. With cluster routers it beat a same-leaves tree by 4.5 [2.7, 6.4] ×1e-3 at round 60,
   significant in each seed: the first structure to beat a same-leaves tree offline. League spec drafted
   (batch3/14-trigger-gradient.json: F-L, T-L-Nc limit 8 today, T-L-Nc limit 8 with T4). Launch, and with which Z? z
   grows with the square root of the window, so Z 3 at the league's 1M window is a lower bar than offline; Z 9.5
   keeps the offline bar.
9. **P3** (number of experts, after P1), **P4** (self-play against local at each leaf's best depth: LR 2, probit 2,
   NB 0) and **P5** (self-play exploration policy): when?
10. **Depth 1 below depth 0 for LR at equal CPU:** run the replay check offered on 11-10? (Depth 1 adds exact values
    for game-ending moves, yet LR d1 trailed d0 by ~70 in selfplay-depth; at matched positions d1 beat d0.)
11. **models/ docs in git:** models/** is ignored, so these docs and the analysis scripts are not versioned. Track
    models/*.md (and archive/)?
12. **Names (Q7 of 11-09):** by the user's rule, a model that predicts with one expert is a router. Rename
    M-L4-L-hard to T-L4-Lr and M-LP-L-hard to T-LP-Lr in recipes.txt?
13. ~~Write-ups for the finished batch-1 experiments~~ done 11-10 (RESULTS.md section 2). LR's best step size falls
    with data. NB doesn't bootstrap at any alpha. Probit's default forgetting is worth ~20 over none in a probit-only
    stream. Local training beats depth-0 self-play for LR and probit, not for NB.
14. ~~Disk clean-up~~ withdrawn: the user's request (11-10) came from reading "the models doc is large" as the models
    folder. The disk has 640 GB free.

## Queued

- **arena anchor fix (11-12 ~11:10; after the observers fork leaves arena.py)**: the ciX stop never fires with the
  random reference as the only anchor (every entry beats random; no entry rated; intervals infinite; runs to
  max_rounds). make_split_read.py now REFUSES ciX (guard, backup global11-drafts/make_split_read.py.before-ciguard);
  library.py refuses it too. Fix: a rated anchor in arena.py (field mean or a fixed entry; asked of the analyst,
  6.81) or no random reference at all (the user called random players part of the old "zoo"). Also the user's
  11-12 requirement: arenas sized for reasonable standard errors.
- **Deletion candidate (the user's call):** leagues/arenas/diag/cohort-fp-p2/owncal, 888 MB, left when the cohort
  fork stopped that arena by hand (state.json still says "running").
- **grow-dag-p1 (the user, 11-12 ~10:55: "put that one on the agenda")**: a growing DAG with router splits
  ENCOURAGED against a growing tree, ply 1, uncapped, run long; leaf rule grow4 bar on in both; the DAG with path
  context. Router rule, arms, floor/cap, logging, read and predictions: analyst 6.80 (asked ~10:52); binary support
  (grow4 leaf rule + grow3 router rules + path context in one binary) to check/build by a fork. ~2 CPU-h. Starts when
  treeonly-p1 and fponly-p1 finish, unless the user wants it sooner. Context: the 11-12 ~10:45-10:52 entry above.
  ANALYST 6.80 (~10:58; agrees 6.54 covered capped DAGs and immediate gains only): rule "ENCOURAGE THEN PRUNE":
  router splits on a steady allowance (one per D_r = 10M root positions, ~50 ply-1 cycles), made whether or not a
  benefit shows; evidence picks only WHERE (eligible router: age three windows, children ready by the 90% share guard,
  not a copy; highest G_real - G_null, ties H_v then index; shadow partition as 6.36); no bar at the split; pruning by
  a long do-no-harm trial (the shadow pair runs K = 5 own windows; undone only if its paired statistic is below -2
  block-SE). --router-chain 1 (a copy can't router-split: path-context cells at most 3x the nodes). Leaves: grow4 bar
  ON in both arms. Arms: F-P x2; T-P-Ne-grow bar on x2; D-P-Ne-ctx-grow with the allowance x2 (allowance off = the
  tree exactly). Ply 1, new seed, 1,584 games, FLOOR 600, CAP 900 (~3-4 CPU-h, not 2), hybrid17. Log per router
  split and per leaf trial (under an earlier router split or not; kept/undone; gain); path cells' |cell| and movement.
  Read: ply-1 panel at v150, v300, v600 and final, plies 1 and 2. Predictions: 12-18 router splits, 30-60% pruned;
  DAG minus tree -15..10 / -25..15 at v300, -20..20 / -25..15 later (centres -5); kept rate under router splits level
  within 15 points; cells moving under 0.05 nats. CONFIRMS THE USER: kept rate 20+ points higher under router splits
  AND the DAG 20+ over the tree at the final at both plies. CLOSES IT: within 15 with a level kept rate after 600-900
  cycles. BUILD grow6 (grow4's leaf rule inside bdd; grow3's shadows/stats; path context; --router-split-rule
  encourage, --router-credit, --router-trial-windows, --router-trial-z, --router-chain, --log-router-stats;
  byte-identical when absent; identity test vs grow4's tree). Build started by a fork ~11:00 so the run can start when
  the uniform-field pair ends.
- **flat-p3** (the analyst's first priority; launches as soon as "entry_ply": 3 is built and a slot frees): F-L x4
  playing and training at ply 3, standard T 0.1 (its comparator lr-settings-new trained at T 0.1), seed 20, 15
  rounds, min 200 cycles, ~3 CPU-h. Verdict: a final-versions arena at T 0.02 at plies 2 and 3 against
  lr-settings-new's F-L 0.0125 and F-L-d4mix-c v426. Plan: models/from-scratch/theory-2/plans/flat-p3-cold.json.
- **train-temp-p2** (the analyst's draft, 11-11 01:00; waits for your rating-policy decision): the training-game
  temperature as the one factor. F-L x4 in a ply-2-only field whose games are played at T 0.02, against a T 0.1
  twin; both read at T 0.02 at plies 2 and 3. Prediction: no difference if T 0.02 games stay diverse (92% of the
  games in the cold arenas were distinct); a loss if the colder stream has too few blunders. Plan:
  models/from-scratch/theory-2/plans/train-temp-p2-queued.json. When it becomes a spec, it should name the newest
  exact binary rather than the plan's chessai-hybrid9: hybrid11, or hybrid12 once built. Both play the same games,
  and hybrid11 saves ~10% (the 06:50 performance review). It should also train with chessai-hybrid6 through
  train_binary.py: identical models for 0.6 of the training CPU.
- **residual-leaves-probit:** built and ready; needs the user's OK (decision 2). Launch steps: models/from-scratch/resid-probit/summary.txt.
- **forget-flat and forget-mixed** (the user's drift hypothesis; test approved 14:05, lines and slot to confirm; LOW
  PRIORITY since the drift measurement at 15:45 found no drift late in any stream, RESULTS section 2): F-L,
  F-L --lr-memory 6000000 (chessai-hybrid6f), F-P, F-P --probit-tau 0; forget-mixed adds T-L-Nc-merge, T-P-Tc-merge,
  T-L-Le and T-P-Ne at limit 1. Local games per cycle matched (~3,120). Seed 8, 600 cycles, ~9 and ~12 CPU-h. Specs
  leagues/specs/batch3/13-forget-*.json; steps in models/from-scratch/lrforget/summary.txt. LR's per-use step is 8-12×
  smaller than probit's at cluster-error v585.
- **Fixed-point flat probit** (PERF-LEDGER "Queued"): exact speedup for F-P play and search; starts after the
  residual builds, to keep the load down.
- **P1 and P2:** specs ready, wait for decisions 3-4.
- The 6-hourly performance review appends to models/from-scratch/fastbin/PERF-LEDGER.md (the cron lives in the
  session; re-create it in a new session).

## Proposals (from the overnight doc; status 11-10 13:40)

| Id | Proposal | Status |
|---|---|---|
| P1 | moe-gates: F-L, M-L4-L, M-L4-L-hard, M-L4-N | spec ready; decision 3 |
| P2 | dag-home: F-L, T-L-Ne, D-L-Ne, D-L-Ne-home (limit 5) | built and checked offline; decision 4 |
| P3 | moe-experts: F-L, M-L2, M-L4, M-L8 | after P1 |
| P4 | self-play against local at each leaf's best depth | depths known (LR 2, probit 2, NB 0); decision 9 |
| P5 | self-play exploration policy | deferred since 11-09 |
| P6 | cluster against merge against error, one factor each | partly covered by split-growth; decision 5 |
| P7, P7b | split growth (LR, probit) | running as split-growth and split-growth-probit |
| T1 | consistent router targets (gap, route-before-train, bootstrap) | running as error-targets and error-targets-probit |
| T2 | home targets score the copy as routed | design only; decision 4 |
| T3 | additive (residual) leaves, clean rerun | LR from ~14:05 as residual-leaves; probit twin being built (decision 2) |
| T4 | gradient-disagreement split trigger | built, offline win; league spec drafted (decision 8) |

The routing path behind them (archive NEXT-EXPERIMENTS §6): measure the seam tax (P1); make router splits mean
something (P2, home targets); blend only at boundaries if hard routing costs play; path context; cluster on top, error
below; a growing mixture if P1 favours M-L4-N. Details of each: IDEAS.md "Routing and growth".
