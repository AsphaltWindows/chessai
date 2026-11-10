# Cycle 34

**Own-games step, pooled over the lineages: 23 gates, +9806 -9333 =3861, score 51.0%, Elo +7.1 ± 4.1; kept 14 of 23.**

All cycles so far: 1126 gates, +500951 -443313 =181736, score 52.6%, Elo +17.8 ± 0.6; kept 742 of 1126.

2026-11-07 09:22: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 42 players to retired.txt (their games stay in the fit), added 42.
Tournament: 5 double round-robins of the 57 active players (15960 games) in 5.1 min, 15960 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| F-L-c | 9 of 10 | cycle 35 | 26–34 | 979.4 | 1683.5 | retire |
| T-N-NLs-merge-c | 9 of 10 | cycle 35 | 26–34 | 631.6 | 1134.6 | retire |
| F-L-d1play-c | 8 of 10 | cycle 36 | 27–34 | 980.7 | 1545.4 | retire |
| T-L-NLs-c | 7 of 10 | cycle 37 | 28–34 | 981.9 | 1537.6 | retire |
| T-L-Le-c | 4 of 10 | cycle 40 | 31–34 | 922.8 | 1538.7 | retire |
| F-L-d2jit-c | 3 of 10 | cycle 41 | 32–34 | 792.5 | 1785.4 | freeze 1 at 800 |
| F-N-d2jit-c | 2 of 10 | cycle 42 | 33–34 | 605.8 | 1216.8 | freeze 1 at 600 |
| T-L-Le-fresh-c | 1 of 10 | cycle 43 | 34–34 | 931.9 | 1605.1 | retire |
| T-N-NNs-merge-d2jit-c | 0 of 10 | cycle 44 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | dormant | 3145 |  |  | 1702 | 52 | 1682 | 1713 (1713) | v2860 (v2860) | 1561 | dormant (-11 ± 31 vs peak v2860 before dormancy at v3138, at 1572 vs refs; needs > +31) | 1010 (81500) | +419 -405 =176, 50.7%, +5 ± 20: kept | 3146 |
| F-L-d1play-c | dormant | 3321 |  |  | 1525 | 47 | 1568 | 1596 (1596) | v3006 (v3006) | 1384 | dormant (-72 ± 31 vs peak v3006 before dormancy at v3313, at 1455 vs refs; needs > +31) | 1010 (85882) | +439 -409 =152, 51.5%, +10 ± 20: kept | 3322 |
| F-L-d2jit-c | dormant | 3941 |  |  | 1797 | 29 |  | 1814 (1814) | v3348 (v3348) | 1656 | dormant (-17 ± 28 vs peak v3348 before dormancy at v3941, at 1673 vs refs; needs > +28) | 1010 (87674) | +386 -356 =258, 51.5%, +10 ± 19: kept | 3942 |
| F-N-c | frozen | 1691 |  |  | 1376 | 16 |  | 1475 | v1061 | 1234 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1705 | 14 |  | 1719 | v300 | 1564 | frozen |  |  | 373 |
| FM-c | qualified | 646 | 20 x 1000 | 236 | 1422 | 46 | 1668 | 1668 | v626 | 1281 | strike 1 (-246 ± 51 vs peak v626; predecessor -246) | 1010 (91133) | +575 -309 =116, 63.3%, +95 ± 21: kept | 647 |
| F-P-c | qualified | 1808 | 162 x 1000 | 240 | 1492 | 46 | 1519 | 1526 | v1087 | 1350 | ok (-34 ± 48 vs peak v1087; predecessor -27) | 1010 (89773) | +396 -379 =225, 50.8%, +6 ± 19: kept | 1809 |
| T-L-Ne-c | qualified | 1259 | 30 x 1000 | 242 | 1616 | 49 | 1558 | 1622 | v1155 | 1475 | ok (-7 ± 50 vs peak v1155; predecessor +57) | 1010 (85996) | +352 -489 =159, 43.1%, -48 ± 20: reverted | 1259 |
| T-L-Ne-fresh-c | qualified | 633 | 28 x 1000 | 244 | 1614 | 49 | 1550 | 1571 | v524 | 1473 | new peak (+43 ± 50 over v524; predecessor +64) | 1010 (86819) | +430 -393 =177, 51.8%, +13 ± 20: kept | 634 |
| T-L-Ne-pre-c | qualified | 458 | 29 x 1000 | 243 | 1566 | 47 | 1464 | 1524 | v310 | 1425 | new peak (+41 ± 49 over v310; predecessor +102) | 1010 (89019) | +447 -410 =143, 51.8%, +13 ± 20: kept | 459 |
| T-L-Nc-fresh-c | qualified | 822 | 32 x 1000 | 240 | 1595 | 48 | 1549 | 1594 | v701 | 1454 | new peak (+0 ± 49 over v701; predecessor +46) | 1010 (85906) | +406 -393 =201, 50.6%, +4 ± 19: kept | 823 |
| D-L-Nc-c | qualified | 400 | 35 x 1000 | 246 | 1487 | 46 | 1558 | 1558 | v365 | 1346 | strike 1 (-71 ± 49 vs peak v365; predecessor -71) | 1010 (90018) | +510 -361 =129, 57.5%, +52 ± 20: kept | 401 |
| T-L-LLs-c | frozen | 651 |  |  | 1611 | 12 |  | 1631 | v540 | 1470 | frozen |  |  | 651 |
| T-L-NLs-c | dormant | 1433 |  |  | 1524 | 17 |  | 1587 (1587) | v1345 (v1345) | 1383 | dormant (-62 ± 22 vs peak v1345 before dormancy at v1433, at 1446 vs refs; needs > +22) | 1010 (88849) | +377 -452 =171, 46.2%, -26 ± 20: reverted | 1433 |
| T-L-NNs-c | qualified | 1710 | 33 x 1000 | 238 | 1622 | 49 | 1610 | 1628 | v1590 | 1481 | ok (-6 ± 50 vs peak v1590; predecessor +12) | 1010 (88718) | +416 -436 =148, 49.0%, -7 ± 20: reverted | 1710 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 968 | 12 |  | 1704 | v818 | 827 | frozen |  |  | 1070 |
| T-L-Le-c | dormant | 1175 |  |  | 1605 | 48 | 1558 | 1679 (1679) | v1081 (v1081) | 1464 | dormant (-74 ± 33 vs peak v1081 before dormancy at v1171, at 1538 vs refs; needs > +33) | 1010 (84709) | +399 -452 =149, 47.3%, -18 ± 20: reverted | 1175 |
| T-L-Le-fresh-c | dormant | 681 |  |  | 1605 | 48 | 1549 | 1610 (1610) | v594 (v594) | 1464 | dormant (-5 ± 33 vs peak v594 before dormancy at v680, at 1469 vs refs; needs > +33) | 1010 (87286) | +486 -409 =105, 53.8%, +27 ± 20: kept | 682 |
| T-L-Le-pre-c | qualified | 440 | 31 x 1000 | 241 | 1476 | 46 | 1469 | 1501 | v299 | 1335 | ok (-25 ± 48 vs peak v299; predecessor +7) | 1010 (85423) | +405 -465 =130, 47.0%, -21 ± 20: reverted | 440 |
| T-N-Ne-c | frozen | 637 |  |  | 1274 | 11 |  | 1370 | v574 | 1133 | frozen |  |  | 637 |
| T-N-Nc-c | qualified | 1215 | 30 x 1000 | 238 | 1352 | 46 | 1315 | 1389 | v472 | 1211 | ok (-37 ± 51 vs peak v472; predecessor +38) | 1010 (85818) | +420 -422 =158, 49.9%, -1 ± 20: reverted | 1215 |
| T-N-Nc-merge-c | qualified | 1882 | 49 x 1000 | 241 | 1404 | 46 | 1433 | 1448 | v1598 | 1263 | ok (-44 ± 47 vs peak v1598; predecessor -29) | 1010 (83239) | +413 -433 =154, 49.0%, -7 ± 20: reverted | 1882 |
| T-N-Nc-fresh-c | qualified | 721 | 41 x 1000 | 238 | 1339 | 46 | 1309 | 1370 | v624 | 1198 | ok (-32 ± 48 vs peak v624; predecessor +29) | 1010 (82703) | +418 -431 =151, 49.4%, -4 ± 20: reverted | 721 |
| T-N-NLs-merge-c | dormant | 1037 |  |  | 1094 | 45 | 1102 | 1416 (1416) | v787 (v787) | 953 | dormant (-322 ± 26 vs peak v787 before dormancy at v1030, at 1275 vs refs; needs > +26) | 1010 (87972) | +439 -399 =162, 52.0%, +14 ± 20: kept | 1038 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 719 | 14 |  | 1347 | v270 | 578 | frozen |  |  | 417 |
| F-N-d2jit-c | dormant | 2048 |  |  | 1212 | 46 | 1221 | 1261 (1261) | v1786 (v1786) | 1071 | dormant (-50 ± 31 vs peak v1786 before dormancy at v2046, at 1120 vs refs; needs > +31) | 1010 (87269) | +421 -380 =199, 52.0%, +14 ± 19: kept | 2049 |
| T-N-NNs-merge-d2jit-c | dormant | 2112 | 55 x 40 | 241 | 1126 | 45 | 1127 | 1171 | v1553 | 985 | dormant (strike 2; -45 ± 44 vs peak v1553; predecessor -1) | 1010 (86818) | +381 -387 =232, 49.7%, -2 ± 19: reverted | 2112 |
| F-L-d4mix-c | qualified | 192 | 8 x 10 | 237 | 1694 | 52 | 1709 | 1717 | v141 | 1553 | ok (-23 ± 52 vs peak v141; predecessor -15) | 1010 (84100) | +444 -388 =168, 52.8%, +20 ± 20: kept | 193 |
| MoE-d2jit-c | qualified | 918 | 36 x 40 | 241 | 1666 | 50 | 1727 | 1727 | v882 | 1524 | strike 1 (-61 ± 55 vs peak v882; predecessor -61) | 1010 (91709) | +427 -375 =198, 52.6%, +18 ± 19: kept | 919 |
