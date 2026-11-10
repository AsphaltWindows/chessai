# Cycle 67

**Own-games step, pooled over the lineages: 7 gates, +2668 -2743 =1589, score 49.5%, Elo -3.7 ± 7.2; kept 3 of 7.**

All cycles so far: 1681 gates, +732317 -658672 =290011, score 52.2%, Elo +15.2 ± 0.5; kept 1104 of 1681.

2026-11-07 20:47: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10.
League: retired 8 players to retired.txt (their games stay in the fit), added 8.
Tournament: 5 double round-robins of the 28 active players (3780 games) in 9.3 min, 3780 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| T-L-NLs-d2jit-c | 9 of 10 | cycle 68 | 59–67 | 853.3 | 1604.8 | retire |
| D-L-Nc-c | 6 of 10 | cycle 71 | 62–67 | 974.4 | 1616.5 | retire |
| F-P-tau0-d2jit-c | 3 of 10 | cycle 74 | 65–67 | 843.8 | 1765.0 | retire |
| MoE-d2jit-c | 0 of 10 | cycle 77 | - | - | - | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 900 T-L-Le-fresh-c-v742-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1500 F-P-c-v2808-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d2jit-c | frozen | 3943 |  |  | 801 | 10 |  | 1790 | v1651 | 662 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1359 | 9 |  | 1467 | v1061 | 1221 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1659 | 9 |  | 1711 | v300 | 1521 | frozen |  |  | 373 |
| F-P-c | frozen | 2808 |  |  | 1477 | 14 |  | 1566 | v2138 | 1339 | frozen |  |  | 2808 |
| F-P-tau0-d2jit-c | dormant | 3840 |  |  | 1757 | 59 |  | 1788 (1788) | v2036 (v2036) | 1619 | dormant (-31 ± 44 vs peak v2036 before dormancy at v3839, at 1650 vs refs; needs > +44) | 430 (36812) | +331 -331 =338, 50.0%, -0 ± 18: kept | 3841 |
| T-L-Nc-fresh-c | qualified | 2061 | 31 x 1000 | 239 | 1605 | 72 | 1544 | 1588 | v1995 | 1466 | new peak (+17 ± 76 over v1995; predecessor +61) | 430 (35845) | +423 -413 =164, 50.5%, +4 ± 20: kept | 2062 |
| D-L-Nc-c | dormant | 1544 |  |  | 1607 | 52 |  | 1626 (1622) | v1543 (v1370) | 1468 | dormant (-15 ± 39 vs peak v1370 before dormancy at v1541, at 1484 vs refs; needs > +39) | 430 (36918) | +404 -438 =158, 48.3%, -12 ± 20: reverted | 1544 |
| T-L-LLs-c | frozen | 651 |  |  | 1569 | 8 |  | 1624 | v540 | 1431 | frozen |  |  | 651 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 949 | 8 |  | 1696 | v818 | 811 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | frozen | 742 |  |  | 894 | 12 |  | 1630 | v682 | 756 | frozen |  |  | 742 |
| T-N-Ne-c | frozen | 637 |  |  | 1246 | 7 |  | 1363 | v574 | 1108 | frozen |  |  | 637 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 707 | 8 |  | 1339 | v270 | 569 | frozen |  |  | 417 |
| T-L-NLs-d2jit-c | dormant | 1462 |  |  | 1626 | 53 |  | 1633 (1600) | v1461 (v1331) | 1488 | dormant (+27 ± 38 vs peak v1331 before dormancy at v1457, at 1462 vs refs; needs > +38) | 430 (38255) | +379 -397 =224, 49.1%, -6 ± 19: reverted | 1462 |
| T-N-Nc-merge-d2jit-c | qualified | 1986 | 55 x 40 | 238 | 1208 | 72 | 1206 | 1263 | v1441 | 1070 | ok (-55 ± 71 vs peak v1441; predecessor +2) | 430 (38928) | +384 -386 =230, 49.9%, -1 ± 19: reverted | 1986 |
| T-L-NLs-merge-d2jit-c | qualified | 2368 | 88 x 40 | 240 | 1588 | 72 | 1608 | 1649 | v1716 | 1450 | ok (-61 ± 70 vs peak v1716; predecessor -20) | 430 (39660) | +331 -413 =256, 45.9%, -29 ± 19: reverted | 2368 |
| MoE-d2jit-c | dormant | 2311 | 40 x 40 | 238 | 1595 | 72 | 1677 | 1762 | v2156 | 1457 | dormant (strike 2; -167 ± 81 vs peak v2156; predecessor -82) | 430 (37264) | +416 -365 =219, 52.5%, +18 ± 19: kept | 2312 |
