# Cycle 51

**Own-games step, pooled over the lineages: 17 gates, +6874 -6744 =3382, score 50.4%, Elo +2.7 ± 4.7; kept 9 of 17.**

All cycles so far: 1504 gates, +662282 -589195 =252523, score 52.4%, Elo +16.9 ± 0.5; kept 1002 of 1504.

2026-11-07 16:10: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 26 players to retired.txt (their games stay in the fit), added 26.
Tournament: 5 double round-robins of the 48 active players (11280 games) in 6.0 min, 11280 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| F-N-d2jit-c | 9 of 10 | cycle 52 | 43–51 | 654.3 | 1231.4 | retire |
| T-N-Nc-c | 9 of 10 | cycle 52 | 43–51 | 752.1 | 1343.1 | retire |
| T-L-Ne-fresh-c | 5 of 10 | cycle 56 | 47–51 | 946.1 | 1585.9 | retire |
| F-P-tau0-c | 3 of 10 | cycle 58 | 49–51 | 1042.0 | 1537.8 | retire |
| T-L-NNs-c | 3 of 10 | cycle 58 | 49–51 | 969.6 | 1594.3 | retire |
| F-L-d4mix-c | 1 of 10 | cycle 60 | 51–51 | 896.6 | 1714.9 | retire |
| T-L-Le-merge-d2jit-c | 1 of 10 | cycle 60 | 51–51 | 816.8 | 1667.8 | retire |
| T-N-Nc-merge-d2jit-c | 1 of 10 | cycle 60 | 51–51 | 591.7 | 1101.2 | freeze 2 at 1100 |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 900 T-L-Le-fresh-c-v742-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1500 F-P-c-v2808-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d2jit-c | frozen | 3943 |  |  | 814 | 12 |  | 1797 | v3348 | 674 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1366 | 10 |  | 1470 | v1061 | 1226 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1676 | 10 |  | 1714 | v300 | 1536 | frozen |  |  | 373 |
| F-P-c | frozen | 2808 |  |  | 1506 | 30 |  | 1576 | v2138 | 1367 | frozen |  |  | 2808 |
| F-P-tau0-c | dormant | 1960 |  |  | 1531 | 52 | 1552 | 1560 (1560) | v862 (v862) | 1392 | dormant (-29 ± 35 vs peak v862 before dormancy at v1957, at 1420 vs refs; needs > +35) | 830 (70582) | +413 -365 =222, 52.4%, +17 ± 19: kept | 1961 |
| F-P-tau0-d2jit-c | qualified | 1886 | 157 x 40 | 241 | 1709 | 57 | 1692 | 1744 | v1574 | 1569 | ok (-35 ± 61 vs peak v1574; predecessor +17) | 830 (74257) | +324 -327 =349, 49.9%, -1 ± 17: reverted | 1886 |
| T-L-Ne-fresh-c | dormant | 1041 |  |  | 1584 | 31 |  | 1596 (1596) | v633 (v633) | 1444 | dormant (-13 ± 29 vs peak v633 before dormancy at v1038, at 1457 vs refs; needs > +29) | 830 (70775) | +418 -425 =157, 49.6%, -2 ± 20: reverted | 1041 |
| T-L-Ne-pre-c | qualified | 791 | 32 x 1000 | 238 | 1559 | 52 | 1624 | 1624 | v758 | 1419 | strike 1 (-66 ± 56 vs peak v758; predecessor -66) | 830 (72978) | +451 -376 =173, 53.8%, +26 ± 20: kept | 792 |
| T-L-Nc-fresh-c | qualified | 1480 | 39 x 1000 | 237 | 1552 | 52 | 1560 | 1587 | v1015 | 1412 | ok (-35 ± 53 vs peak v1015; predecessor -8) | 830 (70877) | +383 -437 =180, 47.3%, -19 ± 20: reverted | 1480 |
| D-L-Nc-c | qualified | 1114 | 43 x 1000 | 241 | 1538 | 52 | 1566 | 1570 | v851 | 1398 | ok (-32 ± 52 vs peak v851; predecessor -28) | 830 (69808) | +508 -338 =154, 58.5%, +60 ± 20: kept | 1115 |
| T-L-LLs-c | frozen | 651 |  |  | 1584 | 9 |  | 1627 | v540 | 1445 | frozen |  |  | 651 |
| T-L-NNs-c | dormant | 2276 |  |  | 1601 | 38 |  | 1610 (1610) | v1590 (v1590) | 1462 | dormant (-9 ± 30 vs peak v1590 before dormancy at v2274, at 1471 vs refs; needs > +30) | 830 (71889) | +407 -409 =184, 49.9%, -1 ± 20: reverted | 2276 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 956 | 8 |  | 1700 | v818 | 817 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | frozen | 742 |  |  | 908 | 19 |  | 1641 | v682 | 769 | frozen |  |  | 742 |
| T-L-Le-pre-c | qualified | 808 | 37 x 1000 | 243 | 1516 | 51 | 1516 | 1567 | v739 | 1376 | ok (-52 ± 54 vs peak v739; predecessor -0) | 830 (72847) | +496 -379 =125, 55.9%, +41 ± 20: kept | 809 |
| T-N-Ne-c | frozen | 637 |  |  | 1257 | 8 |  | 1366 | v574 | 1117 | frozen |  |  | 637 |
| T-N-Nc-c | dormant | 1505 |  |  | 1337 | 30 |  | 1386 (1386) | v472 (v472) | 1197 | dormant (-49 ± 28 vs peak v472 before dormancy at v1500, at 1246 vs refs; needs > +28) | 830 (69928) | +403 -431 =166, 48.6%, -10 ± 20: reverted | 1505 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 711 | 10 |  | 1342 | v270 | 571 | frozen |  |  | 417 |
| F-N-d2jit-c | dormant | 2425 |  |  | 1236 | 51 | 1244 | 1285 (1285) | v2053 (v2053) | 1097 | dormant (-49 ± 34 vs peak v2053 before dormancy at v2418, at 1146 vs refs; needs > +34) | 830 (73370) | +407 -395 =198, 50.6%, +4 ± 19: kept | 2426 |
| T-N-NNs-merge-d2jit-c | qualified | 2990 | 82 x 40 | 244 | 1111 | 49 | 1175 | 1209 | v2345 | 971 | strike 1 (-99 ± 50 vs peak v2345; predecessor -64) | 830 (74027) | +384 -383 =233, 50.0%, +0 ± 19: kept | 2991 |
| T-L-Le-merge-d2jit-c | dormant | 1593 |  |  | 1668 | 55 | 1636 | 1774 (1774) | v1404 (v1404) | 1528 | dormant (-106 ± 42 vs peak v1404 before dormancy at v1592, at 1635 vs refs; needs > +42) | 830 (71799) | +232 -577 =191, 32.8%, -125 ± 20: reverted | 1593 |
| T-L-NLs-d2jit-c | qualified | 1018 | 60 x 40 | 241 | 1509 | 51 | 1518 | 1520 | v898 | 1369 | ok (-12 ± 53 vs peak v898; predecessor -9) | 830 (79756) | +397 -387 =216, 50.5%, +4 ± 19: kept | 1019 |
| T-N-Nc-merge-d2jit-c | dormant | 1434 |  |  | 1101 | 49 | 1125 | 1202 (1202) | v1263 (v1263) | 962 | dormant (-100 ± 35 vs peak v1263 before dormancy at v1433, at 1062 vs refs; needs > +35) | 830 (74537) | +379 -402 =219, 48.9%, -8 ± 19: reverted | 1434 |
| T-L-NLs-merge-d2jit-c | qualified | 1109 | 89 x 40 | 239 | 1622 | 54 | 1634 | 1634 | v1019 | 1482 | ok (-12 ± 57 vs peak v1019; predecessor -12) | 830 (74052) | +455 -349 =196, 55.3%, +37 ± 19: kept | 1110 |
| F-L-d4mix-c | dormant | 319 |  |  | 1715 | 41 |  | 1745 (1745) | v289 (v289) | 1575 | dormant (-30 ± 35 vs peak v289 before dormancy at v319, at 1605 vs refs; needs > +35) | 830 (69731) | +392 -425 =183, 48.4%, -12 ± 20: reverted | 319 |
| MoE-d2jit-c | qualified | 1638 | 51 x 40 | 241 | 1686 | 56 | 1688 | 1718 | v1364 | 1546 | ok (-32 ± 58 vs peak v1364; predecessor -2) | 830 (75406) | +425 -339 =236, 54.3%, +30 ± 19: kept | 1639 |
