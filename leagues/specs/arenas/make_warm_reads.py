#!/usr/bin/env python3
"""Write the warm-start reads (11-11 ~04:00): every line's final version of flat-p3-w4 (ply-4 field), flat-p3-w3
(ply-3 field), both warm-started from flat-p3's finals v199, and flat-p2-w3 (flat-p2's finals v244 warm-started into
a ply-3 field), beside the starts (flat-p3 v199, flat-p2 v244); near argmax (T 0.02), chessai-hybrid11.
  warm-reads-p23:  all five at plies 2 and 3 (30 round-robins)
  warm-reads-p4:   W4, W3 and flat-p3's finals at ply 4 (30 round-robins)"""
import csv, json, os
REPO = '/home/iv/dev/chessai'
tags = {'flat-p3-w4': 'w4', 'flat-p3-w3': 'w3', 'flat-p2-w3': 'l3', 'flat-p3': 'fp3', 'flat-p2': 'fp2'}
def finals(exp):
    rows = list(csv.DictReader(open(os.path.join(REPO, 'leagues', 'experiments', exp, 'lines.csv'))))
    return [{'version': f"{exp}:{r['line']}@{r['version']}", 'name': f"{tags[exp]}.{r['line'][:-4]}@{r['version']}",
             'label': f"{exp} {r['line']} final version v{r['version']}"} for r in rows]
base = {'seed': 1, 'nice': 15, 'threads': 2, 'jobs': 2,
        'binary': 'models/from-scratch/bin/chessai-hybrid11', 'owncal_binary': 'models/from-scratch/bin/chessai-hybrid9',
        'flags': ['--reports-last', '--sparse-fit', '--policy', 'log-odds-softmax', '--temperature', '0.02'],
        'references': ['random', 'random-1', 'random-2', 'random-3'],
        'stop': {'ci95': 0.0, 'chunk_rounds': 5, 'min_rounds': 30, 'max_rounds': 30}}
a = dict(base, name='warm-reads-p23', dir='leagues/arenas/diag/warm-reads-p23', plies=[2, 3],
         description="Warm-start reads at plies 2 and 3: W4, W3 and the ladder rung flat-p2-w3 against their starts (flat-p3 v199, flat-p2 v244).",
         entries=sum((finals(e) for e in tags), []))
b = dict(base, name='warm-reads-p4', dir='leagues/arenas/diag/warm-reads-p4', plies=[4],
         description="Warm-start reads at ply 4: W4 (ply-4 field) against W3 (ply-3 field), both from flat-p3's finals, which play beside them.",
         entries=finals('flat-p3-w4') + finals('flat-p3-w3') + finals('flat-p3'))
for spec in (a, b):
    out = os.path.join(REPO, 'leagues', 'specs', 'arenas', spec['name'] + '.json')
    json.dump(spec, open(out, 'w'), indent=1)
    print(out, len(spec['entries']), 'entries')
