#!/usr/bin/env python3
"""Structure events of an experiment's DAG lines (11-11, for router-p1; split_table.py reads only bdt headers).
    python3 dag_events.py EXPERIMENT [LINE_REGEX]
For each line's runs/<line>.loc.log: the cycle and cumulative training positions (from "round N: X positions" lines,
version v trained in cycle v) of every log line that is not a cycle header, a round count or a routine "router close"
check, so router splits ("router split of router ...") and leaf splits show up whatever their wording; then per line
the number of router-close checks and of router splits. Read-only."""
import glob, os, re, sys
REPO = '/home/iv/dev/chessai'
exp = sys.argv[1]
rx = re.compile(sys.argv[2]) if len(sys.argv) > 2 else None
for log in sorted(glob.glob(os.path.join(REPO, 'leagues', 'experiments', exp, 'runs', '*.loc.log'))):
    line = os.path.basename(log)[:-len('.loc.log')]
    if rx and not rx.search(line):
        continue
    cyc, pos, closes, rsplits, events = None, 0, 0, 0, []
    for t in open(log):
        m = re.match(r'== cycle (\d+) train \d', t)
        if m:
            cyc = int(m.group(1))
            continue
        m = re.match(r'round \d+: (\d+) positions', t)
        if m:
            pos += int(m.group(1))
            continue
        if t.startswith('router close'):
            closes += 1
            continue
        if t.startswith('=='):
            continue
        rsplits += t.startswith('router split of router')
        events.append(f'  c{cyc} {pos / 1e6:7.2f}M  {t.strip()[:200]}')
    print(f'== {line}: last cycle {cyc}, {pos / 1e6:.1f}M positions, {closes} router-close checks, {rsplits} router splits')
    if events:
        print('\n'.join(events))
