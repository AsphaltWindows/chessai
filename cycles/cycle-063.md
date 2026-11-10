# Cycle 63

**Own-games step, pooled over the lineages: 8 gates, +3186 -3014 =1800, score 51.1%, Elo +7.5 ± 6.7; kept 5 of 8.**

All cycles so far: 1652 gates, +721121 -647458 =283421, score 52.2%, Elo +15.5 ± 0.5; kept 1089 of 1652.

2026-11-07 19:35: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10.
League: retired 14 players to retired.txt (their games stay in the fit), added 14.
Tournament: 5 double round-robins of the 30 active players (4350 games) in 7.2 min, 4350 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| T-L-Le-pre-c | 8 of 10 | cycle 65 | 56–63 | 886.2 | 1527.1 | retire |
| T-L-NLs-d2jit-c | 5 of 10 | cycle 68 | 59–63 | 847.9 | 1589.0 | retire |
| D-L-Nc-c | 2 of 10 | cycle 71 | 62–63 | 972.4 | 1622.1 | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 900 T-L-Le-fresh-c-v742-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1500 F-P-c-v2808-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d2jit-c | frozen | 3943 |  |  | 805 | 10 |  | 1791 | v1651 | 666 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1360 | 9 |  | 1468 | v1061 | 1222 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1662 | 9 |  | 1712 | v300 | 1523 | frozen |  |  | 373 |
| F-P-c | frozen | 2808 |  |  | 1482 | 15 |  | 1568 | v2138 | 1343 | frozen |  |  | 2808 |
| F-P-tau0-d2jit-c | qualified | 3699 | 154 x 40 | 241 | 1697 | 74 | 1726 | 1791 | v2036 | 1559 | strike 1 (-93 ± 74 vs peak v2036; predecessor -28) | 470 (40215) | +318 -336 =346, 49.1%, -6 ± 17: reverted | 3699 |
| T-L-Nc-fresh-c | qualified | 1926 | 36 x 1000 | 238 | 1535 | 67 | 1546 | 1584 | v1556 | 1397 | ok (-49 ± 66 vs peak v1556; predecessor -11) | 470 (40746) | +405 -404 =191, 50.0%, +0 ± 19: kept | 1927 |
| D-L-Nc-c | dormant | 1543 |  |  | 1648 | 71 | 1596 | 1625 (1625) | v1370 (v1370) | 1510 | dormant (+24 ± 46 vs peak v1370 before dormancy at v1541, at 1486 vs refs; needs > +46) | 470 (39362) | +397 -440 =163, 47.9%, -15 ± 20: reverted | 1543 |
| T-L-LLs-c | frozen | 651 |  |  | 1572 | 8 |  | 1624 | v540 | 1434 | frozen |  |  | 651 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 951 | 8 |  | 1697 | v818 | 813 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | frozen | 742 |  |  | 899 | 13 |  | 1632 | v682 | 761 | frozen |  |  | 742 |
| T-L-Le-pre-c | dormant | 937 |  |  | 1518 | 25 |  | 1556 (1556) | v739 (v739) | 1380 | dormant (-38 ± 28 vs peak v739 before dormancy at v935, at 1418 vs refs; needs > +28) | 470 (40911) | +433 -390 =177, 52.1%, +15 ± 20: kept | 938 |
| T-N-Ne-c | frozen | 637 |  |  | 1248 | 8 |  | 1364 | v574 | 1110 | frozen |  |  | 637 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 709 | 8 |  | 1340 | v270 | 570 | frozen |  |  | 417 |
| T-L-NLs-d2jit-c | dormant | 1461 |  |  | 1573 | 68 | 1588 | 1630 (1603) | v1458 (v1331) | 1435 | dormant (-30 ± 44 vs peak v1331 before dormancy at v1457, at 1465 vs refs; needs > +44) | 470 (42351) | +390 -405 =205, 49.2%, -5 ± 19: reverted | 1461 |
| T-N-Nc-merge-d2jit-c | qualified | 1723 | 78 x 40 | 241 | 1231 | 68 | 1217 | 1266 | v1441 | 1093 | ok (-35 ± 68 vs peak v1441; predecessor +14) | 470 (40952) | +405 -376 =219, 51.4%, +10 ± 19: kept | 1724 |
| T-L-NLs-merge-d2jit-c | qualified | 2105 | 60 x 40 | 238 | 1588 | 68 | 1615 | 1652 | v1716 | 1449 | ok (-64 ± 68 vs peak v1716; predecessor -27) | 470 (41029) | +458 -316 =226, 57.1%, +50 ± 19: kept | 2106 |
| MoE-d2jit-c | qualified | 2156 | 41 x 40 | 240 | 1759 | 79 | 1707 | 1711 | v1773 | 1620 | new peak (+48 ± 76 over v1773; predecessor +52) | 470 (42048) | +380 -347 =273, 51.6%, +12 ± 18: kept | 2157 |
