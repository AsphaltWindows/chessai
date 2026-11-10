# Cycle 68

**Own-games step, pooled over the lineages: 7 gates, +2753 -2665 =1582, score 50.6%, Elo +4.4 ± 7.2; kept 4 of 7.**

All cycles so far: 1688 gates, +735070 -661337 =291593, score 52.2%, Elo +15.2 ± 0.5; kept 1108 of 1688.

2026-11-07 21:03: budget 240 CPU s per qualified lineage, measured in the loop: each job measured its CPU per game over its first rounds (at least 5% of the budget), then the rest in chunks of about 22% of it, each re-planned from the last one's CPU per game until the budget was spent; round sizes d0 1000, d1 860, d2 40, d3 10.
League: retired 10 players to retired.txt (their games stay in the fit), added 10.
Tournament: 5 double round-robins of the 28 active players (3780 games) in 7.9 min, 3780 games saved.

**Freeze/retire rule (the user): a lineage dormant 10 cycles is frozen at the ply whose mean Elo over them is within 10 of an untaken multiple of 100 (the closer ply; a tie, ply 2; closest lineage first), else retired. Exempt: none.** Dormant lineages, their running mean Elo per ply and what the rule would do now (lineages due earlier claim first):

| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |
|---|---:|---:|---|---:|---:|---|
| D-L-Nc-c | 7 of 10 | cycle 71 | 62–68 | 972.9 | 1616.5 | retire |
| F-P-tau0-d2jit-c | 4 of 10 | cycle 74 | 65–68 | 843.4 | 1756.7 | retire |
| MoE-d2jit-c | 1 of 10 | cycle 77 | 68–68 | 774.6 | 1694.5 | retire |

Milestones taken (multiple: takers): 0 random, 200 random-1, 300 random-2, random-3, 400 random-4, 700 T-N-LNs-merge-c-v417-1, 800 F-L-d2jit-c-v3943-1, 900 T-L-Le-fresh-c-v742-1, 1000 T-L-NNs-merge-c-v1070-1, 1300 T-N-Ne-c-v637-2, 1400 F-N-c-v1691-2, 1500 F-P-c-v2808-2, 1600 T-L-LLs-c-v651-2, 1700 MoE-c-v373-2.

Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed reference; it neither self-plays nor takes own steps or gates and gets no decision. Its Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the mean Elo of random, random-1, random-2 in this fit. Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those were the random players alone (every lineage entered a new version each cycle), so comparisons across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies deep) play every cycle as references too and link the fits better; they stay out of that mean, so that vs refs keeps its meaning.

Decisions: each lineage's current version is compared with its peak, the best of its earlier versions at ply 2 in this fit, retired ones included (peak and peak version; Elo scale). A qualified lineage more than 1.5 standard errors of that difference below its peak gets a strike, and 2 in a row make it dormant; otherwise its strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps its own-games step and gate, and re-qualifies when its current version beats its peak from before it went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions made while dormant never raise that bar) by more than 1 standard errors. The ± in a decision is that many standard errors, from the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is shown for information only; until the peak rule replaced it (see README.md), a new version below its predecessor got a strike.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-d2jit-c | frozen | 3943 |  |  | 799 | 10 |  | 1789 | v1651 | 662 | frozen |  |  | 3943 |
| F-N-c | frozen | 1691 |  |  | 1358 | 8 |  | 1466 | v1061 | 1221 | frozen |  |  | 1691 |
| MoE-c | frozen | 373 |  |  | 1657 | 9 |  | 1709 | v300 | 1520 | frozen |  |  | 373 |
| F-P-c | frozen | 2808 |  |  | 1476 | 14 |  | 1564 | v2138 | 1338 | frozen |  |  | 2808 |
| F-P-tau0-d2jit-c | dormant | 3841 |  |  | 1732 | 79 | 1760 | 1786 (1786) | v2036 (v2036) | 1595 | dormant (-54 ± 52 vs peak v2036 before dormancy at v3839, at 1649 vs refs; needs > +52) | 430 (36909) | +321 -347 =332, 48.7%, -9 ± 18: reverted | 3841 |
| T-L-Nc-fresh-c | qualified | 2097 | 35 x 1000 | 243 | 1596 | 72 | 1607 | 1607 | v2061 | 1458 | ok (-12 ± 77 vs peak v2061; predecessor -12) | 430 (36493) | +434 -413 =153, 51.0%, +7 ± 20: kept | 2098 |
| D-L-Nc-c | dormant | 1544 |  |  | 1617 | 43 |  | 1624 (1620) | v1543 (v1370) | 1480 | dormant (-3 ± 36 vs peak v1370 before dormancy at v1541, at 1483 vs refs; needs > +36) | 430 (34676) | +419 -415 =166, 50.2%, +1 ± 20: kept | 1545 |
| T-L-LLs-c | frozen | 651 |  |  | 1567 | 8 |  | 1622 | v540 | 1430 | frozen |  |  | 651 |
| T-L-NNs-merge-c | frozen | 1070 |  |  | 948 | 7 |  | 1695 | v818 | 811 | frozen |  |  | 1070 |
| T-L-Le-fresh-c | frozen | 742 |  |  | 892 | 12 |  | 1629 | v682 | 755 | frozen |  |  | 742 |
| T-N-Ne-c | frozen | 637 |  |  | 1245 | 7 |  | 1362 | v574 | 1108 | frozen |  |  | 637 |
| T-N-LNs-merge-c | frozen | 417 |  |  | 706 | 8 |  | 1337 | v270 | 568 | frozen |  |  | 417 |
| T-L-NLs-d2jit-c | qualified | 1462 |  |  | 1643 | 44 |  | 1631 (1598) | v1461 (v1331) | 1505 | re-qualified (+44 ± 35 vs peak v1331 before dormancy at v1457, at 1461 vs refs) | 430 (37809) | +369 -393 =238, 48.8%, -8 ± 19: reverted | 1462 |
| T-N-Nc-merge-d2jit-c | qualified | 2046 | 60 x 40 | 239 | 1180 | 72 | 1206 | 1261 | v1441 | 1042 | strike 1 (-82 ± 71 vs peak v1441; predecessor -27) | 430 (39142) | +414 -374 =212, 52.0%, +14 ± 19: kept | 2047 |
| T-L-NLs-merge-d2jit-c | qualified | 2454 | 86 x 40 | 241 | 1656 | 74 | 1591 | 1648 | v1716 | 1519 | new peak (+8 ± 71 over v1716; predecessor +65) | 430 (38219) | +410 -336 =254, 53.7%, +26 ± 19: kept | 2455 |
| MoE-d2jit-c | dormant | 2312 |  |  | 1694 | 76 | 1597 | 1760 (1760) | v2156 (v2156) | 1557 | dormant (-66 ± 56 vs peak v2156 before dormancy at v2311, at 1623 vs refs; needs > +56) | 430 (38436) | +386 -387 =227, 50.0%, -0 ± 19: reverted | 2312 |
