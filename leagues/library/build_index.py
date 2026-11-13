#!/usr/bin/env python3
"""Builds the model library's index (leagues/library/index.csv) and snapshots its models (11-12 ~10:00; the user:
"We should have a library of fully-plateaud models to choose from for future tests in arenas, so that we don't have
to retrain them"). See LIBRARY.md.

    python3 leagues/library/build_index.py [--extra EXP:LINE@V ...] [--extra-file FILE]

One row per line of every experiment under leagues/experiments that ran, at its final version, and one row per
global-league line for its best champion (this morning's all-champions arena) and its last champion. Each row's model
file is snapshotted with `cp --reflink=always -n` into leagues/library/models/<experiment>/runs/<line>/ (the
experiment's own layout, with its config.json and lines.csv beside it, so arena.py can read an entry from the
snapshot as "<library>/models/<experiment>:<line>@<v>"), and the copy's md5 checked against the source. --extra
snapshots more versions (Part 2 of the cohort read uses v279/v299 and finals). Never deletes anything; reruns add
what is missing and re-check the md5s (an existing snapshot that differs from its source is an error)."""
import argparse, csv, glob, hashlib, json, os, re, statistics, subprocess, sys

REPO = '/home/iv/dev/chessai'
LEAGUES = f'{REPO}/leagues'
EXP = f'{LEAGUES}/experiments'
LIB = f'{LEAGUES}/library'
SPECS = f'{LEAGUES}/specs'
GLOBAL = f'{LEAGUES}/global'
CHAMP_ARENA = f'{LEAGUES}/arenas/diag/global-all-champions-p1'
LINEAGE_SCAN = 20  # versions 1..20 of every line are compared across runs (md5) to find shared early lineage

FIELDS = ['id', 'experiment', 'line', 'version', 'version_role', 'model', 'archetype', 'kind', 'line_status',
          'train_ply', 'end_reason', 'plateaued', 'end_cycle', 'floor', 'cap', 'plateau_window', 'last_champion',
          'field', 'field_class', 'setup_group', 'seed', 'games_per_cycle', 'rounds_per_cycle',
          'train_games_per_cycle', 'self_games', 'ply1_entries', 'random_players', 'frozen_references',
          'reference_window', 'replay', 'warm_start', 'lineage', 'train_binary', 'train_args', 'train_overrides',
          'tournament_binary', 'spec', 'source_model', 'md5', 'snapshot', 'loads_hybrid17', 'note']


def md5(p):
    h = hashlib.md5()
    with open(p, 'rb') as f:
        for b in iter(lambda: f.read(1 << 20), b''):
            h.update(b)
    return h.hexdigest()


def read_json(p):
    try:
        return json.load(open(p))
    except (OSError, ValueError):
        return None


def spec_index():
    out = {}
    for p in glob.glob(f'{SPECS}/**/*.json', recursive=True):
        if '/arenas/' in p:
            continue
        s = read_json(p)
        if isinstance(s, dict) and s.get('kind') == 'experiment' and s.get('name'):
            out.setdefault(s['name'], []).append(p)
    return out


def end_state(E):
    """(reason, cycle): plateau / cap / manual-stop / not-run / unknown, from driver.log and state.json; extending
    (2026-11-12) while a run that reached its cap runs on after `leagues.py extend` (its log's "extend: cap C -> N"
    line is the latest; main indexes its final at the old cap until it ends again)."""
    st = read_json(f'{EXP}/{E}/state.json') or {}
    reason, cyc = 'unknown', st.get('cycle')
    try:
        lines = open(f'{EXP}/{E}/driver.log', errors='replace').read().splitlines()
    except OSError:
        lines = []
    for s in reversed(lines):
        m = re.search(r'cycle (\d+) done \(experiment finished(?:: (\w+))?\)', s)
        if m:
            reason = {'plateau': 'plateau', None: 'cap'}.get(m.group(2), m.group(2))
            cyc = int(m.group(1))
            break
        if 'STOP found' in s:
            reason = 'manual-stop'
            break
        if ' extend: cap ' in s:  # leagues.py extend: not finished again yet
            reason = 'extending'
            break
    if st.get('status') == 'created':
        reason = 'not-run'
    return reason, cyc


def strip_rep(model):
    return re.sub(r'#\d+$', '', model)


def field_of(cfg):
    counts = {}
    lines = [l for l in cfg['lines'] if not l.get('observer')]  # observers train on the field's games, never play
    for l in lines:
        k = strip_rep(l['model'])
        counts[k] = counts.get(k, 0) + 1
    arche = {l.get('archetype') for l in lines}
    field = ', '.join(f'{k} x{n}' if n > 1 else k for k, n in counts.items())
    return field, ('pure' if len(arche) == 1 else 'mixed')


def setup_signature(E, cfg, spec):
    """What makes two runs the same setup: the spec without its name, description, seed and train seed (and the
    tournament binary: those are byte-identical speedups); runs without a spec use their config the same way."""
    if spec:
        s = json.loads(json.dumps(spec))
        for k in ('name', 'description', 'seed'):
            s.pop(k, None)
        if isinstance(s.get('tournament'), dict):  # the binary only (byte-identical speedups); flags count
            s['tournament'] = {k: v for k, v in s['tournament'].items() if k != 'binary'}
        if isinstance(s.get('train'), dict):
            s['train'] = {k: v for k, v in s['train'].items() if k != 'seed'}
        return 'spec:' + json.dumps(s, sort_keys=True)
    c = {k: v for k, v in cfg.items() if k not in ('name', 'created', 'seed', 'prices', 'root')}
    if isinstance(c.get('tournament'), dict):
        c['tournament'] = {k: v for k, v in c['tournament'].items() if k != 'binary'}
    if isinstance(c.get('train'), dict):
        c['train'] = {k: v for k, v in c['train'].items() if k != 'seed'}
    c['lines'] = [{k: (re.sub(r"'--seed', '\d+'", '', json.dumps(v)) if k == 'selfplay_args' else v)
                   for k, v in l.items() if k not in ('entered_cycle',)} for l in cfg['lines']]
    return 'cfg:' + json.dumps(c, sort_keys=True)


def train_ply_of(E, cfg, line):
    if cfg.get('entry_ply') is not None:
        return str(cfg['entry_ply'])
    sp = line.get('selfplay_args') or []
    if line['line'].endswith('.sp'):
        d = sp[sp.index('--depth') + 1] if '--depth' in sp else 'default'
        return f'self-play depth {d}'
    plies = set()
    try:
        for s in open(f'{EXP}/{E}/tour/players.txt'):
            if s.startswith('#') or s.startswith('random'):
                continue
            parts = s.split()
            if len(parts) >= 2:
                plies.add(parts[1])
    except OSError:
        pass
    if len(plies) == 1:
        return plies.pop()
    return 'mixed ' + ','.join(sorted(plies)) if plies else '?'


def train_games(E):
    """{line: median train_games over the line's last 100 cycles} from cycles.csv."""
    per = {}
    try:
        for r in csv.DictReader(open(f'{EXP}/{E}/cycles.csv')):
            try:
                per.setdefault(r['line'], []).append(int(float(r['train_games'])))
            except (ValueError, KeyError, TypeError):
                pass
    except OSError:
        pass
    return {k: int(statistics.median(v[-100:])) for k, v in per.items() if v}


def snapshot(src, E, line, fname, extra_files=()):
    """Reflinks src into LIB/models/E/runs/line/fname (and E's config.json, lines.csv); returns the path."""
    d = f'{LIB}/models/{E}'
    os.makedirs(f'{d}/runs/{line}', exist_ok=True)
    for f in ('config.json', 'lines.csv'):
        s = f'{GLOBAL}/{f}' if E == 'global' else f'{EXP}/{E}/{f}'
        if os.path.exists(s) and not os.path.exists(f'{d}/{f}'):
            subprocess.run(['cp', '--reflink=always', '-n', s, f'{d}/{f}'], check=True)
    dst = f'{d}/runs/{line}/{fname}'
    if not os.path.exists(dst):
        subprocess.run(['cp', '--reflink=always', '-n', src, dst], check=True)
    a, b = md5(src), md5(dst)
    if a != b:
        sys.exit(f'snapshot {dst} differs from {src}')
    return dst, a


def lineage_scan(runs):
    """{(E, line): {other E: last shared version}} from md5s of versions 1..LINEAGE_SCAN."""
    seen = {}
    for E, cfg in runs.items():
        for l in cfg['lines']:
            for v in range(1, LINEAGE_SCAN + 1):
                p = f"{EXP}/{E}/runs/{l['line']}/{l['kind']}{v}.model"
                if os.path.exists(p):
                    seen.setdefault(md5(p), []).append((E, l['line'], v))
    shared = {}
    for hits in seen.values():
        exps = {h[0] for h in hits}
        if len(exps) < 2:
            continue
        for E, line, v in hits:
            for F, line2, v2 in hits:
                if F != E:
                    k = (E, line)
                    shared.setdefault(k, {})
                    shared[k][F] = max(shared[k].get(F, 0), v)
    return shared


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--extra', nargs='*', default=[], help='EXP:LINE@V versions to snapshot too')
    ap.add_argument('--extra-file', help='file with one EXP:LINE@V per line')
    args = ap.parse_args()
    specs = spec_index()
    runs, rows = {}, []
    for cf in sorted(glob.glob(f'{EXP}/*/config.json')):
        E = cf.split('/')[-2]
        cfg = read_json(cf)
        if not cfg or not os.path.exists(f'{EXP}/{E}/lines.csv'):
            continue
        runs[E] = cfg
    shared = lineage_scan(runs)
    groups = {}
    meta = {}
    for E, cfg in runs.items():
        sp = specs.get(E, [])
        spec = read_json(sp[0]) if sp else None
        sig = setup_signature(E, cfg, spec)
        groups.setdefault(sig, []).append(E)
        meta[E] = (sp[0] if sp else '', spec, sig)
    group_name = {}
    for sig, es in groups.items():
        for E in es:
            group_name[E] = sorted(es)[0] + (f' (+{len(es) - 1} seeds: ' + ', '.join(sorted(es)) + ')'
                                             if len(es) > 1 else '')
    skipped, xsnaps = [], []
    for E, cfg in runs.items():
        reason, cyc = end_state(E)
        if reason == 'not-run':
            skipped.append(E)
            continue
        st = read_json(f'{EXP}/{E}/state.json') or {}
        hist = cfg.get('cap_history') or []  # leagues.py extend: the run's caps before this one
        xh, xst = None, None
        if reason == 'extending' and hist:  # the final at the old cap stands until the run ends again
            xh = hist[-1]
            cyc = xh['old']
            xst = read_json(f"{EXP}/{E}/{xh['backup']}/state.json") or {}
        lrows = {r['line']: r for r in csv.DictReader(open(f'{EXP}/{E}/lines.csv'))}
        tg = train_games(E)
        field, fclass = field_of(cfg)
        spath, spec, _ = meta[E]
        pl = cfg.get('plateau') or {}
        refs = cfg.get('references') or []
        frozen = cfg.get('frozen_references') or []
        warm = []
        for m in (spec or {}).get('models', []):
            if isinstance(m, dict) and m.get('start'):
                warm.append(f"{m.get('model')} from {m['start'].get('experiment')}@{m['start'].get('version')}")
        code = re.sub(r'[^A-Za-z0-9]', '', E)
        for i, l in enumerate(cfg['lines']):
            line = l['line']
            lr = lrows.get(line, {})
            v = int(lr.get('version') or st.get('lines', {}).get(line, {}).get('version') or 0)
            if xh is not None and line in xh.get('finals', {}):
                v = int(xh['finals'][line])
            src = f"{EXP}/{E}/runs/{line}/{l['kind']}{v}.model"
            if not os.path.exists(src):
                print(f'missing {src}', file=sys.stderr)
                continue
            snap, h = snapshot(src, E, line, os.path.basename(src))
            for x in hist:  # leagues.py extend: each final at an old cap keeps its snapshot
                xv = x.get('finals', {}).get(line)
                xsrc = f"{EXP}/{E}/runs/{line}/{l['kind']}{xv}.model"
                if xv is not None and xv != v and os.path.exists(xsrc):
                    xsnap, xmd5 = snapshot(xsrc, E, line, os.path.basename(xsrc))
                    xsnaps.append(dict(experiment=E, line=line, version=int(xv),
                                       snapshot=os.path.relpath(xsnap, REPO), md5=xmd5))
            crowned = (xst if xst is not None else st).get('lines', {}).get(line, {}).get('crowned') or []
            if xh is not None:
                note = (f"extending: reached its cap {xh['old']} ({xh['date']}), raised to {xh['new']} (leagues.py "
                        f"extend); this row is its final at the old cap until the run ends again")
            else:
                note = '; '.join(f"extended: cap {h['old']} -> {h['new']} on {h['date']} (the final at cap {h['old']}: "
                                 f"v{h['finals'].get(line, '?')}, its snapshot kept)" for h in hist)
            lin = []
            for F, upto in sorted(shared.get((E, line), {}).items()):
                lin.append(f'versions to v{upto} identical with {F}')
            for fr in frozen:
                lin.append(f"frozen reference {fr.get('version')}")
            rows.append(dict(
                id=f'{code}_{"abcdefghijklmnopqrstuvwxyz"[i]}', experiment=E, line=line, version=v,
                version_role='final', model=l['model'], archetype=l.get('archetype', ''), kind=l['kind'],
                line_status=lr.get('status', ''), train_ply=train_ply_of(E, cfg, l), end_reason=reason,
                plateaued='yes' if reason == 'plateau' else 'no', end_cycle=cyc, floor=pl.get('min_cycles', ''),
                cap=xh['old'] if xh is not None else cfg.get('cycles', ''), plateau_window=pl.get('cycles', ''),
                last_champion=crowned[-1] if crowned else '', field=field, field_class=fclass,
                setup_group=group_name[E], seed=cfg.get('seed', ''),
                games_per_cycle=cfg.get('games_per_cycle') or '', rounds_per_cycle=cfg.get('rounds_per_cycle') or '',
                train_games_per_cycle=tg.get(line, ''), self_games=cfg.get('self_games', ''),
                ply1_entries=cfg.get('ply1_entries', ''), random_players=len([r for r in refs if 'random' in r]),
                frozen_references='; '.join(fr.get('version', '') for fr in frozen),
                reference_window=json.dumps(cfg['reference_window']) if cfg.get('reference_window') else '',
                replay=json.dumps(cfg['replay']) if cfg.get('replay') else '', warm_start='; '.join(warm),
                lineage='; '.join(lin), train_binary=os.path.basename(l.get('train_binary') or ''),
                train_args=' '.join(l.get('train_args') or []),
                train_overrides=json.dumps(l.get('train_overrides') or []),
                tournament_binary=os.path.basename(cfg['tournament']['binary']), spec=os.path.relpath(spath, REPO)
                if spath else '', source_model=os.path.relpath(src, REPO), md5=h,
                snapshot=os.path.relpath(snap, REPO), loads_hybrid17='', note=note))
    # The global league: each line's best champion in the all-champions arena and its last champion.
    gcfg = read_json(f'{GLOBAL}/config.json')
    if gcfg:
        acfg = read_json(f'{CHAMP_ARENA}/config.json') or {'entries': []}
        src_of = {e['name']: e['source'] for e in acfg['entries']}
        best, last = {}, {}
        for r in csv.DictReader(open(f'{CHAMP_ARENA}/tour/ratings.csv')):
            if r['player'].startswith('random'):
                continue
            s = src_of[r['player'].rsplit('-', 1)[0]]
            line, v = s.split(':', 1)[1].rsplit('@', 1)
            v, elo = int(v), float(r['elo'])
            if line not in best or elo > best[line][1]:
                best[line] = (v, elo, r['ci95'])
            if line not in last or v > last[line][0]:
                last[line] = (v, elo, r['ci95'])
        field, fclass = field_of(gcfg)
        for i, l in enumerate(gcfg['lines']):
            line = l['line']
            if line not in best:
                continue
            todo = [(best[line], 'best champion')]
            if last[line][0] != best[line][0]:
                todo.append((last[line], 'last champion'))
            else:
                todo = [(best[line], 'best and last champion')]
            for (v, elo, ci), role in todo:
                src = f"{GLOBAL}/runs/{line}/{l['kind']}{v}.model"
                snap, h = snapshot(src, 'global', line, os.path.basename(src))
                rows.append(dict(
                    id=f'global_{"abcdefghijklmnopqrstuvwxyz"[i]}', experiment='global', line=line, version=v,
                    version_role=role, model=l['model'], archetype=l.get('archetype', ''), kind=l['kind'],
                    line_status='champion', train_ply='1', end_reason='killed (user, cycle 5918)', plateaued='no',
                    end_cycle=5918, floor='', cap='', plateau_window='', last_champion=last[line][0], field=field,
                    field_class=fclass, setup_group='global', seed=gcfg.get('seed', ''), games_per_cycle='',
                    rounds_per_cycle=gcfg.get('rounds_per_cycle', ''), train_games_per_cycle='',
                    self_games=gcfg.get('self_games', ''), ply1_entries=gcfg.get('ply1_entries', ''),
                    random_players=len([r for r in (gcfg.get('references') or []) if 'random' in r]),
                    frozen_references='', reference_window='', replay='', warm_start='',
                    lineage='co-evolved global field (champion gating, no plateau rule)',
                    train_binary=os.path.basename(l.get('train_binary') or ''),
                    train_args=' '.join(l.get('train_args') or []),
                    train_overrides=json.dumps(l.get('train_overrides') or []),
                    tournament_binary=os.path.basename(gcfg['tournament']['binary']), spec='',
                    source_model=os.path.relpath(src, REPO), md5=h, snapshot=os.path.relpath(snap, REPO),
                    loads_hybrid17='', note=f'all-champions arena (ply 1, T 0.02): {elo:.0f} +-{ci}'))
    # Extra versions (snapshots only; listed in snapshots.csv with every snapshot).
    extras = list(args.extra)
    if args.extra_file:
        extras += [s.strip() for s in open(args.extra_file) if s.strip() and not s.startswith('#')]
    snaps = [dict(experiment=r['experiment'], line=r['line'], version=r['version'], snapshot=r['snapshot'],
                  md5=r['md5']) for r in rows] + xsnaps
    for x in extras:
        E, rest = x.split(':', 1)
        line, v = rest.rsplit('@', 1)
        cfg = runs[E] if E != 'global' else gcfg
        kind = next(l['kind'] for l in cfg['lines'] if l['line'] == line)
        base = f'{EXP}/{E}' if E != 'global' else GLOBAL
        src = f'{base}/runs/{line}/{kind}{v}.model'
        snap, h = snapshot(src, E, line, os.path.basename(src))
        snaps.append(dict(experiment=E, line=line, version=int(v), snapshot=os.path.relpath(snap, REPO), md5=h))
    # Keep any earlier smoke-check results.
    old = {}
    if os.path.exists(f'{LIB}/index.csv'):
        for r in csv.DictReader(open(f'{LIB}/index.csv')):
            old[(r['experiment'], r['line'], r['version'])] = r.get('loads_hybrid17', '')
    for r in rows:
        r['loads_hybrid17'] = old.get((r['experiment'], r['line'], str(r['version'])), '')
    with open(f'{LIB}/index.csv', 'w', newline='') as f:
        w = csv.DictWriter(f, FIELDS)
        w.writeheader()
        w.writerows(rows)
    seen, uniq = set(), []
    for s in snaps:
        k = (s['experiment'], s['line'], int(s['version']))
        if k not in seen:
            seen.add(k)
            uniq.append(s)
    old_snaps = []
    if os.path.exists(f'{LIB}/snapshots.csv'):
        old_snaps = list(csv.DictReader(open(f'{LIB}/snapshots.csv')))
    for s in old_snaps:
        k = (s['experiment'], s['line'], int(s['version']))
        if k not in seen:
            seen.add(k)
            uniq.append(s)
    with open(f'{LIB}/snapshots.csv', 'w', newline='') as f:
        w = csv.DictWriter(f, ['experiment', 'line', 'version', 'snapshot', 'md5'])
        w.writeheader()
        w.writerows(uniq)
    print(f'{len(rows)} index rows ({sum(r["plateaued"] == "yes" for r in rows)} plateaued), {len(uniq)} snapshots; '
          f'skipped (never ran): {", ".join(skipped) or "none"}')


if __name__ == '__main__':
    main()
