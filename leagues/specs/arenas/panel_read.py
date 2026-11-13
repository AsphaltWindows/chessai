#!/usr/bin/env python3
"""Common-opponent (panel) read of tree vs flat per field (11-12 ~06:00; analyst 6.68: architecture verdicts from
strangers, never from the within-run pair, which carries co-adaptation and the kind-vs-kind matchup).
    python3 panel_read.py TOUR FIELD=TREE_TAG,FLAT_TAG [FIELD=TREE_TAG,FLAT_TAG ...] [--exclude FIELD:OTHER ...]
Each field's tree and flat are scored against the panel of every OTHER field's tree and flat (both kinds; --exclude
drops OTHER from FIELD's panel, e.g. a field whose frozen references came from OTHER's lineage), at each ply. The
verdict is Elo(tree vs panel) - Elo(flat vs panel), with a 95% interval from the two binomial SEs (delta method,
draws counted half, conservative). The direct within-field cell, tree vs flat, is printed beside it as the matchup.
Tags as h2h_btg.py: a player's tag is its name up to the first '.'; the ply is the suffix after the last '-'."""
import csv, math, sys
from collections import defaultdict

tour = sys.argv[1]
fields, excl = {}, set()
args = sys.argv[2:]
i = 0
while i < len(args):
    if args[i] == '--exclude':
        f, o = args[i + 1].split(':')
        excl.add((f, o)); i += 2; continue
    f, tags = args[i].split('=')
    t, fl = tags.split(',')
    fields[f] = (t, fl); i += 1
tag_field = {}
for f, (t, fl) in fields.items():
    tag_field[t] = f; tag_field[fl] = f


def tag(p):
    return p.split('.')[0]


def ply(p):
    return p.rsplit('-', 1)[1]


def elo(s):
    s = min(max(s, 1e-4), 1 - 1e-4)
    return -400 * math.log10(1 / s - 1)


# score[(ply, a_tag, b_tag)] = [games, score for a]
acc = defaultdict(lambda: [0, 0.0])
for r in csv.DictReader(open(f'{tour}/games.csv')):
    w, bl, res = r['white'], r['black'], r['result']
    if w.startswith('random') or bl.startswith('random') or ply(w) != ply(bl):
        continue
    s = {'1-0': 1.0, '0-1': 0.0, '1/2-1/2': 0.5}[res]
    tw, tb = tag(w), tag(bl)
    if tw not in tag_field or tb not in tag_field:
        continue
    p = ply(w)
    acc[(p, tw, tb)][0] += 1; acc[(p, tw, tb)][1] += s
    acc[(p, tb, tw)][0] += 1; acc[(p, tb, tw)][1] += 1 - s


def vs(p, a, opp):
    n = sum(acc[(p, a, b)][0] for b in opp)
    sc = sum(acc[(p, a, b)][1] for b in opp)
    if n == 0:
        return None
    m = sc / n
    se_m = math.sqrt(max(m * (1 - m), 1e-9) / n)
    se_e = 400 / math.log(10) * se_m / max(m * (1 - m), 1e-9)
    return n, elo(m), se_e


plies = sorted({k[0] for k in acc})
for p in plies:
    for f, (t, fl) in fields.items():
        panel = [x for g, (tt, ff) in fields.items() if g != f and (f, g) not in excl for x in (tt, ff)]
        a, b, d = vs(p, t, panel), vs(p, fl, panel), vs(p, t, [fl])
        if not a or not b:
            continue
        diff = a[1] - b[1]
        se = math.sqrt(a[2] ** 2 + b[2] ** 2)
        dtxt = f'{d[1]:.0f} [{d[1] - 1.96 * d[2]:.0f}, {d[1] + 1.96 * d[2]:.0f}] ({d[0]} games)' if d else 'none'
        print(f'ply {p} {f}: tree vs panel {a[1]:.0f} ({a[0]} games), flat vs panel {b[1]:.0f} ({b[0]}); '
              f'PANEL tree - flat {diff:.0f} [{diff - 1.96 * se:.0f}, {diff + 1.96 * se:.0f}]; direct {dtxt}')
