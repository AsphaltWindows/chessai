#!/usr/bin/env python3
"""The pre-screen's champion-run block alone (11-11 ~08:50): prescreen-p12's 12 old.* entries (same models, settings
and own calibrations; cached), so their ratings come from same-vintage games only. In prescreen-p12 the old trees
and mixtures rated 55-93 over the old flat LR at ply 2 in the full field, but direct head-to-heads were mostly level
(60 games a pairing), and the chain through old-growth-cold and ply3-cold put T-L-Ne-share3-c only ~18 above it:
the full field's weak new-vintage entries may be shaping the fit."""
import json, os
REPO = '/home/iv/dev/chessai'
src = json.load(open(os.path.join(REPO, 'leagues/specs/arenas/prescreen-p12.json')))
spec = dict(src, name='prescreen-old-p12', dir='leagues/arenas/diag/prescreen-old-p12',
            description="The champion run's archetype champions and its flat LR and flat probit alone, plies 1-2, T 0.02 "
                        "(prescreen-p12's old.* block without the new-vintage entries).",
            entries=[e for e in src['entries'] if e['name'].startswith('old.')])
out = os.path.join(REPO, 'leagues', 'specs', 'arenas', spec['name'] + '.json')
assert not os.path.exists(out), out
json.dump(spec, open(out, 'w'), indent=1)
print(out, len(spec['entries']), 'entries')
