# Cycle 1

**Own-games step, pooled over the lineages: 30 gates, +14604 -10792 =4604, score 56.4%, Elo +44.4 ± 3.6; kept 25 of 30.**

All cycles so far: 30 gates, +14604 -10792 =4604, score 56.4%, Elo +44.4 ± 3.6; kept 25 of 30.

2026-11-06 17:25: budget 480 CPU s per qualified lineage; round sizes d0 1000, d1 860, d2 40, d3 10.
League: retired 0 players to retired.txt (their games stay in the fit), added 60.
Tournament: 5 double round-robins of the 63 active players (19530 games) in 1.5 min, 19530 games saved.

Elo: the lineage's 2-ply entry (its entries at other plies play for information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the fixed references' mean, which is comparable across fits.

| lineage | status | version | self-play | CPU s | Elo | ± | previous | vs refs | decision | own games (positions) | gate | next |
|---|---|---:|---|---:|---:|---:|---:|---:|---|---|---|---:|
| F-L-c | qualified | 424 | 424 x 1000 | 764 | +1648 | 47 |  | +1463 | qualified (first rating) | 1170 (101270) | +538 -355 =107, 59.2%, +64 ± 21: kept | 425 |
| F-L-d0soft-c | qualified | 537 | 537 x 1000 | 817 | +1427 | 39 |  | +1242 | qualified (first rating) | 1170 (109433) | +553 -322 =125, 61.6%, +82 ± 21: kept | 538 |
| F-L-d1jit-c | qualified | 423 | 423 x 860 | 772 | +1539 | 42 |  | +1354 | qualified (first rating) | 1170 (110069) | +601 -246 =153, 67.8%, +129 ± 21: kept | 424 |
| F-L-d1play-c | qualified | 446 | 446 x 860 | 792 | +1606 | 45 |  | +1422 | qualified (first rating) | 1170 (113607) | +519 -330 =151, 59.5%, +66 ± 20: kept | 447 |
| F-L-d2play-c | qualified | 454 | 454 x 40 | 1143 | +1472 | 40 |  | +1288 | qualified (first rating) | 1170 (122546) | +472 -291 =237, 59.1%, +64 ± 19: kept | 455 |
| F-L-d2mix-c | qualified | 346 | 346 x 40 | 982 | +1717 | 52 |  | +1532 | qualified (first rating) | 1170 (101993) | +344 -473 =183, 43.5%, -45 ± 20: reverted | 346 |
| F-L-d2jit-c | qualified | 473 | 473 x 40 | 1082 | +1286 | 38 |  | +1102 | qualified (first rating) | 1170 (128348) | +478 -232 =290, 62.3%, +87 ± 18: kept | 474 |
| F-N-c | qualified | 450 | 450 x 1000 | 864 | +1327 | 38 |  | +1142 | qualified (first rating) | 1170 (104469) | +420 -419 =161, 50.0%, +0 ± 20: kept | 451 |
| MoE-c | qualified | 99 | 99 x 1000 | 924 | +1360 | 38 |  | +1176 | qualified (first rating) | 1170 (113643) | +603 -241 =156, 68.1%, +132 ± 21: kept | 100 |
| FM-c | qualified | 78 | 78 x 1000 | 1058 | +880 | 38 |  | +696 | qualified (first rating) | 1170 (115580) | +528 -236 =236, 64.6%, +104 ± 19: kept | 79 |
| T-L-Ne-c | qualified | 250 | 250 x 1000 | 890 | +1482 | 40 |  | +1298 | qualified (first rating) | 1170 (107038) | +455 -381 =164, 53.7%, +26 ± 20: kept | 251 |
| T-L-Ne-merge-c | qualified | 224 | 224 x 1000 | 522 | +1520 | 41 |  | +1336 | qualified (first rating) | 1170 (102669) | +509 -393 =98, 55.8%, +40 ± 21: kept | 225 |
| T-L-Nc-c | qualified | 227 | 227 x 1000 | 647 | +1513 | 41 |  | +1329 | qualified (first rating) | 1170 (102526) | +457 -409 =134, 52.4%, +17 ± 20: kept | 228 |
| T-L-Nc-merge-c | qualified | 231 | 231 x 1000 | 490 | +1524 | 41 |  | +1340 | qualified (first rating) | 1170 (107479) | +564 -307 =129, 62.8%, +91 ± 21: kept | 232 |
| T-L-LLs-c | qualified | 229 | 229 x 1000 | 656 | +1438 | 39 |  | +1253 | qualified (first rating) | 1170 (106518) | +472 -371 =157, 55.0%, +35 ± 20: kept | 230 |
| T-L-LLs-merge-c | qualified | 232 | 232 x 1000 | 524 | +1519 | 41 |  | +1335 | qualified (first rating) | 1170 (103525) | +532 -353 =115, 59.0%, +63 ± 21: kept | 233 |
| T-L-NLs-c | qualified | 226 | 226 x 1000 | 630 | +1465 | 40 |  | +1281 | qualified (first rating) | 1170 (104096) | +483 -369 =148, 55.7%, +40 ± 20: kept | 227 |
| T-L-NLs-merge-c | qualified | 225 | 225 x 1000 | 467 | +1542 | 42 |  | +1358 | qualified (first rating) | 1170 (98601) | +442 -419 =139, 51.1%, +8 ± 20: kept | 226 |
| T-L-NNs-c | qualified | 230 | 230 x 1000 | 700 | +1479 | 40 |  | +1295 | qualified (first rating) | 1170 (105715) | +408 -454 =138, 47.7%, -16 ± 20: reverted | 230 |
| T-L-NNs-merge-c | qualified | 224 | 224 x 1000 | 518 | +1488 | 40 |  | +1304 | qualified (first rating) | 1170 (104231) | +594 -294 =112, 65.0%, +108 ± 21: kept | 225 |
| T-L-Le-c | qualified | 191 | 191 x 1000 | 579 | +1470 | 40 |  | +1286 | qualified (first rating) | 1170 (106583) | +482 -372 =146, 55.5%, +38 ± 20: kept | 192 |
| T-L-Le-merge-c | qualified | 199 | 199 x 1000 | 525 | +1502 | 41 |  | +1318 | qualified (first rating) | 1170 (106453) | +574 -336 =90, 61.9%, +84 ± 21: kept | 200 |
| T-N-Ne-c | qualified | 190 | 190 x 1000 | 536 | +1280 | 38 |  | +1096 | qualified (first rating) | 1170 (104203) | +414 -419 =167, 49.8%, -2 ± 20: reverted | 190 |
| T-N-Ne-merge-c | qualified | 188 | 188 x 1000 | 436 | +1319 | 38 |  | +1135 | qualified (first rating) | 1170 (103857) | +413 -397 =190, 50.8%, +6 ± 19: kept | 189 |
| T-N-Nc-c | qualified | 194 | 194 x 1000 | 507 | +1314 | 38 |  | +1130 | qualified (first rating) | 1170 (105280) | +405 -452 =143, 47.6%, -16 ± 20: reverted | 194 |
| T-N-Nc-merge-c | qualified | 187 | 187 x 1000 | 448 | +1330 | 38 |  | +1146 | qualified (first rating) | 1170 (100369) | +472 -384 =144, 54.4%, +31 ± 20: kept | 188 |
| T-N-NNs-c | qualified | 184 | 184 x 1000 | 517 | +1287 | 38 |  | +1103 | qualified (first rating) | 1170 (102850) | +443 -395 =162, 52.4%, +17 ± 20: kept | 185 |
| T-N-NNs-merge-c | qualified | 181 | 181 x 1000 | 420 | +1349 | 38 |  | +1164 | qualified (first rating) | 1170 (101144) | +405 -439 =156, 48.3%, -12 ± 20: reverted | 181 |
| T-L-H-c | qualified | 200 | 200 x 1000 | 424 | +1500 | 40 |  | +1316 | qualified (first rating) | 1170 (103320) | +568 -317 =115, 62.5%, +89 ± 21: kept | 201 |
| T-N-H-c | qualified | 207 | 207 x 1000 | 411 | +1161 | 37 |  | +977 | qualified (first rating) | 1170 (105406) | +456 -386 =158, 53.5%, +24 ± 20: kept | 208 |
