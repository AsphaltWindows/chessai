# Cycle 39

**Changes from this cycle** (deployed after cycle 38, `staged-c38/`), the user's ("throw them into the tournament league and see how they do against a more balanced field", rather than long equal-CPU arenas): F-P-tau0-d2jit-c joins, from scratch on `chessai-probit`: F-P-tau0-c's line plus `--depth 2`, one factor, self-play by depth-2 search, the classifier-by-depth question for probit (the league has LR and NB at depth 2 in F-L-d2jit-c and F-N-d2jit-c). On cycle 15's league games, probit trained on ply-2 games beat the same model trained on ply-1 games by 174 Elo at ply 2 (LR by 201, NB's 203 below), and in pure self-play from scratch at equal CPU probit at depth 2 closed on depth 1 fastest of the three classifiers. Tau 0, as at depth 2 the model sees 20-50 times fewer positions a cycle. Its own step trains and gates with `chessai-probit` (`own_games.binary_by_lineage`). Checked before the deploy, on `chessai-probit`: from scratch at depth 2, 6 rounds of 40 games ran (52 ms a game), with tau 0 in every model file; 3 rounds and `--resume` to 6 gave the same files as one run; its own step through the driver, from its v6 on saved league games, trained (`--start`, with `--probit-tau 0`) and gated (1000 games at depth 2, kept), and the kept v7 still has tau 0; and a round-robin as the driver runs it, with its entries, F-P-tau0-c's, F-P-c's and two random players, played all 56 games.

**Own-games step, pooled over the lineages: 25 gates, +10747 -9582 =4671, score 52.3%, Elo +16.2 ± 3.9; kept 15 of 25.**

All cycles so far: 1246 gates, +552889 -489269 =203842, score 52.6%, Elo +17.8 ± 0.6; kept 827 of 1246.

2026-11-07 11:40: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 40 players to retired.txt (their games stay in the fit), added 42.
Tournament: 5 double round-robins of the 61 active players (18300 games) in 5.0 min, 18300 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| T-L-Le-c | 9 of 10 | cycle 40 | 31–39 | 925.2 | 1573.2 | retire |
| F-L-d2jit-c | 8 of 10 | cycle 41 | 32–39 | 802.5 | 1783.9 | freeze 1 at 800 |
| T-N-NNs-merge-d2jit-c | 5 of 10 | cycle 44 | 35–39 | 584.9 | 1137.1 | retire |
| FM-c | 4 of 10 | cycle 45 | 36–39 | 895.5 | 1629.9 | freeze 1 at 900 |
| T-N-Nc-fresh-c | 3 of 10 | cycle 46 | 37–39 | 715.1 | 1294.4 | retire |
| F-L-d1play-c | 2 of 10 | cycle 47 | 38–39 | 975.0 | 1521.0 | retire |
| T-L-Le-fresh-c | 2 of 10 | cycle 47 | 38–39 | 884.8 | 1666.0 | retire |
| T-L-Ne-c | 0 of 10 | cycle 49 | - | - | - | retire |
| T-N-Nc-merge-c | 0 of 10 | cycle 49 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d1play-c | dormant | 3587 |  |  | 1526 | 46 | 1514 | 1634 (1634) | v3322 (v3322) | 1386 | dormant (-108 ± 34 vs peak v3322 before dormancy at v3585, at 1494 vs refs; needs > +34) | 1090 (98030) | +495 -352 =153, 57.1%, +50 ± 20: kept | 3588 |
| F-L-d2jit-c | dormant | 3943 |  |  | 1782 | 34 |  | 1810 (1810) | v3348 (v3348) | 1642 | dormant (-27 ± 30 vs peak v3348 before dormancy at v3941, at 1669 vs refs; needs > +30) | 1090 (95317) | +332 -418 =250, 45.7%, -30 ± 19: reverted | 3943 |
| F-N-c | frozen | 1691 |  |  | 1372 | 13 |  | 1473 | v1061 | 1232 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1697 | 13 |  | 1718 | v300 | 1557 | frozen |  |  | 373 |
| FM-c | dormant | 669 |  |  | 1644 | 48 | 1614 | 1662 (1662) | v626 (v626) | 1504 | dormant (-18 ± 35 vs peak v626 before dormancy at v666, at 1522 vs refs; needs > +35) | 1090 (97621) | +415 -454 =131, 48.0%, -14 ± 20: reverted | 669 |
| F-P-c | qualified | 2640 | 174 x 1000 | 240 | 1494 | 45 | 1561 | 1589 | v2138 | 1354 | strike 1 (-94 ± 50 vs peak v2138; predecessor -67) | 1090 (92667) | +358 -444 =198, 45.7%, -30 ± 19: reverted | 2640 |
| F-P-tau0-c | qualified | 339 | 182 x 1000 | 240 | 1520 | 46 | 1382 | 1382 | v156 | 1379 | new peak (+137 ± 48 over v156; predecessor +137) | 1090 (92306) | +414 -363 =223, 52.5%, +18 ± 19: kept | 340 |
| F-P-tau0-d2jit-c | qualified | 150 | 150 x 40 | 241 | 980 | 42 |  |  |  | 840 | qualified (first rating) | 1090 (112644) | +570 -114 =316, 72.8%, +171 ± 19: kept | 151 |
| T-L-Ne-c | dormant | 1425 | 36 x 1000 | 238 | 1560 | 46 | 1551 | 1619 | v1288 | 1420 | dormant (strike 2; -59 ± 51 vs peak v1288; predecessor +9) | 1090 (93532) | +391 -441 =168, 47.5%, -17 ± 20: reverted | 1425 |
| T-L-Ne-fresh-c | qualified | 796 | 37 x 1000 | 241 | 1595 | 47 | 1554 | 1609 | v633 | 1455 | ok (-14 ± 51 vs peak v633; predecessor +41) | 1090 (93076) | +392 -467 =141, 46.2%, -26 ± 20: reverted | 796 |
| T-L-Ne-pre-c | qualified | 624 | 38 x 1000 | 242 | 1582 | 47 | 1522 | 1579 | v548 | 1442 | new peak (+3 ± 50 over v548; predecessor +60) | 1090 (96557) | +441 -402 =157, 51.9%, +14 ± 20: kept | 625 |
| T-L-Nc-fresh-c | qualified | 1015 | 43 x 1000 | 239 | 1603 | 47 | 1550 | 1590 | v822 | 1463 | new peak (+14 ± 51 over v822; predecessor +54) | 1090 (92900) | +435 -394 =171, 52.0%, +14 ± 20: kept | 1016 |
| D-L-Nc-c | qualified | 601 | 45 x 1000 | 242 | 1529 | 46 | 1521 | 1553 | v365 | 1389 | ok (-24 ± 49 vs peak v365; predecessor +8) | 1090 (92431) | +485 -371 =144, 55.7%, +40 ± 20: kept | 602 |
| T-L-LLs-c | frozen | 651 |  |  | 1605 | 11 |  | 1630 | v540 | 1465 | frozen |  |  | 651 |
| T-L-NNs-c | qualified | 1902 | 45 x 1000 | 239 | 1613 | 47 | 1605 | 1623 | v1590 | 1473 | ok (-9 ± 50 vs peak v1590; predecessor +9) | 1090 (92804) | +397 -414 =189, 49.1%, -6 ± 19: reverted | 1902 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 964 | 10 |  | 1702 | v818 | 824 | frozen |  |  | 1070 |
| T-L-Le-c | dormant | 1175 |  |  | 1599 | 20 |  | 1674 (1674) | v1081 (v1081) | 1459 | dormant (-74 ± 24 vs peak v1081 before dormancy at v1171, at 1534 vs refs; needs > +24) | 1090 (93305) | +440 -405 =155, 51.7%, +12 ± 20: kept | 1176 |
| T-L-Le-fresh-c | dormant | 739 |  |  | 1686 | 50 | 1643 | 1653 (1653) | v682 (v682) | 1546 | dormant (+33 ± 35 vs peak v682 before dormancy at v737, at 1513 vs refs; needs > +35) | 1090 (92512) | +433 -441 =126, 49.6%, -3 ± 20: reverted | 739 |
| T-L-Le-pre-c | qualified | 600 | 35 x 1000 | 242 | 1508 | 45 | 1472 | 1519 | v529 | 1367 | ok (-12 ± 48 vs peak v529; predecessor +36) | 1090 (95288) | +433 -420 =147, 50.6%, +4 ± 20: kept | 601 |
| T-N-Ne-c | frozen | 637 |  |  | 1270 | 10 |  | 1369 | v574 | 1130 | frozen |  |  | 637 |
| T-N-Nc-c | qualified | 1400 | 40 x 1000 | 241 | 1379 | 45 | 1390 | 1390 | v1359 | 1239 | ok (-11 ± 47 vs peak v1359; predecessor -11) | 1090 (92967) | +414 -428 =158, 49.3%, -5 ± 20: reverted | 1400 |
| T-N-Nc-merge-c | dormant | 2126 | 45 x 1000 | 241 | 1393 | 45 | 1358 | 1447 | v631 | 1253 | dormant (strike 2; -54 ± 41 vs peak v631; predecessor +35) | 1090 (92584) | +426 -420 =154, 50.3%, +2 ± 20: kept | 2127 |
| T-N-Nc-fresh-c | dormant | 805 |  |  | 1292 | 32 |  | 1365 (1365) | v624 (v624) | 1151 | dormant (-74 ± 27 vs peak v624 before dormancy at v803, at 1225 vs refs; needs > +27) | 1090 (90378) | +442 -390 =168, 52.6%, +18 ± 20: kept | 806 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 716 | 12 |  | 1346 | v270 | 576 | frozen |  |  | 417 |
| F-N-d2jit-c | qualified | 2053 |  |  | 1297 | 44 | 1198 | 1256 (1256) | v1786 (v1786) | 1157 | re-qualified (+41 ± 31 vs peak v1786 before dormancy at v2046, at 1116 vs refs) | 1090 (95349) | +379 -384 =237, 49.8%, -2 ± 19: reverted | 2053 |
| T-N-NNs-merge-d2jit-c | dormant | 2115 |  |  | 1144 | 31 |  | 1167 (1167) | v1553 (v1553) | 1004 | dormant (-23 ± 24 vs peak v1553 before dormancy at v2112, at 1027 vs refs; needs > +24) | 1090 (99223) | +414 -367 =219, 52.3%, +16 ± 19: kept | 2116 |
| T-L-Le-merge-d2jit-c | qualified | 505 | 100 x 40 | 242 | 1599 | 47 | 1499 | 1499 | v404 | 1459 | new peak (+100 ± 49 over v404; predecessor +100) | 1090 (96772) | +517 -314 =169, 60.2%, +72 ± 20: kept | 506 |
| T-L-NLs-d2jit-c | qualified | 330 | 57 x 40 | 240 | 1210 | 44 | 1146 | 1146 | v272 | 1070 | new peak (+64 ± 46 over v272; predecessor +64) | 1090 (109823) | +456 -237 =307, 61.0%, +77 ± 18: kept | 331 |
| T-N-Nc-merge-d2jit-c | qualified | 557 | 115 x 40 | 241 | 1127 | 43 | 1129 | 1140 | v335 | 987 | ok (-14 ± 46 vs peak v335; predecessor -2) | 1090 (100397) | +425 -350 =225, 53.8%, +26 ± 19: kept | 558 |
| F-L-d4mix-c | qualified | 233 | 7 x 10 | 228 | 1690 | 50 | 1726 | 1750 | v200 | 1549 | strike 1 (-61 ± 56 vs peak v200; predecessor -37) | 1090 (89425) | +401 -446 =153, 47.8%, -16 ± 20: reverted | 233 |
| MoE-d2jit-c | qualified | 1113 | 35 x 40 | 240 | 1696 | 50 | 1719 | 1721 | v882 | 1556 | ok (-25 ± 55 vs peak v882; predecessor -23) | 1090 (95605) | +442 -346 =212, 54.8%, +34 ± 19: kept | 1114 |
