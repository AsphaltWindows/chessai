# Cycle 61

**Own-games step, pooled over the lineages: 10 gates, +4018 -3819 =2163, score 51.0%, Elo +6.9 ± 6.0; kept 7 of 10.**

All cycles so far: 1636 gates, +714693 -641345 =279962, score 52.2%, Elo +15.6 ± 0.5; kept 1080 of 1636.

2026-11-07 19:02: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10.
League: retired 14 players to retired.txt (their games stay in the fit), added 14.
Tournament: 5 double round-robins of the 34 active players (5610 games) in 7.1 min, 5610 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| T-L-Ne-pre-c | 9 of 10 | cycle 62 | 53–61 | 939.0 | 1602.0 | retire |
| T-N-NNs-merge-d2jit-c | 9 of 10 | cycle 62 | 53–61 | 553.9 | 1114.2 | retire |
| T-L-Le-pre-c | 6 of 10 | cycle 65 | 56–61 | 885.5 | 1530.2 | retire |
| T-L-NLs-d2jit-c | 3 of 10 | cycle 68 | 59–61 | 839.0 | 1594.7 | retire |
| D-L-Nc-c | 0 of 10 | cycle 71 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 900 T-L-Le-fresh-c-v742-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1500 F-P-c-v2808-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d2jit-c | frozen | 3943 |  |  | 805 | 10 |  | 1791 | v1651 | 667 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1361 | 9 |  | 1468 | v1061 | 1223 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1664 | 9 |  | 1712 | v300 | 1525 | frozen |  |  | 373 |
| F-P-c | frozen | 2808 |  |  | 1485 | 16 |  | 1569 | v2138 | 1346 | frozen |  |  | 2808 |
| F-P-tau0-d2jit-c | qualified | 3392 | 150 x 40 | 239 | 1721 | 71 | 1706 | 1792 | v2036 | 1583 | ok (-71 ± 73 vs peak v2036; predecessor +15) | 550 (49841) | +332 -302 =366, 51.5%, +10 ± 17: kept | 3393 |
| T-L-Ne-pre-c | dormant | 829 |  |  | 1608 | 36 |  | 1614 (1614) | v758 (v758) | 1469 | dormant (-6 ± 32 vs peak v758 before dormancy at v824, at 1476 vs refs; needs > +32) | 550 (47930) | +423 -389 =188, 51.7%, +12 ± 19: kept | 830 |
| T-L-Nc-fresh-c | qualified | 1854 | 36 x 1000 | 239 | 1522 | 62 | 1554 | 1586 | v1556 | 1384 | strike 1 (-64 ± 63 vs peak v1556; predecessor -32) | 550 (48428) | +399 -426 =175, 48.6%, -9 ± 20: reverted | 1854 |
| D-L-Nc-c | dormant | 1541 | 41 x 1000 | 240 | 1547 | 62 | 1559 | 1626 | v1370 | 1409 | dormant (strike 2; -79 ± 64 vs peak v1370; predecessor -11) | 550 (48871) | +459 -409 =132, 52.5%, +17 ± 20: kept | 1542 |
| T-L-LLs-c | frozen | 651 |  |  | 1574 | 8 |  | 1625 | v540 | 1436 | frozen |  |  | 651 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 952 | 8 |  | 1698 | v818 | 814 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | frozen | 742 |  |  | 900 | 13 |  | 1634 | v682 | 761 | frozen |  |  | 742 |
| T-L-Le-pre-c | dormant | 937 |  |  | 1516 | 30 |  | 1558 (1558) | v739 (v739) | 1377 | dormant (-42 ± 29 vs peak v739 before dormancy at v935, at 1420 vs refs; needs > +29) | 550 (48694) | +407 -414 =179, 49.6%, -2 ± 20: reverted | 937 |
| T-N-Ne-c | frozen | 637 |  |  | 1250 | 8 |  | 1364 | v574 | 1111 | frozen |  |  | 637 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 708 | 9 |  | 1340 | v270 | 570 | frozen |  |  | 417 |
| T-N-NNs-merge-d2jit-c | dormant | 3064 |  |  | 1092 | 60 | 1192 | 1203 (1203) | v2345 (v2345) | 954 | dormant (-111 ± 38 vs peak v2345 before dormancy at v3060, at 1065 vs refs; needs > +38) | 550 (48344) | +402 -390 =208, 50.6%, +4 ± 19: kept | 3065 |
| T-L-NLs-d2jit-c | dormant | 1459 |  |  | 1563 | 43 |  | 1631 (1605) | v1458 (v1331) | 1424 | dormant (-42 ± 35 vs peak v1331 before dormancy at v1457, at 1466 vs refs; needs > +35) | 550 (51987) | +413 -381 =206, 51.6%, +11 ± 19: kept | 1460 |
| T-N-Nc-merge-d2jit-c | qualified | 1549 | 107 x 40 | 246 | 1214 | 61 | 1267 | 1267 | v1441 | 1076 | ok (-52 ± 65 vs peak v1441; predecessor -52) | 550 (49359) | +420 -366 =214, 52.7%, +19 ± 19: kept | 1550 |
| T-L-NLs-merge-d2jit-c | qualified | 1980 | 81 x 40 | 241 | 1634 | 66 | 1581 | 1653 | v1716 | 1496 | ok (-19 ± 66 vs peak v1716; predecessor +54) | 550 (47137) | +360 -400 =240, 48.0%, -14 ± 19: reverted | 1980 |
| MoE-d2jit-c | qualified | 2076 | 41 x 40 | 240 | 1623 | 65 | 1646 | 1713 | v1773 | 1485 | strike 1 (-89 ± 67 vs peak v1773; predecessor -22) | 550 (49977) | +403 -342 =255, 53.0%, +21 ± 19: kept | 2077 |
