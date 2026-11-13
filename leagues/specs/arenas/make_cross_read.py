#!/usr/bin/env python3
"""Cold read across experiments, one arena per ply (11-11 ~22:15, for replay-p2 against probit-late-p2).
    python3 make_cross_read.py NAME VERSIONS PLIES TAG=EXPERIMENT:REGEX [TAG=EXPERIMENT:REGEX ...]
VERSIONS and PLIES are comma lists (versions are numbers or 'final', the experiment's own last version). Each rule's
lines (re.search on the line name in EXPERIMENT's lines.csv) enter at each version as TAG<v>.L<k>@<version>, <v> the
version number or F, so arm_btg.py and h2h_btg.py group them by TAG<v>. One arena per ply, NAME-p<ply>: the read's
verdicts are head-to-heads at the same ply (analyst 6.44), and in a mixed-ply arena the cross-ply games, which no
verdict uses, took ~60% of the CPU (base-probit-p2-stage: ply-1/2 vs ply-3 games). Settings as make_stage_read.py:
near argmax (T 0.02), 4 random references, 75 round-robins, chessai-hybrid16. Writes
leagues/specs/arenas/NAME-p<ply>.json for each ply; refuses to overwrite."""
import csv, json, os, re, sys
REPO = '/home/iv/dev/chessai'
name, versions, plies = sys.argv[1], sys.argv[2].split(','), [int(p) for p in sys.argv[3].split(',')]
rules = []
for a in sys.argv[4:]:
    tag, rest = a.split('=', 1)
    exp, rx = rest.split(':', 1)
    rules.append((tag, exp, rx))
entries = []
for want in versions:
    for tag, exp, rx in rules:
        rows = list(csv.DictReader(open(os.path.join(REPO, 'leagues', 'experiments', exp, 'lines.csv'))))
        hit = [r for r in rows if re.search(rx, r['line'])]
        if not hit:
            sys.exit(f'no line of {exp} matches {rx}')
        for r in hit:
            v = r['version'] if want == 'final' else want
            assert int(v) <= int(r['version']), (exp, r['line'], v, r['version'])
            k = r['line'].rsplit('#', 1)[1].split('.')[0]
            vl = 'F' if want == 'final' else want
            entries.append({'version': f"{exp}:{r['line']}@{v}", 'name': f"{tag}{vl}.L{k}@{v}",
                            'label': f"{exp} {r['line']} version v{v}" + (' (final)' if want == 'final' else '')})
names = [e['name'] for e in entries]
assert len(names) == len(set(names)), 'two entries share a name: give each experiment its own tags'
binary = 'models/from-scratch/bin/chessai-hybrid16'
for p in plies:
    spec = {'seed': 1, 'nice': 15, 'threads': 2, 'jobs': 2, 'binary': binary, 'owncal_binary': binary,
            'flags': ['--reports-last', '--sparse-fit', '--policy', 'log-odds-softmax', '--temperature', '0.02'],
            'references': ['random', 'random-1', 'random-2', 'random-3'],
            'stop': {'ci95': 0.0, 'chunk_rounds': 15, 'min_rounds': 75, 'max_rounds': 75},
            'name': f'{name}-p{p}', 'dir': f'leagues/arenas/diag/{name}-p{p}', 'plies': [p],
            'description': f"Cold read {name} at ply {p} only (one arena per ply), versions {', '.join(versions)}, near "
                           f"argmax (T 0.02), 75 round-robins. Tags: " + ', '.join(f'{t}={e}:{rx}' for t, e, rx in rules),
            'entries': entries}
    out = os.path.join(REPO, 'leagues', 'specs', 'arenas', spec['name'] + '.json')
    assert not os.path.exists(out), out
    json.dump(spec, open(out, 'w'), indent=1)
    print('wrote', out, len(entries), 'entries')
print(' '.join(names))
