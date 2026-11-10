# Cycle 73

**Own-games step, pooled over the lineages: 6 gates, +2238 -2323 =1439, score 49.3%, Elo -4.9 ± 7.7; kept 3 of 6.**

All cycles so far: 1720 gates, +747238 -673618 =299144, score 52.1%, Elo +14.9 ± 0.5; kept 1122 of 1720.

2026-11-07 22:30: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10.
League: retired 6 players to retired.txt (their games stay in the fit), added 6.
Tournament: 5 double round-robins of the 26 active players (3250 games) in 8.5 min, 3250 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| F-P-tau0-d2jit-c | 9 of 10 | cycle 74 | 65–73 | 841.4 | 1744.4 | retire |
| MoE-d2jit-c | 6 of 10 | cycle 77 | 68–73 | 826.6 | 1680.2 | retire |
| T-N-Nc-merge-d2jit-c | 4 of 10 | cycle 79 | 70–73 | 566.2 | 1142.0 | retire |
| T-L-NLs-d2jit-c | 1 of 10 | cycle 82 | 73–73 | 836.7 | 1619.0 | retire |
| T-L-NLs-merge-d2jit-c | 1 of 10 | cycle 82 | 73–73 | 737.2 | 1566.6 | retire |

Milestones taken (multiple: takers): 0 random, 100 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 900 T-L-Le-fresh-c-v742-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1500 F-P-c-v2808-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d2jit-c | frozen | 3943 |  |  | 794 | 10 |  | 1786 | v1651 | 658 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1354 | 8 |  | 1463 | v1061 | 1218 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1652 | 9 |  | 1707 | v300 | 1517 | frozen |  |  | 373 |
| F-P-c | frozen | 2808 |  |  | 1470 | 13 |  | 1560 | v2138 | 1335 | frozen |  |  | 2808 |
| F-P-tau0-d2jit-c | dormant | 3841 |  |  | 1730 | 34 |  | 1781 (1781) | v2036 (v2036) | 1595 | dormant (-50 ± 37 vs peak v2036 before dormancy at v3839, at 1646 vs refs; needs > +37) | 390 (32332) | +342 -333 =325, 50.4%, +3 ± 18: kept | 3842 |
| T-L-Nc-fresh-c | qualified | 2268 | 36 x 1000 | 239 | 1540 | 74 | 1458 | 1602 | v2061 | 1404 | ok (-62 ± 79 vs peak v2061; predecessor +81) | 390 (32382) | +438 -401 =161, 51.8%, +13 ± 20: kept | 2269 |
| T-L-LLs-c | frozen | 651 |  |  | 1561 | 8 |  | 1619 | v540 | 1426 | frozen |  |  | 651 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 943 | 7 |  | 1692 | v818 | 808 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | frozen | 742 |  |  | 885 | 11 |  | 1624 | v682 | 750 | frozen |  |  | 742 |
| T-N-Ne-c | frozen | 637 |  |  | 1240 | 7 |  | 1359 | v574 | 1105 | frozen |  |  | 637 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 701 | 8 |  | 1335 | v270 | 566 | frozen |  |  | 417 |
| T-L-NLs-d2jit-c | dormant | 1683 |  |  | 1619 | 76 | 1455 | 1637 (1637) | v1462 (v1462) | 1484 | dormant (-18 ± 44 vs peak v1462 before dormancy at v1682, at 1502 vs refs; needs > +44) | 390 (35071) | +362 -391 =247, 48.5%, -10 ± 19: reverted | 1683 |
| T-N-Nc-merge-d2jit-c | dormant | 2108 |  |  | 1174 | 74 | 1104 | 1256 (1256) | v1441 (v1441) | 1039 | dormant (-82 ± 48 vs peak v1441 before dormancy at v2105, at 1121 vs refs; needs > +48) | 390 (33642) | +401 -365 =234, 51.8%, +12 ± 19: kept | 2109 |
| T-L-NLs-merge-d2jit-c | dormant | 2740 |  |  | 1567 | 54 |  | 1649 (1649) | v2454 (v2454) | 1431 | dormant (-83 ± 46 vs peak v2454 before dormancy at v2740, at 1514 vs refs; needs > +46) | 390 (34482) | +338 -444 =218, 44.7%, -37 ± 19: reverted | 2740 |
| MoE-d2jit-c | dormant | 2313 |  |  | 1653 | 39 |  | 1755 (1755) | v2156 (v2156) | 1518 | dormant (-102 ± 45 vs peak v2156 before dormancy at v2311, at 1620 vs refs; needs > +45) | 390 (32438) | +357 -389 =254, 48.4%, -11 ± 19: reverted | 2313 |
