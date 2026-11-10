# Cycle 3

**Own-games step, pooled over the lineages: 30 gates, +14864 -10906 =4230, score 56.6%, Elo +46.1 ± 3.7; kept 26 of 30.**

All cycles so far: 90 gates, +43905 -32695 =13400, score 56.2%, Elo +43.5 ± 2.1; kept 77 of 90.

2026-11-06 18:58: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then ran the rounds that fill the budget; round sizes d0 1000, d1 860, d2 40, d3 10.
League: retired 60 players to retired.txt (their games stay in the fit), added 60.
Tournament: 10 double round-robins of the 63 active players (39060 games) in 2.7 min, 39060 games saved.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the fixed references' mean, which is comparable across fits.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | qualified | 836 | 141 x 1000 | 257 | +1671 | 60 | +1627 | +1505 | up +44 | 2340 (187817) | +522 -375 =103, 57.4%, +51 ± 21: kept | 837 |
| F-L-d0soft-c | qualified | 1033 | 172 x 1000 | 254 | +1555 | 59 | +1447 | +1389 | up +109 | 2340 (204888) | +618 -275 =107, 67.2%, +124 ± 22: kept | 1034 |
| F-L-d1jit-c | qualified | 851 | 146 x 860 | 256 | +1603 | 60 | +1523 | +1436 | up +80 | 2340 (207179) | +635 -221 =144, 70.7%, +153 ± 22: kept | 852 |
| F-L-d1play-c | qualified | 854 | 138 x 860 | 244 | +1572 | 59 | +1543 | +1406 | up +30 | 2340 (218973) | +532 -330 =138, 60.1%, +71 ± 20: kept | 855 |
| F-L-d2play-c | qualified | 724 | 89 x 40 | 233 | +1706 | 61 | +1566 | +1539 | up +140 | 2340 (207346) | +467 -380 =153, 54.4%, +30 ± 20: kept | 725 |
| F-L-d2mix-c | qualified | 591 | 81 x 40 | 243 | +1766 | 62 | +1706 | +1600 | up +60 | 2340 (192487) | +403 -415 =182, 49.4%, -4 ± 20: reverted | 591 |
| F-L-d2jit-c | qualified | 745 | 93 x 40 | 215 | +1643 | 60 | +1455 | +1477 | up +188 | 2340 (215550) | +476 -362 =162, 55.7%, +40 ± 20: kept | 746 |
| F-N-c | qualified | 804 | 126 x 1000 | 237 | +1421 | 58 | +1343 | +1255 | up +78 | 2340 (193887) | +440 -402 =158, 51.9%, +13 ± 20: kept | 805 |
| MoE-c | qualified | 178 | 29 x 1000 | 264 | +1463 | 59 | +1348 | +1296 | up +115 | 2340 (210582) | +723 -180 =97, 77.1%, +211 ± 24: kept | 179 |
| FM-c | qualified | 138 | 21 x 1000 | 260 | +1191 | 58 | +1027 | +1024 | up +164 | 2340 (229443) | +577 -265 =158, 65.6%, +112 ± 21: kept | 139 |
| T-L-Ne-c | qualified | 375 | 42 x 1000 | 249 | +1587 | 59 | +1469 | +1421 | up +118 | 2340 (204660) | +505 -328 =167, 58.9%, +62 ± 20: kept | 376 |
| T-L-Ne-merge-c | qualified | 474 | 72 x 1000 | 156 | +1590 | 59 | +1539 | +1423 | up +50 | 2340 (192397) | +594 -297 =109, 64.8%, +106 ± 21: kept | 475 |
| T-L-Nc-c | qualified | 397 | 53 x 1000 | 274 | +1570 | 59 | +1502 | +1404 | up +68 | 2340 (205137) | +480 -371 =149, 55.5%, +38 ± 20: kept | 398 |
| T-L-Nc-merge-c | qualified | 576 | 100 x 1000 | 212 | +1673 | 60 | +1545 | +1507 | up +128 | 2340 (187492) | +534 -351 =115, 59.2%, +64 ± 21: kept | 577 |
| T-L-LLs-c | qualified | 375 | 44 x 1000 | 287 | +1548 | 59 | +1444 | +1382 | up +104 | 2340 (200856) | +437 -372 =191, 53.2%, +23 ± 19: kept | 376 |
| T-L-LLs-merge-c | qualified | 662 | 140 x 1000 | 306 | +1604 | 60 | +1540 | +1437 | up +64 | 2340 (193623) | +572 -324 =104, 62.4%, +88 ± 21: kept | 663 |
| T-L-NLs-c | qualified | 396 | 53 x 1000 | 314 | +1544 | 59 | +1482 | +1378 | up +63 | 2340 (208951) | +416 -398 =186, 50.9%, +6 ± 20: kept | 397 |
| T-L-NLs-merge-c | qualified | 499 | 83 x 1000 | 194 | +1633 | 60 | +1566 | +1466 | up +67 | 2340 (192833) | +542 -369 =89, 58.7%, +61 ± 21: kept | 500 |
| T-L-NNs-c | qualified | 397 | 47 x 1000 | 262 | +1579 | 59 | +1496 | +1413 | up +83 | 2340 (202213) | +476 -395 =129, 54.0%, +28 ± 20: kept | 398 |
| T-L-NNs-merge-c | qualified | 541 | 131 x 1000 | 257 | +1685 | 60 | +1608 | +1518 | up +76 | 2340 (193984) | +522 -371 =107, 57.6%, +53 ± 21: kept | 542 |
| T-L-Le-c | qualified | 374 | 48 x 1000 | 277 | +1566 | 59 | +1453 | +1400 | up +114 | 2340 (200458) | +454 -420 =126, 51.7%, +12 ± 20: kept | 375 |
| T-L-Le-merge-c | qualified | 621 | 148 x 1000 | 316 | +1651 | 60 | +1605 | +1484 | up +46 | 2340 (194547) | +534 -354 =112, 59.0%, +63 ± 21: kept | 622 |
| T-N-Ne-c | qualified | 358 | 50 x 1000 | 250 | +1303 | 58 | +1257 | +1137 | up +46 | 2340 (200456) | +454 -391 =155, 53.1%, +22 ± 20: kept | 359 |
| T-N-Ne-merge-c | qualified | 500 | 101 x 1000 | 213 | +1444 | 59 | +1319 | +1278 | up +125 | 2340 (194680) | +391 -449 =160, 47.1%, -20 ± 20: reverted | 500 |
| T-N-Nc-c | qualified | 390 | 46 x 1000 | 247 | +1398 | 58 | +1286 | +1232 | up +112 | 2340 (195909) | +403 -418 =179, 49.2%, -5 ± 20: reverted | 390 |
| T-N-Nc-merge-c | qualified | 487 | 90 x 1000 | 200 | +1451 | 59 | +1329 | +1285 | up +122 | 2340 (190143) | +432 -405 =163, 51.3%, +9 ± 20: kept | 488 |
| T-N-NNs-c | qualified | 341 | 49 x 1000 | 268 | +1268 | 58 | +1192 | +1101 | up +76 | 2340 (198034) | +382 -449 =169, 46.7%, -23 ± 20: reverted | 341 |
| T-N-NNs-merge-c | qualified | 415 | 82 x 1000 | 288 | +1281 | 58 | +1257 | +1114 | up +24 | 2340 (209113) | +451 -406 =143, 52.2%, +16 ± 20: kept | 416 |
| T-L-H-c | qualified | 618 | 139 x 1000 | 263 | +1575 | 59 | +1507 | +1408 | up +68 | 2340 (195502) | +461 -420 =119, 52.0%, +14 ± 20: kept | 619 |
| T-N-H-c | qualified | 601 | 130 x 1000 | 234 | +1326 | 58 | +1178 | +1159 | up +148 | 2340 (202637) | +431 -413 =156, 50.9%, +6 ± 20: kept | 602 |
