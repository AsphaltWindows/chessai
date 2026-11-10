# Cycle 7

**Own-games step, pooled over the lineages: 31 gates, +14156 -12154 =4690, score 53.2%, Elo +22.5 ± 3.6; kept 20 of 31.**

All cycles so far: 212 gates, +101165 -79127 =31708, score 55.2%, Elo +36.2 ± 1.4; kept 174 of 212.

2026-11-06 21:04: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then ran the rounds that fill the budget; round sizes d0 1000, d1 860, d2 40, d3 10.
League: retired 60 players to retired.txt (their games stay in the fit), added 60.
Tournament: 5 double round-robins of the 67 active players (22110 games) in 2.0 min, 22110 games saved.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: a lineage whose new version rates below its predecessor gets a strike; 2 in a row make it dormant. A dormant lineage re-qualifies when its current version's vs refs beats its peak, the best vs refs of its earlier versions at ply 2 in this fit (retired ones included), by more than 1 x the difference's 95% range (the ± in the decision: 1.96 standard errors from the fit's covariance, which carries the shift a whole cycle shares); dormant rows show the gap.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | qualified | 1455 | 139 x 1000 | 261 | +1731 | 57 | +1719 | +1565 | up +12 | 1210 (98804) | +522 -352 =126, 58.5%, +60 ± 20: kept | 1456 |
| F-L-d0soft-c | dormant | 1618 |  |  | +1660 | 56 | +1603 | +1495 | dormant (+50 ± 68 vs peak v1457 at 1445 vs refs; needs > +68) | 1210 (102028) | +509 -371 =120, 56.9%, +48 ± 20: kept | 1619 |
| F-L-d1jit-c | qualified | 1290 | 149 x 860 | 261 | +1652 | 56 | +1701 | +1487 | strike 1 (-49) | 1210 (109107) | +539 -314 =147, 61.3%, +80 ± 20: kept | 1291 |
| F-L-d1play-c | qualified | 1412 | 142 x 860 | 250 | +1565 | 55 | +1637 | +1399 | strike 1 (-73) | 1210 (110525) | +474 -364 =162, 55.5%, +38 ± 20: kept | 1413 |
| F-L-d2play-c | dormant | 1010 |  |  | +1782 | 59 | +1770 | +1617 | dormant (+3 ± 74 vs peak v913 at 1614 vs refs; needs > +74) | 1210 (103693) | +366 -482 =152, 44.2%, -40 ± 20: reverted | 1010 |
| F-L-d2mix-c | dormant | 750 |  |  | +1807 | 42 |  | +1641 | dormant (+40 ± 87 vs peak v591 at 1601 vs refs; needs > +87) | 1210 (105491) | +352 -452 =196, 45.0%, -35 ± 19: reverted | 750 |
| F-L-d2jit-c | qualified | 1180 | 108 x 40 | 231 | +1775 | 59 | +1835 | +1610 | strike 1 (-60) | 1210 (103226) | +459 -381 =160, 53.9%, +27 ± 20: kept | 1181 |
| F-N-c | dormant | 1058 |  |  | +1510 | 55 | +1497 | +1344 | dormant (+12 ± 67 vs peak v1057 at 1332 vs refs; needs > +67) | 1210 (100198) | +422 -416 =162, 50.3%, +2 ± 20: kept | 1059 |
| MoE-c | qualified | 266 | 32 x 1000 | 278 | +1520 | 55 | +1700 | +1354 | strike 1 (-180) | 1210 (104452) | +738 -180 =82, 77.9%, +219 ± 24: kept | 267 |
| FM-c | qualified | 223 | 21 x 1000 | 254 | +1348 | 54 | +1204 | +1183 | up +144 | 1210 (108973) | +608 -266 =126, 67.1%, +124 ± 21: kept | 224 |
| TAN-c | qualified | 175 | 106 x 1000 | 235 | +1287 | 54 | +1228 | +1122 | up +59 | 1210 (103691) | +458 -391 =151, 53.3%, +23 ± 20: kept | 176 |
| T-L-Ne-c | qualified | 519 | 36 x 1000 | 247 | +1661 | 56 | +1680 | +1496 | strike 1 (-19) | 1210 (102889) | +421 -425 =154, 49.8%, -1 ± 20: reverted | 519 |
| T-L-Ne-merge-c | dormant | 791 |  |  | +1718 | 57 | +1704 | +1552 | dormant (-14 ± 71 vs peak v700 at 1567 vs refs; needs > +71) | 1210 (95800) | +416 -430 =154, 49.3%, -5 ± 20: reverted | 791 |
| T-L-Nc-c | dormant | 512 |  |  | +1645 | 56 | +1652 | +1479 | dormant (-14 ± 69 vs peak v477 at 1493 vs refs; needs > +69) | 1210 (103629) | +421 -373 =206, 52.4%, +17 ± 19: kept | 513 |
| T-L-Nc-merge-c | qualified | 980 | 146 x 1000 | 318 | +1710 | 57 | +1768 | +1545 | strike 1 (-57) | 1210 (97862) | +521 -356 =123, 58.2%, +58 ± 20: kept | 981 |
| T-L-LLs-c | qualified | 504 | 31 x 1000 | 237 | +1638 | 56 | +1638 | +1472 | strike 1 (-1) | 1210 (104217) | +493 -333 =174, 58.0%, +56 ± 20: kept | 505 |
| T-L-LLs-merge-c | qualified | 1075 | 64 x 1000 | 372 | +1637 | 56 | +1674 | +1471 | strike 1 (-38) | 1210 (102726) | +447 -388 =165, 52.9%, +20 ± 20: kept | 1076 |
| T-L-NLs-c | qualified | 571 | 43 x 1000 | 291 | +1640 | 56 | +1598 | +1475 | up +42 | 1210 (103614) | +330 -495 =175, 41.8%, -58 ± 20: reverted | 571 |
| T-L-NLs-merge-c | qualified | 918 | 60 x 1000 | 286 | +1664 | 56 | +1674 | +1498 | strike 1 (-11) | 1210 (98918) | +405 -444 =151, 48.0%, -14 ± 20: reverted | 918 |
| T-L-NNs-c | qualified | 561 | 33 x 1000 | 243 | +1677 | 56 | +1666 | +1512 | up +11 | 1210 (104402) | +419 -440 =141, 48.9%, -7 ± 20: reverted | 561 |
| T-L-NNs-merge-c | dormant | 817 |  |  | +1728 | 57 | +1766 | +1563 | dormant (-38 ± 71 vs peak v816 at 1601 vs refs; needs > +71) | 1210 (99318) | +456 -407 =137, 52.4%, +17 ± 20: kept | 818 |
| T-L-Le-c | qualified | 508 | 28 x 1000 | 252 | +1640 | 56 | +1610 | +1475 | up +30 | 1210 (100389) | +481 -391 =128, 54.5%, +31 ± 20: kept | 509 |
| T-L-Le-merge-c | qualified | 1083 | 65 x 1000 | 226 | +1732 | 57 | +1728 | +1567 | up +4 | 1210 (94437) | +512 -366 =122, 57.3%, +51 ± 20: kept | 1084 |
| T-N-Ne-c | qualified | 505 | 34 x 1000 | 243 | +1433 | 55 | +1425 | +1267 | up +7 | 1210 (101136) | +401 -453 =146, 47.4%, -18 ± 20: reverted | 505 |
| T-N-Ne-merge-c | dormant | 697 |  |  | +1525 | 55 | +1480 | +1360 | dormant (+45 ± 67 vs peak v696 at 1315 vs refs; needs > +67) | 1210 (100309) | +419 -431 =150, 49.4%, -4 ± 20: reverted | 697 |
| T-N-Nc-c | dormant | 473 |  |  | +1438 | 55 | +1467 | +1273 | dormant (-29 ± 67 vs peak v472 at 1302 vs refs; needs > +67) | 1210 (99545) | +431 -405 =164, 51.3%, +9 ± 20: kept | 474 |
| T-N-Nc-merge-c | dormant | 629 |  |  | +1493 | 55 | +1519 | +1327 | dormant (-27 ± 59 vs peak v628 at 1354 vs refs; needs > +59) | 1210 (98878) | +429 -385 =186, 52.2%, +15 ± 20: kept | 630 |
| T-N-NNs-c | qualified | 496 | 43 x 1000 | 256 | +1261 | 54 | +1310 | +1096 | strike 1 (-49) | 1210 (105880) | +416 -418 =166, 49.9%, -1 ± 20: reverted | 496 |
| T-N-NNs-merge-c | qualified | 749 | 97 x 1000 | 369 | +1531 | 55 | +1528 | +1366 | up +2 | 1210 (97642) | +402 -423 =175, 48.9%, -7 ± 20: reverted | 749 |
| T-L-H-c | qualified | 1106 | 136 x 1000 | 253 | +1661 | 56 | +1712 | +1496 | strike 1 (-51) | 1210 (102757) | +463 -406 =131, 52.8%, +20 ± 20: kept | 1107 |
| T-N-H-c | qualified | 1104 | 131 x 1000 | 226 | +1434 | 55 | +1427 | +1268 | up +7 | 1210 (101058) | +426 -416 =158, 50.5%, +4 ± 20: kept | 1105 |
