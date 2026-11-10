# Cycle 16

**Changes from this cycle** (deployed after cycle 15, `staged-c15/`): the old dormancy rule undone (the user): the 10 lineages whose dormancy began at its decisions (cycles 5-8) were woken with `cycle.py wake` (qualified, strikes 0, no pre-dormancy bar) and self-play again, among them the 4 that cycle 15's freeze/retire rule took (F-L-d2mix-c, T-N-Nc-c and T-N-Ne-merge-c retired, T-N-Nc-merge-c frozen at 800), whose 7 entries came back to the field; the 12 dormant under the peak rule stay dormant. Lower thresholds from this cycle's decisions (the user): a strike at more than 1.5 standard errors below the peak (was 2), re-qualifying at more than 1.0 standard error above the pre-dormancy peak (was 2). Self-play in re-planned chunks (the user): after its measuring segment a job plays the rest in chunks of about 22% of the budget, each re-planned from the last chunk's CPU per game against the CPU used so far, so its cost drifting within the cycle (a tree as it grows, a model leaving its from-scratch start) no longer over- or undershoots the 240 CPU-s, which cycles 9-13 did by 153-467 CPU-s a job. And MoE-d2jit-c, from scratch (the user): MoE-c's recipe with --depth 2, to test whether self-play by depth-2 search stops the drops MoE-c shows on its own depth-0 self-play (README.md).

**Own-games step, pooled over the lineages: 40 gates, +17651 -15150 =7199, score 53.1%, Elo +21.8 ± 3.1; kept 25 of 40.**

All cycles so far: 538 gates, +245734 -206834 =85432, score 53.6%, Elo +25.2 ± 0.9; kept 375 of 538.

2026-11-07 01:38: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 72 players to retired.txt (their games stay in the fit), added 74.
Tournament: 5 double round-robins of the 85 active players (35700 games) in 4.3 min, 35700 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| F-L-d0soft-c | 5 of 10 | cycle 21 | 12–16 | 1011.3 | 1636.9 | retire |
| T-L-NNs-merge-c | 5 of 10 | cycle 21 | 12–16 | 1009.0 | 1684.6 | freeze 1 at 1000 |
| T-N-Ne-c | 5 of 10 | cycle 21 | 12–16 | 742.0 | 1311.7 | retire |
| T-L-Nc-merge-c | 4 of 10 | cycle 22 | 13–16 | 1012.6 | 1660.3 | retire |
| T-N-H-c | 4 of 10 | cycle 22 | 13–16 | 754.5 | 1287.2 | retire |
| T-N-NNs-merge-c | 4 of 10 | cycle 22 | 13–16 | 761.4 | 1379.3 | retire |
| F-L-d1jit-c | 2 of 10 | cycle 24 | 15–16 | 1047.5 | 1642.2 | retire |
| MoE-c | 2 of 10 | cycle 24 | 15–16 | 968.2 | 1691.3 | freeze 2 at 1700 |
| T-L-LLs-c | 2 of 10 | cycle 24 | 15–16 | 958.7 | 1598.1 | freeze 2 at 1600 |
| T-N-LNs-merge-c | 2 of 10 | cycle 24 | 15–16 | 663.3 | 1104.1 | freeze 2 at 1100 |
| T-N-Lc-merge-c | 2 of 10 | cycle 24 | 15–16 | 767.5 | 1425.3 | retire |
| TAN-c | 2 of 10 | cycle 24 | 15–16 | 732.1 | 1218.0 | retire |
| F-L-c | 0 of 10 | cycle 26 | - | - | - | retire |
| F-N-c | 0 of 10 | cycle 26 | - | - | - | retire |
| T-L-Le-merge-c | 0 of 10 | cycle 26 | - | - | - | retire |
| T-L-Nc-c | 0 of 10 | cycle 26 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, 400 random-3, random-4.


Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | dormant | 2854 | 123 x 1000 | 240 | 1640 | 44 | 1609 | 1703 | v2247 | 1492 | dormant (strike 2; -62 ± 47 vs peak v2247; predecessor +32) | 1570 (128055) | +471 -364 =165, 55.4%, +37 ± 20: kept | 2855 |
| F-L-d0soft-c | dormant | 2011 |  |  | 1649 | 44 | 1634 | 1663 (1663) | v1619 (v1619) | 1500 | dormant (-14 ± 32 vs peak v1619 before dormancy at v2006, at 1514 vs refs; needs > +32) | 1570 (126645) | +443 -394 =163, 52.4%, +17 ± 20: kept | 2012 |
| F-L-d1jit-c | dormant | 1765 |  |  | 1671 | 45 | 1614 | 1711 (1711) | v1451 (v1451) | 1522 | dormant (-40 ± 32 vs peak v1451 before dormancy at v1763, at 1562 vs refs; needs > +32) | 1570 (131277) | +397 -442 =161, 47.8%, -16 ± 20: reverted | 1765 |
| F-L-d1play-c | qualified | 2865 | 133 x 860 | 240 | 1517 | 42 | 1523 | 1572 | v1269 | 1368 | strike 1 (-55 ± 48 vs peak v1269; predecessor -6) | 1570 (146777) | +474 -379 =147, 54.8%, +33 ± 20: kept | 2866 |
| F-L-d2play-c | qualified | 1106 | 93 x 40 | 239 | 1724 | 47 | 1738 | 1767 | v1011 | 1575 | ok (-44 ± 45 vs peak v1011; predecessor -15) | 1570 (134584) | +386 -432 =182, 47.7%, -16 ± 20: reverted | 1106 |
| F-L-d2mix-c | qualified | 830 | 79 x 40 | 242 | 1694 | 46 | 1727 | 1745 | v591 | 1545 | ok (-52 ± 65 vs peak v591; predecessor -34) | 1570 (133216) | +452 -317 =231, 56.8%, +47 ± 19: kept | 831 |
| F-L-d2jit-c | qualified | 2208 | 114 x 40 | 240 | 1786 | 51 | 1773 | 1813 | v1651 | 1638 | ok (-26 ± 55 vs peak v1651; predecessor +14) | 1570 (145191) | +363 -450 =187, 45.6%, -30 ± 20: reverted | 2208 |
| F-N-c | dormant | 1683 | 123 x 1000 | 240 | 1430 | 41 | 1425 | 1490 | v1061 | 1281 | dormant (strike 2; -60 ± 43 vs peak v1061; predecessor +5) | 1570 (127717) | +443 -389 =168, 52.7%, +19 ± 20: kept | 1684 |
| MoE-c | dormant | 369 |  |  | 1735 | 48 | 1648 | 1735 (1735) | v300 (v300) | 1587 | dormant (+1 ± 33 vs peak v300 before dormancy at v367, at 1586 vs refs; needs > +33) | 1570 (126062) | +407 -427 =166, 49.0%, -7 ± 20: reverted | 369 |
| FM-c | qualified | 419 | 19 x 1000 | 237 | 1463 | 41 | 1321 | 1383 | v331 | 1314 | new peak (+80 ± 42 over v331; predecessor +142) | 1570 (142698) | +536 -317 =147, 61.0%, +77 ± 20: kept | 420 |
| TAN-c | dormant | 929 |  |  | 1225 | 28 |  | 1253 (1253) | v507 (v507) | 1077 | dormant (-27 ± 24 vs peak v507 before dormancy at v928, at 1104 vs refs; needs > +24) | 1570 (135795) | +383 -496 =121, 44.4%, -39 ± 20: reverted | 929 |
| T-L-Ne-c | qualified | 588 | 30 x 1000 | 245 | 1596 | 43 | 1610 | 1622 | v556 | 1448 | ok (-26 ± 37 vs peak v556; predecessor -14) | 1570 (133716) | +483 -371 =146, 55.6%, +39 ± 20: kept | 589 |
| T-L-Ne-merge-c | qualified | 898 | 104 x 1000 | 249 | 1632 | 44 | 1720 | 1720 | v794 | 1483 | strike 1 (-89 ± 37 vs peak v794; predecessor -89) | 1570 (125423) | +515 -349 =136, 58.3%, +58 ± 20: kept | 899 |
| T-L-Nc-c | dormant | 709 | 37 x 1000 | 239 | 1575 | 43 | 1589 | 1661 | v516 | 1426 | dormant (strike 2; -87 ± 46 vs peak v516; predecessor -15) | 1570 (135305) | +430 -398 =172, 51.6%, +11 ± 20: kept | 710 |
| T-L-Nc-merge-c | dormant | 1630 |  |  | 1660 | 45 | 1670 | 1702 (1702) | v833 (v833) | 1512 | dormant (-42 ± 34 vs peak v833 before dormancy at v1626, at 1554 vs refs; needs > +34) | 1570 (126404) | +451 -400 =149, 52.5%, +18 ± 20: kept | 1631 |
| T-L-LLs-c | dormant | 649 |  |  | 1585 | 43 | 1612 | 1646 (1646) | v540 (v540) | 1436 | dormant (-61 ± 30 vs peak v540 before dormancy at v647, at 1497 vs refs; needs > +30) | 1570 (135060) | +445 -357 =198, 54.4%, +31 ± 19: kept | 650 |
| T-L-LLs-merge-c | qualified | 1595 | 67 x 1000 | 242 | 1604 | 43 | 1607 | 1621 | v1251 | 1455 | ok (-18 ± 46 vs peak v1251; predecessor -3) | 1570 (129828) | +397 -427 =176, 48.5%, -10 ± 20: reverted | 1595 |
| T-L-NLs-c | qualified | 962 | 46 x 1000 | 243 | 1556 | 42 | 1513 | 1581 | v571 | 1407 | ok (-26 ± 47 vs peak v571; predecessor +42) | 1570 (133791) | +431 -394 =175, 51.8%, +13 ± 20: kept | 963 |
| T-L-NLs-merge-c | qualified | 1060 | 66 x 1000 | 242 | 1595 | 43 | 1570 | 1661 | v641 | 1446 | strike 1 (-66 ± 61 vs peak v641; predecessor +25) | 1570 (128712) | +397 -448 =155, 47.4%, -18 ± 20: reverted | 1060 |
| T-L-NNs-c | qualified | 939 | 42 x 1000 | 240 | 1607 | 43 | 1588 | 1618 | v561 | 1458 | ok (-11 ± 48 vs peak v561; predecessor +18) | 1570 (135759) | +393 -430 =177, 48.1%, -13 ± 20: reverted | 939 |
| T-L-NNs-merge-c | dormant | 1066 |  |  | 1672 | 32 |  | 1718 (1718) | v818 (v818) | 1524 | dormant (-46 ± 29 vs peak v818 before dormancy at v1062, at 1569 vs refs; needs > +29) | 1570 (126779) | +462 -390 =148, 53.6%, +25 ± 20: kept | 1067 |
| T-L-Le-c | qualified | 855 | 39 x 1000 | 240 | 1524 | 42 | 1554 | 1603 | v696 | 1375 | strike 1 (-79 ± 44 vs peak v696; predecessor -30) | 1570 (126381) | +529 -356 =115, 58.7%, +61 ± 21: kept | 856 |
| T-L-Le-merge-c | dormant | 2049 | 113 x 1000 | 245 | 1606 | 43 | 1630 | 1707 | v1857 | 1457 | dormant (strike 2; -102 ± 47 vs peak v1857; predecessor -24) | 1570 (129945) | +543 -299 =158, 62.2%, +86 ± 20: kept | 2050 |
| T-N-Ne-c | dormant | 635 |  |  | 1303 | 40 | 1328 | 1385 (1385) | v574 (v574) | 1154 | dormant (-82 ± 30 vs peak v574 before dormancy at v633, at 1236 vs refs; needs > +30) | 1570 (134288) | +440 -440 =120, 50.0%, -0 ± 20: kept | 636 |
| T-N-Ne-merge-c | qualified | 773 | 70 x 1000 | 240 | 1430 | 41 | 1401 | 1461 | v699 | 1281 | ok (-31 ± 38 vs peak v699; predecessor +29) | 1570 (129601) | +399 -423 =178, 48.8%, -8 ± 20: reverted | 773 |
| T-N-Nc-c | qualified | 514 | 39 x 1000 | 238 | 1401 | 41 | 1393 | 1403 | v472 | 1252 | ok (-2 ± 47 vs peak v472; predecessor +8) | 1570 (127998) | +413 -435 =152, 48.9%, -8 ± 20: reverted | 514 |
| T-N-Nc-merge-c | qualified | 748 | 116 x 1000 | 250 | 1470 | 41 | 1454 | 1464 | v631 | 1321 | new peak (+6 ± 38 over v631; predecessor +16) | 1570 (124931) | +399 -400 =201, 50.0%, -0 ± 19: reverted | 748 |
| T-N-NNs-c | qualified | 581 | 41 x 1000 | 241 | 1184 | 39 | 1191 | 1247 | v452 | 1035 | strike 1 (-63 ± 46 vs peak v452; predecessor -7) | 1570 (138711) | +434 -368 =198, 53.3%, +23 ± 19: kept | 582 |
| T-N-NNs-merge-c | dormant | 998 |  |  | 1386 | 21 |  | 1481 (1481) | v855 (v855) | 1238 | dormant (-95 ± 24 vs peak v855 before dormancy at v997, at 1332 vs refs; needs > +24) | 1570 (131212) | +392 -393 =215, 50.0%, -0 ± 19: reverted | 998 |
| T-N-Le-merge-c | qualified | 617 | 36 x 1000 | 240 | 1388 | 41 | 1413 | 1413 | v580 | 1239 | ok (-25 ± 41 vs peak v580; predecessor -25) | 1570 (127041) | +463 -367 =170, 54.8%, +34 ± 20: kept | 618 |
| T-N-Lc-merge-c | dormant | 366 |  |  | 1445 | 41 | 1406 | 1453 (1453) | v287 (v287) | 1296 | dormant (-8 ± 28 vs peak v287 before dormancy at v364, at 1304 vs refs; needs > +28) | 1570 (125372) | +416 -418 =166, 49.9%, -1 ± 20: reverted | 366 |
| T-N-LLs-merge-c | qualified | 672 | 87 x 1000 | 240 | 1288 | 40 | 1386 | 1386 | v585 | 1139 | strike 1 (-99 ± 41 vs peak v585; predecessor -99) | 1570 (129858) | +459 -366 =175, 54.6%, +32 ± 20: kept | 673 |
| T-N-NLs-merge-c | qualified | 672 | 86 x 1000 | 245 | 1414 | 41 | 1026 | 1378 | v287 | 1265 | new peak (+36 ± 42 over v287; predecessor +388) | 1570 (130169) | +411 -431 =158, 49.0%, -7 ± 20: reverted | 672 |
| T-N-LNs-merge-c | dormant | 412 |  |  | 1140 | 39 | 1069 | 1362 (1362) | v270 (v270) | 992 | dormant (-222 ± 27 vs peak v270 before dormancy at v410, at 1214 vs refs; needs > +27) | 1570 (137795) | +472 -362 =166, 55.5%, +38 ± 20: kept | 413 |
| F-N-d2jit-c | qualified | 748 | 125 x 40 | 240 | 1180 | 39 | 1152 | 1190 | v389 | 1032 | ok (-10 ± 40 vs peak v389; predecessor +29) | 1570 (141588) | +461 -334 =205, 56.4%, +44 ± 19: kept | 749 |
| T-N-NNs-merge-d2jit-c | qualified | 551 | 100 x 40 | 240 | 1167 | 39 | 1112 | 1112 | v450 | 1018 | new peak (+55 ± 39 over v450; predecessor +55) | 1570 (136287) | +453 -349 =198, 55.2%, +36 ± 19: kept | 552 |
| F-L-d4mix-c | qualified | 34 | 10 x 10 | 239 | 656 | 42 | 598 | 598 | v24 | 507 | new peak (+58 ± 45 over v24; predecessor +58) | 1570 (145970) | +348 -184 =468, 58.2%, +58 ± 16: kept | 35 |
| MoE-d2jit-c | qualified | 32 | 32 x 40 | 243 | 571 | 45 |  |  |  | 422 | qualified (first rating) | 1570 (143971) | +520 -55 =425, 73.2%, +175 ± 16: kept | 33 |
| T-L-H-c | qualified | 1366 | 120 x 1000 | 242 | 1621 | 44 | 1634 | 1674 | v1245 | 1472 | strike 1 (-53 ± 46 vs peak v1245; predecessor -14) | 1570 (129375) | +524 -352 =124, 58.6%, +60 ± 20: kept | 1367 |
| T-N-H-c | dormant | 1731 |  |  | 1297 | 40 | 1302 | 1374 (1374) | v1104 (v1104) | 1148 | dormant (-77 ± 30 vs peak v1104 before dormancy at v1728, at 1225 vs refs; needs > +30) | 1570 (132610) | +416 -447 =137, 48.4%, -11 ± 20: reverted | 1731 |
