# Path forward: routers, DAGs and the ply question

Written 2026-11-10 ~19:00 while the user was away. They asked me to "continue thinking about [ply 1 against ply 2]
as well as trying to synthesize a path forward for our research into routers and DAGs". The numbers come from
RESULTS.md (sections 1, 3-6 and 8), with the corrections of ~18:40: the table tool had counted 5 rounds a cycle
(RESULTS section 1). Nothing proposed here has been launched.

## 1. What changed today

1. **Models get good at the ply they train at.**
   - selfplay-depth, played at ply 1: the d1-trained models beat the d2-trained ones by 238-321 (LR and probit).
     Played at ply 2, d2 leads by 146-192.
   - lr-settings pair arena: the rerun (ply-2-only field) beat the original by 424 at ply 2 and by 36 at ply 1.
     probit-settings (19:38): 754 at ply 2 and 111 at ply 1. NB keeps most of its gain at ply 1: nb-settings 94
     and 69, nb-settings-no-random3 58 and 23 (19:49-19:51).
   - So "deeper data is better" holds for ply-2 play only.
2. **Ply 1 and ply 2 rank the same versions differently.**
   - Over 1,328 comparisons in 18 runs, ply-1 gaps are about half the ply-2 gaps.
   - 14% of clear ply-2 verdicts reverse at ply 1.
3. **NB and TAN cluster trees with LR leaves beat flat at ply 1 in every run.** Ply 2 is mixed. Last quarter of each
   run, tree minus F-L:

   | Run (seed) | Tree | ply 1 | ply 2 |
   |---|---|---|---|
   | single-split (1) | T-L-Nc-merge, T-L-Tc-merge | 43, 64 | 81, 83 |
   | cluster-error (1) | the four NB/TAN cluster trees | 7 to 25 | -36 to -14 |
   | split-growth (2) | the four NB cluster trees | 33 to 41 | 8 to 45 |
   | residual-leaves (6) | copy tree, limit 4 | 73 | 6 |
   | trigger-gradient (9) | T4, today's trigger (limit 8) | 48, 27 | 3, -8 |
   | small-field (10), final (303-400) | T-L-Tc-merge | 47 | 28 |

   - Ply-1 Elo is compressed about 2×, so 40 at ply 1 is roughly 80 on the ply-2 scale.
   - Probit trees agree across plies, and their ply-2 gaps are the larger. single-split-probit's cluster trees are
     15-26 at ply 1 and 65-66 at ply 2. T-P-Ne is 0 at ply 1 and 48 at ply 2.
4. **Corrected batch-3 results.**
   - The single-split LR gain half replicated in split-growth: 35-43 at the end, still rising.
   - Growth to limit 4 with merges ended furthest ahead (34-53).
   - Gap targets recovered 29-80 of Best's loss.

## 2. Where routing stands (at ply 2 unless stated)

- **Cluster routers** (NB, TAN) find the game-phase partition: opening, queenless middlegame, endgame.
  - The gain is slow: it costs 70-150 at first and pays after 40-120 cycles.
  - Its size varies by run: LR leaves -36 to 83, probit leaves 2 to 66.
  - Field size (small-field, 2 lines, final 19:39): the tree led F-L by 20-29 over its last 150 cycles at ply 2. The
    same recipe in the 7-line single-split led by 64-118 at the same cycles. At ply 1 the two runs are close (42-47
    and 50-62).
  - LR cluster routers churn and are flat in effect.
- **Error routers.**
  - LR error routers never beat flat. Gap targets bring them within 19-29.
  - NB error routers route like clustering. With probit leaves they work: 48 in single-split-probit, and 39-48 from
    cycle 500 to the end in split-growth-probit (final, 1000 cycles), at ply 2 only: level with F-P at ply 1.
  - Probit leaves with LR error routers ended 104-143 below F-P at ply 2 (36-84 at ply 1) whatever the targets;
    gap targets were 20-30 worse than Best there (error-targets-probit, final).
- **Growth.**
  - LR at limit 4 with merges ended 34-53 ahead, in one run.
  - Probit at limit 4 stayed 17-62 behind to cycle 1000 (26-47 at ply 1).
  - T4 and residual leaves gave nothing in play.
- **DAG router splits** never beat a same-leaves tree offline (5 studies, 7 variants). The measured routing headroom
  is 0.001-0.003 nats. Path context isn't linear in growth.
- **Nothing is replicated across independent runs.** One recipe's gap to flat moves 50-90 between runs.

## 3. My reading

- The routing gain is real for the static evaluator: every LR cluster tree beat flat at ply 1.
- At ply 2 the search exposes something in LR-leaf trees that probit leaves resist. Three candidates, each testable
  without training:
  - **(a) Calibration.** cal-own fits one slope on the version's own ply-1 games. A tree's leaves may need different
    scales, and ply-2 values (minima over replies) are distributed differently from ply-1 values.
    - Tested 11-10 ~19:15-19:30 (RESULTS section 1): not the general cause. Over 15 tree-vs-flat gaps,
      recalibrating on ply-2 games changed the gap by ~4 on average. It is noise of about ±30 per comparison: one
      gap moved 71, two about -70.
  - **(b) Search exploits uneven noise.** The minimum over ~30 replies is pulled down most where the evaluator is
    noisiest. A tree with a data-poor, noisier leaf then steers its games away from that leaf's positions.
    - Probit's posterior shrinkage damps noise where data is thin, which fits probit trees keeping their gain.
    - It also fits LR trees catching up late, as their leaves fill (split-growth limit 4, residual-leaves'
      copy tree).
  - **(c) Positions that never occur in games.** A ply-2 search scores ~30× more positions reached only by bad
    replies. A router can send those to the wrong leaf; flat LR has nothing to misroute.
- If (a)-(c) hold, the fix lies in the leaves and the calibration, not in more routing structure. Options:
  per-leaf calibration, shrinkage toward the parent or a shared base, probit-style uncertainty.
- DAGs come after that: a router-split refinement can't be measured until the base tree gain is robust at a fixed
  ply.

## 4. The ply question

The user's proposal: run experiments at ply 1 because it's cheaper, and dip into ply 2 now and then to see the gain.

- **For:**
  - Ply 1 measures the evaluator alone, which is what routing research builds.
  - Its tree verdicts were consistent across runs, where ply 2's swung.
  - A cycle costs less. Not the 12× of a single game, because training, calibration and reports don't shrink:
    - flat LR (lr-settings-new: games 10.7 of ~26 CPU-s a cycle): ~35-40% less;
    - tree fields (small-field-x4's estimate: tournament ~47 of ~66): ~60% less.
- **Against:**
  - Models specialise to their ply, so a ply-1 experiment builds ply-1 players.
  - The dip is not a constant offset. A ply-1 screen would have dropped our best probit result (T-P-Ne: 0 at ply 1,
    48 at ply 2) and promoted the LR gap-error trees (27-47 at ply 1, -18 to -31 at ply 2).
  - The eventual player searches.
- **Untested:** all of the above measured, at ply 1, models that trained on mixed games. The old field was ~40% ply 1
  against ply 2, ~37% ply 2 against ply 2 and ~23% against random. No league has trained and measured at ply 1
  together.
- **Recommendation: test it once, on the comparison that matters.**
  - The test: a ply-1 twin of small-field-x4 (4 seeds each of F-L and T-L-Tc-merge), with its field and training at
    ply 1 and the same rounds, so ply is the one factor.
  - Then arenas of both twins' champions, at ply 1 and at ply 2.
  - If trees beat flat at both plies, use ply 1 as the cheap screen and confirm winners at ply 2.
  - If they beat flat only at ply 1, work on (a)-(c) before growth.
  - It needs an engine option for the entries' ply: main entries at ply 1, training games with a model at that ply,
    and the rating zero and champion test at it. The option must be byte-identical for existing configs.
  - Cost: ~27 CPU-s a cycle against ~66 at ply 2, so ~2-4.5 CPU-h for 300-600 cycles.

## 5. Proposed sequence (each step waits for the user's OK)

1. **A ply-1 replay of every batch-4 pair arena.** ~70-150 CPU-s each, ~25-50 CPU-min for all 18. The pair watcher
   would run it after each ply-2 pair, so every pair reports both plies. lr-settings' replay already ran (~18:48,
   71 CPU-s, without asking). probit-settings' ran at 19:39 (43 CPU-s): 754 at ply 2, 111 at ply 1. The two NB
   pairs' ran at 19:50-19:51 (51 CPU-s each): 94 and 69, 58 and 23.
2. **Diagnostics on existing champions** (arenas and offline only, minutes each):
   - (a) Done ~19:30: calibrated on their own ply-2 games, 20 champions of five runs. Not the cause; noise of about
     ±30 per comparison (section 3).
   - (b), (c) offline: each leaf's noise, and the tree's error on searched off-game positions against flat's.
     These need a small evaluation tool. They are next if the user wants the mechanism.
   - The user said "continue the tests" (~19:05). The ply-1 replays (a watcher, pid 1860987) and diagnostic (a) are
     running or done.
3. **Routing reruns first.** After the old runs end (~22:00), either give batch 4 a second lane for the routing
   experiments or move them up the one lane: single-split, single-split-probit, split-growth, split-growth-probit,
   then small-field-x4. Their pure-ply-2 verdicts and ply-1 replays are what this path needs. The stage-1 reruns
   (settings, leaf-sources) can wait. The user asked for one at a time, so this is their call.
4. **The ply-1 twin of small-field-x4** (section 4).
5. **Then, depending on 2-4:**
   - If trees hold at ply 2: growth. LR cluster trees with merges at limits 4, 8 and 16 (merges made the difference
     in split-growth), 2 seeds each.
   - If trees hold only at ply 1: leaf and calibration fixes first (per-leaf calibration, shrinkage to a shared
     base), tested at ply 2.
   - Error routing: gap targets with the bootstrap were still closing on flat at cycle 400 (-54 to -19).
     error-targets-new runs them to their plateau.
6. **DAGs parked** until step 5 shows a robust tree gain. Then: home-target router splits (P2) on the best tree
   recipe, and the linear path-context variant (cells for the parents' sets only).

## 6. Measurement rules to add (NEW-DESIGN rules)

- State the ply with every verdict, and report ply 1 beside ply 2 where it costs nothing.
- Replicate a tree-vs-flat gap before concluding: one run's gap moves 50-90 between seeds.
- Tables only from the fixed tools, which take the rounds per cycle from config.json.
