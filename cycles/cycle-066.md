# Cycle 66

**Own-games step, pooled over the lineages: 7 gates, +2706 -2664 =1630, score 50.3%, Elo +2.1 ± 7.1; kept 2 of 7.**

All cycles so far: 1674 gates, +729649 -655929 =288422, score 52.2%, Elo +15.3 ± 0.5; kept 1101 of 1674.

2026-11-07 20:29: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10.
League: retired 14 players to retired.txt (their games stay in the fit), added 14.
Tournament: 5 double round-robins of the 28 active players (3780 games) in 8.6 min, 3780 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| T-L-NLs-d2jit-c | 8 of 10 | cycle 68 | 59–66 | 850.9 | 1602.1 | retire |
| D-L-Nc-c | 5 of 10 | cycle 71 | 62–66 | 975.4 | 1618.5 | retire |
| F-P-tau0-d2jit-c | 2 of 10 | cycle 74 | 65–66 | 850.8 | 1769.0 | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 900 T-L-Le-fresh-c-v742-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1500 F-P-c-v2808-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d2jit-c | frozen | 3943 |  |  | 801 | 10 |  | 1790 | v1651 | 663 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1359 | 9 |  | 1467 | v1061 | 1221 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1659 | 9 |  | 1711 | v300 | 1521 | frozen |  |  | 373 |
| F-P-c | frozen | 2808 |  |  | 1480 | 14 |  | 1566 | v2138 | 1342 | frozen |  |  | 2808 |
| F-P-tau0-d2jit-c | dormant | 3840 |  |  | 1775 | 86 | 1763 | 1789 (1789) | v2036 (v2036) | 1637 | dormant (-14 ± 54 vs peak v2036 before dormancy at v3839, at 1651 vs refs; needs > +54) | 430 (36315) | +323 -325 =352, 49.9%, -1 ± 17: reverted | 3840 |
| T-L-Nc-fresh-c | qualified | 2030 | 34 x 1000 | 243 | 1543 | 72 | 1589 | 1589 | v1995 | 1405 | ok (-46 ± 76 vs peak v1995; predecessor -46) | 430 (36185) | +383 -438 =179, 47.2%, -19 ± 20: reverted | 2030 |
| D-L-Nc-c | dormant | 1544 |  |  | 1578 | 73 | 1627 | 1627 (1623) | v1543 (v1370) | 1440 | dormant (-44 ± 47 vs peak v1370 before dormancy at v1541, at 1485 vs refs; needs > +47) | 430 (34140) | +425 -429 =146, 49.8%, -1 ± 20: reverted | 1544 |
| T-L-LLs-c | frozen | 651 |  |  | 1570 | 8 |  | 1624 | v540 | 1432 | frozen |  |  | 651 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 950 | 8 |  | 1697 | v818 | 812 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | frozen | 742 |  |  | 894 | 12 |  | 1631 | v682 | 756 | frozen |  |  | 742 |
| T-N-Ne-c | frozen | 637 |  |  | 1247 | 8 |  | 1363 | v574 | 1109 | frozen |  |  | 637 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 707 | 8 |  | 1339 | v270 | 569 | frozen |  |  | 417 |
| T-L-NLs-d2jit-c | dormant | 1462 |  |  | 1632 | 75 | 1634 | 1634 (1601) | v1461 (v1331) | 1494 | dormant (+30 ± 47 vs peak v1331 before dormancy at v1457, at 1463 vs refs; needs > +47) | 430 (38605) | +380 -386 =234, 49.7%, -2 ± 19: reverted | 1462 |
| T-N-Nc-merge-d2jit-c | qualified | 1930 | 67 x 40 | 242 | 1205 | 72 | 1212 | 1264 | v1441 | 1067 | ok (-58 ± 71 vs peak v1441; predecessor -6) | 430 (38187) | +419 -395 =186, 51.2%, +8 ± 20: kept | 1931 |
| T-L-NLs-merge-d2jit-c | qualified | 2279 | 55 x 40 | 242 | 1608 | 74 | 1640 | 1650 | v1716 | 1470 | ok (-42 ± 71 vs peak v1716; predecessor -32) | 430 (39119) | +422 -328 =250, 54.7%, +33 ± 19: kept | 2280 |
| MoE-d2jit-c | qualified | 2271 | 38 x 40 | 243 | 1677 | 77 | 1714 | 1763 | v2156 | 1539 | strike 1 (-86 ± 84 vs peak v2156; predecessor -38) | 430 (34902) | +354 -363 =283, 49.5%, -3 ± 18: reverted | 2271 |
