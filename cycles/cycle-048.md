# Cycle 48

**Own-games step, pooled over the lineages: 20 gates, +8460 -7605 =3935, score 52.1%, Elo +14.9 ± 4.3; kept 17 of 20.**

All cycles so far: 1452 gates, +640952 -568747 =242301, score 52.5%, Elo +17.3 ± 0.5; kept 972 of 1452.

2026-11-07 15:13: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 38 players to retired.txt (their games stay in the fit), added 38.
Tournament: 5 double round-robins of the 53 active players (13780 games) in 5.7 min, 13780 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| T-L-Ne-c | 9 of 10 | cycle 49 | 40–48 | 1011.6 | 1567.2 | retire |
| T-N-Nc-merge-c | 9 of 10 | cycle 49 | 40–48 | 757.5 | 1390.7 | retire |
| F-P-c | 8 of 10 | cycle 50 | 41–48 | 927.4 | 1488.3 | retire |
| F-N-d2jit-c | 6 of 10 | cycle 52 | 43–48 | 650.8 | 1225.8 | retire |
| T-N-Nc-c | 6 of 10 | cycle 52 | 43–48 | 750.7 | 1348.0 | retire |
| T-L-Le-pre-c | 5 of 10 | cycle 53 | 44–48 | 897.8 | 1502.1 | freeze 2 at 1500 |
| T-L-Ne-pre-c | 5 of 10 | cycle 53 | 44–48 | 934.5 | 1579.5 | retire |
| T-L-Ne-fresh-c | 2 of 10 | cycle 56 | 47–48 | 921.3 | 1571.9 | retire |
| F-P-tau0-c | 0 of 10 | cycle 58 | - | - | - | retire |
| T-L-NNs-c | 0 of 10 | cycle 58 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 900 T-L-Le-fresh-c-v742-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d2jit-c | frozen | 3943 |  |  | 812 | 14 |  | 1801 | v3348 | 672 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1369 | 10 |  | 1472 | v1061 | 1229 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1683 | 10 |  | 1716 | v300 | 1543 | frozen |  |  | 373 |
| F-P-c | dormant | 2807 |  |  | 1461 | 49 | 1444 | 1580 (1580) | v2138 (v2138) | 1321 | dormant (-119 ± 35 vs peak v2138 before dormancy at v2801, at 1440 vs refs; needs > +35) | 930 (85876) | +421 -408 =171, 50.6%, +4 ± 20: kept | 2808 |
| F-P-tau0-c | dormant | 1957 | 183 x 1000 | 240 | 1492 | 49 | 1512 | 1563 | v862 | 1352 | dormant (strike 2; -71 ± 51 vs peak v862; predecessor -20) | 930 (77501) | +414 -361 =225, 52.6%, +18 ± 19: kept | 1958 |
| F-P-tau0-d2jit-c | qualified | 1423 | 144 x 40 | 240 | 1724 | 57 | 1708 | 1714 | v847 | 1584 | new peak (+10 ± 58 over v847; predecessor +16) | 930 (83950) | +317 -316 =367, 50.0%, +0 ± 17: kept | 1424 |
| T-L-Ne-c | dormant | 1428 |  |  | 1591 | 51 | 1534 | 1610 (1610) | v1288 (v1288) | 1451 | dormant (-20 ± 36 vs peak v1288 before dormancy at v1425, at 1470 vs refs; needs > +36) | 930 (79414) | +400 -438 =162, 48.1%, -13 ± 20: reverted | 1428 |
| T-L-Ne-fresh-c | dormant | 1040 |  |  | 1570 | 50 | 1570 | 1600 (1600) | v633 (v633) | 1430 | dormant (-30 ± 35 vs peak v633 before dormancy at v1038, at 1460 vs refs; needs > +35) | 930 (77581) | +443 -401 =156, 52.1%, +15 ± 20: kept | 1041 |
| T-L-Ne-pre-c | dormant | 756 |  |  | 1602 | 51 | 1549 | 1578 (1574) | v753 (v691) | 1462 | dormant (+28 ± 35 vs peak v691 before dormancy at v752, at 1434 vs refs; needs > +35) | 930 (78925) | +427 -388 =185, 51.9%, +14 ± 20: kept | 757 |
| T-L-Nc-fresh-c | qualified | 1365 | 39 x 1000 | 243 | 1554 | 50 | 1572 | 1590 | v1015 | 1414 | ok (-36 ± 52 vs peak v1015; predecessor -18) | 930 (80437) | +436 -393 =171, 52.1%, +15 ± 20: kept | 1366 |
| D-L-Nc-c | qualified | 981 | 43 x 1000 | 241 | 1568 | 50 | 1509 | 1573 | v851 | 1428 | ok (-5 ± 52 vs peak v851; predecessor +59) | 930 (79623) | +519 -328 =153, 59.6%, +67 ± 20: kept | 982 |
| T-L-LLs-c | frozen | 651 |  |  | 1590 | 9 |  | 1628 | v540 | 1450 | frozen |  |  | 651 |
| T-L-NNs-c | dormant | 2274 | 40 x 1000 | 238 | 1535 | 50 | 1560 | 1614 | v1590 | 1395 | dormant (strike 2; -79 ± 51 vs peak v1590; predecessor -25) | 930 (82445) | +427 -415 =158, 50.6%, +4 ± 20: kept | 2275 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 958 | 9 |  | 1701 | v818 | 818 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | frozen | 742 |  |  | 907 | 26 |  | 1644 | v682 | 767 | frozen |  |  | 742 |
| T-L-Le-pre-c | dormant | 738 |  |  | 1489 | 49 | 1510 | 1523 (1523) | v670 (v670) | 1349 | dormant (-34 ± 34 vs peak v670 before dormancy at v735, at 1383 vs refs; needs > +34) | 930 (79614) | +427 -407 =166, 51.0%, +7 ± 20: kept | 739 |
| T-N-Ne-c | frozen | 637 |  |  | 1261 | 9 |  | 1368 | v574 | 1121 | frozen |  |  | 637 |
| T-N-Nc-c | dormant | 1504 |  |  | 1352 | 48 | 1348 | 1387 (1387) | v472 (v472) | 1212 | dormant (-35 ± 35 vs peak v472 before dormancy at v1500, at 1247 vs refs; needs > +35) | 930 (78170) | +439 -412 =149, 51.3%, +9 ± 20: kept | 1505 |
| T-N-Nc-merge-c | dormant | 2129 |  |  | 1393 | 34 |  | 1455 (1445) | v2127 (v631) | 1253 | dormant (-52 ± 23 vs peak v631 before dormancy at v2126, at 1305 vs refs; needs > +23) | 930 (75211) | +436 -419 =145, 50.8%, +6 ± 20: kept | 2130 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 711 | 10 |  | 1344 | v270 | 571 | frozen |  |  | 417 |
| F-N-d2jit-c | dormant | 2423 |  |  | 1268 | 48 | 1209 | 1288 (1288) | v2053 (v2053) | 1128 | dormant (-20 ± 33 vs peak v2053 before dormancy at v2418, at 1148 vs refs; needs > +33) | 930 (80824) | +422 -382 =196, 52.0%, +14 ± 19: kept | 2424 |
| T-N-NNs-merge-d2jit-c | qualified | 2750 | 104 x 40 | 240 | 1181 | 48 | 1172 | 1212 | v2345 | 1041 | ok (-31 ± 49 vs peak v2345; predecessor +9) | 930 (81957) | +385 -396 =219, 49.5%, -4 ± 19: reverted | 2750 |
| T-L-Le-merge-d2jit-c | qualified | 1404 | 119 x 40 | 249 | 1776 | 61 | 1556 | 1714 | v1182 | 1636 | new peak (+63 ± 61 over v1182; predecessor +221) | 930 (81251) | +402 -370 =228, 51.6%, +11 ± 19: kept | 1405 |
| T-L-NLs-d2jit-c | qualified | 841 | 58 x 40 | 239 | 1495 | 49 | 1470 | 1484 | v725 | 1355 | new peak (+11 ± 51 over v725; predecessor +25) | 930 (89113) | +434 -333 =233, 55.0%, +35 ± 19: kept | 842 |
| T-N-Nc-merge-d2jit-c | qualified | 1263 |  |  | 1203 | 48 | 1158 | 1169 (1169) | v655 (v655) | 1063 | re-qualified (+34 ± 32 vs peak v655 before dormancy at v1261, at 1029 vs refs) | 930 (82265) | +382 -385 =233, 49.9%, -1 ± 19: reverted | 1263 |
| T-L-NLs-merge-d2jit-c | qualified | 852 | 85 x 40 | 240 | 1566 | 50 | 1502 | 1614 | v676 | 1426 | ok (-48 ± 53 vs peak v676; predecessor +64) | 930 (84342) | +498 -279 =223, 61.0%, +77 ± 19: kept | 853 |
| F-L-d4mix-c | qualified | 303 | 7 x 10 | 236 | 1714 | 56 | 1673 | 1748 | v289 | 1574 | ok (-34 ± 59 vs peak v289; predecessor +41) | 930 (81947) | +405 -392 =203, 50.6%, +4 ± 19: kept | 304 |
| MoE-d2jit-c | qualified | 1501 | 47 x 40 | 240 | 1672 | 54 | 1659 | 1721 | v1364 | 1532 | ok (-49 ± 57 vs peak v1364; predecessor +13) | 930 (82866) | +426 -382 =192, 52.2%, +15 ± 19: kept | 1502 |
