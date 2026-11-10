# Cycle 26

**Changes from this cycle** (deployed after cycle 25, `staged-c25/`), the user's: three lineages join, from scratch, each with its own binary for self-play, own-step train and gate (`own_games.binary_by_lineage`). (1) T-L-Ne-pre-c and T-L-Le-pre-c: T-L-Ne-c's and T-L-Le-c's recipes plus `--pre-split` on `chessai-presplit`, one factor each (the user: "train the Error router from the beginning so that when the split happens it's not as random"): before each split check a leaf trains its would-be split, the children and the router, in the shadow over the window, and a split installs those trained candidates. (2) D-L-Nc-c, the user's DAG, on `chessai-dag`: one LR leaf until the root's first split check, then a CNBP cluster router over 9 cells that point to 3 shared LR leaves, a cell re-pointed when another leaf's fresh loss on it is lower by 2 standard errors over 50,000 judged positions. Checked before the deploy, from scratch against each line's twin on `chessai-cyc5` (T-L-Ne-c's and T-L-Le-c's lines; for the DAG, its line without the `--dag-*` options): the same games and models over 2 rounds of 1000 games; at a 300k split threshold the same models up to the first split (through v4 for the pre-split trees, which split at v5 in both runs, and through v1 for the DAG, which formed at v2); a resume across a chunk boundary byte-identical to one run, with promoted candidates and the formed DAG; the same games on `chessai-probit` as on `chessai-cyc5` in a round-robin of their entries; and each one's own step through the driver, train and gate on its binary.

**Own-games step, pooled over the lineages: 35 gates, +15401 -14024 =5575, score 52.0%, Elo +13.7 ± 3.3; kept 22 of 35.**

All cycles so far: 920 gates, +412793 -358978 =148229, score 52.9%, Elo +20.3 ± 0.7; kept 617 of 920.

2026-11-07 06:02: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 58 players to retired.txt (their games stay in the fit), added 64.
Tournament: 5 double round-robins of the 86 active players (36550 games) in 4.6 min, 36550 games saved.
Freeze/retire rule, after 10 cycles dormant: F-N-c frozen at ply 2 (avg 1398.2 over cycles 17–26; milestone 1400); T-L-Le-merge-c retired (avg ply 1 977.0, ply 2 1679.6; no untaken milestone within 10); T-L-Nc-c retired (avg ply 1 972.6, ply 2 1609.3; no untaken milestone within 10). 5 entries leave the field for retired.txt (their games stay in the fit): 86 players become 81, 36,550 games a cycle become 32,400.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| T-L-H-c | 9 of 10 | cycle 27 | 18–26 | 985.9 | 1614.1 | retire |
| T-L-NLs-merge-c | 9 of 10 | cycle 27 | 18–26 | 939.3 | 1607.2 | retire |
| T-L-Ne-merge-c | 9 of 10 | cycle 27 | 18–26 | 970.5 | 1673.7 | retire |
| T-N-LLs-merge-c | 9 of 10 | cycle 27 | 18–26 | 624.5 | 1122.4 | retire |
| T-N-NNs-c | 9 of 10 | cycle 27 | 18–26 | 647.5 | 1132.2 | retire |
| T-N-Ne-merge-c | 8 of 10 | cycle 28 | 19–26 | 746.5 | 1400.3 | retire |
| F-L-d2mix-c | 7 of 10 | cycle 29 | 20–26 | 845.3 | 1730.8 | retire |
| T-L-LLs-merge-c | 7 of 10 | cycle 29 | 20–26 | 925.1 | 1552.7 | retire |
| T-L-Nc-merge-c | 7 of 10 | cycle 29 | 20–26 | 967.7 | 1667.8 | retire |
| F-L-d2play-c | 5 of 10 | cycle 31 | 22–26 | 819.6 | 1706.1 | retire |
| F-L-d0soft-c | 4 of 10 | cycle 32 | 23–26 | 985.0 | 1638.5 | retire |
| T-N-Le-merge-c | 3 of 10 | cycle 33 | 24–26 | 728.8 | 1346.4 | retire |
| FM-c | 2 of 10 | cycle 34 | 25–26 | 883.8 | 1567.8 | retire |
| F-L-c | 1 of 10 | cycle 35 | 26–26 | 1008.9 | 1632.5 | retire |
| T-N-NLs-merge-c | 1 of 10 | cycle 35 | 26–26 | 644.8 | 1158.3 | retire |
| F-L-d1play-c | 0 of 10 | cycle 36 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | dormant | 3139 |  |  | 1632 | 40 | 1640 | 1718 (1718) | v2860 (v2860) | 1491 | dormant (-85 ± 25 vs peak v2860 before dormancy at v3138, at 1576 vs refs; needs > +25) | 1590 (132620) | +444 -420 =136, 51.2%, +8 ± 20: kept | 3140 |
| F-L-d0soft-c | dormant | 2363 |  |  | 1648 | 25 |  | 1702 (1702) | v2014 (v2014) | 1507 | dormant (-54 ± 25 vs peak v2014 before dormancy at v2361, at 1561 vs refs; needs > +25) | 1590 (134253) | +460 -395 =145, 53.2%, +23 ± 20: kept | 2364 |
| F-L-d1play-c | dormant | 3313 | 142 x 860 | 240 | 1513 | 38 | 1535 | 1601 | v3006 | 1372 | dormant (strike 2; -88 ± 41 vs peak v3006; predecessor -22) | 1590 (152132) | +492 -363 =145, 56.5%, +45 ± 20: kept | 3314 |
| F-L-d2play-c | dormant | 1623 |  |  | 1735 | 44 | 1712 | 1754 (1754) | v1011 (v1011) | 1594 | dormant (-19 ± 29 vs peak v1011 before dormancy at v1620, at 1613 vs refs; needs > +29) | 1590 (140655) | +373 -403 =224, 48.5%, -10 ± 19: reverted | 1623 |
| F-L-d2mix-c | dormant | 1075 |  |  | 1762 | 45 | 1724 | 1746 (1744) | v1073 (v910) | 1621 | dormant (+18 ± 33 vs peak v910 before dormancy at v1068, at 1603 vs refs; needs > +33) | 1590 (136034) | +405 -409 =186, 49.8%, -1 ± 19: reverted | 1075 |
| F-L-d2jit-c | qualified | 3348 | 113 x 40 | 240 | 1828 | 49 | 1794 | 1800 | v1651 | 1686 | new peak (+28 ± 54 over v1651; predecessor +34) | 1590 (141300) | +320 -431 =249, 44.5%, -39 ± 19: reverted | 3348 |
| F-N-c | frozen | 1691 |  |  | 1397 | 38 | 1405 | 1477 (1477) | v1061 (v1061) | 1256 | frozen at ply 2 (avg 1398.2 over cycles 17–26; milestone 1400); dormant (-80 ± 28 vs peak v1061 before dormancy at v1683, at 1335 vs refs; needs > +28) |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1713 | 21 |  | 1721 | v300 | 1572 | frozen |  |  | 373 |
| FM-c | dormant | 578 |  |  | 1551 | 39 | 1580 | 1580 (1568) | v577 (v535) | 1410 | dormant (-17 ± 27 vs peak v535 before dormancy at v576, at 1427 vs refs; needs > +27) | 1590 (142242) | +488 -381 =131, 55.4%, +37 ± 20: kept | 579 |
| F-P-c | qualified | 353 | 187 x 1000 | 240 | 1492 | 38 | 1366 | 1366 | v165 | 1350 | new peak (+125 ± 40 over v165; predecessor +125) | 1590 (148476) | +325 -498 =177, 41.3%, -61 ± 20: reverted | 353 |
| T-L-Ne-c | qualified | 953 | 38 x 1000 | 241 | 1619 | 40 | 1613 | 1613 | v914 | 1478 | new peak (+6 ± 42 over v914; predecessor +6) | 1590 (140009) | +442 -396 =162, 52.3%, +16 ± 20: kept | 954 |
| T-L-Ne-merge-c | dormant | 1013 |  |  | 1673 | 22 |  | 1706 (1706) | v794 (v794) | 1532 | dormant (-33 ± 17 vs peak v794 before dormancy at v1008, at 1564 vs refs; needs > +17) | 1590 (126711) | +448 -412 =140, 51.8%, +12 ± 20: kept | 1014 |
| T-L-Ne-fresh-c | qualified | 289 | 69 x 1000 | 245 | 1553 | 39 | 1488 | 1488 | v219 | 1412 | new peak (+65 ± 41 over v219; predecessor +65) | 1590 (137505) | +529 -359 =112, 58.5%, +60 ± 21: kept | 290 |
| T-L-Ne-pre-c | qualified | 100 | 100 x 1000 | 241 | 1382 | 38 |  |  |  | 1241 | qualified (first rating) | 1590 (142168) | +622 -279 =99, 67.2%, +124 ± 22: kept | 101 |
| T-L-Nc-c | retired | 719 |  |  | 1610 | 40 | 1638 | 1648 (1648) | v516 (v516) | 1469 | retired (avg ply 1 972.6, ply 2 1609.3; no untaken milestone within 10); dormant (-38 ± 30 vs peak v516 before dormancy at v709, at 1507 vs refs; needs > +30) |  |  | 719 |
| T-L-Nc-merge-c | dormant | 1883 |  |  | 1723 | 43 | 1697 | 1728 (1728) | v1631 (v1631) | 1582 | dormant (-5 ± 32 vs peak v1631 before dormancy at v1878, at 1587 vs refs; needs > +32) | 1590 (130569) | +445 -417 =138, 51.4%, +10 ± 20: kept | 1884 |
| T-L-Nc-fresh-c | qualified | 473 | 44 x 1000 | 243 | 1580 | 39 | 1511 | 1550 | v348 | 1439 | new peak (+30 ± 41 over v348; predecessor +69) | 1590 (142793) | +468 -369 =163, 54.9%, +34 ± 20: kept | 474 |
| D-L-Nc-c | qualified | 53 | 53 x 1000 | 241 | 1156 | 36 |  |  |  | 1015 | qualified (first rating) | 1590 (155859) | +484 -376 =140, 55.4%, +38 ± 20: kept | 54 |
| T-L-LLs-c | frozen | 651 |  |  | 1613 | 16 |  | 1633 | v540 | 1472 | frozen |  |  | 651 |
| T-L-LLs-merge-c | dormant | 1759 |  |  | 1584 | 39 | 1579 | 1608 (1608) | v1251 (v1251) | 1443 | dormant (-24 ± 30 vs peak v1251 before dormancy at v1754, at 1467 vs refs; needs > +30) | 1590 (137730) | +448 -380 =172, 53.4%, +24 ± 20: kept | 1760 |
| T-L-NLs-c | qualified | 1389 | 44 x 1000 | 238 | 1524 | 39 | 1592 | 1592 | v1345 | 1383 | strike 1 (-67 ± 41 vs peak v1345; predecessor -67) | 1590 (137369) | +397 -444 =159, 47.6%, -16 ± 20: reverted | 1389 |
| T-L-NLs-merge-c | dormant | 1121 |  |  | 1666 | 41 | 1649 | 1653 (1653) | v641 (v641) | 1525 | dormant (+14 ± 40 vs peak v641 before dormancy at v1115, at 1512 vs refs; needs > +40) | 1590 (137081) | +422 -406 =172, 50.8%, +6 ± 20: kept | 1122 |
| T-L-NNs-c | qualified | 1366 | 46 x 1000 | 242 | 1608 | 40 | 1578 | 1606 | v561 | 1467 | new peak (+2 ± 47 over v561; predecessor +30) | 1590 (133602) | +405 -436 =159, 48.4%, -11 ± 20: reverted | 1366 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 973 | 16 |  | 1706 | v818 | 832 | frozen |  |  | 1070 |
| T-L-Le-c | qualified | 1035 | 44 x 1000 | 240 | 1539 | 39 | 1623 | 1623 | v990 | 1397 | strike 1 (-84 ± 42 vs peak v990; predecessor -84) | 1590 (132310) | +508 -366 =126, 57.1%, +50 ± 20: kept | 1036 |
| T-L-Le-merge-c | retired | 2053 |  |  | 1676 | 42 | 1671 | 1704 (1693) | v2051 (v1857) | 1535 | retired (avg ply 1 977.0, ply 2 1679.6; no untaken milestone within 10); dormant (-17 ± 31 vs peak v1857 before dormancy at v2049, at 1552 vs refs; needs > +31) |  |  | 2053 |
| T-L-Le-fresh-c | qualified | 260 | 46 x 1000 | 240 | 1522 | 39 | 1514 | 1514 | v213 | 1380 | new peak (+8 ± 41 over v213; predecessor +8) | 1590 (139800) | +430 -470 =100, 48.0%, -14 ± 20: reverted | 260 |
| T-L-Le-pre-c | qualified | 95 | 95 x 1000 | 241 | 1359 | 38 |  |  |  | 1217 | qualified (first rating) | 1590 (141692) | +597 -286 =117, 65.5%, +112 ± 21: kept | 96 |
| T-N-Ne-c | frozen | 637 |  |  | 1279 | 14 |  | 1372 | v574 | 1138 | frozen |  |  | 637 |
| T-N-Ne-merge-c | dormant | 907 |  |  | 1392 | 28 |  | 1448 (1448) | v699 (v699) | 1250 | dormant (-56 ± 21 vs peak v699 before dormancy at v904, at 1307 vs refs; needs > +21) | 1590 (132205) | +421 -403 =176, 50.9%, +6 ± 20: kept | 908 |
| T-N-Nc-c | qualified | 892 | 41 x 1000 | 238 | 1348 | 38 | 1354 | 1391 | v472 | 1206 | ok (-44 ± 46 vs peak v472; predecessor -6) | 1590 (132927) | +443 -404 =153, 51.9%, +14 ± 20: kept | 893 |
| T-N-Nc-merge-c | qualified | 1414 | 59 x 1000 | 241 | 1445 | 38 | 1409 | 1450 | v631 | 1304 | ok (-5 ± 37 vs peak v631; predecessor +36) | 1590 (128753) | +451 -374 =175, 53.8%, +27 ± 20: kept | 1415 |
| T-N-Nc-fresh-c | qualified | 297 | 61 x 1000 | 240 | 1374 | 38 | 1340 | 1340 | v235 | 1233 | new peak (+34 ± 40 over v235; predecessor +34) | 1590 (134312) | +402 -420 =178, 49.1%, -6 ± 20: reverted | 297 |
| T-N-NNs-c | dormant | 631 |  |  | 1145 | 22 |  | 1238 (1238) | v341 (v341) | 1004 | dormant (-93 ± 36 vs peak v341 before dormancy at v625, at 1097 vs refs; needs > +36) | 1590 (142975) | +424 -407 =169, 50.8%, +6 ± 20: kept | 632 |
| T-N-Le-merge-c | dormant | 754 |  |  | 1343 | 20 |  | 1431 (1431) | v700 (v700) | 1202 | dormant (-88 ± 21 vs peak v700 before dormancy at v754, at 1290 vs refs; needs > +21) | 1590 (133305) | +403 -466 =131, 46.9%, -22 ± 20: reverted | 754 |
| T-N-LLs-merge-c | dormant | 725 |  |  | 1116 | 15 |  | 1371 (1371) | v585 (v585) | 975 | dormant (-255 ± 21 vs peak v585 before dormancy at v723, at 1230 vs refs; needs > +21) | 1590 (140267) | +369 -487 =144, 44.1%, -41 ± 20: reverted | 725 |
| T-N-NLs-merge-c | dormant | 1031 |  |  | 1158 | 36 | 1131 | 1420 (1420) | v787 (v787) | 1017 | dormant (-262 ± 22 vs peak v787 before dormancy at v1030, at 1279 vs refs; needs > +22) | 1590 (143528) | +391 -434 =175, 47.9%, -15 ± 20: reverted | 1031 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 714 | 24 |  | 1349 | v270 | 572 | frozen |  |  | 417 |
| F-N-d2jit-c | qualified | 1650 | 126 x 40 | 241 | 1170 | 36 | 1220 | 1220 | v1523 | 1029 | strike 1 (-50 ± 39 vs peak v1523; predecessor -50) | 1590 (138988) | +402 -392 =206, 50.5%, +4 ± 19: kept | 1651 |
| T-N-NNs-merge-d2jit-c | qualified | 1471 | 98 x 40 | 240 | 1150 | 36 | 1150 | 1172 | v1107 | 1009 | ok (-22 ± 38 vs peak v1107; predecessor -1) | 1590 (142658) | +413 -389 =198, 51.2%, +8 ± 19: kept | 1472 |
| F-L-d4mix-c | qualified | 124 | 8 x 10 | 228 | 1660 | 41 | 1575 | 1610 | v106 | 1519 | new peak (+50 ± 42 over v106; predecessor +85) | 1590 (140300) | +518 -310 =172, 60.4%, +73 ± 20: kept | 125 |
| MoE-d2jit-c | qualified | 538 | 52 x 40 | 243 | 1634 | 40 | 1660 | 1660 | v486 | 1493 | ok (-26 ± 43 vs peak v486; predecessor -26) | 1590 (138498) | +413 -414 =173, 50.0%, -0 ± 20: reverted | 538 |
| T-L-H-c | dormant | 1526 |  |  | 1634 | 40 | 1643 | 1660 (1660) | v1245 (v1245) | 1493 | dormant (-25 ± 30 vs peak v1245 before dormancy at v1521, at 1518 vs refs; needs > +30) | 1590 (134412) | +399 -428 =173, 48.5%, -10 ± 20: reverted | 1526 |
