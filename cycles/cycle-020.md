# Cycle 20

**Own-games step, pooled over the lineages: 41 gates, +17768 -16373 =6859, score 51.7%, Elo +11.8 ± 3.1; kept 25 of 41.**

All cycles so far: 700 gates, +317312 -270300 =112388, score 53.4%, Elo +23.4 ± 0.7; kept 480 of 700.

2026-11-07 03:22: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 58 players to retired.txt (their games stay in the fit), added 58.
Tournament: 5 double round-robins of the 87 active players (37410 games) in 3.8 min, 37410 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| T-L-NNs-merge-c | 9 of 10 | cycle 21 | 12–20 | 993.5 | 1691.0 | freeze 1 at 1000 |
| T-N-Ne-c | 9 of 10 | cycle 21 | 12–20 | 735.7 | 1299.1 | freeze 2 at 1300 |
| T-N-H-c | 8 of 10 | cycle 22 | 13–20 | 749.2 | 1289.8 | retire |
| T-N-NNs-merge-c | 8 of 10 | cycle 22 | 13–20 | 747.7 | 1375.8 | retire |
| F-L-d1jit-c | 6 of 10 | cycle 24 | 15–20 | 1029.8 | 1636.8 | retire |
| MoE-c | 6 of 10 | cycle 24 | 15–20 | 963.7 | 1688.2 | retire |
| T-L-LLs-c | 6 of 10 | cycle 24 | 15–20 | 954.8 | 1605.8 | freeze 2 at 1600 |
| T-N-LNs-merge-c | 6 of 10 | cycle 24 | 15–20 | 689.7 | 1196.8 | freeze 2 at 1200 |
| T-N-Lc-merge-c | 6 of 10 | cycle 24 | 15–20 | 755.2 | 1417.2 | retire |
| TAN-c | 6 of 10 | cycle 24 | 15–20 | 727.9 | 1218.7 | retire |
| F-L-c | 4 of 10 | cycle 26 | 17–20 | 1004.6 | 1674.0 | retire |
| F-N-c | 4 of 10 | cycle 26 | 17–20 | 733.8 | 1395.9 | freeze 2 at 1400 |
| T-L-Le-merge-c | 4 of 10 | cycle 26 | 17–20 | 978.5 | 1685.0 | retire |
| T-L-Nc-c | 4 of 10 | cycle 26 | 17–20 | 993.5 | 1589.5 | retire |
| F-L-d1play-c | 3 of 10 | cycle 27 | 18–20 | 1002.6 | 1523.0 | retire |
| T-L-H-c | 3 of 10 | cycle 27 | 18–20 | 998.7 | 1599.6 | retire |
| T-L-NLs-merge-c | 3 of 10 | cycle 27 | 18–20 | 937.0 | 1574.4 | retire |
| T-L-Ne-merge-c | 3 of 10 | cycle 27 | 18–20 | 974.6 | 1668.8 | retire |
| T-N-LLs-merge-c | 3 of 10 | cycle 27 | 18–20 | 632.5 | 1119.3 | retire |
| T-N-NNs-c | 3 of 10 | cycle 27 | 18–20 | 665.6 | 1138.6 | retire |
| T-N-Le-merge-c | 2 of 10 | cycle 28 | 19–20 | 751.5 | 1394.2 | retire |
| T-N-NLs-merge-c | 2 of 10 | cycle 28 | 19–20 | 772.4 | 1391.1 | retire |
| T-N-Ne-merge-c | 2 of 10 | cycle 28 | 19–20 | 750.4 | 1387.3 | retire |
| F-L-d2mix-c | 1 of 10 | cycle 29 | 20–20 | 814.4 | 1735.5 | retire |
| T-L-LLs-merge-c | 1 of 10 | cycle 29 | 20–20 | 938.2 | 1539.2 | retire |
| T-L-Le-c | 1 of 10 | cycle 29 | 20–20 | 943.9 | 1541.6 | retire |
| T-L-Nc-merge-c | 1 of 10 | cycle 29 | 20–20 | 989.9 | 1616.9 | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4.


Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | dormant | 2858 |  |  | 1666 | 43 | 1663 | 1701 (1695) | v2855 (v2247) | 1519 | dormant (-29 ± 31 vs peak v2247 before dormancy at v2854, at 1549 vs refs; needs > +31) | 1610 (131858) | +430 -406 =164, 51.2%, +8 ± 20: kept | 2859 |
| F-L-d0soft-c | qualified | 2014 |  |  | 1709 | 45 | 1638 | 1663 (1656) | v2012 (v1619) | 1562 | re-qualified (+52 ± 33 vs peak v1619 before dormancy at v2006, at 1510 vs refs) | 1610 (130806) | +443 -409 =148, 51.7%, +12 ± 20: kept | 2015 |
| F-L-d1jit-c | dormant | 1768 |  |  | 1668 | 43 | 1635 | 1704 (1704) | v1451 (v1451) | 1522 | dormant (-36 ± 31 vs peak v1451 before dormancy at v1763, at 1557 vs refs; needs > +31) | 1610 (132723) | +399 -445 =156, 47.7%, -16 ± 20: reverted | 1768 |
| F-L-d1play-c | dormant | 3002 |  |  | 1561 | 40 | 1524 | 1566 (1566) | v1269 (v1269) | 1414 | dormant (-5 ± 32 vs peak v1269 before dormancy at v2999, at 1420 vs refs; needs > +32) | 1610 (143629) | +453 -406 =141, 52.3%, +16 ± 20: kept | 3003 |
| F-L-d2play-c | qualified | 1514 | 100 x 40 | 241 | 1700 | 44 | 1719 | 1760 | v1011 | 1553 | strike 1 (-60 ± 43 vs peak v1011; predecessor -19) | 1610 (136228) | +386 -380 =234, 50.3%, +2 ± 19: kept | 1515 |
| F-L-d2mix-c | dormant | 1069 |  |  | 1736 | 46 | 1674 | 1752 (1752) | v910 (v910) | 1589 | dormant (-16 ± 33 vs peak v910 before dormancy at v1068, at 1605 vs refs; needs > +33) | 1610 (136916) | +383 -349 =268, 51.7%, +12 ± 18: kept | 1070 |
| F-L-d2jit-c | qualified | 2676 | 110 x 40 | 241 | 1732 | 46 | 1750 | 1806 | v1651 | 1585 | strike 1 (-74 ± 52 vs peak v1651; predecessor -19) | 1610 (145477) | +391 -384 =225, 50.3%, +2 ± 19: kept | 2677 |
| F-N-c | dormant | 1687 |  |  | 1386 | 38 | 1407 | 1483 (1483) | v1061 (v1061) | 1239 | dormant (-97 ± 28 vs peak v1061 before dormancy at v1683, at 1336 vs refs; needs > +28) | 1610 (135610) | +414 -409 =177, 50.2%, +2 ± 20: kept | 1688 |
| MoE-c | dormant | 372 |  |  | 1649 | 42 | 1690 | 1727 (1727) | v300 (v300) | 1503 | dormant (-78 ± 31 vs peak v300 before dormancy at v367, at 1581 vs refs; needs > +31) | 1610 (133220) | +413 -457 =130, 47.8%, -15 ± 20: reverted | 372 |
| FM-c | qualified | 510 | 24 x 1000 | 240 | 1386 | 38 | 1414 | 1448 | v419 | 1239 | strike 1 (-63 ± 40 vs peak v419; predecessor -29) | 1610 (141340) | +533 -330 =137, 60.2%, +72 ± 20: kept | 511 |
| TAN-c | dormant | 929 |  |  | 1218 | 17 |  | 1246 (1246) | v507 (v507) | 1071 | dormant (-28 ± 22 vs peak v507 before dormancy at v928, at 1099 vs refs; needs > +22) | 1610 (137962) | +417 -465 =118, 47.6%, -17 ± 20: reverted | 929 |
| T-L-Ne-c | qualified | 735 | 38 x 1000 | 240 | 1589 | 41 | 1608 | 1616 | v556 | 1442 | ok (-26 ± 36 vs peak v556; predecessor -19) | 1610 (136941) | +435 -426 =139, 50.4%, +3 ± 20: kept | 736 |
| T-L-Ne-merge-c | dormant | 1011 |  |  | 1665 | 43 | 1676 | 1712 (1712) | v794 (v794) | 1518 | dormant (-47 ± 25 vs peak v794 before dormancy at v1008, at 1566 vs refs; needs > +25) | 1610 (131548) | +441 -394 =165, 52.3%, +16 ± 20: kept | 1012 |
| T-L-Nc-c | dormant | 713 |  |  | 1605 | 41 | 1569 | 1654 (1654) | v516 (v516) | 1459 | dormant (-49 ± 30 vs peak v516 before dormancy at v709, at 1508 vs refs; needs > +30) | 1610 (143616) | +437 -406 =157, 51.5%, +11 ± 20: kept | 714 |
| T-L-Nc-merge-c | dormant | 1879 |  |  | 1617 | 41 | 1590 | 1736 (1736) | v1631 (v1631) | 1470 | dormant (-119 ± 31 vs peak v1631 before dormancy at v1878, at 1590 vs refs; needs > +31) | 1610 (129459) | +487 -377 =136, 55.5%, +38 ± 20: kept | 1880 |
| T-L-Nc-fresh-c | qualified | 197 | 73 x 1000 | 240 | 1458 | 38 | 1375 | 1375 | v123 | 1312 | new peak (+83 ± 39 over v123; predecessor +83) | 1610 (139453) | +557 -317 =126, 62.0%, +85 ± 21: kept | 198 |
| T-L-LLs-c | dormant | 651 |  |  | 1622 | 26 |  | 1639 (1639) | v540 (v540) | 1475 | dormant (-18 ± 25 vs peak v540 before dormancy at v647, at 1493 vs refs; needs > +25) | 1610 (135740) | +391 -397 =212, 49.7%, -2 ± 19: reverted | 651 |
| T-L-LLs-merge-c | dormant | 1754 |  |  | 1539 | 30 |  | 1614 (1614) | v1251 (v1251) | 1393 | dormant (-75 ± 26 vs peak v1251 before dormancy at v1754, at 1468 vs refs; needs > +26) | 1610 (132197) | +402 -436 =162, 48.3%, -12 ± 20: reverted | 1754 |
| T-L-NLs-c | qualified | 1140 | 46 x 1000 | 240 | 1486 | 39 | 1531 | 1575 | v571 | 1339 | strike 1 (-89 ± 46 vs peak v571; predecessor -45) | 1610 (136356) | +408 -408 =184, 50.0%, -0 ± 20: kept | 1141 |
| T-L-NLs-merge-c | dormant | 1117 |  |  | 1568 | 30 |  | 1660 (1660) | v641 (v641) | 1421 | dormant (-92 ± 38 vs peak v641 before dormancy at v1115, at 1513 vs refs; needs > +38) | 1610 (135672) | +325 -518 =157, 40.4%, -68 ± 20: reverted | 1117 |
| T-L-NNs-c | qualified | 1119 | 45 x 1000 | 243 | 1592 | 41 | 1542 | 1612 | v561 | 1445 | ok (-20 ± 47 vs peak v561; predecessor +50) | 1610 (139186) | +450 -384 =166, 53.3%, +23 ± 20: kept | 1120 |
| T-L-NNs-merge-c | dormant | 1069 |  |  | 1680 | 32 |  | 1713 (1712) | v1067 (v818) | 1534 | dormant (-31 ± 30 vs peak v818 before dormancy at v1062, at 1565 vs refs; needs > +30) | 1610 (133911) | +464 -395 =141, 53.4%, +24 ± 20: kept | 1070 |
| T-L-Le-c | dormant | 986 |  |  | 1542 | 40 | 1510 | 1596 (1596) | v696 (v696) | 1395 | dormant (-54 ± 29 vs peak v696 before dormancy at v985, at 1449 vs refs; needs > +29) | 1610 (141803) | +531 -313 =156, 60.9%, +77 ± 20: kept | 987 |
| T-L-Le-merge-c | dormant | 2052 |  |  | 1673 | 32 |  | 1711 (1700) | v2051 (v1857) | 1526 | dormant (-27 ± 28 vs peak v1857 before dormancy at v2049, at 1553 vs refs; needs > +28) | 1610 (132757) | +402 -465 =133, 46.9%, -22 ± 20: reverted | 2052 |
| T-N-Ne-c | dormant | 637 |  |  | 1275 | 24 |  | 1378 (1378) | v574 (v574) | 1128 | dormant (-103 ± 25 vs peak v574 before dormancy at v633, at 1232 vs refs; needs > +25) | 1610 (135387) | +426 -458 =116, 48.4%, -11 ± 20: reverted | 637 |
| T-N-Ne-merge-c | dormant | 905 |  |  | 1382 | 38 | 1390 | 1454 (1454) | v699 (v699) | 1235 | dormant (-72 ± 24 vs peak v699 before dormancy at v904, at 1307 vs refs; needs > +24) | 1610 (131388) | +412 -425 =163, 49.4%, -4 ± 20: reverted | 905 |
| T-N-Nc-c | qualified | 674 | 40 x 1000 | 240 | 1345 | 38 | 1352 | 1397 | v472 | 1198 | strike 1 (-52 ± 46 vs peak v472; predecessor -7) | 1610 (132146) | +413 -415 =172, 49.9%, -1 ± 20: reverted | 674 |
| T-N-Nc-merge-c | qualified | 1082 | 67 x 1000 | 252 | 1435 | 38 | 1442 | 1456 | v631 | 1288 | ok (-22 ± 36 vs peak v631; predecessor -8) | 1610 (131315) | +386 -421 =193, 48.2%, -12 ± 19: reverted | 1082 |
| T-N-NNs-c | dormant | 628 |  |  | 1153 | 36 | 1117 | 1245 (1245) | v341 (v341) | 1006 | dormant (-92 ± 39 vs peak v341 before dormancy at v625, at 1098 vs refs; needs > +39) | 1610 (143799) | +420 -396 =184, 51.2%, +8 ± 20: kept | 629 |
| T-N-NNs-merge-c | dormant | 999 |  |  | 1371 | 28 |  | 1474 (1474) | v855 (v855) | 1224 | dormant (-103 ± 26 vs peak v855 before dormancy at v997, at 1328 vs refs; needs > +26) | 1610 (131644) | +350 -403 =247, 47.3%, -18 ± 19: reverted | 999 |
| T-N-Le-merge-c | dormant | 699 |  |  | 1400 | 38 | 1384 | 1405 (1405) | v580 (v580) | 1253 | dormant (-5 ± 27 vs peak v580 before dormancy at v697, at 1258 vs refs; needs > +27) | 1610 (131051) | +453 -395 =152, 52.9%, +20 ± 20: kept | 700 |
| T-N-Lc-merge-c | dormant | 366 |  |  | 1417 | 20 |  | 1446 (1446) | v287 (v287) | 1271 | dormant (-28 ± 22 vs peak v287 before dormancy at v364, at 1299 vs refs; needs > +22) | 1610 (135222) | +397 -426 =177, 48.5%, -10 ± 20: reverted | 366 |
| T-N-LLs-merge-c | dormant | 725 |  |  | 1126 | 27 |  | 1378 (1378) | v585 (v585) | 979 | dormant (-253 ± 23 vs peak v585 before dormancy at v723, at 1231 vs refs; needs > +23) | 1610 (143629) | +335 -480 =185, 42.8%, -51 ± 20: reverted | 725 |
| T-N-NLs-merge-c | dormant | 786 |  |  | 1380 | 28 |  | 1400 (1400) | v672 (v672) | 1234 | dormant (-20 ± 23 vs peak v672 before dormancy at v785, at 1253 vs refs; needs > +23) | 1610 (135845) | +448 -396 =156, 52.6%, +18 ± 20: kept | 787 |
| T-N-LNs-merge-c | dormant | 416 |  |  | 1248 | 37 | 1254 | 1355 (1355) | v270 (v270) | 1101 | dormant (-107 ± 27 vs peak v270 before dormancy at v410, at 1209 vs refs; needs > +27) | 1610 (134100) | +411 -418 =171, 49.6%, -2 ± 20: reverted | 416 |
| F-N-d2jit-c | qualified | 1269 | 133 x 40 | 240 | 1104 | 36 | 1148 | 1182 | v389 | 957 | strike 1 (-78 ± 39 vs peak v389; predecessor -44) | 1610 (141581) | +390 -384 =226, 50.3%, +2 ± 19: kept | 1270 |
| T-N-NNs-merge-d2jit-c | qualified | 923 | 102 x 40 | 239 | 1173 | 37 | 1151 | 1177 | v642 | 1026 | ok (-4 ± 39 vs peak v642; predecessor +22) | 1610 (139402) | +427 -379 =194, 52.4%, +17 ± 19: kept | 924 |
| F-L-d4mix-c | qualified | 68 | 7 x 10 | 228 | 1235 | 37 | 947 | 947 | v60 | 1088 | new peak (+288 ± 37 over v60; predecessor +288) | 1610 (152547) | +669 -202 =129, 73.4%, +176 ± 22: kept | 69 |
| MoE-d2jit-c | qualified | 249 | 57 x 40 | 239 | 1409 | 38 | 1319 | 1319 | v191 | 1262 | new peak (+90 ± 38 over v191; predecessor +90) | 1610 (155336) | +566 -263 =171, 65.1%, +109 ± 20: kept | 250 |
| T-L-H-c | dormant | 1523 |  |  | 1612 | 31 |  | 1666 (1666) | v1245 (v1245) | 1465 | dormant (-54 ± 27 vs peak v1245 before dormancy at v1521, at 1519 vs refs; needs > +27) | 1610 (134465) | +424 -427 =149, 49.9%, -1 ± 20: reverted | 1523 |
| T-N-H-c | dormant | 1733 |  |  | 1294 | 37 | 1298 | 1368 (1368) | v1104 (v1104) | 1148 | dormant (-74 ± 30 vs peak v1104 before dormancy at v1728, at 1221 vs refs; needs > +30) | 1610 (133482) | +449 -409 =142, 52.0%, +14 ± 20: kept | 1734 |
