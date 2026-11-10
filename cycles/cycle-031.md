# Cycle 31

**Own-games step, pooled over the lineages: 25 gates, +10713 -10245 =4042, score 50.9%, Elo +6.5 ± 3.9; kept 15 of 25.**

All cycles so far: 1056 gates, +471438 -414556 =170006, score 52.7%, Elo +18.7 ± 0.6; kept 701 of 1056.

2026-11-07 08:08: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 42 players to retired.txt (their games stay in the fit), added 42.
Tournament: 5 double round-robins of the 63 active players (19530 games) in 3.8 min, 19530 games saved.
Freeze/retire rule, after 10 cycles dormant: F-L-d2play-c retired (avg ply 1 815.1, ply 2 1700.4; no untaken milestone within 10). 2 entries leave the field for retired.txt (their games stay in the fit): 63 players become 61, 19,530 games a cycle become 18,300.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| F-L-d0soft-c | 9 of 10 | cycle 32 | 23–31 | 979.1 | 1642.2 | retire |
| T-N-Le-merge-c | 8 of 10 | cycle 33 | 24–31 | 724.8 | 1347.7 | retire |
| F-L-c | 6 of 10 | cycle 35 | 26–31 | 978.7 | 1679.1 | retire |
| T-N-NLs-merge-c | 6 of 10 | cycle 35 | 26–31 | 640.4 | 1139.8 | retire |
| F-L-d1play-c | 5 of 10 | cycle 36 | 27–31 | 976.3 | 1535.2 | retire |
| T-L-NLs-c | 4 of 10 | cycle 37 | 28–31 | 979.9 | 1543.5 | retire |
| FM-c | 2 of 10 | cycle 39 | 30–31 | 875.9 | 1526.8 | retire |
| T-L-Le-c | 1 of 10 | cycle 40 | 31–31 | 889.5 | 1454.3 | retire |
| F-L-d2jit-c | 0 of 10 | cycle 41 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | dormant | 3143 |  |  | 1677 | 35 |  | 1716 (1716) | v2860 (v2860) | 1535 | dormant (-40 ± 23 vs peak v2860 before dormancy at v3138, at 1574 vs refs; needs > +23) | 1130 (92898) | +412 -429 =159, 49.1%, -6 ± 20: reverted | 3143 |
| F-L-d0soft-c | dormant | 2366 |  |  | 1647 | 34 |  | 1701 (1701) | v2014 (v2014) | 1505 | dormant (-54 ± 28 vs peak v2014 before dormancy at v2361, at 1559 vs refs; needs > +28) | 1130 (93601) | +440 -411 =149, 51.4%, +10 ± 20: kept | 2367 |
| F-L-d1play-c | dormant | 3318 |  |  | 1519 | 44 | 1558 | 1600 (1600) | v3006 (v3006) | 1377 | dormant (-80 ± 29 vs peak v3006 before dormancy at v3313, at 1458 vs refs; needs > +29) | 1130 (97202) | +435 -434 =131, 50.0%, +0 ± 20: kept | 3319 |
| F-L-d2play-c | retired | 1624 |  |  | 1684 | 35 |  | 1754 (1754) | v1011 (v1011) | 1542 | retired (avg ply 1 815.1, ply 2 1700.4; no untaken milestone within 10); dormant (-69 ± 26 vs peak v1011 before dormancy at v1620, at 1612 vs refs; needs > +26) |  |  | 1624 |
| F-L-d2jit-c | dormant | 3941 | 115 x 40 | 239 | 1750 | 52 | 1741 | 1818 | v3348 | 1608 | dormant (strike 2; -69 ± 54 vs peak v3348; predecessor +9) | 1130 (99203) | +345 -419 =236, 46.3%, -26 ± 19: reverted | 3941 |
| F-N-c | frozen | 1691 |  |  | 1379 | 18 |  | 1476 | v1061 | 1237 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1715 | 16 |  | 1720 | v300 | 1573 | frozen |  |  | 373 |
| FM-c | dormant | 625 |  |  | 1523 | 44 | 1527 | 1631 (1631) | v579 (v579) | 1381 | dormant (-108 ± 30 vs peak v579 before dormancy at v623, at 1489 vs refs; needs > +30) | 1130 (98491) | +404 -441 =155, 48.1%, -13 ± 20: reverted | 625 |
| F-P-c | qualified | 1261 | 174 x 1000 | 240 | 1468 | 43 | 1527 | 1527 | v1087 | 1326 | strike 1 (-60 ± 46 vs peak v1087; predecessor -60) | 1130 (98628) | +462 -317 =221, 57.2%, +51 ± 19: kept | 1262 |
| T-L-Ne-c | qualified | 1155 | 41 x 1000 | 242 | 1618 | 46 | 1551 | 1613 | v1034 | 1476 | new peak (+5 ± 47 over v1034; predecessor +67) | 1130 (97917) | +389 -455 =156, 46.7%, -23 ± 20: reverted | 1155 |
| T-L-Ne-fresh-c | qualified | 524 | 43 x 1000 | 240 | 1567 | 44 | 1538 | 1566 | v339 | 1425 | new peak (+1 ± 45 over v339; predecessor +28) | 1130 (95228) | +472 -394 =134, 53.9%, +27 ± 20: kept | 525 |
| T-L-Ne-pre-c | qualified | 349 | 38 x 1000 | 240 | 1497 | 44 | 1526 | 1526 | v310 | 1355 | ok (-29 ± 46 vs peak v310; predecessor -29) | 1130 (98798) | +476 -386 =138, 54.5%, +31 ± 20: kept | 350 |
| T-L-Nc-fresh-c | qualified | 701 | 45 x 1000 | 240 | 1590 | 45 | 1593 | 1596 | v610 | 1448 | ok (-6 ± 47 vs peak v610; predecessor -3) | 1130 (97124) | +376 -460 =164, 45.8%, -29 ± 20: reverted | 701 |
| D-L-Nc-c | qualified | 274 | 44 x 1000 | 240 | 1477 | 43 | 1451 | 1451 | v229 | 1335 | new peak (+26 ± 45 over v229; predecessor +26) | 1130 (96920) | +496 -389 =115, 55.4%, +37 ± 20: kept | 275 |
| T-L-LLs-c | frozen | 651 |  |  | 1611 | 13 |  | 1632 | v540 | 1469 | frozen |  |  | 651 |
| T-L-NLs-c | dormant | 1433 |  |  | 1536 | 20 |  | 1590 (1590) | v1345 (v1345) | 1394 | dormant (-54 ± 22 vs peak v1345 before dormancy at v1433, at 1448 vs refs; needs > +22) | 1130 (97825) | +354 -472 =174, 44.1%, -41 ± 20: reverted | 1433 |
| T-L-NNs-c | qualified | 1590 | 43 x 1000 | 238 | 1624 | 46 | 1573 | 1605 | v561 | 1482 | new peak (+18 ± 50 over v561; predecessor +50) | 1130 (99872) | +432 -429 =139, 50.1%, +1 ± 20: kept | 1591 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 971 | 13 |  | 1705 | v818 | 829 | frozen |  |  | 1070 |
| T-L-Le-c | dormant | 1172 |  |  | 1454 | 43 | 1506 | 1684 (1684) | v1081 (v1081) | 1312 | dormant (-229 ± 31 vs peak v1081 before dormancy at v1171, at 1542 vs refs; needs > +31) | 1130 (94401) | +534 -318 =148, 60.8%, +76 ± 20: kept | 1173 |
| T-L-Le-fresh-c | qualified | 594 | 48 x 1000 | 241 | 1606 | 45 | 1580 | 1580 | v545 | 1464 | new peak (+26 ± 47 over v545; predecessor +26) | 1130 (94672) | +465 -430 =105, 51.7%, +12 ± 20: kept | 595 |
| T-L-Le-pre-c | qualified | 335 | 36 x 1000 | 239 | 1474 | 43 | 1503 | 1503 | v299 | 1332 | ok (-29 ± 46 vs peak v299; predecessor -29) | 1130 (96716) | +414 -431 =155, 49.1%, -6 ± 20: reverted | 335 |
| T-N-Ne-c | frozen | 637 |  |  | 1276 | 12 |  | 1372 | v574 | 1134 | frozen |  |  | 637 |
| T-N-Nc-c | qualified | 1103 | 40 x 1000 | 238 | 1333 | 43 | 1364 | 1390 | v472 | 1191 | strike 1 (-57 ± 49 vs peak v472; predecessor -30) | 1130 (97075) | +416 -384 =200, 51.6%, +11 ± 19: kept | 1104 |
| T-N-Nc-merge-c | qualified | 1717 | 59 x 1000 | 239 | 1424 | 43 | 1398 | 1453 | v1598 | 1282 | ok (-29 ± 45 vs peak v1598; predecessor +27) | 1130 (93083) | +415 -427 =158, 49.4%, -4 ± 20: reverted | 1717 |
| T-N-Nc-fresh-c | qualified | 563 | 45 x 1000 | 238 | 1361 | 43 | 1366 | 1366 | v297 | 1219 | ok (-5 ± 43 vs peak v297; predecessor -5) | 1130 (93892) | +407 -444 =149, 48.1%, -13 ± 20: reverted | 563 |
| T-N-Le-merge-c | dormant | 754 |  |  | 1350 | 15 |  | 1430 (1430) | v700 (v700) | 1208 | dormant (-80 ± 20 vs peak v700 before dormancy at v754, at 1288 vs refs; needs > +20) | 1130 (90702) | +387 -474 =139, 45.6%, -30 ± 20: reverted | 754 |
| T-N-NLs-merge-c | dormant | 1034 |  |  | 1132 | 42 | 1138 | 1418 (1418) | v787 (v787) | 990 | dormant (-287 ± 25 vs peak v787 before dormancy at v1030, at 1276 vs refs; needs > +25) | 1130 (100322) | +423 -400 =177, 51.1%, +8 ± 20: kept | 1035 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 722 | 16 |  | 1348 | v270 | 580 | frozen |  |  | 417 |
| F-N-d2jit-c | qualified | 1919 | 132 x 40 | 241 | 1169 | 43 | 1263 | 1263 | v1786 | 1027 | strike 1 (-94 ± 45 vs peak v1786; predecessor -94) | 1130 (99714) | +405 -367 =228, 51.9%, +13 ± 19: kept | 1920 |
| T-N-NNs-merge-d2jit-c | qualified | 1920 | 75 x 40 | 240 | 1160 | 42 | 1156 | 1175 | v1553 | 1018 | ok (-15 ± 42 vs peak v1553; predecessor +4) | 1130 (96375) | +419 -379 =202, 52.0%, +14 ± 19: kept | 1921 |
| F-L-d4mix-c | qualified | 167 | 8 x 10 | 235 | 1651 | 47 | 1685 | 1722 | v141 | 1509 | strike 1 (-70 ± 50 vs peak v141; predecessor -34) | 1130 (98789) | +507 -361 =132, 57.3%, +51 ± 20: kept | 168 |
| MoE-d2jit-c | qualified | 789 | 49 x 40 | 240 | 1677 | 48 | 1615 | 1695 | v593 | 1535 | ok (-18 ± 48 vs peak v593; predecessor +62) | 1130 (98378) | +428 -394 =178, 51.7%, +12 ± 20: kept | 790 |
