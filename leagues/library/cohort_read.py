#!/usr/bin/env python3
"""The cohort-variance read (11-12 ~10:00; the user: how do identically-architected, fully plateaued models perform
against each other depending on the cohort they plateaued in: how far they deviate from models of identical
experiment set-ups, and how far from models of other set-ups). Design: analyst 6.77. F-P only; one arena per
training ply, played at that ply; plateaued runs only; plain F-P lines (train args --use-probs, no overrides).

    python3 leagues/library/cohort_read.py spec PLY [ROUNDS]     the entries (and their snapshots), then the arena spec
    python3 leagues/library/cohort_read.py analyze PLY [--boot N] the read (after the arena has run)

Versions per line (6.77 (1)): v299 and v279 (equal length, both in the plateau region: the version pair) and the
final when it is past 299 (final minus v299 = training length); a line whose final is under 299: its final and
final-20, flagged short. The global league's F-P (ply 1) enters flagged, its best/last champion v463 and the one
before, v365, and is kept out of the pooled components (6.77 (7)).
"""
import argparse, csv, json, math, os, random, re, subprocess, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import library as L  # noqa: E402

REPO = L.REPO
POOL_RUNS = {'diversity-p2d': 'diversity-p2c'}  # 6.77 (3): identical through v9, same references: one run
GLOBAL_PAIR = (463, 365)


def name(ply):
    return f'cohort-fp-p{ply}'


def lines_for(ply):
    out = []
    for r in L.rows():
        if r['archetype'] != 'F-P' or r['kind'] != 'probit' or r['train_ply'] != str(ply):
            continue
        if r['train_args'].strip() != '--use-probs' or r['train_overrides'] not in ('[]', ''):
            continue  # not plain F-P (weighted arms, probit-tau settings, ...)
        if r['experiment'] == 'global':
            if r['version_role'] in ('best and last champion', 'last champion'):
                out.append(r)
            continue
        if r['plateaued'] != 'yes':
            continue
        out.append(r)
    return out


def versions(r):
    f = int(r['version'])
    if r['experiment'] == 'global':
        return list(GLOBAL_PAIR), 'global'
    if f >= 299:
        return [299, 279] + ([f] if f > 299 else []), ''
    return [f, f - 20], 'short'


def cmd_spec(args):
    rs = lines_for(args.ply)
    extras, entries = [], []
    for r in rs:
        vs, _ = versions(r)
        for v in vs:
            extras.append(f"{r['experiment']}:{r['line']}@{v}")
    subprocess.run([sys.executable, f'{REPO}/leagues/library/build_index.py', '--extra', *extras], check=True)
    for r in rs:
        vs, flag = versions(r)
        for v in vs:
            e = L.entry(r, v, 'experiment')
            e['label'] += f" [cohort read: run {POOL_RUNS.get(r['experiment'], r['experiment'])}" + \
                          (f', {flag}' if flag else '') + ']'
            entries.append(e)
    L.write_spec(name(args.ply), args.ply, args.rounds, entries, threads=4, jobs=2,
                 note=f"Cohort-variance read (analyst 6.77): plain F-P lines of plateaued runs trained at ply "
                      f"{args.ply}, versions 299/279 (+final), short lines final/final-20.")


# ---------------------------------------------------------------- analysis
def elo(s):
    s = min(max(s, 1e-4), 1 - 1e-4)
    return -400 * math.log10(1 / s - 1)


def run_of(r):
    return POOL_RUNS.get(r['experiment'], r['experiment'])


def setup_of(r):
    """The setup group; a pooled run (6.77 (3)) is one setup of its own."""
    if r['experiment'] in POOL_RUNS or r['experiment'] in POOL_RUNS.values():
        return ' + '.join(sorted({r['experiment'], *[k for k, v in POOL_RUNS.items() if v == run_of(r)],
                                  run_of(r)})) + ' (pooled as one run, 6.77)'
    return r['setup_group']


def related_runs(meta):
    """{run: set of runs sharing lineage}: identical early versions or frozen references (index 'lineage')."""
    rel = {}
    for r in meta.values():
        a = run_of(r)
        for m in re.finditer(r'identical with ([\w.+-]+)', r['lineage']):
            b = POOL_RUNS.get(m.group(1), m.group(1))
            rel.setdefault(a, set()).add(b)
            rel.setdefault(b, set()).add(a)
        for m in re.finditer(r'frozen reference ([\w.+-]+):', r['lineage']):
            b = POOL_RUNS.get(m.group(1), m.group(1))
            rel.setdefault(a, set()).add(b)
            rel.setdefault(b, set()).add(a)
    return rel


def load(ply):
    A = f'{REPO}/leagues/arenas/diag/{name(ply)}'
    idx = {r['id']: r for r in L.rows()}
    meta, ver = {}, {}
    for e in json.load(open(f'{A}/config.json'))['entries']:
        n = e['name']
        rid = re.match(r'^(.*?)(\d+)\.L\d+@(\d+)$', n)
        i, v = rid.group(1), int(rid.group(3))
        meta[n] = idx[i]
        ver[n] = v
    games = []
    for g in csv.DictReader(open(f'{A}/tour/games.csv')):
        w, b = g['white'], g['black']
        if w.startswith('random') or b.startswith('random'):
            continue
        w, b = w.rsplit('-', 1)[0], b.rsplit('-', 1)[0]
        s = {'1-0': 1.0, '0-1': 0.0, '1/2-1/2': 0.5}[g['result']]
        games.append((w, b, s))
    return A, meta, ver, games


def pair_class(a, b, meta, rel):
    ra, rb = run_of(meta[a]), run_of(meta[b])
    if meta[a]['id'] == meta[b]['id']:
        return 'same-line'
    if ra == rb:
        return 'same-run'
    if rb in rel.get(ra, ()):
        return 'lineage'
    return 'unrelated'


def bt_fit(players, cells, iters=2000):
    """Bradley-Terry by MM (Hunter 2004) on per-pair (games, score) cells, draws as half points; Elo, mean zero.
    Returns ratings and the covariance of the ratings (Elo^2) from the observed information (sum-to-zero)."""
    idx = {p: k for k, p in enumerate(players)}
    n = len(players)
    W = [0.0] * n
    N = [[0.0] * n for _ in range(n)]
    for (a, b), (g, s) in cells.items():
        i, j = idx[a], idx[b]
        W[i] += s
        W[j] += g - s
        N[i][j] += g
        N[j][i] += g
    gam = [1.0] * n
    for _ in range(iters):
        new = []
        for i in range(n):
            den = sum(N[i][j] / (gam[i] + gam[j]) for j in range(n) if N[i][j])
            new.append(max(W[i], 1e-9) / den if den else gam[i])
        m = math.exp(sum(math.log(x) for x in new) / n)
        new = [x / m for x in new]
        if max(abs(math.log(x / y)) for x, y in zip(new, gam)) < 1e-10:
            gam = new
            break
        gam = new
    th = [math.log(x) for x in gam]
    # observed information in theta (natural log units): H_ij = -N_ij p_ij (1 - p_ij), H_ii = sum_j N_ij p (1-p)
    H = [[0.0] * n for _ in range(n)]
    for i in range(n):
        for j in range(n):
            if i != j and N[i][j]:
                p = 1 / (1 + math.exp(th[j] - th[i]))
                v = N[i][j] * p * (1 - p)
                H[i][j] -= v
                H[i][i] += v
    # sum-to-zero: invert H + (1/n) 11' and subtract (1/n) 11' (pseudo-inverse for a connected design)
    M = [[H[i][j] + 1.0 / n for j in range(n)] for i in range(n)]
    C = invert(M)
    k = (400 / math.log(10)) ** 2
    cov = [[(C[i][j] - 1.0 / n) * k for j in range(n)] for i in range(n)]
    elo_ = [t * 400 / math.log(10) for t in th]
    mu = sum(elo_) / n
    return {p: elo_[idx[p]] - mu for p in players}, {(a, b): cov[idx[a]][idx[b]] for a in players for b in players}


def invert(M):
    n = len(M)
    A = [row[:] + [1.0 if i == j else 0.0 for j in range(n)] for i, row in enumerate(M)]
    for c in range(n):
        p = max(range(c, n), key=lambda r: abs(A[r][c]))
        A[c], A[p] = A[p], A[c]
        piv = A[c][c]
        A[c] = [x / piv for x in A[c]]
        for r in range(n):
            if r != c and A[r][c]:
                f = A[r][c]
                A[r] = [x - f * y for x, y in zip(A[r], A[c])]
    return [row[n:] for row in A]


def var_of(cov, coefs):
    return sum(ca * cb * cov[(a, b)] for a, ca in coefs.items() for b, cb in coefs.items())


def components(data, noise, sv_ind=None, sv_sh=None, vfac=0.5):
    """Method-of-moments nested components, measurement error subtracted.
    data = {setup: {run: {line: (m, d)}}}: m = the line's value (the pair mean of v299/v279, or one version such as
    the final), d = its version-pair difference against the panel (None without a pair);
    noise = {setup: {run: {line: (var_m, var_d)}}}.
    Version: from the d's, split into the part independent per line (pooled within-run variance of d) and the part
    shared by a run's lines (the field drifting together between the two versions): sv_ind, sv_sh. A value m carries
    vfac * sv_ind of independent version noise (0.5 for a pair mean, 1 for a single version); the shared part is
    part of the run term. Returns sigma^2 per level (negative estimates kept, read as 0)."""
    if sv_ind is None:
        num = den = 0.0
        vds, alld = [], []
        for s in data:
            for r in data[s]:
                ds = [(d, noise[s][r][l][1]) for l, (m, d) in data[s][r].items() if d is not None]
                alld += ds
                if len(ds) < 2:
                    continue
                mb = sum(d for d, _ in ds) / len(ds)
                num += sum((d - mb) ** 2 for d, _ in ds)
                den += len(ds) - 1
                vds += [v for _, v in ds]
        if alld:
            mu = sum(d for d, _ in alld) / len(alld)
            tot = (sum((d - mu) ** 2 for d, _ in alld) / (len(alld) - 1) - sum(v for _, v in alld) / len(alld)) / 2
            sv_ind = (num / den - sum(vds) / len(vds)) / 2 if den else tot
            sv_sh = tot - sv_ind
        else:
            sv_ind = sv_sh = float('nan')
    vi = max(sv_ind, 0) if not math.isnan(sv_ind) else 0.0
    # replicate line within run
    num = den = 0.0
    nz = []
    for s in data:
        for r in data[s]:
            ls = list(data[s][r].values())
            if len(ls) < 2:
                continue
            ms = [m for m, _ in ls]
            mb = sum(ms) / len(ms)
            num += sum((m - mb) ** 2 for m in ms)
            den += len(ms) - 1
            nz += [noise[s][r][l][0] for l in data[s][r]]
    sl2 = (num / den - sum(nz) / len(nz) - vfac * vi) if den else float('nan')
    lpart = (max(sl2, 0) if not math.isnan(sl2) else 0.0) + vfac * vi

    def run_mean_var(s, r):
        ls = data[s][r]
        k = len(ls)
        return (lpart + sum(noise[s][r][l][0] for l in ls) / k) / k
    # run within setup (seed term; setups with 2+ runs: few groups)
    num = den = 0.0
    corr = []
    for s in data:
        if len(data[s]) < 2:
            continue
        R = []
        for r in data[s]:
            ls = list(data[s][r].values())
            R.append(sum(m for m, _ in ls) / len(ls))
            corr.append(run_mean_var(s, r))
        rb = sum(R) / len(R)
        num += sum((x - rb) ** 2 for x in R)
        den += len(R) - 1
    sr2 = (num / den - sum(corr) / len(corr)) if den else float('nan')
    # between runs, all runs: run means about the grand mean, minus what lines, versions and noise put in them
    RM = [(sum(m for m, _ in data[s][r].values()) / len(data[s][r]), run_mean_var(s, r)) for s in data for r in data[s]]
    mu = sum(x for x, _ in RM) / len(RM)
    sall = sum((x - mu) ** 2 for x, _ in RM) / (len(RM) - 1) - sum(c for _, c in RM) / len(RM) if len(RM) > 1 \
        else float('nan')
    # setup (total): setup means (mean over the setup's runs), minus the lines', versions' and noise's share; the
    # seed term is NOT subtracted (one or two groups can't estimate it), so this is setup + seed / n_runs
    S, corr = [], []
    for s in data:
        rm = [sum(m for m, _ in ls.values()) / len(ls) for ls in data[s].values()]
        S.append(sum(rm) / len(rm))
        n = len(data[s])
        corr.append(sum(run_mean_var(s, r) for r in data[s]) / n ** 2)
    sb = sum(S) / len(S)
    ss2 = sum((x - sb) ** 2 for x in S) / (len(S) - 1) - sum(corr) / len(corr) if len(S) > 1 else float('nan')
    return dict(version=(sv_ind + sv_sh) if not math.isnan(sv_ind) else float('nan'), version_ind=sv_ind,
                version_shared=sv_sh, line=sl2, run=sr2, runs_all=sall, setup=ss2)


def corr_of(data, noise, comp, vfac, s, r):
    vi = max(comp['version_ind'], 0) if not math.isnan(comp['version_ind']) else 0.0
    lpart = (max(comp['line'], 0) if not math.isnan(comp['line']) else 0.0) + vfac * vi
    ls = data[s][r]
    k = len(ls)
    return (lpart + sum(noise[s][r][l][0] for l in ls) / k) / k


def sd(x):
    return math.sqrt(x) if x > 0 else 0.0


def cmd_analyze(args):
    A, meta, ver, games = load(args.ply)
    rel = related_runs(meta)
    players = sorted(meta)
    cls_cells = {}
    for w, b, s in games:
        a, bb, x = (w, b, s) if w < b else (b, w, 1 - s)
        c = cls_cells.setdefault((a, bb), [0, 0.0])
        c[0] += 1
        c[1] += x
    # BT over unrelated pairs only (6.77 (6): different-seed runs of one setup stay in)
    fitcells = {k: v for k, v in cls_cells.items() if pair_class(k[0], k[1], meta, rel) == 'unrelated'}
    flagged = {p for p in players if meta[p]['experiment'] == 'global'}
    R, COV = bt_fit(players, fitcells)
    se = {p: math.sqrt(max(COV[(p, p)], 0)) for p in players}
    out = [f'# {name(args.ply)}: {len(players)} entries, {sum(v[0] for v in cls_cells.values())} games '
           f'(random excluded); BT on {sum(v[0] for v in fitcells.values())} games between unrelated runs']
    # panel score of each entry (against every unrelated entry), for the version pairs (6.77 (5))
    def vs_panel(p):
        g = s = 0.0
        for (a, b), (n, x) in fitcells.items():
            if a == p:
                g += n; s += x
            elif b == p:
                g += n; s += n - x
        return g, s
    # design: setup -> run -> line -> versions
    design = {}
    for p in players:
        r = meta[p]
        if p in flagged:
            continue
        design.setdefault(setup_of(r), {}).setdefault(run_of(r), {}).setdefault(r['id'], {})[ver[p]] = p
    rows = []
    data, noise = {}, {}
    length = []
    for s, runs in design.items():
        for run, ls in runs.items():
            for lid, vs in ls.items():
                r = meta[next(iter(vs.values()))]
                f = int(r['version'])
                pair = (vs.get(299), vs.get(279)) if f >= 299 else (vs.get(f), vs.get(f - 20))
                a, b = pair
                m = (R[a] + R[b]) / 2
                vm = var_of(COV, {a: 0.5, b: 0.5})
                ga, sa = vs_panel(a)
                gb, sb_ = vs_panel(b)
                ea, eb = elo(sa / ga), elo(sb_ / gb)
                # binomial delta-method variance of each panel Elo (draws counted half: conservative)
                def evar(g, s_):
                    q = min(max(s_ / g, 1e-4), 1 - 1e-4)
                    return (400 / math.log(10)) ** 2 / (g * q * (1 - q))
                d = ea - eb
                vd = evar(ga, sa) + evar(gb, sb_)
                data.setdefault(s, {}).setdefault(run, {})[lid] = (m, d)
                noise.setdefault(s, {}).setdefault(run, {})[lid] = (vm, vd)
                fin = vs.get(f) if f > 299 else None
                if fin:
                    length.append((s, run, lid, f, R[fin] - R[a], math.sqrt(var_of(COV, {fin: 1, a: -1}))))
                rows.append((s, run, lid, r['line'], f, m, math.sqrt(vm), d, math.sqrt(vd),
                             'short' if f < 299 else '', r['field_class'], r['games_per_cycle'],
                             r['train_games_per_cycle']))
    pairs = {}
    for s_, runs in design.items():
        for run, ls in runs.items():
            for lid, vs in ls.items():
                f = int(meta[next(iter(vs.values()))]['version'])
                pairs[lid] = (vs.get(299), vs.get(279)) if f >= 299 else (vs.get(f), vs.get(f - 20))

    def boot_components(data, noise, nboot, seed=12, sv=None, vfac=0.5):
        if sv:
            comp = components(data, noise, sv[0], sv[1], vfac)
        else:
            comp = components(data, noise, vfac=vfac)
        rng = random.Random(seed)
        est = {k: max(v, 0) if not math.isnan(v) else 0 for k, v in comp.items()}
        boot = {k: [] for k in comp}
        for it in range(2 * nboot):
            # pass A (first nboot): set-up effects with the seed term folded in (version, line, setup intervals);
            # pass B: independent run effects (the between-runs interval)
            passB = it >= nboot
            bd = {}
            for s in data:
                us = 0.0 if passB else rng.gauss(0, sd(est['setup']))
                for run in data[s]:
                    ur = rng.gauss(0, sd(est['runs_all'])) if passB else 0.0
                    sh = rng.gauss(0, math.sqrt(2 * est['version_shared']))
                    for l in data[s][run]:
                        ul = rng.gauss(0, sd(est['line']))
                        vm, vd = noise[s][run][l]
                        v1, v2 = rng.gauss(0, sd(est['version_ind'])), rng.gauss(0, sd(est['version_ind']))
                        vpart = (v1 + v2) / 2 if vfac == 0.5 else v1
                        m = us + ur + ul + vpart + rng.gauss(0, math.sqrt(vm))
                        d = (sh + v1 - v2 + rng.gauss(0, math.sqrt(vd))) if data[s][run][l][1] is not None else None
                        bd.setdefault(s, {}).setdefault(run, {})[l] = (m, d)
            c = components(bd, noise, sv[0], sv[1], vfac) if sv else components(bd, noise, vfac=vfac)
            for k in c:
                if (k == 'runs_all') == passB:
                    boot[k].append(c[k])
        iv = {}
        for k in comp:
            xs = sorted(x for x in boot[k] if not math.isnan(x))
            if not xs or math.isnan(comp[k]):
                iv[k] = 'n/a'
                continue
            lo, hi = xs[int(0.025 * len(xs))], xs[int(0.975 * len(xs)) - 1]
            if comp[k] > 0 and lo > 0:
                # scale pivot (as a chi-square interval): estimate / truth is distributed as simulated / est
                a, b = comp[k] * est[k] / hi, comp[k] * est[k] / lo
            else:
                # basic interval (additive pivot) when the estimate or the simulated lower tail reaches 0
                a, b = comp[k] + est[k] - hi, comp[k] + est[k] - lo
            iv[k] = f'[{sd(a):.0f}, {sd(b):.0f}]'
        # the seed term from 1-3 replicate pairs: a chi-square interval on the pooled within-setup variance
        # (df = the pairs), minus what lines, versions and noise put in the run means
        ss = []
        for s in data:
            if len(data[s]) == 2:
                (r1, l1), (r2, l2) = list(data[s].items())
                m1 = sum(m for m, _ in l1.values()) / len(l1)
                m2 = sum(m for m, _ in l2.values()) / len(l2)
                ss.append(((m1 - m2) ** 2 / 2, (corr_of(data, noise, comp, vfac, s, r1) +
                                                corr_of(data, noise, comp, vfac, s, r2)) / 2))
        Q = {1: (0.000982, 5.0239), 2: (0.0506, 7.3778), 3: (0.2158, 9.3484), 4: (0.4844, 11.1433)}
        if ss and len(ss) in Q:
            df = len(ss)
            s2 = sum(x for x, _ in ss) / df
            c = sum(y for _, y in ss) / df
            q_lo, q_hi = Q[df]
            iv['run'] = f'[{sd(df * s2 / q_hi - c):.0f}, {sd(df * s2 / q_lo - c):.0f}] (chi-square, {df} df)'
        nver = sum(1 for s in data for r in data[s] for l in data[s][r] if data[s][r][l][1] is not None)
        groups = dict(version=nver, version_ind=nver, version_shared=sum(1 for s in data for r in data[s]),
                      line=sum(1 for s in data for r in data[s] if len(data[s][r]) > 1),
                      run=sum(1 for s in data if len(data[s]) > 1), runs_all=sum(len(data[s]) for s in data),
                      setup=len(data))
        return comp, iv, groups

    def subset(keep):
        d = {s_: v for s_, v in data.items() if keep(s_)}
        return d, {s_: noise[s_] for s_ in d}

    comp, iv, ngroups = boot_components(data, noise, args.boot)
    SV = (comp['version_ind'], comp['version_shared'])
    out.append('\n## Components at EQUAL LENGTH (line value = mean of v299 and v279; SD, Elo; method of moments, '
               'measurement error subtracted; 95% parametric bootstrap: scale pivot, basic near 0; seed term: chi-square)')
    out.append('level | SD | 95% interval | groups')
    labels = {'version': 'version (v299 vs v279), total', 'version_ind': '  of which independent per line',
              'version_shared': '  of which shared by the run (field drift)', 'line': 'replicate line within run',
              'run': 'run within setup (seed replicates)', 'runs_all': 'between runs, all set-ups',
              'setup': 'between set-ups (seed term included)'}
    for k in ('version', 'version_ind', 'version_shared', 'line', 'run', 'runs_all', 'setup'):
        out.append(f"{labels[k]} | {sd(comp[k]):.0f} (var {comp[k]:.0f}) | {iv[k]} | {ngroups[k]}")
    # the same at the finals: one version per line, the plateau the run stopped on (lengths differ)
    dataF, noiseF = {}, {}
    for s_, runs in design.items():
        for run, ls in runs.items():
            for lid, vs in ls.items():
                f = int(meta[next(iter(vs.values()))]['version'])
                x = vs[f]
                dataF.setdefault(s_, {}).setdefault(run, {})[lid] = (R[x], None)
                noiseF.setdefault(s_, {}).setdefault(run, {})[lid] = (COV[(x, x)], None)
    compF, ivF, gF = boot_components(dataF, noiseF, args.boot, seed=13, sv=SV, vfac=1)
    out.append('\n## Components at the FINALS (each line at the version its run stopped on; lengths differ; the '
               'version term taken from the pairs above)')
    out.append('level | SD | 95% interval | groups')
    for k in ('line', 'run', 'runs_all', 'setup'):
        out.append(f"{labels[k]} | {sd(compF[k]):.0f} (var {compF[k]:.0f}) | {ivF[k]} | {gF[k]}")
    setup_class = {}
    for p_ in players:
        if p_ not in flagged:
            setup_class[setup_of(meta[p_])] = (meta[p_]['field_class'], meta[p_]['experiment'])
    for label, keep in (('pure fields only', lambda s_: setup_class[s_][0] == 'pure'),
                        ('mixed fields only', lambda s_: setup_class[s_][0] == 'mixed'),
                        ('without leaf-sources-new (11-10 system)', lambda s_: 'leaf-sources' not in s_),
                        ('mixed, without leaf-sources-new', lambda s_: setup_class[s_][0] == 'mixed'
                         and 'leaf-sources' not in s_)):
        d, nz = subset(keep)
        if len(d) < 2:
            continue
        c, i_, g = boot_components(d, nz, args.boot, sv=SV)
        dF = {s_: dataF[s_] for s_ in d}
        cF, iF, _ = boot_components(dF, {s_: noiseF[s_] for s_ in d}, args.boot, seed=13, sv=SV, vfac=1)
        out.append(f"setup SD, {label}: equal length {sd(c['setup']):.0f} {i_['setup']}, finals "
                   f"{sd(cF['setup']):.0f} {iF['setup']} ({g['setup']} setups)")

    # contrasts with SEs from the BT covariance: run / setup means of the lines' pair means
    def mean_coefs(runs_):
        lids = [l for s_ in data for r_ in data[s_] if r_ in runs_ for l in data[s_][r_]]
        co = {}
        for l in lids:
            for x in pairs[l]:
                co[x] = co.get(x, 0) + 0.5 / len(lids)
        return co, len(lids)

    def contrast(runs_a, runs_b):
        ca, na = mean_coefs(runs_a)
        cb, nb = mean_coefs(runs_b)
        co = dict(ca)
        for k, v in cb.items():
            co[k] = co.get(k, 0) - v
        est_ = sum(R[k] * v for k, v in co.items())
        return est_, math.sqrt(max(var_of(COV, co), 0)), na, nb

    out.append('\n## Contrasts (pair means at equal length; SE from the BT covariance: measurement only, not the '
               'line/run terms)')
    allruns = {r_ for s_ in data for r_ in data[s_]}
    cmp_ = []
    for s_ in data:
        if len(data[s_]) > 1:
            rs_ = sorted(data[s_])
            cmp_.append((f'run within setup: {rs_[0]} minus {rs_[1]}', [rs_[0]], [rs_[1]]))
    if args.ply == 2:
        mixed_runs = sorted(r_ for s_ in data for r_ in data[s_] if setup_class[s_][0] == 'mixed')
        pure_runs = sorted(r_ for s_ in data for r_ in data[s_] if setup_class[s_][0] == 'pure')
        mixed_core = [r_ for r_ in mixed_runs if r_ not in ('replay-p2', 'leaf-sources-new')]
        cmp_ += [('games per cycle: fponly-p2-720 minus fponly-p2 + fponly-p2b', ['fponly-p2-720'],
                  ['fponly-p2', 'fponly-p2b']),
                 ('pure minus mixed (lines pooled; mixed without replay-p2 and leaf-sources-new)', pure_runs,
                  mixed_core),
                 ('own-past replay: replay-p2 minus probit-late-p2 (its seed twin)', ['replay-p2'], ['probit-late-p2']),
                 ('own-past replay: replay-p2 minus the other mixed runs', ['replay-p2'],
                  [r_ for r_ in mixed_core]),
                 ('frozen references: diversity-p2 minus the other mixed runs', ['diversity-p2'],
                  [r_ for r_ in mixed_core if not r_.startswith('diversity')]),
                 ('frozen references: diversity-p2c+d minus base-probit-p2 (their references\' run)',
                  ['diversity-p2c'], ['base-probit-p2']),
                 ('short runs: probit-confirm-p2 + probit-trees-p2 (finals 241, 276) minus the other mixed runs',
                  ['probit-confirm-p2', 'probit-trees-p2'],
                  [r_ for r_ in mixed_core if r_ not in ('probit-confirm-p2', 'probit-trees-p2')])]
    for label, a_, b_ in cmp_:
        a_ = [x for x in a_ if x in allruns]
        b_ = [x for x in b_ if x in allruns]
        if not a_ or not b_:
            continue
        e_, se_, na, nb = contrast(a_, b_)
        out.append(f'{label}: {e_:.0f} +-{se_:.0f} ({na} vs {nb} lines)')
    # setups: means, classes
    out.append('\n## Setups (mean of line pair means, Elo vs field mean 0; runs; covariates)')
    smeans = []
    for s, runs in data.items():
        rm = {run: sum(m for m, _ in ls.values()) / len(ls) for run, ls in runs.items()}
        rmF = {run: sum(m for m, _ in ls.values()) / len(ls) for run, ls in dataF[s].items()}
        mean = sum(rm.values()) / len(rm)
        r0 = meta[next(p for p in players if p not in flagged and setup_of(meta[p]) == s)]
        smeans.append((mean, s, rm, r0, sum(rmF.values()) / len(rmF)))
    out.append('equal length | finals | set-up | class, covariates | runs (equal length)')
    for mean, s, rm, r0, meanF in sorted(smeans, reverse=True):
        extra = []
        if r0['replay']:
            extra.append('replay ' + r0['replay'])
        if r0['frozen_references']:
            extra.append('frozen refs')
        if r0['reference_window']:
            extra.append('ref window')
        out.append(f"{mean:7.0f} {meanF:7.0f}  {s[:60]:60s} {r0['field_class']:5s} gpc={r0['games_per_cycle'] or '-':>5} "
                   f"train_games={r0['train_games_per_cycle']:>5} floor={r0['floor']} final~{r0['version']} "
                   + ' '.join(f'{k}:{v:.0f}' for k, v in rm.items()) + ('  ' + '; '.join(extra) if extra else ''))
    pure = [m for m, s, rm, r0, _ in smeans if r0['field_class'] == 'pure']
    mixed = [m for m, s, rm, r0, _ in smeans if r0['field_class'] == 'mixed']
    if pure and mixed:
        mp, mm = sum(pure) / len(pure), sum(mixed) / len(mixed)

        def svar(xs):
            mu = sum(xs) / len(xs)
            return sum((x - mu) ** 2 for x in xs) / (len(xs) - 1) if len(xs) > 1 else 0.0
        se_ = math.sqrt(svar(pure) / len(pure) + svar(mixed) / len(mixed)) if len(pure) > 1 and len(mixed) > 1 \
            else float('nan')
        out.append(f'pure minus mixed: {mp - mm:.0f} (SE {se_:.0f}; {len(pure)} pure, {len(mixed)} mixed setups); '
                   f'residual SD within class: pure {math.sqrt(svar(pure)):.0f}, mixed {math.sqrt(svar(mixed)):.0f} '
                   f'(raw, noise not subtracted)')
    # length effect
    if length:
        out.append('\n## Training length: final minus v299 (BT, Elo)')
        per = {}
        for s, run, lid, f, dlt, e in length:
            per.setdefault((run, f), []).append((dlt, e))
        for (run, f), xs in sorted(per.items()):
            out.append(f"{run:22s} final v{f}: " + ', '.join(f'{d:.0f} +-{e:.0f}' for d, e in xs))
    # matchups: observed head-to-head cells against the BT prediction, by pair class
    out.append('\n## Matchups: head-to-head cells against the BT prediction (pairs of lines, versions pooled)')
    agg = {}
    for (a, b), (n, x) in cls_cells.items():
        if a in flagged or b in flagged:
            continue
        c = pair_class(a, b, meta, rel)
        la, lb = meta[a]['id'], meta[b]['id']
        if la == lb:
            continue
        key = (c,) + tuple(sorted((la, lb)))
        sgn = 1 if la < lb else -1
        e = agg.setdefault(key, [0, 0.0, 0.0])
        q = 1 / (1 + 10 ** ((R[b] - R[a]) / 400))
        if sgn == 1:
            e[0] += n; e[1] += x; e[2] += n * q
        else:
            e[0] += n; e[1] += n - x; e[2] += n * (1 - q)
    for c in ('same-run', 'lineage', 'unrelated'):
        res = []
        for key, (n, s_, ex) in agg.items():
            if key[0] != c or n < 20:
                continue
            o, p = s_ / n, ex / n
            r_ = elo(o) - elo(p)
            q = min(max(p, 1e-3), 1 - 1e-3)
            nv = (400 / math.log(10)) ** 2 / (n * q * (1 - q))
            res.append((r_, nv, n))
        if not res:
            continue
        v = sum(r * r for r, _, _ in res) / len(res) - sum(nv for _, nv, _ in res) / len(res)
        mean = sum(r for r, _, _ in res) / len(res)
        out.append(f'{c:9s}: {len(res)} line pairs, mean residual {mean:.0f}, SD beyond noise {sd(v):.0f} '
                   f'(raw RMS {math.sqrt(sum(r * r for r, _, _ in res) / len(res)):.0f}, '
                   f'mean games {sum(n for _, _, n in res) / len(res):.0f})')
    # flagged entries
    for p in sorted(flagged):
        out.append(f'flagged {p}: {R[p]:.0f} +-{se[p]:.0f} (global league, not pooled)')
    # per line table
    out.append('\n## Lines: pair mean (BT) and version-pair difference (panel)')
    for s, run, lid, line, f, m, sm, d, sdd, fl, fc, gpc, tg in sorted(rows, key=lambda x: -x[5]):
        out.append(f'{m:7.0f} +-{sm:3.0f}  d {d:5.0f} +-{sdd:3.0f}  {run:20s} {line:12s} final v{f} {fl}')
    text = '\n'.join(out)
    print(text)
    open(f'{A}/cohort-read.txt', 'w').write(text + '\n')
    json.dump({'components': comp, 'groups': ngroups, 'ratings': R, 'se': se}, open(f'{A}/cohort-read.json', 'w'),
              indent=1)


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest='cmd', required=True)
    p = sub.add_parser('spec'); p.add_argument('ply', type=int); p.add_argument('rounds', nargs='?', default='ci15')
    p = sub.add_parser('analyze'); p.add_argument('ply', type=int); p.add_argument('--boot', type=int, default=2000)
    args = ap.parse_args()
    {'spec': cmd_spec, 'analyze': cmd_analyze}[args.cmd](args)


if __name__ == '__main__':
    main()
