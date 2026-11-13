#!/usr/bin/env python3
"""Two arms against a common panel of strangers (11-12 ~06:05; analyst 6.68 (3): a within-run arm-vs-arm cell carries
co-adaptation, so read each arm against lines neither trained against and difference the results).
    python3 arm_panel.py TOUR A B P1,P2,...      (A, B, P*: tags as h2h_btg.py, the name up to the first '.')
Prints per ply: Elo(A vs panel), Elo(B vs panel), the PANEL difference A - B with a 95% interval from the two binomial
SEs (delta method, draws counted half, conservative), and the direct A vs B cell beside it."""
import csv, math, sys
from collections import defaultdict

tour, A, B = sys.argv[1], sys.argv[2], sys.argv[3]
panel = set(sys.argv[4].split(','))
assert A not in panel and B not in panel


def tag(p):
    return p.split('.')[0]


def ply(p):
    return p.rsplit('-', 1)[1]


def elo(s):
    s = min(max(s, 1e-4), 1 - 1e-4)
    return -400 * math.log10(1 / s - 1)


acc = defaultdict(lambda: [0, 0.0])  # (ply, player tag, opponent group) -> games, score
for r in csv.DictReader(open(f'{tour}/games.csv')):
    w, bl, res = r['white'], r['black'], r['result']
    if w.startswith('random') or bl.startswith('random') or ply(w) != ply(bl):
        continue
    s = {'1-0': 1.0, '0-1': 0.0, '1/2-1/2': 0.5}[res]
    for me, op, x in ((tag(w), tag(bl), s), (tag(bl), tag(w), 1 - s)):
        if me in (A, B):
            g = 'panel' if op in panel else op
            acc[(ply(w), me, g)][0] += 1
            acc[(ply(w), me, g)][1] += x


def est(n, sc):
    m = sc / n
    se_m = math.sqrt(max(m * (1 - m), 1e-9) / n)
    return elo(m), 400 / math.log(10) * se_m / max(m * (1 - m), 1e-9)


for p in sorted({k[0] for k in acc}):
    a, b, d = acc[(p, A, 'panel')], acc[(p, B, 'panel')], acc[(p, A, B)]
    if not a[0] or not b[0]:
        continue
    (ea, sa), (eb, sb) = est(*a), est(*b)
    diff, se = ea - eb, math.sqrt(sa ** 2 + sb ** 2)
    if d[0]:
        ed, sd = est(*d)
        dtxt = f'{ed:.0f} [{ed - 1.96 * sd:.0f}, {ed + 1.96 * sd:.0f}] ({d[0]} games)'
    else:
        dtxt = 'none'
    print(f'ply {p}: {A} vs panel {ea:.0f} ({a[0]} games), {B} vs panel {eb:.0f} ({b[0]}); PANEL {A} - {B} '
          f'{diff:.0f} [{diff - 1.96 * se:.0f}, {diff + 1.96 * se:.0f}]; direct {dtxt}')
