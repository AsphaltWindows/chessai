# Cycle 25

**Changes from this cycle** (deployed after cycle 24, `staged-c24/`), the user's: (1) F-P-c joins, from scratch: F-P, an online Bayesian ordinal probit regression over the logistic regression's features (`--model probit`), with F-L-c's recipe otherwise, on `chessai-probit`, which its own step trains and gates with too (`own_games.binary_by_lineage`). (2) The tournament plays with `chessai-probit` instead of `chessai-cyc5`, which can't load a probit entry. Checked before the deploy: it replayed cycle 22's first round-robin (85 players: lr, cnbp, bdt, ht, tan, moe, fm and the random players at plies 0-4, calibrated as in the league) with every game's result and length as `chessai-cyc5` had played them, and the 1,002 games of cycle 23's three fresh twins (trees `chessai-resid4` trained) in cycle 23's first round-robin the same, saving each game byte for byte as the league had; and it rewrites the league's ratings, head-to-head tables and Elo history byte for byte. (3) The own-games step counts the games against an entry a freeze at the cycle's decision takes out of the field, as against every lineage entry that played: in cycles 15 and 21 every own step had left out 20 games per such entry. Replaying cycle 21's own steps of F-L-c and F-N-c changed only the game set: 1,610 games instead of 1,570, the 40 against the two entries frozen out, with the same commands (the old code gave cycle 21's games, positions, gates and models byte for byte).

**Own-games step, pooled over the lineages: 35 gates, +15076 -14170 =5754, score 51.3%, Elo +9.0 ± 3.3; kept 25 of 35.**

All cycles so far: 885 gates, +397392 -344954 =142654, score 53.0%, Elo +20.6 ± 0.7; kept 595 of 885.

2026-11-07 05:33: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 52 players to retired.txt (their games stay in the fit), added 54.
Tournament: 5 double round-robins of the 80 active players (31600 games) in 4.0 min, 31600 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| F-N-c | 9 of 10 | cycle 26 | 17–25 | 732.9 | 1398.4 | freeze 2 at 1400 |
| T-L-Le-merge-c | 9 of 10 | cycle 26 | 17–25 | 976.1 | 1680.0 | retire |
| T-L-Nc-c | 9 of 10 | cycle 26 | 17–25 | 976.3 | 1609.3 | retire |
| T-L-H-c | 8 of 10 | cycle 27 | 18–25 | 987.4 | 1611.5 | retire |
| T-L-NLs-merge-c | 8 of 10 | cycle 27 | 18–25 | 935.1 | 1599.8 | retire |
| T-L-Ne-merge-c | 8 of 10 | cycle 27 | 18–25 | 971.8 | 1673.8 | retire |
| T-N-LLs-merge-c | 8 of 10 | cycle 27 | 18–25 | 625.2 | 1123.2 | retire |
| T-N-NNs-c | 8 of 10 | cycle 27 | 18–25 | 647.8 | 1130.5 | retire |
| T-N-Ne-merge-c | 7 of 10 | cycle 28 | 19–25 | 746.9 | 1401.6 | retire |
| F-L-d2mix-c | 6 of 10 | cycle 29 | 20–25 | 840.1 | 1725.6 | retire |
| T-L-LLs-merge-c | 6 of 10 | cycle 29 | 20–25 | 927.1 | 1547.5 | retire |
| T-L-Nc-merge-c | 6 of 10 | cycle 29 | 20–25 | 968.8 | 1658.6 | retire |
| F-L-d2play-c | 4 of 10 | cycle 31 | 22–25 | 817.9 | 1698.8 | retire |
| F-L-d0soft-c | 3 of 10 | cycle 32 | 23–25 | 983.1 | 1635.2 | retire |
| T-N-Le-merge-c | 2 of 10 | cycle 33 | 24–25 | 730.3 | 1348.0 | retire |
| FM-c | 1 of 10 | cycle 34 | 25–25 | 884.9 | 1584.5 | retire |
| F-L-c | 0 of 10 | cycle 35 | - | - | - | retire |
| T-N-NLs-merge-c | 0 of 10 | cycle 35 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | dormant | 3138 | 149 x 1000 | 240 | 1644 | 42 | 1629 | 1719 | v2860 | 1503 | dormant (strike 2; -75 ± 38 vs peak v2860; predecessor +15) | 1470 (120964) | +515 -335 =150, 59.0%, +63 ± 20: kept | 3139 |
| F-L-d0soft-c | dormant | 2363 |  |  | 1654 | 30 |  | 1703 (1703) | v2014 (v2014) | 1513 | dormant (-49 ± 26 vs peak v2014 before dormancy at v2361, at 1562 vs refs; needs > +26) | 1470 (118761) | +416 -436 =148, 49.0%, -7 ± 20: reverted | 2363 |
| F-L-d1play-c | qualified | 3170 | 164 x 860 | 241 | 1539 | 40 | 1603 | 1603 | v3006 | 1398 | strike 1 (-64 ± 41 vs peak v3006; predecessor -64) | 1470 (135040) | +482 -361 =157, 56.0%, +42 ± 20: kept | 3171 |
| F-L-d2play-c | dormant | 1622 |  |  | 1716 | 44 | 1689 | 1754 (1754) | v1011 (v1011) | 1575 | dormant (-38 ± 29 vs peak v1011 before dormancy at v1620, at 1613 vs refs; needs > +29) | 1470 (123746) | +427 -367 =206, 53.0%, +21 ± 19: kept | 1623 |
| F-L-d2mix-c | dormant | 1074 |  |  | 1728 | 44 | 1748 | 1748 (1744) | v1073 (v910) | 1587 | dormant (-16 ± 33 vs peak v910 before dormancy at v1068, at 1603 vs refs; needs > +33) | 1470 (125304) | +448 -371 =181, 53.8%, +27 ± 20: kept | 1075 |
| F-L-d2jit-c | qualified | 3235 | 118 x 40 | 240 | 1798 | 48 | 1728 | 1800 | v1651 | 1657 | ok (-2 ± 54 vs peak v1651; predecessor +70) | 1470 (133244) | +379 -380 =241, 50.0%, -0 ± 19: reverted | 3235 |
| F-N-c | dormant | 1690 |  |  | 1408 | 23 |  | 1477 (1477) | v1061 (v1061) | 1267 | dormant (-69 ± 24 vs peak v1061 before dormancy at v1683, at 1335 vs refs; needs > +24) | 1470 (121849) | +418 -414 =168, 50.2%, +1 ± 20: kept | 1691 |
| MoE-c | frozen | 373 |  |  | 1718 | 24 |  | 1721 | v300 | 1577 | frozen |  |  | 373 |
| FM-c | dormant | 577 |  |  | 1584 | 40 | 1318 | 1569 (1569) | v535 (v535) | 1443 | dormant (+15 ± 28 vs peak v535 before dormancy at v576, at 1428 vs refs; needs > +28) | 1470 (135659) | +453 -409 =138, 52.2%, +15 ± 20: kept | 578 |
| F-P-c | qualified | 165 | 165 x 1000 | 240 | 1370 | 39 |  |  |  | 1229 | qualified (first rating) | 1470 (140080) | +600 -226 =174, 68.7%, +137 ± 21: kept | 166 |
| T-L-Ne-c | qualified | 914 | 39 x 1000 | 238 | 1617 | 41 | 1609 | 1609 | v556 | 1476 | new peak (+8 ± 36 over v556; predecessor +8) | 1470 (128535) | +450 -402 =148, 52.4%, +17 ± 20: kept | 915 |
| T-L-Ne-merge-c | dormant | 1013 |  |  | 1678 | 26 |  | 1706 (1706) | v794 (v794) | 1537 | dormant (-27 ± 18 vs peak v794 before dormancy at v1008, at 1565 vs refs; needs > +18) | 1470 (118625) | +411 -463 =126, 47.4%, -18 ± 20: reverted | 1013 |
| T-L-Ne-fresh-c | qualified | 219 | 54 x 1000 | 241 | 1492 | 40 | 1468 | 1468 | v164 | 1351 | new peak (+24 ± 40 over v164; predecessor +24) | 1470 (128055) | +470 -384 =146, 54.3%, +30 ± 20: kept | 220 |
| T-L-Nc-c | dormant | 718 |  |  | 1642 | 42 | 1596 | 1648 (1648) | v516 (v516) | 1501 | dormant (-6 ± 31 vs peak v516 before dormancy at v709, at 1507 vs refs; needs > +31) | 1470 (126943) | +429 -408 =163, 51.0%, +7 ± 20: kept | 719 |
| T-L-Nc-merge-c | dormant | 1882 |  |  | 1702 | 43 | 1665 | 1729 (1729) | v1631 (v1631) | 1560 | dormant (-27 ± 32 vs peak v1631 before dormancy at v1878, at 1588 vs refs; needs > +32) | 1470 (121580) | +450 -401 =149, 52.4%, +17 ± 20: kept | 1883 |
| T-L-Nc-fresh-c | qualified | 428 | 43 x 1000 | 239 | 1515 | 40 | 1529 | 1552 | v348 | 1374 | ok (-37 ± 41 vs peak v348; predecessor -14) | 1470 (126900) | +435 -390 =175, 52.2%, +16 ± 20: kept | 429 |
| T-L-LLs-c | frozen | 651 |  |  | 1612 | 17 |  | 1633 | v540 | 1471 | frozen |  |  | 651 |
| T-L-LLs-merge-c | dormant | 1758 |  |  | 1584 | 40 | 1587 | 1608 (1608) | v1251 (v1251) | 1442 | dormant (-25 ± 30 vs peak v1251 before dormancy at v1754, at 1467 vs refs; needs > +30) | 1470 (127774) | +433 -382 =185, 52.5%, +18 ± 20: kept | 1759 |
| T-L-NLs-c | qualified | 1345 | 47 x 1000 | 240 | 1596 | 41 | 1534 | 1569 | v571 | 1455 | new peak (+27 ± 47 over v571; predecessor +62) | 1470 (129576) | +273 -569 =158, 35.2%, -106 ± 20: reverted | 1345 |
| T-L-NLs-merge-c | dormant | 1120 |  |  | 1653 | 42 | 1612 | 1653 (1653) | v641 (v641) | 1512 | dormant (+0 ± 40 vs peak v641 before dormancy at v1115, at 1512 vs refs; needs > +40) | 1470 (122092) | +425 -403 =172, 51.1%, +8 ± 20: kept | 1121 |
| T-L-NNs-c | qualified | 1319 | 45 x 1000 | 238 | 1582 | 40 | 1580 | 1606 | v561 | 1441 | ok (-23 ± 47 vs peak v561; predecessor +2) | 1470 (123691) | +436 -414 =150, 51.1%, +8 ± 20: kept | 1320 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 973 | 18 |  | 1706 | v1067 | 832 | frozen |  |  | 1070 |
| T-L-Le-c | qualified | 990 |  |  | 1627 | 41 | 1581 | 1589 (1589) | v696 (v696) | 1486 | re-qualified (+37 ± 30 vs peak v696 before dormancy at v985, at 1448 vs refs) | 1470 (124489) | +427 -392 =181, 51.7%, +12 ± 20: kept | 991 |
| T-L-Le-merge-c | dormant | 2052 |  |  | 1673 | 18 |  | 1705 (1693) | v2051 (v1857) | 1532 | dormant (-20 ± 24 vs peak v1857 before dormancy at v2049, at 1552 vs refs; needs > +24) | 1470 (119095) | +445 -416 =139, 51.4%, +10 ± 20: kept | 2053 |
| T-L-Le-fresh-c | qualified | 213 | 52 x 1000 | 242 | 1518 | 40 | 1417 | 1417 | v160 | 1377 | new peak (+101 ± 40 over v160; predecessor +101) | 1470 (126087) | +465 -443 =92, 51.1%, +8 ± 20: kept | 214 |
| T-N-Ne-c | frozen | 637 |  |  | 1281 | 15 |  | 1372 | v574 | 1140 | frozen |  |  | 637 |
| T-N-Ne-merge-c | dormant | 907 |  |  | 1426 | 39 | 1413 | 1448 (1448) | v699 (v699) | 1285 | dormant (-22 ± 25 vs peak v699 before dormancy at v904, at 1307 vs refs; needs > +25) | 1470 (119088) | +410 -417 =173, 49.6%, -2 ± 20: reverted | 907 |
| T-N-Nc-c | qualified | 850 | 41 x 1000 | 242 | 1357 | 39 | 1371 | 1391 | v472 | 1216 | ok (-34 ± 47 vs peak v472; predecessor -14) | 1470 (120983) | +429 -408 =163, 51.0%, +7 ± 20: kept | 851 |
| T-N-Nc-merge-c | qualified | 1355 | 59 x 1000 | 241 | 1413 | 39 | 1430 | 1450 | v631 | 1272 | ok (-37 ± 37 vs peak v631; predecessor -16) | 1470 (119052) | +381 -439 =180, 47.1%, -20 ± 20: reverted | 1355 |
| T-N-Nc-fresh-c | qualified | 235 | 60 x 1000 | 241 | 1343 | 39 | 1317 | 1317 | v174 | 1202 | new peak (+26 ± 40 over v174; predecessor +26) | 1470 (121728) | +429 -410 =161, 50.9%, +7 ± 20: kept | 236 |
| T-N-NNs-c | dormant | 631 |  |  | 1138 | 27 |  | 1238 (1238) | v341 (v341) | 997 | dormant (-100 ± 37 vs peak v341 before dormancy at v625, at 1097 vs refs; needs > +37) | 1470 (129116) | +371 -413 =216, 47.9%, -15 ± 19: reverted | 631 |
| T-N-Le-merge-c | dormant | 754 |  |  | 1348 | 23 |  | 1432 (1432) | v700 (v700) | 1207 | dormant (-84 ± 21 vs peak v700 before dormancy at v754, at 1291 vs refs; needs > +21) | 1470 (120203) | +399 -483 =118, 45.8%, -29 ± 20: reverted | 754 |
| T-N-LLs-merge-c | dormant | 725 |  |  | 1120 | 16 |  | 1371 (1371) | v585 (v585) | 979 | dormant (-251 ± 21 vs peak v585 before dormancy at v723, at 1230 vs refs; needs > +21) | 1470 (133734) | +370 -477 =153, 44.6%, -37 ± 20: reverted | 725 |
| T-N-NLs-merge-c | dormant | 1030 | 66 x 1000 | 239 | 1133 | 38 | 1305 | 1421 | v787 | 992 | dormant (strike 2; -288 ± 34 vs peak v787; predecessor -172) | 1470 (130407) | +430 -387 =183, 52.1%, +15 ± 20: kept | 1031 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 717 | 28 |  | 1349 | v270 | 576 | frozen |  |  | 417 |
| F-N-d2jit-c | qualified | 1523 | 122 x 40 | 241 | 1223 | 38 | 1208 | 1208 | v1400 | 1082 | new peak (+15 ± 39 over v1400; predecessor +15) | 1470 (127185) | +438 -367 =195, 53.5%, +25 ± 19: kept | 1524 |
| T-N-NNs-merge-d2jit-c | qualified | 1372 | 94 x 40 | 242 | 1153 | 38 | 1148 | 1173 | v1107 | 1011 | ok (-21 ± 39 vs peak v1107; predecessor +4) | 1470 (128228) | +413 -382 =205, 51.5%, +11 ± 19: kept | 1373 |
| F-L-d4mix-c | qualified | 115 | 8 x 10 | 245 | 1579 | 40 | 1612 | 1612 | v106 | 1438 | ok (-33 ± 42 vs peak v106; predecessor -33) | 1470 (130481) | +541 -328 =131, 60.7%, +75 ± 20: kept | 116 |
| MoE-d2jit-c | qualified | 486 | 40 x 40 | 239 | 1664 | 42 | 1561 | 1598 | v400 | 1523 | new peak (+66 ± 43 over v400; predecessor +104) | 1470 (124450) | +343 -476 =181, 43.4%, -46 ± 20: reverted | 486 |
| T-L-H-c | dormant | 1525 |  |  | 1646 | 30 |  | 1660 (1660) | v1245 (v1245) | 1505 | dormant (-13 ± 27 vs peak v1245 before dormancy at v1521, at 1518 vs refs; needs > +27) | 1470 (121775) | +435 -417 =148, 50.9%, +6 ± 20: kept | 1526 |
