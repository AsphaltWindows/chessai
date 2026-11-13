#!/usr/bin/env python3
"""diversity-p2c's read (analyst 6.57 (6) and 6.58 (b); written 2026-11-12 ~02:55). Run after the experiment is done.
    python3 make_p2c_read.py
Ply 2: p2c's F-P (cfp) at v100, v150, v200, v300 and the final (cfpF) and its tree (ctr) at v150, v300 and the final;
base-probit-p2's F-P (bfp) at v300 and v450; probit-late-p2's F-P (pfp) at v300 and v399; diversity-p2's F-P (dfp)
at v300 and v314 (its final); the frozen references themselves (rFP: base-probit-p2 F-P v599, rTNc: its
T-P-Nc-merge-fresh v599); and, per reference kind, each line's current version in the cycle the window first cut that
kind (cutFP, cutTNc; from driver.log and history.csv), for the cold test of a cut reference against the version that
cut it. Ply 1: the finals against bfp v450, pfp v399, dfp v314. Settings as make_cross_read.py: near argmax (T 0.02),
4 random references, 75 round-robins, chessai-hybrid16. Writes leagues/specs/arenas/diversity-p2c-read-p<ply>.json
(refuses to overwrite) and prints the cut table."""
import csv, json, os, re, sys
REPO = '/home/iv/dev/chessai'
EXP = 'diversity-p2c'
X = os.path.join(REPO, 'leagues', 'experiments')


def rows(exp):
    return list(csv.DictReader(open(os.path.join(X, exp, 'lines.csv'))))


def pick(exp, rx):
    hit = [r for r in rows(exp) if re.search(rx, r['line'])]
    assert len(hit) == 2, (exp, rx, [r['line'] for r in hit])
    return hit


V = min(int(r['version']) for r in rows(EXP))
v300 = min(300, V)
groups = []  # (tag, exp, rx, version)
for v in (100, 150, 200, v300):
    groups.append((f'cfp{v}', EXP, r'^F-P', v))
groups.append(('cfpF', EXP, r'^F-P', V))
for v in (150, v300):
    groups.append((f'ctr{v}', EXP, r'^T-P-Ne', v))
groups.append(('ctrF', EXP, r'^T-P-Ne', V))
groups += [('bfp300', 'base-probit-p2', r'^F-P', 300), ('bfp450', 'base-probit-p2', r'^F-P', 450),
           ('pfp300', 'probit-late-p2', r'^F-P', 300), ('pfp399', 'probit-late-p2', r'^F-P', 399),
           ('dfp300', 'diversity-p2', r'^F-P', 300), ('dfp314', 'diversity-p2', r'^F-P', 314),
           ('rFP', 'base-probit-p2', r'^F-P', 599), ('rTNc', 'base-probit-p2', r'^T-P-Nc-merge', 599)]


def entries_of(tag, exp, rx, v):
    out = []
    for r in pick(exp, rx):
        assert v <= int(r['version']), (exp, r['line'], v, r['version'])
        k = r['line'].rsplit('#', 1)[1].split('.')[0]
        out.append({'version': f"{exp}:{r['line']}@{v}", 'name': f'{tag}.L{k}@{v}',
                    'label': f"{exp} {r['line']} version v{v}"})
    return out


# the window's first cut of each reference kind per line, and the line's current version in that cycle
cut_rx = re.compile(r'cycle (\d+): reference window \(margin [0-9.]+\): (.*)$')
first = {}  # (line, kind) -> cycle
for ln in open(os.path.join(X, EXP, 'driver.log')):
    m = cut_rx.search(ln)
    if not m:
        continue
    c = int(m.group(1))
    for part in m.group(2).split('; '):
        pm = re.match(r'(\S+) \(floor ([-0-9.]+)\) (\d+) games cut, (.*) below$', part)
        if not pm:
            continue
        for ref in re.findall(r'(x\w+)@', pm.group(4)):
            kind = 'FP' if ref.startswith('xFP') else 'TNc'
            first.setdefault((pm.group(1), kind), c)
cur = {}
for r in csv.DictReader(open(os.path.join(X, EXP, 'history.csv'))):
    if r['role'] == 'current':
        cur[(int(r['cycle']), r['line'])] = int(r['version'])
print('first cuts (line, reference kind: cycle, current version):')
cutents = {'FP': [], 'TNc': []}
for (line, kind), c in sorted(first.items(), key=lambda x: x[1]):
    v = cur.get((c, line))
    print(f'  {line[:40]:40s} {kind:3s} cycle {c} version {v}')
    if v is not None:
        cutents[kind].append((line, v))
ply2 = []
for g in groups:
    ply2 += entries_of(*g)
for kind, lst in cutents.items():
    for i, (line, v) in enumerate(lst, 1):
        ply2.append({'version': f'{EXP}:{line}@{v}', 'name': f'cut{kind}.L{i}@{v}',
                     'label': f'{EXP} {line} version v{v} (its current version when the window first cut the {kind} references)'})
ply1 = []
for g in groups:
    if g[0] in ('cfpF', 'ctrF', 'bfp450', 'pfp399', 'dfp314'):
        ply1 += entries_of(*g)
binary = 'models/from-scratch/bin/chessai-hybrid16'
for p, ents in ((2, ply2), (1, ply1)):
    names = [e['name'] for e in ents]
    assert len(names) == len(set(names)), 'duplicate names'
    spec = {'seed': 1, 'nice': 15, 'threads': 2, 'jobs': 2, 'binary': binary, 'owncal_binary': binary,
            'flags': ['--reports-last', '--sparse-fit', '--policy', 'log-odds-softmax', '--temperature', '0.02'],
            'references': ['random', 'random-1', 'random-2', 'random-3'],
            'stop': {'ci95': 0.0, 'chunk_rounds': 15, 'min_rounds': 75, 'max_rounds': 75},
            'name': f'diversity-p2c-read-p{p}', 'dir': f'leagues/arenas/diag/diversity-p2c-read-p{p}', 'plies': [p],
            'description': (f"diversity-p2c's read at ply {p} (analyst 6.57/6.58), final version v{V}: p2c F-P (cfp) "
                            "and tree (ctr) against base-probit-p2 F-P (bfp), probit-late-p2 F-P (pfp), diversity-p2 F-P "
                            "(dfp); at ply 2 also the acceleration ladder and the cold test of each cut reference kind "
                            "(rFP, rTNc) against the line versions that cut it (cutFP, cutTNc). Near argmax (T 0.02), 75 "
                            "round-robins."),
            'entries': ents}
    out = os.path.join(REPO, 'leagues', 'specs', 'arenas', spec['name'] + '.json')
    assert not os.path.exists(out), out
    json.dump(spec, open(out, 'w'), indent=1)
    print('wrote', out, len(ents), 'entries')
print('FINAL', V)
