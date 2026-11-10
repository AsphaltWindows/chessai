# Cycle 42

**Own-games step, pooled over the lineages: 24 gates, +10420 -9208 =4372, score 52.5%, Elo +17.6 ± 4.0; kept 16 of 24.**

All cycles so far: 1319 gates, +584609 -517063 =217328, score 52.6%, Elo +17.8 ± 0.5; kept 877 of 1319.

2026-11-07 13:01: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 40 players to retired.txt (their games stay in the fit), added 40.
Tournament: 5 double round-robins of the 60 active players (17700 games) in 6.7 min, 17700 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| FM-c | 7 of 10 | cycle 45 | 36–42 | 892.4 | 1621.7 | freeze 1 at 900 |
| T-N-Nc-fresh-c | 6 of 10 | cycle 46 | 37–42 | 723.6 | 1299.4 | retire |
| F-L-d1play-c | 5 of 10 | cycle 47 | 38–42 | 988.3 | 1531.5 | retire |
| T-L-Le-fresh-c | 5 of 10 | cycle 47 | 38–42 | 898.5 | 1653.9 | retire |
| T-L-Ne-c | 3 of 10 | cycle 49 | 40–42 | 1020.9 | 1558.4 | retire |
| T-N-Nc-merge-c | 3 of 10 | cycle 49 | 40–42 | 761.9 | 1396.9 | retire |
| F-P-c | 2 of 10 | cycle 50 | 41–42 | 923.7 | 1484.8 | retire |
| F-N-d2jit-c | 0 of 10 | cycle 52 | - | - | - | retire |
| T-N-Nc-c | 0 of 10 | cycle 52 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d1play-c | dormant | 3590 |  |  | 1561 | 46 | 1531 | 1631 (1631) | v3322 (v3322) | 1421 | dormant (-70 ± 34 vs peak v3322 before dormancy at v3585, at 1492 vs refs; needs > +34) | 1070 (96081) | +440 -422 =138, 50.9%, +6 ± 20: kept | 3591 |
| F-L-d2jit-c | frozen | 3943 |  |  | 815 | 19 |  | 1806 | v3348 | 675 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1371 | 12 |  | 1472 | v1061 | 1231 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1692 | 12 |  | 1716 | v300 | 1553 | frozen |  |  | 373 |
| FM-c | dormant | 669 |  |  | 1604 | 24 |  | 1659 (1659) | v626 (v626) | 1465 | dormant (-55 ± 28 vs peak v626 before dormancy at v666, at 1520 vs refs; needs > +28) | 1070 (94355) | +399 -464 =137, 46.8%, -23 ± 20: reverted | 669 |
| F-P-c | dormant | 2803 |  |  | 1499 | 45 | 1472 | 1585 (1585) | v2138 (v2138) | 1359 | dormant (-87 ± 33 vs peak v2138 before dormancy at v2801, at 1446 vs refs; needs > +33) | 1070 (98167) | +391 -400 =209, 49.5%, -3 ± 19: reverted | 2803 |
| F-P-tau0-c | qualified | 862 | 174 x 1000 | 239 | 1566 | 46 | 1515 | 1515 | v688 | 1427 | new peak (+51 ± 48 over v688; predecessor +51) | 1070 (88508) | +413 -378 =209, 51.7%, +12 ± 19: kept | 863 |
| F-P-tau0-d2jit-c | qualified | 563 | 140 x 40 | 241 | 1670 | 50 | 1627 | 1627 | v422 | 1531 | new peak (+43 ± 51 over v422; predecessor +43) | 1070 (98884) | +331 -334 =335, 49.9%, -1 ± 18: reverted | 563 |
| T-L-Ne-c | dormant | 1425 |  |  | 1562 | 24 |  | 1616 (1616) | v1288 (v1288) | 1422 | dormant (-54 ± 27 vs peak v1288 before dormancy at v1425, at 1477 vs refs; needs > +27) | 1070 (95033) | +398 -424 =178, 48.7%, -9 ± 20: reverted | 1425 |
| T-L-Ne-fresh-c | qualified | 893 | 34 x 1000 | 241 | 1535 | 45 | 1593 | 1606 | v633 | 1395 | strike 1 (-71 ± 50 vs peak v633; predecessor -58) | 1070 (90272) | +452 -400 =148, 52.6%, +18 ± 20: kept | 894 |
| T-L-Ne-pre-c | qualified | 722 | 30 x 1000 | 236 | 1520 | 45 | 1578 | 1578 | v691 | 1381 | strike 1 (-58 ± 48 vs peak v691; predecessor -58) | 1070 (94437) | +469 -377 =154, 54.6%, +32 ± 20: kept | 723 |
| T-L-Nc-fresh-c | qualified | 1133 | 35 x 1000 | 241 | 1555 | 46 | 1575 | 1595 | v1015 | 1416 | ok (-40 ± 49 vs peak v1015; predecessor -20) | 1070 (88848) | +437 -398 =165, 51.9%, +14 ± 20: kept | 1134 |
| D-L-Nc-c | qualified | 726 | 37 x 1000 | 239 | 1515 | 45 | 1558 | 1558 | v688 | 1375 | ok (-43 ± 48 vs peak v688; predecessor -43) | 1070 (90506) | +589 -284 =127, 65.2%, +110 ± 21: kept | 727 |
| T-L-LLs-c | frozen | 651 |  |  | 1600 | 10 |  | 1629 | v540 | 1461 | frozen |  |  | 651 |
| T-L-NNs-c | qualified | 2026 | 42 x 1000 | 238 | 1609 | 47 | 1585 | 1620 | v1590 | 1470 | ok (-11 ± 50 vs peak v1590; predecessor +24) | 1070 (90963) | +439 -408 =153, 51.5%, +11 ± 20: kept | 2027 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 961 | 10 |  | 1701 | v818 | 821 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | dormant | 739 |  |  | 1646 | 25 |  | 1650 (1650) | v682 (v682) | 1507 | dormant (-4 ± 28 vs peak v682 before dormancy at v737, at 1510 vs refs; needs > +28) | 1070 (90693) | +379 -488 =133, 44.5%, -38 ± 20: reverted | 739 |
| T-L-Le-pre-c | qualified | 702 | 31 x 1000 | 238 | 1474 | 45 | 1527 | 1527 | v670 | 1334 | strike 1 (-53 ± 47 vs peak v670; predecessor -53) | 1070 (92016) | +480 -387 =133, 54.6%, +32 ± 20: kept | 703 |
| T-N-Ne-c | frozen | 637 |  |  | 1267 | 9 |  | 1368 | v574 | 1127 | frozen |  |  | 637 |
| T-N-Nc-c | dormant | 1500 | 33 x 1000 | 242 | 1325 | 44 | 1313 | 1388 | v1359 | 1185 | dormant (strike 2; -63 ± 47 vs peak v1359; predecessor +12) | 1070 (89430) | +400 -416 =184, 49.2%, -6 ± 20: reverted | 1500 |
| T-N-Nc-merge-c | dormant | 2128 |  |  | 1376 | 32 |  | 1460 (1446) | v2127 (v631) | 1237 | dormant (-69 ± 22 vs peak v631 before dormancy at v2126, at 1306 vs refs; needs > +22) | 1070 (87319) | +393 -402 =205, 49.5%, -3 ± 19: reverted | 2128 |
| T-N-Nc-fresh-c | dormant | 808 |  |  | 1288 | 44 | 1307 | 1362 (1362) | v624 (v624) | 1149 | dormant (-74 ± 31 vs peak v624 before dormancy at v803, at 1223 vs refs; needs > +31) | 1070 (90842) | +412 -417 =171, 49.8%, -2 ± 20: reverted | 808 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 715 | 11 |  | 1344 | v270 | 575 | frozen |  |  | 417 |
| F-N-d2jit-c | dormant | 2418 | 120 x 40 | 240 | 1192 | 43 | 1221 | 1292 | v2053 | 1052 | dormant (strike 2; -101 ± 46 vs peak v2053; predecessor -29) | 1070 (95435) | +416 -397 =187, 50.9%, +7 ± 19: kept | 2419 |
| T-N-NNs-merge-d2jit-c | qualified | 2221 | 50 x 40 | 242 | 1138 | 43 | 1174 | 1206 | v2116 | 998 | strike 1 (-68 ± 46 vs peak v2116; predecessor -37) | 1070 (96794) | +391 -377 =232, 50.7%, +5 ± 19: kept | 2222 |
| T-L-Le-merge-d2jit-c | qualified | 798 | 99 x 40 | 239 | 1653 | 49 | 1650 | 1650 | v698 | 1513 | new peak (+3 ± 51 over v698; predecessor +3) | 1070 (94626) | +416 -378 =206, 51.9%, +13 ± 19: kept | 799 |
| T-L-NLs-d2jit-c | qualified | 495 | 54 x 40 | 240 | 1372 | 44 | 1302 | 1302 | v440 | 1233 | new peak (+70 ± 46 over v440; predecessor +70) | 1070 (104957) | +463 -299 =238, 58.2%, +58 ± 19: kept | 496 |
| T-N-Nc-merge-d2jit-c | qualified | 851 | 92 x 40 | 244 | 1140 | 43 | 1171 | 1173 | v655 | 1001 | ok (-33 ± 46 vs peak v655; predecessor -30) | 1070 (96905) | +435 -361 =204, 53.7%, +26 ± 19: kept | 852 |
| T-L-NLs-merge-d2jit-c | qualified | 306 | 95 x 40 | 246 | 1363 | 44 | 1211 | 1211 | v210 | 1224 | new peak (+152 ± 46 over v210; predecessor +152) | 1070 (106959) | +609 -232 =159, 68.8%, +138 ± 21: kept | 307 |
| F-L-d4mix-c | qualified | 257 | 7 x 10 | 225 | 1695 | 51 | 1688 | 1747 | v200 | 1556 | ok (-52 ± 57 vs peak v200; predecessor +7) | 1070 (95480) | +462 -363 =175, 54.9%, +34 ± 20: kept | 258 |
| MoE-d2jit-c | qualified | 1234 | 38 x 40 | 242 | 1700 | 51 | 1674 | 1718 | v882 | 1561 | ok (-18 ± 56 vs peak v882; predecessor +26) | 1070 (94252) | +406 -402 =192, 50.2%, +1 ± 19: kept | 1235 |
