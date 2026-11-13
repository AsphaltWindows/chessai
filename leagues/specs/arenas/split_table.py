#!/usr/bin/env python3
"""Split table of an experiment's tree lines (11-11, analyst 6.41: "a per-line table of split version, positions at
the split and level, so the constant-rate claim is read directly").
    python3 split_table.py EXPERIMENT [LINE_REGEX]
For each BDT line (runs/*.loc holding bdtN.model), reads every version's header (feature count, the arities, then
branch, ?, split threshold, split limit, splits made, node count) and prints the versions where the split or node
count changed, with the cumulative training positions up to that version (from the line's .loc.log: version v is
trained in cycle v, "round 0: N positions" lines). Read-only."""
import glob, os, re, sys
REPO = '/home/iv/dev/chessai'
exp = sys.argv[1]
rx = re.compile(sys.argv[2]) if len(sys.argv) > 2 else None
runs = os.path.join(REPO, 'leagues', 'experiments', exp, 'runs')


def header(path):
    with open(path) as f:
        nf = int(f.readline())
        for _ in range(nf):
            f.readline()
        vals = [f.readline().strip() for _ in range(6)]
    branch, _, thr, limit, splits, nodes = vals
    return int(splits), int(nodes)


def positions(log):
    pos, cyc = {}, None
    for line in open(log):
        m = re.match(r'== cycle (\d+) train \d', line)
        if m:
            cyc = int(m.group(1))
            continue
        m = re.match(r'round \d+: (\d+) positions', line)
        if m and cyc is not None:
            pos[cyc] = pos.get(cyc, 0) + int(m.group(1))
    return pos


for d in sorted(glob.glob(os.path.join(runs, '*.loc'))):
    line = os.path.basename(d)
    if rx and not rx.search(line):
        continue
    models = {int(m.group(1)): p for p in glob.glob(os.path.join(d, 'bdt*.model'))
              for m in [re.search(r'bdt(\d+)\.model$', p)] if m}
    if not models:
        continue
    pos = positions(d + '.log') if os.path.exists(d + '.log') else {}
    cum, acc = {}, 0
    for v in range(0, max(models) + 1):
        acc += pos.get(v, 0)
        cum[v] = acc
    print(f'== {line}: versions 0-{max(models)}, {cum[max(models)] / 1e6:.1f}M positions')
    prev = None
    for v in sorted(models):
        s, n = header(models[v])
        if prev is None or (s, n) != prev:
            leaves = (n - 1) // 3 * 2 + 1 if n > 1 else 1
            print(f'  v{v:<4} {cum[v] / 1e6:7.2f}M  splits {s:<3} nodes {n:<3} leaves {leaves}')
        prev = (s, n)
