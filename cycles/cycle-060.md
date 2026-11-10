# Cycle 60

**Own-games step, pooled over the lineages: 10 gates, +3892 -3967 =2141, score 49.6%, Elo -2.6 ± 6.0; kept 4 of 10.**

All cycles so far: 1626 gates, +710675 -637526 =277799, score 52.2%, Elo +15.6 ± 0.5; kept 1073 of 1626.

2026-11-07 18:45: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 18 players to retired.txt (their games stay in the fit), added 18.
Tournament: 5 double round-robins of the 38 active players (7030 games) in 6.8 min, 7030 games saved.
Freeze/retire rule, after 10 cycles dormant: F-L-d4mix-c retired (avg ply 1 909.4, ply 2 1732.5; no untaken milestone within 10); T-L-Le-merge-d2jit-c retired (avg ply 1 795.9, ply 2 1640.2; no untaken milestone within 10). 4 entries leave the field for retired.txt (their games stay in the fit): 38 players become 34, 7,030 games a cycle become 5,610.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| T-L-Ne-pre-c | 8 of 10 | cycle 62 | 53–60 | 939.8 | 1601.3 | retire |
| T-N-NNs-merge-d2jit-c | 8 of 10 | cycle 62 | 53–60 | 551.7 | 1117.0 | retire |
| T-L-Le-pre-c | 5 of 10 | cycle 65 | 56–60 | 884.4 | 1533.1 | retire |
| T-L-NLs-d2jit-c | 2 of 10 | cycle 68 | 59–60 | 827.0 | 1610.7 | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 900 T-L-Le-fresh-c-v742-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1500 F-P-c-v2808-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d2jit-c | frozen | 3943 |  |  | 805 | 11 |  | 1792 | v1651 | 667 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1361 | 9 |  | 1468 | v1061 | 1223 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1664 | 9 |  | 1712 | v300 | 1526 | frozen |  |  | 373 |
| F-P-c | frozen | 2808 |  |  | 1484 | 16 |  | 1569 | v2138 | 1346 | frozen |  |  | 2808 |
| F-P-tau0-d2jit-c | qualified | 3242 | 151 x 40 | 240 | 1709 | 64 | 1757 | 1792 | v2036 | 1570 | strike 1 (-84 ± 69 vs peak v2036; predecessor -49) | 630 (57234) | +318 -337 =345, 49.0%, -7 ± 17: reverted | 3242 |
| T-L-Ne-pre-c | dormant | 829 |  |  | 1612 | 43 |  | 1614 (1614) | v758 (v758) | 1473 | dormant (-2 ± 34 vs peak v758 before dormancy at v824, at 1475 vs refs; needs > +34) | 630 (54763) | +390 -432 =178, 47.9%, -15 ± 20: reverted | 829 |
| T-L-Nc-fresh-c | qualified | 1818 | 37 x 1000 | 241 | 1557 | 58 | 1583 | 1586 | v1556 | 1418 | ok (-29 ± 60 vs peak v1556; predecessor -26) | 630 (53210) | +383 -430 =187, 47.6%, -16 ± 20: reverted | 1818 |
| D-L-Nc-c | qualified | 1499 | 42 x 1000 | 240 | 1561 | 58 | 1592 | 1626 | v1370 | 1423 | strike 1 (-65 ± 62 vs peak v1370; predecessor -31) | 630 (52414) | +463 -380 =157, 54.1%, +29 ± 20: kept | 1500 |
| T-L-LLs-c | frozen | 651 |  |  | 1574 | 8 |  | 1625 | v540 | 1436 | frozen |  |  | 651 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 952 | 8 |  | 1698 | v818 | 813 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | frozen | 742 |  |  | 899 | 14 |  | 1634 | v682 | 761 | frozen |  |  | 742 |
| T-L-Le-pre-c | dormant | 937 |  |  | 1517 | 34 |  | 1558 (1558) | v739 (v739) | 1378 | dormant (-41 ± 30 vs peak v739 before dormancy at v935, at 1419 vs refs; needs > +30) | 630 (56197) | +371 -454 =175, 45.9%, -29 ± 20: reverted | 937 |
| T-N-Ne-c | frozen | 637 |  |  | 1250 | 8 |  | 1364 | v574 | 1111 | frozen |  |  | 637 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 708 | 9 |  | 1340 | v270 | 569 | frozen |  |  | 417 |
| T-N-NNs-merge-d2jit-c | dormant | 3063 |  |  | 1191 | 59 | 1123 | 1203 (1203) | v2345 (v2345) | 1053 | dormant (-12 ± 37 vs peak v2345 before dormancy at v3060, at 1064 vs refs; needs > +37) | 630 (56140) | +402 -369 =229, 51.6%, +12 ± 19: kept | 3064 |
| T-L-Le-merge-d2jit-c | retired | 1593 |  |  | 1638 | 19 |  | 1766 (1766) | v1404 (v1404) | 1499 | retired (avg ply 1 795.9, ply 2 1640.2; no untaken milestone within 10); dormant (-129 ± 32 vs peak v1404 before dormancy at v1592, at 1628 vs refs; needs > +32) |  |  | 1593 |
| T-L-NLs-d2jit-c | dormant | 1459 |  |  | 1588 | 59 | 1631 | 1631 (1604) | v1458 (v1331) | 1450 | dormant (-16 ± 41 vs peak v1331 before dormancy at v1457, at 1466 vs refs; needs > +41) | 630 (56412) | +372 -409 =219, 48.1%, -13 ± 19: reverted | 1459 |
| T-N-Nc-merge-d2jit-c | qualified | 1441 |  |  | 1266 | 59 | 1227 | 1227 (1195) | v1440 (v1263) | 1128 | re-qualified (+72 ± 39 vs peak v1263 before dormancy at v1433, at 1056 vs refs) | 630 (56173) | +391 -360 =249, 51.5%, +11 ± 19: kept | 1442 |
| T-L-NLs-merge-d2jit-c | qualified | 1898 | 101 x 40 | 240 | 1584 | 59 | 1631 | 1653 | v1716 | 1445 | strike 1 (-70 ± 62 vs peak v1716; predecessor -48) | 630 (57203) | +464 -339 =197, 56.2%, +44 ± 19: kept | 1899 |
| F-L-d4mix-c | retired | 324 |  |  | 1756 | 68 | 1721 | 1745 (1738) | v320 (v289) | 1617 | retired (avg ply 1 909.4, ply 2 1732.5; no untaken milestone within 10); dormant (+18 ± 44 vs peak v289 before dormancy at v319, at 1599 vs refs; needs > +44) |  |  | 324 |
| MoE-d2jit-c | qualified | 2035 | 46 x 40 | 238 | 1648 | 61 | 1655 | 1712 | v1773 | 1510 | ok (-64 ± 65 vs peak v1773; predecessor -7) | 630 (54797) | +338 -457 =205, 44.0%, -42 ± 19: reverted | 2035 |
