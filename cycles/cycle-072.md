# Cycle 72

**Own-games step, pooled over the lineages: 6 gates, +2258 -2342 =1400, score 49.3%, Elo -4.9 ± 7.7; kept 2 of 6.**

All cycles so far: 1714 gates, +745000 -671295 =297705, score 52.2%, Elo +14.9 ± 0.5; kept 1119 of 1714.

2026-11-07 22:13: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10.
League: retired 8 players to retired.txt (their games stay in the fit), added 8.
Tournament: 5 double round-robins of the 26 active players (3250 games) in 7.9 min, 3250 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| F-P-tau0-d2jit-c | 8 of 10 | cycle 74 | 65–72 | 843.2 | 1746.1 | retire |
| MoE-d2jit-c | 5 of 10 | cycle 77 | 68–72 | 826.0 | 1685.6 | retire |
| T-N-Nc-merge-d2jit-c | 3 of 10 | cycle 79 | 70–72 | 555.4 | 1131.4 | retire |
| T-L-NLs-d2jit-c | 0 of 10 | cycle 82 | - | - | - | retire |
| T-L-NLs-merge-d2jit-c | 0 of 10 | cycle 82 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 100 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 900 T-L-Le-fresh-c-v742-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1500 F-P-c-v2808-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d2jit-c | frozen | 3943 |  |  | 794 | 10 |  | 1787 | v1651 | 658 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1354 | 8 |  | 1464 | v1061 | 1219 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1653 | 9 |  | 1707 | v300 | 1517 | frozen |  |  | 373 |
| F-P-c | frozen | 2808 |  |  | 1471 | 13 |  | 1560 | v2138 | 1335 | frozen |  |  | 2808 |
| F-P-tau0-d2jit-c | dormant | 3841 |  |  | 1732 | 37 |  | 1782 (1782) | v2036 (v2036) | 1596 | dormant (-50 ± 38 vs peak v2036 before dormancy at v3839, at 1646 vs refs; needs > +38) | 390 (36118) | +307 -357 =336, 47.5%, -17 ± 18: reverted | 3841 |
| T-L-Nc-fresh-c | qualified | 2232 | 35 x 1000 | 240 | 1459 | 74 | 1569 | 1603 | v2061 | 1323 | strike 1 (-144 ± 79 vs peak v2061; predecessor -110) | 390 (31990) | +415 -419 =166, 49.8%, -1 ± 20: reverted | 2232 |
| T-L-LLs-c | frozen | 651 |  |  | 1561 | 8 |  | 1620 | v540 | 1425 | frozen |  |  | 651 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 944 | 7 |  | 1693 | v818 | 808 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | frozen | 742 |  |  | 886 | 11 |  | 1625 | v682 | 750 | frozen |  |  | 742 |
| T-N-Ne-c | frozen | 637 |  |  | 1241 | 7 |  | 1360 | v574 | 1105 | frozen |  |  | 637 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 702 | 8 |  | 1335 | v270 | 566 | frozen |  |  | 417 |
| T-L-NLs-d2jit-c | dormant | 1682 | 59 x 40 | 239 | 1455 | 74 | 1543 | 1638 | v1462 | 1320 | dormant (strike 2; -182 ± 65 vs peak v1462; predecessor -88) | 390 (36709) | +448 -330 =222, 55.9%, +41 ± 19: kept | 1683 |
| T-N-Nc-merge-d2jit-c | dormant | 2107 |  |  | 1105 | 73 | 1130 | 1257 (1257) | v1441 (v1441) | 969 | dormant (-152 ± 48 vs peak v1441 before dormancy at v2105, at 1121 vs refs; needs > +48) | 390 (36079) | +398 -369 =233, 51.4%, +10 ± 19: kept | 2108 |
| T-L-NLs-merge-d2jit-c | dormant | 2740 | 79 x 40 | 239 | 1552 | 76 | 1549 | 1650 | v2454 | 1416 | dormant (strike 2; -99 ± 80 vs peak v2454; predecessor +2) | 390 (34985) | +298 -466 =236, 41.6%, -59 ± 19: reverted | 2740 |
| MoE-d2jit-c | dormant | 2313 |  |  | 1674 | 46 |  | 1756 (1756) | v2156 (v2156) | 1538 | dormant (-81 ± 46 vs peak v2156 before dormancy at v2311, at 1620 vs refs; needs > +46) | 390 (35324) | +392 -401 =207, 49.5%, -3 ± 19: reverted | 2313 |
