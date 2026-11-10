# Cycle 54

**Own-games step, pooled over the lineages: 15 gates, +6035 -6014 =2951, score 50.1%, Elo +0.5 ± 5.0; kept 9 of 15.**

All cycles so far: 1549 gates, +680301 -607242 =261457, score 52.4%, Elo +16.4 ± 0.5; kept 1027 of 1549.

2026-11-07 17:04: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 26 players to retired.txt (their games stay in the fit), added 26.
Tournament: 5 double round-robins of the 44 active players (9460 games) in 6.2 min, 9460 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| T-L-Ne-fresh-c | 8 of 10 | cycle 56 | 47–54 | 953.3 | 1584.1 | retire |
| F-P-tau0-c | 6 of 10 | cycle 58 | 49–54 | 1011.6 | 1549.8 | retire |
| T-L-NNs-c | 6 of 10 | cycle 58 | 49–54 | 967.1 | 1592.1 | retire |
| F-L-d4mix-c | 4 of 10 | cycle 60 | 51–54 | 904.0 | 1730.5 | retire |
| T-L-Le-merge-d2jit-c | 4 of 10 | cycle 60 | 51–54 | 803.2 | 1649.3 | retire |
| T-N-Nc-merge-d2jit-c | 4 of 10 | cycle 60 | 51–54 | 571.2 | 1118.8 | retire |
| T-L-Ne-pre-c | 2 of 10 | cycle 62 | 53–54 | 950.1 | 1563.2 | retire |
| T-N-NNs-merge-d2jit-c | 2 of 10 | cycle 62 | 53–54 | 582.7 | 1122.1 | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 900 T-L-Le-fresh-c-v742-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1500 F-P-c-v2808-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d2jit-c | frozen | 3943 |  |  | 811 | 12 |  | 1794 | v3348 | 672 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1365 | 10 |  | 1469 | v1061 | 1226 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1671 | 10 |  | 1713 | v300 | 1532 | frozen |  |  | 373 |
| F-P-c | frozen | 2808 |  |  | 1502 | 22 |  | 1573 | v2138 | 1364 | frozen |  |  | 2808 |
| F-P-tau0-c | dormant | 1963 |  |  | 1560 | 54 | 1571 | 1571 (1557) | v1962 (v862) | 1421 | dormant (+3 ± 36 vs peak v862 before dormancy at v1957, at 1418 vs refs; needs > +36) | 750 (59621) | +384 -403 =213, 49.0%, -7 ± 19: reverted | 1963 |
| F-P-tau0-d2jit-c | qualified | 2336 | 148 x 40 | 240 | 1716 | 60 | 1754 | 1797 | v2036 | 1577 | strike 1 (-81 ± 67 vs peak v2036; predecessor -38) | 750 (66770) | +301 -350 =349, 47.5%, -17 ± 17: reverted | 2336 |
| T-L-Ne-fresh-c | dormant | 1042 |  |  | 1582 | 55 | 1574 | 1593 (1593) | v633 (v633) | 1444 | dormant (-11 ± 37 vs peak v633 before dormancy at v1038, at 1454 vs refs; needs > +37) | 750 (64583) | +401 -430 =169, 48.5%, -10 ± 20: reverted | 1042 |
| T-L-Ne-pre-c | dormant | 826 |  |  | 1575 | 54 | 1549 | 1620 (1620) | v758 (v758) | 1436 | dormant (-45 ± 38 vs peak v758 before dormancy at v824, at 1481 vs refs; needs > +38) | 750 (65999) | +424 -401 =175, 51.1%, +8 ± 20: kept | 827 |
| T-L-Nc-fresh-c | qualified | 1592 | 36 x 1000 | 238 | 1558 | 54 | 1590 | 1590 | v1556 | 1419 | ok (-32 ± 58 vs peak v1556; predecessor -32) | 750 (63925) | +442 -409 =149, 51.6%, +12 ± 20: kept | 1593 |
| D-L-Nc-c | qualified | 1245 | 41 x 1000 | 238 | 1535 | 54 | 1594 | 1594 | v1203 | 1397 | strike 1 (-59 ± 58 vs peak v1203; predecessor -59) | 750 (64685) | +497 -366 =137, 56.5%, +46 ± 20: kept | 1246 |
| T-L-LLs-c | frozen | 651 |  |  | 1579 | 9 |  | 1625 | v540 | 1440 | frozen |  |  | 651 |
| T-L-NNs-c | dormant | 2277 |  |  | 1582 | 55 | 1589 | 1607 (1607) | v1590 (v1590) | 1444 | dormant (-25 ± 36 vs peak v1590 before dormancy at v2274, at 1468 vs refs; needs > +36) | 750 (66177) | +463 -377 =160, 54.3%, +30 ± 20: kept | 2278 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 954 | 8 |  | 1698 | v818 | 815 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | frozen | 742 |  |  | 904 | 16 |  | 1638 | v682 | 766 | frozen |  |  | 742 |
| T-L-Le-pre-c | qualified | 899 | 29 x 1000 | 240 | 1476 | 54 | 1547 | 1564 | v739 | 1337 | strike 1 (-88 ± 55 vs peak v739; predecessor -71) | 750 (64234) | +465 -410 =125, 52.8%, +19 ± 20: kept | 900 |
| T-N-Ne-c | frozen | 637 |  |  | 1254 | 8 |  | 1365 | v574 | 1115 | frozen |  |  | 637 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 710 | 9 |  | 1341 | v270 | 571 | frozen |  |  | 417 |
| T-N-NNs-merge-d2jit-c | dormant | 3060 |  |  | 1122 | 30 |  | 1207 (1207) | v2345 (v2345) | 984 | dormant (-84 ± 27 vs peak v2345 before dormancy at v3060, at 1068 vs refs; needs > +27) | 750 (64166) | +418 -384 =198, 51.7%, +12 ± 19: kept | 3061 |
| T-L-Le-merge-d2jit-c | dormant | 1593 |  |  | 1630 | 28 |  | 1771 (1771) | v1404 (v1404) | 1491 | dormant (-141 ± 34 vs peak v1404 before dormancy at v1592, at 1632 vs refs; needs > +34) | 750 (63766) | +239 -587 =174, 32.6%, -126 ± 21: reverted | 1593 |
| T-L-NLs-d2jit-c | qualified | 1209 | 59 x 40 | 241 | 1535 | 54 | 1489 | 1536 | v1081 | 1397 | ok (-0 ± 56 vs peak v1081; predecessor +46) | 750 (70318) | +419 -355 =226, 53.2%, +22 ± 19: kept | 1210 |
| T-N-Nc-merge-d2jit-c | dormant | 1435 |  |  | 1134 | 53 | 1121 | 1199 (1199) | v1263 (v1263) | 995 | dormant (-65 ± 36 vs peak v1263 before dormancy at v1433, at 1060 vs refs; needs > +36) | 750 (63981) | +413 -376 =211, 51.8%, +13 ± 19: kept | 1436 |
| T-L-NLs-merge-d2jit-c | qualified | 1387 | 93 x 40 | 250 | 1535 | 54 | 1608 | 1629 | v1019 | 1397 | strike 1 (-94 ± 57 vs peak v1019; predecessor -72) | 750 (68465) | +358 -385 =257, 48.6%, -9 ± 19: reverted | 1387 |
| F-L-d4mix-c | dormant | 321 |  |  | 1739 | 62 | 1749 | 1749 (1742) | v320 (v289) | 1601 | dormant (-2 ± 42 vs peak v289 before dormancy at v319, at 1603 vs refs; needs > +42) | 750 (64145) | +396 -416 =188, 49.0%, -7 ± 19: reverted | 321 |
| MoE-d2jit-c | qualified | 1773 | 41 x 40 | 242 | 1718 | 60 | 1643 | 1714 | v1364 | 1580 | new peak (+4 ± 60 over v1364; predecessor +75) | 750 (66803) | +415 -365 =220, 52.5%, +17 ± 19: kept | 1774 |
