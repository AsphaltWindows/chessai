# Cycle 18

**Own-games step, pooled over the lineages: 40 gates, +18128 -15321 =6551, score 53.5%, Elo +24.4 ± 3.1; kept 29 of 40.**

All cycles so far: 618 gates, +281325 -237839 =98836, score 53.5%, Elo +24.5 ± 0.8; kept 430 of 618.

2026-11-07 02:36: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 72 players to retired.txt (their games stay in the fit), added 72.
Tournament: 5 double round-robins of the 85 active players (35700 games) in 3.8 min, 35700 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| F-L-d0soft-c | 7 of 10 | cycle 21 | 12–18 | 1008.9 | 1640.9 | retire |
| T-L-NNs-merge-c | 7 of 10 | cycle 21 | 12–18 | 999.0 | 1689.4 | freeze 1 at 1000 |
| T-N-Ne-c | 7 of 10 | cycle 21 | 12–18 | 738.2 | 1305.2 | freeze 2 at 1300 |
| T-N-H-c | 6 of 10 | cycle 22 | 13–18 | 750.8 | 1286.9 | retire |
| T-N-NNs-merge-c | 6 of 10 | cycle 22 | 13–18 | 756.1 | 1378.1 | retire |
| F-L-d1jit-c | 4 of 10 | cycle 24 | 15–18 | 1039.0 | 1628.0 | retire |
| MoE-c | 4 of 10 | cycle 24 | 15–18 | 969.4 | 1696.0 | freeze 2 at 1700 |
| T-L-LLs-c | 4 of 10 | cycle 24 | 15–18 | 955.4 | 1597.7 | freeze 2 at 1600 |
| T-N-LNs-merge-c | 4 of 10 | cycle 24 | 15–18 | 678.2 | 1168.7 | retire |
| T-N-Lc-merge-c | 4 of 10 | cycle 24 | 15–18 | 757.0 | 1417.6 | retire |
| TAN-c | 4 of 10 | cycle 24 | 15–18 | 728.9 | 1218.3 | retire |
| F-L-c | 2 of 10 | cycle 26 | 17–18 | 996.9 | 1680.6 | retire |
| F-N-c | 2 of 10 | cycle 26 | 17–18 | 730.0 | 1393.2 | freeze 2 at 1400 |
| T-L-Le-merge-c | 2 of 10 | cycle 26 | 17–18 | 981.4 | 1685.6 | retire |
| T-L-Nc-c | 2 of 10 | cycle 26 | 17–18 | 1004.0 | 1589.0 | retire |
| F-L-d1play-c | 1 of 10 | cycle 27 | 18–18 | 1016.2 | 1478.7 | retire |
| T-L-H-c | 1 of 10 | cycle 27 | 18–18 | 1008.3 | 1574.6 | retire |
| T-L-NLs-merge-c | 1 of 10 | cycle 27 | 18–18 | 969.1 | 1587.6 | retire |
| T-L-Ne-merge-c | 1 of 10 | cycle 27 | 18–18 | 979.3 | 1659.9 | retire |
| T-N-LLs-merge-c | 1 of 10 | cycle 27 | 18–18 | 646.6 | 1096.6 | freeze 2 at 1100 |
| T-N-NNs-c | 1 of 10 | cycle 27 | 18–18 | 647.7 | 1141.5 | retire |
| T-N-Le-merge-c | 0 of 10 | cycle 28 | - | - | - | retire |
| T-N-NLs-merge-c | 0 of 10 | cycle 28 | - | - | - | retire |
| T-N-Ne-merge-c | 0 of 10 | cycle 28 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4.


Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | dormant | 2856 |  |  | 1662 | 45 | 1700 | 1700 (1694) | v2855 (v2247) | 1518 | dormant (-31 ± 32 vs peak v2247 before dormancy at v2854, at 1549 vs refs; needs > +32) | 1570 (129255) | +448 -418 =134, 51.5%, +10 ± 20: kept | 2857 |
| F-L-d0soft-c | dormant | 2013 |  |  | 1642 | 45 | 1662 | 1662 (1654) | v2012 (v1619) | 1498 | dormant (-12 ± 33 vs peak v1619 before dormancy at v2006, at 1510 vs refs; needs > +33) | 1570 (125195) | +424 -432 =144, 49.6%, -3 ± 20: reverted | 2013 |
| F-L-d1jit-c | dormant | 1766 |  |  | 1603 | 43 | 1626 | 1702 (1702) | v1451 (v1451) | 1459 | dormant (-99 ± 31 vs peak v1451 before dormancy at v1763, at 1558 vs refs; needs > +31) | 1570 (136030) | +456 -375 =169, 54.0%, +28 ± 20: kept | 1767 |
| F-L-d1play-c | dormant | 3000 |  |  | 1479 | 41 | 1518 | 1564 (1564) | v1269 (v1269) | 1334 | dormant (-86 ± 32 vs peak v1269 before dormancy at v2999, at 1420 vs refs; needs > +32) | 1570 (143390) | +469 -396 =135, 53.6%, +25 ± 20: kept | 3001 |
| F-L-d2play-c | qualified | 1308 | 102 x 40 | 240 | 1686 | 46 | 1728 | 1759 | v1011 | 1542 | strike 1 (-72 ± 45 vs peak v1011; predecessor -42) | 1570 (132748) | +404 -396 =200, 50.4%, +3 ± 19: kept | 1309 |
| F-L-d2mix-c | qualified | 987 | 77 x 40 | 239 | 1689 | 46 | 1750 | 1750 | v910 | 1544 | strike 1 (-62 ± 50 vs peak v910; predecessor -62) | 1570 (130531) | +347 -406 =247, 47.0%, -20 ± 19: reverted | 987 |
| F-L-d2jit-c | qualified | 2444 | 125 x 40 | 241 | 1774 | 51 | 1762 | 1804 | v1651 | 1630 | ok (-30 ± 55 vs peak v1651; predecessor +12) | 1570 (136297) | +361 -427 =212, 46.7%, -23 ± 19: reverted | 2444 |
| F-N-c | dormant | 1685 |  |  | 1384 | 40 | 1404 | 1481 (1481) | v1061 (v1061) | 1240 | dormant (-97 ± 28 vs peak v1061 before dormancy at v1683, at 1337 vs refs; needs > +28) | 1570 (128587) | +439 -413 =148, 51.3%, +9 ± 20: kept | 1686 |
| MoE-c | dormant | 370 |  |  | 1706 | 47 | 1697 | 1726 (1726) | v300 (v300) | 1562 | dormant (-20 ± 33 vs peak v300 before dormancy at v367, at 1581 vs refs; needs > +33) | 1570 (130266) | +473 -400 =127, 53.6%, +25 ± 20: kept | 371 |
| FM-c | qualified | 462 | 21 x 1000 | 242 | 1410 | 40 | 1303 | 1447 | v419 | 1265 | ok (-37 ± 41 vs peak v419; predecessor +106) | 1570 (136119) | +618 -252 =130, 68.3%, +133 ± 21: kept | 463 |
| TAN-c | dormant | 929 |  |  | 1220 | 21 |  | 1244 (1244) | v507 (v507) | 1076 | dormant (-24 ± 22 vs peak v507 before dormancy at v928, at 1100 vs refs; needs > +22) | 1570 (135310) | +391 -477 =132, 45.7%, -30 ± 20: reverted | 929 |
| T-L-Ne-c | qualified | 656 | 37 x 1000 | 243 | 1560 | 42 | 1606 | 1614 | v556 | 1416 | strike 1 (-54 ± 36 vs peak v556; predecessor -46) | 1570 (134537) | +497 -391 =112, 55.3%, +37 ± 20: kept | 657 |
| T-L-Ne-merge-c | dormant | 1009 |  |  | 1660 | 45 | 1622 | 1711 (1711) | v794 (v794) | 1516 | dormant (-51 ± 26 vs peak v794 before dormancy at v1008, at 1567 vs refs; needs > +26) | 1570 (124314) | +444 -420 =136, 51.2%, +8 ± 20: kept | 1010 |
| T-L-Nc-c | dormant | 711 |  |  | 1579 | 43 | 1600 | 1653 (1653) | v516 (v516) | 1435 | dormant (-74 ± 31 vs peak v516 before dormancy at v709, at 1508 vs refs; needs > +31) | 1570 (138918) | +448 -363 =189, 54.2%, +30 ± 20: kept | 712 |
| T-L-Nc-merge-c | qualified | 1765 | 134 x 1000 | 240 | 1569 | 42 | 1735 | 1735 | v1631 | 1425 | strike 1 (-166 ± 47 vs peak v1631; predecessor -166) | 1570 (129688) | +486 -392 =122, 54.7%, +33 ± 20: kept | 1766 |
| T-L-LLs-c | dormant | 651 |  |  | 1612 | 44 | 1584 | 1638 (1638) | v540 (v540) | 1468 | dormant (-26 ± 31 vs peak v540 before dormancy at v647, at 1493 vs refs; needs > +31) | 1570 (137360) | +379 -412 =209, 48.4%, -12 ± 19: reverted | 651 |
| T-L-LLs-merge-c | qualified | 1695 | 49 x 1000 | 240 | 1508 | 41 | 1569 | 1613 | v1251 | 1364 | strike 1 (-104 ± 45 vs peak v1251; predecessor -60) | 1570 (134227) | +415 -421 =164, 49.7%, -2 ± 20: reverted | 1695 |
| T-L-NLs-c | qualified | 1048 | 46 x 1000 | 239 | 1518 | 41 | 1573 | 1573 | v1002 | 1373 | strike 1 (-56 ± 43 vs peak v1002; predecessor -56) | 1570 (134759) | +339 -511 =150, 41.4%, -60 ± 20: reverted | 1048 |
| T-L-NLs-merge-c | dormant | 1116 |  |  | 1588 | 43 | 1554 | 1656 (1656) | v641 (v641) | 1443 | dormant (-69 ± 41 vs peak v641 before dormancy at v1115, at 1512 vs refs; needs > +41) | 1570 (133596) | +485 -334 =181, 57.6%, +53 ± 20: kept | 1117 |
| T-L-NNs-c | qualified | 1028 | 47 x 1000 | 243 | 1568 | 42 | 1595 | 1610 | v561 | 1424 | ok (-42 ± 48 vs peak v561; predecessor -27) | 1570 (131243) | +410 -444 =146, 48.3%, -12 ± 20: reverted | 1028 |
| T-L-NNs-merge-c | dormant | 1068 |  |  | 1693 | 47 | 1712 | 1712 (1710) | v1067 (v818) | 1548 | dormant (-17 ± 34 vs peak v818 before dormancy at v1062, at 1565 vs refs; needs > +34) | 1570 (129546) | +462 -386 =152, 53.8%, +26 ± 20: kept | 1069 |
| T-L-Le-c | qualified | 941 | 43 x 1000 | 241 | 1486 | 41 | 1557 | 1594 | v696 | 1342 | strike 1 (-108 ± 44 vs peak v696; predecessor -71) | 1570 (127659) | +588 -314 =98, 63.7%, +98 ± 21: kept | 942 |
| T-L-Le-merge-c | dormant | 2051 |  |  | 1710 | 47 | 1663 | 1698 (1698) | v1857 (v1857) | 1566 | dormant (+12 ± 33 vs peak v1857 before dormancy at v2049, at 1554 vs refs; needs > +33) | 1570 (127757) | +472 -384 =144, 54.4%, +31 ± 20: kept | 2052 |
| T-N-Ne-c | dormant | 637 |  |  | 1288 | 38 | 1291 | 1376 (1376) | v574 (v574) | 1144 | dormant (-89 ± 29 vs peak v574 before dormancy at v633, at 1232 vs refs; needs > +29) | 1570 (131604) | +422 -437 =141, 49.2%, -5 ± 20: reverted | 637 |
| T-N-Ne-merge-c | dormant | 904 | 58 x 1000 | 240 | 1374 | 39 | 1397 | 1452 | v699 | 1230 | dormant (strike 2; -78 ± 37 vs peak v699; predecessor -23) | 1570 (128469) | +420 -424 =156, 49.8%, -1 ± 20: reverted | 904 |
| T-N-Nc-c | qualified | 592 | 39 x 1000 | 238 | 1341 | 39 | 1359 | 1395 | v472 | 1197 | strike 1 (-54 ± 47 vs peak v472; predecessor -18) | 1570 (133915) | +430 -422 =148, 50.4%, +3 ± 20: kept | 593 |
| T-N-Nc-merge-c | qualified | 919 | 72 x 1000 | 243 | 1434 | 40 | 1416 | 1455 | v631 | 1290 | ok (-20 ± 37 vs peak v631; predecessor +18) | 1570 (127781) | +409 -404 =187, 50.2%, +2 ± 19: kept | 920 |
| T-N-NNs-c | dormant | 626 |  |  | 1142 | 37 | 1178 | 1241 (1241) | v341 (v341) | 997 | dormant (-100 ± 39 vs peak v341 before dormancy at v625, at 1097 vs refs; needs > +39) | 1570 (139157) | +421 -408 =171, 50.6%, +4 ± 20: kept | 627 |
| T-N-NNs-merge-c | dormant | 998 |  |  | 1375 | 18 |  | 1472 (1472) | v855 (v855) | 1231 | dormant (-98 ± 23 vs peak v855 before dormancy at v997, at 1328 vs refs; needs > +23) | 1570 (128022) | +396 -378 =226, 50.9%, +6 ± 19: kept | 999 |
| T-N-Le-merge-c | dormant | 697 | 39 x 1000 | 239 | 1346 | 39 | 1347 | 1404 | v580 | 1202 | dormant (strike 2; -57 ± 41 vs peak v580; predecessor -0) | 1570 (132334) | +446 -395 =159, 52.5%, +18 ± 20: kept | 698 |
| T-N-Lc-merge-c | dormant | 366 |  |  | 1412 | 25 |  | 1444 (1444) | v287 (v287) | 1268 | dormant (-32 ± 23 vs peak v287 before dormancy at v364, at 1300 vs refs; needs > +23) | 1570 (128287) | +397 -419 =184, 48.9%, -8 ± 20: reverted | 366 |
| T-N-LLs-merge-c | dormant | 724 |  |  | 1097 | 37 | 1129 | 1377 (1377) | v585 (v585) | 952 | dormant (-281 ± 27 vs peak v585 before dormancy at v723, at 1233 vs refs; needs > +27) | 1570 (143932) | +459 -380 =161, 53.9%, +28 ± 20: kept | 725 |
| T-N-NLs-merge-c | dormant | 785 | 52 x 1000 | 238 | 1061 | 37 | 1193 | 1399 | v672 | 917 | dormant (strike 2; -337 ± 39 vs peak v672; predecessor -132) | 1570 (143785) | +650 -239 =111, 70.5%, +152 ± 22: kept | 786 |
| T-N-LNs-merge-c | dormant | 414 |  |  | 1240 | 38 | 1228 | 1354 (1354) | v270 (v270) | 1096 | dormant (-113 ± 27 vs peak v270 before dormancy at v410, at 1209 vs refs; needs > +27) | 1570 (136400) | +426 -423 =151, 50.1%, +1 ± 20: kept | 415 |
| F-N-d2jit-c | qualified | 1003 | 130 x 40 | 241 | 1114 | 37 | 1154 | 1181 | v389 | 969 | strike 1 (-67 ± 39 vs peak v389; predecessor -40) | 1570 (139658) | +432 -365 =203, 53.3%, +23 ± 19: kept | 1004 |
| T-N-NNs-merge-d2jit-c | qualified | 716 | 73 x 40 | 239 | 1174 | 38 | 1176 | 1176 | v642 | 1030 | ok (-2 ± 39 vs peak v642; predecessor -2) | 1570 (143292) | +396 -387 =217, 50.4%, +3 ± 19: kept | 717 |
| F-L-d4mix-c | qualified | 51 | 7 x 10 | 228 | 869 | 37 | 693 | 693 | v43 | 725 | new peak (+176 ± 39 over v43; predecessor +176) | 1570 (167525) | +710 -80 =210, 81.5%, +258 ± 22: kept | 52 |
| MoE-d2jit-c | qualified | 136 | 59 x 40 | 238 | 1179 | 38 | 966 | 966 | v76 | 1035 | new peak (+213 ± 38 over v76; predecessor +213) | 1570 (155502) | +542 -218 =240, 66.2%, +117 ± 20: kept | 137 |
| T-L-H-c | dormant | 1522 |  |  | 1575 | 43 | 1603 | 1664 (1664) | v1245 (v1245) | 1430 | dormant (-90 ± 31 vs peak v1245 before dormancy at v1521, at 1520 vs refs; needs > +31) | 1570 (134597) | +479 -366 =155, 55.6%, +39 ± 20: kept | 1523 |
| T-N-H-c | dormant | 1731 |  |  | 1286 | 24 |  | 1366 (1366) | v1104 (v1104) | 1141 | dormant (-80 ± 26 vs peak v1104 before dormancy at v1728, at 1222 vs refs; needs > +26) | 1570 (132828) | +438 -412 =150, 51.3%, +9 ± 20: kept | 1732 |
