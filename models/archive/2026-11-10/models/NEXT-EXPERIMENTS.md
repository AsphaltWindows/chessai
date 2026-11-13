# Next experiments: proposals (written overnight 2026-11-09/10, while the user slept)

The user's brief before bed: ideate future experiments, keep in mind what we're testing and control the variables,
work towards "compound models with routing" (trees, MoEs, DAGs: the user clarified that's what "compound DAGs"
meant), and look back at last night's work for inspiration. Nothing here is launched or created. Every line list is a
proposal for the user to edit. Drafted specs live in leagues/specs/batch3/ and were checked with make_config only
(which writes nothing).

## 1. What we're testing

- The project's aim: a broad model that uses only a small part of its weights per position, by splitting chess into
  sub-problems (routing). So the central question is: **can a compound model with routing beat a flat model of its
  own leaf type, on equal data, and stay compound (not collapse to one leaf)?**
- The new league separates the questions the old one blended (the user's rethink, 2026-11-09):
  - A. strongest by self-play;
  - B. strongest on the same data (local lines: every line trains on the experiment's qualifying games);
  - C. is deeper-search data worth its CPU, and from when;
  - D. which model generates the best training data.
  Architecture questions are B questions, so the routing experiments below are local-only.

## 2. Rules for controlling variables (lessons from batches 1-2)

1. **One factor per comparison**, against a named comparator. Seeds count as a factor. In a local experiment, lines
   without replicates share the train seed, so two lines that differ only in a play-time flag hold identical weights
   (checked for M-L4-L and M-L4-L-hard: byte-identical after two training rounds).
2. **Only the lines the user asks for.** No replicates or extra conditions slipped in: in a local experiment every
   line's games are every local line's training data, so one extra line shifts all of them.
3. **Field shape.** The share of training games against the random references depends on the number of lines n:
   8n / (C(3n+4, 2) - C(n+4, 2)) = 42% (n=3), 35% (4), 30% (5), 26% (6), 23% (7), 13% (14). The user's rule for
   per-family experiments is 4 lines (35%), so every proposal below has 4 lines.
4. **Same tree rules unless the rule is the factor.** Split limit, threshold, children, error measure, forget
   factor and merge setting must match across tree lines (the --error abs/log mismatch in single-split was exactly
   this kind of hidden second factor).
5. **Unpriced compute is a factor.** Training CPU is not priced; recipes that spend extra training compute (search
   labels: 12x) get it free. Price it or keep it out.
6. **Noise.** One version's strength swings about 17 Elo (SD) and windows of versions drift about 30. Compare pooled
   windows of ratings (e.g. the mean over the last 50 cycles), and treat gaps under ~40 as unresolved unless they hold
   in a second window.
7. **Check that a compound model is still compound.** Report routing or gate shares and leaf distinctness at
   checkpoints. A model that collapsed to one leaf is testing a flat model, not routing.

## 3. Where compound routed models stand

Sources: experiment-log memory; models/from-scratch/{twin, whytrees, fieldpanel, degeneracy, bddfix, bddtrig,
resid}/summary.txt.

What failed, and why it matters:
- **The old league's tree edge was data, not routing.** A flat LR trained on T-L-Ne-share-c's exact stream (its
  self-play and league games) matched or beat the tree: tree minus twin, pooled over v444-v452, was -26 [-42, -10].
- **No extra routing value against the field** (field panel): error trees against their own busiest leaf, scored
  against 12 field entries, were -3 (T-L-Ne-c) and -32 (T-L-Ne-share-c).
- **Degeneracy.** At cycle 71, 25 of 64 routed or mixture lineages were functionally flat: LR routers collapse to
  one leaf; every LR-leaf tree with --leaf-share converged to near-copy leaves; 18 of 20 merge lineages were flat.
  NB-routed trees stayed healthier.
- **Hard boundaries cost play at depth 2 (seams).** On 11-05, NB-leaf trees lost 92-259 to their unsplit control at
  ply 2 but only 2-28 at ply 1. A search compares sibling positions, and when siblings go to different leaves it
  compares numbers from different models. Later LR-leaf trees showed seams on 5-7% of moves.
- **Things tried and rejected (not proposed again):**
  - Residual leaves (11-07): 171-266 below the plain tree in play at v60.
  - Soft (posterior) error targets with LR leaves (11-05): the routers learned to route by who's winning.
  - A larger step for leaves: the pooled Q6 recheck (11-09 05:43) found the larger step 46 below the twin.
    Random 1/0.1 data weighting was 15 above at either step size, and the router-region effect fell within noise.
  - LR routers (they collapse) and merges for error routers (the user's ban, 11-08).
  - Route-once play (the user: misguided).
  - DAG router splits as built: five variants, never better than a same-leaves tree offline. After a split, R'
    passes positions through: its copies agree on ~97% of positions (live D-L-Ne-c, v140).

What gives hope:
- **Final, 2026-11-10 11:00 (single-split and single-split-probit ran to cycle 1000; direct current-against-current
  games, see "Final check" at the end of this item):**
  - LR leaves: the NB and TAN cluster trees led flat by ~70-100 from cycle ~200 to 1000, saturated.
  - Probit leaves: the cluster trees led by 75-77 at the end and were still climbing. The NB error tree led by
    44-47, the first error tree to beat flat clearly.
  - In cluster-error (to cycle 415) the probit cluster trees lead (TAN ~45-50, NB ~25), but the LR-leaf ones do not
    (-25 to 26).
- **Update 2026-11-10 05:40 (first seen 02:20): cluster-routed single splits beat flat on equal data in all three
  experiments. With LR leaves the lead stopped growing at ~80-90, and its size depends on the experiment.**
  - **Source:** scratchpad traj/quick.py. It reads history.csv (current@2 per cycle) and tour/ratings.csv (every
    version's final rating), small files with stable reads. The tables below the 05:40 bullets are from 04:00.
  - **single-split (to cycle 527): the LR-leaf cluster trees have saturated too, ~80-90 above flat.**
    - Version bins on today's scale, v201-240 through v481-520:
      - T-L-Nc-merge: 1397, 1406, 1413, 1402, 1421, 1421, 1401, 1420;
      - T-L-Tc-merge: 1385, 1406, 1411, 1409, 1417, 1420, 1415, 1401;
      - F-L: 1333, 1326, 1334, 1313, 1351, 1326, 1339, 1330.
    - Trajectory leads at 361-527: 64-96.
    - So a single cluster split bought a fixed step, about 80-90, and was used up by cycle ~280. A fixed number of
      leaves predicts exactly that. Whether more splits keep paying is P7's question.
  - **single-split-probit (to cycle 534): every tree still climbs, and flat does not.**
    - T-P-Nc-merge went from 1274 to 1326 (v201-240 to v521-533), and T-P-Tc-merge from 1261 to 1306.
    - F-P stays at ~1270.
    - Trajectory leads at 481-534: T-P-Nc 27-51, T-P-Tc 16-31.
  - **cluster-error (to cycle 183): the LR-leaf cluster lead is small.**
    - T-L-Tc-merge leads F-L by 5-32 at cycles 141-183, and T-L-Nc-merge by -9 to 19. At the same cycles
      single-split led by 36-48.
    - cluster-error's lines train on 8,820 games a cycle against single-split's 2,450, so it is not behind in data.
      The 04:00 note that it "follows single-split's course ~250 cycles later" is withdrawn.
    - Its probit cluster trees lead F-P by ~30-50 (bins 1650-1671 against 1619-1622 at v141-180), and F-L is still
      rising there.
  - **Reading:** the cluster advantage is positive everywhere, but its size depends on the data stream:
    - about 80-90 with LR leaves in single-split;
    - 20-50 with probit leaves, in both experiments;
    - 5-15 with LR leaves in cluster-error.

  - **07:10 check (single-split and probit to cycle 664, cluster-error to 242):**
    - single-split is unchanged. Both cluster trees hold ~80 above F-L (bins ~1398 and ~1400 against ~1318 for
      v321-664), saturated since ~cycle 280.
    - single-split-probit: every probit tree keeps climbing while F-P stays flat (~1268).
      - From v321-360 to v641-663: T-P-Nc-merge 1272 to 1323, T-P-Tc-merge 1272 to 1302-1313, T-P-Ne 1265 to
        1289-1295.
      - Trajectory leads at 641-664: T-P-Nc 70, T-P-Tc 45, T-P-Ne 30.
      - So with probit leaves even the NB error tree now beats flat (16-30 since cycle 521). With LR leaves no
        error tree ever has.
    - cluster-error is unchanged: an LR-leaf cluster lead of 5-15, and probit cluster trees 20-40 above F-P.
  - **08:40 check (single-split to cycle 823, probit to 818, cluster-error to 317):**
    - **single-split is unchanged:** the cluster trees hold ~80 above F-L.
      - Trajectory leads at 641-821: T-L-Nc 67-93, T-L-Tc 58-101.
      - Bins v641-820: T-L-Nc 1383-1391, T-L-Tc 1373-1398, F-L 1295-1318.
      - T-L-Te may be sliding: bins 1287-1294 at v481-560, 1260-1271 at v761-820, while F-L held. The slide is
        inside the ~30 window drift we have seen before.
    - **single-split-probit: the cluster trees may be levelling off, while the error trees still climb.**
      - T-P-Nc-merge bins were 1320-1340 from v561 on (1308 at v481-560); trajectory leads at 641-815 were 45-70.
      - T-P-Tc-merge bins were 1307-1337 from v601 on; leads 35-53.
      - T-P-Ne went from 1280-1288 (v481-560) to 1317 (v801-815); its lead went from 16-25 to 34-38.
      - T-P-Te went from 1234-1250 to 1267-1274 and is now about level with F-P (-11 to 11 at 721-815).
      - F-P stays at 1263-1282.
    - **cluster-error: the LR-leaf cluster lead is gone.**
      - At 241-315 the T-L-Nc twins average ~0 against F-L and the T-L-Tc twins ~-2 (windows -22 to 19).
      - Bins v241-315: T-L-Nc 1621-1645, T-L-Tc 1618-1638, F-L 1617-1642.
      - The probit cluster trees hold, ~30-40 above F-P. Bins at v241-315: T-P-Tc ~1657, T-P-Nc ~1646, F-P ~1616,
        F-L ~1633. So the probit TAN cluster tree is the strongest line there, ~25 above F-L.
    - **Routing (routing_health.py; held-out shares per leaf at the current version):**
      - single-split v828: the NB and TAN cluster trees split 56/19/25 and 53/21/26. T-L-Lc-merge has merged back to
        1 leaf.
      - single-split-probit v823: T-P-Nc 29/16/55, T-P-Tc 30/20/51. T-P-Lc-merge has 1 leaf. T-P-Ne has an empty
        leaf (53/47/0), and T-L-Le has one too (73/27/0).
      - **The LR cluster routers churn and are effectively flat.** This comes from a version-by-version trace
        (scratchpad shares/shares.py, the same `routing` command).
        - In cluster-error, T-L-Lc-merge and both T-P-Lc twins follow the same path:
          - from v100 to v305 they had 1 leaf, or 3 leaves with every held-out position sent to one;
          - they split 24/38/38 at v310 and collapsed to one leaf by v314;
          - they merged by v318, split again (26/38/36) at v319, and were back to 1 leaf at v321.
        - The T-L-Lc-merge+error=log twin sent everything to one leaf at every version.
        - In single-split, T-L-Lc-merge split 30/47/24 at v200 and has had 1 leaf at v400-v828.
        - So the LR-cluster lines remain flat controls.
        - Withdrawn after a few minutes: an 08:45 line here read the single balanced snapshot at v319 as "a
          balanced split buys nothing". The versions it was rated on were flat.
      - **The NB and TAN cluster partitions are fixed once made.**
        - cluster-error: identical shares from v150 to v321, e.g. T-L-Nc 28/16/56 and T-P-Tc 28/57/15.
        - single-split: T-L-Nc 55-56/19-20/25 and T-L-Tc 52-53/21-23/25-26 from v200 to v828.
        - The twins share one partition. The probit twins differ only in which leaf is which.
        - So in cluster-error, the LR-leaf lead faded (5-15 earlier, ~0 now) under a partition that never changed.
          The fade came from the leaves and the data stream, not from re-routing. The two experiments' partitions
          differ (T-L-Nc 56/19/25 in single-split, 28/16/56 in cluster-error).
      - **The leaves are not near-copies in either experiment, so leaf collapse does not explain cluster-error's
        missing LR-leaf lead.**
        - Method: leaves exported with chessai-whytrees `leaves`. Weight-vector correlations as in
          whytrees/analysis/weights.py, in scratchpad leafsim/sim.py.
        - Leaf-pair correlations, win-loss / draw:
          - T-L-Nc at v321 in cluster-error: 0.94-0.98 / 0.52-0.65 (0.94-0.98 / 0.49-0.60 at v150);
          - T-L-Nc at v830 in single-split: 0.91-0.98 / 0.40-0.66 (0.91-0.98 / 0.47-0.58 at v300);
          - T-L-Tc: 0.94-0.98 / 0.56-0.67 in cluster-error, 0.93-0.98 / 0.49-0.71 in single-split.
        - In both experiments the largest leaf is most like F-L (0.99 / 0.92-0.95). The 16-21% leaf differs most
          (0.93-0.96 / 0.58-0.66). So the trees have the same shape in both experiments. Why the lead is ~80 in one
          and ~0 in the other is still open.
      - **Every NB and TAN cluster tree found the same partition, by game phase.**
        - Method: per-position routing on heldout-t01's 181,626 test positions (chessai-whytrees `leaf-eval`,
          column owntop). Agreement is under the best matching of leaf labels. Scratchpad leafsim/agree.py and
          what.py.
        - All nine cluster trees route 88-99% of positions alike. That covers the LR- and probit-leaf trees of all
          three experiments, NB and TAN routers, and T-L-Nc at v300 and v830. Chance, with these shares, would be
          ~54%.
        - The three regions (all positions, so 70% endgame, because heldout-t01 is mostly long drawn endgames;
          56/19/25 on decisive positions):

          | Region | Share | Pieces (no kings) | Pawns | Queens | Castling rights | Material |
          |---|---|---|---|---|---|---|
          | opening | 10-12% | 28 | 15 | 1.4-1.5 | ~2 | 70-72 |
          | queenless middlegame | 18-19% | 20 | 10 | 0.2-0.3 | 0.3-0.4 | 45-47 |
          | endgame | 70-71% | 5 | 2 | 0.55 | 0 | 16 |

        - The error routers also split off the endgame (69-73%) but cut the rest differently: 79-87% agreement with
          the cluster partition. T-L-Ne's two other regions have similar piece counts (24 and 23). T-P-Ne has just
          endgame against the rest.
        - So the cluster routers learned game phase with no hand features. The ~80 against ~0 contrast between
          single-split and cluster-error is not a different partition: it is the same one, with leaves trained on
          different streams.
        - Direct head-to-head (tour/head_to_head.csv, ply-2 versions in the window; rough intervals):
          - single-split: T-L-Nc against F-L scored 63% and 62%, i.e. 94 and 83, at v401-600 and v601-830; T-L-Tc
            73 and 97.
          - cluster-error: T-L-Nc -18 and 17 (v161-240, v241-330); T-L-Tc 8 and 9. T-P-Tc beat F-P by 33 and 81.
          - So the ratings' gaps are real direct gaps, not effects of the field.
          - These version windows also count a line's champion when its version falls inside the window, which
            favours that line. The final check below uses current-against-current games instead.
        - The decisive rate does not explain it either. Games between the experiment's own lines (tour/games.csv,
          last 300k rows) are 91% decisive in both experiments. cluster-error's games are shorter: 96 plies against
          108.
        - Nor does the phase mix of the training stream.
          - Method: one cycle's local set captured by hard links (single-split c891, 246k positions; cluster-error
            c351, 786k), routed through the phase partition (models/from-scratch/phase-mix/mix.py).
          - Shares, endgame / opening / middlegame: 29 / 24 / 47% in single-split, 28 / 28 / 44% in cluster-error.
          - The results per phase are alike too: draws are 19-21% of endgame positions and 3-6% of the others.
          - Ruled out so far: the partition, near-copy leaves, the decisive rate, the phase mix.
          - Left: 3.6 times more positions per cycle in cluster-error; who generates the games (single-split's
            stream includes three weak error trees); and chance, with one run per experiment.
      - **Error routers drift early and then settle too.**
        - T-L-Ne went from 50/34/17 at v100 to 58/26/16 at v400, and was 59/25/16 at v830.
        - T-L-Te went from 46/36/19 to 52-53/29-31/17-18.
        - T-P-Te went from 43/29/28 to 43-45/15-17/38-42 by v300.
        - T-P-Le drifts most: from 4/55/41 at v100 to 16-20/20-22/60-62 at v600-v800.
        - T-P-Ne was 51-54/46-49/0 at every version from v100: one leaf has never been used.
        - So T-P-Ne's climb after v500 (from ~20 to 34-38 above F-P) happened under a settled 2-way split. It came
          from the probit leaves still filling their regions. Flat probit, with the same data, stays level.
      - P6 should log leaf shares every few cycles, so we know how old a partition is.
  - **Final check, cycle 1000 (single-split ended 10:51, single-split-probit 10:56; cluster-error to cycle 415):**
    - **A cleaner direct measure: current against current.** It counts only games between two lines' current ply-2
      versions (tour/games.csv; scratchpad xtab/curtab.py). The version windows used above (head_to_head.csv,
      h2h.py) also count a line's champion whenever its version falls inside the window. A champion is a selected
      version, so that line looks stronger.
      - Example: T-L-Lc's champion v876 put it 29 above F-L at v831-999. Current against current it was -1 to -4.
      - The 08:40 direct numbers carry some of this: T-L-Tc's 97 at v601-830 includes its champion, v746.
    - Direct Elo against the experiment's flat line, by cycle window. Cells hold 1000 games (2000 in the 200-cycle
      windows), about ±22 (±16) at 95%:

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

    - **LR leaves (single-split): the cluster lead formed by cycles 200-300 and held at ~70-100 to the end.** No
      error tree ever beat flat.
    - **Probit leaves (single-split-probit): the trees were still climbing at cycle 1000.**
      - The cluster trees started below flat, crossed it around cycle 300 and reached 75-77.
      - T-P-Ne reached 44-47 above flat, the first error-routed tree in these experiments to beat flat clearly.
      - T-P-Te rose from -57 to about -10.
    - **cluster-error by the same measure** (twins as plain / +error=log):

      | Line | 1-100 | 101-200 | 201-300 | 301-415 |
      |---|---|---|---|---|
      | T-L-Nc (against F-L) | 14 / -2 | 3 / 6 | 9 / -13 | -25 / -19 |
      | T-L-Tc | 10 / 18 | 26 / 17 | 5 / 19 | 15 / 2 |
      | T-P-Nc (against F-P) | 29 / 24 | 25 / 41 | 25 / 25 | 21 / 25 |
      | T-P-Tc | 37 / 54 | 51 / 64 | 58 / 41 | 45 / 43 |
      | F-P (against F-L) | -83 | -26 | -10 | -25 |

    - **So the two leaf types react in opposite ways to cluster-error's stream.** There the LR-leaf cluster trees never
      formed a lead, while the probit cluster trees led within the first 100 cycles. In single-split-probit the
      probit trees needed ~300 cycles to cross flat.
      - Data volume alone does not obviously fit. cluster-error trains on 8,820 games a cycle against 2,450 (3.6×;
        3.2× in positions). By games seen, its cycle 100 matches single-split's ~360, where the LR lead was already
        ~80, yet the lead never formed there by any clock.
      - Left: what the stream is made of, and chance (one run each). In cluster-error half the lines play with probit
        leaves and every line trains on everyone's games. In single-split all lines play with LR leaves.
      - P6 is a partial replicate of single-split (3 of its 7 lines). If its T-L-Nc-merge again leads F-L by ~80,
        chance becomes unlikely.

  The 04:00 tables. single-split (LR leaves, to cycle 385) and single-split-probit (probit leaves, to cycle 394):

  | Line | 121-160 | 161-200 | 201-240 | 241-280 | 281-320 | 321-360 | 361-385/394 |
  |---|---|---|---|---|---|---|---|
  | T-L-Nc-merge (NB cluster router) | 36 | 48 | 62 | 81 | 73 | 92 | 68 |
  | T-L-Tc-merge (TAN cluster router) | 36 | 41 | 53 | 80 | 68 | 95 | 58 |
  | T-L-Lc-merge (LR cluster router; 1 leaf in use, so flat) | -6 | -2 | -1 | -3 | -3 | 10 | -7 |
  | T-L-Ne (NB error router) | -22 | -33 | -16 | -1 | -9 | 7 | -41 |
  | T-L-Te (TAN error router) | -39 | -47 | -39 | -28 | -39 | -12 | -50 |
  | T-L-Le (LR error router, a leaf starved) | -88 | -106 | -96 | -78 | -82 | -72 | -128 |
  | T-P-Nc-merge | 9 | 20 | 10 | 20 | 24 | 23 | 32 |
  | T-P-Tc-merge | -21 | -20 | -6 | -1 | -2 | 12 | 23 |
  | T-P-Lc-merge (1 leaf, so flat) | 7 | 9 | 14 | 0 | 8 | 6 | 3 |
  | T-P-Ne (a leaf starved) | -1 | -11 | -7 | 3 | 8 | 12 | 4 |
  | T-P-Te | -61 | -62 | -59 | -47 | -42 | -40 | -49 |
  | T-P-Le | -84 | -88 | -99 | -105 | -89 | -85 | -90 |

  cluster-error (to cycle 130; abs / log twins; T-L lines minus F-L, T-P lines minus F-P):

  | Line | 41-60 | 61-80 | 81-120 | 121-130 |
  |---|---|---|---|---|
  | T-L-Nc-merge | -1 / 5 | 34 / 35 | 3 / 7 | 3 / 12 |
  | T-L-Tc-merge | 14 / 13 | 47 / 47 | 19 / 21 | 19 / 23 |
  | T-L-Lc-merge (flat) | -1 / 7 | 5 / 3 | -2 / -4 | 21 / -20 |
  | T-P-Nc-merge | 27 / 44 | 50 / 52 | 38 / 31 | 69 / 76 |
  | T-P-Tc-merge | 19 / 35 | 44 / 51 | 38 / 40 | 49 / 74 |
  | T-P-Lc-merge (flat) | 6 / 17 | 8 / 2 | 4 / -1 | 29 / 30 |

  - **LR leaves:** the NB and TAN cluster trees lead F-L by 68-95 over cycles 241-360, up from 36-48 at 121-200.
    (Withdrawn at 05:40: "cluster-error runs ~250 cycles behind and follows the same course". Its 81-120 lead,
    3-21, did match single-split's at 81-120, 15-22, but it has not grown since.)
  - **Probit leaves:** a smaller lead in single-split-probit (T-P-Nc-merge 20-32 since cycle 241; T-P-Tc-merge
    crossed above flat at ~cycle 320). In cluster-error the probit cluster trees lead F-P by 31-40 at 81-120 and
    their flat-like twin T-P-Lc-merge by ~35. Its 121-130 window is short, and F-P's current version sits low there.
  - **Error trees:** T-L-Ne reached flat at 241-360 (-9 to 7) and T-P-Ne sits at 3-12. The TAN and LR error trees are
    still 40-130 below.
  - **Noise:** cluster-error's T-L-Nc and T-L-Tc twins are still identical models (same leaf shares at cycle 131).
    Their 40-cycle windows differ by 2-4 and their 10-cycle windows by up to 27. The T-L-Lc twins have diverged
    (each sends 100% of held-out positions to a different leaf), hence 21 / -20.
  - **Flat has saturated; the cluster trees still climb (04:00; by 05:40 the LR-leaf cluster trees had levelled off
    too, see above).** This is the clearest view. It shows the mean final ply-2 rating of the versions in each
    40-version bin, with every version on the 04:00 scale (single-split to v384):

    | Line | 81-120 | 121-160 | 161-200 | 201-240 | 241-280 | 281-320 | 321-360 | 361-384 |
    |---|---|---|---|---|---|---|---|---|
    | F-L | 1367 | 1351 | 1360 | 1348 | 1342 | 1351 | 1330 | 1372 |
    | T-L-Nc-merge | 1379 | 1394 | 1409 | 1412 | 1422 | 1430 | 1419 | 1439 |
    | T-L-Tc-merge | 1382 | 1393 | 1402 | 1400 | 1422 | 1428 | 1426 | 1433 |
    | T-L-Ne | 1325 | 1339 | 1335 | 1333 | 1342 | 1341 | 1333 | 1331 |
    | T-L-Te | 1311 | 1321 | 1318 | 1316 | 1316 | 1310 | 1320 | 1325 |
    | T-L-Le | 1266 | 1268 | 1260 | 1257 | 1264 | 1270 | 1263 | 1245 |

    - F-L has not improved since cycle ~80, and T-L-Lc-merge tracks it. The error trees are flat too.
    - The NB and TAN cluster trees gained 50-60 over the same 300 cycles of the shared data stream.
    - Probit leaves, single-split-probit (from v81-120 to v361-393):
      - F-P went from 1297 to 1268, flat or slightly down;
      - T-P-Nc-merge went from 1256 to 1300, and T-P-Tc-merge from 1235 to 1293;
      - T-P-Ne stayed at ~1270.
    - This is the growth story in miniature. On the same data, flat saturates early, and the label-free 3-leaf trees
      turn more data into strength before they saturate in turn (05:40). The error trees have the same 3 leaves and
      the same data, yet saturate at flat's level.
    - In cluster-error (to v130) everything still climbs. Its probit cluster trees are the top lines at v121-130
      (1674-1697, against F-P 1635 and F-L 1650).
  - **No real late dip.** In the trajectory table, every single-split tree fell 20-50 against F-L at cycles 371-385.
    That came mostly from F-L's recent versions rating high (1372 for v361-384, against 1330-1351 before). The cluster
    trees did not fall: v361-384 is their best bin.
  - **Champions mislead here.** F-L's champion (v150, 1417) and F-P's (v222, 1322) are lucky versions. Both are
    well rated by now, and they sit 57-70 above their lines' mean recent version. Champion against champion, the NB
    and TAN cluster trees are 56-57 above F-L but 21-32 below F-P. The mean over many versions is the fairer
    measure of a recipe. Separately, real version-to-version variation exists: a champion's rating is settled, yet
    it stands well clear of its line's mean.
  - **Routing, cycle 387/396:** unchanged. The NB and TAN cluster trees use all 3 leaves: 55/19/25 and 52/22/26 with
    LR leaves, 29/16/55 and 28/20/52 with probit leaves. T-L-Le and T-P-Ne still starve a leaf (0.1%, 0.0%).
    T-L-Ne's third leaf holds 17%. The LR cluster routers send 100% to one leaf.
  - All of these crossed above flat after about 40-120 cycles, the order of the break-even in theory 3.5.
  - This is the first clean equal-data win over flat that we have. Per the version-noise rule, a 20-40 contrast
    needs 2+ windows. With LR leaves this one is now 60-95 in five 40-cycle windows, and it shows in all three
    experiments.
  - **Confound: two factors differ, not one.** The cluster lines are T-L-Nc + --revise 0, a merge test that undoes
    a split that doesn't pay. The error lines have no merges (the user's ban for error routers). So the win could
    come from cluster routing (theory section 8, item 2: label-free routers don't learn who is winning) or from the
    merge test keeping only splits that help, or both.
  - **One-factor follow-up (P6, below):** F-L, T-L-Nc, T-L-Nc-merge and T-L-Ne, all with split limit 1.
    - T-L-Nc against T-L-Nc-merge isolates the merge test.
    - T-L-Nc against T-L-Ne isolates cluster against error routing.
- **M-L4-N-c** (4 LR experts, NB gate) uses all 4 experts, gates almost hard (68% of positions above 0.99), and
  beats its own best expert by 73 at ply 1 and 46 at ply 2.
- **T-L-Nc-share-c** beat its best leaf alone by 40 at ply 2, the only tree that did.
- **New league, equal data:** every error-routed tree has split once and uses all 3 leaves. Shares of held-out
  decisive positions (games/heldout-t01) at cycle 30-32, 2026-11-09 22:50:

  | Router | LR leaves (single-split) | Probit leaves (single-split-probit) |
  |---|---|---|
  | LR | 51/38/11% | 6/60/34% |
  | NB | 40/35/25% | 48/26/26% |
  | TAN | 34/38/29% | 40/15/46% |

  The cluster trees with merges flip-flop as expected: at this moment the LR-leaf ones and T-P-Lc-merge are unsplit,
  while T-P-Nc-merge and T-P-Tc-merge have 3 leaves (8/28/64%, 8/32/60%). Too early for ratings. The routing
  command's recalibrated losses overflow on these models, so only shares are usable. The check is
  models/from-scratch/routing-health/routing_health.py EXPERIMENT... (read-only, about a minute of CPU).
- **First ratings on equal data (2026-11-10 00:25): one leaf split costs a lot, then slowly recovers.** Tree minus
  its flat twin, mean of current@2 over cycle windows (the trees split at cycle ~12; before that they play like the
  flat line, and the -14 to 5 spread in cycles 1-10 is the noise between near-identical players):

  | Line | 1-10 | 11-20 | 21-40 | 41-60 | 61-80 | 81-now (88 or 94) |
  |---|---|---|---|---|---|---|
  | T-L-Le (LR router) | 0 | -45 | -148 | -148 | -136 | -117 |
  | T-L-Te (TAN router) | -11 | -37 | -100 | -89 | -82 | -68 |
  | T-L-Ne (NB router) | 1 | -27 | -70 | -55 | -42 | -20 |
  | T-P-Te (probit leaves) | 9 | -8 | -18 | -66 | -77 | -66 |
  | T-P-Ne (probit leaves) | -3 | -10 | 6 | -43 | -43 | -39 |

  The cluster-merge lines sat within about 25 of their flat twins then, since they were mostly unsplit
  (flip-flop). By cycle ~120 they had split and moved above flat (see the update above).
  Routing health at cycle 87-93: T-L-Le and T-P-Ne have each starved a leaf (0% of held-out positions). The other error
  trees still use all 3 leaves, but their third leaf is shrinking (T-L-Ne 25% -> 17%).
  Reading: on equal data a single leaf split loses 70-150 at first. With NB routers the loss closes over ~80 cycles,
  consistent with the user's point that leaf splits fragment the data. Seams may add to it; P1 measures them.

## 4. Running now, and what's left (updated 2026-11-10 11:03)

| Experiment | Lines | Cycle | Cycle wall time | Left (wall) |
|---|---|---|---|---|
| cluster-error | 14 | 423 of 1000 | ~45 s since it runs alone (was ~74 s) | ~7.2 h (about 18:15) |
| split-growth (P7, launched 11:39) | 5 | 190 of 400 at 12:27 | ~15 s | about 13:20 |
| split-growth-probit (P7b, launched 12:31) | 5 | 5 of 1000 | 17-27 s so far | ~5 h (about 18:00), an early estimate |
| single-split | 7 | done (1000 of 1000, 10:51) | | |
| single-split-probit | 7 | done (1000 of 1000, 10:56) | | |
| selfplay-depth | 13 (self-play) | done (100 of 100, 06:52) | | |
| lr-settings-2, nb-settings-2, probit-settings, leaf-sources-2 | | done | | |

The machine has 8 cores. Load fell to ~4 when the single-split pair ended (~7 before, ~12 before 06:52), so about four
cores are free. Nothing new has been launched: new experiments wait for the user (§7).

- **selfplay-depth final (cycle 100).** Mean final ply-2 rating of versions v81-99 (tour/ratings.csv; same priced
  self-play compute per line, so deeper search means fewer games):

  | Leaf | depth 0 | depth 1 | depth 2 | depth 4 |
  |---|---|---|---|---|
  | F-L | 1529 | 1459 | 1595 / 1616 (twins) | 1397 (still climbing) |
  | F-P | 1416 | 1565 | 1696 (still climbing) | 1224 (still climbing) |
  | F-N | 1260 | 847 (unstable) | 458 | 539 |

  - Depths 0 and 1 plateaued by v21-40. Depth 2 was still climbing at the end (probit 78 above its v21-40 bin).
  - Depth 4 climbs steeply but is far behind, starved of games. 100 cycles may understate it.
  - F-N d1 went 838, 568, 424, 654, 847 across the 20-version bins: it collapsed, then partly recovered.
  - Best depths for P4: LR 2, probit 2, NB 0. F-P at depth 2 is the strongest line in the experiment.

No errors in any repeat.log; every cycle since the last check finished, including those on the new train binary
(chessai-hybrid6, from 05:20; see the item further down).

- **selfplay-depth at cycle 67** (current@2 minus F-L+depth=2#1, mean over cycles 61-67; each line has the same
  priced self-play compute, so deeper search means fewer games). At cycle 85 (81-85) the ranking is unchanged:
  - LR: d0 -126, d1 -172, d2 twin 9, d4 -249;
  - probit: d0 -207, d1 -77, d2 20, d4 -450;
  - NB: d0 -369, d1-4 -855 to -1185.

  | Leaf | depth 0 | depth 1 | depth 2 | depth 4 |
  |---|---|---|---|---|
  | F-L | -88 | -145 | 0 / 20 (twins) | -317 |
  | F-P | -186 | -26 | 49 | -447 |
  | F-N | -354 | -1054 | -1135 | -1096 |

  - Self-play totals so far:
    - depth 0: ~4M games, 88-94% draws;
    - depth 1: ~3M games, 57-72% draws;
    - depth 2: ~0.2-0.3M games, 25-28% draws (NB 67%);
    - depth 4: 2-7 thousand games.
  - Depth 2 wins for LR and probit despite 13-18× fewer games than depth 0. Depth 4 is starved of data.
  - **NB at depth 1-4 has collapsed to near random**: the random references sit at -1278 to -1587. F-N+depth=1's
    champion is v11 and its current version is 478 below it. F-N+depth=2 never improved on v1.
    - In its own self-play each new version scores 50% against the previous one, as every line does.
    - Its games are lopsided by colour: White 28%, Black 16%, draws 57% at depth 1. LR at depth 2 has 35/37/28.
    - Cause unknown. Only depth-0 NB is healthy, so for P4 NB's best depth is 0.

- **cluster-error's paired lines are, so far, the same models.** At v45 each "+error=log" line differs from its
  twin only in the error-statistics lines of the model file (9 of 18,224 lines in the T-L-Nc pair). Routers and
  leaves are identical, so the pairs play identically. With one-hot labels, abs and log give the same Best target
  (theory 7.4). The measure acts only through split and merge windows, and none has diverged yet. Until one does,
  half the experiment measures noise. This adds to decision 3.
  - At cycle 131 the T-L-Nc and T-L-Tc pairs still have identical leaf shares. The T-L-Lc pair has diverged (each
    routes everything to a different leaf), and so have both T-P pairs.
- single-split and single-split-probit still run their cluster lines (abs split check), which cluster-error also
  covers.

**Fixed point for trees and DAGs: deployed** (2026-11-10 00:53-00:58, under the user's blessing for exact speedups).
- **Binary:** bin/chessai-hybrid5 (md5 3154f5a9...). It is chessai-hybrid4gfx with fixed-point play for calibrated
  trees and DAGs at ply 1 and 2. Inside the proven error bound it falls back to float, so every value that reaches
  the move choice is the float value. Write-up: models/from-scratch/fastbin/hybrid5/summary.txt.
- **Identity:** every check was byte-identical:
  - 17 arenas (2,260 games, 4 of them in a step-by-step check mode against float);
  - a 21-player tournament (3,371 files);
  - training from scratch and resumed, and the 155 unit tests;
  - single-split-probit's first hybrid5 cycle, replayed on hybrid4gfx (12,000 game files equal).
  - Float fallbacks: 2 of 103.8M router decisions; about one depth-2 reply per move.
- **Swapped:** only tournament.binary changed, in single-split (after cycle 111), single-split-probit (after 122) and
  cluster-error (after 39). Every cycle since has finished cleanly.
- **Saving, tournament CPU per game:** single-split 23.5 to 8.8 ms, probit 20.6 to 12.0 ms, cluster-error 18.5 to
  9.1 ms. That is about 167 CPU-s less per cycle across the three.
- **Pinned benchmark, ratio new/old:**
  - depth 2: LR-router trees 0.33, NB 0.27, TAN 0.38, probit leaves 0.44-0.57, DAG 0.15;
  - depth 1: 0.43-0.92, since the leaf value is still float per move;
  - flat: 1.00.
- **Not done:** mixtures and FM still play the old float code. Faster training is done: see the next item.
- P2's spec (5-dag-home.json) now names chessai-hybrid5 as its tournament binary.

**Faster training: deployed** (2026-11-10 05:20, under the same blessing).
- **Binary:** bin/chessai-hybrid6 (md5 0bd96e76...), a copy of hybrid5 with a faster training path; play is
  hybrid5's. Write-up: models/from-scratch/fastbin/hybrid6/summary.txt.
- **Profile first:** in TAN-router trees, pair counting and prediction while training were 60-76% of a step; in LR
  lines, the LR gradient step, prediction and game parsing.
- **What changed:** float operations stay in the same order.
  - TAN pair counts go into an integer batch, with zero cells derived.
  - TAN training predictions are reused for classes whose counts haven't changed.
  - LR nodes step from the prediction they just made.
  - There is an AVX2 LR step and a fast parser for plain lines.
- **Identity:** every check was byte-identical. Models, states, stdout and stderr were compared on 20 captured live
  cycles:
  - every line of the three experiments, chained over 4-8 cycles, resumed and from scratch (280 steps);
  - forced splits and merges on all 24 tree lines;
  - the 155 unit tests and 2 arenas.
- **Swapped:** only train_binary changed, on every line, in single-split (after cycle 502), single-split-probit
  (after 511) and cluster-error (after 174). The tournament binary is unchanged. selfplay-depth was not touched. Every
  cycle since has finished cleanly.
- **Saving, training CPU per cycle:**
  - single-split: 13.3 to 8.0 s (0.60);
  - probit: 9.6 to 6.1 s (0.63);
  - cluster-error: 70.0 to 42.5 s (0.61).
  - Together that is about 36 CPU-s less per cycle. Cycles finish 11-19% sooner.
- **Gains least:** NB routers and probit leaves (0.8-0.94), whose training code is untouched.

**Faster tournament reports: deployed in cluster-error** (2026-11-10 09:16, under the same blessing).
- **Binary:** bin/chessai-hybrid7 (md5 9c7c4c2b...), a copy of hybrid6 with a faster report step ("tables in N s");
  play and training are hybrid6's. Write-up: models/from-scratch/fastbin/hybrid7/summary.txt.
- **Why:** hybrid5 read and rewrote all of tour/elo_history.csv every cycle. It also built a dense head-to-head
  table over all players, retired ones included. Both grow with the square of the cycles:
  - cluster-error at cycle 324: 2.3 GB peak; ~21 GB projected by cycle 1000 on this 30 GB machine;
  - single-split at cycle 858: 5.7 GB.
- **What changed:**
  - The history is appended to when the file is as the last run left it; otherwise the old code runs.
  - The full fit runs beside the prefix fits.
  - Head-to-head counts are kept only for pairs that met.
  - games.csv is read faster.
  - The retired-name check uses a set.
  - Every number is computed by the same operations in the same order.
- **Identity:** 16 suite cases against hybrid5 were byte-identical (every tour file, every saved game, stdout):
  - real snapshots of single-split and cluster-error: report step, a cycle, and a chained cycle;
  - the slow-path cases (changed key, csv newer than its key), and a fresh league;
  - three finished experiments;
  - 4 own-calibration arenas;
  - 157 unit tests.
- **Swapped:** only cluster-error's tournament.binary, after cycle 341 (it also plays the own-calibration arenas).
  The train binaries are unchanged. single-split and single-split-probit stay on hybrid5; they end ~10:30-10:40.
- **Saving:**
  - Report step on the same inputs: cluster-error 12-17 s to 7-9.5 s and 2.3 GB to 0.44 GB; single-split 18.5 to
    7.4 s and 5.7 GB to 0.4 GB.
  - Live: "tables in" went from 6-8 s to 2-3 s from cycle 342. Tournament wall went from 39.0 to 36.0 s a cycle, and
    CPU outside the games from 14.7 to 12.1 s (cycles 308-317 against 342-347).
  - Projected over cluster-error's remaining cycles: about 2-3 h of wall time (an extrapolation), and peak memory
    stays near 1 GB.
- **Not touched, and now bigger:** leagues.py reads all of games.csv and game_cpu.csv several times a cycle in
  Python: done_round_robins twice, game_rows, clean_partial. On cluster-error's files at cycle 324 (nice 19)
  done_round_robins took 4.9 s and game_rows 9.6 s, and the cost grows with the games. An exact fix would read only
  the tail, since rows are in tournament order. It needs an engine change. (Done since without one: see the next
  item.)

**Faster driver scans: deployed in cluster-error** (2026-11-10 10:56, under the same blessing; no engine change).
- **Code:** leagues/leagues.py (not committed; `git checkout leagues/leagues.py` undoes it). It is git HEAD's plus a
  scan-cache section and five call sites. Write-up: models/from-scratch/fastbin/drivercache/summary.txt.
- **Why:** each cycle the driver read all of games.csv and game_cpu.csv five times in Python:
  done_round_robins twice, clean_partial, game_rows, and contrast_ses' pair counts. On cluster-error at cycle 370
  that was ~12.8 CPU-s a cycle on the critical path, growing with the games.
- **What changed:**
  - <league>/scan-cache/ keeps a summary of each file's complete lines, computed with the original code's own
    expressions: rounds, round offsets, pair counts in first-seen order, whether every line parses.
  - It is trusted only when the file is unchanged (device, inode, size, mtime, ctime) or the SHA-256 of the cached
    prefix matches. Then only the new lines are read, or only the cycle's rounds.
  - Anything the summary can't prove the same falls back to the original code: a "\r", odd lines, rounds out of
    order, a partial last line for clean_partial, a damaged or stale cache.
- **Identity:** 0 failures across 1,178 old-vs-new checks on snapshots of cluster-error and single-split and on 4
  finished experiments. The checks include faults: partial lines, a rewritten byte, damaged or foreign caches, a
  "\r", a blank line, rounds out of order. The driver's selftest replay (run straight and interrupted 18 times) is
  identical to the original's (502 items).
- **Swapped:** leagues.py was replaced atomically during cluster-error cycle 416; cycle 417 is the first on the new
  code. single-split (10:51) and single-split-probit (10:56) finished cycle 1000 on the old code. Cycles 417-423
  finished cleanly.
- **Saving:**
  - The driver process's own CPU went from 8.8 s (cycle 416, load ~3) to 2.4-2.6 s a cycle. At this morning's
    load (~10) it was 12-16 s.
  - Pre-tournament wall went from 9.0 to 5.2 s, and the driver part of the tournament phase from 5.1 to 1.8 s
    (cycle 416 against cycles 418-422).
  - Projected over cycles 424-1000: about 1.7-2.4 h of wall time at today's load, or ~3.3 h if the machine is
    loaded again.
- **Not done:** contrast_ses' elimination (~1 s a cycle, growing with the players) and the records CSV rewrites
  (~0.4 s a cycle).

## 5. Proposals, ranked

All local, LR leaves or experts, 4 lines, 300 cycles x 5 rounds unless noted.

### P1. moe-gates: does a mixture beat flat LR on equal data, and what does a hard gate cost in play? (Q4, Q6)

Spec drafted: leagues/specs/batch3/1-moe-gates.json.

| Line | What it is | Compared with |
|---|---|---|
| F-L | flat LR | - |
| M-L4-L | 4 LR experts, LR gate, soft mixture | F-L: mixture vs flat |
| M-L4-L-hard | the same model, playing with the gate's top expert only | M-L4-L: the cost of hard routing |
| M-L4-N | 4 LR experts, NB gate (the strongest compound signal) | M-L4-L: NB gate vs LR gate |

- **The soft/hard pair is the cleanest test we've had.** --moe-hard changes only prediction, and the two lines share
  data and seed, so their weights are byte-identical every cycle. Any gap between them is the price, or the gain, of
  hard routing in play: the seam question for every tree and DAG.
  - If hard costs a lot, every hard-routed design pays that tax, and boundaries need fixing first (section 6.3).
  - If it costs nothing, the trees' problem is elsewhere: routing quality or data per leaf.
- Checks at cycles 100, 200 and 300: gate shares, how hard the gates are, expert distinctness, and who-won
  asymmetry: each expert's share of white-won against black-won positions. Responsibility targets are posteriors,
  and in 11-05's trees, posterior targets with LR leaves routed by who's winning.
- Cost: MoE players are 6-12x slower than flat LR at ply 2 (costs.csv: moe 0.15 s a game against 0.013-0.025 for
  lr). Estimate 60-90 CPU-s a cycle, so 5-8 CPU-h for 300 cycles.

### P2. dag-home: a DAG whose router splits mean something (Q5)

Built and checked overnight; ready to launch. Spec: leagues/specs/batch3/5-dag-home.json. Binary:
bin/chessai-bddhome2 (training); tournament on bin/chessai-hybrid5. Write-up: models/from-scratch/bddhome/summary.txt.
Design in section 6.2. Before launching, see section 8, item 4: home targets train R' on a counterfactual.

| Line | What it is | Compared with |
|---|---|---|
| F-L | flat LR | - |
| T-L-Ne | NB error tree, no router splits | F-L |
| D-L-Ne | DAG as built: R' trains on which copy routes best | T-L-Ne: router splits |
| D-L-Ne-home | DAG with home targets: R' keeps R's target | D-L-Ne: R''s target only |

- All trees: split limit 5, threshold 1M, --error log, 3 children, no merges. The two D lines differ only in the
  option (and the train binary, which is byte-identical to chessai-hybrid4gfx without it). The spec's tournament
  binary is now chessai-hybrid5. chessai-hybrid4gfx plays home-trained models with identical games, and fork 2's
  identity checks for hybrid5 included two DAG models. T-L-Ne's recipe also carries
  --reset-leaves inf, whose shadows only log; the DAGs refuse that flag.
- **Checks passed:** 154 unit tests, including:
  - routing unchanged at the split;
  - R' matching a router trained on the old children's target, step by step for 3,000 steps.
  Without the option, the new binary trains and plays byte for byte like chessai-hybrid4gfx. Home states resume
  only with the option.
- **Offline mechanism check** (one seed; it shows the mechanism works, not that it's stronger):
  - R' doesn't collapse. The root's shares stayed at the unsplit tree's: 38/48/14 at round 60, against 39/41/19
    before the split. The plain DAG's new root drifted to 42/56/2 by round 8.
  - The copies part because they see different positions. They overruled their home pick on 4% of positions at
    round 4, 22% at round 8 and 35-40% from round 30.
  - The re-routing cost a little early on: R' was 0.0016 worse than its busiest copy alone at round 6 (plain: 0.0007).
    It helped slightly by round 60, by -0.0038 summed over the R' nodes (plain: -0.0028).
  - Held-out loss: level with a tree with the same 9 leaves after round 20, and about 0.001 better than the plain DAG
    at rounds 40-60.
- **Router splits are welcome, the trigger is the open part.** Offline, 6 of the 10 splits were router splits,
  and the root became a chain of 5 routers. The user (2026-11-10): many router splits are probably fine, since
  leaf splits fragment the training data and router splits don't (the leaves stay shared). With home targets a
  router split changes nothing when it happens. What's unresolved is that the trigger fires on a rise that
  happens anyway, so it can't tell when a split helps. At split limit 5, expect the DAG lines to end with fewer
  leaves than T-L-Ne, so T-L-Ne against the DAGs compares equal split budgets, not equal leaves.
- Cost: about 45 CPU-s a cycle, so ~4 CPU-h for 300 cycles. The prototype took ~15 CPU-min.

### P3. moe-experts (after P1, with P1's better gate): F-L, M-L2, M-L4, M-L8

- One factor: the number of experts. Needs hand-added recipe rows for the missing counts (make_recipes.py HAND), so
  the line names say what they are. Cost grows with experts: 8 soft experts cost about twice 4.

### P4. Experiment 2: self-play against local at each leaf's best depth (question C)

- After selfplay-depth (cycle 21 of 100). So far depth 0 and 1 plateau by cycle 3 and depth 2 is still climbing for
  LR and probit; NB is best at depth 0.
- At cycle 67 (section 4): depth 2 is best for LR and probit, by 75-235 over depths 0 and 1. Depth 4 is starved of
  data. NB at depths 1-4 has collapsed to near random. So P4's depths would be 2, 2 and 0. selfplay-depth ends
  ~07:00.

### P5. Self-play exploration policy (deferred since 11-09): jitter10, jitter20, two softmax temperatures

### P6. cluster-vs-merge: what makes the cluster single split win? (one factor at a time)

- Lines: F-L, T-L-Nc, T-L-Nc-merge, T-L-Ne. All local, split limit 1, threshold 1M, as in single-split.
- Comparisons:
  - T-L-Nc against T-L-Nc-merge: the merge test only.
  - T-L-Nc against T-L-Ne: cluster against error routing (neither merges).
  - Each against F-L.
- The win took 40-120 cycles to appear, so it needs ~300 cycles.
- Cost: 4 lines, about 30 s of wall time a cycle on hybrid5 at today's load, so ~2.5 h of wall time for 300 cycles.
- Would add T-L-Nc-gap (chessai-target with gap targets and route-before-train) as a 5th line only if the user
  wants T1 in the same field. Otherwise T1 gets its own experiment.
- Notes from 08:40-09:30 (§3):
  - Every NB/TAN cluster router so far found the same game-phase partition, so P6's will almost surely be that one.
  - With LR leaves, that one partition is worth ~80 in single-split's stream and ~0 in cluster-error's, so P6's
    result will depend on its own stream. Its F-L comparator measures it there.
  - A probit-leaf P6 (F-P, T-P-Nc, T-P-Nc-merge, T-P-Ne) would test the setting where the cluster lead held in both
    experiments (30-65).
  - LR cluster routers churn through split and merge, so a T-L-Lc line is a flat control, not a third router.
  - Log leaf shares every 10 cycles (routing_health.py).
  - Keep each cycle's local games for a while. The pool is deleted at cycle end, so we cannot now replay one
    stream into the other experiment's lines to test the stream explanation.
- Notes from the final check (11:00, §3):
  - Measure with current-against-current direct games (scratchpad xtab/curtab.py), not version windows, which
    count champions.
  - P6's LR lines are 3 of single-split's 7. Its T-L-Nc-merge against F-L therefore also tests whether
    single-split's ~80 was chance.
  - A probit-leaf P6 needs a long run: in single-split-probit the cluster trees crossed flat only around cycle 300
    and were still climbing at 1000. In cluster-error's mixed stream they led within 100 cycles.

### P7. cluster growth: does a second and third cluster split keep paying? (after P6)

- **Launched 2026-11-10 11:39 as split-growth, in the user's 2×2 design** (merge gate × split limit), before P6:
  - Lines: F-L; T-L-Nc-merge at split limits 1 and 4; T-L-Nc (no merge gate) at split limits 1 and 4. NB cluster
    routers, LR leaves, threshold 1M, branch factor 3 (limit 4 allows up to 9 leaves).
  - All local (the experiment's own games only). 9 rounds a cycle, so 2,430 games a cycle, single-split's volume.
    Seed 2, 400 cycles. Train binary hybrid6, tournament binary hybrid7. Spec: leagues/specs/batch3/7-split-growth.json.
  - Each tree differs from T-L-Nc-merge at limit 1 (single-split's winner) in one factor. The 2×2 also shows whether
    growth needs the gate. The gate undid both winning trees' first two splits in single-split (cycles 18 and 25).
  - Reading the outcomes:
    - Limit 4 above limit 1: growth pays even with leaf splits.
    - About equal: the merge log tells whether the gate rejected the extra splits or they stayed and added nothing.
    - Below: splitting the data costs strength, which argues for growing by router splits (DAG, P2).
  - Measure with current-against-current games. Leaf shares can be computed afterwards from the saved versions.
- **P7b, launched 2026-11-10 12:31 as split-growth-probit (the user approved):** probit leaves, where error routing
  has worked (T-P-Ne 44-47 above flat) and trees were still climbing at cycle 1000.
  - Lines: F-P; T-P-Nc-merge at split limits 1 and 4; T-P-Ne (error routing, never merges) at split limits 1 and 4.
  - The second pair is error trees, not no-gate trees: the LR experiment already answers the gate question, and
    only probit leaves can ask whether growth helps error routing.
  - Same settings as split-growth (local, 9 rounds a cycle, 2,430 games), except 1000 cycles and seed 3. Spec:
    leagues/specs/batch3/8-split-growth-probit.json.
  - Flat probit's memory length is probably not the cause of the probit trees' slow climb. In probit-settings,
    flat probit with tau 1e-6 or tau 0 did no better than the default tau 1e-5 (current against current, cycles
    101-500: 1e-5 4 to 21 above tau 0, and 1e-6 2 to 18 above).

- Lines: F-L and the winner of P6 at split limits 1 and 4. One factor: the split budget.
- This tests the user's growth goal directly. Theory 3.5 expects each further split to need more data before it
  pays.

## 6. Towards compound models with routing: a path

1. **Measure the seam tax first (P1).** Of the explanations for "routing doesn't beat flat in play", seams have the
   strongest support from play (section 3: trees lose far more at ply 2 than at ply 1, and M-L4-N-c's edge over its
   best expert shrinks from 73 to 46). That evidence is indirect, and P1 measures the tax directly.
2. **Make router splits mean something (P2).** The degeneracy is mechanical: after R splits into R' over copies
   R1-R3, all copies pick the same leaf, so every R' target is a tie, ties go to the first child, and R' becomes a
   pass-through. Fix, "home targets": R' keeps training on R's target (which shared child predicts best), and its
   pick i sends the position to copy i, whose "home" is child i.
   - At the split nothing changes: R' routes as R did and each copy routes as R did.
   - Each copy then learns, within its own region, where R's first guess is wrong. The copies part because their
     data differ, not because of noise or forced forgetting (bddfix's 16x forgetting damaged routing).
   - R' never sees ties, because its targets don't depend on the copies.
   - Risk: little routing headroom on these leaves (the ridge check, weak evidence). Per-region routers can carve
     regions a single router can't, so they may find some, or none.
   - Built overnight as --subrouter-target home (P2), and offline it behaves as designed.
   - The user (2026-11-10 ~00:00) confirmed the design matches theirs (a central router over sub-routers that
     route to the old router's children), and that tying the number of sub-routers to the leaf split count
     (the branch factor) is good.
   - Next, if P2 shows the copies' re-routing pays in play: a trigger that splits a router only when its copies
     would beat it. bddfix found the current trigger fires on a rise that happens anyway, which makes the root
     chains.
3. **If hard routing costs play: blend only at the boundaries.** Predict with one leaf or expert when the router is
   confident (top probability above a cutoff c), and blend the top two by their probabilities otherwise. Most
   positions still use one leaf (the sparsity the user wants), and the jumps a search exploits shrink.
   - Testable like P1: a play-time flag, so lines with identical weights differ only in c.
   - Needs the user's view first. On 11-05 they said blending leaves "defeats the purpose of the delegation". This
     blends only near boundaries.
4. **Path context** (the user's design, 2026-11-10 ~00:30; fork 3 is building it as --path-context, off by default):
   - Every node gets one categorical cell per upstream sibling set (the copies made by one router split), in
     registry order.
   - A cell is 0 if the path skipped the set, or 1..B for the member it went through. The central router made by a
     split keeps the split router's place in its set.
   - Theory (section 8):
     - For LR nodes this is a per-route intercept, additive over sets, and a nested convex class, so it can't hurt in
       the population.
     - For NB nodes (leaves or routers) it double-counts whatever the routers route by. Counterexample: 0.500 to
       0.612 nats. It also doubles the class prior for a while after each new cell.
     - So test it on LR leaves first.
   - Lines with the option play on the float path until it is merged into hybrid5.
   - **Built (fork 3, 2026-11-10 ~02:00):** bin/chessai-bddctx (md5 0918dd1a...), --path-context, BDD only.
     Write-up: models/from-scratch/bddctx/summary.txt.
     - **Identity:**
       - 164 tests pass. They cover: no split changes a prediction bit; cells equal a fresh recomputation; every
         path decodes; the user's nested P1/P2/P3 case.
       - Without the flag it trains byte for byte like bddhome2 (8 rounds through a router split) and plays
         hybrid4gfx's 72 games.
       - hybrid4gfx and hybrid5 refuse its model files, so the tournament must run on chessai-bddctx.
     - **Departures from the spec, all forced:**
       - Copies of a router that is itself in a set get a neutral cell for that set.
       - With routing targets the central router's table starts blank.
       - Cluster routing, route switching and TAN routers are refused.
       - CNBP cell factors are rescaled so a new cell multiplies by exactly 1.0.
     - **Mechanism check** (bddstudy setup, seed 1):
       - Both arms made 7 router splits and 3 leaf splits.
       - Cells shift LR leaves' log-odds by 0.42 on average, but the part that differs by path is 0.13.
       - CNBP router cells flip 7% of router decisions (double counting, theory 2.6).
       - 58% of positions reach their leaf through a non-home route by round 60. Home targets with cells score
         those paths counterfactually (theory 2.5 b).
       - Held-out loss is level with the control from round 40 (worse at round 20). The structures differ, so this
         doesn't isolate the cells.
     - **Growth is not linear.** In a 200-split growth test nodes went 76 to 601 and table numbers 6.3k to 259k. A
       router split high up adds a cell to every node below, and chains of router splits make that quadratic.
       Linear alternatives, not built: cells for the parents' sets only, or for the K nearest sets.
     - **Side fix:** bddhome2's home-target code is exponential in the depth of sub-router chains. bddctx computes
       each node once and trains identically without the flag.
5. **Cluster on top, error below** (11-08 design talk): a cluster router at the root hands each error sub-router a
   coherent region. Needs a per-depth routing option. After single-split and cluster-error show how the two
   approaches compare on equal data.
6. **A growing mixture, if P1 favours M-L4-N** (a hierarchical mixture of experts). M-L4-N differs from our trees in
   four ways:
   - its experts train side by side from the start, after a random warm-up, instead of being copied at a split;
   - its gate learns posteriors (responsibilities), not best-child targets;
   - each expert's step is weighted by its responsibility;
   - its prediction blends experts on the 32% of positions where the gate's top weight is below 0.99.
   A tree that grows the M-L4-N way would turn a stalled expert into a small NB-gated mixture of its own copies.
   It would keep the user's growing structure and take the parts that work. Design only after P1.

## 7. Decisions for the user

1. Launch P1 (moe-gates)? Its spec is ready.
2. Launch P2 (dag-home)? The prototype is built and checked, and its spec is ready.
3. Make room: drop the cluster lines from single-split and single-split-probit (cluster-error covers them), or shorten
   cluster-error to 500 cycles?
4. Boundary blending (6.3): acceptable in principle, if P1 shows a large seam tax?
5. ~~Build fixed point for trees and mixtures (section 4)?~~ Done for trees and DAGs and deployed 00:53-00:58, every
   check byte-identical (section 4). Mixtures not yet.
6. Consistent router targets (section 8, T1): built and checked offline (bin/chessai-target). A league test, and
   which lines?
7. P2: launch as is, or add the home-target refinement (section 8, item 4) first?
8. Residual (additive) leaves: a clean rerun (T3)?
9. cluster-error's abs/log pairs are identical models so far (section 4). Does that settle decision 3?
10. Path context (section 6.4) before any league test:
    - CNBP cells: LR nodes only, LR routers, or accept the double counting?
    - Full path (quadratic in chains of router splits), or cells for the nearest K sets only?
    - The test design: which lines?
11. P2's D lines: train on chessai-bddctx without the flag, which is identical so far and avoids bddhome2's
    exponential home-target recursion? At split limit 5 the chains are short (the offline prototype ran fine), so
    this is a safeguard, not a blocker.
12. P6 (cluster vs merge, one factor each) and then P7 (cluster growth): launch?

## 8. Theory and the backprop lens (2026-11-10 ~01:10)

The user (~00:45): the tree/DAG with error routers and error-based training was meant as an answer to
backpropagation. A math agent (Fable) analysed the construction. Write-up: models/from-scratch/theory/summary.txt.
Results there are labelled THEOREM, COUNTEREXAMPLE, CONJECTURE or SIMULATION, and there are three small pure-Python
simulations, about 3 CPU-min. In order of importance:

1. **The router targets optimise the wrong thing.**
   - The routed loss is a cost-sensitive classification problem. For B=2 it is TAO's node problem: label the better
     child, weight the example by the loss gap, drop ties.
   - **Best is inconsistent.** With two outcomes it picks the child leaning to the commoner outcome whenever
     p > 1/2, whatever the magnitudes. Example: p(W)=0.6, children 0.95 and 0.6. Best picks the first, with expected
     loss 1.229 against 0.673.
   - **Soft** is right for the Brier score in the binary case and wrong for log-loss.
   - **Balanced** routes by lift, so it is wrong even where Best is right.
   - **Consistent targets:**
     - the pairwise gap (B=2);
     - fractional labels w_k = M - l_k for any B (M can be the example's largest loss, so ties give zero weight);
     - weighted all-pairs.
   - Cost: one weighted step. For an LR router it must be one step on w/sum(w) with weight sum(w); B separate steps
     freeze an AdaGrad router (shown in simulation).
2. **Every error router is an evaluator in disguise.**
   - For two children that predict differently, the Best label depends on the outcome. So the router's target is
     an outcome probability: it learns who is winning, and it is a weaker evaluator than the leaves it routes to.
   - **Checked on our trees tonight.** Held-out t01 positions, leaves from chessai-predict (matches `routing`
     exactly). U is the share of a tree's routing entropy explained by a flat LR's "who's winning" decile:

     | Router | Lines | U |
     |---|---|---|
     | LR, error | single-split T-L-Le; Le-fresh-c; Le-share3-c; LLs-c | 0.31; 0.25; 0.14; 0.53 |
     | LR, Balanced | Le-bal-c | 0.05 |
     | NB, error | single-split T-L-Ne; Ne-c, -pre, -share3, -bal, -fresh | 0.02; 0.04-0.12 |
     | NB, cluster (label-free) | single-split Nc-merge; Nc-c, -share, -fresh | 0.03; 0.10-0.11 |

   - So the mechanism shows in LR routers and not in NB routers, which route like label-free clustering. That fits
     T-L-Le being the worst line and T-L-Ne the best (section 3), and the LR lock-in found on 11-08.
   - Outcome per leaf on its own is no test. Label-free trees show it too (T-L-Nc-c's leaves range from 1% to 99%
     white wins among decisive positions), because positions carry the outcome.
3. **After a split, every consistent target is silent.** Identical copies give zero gaps, so symmetry has to be
   broken explicitly and without labels:
   - hash routing for a window, as cluster routers do;
   - disjoint halves of the first window;
   - small noise, as in Net2Net.

   Best breaks symmetry only through outcome noise plus ties going to the first child. That is the
   routing-by-outcome mechanism.
4. **Home targets train R' on a counterfactual.**
   - R' learns which base child is best. But its pick i hands the position to copy i, which overrules the home child
     on 35-40% of positions.
   - Fix: target the copy whose subtree, as actually routed, does best, and break ties towards the home pick. This
     keeps the tie-freedom that motivated home targets.
   - With path cells the mismatch is sharper.
5. **What a home DAG computes:** a tree over the same leaves with a deeper gate (a tree of NB classifiers).
   - Router splits grow the gate; leaf splits grow the experts.
   - The ceiling over the tree is the routing headroom we measured: 0.001-0.003 nats.
6. **Growth that preserves every prediction:**
   - Holds for leaf splits with F=1, LR under any F, home router splits with F=1, and neutral path cells (exactly
     1.0 in IEEE for CNBP).
   - Fails for CNBP with F>1, which acts like extra smoothing (up to ~1 nat on a rare square).
7. **Descent:**
   - Exact alternating steps with a consistent router problem never raise the routed loss (TAO, classification EM).
   - Leaf steps are unbiased SGD on the global routed loss. Routers can't do SGD (hard decisions), so they need a
     surrogate, and Best, Soft and Balanced are the wrong surrogates.
   - Online and in self-play there is no fixed objective, so nothing guarantees convergence.
8. **Additive (residual) nodes** are the only growth operation here with a monotone guarantee, and the closest match
   to the user's residual-layer picture. Each node gets a zero-initialised delta, and the prediction is the softmax
   of the sum along the path. Path cells are the intercept-only case.
   - With routing fixed the loss is convex, and growth preserves predictions without raising the optimum.
   - The shared part tracks drift common to all regions B times faster than copies do.
   - **Caveat from our record:** residual leaves (shadow + delta, resid2/resid4) were tested on 11-07 and lost in
     play: 171-266 Elo below at v60, and overconfident.
     - Those runs also had merge tests on, which the residual leaves fooled, so no clean one-factor run was made.
     - Theory 7.7 fits the overconfidence: fresh deltas take full-size AdaGrad steps.
9. **Split trigger:**
   - Splitting a leaf by a partition the router can learn gains at first order: 2|d||G_A|, with G_A the leaf's
     gradient summed over region A. So the right trigger is the size of the leaf's mean gradient over candidate
     regions.
   - Today's trigger measures stalling, which every leaf does as its learning curve flattens. That is why it fires
     everywhere eventually.
10. **Train-then-route leak:** a training example goes to the router's pick after the router has learned from that
    example's label.
    - Negligible for a mature router (0.1-0.2% of picks); large for a fresh LR router.
    - Free fixes: route by the pick before the update, and break ties at random or by hash instead of always to the
      first child.
11. **Expectations (conjecture):** a leaf split breaks even at about a million positions per leaf (~2,700 parameters,
    gain ~0.003 nats). That is the order of the ~80-cycle recovery we see.

Proposals from it (one factor each, options off by default; the user designs any league test):
- **T1 consistent router targets:** --error-target gap. The free fixes are separate flags:
  - --route-before-train;
  - hash tie-breaks;
  - a hash bootstrap after splits.

  - **Built 2026-11-10 ~02:05:** bin/chessai-target (md5 ff27f7d3...; source target-src, a copy of hybrid5-src).
    Write-up: models/from-scratch/target/summary.txt; tables in target/analysis.txt.
    - Flags: --error-target gap (with --target-gap-cap 4), --route-before-train, --split-bootstrap N. All off by
      default and not saved in model files, so hybrid5 plays the models; resume needs the same options.
    - With the flags off it is byte-identical to hybrid5: 18 training runs from scratch and resumed, self-play, a
      72-game tournament, and bddstudy's bdt-s1 reproduced exactly. 173 tests pass.
  - **Offline mechanism check** (bddstudy setup, seed 1, threshold 100k, limit 10, 21 leaves by round 40):
    - **Gap stops LR routers routing by who's winning.** U fell from 0.14-0.16 under Best to 0.07-0.11 in every
      gap arm, inside the NB and cluster range.
    - **Less lock-in:**
      - Mean largest share at a delegator, rounds 40/60: Best 74-75%, gap 53-58%, gap with bootstrap 44-49%.
      - Delegators starving a child: Best 2-4, gap 2-3, gap with bootstrap 0.
      - Effective leaves out of 21: Best 8.5-9.9, gap 12.4-12.6, gap with bootstrap 13.9-17.5.
    - **Not seen: children used one at a time under gap.** That happened under Best: 7 of 10 LR-router splits
      started with a child at 0-1%, and one stayed at 1% for 53 rounds. The likely cause is the train-then-route
      leak breaking ties, which wasn't tested separately.
    - **Held-out loss, a sanity check only: nothing beats Best.**
      - Gap costs 0.002-0.015 nats, and the bootstrap costs more.
      - --route-before-train wins back 0.005-0.015.
      - All three together are level with Best at round 40 (0.0002 [-0.0057, 0.0061]) and slightly behind at 60
        (0.0049 [-0.0001, 0.0100]).
      - NB routers with gap and bootstrap: worse at rounds 20 and 40, level at 60.
    - **Reading:** the targets now do what the theory says, routing by expected loss and spreading the data.
      - Spreading the data is the cost at this scale: about 2.4M positions over 21 leaves, far below the ~1M per leaf
        that theory 3.5 puts at break-even.
      - Best's concentration acts like fewer leaves, so less fragmentation.
      - The league is the judge. The user designs any test.
      - Cheapest clean test: route-before-train alone against Best (a free fix, one factor). Then gap with
        route-before-train against Best.
- **T2 home-target refinement** (item 4): an option on bddhome2, as a fifth P2 line or instead of D-L-Ne-home's
  target.
- **T3 additive deltas:** a clean one-factor rerun against copy-on-split, with no merges and a smaller delta step.
- **T4 a gradient-disagreement split trigger** (item 9).
