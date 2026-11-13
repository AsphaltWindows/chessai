#!/usr/bin/env python3
"""Cold read of a ply-2 stage-1 experiment (11-11): every line at the given versions (numbers or 'final'), near argmax
(T 0.02), at the given plies, 75 round-robins, binary chessai-hybrid16 (plays flat, tree and path-context DAG models).
    python3 make_stage_read.py EXPERIMENT VERSIONS PLIES TAG=REGEX [TAG=REGEX ...]
VERSIONS and PLIES are comma lists, e.g. 300,final and 1,2,3. Each line gets the first TAG whose REGEX matches its line
name (re.search). Entries are named TAG<v>.L<k>@<version>, <v> the version number or F for the final, so arm_btg.py
groups them by TAG<v>. Writes leagues/specs/arenas/EXPERIMENT-stage.json (refuses to overwrite)."""
import csv, json, os, re, sys
REPO = '/home/iv/dev/chessai'
exp, versions, plies = sys.argv[1], sys.argv[2].split(','), [int(p) for p in sys.argv[3].split(',')]
rules = [a.split('=', 1) for a in sys.argv[4:]]
rows = list(csv.DictReader(open(os.path.join(REPO, 'leagues', 'experiments', exp, 'lines.csv'))))
entries = []
for want in versions:
    for r in rows:
        tag = next((t for t, rx in rules if re.search(rx, r['line'])), None)
        if tag is None:
            sys.exit(f"no rule matches {r['line']}")
        v = r['version'] if want == 'final' else want
        assert int(v) <= int(r['version']), (r['line'], v, r['version'])
        k = r['line'].rsplit('#', 1)[1].split('.')[0]
        vl = 'F' if want == 'final' else want
        entries.append({'version': f"{exp}:{r['line']}@{v}", 'name': f"{tag}{vl}.L{k}@{v}",
                        'label': f"{exp} {r['line']} version v{v}" + (' (final)' if want == 'final' else '')})
binary = 'models/from-scratch/bin/chessai-hybrid16'
spec = {'seed': 1, 'nice': 15, 'threads': 2, 'jobs': 2, 'binary': binary, 'owncal_binary': binary,
        'flags': ['--reports-last', '--sparse-fit', '--policy', 'log-odds-softmax', '--temperature', '0.02'],
        'references': ['random', 'random-1', 'random-2', 'random-3'],
        'stop': {'ci95': 0.0, 'chunk_rounds': 15, 'min_rounds': 75, 'max_rounds': 75},
        'name': f'{exp}-stage', 'dir': f'leagues/arenas/diag/{exp}-stage', 'plies': plies,
        'description': f"Stage-1 cold read of {exp} at versions {', '.join(versions)}, plies {sys.argv[3]}, near argmax "
                       f"(T 0.02), 75 round-robins (theory-2 6.34). Tags: " + ', '.join(f'{t}={rx}' for t, rx in rules),
        'entries': entries}
out = os.path.join(REPO, 'leagues', 'specs', 'arenas', spec['name'] + '.json')
assert not os.path.exists(out), out
json.dump(spec, open(out, 'w'), indent=1)
print('wrote', out, len(entries), 'entries:', ' '.join(e['name'] for e in entries))
