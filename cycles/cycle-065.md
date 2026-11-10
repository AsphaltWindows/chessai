# Cycle 65

**Own-games step, pooled over the lineages: 7 gates, +2811 -2565 =1624, score 51.8%, Elo +12.2 ± 7.1; kept 7 of 7.**

All cycles so far: 1667 gates, +726943 -653265 =286792, score 52.2%, Elo +15.4 ± 0.5; kept 1099 of 1667.

2026-11-07 20:11: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10.
League: retired 8 players to retired.txt (their games stay in the fit), added 8.
Tournament: 5 double round-robins of the 30 active players (4350 games) in 8.4 min, 4350 games saved.
Freeze/retire rule, after 10 cycles dormant: T-L-Le-pre-c retired (avg ply 1 890.6, ply 2 1529.0; no untaken milestone within 10). 2 entries leave the field for retired.txt (their games stay in the fit): 30 players become 28, 4,350 games a cycle become 3,780.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| T-L-NLs-d2jit-c | 7 of 10 | cycle 68 | 59–65 | 848.8 | 1597.9 | retire |
| D-L-Nc-c | 4 of 10 | cycle 71 | 62–65 | 978.6 | 1628.5 | retire |
| F-P-tau0-d2jit-c | 1 of 10 | cycle 74 | 65–65 | 855.9 | 1763.4 | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 900 T-L-Le-fresh-c-v742-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1500 F-P-c-v2808-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d2jit-c | frozen | 3943 |  |  | 802 | 10 |  | 1791 | v1651 | 664 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1359 | 9 |  | 1468 | v1061 | 1221 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1660 | 9 |  | 1711 | v300 | 1522 | frozen |  |  | 373 |
| F-P-c | frozen | 2808 |  |  | 1481 | 15 |  | 1567 | v2138 | 1343 | frozen |  |  | 2808 |
| F-P-tau0-d2jit-c | dormant | 3839 |  |  | 1763 | 56 |  | 1789 (1789) | v2036 (v2036) | 1625 | dormant (-26 ± 43 vs peak v2036 before dormancy at v3839, at 1651 vs refs; needs > +43) | 470 (40449) | +347 -322 =331, 51.2%, +9 ± 18: kept | 3840 |
| T-L-Nc-fresh-c | qualified | 1995 | 35 x 1000 | 242 | 1590 | 68 | 1581 | 1583 | v1556 | 1451 | new peak (+7 ± 66 over v1556; predecessor +9) | 470 (41405) | +429 -394 =177, 51.7%, +12 ± 20: kept | 1996 |
| D-L-Nc-c | dormant | 1543 |  |  | 1628 | 41 |  | 1624 (1624) | v1370 (v1370) | 1489 | dormant (+4 ± 35 vs peak v1370 before dormancy at v1541, at 1485 vs refs; needs > +35) | 470 (39708) | +440 -409 =151, 51.5%, +11 ± 20: kept | 1544 |
| T-L-LLs-c | frozen | 651 |  |  | 1571 | 8 |  | 1624 | v540 | 1433 | frozen |  |  | 651 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 950 | 8 |  | 1697 | v818 | 812 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | frozen | 742 |  |  | 896 | 12 |  | 1631 | v682 | 758 | frozen |  |  | 742 |
| T-L-Le-pre-c | retired | 938 |  |  | 1563 | 48 |  | 1555 (1555) | v739 (v739) | 1425 | retired (avg ply 1 890.6, ply 2 1529.0; no untaken milestone within 10); dormant (+8 ± 35 vs peak v739 before dormancy at v935, at 1417 vs refs; needs > +35) |  |  | 938 |
| T-N-Ne-c | frozen | 637 |  |  | 1247 | 8 |  | 1364 | v574 | 1109 | frozen |  |  | 637 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 708 | 8 |  | 1339 | v270 | 570 | frozen |  |  | 417 |
| T-L-NLs-d2jit-c | dormant | 1461 |  |  | 1635 | 41 |  | 1629 (1602) | v1458 (v1331) | 1496 | dormant (+33 ± 35 vs peak v1331 before dormancy at v1457, at 1463 vs refs; needs > +35) | 470 (42420) | +435 -355 =210, 54.0%, +28 ± 19: kept | 1462 |
| T-N-Nc-merge-d2jit-c | qualified | 1862 | 66 x 40 | 242 | 1212 | 69 | 1218 | 1265 | v1441 | 1074 | ok (-52 ± 69 vs peak v1441; predecessor -5) | 470 (42356) | +389 -383 =228, 50.3%, +2 ± 19: kept | 1863 |
| T-L-NLs-merge-d2jit-c | qualified | 2223 | 61 x 40 | 242 | 1641 | 69 | 1605 | 1651 | v1716 | 1503 | ok (-10 ± 69 vs peak v1716; predecessor +36) | 470 (40913) | +368 -349 =283, 50.9%, +7 ± 18: kept | 2224 |
| MoE-d2jit-c | qualified | 2232 | 42 x 40 | 242 | 1715 | 73 | 1682 | 1764 | v2156 | 1577 | ok (-48 ± 81 vs peak v2156; predecessor +33) | 470 (40350) | +403 -353 =244, 52.5%, +17 ± 19: kept | 2233 |
