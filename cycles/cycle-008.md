# Cycle 8

**Own-games step, pooled over the lineages: 31 gates, +13942 -12190 =4868, score 52.8%, Elo +19.7 ± 3.6; kept 20 of 31.**

All cycles so far: 243 gates, +115107 -91317 =36576, score 54.9%, Elo +34.1 ± 1.3; kept 194 of 243.

2026-11-06 21:30: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then ran the rounds that fill the budget; round sizes d0 1000, d1 860, d2 40, d3 10.
League: retired 54 players to retired.txt (their games stay in the fit), added 54.
Tournament: 5 double round-robins of the 67 active players (22110 games) in 2.0 min, 22110 games saved.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: a lineage whose new version rates below its predecessor gets a strike; 2 in a row make it dormant. A dormant lineage re-qualifies when its current version's vs refs beats its peak, the best vs refs of its earlier versions at ply 2 in this fit (retired ones included), by more than 1 x the difference's 95% range (the ± in the decision: 1.96 standard errors from the fit's covariance, which carries the shift a whole cycle shares); dormant rows show the gap.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | qualified | 1607 | 151 x 1000 | 267 | +1693 | 52 | +1724 | +1533 | strike 1 (-31) | 1210 (97736) | +485 -398 =117, 54.4%, +30 ± 20: kept | 1608 |
| F-L-d0soft-c | dormant | 1619 |  |  | +1712 | 53 | +1653 | +1552 | dormant (+60 ± 63 vs peak v1618 at 1493 vs refs; needs > +63) | 1210 (96301) | +448 -424 =128, 51.2%, +8 ± 20: kept | 1620 |
| F-L-d1jit-c | dormant | 1447 | 156 x 860 | 262 | +1625 | 51 | +1644 | +1465 | dormant (-20, strike 2; -67 ± 67 vs peak v1140) | 1210 (105234) | +492 -327 =181, 58.2%, +58 ± 20: kept | 1448 |
| F-L-d1play-c | qualified | 1560 | 147 x 860 | 255 | +1558 | 50 | +1556 | +1398 | up +1 | 1210 (109649) | +483 -361 =156, 56.1%, +43 ± 20: kept | 1561 |
| F-L-d2play-c | dormant | 1010 |  |  | +1788 | 44 |  | +1627 | dormant (+18 ± 65 vs peak v913 at 1610 vs refs; needs > +65) | 1210 (104617) | +446 -375 =179, 53.5%, +25 ± 20: kept | 1011 |
| F-L-d2mix-c | dormant | 750 |  |  | +1802 | 35 |  | +1641 | dormant (+41 ± 83 vs peak v591 at 1600 vs refs; needs > +83) | 1210 (99954) | +389 -404 =207, 49.2%, -5 ± 19: reverted | 750 |
| F-L-d2jit-c | qualified | 1297 | 116 x 40 | 240 | +1808 | 57 | +1768 | +1648 | up +40 | 1210 (102844) | +349 -464 =187, 44.2%, -40 ± 20: reverted | 1297 |
| F-N-c | dormant | 1059 |  |  | +1516 | 50 | +1501 | +1356 | dormant (+16 ± 60 vs peak v1058 at 1341 vs refs; needs > +60) | 1210 (97119) | +412 -407 =181, 50.2%, +2 ± 20: kept | 1060 |
| MoE-c | dormant | 296 | 29 x 1000 | 245 | +1397 | 50 | +1511 | +1237 | dormant (-114, strike 2; -294 ± 66 vs peak v233) | 1210 (101963) | +691 -215 =94, 73.8%, +180 ± 23: kept | 297 |
| FM-c | qualified | 246 | 22 x 1000 | 256 | +1380 | 50 | +1338 | +1220 | up +43 | 1210 (106754) | +548 -290 =162, 62.9%, +92 ± 20: kept | 247 |
| TAN-c | qualified | 283 | 107 x 1000 | 224 | +1245 | 49 | +1276 | +1085 | strike 1 (-31) | 1210 (104586) | +536 -305 =159, 61.6%, +82 ± 20: kept | 284 |
| T-L-Ne-c | dormant | 555 | 36 x 1000 | 228 | +1643 | 51 | +1654 | +1483 | dormant (-11, strike 2; -28 ± 67 vs peak v483) | 1210 (101887) | +484 -377 =139, 55.4%, +37 ± 20: kept | 556 |
| T-L-Ne-merge-c | dormant | 791 |  |  | +1730 | 42 |  | +1570 | dormant (+7 ± 62 vs peak v700 at 1563 vs refs; needs > +62) | 1210 (99957) | +454 -410 =136, 52.2%, +15 ± 20: kept | 792 |
| T-L-Nc-c | dormant | 513 |  |  | +1642 | 51 | +1637 | +1482 | dormant (-7 ± 67 vs peak v477 at 1489 vs refs; needs > +67) | 1210 (105069) | +421 -382 =197, 51.9%, +14 ± 19: kept | 514 |
| T-L-Nc-merge-c | qualified | 1132 | 151 x 1000 | 324 | +1705 | 52 | +1703 | +1544 | up +2 | 1210 (97191) | +495 -390 =115, 55.2%, +37 ± 20: kept | 1133 |
| T-L-LLs-c | dormant | 538 | 33 x 1000 | 248 | +1564 | 50 | +1630 | +1404 | dormant (-66, strike 2; -66 ± 61 vs peak v504) | 1210 (97726) | +494 -334 =172, 58.0%, +56 ± 20: kept | 539 |
| T-L-LLs-merge-c | qualified | 1136 | 60 x 1000 | 258 | +1663 | 52 | +1629 | +1502 | up +34 | 1210 (100202) | +383 -459 =158, 46.2%, -26 ± 20: reverted | 1136 |
| T-L-NLs-c | qualified | 609 | 38 x 1000 | 215 | +1596 | 50 | +1633 | +1436 | strike 1 (-36) | 1210 (100126) | +310 -517 =173, 39.6%, -73 ± 20: reverted | 609 |
| T-L-NLs-merge-c | dormant | 994 | 76 x 1000 | 276 | +1625 | 51 | +1656 | +1465 | dormant (-32, strike 2; -51 ± 88 vs peak v641) | 1210 (105902) | +340 -476 =184, 43.2%, -48 ± 20: reverted | 994 |
| T-L-NNs-c | qualified | 602 | 41 x 1000 | 235 | +1648 | 51 | +1670 | +1488 | strike 1 (-22) | 1210 (101171) | +408 -438 =154, 48.5%, -10 ± 20: reverted | 602 |
| T-L-NNs-merge-c | dormant | 818 |  |  | +1768 | 55 | +1721 | +1608 | dormant (+11 ± 71 vs peak v816 at 1597 vs refs; needs > +71) | 1210 (96779) | +481 -363 =156, 55.9%, +41 ± 20: kept | 819 |
| T-L-Le-c | qualified | 544 | 35 x 1000 | 251 | +1619 | 51 | +1633 | +1459 | strike 1 (-14) | 1210 (100178) | +473 -395 =132, 53.9%, +27 ± 20: kept | 545 |
| T-L-Le-merge-c | qualified | 1241 | 157 x 1000 | 353 | +1624 | 51 | +1725 | +1464 | strike 1 (-101) | 1210 (100327) | +530 -348 =122, 59.1%, +64 ± 20: kept | 1242 |
| T-N-Ne-c | qualified | 537 | 32 x 1000 | 231 | +1402 | 50 | +1423 | +1242 | strike 1 (-21) | 1210 (100762) | +410 -459 =131, 47.5%, -17 ± 20: reverted | 537 |
| T-N-Ne-merge-c | dormant | 697 |  |  | +1510 | 40 |  | +1350 | dormant (+40 ± 58 vs peak v696 at 1310 vs refs; needs > +58) | 1210 (93590) | +429 -407 =164, 51.1%, +8 ± 20: kept | 698 |
| T-N-Nc-c | dormant | 474 |  |  | +1406 | 50 | +1429 | +1246 | dormant (-52 ± 65 vs peak v472 at 1297 vs refs; needs > +65) | 1210 (100816) | +412 -424 =164, 49.4%, -4 ± 20: reverted | 474 |
| T-N-Nc-merge-c | dormant | 630 |  |  | +1502 | 50 | +1484 | +1341 | dormant (-8 ± 57 vs peak v628 at 1350 vs refs; needs > +57) | 1210 (94993) | +388 -430 =182, 47.9%, -15 ± 20: reverted | 630 |
| T-N-NNs-c | dormant | 536 | 40 x 1000 | 240 | +1212 | 49 | +1250 | +1052 | dormant (-38, strike 2; -88 ± 64 vs peak v452) | 1210 (105841) | +411 -416 =173, 49.8%, -2 ± 20: reverted | 536 |
| T-N-NNs-merge-c | qualified | 800 | 51 x 1000 | 230 | +1490 | 50 | +1522 | +1330 | strike 1 (-32) | 1210 (95895) | +397 -421 =182, 48.8%, -8 ± 20: reverted | 800 |
| T-L-H-c | dormant | 1240 | 133 x 1000 | 255 | +1654 | 51 | +1654 | +1494 | dormant (+0, strike 2; -49 ± 67 vs peak v970) | 1210 (103126) | +511 -347 =142, 58.2%, +58 ± 20: kept | 1241 |
| T-N-H-c | qualified | 1232 | 127 x 1000 | 208 | +1380 | 50 | +1424 | +1220 | strike 1 (-44) | 1210 (99420) | +432 -427 =141, 50.2%, +2 ± 20: kept | 1233 |
