"""Group contrasts from a Bradley-Terry (Elo) fit among the arena's model entries from their mutual games only (11-11 ~00:55): every entry
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
rows = [('W3 forget 1 - flat-p3 finals (its start)', ['w3f1'], ['fp3']),
        ('W3 forget 8 - flat-p3 finals (its start)', ['w3'], ['fp3']),
        ('W3 forget 1 - W3 forget 8', ['w3f1'], ['w3'])]
for label, A, B in rows:
    out = []
    for pl in plies:
        a = sum((G.get((g, pl), []) for g in A), []); b = sum((G.get((g, pl), []) for g in B), [])
        if a and b:
            e, h = contrast(a, b); out.append(f'ply {pl} {e:6.1f} ±{h:4.1f}')
    print(f'{label:50s} ' + '   '.join(out))
for pl in plies:
    base = sum(x[p] for p in G[('fp3', pl)]) / len(G[('fp3', pl)])
    print(f'ply {pl} minus the flat-p3 mean: ' + ', '.join(f"{p.rsplit('-', 1)[0]} {x[p] - base:.0f}" for p in sorted(players, key=lambda q: -x[q]) if grp(p)[1] == pl))
