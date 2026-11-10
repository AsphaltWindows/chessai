# Cycle 69

**Own-games step, pooled over the lineages: 7 gates, +2716 -2675 =1609, score 50.3%, Elo +2.0 ± 7.1; kept 4 of 7.**

All cycles so far: 1695 gates, +737786 -664012 =293202, score 52.2%, Elo +15.1 ± 0.5; kept 1112 of 1695.

2026-11-07 21:23: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10.
League: retired 10 players to retired.txt (their games stay in the fit), added 10.
Tournament: 5 double round-robins of the 28 active players (3780 games) in 9.3 min, 3780 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| D-L-Nc-c | 8 of 10 | cycle 71 | 62–69 | 967.7 | 1616.5 | retire |
| F-P-tau0-d2jit-c | 5 of 10 | cycle 74 | 65–69 | 846.0 | 1750.7 | retire |
| MoE-d2jit-c | 2 of 10 | cycle 77 | 68–69 | 791.0 | 1687.3 | retire |
| T-N-Nc-merge-d2jit-c | 0 of 10 | cycle 79 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 900 T-L-Le-fresh-c-v742-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1500 F-P-c-v2808-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d2jit-c | frozen | 3943 |  |  | 798 | 10 |  | 1789 | v1651 | 661 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1358 | 8 |  | 1466 | v1061 | 1220 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1657 | 9 |  | 1709 | v300 | 1520 | frozen |  |  | 373 |
| F-P-c | frozen | 2808 |  |  | 1476 | 14 |  | 1564 | v2138 | 1338 | frozen |  |  | 2808 |
| F-P-tau0-d2jit-c | dormant | 3841 |  |  | 1727 | 57 |  | 1786 (1786) | v2036 (v2036) | 1590 | dormant (-59 ± 43 vs peak v2036 before dormancy at v3839, at 1649 vs refs; needs > +43) | 430 (35726) | +326 -339 =335, 49.4%, -4 ± 18: reverted | 3841 |
| T-L-Nc-fresh-c | qualified | 2132 | 34 x 1000 | 238 | 1538 | 71 | 1594 | 1607 | v2061 | 1401 | ok (-69 ± 77 vs peak v2061; predecessor -55) | 430 (36672) | +389 -427 =184, 48.1%, -13 ± 20: reverted | 2132 |
| D-L-Nc-c | dormant | 1545 |  |  | 1616 | 73 | 1616 | 1624 (1620) | v1543 (v1370) | 1479 | dormant (-4 ± 47 vs peak v1370 before dormancy at v1541, at 1483 vs refs; needs > +47) | 430 (36820) | +399 -437 =164, 48.1%, -13 ± 20: reverted | 1545 |
| T-L-LLs-c | frozen | 651 |  |  | 1566 | 8 |  | 1622 | v540 | 1429 | frozen |  |  | 651 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 947 | 7 |  | 1695 | v818 | 810 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | frozen | 742 |  |  | 892 | 12 |  | 1628 | v682 | 755 | frozen |  |  | 742 |
| T-N-Ne-c | frozen | 637 |  |  | 1245 | 7 |  | 1362 | v574 | 1108 | frozen |  |  | 637 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 706 | 8 |  | 1337 | v270 | 568 | frozen |  |  | 417 |
| T-L-NLs-d2jit-c | qualified | 1514 | 52 x 40 | 240 | 1567 | 71 | 1642 | 1642 | v1462 | 1430 | strike 1 (-75 ± 63 vs peak v1462; predecessor -75) | 430 (40535) | +392 -377 =231, 50.7%, +5 ± 19: kept | 1515 |
| T-N-Nc-merge-d2jit-c | dormant | 2105 | 58 x 40 | 239 | 1166 | 71 | 1182 | 1261 | v1441 | 1029 | dormant (strike 2; -95 ± 70 vs peak v1441; predecessor -15) | 430 (37561) | +390 -386 =224, 50.2%, +1 ± 19: kept | 2106 |
| T-L-NLs-merge-d2jit-c | qualified | 2521 | 66 x 40 | 239 | 1541 | 71 | 1654 | 1654 | v2454 | 1404 | strike 1 (-112 ± 77 vs peak v2454; predecessor -112) | 430 (42397) | +403 -353 =244, 52.5%, +17 ± 19: kept | 2522 |
| MoE-d2jit-c | dormant | 2312 |  |  | 1680 | 54 |  | 1760 (1760) | v2156 (v2156) | 1543 | dormant (-80 ± 48 vs peak v2156 before dormancy at v2311, at 1623 vs refs; needs > +48) | 430 (37699) | +417 -356 =227, 53.0%, +21 ± 19: kept | 2313 |
