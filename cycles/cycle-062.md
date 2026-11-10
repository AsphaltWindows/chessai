# Cycle 62

**Own-games step, pooled over the lineages: 8 gates, +3242 -3099 =1659, score 50.9%, Elo +6.2 ± 6.8; kept 4 of 8.**

All cycles so far: 1644 gates, +717935 -644444 =281621, score 52.2%, Elo +15.5 ± 0.5; kept 1084 of 1644.

2026-11-07 19:18: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10.
League: retired 18 players to retired.txt (their games stay in the fit), added 18.
Tournament: 5 double round-robins of the 34 active players (5610 games) in 7.3 min, 5610 games saved.
Freeze/retire rule, after 10 cycles dormant: T-L-Ne-pre-c retired (avg ply 1 938.9, ply 2 1607.1; no untaken milestone within 10); T-N-NNs-merge-d2jit-c retired (avg ply 1 559.3, ply 2 1118.7; no untaken milestone within 10). 4 entries leave the field for retired.txt (their games stay in the fit): 34 players become 30, 5,610 games a cycle become 4,350.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| T-L-Le-pre-c | 7 of 10 | cycle 65 | 56–62 | 885.7 | 1528.4 | retire |
| T-L-NLs-d2jit-c | 4 of 10 | cycle 68 | 59–62 | 842.6 | 1593.0 | retire |
| D-L-Nc-c | 1 of 10 | cycle 71 | 62–62 | 973.0 | 1595.7 | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 900 T-L-Le-fresh-c-v742-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1500 F-P-c-v2808-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d2jit-c | frozen | 3943 |  |  | 804 | 10 |  | 1790 | v1651 | 667 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1359 | 9 |  | 1467 | v1061 | 1221 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1661 | 9 |  | 1711 | v300 | 1523 | frozen |  |  | 373 |
| F-P-c | frozen | 2808 |  |  | 1482 | 16 |  | 1568 | v2138 | 1344 | frozen |  |  | 2808 |
| F-P-tau0-d2jit-c | qualified | 3545 | 152 x 40 | 241 | 1725 | 72 | 1719 | 1790 | v2036 | 1587 | ok (-65 ± 73 vs peak v2036; predecessor +6) | 550 (47593) | +319 -337 =344, 49.1%, -6 ± 18: reverted | 3545 |
| T-L-Ne-pre-c | retired | 830 |  |  | 1653 | 67 | 1606 | 1612 (1612) | v758 (v758) | 1516 | retired (avg ply 1 938.9, ply 2 1607.1; no untaken milestone within 10); dormant (+41 ± 43 vs peak v758 before dormancy at v824, at 1474 vs refs; needs > +43) |  |  | 830 |
| T-L-Nc-fresh-c | qualified | 1890 | 36 x 1000 | 243 | 1546 | 64 | 1520 | 1584 | v1556 | 1408 | ok (-38 ± 64 vs peak v1556; predecessor +26) | 550 (47895) | +399 -424 =177, 48.8%, -9 ± 20: reverted | 1890 |
| D-L-Nc-c | dormant | 1542 |  |  | 1596 | 65 | 1546 | 1624 (1624) | v1370 (v1370) | 1458 | dormant (-29 ± 44 vs peak v1370 before dormancy at v1541, at 1487 vs refs; needs > +44) | 550 (46489) | +477 -374 =149, 55.1%, +36 ± 20: kept | 1543 |
| T-L-LLs-c | frozen | 651 |  |  | 1572 | 8 |  | 1624 | v540 | 1435 | frozen |  |  | 651 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 951 | 8 |  | 1696 | v818 | 813 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | frozen | 742 |  |  | 899 | 13 |  | 1632 | v682 | 761 | frozen |  |  | 742 |
| T-L-Le-pre-c | dormant | 937 |  |  | 1517 | 27 |  | 1556 (1556) | v739 (v739) | 1380 | dormant (-39 ± 28 vs peak v739 before dormancy at v935, at 1418 vs refs; needs > +28) | 550 (50020) | +410 -442 =148, 48.4%, -11 ± 20: reverted | 937 |
| T-N-Ne-c | frozen | 637 |  |  | 1248 | 8 |  | 1363 | v574 | 1110 | frozen |  |  | 637 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 708 | 9 |  | 1339 | v270 | 570 | frozen |  |  | 417 |
| T-N-NNs-merge-d2jit-c | retired | 3065 |  |  | 1159 | 63 | 1090 | 1202 (1202) | v2345 (v2345) | 1021 | retired (avg ply 1 559.3, ply 2 1118.7; no untaken milestone within 10); dormant (-42 ± 39 vs peak v2345 before dormancy at v3060, at 1064 vs refs; needs > +39) |  |  | 3065 |
| T-L-NLs-d2jit-c | dormant | 1460 |  |  | 1588 | 65 | 1561 | 1629 (1602) | v1458 (v1331) | 1450 | dormant (-15 ± 43 vs peak v1331 before dormancy at v1457, at 1465 vs refs; needs > +43) | 550 (48481) | +399 -397 =204, 50.1%, +1 ± 19: kept | 1461 |
| T-N-Nc-merge-d2jit-c | qualified | 1645 | 95 x 40 | 242 | 1216 | 63 | 1213 | 1265 | v1441 | 1079 | ok (-49 ± 66 vs peak v1441; predecessor +4) | 550 (50306) | +366 -394 =240, 48.6%, -10 ± 19: reverted | 1645 |
| T-L-NLs-merge-d2jit-c | qualified | 2044 | 64 x 40 | 241 | 1614 | 66 | 1633 | 1652 | v1716 | 1477 | ok (-37 ± 66 vs peak v1716; predecessor -18) | 550 (47122) | +432 -378 =190, 52.7%, +19 ± 19: kept | 2045 |
| MoE-d2jit-c | qualified | 2114 | 37 x 40 | 240 | 1706 | 70 | 1622 | 1710 | v1773 | 1568 | ok (-4 ± 70 vs peak v1773; predecessor +84) | 550 (48366) | +440 -353 =207, 54.4%, +30 ± 19: kept | 2115 |
