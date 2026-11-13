#!/usr/bin/env python3
"""DRAFT read of the ply-2 rung of training standard v1 (models/STANDARD.md; analyst 6.95; written 2026-11-12 by the
ply-2 prep fork, nothing launched): the warm runs std-v1-p2-s1 and -s2 (each line from its own seed's ply-1 final)
against the scratch run std-v1-p2-x3, per recipe ("kind") and pooled, on a standing stranger panel at ply 2.

    python3 std_p2_read.py make early|final [--rounds N|ciX] [--name NAME] [--out-dir DIR] [--arena-dir DIR]
                                            [--runs w1=EXP,w2=EXP,x3=EXP] [--at w1=V,w2=V,x3=V[+V2]]
    python3 std_p2_read.py read early|final [ARENA_DIR] [--runs ...]

POINTS (6.95: "against the ply-2 standing panel at v75, v150 and the plateau"; the warm runs cap at 101, so they have
no v150):
  early  the warm runs at v75 (their final if they stop before 75) and the scratch run at v75 and v150: per kind
         warm@75 - scratch@75, and warm@75 - scratch@150 (the prediction: warm reaches scratch's v150 level by
         v50-75). Made once the scratch run has v150 and each warm run has v75 or has ended.
  final  the three runs' finals (lines.csv's version once state.json says done): per kind warm - scratch (the
         prediction: within 25), and THE RULE: scratch 30+ over warm (pooled) -> rungs start cold; otherwise warm.
Each point is one arena at ply 2 (make_split_read.py: near argmax T 0.02, own calibration as every ply-2 read; entries
TAG<v>.L1@<v>, TAG = run code + kind code, e.g. w1tlg75 = std-v1-p2-s1's T-L-Ne-grow-f10 at v75). `make` only writes
the spec (leagues/specs/arenas/NAME.json; refuses to overwrite); create and run it with leagues/arena.py.

PANEL (judged entries never count their games against each other: the stranger read, 6.68 (3) / arm_panel.py).
  upper  y/v/l @299: replay-p2, diversity-p2 and probit-late-p2, F-P x2 and T-P-Ne capped x2 each (12 entries), the
         ply-2 standing panel since archparity-p2. Elo 0 = their mean, in both points (one scale).
  lower  (early only) the ply-1 standing panel played at ply 2: error-routing-p1, error-dag-probit-p1, grow-p1 and
         grow-p1b at v300, F-P and tree, replicate 1 (8 entries).
  Why these (6.93: a panel judges well within ~150 of what it judges):
  - Strangers: other experiments, no shared games or lineage with any std run (warm or scratch).
  - The plateau level of ply-2 training: every ply-2 field at ~v299 sat within ~60 of each other (panel-p2fields-
    p2-v299: base-probit-p2's F-P, merge tree and DAG, y/v/l, diversity-p2c/p2d all scored 38-58% in one field), and
    ply-2-trained LR is there too (ladder-1: base-lr-p2's F-L @599 1655 at ply 2 against y/v/l @299's 1540-1668). So
    the std finals at ply 2 (scratch ~v300+, warm ply-1 finals + 41-101 ply-2 cycles) should sit inside the upper
    tier's span (+118..+268 on p3p2-p2-v301's scale, 150 wide).
  - Early versions and ply-1-trained starts sit lower: ply-1-trained v300 models played at ply 2 are 150-500 below
    y/v/l (p3p2-p2-v301: -66..-305 against +118..+268; ladder-1: error-routing-p1 @300 at ply 2 1375-1409). The
    lower tier covers that span in the early arena, so a scratch v75 or a slow warm v75 still has panel members
    within 150.
  `read` prints each judged entry's distance to the nearest panel member and flags those beyond 150.

THE FIT. A Bradley-Terry fit on the stranger games only (judged against panel, panel against panel; draws half a
win), the upper tier's mean pinned at 0; SEs from the Fisher information (binomial, draws counted as half: a little
conservative, as arm_panel.py). Contrasts use the fit's covariance. For continuity the final point also prints
arm_panel.py's number (each entry's pooled score against the upper tier, as Elo).

NOISE. The fit's intervals are sampling only. Run-to-run noise is larger (version-noise: SD ~35-45 per line between
runs): the two warm seeds measure it here (s1 - s2 per kind and pooled), and with one scratch seed a per-kind
warm - scratch contrast carries ~sqrt(1.5) run SDs. The decision reads the POOLED contrast (8 kinds)."""
import csv, json, math, os, re, subprocess, sys

REPO = '/home/iv/dev/chessai'
EXPS = os.path.join(REPO, 'leagues', 'experiments')
MAKE_SPLIT = os.path.join(REPO, 'leagues', 'specs', 'arenas', 'make_split_read.py')
RUNS = {'w1': 'std-v1-p2-s1', 'w2': 'std-v1-p2-s2', 'x3': 'std-v1-p2-x3'}
WARM, SCRATCH = ('w1', 'w2'), 'x3'
KINDS = [  # code, regex on lines.csv's line, label
    ('fl', r'^F-L\.', 'F-L'), ('fp', r'^F-P\.', 'F-P'),
    ('tlc', r'^T-L-Ne\+', 'T-L-Ne capped'), ('tpc', r'^T-P-Ne\+', 'T-P-Ne capped'),
    ('tlg', r'^T-L-Ne-grow', 'T-L-Ne-grow-f10'), ('tpg', r'^T-P-Ne-grow', 'T-P-Ne-grow'),
    ('dl', r'^D-L-', 'D-L-Ne-ctx-grow'), ('dp', r'^D-P-', 'D-P-Ne-ctx-grow')]
UPPER = ['yfp=replay-p2:^F-P#@299', r'ytr=replay-p2:^T-P-Ne\+@299', 'vfp=diversity-p2:^F-P#@299',
         r'vtr=diversity-p2:^T-P-Ne\+@299', 'lfp=probit-late-p2:^F-P#@299', r'ltr=probit-late-p2:^T-P-Ne\+@299']
LOWER = ['rfp=error-routing-p1:^F-P#1@300', r'rtr=error-routing-p1:^T-P-Ne\+.*#1@300',
         'efp=error-dag-probit-p1:^F-P#1@300', r'etr=error-dag-probit-p1:^T-P-Ne\+.*#1@300',
         'gfp=grow-p1:^F-P#1@300', r'gtr=grow-p1:^T-P-Ne\+.*#1@300',
         'hfp=grow-p1b:^F-P#1@300', 'htr=grow-p1b:^T-P-Ne-grow#1@300']
UPPER_TAGS = [a.split('=')[0] for a in UPPER]
LOWER_TAGS = [a.split('=')[0] for a in LOWER]
DEFAULT_ROUNDS = {'early': '20', 'final': '30'}
ELO = 400 / math.log(10)
JUDGED_RE = re.compile(r'^(w1|w2|x3)(' + '|'.join(k for k, _, _ in KINDS) + r')(\d+)$')
PANEL_RE = re.compile(r'^(' + '|'.join(UPPER_TAGS + LOWER_TAGS) + r')(\d+)$')


def opt(argv, name, default=None):
    if name in argv:
        i = argv.index(name)
        val = argv[i + 1]
        del argv[i:i + 2]
        return val
    return default


def run_state(exp):
    """(status, version): state.json's status and the lines' common version (lines.csv)."""
    d = os.path.join(EXPS, exp)
    if not os.path.isfile(os.path.join(d, 'state.json')):
        sys.exit(f'{exp}: no such experiment (not created yet?)')
    st = json.load(open(os.path.join(d, 'state.json'))).get('status')
    vs = {int(r['version']) for r in csv.DictReader(open(os.path.join(d, 'lines.csv')))}
    assert len(vs) == 1, f'{exp}: the lines have versions {sorted(vs)}'
    return st, vs.pop()


def points(point, runs, at):
    """{run code: [versions]} of a point: given (--at) or from the runs' states."""
    if at:
        out = {}
        for a in at.split(','):
            r, v = a.split('=')
            out[r] = [int(x) for x in v.split('+')]
        assert set(out) == set(runs), f'--at needs {sorted(runs)}'
        return out
    st = {r: run_state(e) for r, e in runs.items()}
    out = {}
    if point == 'early':
        for r in WARM:
            status, v = st[r]
            if v < 75 and status != 'done':
                sys.exit(f'{runs[r]} is at v{v} and running: the early read needs v75 (or its end)')
            out[r] = [min(75, v)]
        status, v = st[SCRATCH]
        if v < 150:
            sys.exit(f'{runs[SCRATCH]} is at v{v}: the early read needs its v150')
        out[SCRATCH] = [75, 150]
    else:
        for r, (status, v) in st.items():
            if status != 'done':
                sys.exit(f'{runs[r]} is {status} at v{v}: the final read needs every run done')
            out[r] = [v]
    return out


def make(point, argv):
    rounds = opt(argv, '--rounds', DEFAULT_ROUNDS[point])
    name = opt(argv, '--name', f'std-v1-p2-read-{point}')
    out_dir, arena_dir = opt(argv, '--out-dir'), opt(argv, '--arena-dir')
    runs = dict(RUNS)
    r_arg, at = opt(argv, '--runs'), opt(argv, '--at')
    assert not argv, f'unknown arguments {argv}'
    if r_arg:
        runs.update(dict(a.split('=') for a in r_arg.split(',')))
    ver = points(point, runs, at)
    tags = []
    for r, vs in ver.items():
        for v in vs:
            for k, rx, _ in KINDS:
                tags.append(f'{r}{k}={runs[r]}:{rx}@{v}')
    tags += UPPER + (LOWER if point == 'early' else [])
    cmd = [sys.executable, MAKE_SPLIT, name, str(max(max(v) for v in ver.values())), '2', rounds] + tags
    cmd += ['--anchor', ','.join(UPPER_TAGS)]
    if out_dir:
        cmd += ['--out-dir', out_dir]
    if arena_dir:
        cmd += ['--arena-dir', arena_dir]
    print(' '.join(cmd[1:3]), '...', flush=True)
    subprocess.run(cmd, check=True)
    spec = os.path.join(out_dir or os.path.join(REPO, 'leagues', 'specs', 'arenas'), name + '.json')
    s = json.load(open(spec))
    judged = sum(1 for e in s['entries'] if JUDGED_RE.match(e['name'].split('.')[0]))
    assert judged == 8 * sum(len(v) for v in ver.values()), (judged, ver)
    for e in s['entries']:  # one line per (run, kind) and per panel rule, except the upper tier's two replicates
        assert e['name'].split('.')[1].split('@')[0] in ('L1', 'L2'), e['name']
    s['description'] = (f"std-v1-p2 {point} read (std_p2_read.py, DRAFT 2026-11-12): " +
                        ', '.join(f"{runs[r]} @{'+'.join(map(str, v))}" for r, v in ver.items()) +
                        f"; panel y/v/l @299{' + the ply-1 panel r/e/g/h @300 (replicate 1)' if point == 'early' else ''}"
                        f", stranger read on its upper tier's mean. " + s['description'])
    json.dump(s, open(spec, 'w'), indent=1)
    n = len(s['entries'])
    print(f'{spec}: {n} entries ({judged} judged), {rounds} round-robins of {n * (n - 1)} games')


# ------------------------------------------------------------------------------------------------ the read

def tag_of(player):
    """An entry's tag, the name up to the first '.' (h2h_btg.py's): w1tlg75, yfp299."""
    return player.split('.')[0]


def player_of(name):
    """A games.csv player as a fit player: the entry name without its ply suffix (yfp299.L1@299-2 ->
    yfp299.L1@299), so a panel tag's two replicates stay two players; a judged tag has one entry (L1)."""
    return name.rsplit('-', 1)[0]


def load(arena):
    """Stranger games: {(i, j): [games, score of i]} over players (judged entries and panel entries), judged v
    judged games dropped."""
    pairs, names = {}, set()
    for r in csv.DictReader(open(os.path.join(arena, 'tour', 'games.csv'))):
        a, b = player_of(r['white']), player_of(r['black'])
        ja, jb = JUDGED_RE.match(tag_of(a)), JUDGED_RE.match(tag_of(b))
        if (ja and jb) or not (ja or PANEL_RE.match(tag_of(a))) or not (jb or PANEL_RE.match(tag_of(b))):
            continue  # judged v judged (a matchup, not a stranger read), or a player of neither group
        s = {'1-0': 1.0, '0-1': 0.0, '1/2-1/2': 0.5}[r['result']]
        key, sc = ((a, b), s) if a < b else ((b, a), 1 - s)
        g = pairs.setdefault(key, [0, 0.0])
        g[0] += 1
        g[1] += sc
        names.update((a, b))
    return sorted(names), pairs


def bt_fit(names, pairs, anchor):
    """Bradley-Terry (logistic) ratings in natural units, mean of `anchor` at 0, by Newton's method; returns
    (ratings, covariance) as dicts. The free parameters are all but the first anchor player, whose rating is set so
    that the anchor's mean is 0."""
    idx = {p: i for i, p in enumerate(names)}
    n = len(names)
    a_idx = [idx[p] for p in anchor]
    free = [i for i in range(n) if i != a_idx[0]]
    # theta_free -> full ratings: x_i = t_i (free), x_a0 = -sum(other anchors' t)
    m = len(free)
    J = [[0.0] * m for _ in range(n)]  # d x / d t
    for c, i in enumerate(free):
        J[i][c] = 1.0
    for i in a_idx[1:]:
        J[a_idx[0]][free.index(i)] = -1.0
    t = [0.0] * m

    def full(t):
        x = [0.0] * n
        for c, i in enumerate(free):
            x[i] = t[c]
        x[a_idx[0]] = -sum(t[free.index(i)] for i in a_idx[1:])
        return x

    for it in range(100):
        x = full(t)
        g = [0.0] * n
        H = [[0.0] * n for _ in range(n)]
        for (a, b), (ng, sc) in pairs.items():
            i, j = idx[a], idx[b]
            p = 1 / (1 + math.exp(x[j] - x[i]))
            d = sc - ng * p
            g[i] += d
            g[j] -= d
            w = ng * p * (1 - p)
            H[i][i] += w
            H[j][j] += w
            H[i][j] -= w
            H[j][i] -= w
        gt = [sum(J[k][c] * g[k] for k in range(n)) for c in range(m)]
        Ht = [[sum(J[k][c] * sum(H[k][l] * J[l][e] for l in range(n) if J[l][e]) for k in range(n) if J[k][c])
               for e in range(m)] for c in range(m)]
        step = solve(Ht, gt)
        t = [t[c] + step[c] for c in range(m)]
        if max(abs(s) for s in step) < 1e-10:
            break
    cov_t = inverse(Ht)
    x = full(t)
    cov = [[sum(J[k][c] * sum(cov_t[c][e] * J[l][e] for e in range(m) if J[l][e]) for c in range(m) if J[k][c])
            for l in range(n)] for k in range(n)]
    return ({p: x[idx[p]] for p in names},
            {(p, q): cov[idx[p]][idx[q]] for p in names for q in names})


def solve(A, b):
    n = len(b)
    M = [row[:] + [b[i]] for i, row in enumerate(A)]
    for c in range(n):
        piv = max(range(c, n), key=lambda r: abs(M[r][c]))
        M[c], M[piv] = M[piv], M[c]
        for r in range(n):
            if r != c and M[r][c]:
                f = M[r][c] / M[c][c]
                for k in range(c, n + 1):
                    M[r][k] -= f * M[c][k]
    return [M[i][n] / M[i][i] for i in range(n)]


def inverse(A):
    n = len(A)
    return [list(col) for col in zip(*[solve(A, [1.0 if i == j else 0.0 for i in range(n)]) for j in range(n)])]


def contrast(w, rat, cov):
    """sum w_p x_p in Elo, with its SE."""
    est = sum(c * rat[p] for p, c in w.items())
    var = sum(c * d * cov[(p, q)] for p, c in w.items() for q, d in w.items())
    return est * ELO, math.sqrt(max(var, 0.0)) * ELO


def fmt(e, se):
    return f'{e:+6.0f} [{e - 1.96 * se:+5.0f}, {e + 1.96 * se:+5.0f}]'


def panel_elo(pairs, player, panel):
    """arm_panel.py's number: the player's pooled score against the panel as Elo, with its delta-method SE."""
    n = s = 0.0
    for (a, b), (ng, sc) in pairs.items():
        if a == player and b in panel:
            n, s = n + ng, s + sc
        elif b == player and a in panel:
            n, s = n + ng, s + ng - sc
    if not n:
        return None
    p = min(max(s / n, 1e-4), 1 - 1e-4)
    return -400 * math.log10(1 / p - 1), ELO * math.sqrt(p * (1 - p) / n) / (p * (1 - p))


def read(point, argv):
    runs = dict(RUNS)
    r_arg = opt(argv, '--runs')
    if r_arg:
        runs.update(dict(a.split('=') for a in r_arg.split(',')))
    arena = argv[0] if argv else os.path.join(REPO, 'leagues', 'arenas', 'diag', f'std-v1-p2-read-{point}')
    names, pairs = load(arena)
    upper = [p for p in names if PANEL_RE.match(tag_of(p)) and PANEL_RE.match(tag_of(p)).group(1) in UPPER_TAGS]
    panel = [p for p in names if PANEL_RE.match(tag_of(p))]
    judged = {}
    for p in names:
        mm = JUDGED_RE.match(tag_of(p))
        if mm:
            key = (mm.group(1), mm.group(2), int(mm.group(3)))
            assert key not in judged, f'two entries of {tag_of(p)}'
            judged[key] = p
    assert upper, 'no upper-tier panel players in the arena'
    rat, cov = bt_fit(names, pairs, upper)
    ngames = sum(g for g, _ in pairs.values())
    print(f'{arena}: {point} read, {len(judged)} judged and {len(panel)} panel players, {ngames} stranger games '
          f'(judged v judged dropped); Elo 0 = the mean of y/v/l @299 ({len(upper)} players)')
    print('\nPanel:', '  '.join(f'{p.split("@")[0]} {rat[p] * ELO:+.0f}' for p in sorted(panel, key=lambda p: -rat[p])))
    print('\nJudged entries (stranger fit, Elo +- 95%; nearest panel member; arm_panel Elo vs the upper tier):')
    far = []
    for key in sorted(judged, key=lambda k: ([c for c, _, _ in KINDS].index(k[1]), k[0], k[2])):
        p = judged[key]
        e = rat[p] * ELO
        se = math.sqrt(cov[(p, p)]) * ELO
        near = min(panel, key=lambda q: abs(rat[q] - rat[p]))
        dist = (rat[p] - rat[near]) * ELO
        ap = panel_elo(pairs, p, set(upper))
        flag = '  BEYOND 150' if abs(dist) > 150 else ''
        if flag:
            far.append(p)
        print(f'  {tag_of(p):10s} {runs[key[0]]:14s} {dict((k, l) for k, _, l in KINDS)[key[1]]:16s} v{key[2]:<4d} '
              f'{e:+6.0f} +-{1.96 * se:3.0f}  nearest {near.split("@")[0]:12s} {dist:+5.0f}{flag}'
              + (f'  arm_panel {ap[0]:+5.0f} +-{1.96 * ap[1]:3.0f}' if ap else '  arm_panel -'))
    if far:
        print(f'  {len(far)} entries have no panel member within 150 (6.93): their contrasts lean on the fit\'s '
              f'extrapolation; read them with care')
    vers = {r: sorted({v for (rr, _, v) in judged if rr == r}) for r in runs}
    rows = []
    if point == 'early':
        wv = {r: vers[r][0] for r in WARM}
        rows.append(('warm@75 - scratch@75', {**{(r, wv[r]): 0.5 for r in WARM}, (SCRATCH, 75): -1.0}))
        rows.append(('warm@75 - scratch@150 (6.95: >= 0 by v50-75)',
                     {**{(r, wv[r]): 0.5 for r in WARM}, (SCRATCH, 150): -1.0}))
        rows.append(('scratch@150 - scratch@75', {(SCRATCH, 150): 1.0, (SCRATCH, 75): -1.0}))
        rows.append(('s1 - s2 @75 (replicate noise)', {(WARM[0], wv[WARM[0]]): 1.0, (WARM[1], wv[WARM[1]]): -1.0}))
    else:
        fv = {r: vers[r][0] for r in runs}
        rows.append(('warm - scratch (6.95: within 25)',
                     {**{(r, fv[r]): 0.5 for r in WARM}, (SCRATCH, fv[SCRATCH]): -1.0}))
        rows.append(('s1 - s2 (replicate noise)', {(WARM[0], fv[WARM[0]]): 1.0, (WARM[1], fv[WARM[1]]): -1.0}))
    print('\nContrasts per kind (stranger fit; sampling 95% only), and POOLED = the mean over kinds:')
    kinds = [k for k, _, _ in KINDS]
    labels = dict((k, l) for k, _, l in KINDS)
    res = {}
    for title, w in rows:
        print(f'  {title}')
        for k in kinds + ['POOLED']:
            ks = kinds if k == 'POOLED' else [k]
            ww = {}
            ok = True
            for (r, v), c in w.items():
                for kk in ks:
                    p = judged.get((r, kk, v))
                    if p is None:
                        ok = False
                        continue
                    ww[p] = ww.get(p, 0.0) + c / len(ks)
            if not ok:
                print(f'    {labels.get(k, k):16s} (missing entries)')
                continue
            e, se = contrast(ww, rat, cov)
            res[(title, k)] = (e, se)
            print(f'    {labels.get(k, k):16s} {fmt(e, se)}')
    # Replicate-based run noise: s1 - s2 per kind, SD_run^2 ~ mean(d^2)/2 - sampling.
    rep = [t for t, _ in rows if t.startswith('s1 - s2')][0]
    d2 = [res[(rep, k)][0] ** 2 - res[(rep, k)][1] ** 2 for k in kinds if (rep, k) in res]
    if d2:
        sd_run = math.sqrt(max(sum(d2) / len(d2) / 2, 0.0))
        print(f'\nRun noise from the two warm seeds: SD ~{sd_run:.0f} Elo per line (8 kinds, 1 df each; '
              f'version-noise says ~35-45). A per-kind warm - scratch contrast then carries ~{sd_run * math.sqrt(1.5):.0f}'
              f' of run noise (1 SD), the pooled one ~{sd_run * math.sqrt(1.5) / math.sqrt(8):.0f} if the kinds\' run '
              f'effects were independent (they share each run\'s games, so likely more: compare the pooled s1 - s2 '
              f'{res[(rep, "POOLED")][0]:+.0f}).')
    if point == 'final':
        e, se = res[(rows[0][0], 'POOLED')]
        print(f'\nTHE RULE (6.95): scratch - warm pooled = {-e:+.0f} [{-e - 1.96 * se:+.0f}, {-e + 1.96 * se:+.0f}] '
              f'(sampling): {"30+ -> rungs start COLD" if -e >= 30 else "below 30 -> WARM is the rule"}'
              f'{" (but within the noise of the bar: see the run noise above)" if abs(-e - 30) < 1.96 * se else ""}')


def selftest():
    """The fit against a known truth: a simulated stranger arena (judged v panel and panel v panel games, draws),
    whose fitted ratings must sit near the truth, the anchor's mean at 0, and contrasts' SEs near their spread."""
    import random
    rnd = random.Random(7)
    truth = {f'{t}299.L{k}@299': rnd.uniform(-75, 75) / ELO for t in UPPER_TAGS for k in (1, 2)}
    truth.update({f'{t}300.L1@300': rnd.uniform(-450, -200) / ELO for t in LOWER_TAGS})
    up = [p for p in truth if '299' in p]
    m = sum(truth[p] for p in up) / len(up)
    truth = {p: x - m for p, x in truth.items()}
    for r in ('w1', 'w2', 'x3'):
        for k, _, _ in KINDS:
            truth[f'{r}{k}75.L1@75'] = rnd.uniform(-300, 50) / ELO
    names = sorted(truth)
    pairs = {}
    for a in names:
        for b in names:
            if a >= b or (JUDGED_RE.match(tag_of(a)) and JUDGED_RE.match(tag_of(b))):
                continue
            p = 1 / (1 + math.exp(truth[b] - truth[a]))
            dr = 0.4 * p * (1 - p)  # draws (up to 10%) that keep the expected score at p
            ng, sc = 400, 0.0
            for _ in range(ng):
                u = rnd.random()
                sc += 1.0 if u < p - dr / 2 else 0.5 if u < p + dr / 2 else 0.0
            pairs[(a, b)] = [ng, sc]
    upper = [p for p in names if p.endswith('@299')]
    rat, cov = bt_fit(names, pairs, upper)
    assert abs(sum(rat[p] for p in upper)) < 1e-9
    err = [abs(rat[p] - truth[p]) * ELO for p in names]
    se = [math.sqrt(cov[(p, p)]) * ELO for p in names]
    bad = [p for p, e, s in zip(names, err, se) if e > 4 * s + 1]
    print(f'selftest: {len(names)} players, max error {max(err):.1f} Elo, median SE {sorted(se)[len(se) // 2]:.1f}; '
          f'bad {bad}')
    assert not bad
    w = {'w1fl75.L1@75': 0.5, 'w2fl75.L1@75': 0.5, 'x3fl75.L1@75': -1.0}
    e, s = contrast(w, rat, cov)
    t = sum(c * truth[p] for p, c in w.items()) * ELO
    print(f'selftest: contrast {e:+.1f} +- {s:.1f} (truth {t:+.1f})')
    assert abs(e - t) < 4 * s + 1
    print('selftest ok')


if __name__ == '__main__':
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    cmd, rest = sys.argv[1], sys.argv[2:]
    if cmd == 'selftest':
        selftest()
    elif cmd in ('make', 'read') and rest and rest[0] in ('early', 'final'):
        (make if cmd == 'make' else read)(rest[0], rest[1:])
    else:
        sys.exit(__doc__)
