# Cycle 17

**Own-games step, pooled over the lineages: 40 gates, +17463 -15684 =6853, score 52.2%, Elo +15.5 ± 3.1; kept 26 of 40.**

All cycles so far: 578 gates, +263197 -222518 =92285, score 53.5%, Elo +24.5 ± 0.8; kept 401 of 578.

2026-11-07 02:09: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 68 players to retired.txt (their games stay in the fit), added 68.
Tournament: 5 double round-robins of the 85 active players (35700 games) in 3.8 min, 35700 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| F-L-d0soft-c | 6 of 10 | cycle 21 | 12–17 | 1014.1 | 1640.8 | retire |
| T-L-NNs-merge-c | 6 of 10 | cycle 21 | 12–17 | 1003.5 | 1688.9 | freeze 1 at 1000 |
| T-N-Ne-c | 6 of 10 | cycle 21 | 12–17 | 740.9 | 1308.0 | freeze 2 at 1300 |
| T-N-H-c | 5 of 10 | cycle 22 | 13–17 | 751.9 | 1287.2 | retire |
| T-N-NNs-merge-c | 5 of 10 | cycle 22 | 13–17 | 758.5 | 1378.8 | retire |
| F-L-d1jit-c | 3 of 10 | cycle 24 | 15–17 | 1045.2 | 1636.3 | retire |
| MoE-c | 3 of 10 | cycle 24 | 15–17 | 974.8 | 1692.7 | freeze 2 at 1700 |
| T-L-LLs-c | 3 of 10 | cycle 24 | 15–17 | 953.4 | 1592.9 | freeze 2 at 1600 |
| T-N-LNs-merge-c | 3 of 10 | cycle 24 | 15–17 | 667.4 | 1144.8 | retire |
| T-N-Lc-merge-c | 3 of 10 | cycle 24 | 15–17 | 760.0 | 1419.6 | retire |
| TAN-c | 3 of 10 | cycle 24 | 15–17 | 728.1 | 1217.8 | retire |
| F-L-c | 1 of 10 | cycle 26 | 17–17 | 1000.4 | 1698.7 | retire |
| F-N-c | 1 of 10 | cycle 26 | 17–17 | 732.6 | 1402.0 | freeze 2 at 1400 |
| T-L-Le-merge-c | 1 of 10 | cycle 26 | 17–17 | 978.0 | 1661.3 | retire |
| T-L-Nc-c | 1 of 10 | cycle 26 | 17–17 | 1001.2 | 1599.0 | retire |
| F-L-d1play-c | 0 of 10 | cycle 27 | - | - | - | retire |
| T-L-H-c | 0 of 10 | cycle 27 | - | - | - | retire |
| T-L-NLs-merge-c | 0 of 10 | cycle 27 | - | - | - | retire |
| T-L-Ne-merge-c | 0 of 10 | cycle 27 | - | - | - | retire |
| T-N-LLs-merge-c | 0 of 10 | cycle 27 | - | - | - | retire |
| T-N-NNs-c | 0 of 10 | cycle 27 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4.


Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | dormant | 2855 |  |  | 1699 | 46 | 1621 | 1694 (1694) | v2247 (v2247) | 1554 | dormant (+5 ± 32 vs peak v2247 before dormancy at v2854, at 1549 vs refs; needs > +32) | 1570 (129838) | +477 -396 =127, 54.0%, +28 ± 20: kept | 2856 |
| F-L-d0soft-c | dormant | 2012 |  |  | 1660 | 45 | 1630 | 1654 (1654) | v1619 (v1619) | 1515 | dormant (+6 ± 33 vs peak v1619 before dormancy at v2006, at 1510 vs refs; needs > +33) | 1570 (129029) | +445 -408 =147, 51.8%, +13 ± 20: kept | 2013 |
| F-L-d1jit-c | dormant | 1765 |  |  | 1625 | 32 |  | 1703 (1703) | v1451 (v1451) | 1480 | dormant (-78 ± 28 vs peak v1451 before dormancy at v1763, at 1558 vs refs; needs > +28) | 1570 (136031) | +444 -405 =151, 51.9%, +14 ± 20: kept | 1766 |
| F-L-d1play-c | dormant | 2999 | 133 x 860 | 241 | 1516 | 42 | 1499 | 1564 | v1269 | 1371 | dormant (strike 2; -48 ± 48 vs peak v1269; predecessor +17) | 1570 (148505) | +494 -343 =163, 57.6%, +53 ± 20: kept | 3000 |
| F-L-d2play-c | qualified | 1206 | 100 x 40 | 240 | 1727 | 48 | 1704 | 1759 | v1011 | 1582 | ok (-32 ± 45 vs peak v1011; predecessor +23) | 1570 (131901) | +393 -421 =186, 48.6%, -10 ± 20: reverted | 1206 |
| F-L-d2mix-c | qualified | 910 | 79 x 40 | 241 | 1749 | 49 | 1674 | 1740 | v591 | 1604 | new peak (+9 ± 66 over v591; predecessor +75) | 1570 (138301) | +358 -399 =243, 47.9%, -14 ± 19: reverted | 910 |
| F-L-d2jit-c | qualified | 2319 | 111 x 40 | 240 | 1761 | 50 | 1766 | 1804 | v1651 | 1616 | ok (-44 ± 54 vs peak v1651; predecessor -6) | 1570 (139615) | +344 -471 =185, 43.6%, -44 ± 20: reverted | 2319 |
| F-N-c | dormant | 1684 |  |  | 1402 | 40 | 1413 | 1481 (1481) | v1061 (v1061) | 1257 | dormant (-79 ± 29 vs peak v1061 before dormancy at v1683, at 1336 vs refs; needs > +29) | 1570 (129069) | +436 -430 =134, 50.3%, +2 ± 20: kept | 1685 |
| MoE-c | dormant | 369 |  |  | 1696 | 34 |  | 1726 (1726) | v300 (v300) | 1551 | dormant (-30 ± 28 vs peak v300 before dormancy at v367, at 1581 vs refs; needs > +28) | 1570 (130106) | +492 -376 =132, 55.8%, +40 ± 20: kept | 370 |
| FM-c | qualified | 440 | 20 x 1000 | 246 | 1302 | 39 | 1446 | 1446 | v419 | 1157 | strike 1 (-144 ± 40 vs peak v419; predecessor -144) | 1570 (139123) | +615 -278 =107, 66.8%, +122 ± 22: kept | 441 |
| TAN-c | dormant | 929 |  |  | 1217 | 24 |  | 1244 (1244) | v507 (v507) | 1073 | dormant (-27 ± 23 vs peak v507 before dormancy at v928, at 1100 vs refs; needs > +23) | 1570 (138706) | +407 -458 =135, 47.4%, -18 ± 20: reverted | 929 |
| T-L-Ne-c | qualified | 619 | 30 x 1000 | 243 | 1604 | 44 | 1577 | 1614 | v556 | 1460 | ok (-9 ± 37 vs peak v556; predecessor +27) | 1570 (131796) | +421 -431 =148, 49.5%, -4 ± 20: reverted | 619 |
| T-L-Ne-merge-c | dormant | 1008 | 109 x 1000 | 240 | 1620 | 44 | 1612 | 1711 | v794 | 1476 | dormant (strike 2; -91 ± 38 vs peak v794; predecessor +8) | 1570 (123779) | +513 -364 =123, 57.5%, +52 ± 20: kept | 1009 |
| T-L-Nc-c | dormant | 710 |  |  | 1599 | 43 | 1556 | 1653 (1653) | v516 (v516) | 1454 | dormant (-54 ± 31 vs peak v516 before dormancy at v709, at 1508 vs refs; needs > +31) | 1570 (136835) | +405 -390 =205, 50.7%, +5 ± 19: kept | 711 |
| T-L-Nc-merge-c | qualified | 1631 |  |  | 1733 | 48 | 1641 | 1694 (1694) | v833 (v833) | 1589 | re-qualified (+39 ± 36 vs peak v833 before dormancy at v1626, at 1550 vs refs) | 1570 (128275) | +400 -451 =149, 47.4%, -18 ± 20: reverted | 1631 |
| T-L-LLs-c | dormant | 650 |  |  | 1582 | 43 | 1566 | 1638 (1638) | v540 (v540) | 1438 | dormant (-55 ± 31 vs peak v540 before dormancy at v647, at 1493 vs refs; needs > +31) | 1570 (134630) | +436 -373 =191, 53.1%, +22 ± 19: kept | 651 |
| T-L-LLs-merge-c | qualified | 1646 | 51 x 1000 | 242 | 1567 | 43 | 1585 | 1613 | v1251 | 1423 | ok (-45 ± 46 vs peak v1251; predecessor -17) | 1570 (131948) | +371 -454 =175, 45.9%, -29 ± 20: reverted | 1646 |
| T-L-NLs-c | qualified | 1002 | 39 x 1000 | 243 | 1572 | 43 | 1537 | 1573 | v571 | 1427 | ok (-1 ± 48 vs peak v571; predecessor +35) | 1570 (136083) | +272 -538 =190, 36.7%, -95 ± 20: reverted | 1002 |
| T-L-NLs-merge-c | dormant | 1115 | 55 x 1000 | 240 | 1552 | 42 | 1576 | 1656 | v641 | 1408 | dormant (strike 2; -104 ± 61 vs peak v641; predecessor -24) | 1570 (138186) | +430 -402 =168, 51.4%, +10 ± 20: kept | 1116 |
| T-L-NNs-c | qualified | 981 | 42 x 1000 | 240 | 1593 | 43 | 1588 | 1610 | v561 | 1449 | ok (-16 ± 48 vs peak v561; predecessor +6) | 1570 (131033) | +400 -447 =153, 47.6%, -16 ± 20: reverted | 981 |
| T-L-NNs-merge-c | dormant | 1067 |  |  | 1710 | 47 | 1658 | 1710 (1710) | v818 (v818) | 1566 | dormant (+1 ± 34 vs peak v818 before dormancy at v1062, at 1565 vs refs; needs > +34) | 1570 (130933) | +446 -411 =143, 51.7%, +12 ± 20: kept | 1068 |
| T-L-Le-c | qualified | 897 | 41 x 1000 | 242 | 1556 | 42 | 1506 | 1594 | v696 | 1411 | ok (-38 ± 45 vs peak v696; predecessor +50) | 1570 (130391) | +464 -403 =133, 53.0%, +21 ± 20: kept | 898 |
| T-L-Le-merge-c | dormant | 2050 |  |  | 1661 | 45 | 1587 | 1698 (1698) | v1857 (v1857) | 1517 | dormant (-37 ± 32 vs peak v1857 before dormancy at v2049, at 1554 vs refs; needs > +32) | 1570 (128038) | +471 -381 =148, 54.5%, +31 ± 20: kept | 2051 |
| T-N-Ne-c | dormant | 636 |  |  | 1290 | 39 | 1288 | 1377 (1377) | v574 (v574) | 1145 | dormant (-87 ± 29 vs peak v574 before dormancy at v633, at 1232 vs refs; needs > +29) | 1570 (135581) | +448 -435 =117, 50.6%, +4 ± 20: kept | 637 |
| T-N-Ne-merge-c | qualified | 845 | 72 x 1000 | 240 | 1395 | 40 | 1413 | 1452 | v699 | 1251 | strike 1 (-57 ± 38 vs peak v699; predecessor -18) | 1570 (128056) | +425 -411 =164, 50.7%, +5 ± 20: kept | 846 |
| T-N-Nc-c | qualified | 553 | 39 x 1000 | 240 | 1357 | 40 | 1384 | 1395 | v472 | 1213 | ok (-38 ± 47 vs peak v472; predecessor -27) | 1570 (127854) | +405 -418 =177, 49.4%, -4 ± 20: reverted | 553 |
| T-N-Nc-merge-c | qualified | 847 | 99 x 1000 | 239 | 1414 | 40 | 1452 | 1455 | v631 | 1270 | strike 1 (-40 ± 37 vs peak v631; predecessor -38) | 1570 (128841) | +393 -423 =184, 48.5%, -10 ± 20: reverted | 847 |
| T-N-NNs-c | dormant | 625 | 43 x 1000 | 245 | 1176 | 38 | 1170 | 1241 | v341 | 1031 | dormant (strike 2; -66 ± 59 vs peak v341; predecessor +6) | 1570 (141461) | +428 -402 =170, 51.3%, +9 ± 20: kept | 626 |
| T-N-NNs-merge-c | dormant | 998 |  |  | 1377 | 19 |  | 1473 (1473) | v855 (v855) | 1232 | dormant (-96 ± 23 vs peak v855 before dormancy at v997, at 1328 vs refs; needs > +23) | 1570 (129633) | +378 -385 =237, 49.6%, -2 ± 19: reverted | 998 |
| T-N-Le-merge-c | qualified | 657 | 39 x 1000 | 240 | 1345 | 40 | 1371 | 1404 | v580 | 1200 | strike 1 (-58 ± 41 vs peak v580; predecessor -26) | 1570 (129259) | +465 -396 =139, 53.4%, +24 ± 20: kept | 658 |
| T-N-Lc-merge-c | dormant | 366 |  |  | 1408 | 30 |  | 1444 (1444) | v287 (v287) | 1263 | dormant (-36 ± 25 vs peak v287 before dormancy at v364, at 1299 vs refs; needs > +25) | 1570 (129886) | +389 -440 =171, 47.4%, -18 ± 20: reverted | 366 |
| T-N-LLs-merge-c | dormant | 723 | 50 x 1000 | 240 | 1128 | 38 | 1272 | 1377 | v585 | 983 | dormant (strike 2; -249 ± 40 vs peak v585; predecessor -145) | 1570 (141776) | +462 -355 =183, 55.4%, +37 ± 20: kept | 724 |
| T-N-NLs-merge-c | qualified | 732 | 60 x 1000 | 243 | 1191 | 38 | 1397 | 1397 | v672 | 1047 | strike 1 (-206 ± 39 vs peak v672; predecessor -206) | 1570 (141372) | +454 -378 =168, 53.8%, +26 ± 20: kept | 733 |
| T-N-LNs-merge-c | dormant | 413 |  |  | 1226 | 38 | 1127 | 1354 (1354) | v270 (v270) | 1081 | dormant (-128 ± 27 vs peak v270 before dormancy at v410, at 1209 vs refs; needs > +27) | 1570 (137833) | +470 -377 =153, 54.6%, +32 ± 20: kept | 414 |
| F-N-d2jit-c | qualified | 872 | 123 x 40 | 239 | 1152 | 38 | 1166 | 1181 | v389 | 1007 | ok (-29 ± 40 vs peak v389; predecessor -14) | 1570 (143639) | +458 -350 =192, 55.4%, +38 ± 20: kept | 873 |
| T-N-NNs-merge-d2jit-c | qualified | 642 | 90 x 40 | 240 | 1174 | 38 | 1153 | 1153 | v551 | 1029 | new peak (+21 ± 38 over v551; predecessor +21) | 1570 (139766) | +410 -407 =183, 50.1%, +1 ± 20: kept | 643 |
| F-L-d4mix-c | qualified | 43 | 8 x 10 | 245 | 691 | 40 | 644 | 644 | v34 | 547 | new peak (+47 ± 42 over v34; predecessor +47) | 1570 (160313) | +420 -167 =413, 62.6%, +90 ± 17: kept | 44 |
| MoE-d2jit-c | qualified | 76 | 43 x 40 | 239 | 964 | 37 | 560 | 560 | v32 | 820 | new peak (+405 ± 42 over v32; predecessor +405) | 1570 (163133) | +602 -147 =251, 72.8%, +171 ± 20: kept | 77 |
| T-L-H-c | dormant | 1521 | 154 x 1000 | 240 | 1601 | 43 | 1602 | 1665 | v1245 | 1456 | dormant (strike 2; -64 ± 46 vs peak v1245; predecessor -0) | 1570 (131630) | +504 -341 =155, 58.1%, +57 ± 20: kept | 1522 |
| T-N-H-c | dormant | 1731 |  |  | 1287 | 29 |  | 1366 (1366) | v1104 (v1104) | 1142 | dormant (-79 ± 27 vs peak v1104 before dormancy at v1728, at 1221 vs refs; needs > +27) | 1570 (133255) | +418 -422 =160, 49.8%, -1 ± 20: reverted | 1731 |
