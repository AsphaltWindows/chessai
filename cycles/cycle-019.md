# Cycle 19

**Changes from this cycle** (deployed after cycle 18, `staged-c18/`): T-L-Nc-fresh-c, from scratch (the user): T-L-Nc-merge-c's recipe with another merge test, on `chessai-resid4`. Its shadow test scores each self-play round's positions by the model as it starts the round, before training on them (`--revise-on fresh`), and a branch merges back into its shadow only when it is worse by more than 2 standard errors (`--revise 2`); T-L-Nc-merge-c tests on the training stream at z 0 and T-L-Nc-c never merges. Its own step trains and gates with `chessai-resid4` too (`own_games.binary_by_lineage` in `config.json`), since `chessai-cyc5-labels` lacks `--revise-on`.

**Own-games step, pooled over the lineages: 41 gates, +18219 -16088 =6693, score 52.6%, Elo +18.1 ± 3.1; kept 25 of 41.**

All cycles so far: 659 gates, +299544 -253927 =105529, score 53.5%, Elo +24.1 ± 0.8; kept 455 of 659.

2026-11-07 03:00: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 68 players to retired.txt (their games stay in the fit), added 70.
Tournament: 5 double round-robins of the 87 active players (37410 games) in 4.0 min, 37410 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| F-L-d0soft-c | 8 of 10 | cycle 21 | 12–19 | 1005.0 | 1641.0 | retire |
| T-L-NNs-merge-c | 8 of 10 | cycle 21 | 12–19 | 996.4 | 1692.3 | freeze 1 at 1000 |
| T-N-Ne-c | 8 of 10 | cycle 21 | 12–19 | 737.2 | 1302.1 | freeze 2 at 1300 |
| T-N-H-c | 7 of 10 | cycle 22 | 13–19 | 749.9 | 1289.2 | retire |
| T-N-NNs-merge-c | 7 of 10 | cycle 22 | 13–19 | 750.9 | 1376.5 | retire |
| F-L-d1jit-c | 5 of 10 | cycle 24 | 15–19 | 1034.3 | 1630.5 | retire |
| MoE-c | 5 of 10 | cycle 24 | 15–19 | 965.3 | 1696.0 | freeze 2 at 1700 |
| T-L-LLs-c | 5 of 10 | cycle 24 | 15–19 | 955.4 | 1602.5 | freeze 2 at 1600 |
| T-N-LNs-merge-c | 5 of 10 | cycle 24 | 15–19 | 685.1 | 1186.5 | retire |
| T-N-Lc-merge-c | 5 of 10 | cycle 24 | 15–19 | 755.7 | 1417.2 | retire |
| TAN-c | 5 of 10 | cycle 24 | 15–19 | 728.7 | 1218.8 | retire |
| F-L-c | 3 of 10 | cycle 26 | 17–19 | 1003.2 | 1676.7 | retire |
| F-N-c | 3 of 10 | cycle 26 | 17–19 | 741.2 | 1399.4 | freeze 2 at 1400 |
| T-L-Le-merge-c | 3 of 10 | cycle 26 | 17–19 | 980.0 | 1689.1 | retire |
| T-L-Nc-c | 3 of 10 | cycle 26 | 17–19 | 995.8 | 1584.1 | retire |
| F-L-d1play-c | 2 of 10 | cycle 27 | 18–19 | 1013.3 | 1504.0 | freeze 2 at 1500 |
| T-L-H-c | 2 of 10 | cycle 27 | 18–19 | 1002.4 | 1593.6 | retire |
| T-L-NLs-merge-c | 2 of 10 | cycle 27 | 18–19 | 947.0 | 1577.5 | retire |
| T-L-Ne-merge-c | 2 of 10 | cycle 27 | 18–19 | 978.6 | 1670.8 | retire |
| T-N-LLs-merge-c | 2 of 10 | cycle 27 | 18–19 | 634.7 | 1116.2 | retire |
| T-N-NNs-c | 2 of 10 | cycle 27 | 18–19 | 654.9 | 1131.3 | retire |
| T-N-Le-merge-c | 1 of 10 | cycle 28 | 19–19 | 743.6 | 1388.6 | retire |
| T-N-NLs-merge-c | 1 of 10 | cycle 28 | 19–19 | 777.5 | 1401.9 | retire |
| T-N-Ne-merge-c | 1 of 10 | cycle 28 | 19–19 | 760.0 | 1392.8 | retire |
| F-L-d2mix-c | 0 of 10 | cycle 29 | - | - | - | retire |
| T-L-LLs-merge-c | 0 of 10 | cycle 29 | - | - | - | retire |
| T-L-Le-c | 0 of 10 | cycle 29 | - | - | - | retire |
| T-L-Nc-merge-c | 0 of 10 | cycle 29 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4.


Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | dormant | 2857 |  |  | 1669 | 44 | 1666 | 1702 (1695) | v2855 (v2247) | 1523 | dormant (-26 ± 32 vs peak v2247 before dormancy at v2854, at 1549 vs refs; needs > +32) | 1610 (129042) | +469 -367 =164, 55.1%, +36 ± 20: kept | 2858 |
| F-L-d0soft-c | dormant | 2013 |  |  | 1642 | 32 |  | 1663 (1656) | v2012 (v1619) | 1496 | dormant (-14 ± 29 vs peak v1619 before dormancy at v2006, at 1509 vs refs; needs > +29) | 1610 (131379) | +453 -385 =162, 53.4%, +24 ± 20: kept | 2014 |
| F-L-d1jit-c | dormant | 1767 |  |  | 1641 | 43 | 1606 | 1704 (1704) | v1451 (v1451) | 1495 | dormant (-63 ± 31 vs peak v1451 before dormancy at v1763, at 1558 vs refs; needs > +31) | 1610 (134598) | +446 -406 =148, 52.0%, +14 ± 20: kept | 1768 |
| F-L-d1play-c | dormant | 3001 |  |  | 1529 | 40 | 1482 | 1565 (1565) | v1269 (v1269) | 1383 | dormant (-36 ± 32 vs peak v1269 before dormancy at v2999, at 1419 vs refs; needs > +32) | 1610 (143198) | +474 -384 =142, 54.5%, +31 ± 20: kept | 3002 |
| F-L-d2play-c | qualified | 1413 | 104 x 40 | 240 | 1725 | 46 | 1689 | 1760 | v1011 | 1579 | ok (-34 ± 45 vs peak v1011; predecessor +36) | 1610 (141953) | +420 -401 =179, 50.9%, +7 ± 20: kept | 1414 |
| F-L-d2mix-c | dormant | 1068 | 81 x 40 | 242 | 1679 | 44 | 1692 | 1752 | v910 | 1533 | dormant (strike 2; -73 ± 49 vs peak v910; predecessor -13) | 1610 (142346) | +388 -312 =300, 53.8%, +26 ± 18: kept | 1069 |
| F-L-d2jit-c | qualified | 2566 | 122 x 40 | 239 | 1756 | 48 | 1777 | 1805 | v1651 | 1610 | ok (-49 ± 53 vs peak v1651; predecessor -21) | 1610 (140429) | +365 -426 =209, 46.9%, -21 ± 19: reverted | 2566 |
| F-N-c | dormant | 1686 |  |  | 1412 | 39 | 1387 | 1482 (1482) | v1061 (v1061) | 1266 | dormant (-70 ± 28 vs peak v1061 before dormancy at v1683, at 1336 vs refs; needs > +28) | 1610 (132692) | +423 -419 =158, 50.2%, +1 ± 20: kept | 1687 |
| MoE-c | dormant | 371 |  |  | 1696 | 45 | 1709 | 1727 (1727) | v300 (v300) | 1550 | dormant (-31 ± 32 vs peak v300 before dormancy at v367, at 1581 vs refs; needs > +32) | 1610 (132250) | +435 -382 =183, 52.6%, +18 ± 20: kept | 372 |
| FM-c | qualified | 485 | 22 x 1000 | 241 | 1419 | 39 | 1412 | 1448 | v419 | 1273 | ok (-30 ± 41 vs peak v419; predecessor +7) | 1610 (139383) | +607 -259 =134, 67.4%, +126 ± 21: kept | 486 |
| TAN-c | dormant | 929 |  |  | 1221 | 19 |  | 1245 (1245) | v507 (v507) | 1075 | dormant (-24 ± 22 vs peak v507 before dormancy at v928, at 1099 vs refs; needs > +22) | 1610 (141649) | +410 -460 =130, 47.5%, -17 ± 20: reverted | 929 |
| T-L-Ne-c | qualified | 696 | 39 x 1000 | 243 | 1614 | 42 | 1563 | 1615 | v556 | 1468 | ok (-1 ± 37 vs peak v556; predecessor +51) | 1610 (135190) | +481 -357 =162, 56.2%, +43 ± 20: kept | 697 |
| T-L-Ne-merge-c | dormant | 1010 |  |  | 1682 | 44 | 1663 | 1712 (1712) | v794 (v794) | 1536 | dormant (-30 ± 26 vs peak v794 before dormancy at v1008, at 1566 vs refs; needs > +26) | 1610 (128678) | +446 -446 =108, 50.0%, -0 ± 20: kept | 1011 |
| T-L-Nc-c | dormant | 712 |  |  | 1574 | 41 | 1582 | 1654 (1654) | v516 (v516) | 1428 | dormant (-79 ± 30 vs peak v516 before dormancy at v709, at 1508 vs refs; needs > +30) | 1610 (137215) | +453 -385 =162, 53.4%, +24 ± 20: kept | 713 |
| T-L-Nc-merge-c | dormant | 1878 | 112 x 1000 | 240 | 1595 | 42 | 1572 | 1736 | v1631 | 1449 | dormant (strike 2; -142 ± 47 vs peak v1631; predecessor +22) | 1610 (132977) | +510 -376 =114, 56.7%, +47 ± 20: kept | 1879 |
| T-L-Nc-fresh-c | qualified | 123 | 123 x 1000 | 255 | 1380 | 39 |  |  |  | 1234 | qualified (first rating) | 1610 (138280) | +555 -310 =135, 62.3%, +87 ± 21: kept | 124 |
| T-L-LLs-c | dormant | 651 |  |  | 1622 | 32 |  | 1639 (1639) | v540 (v540) | 1476 | dormant (-17 ± 27 vs peak v540 before dormancy at v647, at 1493 vs refs; needs > +27) | 1610 (135496) | +375 -431 =194, 47.2%, -20 ± 19: reverted | 651 |
| T-L-LLs-merge-c | dormant | 1754 | 59 x 1000 | 242 | 1528 | 40 | 1511 | 1614 | v1251 | 1382 | dormant (strike 2; -86 ± 45 vs peak v1251; predecessor +17) | 1610 (133796) | +409 -418 =173, 49.5%, -3 ± 20: reverted | 1754 |
| T-L-NLs-c | qualified | 1094 | 46 x 1000 | 241 | 1536 | 40 | 1520 | 1575 | v1002 | 1390 | ok (-39 ± 43 vs peak v1002; predecessor +16) | 1610 (135878) | +399 -466 =135, 46.7%, -23 ± 20: reverted | 1094 |
| T-L-NLs-merge-c | dormant | 1117 |  |  | 1568 | 41 | 1591 | 1659 (1659) | v641 (v641) | 1421 | dormant (-91 ± 40 vs peak v641 before dormancy at v1115, at 1513 vs refs; needs > +40) | 1610 (134514) | +406 -425 =169, 49.0%, -7 ± 20: reverted | 1117 |
| T-L-NNs-c | qualified | 1073 | 45 x 1000 | 241 | 1547 | 41 | 1571 | 1611 | v561 | 1401 | strike 1 (-64 ± 47 vs peak v561; predecessor -25) | 1610 (138257) | +429 -417 =154, 50.6%, +4 ± 20: kept | 1074 |
| T-L-NNs-merge-c | dormant | 1069 |  |  | 1712 | 46 | 1696 | 1714 (1711) | v1067 (v818) | 1566 | dormant (+2 ± 34 vs peak v818 before dormancy at v1062, at 1565 vs refs; needs > +34) | 1610 (134208) | +398 -459 =143, 46.9%, -21 ± 20: reverted | 1069 |
| T-L-Le-c | dormant | 985 | 43 x 1000 | 238 | 1515 | 40 | 1490 | 1595 | v696 | 1369 | dormant (strike 2; -80 ± 44 vs peak v696; predecessor +25) | 1610 (128794) | +498 -366 =136, 56.6%, +46 ± 20: kept | 986 |
| T-L-Le-merge-c | dormant | 2052 |  |  | 1696 | 45 | 1713 | 1713 (1699) | v2051 (v1857) | 1550 | dormant (-3 ± 32 vs peak v1857 before dormancy at v2049, at 1553 vs refs; needs > +32) | 1610 (128988) | +407 -450 =143, 47.9%, -15 ± 20: reverted | 2052 |
| T-N-Ne-c | dormant | 637 |  |  | 1281 | 28 |  | 1378 (1378) | v574 (v574) | 1135 | dormant (-97 ± 26 vs peak v574 before dormancy at v633, at 1232 vs refs; needs > +26) | 1610 (138701) | +442 -447 =111, 49.8%, -2 ± 20: reverted | 637 |
| T-N-Ne-merge-c | dormant | 904 |  |  | 1393 | 29 |  | 1454 (1454) | v699 (v699) | 1247 | dormant (-61 ± 21 vs peak v699 before dormancy at v904, at 1307 vs refs; needs > +21) | 1610 (131525) | +410 -401 =189, 50.4%, +3 ± 19: kept | 905 |
| T-N-Nc-c | qualified | 634 | 41 x 1000 | 241 | 1357 | 38 | 1344 | 1396 | v472 | 1211 | ok (-39 ± 46 vs peak v472; predecessor +13) | 1610 (131141) | +398 -442 =160, 47.8%, -15 ± 20: reverted | 634 |
| T-N-Nc-merge-c | qualified | 1015 | 95 x 1000 | 274 | 1447 | 39 | 1437 | 1456 | v631 | 1301 | ok (-9 ± 37 vs peak v631; predecessor +10) | 1610 (130228) | +402 -418 =180, 49.2%, -6 ± 20: reverted | 1015 |
| T-N-NNs-c | dormant | 627 |  |  | 1121 | 37 | 1144 | 1244 (1244) | v341 (v341) | 975 | dormant (-123 ± 39 vs peak v341 before dormancy at v625, at 1098 vs refs; needs > +39) | 1610 (142847) | +430 -405 =165, 51.2%, +9 ± 20: kept | 628 |
| T-N-NNs-merge-c | dormant | 999 |  |  | 1366 | 38 | 1376 | 1474 (1474) | v855 (v855) | 1220 | dormant (-107 ± 29 vs peak v855 before dormancy at v997, at 1328 vs refs; needs > +29) | 1610 (132655) | +372 -397 =231, 48.8%, -9 ± 19: reverted | 999 |
| T-N-Le-merge-c | dormant | 698 |  |  | 1389 | 39 | 1349 | 1405 (1405) | v580 (v580) | 1243 | dormant (-16 ± 27 vs peak v580 before dormancy at v697, at 1259 vs refs; needs > +27) | 1610 (130294) | +426 -413 =161, 50.6%, +4 ± 20: kept | 699 |
| T-N-Lc-merge-c | dormant | 366 |  |  | 1415 | 22 |  | 1445 (1445) | v287 (v287) | 1269 | dormant (-30 ± 22 vs peak v287 before dormancy at v364, at 1299 vs refs; needs > +22) | 1610 (131340) | +396 -432 =172, 48.2%, -12 ± 20: reverted | 366 |
| T-N-LLs-merge-c | dormant | 725 |  |  | 1136 | 37 | 1099 | 1378 (1378) | v585 (v585) | 990 | dormant (-242 ± 27 vs peak v585 before dormancy at v723, at 1232 vs refs; needs > +27) | 1610 (145065) | +350 -473 =177, 43.9%, -43 ± 20: reverted | 725 |
| T-N-NLs-merge-c | dormant | 786 |  |  | 1402 | 39 | 1063 | 1400 (1400) | v672 (v672) | 1256 | dormant (+2 ± 27 vs peak v672 before dormancy at v785, at 1254 vs refs; needs > +27) | 1610 (131769) | +401 -418 =181, 49.1%, -6 ± 20: reverted | 786 |
| T-N-LNs-merge-c | dormant | 415 |  |  | 1258 | 38 | 1243 | 1355 (1355) | v270 (v270) | 1112 | dormant (-97 ± 27 vs peak v270 before dormancy at v410, at 1209 vs refs; needs > +27) | 1610 (136658) | +428 -403 =169, 51.2%, +9 ± 20: kept | 416 |
| F-N-d2jit-c | qualified | 1135 | 131 x 40 | 240 | 1152 | 37 | 1116 | 1182 | v389 | 1006 | ok (-30 ± 40 vs peak v389; predecessor +36) | 1610 (143470) | +458 -357 =185, 55.0%, +35 ± 20: kept | 1136 |
| T-N-NNs-merge-d2jit-c | qualified | 820 | 103 x 40 | 242 | 1155 | 37 | 1176 | 1177 | v642 | 1009 | ok (-22 ± 39 vs peak v642; predecessor -21) | 1610 (141978) | +450 -375 =175, 53.8%, +26 ± 20: kept | 821 |
| F-L-d4mix-c | qualified | 60 | 8 x 10 | 246 | 951 | 36 | 871 | 871 | v51 | 805 | new peak (+80 ± 37 over v51; predecessor +80) | 1610 (169682) | +758 -112 =130, 82.3%, +267 ± 25: kept | 61 |
| MoE-d2jit-c | qualified | 191 | 54 x 40 | 241 | 1323 | 38 | 1181 | 1181 | v136 | 1177 | new peak (+142 ± 38 over v136; predecessor +142) | 1610 (150882) | +621 -212 =167, 70.5%, +151 ± 21: kept | 192 |
| T-L-H-c | dormant | 1523 |  |  | 1613 | 42 | 1578 | 1666 (1666) | v1245 (v1245) | 1467 | dormant (-53 ± 31 vs peak v1245 before dormancy at v1521, at 1520 vs refs; needs > +31) | 1610 (135334) | +391 -456 =153, 46.8%, -23 ± 20: reverted | 1523 |
| T-N-H-c | dormant | 1732 |  |  | 1303 | 38 | 1287 | 1367 (1367) | v1104 (v1104) | 1157 | dormant (-64 ± 30 vs peak v1104 before dormancy at v1728, at 1221 vs refs; needs > +30) | 1610 (134058) | +430 -420 =150, 50.5%, +4 ± 20: kept | 1733 |
