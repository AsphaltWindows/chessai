# Cycle 70

**Own-games step, pooled over the lineages: 7 gates, +2648 -2696 =1656, score 49.7%, Elo -2.4 ± 7.1; kept 2 of 7.**

All cycles so far: 1702 gates, +740434 -666708 =294858, score 52.2%, Elo +15.1 ± 0.5; kept 1114 of 1702.

2026-11-07 21:41: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10.
League: retired 10 players to retired.txt (their games stay in the fit), added 10.
Tournament: 5 double round-robins of the 28 active players (3780 games) in 10.0 min, 3780 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| D-L-Nc-c | 9 of 10 | cycle 71 | 62–70 | 964.4 | 1616.2 | retire |
| F-P-tau0-d2jit-c | 6 of 10 | cycle 74 | 65–70 | 846.1 | 1750.5 | retire |
| MoE-d2jit-c | 3 of 10 | cycle 77 | 68–70 | 814.1 | 1700.6 | retire |
| T-N-Nc-merge-d2jit-c | 1 of 10 | cycle 79 | 70–70 | 553.9 | 1157.0 | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 900 T-L-Le-fresh-c-v742-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1500 F-P-c-v2808-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d2jit-c | frozen | 3943 |  |  | 798 | 10 |  | 1789 | v1651 | 661 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1357 | 8 |  | 1466 | v1061 | 1220 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1656 | 9 |  | 1709 | v300 | 1519 | frozen |  |  | 373 |
| F-P-c | frozen | 2808 |  |  | 1475 | 14 |  | 1563 | v2138 | 1338 | frozen |  |  | 2808 |
| F-P-tau0-d2jit-c | dormant | 3841 |  |  | 1749 | 48 |  | 1785 (1785) | v2036 (v2036) | 1612 | dormant (-36 ± 40 vs peak v2036 before dormancy at v3839, at 1648 vs refs; needs > +40) | 430 (36450) | +315 -344 =341, 48.5%, -10 ± 18: reverted | 3841 |
| T-L-Nc-fresh-c | qualified | 2163 | 31 x 1000 | 236 | 1576 | 72 | 1540 | 1606 | v2061 | 1439 | ok (-30 ± 78 vs peak v2061; predecessor +37) | 430 (36897) | +385 -419 =196, 48.3%, -12 ± 19: reverted | 2163 |
| D-L-Nc-c | dormant | 1545 |  |  | 1614 | 52 |  | 1623 (1619) | v1543 (v1370) | 1477 | dormant (-5 ± 39 vs peak v1370 before dormancy at v1541, at 1482 vs refs; needs > +39) | 430 (34880) | +416 -415 =169, 50.0%, +0 ± 20: kept | 1546 |
| T-L-LLs-c | frozen | 651 |  |  | 1565 | 8 |  | 1622 | v540 | 1428 | frozen |  |  | 651 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 947 | 7 |  | 1695 | v818 | 810 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | frozen | 742 |  |  | 890 | 12 |  | 1628 | v682 | 753 | frozen |  |  | 742 |
| T-N-Ne-c | frozen | 637 |  |  | 1244 | 7 |  | 1362 | v574 | 1107 | frozen |  |  | 637 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 705 | 8 |  | 1337 | v270 | 568 | frozen |  |  | 417 |
| T-L-NLs-d2jit-c | qualified | 1563 | 48 x 40 | 239 | 1590 | 72 | 1569 | 1642 | v1462 | 1453 | ok (-53 ± 64 vs peak v1462; predecessor +21) | 430 (38326) | +417 -335 =248, 54.1%, +29 ± 19: kept | 1564 |
| T-N-Nc-merge-d2jit-c | dormant | 2106 |  |  | 1157 | 72 | 1166 | 1260 (1260) | v1441 (v1441) | 1020 | dormant (-104 ± 47 vs peak v1441 before dormancy at v2105, at 1124 vs refs; needs > +47) | 430 (37510) | +379 -401 =220, 48.9%, -8 ± 19: reverted | 2106 |
| T-L-NLs-merge-d2jit-c | qualified | 2577 | 55 x 40 | 243 | 1599 | 73 | 1543 | 1656 | v2454 | 1462 | ok (-57 ± 78 vs peak v2454; predecessor +56) | 430 (37298) | +373 -377 =250, 49.8%, -1 ± 19: reverted | 2577 |
| MoE-d2jit-c | dormant | 2313 |  |  | 1727 | 79 | 1683 | 1759 (1759) | v2156 (v2156) | 1590 | dormant (-32 ± 57 vs peak v2156 before dormancy at v2311, at 1622 vs refs; needs > +57) | 430 (36409) | +363 -405 =232, 47.9%, -15 ± 19: reverted | 2313 |
