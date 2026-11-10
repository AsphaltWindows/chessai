# Cycle 29

**Own-games step, pooled over the lineages: 26 gates, +11516 -10278 =4206, score 52.4%, Elo +16.6 ± 3.9; kept 19 of 26.**

All cycles so far: 1005 gates, +449746 -393432 =161822, score 52.8%, Elo +19.5 ± 0.6; kept 672 of 1005.

2026-11-07 07:19: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 42 players to retired.txt (their games stay in the fit), added 42.
Tournament: 5 double round-robins of the 69 active players (23460 games) in 3.7 min, 23460 games saved.
Freeze/retire rule, after 10 cycles dormant: F-L-d2mix-c retired (avg ply 1 849.9, ply 2 1735.4; no untaken milestone within 10); T-L-LLs-merge-c retired (avg ply 1 918.7, ply 2 1574.2; no untaken milestone within 10); T-L-Nc-merge-c retired (avg ply 1 957.3, ply 2 1682.1; no untaken milestone within 10). 6 entries leave the field for retired.txt (their games stay in the fit): 69 players become 63, 23,460 games a cycle become 19,530.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| F-L-d2play-c | 8 of 10 | cycle 31 | 22–29 | 814.8 | 1703.0 | retire |
| F-L-d0soft-c | 7 of 10 | cycle 32 | 23–29 | 977.3 | 1642.7 | retire |
| T-N-Le-merge-c | 6 of 10 | cycle 33 | 24–29 | 725.6 | 1347.4 | retire |
| F-L-c | 4 of 10 | cycle 35 | 26–29 | 982.5 | 1676.9 | retire |
| T-N-NLs-merge-c | 4 of 10 | cycle 35 | 26–29 | 640.4 | 1141.3 | retire |
| F-L-d1play-c | 3 of 10 | cycle 36 | 27–29 | 972.6 | 1531.7 | retire |
| F-N-d2jit-c | 2 of 10 | cycle 37 | 28–29 | 627.5 | 1210.5 | retire |
| T-L-NLs-c | 2 of 10 | cycle 37 | 28–29 | 976.8 | 1546.0 | retire |
| FM-c | 0 of 10 | cycle 39 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | dormant | 3142 |  |  | 1700 | 46 | 1702 | 1717 (1717) | v2860 (v2860) | 1558 | dormant (-16 ± 28 vs peak v2860 before dormancy at v3138, at 1574 vs refs; needs > +28) | 1250 (101915) | +471 -397 =132, 53.7%, +26 ± 20: kept | 3143 |
| F-L-d0soft-c | dormant | 2365 |  |  | 1657 | 32 |  | 1701 (1701) | v2014 (v2014) | 1515 | dormant (-44 ± 27 vs peak v2014 before dormancy at v2361, at 1559 vs refs; needs > +27) | 1250 (102513) | +427 -426 =147, 50.0%, +0 ± 20: kept | 2366 |
| F-L-d1play-c | dormant | 3316 |  |  | 1548 | 42 | 1540 | 1600 (1600) | v3006 (v3006) | 1406 | dormant (-52 ± 29 vs peak v3006 before dormancy at v3313, at 1458 vs refs; needs > +29) | 1250 (109426) | +442 -402 =156, 52.0%, +14 ± 20: kept | 3317 |
| F-L-d2play-c | dormant | 1623 |  |  | 1694 | 24 |  | 1755 (1755) | v1011 (v1011) | 1551 | dormant (-61 ± 22 vs peak v1011 before dormancy at v1620, at 1612 vs refs; needs > +22) | 1250 (108729) | +449 -359 =192, 54.5%, +31 ± 19: kept | 1624 |
| F-L-d2mix-c | retired | 1075 |  |  | 1745 | 25 |  | 1745 (1744) | v1073 (v910) | 1603 | retired (avg ply 1 849.9, ply 2 1735.4; no untaken milestone within 10); dormant (+1 ± 27 vs peak v910 before dormancy at v1068, at 1601 vs refs; needs > +27) |  |  | 1075 |
| F-L-d2jit-c | qualified | 3711 | 125 x 40 | 240 | 1812 | 52 | 1793 | 1820 | v3348 | 1669 | ok (-8 ± 54 vs peak v3348; predecessor +19) | 1250 (112082) | +346 -372 =282, 48.7%, -9 ± 18: reverted | 3711 |
| F-N-c | frozen | 1691 |  |  | 1388 | 21 |  | 1477 | v1061 | 1246 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1715 | 17 |  | 1721 | v300 | 1573 | frozen |  |  | 373 |
| FM-c | dormant | 623 | 22 x 1000 | 244 | 1391 | 42 | 1431 | 1634 | v579 | 1249 | dormant (strike 2; -243 ± 44 vs peak v579; predecessor -40) | 1250 (102321) | +677 -240 =83, 71.9%, +163 ± 23: kept | 624 |
| F-P-c | qualified | 908 | 191 x 1000 | 240 | 1502 | 42 | 1503 | 1508 | v528 | 1360 | ok (-6 ± 43 vs peak v528; predecessor -0) | 1250 (113100) | +411 -383 =206, 51.4%, +10 ± 19: kept | 909 |
| T-L-Ne-c | qualified | 1074 | 39 x 1000 | 239 | 1609 | 43 | 1616 | 1616 | v1034 | 1466 | ok (-7 ± 45 vs peak v1034; predecessor -7) | 1250 (108121) | +412 -421 =167, 49.5%, -3 ± 20: reverted | 1074 |
| T-L-Ne-fresh-c | qualified | 437 | 48 x 1000 | 241 | 1563 | 42 | 1566 | 1568 | v339 | 1421 | ok (-5 ± 43 vs peak v339; predecessor -2) | 1250 (107134) | +469 -388 =143, 54.0%, +28 ± 20: kept | 438 |
| T-L-Ne-pre-c | qualified | 265 | 47 x 1000 | 238 | 1507 | 42 | 1475 | 1475 | v217 | 1364 | new peak (+32 ± 43 over v217; predecessor +32) | 1250 (112235) | +492 -361 =147, 56.5%, +46 ± 20: kept | 266 |
| T-L-Nc-merge-c | retired | 1885 |  |  | 1710 | 34 |  | 1731 (1728) | v1884 (v1631) | 1567 | retired (avg ply 1 957.3, ply 2 1682.1; no untaken milestone within 10); dormant (-18 ± 29 vs peak v1631 before dormancy at v1878, at 1586 vs refs; needs > +29) |  |  | 1885 |
| T-L-Nc-fresh-c | qualified | 610 | 45 x 1000 | 243 | 1599 | 43 | 1576 | 1576 | v564 | 1457 | new peak (+23 ± 45 over v564; predecessor +23) | 1250 (104455) | +414 -408 =178, 50.3%, +2 ± 20: kept | 611 |
| D-L-Nc-c | qualified | 183 | 43 x 1000 | 242 | 1345 | 42 | 1406 | 1406 | v139 | 1202 | strike 1 (-61 ± 43 vs peak v139; predecessor -61) | 1250 (110412) | +641 -230 =129, 70.5%, +152 ± 22: kept | 184 |
| T-L-LLs-c | frozen | 651 |  |  | 1610 | 14 |  | 1633 | v540 | 1468 | frozen |  |  | 651 |
| T-L-LLs-merge-c | retired | 1761 |  |  | 1617 | 32 |  | 1627 (1609) | v1760 (v1251) | 1475 | retired (avg ply 1 918.7, ply 2 1574.2; no untaken milestone within 10); dormant (+8 ± 27 vs peak v1251 before dormancy at v1754, at 1466 vs refs; needs > +27) |  |  | 1761 |
| T-L-NLs-c | dormant | 1433 |  |  | 1550 | 25 |  | 1590 (1590) | v1345 (v1345) | 1408 | dormant (-40 ± 23 vs peak v1345 before dormancy at v1433, at 1448 vs refs; needs > +23) | 1250 (107846) | +353 -517 =130, 41.8%, -58 ± 20: reverted | 1433 |
| T-L-NNs-c | qualified | 1502 | 44 x 1000 | 242 | 1594 | 43 | 1593 | 1606 | v561 | 1452 | ok (-12 ± 49 vs peak v561; predecessor +1) | 1250 (109847) | +434 -402 =164, 51.6%, +11 ± 20: kept | 1503 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 971 | 14 |  | 1706 | v818 | 829 | frozen |  |  | 1070 |
| T-L-Le-c | qualified | 1125 | 44 x 1000 | 239 | 1470 | 42 | 1686 | 1686 | v1081 | 1328 | strike 1 (-216 ± 45 vs peak v1081; predecessor -216) | 1250 (108863) | +496 -387 =117, 55.5%, +38 ± 20: kept | 1126 |
| T-L-Le-fresh-c | qualified | 487 | 78 x 1000 | 242 | 1564 | 42 | 1536 | 1536 | v408 | 1422 | new peak (+28 ± 44 over v408; predecessor +28) | 1250 (101939) | +463 -399 =138, 53.2%, +22 ± 20: kept | 488 |
| T-L-Le-pre-c | qualified | 254 | 45 x 1000 | 242 | 1470 | 42 | 1486 | 1486 | v209 | 1328 | ok (-16 ± 43 vs peak v209; predecessor -16) | 1250 (105547) | +376 -492 =132, 44.2%, -40 ± 20: reverted | 254 |
| T-N-Ne-c | frozen | 637 |  |  | 1278 | 13 |  | 1372 | v574 | 1136 | frozen |  |  | 637 |
| T-N-Nc-c | qualified | 1020 | 43 x 1000 | 243 | 1383 | 42 | 1377 | 1392 | v472 | 1241 | ok (-8 ± 48 vs peak v472; predecessor +6) | 1250 (103073) | +436 -399 =165, 51.8%, +13 ± 20: kept | 1021 |
| T-N-Nc-merge-c | qualified | 1598 | 65 x 1000 | 239 | 1456 | 42 | 1429 | 1450 | v631 | 1313 | new peak (+5 ± 39 over v631; predecessor +27) | 1250 (99655) | +415 -408 =177, 50.3%, +2 ± 20: kept | 1599 |
| T-N-Nc-fresh-c | qualified | 457 | 48 x 1000 | 238 | 1364 | 42 | 1352 | 1368 | v297 | 1222 | ok (-4 ± 42 vs peak v297; predecessor +12) | 1250 (104389) | +396 -460 =144, 46.8%, -22 ± 20: reverted | 457 |
| T-N-Le-merge-c | dormant | 754 |  |  | 1347 | 16 |  | 1430 (1430) | v700 (v700) | 1205 | dormant (-83 ± 20 vs peak v700 before dormancy at v754, at 1288 vs refs; needs > +20) | 1250 (103557) | +377 -501 =122, 43.8%, -43 ± 20: reverted | 754 |
| T-N-NLs-merge-c | dormant | 1032 |  |  | 1136 | 30 |  | 1419 (1419) | v787 (v787) | 994 | dormant (-283 ± 20 vs peak v787 before dormancy at v1030, at 1277 vs refs; needs > +20) | 1250 (109587) | +432 -409 =159, 51.1%, +8 ± 20: kept | 1033 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 722 | 18 |  | 1349 | v270 | 579 | frozen |  |  | 417 |
| F-N-d2jit-c | dormant | 1785 |  |  | 1216 | 41 | 1206 | 1219 (1219) | v1523 (v1523) | 1073 | dormant (-3 ± 28 vs peak v1523 before dormancy at v1783, at 1077 vs refs; needs > +28) | 1250 (107540) | +432 -348 =220, 54.2%, +29 ± 19: kept | 1786 |
| T-N-NNs-merge-d2jit-c | qualified | 1747 | 94 x 40 | 240 | 1145 | 41 | 1145 | 1177 | v1553 | 1003 | ok (-32 ± 41 vs peak v1553; predecessor +0) | 1250 (108238) | +424 -372 =204, 52.6%, +18 ± 19: kept | 1748 |
| F-L-d4mix-c | qualified | 150 | 8 x 10 | 241 | 1648 | 44 | 1724 | 1724 | v141 | 1506 | strike 1 (-76 ± 48 vs peak v141; predecessor -76) | 1250 (107620) | +443 -372 =185, 53.5%, +25 ± 20: kept | 151 |
| MoE-d2jit-c | qualified | 689 | 51 x 40 | 243 | 1698 | 46 | 1670 | 1698 | v593 | 1555 | ok (-0 ± 47 vs peak v593; predecessor +28) | 1250 (109910) | +388 -425 =187, 48.1%, -13 ± 19: reverted | 689 |
