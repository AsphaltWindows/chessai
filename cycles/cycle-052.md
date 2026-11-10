# Cycle 52

**Own-games step, pooled over the lineages: 15 gates, +6011 -5931 =3058, score 50.3%, Elo +1.9 ± 5.0; kept 6 of 15.**

All cycles so far: 1519 gates, +668293 -595126 =255581, score 52.4%, Elo +16.7 ± 0.5; kept 1008 of 1519.

2026-11-07 16:29: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 22 players to retired.txt (their games stay in the fit), added 22.
Tournament: 5 double round-robins of the 48 active players (11280 games) in 5.9 min, 11280 games saved.
Freeze/retire rule, after 10 cycles dormant: F-N-d2jit-c retired (avg ply 1 652.8, ply 2 1239.4; no untaken milestone within 10); T-N-Nc-c retired (avg ply 1 752.9, ply 2 1343.3; no untaken milestone within 10). 4 entries leave the field for retired.txt (their games stay in the fit): 48 players become 44, 11,280 games a cycle become 9,460.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| T-L-Ne-fresh-c | 6 of 10 | cycle 56 | 47–52 | 948.8 | 1585.5 | retire |
| F-P-tau0-c | 4 of 10 | cycle 58 | 49–52 | 1035.5 | 1541.2 | retire |
| T-L-NNs-c | 4 of 10 | cycle 58 | 49–52 | 969.4 | 1594.6 | retire |
| F-L-d4mix-c | 2 of 10 | cycle 60 | 51–52 | 896.1 | 1715.3 | retire |
| T-L-Le-merge-d2jit-c | 2 of 10 | cycle 60 | 51–52 | 812.6 | 1663.6 | retire |
| T-N-Nc-merge-d2jit-c | 2 of 10 | cycle 60 | 51–52 | 588.4 | 1109.0 | freeze 2 at 1100 |
| T-L-Ne-pre-c | 0 of 10 | cycle 62 | - | - | - | retire |
| T-N-NNs-merge-d2jit-c | 0 of 10 | cycle 62 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 900 T-L-Le-fresh-c-v742-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1500 F-P-c-v2808-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d2jit-c | frozen | 3943 |  |  | 814 | 12 |  | 1796 | v3348 | 674 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1366 | 10 |  | 1470 | v1061 | 1226 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1674 | 10 |  | 1714 | v300 | 1534 | frozen |  |  | 373 |
| F-P-c | frozen | 2808 |  |  | 1504 | 26 |  | 1575 | v2138 | 1364 | frozen |  |  | 2808 |
| F-P-tau0-c | dormant | 1961 |  |  | 1552 | 52 | 1531 | 1559 (1559) | v862 (v862) | 1412 | dormant (-7 ± 35 vs peak v862 before dormancy at v1957, at 1419 vs refs; needs > +35) | 830 (68765) | +406 -371 =223, 51.7%, +12 ± 19: kept | 1962 |
| F-P-tau0-d2jit-c | qualified | 2036 | 150 x 40 | 240 | 1803 | 64 | 1708 | 1743 | v1574 | 1664 | new peak (+60 ± 65 over v1574; predecessor +95) | 830 (70948) | +330 -339 =331, 49.5%, -3 ± 18: reverted | 2036 |
| T-L-Ne-fresh-c | dormant | 1041 |  |  | 1584 | 27 |  | 1595 (1595) | v633 (v633) | 1444 | dormant (-12 ± 28 vs peak v633 before dormancy at v1038, at 1456 vs refs; needs > +28) | 830 (70198) | +406 -431 =163, 48.8%, -9 ± 20: reverted | 1041 |
| T-L-Ne-pre-c | dormant | 824 | 32 x 1000 | 244 | 1524 | 51 | 1558 | 1624 | v758 | 1384 | dormant (strike 2; -100 ± 55 vs peak v758; predecessor -35) | 830 (70991) | +442 -384 =174, 52.9%, +20 ± 20: kept | 825 |
| T-L-Nc-fresh-c | qualified | 1519 | 39 x 1000 | 243 | 1527 | 51 | 1551 | 1586 | v1015 | 1388 | strike 1 (-59 ± 52 vs peak v1015; predecessor -24) | 830 (70919) | +391 -399 =210, 49.6%, -3 ± 19: reverted | 1519 |
| D-L-Nc-c | qualified | 1157 | 42 x 1000 | 241 | 1562 | 52 | 1538 | 1569 | v851 | 1422 | ok (-6 ± 52 vs peak v851; predecessor +25) | 830 (70790) | +508 -364 =128, 57.2%, +50 ± 20: kept | 1158 |
| T-L-LLs-c | frozen | 651 |  |  | 1582 | 9 |  | 1627 | v540 | 1442 | frozen |  |  | 651 |
| T-L-NNs-c | dormant | 2276 |  |  | 1596 | 31 |  | 1609 (1609) | v1590 (v1590) | 1456 | dormant (-14 ± 28 vs peak v1590 before dormancy at v2274, at 1470 vs refs; needs > +28) | 830 (70842) | +412 -416 =172, 49.8%, -1 ± 20: reverted | 2276 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 955 | 8 |  | 1699 | v818 | 816 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | frozen | 742 |  |  | 908 | 18 |  | 1640 | v682 | 768 | frozen |  |  | 742 |
| T-L-Le-pre-c | qualified | 839 | 30 x 1000 | 241 | 1527 | 51 | 1515 | 1567 | v739 | 1388 | ok (-40 ± 54 vs peak v739; predecessor +12) | 830 (71331) | +393 -468 =139, 46.2%, -26 ± 20: reverted | 839 |
| T-N-Ne-c | frozen | 637 |  |  | 1256 | 8 |  | 1366 | v574 | 1116 | frozen |  |  | 637 |
| T-N-Nc-c | retired | 1505 |  |  | 1345 | 26 |  | 1386 (1386) | v472 (v472) | 1205 | retired (avg ply 1 752.9, ply 2 1343.3; no untaken milestone within 10); dormant (-41 ± 28 vs peak v472 before dormancy at v1500, at 1246 vs refs; needs > +28) |  |  | 1505 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 711 | 9 |  | 1342 | v270 | 571 | frozen |  |  | 417 |
| F-N-d2jit-c | retired | 2426 |  |  | 1312 | 51 | 1237 | 1285 (1285) | v2053 (v2053) | 1172 | retired (avg ply 1 652.8, ply 2 1239.4; no untaken milestone within 10); dormant (+27 ± 34 vs peak v2053 before dormancy at v2418, at 1145 vs refs; needs > +34) |  |  | 2426 |
| T-N-NNs-merge-d2jit-c | dormant | 3060 | 69 x 40 | 241 | 1103 | 49 | 1112 | 1209 | v2345 | 963 | dormant (strike 2; -106 ± 50 vs peak v2345; predecessor -9) | 830 (73980) | +369 -390 =241, 48.9%, -7 ± 19: reverted | 3060 |
| T-L-Le-merge-d2jit-c | dormant | 1593 |  |  | 1659 | 39 |  | 1773 (1773) | v1404 (v1404) | 1520 | dormant (-114 ± 37 vs peak v1404 before dormancy at v1592, at 1634 vs refs; needs > +37) | 830 (72492) | +258 -512 =230, 37.3%, -90 ± 19: reverted | 1593 |
| T-L-NLs-d2jit-c | qualified | 1081 | 62 x 40 | 242 | 1541 | 51 | 1508 | 1520 | v898 | 1401 | new peak (+21 ± 53 over v898; predecessor +33) | 830 (75255) | +434 -347 =219, 54.4%, +30 ± 19: kept | 1082 |
| T-N-Nc-merge-d2jit-c | dormant | 1434 |  |  | 1117 | 35 |  | 1201 (1201) | v1263 (v1263) | 977 | dormant (-84 ± 30 vs peak v1263 before dormancy at v1433, at 1061 vs refs; needs > +30) | 830 (72695) | +384 -401 =215, 49.1%, -6 ± 19: reverted | 1434 |
| T-L-NLs-merge-d2jit-c | qualified | 1210 | 100 x 40 | 242 | 1633 | 54 | 1621 | 1633 | v1019 | 1493 | ok (-0 ± 57 vs peak v1019; predecessor +12) | 830 (74653) | +441 -345 =214, 54.8%, +34 ± 19: kept | 1211 |
| F-L-d4mix-c | dormant | 319 |  |  | 1716 | 34 |  | 1744 (1744) | v289 (v289) | 1576 | dormant (-28 ± 32 vs peak v289 before dormancy at v319, at 1604 vs refs; needs > +32) | 830 (73138) | +450 -371 =179, 53.9%, +28 ± 20: kept | 320 |
| MoE-d2jit-c | qualified | 1688 | 49 x 40 | 241 | 1694 | 56 | 1685 | 1716 | v1364 | 1554 | ok (-23 ± 58 vs peak v1364; predecessor +9) | 830 (76422) | +387 -393 =220, 49.7%, -2 ± 19: reverted | 1688 |
