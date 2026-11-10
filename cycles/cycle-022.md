# Cycle 22

**Own-games step, pooled over the lineages: 37 gates, +15797 -15146 =6057, score 50.9%, Elo +6.1 ± 3.2; kept 22 of 37.**

All cycles so far: 776 gates, +349891 -301183 =124926, score 53.1%, Elo +21.8 ± 0.7; kept 527 of 776.

2026-11-07 04:08: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 56 players to retired.txt (their games stay in the fit), added 56.
Tournament: 5 double round-robins of the 85 active players (35700 games) in 5.1 min, 35700 games saved.
Freeze/retire rule, after 10 cycles dormant: T-N-H-c retired (avg ply 1 745.3, ply 2 1291.8; no untaken milestone within 10); T-N-NNs-merge-c retired (avg ply 1 741.1, ply 2 1374.2; no untaken milestone within 10). 4 entries leave the field for retired.txt (their games stay in the fit): 85 players become 81, 35,700 games a cycle become 32,400.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| F-L-d1jit-c | 8 of 10 | cycle 24 | 15–22 | 1024.8 | 1641.5 | retire |
| MoE-c | 8 of 10 | cycle 24 | 15–22 | 959.1 | 1689.5 | retire |
| T-L-LLs-c | 8 of 10 | cycle 24 | 15–22 | 953.1 | 1608.6 | freeze 2 at 1600 |
| T-N-LNs-merge-c | 8 of 10 | cycle 24 | 15–22 | 693.9 | 1212.3 | freeze 1 at 700 |
| T-N-Lc-merge-c | 8 of 10 | cycle 24 | 15–22 | 754.3 | 1413.5 | retire |
| TAN-c | 8 of 10 | cycle 24 | 15–22 | 727.2 | 1217.9 | retire |
| F-L-c | 6 of 10 | cycle 26 | 17–22 | 993.2 | 1676.9 | retire |
| F-N-c | 6 of 10 | cycle 26 | 17–22 | 735.6 | 1400.8 | freeze 2 at 1400 |
| T-L-Le-merge-c | 6 of 10 | cycle 26 | 17–22 | 977.4 | 1681.7 | retire |
| T-L-Nc-c | 6 of 10 | cycle 26 | 17–22 | 985.6 | 1602.8 | retire |
| F-L-d1play-c | 5 of 10 | cycle 27 | 18–22 | 1001.4 | 1534.9 | retire |
| T-L-H-c | 5 of 10 | cycle 27 | 18–22 | 997.1 | 1600.3 | retire |
| T-L-NLs-merge-c | 5 of 10 | cycle 27 | 18–22 | 935.4 | 1581.7 | retire |
| T-L-Ne-merge-c | 5 of 10 | cycle 27 | 18–22 | 972.1 | 1675.1 | retire |
| T-N-LLs-merge-c | 5 of 10 | cycle 27 | 18–22 | 629.5 | 1122.9 | retire |
| T-N-NNs-c | 5 of 10 | cycle 27 | 18–22 | 657.2 | 1126.5 | retire |
| T-N-Ne-merge-c | 4 of 10 | cycle 28 | 19–22 | 747.2 | 1393.0 | retire |
| F-L-d2mix-c | 3 of 10 | cycle 29 | 20–22 | 824.7 | 1725.9 | retire |
| T-L-LLs-merge-c | 3 of 10 | cycle 29 | 20–22 | 934.0 | 1524.3 | retire |
| T-L-Le-c | 3 of 10 | cycle 29 | 20–22 | 938.0 | 1550.9 | retire |
| T-L-Nc-merge-c | 3 of 10 | cycle 29 | 20–22 | 974.0 | 1638.5 | retire |
| F-L-d2play-c | 1 of 10 | cycle 31 | 22–22 | 814.7 | 1697.8 | freeze 2 at 1700 |
| F-N-d2jit-c | 1 of 10 | cycle 31 | 22–22 | 603.7 | 1181.0 | freeze 1 at 600 |
| F-L-d0soft-c | 0 of 10 | cycle 32 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | dormant | 2860 |  |  | 1715 | 44 | 1650 | 1699 (1693) | v2855 (v2247) | 1570 | dormant (+22 ± 32 vs peak v2247 before dormancy at v2854, at 1548 vs refs; needs > +32) | 1570 (124856) | +422 -430 =148, 49.6%, -3 ± 20: reverted | 2860 |
| F-L-d0soft-c | dormant | 2361 | 170 x 1000 | 240 | 1554 | 39 | 1595 | 1707 | v2014 | 1409 | dormant (strike 2; -153 ± 43 vs peak v2014; predecessor -40) | 1570 (131119) | +533 -332 =135, 60.1%, +71 ± 20: kept | 2362 |
| F-L-d1jit-c | dormant | 1769 |  |  | 1658 | 42 | 1653 | 1702 (1702) | v1451 (v1451) | 1512 | dormant (-44 ± 31 vs peak v1451 before dormancy at v1763, at 1557 vs refs; needs > +31) | 1570 (129087) | +457 -402 =141, 52.8%, +19 ± 20: kept | 1770 |
| F-L-d1play-c | dormant | 3004 |  |  | 1575 | 40 | 1530 | 1564 (1564) | v1269 (v1269) | 1430 | dormant (+11 ± 32 vs peak v1269 before dormancy at v2999, at 1419 vs refs; needs > +32) | 1570 (133592) | +478 -383 =139, 54.8%, +33 ± 20: kept | 3005 |
| F-L-d2play-c | dormant | 1621 |  |  | 1698 | 43 | 1669 | 1758 (1758) | v1011 (v1011) | 1553 | dormant (-60 ± 29 vs peak v1011 before dormancy at v1620, at 1613 vs refs; needs > +29) | 1570 (133876) | +349 -435 =216, 45.7%, -30 ± 19: reverted | 1621 |
| F-L-d2mix-c | dormant | 1071 |  |  | 1716 | 44 | 1726 | 1750 (1750) | v910 (v910) | 1571 | dormant (-34 ± 33 vs peak v910 before dormancy at v1068, at 1605 vs refs; needs > +33) | 1570 (132667) | +406 -381 =213, 51.2%, +9 ± 19: kept | 1072 |
| F-L-d2jit-c | qualified | 2900 | 106 x 40 | 239 | 1768 | 47 | 1768 | 1804 | v1651 | 1622 | ok (-36 ± 53 vs peak v1651; predecessor -0) | 1570 (137574) | +342 -475 =183, 43.4%, -46 ± 20: reverted | 2900 |
| F-N-c | dormant | 1689 |  |  | 1424 | 38 | 1397 | 1480 (1480) | v1061 (v1061) | 1279 | dormant (-56 ± 28 vs peak v1061 before dormancy at v1683, at 1335 vs refs; needs > +28) | 1570 (128810) | +447 -393 =160, 52.7%, +19 ± 20: kept | 1690 |
| MoE-c | dormant | 373 |  |  | 1739 | 45 | 1647 | 1725 (1725) | v300 (v300) | 1594 | dormant (+14 ± 33 vs peak v300 before dormancy at v367, at 1580 vs refs; needs > +33) | 1570 (127721) | +383 -468 =149, 45.8%, -30 ± 20: reverted | 373 |
| FM-c | qualified | 535 |  |  | 1573 | 40 | 1405 | 1446 (1446) | v419 (v419) | 1428 | re-qualified (+127 ± 28 vs peak v419 before dormancy at v534, at 1301 vs refs) | 1570 (137672) | +468 -387 =145, 54.0%, +28 ± 20: kept | 536 |
| TAN-c | dormant | 929 |  |  | 1216 | 15 |  | 1244 (1244) | v507 (v507) | 1071 | dormant (-28 ± 21 vs peak v507 before dormancy at v928, at 1098 vs refs; needs > +21) | 1570 (133365) | +396 -465 =139, 46.6%, -24 ± 20: reverted | 929 |
| T-L-Ne-c | qualified | 812 | 37 x 1000 | 238 | 1593 | 40 | 1552 | 1613 | v556 | 1448 | ok (-20 ± 35 vs peak v556; predecessor +41) | 1570 (133995) | +409 -430 =161, 48.9%, -7 ± 20: reverted | 812 |
| T-L-Ne-merge-c | dormant | 1012 |  |  | 1682 | 31 |  | 1710 (1710) | v794 (v794) | 1537 | dormant (-28 ± 20 vs peak v794 before dormancy at v1008, at 1565 vs refs; needs > +20) | 1570 (123550) | +450 -412 =138, 51.9%, +13 ± 20: kept | 1013 |
| T-L-Nc-c | dormant | 715 |  |  | 1632 | 41 | 1627 | 1652 (1652) | v516 (v516) | 1486 | dormant (-21 ± 30 vs peak v516 before dormancy at v709, at 1507 vs refs; needs > +30) | 1570 (138827) | +425 -382 =193, 52.1%, +15 ± 19: kept | 716 |
| T-L-Nc-merge-c | dormant | 1881 |  |  | 1674 | 42 | 1624 | 1734 (1734) | v1631 (v1631) | 1529 | dormant (-60 ± 32 vs peak v1631 before dormancy at v1878, at 1589 vs refs; needs > +32) | 1570 (124573) | +409 -442 =149, 48.4%, -12 ± 20: reverted | 1881 |
| T-L-Nc-fresh-c | qualified | 309 | 52 x 1000 | 239 | 1494 | 39 | 1516 | 1516 | v256 | 1349 | ok (-22 ± 40 vs peak v256; predecessor -22) | 1570 (137931) | +516 -340 =144, 58.8%, +62 ± 20: kept | 310 |
| T-L-LLs-c | dormant | 651 |  |  | 1613 | 21 |  | 1637 (1637) | v540 (v540) | 1468 | dormant (-24 ± 24 vs peak v540 before dormancy at v647, at 1492 vs refs; needs > +24) | 1570 (131653) | +373 -409 =218, 48.2%, -12 ± 19: reverted | 651 |
| T-L-LLs-merge-c | dormant | 1755 |  |  | 1501 | 39 | 1532 | 1612 (1612) | v1251 (v1251) | 1356 | dormant (-111 ± 29 vs peak v1251 before dormancy at v1754, at 1467 vs refs; needs > +29) | 1570 (137051) | +439 -411 =150, 51.4%, +10 ± 20: kept | 1756 |
| T-L-NLs-c | qualified | 1226 | 42 x 1000 | 240 | 1559 | 40 | 1561 | 1572 | v571 | 1414 | ok (-13 ± 46 vs peak v571; predecessor -2) | 1570 (135972) | +358 -480 =162, 43.9%, -43 ± 20: reverted | 1226 |
| T-L-NLs-merge-c | dormant | 1118 |  |  | 1601 | 40 | 1583 | 1658 (1658) | v641 (v641) | 1456 | dormant (-56 ± 40 vs peak v641 before dormancy at v1115, at 1513 vs refs; needs > +40) | 1570 (127842) | +435 -407 =158, 51.4%, +10 ± 20: kept | 1119 |
| T-L-NNs-c | qualified | 1204 | 40 x 1000 | 238 | 1538 | 39 | 1600 | 1609 | v561 | 1393 | strike 1 (-71 ± 47 vs peak v561; predecessor -62) | 1570 (137653) | +437 -400 =163, 51.8%, +13 ± 20: kept | 1205 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 982 | 26 |  | 1711 | v1067 | 837 | frozen |  |  | 1070 |
| T-L-Le-c | dormant | 988 |  |  | 1547 | 39 | 1564 | 1593 (1593) | v696 (v696) | 1402 | dormant (-47 ± 29 vs peak v696 before dormancy at v985, at 1448 vs refs; needs > +29) | 1570 (133878) | +378 -484 =138, 44.7%, -37 ± 20: reverted | 988 |
| T-L-Le-merge-c | dormant | 2052 |  |  | 1675 | 23 |  | 1709 (1697) | v2051 (v1857) | 1530 | dormant (-22 ± 26 vs peak v1857 before dormancy at v2049, at 1552 vs refs; needs > +26) | 1570 (127409) | +426 -436 =138, 49.5%, -4 ± 20: reverted | 2052 |
| T-N-Ne-c | frozen | 637 |  |  | 1286 | 19 |  | 1376 | v574 | 1141 | frozen |  |  | 637 |
| T-N-Ne-merge-c | dormant | 905 |  |  | 1399 | 24 |  | 1452 (1452) | v699 (v699) | 1254 | dormant (-53 ± 19 vs peak v699 before dormancy at v904, at 1307 vs refs; needs > +19) | 1570 (124111) | +405 -416 =179, 49.5%, -4 ± 20: reverted | 905 |
| T-N-Nc-c | qualified | 746 | 33 x 1000 | 240 | 1383 | 38 | 1368 | 1395 | v472 | 1238 | ok (-12 ± 46 vs peak v472; predecessor +15) | 1570 (130658) | +431 -409 =160, 51.1%, +8 ± 20: kept | 747 |
| T-N-Nc-merge-c | qualified | 1199 | 52 x 1000 | 240 | 1421 | 38 | 1404 | 1454 | v631 | 1276 | ok (-33 ± 36 vs peak v631; predecessor +18) | 1570 (127214) | +422 -400 =178, 51.1%, +8 ± 20: kept | 1200 |
| T-N-NNs-c | dormant | 629 |  |  | 1119 | 27 |  | 1243 (1243) | v341 (v341) | 974 | dormant (-124 ± 37 vs peak v341 before dormancy at v625, at 1098 vs refs; needs > +37) | 1570 (138653) | +415 -403 =182, 50.6%, +4 ± 20: kept | 630 |
| T-N-NNs-merge-c | retired | 1000 |  |  | 1370 | 38 | 1365 | 1472 (1472) | v855 (v855) | 1225 | retired (avg ply 1 741.1, ply 2 1374.2; no untaken milestone within 10); dormant (-102 ± 29 vs peak v855 before dormancy at v997, at 1327 vs refs; needs > +29) |  |  | 1000 |
| T-N-Le-merge-c | qualified | 729 | 28 x 1000 | 238 | 1357 | 38 | 1436 | 1436 | v700 | 1212 | strike 1 (-80 ± 39 vs peak v700; predecessor -80) | 1570 (127500) | +415 -451 =134, 48.2%, -12 ± 20: reverted | 729 |
| T-N-Lc-merge-c | dormant | 367 |  |  | 1388 | 38 | 1415 | 1443 (1443) | v287 (v287) | 1243 | dormant (-55 ± 28 vs peak v287 before dormancy at v364, at 1298 vs refs; needs > +28) | 1570 (126351) | +414 -402 =184, 50.6%, +4 ± 20: kept | 368 |
| T-N-LLs-merge-c | dormant | 725 |  |  | 1127 | 20 |  | 1376 (1376) | v585 (v585) | 982 | dormant (-249 ± 22 vs peak v585 before dormancy at v723, at 1231 vs refs; needs > +22) | 1570 (138096) | +375 -468 =157, 45.4%, -32 ± 20: reverted | 725 |
| T-N-NLs-merge-c | qualified | 787 |  |  | 1425 | 28 |  | 1398 (1398) | v672 (v672) | 1280 | re-qualified (+28 ± 23 vs peak v672 before dormancy at v785, at 1253 vs refs) | 1570 (125232) | +454 -376 =170, 53.9%, +27 ± 20: kept | 788 |
| T-N-LNs-merge-c | dormant | 416 |  |  | 1258 | 23 |  | 1353 (1353) | v270 (v270) | 1113 | dormant (-95 ± 23 vs peak v270 before dormancy at v410, at 1208 vs refs; needs > +23) | 1570 (133885) | +404 -412 =184, 49.6%, -3 ± 20: reverted | 416 |
| F-N-d2jit-c | dormant | 1398 |  |  | 1181 | 37 | 1123 | 1180 (1180) | v389 (v389) | 1036 | dormant (+1 ± 26 vs peak v389 before dormancy at v1397, at 1035 vs refs; needs > +26) | 1570 (135115) | +429 -367 =204, 53.1%, +22 ± 19: kept | 1399 |
| T-N-NNs-merge-d2jit-c | qualified | 1107 | 97 x 40 | 248 | 1178 | 37 | 1138 | 1175 | v642 | 1032 | new peak (+3 ± 39 over v642; predecessor +39) | 1570 (137249) | +418 -394 =188, 51.2%, +8 ± 19: kept | 1108 |
| F-L-d4mix-c | qualified | 88 | 9 x 10 | 254 | 1440 | 38 | 1377 | 1377 | v78 | 1295 | new peak (+64 ± 39 over v78; predecessor +64) | 1570 (139153) | +571 -286 =143, 64.2%, +102 ± 21: kept | 89 |
| MoE-d2jit-c | qualified | 352 | 46 x 40 | 241 | 1578 | 40 | 1506 | 1506 | v305 | 1433 | new peak (+72 ± 40 over v305; predecessor +72) | 1570 (136936) | +461 -374 =165, 54.4%, +30 ± 20: kept | 353 |
| T-L-H-c | dormant | 1523 |  |  | 1597 | 22 |  | 1664 (1664) | v1245 (v1245) | 1452 | dormant (-67 ± 25 vs peak v1245 before dormancy at v1521, at 1519 vs refs; needs > +25) | 1570 (133242) | +447 -404 =149, 52.1%, +15 ± 20: kept | 1524 |
| T-N-H-c | retired | 1734 |  |  | 1302 | 27 |  | 1366 (1366) | v1104 (v1104) | 1157 | retired (avg ply 1 745.3, ply 2 1291.8; no untaken milestone within 10); dormant (-63 ± 27 vs peak v1104 before dormancy at v1728, at 1220 vs refs; needs > +27) |  |  | 1734 |
