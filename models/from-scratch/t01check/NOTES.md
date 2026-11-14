# T 0.1 vs T 0.02 (cold) ratings — 2026-11-12 ~15:30

The user, 11-12 ~14:58, asked: "can we double check that the T = 0.1 games are really that bad for establishing model
elo?"

## Data and method
- The six runs that kept both fits every cycle:
  - fponly-p1-coldrate/coldtrain and treeonly-p1-coldrate/coldtrain: 300 cycles, cold share 0.25;
  - filtered-p1 (cycle 113) and filtered-p1-twin (cycle 157): cold share 1.0, mixed 5-kind fields.
- leagues.py's champion test was replayed on each fit at each cycle's last round:
  - the diff comes from elo_history.csv at row t+1 (its tournament column is 1-based);
  - the SE is contrast_ses on the cumulative pair counts.
- The cold replica reproduced all 232 sampled recorded tests exactly (diff and se_diff).
- Scripts: t01check.py (per test and per entry), t01read.py (Q1-Q3), t01more.py (by kind, line ranks, CPU).
- Outputs: t01check-RUN.csv, t01field-RUN.csv, t01read.out, t01more.out.

## Findings
- **Crownings at 2.0 (T 0.1 / cold / both):**
  - F-P fields: 365/54/45 and 311/71/52;
  - tree fields: 248/105/62 and 192/128/61;
  - filtered pair: 109/61/38 and 101/117/44.
  - Agreement is 72-83% at 2.0 and 81-90% at 2.5.
- **Correlation of the diffs:** r 0.47-0.79 overall; 0.64-0.92 in cycles 1-50; 0.04-0.46 later.
- **Median SE:** T 0.1 19 against cold 38 in the twins; 21 against 21 in the filtered pair.
- **Late uniform fields (cycles 51-300):**
  - Versions T 0.1 would crown are 28-43 better at cold than their champion; versions it keeps are 0-21 better.
  - The bias depends on kind. Per test, T 0.1 reads F-P progress 0-19 higher and tree progress 0-23 lower.
  - In treeonly-coldtrain the tree bias grows with champion age: 4 at a gap of 1 version, -23 at 6 or more.
- **Mixed fields:**
  - After cycle 30 the trees' within-line bias changes sign by seed: 22-30 higher at T 0.1 in filtered-p1, 4-10
    lower in the twin.
  - filtered-p1's T 0.1 crownings after cycle 50 were worth 1.5 ±3.8 at cold.
- **Same kind, between lines:**
  - Per cycle the order agrees 45-56% of the time, which is chance.
  - On cycle 101-300 averages both tree fields have the same order in both fits.
  - The F-P lines are all within 2 of each other.
- **Across kinds (trees minus flats, T 0.1 minus cold):**
  - filtered-p1: -13, -11, -33 over cycles 1-30, 31-60 and 61-90.
  - twin: 2, -28, -49, -62 over cycles 1-30, 31-60, 61-90 and 91 onwards.
  - In the twin T 0.1 reverses the order: trees 35 below flats, while cold has them 27 above.
- **CPU:**
  - Cold games are 2-3% of a ply-1 run at share 0.25 and 7% at share 1.0.
  - Round-robins are 9-11% of a ply-1 run and 17-23% of a ply-2 or ply-3 run (fponly-p2, diversity-p2c, probit-p3).
  - A ply-1 panel arena costs under 1 CPU-min of games; a ply-2 one costs 2-4.
- **Draws:**
  - Draws are 45-73% of cold games and 38-58% of T 0.1 games.
  - contrast_ses uses p(1-p), so it treats draws as full-information games.
  - The late cold diffs spread 32-37 against nominal SEs of 38-39, so the true noise is at least 5-16% lower than the
    nominal SE.
  - The draw arithmetic, sqrt(1-d), allows 30-47% lower. This is unverified.

## Verdict
**(c) Cold games are needed** for crowning and for cross-kind standings.
- T 0.1 is usable only for same-kind, same-age comparisons.
- The cheap fix for big SEs is a larger cold share.

## SE verification (coordinator follow-up, 11-12 15:25)
Script: t01split.py. Outputs: t01split-tests.csv, t01split-arenas.csv, t01split.out. Run time 78 s.

**In-run fit.**
- The in-run fit is the half-point Bradley-Terry MLE. The gradient left at the recorded ratings is at most 2.8 games,
  which is what elo_history's 0.1-Elo rounding produces.
- My nominal SE matches the recorded se_diff to within 0.05 Elo on all 7,334 tests.

**In-run split-half** (alternate cold round-robins; each half refit; history held by its exact Schur complement):
- The split-half SD over its nominal prediction is 0.60 [0.59, 0.61] pooled. By group:

  | Group | Split-half SD / nominal |
  |---|---|
  | F-P twins | 0.49 |
  | Tree twins | 0.66 |
  | Filtered flats | 0.68 |
  | Filtered trees | 0.70 |

- It is the same in cycles 1-50 and later. The literal version, Var(h1-h2)/4 over se_diff^2, gives the same 0.60.
- The draw-aware HC0 SE (observed (s-p)^2 per game) is 0.63 of nominal (0.53 / 0.69 / 0.66 / 0.71 by group). It
  predicts the split-half within 0.02-0.04. There is no sign of cross-round duplicate games.
- Pair-level chi2/dof of the cycles' cold games is 0.27-0.63 with the binomial z and 1.09-1.36 with each pair's
  draw-aware variance, so the matchup excess is small.
- A pair-cluster sandwich gives 0.61 pooled. It is slightly low because the candidate's pair residuals sum to zero.

**Arenas** (28 PANEL contrasts in 6 arenas; my BT fit reproduces fit.md's chi2 of 0.54, 1.11, 1.06 and 1.15):
- Split-half SD over arm_panel's binomial SE is 0.82 [0.61, 1.01].
- The round-level estimate is 0.82 [0.76, 0.84]: about 0.64-0.71 at ply 1 (37-57% draws) and 0.81-0.96 at plies 2-3
  (4-9% draws).
- The within-cell draw-aware estimate is 0.84.
- The arm x panel pairs' chi2/dof is 0.58-1.51 with the binomial z (sqrt pooled 1.04) and 1.12-2.13 with draw-aware
  variance. So matchup excess is present in every arena.

**Implications.**
- Today's 2.0-SE champion test is about a 3.3 true-SE test: 4.1 for F-P, 3.0 for trees, 2.9 in the filtered pair.
- At 2.0 under the HC0 SE the twins would crown 279, 307, 267 and 265 times, against the recorded 54, 71, 104 and 128.
- Panel-read CIs should not shrink: matchup effects cancel in split halves but not in a read about strength.

## In-run rank vs arena rank of classmates (coordinator follow-up, 11-12 ~15:58)
Scripts: t01class.py and t01class_rel.py. Outputs: t01class-pairs.csv, t01class-groups.csv, t01class.out.
Data: 790 classmate groups (same run, same version, same arena ply) in 159 finished arenas, from 168 run-versions and
4,966 pairs.

**Concordance between the in-run order and the arena order:**
- Single version: 55% [50, 60]. Smoothed over 10 versions: 55% [49, 61]. With each run-version counted once: 54%.
- Per-group Spearman: median 0.20, mean 0.11.

**Does the in-run best come first in the arena?** 32% of groups, against 30% by chance (1/k).

**Splits:**
- Same-kind pairs agree 51%; cross-kind pairs 60% [50, 69].
- Arena ply equal to the training ply, or not: 55% either way.
- Gaps above 2 nominal SE (16% of pairs) agree 75% [68, 82]; gaps above 2 draw-aware SE agree 69% [61, 76].
- Smaller gaps agree 51%.

**Ceiling:** two arenas agree on the same classmate pair 62% of the time with run-versions weighted once (78% per
arena pair). The median classmate gap is 15 in arenas and 23 in-run; the in-run SE is 29 nominal, 22 draw-aware.
