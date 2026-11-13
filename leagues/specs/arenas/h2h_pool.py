#!/usr/bin/env python3
"""Pooled head-to-head of two SETS of tags in an arena (11-12 ~04:40, for diversity-p2d's read: run d against run c
over all their lines, F-P and trees alike; the kind gaps cancel across the cross-kind pairs when both arms carry the
same kinds). Every game between a player of set A and a player of set B at the same ply, scored for A, as
h2h_btg.py scores one pair of tags.
    python3 h2h_pool.py TOUR A1,A2,... B1,B2,..."""
import csv, math, sys
from collections import defaultdict

tour, sa, sb = sys.argv[1], set(sys.argv[2].split(',')), set(sys.argv[3].split(','))
assert not sa & sb, 'the sets overlap'


def tag(p):
    return p.split('.')[0]


def ply(p):
    return p.rsplit('-', 1)[1]


def elo(s):
    s = min(max(s, 1e-4), 1 - 1e-4)
    return -400 * math.log10(1 / s - 1)


acc = defaultdict(lambda: [0, 0.0, 0])
for r in csv.DictReader(open(f'{tour}/games.csv')):
    w, bl, res = r['white'], r['black'], r['result']
    if w.startswith('random') or bl.startswith('random') or ply(w) != ply(bl):
        continue
    s = {'1-0': 1.0, '0-1': 0.0, '1/2-1/2': 0.5}[res]
    if tag(w) in sa and tag(bl) in sb:
        x = s
    elif tag(w) in sb and tag(bl) in sa:
        x = 1 - s
    else:
        continue
    d = acc[ply(w)]
    d[0] += 1; d[1] += x; d[2] += res == '1/2-1/2'
for p in sorted(acc):
    n, sc, dr = acc[p]
    m = sc / n
    se = math.sqrt(max(m * (1 - m), 1e-9) / n)
    lo, hi = elo(m - 1.96 * se), elo(m + 1.96 * se)
    print(f'{"+".join(sorted(sa))} vs {"+".join(sorted(sb))} ply {p}: {n} games, score {m:.3f} (draws {dr}), '
          f'{elo(m):.0f} Elo [{lo:.0f}, {hi:.0f}]')
