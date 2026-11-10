# Cycle 64

**Own-games step, pooled over the lineages: 8 gates, +3011 -3242 =1747, score 48.6%, Elo -10.0 ± 6.7; kept 3 of 8.**

All cycles so far: 1660 gates, +724132 -650700 =285168, score 52.2%, Elo +15.4 ± 0.5; kept 1092 of 1660.

2026-11-07 19:53: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10.
League: retired 12 players to retired.txt (their games stay in the fit), added 12.
Tournament: 5 double round-robins of the 30 active players (4350 games) in 8.6 min, 4350 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| T-L-Le-pre-c | 9 of 10 | cycle 65 | 56–64 | 888.4 | 1525.2 | retire |
| T-L-NLs-d2jit-c | 6 of 10 | cycle 68 | 59–64 | 849.5 | 1591.8 | retire |
| D-L-Nc-c | 3 of 10 | cycle 71 | 62–64 | 979.2 | 1628.8 | retire |
| F-P-tau0-d2jit-c | 0 of 10 | cycle 74 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 900 T-L-Le-fresh-c-v742-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1500 F-P-c-v2808-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d2jit-c | frozen | 3943 |  |  | 804 | 10 |  | 1792 | v1651 | 665 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1361 | 9 |  | 1468 | v1061 | 1222 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1662 | 9 |  | 1712 | v300 | 1523 | frozen |  |  | 373 |
| F-P-c | frozen | 2808 |  |  | 1482 | 15 |  | 1569 | v2138 | 1343 | frozen |  |  | 2808 |
| F-P-tau0-d2jit-c | dormant | 3839 | 140 x 40 | 240 | 1713 | 74 | 1702 | 1791 | v2036 | 1574 | dormant (strike 2; -78 ± 75 vs peak v2036; predecessor +11) | 470 (40484) | +335 -342 =323, 49.6%, -2 ± 18: reverted | 3839 |
| T-L-Nc-fresh-c | qualified | 1960 | 33 x 1000 | 238 | 1570 | 68 | 1539 | 1584 | v1556 | 1431 | ok (-14 ± 66 vs peak v1556; predecessor +31) | 470 (38957) | +375 -452 =173, 46.2%, -27 ± 20: reverted | 1960 |
| D-L-Nc-c | dormant | 1543 |  |  | 1642 | 50 |  | 1625 (1625) | v1370 (v1370) | 1503 | dormant (+17 ± 38 vs peak v1370 before dormancy at v1541, at 1486 vs refs; needs > +38) | 470 (40176) | +404 -431 =165, 48.6%, -9 ± 20: reverted | 1543 |
| T-L-LLs-c | frozen | 651 |  |  | 1572 | 8 |  | 1625 | v540 | 1434 | frozen |  |  | 651 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 952 | 8 |  | 1698 | v818 | 813 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | frozen | 742 |  |  | 899 | 13 |  | 1633 | v682 | 760 | frozen |  |  | 742 |
| T-L-Le-pre-c | dormant | 938 |  |  | 1510 | 67 | 1519 | 1557 (1557) | v739 (v739) | 1371 | dormant (-47 ± 42 vs peak v739 before dormancy at v935, at 1418 vs refs; needs > +42) | 470 (41051) | +372 -488 =140, 44.2%, -40 ± 20: reverted | 938 |
| T-N-Ne-c | frozen | 637 |  |  | 1249 | 8 |  | 1364 | v574 | 1110 | frozen |  |  | 637 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 710 | 8 |  | 1340 | v270 | 571 | frozen |  |  | 417 |
| T-L-NLs-d2jit-c | dormant | 1461 |  |  | 1606 | 49 |  | 1630 (1603) | v1458 (v1331) | 1467 | dormant (+3 ± 37 vs peak v1331 before dormancy at v1457, at 1464 vs refs; needs > +37) | 470 (41857) | +351 -419 =230, 46.6%, -24 ± 19: reverted | 1461 |
| T-N-Nc-merge-d2jit-c | qualified | 1795 | 71 x 40 | 243 | 1216 | 68 | 1234 | 1266 | v1441 | 1077 | ok (-50 ± 68 vs peak v1441; predecessor -18) | 470 (38870) | +406 -385 =209, 51.0%, +7 ± 19: kept | 1796 |
| T-L-NLs-merge-d2jit-c | qualified | 2161 | 55 x 40 | 242 | 1593 | 68 | 1592 | 1653 | v1716 | 1454 | ok (-59 ± 68 vs peak v1716; predecessor +2) | 470 (40900) | +376 -345 =279, 51.5%, +11 ± 18: kept | 2162 |
| MoE-d2jit-c | qualified | 2189 | 32 x 40 | 242 | 1670 | 71 | 1763 | 1763 | v2156 | 1531 | strike 1 (-94 ± 80 vs peak v2156; predecessor -94) | 470 (38031) | +392 -380 =228, 50.6%, +4 ± 19: kept | 2190 |
