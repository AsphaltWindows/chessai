# Cycle 2

**Own-games step, pooled over the lineages: 30 gates, +14437 -10997 =4566, score 55.7%, Elo +40.0 ± 3.6; kept 26 of 30.**

All cycles so far: 60 gates, +29041 -21789 =9170, score 56.0%, Elo +42.2 ± 2.6; kept 51 of 60.

2026-11-06 18:23: budget 480 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then ran the rounds that fill the budget; round sizes d0 1000, d1 860, d2 40, d3 10.
League: retired 60 players to retired.txt (their games stay in the fit), added 60.
Tournament: 5 double round-robins of the 63 active players (19530 games) in 1.0 min, 19530 games saved.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the fixed references' mean, which is comparable across fits.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | qualified | 694 | 269 x 1000 | 489 | +1657 | 67 | +1656 | +1467 | up +1 | 1170 (96836) | +499 -369 =132, 56.5%, +45 ± 20: kept | 695 |
| F-L-d0soft-c | qualified | 860 | 322 x 1000 | 480 | +1477 | 63 | +1435 | +1287 | up +42 | 1170 (102731) | +540 -332 =128, 60.4%, +73 ± 20: kept | 861 |
| F-L-d1jit-c | qualified | 704 | 280 x 860 | 507 | +1553 | 64 | +1547 | +1363 | up +6 | 1170 (105125) | +605 -254 =141, 67.5%, +127 ± 21: kept | 705 |
| F-L-d1play-c | qualified | 715 | 268 x 860 | 480 | +1573 | 65 | +1614 | +1383 | strike 1 (-41) | 1170 (107213) | +498 -362 =140, 56.8%, +48 ± 20: kept | 716 |
| F-L-d2play-c | qualified | 634 | 179 x 40 | 477 | +1596 | 65 | +1481 | +1407 | up +116 | 1170 (106680) | +475 -354 =171, 56.0%, +42 ± 20: kept | 635 |
| F-L-d2mix-c | qualified | 510 | 164 x 40 | 491 | +1737 | 71 | +1725 | +1547 | up +12 | 1170 (96118) | +397 -435 =168, 48.1%, -13 ± 20: reverted | 510 |
| F-L-d2jit-c | qualified | 651 | 177 x 40 | 472 | +1485 | 64 | +1294 | +1295 | up +191 | 1170 (115025) | +473 -327 =200, 57.3%, +51 ± 19: kept | 652 |
| F-N-c | qualified | 678 | 227 x 1000 | 466 | +1374 | 63 | +1335 | +1184 | up +39 | 1170 (95391) | +423 -424 =153, 50.0%, -0 ± 20: reverted | 678 |
| MoE-c | qualified | 148 | 48 x 1000 | 468 | +1378 | 63 | +1368 | +1188 | up +10 | 1170 (103853) | +685 -184 =131, 75.0%, +191 ± 23: kept | 149 |
| FM-c | qualified | 116 | 37 x 1000 | 498 | +1058 | 61 | +888 | +868 | up +169 | 1170 (107209) | +556 -223 =221, 66.6%, +120 ± 20: kept | 117 |
| T-L-Ne-c | qualified | 332 | 81 x 1000 | 467 | +1499 | 64 | +1491 | +1309 | up +8 | 1170 (103258) | +435 -420 =145, 50.7%, +5 ± 20: kept | 333 |
| T-L-Ne-merge-c | qualified | 401 | 176 x 1000 | 420 | +1569 | 65 | +1528 | +1380 | up +41 | 1170 (93984) | +533 -336 =131, 59.9%, +69 ± 20: kept | 402 |
| T-L-Nc-c | qualified | 343 | 115 x 1000 | 507 | +1532 | 64 | +1521 | +1343 | up +11 | 1170 (102893) | +426 -391 =183, 51.7%, +12 ± 20: kept | 344 |
| T-L-Nc-merge-c | qualified | 475 | 243 x 1000 | 507 | +1576 | 65 | +1532 | +1386 | up +44 | 1170 (95323) | +493 -361 =146, 56.6%, +46 ± 20: kept | 476 |
| T-L-LLs-c | qualified | 330 | 100 x 1000 | 508 | +1475 | 63 | +1446 | +1285 | up +29 | 1170 (105853) | +423 -399 =178, 51.2%, +8 ± 20: kept | 331 |
| T-L-LLs-merge-c | qualified | 521 | 288 x 1000 | 574 | +1571 | 65 | +1527 | +1381 | up +43 | 1170 (98367) | +576 -330 =94, 62.3%, +87 ± 21: kept | 522 |
| T-L-NLs-c | qualified | 342 | 115 x 1000 | 480 | +1512 | 64 | +1474 | +1322 | up +38 | 1170 (103429) | +448 -367 =185, 54.0%, +28 ± 20: kept | 343 |
| T-L-NLs-merge-c | qualified | 415 | 189 x 1000 | 392 | +1596 | 65 | +1551 | +1407 | up +46 | 1170 (95940) | +533 -342 =125, 59.6%, +67 ± 20: kept | 416 |
| T-L-NNs-c | qualified | 349 | 119 x 1000 | 533 | +1526 | 64 | +1487 | +1337 | up +39 | 1170 (98714) | +433 -417 =150, 50.8%, +6 ± 20: kept | 350 |
| T-L-NNs-merge-c | qualified | 409 | 184 x 1000 | 388 | +1638 | 67 | +1497 | +1449 | up +142 | 1170 (91623) | +510 -378 =112, 56.6%, +46 ± 20: kept | 410 |
| T-L-Le-c | qualified | 325 | 133 x 1000 | 630 | +1483 | 63 | +1478 | +1293 | up +5 | 1170 (100785) | +525 -328 =147, 59.9%, +69 ± 20: kept | 326 |
| T-L-Le-merge-c | qualified | 472 | 272 x 1000 | 564 | +1636 | 66 | +1511 | +1446 | up +125 | 1170 (97233) | +505 -389 =106, 55.8%, +40 ± 20: kept | 473 |
| T-N-Ne-c | qualified | 308 | 118 x 1000 | 491 | +1288 | 62 | +1288 | +1098 | strike 1 (-1) | 1170 (100177) | +403 -435 =162, 48.4%, -11 ± 20: reverted | 308 |
| T-N-Ne-merge-c | qualified | 398 | 209 x 1000 | 690 | +1349 | 62 | +1328 | +1159 | up +22 | 1170 (97467) | +410 -396 =194, 50.7%, +5 ± 19: kept | 399 |
| T-N-Nc-c | qualified | 343 | 149 x 1000 | 700 | +1316 | 62 | +1322 | +1126 | strike 1 (-6) | 1170 (99542) | +436 -395 =169, 52.0%, +14 ± 20: kept | 344 |
| T-N-Nc-merge-c | qualified | 396 | 208 x 1000 | 511 | +1359 | 62 | +1338 | +1169 | up +21 | 1170 (99660) | +437 -396 =167, 52.0%, +14 ± 20: kept | 397 |
| T-N-NNs-c | qualified | 291 | 106 x 1000 | 482 | +1222 | 62 | +1295 | +1032 | strike 1 (-73) | 1170 (100394) | +418 -417 =165, 50.0%, +0 ± 20: kept | 292 |
| T-N-NNs-merge-c | qualified | 333 | 152 x 1000 | 359 | +1288 | 62 | +1357 | +1098 | strike 1 (-69) | 1170 (101924) | +424 -426 =150, 49.9%, -1 ± 20: reverted | 333 |
| T-L-H-c | qualified | 478 | 277 x 1000 | 508 | +1537 | 64 | +1508 | +1348 | up +29 | 1170 (98423) | +487 -399 =114, 54.4%, +31 ± 20: kept | 479 |
| T-N-H-c | qualified | 470 | 262 x 1000 | 471 | +1208 | 62 | +1169 | +1018 | up +39 | 1170 (102203) | +431 -411 =158, 51.0%, +7 ± 20: kept | 471 |
