# Cycle 58

**Own-games step, pooled over the lineages: 12 gates, +4653 -4712 =2635, score 49.8%, Elo -1.7 ± 5.5; kept 9 of 12.**

All cycles so far: 1604 gates, +702167 -628777 =273056, score 52.3%, Elo +15.9 ± 0.5; kept 1061 of 1604.

2026-11-07 18:13: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 20 players to retired.txt (their games stay in the fit), added 20.
Tournament: 5 double round-robins of the 42 active players (8610 games) in 6.4 min, 8610 games saved.
Freeze/retire rule, after 10 cycles dormant: F-P-tau0-c retired (avg ply 1 998.3, ply 2 1541.3; no untaken milestone within 10); T-L-NNs-c retired (avg ply 1 958.8, ply 2 1590.5; no untaken milestone within 10). 4 entries leave the field for retired.txt (their games stay in the fit): 42 players become 38, 8,610 games a cycle become 7,030.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| F-L-d4mix-c | 8 of 10 | cycle 60 | 51–58 | 906.9 | 1730.8 | retire |
| T-L-Le-merge-d2jit-c | 8 of 10 | cycle 60 | 51–58 | 796.6 | 1641.2 | retire |
| T-N-Nc-merge-d2jit-c | 8 of 10 | cycle 60 | 51–58 | 577.1 | 1138.8 | retire |
| T-L-Ne-pre-c | 6 of 10 | cycle 62 | 53–58 | 946.8 | 1596.1 | retire |
| T-N-NNs-merge-d2jit-c | 6 of 10 | cycle 62 | 53–58 | 554.9 | 1103.8 | freeze 2 at 1100 |
| T-L-Le-pre-c | 3 of 10 | cycle 65 | 56–58 | 886.3 | 1540.8 | retire |
| T-L-NLs-d2jit-c | 0 of 10 | cycle 68 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 900 T-L-Le-fresh-c-v742-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1500 F-P-c-v2808-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d2jit-c | frozen | 3943 |  |  | 806 | 11 |  | 1791 | v1651 | 668 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1362 | 9 |  | 1468 | v1061 | 1224 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1665 | 9 |  | 1712 | v300 | 1527 | frozen |  |  | 373 |
| F-P-c | frozen | 2808 |  |  | 1486 | 18 |  | 1569 | v2138 | 1348 | frozen |  |  | 2808 |
| F-P-tau0-c | retired | 1964 |  |  | 1532 | 39 |  | 1566 (1553) | v1962 (v862) | 1394 | retired (avg ply 1 998.3, ply 2 1541.3; no untaken milestone within 10); dormant (-21 ± 30 vs peak v862 before dormancy at v1957, at 1415 vs refs; needs > +30) |  |  | 1964 |
| F-P-tau0-d2jit-c | qualified | 2939 | 150 x 40 | 240 | 1731 | 62 | 1714 | 1792 | v2036 | 1593 | ok (-61 ± 67 vs peak v2036; predecessor +17) | 710 (60937) | +337 -337 =326, 50.0%, -0 ± 18: kept | 2940 |
| T-L-Ne-pre-c | dormant | 828 |  |  | 1614 | 56 | 1611 | 1614 (1614) | v758 (v758) | 1476 | dormant (-0 ± 39 vs peak v758 before dormancy at v824, at 1476 vs refs; needs > +39) | 710 (63368) | +421 -377 =202, 52.2%, +15 ± 19: kept | 829 |
| T-L-Nc-fresh-c | qualified | 1742 | 38 x 1000 | 242 | 1549 | 55 | 1585 | 1585 | v1703 | 1411 | ok (-36 ± 59 vs peak v1703; predecessor -36) | 710 (60154) | +415 -396 =189, 50.9%, +7 ± 19: kept | 1743 |
| D-L-Nc-c | qualified | 1414 | 43 x 1000 | 240 | 1579 | 56 | 1626 | 1626 | v1370 | 1441 | ok (-47 ± 60 vs peak v1370; predecessor -47) | 710 (61546) | +435 -406 =159, 51.4%, +10 ± 20: kept | 1415 |
| T-L-LLs-c | frozen | 651 |  |  | 1574 | 8 |  | 1625 | v540 | 1436 | frozen |  |  | 651 |
| T-L-NNs-c | retired | 2281 |  |  | 1594 | 56 | 1579 | 1627 (1604) | v2278 (v1590) | 1455 | retired (avg ply 1 958.8, ply 2 1590.5; no untaken milestone within 10); dormant (-10 ± 37 vs peak v1590 before dormancy at v2274, at 1465 vs refs; needs > +37) |  |  | 2281 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 950 | 8 |  | 1697 | v818 | 812 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | frozen | 742 |  |  | 900 | 14 |  | 1634 | v682 | 762 | frozen |  |  | 742 |
| T-L-Le-pre-c | dormant | 937 |  |  | 1545 | 55 | 1530 | 1558 (1558) | v739 (v739) | 1407 | dormant (-13 ± 38 vs peak v739 before dormancy at v935, at 1420 vs refs; needs > +38) | 710 (60533) | +395 -437 =168, 47.9%, -15 ± 20: reverted | 937 |
| T-N-Ne-c | frozen | 637 |  |  | 1250 | 8 |  | 1364 | v574 | 1112 | frozen |  |  | 637 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 707 | 9 |  | 1340 | v270 | 569 | frozen |  |  | 417 |
| T-N-NNs-merge-d2jit-c | dormant | 3061 |  |  | 1101 | 27 |  | 1203 (1203) | v2345 (v2345) | 963 | dormant (-102 ± 26 vs peak v2345 before dormancy at v3060, at 1065 vs refs; needs > +26) | 710 (62502) | +406 -373 =221, 51.6%, +12 ± 19: kept | 3062 |
| T-L-Le-merge-d2jit-c | dormant | 1593 |  |  | 1632 | 21 |  | 1767 (1767) | v1404 (v1404) | 1494 | dormant (-135 ± 32 vs peak v1404 before dormancy at v1592, at 1629 vs refs; needs > +32) | 710 (59188) | +276 -522 =202, 37.7%, -87 ± 20: reverted | 1593 |
| T-L-NLs-d2jit-c | dormant | 1457 | 64 x 40 | 242 | 1543 | 55 | 1528 | 1604 | v1331 | 1405 | dormant (strike 2; -61 ± 59 vs peak v1331; predecessor +16) | 710 (62796) | +425 -358 =217, 53.3%, +23 ± 19: kept | 1458 |
| T-N-Nc-merge-d2jit-c | dormant | 1439 |  |  | 1187 | 55 | 1210 | 1210 (1195) | v1438 (v1263) | 1049 | dormant (-8 ± 37 vs peak v1263 before dormancy at v1433, at 1057 vs refs; needs > +37) | 710 (63989) | +386 -366 =248, 51.0%, +7 ± 19: kept | 1440 |
| T-L-NLs-merge-d2jit-c | qualified | 1716 | 90 x 40 | 240 | 1656 | 58 | 1538 | 1648 | v1552 | 1518 | new peak (+8 ± 61 over v1552; predecessor +118) | 710 (64562) | +311 -434 =255, 43.9%, -43 ± 19: reverted | 1716 |
| F-L-d4mix-c | dormant | 322 |  |  | 1732 | 44 |  | 1744 (1738) | v320 (v289) | 1594 | dormant (-6 ± 35 vs peak v289 before dormancy at v319, at 1600 vs refs; needs > +35) | 710 (61923) | +447 -353 =200, 54.7%, +33 ± 19: kept | 323 |
| MoE-d2jit-c | qualified | 1942 | 43 x 40 | 240 | 1680 | 59 | 1662 | 1712 | v1773 | 1542 | ok (-32 ± 63 vs peak v1773; predecessor +18) | 710 (63563) | +399 -353 =248, 52.3%, +16 ± 19: kept | 1943 |
