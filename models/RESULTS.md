# Results

Every experiment and study since 2026-11-05, grouped by topic. Each topic opens with where it stands. Its entries
follow, newest first, and each gives the date, the design, the numbers and the conclusion.

**Conventions**
- Elo is at depth 2 (ply 2) unless noted, with no plus signs. [a, b] is a 95% interval.
- `fs/X` is models/from-scratch/X, the raw write-up, usually `summary.txt`.
- `exp X` is leagues/experiments/X.
- `archive X` is models/archive/2026-11-10/models/X, the verbatim originals. Full tables and intermediate checks are
  kept there.
- Superseded results are marked as such, with the reason.

Topics:
1. Measurement and the rating method
2. Leaf types: LR, probit, NB
3. Routing and compound models
4. Splits, merges, triggers and growth
5. Error routing and router targets
6. Self-play depth, data sources and league data
7. League runs and experiment batches
8. Arenas
9. Speedups
10. Theory
11. Early results (2026-11-05/06)

---

## 1. Measurement and the rating method

Where it stands:
- **Current against current.** Compare an experiment's lines by direct games between their current ply-2 versions
  (fs/analysis-2026-11-10/xtab/curtab.py). Version windows of head_to_head.csv also count a line's champion, a
  selected version, so they flatter that line.
- **Ply 1 and ply 2 are different measurements** (11-10 ~18:30, entry below and section 6). On the same versions,
  ply-1 gaps are about half the ply-2 gaps, and in 14% of clear ply-2 verdicts ply 1 says the opposite. Models get
  good at the ply they train at. State the ply with every verdict.
- **Noise.** A single version's strength swings about 17 Elo (SD) on top of the match range. Windows of versions
  drift about 30. So a 20-40 Elo contrast needs two or more separate windows. Effects of 100 or more stand alone.
- **Offline is no guide to play.** Offline log-loss did not predict play anywhere it was checked. For offline
  comparisons meant to predict play, use the winner loss, not the total.
- **Calibrate on the version's own games** (cal-own). A fixed held-out calibration biased ratings by about 132 Elo per
  unit of ln(slope ratio).
- **Architecture verdicts come from a panel of strangers, not the within-run pair** (11-12, analyst 6.68). A tree
  against its own flat in the same run carries co-adaptation (they trained on each other's games) and the
  kind-vs-kind matchup; such cells sat 40-70 off the strangers' view. Score each contrasted entry against lines of
  fields it never trained against (same version and ply) and difference the results
  (leagues/specs/arenas/arm_panel.py, panel_read.py). Same-kind cross-run cells carry lineage matchups too
  (25-45; 49 between two sibling runs), so one-factor verdicts are read against the panel as well, with the
  direct cell beside it as the matchup (analyst 6.69).

Entries:
- **2026-11-12 ~10:50, the cohort read: how far identically built, plateaued models deviate within a set-up and
  across set-ups.** This is the user's question, on analyst 6.77's design. Code: leagues/library/cohort_read.py.
  Arenas: leagues/arenas/diag/cohort-fp-p2 and cohort-fp-p1; their analyze.out holds the full tables.
  - **Entries.**
    - Plain F-P lines only (--use-probs, no overrides) from plateau-ended runs. One arena per training ply, played
      at that ply; T 0.02; hybrid17.
    - Ply 2: 31 lines of 12 runs in 11 set-ups. diversity-p2c and p2d are pooled as one run (identical to v9).
      85 entries, 67 round-robins, 478k games.
    - Ply 1: 26 lines of 13 runs in 11 set-ups, plus the global league's F-P (flagged, not pooled). 72 entries,
      60 round-robins, 307k games.
    - Versions per line:
      - v299 and v279 (equal length), plus the final when it is past 299;
      - for runs that stopped below 299: the final and final-20, flagged short.
  - **Ratings.** Bradley-Terry over games between unrelated runs only. Dropped pairs:
    - same-run pairs;
    - shared lineage: fponly-p2/-720 share v1, probit-late-p2/replay-p2 share v1, and diversity-p2c+d share
      base-probit-p2's lineage through frozen references.
    Games between seed replicates are kept.
  - **Components.** Method of moments, with measurement error subtracted. Intervals: parametric bootstrap; the seed
    term has a chi-square interval (1-2 pairs).

    | Level (SD, Elo) | Ply 2 | Ply 1 | Analyst 6.77 (ply 2 / 1) |
    |---|---|---|---|
    | Version (v299 vs v279), total | 11 [8, 18] | 8 [5, 14] | 10-15 / 10-15 |
    | - independent per line | 5 [4, 11] | 0 [0, 1] | |
    | - shared by the run's lines (field drift) | 10 [7, 22] | 9 [6, 15] | |
    | Replicate line within a run | 7 [5, 16] (11 runs) | 0 [0, 2] (13 runs) | 5-10 / 5-10 |
    | Seed (run within set-up), equal length | 0 [0, 120] (1 pair: -5 +-2) | 21 [11, 130] (2 pairs: -16 +-3, -38 +-3 *) | 20-30 / 25-40 |
    | Seed, at the finals | 27 [12, 870] (fponly-p2 v320 54, fponly-p2b v352 16) | 19 [10, 122] | |
    | Between set-ups, equal length | 91 [65, 160]; 48 [32, 89] without leaf-sources-new | 9 [7, 17] | 30-45 / 20-35 |
    | Between set-ups, at the finals | 82 [57, 145]; 18 [12, 34] without leaf-sources-new | 16 [11, 28] | |
    | Matchup (direct line-pair cells vs BT, beyond noise) | same run 0, lineage 6, unrelated 13 | same run 0, unrelated 0 | 25-35 / 25-35 |

    \* probit-widen-p1b/p1c stopped at 199 and 250, so that pair is confounded by length. The clean ply-1 pair is
    grow-p1b/p1b2: -16 at equal length, -20 at the finals.
  - **The answer: same set-up, near-interchangeable models.**
    - The F-P lines of one run agree within ~7 at ply 2 and ~0 at ply 1.
    - Seeds of one set-up agree within 5 at ply 2 (fponly-p2/p2b at equal length) and within 16-20 at ply 1
      (grow-p1b/b2).
  - **Across set-ups, the spread at equal length is mostly learning speed.**
    - At v299, set-ups spread 48 at ply 2 (without the 11-10 run) and 9 at ply 1.
    - At the finals (each run where its plateau rule stopped it), they spread 18 at ply 2 and 16 at ply 1.
    - So fully plateaued F-Ps from different set-ups land within about 20 of each other, roughly one seed-to-seed
      difference.
    - The exception is leaf-sources-new (the 11-10 system's batch 4; F-P.loc in a field with self-play lines): -270
      at v299 and -245 at v319. Its cause was not examined.
  - **Version noise is mostly the run moving together.** A run's lines drift together between v279 and v299 (SD
    9-10); the part independent per line is 0-5. Picking one version of a run costs ~10, and no extra per line.
  - **No co-training matchup.** Within-run F-P/F-P cells show none at either ply (0 against the analyst's 25-35).
    The 40-70 matchups seen before were KIND pairs (tree vs flat), not co-training as such. Unrelated F-P pairs carry
    13 at ply 2 and 0 at ply 1.
  - **Known effects at ply 2** (equal length; SEs are measurement only):
    - Own-past replay: -106 +-3 against its seed twin probit-late-p2 (6.77: -117). But replay-p2 gains 128 from v299
      to its final v542 and ends level with the field (-5 against a mixed core of 17). Replay slows learning; it
      does not lower the plateau.
    - Pure minus mixed: 15 +-1 (lines pooled; mixed without replay and the 11-10 run). Set-up means: 24 vs 3 at
      equal length, 37 vs 17 at the finals. The analyst predicted 60-95, which fails. The 43-95 measured before was
      against panels with trees and F-L; against an all-F-P panel the pure-field lead is ~20.
    - Frozen references: diversity-p2 is 10 +-2 over the other mixed runs, and diversity-p2c+d 27 +-2 over
      base-probit-p2 (6.77: ~0).
    - 720 games a cycle: -22 +-2 at equal version (it has seen fewer games), and level at the finals (35 vs 35,
      v559 vs v320/352).
  - **Training length** (final minus v299):
    - ply 2: 0-46 for most runs (base-probit-p2 v599 40, fponly-p2-720 v559 32, probit-late-p2 v399 35,
      fponly-p2b v352 -10); replay-p2 128;
    - ply 1: -18 to 31 (error-growth-p1 v449 31).
  - **The global league's F-P** (ply 1, flagged): its best champion v463 is 3 +-4 against the ply-1 cohort's mean,
    and v365 is -16. 5,918 cycles of the co-evolved league gave an F-P level with the plateaued experiments' F-Ps.
  - **Cost:** 1.84 CPU-h (ply 2 1.77, ply 1 0.07). Ply 2 overran because the ci15 stop never triggers; see the next
    bullet.
  - **Tool bug: the ci stop with a random anchor** (make_split_read.py ciX; library.py). With one random reference
    as the anchor, the tournament rates no entry: every entry beats random in every game, so none is linked both
    ways to the anchor. Every interval stays infinite and the stop runs on to max_rounds (200). cohort-fp-p2 played
    67 round-robins before it was stopped by hand; the read needed ~15.
    - library.py now refuses ciX.
    - make_split_read.py has the same flaw and was not edited.
    - A fix needs a rated anchor (an entry, or mean zero) in arena.py.
- **2026-11-12 ~10:00, every champion of the second global league in one arena (the user: "Run all the champion models in
  1 quick arena to see how they all compare"; leagues/arenas/diag/global-all-champions-p1).** The league was stopped
  at cycle 5918 at the user's word; all 210 crowned versions (8 lines) played 4 double round-robins at ply 1, T 0.02,
  hybrid17 (1,680 games each, ±21). A ranking in one field, not an architecture verdict (no stranger panel).
  - Best champion per line (version, crowned cycle, Elo; random 0): D-L z160 v5626 (c5627) 999; D-P z30 v5375 (c5376)
    999; T-L-Nc-merge-fresh v581 969; D-P z20 v513 942; D-L z120 v638 940; T-P-Nc-merge-fresh v354 912; F-L v538 893;
    F-P v463 874. In 7 of 8 lines the last champion is the best (D-P z20's last is 14 below its best: noise).
  - Champions improve steadily in crowning order in every line (D-P z30: 101, 698, 838, 921, 965, 987, 999).
  - THE LATE CHAMPIONS DID NOT GET WORSE (the user's guess): the two lines of the two-line endgame (cycles 1202-5918)
    crowned their best versions in it, but gained little there: D-L z160's best before the endgame 982 (v669), after
    999 (within noise); D-P z30's 959, then 999.
  - At similar versions (~350-650) the trees and DAGs sit 50-110 over the flats (D-L z160 v669 982, T-L-Nc 969,
    D-P z20 942, D-L z120 940, T-P-Nc 912 against F-L 893, F-P 874), the ply-1 tree gain again.
- **2026-11-12 ~08:03, ply-3 panel read of existing models (analyst 6.72 (c) 1; leagues/arenas/diag/panel-ply3-existing).**
  - One arena at PLY 3 (T 0.02, 30 round-robins, 26 lines): x4 fponly-p2 and x4b fponly-p2b F-P x4 at v299, bfp
    base-probit-p2 and pfp probit-late-p2 F-P at v299, f3 flat-p3 and f3b flat-p3-b F-L x4 (ply-3-trained, seeds 20 and
    23) at v199, w4 flat-p3-w4 (warm-started from flat-p3 v199, trained at ply 4) at v99, rtr error-routing-p1's
    T-P-Ne (ply-1-trained) at v300. Each contrast against the entries that are strangers to both sides (w4 = flat-p3's
    lineage); 2,400-4,800 games an arm.
  - PLY-3 TRAINING DOMINATES AT PLY 3: f3 minus x4 147 [130, 164] (direct 197), f3b minus x4b 182 [166, 197] (direct
    194); prediction 150-300 (met). Two factors differ (LR vs probit, ply 3 vs ply 2 training), but the size matches
    "training ply >> architecture".
  - THE PURE-FIELD LEAD IS PLY-SPECIFIC: x4 minus bfp -2 [-20, 15], x4b minus bfp -7 [-25, 10] at ply 3, against 95-96 at
    ply 2 (prediction 30-80: failed). Composition (own kind vs weaker other kinds) bought ply-2 play only.
  - Replicates agree at ply 3: x4b minus x4 0 [-15, 15]; f3b minus f3 6 [-13, 26].
  - PLY-4 TRAINING HURTS PLY-3 PLAY: w4 minus its own start f3 -81 [-96, -66] (direct -38).
  - Others: pfp minus bfp 17 [-2, 36] at ply 3 (direct 13; the batch-size read's direct ply-3 cell was -17 at v300);
    the ply-1-trained tree minus x4 -83 [-102, -65] (direct -172).
- **2026-11-12 ~05:50-06:15, within-run tree-vs-flat cells are not strength verdicts; panel reads (analyst 6.68).**
  - diversity-p2d's read (2,400-game cells): inside d the tree beat its own flat by 61 / 51 at ply 2 (v250 / v299);
    against c's lines (strangers) d's tree minus d's flat was -8 / 9. In c: -42 / -9 direct, -18 / 5 against d's
    lines. In p2c's own read at v300: c's tree minus flat -10 direct, -65 via bfp, -55 via pfp, 10 via dfp.
  - PANEL 1 (leagues/arenas/diag/panel-p1fields-p1/-p2-v300): four ply-1-trained probit fields at v300, strangers
    to each other, F-P x2 and the field's probit error tree x2 each; each tree and flat against the other three
    fields' eight lines (14,400 games each at ply 1, 300 round-robins; 7,200 at ply 2, 150):

    | field (tree) | ply 1 panel tree - flat | ply 1 direct | ply 2 panel | ply 2 direct |
    |---|---|---|---|---|
    | error-routing-p1 (G2; T-P-Ne capped 4) | 127 [119, 136] | 147 | 26 [15, 38] | 80 |
    | error-dag-probit-p1 (T-P-Ne capped 4) | 118 [110, 127] | 81 | 57 [45, 69] | 21 |
    | grow-p1 (T-P-Ne capped 4, current rule) | 11 [3, 19] | 47 | 45 [32, 57] | 67 |
    | grow-p1b (T-P-Ne-grow) | 85 [77, 93] | 79 | 48 [37, 60] | 83 |

  - THE PLY-1 TREE GAIN IS REAL: mean 85 against strangers (within-field direct mean 89), not co-adaptation
    (6.68's kill line was under 30; prediction 40-80). Fields differ a lot: grow-p1's tree only 11 (its tree -19
    against the panel, against 78 / 91 for the same recipe in r and e). Ply-1-trained trees also beat their flats
    at ply 2 against the panel, by 26-57 (mean 44). Single direct cells were off by up to 37 either way.
  - PANEL 2 (leagues/arenas/diag/panel-p2fields-p2-v299): ply-2-trained probit fields at v299, at ply 2, 75
    round-robins: b base-probit-p2 (F-P, cluster tree, DAG), l probit-late-p2, y replay-p2, v diversity-p2, c p2c, d
    p2d (F-P and T-P-Ne). b's panel drops c and d (their frozen references were b's v599 lineage), c's and d's drop
    b and each other (identical through v9). Tree minus flat against strangers (direct in brackets): b cluster -11
    [-27, 5] (-14), b DAG -8 (-16), l -34 [-46, -22] (-69), y -12 (48), v -45 [-57, -33] (-41), c -38 (-8), d -28
    (33). EVERY PLY-2-TRAINED TREE IS BELOW ITS FLAT against strangers (mean about -25); the direct cells ran from
    -69 to 48. 6.68 predicted within 20: b and y only.
  - SAME-KIND CELLS CARRY MATCHUPS TOO. Each F-P against the common panel {y, v} (2,400 games each): d 101, c 80, l
    73, b 40. Panel against direct: d - c 21 vs -19 (p2d's read: -19 [-33, -5] at 2,400 games; against {l, y, v}: 6);
    d - l 28 vs -16; c - l 7 vs 37; b - l -33 vs -52. A direct 2x2 same-kind cross-run cell can sit 25-45 off the
    strangers' view (binomial noise explains ~15-20). So p2d's -19 is not a verdict: no detectable effect of the
    window either way.
  - Same arena, other one-factor cells (panel = the fields that are strangers to both): replay-p2 minus
    probit-late-p2, F-P -117 [-135, -99] against {b, v} (direct -116), tree -96 (direct -115): replay's harm
    survives. base-probit-p2's F-P minus probit-late-p2's -33 [-53, -13] against {y, v} (direct -52).
  - fponly-p2 re-check (leagues/arenas/diag/panel-fponly-p2-v299, panel {y, v}): x4 minus bfp 87, minus probit-late
    65, minus p2c 56, minus p2d 43 (direct 42, 24, 21, 17): direct cells UNDERSTATED x4's lead. Repeat values across
    the two panel arenas (each F-P against {y, v}): b 40 / 47, l 73 / 69, c 80 / 78, d 101 / 91: replicable to ~10.
  - PLY-1 RE-READS (leagues/arenas/diag/panel-router-p1-p1/-p2, panel-grow-p1-p1/-p2; standing ply-1 panel at v300,
    40 round-robins). router-p1 at v299, each arm minus C (no router splits), ply 1 / ply 2: Z -21 / -14, H -12 /
    -24, HG 11 / -8, F-P -94 / -69 (direct at ply 1: -41, -22, 7, -86). Router splits cost ~10-25 against strangers
    (35-40 direct): same sign, the router question stays closed. GROW at v300: grow-p1b's no-bar A minus grow1
    (MDL bar) -56 [-78, -34] / -52 [-75, -30] (direct -67 / -21); A minus F-P 38 / 2, grow1 minus F-P 94 / 54;
    grow-p1c's B minus A 5 / 11, B minus F-P 22 / 36. THE BAR RULING REVERSES AT v300 (it rested on direct cells at
    v150 and v352: -13 / 40, -2 / 19, with a non-transitive triangle at v352); a panel re-read at v150 and v350 followed.
  - GROW RE-READ 2 (06:35; leagues/arenas/diag/panel-grow-p1-p1/-p2-v150/-v350; panel r + e + g at v300, 40
    round-robins, 1,920 games an arm, ±22 on a difference). A minus grow1, v150 / v350: ply 1 -15 [-37, 7] / -56
    [-78, -34], ply 2 -4 [-26, 18] / -4 [-26, 18] (direct 9 / -26, 12 / -30). AGAINST STRANGERS A NEVER LEADS
    grow1 (six reads, -4 to -56), so the "bar goes" ruling does not survive. The bar's value shows late at ply 1
    (-56 at v300 and again at v350); early, and at ply 2 at v350, it is level, so the pre-set rule (30+ below at
    both v150 and v350) is not met. B minus A: ply 1 -7 / -3, ply 2 16 / 32 [10, 54] (v300: 5 / 11). Trees minus
    their flats, v150 / v350: grow1 -12 / 101 at ply 1, -11 / 60 at ply 2; A -27 / 45, -15 / 56; B 37 / 20, 55 / 14.
    Fields differ (at v350, ply 1, grow1 50 against the panel, grow-p1c's trees -23 / -26), so only within-field
    contrasts are verdicts.
  - fponly-p2-720 against fponly-p2 (06:30; panel-fp720-p2/-p1, panel y + v + l at v299; section 7): per compute
    720 is level at ply 2 and 11-43 behind at ply 1; per version 1,584 leads 14-16 at ply 2, 21-27 at ply 1. One
    direct cell between these sibling runs (they share cycle 1) sat 49 off the panel (-59 against -10).

- **2026-11-10 ~19:15-19:30, calibration fitted at the wrong ply (diagnostic (a) of PATH-FORWARD.md)**
  (leagues/arenas/diag/cal1-routing, cal2-routing, cal1-routing2, cal2-routing2; specs in leagues/specs/arenas/).
  - Question: experiments calibrate every version on 600 of its own games at depth 1 and then rate it at ply 2. Is
    that why LR trees lose their ply-1 lead at ply 2?
  - Setup: each set's final champions play at ply 2 twice, in separate arenas. Once with their experiment's
    calibration (own games at depth 1) and once calibrated on 600 own games at depth 2. 40 double round robins
    with the random players, ~960 games each, ci95 ~29-34.
    - First set: split-growth's five and single-split's F-L, T-L-Le, T-L-Nc-merge and T-L-Tc-merge.
    - Second set: error-targets' five, and residual-leaves' and trigger-gradient's three each.
  - The two fits differ a lot (sg.F-L's parameters 0.50, 0.46, -2.18, -2.00 against 0.64, 0.79, -0.35, 0.07). The
    ply-1 fit expects many more draws.
  - Tree minus its F-L, ply-1 calibration then ply-2 calibration:
    - single-split: T-L-Le -89 then -18, T-L-Nc-merge 7 then 51, T-L-Tc-merge 17 then 61;
    - split-growth: the four trees moved -8 to 4;
    - error-targets: T-L-Le -60 then -45, route-before-train -95 then -55, gap -66 then -71, gap with bootstrap
      -37 then -10;
    - residual-leaves: copy tree -1 then -77, residual tree -23 then -91;
    - trigger-gradient: today's trigger -8 then -30, T4 -2 then 0.
  - The change of gap has an SE of ~31 from match noise. Over the 15 comparisons the mean change is ~4 and the SD
    ~42, so calibration adds roughly ±30 per comparison and has no direction.
  - Absolute Elo across the two arenas (approximate, linked only through the random players): each model moved -16
    to 78.
  - Reading:
    - Calibration at the wrong ply does not explain the LR trees' ply-2 deficit in general.
    - It is a noise source the size of the effects we measure, though: one tree-vs-flat gap moved 71 (single-split's
      T-L-Le) and two moved about -70 (residual-leaves).
    - Ply-2-only experiments (batch 4) still calibrate on ply-1 games. Calibrating at ply 2 costs ~10× as much a
      version, and a scale-free policy is the other option (NEW-DESIGN, Open). This is a design question for the
      user; running experiments are unchanged.
- **2026-11-10 ~18:40, the table tool counted 5 rounds a cycle** (curtab.py and curcur.py in fs/analysis-2026-11-10/
  xtab, and traj.py, traj10.py and bins.py in .../traj).
  - They mapped round t to cycle t // 5 + 1. Batch 3 plays 9 rounds a cycle and small-field 56, so on those runs
    the windows were mislabelled: "cycles 350-400" of a 9-round run were really cycles ~195-222. The tables covered
    only the first ~55% of each 400-cycle run, and small-field's the first ~9%.
  - Fixed: they now read rounds_per_cycle from config.json, and give byte-identical output on 5-round runs.
    champtab.py always read it, so the champion tables were right. Originals:
    /tmp/claude-1000/-home-iv-dev-chessai/1265e529-e944-48f6-8cee-07f303fc678a/scratchpad/rpcfix/.
  - Corrected in place, with the readings redone:
    - split-growth: the single-split gain half replicated (14-43 at cycles 200-400), where the first table said it
      didn't. Limit 4 with merges ended furthest ahead (34-53), and limit 4 without merges caught up (11).
    - residual-leaves: residual leaves led only in cycles 51-150 and ended 38 behind the copy tree. The copy tree was
      level with flat from cycle 200.
    - error-targets: gap targets are 29-80 better than Best (first table: 27-50). Route-before-train alone is level
      with Best (first table: 3-43 worse). The probit twin's note is redone too.
  - Numbers given only in chat or the memory log:
    - small-field's head-to-head "10, 2, -4, 22" covered its first ~21 cycles. Corrected: F-L led by 18-33 (current
      and champions) in cycles 51-200, and they have been level since.
    - split-growth-probit's T-P-Nc-merge limit 1 "19, 0, 18, 12 through 400" is really 5, 13, 13, 20.
- **2026-11-10 ~18:30, ply 1 against ply 2 on the same versions** (fs/analysis-2026-11-10/plyfid, README.txt there).
  - Data: the 18 old-design runs. Every line had a ply-1 and a ply-2 entry of its current version. For each pair of
    lines and each quarter of the run, direct Elo at ply 1 and at ply 2: 1,328 comparisons in all.
  - Ply-1 gaps are about half the ply-2 gaps (slope 0.50, correlation 0.81). Ply-1 games between models are 43%
    draws on average, against 10% at ply 2.
  - Of the 949 comparisons that are clear at ply 2 (|z| > 2), ply 1 agrees clearly in 603 (64%), is unclear in 211
    (22%) and clearly says the opposite in 135 (14%).
  - Agreement by run:
    - High (correlation 0.90-0.98, almost no reversals): single-split, single-split-probit, split-growth-probit,
      error-targets-probit.
    - Low (0.24-0.49): cluster-error, split-growth, error-targets, trigger-gradient. These are LR runs where the gaps
      are small.
  - The reversals come in two kinds:
    - Lines trained on data of different depth (selfplay-depth, leaf-sources; section 6).
    - LR trees against flat LR. In the last quarter, at ply 1, the cluster trees of split-growth, residual-leaves,
      trigger-gradient and small-field were 27-73 ahead of F-L, and the gap-target error trees 27-47 ahead. At ply 2
      they were between 31 behind and 45 ahead.
    - Probit trees mostly agree across plies, and their ply-2 gaps are the larger.
    - Over all 45 tree lines in the last quarter: mean tree minus flat 6.9 at ply 1 and -11.2 at ply 2.
- **2026-11-10 11:00, champion bias in version windows.**
  - Example: T-L-Lc's champion v876 put it 29 above F-L at v831-999. Current against current, it was -1 to -4.
  - The 08:40 direct numbers carry some of this bias. T-L-Tc's 97 at v601-830 includes its champion, v746.
  - Source: archive NEXT-EXPERIMENTS §3 "Final check".
- **2026-11-10 04:00, champions mislead (single-split).**
  - F-L's champion (v150, 1417) and F-P's (v222, 1322) sit 57-70 above their lines' mean recent version.
  - Champion against champion, the NB and TAN cluster trees were 56-57 above F-L but 21-32 below F-P.
  - The mean over many versions is the fairer measure of a recipe.
- **2026-11-09 05:09-06:05, version noise** (fs/twin/league: vnoise, rates, ratepool).
  - Setup: twin r0.05 at five adjacent versions, v426-v430, each against tree v428.
  - Results: 18, 5, 0, -2, -32. That is a version SD of about 17 (one-sided 95% bound 42); match noise alone would
    give SD 7.3.
  - Drift: the rate ladder's second window moved about 30 against its first, so strength also drifts over tens of
    versions.
  - Readings that fell to this: "routing to L9's region is worth 23", "a data × step interaction of ~80", and
    "rate 0.025 beats 0.05 by 31".
- **2026-11-09 11:15, field panel** (fs/fieldpanel). Did head-to-head tests hide a benefit of routing against the
  field? No.
  - Setup: each tree against the same 12 ply-2 entries, d2, 3 versions each.
  - T-L-Ne-c tree minus L7: -3 [-15, 8] (head-to-head -12 / -27).
  - T-L-Ne-share-c tree minus L9: -32 [-44, -20] (head-to-head -41 / -36). Tree minus flat twin: -23 [-35, -12].
    L9 minus twin: 9 [-3, 20].
  - The tree routes the same share to its busiest leaf against the field as against that leaf (0.634 vs 0.632).
- **2026-11-09 10:45, live-league drift and gates** (archive IDEAS, 11-09 ~10:45).
  - The rating scale was stable. 102 unchanged ply-2 entries moved 30 from cycle 5 to 60, and 6 from cycle 30 to 60.
    The ~140 drop happened inside cycle 1.
  - The top 10 minus that panel: 82 (c1), 115 (c15), 158 (c35), 179 (c60). Lineages active in cycles 40-60 gained a
    median 44 (d0/d1), 32 (d2) and ~10 (d4). Current versions sat ~30 below their champions every cycle.
  - Gates (post vs pre, 100 games at d2): 3,807 gates at 54.4% (~30 Elo). Their claimed gains, 360-650 over 20 cycles,
    far exceed the 30-45 seen in the league. So gate gains don't transfer. Untested causes: predecessor-specific
    gains, lucky keeps.
- **2026-11-07/08, league calibration confound** (fs/leaguecal).
  - Own-play calibration slopes ranged 0.55-2.03. d0 LR entries played hot and NB and d2 entries cold, and colder won.
  - The effect is 132 Elo per unit of ln(slope ratio) at ply 2. It explains most of the d2-trained lead over d0 LR
    trees.
  - Fix: cal-own, at 1-3 CPU s a version. It is now the experiments' rule.
- **Offline against play** (whytrees, leafcmp, rates, d2pair, lrprobit).
  - The offline-best leaf can play worst: 171 below its tree.
  - Per-leaf calibration gains 0.025-0.041 offline (d2play) and loses in play: -19 to -93 at d1, -101 and -147 at d2
    (leafcal against cal; whytrees).
  - Smaller LR rates scored worse offline and played better in one window.
  - Probit: far worse offline log-loss, yet the stronger player at depths 0-1.
- **2026-11-07/08, blendsp:** adjacent versions of one lineage differed by up to 75 Elo at ply 1 and 200 at ply 2
  against the same opponent. Pool versions.
- **2026-11-08 22:45, a 123-Elo drop was self-play drift** (fs/sharedrop). T-L-Ne-share-c v433 was -98 against v428
  (calibration ~14 of it), sliding over 5 rounds (-9, -18, -57, -36, -84) at a fixed structure. Self-play's
  vs-previous check saw at most ±8 a round (~92% draws). The next own step beat v433 by 100; v440 was level with v428
  (-6 [-20, 9]). Every lineage's d2 entries swung 40-70 between versions; this one ~120.
- **2026-11-07:** 200-game arenas of near-flat models swing about 60 Elo (resid/fresh).
- **2026-11-05:** measure at depth 2 or more. One-ply results showed a plateau that wasn't there (LEDGER).
- **2026-11-11 00:30, play temperature is a confound even with cal-own** (section 8, "Play temperature changes the
  verdicts"). At the standard log-odds softmax T 0.1, a model's effective temperature is T over its calibrated
  scale at the ply it plays, and depth-1 own calibration doesn't fix the scale at ply 2. The late copy tree's
  63-Elo lead over F-L vanished at T 0.02, and both tuned flat leaves passed every old routed champion. Alpha-beta
  plays argmax, so rate near argmax (T 0.02) when the question is evaluation quality.

---

## 2. Leaf types: LR, probit, NB

Where it stands:
- **Flat.**
  - LR is the strongest learner on equal data.
  - Probit trails LR at depth 2, and more so as data grows. Its offline deficit is the draw question.
  - NB (CNBP) is far behind. In a field of NB models only it can't bootstrap at any alpha from 0.1 to 100
    (nb-settings-2, 500 cycles).
  - NB as a predictor was abandoned on 11-08 (the user); NB routers and gates stay.
- **LR's best step size falls with data** in an LR-only stream (lr-settings-2): rate 0.1 led the default 0.05 by 79
  in cycles 1-50, and 0.0125 led it by 31-33 in cycles 401-450 and 451-500.
- **Forgetting in flat probit depends on the stream.** In a probit-only stream the default tau 1e-5 is about 20 above
  tau 0 from cycle 100, and tau 1e-4 is 26-61 below it (probit-settings). In self-play and on a tree's kept stream
  tau 0 was the better one (11-07, 11-09).
- **Data source:** at equal CPU, LR and probit learn more from the experiment's games than from their own depth-0
  self-play, by about 90-200, in both leaf-sources runs. NB is the reverse.
- **In trees, on equal data, probit leaves keep improving longer.** Probit cluster trees were still climbing at cycle
  1000, while the LR-leaf ones saturated by ~200.
  - In cluster-error, which mixes leaf types, the probit cluster trees lead flat and the LR-leaf ones don't. Why is
    open (section 3).
- **Self-play, each line on its own games** (selfplay-depth): at depth 2 F-P was the strongest line (1696 against
  LR's 1595-1616), and still climbing.
- **Matched training positions:** LR d4 beats probit d4 by 78 [53, 102] (section 8).

Entries:
- **2026-11-12 ~05:10, LR with decaying AdaGrad sums (analyst 6.64 (b)): every memory loses, offline.** Build
  chessai-lrdecay1 (md5 cab29ea7; grow5 + --lr-sum-memory N: S <- S (1 - 1/N) + g^2 per example of the node's own,
  lazy, exact; byte-identical to grow5 at N 0 on 9 arms; models/from-scratch/fastbin/lrdecay1/summary.txt). F-L,
  one pass, prequential by round; gain = AdaGrad's loss minus N's (negative: the decay is worse):

  | N | self-play drift, raw / calibrated | abrupt change P2x -> P2, raw | stationary P1 held-out cost, raw / calibrated |
  |---|---|---|---|
  | 1M | -0.0062 / -0.0041 | -0.0111 | 0.0259 / 0.0063 |
  | 2M | -0.0037 / -0.0025 | -0.0066 | 0.0181 / 0.0043 |
  | 4M | -0.0020 / -0.0014 | -0.0036 | 0.0120 / 0.0027 |
  | 8M | -0.0010 / -0.0007 | -0.0018 | 0.0074 / 0.0015 |

  - Every N loses on every stream, by about half as much per doubling of N; the losses grow along the drift stream
    and shrink, without turning into gains, after the abrupt change. Calibration removes a fifth to a third.
  - Prediction (6.64 (b)): drift gains 0.003-0.008 at 1-2M. Measured -0.006 and -0.004. lr-decay-p1 not run (draft
    leagues/specs/batch6/lr-decay-p1.DRAFT.json). The league record agrees: in lr-settings-2 the smaller late rate
    won, so AdaGrad's late steps are not too slow for a flat. The split boost (--forget-factor 10, grow-l-p1b) is a
    fresh child's transient need, not a steady-state one.
  - Caveat: an offline stream has no on-policy loop.
- **2026-11-10, probit against LR in trees:** see section 3 (single-split vs single-split-probit; cluster-error).
  - In cluster-error, flat F-P minus F-L by window was -83, -26, -10 and -25 (cycles 1-100, 101-200, 201-300 and
    301-415).
  - **Update 11-10 14:00: F-P caught up.** Current against current, F-P against F-L: -25 (cycles 300-399), -33
    (400-449), -26 (450-499), -22 (500-529), -5 (530-549), 24 (550-571; ~200 games, ±45). The probit trees did not
    climb with it: T-P-Tc stayed 10-20 above F-L throughout, so its lead over F-P shrank to about 0.
  - **It did not hold (16:10):** F-P against F-L was -15 (cycles 500-549), 7 (550-589) and -44 (590-630, 400 games).
    The 24 at 550-571 was a short-window swing; the drift measurement found no offline counterpart either.
  - **Champions (16:25; analysis-2026-11-10/xtab/champtab.py).** F-P's champion (v236, crowned at cycle 238) beat
    F-L's (v317, crowned 319) in every window since cycle 400: 23±22, 18±30, 20±34 and 35±33, ~24 over ~2,300 games.
    Champion-role games all come after crowning, so the selection does not bias them. Best against best favours
    probit; typical against typical (current@2) favours LR.
  - Every line's latest version is 40-140 below its own champion in every window since cycle 400. Most champions were
    crowned at cycles 211-340; since 340 only the probit NB trees crowned new ones (cycles 356, 387 and 477). The run
    has plateaued, and choosing the best version is worth 40-140 Elo.
  - **Forgetting against the stream (the user's hypothesis, 11-10 14:05: games from many architectures drift more,
    so gradual forgetting pays more).** The tau results so far order that way: tau 0 (no forgetting) was better by
    48-85 in pure self-play (11-07) and by 37-129 on a single tree's stream (11-09); in a probit-only league stream
    (probit-settings) forgetting led by ~20; in the 14-architecture cluster-error stream flat probit overtook flat LR
    late. lr-settings-2 agrees from the other side: in an LR-only stream the smaller late step won (rate 0.0125 led
    0.05 by 31-33 in cycles 401-500). Being tested: the drift measurement (models/from-scratch/drift/) and the
    forgetting experiment (LR --lr-memory, probit tau 0; specs batch3/13-forget-*).
  - **Drift measured (11-10 ~15:45; models/from-scratch/drift/summary.txt): the mechanism is not supported.**
    - Staleness (frozen version t minus the version that played t+k, winner loss on the same replayed games): from
      t 200-300 on, no stream outdates a frozen model, at any lag up to k 100 (882k games later in cluster-error).
      Early (t 100) every stream outdates except split-growth, and per game cluster-error outdates more slowly than
      single-split and single-split-probit.
    - Cluster-error's position mix differs 2-4× more between nearby cycles (JS divergence), but the difference does
      not build up with lag: jitter, not drift. Game length and phase mix are stable.
    - Step sizes: F-L's falls as 1/sqrt(positions seen), on the same curve in both experiments (0.22 of its v25 value
      by v575); F-P's variance sits at the tau equilibrium from v25. The probit/LR step ratio rises smoothly (1.0 at
      v25 to 4.6 at v575), with no event at the Elo crossover.
    - Both already forget in effect: a probit weight holds its last ~145k uses (0.26 cycles in cluster-error), F-L
      about 1-1.5 cycles. Jitter that doesn't carry over favours longer memory, not shorter.
    - Offline, F-P has the better winner loss in 18 of 20 sampled cluster-error cycles (by 0.006-0.008), before and
      after the crossover. The Elo crossover has no offline counterpart.
    - Verdict: the first half (more diverse games) is weakly supported, as jitter. The second half (forgetting more
      valuable) is not. The forgetting test (13-forget-*) is now low priority.
- **2026-11-10 01:08, nb-settings-2** (exp nb-settings-2; spec batch1/1c2-nb-settings-2.json). F-N at --alpha 0.1 /
  1 / 10 / 100 (default 1), local, seed 1, 500 cycles of 5 rounds. The nb-settings rerun with alpha 100 added.
  - No line bootstrapped. Each line's best-rated version is v1 (377-429; alpha 100's is v158 at 349). Current versions
    stayed at 250-330 for 500 cycles, between random-2 (245) and random-3 (356); at v499 they were 279-313.
  - Current against current, Elo against alpha 1 (500 games a cell in 50-cycle windows and 1,000 in 100-cycle ones;
    95% intervals about ±16 and ±12):

    | alpha | 1-50 | 51-100 | 101-200 | 201-300 | 301-400 | 401-500 |
    |---|---|---|---|---|---|---|
    | 0.1 | -5 | -7 | -6 | 8 | 6 | -4 |
    | 10 | 11 | -5 | 2 | 1 | -1 | 5 |
    | 100 | -16 | -2 | 9 | 3 | 8 | 7 |

  - No cell clears its interval.
  - Conclusion: the smoothing prior changes nothing, and no alpha lets NB bootstrap from NB games alone. It replicates
    nb-settings (below).
- **2026-11-10 00:35, lr-settings-2** (exp lr-settings-2; spec batch1/1a2-lr-settings-2.json). F-L at --lr-rate
  0.0125 / 0.025 / 0.05 / 0.1 (default 0.05), local, seed 1, 500 cycles of 5 rounds. The lr-settings rerun without
  its search-label line.
  - Current against current, Elo against rate 0.05 (500 games a cell in 50-cycle windows and 1,000 in 100-cycle
    ones; 95% intervals about ±29 and ±20):

    | Rate | 1-50 | 51-100 | 101-200 | 201-300 | 301-400 | 401-450 | 451-500 |
    |---|---|---|---|---|---|---|---|
    | 0.0125 | -212 | -175 | -60 | -26 | -16 | 33 | 31 |
    | 0.025 | -151 | -50 | -29 | -13 | 16 | -2 | 19 |
    | 0.1 | 79 | 26 | 20 | 8 | -8 | -5 | -14 |

  - Larger rates lead early. 0.1 was 79 [53, 106] above the default in cycles 1-50, and level from cycle 200.
  - The smallest rate climbed in every window. 0.0125 led the default by 33 [4, 62] and 31 [2, 61] in cycles 401-450
    and 451-500, two separate windows, and led 0.1 by 32 in cycles 401-500 (1,000 games).
  - 0.025 was level with the default from cycle 200 on.
  - Final ratings (v499): 0.0125 1390, 0.1 1368, 0.025 1364, 0.05 1302 (single versions: about 17 Elo of noise).
  - Conclusion: in an LR-only stream the best step size falls with data. 0.05 is right for about the first 300 cycles
    and 0.0125 after cycle 400. This supports, for this stream, the claim withdrawn on 11-09 on the twin study (below),
    whose stream was a tree's and whose windows ended at v452.
- **2026-11-09 23:46, probit-settings** (exp probit-settings; spec batch1/1b-probit-settings.json). F-P at
  --probit-tau 0 / 1e-6 / 1e-5 / 1e-4 (default 1e-5), local, seed 1, 500 cycles of 5 rounds.
  - Current against current, Elo against tau 1e-5 (500 or 1,000 games a cell, as above; 95% intervals about ±29 and
    ±20):

    | tau | 1-50 | 51-100 | 101-200 | 201-300 | 301-400 | 401-500 |
    |---|---|---|---|---|---|---|
    | 0 | -9 | 11 | -19 | -4 | -19 | -21 |
    | 1e-6 | -10 | 15 | -11 | 5 | 11 | 11 |
    | 1e-4 | -30 | -61 | -30 | -37 | -26 | -45 |

  - tau 1e-4, ten times the default's forgetting, is below the default in every window, by 26-61.
  - tau 0, no forgetting, is level for 100 cycles and then about 20 below: -19, -4, -19 and -21 in the 100-cycle
    windows, and -26, -13, -13 and -28 in the four 50-cycle windows from cycle 301. Small, but the same sign in every
    window from cycle 101.
  - tau 1e-6 is level throughout (-11 to 15).
  - Final ratings (v499): 1e-5 1419, 1e-6 1406, 0 1402, 1e-4 1385.
  - Earlier settings gave the other sign: tau 0 led tau 1e-5 by 48-85 in equal-CPU self-play (11-07, below) and by
    37-129 on a tree's kept stream (twin study, 11-09, below). So the value of forgetting depends on the stream. This
    is the single-architecture baseline for the drift question (NEXT-EXPERIMENTS decision 1).
  - Flat probit's memory length doesn't explain the probit trees' slow climb.
- **2026-11-09 23:25, leaf-sources-2** (exp leaf-sources-2; spec batch1/4b-leaf-sources-2.json). F-L, F-P and F-N,
  each as a local line (trains on the experiment's games) and a self-play line (depth 0 against its previous
  version); seed 1, 100 cycles of 5 rounds, equal priced CPU per line. The leaf-sources rerun without its second F-L.
  - Data: each local line trained on 183k games (13.7M positions). Each self-play line played 0.90-0.97M games of its
    own, about five times as many.
  - Current against current, Elo against F-L local (200 games a cell; 95% intervals about ±50 for gaps under 150,
    wider beyond):

    | Line | 1-20 | 21-40 | 41-60 | 61-80 | 81-100 |
    |---|---|---|---|---|---|
    | F-L self-play | -91 | -149 | -113 | -121 | -149 |
    | F-P local | -49 | -141 | -96 | -47 | -98 |
    | F-P self-play | -109 | -269 | -275 | -279 | -252 |
    | F-N local | -449 | -552 | -552 | -564 | -676 |
    | F-N self-play | -162 | -220 | -210 | -295 | -238 |

  - F-L local leads every line in every window.
  - Local beats self-play for LR (by 91-149) and for probit (directly, 139 in cycles 41-60 and 203 in 81-100).
  - NB is the reverse: self-play beats local directly by 215 and 291 in the same windows.
  - Final ratings (v99): F-L local 1759, F-P local 1670, F-L self-play 1662, F-P self-play 1510, F-N self-play 1366,
    F-N local 1028.
  - It replicates leaf-sources (the same design with a second F-L local line; finished 22:22 on 11-09). Against
    F-L#1 local in the same windows: F-L self-play -60 to -182 (two lines), F-P local -10 to -113, F-P self-play -149
    to -330, F-N local -449 to -576, F-N self-play -207 to -291. Its two F-L local replicates differed by -38 to 16.
  - Conclusion: at equal CPU, LR and probit learn more from the experiment's games than from their own depth-0 games,
    by 90-200, despite five times fewer games. NB learns better from its own. The confounds noted at cycle 36 stand:
    who plays the games, how decisive they are, and volume.
- **2026-11-09 ~21:00, leaf-sources at cycle 36** (exp leaf-sources, 8 lines; mean of the last 5 cycles' ply-2
  ratings; from the session's experiment log, not in the docs before). The design is in section 7.
  - LR: local 1,699 and 1,702 (two replicates), against self-play 1,607 and 1,622.
  - Probit: local 1,644 against self-play 1,479.
  - NB: local 1,064 against self-play 1,371. That -307 matches the offline F-N-onL result of -305 (11-06).
  - Local lines trained on 110k games, self-play lines on ~550k (equal CPU). It is confounded three ways: who plays
    the games, how decisive they are, and volume.
  - Final, cycle 100, current against current: in the leaf-sources-2 entry above.
- **2026-11-09 ~20:50, NB doesn't bootstrap alone** (exp nb-settings: F-N at alpha 0.1 / 1 / 10, local).
  - All three stayed flat at 250-380 for 92 cycles (random-2 241, random-3 352), with champions still v1-v2.
  - The same F-N recipe trained locally in leaf-sources' mixed field reached 1,168 by cycle 17 on about the same games.
    F-N self-play reached 1,323.
  - F-N self-play reached 1,323 in leaf-sources and 1,186 in selfplay-depth (depth 0, cycle 3).
  - Why: own-game calibration shrinks the NB's winner log-odds to slope 0.07-0.09, against 0.88 (F-N.loc) and 1.84
    (F-N.sp) in leaf-sources and 0.55-1.0 for LR. 42% of nb-settings' qualifying games were against random (240 of
    570 a cycle), and 53% were decisive.
  - nb-settings was stopped at cycle 265, and nb-settings-no-random3 at cycle 126 (21:15); see section 7.
  - Rerun to 500 cycles with alpha 100 added: nb-settings-2 above, with the same result.
- **2026-11-09 21:13, lr-settings** (F-L at four rates, plus a search-label line at 0.05).
  - The label line was the strongest: 1,734 at cycle 57, against 1,440-1,580. But its training took 12 times the CPU
    (3.57 s against 0.28-0.29 s a cycle), which experiments don't price, and its games fed every rate line.
  - It was stopped at cycle 172 and rerun as lr-settings-2 without the label line.
- **2026-11-09 02:45-05:17, LR against probit on equal data** (fs/twin/league/summary.txt; learners on
  T-L-Ne-share-c's kept stream; d2, 2000 games, own cal). Probit minus the LR twin r0.05:

  | Probit | v122 | v217 | v452 |
  |---|---|---|---|
  | tau 0 (F-P-tau0-c's recipe), d2 | -10 | -50 | -101 |
  | tau 0, d1 | -30 | -62 | -57 |
  | tau 1e-5 (F-P-c's recipe), d2 | -69 | -191 | -197 |
  | tau 1e-5, d1 | -14 | -56 | -62 |

  - tau 1e-5 against tau 0 at d2: -37, -129, -117. So tau makes the probit worse, and stiffening is not why it
    trails.
- **2026-11-09 01:30-06:05, LR step size: 0.05 is near a flat optimum** (fs/twin/league: rates, ratepool; flat twins
  on T-L-Ne-share-c's stream, d2, against twin r0.05 at the same version).

  | Rate | v122 | v217 | v424-v432 pooled | v444-v452 pooled |
  |---|---|---|---|---|
  | 0.025 | -25 | -2 | 31 [20, 43] | 2 [-11, 14] |
  | 0.0166 | -69 | -26 | 15 [0, 30] | -5 [-28, 18] |

  - Larger rates lose at every checkpoint (r0.10 and r0.165 against the tree; section 3). A ~1.7× step on all data
    loses 46 (Q6).
  - The smaller rates' lead in the first window did not replicate in the second. "The best rate falls with data"
    was withdrawn.
  - lr-settings-2 (above) supports it for an LR-only stream: 0.0125 led 0.05 in both of the last two 50-cycle windows.
  - In the live league, F-L-agdiv-c (F-L-c v4725 with its AdaGrad sums divided by 2.7, a 1.64× step) against F-L-c at
    the same age: 9, -20, -8, -11 and -29 over cycles 48-52. No gain (fs/agdiv/README.txt).
- **2026-11-09, solo lineages** (live league, no league data, ages 1-4 cycles).
  - Probit led LR at d0 by 86, 75, 31 and 23, and at d2 by 124, 141, 84 and 59.
- **2026-11-08 ~22:00, d2pair** (fs/d2pair; 4,000 games each, league policy, own calibrations; probit minus LR):

  | Pair | depth 0 | depth 1 | depth 2 |
  |---|---|---|---|
  | d2-trained (F-P-tau0-d2jit-c v5150, F-L-d2jit-c v4978) | 7 [4, 10] | 46 [40, 53] | 35 [25, 46] |
  | flat-trained (F-P-tau0-c v3595, F-L-c v4100) | 6 [3, 9] | 31 [24, 39] | -146 [-158, -135] |

  - The probit was the stronger player at depths 0-1 in both pairs, despite far worse offline log-loss. At depth 2
    the flat-trained probit collapsed.
  - Later, on equal data (above), it trailed LR at both depths. So its league edge was not the learner itself.
  - League champions then: F-P-tau0-d2jit-c 1831 against F-L-d2jit-c 1805. Both lineages were 74-79% league-trained
    a cycle.
- **2026-11-08 ~21:30, lrprobit** (fs/lrprobit).
  - Offline, LR is better everywhere. The probit is 0.140 [0.125, 0.155] behind at full data: 0.136 of that is draw
    against decisive, and 0.0045 [0.0007, 0.0083] is who wins.
  - It has one latent and one draw margin, so a draw can only mean "balanced". Its calibrated draw slope is -0.35 to
    -0.40.
  - No crossover from 10k to 2.3M positions. It is worse in 50 of 59 groups and ties in 9.
  - No gate helps: the best scores -0.0011 [-0.0032, 0.0009].
  - The LR + probit hybrid (bin/chessai-hybrid, M-LP-L) is LR offline (-0.0014). Its hard version hands the probit
    0.38% of positions.
- **2026-11-08 ~13:40, lrnb** (fs/lrnb). LR is better everywhere; NB is a weaker model of the same thing.
  - The two tie at about 10k positions (NB minus LR 0.016 [-0.003, 0.035]). The gap grows to 0.110 [0.079, 0.142] at
    full data. NB flattens at about 0.74 from 300k positions.
  - NB is worse in 55 of 58 groups and ties in 3. The gap is largest where the two disagree on the likeliest result
    (0.198), in long games (0.234), bare endgames (0.147) and draws (0.143).
  - A gate mixing the two gains nothing (-0.0002 [-0.0046, 0.0041]). The oracle's -0.109 is loose: two LRs "gain"
    0.071.
- **2026-11-07 04:30-11-08, flat probit (F-P) introduced** (idea 19; fs/probit, fs/probit/tau).
  - The model: an online Bayesian ordinal probit with assumed-density filtering, dynamics noise tau and a draw margin.
  - First self-play checks against F-L-c's versions at d2: v10 2 [-30, 34], v30 80 [48, 112]; at d1 v30 38. Offline
    winner log-loss 0.3341 (LR 0.3293). About 20% cheaper per game than LR then.
  - Single runs like this are not conclusive: an LR tree that had merged back to one leaf scored 87 against F-L-c's
    v30 from its different history.
  - Correction: at matched self-play CPU it is about 100-130 below F-L-c at ply 2 in the league. Its self-play is
    near-random, because its learned draw band (eps ≈ 2.0) compresses expected-score gaps: jitter10 picks a non-top
    move 84% of the time, against 61% for LR.
  - Self-play at eps 0 was worse still: -346 at equal CPU.
  - Memory: at tau 1e-5 the weight variance floors at about 2 tau, a memory of about one round.
  - Equal-CPU checks against tau 1e-5 (d2, 2 × 1000 games):
    - tau 1e-4: -415 / -421 at v30 / v100;
    - tau 1e-3: -512 / -561;
    - tau 1e-6: 48 / 47 / 63 at v100 / v200 / v300;
    - tau 0: 73 / 48 / 85.
- **2026-11-07, probit leaves in trees (T-P), closed at the time.**
  - Children started from the parent's posterior. Offline they matched LR trees on winner log-loss.
  - From scratch in self-play they lost to flat F-P more as they split (v10 -163, v30 -233). Branches survived only
    through per-leaf draw margins, a per-leaf temperature that makes seams. With one shared margin the fresh test
    merged every branch back (v30 a single leaf, -70, within self-play noise).
  - Superseded on equal data: T-P cluster trees beat flat by 75-77 in single-split-probit (section 3).
- **2026-11-07, WANBIA** (fs/wanbia). NB's gap to LR is not double counting between whole features.
  - Weighting features moves NB's winner loss from 0.4051 only to 0.3874 (LR 0.3293).
  - Deeper NB trees don't help either: 27 leaves 0.7199, 81 leaves 0.7428, against 3 leaves 0.7208.
- **2026-11-06, factorization machine** (F-FM-s21, from scratch).
  - -259 [-273, -246] against F-L at d2, still climbing at v1000.
  - Worse held out: 0.531 against 0.497. Untuned. FM was kept out of stage 1 of the new league (11-09).
- **2026-11-05, LR over NB.** LR beat CNBP by about 370 (LR v1000 against CNBP v2000: 374).
  - NB plateaus at about v300-500 at d2 and can't learn from model-derived labels (LEDGER).

---

## 3. Routing and compound models

Where it stands:
- **On equal data (the new league), cluster-routed single splits beat flat:**
  - by 70-100 with LR leaves, from cycle ~200 to 1000, saturated (single-split);
  - by 75-77 with probit leaves at cycle 1000, still climbing (single-split-probit).
- **Error trees:**
  - T-P-Ne (NB error router, probit leaves) reached 44-47 above flat, the first error tree clearly above flat.
  - No LR-leaf error tree ever beat flat.
- **cluster-error, where LR and probit lines share one stream:**
  - the probit cluster trees lead their flat line (TAN ~45-50, NB ~25);
  - the LR-leaf cluster trees don't (-25 to 26);
  - flat F-P was 10-25 below F-L to cycle ~530, then caught up (24 above at cycles 550-571; section 2).
  - Why the two leaf types react oppositely is open. Ruled out: the partition, near-copy leaves, the decisive rate
    and the phase mix. Data volume doesn't fit either. Left: what the stream is made of, and chance.
- **Partition.** Every NB and TAN cluster router found the same partition, by game phase: opening, queenless
  middlegame and endgame. The LR cluster routers churn and are flat in effect.
- **Confound:** the winning cluster lines also had the merge gate (--revise 0) and the error lines didn't.
  split-growth tests the gate (section 4).
- **The old league** (champion run): the trees' league edge was league data, not routing. Many routed lineages were
  flat in effect.

Entries:
- **2026-11-10 11:00, the final check at cycle 1000** (single-split ended 10:51, single-split-probit 10:56; cluster-error
  to cycle 415).
  - Method: direct Elo against the experiment's flat line, current@2 against current@2 only
    (fs/analysis-2026-11-10/xtab/curtab.py).
  - Cells hold 1000 games, or 2000 in the 200-cycle windows: about ±22 (±16) at 95%.

  | Line | 1-100 | 101-200 | 201-300 | 301-400 | 401-600 | 601-800 | 801-900 | 901-1000 |
  |---|---|---|---|---|---|---|---|---|
  | T-L-Nc-merge (against F-L) | 6 | 41 | 83 | 77 | 76 | 89 | 53 | 88 |
  | T-L-Tc-merge | 13 | 29 | 81 | 90 | 73 | 92 | 68 | 99 |
  | T-L-Lc-merge (flat) | -5 | 16 | -16 | -7 | -7 | -9 | -4 | -1 |
  | T-L-Ne | -57 | -29 | 0 | -25 | -22 | -7 | -25 | -16 |
  | T-L-Te | -76 | -40 | -23 | -30 | -18 | -28 | -31 | -37 |
  | T-L-Le | -118 | -91 | -78 | -88 | -98 | -69 | -95 | -111 |
  | T-P-Nc-merge (against F-P) | -16 | -13 | 3 | 14 | 35 | 57 | 60 | 77 |
  | T-P-Tc-merge | -29 | -48 | -14 | 18 | 37 | 49 | 63 | 75 |
  | T-P-Lc-merge (flat) | -9 | 1 | 18 | -2 | 2 | 2 | 3 | 9 |
  | T-P-Ne | -41 | -14 | 16 | 4 | 0 | 30 | 47 | 44 |
  | T-P-Te | -30 | -57 | -55 | -41 | -38 | -12 | -10 | -9 |
  | T-P-Le | -42 | -101 | -103 | -119 | -89 | -67 | -46 | -54 |

  cluster-error by the same measure (twins: plain / +error=log):

  | Line | 1-100 | 101-200 | 201-300 | 301-415 |
  |---|---|---|---|---|
  | T-L-Nc (against F-L) | 14 / -2 | 3 / 6 | 9 / -13 | -25 / -19 |
  | T-L-Tc | 10 / 18 | 26 / 17 | 5 / 19 | 15 / 2 |
  | T-P-Nc (against F-P) | 29 / 24 | 25 / 41 | 25 / 25 | 21 / 25 |
  | T-P-Tc | 37 / 54 | 51 / 64 | 58 / 41 | 45 / 43 |
  | F-P (against F-L) | -83 | -26 | -10 | -25 |

  - **LR leaves:** the cluster lead formed by cycles 200-300 and held at about 70-100 to the end. A single split bought
    a fixed step, as a fixed number of leaves predicts.
  - **Probit leaves:** the cluster trees started below flat, crossed it around cycle 300 and reached 75-77. T-P-Te rose
    from -57 to about -10.
  - **The two leaf types react in opposite ways to cluster-error's stream.** There the LR-leaf cluster trees never
    formed a lead, while the probit cluster trees led within the first 100 cycles. In single-split-probit they needed
    ~300 cycles to cross flat.
- **2026-11-10 08:40-09:30, why does the LR-leaf lead differ between experiments?** (archive NEXT-EXPERIMENTS §3)
  - **Partition:** the same partition in both, so it is not routing. Every NB and TAN cluster tree routes 88-99% of
    held-out positions alike, against ~54% by chance. The regions (heldout-t01, all positions):

    | Region | Share | Pieces (no kings) | Pawns | Queens | Castling rights | Material |
    |---|---|---|---|---|---|---|
    | opening | 10-12% | 28 | 15 | 1.4-1.5 | ~2 | 70-72 |
    | queenless middlegame | 18-19% | 20 | 10 | 0.2-0.3 | 0.3-0.4 | 45-47 |
    | endgame | 70-71% | 5 | 2 | 0.55 | 0 | 16 |

    (56/19/25 on decisive positions.) Error routers also split off the endgame (69-73%) but cut the rest
    differently, with 79-87% agreement. T-P-Ne has just the endgame against the rest.
  - **Partitions fixed once made:** NB and TAN cluster partitions didn't change after the split.
    - cluster-error: T-L-Nc 28/16/56 and T-P-Tc 28/57/15, identical from v150 to v321.
    - single-split: T-L-Nc 55-56/19-20/25 and T-L-Tc 52-53/21-23/25-26 from v200 to v828.
    - In single-split the merge gate undid both winning trees' first two splits (cycles 18 and 25); the third held.
  - **LR cluster routers churn** and are flat in effect (version-by-version trace). In cluster-error, T-L-Lc-merge and
    both T-P-Lc twins split and collapsed repeatedly (1 leaf v100-v305; 24/38/38 at v310; 1 leaf by v314; ...).
    T-L-Lc-merge+error=log sent everything to one leaf at every version. In single-split T-L-Lc-merge had 1 leaf from
    v400 on.
  - **The leaves are not near-copies** in either experiment (fs/analysis-2026-11-10/leafsim). Leaf-pair weight
    correlations, win-loss / draw:
    - T-L-Nc 0.94-0.98 / 0.52-0.65 (cluster-error v321) against 0.91-0.98 / 0.40-0.66 (single-split v830);
    - T-L-Tc 0.94-0.98 / 0.56-0.67 against 0.93-0.98 / 0.49-0.71.
    - In both experiments the largest leaf is most like F-L (0.99 / 0.92-0.95).
  - **The decisive rate:** 91% of own-line games are decisive in both. cluster-error's games are shorter: 96 plies
    against 108.
  - **The phase mix of the stream** (fs/phase-mix; one cycle captured): endgame / opening / middlegame 29/24/47% in
    single-split and 28/28/44% in cluster-error. Results per phase are alike too.
  - **Data volume:** cluster-error trains on 8,820 games a cycle against 2,450 (3.6×; 3.2× in positions). By games
    seen, its cycle 100 matches single-split's ~360, where the LR lead was already ~80. Yet the lead never formed there
    by any clock.
  - **Left:** what the stream is made of, and chance (one run each). In cluster-error half the lines play with probit
    leaves. split-growth's T-L-Nc-merge at limit 1 against F-L re-tests chance.
  - **Re-test (split-growth, 11-10 14:05):** the same recipe led F-L by only 10-20 at cycles 200-400 (section 4). So
    the single-split LR gain of 70-100 did not replicate: LR cluster trees against flat LR are 70-100, 10-20 and
    -25 to 26 in three experiments, while probit cluster trees gained in both of theirs.
- **2026-11-10, error routers drift early, then settle.**
  - T-L-Ne went from 50/34/17 at v100 to 59/25/16 at v830.
  - T-P-Le drifts most: from 4/55/41 at v100 to 16-20/20-22/60-62 at v600-v800.
  - T-P-Ne was 51-54/46-49/0 at every version from v100: one leaf never used. Its climb after v500 happened under a
    settled 2-way split, so it came from the probit leaves filling their regions.
- **Superseded intermediate checks** (2026-11-10 04:00, 05:40, 07:10, 08:40). Each is replaced by the final check,
  because its windows and bins rate single versions and its direct numbers include champions. The tables are in
  archive NEXT-EXPERIMENTS §3. Readings worth keeping:
  - 04:00 bins: flat F-L stopped improving by cycle ~80, while the NB and TAN cluster trees gained 50-60 over the same
    300 cycles of the shared stream, then levelled off (05:40). On the same data flat saturates early and the
    3-leaf trees turn more data into strength before they saturate in turn.
  - 05:40-08:40: the LR-leaf cluster lead in cluster-error went from 5-15 to ~0. An 08:45 reading ("a balanced split
    buys nothing") was withdrawn within minutes.
- **2026-11-10 00:25, first ratings on equal data: one leaf split costs a lot, then slowly recovers.** Tree minus its
  flat twin, mean current@2 by window; the trees split at cycle ~12:

  | Line | 1-10 | 11-20 | 21-40 | 41-60 | 61-80 | 81-88/94 |
  |---|---|---|---|---|---|---|
  | T-L-Le (LR router) | 0 | -45 | -148 | -148 | -136 | -117 |
  | T-L-Te (TAN router) | -11 | -37 | -100 | -89 | -82 | -68 |
  | T-L-Ne (NB router) | 1 | -27 | -70 | -55 | -42 | -20 |
  | T-P-Te | 9 | -8 | -18 | -66 | -77 | -66 |
  | T-P-Ne | -3 | -10 | 6 | -43 | -43 | -39 |

  - A single leaf split loses 70-150 at first. With NB routers the loss closes over ~80 cycles. This fits the user's
    point that leaf splits fragment the data.
- **2026-11-09 22:50, first shares** (cycle 30-32, held-out decisive positions; fs/routing-health/routing_health.py).
  - Every error tree had split once and used all 3 leaves.
  - LR router: 51/38/11 (LR leaves), 6/60/34 (probit). NB: 40/35/25, 48/26/26. TAN: 34/38/29, 40/15/46.
  - The routing command's recalibrated losses overflow on these models, so only shares are usable.
- **2026-11-09 18:20, degeneracy census of the champion run** (fs/degeneracy; 64 tree, DAG and mixer lineages,
  cycle 71).
  - At their latest versions, 25 of 64 were flat in effect: 14 converged leaves, 8 router to flat, 3 mixer to flat.
    At their champion versions, 31.
  - Every LR-leaf tree with --leaf-share converged to near-copy leaves. 18 of 20 merge lineages were flat or converged
    at their champion.
  - NB-routed trees without share or merge stayed healthy (15).
  - LR routers collapse with or without --revise: T-L-Le 100% to one leaf, Le-pre 99.7%, Le-merge 99.0%; T-L-Le-fresh
    98.4% to one of 5 leaves at its champion v1542, and 3 near-copy leaves at v1744. The three NB-routed fresh trees
    stayed healthy (5-11 leaves). (From the session's experiment log.)
  - "Flat in effect": whole and dominant part correlate at least 0.9922 (the correlation of flat LR with flat
    probit) on 256k cycle-70 positions. Also 2 mixers flat by weight and 1 mixer that acts as a router (M-L4-N-c).
  - 4 of the top 8 were flat in effect, T-L-Ne-share3-c included (busiest leaf alone level with the whole:
    5 [-41, 51]).
  - Busiest leaf against the whole (200-game matches, ±45):
    - the leaf won in T-L-LLs-c (145), M-L4-L-hard-c (123), T-L-NLs-merge-c (94) and T-N-LLs-merge-c (63);
    - the whole won only in M-L8-L-c (89) and D-L-Ne-rf16-c (63).
- **2026-11-09 18:11, leaf view** (fs/leafview).
  - T-L-Ne-share3-c v394 (1889.9, the champion run's best): 11 near-copy leaves, median weight correlation 0.9989.
    v394 is 58 above its previous ten versions' mean, partly luck.
  - T-L-Ne-fresh-c v1659: distinct leaves (0.986); L2 takes 60.6%.
  - Win-against-loss log-loss on decisive league positions: share3 0.3801, fresh 0.3818, F-L-c 0.3911. Neither tree
    beats its best single leaf beyond noise.
  - Both lineages' recent self-play was 91% draws.
- **2026-11-09 00:15-06:05, the twin study: the old league's tree edge was data** (fs/twin/summary.txt,
  fs/twin/league/summary.txt). T-L-Ne-share-c against flat LR twins trained on exactly its stream.
  - The self-play-only pilot (tree minus twin): 191, 236, 292 and 294 at v122, v217, v428 and v452. But that twin
    lacked the tree's league games.
  - Full-data twins (league games included), tree minus twin:

    | Twin | v122 | v217 | v452 | pooled v444-v452 |
    |---|---|---|---|---|
    | r0.05 (F-L-c's rate) | -5 | 14 | -57 | -26 [-42, -10] |
    | r0.10 | 41 | 57 | 18 | |
    | r0.165 | 99 | 130 | 82 | |

  - Pilot details: twin v452 against F-L-c v452 -19, so the twin was an ordinary F-L-c of its age. Tree v452 against
    F-L-c v452 247, against F-L-c v4100 4. The tree's kept own steps had gained a median 74 over the pre-step model
    in the 100-game gate (37 of 45 kept), which is what the user's correction (00:25) pointed at.
  - League data, 6-10% of positions, was worth 221, 212 and 302 Elo to a flat LR (twin r0.05 against the self-play-only
    twin).
  - Offline, the tree still had the better loss (0.9069 against 0.9104 at v452) and lost in play.
  - Mechanism: self-play had a fixed CPU budget and every lineage got about the same league games, so slow
    architectures trained on a larger share of league data. That flattered them.
- **2026-11-09 02:45-05:43, Q6: why T-L-Ne-share-c's busiest leaf L9 is strong** (fs/twin/league; pooled rechecks
  over v424-v428 supersede the single-version readings).
  - Stands:
    - C (all data, L9's young step, about 1.7×) against the twin: -46 [-54, -38];
    - a random 1/0.1 weighting beats the twin by 15 [7, 23], at either step;
    - R-young equals R-old: 0 [-8, 8].
  - Falls:
    - the router's region: L9 against R-young 5 [-17, 26], B against R-old -9 [-44, 27];
    - L9 against the twin, 18 [-6, 43], is about what a random weighting gives;
    - Q6's "interaction ~80" and the "router worth 23".
- **2026-11-09 ~02:30, whytrees** (fs/whytrees). Tree minus one of its leaves alone, 2000 games, own cal:

  | Model | Leaf | d1 | d2 |
  |---|---|---|---|
  | T-L-Nc-share-c v545 | L9, the busiest and offline best | 69 [59, 80] | 40 [26, 55] |
  | T-L-Ne-c v1878 | L7, the busiest | 56 [47, 65] | -12 [-26, 2] |
  | | L4, offline best | 23 [13, 32] | 171 [154, 187] |
  | T-L-Ne-share-c v428 | L9, the busiest | 46 [37, 56] | -41 [-56, -27] |
  | | L13, offline best | 51 [42, 61] | 21 [7, 36] |
  | T-L-Te-merge-c v833 | L2, the busiest | 26 [18, 35] | 10 [-4, 24] |
  | | L1, offline best | 0 [-8, 9] | 85 [70, 100] |
  | M-LP-L-c v62 | its probit expert | 43 [35, 52] | -44 [-59, -29] |

  - At d2 only T-L-Nc-share-c's routing helped. The other three champions play as one young leaf.
  - Headroom: learned flat routers never beat the trees' own routing (Ne 0.017-0.038 worse, the share trees
    0.002-0.009 worse, Te level). Re-pointing regions to their best leaf gains at most 0.005.
  - Census (Q4): NB routers and gates led (T-L-Nc-share-c 1852, T-L-Ne-c 1844, M-L4-N-c 1822). LR routers trailed, and
    TAN sat between.
  - Champions by cycle 52 (archive MORNING-2026-11-09 Q4): T-L-Nc-share-c 1859, D-L-Nc-c 1850, M-L4-N-c 1836,
    T-L-Ne-c 1828. F-L-c's new champion (cycle 51, 1830) was level with most of them. T-L-Te-c made a new champion
    at cycle 51 (1802, 71 above its last). Trees and MoEs were 15-26% league-trained a cycle, F-L-c 6%.
- **2026-11-09 ~00:25, M-L4-N-c is not one LR** (fs/l4ncmp).
  - v338 against its busiest expert: 73 [64, 83] at d1 and 46 [32, 61] at d2. Against expert 3: 126 and 92.
  - Offline, hard routing (top expert alone) is only 0.001 worse than the soft mixture.
  - Experts alone lose to F-L-c at d1 (-96, -150).
- **2026-11-08 23:45, near-flat tree leaves against F-L-c** (fs/leafcmp).
  - D-L-Nc-c v2043 plays exactly as its leaf 3: 1 [-14, 15].
  - Leaf 3 against F-L-c: 54 [39, 69] at v2043, but -25 [-40, -10] at v2064, a 79-Elo swing in 21 versions. So the
    champions' edge is mostly peak selection.
  - The leaves' AdaGrad sums are 0.19-0.69 of F-L-c's (younger, larger steps). Splits don't restart the sums.
- **2026-11-08 23:00-23:30, MoEs and near-flat trees** (fs/moespec; archive IDEAS 11-08).
  - LR-gated MoEs collapse onto 1-2 experts: M-L4-L-c uses 2 of 4, one at ~90%; M-L8-L-c 3 of 8, one at ~95%.
  - M-L4-N-c (NB gate) uses all 4 and routes almost hard: above 0.99 on one expert for 68% of positions.
  - T-L-Te-merge-c was flat LR until v776 (typical ~1759, F-L-c 1762); after its first surviving split, ~1787.
  - T-L-LLs-merge-c was flat from v2857. Its split versions averaged 1758 and its flat ones 1769.
  - D-L-Nc-c collapsed onto leaf 3 from ~v1939.
  - Cycle 43 champions: share trees 1819 (Nc) and 1831 (Ne); every MoE at or below 1790.
  - M-L4-L-hard-c (top expert alone, training unchanged) was at 989 after six cycles (v54), about 500 below
    M-LP-L-hard-c at a similar version (1480 at v53), and climbing. Unexplained (archive MORNING-2026-11-09 Q7).
- **2026-11-07/08, mixture checks** (fs/moe8, fs/tangate, fs/nbcal).
  - MoE with 8 experts uses about 4.4 of them and costs 1.6-1.8× MoE-c.
  - A TAN gate over LR experts is as hard as the NB gate and worse than MoE-c offline.
  - Calibrating NB's self-play policy on its own games doesn't change how randomly it plays.
  - 11-07: MoE-c's gate flipped the whole model between an expert trained on tournament games and one trained on its
    depth-0 self-play (v300 → v336: 93% of positions on expert A → 99% on expert D; 1744 → 1368). The gate told data
    sources apart, not chess situations.
- **2026-11-08 ~16:00, LR error routers lock in** (fs/balance).
  - T-L-Le-c and T-L-Le-pre-c send 97-100% of each self-play round to one leaf, a different one after each own step.
  - NB routers spread every round (4.5-5.5 effective leaves).
  - The cell DAG D-L-Nc-c collapsed the same way: about 98% to leaf 3.
- **2026-11-08 ~14:30, routercmp** (fs/routercmp). NB and LR routers on the same fixed leaves differ by at most 0.0017
  nats.
  - LR routers use fewer leaves in all 4 settings. Effective leaves, NB / LR: 6.4/2.0, 6.2/4.3, 5.5/4.6 and 2.6/1.0.
  - Live unshared LR-router error trees ran on 1.5-1.8 effective leaves.
- **2026-11-08 ~14:00, tree balance** (fs/balance; cycle 30's self-play routing).
  - Cluster trees stay fairly balanced. T-L-Nc-c: 11 leaves, the largest 20%, 8.5 effective.
  - Error trees concentrate. T-L-Ne-c: largest 47%, 5.0 effective, two dead leaves.
  - Pre-splitting balances the start of a split, not where it ends (6.1 effective).
  - Balance doesn't track strength.
- **2026-11-08 ~14:20, split phases of merge trees** (balance/splitphase.py).
  - T-L-Te-merge-c's split-phase entries rated 38.6 ± 12.3 above its flat ones.
  - Others were mixed: T-L-Ne-merge-c 9.8, T-L-Nc-merge-c 14.0, T-L-NNs-merge-c 16.7, T-L-LLs-merge-c -30.6, and the NB-leaf
    trees -12 to -74.
- **2026-11-07, optimism** (fs/optimism; cycle 15's league games).
  - Seams are not exploited: LR trees overrate crossings less than other positions (0.5-2.9 points at d1 and d2).
  - Selection optimism at d1: flat LR 0.3-1.0 points (mean 0.64), structured LR trees 0.9-1.7, NB 1-3. At d2 the
    opponent's reply absorbs it (-0.3 to 0.2).
  - So the trees' d2 deficit came from leaves that each learn from a fraction of the data, not from search
    exploiting seams.
- **2026-11-06/07, leaf disagreement tracks play, router regret doesn't** (`chessai routing`; archive IDEAS "Done
  overnight" and idea 16).
  - From-scratch trees at v1000: root regret 0.0000-0.0001, lower delegators up to 0.007 (soft-target NB routers
    0.15-0.29). So the routers were not the problem.
  - Mean leaf-pair |log-odds difference|: LR trees T-L-Nc 0.74, T-L-Ne 0.80, merging T-L-Ne 0.47 (5 leaves), the old
    plateau-start T-L-Nc 0.41. NB trees: T-N-Nc 1.91, with sharing 0.75 (158 better), T-N-Ne 0.64, T-N-Ne soft 3.98
    (-302).
  - Old plateau-start T-L-Nc v300: root regret 0; sibling leaves 7-9 near-duplicates (0.08-0.20 against 0.41 for an
    average pair). The waste was splits that didn't specialise.
  - League, 11-07: merging LR trees collapsed back to 1-3 leaves that barely disagree (0-0.15) and rated level with or
    above flat LR (T-L-Ne-merge-c v794 1718, T-L-NNs-merge-c 1714, F-L-c 1676 at d2). Non-merging LR trees kept 11
    leaves that disagree a lot (0.52-0.78) and rated 43-115 below flat LR.
- **2026-11-05/06, the first trees** (LEDGER.md; archive README):
  - From a plateau start (mix1000), T-L-Nc beat its LR control by 29 and 30 (seeds 21 and 22), and T-L-Ne with log
    error by 27 and 23.
  - From scratch every tree lost to its flat model until it could undo splits (section 4).
  - LR cluster routers collapsed to one leaf. Soft error targets routed by who is winning.
  - No NB tree beat the ÷16 NB baseline.

---

## 4. Splits, merges, triggers and growth

Where it stands:
- **The LR anchor (11-11 ~09:15, offline): no strength helps.** At each split the children keep a frozen copy of
  the parent's weights, and every training step pulls the weights it used back toward it (`--split-anchor-l2 L`,
  bin/chessai-hybrid6an; byte-identical to hybrid6 when off). It is the LR counterpart of the probit trees' split
  noise 0.03, and matching that width gives L* = 1.3e-3 at a 1M threshold.
  - Ply-2 l2sweep testbed (no ply-1 stream is saved), 3 seeds × 60 rounds, fresh log-loss against no anchor: L 0.0013
    +0.0007, 0.004 +0.0029, 0.012 +0.0028, 0.04 +0.0031 (worse). No anchor is 0.0078 better than flat.
  - From L* up, the fresh gate merges the whole tree back to its root at round 33 in every seed.
  - Write-up: models/from-scratch/lranchor/summary.txt.
  - **The league test (G5) is dropped** (theory-2 6.24). A tether and a fresh gate on the same window work against
    each other. At L* the tethered children can't beat the shadow inside a window, so the gate merges the tree;
    the one strength that never merges (4e-4) does nothing. The probit widening worked partly by plasticity
    (widening 0 gained 95 too), the opposite of a tether. Revisit at ply 2 with --revise 2 or a longer window,
    L in {4e-4, 1.3e-3}. chessai-hybrid6an is kept.
- **A leaf split costs before it pays.** On equal data it costs 70-150 at first and pays after ~40-120 cycles with
  cluster routers (theory: break-even near a million positions per leaf).
- **Merges.**
  - The merge test that scores the stream it trains on is biased. Scoring each round before training on it (fresh)
    fixes its sign.
  - Offline (11-11 00:10, the analyst, theory-2 section 6.5): with the recipes' stream gate (--revise 0) a limit-4 LR
    tree split 6 times, merged 6 times and ended with one leaf (-0.0006 nats against flat); with --revise-on fresh
    (at --revise 0 or 2) it never merged and matched no-merge copies (9 leaves, -0.0077). So the "-merge" recipes'
    gate can undo gaining splits (growth-limits-p1 oscillated the same way). New merge-gated lines use
    --revise-on fresh from 11-11.
  - Merges are banned for error routers (the user, 11-08 21:10). Cluster routers may merge.
- **Router splits (DAG)** never beat a tree with the same leaves offline, across five studies and seven variants.
  Home targets (P2) make router splits harmless when they happen. The trigger still fires on a rise that happens
  anyway.
- **Split trigger T4** (gradient disagreement, 11-10).
  - Offline it beat today's trigger with fewer splits, and with cluster routers it beat a same-leaves tree.
  - In the league (trigger-gradient: LR, limit 8, 400 cycles) it tied today's trigger: champions 1, -7, -4 ±16. It
    made 3 splits to today's 4.
  - Both trees ended level with flat at ply 2 (-4 to 7 in cycles 300-400) and 27-48 ahead of it at ply 1.
- **Residual leaves** (residual-leaves, the clean LR rerun at limit 4): no gain. They led the copy tree only in cycles
  51-150 and ended 38 behind it. Offline, shrinkage (--residual-l2) only makes them less bad: they move toward flat,
  never toward copies (the S1 sweep, 11-10 23:25, entry below).
- **Growth** (split-growth, corrected 11-10 ~18:50).
  - LR at limit 4 with merges ended 34-53 ahead of flat, the most of the four trees.
  - Without merges, limit 4 trailed by 59-91 to cycle 200, then caught up.
  - In probit (split-growth-probit, finished at cycle 1000) both limit-4 trees stayed behind F-P to the end: 17-62
    at ply 2 and 26-47 at ply 1 over cycles 701-1000. At limit 1 both trees led at ply 2 (T-P-Ne 39-48 from cycle 500
    on, T-P-Nc-merge 20-51 late) and were level at ply 1 (-5 to 6).
- **Field size** (small-field, 2 lines, finished 19:39): the single-split tree T-L-Tc-merge trailed F-L at ply 2 until
  cycle ~200, then led by 20-29 over the last 150 cycles; at ply 1 it led from cycle ~110 (42-47 late). At the same
  cycles the same recipe in the 7-line single-split led by 26-93 more at ply 2, but only 8-28 more at ply 1. One run
  each; small-field-x4 (queued) measures the seed spread.
- split-growth-probit and error-targets-probit (old design) finished at the 1000-cycle cap at 21:36 and 21:47 on
  11-10: entries below and in section 5.

Entries:
- **2026-11-11 00:01, the first two ply-1 experiments finished** (entries, training and champion test at ply 1;
  self-games; no references; plateau windows scaled by lines).
  - small-field-x4-p1 (F-L ×4 against T-L-Tc-merge ×4, seed 11, 10 rounds a cycle) stopped at cycle 190 (no new
    champion for 25 cycles). Current against current, direct Elo against F-L#1, 600-800 games a cell:

    | Line | 1-40 | 41-80 | 81-120 | 121-160 | 161-190 |
    |---|---|---|---|---|---|
    | F-L#2 | 3 | 3 | 5 | -1 | 14 |
    | F-L#3 | 0 | 7 | 7 | -1 | 14 |
    | F-L#4 | 3 | -2 | -11 | 13 | 11 |
    | T-L-Tc-merge#1 | -5 | -5 | -3 | -18 | 10 |
    | T-L-Tc-merge#2 | 4 | 11 | 3 | -15 | 9 |
    | T-L-Tc-merge#3 | -4 | 10 | 2 | 1 | 5 |
    | T-L-Tc-merge#4 | 7 | 13 | -8 | -8 | -2 |

    The four tree seeds sit inside the four flat seeds' spread (-11 to 14) all run long: no tree gain in a pure
    ply-1 league within 190 cycles. The trees split at versions 30-60 (the analyst), so they had 130-160 cycles
    after the split. In the old field the same recipe led F-L at ply 1 from cycle ~110 (small-field, 42-47 late),
    but those models trained on ply-2 and mixed games.
  - growth-limits-p1 (T-L-Nc-merge at split limits 1 / 4 / 8, 2 seeds each, seed 12, 18 rounds a cycle) stopped at
    cycle 340. All six lines stayed within -20 to 11 of limit 1 seed 1 in every 60-cycle window. Not a test of the
    limit: the stream-scored merge gate took the limit-4 and limit-8 trees from 9 leaves (v100) back to 3 (v120),
    then they regrew (about 7 at v130), and same-seed lines at limits 4 and 8 stayed identical (11-10 ~23:20).
- **2026-11-10 23:25, the S1 shrinkage sweep, offline** (models/from-scratch/theory-2/l2sweep/, run by the analyst
  agent). The testbed every offline study used: bddstudy/train offline-c15, 60 rounds of ~115k ply-2 positions from
  the old field's cycle-15 round robins (residual-leaves' own games were deleted when its run ended). T-L-Nc, limit 4,
  threshold 300k so that the 4 splits happen within 60 rounds, seed 1, chessai-hybrid6. Every model is scored on the
  next round before training on it.
  - Fresh log-loss against flat (nats; negative = better), over all rounds and over rounds 51-59: copies -0.0077 and
    -0.0138. Residual leaves with inherited sums at l2 0 / 0.1 / 1: 0.0044 / 0.0033 / 0.0004 and -0.0010 / -0.0013 /
    -0.0028. Without inherited sums: 0.0233 at l2 0 down to 0.0005 at l2 1.
  - Copies had no dip after splits: they gained at once (-0.0033 in the 5 rounds after the first split, -0.0041 in the
    next 5). Residual leaves fell behind flat after the first split: 0.0088 and 0.0133 at l2 0, 0.0030 and 0.0037
    at l2 1.
  - Leaf noise (SD across versions 55-59 of the log-odds on round 59): flat 0.231, copies 0.224, residual leaves with
    inherited sums 0.338 at l2 0 and 0.301 at l2 1, 30-50% noisier, as the analyst's simulation said.
  - So shrinkage moves residual leaves toward flat, never toward copies, and S1 isn't worth a ply-1 experiment in this
    form. The copies' immediate gain also means the league's post-split dip doesn't come from how copied leaves
    learn, at least on this testbed. One seed, one testbed, early (cycle-15) games.
- **2026-11-10 21:36, split-growth-probit finished** (1000 cycles, seed 3; the old field, with random players and
  ply-1 entries; 9 rounds a cycle; F-P, T-P-Nc-merge and T-P-Ne at split limits 1 and 4, local).
  - Current against current, direct Elo against F-P by 100-cycle window, 1,800 games a cell (xtab/curtab.py, and a
    copy filtering ply-1 entries instead of ply-2; analysis 11-10 ~22:40):

    | Line | Ply | 1-100 | 101-200 | 201-300 | 301-400 | 401-500 | 501-600 | 601-700 | 701-800 | 801-900 | 901-1000 |
    |---|---|---|---|---|---|---|---|---|---|---|---|
    | T-P-Nc-merge, limit 1 | 2 | 3 | 13 | 13 | 20 | 20 | 12 | 22 | 27 | 51 | 20 |
    |  | 1 | -6 | 7 | -4 | -8 | 4 | 0 | 6 | 6 | 4 | -5 |
    | T-P-Nc-merge, limit 4 | 2 | -10 | 3 | -45 | -54 | -41 | -44 | -24 | -29 | -17 | -39 |
    |  | 1 | -5 | 0 | -29 | -30 | -37 | -33 | -19 | -26 | -30 | -39 |
    | T-P-Ne, limit 1 | 2 | 0 | 5 | -2 | -10 | 14 | 39 | 36 | 40 | 48 | 40 |
    |  | 1 | 3 | 0 | -10 | -11 | 1 | 2 | 13 | 3 | 4 | -2 |
    | T-P-Ne, limit 4 | 2 | -15 | -56 | -93 | -73 | -86 | -73 | -74 | -62 | -51 | -48 |
    |  | 1 | -1 | -18 | -37 | -44 | -32 | -46 | -42 | -47 | -30 | -27 |

  - The NB error tree at limit 1 (T-P-Ne) replicated single-split-probit's late gain: level to cycle 400, then
    39-48 ahead at ply 2 from cycle 500 to the end (single-split-probit: 48). At ply 1 it stayed level (-11 to 13),
    as there (0).
  - Both limit-1 probit trees gained at ply 2 only. A ply-1 screen would have dropped both (section 6).
  - Growth to limit 4 cost both probit trees at both plies to the end, unlike LR with merges (34-53 ahead in
    split-growth).
  - One run, in the old field, so batch 4's field-composition confound applies (sections 7-8).
- **2026-11-10 19:39, small-field finished** (400 cycles, seed 10; F-L and T-L-Tc-merge with single-split's
  arguments; references random and random-3 only; 56 rounds a cycle, 2,464 local games as in single-split; exp
  small-field). The user's idea: trees gain more from local training when there are more competitors.
  - Tree minus F-L, Elo, current@2 against current@2 (small-field ~5,600 games a cell, single-split ~500):

    | Run | 5-50 | 51-100 | 101-150 | 151-200 | 201-250 | 251-300 | 301-350 | 351-400 |
    |---|---|---|---|---|---|---|---|---|
    | small-field (2 lines) | 2 | -27 | -18 | -9 | 1 | 20 | 25 | 29 |
    | single-split (7 lines) | 28 | 1 | 36 | 21 | 65 | 96 | 118 | 64 |

  - Both plies by quarter (plyfid; small-field 12-108, 109-205, 206-302, 303-400; single-split 12-109 to 308-406):

    | Run | Ply | Q1 | Q2 | Q3 | Q4 |
    |---|---|---|---|---|---|
    | small-field | 1 | 2 | 29 | 42 | 47 |
    | small-field | 2 | -15 | -12 | 9 | 28 |
    | single-split | 1 | 14 | 57 | 50 | 62 |
    | single-split | 2 | 16 | 30 | 87 | 89 |

  - Champions at ply 2, the tree's minus F-L's: -26, -13 and 7 (±6) over cycles 101-200, 201-300 and 301-400.
    Crownings: F-L 27 (the last at cycle 391), the tree 28 (the last at 320).
  - Reading:
    - The tree caught up late at ply 2 and led clearly at ply 1 from cycle ~110, the pattern of the other LR cluster
      trees (section 1).
    - The 7-line run's tree led by 26-93 more at ply 2, but by only 8-28 more at ply 1. That fits the user's idea at
      ply 2, but it is one run against one other with different seeds, and one recipe's gap has moved 50-90 between
      runs (section 3). small-field also had more games against the random players: 36% of local games, against
      single-split's 23%.
    - small-field-x4 (4 seeds of each line in one field) measures the seed spread for this pair directly.
- **2026-11-10 18:19, trigger-gradient finished** (400 cycles, seed 9; T-L-Nc limit 8, no merges; today's trigger
  against T4 at --split-gradient 9.5; exp trigger-gradient).
  - Current@2 against current@2, Elo against F-L, ~900 games a cell:

    | Line | 12-50 | 51-100 | 101-150 | 151-200 | 201-250 | 251-300 | 301-350 | 351-400 |
    |---|---|---|---|---|---|---|---|---|
    | T4 trigger | -43 | -19 | -15 | -19 | -16 | -1 | 7 | -3 |
    | today's trigger | -9 | -3 | -4 | -16 | -33 | -26 | -10 | -4 |

  - Champion against champion (1,800 games each, cycles 101-200 / 201-300 / 301-400):
    - T4 against today's: 1, -7 and -4 (±16);
    - T4 against F-L: -16, -47 and -14;
    - today's against F-L: -26, -46 and -16.
  - No line crowned after cycle 297 (F-L last at 236, today's at 273, T4 at 297). The plateau rule would have
    stopped the run at ~347.
  - Splits (training logs): T4 split 3 times (the root at window 3, then leaves 1 and 5). Today's trigger made 4
    delegators.
  - At ply 1 (section 1): T4 was 48 ahead of F-L in the last quarter and today's 27 ahead. At ply 2 they were 3
    ahead and 8 behind.
  - Reading: the offline advantage of T4 did not carry into play. It splits less, at no cost and no gain. For LR
    leaves without merges, neither trigger beats flat at ply 2.
- **2026-11-10 16:22, residual-leaves finished (T3, the clean LR rerun; 400 cycles, seed 6; no merges, limit 4).**
  Direct games, current@2 against current@2, ~900 per window (corrected 11-10 ~18:50: the first table counted 5
  rounds a cycle, and this run plays 9, so its windows were really cycles ~7-222; section 1 has the bug):

  | Cycles | 12-50 | 51-100 | 101-150 | 151-200 | 201-250 | 251-300 | 301-350 | 351-400 |
  |---|---|---|---|---|---|---|---|---|
  | residual tree against copy tree | -14 | 27 | 29 | -3 | -3 | -20 | -9 | -38 |
  | F-L against copy tree | 1 | 42 | 47 | 32 | 4 | 12 | -10 | -4 |

  - Champion against champion (residual against copy, 1,800 games each): 13±16 (cycles 101-200), -20±16 (201-300),
    -18±16 (301-400). The copy tree kept crowning (cycles 322, 346, 382); the residual tree's last was at 350.
  - Reading (redone): residual leaves led the copy tree only in cycles 51-150, then fell behind (38 below at the
    end). No gain for LR at limit 4. The copy tree trailed flat by 32-47 in cycles 51-200 and was level with it from
    cycle 200, as the same line was in split-growth (corrected).
  - The probit twin (residual-leaves-probit) is built and ready.
- **2026-11-10 14:05, split-growth finished** (400 cycles; exp split-growth; spec batch3/7-split-growth.json). Elo
  against F-L, current@2 against current@2, ~900 games per window (corrected 11-10 ~18:50: the first table counted 5
  rounds a cycle, and this run plays 9, so its windows were really cycles ~12-222; section 1 has the bug):

  | Line | 20-49 | 50-99 | 100-149 | 150-199 | 200-249 | 250-299 | 300-349 | 350-400 |
  |---|---|---|---|---|---|---|---|---|
  | T-L-Nc-merge, limit 1 | -15 | -7 | 16 | 6 | 30 | 14 | 43 | 35 |
  | T-L-Nc, limit 1 | -10 | -21 | 27 | 15 | 4 | 28 | 31 | 34 |
  | T-L-Nc-merge, limit 4 | 3 | -15 | 9 | 0 | 1 | 23 | 34 | 53 |
  | T-L-Nc, limit 4 | -91 | -91 | -59 | -66 | -25 | -1 | -2 | 11 |

  - Readings, redone (the first ones were wrong):
    - The single-split LR gain (70-100) half replicated: the same recipe leads by 14-43 at cycles 200-400, by 35-43
      in the last 100, and was still rising.
    - At limit 1 the merge gate changes little (31-43 ahead either way at the end).
    - At limit 4 without merges the tree reached 9 leaves by v50, trailed flat by 59-91 to cycle 200, and caught up
      by cycle 250 (-2 to 11 after).
    - With merges it grew slowly (3, 7, 5, 9 leaves at v50, v100, v200, v300) and ended furthest ahead: 34-53 in
      cycles 300-400. Growth to 9 leaves with the gate paid off late.
    - Reading: NB partitions are fixed once made, so early splits lock in a partition learned on early games; the
      merge gate works as a redo.
  - The residual rerun's comparator is this worst line (T-L-Nc limit 4, no merges).
  - The table script used to pool lines whose names differ only by "-merge"; fixed 14:15 (only split-growth had the
    clash).
- **2026-11-10 ~15:20, T4 gradient-disagreement split trigger** (bin/chessai-trigger, --split-gradient Z; src
  models/from-scratch/trigger-src; write-up models/from-scratch/trigger/summary.txt).
  - Design: each leaf trains a label-free probe router on the side. At each window's end, the regions' mean gradients
    minus the leaf's give directions, and the next window's region gradients are projected on them. The z is
    clustered by game. Off is byte-identical to hybrid6 and reproduces bddstudy exactly.
  - Offline, threshold 100k, limit 10, 2 seeds, Z 3. Held-out log-loss ×1e-3 on RR 84; negative favours T4:

    | | T4 − today, round 20 | round 40 | round 60 | T4 − same leaves, round 60 |
    |---|---|---|---|---|
    | T-L-Nc | -1.7 [-4.1, 0.8] | -4.8 [-7.2, -2.4] | -5.3 [-6.9, -3.7] | -4.5 [-6.4, -2.7] |
    | T-L-Ne | -2.8 [-4.3, -1.3] | -12.1 [-15.6, -8.5] | -3.1 [-5.1, -1.1] | -1.5 [-3.4, 0.5] |

  - T4 split 5-6 times where today's trigger split 10. It split the root at its second test (z 5.5-5.7), then the
    root's children. The leaves it left alone sat at z 1-3 test after test. It costs ~20% more training CPU.
  - With cluster routers it is the first structure to beat a same-leaves tree offline, in each seed alone.
  - In the league: z grows with the square root of the window, so Z 3 at 1M is a lower bar than offline; Z 9.5 keeps
    the offline bar. Draft spec batch3/14-trigger-gradient.json, not created.
- **2026-11-10 11:39 and 12:31, split-growth and split-growth-probit launched.** Designs are in section 7.
- **2026-11-10 ~02:00, path context** (the user's sibling-set design; bin/chessai-bddctx; fs/bddctx).
  - 164 tests pass. Without the flag it is byte-identical to bddhome2. The tournament must run on chessai-bddctx.
  - Mechanism check (seed 1): both arms made 7 router splits and 3 leaf splits.
    - Cells shift LR leaves' log-odds by 0.42 on average, of which 0.13 differs by path.
    - CNBP router cells flip 7% of router decisions: double counting.
    - 58% of positions reach their leaf by a non-home route by round 60.
    - Held-out loss level with the control from round 40.
  - Not linear in growth. In a 200-split test, nodes went from 76 to 601 and table numbers from 6.3k to 259k. Linear
    variants (the parents' sets only, or the K nearest) were proposed, not built.
- **2026-11-10 ~00:00, P2 dag-home built** (bin/chessai-bddhome2, --subrouter-target home; fs/bddhome). Offline,
  one seed:
  - R' doesn't collapse: the root stayed at 38/48/14 at round 60, against 39/41/19 before the split. The plain DAG's
    root drifted to 42/56/2 by round 8.
  - The copies part because they see different positions. They overrule their home pick on 4% at round 4 and 35-40%
    from round 30.
  - Held-out loss level with a same-leaves tree after round 20, and about 0.001 better than the plain DAG at rounds
    40-60.
  - 6 of 10 splits were router splits: the root became a chain of 5 routers.
- **2026-11-09 ~03:35 and morning, DAG variants in the live league** (age-matched against the fresh tree T-L-Ne-new-c):

  | Lineage | age 1 | 2 | 3 | 5 | 6 |
  |---|---|---|---|---|---|
  | T-L-Ne-new-c (fresh tree) | 1149 | 1402 | 1496 | 1631 | 1620 |
  | D-L-Ne-rf16-c | 1093 | 1406 | 1497 | 1658 | 1664 |
  | D-L-Ne-skill-c | 1168 | 1398 | 1488 | 1649 | 1616 |
  | D-L-Ne-skip-c | 1106 | 1395 | 1474 | 1579 | 1638 |
  | D-L-Ne-lreg-c | 1124 | 1352 | 1447 | 1562 | 1595 |

  - At cycle 52: skill 1656, fresh tree 1647, lreg 1637, skip 1635, rf16 1627. None is ahead.
- **2026-11-08 ~17:45-21:35, DAG router-split studies** (fs/bddstudy, bddregret, bddfix, bddtrig; offline-c15,
  threshold 100k, limit 10, 60 rounds, 2 seeds). Held-out RR 84 at round 60, arm minus the same-leaves BDT:

  | Study | Arm | Result |
  |---|---|---|
  | bddstudy | plain BDD | 0.0010 [-0.0002, 0.0021] |
  | bddregret | A: split on regret | -0.0012 [-0.0027, 0.0003] |
  | | B: R' trained on its sub-routers' leaves | -0.0008 [-0.0023, 0.0006] |
  | | A+B | -0.0008 [-0.0017, 0.0001] |
  | bddfix | A' + B' + C together | 0.0005 [-0.0011, 0.0020] |
  | bddtrig | C alone (router copies forget 16×) | -0.0005 [-0.0021, 0.0010] |
  | | skill trigger | -0.0011 [-0.0029, 0.0007] |
  | | 2-SE margin | byte-identical to the plain BDD |

  - In the BDD, the first router split lowers the root's regret (-0.0084) but not the tree's loss: the children grow
    more alike instead. Siblings reach the same leaf on 94-98% of positions one round after a split.
  - bddfix's forgetting copies routed worse at first (tree loss up 0.005-0.010 at rounds 4-8). Its rising regret
    fired more splits.
  - A ridge router fitted to each leaf's loss never beat the trees' own routing.
  - In the live league, R' of D-L-Ne-c was a pass-through: its copies agree on 97%.
  - Why router splits don't pay: the trigger fires on a climb that happens anyway (the root split in round 4 in all
    10 runs), and copies add no routing skill. Leaves past 7 don't help on this data (BDT-3 ties BDT-10).
- **2026-11-08 ~15:45, bddcheck** (fs/bddcheck): BDD minus BDT -0.0018 [-0.0043, 0.0006] in log-loss; winner loss
  0.0001. 3 leaf splits and 2 delegator splits, 7 leaves each.
- **2026-11-07 ~01:45-02:40, residual leaves** (bin/chessai-resid2 and resid4; fs/resid). A child's LR is its
  parent's live shadow plus a delta that starts at 0.
  - In 60 rounds from scratch at a 300k threshold, the residual trees kept their splits (the stream merge test
    favoured them in 110 of 110 windows). But they played 171-266 below the plain merging tree at the same version.
    Held-out loss was worse too: 0.594-0.595 against 0.558.
  - Likely cause: each example steps both the shadow and the delta. That pays on the round being trained on but not on
    new games, so the stream test rewards it.
  - With fresh scoring (--revise-on fresh), the inherit variant at v60 went from 171 below to 16 below, not
    significant.
  - Theory 7.7 fits the overconfidence: fresh deltas take full-size AdaGrad steps (--residual-inherit-sums and
    --residual-delta-rate exist for this).
  - Offline (offline-c15): residual leaves with inherited sums and merging scored 0.6263, against flat 0.6220.
  - No clean one-factor run was made, hence T3.
- **2026-11-07 ~01:55-04:00, merge tests offline** (fs/offline-c15: trained once on cycle 15's round-robins 80-83,
  about 2.4M positions, scored on RR 84).
  - Benchmark (log-loss / winner log-loss):
    - LR on 1/3, 2/3 and all of the data: 0.6380 / 0.3405, 0.6274 / 0.3337, 0.6220 / 0.3293;
    - LR, 2 passes: 0.6197 / 0.3271;
    - T-L-Nc at 300k (3 leaves) 0.6199; at 100k (9 leaves) 0.6178;
    - T-L-Nc 100k with stream merging: merged to 1 leaf, 0.6220;
    - T-L-Ne at 300k 0.6226, at 100k 0.6242;
    - naive Bayes 0.7328;
    - league self-play models 0.6386-0.6517.
  - **The stream merge test discarded the better 9-leaf tree.** It is biased both ways: with copied leaves it favours
    the shadow, which sees every example, and with residual leaves it favours the branch.
  - Fresh scoring (each round scored before training on it) reverses the stream test's sign in every run. At z 0 it
    still merged T-L-Nc-100k on one bad window (1 leaf, 0.6220); at --revise 2 it kept 5 leaves at 0.6170, the best
    on the testbed, but fragile (the root's worst window was z -1.96).
  - The league's own T-N models predicted league games no better than F-N-c unless they had collapsed (F-N-c 0.7456;
    11-leaf trees 0.754-0.835; 1-leaf T-N-NLs-merge-c 0.7264).
  - Overnight table (100k threshold; no merge / stream / fresh):
    - T-L-Nc 0.6178 / 0.6220 (1 leaf) / 0.6170;
    - T-L-Ne 0.6242 / 0.6199 / 0.6206;
    - T-L-Le 0.6257 / 0.6309 / 0.6220 (1 leaf);
    - T-N-Nc 0.7211 / 0.7328 (1 leaf) / 0.7169;
    - T-N-Le 0.8858 / 0.7328 (1 leaf) / 0.7342.
  - Data matters more than structure here: 3 times the data is worth 0.016, the best structure 0.004.
  - Models trained on league games predict league games far better than self-play models (0.62 against 0.646-0.652).
- **2026-11-07, pre-split candidates and the cell-assigned DAG** (archive IDEAS 11-07 "Results").
  - Pre-split (the user's idea: train the would-be split in the shadow before installing it): without merging the
    error trees reach flat LR (Ne 0.6242 → 0.6219, Le 0.6257 → 0.6246), and splits stop being lopsided (a 2/27/71
    split becomes at worst 47/12/41). With fresh merging Ne and Le collapse to flat; the best switch tree (NNs) came
    from pre-split plus fresh (0.6171). Promotion by the fresh test never fired for Ne or Le. Pre-split balances the
    start of a split, not where it ends. It costs about 1.3× CPU per game.
  - Cell DAG: at z 2 no cell was re-pointed in 20 rounds, because the incumbent always predicts its cell best.
    Offline 0.6230. In the league it later collapsed onto one leaf.
- **2026-11-07/08 to 11-09, leaf sharing** (fs/lrshare, leshare, sharesize, swshare).
  - Offline, sharing made no-merge LR error trees better: T-L-Ne -0.0058; T-L-Le -0.0047 (0.1) and -0.0063 (0.3).
    It halves leaf disagreement and shrinks seams. With merges it changes nothing.
  - Dose sweep, log-loss at 0.1 / 0.3 / 0.5 / 1.0 (flat LR 0.6116):
    - Le 0.6111 / 0.6095 / 0.6101 / 0.6116;
    - Ne 0.6086 / 0.6075 / 0.6087;
    - Nc 0.6067 / 0.6074 / 0.6086.
  - With sharing, trees of 25-35 leaves are as good as 7-11 (without it they lose 0.005-0.010), but the extra leaves
    gain nothing on 2.4M positions (sharesize).
  - Switch trees and the DAG cleared no bar (swshare: NLs = NNs offline 0.1 -0.0018 [-0.0031, -0.0005], 0.3 -0.0016;
    LLs 0.0000 and -0.0001; D-L-Nc 0.1 0.0000, 0.3 -0.0014).
  - 11-05, NB leaves: share 0.1 against none 12 at d2 (T-N-Nc from ÷16); from scratch 158, and 168 with merges against
    merges alone (LEDGER).
  - In the champion run, every LR-leaf share tree converged to near-copy leaves (degeneracy, section 3).
- **2026-11-07/08, Polyak averaging** (fs/polyak).
  - avg5 against a single version: T-L-Le-c 40 at ply 1 and -11 at ply 2; T-L-Ne-c 7 and 28. Version-to-version
    swings shrink 3-5×.
  - Held-out loss -0.005 (b-hard) and -0.001 to -0.002 (T-L-Ne-c, T-L-Nc-c). In a young run (EMA 0.8, v22-v26) -15
    at ply 1: the lag costs while the model improves fast.
  - The window check found the T-L-Le-c result was phase, not noise reduction. Within one cycle's self-play versions,
    avg5 is level (ply 2 8 [-13, 29], ply 1 -6 [-15, 4]). Windows ending on an own-step version: ply 2 -102, ply 1 96.
  - Halfway in weights between an own-step and a post-self-play version was worse than either end (1538 against 1695
    and 1602).
- **2026-11-06, merges from scratch** (LEDGER "From scratch").
  - Trees without merges lost to their flat models: LR trees -69 to -144, NB trees -129 to -476.
  - Merges (--revise 0) against their twins: LR trees 69 to 129, T-N-Nc 41, T-N-NNs 104 and 190, T-N-Ne -52.
  - Merging T-L-Ne reached 36 [26, 47] above F-L. Leaf resets added nothing on top of merges (-9).
- **2026-11-05/06, split rules** (LEDGER "Trees: when to split").
  - Settled: rule worse; branch factor 3; error log for LR-leaf error trees (T-L-Ne log 27 against abs 14); forgetting
    at splits 1.
  - Split trials were closed (-24).
  - Hoeffding trees were closed on 11-09: the bound never decides. Offline they were -90 (LR) and -200 (NB) against
    flat.

---

## 5. Error routing and router targets

Where it stands:
- **Theory (11-10):** the Best, Soft and Balanced targets are inconsistent for routed log-loss, and every error router
  is an evaluator in disguise.
  - Consistent targets (pairwise gap, fractional labels) are built in bin/chessai-target (T1).
  - Offline, gap stops LR routers routing by who's winning and spreads the data. It is level with Best only together
    with --route-before-train.
- **NB error routers route like label-free clustering** (U 0.02-0.12). LR error routers route by who's winning
  (0.14-0.53) and lock in.
- **error-targets (LR) finished:** every T-L-Le variant stayed below flat F-L. Gap targets recovered 29-80 Elo of
  Best's loss and ended 19-29 below flat; route-before-train alone was level with Best (corrected 11-10 ~18:50).
  error-targets-probit (finished at cycle 1000, 21:47): every variant ended 104-143 below F-P at ply 2 and 36-84
  below at ply 1, and gap targets were 20-30 worse than Best there.

Entries:
- **2026-11-10 21:47, error-targets-probit finished** (1000 cycles, seed 5; the old field; 9 rounds a cycle; F-P and
  T-P-Le at limit 1: Best, then route-before-train, gap targets and a split bootstrap added one per line; lines 3-5
  train on bin/chessai-target).
  - Current against current, direct Elo against F-P by 100-cycle window, 1,800 games a cell (as for
    split-growth-probit, section 4):

    | Line | Ply | 1-100 | 101-200 | 201-300 | 301-400 | 401-500 | 501-600 | 601-700 | 701-800 | 801-900 | 901-1000 |
    |---|---|---|---|---|---|---|---|---|---|---|---|
    | T-P-Le (Best) | 2 | -23 | -60 | -75 | -117 | -132 | -122 | -153 | -111 | -119 | -119 |
    |  | 1 | -12 | -28 | -62 | -46 | -45 | -51 | -60 | -42 | -36 | -44 |
    | route-before-train | 2 | -42 | -43 | -88 | -108 | -103 | -106 | -129 | -104 | -105 | -121 |
    |  | 1 | -17 | -23 | -52 | -55 | -53 | -46 | -50 | -51 | -38 | -40 |
    | and gap targets | 2 | -89 | -139 | -129 | -113 | -119 | -127 | -121 | -137 | -139 | -142 |
    |  | 1 | -45 | -84 | -62 | -55 | -67 | -69 | -71 | -68 | -70 | -77 |
    | and a split bootstrap | 2 | -100 | -147 | -117 | -112 | -123 | -120 | -132 | -143 | -127 | -137 |
    |  | 1 | -42 | -83 | -61 | -46 | -65 | -72 | -66 | -84 | -70 | -74 |

  - Over cycles 701-1000 every LR-routed probit tree was 104-143 below F-P at ply 2 and 36-84 below at ply 1.
  - Gap targets didn't help with probit leaves: 20-30 worse than Best at both plies, all run long. With LR leaves
    they had recovered 29-80 (error-targets).
  - route-before-train alone was level with Best, as with LR leaves.
- **2026-11-10 17:11, error-targets finished** (400 cycles, seed 4; lines 3-5 on bin/chessai-target).
  - Current@2 against current@2, Elo against F-L (1,584-1,800 games a cell; corrected 11-10 ~18:50: the first table
    counted 5 rounds a cycle, and this run plays 9, so its windows were really cycles ~7-222; section 1 has the bug):

    | Line | 13-100 | 101-200 | 201-300 | 301-400 |
    |---|---|---|---|---|
    | T-L-Le, Best targets | -118 | -90 | -100 | -109 |
    | with --route-before-train | -119 | -138 | -97 | -117 |
    | and --error-target gap | -65 | -61 | -47 | -29 |
    | and --split-bootstrap 1000000 | -54 | -28 | -42 | -19 |

  - Readings, redone:
    - Routing before training on its own is level with Best (-48 to 3).
    - Gap targets are 50-88 better than route-before-train alone and 29-80 better than Best.
    - The bootstrap adds 5-33 more.
  - Every variant is below flat in every window. Gap and bootstrap close in over the run (-54 to -19), not yet level.
  - Current against its own champion, cycles 201-400 (champtab, which was right): F-L -60 (last crowned at 167), Best
    -76, gap and bootstrap -17 (last crowned at 364).
  - The probit twin (corrected, to cycle 555):
    - every variant is 100-140 below F-P from cycle 300;
    - gap targets were 60-80 worse than Best to cycle 200 and level with it after.
- **2026-11-10 13:14, error-targets and error-targets-probit launched.** The design is in section 7.
  - Their lines 2-5 are byte-identical until the first split, so their early spread is a free noise measure.
  - With gap targets the router error that drives splits is on a different scale from Best's. With a single split this
    can't confound the comparison.
- **2026-11-10 ~02:05, T1 offline mechanism check** (bin/chessai-target; fs/target; bddstudy setup, seed 1, threshold
  100k, limit 10, 21 leaves by round 40).
  - Gap stops LR routers routing by who's winning: U fell from 0.14-0.16 under Best to 0.07-0.11.
  - Less lock-in. Rounds 40/60:
    - mean largest share at a delegator: Best 74-75%, gap 53-58%, gap with bootstrap 44-49%;
    - effective leaves of 21: Best 8.5-9.9, gap 12.4-12.6, gap with bootstrap 13.9-17.5;
    - delegators starving a child: Best 2-4, gap with bootstrap 0.
  - Held-out loss: nothing beats Best.
    - Gap costs 0.002-0.015, and the bootstrap more. --route-before-train wins back 0.005-0.015.
    - All three together are level at round 40 (0.0002 [-0.0057, 0.0061]) and slightly behind at 60 (0.0049
      [-0.0001, 0.0100]).
  - Reading: spreading the data is the cost at this scale (2.4M positions over 21 leaves, far below ~1M per leaf).
- **2026-11-10 ~01:10, who's winning** (fs/analysis-2026-11-10/leafoutcome/who_wins_share.py). U is the share of a
  tree's routing entropy explained by a flat LR's who's-winning decile.
  - LR error routers: 0.31 (single-split T-L-Le), 0.25, 0.14 and 0.53 (old-league Le-fresh, Le-share3, LLs). LR
    Balanced: 0.05.
  - NB error routers: 0.02 (single-split T-L-Ne) and 0.04-0.12. NB cluster routers: 0.03 and 0.10-0.11.
  - Outcome per leaf alone is no test: label-free trees show it too, because positions carry the outcome.
- **2026-11-08 ~16:00-16:30, balanced targets** (bin/chessai-bal; fs/bal).
  - Continuing T-L-Le-c v1538 for 16 rounds: Best put 100% on one leaf every round. Balanced spread to 1.8-3.0
    effective leaves and predicted held-out self-play better (-0.0029 [-0.0049, -0.0010]).
  - NB routers barely move under it (-0.0005), since their counts hold their whole past.
  - In the champion run T-L-Le-bal-c went fully to one leaf for about 3 rounds after each own step, then spread again.
- **2026-11-07/08, router-target and assignment studies** (archive IDEAS 11-07/08).
  - errwhy: error routers predict better but play worse at ply 1; the loss is at seams. An LR router regressing each
    child's expected loss on frozen children recovers 51% (T-L-Ne-c, 0.013 of 0.026) and 20% (T-L-Le-c) of the
    reachable gain. Even perfect routing would only bring T-L-Ne-c to flat LR: the leaves bind.
  - Blend at play time (leaves mixed by the routers' probabilities; errwhy E, against the same tree on cal-own): 58
    (T-L-Le-c) and 45 (T-L-Le-fresh-c) at ply 1, 41 and 1 at ply 2, at 4-5× CPU a game. Not adopted (the user, 11-05:
    blending "defeats the purpose of the delegation"; LEDGER "Blending leaves").
  - eloss: the expected-loss target for training is harmful (T-L-Le 0.0235 worse). Specialisation feeds itself; root
    regret falls to 0 while leaf disagreement rises from 0.50 to 1.22.
  - playrouter: online eloss play routers are harmful (0.0034-0.0071 worse; arenas -30/-32 at ply 1, -67/-26 at
    ply 2).
  - assign: sending a random share E of examples to a uniform child gains 0.0017 / 0.0012 / 0.0027 offline at E 0.05 /
    0.1 / 0.2 (about 40% of sharing 0.3's gain), adds nothing on top of sharing, and is level in play (E 0.2: 9 at
    ply 1, -17 [-41, 7] at ply 2). It halves leaf disagreement (0.80 to 0.42) without a stronger player.
- **2026-11-05, soft targets and switch routers** (LEDGER "Trees: routing").
  - Soft targets with LR leaves were -23 against Best: they route by who is winning (seams 0.57 against 0.37 nats).
  - Switch routers were closed: NB leaves -43 against cluster, LR leaves -10.

---

## 6. Self-play depth, data sources and league data

Where it stands:
- **Models get good at the ply they train at** (11-10 ~18:30, entries below; the ply gain's outliers: ~19:05).
  - In selfplay-depth, played at ply 1, the d1-trained models beat the d2-trained ones by 238-321 (LR and probit).
    Played at ply 2, the order flips: d2 is ahead by 146-192.
  - The lr-settings rerun's field is ply 2 only. Its champions beat the original's by 424 at ply 2 but only 36 at
    ply 1 (section 8).
  - So "deeper data is better" holds for ply-2 play only. For ply-1 play, shallow data is better.
- **Probit trees' gains show mainly at ply 2** (11-10 ~22:40, split-growth-probit's final, section 4). Its two
  limit-1 probit trees were level with F-P at ply 1 (-5 to 6 late) and 20-51 ahead at ply 2; single-split-probit's
  T-P-Ne was 0 and 48, and its cluster trees 15-26 and 65-66. A ply-1 screen (the ply ladder, NEW-DESIGN) would have
  dropped three of these five, so probit trees need the ply-2 test whatever their ply-1 result.
- **What the depths are** (rs/src/player.rs, search.rs):
  - Depth 0 scores the position after each move.
  - Depth 1 does the same through the search, which also scores game-ending moves exactly (mate, stalemate,
    repetition, 50 moves). So depth 0 and depth 1 look equally far ahead, and depth 1 costs ~1.9× per ply for LR.
  - A tournament's ply k is the same `depth` field as self-play's depth k.
- **At matched training positions** (the matched-depth arena, played at ply 2), the order is d4, d1, d2, d0 for both
  LR and probit, with d4 200-700 above the others.
- **At equal priced self-play CPU** (selfplay-depth): at ply 2, d2 is best for LR and probit; at ply 1, d1 and d0 are
  best. NB is best at d0 at both plies; at d1-4 it collapsed.
- **League games are worth far more per position than self-play, at ply 2.** About 300 Elo came from 10% of the
  positions, mostly from the ply-2 entry's games. At ply 1, leaf-sources' d0 self-play lines beat its league lines
  (by 38-138). In the old league slow lineages were mostly league-trained, which flattered them.
- **Open:** why d1 is below d0 for LR at equal CPU, at ply 2. Part of the answer is likely cost: d1 looks no further
  than d0, and at ~1.9× the price it gets about half the games.

Entries:
- **2026-11-10 ~20:05, the champion run at both plies, in direct games** (fs/analysis-2026-11-10/champply, README.txt
  there). The user asked whether, apart from the training ply, some models that do well at ply 1 gain much less from
  the second ply.
  - Final fit, each lineage's versions still playing at the end (71 lineages): ply-1 against ply-2 Elo, correlation
    0.75, rank correlation 0.48, slope 0.50. Over each lineage's latest third (87 lineages): 0.81, 0.66 and 0.50.
  - The three flat probit lineages were 1st, 3rd and 4th at ply 1, and 52nd-59th at ply 2.
  - Direct games against F-L-c in cycles 46-60 and 61-75 (150 games a cell at ply 1, 600 at ply 2):
    - F-P-tau0-c, the same setup as F-L-c (its own league games, result labels, depth 0, own calibration): 12 and 14
      (±37) at ply 1; -163 and -166 (±27) at ply 2.
    - 26 LR trees with NB or TAN routers, pooled: -39 and -22 (±7) at ply 1; -39 and -13 (±5) at ply 2.
  - Reading: at the same training setup the leaf type changed the ply gain by about 170. The routers changed it by
    0-9 here; in the batch 2-3 runs by -88 to 36.
- **2026-11-10 ~19:05, the ply gain: what the second ply is worth, and its outliers** (the user asked for outliers in
  "the elo boost of having that additional ply" across every league that played both plies;
  fs/analysis-2026-11-10/plygain, README.txt there). Gain = a version's Elo at ply 2 minus its Elo at ply 1, in the
  same fit.
  - **The old leagues** (final fits, ~1,250 and ~4,970 versions). Median gain 606 (first run) and 735 (champion run).
    By the self-play depth in the lineage's name, league-trained lineages, champion run (first run in brackets):

    | Data | Leaf | Versions | ply 1 | ply 2 | Gain |
    |---|---|---|---|---|---|
    | d0 | LR and others | 2,855 | 974 | 1709 | 748 (599) |
    | d0 | probit | 112 | 1067 | 1624 | 555 (522) |
    | d0 | NB | 941 | 757 | 1404 | 664 (615) |
    | d2 | LR and others | 534 | 830 | 1679 | 871 (862) |
    | d2 | probit | 75 | 894 | 1814 | 918 (868) |
    | d2 | NB | 225 | 768 | 1490 | 720 (551) |
    | d4 | LR | 121 | 902 | 1758 | 876 (786) |

    - Lineages furthest from the gain their ply-1 Elo predicts:
      - highest: F-L-d2jit, d2mix, d2play and d4mix, F-P-tau0-d2jit, MoE-d2jit and T-L-NLs-merge-d2jit (190-370
        above);
      - lowest: F-P-c, F-P-tau0-solo, F-P-tau0, F-L-solo, FM, TAN and the NB-heavy M-N4-N and T-N-NNs (100-230
        below).
  - **selfplay-depth** (second half, self-play only; gain by training depth 0 / 1 / 2 / 4):
    - F-L: 574 / 418 / 937-972 / 606;
    - F-P: 471 / 500 / 972 / 625;
    - F-N: 577 / 242 / 213 / 242.
  - **Batch 1-3 experiments** (second half; full list in newgain.out):
    - Flat lines: LR 744-954 and probit 666-922, depending on the run.
    - Self-play d0 lines: F-L 593-627, F-P 465-507.
    - NB:
      - in NB-only fields (nb-settings): 155-171;
      - league-trained NB in mixed fields: 375-391;
      - self-play d0 NB: 628-679.
    - lr-settings' search-label line: 945, against 788-834 for the rate lines.
    - Probit tau: no effect (671-687).
  - **Trees against their flat line** (gain difference, same run):
    - Error trees with LR routers lose gain in every run: T-L-Le 39-88 below F-L, and T-P-Le 39-62 below F-P.
    - Probit trees with NB or TAN routers gain more than F-P: 26-36 more in single-split-probit and
      split-growth-probit, and 0-13 more in cluster-error.
    - LR cluster trees depend on the run: 30 more than F-L in single-split, but 10-71 less in batch 3 and 29-36 less
      in cluster-error.
  - **The lr-settings pair arenas** (approximate: ply 1 and ply 2 are separate arenas, linked only through the random
    players): the rerun's champions gain 703-781, the originals' 285-436.
  - Readings:
    - The second ply is worth most to models trained on depth-2 games or labels, and least to models trained on
      depth-1 games. This is the specialisation of section 1 seen as a gain.
    - Probit is the extreme case. At d0 it is the best ply-1 player and gains least; at d2 it is the best ply-2
      player and gains most.
    - NB barely gains from search unless it trained on its own d0 games.
    - Routing: LR error routers cut the gain in every run. NB and TAN routers raise it with probit leaves. With LR
      leaves it depends on the run. The LR trees' ply-2 deficit (section 1) is exactly a smaller gain.
- **2026-11-10 ~18:30, the depth of the training games against the depth of play** (fs/analysis-2026-11-10/plyfid;
  section 1 has the whole comparison).
  - selfplay-depth, cycles 78-100. Direct Elo against F-L d2 #1, with each pair of current versions playing at the
    same ply, and the draw rates at ply 1 and ply 2:

    | Line | at ply 1 | at ply 2 | draws ply 1 / 2 |
    |---|---|---|---|
    | F-L d0 | 279 | -99 | 0.27 / 0.04 |
    | F-L d1 | 310 | -166 | 0.27 / 0.04 |
    | F-L d2 #2 | -6 | 26 | 0.71 / 0.14 |
    | F-L d4 | 55 | -191 | 0.57 / 0.05 |
    | F-P d0 | 183 | -189 | 0.35 / 0.05 |
    | F-P d1 | 354 | -56 | 0.20 / 0.07 |
    | F-P d2 | 33 | 90 | 0.70 / 0.09 |
    | F-P d4 | -23 | -330 | 0.67 / 0.04 |
    | F-N d0 | 21 | -218 | 0.60 / 0.04 |
    | F-N d1 | -86 | -601 | 0.56 / 0.01 |
    | F-N d2 | -374 | -944 | 0.17 / 0.00 |
    | F-N d4 | -307 | -1065 | 0.22 / 0.00 |

  - Reading: each depth of data makes the best player at its own depth: d1 (and d0) at ply 1, d2 at ply 2. Models
    trained on d2 data draw 70-78% of their ply-1 games with each other.
  - A likely mechanism: the evaluator learns what its search can't see. A ply-1 player must spot one-move threats
    statically. A ply-2 player's search sees them, so its evaluator can spend its weights elsewhere.
  - leaf-sources' self-play lines were d0 (no --depth). They beat the league-trained lines at ply 1 by 38-138 and
    lost by 191-324 at ply 2.
- **2026-11-10, matched-depth arena:** section 8.
- **2026-11-10 06:52, selfplay-depth final** (exp selfplay-depth: F-L, F-P and F-N at self-play depth 0, 1, 2 and 4,
  with a second F-L d2; self-play only, 60 priced s a line a cycle, 100 cycles).
  - Mean final ply-2 rating of v81-99:

    | Leaf | depth 0 | depth 1 | depth 2 | depth 4 |
    |---|---|---|---|---|
    | F-L | 1529 | 1459 | 1595 / 1616 (twins) | 1397 (still climbing) |
    | F-P | 1416 | 1565 | 1696 (still climbing) | 1224 (still climbing) |
    | F-N | 1260 | 847 (unstable) | 458 | 539 |

  - Depths 0 and 1 plateaued by v21-40. Depth 2 was still climbing at the end (probit 78 above its v21-40 bin).
  - Superseded early reading (cycle 6, 11-09): NB d0 1,176 against d1 696; LR d0 1,427 against d1 1,426; probit d0
    1,330 against d1 1,510.
  - Depth 4 climbs steeply but is starved of games.
  - At cycle 67 (current@2 minus F-L d2 #1, cycles 61-67):
    - F-L: d0 -88, d1 -145, d2 0 / 20, d4 -317;
    - F-P: -186, -26, 49, -447;
    - F-N: -354, -1054, -1135, -1096.
  - At cycle 85 (cycles 81-85) the order was unchanged: LR d0 -126, d1 -172, d2 twin 9, d4 -249; probit d0 -207,
    d1 -77, d2 20, d4 -450; NB d0 -369, d1-4 -855 to -1185.
  - Self-play totals by cycle 67:
    - d0: ~4M games, 88-94% draws;
    - d1: ~3M games, 57-72% draws;
    - d2: 0.2-0.3M games, 25-28% draws (NB 67%);
    - d4: 2-7 thousand games.
  - Depth 2 won for LR and probit despite 13-18× fewer games than depth 0.
  - NB at depths 1-4 collapsed to near random (the random references sat at -1278 to -1587 on this scale). F-N d1's
    champion was v11 and its current version 478 below it; F-N d2 never improved on v1. F-N d1's ratings by
    20-version bin were 838, 568, 424, 654 and 847. Its games are lopsided by colour (White 28%, Black 16%, draws 57%;
    LR at depth 2 35/37/28). The cause is unknown.
  - The depth semantics are in rs/src/player.rs, search.rs and main.rs:
    - depth 0 scores children by the model;
    - depth 1 also values game-ending moves exactly;
    - depth 2 also avoids allowing a mate in one;
    - the default policy is WldScoreJitter10.
- **2026-11-09 02:45-05:17, Q3: which league games carry the value** (fs/twin/league; over the self-play-only twin,
  per 100k positions in brackets):

  | League subset | v122 | v217 | v452 |
  |---|---|---|---|
  | all | 221 (11.9) | 212 (5.3) | 302 (2.6) |
  | the d2 entry's games | 190 (20.4) | 202 (10.1) | 307 (5.4) |
  | the d1 entry's games | 98 (10.6) | 78 (3.9) | 85 (1.5) |
  | games against stronger opponents | 128 (17.7) | 133 (11.2) | 219 (8.0) |
  | games against weaker opponents | 106 (9.4) | 166 (5.9) | 261 (3.0) |

  - By v452 the ply-2 entry's games alone carry all of it. Per position, d2-entry games are worth 3.6× d1-entry
    games, and games against stronger opponents 2.7× those against weaker ones.
- **2026-11-09 ~04:50, league-data share per lineage, cycle 51** (models/league-share-c51.txt). The share of a
  cycle's training positions that came from league games:
  - d4 self-play 99%, d2 self-play 65-91%;
  - trees, MoEs and DAGs at d0 8-26%;
  - d1 flat 10%, d0 flat 4.6-6.2%, solo 0%.
  - Own step yes or no, both from scratch (F-L-new-c minus F-L-solo-c): 67, 199, 174 and 168 at ages 1-4.
  - Without league data, d2 self-play was far behind d0 at equal CPU: 825 and 909 against 1330 and 1361 at age 3 (solo
    LR and probit).
- **2026-11-08 23:50, the league-data confound** (the user: "the d2 models use more league data proportionally").
  - d2jit lineages trained 46-85% on league games, F-L-c ~3%.
  - So "depth-2 training helps" was confounded with league data. That is why the solo lineages were added.
- **2026-11-08, spdrop** (fs/spdrop): the post-self-play drop is real, at ply 2 only, and kind-specific.
  - New-league cycles 2-4, current version against its champion: FM -147, T-L-Le -105, MoE -91, flat LR -21.
  - Depth-0 self-play moves skill from ply-2 to ply-1 positions, all of it in the first round, and the own step moves
    it back. On T-L-Le-fresh-c the league ply-2 winner loss went 0.426 to 0.450 and ply 1 0.343 to 0.307. Waking from
    dormancy cost -148 at ply 2 for MoE, FM, T-L-Le and T-L-LLs, and -50 for the rest.
  - Fix (b), the own step first on last cycle's games, recovers it (119 against 124). Fix (a) was right on average
    (152 against 134) but off by up to 100 for one lineage.
  - Swing diagnostic (E minus O on the league sample): FM 0.041, T-L-Le 0.026, MoE 0.019, flat LR 0.009.
  - The old league's gates played the arena's default policy (WldScoreJitter10), not the league's softmax.
- **2026-11-07, depth-train** (fs/depth-train): look-ahead training by classifier.
  - Compute-matched league pairs (1-ply against 2-ply self-play; the 2-ply side's Elo):

    | Pair | CPU-s | ply 1 | ply 2 |
    |---|---|---|---|
    | LR jitter d1 → d2 | 1000 / 3000 | -137 / -65 | -253 / 217 |
    | LR jitter d0 → d2 | 5600 | -91 | 143 |
    | LR softmax d1 → d2 | 1000 / 4000 | -100 / -129 | -103 / 138 |
    | MoE d0 → d2 | 1000 / 3200 | -25 / -16 | 14 / 151 |
    | NB d0 → d2 | 1000 / 3200 | -19 / -68 | -128 / -151 |
    | NB tree d0 → d2 | 1000 / 3300 | -21 / -89 | -139 / -164 |

  - Equal data, no self-play (ply-1-trained / ply-2-trained):
    - winner log-loss: LR 0.4066 / 0.3535, probit 0.4120 / 0.3579, NB 0.4150 / 0.4583;
    - arenas at ply 1 / ply 2: LR -34 / 201, probit -45 / 174, NB -53 / -203.
  - So NB fails at learning from look-ahead games, while LR and probit gain about the same.
  - Pure self-play from scratch (depth-train/pure; depth 1 against depth 2 at equal CPU; the depth-2 side's ply 1 /
    ply 2 Elo at 450, 900 and 1800 CPU-s): LR -198/-370, -142/-260, -108/-157 (closing); probit -189/-231,
    -135/-53, -119/-51 (closing fastest); NB -210/-459, -257/-599, -280/-597 (widening). The league's LR pair
    crossed over between 1000 and 3000 CPU-s.
  - Depth-2 self-play is 74-87% decisive for LR-based models and 37-40% for NB-based ones. NB's raw P(draw) is
    saturated (median 0.9997-0.99999; top-two move gap ~1e-6), so jitter decides most of its moves. Own-step gates:
    NB-d2 lineages gained 40-44 a cycle from league games and lost it again in self-play; LR-d2 gained 9 (kept 49%).
  - This study has no summary.txt; its numbers are in archive IDEAS "Look-ahead training by classifier" and
    fs/depth-train (arenas.txt, data/, pure/).
- **2026-11-07/08, blended self-play** (fs/blendsp; play blended by the routers' probabilities in self-play). Not
  worth its cost.
  - Equal data, pooled over five version pairs: 10 (-1 to 21) at ply 1 and 69 (45 to 93) at ply 2.
  - Equal CPU (2.2-2.3× a round): -35 (-47 to -24) at ply 1, -73 (-98 to -47) at ply 2.
  - Resumed from T-L-Le-c v326 at equal data: -20 (-32 to -8) at ply 1, -13 (-40 to 14) at ply 2.
- **2026-11-08, data per entry and search labels in the old league** (archive IDEAS "Coordinator notes").
  - The top entry then, F-L-d4mix-c v363, had trained on 7.6M positions, 93% of them own-step league games. The
    median ply-2 entry had 258M, F-L-c about 0.9B, the d2/d4 lineages 12-35M.
  - 50/50 look-ahead labels at depth 2 (F-L-d2mix-c against F-L-d2play-c, labels the only difference), cycle 27: 12 on
    current versions, 57 on champions (SE about 30). F-L-d2jit-c (result labels) was ahead of both. They are not the
    fix for the ply-2 drop (spdrop).
- **2026-11-05, search in games and labels** (LEDGER; archive README).
  - LR self-play with depth-2 search from scratch: v50 was 93 over the jitter plateau. It was deferred then for cost.
  - 50/50 search labels gave about 100 over result labels. They are outside the standard: no search in training.
  - In lr-settings they cost 12× the training CPU, unpriced (section 2).

---

## 7. League runs and experiment batches

Where it stands:
- The live league runs are over. The first run stopped after cycle 73 (11-07 22:30), and the champion run after
  cycle 75 (11-09 20:06).
- The new league system runs experiments only; the global league has not been launched. The design is in
  NEW-DESIGN.md, the engine in leagues/BUILD.md and leagues/FORMATS.md.
- **Lessons for experiment design** (rules in NEW-DESIGN.md):
  - one factor per comparison;
  - only the lines the user asks for;
  - the same field shape across experiments;
  - unpriced compute is a factor;
  - pool windows of versions;
  - check that compound models stay compound.

**New-league experiments** (`exp <name>`; seeds and lines from each config.json):

| Experiment | Batch | Lines | Seed | Cycles | Ran | Status |
|---|---|---|---|---|---|---|
| lr-settings | 1 | F-L at 4 rates + search labels (5) | 1 | 500 | 11-09 20:10-21:15 | stopped at 172 (the label line fed every rate line) |
| probit-settings | 1 | F-P tau 0 / 1e-6 / 1e-5 / 1e-4 (4) | 1 | 500 | 20:10-23:46 | done |
| nb-settings | 1 | F-N alpha 0.1 / 1 / 10 (3) | 1 | 500 | 20:10-21:15 | stopped at 265 (NB doesn't bootstrap) |
| nb-settings-no-random3 | 1 | as nb-settings, without random-3 (3) | 1 | 500 | 20:50-21:15 | stopped at 126 |
| selfplay-depth | 1 | F-L, F-P, F-N × depth 0/1/2/4, + F-L d2 #2 (13, self-play) | 1 | 100 | 20:10-06:52 | done |
| leaf-sources | 1 | F-L ×2, F-P, F-N × self-play and local (8) | 1 | 100 | 20:10-22:22 | done |
| lr-settings-2 | 1 | F-L --lr-rate 0.0125 / 0.025 / 0.05 / 0.1 (4) | 1 | 500 | 21:15-00:35 | done (section 2) |
| nb-settings-2 | 1 | F-N --alpha 0.1 / 1 / 10 / 100 (4) | 1 | 500 | 21:15-01:08 | done (section 2) |
| leaf-sources-2 | 1 | F-L, F-P, F-N × self-play and local (6) | 1 | 100 | 21:34-23:25 | done (section 2) |
| single-split | 2 | F-L; T-L-Le, -Ne, -Te; T-L-Lc-, -Nc-, -Tc-merge (7, local) | 1 | 1000 | 22:01-10:51 | done |
| single-split-probit | 2 | the same with probit leaves (7, local) | 1 | 1000 | 22:15-10:55 | done |
| cluster-error | 2 | F-L, F-P, 6 cluster-merge trees × (abs, log) (14, local) | 1 | 1000 | 11-09 22:26 to 11-10 16:25 | stopped by the user after cycle 637 |
| split-growth | 3 | F-L; T-L-Nc-merge and T-L-Nc at limits 1 and 4 (5, local) | 2 | 400 | 11-10 11:39-14:05 | done (section 4) |
| split-growth-probit | 3 | F-P; T-P-Nc-merge and T-P-Ne at limits 1 and 4 (5, local) | 3 | 1000 | 11-10 12:31-21:36 | done (section 4) |
| error-targets | 3 | F-L; T-L-Le (Best), then one flag added per line: route-before-train, gap targets, a split bootstrap (5, local) | 4 | 400 | 11-10 13:14-17:11 | done (section 5) |
| error-targets-probit | 3 | the same with probit leaves (5, local) | 5 | 1000 | 11-10 13:15-21:47 | done (section 5) |
| residual-leaves | 3 | F-L; T-L-Nc limit 4, copy-on-split against residual leaves with inherited sums (3, local, no merges) | 6 | 400 | 11-10 14:06-16:22 | done (section 4) |
| trigger-gradient | 3 | F-L; T-L-Nc limit 8, today's trigger against --split-gradient 9.5 (3, local) | 9 | 400 | 11-10 16:34-18:19 | done (section 4) |
| small-field | 3 | F-L, T-L-Tc-merge (single-split's args); references random and random-3 only; 56 rounds (2,464 local games) (2, local) | 10 | 400 | 11-10 16:34-19:39 | done (section 4) |
| lr-settings-new | 4 | as lr-settings (5) | 1 | plateau (cap 1000) | 11-10 17:41-18:41 | done: plateau stop at 194 (last crowning 144); pair arena in section 8 |
| probit-settings-new | 4 | as probit-settings (4) | 1 | plateau (cap 1000) | 11-10 18:41-19:36 | done: plateau stop at 256 (last crowning 206); pair arena in section 8 |
| nb-settings-new | 4 | as nb-settings (3); stands in for nb-settings-no-random3 too | 1 | plateau (cap 1000) | 11-10 19:36-19:46 | done: plateau stop at 100 (last crowning 39); pair arenas in section 8 |
| leaf-sources-new | 4 | as leaf-sources (8, self-play and local) | 1 | plateau (cap 1000) | 11-10 19:46-23:47 | done: plateau stop at 320; pair arena: rerun - original 36.7 [22.6, 50.9] at ply 2, -3.7 at ply 1 |
| lr-settings-2-new | 4 | as lr-settings-2 (4) | 1 | plateau (cap 1000) |  | queued |
| nb-settings-2-new | 4 | as nb-settings-2 (4) | 1 | plateau (cap 1000) |  | queued |
| leaf-sources-2-new | 4 | as leaf-sources-2 (6, self-play and local) | 1 | plateau (cap 1000) |  | queued |
| single-split-new | 4 | as single-split (7) | 1 | plateau (cap 1000) |  | queued |
| single-split-probit-new | 4 | as single-split-probit (7) | 1 | plateau (cap 1000) |  | queued |
| cluster-error-new | 4 | as cluster-error (14) | 1 | plateau (cap 1000) |  | queued |
| split-growth-new | 4 | as split-growth (5) | 2 | plateau (cap 1000) |  | queued |
| split-growth-probit-new | 4 | as split-growth-probit (5) | 3 | plateau (cap 1000) |  | queued |
| error-targets-new | 4 | as error-targets (5) | 4 | plateau (cap 1000) |  | queued |
| error-targets-probit-new | 4 | as error-targets-probit (5) | 5 | plateau (cap 1000) |  | queued |
| residual-leaves-new | 4 | as residual-leaves (3) | 6 | plateau (cap 1000) |  | queued |
| trigger-gradient-new | 4 | as trigger-gradient (3) | 9 | plateau (cap 1000) |  | queued |
| small-field-new | 4 | as small-field (2); no references at all | 10 | plateau (cap 1000) |  | queued |
| small-field-x4 | 4 | F-L ×4, T-L-Tc-merge ×4 (small-field's models as replicates; 8, local) | 11 | plateau (cap 1000) |  | parked 11-10 ~22:15 (no more ply-2 experiments for now) |
| small-field-x4-p1 | ply 1 | small-field-x4 at ply 1: F-L ×4, T-L-Tc-merge ×4 (8, local); entries, training and champion test at ply 1; self-games; no references; 10 rounds | 11 | plateau K = 25 (cap 1000) | 11-10 22:52-23:43 | done: plateau stop at 190 (section 4) |
| growth-limits-p1 | ply 1 | T-L-Nc-merge at split limits 1, 4, 8 (at most 3, 9, 17 leaves), 2 replicates each (6, local); ply 1, self-games, no references; 18 rounds | 12 | plateau K = 29 (cap 1000) | 11-10 22:52 to 11-11 00:01 | done: plateau stop at 340; the merge gate undid growth (section 4) |
| probit-widen-p1 | ply 1 | T-P-Nc-merge limit 4, threshold 1M, fresh merge gate; --split-noise 0 / 0.03 / 0.1, 2 replicates each (6, local); ply 1, self-games, no references; 12 rounds; min_cycles 200 | 14 | plateau K = 29 (cap 1000) | 11-11 00:11-00:18 | stopped at cycle 63 to add F-P (the user: "yeah restart it"); not a result |
| probit-widen-p1b | ply 1 | as probit-widen-p1 plus F-P x2 (8, local) | 14 | plateau K = 25 (cap 1000) | 11-11 00:18-00:54 | done at 200 (plateau); cold read: fresh-gate trees 80-104 over F-P at ply 1 (section 8) |
| probit-widen-p1c | ply 1 | probit-widen-p1b's replicate (seed 18 the only difference) | 18 | plateau K = 25 | 11-11 00:58-02:02 | done at 251 (plateau); cold read: fresh-gate trees ~95 over F-P at ply 1, replicating p1b; noise 0.03 ~40 at ply 2 in both (section 8) |
| lr-trees-p1 | ply 1 | F-L x2, T-L-Nc-merge limit 4 threshold 1M fresh gate x2 (local); p1b's conditions with LR leaves | 21 | plateau K = 35, min 200 | 11-11 01:00-01:32 | done at 302 (plateau); cold read: trees 43 ±21 over F-L at ply 1, -43 ±22 at ply 2 (section 8) |
| flat-p3 | ply 3 | F-L x4 (local), pure ply-3 field, self-games, 15 rounds, chessai-hybrid10 | 20 | plateau K = 35, min 200 | 11-11 01:34-02:36 | done at 200 (plateau; champions v101-161); interim cold read running |
| flat-p2 | ply 2 | flat-p3's twin at ply 2 (ply1_entries false); the field's ply the one factor | 20 | plateau K = 35, min 200 | 11-11 02:02-02:37 | done at 245 (plateau; champions v209, v196, v205, v209); read with the -b runs |
| flat-p3-b | ply 3 | flat-p3's seed replicate | 23 | plateau K = 35, min 200 | 11-11 02:12-03:18 | done at 200 (plateau; champions v141, v137, v139, v151; 3.2 CPU-h) |
| flat-p2-b | ply 2 | flat-p2's seed replicate | 23 | plateau K = 35, min 200 | 11-11 02:37-03:20 | done at 335 (plateau; champions v254, v253, v299, v225; 1.1 CPU-h) |
| flat-p3-w4 | ply 4 | F-L x4 warm-started from flat-p3's finals v199 (AdaGrad sums / 8 at the start), pure ply-4 field, self-games, 6 rounds; chessai-hybrid10 to cycle 21, hybrid11 (exact) from 22 | 24 | plateau scaled, min 60, cap 100 | 11-11 03:32-04:43 | done at the cap (100; champions v87, v49, v87, v83; 4.0 CPU-h) |
| flat-p3-w3 | ply 3 | flat-p3-w4's twin at ply 3 (same starts, cycles, rounds); the field's ply the one factor | 24 | plateau scaled, min 60, cap 100 | 11-11 03:32-03:48 | done at 79 (plateau; champions v43, v13, v0, v1; 0.6 CPU-h) |
| flat-p2-w3 | ply 3 | F-L x4 warm-started from flat-p2's ply-2 finals v244 (sums / 8), ply-3 field, W3's settings; tests the warm-started ladder | 24 | plateau scaled, min 60, cap 100 | 11-11 03:55-04:09 | done at the cap (100; champions v76, v76, v84, v82; 0.7 CPU-h) |
| flat-p3-w3f1 | ply 3 | flat-p3-w3 exactly but start_forget 1 (sums kept): is the reheated step W3's 27-51 loss? | 24 | plateau scaled, min 60, cap 100 | 11-11 04:59-05:12 | done at the cap (100; champions v63, v17, v70 and v0: one line never beat its start) |
| flat-p3-w4-w3f1 | ply 3 | the reverse rung (theory-2 6.21): flat-p3-w3f1 exactly but started from W4's finals (flat-p3-w4 v99); does ply-3 training overwrite W4's even-ply gain? | 24 | plateau scaled, min 60, cap 100 | 11-11 06:51-07:03 | done at 97 (plateau; champions v60, v45, v61, v56; 0.49 CPU-h); read cold at plies 1-4: minus W3f1 4, 45, -2, 19 (W4 minus W3f1 -96, 131, -87, 131); W4's even-ply gain mostly undone (section 8) |
| train-t002-p1 | ply 1 | F-L x2, pure ply-1 field, own games (tournament and self-games) at T 0.02; twin of train-t01-p1 (theory-2 6.23 item 1) | 27 | plateau scaled, min 200, cap 600 | 11-11 08:34-08:40 | done at 211 (plateau; champions v160, v103; 0.10 CPU-h); read: 106 and 221 BELOW its T 0.1 twin at plies 1 and 2 (section 8) |
| train-t01-p1 | ply 1 | train-t002-p1's twin with its own games at the default T 0.1 (the temperature the one factor) | 27 | plateau scaled, min 200, cap 600 | 11-11 08:34-08:40 | done at 202 (plateau; champions v151, v151; 0.10 CPU-h) |
| probit-limit8-p1 | ply 1 | T-P-Nc-merge fresh gate, split noise 0.03, threshold 1M: split limit 8 x2 against limit 4 x2 (theory-2 6.23 item 2) | 26 | plateau scaled, min 200, cap 600 | 11-11 08:34-09:10 | done at 415 (plateau; 0.74 CPU-h). The limit never bound: each limit-8 line has its limit-4 twin's leaf count at every version checked (v100/200/300/414: 3, 1, 1, 3 and 3, 7, 3, 3; `leaves`). The fresh gate's 1M window undoes splits first. G4 (growth-300k-p1) tests a shorter window |
| router-kind-p1 | ply 1 | G1 (theory-2 6.24): F-P x2; fresh-gate probit trees limit 4, threshold 1M, noise 0.03 with an LR cluster router (T-P-Lc-merge) x2 against the NB router (T-P-Nc-merge) x2 | 28 | plateau scaled, min 300, cap 600 | 11-11 08:47-09:33 | done at 300 (plateau; champions F-P v183, v244, LR-router v177, v265, NB-router v268, v203; 0.94 CPU-h); cold read below |
| growth-300k-p1 | ply 1 | G4 (theory-2 6.24): F-P x2; fresh-gate probit trees (NB router, noise 0.03) limit 8, split threshold 300k x2 against 1M x2 | 29 | plateau scaled, min 300, cap 600 | 11-11 08:47-09:46 | done at 341 (plateau; ~1.0 CPU-h). Leaves at v340: 1M 5 and 7, 300k 1 and 3. The 1M trees grew (3-7 early, 5-11 late); the 300k trees split early (5-9 at v25-50), then the gate kept merging them (mostly 1 leaf from v100). Cold read: diag/growth-300k-p1-cold |
| error-routing-p1 | ply 1 | G2 (theory-2 6.24): F-P x2; probit error trees T-P-Ne limit 4, threshold 1M, noise 0.03, non-merging (the user's 11-08 rule) x2; seat 1's cluster trees (T-P-Nc-merge, fresh gate) x2 | 30 | plateau scaled, min 300, cap 600 | 11-11 09:10-10:04 | done at 315 (plateau; ~1.05 CPU-h). All four trees hold 9 leaves (limit 4 reached). Cold read: error trees 118 over F-P at ply 1, 77 at ply 2; cluster trees 106 / 62; level with each other (section 8) |
| lr-share-p1 | ply 1 | G6 (theory-2 6.24): F-L x2; LR error trees with leaf sharing 0.3 (T-L-Ne-share3, non-merging) limit 4, threshold 1M x2; fresh-gate LR trees (T-L-Nc-merge limit 4, threshold 1M; seat 2's second seed) x2 | 31 | plateau scaled, min 300, cap 600 | 11-11 09:33-10:41 | done at 343 (plateau; ~1.3 CPU-h). All four trees 9 leaves. Cold read: LR cluster trees 19 over F-L at ply 1, -17 at ply 2; shared-leaf error trees 22 / -34 (section 8) |
| dag-lr-p1 | ply 1 | G3 (theory-2 6.24, the analyst's form (a)): F-L x2; the cell DAG D-L-Nc (chessai-dag-q9: LR leaves, a cluster router over 9 cells pointing to 5 shared leaves, cells re-pointed by fresh scoring, dag-z 2, window 50k, first check at 1M; no --revise, which the cell DAG excludes) x2 against fresh-gate LR trees T-L-Nc-merge limit 4, 1M (a third seed for seat 2) x2. The DAG code has no fresh gate (BDD) or probit leaves (BDD), so this is the only DAG runnable without new code; its size is fixed by --dag-leaves. hybrid13 plays dag-q9 DAGs byte-identically (8 games each at depths 1-2). Training: dag-q9 for the DAG, hybrid6 for the rest (train_binary.py --from chessai-hybrid4gfx) | 33 | plateau scaled, min 300, cap 600 | 11-11 09:48-11:19 | done at 426 (plateau); cold read v425 (T 0.02, 75 RR): cell DAG minus F-L 30 ±23 at ply 1, 15 ±22 at ply 2; trees (seat 2) 59 / 16; cell DAG minus trees -30 / -1; the DAGs use 3 of 5 leaves (section 8) |
| error-dag-probit-p1 | ply 1 | The user's error-DAG test on the six's error tree (11-11 ~10:10: error trees become DAGs with path parameters, non-merging, no leaf sharing, router-split threshold calibrated; "they should at least be equal"): F-P x2; T-P-Ne limit 4, 1M, noise 0.03 x2 (the tree = the DAG at z inf); D-P-Ne-ctx (chessai-errdag3: probit leaves, NB error routers, path cells, leaf limit 4 plus router limit 4) at router-split z 0 x2 (busy) and z 30 x2 (the calibrated arm: offline z 5-10, the best band, mapped to 1M windows). The analyst's prediction (theory-2 6.29): both DAG arms within 20 of T-P-Ne at plies 1 and 2, no seat on Elo; z 30 0 to 10 above the tree, z 0 -15 to 5; the structural reads matter more (the copies' overrule rate; mean |cell| at the end, near 0.03 = the cells never learned); 25 or more above the tree at both plies would be the first router-split gain in play. Replaces the deferred LR stand-in error-dag-p1 in G3's slot | 36 | plateau scaled (K 25 for 8 lines), min 300, cap 600 | 11-11 11:20-15:02 | done (367; last crowning 342), read (section 8): z 30 = tree, z 0 -41 at ply 2 |
| error-dag-p1 | ply 1 | The LR error-DAG test (the user, 11-11 ~12:10: "have we tried the LR error dag as well?"; the six to include error DAGs): F-L x2; T-L-Ne limit 4, 1M x2 (the tree = the DAG at z inf); D-L-Ne-ctx (chessai-errdag2: LR leaves, NB error routers, path cells, leaf limit 4 plus router limit 4) at router-split z 0 x2 and z 160 x2 (offline: no z beat the tree). Deferred 11:20, launched in error-growth-p1's slot. Read cold at plies 1-2 with path-stats on the DAG finals (leagues/specs/arenas/dag_read.sh) | 34 | plateau scaled (K 25 for 8 lines), min 300, cap 600 | 11-11 12:12-15:50 | done (362), read (section 8): z 0 and z 160 = tree at plies 1-2 |
| error-growth-p1 | ply 1 | Growth of the flagship error tree: F-P x2; T-P-Ne (NB error router, non-merging, 1M, noise 0.03) split limit 4 x2 against 8 x2. G2's T-P-Ne reached its limit (9 leaves) by cycle 50-75 and held it. The analyst's prediction (theory-2 6.28, revised after the error-tree routecells check): limit 8 minus limit 4 between -20 and 20 at ply 1 by cycle 300, centred near 0, a late gain of 10-30 plausible past 400 cycles; a loss beyond 30 = the third level fragments faster than it learns. Read beside Elo: routecells D_jk of the third-level leaves (decides whether limit 16 is worth anything) | 37 | plateau scaled, min 450 (raised from 300 at cycle 28, 10:46: 6.28's late-gain window), cap 600 | 11-11 10:43-12:11 | done at 450; cold read v449 (T 0.02, 75 RR): limit 4 minus F-P 84 / 64 (plies 1 / 2), limit 8 (17 leaves) 59 / 67, limit 4 minus 8 25 ±22 / -3: growth adds nothing (section 8) |
| probit-confirm-p2 | ply 2 | The ply-2 confirmation of G2's winners (Batch 5 item 3, updated): F-P x2; T-P-Ne limit 4, 1M, noise 0.03, non-merging x2 (holds its splits by construction: the user's NB-router-at-ply-2 question); T-P-Nc-merge limit 4, 1M, noise 0.03, fresh gate at --revise 2 x2; ply-2-only field, hybrid13 games, hybrid6 training. The analyst's prediction (theory-2 6.27): T-P-Ne minus F-P 50-90 at ply 2, 40-80 at ply 1, 20-60 at ply 3, 9 leaves held; the --revise 2 cluster tree 5-9 leaves and 10-30 below the error tree, or near 0 if the gate still merges it to one leaf | 35 | plateau scaled, min 200, cap 600 | 11-11 10:08-11:19 | done at 242 (plateau); cold read v241 (T 0.02, 75 RR, plies 1-3): error trees minus F-P -17 / 18 / -49, cluster trees -3 / -9 / -65, all 9 leaves held; 6.27 missed: no tree gain when trained at ply 2 (section 8) |
| probit-late-p2 | ply 2 | Ply-2 late-gain test (theory-2 6.31's next step, lean): F-P x2; T-P-Ne (NB error router, non-merging, limit 4, 1M, noise 0.03) x2, in a ply-2 field. One factor against probit-confirm-p2: the plateau floor 400 (was 200), so the run passes G2's 64M trained positions; G2's trees gained 78 at ply 1 between v226 and v314 while F-P stood still (diag/g2-versions-cold). Question: does the ply-2-field error tree pull ahead of its flat late? Read cold at plies 1-3 with v241 beside the finals. The analyst's prediction (theory-2 6.32): T-P-Ne minus F-P at the finals 15 to 45 at ply 2, 0 to 30 at ply 1, -60 to -20 at ply 3; T-P-Ne final minus v241 10 to 40 at ply 2, mostly after cycle 300. Decision rule for the user: a final ply-2 gain of 40 or more keeps the ply-1 screen (with longer ply-2 confirmations); under 20, screen at ply 2 and turn to features | 38 | plateau scaled (K 35 for 4 lines), min 400, cap 600 | 11-11 11:53-12:48 | done (400; last crowning 360), read (section 8): tree minus F-P -3 ±18 at ply 2 |
| probit-trees-p2 | ply 2 | F-P x2, T-P-Nc-merge limit 4 fresh gate noise 0.03 x2 (local), ply-2-only field | 18 | plateau K = 35, min 200 | 11-11 01:00-01:40 | done at 277 (plateau); cold read: trees level with F-P at plies 1-3 (-10, -8, 1; section 8) |
| selfplay-depth-new | 4 | as selfplay-depth (13, self-play) | 1 | plateau (cap 1000) |  | skipped by the user (d4 would take too long to plateau) |
| global (league) | ply 1 | The user's global league, 11-10 design, the eight of 11-11 ~16:20 untrained at ply 1: F-L, F-P, T-P-Nc-merge-fresh, T-L-Nc-merge-fresh, D-P-Ne-ctx z 30 and z 20, D-L-Ne-ctx z 160 and z 120; current + champion entries, 4 random references, 5 double round-robins a cycle, Elo-margin training (30), self-games, patience 20M positions (5-cycle floor). hybrid15 games and DAG training, hybrid6 for the rest (leagues/specs/global/eight-p1.json) | 41 | open-ended | 11-11 16:40- | running |
| base-probit-p2 | ply 2 | Ply-2 stage 1 (the user, 11-11 ~16:30: ply 2 "nearly from scratch", one-factor exploratory experiments): F-P x2; T-P-Nc-merge (fresh gate at --revise 2, limit 4, 1M, noise 0.03) x2; D-P-Ne-ctx (path context, leaf limit 4 plus router limit 4, 1M, noise 0.03) at z 20 x2. Ply-2-only field, self-games; hybrid15 games and DAG training, hybrid6 for the rest. Reads at v300, v600, final | 39 | plateau scaled, min 600, cap 900 | 11-11 17:03-21:45 | DONE at v599 (floor 600). Read leagues/arenas/diag/base-probit-p2-stage (v300 and v599, plies 1-3, T 0.02) plus a ply-2 ladder leagues/arenas/diag/base-probit-p2-ladder-p2 (v300, v450, v599); verdicts head to head (600 games, CI about ±28; ply 1 / 2 / 3). Tree (T-P-Nc-merge fresh) vs F-P: -1 / -40 / -39 at v300, -14 / -2 / -53 at v599 (ladder at ply 2: -39, -51, -23 at v300, v450, v599). DAG (D-P-Ne-ctx z20) vs F-P: -15 / -15 / 6 at v300, -17 / -11 / -54 at v599 (ladder: -18, -21, -8). DAG vs tree: -13 / 9 / 7 at v300, -9 / 16 / 10 at v599 (ladder: 27, 1, 9). v599 over v300: F-P 8 / 46 / 41 (ladder 67; v450 between), tree 6 / 14 / 20 (ladder 33, v599 -7 vs v450), DAG -1 / 23 / 16 (ladder 34). In a pure ply-2 probit field the flat is best at every ply, still improving at v599 (best version = the last); trees lose ~53 at ply 3. (base-lr-p2 for comparison: LR tree over F-L 27 / 20 / -26 at v599.) |
| base-lr-p2 | ply 2 | Ply-2 stage 1, LR: F-L x2; T-L-Nc-merge (fresh gate at --revise 2, limit 4, 1M) x2; D-L-Ne-ctx (path context, leaf limit 4 plus router limit 4, 1M) at z 120 x2. As base-probit-p2 otherwise | 40 | plateau scaled, min 600, cap 900 | 11-11 17:03- | DONE at v599 (20:29). Read leagues/arenas/diag/base-lr-p2-stage (v300 and v599, plies 1-3, T 0.02; verdict = pooled head-to-head, 600 games): tree over F-L 8 / 26 / -20 at v300 and 27 / 20 / -26 at v599 (plies 1/2/3); DAG over F-L 6 / -20 / -55 and 0 / -9 / -48; tree over DAG 23-26 at plies 1-2. F-L v599 against F-L v300: -7 / -51 / 57 (lost ply-2 strength and gained ply 3 as its field moved). No architecture beats F-L clearly at ply 2; both lose at ply 3 |
| stall-p1 | ply 1 | The user's stall split rule (11-11 ~19:10, "just put the experiment on the dashboard"): one factor, the leaf split rule, cap held at 4. F-P x2; T-P-Ne (NB error router, non-merging, limit 4, 1M, noise 0.03) with the current rule (worse, z 0) x2 and with --split-rule stall-high --split-z 2 x2. Ply 1 (the analyst: the tree gain exists there). hybrid15 games, hybrid6 training (leagues/specs/batch6/stall-p1.json) | 43 | plateau scaled, min 300, cap 600 | 11-11 19:14- | DONE at v392 (21:13). Read leagues/arenas/diag/stall-p1-stage (v150 and v392, plies 1-2, head-to-heads): stall-high against the current rule -8 / 16 at v150 and -16 / -23 at v392 (plies 1/2), all within noise; both over F-P 57-77 at v392. The stall rule spread the splits: 4th split at 27-35M positions (v133/v167) against 11.5M (v57/58) for the current rule; strength the same |
| grow-p1 | ply 1 | Growth rule phase 1 (growth-spec section 4; the user 11-11: splits should wait for stagnation and come at a steady rate per unit of data, tested at ply 1). One factor, the leaf split rule (the cap goes with it): F-P x2; T-P-Ne current rule (worse, z 0, limit 4, 1M, noise 0.03) x2; T-P-Ne-g (T-P-Ne without --reset-leaves inf, with --trial-error log) with --split-rule grow (stagnation 2 windows at 2 block-SE, one credit per 5M positions capped at 2, trial against an unsplit shadow kept iff gain > 2 block-SE and the MDL bar, no cap) x2. chessai-grow1 training (byte-identical without grow flags), hybrid16 games (leagues/specs/batch6/grow-p1.json). Prediction (analyst): grow minus current -15..10 at v150, -10..15 at v300; grow 9-11 leaves at v300 | 44 | plateau scaled, min 300, cap 400 | 11-11 20:36-22:02 | DONE at v399 (cap 400; three failed creates first: grow refuses --reset-leaves, needs --trial-error log; then a self-games file name too long). Read leagues/arenas/diag/grow-p1-stage (v150 and v399, plies 1-2, T 0.02), head to head (ply 1 / ply 2, 600 games each, 95% CI about ±28): grow - current -11 / 21 at v150, -3 / -22 at v399 (a tie; prediction -15..10 at v150 and -10..15 at v300). Over F-P: grow 54 / 21 and current 58 / -43 at v150; grow 97 / 5 and current 110 / 18 at v399. v399 over v150: grow 122 / 92, current 108 / 81, F-P -5 / 45 (trees keep gaining at ply 1 long after their last split, F-P doesn't). Growth: kept splits at 7.0, 12.3, 27.3, 49.9, 77.1M (seed 1, 11 leaves) and 7.0, 15.7, 26.4, 36.6, 71.6, 78.0M (seed 2, 13 leaves); current rule 9 leaves by v51 (10.8M). 27 trials, 11 kept, 16 undone: EVERY undone trial passed the z bar (z 3.1-9.9, gains 0.0024-0.0068) and failed only the MDL bar (0.0055-0.0081); leaf 3 was retried 4 times per seed. The growth rule removes the cap at no cost in strength; the MDL bar sets its pace (split_table.py grow-p1) |
| router-p1 | ply 1 | Router-split rule as the only factor (the user 11-11 ~20:40: "test multiple various router split strategies in an experiment"; analyst 6.41/6.46). Recipe D-P-Ne-ctx-r4 (D-P-Ne-ctx without --router-split-z 20; leaf rule fixed at worse, z 0, limit 4, 1M, noise 0.03; router limit 4): F-P x2; C: no router splits (--router-split-z 1e9, plays as T-P-Ne) x2; Z: --router-split-z 20 (today's drift trigger) x2; H: --router-split-rule stall-h (own routing error stagnant plus oracle headroom H_v over 2 block-SE and MDL, share guard 0.9) x2; HG: --router-split-rule stall-hg (H plus a paired shadow test G_real - G_null, k 5,112) x2; --log-router-stats on H and HG. chessai-grow3 training (byte-identical without the new flags), hybrid16 games (leagues/specs/batch6/router-p1.json). Predictions (analyst 6.41, ply 1 / ply 2): C - F-P 80-120 / 45-80; Z - C -10..10 / -45..0; H - C -20..10 / -30..10 with 2-4 router splits per seed at v120-v250; HG - C within 15 with 0-1 splits per seed | 45 | plateau scaled, min 300, cap 400 | 11-11 21:56-11-12 01:39 | DONE at v299 (plateau). Read leagues/arenas/diag/router-p1-stage (v150 and v299, plies 1-2, T 0.02), head to head against C (600 games, CI about ±28; ply 1 / ply 2): HG (stall-hg, the shadow test): 2 / -10 at v150, 4 / 5 at v299 (0 router splits in either seed: C's twin, as predicted); H 4 router splits per seed (v26-v51; the schedule arm): -8 / 5 at v150, -41 / 17 at v299; Z (z 20) 4 router splits per seed (v29-v44; not logged by the z rule, read from the model headers: 8 splits, 25 nodes vs C's 4 and 13): -30 / -9 at v150, -37 / -7 at v299; F-P: -74 / -69 at v150, -99 / -70 at v299 (C over F-P 99 / 70, inside 80-120 / 45-80). Against 6.41/6.47: HG = C (right); H -41 at ply 1 below its band (-20..10): by the analyst's rule G was right to refuse; Z -37 at ply 1 below its band (-10..10), ply 2 inside. Router splits (cluster sibling sets, all made by ~v50) cost ~35-40 at ply 1 and nothing significant at ply 2; the shadow test refused them all. leagues/specs/arenas/dag_events.py lists logged split events only (H, HG) |
| replay-p2 | ply 2 | Replay test (analyst 6.46 (i); offline drift/order checks: the probit and LR learners follow their newest data, and a stationary mix was best). One factor against probit-late-p2: "replay": {"share": 0.5, "window": 50}, i.e. each cycle's local set is its own games plus as many again sampled from its own games of the 50 cycles before (leagues.py replay, built 21:55). Everything else is probit-late-p2's: seed 38 (a twin), F-P x2, T-P-Ne (NB error router, non-merging, limit 4, 1M, noise 0.03) x2, ply-2 field, self-games, 12 rounds, scaled plateau floor 400, cap 600; hybrid16 games (= hybrid13's), hybrid6 training. A 3-cycle pilot reproduced probit-late-p2's cycles 1-2 byte for byte. Prediction (analyst 6.46): F-P over pure F-P 60-150 at v400; tree over flat 20-60 | 38 | plateau scaled, min 400, cap 600 | 11-11 22:09-11-12 00:04 | DONE at v542 (plateau). Read per ply (leagues/arenas/diag/replay-p2-read-p1, -p2, -p3; v300 and v399 against probit-late-p2; T 0.02), head to head (600 games, CI about ±28; ply 1 / 2 / 3): replay F-P minus pure F-P 38 / -123 / -1 at v300, 37 / -64 / -5 at v399; replay tree minus pure tree 27 / -131 / -47 at v300, 79 / -42 / -30 at v399. Tree minus flat inside replay 16 / 49 / -40 at v300, 3 / -64 / -5 at v399; inside pure -6 / 7 / -27, -36 / -18 / -1. Replay helps at ply 1 and HURTS at ply 2 (the training ply), the analyst's case (4) (flat 30+ below pure); prediction (F-P 60-150 over pure at ply 2) failed. Replayed games are shuffled into each cycle's positions (train_rounds), so the cycle does not end on stale games. Per-ply reads: ply 2 3.5 min, ply 1 0.7 min, ply 3 28 min Late read (leagues/arenas/diag/replay-p2-late-p2, ply 2): replay v542 over replay v399 F-P 7, tree 36; replay v542 vs pure v399 F-P 20, tree -20; v399 replay vs pure again F-P -42, tree -74. Replay catches up with pure only ~140 cycles later. Replayed games carry only outcomes and moves (no stale evals). Analyst 6.51: weaker play has the blunder structure of shallower play, so the lineage's own early games pull the value function toward shallow play (helps ply 1, hurts ply 2); P2x helped offline because it is depth-2 play by OTHER fixed players. Own-field replay at 0.5 is closed; an Elo-windowed replay is the only variant worth a field, after diversity-p2 |
| grow-p1b | ply 1 | Growth rule, the MDL bar as the only factor (analyst 6.48 after grow-p1: all 16 undone trials passed the z bar and failed only MDL; the trial gain is prequential, so the bar double-counts). F-P x2; T-P-Ne-grow (grow-p1's grow line as a recipe row: grow1's rule, uncapped) x2; the same with --grow-mdl 0 (z bar alone) x2. The analyst's arm A also had --grow-min-gain 0.001, which no binary has (the z bar sat at 0.0006-0.0017 in grow-p1); arm B (grow4, no stagnation gate) goes in the next field. All lines train on chessai-grow1, hybrid16 games (leagues/specs/batch6/grow-p1b.json). Predictions (6.48): A minus grow1 -15..10 at ply 1, -30..0 at ply 2; A over F-P 80-120 at ply 1; A 13-16 kept splits per seed by v399 (27-33 leaves, 40-49 nodes; 6.48 addendum, restated within the credit bound of one per 5M, kept splits not refunded), 0-3 undone. Rule: A within 10 of grow1 at both plies with 25+ leaves = growth governed by the z bar and D alone; A 20+ below at ply 2 = a min-gain floor is needed (~0.003) | 46 | plateau scaled, min 300, cap 400 | 11-11 22:15-23:29 | DONE at v352 (plateau). Read per ply (leagues/arenas/diag/grow-p1b-read-p1, -p2; v150 and v352; T 0.02), head to head (600 games, CI about ±28; ply 1 / ply 2): no-bar (A) minus grow1 -13 / 40 at v150, -2 / 19 at v352. Over F-P: A 16 / 23 and grow1 55 / 20 at v150; A 80 / 48 and grow1 115 / 99 at v352. Growth: A kept 11 of 11 trials in both seeds (25 leaves at the end, one trial in flight), grow1 5-6 of 11 (13-15 leaves). Against 6.48 (addendum): ply 1 inside (-15..10), ply 2 ABOVE the band (-30..0), A over F-P 80 at ply 1 (80-120). By the rule (A within 10 of grow1, or better, with 25 leaves) growth is governed by the z bar and the credit rate D alone; no min-gain floor needed; the bar goes. Arm B (grow4's age gate + snapshot priority) runs on top of A in grow-p1c |
| grow-p1c | ply 1 | Growth rule, the trigger as the only factor, stacked on grow-p1b's winner (analyst 6.48 note). F-P x2; A = T-P-Ne-grow --grow-mdl 0 (grow1's stall gate, excess priority, no MDL bar) x2; B = A with --grow-gate age --grow-priority snapshot (grow4: a leaf eligible after 3 closed own windows plus a credit; the most stagnant by paired snapshot gain tried first) x2. Every line trains on chessai-grow4 (the T-P-Ne-grow recipe row moved to grow4 at 23:40; byte-identical to grow1 without the new flags), hybrid16 games (leagues/specs/batch6/grow-p1c.json). Prediction (6.50, no-bar regime): B's first split at 5M (A 6-7M); B 14 kept splits per seed by 72M (29 leaves) against A's 11-12 (23-25); leaves at v300 B 23-27, A 19-23; 0-2 undone each; B's priority goes to the most converged (often largest, low-material) leaf, A's to the highest-loss middlegame leaf; B minus A within 10 at both plies; both over F-P 70-120 at ply 1, 20-80 at ply 2 | 47 | plateau scaled, min 300, cap 400 | 11-11 23:35-11-12 00:41 | DONE at v354 (plateau). Read per ply (leagues/arenas/diag/grow-p1c-read-p1, -p2; v150 and v354; T 0.02), head to head (600 games, CI about ±28; ply 1 / ply 2): B (age gate + snapshot priority) minus A (stall gate), both without the MDL bar: 2 / -6 at v150, 1 / 27 at v354. Over F-P: B 52 / 66 and A 72 / 50 at v150; B 48 / 28 and A 47 / 9 at v354. Growth: B's first trial at 5.0M (A 6.0M); B kept 11 and 12 of 11 and 12 trials (27 leaves at the end with trials in flight), A 10 of 10 (25). Against 6.50: first split 5M vs 6-7M (right); B 14 kept by 72M (11-12 by 68M); B minus A within 10 at ply 1 (right), 27 [-1, 55] at ply 2 at v354; over F-P 47-52 at ply 1 (band 70-120, below), 9-28 at ply 2 (20-80). The trigger is neutral in strength; B grows slightly faster and without the drift-driven gate. Leaf-growth series at ply 1 done: grow1 = capped rule (grow-p1), no MDL bar >= grow1 with twice the leaves (grow-p1b), age + snapshot = stall (grow-p1c) |
| diversity-p2 | ply 2 | The data regime at ply 2 (the user 11-12 ~01:05: "continue the research with the analyst's suggestions", the go-ahead for frozen foreign references; analyst 6.45 (2), 6.51, 6.53 and its correction). F-P x2 and T-P-Ne (limit 4, 1M, noise 0.03) x2 as probit-late-p2, ply-2 field, self-games (lines only), 12 rounds, plus four FROZEN REFERENCES at ply 2: error-routing-p1's v314 finals (F-P x2, T-P-Ne x2, ply-1-trained; the offline P2x players), new driver key frozen_references (installed 01:20, byte-identical without it). All game kinds train: per cycle 1,584 round-robin games (line-line 672, line-ref 768, ref-ref 144) plus 96 self-games, the batch of base-probit-p2. hybrid16 games, hybrid6 training, seed 48. Comparators: base-probit-p2's F-P at equal versions (equal batch), probit-late-p2 at equal versions and at equal positions (diversity ~v169 = pure v399). Predictions (6.53 correction, ply 2, head to head): over base-probit-p2's F-P at v300 100-200; over probit-late-p2's F-P at equal positions 60-150, at v300 120-220; tree over its flat inside 0-50; ply 1 0-40; ply 3 within 40. Null: within 30 of base-probit-p2's F-P at equal batch (then the next league goes back to ply 1) | 48 | plateau scaled, min 300, cap 400 | 11-12 01:22- | running (~11 s a cycle, ~4-5 CPU-h); read waiter queued (models/from-scratch/global11-drafts/diversity-read.sh: per ply 2, 1 at v300 and the final; ply 3 at the final only; output diversity-p2-read.out) EQUAL-POSITIONS READ (by hand, 02:05; leagues/arenas/diag/diversity-p2-eqpos-p2, ply 2, T 0.02, 600 games a cell, CI about ±28): equal positions are diversity v182 (34.73M a line) = probit-late v399 (34.66M) and v140 = v300 (2.19x positions a cycle, not 2.36x). F-P diversity minus probit-late -2 [-30, 26] at v182/v399, 6 at v140/v300 (prediction 60-150: inside the null of 30); tree -59 [-88, -31] and -48 [-76, -20]; tree minus flat inside diversity -21 (v182) and -61 (v140), inside probit-late -3 (v399). The main read (equal batch vs base-probit-p2, plies 1-3) follows the run's end. ANALYST 6.56: equal positions can't decide anything (a league progresses per cycle; 2.19x positions a cycle is worth far less than 2.19x cycles); the informative reads are equal versions and equal batch. CROSSING (history.csv, ply 2): the lines passed the frozen references at ~cycle 8 (1545-1557 vs 1425/1461) and sat 150-250 above them (~1640-1690 vs ~1436/1449) for the rest of the run, so diversity-p2 tested 58% of positions from much weaker foreign play (analyst predicted a crossing at v60-150). SOURCE PER LEAF (tree #1 v226, latest cycle's games routed by treeview; models/from-scratch/srcleaf): reference games 60.8% of positions; leaf 12 84% reference (0.048 nats worse than the flat on line-line games, 0.079 better on reference games), leaf 10 2.7% and leaf 11 15% (line-specific), the other six 52-72%; prediction was 2+ of 9 leaves over 80%. On line games the tree's loss beats the flat's (0.779 vs 0.814, mostly the endgame leaf) while it plays 21-61 worse at ply 2. **DONE at v314 (plateau, 02:29). MAIN READ** (per-ply arenas leagues/arenas/diag/diversity-p2-read-p1/-p2 at v300 and v314, -read3-p3 at v314; head to head, 600 games, CI about ±28; v300 / v314): PLY 2: F-P minus base-probit-p2's F-P (equal batch) 6 / 15 (the 6.53 null: weaker foreign play is worth as much as own-field sibling games; 6.56 had predicted -80..-20); F-P minus probit-late's F-P (equal versions, 2.2x less batch) -10 / 44; tree minus probit-late's tree -34 / -15; tree minus its flat -15 / -5. PLY 1: F-P minus bfp 6 / 31, minus pfp 84 / 55; tree minus ptr 58 / 51; tree minus flat -2 / -34. PLY 3 (v314): F-P minus bfp -5, minus pfp 74; tree minus ptr 56; tree minus flat -42. Reading: no replay-style harm from foreign weak play at equal batch; the batch per cycle, not the references, separates diversity (and base-probit) from probit-late at plies 1 and 3. BATCH-SIZE READ (02:52, analyst 6.58 (c); leagues/arenas/diag/batch-read-p1/-p2/-p3): base-probit-p2's F-P minus probit-late-p2's at equal versions (v300 / v399): -37 / -55 at ply 2 (prediction 10-50: opposite sign), 83 / 47 at ply 1, -17 / 68 at ply 3. The bigger mixed-kind batch (1,584 games a cycle, six lines of three kinds) made a better ply-1 player and a worse ply-2 one than the smaller F-P/T-P-Ne batch (720); volume and composition are confounded. |
| grow-l-p1 | ply 1 | The growth rule's first LR test (analyst 6.55; the LR seat of the 6.54 six has never been played, and the trial statistic is untested on LR leaves). One factor, growing vs capped LR error tree: F-L x2; T-L-Ne capped (limit 4, 1M; error-dag-p1's tree, 46 / 55 over F-L) x2; T-L-Ne-grow (new recipe row: T-P-Ne-grow with LR leaves, no split noise) on the recommended leaf rule --grow-mdl 0 --grow-gate age --grow-priority snapshot (W 1M, C 2, z 2, D 5M as the probit series) x2. Every line trains on chessai-grow4, hybrid16 games, games_per_cycle 1584 (12 entries, 12 rounds), seed 49 (leagues/specs/batch6/grow-l-p1.json). Predictions (6.55, ply 1 / ply 2, ±28): capped over F-L 30-60 / 20-60; grow minus capped -10..10 / -10..20; grow over F-L 25-65 at ply 1; grow 25-33 leaves by v400, 0-3 undone, kept trials at z 3-10. Nulls: grow 20+ below capped at ply 1 at the final, or fewer than 6 kept splits by v400 with trials failing the z bar, pull T-L-Ne-grow from the six (follow-up W 2M, not a lower z); capped within 15 of F-L fails the LR seats | 49 | plateau scaled, min 300, cap 400 | 11-12 01:59-02:46 | DONE at v299 (plateau at the floor; relaunched between cycles 12 and 13 at 02:01 to log to repeat.log, nothing else changed). Read queued: models/from-scratch/global11-drafts/grow-l-p1-read.out (plies 1, 2; v150 and the final; split table and growth log) READ (leagues/arenas/diag/grow-l-p1-read-p1/-p2; head to head, 600 games, CI about ±28; v150 / v299): PLY 1: grow minus capped -25 / -16; capped minus F-L 38 / 21; grow minus F-L 17 / 34. PLY 2: grow minus capped 48 / 41; capped minus F-L -101 / -38 (error-dag-p1 had 55); grow minus F-L -64 / -47. GROWTH: LR #1 15 trials, 5 kept, 10 undone (7 with negative gain, z -1.1..-4.9), kept z 2.1-7.0, mean kept gain 0.0008 nats; LR #2 14 trials, 4 kept, 10 undone (8 negative), mean kept gain 0.0007; 13 leaves at the end, the count going up and down all run. Probit under the same rule (grow-p1c): every trial kept (10-12 a line), z 2.4-35, mean kept gain 0.011-0.014 nats, 15x LR's. LR splits are mostly WORSE than the unsplit leaf within the trial, not under-powered. |
| grow-l-p1b | ply 1 | The LR growing tree's split mechanics as the only factor (analyst 6.59 after grow-l-p1; mechanism checked in grow4-src: a split clones the leaf for each child and for the trial's shadow and applies forget(forget_factor); LR's forget divides its AdaGrad sums, so at --forget-factor 1, every recipe's setting, the copies step at the parent's late-life rate). F-L x2; T-L-Ne-grow (grow-l-p1's rule) with --forget-factor 1 x2 and with --forget-factor 10 (the engine's default) x2. grow4 training, hybrid16 games, games_per_cycle 1584, seed 50 (leagues/specs/batch6/grow-l-p1b.json). Predictions (6.59): forget 10 keeps 8+ of 12 trials (z 3-8, gains 0.002-0.005 nats), 0-3 undone, 17-25 leaves; forget 10 minus forget 1 0-30 at ply 1, -20..20 at ply 2; forget 10 over F-L 30-60 at ply 1. Plan: holds = T-L-Ne-grow (forget 10) takes the LR tree seat of the six; fewer than 6 kept or 20+ below forget 1 at ply 1 = F-L alone on the LR side | 50 | plateau scaled, min 300, cap 400 | 11-12 02:53-03:44 | DONE at v347 (plateau). READ (leagues/arenas/diag/grow-l-p1b-read-p1/-p2; head to head, 600 games, CI about ±28; v150 / v347): PLY 1: forget 10 minus forget 1 -24 / -13; forget 10 minus F-L -17 / 41; forget 1 minus F-L 16 / 89. PLY 2: forget 10 minus forget 1 30 / 75; forget 10 minus F-L 61 / 36; forget 1 minus F-L 9 / -28. TRIALS: forget 1 18 and 17 trials, 4 and 4 kept, 14 and 13 undone (10 and 11 with negative gain), mean kept gain 0.0009-0.0010 nats, 13 leaves each; forget 10 15 and 12 trials, 6 and 7 kept, 9 and 5 undone (5 and 4 negative), kept z 2.0-8.8, mean kept gain 0.0021-0.0022 nats, 15 and 19 leaves. Against 6.59: gains inside, kept and undone short of the band (8+ kept, 0-3 undone predicted); the plan rule (6+ kept, not 20+ below forget 1 at ply 1) passes: T-L-Ne-grow with forget 10 takes the LR seat, half-confirmed (LR trials still fail 40-60% where probit's never do) |
| diversity-p2c | ply 2 | The user's regime at ply 2: coverage from STRONGER foreign play under the Elo window (analyst 6.56-6.57, after diversity-p2's lines passed its ply-1-trained references at ~cycle 8 and then trained on 200-Elo-weaker foreign play). Four frozen references = base-probit-p2's v599 finals (F-P x2, T-P-Nc-merge fresh cluster trees x2), new driver key reference_window margin 30 (scope (a): a line trains on a game with a frozen reference only while every reference in it is rated at or above its current version's Elo minus 30; line-line games and self-games always; installed 02:43, byte-identical without it). Lines, seed and settings as diversity-p2 (F-P x2, T-P-Ne limit 4, 1M, noise 0.03 x2, seed 48, self-games, hybrid16 games, hybrid6 training) but games_per_cycle 1584 (leagues/specs/ply2/diversity-p2c.json). Comparators: base-probit-p2's F-P at equal versions (equal batch), probit-late-p2 and diversity-p2 at equal versions. Predictions (6.57; ply 2, ±28): p2c F-P v150 vs probit-late F-P v300 0-60, v200 20-80; vs bfp at v300 -20..40, at v400 vs bfp v450 -40..20; over diversity-p2's F-P at v300 20-80; tree vs its flat -40..0; ply 1 final vs pure -20..30. Plan rules: F-P 40+ over bfp at v400 = the league goes to ply 2 with the window; within 30 = speed only; 40+ below = the references must be a mix of strengths above the window | 48 | plateau scaled, min 300, cap 400 | 11-12 02:46-03:50 | DONE at v331 (plateau; launched 02:46 after analyst 6.58: launch as built, the window cutting passed references is the league's rule). Created 02:43. Pilot (3 cycles, scratch): the references rated 1661/1614 (F-P) and 1511/1492 (cluster trees) against the lines' ~920-950 at cycle 3, so the window was inert; the cluster-tree references sit ~150 below the F-P ones, and diversity-p2's lines reached ~1550-1650 on the same zero by cycles 8-15, so the window may cut the two tree references early (every cut is logged) ANALYST 6.58 (after diversity-p2's null): no ply-2 player on disk stays above a from-scratch ply-2 line within the run, so 'coverage from stronger foreign play' can't be tested with fixed references; p2c tests the window regime literally. Revised predictions (ply 2, ±28): cluster references cut by cycle ~10, the F-P references between v100 and v300; F-P vs bfp at v300 within 30 (centre 10), at v400 vs bfp v450 within 30; ladder p2c v150 vs pfp v300 0-40; tree under its flat 0-40. Meaning: 30+ over bfp = window-mediated references add strength; within 30 = a ply-2 league's value is players and compute, not a data effect; 30+ below = the window cut references still stronger cold (then the league's window needs cold or pooled ratings). Read adds: the cut cycles, and each cut reference cold against the line's version of its cut cycle. READ (leagues/arenas/diag/diversity-p2c-read-p2/-p1; head to head, 600 games, CI about ±28): PLY 2: F-P minus bfp 73 at v300 (prediction within 30), final v331 minus bfp v450 12; minus pfp 51 at v300, final minus pfp v399 22; minus diversity-p2's F-P -1 at v300, 37 at the final; tree minus flat -18 / -10 / 8 (v150 / v300 / final); ladder against pfp v300: v100 9, v150 77, v200 37; v150 minus bfp v300 -12. PLY 1 (final): minus bfp v450 -8, minus pfp v399 45, minus dfp v314 32; tree minus flat -34. WINDOW: cluster references cut from cycle 10-11, F-P references from cycle 19 (F-P lines) and 97 (one tree line); 322 of 332 cycles cut something; F-P lines kept 49% of the reference games in cycles 1-50, then 32%, 26-28%, 22-24%, 16-18% by block, the tree lines 60-88% throughout (rated lower in-run). COLD TEST OF THE CUTS: cluster references against the line versions that cut them 162 [140, 184] (1,200 games); F-P references against theirs 37 [14, 60]; F-P references against the final F-P 23 [-5, 51]: the window, on in-run T 0.1 ratings, cut references that were stronger cold. Triangle at v300 non-transitive: cfp-dfp -1, dfp-bfp 6 (diversity read), cfp-bfp 73. DIRECT CHECK (04:15): cold, the cluster references against the F-P references -12 [-40, 16] (600 games), while in-run they averaged ~1560 against ~1685: the in-run T 0.1 ratings under-rate this tree kind by ~110 (analyst 6.63: kind-specific calibration-slope bias; every in-run decision uses these ratings). |
| fponly-p2 | ply 2 | Composition vs volume at ply 2 (analyst 6.61, after the batch-size read): base-probit-p2's F-P (1,584 games a cycle among F-P x2, cluster trees x2 and DAGs x2, the trees and DAGs 10-50 below the flats at ply 2) was 37-55 below probit-late-p2's F-P (720 games among F-P x2 and T-P-Ne x2, trees level with flats) at ply 2 and 47-83 above at ply 1, at equal versions. F-P x4, games_per_cycle 1584, self-games, entry ply 2, no ply-1 entries, hybrid6 training (leagues/specs/ply2/fponly-p2.json). Predictions (6.61): minus bfp 20-60 at ply 2, -60..-20 at ply 1; minus pfp within 30. Kill result: within 15 of bfp at ply 2 and 30+ below pfp = the harm is volume (then 1,584 -> ~720 for ply-2 fields) | 51 | plateau scaled, min 300, cap 400 | 11-12 03:47-04:45 | DONE at v320 (plateau, 35 cycles). READ (leagues/arenas/diag/fponly-p2-read-p2/-p1; head to head, x4's four lines against each comparator's two: 1,200 games, about ±20; bfp-pfp 600, ±28), v300 / v320: PLY 2: x4 minus bfp 65 [46, 86] / 57 [37, 77]; minus pfp 78 [58, 98] / 28 [8, 48]; bfp minus pfp -53 / -10. PLY 1: x4 minus bfp 12 [-7, 32] / 52 [32, 72]; minus pfp 39 / 70; bfp minus pfp 83 / 27. VOLUME DOESN'T HARM (kill result not met): at equal volume the pure F-P field beats the mixed one by ~60 at ply 2 (prediction 20-60: at the top) and is not worse at ply 1 (prediction -60..-20: wrong sign, so bfp's ply-1 lead over pfp came from volume, not from its mix); with twice the volume and no trees x4 beats pfp by 28-78 at ply 2. On the bfp zero at v300: x4 65, diversity-p2c's F-P 73, diversity-p2's 6. PANEL CHECK (06:15; leagues/arenas/diag/panel-fponly-p2-v299; ply 2, v299, each F-P against the strangers replay-p2 + diversity-p2, both kinds, 75 round-robins): x4 minus bfp 87 [70, 105] (direct 42), minus probit-late's F-P 65 [47, 82] (direct 24), minus p2c's 56 (21), minus p2d's 43 (17). THE VERDICT HOLDS AND GROWS: the pure F-P field's F-P is the strongest ply-2 evaluator on disk, 43-87 over every other ply-2 F-P against strangers. |
| diversity-p2d | ply 2 | diversity-p2c with the reference window REMOVED (analyst 6.65; one factor: the window as rated in-run, whose T 0.1 ratings cut references that were stronger cold). Same seed, references (md5-checked), games_per_cycle, floor and cap; byte-identical to p2c through v9, diverging at v10, p2c's first cut (leagues/specs/ply2/diversity-p2d.json). Read power in the read (6.65): one arena per (ply, version) with only d's and c's lines, 300 round-robins (2,400 games a 2x2 cell, ±14), at v300 and min(d final, c final 331), plies 2 and 1, plus d's F-P at the final vs bfp v450 at ply 2 (models/from-scratch/global11-drafts/p2d-read.sh). Predictions: d over c 0-40 at ply 2 (centre 15); d over bfp450 0-40; ply 1 within 20. Rule: 30+ over c at both reads = the window as rated cost the ceiling; within 15 at both = it cost nothing here | 48 | plateau scaled, min 300, cap 400 | 11-12 04:34-05:43 | DONE at v299 (plateau at the floor; the last cycle doesn't train, so v300 doesn't exist: the read is at v250 and v299). READ (leagues/arenas/diag/diversity-p2d-read-p2/p1-v250/v299; one arena per (ply, version), d's and c's lines only, 300 round-robins, 2,400 games a cell, about ±14; head to head), v250 / v299: PLY 2: d's F-P minus c's -7 [-21, 7] / -19 [-33, -5]; d's tree minus c's 4 / -14. PLY 1: F-P -9 / 8; tree 0 / -2. d's F-P v299 minus bfp v450 -30 [-58, -2] (c's final v331: 12). Prediction (6.65) d over c 0-40 at ply 2: below it. Rule: within 15 at both = the window as rated cost nothing; the F-P's -19 at v299 is a small COST of removing it (the lines passed the references late, so the windowless lines kept training on weaker same-kind play, which 6.66 says costs at ply 2). The in-run bias is a standings problem here, not a strength one. CORRECTION (06:15, panel 2, section 1): against strangers ({y, v}: replay-p2 and diversity-p2 lines) d's F-P is 21 ABOVE c's (6 against {l, y, v}), so the direct -19 is a matchup, not a verdict: no detectable effect of the window either way. CO-ADAPTATION: inside d the tree beats its own flat by 61 / 51 at ply 2 (26 / -21 at ply 1), but routed through c's lines (strangers) d's tree minus d's flat is -37..22 / -11..30: within-run tree-vs-flat head-to-heads can sit 40-70 off the strangers' view (in c: direct -42 / -9, routes -48..11 / -16..25; in p2c's own read at v300, c's tree minus flat -10 direct, -65 via bfp, -55 via pfp, 10 via dfp). Same-kind cross-run cells (the verdicts) are not affected. |
| fponly-p2-720 | ply 2 | fponly-p2 at games_per_cycle 720 (analyst 6.66/6.67: volume as the only factor in a pure F-P field, per version and per CPU; the user's games-per-cycle standard). 13 rounds, 728 games a cycle (fponly-p2: 28, 1,568); self-games unchanged; same lines (F-P x4), seed 51, hybrid6 training (leagues/specs/ply2/fponly-p2-720.json). Floor 560, cap 700, so the run passes the version of equal measured CPU with fponly-p2's v300 (2.11 CPU-h) and v320. Predictions (±20): equal versions 1,584 over 720 by 20-60 at ply 2, 10-50 at ply 1; equal CPU 720 over 1,584 by -10..30 (centre 10; 6.67 note: moved from 0-40, since equal CPU is ~v520, not v660) at ply 2. Rule: 720 at equal CPU 30+ ahead = set the standard per compute; within 20 = keep 1,584; 1,584 failing to beat 720 at equal versions by 20 ends the useful-volume account | 51 | plateau scaled, min 560, cap 700 | 11-12 05:08-06:22 | DONE at v559 (plateau at the floor). Equal measured CPU: x4's v300 (2.11 CPU-h) = x7's v494, x4's v320 (2.27) = x7's v534. DIRECT READ (fponly720-read-*; 2,400 games a cell; the two runs share cycle 1, near-sibling lineages, so matchups possible): EQUAL VERSIONS 1,584 minus 720 at ply 2: 35 [21, 49] (v300), 4 [-9, 18] (v320); ply 1: 21, 14. EQUAL CPU 720 minus 1,584 at ply 2: -6 [-20, 8] (x7 v494 vs x4 v300), -62 [-77, -48] (x7 v534 vs x4 v320); ply 1: -33, -33. 720 is not ahead per compute (keep 1,584 by the rule); the equal-version gap is 35 then 4. PANEL READ (06:30; leagues/arenas/diag/panel-fp720-p2/-p1; 30 round-robins, each arm 2,880 games against the standing ply-2 panel y + v + l at v299, ±19 on a difference): PLY 2: equal versions 1,584 minus 720 16 [-3, 34] (v300; direct 40), 14 [-5, 33] (v320; direct 24); equal CPU 720 minus 1,584 8 [-11, 27] (x7 v494 vs x4 v300; direct -3), -10 [-29, 9] (x7 v534 vs x4 v320; direct -59), x7's final v559 minus x4 v320 -8 [-27, 11]; against the panel x7 is flat from v494 (116, 113, 115), x4 108 at v300, 123 at v320. PLY 1: equal versions 21 [3, 39], 27 [8, 45]; equal CPU -11 [-29, 7], -43 [-61, -25], final -28 [-46, -10]. VERDICT: per compute 720 is level at ply 2 and 11-43 behind at ply 1; per version 1,584 leads by 14-16 at ply 2 (CIs reach 0) and 21-27 at ply 1. KEEP 1,584 (the rule: within 20 at equal CPU). The ply-2 equal-version gap fell short of the rule's 20, so by its letter 6.66's useful-volume account fails at ply 2 (it holds at ply 1): sent to the analyst. Direct cells ran up to 49 off the panel (v534 vs v320: -59 direct, -10 panel; the runs share cycle 1). ANALYST 6.71: the account is revised: doubling useful games a cycle is worth ~15 at ply 2 and ~25 at ply 1 per version, nothing per CPU; x4's 65 over probit-late's F-P (720 games, two thirds against level trees) was composition, not volume. 1,584 stays. |
| capped-l-f10-p1 | ply 1 | The capped LR error tree's split boost as the only factor (analyst 6.62 (b), revived by 6.67 after lrdecay failed offline): F-L x2; T-L-Ne capped (limit 4, 1M) at --forget-factor 1 x2 and at 10 x2 (leagues/specs/batch6/capped-l-f10-p1.json; chessai-grow5 training, hybrid16 games, 1,584 games a cycle). Predictions: forget 10 over forget 1 -10..10 at ply 1 and 20-60 at ply 2; over F-L 30-60 / 20-60. Rule: at or above at ply 1 and 20+ over at ply 2 = forget 10 for the capped LR control; 20 below at ply 1 = capped trees keep forget 1 | 54 | plateau scaled, min 300, cap 400 | 11-12 05:10-06:08 | DONE at v319 (plateau). DIRECT READ (leagues/arenas/diag/capped-f10-read-p1/p2-v150/v319; 300 round-robins, 2,400 games a cell, ±14; within-run cells, so matchups possible: panel read against grow-l-p1 and grow-l-p1b's lines running), v150 / v319: PLY 1: forget 10 minus forget 1 16 / 0; forget 10 minus F-L 16 / 44; forget 1 minus F-L -10 / 51. PLY 2: forget 10 minus forget 1 107 [93, 122] / 50 [36, 64]; forget 10 minus F-L 45 / 49; forget 1 minus F-L -56 / -27. Predictions met (forget 10 level at ply 1, 20-60 over at ply 2: at or above); rule: forget 10 becomes the capped LR control's setting (confirms 6.62 (b)): the capped LR tree's ply-2 fragility (below F-L at ply 2 at forget 1) was the frozen-step defect. PANEL READ (06:20; leagues/arenas/diag/capped-f10-panel-p1/p2-v150/v299; each arm against the 12 lines of grow-l-p1 and grow-l-p1b, strangers; 7,200 games an arm), v150 / v299: PLY 2: forget 10 minus forget 1 70 [58, 81] / 83 [72, 95] (direct 107 / 82); forget 10 minus F-L 5 / 21; forget 1 minus F-L -65 / -62. PLY 1: forget 10 minus forget 1 -19 [-30, -7] / 8; forget 10 minus F-L 14 / 39; forget 1 minus F-L 33 / 31. The ply-2 verdict holds against strangers (forget 10 takes the capped LR tree from ~63 below F-L to ~13 above); at ply 1 the two are level by the final. Same arena: grow-l-p1b's growing tree forget 10 minus forget 1 4 / 16 at ply 1, -19 / 33 at ply 2 (direct -10 / 3, 35 / 44); grow-l-p1's capped tree (forget 1) minus its F-L 5 / 35 at ply 1, -103 / -109 at ply 2. |
| fponly-p2b | ply 2 | fponly-p2 replicated at seed 55 (analyst 6.70 slot 1): the pure F-P field's lead (43-87 over every other ply-2 F-P against strangers) rests on one run. F-P x4, 1,584 games a cycle, self-games, hybrid6 training (leagues/specs/ply2/fponly-p2b.json). Prediction: within 30 of fponly-p2's F-P against the ply-2 panel (y + v + l at v299), i.e. 55-120 over bfp. Rule: 40+ below = a lucky seed, and the pure-field advantage is 30-60 | 55 | plateau scaled, min 300, cap 400 | 11-12 06:32-07:49 | DONE at v352 (plateau). PANEL READ (fponly-p2b-read.sh; leagues/arenas/diag/panel-fponly-p2b; ply 2, panel y + v + l at v299, 30 round-robins, 2,880 games an arm): at v299 fponly-p2b's F-P and fponly-p2's both score 104 against the panel: difference -1 [-20, 18] (direct 16); over bfp 95 [73, 117] and 96 [73, 118] (direct 53 and 32). THE PURE-FIELD LEAD REPLICATES (prediction: within 30 of fponly-p2, 55-120 over bfp: met). Finals: fponly-p2b v352 minus fponly-p2 v320 -48 [-67, -29] (direct -40), v352 minus its own v299 -27 [-45, -8]: single versions at unequal points (fponly-p2's v320 is 21 over its v299), within version noise (~17 SD a version); by the best-version rule the read is v299's. A third seed (weight-p2's controls, 11-12 09:16) came out 37 [17, 57] over fponly-p2's F-P at ply 2, so the pure-field lead at ply 2 is 95 ±40 across seeds (analyst 6.75). At ply 3 the lead is gone (-2 / -7: section 1, 11-12 ~08:03). |
| probit-f10-p1 | ply 1 | The capped PROBIT error tree's split forget as the only factor (6.70 slot 2): F-P x2; T-P-Ne capped (limit 4, 1M, noise 0.03) at --forget-factor 1 x2 and 10 x2; 1,584 games, hybrid16 games, hybrid6 training (leagues/specs/batch6/probit-f10-p1.json). Prediction: forget 10 within 15 of forget 1 either way (ADF keeps the probit leaves' steps alive; only the NB routers' counts change). Rule: 20+ below keeps forget 1 | 56 | plateau scaled, min 300, cap 400 | 11-12 06:32-07:41 | DONE at v337 (plateau). PANEL READ (probit-f10-read.sh; leagues/arenas/diag/panel-probit-f10-p1/-p2-v150/-v337; panel r + e + g + h at v300, 40 round-robins, 2,560 games an arm, ±19 on a difference), v150 / v337: forget 10 minus forget 1, ply 1 -17 [-36, 2] / 7 [-12, 26], ply 2 3 [-16, 22] / 24 [5, 44] (direct -40 / 11, -40 / 20, 320 games). Trees over F-P: forget 10 53 / 120 at ply 1, 20 / 80 at ply 2; forget 1 70 / 113, 17 / 56. Forget 10 never 20 below (the rule's trigger), level at ply 1 at the final, 24 above at ply 2 at the final (prediction within 15: misses on the upside at ply 2, one read). The probit capped tree's gain over its flat against strangers, 113-120 at ply 1 and 56-80 at ply 2, matches Panel 1 (section 1). |
| grow-p1b2 | ply 1 | grow-p1b replicated at seed 57 (6.71 (b): insurance on the leaf-bar ruling): F-P x2, grow1 (bar on) x2, A (--grow-mdl 0) x2, 12 rounds, self-games, every line on chessai-grow1 (leagues/specs/batch6/grow-p1b2.json). Prediction: A under grow1 by 20-60 at ply 1 at the final, within 20 at v150, ply 2 within 30 throughout. Rule: A 20+ below at the final confirms the bar; within 10 at every read = a seed term (the bar stays on) | 57 | plateau scaled, min 300, cap 400 | 11-12 06:40-07:45 | DONE at v349 (plateau). Sizes at the end: grow1 15 and 11 leaves (5 and 4 kept, 7-8 undone), A 27 and 23 (11 and 10 kept, none undone): as grow-p1b (25 vs 13-15). PANEL READ (grow-p1b2-read.sh; leagues/arenas/diag/panel-grow-p1b2-p1/-p2-v150/-v300/-v349; panel r + e + g + h at v300, 2,560 games an arm, ±19; r + e + g in brackets, the panel of grow-p1b's re-reads): A minus grow1, v150 / v300 / v349: ply 1 -19 (-10) / -1 (10) / -18 [-37, 1] (-9); ply 2 -1 (-13) / -20 [-39, -1] (-33) / 17 [-2, 36] (15); direct -29 / -22 / -14 and -1 / 8 / 56. Trees over F-P: grow1 47 / 45 / 37 at ply 1, -14 / -8 / -2 at ply 2; A 28 / 44 / 19, -15 / -28 / 15. Prediction (6.71: A under grow1 by 20-60 at ply 1 at the final; within 20 at v150; ply 2 within 30) half met: v150 and ply 2 hold, the final's -18 falls short of 20. Rule: not 20+ below at the final (the bar not confirmed), not within 10 at every read (not a pure seed term): in between; the bar stays on under either branch. Across the two seeds A minus grow1 at ply 1: -15, -56, -56 (grow-p1b) and -19, -1, -18 (this run): never ahead, mean about -28. |
| weight-p2 | ply 2 | Weighting training positions by the model's own uncertainty as the only factor (analyst 6.72 (c) 3, spec 6.73): a pure F-P field at ply 2, F-P x2 controls and F-P x2 with --weight-by-uncertainty score (s = P(W) + P(D)/2 from the pre-update prediction; u = 4 s(1-s) clipped to [0.2, 1]; w = u / running mean, memory 1M, cap 3; probit through the likelihood power). One training binary, chessai-weight1 (md5 9f6a7ae9; flag off = hybrid6's bytes), recipe row F-P-w; 1,584 games, self-games, hybrid17 games (leagues/specs/ply2/weight-p2.json). Predictions: weighted over controls 0-40 at ply 2, within 20 at ply 3; offline, worse in full loss by 0.005-0.02 and better on undecided positions. Rule: 30+ over = adopt for the league's flats; within 15 or 30+ below = the lever closes | 58 | plateau scaled, min 300, cap 400 | 11-12 08:23-09:03 | DONE at v299 (plateau at the floor). PANEL READ (weight-p2-read.sh; leagues/arenas/diag/panel-weight-p2-p1/-p2/-p3-v150/-v299; panel y + v + l + x4 + x4b at v299, 30 round-robins, 2,400 games an arm), weighted minus controls, v150 / v299: ply 2 -2 [-22, 18] / -16 [-36, 4]; ply 1 4 / -4; ply 3 24 [5, 44] / 8 [-11, 28] (direct -10 / 12, -3 / -25, 33 / -1; 240 games). SIDE READ (uloss.py on heldout-p2, 81,781 positions, control minus weighted per pair, game-clustered SE ~0.0014 full, ~0.0009 undecided): full loss -0.0005 / -0.0002 at v150, -0.0002 / -0.0002 at v299 (level; predicted 0.005-0.02 WORSE); on the control's undecided positions (69-70%) -0.0033 / -0.0028 at v150, -0.0019 / -0.0019 at v299 (better, 2-3.5 SE). THE WEIGHTING DOES WHAT IT SAYS OFFLINE (better on undecided positions at no full-loss cost) AND BUYS NOTHING IN PLAY: within 15 at ply 2 except the final's -16, nowhere near 30 up: by the rule the lever closes. Seed term: the controls (a third seed of the pure F-P field) minus fponly-p2's F-P at v299: 37 [17, 57] at ply 2, -18 at ply 1, -21 at ply 3 (fponly-p2b was -1 at ply 2): field seeds differ by up to ~40 at ply 2. |

- **Batch 1** (11-09 20:10; specs leagues/specs/batch1/). Stage 1 is the leaf classifiers alone (LR, probit, NB).
  - Two lines that the user hadn't asked for were removed and rerun: the search-label line in lr-settings and the LR
    replicate pair in leaf-sources.
  - From then on, an experiment has only the lines the user asked for.
- **Batch 2** (11-09 22:01-22:26; leagues/specs/batch2/).
  - Trees split once (--split-limit 1, threshold 1,000,000, 3 children). The error trees use --error log and no
    merges. The cluster trees merge (--revise 0).
  - The cluster-merge lines in single-split split by abs error, the binary's default. cluster-error pairs abs with
    log: its pairs were identical models at v45 (9 of 18,224 lines differ), since with one-hot labels abs and log
    give the same Best target. By cycle 131 the T-L-Lc pair and both T-P pairs had diverged; the T-L-Nc and T-L-Tc
    pairs still had identical leaf shares.
  - Tournament binary hybrid5 from cycles 112 / 123 / 40, train binary hybrid6 from 503 / 512 / 175. cluster-error
    has used hybrid7 since cycle 342, and the driver scan cache since 417.
- **Batch 3** (11-10; leagues/specs/batch3/). 9 rounds a cycle (2,430 local games), train binary hybrid6,
  tournament binary hybrid7.
  - split-growth (P7) is a 2×2 of the merge gate and the split limit, each tree one factor from single-split's
    winner, T-L-Nc-merge at limit 1.
  - split-growth-probit (P7b) asks whether growth helps error routing, which only probit leaves have made work. Its
    second pair is T-P-Ne, not no-gate trees.
  - error-targets (T1) changes one factor per line from T-L-Le, in turn: --route-before-train, --error-target gap
    (cap 4) and --split-bootstrap 1000000.
    - Lines 3-5 train on bin/chessai-target; lines 1-2 on hybrid6.
    - `leagues.py create` refuses those flags, because it checks them against the recipe's own binary. The setup
      used a wrapper and a binary swap before cycle 1 (models/from-scratch/target/league/NOTES.txt).
  - error-targets-probit is the same with probit leaves.
- **Batch 4** (from 11-10 17:41; leagues/specs/batch4/). The user (11-10 ~17:15): rerun every experiment so far,
  "even the unsuccessful ones", with no random players, no ply-1 entries, until the champions plateau, then rate the
  original's and the rerun's champions together in an arena with random players.
  - Engine options (leagues/FORMATS.md): `references` [] (ratings relative to the first line's untrained version,
    `rating_zero`; the tournament runs without --anchor), `ply1_entries` false, `plateau` {cycles 50, min_cycles
    100}, `cycles` 1000 as the cap. Swapped into leagues.py at 17:39 after a byte-identity check on a copy of
    small-field (absent keys: the same files as the old driver).
  - Each rerun keeps its original's lines, seeds, calibration, champion_se and prices; rounds per cycle match the
    original's games with a model at ply 2 per cycle at two ply-2 entries per line (lr-settings 5 -> 15 rounds,
    small-field 56 -> 204; NEXT-EXPERIMENTS.md, "Batch 4", has every row and the estimates). Binaries: tournament
    hybrid8, train hybrid6 (chessai-target, chessai-trigger and the search-label line kept), self-play as the
    originals.
  - One at a time in the originals' order (leagues/specs/batch4/queue.sh; leagues/experiments/batch4-queue.txt).
    nb-settings-no-random3 is covered by nb-settings-new (the same design without references); selfplay-depth is
    held.
  - Then one extra, small-field-x4 (the user, 11-10 ~18:00: "the same idea but with 4 of each all differently
    seeded"): small-field's two models, four replicates each (replicate r trains with seed + cycle + 100003 x (r - 1);
    a scratch run showed versions 1-3 differ in all four copies), seed 11, 10 rounds (2,400 games a cycle), started by
    a follow-on queue once the reruns are done. Not a rerun, so no pair arena; the combined arena doesn't include it.
- **Champion run** (cycles/champ/, 11-07 22:30 to 11-09 20:06, stopped after cycle 75; selfplay/league-champ/).
  - Champion rule and own steps.
  - Best: T-L-Ne-share3-c 1889.9 (cycle 71, v394), partly luck.
  - F-L-c's new champion at cycle 51 (v4827, 1830) was its first since v4100.
  - The studies in sections 3-6 come from it: twins, whytrees, degeneracy, league-data share and solo lineages.
  - Records: cycles/champ/README.md and the cycle reports.
- **First run** (cycles/, to cycle 73, 11-07). Records: cycles/README.md.
- **2026-11-05/06 runs and tournaments:** section 11.

---

## 8. Arenas

- **2026-11-10 19:49, nb-settings pairs, and their ply-1 replays at 19:50-19:51** (leagues/arenas/pairs/nb-settings
  and nb-settings-no-random3; leagues/arenas/ply1/, the same names; 51 s of CPU per replay).
  - nb-settings-new (plateau stop at 100, last crowning 39) stands in for both originals: nb-settings (stopped at 265)
    and nb-settings-no-random3 (stopped at 126). 56 double round robins each, with the four random players.
  - Rerun minus original, Elo:

    | Line | nb-settings, ply 2 [95%] | ply 1 [95%] | nb-settings-no-random3, ply 2 [95%] | ply 1 [95%] |
    |---|---|---|---|---|
    | alpha 0.1 | 94 [62, 125] | 51 [21, 81] | 80 [49, 111] | 8 [-22, 38] |
    | alpha 1 | 107 [75, 138] | 85 [55, 115] | 72 [40, 103] | 60 [30, 90] |
    | alpha 10 | 81 [50, 112] | 71 [41, 101] | 22 [-10, 53] | 2 [-28, 32] |
    | mean | 94 [76, 112] | 69 [51, 86] | 58 [40, 76] | 23 [6, 41] |

  - Champions: NB doesn't bootstrap, so they are first versions: the originals' v1-v7, the reruns' v1, v1 and v38.
    The gain comes from the first cycles' games: all ply 2 against ply 2 in the rerun, with random players and ply-1
    entries in the originals.
  - Arena Elo: at ply 2 the reruns 402-413, the originals 306-380, random-3 294-302. At ply 1 every NB model
    (204-303) is below random-3 (380-382).
  - Reading: NB keeps most of its gain at ply 1 (69 of 94, 23 of 58), where LR keeps 36 of 424 and probit 111 of
    754. That fits NB's small ply gain (section 6): it barely specialises to the ply it trains at.
- **2026-11-10 19:38, probit-settings pair, and its ply-1 replay at 19:39** (leagues/arenas/pairs/probit-settings,
  leagues/arenas/ply1/probit-settings; the ply-1 watcher ran the replay, 43 s of CPU).
  - Each line's final champion from probit-settings (old field, 500 cycles) and from probit-settings-new (ply-2
    entries only, no random players, plateau stop at 256). 46 double round robins with the four random players.
  - Rerun minus original, Elo:

    | Line | at ply 2 [95%] | at ply 1 [95%] |
    |---|---|---|
    | tau 0 | 755 [666, 844] | 100 [65, 136] |
    | tau 1e-6 | 768 [679, 857] | 126 [90, 161] |
    | tau 1e-5 | 766 [677, 855] | 129 [94, 165] |
    | tau 1e-4 | 727 [639, 815] | 89 [54, 124] |
    | mean | 754 [676, 832] | 111 [93, 129] |

  - Champions: originals v357-375, reruns v151-205, so the reruns had under half the cycles. The rerun's cycles have
    16 rounds of ~56 games, all ply 2 against ply 2; the original's had 5 rounds with random players and ply-1
    entries.
  - Arena Elo at ply 1: the models 665-836, random-3 342. At ply 2 (pair report) the models are 1072-1882.
  - Reading:
    - The lr-settings pattern again, larger: 754 at ply 2 and 111 at ply 1 (about 220 on the ply-2 scale). Probit's
      ply-1 gain is clear, LR's (36) small.
    - tau 1e-4 is last in all four columns (both runs, both plies), by 15-63. tau 0, 1e-6 and 1e-5 don't separate.
- **2026-11-10 18:43, lr-settings pair, and its ply-1 replay at 18:48** (leagues/arenas/pairs/lr-settings,
  leagues/arenas/ply1/lr-settings; spec leagues/specs/arenas/ply1-lr-settings.json).
  - The first batch-4 pair: each line's final champion from lr-settings (old field, stopped at 172) and from
    lr-settings-new (ply-2 entries only, no random players, plateau stop at 194). 39 double round robins with the
    four random players, ~1,000 games a champion. The ply-1 replay is the same arena with every entry at ply 1
    (71 s of CPU each).
  - Rerun minus original, Elo:

    | Line | at ply 2 [95%] | at ply 1 |
    |---|---|---|
    | rate 0.0125 | 516 [465, 566] | 20 |
    | rate 0.025 | 474 [425, 523] | 86 |
    | rate 0.05 | 405 [357, 452] | 35 |
    | rate 0.1 | 423 [375, 471] | 58 |
    | label line (depth-2 search labels) | 301 [256, 346] | -21 |
    | mean | 424 [396, 452] | 36 |

  - Champions: originals v124-167, reruns v129-143, so about the same number of cycles. The rerun's cycles have 15
    rounds of 90 games, all ply 2 against ply 2. The original's had 5 rounds with random players and ply-1 entries.
  - Arena Elo at ply 1: the models 638-738, random-3 304. At ply 2 (pair report) the models are 975-1517.
  - Reading:
    - Training only on ply-2 games made far better ply-2 players: 424, about ten times the earlier field's effects.
      At ply 1 the gain is 36, or roughly 70 on the ply-2 scale given the 0.5 slope.
    - Most of the rerun's gain is specialisation to ply-2 play (section 6), not a better evaluator in general.
    - Each pair crosses other changes too: twice the games a cycle, a different stop rule, and the label line's
      influence on the field.
- **2026-11-10 17:27, a strong player in the field lifts every line** (the batch-4 pair-arena test;
  leagues/arenas/pairs/test-lr-settings-2-vs-lr-settings/).
  - Each line's final champion from lr-settings (stopped at 172; its field included the search-label line) against
    lr-settings-2's (500 cycles, no label line). Four random players were in the field. Ply 2, own calibration,
    about 1,000 games per champion.
  - lr-settings' F-L champions (v124-167) beat lr-settings-2's (v290-443) by 117 [98, 136] on average, and by 102-131
    on each of the four rate lines. The label line was the strongest player, at 1363. Arena Elo of the references:
    random 0, random-1 149, random-2 262, random-3 422.
  - Reading: training on games with a much stronger player in them was worth about 100 Elo to every line, with fewer
    cycles. This fits the user's point (11-10) that opponent quality matters.
  - Caveat: one pair of runs, two different fields, not a planned comparison. The four lines share their field, so
    they are not independent replicates.

- **2026-11-10, matched-depth** (leagues/arenas/matched-depth; spec leagues/specs/arenas/matched-depth.json; models
  in models/from-scratch/matched-depth/ with a README).
  - The question: how do the depth-4 self-play models (selfplay-depth's final versions) do against flat LR and probit
    models of the other depths trained on about as many positions?
  - Setup: 400 round robins, every entry ci95 9-10, Elo relative to random. Cost: 1,524 CPU s and 18 min of wall.
  - **Matched positions are not matched rounds.** LR d4 had 836 rounds of 10 games and probit d4 368. The matched LR
    models had 5-6 (d0), 14 (d1) and 86-87 (d2) rounds; the probit ones 2-3, 7 and 49.
  - **Ratings:**

    | Model | Elo | Positions | Of target |
    |---|---|---|---|
    | LR d4 @99 (target) | 1447 | 1.65M | 1.00 |
    | LR d1, round 14 | 1102 | 1.68M | 1.02 |
    | LR d2 #1 @2 | 1000 | 1.61M | 0.98 |
    | LR d2 #2 @2 | 947 | 1.60M | 0.97 |
    | LR d0, round 5 | 811 | 1.51M | 0.92 |
    | LR d0, round 6 | 769 | 1.80M | 1.09 |
    | LR d1 @1 (context) | 1364 | 7.4M | 4.5× |
    | LR d0 @1 (context) | 1201 | 14.3M | 8.7× |
    | Probit d4 @99 (target) | 1335 | 0.79M | 1.00 |
    | Probit d1, round 7 | 1094 | 0.79M | 1.00 |
    | Probit d2 @2 | 940 | 0.85M | 1.08 |
    | Probit d0, round 3 | 832 | 0.91M | 1.16 |
    | Probit d0, round 2 | 719 | 0.64M | 0.81 |
    | Probit d1 @1 (context) | 1411 | 5.9M | 7.5× |
    | Probit d0 @1 (context) | 1234 | 13.3M | 16.9× |

    References: random-3 416, random-2 231, random-1 125.
  - **Direct games** (800 a pair; W-L-D, score, Elo with a 95% Wilson interval; `arena.py status --vs` reproduces
    them):

    | Pair | W-L-D | Score | Elo |
    |---|---|---|---|
    | LR d4 against LR d0 round 5 | 752-44-4 | 94.2% | 486 [434, 537] |
    | LR d4 against LR d0 round 6 | 784-12-4 | 98.2% | 700 [609, 790] |
    | LR d4 against LR d1 round 14 | 679-90-31 | 86.8% | 327 [292, 363] |
    | LR d4 against LR d2 #1 @2 | 750-26-24 | 95.2% | 521 [465, 577] |
    | LR d4 against LR d2 #2 @2 | 775-11-14 | 97.8% | 655 [575, 736] |
    | LR d4 against LR d0 @1 | | 82.2% | 266 [234, 297] |
    | LR d4 against LR d1 @1 | | 67.2% | 124 [99, 150] |
    | LR d4 against probit d4 | | 61.0% | 78 [53, 102] |
    | Probit d4 against probit d0 round 2 | 775-14-11 | 97.6% | 641 [564, 718] |
    | Probit d4 against probit d0 round 3 | 750-30-20 | 95.0% | 512 [456, 567] |
    | Probit d4 against probit d1 round 7 | 607-166-27 | 77.6% | 215 [187, 244] |
    | Probit d4 against probit d2 @2 | 684-93-23 | 86.9% | 329 [294, 365] |
    | Probit d4 against probit d0 @1 | | 69.1% | 140 [114, 166] |
    | Probit d4 against probit d1 @1 | | 53.6% | 25 [1, 49] |
  - **Reading:**
    - At matched positions the order is d4, d1, d2, d0 for both LR and probit.
    - LR d4 also beats LR models trained on 4.5× and 8.7× more positions.
    - Probit d1 @1 (7.5× the positions) is rated 76 above probit d4, yet loses their direct games (53.6%). So the
      ratings aren't fully transitive here.
  - **Arena Elo is on its own scale.** The 9 experiment versions rate 50-137 above their selfplay-depth ratings.
    Their calibrations reproduced identically.
  - The arena kind: leagues/ARENA.md and leagues/arena.py.

**The dip arena** (11-11 00:19-00:21, leagues/arenas/diag/dip; the analyst's diagnostic (a), theory-2 section 6.2).
residual-leaves' copy tree (T-L-Nc limit 4, no merges) and its F-L at the same versions, during the post-split dip
(v60-150) and late (v300-399), every entry at ply 1 and at ply 2, own calibration (all 14 identical to their
experiment's), the four random references, 15 double round-robins of 32 players, ~1 min wall. Errors from the fit's
covariance (contrasts.py in the arena folder).

| Tree minus F-L | ply 1 [95%] | ply 2 [95%] |
|---|---|---|
| v60 | -5 ±45 | -24 ±51 |
| v90 | -6 ±45 | -50 ±50 |
| v120 | 14 ±45 | -84 ±48 |
| v150 | 39 ±45 | 1 ±49 |
| v300 | 65 ±45 | 88 ±49 |
| v350 | 38 ±45 | 27 ±50 |
| v399 | 83 ±45 | 75 ±49 |
| dip mean (v60-150) | 10 ±23 | -39 ±25 |
| late mean (v300-399) | 62 ±26 | 63 ±29 |

- **The dip is a ply-2 effect.** At the dip versions the tree is level with F-L at ply 1 and 39 behind at ply 2:
  the ply-2 gap minus the ply-1 gap is -50 ±33. Late, the two plies agree (2 ±39).
- At ply 1, F-L gets worse with version (737 at v60 to 625 at v399, random = 0) while the tree holds 708-746: the
  late ply-1 lead is mostly flat's decline. At ply 2 both improve (tree 1353 to 1631, F-L 1377 to 1556).
- The late tree leads by 63 ±29 at ply 2 here, against ~6 in residual-leaves' own league (last quarter; another
  field).
- **The seam statistic doesn't track the dip** (models/from-scratch/optimism/seams.py on the arena's saved games,
  reference flat@399-2; seams.out in the arena folder; ± bootstrap SEs). Seam = how much more the tree overrates
  its own side after a leaf crossing than elsewhere, net of the reference (negative = overrates):
  - v60 -0.012 ±0.006 (E to move) / -0.019 ±0.006 (opponent to move); v90 0.004 / -0.001 ±0.004; v120 0.012 /
    0.013 ±0.004; v150 0.004 / -0.001 ±0.003; v300-399 0.003 to 0.013 ±0.002.
  - The largest ply-2 deficit (v120) has a positive seam, like the late versions. Crossings are rarer in the dip
    (3.6-3.8% of positions at v60-120, 6.6% at v150, 11% late).
  - So the analyst's first-ranked cause (across-leaf comparisons in the minimum over replies) is not supported here.
  - The tool's own calibration slope for the tree on its ply-2 positions rises with version (0.60, 0.86, 1.04, 1.15,
    1.60, 1.66, 1.61; the reference's 1.9-2.5 throughout): the young tree is overconfident on them. Sent to the
    analyst as a possible lead.

**Top models across leagues at ply 2** (11-11 00:20-00:24, leagues/arenas/top-champions-1; the user's request: the
old champion league's top 5 champions, the top 5 of the leaf-sources pair arena, and the top 1 of the lr-settings and
probit-settings pair arenas, all at ply 2). Each model calibrated as its source was (all 12 identical); the four
random references; 200 round-robins (the cap: the ±22 includes the anchor's uncertainty, so the 20 stop never
held); 392 CPU-s.

| # | Model | From | Elo | 95% |
|---|---|---|---|---|
| 1 | F-L+lr-rate=0.0125.loc@136 | lr-settings-new (pair #1) | 1581 | ±23 |
| 2 | T-L-Te-merge-c@999 | old champion league #3 | 1535 | ±22 |
| 3 | T-L-Nc-share-c@545 | old champion league #5 | 1520 | ±22 |
| 4 | F-P+probit-tau=1e-5.loc@151 | probit-settings-new (pair #1) | 1516 | ±22 |
| 5 | T-L-Ne-share3-c@394 | old champion league #1 | 1513 | ±22 |
| 6 | T-L-Ne-fresh-c@1720 | old champion league #2 | 1506 | ±22 |
| 7 | T-L-Ne-c@2101 | old champion league #4 | 1492 | ±22 |
| 8 | F-L#1.loc@86 | leaf-sources | 1403 | ±22 |
| 9 | F-L#2.loc@93 | leaf-sources | 1397 | ±22 |
| 10 | F-L#1.loc@105 | leaf-sources-new | 1362 | ±22 |
| 11 | F-L#2.loc@144 | leaf-sources-new | 1353 | ±22 |
| 12 | F-P.loc@43 | leaf-sources-new | 1339 | ±22 |

- Contrasts (95%, from the fit's covariance): the lr-settings model over the best champion 46 [31, 60], over the
  champions' mean 68 [56, 79], over the probit-settings model 65 [50, 80]; the probit-settings model against the
  champions' mean 2 [-9, 14]; champions' mean over the leaf-sources mean 143 [136, 149]; leaf-sources originals over
  their reruns (F-L) 42 [32, 53].
- Direct games (400 a pair): the lr-settings model beat every champion, 55% (against Te-merge, 210-171-19) to 62%
  (Ne-c, 234-137-29). No leaf-sources model beat a champion.
- **The strongest ply-2 player we have is a flat LR** trained in a ply-2-only field with learning rate 0.0125. It
  beats the old league's best routed trees, which trained in mixed fields (random players, ply-1 entries).
- The lr-settings model is 223 [210, 237] above the leaf-sources-new F-L lines, also flat LR from ply-2-only fields.
  The two differ in learning rate (0.0125 against the default) but also in experiment, field and data, so the rate
  is a candidate, not a finding; the lr-settings pair arena found lower rates better at every step.

**Play temperature changes the verdicts** (11-11 00:26-00:33; arenas diag/dip-cal2, diag/dip-cold, old-growth,
old-growth-cold, top-champions-1-cold). Every league and arena so far rated models with a log-odds softmax at T 0.1
on values calibrated on depth-1 own games. A model's effective temperature is then T over its calibrated scale at
the ply it plays (the analyst, theory-2 6.8), so two models with the same move ranking play with different
randomness. Near-argmax play (T 0.02) removes most of that, and alpha-beta plays argmax, so the cold measurement is
the one that bears on the goal. Colder play also stretches the Elo scale: compare orderings and signs across
temperatures, not sizes. Game diversity stays ~92% distinct per pairing at T 0.02.
- **The dip twins** (residual-leaves' copy tree minus F-L, group means):

  | Setup | dip ply 1 | dip ply 2 | late ply 1 | late ply 2 |
  |---|---|---|---|---|
  | standard (depth-1 cal, T 0.1) | 10 ±23 | -39 ±25 | 62 ±26 | 63 ±29 |
  | depth-2 own cal, T 0.1 | -2 ±23 | -60 ±25 | -9 ±26 | -29 ±29 |
  | depth-1 cal, T 0.02 | -5 ±22 | -39 ±25 | 1 ±26 | -42 ±30 |

  - The dip survives both: the young tree's ply-2 play is genuinely worse. Seams, calibration and policy are
    excluded; the analyst's remaining candidate is positions a ply-2 search scores that never occur in games.
  - The late tree's lead exists only in the standard setup: it was the calibration's effect on play randomness.
  - The cal2 twin needs file entries: arena.py gives version entries their experiment's calibration.
- **The old champion lineages' league growth was real** (old-growth: versions spanning each lineage's growth in the
  old champion league, the top flat LR as a yardstick, ply 2, 60 round-robins):

  | Contrast | league | T 0.1 | T 0.02 |
  |---|---|---|---|
  | T-L-Ne-c v2101 - v1428 | 235 | 163 ±26 | 244 ±28 |
  | T-L-Te-merge-c v999 - v460 | 147 | 127 ±25 | 149 ±26 |
  | T-L-Ne-share3-c v394 - v152 | 140 | 137 ±25 | 192 ±26 |
  | yardstick - T-L-Ne-c v2101 | | 60 ±25 | 104 ±26 |
  | yardstick - T-L-Te-merge-c v999 | | 33 ±25 | 86 ±27 |
  | yardstick - T-L-Ne-share3-c v394 | | 31 ±26 | 48 ±27 |

  Every later version beats every earlier one of its lineage at both temperatures. The user's question (11-11
  ~00:35): the old champions trained on ~10x the games of the new models, yet trail them although their league Elo
  kept growing. Answer: the growth was real; the new recipe is far more efficient per game.
- **top-champions-1 at T 0.02** (100 round-robins): the flat LR leads the best champion by 96 ±21 (46 ±15 at T 0.1)
  and the champions' mean by 118 ±17 (68 ±12). The flat probit moves from level with the champions (2 ±11) to 55 ±16
  above them, second overall. The champions reorder: T-L-Te-merge-c@999, best at T 0.1, is last of them cold. The
  leaf-sources F-L reruns draw level with or pass their originals (1518 and 1485 against 1483 and 1480). Champions
  over leaf-sources: 107 ±9 (143 ±7).
- So under near-argmax play, both tuned flat leaves from ply-2-only fields beat every old routed champion.
- **No routing gain survives near-argmax play** (11-11 00:33-00:35; diag/cold-trees at T 0.02 and its twin
  diag/warm-trees at T 0.1, the only difference; the analyst's plan, theory-2 6.8). The final versions of every flat
  line and its trees from single-split, single-split-probit, split-growth, split-growth-probit and small-field, at
  ply 1 and 2, own depth-1 calibrations, 15 double round-robins of 36 players each. Each tree minus its own
  experiment's flat line (contrasts.py in cold-trees/):

  | Tree minus own flat | ply 1 T 0.1 | ply 1 T 0.02 | ply 2 T 0.1 | ply 2 T 0.02 |
  |---|---|---|---|---|
  | single-split T-L-Nc-merge | 23 ±40 | 36 ±37 | -18 ±40 | -39 ±43 |
  | single-split T-L-Tc-merge | 67 ±39 | 17 ±37 | 49 ±41 | -41 ±43 |
  | single-split T-L-Ne (control) | 11 ±40 | -6 ±37 | -50 ±40 | -71 ±42 |
  | single-split-probit T-P-Nc-merge | 34 ±38 | -23 ±38 | 73 ±40 | 2 ±38 |
  | single-split-probit T-P-Tc-merge | 19 ±38 | -33 ±38 | 92 ±40 | 9 ±38 |
  | single-split-probit T-P-Ne | 22 ±38 | -34 ±38 | 72 ±40 | -2 ±38 |
  | split-growth T-L-Nc-merge limit 1 | 3 ±38 | 30 ±37 | 3 ±39 | 3 ±38 |
  | split-growth T-L-Nc-merge limit 4 | -9 ±38 | 5 ±37 | -29 ±39 | 6 ±38 |
  | split-growth-probit T-P-Nc-merge limit 1 | 6 ±38 | -39 ±38 | 33 ±40 | 30 ±38 |
  | split-growth-probit T-P-Ne limit 1 | 12 ±38 | -27 ±38 | 38 ±40 | 7 ±38 |
  | small-field T-L-Tc-merge | 38 ±39 | 5 ±38 | 17 ±39 | -16 ±39 |
  | **5 LR trees that gained in their leagues** | 24 ±21 | 18 ±20 | 4 ±21 | -17 ±21 |
  | **5 probit trees** | 18 ±23 | -31 ±23 | 62 ±24 | 9 ±23 |

  - The probit trees' ply-2 gain reproduces at T 0.1 in this field (62 ±24; 44-77 in their leagues) and is gone at
    T 0.02 (9 ±23). The LR trees' is gone at both here (4 and -17).
  - At 95%, a ply-2 routing gain under near-argmax play is at most ~4 for these LR trees and ~32 for these probit
    trees (one final version each; version noise ~17 SD).
  - So every routing gain measured to date was the play-temperature confound: a tree's calibrated scale at ply 2
    differs from its flat line's, and at T 0.1 that sets how randomly each plays. For an alpha-beta evaluator
    (argmax, calibration-free) no routed model has yet beaten its flat leaf.
- **Residual leaves don't rescue routing** (diag/residual-cold, ply 2, T 0.02, residual-leaves' F-L, copy tree and
  residual tree at v150/300/399; the analyst's last routing test): residual minus copy 18 ±36, residual minus flat
  -45 ±36, copy minus flat -63 ±36. By the analyst's criterion (residual must beat copy cold) routing has no case
  left at ply 2; parking it is the user's call.
- **The ply-2-only field's gain is robust** (diag/pair-lr-settings-cold, the lr-settings pair arena replayed at
  T 0.02): rerun minus original 542 ±38 (424 at T 0.1); per line 471-643.
- **The learning-rate effect was the confound too.** Rerun lines minus the 0.0125 line, ply 2:

  | Line | T 0.1 | T 0.02 |
  |---|---|---|
  | lr-rate 0.025 | -29 ±44 | 23 ±45 |
  | lr-rate 0.05 (the default) | -74 ±44 | -18 ±45 |
  | lr-rate 0.1 | -118 ±44 | -8 ±45 |
  | label-depth 2, label-result 0.5 | -41 ±44 | 48 ±46 |

  Near argmax the rates 0.0125-0.1 are level. So the flat LR's lead over leaf-sources-new's F-L lines (~200 cold in
  top-champions-1-cold; both ply-2-only fields, and those lines use the default 0.05) isn't the rate. The candidate
  is the field again: lr-settings-new's field was five F-L lines, leaf-sources-new's included weak NB lines and
  self-play lines whose games every local line trained on. Cross-experiment, so confounded.
- **First ratings at ply 3** (11-11 00:41-00:47, diag/ply3-cold; the analyst's "goal's own measurement"): the best
  flat evaluators, the old champion run's depth-4-trained flat LR and its plain flat LR (their cycle-75 champions),
  and the FM champion, each at ply 2 and ply 3, T 0.02, own depth-1 calibrations, 40 double round-robins (~13
  CPU-min). Minus the lr-settings-new F-L lr-rate 0.0125 champion (v136):

  | Model | ply 2 | ply 3 | ply 3 - ply 2 |
  |---|---|---|---|
  | old F-L-d4mix-c v426 (depth-4 self-play, 50/50 search labels) | -46 ±35 | 63 ±38 | 109 ±52 |
  | lr-settings-new F-L 0.0125 v193 (final) | 24 ±34 | 7 ±37 | -17 ±51 |
  | lr-settings-new F-L 0.05 v193 | -9 ±35 | -28 ±37 | -18 ±50 |
  | old F-L-c v4827 (old league's flat LR) | -66 ±35 | -36 ±37 | 31 ±51 |
  | leaf-sources-new F-L#1 v105 | -147 ±36 | -76 ±37 | 70 ±52 |
  | probit-settings-new F-P tau 1e-5 v151 | -32 ±35 | -177 ±37 | -146 ±50 |
  | old FM-c v669 (factorization machine) | -340 ±44 | -307 ±38 | 32 ±58 |

  - Ply 3 is ~480-610 Elo above ply 2 for the LR models, 354 for probit.
  - **The training ply carries to ply 3:** the depth-4-trained LR trails at ply 2 and leads at ply 3 (a 109 ±52
    swing), as "models specialise to the ply they train at" predicts. Deeper training data is a lever for the
    goal.
  - **Probit falls behind LR with depth:** level-ish at ply 2 (-32), 177 behind at ply 3 (and ~2.4x the cost per
    node). LR is the leaf for deeper search.
  - FM (an old mixed-field model) is far behind at both plies; not a fair test of pairwise features.
  - Learning rate 0.0125 against 0.05: 9 ±35 at ply 2, 28 ±37 at ply 3 (no rate effect cold, as in the pair).
- **Search labels on ply-2 games don't carry to ply 3** (diag/ply3-labels-cold, ply 3 only, T 0.02, 200
  round-robins; every model beat every random player, so the anchored fit is empty: bt.py in the arena folder fits
  the 2,400 games among the four models). Minus lr-settings-new's label line (--label-depth 2 --label-result 0.5,
  v193): old F-L-d4mix-c v426 92 ±25; result labels at 0.05 37 ±24; at 0.0125 37 ±24. The label line led by 48 ±46
  at ply 2 (pair-lr-settings-cold). So the analyst's labels-p2-cold isn't the first experiment; deeper games are.
- **Self-play depth at ply 2 and 3** (diag/selfplay-depth-ply3-cold, T 0.02, 20 round-robins; selfplay-depth's LR
  self-play champions, each trained only on its own games). Minus the depth-2 line #1 (v80):

  | Model | ply 2 | ply 3 |
  |---|---|---|
  | self-play depth 0 (v78) | -188 ±56 | -52 ±54 |
  | self-play depth 1 (v95) | -398 ±68 | -152 ±54 |
  | self-play depth 2 #2 (v49) | -15 ±51 | -37 ±54 |
  | self-play depth 4 (v76) | -413 ±69 | -376 ±57 |
  | old F-L-d4mix-c v426 | -14 ±51 | 113 ±58 |
  | lr-settings-new F-L 0.0125 v136 | 52 ±51 | 36 ±55 |

  - selfplay-depth's depth-4 line is poor at both plies: depth-4 games cost ~70x, and on the experiment's CPU
    budget it had very little data by v76. The old depth-4 lineage, trained far longer, is the best at ply 3.
  - Gaps shrink at ply 3: search covers for a weaker evaluator.
- **probit-widen-p1b, read cold: the first routing gain that survives T 0.02** (11-11 00:55-00:58). The run (ply 1,
  pure ply-1 field, F-P x2 and T-P-Nc-merge limit 4 with the fresh merge gate at --split-noise 0 / 0.03 / 0.1, 2
  seeds each) ended at cycle 200 on the plateau rule. Its own T 0.1 tables (current against current, against
  F-P#1, cycles 151-200, ~±20): F-P#2 2; noise 0 4 and -2; noise 0.03 15 and 19; noise 0.1 -15 and -20.
  T 0.02 arenas at ply 1 and 2 (15 round-robins; contrasts.py in each folder), arm means minus the two F-P seeds:

  | Arm | final versions v199, ply 1 | ply 2 | champions, ply 1 | ply 2 |
  |---|---|---|---|---|
  | noise 0 (fresh gate) | 104 ±42 | -21 ±43 | 90 ±41 | 33 ±43 |
  | noise 0.03 | 81 ±41 | 36 ±43 | 95 ±41 | 91 ±43 |
  | noise 0.1 | -2 ±41 | -10 ±43 | -4 ±41 | -27 ±42 |
  | F-P#2 - F-P#1 | 13 ±58 | -37 ±60 | 4 ±58 | 63 ±60 |

  - At the ply they trained at, the fresh-gate probit trees beat flat probit by 80-100 near argmax, same-age
    versions included. T 0.1 had them level: the confound can hide a gain as well as fake one.
  - It doesn't carry to ply 2 (models specialise to their ply). Widening by 0.1 removes it.
  - Contrast with cold-trees' probit trees (-31 ±23 at ply 1 cold): those trained at ply 2 in mixed fields, limit 1,
    with stream-gate merges. Which difference matters is open (sent to the analyst).
- **probit-widen-p1c replicates p1b: fresh-gate probit trees gain ~95 at ply 1 near argmax** (11-11 02:02-02:05).
  p1c is p1b with seed 18 (ended at cycle 251 on the plateau rule). Both final-version reads at T 0.02, 75
  round-robins each (p1b's extended from 15 at 01:37; diag/cold-final-probit-widen-p1b and -p1c, contrasts.py in each).
  Arm means minus the two F-P seeds:

  | Arm | p1b ply 1 | p1c ply 1 | p1b ply 2 | p1c ply 2 |
  |---|---|---|---|---|
  | noise 0 (fresh gate) | 98 ±19 | 95 ±19 | -39 ±19 | 14 ±19 |
  | noise 0.03 | 87 ±18 | 94 ±19 | 34 ±19 | 49 ±20 |
  | noise 0.1 | 7 ±18 | -11 ±19 | -23 ±19 | -147 ±20 |
  | F-P#2 - F-P#1 | 4 ±26 | -5 ±26 | -13 ±27 | 9 ±27 |

  Per line (each ±~26), minus the F-P mean:
  - At ply 1: the eight fresh-gate trees at noise 0 or 0.03 are 84 to 104. The noise-0.1 trees are -16 to 7.
  - At ply 2: the noise-0.03 trees are 51, 17, 43 and 55. The noise-0 trees are -36 and -42 in p1b, 20 and 8 in p1c.
    The noise-0.1 trees are -7 and -39 in p1b, -149 and -144 in p1c.

  - **The ply-1 gain is real and reproducible:** ~95 near argmax with both seeds, at noise 0 and 0.03. Noise 0.1
    removes it.
  - **Carry to ply 2 depends on the arm.** Noise 0.03 gains ~40 at ply 2 in both seeds (all four lines positive).
    Noise 0 changes sign between seeds. So "a ply-1 tree loses one ply deeper" (lr-trees-p1's LR trees, -43) is not
    general.
  - The 15-round table above (p1b) had ±42; the 75-round numbers replace it.
- **The late gain is the trees': F-P stood still from v226 to v314 while the error trees gained 78 at ply 1** (11-11
  11:50-11:52; diag/g2-versions-cold).
  - One field of G2's (error-routing-p1, ply-1 field) F-P x2 and T-P-Ne x2 at v226 and v314. T 0.02, hybrid13,
    plies 1-2, 75 round-robins (18,000 games among entries).

  | Contrast (T 0.02; ± is the 95% interval) | ply 1 | ply 2 |
  |---|---|---|
  | F-P v314 minus F-P v226 | 3 ±19 | 4 ±19 |
  | T-P-Ne v314 minus T-P-Ne v226 | 78 ±19 | 24 ±19 |
  | T-P-Ne v226 minus F-P v226 | 20 ±19 | 58 ±19 |
  | T-P-Ne v314 minus F-P v314 | 94 ±19 | 77 ±19 |

  - Per entry minus the F-P v226 mean, ply 1: error trees v314 108 and 87, v226 27 and 13; F-P v314 14 and -7.
  - So the flat had plateaued by v226 and the trees went on learning for 90 more cycles (18M more positions). The
    tree's edge needs data and time the flat doesn't; probit-confirm-p2's plateau rule (floor 200; crownings judged
    in the league's T 0.1 games, which barely see a tree's ranking edge) stopped it at v241, inside that window.
  - Follow-up launched 11:53: probit-late-p2 (section 7).

- **Equal-data read of G2: at v226 the ply-1-field trees were NOT ahead at ply 1; their 106-118 formed later**
  (11-11 11:48-11:50; diag/error-routing-p1-v226-cold; theory-2 6.30-6.31's data-vs-field test).
  - G2 (error-routing-p1, ply-1 field) at v226, the version whose lines had trained 46.46M positions, as many as
    probit-confirm-p2's finals v241 (cycles.csv: ~205k positions a cycle at ply 1, ~180k at ply 2, 1,728 games
    each; both runs' error trees had 7 leaves by v49 and 9 by ~v99). F-P x2, T-P-Ne x2, T-P-Nc-merge x2; T 0.02,
    hybrid13, plies 1-2, 75 round-robins (9,900 games among entries).

  | Contrast (T 0.02; ± is the 95% interval) | ply 1 at v226 | ply 2 at v226 | ply 1 at v314 (G2 read) | ply 2 at v314 |
  |---|---|---|---|---|
  | Error trees (T-P-Ne) minus F-P | 14 ±22 | 63 ±23 | 118 ±23 | 77 ±22 |
  | Cluster trees (T-P-Nc-merge) minus F-P | -4 ±22 | -6 ±22 | 106 ±23 | 62 ±22 |

  - Per line minus the F-P mean at v226, ply 1 / ply 2: error 25 / 76 and 3 / 50; cluster 7 / 21 and -16 / -32.
  - All four tree lines moved together between v226 and v314 at ply 1 (from about 0 to about 100). Which side moved,
    the trees up or both F-P lines down, is read in diag/g2-versions-cold (running 11:52).
  - The analyst (6.31): both mechanisms. The data/time term is real (the ply-1 gain formed late, and
    probit-confirm-p2's plateau stopped it at v241, before any such late gain); the field's play is real too (at
    ply 2 the ply-1-field error tree already led its flat by 63 at v226, where the ply-2-field tree with the same
    data and 9 leaves held leads by nothing).

- **error-growth-p1: the error tree grows to 17 leaves at limit 8 but gains nothing over 9; limit 4 replicates G2
  (84 / 64)** (11-11 12:11-12:14; theory-2 6.28's growth test).
  - Run: error-growth-p1, seed 37, ply-1 field, F-P x2, T-P-Ne (NB error router, non-merging, 1M, noise 0.03) at
    split limit 4 x2 and 8 x2; done at 450 (the floor, raised from 300 at cycle 28; plateau).
  - Read: diag/error-growth-p1-cold, final versions v449, T 0.02, hybrid13, plies 1-2, 75 round-robins (9,900
    games among entries).

  | Contrast (T 0.02; ± is the 95% interval) | ply 1 | ply 2 |
  |---|---|---|
  | Limit 4 (9 leaves) minus F-P | 84 ±22 | 64 ±22 |
  | Limit 8 (17 leaves) minus F-P | 59 ±22 | 67 ±22 |
  | Limit 4 minus limit 8 | 25 ±22 | -3 ±22 |

  - Per line minus the F-P mean, ply 1 / ply 2: limit 4 98 / 70 and 70 / 58; limit 8 67 / 64 and 52 / 70.
  - Leaves every 25 versions, both seeds alike: limit 4 3, 7, 9 by v74, then 9; limit 8 3, 7, 9, 11 at v99, 17 by
    v124, then 17. So the error tree does expand (17 leaves, every split kept), and the third level forms early.
  - The analyst's prediction (6.28, revised): limit 8 minus limit 4 between -20 and 20 at ply 1, centred near 0, a
    late gain of 10-30 plausible past 400 cycles; a loss beyond 30 = the third level fragments faster than it
    learns. Read: -25 ±22 at ply 1, 3 at ply 2: no gain from the extra 8 leaves after 325 cycles at 17, a small
    loss at ply 1 within the interval.
  - The error tree's gain replicates in a second field: 84 / 64 here against 118 / 77 in G2.
  - Are the extra leaves distinct? routecells on each limit-8 final (v449), 4000 games against its own F-P at depth
    1, T 0.1 (models/from-scratch/unified/errtrees-l8/cells1-2.txt; the morning's method for G2's limit-4 trees):
    seed 1 duplicates (within 2 SE, as they are or on slopes) under 4 of its 8 routers, seed 2 under 7 of 9 (every
    sibling pair under routers 4, 5, 12; the root's pair 1-3 in both seeds). G2's limit-4 error trees had one
    duplicate pair in one seed and none in the other. Re-routing headroom: G cells -0.0022 to 0.0021 per decisive
    position (nil, as before). So past 9 leaves the error splits mostly make copies: on these features there is
    little left for a third level to separate (the analyst's 6.25-6.28 reading).

- **Cross-ply read: the training field's ply outweighs the tree architecture several-fold at every ply** (11-11
  11:31-11:48; diag/cross-ply-probit-cold; the coordinator, after probit-confirm-p2).
  - One field of both runs' finals: error-routing-p1 (G2, ply-1 field, v314) and probit-confirm-p2 (ply-2 field,
    v241), F-P x2 and T-P-Ne x2 from each. Tags: fp1/ne1 trained in the ply-1 field, fp2/ne2 in the ply-2 field.
    T 0.02, hybrid13, plies 1-3, 75 round-robins (41,400 games among entries), 16.5 min wall.

  | Contrast (T 0.02; ± is the 95% interval) | ply 1 | ply 2 | ply 3 |
  |---|---|---|---|
  | fp1 minus fp2 (flat probit: ply-1 field minus ply-2 field) | 167 ±22 | -316 ±21 | -159 ±19 |
  | ne1 minus fp2 (ply-1-field error tree minus ply-2-field flat) | 304 ±24 | -267 ±20 | -246 ±19 |
  | ne2 minus fp2 (ply-2-field error tree minus its flat) | 0 ±21 | -4 ±17 | -71 ±19 |
  | ne1 minus fp1 (ply-1-field error tree minus its flat) | 137 ±22 | 48 ±20 | -88 ±17 |
  | ne1 minus ne2 (error tree: ply-1 field minus ply-2 field) | 304 ±24 | -263 ±20 | -175 ±18 |

  - Per entry minus the fp2 mean: ply 1 ne1 317 and 292, fp1 167 and 167, fp2/ne2 -4 to 4; ply 2 ne2 14 and -22,
    fp2 2 and -2, ne1 -265 and -269, fp1 -314 and -317; ply 3 fp2 0, ne2 -62 and -80, fp1 -157 and -160, ne1 -241
    and -251.
  - The analyst's prediction (theory-2 6.30): fp1 - fp2 100 to 200 / -300 to -450 / -400 to -600; ne1 - fp2 200 to
    320 / -230 to -380 / -360 to -570; ne2 - fp2 within 25 everywhere. Plies 1 and 2 held throughout; ply 3 missed
    (the ply-1-field models lose 159-246 there, not 360-600, and ne2 loses 71).
  - Reading: the evaluator is good at the ply its training field played (the LR verdict of 03:48, now for probit
    flats and trees). The tree's architecture gain lives inside the ply-1 field: 137 at ply 1 there, 48 carried to
    ply 2, -88 at ply 3; trained in a ply-2 field it is 0 / -4 / -71. At ply 2 the ply-2 flat beats the ply-1 tree
    by 267. At ply 3 both trees are below their own field's flat (-88, -71).

- **probit-confirm-p2: trained in a ply-2 field, the probit trees gain nothing at ply 2 and lose at ply 3, with all 9
  leaves held** (11-11 11:19-11:30; Batch 5 item 3 widened; theory-2 6.27's prediction missed).
  - Run: probit-confirm-p2, seed 35, ply-2 field (entry_ply 2, no ply-1 entries), F-P x2, T-P-Ne (error tree, limit
    4, 1M, noise 0.03, non-merging) x2, T-P-Nc-merge (cluster tree, fresh gate at --revise 2, limit 4, 1M, noise 0.03)
    x2; hybrid13 games, hybrid6 training; done at 242 (plateau).
  - Read: diag/probit-confirm-p2-cold, final versions v241, T 0.02, hybrid13, plies 1-3, 75 round-robins (22,950
    games among entries).

  | Contrast (T 0.02; ± is the 95% interval) | ply 1 | ply 2 | ply 3 |
  |---|---|---|---|
  | Error trees (T-P-Ne) minus F-P | -17 ±23 | 18 ±21 | -49 ±22 |
  | Cluster trees (T-P-Nc-merge, --revise 2) minus F-P | -3 ±23 | -9 ±21 | -65 ±22 |
  | Error minus cluster | -14 ±23 | 26 ±21 | 17 ±21 |

  - Per line minus the F-P mean: ply 1 error -14 and -19, cluster 0 and -6; ply 2 error 25 and 10, cluster 18 and
    -36; ply 3 error -42 and -56, cluster -55 and -76.
  - Leaves at v49/99/149/199/241: error trees 7, 9, 9, 9, 9 (both seeds); cluster trees 5, 5, 5, 9, 9 (both seeds:
    --revise 2 held the splits, the second level only from ~v199).
  - The analyst's prediction (6.27): error trees 50-90 over F-P at ply 2, 40-80 at ply 1, 20-60 at ply 3; cluster
    10-30 below error. Missed: with 9 leaves held, the error tree is at F-P's level at its own ply. By 6.27's own
    conditional, the ply-2 field's games are the difference and the probit trees are a ply-1 family.
    probit-trees-p2 (gate merged to 1 leaf) said the same; here the splits are held.
  - Against G2 (ply-1 field: error trees 118 over F-P at ply 1 and 77 at ply 2), the tree's edge is a property of
    training in a ply-1 field. Cross read of both fields' finals: diag/cross-ply-probit-cold (running 11:35).

- **G3, the cell DAG: 30 over F-L at ply 1 but 30 below the LR cluster tree in the same field; it uses 3 of its 5
  leaves. Seat 2's third seed: 59 at ply 1** (11-11 11:19-11:22; theory-2 6.24 form (a)).
  - Run: dag-lr-p1, seed 33, done at 426 (plateau; last crowning 397).
  - Read: diag/dag-lr-p1-cold, final versions v425, T 0.02, hybrid13, plies 1-2, 75 round-robins (9,900 games).

  | Contrast (T 0.02; ± is the 95% interval) | ply 1 | ply 2 |
  |---|---|---|
  | Cell DAG (D-L-Nc: 9 cells over 5 shared LR leaves, cells re-pointed by fresh scoring) minus F-L | 30 ±23 | 15 ±22 |
  | LR cluster trees (T-L-Nc-merge limit 4, 1M, fresh gate; seat 2) minus F-L | 59 ±23 | 16 ±22 |
  | Cell DAG minus trees | -30 ±23 | -1 ±22 |

  - Per line minus the F-L mean, ply 1 / ply 2: trees 66 / 26 and 52 / 6; cell DAGs 35 / 12 and 24 / 18.
  - Cell-to-leaf shares (decisive positions of the read's games): seed 1's 9 cells point to leaves (5 2 3 5 5 5 2 5
    4): leaf 5 42.5%, leaf 2 37.4%, leaf 4 20.1%, leaf 3 0.0% (its one cell is empty), leaf 1 no cell. Seed 2: leaf 5
    41.9%, leaf 2 39.7%, leaf 4 18.4%, leaf 3 0.0%, leaf 1 no cell. So both use 3 of 5 leaves (the old D-L-Nc-c
    collapsed to 98% on one leaf; this one doesn't). In-sample cell regret 0.0000: each cell already points to its
    best leaf. Mean |log-odds difference| between leaf pairs 1.1-1.3.
  - Trees at v425: 9 leaves each (limit 4 reached).
  - Seat 2's recipe at ply 1 now reads 43 (lr-trees-p1), 19 (G6) and 59 here, in three fields: mean ~40, positive
    in all three. At ply 2: -17 (G6) and 16 here; lr-trees-p1's LR trees lost 43.
  - The cell DAG takes no seat: below the tree it would replace at ply 1, level at ply 2, and fixed in size (its
    leaves never split), so it is not a model that expands.

- **error-dag-probit-p1: the probit error DAG at z 30 equals the error tree at plies 1 and 2; at z 0 it ties at ply 1
  and loses 41 at ply 2** (11-11 15:02-15:10; theory-2 6.29's prediction holds at ply 1, misses z 0 at ply 2).
  - Run: error-dag-probit-p1, seed 36, ply-1 field, F-P x2, T-P-Ne (limit 4, 1M, noise 0.03) x2, D-P-Ne-ctx (path
    context, router limit 4) at router-split z 0 x2 and z 30 x2; errdag3 then errdag4 (byte-identical) from cycle 248.
    Done at 367 by the plateau rule (last crowning 342).
  - Read: diag/error-dag-probit-p1-cold, final versions v366, T 0.02, plies 1-2, 18,000 games among entries.

  | Contrast (T 0.02; ± is the 95% interval) | ply 1 | ply 2 |
  |---|---|---|
  | DAG z 0 minus error tree | 0 ±19 | -41 ±19 |
  | DAG z 30 minus error tree | 3 ±19 | 1 ±19 |
  | Error tree minus F-P | 98 ±20 | 52 ±19 |
  | DAG z 30 minus F-P | 101 ±20 | 53 ±19 |

  - Per line at ply 2 against the error trees' mean: z 0 -54 and -28 (both seeds below), z 30 -7 and 9.
  - Router splits: z 0 used its budget of 4 in both seeds (by v79 and v49); z 30 seed 1 none, seed 2 four (v88-v141).
    Every router split was by the root or a copy of it, stacking copies of the game-phase split (treeview pages;
    closeness.py: z 30 seed 1's largest window rise was ~37 SE once, 20-26 several times).
  - Path cells (treeview2, model log-loss with cells minus all cells neutral, SE by game): z 0 seed 1 0.0004 worse
    ±0.0005, seed 2 0.0022 better ±0.0011. In error-dag-p1 (LR, v340): z 0 0.0084 and 0.0032 better (±0.0024,
    ±0.0012).
  - G2 replicated: the error tree is ~100 over F-P at ply 1 (G2 118), 52 at ply 2 (G2 77).
  - Twins (hybrid14's identity checks, 11-11 ~16:45): a path-context DAG that never router-splits is the same model
    as the error tree of the same recipe and seed. D-P z 30 #1 (no router split) is T-P-Ne #1, with identical own
    games and self-games. So "z 30 minus tree" partly compares a model with itself. The gaps between those twins in
    the cold read (per line, about 10 Elo) are the read's noise.

- **error-dag-p1: the LR error DAG ties the LR error tree at plies 1 and 2, at z 0 and at z 160** (11-11 15:50-15:56;
  my prediction, within 15 of the tree at ply 1, holds just).
  - Run: error-dag-p1, seed 34, ply-1 field, F-L x2, T-L-Ne (limit 4, 1M) x2, D-L-Ne-ctx (path context, router limit
    4) at router-split z 0 x2 and z 160 x2; errdag2 then errdag4 (byte-identical) from cycle 140. Done at 362 by
    the plateau rule.
  - Read: diag/error-dag-p1-cold, final versions v361, T 0.02, plies 1-2, 18,000 games among entries.

  | Contrast (T 0.02; ± is the 95% interval) | ply 1 | ply 2 |
  |---|---|---|
  | DAG z 0 minus error tree | -11 ±19 | -16 ±19 |
  | DAG z 160 minus error tree | -15 ±19 | -1 ±19 |
  | F-L minus error tree | -46 ±19 | -55 ±19 |
  | DAG z 160 minus F-L | 31 ±19 | 55 ±19 |

  - Per line at ply 2 against the error trees' mean: z 0 6 and -37, z 160 -7 and 6.
  - Router splits: z 0 used its budget of 4 in both seeds; z 160 none in either (model files "pathcontext 1 0";
    closeness.py: largest window rises ~158 SE in seed 1, ~184 in seed 2).
  - Path cells: see the probit entry above (LR z 0 at v340: 0.0084 and 0.0032 better).
  - Path-stats on the finals did not run: the run's pool rounds were already gone (treeview pages cover it).
  - Twins: z 160 never split, so its lines are the LR error trees of the same seed (hybrid14's checks: D-L z 160 #1
    is T-L-Ne #1). "z 160 minus tree" (-15 / -1) compares identical models: per line, dz160.L1 minus ne.L1 is -10 at
    ply 1 and -11 at ply 2, and dz160.L2 minus ne.L2 is -20 and 10. That is the read's noise for one pair of lines.
  - With the probit test: no error DAG beats its tree at either ply, at any z tried; z 30 / z 160 behave as the tree
    except probit z 30 seed 2, which split 4 times and still tied.

- **probit-late-p2: after 400 cycles in a ply-2 field the probit error tree is still level with F-P at ply 2; late,
  it closed 22 at ply 2 and gained 64 at ply 3 on its flat** (11-11 12:48-13:08; theory-2 6.32's prediction missed).
  - Run: probit-late-p2, seed 38, ply-2 field (entry_ply 2, no ply-1 entries, self-games), F-P x2, T-P-Ne (error tree,
    limit 4, 1M, noise 0.03, non-merging) x2; hybrid13 games, hybrid6 training. One factor against probit-confirm-p2:
    the plateau floor 400 (K 35, was floor 200). Done at 400 by the plateau rule (last crowning in cycle 360), about
    2 CPU-h.
  - Read: diag/probit-late-p2-cold, v241 and v399 of every line, T 0.02, hybrid13, plies 1-3 (41,400 games among
    entries); arm_btg.py.

  | Contrast (T 0.02; ± is the 95% interval) | ply 1 | ply 2 | ply 3 |
  |---|---|---|---|
  | Error trees minus F-P, finals (v399) | -23 ±20 | -3 ±18 | 17 ±19 |
  | Error trees minus F-P, at v241 | -58 ±20 | -25 ±19 | -47 ±18 |
  | F-P v399 minus v241 | -24 ±20 | 122 ±19 | 18 ±18 |
  | Error trees v399 minus v241 | 12 ±20 | 144 ±19 | 83 ±19 |

  - Leaves (`leaves`): 7 at v99, 9 at v241 and v399 in both seeds; every split held.
  - Per line minus the F-P finals' mean: ply 2 F-P 15 and -15, trees 9 and -15; ply 3 trees 53 and -19 (one seed
    carries the ply-3 gain), F-P 9 and -9.
  - The ply-2 field was far from its plateau at v241: both kinds gained 120-145 at ply 2 after it, with crownings to
    cycle 360. probit-confirm-p2 (floor 200) stopped at 242, so its "no gain at ply 2" was read well before the
    field's plateau. Ply 1 got no part of the late gain (the flat lost 24 there): the late games specialise to ply 2.
  - The analyst's prediction (6.32): tree minus F-P 15 to 45 at ply 2, 0 to 30 at ply 1, -60 to -20 at ply 3; tree
    v399 minus v241 10 to 40 at ply 2. Missed: -3, -23 and 17; the tree gained 144 from v241 (the flat 122).
  - 6.32's decision rule for the user: a final ply-2 gain of 40 or more keeps the ply-1 screen; under 20, screen at
    ply 2. The result, -3 ±18, is under 20. The tree was still closing on F-P at the end (22 at ply 2 and 64 at
    ply 3 over the last 160 cycles), so a longer ply-2 run could still separate them.

- **G6, LR trees: both kinds gain only ~20 over F-L at ply 1 and lose at ply 2; seat 2 does not replicate its 43**
  (11-11 10:41-10:42; theory-2 6.24).
  - Run: lr-share-p1, seed 31, F-L x2 in the field, done at 343 (plateau; ~1.3 CPU-h).
  - Read: diag/lr-share-p1-cold, final versions v342, T 0.02, hybrid13, plies 1-2, 75 round-robins (9,900 games).

  | Contrast (T 0.02) | ply 1 | ply 2 |
  |---|---|---|
  | LR cluster trees (T-L-Nc-merge, fresh gate; seat 2) minus F-L | 19 ±22 | -17 ±22 |
  | LR error trees with leaf sharing 0.3 (T-L-Ne-share3) minus F-L | 22 ±22 | -34 ±22 |
  | Cluster minus sharing | -3 ±22 | 17 ±22 |

  - Per line minus the F-L mean, ply 1 / ply 2: sharing 33 / -31 and 10 / -37; cluster 27 / -25 and 11 / -9.
  - Leaves at v342: all four 9 (limit 4 reached).
  - Seat 2's recipe now reads 43 (lr-trees-p1) and 19 here; the seat needs 40 in two seeds. G3 (dag-lr-p1) has a
    third seed. The LR trees lose at ply 2 again (lr-trees-p1: -43). The leaf-sharing arm is out of the six
    regardless (the user, 10:05: no leaf sharing in error trees/DAGs).

- **Error DAG option built (chessai-errdag): it nests the tree in play; offline, no router-split threshold beats the
  tree** (11-11 ~10:45; the user's error-family decision; models/from-scratch/errdag/summary.txt, errdag.patch).
  - Build: errdag-src = bddctx-src plus `--router-split-z Z` (BDD only): a router splits when its mean error rose
    by more than Z SE over its previous window (and is at least the mean of all routers); 0 = the old stall rule,
    inf = never. 166 tests pass; without the flag byte-identical to chessai-bddctx.
  - Identity: at z = inf the BDD, with or without path context, makes the tree's splits, predictions and games
    (T-L-Ne flags; limit L maps to L). Model files differ only in the routers' window counters (no effect on
    routing or prediction). Play: errdag = hybrid13 in 12 of 12 combinations (F-L, T-L-Ne, D-L-Ne x depth 1-2 x
    T 0.1 and 0.02); errdag loads path-context models, hybrid13 refuses them.
  - Offline sweep (30 fixed ply-1 rounds, 2 seeds, 10 held-out parts; the tree 9 leaves, log-loss 0.5625):
    z 0-10 train identical models (3-4 router splits at limit 8; winner log-loss 0.0007 better, SE 0.0007; full
    log-loss 0.0030 worse, SE 0.0013); z 0 at limit 4 makes 2 router splits and leaves 5 leaves against 9 (router
    splits share the limit); z 30 1-4 router splits, level; z 100 none (= the tree). The triggering rises are 10-100
    SE, systematic (the routing target shifts as copies diverge), so z is a count dial, not a significance level.
    Seeds disagree by more than the SEs. Without path context z 0 equals the tree; home targets no better.
  - Separate router budget (10:53): chessai-errdag2 (md5 2f963aa0) adds `--router-split-limit N`: --split-limit then
    counts leaf splits only, router splits cap at N, and a router splits only while router splits < leaf splits.
    168 tests pass; without the flag errdag2 = errdag byte for byte; with it at z = inf (with or without path
    context) held-out eval-loss and games at depths 1-2, T 0.1 and 0.02 are the tree's.
  - Sweep again (leaf 4 plus router 4, path context, 100k windows): every arm keeps the tree's 9 leaves. Router
    splits per seed: z 0: 4 and 3; z 30: 4 and 1; z 50: 4 and 0; z 100: 0. No z beats the tree (winner log-loss
    within 0.0007, SE ~0.0006; full log-loss z 0 0.0030 worse, SE 0.0013; z 50 0.0018 better, SE 0.0012).
  - League arms chosen by count: z 0 (~4 router splits) and z 160 (the sweep's z 50 scaled by sqrt(10) for 1M
    windows, ~2; to be checked by counting sibling sets). Spec leagues/specs/batch5/error-dag-p1.json, DEFERRED
    (11:20): the probit test below took G3's slot, since the six's error tree is the probit T-P-Ne.
  - Probit leaves in the BDD (11:20): chessai-errdag3 (md5 261fe199). Probit path cells: one Gaussian latent
    weight per (cell, value), mean 0 and variance noise^2 (0.03^2) at creation, summed into the node's latent and
    trained in its ADF update; path context with probit leaves needs --split-noise > 0. 171 tests pass. Identity:
    errdag3 = errdag2 on every earlier kind; hybrid6 = errdag3 on the probit tree and F-P; errdag3 plays T-P-Ne vs
    F-P exactly as hybrid13; at z = inf (router limit 4, with or without path context) the probit BDD gives T-P-Ne's
    held-out eval-loss on all 10 parts and its games.
  - Probit sweep (30 fixed ply-1 rounds, 100k windows, seeds 11 and 12, leaf 4 plus router 4, path context; against
    the probit tree of the same seed, mean (SE over 10 held-out parts)):

    | z | router splits | leaves | log-loss vs tree | winner log-loss vs tree |
    |---|---|---|---|---|
    | 0 | 4, 3 | 9, 7 | 0.0015 worse (0.0008) | 0.0037 better (0.0009) |
    | 5 and 10 (identical models) | 4, 2 | 9, 7 | 0.0002 worse (0.0006) | 0.0041 better (0.0011) |
    | 20 | 3, 2 | 9, 9 | 0.0024 worse (0.0007) | 0.0025 worse (0.0010) |
    | 25 to 100 | none | 9, 9 | the tree | the tree |

    Unlike LR (no z beat the tree), probit DAGs with router splits predict the winner better offline at z 0-10.
    Seed 12 at z 0-10 made only 3 leaf splits in the 30 rounds (7 leaves), so leaf count is mixed in.
  - League arms (the coordinator): z 0 (busy, router splits at the cap) and z 30 (offline z 5-10, the best band,
    scaled by sqrt(10) to 1M windows is 16-32; the fork's draft had z 60, which maps to offline z 19, just under the
    cliff at 20-25 and at the one arm worse than the tree on both losses). error-dag-probit-p1 launches in G3's slot.

- **G2, routing kind for probit trees: the non-merging probit ERROR tree gains 118 over F-P at ply 1 and 77 at ply 2,
  level with seat 1's cluster tree (106 / 62) in the same field** (11-11 10:04-10:07; theory-2 6.24).
  - Run: error-routing-p1, seed 30, F-P x2 in the field, done at 315 (plateau; ~1.05 CPU-h).
  - Read: diag/error-routing-p1-cold, final versions v314, T 0.02, hybrid13, plies 1-2, 75 round-robins (9,900 games
    among entries).

  | Contrast (T 0.02) | ply 1 | ply 2 |
  |---|---|---|
  | Probit error trees (T-P-Ne, NB error router, non-merging) minus F-P | 118 ±23 | 77 ±22 |
  | Seat 1's cluster trees (T-P-Nc-merge, fresh gate) minus F-P | 106 ±23 | 62 ±22 |
  | Cluster minus error | -11 ±22 | -15 ±22 |

  - Per line minus the F-P mean, ply 1 / ply 2: error 125 / 78 and 110 / 76; cluster 118 / 70 and 94 / 55.
  - Leaves at v314: all four trees 9 (limit 4 = 4 splits of 3 reached), the cluster trees included (they held 3-7
    in G1 and G4).
  - The first cold ply-1 evidence for an error tree in the new setup: it clears every seat requirement (two lines,
    over 40 at ply 1, leaves held, no ply-2 loss). The analyst had predicted "within 20 of the cluster tree": held.
  - Seat 1's recipe reads 106 here, 44 in G1, 51 in G4 and 87-104 in p1b/p1c: the field moves the size by a
    factor of two; in-field contrasts are what count.
  - Confirmation at ply 2: probit-confirm-p2 (running from 10:08).

- **G4, growth: the 300k window does not grow trees, it churns them; the 1M trees hold 5-7 leaves and gain 51**
  (11-11 09:48-09:50; theory-2 6.24).
  - Run: growth-300k-p1, seed 29, F-P x2 in the field, limit 8, done at 341 (plateau; ~1.0 CPU-h).
  - Read: diag/growth-300k-p1-cold, final versions v340, T 0.02, hybrid13, plies 1-2, 75 round-robins (9,900 games
    among entries); make_arm_read.py and arm_btg.py.

  | Contrast (T 0.02) | ply 1 | ply 2 |
  |---|---|---|
  | 1M-window trees minus F-P | 51 ±22 | 4 ±22 |
  | 300k-window trees minus F-P | -47 ±23 | -63 ±23 |
  | 1M minus 300k | 99 ±23 | 67 ±23 |

  - Per line minus the F-P mean, ply 1 / ply 2: 1M #2 57 / 4, 1M #1 46 / 4; 300k #1 -3 / 3 (1 leaf at the end:
    flat probit); 300k #2 -91 / -129 (3 leaves at the end).
  - Leaves (`leaves`): 1M 3-7 early, 5-11 late, 5 and 7 at v340 (limit 8 never reached); 300k 5-9 at v25-50, then
    mostly 1 (the smaller window's merge test fires on noise; revise 0 merges at any negative mean).
  - Against the prediction (7-15 leaves at 300k, 3-5 at 1M, 10-30 more Elo for the grown tree): wrong in sign. Growth
    by a shorter window fails; the 1M trees do grow slowly with data.
  - Seat 1's recipe (limit 4 or 8, 1M, 0.03, fresh) now: 44 (G1) and 51 (G4) in 6-line fields with floor 300, 87-104
    in p1b/p1c; at ply 2: 12 and 4 here, ~40 in p1b/p1c.

- **Routing loss on G4's grown trees is tiny; their second-level leaves are near-copies** (11-11 ~09:45; for the
  user's question whether routers could track what routing loses and split on it).
  - Sample: 1000 games, growth-300k-p1's 1M-window tree #2 v334 against F-P#1 v334, depth 1, T 0.1, seed 11; 44,057
    decisive positions. `chessai-hybrid13 routing --use-probs` on three trees of the 1M arm.
  - Group-level regret (what sending each router-output group to its best child would save), nats per position
    through the router; the per-position hindsight figure (each position to whichever child predicted it best:
    mostly outcome noise) in brackets:

  | Tree | Leaves | Root | Second-level routers | Per position overall |
  |---|---|---|---|---|
  | #2 v334 | 7 | 0.0000 (0.098) | 0.0000 (0.036), 0.0035 (0.070) | ~0.0013 |
  | #2 v318 | 11 | 0.0002 (0.103) | 0.0000, 0.0013, 0.0040, 0.0002 | ~0.0012 |
  | #1 v338 | 5 | 0.0000 (0.123) | 0.0008 (0.070) | ~0.0003 |

  - The trees' recalibrated winner log-loss is 0.343-0.348, so routing costs under 0.4% of it. The root (the phase
    split) is right everywhere; what little regret there is sits in second-level routers.
  - Near-duplicates: each second-level router's children are close to each other. The closest pairs by mean
    |log-odds difference| are 0.21-0.44, against an all-pairs mean of 1.4-4.8 (#2 v318 leaves 8-9 0.214; #1 v338
    leaves 4-5 0.273). The trees split their data three ways among near-identical leaves below the root.
  - Reading: a router-split (DAG) trigger driven by achievable routing loss would rarely fire on today's trees; the
    reverse DAG move (near-copies share one leaf) has more room. The earlier regret trigger (bddregret, 11-08) used
    the per-position regret and split when it rose; level with a same-leaves BDT. The user's unified tree/DAG idea
    (11-11 ~09:50) and the analyst's design: theory-2 6.25; IDEAS.md.
  - **Out of sample, the routing loss is nil** (11-11 ~10:15, chessai-routecells, md5 b01ed2db: hybrid13 plus a
    `routing-cells` subcommand; `routing` byte-identical; models/from-scratch/unified/summary.txt and out/; ~0.05
    CPU-h). Six probit trees (G4 #2 v318/v334/v340, #1 v338/v340; G1's 5-leaf NB tree), 4000 games each against
    the field's F-P final at depth 1, T 0.1. G = the gain of re-pointing finer posterior cells (first child,
    second child, log-margin bin) to their best child, chosen on one half of the games and scored on the other,
    SE clustered by game:
    - pooled per decisive position -0.0014 to 0.0004 (calibrated loss), -0.0032 to 0.0007 (the gate's
      uncalibrated loss); no router reaches 0.005 or is positive by 2 SE (largest 0.0040 ±0.0022); 15 of 17
      readings at or below 0.001. The in-sample regret above is selection bias (v334 d3: 0.0019 in sample,
      -0.0012 ±0.0020 out of sample).
    - Duplicates: all 10 second-level routers in the five G4 trees hold a pair within 2 SE both ways (16 of 30
      pairs raw, 20 of 30 on slopes only, i.e. after a per-route intercept); root children never; G1's tree none.
      "Within 2 SE" cannot exclude 0.005-0.01. One asymmetric pattern: a generalist leaf matches its siblings on
      their positions while they are 0.23-0.36 worse on its own.
    - Verdict (6.25's rule): the routing move cannot pay on these leaves; only sharing has targets. Router splits,
      by any trigger (the user's subtree-error stall included), would mostly be merged back by the gate.
  - **The probit ERROR trees (G2's T-P-Ne v314, 9 leaves) checked the same way** (11-11 ~10:55; coordinator;
    models/from-scratch/unified/errtrees/; 4000 games each against F-P#1 v314, depth 1, T 0.1, seeds 111-112;
    ~195k decisive positions each):
    - G at the cells, overall per decisive position: 0.0011 (calibrated) and -0.0000; at the B-way partition
      0.0002 both. Largest single router: 0.0019 ±0.0012 (1.6 SE); one B-way reading 0.0008 ±0.0004 (2 SE).
      Routing headroom is about zero here too.
    - Duplicates: far fewer than in the cluster trees. One tree has one duplicate pair (and one more on slopes
      only), the other none; most sibling pairs differ by 0.01-0.07 nats with SEs of 0.002-0.005. Label-driven
      splits do place children where their losses differ (the analyst's one real difference), but re-routing
      among those children has nothing left to win.
    - (At T 0.1 and without calibration these trees were level with raw F-P in the game-generating arenas, 1 and
      6 ±8, against 118 in the T 0.02 calibrated read: the known temperature/calibration confound.)

- **G1, the router kind: the NB router keeps its splits and wins; the LR router never splits** (11-11 09:33-09:36;
  theory-2 6.24).
  - Run: router-kind-p1, seed 28, F-P x2 in the field, done at 300 (0.94 CPU-h).
  - Read: diag/router-kind-p1-cold, final versions v299, T 0.02, hybrid13, plies 1-2, 75 round-robins (9,900 games
    among entries); leagues/specs/arenas/make_arm_read.py and arm_btg.py.

  | Contrast (T 0.02) | ply 1 | ply 2 |
  |---|---|---|
  | NB-router probit trees (seat 1's config) minus F-P | 44 ±22 | 12 ±22 |
  | LR-router probit trees minus F-P | -9 ±22 | 4 ±22 |
  | LR router minus NB router | -53 ±22 | -7 ±22 |

  - Leaves (`leaves`, v100/200/299): the NB-router trees have 3, 5 and 5 in both lines. The LR-router trees have 1
    throughout, because the fresh gate undid every split, so they are flat probit in effect.
  - Per line at ply 1: NB trees 43 and 45, LR trees -2 and -15, F-P 7 and -7.
  - Seat 1's config gains 44 here against 87-94 in p1b/p1c. The field and floor differ (6 lines and floor 300, against
    8 lines and floor 200).

- **Pre-screen of old champions never rated cold: none beats flat by more than ~34 at ply 1** (11-11 ~08:55-09:10;
  theory-2 6.23 item 5).
  - Arenas: diag/prescreen-p12 (25 entries) and diag/prescreen-old-p12 (the champion run's 12 alone, the fair read for
    that block). T 0.02, hybrid13, plies 1-2, 0.21 CPU-h. Specs, scripts and the write-up: leagues/specs/arenas/
    make_prescreen*.py and prescreen_btg.py, leagues/arenas/diag/prescreen-p12/summary.txt.
  - Each side is one version, picked at T 0.1 by its own league's record.

  | Champion-run family minus its flat LR (±36-37) | ply 1 | ply 2 |
  |---|---|---|
  | T-L-Ne-share3-c | 31 | 81 |
  | T-L-Nc-c | 34 | 45 |
  | M-L4-N-c | 16 | 43 |
  | T-L-Ne-pre-c | 26 | 41 |
  | M-L8-L-c | 11 | 36 |
  | T-L-Te-c | -32 | 14 |
  | D-L-Nc-c | 6 | 11 |
  | D-L-Ne-skill-c | 7 | 9 |
  | T-L-LLs-merge-c | -11 | -28 |
  | M-LP-L-hard-c | -6 | -29 |

  - The champion run's flat probit (F-P-tau0-c) is 219 below its flat LR at ply 2.
  - cluster-error's limit-1 trees are level with their F-L and F-P at ply 1 (-24 to 7). At ply 2 they are 81-171
    below F-L, except T-L-Lc, which is level.
  - single-split-probit's three arms never read before (T-P-Lc, T-P-Le, T-P-Te) are -11 to -21 against its F-P at
    ply 1.
  - **Closed for the shortlist:** both champion-run DAGs, cluster-error's limit-1 trees, single-split-probit's arms,
    the TAN error tree, the LR-router split tree and the LR+probit hybrid.
  - **Worth reopening at ply 2 only:** T-L-Ne-share3-c (81 ±37, head-to-head 168 ±91 over 60 games). It is large and
    long-trained. T-L-Nc-c, M-L4-N-c, T-L-Ne-pre-c and M-L8-L-c are marginal (36-45).
  - Caveat: the results depend on the field. Chaining through two earlier arenas puts share3 only ~18 ±45 over the old
    flat LR.

- **Training on T 0.02 games is much worse: keep the experiments' own games at T 0.1** (11-11 08:34-08:43;
  theory-2 6.23 item 1).
  - Runs: train-t002-p1 and train-t01-p1 are F-L x2 each in a pure ply-1 field, seed 27, hybrid6 training. They are
    identical except that the first plays its own games (tournament and self-games) at T 0.02. Both stopped on the
    plateau just after cycle 200 (0.10 CPU-h each).
  - Read: diag/train-temp-p1, final versions, T 0.02, hybrid13, plies 1-2, 150 round-robins (8,400 games among
    entries, 106 CPU-s; leagues/specs/arenas/temp_btg.py).

  | Contrast (T 0.02) | ply 1 | ply 2 |
  |---|---|---|
  | trained on T 0.02 games minus trained on T 0.1 games | -106 ±18 | -221 ±20 |
  | line #2 minus line #1, T 0.02 arm (the floor) | 5 ±25 | 18 ±24 |
  | line #2 minus line #1, T 0.1 arm | 5 ±24 | 10 ±28 |

  - Far beyond the analyst's 10-30 and the floor. Rate at T 0.02, but train on T 0.1 games (or another source of
    variety; untested).
  - The mechanism is open: games.csv shows no extra repetition by players, colours, result and length (8% in both
    arms), but it doesn't record moves.

- **The reverse rung: ply-3 training undoes W4's parity.** It keeps a third of W4's even-ply gain at ply 2 and
  little at ply 4.
  - Run: 11-11 06:51-07:22. flat-p3-w4-w3f1 is W4's finals (flat-p3-w4 v99) trained at ply 3 exactly as W3f1 (seed
    24, forget 1, cap 100). Only the start differs. It stopped on the plateau at cycle 97 (0.49 CPU-h).
  - Read: cold, final versions, T 0.02, hybrid11, 30 round-robins each, leagues/specs/arenas/rev_btg.py; 0.86 CPU-h.
    - diag/reverse-rung-p123: plies 1-3 with cross-ply games, 67,680 games among entries.
    - diag/reverse-rung-p4: ply 4, 7,200 games.

  | Contrast (T 0.02) | ply 1 | ply 2 | ply 3 | ply 4 |
  |---|---|---|---|---|
  | reverse rung minus W3f1 (one factor: the start) | 4 ±16 | 45 ±15 | -2 ±15 | 19 ±16 |
  | reverse rung minus W4 (its start) | 100 ±16 | -86 ±15 | 85 ±15 | -112 ±16 |
  | W4 minus W3f1 | -96 ±16 | 131 ±15 | -87 ±15 | 131 ±17 |
  | W3f1 minus flat-p3's finals (its start) | -7 ±16 | 9 ±15 | 17 ±15 | 50 ±16 |

  - **The analyst's four predictions (theory-2 6.21) hold.** The reverse rung is level with W3f1 at plies 1 and 3,
    and keeps 0-60 of W4's even-ply gain at plies 2 and 4.
  - Ply-3 games remove W4's odd-ply losses completely and most of its even-ply gain. A third stays at ply 2 (45 of
    131). A sixth stays at ply 4 (19 ±16 of 131, not significant).
  - Both residues are under the analyst's threshold of 60 for "one linear model holds both parities' corrections".
    So the parity correction mostly trades off in the same weights. What this implies is in theory-2 6.22.
  - W4 minus W3f1 is 131 at both even plies. That is less than W4 minus W3 (172, 200), because W3 lost 27-51 to
    reheating (next entry).
  - Unexplained so far: W3f1 is 50 ±16 above its start at ply 4, against only 9-17 at plies 2-3. It is with the
    analyst.
  - **One more ply of search, same evaluator** (cross-ply games):
    - ply 1 to 2: about 1000 (reverse rung 1049, W3f1 1008, flat-p3 992, W4 1235; ±59);
    - ply 2 to 3: about 700 (reverse rung 676, W3f1 722, flat-p3 714, W4 505; ±22).
    - A training rung's 130-200 is a fraction of one ply of search.

- **The warm start's cost is the reheated step** (11-11 05:15-05:18; diag/w3-forget-p23, T 0.02, hybrid11, 30
  round-robins at plies 2 and 3, 16,560 games among entries; leagues/specs/arenas/w3f_btg.py). flat-p3-w3f1 is W3
  exactly but with start_forget 1, so the AdaGrad sums are kept (0.6 CPU-h, cap 100). Final versions:

  | Contrast | ply 2 | ply 3 |
  |---|---|---|
  | W3 forget 1 minus flat-p3's finals (its start) | 7 ±17 | 22 ±17 |
  | W3 forget 8 minus flat-p3's finals | -49 ±17 | -14 ±17 |
  | W3 forget 1 minus W3 forget 8 | 56 ±17 | 36 ±17 |

  - Without reheating, the extra ply-3 training holds level with or above its start, so dividing the sums by 8 cost
    W3 its 27-51. But unreheated models barely move: one forget-1 line's champion stayed v0 for 100 cycles.
  - A rung needs to move (W4 doubled the pawn's value), so W4's 173 at ply 4 came despite a reheating cost of ~40.
    The analyst suggests dividing by 2-4 for the next rung.

- **Parity: W4's gain shows at even plies only** (11-11 04:59-05:15; T 0.02, hybrid11, Bradley-Terry among the
  entries; diag/parity-p1 30 round-robins, diag/parity-p5 10 round-robins, 560 games among entries). W4 (ply-4
  field) minus W3 (ply-3 field), same starts, final versions, at every ply read:

  | ply | 1 | 2 | 3 | 4 | 5 |
  |---|---|---|---|---|---|
  | W4 minus W3 | -85 ±23 | 172 ±14 | -1 ±13 | 200 ±20 | 3 ±38 |

  - At even plies the ply-4-trained models win by 170-200; at odd plies they are level or behind. Depth distance
    alone would put W4 ahead at ply 5 (it was one ply away, W3 two), as flat-p3 was ahead of flat-p2 at ply 4 (100).
    It isn't (3 ±38).
  - The analyst's mechanism (theory-2 6.19): a ply-k game's positions are chosen by a search whose horizon has one
    parity, so they carry that horizon's systematic errors. The evaluator fitted to their outcomes learns corrections
    for that parity, in the piece and square weights. The side-to-move weight still cancels at a fixed depth.
  - Open: flat-p3 (odd-trained) beat flat-p2 (even-trained) by 100 at ply 4, so parity is not the whole story. Depth
    and parity both act, and W3 carries a warm-start cost.
  - For the goal, if this holds: climb the ladder in steps of two for an even target depth and use the evaluator at
    its own parity. Or extend odd horizons with a capture search, the standard cure for the odd-even effect.

- **The ply-4 rung, warm-started: a ply-4 field gives a far better ply-4 evaluator, and the ladder works**
  (11-11 04:45-04:56; T 0.02, chessai-hybrid11; Bradley-Terry among the entries):
  - Plies 2 and 3: diag/warm-reads-p23, 30 round-robins, 46,800 games among entries.
  - Ply 4: diag/warm-reads-p4, 30 round-robins.
  - Contrast script: leagues/specs/arenas/warm_btg.py.

  The runs (seed 24, 6 rounds a cycle, AdaGrad sums divided by 8 at the start, scaled plateau min 60, cap 100):
  - W4 (flat-p3-w4): flat-p3's finals v199 trained on in a ply-4 field. Ended at the cap, final v99, 4.0 CPU-h.
  - W3 (flat-p3-w3): the same starts in a ply-3 field. Plateau at 79, final v78, 0.56 CPU-h.
  - Ladder rung (flat-p2-w3): flat-p2's ply-2 finals v244 trained on in a ply-3 field. Ended at the cap, final v99,
    0.69 CPU-h.

  | Contrast (final versions) | ply 2 | ply 3 | ply 4 |
  |---|---|---|---|
  | W4 minus W3 (the rung: field ply the one factor) | 172 ±14 | -1 ±13 | 200 ±20 |
  | W4 minus flat-p3's finals (its start) | 126 ±14 | -51 ±14 | 173 ±20 |
  | W3 minus flat-p3's finals (its start) | -46 ±14 | -51 ±14 | -27 ±19 |
  | ladder rung minus flat-p3's finals | -9 ±14 | -11 ±14 | |
  | ladder rung minus its start flat-p2 | -195 ±14 | 157 ±14 | |
  | flat-p3 minus flat-p2 (reference) | -186 ±14 | 168 ±14 | |

  - **The rung:** at ply 4, W4 beats W3 by 200 and its own start by 173. All four W4 lines are 165-179 above
    flat-p3's mean; all four W3 lines are -12 to -39. The analyst predicted 50-100.
  - **The ladder works.** A warm rung from the ply-2 finals reaches the from-scratch ply-3 level (-11 ±14) for
    0.69 CPU-h, against 3.2 for flat-p3. Like flat-p3, it pays ~190 at ply 2.
  - **The warm start costs something:** W3 ends 27-51 below its own start at every ply. Reheating the AdaGrad step
    (sums / 8), the 6-round cycles, or W3's earlier plateau stop (79) are the candidates.
  - **W4 is also strong at ply 2:** 126 over flat-p3, though ~60 below the ply-2-trained flat-p2. At ply 3 it equals
    W3. So the ply-4-trained models are good at plies 2 and 4 and not at 3. This looks like a parity pattern, which
    the analyst's fixed-depth argument (the side-to-move weight cancels) doesn't exclude for piece values.
  - **Piece values** (piece_values.py; relative to the queen):

    | Models | P | N | B | R | Q | side to move |
    |---|---|---|---|---|---|---|
    | W4 | 0.22 | 0.28 | 0.37 | 0.65 | 3.42 | -0.263 |
    | W3 | 0.12 | 0.24 | 0.31 | 0.56 | 3.12 | -0.205 |
    | ladder rung | 0.11 | 0.22 | 0.27 | 0.49 | 3.11 | -0.196 |
    | flat-p3 | 0.13 | 0.24 | 0.30 | 0.56 | 3.05 | -0.188 |
    | flat-p2 | 0.09 | 0.13 | 0.19 | 0.40 | 3.16 | -0.105 |

    The ply-4 field doubles the pawn's value and raises every piece. The ladder rung learned most of the ply-3
    values.

- **VERDICT (11-11 03:48): the ply of the training field decides the ply the evaluator is good at, replicated in
  two seeds.** The twins flat-p3 and flat-p2 (seed 20) and flat-p3-b and flat-p2-b (seed 23) differ only in the
  field's ply (F-L x4, pure field, self-games, 15 rounds, scaled plateau min 200). Final versions, near argmax
  (T 0.02), Bradley-Terry among the entries:
  - Plies 2 and 3: diag/flat-twins-cold, 40 round-robins, 50,400 games among entries, btg.py there.
  - Ply 4: diag/flat-ply4-interim-cold (seed 20) and diag/flat-twins-b-ply4 (seed 23), 40 round-robins each.

  | Contrast | ply 2 | ply 3 | ply 4 |
  |---|---|---|---|
  | ply-3 field minus ply-2 field, seed 20 | -153 ±13 | 181 ±13 | 100 ±18 |
  | ply-3 field minus ply-2 field, seed 23 | -216 ±14 | 173 ±13 | 108 ±18 |
  | both seeds | -185 ±10 | 177 ±10 | |
  | run to run, ply-3 fields (fp3 minus fp3b) | 31 ±13 | -7 ±13 | |
  | run to run, ply-2 fields (fp2 minus fp2b) | -33 ±12 | -15 ±12 | |
  | ply-3 fields minus lr-settings-new F-L 0.0125 v136 | -204 ±19 | 168 ±19 | 57 ±27, 75 ±27 |
  | ply-3 fields minus F-L-d4mix-c v426 | -167 ±19 | 96 ±19 | -15 ±28, 1 ±28 |
  | ply-2 fields minus lr-settings-new v136 | -19 ±19 | -9 ±19 | |

  - At ply 3 all eight ply-3-trained lines are 172 to 205 above the ply-2 lines' mean. All eight ply-2-trained
    lines are -11 to 25.
  - Training in a ply-3 field also wins at ply 4 (100 and 108), and loses ~185 at ply 2.
  - The run-to-run spread is small here (7-33), so self-games, the plateau rule and the rate don't explain it.
    flat-p2/-b match lr-settings-new, which had neither self-games nor the scaled plateau.
  - The old depth-4 lineage is level with the ply-3 fields at ply 4 and 96 behind at ply 3. Its games are 93% ply-2
    league games and 7% its own depth-4 games.
  - CPU: flat-p3 3.2 and flat-p3-b 3.2 CPU-h; flat-p2 0.9 and flat-p2-b 1.1 CPU-h.
  - **What changes: the piece values** (the analyst's lookup, theory-2 6.18; reproduced by
    diag/flat-twins-cold/piece_values.py). Value = the white-win minus black-win weight of a piece on a square minus
    that of the empty square, averaged over the squares the model has seen, for the 8 final models of each arm (black
    pieces mirror within 0.1):

    | Arm | P | N | B | R | Q | N, B, R relative to Q | side to move |
    |---|---|---|---|---|---|---|---|
    | ply-3 fields | 0.39 | 0.75 | 0.91 | 1.68 | 3.04 | 0.25, 0.30, 0.55 | -0.185 |
    | ply-2 fields | 0.29 | 0.42 | 0.59 | 1.24 | 3.22 | 0.13, 0.18, 0.38 | -0.102 |
    | classical 1-3-3-5-9 | | | | | | 0.33, 0.33, 0.56 | |

    Trained on ply-3 games, the minor pieces and rooks are worth 1.4-1.8 times more (the queen about the same). Their
    ratios move most of the way to the classical ones. The analyst's reading: a learned evaluator estimates the
    outcome under the play that generated its games. Deeper play converts a piece more often, so the weight rises.
    Shallow play can't cash a deeper model's material bets, which is why the loss below its ply (185) is larger than
    the gain above (~104).

- **flat-p3, interim read: a flat LR trained in a ply-3 field is the best ply-3 evaluator by far, and weak at ply
  2** (11-11 02:37-02:40; diag/flat-p3-interim-cold, T 0.02, 40 round-robins, plies 2 and 3). flat-p3's four lines
  (F-L x4 in a pure ply-3 field, self-games, seed 20, ended at cycle 200 on the plateau rule) at their final
  versions v199, against the best ply-2-trained flat LRs and the old depth-4 lineage:

  | flat-p3 mean minus | ply 2 | ply 3 |
  |---|---|---|
  | lr-settings-new F-L 0.0125 champion v136 (ply-2 field) | -221 ±34 | 174 ±33 |
  | lr-settings-new F-L 0.05 final v193 | -224 ±34 | 169 ±33 |
  | F-L-d4mix-c v426 (old; depth-4 self-play) | -123 ±33 | 98 ±32 |

  - The four lines are within 35 of each other at both plies (#2 minus #1: 7 ±42, 4 ±41).
  - Each evaluator is best at the depth of the games it trained on: the ply-3-trained lines win by 174 at ply 3 and
    lose by 221 at ply 2. The depth-4 lineage is in between at both plies.
  - The twin flat-p2 (identical recipe at ply 2) and both seed replicates give the one-factor verdict.
- **Ply 4: training at ply 3 also wins one ply deeper** (11-11 02:41-03:02; diag/flat-ply4-interim-cold, T 0.02,
  40 round-robins at ply 4 only; every entry beat every random player, so the fit is Bradley-Terry among the entries
  from their 3,600 mutual games, btg.py there). Final versions:

  | Contrast at ply 4 (T 0.02) | Elo |
  |---|---|
  | flat-p3 (ply-3 field) minus its twin flat-p2 (ply-2 field), 4 lines each | 100 ±18 |
  | flat-p3 minus lr-settings-new F-L 0.0125 v136 | 57 ±27 |
  | flat-p3 minus F-L-d4mix-c v426 | -15 ±28 |
  | flat-p2 minus lr-settings-new F-L 0.0125 v136 | -44 ±27 |
  | flat-p3 #2 minus #1; flat-p2 #2 minus #1 | 12 ±35; -20 ±35 |

  - The twins differ only in the field's ply: training in a ply-3 field gives a better ply-4 evaluator by 100.
  - No parity effect. The analyst predicted this (theory-2 6.16): at a fixed depth every leaf has the same side to
    move, so a linear evaluator's side-to-move weight cancels. The training ply still shapes the weights: the
    side-to-move weight is 0.187-0.188 in all four flat-p3 models and 0.105 in all four flat-p2 models.
  - Order at ply 4: d4mix 95, flat-p3 73-86, ply-2-trained LRs -42 to 24 (relative to flat-p2 #1). d4mix's games
    are 93% old-league ply-2 games and 7% its own depth-4 games.
  - Cost: a ply-4 game between flat LRs is 0.51 CPU-s on hybrid10, against 0.059 at ply 3 and 0.007 at ply 2
    (game_cpu.csv of the two arenas).

- **probit-trees-p2, read cold: probit trees trained at ply 2 gain nothing at any ply** (11-11 02:06-02:10;
  diag/cold-final-probit-trees-p2, final versions v276, T 0.02, 75 round-robins, plies 1-3). The run: p1b's best arm
  (T-P-Nc-merge limit 4, fresh gate, split noise 0.03) x2 against F-P x2 in a ply-2-only field (seed 18), ended at
  cycle 277 on the plateau rule. Its own T 0.1 table (current against current, against F-P#1): trees -42 and -53 in
  cycles 151-200, -36 and -12 in 201-277; F-P#2 -3 and 3.

  | Contrast (T 0.02) | ply 1 | ply 2 | ply 3 |
  |---|---|---|---|
  | probit trees minus F-P | -10 ±24 | -8 ±27 | 1 ±27 |
  | tree#2 minus tree#1 | 2 ±34 | 16 ±38 | -8 ±38 |
  | F-P#2 minus F-P#1 | -3 ±34 | 30 ±38 | -4 ±38 |

  - The analyst predicted 60-90 at the tree's own ply (2). There is no gain at ply 2, so "a tree gains at the ply it
    trained at" fails at ply 2. The ~95 ply-1 gain of the same arm (p1b, p1c) is a ply-1 effect.
  - Whether the trees grew leaves at ply 2 is with the analyst.

- **small-field-x4-p1 read cold: no LR-tree gain at ply 1** (diag/cold-final-small-field-x4-p1, final versions
  v189, T 0.02, 15 round-robins): T-L-Tc-merge limit 1 (stream gate, TAN router) x4 minus F-L x4: -7 ±30 at ply 1,
  8 ±31 at ply 2; per tree at ply 1 11, -15, -10, -13 (±47). Against probit-widen's 80-104, the two differ in leaf
  (LR/probit), gate (stream/fresh), leaves (3/up to 9) and router (TAN/NB cluster).

- **lr-trees-p1, read cold: LR trees gain at ply 1 too, and lose at ply 2** (11-11 01:33-01:38; diag/cold-final-
  lr-trees-p1, final versions v301, T 0.02, 115 round-robins (--rounds adds rounds)). The run: p1b's conditions with LR leaves (F-L x2,
  T-L-Nc-merge limit 4 threshold 1M with the fresh merge gate x2, pure ply-1 field, seed 21), ended at cycle 302 on the
  plateau rule. Its own T 0.1 table (current against current, against F-L#1): trees -16 and -15 in cycles 151-200, 1
  and 2 in 251-302; F-L#2 3 and 9.

  | Contrast (T 0.02) | ply 1 | ply 2 |
  |---|---|---|
  | LR trees minus F-L (2 lines each) | 43 ±21 | -43 ±22 |
  | tree#2 minus tree#1 | -21 ±30 | -14 ±31 |
  | F-L#2 minus F-L#1 | 6 ±30 | -10 ±31 |

  - Fresh-gate trees gain at the ply they trained at with either leaf (probit 80-104, LR 43), and T 0.1 hides it
    both times. The stream-gate limit-1 LR trees of small-field-x4-p1 showed nothing (-7 ±30).
  - At ply 2 these LR trees lose 43: a tree trained at ply 1 is worse one ply deeper than its flat twin. The test of a
    tree trained at ply 2 is probit-trees-p2 (running).

---

## 9. Speedups

Every deployed speedup is exact: games, models and reports are byte-identical. Rules, the queue and the reviews are in
models/from-scratch/fastbin/PERF-LEDGER.md.

| Date | Speedup | Saving | Write-up |
|---|---|---|---|
| 11-09 03:51 | chessai-hybrid4: three exact fast paths in the live tournament (cheap any-legal-move test at leaves, lazy candidates, bitboard encode) | -23% CPU like for like; cycles 42 min instead of 60 | fs/movegen |
| 11-09 17:15 | fast binaries, hybrid4g (new league) | new/old CPU: depth-0 flat 0.74-0.87, depth 1 0.53-0.62, depth-2 flat 0.45-0.59, trees 0.54-0.76 | fs/fastbin/summary.txt |
| 11-09 18:05 | hybrid4gfx: fixed-point flat LR and NB in search | LR self-play d0 0.74 and d2 0.39; NB 0.67 and 0.36; probit ~1.0; label training 0.29 | fs/fastbin/fx/tables.txt |
| 11-10 00:53 | hybrid5: fixed-point play for calibrated trees and DAGs at ply 1-2, float fallback inside the bound | tournament CPU per game: single-split 23.5 → 8.8 ms, probit 20.6 → 12.0, cluster-error 18.5 → 9.1 | fs/fastbin/hybrid5 |
| 11-10 05:20 | hybrid6: faster training (TAN integer batches, reused predictions, AVX2 LR step, fast parser) | training CPU ×0.60-0.63 | fs/fastbin/hybrid6 |
| 11-10 09:16 | hybrid7: faster tournament reports (append-only elo_history, sparse head-to-head) | report step 12-17 s → 7-9.5 s; peak 2.3 GB → 0.44 GB | fs/fastbin/hybrid7 |
| 11-10 10:56 | driver scan cache in leagues/leagues.py (not committed) | driver CPU 8.8 → 2.5 s a cycle | fs/fastbin/drivercache |
| 11-11 ~01:30 | chessai-hybrid10: captures and promotions first one ply above the leaves (values-only path) | ply-3 CPU: LR ×0.79, probit ×0.62, NB ×0.87, trees ×0.68-0.78, FM ×0.59; ply 2 unchanged | fs/fastbin/hybrid10 |
| 11-11 ~01:30 | arena own-games cache (leagues/arena.py) | a calibration hit ~0.1 s against ~1 s (depth 1) or ~9.5 s (depth 2) | fs/fastbin/owncache |
| 11-11 03:46 | chessai-hybrid11: each child's sums from the move itself, cheaper leaf hashing (values-only fixed-point path) | LR ×0.85 / ×0.88 / ×0.75 at plies 2/3/4; NB ×0.72 at ply 4; probit ×0.92 / ×0.87 at plies 3/4 | fs/fastbin/hybrid11 |
| 11-11 07:34 | chessai-hybrid12: one ply above the leaves, a legal capture found from the bitboards is searched before any moves are generated (cut windows; values-only fixed-point path) | game CPU at plies 3/4/5: W4 ×0.65 / ×0.95 / ×0.71, flat LR ×0.69 / ×0.97 / ×0.75, probit ×0.84 / ×0.99 / ×0.85, NB ×0.89 / ×0.99 / ×0.82; ply 2 and trees unchanged; games byte-identical to hybrid11 | fs/fastbin/hybrid12 |
| 11-11 08:11 | chessai-hybrid13: root cut windows for the log-odds softmax at T ≤ 0.05 and for argmax, at ply 2 and up (clearly worse root moves are searched only far enough to prove the sampled move unchanged; re-searched when the bounds don't decide, 0.98% of cut moves) | T 0.02 game CPU: flat LR ×0.72-0.84 (×0.77 typical), probit ×0.66-0.87 at plies 3-5, trees ×0.76-0.88 at plies 3-4; argmax W4 ×0.44 / ×0.24 at plies 3/4; T 0.1 and NB unchanged; games byte-identical to hybrid12 (55 of 55 comparisons; 59,022 searches asserted) | fs/fastbin/hybrid13 |

- **Compute profile** (2026-11-09 ~02:55, fs/movegen).
  - Tournament 70% of a live cycle's CPU. Move generation ~17% of the league's CPU, mostly leaves testing for mate.
  - Caches don't pay at depth ≤ 2: 0-9% of evaluations repeat. At d3 14% and d4 31% do.
- **Probit and the speedups:** probit costs 2.4× LR per ply at depth 2 and 2.9× at depth 4 (prices P2). LR and NB got
  fixed point and flat probit didn't; flat probit fixed point is queued in the ledger.

---

## 10. Theory

The backprop lens (2026-11-10 ~01:10; models/from-scratch/theory/summary.txt, labelled THEOREM, COUNTEREXAMPLE,
CONJECTURE or SIMULATION). The user's framing: the tree/DAG with error routers is meant as an answer to backprop. In
order of importance:
1. **The router targets optimise the wrong thing.** Best is inconsistent: with two outcomes it picks the child leaning
   to the commoner outcome. Soft is right only for Brier. Balanced routes by lift.
   - Consistent targets: the pairwise gap (B=2), fractional labels M - l_k, weighted all-pairs.
   - For an LR router this is one step on w/sum(w) with weight sum(w).
2. **Every error router is an evaluator in disguise:** its target is an outcome probability. Checked: LR routers yes,
   NB routers no (section 5).
3. **After a split every consistent target is silent.** Symmetry must be broken explicitly and without labels: hash
   routing for a window, disjoint halves or small noise.
4. **Home targets train R' on a counterfactual.** Fix: target the copy whose subtree, as routed, does best, with
   ties to home.
5. **A home DAG is a tree with a deeper gate.** Its ceiling is the routing headroom, 0.001-0.003 nats as measured.
6. **Growth that preserves every prediction** holds for leaf splits with F=1, LR under any F, home router splits and
   neutral path cells. It fails for CNBP with F>1.
7. **Descent.** Exact alternating steps with a consistent router problem never raise the routed loss. Leaf steps are
   unbiased SGD. Routers need a surrogate, and Best, Soft and Balanced are the wrong ones.
   - 7.7, AdaGrad asymmetry: a new parameter moves at full rate while old ones are nearly frozen. So every growth
     operation costs at first.
8. **Additive (residual) nodes** are the only growth operation with a monotone guarantee, and the closest to the
   user's residual-layer picture. With routing fixed the loss is convex, and the shared part tracks common drift B
   times faster than copies do.
9. **Split trigger.** The first-order gain of a split is 2|d||G_A|, so the right trigger is the size of the leaf's mean
   gradient over candidate regions. Today's trigger measures stalling, which every leaf does eventually.
10. **Train-then-route leak.** It is negligible for a mature router and large for a fresh LR router. Free fixes: route
    before the update, and break ties by hash.
11. **Conjecture:** a leaf split breaks even at about a million positions per leaf (~2,700 parameters, gain ~0.003
    nats).

Proposals from it: T1 consistent targets (now error-targets), T2 home-target refinement, T3 additive deltas (ready
to create) and T4 a gradient-disagreement trigger (IDEAS.md).

**Theory 2: the DAG ideas after T1, T3 and T4** (2026-11-10 ~22:45; models/from-scratch/theory-2/summary.txt, by a
resident analyst agent on Fable, ~11 CPU-min; the user asked for a fresh look after residual-leaves, trigger-gradient
and error-targets). Labels as before.
- None of the three contradicts the theory. Each fixed what it set out to fix and was measured on an effect too small
  to show over the noise: every routing effect measured is 0.001-0.003 nats, and one 0.004-nat recipe gave 70-100,
  10-40 and -25 to 26 Elo in three runs.
- Residual leaves (T3): the code's deviations from the theory don't matter (SIMULATION). Only about half the league's
  drift is common to the leaves (EVIDENCE: 0.47-0.50 in the 9-leaf copy tree, 0.01-0.22 across single-split's phase
  leaves), and a residual leaf is 30-50% noisier than a copy (SIMULATION). The run had no shrinkage (--residual-l2 at
  its default 0; checked in its config).
- T4 decides when to split, not where (it installs the same fresh cluster router). In its log the first-order gain per
  example was nearly the same at every leaf (0.0022-0.0038 nats, 36 tests), so it fired where the data were largest.
  The arms differ by the timing of one or two splits, 10-20 Elo over a run.
- Consistent targets (T1): THEOREM, leaves that differ only in temperature make the consistent target a threshold on
  certainty, and leaves that differ in lean make it who's winning. Both are stable fixed points an LR router can
  learn, with ~10x the signal of competence differences. EVIDENCE: T-L-Le's U 0.37 against 0.03 for the cluster tree;
  gap targets 0.07-0.11. Probit's shrinkage gives data-poor leaves a different temperature, hence T-P-Le's failure
  (U can't be measured on probit trees yet: the routing tool overflows).
- DAG router splits: the null was predicted. THEOREM: a shared-leaf DAG is a constrained tree (gain ~0.001 nats), and
  a home DAG is capped by the routing headroom. The one DAG that adds conditioning without fragmenting data is
  additive per-router tables (its section 2.4).
- The LR trees' ply-2 dip is mostly scale and noise (about 25 Elo over 45 lines, SE ~12). PATH-FORWARD's mechanism (b)
  fails in a toy search (COUNTEREXAMPLE): the minimum over noisy replies steers away from noisy regions. Shrinkage to
  a shared base halves the toy's search regret.
- Suggestions (its section 3; tests by cost in section 4): S1 shrinkage leaves (residual leaves with --residual-l2 > 0,
  already built; l2 unknown, so an offline sweep first), S2 calibration-invariant error targets (a small change in
  delegation.rs), S3 growth as boosting (split children initialised along T4's per-region gradients). Its rule: no
  experiment on a contrast under 0.005 nats without two seeds and a ply-1 twin.

**Theory 2, overnight additions** (11-11 ~00:10, theory-2/summary.txt sections 5-6):
- Router focus (THEOREM 5.1): a router's training signal is either label-free or a comparison of the children's
  losses, and every loss comparison has lean and certainty fixed points. So routers train label-free, and error
  enters only as the accept/reject test of a split on fresh data: a boosting loop, not backprop. Design A (now): data-
  threshold splits accepted by a fresh-scored merge gate, an NB cluster router trained on one window then frozen,
  hard routing, copied leaves with inherited AdaGrad sums (S1 dropped). Design B (growth, later): an additive
  conditioning DAG of frozen NB routers with shrunk per-router delta tables over one shared base.
- Doors it would close: LR error routers (one S2 test excepted); Best, Soft and Balanced; error routing as router
  training; LR cluster routers; TAN over NB; router splits, home DAGs, path cells; the stall and T4 triggers;
  copies beyond one split without a fresh gate; forgetting at splits; stream-scored merge gates.
- The post-split dip is a ply-2 play effect (not the leaves, not ply 1). Ranked causes: across-leaf comparisons
  inside the minimum over replies (a phase-changing reply is scored by another leaf with its own offset), gate
  churn, own-ply-1 calibration of a fresh split. A cheap arena test is drafted.
- Scale (THEOREM, section 6.3): a 30-100 Elo routing gain at ply 2 is 15-50 at ply 1, inside the four flat seeds'
  spread (25) at ~700 games a cell, so a pure ply-1 league can't resolve the routing effects measured so far.
  Earlier ply-1 tree gains belonged to ply-2-trained evaluators. Ply 1 screens only large effects (leaf kind, data,
  a 40-Elo loss to repair) and mechanisms; routing growth must be judged at ply 2. Plateau floor: 200 cycles at
  ply 1 and 300 at ply 2 for tree lines.

**Theory 2, overnight diagnostics** (11-11 00:20-00:50, theory-2/summary.txt section 6.6, ~6 CPU-min):
- (b) growth-limits-p1 at ply 1: each limit-4/8 line's score against the field, grouped by the leaf count of its
  playing version (the stream gate made them oscillate 3-5-7-9-7-5-3). Per bucket, 12,000-20,000 games, ±5-6 at 95%:
  3 leaves -0.6 / 1.5 / 0.0 / 0.3; 5 leaves -4.9 / -0.5 / -4.0 / 1.6; 7 leaves 3.6 / -0.5 / 1.4 / 2.4; 9 or more
  -4.5 / -3.0 / -3.8 / -4.2. EVIDENCE: at ply 1 a line plays the same with 9 leaves as with 3, and the gate's churn
  costs nothing visible.
- (c) small-field-x4-p1's final trees (T-L-Tc-merge #1-4, v189), read with `routing` on held-out subsets: all four
  seeds route identically (shares 35.4 / 24.1 / 40.5, routing regret 0.0000). Opening (24+ pieces) 91.5% to one
  child, middlegame 92.1% to another, endgame (under 12) 83.0% to the third; material balance follows phase. The
  ply-2 single-split tree (T-L-Tc v999) has the same partition with a different boundary. EVIDENCE: the ply-1 trees
  found the phase partition and specialised their leaves, yet gained nothing in ply-1 play. So the phase split's
  gain, like the dip, shows only where the search compares positions scored by different leaves: ply 2.
- (a) The dip arena (plans/diag-dip.json): residual-leaves' copy tree and F-L at v60/90/120/150 (the dip: the tree
  trailed by 32-47 at ply 2) and v300/350/399 (level), each at ply 1 and ply 2, 15 double round-robins, games saved
  for the seam statistic. Running from 11-11 00:19 as leagues/arenas/diag/dip (the four standard references, not
  random-3 alone: arena.py always passes --anchor random). Result (section 8): the dip is a ply-2 effect (-50 ±33
  against ply 1), but the seam statistic doesn't track it.
- P3, the case for a ply-2 gate experiment (fresh against stream merge gate, ~5.5 CPU-h), is in
  theory-2/plans/P3-case.txt, for the user to decide (ply 2 is paused).

---

## 11. Early results (2026-11-05/06)

Where they live: LEDGER.md is the options ledger of those days, with verdicts and every arena number (a status block
at its top says what changed since). Per-run notes are in archive README.md ("High-watermarks" and "Finished later");
the run folders are listed in README.md. Highlights:
- **The evaluator.** LR beat NB by about 370. Depth-2 measurement is the standard.
- **Data.** Better games are the strongest lever: depth-2 search self-play from scratch was 93 over the jitter plateau
  by v50, and 392 over CNBP v2000. It was deferred for cost.
- **Forgetting.** It helped NB only after very long accumulation (÷16 at v2000: 55). From scratch every form hurt or
  tied. LR barely reacts.
- **Trees from a plateau.** T-L-Nc beat its LR control by 29 and 30 (d1 32 and 21). T-L-Ne with log error by 27 and
  23. LR cluster routers collapsed.
- **Trees from scratch** (batches 29-32, 1000 rounds): every tree lost to its flat model until merges.
  - The archetype tournament (selfplay/tournament-2026-11-06, 37 players) at d2: T-L-Ne merges 1457, F-L 1431, T-L-Le
    1427, T-L-NLs 1423, T-L-Nc 1421, T-L-NNs 1415, T-L-LLs 1408, MoE 1282, F-N 1173, T-N-Nc 1107, T-N-NNs 1101,
    T-N-Ne 1054.
- **Rounds.** v1000 against v750: F-L 69, T-L-Nc 53, merging T-L-Ne 30, F-N 10. F-L peaks at v1000: v2000 against
  v1000 is -25.
- **MoE** (11-05/06) was neutral or worse then (from scratch -205 against F-L). Later M-L4-N-c showed what an NB gate
  does (section 3).

High-watermarks by approach, 2026-11-05 (d2, against the named reference; archive README):

| Approach | Model | Result |
|---|---|---|
| CNBP, jitter self-play | ladder-cnbp-long v2000 | plateau from ~v500 |
| CNBP + one-time forgetting (the ÷16 baseline) | cnbp-from2000-if16 v500 | 55 [45, 66] over v2000 |
| CNBP tree, error routing | bdt3-from2000-t5M-l5-f2 v500 | 15 [5, 26] over v2000 (the forgetting at splits) |
| CNBP tree, cluster routing | bdtc3-f2-s13 v500 | 39 [29, 50] over v2000 (s11 31, s12 35); s11 against ÷16 -27 |
| CNBP DAG, cluster routing | bddc3-f2-l10 v500 | 25 over v2000; -18 [-28, -8] against ÷16; no router split fired |
| LR, jitter self-play | lr-jit-s1 v1000 | 374 over CNBP v2000 |
| LR, 50/50 depth-2 + result labels | lr-lab-d2-r0.5-s1 v1000 ("mix1000") | 48 over result-label LR v1000 |
| LR, depth-2 search self-play from mix1000 | lr-d2play-T0.3-from-mix1000 v100 | 81 over its LR control; 392 over CNBP v2000 |
| LR, depth-2 search self-play from scratch | lr-d2play-T0.3-scratch v300 | v50 93 over mix1000; v300 level with the row above (8 [-2, 19]) |
| Mixture of LR experts | moe4-from-mix1000 v100 | 2 / 23 over its control (two arena seeds) |
| LR tree, LR routers | lrtree3-from-mix1000-f1 v300 | -1 [-11, 10] against its control |
| LR tree, CNBP cluster routers | lrtree-c3-nbr v300 | 29 [18, 40] against its control; seed 22 30 |
