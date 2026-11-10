# Cycle 53

**Own-games step, pooled over the lineages: 15 gates, +5973 -6102 =2925, score 49.6%, Elo -3.0 ± 5.0; kept 10 of 15.**

All cycles so far: 1534 gates, +674266 -601228 =258506, score 52.4%, Elo +16.6 ± 0.5; kept 1018 of 1534.

2026-11-07 16:46: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 20 players to retired.txt (their games stay in the fit), added 20.
Tournament: 5 double round-robins of the 44 active players (9460 games) in 6.2 min, 9460 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| T-L-Ne-fresh-c | 7 of 10 | cycle 56 | 47–53 | 949.8 | 1584.3 | retire |
| F-P-tau0-c | 5 of 10 | cycle 58 | 49–53 | 1020.1 | 1547.8 | retire |
| T-L-NNs-c | 5 of 10 | cycle 58 | 49–53 | 968.5 | 1594.0 | retire |
| F-L-d4mix-c | 3 of 10 | cycle 60 | 51–53 | 902.8 | 1727.6 | retire |
| T-L-Le-merge-d2jit-c | 3 of 10 | cycle 60 | 51–53 | 807.7 | 1655.8 | retire |
| T-N-Nc-merge-d2jit-c | 3 of 10 | cycle 60 | 51–53 | 584.2 | 1113.7 | retire |
| T-L-Ne-pre-c | 1 of 10 | cycle 62 | 53–53 | 948.1 | 1551.5 | retire |
| T-N-NNs-merge-d2jit-c | 1 of 10 | cycle 62 | 53–53 | 584.7 | 1121.7 | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 900 T-L-Le-fresh-c-v742-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1500 F-P-c-v2808-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d2jit-c | frozen | 3943 |  |  | 812 | 12 |  | 1795 | v3348 | 673 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1365 | 10 |  | 1470 | v1061 | 1225 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1672 | 10 |  | 1714 | v300 | 1533 | frozen |  |  | 373 |
| F-P-c | frozen | 2808 |  |  | 1506 | 24 |  | 1574 | v2138 | 1366 | frozen |  |  | 2808 |
| F-P-tau0-c | dormant | 1962 |  |  | 1574 | 54 | 1549 | 1558 (1558) | v862 (v862) | 1435 | dormant (+16 ± 36 vs peak v862 before dormancy at v1957, at 1418 vs refs; needs > +36) | 750 (60350) | +393 -393 =214, 50.0%, -0 ± 19: kept | 1963 |
| F-P-tau0-d2jit-c | qualified | 2187 | 151 x 40 | 240 | 1757 | 62 | 1800 | 1800 | v2036 | 1618 | ok (-43 ± 67 vs peak v2036; predecessor -43) | 750 (65019) | +327 -320 =353, 50.3%, +2 ± 17: kept | 2188 |
| T-L-Ne-fresh-c | dormant | 1041 |  |  | 1577 | 24 |  | 1594 (1594) | v633 (v633) | 1438 | dormant (-17 ± 27 vs peak v633 before dormancy at v1038, at 1455 vs refs; needs > +27) | 750 (63875) | +440 -405 =155, 51.7%, +12 ± 20: kept | 1042 |
| T-L-Ne-pre-c | dormant | 825 |  |  | 1552 | 54 | 1521 | 1622 (1622) | v758 (v758) | 1412 | dormant (-70 ± 38 vs peak v758 before dormancy at v824, at 1482 vs refs; needs > +38) | 750 (64203) | +422 -388 =190, 51.7%, +12 ± 19: kept | 826 |
| T-L-Nc-fresh-c | qualified | 1556 | 37 x 1000 | 241 | 1593 | 54 | 1524 | 1585 | v1015 | 1454 | new peak (+8 ± 54 over v1015; predecessor +69) | 750 (64022) | +388 -446 =166, 47.1%, -20 ± 20: reverted | 1556 |
| D-L-Nc-c | qualified | 1203 | 45 x 1000 | 242 | 1597 | 54 | 1559 | 1568 | v851 | 1458 | new peak (+29 ± 54 over v851; predecessor +38) | 750 (65156) | +492 -361 =147, 56.5%, +46 ± 20: kept | 1204 |
| T-L-LLs-c | frozen | 651 |  |  | 1581 | 9 |  | 1626 | v540 | 1442 | frozen |  |  | 651 |
| T-L-NNs-c | dormant | 2276 |  |  | 1592 | 27 |  | 1608 (1608) | v1590 (v1590) | 1452 | dormant (-16 ± 27 vs peak v1590 before dormancy at v2274, at 1469 vs refs; needs > +27) | 750 (63877) | +447 -402 =151, 52.2%, +16 ± 20: kept | 2277 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 955 | 8 |  | 1699 | v818 | 815 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | frozen | 742 |  |  | 907 | 17 |  | 1638 | v682 | 768 | frozen |  |  | 742 |
| T-L-Le-pre-c | qualified | 870 | 31 x 1000 | 240 | 1550 | 54 | 1524 | 1566 | v739 | 1410 | ok (-16 ± 55 vs peak v739; predecessor +25) | 750 (63845) | +374 -493 =133, 44.0%, -42 ± 20: reverted | 870 |
| T-N-Ne-c | frozen | 637 |  |  | 1254 | 8 |  | 1366 | v574 | 1115 | frozen |  |  | 637 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 710 | 9 |  | 1341 | v270 | 570 | frozen |  |  | 417 |
| T-N-NNs-merge-d2jit-c | dormant | 3060 |  |  | 1122 | 36 |  | 1208 (1208) | v2345 (v2345) | 982 | dormant (-86 ± 29 vs peak v2345 before dormancy at v3060, at 1068 vs refs; needs > +29) | 750 (65771) | +382 -431 =187, 47.5%, -17 ± 20: reverted | 3060 |
| T-L-Le-merge-d2jit-c | dormant | 1593 |  |  | 1640 | 32 |  | 1772 (1772) | v1404 (v1404) | 1501 | dormant (-132 ± 35 vs peak v1404 before dormancy at v1592, at 1633 vs refs; needs > +35) | 750 (64613) | +238 -567 =195, 33.6%, -119 ± 20: reverted | 1593 |
| T-L-NLs-d2jit-c | qualified | 1149 | 67 x 40 | 242 | 1492 | 53 | 1538 | 1538 | v1081 | 1352 | ok (-47 ± 56 vs peak v1081; predecessor -47) | 750 (70280) | +446 -322 =232, 56.2%, +43 ± 19: kept | 1150 |
| T-N-Nc-merge-d2jit-c | dormant | 1434 |  |  | 1123 | 30 |  | 1200 (1200) | v1263 (v1263) | 984 | dormant (-77 ± 28 vs peak v1263 before dormancy at v1433, at 1060 vs refs; needs > +28) | 750 (64264) | +396 -383 =221, 50.6%, +4 ± 19: kept | 1435 |
| T-L-NLs-merge-d2jit-c | qualified | 1294 | 83 x 40 | 246 | 1610 | 54 | 1630 | 1631 | v1019 | 1471 | ok (-21 ± 58 vs peak v1019; predecessor -20) | 750 (66889) | +378 -447 =175, 46.6%, -24 ± 20: reverted | 1294 |
| F-L-d4mix-c | dormant | 320 |  |  | 1752 | 61 | 1713 | 1743 (1743) | v289 (v289) | 1613 | dormant (+9 ± 41 vs peak v289 before dormancy at v319, at 1603 vs refs; needs > +41) | 750 (63189) | +406 -400 =194, 50.3%, +2 ± 19: kept | 321 |
| MoE-d2jit-c | qualified | 1731 | 43 x 40 | 238 | 1646 | 56 | 1690 | 1715 | v1364 | 1506 | strike 1 (-70 ± 58 vs peak v1364; predecessor -45) | 750 (66400) | +444 -344 =212, 55.0%, +35 ± 19: kept | 1732 |
