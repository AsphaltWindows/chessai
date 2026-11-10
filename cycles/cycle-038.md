# Cycle 38

**Changes from this cycle** (deployed after cycle 37, `staged-c37/`), the user's (the probit agent's proposal): F-P-tau0-c joins, from scratch on `chessai-probit`: F-P-c's line plus `--probit-tau 0`, one factor, the posterior's dynamics noise tau 0 instead of 1e-5. At 1e-5 the weights' variance floors at about 2 tau, which caps the filter's memory at about one round (about 200k positions), and F-P-c's v1646 already sat at that floor; at tau 0 the model averages over all its data. In the agent's check at equal CPU, tau 0 beat tau 1e-5 by 73, 48 and 85 Elo at v100, v200 and v300 (depth-2 arenas of 2 x 1000 games); the risk is that it keeps stiffening over the league's 1000+ rounds (tau 1e-6 is the hedge). Its own step trains and gates with `chessai-probit` (`own_games.binary_by_lineage`). Checked before the deploy, on `chessai-probit`: from scratch, 6 rounds of 1000 games gave the agent's tau-0 run byte for byte, with tau 0 in every model file; 3 rounds and `--resume` to 6 gave the same files as one run; its own step through the driver, from the agent's tau-0 v330 on saved league games, trained (`--start`, with `--probit-tau 0`) and gated (1000 games at depth 2, 55.4%, kept), and the kept v331 still has tau 0; and a round-robin as the driver runs it, with its entries, F-P-c's and two random players, played all 56 games.

**Own-games step, pooled over the lineages: 24 gates, +10261 -9403 =4336, score 51.8%, Elo +12.4 ± 4.0; kept 17 of 24.**

All cycles so far: 1221 gates, +542142 -479687 =199171, score 52.6%, Elo +17.8 ± 0.6; kept 812 of 1221.

2026-11-07 11:14: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 40 players to retired.txt (their games stay in the fit), added 42.
Tournament: 5 double round-robins of the 59 active players (17110 games) in 4.6 min, 17110 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| T-L-Le-c | 8 of 10 | cycle 40 | 31–38 | 925.0 | 1570.0 | retire |
| F-L-d2jit-c | 7 of 10 | cycle 41 | 32–38 | 802.4 | 1784.1 | freeze 1 at 800 |
| F-N-d2jit-c | 6 of 10 | cycle 42 | 33–38 | 623.1 | 1214.5 | retire |
| T-N-NNs-merge-d2jit-c | 4 of 10 | cycle 44 | 35–38 | 581.8 | 1135.5 | retire |
| FM-c | 3 of 10 | cycle 45 | 36–38 | 893.9 | 1625.1 | freeze 1 at 900 |
| T-N-Nc-fresh-c | 2 of 10 | cycle 46 | 37–38 | 704.5 | 1295.8 | retire |
| F-L-d1play-c | 1 of 10 | cycle 47 | 38–38 | 975.9 | 1515.8 | retire |
| T-L-Le-fresh-c | 1 of 10 | cycle 47 | 38–38 | 868.2 | 1645.8 | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d1play-c | dormant | 3586 |  |  | 1516 | 46 | 1493 | 1636 (1636) | v3322 (v3322) | 1375 | dormant (-120 ± 34 vs peak v3322 before dormancy at v3585, at 1495 vs refs; needs > +34) | 1050 (96104) | +467 -385 =148, 54.1%, +29 ± 20: kept | 3587 |
| F-L-d2jit-c | dormant | 3943 |  |  | 1817 | 44 |  | 1811 (1811) | v3348 (v3348) | 1677 | dormant (+7 ± 33 vs peak v3348 before dormancy at v3941, at 1670 vs refs; needs > +33) | 1050 (88480) | +338 -423 =239, 45.8%, -30 ± 19: reverted | 3943 |
| F-N-c | frozen | 1691 |  |  | 1373 | 13 |  | 1474 | v1061 | 1233 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1698 | 13 |  | 1718 | v300 | 1558 | frozen |  |  | 373 |
| FM-c | dormant | 668 |  |  | 1618 | 35 |  | 1664 (1664) | v626 (v626) | 1477 | dormant (-46 ± 31 vs peak v626 before dormancy at v666, at 1523 vs refs; needs > +31) | 1050 (91741) | +462 -401 =137, 53.0%, +21 ± 20: kept | 669 |
| F-P-c | qualified | 2466 | 163 x 1000 | 240 | 1563 | 47 | 1482 | 1590 | v2138 | 1423 | ok (-26 ± 51 vs peak v2138; predecessor +81) | 1050 (96768) | +356 -414 =230, 47.1%, -20 ± 19: reverted | 2466 |
| F-P-tau0-c | qualified | 156 | 156 x 1000 | 239 | 1383 | 45 |  |  |  | 1243 | qualified (first rating) | 1050 (92619) | +500 -318 =182, 59.1%, +64 ± 20: kept | 157 |
| T-L-Ne-c | qualified | 1389 | 38 x 1000 | 239 | 1554 | 47 | 1584 | 1620 | v1288 | 1413 | strike 1 (-67 ± 51 vs peak v1288; predecessor -30) | 1050 (89203) | +371 -454 =175, 45.9%, -29 ± 20: reverted | 1389 |
| T-L-Ne-fresh-c | qualified | 758 | 35 x 1000 | 239 | 1556 | 47 | 1585 | 1610 | v633 | 1416 | strike 1 (-54 ± 51 vs peak v633; predecessor -29) | 1050 (91857) | +466 -379 =155, 54.4%, +30 ± 20: kept | 759 |
| T-L-Ne-pre-c | qualified | 585 | 36 x 1000 | 239 | 1524 | 46 | 1582 | 1582 | v548 | 1384 | strike 1 (-58 ± 50 vs peak v548; predecessor -58) | 1050 (91338) | +439 -396 =165, 52.1%, +15 ± 20: kept | 586 |
| T-L-Nc-fresh-c | qualified | 971 | 44 x 1000 | 241 | 1552 | 47 | 1570 | 1591 | v822 | 1412 | ok (-38 ± 51 vs peak v822; predecessor -17) | 1050 (89850) | +426 -408 =166, 50.9%, +6 ± 20: kept | 972 |
| D-L-Nc-c | qualified | 555 | 43 x 1000 | 239 | 1523 | 46 | 1529 | 1554 | v365 | 1382 | ok (-32 ± 50 vs peak v365; predecessor -6) | 1050 (92560) | +522 -363 =115, 58.0%, +56 ± 20: kept | 556 |
| T-L-LLs-c | frozen | 651 |  |  | 1607 | 11 |  | 1630 | v540 | 1467 | frozen |  |  | 651 |
| T-L-NNs-c | qualified | 1857 | 42 x 1000 | 242 | 1607 | 48 | 1606 | 1624 | v1590 | 1467 | ok (-17 ± 50 vs peak v1590; predecessor +2) | 1050 (90768) | +391 -426 =183, 48.2%, -12 ± 20: reverted | 1857 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 965 | 11 |  | 1703 | v818 | 825 | frozen |  |  | 1070 |
| T-L-Le-c | dormant | 1175 |  |  | 1600 | 22 |  | 1675 (1675) | v1081 (v1081) | 1460 | dormant (-75 ± 25 vs peak v1081 before dormancy at v1171, at 1534 vs refs; needs > +25) | 1050 (91052) | +428 -434 =138, 49.7%, -2 ± 20: reverted | 1175 |
| T-L-Le-fresh-c | dormant | 738 |  |  | 1646 | 50 | 1548 | 1654 (1654) | v682 (v682) | 1505 | dormant (-8 ± 35 vs peak v682 before dormancy at v737, at 1514 vs refs; needs > +35) | 1050 (90568) | +482 -391 =127, 54.5%, +32 ± 20: kept | 739 |
| T-L-Le-pre-c | qualified | 564 | 34 x 1000 | 242 | 1474 | 46 | 1522 | 1522 | v529 | 1333 | strike 1 (-49 ± 49 vs peak v529; predecessor -49) | 1050 (90751) | +438 -427 =135, 50.5%, +4 ± 20: kept | 565 |
| T-N-Ne-c | frozen | 637 |  |  | 1271 | 10 |  | 1370 | v574 | 1130 | frozen |  |  | 637 |
| T-N-Nc-c | qualified | 1359 | 40 x 1000 | 241 | 1391 | 45 | 1357 | 1389 | v472 | 1251 | new peak (+2 ± 50 over v472; predecessor +34) | 1050 (86595) | +426 -424 =150, 50.1%, +1 ± 20: kept | 1360 |
| T-N-Nc-merge-c | qualified | 2080 | 54 x 1000 | 242 | 1360 | 45 | 1411 | 1448 | v631 | 1219 | strike 1 (-88 ± 41 vs peak v631; predecessor -51) | 1050 (85879) | +419 -405 =176, 50.7%, +5 ± 20: kept | 2081 |
| T-N-Nc-fresh-c | dormant | 805 |  |  | 1282 | 45 | 1307 | 1367 (1367) | v624 (v624) | 1142 | dormant (-84 ± 31 vs peak v624 before dormancy at v803, at 1226 vs refs; needs > +31) | 1050 (87588) | +400 -423 =177, 48.9%, -8 ± 20: reverted | 805 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 717 | 12 |  | 1346 | v270 | 577 | frozen |  |  | 417 |
| F-N-d2jit-c | dormant | 2052 |  |  | 1198 | 44 | 1228 | 1258 (1258) | v1786 (v1786) | 1057 | dormant (-60 ± 31 vs peak v1786 before dormancy at v2046, at 1117 vs refs; needs > +31) | 1050 (94195) | +412 -387 =201, 51.2%, +9 ± 19: kept | 2053 |
| T-N-NNs-merge-d2jit-c | dormant | 2115 |  |  | 1151 | 44 | 1127 | 1168 (1168) | v1553 (v1553) | 1011 | dormant (-17 ± 29 vs peak v1553 before dormancy at v2112, at 1027 vs refs; needs > +29) | 1050 (92820) | +365 -404 =231, 48.0%, -14 ± 19: reverted | 2115 |
| T-L-Le-merge-d2jit-c | qualified | 404 | 107 x 40 | 240 | 1501 | 46 | 1383 | 1383 | v296 | 1360 | new peak (+117 ± 48 over v296; predecessor +117) | 1050 (91986) | +502 -359 =139, 57.1%, +50 ± 20: kept | 405 |
| T-L-NLs-d2jit-c | qualified | 272 | 61 x 40 | 239 | 1146 | 44 | 1058 | 1058 | v210 | 1006 | new peak (+88 ± 46 over v210; predecessor +88) | 1050 (109231) | +374 -260 =366, 55.7%, +40 ± 17: kept | 273 |
| T-N-Nc-merge-d2jit-c | qualified | 441 | 105 x 40 | 241 | 1129 | 44 | 1142 | 1142 | v335 | 988 | ok (-13 ± 46 vs peak v335; predecessor -13) | 1050 (95029) | +461 -355 =184, 55.3%, +37 ± 20: kept | 442 |
| F-L-d4mix-c | qualified | 225 | 7 x 10 | 224 | 1730 | 54 | 1694 | 1751 | v200 | 1589 | ok (-22 ± 58 vs peak v200; predecessor +35) | 1050 (90318) | +420 -399 =181, 51.0%, +7 ± 20: kept | 226 |
| MoE-d2jit-c | qualified | 1077 | 37 x 40 | 242 | 1722 | 53 | 1647 | 1722 | v882 | 1582 | ok (-0 ± 57 vs peak v882; predecessor +76) | 1050 (93152) | +396 -368 =236, 51.4%, +10 ± 19: kept | 1078 |
