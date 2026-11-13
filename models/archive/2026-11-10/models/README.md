# Models

Copied on 2026-11-05 from the Claude session scratchpad, which lives in RAM (`/tmp`). The
copy holds every finished run that would take real compute to redo, except
superseded or killed ones and identity checks.

Each folder is one self-play run with 1000 games per round. It holds:
- one model file per round (`cnbp500.model`, `lr300.model`, `bdt500.model`, ...);
- `results.txt`, one line per round;
- `<run>.log`, the training log, whose first line gives the run's settings.

Arena results: depth 2 vs depth 2, log-odds-softmax T=0.1, Elo with 95% range, usually 4 arena
seeds of 1000 games pooled. CNBP models and trees play calibrated:
`cal:../games/heldout-cnbp-d2-s555/round0:<kind>:<model>`. LR models play uncalibrated.

`LEDGER.md` lists every option tested so far with a verdict (adopted, closed, open).

## Names

`Architecture-Leaf-RoutersApproach-Depth`, e.g. `T-L-Nc-2`:
- Architecture: F flat, T tree (BDT), D DAG (BDD); mixtures of LR experts are `moe`.
- Leaf classifier: L logistic regression, N naive Bayes (CNBP).
- Router classifier: L or N; switch trees name the cluster router, then the error router (NN, NL, LN, LL).
- Routing approach: c cluster, e error, s switch (cluster first, error once it is better).
- Depth: the plies of search the model plays at; left off when speaking of the model itself.
- Not in the scheme: soft error targets, branch merges and leaf sharing, written as a note, e.g. `T-L-Ne (soft)`,
  `T-L-Ne (merges)`.

| Code | Runs |
|---|---|
| F-N | `ladder-cnbp-long`, `cnbp-from2000-if16` (the ÷16 baseline) |
| F-L | `lr-from-mix1000` (control of the T-L trees), `lr-lab-d2-r0.5-s1` (mix1000), `lr-d2play-T0.3-*` |
| T-N-Nc | `bdtc3-f2-s1x`, `bdtc3-f1-if16` |
| T-N-Ne | `bdt3-from2000-t5M-l5-f2`, `bdte3-f1-if16-log-best`, `-log-soft` (soft) |
| T-N-Le | `bdte3-f1-if16-log-best-lrr`, `-log-soft-lrr` (soft) |
| T-N-NNs | `bdtc3-f1-if16-switch3`, `-switch3-soft` (soft) |
| T-L-Nc | `lrtree-c3-nbr`, `lrtree-c3-nbr-s22` |
| T-L-Lc | `lrtree-c3-lrr` (collapsed to one leaf) |
| T-L-Ne | `lrtree-e3-nbr` (abs error), `lrtree-e3-nbr-log-best`, `-log-best-s22`, `-log-soft` (soft) |
| T-L-Le | `lrtree3-from-mix1000-f1` |
| T-L-NNs | `lrtree-c3-nbr-switch3` |
| T-L-LLs | `lrtree-c3-lrr-switch3` |
| D-N-Nc | `bddc3-f2-l10` (no router ever split: in effect a T-N-Nc) |

## From scratch (2026-11-06)

`from-scratch/` holds every archetype trained from scratch on the standard recipe: jitter self-play, result labels,
no search in training, no forgetting, 1000 rounds. The LR family uses seed 21 and the naive-Bayes family seed 11.
Trees are b3 t5M l5 f1, and every tree logs revision windows. Also in the folder:
- the batch scripts `batch29.sh`–`batch32.sh`;
- the arenas, `batch*-arenas.txt` with pooled results in `*-pooled.txt`;
- `make-roster.py`, which picked the tournament's representatives;
- the binaries used, in `bin/`.
- `batch28/`: batch 28's forgetting runs (naive Bayes from scratch), with their scripts, arenas and the
  forget-signal diagnostic (`diag-forget/`), copied from the scratchpad.

Results are in `LEDGER.md`, section "From scratch". The tournament's representative is in bold.

| Code | Runs |
|---|---|
| F-N | **`F-N-s11`** (batch 28's control) |
| F-L | **`F-L-s21`**; `F-L-s21-to2000`, continued to v2000 (links to v1–v1000) |
| moe | **`moe-s21`** |
| T-L-Nc | `T-L-Nc-s21`, **`T-L-Nc-merge-s21`** |
| T-L-Ne | `T-L-Ne-s21`, `-soft-s21`, **`-merge-s21`**, `-reset-s21`, `-merge-reset-s21` |
| T-L-Le | `T-L-Le-s21`, **`T-L-Le-merge-s21`** |
| T-L-NNs | `T-L-NNs-s21`, **`T-L-NNs-merge-s21`** |
| T-L-NLs | `T-L-NLs-s21`, **`T-L-NLs-merge-s21`** |
| T-L-LLs | `T-L-LLs-s21`, **`T-L-LLs-merge-s21`** |
| T-N-Nc | `T-N-Nc-s11`, `-share-s11`, `-merge-s11`, **`-share-merge-s11`** |
| T-N-Ne | **`T-N-Ne-s11`**, `-soft-s11`, `-merge-s11` |
| T-N-NNs | `T-N-NNs-soft-s11`, `-soft-merge-s11`, `T-N-NNs-s11` (best-child targets), **`T-N-NNs-merge-s11`** |
| F-FM | `F-FM-s21`: factorization machine, not in the tournament |

## High-watermarks by approach

| Approach | Model | Best result |
|---|---|---|
| CNBP, jitter self-play | `ladder-cnbp-long/cnbp2000.model` | plateau from ~v500 (seed 1); parent of every `*-from2000*` run |
| CNBP + one-time forgetting (**CNBP baseline**) | `cnbp-from2000-if16/cnbp500.model` | +55 (+45..+66) vs v2000; best CNBP |
| CNBP tree, error routing | `bdt3-from2000-t5M-l5-f2/bdt500.model` | +15 (+5..+26) vs v2000; gain was the forgetting at splits |
| CNBP tree, cluster routing | `bdtc3-f2-s13/bdt500.model` | +39 (+29..+50) vs v2000 (s11 +31, s12 +35); s11 vs ÷16 −27 (−37..−16) |
| CNBP DAG (BDD), cluster routing | `bddc3-f2-l10/bdd500.model` | +25 vs v2000; −18 (−28..−8) vs ÷16, the closest any tree came; no router splits fired |
| LR, jitter self-play, result labels | `lr-jit-s1/lr1000.model` | plateau ~v600–v1000; +374 vs CNBP v2000 |
| LR, jitter self-play, 50/50 depth-2 + result labels | `lr-lab-d2-r0.5-s1/lr1000.model` ("mix1000") | +48 vs result-label LR v1000; start of the LR plateau tests |
| LR, depth-2 search self-play from mix1000 | `lr-d2play-T0.3-from-mix1000/lr100.model` | +81 vs LR control v100; +392 vs CNBP v2000; plateaus ~v100 |
| LR, depth-2 search self-play from scratch | `lr-d2play-T0.3-scratch/lr300.model` | v50 already +93 vs mix1000; v300 vs the row above +8 (−2..+19): tied, joint best |
| Mixture of LR experts | `moe4-from-mix1000/moe100.model` | +2/+23 vs LR control v100 (2 arena seeds), about neutral; later versions worse |
| LR tree (LR leaves and routers) | `lrtree3-from-mix1000-f1/bdt300.model` | −1 (−11..+10) vs LR control v300, both calibrated (hard routing) |
| LR tree, LR leaves + CNBP cluster routers | `lrtree-c3-nbr/bdt300.model` | +29 (+18..+40) vs LR control v300 at d2, both calibrated; replicated by `lrtree-c3-nbr-s22` (+30) |

Finished later, not high-watermarks:
- `bdtc3-f1-if16`: cluster tree started exactly like the ÷16 baseline (v2000 ÷16, seed 11), no
  forgetting at splits. −66 (−77..−55) vs ÷16 v500.
- `bdtc3-f1-if16-trialw`: the same with split trials (`--split-trial 0 --trial-error winner`).
  −72 (−82..−61) vs ÷16 v500; −24 (−34..−13) vs the no-trial tree.
- `bdtc3-f1-if16-switch3`: `bdtc3-f1-if16` plus `--route-switch 3` (delegator 3 switched to error
  routing at ~round 200, the root at ~round 407). −43 (−54..−33) vs the no-switch tree,
  −117 (−128..−106) vs ÷16 v500.
- `lr-from-mix1000-if1` and `lr-from-mix1000-if100`: LR forgetting ÷1 and ÷100 (the control is ÷10).
- `lrtree-c3-nbr`: LR leaves with CNBP cluster routers (b3 t5M l5 f1, from mix1000 ÷10, seed 21).
- `lrtree-c3-nbr-s22` and its control `lr-from-mix1000-s22`: the same with training seed 22. Tree vs
  control d2 +30 (+19..+41), d1 +21 (+14..+28) (seed 21: d2 +29, d1 +32).
- LR-leaf trees from mix1000 (b3 t5M l5 f1, seed 21, 300 rounds), vs the LR control `lr-from-mix1000` v300, both
  calibrated, 4k games each:
  - `lrtree-c3-lrr`: LR cluster routers. They collapsed: every held-out position reaches one leaf, so it is in
    effect one LR. d2 +22 (+12..+33), d1 +14 (+7..+20): a measure of training-trajectory noise, not of
    delegation. vs `lrtree-c3-nbr` d2 −17 (−28..−7), d1 −14 (−20..−7).
  - `lrtree-c3-nbr-switch3`: `lrtree-c3-nbr` plus `--route-switch 3` (one switch, below the root). d2 +28
    (+18..+39), d1 +32 (+25..+39); vs `lrtree-c3-nbr` d2 −10 (−20..+1), d1 +2 (−5..+9).
  - `lrtree-c3-lrr-switch3`: `lrtree-c3-lrr` plus `--route-switch 3` (root switched to an LR error router at
    ~round 103; 3 switches). d2 +26 (+15..+36), d1 +10 (+3..+16); vs `lrtree-c3-lrr` d2 +8 (−3..+18).
  - `lrtree-e3-nbr`: CNBP error routers (abs error, best-child targets). d2 +14 (+3..+24), d1 +5 (−2..+12).
- `bdtc3-f1-if16-share0.1`, `-share0.3`: `bdtc3-f1-if16` plus `--leaf-share 0.1` / `0.3`. vs ÷16 v500: d2 −41
  (−51..−30) / −58 (−68..−47), d1 +15 (+7..+23) / +6 (−2..+14); share 0.1 vs no sharing d2 +12 (+1..+22).
  Seams per move 0.087 / 0.075 (no sharing 0.112).
- `bdte3-f1-if16-log-{best,soft}{,-lrr}`: error-routed trees from v2000 ÷16 (seed 11, f1, b3 t5M l5,
  `--route error --error log`, 300 rounds), best-child vs soft targets (`--error-target`), CNBP vs LR (`-lrr`)
  routers. vs ÷16 v300, d2 / d1: CNBP best −105 (−116..−94) / −35, CNBP soft −58 (−69..−48) / −8 (−16..−1),
  LR best −146 / −46, LR soft −705 / −293. Soft vs best: CNBP routers +57 (+47..+68), LR routers −549.
  CNBP soft vs the cluster tree `bdtc3-f1-if16` v300 −11 (−22..−1). The soft LR routers route by who is
  winning (leaf 1: 12.6% of white-won vs 3.6% of black-won positions; leaf 5 the reverse), so leaves carry
  opposite biases with 2.5–4.5-nat seams.
- `lrtree-e3-nbr-log-best`, `-log-soft`: `lrtree-e3-nbr` (LR leaves, CNBP error routers, seed 21) judged by
  log-loss (`--error log`), best-child vs soft targets. vs the LR control, d2 / d1: best +27 (+17..+38) / +32
  (+25..+39), level with the cluster tree `lrtree-c3-nbr`; soft +19 (+9..+30) / −12 (−19..−5); soft vs best d2
  −23 (−34..−13); soft vs `lrtree-c3-nbr` d2 −9 (−20..+1). The soft tree routes by who is winning (leaf 6:
  13.8% of white-won vs 1.9% of black-won positions; leaf 5: 7.6% vs 19.8%); mean |seam| 0.57 vs 0.37 nats.
- `bdtc3-f1-if16-switch3-soft`: `bdtc3-f1-if16` with switch routers on soft targets (`--route-switch 3
  --error-target soft --trial-error log`, 300 rounds). Root, delegators 3 and 6 switched (root gain +0.0024 per
  example, about 13 SE). d2 vs the cluster tree v300 −16 (−26..−6), vs `bdte3-f1-if16-log-soft` −11 (−22..−1),
  vs ÷16 v300 −70 (−81..−60); d1 vs ÷16 −4 (−12..+4). The same switch on the LR-leaf tree never fired (the soft
  error router lost every window), so that run equals `lrtree-c3-nbr` and is not kept; its log is in
  `../selfplay/session-2026-11-05/results/`.
- `lrtree-e3-nbr-log-best-s22`: `lrtree-e3-nbr-log-best` with training seed 22. vs the seed-22 LR control d2 +23
  (+12..+34), d1 +1 (−6..+8); vs `lrtree-c3-nbr-s22` d2 −19 (−29..−8). Seed 21 head-to-head with `lrtree-c3-nbr`:
  d2 +0 (−10..+11), d1 −1 (−8..+6).

## Ratings

`../selfplay/tournament-2026-11-05/`: the 11 high-watermarks above at search depths 1, 2 and 3,
plus random play, in 50 double round-robins (56,100 games). `ratings.md` is the rating list,
`head_to_head.md` the score matrix, `games.csv` every game; `chessai tournament` with the same
`players.txt` resumes or extends it.

`../selfplay/tournament-2026-11-06/`: the from-scratch archetypes' representatives (above) at depths 1, 2 and 3,
plus random play: 37 players, 50 double round-robins, 66,600 games. `../selfplay/tournament-2026-11-06-premerge/`
is the same before merges (34 players; its README says why it exists).

## Elsewhere

- `../games/`: held-out game sets.
  - `heldout-cnbp-d2-s555`, `heldout-cnbp-d2-s556`: 2000 depth-2 T=0.1 games each of CNBP v2000. s555 fits calibrations; s556 is for held-out evaluation.
  - `heldout-lr-d2-s555`: the same for mix1000.
  - `heldout-parent-jitter-s600`, `heldout-parent-jitter-s601`: jitter-play games of v2000.
  - `heldout-d2play`: games by the search-play start model.
  - `heldout`, `heldout-t01`: CNBP v200 games.
- `../selfplay/session-2026-11-05/`:
  - `bin/`: the binary for each stage; `chessai-switch` is the current one.
  - `scripts/`: `batch*.sh`, `elo.sh`, `pool.py` and analysis scripts. They still point at the scratchpad paths.
  - `results/`: arena and batch outputs.
  - `tools/`: sources of the samestream, blendstats, errroute and evalbench tools.
