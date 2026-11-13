#!/usr/bin/env python3
"""Write a near-argmax (T 0.02) arena spec of a finished experiment's final champions, at ply 1 and ply 2.
(11-11 ~00:50: every routing gain measured at the standard T 0.1 turned out to be the play-temperature confound,
RESULTS section 8, so experiments rated at T 0.1 get a cold read of their champions.)

    python3 leagues/specs/arenas/make_cold_read.py EXPERIMENT [ROUNDS] [--final] [--plies 2,3]
        -> leagues/specs/arenas/cold-<EXPERIMENT>.json (champions), or cold-final-<EXPERIMENT>.json with --final (every
        line's last version: same age for every line; champions were picked by the T 0.1 tournament)
"""
import csv, json, os, sys
REPO = '/home/iv/dev/chessai'
final = '--final' in sys.argv
argv = sys.argv[1:]
plies = [1, 2]
if '--plies' in argv:
    i = argv.index('--plies')
    plies = [int(x) for x in argv[i + 1].split(',')]
    del argv[i:i + 2]
args = [a for a in argv if a != '--final']
exp = args[0]
rounds = int(args[1]) if len(args) > 1 else 15
tag = 'cold-final-' if final else 'cold-'
rows = list(csv.DictReader(open(os.path.join(REPO, 'leagues', 'experiments', exp, 'lines.csv'))))
ents = []
for r in rows:
    v = r['version'] if final else r['champion']
    short = r['line'][:-4] if r['line'].endswith(('.loc', '.sp.', '.glo')) else r['line']
    for common in ('+split-limit=4', '+split-threshold=1000000', '+revise-on=fresh'):
        short = short.replace(common, '')
    ents.append({'version': f"{exp}:{r['line']}@{v}", 'name': f'{short}@{v}',
                 'label': f"{r['line']} {'final version' if final else 'champion'} v{v}"})
names = [e['name'] for e in ents]
assert len(set(names)) == len(names), names
spec = {'name': f'{tag}{exp}', 'dir': f'leagues/arenas/diag/{tag}{exp}',
        'description': f"Near-argmax (T 0.02) read of {exp}'s {'last versions' if final else 'final champions'} at plies {', '.join(map(str, plies))} (its own tables are at "
                       f"T 0.1, which RESULTS section 8 found confounded by each model's calibrated scale).",
        'seed': 1, 'nice': 15, 'threads': 2, 'jobs': 2, 'binary': 'models/from-scratch/bin/chessai-hybrid11',
        'flags': ['--reports-last', '--sparse-fit', '--policy', 'log-odds-softmax', '--temperature', '0.02'],
        'references': ['random', 'random-1', 'random-2', 'random-3'], 'plies': plies,
        'stop': {'ci95': 0.0, 'chunk_rounds': 5, 'min_rounds': rounds, 'max_rounds': rounds}, 'entries': ents}
out = os.path.join(REPO, 'leagues', 'specs', 'arenas', f'{tag}{exp}.json')
json.dump(spec, open(out, 'w'), indent=1)
print(out); [print(' ', e['name'], '|', e['version']) for e in ents]
