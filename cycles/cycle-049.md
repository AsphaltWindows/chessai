# Cycle 49

**Own-games step, pooled over the lineages: 18 gates, +7454 -7051 =3495, score 51.1%, Elo +7.8 ± 4.6; kept 9 of 18.**

All cycles so far: 1470 gates, +648406 -575798 =245796, score 52.5%, Elo +17.2 ± 0.5; kept 981 of 1470.

2026-11-07 15:31: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 38 players to retired.txt (their games stay in the fit), added 38.
Tournament: 5 double round-robins of the 53 active players (13780 games) in 5.7 min, 13780 games saved.
Freeze/retire rule, after 10 cycles dormant: T-L-Ne-c retired (avg ply 1 1011.1, ply 2 1570.2; no untaken milestone within 10); T-N-Nc-merge-c retired (avg ply 1 755.4, ply 2 1392.6; no untaken milestone within 10). 4 entries leave the field for retired.txt (their games stay in the fit): 53 players become 49, 13,780 games a cycle become 11,760.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| F-P-c | 9 of 10 | cycle 50 | 41–49 | 926.4 | 1490.4 | freeze 2 at 1500 |
| F-N-d2jit-c | 7 of 10 | cycle 52 | 43–49 | 656.1 | 1228.5 | retire |
| T-N-Nc-c | 7 of 10 | cycle 52 | 43–49 | 749.2 | 1344.5 | retire |
| T-L-Ne-pre-c | 6 of 10 | cycle 53 | 44–49 | 934.8 | 1579.3 | retire |
| T-L-Ne-fresh-c | 3 of 10 | cycle 56 | 47–49 | 937.0 | 1582.9 | retire |
| F-P-tau0-c | 1 of 10 | cycle 58 | 49–49 | 1055.6 | 1529.7 | retire |
| T-L-NNs-c | 1 of 10 | cycle 58 | 49–49 | 985.7 | 1556.4 | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 900 T-L-Le-fresh-c-v742-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d2jit-c | frozen | 3943 |  |  | 814 | 13 |  | 1800 | v3348 | 674 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1369 | 10 |  | 1471 | v1061 | 1229 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1681 | 10 |  | 1715 | v300 | 1541 | frozen |  |  | 373 |
| F-P-c | dormant | 2808 |  |  | 1506 | 49 | 1462 | 1579 (1579) | v2138 (v2138) | 1367 | dormant (-73 ± 35 vs peak v2138 before dormancy at v2801, at 1439 vs refs; needs > +35) | 930 (91044) | +326 -478 =196, 42.4%, -53 ± 20: reverted | 2808 |
| F-P-tau0-c | dormant | 1958 |  |  | 1530 | 50 | 1493 | 1562 (1562) | v862 (v862) | 1390 | dormant (-33 ± 34 vs peak v862 before dormancy at v1957, at 1423 vs refs; needs > +34) | 930 (75748) | +402 -392 =206, 50.5%, +4 ± 19: kept | 1959 |
| F-P-tau0-d2jit-c | qualified | 1574 | 150 x 40 | 240 | 1747 | 58 | 1725 | 1725 | v1423 | 1607 | new peak (+22 ± 62 over v1423; predecessor +22) | 930 (83874) | +336 -329 =335, 50.3%, +2 ± 18: kept | 1575 |
| T-L-Ne-c | retired | 1428 |  |  | 1598 | 36 |  | 1610 (1610) | v1288 (v1288) | 1458 | retired (avg ply 1 1011.1, ply 2 1570.2; no untaken milestone within 10); dormant (-12 ± 31 vs peak v1288 before dormancy at v1425, at 1470 vs refs; needs > +31) |  |  | 1428 |
| T-L-Ne-fresh-c | dormant | 1041 |  |  | 1605 | 51 | 1571 | 1599 (1599) | v633 (v633) | 1465 | dormant (+6 ± 36 vs peak v633 before dormancy at v1038, at 1459 vs refs; needs > +36) | 930 (76922) | +395 -413 =192, 49.1%, -6 ± 19: reverted | 1041 |
| T-L-Ne-pre-c | dormant | 757 |  |  | 1579 | 51 | 1603 | 1603 (1574) | v756 (v691) | 1439 | dormant (+5 ± 34 vs peak v691 before dormancy at v752, at 1434 vs refs; needs > +34) | 930 (82287) | +427 -397 =176, 51.5%, +10 ± 20: kept | 758 |
| T-L-Nc-fresh-c | qualified | 1403 | 37 x 1000 | 243 | 1545 | 50 | 1555 | 1590 | v1015 | 1405 | ok (-44 ± 52 vs peak v1015; predecessor -9) | 930 (78318) | +366 -435 =199, 46.6%, -24 ± 19: reverted | 1403 |
| D-L-Nc-c | qualified | 1022 | 40 x 1000 | 241 | 1527 | 50 | 1569 | 1572 | v851 | 1387 | ok (-46 ± 51 vs peak v851; predecessor -42) | 930 (81020) | +460 -402 =138, 52.9%, +20 ± 20: kept | 1023 |
| T-L-LLs-c | frozen | 651 |  |  | 1588 | 9 |  | 1628 | v540 | 1448 | frozen |  |  | 651 |
| T-L-NNs-c | dormant | 2275 |  |  | 1556 | 50 | 1536 | 1613 (1613) | v1590 (v1590) | 1416 | dormant (-57 ± 34 vs peak v1590 before dormancy at v2274, at 1473 vs refs; needs > +34) | 930 (79934) | +442 -409 =149, 51.6%, +12 ± 20: kept | 2276 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 958 | 9 |  | 1701 | v818 | 818 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | frozen | 742 |  |  | 914 | 23 |  | 1644 | v682 | 774 | frozen |  |  | 742 |
| T-L-Le-pre-c | qualified | 739 |  |  | 1571 | 50 | 1490 | 1523 (1523) | v670 (v670) | 1431 | re-qualified (+48 ± 34 vs peak v670 before dormancy at v735, at 1383 vs refs) | 930 (80800) | +373 -475 =152, 44.9%, -36 ± 20: reverted | 739 |
| T-N-Ne-c | frozen | 637 |  |  | 1260 | 8 |  | 1367 | v574 | 1120 | frozen |  |  | 637 |
| T-N-Nc-c | dormant | 1505 |  |  | 1324 | 48 | 1354 | 1387 (1387) | v472 (v472) | 1184 | dormant (-63 ± 35 vs peak v472 before dormancy at v1500, at 1247 vs refs; needs > +35) | 930 (75789) | +416 -437 =147, 48.9%, -7 ± 20: reverted | 1505 |
| T-N-Nc-merge-c | retired | 2130 |  |  | 1410 | 49 | 1394 | 1455 (1445) | v2127 (v631) | 1270 | retired (avg ply 1 755.4, ply 2 1392.6; no untaken milestone within 10); dormant (-34 ± 29 vs peak v631 before dormancy at v2126, at 1305 vs refs; needs > +29) |  |  | 2130 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 712 | 10 |  | 1343 | v270 | 572 | frozen |  |  | 417 |
| F-N-d2jit-c | dormant | 2424 |  |  | 1245 | 48 | 1269 | 1288 (1288) | v2053 (v2053) | 1105 | dormant (-43 ± 33 vs peak v2053 before dormancy at v2418, at 1148 vs refs; needs > +33) | 930 (84031) | +378 -408 =214, 48.5%, -10 ± 19: reverted | 2424 |
| T-N-NNs-merge-d2jit-c | qualified | 2825 | 75 x 40 | 241 | 1138 | 47 | 1182 | 1212 | v2345 | 998 | strike 1 (-74 ± 49 vs peak v2345; predecessor -44) | 930 (84534) | +390 -398 =212, 49.6%, -3 ± 19: reverted | 2825 |
| T-L-Le-merge-d2jit-c | qualified | 1492 | 87 x 40 | 240 | 1658 | 53 | 1778 | 1778 | v1404 | 1518 | strike 1 (-120 ± 61 vs peak v1404; predecessor -120) | 930 (85022) | +456 -333 =211, 56.1%, +43 ± 19: kept | 1493 |
| T-L-NLs-d2jit-c | qualified | 898 | 56 x 40 | 238 | 1524 | 50 | 1496 | 1496 | v841 | 1384 | new peak (+28 ± 53 over v841; predecessor +28) | 930 (91124) | +465 -314 =221, 57.6%, +53 ± 19: kept | 899 |
| T-N-Nc-merge-d2jit-c | qualified | 1356 | 93 x 40 | 250 | 1139 | 47 | 1204 | 1204 | v1263 | 999 | strike 1 (-65 ± 51 vs peak v1263; predecessor -65) | 930 (84997) | +403 -410 =187, 49.6%, -2 ± 19: reverted | 1356 |
| T-L-NLs-merge-d2jit-c | qualified | 944 | 91 x 40 | 240 | 1479 | 49 | 1568 | 1614 | v676 | 1339 | strike 1 (-135 ± 52 vs peak v676; predecessor -89) | 930 (89598) | +643 -233 =124, 70.5%, +151 ± 22: kept | 945 |
| F-L-d4mix-c | qualified | 311 | 7 x 10 | 232 | 1687 | 55 | 1715 | 1748 | v289 | 1547 | strike 1 (-61 ± 59 vs peak v289; predecessor -28) | 930 (79855) | +411 -396 =193, 50.7%, +5 ± 19: kept | 312 |
| MoE-d2jit-c | qualified | 1546 | 44 x 40 | 242 | 1718 | 56 | 1673 | 1720 | v1364 | 1578 | ok (-2 ± 58 vs peak v1364; predecessor +46) | 930 (84139) | +365 -392 =243, 48.6%, -9 ± 19: reverted | 1546 |
