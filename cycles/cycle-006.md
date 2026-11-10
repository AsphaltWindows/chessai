# Cycle 6

**Changes from this cycle** (deployed after cycle 5, `staged-c6/`): a dormant lineage re-qualifies only when its difference with its peak exceeds that difference's 95% range, from the fit's covariance (in cycle 5: the current entry's own ±), and the decisions show the difference and its range. TAN-c joins from scratch (`--model tan --tan-m 200000` on `chessai-cyc5`), so the field has 67 players.

**Own-games step, pooled over the lineages: 31 gates, +14055 -11996 =4949, score 53.3%, Elo +23.1 ± 3.6; kept 25 of 31.**

All cycles so far: 181 gates, +87009 -66973 =27018, score 55.5%, Elo +38.6 ± 1.5; kept 154 of 181.

2026-11-06 20:36: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then ran the rounds that fill the budget; round sizes d0 1000, d1 860, d2 40, d3 10.
League: retired 56 players to retired.txt (their games stay in the fit), added 58.
Tournament: 5 double round-robins of the 67 active players (22110 games) in 2.3 min, 22110 games saved.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: a lineage whose new version rates below its predecessor gets a strike; 2 in a row make it dormant. A dormant lineage re-qualifies when its current version's vs refs beats its peak, the best vs refs of its earlier versions at ply 2 in this fit (retired ones included), by more than 1 x the difference's 95% range (the ± in the decision: 1.96 standard errors from the fit's covariance, which carries the shift a whole cycle shares); dormant rows show the gap.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | qualified | 1315 | 129 x 1000 | 250 | +1715 | 61 | +1746 | +1544 | strike 1 (-31) | 1210 (98311) | +495 -351 =154, 57.2%, +50 ± 20: kept | 1316 |
| F-L-d0soft-c | dormant | 1617 | 159 x 1000 | 246 | +1600 | 59 | +1608 | +1429 | dormant (-8, strike 2; -8 ± 64 vs peak v1457) | 1210 (104873) | +548 -339 =113, 60.5%, +74 ± 21: kept | 1618 |
| F-L-d1jit-c | qualified | 1140 |  |  | +1698 | 61 | +1626 | +1526 | re-qualified (+72 ± 65 vs peak v1139 at 1454 vs refs) | 1210 (100986) | +477 -369 =154, 55.4%, +38 ± 20: kept | 1141 |
| F-L-d1play-c | qualified | 1269 | 132 x 860 | 242 | +1634 | 60 | +1606 | +1463 | up +28 | 1210 (108862) | +459 -388 =153, 53.5%, +25 ± 20: kept | 1270 |
| F-L-d2play-c | dormant | 1009 | 95 x 40 | 251 | +1767 | 62 | +1776 | +1595 | dormant (-9, strike 2; -9 ± 70 vs peak v913) | 1210 (104908) | +429 -383 =188, 52.3%, +16 ± 19: kept | 1010 |
| F-L-d2mix-c | dormant | 750 |  |  | +1792 | 53 |  | +1621 | dormant (+17 ± 94 vs peak v591 at 1604 vs refs; needs > +94) | 1210 (102075) | +377 -416 =207, 48.0%, -14 ± 19: reverted | 750 |
| F-L-d2jit-c | qualified | 1071 | 109 x 40 | 259 | +1832 | 65 | +1746 | +1660 | up +85 | 1210 (107056) | +480 -336 =184, 57.2%, +50 ± 20: kept | 1072 |
| F-N-c | dormant | 1057 |  |  | +1494 | 59 | +1470 | +1323 | dormant (+25 ± 63 vs peak v1056 at 1298 vs refs; needs > +63) | 1210 (97202) | +425 -384 =191, 52.0%, +14 ± 19: kept | 1058 |
| MoE-c | qualified | 233 |  |  | +1697 | 61 | +1294 | +1525 | re-qualified (+225 ± 96 vs peak v178 at 1300 vs refs) | 1210 (102723) | +501 -359 =140, 57.1%, +50 ± 20: kept | 234 |
| FM-c | qualified | 201 | 19 x 1000 | 254 | +1202 | 58 | +1344 | +1031 | strike 1 (-142) | 1210 (117157) | +574 -238 =188, 66.8%, +122 ± 20: kept | 202 |
| TAN-c | qualified | 68 | 68 x 1000 | 195 | +1227 | 58 |  | +1055 | qualified (first rating) | 1210 (106833) | +524 -316 =160, 60.4%, +73 ± 20: kept | 69 |
| T-L-Ne-c | qualified | 483 | 31 x 1000 | 254 | +1677 | 60 | +1611 | +1506 | up +66 | 1210 (104847) | +379 -472 =149, 45.4%, -32 ± 20: reverted | 483 |
| T-L-Ne-merge-c | dormant | 790 | 89 x 1000 | 254 | +1700 | 61 | +1729 | +1529 | dormant (-29, strike 2; -29 ± 67 vs peak v700) | 1210 (98635) | +514 -352 =134, 58.1%, +57 ± 20: kept | 791 |
| T-L-Nc-c | dormant | 511 | 34 x 1000 | 224 | +1648 | 60 | +1655 | +1477 | dormant (-7, strike 2; -7 ± 65 vs peak v477) | 1210 (100246) | +409 -393 =198, 50.8%, +6 ± 19: kept | 512 |
| T-L-Nc-merge-c | qualified | 833 |  |  | +1764 | 62 | +1686 | +1593 | re-qualified (+78 ± 68 vs peak v832 at 1515 vs refs) | 1210 (98746) | +499 -402 =99, 54.9%, +34 ± 21: kept | 834 |
| T-L-LLs-c | qualified | 472 | 29 x 1000 | 243 | +1635 | 60 | +1557 | +1464 | up +78 | 1210 (103926) | +443 -387 =170, 52.8%, +20 ± 20: kept | 473 |
| T-L-LLs-merge-c | qualified | 1010 | 75 x 1000 | 254 | +1671 | 60 | +1615 | +1499 | up +56 | 1210 (104782) | +488 -372 =140, 55.8%, +40 ± 20: kept | 1011 |
| T-L-NLs-c | qualified | 528 | 39 x 1000 | 234 | +1594 | 59 | +1582 | +1423 | up +13 | 1210 (103933) | +355 -444 =201, 45.6%, -31 ± 19: reverted | 528 |
| T-L-NLs-merge-c | qualified | 857 | 75 x 1000 | 254 | +1671 | 60 | +1546 | +1499 | up +125 | 1210 (102078) | +434 -396 =170, 51.9%, +13 ± 20: kept | 858 |
| T-L-NNs-c | qualified | 528 | 41 x 1000 | 265 | +1662 | 60 | +1643 | +1491 | up +20 | 1210 (103728) | +379 -443 =178, 46.8%, -22 ± 20: reverted | 528 |
| T-L-NNs-merge-c | dormant | 816 |  |  | +1763 | 62 | +1692 | +1591 | dormant (+69 ± 98 vs peak v541 at 1522 vs refs; needs > +98) | 1210 (99362) | +454 -406 =140, 52.4%, +17 ± 20: kept | 817 |
| T-L-Le-c | qualified | 479 | 32 x 1000 | 266 | +1607 | 59 | +1639 | +1436 | strike 1 (-32) | 1210 (101833) | +539 -350 =111, 59.5%, +66 ± 21: kept | 480 |
| T-L-Le-merge-c | qualified | 1017 | 132 x 1000 | 332 | +1724 | 61 | +1687 | +1553 | up +37 | 1210 (97831) | +468 -396 =136, 53.6%, +25 ± 20: kept | 1018 |
| T-N-Ne-c | qualified | 470 | 30 x 1000 | 235 | +1423 | 59 | +1396 | +1251 | up +26 | 1210 (103572) | +437 -423 =140, 50.7%, +5 ± 20: kept | 471 |
| T-N-Ne-merge-c | dormant | 696 |  |  | +1477 | 59 | +1426 | +1306 | dormant (+24 ± 95 vs peak v500 at 1281 vs refs; needs > +95) | 1210 (99425) | +422 -405 =173, 50.8%, +6 ± 20: kept | 697 |
| T-N-Nc-c | dormant | 472 |  |  | +1464 | 59 | +1408 | +1293 | dormant (+56 ± 63 vs peak v471 at 1237 vs refs; needs > +63) | 1210 (99935) | +441 -409 =150, 51.6%, +11 ± 20: kept | 473 |
| T-N-Nc-merge-c | dormant | 628 |  |  | +1516 | 50 |  | +1345 | dormant (+56 ± 89 vs peak v487 at 1289 vs refs; needs > +89) | 1210 (99250) | +434 -398 =168, 51.8%, +12 ± 20: kept | 629 |
| T-N-NNs-c | qualified | 452 | 37 x 1000 | 261 | +1308 | 58 | +1290 | +1137 | up +18 | 1210 (102279) | +435 -410 =155, 51.2%, +9 ± 20: kept | 453 |
| T-N-NNs-merge-c | qualified | 652 | 61 x 1000 | 153 | +1526 | 59 | +1506 | +1354 | up +19 | 1210 (98269) | +402 -426 =172, 48.8%, -8 ± 20: reverted | 652 |
| T-L-H-c | qualified | 970 | 111 x 1000 | 235 | +1709 | 61 | +1621 | +1538 | up +88 | 1210 (99593) | +412 -428 =160, 49.2%, -6 ± 20: reverted | 970 |
| T-N-H-c | qualified | 972 | 115 x 1000 | 227 | +1424 | 59 | +1372 | +1253 | up +52 | 1210 (101737) | +422 -405 =173, 50.8%, +6 ± 20: kept | 973 |
