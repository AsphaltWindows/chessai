# Cycle 4

**Own-games step, pooled over the lineages: 30 gates, +14923 -10803 =4274, score 56.9%, Elo +48.0 ± 3.7; kept 27 of 30.**

All cycles so far: 120 gates, +58828 -43498 =17674, score 56.4%, Elo +44.6 ± 1.8; kept 104 of 120.

2026-11-06 19:33: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then ran the rounds that fill the budget; round sizes d0 1000, d1 860, d2 40, d3 10.
League: retired 60 players to retired.txt (their games stay in the fit), added 60.
Tournament: 10 double round-robins of the 63 active players (39060 games) in 2.5 min, 39060 games saved.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the fixed references' mean, which is comparable across fits.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | qualified | 980 | 143 x 1000 | 268 | +1654 | 60 | +1690 | +1474 | strike 1 (-37) | 2340 (193249) | +538 -352 =110, 59.3%, +65 ± 21: kept | 981 |
| F-L-d0soft-c | qualified | 1203 | 169 x 1000 | 256 | +1593 | 60 | +1575 | +1413 | up +18 | 2340 (200324) | +583 -297 =120, 64.3%, +102 ± 21: kept | 1204 |
| F-L-d1jit-c | qualified | 991 | 139 x 860 | 257 | +1604 | 60 | +1622 | +1424 | strike 1 (-18) | 2340 (205971) | +581 -297 =122, 64.2%, +102 ± 21: kept | 992 |
| F-L-d1play-c | qualified | 987 | 132 x 860 | 241 | +1557 | 60 | +1592 | +1378 | strike 1 (-34) | 2340 (218247) | +569 -315 =116, 62.7%, +90 ± 21: kept | 988 |
| F-L-d2play-c | qualified | 818 | 93 x 40 | 242 | +1780 | 62 | +1725 | +1600 | up +55 | 2340 (200979) | +358 -471 =171, 44.4%, -39 ± 20: reverted | 818 |
| F-L-d2mix-c | qualified | 668 | 77 x 40 | 235 | +1785 | 63 | +1786 | +1605 | strike 1 (-1) | 2340 (195921) | +355 -439 =206, 45.8%, -29 ± 19: reverted | 668 |
| F-L-d2jit-c | qualified | 847 | 101 x 40 | 212 | +1740 | 62 | +1662 | +1561 | up +78 | 2340 (203297) | +446 -386 =168, 53.0%, +21 ± 20: kept | 848 |
| F-N-c | qualified | 929 | 124 x 1000 | 246 | +1435 | 59 | +1441 | +1255 | strike 1 (-6) | 2340 (191108) | +407 -402 =191, 50.2%, +2 ± 19: kept | 930 |
| MoE-c | qualified | 206 | 27 x 1000 | 259 | +1307 | 59 | +1482 | +1128 | strike 1 (-175) | 2340 (212245) | +823 -109 =68, 85.7%, +311 ± 29: kept | 207 |
| FM-c | qualified | 158 | 19 x 1000 | 240 | +1273 | 59 | +1210 | +1093 | up +63 | 2340 (219373) | +623 -263 =114, 68.0%, +131 ± 22: kept | 159 |
| T-L-Ne-c | qualified | 413 | 37 x 1000 | 246 | +1611 | 60 | +1606 | +1431 | up +5 | 2340 (209247) | +533 -311 =156, 61.1%, +78 ± 20: kept | 414 |
| T-L-Ne-merge-c | qualified | 562 | 87 x 1000 | 233 | +1696 | 61 | +1609 | +1517 | up +87 | 2340 (195215) | +540 -334 =126, 60.3%, +73 ± 20: kept | 563 |
| T-L-Nc-c | qualified | 436 | 38 x 1000 | 268 | +1615 | 60 | +1590 | +1436 | up +26 | 2340 (203032) | +482 -360 =158, 56.1%, +43 ± 20: kept | 437 |
| T-L-Nc-merge-c | qualified | 692 | 115 x 1000 | 312 | +1691 | 61 | +1692 | +1511 | strike 1 (-2) | 2340 (192174) | +520 -382 =98, 56.9%, +48 ± 21: kept | 693 |
| T-L-LLs-c | qualified | 408 | 32 x 1000 | 243 | +1582 | 60 | +1568 | +1403 | up +15 | 2340 (204483) | +491 -363 =146, 56.4%, +45 ± 20: kept | 409 |
| T-L-LLs-merge-c | qualified | 793 | 130 x 1000 | 292 | +1650 | 60 | +1623 | +1471 | up +27 | 2340 (192343) | +557 -318 =125, 62.0%, +85 ± 21: kept | 794 |
| T-L-NLs-c | qualified | 445 | 48 x 1000 | 266 | +1525 | 59 | +1564 | +1346 | strike 1 (-38) | 2340 (204634) | +406 -404 =190, 50.1%, +1 ± 19: kept | 446 |
| T-L-NLs-merge-c | qualified | 641 | 141 x 1000 | 317 | +1702 | 61 | +1652 | +1522 | up +49 | 2340 (200183) | +509 -370 =121, 57.0%, +49 ± 20: kept | 642 |
| T-L-NNs-c | qualified | 444 | 46 x 1000 | 265 | +1623 | 60 | +1599 | +1444 | up +25 | 2340 (205059) | +484 -379 =137, 55.2%, +37 ± 20: kept | 445 |
| T-L-NNs-merge-c | qualified | 683 | 141 x 1000 | 302 | +1691 | 61 | +1704 | +1512 | strike 1 (-13) | 2340 (191752) | +520 -376 =104, 57.2%, +50 ± 21: kept | 684 |
| T-L-Le-c | qualified | 411 | 36 x 1000 | 257 | +1570 | 60 | +1586 | +1390 | strike 1 (-16) | 2340 (198937) | +511 -352 =137, 58.0%, +56 ± 20: kept | 412 |
| T-L-Le-merge-c | qualified | 736 | 114 x 1000 | 258 | +1687 | 61 | +1670 | +1508 | up +17 | 2340 (194144) | +513 -381 =106, 56.6%, +46 ± 20: kept | 737 |
| T-N-Ne-c | qualified | 403 | 44 x 1000 | 274 | +1360 | 59 | +1322 | +1180 | up +37 | 2340 (193121) | +471 -384 =145, 54.4%, +30 ± 20: kept | 404 |
| T-N-Ne-merge-c | qualified | 623 | 123 x 1000 | 300 | +1437 | 59 | +1463 | +1258 | strike 1 (-26) | 2340 (193921) | +445 -384 =171, 53.0%, +21 ± 20: kept | 624 |
| T-N-Nc-c | qualified | 431 | 41 x 1000 | 278 | +1409 | 59 | +1417 | +1230 | strike 1 (-8) | 2340 (193729) | +406 -414 =180, 49.6%, -3 ± 20: reverted | 431 |
| T-N-Nc-merge-c | qualified | 559 | 71 x 1000 | 169 | +1466 | 59 | +1471 | +1287 | strike 1 (-4) | 2340 (188239) | +424 -392 =184, 51.6%, +11 ± 20: kept | 560 |
| T-N-NNs-c | qualified | 378 | 37 x 1000 | 258 | +1240 | 58 | +1287 | +1060 | strike 1 (-47) | 2340 (204130) | +416 -399 =185, 50.8%, +6 ± 20: kept | 379 |
| T-N-NNs-merge-c | qualified | 466 | 50 x 1000 | 211 | +1448 | 59 | +1300 | +1268 | up +148 | 2340 (194760) | +458 -382 =160, 53.8%, +26 ± 20: kept | 467 |
| T-L-H-c | qualified | 733 | 114 x 1000 | 239 | +1613 | 60 | +1594 | +1433 | up +19 | 2340 (199308) | +525 -360 =115, 58.2%, +58 ± 20: kept | 734 |
| T-N-H-c | qualified | 728 | 126 x 1000 | 236 | +1332 | 59 | +1345 | +1152 | strike 1 (-13) | 2340 (198552) | +429 -427 =144, 50.1%, +1 ± 20: kept | 729 |
