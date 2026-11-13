#!/usr/bin/env python3
"""Cold read of one batch-5 experiment's arms (11-11, the six-model shortlist): every line's final version, near argmax
(T 0.02), chessai-hybrid13, plies 1 and 2, 75 round-robins.
    python3 make_arm_read.py EXPERIMENT TAG=REGEX [TAG=REGEX ...]
Each line gets the first TAG whose REGEX matches its line name (re.search); entries are named TAG.LINE#k@VERSION, so the
contrast script (arm_btg.py) groups them by TAG. Writes leagues/specs/arenas/EXPERIMENT-cold.json (refuses to overwrite)."""
import csv, json, os, re, sys
REPO = '/home/iv/dev/chessai'
exp, rules = sys.argv[1], [a.split('=', 1) for a in sys.argv[2:]]
rows = list(csv.DictReader(open(os.path.join(REPO, 'leagues', 'experiments', exp, 'lines.csv'))))
entries = []
for r in rows:
    tag = next((t for t, rx in rules if re.search(rx, r['line'])), None)
    if tag is None:
        sys.exit(f"no rule matches {r['line']}")
    k = r['line'].rsplit('#', 1)[1].split('.')[0]
    entries.append({'version': f"{exp}:{r['line']}@{r['version']}", 'name': f"{tag}.L{k}@{r['version']}",
                    'label': f"{exp} {r['line']} final version v{r['version']}"})
spec = {'seed': 1, 'nice': 15, 'threads': 2, 'jobs': 2,
        'binary': 'models/from-scratch/bin/chessai-hybrid13', 'owncal_binary': 'models/from-scratch/bin/chessai-hybrid9',
        'flags': ['--reports-last', '--sparse-fit', '--policy', 'log-odds-softmax', '--temperature', '0.02'],
        'references': ['random', 'random-1', 'random-2', 'random-3'],
        'stop': {'ci95': 0.0, 'chunk_rounds': 15, 'min_rounds': 75, 'max_rounds': 75},
        'name': f'{exp}-cold', 'dir': f'leagues/arenas/diag/{exp}-cold', 'plies': [1, 2],
        'description': f"Cold read of {exp}'s final versions at plies 1-2 near argmax (six-model shortlist, theory-2 6.24). Tags: "
                       + ', '.join(f'{t}={rx}' for t, rx in rules),
        'entries': entries}
out = os.path.join(REPO, 'leagues', 'specs', 'arenas', spec['name'] + '.json')
assert not os.path.exists(out), out
json.dump(spec, open(out, 'w'), indent=1)
print(out, len(entries), 'entries:', ', '.join(e['name'] for e in entries))
