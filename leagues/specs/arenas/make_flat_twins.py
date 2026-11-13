#!/usr/bin/env python3
"""Write the flat twins' verdict arenas (11-11 ~03:10): every line's final version of flat-p3, flat-p2, flat-p3-b and
flat-p2-b, with lr-settings-new's best flat LR (v136) and the old d4mix (v426), near argmax (T 0.02).
  flat-twins-cold:     all four runs at plies 2 and 3 (40 round-robins)
  flat-twins-b-ply4:   the seed-23 runs at ply 4 (40 round-robins; the seed-20 twins were read at ply 4 in
                       diag/flat-ply4-interim-cold)"""
import csv, json, os
REPO = '/home/iv/dev/chessai'
tags = {'flat-p3': 'fp3', 'flat-p2': 'fp2', 'flat-p3-b': 'fp3b', 'flat-p2-b': 'fp2b'}
def finals(exp):
    rows = list(csv.DictReader(open(os.path.join(REPO, 'leagues', 'experiments', exp, 'lines.csv'))))
    return [{'version': f"{exp}:{r['line']}@{r['version']}", 'name': f"{tags[exp]}.{r['line'][:-4]}@{r['version']}",
             'label': f"{exp} {r['line']} final version v{r['version']}"} for r in rows]
common = [
 {'version': 'lr-settings-new:F-L+lr-rate=0.0125.loc@136', 'name': 'lrs.F-L-0.0125@136', 'label': 'lr-settings-new champion (best ply-2-trained flat LR)'},
 {'file': 'cycles/runs/F-L-d4mix-c/lr426.model', 'kind': 'lr', 'heldout': 'lr', 'name': 'old.F-L-d4mix-c@426', 'label': 'old champion run: depth-4 self-play, 50/50 search labels; its champion at cycle 75'},
]
base = {'seed': 1, 'nice': 15, 'threads': 2, 'jobs': 2,
        'binary': 'models/from-scratch/bin/chessai-hybrid10', 'owncal_binary': 'models/from-scratch/bin/chessai-hybrid9',
        'flags': ['--reports-last', '--sparse-fit', '--policy', 'log-odds-softmax', '--temperature', '0.02'],
        'references': ['random', 'random-1', 'random-2', 'random-3'],
        'stop': {'ci95': 0.0, 'chunk_rounds': 5, 'min_rounds': 40, 'max_rounds': 40}}
a = dict(base, name='flat-twins-cold', dir='leagues/arenas/diag/flat-twins-cold', plies=[2, 3],
         description="The field-ply verdict at plies 2 and 3: flat-p3/-b (ply-3 fields) against flat-p2/-b (ply-2 fields), final versions, two seeds per arm, near argmax.",
         entries=sum((finals(e) for e in tags), []) + common)
b = dict(base, name='flat-twins-b-ply4', dir='leagues/arenas/diag/flat-twins-b-ply4', plies=[4],
         description="The seed-23 twins at ply 4 (flat-p3-b against flat-p2-b, final versions, near argmax); the seed-20 twins are in diag/flat-ply4-interim-cold.",
         entries=finals('flat-p3-b') + finals('flat-p2-b') + common)
for spec in (a, b):
    out = os.path.join(REPO, 'leagues', 'specs', 'arenas', spec['name'] + '.json')
    json.dump(spec, open(out, 'w'), indent=1)
    print(out, len(spec['entries']), 'entries')
