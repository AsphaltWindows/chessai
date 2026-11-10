# Cycle 23

**Changes from this cycle** (deployed after cycle 22, `staged-c22/`): three more fresh twins, from scratch (the user): T-L-Ne-fresh-c, T-L-Le-fresh-c and T-N-Nc-fresh-c, the recipes of T-L-Ne-merge-c, T-L-Le-merge-c and T-N-Nc-merge-c with the merge test T-L-Nc-fresh-c has (`--revise 2 --revise-on fresh`: each self-play round scored before training on it, a branch merged back only when worse than its shadow by more than 2 standard errors) on `chessai-resid4`, one factor each. Their own steps train and gate with `chessai-resid4` too (`own_games.binary_by_lineage`).

**Own-games step, pooled over the lineages: 40 gates, +17507 -16059 =6434, score 51.8%, Elo +12.6 ± 3.1; kept 21 of 40.**

All cycles so far: 816 gates, +367398 -317242 =131360, score 53.1%, Elo +21.4 ± 0.7; kept 548 of 816.

2026-11-07 04:39: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 52 players to retired.txt (their games stay in the fit), added 58.
Tournament: 5 double round-robins of the 87 active players (37410 games) in 6.0 min, 37410 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| F-L-d1jit-c | 9 of 10 | cycle 24 | 15–23 | 1019.1 | 1643.2 | retire |
| MoE-c | 9 of 10 | cycle 24 | 15–23 | 958.9 | 1693.6 | freeze 2 at 1700 |
| T-L-LLs-c | 9 of 10 | cycle 24 | 15–23 | 952.6 | 1609.1 | freeze 2 at 1600 |
| T-N-LNs-merge-c | 9 of 10 | cycle 24 | 15–23 | 695.0 | 1217.4 | freeze 1 at 700 |
| T-N-Lc-merge-c | 9 of 10 | cycle 24 | 15–23 | 752.7 | 1412.5 | retire |
| TAN-c | 9 of 10 | cycle 24 | 15–23 | 726.2 | 1217.4 | retire |
| F-N-c | 7 of 10 | cycle 26 | 17–23 | 734.6 | 1396.1 | freeze 2 at 1400 |
| T-L-Le-merge-c | 7 of 10 | cycle 26 | 17–23 | 977.4 | 1681.8 | retire |
| T-L-Nc-c | 7 of 10 | cycle 26 | 17–23 | 981.4 | 1606.5 | retire |
| F-L-d1play-c | 6 of 10 | cycle 27 | 18–23 | 1004.4 | 1539.1 | retire |
| T-L-H-c | 6 of 10 | cycle 27 | 18–23 | 989.5 | 1600.2 | retire |
| T-L-NLs-merge-c | 6 of 10 | cycle 27 | 18–23 | 931.5 | 1588.8 | retire |
| T-L-Ne-merge-c | 6 of 10 | cycle 27 | 18–23 | 973.0 | 1672.6 | retire |
| T-N-LLs-merge-c | 6 of 10 | cycle 27 | 18–23 | 627.2 | 1123.1 | retire |
| T-N-NNs-c | 6 of 10 | cycle 27 | 18–23 | 655.1 | 1130.4 | retire |
| T-N-Ne-merge-c | 5 of 10 | cycle 28 | 19–23 | 745.2 | 1394.3 | retire |
| F-L-d2mix-c | 4 of 10 | cycle 29 | 20–23 | 831.9 | 1719.2 | retire |
| T-L-LLs-merge-c | 4 of 10 | cycle 29 | 20–23 | 931.6 | 1528.7 | retire |
| T-L-Le-c | 4 of 10 | cycle 29 | 20–23 | 938.2 | 1553.7 | retire |
| T-L-Nc-merge-c | 4 of 10 | cycle 29 | 20–23 | 973.5 | 1646.1 | retire |
| F-L-d2play-c | 2 of 10 | cycle 31 | 22–23 | 814.9 | 1695.2 | retire |
| F-N-d2jit-c | 2 of 10 | cycle 31 | 22–23 | 614.3 | 1191.3 | freeze 2 at 1200 |
| F-L-d0soft-c | 1 of 10 | cycle 32 | 23–23 | 976.1 | 1597.8 | retire |
| T-N-Le-merge-c | 0 of 10 | cycle 33 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | qualified | 2860 |  |  | 1720 | 32 |  | 1697 (1690) | v2855 (v2247) | 1578 | re-qualified (+30 ± 28 vs peak v2247 before dormancy at v2854, at 1548 vs refs) | 1610 (128141) | +428 -403 =169, 51.2%, +9 ± 20: kept | 2861 |
| F-L-d0soft-c | dormant | 2362 |  |  | 1598 | 40 | 1554 | 1707 (1707) | v2014 (v2014) | 1456 | dormant (-109 ± 29 vs peak v2014 before dormancy at v2361, at 1565 vs refs; needs > +29) | 1610 (130503) | +457 -392 =151, 53.2%, +23 ± 20: kept | 2363 |
| F-L-d1jit-c | dormant | 1770 |  |  | 1657 | 42 | 1657 | 1699 (1699) | v1451 (v1451) | 1515 | dormant (-42 ± 31 vs peak v1451 before dormancy at v1763, at 1557 vs refs; needs > +31) | 1610 (130434) | +422 -434 =144, 49.4%, -4 ± 20: reverted | 1770 |
| F-L-d1play-c | dormant | 3005 |  |  | 1560 | 39 | 1575 | 1575 (1562) | v3004 (v1269) | 1418 | dormant (-1 ± 32 vs peak v1269 before dormancy at v2999, at 1419 vs refs; needs > +32) | 1610 (137713) | +469 -383 =148, 54.3%, +30 ± 20: kept | 3006 |
| F-L-d2play-c | dormant | 1621 |  |  | 1692 | 31 |  | 1756 (1756) | v1011 (v1011) | 1550 | dormant (-63 ± 24 vs peak v1011 before dormancy at v1620, at 1614 vs refs; needs > +24) | 1610 (133020) | +400 -410 =190, 49.5%, -4 ± 19: reverted | 1621 |
| F-L-d2mix-c | dormant | 1072 |  |  | 1699 | 43 | 1716 | 1747 (1747) | v910 (v910) | 1557 | dormant (-48 ± 32 vs peak v910 before dormancy at v1068, at 1605 vs refs; needs > +32) | 1610 (132298) | +417 -401 =182, 50.8%, +6 ± 20: kept | 1073 |
| F-L-d2jit-c | qualified | 3001 | 101 x 40 | 239 | 1764 | 46 | 1767 | 1801 | v1651 | 1621 | ok (-38 ± 53 vs peak v1651; predecessor -4) | 1610 (144658) | +378 -389 =233, 49.5%, -4 ± 19: reverted | 3001 |
| F-N-c | dormant | 1690 |  |  | 1368 | 37 | 1424 | 1478 (1478) | v1061 (v1061) | 1226 | dormant (-110 ± 28 vs peak v1061 before dormancy at v1683, at 1336 vs refs; needs > +28) | 1610 (132606) | +388 -420 =192, 48.4%, -11 ± 19: reverted | 1690 |
| MoE-c | dormant | 373 |  |  | 1726 | 32 |  | 1722 (1722) | v300 (v300) | 1584 | dormant (+4 ± 28 vs peak v300 before dormancy at v367, at 1580 vs refs; needs > +28) | 1610 (134098) | +371 -455 =174, 45.8%, -29 ± 20: reverted | 373 |
| FM-c | qualified | 555 | 19 x 1000 | 236 | 1399 | 37 | 1573 | 1573 | v535 | 1257 | strike 1 (-174 ± 40 vs peak v535; predecessor -174) | 1610 (141419) | +632 -254 =114, 68.9%, +138 ± 22: kept | 556 |
| TAN-c | dormant | 929 |  |  | 1213 | 14 |  | 1241 (1241) | v507 (v507) | 1071 | dormant (-28 ± 21 vs peak v507 before dormancy at v928, at 1099 vs refs; needs > +21) | 1610 (134844) | +421 -457 =122, 48.2%, -12 ± 20: reverted | 929 |
| T-L-Ne-c | qualified | 843 | 31 x 1000 | 240 | 1547 | 39 | 1593 | 1611 | v556 | 1404 | strike 1 (-64 ± 35 vs peak v556; predecessor -46) | 1610 (135889) | +472 -361 =167, 55.5%, +39 ± 20: kept | 844 |
| T-L-Ne-merge-c | dormant | 1013 |  |  | 1660 | 42 | 1682 | 1707 (1707) | v794 (v794) | 1518 | dormant (-47 ± 24 vs peak v794 before dormancy at v1008, at 1565 vs refs; needs > +24) | 1610 (129498) | +424 -431 =145, 49.6%, -2 ± 20: reverted | 1013 |
| T-L-Ne-fresh-c | qualified | 107 | 107 x 1000 | 241 | 1359 | 37 |  |  |  | 1217 | qualified (first rating) | 1610 (142576) | +622 -288 =90, 66.7%, +121 ± 22: kept | 108 |
| T-L-Nc-c | dormant | 716 |  |  | 1629 | 41 | 1631 | 1650 (1650) | v516 (v516) | 1487 | dormant (-21 ± 30 vs peak v516 before dormancy at v709, at 1508 vs refs; needs > +30) | 1610 (139308) | +427 -400 =173, 51.3%, +9 ± 20: kept | 717 |
| T-L-Nc-merge-c | dormant | 1881 |  |  | 1669 | 31 |  | 1731 (1731) | v1631 (v1631) | 1527 | dormant (-62 ± 28 vs peak v1631 before dormancy at v1878, at 1589 vs refs; needs > +28) | 1610 (128347) | +420 -426 =154, 49.7%, -2 ± 20: reverted | 1881 |
| T-L-Nc-fresh-c | qualified | 348 | 38 x 1000 | 244 | 1550 | 39 | 1494 | 1516 | v256 | 1408 | new peak (+35 ± 40 over v256; predecessor +56) | 1610 (136166) | +460 -364 =176, 54.8%, +34 ± 20: kept | 349 |
| T-L-LLs-c | dormant | 651 |  |  | 1613 | 19 |  | 1634 (1634) | v540 (v540) | 1471 | dormant (-21 ± 24 vs peak v540 before dormancy at v647, at 1492 vs refs; needs > +24) | 1610 (134999) | +389 -419 =192, 48.5%, -10 ± 19: reverted | 651 |
| T-L-LLs-merge-c | dormant | 1756 |  |  | 1542 | 39 | 1501 | 1610 (1610) | v1251 (v1251) | 1400 | dormant (-68 ± 29 vs peak v1251 before dormancy at v1754, at 1468 vs refs; needs > +29) | 1610 (143407) | +440 -411 =149, 51.4%, +10 ± 20: kept | 1757 |
| T-L-NLs-c | qualified | 1260 | 34 x 1000 | 243 | 1505 | 38 | 1559 | 1570 | v571 | 1363 | strike 1 (-65 ± 46 vs peak v571; predecessor -54) | 1610 (138604) | +376 -451 =173, 46.2%, -26 ± 20: reverted | 1260 |
| T-L-NLs-merge-c | dormant | 1119 |  |  | 1625 | 41 | 1601 | 1654 (1654) | v641 (v641) | 1482 | dormant (-29 ± 40 vs peak v641 before dormancy at v1115, at 1512 vs refs; needs > +40) | 1610 (129722) | +418 -432 =150, 49.3%, -5 ± 20: reverted | 1119 |
| T-L-NNs-c | qualified | 1236 | 31 x 1000 | 238 | 1572 | 39 | 1538 | 1607 | v561 | 1430 | ok (-35 ± 47 vs peak v561; predecessor +34) | 1610 (142577) | +404 -428 =168, 48.8%, -8 ± 20: reverted | 1236 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 975 | 22 |  | 1708 | v1067 | 833 | frozen |  |  | 1070 |
| T-L-Le-c | dormant | 988 |  |  | 1562 | 29 |  | 1591 (1591) | v696 (v696) | 1420 | dormant (-29 ± 25 vs peak v696 before dormancy at v985, at 1449 vs refs; needs > +25) | 1610 (136617) | +426 -407 =167, 50.9%, +7 ± 20: kept | 989 |
| T-L-Le-merge-c | dormant | 2052 |  |  | 1682 | 21 |  | 1708 (1695) | v2051 (v1857) | 1540 | dormant (-12 ± 25 vs peak v1857 before dormancy at v2049, at 1553 vs refs; needs > +25) | 1610 (131563) | +400 -450 =150, 47.5%, -17 ± 20: reverted | 2052 |
| T-L-Le-fresh-c | qualified | 95 | 95 x 1000 | 239 | 1339 | 37 |  |  |  | 1197 | qualified (first rating) | 1610 (140323) | +661 -260 =79, 70.0%, +148 ± 22: kept | 96 |
| T-N-Ne-c | frozen | 637 |  |  | 1286 | 17 |  | 1374 | v574 | 1144 | frozen |  |  | 637 |
| T-N-Ne-merge-c | dormant | 905 |  |  | 1400 | 21 |  | 1449 (1449) | v699 (v699) | 1258 | dormant (-50 ± 18 vs peak v699 before dormancy at v904, at 1307 vs refs; needs > +18) | 1610 (125879) | +452 -392 =156, 53.0%, +21 ± 20: kept | 906 |
| T-N-Nc-c | qualified | 777 | 30 x 1000 | 236 | 1369 | 37 | 1383 | 1392 | v472 | 1227 | ok (-23 ± 46 vs peak v472; predecessor -14) | 1610 (133099) | +410 -419 =171, 49.5%, -3 ± 20: reverted | 777 |
| T-N-Nc-merge-c | qualified | 1243 | 43 x 1000 | 242 | 1385 | 37 | 1421 | 1452 | v631 | 1243 | strike 1 (-67 ± 36 vs peak v631; predecessor -36) | 1610 (130441) | +420 -420 =160, 50.0%, -0 ± 20: kept | 1244 |
| T-N-Nc-fresh-c | qualified | 102 | 102 x 1000 | 240 | 1257 | 36 |  |  |  | 1115 | qualified (first rating) | 1610 (138969) | +408 -464 =128, 47.2%, -20 ± 20: reverted | 102 |
| T-N-NNs-c | dormant | 630 |  |  | 1150 | 36 | 1118 | 1239 (1239) | v341 (v341) | 1008 | dormant (-89 ± 39 vs peak v341 before dormancy at v625, at 1097 vs refs; needs > +39) | 1610 (148102) | +433 -383 =184, 52.5%, +17 ± 20: kept | 631 |
| T-N-Le-merge-c | dormant | 754 | 25 x 1000 | 238 | 1330 | 37 | 1356 | 1436 | v700 | 1187 | dormant (strike 2; -106 ± 39 vs peak v700; predecessor -27) | 1610 (134372) | +409 -465 =126, 47.2%, -20 ± 20: reverted | 754 |
| T-N-Lc-merge-c | dormant | 368 |  |  | 1404 | 37 | 1388 | 1441 (1441) | v287 (v287) | 1262 | dormant (-36 ± 28 vs peak v287 before dormancy at v364, at 1299 vs refs; needs > +28) | 1610 (134368) | +381 -418 =201, 48.1%, -13 ± 19: reverted | 368 |
| T-N-LLs-merge-c | dormant | 725 |  |  | 1124 | 18 |  | 1373 (1373) | v585 (v585) | 982 | dormant (-249 ± 21 vs peak v585 before dormancy at v723, at 1231 vs refs; needs > +21) | 1610 (144594) | +401 -456 =143, 47.2%, -19 ± 20: reverted | 725 |
| T-N-NLs-merge-c | qualified | 897 | 109 x 1000 | 239 | 1412 | 37 | 1425 | 1425 | v787 | 1270 | ok (-12 ± 34 vs peak v787; predecessor -12) | 1610 (128193) | +415 -413 =172, 50.1%, +1 ± 20: kept | 898 |
| T-N-LNs-merge-c | dormant | 416 |  |  | 1258 | 20 |  | 1350 (1350) | v270 (v270) | 1116 | dormant (-92 ± 22 vs peak v270 before dormancy at v410, at 1208 vs refs; needs > +22) | 1610 (134432) | +421 -384 =195, 51.8%, +13 ± 19: kept | 417 |
| F-N-d2jit-c | dormant | 1399 |  |  | 1202 | 36 | 1180 | 1180 (1177) | v1398 (v389) | 1059 | dormant (+24 ± 26 vs peak v389 before dormancy at v1397, at 1035 vs refs; needs > +26) | 1610 (140078) | +444 -346 =210, 54.9%, +34 ± 19: kept | 1400 |
| T-N-NNs-merge-d2jit-c | qualified | 1201 | 93 x 40 | 239 | 1168 | 36 | 1177 | 1177 | v1107 | 1026 | ok (-8 ± 37 vs peak v1107; predecessor -8) | 1610 (141550) | +397 -402 =201, 49.8%, -2 ± 19: reverted | 1201 |
| F-L-d4mix-c | qualified | 97 | 8 x 10 | 238 | 1516 | 38 | 1440 | 1440 | v88 | 1374 | new peak (+76 ± 39 over v88; predecessor +76) | 1610 (138548) | +528 -337 =135, 59.6%, +67 ± 20: kept | 98 |
| MoE-d2jit-c | qualified | 400 | 47 x 40 | 243 | 1597 | 40 | 1578 | 1578 | v352 | 1455 | new peak (+19 ± 41 over v352; predecessor +19) | 1610 (142744) | +439 -426 =135, 50.6%, +4 ± 20: kept | 401 |
| T-L-H-c | dormant | 1524 |  |  | 1600 | 40 | 1597 | 1661 (1661) | v1245 (v1245) | 1458 | dormant (-61 ± 30 vs peak v1245 before dormancy at v1521, at 1519 vs refs; needs > +30) | 1610 (133129) | +427 -408 =165, 50.9%, +7 ± 20: kept | 1525 |
