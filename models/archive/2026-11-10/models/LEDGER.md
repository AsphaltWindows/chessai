# Ledger of options

One line per option or idea tested so far, with a proposed verdict, for review. Drafted
2026-11-05 from the experiment logs; the user decides what to close.

- **Adopted**: part of the current best pipeline.
- **Closed**: settled; no more tests. "Remove?" marks code that could go.
- **Open**: a decisive test is running, or one more is worth it.
- **Tool**: a diagnostic, kept.

Elo is at depth 2, 4,000 games, unless noted; "vs control" means against the same recipe without
the option. Names follow the scheme in `README.md` (e.g. `T-L-Nc`).

## Conclusions so far

1. Logistic regression is the evaluator. Naive Bayes plateaus, double-counts correlated features
   and cannot learn from model-derived labels; LR beats it by ~+370.
2. Better games are the strongest lever found (LR self-play with depth-2 search: from scratch, v50
   is +93 over the jitter plateau), but they are expensive; deferred so we can study architecture
   on one standard recipe per family (see Base model and data).
3. Forgetting helped naive Bayes only after a very long accumulation (÷16 at v2000, +55); from
   scratch every form tried hurts or ties (batch 28: ÷16 at the v500 plateau −113, constant ÷1.002 or
   ÷1.005 per round −53 / −40, age-scaled γ=3 −531, γ=1 −1). LR hardly reacts (÷1 −5, ÷100 −12 vs ÷10).
4. Naive-Bayes trees never beat the ÷16 naive Bayes; their early gains were the forgetting at
   splits.
5. From a plateau start (mix1000), one tree archetype beat its flat control: LR leaves with naive-Bayes
   cluster routers (`T-L-Nc`, +29 and +30 on two seeds); error routing with log-loss close behind (`T-L-Ne`,
   +27 / +23). From scratch, every tree loses to its flat model unless it can undo splits (conclusion 8).
6. Trees predict held-out games better yet play worse unless their leaves agree where games cross
   between them (seams): search exploits the disagreements.
7. Measure at depth 2 (or more): one-ply results showed a plateau that was not there.
8. From scratch (batches 29–30), trees split while the flat model is still improving and lose to it: LR trees
   −69 to −144 vs F-L, naive-Bayes trees −129 to −476 vs F-N. Branch merges (`--revise 0`) turn `T-L-Ne` from −83
   into +36 (+26..+47) vs F-L. A branch is merged when its winner log-loss is worse than its delegator's shadow's,
   the unsplit leaf, which keeps training on everything through the delegator. The root split was undone 7 times
   in rounds 132–559. Merges help every tree archetype but `T-N-Ne` (+41 to +190 vs twin); with them the LR trees
   are level with F-L in the archetype tournament, and `T-L-Ne` leads it (+26 at d2, +46 at d3 over F-L).

## Base model and data

Standard recipe (user, 2026-11-05), which every architecture comparison uses: self-play FROM SCRATCH with
jitter games and result labels, no search anywhere in training (neither search-played games nor look-ahead
labels), seed 11 for the naive-Bayes family and 21 for the LR family (22 to replicate). Rounds: 1000 proposed,
not yet confirmed. At 1000 the LR family is still climbing at depth 2 (v1000 vs v750: F-L +69 (+58..+80), `T-L-Nc`
+53, merging `T-L-Ne` +30), while naive Bayes has stopped (F-N +10 (−1..+20)). F-L peaks there: v2000 vs v1000 −25. No forgetting for either family (batch 28). The archetype tournament:
arenas pick each archetype's best variant, then the winners and random play a round-robin at depths 1–3.

| Option | Verdict | Evidence | Code |
|---|---|---|---|
| Naive Bayes (CNBP) as evaluator | Closed (kept as router) | Plateau ~v300–v500 at d2; LR +374 over CNBP v2000 | Keep |
| Logistic regression (`--model lr`) | Adopted | Above; LR seed spread ≈ 0 by v300 | Keep |
| Factorization machine (`--model fm`) | Open (untuned) | From scratch v1000 −259 vs F-L, and worse held-out prediction (0.531 vs 0.497): the interactions as trained add noise | Keep |
| Mixture of LR experts (`--model moe`) | Closed | Experts split by game phase unprompted, but play is neutral or worse: v300 −28/−13 vs control; smoothed gate −11..−27; on search-play data −14/−9; from scratch v1000 −205 vs F-L | Remove? |
| Self-play with search (`--depth 2`, log-odds-softmax T=0.3) | Deferred (user, 2026-11-05): too expensive; architecture first | From scratch v50 +93 over the jitter-LR plateau; CNBP collapses with it (−249/−352) | Keep |
| Jitter play (wld-score-jitter10) | Adopted: the standard games for both families | CNBP learns only from it | Keep |
| Softmax sampling of expected score | Closed | v20 vs jitter +7/+6/+2 (±7) | Remove? |
| Log-odds-softmax as a one-ply data policy | Closed for CNBP; open for LR (deferred with the data side) | CNBP: T=1 ≈ jitter, T=0.3 −28..−124. Jitter is about as random as T=1: on v300 models it plays a non-top move 85% of the time (T=3 90%, T=1 87%, T=0.3 70%, T=0.1 38%, LR; CNBP similar), so a higher temperature changes little; only a lower one (more exploitation) is a real alternative, untested for LR | Keep (it is the play policy) |
| Greedy data | Closed | Collapses to 1–2 repeated games | — |
| Round size | Closed | Irrelevant; total games matter | — |
| Search labels, 50/50 with result (`--label-depth 2 --label-result 0.5`) | Out of the standard (user: no search in training) | +~100 over result labels, two seeds; pure look-ahead labels reverse by v100; CNBP collapses under any model-derived labels | Keep |
| Fixed teacher (`--label-model`) | Closed | Diagnosed CNBP's double counting | Remove? |
| Colour symmetry | On hold (user: zero knowledge) | CNBP symmetrised −109/+43/+36/−28 | — |

## From scratch (batches 29–32, 2026-11-06)

Every archetype trained from scratch on the standard recipe, 1000 rounds (LR family seed 21, naive-Bayes family
seed 11), trees b3 t5M l5 f1. Depth 2, 4,000 games, calibrated on the family's held-out games. "Twin": the same
run with log-only revision, identical until the option first acts.

| Comparison | Elo (95% range) | Note |
|---|---|---|
| `T-L-Nc` vs F-L | −92 (−103..−81) | |
| `T-L-Ne` vs F-L | −83 (−94..−72) | |
| `T-L-Le` vs F-L | −114 (−125..−103) | |
| `T-L-NNs` vs F-L | −144 (−156..−133) | 1 switch |
| `T-L-NLs` vs F-L | −134 (−146..−123) | 3 switches |
| `T-L-LLs` vs F-L | −69 (−80..−59) | 1 switch |
| `T-L-Ne` soft vs best-child targets | +21 (+11..+32) | |
| `T-L-Ne` merges vs twin | +128 (+116..+139) | 11 merges; 5 leaves at v1000 (twin: 11) |
| `T-L-Ne` leaf resets vs twin | +46 (+36..+57) | 23 resets |
| `T-L-Ne` merges vs F-L | +36 (+26..+47) | the first from-scratch tree above its flat model |
| `T-N-Nc` vs F-N | −275 (−289..−261) | |
| `T-N-Nc` leaf sharing 0.1 vs none | +158 (+146..+169) | share vs F-N −143 (−155..−132) |
| `T-N-Ne` vs F-N | −129 (−141..−118) | |
| `T-N-Ne` soft vs best-child targets | −302 (−317..−287) | the reverse of +57 from v2000 ÷16; soft vs F-N −476 |
| `T-N-NNs` (soft targets) vs F-N | −405 (−425..−388) | 5 switches |
| `T-L-Ne` merges and resets vs merges | −9 (−20..+1) | 10 merges, 3 resets |
| `T-L-Nc` merges vs twin | +76 (+65..+86) | 12 merges |
| `T-L-Le` merges vs twin | +112 (+101..+123) | 12 merges |
| `T-L-NNs` merges vs twin | +129 (+117..+140) | 12 merges |
| `T-L-NLs` merges vs twin | +127 (+116..+139) | 12 merges |
| `T-L-LLs` merges vs twin | +69 (+58..+79) | 10 merges |
| `T-N-Nc` merges vs twin | +41 (+30..+51) | 14 merges |
| `T-N-Nc` sharing + merges vs sharing | +37 (+26..+47) | 13 merges; vs merges alone +168 |
| `T-N-Ne` merges vs twin | −52 (−63..−42) | 8 merges: the one archetype merges hurt |
| `T-N-NNs` merges vs twin (soft targets) | +104 (+93..+115) | 9 merges |
| `T-N-NNs` best-child vs soft targets | +143 (+132..+155) | 4 switches |
| `T-N-NNs` best-child targets + merges vs best-child | +190 (+178..+202) | 12 merges |
| MoE vs F-L | −205 (−217..−192) | |
| Factorization machine (`--model fm`, 8 factors) vs F-L | −259 (−273..−246) | not in the tournament; held-out calibrated winner log-loss 0.531 vs F-L 0.497 |

Rounds, depth 2: v1000 vs v750 F-L +69 (+58..+80), `T-L-Nc` +53 (+42..+63), merging `T-L-Ne` +30 (+20..+41), F-N +10
(−1..+20). F-L continued to v2000 (`F-L-s21-to2000`): each 250-round step after v1000 is within ±8 (+4, −8, −6, −3), and v2000
vs v1000 directly is −25 (−35..−14), so flat LR peaks at about v1000; the trees were still climbing there.

Archetype tournament (`../selfplay/tournament-2026-11-06`, 37 players, 66,600 games, Elo relative to random, ±16–24
against the field's mean). Picks by the arenas above: merges on every tree but `T-N-Ne`, best-child targets on
`T-N-NNs`, sharing on `T-N-Nc`.

| Archetype | Representative | d1 | d2 | d3 |
|---|---|---:|---:|---:|
| `T-L-Ne` | merges | +867 | +1457 | +1871 |
| F-L | | +865 | +1431 | +1825 |
| `T-L-Le` | merges | +884 | +1427 | +1808 |
| `T-L-NLs` | merges | +860 | +1423 | +1818 |
| `T-L-Nc` | merges | +864 | +1421 | +1790 |
| `T-L-NNs` | merges | +859 | +1415 | +1800 |
| `T-L-LLs` | merges | +862 | +1408 | +1823 |
| MoE | | +828 | +1282 | +1566 |
| F-N | | +593 | +1173 | +1468 |
| `T-N-Nc` | sharing + merges | +562 | +1107 | +1412 |
| `T-N-NNs` | best-child targets + merges | +577 | +1101 | +1429 |
| `T-N-Ne` | base | +570 | +1054 | +1371 |

The same tournament before merges (`../selfplay/tournament-2026-11-06-premerge`, 34 players) had the LR trees
without merges 33–88 below F-L at d2 and 94–175 below at d3.

## Forgetting

| Option | Verdict | Evidence | Code |
|---|---|---|---|
| One-time forgetting from a plateau (`--init-forget`) | Closed for the from-scratch standard | From v2000: CNBP ÷16 +55 (÷8 +20, ÷32 +41); LR ÷1 −5, ÷100 −12 vs ÷10. From scratch (batch 28, d2, 4k): ÷16 at the v500 plateau −113 (−124..−102) vs continuing; a second ÷16 on the ÷16 run −198 | Keep (starting from saved models) |
| Steady per-round decay (`--round-forget`) | Closed for CNBP; untested for LR | From v2000: ÷1.005/round −97 vs ÷16; ÷16 then ÷1.002/round −4. From scratch, v1000 vs none: ÷1.002 −53 (−64..−43), ÷1.005 −40 (−50..−30) | Keep; trees can forget now (`Graph::forget`) |
| Age-proportional fading (`--age-forget γ`): round s weighs ((s+1)/(N+1))^γ at round N | Closed | From scratch, v1000 vs none: γ=1 −1 (−11..+9), γ=3 −531 (its versions swing between white-winning and black-winning games: it chases its own last games) | Keep (cheap) |
| Predictive loss as the forgetting trigger | Moot (no forgetting from scratch); not usable as is anyway | v500s of the forget runs on held-out d2 games rank ÷8 best by winner log-loss, while play ranks ÷16 > ÷32 > ÷8; progress of v(t) over v(t−50) on fresh games is ~0 from v200, before the d2 plateau | — |
| Forgetting at splits (`--forget-factor`) | Settled at 1 | f10 starves leaves (≈ −300); CNBP trees' gains with f2 were this forgetting | Keep; the CLI default (10) should become 1 |

## Play and evaluation

| Option | Verdict | Evidence | Code |
|---|---|---|---|
| Log-odds move ranking | Adopted | +100s over expected score; evaluation policy log-odds-softmax T=0.1 | Keep |
| Depth-2 search in arenas | Adopted as the standard (d1, d3 as profile) | One-ply results misled | Keep |
| Whole-model calibration (`cal:`) | Adopted for CNBP and trees | +35 for CNBP; undoes trees' compressed log-odds | Keep |
| Per-leaf calibration (`leafcal:`) | Closed | Creates seams across phase leaves; never clearly beat `cal:` | Remove? |
| Blending leaves (`blend:`) | Closed (user: defeats delegation) | Also hurts cluster trees (−1..−17) | Remove? |
| Draw-aware search value (`--a-value score`) | Closed | Neutral at d2 (+107 vs +104) | Remove? |
| Route-once play | Removed | — | — |
| `hashmix:` | Tool | Mixing two models by hash costs ~−90: per-model offsets | Keep |

## Trees: when to split

| Option | Verdict | Evidence | Code |
|---|---|---|---|
| Split rules (`worse`, `stall-high`), `--split-z` | Closed: keep `worse` | Rules changed when trees split, never whether they played better | Remove `stall-high`? |
| Error measure (`--error`) | Adopted: `log` for LR-leaf error trees | `T-L-Ne` with log +27 vs abs +14 vs LR control | Keep |
| `--error winner` for splitting | Closed | Counts only decisive examples (2.7%), so t5M trees never split | Keep as `--trial-error` judge |
| Split trials (`--split-trial`) | Closed; replaced by revision | −24 vs no trials | Remove once revision is settled |
| Branch factor, split threshold, limit | Settled: b3, t5M, l5 | l10/l20 no better | Keep |
| Hoeffding trees (`--model ht`, `x[f]==v` splits; `T-L-H-c`, `T-N-H-c`) | Closed (user, 2026-11-09: "a bit too deterministic for a game like chess") | The Hoeffding bound never decides: every split comes from the tie rule on single-square tests (~0.005 bits, top two within 1e-4), mostly pieces on home squares. Offline −90 (LR) / −200 (NB) vs flat. League c74: `T-L-H-c` best 1774 (unchanged since c55) vs `F-L-c` 1833; `T-N-H-c` 1540 vs `F-N-d2jit-c` 1558. The model-gain variant (validated on held-out games) never split | Keep while the live lineages run |

## Trees: routing

| Option | Verdict | Evidence | Code |
|---|---|---|---|
| Cluster routing, CNBP routers, LR leaves (`T-L-Nc`) | Adopted as the tree reference | +29/+30 vs LR control (seeds 21/22); d1 +32/+21. From scratch with merges level with F-L (tournament d2 −10, d3 −35) | Keep |
| Cluster routing with CNBP leaves (`T-N-Nc`) | Closed | +31..+39 vs parent v2000 but −27 vs ÷16 | — |
| LR cluster routers (`T-?-Lc`) | Closed | Collapse to one leaf | — |
| Error routing, CNBP routers, LR leaves (`T-L-Ne`) | Adopted, with merges: the top archetype | +27/+23 vs LR control; vs `T-L-Nc` +0 (s21), −19 (s22). From scratch with merges +36 vs F-L (arena); tournament d2 +26, d3 +46 over F-L | Keep |
| Error routing with CNBP leaves (`T-N-Ne`) | Closed | −52..−105 vs ÷16 | — |
| LR error routers | Closed | CNBP leaves −146; LR leaves (`lrtree3`) −1 | — |
| Soft error targets (`--error-target soft`) | Closed | LR leaves −23 vs best targets (routes by who is winning; seams 0.57 vs 0.37 nats); CNBP leaves +57 over best but still −58 vs ÷16. From scratch: `T-L-Ne` +21, but `T-N-Ne` −302 and `T-N-NNs` −143 | Remove? |
| Switch routers (`--route-switch`) | Closed | CNBP leaves −43, soft −16 vs cluster; LR leaves NN −10 (d2) / +2 (d1) vs no switch. From scratch with merges, `T-L-NNs`/`NLs`/`LLs` level with `T-L-Nc` and F-L | Keep for now |
| Different cluster and error routers (`--error-router`) | Closed | From scratch with merges `T-L-NLs` ≈ `T-L-NNs` (tournament d2 +1423 vs +1415) | Keep for now |
| Drift (error router nudges cluster boundaries) | Removed | Didn't move boundaries | — |
| Leaf sharing (`--leaf-share`) | Adopted for `T-N-Nc` | +12 vs no sharing, but −41 vs ÷16 (CNBP leaves only). From scratch +158 vs none, +168 with merges vs merges alone | Keep |

## Trees: revisable structure (batch 26)

| Option | Verdict | Evidence | Code |
|---|---|---|---|
| Branch merges into the shadow (`--revise 0`) | Adopted for trees (not `T-N-Ne`) | From scratch vs twin: LR trees +69..+129, `T-N-Nc` +41, `T-N-NNs` +104/+190, `T-N-Ne` −52; merging `T-L-Ne` +36 vs F-L | Keep |
| Seam penalty (`--seam-weight`) | Open | Waiting for the log-only run's scales | — |
| Leaf resets to the shadow (`--reset-leaves`) | Closed | From scratch `T-L-Ne` +46 vs twin, but nothing on top of merges (−9 (−20..+1)) | Remove? |

## DAG

| Option | Verdict | Evidence | Code |
|---|---|---|---|
| BDD (`--model bdd`) | Deferred (user): revisit when routing looks like the bottleneck | Error-routed BDDs grew chains of routers re-routing among 3 leaves (−33 vs CNBP); cluster-routed BDDs never split a router, which makes them BDTs. Signals to log: each router's regret (error of its pick minus the best child's) and near-duplicate leaves in different branches | Keep |

## Tools

`eval-loss`, `calibration`, `policy-stats`, `rank-test` (too coarse to resolve 15–30 Elo), `leaves`,
`tournament`, and the scratchpad tools `seams` and `leafstats`.
