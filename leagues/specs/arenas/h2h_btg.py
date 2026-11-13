#!/usr/bin/env python3
"""Pooled head-to-head of two tags in an arena (11-11, analyst 6.44's read rule: the verdict of a one-factor contrast
is the head-to-head score pooled over arm x comparator lines at the same version and ply, with a binomial SE; BT over
a field that mixes training fields or plies can invert it).
    python3 h2h_btg.py TOUR TAG_A TAG_B [TAG_A TAG_B ...]
A player's tag is its name up to the first '.', as arm_btg.py groups them; the ply is the suffix after the last '-'.
Prints, per pair of tags and ply: games, A's score, draws, Elo from the score, and its 95% interval (binomial SE of
the score, draws counted half; conservative with draws)."""
import csv, math, sys
from collections import defaultdict

tour = sys.argv[1]
pairs = list(zip(sys.argv[2::2], sys.argv[3::2]))
games = list(csv.DictReader(open(f'{tour}/games.csv')))


def tag(p):
    return p.split('.')[0]


def ply(p):
    return p.rsplit('-', 1)[1]


def elo(s):
    s = min(max(s, 1e-4), 1 - 1e-4)
    return -400 * math.log10(1 / s - 1)


for a, b in pairs:
    acc = defaultdict(lambda: [0, 0.0, 0])
    for r in games:
        w, bl, res = r['white'], r['black'], r['result']
        if w.startswith('random') or bl.startswith('random') or ply(w) != ply(bl):
            continue
        s = {'1-0': 1.0, '0-1': 0.0, '1/2-1/2': 0.5}[res]
        if tag(w) == a and tag(bl) == b:
            sa = s
        elif tag(w) == b and tag(bl) == a:
            sa = 1 - s
        else:
            continue
        d = acc[ply(w)]
        d[0] += 1; d[1] += sa; d[2] += res == '1/2-1/2'
    for p in sorted(acc):
        n, sc, dr = acc[p]
        m = sc / n
        se = math.sqrt(max(m * (1 - m), 1e-9) / n)
        lo, hi = elo(m - 1.96 * se), elo(m + 1.96 * se)
        print(f'{a} vs {b} ply {p}: {n} games, score {m:.3f} (draws {dr}), {elo(m):.0f} Elo [{lo:.0f}, {hi:.0f}]')
