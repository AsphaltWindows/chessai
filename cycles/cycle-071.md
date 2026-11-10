# Cycle 71

**Own-games step, pooled over the lineages: 6 gates, +2308 -2245 =1447, score 50.5%, Elo +3.6 ± 7.7; kept 3 of 6.**

All cycles so far: 1708 gates, +742742 -668953 =296305, score 52.2%, Elo +15.0 ± 0.5; kept 1117 of 1708.

2026-11-07 21:57: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10.
League: retired 8 players to retired.txt (their games stay in the fit), added 8.
Tournament: 5 double round-robins of the 28 active players (3780 games) in 7.9 min, 3780 games saved.
Freeze/retire rule, after 10 cycles dormant: D-L-Nc-c retired (avg ply 1 964.0, ply 2 1619.0; no untaken milestone within 10). 2 entries leave the field for retired.txt (their games stay in the fit): 28 players become 26, 3,780 games a cycle become 3,250.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| F-P-tau0-d2jit-c | 7 of 10 | cycle 74 | 65–71 | 844.8 | 1748.2 | retire |
| MoE-d2jit-c | 4 of 10 | cycle 77 | 68–71 | 823.1 | 1688.4 | retire |
| T-N-Nc-merge-d2jit-c | 2 of 10 | cycle 79 | 70–71 | 557.0 | 1144.7 | retire |

Milestones taken (multiple: takers): 0 random, 100 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 900 T-L-Le-fresh-c-v742-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1500 F-P-c-v2808-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d2jit-c | frozen | 3943 |  |  | 796 | 10 |  | 1788 | v1651 | 660 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1356 | 8 |  | 1464 | v1061 | 1220 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1654 | 9 |  | 1708 | v300 | 1518 | frozen |  |  | 373 |
| F-P-c | frozen | 2808 |  |  | 1473 | 13 |  | 1562 | v2138 | 1336 | frozen |  |  | 2808 |
| F-P-tau0-d2jit-c | dormant | 3841 |  |  | 1734 | 41 |  | 1783 (1783) | v2036 (v2036) | 1598 | dormant (-49 ± 38 vs peak v2036 before dormancy at v3839, at 1647 vs refs; needs > +38) | 430 (37908) | +320 -337 =343, 49.1%, -6 ± 18: reverted | 3841 |
| T-L-Nc-fresh-c | qualified | 2197 | 34 x 1000 | 239 | 1567 | 71 | 1562 | 1604 | v2061 | 1431 | ok (-37 ± 77 vs peak v2061; predecessor +5) | 430 (36462) | +363 -449 =188, 45.7%, -30 ± 20: reverted | 2197 |
| D-L-Nc-c | retired | 1546 |  |  | 1644 | 74 | 1604 | 1621 (1617) | v1543 (v1370) | 1508 | retired (avg ply 1 964.0, ply 2 1619.0; no untaken milestone within 10); dormant (+27 ± 47 vs peak v1370 before dormancy at v1541, at 1481 vs refs; needs > +47) |  |  | 1546 |
| T-L-LLs-c | frozen | 651 |  |  | 1562 | 8 |  | 1621 | v540 | 1426 | frozen |  |  | 651 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 945 | 7 |  | 1694 | v818 | 809 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | frozen | 742 |  |  | 888 | 12 |  | 1626 | v682 | 752 | frozen |  |  | 742 |
| T-N-Ne-c | frozen | 637 |  |  | 1242 | 7 |  | 1360 | v574 | 1106 | frozen |  |  | 637 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 703 | 8 |  | 1336 | v270 | 567 | frozen |  |  | 417 |
| T-L-NLs-d2jit-c | qualified | 1622 | 58 x 40 | 239 | 1542 | 71 | 1575 | 1639 | v1462 | 1405 | strike 1 (-98 ± 63 vs peak v1462; predecessor -34) | 430 (41650) | +392 -330 =278, 53.1%, +22 ± 18: kept | 1623 |
| T-N-Nc-merge-d2jit-c | dormant | 2106 |  |  | 1132 | 50 |  | 1259 (1259) | v1441 (v1441) | 996 | dormant (-126 ± 39 vs peak v1441 before dormancy at v2105, at 1122 vs refs; needs > +39) | 430 (38679) | +407 -379 =214, 51.4%, +10 ± 19: kept | 2107 |
| T-L-NLs-merge-d2jit-c | qualified | 2660 | 83 x 40 | 245 | 1548 | 71 | 1585 | 1652 | v2454 | 1412 | strike 1 (-104 ± 77 vs peak v2454; predecessor -37) | 430 (41280) | +446 -352 =202, 54.7%, +33 ± 19: kept | 2661 |
| MoE-d2jit-c | dormant | 2313 |  |  | 1652 | 53 |  | 1757 (1757) | v2156 (v2156) | 1515 | dormant (-106 ± 48 vs peak v2156 before dormancy at v2311, at 1621 vs refs; needs > +48) | 430 (37213) | +380 -398 =222, 49.1%, -6 ± 19: reverted | 2313 |
