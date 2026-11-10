# Cycle 1 (seeding)


2026-11-07 22:40: the seeding cycle: no training.
League: 0 players to retired.txt (their games stay in the fit), 156 added.
Own calibration: the field's 103 versions calibrated on their own games; 10 made (26 CPU s, 5 s), 93 reused from a seed directory, 0 deleted (versions that left the field); 1,084 MB in cycles/champ/owncal.
Tournament: 5 double round-robins of the 161 players (128,800 games) in 9.9 min.
Leaving the field: 71 entries to retired.txt (F-L-c-v3146-1 F-L-c-v3146-2 F-L-d0soft-c-v2014-2 F-L-d1jit-c-v1451-2 F-L-d1play-c-v3322-2 F-L-d2jit-c-v1651-2 F-L-d2mix-c-v591-2 F-L-d2play-c-v1011-2 F-N-c-v1691-1 F-N-c-v1691-2 F-P-c-v2138-2 F-P-tau0-c-v1964-1 F-P-tau0-c-v1964-2 F-P-tau0-d2jit-c-v3842-1 F-P-tau0-d2jit-c-v3842-2 FM-c-v626-2 MoE-c-v300-2 MoE-d2jit-c-v2156-2 T-L-H-c-v1245-2 T-L-LLs-c-v540-2 T-L-LLs-merge-c-v1251-2 T-L-Le-c-v1176-1 T-L-Le-c-v1176-2 T-L-Le-fresh-c-v682-2 T-L-Le-merge-c-v1857-2 T-L-Le-merge-d2jit-c-v1593-1 T-L-Le-merge-d2jit-c-v1593-2 T-L-NLs-c-v571-2 T-L-NLs-d2jit-c-v1683-1 T-L-NLs-d2jit-c-v1683-2 T-L-NLs-merge-c-v641-2 T-L-NLs-merge-d2jit-c-v2740-1 T-L-NLs-merge-d2jit-c-v2740-2 T-L-NNs-c-v2278-2 T-L-NNs-merge-c-v818-2 T-L-Nc-c-v719-1 T-L-Nc-c-v719-2 T-L-Nc-fresh-c-v2269-1 T-L-Nc-fresh-c-v2269-2 T-L-Nc-merge-c-v1631-2 T-L-Ne-c-v556-2 T-L-Ne-merge-c-v792-2 T-N-H-c-v1104-2 T-N-LLs-merge-c-v725-1 T-N-LLs-merge-c-v725-2 T-N-LNs-merge-c-v417-1 T-N-LNs-merge-c-v417-2 T-N-Lc-merge-c-v368-1 T-N-Lc-merge-c-v368-2 T-N-Le-merge-c-v754-1 T-N-Le-merge-c-v754-2 T-N-NLs-merge-c-v1038-1 T-N-NLs-merge-c-v1038-2 T-N-NNs-c-v632-1 T-N-NNs-c-v632-2 T-N-NNs-merge-c-v1000-1 T-N-NNs-merge-c-v1000-2 T-N-NNs-merge-d2jit-c-v3065-1 T-N-NNs-merge-d2jit-c-v3065-2 T-N-Nc-c-v1505-1 T-N-Nc-c-v1505-2 T-N-Nc-fresh-c-v811-1 T-N-Nc-fresh-c-v811-2 T-N-Nc-merge-c-v631-2 T-N-Nc-merge-d2jit-c-v2109-1 T-N-Nc-merge-d2jit-c-v2109-2 T-N-Ne-c-v637-1 T-N-Ne-c-v637-2 T-N-Ne-merge-c-v908-1 T-N-Ne-merge-c-v908-2 TAN-c-v507-2).

Elo: each entry's in this fit (pure maximum likelihood, random at 0); vs refs: Elo minus the mean of random, random-1, random-2 in this fit. The champion test compares the current version's ply-2 entry with the champion's: it becomes the champion when it is at least 2 standard errors of their difference above it (from the fit's covariance). Patience: self-play CPU s since the lineage's last new champion, of 1920; at that the lineage finishes, its champion frozen at an untaken milestone within 10 of its ply-2 Elo, else retired.

| lineage | status | version | self-play | CPU s | budget | carry | patience | Elo | ± | champion | champion Elo | current vs champion | decision | note | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---|---|---|---|---|---:|
| F-L-c | active | 3146 |  |  |  |  | 0 | 1721 | 25 | v2860 | 1724 |  | first champion: old peak | latest v3146 1721 ± 25, old peak v2860 1724 ± 25 (first run: 1680); trains from v3147, a copy of v2860 |  |  | 3147 |
| F-L-d0soft-c | active | 2367 |  |  |  |  | 0 | 1685 | 24 | v2367 | 1685 |  | first champion: latest | latest v2367 1685 ± 24, old peak v2014 1677 ± 24 (first run: 1669) |  |  | 2367 |
| F-L-d1jit-c | active | 1770 |  |  |  |  | 0 | 1663 | 24 | v1770 | 1663 |  | first champion: latest | latest v1770 1663 ± 24, old peak v1451 1645 ± 24 (first run: 1683) |  |  | 1770 |
| F-L-d1play-c | active | 3595 |  |  |  |  | 0 | 1633 | 24 | v3595 | 1633 |  | first champion: latest | latest v3595 1633 ± 24, old peak v3322 1628 ± 24 (first run: 1605) |  |  | 3595 |
| F-L-d2play-c | active | 1624 |  |  |  |  | 0 | 1614 | 24 | v1624 | 1614 |  | first champion: latest | latest v1624 1614 ± 24, old peak v1011 1604 ± 24 (first run: 1741) |  |  | 1624 |
| F-L-d2mix-c | active | 1075 |  |  |  |  | 0 | 1656 | 24 | v1075 | 1656 |  | first champion: latest | latest v1075 1656 ± 24, old peak v591 1620 ± 24 (first run: 1729) |  |  | 1075 |
| F-L-d2jit-c | active | 3943 |  |  |  |  | 0 | 1702 | 25 | v3943 | 1702 |  | first champion: latest | latest v3943 1702 ± 25, old peak v1651 1680 ± 24 (first run: 1786) |  |  | 3943 |
| F-N-c | active | 1691 |  |  |  |  | 0 | 1415 | 24 | v1061 | 1419 |  | first champion: old peak | latest v1691 1415 ± 24, old peak v1061 1419 ± 24 (first run: 1463); trains from v1692, a copy of v1061 |  |  | 1692 |
| MoE-c | active | 373 |  |  |  |  | 0 | 1682 | 24 | v373 | 1682 |  | first champion: latest | latest v373 1682 ± 24, old peak v300 1641 ± 24 (first run: 1707) |  |  | 373 |
| FM-c | active | 669 |  |  |  |  | 0 | 1632 | 24 | v669 | 1632 |  | first champion: latest | latest v669 1632 ± 24, old peak v626 1617 ± 24 (first run: 1633) |  |  | 669 |
| F-P-c | active | 2808 |  |  |  |  | 0 | 1541 | 23 | v2808 | 1541 |  | first champion: latest | latest v2808 1541 ± 23, old peak v2138 1537 ± 23 (first run: 1560) |  |  | 2808 |
| F-P-tau0-c | active | 1964 |  |  |  |  | 0 | 1557 | 23 | v1962 | 1566 |  | first champion: old peak | latest v1964 1557 ± 23, old peak v1962 1566 ± 23 (first run: 1555); trains from v1965, a copy of v1962 |  |  | 1965 |
| F-P-tau0-d2jit-c | active | 3842 |  |  |  |  | 0 | 1747 | 26 | v2036 | 1782 |  | first champion: old peak | latest v3842 1747 ± 26, old peak v2036 1782 ± 27 (first run: 1781); trains from v3843, a copy of v2036 |  |  | 3843 |
| TAN-c | active | 929 |  |  |  |  | 0 | 1249 | 25 | v929 | 1249 |  | first champion: latest | latest v929 1249 ± 25, old peak v507 1225 ± 25 (first run: 1227) |  |  | 929 |
| T-L-Ne-c | active | 1428 |  |  |  |  | 0 | 1631 | 24 | v1428 | 1631 |  | first champion: latest | latest v1428 1631 ± 24, old peak v556 1589 ± 23 (first run: 1596) |  |  | 1428 |
| T-L-Ne-merge-c | active | 1014 |  |  |  |  | 0 | 1676 | 24 | v1014 | 1676 |  | first champion: latest | latest v1014 1676 ± 24, old peak v792 1651 ± 24 (first run: 1691) |  |  | 1014 |
| T-L-Ne-fresh-c | active | 1043 |  |  |  |  | 0 | 1659 | 24 | v1043 | 1659 |  | first champion: latest | latest v1043 1659 ± 24 (also its old peak) |  |  | 1043 |
| T-L-Ne-pre-c | active | 830 |  |  |  |  | 0 | 1664 | 24 | v830 | 1664 |  | first champion: latest | latest v830 1664 ± 24 (also its old peak) |  |  | 830 |
| T-L-Nc-c | active | 719 |  |  |  |  | 0 | 1621 | 24 | v516 | 1622 |  | first champion: old peak | latest v719 1621 ± 24, old peak v516 1622 ± 24 (first run: 1635); trains from v720, a copy of v516 |  |  | 720 |
| T-L-Nc-merge-c | active | 1885 |  |  |  |  | 0 | 1705 | 25 | v1885 | 1705 |  | first champion: latest | latest v1885 1705 ± 25, old peak v1631 1698 ± 25 (first run: 1705) |  |  | 1885 |
| T-L-Nc-fresh-c | active | 2269 |  |  |  |  | 0 | 1599 | 23 | v2061 | 1608 |  | first champion: old peak | latest v2269 1599 ± 23, old peak v2061 1608 ± 24 (first run: 1602); trains from v2270, a copy of v2061 |  |  | 2270 |
| D-L-Nc-c | active | 1546 |  |  |  |  | 0 | 1684 | 24 | v1546 | 1684 |  | first champion: latest | latest v1546 1684 ± 24 (also its old peak) |  |  | 1546 |
| T-L-LLs-c | active | 651 |  |  |  |  | 0 | 1623 | 24 | v651 | 1623 |  | first champion: latest | latest v651 1623 ± 24, old peak v540 1597 ± 23 (first run: 1619) |  |  | 651 |
| T-L-LLs-merge-c | active | 1761 |  |  |  |  | 0 | 1632 | 24 | v1761 | 1632 |  | first champion: latest | latest v1761 1632 ± 24, old peak v1251 1542 ± 23 (first run: 1595) |  |  | 1761 |
| T-L-NLs-c | active | 1433 |  |  |  |  | 0 | 1536 | 23 | v1433 | 1536 |  | first champion: latest | latest v1433 1536 ± 23, old peak v571 1509 ± 23 (first run: 1556) |  |  | 1433 |
| T-L-NLs-merge-c | active | 1122 |  |  |  |  | 0 | 1692 | 25 | v1122 | 1692 |  | first champion: latest | latest v1122 1692 ± 25, old peak v641 1587 ± 23 (first run: 1645) |  |  | 1122 |
| T-L-NNs-c | active | 2281 |  |  |  |  | 0 | 1650 | 24 | v2281 | 1650 |  | first champion: latest | latest v2281 1650 ± 24, old peak v2278 1647 ± 24 (first run: 1616) |  |  | 2281 |
| T-L-NNs-merge-c | active | 1070 |  |  |  |  | 0 | 1715 | 25 | v1070 | 1715 |  | first champion: latest | latest v1070 1715 ± 25, old peak v818 1635 ± 24 (first run: 1692) |  |  | 1070 |
| T-L-Le-c | active | 1176 |  |  |  |  | 0 | 1620 | 24 | v1081 | 1623 |  | first champion: old peak | latest v1176 1620 ± 24, old peak v1081 1623 ± 24 (first run: 1645); trains from v1177, a copy of v1081 |  |  | 1177 |
| T-L-Le-merge-c | active | 2053 |  |  |  |  | 0 | 1660 | 24 | v2053 | 1660 |  | first champion: latest | latest v2053 1660 ± 24, old peak v1857 1596 ± 23 (first run: 1678) |  |  | 2053 |
| T-L-Le-fresh-c | active | 742 |  |  |  |  | 0 | 1676 | 24 | v742 | 1676 |  | first champion: latest | latest v742 1676 ± 24, old peak v682 1638 ± 24 (first run: 1624) |  |  | 742 |
| T-L-Le-pre-c | active | 938 |  |  |  |  | 0 | 1582 | 23 | v938 | 1582 |  | first champion: latest | latest v938 1582 ± 23 (also its old peak) |  |  | 938 |
| T-N-Ne-c | active | 637 |  |  |  |  | 0 | 1194 | 25 | v574 | 1274 |  | first champion: old peak | latest v637 1194 ± 25, old peak v574 1274 ± 24 (first run: 1359); trains from v638, a copy of v574 |  |  | 638 |
| T-N-Ne-merge-c | active | 908 |  |  |  |  | 0 | 1324 | 24 | v697 | 1336 |  | first champion: old peak | latest v908 1324 ± 24, old peak v697 1336 ± 24 (first run: 1435); trains from v909, a copy of v697 |  |  | 909 |
| T-N-Nc-c | active | 1505 |  |  |  |  | 0 | 1311 | 24 | v472 | 1336 |  | first champion: old peak | latest v1505 1311 ± 24, old peak v472 1336 ± 24 (first run: 1379); trains from v1506, a copy of v472 |  |  | 1506 |
| T-N-Nc-merge-c | active | 2130 |  |  |  |  | 0 | 1429 | 24 | v2130 | 1429 |  | first champion: latest | latest v2130 1429 ± 24, old peak v631 1402 ± 24 (first run: 1436) |  |  | 2130 |
| T-N-Nc-fresh-c | active | 811 |  |  |  |  | 0 | 1240 | 25 | v624 | 1298 |  | first champion: old peak | latest v811 1240 ± 25, old peak v624 1298 ± 24 (first run: 1338); trains from v812, a copy of v624 |  |  | 812 |
| T-N-NNs-c | active | 632 |  |  |  |  | 0 | 855 | 28 | v341 | 1100 |  | first champion: old peak | latest v632 855 ± 28, old peak v341 1100 ± 26 (first run: 1230); trains from v633, a copy of v341 |  |  | 633 |
| T-N-NNs-merge-c | active | 1000 |  |  |  |  | 0 | 1354 | 24 | v855 | 1428 |  | first champion: old peak | latest v1000 1354 ± 24, old peak v855 1428 ± 24 (first run: 1455); trains from v1001, a copy of v855 |  |  | 1001 |
| T-N-Le-merge-c | active | 754 |  |  |  |  | 0 | 1297 | 24 | v700 | 1339 |  | first champion: old peak | latest v754 1297 ± 24, old peak v700 1339 ± 24 (first run: 1398); trains from v755, a copy of v700 |  |  | 755 |
| T-N-Lc-merge-c | active | 368 |  |  |  |  | 0 | 1344 | 24 | v287 | 1357 |  | first champion: old peak | latest v368 1344 ± 24, old peak v287 1357 ± 24 (first run: 1425); trains from v369, a copy of v287 |  |  | 369 |
| T-N-LLs-merge-c | active | 725 |  |  |  |  | 0 | 1025 | 26 | v469 | 1372 |  | first champion: old peak | latest v725 1025 ± 26, old peak v469 1372 ± 24 (first run: 1354); trains from v726, a copy of v469 |  |  | 726 |
| T-N-NLs-merge-c | active | 1038 |  |  |  |  | 0 | 1013 | 26 | v787 | 1384 |  | first champion: old peak | latest v1038 1013 ± 26, old peak v787 1384 ± 24 (first run: 1386); trains from v1039, a copy of v787 |  |  | 1039 |
| T-N-LNs-merge-c | active | 417 |  |  |  |  | 0 | 1246 | 25 | v270 | 1253 |  | first champion: old peak | latest v417 1246 ± 25, old peak v270 1253 ± 25 (first run: 1335); trains from v418, a copy of v270 |  |  | 418 |
| F-N-d2jit-c | active | 2426 |  |  |  |  | 0 | 1306 | 24 | v2426 | 1306 |  | first champion: latest | latest v2426 1306 ± 24 (also its old peak) |  |  | 2426 |
| T-N-NNs-merge-d2jit-c | active | 3065 |  |  |  |  | 0 | 1131 | 25 | v2345 | 1221 |  | first champion: old peak | latest v3065 1131 ± 25, old peak v2345 1221 ± 25 (first run: 1194); trains from v3066, a copy of v2345 |  |  | 3066 |
| T-L-Le-merge-d2jit-c | active | 1593 |  |  |  |  | 0 | 1655 | 24 | v1404 | 1738 |  | first champion: old peak | latest v1593 1655 ± 24, old peak v1404 1738 ± 26 (first run: 1756); trains from v1594, a copy of v1404 |  |  | 1594 |
| T-L-NLs-d2jit-c | active | 1683 |  |  |  |  | 0 | 1554 | 23 | v1462 | 1608 |  | first champion: old peak | latest v1683 1554 ± 23, old peak v1462 1608 ± 24 (first run: 1637); trains from v1684, a copy of v1462 |  |  | 1684 |
| T-N-Nc-merge-d2jit-c | active | 2109 |  |  |  |  | 0 | 1196 | 25 | v1441 | 1348 |  | first champion: old peak | latest v2109 1196 ± 25, old peak v1441 1348 ± 24 (first run: 1256); trains from v2110, a copy of v1441 |  |  | 2110 |
| T-L-NLs-merge-d2jit-c | active | 2740 |  |  |  |  | 0 | 1582 | 23 | v2454 | 1643 |  | first champion: old peak | latest v2740 1582 ± 23, old peak v2454 1643 ± 24 (first run: 1649); trains from v2741, a copy of v2454 |  |  | 2741 |
| F-L-d4mix-c | active | 324 |  |  |  |  | 0 | 1730 | 25 | v324 | 1730 |  | first champion: latest | latest v324 1730 ± 25 (also its old peak) |  |  | 324 |
| MoE-d2jit-c | active | 2313 |  |  |  |  | 0 | 1642 | 24 | v2313 | 1642 |  | first champion: latest | latest v2313 1642 ± 24, old peak v2156 1634 ± 24 (first run: 1755) |  |  | 2313 |
| T-L-H-c | active | 1526 |  |  |  |  | 0 | 1644 | 24 | v1526 | 1644 |  | first champion: latest | latest v1526 1644 ± 24, old peak v1245 1619 ± 24 (first run: 1644) |  |  | 1526 |
| T-N-H-c | active | 1734 |  |  |  |  | 0 | 1312 | 24 | v1734 | 1312 |  | first champion: latest | latest v1734 1312 ± 24, old peak v1104 1296 ± 24 (first run: 1350) |  |  | 1734 |
| MoE-N-L-c | active |  |  |  |  |  | 0 |  |  |  |  |  | waits for a first version |  |  |  |  |
| MoE-L-N-c | active |  |  |  |  |  | 0 |  |  |  |  |  | waits for a first version |  |  |  |  |
| MoE-N-N-c | active |  |  |  |  |  | 0 |  |  |  |  |  | waits for a first version |  |  |  |  |
| T-L-Te-merge-c | active |  |  |  |  |  | 0 |  |  |  |  |  | waits for a first version |  |  |  |  |
| MoE-8-c | active |  |  |  |  |  | 0 |  |  |  |  |  | waits for a first version |  |  |  |  |
| T-L-Ne-share-c | active |  |  |  |  |  | 0 |  |  |  |  |  | waits for a first version |  |  |  |  |
