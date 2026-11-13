# Training standard (v1; ply-1 rung LAUNCHED 11-12 17:31: std-v1-p1-s1 seed 65, std-v1-p1-s2 seed 66; ply-2 rung DRAFTED 11-12 ~19:00, pending decision 2)

The user, 11-12 ~16:38: "I feel like we need to keep track of what we consider to be 'current state of the art'
conditions for having trained a specific model recipe to its plateau under repeatable conditions (so not talking about
filtered experiments, but some kind of experiment setup) at a specific ply. That we can maintain a more organized
library of models to choose from to run arena tests and put into the global league tournament (or do further training
on). That way we can do more decisive clean ups and etc."

Then, ~17:00: "First let's establish the standard for ply 1. then we can move onto ply 2".

## What the standard is for
- Each recipe has reference models per ply, trained to their plateau under the standard below. The library tags them
  "standard vN, ply P, seed S".
- Replicates are kept. The user, ~16:45: "we may want multiple version of the same model per ply if somehow we ran
  the same experiment multiple times and they were both trained to the same standard. It can be useful to have
  multiple versions because it helps us calibrate noise (probably)". So every replicate run (another seed) of the
  same standard version is kept, and replicates measure run-to-run noise directly, per recipe and ply.
- Arenas, the global ladder and further training pick from these references and the curated holdouts.
- Cleanup rule: keep the standard references, the holdouts, and whatever an arena or read still uses. Everything
  else is a delete candidate, approved by the user tier by tier.
- When a finding changes the standard, the version goes up. Older references keep their version tag until a new run
  replaces them.

## Standard v1, one rung per ply P

| Setting | v1 | Why (finding) |
|---|---|---|
| Field | Mixed ("diverse class"): the 8 standard recipes, 1 line each (decision 1, agreed) | The user, 11-12 ~13:25: hybrid fields for the strongest models. Uniform fields didn't help (11-12 12:00). |
| Ply | Every game at P (entry_ply P) | Training ply dominates: 176-224 at the home ply, ~185 head to head. |
| Start | Ply 1 from scratch. Ply P >= 2 warm from the ply P-1 standard finals, each line from its own predecessor (decision 2). First training: start_forget 1 for probit, a one-off 10 for LR (flat `--init-forget`, trees' leaves `--start-leaf-forget`); growing trees and DAGs `--warm-reset` (6.95; see the ply-2 rung) | W4: 173 at ply 4 from a warm start. From scratch at ply 3+ costs ~10x. Tree and DAG warm starts are built (leagues.py 6fb93c92); leaf forget and warm reset in chessai-grow7. |
| Training games | Round-robins + self-games at T 0.1 (log-odds softmax) | T 0.02 training games cost 54-138 (redo 11-12). |
| Games a cycle | 1,584 at plies 1-2, 720 at plies 3+ | The user's games-per-cycle standard. 720 halves deep rungs (6.84). |
| Rating | Rate every 4 cycles. Cold set T 0.02, share 0.5, rating-only | Sub-cycles (the user, 11-12). Cold games in training failed (11-12 14:21). T 0.1 can't crown (t01check). |
| Champion test | Draw-aware (HC0) SE, crown at 3: `"champion_se_kind": "draw-aware"`, `"champion_se": 3` | In-run nominal SEs are ~1.7x the split-half noise (t01check). 3 draw-aware SEs is ~1.8 nominal, today's false-crowning rate (6.93). Built, verified and swapped in 11-12 17:22:24 (leagues.py dfba7832; leagues/BUILD.md "Draw-aware champion test and plateau guard"). |
| Plateau | Scaled, rate_scale sqrt (DECIDED, the coordinator ~17:15), with `guard_z` 1.0. Min 300 at plies 1-2 from scratch, 41 for warm rungs. Cap 601 / 101 (rated cycles) | sqrt: 2x the training-cycle window under rate_every 4. Guard (6.94): never stop while any line's last rated test has current minus champion above 1 draw-aware SE. |
| DAG router splits | `--router-chain 2`, `--router-trial-z -2`, router credit at the recipe's 10M | See "DAG settings" below. |
| LR leaves | `--forget-factor 10` on every LR line that splits. The LR grow tree and LR DAG take grow-l-p1b's leaf rule: `--grow-mdl 0`, gate age, priority snapshot | See "LR settings" below. |
| Not in v1 | Train filters, observers, openings, cold games in training | Experiments, not the standard. |
| Binaries | Training: chessai-hybrid6 for flats and capped trees, chessai-grow6 for grow trees and DAGs; warm rungs chessai-grow7 for the capped LR tree and the growing lines (= grow6 without its two flags, = hybrid6 on capped trees). Tournament: chessai-hybrid19 | Byte-identical swaps. hybrid19 plays grow6's DAGs byte for byte as grow6 at plies 1-4 (ply4-prep). grow7: fastbin/grow7/summary.txt. |
| Seeds | Two seeds per rung where affordable (plies 1-2); every replicate seed is kept (decision 3) | Run-to-run SD ~35-45, so one run resolves ~90. Replicates calibrate that per recipe and ply. |

## The 8 lines (ply 1)
1. F-L
2. F-P
3. T-L-Ne capped: `--split-limit 4 --split-threshold 1000000 --forget-factor 10`
4. T-P-Ne capped: `--split-limit 4 --split-threshold 1000000 --split-noise 0.03`
5. T-L-Ne-grow-f10 (forget 10 in the row): `--grow-mdl 0 --grow-gate age --grow-priority snapshot`
6. T-P-Ne-grow: `--grow-mdl 1 --grow-gate age --grow-priority snapshot`
7. D-L-Ne-ctx-grow: `--router-chain 2 --router-trial-z -2 --grow-mdl 0 --forget-factor 10`. This is a new recipe
   row, D-P-Ne-ctx-grow with LR leaves and no split noise (make_recipes.py HAND, recipes.txt, 11-12 ~17:05).
8. D-P-Ne-ctx-grow: `--router-chain 2 --router-trial-z -2`

The longest line name is 80 characters with `.loc`, under the ~100 that broke self-games.

## DAG settings (v1)
The user, ~17:00: "I looked at some of the DAGs and i feel like maybe they split a little too liberally? I don't know if
we have a good way to controll it, but i guess just pick some values".

What grow-dag-p1 showed (router-chain 2, router credit 5M, trial rule "undo only if 2 SEs worse"):
- The DAGs reached 63-76 nodes by cycle ~430 (46-57 routers for 15-19 leaves). The grow trees had 22-25 nodes.
- Of 18 router trials, 2 were undone. 14 of the 16 kept splits passed by 3+ SE.
- One unsharing made 9 routers at once.

Hence:
- **`--router-trial-z -2`.** A router split is now kept only if its trial gain over the unsplit router is at least
  +2 block SEs. Before, it stayed unless it was 2 SEs worse. The bar trims marginal splits.
- **Router credit at the recipe default, 10M root positions.** grow-dag-p1 used 5M, so this halves the opportunities.
- **`--router-chain 2`.** Analyst 6.94: chain 1 ends router growth once each router has split, which drops the
  long-horizon case the user reopened.
- **Prediction (6.94):** 35-45 nodes at v300 and 55-70 at v600, against ~90+ at grow-dag-p1's pace.

## LR settings (v1)
- **LR leaves split at `--forget-factor 10`** (analyst 6.59/6.67-6.69).
  - At forget 1 a split's clones keep the parent's AdaGrad sums, step at its late-life rate and can't specialise.
  - grow-l-p1b: forget 10 doubled the LR split gains and kept 6-7 of 12-15 trials.
  - capped-l-f10-p1: the capped LR tree on forget 10 was 45-49 over F-L at ply 2.
- **The LR grow tree and the LR DAG grow leaves with the MDL bar off (`--grow-mdl 0`), as grow-l-p1b settled.**
  - An MDL bar at 1 is ~0.006 nats for these leaves (k 4262).
  - LR splits gain 0.0007-0.002 nats, so the bar would block every LR leaf split.
  - Probit splits gain ~0.01 nats and keep the bar on.
- **So each DAG differs from its leaf type's grow tree by the DAG features only** (path context, router splits).

## The ply-1 rung: std-v1-p1-s1 and std-v1-p1-s2 (LAUNCHED 11-12 17:31; leagues/experiments/std-v1-p1-s1, -s2)
- **Specs:** leagues/specs/standard/std-v1-p1-s1.json (seed 65) and -s2 (seed 66), from the .DRAFT files. They are identical but for
  the name and the seed.
- **Settings:**
  - Ply 1, 1,584 games a cycle: 16 entries, 7 double round-robins, 1,680 games.
  - Self-games.
  - rate_every 4.
  - Cold T 0.02 at share 0.5: 720 games a rated cycle, rating only.
  - Draw-aware crowning at 3.
  - Plateau: K = 50 training cycles (25 x sqrt 4), checked on rated cycles from 300, guard_z 1.0. Cap 601.
- **Create, then pin the training binaries before cycle 1:**
  - `train_binary.py --from chessai-hybrid4gfx LDIR chessai-hybrid6` (F-L, F-P, both capped trees);
  - `--from chessai-grow4 LDIR chessai-grow6` (T-P-Ne-grow);
  - `--from chessai-grow5 LDIR chessai-grow6` (T-L-Ne-grow-f10).
  - The DAG rows are on grow6 already.
- **Cost:** ~19 CPU-s a cycle early, ~33 by cycle 300 and ~45 by 600, from measured per-line rates. That is ~2.2
  CPU-h to the floor (cycle 301) and ~5.7 to the cap per seed (±30%), or ~4.4 / ~11.4 for both.
- **Scratch check (models/from-scratch/std-v1/createcheck):** both specs created on the new driver and pinned; the
  previous driver (6fb93c92) refuses them (plateau.guard_z). s1 ran 81 cycles there (17:16-17:29, 1,329 CPU-s;
  11.7 CPU-s a cycle at first, 21.5 by cycles 72-81):
  - Only cycles 1, 5, ..., 81 rated. All 8 lines crowned on rated cycles 5-17, then 0-4 lines a rated cycle (0-2
    from cycle 45).
  - The draw-aware SE was 0.54-0.85 of the nominal one (median 0.69, steady from cycle 13). So the bar of 3 was
    ~2.1 nominal SEs here, a little stricter than the ~1.8 the analyst assumed.
  - The DAGs' first router split (cycle 48, at 10M positions) was kept at its trial (cycle 71): z +19.4 (LR) and
    +3.2 (probit) against the +2 bar. Both DAGs had 13 nodes at cycle 81 and the grow trees 10.

## The ply-2 rung: std-v1-p2-s1, -s2 (warm) and -x3 (scratch) (DRAFTED 11-12 ~19:00; PENDING DECISION 2)
Analyst 6.95's design, drafted by the ply-2 prep fork. Nothing launched.
- **Runs.** Warm is the standard: std-v1-p2-s1 (seed 67) and -s2 (seed 68), each line from its own seed's ply-1 final
  (std-v1-p1-s1, -s2). Plus ONE scratch run on a third seed, std-v1-p2-x3 (seed 69), a one-off calibration arm.
- **Specs:** leagues/specs/standard/std-v1-p2-{s1,s2,x3}.DRAFT.json, written by
  models/from-scratch/std-v1-p2/prep/make_drafts.py from the ply-1 spec, so the 8 recipes and every shared setting are
  the ply-1 rung's token for token. Changes: every game at ply 2 (`entry_ply` 2, `ply1_entries` false), names, seeds,
  and each line's training binary in its entry (`"train_binary"`: chessai-hybrid6 for F-L, F-P and the capped probit
  tree, chessai-grow7 for the capped LR tree and the four growing lines; the same in all three runs, so no
  train_binary.py step).
- **Warm starts (s1, s2).** Version 0 is a byte copy of the ply-1 final (trees: model and training state).
  - First training: start_forget 1 for the probit lines; a one-off forget 10 on the LR lines (F-L `--init-forget 10`;
    the LR trees' and DAG's leaves `--start-leaf-forget 10`, routers untouched).
  - The growing trees and DAGs keep their structure and growth state (credits, closes, back-off, router chain), with
    `--warm-reset`: leaf trials in flight closed on their statistic so far (kept when fewer than 2 blocks have
    finished); DAG router trials closed under their own keep rule (kept with no finished window); windows, snapshots,
    router shadows, null shadows and pending post-split checks restarted; path tables carried over.
  - Plateau min 41, cap 101 (both rated cycles).
  - The drafts' start version is null (create refuses it). Once std-v1-p1-sN is done,
    `python3 leagues/specs/standard/fill_p2_starts.py sN` writes std-v1-p2-sN.json with its final version, after the
    driver's own create checks.
- **Scratch (x3):** no start; plateau min 300, cap 601, as the ply-1 rung. 6.95's reads compare at v75, v150 and the
  plateau, which gives no reason for another floor.
- **Shared:** 1,584 games a cycle (7 double round-robins, 1,680 games), self-games, T 0.1, rate_every 4, cold games T
  0.02 share 0.5 rating-only, draw-aware crowning at 3, plateau scaled sqrt (K 50) with guard_z 1. Tournament
  chessai-hybrid19.
- **Read (leagues/specs/standard/std_p2_read.py, draft):** warm against scratch per recipe and pooled, on a stranger
  fit (a Bradley-Terry fit on games against the panel only; the in-run cells are matchups), Elo 0 = the panel's mean.
  - **Early read:** warm v75 (or its final if it stops earlier); scratch v75 and v150.
  - **Final read:** the three finals.
  - **Panel:** y/v/l @299, the ply-2 standing panel. The early read adds the ply-1 standing panel (r/e/g/h @300, at
    ply 2) as a lower tier.
  - **Why this panel (6.93: within ~150 of what it judges):**
    - The ply-2 fields at ~v299 sat within ~60 of each other (panel-p2fields-p2-v299).
    - Ply-2-trained LR is at that level too (ladder-1: base-lr-p2's F-L @599 against y/v/l).
    - Ply-1-trained models sit 150-500 below at ply 2 (p3p2-p2-v301), hence the lower tier for the early read.
  - The script flags every entry that has no panel member within 150.
  - It reports run noise from s1 - s2.
  - Cost: ~0.4 CPU-h for the early read, ~0.3 for the final.
- **Prediction (6.95):** warm = scratch within 25 at the plateau, and warm reaches scratch's v150 level by v50-75.
- **Rule:** if scratch is 30+ over warm at the plateau (pooled), rungs start cold; otherwise warm is the rule.
- **RESULT (11-13 05:10, the coordinator; arena diag/std-v1-p2-read-final): WARM STAYS THE RULE.** Scratch - warm pooled +18 [+6, +29] (sampling), below 30. Warm runs ended at their cap 301 still crowning; scratch plateaued at 437. Seed noise: s1 - s2 pooled -35 [-48, -22], per-line run SD ~28 at ply 2 (models/NEXT-EXPERIMENTS.md, same time).
- **Cost (measured at ply 2, models/from-scratch/std-v1-p2/proof):**
  - ~0.024 CPU-s a game, so ~41 CPU-s of tournament a cycle.
  - Warm: ~80 CPU-s a cycle, ~0.9 CPU-h to the floor and ~2.3 to the cap per run.
  - Scratch: ~6 CPU-h to its floor and ~13 to its cap.
  - Whole rung: ~8-18 CPU-h (±30%).
  - The scratch arm is most of it.
- **Proof** (models/from-scratch/std-v1-p2/proof, 19:00): w2proof = s1's draft warm from a frozen
  std-v1-p1-s1@340, at ply 2, 5 cycles. replicate_w2.py ran 35 checks, failures 0:
  - every version 0 equals its source;
  - cycle 1 carries exactly each line's flags, and cycle 2 carries none;
  - every version 1 equals its cycle-1 training replayed from the source;
  - every control without the flags differs;
  - without the flags, chessai-hybrid6 (capped LR tree) and chessai-grow6 (growing lines) equal grow7.
- **What the warm reset did** on the real ply-1 models:
  - D-L: 2 router trials judged (kept, z +6.4 and +11.6) and 1 kept without a window; 1 leaf trial kept by its
    verdict and 1 without one; 43 nodes restarted.
  - D-P: router trials kept at z +3.0 and +58.4, and 1 without a window; 1 leaf trial undone and 1 kept without a
    statistic.
  - The grow trees: all 3 leaf trials undone (closing early mostly undoes; the credit is refunded).
  - The LR leaf forget hit 9 (capped tree), 15 (grow tree) and 17 (DAG) leaves.
- **Scratch smoke run** (x3smoke): x3's draft for 5 cycles at ply 2, which ran.

## Decisions
1. **The standard recipe set.** AGREED by the user ~16:55: "F-L F-P capped trees, growing trees and router-splitting
   dags (but all of those might have either LR or probit leaves) so that's 8 entries?" One line each per run.
2. **Rungs above ply 1:** warm from the previous rung (cheap, a ladder) or from scratch (clean, ~10x the cost at ply
   3+). The user: ply 1 first, then ply 2. PENDING. The ply-2 rung is drafted to settle it with data (6.95): two warm
   runs plus one scratch calibration arm. If scratch is 30+ over warm at the plateau, rungs start cold. Launch after the
   ply-1 runs end, with the user's go.
3. **Seeds:** two at ply 1 (the coordinator, ~17:15). Replicates are kept whenever run.
4. **Plateau window:** sqrt, DECIDED (the coordinator, ~17:15), with the guard at 1.0.
5. **Crowning:** draw-aware SE at 3, adopted for v1 once the build was verified. The fallback would have been
   champion_se 2.0 nominal, about the same bar on average but kind-biased.

6. **Search labels at ply 1 (ADOPTED by the user, 11-13 ~12:35: "sure, we can adopt it for ply 1").** The ply-1 rung's F-P trains with `--label-depth 2 --label-result 0.5` (labeldepth-p1, two seeds, version windows: +43-58 at ply 1, +53-57 at ply 2, +38 at ply 3 over result labels; recipe row F-P-lab). F-L stays on result labels until a short F-L test (labels untested for LR). Ply 2 and up stay on result labels (depth 2 at ply 2: ~+26 +- 16 over four seeds, below the 40 line; depth 3 closed). Trees: see the distillation result (labels-tree-p2), not adopted.

## v2 candidates (not in v1)
- **DAGs without path cells, keeping unsharing by path (analyst, ~17:15).**
  - The nocells arena found zeroed cells change nothing in play: 2 [-11, 15] over plies 1-2.
  - So cells-off would be simpler and faster at ply 4.
  - It needs a build (a training option with no cells) and its own test.

## Runs that come close today (pre-standard, kept as references until replaced)
- **Ply 1:** filtered-p1-twin. 5 recipes x 1 line, 1,584 games, cold share 1.0 every cycle, nominal SE; plateau at 380.
- **Ply 3:** probit-p3. F-P x2 + T-P-Ne capped x2, 720 games, no cold games (T 0.1 crowning); plateau at 302.
- **Ply 4:** probit-p4w. Prepared, not launched: warm from probit-p3, plus 2 router-splitting DAGs.
