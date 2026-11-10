# Cycle 36

**Own-games step, pooled over the lineages: 24 gates, +10333 -9258 =4409, score 52.2%, Elo +15.6 ± 4.0; kept 18 of 24.**

All cycles so far: 1174 gates, +521873 -461477 =190650, score 52.6%, Elo +17.9 ± 0.6; kept 777 of 1174.

2026-11-07 10:23: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 42 players to retired.txt (their games stay in the fit), added 42.
Tournament: 5 double round-robins of the 59 active players (17110 games) in 6.0 min, 17110 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| T-L-NLs-c | 9 of 10 | cycle 37 | 28–36 | 981.3 | 1533.7 | retire |
| T-L-Le-c | 6 of 10 | cycle 40 | 31–36 | 925.0 | 1558.7 | retire |
| F-L-d2jit-c | 5 of 10 | cycle 41 | 32–36 | 802.4 | 1772.6 | freeze 1 at 800 |
| F-N-d2jit-c | 4 of 10 | cycle 42 | 33–36 | 620.4 | 1214.7 | retire |
| T-N-NNs-merge-d2jit-c | 2 of 10 | cycle 44 | 35–36 | 572.5 | 1131.1 | retire |
| FM-c | 1 of 10 | cycle 45 | 36–36 | 883.9 | 1614.9 | retire |
| T-N-Nc-fresh-c | 0 of 10 | cycle 46 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d1play-c | qualified | 3453 | 131 x 860 | 240 | 1509 | 46 | 1637 | 1637 | v3322 | 1369 | strike 1 (-127 ± 51 vs peak v3322; predecessor -127) | 1050 (96407) | +470 -383 =147, 54.4%, +30 ± 20: kept | 3454 |
| F-L-d2jit-c | dormant | 3942 |  |  | 1765 | 40 |  | 1812 (1812) | v3348 (v3348) | 1625 | dormant (-47 ± 32 vs peak v3348 before dormancy at v3941, at 1671 vs refs; needs > +32) | 1050 (88828) | +387 -381 =232, 50.3%, +2 ± 19: kept | 3943 |
| F-N-c | frozen | 1691 |  |  | 1374 | 14 |  | 1473 | v1061 | 1233 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1701 | 14 |  | 1718 | v300 | 1561 | frozen |  |  | 373 |
| FM-c | dormant | 667 |  |  | 1615 | 48 | 1423 | 1664 (1664) | v626 (v626) | 1475 | dormant (-49 ± 35 vs peak v626 before dormancy at v666, at 1524 vs refs; needs > +35) | 1050 (94364) | +478 -392 =130, 54.3%, +30 ± 20: kept | 668 |
| F-P-c | qualified | 2138 | 167 x 1000 | 240 | 1591 | 48 | 1508 | 1522 | v1087 | 1451 | new peak (+69 ± 49 over v1087; predecessor +83) | 1050 (93250) | +318 -447 =235, 43.5%, -45 ± 19: reverted | 2138 |
| T-L-Ne-c | qualified | 1318 | 30 x 1000 | 242 | 1601 | 48 | 1622 | 1622 | v1288 | 1461 | ok (-20 ± 51 vs peak v1288; predecessor -20) | 1050 (88896) | +388 -445 =167, 47.1%, -20 ± 20: reverted | 1318 |
| T-L-Ne-fresh-c | qualified | 690 | 27 x 1000 | 236 | 1597 | 48 | 1542 | 1611 | v633 | 1456 | ok (-14 ± 51 vs peak v633; predecessor +55) | 1050 (91607) | +449 -399 =152, 52.5%, +17 ± 20: kept | 691 |
| T-L-Ne-pre-c | qualified | 516 | 28 x 1000 | 242 | 1578 | 48 | 1533 | 1563 | v458 | 1437 | new peak (+15 ± 50 over v458; predecessor +45) | 1050 (92744) | +450 -419 =131, 51.5%, +11 ± 20: kept | 517 |
| T-L-Nc-fresh-c | qualified | 890 | 34 x 1000 | 244 | 1588 | 48 | 1561 | 1592 | v822 | 1448 | ok (-4 ± 51 vs peak v822; predecessor +27) | 1050 (88131) | +385 -440 =175, 47.2%, -19 ± 20: reverted | 890 |
| D-L-Nc-c | qualified | 474 | 36 x 1000 | 241 | 1550 | 47 | 1530 | 1555 | v365 | 1410 | ok (-5 ± 50 vs peak v365; predecessor +20) | 1050 (93531) | +460 -401 =139, 52.9%, +20 ± 20: kept | 475 |
| T-L-LLs-c | frozen | 651 |  |  | 1610 | 12 |  | 1630 | v540 | 1469 | frozen |  |  | 651 |
| T-L-NLs-c | dormant | 1433 |  |  | 1518 | 15 |  | 1584 (1584) | v1345 (v1345) | 1377 | dormant (-66 ± 21 vs peak v1345 before dormancy at v1433, at 1444 vs refs; needs > +21) | 1050 (88679) | +327 -541 =132, 39.3%, -76 ± 20: reverted | 1433 |
| T-L-NNs-c | qualified | 1779 | 34 x 1000 | 241 | 1600 | 48 | 1593 | 1625 | v1590 | 1459 | ok (-25 ± 50 vs peak v1590; predecessor +7) | 1050 (93663) | +421 -395 =184, 51.3%, +9 ± 20: kept | 1780 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 965 | 11 |  | 1702 | v818 | 824 | frozen |  |  | 1070 |
| T-L-Le-c | dormant | 1175 |  |  | 1603 | 29 |  | 1676 (1676) | v1081 (v1081) | 1463 | dormant (-72 ± 26 vs peak v1081 before dormancy at v1171, at 1535 vs refs; needs > +26) | 1050 (90440) | +386 -461 =153, 46.2%, -26 ± 20: reverted | 1175 |
| T-L-Le-fresh-c | qualified | 711 | 28 x 1000 | 244 | 1563 | 47 | 1656 | 1656 | v682 | 1423 | strike 1 (-92 ± 51 vs peak v682; predecessor -92) | 1050 (90586) | +579 -306 =115, 63.6%, +97 ± 21: kept | 712 |
| T-L-Le-pre-c | qualified | 500 | 26 x 1000 | 237 | 1505 | 46 | 1498 | 1498 | v473 | 1365 | new peak (+7 ± 49 over v473; predecessor +7) | 1050 (86811) | +440 -439 =121, 50.0%, +0 ± 20: kept | 501 |
| T-N-Ne-c | frozen | 637 |  |  | 1271 | 11 |  | 1369 | v574 | 1130 | frozen |  |  | 637 |
| T-N-Nc-c | qualified | 1278 | 30 x 1000 | 237 | 1378 | 46 | 1373 | 1388 | v472 | 1237 | ok (-11 ± 50 vs peak v472; predecessor +4) | 1050 (88971) | +425 -403 =172, 51.1%, +8 ± 20: kept | 1279 |
| T-N-Nc-merge-c | qualified | 1973 | 45 x 1000 | 239 | 1420 | 46 | 1408 | 1447 | v631 | 1279 | ok (-27 ± 42 vs peak v631; predecessor +12) | 1050 (88535) | +384 -449 =167, 46.8%, -23 ± 20: reverted | 1973 |
| T-N-Nc-fresh-c | dormant | 803 | 36 x 1000 | 240 | 1287 | 45 | 1281 | 1367 | v624 | 1147 | dormant (strike 2; -80 ± 47 vs peak v624; predecessor +6) | 1050 (87931) | +443 -395 =162, 52.4%, +17 ± 20: kept | 804 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 718 | 13 |  | 1345 | v270 | 578 | frozen |  |  | 417 |
| F-N-d2jit-c | dormant | 2050 |  |  | 1209 | 45 | 1217 | 1258 (1258) | v1786 (v1786) | 1069 | dormant (-49 ± 31 vs peak v1786 before dormancy at v2046, at 1118 vs refs; needs > +31) | 1050 (94265) | +406 -379 =215, 51.3%, +9 ± 19: kept | 2051 |
| T-N-NNs-merge-d2jit-c | dormant | 2113 |  |  | 1136 | 44 | 1126 | 1168 (1168) | v1553 (v1553) | 996 | dormant (-32 ± 29 vs peak v1553 before dormancy at v2112, at 1028 vs refs; needs > +29) | 1050 (92265) | +403 -390 =207, 50.6%, +4 ± 19: kept | 2114 |
| T-L-Le-merge-d2jit-c | qualified | 199 | 102 x 40 | 260 | 1227 | 45 | 774 | 774 | v96 | 1087 | new peak (+453 ± 47 over v96; predecessor +453) | 1050 (108577) | +546 -198 =256, 67.4%, +126 ± 19: kept | 200 |
| T-L-NLs-d2jit-c | qualified | 148 | 64 x 40 | 240 | 950 | 43 | 703 | 703 | v83 | 810 | new peak (+248 ± 47 over v83; predecessor +248) | 1050 (114710) | +343 -180 =477, 58.1%, +57 ± 16: kept | 149 |
| T-N-Nc-merge-d2jit-c | qualified | 223 | 106 x 40 | 244 | 1070 | 43 | 682 | 682 | v116 | 930 | new peak (+388 ± 47 over v116; predecessor +388) | 1050 (106286) | +578 -235 =187, 67.2%, +124 ± 20: kept | 224 |
| F-L-d4mix-c | qualified | 208 | 7 x 10 | 226 | 1750 | 55 | 1752 | 1752 | v200 | 1609 | ok (-3 ± 58 vs peak v200; predecessor -3) | 1050 (91005) | +435 -398 =167, 51.8%, +13 ± 20: kept | 209 |
| MoE-d2jit-c | qualified | 991 | 35 x 40 | 238 | 1709 | 52 | 1683 | 1723 | v882 | 1569 | ok (-14 ± 56 vs peak v882; predecessor +26) | 1050 (94039) | +432 -382 =186, 52.5%, +17 ± 20: kept | 992 |
