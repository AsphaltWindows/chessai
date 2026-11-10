# Cycle 9

**Changes from this cycle** (deployed after cycle 8, `staged-c8/`): the peak rule for strikes (the user). Each lineage's current version is compared with its peak, the best of its earlier versions in this fit: a qualified lineage more than 2 standard errors of that difference below it gets a strike (2 in a row: dormant), otherwise its strikes reset; a dormant one re-qualifies more than 2 standard errors above it. The predecessor is information only. At the switch, qualified lineages' strikes were recomputed from cycle 8's fit under this rule (deploy.log has the table); dormant lineages stayed dormant. New columns: peak and peak version (`peak_elo`, `peak_version` in cycles.csv).

**Own-games step, pooled over the lineages: 31 gates, +13584 -12511 =4905, score 51.7%, Elo +12.0 ± 3.6; kept 19 of 31.**

All cycles so far: 274 gates, +128691 -103828 =41481, score 54.5%, Elo +31.6 ± 1.2; kept 213 of 274.

2026-11-06 21:52: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then ran the rounds that fill the budget; round sizes d0 1000, d1 860, d2 40, d3 10.
League: retired 52 players to retired.txt (their games stay in the fit), added 52.
Tournament: 5 double round-robins of the 67 active players (22110 games) in 2.2 min, 22110 games saved.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 2 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak by more than 2 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | qualified | 1818 | 210 x 1000 | 374 | +1683 | 50 | +1698 | +1737 | 1185 | +1524 | ok (-54 ± 71 vs peak v1185; predecessor -15) | 1210 (97695) | +492 -373 =135, 56.0%, +42 ± 20: kept | 1819 |
| F-L-d0soft-c | dormant | 1620 |  |  | +1707 | 50 | +1717 | +1717 | 1619 | +1548 | dormant (-10 ± 65 vs peak v1619 at 1559 vs refs; needs > +65) | 1210 (99159) | +442 -410 =148, 51.6%, +11 ± 20: kept | 1621 |
| F-L-d1jit-c | dormant | 1448 |  |  | +1644 | 49 | +1630 | +1689 | 1140 | +1486 | dormant (-45 ± 68 vs peak v1140 at 1530 vs refs; needs > +68) | 1210 (103430) | +487 -337 =176, 57.5%, +52 ± 20: kept | 1449 |
| F-L-d1play-c | qualified | 1789 | 228 x 860 | 366 | +1617 | 49 | +1563 | +1625 | 1269 | +1458 | ok (-8 ± 67 vs peak v1269; predecessor +54) | 1210 (109295) | +489 -360 =151, 56.5%, +45 ± 20: kept | 1790 |
| F-L-d2play-c | dormant | 1011 |  |  | +1812 | 54 | +1790 | +1790 | 1010 | +1653 | dormant (+22 ± 62 vs peak v1010 at 1632 vs refs; needs > +62) | 1210 (105907) | +381 -428 =191, 47.6%, -16 ± 19: reverted | 1011 |
| F-L-d2mix-c | dormant | 750 |  |  | +1796 | 30 |  | +1758 | 591 | +1637 | dormant (+38 ± 82 vs peak v591 at 1599 vs refs; needs > +82) | 1210 (103925) | +335 -486 =179, 42.4%, -53 ± 20: reverted | 750 |
| F-L-d2jit-c | qualified | 1415 | 118 x 40 | 237 | +1823 | 55 | +1813 | +1823 | 1071 | +1664 | ok (+0 ± 76 vs peak v1071; predecessor +10) | 1210 (100229) | +443 -384 =173, 52.9%, +20 ± 20: kept | 1416 |
| F-N-c | dormant | 1060 |  |  | +1518 | 48 | +1521 | +1521 | 1059 | +1360 | dormant (-3 ± 61 vs peak v1059 at 1363 vs refs; needs > +61) | 1210 (97966) | +391 -406 =203, 49.2%, -5 ± 19: reverted | 1060 |
| MoE-c | dormant | 297 |  |  | +1665 | 49 | +1402 | +1688 | 233 | +1506 | dormant (-23 ± 68 vs peak v233 at 1529 vs refs; needs > +68) | 1210 (101066) | +515 -342 =143, 58.7%, +61 ± 20: kept | 298 |
| FM-c | qualified | 266 | 19 x 1000 | 215 | +1373 | 48 | +1385 | +1385 | 246 | +1214 | ok (-12 ± 61 vs peak v246; predecessor -12) | 1210 (111742) | +474 -378 =148, 54.8%, +34 ± 20: kept | 267 |
| TAN-c | qualified | 395 | 111 x 1000 | 239 | +1193 | 47 | +1250 | +1278 | 175 | +1034 | strike 1 (-85 ± 62 vs peak v175; predecessor -57) | 1210 (114375) | +515 -321 =164, 59.7%, +68 ± 20: kept | 396 |
| T-L-Ne-c | dormant | 556 |  |  | +1670 | 50 | +1648 | +1668 | 483 | +1511 | dormant (+1 ± 68 vs peak v483 at 1510 vs refs; needs > +68) | 1210 (102900) | +435 -443 =122, 49.6%, -3 ± 20: reverted | 556 |
| T-L-Ne-merge-c | dormant | 792 |  |  | +1771 | 52 | +1733 | +1733 | 791 | +1612 | dormant (+38 ± 59 vs peak v791 at 1574 vs refs; needs > +59) | 1210 (96762) | +487 -381 =132, 55.3%, +37 ± 20: kept | 793 |
| T-L-Nc-c | dormant | 514 |  |  | +1682 | 50 | +1647 | +1647 | 513 | +1523 | dormant (+35 ± 63 vs peak v513 at 1488 vs refs; needs > +63) | 1210 (101024) | +432 -400 =168, 51.6%, +11 ± 20: kept | 515 |
| T-L-Nc-merge-c | qualified | 1279 | 146 x 1000 | 293 | +1677 | 50 | +1709 | +1756 | 833 | +1518 | strike 1 (-78 ± 70 vs peak v833; predecessor -32) | 1210 (98953) | +522 -347 =131, 58.8%, +61 ± 20: kept | 1280 |
| T-L-LLs-c | dormant | 539 |  |  | +1649 | 49 | +1569 | +1631 | 504 | +1490 | dormant (+18 ± 64 vs peak v504 at 1473 vs refs; needs > +64) | 1210 (100468) | +379 -414 =207, 48.2%, -12 ± 19: reverted | 539 |
| T-L-LLs-merge-c | qualified | 1191 | 55 x 1000 | 227 | +1632 | 49 | +1667 | +1667 | 1136 | +1474 | ok (-35 ± 62 vs peak v1136; predecessor -35) | 1210 (99275) | +394 -462 =144, 46.6%, -24 ± 20: reverted | 1191 |
| T-L-NLs-c | qualified | 652 | 43 x 1000 | 238 | +1588 | 48 | +1601 | +1634 | 571 | +1429 | ok (-46 ± 64 vs peak v571; predecessor -14) | 1210 (102899) | +364 -458 =178, 45.3%, -33 ± 20: reverted | 652 |
| T-L-NLs-merge-c | dormant | 994 |  |  | +1642 | 38 |  | +1674 | 641 | +1484 | dormant (-31 ± 82 vs peak v641 at 1515 vs refs; needs > +82) | 1210 (106128) | +359 -465 =176, 44.7%, -37 ± 20: reverted | 994 |
| T-L-NNs-c | qualified | 644 | 42 x 1000 | 231 | +1617 | 49 | +1653 | +1671 | 561 | +1458 | ok (-54 ± 64 vs peak v561; predecessor -36) | 1210 (101268) | +440 -407 =153, 51.6%, +12 ± 20: kept | 645 |
| T-L-NNs-merge-c | dormant | 819 |  |  | +1735 | 51 | +1773 | +1773 | 818 | +1577 | dormant (-38 ± 67 vs peak v818 at 1614 vs refs; needs > +67) | 1210 (98260) | +403 -434 =163, 48.4%, -11 ± 20: reverted | 819 |
| T-L-Le-c | qualified | 581 | 36 x 1000 | 252 | +1636 | 49 | +1624 | +1634 | 508 | +1477 | new peak (+2 ± 64 over v508; predecessor +12) | 1210 (100162) | +466 -391 =143, 53.8%, +26 ± 20: kept | 582 |
| T-L-Le-merge-c | qualified | 1330 | 88 x 1000 | 183 | +1704 | 50 | +1628 | +1726 | 1083 | +1546 | ok (-22 ± 67 vs peak v1083; predecessor +76) | 1210 (100037) | +484 -385 =131, 54.9%, +34 ± 20: kept | 1331 |
| T-N-Ne-c | qualified | 574 | 37 x 1000 | 262 | +1436 | 48 | +1407 | +1425 | 505 | +1277 | new peak (+12 ± 63 over v505; predecessor +29) | 1210 (98657) | +426 -429 =145, 49.9%, -1 ± 20: reverted | 574 |
| T-N-Ne-merge-c | dormant | 698 |  |  | +1495 | 48 | +1513 | +1513 | 697 | +1336 | dormant (-18 ± 54 vs peak v697 at 1354 vs refs; needs > +54) | 1210 (96387) | +428 -404 =168, 51.2%, +8 ± 20: kept | 699 |
| T-N-Nc-c | dormant | 474 |  |  | +1434 | 37 |  | +1455 | 472 | +1276 | dormant (-20 ± 58 vs peak v472 at 1296 vs refs; needs > +58) | 1210 (99184) | +405 -427 =168, 48.9%, -8 ± 20: reverted | 474 |
| T-N-Nc-merge-c | dormant | 630 |  |  | +1496 | 37 |  | +1507 | 628 | +1337 | dormant (-12 ± 48 vs peak v628 at 1348 vs refs; needs > +48) | 1210 (97918) | +392 -448 =160, 47.2%, -20 ± 20: reverted | 630 |
| T-N-NNs-c | dormant | 536 |  |  | +1212 | 37 |  | +1298 | 452 | +1053 | dormant (-86 ± 57 vs peak v452 at 1139 vs refs; needs > +57) | 1210 (103117) | +444 -406 =150, 51.9%, +13 ± 20: kept | 537 |
| T-N-NNs-merge-c | qualified | 855 | 55 x 1000 | 293 | +1533 | 48 | +1495 | +1524 | 749 | +1375 | new peak (+10 ± 63 over v749; predecessor +38) | 1210 (97190) | +455 -381 =164, 53.7%, +26 ± 20: kept | 856 |
| T-L-H-c | dormant | 1241 |  |  | +1653 | 49 | +1659 | +1700 | 970 | +1494 | dormant (-48 ± 68 vs peak v970 at 1542 vs refs; needs > +68) | 1210 (99943) | +463 -389 =148, 53.7%, +26 ± 20: kept | 1242 |
| T-N-H-c | qualified | 1364 | 131 x 1000 | 213 | +1391 | 48 | +1385 | +1426 | 1104 | +1232 | ok (-35 ± 63 vs peak v1104; predecessor +6) | 1210 (101628) | +442 -415 =143, 51.3%, +9 ± 20: kept | 1365 |
