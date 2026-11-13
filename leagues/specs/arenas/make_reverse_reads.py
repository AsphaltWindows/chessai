#!/usr/bin/env python3
"""Write the reverse-rung reads (theory-2 6.21, 11-11 ~06:55): every line's final version of flat-p3-w4-w3f1 (W4's
finals trained further at ply 3, forget 1) beside its comparator flat-p3-w3f1 (flat-p3's finals trained at ply 3,
forget 1), its start flat-p3-w4 (W4) and flat-p3's finals; near argmax (T 0.02), chessai-hybrid11, the warm reads'
settings.
  reverse-rung-p123:  all four at plies 1, 2 and 3 (30 round-robins; cross-ply games give the value of a ply)
  reverse-rung-p4:    all four at ply 4 (30 round-robins)
Ply 5 is left out (the analyst's predictions are at plies 1-4; a ply-5 read is ~1 CPU-h more)."""
import csv, json, os
REPO = '/home/iv/dev/chessai'
tags = {'flat-p3-w4-w3f1': 'rev', 'flat-p3-w3f1': 'w3f1', 'flat-p3-w4': 'w4', 'flat-p3': 'fp3'}
def finals(exp):
    rows = list(csv.DictReader(open(os.path.join(REPO, 'leagues', 'experiments', exp, 'lines.csv'))))
    return [{'version': f"{exp}:{r['line']}@{r['version']}", 'name': f"{tags[exp]}.{r['line'][:-4]}@{r['version']}",
             'label': f"{exp} {r['line']} final version v{r['version']}"} for r in rows]
base = {'seed': 1, 'nice': 15, 'threads': 2, 'jobs': 2,
        'binary': 'models/from-scratch/bin/chessai-hybrid11', 'owncal_binary': 'models/from-scratch/bin/chessai-hybrid9',
        'flags': ['--reports-last', '--sparse-fit', '--policy', 'log-odds-softmax', '--temperature', '0.02'],
        'references': ['random', 'random-1', 'random-2', 'random-3'],
        'stop': {'ci95': 0.0, 'chunk_rounds': 5, 'min_rounds': 30, 'max_rounds': 30}}
entries = sum((finals(e) for e in tags), [])
a = dict(base, name='reverse-rung-p123', dir='leagues/arenas/diag/reverse-rung-p123', plies=[1, 2, 3],
         description="Reverse rung (theory-2 6.21) at plies 1-3: W4's finals trained further at ply 3 (rev) against its comparator W3f1 (flat-p3's finals at ply 3, forget 1), its start W4 and flat-p3's finals.",
         entries=entries)
b = dict(base, name='reverse-rung-p4', dir='leagues/arenas/diag/reverse-rung-p4', plies=[4],
         description="Reverse rung (theory-2 6.21) at ply 4: does rev keep W4's even-ply gain over W3f1?",
         entries=entries)
for spec in (a, b):
    out = os.path.join(REPO, 'leagues', 'specs', 'arenas', spec['name'] + '.json')
    json.dump(spec, open(out, 'w'), indent=1)
    print(out, len(spec['entries']), 'entries')
