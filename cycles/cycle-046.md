# Cycle 46

**Own-games step, pooled over the lineages: 22 gates, +9288 -8614 =4098, score 51.5%, Elo +10.6 ± 4.1; kept 13 of 22.**

All cycles so far: 1412 gates, +623894 -553650 =234456, score 52.5%, Elo +17.3 ± 0.5; kept 939 of 1412.

2026-11-07 14:33: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 40 players to retired.txt (their games stay in the fit), added 40.
Tournament: 5 double round-robins of the 58 active players (16530 games) in 5.6 min, 16530 games saved.
Freeze/retire rule, after 10 cycles dormant: T-N-Nc-fresh-c retired (avg ply 1 729.1, ply 2 1308.4; no untaken milestone within 10). 2 entries leave the field for retired.txt (their games stay in the fit): 58 players become 56, 16,530 games a cycle become 15,400.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| F-L-d1play-c | 9 of 10 | cycle 47 | 38–46 | 990.4 | 1546.2 | retire |
| T-L-Le-fresh-c | 9 of 10 | cycle 47 | 38–46 | 908.8 | 1645.7 | freeze 1 at 900 |
| T-L-Ne-c | 7 of 10 | cycle 49 | 40–46 | 1018.2 | 1568.0 | retire |
| T-N-Nc-merge-c | 7 of 10 | cycle 49 | 40–46 | 760.1 | 1387.6 | retire |
| F-P-c | 6 of 10 | cycle 50 | 41–46 | 932.0 | 1499.7 | freeze 2 at 1500 |
| F-N-d2jit-c | 4 of 10 | cycle 52 | 43–46 | 652.8 | 1218.8 | retire |
| T-N-Nc-c | 4 of 10 | cycle 52 | 43–46 | 748.5 | 1346.2 | retire |
| T-L-Le-pre-c | 3 of 10 | cycle 53 | 44–46 | 900.9 | 1503.1 | retire |
| T-L-Ne-pre-c | 3 of 10 | cycle 53 | 44–46 | 939.9 | 1580.9 | retire |
| T-L-Ne-fresh-c | 0 of 10 | cycle 56 | - | - | - | retire |
| T-N-Nc-merge-d2jit-c | 0 of 10 | cycle 56 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d1play-c | dormant | 3594 |  |  | 1604 | 48 | 1538 | 1629 (1629) | v3322 (v3322) | 1463 | dormant (-26 ± 35 vs peak v3322 before dormancy at v3585, at 1488 vs refs; needs > +35) | 1030 (91342) | +465 -406 =129, 52.9%, +20 ± 20: kept | 3595 |
| F-L-d2jit-c | frozen | 3943 |  |  | 814 | 15 |  | 1804 | v3348 | 673 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1372 | 11 |  | 1474 | v1061 | 1231 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1687 | 11 |  | 1718 | v300 | 1547 | frozen |  |  | 373 |
| F-P-c | dormant | 2805 |  |  | 1509 | 32 |  | 1584 (1584) | v2138 (v2138) | 1368 | dormant (-74 ± 29 vs peak v2138 before dormancy at v2801, at 1443 vs refs; needs > +29) | 1030 (97546) | +405 -402 =193, 50.1%, +1 ± 19: kept | 2806 |
| F-P-tau0-c | qualified | 1589 | 180 x 1000 | 240 | 1531 | 46 | 1517 | 1566 | v862 | 1390 | ok (-35 ± 49 vs peak v862; predecessor +14) | 1030 (88802) | +378 -405 =217, 48.6%, -9 ± 19: reverted | 1589 |
| F-P-tau0-d2jit-c | qualified | 1135 | 142 x 40 | 241 | 1708 | 52 | 1682 | 1718 | v847 | 1567 | ok (-10 ± 55 vs peak v847; predecessor +26) | 1030 (91649) | +319 -327 =354, 49.6%, -3 ± 17: reverted | 1135 |
| T-L-Ne-c | dormant | 1426 |  |  | 1610 | 48 | 1562 | 1614 (1614) | v1288 (v1288) | 1469 | dormant (-4 ± 34 vs peak v1288 before dormancy at v1425, at 1473 vs refs; needs > +34) | 1030 (89411) | +489 -379 =132, 55.5%, +38 ± 20: kept | 1427 |
| T-L-Ne-fresh-c | dormant | 1038 | 37 x 1000 | 243 | 1545 | 47 | 1536 | 1604 | v633 | 1404 | dormant (strike 2; -59 ± 51 vs peak v633; predecessor +10) | 1030 (89061) | +505 -342 =153, 58.1%, +57 ± 20: kept | 1039 |
| T-L-Ne-pre-c | dormant | 754 |  |  | 1573 | 47 | 1581 | 1581 (1578) | v753 (v691) | 1433 | dormant (-4 ± 33 vs peak v691 before dormancy at v752, at 1437 vs refs; needs > +33) | 1030 (87915) | +429 -411 =160, 50.9%, +6 ± 20: kept | 755 |
| T-L-Nc-fresh-c | qualified | 1286 | 39 x 1000 | 240 | 1565 | 47 | 1564 | 1594 | v1015 | 1424 | ok (-29 ± 50 vs peak v1015; predecessor +1) | 1030 (90724) | +448 -361 =191, 54.4%, +30 ± 19: kept | 1287 |
| D-L-Nc-c | qualified | 894 | 42 x 1000 | 241 | 1534 | 46 | 1576 | 1576 | v851 | 1393 | ok (-42 ± 49 vs peak v851; predecessor -42) | 1030 (86124) | +530 -345 =125, 59.2%, +65 ± 20: kept | 895 |
| T-L-LLs-c | frozen | 651 |  |  | 1593 | 9 |  | 1630 | v540 | 1452 | frozen |  |  | 651 |
| T-L-NNs-c | qualified | 2193 | 40 x 1000 | 241 | 1585 | 47 | 1606 | 1618 | v1590 | 1444 | ok (-33 ± 50 vs peak v1590; predecessor -22) | 1030 (89373) | +389 -445 =166, 47.2%, -20 ± 20: reverted | 2193 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 961 | 9 |  | 1703 | v818 | 820 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | dormant | 742 |  |  | 1628 | 48 | 1635 | 1648 (1648) | v682 (v682) | 1487 | dormant (-20 ± 35 vs peak v682 before dormancy at v737, at 1507 vs refs; needs > +35) | 1030 (86804) | +430 -455 =115, 48.8%, -9 ± 20: reverted | 742 |
| T-L-Le-pre-c | dormant | 737 |  |  | 1510 | 32 |  | 1527 (1527) | v670 (v670) | 1369 | dormant (-17 ± 28 vs peak v670 before dormancy at v735, at 1386 vs refs; needs > +28) | 1030 (91188) | +396 -427 =177, 48.4%, -11 ± 20: reverted | 737 |
| T-N-Ne-c | frozen | 637 |  |  | 1266 | 9 |  | 1370 | v574 | 1125 | frozen |  |  | 637 |
| T-N-Nc-c | dormant | 1503 |  |  | 1340 | 46 | 1352 | 1389 (1389) | v472 (v472) | 1199 | dormant (-50 ± 34 vs peak v472 before dormancy at v1500, at 1248 vs refs; needs > +34) | 1030 (84738) | +418 -419 =163, 50.0%, -0 ± 20: reverted | 1503 |
| T-N-Nc-merge-c | dormant | 2128 |  |  | 1386 | 19 |  | 1458 (1447) | v2127 (v631) | 1246 | dormant (-61 ± 18 vs peak v631 before dormancy at v2126, at 1306 vs refs; needs > +18) | 1030 (86374) | +419 -411 =170, 50.4%, +3 ± 20: kept | 2129 |
| T-N-Nc-fresh-c | retired | 811 |  |  | 1352 | 46 | 1340 | 1361 (1361) | v624 (v624) | 1211 | retired (avg ply 1 729.1, ply 2 1308.4; no untaken milestone within 10); dormant (-9 ± 32 vs peak v624 before dormancy at v803, at 1220 vs refs; needs > +32) |  |  | 811 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 714 | 10 |  | 1346 | v270 | 573 | frozen |  |  | 417 |
| F-N-d2jit-c | dormant | 2422 |  |  | 1230 | 46 | 1216 | 1292 (1292) | v2053 (v2053) | 1089 | dormant (-62 ± 32 vs peak v2053 before dormancy at v2418, at 1151 vs refs; needs > +32) | 1030 (90500) | +376 -421 =203, 47.8%, -16 ± 19: reverted | 2422 |
| T-N-NNs-merge-d2jit-c | qualified | 2548 | 100 x 40 | 240 | 1179 | 45 | 1175 | 1215 | v2345 | 1038 | ok (-36 ± 48 vs peak v2345; predecessor +5) | 1030 (92930) | +414 -380 =206, 51.7%, +12 ± 19: kept | 2549 |
| T-L-Le-merge-d2jit-c | qualified | 1182 | 95 x 40 | 244 | 1717 | 52 | 1634 | 1653 | v798 | 1576 | new peak (+64 ± 54 over v798; predecessor +83) | 1030 (93631) | +467 -336 =197, 56.5%, +46 ± 19: kept | 1183 |
| T-L-NLs-d2jit-c | qualified | 725 | 58 x 40 | 242 | 1487 | 46 | 1396 | 1397 | v548 | 1346 | new peak (+90 ± 48 over v548; predecessor +91) | 1030 (97011) | +431 -330 =239, 55.0%, +35 ± 19: kept | 726 |
| T-N-Nc-merge-d2jit-c | dormant | 1261 | 94 x 40 | 240 | 1124 | 45 | 1085 | 1172 | v655 | 983 | dormant (strike 2; -48 ± 47 vs peak v655; predecessor +40) | 1030 (91936) | +419 -345 =236, 53.7%, +26 ± 19: kept | 1262 |
| T-L-NLs-merge-d2jit-c | qualified | 676 | 102 x 40 | 242 | 1617 | 48 | 1569 | 1614 | v495 | 1476 | new peak (+3 ± 51 over v495; predecessor +48) | 1030 (93696) | +453 -382 =165, 53.5%, +25 ± 20: kept | 677 |
| F-L-d4mix-c | qualified | 289 | 8 x 10 | 241 | 1751 | 54 | 1665 | 1745 | v200 | 1611 | new peak (+7 ± 58 over v200; predecessor +86) | 1030 (87385) | +359 -434 =207, 46.2%, -26 ± 19: reverted | 289 |
| MoE-d2jit-c | qualified | 1410 | 46 x 40 | 242 | 1706 | 52 | 1724 | 1724 | v1364 | 1565 | ok (-18 ± 55 vs peak v1364; predecessor -18) | 1030 (91909) | +349 -451 =200, 44.9%, -36 ± 19: reverted | 1410 |
