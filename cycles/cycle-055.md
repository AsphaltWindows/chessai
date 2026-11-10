# Cycle 55

**Own-games step, pooled over the lineages: 15 gates, +6068 -5886 =3046, score 50.6%, Elo +4.2 ± 5.0; kept 9 of 15.**

All cycles so far: 1564 gates, +686369 -613128 =264503, score 52.3%, Elo +16.3 ± 0.5; kept 1036 of 1564.

2026-11-07 17:22: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 22 players to retired.txt (their games stay in the fit), added 22.
Tournament: 5 double round-robins of the 44 active players (9460 games) in 6.5 min, 9460 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| T-L-Ne-fresh-c | 9 of 10 | cycle 56 | 47–55 | 952.1 | 1586.0 | retire |
| F-P-tau0-c | 7 of 10 | cycle 58 | 49–55 | 1004.9 | 1547.0 | retire |
| T-L-NNs-c | 7 of 10 | cycle 58 | 49–55 | 959.9 | 1597.8 | retire |
| F-L-d4mix-c | 5 of 10 | cycle 60 | 51–55 | 900.0 | 1733.7 | retire |
| T-L-Le-merge-d2jit-c | 5 of 10 | cycle 60 | 51–55 | 800.8 | 1646.5 | retire |
| T-N-Nc-merge-d2jit-c | 5 of 10 | cycle 60 | 51–55 | 570.8 | 1121.3 | retire |
| T-L-Ne-pre-c | 3 of 10 | cycle 62 | 53–55 | 949.0 | 1582.8 | retire |
| T-N-NNs-merge-d2jit-c | 3 of 10 | cycle 62 | 53–55 | 570.7 | 1110.5 | retire |
| T-L-Le-pre-c | 0 of 10 | cycle 65 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 900 T-L-Le-fresh-c-v742-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1500 F-P-c-v2808-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d2jit-c | frozen | 3943 |  |  | 808 | 12 |  | 1792 | v3348 | 671 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1363 | 9 |  | 1467 | v1061 | 1225 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1669 | 10 |  | 1711 | v300 | 1531 | frozen |  |  | 373 |
| F-P-c | frozen | 2808 |  |  | 1500 | 21 |  | 1571 | v2138 | 1362 | frozen |  |  | 2808 |
| F-P-tau0-c | dormant | 1963 |  |  | 1530 | 39 |  | 1570 (1554) | v1962 (v862) | 1392 | dormant (-24 ± 30 vs peak v862 before dormancy at v1957, at 1417 vs refs; needs > +30) | 750 (60709) | +385 -402 =213, 49.1%, -6 ± 19: reverted | 1963 |
| F-P-tau0-d2jit-c | qualified | 2485 | 149 x 40 | 240 | 1743 | 60 | 1714 | 1795 | v2036 | 1606 | ok (-52 ± 67 vs peak v2036; predecessor +29) | 750 (64780) | +341 -322 =337, 50.9%, +7 ± 18: kept | 2486 |
| T-L-Ne-fresh-c | dormant | 1042 |  |  | 1602 | 40 |  | 1591 (1591) | v633 (v633) | 1464 | dormant (+11 ± 31 vs peak v633 before dormancy at v1038, at 1453 vs refs; needs > +31) | 750 (62653) | +431 -412 =157, 50.9%, +7 ± 20: kept | 1043 |
| T-L-Ne-pre-c | dormant | 827 |  |  | 1622 | 55 | 1572 | 1618 (1618) | v758 (v758) | 1484 | dormant (+4 ± 39 vs peak v758 before dormancy at v824, at 1480 vs refs; needs > +39) | 750 (63056) | +397 -423 =180, 48.7%, -9 ± 20: reverted | 827 |
| T-L-Nc-fresh-c | qualified | 1630 | 37 x 1000 | 242 | 1583 | 55 | 1555 | 1589 | v1556 | 1445 | ok (-6 ± 58 vs peak v1556; predecessor +28) | 750 (62334) | +406 -407 =187, 50.0%, -0 ± 19: reverted | 1630 |
| D-L-Nc-c | qualified | 1287 | 41 x 1000 | 238 | 1602 | 55 | 1533 | 1593 | v1203 | 1465 | new peak (+10 ± 58 over v1203; predecessor +70) | 750 (63741) | +465 -371 =164, 54.7%, +33 ± 20: kept | 1288 |
| T-L-LLs-c | frozen | 651 |  |  | 1576 | 8 |  | 1624 | v540 | 1438 | frozen |  |  | 651 |
| T-L-NNs-c | dormant | 2278 |  |  | 1632 | 56 | 1580 | 1605 (1605) | v1590 (v1590) | 1494 | dormant (+27 ± 36 vs peak v1590 before dormancy at v2274, at 1467 vs refs; needs > +36) | 750 (63490) | +406 -400 =194, 50.3%, +2 ± 19: kept | 2279 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 951 | 8 |  | 1697 | v818 | 813 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | frozen | 742 |  |  | 900 | 16 |  | 1635 | v682 | 762 | frozen |  |  | 742 |
| T-L-Le-pre-c | dormant | 935 | 35 x 1000 | 239 | 1500 | 54 | 1473 | 1562 | v739 | 1363 | dormant (strike 2; -61 ± 56 vs peak v739; predecessor +28) | 750 (63290) | +502 -346 =152, 57.8%, +55 ± 20: kept | 936 |
| T-N-Ne-c | frozen | 637 |  |  | 1251 | 8 |  | 1363 | v574 | 1114 | frozen |  |  | 637 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 708 | 9 |  | 1339 | v270 | 570 | frozen |  |  | 417 |
| T-N-NNs-merge-d2jit-c | dormant | 3061 |  |  | 1087 | 53 | 1119 | 1204 (1204) | v2345 (v2345) | 950 | dormant (-117 ± 35 vs peak v2345 before dormancy at v3060, at 1067 vs refs; needs > +35) | 750 (65838) | +372 -428 =200, 47.2%, -20 ± 19: reverted | 3061 |
| T-L-Le-merge-d2jit-c | dormant | 1593 |  |  | 1635 | 26 |  | 1768 (1768) | v1404 (v1404) | 1497 | dormant (-133 ± 33 vs peak v1404 before dormancy at v1592, at 1631 vs refs; needs > +33) | 750 (64896) | +274 -510 =216, 38.2%, -84 ± 20: reverted | 1593 |
| T-L-NLs-d2jit-c | qualified | 1272 | 62 x 40 | 242 | 1504 | 54 | 1533 | 1534 | v1081 | 1367 | ok (-30 ± 56 vs peak v1081; predecessor -28) | 750 (71164) | +431 -337 =232, 54.7%, +33 ± 19: kept | 1273 |
| T-N-Nc-merge-d2jit-c | dormant | 1436 |  |  | 1132 | 54 | 1128 | 1196 (1196) | v1263 (v1263) | 994 | dormant (-65 ± 37 vs peak v1263 before dormancy at v1433, at 1059 vs refs; needs > +37) | 750 (66212) | +402 -362 =236, 52.0%, +14 ± 19: kept | 1437 |
| T-L-NLs-merge-d2jit-c | qualified | 1469 | 82 x 40 | 240 | 1638 | 56 | 1533 | 1627 | v1019 | 1500 | new peak (+11 ± 58 over v1019; predecessor +105) | 750 (67083) | +457 -361 =182, 54.8%, +34 ± 20: kept | 1470 |
| F-L-d4mix-c | dormant | 321 |  |  | 1746 | 44 |  | 1748 (1739) | v320 (v289) | 1609 | dormant (+7 ± 35 vs peak v289 before dormancy at v319, at 1602 vs refs; needs > +35) | 750 (63927) | +399 -435 =166, 48.2%, -12 ± 20: reverted | 321 |
| MoE-d2jit-c | qualified | 1815 | 41 x 40 | 242 | 1702 | 58 | 1716 | 1716 | v1773 | 1565 | ok (-14 ± 63 vs peak v1773; predecessor -14) | 750 (66815) | +400 -370 =230, 51.5%, +10 ± 19: kept | 1816 |
