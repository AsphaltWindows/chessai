"""Pre-screen read (theory-2 6.23 item 5), from rev_btg.py. Each family's champion minus the flat LR and flat probit of its vintage block. Group contrasts from a Bradley-Terry (Elo) fit among the arena's model entries from their mutual games only (11-11 ~00:55): every entry
beat every random player, so the tournament's anchored fit leaves them unrated. Draws count half. Ratings relative to
the first entry; 95% intervals of each contrast from the inverse information. Pure Python."""
import csv, math, sys
from collections import defaultdict
tour = sys.argv[1]
ref = sys.argv[2] if len(sys.argv) > 2 else None
score = defaultdict(float); n = defaultdict(int)
for r in csv.DictReader(open(f'{tour}/games.csv')):
    w, b, res = r['white'], r['black'], r['result']
    if w.startswith('random') or b.startswith('random') or w == b:
        continue
    s = {'1-0': 1.0, '0-1': 0.0, '1/2-1/2': 0.5}[res]
    key = tuple(sorted((w, b)))
    n[key] += 1
    score[key] += s if w == key[0] else 1 - s
players = sorted({p for k in n for p in k})
ref = ref or players[0]
free = [p for p in players if p != ref]
x = {p: 0.0 for p in players}
k = math.log(10) / 400
for it in range(100):
    g = {p: 0.0 for p in free}; H = {(p, q): 0.0 for p in free for q in free}
    for (a, b), m in n.items():
        e = 1 / (1 + math.exp(-k * (x[a] - x[b])))
        resid = score[(a, b)] - m * e
        w = m * e * (1 - e) * k * k
        if a in g: g[a] += k * resid; H[(a, a)] += w
        if b in g: g[b] -= k * resid; H[(b, b)] += w
        if a in g and b in g: H[(a, b)] -= w; H[(b, a)] -= w
    # solve H d = g (Gauss-Jordan)
    F = len(free); M = [[H[(p, q)] for q in free] + [g[p]] for p in free]
    for i in range(F):
        piv = M[i][i]
        M[i] = [v / piv for v in M[i]]
        for j in range(F):
            if j != i:
                f = M[j][i]; M[j] = [vj - f * vi for vj, vi in zip(M[j], M[i])]
    d = [M[i][F] for i in range(F)]
    for p, dp in zip(free, d): x[p] += dp
    if max(abs(v) for v in d) < 1e-9: break
# covariance = H^-1
F = len(free); M = [[H[(p, q)] for q in free] + [1.0 if i == j else 0.0 for j in range(F)] for i, p in enumerate(free)]
for i in range(F):
    piv = M[i][i]; M[i] = [v / piv for v in M[i]]
    for j in range(F):
        if j != i:
            f = M[j][i]; M[j] = [vj - f * vi for vj, vi in zip(M[j], M[i])]
C = {(free[i], free[j]): M[i][F + j] for i in range(F) for j in range(F)}

def contrast(A, B):
    """mean(A) - mean(B) and its 95% half-width from the inverse information (the reference has fixed 0)."""
    c = {p: 0.0 for p in players}
    for p in A: c[p] += 1 / len(A)
    for p in B: c[p] -= 1 / len(B)
    est = sum(c[p] * x[p] for p in players)
    v = sum(c[p] * c[q] * C.get((p, q), 0.0) for p in free for q in free)
    return est, 1.96 * math.sqrt(max(v, 0.0))
import re
def grp(p):
    pre = p.split('.')[0]
    return pre, int(p.rsplit('-', 1)[1])
G = {}
for p in players:
    G.setdefault(grp(p), []).append(p)
plies = sorted({k[1] for k in G})
print(f'games among entries: {sum(n.values())}')


def split(p):
    name, ply = p.rsplit('-', 1)
    return name, int(ply)
names = sorted({split(p)[0] for p in players})
refs = {'old': ('old.F-L-c@4827', 'old.F-P-tau0-c@4359'), 'ce': ('ce.F-L@317', 'ce.F-P@236'), 'ssp': ('ssp.F-L@150', 'ssp.F-P@942')}
print(f"{'entry':34s} " + '   '.join(f"ply {pl}: - flat LR      - flat probit" for pl in plies))
for block, (lr, pb) in refs.items():
    if f"{lr}-{plies[0]}" not in x:
        continue
    print(f"-- {block}")
    for nm in names:
        if not nm.startswith(block + '.') or nm in (lr, pb):
            continue
        cells = []
        for pl in plies:
            a = f"{nm}-{pl}"
            e1, h1 = contrast([a], [f"{lr}-{pl}"]); e2, h2 = contrast([a], [f"{pb}-{pl}"])
            cells.append(f"ply {pl}: {e1:7.1f} ±{h1:4.1f}  {e2:7.1f} ±{h2:4.1f}")
        print(f"{nm:34s} " + '   '.join(cells))
    cells = []
    for pl in plies:
        e, h = contrast([f"{pb}-{pl}"], [f"{lr}-{pl}"]); cells.append(f"ply {pl}: {e:7.1f} ±{h:4.1f}")
    print(f"{'(flat probit - flat LR)':34s} " + '   '.join(cells))
print('-- flat references across vintages (LR minus the champion run\'s flat LR)')
for pl in plies:
    out = []
    for nm in ('ce.F-L@317', 'ssp.F-L@150', 'ce.F-P@236', 'ssp.F-P@942', 'old.F-P-tau0-c@4359'):
        if f"{nm}-{pl}" not in x:
            continue
        e, h = contrast([f"{nm}-{pl}"], [f"old.F-L-c@4827-{pl}"]); out.append(f"{nm} {e:.0f} ±{h:.0f}")
    print(f"ply {pl}: " + ', '.join(out))
for pl in plies:
    base = x[f"old.F-L-c@4827-{pl}"]
    print(f"ply {pl} minus old.F-L-c: " + ', '.join(f"{split(p)[0]} {x[p] - base:.0f}" for p in sorted(players, key=lambda q: -x[q]) if split(p)[1] == pl))
