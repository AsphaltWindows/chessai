# Ideas to explore

Written overnight 2026-11-06 at the user's request, after the from-scratch standard (no search in training) was
set. Ordered by how much I expect each to tell us about a stronger evaluator, with the evidence behind it
and a first test. Everything stays within the ground rules: self-play only, zero knowledge, hard routing.
`LEDGER.md` has the options already tested.

## Where we stand

- LR's jump over naive Bayes (+370) came from dropping the independence assumption. Since then, every
  architecture gain has been small: the best trees beat flat LR by +23 to +30.
- Trees predict held-out games better, but play worse unless their leaves agree where games cross from one
  leaf to another (seams): search finds and exploits the disagreements.
- From scratch, naive Bayes is hurt by every form of forgetting tried (batch 28). The one-time ÷16 gain came
  only after 2000 rounds of accumulation.

## Architecture: more capacity without seams

1. **Pairwise interactions for LR (factorization machine).** The model is LR plus a learned low-rank vector
   per (feature, value). Every pair of features interacts through the dot product of their vectors, so
   nothing is chosen by hand. This captures what trees approximate by partitioning, but with no routing,
   so there are no seams. Cost: about 8 times an LR evaluation with 8 factors (the usual O(features × k)
   trick), plus a new model kind. Test: `F-FM` against `F-L` from scratch, at v300 and v1000.
   *Prototype built overnight: see the end of this file.*
2. **Hierarchical (shrinkage) leaves.** A child is its parent plus a correction that a prior pulls towards
   zero. For CNBP, the parent's counts act as a Dirichlet prior of fixed strength. For LR, there's an L2
   penalty towards the parent's weights; under revision, the parent is the shadow, which already trains on
   every example. Children then only disagree where their data says so, which attacks both seams and
   data-starved leaves. Each position is still handled by one leaf, so I don't think this is blending,
   but it's your call. Test: `T-L-Nc` with and without.
3. **A small neural network** (one hidden layer) on the same encoding. This is the standard next step in
   capacity. It's further from the project's Bayesian roots, so only if you want it.

## Trees and routing

4. **Router regret (the signal for DAGs you asked to keep).** For each router, it's how much worse its
   pick does than the best of its children on the same positions. Alongside it, near-duplicate leaves:
   leaves in different branches that predict alike. High regret, or duplicates across branches, is when
   merging paths (DAGs) should start paying. *Built overnight: see the end of this file.*
5. **Seam-targeted sharing.** Where consecutive positions of a game cross from one leaf to another, each
   position also trains the neighbouring leaf with a small weight. Uniform leaf sharing gave +12 for CNBP
   trees; aiming it at the boundaries search exploits should do more for less. The crossings are already
   measured (`measure_seams`).
6. **Revision follow-ups.** If branch merges or leaf resets beat plain `T-L-Ne`, try them on the other
   tree archetypes. Choose the seam weight from the from-scratch `T-L-Ne` windows and test it.
   *Done. Merges help every tree archetype but `T-N-Ne` (+41 to +190 vs the twin), and resets add nothing on top
   of them (−9). With merges the LR trees are level with F-L in the tournament, and `T-L-Ne` leads (+26 at d2,
   +46 at d3). The seam weight is still untested.*
7. **Split timing from scratch.** Batches 29 and 30 log when each tree splits. If trees split while the
   flat model is still improving, a split threshold tied to the leaf's own learning curve may beat the fixed
   5M examples.
   *From scratch every tree loses to its flat model (LR trees −69..−144, naive-Bayes trees −129..−476), and the
   merging `T-L-Ne` undid its root split 7 times in rounds 132–559: the unsplit leaf, trained on 3 times the
   data, kept predicting winners better than the split. Merges are one automatic answer; a direct one is to
   split only when the leaf's own learning has flattened.*

## Training dynamics

8. **Why forgetting from scratch hurts.** The γ=3 age-scaled run swung between white-won and black-won
   self-play. It chases its own last games, the classic self-play instability when recent games weigh too
   much. If forgetting comes back (you wanted a model that trains indefinitely), it may need a damping term
   rather than a smaller factor.
9. **Is v1000 enough for every archetype?** A depth-2 arena of v750 against v1000 per archetype is cheap.
   If the trees are still climbing at v1000 while flat models are not, the 1000-round budget favours fast
   learners.
   *Partly answered. v1000 vs v750: F-L +69, `T-L-Nc` +53, merging `T-L-Ne` +30, F-N +10 (−1..+20). F-L continued
   to v2000 gains nothing after v1000 (steps of 250: +4, −8, −6, −3; v2000 vs v1000 directly −25 (−35..−14)), so
   flat LR peaks right at v1000. Whether
   the trees keep climbing needs longer tree runs: trees can't `--resume`, so that means 2000-round runs from
   scratch, or saving the trees' full state.*

## Measurement

10. **Equal time instead of equal depth.** Trees and mixtures evaluate more slowly. At equal thinking
    time, flat models would search deeper. The tournament compares at fixed depth, which favours the slow
    archetypes.
11. **An external anchor** (an engine at its weakest settings), to put "random = 0" on a human scale
    (your 800-player question). For evaluation only.

## Done overnight (uncommitted code)

- **`chessai routing bdt:PATH GAMES --use-probs`** (idea 4). It refits one scale and offset for the tree's
  log-odds. Then, for each delegator, it shows a table: one row per child that positions are routed to, with
  the winner log-loss of every child's subtree on those positions, the routed child's loss in brackets.
  Regret is what sending each row to its best child would save. Last come the leaf pairs whose log-odds
  differ least (near-duplicates). It takes about 3 s per tree.
  - On the old `T-L-Nc` (`lrtree-c3-nbr` v300), the root routes perfectly (regret 0). Its two lower
    delegators lose 0.0008 and 0.0015 nats per position.
  - Sibling leaves 7, 8 and 9 are near-duplicates: their log-odds differ by 0.08–0.20, against 0.41 for the
    average pair. So the waste is splits that didn't specialise, not routing mistakes; nothing points to
    DAGs yet.
  - From-scratch trees at v1000, on the family's held-out games: the routers are not the problem (root regret
    0.0000–0.0001; lower delegators up to 0.007, except soft-target naive-Bayes routers at 0.15–0.29). What tracks
    play is how much the leaves disagree, the mean |log-odds difference| of a leaf pair:
    - LR trees: `T-L-Nc` 0.74, `T-L-Ne` 0.80, the merging `T-L-Ne` 0.47 (5 leaves); the old plateau-start `T-L-Nc`
      0.41.
    - Naive-Bayes trees: `T-N-Nc` 1.91, with sharing 0.75 (+158), `T-N-Ne` 0.64, `T-N-Ne` soft 3.98 (−302).
- **Factorization machine (`--model fm`, `fm:PATH`; `--fm-factors 8 --fm-scale 0.01 --fm-seed 1`)** (idea 1).
  - Tests: it learns XOR, which LR cannot; text round trip and forgetting keep predictions.
  - Speed: play is about 3–4 times LR, training about 15–20 times (single-threaded AdaGrad on 1,700
    weights per example).
  - From scratch (`from-scratch/F-FM-s21`, seed 21, 1000 rounds, 8 factors): −259 (−273..−246) vs F-L at d2,
    still climbing (v1000 vs v750 +27). It also predicts worse: calibrated winner log-loss on the LR held-out
    games 0.531 against F-L's 0.497. So the interactions as trained add noise. Untuned so far: the vectors share
    LR's learning rate and have no penalty. A smaller rate or an L2 penalty on the vectors, or fewer factors, are
    the first things to try.
- **Forgetting in trees** (`Graph::forget`): `--round-forget` and `--age-forget` now work for trees too.


## Self-regulating trees and DAGs (2026-11-07, user: "more work and ideas on self-regulating trees or even dags")

Evidence this builds on: with shadow merging (`--revise 0`) every cluster-routed LR tree kept merging its root back
into flat LR (gain vs shadow negative at every check), so the tree's regulator works; it just never finds a split
worth keeping. MoE shows the opposite failure: its gate collapsed onto one or two experts and flips the whole model
between an expert trained on tournament games and one trained on its own depth-0 games (v300 → v336: 93% of
positions on expert A → 99% on expert D, 1744 → 1368). Experiments go into the league as lineages (user, 2026-11-07).

10. **Residual leaves (idea 2, now being built: `--leaf-residual`, binary chessai-resid).** A child's LR is its
    parent's live shadow plus a correction that starts at zero; the parent part keeps learning from all the parent's
    data, so splitting no longer starves the children. Still one LR per position (weights add). Planned lineages:
    T-L-Ne-merge-res-c and T-L-Nc-merge-res-c. If splits now hold under `--revise 0`, the regulator has something to
    keep; if they still merge back, fragmentation was not the reason.
11. **One currency for every structural move: recent prequential log-loss.** Predict-then-train loss with forgetting
    is online MDL. Use it for splits as well as merges: candidate children run as shadows (residual deltas trained
    on the side, not used in play) and are promoted only when their gain over the leaf is significant, the same test
    that already demotes branches. Then the tree grows, holds and shrinks by one rule, and early splits that a
    stronger model no longer needs dissolve on their own.
12. **Search-aware validation.** Seams cost games because 2-ply search seeks out over-rated positions. Judge
    structural moves on the positions the model's own search picks (the principal-variation leaves), not on all
    positions, and track each leaf's exploitation gap: predicted value of its search-picked positions minus the
    results. A leaf that is systematically optimistic there is shrunk toward its parent or merged. Zero knowledge;
    search only chooses which positions validate, it makes no labels.
13. **Split–merge DAG.** Leaves in different branches whose models have converged (near-duplicate predictions on
    their pooled data: the duplicate-leaf signal from idea 4) merge into one shared node; both routers keep their
    paths but point to it, and its data re-pools. With splits from idea 11, the number of distinct leaves tracks what
    the data supports instead of only growing. Router regret (idea 4) decides when a router itself should be
    revised rather than its leaves.
14. **Live parent smoothing for naive-Bayes leaves** (the CNBP version of idea 10): child probabilities =
    (child counts + m · parent-shadow probabilities) / (child total + m), with the shadow learning from all data and
    m chosen per node by prequential loss among a few values (empirical Bayes). Replaces the static parent-copy
    prior that forgetting factor 1 gives today.
15. **Collapse and source detection.** MoE's gate learned to tell data sources apart (own depth-0 games vs
    tournament games) rather than chess situations. Trees can do the same once lineages train on both. Cheap
    regulation signals: each leaf's share of self-play vs tournament positions, and the gate/routing entropy per
    version. A leaf fed by one source is a source detector, not a specialist; for MoE, a load-balancing penalty on
    the gate is the standard fix.
16. **Additive multi-router DAG (parallel splits, one LR per position).** Generalise residual leaves (idea 10) from one
    path to several: R routers each send a position to one of their children, and the position's LR is the base plus
    the deltas of the R children it lands in (weights add, so still one LR per position and hard routing, no blending).
    The effective cells multiply (2^R for binary routers) while every delta trains on everything its router sends it,
    so cells never starve. A router that stops paying (its deltas' prequential gain, idea 11) is dropped; a new one is
    grown where the residual errors cluster. This is the DAG form of a tree in which branches share structure: an
    oblivious tree (the same split on every branch of a level), learned online. League evidence 2026-11-07: merging LR
    trees collapse back to 1-3 leaves that barely disagree (mean leaf-pair |log-odds difference| 0-0.15) and rate level
    with or above flat LR (T-L-Ne-merge-c v794 1718, T-L-NNs-merge-c 1714, F-L-c 1676 at d2); non-merging LR trees keep
    11 leaves that disagree a lot (0.52-0.78) and rate 43-115 below flat LR. So structure that each leaf must learn on
    its own share of the data has not paid; structure whose parts learn from overlapping data might.

**Measured 2026-11-07 on cycle 15's league games (34,030 games, no games played; `models/from-scratch/optimism/`:
`chessai-predict` + seams.py + selection.py).** These results move ideas 5 and 12 down the list and leave 10, 11 and 16:
- *Seams are not exploited.* A position counts as a crossing when the tree routes it to another leaf than the position
  two plies earlier (5-21% of positions). Compared with flat LR on the same positions, LR trees overrate crossings
  *less* than other positions: +0.5 to +2.9 points of win probability at depth 1 and 2, mostly significant. Only
  T-N-NNs-merge-c (near-duplicate NB leaves) shows the predicted overrating (−3.6 ± 0.5 points at d1, −3.6 at d2).
- *Selection optimism (the search's winner's curse)*, measured as how much more a model overrates the positions it
  moved into than those its opponent moved into, net of a side-to-move baseline from games it didn't play:
  - At depth 1, every model shows it. Flat LR runs 0.3-1.0 points (mean 0.64), and merged single-leaf trees match
    (0.67, 0.82). LR trees with real structure are about twice as high, 0.9-1.7 (mean 1.2), and NB models higher
    still, 1-3 points.
  - At depth 2, the opponent's reply in the minimax absorbs most of it. LR models fall between −0.3 and +0.2
    (sign flipped: depth-2 search picks positions with itself to move), and trees are no worse than flat LR.
  - So at the league's main depth, search does not exploit trees more than flat models. Their deficit at depth 2
    most likely comes from leaves that predict worse because each learns from a fraction of the data. That points
    to residual leaves (idea 10) and the prequential split rule (idea 11), not seam targeting (ideas 5 and 12).

**Residual leaves tested (idea 10, 2026-11-07 ~01:45; binary chessai-resid2, `models/from-scratch/resid/`).** Clearly
negative as built. In 60 rounds from scratch at a 300k split threshold, the residual trees kept their splits (the shadow
test favoured them in 110 of 110 windows) but played 171-266 Elo below the plain merging tree at the same version, and
their held-out loss was worse (0.594-0.595 vs 0.558). The likely reason is plasticity. Each example steps both the
parent's shadow and the leaf's delta, and that pays off on the round being trained on, where about 1,000 games' positions
share their results, but not on new games. **The lesson for idea 11 and every self-regulating rule: the regulator must
score each round before training on it.** The shadow test scores the stream it trains on, so it rewards whichever model
adapts fastest. Under test now: a log-only fresh-round comparison, then `--revise-on fresh` (merge test on unseen rounds),
and a smaller delta learning rate.

**Offline benchmark (2026-11-07 ~01:55; `models/from-scratch/offline-c15/`).** Models are trained once through cycle 15's
league round-robins 80-83, about 2.4M positions from 83 players, and scored on round-robin 84 (575k positions) by
calibrated log-loss. No self-play, about 20 s per model, deterministic: a fast testbed for structure and regulators.

| Model | Log-loss | Winner log-loss |
|---|---|---|
| LR, 1/3 of the data | 0.6380 | 0.3405 |
| LR, 2/3 of the data | 0.6274 | 0.3337 |
| LR, all the data | 0.6220 | 0.3293 |
| LR, 2 passes | 0.6197 | 0.3271 |
| T-L-Nc, split at 300k (3 leaves) | 0.6199 | 0.3287 |
| T-L-Nc, split at 100k (9 leaves) | **0.6178** | 0.3294 |
| T-L-Nc 100k with merging (`--revise 0`): merged to 1 leaf | 0.6220 | 0.3293 |
| T-L-Ne, split at 300k (3 leaves) | 0.6226 | 0.3298 |
| T-L-Ne, split at 100k (7 leaves) | 0.6242 | 0.3316 |
| T-L-Nc 300k, residual leaves (inherited sums), merging | 0.6263 | 0.3310 |
| Naive Bayes | 0.7328 | 0.4050 |
| League models (self-play): F-L-d2mix v751, the best | 0.6463 | 0.3403 |
| League: T-L-NNs-merge v1066 | 0.6386 | 0.3438 |
| League: F-L-c v2730 | 0.6512 | 0.3561 |
| League: T-L-Nc-c v671 | 0.6517 | 0.3554 |

- Cluster-routed structure helps a little on unseen games, mostly on draws. Error routing and residual leaves hurt.
- **The merge test discarded the better 9-leaf tree.** The stream test is biased in both directions: with copied
  leaves it favours the shadow, and with residual leaves it favours the branch. It must score rounds before training
  on them.
- Data matters more than structure here: 3 times the data is worth 0.016, the best structure 0.004.
- Models trained on league games predict league games far better than any self-play model (0.62 vs 0.646-0.652).
  Self-play models pay a large distribution-shift cost. This suggests a league-trained lineage, which needs the
  user's OK: the games are all our models' own, but it is a new data source.

**Fresh vs stream merge test (2026-11-07 ~02:40; chessai-resid4, `--revise-on fresh`, `models/from-scratch/resid/fresh/`).**
Confirmed: scoring each round's positions with the model as it stood at the round's start (fresh) reverses the stream
test's sign in every run. The stream test favours whichever side adapts faster: with residual leaves the branch, with
copied leaves the shadow, which sees every example.
- **Fresh scoring removes the residual damage:** inherit at v60 went from 171 below the plain tree to 16 below, not
  significant.
- **Nothing beat the plain tree in self-play** at a 300k threshold over 60 rounds. 200-game arenas of near-flat
  models swing about 60 Elo.
- **Offline (offline-c15, T-L-Nc split at 100k):** fresh at z 0 still merges on one bad window (1 leaf, 0.6220). Fresh
  at `--revise 2` kept 5 leaves at 0.6170, the best on the testbed. It is fragile: the root's worst window was z −1.96.
- **League experiment staged (2026-11-07):** T-L-Nc-fresh-c, which is T-L-Nc-merge-c with `--revise 2 --revise-on fresh`
  (binary chessai-resid4). Its comparators are T-L-Nc-merge-c (stream test, z 0) and T-L-Nc-c (no merging).

**Overnight 2026-11-07 (~03:40-04:00), offline-c15, fitted-calibration log-loss (lower is better).**

| Recipe (100k split threshold unless noted) | No merge | Stream merge (`--revise 0`) | Fresh (`--revise 2 --revise-on fresh`) |
|---|---|---|---|
| T-L-Nc (LR leaves, cluster) | 0.6178 | 0.6220 (1 leaf) | 0.6170 |
| T-L-Ne (LR leaves, cnbp router, error) | 0.6242 | 0.6199 | 0.6206 |
| T-L-Le (LR leaves, LR router, error) | 0.6257 | 0.6309 | 0.6220 (1 leaf) |
| T-N-Nc (NB leaves, cluster) | 0.7211 | 0.7328 (1 leaf) | 0.7169 |
| T-N-Le (NB leaves, LR router, error) | 0.8858 | 0.7328 (1 leaf) | 0.7342 |

Flat LR 0.6220, flat NB 0.7328. The fresh test never ends much worse than flat and keeps cluster structure that helps
(the best LR and NB trees on the testbed). Staged as league twins: T-L-Ne-fresh-c, T-L-Le-fresh-c (the user: Ne and Le
rate above Nc in the league), T-N-Nc-fresh-c. The league's own T-N models predict league games no better than F-N-c
unless they collapsed (F-N-c 0.7456; 11-leaf trees 0.754-0.835; 1-leaf T-N-NLs-merge-c 0.7264), but for NB prediction
and play diverge (forgetting improves prediction and hurt play in batch 28).

17. **Pre-split candidates (the user's idea, 2026-11-07: "train the Error router from the beginning so that when the
    split happens it's not as random").** Today an error split starts from identical clones and an untrained router:
    every tie goes to child 0, which takes all the training while the others stay stale copies, so the router learns
    from noise. Instead a leaf trains its would-be split in the shadow (children, router(s), a hash bootstrap to break
    the symmetry) during the window before the split check, and the split installs the trained candidates. Variant:
    promote the candidate split by the fresh test (idea 11). Being built (presplit-src, chessai-presplit).
18. **Cell-assigned DAG.** Route fine and task-blind, group by error with lots of data per decision: a cluster router
    with C cells, M < C shared leaves, each cell's edge re-pointed to another leaf when that leaf's fresh loss on the
    cell's positions is lower by z standard errors. Per-cell decisions aggregate thousands of positions, unlike the
    per-example argmin of error routing; leaves learn from several cells, so they fragment less than a C-leaf tree.
    Saved in the BDT format with shared children, so the league's tournament binary plays it. Being built (dag-src,
    chessai-dag; proposed lineage D-L-Nc-c).
19. **Online Bayesian ordinal probit regression (F-P).** LR's features, Gaussian weights (mean, variance), ordered
    outcomes with a draw margin, assumed-density-filtering updates (TrueSkill/AdPredictor), dynamics noise for drifting
    self-play data, tempered updates because a game's positions share one result. Being built (probit-src,
    chessai-probit); needs a tournament binary switch to join the league.

**Results, 2026-11-07 ~04:30-05:00 (brief checks; the league decides).**
- **F-P (idea 19) works.** Self-play from scratch against F-L-c's own versions at depth 2: v10 +2 (−30..+34), v30 +80
  (+48..+112); depth 1 v30 +38; ~20% cheaper per game than LR. Offline winner log-loss 0.3341 (LR 0.3293). Its draw
  prediction is poor (one global draw margin), but league search uses win-vs-loss log-odds only. F-P-c joins at
  cycle 25 with the tournament binary switched to chessai-probit (identical games and ratings to chessai-cyc5 on a
  full round-robin). Follow-up being checked: probit leaves in trees, children starting from the parent's posterior.
- **Pre-split candidates (idea 17).** Without merging, the error trees reach flat-LR level (Ne 0.6242 → 0.6219, Le
  0.6257 → 0.6246) and their splits stop being lopsided (a 2/27/71 split becomes at worst 47/12/41). With fresh
  merging, Ne and Le collapse to flat; the best switch tree (NNs) came from pre-split plus fresh (0.6171). Promotion by
  the fresh test (variant B) never fired for Ne or Le. Cost about 1.3× CPU per game. League twins of the non-merging
  T-L-Ne-c and T-L-Le-c (`--pre-split`) join at cycle 26.
- **Cell-assigned DAG (idea 18).** Mechanically sound (resume, league play, own step), but at z 2 no cell was re-pointed
  in 20 self-play rounds: the leaf that trained on a cell always predicts it best, so the comparison favours the
  incumbent. Offline 0.6230 (flat LR 0.6220, cluster tree 0.6178). Joins at cycle 26 as D-L-Nc-c, effectively a fixed
  9-cluster → 3-leaf sharing. A fair re-pointing test would need every leaf to have trained on the cell.
- **Naive Bayes's gap to LR is not double counting between whole features.** Weighting each feature's log-likelihood
  (WANBIA, fitted on the win-vs-loss question; numpy prototype in `models/from-scratch/wanbia/`, its naive Bayes
  reproduces the Rust cnbp exactly) moves winner log-loss from 0.4051 only to 0.3874 (LR 0.3293). Deeper NB trees
  don't help either: 27 leaves 0.7199, 81 leaves 0.7428, vs 3 leaves 0.7208; leaf sharing makes them worse. So idea 14
  (parent smoothing for deep NB trees) is unlikely to pay. The NB deficit is in the per-piece-square estimates.
- **Probit leaves in trees (T-P): closed.** Children start from the parent's posterior. Offline they match LR trees on
  winner log-loss, but in self-play they lose to flat F-P more as they split (v10 −163, v30 −233). The branches
  survive only because each leaf learns its own draw margin, which rescales its win-vs-loss log-odds: a per-leaf
  temperature that helps prediction on the tree's own games and creates seams in play. With one shared margin the
  fresh test merges every branch back (v30 single leaf, −70, within self-play history noise). Caveat for all
  single-run checks: an LR tree that merged back to one leaf scored +87 against F-L-c's v30 from its different history,
  so F-P's +80 at v30 is not conclusive either; the league decides.

## Look-ahead training by classifier (2026-11-07 morning; user: "how much different models benefit from training on 2ply vs 1ply games ... and how that correlates to leaf classifier type")

Files: `models/from-scratch/depth-train/` (scripts, arenas.txt, data/, pure/). Elo is the 2-ply-trained side's.

- **Compute-matched league pairs** (same recipe, 1-ply vs 2-ply self-play, versions at equal cumulative self-play
  CPU from cycles.csv; head-to-head 2×1000 games, calibrated, both sides at the same depth; own steps not matched):

  | pair | CPU-s | ply 1 | ply 2 |
  |---|---|---|---|
  | LR jitter d1 → d2 | 1000 / 3000 | −137 / −65 | −253 / +217 |
  | LR jitter d0 → d2 | 5600 | −91 | +143 |
  | LR softmax d1 → d2 | 1000 / 4000 | −100 / −129 | −103 / +138 |
  | MoE d0 → d2 | 1000 / 3200 | −25 / −16 | +14 / +151 |
  | NB d0 → d2 | 1000 / 3200 | −19 / −68 | −128 / −151 |
  | NB tree d0 → d2 | 1000 / 3300 | −21 / −89 | −139 / −164 |

  The league's own single fit (every entered version vs its CPU) agrees: LR-based models trained on 2-ply games pull
  ahead at ply 2 from ~1500-2000 CPU-s (they start slowly: ~50× fewer positions per CPU-s) and stay behind at ply 1;
  NB-based ones are behind at both plies at every CPU level.
- **Equal data, no self-play** (cycle 15's league games split by search depth, both sides ply 2 vs both ply 1,
  506k positions each, 3 epochs): round-84 winner log-loss ply-1-trained / ply-2-trained LR 0.4066 / 0.3535, probit
  0.4120 / 0.3579, NB 0.4150 / 0.4583; arenas ply 1 / ply 2: LR −34 / +201, probit −45 / +174, NB −53 / −203. So NB's
  failure is in learning from look-ahead games, not in its own self-play; the linear discriminative models (LR,
  probit) gain about the same.
- **Pure self-play, no tournament games** (depth-train/pure/: from scratch, seed 21, depth 1 vs depth 2 at equal
  cumulative CPU, 1000 vs 40 games a round; the 2-ply side gets ~20× fewer positions), ply 1 / ply 2 at 450, 900 and
  1800 CPU-s: LR −198/−370, −142/−260, −108/−157 (closing); probit −189/−231, −135/−53, −119/−51 (closing fastest);
  NB −210/−459, −257/−599, −280/−597 (widening). The league's LR pair crosses over between 1000 and 3000 CPU-s.
- **Mechanism data.** Depth-2 self-play is 74-87% decisive for LR-based models but 37-40% for NB-based ones (~250
  plies a game): NB's raw P(draw) is saturated (median 0.9997-0.99999; top-two move gap ~1e-6), so jitter decides
  most of its moves. Own-step gates: NB-d2 lineages gain +40-44 a cycle from tournament games (kept 86-100%) and lose
  it again in self-play; LR-d2 gains +9 (kept 49%).
- **F-P correction.** At matched self-play CPU, F-P-c is ~100-130 below F-L-c at ply 2 in the league (the overnight
  +80 at v30 did not hold). Its self-play is near-random (jitter10 non-top 84% vs LR's 61%) because its learned draw
  band (ε ≈ 2.0 on self-play data) compresses expected-score gaps. Playing self-play at ε = 0 (bin/chessai-probit4,
  `--probit-play-eps 0`) made it much worse (−346 at equal CPU: greedier play, 96% draws, many repetitions), so the
  randomness is not what holds F-P back. Files: `models/from-scratch/probit/fpfix/`. Its memory is: at τ 1e-5 the
  weight variance floors at about 2τ, a memory of about one round. Equal-CPU checks against τ 1e-5 (d2, 2×1000):
  τ 1e-4 −415/−421 and τ 1e-3 −512/−561 at v30/v100 (arena confirmed by a swapped control); τ 1e-6 48/47/63 and
  τ 0 73/48/85 (seeds 94/76) above the baseline at v100/v200/v300. F-P-tau0-c (`--probit-tau 0`) joins at cycle
  38. Files: `models/from-scratch/probit/tau/summary.txt`.
- **League follow-up (cycle 35):** depth-2 twins of the best real T-L tree (T-L-Le-merge-d2jit-c, 100k), the weakest
  original T-L (T-L-NLs-d2jit-c, 100k) and the best active T-N (T-N-Nc-merge-d2jit-c, 200k). Thresholds scaled to
  positions per cycle as for T-N-NNs-merge-d2jit-c. Expect them behind their twins for the first ~6-8 cycles.


## 2026-11-07/08 night (exploration agent; user: "continue with the research, don't forget to save new ideas and conclusions")

Each topic has its own dir under models/from-scratch/ with a summary.txt. Elo without plus signs.

### Conclusions so far
- **errwhy** (from-scratch/errwhy): error routers predict better but play worse at ply 1.
  - The loss is at seams: the 1-ply argmax over hard-routed siblings takes moves that another leaf overrates.
  - The best-child router target is biased towards the result. An LR router that regresses each child's expected
    loss recovers 51% (T-L-Ne-c) and 20% (T-L-Le-c) of the reachable gain.
  - Even perfect routing would only bring T-L-Ne-c to flat LR's level: the leaves bind.
- **Blend** (soft routing at play time; the leaves blended by the routers' probabilities), re-judged against cal-own
  (errwhy section E):
  - at ply 1, 58 (T-L-Le-c) and 45 (T-L-Le-fresh-c) over the same tree calibrated on its own games; the earlier 95
    and 75 were against hard routing and included calibration;
  - at ply 2, 41 and 1, so uncertain;
  - 4-5x CPU per game.
- **leaguecal** (from-scratch/leaguecal): the league's fixed 0.1-nat softmax over held-out-calibrated specs confounds
  the ratings.
  - Own-play calibration slopes range from 0.55 to 2.03. d0 LR entries play hot and NB/d2 entries play cold, and
    colder wins.
  - Head to head the effect is 132 Elo per unit of ln(slope ratio) at ply 2. It explains most of the d2-trained lead
    over d0 LR trees.
  - Fix: own-game calibration per version (cal-own), at 1-3 CPU-s a version. It is a stage-2 option.
- **lrshare / leshare**: leaf sharing (every leaf also steps on 0.1 of the other leaves' log-loss) makes no-merge LR
  error trees better offline.
  - T-L-Ne: -0.0058. T-L-Le: -0.0047, and -0.0063 at share 0.3, which brings it to flat LR's level or better.
  - It halves the leaf disagreement and shrinks seams in self-play.
  - With merges it changes nothing.
  - Lines: T-L-Ne-share-c (stage 2), T-L-Nc-share-c (being deployed), T-L-Le-share-c (proposed).
- **moe8**: MoE with 8 experts is not broken. It uses about 4.4 of its 8 experts and costs 1.6-1.8x MoE-c. It is a
  candidate.
- **nbcal**: calibrating NB's self-play policy on its own games doesn't change how random it plays. Skipped.
- **tangate**: a TAN gate over LR experts is as hard as the NB gate and worse than MoE-c offline. Broken as a
  candidate.
- **Topic 1, leshare** (from-scratch/leshare): leaf sharing on T-L-Le, offline against no sharing.
  - Share 0.1: -0.0047 [-0.0071, -0.0024]. Share 0.3: -0.0063, with 0.3 beating 0.1 by -0.0016 [-0.0026, -0.0005].
  - That takes the LR-router error tree from 0.0042 worse than flat LR to level (0.1) or about 0.002 better (0.3).
  - Smoke: the split's loss to its shadow shrinks by a quarter, seams by a quarter, at 1.4x CPU.
  - Lines: T-L-Le-share-c (0.1), optional T-L-Le-share3-c (0.3).
- **Topic 2, eloss** (from-scratch/eloss; bin/chessai-eloss): the expected-loss router target used for training is
  harmful.
  - T-L-Le offline: 0.0235 [0.0196, 0.0274] worse; with sharing, 0.0177 worse.
  - The router's pick also picks who trains, so routing to the child already best there makes specialization feed
    itself: root regret falls to 0, while leaf-pair disagreement rises from 0.50 to 1.22.
  - Switch trees never switched to their LR error routers offline, even at --route-switch 0.
  - Lesson: whatever assigns training examples must keep the leaves' data broad. Routing quality and data assignment
    should be separate jobs.

- **Topic 3, blendsp** (from-scratch/blendsp; bin/chessai-blendplay, --play-blend TAU): blended self-play is not
  worth its cost. No blend lineage.
  - Pooled over five version pairs, from scratch at equal data: 10 (-1 to 21) at ply 1 and 69 (45 to 93) at ply 2.
    So per round, blended data trains better ply-2 players.
  - At equal CPU (2.2-2.3x per round): -35 (-47 to -24) at ply 1, -73 (-98 to -47) at ply 2.
  - Resumed from the league's T-L-Le-c v326, equal data: -20 (-32 to -8) at ply 1, -13 (-40 to 14) at ply 2.
  - **Version noise:** adjacent versions of one lineage differ by up to 75 Elo at ply 1 and 200 at ply 2 against the
    same opponent, with the tree's structure fixed. Single-pair results misled; pool versions. For the league,
    single-version ratings at ply 2 are this noisy, and a peak rule picks lucky versions.
- **Leaf-share dose** (from-scratch/leshare, "Dose sweep"): every family has an interior optimum, and error routing
  needs more sharing than cluster routing.
  - Offline log-loss at 0.1 / 0.3 / 0.5 / 1.0: Le 0.6111 / 0.6095 / 0.6101 / 0.6116; Ne 0.6086 / 0.6075 / 0.6087;
    Nc 0.6067 / 0.6074 / 0.6086. Flat LR is 0.6116.
  - At 1.0 every leaf is the flat LR, exactly.
  - Lines: T-L-Le-share3-c (recommended over 0.1), optional T-L-Ne-share3-c, both on chessai-sparsefit.

- **Topic 4a, playrouter** (from-scratch/playrouter; bin/chessai-playrouter, --play-router): online eloss play
  routers are harmful. No line.
  - Each delegator learns a second, play-only LR router by regressing its children's expected loss. Training is
    unchanged: the leaves are identical, and so are the self-play ladders.
  - Offline: 0.0034-0.0071 worse on every recipe. Arenas: -30/-32 at ply 1, -67/-26 at ply 2.
  - Self-play smoke, five version pairs: -37 at ply 1, -80 at ply 2.
  - Spliced hybrids: in T-L-Ne the damage is mostly the root's (covariate shift down the tree), but bottom-only play
    routers don't gain either.
  - errwhy B's gain needed frozen children, scored one delegator at a time; online regression doesn't reproduce it.
- **Topic 4c, sharesize** (from-scratch/sharesize): sharing makes trees of 25-35 leaves as good as 7-11, where
  without it they lose 0.005-0.010. But the extra leaves gain nothing on 2.4M positions, and training cost grows
  with the leaves.

- **Topic 4d, polyak** (from-scratch/polyak; bin/chessai-polyak, --polyak DECAY): averaging a lineage's last versions'
  LR weights (Polyak) helps mature lineages and costs young ones.
  - avg5 against single: T-L-Le-c 40 at ply 1, every window positive, and -11 at ply 2; T-L-Ne-c 7 and 28;
    b-hard v40-v60 21 and 45.
  - Held-out loss -0.005 (b-hard) and -0.001 to -0.002 (league T-L-Ne-c, T-L-Nc-c). Version-to-version swings shrink
    3-5x.
  - In a young run (EMA 0.8, v22-v26): -15 at ply 1, because the lag costs while the model improves fast.
  - Proposed as driver-side averaged entries for mature lineages (polyak/avg.py), at no training cost.
  - Champion-stabiliser angle (held for the user: entry types and the champion test are design changes):
    - The new league's champion test compares single versions at 2 SE of the fit (SE about 14).
    - Adjacent single versions differ in true strength by up to 75 at ply 1 and 200 at ply 2, far more than that SE.
      So the test picks lucky versions, and the versions after a lucky champion regress below it and burn patience.
    - If the field's entries (or just the champion test's candidate and champion) were avg5 or EMA files, the test
      would compare stable entries: fewer lucky champions, and fewer lineages that finish on an unlucky draw.
    - The file is an ordinary BDT file, so the tournament binary doesn't change.

- **Topic spdrop** (from-scratch/spdrop; the coordinator's diagnostic of the new league's post-self-play drop):
  - Real, at ply 2 only, and kind-specific. New league, cycles 2-4, current version against champion at ply 2:
    FM -147, T-L-Le* -105, MoE -91, T-L-LLs* -70, T-N-Le -60, flat LR -21, the rest within about 30. At ply 1 the
    same versions hold or gain (FM 71, T-L-Le 44).
  - The first run's switches between self-play and own steps agree. Waking from dormancy cost -148 at ply 2 for
    MoE/FM/T-L-Le/T-L-LLs and -50 for the rest, and about 0 at ply 1.
  - Mechanism: depth-0 self-play moves the model's skill from ply-2 to ply-1 positions, all of it in the first round
    of 1000 games. On T-L-Le-fresh-c the league ply-2 winner loss went 0.426 to 0.450 and ply 1 0.343 to 0.307. The
    own step, half ply-2 games, moves it back.
    - FM factors, MoE gates and LR routers over LR leaves swing most; flat LR barely swings.
    - This matches depth-train's equal-data result: ply-2-trained LR was 201 at ply 2 and -34 at ply 1.
  - Not the cause:
    - Draw drift: post-self-play raw log-loss is 2.3-2.8 on ply-2 league positions and the cal-own offsets flip
      sign, but in every lineage, controls too, and the policy ignores draws.
    - The cal-own slope: it moves the wrong way.
  - The own step wins the whole drop back. In a league-policy panel, own-step versions are level with their
    champions (mean 5, se 15, four lineages). The champions are own-step versions from the first run's dormancy, so
    the champion test compares the bottom of each swing with a top.
  - The gate plays the arena's default policy (WldScoreJitter10), not the league's log-odds softmax:
    cycle.py's gate_match passes no policy flags. All gates of both runs are self-play-policy measurements.
  - Fixes:
    - (a) Rating the own-step version as E plus its gate: right on average (152 against the panel's 134), but off by
      up to 100 for a single lineage. MoE-c would have made a false champion.
    - (b) The own step first, on last cycle's games: it recovers as much as this cycle's games (119 against 124,
      three lineages), and the entering version would be level with its champion. This one works.
    - (c) A smaller self-play rate: not cheap. One round already does the whole shift, and weight interpolation at a
      quarter step costs 85.
  - T-L-NLs-d2jit-c's drop was something else: its self-play is at depth 2 with 25% draws, so there is no ply shift.
    It had one bad version, a lucky old-peak champion and a lower slope. Its v1728 is level with the champion.

### New ideas (2026-11-07/08 night)
- **Eloss play router** (topic 4 candidate; tested as topic 4a: negative online, see above):
  - Each delegator trains a second LR router with the expected-loss target on every scored example. Play and
    prediction use it; the training router keeps assigning the training examples.
  - Model files carry the play routers in the routers' place, so no tournament change; states keep both.
  - Offline it is exactly paired.
  - Evidence: errwhy B's refitted LR regress routers on frozen T-L-Ne-c children gained 0.013 (51% of the
    reachable 0.026).
  - Best fit: error trees with CNBP routers (T-L-Ne). Not cluster trees, where regress gained nothing.
- **Leaf-share schedule:** young or small leaves borrow more, e.g. share = s0 * k / (k + n_leaf). Share 0.3 beat 0.1 on
  T-L-Le, so the dose isn't settled. A sweep of 0.3/0.5/1.0 is cheap offline. At 1.0 every leaf trains on everything
  and the routing stops mattering.
  - The sweep ran: interior optima at 0.1-0.3, and the curve is flat near them (about 0.001), so a schedule could add
    little.
- **Blend for share trees:** sharing brings leaves closer (0.15-0.29), so blend's play gain should shrink. Test
  whether share 0.3 plus hard routing matches share 0.1 plus blend at a fifth of blend's play cost.
- **Separate the training router from the play router** (generalises the eloss play router): the training router may
  even be random or balanced, to give every leaf broad data, while the play router is fitted to the frozen leaves.
  Cheapest version: refit play routers once per version on that round's games, by 2 passes of regress.
- **Data-assignment regularizer for error routers:** mix the training router's pick with a uniform pick (epsilon of
  examples go to a random child; or train the routed child plus share). That is a direct check of "the best-child
  target's noise keeps the leaves general".
- **Cal-own everywhere a softmax reads a spec** (from leaguecal): self-play's own move choice might also suffer from
  the scale; d0 LR self-play with an underconfident model plays hot. Not measured.
- **Seam-weighted sharing** (from topic 3): hard self-play's useful data is at the seams, where the routed leaf and
  its siblings disagree. Share only there: a sibling also trains on an example with weight growing with their
  disagreement (or with the training router's uncertainty). It targets what blend hides, without diluting the
  leaves' specialization elsewhere. Worth it if the dose sweep finds an interior optimum.
  - Its motivation, that blended play hides seams and hurts search, fell when the versions were pooled.
- **Why blend data hurt at ply 2** (run, blendsp/run3.sh): against their common start v326, a-hard v341 is 15 at
  ply 1 and -43 at ply 2, a-blend v341 1 and -121. These are single versions, and the pooled (a) result at ply 2 is
  level, so this was version noise more than blend.
- **Rule-symmetry augmentation stays on hold:** the user put colour-mirrored training on hold on 2026-11-05 ("almost
  begins to encroach on the 0 knowledge assumption"). Topic 3 points at data per leaf, but it is not built.
- **TD(lambda) targets** (later, if ever): about 89% of self-play games are draws, so most result labels carry little
  information, and bootstrapping on the model's own later predictions in the same game would lower the label noise.
  It is against the training standard (result labels; "data optimisation not now", 2026-11-05) and close to the look-ahead labels the
  user ruled out, so it is only listed.
- **Play routers in self-play stay off:** chessai-playrouter's self-play keeps the training routers in both players
  (the previous version is an in-memory clone), so a play-router lineage's data equals its base lineage's, and only
  its league files differ. By topic 3's lesson, routing self-play by the play routers would hide seams too.
- **Per-version play-router refit** (after topic 4a): refit each delegator's play router on frozen children with
  the next round's games before training on them, the errwhy B way. Unproven: bottom-only online play routers
  already failed, and errwhy B scored delegators one at a time. Low priority, since the leaves bind.
- **Split limit for share lineages** (after topic 4c): with sharing, size costs nothing offline, so T-L-Nc-share-c
  with --split-limit 20 would test whether the league regime (millions of positions a leaf) uses more leaves.
- **Sampled sharing:** each example trains k random other leaves at weight share x (L-1)/k, unbiased, so the cost
  stops growing with the number of leaves.
- **Version-pooled judging** (from topic 3): adjacent versions differ by up to 200 Elo at ply 2. Arenas and league
  decisions should pool a window of versions; a peak rule over single versions picks lucky ones.
- **Polyak in self-play** (after topic 4d): let self-play's players use the averaged weights, as the league entries
  would. The data changes, so it is a training experiment, and data optimisation is "not now".
- **Average restarts at splits:** chessai-polyak's split children inherit the parent's average. Restarting it at
  the split (and for shadows) is cleaner for from-scratch runs, and irrelevant for mature lineages.
- **Own step first** (spdrop fix b; held for the user, a driver change):
  - Each cycle: self-play, then the own step on the previous cycle's league games, gated against the post-self-play
    version, then the kept version enters the field and the tournament.
  - The league then rates versions in the same phase as the champions. Cost: keeping one cycle's pool (about 1 GB
    for 11 lineages, more for the field).
  - Offline it recovers 119 of the drop against 124 with this cycle's games.
- **Gate in the league's policy:** gate_match should pass --a-policy log-odds-softmax --b-policy log-odds-softmax
  --temperature 0.1. A 100-game gate has an SE of about 40, three times the fit's.
- **Self-play depth mix for swinging kinds:** the swing comes from depth-0 self-play positions against ply-2 league
  positions, and d2jit lineages show none.
  - A middle way: play a fraction of each round's games at depth 2. This is search in play, not in labels.
  - It is a new recipe and a one-factor lineage, e.g. FM-c with a quarter of its games at depth 2.
- **League-position replay in self-play rounds:** mix the last own-step pool into each round, so that neither step
  overwrites the other. It changes the data, so it is listed only ("data optimisation not now").
- **Swing diagnostic for new kinds:** E minus O on the league sample's ply-2 and ply-1 positions (spdrop/evals.sh,
  seconds a version) predicts a kind's ply-2 drop before any arena. FM 0.041, T-L-Le 0.026, MoE 0.019, flat LR
  0.009.
- **Count-based routing where training alternates between distributions:** LR routers, MoE gates and FM factors re-fit
  within one round; CNBP-routed trees barely swing. NB-leaf trees under LR routers mostly don't swing either, so
  it isn't the router alone.
- **Average across the swing?** Not with LR routers: halfway between an own-step and a post-self-play version (in
  weights) was worse than either end (1538 against 1695 and 1602). This bears on Polyak windows that mix phases.
  - A run's versions alternate: a cycle's self-play versions, then its own-step version (E+1, when kept). So an avg5
    window should stay within one cycle's self-play versions.
  - Topic 4d's league windows may have straddled own-step versions. T-L-Le-c's ply-2 result there (-11, against 40
    at ply 1) could reflect that. Unchecked.


## 2026-11-08 (day; the coordinator's fork, research restarted at ~12:00 after idling from 00:23)

### League
- **T-L-Ne-share3-c deployed** at the cycle 27->28 boundary (loop pid 571237): T-L-Ne-share-c's line with
  --leaf-share 0.3 instead of 0.1, on chessai-sparsefit. Why: T-L-Le-share3-c's champion v264 was at 1801 in cycle
  26, the best of the six T-L-Le lineages (T-L-Le-fresh-c 1770, T-L-Le-c 1651), climbing steadily from 1626 at
  cycle 10. The dose sweep put T-L-Ne's optimum at 0.3 too (0.6075 against 0.6086 at 0.1). Records:
  champion-sim/stage2/report.md, cycles/champ/README.md.

### Conclusions
- **assign** (from-scratch/assign; bin/chessai-assign, --assign-epsilon E): sending a random share of each
  delegator's training examples to a uniformly drawn child is not a league line.
  - Offline (offline-c15, T-L-Le's recipe): E 0.05 / 0.1 / 0.2 gain 0.0017 / 0.0012 / 0.0027 log-loss against
    le-100k (E 0.2: [-0.0043, -0.0011]). That is about 40% of sharing 0.3's gain, and le-e20 stays 0.0036
    [0.0014, 0.0058] behind le-s03.
  - On top of sharing 0.3 it adds nothing: all three within 0.0005, intervals about 0.001 wide.
  - Play check of E 0.2 (no sharing), from scratch at equal data, five version pairs, cal-own, league policy:
    9 (-2 to 20) at ply 1, -17 (-41 to 7) at ply 2. Level.
  - It halves the self-play leaves' disagreement (0.80 to 0.42 at v24) without a stronger player. So the leaves'
    closeness is not the target in itself. Sharing's gain comes from every leaf seeing every example with a weight,
    not from leaves that agree.
  - The hypothesis "broader leaf data helps error trees" holds offline in direction, but taking examples away from
    the routed leaf (epsilon moves the example) is a poor way to broaden. Sharing adds data without taking any.

### New ideas
- **Additive random assignment (a sampled share):** with probability E, a random other leaf also trains on the
  example at weight 1, and the routed leaf keeps it. It sits between epsilon (which moves the example) and full
  sharing (every leaf, at weight s). It is the "sampled sharing" idea above, whose cost stays flat in the number of
  leaves. Only worth building if the share lines' training cost starts to bind.
- **Share lines for the other LR-leaf trees in the league,** if T-L-Ne-share3-c follows T-L-Le-share3-c. The
  unmerged LR-leaf trees without a share line: the switch trees T-L-NLs-c, T-L-LLs-c and T-L-NNs-c, and D-L-Nc-c (a
  DAG whose cells already point to 3 LR leaves, --dag-leaves 3; --leaf-share is a different thing, on chessai-dag
  unchecked). Check each offline first. Merge trees gained nothing from sharing offline (lrshare), so no merge-tree
  share lines.

### Batch 2 (the coordinator's fork, ~12:25-13:00)
- **swshare** (from-scratch/swshare): leaf sharing for the switch trees and the DAG, offline as leshare (offline-c15,
  100k threshold, paired over 10 parts). None clears the batch's bar (a gain of 0.002 or more, interval excluding 0),
  so no share line.
  - T-L-NLs = T-L-NNs offline: 0.1 -0.0018 [-0.0031, -0.0005]; 0.3 -0.0016 [-0.0037, 0.0006]. T-L-LLs: 0.0000 and
    -0.0001 (offline it is flat LR). D-L-Nc on chessai-dag (same LR-leaf sharing step): 0.1 0.0000, 0.3 -0.0014
    [-0.0026, -0.0002], which brings the DAG to flat LR's level.
  - Identity at share 0: cyc5 and sparsefit give md5-identical models for all three switch recipes.
  - Offline no switch tree switches (as in eloss), so NLs and NNs are the same tree and the check sees only the
    cluster-routed phase. In the league they did switch (NLs 2 delegators, NNs 1, LLs 1). Error-routed delegators are
    where sharing helped most, so this check probably understates it for the league's switch trees.
- **FM-d2jit-c deployed** at the cycle 28->29 boundary (from-scratch/fmd2): FM-c plus --depth 2, from scratch. FM had
  the biggest ply-2 drop after depth-0 self-play (-147). MoE-d2jit-c already exists (since the first run's cycle 16;
  champion 1761 against MoE-c's 1715 at cycle 27), so MoE's twin was not added. fm d2 costs 0.222 CPU-s a game (load
  11.8): about 6 x 40 self-play games a cycle, so most of its data will be the own step's league games.
- **Polyak window check** (from-scratch/polyak, "Window check"): 4d's T-L-Le-c result was mostly phase, not noise
  reduction. Three of its five windows ended on an own-step version.
  - Today's league, cycles 23-27, avg5 against single inside one cycle's self-play versions: ply 2 8 (-13 to 29), ply
    1 -6 (-15 to 4). Level.
  - Windows ending on the own-step version: ply 2 -102 (-124 to -80), ply 1 96 (85 to 107). The spdrop swing,
    measured on weights.
  - For the Polyak-entries decision: no case for T-L-Le-c; windows must stay inside one cycle's self-play versions;
    T-L-Ne-c's 28 at ply 2 needs the same check. Within a cycle, mature T-L-Le-c's versions don't differ enough for
    averaging to help; the big swings are between phases.

### New ideas (batch 2)
- **Resumed share twins for switched trees:** a twin resumed from a league switch tree's current state with
  --leaf-share added differs in that one factor from then on, and starts with the switched delegators that the
  offline benchmark can't produce. It would test sharing where error routing is live.
- **Phase-aware entries and tests:** anything that pools or compares versions (Polyak windows, version-noise
  estimates, champion tests) should tag each version's phase (self-play or own step). Comparing across phases
  measures the ply-1/ply-2 swing, not the factor. The own step first (spdrop fix b) would put every rated version in
  the same phase.
- **Data mix for depth-2 lineages:** with about 240 self-play games a cycle against the own step's ~3,700 league
  games, FM-d2jit-c and the other d2jit lineages learn mostly from league positions. Their results speak to "depth-2
  self-play plus league data", not depth-2 self-play alone. The positions column on the dashboard should show it.

### Coordinator notes (2026-11-08 day)
- **Data per entry** (the dashboard's new positions column): the top entry, F-L-d4mix-c v363, trained on 7.6M
  positions, 93% of them own-step league games. The median ply-2 entry has 258M, F-L-c about 0.9B, and the d2/d4
  lineages 12-35M. Correlation only. Own-step-only lineages are ruled out (user, 2026-11-07: no purely
  tournament-trained lineages), so this is followed up through the d2jit data-mix angle above, not a league-trained line.
- **50/50 look-ahead/result labels against the ply-2 drop** (user's question, 2026-11-08): not the fix. The labels
  don't change which positions are trained on; 2-ply self-play does (d2jit lineages show no drop), and the own step
  first fixes the measurement.
  - The mix's own effect at depth 2 (F-L-d2mix-c against F-L-d2play-c, labels the only difference), cycle 27: 12 on
    current versions, 57 on champions, about 30 SE. F-L-d2jit-c (result labels, jitter moves) is ahead of both.
    F-L-d4mix-c's 1813 champion against its 1755 current version looks partly lucky (20 games a cycle).
  - Plan put to the user: mix twins of FM-d2jit-c and MoE-d2jit-c (one factor: --label-depth 2 --label-result 0.5)
    once FM-d2jit-c has settled. Look-ahead labels are outside the training standard except for the user's F-L
    experiments, so the user decides.

### League (the user's request, ~13:05)
- **F-L-d4jit-c deployed** at the cycle 29->30 boundary (from-scratch/d4jit; loop pid 588960): F-L-c plus --depth 4, the
  depth-4 jit twin; one factor against F-L-d2jit-c. F-L-d4mix-c differs from it in two (softmax 0.3, the 50/50 label
  mix). Own step on result labels. 3.7 CPU-s a game from scratch, about 2 x 10 self-play games a cycle, so mostly the
  own step's league games. Question it can answer with F-L-d2jit-c and F-L-d4mix-c: does deeper self-play search
  help on its own, and how much of F-L-d4mix-c's lead is the label mix or the softmax?

### LR against NB (lrnb; the user, ~13:40: "whether LR are strictly better than NB models or LR are better in more circumstances than NB models are but are worse in some also")
- **LR is better everywhere we looked** (models/from-scratch/lrnb/summary.txt). NB is not a specialist for any kind
  of position; it is a weaker model of the same thing.
  - Setup: offline-c15. Each model's calibration is held out, fitted on the even or odd test parts and scored on
    the other parts. Intervals are 95% over the 10 parts.
- **No crossover in data size.** Over 10k to 2.3M training positions NB is never ahead.
  - At about 10k positions the two tie on log-loss: NB-LR 0.016 [-0.003, 0.035], and NB's winner loss is still
    worse.
  - The gap then grows to 0.110 [0.079, 0.142] at full data. NB flattens at about 0.74 from 300k positions while
    LR keeps improving.
- **No group of positions where NB wins.** There are 58 label-free groups: phase, ply, side to move, castling
  rights, rarity, each model's confidence, and whether the two agree on the likeliest result.
  - NB is worse in 55 of them, with intervals above zero.
  - It ties in 3: where NB is at least 97% confident (1.8% of positions), where F-N-c is, and on part-boundary
    positions. The league's F-N-c against F-L-c gives the same picture.
  - The gap is smallest where NB is confident, in the opening (ply 1-10: 0.070), and in positions made only of
    common entries.
  - It is largest where the two disagree on the likeliest result (0.198, 20% of positions), in long games (ply
    251 and up: 0.234), in bare endgames (0.147) and in draws (0.143).
  - Rare entries don't help NB. Nothing in the test set is truly rare: every (square, piece) entry occurs at
    least 598 times in training.
- **Not predictable either.** A gate mixing the two per position gained nothing against LR alone:
  - full gate (an LR over the 70 features plus both outputs, fitted on the other fold): -0.0002 [-0.0046,
    0.0041];
  - gate on the outputs only: -0.0006;
  - constant mix: 0.0003.
- **The oracle's -0.109 is loose.** It picks after seeing the result, so it exploits outcome noise. The same
  oracle over two LRs (lr-all and the league's F-L-c) also "gains" 0.071.
- **Implication for the LR+NB mixture-of-experts question:** a mix of an LR and an NB trained on the same data,
  with the experts fixed, gains nothing. Any value NB has in trees (T-N-*) has to come from what the router
  gives each leaf, not from NB beating LR somewhere.

### Tree balance (the user, ~14:00: "does it use pre-splitting? can you check how well balanced it is?")
Measured from the per-node counts in the tree files: the counts grow only for the routed path, so the difference
between v(N-7) and vN is the routing of cycle 30's self-play (about 1.8M positions, no splits in between). Script and
output: models/from-scratch/balance/ (leafshare.py, balance-c30.txt).

| tree | routing | leaves | largest leaf | leaves under 2% | effective leaves (exp entropy) |
|---|---|---:|---:|---:|---:|
| T-L-Nc-share-c (best tree, #3) | NB cluster, share 0.1 | 11 | 25% | 3 | 8.2 |
| T-L-Nc-c | NB cluster | 11 | 20% | 2 | 8.5 |
| T-L-Ne-share-c | NB error, share 0.1 | 9 | 43% | 1 | 5.8 |
| T-L-Ne-c (#5) | NB error | 11 | 47% | 4 | 5.0 |
| T-L-Ne-pre-c | NB error, pre-split | 11 | 41% | 3 | 6.1 |

- Only T-L-Ne-pre-c and T-L-Le-pre-c use --pre-split. Cluster trees break the tie with hash targets for a new
  router's first split_threshold/5 = 1M positions instead.
- Cluster trees stay fairly balanced. Error trees concentrate: one leaf takes 41-47% and some leaves are dead
  (T-L-Ne-c's d6 routes 97/2/1, two leaves at 0.0%).
- Pre-splitting balances the start of a split but not where it ends: T-L-Ne-pre-c's later splits route 1/96/3 and
  0/10/90, and it is barely more even than T-L-Ne-c (6.1 against 5.0).
- Balance does not track strength: T-L-Ne-c is the most lopsided and still #5 overall.

### Merge trees: do split phases play better? (~14:20, while answering "what is T-L-Te")
- T-L-Te-merge-c has split 9 times and merged back 8 (v37-55, 134-153, 214-233, 274-293, 374-393, 454-473,
  514-533, 616-635; v676 on is split now): each split lasts one 5M-position revise window, so it is flat LR about
  75% of the time. Its last five champions (v143, v219, v283, v460, v687) all come from split phases.
- models/from-scratch/balance/splitphase.py: each ply-2 entry's Elo at the last tournament minus the median of the
  2 entries before and 2 after it, split entries against flat ones. T-L-Te-merge-c 38.6 ± 12.3. The rest are mixed:
  LR-leaf NB-router trees lean the same way (T-L-Ne-merge-c 9.8 ± 10.8, T-L-Nc-merge-c 14.0 ± 10.0, T-L-NNs-merge-c
  16.7 ± 11.0), T-L-LLs-merge-c -30.6 ± 13.3, NB-leaf trees negative (-12 to -74). A lead, not a conclusion: many
  comparisons, stale ratings of idle entries.
- Idea: T-L-Te-c, T-L-Te-merge-c's line without --revise 0 (one factor), to test whether keeping TAN splits helps
  play. Offered to the user, not deployed.

### NB against LR error routers on the same fixed leaves (routercmp; the user, ~14:30: "maybe there's a difference in how well they predict the correct leaf to send the position to")
models/from-scratch/routercmp/summary.txt; bin/chessai-routercmp (md5 8d607093) adds refit-routers (fresh routers of
one kind, leaves frozen, the tree code's own error routing) and route-stats.
- On each tree's own self-play positions (T-L-Ne-c v1681's and T-L-Le-c v1505's leaves), NB- and LR-routed trees
  differ by at most 0.0017 nats of calibrated log-loss. The sign flips with the leaf set and the data size (10k to
  2.3M positions). There is no consistent small-data edge for NB.
  - Full data, NB minus LR: ne -0.0007 [-0.0011, -0.0004], le 0.0013 [0.0004, 0.0022].
  - Fresh routers reproduce each tree's own routing quality.
- LR routers pick the realized-best child more often (6-17 points at the root), by routing almost everything to the
  child that is most often best. NB routers lose the same or less.
- The consistent difference is concentration. On the same leaves, LR routers use fewer leaves in all 4 settings
  (effective leaves at full data, NB against LR: 6.4/2.0, 6.2/4.3, 5.5/4.6, 2.6/1.0).
  - Live lineages: the unshared LR-router error trees run on 1.5-1.8 effective leaves (about 84% of positions to one
    leaf). T-L-Le-c's champion sent 99.9% of cycle 30's self-play to one leaf.
  - The NB-router ones run on 5.1-6.5. T-L-Le-share3-c runs on 8.5.
- Reading: the Ne > Le league gap is more likely training feedback than routing skill. LR routers favour the majority
  leaf, which gets the data, and the tree collapses towards one LR. Sharing breaks it. Not proven: the leaves were
  frozen.
- On offline-c15 (league positions), the leaves are badly calibrated (raw 1.85-1.96 against 0.63-0.65 calibrated).
  Raw-loss targets there favour nearly untrained leaves, and every refit loses to the best single leaf. That setting is
  a poor stand-in for routing.
- Idea, not built: measure the feedback directly. Train T-L-Le-c's and T-L-Ne-c's recipes from one tree with leaves
  live, the router kind as the one factor, and track the effective leaves per round.

### LR error routers lock in (coordinator, ~16:00; models/from-scratch/balance/summary.txt)
- Per round, T-L-Le-c and T-L-Le-pre-c send 97-100% of each self-play round to ONE leaf, a different one after each
  own step (T-L-Le-c: L12, L6, L12, L6, L6, L4, L4, L12/L11, L4 over cycles 25-33). Own-step rounds spread (3-5
  effective leaves). NB routers (T-L-Ne-c) spread every round (4.5-5.5); LR routers with share 0.3 also spread.
- Reading: a calibrated router's argmax under a weak signal is the child most often best; the locked leaf then gets
  all the self-play and stays best. NB's per-position overconfidence spreads its argmax.
- So routercmp's "collapse" is a lock-in at the scale of a cycle, not a slow slide; the long-window averages hid it.
- Idea (to build): route by lift, argmax_k log p(k|x) - log pbar(k), pbar the router's running mean prediction.
  T-L-Le-lift-c, one factor vs T-L-Le-c.
- (same day, ~16:15) The cell DAG D-L-Nc-c has collapsed the same way: re-pointing fired from v56 on, mostly towards
  leaf 3. Cells -> leaves went from [1,2,3,1,2,3,1,2,3] to [3,2,3,3,3,3,1,3,3]. About 98% of recent positions go to
  leaf 3, and leaf 2 has been starved since v96. It plays as flat LR (both champions 1784). Error-driven assignment
  (router targets or re-pointing) pulls towards the most-trained model. Sharing and balanced targets are the
  counter-moves.

### League additions at the cycle 33->34 boundary (2026-11-08 ~15:45, the coordinator's fork)
Under the user's "please continue your research indepedently into both trees, dags and routing. Feel free to add
whichever models you want to the tournament". Staged in cycles/champ/staged-c33/, armed 15:44 (watcher pid 617950).
- T-L-Te-c: T-L-Te-merge-c's line without --revise 0 (keeps its splits). Tests the user's intuition that error
  routers don't need merging; T-L-Te-merge-c's split phases rated 39 ± 12 above its flat ones.
- T-L-Nc-share-merge-c: T-L-Nc-share-c's line plus --revise 0. Tests the user's intuition that cluster routers need
  merging, on top of sharing (T-L-Nc-share-c is the best tree, 1810 at cycle 30).
- D-L-Ne-c: the BDD (bdd.rs, the user's router-split DAG: a delegator whose routing stopped improving splits into a
  router over copies of itself that share its children), T-L-Ne-c's recipe otherwise. The BDD refuses
  --reset-leaves, which only logs in a BDT (byte-identical models with and without it), so the line drops it.
  Offline check (models/from-scratch/bddcheck/summary.txt, 100k threshold, one seed): BDD minus BDT log-loss -0.0018
  [-0.0043, 0.0006], winner 0.0001 [-0.0017, 0.0019]; 3 leaf + 2 delegator splits, 7 leaves each; the delegator splits
  chain (root -> 4 -> 13) but the sub-routers route differently (one sends 99.9% to one child, another spreads
  30/38/33). Watch in the league: how many of its 5 splits go to delegators, and whether it ends with fewer leaves.

### Balanced error targets (the coordinator's fork, 2026-11-08 ~16:00; models/from-scratch/bal/summary.txt)
- --error-target balanced (bin/chessai-bal): Best's one-hot target, router step weighted by 1/(K pbar_best) (pbar a
  per-delegator EMA of how often each child is best, horizon 1M, clipped weights [0.1, 10]); the router's argmax
  routes by lift. Identical to chessai-cyc5 without the option (models and states, from scratch and resuming).
- Continuing T-L-Le-c v1538 for 16 rounds: Best 100% on one leaf every round; balanced spreads after its 5-round
  warm-up to 1.8-3.0 effective leaves, and predicts held-out self-play better (log-loss -0.0029 [-0.0049, -0.0010]).
- T-L-Le-bal-c (T-L-Le-c + --error-target balanced, from scratch) staged for the 34->35 boundary (staged-c34).
- Ideas: T-L-Ne-bal-c if it helps (NB routers already spread); a shorter horizon or a pbar that starts from the
  router's own mean prediction would skip the warm-up; switch error routers and BDD comparisons could take it too.

### Balanced targets on NB routers; a bigger best tree (coordinator's fork, 2026-11-08 ~16:30; models/from-scratch/bal/summary.txt)
- chessai-bal without the option = T-L-Ne-c's binaries on its line (self-play vs cyc5 from scratch with 5 router
  splits, exact line, resume; own step vs cyc5-labels): byte-identical.
- Continuing T-L-Ne-c v1751 for 16 rounds, balanced changes nothing: a CNBP router's counts hold its whole past (about
  400M positions at the root), so reweighting new counts barely moves it (held-out log-loss -0.0005 [-0.0011,
  +0.0002]). The LR routers of the Le check moved within 5 rounds.
- From scratch (split threshold 200000, 24 rounds), balanced NB routers spread a little more once the tree has 11
  leaves (effective leaves 7.0-7.6 against 6.2-7.0; top leaf 28-33% against 30-43%) at level held-out loss.
- Deployed at the cycle 35->36 boundary: T-L-Ne-bal-c (vs T-L-Ne-c) and T-L-Nc-share-l10-c (T-L-Nc-share-c with split
  limit 10: does a bigger tree help once sharing keeps splits cheap?).
- Note for later: NB routers' inertia (counts never forget) is why they don't lock in, and also why they can't
  re-route when leaves change. A router with bounded memory (forgetting counts) would sit between NB and LR.

### What router splits do in the BDD (bddstudy, 2026-11-08 ~17:45; models/from-scratch/bddstudy/summary.txt)
Setup: offline-c15, T-L-Ne-c's recipe at split threshold 100k and split limit 10, 60 rounds (3 passes), seeds 1 and 2.
The arms were the BDT, the BDD, a BDT with split limit 4 (the BDD's 9 leaves; the control), and bddbal (one seed,
exploratory). The new bdd-stats command (chessai-bddstudy) gives each delegator's routing regret against its base
children, R''s shares, and sibling agreement, on RR 84.
- Structure: the BDD spends 6 of its 10 splits on routers by round 15, with the root splitting 3-4 times, and ends with
  9 leaves under 22 delegators (depth 8). The BDT ends with 21 leaves at depth 4. The router-split trigger compares
  noisy windows of an error that is above a uniform guess (CNBP routing errors of 1.29-2.07 nats against ln 3 = 1.10).
- Regret: the first split, in round 4, lowers the root's regret one round later against the exact no-split twin:
  -0.0084 [-0.0093, -0.0075] and -0.0068 [-0.0087, -0.0049]. The tree's loss doesn't move: -0.0006 [-0.0015, +0.0004]
  and +0.0003 [-0.0008, +0.0015]. The children got more alike instead.
- Divergence: siblings reach the same leaf on 94-98% of positions one round after a split, a median 87.5% at round 15
  and 80% at round 60. Copies keep the counts they were made with and train only on what R' routes to them, and a few
  stacks stay 96-99% alike for 46 rounds.
- Use: R''s routing against always using its most-used sub-router, summed over the R' nodes per test position, at
  round 60: -0.0028 [-0.0052, -0.0005] (s1), +0.0056 [+0.0023, +0.0089] (s2), -0.0014 [-0.0039, +0.0010] (bal). There
  is no consistent gain.
- Held-out, mean of the two seeds, round 60: BDD minus the 9-leaf BDT +0.0010 [-0.0002, +0.0021]; BDD minus the BDT
  +0.0000 [-0.0023, +0.0024]; 9 against 21 leaves -0.0010 [-0.0032, +0.0012] (-0.0097 at round 40, while the BDT's
  late leaves were fresh). The BDD ties the BDT because it has fewer leaves, not because of its routers.
- League: D-L-Ne-c is kept as it is. At v61 it has 3 leaves and no router split, since its 5M windows are slow, so it
  plays as T-L-Ne-c's twin.
- Idea, the next one-factor test: train R' on its sub-routers' prediction errors (the loss of the leaf each would route
  to, as BDT routers do) instead of their routing errors (their log-loss at naming the best child, which is mostly
  CNBP overconfidence). In bdd.rs train_node, that means calling prediction_errors where shared children now take
  routing_errors. A CNBP router with forget 1 also gives copies its whole past, so they diverge slowly. A sub-router
  with bounded memory, or one trained fresh after the split, would diverge faster.
- Exploratory (one seed, a second factor): bddbal minus bdd at round 60 is -0.0010 [-0.0022, +0.0002] in log-loss and
  -0.0014 [-0.0021, -0.0006] in winner log-loss. The balanced target also changes the leaf-split routers.

### Router splits on regret, and R' trained on its sub-routers' leaves (bddregret, 2026-11-08 ~18:15; models/from-scratch/bddregret/summary.txt)
The user's DAG, as two one-factor options. Binary chessai-bddregret (md5 2b76d0e2); without the options it is
byte-identical to chessai-cyc5.
- A, `--delegator-split-on regret`: a delegator's window statistic is its routing regret, the routed child's loss
  minus the best child's. It splits by the leaves' rule (Worse, z = 0 here). Under Worse the routers-mean condition is
  dropped; under StallHigh it is kept.
- B, `--subrouter-target leaf`: R' learns from the loss of the leaf each sub-router reaches, with the Best target and
  ties going to the first child.
Setup: bddstudy's (offline-c15, 60 rounds, T-L-Ne-c's recipe, threshold 100k, split limit 10, seeds 1 and 2).
Router splits use up the split limit, so the arms ended with 7 leaves (9 for B seed 2). The controls are BDTs with the
same leaves (split limit 3, new; or 4).
- Verdict: router splits still don't clearly help, and nothing was staged.
  - Arm minus the same-leaves BDT on held-out RR 84 at round 60, mean of two seeds:
    - A -0.0012 [-0.0027, +0.0003]
    - B -0.0008 [-0.0023, +0.0006]
    - A+B -0.0008 [-0.0017, +0.0001]
    - the old BDD +0.0010 [-0.0002, +0.0021]
  - All three beat the old BDD by 0.0022-0.0027. Against the same-leaves controls they move the effect by about -0.002
    at round 60, but by +0.0013 to +0.0021 at round 20.
- A misfires on fresh R' nodes. Copies reach the same leaf, so an R''s regret starts near 0 and can only rise as they
  diverge. Half of A's router splits (14 of 28) were R' re-splits, at windows of 0.000-0.017. Routers over leaves
  split at 0.10-0.12.
- B starves the copies. Their leaf losses tie wherever they agree (97%), so the Best target sends ties to sub-router 1:
  95% / 83% right after the root's split. At round 60, 4 of 7 and 2 of 8 R' nodes still send at least 96% to one
  sub-router, and R' is level with its most-used sub-router (+0.0000, -0.0001). A router split becomes a near no-op.
- Ideas, not run, about 8 CPU-minutes each:
  - A with the regret measured against an R''s base children, so a new R' starts where the old router was. Or a skill
    score, for example regret relative to always taking the most-used child.
  - B skipping R''s update when every sub-router reaches the same leaf, or breaking ties by routing error. Ties then
    stop starving the copies, as LR leaf losses never tie after a leaf split.
  - The DAG's edge over the same-leaves BDT drifted from +0.0005 to -0.0012 between rounds 20 and 60 (A), so a longer
    run might separate them.

### Three router-split fixes together (bddfix, 2026-11-08 ~18:40; models/from-scratch/bddfix/summary.txt)
The user's DAG with the three fixes bddregret pointed to, in one arm. Binary chessai-bddfix (md5 963e86ed). Without
the new options it retrains bddregret's A and A+B and bddstudy's BDD byte for byte.
- A', `--delegator-split-on leaf-regret`: regret against the leaves reachable below, so a new R' starts at its old
  router's regret.
- B', `--subrouter-target leaf-skip`: B, but R' skips positions where every sub-router reaches the same leaf.
- C, `--router-forget-factor 16`: router-split copies forget 16x; leaf-split copies keep --forget-factor.
- Verdict: no. Against the 7-leaf BDT on held-out RR 84, mean of two seeds:
  - round 20: +0.0031 [+0.0012, +0.0050]
  - round 40: +0.0017 [+0.0005, +0.0028]
  - round 60: +0.0005 [-0.0011, +0.0020]
  No drop arms, nothing staged.
- Each fix removed its defect:
  - no near-zero windows;
  - no starved copies;
  - the copies part at once: 80% agreement after the split's round, against 94-96%.
- But the forgetting copies route worse than the unsplit router would have: root regret +0.03 to +0.07 and tree loss
  +0.005 to +0.010 at rounds 4-8. The rising regret fired more splits, and the root split 4 times by round 10.
- Across bddcheck, bddstudy, bddregret and bddfix (five variants, two seeds each), router splits never beat a BDT
  with the same leaves. Why, on this data:
  - The trigger fires on a climb that happens anyway. After a leaf split the new leaves are copies, so the router's
    error and regret start low and climb as the leaves specialize. Worse with margin 0 calls that a stall: in all 10
    runs the root split in round 4, its first chance.
  - A split adds no routing skill. Copies that keep their counts stay alike, and copies that forget get worse. R'
    ends level with its most-used sub-router in every variant.
  - Leaves past 7 don't help here (BDT-3 ties BDT-10). So router splits cost nothing and gain nothing, and the DAG
    ties whichever BDT it is compared with.
- Not run: a shadow test for router splits. Keep the old router, and split only if R' and its copies beat it over a
  window, as --revise does for leaves. I expect it to suppress most router splits rather than help.
- (coordinator, ~19:00, cycle 38) In-league check of the balanced targets after the first split (3 leaves, so at most
  3 effective): T-L-Le-bal-c 1.0-2.7 per round, top leaf 48-100%. It goes fully to one leaf for about 3 rounds after
  each own step (v47-49, v61-63), then spreads again, consistent with pbar's ~1M-position EMA lag. T-L-Ne-bal-c is
  steady at 2.2-2.6. At the same stage the old trees looked similar (T-L-Le-c after its first split: 1.1-2.7, rising),
  and T-L-Le-c's lock-in only showed later, with 11 leaves. So it's too early to judge; re-check once the bal trees
  have 3 or more splits.
- (coordinator, ~20:00, cycle 39) The live D-L-Ne-c has made its first router split: the root split at v93 (between
  v92 and v93), so it is now R' over three copies of the old root, all routing to the same 3 leaves. 2 of its 5
  splits are used (1 leaf, 1 router). At v119 the copies' current error windows are 2.9, 3.1 and 6.2 nats (1.2-3.0M
  of 5M positions, none closed yet), far above ln 3 = 1.10: the same miscalibrated trigger bddstudy found. The ply-2
  entries rate v77 1497, v93 1622 (its champion), v106 1630, v119 1599. The jump to v93 can't be credited to the
  split: the lineage is young and still climbing, and there is no matched comparator (T-L-Ne-c is at v1681).
- (coordinator, ~21:05, cycle 41) Why T-L-Te-merge-c merges splits that play better. --revise 0 compares the branch
  with its shadow (the unsplit leaf, trained on every position through the node) every 5M positions, by winner
  log-loss (--trial-error's default) on the lineage's own self-play positions before training, and merges whenever
  the mean gain is below zero. Its log (cycles/runs/T-L-Te-merge-c.log) shows the last 9 merges at gains of -0.00006
  to -0.00033 nats per position, 1.3-6.7 SE below zero: on its own yardstick the shadow really was a little better.
  The league rates split-phase versions 38.6 +/- 12.3 higher. Likely reasons: average loss on its own positions is not
  playing strength; the shadow gets all the data while each leaf gets about a third (LR keeps gaining from data, per
  lrnb); and z = 0 has no margin. The current split has survived 5 windows at gains of 0.00001-0.00023. Tests in the
  league: T-L-Te-c (never merges) and T-L-Te-merge-z3-c (--revise 3, staged ~21:05; z = 3 would have kept 6 of the 9).
- (coordinator, ~21:05) User: don't remove lineages and don't delay additions; the league, not self-play tests,
  decides strength. Staged by a deploy agent: D-L-Ne-lreg-c (A'), D-L-Ne-skip-c (B'), D-L-Ne-rf16-c (C), resumed from
  D-L-Ne-c v92 where possible, and T-L-Te-merge-z3-c. The DAG and probit agents now send league lines as soon as their
  variants run.
- (coordinator, ~21:10) User decision: abandon merges for error routers ("Both the router and the leaves should be
  capable of drifting ... merging just undoes all that specialization and router training"). T-L-Te-merge-z3-c was
  dropped from the staging before deploy. Existing merge lineages stay. Error-tree and DAG work is non-merging from
  here on; cluster routers may still merge.
- (coordinator, ~21:15) User decision: abandon NB leaves as a research line (NB as the predictor). No new NB-leaf or
  NB-expert lineages; the LR-NB hybrid was dropped from the probit agent's work. Existing NB-leaf lineages stay as
  rating anchors. NB routers (Ne, Nc) and NB gates stay in use.
- (deploy fork, ~21:20, cycle 43) Staged five one-factor variants of D-L-Ne-c's router splits (cycles/champ/staged-c43/,
  armed for the cycle 43->44 boundary: STOP touched 21:15, watcher pid 692940), all from scratch with seed 21 because
  a tree's state records its options: D-L-Ne-lreg-c (leaf-regret trigger), D-L-Ne-skip-c (R' on leaf losses, skipping
  agreeing copies) and D-L-Ne-rf16-c (router-split copies /16) on chessai-bddfix; D-L-Ne-skill-c (skill trigger) and
  D-L-Ne-margin-c (router splits need a 2-SE rise) on chessai-bddtrig. Own steps on the same binaries. Identity:
  chessai-bddfix without the options = chessai-cyc5 / chessai-cyc5-labels on D-L-Ne-c's line (models/from-scratch/
  dagdeploy/identity.log); smokes show router splits, resume, own step, tournament and gate arenas for all five.
  T-L-Te-merge-z3-c was dropped before staging (the user abandoned merges for error routers).
- (deploy fork, ~21:22, correction to the entry above) staged-c43 was restaged on the coordinator's instructions:
  D-L-Ne-skill-c now runs on chessai-bddtrig2 (fixes a sentinel bug that read a negative previous window as "no
  window", so skill never split), D-L-Ne-margin-c was withdrawn (offline byte-identical to the plain BDD: every router
  split already cleared 2 SE), and T-L-Ne-new-c (T-L-Ne-c's exact line, from scratch) was added as a same-age BDT
  control for D-L-Ne-c and the four variants. Cost rows measured from scratch by the driver's measure step (3.45-3.59
  ms a game). The watcher was replaced (pid 696653) because a running watcher keeps its old lineage list.

### LR against probit, and an LR + probit hybrid (lrprobit, 2026-11-08 ~21:30; models/from-scratch/lrprobit/summary.txt)
The user: "a potential probit-LR MoE investigation the way we did for LR + NB". lrnb's study with the probit
(F-P-tau0-c's recipe) in NB's place, then the hybrid itself.
- **LR is better everywhere we looked, but the probit's deficit is the draw question.** At full data it is 0.140
  [0.125, 0.155] behind: draw-vs-decisive 0.136, who wins only 0.0045 [0.0007, 0.0083].
  - An ordinal probit has one latent and one draw margin, so a draw can only mean "balanced". Its calibrated draw
    slope is negative on these games (-0.35 to -0.40).
  - No crossover from 10k to 2.3M positions. No group where the probit wins: worse in 50 of 59, ties in 9.
- **League reading.** The log-odds play policies rank moves by ln P(win) - ln P(loss), so draws never reach moves.
  - The flat champions differ on who wins (the probit's 0.008 [0.004, 0.013] worse), and flat probit trails (1646
    vs 1790).
  - The depth-2 champions tie on it (-0.001 [-0.007, 0.006]), and depth-2 probit leads (1831 vs 1806).
  - Suggestive only: the champions differ in age and data.
  - Idea: for offline comparisons meant to predict play, the winner loss is the relevant part, not the total.
- **No gate helps.**
  - The best mixing gate scores -0.0011 [-0.0032, 0.0009].
  - Hard gates from a mixing gate's weight hand the probit 0.01-1.2% of positions and score as LR does.
  - A hard gate regressing which model does better hands it 17-19%, where it is worse, and loses 0.0063 [0.0019,
    0.0106].
  - The oracle's -0.108 is lrnb's noise bound (two LRs: 0.071).
- **The hybrid is built and staged anyway**, since the user wants lineages in the league without offline gating.
  - bin/chessai-hybrid (md5 89605f85): --model hybrid --hybrid-experts lr,probit [--hybrid-hard]. Both experts
    train as they would alone, and an LR gate learns their responsibilities.
  - Pinned to either expert, it self-plays byte for byte as that line's binary does. With the live flags, its
    tournament writes the same 3,586 files as chessai-sparsefit over every kind in the live field.
  - Offline it is LR: -0.0014 [-0.0034, 0.0005]. The hard version hands the probit 0.38% of positions.
  - MoE-LP-L-c and MoE-LP-L-hard-c went to the coordinator. The tournament binary must switch to chessai-hybrid.

### Router-split triggers, one factor each (bddtrig, 2026-11-08 ~21:35; models/from-scratch/bddtrig/summary.txt)
T-L-Ne-c's recipe (LR leaves, CNBP error routers, --error log, branch 3, forget 1, no --revise), bddstudy's setup.
- Verdict: no trigger beats the BDT with the same leaves. Held out on RR 84, round 60, mean of two seeds, log-loss:
  - C alone (--router-forget-factor 16): -0.0005 [-0.0021, 0.0010]; worse at round 20, 0.0017 [0.0005, 0.0028].
  - Skill trigger (routed child's loss minus the best single child's so far; chessai-bddtrig2, after a sentinel bug
    that stopped it splitting): -0.0011 [-0.0029, 0.0007].
  - 2-SE margin on router splits: byte-identical to the plain BDD; every router split already cleared 2 SE.
  Both real arms beat the plain BDD at round 60 (-0.0016 [-0.0028, -0.0005] and -0.0026 [-0.0041, -0.0011]), also
  after allowing for leaf count. So their splits cost less than the BDD's, but they still gain nothing.
- C alone caused bddfix's early damage: copies 71-78% in agreement after the split's round, tree loss 0.002-0.008
  above the no-split BDT at round 4. It fades by round 40.
- The skill trigger re-splits R' nodes on near-zero windows (their children are copies that route alike), bddregret's
  A defect again. Untried fix: skill with --router-split-z.
- Headroom: a ridge router fitted to each leaf's loss never beat the trees' own routing (0.0013-0.0078 worse fitted on
  the training rounds, 0.015-0.021 on half of RR 84). No linear routing gain is visible on these leaves.
- Live D-L-Ne-c: R' is a pass-through. At v140 its copies agree on 97% and R' minus its busiest copy is 0.0000
  [-0.0002, 0.0002].
- League: D-L-Ne-skill-c (chessai-bddtrig2) and D-L-Ne-rf16-c join at the cycle 43->44 boundary; the margin line was
  withdrawn. Across five studies and seven variants, router splits have never beaten a same-leaves BDT offline.
- (coordinator, ~22:00) Direct matches, probit vs LR champions (models/from-scratch/d2pair/, 4,000 games each, league
  policy log-odds-softmax T0.1, own calibrations; Elo probit minus LR):
    d2-trained  F-P-tau0-d2jit-c v5150 vs F-L-d2jit-c v4978: depth 0: 7 [4, 10]; depth 1: 46 [40, 53]; depth 2: 35 [25, 46]
    flat-trained F-P-tau0-c v3595 vs F-L-c v4100:            depth 0: 6 [3, 9];  depth 1: 31 [24, 39]; depth 2: -146 [-158, -135]
  The league agrees: depth-2 champions 28 and -142; depth-1 current entries about 47 and 40. Depth 0 is 92-93% draws.
  So the probit is the stronger player at depths 0-1 in both pairs despite its far worse offline log-loss. At depth 2
  the flat-trained probit collapses while the d2-trained one leads: a ply-2 problem that d2 self-play fixes, much worse
  for the probit than for LR. Offline prediction loss predicted none of this. About 4 CPU-min.
- (deploy fork, 2026-11-08 21:53) staged-c43 deployed at the 43->44 boundary with seven lineages from scratch:
  D-L-Ne-lreg-c, -skip-c, -rf16-c (chessai-bddfix), D-L-Ne-skill-c (chessai-bddtrig2), T-L-Ne-new-c (control), and the
  coordinator-confirmed hybrids MoE-LP-L-c / MoE-LP-L-hard-c (chessai-hybrid, 4.81 / 4.03 ms a game). The tournament
  binary is now chessai-hybrid (identity over every live kind). Loop pid 705046.

## 2026-11-08 ~22:20: MoE vs trees with leaf sharing; M-L4-L-hard-c (user: "sure, why not")
- How the experts differentiate. moe and moenb start the gate random (uniform in +-0.1, moe_seed 1). nbgate and nbmix send each of the first 10,000 examples to a random expert. After that, responsibilities (gate weight x the expert's probability of the result, normalised) feed back, so whoever explains a position learns it. Hybrid experts all train on every position, so the gate never differentiates them.
- MoE vs share tree, five differences:
  - Prediction: blended vs one routed leaf.
  - Training: by responsibility, which uses the actual result, vs the routed leaf at full weight and every other leaf at a fixed 0.1/0.3.
  - Effect: pushes experts apart vs pulls leaves together.
  - Shape: fixed K experts under one gate vs grown by 3-way splits.
  - Gate target: responsibilities, which equal an error router with --error-target soft. The share trees use cluster or best-child targets.
- Cycle 43 champions: share trees 1819 (Nc) and 1831 (Ne); every MoE at or below 1790.
- One-factor test of the blend: M-L4-L-hard-c = M-L4-L-c with prediction from the gate's top expert alone; training unchanged. Assigned to the rename fork, in the same new binary as the win gate.
- Unmeasured: how specialised the live MoEs are (expert agreement, gate weights). hybrid-src main.rs (~l.390, ~l.2662) already reports gate entropy and the share of positions whose top weight is >0.99.

## 2026-11-08 ~22:45: T-L-Ne-share-c's 123-Elo drop (v428 -> v433) was self-play drift; the own step undid it
- Direct d2 matches at league conditions: v433 -98 vs v428 (calibration ~14 of it). The slide is gradual over the 5 rounds: -9, -18, -57, -36, -84. No split, merge or reset in that time.
- Self-play's vs-previous check saw at most ±8 per round: its policy values draws, and ~92% of its games are draws.
- Cycle 43's own step beat v433 by 100 and was kept. v440 is level with v428 (-6 [-20, 9]).
- Every lineage's d2 entries swing 40-70 Elo between versions; this lineage's range is ~120. The champion rule keeps the peaks, so current-vs-champion gaps are partly selection.
- Open: is the drift smaller in d2-trained (d2jit) lineages? Would a per-version check at league policy help?
- Files: models/from-scratch/sharedrop/.
- Structure check: the fifth and final split came at v416 (split limit 5). The two best versions, v421 and v428, came right after it; v413 was 1721. The dip happened at a fixed structure.

## 2026-11-08 ~23:00: MoE expert usage (models/from-scratch/moespec/summary.txt)
- The LR-gated LR MoEs collapse onto 1-2 experts:
  - M-L4-L-c: 2 of 4 alive, one at ~90%.
  - M-L8-L-c: 3 of 8 alive, one at ~95%.
  - M-L4-L-d2jit-c: 2 near-identical experts blended 50/50.
- M-L4-N-c (NB gate, the best MoE at 1807) uses all 4 experts and routes almost hard: >0.99 on one expert in 68% of positions.
- M-N4-L-c's LR gate spreads its weight, while the results put 96% on one expert.
- Hence M-L4-L-hard-c will mostly mean "the one live expert". To keep experts alive, test one factor at a time: the gate's init scale, a responsibility floor, or a load-balancing term.

## 2026-11-08 ~23:15: why T-L-Te-merge-c is strong
- It was flat LR until v776: every split merged back, as the TAN study predicted. As a flat model its typical d2 level (v687-v766) was ~1759, the same as F-L-c (1762).
- First surviving split at v776 (root TAN router + 3 LR leaves): typical ~1787 over v784-v872. Second split at v881.
- Level with the other top trees: last-8 means T-L-Ne-c 1785, T-L-Nc-merge-c 1786, T-L-Ne-merge-c 1779, D-L-Nc-c 1795. Its champion v833 (1829) is a peak.
- T-L-Te-c (no merges) is on the same track at the same version count (~1620 near v140).
- So: no TAN-specific edge is visible. It gains what the other trees gain over flat LR once a split survives.

## 2026-11-08 ~23:20: T-L-LLs-merge-c router state
- Flat (one LR leaf) since v2857. About every 60-80 versions it splits: an LR cluster router plus an LR error router beside it (--route-switch 3).
- 6 of the last 7 splits merged after one window. Their shortfalls were 0.2-1.8 block SE (z=0, so coin flips).
- The v2573 split switched to error routing (+4.3 SE over cluster; the routers disagreed on ~half the positions), beat its shadow for 4 windows (up to 6.1 SE), then merged at v2694 on a -0.3 SE window.
- Play: over the last 22 rated d2 versions, split versions averaged 1758 and flat ones 1769. The long split played ~1748 vs ~1773 for the flat versions around it (about -25, ~1.6 SE). Champion v2767 (1829) is a split version and a peak.


## 2026-11-08 ~22:00-23:20: the M- names; M-LP-L-d2jit-c; the win gate and M-L4-L-hard-c (rename fork)
- Naming (the user, final): a mixture of experts is M-<each expert kind with its count, unhyphenated, a count of 1
  left out>-<gate L or N>-<qualifiers>-c; archetypes stay count-free (M-L-L, M-L-N, M-N-L, M-N-N, M-LP-L), so the
  groups are as before. Mapping: MoE-c -> M-L4-L-c, MoE-8-c -> M-L8-L-c, MoE-d2jit-c -> M-L4-L-d2jit-c (archetype
  MoE -> M-L-L); MoE-L-N-c -> M-L4-N-c (M-L-N); MoE-N-L-c -> M-N4-L-c (M-N-L); MoE-N-N-c -> M-N4-N-c (M-N-N);
  MoE-LP-L-c -> M-LP-L-c, MoE-LP-L-hard-c -> M-LP-L-hard-c (M-LP-L).
- How the rename was done (cycles/champ/staged-c45/rename_lineages.py, at the 45->46 boundary, rehearsed on a full
  sandbox copy first): whole-token renames (a name not preceded by a name character and followed by -v<digit>, /, .,
  |, a comma, white space, a quote, an apostrophe, a colon, ) or ]) in state.json (the seeding records, which cite the
  first run, kept), cycles.csv, and the league's games.csv, players.txt, retired.txt, head_to_head, elo_history,
  ratings and league.md; models.csv's archetype column per renamed lineage; the own-calibration records and games
  (KEY.json, KEY.arena, KEY/, paths inside); the run directories and their .log files. The first run shares the six
  older directories (its lineages had the same -c names), so cycles/runs/<old> is now a symlink to the new directory
  and the first run's records (cycles/, selfplay/league-cycles) still resolve. lineages.txt, config.json and costs.csv
  were staged with the new names. Cycle reports, logs and backups keep the old names. Checks: every players.txt spec
  resolves, no old name left, selftest and status, a refit of the league from the renamed games (ratings as before),
  the dashboard renders.
  Deployed 23:15:32-23:16:15: all of these ok, refit 13 s, ratings unchanged (largest change 0.00 Elo over 5,705
  players); loop pid 774717. Old names left on purpose: the per-lineage seeding records' first-run peak entries
  (MoE-c-v300-2, MoE-d2jit-c-v2156-2, which name league-cycles entries), the owncal .arena comment lines (the arena
  command's old .tmp-<name> paths) and the six cycles/runs/<old> symlinks. The dashboard server needs a restart to
  classify the M- archetypes (dashboard.py's FX_ARCH takes them).
- M-LP-L-d2jit-c (from cycle 45; deployed at 44->45, 22:35:55, loop pid 723814): MoE-LP-L-c's line plus --depth 2 on chessai-hybrid; smoke
  models/from-scratch/lrprobit/mlpl/d2smoke.out; 128.6 ms a game from scratch.
- chessai-hybrid3 (hybrid2-src, a copy of hybrid-src; md5 1bf4176e) adds two options, each kept in the model file only
  when set (so every other file is byte-identical):
  - --hybrid-gate-target win: the hybrid's gate learns from decisive results only, its responsibilities from each
    expert's P(winner | decisive) = P(result) / (P(white wins) + P(black wins)); draws teach the gate nothing (a label's
    decisive mass weighs its step). Experts and prediction unchanged. File format version 2 with a "win" line.
  - --moe-hard: a moe predicts with the gate's top expert alone (ties to the first); training unchanged (soft
    responsibilities). File: a "hard" line after the gate's L2.
  Identity: without the options it self-plays, resumes and trains as chessai-hybrid (M-LP-L-c's and M-LP-L-hard-c's
  lines) and as chessai-cyc5 / chessai-cyc5-labels (M-L4-L-c's line), byte for byte; as the tournament binary it
  writes chessai-hybrid's 13,857 files over every kind and ply of the field (lrprobit/wingate/tourney3.out). The old
  binaries refuse the new files, hence the tournament switch (approved by the coordinator). chessai-hybrid2
  (f3800825, the win option only) was an intermediate build; unused, kept.
- M-LP-L-win-c and M-L4-L-hard-c (from cycle 47, staged-c46, armed 23:16: watcher 775609 on loop pid
  774717, STOP set; the tournament binary switch approved by the coordinator): one factor each against M-LP-L-c and M-L4-L-c. Costs
  5.48 and 10.0 ms a game (the latter measured on soft reference models; the driver re-measures).
- Early gate weights (mix-stats on lr's held-out set at v4, after 4 rounds of 1000 games from scratch): the full gate
  gives LR 96.3% (largest weight over 0.99 on 37% of positions), the win gate LR 99.9% (on 99%). Early on LR's
  who-wins predictions beat the probit's too, so whether the win gate turns to the probit later, as the direct
  matches (probit stronger at plies 0-1) suggest it might, is what M-LP-L-win-c will show.

## 2026-11-08 ~23:30: D-L-Nc-c state (cell-assigned DAG: CNBP cluster router over 9 cells -> 3 shared LR leaves, re-point at 2 SE over 50k)
- Formed at round 29. Cells 3, 4, 5 and 7 carry nearly all positions (8-10M judged each). Cells 0 and 8 had one window each in ~2,000 rounds; cells 1, 2 and 6 were never checked.
- Cells 3 and 7 flip-flopped between leaves 1 and 3 ~12 times in rounds 1811-1938. Nothing has moved since 1938: every cell with traffic points to leaf 3.
- Leaves 1 and 2 keep only the empty cells 6 and 1, so they no longer learn. Since ~v1939 the model is effectively one LR (leaf 3), champion v2043 (1852) included.
- Typical d2 level ~1795 vs F-L-c 1762: a near-flat LR that is stronger than F-L-c, cause unknown. Same pattern as T-L-Te-merge-c and T-L-LLs-merge-c: the top "trees" spend much of their time as one LR.

## 2026-11-08 ~23:45: near-flat tree leaves vs F-L-c (models/from-scratch/leafcmp/summary.txt)
- The structure contributes nothing in play: D-L-Nc-c v2043 plays exactly as its extracted leaf 3 (+1 [-14, 15]).
- Direct d2 matches against F-L-c:
  - D-L-Nc-c leaf 3: v2043 +54 [39, 69] vs v4100 and +54 vs v4664; v2064 -25 [-40, -10] vs v4664 (a 79-Elo swing in 21 versions).
  - T-L-LLs-merge-c v2865 -12 [-27, 2]; T-L-Te-merge-c v766 -21 [-36, -6].
  - So the champions edge is mostly selection of peaks.
- AdaGrad: splits do not restart the sums (clones; forget factor 1). The leaves are younger: 0.19-0.69 of F-L-c squared-gradient mass, 1.2-2.3x larger steps.
- D-L-Nc-c improves +14.5 per 100 versions vs F-L-c +2.3.
- Next one-factor test: F-L-c with its AdaGrad sums divided (re-aged), continued from v4664.

## 2026-11-08 ~23:50: league-data confound for d2jit lineages (user: "the d2 models use more league data proportionally")
- Rough share of league positions in training (cycles 1-44: self-play games/cycle x plies, own positions x kept rate):
  F-L-c ~3%, F-P-tau0-c ~1.5%, M-L4-L-c ~14%; F-L-d2jit-c ~51%, F-P-tau0-d2jit-c ~46%, M-L4-L-d2jit-c ~85% (315 self-play games a cycle).
- Within-pair probit-vs-LR comparisons are matched on league share, so these hold: probit beats LR at d1 in both pairs, and at d2 among the d2-trained pair.
- Confounded: "depth-2 training rescues probit at depth 2". It may be the league games instead, which include the depth-2 entries' play. The same caveat applies to every d2jit-vs-flat comparison, M-LP-L-d2jit-c included.
- Separating test: a self-play-only twin of F-P-tau0-d2jit-c (regenerated d2 self-play, no league games), played at d2 against the real one.
- First run (cycles/cycles.csv, c39-c46): F-P-tau0-d2jit-c made ~5,300-6,000 self-play games a cycle against ~91-113k league positions, so league games were roughly 12% of its training. It still rose above F-P-tau0-c (1719 vs 1557 at c44). But first-run ratings carry the cal:heldout T0.1 bias that favoured cold-playing d2 models, so this does not settle it.
- Proposed clean test (not yet approved): a per-lineage switch that skips the own step, with two fresh self-play-only probit lineages that differ only in self-play depth (provisional names F-P-tau0-solo-c and F-P-tau0-d2jit-solo-c).
- Outcome (2026-11-09 00:02): staged-c46 deployed at 46->47; M-LP-L-win-c and M-L4-L-hard-c from cycle 47, the tournament on chessai-hybrid3; 70 active; loop pid 797544.

## 2026-11-09 ~00:05: staged-c47 (47->48): F-L-new-c and F-L-agdiv-c, both against F-L-c
- F-L-new-c: F-L-c's exact line from scratch (user: "Maybe we should launch a fresh F-L-c to see how well it does in the current field?").
  - Same binary, seed 21 and 1000-game rounds, so its self-play replays F-L-c's first versions bit for bit (lr1-lr3 checked).
  - It parts from F-L-c at its first own step, which trains on the current field's games.
- F-L-agdiv-c: F-L-c continued from its version at the boundary, with its AdaGrad sums divided by 2.7 once.
  - That is D-L-Nc-c leaf 3's level (leafcmp: 0.37 of F-L-c's sums), so its steps are 1.64 times F-L-c's.
  - Mechanism: an init: source, with no new option or binary.
    - At the boundary, staged-c47/deploy.py runs `chessai-cyc5 train --start <F-L-c lrN> --init-forget 2.7` on an empty round.
    - It then checks the result: weights byte-identical to F-L-c's (so the predictions are F-L-c's exactly), and every sum is F-L-c's / 2.7 exactly.
  - The flags deliberately carry no --init-forget. The own step's train gets every flag but SELFPLAY_ONLY, so the flag would divide the sums again every cycle.
  - A pre-divided file with --init plays bit for bit as --init <F-L-c> --init-forget 2.7 (lr1 and lr2 checked).
- Reading the results:
  - If F-L-agdiv-c rises above F-L-c within ~10 cycles, toward the near-flat trees' 1795-1850, then their edge is younger AdaGrad state plus peak selection, not structure.
  - If it stays level with F-L-c, look elsewhere: the trees' own-game mix or selection.
  - F-L-new-c separates out youth together with this field's league games.
- Files: models/from-scratch/agdiv/README.txt (smokes, costs) and F-L-agdiv-c-init.txt (the init's source version and md5s, written at the deploy).
- 2026-11-09 00:05 MECHANISM (league share grows with N^2): champ self-play is a fixed 3,600 CPU s a cycle split equally (53 s each at 68 lineages, 51 s at 70), while league games per lineage are ~60 per opponent lineage (own games 3,690 at 62 lineages, 4,070 at 68). So each added lineage both cuts every lineage's self-play and adds to its league intake: from 62 to 76 lineages (after the 47->48 and 48->49 additions) the league-to-self-play ratio rises ~1.5x. This hits d2 lineages hardest (already ~50% league).

## 2026-11-09 00:15: twin pilot result (models/from-scratch/twin/summary.txt)
- T-L-Ne-share-c against a flat LR "twin" trained on exactly its self-play games (all 416 rounds replayed exactly; the twin lacks the 9.9% of positions that came from own steps), at d2, 2000 games each, own cal:
  tree minus twin: v122 191, v217 236, v428 292, v452 294 (95% ranges about 20 wide). Twin v452 vs F-L-c v452: -19, so the twin is an ordinary F-L-c of the same age. Tree v452 vs F-L-c v452: 247; vs F-L-c v4100: 4.
- So the tree's games are not better data; the edge is in the model. Not separated: routing/specialisation vs the leaves' younger AdaGrad sums (11 leaves at v452, so each leaf's sums are a fraction of the twin's and its steps bigger).
- Proposed next (awaiting the user): on the same games, flat twins with bigger steps (rate x2 and x3.3), plus the tree against its busiest leaf alone; about 1 CPU hour. Scaling the twin test to T-L-Ne-c, T-L-Nc-c, T-L-Nc-share-c and D-L-Nc-c would be about 9 CPU hours.
- CORRECTION (user, 00:25): the twin lacks the tree's league data, and the "10% can't explain 290" argument assumed league positions are worth what self-play positions are. By v452 the tree had 36 kept own steps, F-L-c v452 had 1 (first run, cycle 1), and the twin 0. The tree's kept own steps gained a median 74 Elo over the pre-step model in the 100-game d2 gate (quartiles 38-98, the first three 135-160; 37 of 45 kept), so league data may be a large share of the 290.
  Clean redo proposed: rerun the tree from scratch self-play only (same seeds, no own steps) with flat twins on exactly its games (rate x1, x2, x3.3). About 1-1.5 CPU hours.

## 2026-11-09 ~00:25: M-L4-N-c is not one LR (models/from-scratch/l4ncmp/summary.txt; user: "to what extent is it just a single LR node that's doing all the work?")
- v338 full vs its busiest expert (1, 39% of gate mass): 73 [64, 83] at d1, 46 [32, 61] at d2; vs expert 3: 126 at d1, 92 at d2.
- Each expert alone vs F-L-c v4696 at d1: -96 and -150. Expert 1 at d2: 19 [5, 34]. Full v338 vs F-L-c v4696: -25 at d1, 76 at d2 (the league agrees: -29 and 60), so it leads at depth 2 only.
- Offline: hard routing (the top expert alone) is only 0.001 worse than the soft mixture; the best single expert is 0.019-0.024 worse. Win-loss vectors correlate 0.89-0.96 between experts, draw vectors 0.66-0.84.
- With a calibration refitted per region, expert 1 is best or level in most regions, so part of the specialisation is per-region scale and offsets rather than ranking.
- Youth does not explain it: the experts' AdaGrad sums are 2-4% of F-L-c's (steps 6-9x larger), yet alone they lose to F-L-c at d1. Contrast D-L-Nc-c, whose full model played as its leaf 3.

## 2026-11-09 ~00:30: four solo lineages staged for 48->49 (staged-c48; the user: "Ok, let's do that", then the LR pair too, all four at one boundary)
- Why: the league-data confound (2026-11-08 ~23:50 above). A d2jit lineage trains about half on league games, a depth-0 one about 1.5-3%. So "depth-2 training helps" may be league data.
- The switch: steps - in lineages.txt (the registry already had it; the driver already skipped such a lineage's step). A solo lineage takes no own step and no gate, so it never trains on league games. Its entries play and are rated, and the others' own steps train on games against them.
  - cycle.py: own_steps marks a solo lineage's row with gate "solo: no own step" (own_*/gate_* empty), so cycles.csv and cycle-NNN.md say so. The selftest checks it.
  - dashboard.py: such a gate shows as a muted "solo" cell, not a failed step, and stays out of the Now panel's own-steps table.
- The 2x2, each its original's exact line from scratch with steps -: F-L-solo-c (F-L-c's line, = F-L-new-c's), F-L-d2jit-solo-c, F-P-tau0-solo-c, F-P-tau0-d2jit-solo-c.
  - NOTE: the "flat" lines self-play at depth 0 (no --depth flag), not depth 1. The league plays all four at plies 1 and 2.
- Identity: a fresh solo lineage replays its original's first versions bit for bit (sandbox: F-L v1, F-L-d2jit v1-v3, F-P-tau0 v1-v2, F-P-tau0-d2jit v1-v5). It parts from the original at the original's first own step (first run: F-L-c v424, F-L-d2jit-c v473, F-P-tau0-c v156, F-P-tau0-d2jit-c v150). F-L-solo-c also equals F-L-new-c up to F-L-new-c's first own step (end of cycle 48).
- Reading it:
  - F-P-tau0-d2jit-solo-c vs F-P-tau0-solo-c at ply 2 (and the LR pair): depth-2 self-play with no league data on either side. If the d2 one still wins at ply 2, depth-2 training helps by itself; if not, the d2jit lineages' edge was the league's games.
  - Each solo lineage vs its original (match by versions or CPU; the originals are much older): what the own step adds.
  - F-L-solo-c vs F-L-new-c (same age, same start): the own step's effect on a fresh F-L in today's field.
- Checks: models/from-scratch/solo/README.txt (harness.out: the staged and live drivers' own phases are byte-identical for lineages with steps own; rehearse.out: the deploy's checks all pass). Cost: the self-play budget is fixed, so every lineage's share drops from 50.0 to 47.4 CPU s. The tournament gets ~7.5% longer (8 more entries). Not running their steps saves ~20-30 CPU s a cycle.
- Deployed 2026-11-09 00:51:28 at the 47->48 boundary (loop pid 828580). F-L-agdiv-c's init is F-L-c v4725 (md5 64c78685...), init md5 e29c26f8...; checked again independently: 2556 weights identical, 2556 sums exactly F-L-c's / 2.7.

## 2026-11-09 ~01:30: full-data twin of T-L-Ne-share-c (Q3, Q6): level with the tree at v122 (models/from-scratch/twin/league/summary.txt)
- A flat LR trained on exactly the tree's data in its order: the 116 self-play rounds plus its 6 kept own steps (cycles 2-7, 21,740 league games, 1.86M positions = 5.7% of the positions), every step regenerated exactly (replays equal to games.csv; the tree's own steps retrained byte for byte). Three rates; d2, own cal, 2000 games, seed 11:
  tree vs twin r0.05 -5 [-19, 10]; vs r0.10 41 [26, 56]; vs r0.165 99 [84, 114]. Twin r0.05 vs the self-play-only pilot twin: 221 [204, 239] (pilot: tree vs that twin 191).
  Eval-loss (heldout-d2play): tree 0.9256, twin r0.05 0.9263, r0.10 0.9352, r0.165 0.9478, self-play-only twin 0.9475.
- So at v122 (5 leaves) the tree's lead over a flat LR is all data, and league positions are worth far more than self-play positions (5.7% of the positions, 221 Elo). Bigger LR steps are monotonically worse.
- The agreed rule (tree >= 30 over the best twin) stopped the pass at v122. Open: v217/v452, where the pilot gap grew to 294; resumable (about 1-2 CPU h to v217, 6-8 to v452).
- The stream is kept for more learners on identical data: models/from-scratch/twin/league/stream/ (steps 0-121 as train source dirs plus seeds; README, steps.tsv).

## 2026-11-09 ~02:30: why the routed models win, and routing headroom (Q6, Q4, Q5, part of Q1) (models/from-scratch/whytrees/summary.txt)
- Q6, play at d2 (2000 games, own cal, the league's way; full tree minus one of its leaves alone): T-L-Nc-share-c v545 vs its best and busiest leaf 40 [26, 55]; T-L-Ne-c v1878 vs its busiest leaf -12 [-26, 2]; T-L-Ne-share-c v428 vs its busiest leaf -41 [-56, -27] (the leaf is stronger); T-L-Te-merge-c v833 vs its busiest leaf 10 [-4, 24].
  - So at d2 only Nc-share's routing helps; the other three champions play as one leaf. At d1 every tree beats its busiest leaf (26-69).
  - Those leaves are young (AdaGrad sums 5-20% of F-L-c's, median steps 2.3-4.9x). Youth vs filtered data is open: the twin stream's leaf 2x2 tests it.
  - The offline-best leaf is a poor guide: T-L-Ne-c v1878's L4 is 171 below the tree at d2 (L7 is level); Te's L1 is 85 below.
- Q4: NB (CNBP) routers and gates lead (T-L-Nc-share-c 1852, T-L-Ne-c 1844, M-L4-N-c 1822); LR routers trail; TAN sits between, on two lineages.
- Q5, headroom: learned flat routers (LR or CNBP, responsibility or hard targets, with or without the own pick) never beat own routing (Ne 0.017-0.038 worse; the share trees 0.002-0.009 worse; Te level). Re-pointing regions to their best leaf gains at most 0.005 (Ne-share, d2play only). No case for router re-splits from headroom.
- Per-region calibration gains 0.025-0.041 offline (d2play) but loses in play (leafcal vs cal): -19 to -93 at d1, -101 and -147 at d2.
  - Mechanism untested: per-leaf scales and offsets, or small own-play fit sets.
  - Offline log-loss did not predict play anywhere in this series.
- Q1: M-LP-L-c v62's gate gives its LR expert ~100%, so the mix is its LR. The starved probit expert beats it at d2 (44) and loses at d1 (43). A learned gate between the two does no better offline. A gate trained on likelihood will keep picking the LR, which wins on draws; a mix that helps at d2 needs another signal.

## 2026-11-09 ~02:45: learners on the kept twin stream at v122 (models/from-scratch/twin/league/learn/; summary.txt there)
Each trains on the same stream as twin r0.05 (T-L-Ne-share-c's self-play plus its 6 own steps before v122, in order). d2, 2000 games, own cal, seed 11.
- Q1 (LR vs probit at equal data): a probit twin (F-P-tau0-c's recipe) vs twin r0.05: -10 [-24, 5] at d2, -30 [-39, -21] at d1. So at equal data probit is no better at d1: the league's d1 probit edge (F-P-tau0-c over F-L-c at ply 1) is not the model on equal data. It may be its own data, age, or league share. Offline as before: log-loss 0.979 vs 0.926 on d2play (draws), winner log-loss 0.5695 vs 0.5717.
- Q2 (step size): LR rates 0.10 and 0.165 are monotonically worse than 0.05 at v122 and v217, in play and offline. Bigger LR steps do not help on this stream. No probit-side test yet (tau 0 only).
- Q3 (which league games carry the value; LR with the self-play rounds plus only part of each own step; Elo over the self-play-only twin, per 100k subset positions):
  all 221 (11.9); the tree's d2 entry's games 190 (20.4); its d1 entry's 98 (10.6); games vs opponents rated above the tree 128 (17.7); vs below 106 (9.4).
  The ply-2 entry's games are half the positions and give 86% of the gain. Per position, d2-entry games and games against stronger opponents are worth about twice the others. Halves don't add up (98 + 190 > 221): diminishing returns.
- Ideas: weight league positions by entry depth (or keep only the ply-2 entry's games) in the own step, as a one-factor lineage. Similarly, oversample games against stronger opponents. Both are cheap to stage (the own step already chooses its games).

## 2026-11-09 ~02:45: Q6 leaf-data 2x2 for T-L-Ne-share-c's L9 (planned and running; models/from-scratch/twin/league/q6.py)
- Question (the user): is a young leaf like L9 (alone it beats the full v428 tree by 41 at d2; AdaGrad sums 5% of F-L-c's) strong because it gets filtered data, or because its step stays large?
- 2x2 at v428:
  - L9 itself;
  - B: L9's exact data (weight 1 when routed to L9's ancestor, 0.1 share otherwise), with sums reset to twin r0.05's each step;
  - C: all data, with sums reset to L9's ancestor's each step;
  - twin r0.05.
  Data effect: L9 vs C and B vs twin. Step effect: L9 vs B and C vs twin.
- Method: retrain the tree step by step with a traced copy of chessai-lrshare, which trains B and C beside it; the tree is checked byte for byte at every step. About 45-50 CPU min.

## 2026-11-09 ~02:55: Q9 compute profile: movegen share and position/eval caches (models/from-scratch/movegen/summary.txt)
- Cycle 49's CPU: tournament 70% (11,566 of 16,552 CPU s; 263k games), self-play 22% (fixed budget), gates, own-cal
  and own steps 8%. About 17% of the league's CPU is movegen: 19% of the live tournament (perf), 33% of a d2 flat-LR
  game, 15% of a d2 tree/MoE game, 24% at d1, 8% in d0 F-L-c self-play. At d2, 96% of movegen calls are at leaves,
  building the whole legal-move list only to test for mate/stalemate. The model's sums are the largest part
  (tournament: model 50%, encode 11%, exp/ln and calibration 11%).
- Caches don't pay at d<=2. Repeated evals: 0-0.02% within a search (2 plies hardly transpose), 0-0.4% with the last
  two searches, 0.6-2.4% within a game, 1.9-9% over a run (openings). Probes cost 3-19 ns against evals of
  0.2-1.2 us, so the net is -1% to 2%. Within d3 and d4 searches 14% and 31% of evals repeat. Engines: Stockfish's
  transposition table stores search results (static eval on the side), Lc0 caches GPU evals, nobody caches move lists.
- Exact speedups, byte-identical games (checked; 12.7M leaves asserted): a cheap "any legal move?" leaf test, lazy
  candidates when the mover searches, bitboard encode. Together: -44% CPU at d1, -46% at d2 flat, -21% at d2
  tree/MoE, -17% d0 self-play (encode alone). Tournament estimate: -25% (20-30%), ~9 min less per cycle.
- Next lever, not exact: NNUE-style incremental sums for the additive parts (~40% of the tournament); fixed point
  would make it exact from then on.
- Proposal: a tournament binary with the three fast paths always on, checked identical on a field sample, switched at
  a boundary. Self-play speedups would change games per cycle, unevenly by model, so they're a separate decision.
- v217 (~03:00, 13 own steps): probit vs twin r0.05 -50 at d2 and -62 at d1 (v122: -10, -30). As data grows, probit at tau 0 falls further behind LR at equal data (Q1). That is consistent with its step shrinking like 1/n (Q2), but not shown; a tau 1e-6 probit twin on the same stream would test it. Q3 subsets over the self-play-only twin (per 100k): d2 entry 202 (10.1), d1 entry 78 (3.9), stronger opponents 133 (11.2), weaker 166 (5.9), all 212 (5.3). The ply-2 entry's games alone match the full league data.
- 2026-11-09 01:47 DEPLOYED (staged-c48, loop pid 863090). Cycle 49's rows say gate "solo: no own step".
  - F-L-solo-c = F-L-new-c byte for byte for v1-v21; they differ from v22 (F-L-new-c's first own step).
  - First ratings (v17/v18/v25/v26 after one cycle): F-L-solo-c 1057, F-L-d2jit-solo-c 594, F-P-tau0-solo-c 1143, F-P-tau0-d2jit-solo-c 719. Too young to read.

## 2026-11-09 ~04:30: Q6 leaf-data 2x2 for T-L-Ne-share-c's L9, result; Q2 probit-tau twin (models/from-scratch/twin/league/summary.txt)
- Q6 at v428 (Elo vs twin r0.05, d2, 2000 games, own cal): L9 (its routed data, young step) 16 [1, 30]; B (L9's data, the twin's sums) -11 [-26, 4]; C (all data, L9's sums) -45 [-60, -31]; twin 0. Direct: L9 vs B 39, L9 vs C 65, B vs C 40, L9 vs tree 39.
- Q6 reading: L9's edge needs both its filtered data and its larger step (sums ~35% of the same-age twin's, a step ~1.7x). The larger step helps only on L9's data and costs 45 on all data. L9's data helps only at the larger step. Interaction about 80 Elo. Offline loss gives the filtered data a small edge at both steps and misses the interaction.
- Q6: tree v428 vs twin v428 0 [-15, 14] (v452: -57). L9's data: 25.8% of the stream at weight 1, the rest at 0.1.
- Q6 follow-up running (q6r.py): the same 1 / 0.1 weighting on a random 25.8% of positions (L9's count per step), at the young step (R-young, vs L9) and the old step (R-old, vs B). It separates the router's region from "any partial weighting plus a large step".
- Q2: a probit twin with F-P-c's exact recipe (default tau 1e-5; header = F-P-c's) on the same stream: vs twin r0.05 at d2 -69 (v122) and -191 (v217); vs the tau-0 probit twin -37 and -129; at d1 vs twin -14 and -56 (tau 0: -30, -62). tau does not rescue the probit at equal data. It makes the d2 decline much worse, so stiffening is not why probit falls behind LR. Untested guess: tau's variance floor forgets, losing the league data's value in the self-play rounds between own steps.
- Q6 random control (~04:51, q6r.py): L9's 1 / 0.1 weighting on random positions (L9's count per step). Joint fit of all 14 matches (twin = 0): L9 26, R-old (random, twin's sums) 19, R-young (random, own sums) 4, tree -7, B -13, C -46. The router's region beats random positions by 23 ± 10 at the young step but loses to them by 33 ± 10 with the twin's sums. Arms whose sums were built on their own data are all at or above the twin. The two whose sums come from other data, B and C, are the worst (C inherits L9's per-feature sum profile, p10-p90 21-51% of the twin's, vs R-young's uniform 25-34%). So Q6's "young step" mixed step scale with sum profile. A random weighting with the twin's sums comes within 7 ± 9 of L9: average weight 0.33, a smaller effective step.
- Follow-ups (not run): (a) flat twins at rate 0.025 and 0.0166 on the stream. Is R-old's 19 just a smaller step, and does the rate ladder keep improving below 0.05? About 15 CPU min. (b) B' = L9's data with L9's per-feature sum profile at the twin's scale. Profile vs scale on routed data. A traced tree replay plus matches, about 20 CPU min.
- v452 learners (~05:17; single versions, ~17 Elo version noise each): Q1 the tau-0 probit vs the LR twin at equal data, d2 -10 / -50 / -101 at v122 / v217 / v452, d1 -30 / -62 / -57. Q2 probit-tau (F-P-c recipe) vs tau 0 at d2 -37 / -129 / -117, about level at d1 from v217. Q3 by v452 the ply-2 entry's league games alone carry all the league value (307 vs 302 over the self-play-only twin), ply-1's 85. Per position, games vs stronger opponents are worth 2.7x those vs weaker, d2-entry 3.6x d1-entry.
- POOLED RECHECKS (~05:43; supersede the single-version Q6/R readings above): both sides at the same version, 1000 games a version. Headline tree vs twin r0.05 over v444-v452: mean -26 [-42, -10] (v452 alone said -57; version SD 18). Q6/R arms over v424-v428: the flat arms on the shared stream swing together (no version noise in their contrasts), L9 and B swing on their own. Stands: C vs twin -46 [-54, -38] (a ~1.7x step, as the rate ladder predicts); a random 1/0.1 weighting beats the twin by 15 [7, 23] at either step (R-young = R-old, 0 ± 8). Falls: the router's region (L9 - R-young 5 [-17, 26], B - R-old -9 [-44, 27]), Q6's ~80 interaction, the R control's "router worth 23", the mismatched-sums reading. L9's edge over the twin (18 [-6, 43]) is about what a random weighting gives. Lesson: single-version contrasts of models on different data need pooling over adjacent versions; same-stream flat models can be compared at one version.
- RATE LADDER BELOW 0.05 (~05:58, twin/league/rates.py): flat twins at 0.025 and 0.0166 on the same stream, vs r0.05 at the same version. v122 -25 / -69; v217 -2 / -26; pooled v424-v432 31 [20, 43] / 15 [0, 30]; single v452 -18 / -37 (r0.05's v452 looks lucky, the extreme of the headline recheck too). The best AdaGrad rate falls as data grows. A plain smaller rate explains R-old's 15, so Q6 (b) is moot. Offline is slightly worse at the smaller rates while play is better. Idea for the user (not deployed): the mirror of F-L-agdiv-c, F-L-c with AdaGrad sums x4 (= rate 0.025 from then on), one factor against F-L-c. Open: a pooled v444-v452 point for the rates (~20 CPU min).
- RATE LADDER SECOND WINDOW (~06:05, ratepool.py): v444-v452, vs r0.05 at the same version: 0.025 2 [-11, 14], 0.0166 -5 [-28, 18], against 31 and 15 in v424-v432. The lead does not replicate. The windows differ by ~30, so strength drifts over tens of versions too, and one 9-version window is not a replicate. WITHDRAWN: "best rate falls with data" and the F-L-c sums x4 idea; 0.05 looks near a flat optimum (with agdiv's no-gain). R-old's +15 (v424-v428) and the headline's 26 (v444-v452) are single windows too.
- 2026-11-09 ~10:45 LEAGUE DRIFT AND GATES (the user asked about the league's overall Elo drift; to be discussed further later). From selfplay/league-champ/elo_history.csv and the cycle tables:
  (1) Scale: 102 ply-2 entries unchanged since cycle 1 moved 30 from cycle 5 to 60 and 6 from cycle 30 to 60 (SD of the change 0.3); the ~140 drop happened inside cycle 1 while the first fit settled. Ratings are comparable across cycles.
  (2) Progress: top 10 ply-2 entries minus that panel: 82 (c1), 115 (c15), 158 (c35), 179 (c60). Lineages active c40-60 gained a median 44 (d0/d1), 32 (d2), ~10 (d4). Current versions sit ~30 below their champions every cycle (version noise plus selection).
  (3) Gates (post vs pre, 100 games d2, keep at >= 50%): 3,807 gates at 54.4% (~30 Elo); c40-60 all-gate means 18 (d0/d1), 33 (d2), 1 (d4) a cycle, i.e. 360-650 expected over 20 cycles against 30-45 seen in the league. d2/d4 lineages (almost no self-play between steps) lose it too, so not self-play erosion. Likely (untested): the step learns from its predecessor's own league games and beats that predecessor specifically. Plus lucky keeps (d4 gates average ~0, yet 9 and 11 kept).
  Proposed, not run: gate-panel test (~20 recent kept steps, post and pre against a fixed field panel, ~10 CPU min); if confirmed, gate against the champion plus field entries. Retired-entry replay (~5 CPU min) to re-measure the idle overrating (first run: 25-40).
- 2026-11-09 ~11:10 THE USER'S HYPOTHESIS (error routers vs the field): head-to-head tests (tree vs its best leaf, tree vs twin) may understate routing. Against its own best leaf, the positions look like that leaf's training positions, so the error router sends most of them to it and the rest to weaker leaves (more likely to hurt). Against a field that plays less expected moves, routing to other leaves may help. Supporting: head-to-head misleads elsewhere (own-step gates claim 10-20x the league's gains). Against: at d1 every tree beat its busiest leaf head-to-head (26-69); offline, on other games' positions the routed leaf was its region's best predictor in only 0-3 of 9-11 regions (but offline doesn't predict play). Proposed test (not run): tree, best leaf and (Ne-share) the twin, each against the same ~12-entry field panel at d2, paired seeds, 3 versions each of T-L-Ne-c (most distinct leaves) and T-L-Ne-share-c, ~30 CPU min; plus a routing-share check on the tree's league games vs its games against its leaf. If the tree beats its leaf against the panel while losing head-to-head, findings 1, 4 and 5 need redoing against the field.
- 2026-11-09 ~11:15 FIELD-PANEL TEST of the user's hypothesis (models/from-scratch/fieldpanel/summary.txt; ~31 CPU min): each candidate vs the same 12 ply-2 entries of the cycle-60 field, d2, 200 games each, own cal, hybrid4, paired seeds; 3 versions per tree. Against the field: T-L-Ne-c tree - L7 (busiest leaf) -3 [-15, 8] (head-to-head -12 / -27); T-L-Ne-share-c tree - L9 -32 [-44, -20] (h2h -41 / -36); tree - flat twin -23 [-35, -12] (h2h 0 at v428); L9 - twin 9 [-3, 20]. Against the weaker half of the panel (1439-1767) the trees do no better vs their leaves (-28, -31). Mechanism: the tree routes the same share of positions to its busiest leaf against the field as against that leaf (Ne-c 0.634 vs 0.632; Ne-share 0.281 vs 0.253). Hypothesis not supported for these two error trees; findings 1, 4 and 5 hold against the field (finding 1 more clearly). Not tested: the cluster-routed models with head-to-head gains (T-L-Nc-share-c, M-L4-N-c).
- 2026-11-09 ~11:45 THE USER: RETHINK THE TOURNAMENT. (1) We spend a lot of compute on (ostensibly) high-quality tournament games and don't use them fully (each lineage trains on ~4,300 of ~250k games a cycle; 12.2M games saved). (2) Separate questions: A. which model gets strongest by self-play; B. which gets strongest given the same dataset; C. is (ostensibly) higher-quality data (deeper search, the d2jit lineages) worth it, and from what point; D. which model generates the best training data (with interactions: some learners may train better on some generators' games). (3) Patience ignores the training lineages get from tournament games, which made the league move much slower than expected. Numbers: patience 1,920 CPU s of self-play at ~50 s a cycle = ~38 cycles; first finishes at cycle 35; 18 finished in 60 cycles; it lengthens as N grows (3,600/N each); each lineage's tournament games cost ~105 CPU s a cycle (cycle 51: 30 ms a game). Charging self-play plus the tournament share: ~12 cycles; 23 of 71 active are past 12 cycles without a champion. My suggestion (under discussion): league answers A and rates everything; B and D offline on the saved games against a fixed panel; C as solo depth pairs.
