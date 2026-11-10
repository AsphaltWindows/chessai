# Cycle 37

**Own-games step, pooled over the lineages: 23 gates, +10008 -8807 =4185, score 52.6%, Elo +18.2 ± 4.1; kept 18 of 23.**

All cycles so far: 1197 gates, +531881 -470284 =194835, score 52.6%, Elo +17.9 ± 0.6; kept 795 of 1197.

2026-11-07 10:49: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 44 players to retired.txt (their games stay in the fit), added 44.
Tournament: 5 double round-robins of the 59 active players (17110 games) in 4.2 min, 17110 games saved.
Freeze/retire rule, after 10 cycles dormant: T-L-NLs-c retired (avg ply 1 980.9, ply 2 1532.4; no untaken milestone within 10). 2 entries leave the field for retired.txt (their games stay in the fit): 59 players become 57, 17,110 games a cycle become 15,960.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| T-L-Le-c | 7 of 10 | cycle 40 | 31–37 | 925.0 | 1565.6 | retire |
| F-L-d2jit-c | 6 of 10 | cycle 41 | 32–37 | 802.3 | 1778.6 | freeze 1 at 800 |
| F-N-d2jit-c | 5 of 10 | cycle 42 | 33–37 | 618.5 | 1217.9 | retire |
| T-N-NNs-merge-d2jit-c | 3 of 10 | cycle 44 | 35–37 | 584.8 | 1130.3 | retire |
| FM-c | 2 of 10 | cycle 45 | 36–37 | 891.2 | 1628.8 | freeze 1 at 900 |
| T-N-Nc-fresh-c | 1 of 10 | cycle 46 | 37–37 | 689.0 | 1309.3 | retire |
| F-L-d1play-c | 0 of 10 | cycle 47 | - | - | - | retire |
| T-L-Le-fresh-c | 0 of 10 | cycle 47 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d1play-c | dormant | 3585 | 131 x 860 | 240 | 1496 | 46 | 1510 | 1638 | v3322 | 1356 | dormant (strike 2; -142 ± 50 vs peak v3322; predecessor -14) | 1050 (98074) | +443 -399 =158, 52.2%, +15 ± 20: kept | 3586 |
| F-L-d2jit-c | dormant | 3943 |  |  | 1808 | 60 | 1766 | 1812 (1812) | v3348 (v3348) | 1668 | dormant (-4 ± 39 vs peak v3348 before dormancy at v3941, at 1672 vs refs; needs > +39) | 1050 (90708) | +339 -425 =236, 45.7%, -30 ± 19: reverted | 3943 |
| F-N-c | frozen | 1691 |  |  | 1373 | 14 |  | 1474 | v1061 | 1233 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1702 | 13 |  | 1719 | v300 | 1561 | frozen |  |  | 373 |
| FM-c | dormant | 668 |  |  | 1643 | 49 | 1616 | 1665 (1665) | v626 (v626) | 1502 | dormant (-22 ± 35 vs peak v626 before dormancy at v666, at 1524 vs refs; needs > +35) | 1050 (92423) | +396 -433 =171, 48.1%, -13 ± 20: reverted | 668 |
| F-P-c | qualified | 2302 | 164 x 1000 | 241 | 1485 | 46 | 1592 | 1592 | v2138 | 1345 | strike 1 (-107 ± 50 vs peak v2138; predecessor -107) | 1050 (91771) | +423 -376 =201, 52.3%, +16 ± 19: kept | 2303 |
| T-L-Ne-c | qualified | 1351 | 33 x 1000 | 240 | 1587 | 47 | 1602 | 1622 | v1288 | 1447 | ok (-35 ± 51 vs peak v1288; predecessor -15) | 1050 (88353) | +367 -418 =215, 47.4%, -18 ± 19: reverted | 1351 |
| T-L-Ne-fresh-c | qualified | 722 | 31 x 1000 | 241 | 1589 | 47 | 1598 | 1612 | v633 | 1448 | ok (-23 ± 51 vs peak v633; predecessor -9) | 1050 (94350) | +479 -383 =138, 54.8%, +34 ± 20: kept | 723 |
| T-L-Ne-pre-c | qualified | 548 | 31 x 1000 | 238 | 1586 | 47 | 1579 | 1579 | v516 | 1445 | new peak (+7 ± 51 over v516; predecessor +7) | 1050 (93342) | +449 -418 =133, 51.5%, +11 ± 20: kept | 549 |
| T-L-Nc-fresh-c | qualified | 926 | 36 x 1000 | 239 | 1573 | 47 | 1589 | 1593 | v822 | 1432 | ok (-20 ± 51 vs peak v822; predecessor -16) | 1050 (91859) | +421 -394 =185, 51.3%, +9 ± 20: kept | 927 |
| D-L-Nc-c | qualified | 511 | 36 x 1000 | 240 | 1533 | 46 | 1551 | 1556 | v365 | 1392 | ok (-23 ± 49 vs peak v365; predecessor -19) | 1050 (92889) | +498 -374 =128, 56.2%, +43 ± 20: kept | 512 |
| T-L-LLs-c | frozen | 651 |  |  | 1610 | 11 |  | 1631 | v540 | 1469 | frozen |  |  | 651 |
| T-L-NLs-c | retired | 1433 |  |  | 1521 | 15 |  | 1585 (1585) | v1345 (v1345) | 1380 | retired (avg ply 1 980.9, ply 2 1532.4; no untaken milestone within 10); dormant (-64 ± 21 vs peak v1345 before dormancy at v1433, at 1444 vs refs; needs > +21) |  |  | 1433 |
| T-L-NNs-c | qualified | 1815 | 35 x 1000 | 240 | 1609 | 48 | 1601 | 1626 | v1590 | 1469 | ok (-16 ± 50 vs peak v1590; predecessor +8) | 1050 (89060) | +415 -427 =158, 49.4%, -4 ± 20: reverted | 1815 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 966 | 11 |  | 1703 | v818 | 826 | frozen |  |  | 1070 |
| T-L-Le-c | dormant | 1175 |  |  | 1607 | 25 |  | 1676 (1676) | v1081 (v1081) | 1466 | dormant (-69 ± 26 vs peak v1081 before dormancy at v1171, at 1536 vs refs; needs > +26) | 1050 (92717) | +383 -460 =157, 46.2%, -27 ± 20: reverted | 1175 |
| T-L-Le-fresh-c | dormant | 737 | 25 x 1000 | 239 | 1552 | 46 | 1564 | 1657 | v682 | 1411 | dormant (strike 2; -105 ± 51 vs peak v682; predecessor -12) | 1050 (88470) | +564 -332 =104, 61.6%, +82 ± 21: kept | 738 |
| T-L-Le-pre-c | qualified | 529 | 28 x 1000 | 237 | 1526 | 46 | 1506 | 1506 | v500 | 1385 | new peak (+20 ± 49 over v500; predecessor +20) | 1050 (91660) | +522 -351 =127, 58.6%, +60 ± 20: kept | 530 |
| T-N-Ne-c | frozen | 637 |  |  | 1272 | 10 |  | 1370 | v574 | 1131 | frozen |  |  | 637 |
| T-N-Nc-c | qualified | 1318 | 39 x 1000 | 239 | 1360 | 45 | 1378 | 1389 | v472 | 1220 | ok (-29 ± 50 vs peak v472; predecessor -18) | 1050 (90438) | +428 -414 =158, 50.7%, +5 ± 20: kept | 1319 |
| T-N-Nc-merge-c | qualified | 2025 | 52 x 1000 | 239 | 1414 | 45 | 1420 | 1448 | v631 | 1273 | ok (-34 ± 41 vs peak v631; predecessor -7) | 1050 (87785) | +438 -397 =165, 52.0%, +14 ± 20: kept | 2026 |
| T-N-Nc-fresh-c | dormant | 804 |  |  | 1309 | 45 | 1288 | 1368 (1368) | v624 (v624) | 1169 | dormant (-59 ± 31 vs peak v624 before dormancy at v803, at 1227 vs refs; needs > +31) | 1050 (89334) | +423 -404 =173, 50.9%, +7 ± 20: kept | 805 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 718 | 13 |  | 1346 | v270 | 578 | frozen |  |  | 417 |
| F-N-d2jit-c | dormant | 2051 |  |  | 1231 | 44 | 1210 | 1259 (1259) | v1786 (v1786) | 1090 | dormant (-28 ± 31 vs peak v1786 before dormancy at v2046, at 1118 vs refs; needs > +31) | 1050 (91943) | +432 -353 =215, 53.9%, +28 ± 19: kept | 2052 |
| T-N-NNs-merge-d2jit-c | dormant | 2114 |  |  | 1129 | 44 | 1137 | 1169 (1169) | v1553 (v1553) | 988 | dormant (-40 ± 29 vs peak v1553 before dormancy at v2112, at 1028 vs refs; needs > +29) | 1050 (96528) | +403 -401 =196, 50.1%, +1 ± 19: kept | 2115 |
| T-L-Le-merge-d2jit-c | qualified | 296 | 96 x 40 | 242 | 1386 | 45 | 1228 | 1228 | v199 | 1246 | new peak (+158 ± 48 over v199; predecessor +158) | 1050 (101322) | +528 -283 =189, 62.3%, +87 ± 20: kept | 297 |
| T-L-NLs-d2jit-c | qualified | 210 | 61 x 40 | 242 | 1060 | 43 | 951 | 951 | v148 | 919 | new peak (+109 ± 46 over v148; predecessor +109) | 1050 (111150) | +325 -269 =406, 52.8%, +20 ± 17: kept | 211 |
| T-N-Nc-merge-d2jit-c | qualified | 335 | 111 x 40 | 254 | 1144 | 44 | 1070 | 1070 | v223 | 1003 | new peak (+73 ± 46 over v223; predecessor +73) | 1050 (96435) | +460 -336 =204, 56.2%, +43 ± 19: kept | 336 |
| F-L-d4mix-c | qualified | 217 | 8 x 10 | 257 | 1698 | 52 | 1751 | 1754 | v200 | 1558 | ok (-55 ± 57 vs peak v200; predecessor -53) | 1050 (93766) | +436 -399 =165, 51.8%, +13 ± 20: kept | 218 |
| MoE-d2jit-c | qualified | 1039 | 47 x 40 | 240 | 1650 | 49 | 1710 | 1724 | v882 | 1510 | strike 1 (-74 ± 55 vs peak v882; predecessor -60) | 1050 (93375) | +436 -361 =203, 53.8%, +26 ± 19: kept | 1040 |
