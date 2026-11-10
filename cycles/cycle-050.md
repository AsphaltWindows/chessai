# Cycle 50

**Own-games step, pooled over the lineages: 17 gates, +7002 -6653 =3345, score 51.0%, Elo +7.1 ± 4.7; kept 12 of 17.**

All cycles so far: 1487 gates, +655408 -582451 =249141, score 52.5%, Elo +17.1 ± 0.5; kept 993 of 1487.

2026-11-07 15:50: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 28 players to retired.txt (their games stay in the fit), added 28.
Tournament: 5 double round-robins of the 49 active players (11760 games) in 5.8 min, 11760 games saved.
Freeze/retire rule, after 10 cycles dormant: F-P-c frozen at ply 2 (avg 1491.0 over cycles 41–50; milestone 1500). 1 entries leave the field for retired.txt (their games stay in the fit): 49 players become 48, 11,760 games a cycle become 11,280.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| F-N-d2jit-c | 8 of 10 | cycle 52 | 43–50 | 657.4 | 1230.8 | retire |
| T-N-Nc-c | 8 of 10 | cycle 52 | 43–50 | 750.8 | 1343.9 | retire |
| T-L-Ne-fresh-c | 4 of 10 | cycle 56 | 47–50 | 942.5 | 1586.5 | retire |
| F-P-tau0-c | 2 of 10 | cycle 58 | 49–50 | 1040.1 | 1541.2 | retire |
| T-L-NNs-c | 2 of 10 | cycle 58 | 49–50 | 976.3 | 1590.7 | retire |
| F-L-d4mix-c | 0 of 10 | cycle 60 | - | - | - | retire |
| T-L-Le-merge-d2jit-c | 0 of 10 | cycle 60 | - | - | - | retire |
| T-N-Nc-merge-d2jit-c | 0 of 10 | cycle 60 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 900 T-L-Le-fresh-c-v742-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1500 F-P-c-v2808-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d2jit-c | frozen | 3943 |  |  | 815 | 13 |  | 1799 | v3348 | 675 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1367 | 10 |  | 1472 | v1061 | 1227 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1678 | 10 |  | 1716 | v300 | 1537 | frozen |  |  | 373 |
| F-P-c | frozen | 2808 |  |  | 1497 | 36 |  | 1578 (1578) | v2138 (v2138) | 1357 | frozen at ply 2 (avg 1491.0 over cycles 41–50; milestone 1500); dormant (-81 ± 30 vs peak v2138 before dormancy at v2801, at 1437 vs refs; needs > +30) |  |  | 2808 |
| F-P-tau0-c | dormant | 1959 |  |  | 1553 | 52 | 1529 | 1562 (1562) | v862 (v862) | 1412 | dormant (-9 ± 35 vs peak v862 before dormancy at v1957, at 1421 vs refs; needs > +35) | 850 (69795) | +404 -392 =204, 50.6%, +4 ± 19: kept | 1960 |
| F-P-tau0-d2jit-c | qualified | 1728 | 153 x 40 | 240 | 1692 | 56 | 1746 | 1746 | v1574 | 1551 | ok (-54 ± 61 vs peak v1574; predecessor -54) | 850 (74380) | +316 -315 =369, 50.0%, +0 ± 17: kept | 1729 |
| T-L-Ne-fresh-c | dormant | 1041 |  |  | 1598 | 37 |  | 1598 (1598) | v633 (v633) | 1457 | dormant (-0 ± 31 vs peak v633 before dormancy at v1038, at 1457 vs refs; needs > +31) | 850 (70726) | +394 -448 =158, 47.3%, -19 ± 20: reverted | 1041 |
| T-L-Ne-pre-c | qualified | 758 |  |  | 1625 | 54 | 1578 | 1602 (1573) | v756 (v691) | 1484 | re-qualified (+52 ± 36 vs peak v691 before dormancy at v752, at 1432 vs refs) | 850 (72611) | +414 -395 =191, 50.9%, +7 ± 19: kept | 759 |
| T-L-Nc-fresh-c | qualified | 1441 | 38 x 1000 | 241 | 1561 | 52 | 1544 | 1589 | v1015 | 1421 | ok (-27 ± 53 vs peak v1015; predecessor +17) | 850 (71675) | +398 -428 =174, 48.5%, -10 ± 20: reverted | 1441 |
| D-L-Nc-c | qualified | 1070 | 47 x 1000 | 242 | 1566 | 52 | 1526 | 1571 | v851 | 1426 | ok (-5 ± 53 vs peak v851; predecessor +41) | 850 (73299) | +466 -382 =152, 54.2%, +29 ± 20: kept | 1071 |
| T-L-LLs-c | frozen | 651 |  |  | 1587 | 9 |  | 1628 | v540 | 1446 | frozen |  |  | 651 |
| T-L-NNs-c | dormant | 2276 |  |  | 1625 | 54 | 1555 | 1612 (1612) | v1590 (v1590) | 1484 | dormant (+13 ± 36 vs peak v1590 before dormancy at v2274, at 1471 vs refs; needs > +36) | 850 (73334) | +412 -444 =144, 48.4%, -11 ± 20: reverted | 2276 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 958 | 8 |  | 1701 | v818 | 817 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | frozen | 742 |  |  | 911 | 21 |  | 1642 | v682 | 771 | frozen |  |  | 742 |
| T-L-Le-pre-c | qualified | 770 | 31 x 1000 | 241 | 1517 | 51 | 1570 | 1570 | v739 | 1376 | ok (-53 ± 54 vs peak v739; predecessor -53) | 850 (74770) | +464 -409 =127, 52.8%, +19 ± 20: kept | 771 |
| T-N-Ne-c | frozen | 637 |  |  | 1259 | 8 |  | 1368 | v574 | 1118 | frozen |  |  | 637 |
| T-N-Nc-c | dormant | 1505 |  |  | 1340 | 36 |  | 1387 (1387) | v472 (v472) | 1200 | dormant (-47 ± 30 vs peak v472 before dormancy at v1500, at 1247 vs refs; needs > +30) | 850 (70201) | +404 -423 =173, 49.0%, -7 ± 20: reverted | 1505 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 712 | 10 |  | 1344 | v270 | 572 | frozen |  |  | 417 |
| F-N-d2jit-c | dormant | 2424 |  |  | 1246 | 35 |  | 1287 (1287) | v2053 (v2053) | 1106 | dormant (-41 ± 28 vs peak v2053 before dormancy at v2418, at 1146 vs refs; needs > +28) | 850 (73947) | +412 -390 =198, 51.1%, +8 ± 19: kept | 2425 |
| T-N-NNs-merge-d2jit-c | qualified | 2907 | 82 x 40 | 242 | 1178 | 50 | 1138 | 1211 | v2345 | 1038 | ok (-33 ± 50 vs peak v2345; predecessor +40) | 850 (74785) | +414 -381 =205, 51.6%, +12 ± 19: kept | 2908 |
| T-L-Le-merge-d2jit-c | dormant | 1592 | 99 x 40 | 241 | 1636 | 54 | 1657 | 1776 | v1404 | 1495 | dormant (strike 2; -140 ± 62 vs peak v1404; predecessor -21) | 850 (71802) | +394 -379 =227, 50.7%, +5 ± 19: kept | 1593 |
| T-L-NLs-d2jit-c | qualified | 957 | 58 x 40 | 239 | 1518 | 51 | 1522 | 1522 | v898 | 1378 | ok (-4 ± 53 vs peak v898; predecessor -4) | 850 (78770) | +393 -362 =245, 51.5%, +11 ± 19: kept | 958 |
| T-N-Nc-merge-d2jit-c | dormant | 1433 | 77 x 40 | 242 | 1128 | 49 | 1140 | 1204 | v1263 | 987 | dormant (strike 2; -76 ± 52 vs peak v1263; predecessor -12) | 850 (77079) | +414 -387 =199, 51.3%, +9 ± 19: kept | 1434 |
| T-L-NLs-merge-d2jit-c | qualified | 1019 | 74 x 40 | 239 | 1634 | 54 | 1478 | 1613 | v676 | 1494 | new peak (+21 ± 55 over v676; predecessor +156) | 850 (75064) | +495 -325 =180, 58.5%, +60 ± 20: kept | 1020 |
| F-L-d4mix-c | dormant | 319 | 7 x 10 | 240 | 1672 | 55 | 1686 | 1747 | v289 | 1531 | dormant (strike 2; -75 ± 59 vs peak v289; predecessor -14) | 850 (73908) | +378 -434 =188, 47.2%, -20 ± 19: reverted | 319 |
| MoE-d2jit-c | qualified | 1586 | 40 x 40 | 240 | 1688 | 56 | 1717 | 1719 | v1364 | 1547 | ok (-32 ± 58 vs peak v1364; predecessor -29) | 850 (71803) | +430 -359 =211, 53.5%, +25 ± 19: kept | 1587 |
