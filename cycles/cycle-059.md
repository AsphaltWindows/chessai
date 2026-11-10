# Cycle 59

**Own-games step, pooled over the lineages: 12 gates, +4616 -4782 =2602, score 49.3%, Elo -4.8 ± 5.5; kept 8 of 12.**

All cycles so far: 1616 gates, +706783 -633559 =275658, score 52.3%, Elo +15.8 ± 0.5; kept 1069 of 1616.

2026-11-07 18:29: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 20 players to retired.txt (their games stay in the fit), added 20.
Tournament: 5 double round-robins of the 38 active players (7030 games) in 6.4 min, 7030 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| F-L-d4mix-c | 9 of 10 | cycle 60 | 51–59 | 907.5 | 1729.9 | retire |
| T-L-Le-merge-d2jit-c | 9 of 10 | cycle 60 | 51–59 | 796.2 | 1640.4 | retire |
| T-N-Nc-merge-d2jit-c | 9 of 10 | cycle 60 | 51–59 | 580.1 | 1148.5 | retire |
| T-L-Ne-pre-c | 7 of 10 | cycle 62 | 53–59 | 942.5 | 1599.8 | retire |
| T-N-NNs-merge-d2jit-c | 7 of 10 | cycle 62 | 53–59 | 550.7 | 1106.4 | freeze 2 at 1100 |
| T-L-Le-pre-c | 4 of 10 | cycle 65 | 56–59 | 883.1 | 1537.2 | retire |
| T-L-NLs-d2jit-c | 1 of 10 | cycle 68 | 59–59 | 819.5 | 1633.3 | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 900 T-L-Le-fresh-c-v742-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1500 F-P-c-v2808-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d2jit-c | frozen | 3943 |  |  | 805 | 11 |  | 1791 | v1651 | 666 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1362 | 9 |  | 1468 | v1061 | 1223 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1664 | 9 |  | 1712 | v300 | 1526 | frozen |  |  | 373 |
| F-P-c | frozen | 2808 |  |  | 1485 | 17 |  | 1570 | v2138 | 1346 | frozen |  |  | 2808 |
| F-P-tau0-d2jit-c | qualified | 3091 | 151 x 40 | 241 | 1759 | 68 | 1729 | 1792 | v2036 | 1621 | ok (-33 ± 71 vs peak v2036; predecessor +30) | 630 (56385) | +318 -321 =361, 49.9%, -1 ± 17: reverted | 3091 |
| T-L-Ne-pre-c | dormant | 829 |  |  | 1622 | 61 | 1612 | 1614 (1614) | v758 (v758) | 1483 | dormant (+8 ± 41 vs peak v758 before dormancy at v824, at 1475 vs refs; needs > +41) | 630 (55293) | +389 -419 =192, 48.5%, -10 ± 19: reverted | 829 |
| T-L-Nc-fresh-c | qualified | 1780 | 37 x 1000 | 242 | 1584 | 60 | 1548 | 1585 | v1556 | 1446 | ok (-1 ± 61 vs peak v1556; predecessor +37) | 630 (54429) | +441 -385 =174, 52.8%, +20 ± 20: kept | 1781 |
| D-L-Nc-c | qualified | 1456 | 41 x 1000 | 239 | 1594 | 60 | 1578 | 1626 | v1370 | 1455 | ok (-32 ± 63 vs peak v1370; predecessor +16) | 630 (50542) | +468 -369 =163, 54.9%, +34 ± 20: kept | 1457 |
| T-L-LLs-c | frozen | 651 |  |  | 1575 | 8 |  | 1625 | v540 | 1436 | frozen |  |  | 651 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 951 | 8 |  | 1698 | v818 | 813 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | frozen | 742 |  |  | 900 | 14 |  | 1634 | v682 | 761 | frozen |  |  | 742 |
| T-L-Le-pre-c | dormant | 937 |  |  | 1526 | 41 |  | 1558 (1558) | v739 (v739) | 1388 | dormant (-32 ± 32 vs peak v739 before dormancy at v935, at 1419 vs refs; needs > +32) | 630 (55246) | +374 -462 =164, 45.6%, -31 ± 20: reverted | 937 |
| T-N-Ne-c | frozen | 637 |  |  | 1250 | 8 |  | 1364 | v574 | 1111 | frozen |  |  | 637 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 708 | 9 |  | 1340 | v270 | 569 | frozen |  |  | 417 |
| T-N-NNs-merge-d2jit-c | dormant | 3062 |  |  | 1122 | 59 | 1101 | 1203 (1203) | v2345 (v2345) | 983 | dormant (-81 ± 37 vs peak v2345 before dormancy at v3060, at 1064 vs refs; needs > +37) | 630 (56768) | +421 -395 =184, 51.3%, +9 ± 20: kept | 3063 |
| T-L-Le-merge-d2jit-c | dormant | 1593 |  |  | 1634 | 20 |  | 1766 (1766) | v1404 (v1404) | 1496 | dormant (-132 ± 32 vs peak v1404 before dormancy at v1592, at 1628 vs refs; needs > +32) | 630 (54270) | +235 -582 =183, 32.6%, -126 ± 20: reverted | 1593 |
| T-L-NLs-d2jit-c | dormant | 1458 |  |  | 1633 | 61 | 1542 | 1604 (1604) | v1331 (v1331) | 1495 | dormant (+29 ± 41 vs peak v1331 before dormancy at v1457, at 1466 vs refs; needs > +41) | 630 (56849) | +390 -382 =228, 50.4%, +3 ± 19: kept | 1459 |
| T-N-Nc-merge-d2jit-c | dormant | 1440 |  |  | 1226 | 60 | 1187 | 1210 (1195) | v1438 (v1263) | 1088 | dormant (+32 ± 39 vs peak v1263 before dormancy at v1433, at 1056 vs refs; needs > +39) | 630 (56378) | +399 -360 =241, 51.9%, +14 ± 19: kept | 1441 |
| T-L-NLs-merge-d2jit-c | qualified | 1796 | 80 x 40 | 240 | 1633 | 61 | 1654 | 1654 | v1716 | 1495 | ok (-21 ± 63 vs peak v1716; predecessor -21) | 630 (56525) | +360 -311 =329, 52.4%, +17 ± 18: kept | 1797 |
| F-L-d4mix-c | dormant | 323 |  |  | 1723 | 65 | 1731 | 1744 (1738) | v320 (v289) | 1585 | dormant (-14 ± 43 vs peak v289 before dormancy at v319, at 1599 vs refs; needs > +43) | 630 (54863) | +406 -403 =191, 50.1%, +1 ± 19: kept | 324 |
| MoE-d2jit-c | qualified | 1988 | 45 x 40 | 241 | 1657 | 62 | 1678 | 1712 | v1773 | 1519 | ok (-55 ± 65 vs peak v1773; predecessor -21) | 630 (54145) | +415 -393 =192, 51.1%, +8 ± 19: kept | 1989 |
