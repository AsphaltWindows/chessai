# Cycle 40

**Changes from this cycle** (deployed after cycle 39, `staged-c39/`), the user's: T-L-NLs-merge-d2jit-c joins, from scratch on `chessai-cyc5`: T-L-NLs-d2jit-c's line plus `--revise 0` (where T-L-NLs-merge-c's line has it), at the same split threshold of 100,000, so the depth-2 pair differs only in merging. The user reasons that merging beats not merging, which holds in 8 of the 9 tree pairs by peak (T-L-LLs about level). At depth 0 T-L-NLs-merge-c reverted 7 of its 8 splits and kept the 8th, peaking at 1658 with about 7 leaves against T-L-NLs-c's 1585, and T-L-NLs-d2jit-c has kept the split it made at v18; the pair tests whether the merge rule keeps any of its depth-2 splits. (A correction to cycle 35's twin note in `lineages.txt` and the README: T-L-Le-merge-c is not a real tree. It reverted 25 of its 26 splits and was one leaf in 71% of its versions, and at depth 2 T-L-Le-merge-d2jit-c has reverted all 5 of its splits so far.) Checked before the deploy, on `chessai-cyc5`: from scratch at depth 2, its first 6 versions equal T-L-NLs-d2jit-c's check run's (no split yet); at a split threshold of 20,000 it split at v3 like T-L-NLs-d2jit-c and merged back at v5, and 4 rounds plus `--resume` to 8 gave the same files as 8 rounds; its own step through the driver trained and gated (1,000 games at depth 2, kept); and a round-robin with its entries played all 30 games on `chessai-probit`.

**Own-games step, pooled over the lineages: 25 gates, +10818 -9505 =4677, score 52.6%, Elo +18.3 ± 3.9; kept 18 of 25.**

All cycles so far: 1271 gates, +563707 -498774 =208519, score 52.6%, Elo +17.8 ± 0.6; kept 845 of 1271.

2026-11-07 12:06: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 42 players to retired.txt (their games stay in the fit), added 44.
Tournament: 5 double round-robins of the 63 active players (19530 games) in 5.5 min, 19530 games saved.
Freeze/retire rule, after 10 cycles dormant: T-L-Le-c retired (avg ply 1 923.1, ply 2 1575.8; no untaken milestone within 10). 2 entries leave the field for retired.txt (their games stay in the fit): 63 players become 61, 19,530 games a cycle become 18,300.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| F-L-d2jit-c | 9 of 10 | cycle 41 | 32–40 | 803.6 | 1783.5 | freeze 1 at 800 |
| FM-c | 5 of 10 | cycle 45 | 36–40 | 893.7 | 1626.9 | freeze 1 at 900 |
| T-N-Nc-fresh-c | 4 of 10 | cycle 46 | 37–40 | 721.2 | 1300.6 | retire |
| F-L-d1play-c | 3 of 10 | cycle 47 | 38–40 | 990.1 | 1522.4 | retire |
| T-L-Le-fresh-c | 3 of 10 | cycle 47 | 38–40 | 892.5 | 1658.6 | retire |
| T-L-Ne-c | 1 of 10 | cycle 49 | 40–40 | 1021.9 | 1560.2 | retire |
| T-N-Nc-merge-c | 1 of 10 | cycle 49 | 40–40 | 763.9 | 1461.0 | retire |
| F-P-c | 0 of 10 | cycle 50 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d1play-c | dormant | 3588 |  |  | 1525 | 44 | 1520 | 1633 (1633) | v3322 (v3322) | 1386 | dormant (-108 ± 33 vs peak v3322 before dormancy at v3585, at 1494 vs refs; needs > +33) | 1130 (103553) | +425 -417 =158, 50.4%, +3 ± 20: kept | 3589 |
| F-L-d2jit-c | dormant | 3943 |  |  | 1780 | 29 |  | 1808 (1808) | v3348 (v3348) | 1641 | dormant (-28 ± 28 vs peak v3348 before dormancy at v3941, at 1669 vs refs; needs > +28) | 1130 (100629) | +356 -394 =250, 48.1%, -13 ± 19: reverted | 3943 |
| F-N-c | frozen | 1691 |  |  | 1370 | 12 |  | 1472 | v1061 | 1230 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1696 | 12 |  | 1716 | v300 | 1556 | frozen |  |  | 373 |
| FM-c | dormant | 669 |  |  | 1615 | 34 |  | 1660 (1660) | v626 (v626) | 1476 | dormant (-46 ± 30 vs peak v626 before dormancy at v666, at 1521 vs refs; needs > +30) | 1130 (101015) | +394 -437 =169, 47.9%, -15 ± 20: reverted | 669 |
| F-P-c | dormant | 2801 | 161 x 1000 | 240 | 1466 | 44 | 1488 | 1587 | v2138 | 1327 | dormant (strike 2; -121 ± 49 vs peak v2138; predecessor -22) | 1130 (103560) | +429 -403 =168, 51.3%, +9 ± 20: kept | 2802 |
| F-P-tau0-c | qualified | 511 | 171 x 1000 | 240 | 1512 | 44 | 1513 | 1513 | v339 | 1373 | ok (-1 ± 47 vs peak v339; predecessor -1) | 1130 (93704) | +411 -379 =210, 51.6%, +11 ± 19: kept | 512 |
| F-P-tau0-d2jit-c | qualified | 284 | 133 x 40 | 241 | 1565 | 45 | 979 | 979 | v150 | 1426 | new peak (+586 ± 46 over v150; predecessor +586) | 1130 (99903) | +389 -295 =316, 54.7%, +33 ± 18: kept | 285 |
| T-L-Ne-c | dormant | 1425 |  |  | 1560 | 33 |  | 1617 (1617) | v1288 (v1288) | 1421 | dormant (-57 ± 29 vs peak v1288 before dormancy at v1425, at 1478 vs refs; needs > +29) | 1130 (97860) | +419 -426 =155, 49.6%, -2 ± 20: reverted | 1425 |
| T-L-Ne-fresh-c | qualified | 824 | 28 x 1000 | 241 | 1604 | 46 | 1588 | 1607 | v633 | 1465 | ok (-3 ± 50 vs peak v633; predecessor +16) | 1130 (100180) | +457 -385 =158, 53.6%, +25 ± 20: kept | 825 |
| T-L-Ne-pre-c | qualified | 656 | 31 x 1000 | 243 | 1547 | 45 | 1576 | 1578 | v548 | 1408 | ok (-30 ± 49 vs peak v548; predecessor -29) | 1130 (101732) | +431 -409 =160, 51.1%, +8 ± 20: kept | 657 |
| T-L-Nc-fresh-c | qualified | 1055 | 39 x 1000 | 241 | 1596 | 46 | 1597 | 1597 | v1015 | 1456 | ok (-1 ± 49 vs peak v1015; predecessor -1) | 1130 (97951) | +416 -413 =171, 50.1%, +1 ± 20: kept | 1056 |
| D-L-Nc-c | qualified | 643 | 41 x 1000 | 244 | 1535 | 45 | 1523 | 1552 | v365 | 1396 | ok (-16 ± 49 vs peak v365; predecessor +13) | 1130 (97412) | +472 -385 =143, 54.4%, +30 ± 20: kept | 644 |
| T-L-LLs-c | frozen | 651 |  |  | 1603 | 10 |  | 1628 | v540 | 1464 | frozen |  |  | 651 |
| T-L-NNs-c | qualified | 1937 | 35 x 1000 | 238 | 1590 | 46 | 1606 | 1621 | v1590 | 1451 | ok (-31 ± 49 vs peak v1590; predecessor -16) | 1130 (93389) | +412 -415 =173, 49.9%, -1 ± 20: reverted | 1937 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 961 | 10 |  | 1701 | v818 | 822 | frozen |  |  | 1070 |
| T-L-Le-c | retired | 1176 |  |  | 1598 | 46 | 1597 | 1672 (1672) | v1081 (v1081) | 1459 | retired (avg ply 1 923.1, ply 2 1575.8; no untaken milestone within 10); dormant (-74 ± 32 vs peak v1081 before dormancy at v1171, at 1533 vs refs; needs > +32) |  |  | 1176 |
| T-L-Le-fresh-c | dormant | 739 |  |  | 1644 | 34 |  | 1651 (1651) | v682 (v682) | 1505 | dormant (-8 ± 30 vs peak v682 before dormancy at v737, at 1512 vs refs; needs > +30) | 1130 (99124) | +388 -455 =157, 46.7%, -23 ± 20: reverted | 739 |
| T-L-Le-pre-c | qualified | 634 | 33 x 1000 | 242 | 1506 | 44 | 1502 | 1518 | v529 | 1367 | ok (-12 ± 48 vs peak v529; predecessor +5) | 1130 (97406) | +465 -428 =107, 51.8%, +13 ± 20: kept | 635 |
| T-N-Ne-c | frozen | 637 |  |  | 1269 | 10 |  | 1368 | v574 | 1130 | frozen |  |  | 637 |
| T-N-Nc-c | qualified | 1431 | 31 x 1000 | 243 | 1350 | 44 | 1374 | 1389 | v1359 | 1211 | ok (-38 ± 47 vs peak v1359; predecessor -24) | 1130 (96180) | +400 -437 =163, 48.1%, -13 ± 20: reverted | 1431 |
| T-N-Nc-merge-c | dormant | 2127 |  |  | 1461 | 44 | 1388 | 1445 (1445) | v631 (v631) | 1322 | dormant (+16 ± 27 vs peak v631 before dormancy at v2126, at 1306 vs refs; needs > +27) | 1130 (94984) | +420 -415 =165, 50.2%, +2 ± 20: kept | 2128 |
| T-N-Nc-fresh-c | dormant | 806 |  |  | 1319 | 44 | 1289 | 1364 (1364) | v624 (v624) | 1180 | dormant (-44 ± 31 vs peak v624 before dormancy at v803, at 1224 vs refs; needs > +31) | 1130 (94879) | +420 -418 =162, 50.1%, +1 ± 20: kept | 807 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 715 | 12 |  | 1344 | v270 | 576 | frozen |  |  | 417 |
| F-N-d2jit-c | qualified | 2165 | 112 x 40 | 240 | 1257 | 44 | 1294 | 1294 | v2053 | 1118 | ok (-36 ± 46 vs peak v2053; predecessor -36) | 1130 (102712) | +391 -390 =219, 50.0%, +0 ± 19: kept | 2166 |
| T-N-NNs-merge-d2jit-c | qualified | 2116 |  |  | 1207 | 43 | 1142 | 1165 (1165) | v1553 (v1553) | 1068 | re-qualified (+42 ± 29 vs peak v1553 before dormancy at v2112, at 1026 vs refs) | 1130 (100706) | +391 -397 =212, 49.7%, -2 ± 19: reverted | 2116 |
| T-L-Le-merge-d2jit-c | qualified | 599 | 93 x 40 | 239 | 1644 | 47 | 1592 | 1592 | v505 | 1505 | new peak (+52 ± 49 over v505; predecessor +52) | 1130 (99723) | +480 -358 =162, 56.1%, +43 ± 20: kept | 600 |
| T-L-NLs-d2jit-c | qualified | 383 | 52 x 40 | 239 | 1239 | 43 | 1208 | 1208 | v330 | 1100 | new peak (+31 ± 46 over v330; predecessor +31) | 1130 (115328) | +424 -310 =266, 55.7%, +40 ± 18: kept | 384 |
| T-N-Nc-merge-d2jit-c | qualified | 655 | 97 x 40 | 240 | 1174 | 43 | 1125 | 1139 | v335 | 1035 | new peak (+35 ± 46 over v335; predecessor +49) | 1130 (102417) | +427 -349 =224, 53.9%, +27 ± 19: kept | 656 |
| T-L-NLs-merge-d2jit-c | qualified | 103 | 103 x 40 | 242 | 819 | 43 |  |  |  | 680 | qualified (first rating) | 1130 (121276) | +715 -56 =229, 83.0%, +275 ± 22: kept | 104 |
| F-L-d4mix-c | qualified | 240 | 7 x 10 | 242 | 1726 | 51 | 1682 | 1748 | v200 | 1587 | ok (-22 ± 57 vs peak v200; predecessor +44) | 1130 (97937) | +456 -385 =159, 53.5%, +25 ± 20: kept | 241 |
| MoE-d2jit-c | qualified | 1152 | 38 x 40 | 238 | 1649 | 47 | 1689 | 1720 | v882 | 1509 | strike 1 (-71 ± 54 vs peak v882; predecessor -40) | 1130 (104080) | +430 -349 =221, 54.0%, +28 ± 19: kept | 1153 |
