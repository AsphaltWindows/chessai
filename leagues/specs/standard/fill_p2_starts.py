#!/usr/bin/env python3
"""Fill the ply-2 rung's warm drafts with their start versions (training standard v1, models/STANDARD.md; written
2026-11-12 by the ply-2 prep fork). Launches nothing.

    python3 fill_p2_starts.py s1|s2|all [--check] [--src DIR] [--driver PATH] [--out-dir DIR]

For s1 (s2): reads leagues/experiments/std-v1-p1-s1 (-s2), which must be done (state.json status "done": its
plateau stop or its cap); its final version V is the lines' common version in lines.csv (a done run's last cycle
trains nothing, so V is its last cycle - 1). Checks that every line's model at V and, for trees, its training state
at V are there (a run keeps its current version's state only; a done run keeps its final's), that every model entry
of std-v1-p2-s1.DRAFT.json starts from that experiment with "version": null, and that the filled spec passes the
driver's own create checks (make_config: the start lines, kinds, recipes, train arguments, md5s and the train
binary's flags) without creating anything. Then writes leagues/specs/standard/std-v1-p2-s1.json (refuses to
overwrite) with V in every start and a FILLED note at the head of the description. --check: everything but the write.
Create and launch stay the coordinator's (pending decision 2).
Tests: --src DIR reads that directory (a copy with config.json, lines.csv, state.json and runs/) instead and writes
its absolute path as start.experiment; --driver PATH checks with another driver (the work copy); --out-dir DIR writes
there."""
import csv, importlib.util, json, os, sys, time

STD = os.path.dirname(os.path.abspath(__file__))
LEAGUES = os.path.dirname(os.path.dirname(STD))
EXPS = os.path.join(LEAGUES, 'experiments')
SRC = {'s1': 'std-v1-p1-s1', 's2': 'std-v1-p1-s2'}
TREES = ('bdt', 'bdd', 'ht')


OPTS = {}


def driver():
    s = importlib.util.spec_from_file_location('leagues_fill', OPTS.get('driver') or os.path.join(LEAGUES, 'leagues.py'))
    m = importlib.util.module_from_spec(s)
    s.loader.exec_module(m)
    return m


def fill(run, check):
    exp = SRC[run]
    src = os.path.abspath(OPTS['src']) if OPTS.get('src') else os.path.join(EXPS, exp)
    st = json.load(open(os.path.join(src, 'state.json')))
    if st.get('status') != 'done':
        sys.exit(f'{exp} is {st.get("status")} at cycle {st.get("cycle")}: the ply-2 warm start needs its final '
                 f'(state "done")')
    rows = list(csv.DictReader(open(os.path.join(src, 'lines.csv'))))
    vs = {int(r['version']) for r in rows}
    if len(vs) != 1:
        sys.exit(f'{exp}: the lines end at versions {sorted(vs)}, not one final')
    v = vs.pop()
    for r in rows:
        for ext in ('model',) + (('state',) if r['kind'] in TREES else ()):
            p = os.path.join(src, 'runs', r['line'], f"{r['kind']}{v}.{ext}")
            if not os.path.isfile(p):
                sys.exit(f'{exp}: no {p}')
    draft_path = os.path.join(STD, f'std-v1-p2-{run}.DRAFT.json')
    spec = json.load(open(draft_path))
    for m in spec['models']:
        if m.get('start') != {'experiment': exp, 'version': None}:
            sys.exit(f'{draft_path}: {m["model"]} starts from {m.get("start")}, not {exp} at null')
        m['start'] = {'experiment': src if OPTS.get('src') else exp, 'version': v}
    stamp = time.strftime('%Y-%m-%d %H:%M')
    spec['description'] = (f'FILLED {stamp} by fill_p2_starts.py: every line starts from {exp} v{v}, its final '
                           f'(done at cycle {st.get("cycle")}). ' + spec['description'])
    LG = driver()
    try:
        _, cfg = LG.make_config(json.loads(json.dumps(spec)))
    except SystemExit as e:
        sys.exit(f'{run}: the driver refuses the filled spec: {e}')
    starts = [(lc['line'], lc['start']['version'], lc['start'].get('forget'), lc['start'].get('reset', False),
               os.path.basename(lc['train_binary'])) for lc in cfg['lines']]
    out = os.path.join(OPTS.get('out-dir') or STD, f'std-v1-p2-{run}.json')
    print(f'{run}: {exp} done at cycle {st.get("cycle")}, final v{v}; the driver accepts the filled spec:')
    for s in starts:
        print(f'  {s[0][:70]:70s} v{s[1]} forget {s[2]} reset {s[3]} {s[4]}')
    if check:
        print(f'  --check: {out} not written')
        return
    if os.path.exists(out):
        sys.exit(f'{out} exists')
    with open(out + '.tmp', 'w') as f:
        json.dump(spec, f, indent=1)
        f.write('\n')
    os.rename(out + '.tmp', out)
    print(f'  wrote {out}')


if __name__ == '__main__':
    argv = sys.argv[1:]
    for k in ('--src', '--driver', '--out-dir'):
        if k in argv:
            i = argv.index(k)
            OPTS[k[2:]] = argv[i + 1]
            del argv[i:i + 2]
    args = [a for a in argv if a != '--check']
    if len(args) != 1 or args[0] not in ('s1', 's2', 'all'):
        sys.exit(__doc__)
    for run in (('s1', 's2') if args[0] == 'all' else (args[0],)):
        fill(run, '--check' in sys.argv)
