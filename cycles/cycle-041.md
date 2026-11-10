# Cycle 41

**Own-games step, pooled over the lineages: 24 gates, +10482 -9081 =4437, score 52.9%, Elo +20.3 ± 4.0; kept 16 of 24.**

All cycles so far: 1295 gates, +574189 -507855 =212956, score 52.6%, Elo +17.8 ± 0.5; kept 861 of 1295.

2026-11-07 12:33: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 42 players to retired.txt (their games stay in the fit), added 42.
Tournament: 5 double round-robins of the 61 active players (18300 games) in 5.2 min, 18300 games saved.
Freeze/retire rule, after 10 cycles dormant: F-L-d2jit-c frozen at ply 1 (avg 804.4 over cycles 32–41; milestone 800). 1 entries leave the field for retired.txt (their games stay in the fit): 61 players become 60, 18,300 games a cycle become 17,700.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| FM-c | 6 of 10 | cycle 45 | 36–41 | 893.2 | 1624.6 | freeze 1 at 900 |
| T-N-Nc-fresh-c | 5 of 10 | cycle 46 | 37–41 | 723.9 | 1301.6 | retire |
| F-L-d1play-c | 4 of 10 | cycle 47 | 38–41 | 989.8 | 1524.2 | retire |
| T-L-Le-fresh-c | 4 of 10 | cycle 47 | 38–41 | 896.3 | 1655.8 | retire |
| T-L-Ne-c | 2 of 10 | cycle 49 | 40–41 | 1022.3 | 1556.8 | retire |
| T-N-Nc-merge-c | 2 of 10 | cycle 49 | 40–41 | 762.6 | 1407.2 | retire |
| F-P-c | 1 of 10 | cycle 50 | 41–41 | 904.2 | 1470.7 | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d1play-c | dormant | 3589 |  |  | 1529 | 45 | 1524 | 1631 (1631) | v3322 (v3322) | 1390 | dormant (-102 ± 34 vs peak v3322 before dormancy at v3585, at 1492 vs refs; needs > +34) | 1090 (97212) | +447 -420 =133, 51.3%, +9 ± 20: kept | 3590 |
| F-L-d2jit-c | frozen | 3943 |  |  | 1780 | 26 |  | 1807 (1807) | v3348 (v3348) | 1640 | frozen at ply 1 (avg 804.4 over cycles 32–41; milestone 800); dormant (-27 ± 28 vs peak v3348 before dormancy at v3941, at 1667 vs refs; needs > +28) |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1369 | 12 |  | 1472 | v1061 | 1229 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1693 | 12 |  | 1717 | v300 | 1553 | frozen |  |  | 373 |
| FM-c | dormant | 669 |  |  | 1613 | 28 |  | 1659 (1659) | v626 (v626) | 1474 | dormant (-46 ± 29 vs peak v626 before dormancy at v666, at 1520 vs refs; needs > +29) | 1090 (98986) | +390 -424 =186, 48.3%, -12 ± 20: reverted | 669 |
| F-P-c | dormant | 2802 |  |  | 1471 | 44 | 1465 | 1586 (1586) | v2138 (v2138) | 1331 | dormant (-115 ± 33 vs peak v2138 before dormancy at v2801, at 1446 vs refs; needs > +33) | 1090 (102693) | +498 -331 =171, 58.4%, +59 ± 20: kept | 2803 |
| F-P-tau0-c | qualified | 688 | 176 x 1000 | 240 | 1514 | 45 | 1512 | 1512 | v339 | 1374 | new peak (+1 ± 48 over v339; predecessor +2) | 1090 (91084) | +388 -390 =222, 49.9%, -1 ± 19: reverted | 688 |
| F-P-tau0-d2jit-c | qualified | 422 | 137 x 40 | 240 | 1626 | 47 | 1564 | 1564 | v284 | 1486 | new peak (+62 ± 49 over v284; predecessor +62) | 1090 (95334) | +350 -342 =308, 50.4%, +3 ± 18: kept | 423 |
| T-L-Ne-c | dormant | 1425 |  |  | 1553 | 27 |  | 1616 (1616) | v1288 (v1288) | 1414 | dormant (-63 ± 28 vs peak v1288 before dormancy at v1425, at 1476 vs refs; needs > +28) | 1090 (96345) | +410 -422 =168, 49.4%, -4 ± 20: reverted | 1425 |
| T-L-Ne-fresh-c | qualified | 859 | 34 x 1000 | 243 | 1592 | 46 | 1603 | 1606 | v633 | 1452 | ok (-14 ± 51 vs peak v633; predecessor -11) | 1090 (92282) | +416 -418 =166, 49.9%, -1 ± 20: reverted | 859 |
| T-L-Ne-pre-c | qualified | 691 | 34 x 1000 | 240 | 1577 | 46 | 1546 | 1576 | v548 | 1437 | new peak (+0 ± 50 over v548; predecessor +31) | 1090 (96273) | +444 -396 =160, 52.4%, +17 ± 20: kept | 692 |
| T-L-Nc-fresh-c | qualified | 1098 | 42 x 1000 | 240 | 1574 | 46 | 1595 | 1596 | v1015 | 1434 | ok (-22 ± 49 vs peak v1015; predecessor -21) | 1090 (91925) | +395 -414 =191, 49.0%, -7 ± 19: reverted | 1098 |
| D-L-Nc-c | qualified | 688 | 44 x 1000 | 238 | 1556 | 46 | 1534 | 1550 | v365 | 1416 | new peak (+6 ± 49 over v365; predecessor +22) | 1090 (93992) | +510 -344 =146, 58.3%, +58 ± 20: kept | 689 |
| T-L-LLs-c | frozen | 651 |  |  | 1602 | 10 |  | 1629 | v540 | 1462 | frozen |  |  | 651 |
| T-L-NNs-c | qualified | 1983 | 46 x 1000 | 240 | 1584 | 46 | 1589 | 1620 | v1590 | 1444 | ok (-36 ± 49 vs peak v1590; predecessor -6) | 1090 (95868) | +432 -396 =172, 51.8%, +12 ± 20: kept | 1984 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 960 | 10 |  | 1701 | v818 | 821 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | dormant | 739 |  |  | 1648 | 28 |  | 1650 (1650) | v682 (v682) | 1508 | dormant (-3 ± 29 vs peak v682 before dormancy at v737, at 1510 vs refs; needs > +29) | 1090 (95226) | +402 -467 =131, 46.8%, -23 ± 20: reverted | 739 |
| T-L-Le-pre-c | qualified | 670 | 35 x 1000 | 239 | 1526 | 45 | 1505 | 1517 | v529 | 1386 | new peak (+9 ± 48 over v529; predecessor +20) | 1090 (91025) | +453 -385 =162, 53.4%, +24 ± 20: kept | 671 |
| T-N-Ne-c | frozen | 637 |  |  | 1268 | 10 |  | 1368 | v574 | 1128 | frozen |  |  | 637 |
| T-N-Nc-c | qualified | 1467 | 36 x 1000 | 240 | 1312 | 44 | 1350 | 1388 | v1359 | 1172 | strike 1 (-76 ± 47 vs peak v1359; predecessor -38) | 1090 (95227) | +415 -419 =166, 49.8%, -1 ± 20: reverted | 1467 |
| T-N-Nc-merge-c | dormant | 2128 |  |  | 1354 | 44 | 1460 | 1460 (1446) | v2127 (v631) | 1214 | dormant (-92 ± 27 vs peak v631 before dormancy at v2126, at 1306 vs refs; needs > +27) | 1090 (91370) | +416 -419 =165, 49.9%, -1 ± 20: reverted | 2128 |
| T-N-Nc-fresh-c | dormant | 807 |  |  | 1306 | 44 | 1319 | 1363 (1363) | v624 (v624) | 1166 | dormant (-57 ± 31 vs peak v624 before dormancy at v803, at 1223 vs refs; needs > +31) | 1090 (93472) | +421 -413 =166, 50.4%, +3 ± 20: kept | 808 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 715 | 12 |  | 1344 | v270 | 575 | frozen |  |  | 417 |
| F-N-d2jit-c | qualified | 2297 | 131 x 40 | 241 | 1220 | 43 | 1257 | 1293 | v2053 | 1080 | strike 1 (-73 ± 46 vs peak v2053; predecessor -37) | 1090 (98866) | +413 -382 =205, 51.5%, +11 ± 19: kept | 2298 |
| T-N-NNs-merge-d2jit-c | qualified | 2170 | 54 x 40 | 240 | 1173 | 43 | 1206 | 1206 | v2116 | 1034 | ok (-33 ± 46 vs peak v2116; predecessor -33) | 1090 (96959) | +401 -376 =223, 51.2%, +9 ± 19: kept | 2171 |
| T-L-Le-merge-d2jit-c | qualified | 698 | 98 x 40 | 241 | 1648 | 48 | 1643 | 1643 | v599 | 1509 | new peak (+5 ± 50 over v599; predecessor +5) | 1090 (95896) | +484 -371 =145, 55.6%, +39 ± 20: kept | 699 |
| T-L-NLs-d2jit-c | qualified | 440 | 56 x 40 | 241 | 1301 | 44 | 1238 | 1238 | v383 | 1161 | new peak (+62 ± 46 over v383; predecessor +62) | 1090 (107081) | +492 -260 =248, 61.6%, +82 ± 19: kept | 441 |
| T-N-Nc-merge-d2jit-c | qualified | 758 | 102 x 40 | 241 | 1170 | 43 | 1174 | 1174 | v655 | 1030 | ok (-4 ± 46 vs peak v655; predecessor -4) | 1090 (97836) | +439 -345 =216, 54.7%, +33 ± 19: kept | 759 |
| T-L-NLs-merge-d2jit-c | qualified | 210 | 106 x 40 | 243 | 1210 | 43 | 819 | 819 | v103 | 1070 | new peak (+391 ± 46 over v103; predecessor +391) | 1090 (107972) | +622 -192 =186, 71.5%, +160 ± 21: kept | 211 |
| F-L-d4mix-c | qualified | 249 | 8 x 10 | 254 | 1687 | 50 | 1725 | 1747 | v200 | 1548 | strike 1 (-60 ± 56 vs peak v200; predecessor -38) | 1090 (92779) | +411 -395 =194, 50.8%, +6 ± 19: kept | 250 |
| MoE-d2jit-c | qualified | 1195 | 42 x 40 | 240 | 1673 | 49 | 1648 | 1718 | v882 | 1533 | ok (-45 ± 55 vs peak v882; predecessor +25) | 1090 (93850) | +433 -360 =207, 53.6%, +25 ± 19: kept | 1196 |
