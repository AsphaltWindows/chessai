# Cycle 57

**Own-games step, pooled over the lineages: 14 gates, +5599 -5452 =2949, score 50.5%, Elo +3.6 ± 5.1; kept 8 of 14.**

All cycles so far: 1592 gates, +697514 -624065 =270421, score 52.3%, Elo +16.0 ± 0.5; kept 1052 of 1592.

2026-11-07 17:56: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 20 players to retired.txt (their games stay in the fit), added 20.
Tournament: 5 double round-robins of the 42 active players (8610 games) in 6.5 min, 8610 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| F-P-tau0-c | 9 of 10 | cycle 58 | 49–57 | 998.8 | 1542.4 | retire |
| T-L-NNs-c | 9 of 10 | cycle 58 | 49–57 | 956.0 | 1590.1 | retire |
| F-L-d4mix-c | 7 of 10 | cycle 60 | 51–57 | 904.0 | 1730.6 | retire |
| T-L-Le-merge-d2jit-c | 7 of 10 | cycle 60 | 51–57 | 797.3 | 1642.6 | retire |
| T-N-Nc-merge-d2jit-c | 7 of 10 | cycle 60 | 51–57 | 569.5 | 1131.9 | retire |
| T-L-Ne-pre-c | 5 of 10 | cycle 62 | 53–57 | 941.1 | 1592.6 | retire |
| T-N-NNs-merge-d2jit-c | 5 of 10 | cycle 62 | 53–57 | 556.6 | 1104.4 | freeze 2 at 1100 |
| T-L-Le-pre-c | 2 of 10 | cycle 65 | 56–57 | 897.1 | 1538.5 | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 900 T-L-Le-fresh-c-v742-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1500 F-P-c-v2808-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d2jit-c | frozen | 3943 |  |  | 805 | 11 |  | 1790 | v1651 | 668 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1361 | 9 |  | 1467 | v1061 | 1224 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1665 | 9 |  | 1710 | v300 | 1528 | frozen |  |  | 373 |
| F-P-c | frozen | 2808 |  |  | 1487 | 18 |  | 1569 | v2138 | 1350 | frozen |  |  | 2808 |
| F-P-tau0-c | dormant | 1964 |  |  | 1527 | 55 | 1524 | 1566 (1552) | v1962 (v862) | 1390 | dormant (-25 ± 36 vs peak v862 before dormancy at v1957, at 1415 vs refs; needs > +36) | 710 (56853) | +384 -387 =229, 49.9%, -1 ± 19: reverted | 1964 |
| F-P-tau0-d2jit-c | qualified | 2789 | 152 x 40 | 240 | 1710 | 60 | 1769 | 1791 | v2036 | 1573 | strike 1 (-81 ± 67 vs peak v2036; predecessor -58) | 710 (62881) | +327 -338 =335, 49.5%, -4 ± 18: reverted | 2789 |
| T-L-Ne-pre-c | dormant | 827 |  |  | 1609 | 33 |  | 1614 (1614) | v758 (v758) | 1472 | dormant (-4 ± 31 vs peak v758 before dormancy at v824, at 1476 vs refs; needs > +31) | 710 (62019) | +411 -411 =178, 50.0%, -0 ± 20: kept | 828 |
| T-L-Nc-fresh-c | qualified | 1703 | 36 x 1000 | 238 | 1582 | 55 | 1555 | 1584 | v1556 | 1445 | ok (-2 ± 59 vs peak v1556; predecessor +27) | 710 (60502) | +420 -409 =171, 50.5%, +4 ± 20: kept | 1704 |
| D-L-Nc-c | qualified | 1370 | 41 x 1000 | 242 | 1623 | 56 | 1580 | 1597 | v1287 | 1485 | new peak (+26 ± 59 over v1287; predecessor +43) | 710 (60722) | +423 -400 =177, 51.1%, +8 ± 20: kept | 1371 |
| T-L-LLs-c | frozen | 651 |  |  | 1573 | 8 |  | 1623 | v540 | 1436 | frozen |  |  | 651 |
| T-L-NNs-c | dormant | 2280 |  |  | 1576 | 55 | 1548 | 1626 (1603) | v2278 (v1590) | 1439 | dormant (-27 ± 36 vs peak v1590 before dormancy at v2274, at 1466 vs refs; needs > +36) | 710 (61873) | +417 -410 =173, 50.3%, +2 ± 20: kept | 2281 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 949 | 8 |  | 1696 | v818 | 812 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | frozen | 742 |  |  | 898 | 15 |  | 1633 | v682 | 761 | frozen |  |  | 742 |
| T-L-Le-pre-c | dormant | 936 |  |  | 1529 | 39 |  | 1558 (1558) | v739 (v739) | 1391 | dormant (-29 ± 32 vs peak v739 before dormancy at v935, at 1420 vs refs; needs > +32) | 710 (61243) | +467 -394 =139, 53.6%, +25 ± 20: kept | 937 |
| T-N-Ne-c | frozen | 637 |  |  | 1249 | 8 |  | 1363 | v574 | 1112 | frozen |  |  | 637 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 706 | 9 |  | 1338 | v270 | 569 | frozen |  |  | 417 |
| T-N-NNs-merge-d2jit-c | dormant | 3061 |  |  | 1102 | 31 |  | 1202 (1202) | v2345 (v2345) | 965 | dormant (-100 ± 27 vs peak v2345 before dormancy at v3060, at 1065 vs refs; needs > +27) | 710 (64426) | +386 -401 =213, 49.2%, -5 ± 19: reverted | 3061 |
| T-L-Le-merge-d2jit-c | dormant | 1593 |  |  | 1630 | 22 |  | 1766 (1766) | v1404 (v1404) | 1493 | dormant (-135 ± 33 vs peak v1404 before dormancy at v1592, at 1629 vs refs; needs > +33) | 710 (62714) | +287 -521 =192, 38.3%, -83 ± 20: reverted | 1593 |
| T-L-NLs-d2jit-c | qualified | 1392 | 60 x 40 | 242 | 1525 | 55 | 1603 | 1603 | v1331 | 1388 | strike 1 (-78 ± 58 vs peak v1331; predecessor -78) | 710 (64403) | +400 -330 =270, 53.5%, +24 ± 18: kept | 1393 |
| T-N-Nc-merge-d2jit-c | dormant | 1438 |  |  | 1208 | 55 | 1110 | 1194 (1194) | v1263 (v1263) | 1071 | dormant (+14 ± 37 vs peak v1263 before dormancy at v1433, at 1057 vs refs; needs > +37) | 710 (61967) | +445 -354 =201, 54.5%, +32 ± 19: kept | 1439 |
| T-L-NLs-merge-d2jit-c | qualified | 1625 | 72 x 40 | 242 | 1534 | 55 | 1647 | 1647 | v1552 | 1397 | strike 1 (-112 ± 59 vs peak v1552; predecessor -112) | 710 (63976) | +480 -294 =226, 59.3%, +65 ± 19: kept | 1626 |
| F-L-d4mix-c | dormant | 322 |  |  | 1708 | 60 | 1735 | 1744 (1737) | v320 (v289) | 1571 | dormant (-29 ± 41 vs peak v289 before dormancy at v319, at 1600 vs refs; needs > +41) | 710 (62210) | +384 -429 =187, 47.8%, -16 ± 20: reverted | 322 |
| MoE-d2jit-c | qualified | 1899 | 42 x 40 | 242 | 1658 | 58 | 1673 | 1711 | v1773 | 1521 | ok (-53 ± 63 vs peak v1773; predecessor -15) | 710 (65332) | +368 -374 =258, 49.7%, -2 ± 19: reverted | 1899 |
