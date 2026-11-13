#!/usr/bin/env python3
"""Write the training-temperature read (theory-2 6.23 item 1, 11-11 ~08:45): the final versions of train-t002-p1 (F-L x2
whose own games were at T 0.02) and train-t01-p1 (its twin at T 0.1), at plies 1 and 2, near argmax (T 0.02),
chessai-hybrid13, 150 round-robins (four entries: the arm contrast needs many games; ply 1-2 games are cheap)."""
import csv, json, os
REPO = '/home/iv/dev/chessai'
tags = {'train-t002-p1': 't002', 'train-t01-p1': 't01'}
def finals(exp):
    rows = list(csv.DictReader(open(os.path.join(REPO, 'leagues', 'experiments', exp, 'lines.csv'))))
    return [{'version': f"{exp}:{r['line']}@{r['version']}", 'name': f"{tags[exp]}.{r['line'][:-4]}@{r['version']}",
             'label': f"{exp} {r['line']} final version v{r['version']}"} for r in rows]
spec = {'seed': 1, 'nice': 15, 'threads': 2, 'jobs': 2,
        'binary': 'models/from-scratch/bin/chessai-hybrid13', 'owncal_binary': 'models/from-scratch/bin/chessai-hybrid9',
        'flags': ['--reports-last', '--sparse-fit', '--policy', 'log-odds-softmax', '--temperature', '0.02'],
        'references': ['random', 'random-1', 'random-2', 'random-3'],
        'stop': {'ci95': 0.0, 'chunk_rounds': 10, 'min_rounds': 150, 'max_rounds': 150},
        'name': 'train-temp-p1', 'dir': 'leagues/arenas/diag/train-temp-p1', 'plies': [1, 2],
        'description': "Training-game temperature (theory-2 6.23 item 1): F-L trained on its field's T 0.02 games against its twin trained on T 0.1 games, final versions, read at plies 1-2 near argmax.",
        'entries': finals('train-t002-p1') + finals('train-t01-p1')}
out = os.path.join(REPO, 'leagues', 'specs', 'arenas', spec['name'] + '.json')
assert not os.path.exists(out)
json.dump(spec, open(out, 'w'), indent=1)
print(out, len(spec['entries']), 'entries')
