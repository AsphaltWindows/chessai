# Cycle 47

**Own-games step, pooled over the lineages: 20 gates, +8598 -7492 =3910, score 52.8%, Elo +19.2 ± 4.3; kept 16 of 20.**

All cycles so far: 1432 gates, +632492 -561142 =238366, score 52.5%, Elo +17.3 ± 0.5; kept 955 of 1432.

2026-11-07 14:53: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 36 players to retired.txt (their games stay in the fit), added 36.
Tournament: 5 double round-robins of the 56 active players (15400 games) in 5.9 min, 15400 games saved.
Freeze/retire rule, after 10 cycles dormant: F-L-d1play-c retired (avg ply 1 986.4, ply 2 1547.5; no untaken milestone within 10); T-L-Le-fresh-c frozen at ply 1 (avg 908.7 over cycles 38–47; milestone 900). 3 entries leave the field for retired.txt (their games stay in the fit): 56 players become 53, 15,400 games a cycle become 13,780.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| T-L-Ne-c | 8 of 10 | cycle 49 | 40–47 | 1012.3 | 1564.2 | retire |
| T-N-Nc-merge-c | 8 of 10 | cycle 49 | 40–47 | 758.3 | 1390.4 | retire |
| F-P-c | 7 of 10 | cycle 50 | 41–47 | 926.9 | 1492.2 | freeze 2 at 1500 |
| F-N-d2jit-c | 5 of 10 | cycle 52 | 43–47 | 649.2 | 1217.4 | retire |
| T-N-Nc-c | 5 of 10 | cycle 52 | 43–47 | 746.7 | 1347.0 | retire |
| T-L-Le-pre-c | 4 of 10 | cycle 53 | 44–47 | 899.3 | 1505.4 | retire |
| T-L-Ne-pre-c | 4 of 10 | cycle 53 | 44–47 | 937.5 | 1573.8 | retire |
| T-L-Ne-fresh-c | 1 of 10 | cycle 56 | 47–47 | 945.4 | 1574.1 | retire |
| T-N-Nc-merge-d2jit-c | 1 of 10 | cycle 56 | 47–47 | 613.9 | 1160.6 | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 900 T-L-Le-fresh-c-v742-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d1play-c | retired | 3595 |  |  | 1560 | 47 | 1603 | 1628 (1628) | v3322 (v3322) | 1419 | retired (avg ply 1 986.4, ply 2 1547.5; no untaken milestone within 10); dormant (-68 ± 34 vs peak v3322 before dormancy at v3585, at 1487 vs refs; needs > +34) |  |  | 3595 |
| F-L-d2jit-c | frozen | 3943 |  |  | 813 | 14 |  | 1803 | v3348 | 672 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1371 | 10 |  | 1474 | v1061 | 1230 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1685 | 11 |  | 1718 | v300 | 1544 | frozen |  |  | 373 |
| F-P-c | dormant | 2806 |  |  | 1447 | 46 | 1508 | 1582 (1582) | v2138 (v2138) | 1306 | dormant (-135 ± 34 vs peak v2138 before dormancy at v2801, at 1441 vs refs; needs > +34) | 990 (89657) | +441 -386 =173, 52.8%, +19 ± 20: kept | 2807 |
| F-P-tau0-c | qualified | 1773 | 184 x 1000 | 240 | 1515 | 47 | 1530 | 1565 | v862 | 1374 | strike 1 (-50 ± 50 vs peak v862; predecessor -15) | 990 (82695) | +403 -388 =209, 50.7%, +5 ± 19: kept | 1774 |
| F-P-tau0-d2jit-c | qualified | 1279 | 144 x 40 | 241 | 1712 | 54 | 1707 | 1716 | v847 | 1571 | ok (-4 ± 56 vs peak v847; predecessor +5) | 990 (89243) | +316 -354 =330, 48.1%, -13 ± 18: reverted | 1279 |
| T-L-Ne-c | dormant | 1427 |  |  | 1538 | 47 | 1609 | 1612 (1612) | v1288 (v1288) | 1397 | dormant (-74 ± 34 vs peak v1288 before dormancy at v1425, at 1472 vs refs; needs > +34) | 990 (85898) | +442 -408 =150, 51.7%, +12 ± 20: kept | 1428 |
| T-L-Ne-fresh-c | dormant | 1039 |  |  | 1574 | 48 | 1544 | 1602 (1602) | v633 (v633) | 1433 | dormant (-28 ± 34 vs peak v633 before dormancy at v1038, at 1461 vs refs; needs > +34) | 990 (84508) | +433 -381 =186, 52.6%, +18 ± 20: kept | 1040 |
| T-L-Ne-pre-c | dormant | 755 |  |  | 1552 | 47 | 1573 | 1580 (1576) | v753 (v691) | 1411 | dormant (-24 ± 33 vs peak v691 before dormancy at v752, at 1435 vs refs; needs > +33) | 990 (87884) | +437 -393 =170, 52.2%, +15 ± 20: kept | 756 |
| T-L-Nc-fresh-c | qualified | 1326 | 39 x 1000 | 240 | 1576 | 48 | 1564 | 1592 | v1015 | 1435 | ok (-17 ± 50 vs peak v1015; predecessor +11) | 990 (85307) | +376 -421 =203, 47.8%, -16 ± 19: reverted | 1326 |
| D-L-Nc-c | qualified | 937 | 42 x 1000 | 240 | 1513 | 47 | 1533 | 1575 | v851 | 1372 | strike 1 (-62 ± 49 vs peak v851; predecessor -21) | 990 (83838) | +482 -354 =164, 56.4%, +45 ± 20: kept | 938 |
| T-L-LLs-c | frozen | 651 |  |  | 1592 | 9 |  | 1630 | v540 | 1451 | frozen |  |  | 651 |
| T-L-NNs-c | qualified | 2233 | 40 x 1000 | 243 | 1564 | 48 | 1584 | 1616 | v1590 | 1423 | strike 1 (-52 ± 50 vs peak v1590; predecessor -20) | 990 (81882) | +418 -400 =182, 50.9%, +6 ± 20: kept | 2234 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 960 | 9 |  | 1703 | v818 | 819 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | frozen | 742 |  |  | 1628 | 35 |  | 1646 (1646) | v682 (v682) | 1487 | frozen at ply 1 (avg 908.7 over cycles 38–47; milestone 900); dormant (-19 ± 31 vs peak v682 before dormancy at v737, at 1505 vs refs; needs > +31) |  |  | 742 |
| T-L-Le-pre-c | dormant | 737 |  |  | 1512 | 27 |  | 1525 (1525) | v670 (v670) | 1371 | dormant (-13 ± 26 vs peak v670 before dormancy at v735, at 1384 vs refs; needs > +26) | 990 (86116) | +416 -400 =184, 50.8%, +6 ± 20: kept | 738 |
| T-N-Ne-c | frozen | 637 |  |  | 1264 | 9 |  | 1369 | v574 | 1123 | frozen |  |  | 637 |
| T-N-Nc-c | dormant | 1503 |  |  | 1351 | 33 |  | 1389 (1389) | v472 (v472) | 1210 | dormant (-38 ± 29 vs peak v472 before dormancy at v1500, at 1248 vs refs; needs > +29) | 990 (84314) | +436 -391 =173, 52.2%, +16 ± 20: kept | 1504 |
| T-N-Nc-merge-c | dormant | 2129 |  |  | 1410 | 46 | 1385 | 1457 (1447) | v2127 (v631) | 1269 | dormant (-37 ± 28 vs peak v631 before dormancy at v2126, at 1306 vs refs; needs > +28) | 990 (81488) | +386 -429 =185, 47.9%, -15 ± 20: reverted | 2129 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 713 | 10 |  | 1345 | v270 | 572 | frozen |  |  | 417 |
| F-N-d2jit-c | dormant | 2422 |  |  | 1212 | 33 |  | 1290 (1290) | v2053 (v2053) | 1071 | dormant (-78 ± 27 vs peak v2053 before dormancy at v2418, at 1149 vs refs; needs > +27) | 990 (86564) | +434 -374 =192, 53.0%, +21 ± 19: kept | 2423 |
| T-N-NNs-merge-d2jit-c | qualified | 2645 | 96 x 40 | 242 | 1175 | 46 | 1178 | 1214 | v2345 | 1034 | ok (-39 ± 48 vs peak v2345; predecessor -4) | 990 (90989) | +416 -380 =204, 51.8%, +12 ± 19: kept | 2646 |
| T-L-Le-merge-d2jit-c | qualified | 1284 | 101 x 40 | 241 | 1560 | 47 | 1716 | 1716 | v1182 | 1419 | strike 1 (-156 ± 53 vs peak v1182; predecessor -156) | 990 (91169) | +479 -343 =178, 56.8%, +48 ± 20: kept | 1285 |
| T-L-NLs-d2jit-c | qualified | 782 | 56 x 40 | 240 | 1474 | 46 | 1486 | 1486 | v725 | 1333 | ok (-13 ± 49 vs peak v725; predecessor -13) | 990 (92975) | +413 -375 =212, 51.9%, +13 ± 19: kept | 783 |
| T-N-Nc-merge-d2jit-c | dormant | 1262 |  |  | 1161 | 46 | 1123 | 1171 (1171) | v655 (v655) | 1020 | dormant (-10 ± 32 vs peak v655 before dormancy at v1261, at 1030 vs refs; needs > +32) | 990 (90113) | +393 -361 =246, 51.6%, +11 ± 19: kept | 1263 |
| T-L-NLs-merge-d2jit-c | qualified | 766 | 89 x 40 | 241 | 1506 | 47 | 1616 | 1616 | v676 | 1365 | strike 1 (-111 ± 50 vs peak v676; predecessor -111) | 990 (89671) | +659 -187 =154, 73.6%, +178 ± 22: kept | 767 |
| F-L-d4mix-c | qualified | 296 | 7 x 10 | 233 | 1676 | 52 | 1751 | 1751 | v289 | 1535 | strike 1 (-75 ± 56 vs peak v289; predecessor -75) | 990 (85966) | +388 -427 =185, 48.0%, -14 ± 20: reverted | 296 |
| MoE-d2jit-c | qualified | 1453 | 43 x 40 | 243 | 1663 | 51 | 1706 | 1723 | v1364 | 1522 | strike 1 (-60 ± 55 vs peak v1364; predecessor -43) | 990 (85619) | +430 -340 =230, 54.5%, +31 ± 19: kept | 1454 |
