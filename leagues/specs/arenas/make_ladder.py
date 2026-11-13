#!/usr/bin/env python3
"""The global ladder's arena spec, from the model library (leagues/library). The user, 11-12 ~11:11: "not train anyone
in it at all. and simply have it be an arena that runs occasionally with all of our promising models where the models
run at the ply that's specified for them and that helps us establish a global ELO ladder" ("Yeah, I think that's a
better future for it"); analyst 6.85; models/NEW-DESIGN.md, "The global league's new purpose".

    python3 leagues/specs/arenas/make_ladder.py NAME [--rounds N|ciX] [--plies 1,2,3,4] [--max-per-ply 16]
        [--families F,F,...] [--ref-replicates R] [--anchor-plies 1,2] [--depth-anchors K]
        [--include ID[@V][:PLY] ...] [--exclude REGEX] [--binary B] [--threads T] [--jobs J]
        [--out-dir DIR] [--arena-dir DIR] [--dry-run]

Players are (model, depth): each chosen library model plays at its specified ply (its training ply), one entry per
model. The random players are in (the user, 11-12 ~11:55: "for the global ladder we should add the random players back
in"): random, random-1, random-2 and random-3 (plies 0-3) play in the field and random is Elo 0, arena.py's default
anchor. With --anchor panels there is no random player and the reference set's mean is pinned at Elo 0 instead
(arena.py's "anchor"; ARENA.md, Pinned anchor; the 11:30 design). 10 double round-robins by default
(--rounds; ciX: until every 95% interval is at most X Elo). Writes leagues/specs/arenas/NAME.json (the arena dir
leagues/arenas/ladder/NAME) and prints the entries per ply and the estimated CPU. Nothing is run.

THE SELECTION RULE (also written into the spec's description):
 1. The reference set, at the anchor plies (default 1 and 2): the standing panels, their first replicates (--ref-
    replicates): ply 1 error-routing-p1, error-dag-probit-p1, grow-p1, grow-p1b at v300 (an F-P and a tree each);
    ply 2 replay-p2, diversity-p2, probit-late-p2 at v299 (an F-P and a T-P-Ne each). Their mean is Elo 0.
    Plies 3 and 4 have no standing panel; PROVISIONAL sets (flat-p3 and flat-p3-b's F-L at v199; flat-p3-w4's F-L at
    v99) join the anchor only with --anchor-plies.
 2. At a ply with a standing panel, one entry per promising family (--families; default the analyst's six and the two
    DAGs: F-P, F-L, T-P-Ne, T-P-Ne-grow, T-L-Ne, T-L-Ne-grow, D-P-Ne-ctx, D-L-Ne-ctx): the family's latest plateaued
    run at that ply, at its final version; within the run, the variant the record kept (PREFER below: LR trees at
    forget 10, the growing probit tree with its leaf bar on, the DAGs' router-split z of the last global league),
    else its first line.
 3. At a ply without a standing panel (3, 4): one entry per (run, family) of every finished run at that ply.
 4. Candidates are new-system runs only: no global league champions, no old mixed-ply or self-play runs, no random
    players in the field. At most --max-per-ply entries a ply: references first, then families in --families order
    (rule 2) or runs latest first (rule 3).
 5. --include adds library rows (ID[@V][:PLY]); --exclude drops experiments (regex).
 6. --depth-anchors K: the first K reference entries of each anchor ply also play at the plies next to it (within the
    ladder's plies), as separate players outside the anchor, labelled: they calibrate the depth axis.

What the ladder can say (analyst 6.85): it ranks DEPLOYED players (evaluator plus search depth). It cannot rank
evaluators apart from depth or give a transitive cross-ply scale (models swap ~380 between plies; kind pairs carry
40-70 of matchup): arena.py's fit report (fit.md) gives the residual SD by ply pair and the largest deviations.
"""
import argparse, csv, json, os, re, sys

REPO = '/home/iv/dev/chessai'
sys.path.insert(0, f'{REPO}/leagues/library')
import library as L  # noqa: E402  (rows, entry, replicate)

HYBRID17 = 'models/from-scratch/bin/chessai-hybrid17'
# 11-12 14:17: chessai-hybrid18 plays the same games as hybrid17 and saves x0.36-0.62 game CPU a ply at plies 3-5
# (fastbin/PERF-LEDGER.md, hybrid18); the ladder's ply-3/4 players dominate its cost, so it is the default.
HYBRID18 = 'models/from-scratch/bin/chessai-hybrid18'
FAMILIES = ['F-P', 'F-L', 'T-P-Ne', 'T-P-Ne-grow', 'T-L-Ne', 'T-L-Ne-grow', 'D-P-Ne-ctx', 'D-L-Ne-ctx']
# Within a run, the variant the record kept (models/NEXT-EXPERIMENTS.md, 11-12): LR trees at --forget-factor 10
# (6.62/6.67: capped-l-f10-p1, grow-l-p1b); the growing probit tree with its leaf MDL bar on (6.71/6.72: not
# grow-mdl=0); the DAGs at the router-split z of the first global league's last lines (D-P z30, D-L z160; z120 at
# ply 2, where base-lr-p2 ran it). Families not listed take the run's first line.
PREFER = {'T-L-Ne': r'forget-factor=10', 'T-L-Ne-grow': r'forget-factor=10', 'T-P-Ne-grow': r'^T-P-Ne-grow#',
          'D-P-Ne-ctx': r'router-split-z=30\+', 'D-L-Ne-ctx': r'router-split-z=1[26]0\+'}
# The standing panels (the panel reads' tags since 11-12 ~06:20) and the provisional sets of plies 3-4:
# (experiment, line regex, version).
REFERENCES = {
    1: [('error-routing-p1', r'^F-P#', 300), ('error-routing-p1', r'^T-P-Ne\+', 300),
        ('error-dag-probit-p1', r'^F-P#', 300), ('error-dag-probit-p1', r'^T-P-Ne\+', 300),
        ('grow-p1', r'^F-P#', 300), ('grow-p1', r'^T-P-Ne\+', 300),
        ('grow-p1b', r'^F-P#', 300), ('grow-p1b', r'^T-P-Ne-grow#', 300)],
    2: [('replay-p2', r'^F-P#', 299), ('replay-p2', r'^T-P-Ne\+', 299),
        ('diversity-p2', r'^F-P#', 299), ('diversity-p2', r'^T-P-Ne\+', 299),
        ('probit-late-p2', r'^F-P#', 299), ('probit-late-p2', r'^T-P-Ne\+', 299)],
    3: [('flat-p3', r'^F-L#', 199), ('flat-p3-b', r'^F-L#', 199)],  # provisional (no standing panel at ply 3)
    4: [('flat-p3-w4', r'^F-L#', 99)],  # provisional (the only ply-4 run)
}
STANDING = {1, 2}  # plies whose reference set is a standing panel
# CPU per game between two players of the same class at a ply, ms (11-12, the game_cpu.csv of archparity-p1..p4 and
# panel-ply4-existing, chessai-hybrid16, T 0.02; flat = lr/probit/cnbp, tree = bdt/bdd). A game between classes or
# plies costs about the mean of the two players' figures (checked at plies 1-3: 0.9 vs 0.85, 13.4 vs 13.6, 98 vs 102).
COST_MS = {1: {'flat': 0.5, 'tree': 1.2}, 2: {'flat': 7.6, 'tree': 19.5}, 3: {'flat': 41.6, 'tree': 162.7},
           4: {'flat': 438.0, 'tree': 2084.0}}


def family(model):
    return re.split(r'[+#]', model)[0]


def kind_class(kind):
    return 'tree' if kind in ('bdt', 'bdd') else 'flat'


def ended(exp):
    try:
        return json.load(open(f'{REPO}/leagues/experiments/{exp}/state.json')).get('updated', '')
    except (OSError, ValueError):
        return ''


def candidate(r):
    """A library row the ladder may take: a new-system experiment's final, no random players in its field, a
    participant (observers, which train on a field's games without playing, enter only by --include; 11-12 ~12:00)."""
    return (r['experiment'] != 'global' and re.fullmatch(r'\d', r['train_ply'] or '') is not None
            and r['version_role'] == 'final' and r['random_players'] in ('', '0') and r['line_status'] != 'observer')


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('name')
    ap.add_argument('--rounds', default='10')
    ap.add_argument('--plies', default='1,2,3,4')
    ap.add_argument('--max-per-ply', type=int, default=16)
    ap.add_argument('--families', default=','.join(FAMILIES))
    ap.add_argument('--ref-replicates', type=int, default=1)
    ap.add_argument('--anchor-plies', default='1,2')
    ap.add_argument('--depth-anchors', type=int, default=0)
    ap.add_argument('--anchor', choices=['random', 'panels'], default='random',
                    help='random (default; the user, 11-12 ~11:55: "for the global ladder we should add the random '
                         'players back in"): random, random-1, random-2 and random-3 (plies 0-3) play in the field and '
                         'random is Elo 0, arena.py\'s default anchor; panels: no random player, the reference '
                         'set\'s mean pinned at Elo 0 (analyst 6.85; ARENA.md, Pinned anchor)')
    ap.add_argument('--include', action='append', default=[])
    ap.add_argument('--exclude')
    ap.add_argument('--binary', default=HYBRID18)
    ap.add_argument('--threads', type=int, default=2)
    ap.add_argument('--jobs', type=int, default=2)
    ap.add_argument('--out-dir', default=f'{REPO}/leagues/specs/arenas')
    ap.add_argument('--arena-dir')
    ap.add_argument('--dry-run', action='store_true', help='print the selection and the estimate; write nothing')
    args = ap.parse_args()
    plies = [int(p) for p in args.plies.split(',')]
    anchor_plies = [int(p) for p in args.anchor_plies.split(',') if int(p) in plies]
    fams = args.families.split(',')
    rows = [r for r in L.rows() if candidate(r)
            and not (args.exclude and re.search(args.exclude, r['experiment']))]
    by_exp = {}
    for r in L.rows():  # references may be any row (the panels' runs are candidates too)
        by_exp.setdefault(r['experiment'], []).append(r)
    chosen = {p: [] for p in plies}  # ply -> [(row, version, role)]
    seen = set()

    def add(p, r, v, role):
        key = (r['experiment'], r['line'], v)
        if key in seen or len(chosen[p]) >= args.max_per_ply:
            return False
        seen.add(key)
        chosen[p].append((r, v, role))
        return True

    # 1. the reference sets
    for p in plies:
        for exp, rx, v in REFERENCES.get(p, []):
            hit = sorted((r for r in by_exp.get(exp, []) if re.search(rx, r['line']) and r['version_role'] == 'final'),
                         key=lambda r: L.replicate(r['line']))
            if not hit:
                sys.exit(f'reference {exp} {rx}: no library row')
            for r in hit[:args.ref_replicates]:
                add(p, r, v, 'reference' if p in anchor_plies else 'provisional reference')
    # 2./3. promising entries
    for p in plies:
        at = [r for r in rows if r['train_ply'] == str(p)]
        if p in STANDING:
            for f in fams:
                runs = {}
                for r in at:
                    if family(r['model']) == f and r['plateaued'] == 'yes':
                        runs.setdefault(r['experiment'], []).append(r)
                if not runs:
                    continue
                exp = max(runs, key=lambda e: (ended(e), e))
                lines = runs[exp]
                pref = [r for r in lines if f in PREFER and re.search(PREFER[f], r['line'])]
                r = (pref or lines)[0]
                add(p, r, int(r['version']), f'family {f}')
        else:
            groups = {}
            for r in at:
                if r['end_reason'] in ('plateau', 'cap'):
                    groups.setdefault((r['experiment'], family(r['model'])), []).append(r)
            for (exp, f), lines in sorted(groups.items(), key=lambda kv: (ended(kv[0][0]), kv[0]), reverse=True):
                r = sorted(lines, key=lambda r: L.replicate(r['line']))[0]
                add(p, r, int(r['version']), f'run {exp} {f}')
    # 5. includes
    for inc in args.include:
        m = re.fullmatch(r'([^@:]+)(?:@(\d+))?(?::(\d))?', inc)
        if not m:
            sys.exit(f'--include {inc}: ID[@V][:PLY]')
        r = next((x for x in L.rows() if x['id'] == m[1]), None)
        if r is None:
            sys.exit(f'--include {inc}: no library id {m[1]}')
        p = int(m[3] or r['train_ply'])
        if p not in chosen:
            sys.exit(f'--include {inc}: ply {p} is not among --plies')
        if not add(p, r, int(m[2] or r['version']), 'included'):
            print(f'# --include {inc}: already in or ply {p} full', file=sys.stderr)
    # entries
    entries, ref_players, extra, players = [], [], {}, []  # players: (ply, kind class) for the estimate
    lo, hi = min(plies), max(plies)
    for p in plies:
        for i, (r, v, role) in enumerate(chosen[p]):
            e = L.entry(r, v, 'experiment')
            e['ply'] = p
            e['label'] = f"{e['label']}; ladder: ply {p}, {role}"
            if role == 'reference':
                ref_players.append(f"{e['name']}-{p}")
                if i < args.depth_anchors:  # references come first in chosen[p]
                    more = [q for q in (p - 1, p + 1) if lo <= q <= hi]
                    if more:
                        del e['ply']
                        e['plies'] = [p] + more
                        e['label'] += f"; also at ply {'/'.join(map(str, more))} as a depth anchor (outside the anchor)"
                        for q in more:  # counted at the ply they play at
                            extra[q] = extra.get(q, 0) + 1
            entries.append(e)
            players += [(q, kind_class(r['kind'])) for q in e.get('plies', [p])]
    names = [e['name'] for e in entries]
    dup = sorted({n for n in names if names.count(n) > 1})
    if dup:
        sys.exit(f'two entries share a name: {dup}')
    if not ref_players:
        sys.exit('no reference players at the anchor plies')
    # stop rule
    if args.rounds.startswith('ci'):
        ci = float(args.rounds[2:])
        stop = {'ci95': ci, 'chunk_rounds': 5, 'min_rounds': 10, 'max_rounds': 100}
        howlong = f'double round-robins until every 95% interval is at most {ci:g} Elo (10-100)'
    else:
        n = int(args.rounds)
        stop = {'ci95': 0.0, 'chunk_rounds': n, 'min_rounds': n, 'max_rounds': n}
        howlong = f'{n} double round-robins'
    # estimate: per double round-robin every pair plays 2 games at about the mean of the two players' costs, so the
    # total is rounds x (players - 1) x the sum of the players' costs
    randoms = ['random', 'random-1', 'random-2', 'random-3'] if args.anchor == 'random' else []
    rand_ms = sum(COST_MS[max(1, int(r.split('-')[1]) if '-' in r else 1)]['flat'] for r in randoms)  # a guess: a
    # random evaluator costs about a flat model's game at its ply
    per_rr = (len(players) + len(randoms) - 1) * (sum(COST_MS[p][c] for p, c in players) + rand_ms) / 1000
    n_rr = stop['max_rounds']
    counts = {p: sum(1 for x in players if x[0] == p) for p in plies}
    desc = (f"Global ladder {args.name} (the user, 11-12 ~11:11; analyst 6.85): no training; each promising library "
            f"model plays at its specified ply (its training ply), one entry per model; players at plies "
            + ', '.join(f'{p}: {counts[p]}' for p in plies) + f"; {howlong}, near argmax (T 0.02). "
            + (f"Elo 0 = random: the random references random, random-1, random-2 and random-3 (plies 0-3) play in the "
               f"field (the user, 11-12 ~11:55: \"for the global ladder we should add the random players back in\"); "
               f"the standing panels play as players, not as the anchor ("
               if args.anchor == 'random' else f"Elo 0 = the mean of {len(ref_players)} reference players: ")
            + f"the standing panels at plies {','.join(map(str, anchor_plies))} "
            f"(ply 1 error-routing-p1, error-dag-probit-p1, grow-p1, grow-p1b at v300; ply 2 replay-p2, diversity-p2, "
            f"probit-late-p2 at v299; an F-P and a tree each, first {args.ref_replicates} replicate(s))"
            + (")" if args.anchor == 'random' else "; no random player") + ". Selection: at plies with a standing panel, one entry per family in [{', '.join(fams)}] from the "
            f"family's latest plateaued run, its final, the variant the record kept (LR trees forget 10, grow trees "
            f"bar on, DAGs D-P z30 / D-L z160 or z120) else its first line; at plies without one, one entry per (run, "
            f"family) of every finished run there; new-system runs only (no global champions, no mixed-ply, "
            f"self-play or random-player runs); at most {args.max_per_ply} a ply, references first"
            + (f"; {args.depth_anchors} reference(s) a ply also play at the next plies as depth anchors"
               if args.depth_anchors else "")
            + (f"; included: {', '.join(args.include)}" if args.include else "")
            + (f"; excluded experiments matching {args.exclude}" if args.exclude else "")
            + ". It ranks deployed players (evaluator plus depth), not evaluators apart from depth: read fit.md.")
    spec = {'name': args.name, 'description': desc, 'seed': 1, 'nice': 15, 'threads': args.threads,
            'jobs': args.jobs, 'binary': args.binary, 'owncal_binary': args.binary,
            'flags': ['--reports-last', '--sparse-fit', '--policy', 'log-odds-softmax', '--temperature', '0.02'],
            'references': randoms,
            **({} if randoms else {'anchor': {'mean_of': ref_players, 'note': f'the standing panels at plies '
                                              f'{",".join(map(str, anchor_plies))} (make_ladder.py)'}}),
            'stop': stop, 'dir': args.arena_dir or f'leagues/arenas/ladder/{args.name}', 'entries': entries}
    for p in plies:
        print(f'ply {p}: {counts[p]} players' + (f' ({extra[p]} of them depth anchors)' if extra.get(p) else ''))
        for r, v, role in chosen[p]:
            print(f'    {r["id"]:22s} {r["experiment"]:22s} {r["line"][:58]:58s} v{v:<4d} {role}')
    print(f'{len(entries)} entries, {len(players)} players, {len(ref_players)} reference players; estimated '
          f'{per_rr / 60:.1f} CPU-min a double round-robin, {per_rr * n_rr / 3600:.2f} CPU-h for {n_rr}'
          + (' (the cap)' if args.rounds.startswith('ci') else '') + ' (own calibration games: mostly cached)')
    if args.dry_run:
        return
    out = os.path.join(args.out_dir, args.name + '.json')
    if os.path.exists(out):
        sys.exit(f'{out} exists')
    json.dump(spec, open(out, 'w'), indent=1)
    print('wrote', out)


if __name__ == '__main__':
    main()
