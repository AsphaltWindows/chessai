# Cycle 11

**Changes from this cycle** (deployed after cycle 10, `staged-c10/`): F-N-d2jit-c, a lineage that self-plays at depth 2, from scratch (the user): F-N-c's recipe plus --depth 2.

**Own-games step, pooled over the lineages: 37 gates, +16464 -14709 =5827, score 52.4%, Elo +16.5 ± 3.3; kept 23 of 37.**

All cycles so far: 347 gates, +160848 -133108 =53044, score 54.0%, Elo +27.8 ± 1.1; kept 259 of 347.

2026-11-06 22:51: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then ran the rounds that fill the budget; round sizes d0 1000, d1 860, d2 40, d3 10.
League: retired 60 players to retired.txt (their games stay in the fit), added 62.
Tournament: 5 double round-robins of the 79 active players (30810 games) in 3.6 min, 30810 games saved.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 2 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 2 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | qualified | 2119 | 142 x 1000 | 254 | +1645 | 44 | +1652 | +1705 | v1185 | +1490 | ok (-60 ± 68 vs peak v1185; predecessor -7) | 1450 (121306) | +496 -387 =117, 55.5%, +38 ± 20: kept | 2120 |
| F-L-d0soft-c | dormant | 2006 | 180 x 1000 | 254 | +1569 | 43 | +1579 | +1685 | v1619 | +1415 | dormant (strike 2; -116 ± 62 vs peak v1619; predecessor -10) | 1450 (123494) | +563 -298 =139, 63.2%, +94 ± 21: kept | 2007 |
| F-L-d1jit-c | dormant | 1450 |  |  | +1703 | 46 | +1680 | +1680 (+1657) | v1449 (v1140) | +1549 | dormant (+46 ± 67 vs peak v1140 before dormancy at v1447, at 1502 vs refs; needs > +67) | 1450 (120388) | +455 -421 =124, 51.7%, +12 ± 20: kept | 1451 |
| F-L-d1play-c | qualified | 2070 | 134 x 860 | 235 | +1571 | 43 | +1558 | +1593 | v1269 | +1417 | ok (-22 ± 64 vs peak v1269; predecessor +13) | 1450 (134815) | +487 -340 =173, 57.4%, +51 ± 20: kept | 2071 |
| F-L-d2play-c | dormant | 1012 |  |  | +1781 | 49 | +1791 | +1791 (+1745) | v1011 (v818) | +1627 | dormant (+36 ± 88 vs peak v818 before dormancy at v1009, at 1591 vs refs; needs > +88) | 1450 (123247) | +402 -420 =178, 49.1%, -6 ± 20: reverted | 1012 |
| F-L-d2mix-c | dormant | 750 |  |  | +1757 | 23 |  | +1751 (+1751) | v591 (v591) | +1603 | dormant (+6 ± 78 vs peak v591 before dormancy at v750, at 1596 vs refs; needs > +78) | 1450 (121899) | +356 -443 =201, 45.6%, -30 ± 19: reverted | 750 |
| F-L-d2jit-c | qualified | 1651 | 110 x 40 | 230 | +1837 | 53 | +1823 | +1823 | v1540 | +1683 | new peak (+15 ± 71 over v1540; predecessor +15) | 1450 (127142) | +347 -435 =218, 45.6%, -31 ± 19: reverted | 1651 |
| F-N-c | qualified | 1061 |  |  | +1512 | 42 | +1478 | +1490 (+1428) | v1059 (v1056) | +1358 | re-qualified (+84 ± 63 vs peak v1056 before dormancy at v1056, at 1274 vs refs) | 1450 (118725) | +426 -395 =179, 51.5%, +11 ± 20: kept | 1062 |
| MoE-c | dormant | 299 |  |  | +1691 | 46 | +1710 | +1710 (+1656) | v298 (v233) | +1537 | dormant (+36 ± 66 vs peak v233 before dormancy at v296, at 1501 vs refs; needs > +66) | 1450 (119668) | +437 -410 =153, 51.3%, +9 ± 20: kept | 300 |
| FM-c | qualified | 309 | 20 x 1000 | 218 | +1363 | 42 | +1308 | +1355 | v246 | +1208 | new peak (+8 ± 58 over v246; predecessor +55) | 1450 (135331) | +505 -306 =189, 60.0%, +70 ± 20: kept | 310 |
| TAN-c | qualified | 618 | 111 x 1000 | 240 | +1194 | 41 | +1274 | +1274 | v507 | +1040 | strike 1 (-79 ± 54 vs peak v507; predecessor -79) | 1450 (132290) | +441 -437 =122, 50.2%, +1 ± 20: kept | 619 |
| T-L-Ne-c | dormant | 556 |  |  | +1650 | 29 |  | +1636 (+1636) | v483 (v483) | +1496 | dormant (+14 ± 56 vs peak v483 before dormancy at v555, at 1481 vs refs; needs > +56) | 1450 (126108) | +429 -437 =134, 49.6%, -3 ± 20: reverted | 556 |
| T-L-Ne-merge-c | dormant | 793 |  |  | +1723 | 35 |  | +1741 (+1688) | v792 (v700) | +1569 | dormant (+35 ± 62 vs peak v700 before dormancy at v790, at 1534 vs refs; needs > +62) | 1450 (114943) | +457 -407 =136, 52.5%, +17 ± 20: kept | 794 |
| T-L-Nc-c | qualified | 516 |  |  | +1685 | 45 | +1664 | +1664 (+1614) | v515 (v477) | +1531 | re-qualified (+71 ± 66 vs peak v477 before dormancy at v511, at 1460 vs refs) | 1450 (126385) | +423 -426 =151, 49.9%, -1 ± 20: reverted | 516 |
| T-L-Nc-merge-c | qualified | 1478 | 112 x 1000 | 253 | +1652 | 44 | +1685 | +1723 | v833 | +1497 | strike 1 (-72 ± 67 vs peak v833; predecessor -33) | 1450 (119177) | +513 -357 =130, 57.8%, +55 ± 20: kept | 1479 |
| T-L-LLs-c | qualified | 540 |  |  | +1670 | 45 | +1618 | +1618 (+1600) | v539 (v504) | +1515 | re-qualified (+70 ± 63 vs peak v504 before dormancy at v538, at 1446 vs refs) | 1450 (125069) | +396 -431 =173, 48.2%, -12 ± 20: reverted | 540 |
| T-L-LLs-merge-c | qualified | 1311 | 59 x 1000 | 276 | +1585 | 43 | +1644 | +1644 | v1251 | +1430 | strike 1 (-59 ± 58 vs peak v1251; predecessor -59) | 1450 (121591) | +449 -407 =144, 52.1%, +15 ± 20: kept | 1312 |
| T-L-NLs-c | qualified | 738 | 43 x 1000 | 240 | +1565 | 43 | +1594 | +1602 | v571 | +1410 | ok (-38 ± 62 vs peak v571; predecessor -29) | 1450 (126386) | +436 -404 =160, 51.6%, +11 ± 20: kept | 739 |
| T-L-NLs-merge-c | dormant | 994 |  |  | +1600 | 26 |  | +1667 (+1667) | v641 (v641) | +1446 | dormant (-67 ± 75 vs peak v641 before dormancy at v994, at 1512 vs refs; needs > +75) | 1450 (123977) | +354 -470 =176, 44.2%, -40 ± 20: reverted | 994 |
| T-L-NNs-c | qualified | 731 | 39 x 1000 | 220 | +1590 | 43 | +1625 | +1639 | v561 | +1436 | ok (-49 ± 62 vs peak v561; predecessor -35) | 1450 (126119) | +410 -452 =138, 47.9%, -15 ± 20: reverted | 731 |
| T-L-NNs-merge-c | dormant | 1062 | 86 x 1000 | 176 | +1647 | 44 | +1661 | +1741 | v818 | +1493 | dormant (strike 2; -94 ± 64 vs peak v818; predecessor -14) | 1450 (119839) | +451 -396 =153, 52.8%, +19 ± 20: kept | 1063 |
| T-L-Le-c | qualified | 657 | 39 x 1000 | 276 | +1591 | 43 | +1561 | +1606 | v581 | +1437 | ok (-15 ± 59 vs peak v581; predecessor +30) | 1450 (116600) | +516 -374 =110, 57.1%, +50 ± 20: kept | 658 |
| T-L-Le-merge-c | qualified | 1480 | 81 x 1000 | 191 | +1673 | 45 | +1623 | +1695 | v1083 | +1519 | ok (-21 ± 65 vs peak v1083; predecessor +51) | 1450 (119735) | +544 -344 =112, 60.0%, +70 ± 21: kept | 1481 |
| T-N-Ne-c | dormant | 633 | 30 x 1000 | 220 | +1328 | 42 | +1329 | +1407 | v574 | +1174 | dormant (strike 2; -79 ± 58 vs peak v574; predecessor -1) | 1450 (119927) | +408 -458 =134, 47.5%, -17 ± 20: reverted | 633 |
| T-N-Ne-merge-c | dormant | 699 |  |  | +1483 | 32 |  | +1482 (+1429) | v697 (v500) | +1329 | dormant (+55 ± 78 vs peak v500 before dormancy at v695, at 1274 vs refs; needs > +78) | 1450 (120471) | +418 -413 =169, 50.2%, +2 ± 20: kept | 700 |
| T-N-Nc-c | dormant | 474 |  |  | +1395 | 26 |  | +1423 (+1383) | v472 (v390) | +1240 | dormant (+12 ± 75 vs peak v390 before dormancy at v471, at 1228 vs refs; needs > +75) | 1450 (118187) | +439 -399 =162, 52.0%, +14 ± 20: kept | 475 |
| T-N-Nc-merge-c | dormant | 631 |  |  | +1486 | 42 | +1467 | +1475 (+1475) | v628 (v628) | +1332 | dormant (+11 ± 54 vs peak v628 before dormancy at v628, at 1321 vs refs; needs > +54) | 1450 (115630) | +400 -427 =173, 48.6%, -9 ± 20: reverted | 631 |
| T-N-NNs-c | dormant | 538 |  |  | +1179 | 41 | +1210 | +1266 (+1266) | v452 (v452) | +1025 | dormant (-87 ± 61 vs peak v452 before dormancy at v536, at 1112 vs refs; needs > +61) | 1450 (127473) | +428 -412 =160, 50.8%, +6 ± 20: kept | 539 |
| T-N-NNs-merge-c | qualified | 948 | 56 x 1000 | 275 | +1437 | 42 | +1490 | +1504 | v855 | +1282 | strike 1 (-67 ± 58 vs peak v855; predecessor -53) | 1450 (119649) | +403 -400 =197, 50.1%, +1 ± 19: kept | 949 |
| T-N-Le-merge-c | qualified | 208 | 114 x 1000 | 260 | +1343 | 42 | +1287 | +1287 | v93 | +1189 | new peak (+56 ± 55 over v93; predecessor +56) | 1450 (127838) | +381 -457 =162, 46.2%, -26 ± 20: reverted | 208 |
| T-N-Lc-merge-c | qualified | 217 | 113 x 1000 | 296 | +1436 | 42 | +1330 | +1330 | v103 | +1281 | new peak (+105 ± 55 over v103; predecessor +105) | 1450 (117680) | +411 -402 =187, 50.4%, +3 ± 19: kept | 218 |
| T-N-LLs-merge-c | qualified | 213 | 118 x 1000 | 335 | +1326 | 42 | +1268 | +1268 | v95 | +1172 | new peak (+59 ± 55 over v95; predecessor +59) | 1450 (126014) | +399 -436 =165, 48.1%, -13 ± 20: reverted | 213 |
| T-N-NLs-merge-c | qualified | 213 | 115 x 1000 | 330 | +1324 | 42 | +1282 | +1282 | v98 | +1170 | new peak (+42 ± 55 over v98; predecessor +42) | 1450 (122666) | +431 -414 =155, 50.8%, +6 ± 20: kept | 214 |
| T-N-LNs-merge-c | qualified | 218 | 115 x 1000 | 467 | +1322 | 42 | +1310 | +1310 | v102 | +1168 | new peak (+13 ± 55 over v102; predecessor +13) | 1450 (123891) | +397 -441 =162, 47.8%, -15 ± 20: reverted | 218 |
| F-N-d2jit-c | qualified | 100 | 100 x 40 | 195 | +572 | 49 |  |  |  | +418 | qualified (first rating) | 1450 (123095) | +785 -36 =179, 87.5%, +337 ± 25: kept | 101 |
| T-L-H-c | dormant | 1243 |  |  | +1679 | 45 | +1667 | +1668 (+1668) | v970 (v970) | +1525 | dormant (+11 ± 66 vs peak v970 before dormancy at v1240, at 1514 vs refs; needs > +66) | 1450 (121237) | +447 -386 =167, 53.0%, +21 ± 20: kept | 1244 |
| T-N-H-c | qualified | 1605 | 114 x 1000 | 220 | +1326 | 42 | +1345 | +1395 | v1104 | +1172 | strike 1 (-69 ± 60 vs peak v1104; predecessor -19) | 1450 (119760) | +424 -431 =145, 49.6%, -2 ± 20: reverted | 1605 |
