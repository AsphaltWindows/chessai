# Cycle 43

**Own-games step, pooled over the lineages: 24 gates, +10140 -9504 =4356, score 51.3%, Elo +9.2 ± 4.0; kept 17 of 24.**

All cycles so far: 1343 gates, +594749 -526567 =221684, score 52.5%, Elo +17.7 ± 0.5; kept 894 of 1343.

2026-11-07 13:27: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 34 players to retired.txt (their games stay in the fit), added 34.
Tournament: 5 double round-robins of the 60 active players (17700 games) in 6.8 min, 17700 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| FM-c | 8 of 10 | cycle 45 | 36–43 | 891.4 | 1620.6 | freeze 1 at 900 |
| T-N-Nc-fresh-c | 7 of 10 | cycle 46 | 37–43 | 724.4 | 1299.2 | retire |
| F-L-d1play-c | 6 of 10 | cycle 47 | 38–43 | 990.8 | 1538.0 | retire |
| T-L-Le-fresh-c | 6 of 10 | cycle 47 | 38–43 | 899.8 | 1651.6 | retire |
| T-L-Ne-c | 4 of 10 | cycle 49 | 40–43 | 1021.1 | 1560.9 | retire |
| T-N-Nc-merge-c | 4 of 10 | cycle 49 | 40–43 | 761.9 | 1391.5 | retire |
| F-P-c | 3 of 10 | cycle 50 | 41–43 | 932.7 | 1488.7 | retire |
| F-N-d2jit-c | 1 of 10 | cycle 52 | 43–43 | 675.5 | 1187.0 | retire |
| T-N-Nc-c | 1 of 10 | cycle 52 | 43–43 | 749.3 | 1323.9 | retire |
| T-L-Le-pre-c | 0 of 10 | cycle 53 | - | - | - | retire |
| T-L-Ne-pre-c | 0 of 10 | cycle 53 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d1play-c | dormant | 3591 |  |  | 1571 | 46 | 1562 | 1632 (1632) | v3322 (v3322) | 1430 | dormant (-61 ± 34 vs peak v3322 before dormancy at v3585, at 1491 vs refs; needs > +34) | 1070 (97228) | +446 -409 =145, 51.8%, +13 ± 20: kept | 3592 |
| F-L-d2jit-c | frozen | 3943 |  |  | 817 | 18 |  | 1807 | v3348 | 676 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1373 | 12 |  | 1473 | v1061 | 1232 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1691 | 12 |  | 1718 | v300 | 1550 | frozen |  |  | 373 |
| FM-c | dormant | 669 |  |  | 1613 | 22 |  | 1659 (1659) | v626 (v626) | 1473 | dormant (-46 ± 27 vs peak v626 before dormancy at v666, at 1519 vs refs; needs > +27) | 1070 (95152) | +336 -514 =150, 41.1%, -62 ± 20: reverted | 669 |
| F-P-c | dormant | 2803 |  |  | 1497 | 32 |  | 1586 (1586) | v2138 (v2138) | 1356 | dormant (-89 ± 29 vs peak v2138 before dormancy at v2801, at 1446 vs refs; needs > +29) | 1070 (97414) | +420 -404 =176, 50.8%, +6 ± 20: kept | 2804 |
| F-P-tau0-c | qualified | 1039 | 176 x 1000 | 241 | 1542 | 45 | 1568 | 1568 | v862 | 1402 | ok (-26 ± 48 vs peak v862; predecessor -26) | 1070 (88213) | +401 -377 =222, 51.2%, +8 ± 19: kept | 1040 |
| F-P-tau0-d2jit-c | qualified | 701 | 138 x 40 | 240 | 1699 | 50 | 1672 | 1672 | v563 | 1558 | new peak (+27 ± 53 over v563; predecessor +27) | 1070 (95660) | +335 -338 =327, 49.9%, -1 ± 18: reverted | 701 |
| T-L-Ne-c | dormant | 1425 |  |  | 1568 | 22 |  | 1616 (1616) | v1288 (v1288) | 1428 | dormant (-48 ± 27 vs peak v1288 before dormancy at v1425, at 1476 vs refs; needs > +27) | 1070 (94909) | +397 -449 =154, 47.4%, -18 ± 20: reverted | 1425 |
| T-L-Ne-fresh-c | qualified | 926 | 32 x 1000 | 240 | 1589 | 46 | 1536 | 1606 | v633 | 1448 | ok (-17 ± 50 vs peak v633; predecessor +52) | 1070 (90131) | +441 -408 =151, 51.6%, +12 ± 20: kept | 927 |
| T-L-Ne-pre-c | dormant | 752 | 29 x 1000 | 242 | 1525 | 45 | 1522 | 1579 | v691 | 1384 | dormant (strike 2; -55 ± 48 vs peak v691; predecessor +3) | 1070 (94258) | +477 -356 =167, 56.0%, +42 ± 20: kept | 753 |
| T-L-Nc-fresh-c | qualified | 1168 | 34 x 1000 | 239 | 1562 | 45 | 1557 | 1596 | v1015 | 1422 | ok (-34 ± 49 vs peak v1015; predecessor +6) | 1070 (89874) | +394 -437 =169, 47.9%, -15 ± 20: reverted | 1168 |
| D-L-Nc-c | qualified | 763 | 36 x 1000 | 238 | 1562 | 45 | 1516 | 1559 | v688 | 1422 | new peak (+4 ± 48 over v688; predecessor +46) | 1070 (90203) | +457 -408 =135, 52.4%, +17 ± 20: kept | 764 |
| T-L-LLs-c | frozen | 651 |  |  | 1599 | 10 |  | 1630 | v540 | 1459 | frozen |  |  | 651 |
| T-L-NNs-c | qualified | 2070 | 43 x 1000 | 238 | 1599 | 46 | 1610 | 1620 | v1590 | 1458 | ok (-21 ± 49 vs peak v1590; predecessor -12) | 1070 (92534) | +427 -412 =161, 50.7%, +5 ± 20: kept | 2071 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 962 | 10 |  | 1702 | v818 | 821 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | dormant | 739 |  |  | 1640 | 22 |  | 1650 (1650) | v682 (v682) | 1500 | dormant (-10 ± 27 vs peak v682 before dormancy at v737, at 1510 vs refs; needs > +27) | 1070 (95178) | +442 -436 =122, 50.3%, +2 ± 20: kept | 740 |
| T-L-Le-pre-c | dormant | 735 | 32 x 1000 | 243 | 1437 | 44 | 1476 | 1528 | v670 | 1297 | dormant (strike 2; -91 ± 47 vs peak v670; predecessor -38) | 1070 (92428) | +475 -375 =150, 55.0%, +35 ± 20: kept | 736 |
| T-N-Ne-c | frozen | 637 |  |  | 1268 | 9 |  | 1369 | v574 | 1128 | frozen |  |  | 637 |
| T-N-Nc-c | dormant | 1500 |  |  | 1324 | 31 |  | 1389 (1389) | v472 (v472) | 1184 | dormant (-65 ± 29 vs peak v472 before dormancy at v1500, at 1248 vs refs; needs > +29) | 1070 (90364) | +429 -421 =150, 50.4%, +3 ± 20: kept | 1501 |
| T-N-Nc-merge-c | dormant | 2128 |  |  | 1375 | 26 |  | 1461 (1447) | v2127 (v631) | 1235 | dormant (-71 ± 20 vs peak v631 before dormancy at v2126, at 1306 vs refs; needs > +20) | 1070 (88647) | +405 -414 =181, 49.5%, -3 ± 20: reverted | 2128 |
| T-N-Nc-fresh-c | dormant | 808 |  |  | 1298 | 31 |  | 1363 (1363) | v624 (v624) | 1158 | dormant (-65 ± 27 vs peak v624 before dormancy at v803, at 1223 vs refs; needs > +27) | 1070 (89618) | +421 -418 =161, 50.1%, +1 ± 20: kept | 809 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 716 | 11 |  | 1345 | v270 | 576 | frozen |  |  | 417 |
| F-N-d2jit-c | dormant | 2419 |  |  | 1187 | 43 | 1194 | 1294 (1294) | v2053 (v2053) | 1047 | dormant (-107 ± 31 vs peak v2053 before dormancy at v2418, at 1153 vs refs; needs > +31) | 1070 (96111) | +396 -382 =222, 50.7%, +5 ± 19: kept | 2420 |
| T-N-NNs-merge-d2jit-c | qualified | 2273 | 51 x 40 | 241 | 1171 | 43 | 1140 | 1207 | v2116 | 1030 | ok (-36 ± 46 vs peak v2116; predecessor +31) | 1070 (95185) | +369 -384 =247, 49.2%, -5 ± 19: reverted | 2273 |
| T-L-Le-merge-d2jit-c | qualified | 892 | 93 x 40 | 239 | 1529 | 45 | 1654 | 1654 | v798 | 1388 | strike 1 (-126 ± 50 vs peak v798; predecessor -126) | 1070 (96202) | +482 -354 =164, 56.4%, +45 ± 20: kept | 893 |
| T-L-NLs-d2jit-c | qualified | 548 | 52 x 40 | 239 | 1398 | 44 | 1374 | 1374 | v495 | 1257 | new peak (+24 ± 46 over v495; predecessor +24) | 1070 (104565) | +439 -318 =243, 56.0%, +42 ± 19: kept | 549 |
| T-N-Nc-merge-d2jit-c | qualified | 954 | 102 x 40 | 247 | 1156 | 43 | 1142 | 1174 | v655 | 1016 | ok (-18 ± 46 vs peak v655; predecessor +14) | 1070 (94619) | +450 -357 =193, 54.6%, +32 ± 19: kept | 955 |
| T-L-NLs-merge-d2jit-c | qualified | 391 | 84 x 40 | 240 | 1464 | 44 | 1365 | 1365 | v306 | 1324 | new peak (+100 ± 46 over v306; predecessor +100) | 1070 (95655) | +516 -349 =135, 58.4%, +59 ± 20: kept | 392 |
| F-L-d4mix-c | qualified | 265 | 7 x 10 | 241 | 1720 | 52 | 1696 | 1747 | v200 | 1579 | ok (-28 ± 57 vs peak v200; predecessor +23) | 1070 (91603) | +359 -463 =178, 44.8%, -36 ± 20: reverted | 265 |
| MoE-d2jit-c | qualified | 1271 | 36 x 40 | 239 | 1700 | 51 | 1702 | 1718 | v882 | 1560 | ok (-18 ± 55 vs peak v882; predecessor -1) | 1070 (94567) | +426 -321 =253, 55.2%, +37 ± 19: kept | 1272 |
