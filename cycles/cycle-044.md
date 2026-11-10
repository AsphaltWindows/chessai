# Cycle 44

**Own-games step, pooled over the lineages: 24 gates, +10050 -9580 =4370, score 51.0%, Elo +6.8 ± 4.0; kept 18 of 24.**

All cycles so far: 1367 gates, +604799 -536147 =226054, score 52.5%, Elo +17.5 ± 0.5; kept 912 of 1367.

2026-11-07 13:49: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 42 players to retired.txt (their games stay in the fit), added 42.
Tournament: 5 double round-robins of the 60 active players (17700 games) in 5.4 min, 17700 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| FM-c | 9 of 10 | cycle 45 | 36–44 | 890.3 | 1620.2 | freeze 1 at 900 |
| T-N-Nc-fresh-c | 8 of 10 | cycle 46 | 37–44 | 725.4 | 1299.0 | retire |
| F-L-d1play-c | 7 of 10 | cycle 47 | 38–44 | 990.6 | 1539.1 | retire |
| T-L-Le-fresh-c | 7 of 10 | cycle 47 | 38–44 | 903.8 | 1649.9 | retire |
| T-L-Ne-c | 5 of 10 | cycle 49 | 40–44 | 1021.0 | 1560.9 | retire |
| T-N-Nc-merge-c | 5 of 10 | cycle 49 | 40–44 | 760.7 | 1389.0 | retire |
| F-P-c | 4 of 10 | cycle 50 | 41–44 | 934.7 | 1492.1 | freeze 2 at 1500 |
| F-N-d2jit-c | 2 of 10 | cycle 52 | 43–44 | 662.1 | 1214.5 | retire |
| T-N-Nc-c | 2 of 10 | cycle 52 | 43–44 | 758.7 | 1346.7 | retire |
| T-L-Le-pre-c | 1 of 10 | cycle 53 | 44–44 | 921.8 | 1512.8 | retire |
| T-L-Ne-pre-c | 1 of 10 | cycle 53 | 44–44 | 953.9 | 1588.3 | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d1play-c | dormant | 3592 |  |  | 1546 | 45 | 1570 | 1631 (1631) | v3322 (v3322) | 1406 | dormant (-85 ± 34 vs peak v3322 before dormancy at v3585, at 1491 vs refs; needs > +34) | 1070 (92737) | +447 -419 =134, 51.4%, +10 ± 20: kept | 3593 |
| F-L-d2jit-c | frozen | 3943 |  |  | 818 | 16 |  | 1806 | v3348 | 678 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1373 | 11 |  | 1473 | v1061 | 1233 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1689 | 11 |  | 1717 | v300 | 1549 | frozen |  |  | 373 |
| FM-c | dormant | 669 |  |  | 1616 | 20 |  | 1658 (1658) | v626 (v626) | 1476 | dormant (-42 ± 27 vs peak v626 before dormancy at v666, at 1518 vs refs; needs > +27) | 1070 (93781) | +324 -509 =167, 40.8%, -65 ± 20: reverted | 669 |
| F-P-c | dormant | 2804 |  |  | 1502 | 45 | 1496 | 1585 (1585) | v2138 (v2138) | 1362 | dormant (-83 ± 33 vs peak v2138 before dormancy at v2801, at 1445 vs refs; needs > +33) | 1070 (101521) | +411 -386 =203, 51.2%, +9 ± 19: kept | 2805 |
| F-P-tau0-c | qualified | 1223 | 183 x 1000 | 240 | 1557 | 46 | 1541 | 1567 | v862 | 1417 | ok (-10 ± 49 vs peak v862; predecessor +16) | 1070 (89256) | +424 -387 =189, 51.8%, +13 ± 19: kept | 1224 |
| F-P-tau0-d2jit-c | qualified | 847 | 146 x 40 | 241 | 1719 | 51 | 1698 | 1698 | v701 | 1579 | new peak (+21 ± 54 over v701; predecessor +21) | 1070 (96326) | +360 -318 =322, 52.1%, +15 ± 18: kept | 848 |
| T-L-Ne-c | dormant | 1425 |  |  | 1561 | 20 |  | 1616 (1616) | v1288 (v1288) | 1421 | dormant (-55 ± 26 vs peak v1288 before dormancy at v1425, at 1475 vs refs; needs > +26) | 1070 (93502) | +369 -450 =181, 46.0%, -28 ± 20: reverted | 1425 |
| T-L-Ne-fresh-c | qualified | 961 | 34 x 1000 | 244 | 1577 | 46 | 1588 | 1605 | v633 | 1437 | ok (-28 ± 50 vs peak v633; predecessor -10) | 1070 (89777) | +457 -394 =149, 53.1%, +22 ± 20: kept | 962 |
| T-L-Ne-pre-c | dormant | 753 |  |  | 1588 | 46 | 1524 | 1578 (1578) | v691 (v691) | 1448 | dormant (+10 ± 33 vs peak v691 before dormancy at v752, at 1438 vs refs; needs > +33) | 1070 (92502) | +409 -421 =170, 49.4%, -4 ± 20: reverted | 753 |
| T-L-Nc-fresh-c | qualified | 1205 | 37 x 1000 | 244 | 1541 | 45 | 1561 | 1595 | v1015 | 1401 | strike 1 (-54 ± 49 vs peak v1015; predecessor -21) | 1070 (91434) | +402 -421 =177, 49.0%, -7 ± 20: reverted | 1205 |
| D-L-Nc-c | qualified | 804 | 40 x 1000 | 243 | 1517 | 45 | 1561 | 1561 | v763 | 1377 | ok (-45 ± 48 vs peak v763; predecessor -45) | 1070 (93441) | +474 -379 =147, 54.8%, +33 ± 20: kept | 805 |
| T-L-LLs-c | frozen | 651 |  |  | 1597 | 10 |  | 1629 | v540 | 1456 | frozen |  |  | 651 |
| T-L-NNs-c | qualified | 2109 | 38 x 1000 | 244 | 1611 | 47 | 1598 | 1619 | v1590 | 1471 | ok (-8 ± 49 vs peak v1590; predecessor +13) | 1070 (92572) | +435 -390 =175, 52.2%, +16 ± 20: kept | 2110 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 961 | 9 |  | 1702 | v818 | 821 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | dormant | 740 |  |  | 1640 | 48 | 1639 | 1650 (1650) | v682 (v682) | 1500 | dormant (-10 ± 35 vs peak v682 before dormancy at v737, at 1509 vs refs; needs > +35) | 1070 (90084) | +435 -412 =153, 51.1%, +8 ± 20: kept | 741 |
| T-L-Le-pre-c | dormant | 736 |  |  | 1513 | 45 | 1436 | 1527 (1527) | v670 (v670) | 1373 | dormant (-14 ± 32 vs peak v670 before dormancy at v735, at 1387 vs refs; needs > +32) | 1070 (92774) | +461 -371 =168, 54.5%, +31 ± 20: kept | 737 |
| T-N-Ne-c | frozen | 637 |  |  | 1268 | 9 |  | 1369 | v574 | 1128 | frozen |  |  | 637 |
| T-N-Nc-c | dormant | 1501 |  |  | 1370 | 44 | 1323 | 1388 (1388) | v472 (v472) | 1229 | dormant (-19 ± 33 vs peak v472 before dormancy at v1500, at 1248 vs refs; needs > +33) | 1070 (88082) | +429 -411 =160, 50.9%, +6 ± 20: kept | 1502 |
| T-N-Nc-merge-c | dormant | 2128 |  |  | 1379 | 23 |  | 1460 (1446) | v2127 (v631) | 1239 | dormant (-67 ± 19 vs peak v631 before dormancy at v2126, at 1306 vs refs; needs > +19) | 1070 (88375) | +393 -454 =153, 46.9%, -21 ± 20: reverted | 2128 |
| T-N-Nc-fresh-c | dormant | 809 |  |  | 1298 | 44 | 1297 | 1362 (1362) | v624 (v624) | 1158 | dormant (-64 ± 31 vs peak v624 before dormancy at v803, at 1222 vs refs; needs > +31) | 1070 (92072) | +416 -416 =168, 50.0%, -0 ± 20: kept | 810 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 716 | 11 |  | 1345 | v270 | 576 | frozen |  |  | 417 |
| F-N-d2jit-c | dormant | 2420 |  |  | 1242 | 44 | 1186 | 1293 (1293) | v2053 (v2053) | 1102 | dormant (-51 ± 31 vs peak v2053 before dormancy at v2418, at 1153 vs refs; needs > +31) | 1070 (93919) | +422 -366 =212, 52.8%, +20 ± 19: kept | 2421 |
| T-N-NNs-merge-d2jit-c | qualified | 2345 | 72 x 40 | 241 | 1216 | 44 | 1170 | 1206 | v2116 | 1076 | new peak (+10 ± 46 over v2116; predecessor +46) | 1070 (93834) | +417 -375 =208, 52.1%, +15 ± 19: kept | 2346 |
| T-L-Le-merge-d2jit-c | qualified | 989 | 96 x 40 | 239 | 1649 | 48 | 1528 | 1653 | v798 | 1509 | ok (-4 ± 52 vs peak v798; predecessor +121) | 1070 (93640) | +417 -393 =190, 51.2%, +8 ± 19: kept | 990 |
| T-L-NLs-d2jit-c | qualified | 606 | 57 x 40 | 238 | 1338 | 44 | 1397 | 1397 | v548 | 1198 | strike 1 (-58 ± 47 vs peak v548; predecessor -58) | 1070 (101311) | +468 -347 =185, 56.0%, +42 ± 20: kept | 607 |
| T-N-Nc-merge-d2jit-c | qualified | 1062 | 107 x 40 | 249 | 1146 | 44 | 1155 | 1173 | v655 | 1006 | ok (-27 ± 46 vs peak v655; predecessor -9) | 1070 (92239) | +408 -371 =221, 51.8%, +13 ± 19: kept | 1063 |
| T-L-NLs-merge-d2jit-c | qualified | 495 | 103 x 40 | 241 | 1615 | 47 | 1464 | 1464 | v391 | 1475 | new peak (+152 ± 49 over v391; predecessor +152) | 1070 (96690) | +424 -444 =132, 49.0%, -7 ± 20: reverted | 495 |
| F-L-d4mix-c | qualified | 273 | 8 x 10 | 247 | 1728 | 52 | 1718 | 1746 | v200 | 1588 | ok (-18 ± 57 vs peak v200; predecessor +9) | 1070 (93168) | +420 -399 =181, 51.0%, +7 ± 20: kept | 274 |
| MoE-d2jit-c | qualified | 1316 | 44 x 40 | 241 | 1673 | 49 | 1699 | 1717 | v882 | 1533 | ok (-44 ± 55 vs peak v882; predecessor -26) | 1070 (91044) | +428 -347 =225, 54.0%, +28 ± 19: kept | 1317 |
