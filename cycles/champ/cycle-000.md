# Cycle 0: the seeding plan (2026-11-07 22:30)

Copied cycles/runs/F-L-d4mix-c/lr43.model to cycles/champ/refs/F-L-d4mix-c-lr43.model.
Made a new league in selfplay/league-champ.
Added random-1, random-2, random-3, ref-d4mix43-1.
Wrote the cycles' rules into the header of the league's players.txt.

The seeding cycle (cycle 1) trains nothing: 161 players (5 references, 156 lineage entries) play 5 double round-robins, 128,800 games. Each lineage enters with its latest version at plies 1 and 2 and its old peak at ply 2; the better of the two at that ply becomes its first champion.

| lineage | latest | latest's ply-2 Elo (first run) | old peak | peak's ply-2 Elo (first run) | resume from the latest: files | resume from the peak (copied as the next version): files |
|---|---|---|---|---|---|---|
| F-L-c | v3146 | 1661 ± 50 | v2860 | 1680 ± 30 | F-L-c/lr3146.model | F-L-c/lr2860.model |
| F-L-d0soft-c | v2367 | 1628 ± 47 | v2014 | 1669 ± 43 | F-L-d0soft-c/lr2367.model | F-L-d0soft-c/lr2014.model |
| F-L-d1jit-c | v1770 | 1634 ± 29 | v1451 | 1683 ± 44 | F-L-d1jit-c/lr1770.model | F-L-d1jit-c/lr1451.model |
| F-L-d1play-c | v3595 | 1535 ± 47 | v3322 | 1605 ± 48 | F-L-d1play-c/lr3595.model | F-L-d1play-c/lr3322.model |
| F-L-d2play-c | v1624 | 1651 ± 34 | v1011 | 1741 ± 36 | F-L-d2play-c/lr1624.model | F-L-d2play-c/lr1011.model |
| F-L-d2mix-c | v1075 | 1704 ± 23 | v591 | 1729 ± 69 | F-L-d2mix-c/lr1075.model | F-L-d2mix-c/lr591.model |
| F-L-d2jit-c | v3943 | 1754 ± 25 | v1651 | 1786 ± 51 | F-L-d2jit-c/lr3943.model | F-L-d2jit-c/lr1651.model |
| F-N-c | v1691 | 1354 ± 8 | v1061 | 1463 ± 40 | F-N-c/cnbp1691.model | F-N-c/cnbp1061.model |
| MoE-c | v373 | 1652 ± 9 | v300 | 1707 ± 45 | MoE-c/moe373.model | MoE-c/moe300.model |
| FM-c | v669 | 1590 ± 18 | v626 | 1633 ± 49 | FM-c/fm669.model | FM-c/fm626.model |
| F-P-c | v2808 | 1470 ± 13 | v2138 | 1560 ± 47 | F-P-c/probit2808.model | F-P-c/probit2138.model |
| F-P-tau0-c | v1964 | 1520 ± 39 | v1962 | 1555 ± 53 | F-P-tau0-c/probit1964.model | F-P-tau0-c/probit1962.model |
| F-P-tau0-d2jit-c | v3842 | - | v2036 | 1781 ± 64 | F-P-tau0-d2jit-c/probit3842.model | F-P-tau0-d2jit-c/probit2036.model |
| TAN-c | v929 | 1180 ± 12 | v507 | 1227 ± 39 | TAN-c/tan929.model, TAN-c/tan929.state, TAN-c/tan928.state | TAN-c/tan507.model, TAN-c/cycle-states/tan507.state, TAN-c/tan929.state |
| T-L-Ne-c | v1428 | 1575 ± 36 | v556 | 1596 ± 23 | T-L-Ne-c/bdt1428.model, T-L-Ne-c/bdt1428.state, T-L-Ne-c/bdt1427.state | T-L-Ne-c/bdt556.model, T-L-Ne-c/bdt556.state, T-L-Ne-c/bdt1428.state |
| T-L-Ne-merge-c | v1014 | 1644 ± 42 | v792 | 1691 ± 47 | T-L-Ne-merge-c/bdt1014.model, T-L-Ne-merge-c/bdt1014.state, T-L-Ne-merge-c/bdt1013.state | T-L-Ne-merge-c/bdt792.model, T-L-Ne-merge-c/bdt792.state, T-L-Ne-merge-c/bdt1014.state |
| T-L-Ne-fresh-c | v1043 | 1595 ± 54 | v1043 | 1595 ± 54 | T-L-Ne-fresh-c/bdt1043.model, T-L-Ne-fresh-c/bdt1043.state, T-L-Ne-fresh-c/bdt1042.state | the latest (one candidate) |
| T-L-Ne-pre-c | v830 | 1644 ± 67 | v830 | 1644 ± 67 | T-L-Ne-pre-c/bdt830.model, T-L-Ne-pre-c/bdt830.state, T-L-Ne-pre-c/bdt829.state | the latest (one candidate) |
| T-L-Nc-c | v719 | 1565 ± 38 | v516 | 1635 ± 43 | T-L-Nc-c/bdt719.model, T-L-Nc-c/bdt719.state, T-L-Nc-c/bdt718.state | T-L-Nc-c/bdt516.model, T-L-Nc-c/cycle-states/bdt516.state, T-L-Nc-c/bdt719.state |
| T-L-Nc-merge-c | v1885 | 1668 ± 32 | v1631 | 1705 ± 47 | T-L-Nc-merge-c/bdt1885.model, T-L-Nc-merge-c/bdt1885.state, T-L-Nc-merge-c/bdt1884.state | T-L-Nc-merge-c/bdt1631.model, T-L-Nc-merge-c/cycle-states/bdt1631.state, T-L-Nc-merge-c/bdt1885.state |
| T-L-Nc-fresh-c | v2269 | - | v2061 | 1602 ± 72 | T-L-Nc-fresh-c/bdt2269.model, T-L-Nc-fresh-c/bdt2269.state, T-L-Nc-fresh-c/bdt2268.state | T-L-Nc-fresh-c/bdt2061.model, T-L-Nc-fresh-c/cycle-states/bdt2061.state, T-L-Nc-fresh-c/bdt2269.state |
| D-L-Nc-c | v1546 | 1642 ± 74 | v1546 | 1642 ± 74 | D-L-Nc-c/bdt1546.model, D-L-Nc-c/bdt1546.state, D-L-Nc-c/bdt1545.state | the latest (one candidate) |
| T-L-LLs-c | v651 | 1561 ± 8 | v540 | 1619 ± 42 | T-L-LLs-c/bdt651.model, T-L-LLs-c/bdt651.state, T-L-LLs-c/bdt650.state | T-L-LLs-c/bdt540.model, T-L-LLs-c/cycle-states/bdt540.state, T-L-LLs-c/bdt651.state |
| T-L-LLs-merge-c | v1761 | 1576 ± 30 | v1251 | 1595 ± 42 | T-L-LLs-merge-c/bdt1761.model, T-L-LLs-merge-c/bdt1761.state, T-L-LLs-merge-c/bdt1760.state | T-L-LLs-merge-c/bdt1251.model, T-L-LLs-merge-c/cycle-states/bdt1251.state, T-L-LLs-merge-c/bdt1761.state |
| T-L-NLs-c | v1433 | 1490 ± 14 | v571 | 1556 ± 46 | T-L-NLs-c/bdt1433.model, T-L-NLs-c/bdt1433.state, T-L-NLs-c/bdt1432.state | T-L-NLs-c/bdt571.model, T-L-NLs-c/cycle-states/bdt571.state, T-L-NLs-c/bdt1433.state |
| T-L-NLs-merge-c | v1122 | 1618 ± 41 | v641 | 1645 ± 66 | T-L-NLs-merge-c/bdt1122.model, T-L-NLs-merge-c/bdt1122.state, T-L-NLs-merge-c/bdt1121.state | T-L-NLs-merge-c/bdt641.model, T-L-NLs-merge-c/cycle-states/bdt641.state, T-L-NLs-merge-c/bdt1122.state |
| T-L-NNs-c | v2281 | 1580 ± 55 | v2278 | 1616 ± 55 | T-L-NNs-c/bdt2281.model, T-L-NNs-c/bdt2281.state, T-L-NNs-c/bdt2280.state | T-L-NNs-c/bdt2278.model, T-L-NNs-c/bdt2278.state, T-L-NNs-c/bdt2281.state |
| T-L-NNs-merge-c | v1070 | 1623 ± 40 | v818 | 1692 ± 48 | T-L-NNs-merge-c/bdt1070.model, T-L-NNs-merge-c/bdt1070.state, T-L-NNs-merge-c/bdt1069.state | T-L-NNs-merge-c/bdt818.model, T-L-NNs-merge-c/cycle-states/bdt818.state, T-L-NNs-merge-c/bdt1070.state |
| T-L-Le-c | v1176 | 1572 ± 45 | v1081 | 1645 ± 44 | T-L-Le-c/bdt1176.model, T-L-Le-c/bdt1176.state, T-L-Le-c/bdt1175.state | T-L-Le-c/bdt1081.model, T-L-Le-c/cycle-states/bdt1081.state, T-L-Le-c/bdt1176.state |
| T-L-Le-merge-c | v2053 | 1630 ± 40 | v1857 | 1678 ± 44 | T-L-Le-merge-c/bdt2053.model, T-L-Le-merge-c/bdt2053.state, T-L-Le-merge-c/bdt2052.state | T-L-Le-merge-c/bdt1857.model, T-L-Le-merge-c/cycle-states/bdt1857.state, T-L-Le-merge-c/bdt2053.state |
| T-L-Le-fresh-c | v742 | 1604 ± 35 | v682 | 1624 ± 49 | T-L-Le-fresh-c/bdt742.model, T-L-Le-fresh-c/bdt742.state, T-L-Le-fresh-c/bdt741.state | T-L-Le-fresh-c/bdt682.model, T-L-Le-fresh-c/cycle-states/bdt682.state, T-L-Le-fresh-c/bdt742.state |
| T-L-Le-pre-c | v938 | 1555 ± 48 | v938 | 1555 ± 48 | T-L-Le-pre-c/bdt938.model, T-L-Le-pre-c/bdt938.state, T-L-Le-pre-c/bdt937.state | the latest (one candidate) |
| T-N-Ne-c | v637 | 1240 ± 7 | v574 | 1359 ± 43 | T-N-Ne-c/bdt637.model, T-N-Ne-c/bdt637.state, T-N-Ne-c/bdt636.state | T-N-Ne-c/bdt574.model, T-N-Ne-c/cycle-states/bdt574.state, T-N-Ne-c/bdt637.state |
| T-N-Ne-merge-c | v908 | 1364 ± 28 | v697 | 1435 ± 32 | T-N-Ne-merge-c/bdt908.model, T-N-Ne-merge-c/bdt908.state, T-N-Ne-merge-c/bdt907.state | T-N-Ne-merge-c/bdt697.model, T-N-Ne-merge-c/bdt697.state, T-N-Ne-merge-c/bdt908.state |
| T-N-Nc-c | v1505 | 1325 ± 25 | v472 | 1379 ± 47 | T-N-Nc-c/bdt1505.model, T-N-Nc-c/bdt1505.state, T-N-Nc-c/bdt1504.state | T-N-Nc-c/bdt472.model, T-N-Nc-c/bdt472.state, T-N-Nc-c/bdt1505.state |
| T-N-Nc-merge-c | v2130 | 1388 ± 47 | v631 | 1436 ± 29 | T-N-Nc-merge-c/bdt2130.model, T-N-Nc-merge-c/bdt2130.state, T-N-Nc-merge-c/bdt2129.state | T-N-Nc-merge-c/bdt631.model, T-N-Nc-merge-c/cycle-states/bdt631.state, T-N-Nc-merge-c/bdt2130.state |
| T-N-Nc-fresh-c | v811 | 1329 ± 45 | v624 | 1338 ± 43 | T-N-Nc-fresh-c/bdt811.model, T-N-Nc-fresh-c/bdt811.state, T-N-Nc-fresh-c/bdt810.state | T-N-Nc-fresh-c/bdt624.model, T-N-Nc-fresh-c/cycle-states/bdt624.state, T-N-Nc-fresh-c/bdt811.state |
| T-N-NNs-c | v632 | 1134 ± 36 | v341 | 1230 ± 65 | T-N-NNs-c/bdt632.model, T-N-NNs-c/bdt632.state, T-N-NNs-c/bdt631.state | T-N-NNs-c/bdt341.model, T-N-NNs-c/cycle-states/bdt341.state, T-N-NNs-c/bdt632.state |
| T-N-NNs-merge-c | v1000 | 1331 ± 36 | v855 | 1455 ± 43 | T-N-NNs-merge-c/bdt1000.model, T-N-NNs-merge-c/bdt1000.state, T-N-NNs-merge-c/bdt999.state | T-N-NNs-merge-c/bdt855.model, T-N-NNs-merge-c/cycle-states/bdt855.state, T-N-NNs-merge-c/bdt1000.state |
| T-N-Le-merge-c | v754 | 1312 ± 12 | v700 | 1398 ± 36 | T-N-Le-merge-c/bdt754.model, T-N-Le-merge-c/bdt754.state, T-N-Le-merge-c/bdt753.state | T-N-Le-merge-c/bdt700.model, T-N-Le-merge-c/cycle-states/bdt700.state, T-N-Le-merge-c/bdt754.state |
| T-N-Lc-merge-c | v368 | 1372 ± 26 | v287 | 1425 ± 39 | T-N-Lc-merge-c/bdt368.model, T-N-Lc-merge-c/bdt368.state, T-N-Lc-merge-c/bdt367.state | T-N-Lc-merge-c/bdt287.model, T-N-Lc-merge-c/cycle-states/bdt287.state, T-N-Lc-merge-c/bdt368.state |
| T-N-LLs-merge-c | v725 | 1083 ± 12 | v469 | 1354 ± 38 | T-N-LLs-merge-c/bdt725.model, T-N-LLs-merge-c/bdt725.state, T-N-LLs-merge-c/bdt724.state | T-N-LLs-merge-c/bdt469.model, T-N-LLs-merge-c/cycle-states/bdt469.state, T-N-LLs-merge-c/bdt725.state |
| T-N-NLs-merge-c | v1038 | 1071 ± 42 | v787 | 1386 ± 26 | T-N-NLs-merge-c/bdt1038.model, T-N-NLs-merge-c/bdt1038.state, T-N-NLs-merge-c/bdt1037.state | T-N-NLs-merge-c/bdt787.model, T-N-NLs-merge-c/cycle-states/bdt787.state, T-N-NLs-merge-c/bdt1038.state |
| T-N-LNs-merge-c | v417 | 1217 ± 35 | v270 | 1335 ± 38 | T-N-LNs-merge-c/bdt417.model, T-N-LNs-merge-c/bdt417.state, T-N-LNs-merge-c/bdt416.state | T-N-LNs-merge-c/bdt270.model, T-N-LNs-merge-c/cycle-states/bdt270.state, T-N-LNs-merge-c/bdt417.state |
| F-N-d2jit-c | v2426 | 1293 ± 50 | v2426 | 1293 ± 50 | F-N-d2jit-c/cnbp2426.model | the latest (one candidate) |
| T-N-NNs-merge-d2jit-c | v3065 | 1151 ± 62 | v2345 | 1194 ± 44 | T-N-NNs-merge-d2jit-c/bdt3065.model, T-N-NNs-merge-d2jit-c/bdt3065.state, T-N-NNs-merge-d2jit-c/bdt3064.state | T-N-NNs-merge-d2jit-c/bdt2345.model, T-N-NNs-merge-d2jit-c/cycle-states/bdt2345.state, T-N-NNs-merge-d2jit-c/bdt3065.state |
| T-L-Le-merge-d2jit-c | v1593 | 1626 ± 19 | v1404 | 1756 ± 60 | T-L-Le-merge-d2jit-c/bdt1593.model, T-L-Le-merge-d2jit-c/bdt1593.state, T-L-Le-merge-d2jit-c/bdt1592.state | T-L-Le-merge-d2jit-c/bdt1404.model, T-L-Le-merge-d2jit-c/cycle-states/bdt1404.state, T-L-Le-merge-d2jit-c/bdt1593.state |
| T-L-NLs-d2jit-c | v1683 | 1619 ± 76 | v1462 | 1637 ± 43 | T-L-NLs-d2jit-c/bdt1683.model, T-L-NLs-d2jit-c/bdt1683.state, T-L-NLs-d2jit-c/bdt1682.state | T-L-NLs-d2jit-c/bdt1462.model, T-L-NLs-d2jit-c/cycle-states/bdt1462.state, T-L-NLs-d2jit-c/bdt1683.state |
| T-N-Nc-merge-d2jit-c | v2109 | - | v1441 | 1256 ± 59 | T-N-Nc-merge-d2jit-c/bdt2109.model, T-N-Nc-merge-d2jit-c/bdt2109.state, T-N-Nc-merge-d2jit-c/bdt2108.state | T-N-Nc-merge-d2jit-c/bdt1441.model, T-N-Nc-merge-d2jit-c/cycle-states/bdt1441.state, T-N-Nc-merge-d2jit-c/bdt2109.state |
| T-L-NLs-merge-d2jit-c | v2740 | 1567 ± 54 | v2454 | 1649 ± 73 | T-L-NLs-merge-d2jit-c/bdt2740.model, T-L-NLs-merge-d2jit-c/bdt2740.state, T-L-NLs-merge-d2jit-c/bdt2739.state | T-L-NLs-merge-d2jit-c/bdt2454.model, T-L-NLs-merge-d2jit-c/cycle-states/bdt2454.state, T-L-NLs-merge-d2jit-c/bdt2740.state |
| F-L-d4mix-c | v324 | 1742 ± 67 | v324 | 1742 ± 67 | F-L-d4mix-c/lr324.model | the latest (one candidate) |
| MoE-d2jit-c | v2313 | 1653 ± 39 | v2156 | 1755 ± 78 | MoE-d2jit-c/moe2313.model | MoE-d2jit-c/moe2156.model |
| T-L-H-c | v1526 | 1590 ± 28 | v1245 | 1644 ± 43 | T-L-H-c/ht1526.model, T-L-H-c/ht1526.state, T-L-H-c/ht1525.state | T-L-H-c/ht1245.model, T-L-H-c/cycle-states/ht1245.state, T-L-H-c/ht1526.state |
| T-N-H-c | v1734 | 1264 ± 26 | v1104 | 1350 ± 45 | T-N-H-c/ht1734.model, T-N-H-c/ht1734.state, T-N-H-c/ht1733.state | T-N-H-c/ht1104.model, T-N-H-c/cycle-states/ht1104.state, T-N-H-c/ht1734.state |
| MoE-N-L-c | - | - | - | - | no version yet: enters after its first training cycle | - |
| MoE-L-N-c | - | - | - | - | no version yet: enters after its first training cycle | - |
| MoE-N-N-c | - | - | - | - | no version yet: enters after its first training cycle | - |
| T-L-Te-merge-c | - | - | - | - | no version yet: enters after its first training cycle | - |
| MoE-8-c | - | - | - | - | no version yet: enters after its first training cycle | - |
| T-L-Ne-share-c | - | - | - | - | no version yet: enters after its first training cycle | - |
