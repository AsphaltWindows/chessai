# Ideas

Open ideas only, by area, then what was tried and rejected (one line each). Updated 2026-11-10.

- Results and their numbers: RESULTS.md. Decisions waiting for the user: NEXT-EXPERIMENTS.md.
- The full history of ideas from 11-06 to 11-09, with every study's notes, is archived:
  archive/2026-11-10/models/IDEAS.md. Idea numbers below (idea 16 and so on) are that file's.
- Ground rules still apply (NEW-DESIGN.md, "Rules in force"): self-play only, zero knowledge, no hand features, search
  as a last resort, one factor per experiment.

## Open ideas

### Routing and growth (compound models)

- **Parked for the goal, 11-11 ~02:15 (shallow-search routing only).** Fresh-gate limit-4 probit trees gain ~95 at
  ply 1 near argmax, replicated in two seeds (p1b, p1c). LR leaves under the same gate gain 43 at ply 1 and lose 43
  at ply 2 (lr-trees-p1). At ply 2 the fresh gate rejected the splits (probit-trees-p2: one leaf at v276, never more
  than 3), and the trees were level with flat at plies 1-3. RESULTS section 8; theory-2 6.13-6.15. Follow-ups, only
  if shallow search becomes a target:
  - Split noise 0.01 and 0.05 beside 0.03 at ply 1. The analyst's mechanism: the noise is the prior's width around
    the parent leaf; 0.03 tethers rare features while common ones move.
  - An LR counterpart: copy-on-split with an L2 pull to the parent's weights frozen at the split (width-equivalent
    0.03). Needs a new training flag; sweep offline first, then at ply 1 against lr-trees-p1's 43.
    - 11-11: built as --split-anchor-l2 (chessai-hybrid6an). Offline no strength helps, and at the matched L* the
      fresh gate merges the tree. Dropped at ply 1 (theory-2 6.24). Revisit at ply 2 with --revise 2 or a longer
      window, L in {4e-4, 1.3e-3}.
  - Routing at ply 2 for its own sake: hold the splits (--revise 2 or a longer window than 12 ply-2 cycles).

**From theory 2** (2026-11-10 ~22:45, models/from-scratch/theory-2/summary.txt section 3, ranked; RESULTS section 10):
S1 shrinkage leaves (residual leaves with --residual-l2 > 0; tested offline 11-10 23:25: shrinkage moves them toward flat, never toward copies; dropped in this form, RESULTS section 4), S2 calibration-invariant
error targets (gap computed on per-child recalibrated predictions; small change in delegation.rs), S3 growth as
boosting (split children initialised along T4's per-region gradient directions). Five more in the file.

1. **Home-target refinement (T2).** R' targets the copy whose subtree, as actually routed, does best, with ties to the
   home pick. Today's home targets train R' on a counterfactual: the copies overrule their home pick on 35-40% of
   positions. Theory item 4. An option for P2.
2. **Gradient-disagreement split trigger (T4).** Split a leaf when its mean gradient differs across regions a router
   can learn; the first-order gain is 2|d||G_A|. Today's trigger measures stalling, which every leaf does eventually.
   Theory item 9.
3. **A router-split trigger that asks whether the copies would beat the router.** After P2, if re-routing pays in
   play. bddfix found the current trigger fires on a rise that happens anyway, which makes chains of routers.
4. **Boundary blending.** Predict with one leaf when the router is confident (top probability above c), blend the top
   two otherwise. Most positions still use one leaf. A play-time flag, so lines with identical weights differ only in
   c. Needs the user's view (NEXT-EXPERIMENTS decision 6).
5. **Path context, linear variants.** Cells for the parents' sets only, or for the K nearest sets. The full path
   (bin/chessai-bddctx) grows quadratically with chains of router splits (76 → 601 nodes, 6.3k → 259k table numbers
   in a 200-split test). CNBP cells double-count; test on LR leaves first.
6. **Cluster on top, error below** (11-08 design talk). A cluster router at the root hands each error sub-router a
   coherent region. Needs a per-depth routing option. single-split and cluster-error now show how the two compare.
7. **A growing mixture** (hierarchical MoE), if P1 favours M-L4-N. A stalled expert turns into a small NB-gated
   mixture of its own copies, trained side by side by responsibility. Design only after P1.
8. **Additive multi-router DAG** (idea 16). R routers each send a position to one child, and the position's LR is the
   base plus the deltas of the R children it lands in: still one LR per position, hard routing, no blending. Cells
   multiply (2^R) while every delta trains on all its router sends it. An oblivious tree, learned online. Theory item
   8 favours additive growth. After T3.
9. **Split-merge DAG** (idea 13). Leaves in different branches that have converged merge into one shared node, and
   their data re-pools. Router regret decides when a router itself should be revised.
   - 11-11 ~09:50, the user: one model in which a router "splits" (turns DAG-like) only when the loss measured to
     routing justifies it, so tree and DAG merge into one; with no router splits there are no path parameters, so it
     is exactly today's tree. Their "path parameters" are bddctx's path context (12 numbers per node per sibling set
     for B = 3; bddctx refuses the fresh gate, cluster routing and sharing, and was level offline).
   - Evidence: G4's grown 1M trees lose 0.0003-0.0013 nats per position to routing (group-level regret; RESULTS
     section 8), and every second-level router's children are near-copies. So the sharing move has more room than
     the routing move.
   - Design: theory-2 6.25. Cells from the router's own posterior (no new parameters); select-then-score G on
     fresh windows; a cell -> child mapping table (identity = today's tree, byte-identical when the gate never
     fires); leaf sharing for duplicate children, with path cells on shared leaves (per-route intercepts, shared
     slopes). The diagnostic chessai-routecells (models/from-scratch/unified/) decides what to build: G < 0.002
     everywhere -> sharing only; >= 0.005 at any router -> both, then one ply-1 run against T-P-Nc-merge.
10. **Hash tie-breaks for routers** (theory item 10). Break ties at random or by hash instead of always to the first
    child. A separate flag from --route-before-train; not built.
11. **Symmetry breaking after a split, without labels** (theory item 3). Consistent targets are silent while the
    copies are identical. Options: hash routing for a window (--split-bootstrap, now in error-targets), disjoint
    halves of the first window, small noise.
12. **Separate the training router from the play router.** The training router keeps leaf data broad (even random or
    balanced); the play router is refitted on the frozen leaves each version. errwhy B gained 51% of the reachable
    gain offline on frozen children, but online play routers failed (playrouter). Low priority: the leaves bind.
13. **Higher split limits.** The user's goal: trees and DAGs keep growing, so every design must scale linearly with
    size. split-growth tests limit 4.
14. **Seam-weighted or sampled leaf sharing.** Only if sharing returns: every LR-leaf share tree converged to near-copy
    leaves in the champion run.

### Leaves and learners

15. **FM tuning.** A smaller rate or an L2 penalty on the factor vectors, or fewer factors. FM was -259 against F-L
    untuned and is out of stage 1.
16. **FM as a router.** Built for later on the user's word (11-09 ~22:00: "we should still hook it up for the
    future"); fs/fmrouter/summary.txt. Not in any experiment.
17. **A small neural network** (one hidden layer) on the same encoding (11-06 idea 3). Further from the project's
    Bayesian roots, so only if the user wants it.

### Data and self-play

18. **P4: self-play against local at each leaf's best depth** (LR 2, probit 2, NB 0). NEXT-EXPERIMENTS decision 9.
19. **P5: self-play exploration policy:** jitter10, jitter20, two softmax temperatures. Deferred since 11-09.
20. **Self-play depth mix.** Play a fraction of each round's games at depth 2: search in play, not in labels.
21. **Which global-league games to train on.** On the twin stream the ply-2 entry's games carried all the league
    data's value, and games against stronger opponents were worth 2.7× per position. Weight positions by entry depth
    or opponent strength. For the global league's training rule.
22. **Calibration in self-play's own move choice.** Self-play reads uncalibrated specs; an underconfident depth-0 LR
    plays hot. Not measured.
23. **Search labels, priced.** If they return, charge the label search to the line's budget and compare with deeper
    self-play at equal CPU, as a 4-line sweep of label weight. The user (11-09): "basically equivalent to self-play
    with n+1 depth".
24. **Averaged (Polyak) entries** inside one cycle's versions, to steady version noise. Level in the one check.
25. Listed only, against the training standard: TD(lambda) targets; league-position replay in self-play rounds;
    colour-symmetry augmentation (on hold, the user 11-05: zero knowledge).
26. **Parity-indexed piece scales** (the analyst, theory-2 6.22, 11-11 ~07:30; parked behind the capture-extension
    decision).
    - What the weights show: the reverse rung flipped the pawn's value and the piece-to-queen ratios back to ply-3
      values, but kept W4's overall scale.
    - The idea: shared tables plus a small scale per piece type indexed by the horizon's parity, picked at search time
      (a few numbers per model). It would index the parity patch rather than remove its cause, as the extension
      would.
    - Testing it needs a field whose entry ply alternates by cycle, which the driver can't do yet.

### Measurement

26. **Equal time instead of equal depth.** Trees and mixtures evaluate more slowly; at equal time flat models would
    search deeper.
27. **An external anchor** (an engine at its weakest settings), for evaluation only, to put "random = 0" on a human
    scale.

### Performance

- models/from-scratch/fastbin/PERF-LEDGER.md is the list: done, queued (fixed-point flat probit), ruled out. Mixtures
  and FM still play the float code. Fixed-point incremental sums for flat models are done (hybrid4gfx, from
  fs/modelopt's fxlr2).

## Queued candidates (from the old league's queue, not run)

They were queued as league lineages (QUEUE.md, archived). In the new league they would be experiment lines.
- T-L-Ne-share-solo (self-play only) against T-L-Ne-share: tree against flat without league data.
- T-P-Ne-share: T-L-Ne-share with probit leaves.
- M-LP-N: M-LP-L with an NB gate. LR gates starve the probit expert; NB gates keep experts alive.
- M-L4-T: M-L4-N with a TAN gate.
- Offered on 11-09, not chosen: a twin of T-L-Ne-new-c (a non-share tree; ~2 CPU h); a league-only flat LR on the kept
  stream (~15 CPU min); a second window for the twin headline (v432-v440, ~10 CPU min); B' (L9's sum profile at the
  twin's scale); a DAG router-split design memo.
- Dropped: F-P-tau0-agdiv (the user, 11-09 ~10:55).

## Tried and rejected

One line each: what, the result, where it is.

| Idea | Result | Where |
|---|---|---|
| Residual leaves as built (11-07) | 171-266 below the plain tree at v60; confounded by the stream merge test | RESULTS 4; T3 reruns it |
| Stream merge test (scores the stream it trains on) | biased both ways; discarded the better 9-leaf tree | RESULTS 4 |
| Merges for error routers | banned by the user (11-08 21:10) | NEW-DESIGN rules |
| LR routers (cluster or error) | collapse to one leaf or lock in; churn in single-split | RESULTS 3 |
| Soft (posterior) error targets | LR leaves -23 against Best; route by who's winning | RESULTS 5, LEDGER |
| Switch routers | NB leaves -43, LR leaves -10 | LEDGER |
| Balanced targets | spread the data, but route by lift (theory); one leaf for ~3 rounds after each own step | RESULTS 5 |
| Expected-loss (eloss) training target | 0.0235 worse offline; specialisation feeds itself | RESULTS 5 |
| Online eloss play routers | 0.0034-0.0071 worse offline; -26 to -67 in arenas | RESULTS 5 |
| Random example assignment (assign) | level in play; nothing on top of sharing | RESULTS 5 |
| Blend at play time, whole tree | 41 and 1 at ply 2 for 4-5× CPU; the user: defeats delegation | RESULTS 5 |
| Blended self-play | -35 / -73 at equal CPU | RESULTS 6 |
| Per-leaf (per-region) calibration | gains offline, loses 19-147 in play | RESULTS 1 |
| Leaf sharing in LR-leaf trees | helped offline; every share tree converged to near-copy leaves | RESULTS 3, 4 |
| Share schedules and share for switch trees and the DAG | flat optimum; no bar cleared | RESULTS 4 |
| DAG router splits as built (5 variants) | never beat a same-leaves tree offline | RESULTS 4 |
| Cell-assigned DAG | no cell ever re-pointed; collapsed onto one leaf | RESULTS 3, 4 |
| Pre-split candidates | balances only the start of a split; fresh promotion never fired; 1.3× CPU | RESULTS 4 |
| Polyak averaging | level within a cycle; the earlier gain was phase | RESULTS 4 |
| Larger leaf step (young AdaGrad sums) | a ~1.7× step on all data -46 | RESULTS 3 |
| Smaller LR rate | didn't replicate in a second window | RESULTS 2 |
| F-L-agdiv (sums ÷2.7) | no gain over cycles 48-52 | RESULTS 2 |
| Probit tau 1e-4, 1e-3; tau as a fix | -415 to -561; tau makes probit worse on equal data | RESULTS 2 |
| Probit self-play at eps 0 | -346 at equal CPU | RESULTS 2 |
| LR + probit gate (M-LP-L) | it is its LR; no gate beats LR offline | RESULTS 2 |
| NB as a predictor (leaves, experts) | abandoned by the user (11-08 21:15); can't bootstrap alone | RESULTS 2 |
| Parent smoothing for deep NB trees (idea 14) | unlikely to pay: deeper NB trees no better, WANBIA small | RESULTS 2 |
| WANBIA feature weighting for NB | 0.4051 → 0.3874 only (LR 0.3293) | RESULTS 2 |
| TAN gate over LR experts | as hard as NB's, worse than MoE-c | RESULTS 3 |
| MoE with an LR gate | collapses onto 1-2 experts | RESULTS 3 |
| Hoeffding trees | closed by the user (11-09): the bound never decides | RESULTS 4, LEDGER |
| Split trials | -24 | LEDGER |
| Forgetting from scratch | every form hurt or tied | LEDGER |
| Seam-targeted sharing and search-aware validation (ideas 5, 12) | seams are not exploited at depth 2 | RESULTS 3 (optimism) |
| Route-once play | the user: misguided | LEDGER |
| Search labels in experiments | 12× training CPU, unpriced; out unless priced | RESULTS 2 |

## Closed with the old league (stopped 2026-11-09 20:06)

The new league has no own steps, gates or champion-run patience, so these no longer apply: the own step first
(spdrop fix b); gates in the league's policy; the gate-panel test; the retired-entry replay; phase-aware entries;
Polyak entries as a champion stabiliser; the league-data share options (cap, keep, ply-2 games only); share twins
resumed from switched trees; data-mix twins for d2jit lineages.
