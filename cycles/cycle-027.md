# Cycle 27

**Own-games step, pooled over the lineages: 30 gates, +13157 -12033 =4810, score 51.9%, Elo +13.0 ± 3.6; kept 19 of 30.**

All cycles so far: 950 gates, +425950 -371011 =153039, score 52.9%, Elo +20.1 ± 0.6; kept 636 of 950.

2026-11-07 06:29: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 58 players to retired.txt (their games stay in the fit), added 58.
Tournament: 5 double round-robins of the 81 active players (32400 games) in 4.5 min, 32400 games saved.
Freeze/retire rule, after 10 cycles dormant: T-L-H-c retired (avg ply 1 983.2, ply 2 1615.3; no untaken milestone within 10); T-L-NLs-merge-c retired (avg ply 1 940.5, ply 2 1611.9; no untaken milestone within 10); T-L-Ne-merge-c retired (avg ply 1 971.7, ply 2 1674.4; no untaken milestone within 10); T-N-LLs-merge-c retired (avg ply 1 623.6, ply 2 1121.3; no untaken milestone within 10); T-N-NNs-c retired (avg ply 1 648.1, ply 2 1135.4; no untaken milestone within 10). 10 entries leave the field for retired.txt (their games stay in the fit): 81 players become 71, 32,400 games a cycle become 24,850.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| T-N-Ne-merge-c | 9 of 10 | cycle 28 | 19–27 | 743.3 | 1399.9 | retire |
| F-L-d2mix-c | 8 of 10 | cycle 29 | 20–27 | 846.9 | 1732.2 | retire |
| T-L-LLs-merge-c | 8 of 10 | cycle 29 | 20–27 | 923.6 | 1561.6 | retire |
| T-L-Nc-merge-c | 8 of 10 | cycle 29 | 20–27 | 962.8 | 1675.3 | retire |
| F-L-d2play-c | 6 of 10 | cycle 31 | 22–27 | 817.8 | 1704.8 | retire |
| F-L-d0soft-c | 5 of 10 | cycle 32 | 23–27 | 984.0 | 1642.6 | retire |
| T-N-Le-merge-c | 4 of 10 | cycle 33 | 24–27 | 726.8 | 1346.2 | retire |
| F-L-c | 2 of 10 | cycle 35 | 26–27 | 983.8 | 1652.6 | retire |
| T-N-NLs-merge-c | 2 of 10 | cycle 35 | 26–27 | 641.4 | 1145.5 | retire |
| F-L-d1play-c | 1 of 10 | cycle 36 | 27–27 | 965.5 | 1508.0 | freeze 2 at 1500 |
| F-N-d2jit-c | 0 of 10 | cycle 37 | - | - | - | retire |
| T-L-Le-c | 0 of 10 | cycle 37 | - | - | - | retire |
| T-L-NLs-c | 0 of 10 | cycle 37 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | dormant | 3140 |  |  | 1673 | 43 | 1624 | 1714 (1714) | v2860 (v2860) | 1531 | dormant (-42 ± 26 vs peak v2860 before dormancy at v3138, at 1572 vs refs; needs > +26) | 1490 (122306) | +449 -429 =122, 51.0%, +7 ± 20: kept | 3141 |
| F-L-d0soft-c | dormant | 2364 |  |  | 1659 | 42 | 1643 | 1699 (1699) | v2014 (v2014) | 1517 | dormant (-40 ± 30 vs peak v2014 before dormancy at v2361, at 1557 vs refs; needs > +30) | 1490 (123031) | +450 -406 =144, 52.2%, +15 ± 20: kept | 2365 |
| F-L-d1play-c | dormant | 3314 |  |  | 1508 | 40 | 1505 | 1598 (1598) | v3006 (v3006) | 1366 | dormant (-90 ± 28 vs peak v3006 before dormancy at v3313, at 1456 vs refs; needs > +28) | 1490 (134274) | +490 -374 =136, 55.8%, +40 ± 20: kept | 3315 |
| F-L-d2play-c | dormant | 1623 |  |  | 1698 | 31 |  | 1753 (1753) | v1011 (v1011) | 1556 | dormant (-55 ± 24 vs peak v1011 before dormancy at v1620, at 1611 vs refs; needs > +24) | 1490 (128634) | +393 -401 =206, 49.6%, -3 ± 19: reverted | 1623 |
| F-L-d2mix-c | dormant | 1075 |  |  | 1742 | 33 |  | 1743 (1741) | v1073 (v910) | 1600 | dormant (+1 ± 29 vs peak v910 before dormancy at v1068, at 1599 vs refs; needs > +29) | 1490 (126708) | +396 -430 =174, 48.3%, -12 ± 20: reverted | 1075 |
| F-L-d2jit-c | qualified | 3463 | 115 x 40 | 240 | 1746 | 46 | 1818 | 1818 | v3348 | 1604 | strike 1 (-72 ± 50 vs peak v3348; predecessor -72) | 1490 (130378) | +334 -427 =239, 45.4%, -32 ± 19: reverted | 3463 |
| F-N-c | frozen | 1691 |  |  | 1393 | 28 |  | 1475 | v1061 | 1251 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1714 | 20 |  | 1719 | v300 | 1571 | frozen |  |  | 373 |
| FM-c | qualified | 579 |  |  | 1630 | 42 | 1543 | 1577 (1565) | v577 (v535) | 1488 | re-qualified (+65 ± 28 vs peak v535 before dormancy at v576, at 1422 vs refs) | 1490 (129914) | +373 -453 =174, 46.0%, -28 ± 20: reverted | 579 |
| F-P-c | qualified | 528 | 175 x 1000 | 240 | 1504 | 40 | 1483 | 1483 | v353 | 1362 | new peak (+21 ± 40 over v353; predecessor +21) | 1490 (131940) | +388 -450 =162, 46.9%, -22 ± 20: reverted | 528 |
| T-L-Ne-c | qualified | 994 | 40 x 1000 | 238 | 1594 | 41 | 1610 | 1610 | v953 | 1451 | ok (-17 ± 42 vs peak v953; predecessor -17) | 1490 (128872) | +447 -392 =161, 52.8%, +19 ± 20: kept | 995 |
| T-L-Ne-merge-c | retired | 1014 |  |  | 1681 | 43 | 1668 | 1704 (1704) | v794 (v794) | 1539 | retired (avg ply 1 971.7, ply 2 1674.4; no untaken milestone within 10); dormant (-23 ± 25 vs peak v794 before dormancy at v1008, at 1562 vs refs; needs > +25) |  |  | 1014 |
| T-L-Ne-fresh-c | qualified | 339 | 49 x 1000 | 242 | 1564 | 40 | 1545 | 1545 | v289 | 1422 | new peak (+20 ± 41 over v289; predecessor +20) | 1490 (128050) | +492 -378 =130, 55.7%, +40 ± 20: kept | 340 |
| T-L-Ne-pre-c | qualified | 167 | 66 x 1000 | 239 | 1414 | 39 | 1374 | 1374 | v100 | 1272 | new peak (+40 ± 40 over v100; predecessor +40) | 1490 (132488) | +500 -341 =159, 58.0%, +56 ± 20: kept | 168 |
| T-L-Nc-merge-c | dormant | 1884 |  |  | 1728 | 45 | 1714 | 1726 (1726) | v1631 (v1631) | 1585 | dormant (+2 ± 33 vs peak v1631 before dormancy at v1878, at 1584 vs refs; needs > +33) | 1490 (121755) | +433 -409 =158, 51.2%, +8 ± 20: kept | 1885 |
| T-L-Nc-fresh-c | qualified | 519 | 45 x 1000 | 242 | 1545 | 40 | 1572 | 1572 | v473 | 1403 | ok (-26 ± 41 vs peak v473; predecessor -26) | 1490 (129818) | +423 -422 =155, 50.0%, +0 ± 20: kept | 520 |
| D-L-Nc-c | qualified | 95 | 41 x 1000 | 239 | 1238 | 38 | 1148 | 1148 | v53 | 1096 | new peak (+90 ± 39 over v53; predecessor +90) | 1490 (138212) | +573 -313 =114, 63.0%, +92 ± 21: kept | 96 |
| T-L-LLs-c | frozen | 651 |  |  | 1609 | 15 |  | 1632 | v540 | 1467 | frozen |  |  | 651 |
| T-L-LLs-merge-c | dormant | 1760 |  |  | 1624 | 41 | 1576 | 1607 (1607) | v1251 (v1251) | 1481 | dormant (+17 ± 30 vs peak v1251 before dormancy at v1754, at 1465 vs refs; needs > +30) | 1490 (132129) | +437 -384 =179, 52.6%, +18 ± 20: kept | 1761 |
| T-L-NLs-c | dormant | 1433 | 44 x 1000 | 238 | 1529 | 40 | 1516 | 1588 | v1345 | 1387 | dormant (strike 2; -59 ± 42 vs peak v1345; predecessor +13) | 1490 (124668) | +335 -504 =161, 41.5%, -59 ± 20: reverted | 1433 |
| T-L-NLs-merge-c | retired | 1122 |  |  | 1654 | 42 | 1658 | 1658 (1654) | v1121 (v641) | 1512 | retired (avg ply 1 940.5, ply 2 1611.9; no untaken milestone within 10); dormant (+0 ± 40 vs peak v641 before dormancy at v1115, at 1512 vs refs; needs > +40) |  |  | 1122 |
| T-L-NNs-c | qualified | 1412 | 46 x 1000 | 242 | 1574 | 40 | 1599 | 1604 | v561 | 1432 | ok (-30 ± 47 vs peak v561; predecessor -26) | 1490 (128460) | +425 -387 =188, 51.9%, +13 ± 19: kept | 1413 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 968 | 15 |  | 1704 | v818 | 825 | frozen |  |  | 1070 |
| T-L-Le-c | dormant | 1080 | 44 x 1000 | 245 | 1519 | 40 | 1530 | 1619 | v990 | 1377 | dormant (strike 2; -100 ± 42 vs peak v990; predecessor -11) | 1490 (125480) | +650 -240 =110, 70.5%, +151 ± 22: kept | 1081 |
| T-L-Le-fresh-c | qualified | 306 | 46 x 1000 | 244 | 1507 | 40 | 1513 | 1513 | v260 | 1365 | ok (-6 ± 41 vs peak v260; predecessor -6) | 1490 (128399) | +524 -369 =107, 57.8%, +54 ± 21: kept | 307 |
| T-L-Le-pre-c | qualified | 161 | 65 x 1000 | 241 | 1420 | 39 | 1350 | 1350 | v95 | 1278 | new peak (+70 ± 40 over v95; predecessor +70) | 1490 (123786) | +561 -329 =110, 61.6%, +82 ± 21: kept | 162 |
| T-N-Ne-c | frozen | 637 |  |  | 1274 | 14 |  | 1371 | v574 | 1132 | frozen |  |  | 637 |
| T-N-Ne-merge-c | dormant | 908 |  |  | 1397 | 39 | 1386 | 1446 (1446) | v699 (v699) | 1255 | dormant (-50 ± 25 vs peak v699 before dormancy at v904, at 1304 vs refs; needs > +25) | 1490 (122379) | +402 -446 =152, 47.8%, -15 ± 20: reverted | 908 |
| T-N-Nc-c | qualified | 934 | 41 x 1000 | 238 | 1385 | 39 | 1340 | 1390 | v472 | 1243 | ok (-5 ± 47 vs peak v472; predecessor +46) | 1490 (122487) | +424 -421 =155, 50.1%, +1 ± 20: kept | 935 |
| T-N-Nc-merge-c | qualified | 1476 | 61 x 1000 | 240 | 1436 | 39 | 1437 | 1449 | v631 | 1294 | ok (-13 ± 37 vs peak v631; predecessor -1) | 1490 (119696) | +402 -428 =170, 48.7%, -9 ± 20: reverted | 1476 |
| T-N-Nc-fresh-c | qualified | 358 | 61 x 1000 | 244 | 1323 | 39 | 1366 | 1366 | v297 | 1181 | strike 1 (-43 ± 40 vs peak v297; predecessor -43) | 1490 (124517) | +426 -421 =153, 50.2%, +2 ± 20: kept | 359 |
| T-N-NNs-c | retired | 632 |  |  | 1164 | 38 | 1140 | 1239 (1239) | v341 (v341) | 1022 | retired (avg ply 1 648.1, ply 2 1135.4; no untaken milestone within 10); dormant (-75 ± 39 vs peak v341 before dormancy at v625, at 1097 vs refs; needs > +39) |  |  | 632 |
| T-N-Le-merge-c | dormant | 754 |  |  | 1346 | 18 |  | 1428 (1428) | v700 (v700) | 1203 | dormant (-82 ± 20 vs peak v700 before dormancy at v754, at 1286 vs refs; needs > +20) | 1490 (123341) | +382 -462 =156, 46.0%, -28 ± 20: reverted | 754 |
| T-N-LLs-merge-c | retired | 725 |  |  | 1112 | 14 |  | 1369 (1369) | v585 (v585) | 970 | retired (avg ply 1 623.6, ply 2 1121.3; no untaken milestone within 10); dormant (-257 ± 21 vs peak v585 before dormancy at v723, at 1227 vs refs; needs > +21) |  |  | 725 |
| T-N-NLs-merge-c | dormant | 1031 |  |  | 1133 | 27 |  | 1417 (1417) | v787 (v787) | 991 | dormant (-284 ± 19 vs peak v787 before dormancy at v1030, at 1275 vs refs; needs > +19) | 1490 (131186) | +430 -416 =154, 50.7%, +5 ± 20: kept | 1032 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 717 | 21 |  | 1347 | v270 | 575 | frozen |  |  | 417 |
| F-N-d2jit-c | dormant | 1783 | 132 x 40 | 241 | 1143 | 38 | 1162 | 1217 | v1523 | 1001 | dormant (strike 2; -74 ± 40 vs peak v1523; predecessor -19) | 1490 (129373) | +415 -362 =223, 52.6%, +18 ± 19: kept | 1784 |
| T-N-NNs-merge-d2jit-c | qualified | 1553 | 81 x 40 | 240 | 1172 | 38 | 1142 | 1169 | v1107 | 1030 | new peak (+4 ± 39 over v1107; predecessor +31) | 1490 (130385) | +406 -385 =209, 51.0%, +7 ± 19: kept | 1554 |
| F-L-d4mix-c | qualified | 133 | 8 x 10 | 229 | 1641 | 42 | 1651 | 1651 | v124 | 1499 | ok (-10 ± 43 vs peak v124; predecessor -10) | 1490 (132298) | +415 -422 =163, 49.6%, -2 ± 20: reverted | 133 |
| MoE-d2jit-c | qualified | 593 | 55 x 40 | 239 | 1694 | 43 | 1626 | 1656 | v486 | 1552 | new peak (+38 ± 45 over v486; predecessor +68) | 1490 (128896) | +382 -432 =186, 47.5%, -17 ± 20: reverted | 593 |
| T-L-H-c | retired | 1526 |  |  | 1626 | 30 |  | 1658 (1658) | v1245 (v1245) | 1484 | retired (avg ply 1 983.2, ply 2 1615.3; no untaken milestone within 10); dormant (-32 ± 27 vs peak v1245 before dormancy at v1521, at 1516 vs refs; needs > +27) |  |  | 1526 |
