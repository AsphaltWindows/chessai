# Cycle 24

**Own-games step, pooled over the lineages: 34 gates, +14918 -13542 =5540, score 52.0%, Elo +14.1 ± 3.4; kept 22 of 34.**

All cycles so far: 850 gates, +382316 -330784 =136900, score 53.0%, Elo +21.1 ± 0.7; kept 570 of 850.

2026-11-07 05:04: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 54 players to retired.txt (their games stay in the fit), added 54.
Tournament: 5 double round-robins of the 87 active players (37410 games) in 4.7 min, 37410 games saved.
Freeze/retire rule, after 10 cycles dormant: F-L-d1jit-c retired (avg ply 1 1014.9, ply 2 1646.3; no untaken milestone within 10); MoE-c frozen at ply 2 (avg 1695.7 over cycles 15–24; milestone 1700); T-L-LLs-c frozen at ply 2 (avg 1609.3 over cycles 15–24; milestone 1600); T-N-LNs-merge-c frozen at ply 1 (avg 696.6 over cycles 15–24; milestone 700); T-N-Lc-merge-c retired (avg ply 1 752.5, ply 2 1412.2; no untaken milestone within 10); TAN-c retired (avg ply 1 725.2, ply 2 1216.7; no untaken milestone within 10). 9 entries leave the field for retired.txt (their games stay in the fit): 87 players become 78, 37,410 games a cycle become 30,030.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| F-N-c | 8 of 10 | cycle 26 | 17–24 | 733.7 | 1397.2 | freeze 2 at 1400 |
| T-L-Le-merge-c | 8 of 10 | cycle 26 | 17–24 | 976.8 | 1680.8 | retire |
| T-L-Nc-c | 8 of 10 | cycle 26 | 17–24 | 977.4 | 1605.2 | retire |
| T-L-H-c | 7 of 10 | cycle 27 | 18–24 | 988.7 | 1606.6 | retire |
| T-L-NLs-merge-c | 7 of 10 | cycle 27 | 18–24 | 929.5 | 1592.2 | retire |
| T-L-Ne-merge-c | 7 of 10 | cycle 27 | 18–24 | 972.7 | 1673.1 | retire |
| T-N-LLs-merge-c | 7 of 10 | cycle 27 | 18–24 | 626.2 | 1123.6 | retire |
| T-N-NNs-c | 7 of 10 | cycle 27 | 18–24 | 650.7 | 1129.5 | retire |
| T-N-Ne-merge-c | 6 of 10 | cycle 28 | 19–24 | 747.2 | 1397.5 | retire |
| F-L-d2mix-c | 5 of 10 | cycle 29 | 20–24 | 834.9 | 1725.0 | retire |
| T-L-LLs-merge-c | 5 of 10 | cycle 29 | 20–24 | 926.3 | 1540.3 | retire |
| T-L-Le-c | 5 of 10 | cycle 29 | 20–24 | 934.2 | 1559.1 | retire |
| T-L-Nc-merge-c | 5 of 10 | cycle 29 | 20–24 | 972.2 | 1650.0 | retire |
| F-L-d2play-c | 3 of 10 | cycle 31 | 22–24 | 816.3 | 1693.1 | retire |
| F-L-d0soft-c | 2 of 10 | cycle 32 | 23–24 | 982.0 | 1625.6 | retire |
| T-N-Le-merge-c | 1 of 10 | cycle 33 | 24–24 | 732.4 | 1347.6 | retire |
| FM-c | 0 of 10 | cycle 34 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | qualified | 2988 | 127 x 1000 | 241 | 1629 | 40 | 1720 | 1720 | v2860 | 1487 | strike 1 (-91 ± 37 vs peak v2860; predecessor -91) | 1550 (123733) | +466 -358 =176, 55.4%, +38 ± 20: kept | 2989 |
| F-L-d0soft-c | dormant | 2363 |  |  | 1653 | 41 | 1600 | 1705 (1705) | v2014 (v2014) | 1511 | dormant (-52 ± 30 vs peak v2014 before dormancy at v2361, at 1563 vs refs; needs > +30) | 1550 (126411) | +406 -439 =155, 48.4%, -12 ± 20: reverted | 2363 |
| F-L-d1jit-c | retired | 1770 |  |  | 1674 | 30 |  | 1699 (1699) | v1451 (v1451) | 1531 | retired (avg ply 1 1014.9, ply 2 1646.3; no untaken milestone within 10); dormant (-25 ± 27 vs peak v1451 before dormancy at v1763, at 1556 vs refs; needs > +27) |  |  | 1770 |
| F-L-d1play-c | qualified | 3006 |  |  | 1603 | 40 | 1562 | 1572 (1561) | v3004 (v1269) | 1460 | re-qualified (+41 ± 32 vs peak v1269 before dormancy at v2999, at 1419 vs refs) | 1550 (128228) | +427 -434 =139, 49.6%, -2 ± 20: reverted | 3006 |
| F-L-d2play-c | dormant | 1621 |  |  | 1689 | 26 |  | 1755 (1755) | v1011 (v1011) | 1547 | dormant (-66 ± 23 vs peak v1011 before dormancy at v1620, at 1613 vs refs; needs > +23) | 1550 (132976) | +417 -387 =196, 51.5%, +10 ± 19: kept | 1622 |
| F-L-d2mix-c | dormant | 1073 |  |  | 1748 | 45 | 1701 | 1746 (1746) | v910 (v910) | 1606 | dormant (+2 ± 33 vs peak v910 before dormancy at v1068, at 1603 vs refs; needs > +33) | 1550 (128046) | +432 -416 =152, 50.8%, +6 ± 20: kept | 1074 |
| F-L-d2jit-c | qualified | 3116 | 115 x 40 | 240 | 1728 | 44 | 1765 | 1801 | v1651 | 1586 | strike 1 (-73 ± 51 vs peak v1651; predecessor -37) | 1550 (138321) | +401 -391 =208, 50.5%, +4 ± 19: kept | 3117 |
| F-N-c | dormant | 1690 |  |  | 1405 | 27 |  | 1478 (1478) | v1061 (v1061) | 1263 | dormant (-72 ± 25 vs peak v1061 before dormancy at v1683, at 1335 vs refs; needs > +25) | 1550 (125839) | +414 -419 =167, 49.8%, -2 ± 20: reverted | 1690 |
| MoE-c | frozen | 373 |  |  | 1714 | 26 |  | 1722 (1722) | v300 (v300) | 1572 | frozen at ply 2 (avg 1695.7 over cycles 15–24; milestone 1700); dormant (-8 ± 27 vs peak v300 before dormancy at v367, at 1580 vs refs; needs > +27) |  |  | 373 |
| FM-c | dormant | 576 | 20 x 1000 | 237 | 1318 | 37 | 1401 | 1570 | v535 | 1176 | dormant (strike 2; -252 ± 40 vs peak v535; predecessor -83) | 1550 (134438) | +709 -177 =114, 76.6%, +206 ± 23: kept | 577 |
| TAN-c | retired | 929 |  |  | 1211 | 14 |  | 1241 (1241) | v507 (v507) | 1068 | retired (avg ply 1 725.2, ply 2 1216.7; no untaken milestone within 10); dormant (-30 ± 21 vs peak v507 before dormancy at v928, at 1098 vs refs; needs > +21) |  |  | 929 |
| T-L-Ne-c | qualified | 874 | 30 x 1000 | 236 | 1609 | 40 | 1549 | 1610 | v556 | 1466 | ok (-2 ± 35 vs peak v556; predecessor +60) | 1550 (131333) | +452 -393 =155, 52.9%, +20 ± 20: kept | 875 |
| T-L-Ne-merge-c | dormant | 1013 |  |  | 1676 | 30 |  | 1707 (1707) | v794 (v794) | 1534 | dormant (-30 ± 20 vs peak v794 before dormancy at v1008, at 1564 vs refs; needs > +20) | 1550 (127777) | +419 -435 =146, 49.2%, -6 ± 20: reverted | 1013 |
| T-L-Ne-fresh-c | qualified | 164 | 56 x 1000 | 239 | 1468 | 38 | 1362 | 1362 | v107 | 1326 | new peak (+106 ± 39 over v107; predecessor +106) | 1550 (134858) | +501 -385 =114, 55.8%, +40 ± 20: kept | 165 |
| T-L-Nc-c | dormant | 717 |  |  | 1596 | 39 | 1631 | 1649 (1649) | v516 (v516) | 1453 | dormant (-54 ± 30 vs peak v516 before dormancy at v709, at 1507 vs refs; needs > +30) | 1550 (134755) | +438 -378 =184, 53.0%, +21 ± 20: kept | 718 |
| T-L-Nc-merge-c | dormant | 1881 |  |  | 1666 | 26 |  | 1730 (1730) | v1631 (v1631) | 1523 | dormant (-65 ± 27 vs peak v1631 before dormancy at v1878, at 1588 vs refs; needs > +27) | 1550 (123246) | +435 -431 =134, 50.2%, +1 ± 20: kept | 1882 |
| T-L-Nc-fresh-c | qualified | 384 | 35 x 1000 | 243 | 1529 | 38 | 1553 | 1553 | v348 | 1387 | ok (-24 ± 40 vs peak v348; predecessor -24) | 1550 (130208) | +454 -390 =156, 53.2%, +22 ± 20: kept | 385 |
| T-L-LLs-c | frozen | 651 |  |  | 1611 | 18 |  | 1634 (1634) | v540 (v540) | 1469 | frozen at ply 2 (avg 1609.3 over cycles 15–24; milestone 1600); dormant (-23 ± 24 vs peak v540 before dormancy at v647, at 1492 vs refs; needs > +24) |  |  | 651 |
| T-L-LLs-merge-c | dormant | 1757 |  |  | 1587 | 39 | 1544 | 1609 (1609) | v1251 (v1251) | 1444 | dormant (-22 ± 29 vs peak v1251 before dormancy at v1754, at 1467 vs refs; needs > +29) | 1550 (130814) | +427 -409 =164, 50.9%, +6 ± 20: kept | 1758 |
| T-L-NLs-c | qualified | 1298 | 38 x 1000 | 239 | 1534 | 38 | 1507 | 1570 | v571 | 1392 | ok (-36 ± 46 vs peak v571; predecessor +26) | 1550 (134308) | +391 -420 =189, 48.5%, -10 ± 19: reverted | 1298 |
| T-L-NLs-merge-c | dormant | 1119 |  |  | 1612 | 29 |  | 1654 (1654) | v641 (v641) | 1470 | dormant (-42 ± 37 vs peak v641 before dormancy at v1115, at 1512 vs refs; needs > +37) | 1550 (126604) | +421 -402 =177, 50.9%, +7 ± 20: kept | 1120 |
| T-L-NNs-c | qualified | 1274 | 38 x 1000 | 239 | 1580 | 39 | 1574 | 1606 | v561 | 1438 | ok (-27 ± 47 vs peak v561; predecessor +6) | 1550 (131899) | +390 -443 =167, 47.3%, -18 ± 20: reverted | 1274 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 976 | 19 |  | 1707 | v1067 | 834 | frozen |  |  | 1070 |
| T-L-Le-c | dormant | 989 |  |  | 1581 | 39 | 1562 | 1590 (1590) | v696 (v696) | 1439 | dormant (-9 ± 29 vs peak v696 before dormancy at v985, at 1448 vs refs; needs > +29) | 1550 (130359) | +443 -383 =174, 53.0%, +21 ± 20: kept | 990 |
| T-L-Le-merge-c | dormant | 2052 |  |  | 1674 | 19 |  | 1706 (1694) | v2051 (v1857) | 1532 | dormant (-20 ± 25 vs peak v1857 before dormancy at v2049, at 1552 vs refs; needs > +25) | 1550 (124739) | +380 -494 =126, 44.3%, -40 ± 20: reverted | 2052 |
| T-L-Le-fresh-c | qualified | 160 | 64 x 1000 | 239 | 1417 | 37 | 1341 | 1341 | v95 | 1275 | new peak (+76 ± 38 over v95; predecessor +76) | 1550 (131458) | +582 -315 =103, 63.3%, +95 ± 21: kept | 161 |
| T-N-Ne-c | frozen | 637 |  |  | 1283 | 16 |  | 1373 | v574 | 1141 | frozen |  |  | 637 |
| T-N-Ne-merge-c | dormant | 906 |  |  | 1413 | 37 | 1399 | 1449 (1449) | v699 (v699) | 1271 | dormant (-36 ± 24 vs peak v699 before dormancy at v904, at 1307 vs refs; needs > +24) | 1550 (123130) | +423 -410 =167, 50.6%, +4 ± 20: kept | 907 |
| T-N-Nc-c | qualified | 808 | 31 x 1000 | 237 | 1371 | 37 | 1371 | 1392 | v472 | 1229 | ok (-21 ± 46 vs peak v472; predecessor -0) | 1550 (124614) | +416 -410 =174, 50.3%, +2 ± 20: kept | 809 |
| T-N-Nc-merge-c | qualified | 1296 | 52 x 1000 | 240 | 1430 | 38 | 1387 | 1451 | v631 | 1287 | ok (-22 ± 36 vs peak v631; predecessor +42) | 1550 (123435) | +397 -406 =197, 49.5%, -3 ± 19: reverted | 1296 |
| T-N-Nc-fresh-c | qualified | 174 | 72 x 1000 | 241 | 1317 | 37 | 1259 | 1259 | v102 | 1175 | new peak (+59 ± 38 over v102; predecessor +59) | 1550 (129424) | +418 -401 =181, 50.8%, +6 ± 20: kept | 175 |
| T-N-NNs-c | dormant | 631 |  |  | 1124 | 36 | 1152 | 1239 (1239) | v341 (v341) | 982 | dormant (-115 ± 39 vs peak v341 before dormancy at v625, at 1097 vs refs; needs > +39) | 1550 (135021) | +404 -416 =180, 49.4%, -4 ± 20: reverted | 631 |
| T-N-Le-merge-c | dormant | 754 |  |  | 1348 | 27 |  | 1434 (1434) | v700 (v700) | 1205 | dormant (-86 ± 22 vs peak v700 before dormancy at v754, at 1292 vs refs; needs > +22) | 1550 (127753) | +377 -493 =130, 44.2%, -40 ± 20: reverted | 754 |
| T-N-Lc-merge-c | retired | 368 |  |  | 1410 | 27 |  | 1440 (1440) | v287 (v287) | 1268 | retired (avg ply 1 752.5, ply 2 1412.2; no untaken milestone within 10); dormant (-30 ± 24 vs peak v287 before dormancy at v364, at 1298 vs refs; needs > +24) |  |  | 368 |
| T-N-LLs-merge-c | dormant | 725 |  |  | 1127 | 17 |  | 1372 (1372) | v585 (v585) | 984 | dormant (-246 ± 21 vs peak v585 before dormancy at v723, at 1230 vs refs; needs > +21) | 1550 (134786) | +381 -438 =181, 47.1%, -20 ± 20: reverted | 725 |
| T-N-NLs-merge-c | qualified | 963 | 65 x 1000 | 238 | 1305 | 37 | 1415 | 1423 | v787 | 1163 | strike 1 (-118 ± 33 vs peak v787; predecessor -110) | 1550 (129762) | +450 -398 =152, 52.6%, +18 ± 20: kept | 964 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 1252 | 36 | 1258 | 1350 (1350) | v270 (v270) | 1110 | frozen at ply 1 (avg 696.6 over cycles 15–24; milestone 700); dormant (-98 ± 27 vs peak v270 before dormancy at v410, at 1208 vs refs; needs > +27) |  |  | 417 |
| F-N-d2jit-c | qualified | 1400 |  |  | 1208 | 36 | 1203 | 1203 (1177) | v1399 (v389) | 1066 | re-qualified (+31 ± 26 vs peak v389 before dormancy at v1397, at 1034 vs refs) | 1550 (137427) | +410 -385 =205, 51.2%, +9 ± 19: kept | 1401 |
| T-N-NNs-merge-d2jit-c | qualified | 1277 | 76 x 40 | 240 | 1149 | 36 | 1170 | 1175 | v1107 | 1006 | ok (-26 ± 37 vs peak v1107; predecessor -21) | 1550 (133003) | +426 -372 =202, 52.7%, +19 ± 19: kept | 1278 |
| F-L-d4mix-c | qualified | 106 | 8 x 10 | 235 | 1612 | 40 | 1518 | 1518 | v97 | 1470 | new peak (+93 ± 40 over v97; predecessor +93) | 1550 (133396) | +469 -384 =147, 54.2%, +30 ± 20: kept | 107 |
| MoE-d2jit-c | qualified | 445 | 44 x 40 | 241 | 1560 | 39 | 1599 | 1599 | v400 | 1418 | ok (-38 ± 40 vs peak v400; predecessor -38) | 1550 (133751) | +525 -311 =164, 60.7%, +76 ± 20: kept | 446 |
| T-L-H-c | dormant | 1525 |  |  | 1645 | 40 | 1602 | 1660 (1660) | v1245 (v1245) | 1503 | dormant (-16 ± 30 vs peak v1245 before dormancy at v1521, at 1518 vs refs; needs > +30) | 1550 (128531) | +417 -419 =164, 49.9%, -1 ± 20: reverted | 1525 |
