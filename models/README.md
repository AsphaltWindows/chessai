# models/

This folder holds trained models and run folders, the raw write-ups of every study (from-scratch/), and the research
docs below. Restructured on 2026-11-10: the docs were condensed, and the originals are kept byte for byte in
archive/2026-11-10/ (md5 lists beside them).

## The docs

| Doc | What it is |
|---|---|
| NEXT-EXPERIMENTS.md | Start here. What runs now, the user's open decisions, the queue, the headline results. |
| RESULTS.md | Every experiment and study since 11-05, by topic, with its numbers and a pointer to its write-up. |
| PATH-FORWARD.md | The routing and DAG plan of 11-10 evening, with the ply question: what changed, my reading, the proposed sequence. |
| IDEAS.md | Open ideas, queued candidates, and what was tried and rejected (one line each). |
| NEW-DESIGN.md | The new league design: decisions in force, rules for experiments, open questions. |
| LEDGER.md | The options ledger of 11-05/06 with verdicts; a status block at the top lists what changed since. |
| QUEUE.md | Retired stub: the old league's queue (archived); pointers to where queued work lives now. |
| archive/2026-11-10/ | The docs as they were before the restructure, verbatim: models/ (8 docs, including the NIGHT and MORNING reports of 11-09) and leagues/ (4 docs). |

## Where things live

| What | Where |
|---|---|
| Experiments (one folder each: config.json, cycles.csv, tour/, models) | leagues/experiments/<name>/ |
| Arenas (fixed models, no training) | leagues/arenas/<name>/; how: leagues/ARENA.md |
| Experiment and arena specs | leagues/specs/batch1/, batch2/, batch3/, arenas/ |
| League engine, dashboard, recipes, prices | leagues/leagues.py, dashboard.py (port 8766), recipes.txt, prices/ (P2 is the default) |
| Engine docs | leagues/BUILD.md (how to run), FORMATS.md (files), DASHBOARD.md, ARENA.md |
| Binaries | models/from-scratch/bin/ (never overwritten); their sources in models/from-scratch/*-src/ |
| Raw write-ups | models/from-scratch/<topic>/summary.txt (some folders have a README.txt instead, or only data) |
| Theory (the backprop lens) | models/from-scratch/theory/summary.txt |
| Performance ledger and its 6-hourly review | models/from-scratch/fastbin/PERF-LEDGER.md |
| Analysis scripts of 11-10 (curtab.py and the rest, copied from the session's RAM scratchpad) | models/from-scratch/analysis-2026-11-10/ (README.txt) |
| Held-out game sets | games/ (heldout-*; see archive README "Elsewhere") |
| Old league runs | cycles/ (first run) and cycles/champ/ (champion run), with selfplay/league-cycles/ and selfplay/league-champ/ |
| Tournaments of 11-05/06 | selfplay/tournament-2026-11-05/, tournament-2026-11-06*/ |
| Run folders of 11-05/06 | models/<run>/ and models/from-scratch/<code>-s21 or -s11 (below) |

Each run folder of 11-05/06 is one self-play run with 1000 games a round: a model file per round, results.txt, and a
training log whose first line gives the settings.

## Names

- Models: Arch-Leaf-RoutersApproach, for example T-L-Nc.
  - Arch: F flat, T tree (BDT), D DAG (BDD), M mixture of experts.
  - Leaf: L logistic regression, P probit, N naive Bayes (CNBP); FM is F-FM.
  - Routers: L, N, T (TAN); switch trees name the cluster router, then the error router (NN, NL, LN, LL).
  - Approach: c cluster, e error, s switch.
  - Mixtures: M-<each expert kind with its count>-<gate>, for example M-L4-N (a count of 1 left out: M-LP-L).
  - Qualifiers after a hyphen: -merge, -share, -fresh, -pre, -bal, -hard, -solo, -d2jit and so on. The old leagues
    added -c to every lineage (F-L-c).
- New-league lines: <recipe>+<flag>=<value>....<option>, with .loc (local), .sp (self-play) or .glb (global), and #<r>
  for a replicate. Example: T-P-Ne+split-limit=4+split-threshold=1000000.loc.
- New-league versions: E.<S|L|G>.<champion>.<cycle> in experiments, G.<champion>.<cycle> and F (frozen anchor) in the
  global league.

## Run folders of 11-05/06

Results: LEDGER.md and RESULTS.md section 11. Per-run notes: archive README.md.

| Code | Runs (11-05, from a plateau start) |
|---|---|
| F-N | ladder-cnbp-long, cnbp-from2000-if16 (the ÷16 baseline) |
| F-L | lr-from-mix1000 (control of the T-L trees), lr-lab-d2-r0.5-s1 (mix1000), lr-d2play-T0.3-* |
| T-N-Nc | bdtc3-f2-s1x, bdtc3-f1-if16 |
| T-N-Ne | bdt3-from2000-t5M-l5-f2, bdte3-f1-if16-log-best, -log-soft (soft) |
| T-N-Le | bdte3-f1-if16-log-best-lrr, -log-soft-lrr (soft) |
| T-N-NNs | bdtc3-f1-if16-switch3, -switch3-soft (soft) |
| T-L-Nc | lrtree-c3-nbr, lrtree-c3-nbr-s22 |
| T-L-Lc | lrtree-c3-lrr (collapsed to one leaf) |
| T-L-Ne | lrtree-e3-nbr (abs error), lrtree-e3-nbr-log-best, -log-best-s22, -log-soft (soft) |
| T-L-Le | lrtree3-from-mix1000-f1 |
| T-L-NNs | lrtree-c3-nbr-switch3 |
| T-L-LLs | lrtree-c3-lrr-switch3 |
| D-N-Nc | bddc3-f2-l10 (no router ever split: in effect a T-N-Nc) |

From scratch (11-06; from-scratch/, standard recipe: jitter self-play, result labels, no search in training, no
forgetting, 1000 rounds; LR family seed 21, NB family seed 11; trees b3 t5M l5 f1). The tournament's representative is
marked *.

| Code | Runs |
|---|---|
| F-N | F-N-s11* |
| F-L | F-L-s21*; F-L-s21-to2000 (continued to v2000) |
| MoE | moe-s21* |
| T-L-Nc | T-L-Nc-s21, T-L-Nc-merge-s21* |
| T-L-Ne | T-L-Ne-s21, -soft-s21, -merge-s21*, -reset-s21, -merge-reset-s21 |
| T-L-Le | T-L-Le-s21, T-L-Le-merge-s21* |
| T-L-NNs, T-L-NLs, T-L-LLs | T-L-<x>-s21, T-L-<x>-merge-s21* |
| T-N-Nc | T-N-Nc-s11, -share-s11, -merge-s11, -share-merge-s11* |
| T-N-Ne | T-N-Ne-s11*, -soft-s11, -merge-s11 |
| T-N-NNs | T-N-NNs-soft-s11, -soft-merge-s11, T-N-NNs-s11 (best-child targets), T-N-NNs-merge-s11* |
| F-FM | F-FM-s21 (not in the tournament) |

Also in from-scratch/: the batch scripts batch29.sh-batch32.sh, the arenas (batch*-arenas.txt, pooled in
*-pooled.txt), make-roster.py, and batch28/ (NB forgetting runs from scratch).
