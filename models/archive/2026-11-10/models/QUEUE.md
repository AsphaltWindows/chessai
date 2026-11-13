# Queue (since 2026-11-09 ~01:15)

The user's rule: no new league lineages until active lineages fall to about 60 through the champion rule's own
finishes. After that, add only enough to hold about 60. Offline experiments run meanwhile, kept short.
Q-numbers refer to the user's agenda (memory: research-questions.md).

## League lineages queued (each one factor from the named comparator)
- F-P-tau0-agdiv-c (Q2): F-P-tau0-c's latest version with its weight variances multiplied once, the probit
  counterpart of F-L-agdiv-c. Comparator: F-P-tau0-c (and F-L-agdiv-c vs F-L-c). Suggest dropping (11-09 06:05):
  F-L-agdiv-c gained nothing in 5 cycles and tau (a step floor) made the probit worse.
  DROPPED 2026-11-09 ~10:55 by the user ("sure let's drop it"), after F-L-agdiv-c averaged -19 over cycles 48-60.
- T-L-Ne-share-solo-c (Q3, Q6): T-L-Ne-share-c's line, self-play only. Comparator: T-L-Ne-share-c, plus F-L-solo-c
  for tree vs flat without league data.
- T-P-Ne-share-c (Q1): T-L-Ne-share-c with probit leaves. Comparator: T-L-Ne-share-c.
- M-LP-N-c (Q1): M-LP-L-c with an NB gate. M-LP-L-c's LR gate starved its probit expert (whytrees ~01:45), while NB gates
  keep experts alive (M-L4-N-c). Comparator: M-LP-L-c.
- M-L4-T-c (Q8): M-L4-N-c with a TAN gate (the tangate model type). Comparator: M-L4-N-c.

## Pending the user's review
- Q7 renames: M-L4-L-hard-c and M-LP-L-hard-c predict with one expert, so by the Router/MoE spec they are routers
  (one-level trees whose router is trained by responsibilities). Name proposals go in the morning report.

- League drift and gates (discussed 11-09 ~10:45; the user: "let's discuss the league more later"): scale stable, top
  progress ~1 Elo a cycle, gates claim 10-20x the league's gains. Costed, not run: gate-panel test ~10 CPU min,
  retired-entry replay ~5 CPU min (IDEAS.md 11-09 ~10:45).
- Offered in the morning review, not chosen yet: a twin of T-L-Ne-new-c (a non-share tree, v205, ~2 CPU h; T-L-Ne-share-c's
  leaves are near-copies, so finding 1 was the least favourable case for routing); a league-only flat LR on the kept
  stream (~15 CPU min: what self-play adds once the league games are there).

- Faster evaluation with repeatable games (the user, 11-09 ~11:30: "I would like the games to remain repeatable, so if
  it's possible to optimize the models while maintaining that I'm all for it"): NNUE-style incremental sums in
  fixed point (exact, path-independent, repeatable; today's games change once at the switch). Start after the
  field-panel test: prototype on flat LR, measure, then the other model kinds; deploy at a boundary with the user's OK,
  together with the exact self-play fast paths and any league-share change (one break in conditions, not several).

## Offline experiments (status kept current)
- DONE 06:05 (2026-11-09): the full-data twin of T-L-Ne-share-c and everything on its kept stream
  (models/from-scratch/twin/league/summary.txt, "STATE OF THE NIGHT"; morning summary models/MORNING-2026-11-09.md).
  Stands: league data worth ~200-300 Elo to a flat LR; d2-entry games carry it; probit trails LR more with data
  (d2 -10/-50/-101); tau worsens the probit; larger LR rates lose; tree level with or behind a flat twin on equal
  data (-26 [-42,-10] over v444-v452, one window). Fell: routing-region and interaction readings for L9 (version
  noise), "0.025 beats 0.05" (didn't replicate in a second window).
- Open, costed, not run (the user's call): a second window for the headline (v432-v440, ~10 CPU min).
- Set aside (revivable): B' (L9's sum profile at the twin's scale), LR with forgetting (smaller/larger steps give nothing).
- DONE 02:30: why trees win, plus routing headroom (whytrees/summary.txt).
- Not started: a DAG router-split design memo (Q5); no routing headroom was found, so it waits for the user.

## Q9 compute profile (added ~01:50)
- DONE ~02:55 (models/from-scratch/movegen/summary.txt): tournament 70% of cycle CPU; movegen ~17% of the league's CPU;
  caches don't pay at d<=2; three exact fast paths cut d2 games 21-46% (tournament est. 25%), identical games.
- DEPLOYED 03:51 at the 50->51 boundary (config.json swapped 03:50:14, all deploy checks ok, loop pid 919919).
  Cycle 51's tournament on hybrid4: 20.64 min for 270,280 games (cycle 50 on hybrid3: 33.48 min for 286,800):
  -35% wall a game; 8,187 CPU s, -31% CPU a game against cycle 49 (the controlled same-games measure: -23%).
  Own-cal runs on the tournament binary too: 14 own-cal sets re-made by hybrid4 are byte-identical.
  Armed 03:47:44 on the user's OK ("if you have unit tests for everything you should turn it on"; lazy-candidates
  unit test added, 146 tests pass, binary unchanged); deploys at the 50->51 boundary (watcher 917789). chessai-hybrid4 (md5 1e050a86..., models/from-scratch/hybrid4-src) in
  cycles/champ/staged-pending-hybrid4/. Identical to hybrid3 on 4,125 games of cycle 49's field (games.csv rows, 16,500
  saved game files); CPU -23% (~2,600 CPU s a cycle). Arm: python3
  /home/iv/dev/chessai/cycles/champ/staged-pending-hybrid4/arm.py (models/from-scratch/movegen/STAGED.txt).

## Analyses started 2026-11-09 ~17:46 (the user's requests)
- Leaf view of T-L-Ne-share3-c (v394, 1889.9) and T-L-Ne-fresh-c (v1714, 1819.8; champion v1659 1855.5): heatmaps of the
  leaves, routing, similarity (models/from-scratch/leafview/, ~15 CPU min).
- Degeneracy census of every tree, DAG and mixer lineage of the champion league: ratings, structure, usage on a fixed
  position set, leaf similarity, verdict (healthy router or mixer, router->flat, mixer->flat, mixer->router, converged
  leaves), dominant leaf or expert head to head against the whole at ply 2 (models/from-scratch/degeneracy/, ~30 CPU min).
- DONE 18:11 leaf view (models/from-scratch/leafview/share3.html, fresh.html, summary.txt):
  - share3 v394: 11 leaves, near-copies (median weight correlation 0.9989; routed leaf disagrees with the others on who is
    better in 2.6% of positions); split copies plus leaf share 0.3 (small leaves get 1-6% of their weight from their own
    positions). v394 is 58 above its previous ten versions' mean (version SD ~17: partly luck).
  - fresh v1659: one branch merge (v1486->1487), distinct leaves (0.986; disagree 13.3%), L2 takes 60.6%, two leaves
    nearly unused.
  - Win-vs-loss log-loss on decisive league positions: share3 0.3801, fresh 0.3818, F-L-c 0.3911; neither tree beats its
    best single leaf beyond noise. Side finding: both lineages' recent self-play is 91% draws (league positions: 13.5%
    from drawn games), so three-class log-loss is dominated by the draw mismatch.
- DONE 18:20 degeneracy census (models/from-scratch/degeneracy/summary.txt; 1.3 GB, 697 MB of it own-cal games):
  - 25 of 64 tree/DAG/mixer lineages flat in effect at their latest version (14 converged leaves, 8 router->flat,
    3 mixer->flat), 31 at their champion versions; plus 2 mixers flat by weight and 1 mixer->router (M-L4-N-c).
    "Flat in effect": whole vs dominant part correlate >= 0.9922 (flat LR vs flat probit) on 256k cycle-70 positions.
  - Every LR-leaf tree with --leaf-share converged; 18 of 20 merge lineages were flat or converged at their champion;
    some LR routers send ~100% to one leaf; NB-routed trees without share/merge stay healthy (15).
  - 4 of the top 8 are flat in effect, share3 included (busiest leaf alone level with the whole, 5 [-41, 51]). Busiest
    leaf beats the whole in T-L-LLs-c (145), M-L4-L-hard-c (123), T-L-NLs-merge-c (94), T-N-LLs-merge-c (63); the whole
    beats its dominant part only in M-L8-L-c (89) and D-L-Ne-rf16-c (63). 200-game matches, CI about +-45.
