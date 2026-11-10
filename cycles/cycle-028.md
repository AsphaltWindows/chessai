# Cycle 28

**Own-games step, pooled over the lineages: 29 gates, +12280 -12143 =4577, score 50.2%, Elo +1.6 ± 3.7; kept 17 of 29.**

All cycles so far: 979 gates, +438230 -383154 =157616, score 52.8%, Elo +19.6 ± 0.6; kept 653 of 979.

2026-11-07 06:54: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 50 players to retired.txt (their games stay in the fit), added 50.
Tournament: 5 double round-robins of the 71 active players (24850 games) in 3.6 min, 24850 games saved.
Freeze/retire rule, after 10 cycles dormant: T-N-Ne-merge-c retired (avg ply 1 741.5, ply 2 1400.2; no untaken milestone within 10). 2 entries leave the field for retired.txt (their games stay in the fit): 71 players become 69, 24,850 games a cycle become 23,460.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| F-L-d2mix-c | 9 of 10 | cycle 29 | 20–28 | 848.5 | 1734.4 | retire |
| T-L-LLs-merge-c | 9 of 10 | cycle 29 | 20–28 | 920.9 | 1569.4 | retire |
| T-L-Nc-merge-c | 9 of 10 | cycle 29 | 20–28 | 959.4 | 1679.0 | retire |
| F-L-d2play-c | 7 of 10 | cycle 31 | 22–28 | 816.1 | 1704.3 | retire |
| F-L-d0soft-c | 6 of 10 | cycle 32 | 23–28 | 979.8 | 1640.2 | retire |
| T-N-Le-merge-c | 5 of 10 | cycle 33 | 24–28 | 726.1 | 1347.4 | retire |
| F-L-c | 3 of 10 | cycle 35 | 26–28 | 984.8 | 1669.1 | retire |
| T-N-NLs-merge-c | 3 of 10 | cycle 35 | 26–28 | 640.8 | 1143.1 | retire |
| F-L-d1play-c | 2 of 10 | cycle 36 | 27–28 | 975.7 | 1523.5 | retire |
| F-N-d2jit-c | 1 of 10 | cycle 37 | 28–28 | 604.9 | 1205.3 | freeze 1 at 600 |
| T-L-NLs-c | 1 of 10 | cycle 37 | 28–28 | 970.9 | 1542.0 | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | dormant | 3141 |  |  | 1702 | 46 | 1677 | 1717 (1717) | v2860 (v2860) | 1560 | dormant (-15 ± 28 vs peak v2860 before dormancy at v3138, at 1575 vs refs; needs > +28) | 1290 (105113) | +443 -412 =145, 51.5%, +11 ± 20: kept | 3142 |
| F-L-d0soft-c | dormant | 2365 |  |  | 1628 | 44 | 1664 | 1701 (1701) | v2014 (v2014) | 1486 | dormant (-73 ± 31 vs peak v2014 before dormancy at v2361, at 1559 vs refs; needs > +31) | 1290 (104578) | +421 -425 =154, 49.8%, -1 ± 20: reverted | 2365 |
| F-L-d1play-c | dormant | 3315 |  |  | 1539 | 42 | 1513 | 1600 (1600) | v3006 (v3006) | 1397 | dormant (-61 ± 29 vs peak v3006 before dormancy at v3313, at 1458 vs refs; needs > +29) | 1290 (115776) | +491 -374 =135, 55.9%, +41 ± 20: kept | 3316 |
| F-L-d2play-c | dormant | 1623 |  |  | 1701 | 26 |  | 1754 (1754) | v1011 (v1011) | 1559 | dormant (-53 ± 23 vs peak v1011 before dormancy at v1620, at 1612 vs refs; needs > +23) | 1290 (114852) | +350 -489 =161, 43.0%, -49 ± 20: reverted | 1623 |
| F-L-d2mix-c | dormant | 1075 |  |  | 1752 | 28 |  | 1746 (1743) | v1073 (v910) | 1609 | dormant (+8 ± 28 vs peak v910 before dormancy at v1068, at 1601 vs refs; needs > +28) | 1290 (110954) | +376 -452 =172, 46.2%, -26 ± 20: reverted | 1075 |
| F-L-d2jit-c | qualified | 3586 | 123 x 40 | 240 | 1792 | 51 | 1751 | 1822 | v3348 | 1650 | ok (-29 ± 53 vs peak v3348; predecessor +42) | 1290 (114852) | +339 -414 =247, 46.2%, -26 ± 19: reverted | 3586 |
| F-N-c | frozen | 1691 |  |  | 1395 | 24 |  | 1476 | v1061 | 1253 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1715 | 18 |  | 1721 | v300 | 1573 | frozen |  |  | 373 |
| FM-c | qualified | 600 | 21 x 1000 | 237 | 1430 | 41 | 1635 | 1635 | v579 | 1288 | strike 1 (-204 ± 43 vs peak v579; predecessor -204) | 1290 (115884) | +557 -279 =164, 63.9%, +99 ± 20: kept | 601 |
| F-P-c | qualified | 717 | 189 x 1000 | 240 | 1502 | 42 | 1509 | 1509 | v528 | 1360 | ok (-7 ± 42 vs peak v528; predecessor -7) | 1290 (118076) | +350 -488 =162, 43.1%, -48 ± 20: reverted | 717 |
| T-L-Ne-c | qualified | 1034 | 39 x 1000 | 241 | 1615 | 43 | 1598 | 1614 | v953 | 1473 | new peak (+1 ± 44 over v953; predecessor +17) | 1290 (113636) | +430 -425 =145, 50.2%, +2 ± 20: kept | 1035 |
| T-L-Ne-fresh-c | qualified | 388 | 48 x 1000 | 241 | 1565 | 42 | 1569 | 1569 | v339 | 1423 | ok (-4 ± 43 vs peak v339; predecessor -4) | 1290 (110544) | +487 -398 =115, 54.4%, +31 ± 20: kept | 389 |
| T-L-Ne-pre-c | qualified | 217 | 49 x 1000 | 238 | 1474 | 42 | 1419 | 1419 | v167 | 1332 | new peak (+56 ± 42 over v167; predecessor +56) | 1290 (116230) | +500 -384 =116, 55.8%, +40 ± 20: kept | 218 |
| T-L-Nc-merge-c | dormant | 1885 |  |  | 1709 | 46 | 1732 | 1732 (1728) | v1884 (v1631) | 1567 | dormant (-19 ± 33 vs peak v1631 before dormancy at v1878, at 1586 vs refs; needs > +33) | 1290 (105122) | +396 -451 =153, 47.2%, -19 ± 20: reverted | 1885 |
| T-L-Nc-fresh-c | qualified | 564 | 44 x 1000 | 237 | 1575 | 42 | 1550 | 1575 | v473 | 1433 | new peak (+0 ± 43 over v473; predecessor +26) | 1290 (109086) | +418 -391 =191, 51.3%, +9 ± 19: kept | 565 |
| D-L-Nc-c | qualified | 139 | 43 x 1000 | 239 | 1405 | 41 | 1242 | 1242 | v95 | 1263 | new peak (+163 ± 42 over v95; predecessor +163) | 1290 (113654) | +516 -387 =97, 56.5%, +45 ± 21: kept | 140 |
| T-L-LLs-c | frozen | 651 |  |  | 1611 | 14 |  | 1633 | v540 | 1469 | frozen |  |  | 651 |
| T-L-LLs-merge-c | dormant | 1761 |  |  | 1632 | 44 | 1628 | 1628 (1608) | v1760 (v1251) | 1490 | dormant (+24 ± 31 vs peak v1251 before dormancy at v1754, at 1466 vs refs; needs > +31) | 1290 (109356) | +369 -451 =180, 45.9%, -29 ± 20: reverted | 1761 |
| T-L-NLs-c | dormant | 1433 |  |  | 1542 | 30 |  | 1591 (1591) | v1345 (v1345) | 1400 | dormant (-49 ± 25 vs peak v1345 before dormancy at v1433, at 1449 vs refs; needs > +25) | 1290 (108607) | +310 -551 =139, 38.0%, -85 ± 20: reverted | 1433 |
| T-L-NNs-c | qualified | 1457 | 44 x 1000 | 242 | 1593 | 43 | 1578 | 1606 | v561 | 1451 | ok (-13 ± 49 vs peak v561; predecessor +14) | 1290 (111301) | +434 -393 =173, 52.0%, +14 ± 20: kept | 1458 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 971 | 14 |  | 1705 | v818 | 829 | frozen |  |  | 1070 |
| T-L-Le-c | qualified | 1081 |  |  | 1686 | 45 | 1524 | 1622 (1622) | v990 (v990) | 1544 | re-qualified (+64 ± 30 vs peak v990 before dormancy at v1080, at 1480 vs refs) | 1290 (111183) | +372 -428 =200, 47.2%, -20 ± 19: reverted | 1081 |
| T-L-Le-fresh-c | qualified | 408 | 101 x 1000 | 241 | 1536 | 42 | 1512 | 1516 | v260 | 1394 | new peak (+19 ± 42 over v260; predecessor +24) | 1290 (108033) | +556 -357 =87, 60.0%, +70 ± 21: kept | 409 |
| T-L-Le-pre-c | qualified | 209 | 47 x 1000 | 242 | 1486 | 42 | 1425 | 1425 | v161 | 1343 | new peak (+61 ± 42 over v161; predecessor +61) | 1290 (109828) | +366 -491 =143, 43.8%, -44 ± 20: reverted | 209 |
| T-N-Ne-c | frozen | 637 |  |  | 1276 | 13 |  | 1372 | v574 | 1134 | frozen |  |  | 637 |
| T-N-Ne-merge-c | retired | 908 |  |  | 1403 | 29 |  | 1448 (1448) | v699 (v699) | 1261 | retired (avg ply 1 741.5, ply 2 1400.2; no untaken milestone within 10); dormant (-45 ± 21 vs peak v699 before dormancy at v904, at 1306 vs refs; needs > +21) |  |  | 908 |
| T-N-Nc-c | qualified | 976 | 41 x 1000 | 238 | 1376 | 41 | 1390 | 1391 | v472 | 1234 | ok (-15 ± 48 vs peak v472; predecessor -14) | 1290 (105702) | +426 -413 =161, 50.6%, +4 ± 20: kept | 977 |
| T-N-Nc-merge-c | qualified | 1532 | 56 x 1000 | 242 | 1428 | 41 | 1441 | 1450 | v631 | 1286 | ok (-22 ± 39 vs peak v631; predecessor -12) | 1290 (105098) | +440 -400 =160, 52.0%, +14 ± 20: kept | 1533 |
| T-N-Nc-fresh-c | qualified | 408 | 49 x 1000 | 240 | 1352 | 41 | 1327 | 1369 | v297 | 1210 | ok (-17 ± 42 vs peak v297; predecessor +25) | 1290 (106200) | +424 -417 =159, 50.3%, +2 ± 20: kept | 409 |
| T-N-Le-merge-c | dormant | 754 |  |  | 1352 | 17 |  | 1430 (1430) | v700 (v700) | 1210 | dormant (-78 ± 20 vs peak v700 before dormancy at v754, at 1288 vs refs; needs > +20) | 1290 (106331) | +422 -451 =127, 48.5%, -10 ± 20: reverted | 754 |
| T-N-NLs-merge-c | dormant | 1032 |  |  | 1138 | 41 | 1136 | 1419 (1419) | v787 (v787) | 996 | dormant (-281 ± 24 vs peak v787 before dormancy at v1030, at 1277 vs refs; needs > +24) | 1290 (113138) | +393 -459 =148, 46.7%, -23 ± 20: reverted | 1032 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 721 | 19 |  | 1349 | v270 | 579 | frozen |  |  | 417 |
| F-N-d2jit-c | dormant | 1784 |  |  | 1205 | 41 | 1147 | 1220 (1220) | v1523 (v1523) | 1063 | dormant (-14 ± 28 vs peak v1523 before dormancy at v1783, at 1077 vs refs; needs > +28) | 1290 (113035) | +404 -394 =202, 50.5%, +4 ± 19: kept | 1785 |
| T-N-NNs-merge-d2jit-c | qualified | 1652 | 98 x 40 | 241 | 1144 | 41 | 1177 | 1177 | v1553 | 1002 | ok (-33 ± 41 vs peak v1553; predecessor -33) | 1290 (111464) | +405 -398 =197, 50.3%, +2 ± 19: kept | 1653 |
| F-L-d4mix-c | qualified | 141 | 8 x 10 | 245 | 1724 | 47 | 1646 | 1654 | v124 | 1582 | new peak (+70 ± 46 over v124; predecessor +78) | 1290 (115947) | +482 -369 =149, 55.6%, +39 ± 20: kept | 142 |
| MoE-d2jit-c | qualified | 637 | 44 x 40 | 240 | 1669 | 45 | 1699 | 1699 | v593 | 1527 | ok (-29 ± 46 vs peak v593; predecessor -29) | 1290 (113497) | +403 -402 =195, 50.0%, +0 ± 19: kept | 638 |
