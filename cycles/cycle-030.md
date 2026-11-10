# Cycle 30

**Own-games step, pooled over the lineages: 26 gates, +10979 -10879 =4142, score 50.2%, Elo +1.3 ± 3.9; kept 14 of 26.**

All cycles so far: 1031 gates, +460725 -404311 =165964, score 52.7%, Elo +19.0 ± 0.6; kept 686 of 1031.

2026-11-07 07:43: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 48 players to retired.txt (their games stay in the fit), added 48.
Tournament: 5 double round-robins of the 63 active players (19530 games) in 3.5 min, 19530 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| F-L-d2play-c | 9 of 10 | cycle 31 | 22–30 | 815.8 | 1702.2 | retire |
| F-L-d0soft-c | 8 of 10 | cycle 32 | 23–30 | 978.8 | 1641.7 | retire |
| T-N-Le-merge-c | 7 of 10 | cycle 33 | 24–30 | 725.5 | 1347.4 | retire |
| F-L-c | 5 of 10 | cycle 35 | 26–30 | 980.7 | 1679.5 | retire |
| T-N-NLs-merge-c | 5 of 10 | cycle 35 | 26–30 | 637.3 | 1141.5 | retire |
| F-L-d1play-c | 4 of 10 | cycle 36 | 27–30 | 977.7 | 1539.2 | retire |
| T-L-NLs-c | 3 of 10 | cycle 37 | 28–30 | 978.8 | 1546.0 | retire |
| FM-c | 1 of 10 | cycle 39 | 30–30 | 878.4 | 1530.9 | retire |
| T-L-Le-c | 0 of 10 | cycle 40 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | dormant | 3143 |  |  | 1690 | 49 | 1700 | 1717 (1717) | v2860 (v2860) | 1548 | dormant (-26 ± 29 vs peak v2860 before dormancy at v3138, at 1574 vs refs; needs > +29) | 1130 (91639) | +424 -452 =124, 48.6%, -10 ± 20: reverted | 3143 |
| F-L-d0soft-c | dormant | 2366 |  |  | 1634 | 46 | 1657 | 1702 (1702) | v2014 (v2014) | 1492 | dormant (-67 ± 32 vs peak v2014 before dormancy at v2361, at 1559 vs refs; needs > +32) | 1130 (90368) | +430 -432 =138, 49.9%, -1 ± 20: reverted | 2366 |
| F-L-d1play-c | dormant | 3317 |  |  | 1562 | 44 | 1548 | 1600 (1600) | v3006 (v3006) | 1419 | dormant (-38 ± 30 vs peak v3006 before dormancy at v3313, at 1457 vs refs; needs > +30) | 1130 (98635) | +452 -418 =130, 51.7%, +12 ± 20: kept | 3318 |
| F-L-d2play-c | dormant | 1624 |  |  | 1696 | 49 | 1693 | 1755 (1755) | v1011 (v1011) | 1554 | dormant (-59 ± 31 vs peak v1011 before dormancy at v1620, at 1612 vs refs; needs > +31) | 1130 (99381) | +393 -423 =184, 48.5%, -10 ± 20: reverted | 1624 |
| F-L-d2jit-c | qualified | 3825 | 114 x 40 | 240 | 1744 | 52 | 1811 | 1819 | v3348 | 1602 | strike 1 (-75 ± 53 vs peak v3348; predecessor -67) | 1130 (100171) | +378 -363 =259, 50.7%, +5 ± 19: kept | 3826 |
| F-N-c | frozen | 1691 |  |  | 1388 | 20 |  | 1477 | v1061 | 1245 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1712 | 17 |  | 1721 | v300 | 1570 | frozen |  |  | 373 |
| FM-c | dormant | 624 |  |  | 1531 | 44 | 1391 | 1633 (1633) | v579 (v579) | 1388 | dormant (-102 ± 30 vs peak v579 before dormancy at v623, at 1491 vs refs; needs > +30) | 1130 (96318) | +538 -356 =106, 59.1%, +64 ± 21: kept | 625 |
| F-P-c | qualified | 1087 | 178 x 1000 | 240 | 1531 | 44 | 1502 | 1508 | v528 | 1388 | new peak (+23 ± 44 over v528; predecessor +29) | 1130 (102816) | +356 -450 =194, 45.3%, -33 ± 19: reverted | 1087 |
| T-L-Ne-c | qualified | 1113 | 39 x 1000 | 240 | 1554 | 44 | 1608 | 1615 | v1034 | 1412 | strike 1 (-61 ± 46 vs peak v1034; predecessor -54) | 1130 (96097) | +439 -405 =156, 51.7%, +12 ± 20: kept | 1114 |
| T-L-Ne-fresh-c | qualified | 480 | 42 x 1000 | 238 | 1542 | 44 | 1563 | 1568 | v339 | 1399 | ok (-26 ± 44 vs peak v339; predecessor -21) | 1130 (95303) | +472 -379 =149, 54.6%, +32 ± 20: kept | 481 |
| T-L-Ne-pre-c | qualified | 310 | 44 x 1000 | 241 | 1530 | 44 | 1506 | 1506 | v265 | 1387 | new peak (+24 ± 45 over v265; predecessor +24) | 1130 (98369) | +476 -373 =151, 55.1%, +36 ± 20: kept | 311 |
| T-L-Nc-fresh-c | qualified | 656 | 45 x 1000 | 238 | 1596 | 45 | 1598 | 1598 | v610 | 1454 | ok (-2 ± 47 vs peak v610; predecessor -2) | 1130 (97584) | +409 -419 =172, 49.5%, -4 ± 20: reverted | 656 |
| D-L-Nc-c | qualified | 229 | 45 x 1000 | 240 | 1455 | 43 | 1344 | 1405 | v139 | 1312 | new peak (+50 ± 45 over v139; predecessor +111) | 1130 (98674) | +487 -390 =123, 54.9%, +34 ± 20: kept | 230 |
| T-L-LLs-c | frozen | 651 |  |  | 1612 | 13 |  | 1634 | v540 | 1469 | frozen |  |  | 651 |
| T-L-NLs-c | dormant | 1433 |  |  | 1546 | 22 |  | 1590 (1590) | v1345 (v1345) | 1403 | dormant (-45 ± 23 vs peak v1345 before dormancy at v1433, at 1448 vs refs; needs > +23) | 1130 (96894) | +322 -519 =159, 40.2%, -69 ± 20: reverted | 1433 |
| T-L-NNs-c | qualified | 1546 | 43 x 1000 | 238 | 1577 | 45 | 1593 | 1606 | v561 | 1434 | ok (-29 ± 50 vs peak v561; predecessor -16) | 1130 (98528) | +413 -397 =190, 50.8%, +6 ± 19: kept | 1547 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 971 | 13 |  | 1706 | v818 | 828 | frozen |  |  | 1070 |
| T-L-Le-c | dormant | 1171 | 45 x 1000 | 240 | 1509 | 44 | 1470 | 1686 | v1081 | 1366 | dormant (strike 2; -177 ± 47 vs peak v1081; predecessor +39) | 1130 (93751) | +457 -410 =133, 52.3%, +16 ± 20: kept | 1172 |
| T-L-Le-fresh-c | qualified | 545 | 57 x 1000 | 242 | 1583 | 45 | 1564 | 1564 | v487 | 1441 | new peak (+20 ± 46 over v487; predecessor +20) | 1130 (91814) | +513 -396 =91, 55.9%, +41 ± 21: kept | 546 |
| T-L-Le-pre-c | qualified | 299 | 45 x 1000 | 241 | 1506 | 44 | 1470 | 1486 | v209 | 1364 | new peak (+21 ± 45 over v209; predecessor +37) | 1130 (97877) | +338 -498 =164, 42.0%, -56 ± 20: reverted | 299 |
| T-N-Ne-c | frozen | 637 |  |  | 1277 | 12 |  | 1373 | v574 | 1135 | frozen |  |  | 637 |
| T-N-Nc-c | qualified | 1063 | 42 x 1000 | 238 | 1367 | 43 | 1383 | 1392 | v472 | 1225 | ok (-24 ± 49 vs peak v472; predecessor -15) | 1130 (93741) | +418 -437 =145, 49.0%, -7 ± 20: reverted | 1063 |
| T-N-Nc-merge-c | qualified | 1658 | 59 x 1000 | 240 | 1401 | 43 | 1455 | 1455 | v1598 | 1258 | strike 1 (-54 ± 45 vs peak v1598; predecessor -54) | 1130 (93718) | +415 -439 =146, 48.8%, -8 ± 20: reverted | 1658 |
| T-N-Nc-fresh-c | qualified | 517 | 60 x 1000 | 250 | 1370 | 43 | 1364 | 1368 | v297 | 1227 | new peak (+2 ± 43 over v297; predecessor +6) | 1130 (94520) | +422 -417 =161, 50.2%, +2 ± 20: kept | 518 |
| T-N-Le-merge-c | dormant | 754 |  |  | 1347 | 16 |  | 1430 (1430) | v700 (v700) | 1205 | dormant (-83 ± 20 vs peak v700 before dormancy at v754, at 1288 vs refs; needs > +20) | 1130 (93352) | +404 -485 =111, 46.0%, -28 ± 20: reverted | 754 |
| T-N-NLs-merge-c | dormant | 1033 |  |  | 1142 | 43 | 1136 | 1419 (1419) | v787 (v787) | 1000 | dormant (-277 ± 25 vs peak v787 before dormancy at v1030, at 1277 vs refs; needs > +25) | 1130 (98943) | +429 -419 =152, 50.5%, +4 ± 20: kept | 1034 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 722 | 17 |  | 1349 | v270 | 580 | frozen |  |  | 417 |
| F-N-d2jit-c | qualified | 1786 |  |  | 1267 | 43 | 1215 | 1219 (1219) | v1523 (v1523) | 1124 | re-qualified (+48 ± 29 vs peak v1523 before dormancy at v1783, at 1077 vs refs) | 1130 (99801) | +406 -393 =201, 50.6%, +4 ± 19: kept | 1787 |
| T-N-NNs-merge-d2jit-c | qualified | 1845 | 97 x 40 | 243 | 1160 | 43 | 1145 | 1177 | v1553 | 1018 | ok (-17 ± 43 vs peak v1553; predecessor +15) | 1130 (96396) | +393 -395 =212, 49.9%, -1 ± 19: reverted | 1845 |
| F-L-d4mix-c | qualified | 159 | 8 x 10 | 231 | 1689 | 48 | 1648 | 1724 | v141 | 1546 | ok (-35 ± 50 vs peak v141; predecessor +41) | 1130 (94623) | +386 -440 =174, 47.3%, -19 ± 20: reverted | 159 |
| MoE-d2jit-c | qualified | 739 | 50 x 40 | 241 | 1618 | 46 | 1697 | 1697 | v593 | 1476 | strike 1 (-79 ± 47 vs peak v593; predecessor -78) | 1130 (99031) | +409 -374 =217, 51.7%, +12 ± 19: kept | 740 |
