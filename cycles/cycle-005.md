# Cycle 5

**Changes from this cycle** (deployed after cycle 4, `staged-c5/`): the cycles run `chessai-cyc5`, whose merge-switch fix changes the recipe of T-L-LLs-merge-c, T-L-NLs-merge-c, T-L-NNs-merge-c and T-N-NNs-merge-c: a delegator that switches to error routing starts its revision windows afresh, so the switched branch gets a full window of error-routed examples before any merge check (before, the check came on the next example and judged the cluster-routed window). Every other lineage trains byte for byte as on `chessai-cyc2`. Also new: 5 round-robins (10 in cycles 3 and 4), `random-3` and `random-4` as reference players (vs refs still subtracts the mean of `random`, `random-1` and `random-2`), and re-qualification against the peak.

**Own-games step, pooled over the lineages: 30 gates, +14126 -11479 =4395, score 54.4%, Elo +30.7 ± 3.6; kept 25 of 30.**

All cycles so far: 150 gates, +72954 -54977 =22069, score 56.0%, Elo +41.8 ± 1.6; kept 129 of 150.

2026-11-06 20:08: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then ran the rounds that fill the budget; round sizes d0 1000, d1 860, d2 40, d3 10.
League: retired 60 players to retired.txt (their games stay in the fit), added 60.
Tournament: 5 double round-robins of the 65 active players (20800 games) in 2.2 min, 20800 games saved.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: a lineage whose new version rates below its predecessor gets a strike; 2 in a row make it dormant. A dormant lineage re-qualifies when its current version's vs refs beats its peak, the best vs refs of its earlier versions at ply 2 in this fit (retired ones included), by more than 1 x its ±; dormant rows show the gap.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | qualified | 1185 | 204 x 1000 | 348 | +1697 | 76 | +1647 | +1523 | up +50 | 1170 (98366) | +476 -390 =134, 54.3%, +30 ± 20: kept | 1186 |
| F-L-d0soft-c | qualified | 1457 | 253 x 1000 | 342 | +1559 | 73 | +1586 | +1385 | strike 1 (-27) | 1170 (98876) | +506 -386 =108, 56.0%, +42 ± 20: kept | 1458 |
| F-L-d1jit-c | dormant | 1139 | 147 x 860 | 258 | +1577 | 73 | +1596 | +1403 | dormant (-20, strike 2; -38 vs peak v851) | 1170 (103836) | +505 -335 =160, 58.5%, +60 ± 20: kept | 1140 |
| F-L-d1play-c | qualified | 1136 | 148 x 860 | 254 | +1558 | 73 | +1550 | +1384 | up +8 | 1170 (111224) | +522 -333 =145, 59.5%, +66 ± 20: kept | 1137 |
| F-L-d2play-c | qualified | 913 | 95 x 40 | 243 | +1727 | 76 | +1773 | +1553 | strike 1 (-46) | 1170 (99733) | +439 -399 =162, 52.0%, +14 ± 20: kept | 914 |
| F-L-d2mix-c | dormant | 750 | 82 x 40 | 242 | +1735 | 77 | +1778 | +1561 | dormant (-43, strike 2; -44 vs peak v591) | 1170 (100199) | +342 -467 =191, 43.8%, -44 ± 20: reverted | 750 |
| F-L-d2jit-c | qualified | 961 | 113 x 40 | 229 | +1697 | 76 | +1733 | +1523 | strike 1 (-36) | 1170 (105677) | +459 -406 =135, 52.6%, +18 ± 20: kept | 962 |
| F-N-c | dormant | 1056 | 126 x 1000 | 235 | +1422 | 72 | +1428 | +1248 | dormant (-6, strike 2; -12 vs peak v804) | 1170 (97088) | +450 -397 =153, 52.6%, +18 ± 20: kept | 1057 |
| MoE-c | dormant | 232 | 25 x 1000 | 230 | +1249 | 71 | +1300 | +1075 | dormant (-52, strike 2; -227 vs peak v178) | 1170 (109710) | +798 -124 =78, 83.7%, +284 ± 27: kept | 233 |
| FM-c | qualified | 181 | 22 x 1000 | 265 | +1297 | 71 | +1266 | +1123 | up +32 | 1170 (110911) | +545 -353 =102, 59.6%, +68 ± 21: kept | 182 |
| T-L-Ne-c | qualified | 451 | 37 x 1000 | 257 | +1562 | 73 | +1604 | +1389 | strike 1 (-41) | 1170 (105745) | +488 -379 =133, 55.5%, +38 ± 20: kept | 452 |
| T-L-Ne-merge-c | qualified | 700 | 137 x 1000 | 281 | +1680 | 75 | +1689 | +1506 | strike 1 (-9) | 1170 (96920) | +469 -411 =120, 52.9%, +20 ± 20: kept | 701 |
| T-L-Nc-c | qualified | 477 | 40 x 1000 | 257 | +1606 | 73 | +1608 | +1433 | strike 1 (-2) | 1170 (99766) | +403 -424 =173, 48.9%, -7 ± 20: reverted | 477 |
| T-L-Nc-merge-c | dormant | 832 | 139 x 1000 | 324 | +1637 | 74 | +1684 | +1463 | dormant (-47, strike 2; -48 vs peak v576) | 1170 (97694) | +508 -378 =114, 56.5%, +45 ± 20: kept | 833 |
| T-L-LLs-c | qualified | 442 | 33 x 1000 | 265 | +1509 | 72 | +1575 | +1335 | strike 1 (-67) | 1170 (106023) | +521 -338 =141, 59.2%, +64 ± 20: kept | 443 |
| T-L-LLs-merge-c | qualified | 934 | 140 x 1000 | 315 | +1566 | 73 | +1643 | +1392 | strike 1 (-77) | 1170 (107543) | +435 -431 =134, 50.2%, +1 ± 20: kept | 935 |
| T-L-NLs-c | qualified | 489 | 43 x 1000 | 259 | +1533 | 72 | +1518 | +1359 | up +15 | 1170 (101495) | +391 -428 =181, 48.1%, -13 ± 20: reverted | 489 |
| T-L-NLs-merge-c | qualified | 781 | 139 x 1000 | 310 | +1497 | 72 | +1694 | +1323 | strike 1 (-197) | 1170 (103167) | +581 -323 =96, 62.9%, +92 ± 21: kept | 782 |
| T-L-NNs-c | qualified | 486 | 41 x 1000 | 259 | +1594 | 73 | +1616 | +1420 | strike 1 (-22) | 1170 (102671) | +427 -398 =175, 51.4%, +10 ± 20: kept | 487 |
| T-L-NNs-merge-c | dormant | 815 | 131 x 1000 | 268 | +1644 | 74 | +1684 | +1470 | dormant (-41, strike 2; -53 vs peak v541) | 1170 (95960) | +497 -361 =142, 56.8%, +48 ± 20: kept | 816 |
| T-L-Le-c | qualified | 446 | 34 x 1000 | 249 | +1590 | 73 | +1563 | +1416 | up +28 | 1170 (97326) | +497 -388 =115, 55.5%, +38 ± 20: kept | 447 |
| T-L-Le-merge-c | qualified | 884 | 147 x 1000 | 274 | +1638 | 74 | +1680 | +1464 | strike 1 (-42) | 1170 (100611) | +463 -410 =127, 52.6%, +18 ± 20: kept | 885 |
| T-N-Ne-c | qualified | 439 | 35 x 1000 | 245 | +1349 | 71 | +1352 | +1176 | strike 1 (-3) | 1170 (98180) | +443 -436 =121, 50.3%, +2 ± 20: kept | 440 |
| T-N-Ne-merge-c | dormant | 695 | 71 x 1000 | 272 | +1379 | 71 | +1430 | +1205 | dormant (-51, strike 2; -77 vs peak v500) | 1170 (96812) | +411 -388 =201, 51.1%, +8 ± 19: kept | 696 |
| T-N-Nc-c | dormant | 471 | 40 x 1000 | 281 | +1361 | 71 | +1402 | +1187 | dormant (-42, strike 2; -50 vs peak v390) | 1170 (98567) | +420 -409 =171, 50.5%, +4 ± 20: kept | 472 |
| T-N-Nc-merge-c | dormant | 628 | 68 x 1000 | 197 | +1445 | 72 | +1459 | +1271 | dormant (-14, strike 2; -19 vs peak v487) | 1170 (95371) | +397 -423 =180, 48.7%, -9 ± 20: reverted | 628 |
| T-N-NNs-c | qualified | 414 | 35 x 1000 | 240 | +1244 | 71 | +1233 | +1070 | up +11 | 1170 (99160) | +423 -403 =174, 51.0%, +7 ± 20: kept | 415 |
| T-N-NNs-merge-c | qualified | 591 | 124 x 1000 | 280 | +1458 | 72 | +1440 | +1284 | up +18 | 1170 (96751) | +395 -410 =195, 49.2%, -5 ± 19: reverted | 591 |
| T-L-H-c | qualified | 858 | 124 x 1000 | 252 | +1572 | 73 | +1606 | +1398 | strike 1 (-34) | 1170 (98508) | +482 -344 =174, 56.9%, +48 ± 20: kept | 859 |
| T-N-H-c | qualified | 856 | 127 x 1000 | 238 | +1326 | 71 | +1325 | +1152 | up +1 | 1170 (97266) | +433 -407 =160, 51.3%, +9 ± 20: kept | 857 |
