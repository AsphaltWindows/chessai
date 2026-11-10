# Cycle 21

**Own-games step, pooled over the lineages: 39 gates, +16782 -15737 =6481, score 51.3%, Elo +9.3 ± 3.1; kept 25 of 39.**

All cycles so far: 739 gates, +334094 -286037 =118869, score 53.3%, Elo +22.6 ± 0.7; kept 505 of 739.

2026-11-07 03:44: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 54 players to retired.txt (their games stay in the fit), added 54.
Tournament: 5 double round-robins of the 87 active players (37410 games) in 4.2 min, 37410 games saved.
Freeze/retire rule, after 10 cycles dormant: T-L-NNs-merge-c frozen at ply 1 (avg 992.4 over cycles 12–21; milestone 1000); T-N-Ne-c frozen at ply 2 (avg 1297.3 over cycles 12–21; milestone 1300). 2 entries leave the field for retired.txt (their games stay in the fit): 87 players become 85, 37,410 games a cycle become 35,700.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| T-N-H-c | 9 of 10 | cycle 22 | 13–21 | 747.4 | 1290.7 | retire |
| T-N-NNs-merge-c | 9 of 10 | cycle 22 | 13–21 | 744.7 | 1374.6 | retire |
| F-L-d1jit-c | 7 of 10 | cycle 24 | 15–21 | 1026.1 | 1639.2 | retire |
| MoE-c | 7 of 10 | cycle 24 | 15–21 | 961.1 | 1682.5 | retire |
| T-L-LLs-c | 7 of 10 | cycle 24 | 15–21 | 953.7 | 1608.0 | freeze 2 at 1600 |
| T-N-LNs-merge-c | 7 of 10 | cycle 24 | 15–21 | 692.3 | 1205.8 | freeze 2 at 1200 |
| T-N-Lc-merge-c | 7 of 10 | cycle 24 | 15–21 | 754.4 | 1417.1 | retire |
| TAN-c | 7 of 10 | cycle 24 | 15–21 | 727.6 | 1218.2 | retire |
| F-L-c | 5 of 10 | cycle 26 | 17–21 | 995.1 | 1669.3 | retire |
| F-N-c | 5 of 10 | cycle 26 | 17–21 | 736.1 | 1396.1 | freeze 2 at 1400 |
| T-L-Le-merge-c | 5 of 10 | cycle 26 | 17–21 | 977.5 | 1683.0 | retire |
| T-L-Nc-c | 5 of 10 | cycle 26 | 17–21 | 988.6 | 1597.0 | retire |
| F-L-d1play-c | 4 of 10 | cycle 27 | 18–21 | 998.3 | 1524.9 | retire |
| T-L-H-c | 4 of 10 | cycle 27 | 18–21 | 996.9 | 1601.1 | retire |
| T-L-NLs-merge-c | 4 of 10 | cycle 27 | 18–21 | 933.0 | 1576.8 | retire |
| T-L-Ne-merge-c | 4 of 10 | cycle 27 | 18–21 | 973.4 | 1673.2 | retire |
| T-N-LLs-merge-c | 4 of 10 | cycle 27 | 18–21 | 631.7 | 1121.9 | retire |
| T-N-NNs-c | 4 of 10 | cycle 27 | 18–21 | 659.8 | 1128.4 | retire |
| T-N-NLs-merge-c | 3 of 10 | cycle 28 | 19–21 | 762.1 | 1399.3 | retire |
| T-N-Ne-merge-c | 3 of 10 | cycle 28 | 19–21 | 748.0 | 1391.1 | retire |
| F-L-d2mix-c | 2 of 10 | cycle 29 | 20–21 | 810.2 | 1730.8 | retire |
| T-L-LLs-merge-c | 2 of 10 | cycle 29 | 20–21 | 936.9 | 1536.0 | retire |
| T-L-Le-c | 2 of 10 | cycle 29 | 20–21 | 936.3 | 1553.0 | retire |
| T-L-Nc-merge-c | 2 of 10 | cycle 29 | 20–21 | 973.1 | 1620.6 | retire |
| F-L-d2play-c | 0 of 10 | cycle 31 | - | - | - | retire |
| F-N-d2jit-c | 0 of 10 | cycle 31 | - | - | - | retire |
| FM-c | 0 of 10 | cycle 31 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | dormant | 2859 |  |  | 1651 | 41 | 1666 | 1701 (1696) | v2855 (v2247) | 1503 | dormant (-45 ± 31 vs peak v2247 before dormancy at v2854, at 1548 vs refs; needs > +31) | 1570 (123989) | +431 -428 =141, 50.1%, +1 ± 20: kept | 2860 |
| F-L-d0soft-c | qualified | 2190 | 175 x 1000 | 241 | 1595 | 40 | 1709 | 1709 | v2014 | 1447 | strike 1 (-114 ± 43 vs peak v2014; predecessor -114) | 1570 (126702) | +493 -364 =143, 56.5%, +45 ± 20: kept | 2191 |
| F-L-d1jit-c | dormant | 1768 |  |  | 1654 | 31 |  | 1704 (1704) | v1451 (v1451) | 1506 | dormant (-51 ± 27 vs peak v1451 before dormancy at v1763, at 1556 vs refs; needs > +27) | 1570 (130548) | +436 -422 =142, 50.7%, +5 ± 20: kept | 1769 |
| F-L-d1play-c | dormant | 3003 |  |  | 1530 | 39 | 1561 | 1567 (1567) | v1269 (v1269) | 1382 | dormant (-37 ± 31 vs peak v1269 before dormancy at v2999, at 1419 vs refs; needs > +31) | 1570 (137984) | +446 -425 =129, 51.0%, +7 ± 20: kept | 3004 |
| F-L-d2play-c | dormant | 1620 | 105 x 40 | 240 | 1669 | 42 | 1700 | 1761 | v1011 | 1521 | dormant (strike 2; -92 ± 42 vs peak v1011; predecessor -31) | 1570 (128667) | +458 -353 =189, 55.2%, +37 ± 20: kept | 1621 |
| F-L-d2mix-c | dormant | 1070 |  |  | 1726 | 44 | 1736 | 1751 (1751) | v910 (v910) | 1578 | dormant (-25 ± 33 vs peak v910 before dormancy at v1068, at 1603 vs refs; needs > +33) | 1570 (129544) | +407 -363 =230, 52.2%, +15 ± 19: kept | 1071 |
| F-L-d2jit-c | qualified | 2794 | 117 x 40 | 241 | 1769 | 47 | 1732 | 1806 | v1651 | 1620 | ok (-38 ± 53 vs peak v1651; predecessor +37) | 1570 (134429) | +379 -410 =211, 48.4%, -11 ± 19: reverted | 2794 |
| F-N-c | dormant | 1688 |  |  | 1397 | 38 | 1386 | 1483 (1483) | v1061 (v1061) | 1249 | dormant (-86 ± 28 vs peak v1061 before dormancy at v1683, at 1335 vs refs; needs > +28) | 1570 (128481) | +438 -408 =154, 51.5%, +10 ± 20: kept | 1689 |
| MoE-c | dormant | 372 |  |  | 1648 | 31 |  | 1728 (1728) | v300 (v300) | 1500 | dormant (-80 ± 28 vs peak v300 before dormancy at v367, at 1580 vs refs; needs > +28) | 1570 (127933) | +424 -424 =152, 50.0%, -0 ± 20: kept | 373 |
| FM-c | dormant | 534 | 23 x 1000 | 238 | 1405 | 38 | 1386 | 1448 | v419 | 1257 | dormant (strike 2; -42 ± 40 vs peak v419; predecessor +19) | 1570 (133161) | +597 -274 =129, 66.1%, +116 ± 21: kept | 535 |
| TAN-c | dormant | 929 |  |  | 1216 | 16 |  | 1246 (1246) | v507 (v507) | 1067 | dormant (-31 ± 21 vs peak v507 before dormancy at v928, at 1098 vs refs; needs > +21) | 1570 (134416) | +413 -460 =127, 47.6%, -16 ± 20: reverted | 929 |
| T-L-Ne-c | qualified | 774 | 38 x 1000 | 238 | 1552 | 39 | 1589 | 1616 | v556 | 1404 | strike 1 (-64 ± 35 vs peak v556; predecessor -37) | 1570 (136811) | +470 -384 =146, 54.3%, +30 ± 20: kept | 775 |
| T-L-Ne-merge-c | dormant | 1012 |  |  | 1686 | 43 | 1665 | 1712 (1712) | v794 (v794) | 1538 | dormant (-26 ± 25 vs peak v794 before dormancy at v1008, at 1564 vs refs; needs > +25) | 1570 (123591) | +414 -444 =142, 48.5%, -10 ± 20: reverted | 1012 |
| T-L-Nc-c | dormant | 714 |  |  | 1627 | 41 | 1606 | 1655 (1655) | v516 (v516) | 1479 | dormant (-28 ± 30 vs peak v516 before dormancy at v709, at 1507 vs refs; needs > +30) | 1570 (139942) | +413 -399 =188, 50.7%, +5 ± 19: kept | 715 |
| T-L-Nc-merge-c | dormant | 1880 |  |  | 1624 | 40 | 1617 | 1736 (1736) | v1631 (v1631) | 1476 | dormant (-111 ± 31 vs peak v1631 before dormancy at v1878, at 1587 vs refs; needs > +31) | 1570 (125440) | +467 -383 =150, 54.2%, +29 ± 20: kept | 1881 |
| T-L-Nc-fresh-c | qualified | 256 | 58 x 1000 | 239 | 1516 | 38 | 1459 | 1459 | v197 | 1368 | new peak (+58 ± 39 over v197; predecessor +58) | 1570 (134505) | +510 -349 =141, 58.1%, +56 ± 20: kept | 257 |
| T-L-LLs-c | dormant | 651 |  |  | 1622 | 23 |  | 1640 (1640) | v540 (v540) | 1473 | dormant (-18 ± 25 vs peak v540 before dormancy at v647, at 1492 vs refs; needs > +25) | 1570 (132599) | +369 -434 =197, 46.8%, -23 ± 19: reverted | 651 |
| T-L-LLs-merge-c | dormant | 1754 |  |  | 1533 | 25 |  | 1615 (1615) | v1251 (v1251) | 1385 | dormant (-82 ± 25 vs peak v1251 before dormancy at v1754, at 1467 vs refs; needs > +25) | 1570 (129158) | +418 -415 =167, 50.1%, +1 ± 20: kept | 1755 |
| T-L-NLs-c | qualified | 1184 | 43 x 1000 | 239 | 1562 | 39 | 1486 | 1576 | v571 | 1414 | ok (-14 ± 46 vs peak v571; predecessor +75) | 1570 (134314) | +281 -540 =179, 37.0%, -92 ± 20: reverted | 1184 |
| T-L-NLs-merge-c | dormant | 1117 |  |  | 1584 | 25 |  | 1662 (1662) | v641 (v641) | 1436 | dormant (-78 ± 37 vs peak v641 before dormancy at v1115, at 1514 vs refs; needs > +37) | 1570 (132185) | +449 -376 =175, 53.6%, +25 ± 20: kept | 1118 |
| T-L-NNs-c | qualified | 1163 | 43 x 1000 | 242 | 1600 | 40 | 1592 | 1612 | v561 | 1452 | ok (-12 ± 47 vs peak v561; predecessor +8) | 1570 (136501) | +428 -423 =149, 50.2%, +2 ± 20: kept | 1164 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 1663 | 42 | 1680 | 1712 (1712) | v1067 (v818) | 1515 | frozen at ply 1 (avg 992.4 over cycles 12–21; milestone 1000); dormant (-49 ± 33 vs peak v818 before dormancy at v1062, at 1564 vs refs; needs > +33) |  |  | 1070 |
| T-L-Le-c | dormant | 987 |  |  | 1564 | 39 | 1542 | 1596 (1596) | v696 (v696) | 1416 | dormant (-32 ± 29 vs peak v696 before dormancy at v985, at 1448 vs refs; needs > +29) | 1570 (137454) | +453 -401 =146, 52.6%, +18 ± 20: kept | 988 |
| T-L-Le-merge-c | dormant | 2052 |  |  | 1675 | 27 |  | 1711 (1700) | v2051 (v1857) | 1527 | dormant (-25 ± 26 vs peak v1857 before dormancy at v2049, at 1552 vs refs; needs > +26) | 1570 (125928) | +403 -452 =145, 47.5%, -17 ± 20: reverted | 2052 |
| T-N-Ne-c | frozen | 637 |  |  | 1281 | 21 |  | 1379 (1379) | v574 (v574) | 1133 | frozen at ply 2 (avg 1297.3 over cycles 12–21; milestone 1300); dormant (-98 ± 24 vs peak v574 before dormancy at v633, at 1231 vs refs; needs > +24) |  |  | 637 |
| T-N-Ne-merge-c | dormant | 905 |  |  | 1399 | 28 |  | 1455 (1455) | v699 (v699) | 1250 | dormant (-56 ± 21 vs peak v699 before dormancy at v904, at 1307 vs refs; needs > +21) | 1570 (125228) | +378 -446 =176, 46.6%, -24 ± 20: reverted | 905 |
| T-N-Nc-c | qualified | 712 | 38 x 1000 | 240 | 1368 | 37 | 1345 | 1398 | v472 | 1219 | ok (-30 ± 46 vs peak v472; predecessor +22) | 1570 (126835) | +420 -410 =170, 50.5%, +4 ± 20: kept | 713 |
| T-N-Nc-merge-c | qualified | 1147 | 65 x 1000 | 244 | 1403 | 38 | 1435 | 1457 | v631 | 1255 | strike 1 (-54 ± 36 vs peak v631; predecessor -32) | 1570 (125853) | +397 -432 =171, 48.2%, -12 ± 20: reverted | 1147 |
| T-N-NNs-c | dormant | 629 |  |  | 1098 | 36 | 1153 | 1247 (1247) | v341 (v341) | 950 | dormant (-149 ± 39 vs peak v341 before dormancy at v625, at 1099 vs refs; needs > +39) | 1570 (139006) | +393 -417 =190, 48.8%, -8 ± 19: reverted | 629 |
| T-N-NNs-merge-c | dormant | 999 |  |  | 1365 | 24 |  | 1475 (1475) | v855 (v855) | 1217 | dormant (-110 ± 25 vs peak v855 before dormancy at v997, at 1327 vs refs; needs > +25) | 1570 (129879) | +401 -358 =241, 52.1%, +15 ± 19: kept | 1000 |
| T-N-Le-merge-c | qualified | 700 |  |  | 1436 | 38 | 1400 | 1405 (1405) | v580 (v580) | 1288 | re-qualified (+31 ± 27 vs peak v580 before dormancy at v697, at 1257 vs refs) | 1570 (132372) | +446 -391 =163, 52.8%, +19 ± 20: kept | 701 |
| T-N-Lc-merge-c | dormant | 366 |  |  | 1416 | 18 |  | 1446 (1446) | v287 (v287) | 1268 | dormant (-30 ± 22 vs peak v287 before dormancy at v364, at 1298 vs refs; needs > +22) | 1570 (129124) | +411 -396 =193, 50.7%, +5 ± 19: kept | 367 |
| T-N-LLs-merge-c | dormant | 725 |  |  | 1130 | 23 |  | 1378 (1378) | v585 (v585) | 982 | dormant (-248 ± 22 vs peak v585 before dormancy at v723, at 1230 vs refs; needs > +22) | 1570 (141152) | +399 -404 =197, 49.8%, -2 ± 19: reverted | 725 |
| T-N-NLs-merge-c | dormant | 787 |  |  | 1416 | 38 | 1380 | 1399 (1399) | v672 (v672) | 1268 | dormant (+17 ± 27 vs peak v672 before dormancy at v785, at 1251 vs refs; needs > +27) | 1570 (124751) | +418 -437 =145, 49.0%, -7 ± 20: reverted | 787 |
| T-N-LNs-merge-c | dormant | 416 |  |  | 1260 | 27 |  | 1356 (1356) | v270 (v270) | 1112 | dormant (-96 ± 24 vs peak v270 before dormancy at v410, at 1207 vs refs; needs > +24) | 1570 (129753) | +397 -433 =170, 48.2%, -12 ± 20: reverted | 416 |
| F-N-d2jit-c | dormant | 1397 | 127 x 40 | 239 | 1123 | 36 | 1104 | 1183 | v389 | 974 | dormant (strike 2; -60 ± 39 vs peak v389; predecessor +19) | 1570 (136430) | +415 -365 =220, 52.5%, +17 ± 19: kept | 1398 |
| T-N-NNs-merge-d2jit-c | qualified | 1009 | 85 x 40 | 240 | 1138 | 36 | 1173 | 1176 | v642 | 989 | ok (-38 ± 38 vs peak v642; predecessor -35) | 1570 (140517) | +411 -398 =191, 50.6%, +4 ± 19: kept | 1010 |
| F-L-d4mix-c | qualified | 78 | 9 x 10 | 251 | 1376 | 37 | 1235 | 1235 | v68 | 1228 | new peak (+141 ± 38 over v68; predecessor +141) | 1570 (147432) | +561 -316 =123, 62.3%, +87 ± 21: kept | 79 |
| MoE-d2jit-c | qualified | 305 | 55 x 40 | 244 | 1506 | 38 | 1409 | 1409 | v249 | 1358 | new peak (+97 ± 39 over v249; predecessor +97) | 1570 (139887) | +517 -315 =168, 60.1%, +71 ± 20: kept | 306 |
| T-L-H-c | dormant | 1523 |  |  | 1606 | 25 |  | 1666 (1666) | v1245 (v1245) | 1457 | dormant (-61 ± 25 vs peak v1245 before dormancy at v1521, at 1518 vs refs; needs > +25) | 1570 (135555) | +407 -457 =136, 47.5%, -17 ± 20: reverted | 1523 |
| T-N-H-c | dormant | 1734 |  |  | 1298 | 37 | 1295 | 1369 (1369) | v1104 (v1104) | 1150 | dormant (-71 ± 30 vs peak v1104 before dormancy at v1728, at 1220 vs refs; needs > +30) | 1570 (128711) | +414 -431 =155, 49.1%, -6 ± 20: reverted | 1734 |
