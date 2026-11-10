# Cycle 32

**Own-games step, pooled over the lineages: 24 gates, +10236 -9779 =3985, score 51.0%, Elo +6.6 ± 4.0; kept 14 of 24.**

All cycles so far: 1080 gates, +481674 -424335 =173991, score 52.7%, Elo +18.5 ± 0.6; kept 715 of 1080.

2026-11-07 08:32: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 40 players to retired.txt (their games stay in the fit), added 40.
Tournament: 5 double round-robins of the 61 active players (18300 games) in 3.7 min, 18300 games saved.
Freeze/retire rule, after 10 cycles dormant: F-L-d0soft-c retired (avg ply 1 976.6, ply 2 1644.5; no untaken milestone within 10). 2 entries leave the field for retired.txt (their games stay in the fit): 61 players become 59, 18,300 games a cycle become 17,110.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| T-N-Le-merge-c | 9 of 10 | cycle 33 | 24–32 | 724.2 | 1348.0 | retire |
| F-L-c | 7 of 10 | cycle 35 | 26–32 | 977.8 | 1681.1 | retire |
| T-N-NLs-merge-c | 7 of 10 | cycle 35 | 26–32 | 639.1 | 1145.0 | retire |
| F-L-d1play-c | 6 of 10 | cycle 36 | 27–32 | 979.4 | 1545.1 | retire |
| T-L-NLs-c | 5 of 10 | cycle 37 | 28–32 | 981.4 | 1542.0 | retire |
| FM-c | 3 of 10 | cycle 39 | 30–32 | 879.0 | 1536.6 | retire |
| T-L-Le-c | 2 of 10 | cycle 40 | 31–32 | 913.2 | 1495.6 | freeze 2 at 1500 |
| F-L-d2jit-c | 1 of 10 | cycle 41 | 32–32 | 795.2 | 1774.3 | freeze 1 at 800 |
| F-N-d2jit-c | 0 of 10 | cycle 42 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | dormant | 3143 |  |  | 1693 | 29 |  | 1717 (1717) | v2860 (v2860) | 1550 | dormant (-24 ± 21 vs peak v2860 before dormancy at v3138, at 1574 vs refs; needs > +21) | 1090 (86069) | +422 -413 =165, 50.4%, +3 ± 20: kept | 3144 |
| F-L-d0soft-c | retired | 2367 |  |  | 1665 | 48 | 1652 | 1702 (1702) | v2014 (v2014) | 1522 | retired (avg ply 1 976.6, ply 2 1644.5; no untaken milestone within 10); dormant (-37 ± 32 vs peak v2014 before dormancy at v2361, at 1559 vs refs; needs > +32) |  |  | 2367 |
| F-L-d1play-c | dormant | 3319 |  |  | 1594 | 46 | 1526 | 1600 (1600) | v3006 (v3006) | 1452 | dormant (-6 ± 30 vs peak v3006 before dormancy at v3313, at 1458 vs refs; needs > +30) | 1090 (95874) | +453 -407 =140, 52.3%, +16 ± 20: kept | 3320 |
| F-L-d2jit-c | dormant | 3941 |  |  | 1774 | 38 |  | 1818 (1818) | v3348 (v3348) | 1632 | dormant (-44 ± 31 vs peak v3348 before dormancy at v3941, at 1676 vs refs; needs > +31) | 1090 (95638) | +341 -386 =273, 47.8%, -16 ± 18: reverted | 3941 |
| F-N-c | frozen | 1691 |  |  | 1375 | 17 |  | 1477 | v1061 | 1232 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1713 | 15 |  | 1721 | v300 | 1570 | frozen |  |  | 373 |
| FM-c | dormant | 625 |  |  | 1556 | 32 |  | 1631 (1631) | v579 (v579) | 1414 | dormant (-75 ± 26 vs peak v579 before dormancy at v623, at 1489 vs refs; needs > +26) | 1090 (95250) | +463 -377 =160, 54.3%, +30 ± 20: kept | 626 |
| F-P-c | qualified | 1458 | 196 x 1000 | 240 | 1502 | 44 | 1474 | 1530 | v1087 | 1359 | ok (-29 ± 46 vs peak v1087; predecessor +27) | 1090 (96847) | +502 -332 =166, 58.5%, +60 ± 20: kept | 1459 |
| T-L-Ne-c | qualified | 1192 | 37 x 1000 | 240 | 1588 | 45 | 1625 | 1625 | v1155 | 1445 | ok (-38 ± 48 vs peak v1155; predecessor -38) | 1090 (93442) | +412 -419 =169, 49.6%, -2 ± 20: reverted | 1192 |
| T-L-Ne-fresh-c | qualified | 566 | 41 x 1000 | 241 | 1568 | 45 | 1574 | 1574 | v524 | 1425 | ok (-6 ± 47 vs peak v524; predecessor -6) | 1090 (94364) | +430 -419 =151, 50.5%, +4 ± 20: kept | 567 |
| T-L-Ne-pre-c | qualified | 389 | 39 x 1000 | 242 | 1524 | 44 | 1504 | 1529 | v310 | 1382 | ok (-5 ± 46 vs peak v310; predecessor +20) | 1090 (93864) | +436 -403 =161, 51.6%, +12 ± 20: kept | 390 |
| T-L-Nc-fresh-c | qualified | 745 | 44 x 1000 | 243 | 1560 | 45 | 1597 | 1597 | v701 | 1418 | ok (-37 ± 47 vs peak v701; predecessor -37) | 1090 (92137) | +447 -377 =176, 53.5%, +24 ± 20: kept | 746 |
| D-L-Nc-c | qualified | 320 | 45 x 1000 | 238 | 1508 | 44 | 1484 | 1484 | v274 | 1365 | new peak (+24 ± 46 over v274; predecessor +24) | 1090 (91742) | +493 -369 =138, 56.2%, +43 ± 20: kept | 321 |
| T-L-LLs-c | frozen | 651 |  |  | 1614 | 13 |  | 1633 | v540 | 1471 | frozen |  |  | 651 |
| T-L-NLs-c | dormant | 1433 |  |  | 1536 | 19 |  | 1591 (1591) | v1345 (v1345) | 1393 | dormant (-55 ± 22 vs peak v1345 before dormancy at v1433, at 1448 vs refs; needs > +22) | 1090 (90997) | +295 -561 =144, 36.7%, -95 ± 21: reverted | 1433 |
| T-L-NNs-c | qualified | 1635 | 44 x 1000 | 239 | 1616 | 46 | 1631 | 1631 | v1590 | 1474 | ok (-14 ± 49 vs peak v1590; predecessor -14) | 1090 (93073) | +384 -441 =175, 47.1%, -20 ± 20: reverted | 1635 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 972 | 12 |  | 1706 | v818 | 829 | frozen |  |  | 1070 |
| T-L-Le-c | dormant | 1173 |  |  | 1537 | 44 | 1461 | 1684 (1684) | v1081 (v1081) | 1394 | dormant (-147 ± 32 vs peak v1081 before dormancy at v1171, at 1541 vs refs; needs > +32) | 1090 (89727) | +482 -387 =131, 54.8%, +33 ± 20: kept | 1174 |
| T-L-Le-fresh-c | qualified | 642 | 47 x 1000 | 242 | 1527 | 44 | 1613 | 1613 | v594 | 1384 | strike 1 (-87 ± 47 vs peak v594; predecessor -87) | 1090 (90525) | +597 -311 =92, 64.3%, +102 ± 21: kept | 643 |
| T-L-Le-pre-c | qualified | 369 | 34 x 1000 | 242 | 1470 | 44 | 1480 | 1506 | v299 | 1328 | ok (-36 ± 46 vs peak v299; predecessor -10) | 1090 (88552) | +443 -388 =169, 52.8%, +19 ± 20: kept | 370 |
| T-N-Ne-c | frozen | 637 |  |  | 1276 | 12 |  | 1372 | v574 | 1134 | frozen |  |  | 637 |
| T-N-Nc-c | qualified | 1145 | 41 x 1000 | 239 | 1355 | 43 | 1340 | 1392 | v472 | 1212 | ok (-36 ± 49 vs peak v472; predecessor +16) | 1090 (89890) | +410 -421 =169, 49.5%, -4 ± 20: reverted | 1145 |
| T-N-Nc-merge-c | qualified | 1783 | 66 x 1000 | 242 | 1412 | 44 | 1431 | 1453 | v1598 | 1269 | ok (-41 ± 45 vs peak v1598; predecessor -19) | 1090 (89774) | +407 -452 =141, 47.8%, -16 ± 20: reverted | 1783 |
| T-N-Nc-fresh-c | qualified | 624 | 61 x 1000 | 249 | 1374 | 43 | 1368 | 1369 | v517 | 1231 | new peak (+5 ± 46 over v517; predecessor +6) | 1090 (89202) | +423 -428 =149, 49.8%, -2 ± 20: reverted | 624 |
| T-N-Le-merge-c | dormant | 754 |  |  | 1351 | 14 |  | 1430 (1430) | v700 (v700) | 1208 | dormant (-80 ± 19 vs peak v700 before dormancy at v754, at 1288 vs refs; needs > +19) | 1090 (89379) | +390 -482 =128, 45.4%, -32 ± 20: reverted | 754 |
| T-N-NLs-merge-c | dormant | 1035 |  |  | 1176 | 43 | 1137 | 1419 (1419) | v787 (v787) | 1033 | dormant (-244 ± 25 vs peak v787 before dormancy at v1030, at 1277 vs refs; needs > +25) | 1090 (95543) | +404 -392 =204, 50.6%, +4 ± 19: kept | 1036 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 723 | 15 |  | 1349 | v270 | 580 | frozen |  |  | 417 |
| F-N-d2jit-c | dormant | 2046 | 126 x 40 | 240 | 1215 | 43 | 1175 | 1266 | v1786 | 1072 | dormant (strike 2; -50 ± 46 vs peak v1786; predecessor +40) | 1090 (95190) | +406 -382 =212, 51.2%, +8 ± 19: kept | 2047 |
| T-N-NNs-merge-d2jit-c | qualified | 1991 | 70 x 40 | 242 | 1168 | 43 | 1165 | 1176 | v1553 | 1026 | ok (-7 ± 43 vs peak v1553; predecessor +3) | 1090 (96125) | +376 -405 =219, 48.5%, -10 ± 19: reverted | 1991 |
| F-L-d4mix-c | qualified | 176 | 8 x 10 | 235 | 1676 | 48 | 1659 | 1722 | v141 | 1533 | ok (-46 ± 50 vs peak v141; predecessor +17) | 1090 (92233) | +423 -396 =181, 51.3%, +9 ± 20: kept | 177 |
| MoE-d2jit-c | qualified | 837 | 47 x 40 | 240 | 1713 | 50 | 1685 | 1695 | v593 | 1571 | new peak (+18 ± 50 over v593; predecessor +28) | 1090 (94538) | +397 -431 =172, 48.3%, -12 ± 20: reverted | 837 |
