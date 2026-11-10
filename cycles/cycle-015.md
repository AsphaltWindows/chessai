# Cycle 15

**Changes from this cycle** (deployed after cycle 14, `staged-c14/`): the automatic freeze/retire rule (the user). From this cycle's decision on, a lineage dormant 10 cycles in a row is frozen at the ply whose mean Elo over those cycles is within 10 of an untaken multiple of 100 (it claims that milestone; the reference players hold theirs), or else retired: status retired in state.json, the driver skips it, and `cycle.py unretire NAME` brings it back. No exemptions (the user: F-L-d2mix-c too). Its preview is below and in `cycle.py status`; README.md has the details.

**Own-games step, pooled over the lineages: 35 gates, +15210 -13893 =5897, score 51.9%, Elo +13.1 ± 3.3; kept 21 of 35.**

All cycles so far: 498 gates, +228083 -191684 =78233, score 53.7%, Elo +25.4 ± 0.9; kept 350 of 498.

2026-11-07 00:53: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then ran the rounds that fill the budget; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 64 players to retired.txt (their games stay in the fit), added 64.
Tournament: 5 double round-robins of the 83 active players (34030 games) in 4.2 min, 34030 games saved.
Freeze/retire rule, after 10 cycles dormant: F-L-d2mix-c retired (avg ply 1 831.8, ply 2 1767.1; no untaken milestone within 10); T-N-Nc-c retired (avg ply 1 774.1, ply 2 1420.1; no untaken milestone within 10); T-N-Nc-merge-c frozen at ply 1 (avg 798.7 over cycles 6–15; milestone 800); T-N-Ne-merge-c retired (avg ply 1 790.4, ply 2 1468.4; no untaken milestone within 10). 7 entries leave the field for retired.txt (their games stay in the fit): 83 players become 76, 34,030 games a cycle become 28,500.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| F-L-d2play-c | 9 of 10 | cycle 16 | 7–15 | 860.3 | 1773.3 | retire |
| T-L-Ne-merge-c | 9 of 10 | cycle 16 | 7–15 | 1022.1 | 1725.9 | retire |
| T-L-H-c | 7 of 10 | cycle 18 | 9–15 | 1038.6 | 1665.8 | retire |
| T-L-NLs-merge-c | 7 of 10 | cycle 18 | 9–15 | 973.1 | 1594.2 | freeze 2 at 1600 |
| T-L-Ne-c | 7 of 10 | cycle 18 | 9–15 | 1023.3 | 1636.8 | retire |
| T-N-NNs-c | 7 of 10 | cycle 18 | 9–15 | 682.8 | 1196.3 | freeze 2 at 1200 |
| F-L-d0soft-c | 4 of 10 | cycle 21 | 12–15 | 1013.8 | 1633.8 | retire |
| T-L-NNs-merge-c | 4 of 10 | cycle 21 | 12–15 | 1011.6 | 1687.7 | retire |
| T-N-Ne-c | 4 of 10 | cycle 21 | 12–15 | 741.8 | 1313.8 | retire |
| T-L-Nc-merge-c | 3 of 10 | cycle 22 | 13–15 | 1016.8 | 1660.3 | retire |
| T-N-H-c | 3 of 10 | cycle 22 | 13–15 | 753.9 | 1284.0 | retire |
| T-N-NNs-merge-c | 3 of 10 | cycle 22 | 13–15 | 762.5 | 1377.0 | retire |
| F-L-d1jit-c | 1 of 10 | cycle 24 | 15–15 | 1051.0 | 1613.5 | retire |
| MoE-c | 1 of 10 | cycle 24 | 15–15 | 955.6 | 1647.3 | retire |
| T-L-LLs-c | 1 of 10 | cycle 24 | 15–15 | 968.8 | 1611.3 | retire |
| T-N-LNs-merge-c | 1 of 10 | cycle 24 | 15–15 | 655.8 | 1067.7 | retire |
| T-N-Lc-merge-c | 1 of 10 | cycle 24 | 15–15 | 774.0 | 1405.5 | freeze 2 at 1400 |
| TAN-c | 1 of 10 | cycle 24 | 15–15 | 729.4 | 1210.6 | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, 400 random-3, random-4, 800 T-N-Nc-merge-c-v632-1.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 2 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 2 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | qualified | 2730 | 194 x 1000 | 376 | 1608 | 42 | 1672 | 1702 | v2247 | 1457 | strike 1 (-94 ± 60 vs peak v2247; predecessor -64) | 1510 (124419) | +501 -359 =140, 57.1%, +50 ± 20: kept | 2731 |
| F-L-d0soft-c | dormant | 2010 |  |  | 1634 | 43 | 1651 | 1663 (1663) | v1619 (v1619) | 1483 | dormant (-30 ± 63 vs peak v1619 before dormancy at v2006, at 1513 vs refs; needs > +63) | 1510 (126984) | +453 -408 =139, 52.2%, +16 ± 20: kept | 2011 |
| F-L-d1jit-c | dormant | 1764 |  |  | 1614 | 42 | 1561 | 1711 (1711) | v1451 (v1451) | 1463 | dormant (-98 ± 60 vs peak v1451 before dormancy at v1763, at 1561 vs refs; needs > +60) | 1510 (131121) | +484 -378 =138, 55.3%, +37 ± 20: kept | 1765 |
| F-L-d1play-c | qualified | 2731 | 224 x 860 | 401 | 1522 | 40 | 1520 | 1573 | v1269 | 1372 | ok (-51 ± 63 vs peak v1269; predecessor +2) | 1510 (139376) | +471 -377 =152, 54.7%, +33 ± 20: kept | 2732 |
| F-L-d2play-c | dormant | 1013 |  |  | 1738 | 47 | 1743 | 1767 (1741) | v1011 (v818) | 1587 | dormant (-4 ± 85 vs peak v818 before dormancy at v1009, at 1591 vs refs; needs > +85) | 1510 (129588) | +372 -454 =174, 45.9%, -29 ± 20: reverted | 1013 |
| F-L-d2mix-c | retired | 751 |  |  | 1727 | 46 | 1728 | 1747 (1747) | v591 (v591) | 1576 | retired (avg ply 1 831.8, ply 2 1767.1; no untaken milestone within 10); dormant (-20 ± 87 vs peak v591 before dormancy at v750, at 1597 vs refs; needs > +87) |  |  | 751 |
| F-L-d2jit-c | qualified | 2094 | 100 x 40 | 224 | 1772 | 49 | 1787 | 1813 | v1651 | 1622 | ok (-41 ± 71 vs peak v1651; predecessor -15) | 1510 (131478) | +351 -467 =182, 44.2%, -40 ± 20: reverted | 2094 |
| F-N-c | qualified | 1560 | 120 x 1000 | 237 | 1424 | 39 | 1459 | 1490 | v1061 | 1273 | strike 1 (-66 ± 56 vs peak v1061; predecessor -35) | 1510 (120462) | +409 -433 =158, 48.8%, -8 ± 20: reverted | 1560 |
| MoE-c | dormant | 368 |  |  | 1647 | 43 | 1430 | 1735 (1735) | v300 (v300) | 1497 | dormant (-87 ± 62 vs peak v300 before dormancy at v367, at 1584 vs refs; needs > +62) | 1510 (122547) | +534 -333 =133, 60.1%, +71 ± 20: kept | 369 |
| FM-c | qualified | 399 | 20 x 1000 | 230 | 1320 | 39 | 1330 | 1383 | v331 | 1170 | strike 1 (-62 ± 54 vs peak v331; predecessor -10) | 1510 (138225) | +670 -181 =149, 74.5%, +186 ± 22: kept | 400 |
| TAN-c | dormant | 929 |  |  | 1211 | 38 | 1197 | 1253 (1253) | v507 (v507) | 1060 | dormant (-42 ± 54 vs peak v507 before dormancy at v928, at 1102 vs refs; needs > +54) | 1510 (132016) | +414 -458 =128, 47.8%, -15 ± 20: reverted | 929 |
| T-L-Ne-c | dormant | 557 |  |  | 1610 | 26 |  | 1622 (1616) | v556 (v483) | 1459 | dormant (-6 ± 55 vs peak v483 before dormancy at v555, at 1465 vs refs; needs > +55) | 1510 (128722) | +449 -417 =134, 51.6%, +11 ± 20: kept | 558 |
| T-L-Ne-merge-c | dormant | 794 |  |  | 1720 | 25 |  | 1718 (1668) | v792 (v700) | 1569 | dormant (+52 ± 57 vs peak v700 before dormancy at v790, at 1518 vs refs; needs > +57) | 1510 (120615) | +430 -437 =133, 49.6%, -2 ± 20: reverted | 794 |
| T-L-Nc-c | qualified | 671 | 35 x 1000 | 213 | 1588 | 42 | 1629 | 1662 | v516 | 1438 | strike 1 (-73 ± 60 vs peak v516; predecessor -41) | 1510 (132798) | +448 -396 =156, 52.6%, +18 ± 20: kept | 672 |
| T-L-Nc-merge-c | dormant | 1629 |  |  | 1670 | 44 | 1670 | 1703 (1703) | v833 (v833) | 1519 | dormant (-33 ± 68 vs peak v833 before dormancy at v1626, at 1552 vs refs; needs > +68) | 1510 (120315) | +437 -407 =156, 51.5%, +10 ± 20: kept | 1630 |
| T-L-LLs-c | dormant | 648 |  |  | 1611 | 42 | 1559 | 1646 (1646) | v540 (v540) | 1461 | dormant (-35 ± 60 vs peak v540 before dormancy at v647, at 1496 vs refs; needs > +60) | 1510 (128131) | +455 -383 =162, 53.6%, +25 ± 20: kept | 649 |
| T-L-LLs-merge-c | qualified | 1528 | 46 x 1000 | 185 | 1606 | 42 | 1600 | 1621 | v1251 | 1455 | ok (-16 ± 59 vs peak v1251; predecessor +5) | 1510 (125004) | +385 -462 =153, 46.2%, -27 ± 20: reverted | 1528 |
| T-L-NLs-c | qualified | 916 | 46 x 1000 | 276 | 1512 | 40 | 1557 | 1582 | v571 | 1362 | strike 1 (-70 ± 61 vs peak v571; predecessor -45) | 1510 (128751) | +345 -464 =191, 44.0%, -42 ± 20: reverted | 916 |
| T-L-NLs-merge-c | dormant | 994 |  |  | 1570 | 18 |  | 1663 (1663) | v641 (v641) | 1420 | dormant (-93 ± 71 vs peak v641 before dormancy at v994, at 1512 vs refs; needs > +71) | 1510 (126013) | +323 -479 =198, 42.2%, -55 ± 20: reverted | 994 |
| T-L-NNs-c | qualified | 897 | 40 x 1000 | 246 | 1588 | 42 | 1576 | 1618 | v561 | 1437 | ok (-31 ± 62 vs peak v561; predecessor +12) | 1510 (133835) | +411 -414 =175, 49.9%, -1 ± 20: reverted | 897 |
| T-L-NNs-merge-c | dormant | 1066 |  |  | 1677 | 44 | 1711 | 1718 (1718) | v818 (v818) | 1526 | dormant (-42 ± 66 vs peak v818 before dormancy at v1062, at 1568 vs refs; needs > +66) | 1510 (125685) | +420 -422 =158, 49.9%, -1 ± 20: reverted | 1066 |
| T-L-Le-c | qualified | 815 | 36 x 1000 | 220 | 1553 | 41 | 1588 | 1603 | v696 | 1402 | ok (-50 ± 57 vs peak v696; predecessor -35) | 1510 (121433) | +510 -376 =114, 56.7%, +47 ± 20: kept | 816 |
| T-L-Le-merge-c | qualified | 1935 | 77 x 1000 | 198 | 1629 | 43 | 1707 | 1707 | v1857 | 1479 | strike 1 (-78 ± 61 vs peak v1857; predecessor -78) | 1510 (124175) | +495 -409 =96, 54.3%, +30 ± 21: kept | 1936 |
| T-N-Ne-c | dormant | 634 |  |  | 1327 | 39 | 1304 | 1385 (1385) | v574 (v574) | 1176 | dormant (-58 ± 58 vs peak v574 before dormancy at v633, at 1235 vs refs; needs > +58) | 1510 (128835) | +466 -408 =126, 52.9%, +20 ± 20: kept | 635 |
| T-N-Ne-merge-c | retired | 703 |  |  | 1400 | 39 | 1426 | 1461 (1425) | v699 (v500) | 1249 | retired (avg ply 1 790.4, ply 2 1468.4; no untaken milestone within 10); dormant (-25 ± 80 vs peak v500 before dormancy at v695, at 1274 vs refs; needs > +80) |  |  | 703 |
| T-N-Nc-c | retired | 475 |  |  | 1393 | 22 |  | 1404 (1379) | v472 (v390) | 1242 | retired (avg ply 1 774.1, ply 2 1420.1; no untaken milestone within 10); dormant (+14 ± 73 vs peak v390 before dormancy at v471, at 1228 vs refs; needs > +73) |  |  | 475 |
| T-N-Nc-merge-c | frozen | 632 |  |  | 1454 | 25 |  | 1464 (1456) | v631 (v628) | 1303 | frozen at ply 1 (avg 798.7 over cycles 6–15; milestone 800); dormant (-2 ± 42 vs peak v628 before dormancy at v628, at 1305 vs refs; needs > +42) |  |  | 632 |
| T-N-NNs-c | dormant | 539 |  |  | 1191 | 21 |  | 1248 (1248) | v341 (v341) | 1040 | dormant (-58 ± 72 vs peak v341 before dormancy at v536, at 1098 vs refs; needs > +72) | 1510 (129347) | +411 -397 =192, 50.7%, +5 ± 19: kept | 540 |
| T-N-NNs-merge-c | dormant | 998 |  |  | 1384 | 24 |  | 1481 (1481) | v855 (v855) | 1233 | dormant (-98 ± 48 vs peak v855 before dormancy at v997, at 1331 vs refs; needs > +48) | 1510 (125245) | +362 -402 =236, 48.0%, -14 ± 19: reverted | 998 |
| T-N-Le-merge-c | qualified | 580 | 43 x 1000 | 277 | 1412 | 39 | 1395 | 1401 | v341 | 1262 | new peak (+11 ± 54 over v341; predecessor +17) | 1510 (124540) | +454 -395 =151, 52.9%, +20 ± 20: kept | 581 |
| T-N-Lc-merge-c | dormant | 365 |  |  | 1406 | 39 | 1397 | 1453 (1453) | v287 (v287) | 1255 | dormant (-47 ± 54 vs peak v287 before dormancy at v364, at 1302 vs refs; needs > +54) | 1510 (125725) | +431 -399 =170, 51.6%, +11 ± 20: kept | 366 |
| T-N-LLs-merge-c | qualified | 585 | 115 x 1000 | 433 | 1385 | 39 | 1384 | 1384 | v469 | 1235 | new peak (+2 ± 54 over v469; predecessor +2) | 1510 (123951) | +392 -423 =185, 48.4%, -11 ± 20: reverted | 585 |
| T-N-NLs-merge-c | qualified | 585 | 116 x 1000 | 419 | 1025 | 36 | 1377 | 1378 | v287 | 875 | strike 1 (-352 ± 52 vs peak v287; predecessor -352) | 1510 (139105) | +494 -369 =137, 56.2%, +44 ± 20: kept | 586 |
| T-N-LNs-merge-c | dormant | 411 |  |  | 1068 | 37 | 953 | 1362 (1362) | v270 (v270) | 917 | dormant (-295 ± 52 vs peak v270 before dormancy at v410, at 1212 vs refs; needs > +52) | 1510 (136498) | +518 -340 =142, 58.9%, +62 ± 20: kept | 412 |
| F-N-d2jit-c | qualified | 622 | 112 x 40 | 231 | 1151 | 37 | 1179 | 1190 | v389 | 1000 | ok (-39 ± 51 vs peak v389; predecessor -28) | 1510 (137319) | +442 -343 =215, 54.9%, +34 ± 19: kept | 623 |
| T-N-NNs-merge-d2jit-c | qualified | 450 | 126 x 40 | 284 | 1110 | 37 | 1102 | 1102 | v323 | 960 | new peak (+9 ± 51 over v323; predecessor +9) | 1510 (133506) | +505 -314 =181, 59.6%, +67 ± 20: kept | 451 |
| F-L-d4mix-c | qualified | 24 | 10 x 10 | 251 | 597 | 44 | 499 | 499 | v14 | 447 | new peak (+98 ± 65 over v14; predecessor +98) | 1510 (131774) | +99 -327 =574, 38.6%, -81 ± 14: reverted | 24 |
| T-L-H-c | dormant | 1246 |  |  | 1634 | 43 | 1673 | 1673 (1648) | v1245 (v970) | 1483 | dormant (-14 ± 66 vs peak v970 before dormancy at v1240, at 1497 vs refs; needs > +66) | 1510 (123886) | +429 -443 =128, 49.3%, -5 ± 20: reverted | 1246 |
| T-N-H-c | dormant | 1730 |  |  | 1301 | 38 | 1253 | 1375 (1375) | v1104 (v1104) | 1150 | dormant (-74 ± 59 vs peak v1104 before dormancy at v1728, at 1224 vs refs; needs > +59) | 1510 (125426) | +440 -419 =141, 51.0%, +7 ± 20: kept | 1731 |
