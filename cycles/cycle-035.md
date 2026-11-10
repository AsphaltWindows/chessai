# Cycle 35

**Changes from this cycle** (deployed after cycle 34, `staged-c34/`), the user's: three depth-2 trees join, from scratch on `chessai-cyc5`, each its twin's line plus `--depth 2` with its split threshold scaled to its data as for T-N-NNs-merge-d2jit-c (5,000,000 x positions a cycle at depth 2 / the twin's at depth 0): T-L-Le-merge-d2jit-c (T-L-Le-merge-c's line, 100,000), T-L-NLs-d2jit-c (T-L-NLs-c's, 100,000) and T-N-Nc-merge-d2jit-c (T-N-Nc-merge-c's, 200,000). The question is whether look-ahead helps trees with LR leaves and not those with CNBP leaves: at ply 2 F-L-d2jit-c peaked above F-L-c, while F-N-d2jit-c and T-N-NNs-merge-d2jit-c peaked below their twins. Checked before the deploy, from scratch at depth 2 on `chessai-cyc5` in rounds of 40 games: each line ran, and each split within about its first cycle's data. The two T-L trees, the same single LR leaf until then, first split at v18 after 204,781 positions, their second split check (the first never splits), and T-N-Nc-merge-d2jit-c at v57 after 606,071, its third (its run played 1.28M positions in 122 rounds for 253 CPU-s, about one cycle's budget); the merge trees merged back to one leaf (T-L-Le-merge-d2jit-c at v28, T-N-Nc-merge-d2jit-c at v76) and T-L-NLs-d2jit-c kept its 3 leaves through v40. At a 20k split threshold, 4 rounds and a resume to 8 gave the same files as 8 rounds, byte for byte, across a split (and a merge for the merge trees). Each one's own step through the driver, train and gate, ran from its check run's last version.

**Own-games step, pooled over the lineages: 24 gates, +10589 -8906 =4505, score 53.5%, Elo +24.4 ± 4.0; kept 17 of 24.**

All cycles so far: 1150 gates, +511540 -452219 =186241, score 52.6%, Elo +17.9 ± 0.6; kept 759 of 1150.

2026-11-07 09:53: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 40 players to retired.txt (their games stay in the fit), added 46.
Tournament: 5 double round-robins of the 63 active players (19530 games) in 6.3 min, 19530 games saved.
Freeze/retire rule, after 10 cycles dormant: F-L-c retired (avg ply 1 979.9, ply 2 1684.2; no untaken milestone within 10); T-N-NLs-merge-c retired (avg ply 1 629.3, ply 2 1130.7; no untaken milestone within 10). 4 entries leave the field for retired.txt (their games stay in the fit): 63 players become 59, 19,530 games a cycle become 17,110.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| T-L-NLs-c | 8 of 10 | cycle 37 | 28–35 | 981.6 | 1535.7 | retire |
| T-L-Le-c | 5 of 10 | cycle 40 | 31–35 | 924.4 | 1549.8 | retire |
| F-L-d2jit-c | 4 of 10 | cycle 41 | 32–35 | 798.8 | 1774.5 | freeze 1 at 800 |
| F-N-d2jit-c | 3 of 10 | cycle 42 | 33–35 | 609.5 | 1216.6 | freeze 1 at 600 |
| T-N-NNs-merge-d2jit-c | 1 of 10 | cycle 44 | 35–35 | 583.7 | 1125.8 | retire |
| FM-c | 0 of 10 | cycle 45 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | retired | 3146 |  |  | 1690 | 51 | 1699 | 1711 (1711) | v2860 (v2860) | 1550 | retired (avg ply 1 979.9, ply 2 1684.2; no untaken milestone within 10); dormant (-21 ± 30 vs peak v2860 before dormancy at v3138, at 1571 vs refs; needs > +30) |  |  | 3146 |
| F-L-d1play-c | qualified | 3322 |  |  | 1634 | 49 | 1522 | 1594 (1594) | v3006 (v3006) | 1494 | re-qualified (+40 ± 31 vs peak v3006 before dormancy at v3313, at 1454 vs refs) | 1130 (95793) | +410 -455 =135, 47.8%, -16 ± 20: reverted | 3322 |
| F-L-d2jit-c | dormant | 3942 |  |  | 1742 | 54 | 1794 | 1812 (1812) | v3348 (v3348) | 1601 | dormant (-70 ± 37 vs peak v3348 before dormancy at v3941, at 1672 vs refs; needs > +37) | 1130 (99031) | +351 -429 =220, 46.1%, -27 ± 19: reverted | 3942 |
| F-N-c | frozen | 1691 |  |  | 1373 | 15 |  | 1474 | v1061 | 1233 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1702 | 14 |  | 1718 | v300 | 1561 | frozen |  |  | 373 |
| FM-c | dormant | 666 | 19 x 1000 | 237 | 1422 | 46 | 1419 | 1665 | v626 | 1281 | dormant (strike 2; -244 ± 51 vs peak v626; predecessor +3) | 1130 (93776) | +615 -249 =136, 68.3%, +133 ± 21: kept | 667 |
| F-P-c | qualified | 1970 | 161 x 1000 | 240 | 1506 | 46 | 1489 | 1524 | v1087 | 1366 | ok (-17 ± 48 vs peak v1087; predecessor +18) | 1130 (103320) | +389 -384 =227, 50.2%, +2 ± 19: kept | 1971 |
| T-L-Ne-c | qualified | 1288 | 29 x 1000 | 237 | 1619 | 48 | 1613 | 1620 | v1155 | 1479 | ok (-1 ± 50 vs peak v1155; predecessor +7) | 1130 (94843) | +421 -432 =147, 49.5%, -4 ± 20: reverted | 1288 |
| T-L-Ne-fresh-c | qualified | 662 | 28 x 1000 | 244 | 1540 | 47 | 1611 | 1611 | v633 | 1399 | strike 1 (-72 ± 51 vs peak v633; predecessor -72) | 1130 (94733) | +447 -402 =151, 52.2%, +16 ± 20: kept | 663 |
| T-L-Ne-pre-c | qualified | 487 | 28 x 1000 | 236 | 1531 | 47 | 1563 | 1563 | v458 | 1391 | ok (-32 ± 50 vs peak v458; predecessor -32) | 1130 (98146) | +432 -406 =162, 51.3%, +9 ± 20: kept | 488 |
| T-L-Nc-fresh-c | qualified | 856 | 33 x 1000 | 237 | 1559 | 47 | 1592 | 1592 | v701 | 1419 | ok (-33 ± 49 vs peak v701; predecessor -32) | 1130 (97747) | +374 -447 =179, 46.4%, -25 ± 20: reverted | 856 |
| D-L-Nc-c | qualified | 437 | 36 x 1000 | 244 | 1528 | 47 | 1484 | 1556 | v365 | 1388 | ok (-28 ± 50 vs peak v365; predecessor +44) | 1130 (100302) | +517 -342 =141, 58.8%, +61 ± 20: kept | 438 |
| T-L-LLs-c | frozen | 651 |  |  | 1609 | 12 |  | 1630 | v540 | 1468 | frozen |  |  | 651 |
| T-L-NLs-c | dormant | 1433 |  |  | 1522 | 16 |  | 1585 (1585) | v1345 (v1345) | 1381 | dormant (-63 ± 21 vs peak v1345 before dormancy at v1433, at 1444 vs refs; needs > +21) | 1130 (96841) | +315 -540 =145, 38.8%, -80 ± 20: reverted | 1433 |
| T-L-NNs-c | qualified | 1745 | 35 x 1000 | 244 | 1591 | 48 | 1619 | 1626 | v1590 | 1450 | ok (-35 ± 50 vs peak v1590; predecessor -28) | 1130 (96671) | +401 -418 =181, 49.1%, -6 ± 20: reverted | 1745 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 967 | 12 |  | 1703 | v818 | 826 | frozen |  |  | 1070 |
| T-L-Le-c | dormant | 1175 |  |  | 1594 | 34 |  | 1677 (1677) | v1081 (v1081) | 1454 | dormant (-83 ± 28 vs peak v1081 before dormancy at v1171, at 1536 vs refs; needs > +28) | 1130 (94829) | +374 -470 =156, 45.2%, -34 ± 20: reverted | 1175 |
| T-L-Le-fresh-c | qualified | 682 |  |  | 1653 | 50 | 1602 | 1608 (1608) | v594 (v594) | 1512 | re-qualified (+45 ± 34 vs peak v594 before dormancy at v680, at 1468 vs refs) | 1130 (97632) | +463 -403 =134, 53.0%, +21 ± 20: kept | 683 |
| T-L-Le-pre-c | qualified | 473 | 33 x 1000 | 238 | 1497 | 46 | 1474 | 1499 | v299 | 1356 | ok (-2 ± 48 vs peak v299; predecessor +23) | 1130 (96424) | +449 -410 =141, 51.9%, +14 ± 20: kept | 474 |
| T-N-Ne-c | frozen | 637 |  |  | 1273 | 11 |  | 1370 | v574 | 1132 | frozen |  |  | 637 |
| T-N-Nc-c | qualified | 1247 | 32 x 1000 | 239 | 1372 | 46 | 1350 | 1389 | v472 | 1231 | ok (-17 ± 50 vs peak v472; predecessor +22) | 1130 (96810) | +435 -414 =151, 51.0%, +7 ± 20: kept | 1248 |
| T-N-Nc-merge-c | qualified | 1927 | 45 x 1000 | 246 | 1407 | 46 | 1401 | 1447 | v631 | 1266 | ok (-40 ± 41 vs peak v631; predecessor +6) | 1130 (92682) | +445 -386 =169, 52.9%, +20 ± 20: kept | 1928 |
| T-N-Nc-fresh-c | qualified | 766 | 45 x 1000 | 241 | 1281 | 45 | 1336 | 1368 | v624 | 1140 | strike 1 (-88 ± 47 vs peak v624; predecessor -56) | 1130 (93263) | +434 -415 =151, 50.9%, +7 ± 20: kept | 767 |
| T-N-NLs-merge-c | retired | 1038 |  |  | 1096 | 43 | 1092 | 1415 (1415) | v787 (v787) | 955 | retired (avg ply 1 629.3, ply 2 1130.7; no untaken milestone within 10); dormant (-319 ± 26 vs peak v787 before dormancy at v1030, at 1274 vs refs; needs > +26) |  |  | 1038 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 718 | 14 |  | 1346 | v270 | 577 | frozen |  |  | 417 |
| F-N-d2jit-c | dormant | 2049 |  |  | 1216 | 44 | 1209 | 1259 (1259) | v1786 (v1786) | 1076 | dormant (-43 ± 31 vs peak v1786 before dormancy at v2046, at 1119 vs refs; needs > +31) | 1130 (102226) | +410 -391 =199, 50.9%, +7 ± 19: kept | 2050 |
| T-N-NNs-merge-d2jit-c | dormant | 2112 |  |  | 1126 | 32 |  | 1170 (1170) | v1553 (v1553) | 985 | dormant (-44 ± 24 vs peak v1553 before dormancy at v2112, at 1029 vs refs; needs > +24) | 1130 (101912) | +402 -398 =200, 50.2%, +1 ± 19: kept | 2113 |
| T-L-Le-merge-d2jit-c | qualified | 96 | 96 x 40 | 240 | 774 | 44 |  |  |  | 634 | qualified (first rating) | 1130 (114510) | +748 -64 =188, 84.2%, +291 ± 24: kept | 97 |
| T-L-NLs-d2jit-c | qualified | 83 | 83 x 40 | 239 | 703 | 45 |  |  |  | 562 | qualified (first rating) | 1130 (115353) | +353 -152 =495, 60.1%, +71 ± 15: kept | 84 |
| T-N-Nc-merge-d2jit-c | qualified | 116 | 116 x 40 | 241 | 683 | 46 |  |  |  | 542 | qualified (first rating) | 1130 (108868) | +557 -122 =321, 71.8%, +162 ± 19: kept | 117 |
| F-L-d4mix-c | qualified | 200 | 7 x 10 | 223 | 1750 | 55 | 1691 | 1715 | v141 | 1609 | new peak (+35 ± 54 over v141; predecessor +59) | 1130 (95873) | +434 -373 =193, 53.0%, +21 ± 19: kept | 201 |
| MoE-d2jit-c | qualified | 955 | 36 x 40 | 243 | 1680 | 51 | 1662 | 1724 | v882 | 1540 | ok (-44 ± 55 vs peak v882; predecessor +18) | 1130 (102572) | +413 -404 =183, 50.4%, +3 ± 20: kept | 956 |
