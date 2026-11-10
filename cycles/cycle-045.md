# Cycle 45

**Own-games step, pooled over the lineages: 23 gates, +9807 -8889 =4304, score 52.0%, Elo +13.9 ± 4.1; kept 14 of 23.**

All cycles so far: 1390 gates, +614606 -545036 =230358, score 52.5%, Elo +17.4 ± 0.5; kept 926 of 1390.

2026-11-07 14:11: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 40 players to retired.txt (their games stay in the fit), added 40.
Tournament: 5 double round-robins of the 60 active players (17700 games) in 5.4 min, 17700 games saved.
Freeze/retire rule, after 10 cycles dormant: FM-c retired (avg ply 1 889.3, ply 2 1619.5; no untaken milestone within 10). 2 entries leave the field for retired.txt (their games stay in the fit): 60 players become 58, 17,700 games a cycle become 16,530.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| T-N-Nc-fresh-c | 9 of 10 | cycle 46 | 37–45 | 727.5 | 1303.6 | retire |
| F-L-d1play-c | 8 of 10 | cycle 47 | 38–45 | 992.9 | 1539.0 | retire |
| T-L-Le-fresh-c | 8 of 10 | cycle 47 | 38–45 | 908.8 | 1648.0 | freeze 1 at 900 |
| T-L-Ne-c | 6 of 10 | cycle 49 | 40–45 | 1021.2 | 1561.0 | retire |
| T-N-Nc-merge-c | 6 of 10 | cycle 49 | 40–45 | 760.2 | 1387.8 | retire |
| F-P-c | 5 of 10 | cycle 50 | 41–45 | 935.3 | 1497.8 | freeze 2 at 1500 |
| F-N-d2jit-c | 3 of 10 | cycle 52 | 43–45 | 654.2 | 1215.1 | retire |
| T-N-Nc-c | 3 of 10 | cycle 52 | 43–45 | 753.5 | 1348.3 | retire |
| T-L-Le-pre-c | 2 of 10 | cycle 53 | 44–45 | 907.1 | 1499.8 | retire |
| T-L-Ne-pre-c | 2 of 10 | cycle 53 | 44–45 | 950.7 | 1584.7 | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d1play-c | dormant | 3593 |  |  | 1538 | 45 | 1544 | 1630 (1630) | v3322 (v3322) | 1398 | dormant (-92 ± 33 vs peak v3322 before dormancy at v3585, at 1490 vs refs; needs > +33) | 1070 (92693) | +455 -410 =135, 52.2%, +16 ± 20: kept | 3594 |
| F-L-d2jit-c | frozen | 3943 |  |  | 815 | 16 |  | 1805 | v3348 | 675 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1372 | 11 |  | 1473 | v1061 | 1232 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1688 | 11 |  | 1717 | v300 | 1548 | frozen |  |  | 373 |
| FM-c | retired | 669 |  |  | 1613 | 19 |  | 1657 (1657) | v626 (v626) | 1473 | retired (avg ply 1 889.3, ply 2 1619.5; no untaken milestone within 10); dormant (-44 ± 27 vs peak v626 before dormancy at v666, at 1517 vs refs; needs > +27) |  |  | 669 |
| F-P-c | dormant | 2805 |  |  | 1521 | 45 | 1501 | 1584 (1584) | v2138 (v2138) | 1381 | dormant (-63 ± 33 vs peak v2138 before dormancy at v2801, at 1444 vs refs; needs > +33) | 1070 (100671) | +382 -433 =185, 47.4%, -18 ± 20: reverted | 2805 |
| F-P-tau0-c | qualified | 1409 | 185 x 1000 | 240 | 1517 | 45 | 1555 | 1566 | v862 | 1377 | strike 1 (-50 ± 48 vs peak v862; predecessor -39) | 1070 (84306) | +380 -409 =211, 48.5%, -10 ± 19: reverted | 1409 |
| F-P-tau0-d2jit-c | qualified | 993 | 145 x 40 | 240 | 1682 | 49 | 1717 | 1717 | v847 | 1542 | ok (-36 ± 54 vs peak v847; predecessor -36) | 1070 (94458) | +314 -337 =349, 48.9%, -8 ± 17: reverted | 993 |
| T-L-Ne-c | dormant | 1425 |  |  | 1562 | 18 |  | 1614 (1614) | v1288 (v1288) | 1422 | dormant (-53 ± 26 vs peak v1288 before dormancy at v1425, at 1474 vs refs; needs > +26) | 1070 (91902) | +442 -392 =166, 52.5%, +17 ± 20: kept | 1426 |
| T-L-Ne-fresh-c | qualified | 1000 | 38 x 1000 | 242 | 1535 | 45 | 1576 | 1604 | v633 | 1395 | strike 1 (-69 ± 50 vs peak v633; predecessor -41) | 1070 (89450) | +490 -355 =155, 56.8%, +47 ± 20: kept | 1001 |
| T-L-Ne-pre-c | dormant | 753 |  |  | 1581 | 33 |  | 1578 (1578) | v691 (v691) | 1441 | dormant (+4 ± 28 vs peak v691 before dormancy at v752, at 1437 vs refs; needs > +28) | 1070 (93432) | +420 -399 =181, 51.0%, +7 ± 20: kept | 754 |
| T-L-Nc-fresh-c | qualified | 1247 | 42 x 1000 | 241 | 1563 | 45 | 1539 | 1594 | v1015 | 1423 | ok (-31 ± 49 vs peak v1015; predecessor +24) | 1070 (92308) | +391 -437 =172, 47.7%, -16 ± 20: reverted | 1247 |
| D-L-Nc-c | qualified | 851 | 46 x 1000 | 242 | 1575 | 46 | 1515 | 1561 | v763 | 1435 | new peak (+15 ± 49 over v763; predecessor +60) | 1070 (92513) | +507 -363 =130, 57.2%, +50 ± 20: kept | 852 |
| T-L-LLs-c | frozen | 651 |  |  | 1594 | 10 |  | 1629 | v540 | 1454 | frozen |  |  | 651 |
| T-L-NNs-c | qualified | 2153 | 43 x 1000 | 242 | 1606 | 46 | 1609 | 1618 | v1590 | 1466 | ok (-12 ± 49 vs peak v1590; predecessor -3) | 1070 (93639) | +397 -423 =180, 48.7%, -9 ± 20: reverted | 2153 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 961 | 9 |  | 1702 | v818 | 821 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | dormant | 741 |  |  | 1634 | 47 | 1638 | 1648 (1648) | v682 (v682) | 1494 | dormant (-14 ± 35 vs peak v682 before dormancy at v737, at 1508 vs refs; needs > +35) | 1070 (89347) | +443 -410 =147, 51.6%, +12 ± 20: kept | 742 |
| T-L-Le-pre-c | dormant | 737 |  |  | 1487 | 44 | 1511 | 1526 (1526) | v670 (v670) | 1347 | dormant (-40 ± 32 vs peak v670 before dormancy at v735, at 1386 vs refs; needs > +32) | 1070 (95109) | +395 -404 =201, 49.5%, -3 ± 19: reverted | 737 |
| T-N-Ne-c | frozen | 637 |  |  | 1267 | 9 |  | 1368 | v574 | 1127 | frozen |  |  | 637 |
| T-N-Nc-c | dormant | 1502 |  |  | 1352 | 44 | 1368 | 1388 (1388) | v472 (v472) | 1212 | dormant (-36 ± 33 vs peak v472 before dormancy at v1500, at 1248 vs refs; needs > +33) | 1070 (89354) | +425 -411 =164, 50.7%, +5 ± 20: kept | 1503 |
| T-N-Nc-merge-c | dormant | 2128 |  |  | 1382 | 20 |  | 1459 (1446) | v2127 (v631) | 1241 | dormant (-65 ± 18 vs peak v631 before dormancy at v2126, at 1306 vs refs; needs > +18) | 1070 (86674) | +392 -443 =165, 47.4%, -18 ± 20: reverted | 2128 |
| T-N-Nc-fresh-c | dormant | 810 |  |  | 1340 | 44 | 1297 | 1361 (1361) | v624 (v624) | 1200 | dormant (-21 ± 31 vs peak v624 before dormancy at v803, at 1221 vs refs; needs > +31) | 1070 (89016) | +438 -385 =177, 52.6%, +18 ± 20: kept | 811 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 714 | 11 |  | 1345 | v270 | 574 | frozen |  |  | 417 |
| F-N-d2jit-c | dormant | 2421 |  |  | 1216 | 44 | 1241 | 1292 (1292) | v2053 (v2053) | 1076 | dormant (-76 ± 31 vs peak v2053 before dormancy at v2418, at 1152 vs refs; needs > +31) | 1070 (96621) | +438 -351 =211, 54.4%, +30 ± 19: kept | 2422 |
| T-N-NNs-merge-d2jit-c | qualified | 2447 | 101 x 40 | 240 | 1175 | 44 | 1215 | 1215 | v2345 | 1035 | ok (-40 ± 47 vs peak v2345; predecessor -40) | 1070 (97668) | +428 -363 =209, 53.2%, +23 ± 19: kept | 2448 |
| T-L-Le-merge-d2jit-c | qualified | 1086 | 96 x 40 | 241 | 1633 | 47 | 1647 | 1652 | v798 | 1493 | ok (-20 ± 51 vs peak v798; predecessor -14) | 1070 (93141) | +456 -326 =218, 56.5%, +45 ± 19: kept | 1087 |
| T-L-NLs-d2jit-c | qualified | 666 | 59 x 40 | 241 | 1395 | 44 | 1337 | 1396 | v548 | 1255 | ok (-1 ± 47 vs peak v548; predecessor +58) | 1070 (101258) | +440 -328 =232, 55.6%, +39 ± 19: kept | 667 |
| T-N-Nc-merge-d2jit-c | qualified | 1166 | 103 x 40 | 241 | 1086 | 43 | 1145 | 1172 | v655 | 946 | strike 1 (-86 ± 46 vs peak v655; predecessor -59) | 1070 (94360) | +426 -386 =188, 52.0%, +14 ± 19: kept | 1167 |
| T-L-NLs-merge-d2jit-c | qualified | 573 | 78 x 40 | 241 | 1568 | 45 | 1614 | 1614 | v495 | 1428 | ok (-45 ± 49 vs peak v495; predecessor -45) | 1070 (95186) | +595 -289 =116, 65.3%, +110 ± 21: kept | 574 |
| F-L-d4mix-c | qualified | 281 | 7 x 10 | 224 | 1664 | 48 | 1726 | 1745 | v200 | 1524 | strike 1 (-81 ± 55 vs peak v200; predecessor -62) | 1070 (93156) | +410 -412 =178, 49.9%, -1 ± 20: reverted | 281 |
| MoE-d2jit-c | qualified | 1364 | 47 x 40 | 242 | 1723 | 52 | 1672 | 1716 | v882 | 1583 | new peak (+7 ± 56 over v882; predecessor +52) | 1070 (95435) | +343 -423 =234, 46.0%, -28 ± 19: reverted | 1364 |
