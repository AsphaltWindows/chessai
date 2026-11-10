# Cycle 33

**Own-games step, pooled over the lineages: 23 gates, +9471 -9645 =3884, score 49.6%, Elo -2.6 ± 4.1; kept 13 of 23.**

All cycles so far: 1103 gates, +491145 -433980 =177875, score 52.6%, Elo +18.0 ± 0.6; kept 728 of 1103.

2026-11-07 08:56: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10, d4 10.
League: retired 42 players to retired.txt (their games stay in the fit), added 42.
Tournament: 5 double round-robins of the 59 active players (17110 games) in 3.9 min, 17110 games saved.
Freeze/retire rule, after 10 cycles dormant: T-N-Le-merge-c retired (avg ply 1 723.4, ply 2 1347.6; no untaken milestone within 10). 2 entries leave the field for retired.txt (their games stay in the fit): 59 players become 57, 17,110 games a cycle become 15,960.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| F-L-c | 8 of 10 | cycle 35 | 26–33 | 978.2 | 1681.2 | retire |
| T-N-NLs-merge-c | 8 of 10 | cycle 35 | 26–33 | 636.9 | 1139.7 | retire |
| F-L-d1play-c | 7 of 10 | cycle 36 | 27–33 | 977.5 | 1548.4 | retire |
| T-L-NLs-c | 6 of 10 | cycle 37 | 28–33 | 981.7 | 1539.9 | retire |
| T-L-Le-c | 3 of 10 | cycle 40 | 31–33 | 923.1 | 1516.6 | retire |
| F-L-d2jit-c | 2 of 10 | cycle 41 | 32–33 | 794.4 | 1779.7 | freeze 1 at 800 |
| F-N-d2jit-c | 1 of 10 | cycle 42 | 33–33 | 596.4 | 1221.8 | freeze 1 at 600 |
| T-L-Le-fresh-c | 0 of 10 | cycle 43 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | dormant | 3144 |  |  | 1682 | 50 | 1690 | 1714 (1714) | v2860 (v2860) | 1541 | dormant (-32 ± 30 vs peak v2860 before dormancy at v3138, at 1573 vs refs; needs > +30) | 1050 (87978) | +442 -398 =160, 52.2%, +15 ± 20: kept | 3145 |
| F-L-d1play-c | dormant | 3320 |  |  | 1568 | 47 | 1592 | 1597 (1597) | v3006 (v3006) | 1427 | dormant (-29 ± 31 vs peak v3006 before dormancy at v3313, at 1456 vs refs; needs > +31) | 1050 (90793) | +464 -385 =151, 53.9%, +28 ± 20: kept | 3321 |
| F-L-d2jit-c | dormant | 3941 |  |  | 1785 | 33 |  | 1815 (1815) | v3348 (v3348) | 1644 | dormant (-30 ± 29 vs peak v3348 before dormancy at v3941, at 1674 vs refs; needs > +29) | 1050 (95369) | +338 -393 =269, 47.2%, -19 ± 18: reverted | 3941 |
| F-N-c | frozen | 1691 |  |  | 1372 | 16 |  | 1475 | v1061 | 1231 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1708 | 15 |  | 1719 | v300 | 1567 | frozen |  |  | 373 |
| FM-c | qualified | 626 |  |  | 1667 | 50 | 1553 | 1628 (1628) | v579 (v579) | 1526 | re-qualified (+40 ± 32 vs peak v579 before dormancy at v623, at 1486 vs refs) | 1050 (94912) | +384 -447 =169, 46.9%, -22 ± 20: reverted | 626 |
| F-P-c | qualified | 1646 | 187 x 1000 | 242 | 1519 | 46 | 1499 | 1526 | v1087 | 1377 | ok (-8 ± 48 vs peak v1087; predecessor +20) | 1050 (93808) | +339 -456 =205, 44.1%, -41 ± 19: reverted | 1646 |
| T-L-Ne-c | qualified | 1229 | 37 x 1000 | 242 | 1558 | 46 | 1585 | 1623 | v1155 | 1417 | strike 1 (-64 ± 49 vs peak v1155; predecessor -27) | 1050 (89978) | +401 -472 =127, 46.5%, -25 ± 20: reverted | 1229 |
| T-L-Ne-fresh-c | qualified | 604 | 37 x 1000 | 239 | 1550 | 46 | 1565 | 1571 | v524 | 1409 | ok (-21 ± 48 vs peak v524; predecessor -15) | 1050 (88599) | +422 -413 =165, 50.4%, +3 ± 20: kept | 605 |
| T-L-Ne-pre-c | qualified | 428 | 38 x 1000 | 241 | 1464 | 45 | 1521 | 1525 | v310 | 1322 | strike 1 (-61 ± 47 vs peak v310; predecessor -58) | 1050 (93363) | +510 -333 =157, 58.9%, +62 ± 20: kept | 429 |
| T-L-Nc-fresh-c | qualified | 789 | 43 x 1000 | 241 | 1549 | 46 | 1557 | 1595 | v701 | 1407 | ok (-46 ± 48 vs peak v701; predecessor -9) | 1050 (86410) | +431 -395 =174, 51.8%, +12 ± 20: kept | 790 |
| D-L-Nc-c | qualified | 365 | 44 x 1000 | 241 | 1558 | 46 | 1505 | 1505 | v320 | 1417 | new peak (+54 ± 48 over v320; predecessor +54) | 1050 (92021) | +400 -491 =109, 45.5%, -32 ± 20: reverted | 365 |
| T-L-LLs-c | frozen | 651 |  |  | 1612 | 12 |  | 1631 | v540 | 1471 | frozen |  |  | 651 |
| T-L-NLs-c | dormant | 1433 |  |  | 1529 | 18 |  | 1588 (1588) | v1345 (v1345) | 1388 | dormant (-58 ± 22 vs peak v1345 before dormancy at v1433, at 1446 vs refs; needs > +22) | 1050 (91749) | +292 -545 =163, 37.4%, -90 ± 20: reverted | 1433 |
| T-L-NNs-c | qualified | 1677 | 42 x 1000 | 238 | 1610 | 48 | 1614 | 1628 | v1590 | 1468 | ok (-18 ± 50 vs peak v1590; predecessor -4) | 1050 (88700) | +426 -433 =141, 49.6%, -2 ± 20: reverted | 1677 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 969 | 12 |  | 1704 | v818 | 827 | frozen |  |  | 1070 |
| T-L-Le-c | dormant | 1174 |  |  | 1558 | 46 | 1534 | 1680 (1680) | v1081 (v1081) | 1417 | dormant (-121 ± 32 vs peak v1081 before dormancy at v1171, at 1538 vs refs; needs > +32) | 1050 (87873) | +436 -379 =185, 52.8%, +20 ± 20: kept | 1175 |
| T-L-Le-fresh-c | dormant | 680 | 37 x 1000 | 237 | 1549 | 46 | 1524 | 1610 | v594 | 1407 | dormant (strike 2; -62 ± 49 vs peak v594; predecessor +25) | 1050 (86192) | +492 -406 =102, 54.3%, +30 ± 20: kept | 681 |
| T-L-Le-pre-c | qualified | 408 | 38 x 1000 | 241 | 1469 | 45 | 1467 | 1502 | v299 | 1328 | ok (-33 ± 47 vs peak v299; predecessor +2) | 1050 (87168) | +490 -387 =123, 55.1%, +36 ± 20: kept | 409 |
| T-N-Ne-c | frozen | 637 |  |  | 1274 | 12 |  | 1371 | v574 | 1133 | frozen |  |  | 637 |
| T-N-Nc-c | qualified | 1185 | 40 x 1000 | 243 | 1315 | 45 | 1352 | 1390 | v472 | 1174 | strike 1 (-75 ± 50 vs peak v472; predecessor -37) | 1050 (86711) | +410 -425 =165, 49.2%, -5 ± 20: reverted | 1185 |
| T-N-Nc-merge-c | qualified | 1832 | 49 x 1000 | 241 | 1434 | 45 | 1409 | 1449 | v1598 | 1292 | ok (-16 ± 46 vs peak v1598; predecessor +24) | 1050 (83869) | +408 -391 =201, 50.8%, +6 ± 19: kept | 1833 |
| T-N-Nc-fresh-c | qualified | 679 | 55 x 1000 | 239 | 1310 | 45 | 1371 | 1371 | v624 | 1168 | strike 1 (-61 ± 47 vs peak v624; predecessor -61) | 1050 (86643) | +426 -419 =155, 50.3%, +2 ± 20: kept | 680 |
| T-N-Le-merge-c | retired | 754 |  |  | 1344 | 14 |  | 1428 (1428) | v700 (v700) | 1203 | retired (avg ply 1 723.4, ply 2 1347.6; no untaken milestone within 10); dormant (-84 ± 19 vs peak v700 before dormancy at v754, at 1287 vs refs; needs > +19) |  |  | 754 |
| T-N-NLs-merge-c | dormant | 1036 |  |  | 1103 | 44 | 1173 | 1417 (1417) | v787 (v787) | 961 | dormant (-314 ± 26 vs peak v787 before dormancy at v1030, at 1275 vs refs; needs > +26) | 1050 (90384) | +415 -408 =177, 50.3%, +2 ± 20: kept | 1037 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 719 | 15 |  | 1347 | v270 | 578 | frozen |  |  | 417 |
| F-N-d2jit-c | dormant | 2047 |  |  | 1222 | 45 | 1212 | 1262 (1262) | v1786 (v1786) | 1080 | dormant (-40 ± 31 vs peak v1786 before dormancy at v2046, at 1120 vs refs; needs > +31) | 1050 (89925) | +420 -387 =193, 51.6%, +12 ± 19: kept | 2048 |
| T-N-NNs-merge-d2jit-c | qualified | 2056 | 65 x 40 | 238 | 1128 | 44 | 1165 | 1172 | v1553 | 986 | strike 1 (-44 ± 43 vs peak v1553; predecessor -38) | 1050 (91440) | +410 -382 =208, 51.4%, +10 ± 19: kept | 2057 |
| F-L-d4mix-c | qualified | 184 | 7 x 10 | 228 | 1709 | 52 | 1673 | 1718 | v141 | 1567 | ok (-9 ± 52 vs peak v141; predecessor +36) | 1050 (84908) | +375 -461 =164, 45.7%, -30 ± 20: reverted | 184 |
| MoE-d2jit-c | qualified | 882 | 45 x 40 | 240 | 1726 | 53 | 1711 | 1711 | v837 | 1585 | new peak (+16 ± 55 over v837; predecessor +16) | 1050 (98783) | +340 -439 =221, 45.1%, -34 ± 19: reverted | 882 |
