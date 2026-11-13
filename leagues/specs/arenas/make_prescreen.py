#!/usr/bin/env python3
"""Write the low-ply pre-screen (theory-2 6.23 item 5, 11-11 ~09:00): the champions of every archetype never rated
near argmax, beside flat LR and flat probit of the same vintage, at plies 1 and 2, T 0.02, chessai-hybrid13.
Three vintage blocks, each with its own flat references (contrasts by prescreen_btg.py):
  old.*  the champion run (cycles/champ, stopped 11-09 after cycle 75): champions by its cycle-75 table. Merge trees
         whose champions had collapsed to one leaf (T-L-Ne-merge-c v2571, T-L-NNs-merge-c v2645, T-L-NLs-merge-c,
         T-L-Nc-merge-c; ~105 KB files, a flat LR's size) are left out; T-L-LLs-merge-c (4 leaves) stands for split
         trees. FM was read cold at plies 2-3 (ply3-cold); T-L-Ne-share3-c v394 was read cold at ply 2
         (top-champions-1-cold) and links the two reads.
  ce.*   cluster-error (14-architecture stream at ply 2): per archetype the better of its two variants by its own
         ply-2 rating of the line's champion; its own F-L and F-P champions.
  ssp.*  single-split-probit's arms that cold-trees did not read (T-P-Lc, T-P-Le, T-P-Te), its F-P champion, and
         single-split's F-L champion (the sister LR field of the same batch)."""
import json, os
REPO = '/home/iv/dev/chessai'
R = 'cycles/runs'
old = [  # (lineage, kind, version, champion Elo at cycle 75, note)
    ('F-L-c', 'lr', 4827, 1833, 'flat LR reference'),
    ('F-P-tau0-c', 'probit', 4359, 1684, 'flat probit reference'),
    ('M-L4-N-c', 'nbgate', 338, 1846, 'NB-gated mixture of 4 LR experts'),
    ('M-L8-L-c', 'moe', 305, 1821, 'LR-gated mixture of 8 LR experts'),
    ('M-LP-L-hard-c', 'hybrid', 473, 1814, 'hard LR+probit mixture'),
    ('T-L-Nc-c', 'bdt', 1349, 1859, 'NB cluster tree'),
    ('D-L-Nc-c', 'bdt', 2043, 1853, 'NB cluster DAG'),
    ('T-L-LLs-merge-c', 'bdt', 2767, 1827, 'LR-router split tree (merge)'),
    ('T-L-Ne-share3-c', 'bdt', 394, 1886, 'error tree, leaf share 0.3 (read cold at ply 2 before)'),
    ('T-L-Ne-pre-c', 'bdt', 1435, 1836, 'error tree, pre'),
    ('D-L-Ne-skill-c', 'bdd', 284, 1812, 'error DAG, skill trigger'),
    ('T-L-Te-c', 'bdt', 200, 1811, 'TAN-router error tree'),
]
entries = []
for lin, kind, v, elo, note in old:
    entries.append({'file': f'{R}/{lin}/{kind}{v}.model', 'kind': kind, 'heldout': 'lr', 'name': f'old.{lin}@{v}',
                    'label': f'champion run {lin} v{v} (champion Elo {elo} at cycle 75): {note}'})
ce = [('F-L.loc', 317), ('F-P.loc', 236),
      ('T-L-Lc-merge+split-limit=1+split-threshold=1000000.loc', 338),
      ('T-L-Nc-merge+error=log+split-limit=1+split-threshold=1000000.loc', 210),
      ('T-L-Tc-merge+error=log+split-limit=1+split-threshold=1000000.loc', 240),
      ('T-P-Lc-merge+error=log+split-limit=1+split-threshold=1000000.loc', 209),
      ('T-P-Nc-merge+split-limit=1+split-threshold=1000000.loc', 385),
      ('T-P-Tc-merge+split-limit=1+split-threshold=1000000.loc', 246)]
for line, v in ce:
    short = line.split('-merge')[0].split('.loc')[0]
    entries.append({'version': f'cluster-error:{line}@{v}', 'name': f'ce.{short}@{v}',
                    'label': f'cluster-error {line} champion v{v}'})
ssp = [('single-split-probit', 'F-P.loc', 942, 'ssp.F-P'),
       ('single-split', 'F-L.loc', 150, 'ssp.F-L'),
       ('single-split-probit', 'T-P-Lc-merge+split-limit=1+split-threshold=1000000.loc', 669, 'ssp.T-P-Lc'),
       ('single-split-probit', 'T-P-Le+split-limit=1+split-threshold=1000000.loc', 806, 'ssp.T-P-Le'),
       ('single-split-probit', 'T-P-Te+split-limit=1+split-threshold=1000000.loc', 747, 'ssp.T-P-Te')]
for exp, line, v, name in ssp:
    entries.append({'version': f'{exp}:{line}@{v}', 'name': f'{name}@{v}', 'label': f'{exp} {line} champion v{v}'})
spec = {'name': 'prescreen-p12', 'dir': 'leagues/arenas/diag/prescreen-p12',
        'description': "Low-ply pre-screen (theory-2 6.23 item 5): archetype champions never rated near argmax against flat "
                       "LR and flat probit of their own vintage, plies 1 and 2, T 0.02.",
        'seed': 1, 'nice': 15, 'threads': 2, 'jobs': 2,
        'binary': 'models/from-scratch/bin/chessai-hybrid13', 'owncal_binary': 'models/from-scratch/bin/chessai-hybrid9',
        'flags': ['--reports-last', '--sparse-fit', '--policy', 'log-odds-softmax', '--temperature', '0.02'],
        'references': ['random', 'random-1', 'random-2', 'random-3'], 'plies': [1, 2],
        'stop': {'ci95': 0.0, 'chunk_rounds': 5, 'min_rounds': 30, 'max_rounds': 30},
        'entries': entries}
out = os.path.join(REPO, 'leagues', 'specs', 'arenas', spec['name'] + '.json')
assert not os.path.exists(out), out
json.dump(spec, open(out, 'w'), indent=1)
print(out, len(entries), 'entries')
