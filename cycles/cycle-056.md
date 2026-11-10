# Cycle 56

**Own-games step, pooled over the lineages: 14 gates, +5546 -5485 =2969, score 50.2%, Elo +1.5 ± 5.1; kept 8 of 14.**

All cycles so far: 1578 gates, +691915 -618613 =267472, score 52.3%, Elo +16.2 ± 0.5; kept 1044 of 1578.

2026-11-07 17:39: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 20 players to retired.txt (their games stay in the fit), added 20.
Tournament: 5 double round-robins of the 44 active players (9460 games) in 6.5 min, 9460 games saved.
Freeze/retire rule, after 10 cycles dormant: T-L-Ne-fresh-c retired (avg ply 1 951.8, ply 2 1588.3; no untaken milestone within 10). 2 entries leave the field for retired.txt (their games stay in the fit): 44 players become 42, 9,460 games a cycle become 8,610.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| F-P-tau0-c | 8 of 10 | cycle 58 | 49–56 | 1001.4 | 1544.4 | retire |
| T-L-NNs-c | 8 of 10 | cycle 58 | 49–56 | 955.7 | 1591.9 | retire |
| F-L-d4mix-c | 6 of 10 | cycle 60 | 51–56 | 898.6 | 1734.4 | retire |
| T-L-Le-merge-d2jit-c | 6 of 10 | cycle 60 | 51–56 | 799.2 | 1644.6 | retire |
| T-N-Nc-merge-d2jit-c | 6 of 10 | cycle 60 | 51–56 | 571.4 | 1119.2 | retire |
| T-L-Ne-pre-c | 4 of 10 | cycle 62 | 53–56 | 943.6 | 1588.4 | retire |
| T-N-NNs-merge-d2jit-c | 4 of 10 | cycle 62 | 53–56 | 560.2 | 1105.0 | freeze 2 at 1100 |
| T-L-Le-pre-c | 1 of 10 | cycle 65 | 56–56 | 881.1 | 1548.5 | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 900 T-L-Le-fresh-c-v742-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1500 F-P-c-v2808-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d2jit-c | frozen | 3943 |  |  | 808 | 11 |  | 1792 | v3348 | 670 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1363 | 9 |  | 1467 | v1061 | 1226 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1668 | 10 |  | 1711 | v300 | 1531 | frozen |  |  | 373 |
| F-P-c | frozen | 2808 |  |  | 1493 | 19 |  | 1571 | v2138 | 1356 | frozen |  |  | 2808 |
| F-P-tau0-c | dormant | 1963 |  |  | 1526 | 32 |  | 1569 (1554) | v1962 (v862) | 1389 | dormant (-28 ± 28 vs peak v862 before dormancy at v1957, at 1417 vs refs; needs > +28) | 750 (63755) | +413 -371 =216, 52.1%, +15 ± 19: kept | 1964 |
| F-P-tau0-d2jit-c | qualified | 2637 | 151 x 40 | 241 | 1772 | 62 | 1739 | 1794 | v2036 | 1634 | ok (-22 ± 68 vs peak v2036; predecessor +33) | 750 (65926) | +319 -331 =350, 49.4%, -4 ± 17: reverted | 2637 |
| T-L-Ne-fresh-c | retired | 1043 |  |  | 1608 | 55 | 1599 | 1599 (1591) | v1042 (v633) | 1471 | retired (avg ply 1 951.8, ply 2 1588.3; no untaken milestone within 10); dormant (+18 ± 37 vs peak v633 before dormancy at v1038, at 1453 vs refs; needs > +37) |  |  | 1043 |
| T-L-Ne-pre-c | dormant | 827 |  |  | 1605 | 39 |  | 1616 (1616) | v758 (v758) | 1468 | dormant (-11 ± 33 vs peak v758 before dormancy at v824, at 1479 vs refs; needs > +33) | 750 (63180) | +395 -429 =176, 48.3%, -12 ± 20: reverted | 827 |
| T-L-Nc-fresh-c | qualified | 1667 | 37 x 1000 | 239 | 1558 | 54 | 1579 | 1588 | v1556 | 1421 | ok (-30 ± 58 vs peak v1556; predecessor -21) | 750 (62268) | +386 -415 =199, 48.5%, -10 ± 19: reverted | 1667 |
| D-L-Nc-c | qualified | 1328 | 40 x 1000 | 242 | 1583 | 54 | 1599 | 1599 | v1287 | 1446 | ok (-16 ± 58 vs peak v1287; predecessor -16) | 750 (64680) | +483 -339 =178, 57.2%, +50 ± 20: kept | 1329 |
| T-L-LLs-c | frozen | 651 |  |  | 1576 | 8 |  | 1624 | v540 | 1439 | frozen |  |  | 651 |
| T-L-NNs-c | dormant | 2279 |  |  | 1550 | 54 | 1628 | 1628 (1605) | v2278 (v1590) | 1413 | dormant (-54 ± 36 vs peak v1590 before dormancy at v2274, at 1467 vs refs; needs > +36) | 750 (66585) | +439 -385 =176, 52.7%, +19 ± 20: kept | 2280 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 951 | 8 |  | 1697 | v818 | 813 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | frozen | 742 |  |  | 898 | 15 |  | 1635 | v682 | 761 | frozen |  |  | 742 |
| T-L-Le-pre-c | dormant | 936 |  |  | 1548 | 54 | 1497 | 1560 (1560) | v739 (v739) | 1411 | dormant (-12 ± 37 vs peak v739 before dormancy at v935, at 1423 vs refs; needs > +37) | 750 (67634) | +393 -457 =150, 46.8%, -22 ± 20: reverted | 936 |
| T-N-Ne-c | frozen | 637 |  |  | 1252 | 8 |  | 1363 | v574 | 1114 | frozen |  |  | 637 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 707 | 9 |  | 1339 | v270 | 569 | frozen |  |  | 417 |
| T-N-NNs-merge-d2jit-c | dormant | 3061 |  |  | 1089 | 38 |  | 1204 (1204) | v2345 (v2345) | 951 | dormant (-116 ± 29 vs peak v2345 before dormancy at v3060, at 1067 vs refs; needs > +29) | 750 (66614) | +396 -407 =197, 49.5%, -4 ± 19: reverted | 3061 |
| T-L-Le-merge-d2jit-c | dormant | 1593 |  |  | 1635 | 24 |  | 1768 (1768) | v1404 (v1404) | 1498 | dormant (-133 ± 33 vs peak v1404 before dormancy at v1592, at 1630 vs refs; needs > +33) | 750 (64230) | +291 -479 =230, 40.6%, -66 ± 19: reverted | 1593 |
| T-L-NLs-d2jit-c | qualified | 1331 | 58 x 40 | 239 | 1606 | 55 | 1501 | 1533 | v1081 | 1469 | new peak (+73 ± 57 over v1081; predecessor +106) | 750 (68092) | +407 -337 =256, 53.5%, +24 ± 19: kept | 1332 |
| T-N-Nc-merge-d2jit-c | dormant | 1437 |  |  | 1108 | 53 | 1130 | 1196 (1196) | v1263 (v1263) | 971 | dormant (-87 ± 36 vs peak v1263 before dormancy at v1433, at 1058 vs refs; needs > +36) | 750 (69351) | +411 -366 =223, 52.2%, +16 ± 19: kept | 1438 |
| T-L-NLs-merge-d2jit-c | qualified | 1552 | 82 x 40 | 244 | 1650 | 56 | 1634 | 1634 | v1469 | 1512 | new peak (+16 ± 59 over v1469; predecessor +16) | 750 (67846) | +398 -397 =205, 50.0%, +0 ± 19: kept | 1553 |
| F-L-d4mix-c | dormant | 321 |  |  | 1738 | 36 |  | 1747 (1739) | v320 (v289) | 1600 | dormant (-1 ± 33 vs peak v289 before dormancy at v319, at 1602 vs refs; needs > +33) | 750 (63560) | +412 -385 =203, 51.3%, +9 ± 19: kept | 322 |
| MoE-d2jit-c | qualified | 1856 | 40 x 40 | 244 | 1677 | 57 | 1699 | 1714 | v1773 | 1539 | ok (-38 ± 62 vs peak v1773; predecessor -22) | 750 (68040) | +403 -387 =210, 50.8%, +6 ± 19: kept | 1857 |
