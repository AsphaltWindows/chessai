#!/usr/bin/env python3
"""treeview: a node-by-node report of a tree (BDT) or DAG (BDD) model: its structure, where positions go, what each
router looks at, what each leaf has learned and how well it predicts (2026-11-11; generalises
models/from-scratch/leafview of 11-09, which was written for two LR-leaf error trees of the champion run).

    python3 leagues/treeview.py MODEL [--out FILE] [--games DIR ...] [--games-from LEAGUE] [--positions N]
                                      [--flat MODEL|none] [--history N] [--text] [--quiet]

MODEL, one of:
  EXPERIMENT:LINE@VERSION   a line of leagues/experiments/EXPERIMENT at a version (without @VERSION: its current one).
                            LINE may be shortened to its recipe, the options that tell it apart and its replicate,
                            e.g. error-dag-p1:D-L-Ne-ctx+router-split-z=160#1@170
  LINE@VERSION, LINE        a line of the global league (leagues/global), e.g. T-P-Nc-merge-fresh.p1@40
  bdt:PATH, bdd:PATH        a model file (outside a league: give --games)

Games: by default the league's latest finished rounds of games (an experiment's pool/, the global league's batch/),
newest first, until --positions positions (default 200000). Each round is hard-linked under
models/from-scratch/treeview-cache/games/ before it is read, so a pool that rotates can't take it away, and the
snapshots of earlier reports are used too; snapshots under two hours old that hold enough positions are reused as
they are, so the cache takes one set of rounds per league every two hours at most (old snapshots can be deleted at
any time). A finished experiment keeps no games: then the running experiment with the same leaf type and ply whose
games are newest (or --games-from LEAGUE, a directory or an experiment's name).
--games DIR...: round directories of game directories (one game each, as tournament --save-games writes) or game
directories.
--flat MODEL: the flat model the leaves are compared with (as MODEL, or lr:PATH, probit:PATH); by default the
league's first F-L or F-P line (as the leaves' type) at the same version, or its latest; none: no comparison.
--history N: at most N versions of the line are read (evenly spaced, always the last) to date each node and trace
its error windows (default 400). --text: also print a text summary. --out FILE: default
models/from-scratch/treeview/EXPERIMENT_SHORTLINE@VERSION.html (SHORTLINE: the line without the options its recipe's
lines share, its replicate as -rN); one self-contained page (opens from disk; light and dark).

What the page shows: the plain-words recipe, version, field and date; a diagram of the graph (share of positions,
node ids, since which version each node has its role, router splits' sibling sets, shared children); per router its
children's positions in chess terms, the features and values that tell them apart (from the positions it routed and
from its own naive-Bayes counts), piece-square heat maps, routing margins and window error history (for a DAG's
routers, how near each window came to a router split, in SEs estimated from the line's latest .state); per leaf its
positions, log-loss against the flat model on the same positions, weight boards (LR: white-win minus black-win
log-odds against an empty square; probit: the latent's mean), effective piece values and similarity to the other
leaves; for a DAG with path context, the path tables' sizes and effects (path-stats), and what the path parameters
do (path-detail): a top line with the model's log-loss with all its path cells and with all of them neutral (every
router routing by its classifier alone, every leaf predicting by its own) and the difference's standard error
(paired over positions; clustered by game); per node with cells, one table per cell (an upstream sibling set) with a
row per arrival (through member j, or not through the set): its share of the node's positions, the cell's terms
(LR leaf: weights per result and white minus black; probit leaf: the latent weight's mean and SD and the mean change
it makes to the white-minus-black log-odds; router: the centred shift to each child's log routing score and the
share of positions it routes elsewhere without the cell); the node's eight commonest paths from the root with their
total offset and effect, the rest lumped; and the payoff, the model's log-loss and winner log-loss on the node's
positions with its cells and with them neutral (a router: the from-to matrix of its neutral choice against its
choice with cells, and the log-loss of the positions it moves). "Neutral" is a node's classifier alone.

Exact routes and predictions come from models/from-scratch/bin/chessai-treeview (node-dump, node-history,
path-stats; source models/from-scratch/fastbin/treeview-src: chessai-errdag3 plus the dumps), which loads every tree
and DAG the leagues make: bdt and bdd; LR, probit and CNBP leaves; cnbp, lr and tan routers; path context. The path
parameters come from models/from-scratch/bin/chessai-treeview2 path-detail (source fastbin/treeview2-src: treeview-src
plus path-detail; its predictions with cells are bit for bit chessai-errdag4's, its node-dump and path-stats
chessai-treeview's, checked 2026-11-11). TOOL_VERSION (4 since observers, 2026-11-12: the field without them; 3 since path-detail; 2 was its first number format) keys the dashboard's cache of pages. The routes
of every position are streamed from node-dump and summed as they come (nothing per position is kept); the cache
(models/from-scratch/treeview-cache) holds the game snapshots, slots/ (per model directory: its JSON and game list,
rewritten by each report) and hist/ (each line's versions, read once). Pure Python otherwise. Read-only on the
leagues: games are linked into the cache, never moved.
"""
import argparse
import collections
import csv
import fcntl
import hashlib
import html
import json
import math
import os
import re
import shutil
import struct
import subprocess
import sys
import time

REPO = "/home/iv/dev/chessai"
LEAGUES = os.path.join(REPO, "leagues")
BIN = os.path.join(REPO, "models/from-scratch/bin/chessai-treeview")
BIN2 = os.path.join(REPO, "models/from-scratch/bin/chessai-treeview2")  # path-detail (DAG pages)
# The pages' version: the dashboard keys its cache of nodes pages by the model file's md5 and this (read from this
# file at each request), so raise it whenever a page changes.
TOOL_VERSION = "4"
CACHE = os.path.join(REPO, "models/from-scratch/treeview-cache")
OUTDIR = os.path.join(REPO, "models/from-scratch/treeview")

NF = 70
CATS = [13] * 64 + [2, 9, 2, 2, 2, 2]
OFF = [sum(CATS[:i]) for i in range(NF)]
TOT = sum(CATS)
WIDTH = 1 + TOT
DIGITS = "0123456789abcdefghijklmnopqrstuvwxyz"
IDX = [{DIGITS[v]: OFF[i] + v for v in range(CATS[i])} for i in range(NF)]
FEAT_OF = [f for f in range(NF) for _ in range(CATS[f])]
PCH = "123456789abc"  # board values 1..12 as base-36 digits
PIECES = "PNBRQKpnbrqk"
PNAME = ["white pawn", "white knight", "white bishop", "white rook", "white queen", "white king",
         "black pawn", "black knight", "black bishop", "black rook", "black queen", "black king"]
GLYPH = "♙♘♗♖♕♔♟♞♝♜♛♚"
CLASSES = ["white wins", "black wins", "draw"]
QUIET = False


def log(*a):
    if not QUIET:
        print(*a, file=sys.stderr, flush=True)


def sq(s):
    return "abcdefgh"[s % 8] + str(s // 8 + 1)


def featname(f):
    if f < 64:
        return sq(f)
    return {64: "side to move", 65: "en passant", 66: "white O-O-O right", 67: "black O-O-O right",
            68: "white O-O right", 69: "black O-O right"}[f]


def valname(f, v):
    if f < 64:
        return f"{sq(f)} empty" if v == 0 else f"{PIECES[v - 1]}{sq(f)}"
    if f == 64:
        return "white to move" if v == 0 else "black to move"
    if f == 65:
        return "no en passant" if v == 0 else f"en passant {'abcdefgh'[v - 1]}-file"
    return f"{featname(f)} {'kept' if v else 'gone'}"


def read_csv(path):
    with open(path) as f:
        return list(csv.DictReader(f))


def run(cmd):
    r = subprocess.run(["nice", "-n", "15"] + cmd, capture_output=True, text=True)
    if r.returncode:
        sys.exit(f"{os.path.basename(cmd[0])} {cmd[1]} failed: {r.stderr.strip()[-600:]}")
    return r.stdout


# ---------------------------------------------------------------------------------------------- leagues and models
MODEL_RE = re.compile(r"(bdd|bdt|lr|probit|cnbp|tan|fm|ht)(\d+)\.model")


def versions_of(runs_line):
    out = {}
    for fn in os.listdir(runs_line):
        m = MODEL_RE.fullmatch(fn)
        if m:
            out[int(m.group(2))] = (m.group(1), os.path.join(runs_line, fn))
    return out


def find_line(runs_dir, q):
    lines = sorted(d for d in os.listdir(runs_dir) if os.path.isdir(os.path.join(runs_dir, d)) and not d.startswith("."))
    for cand in (q, q + ".loc"):
        if cand in lines:
            return cand

    def parts(s):
        s = s[:-4] if s.endswith(".loc") else s
        rep = None
        if "#" in s:
            s, rep = s.rsplit("#", 1)
        toks = s.split("+")
        return toks[0], set(toks[1:]), rep

    qb, qo, qr = parts(q)
    hits = [l for l in lines if parts(l)[0] == qb and qo <= parts(l)[1] and qr in (None, parts(l)[2])]
    if len(hits) == 1:
        return hits[0]
    sys.exit(f"{q!r} {'matches several lines' if hits else 'matches no line'} in {runs_dir}:\n  "
             + "\n  ".join(hits or lines))


def resolve(spec):
    """{prefix, path, league, line, version, kind ('experiment'/'global'/None), name}."""
    m = re.fullmatch(r"(bdt|bdd|lr|probit|cnbp):(.+)", spec)
    if m:
        path = os.path.abspath(m.group(2))
        mv = re.search(r"(\d+)\.model$", path)
        return dict(prefix=m.group(1), path=path, league=None, line=os.path.basename(os.path.dirname(path)),
                    version=int(mv.group(1)) if mv else None, kind=None, name=None)
    if ":" in spec:
        name, rest = spec.split(":", 1)
        league = os.path.abspath(name) if os.sep in name else os.path.join(LEAGUES, "experiments", name)
        kind, name = "experiment", os.path.basename(league)
        try:
            if json.load(open(os.path.join(league, "config.json"))).get("kind") == "global":
                kind, name = "global", "global"
        except (OSError, ValueError):
            pass
    else:
        league, kind, name, rest = os.path.join(LEAGUES, "global"), "global", "global", spec
    if not os.path.isdir(os.path.join(league, "runs")):
        sys.exit(f"{spec}: no league at {league}")
    q, version = rest, None
    if "@" in rest:
        q, v = rest.rsplit("@", 1)
        version = int(v)
    line = find_line(os.path.join(league, "runs"), q)
    runs_line = os.path.join(league, "runs", line)
    vs = versions_of(runs_line)
    if not vs:
        sys.exit(f"{spec}: no models in {runs_line}")
    if version is None:
        lp = os.path.join(league, "lines.csv")
        rows = {r["line"]: r for r in read_csv(lp)} if os.path.exists(lp) else {}
        r = rows.get(line)
        version = int(r["version"]) if r and (r.get("version") or "").isdigit() and int(r["version"]) in vs else max(vs)
    if version not in vs:
        sys.exit(f"{spec}: no version {version} in {runs_line} (versions {min(vs)}-{max(vs)})")
    prefix, path = vs[version]
    return dict(prefix=prefix, path=path, league=league, line=line, version=version, kind=kind, name=name)


def league_info(m):
    """The league's config and the line's entry in it ({} outside a league)."""
    if not m["league"]:
        return {}, {}
    cfg = json.load(open(os.path.join(m["league"], "config.json")))
    return cfg, next((lc for lc in cfg.get("lines", []) if lc["line"] == m["line"]), {})


def train_args(lc):
    a = lc.get("train_args") or ""
    return " ".join(a) if isinstance(a, list) else a


def leaf_type(lc):
    args = train_args(lc)
    for t in ("probit", "lr", "cnbp"):
        if f"--leaf {t}" in args:
            return t
    return {"F-P": "probit", "F-L": "lr"}.get(lc.get("archetype"))


def default_flat(m, cfg, lc):
    """The league's first flat line of the leaves' type: at the model's version, or its latest."""
    want = {"probit": "F-P", "lr": "F-L"}.get(leaf_type(lc))
    if not want:
        return None
    for l in sorted(l["line"] for l in cfg.get("lines", []) if l.get("archetype") == want):
        runs_line = os.path.join(m["league"], "runs", l)
        vs = versions_of(runs_line) if os.path.isdir(runs_line) else {}
        if vs:
            v = m["version"] if m["version"] in vs else max(vs)
            return f"{vs[v][0]}:{vs[v][1]}", f"{l.removesuffix('.loc')}@{v}"
    return None


def resolve_flat(spec):
    m = re.fullmatch(r"(lr|probit|cnbp|bdt|bdd):(.+)", spec)
    if m:
        return f"{m.group(1)}:{os.path.abspath(m.group(2))}", os.path.basename(m.group(2))
    r = resolve(spec)
    return f"{r['prefix']}:{r['path']}", f"{r['line'].removesuffix('.loc')}@{r['version']}"


# ---------------------------------------------------------------------------------------------- games
def round_source(league):
    """The league's finished rounds of saved games, newest first: [(directory of rounds, round name)]. An
    experiment's pool/ holds the current cycle's rounds; the global league's batch/ holds the last cycle's, complete
    (published as a whole). A pool round counts when rounds.csv records it, or (a cycle's rows come at its end;
    tournament --save-games writes a cycle's rounds when it ends) when it has as many games as the last recorded
    round, each with its three files, and nothing was written for ten seconds."""
    rc = os.path.join(league, "rounds.csv")
    rows = [r for r in read_csv(rc) if (r.get("t") or "").isdigit()] if os.path.exists(rc) else []
    done = max((int(r["t"]) for r in rows), default=-1)
    full = int(rows[-1]["games"]) if rows and (rows[-1].get("games") or "").isdigit() else None
    out = {}
    for sub in ("batch", "pool"):
        d = os.path.join(league, sub)
        try:
            names = [r for r in os.listdir(d) if re.fullmatch(r"round\d+", r)]
        except FileNotFoundError:
            continue
        for r in names:
            t = int(r[5:])
            if t in out:
                continue
            if sub == "pool" and t > done:
                p = os.path.join(d, r)
                try:
                    gs = os.listdir(p)
                    ok = bool(full) and len(gs) >= full and time.time() - os.path.getmtime(p) > 10 and all(
                        os.path.exists(os.path.join(p, g, c + ".games"))
                        for g in gs for c in ("white_wins", "black_wins", "draws"))
                except FileNotFoundError:
                    ok = False
                if not ok:
                    continue
            out[t] = (d, r)
    return [out[t] for t in sorted(out, reverse=True)]


def game_positions(gdir):
    n = 0
    for c in ("white_wins", "black_wins", "draws"):
        p = os.path.join(gdir, c + ".games")
        if os.path.exists(p):
            with open(p, "rb") as f:
                n += f.read().count(b"\n")
    return n


def iter_snapshots(base):
    """A league's snapshotted rounds in the cache, newest first: (dir, positions)."""
    rows = []
    if os.path.isdir(base):
        for sub in os.listdir(base):
            for r in os.listdir(os.path.join(base, sub)):
                p = os.path.join(base, sub, r, ".complete")
                if re.fullmatch(r"round\d+", r) and os.path.exists(p):
                    rows.append((int(r[5:]), os.path.join(base, sub, r), int(open(p).read().split()[0])))
    seen, out = set(), []
    for t, d, n in sorted(rows, reverse=True):
        if t not in seen:
            seen.add(t)
            out.append((d, n))
    return out


def snapshot_rounds(league, target):
    """The league's newest finished rounds, hard-linked into the cache, then with the cache's earlier snapshots of
    the league, newest first, until `target` positions. Returns [(dir, positions)]."""
    base = os.path.join(CACHE, "games", os.path.basename(league))
    cached = iter_snapshots(base)
    out, total = [], 0
    for d, n in cached:  # recent snapshots that hold enough are reused: the cache grows at most once per FRESH
        if total >= target or time.time() - os.path.getmtime(os.path.join(d, ".complete")) > FRESH:
            break
        out.append((d, n))
        total += n
    if total >= target:
        return out
    rounds = round_source(league)
    stale = not cached or time.time() - os.path.getmtime(cached[0][0]) > 3600 or sum(n for _, n in cached) < target
    end = time.time() + WAIT
    while not rounds and stale and running(league) and time.time() < end:
        time.sleep(5)  # a running league's pool is away while its tournament plays
        rounds = round_source(league)
    got = 0
    for src, r in rounds:
        if got >= target:
            break
        dst = os.path.join(base, os.path.basename(src), r)
        done = os.path.join(dst, ".complete")
        if not os.path.exists(done):
            try:
                games = sorted(os.listdir(os.path.join(src, r)))
            except FileNotFoundError:
                continue
            ok = True
            for g in games:
                gs, gd = os.path.join(src, r, g), os.path.join(dst, g)
                if not os.path.isdir(gs):
                    continue
                os.makedirs(gd, exist_ok=True)
                for fn in ("white_wins.games", "black_wins.games", "draws.games"):
                    s, d = os.path.join(gs, fn), os.path.join(gd, fn)
                    if os.path.exists(d):
                        continue
                    try:
                        os.link(s, d)
                    except FileNotFoundError:
                        ok = False
                    except OSError:
                        shutil.copyfile(s, d)
            if not ok:
                log(f"{r} went away while being linked; skipped")
                continue
            n = sum(game_positions(os.path.join(dst, g)) for g in os.listdir(dst))
            with open(done, "w") as f:
                f.write(f"{n}\n")
        got += int(open(done).read().split()[0])
    out, total = [], 0
    for d, n in iter_snapshots(base):
        if total >= target:
            break
        out.append((d, n))
        total += n
    return out


def running(league):
    rl = os.path.join(league, "repeat.log")
    return os.path.exists(rl) and time.time() - os.path.getmtime(rl) < 600


def fallback_league(m, cfg, lc):
    """For a league without games: the experiment with the same leaf type and entry ply whose games are newest."""
    want, ply = leaf_type(lc), cfg.get("entry_ply")
    best = None
    edir = os.path.join(LEAGUES, "experiments")
    for e in sorted(os.listdir(edir)):
        d = os.path.join(edir, e)
        if d == m["league"] or not os.path.exists(os.path.join(d, "config.json")):
            continue
        rounds = round_source(d)
        cached = iter_snapshots(os.path.join(CACHE, "games", e))
        live = running(d)
        if not rounds and not cached and not live:
            continue
        try:
            c = json.load(open(os.path.join(d, "config.json")))
        except ValueError:
            continue
        if c.get("entry_ply") != ply or not any(leaf_type(l) == want for l in c.get("lines", [])):
            continue
        newest = [os.path.getmtime(os.path.join(*rounds[0]))] if rounds else []
        newest += [os.path.getmtime(cached[0][0])] if cached else []
        t = (live, max(newest, default=0))
        if not best or t > best[0]:
            best = (t, d)
    return best[1] if best else None


# ---------------------------------------------------------------------------------------------- classifiers
def parse_classifier(kind, text, k):
    t = (text or "").split()
    if kind == "Cnbp" and t:
        v = list(map(float, t))
        return dict(kind="cnbp", ct=v[:k], cc=[v[k + c * TOT:k + (c + 1) * TOT] for c in range(k)])
    if kind == "Lr" and t and t[0] == "logreg":
        kk, nf = int(t[1]), int(t[3])
        i = 4 + nf
        w = list(map(float, t[i:i + kk * WIDTH]))
        return dict(kind="lr", w=[w[c * WIDTH:(c + 1) * WIDTH] for c in range(kk)])
    if kind == "Probit" and t and t[0] == "probit":
        beta, eps, nc = float(t[3]), float(t[9]), int(t[10])
        v = list(map(float, t[11 + nc:]))
        return dict(kind="probit", beta=beta, eps=eps, mu=v[0::2], s2=v[1::2])
    return dict(kind=kind.lower())


def pst(cl):
    """A leaf's piece-square values, piece p on square s against s empty, in white's favour: LR, white-win minus
    black-win log-odds; probit, the latent's mean (positive favours white). None for other kinds."""
    out = [[0.0] * 64 for _ in range(12)]
    if cl["kind"] == "lr":
        w0, w1 = cl["w"][0], cl["w"][1]
        for s in range(64):
            e = 1 + OFF[s]
            for p in range(12):
                out[p][s] = (w0[e + 1 + p] - w0[e]) - (w1[e + 1 + p] - w1[e])
        return out
    if cl["kind"] == "probit":
        mu = cl["mu"]
        for s in range(64):
            e = 1 + OFF[s]
            for p in range(12):
                out[p][s] = mu[e + 1 + p] - mu[e]
        return out
    return None


def pst_draw(cl):
    """An LR leaf's draw log-odds against the mean of the two wins', piece p on square s against s empty."""
    if cl["kind"] != "lr":
        return None
    w0, w1, w2 = cl["w"][0], cl["w"][1], cl["w"][2]
    out = [[0.0] * 64 for _ in range(12)]
    for s in range(64):
        e = 1 + OFF[s]
        for p in range(12):
            out[p][s] = (w2[e + 1 + p] - w2[e]) - ((w0[e + 1 + p] - w0[e]) + (w1[e + 1 + p] - w1[e])) / 2
    return out


def nb_probs(cl, alpha):
    """A CNBP router's P(value | child) per feature value, and its child priors."""
    ct, cc = cl["ct"], cl["cc"]
    K, T = len(ct), sum(ct)
    pi = [(ct[k] + alpha) / (T + K * alpha) for k in range(K)]
    pv = [[0.0] * TOT for _ in range(K)]
    for k in range(K):
        for f in range(NF):
            den = ct[k] + CATS[f] * alpha
            for v in range(CATS[f]):
                pv[k][OFF[f] + v] = (cc[k][OFF[f] + v] + alpha) / den
    return pi, pv


def nb_mi(pi, pv):
    """Mutual information (nats) between each feature and the child, under a CNBP router's own counts."""
    K, mi = len(pi), []
    for f in range(NF):
        tot = 0.0
        for v in range(CATS[f]):
            j = OFF[f] + v
            m = sum(pi[k] * pv[k][j] for k in range(K))
            for k in range(K):
                tot += pi[k] * pv[k][j] * math.log(pv[k][j] / m)
        mi.append(tot)
    return mi


def nb_llr(pi, pv, k):
    """Per feature value: log P(value | child k) - log P(value | the router's other children), by its counts."""
    K = len(pi)
    rest = sum(pi[j] for j in range(K) if j != k)
    return [math.log(pv[k][j] / (sum(pi[i] * pv[i][j] for i in range(K) if i != k) / rest)) for j in range(TOT)]


def lr_llr(cl, k):
    """An LR router's weights for child k against the mean of its other children's, per feature value."""
    w = cl["w"]
    K = len(w)
    return [w[k][1 + j] - sum(w[i][1 + j] for i in range(K) if i != k) / (K - 1) for j in range(TOT)]


# ---------------------------------------------------------------------------------------------- the analysis
# Sums per route (path and leaf): positions, non-pawn material, pawns, material balance, |balance|, both queens, no
# queens, a castling right, white to move, white wins, black wins, draws, log-loss, winner log-loss (decisive
# positions), decisive positions, the flat model's log-loss and winner log-loss, men on the board, phase (non-pawn
# material >= 52, 26-51, <= 25).
(A_N, A_NPM, A_PAWNS, A_BAL, A_ABS, A_BQ, A_NQ, A_CAS, A_WTM, A_W, A_B, A_D, A_LL, A_WLL, A_DEC, A_FLL, A_FWLL,
 A_PCS, A_PH0, A_PH1, A_PH2) = range(21)
NA = 21


def add_into(dst, src):
    for i, x in enumerate(src):
        if x:
            dst[i] += x


def start_dump(spec, games, role, leaf_odds=True):
    """chessai-treeview node-dump of a model on games with its routes streamed (no per-position file is kept). Returns
    (process, slot directory). The slot (one per model directory and role) holds model.json, games.tsv and the dump's
    stderr, rewritten by every report there, so the cache stays bounded; it is locked while in use."""
    path = spec.split(":", 1)[1]
    slot = os.path.join(CACHE, "slots", hashlib.md5(f"{os.path.dirname(path)}\n{role}".encode()).hexdigest()[:16])
    os.makedirs(slot, exist_ok=True)
    lock = open(os.path.join(slot, ".lock"), "w")
    fcntl.flock(lock, fcntl.LOCK_EX)
    err = open(os.path.join(slot, "stderr.txt"), "w+")
    proc = subprocess.Popen(["nice", "-n", "15", BIN, "node-dump", spec, slot] + games + ["--use-probs", "--routes-stdout"]
                            + (["--leaf-odds"] if leaf_odds else []), stdout=subprocess.PIPE, stderr=err, text=True,
                            bufsize=1 << 16)
    proc.lock, proc.err = lock, err
    return proc, slot


def finish_dump(proc):
    """Drain what is left of the stream, wait, release the slot; exits if the dump failed."""
    while proc.stdout.read(1 << 20):
        pass
    rc = proc.wait()
    proc.err.seek(0)
    err = proc.err.read()
    proc.err.close()
    proc.lock.close()
    if rc:
        sys.exit(f"node-dump failed ({rc}): {err.strip()[-600:]}")


def file_key(path):
    st = os.stat(path)
    return hashlib.md5(f"{path}\n{st.st_size}\n{st.st_mtime_ns}".encode()).hexdigest()[:16]


def stream(rf, fr):
    """Sums per route key (router hops | leaf) over node-dump's streamed positions, plus per-key plies and feature
    counts, routing margins per router and a sample of leaf odds. fr is the flat model's stream on the same positions in
    the same order (or None); returns whether it matched throughout."""
    agg, plies, feat = {}, {}, {}
    margins = collections.defaultdict(list)
    odds = []
    N = 0
    fok = fr is not None
    ln = math.log
    for line in rf:
        parts = line.rstrip("\n").split("\t")
        ply, c, fv, path, leaf, probs = parts[1:7]
        c = int(c)
        hops = path.split(";") if path else []
        key = ";".join(h[:h.rindex(":")] for h in hops) + "|" + leaf
        a = agg.get(key)
        if a is None:
            a = agg[key] = [0.0] * NA
            plies[key] = collections.Counter()
            feat[key] = [0] * TOT
        for h in hops:
            n_, _, sc = h.split(":")
            s = sorted(map(float, sc.split("/")))
            margins[int(n_)].append(s[-1] - s[-2])
        board = fv[:64]
        cw = [board.count(ch) for ch in PCH]
        wm = cw[0] + 3 * (cw[1] + cw[2]) + 5 * cw[3] + 9 * cw[4]
        bm = cw[6] + 3 * (cw[7] + cw[8]) + 5 * cw[9] + 9 * cw[10]
        npm = wm + bm - cw[0] - cw[6]
        p = [max(float(x), 1e-12) for x in probs.split()]
        a[A_N] += 1
        a[A_NPM] += npm
        a[A_PAWNS] += cw[0] + cw[6]
        a[A_BAL] += wm - bm
        a[A_ABS] += abs(wm - bm)
        q = (cw[4] > 0) + (cw[10] > 0)
        a[A_BQ] += q == 2
        a[A_NQ] += q == 0
        a[A_CAS] += fv[66:70] != "0000"
        a[A_WTM] += fv[64] == "0"
        a[A_W + c] += 1
        a[A_LL] -= ln(p[c])
        a[A_PCS] += 64 - board.count("0")
        a[A_PH0 if npm >= 52 else (A_PH1 if npm >= 26 else A_PH2)] += 1
        if c < 2:
            a[A_DEC] += 1
            a[A_WLL] -= ln(p[c] / (p[0] + p[1]))
        if fok:
            fl = fr.readline().split("\t")
            if fl[:2] != parts[:2]:
                fok = False  # the flat model's positions are not these (never expected)
            else:
                fp = [max(float(x), 1e-12) for x in fl[6].split()]
                a[A_FLL] -= ln(fp[c])
                if c < 2:
                    a[A_FWLL] -= ln(fp[c] / (fp[0] + fp[1]))
        plies[key][int(ply)] += 1
        fc = feat[key]
        for i, ch in enumerate(fv):
            fc[IDX[i][ch]] += 1
        if len(parts) > 7 and N % 5 == 0:
            od = {}
            for item in parts[7].split(","):
                x, y = item.split(":")
                od[int(x)] = float(y)
            odds.append((int(leaf), od))
        N += 1
    if fok and fr.readline():
        fok = False
    return agg, plies, feat, margins, odds, N, fok


def analyse(m, games, flat):
    t0 = time.time()
    if m["prefix"] not in ("bdt", "bdd"):
        sys.exit(f"{m['path']}: not a tree or DAG")
    proc, work = start_dump(f"{m['prefix']}:{m['path']}", games, "model")
    fproc = start_dump(flat[0], games, "flat", leaf_odds=False)[0] if flat else None
    try:
        agg, plies, feat, margins, odds, N, fok = stream(proc.stdout, fproc.stdout if fproc else None)
    finally:
        finish_dump(proc)
        if fproc:
            finish_dump(fproc)
    model = json.load(open(os.path.join(work, "model.json")))
    games_tsv = [l.rstrip("\n").split("\t") for l in open(os.path.join(work, "games.tsv"))]
    if model.get("flat"):
        sys.exit(f"{m['path']}: not a tree or DAG")
    bf, nodes = model["branch_factor"], model["nodes"]
    for nd in nodes:
        nd["cl"] = parse_classifier(nd["model"], nd["text"], bf if nd["kind"] == "branch" else model["class_num"])
        nd["text"] = None
        nd["parents"] = []
    for nd in nodes:
        for i, c in enumerate(nd["children"]):
            nodes[c]["parents"].append((nd["id"], i))
    depth, order = {0: 0}, [0]
    for idx in order:  # longest path from the root: a DAG's shared node sits below all its parents
        for c in nodes[idx]["children"]:
            if depth.get(c, -1) < depth[idx] + 1:
                depth[c] = depth[idx] + 1
                order.append(c)
    if flat and not fok:
        log("flat comparison skipped: its positions differ")
    has_flat = bool(flat and fok)
    log(f"model: {len(nodes)} nodes, {sum(1 for n in nodes if n['kind'] == 'leaf')} leaves, "
        f"{len(model['sets'])} sibling sets; {N} positions of {len(games_tsv)} games; {time.time() - t0:.1f}s")
    # --- per node, per edge (router, child slot), per arrival
    node_a = collections.defaultdict(lambda: [0.0] * NA)
    node_ply = collections.defaultdict(collections.Counter)
    edge_a = collections.defaultdict(lambda: [0.0] * NA)
    edge_ply = collections.defaultdict(collections.Counter)
    edge_feat, leaf_feat = {}, {}
    arrivals = collections.defaultdict(collections.Counter)
    paths = collections.Counter()
    for key, a in agg.items():
        hp, leaf = key.split("|")
        hops = [tuple(map(int, h.split(":"))) for h in hp.split(";")] if hp else []
        leaf = int(leaf)
        paths[(tuple(hops), leaf)] += a[A_N]
        prev = None
        for n_, k_ in hops:
            add_into(node_a[n_], a)
            node_ply[n_].update(plies[key])
            add_into(edge_a[(n_, k_)], a)
            edge_ply[(n_, k_)].update(plies[key])
            add_into(edge_feat.setdefault((n_, k_), [0] * TOT), feat[key])
            if prev is not None:
                arrivals[n_][prev] += a[A_N]
            prev = (n_, k_)
        add_into(node_a[leaf], a)
        node_ply[leaf].update(plies[key])
        add_into(leaf_feat.setdefault(leaf, [0] * TOT), feat[key])
        if prev is not None:
            arrivals[leaf][prev] += a[A_N]
    total = [0.0] * NA
    for a in agg.values():
        add_into(total, a)
    res = dict(model=model, nodes=nodes, depth=depth, N=N, ngames=len(games_tsv), flat=has_flat, node_a=node_a,
               node_ply=node_ply, edge_a=edge_a, edge_ply=edge_ply, arrivals=arrivals, paths=paths, total=total,
               total_ply=sum(plies.values(), collections.Counter()))
    # --- routers: what tells their children apart
    rinfo = {}
    for r in (n["id"] for n in nodes if n["kind"] == "branch"):
        n_k = [edge_a[(r, k)][A_N] if (r, k) in edge_a else 0 for k in range(bf)]
        n = sum(n_k)
        cf = [edge_feat.get((r, k), [0] * TOT) for k in range(bf)]
        emp = [0.0] * NF
        if n:
            for j in range(TOT):
                nv = sum(cf[k][j] for k in range(bf))
                if nv:
                    for k in range(bf):
                        x = cf[k][j]
                        if x:
                            emp[FEAT_OF[j]] += x / n * math.log(x * n / (n_k[k] * nv))
        tells, emp_llr = [], []
        for k in range(bf):
            rest = n - n_k[k]
            cand, llr = [], [None] * TOT
            for j in range(TOT):
                x = cf[k][j]
                o = sum(cf[i][j] for i in range(bf) if i != k)
                if n_k[k] and rest and x + o >= 30:
                    llr[j] = math.log((x + 0.5) / (n_k[k] + 1)) - math.log((o + 0.5) / (rest + 1))
                if n_k[k]:
                    pa, pb = x / n_k[k], (o / rest if rest else 0.0)
                    if max(pa, pb) >= 0.05:
                        cand.append((pa - pb, FEAT_OF[j], j - OFF[FEAT_OF[j]], pa, pb))
            cand.sort(key=lambda t: -abs(t[0]))
            tells.append(cand[:12])
            emp_llr.append(llr)
        cl = nodes[r]["cl"]
        nbm = learned = None
        if cl["kind"] == "cnbp":
            pi, pv = nb_probs(cl, model["nb_alpha"])
            nbm = nb_mi(pi, pv)
            learned = [nb_llr(pi, pv, k) for k in range(bf)]
        elif cl["kind"] == "lr":
            learned = [lr_llr(cl, k) for k in range(bf)]
        ms = sorted(margins.get(r, []))
        rinfo[r] = dict(n_k=n_k, emp=emp, nb=nbm, learned=learned, tells=tells, emp_llr=emp_llr,
                        margin_med=ms[len(ms) // 2] if ms else None, margin_q1=ms[len(ms) // 4] if ms else None,
                        close=sum(1 for x in ms if x < 0.1) / len(ms) if ms else None)
    res["rinfo"] = rinfo
    # --- leaves: weights, effective piece values, similarity
    linfo = {}
    for l in (n["id"] for n in nodes if n["kind"] == "leaf"):
        ps = pst(nodes[l]["cl"])
        lf = leaf_feat.get(l)
        pv = None
        if ps and lf:
            pv = []
            for p in range(12):
                w = sum(lf[OFF[s] + 1 + p] for s in range(64))
                pv.append((sum(ps[p][s] * lf[OFF[s] + 1 + p] for s in range(64)) / w) if w else None)
        linfo[l] = dict(pst=ps, pst_dr=pst_draw(nodes[l]["cl"]), pv=pv, occ=lf)
    res["linfo"] = linfo
    sim, dis = {}, {}
    if odds:
        ids = sorted(odds[0][1])
        cols = {i: [o[i] for _, o in odds] for i in ids}
        for x in ids:
            for y in ids:
                if x < y:
                    sim[(x, y)] = sim[(y, x)] = corr(cols[x], cols[y])
        for l in ids:
            own = [o for lf, o in odds if lf == l]
            for j in ids:
                if j != l and own:
                    d = sum(1 for o in own if (o[l] > 0) != (o[j] > 0)) / len(own)
                    mad = sum(abs(o[l] - o[j]) for o in own) / len(own)
                    dis[(l, j)] = (d, mad, len(own))
    res["sim"], res["dis"] = sim, dis
    res["history"] = history(m, nodes)
    res["state_se"] = state_se(m) if m["prefix"] == "bdd" else None
    res["pathstats"] = path_stats(m, games) if model["path_context"] and model["sets"] else None
    res["pathdetail"] = path_detail(m, games) if model["path_context"] and model["sets"] else None
    log(f"analysis: {time.time() - t0:.1f}s")
    return res


def corr(a, b):
    n = len(a)
    if n < 3:
        return float("nan")
    ma, mb = sum(a) / n, sum(b) / n
    num = sum((x - ma) * (y - mb) for x, y in zip(a, b))
    den = math.sqrt(sum((x - ma) ** 2 for x in a) * sum((y - mb) ** 2 for y in b))
    return num / den if den else float("nan")


HISTORY_N = 400
WAIT = 150  # seconds to wait for a running league's pool when the cache's games are few or over an hour old
FRESH = 7200  # seconds a snapshot of a league's rounds is reused before newer rounds are linked


def history(m, final_nodes):
    """The line's versions up to the model's (at most HISTORY_N, evenly spaced, always the last), from node-history:
    per version the node count and splits; per final node the version since which it has had its role (same kind,
    counters never falling; a split's central router restarts its counters) and its closed windows' mean errors."""
    runs_line = os.path.dirname(m["path"])
    vs = {v: p for v, p in versions_of(runs_line).items() if p[0] in ("bdt", "bdd") and v <= (m["version"] or 1 << 30)}
    if len(vs) < 2:
        return None
    keys = sorted(vs)
    if len(keys) > HISTORY_N:
        step = len(keys) / HISTORY_N
        keys = sorted({keys[int(i * step)] for i in range(HISTORY_N)} | {keys[-1]})
    hdir = os.path.join(CACHE, "hist")
    os.makedirs(hdir, exist_ok=True)
    want = {v: os.path.join(hdir, file_key(vs[v][1]) + ".json") for v in keys}
    missing = [v for v in keys if not os.path.exists(want[v])]
    if missing:
        out = os.path.join(hdir, f".batch-{os.getpid()}.jsonl")
        run([BIN, "node-history", out] + [f"{vs[v][0]}:{vs[v][1]}" for v in missing])
        with open(out) as f:
            for v, line in zip(missing, f):
                tmp = want[v] + ".tmp"
                with open(tmp, "w") as w:
                    w.write(line.split("\t", 1)[1])
                os.replace(tmp, want[v])
        os.remove(out)
    snaps = []
    for v in keys:
        js = json.load(open(want[v]))
        snaps.append(dict(v=v, n=len(js["nodes"]), splits=js["split_number"], sets=len(js["sets"]),
                          leaves=sum(1 for x in js["nodes"] if x["kind"] == "leaf"),
                          nodes=[(x["kind"], x["n_total"], x["n_last"], x["err_last"], x["err_prev"]) for x in js["nodes"]]))
    since, windows = {}, {}
    for nd in final_nodes:
        i = nd["id"]
        b, later = snaps[-1]["v"], None
        for s in reversed(snaps):
            if i >= s["n"] or s["nodes"][i][0] != nd["kind"] or (later is not None and s["nodes"][i][1] > later):
                break
            b, later = s["v"], s["nodes"][i][1]
        since[i] = b
        w, prev = [], None
        for s in snaps:
            if s["v"] < b or i >= s["n"]:
                continue
            ep = s["nodes"][i][4]
            if ep is not None and ep >= 0 and ep != prev:
                w.append((s["v"], ep))
            prev = ep
        last = snaps[-1]["nodes"][i] if i < snaps[-1]["n"] else None
        windows[i] = dict(closed=w, open_n=last[2] if last else 0,
                          open_mean=(last[3] / last[2]) if last and last[2] and last[3] is not None else None)
    return dict(snaps=[(s["v"], s["n"], s["leaves"], s["splits"], s["sets"]) for s in snaps], since=since,
                windows=windows, dated=len(keys), total=len(vs))


def state_se(m):
    """The standard error of each node's last closed window, from the line's latest .state (model files keep only the
    windows' means): (state version, {node: SE}); a node is matched by its example counts and previous-window mean in
    that version's model. None without a .state."""
    d = os.path.dirname(m["path"])
    best = None
    for f in os.listdir(d):
        mm = re.fullmatch(r"(bdt|bdd)(\d+)\.state", f)
        if mm and (best is None or int(mm.group(2)) > best[1]):
            best = (mm.group(1), int(mm.group(2)))
    mp = best and os.path.join(d, f"{best[0]}{best[1]}.model")
    if not best or not os.path.exists(mp):
        return None
    hj = os.path.join(CACHE, "hist", file_key(mp) + ".json")
    if not os.path.exists(hj):
        os.makedirs(os.path.dirname(hj), exist_ok=True)
        out = os.path.join(CACHE, "hist", f".state-{os.getpid()}.jsonl")
        run([BIN, "node-history", out, f"{best[0]}:{mp}"])
        with open(out) as f, open(hj + ".tmp", "w") as w:
            w.write(f.readline().split("\t", 1)[1])
        os.replace(hj + ".tmp", hj)
        os.remove(out)
    try:
        js = json.load(open(hj))
        b = open(os.path.join(d, f"{best[0]}{best[1]}.state"), "rb").read()
    except (OSError, ValueError, IndexError):
        return None
    se = {}
    for k, x in enumerate(js["nodes"]):
        try:
            pat = struct.pack("<II", x["n_total"], x["n_last"])
        except (struct.error, TypeError):
            continue
        j = b.find(pat)
        if j < 0 or b.find(pat, j + 1) >= 0 or j + 48 > len(b):
            continue
        vals = struct.unpack("<5d", b[j + 8:j + 48])
        if x["err_prev"] is not None and abs(vals[2] - x["err_prev"]) < 1e-5 and vals[4] > 0:
            se[k] = (x["kind"], math.sqrt(vals[4]))
    return best[1], se


def path_stats(m, games):
    """chessai-treeview path-stats on the report's games (round directories, read as they are)."""
    text = run([BIN, "path-stats", f"{m['prefix']}:{m['path']}"] + games + ["--use-probs"])
    per = {}
    for line in text.splitlines():
        mm = re.match(r"node (\d+) (router|leaf) member (\S+) cells \[([^\]]*)\] (.*)", line)
        if mm:
            kv = mm.group(5).split()
            d = {kv[i]: kv[i + 1] for i in range(0, len(kv) - 1, 2)}
            d["member"], d["cells"] = mm.group(3), mm.group(4)
            per[int(mm.group(1))] = d
    first = next((l for l in text.splitlines() if l.startswith("nodes ")), "")
    overall = next((l for l in text.splitlines() if l.startswith("overall")), "")
    return dict(text=text, per=per, overall=overall, head=first)


def path_detail(m, games):
    """chessai-treeview2 path-detail on the report's games: per node with path cells, each cell's terms by arrival,
    its commonest paths, and the model's log-loss on its positions with its cells and with them neutral; the whole
    model's log-loss with all cells and with all neutral (see Bdd::path_detail_json in fastbin/treeview2-src)."""
    d = json.loads(run([BIN2, "path-detail", f"{m['prefix']}:{m['path']}"] + games + ["--use-probs"]))
    d["nodes"] = {int(k): v for k, v in d["nodes"].items()}
    return d


# ---------------------------------------------------------------------------------------------- words
def flags_of(args):
    toks, fl, i = args.split(), {}, 0
    while i < len(toks):
        t = toks[i]
        if t.startswith("--"):
            if i + 1 < len(toks) and not toks[i + 1].startswith("--"):
                fl[t] = toks[i + 1]
                i += 2
                continue
            fl[t] = True
        i += 1
    return fl


def recipe_words(args):
    fl = flags_of(args)
    out = []
    leaf = {"lr": "logistic-regression", "probit": "probit", "cnbp": "naive-Bayes"}.get(fl.get("--leaf"), fl.get("--leaf"))
    router = {"cnbp": "naive-Bayes", "lr": "logistic-regression", "tan": "tree-augmented naive-Bayes"}.get(
        fl.get("--router"), fl.get("--router"))
    if leaf:
        out.append(f"{leaf} leaves")
    if router:
        how = {"cluster": "cluster routing (a router sends a position to the child whose model already fits it best, "
                          "and that child trains on it)",
               "error": "error routing (a router learns to send a position to the child whose subtree predicts it "
                        "best)"}.get(fl.get("--route"), f"routing {fl.get('--route')}")
        out.append(f"{router} routers with {how}")
    if "--branch-factor" in fl:
        out.append(f"{fl['--branch-factor']} children per split")
    if "--split-threshold" in fl:
        out.append(f"a node's split test every {int(fl['--split-threshold']):,} positions through it")
    if "--split-limit" in fl:
        out.append(f"split limit {fl['--split-limit']}")
    if "--split-noise" in fl:
        out.append(f"a split's new probit leaves are its copies, widened by noise {fl['--split-noise']}")
    if "--revise" in fl or "--revise-on" in fl:
        on = fl.get("--revise-on", "stream")
        out.append("merge-back gate: a split is undone when its branch predicts worse than the leaf it replaced "
                   f"(kept as a shadow), scored on {'fresh positions, before training on them' if on == 'fresh' else 'the training stream'}")
    if "--leaf-share" in fl:
        out.append(f"leaf sharing {fl['--leaf-share']}")
    if fl.get("--path-context"):
        out.append("path context (a node shared by several paths has a cell per upstream sibling set, so it can treat "
                   "each path differently)")
    if "--router-split-z" in fl:
        out.append(f"a router splits into {fl.get('--branch-factor', 3)} copies when its window's mean routing error "
                   f"rises by more than {fl['--router-split-z']} SE over its previous window and is at least the "
                   "routers' mean")
    if "--router-split-limit" in fl:
        out.append(f"at most {fl['--router-split-limit']} router splits, trailing the leaf splits")
    return out


def ply_q(c):
    n = sum(c.values())
    if not n:
        return None, None, None
    out, acc, qs = [], 0, [n * 0.25, n * 0.5, n * 0.75]
    for p in sorted(c):
        acc += c[p]
        while len(out) < 3 and acc >= qs[len(out)]:
            out.append(p)
    return tuple(out)


def region(a, plyc):
    n = a[A_N] if a else 0
    if not n:
        return None
    q1, med, q3 = ply_q(plyc)
    dec = a[A_DEC]
    return dict(n=n, ply=med, q1=q1, q3=q3, npm=a[A_NPM] / n, pawns=a[A_PAWNS] / n, bal=a[A_BAL] / n,
                absbal=a[A_ABS] / n, bq=a[A_BQ] / n, nq=a[A_NQ] / n, cas=a[A_CAS] / n, wtm=a[A_WTM] / n,
                w=a[A_W] / n, b=a[A_B] / n, d=a[A_D] / n, ll=a[A_LL] / n, fll=a[A_FLL] / n,
                wll=a[A_WLL] / dec if dec else None, fwll=a[A_FWLL] / dec if dec else None,
                ph=(a[A_PH0] / n, a[A_PH1] / n, a[A_PH2] / n), pcs=a[A_PCS] / n)


def region_words(g, flat=True):
    if not g:
        return "no positions"
    return (f"ply {g['ply']} ({g['q1']}-{g['q3']}), npm {g['npm']:.0f}, pawns {g['pawns']:.1f}, phase "
            f"{100 * g['ph'][0]:.0f}/{100 * g['ph'][1]:.0f}/{100 * g['ph'][2]:.0f}, queens both {100 * g['bq']:.0f}% "
            f"none {100 * g['nq']:.0f}%, castling {100 * g['cas']:.0f}%, wtm {100 * g['wtm']:.0f}%, |bal| "
            f"{g['absbal']:.1f}, W/B/D {100 * g['w']:.0f}/{100 * g['b']:.0f}/{100 * g['d']:.0f}, ll {g['ll']:.3f}"
            + (f" (flat {g['fll']:.3f})" if flat else ""))


def node_label(res, i):
    nd = res["nodes"][i]
    if nd["kind"] == "leaf":
        return f"leaf {i}"
    mem = nd.get("member")
    return f"router {i}" + (f" (set {mem[0]} copy {mem[1]})" if mem else "")


def text_summary(m, res, flat_name):
    out, model, nodes, N = [], res["model"], res["nodes"], res["N"]
    since = (res["history"] or {}).get("since", {})
    out.append(f"{m['name'] or ''}:{m['line']}@{m['version']}: {len(nodes)} nodes, "
               f"{sum(1 for n in nodes if n['kind'] == 'leaf')} leaves, {len(model['sets'])} sibling sets, "
               f"{model['split_number']} splits; {N} positions of {res['ngames']} games")
    t = region(res["total"], res["total_ply"])
    out.append(f"  all: ll {t['ll']:.4f}" + (f" vs flat {flat_name} {t['fll']:.4f}" if res["flat"] else "")
               + (f"; winner ll {t['wll']:.4f}" + (f" vs {t['fwll']:.4f}" if res["flat"] else "") if t["wll"] else ""))
    for i in sorted(res["depth"], key=lambda x: (res["depth"][x], x)):
        nd = nodes[i]
        na = res["node_a"].get(i)
        share = (na[A_N] / N) if na and N else 0.0
        if nd["kind"] == "branch":
            ri = res["rinfo"][i]
            arr = res["arrivals"].get(i)
            out.append(f"  {node_label(res, i)} [{nd['cl']['kind']}, since v{since.get(i, '?')}] {100 * share:.1f}%: "
                       + ", ".join(f"{c} {100 * ri['n_k'][k] / max(sum(ri['n_k']), 1):.0f}%"
                                   for k, c in enumerate(nd["children"]))
                       + (f"; margin med {ri['margin_med']:.2f}, <0.1 {100 * ri['close']:.0f}%"
                          if ri["margin_med"] is not None else ""))
            if arr and len(arr) > 1:
                out.append("    arrives from " + ", ".join(f"{p}:{k} {100 * x / na[A_N]:.0f}%" for (p, k), x in arr.most_common()))
            top = sorted(range(NF), key=lambda f: -ri["emp"][f])[:6]
            out.append("    routed apart by (MI nats): " + ", ".join(f"{featname(f)} {ri['emp'][f]:.3f}" for f in top))
            if ri["nb"]:
                topn = sorted(range(NF), key=lambda f: -ri["nb"][f])[:6]
                out.append("    its NB counts: " + ", ".join(f"{featname(f)} {ri['nb'][f]:.3f}" for f in topn))
            for k, c in enumerate(nd["children"]):
                g = region(res["edge_a"].get((i, k)), res["edge_ply"].get((i, k), collections.Counter()))
                out.append(f"    -> {c}: {region_words(g, res['flat'])}")
                if ri["tells"][k]:
                    out.append("       " + "; ".join(f"{valname(f, v)} {100 * pa:.0f}% vs {100 * pb:.0f}%"
                                                    for _, f, v, pa, pb in ri["tells"][k][:5]))
        else:
            g = region(na, res["node_ply"].get(i, collections.Counter()))
            out.append(f"  leaf {i} [{nd['cl']['kind']}, since v{since.get(i, '?')}] {100 * share:.1f}%: "
                       f"{region_words(g, res['flat'])}")
    if res["pathstats"]:
        out.append("  path-stats: " + res["pathstats"]["overall"])
    if res.get("pathdetail"):
        d = res["pathdetail"]
        ll, ll0, se = pd_mean_se(d["model"])
        out.append(f"  path-detail: {d['path_params']} path parameters; log-loss with {ll:.4f}, without {ll0:.4f}; "
                   f"difference {ll - ll0:.5f} ± {se or 0:.5f} SE (paired over positions)")
    return "\n".join(out)


# ---------------------------------------------------------------------------------------------- the page
def esc(s):
    return html.escape(str(s), quote=True)


def fnum(x, k=3):
    """Numbers as the pages write them: a minus sign only when negative, no plus signs."""
    if x is None or (isinstance(x, float) and (math.isnan(x) or math.isinf(x))):
        return "–"
    return f"{x:.{k}f}".replace("-", "−")


def pct(x, k=0):
    return "–" if x is None else f"{100 * x:.{k}f}%"


def big(n):
    n = int(n)
    return f"{n / 1e6:.2f}M" if n >= 1e6 else (f"{n / 1e3:.0f}k" if n >= 1e4 else f"{n:,}")


# OKLab interpolation for the diverging and sequential scales (lightness monotonic by construction; from leafview)
def _lin(c):
    c /= 255
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def _gam(c):
    c = max(0.0, min(1.0, c))
    return 12.92 * c if c <= 0.0031308 else 1.055 * c ** (1 / 2.4) - 0.055


def to_oklab(h):
    r, g, b = (_lin(int(h[i:i + 2], 16)) for i in (1, 3, 5))
    l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b
    m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b
    s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b
    l, m, s = (x ** (1 / 3) for x in (l, m, s))
    return (0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
            1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
            0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s)


def _rgb(L, a, b):
    l = (L + 0.3963377774 * a + 0.2158037573 * b) ** 3
    m = (L - 0.1055613458 * a - 0.0638541728 * b) ** 3
    s = (L - 0.0894841775 * a - 1.2914855480 * b) ** 3
    return (4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
            -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
            -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s)


def from_oklab(L, a, b):
    return "#" + "".join(f"{round(255 * _gam(c)):02x}" for c in _rgb(L, a, b))


def in_gamut(L, a, b):
    return all(-1e-4 <= c <= 1 + 1e-4 for c in _rgb(L, a, b))


def match_lightness(hue_hex, L):
    _, a, b = to_oklab(hue_hex)
    hue = math.atan2(b, a)
    c = 0.4
    while c > 0 and not in_gamut(L, c * math.cos(hue), c * math.sin(hue)):
        c -= 0.002
    return from_oklab(L, c * math.cos(hue), c * math.sin(hue))


def arm(mid, end, n):
    m, e = to_oklab(mid), to_oklab(end)
    return [from_oklab(*(m[j] + (e[j] - m[j]) * i / n for j in range(3))) for i in range(1, n + 1)]


def diverging(mid, blue_end, red_hue):
    red_end = match_lightness(red_hue, to_oklab(blue_end)[0])
    return list(reversed(arm(mid, red_end, 5))) + [mid] + arm(mid, blue_end, 5)


DIV = {"light": diverging("#f0efec", "#104281", "#e34948"), "dark": diverging("#383835", "#86b6ef", "#e66767")}
SEQ = {"light": ["#cde2fb", "#9ec5f4", "#6da7ec", "#3987e5", "#256abf", "#184f95", "#0d366b"],
       "dark": ["#104281", "#184f95", "#256abf", "#2a78d6", "#5598e7", "#86b6ef", "#cde2fb"]}

PAGE_CSS = """
:root, [data-theme=light] { color-scheme: light;
  --surface:#fcfcfb; --raised:#f5f5f2; --ink:#0b0b0b; --ink2:#52514e; --muted:#898781; --grid:#e1e0d9; --base:#c3c2b7;
  --s1:#2a78d6; --s2:#eb6834; --s3:#1baf7a; --res-w:#e9e6dc; --res-b:#3d3c39; --res-d:#a9a79d; %(divL)s %(seqL)s }
[data-theme=dark] { color-scheme: dark;
  --surface:#1a1a19; --raised:#222220; --ink:#f0efec; --ink2:#c3c2b7; --muted:#898781; --grid:#2c2c2a; --base:#383835;
  --s1:#3987e5; --s2:#d95926; --s3:#199e70; --res-w:#ebe8df; --res-b:#0c0c0b; --res-d:#77756e; %(divD)s %(seqD)s }
@media (prefers-color-scheme: dark) { :root:not([data-theme=light]) { color-scheme: dark;
  --surface:#1a1a19; --raised:#222220; --ink:#f0efec; --ink2:#c3c2b7; --muted:#898781; --grid:#2c2c2a; --base:#383835;
  --s1:#3987e5; --s2:#d95926; --s3:#199e70; --res-w:#ebe8df; --res-b:#0c0c0b; --res-d:#77756e; %(divD)s %(seqD)s } }
* { box-sizing: border-box; }
body { margin:0; background:var(--surface); color:var(--ink); font:14px/1.45 system-ui, -apple-system, "Segoe UI", sans-serif; }
main { max-width:1180px; margin:0 auto; padding:24px 20px 64px; }
a { color:var(--s1); text-decoration:none; } a:hover { text-decoration:underline; }
h1 { font-size:22px; margin:0 0 4px; } h2 { font-size:18px; margin:34px 0 8px; padding-top:8px; border-top:1px solid var(--grid); }
h3 { font-size:16px; margin:0 0 4px; } h4 { font-size:13px; margin:14px 0 4px; color:var(--ink2); text-transform:uppercase; letter-spacing:.04em; }
p { margin:6px 0; max-width:980px; } .sub { color:var(--ink2); }
.mono, code { font-family: ui-monospace, "SFMono-Regular", Menlo, monospace; font-size:12px; }
.small { font-size:12px; color:var(--ink2); } .muted { color:var(--muted); }
.top { display:flex; justify-content:space-between; align-items:flex-start; gap:16px; }
button { font:inherit; font-size:12.5px; color:var(--ink); background:var(--raised); border:1px solid var(--base);
  border-radius:6px; padding:3px 9px; cursor:pointer; }
button[aria-pressed=true] { background:var(--ink); color:var(--surface); border-color:var(--ink); }
.tiles { display:grid; grid-template-columns:repeat(auto-fit, minmax(150px, 1fr)); gap:12px; margin:16px 0; }
.tile { background:var(--raised); border-radius:8px; padding:10px 14px; }
.tile .k { font-size:12px; color:var(--ink2); } .tile .v { font-size:20px; font-weight:600; margin:2px 0; font-variant-numeric:tabular-nums; }
.tile .s { font-size:12px; color:var(--ink2); }
.findings { background:var(--raised); border-radius:8px; padding:10px 16px 10px 32px; margin:12px 0; }
.findings li { margin:4px 0; max-width:1000px; }
table { border-collapse:collapse; margin:6px 0; font-variant-numeric:tabular-nums; }
th, td { padding:3px 7px; text-align:right; border-bottom:1px solid var(--grid); font-size:12.5px; vertical-align:middle; }
th { color:var(--ink2); font-weight:600; } td.l, th.l { text-align:left; } td.nw { white-space:nowrap; }
.scroll { overflow-x:auto; }
.node { background:var(--raised); border-radius:10px; padding:12px 16px 14px; margin:16px 0; scroll-margin-top:12px; }
.node.leaf { border-left:4px solid var(--s1); } .node.router { border-left:4px solid var(--ink2); }
.node:target { outline:2px solid var(--s1); }
.node table { background:var(--surface); }
.bar { display:inline-block; height:9px; border-radius:2px; background:var(--s1); vertical-align:middle; }
.bar.alt { background:var(--s2); }
.chips { display:flex; flex-wrap:wrap; gap:4px 6px; margin:3px 0 6px; } .chip { font-size:12px; background:var(--surface);
  border:1px solid var(--grid); border-radius:10px; padding:1px 8px; white-space:nowrap; }
.chip b { font-weight:600; } .up { color:var(--s1); } .dn { color:var(--s2); }
details { margin:8px 0; } summary { cursor:pointer; color:var(--ink2); font-size:13px; }
.ctl { display:flex; flex-wrap:wrap; gap:8px 14px; align-items:center; margin:8px 0; }
.seg { display:inline-flex; gap:4px; align-items:center; } .seg .lab { color:var(--ink2); font-size:12px; margin-right:2px; }
.boards { display:grid; grid-template-columns:repeat(6, minmax(92px, 1fr)); gap:12px 10px; margin-top:6px; max-width:900px; }
.board .t { font-size:11.5px; color:var(--ink2); margin-bottom:2px; white-space:nowrap; }
.bgrid { display:grid; grid-template-columns:9px repeat(8, 1fr); gap:1px; }
.bgrid .c { aspect-ratio:1; border-radius:1.5px; } .bgrid .rk, .bgrid .fl { font-size:8.5px; color:var(--muted); text-align:center; line-height:1; }
.bgrid .rk { align-self:center; } .c.na { background:repeating-linear-gradient(45deg, var(--base) 0 1px, transparent 1px 4px); }
.scale { display:flex; gap:2px; align-items:flex-start; margin-top:8px; } .scale > div { width:36px; text-align:center; font-size:10px; color:var(--muted); }
.scale .b { width:34px; height:10px; border-radius:2px; margin:0 auto 2px; }
.cap { font-size:12px; color:var(--ink2); max-width:900px; }
.legend { display:flex; flex-wrap:wrap; gap:14px; align-items:center; font-size:12px; color:var(--ink2); margin:6px 0; }
.sw { display:inline-block; width:12px; height:10px; border-radius:2px; vertical-align:middle; margin-right:4px; border:1px solid var(--base); }
#tip { position:fixed; pointer-events:none; background:var(--raised); color:var(--ink); border:1px solid var(--base);
  border-radius:6px; padding:6px 9px; font-size:12px; line-height:1.4; z-index:20; display:none; max-width:380px;
  box-shadow:0 2px 8px rgba(0,0,0,.15); white-space:pre-line; }
svg text { font-family: system-ui, -apple-system, "Segoe UI", sans-serif; }
.two { display:flex; flex-wrap:wrap; gap:10px 28px; align-items:flex-start; }
.note { color:var(--ink2); font-size:12.5px; }
.graph { overflow-x:auto; background:var(--raised); border-radius:10px; padding:8px; }
pre.raw { background:var(--surface); border:1px solid var(--grid); border-radius:6px; padding:8px; font-size:11.5px; overflow-x:auto; }
"""


def page_css():
    def var(theme, pre, cols):
        return " ".join(f"--{pre}{i}:{c};" for i, c in enumerate(cols[theme]))
    return PAGE_CSS % dict(divL=var("light", "d", DIV), divD=var("dark", "d", DIV), seqL=var("light", "q", SEQ),
                           seqD=var("dark", "q", SEQ))


PHASES = ("full board", "middlegame", "endgame")


def phase_of(g):
    return PHASES[max(range(3), key=lambda j: g["ph"][j])] if g else "–"


def short_region(g):
    """A region in a few words: its main phase, ply and results."""
    if not g:
        return "no positions"
    return (f"{phase_of(g)}s ({100 * max(g['ph']):.0f}%), ply {g['ply']} ({g['q1']}–{g['q3']}), "
            f"W/B/D {100 * g['w']:.0f}/{100 * g['b']:.0f}/{100 * g['d']:.0f}")


def mini_bar(parts, colours, w=64, h=10, tip=""):
    x, out = 0.0, []
    for v, c in zip(parts, colours):
        ww = w * max(v, 0.0)
        if ww > 0:
            out.append(f'<rect x="{x:.1f}" y="0" width="{ww:.1f}" height="{h}" fill="{c}"/>')
        x += ww
    return (f'<svg width="{w}" height="{h}" style="vertical-align:middle" data-tip="{esc(tip)}">'
            + "".join(out) + "</svg>")


def phase_bar(g):
    if not g:
        return ""
    return mini_bar(g["ph"], ["var(--q1)", "var(--q3)", "var(--q5)"],
                    tip=f"full board (non-pawn material ≥ 52) {pct(g['ph'][0])}\nmiddlegame (26–51) {pct(g['ph'][1])}"
                        f"\nendgame (≤ 25) {pct(g['ph'][2])}")


def result_bar(g):
    if not g:
        return ""
    return mini_bar([g["w"], g["b"], g["d"]], ["var(--res-w)", "var(--res-b)", "var(--res-d)"],
                    tip=f"white wins {pct(g['w'])}\nblack wins {pct(g['b'])}\ndraws {pct(g['d'])}")


REGION_HEADS = ("<th>Ply</th><th>Phase</th><th data-tip='non-pawn material, both sides: N, B 3, R 5, Q 9 (62 at "
                "the start)'>Pieces</th><th>Pawns</th><th data-tip='both queens on the board / none'>Queens</th>"
                "<th data-tip='a castling right left to either side'>Castling</th><th data-tip='white to move'>"
                "W to move</th><th data-tip='material, white minus black (P 1, N B 3, R 5, Q 9): its mean, and in "
                "brackets the mean of its size'>Balance</th><th>W/B/D</th>")


def region_cells(g):
    if not g:
        return "<td colspan=9 class='l muted'>no positions</td>"
    return (f"<td class=nw>{g['ply']} <span class=muted>({g['q1']}–{g['q3']})</span></td><td>{phase_bar(g)}</td>"
            f"<td>{g['npm']:.0f}</td><td>{g['pawns']:.1f}</td><td class=nw>{pct(g['bq'])} / {pct(g['nq'])}</td>"
            f"<td>{pct(g['cas'])}</td><td>{pct(g['wtm'])}</td>"
            f"<td class=nw>{fnum(g['bal'], 1)} <span class=muted>({g['absbal']:.1f})</span></td>"
            f"<td class=nw>{result_bar(g)} {100 * g['w']:.0f}/{100 * g['b']:.0f}/{100 * g['d']:.0f}</td>")


def loss_cells(g, flat):
    if not g:
        return "<td></td>" * (4 if flat else 2)
    out = f"<td>{fnum(g['ll'])}</td>"
    if flat:
        d = g["ll"] - g["fll"]
        out += f"<td>{fnum(g['fll'])}</td><td class='{'up' if d < 0 else 'dn'}'>{fnum(d)}</td>"
    out += f"<td>{fnum(g['wll'])}</td>" if g["wll"] is not None else "<td>–</td>"
    return out


def loss_heads(flat):
    return ("<th data-tip='mean log-loss of the model (the whole tree or DAG, as play uses it) on these positions'>"
            "Log-loss</th>" + ("<th data-tip='the flat model on the same positions'>Flat</th><th data-tip='model minus "
                               "flat: negative, the model predicts these positions better'>Gap</th>" if flat else "")
            + "<th data-tip='log-loss of who won, on the decisive positions (draws left out), as the model splits "
              "its white-win and black-win probabilities'>Winner</th>")


def layout(res):
    """Node centres for the diagram: a tree as a tidy tree (leaves in order, routers over their children); a DAG
    in layers by longest path from the root, each layer ordered by its parents' positions."""
    nodes, depth = res["nodes"], res["depth"]
    reach = set(depth)
    shared = [i for i in reach if len({p for p, _ in nodes[i]["parents"] if p in reach}) > 1]
    SLOT = 94
    x = {}
    if not shared:
        order = []

        def walk(i):
            if nodes[i]["kind"] == "leaf":
                order.append(i)
            else:
                for c in nodes[i]["children"]:
                    walk(c)
        walk(0)
        for k, i in enumerate(order):
            x[i] = k * SLOT

        def place(i):
            if nodes[i]["kind"] == "branch":
                for c in nodes[i]["children"]:
                    place(c)
                x[i] = sum(x[c] for c in nodes[i]["children"]) / len(nodes[i]["children"])
        place(0)
    else:
        layers = collections.defaultdict(list)
        seq, seen = [], set()

        def dfs(i):
            if i not in seen:
                seen.add(i)
                seq.append(i)
                for c in nodes[i]["children"]:
                    dfs(c)
        dfs(0)
        rank = {i: k for k, i in enumerate(seq)}
        for i in reach:
            layers[depth[i]].append(i)
        width = max(len(v) for v in layers.values())

        def assign():
            for L in layers.values():
                off = (width - len(L)) / 2
                for k, i in enumerate(L):
                    x[i] = (off + k) * SLOT
        for L in layers.values():
            L.sort(key=lambda i: rank[i])
        assign()
        for _ in range(4):
            for d in sorted(layers)[1:]:
                def bary(i):
                    ps = sorted((x[p], k) for p, k in nodes[i]["parents"] if p in reach)
                    return (sum(px for px, _ in ps) / len(ps), ps[0]) if ps else (x[i], (0, 0))
                layers[d].sort(key=bary)
                assign()
    lo = min(x.values())
    return {i: v - lo for i, v in x.items()}


def graph_svg(res, since):
    nodes, depth, N = res["nodes"], res["depth"], max(res["N"], 1)
    reach = set(depth)
    x = layout(res)
    BW, BH, LH, M = 80, 44, 100, 48
    y = {i: 24 + depth[i] * LH for i in reach}
    width = max(x.values()) + 2 * M
    height = max(y.values()) + BH + 24
    out = [f'<svg viewBox="0 0 {width:.0f} {height:.0f}" width="{width:.0f}" height="{height:.0f}" role="img" '
           f'aria-label="The graph: routers, leaves, sibling sets and shared children">']
    for s_i, members in enumerate(res["model"]["sets"]):
        ms = [m for m in members if m in reach]
        if not ms or len({y[m] for m in ms}) != 1:
            continue
        x0, x1 = min(x[m] for m in ms) + M - BW / 2 - 7, max(x[m] for m in ms) + M + BW / 2 + 7
        yy = y[ms[0]]
        out.append(f'<g data-tip="sibling set {s_i}: the copies one router split made (nodes {", ".join(map(str, ms))})">'
                   f'<rect x="{x0:.1f}" y="{yy - 15}" width="{x1 - x0:.1f}" height="{BH + 21}" rx="9" fill="none" '
                   f'stroke="var(--s3)" stroke-width="1.2" stroke-dasharray="4 3"/>'
                   f'<text x="{x0 + 6:.1f}" y="{yy - 5}" font-size="10" fill="var(--s3)">set {s_i}</text></g>')
    bf = res["model"]["branch_factor"]
    for i in sorted(reach):
        nd = nodes[i]
        if nd["kind"] != "branch":
            continue
        tot = (res["node_a"].get(i) or [0])[A_N] or 1
        for k, c in enumerate(nd["children"]):
            n = (res["edge_a"].get((i, k)) or [0])[A_N]
            x1, y1 = x[i] + M + (k - (bf - 1) / 2) * 24, y[i] + BH
            x2, y2 = x[c] + M, y[c]
            sw = 0.8 + 6 * n / N
            out.append(f'<path d="M{x1:.1f},{y1} C{x1:.1f},{y1 + 30} {x2:.1f},{y2 - 34} {x2:.1f},{y2}" fill="none" '
                       f'stroke="var(--base)" stroke-width="{sw:.2f}" data-tip="{esc(f"router {i}, child {k}: node {c}")}'
                       f'{esc(chr(10) + f"{pct(n / tot, 1)} of the router positions, {pct(n / N, 1)} of all")}"/>')
            t = 0.2
            bx = (1 - t) ** 3 * x1 + 3 * (1 - t) ** 2 * t * x1 + 3 * (1 - t) * t ** 2 * x2 + t ** 3 * x2
            by = (1 - t) ** 3 * y1 + 3 * (1 - t) ** 2 * t * (y1 + 30) + 3 * (1 - t) * t ** 2 * (y2 - 34) + t ** 3 * y2
            out.append(f'<text x="{bx:.1f}" y="{by + 3:.1f}" font-size="9.5" fill="var(--muted)" text-anchor="middle" '
                       f'paint-order="stroke" stroke="var(--raised)" stroke-width="3">{100 * n / tot:.0f}%</text>')
    for i in sorted(reach):
        nd = nodes[i]
        na = res["node_a"].get(i)
        share = (na[A_N] / N) if na else 0.0
        g = region(na, res["node_ply"].get(i, collections.Counter())) if na else None
        cx, cy = x[i] + M, y[i]
        router = nd["kind"] == "branch"
        npar = len({p for p, _ in nd["parents"] if p in reach})
        mem = nd.get("member")
        tip = (f"{node_label(res, i)}: {nd['cl']['kind']}, since v{since.get(i, '?')}\n"
               f"{pct(share, 1)} of positions ({big(na[A_N]) if na else 0})\n{short_region(g)}"
               + (f"\nlog-loss {fnum(g['ll'])}" + (f", flat {fnum(g['fll'])}" if res['flat'] else "") if g else "")
               + (f"\nreached from {npar} parents" if npar > 1 else "")
               + (f"\ncopy {mem[1]} of sibling set {mem[0]}" if mem else ""))
        stroke = "var(--ink2)" if router else "var(--s1)"
        fill = "var(--raised)" if router else "var(--surface)"
        out.append(f'<a href="#node-{i}"><g data-tip="{esc(tip)}"><rect x="{cx - BW / 2:.1f}" y="{cy}" width="{BW}" '
                   f'height="{BH}" rx="7" fill="{fill}" stroke="{stroke}" stroke-width="1.6"/>'
                   f'<rect x="{cx - BW / 2 + 5:.1f}" y="{cy + BH - 8}" width="{max(1.0, (BW - 10) * share):.1f}" '
                   f'height="3" rx="1.5" fill="var(--s1)" opacity=".7"/>'
                   f'<text x="{cx:.1f}" y="{cy + 16}" font-size="12.5" font-weight="600" fill="var(--ink)" '
                   f'text-anchor="middle">{"R" if router else "L"}{i}</text>'
                   f'<text x="{cx:.1f}" y="{cy + 30}" font-size="10.5" fill="var(--ink2)" text-anchor="middle">'
                   f'{pct(share, 1 if share < 0.1 else 0)} · v{since.get(i, "?")}</text>'
                   + (f'<circle cx="{cx + BW / 2 - 2:.1f}" cy="{cy + 2}" r="8" fill="var(--s3)"/>'
                      f'<text x="{cx + BW / 2 - 2:.1f}" y="{cy + 5.5}" font-size="9.5" fill="var(--surface)" '
                      f'text-anchor="middle" font-weight="600">{npar}</text>' if npar > 1 else "")
                   + "</g></a>")
    out.append("</svg>")
    return "\n".join(out)


def growth_svg(h):
    snaps = h["snaps"]
    if len(snaps) < 2:
        return ""
    W, H, ml, mr, mt, mb = 820, 150, 36, 110, 12, 24
    pw, ph = W - ml - mr, H - mt - mb
    v0, v1 = snaps[0][0], snaps[-1][0]
    top = max(max(s[2], s[1] - s[2]) for s in snaps) + 1

    def X(v):
        return ml + (v - v0) / max(v1 - v0, 1) * pw

    def Y(n):
        return mt + (top - n) / top * ph
    out = [f'<svg viewBox="0 0 {W} {H}" width="100%" style="max-width:{W}px" role="img" aria-label="Leaves and '
           f'routers by version">']
    step = max(1, round(top / 4))
    for n in range(0, top + 1, step):
        out.append(f'<line x1="{ml}" x2="{ml + pw}" y1="{Y(n):.1f}" y2="{Y(n):.1f}" stroke="var(--grid)"/>'
                   f'<text x="{ml - 6}" y="{Y(n) + 4:.1f}" font-size="10.5" fill="var(--muted)" text-anchor="end">{n}</text>')
    for v in (v0, (v0 + v1) // 2, v1):
        out.append(f'<text x="{X(v):.1f}" y="{H - 6}" font-size="10.5" fill="var(--muted)" text-anchor="middle">v{v}</text>')
    for k, (col, name, f) in enumerate((("var(--s1)", "leaves", lambda s: s[2]),
                                        ("var(--ink2)", "routers", lambda s: s[1] - s[2]))):
        pts = []
        for j, s in enumerate(snaps):
            if j:
                pts.append(f"{X(s[0]):.1f},{Y(f(snaps[j - 1])):.1f}")
            pts.append(f"{X(s[0]):.1f},{Y(f(s)):.1f}")
        out.append(f'<polyline points="{" ".join(pts)}" fill="none" stroke="{col}" stroke-width="2"/>'
                   f'<text x="{ml + pw + 8}" y="{Y(f(snaps[-1])) + 4 + 12 * k:.1f}" font-size="11.5" fill="{col}">'
                   f'{name} {f(snaps[-1])}</text>')
    for a, b in zip(snaps, snaps[1:]):
        if b[3] != a[3] or b[1] != a[1]:
            grew = b[1] > a[1]
            tip = (f"v{a[0]} → v{b[0]}: {a[1]} → {b[1]} nodes, {a[2]} → {b[2]} leaves"
                   + (f", sibling sets {a[4]} → {b[4]}" if b[4] != a[4] else "")
                   + ("" if grew else "\n(fewer nodes: a split undone by the merge-back gate)"))
            xx = X(b[0])
            out.append(f'<g data-tip="{esc(tip)}"><line x1="{xx:.1f}" x2="{xx:.1f}" y1="{mt}" y2="{mt + ph}" '
                       f'stroke="{"var(--s3)" if grew else "var(--s2)"}" stroke-dasharray="3 3"/>'
                       f'<rect x="{xx - 4:.1f}" y="{mt}" width="8" height="{ph}" fill="transparent"/></g>')
    out.append("</svg>")
    return "\n".join(out)


def window_svg(w, kind):
    pts = w["closed"]
    if not pts:
        return '<span class="small muted">no window closed yet in its current role</span>'
    W, H, ml, mr, mt, mb = 330, 74, 42, 10, 8, 18
    pw, ph = W - ml - mr, H - mt - mb
    vals = [e for _, e in pts] + ([w["open_mean"]] if w["open_mean"] is not None else [])
    lo, hi = min(vals), max(vals)
    if hi - lo < 1e-9:
        lo, hi = lo - 0.01, hi + 0.01
    pad = (hi - lo) * 0.12
    lo, hi = lo - pad, hi + pad
    n = len(pts) + (1 if w["open_mean"] is not None else 0)

    def X(j):
        return ml + (j + 0.5) / max(n, 1) * pw

    def Y(e):
        return mt + (hi - e) / (hi - lo) * ph
    out = [f'<svg width="{W}" height="{H}" role="img" aria-label="Error by window">',
           f'<text x="{ml - 5}" y="{Y(hi - pad) + 4:.1f}" font-size="9.5" fill="var(--muted)" text-anchor="end">{fnum(hi - pad)}</text>',
           f'<text x="{ml - 5}" y="{Y(lo + pad) + 4:.1f}" font-size="9.5" fill="var(--muted)" text-anchor="end">{fnum(lo + pad)}</text>',
           f'<line x1="{ml}" x2="{ml + pw}" y1="{mt + ph}" y2="{mt + ph}" stroke="var(--grid)"/>']
    out.append(f'<polyline points="{" ".join(f"{X(j):.1f},{Y(e):.1f}" for j, (_, e) in enumerate(pts))}" fill="none" '
               f'stroke="var(--s1)" stroke-width="1.6"/>')
    for j, (v, e) in enumerate(pts):
        ch = (e - pts[j - 1][1]) if j else None
        tip = (f"window closed by v{v}: mean {kind} error {fnum(e, 4)}"
               + (f"\nchange from the window before: {fnum(ch, 4)}" if ch is not None else ""))
        out.append(f'<circle cx="{X(j):.1f}" cy="{Y(e):.1f}" r="3" fill="var(--s1)" data-tip="{esc(tip)}"/>')
    if w["open_mean"] is not None:
        j = len(pts)
        out.append(f'<circle cx="{X(j):.1f}" cy="{Y(w["open_mean"]):.1f}" r="3.2" fill="var(--surface)" '
                   f'stroke="var(--s1)" stroke-width="1.4" data-tip="{esc(f"the open window: {big(w["open_n"])} positions, mean {fnum(w["open_mean"], 4)}")}"/>')
    out.append(f'<text x="{ml}" y="{H - 4}" font-size="9.5" fill="var(--muted)">v{pts[0][0]}</text>'
               f'<text x="{ml + pw}" y="{H - 4}" font-size="9.5" fill="var(--muted)" text-anchor="end">'
               f'{"open" if w["open_mean"] is not None else "v" + str(pts[-1][0])}</text></svg>')
    return "".join(out)


def board_vals(v, flat_index):
    """768 values (12 pieces x 64 squares, piece-major) rounded for the page; None where missing."""
    out = []
    for p in range(12):
        for s in range(64):
            x = v(p, s) if not flat_index else v[OFF[s] + 1 + p]
            out.append(None if x is None else round(x, 3))
    return out


def domain(lists):
    xs = sorted(abs(x) for l in lists if l for x in l if x is not None)
    return xs[int(0.98 * (len(xs) - 1))] if xs else 1.0


def feat_rows(ri, k=10):
    top = sorted(range(NF), key=lambda f: -ri["emp"][f])[:k]
    nb = ri["nb"]
    nbrank = {f: r for r, f in enumerate(sorted(range(NF), key=lambda f: -nb[f]), 1)} if nb else {}
    mx = nbmx = max([ri["emp"][f] for f in top] + (list(nb) if nb else [])) or 1
    rows = []
    for f in top:
        rows.append(f"<tr><td class=l>{esc(featname(f))}</td><td class=nw>{fnum(ri['emp'][f])} "
                    f"<span class=bar style='width:{60 * ri['emp'][f] / mx:.0f}px'></span></td>"
                    + (f"<td class=nw>{fnum(nb[f])} <span class='bar alt' style='width:{60 * nb[f] / nbmx:.0f}px'></span>"
                       f"</td><td>{nbrank[f]}</td>" if nb else "") + "</tr>")
    nbtop = ""
    if nb:
        nbtop = ", ".join(f"{esc(featname(f))} {fnum(nb[f])}" for f in sorted(range(NF), key=lambda f: -nb[f])[:6])
    return "".join(rows), nbtop


# ---------------------------------------------------------------------------------------------- path parameters
def pd_d(x):
    """A log-loss difference or its standard error: five decimals, no minus sign on a zero."""
    return fnum(None if x is None else round(x, 5) + 0.0, 5)


def pd_lab(nodes, j):
    return f"<a href='#node-{j}'>{'R' if nodes[j]['kind'] == 'branch' else 'L'}{j}</a>"


def pd_mean_se(p):
    """A path-detail payoff record: the mean log-loss with the cells, with them neutral, and the standard error of
    the per-position difference (paired over positions)."""
    n = p["n"]
    if not n:
        return None, None, None
    ll, ll0 = p["ll"] / n, p["ll0"] / n
    se = math.sqrt(max(p["d2"] - n * (ll - ll0) ** 2, 0.0) / (n - 1) / n) if n > 1 else None
    return ll, ll0, se


def pd_game_se(d):
    """The standard error of the whole model's mean difference (with minus without), clustered by game."""
    gs = [(n, x) for n, x in d["game_d"] if n]
    big_n, games = sum(n for n, _ in gs), len(gs)
    if games < 2 or not big_n:
        return None
    mean = sum(x for _, x in gs) / big_n
    return math.sqrt(games / (games - 1) * sum((x - n * mean) ** 2 for n, x in gs)) / big_n


def pd_model_line(res):
    """The DAG page's top line on the path parameters: the model's log-loss with all cells and with all neutral."""
    d = res.get("pathdetail")
    if not d:
        return ""
    w = d["model"]
    ll, ll0, se = pd_mean_se(w)
    if ll is None:
        return ""
    gse = pd_game_se(d)
    wl = (f" Winner log-loss with {fnum(w['wll'] / w['dec'], 4)}, without {fnum(w['wll0'] / w['dec'], 4)}."
          if w["dec"] else "")
    bad = (f" <b>Warning: {d['mismatched']} routing steps differed from play's.</b>" if d["mismatched"] else "")
    return (f"<p><b>Path parameters {d['path_params']:,}; log-loss with {fnum(ll, 4)}, without {fnum(ll0, 4)}; "
            f"difference {pd_d(ll - ll0)} ± {pd_d(se)} SE (paired over positions"
            + (f"; ± {pd_d(gse)} clustered by game" if gse is not None else "") + ").</b> "
            f"<span class=small>Difference: with minus without, so below zero the cells help. Without: every router "
            f"routes by its classifier alone and every leaf predicts by its own; "
            f"{pct(d['leaf_moved'] / max(d['positions'], 1), 1)} of positions then end at another leaf.{wl}"
            f"{bad}</span></p>")


def pd_section(res, i):
    """A DAG node's path parameters (path-detail): per cell its terms by how positions arrive, its commonest paths
    from the root, and what its cells change in the model's predictions on its positions (a router: where it sends
    them, as a from-to matrix)."""
    d = res.get("pathdetail")
    v = d["nodes"].get(i) if d else None
    if not v:
        return ""
    n = v["positions"]
    if not n:
        return "<p class=note>No positions reached it on these games.</p>"
    nodes, k, kind, leaf = res["nodes"], d["branch_factor"], v["kind"], v["leaf"]
    children = [] if leaf else nodes[i]["children"]

    def lnc(t):  # a router's term as centred log shifts per child (CNBP: log factors; LR: weights)
        xs = [math.log(max(x, 1e-300)) for x in t] if kind == "cnbp" else list(t)
        c = sum(xs) / len(xs)
        return [x - c for x in xs]

    def leaf_lo(t):  # a leaf's term as a white-minus-black log-odds offset (probit: the latent's mean)
        if kind == "cnbp":
            return math.log(max(t[0], 1e-300)) - math.log(max(t[1], 1e-300))
        return t[0] if kind == "probit" else t[0] - t[1]

    # The terms by arrival, one table per cell.
    if leaf and kind == "probit":
        heads = ("<th data-tip='the cell weight’s mean, added to the latent (positive favours white)'>Mean</th>"
                 "<th data-tip='the cell weight’s standard deviation (its variance is added to the latent’s)'>SD</th>"
                 "<th data-tip='mean change the cell makes to the leaf’s white-minus-black log-odds on these "
                 "positions (with it minus without it, the other cells kept)'>W−B effect</th>")
    elif leaf:
        heads = ("".join(f"<th data-tip='{t}'>{h}</th>" for h, t in (
            ("White", "the cell’s term in the white-win log-score"), ("Black", "in the black-win log-score"),
            ("Draw", "in the draw log-score")))
            + "<th data-tip='white minus black: what the cell adds to the leaf’s white-minus-black log-odds, the "
              "same for every position'>W−B</th>")
    else:
        heads = ("".join(f"<th data-tip='the cell’s term in this child’s log routing score, centred over the "
                         f"children (a shift common to all changes nothing)'>→ {pd_lab(nodes, c)}</th>"
                         for c in children)
                 + "<th data-tip='of these positions, the share it routes to another child without this cell (the "
                   "other cells kept)'>Moved without it</th>")
    tables = []
    for cell in v["cells"]:
        s_, ms = cell["set"], cell["members"]
        central = next((r for r in res["depth"] if nodes[r]["kind"] == "branch" and nodes[r]["children"] == ms), None)
        rows = []
        for val, r in enumerate(cell["rows"]):
            t = r["term"]
            arrive = f"not via set {s_}" if val == 0 else f"via {pd_lab(nodes, ms[val - 1])}"
            muted = " class=muted" if not r["n"] else ""
            if leaf and kind == "probit":
                vals = [fnum(t[0], 4), fnum(math.sqrt(max(t[1], 0.0)), 4), fnum(r["eff"] / r["n"], 3) if r["n"] else "–"]
            elif leaf:
                ws = [math.log(max(x, 1e-300)) for x in t] if kind == "cnbp" else t
                vals = [fnum(x, 3) for x in ws] + [fnum(leaf_lo(t), 3)]
            else:
                vals = [fnum(x, 2) for x in lnc(t)] + [pct(r["eff"] / r["n"], 1) if r["n"] else "–"]
            rows.append(f"<tr{muted}><td class='l nw'>{arrive}</td><td>{pct(r['n'] / n, 1)}</td>"
                        + "".join(f"<td>{x}</td>" for x in vals) + "</tr>")
        cap = (f"Set {s_}" + (f", the split of {pd_lab(nodes, central)}" if central is not None else "")
               + f": {', '.join(pd_lab(nodes, x) for x in ms)}")
        tables.append(f"<div><p class=small style='margin:4px 0'>{cap}</p><table><tr><th class=l>Arrives</th>"
                      f"<th data-tip='of the node’s positions'>Positions</th>{heads}</tr>{''.join(rows)}</table></div>")
    # The commonest paths.
    prow = []
    for p_ in v["paths"]:
        t = p_["term"]
        chain = " → ".join(pd_lab(nodes, x) for x in p_["nodes"]) if p_["nodes"] else esc(p_["values"])
        if leaf:
            off = fnum(leaf_lo(t), 4 if kind == "probit" else 3)
            eff = fnum(p_["eff"] / p_["n"], 3)
        else:
            off = " · ".join(f"{pd_lab(nodes, c)} {fnum(x, 2)}" for c, x in zip(children, lnc(t)))
            eff = pct(p_["eff"] / p_["n"], 1)
        prow.append(f"<tr><td class='l nw'>{chain}</td><td>{pct(p_['n'] / n, 1)}</td><td class='{'nw' if leaf else 'l nw'}'>{off}</td>"
                    f"<td>{eff}</td></tr>")
    if v["rest"][0]:
        prow.append(f"<tr class=muted><td class=l>{v['rest'][0]} other path{'s' if v['rest'][0] != 1 else ''}</td>"
                    f"<td>{pct(v['rest'][1] / n, 1)}</td><td></td><td></td></tr>")
    if leaf:
        oh = ("<th data-tip='the latent’s mean shift from all the cells on this path (positive favours white)'>"
              "Latent shift</th>" if kind == "probit" else
              "<th data-tip='what all the cells on this path add to the white-minus-black log-odds'>W−B offset</th>")
        eh = ("<th data-tip='mean change all the cells make to the white-minus-black log-odds on this path’s "
              "positions, against the leaf’s classifier alone'>W−B effect</th>")
    else:
        oh = ("<th class=l data-tip='all the cells’ terms on this path in each child’s log routing score, centred'>"
              "Shift per child</th>")
        eh = "<th data-tip='of this path’s positions, the share routed elsewhere than with its cells neutral'>Moved</th>"
    paths = (f"<table><tr><th class=l>Path from the root</th><th>Positions</th>{oh}{eh}</tr>{''.join(prow)}</table>")
    # The payoff.
    p = v["pay"]
    ll, ll0, se = pd_mean_se(p)
    wl = (f"; winner log-loss {fnum(p['wll'] / p['dec'], 4)} against {fnum(p['wll0'] / p['dec'], 4)}"
          if p["dec"] else "")
    if leaf:
        pay = (f"On its {n:,} positions the log-loss is {fnum(ll, 4)} with its cells and {fnum(ll0, 4)} with its "
               f"classifier alone: difference {pd_d(ll - ll0)} ± {pd_d(se)} SE{wl}.")
        ft = ""
    else:
        mv = p["moved"]
        pay = (f"With its cells it sends {mv:,} of its {n:,} positions ({pct(mv / n, 1)}) to another child than "
               f"its classifier alone would"
               + (f"; on those the model's log-loss is {fnum(p['moved_ll'] / mv, 4)} with its cells and "
                  f"{fnum(p['moved_ll0'] / mv, 4)} with its neutral choice (routing on from that child as play would)"
                  if mv else "")
               + f". Over all its positions {fnum(ll, 4)} against {fnum(ll0, 4)}: difference {pd_d(ll - ll0)} ± "
               f"{pd_d(se)} SE{wl}.")
        m_ = v["from_to"]
        head = "".join(f"<th>{pd_lab(nodes, c)}</th>" for c in children)
        body = "".join(
            f"<tr><td class=l>{pd_lab(nodes, children[a])}</td>"
            + "".join(f"<td{' class=muted' if a == b else ''}>{'<b>' if a != b and m_[a * k + b] else ''}"
                      f"{pct(m_[a * k + b] / n, 1)}{'</b>' if a != b and m_[a * k + b] else ''}</td>"
                      for b in range(len(children))) + "</tr>"
            for a in range(len(children)))
        ft = (f"<div><p class=small style='margin:4px 0'>Its choice: neutral (rows) → with its cells (columns), share "
              f"of its positions</p><table><tr><th class=l>Neutral</th>{head}</tr>{body}</table></div>")
    return (f"<h4>Path parameters</h4><p class=note>{pay} Difference: with minus without, paired over positions "
            f"(below zero the cells help; every other node keeps its cells).</p>"
            f"<div class=two>{''.join(tables)}{ft}</div>"
            f"<p class=small style='margin:8px 0 4px'>Its commonest paths ({len(v['paths'])} shown"
            + (f" of {len(v['paths']) + v['rest'][0]}" if v["rest"][0] else "") + ")</p>"
            f"<div class=scroll>{paths}</div>")


def router_section(m, res, i, since, hist, flat_name):
    nodes, N = res["nodes"], max(res["N"], 1)
    nd, ri = nodes[i], res["rinfo"][i]
    na = res["node_a"].get(i)
    n = na[A_N] if na else 0
    kind = {"cnbp": "naive-Bayes", "lr": "logistic-regression", "tan": "tree-augmented naive-Bayes"}.get(
        nd["cl"]["kind"], nd["cl"]["kind"])
    mem = nd.get("member")
    sets = res["model"]["sets"]
    central = next((s for s, ms in enumerate(sets) if nd["children"] == ms), None)
    parents = sorted({p for p, _ in nd["parents"] if p in res["depth"]})
    head = (f"<h3>Router {i}{' · the root' if i == 0 else ''} <span class=small>· {kind} router · since v"
            f"{since.get(i, '?')}{f' · copy {mem[1]} of sibling set {mem[0]}' if mem else ''}"
            f"{f' · central router of sibling set {central}' if central is not None else ''}</span></h3>")
    sub = [f"{pct(n / N, 1)} of all positions ({int(n):,})"]
    if parents:
        arr = res["arrivals"].get(i)
        if arr and len(arr) > 1:
            sub.append("arrive from " + ", ".join(f"<a href='#node-{p}'>R{p}</a> child {k} {pct(x / n)}"
                                                  for (p, k), x in arr.most_common()))
        else:
            sub.append("under " + ", ".join(f"<a href='#node-{p}'>R{p}</a>" for p in parents))
    if ri["margin_med"] is not None:
        sub.append(f"routing margin (its top child's score minus the next) median {fnum(ri['margin_med'], 2)}, "
                   f"below 0.1 in {pct(ri['close'])} of positions")
    rows = []
    for k, c in enumerate(nd["children"]):
        ea = res["edge_a"].get((i, k))
        g = region(ea, res["edge_ply"].get((i, k), collections.Counter())) if ea else None
        cn = ea[A_N] if ea else 0
        lab = f"{'R' if nodes[c]['kind'] == 'branch' else 'L'}{c}"
        rows.append(f"<tr><td class=l><a href='#node-{c}'>{lab}</a></td><td>{pct(cn / n if n else 0)}</td>"
                    f"<td>{pct(cn / N, 1)}</td>" + region_cells(g) + loss_cells(g, res["flat"]) + "</tr>")
    table = ("<div class=scroll><table><tr><th class=l>Child</th><th data-tip='of the positions through this "
             "router'>Share</th><th>Of all</th>" + REGION_HEADS + loss_heads(res["flat"]) + "</tr>" + "".join(rows)
             + "</table></div>")
    frows, nbtop = feat_rows(ri)
    feats = ("<table><tr><th class=l>Feature</th><th data-tip='mutual information (nats) between the feature and "
             "the child the router picked, on the positions it routed'>Routed apart by</th>"
             + ("<th data-tip='the same between the feature and the child, under its own naive-Bayes counts: what "
                "its training targets were'>Its counts</th><th data-tip='the feature's rank under its counts'>Rank"
                "</th>" if ri["nb"] else "") + "</tr>" + frows + "</table>")
    tells = []
    for k, c in enumerate(nd["children"]):
        t = ri["tells"][k][:7]
        if not t:
            continue
        chips = "".join(f"<span class=chip data-tip='{esc(f'{pct(pa, 1)} of its positions against {pct(pb, 1)} of the router’s other positions')}'>"
                        f"<b>{esc(valname(f, v))}</b> <span class='{'up' if d > 0 else 'dn'}'>{100 * pa:.0f}% vs "
                        f"{100 * pb:.0f}%</span></span>" for d, f, v, pa, pb in t)
        tells.append(f"<div class=small>{'R' if nodes[c]['kind'] == 'branch' else 'L'}{c}: </div><div class=chips>{chips}</div>")
    learned = ri["learned"] is not None
    hm = (f"<details data-r='{i}'><summary>Piece-square heat maps: each child's pieces against the router's other "
          f"children{' (where positions went, or what its counts learned)' if learned else ''}</summary>"
          f"<div class=ctl><span class=seg data-role=child></span>"
          + ("<span class=seg data-role=src></span>" if learned else "")
          + "</div><div class=boards></div><div class=scale></div><p class=cap></p></details>")
    win = ""
    if hist:
        w = hist["windows"].get(i)
        if w:
            rises = [b[1] - a[1] for a, b in zip(w["closed"], w["closed"][1:])]
            z = flags_of(train_args(LC)).get("--router-split-z") if m["prefix"] == "bdd" else None
            sv, se = res["state_se"] or (None, {})
            se = se.get(i, (None, None))[1] if se.get(i, (None,))[0] == "branch" else None
            est = ""
            if rises and z and se:  # the test: rise > z * sqrt(SE_recent^2 + SE_previous^2); one SE for both
                t = [x / (math.sqrt(2) * se) for x in (rises[-1], max(rises))]
                est = (f" (about {t[0]:.1f} and {t[1]:.1f} SE of the difference: an estimate, with the SE "
                       f"{fnum(se, 5)} of its last window in the v{sv} .state taken for every window, as model files "
                       "keep only the means)")
            win = (f"<h4>Error windows</h4><div class=two><div>{window_svg(w, 'routing')}</div><div class=note>"
                   f"{len(w['closed'])} windows closed since v{since.get(i, '?')}"
                   + (f"; the last change {fnum(rises[-1], 4)}, the largest rise {fnum(max(rises), 4)}{est}"
                      if rises else "")
                   + (f". A router split needs a rise of more than {z} SE of the difference (which model files "
                      "don't keep) and a window mean at least the routers' mean." if z else
                      ". Routers of trees don't split.") + "</div></div>")
    ps = ""
    if res["pathstats"] and i in res["pathstats"]["per"]:
        d = res["pathstats"]["per"][i]
        cells = [c for c in d["cells"].split(",") if c.strip()]
        ps = (f"<h4>Path context</h4><p class=note>{len(cells)} cell{'s' if len(cells) != 1 else ''}"
              + (f" (sibling sets {d['cells']})" if cells else "")
              + (f": with them it routes {pct(float(d.get('rerouted', 0)), 1)} of its positions elsewhere than "
                 "with its cells neutral." if cells else ".") + "</p>" + pd_section(res, i))
    return (f"<section class='node router' id='node-{i}'>{head}<p class=sub>{'; '.join(sub)}</p>{table}"
            f"<h4>What tells the children apart</h4><div class=two><div>{feats}</div><div style='flex:1;min-width:360px'>"
            f"{''.join(tells)}</div></div>{hm}{win}{ps}</section>")


def leaf_section(m, res, i, since, hist):
    nodes, N = res["nodes"], max(res["N"], 1)
    nd, li = nodes[i], res["linfo"][i]
    na = res["node_a"].get(i)
    n = na[A_N] if na else 0
    g = region(na, res["node_ply"].get(i, collections.Counter())) if na else None
    kind = {"probit": "probit", "lr": "logistic-regression", "cnbp": "naive-Bayes"}.get(nd["cl"]["kind"], nd["cl"]["kind"])
    parents = sorted({p for p, _ in nd["parents"] if p in res["depth"]})
    sub = [f"{pct(n / N, 1)} of all positions ({int(n):,})"]
    arr = res["arrivals"].get(i)
    if arr and len(arr) > 1:
        sub.append("arrive from " + ", ".join(f"<a href='#node-{p}'>R{p}</a> child {k} {pct(x / n)}"
                                              for (p, k), x in arr.most_common()))
    elif parents:
        sub.append("under " + ", ".join(f"<a href='#node-{p}'>R{p}</a>" for p in parents))
    head = f"<h3>Leaf {i} <span class=small>· {kind} leaf · since v{since.get(i, '?')}</span></h3>"
    table = ("<div class=scroll><table><tr>" + REGION_HEADS + loss_heads(res["flat"]) + "</tr><tr>" + region_cells(g)
             + loss_cells(g, res["flat"]) + "</tr></table></div>")
    pv = li["pv"]
    pvt = ""
    if pv:
        unit = "log-odds" if nd["cl"]["kind"] == "lr" else "latent"
        rows = []
        for t in range(6):
            w, b = pv[t], pv[t + 6]
            rw = (w / pv[0]) if w is not None and pv[0] else None
            rb = (b / pv[6]) if b is not None and pv[6] else None
            rows.append(f"<tr><td class=l>{'Pawn Knight Bishop Rook Queen King'.split()[t]}</td><td>{fnum(w, 2)}</td>"
                        f"<td>{fnum(-b if b is not None else None, 2)}</td><td>{fnum(rw, 1) if t else '1'}</td>"
                        f"<td>{fnum(rb, 1) if t else '1'}</td></tr>")
        pvt = ("<table><tr><th class=l>Piece</th><th data-tip='its mean weight against an empty square, over the "
               f"squares it stood on in this leaf's positions ({unit}, in white's favour)'>White's</th><th "
               "data-tip='the same for black's piece, sign flipped (in black's favour)'>Black's</th><th "
               "data-tip='white's, in its pawns'>÷ pawn</th><th>÷ pawn</th></tr>" + "".join(rows) + "</table>")
    sims = []
    for j in sorted((j for j in res["linfo"] if j != i and j in res["depth"]),
                    key=lambda j: -(res["sim"].get((i, j)) or -2)):
        d = res["dis"].get((i, j))
        sib = any(p in parents for p, _ in nodes[j]["parents"])
        sims.append(f"<tr><td class=l><a href='#node-{j}'>L{j}</a>{' <span class=muted>sibling</span>' if sib else ''}"
                    f"</td><td>{fnum(res['sim'].get((i, j)), 2)}</td><td>{pct(d[0], 1) if d else '–'}</td>"
                    f"<td>{fnum(d[1], 2) if d else '–'}</td></tr>")
    simt = ("<table><tr><th class=l>Leaf</th><th data-tip='correlation of the two leaves’ own white-minus-black "
            "log-odds (each without path cells), on a sample of all positions'>Correlation</th><th data-tip='on this "
            "leaf’s positions: how often the other leaf picks the other winner'>Other winner</th><th data-tip='on this "
            "leaf’s positions: the mean absolute difference of the two log-odds'>Mean |gap|</th></tr>"
            + "".join(sims) + "</table>") if sims else ""
    boards = ""
    if li["pst"]:
        boards = (f"<details data-l='{i}' open><summary>Weight boards: piece on a square against the square empty"
                  f"</summary><div class=ctl><span class=seg data-role=q></span><span class=seg data-role=view></span>"
                  "</div><div class=boards></div><div class=scale></div><p class=cap></p></details>")
    win = ""
    if hist and hist["windows"].get(i):
        w = hist["windows"][i]
        model = res["model"]
        leaf_splits = (sum(1 for x in nodes if x["kind"] == "leaf") - 1) // max(model["branch_factor"] - 1, 1)
        full = leaf_splits >= model["split_limit"]
        openw = (f" The open window: {big(w['open_n'])} positions, mean error {fnum(w['open_mean'], 4)}."
                 if w["open_mean"] is not None else "")
        win = (f"<h4>Error windows</h4><div class=two><div>{window_svg(w, 'prediction')}</div><div class=note>"
               f"{len(w['closed'])} windows closed since v{since.get(i, '?')}.{openw} A leaf splits when a window's "
               "mean error is not lower than the window before by more than its margin (z × standard error) and is "
               "at least the model's mean"
               + ("; with the split limit reached, leaves close no windows." if full else ".") + "</div></div>")
    ps = ""
    if res["pathstats"] and i in res["pathstats"]["per"]:
        d = res["pathstats"]["per"][i]
        cells = [c for c in d["cells"].split(",") if c.strip()]
        k = res["model"]["branch_factor"] + 1
        npar = len(cells) * k * (1 if nd["cl"]["kind"] == "probit" else 3)
        ps = (f"<h4>Path context</h4><p class=note>{len(cells)} cells (sibling sets {esc(d['cells']) or '–'}), "
              f"{npar} parameters. {pct(float(d.get('nonhome', 0)), 1)} of its positions came through a copy that "
              f"routed elsewhere than its own number's child; the cells change its white-minus-black log-odds by "
              f"{fnum(float(d.get('logodds-shift', 0)), 3)} on average (spread over its positions "
              f"{fnum(float(d.get('path-spread', 0)), 3)}); the cells' weights: norm {fnum(float(d.get('table-l2', 0)), 3)}, "
              f"largest {fnum(float(d.get('table-max', 0)), 3)}.</p>" + pd_section(res, i))
    return (f"<section class='node leaf' id='node-{i}'>{head}<p class=sub>{'; '.join(sub)}</p>{table}"
            f"<div class=two><div><h4>Effective piece values</h4>{pvt or '<p class=note>–</p>'}</div>"
            f"<div><h4>Against the other leaves</h4>{simt}</div></div>{boards}{win}{ps}</section>")


LC = {}


def findings(m, res, since, flat_name):
    out, nodes, N = [], res["nodes"], max(res["N"], 1)
    root = res["rinfo"].get(0)
    if root:
        top = sorted(range(NF), key=lambda f: -root["emp"][f])[:4]
        kids = []
        for k, c in enumerate(nodes[0]["children"]):
            ea = res["edge_a"].get((0, k))
            g = region(ea, res["edge_ply"].get((0, k), collections.Counter())) if ea else None
            kids.append(f"{'R' if nodes[c]['kind'] == 'branch' else 'L'}{c} {pct(ea[A_N] / N if ea else 0)}: "
                        f"{short_region(g)}")
        out.append(f"The root routes by {', '.join(featname(f) for f in top)} (most mutual information with its "
                   f"choice): " + "; ".join(kids) + ".")
    t = region(res["total"], res["total_ply"])
    if res["flat"] and t:
        out.append(f"On these positions the model's log-loss is {fnum(t['ll'])} against {fnum(t['fll'])} for the flat "
                   f"{esc(flat_name)}; of who won, {fnum(t['wll'])} against {fnum(t['fwll'])}.")
        gains = []
        for l in res["linfo"]:
            na = res["node_a"].get(l)
            if na and na[A_N] >= 0.005 * N:
                g = region(na, res["node_ply"][l])
                gains.append((g["ll"] - g["fll"], l, g))
        if gains:
            gains.sort()
            d, l, g = gains[0]
            txt = f"The largest gain over the flat model is in L{l} ({short_region(g)}): {fnum(d)} per position"
            worse = [(d, l) for d, l, _ in gains if d > 0]
            txt += ("; " + ", ".join(f"L{l} {fnum(d)}" for d, l in worse) + " predict worse than the flat model"
                    if worse else "; every leaf with 0.5% of the positions or more predicts better than the flat one")
            out.append(txt + ".")
    weak = [(r, ri) for r, ri in res["rinfo"].items() if r in res["depth"] and ri["close"] is not None
            and (ri["close"] > 0.25 or (ri["margin_med"] or 1) < 0.2)]
    if weak:
        out.append("Routers that hardly decide: " + ", ".join(
            f"R{r} (median margin {fnum(ri['margin_med'], 2)}, {pct(ri['close'])} close calls)" for r, ri in weak) + ".")
    unused = []
    for r, ri in res["rinfo"].items():
        tot = sum(ri["n_k"])
        if r in res["depth"] and tot:
            unused += [f"R{r}→{nodes[r]['children'][k]} {pct(x / tot, 1)}" for k, x in enumerate(ri["n_k"])
                       if x < 0.005 * tot]
    if unused:
        out.append("Children that get almost nothing (under 0.5% of their router's positions): " + ", ".join(unused) + ".")
    if res["model"]["sets"]:
        ps = res["pathstats"]
        kv = ps["overall"].split()[1:] if ps else []
        ov = {kv[j]: kv[j + 1] for j in range(0, len(kv) - 1, 2)}
        out.append(f"{len(res['model']['sets'])} router splits made sibling sets ("
                   + "; ".join(f"set {s}: nodes {', '.join(map(str, ms))}" for s, ms in enumerate(res["model"]["sets"]))
                   + ")" + (f". With their path cells the routers send {pct(float(ov.get('rerouted-at-some-router', 0)), 1)}"
                            f" of positions down another path than with the cells neutral, and the leaves' cells move "
                            f"their white-minus-black log-odds by {fnum(float(ov.get('logodds-shift', 0)), 3)} on average."
                            if ov else "."))
    return out


JS_PAGE = r"""
const DATA = JSON.parse(document.getElementById('tv-data').textContent);
const tip = document.getElementById('tip');
document.addEventListener('mousemove', e => {
  const t = e.target.closest('[data-tip]');
  if (!t) { tip.style.display = 'none'; return; }
  tip.textContent = t.getAttribute('data-tip'); tip.style.display = 'block';
  const r = tip.getBoundingClientRect();
  let x = e.clientX + 14, y = e.clientY + 14;
  if (x + r.width > innerWidth - 8) x = e.clientX - r.width - 14;
  if (y + r.height > innerHeight - 8) y = e.clientY - r.height - 14;
  tip.style.left = x + 'px'; tip.style.top = y + 'px';
});
const root = document.documentElement, tbtn = document.getElementById('theme');
const modes = ['auto', 'light', 'dark'];
function setTheme(m) { if (m === 'auto') root.removeAttribute('data-theme'); else root.setAttribute('data-theme', m);
  tbtn.textContent = 'Theme: ' + m; tbtn.dataset.mode = m; }
tbtn.addEventListener('click', () => setTheme(modes[(modes.indexOf(tbtn.dataset.mode) + 1) % 3]));
setTheme('auto');
const PN = ['white pawn','white knight','white bishop','white rook','white queen','white king',
            'black pawn','black knight','black bishop','black rook','black queen','black king'];
const PS = ['P','N','B','R','Q','K','p','n','b','r','q','k'];
const sqn = s => 'abcdefgh'[s % 8] + (Math.floor(s / 8) + 1);
const fmt = v => v === null || v === undefined ? '–' : v.toFixed(2).replace('-', '−');
function boardsHTML(vals, D, tipf) {
  let h = '';
  for (let p = 0; p < 12; p++) {
    h += '<div class="board"><div class="t">' + PN[p] + ' (' + PS[p] + ')</div><div class="bgrid">';
    for (let rk = 7; rk >= 0; rk--) {
      h += '<div class="rk">' + (rk + 1) + '</div>';
      for (let f = 0; f < 8; f++) {
        const s = rk * 8 + f, v = vals[p * 64 + s];
        if (v === null) { h += '<div class="c na" data-tip="' + sqn(s) + ', ' + PN[p] + ': too few positions"></div>'; continue; }
        const k = Math.max(-5, Math.min(5, Math.round(v / D * 5))) + 5;
        h += '<div class="c" style="background:var(--d' + k + ')" data-tip="' + tipf(p, s, v) + '"></div>';
      }
    }
    h += '<div></div>'; for (let f = 0; f < 8; f++) h += '<div class="fl">' + 'abcdefgh'[f] + '</div>';
    h += '</div></div>';
  }
  return h;
}
function scaleHTML(D) {
  let sh = '';
  for (let k = 0; k <= 10; k++) { const lo = (k - 5) / 5 * D; sh += '<div><div class="b" style="background:var(--d' + k + ')"></div>' + (k % 2 === 1 ? '' : fmt(lo)) + '</div>'; }
  return sh;
}
function seg(el, label, opts, cur, on) {
  el.innerHTML = '<span class="lab">' + label + '</span>' + opts.map(o => '<button data-v="' + o[0] + '" aria-pressed="' + (String(cur) === String(o[0])) + '">' + o[1] + '</button>').join('');
  el.querySelectorAll('button').forEach(b => b.addEventListener('click', () => {
    el.querySelectorAll('button').forEach(x => x.setAttribute('aria-pressed', x === b)); on(b.dataset.v);
  }));
}
function routerBoards(d) {
  const r = DATA.routers[d.dataset.r], st = {child: 0, src: 'emp'};
  const draw = () => {
    const vals = r[st.src][st.child], D = r[st.src + 'D'];
    const c = r.children[st.child];
    d.querySelector('.boards').innerHTML = boardsHTML(vals, D, (p, s, v) => sqn(s) + ', ' + PN[p] + '\n' +
      (st.src === 'emp' ? 'log of its share of node ' + c + '’s positions over its share of the router’s others: ' :
       'its counts: log P(it | child ' + c + ') − log P(it | the other children): ') + fmt(v));
    d.querySelector('.scale').innerHTML = scaleHTML(D);
    d.querySelector('.cap').textContent = (st.src === 'emp'
      ? 'Where node ' + c + '’s positions have each piece, against the router’s other positions: blue, more often in this child; red, less often (log ratio of the shares, from the positions routed). Hatched: under 30 positions with that piece there.'
      : 'What the router’s naive-Bayes counts (its training targets) tie to child ' + c + ': blue, evidence for it; red, against it (log-likelihood ratio against the other children).')
      + ' The scale ends at the 98th percentile of the values (' + fmt(D) + ').';
  };
  seg(d.querySelector('[data-role=child]'), 'Child', r.children.map((c, k) => [k, (r.kinds[k] === 'leaf' ? 'L' : 'R') + c]), 0, v => { st.child = +v; draw(); });
  const sel = d.querySelector('[data-role=src]');
  if (sel) seg(sel, 'From', [['emp', 'Positions routed'], ['learned', 'Its counts']], 'emp', v => { st.src = v; draw(); });
  draw();
}
function leafBoards(d) {
  const l = DATA.leaves[d.dataset.l], st = {q: 'wl', view: 'value'};
  const draw = () => {
    const raw = l[st.q], mean = DATA.meanleaf[st.q];
    const vals = st.view === 'value' ? raw : raw.map((v, j) => v === null || mean[j] === null ? null : +(v - mean[j]).toFixed(3));
    const D = DATA.leafD[st.q + st.view];
    d.querySelector('.boards').innerHTML = boardsHTML(vals, D, (p, s, v) => sqn(s) + ', ' + PN[p] + '\nagainst an empty ' + sqn(s) + ': ' + fmt(raw[p * 64 + s]) + (st.view === 'diff' ? '\nthe mean leaf: ' + fmt(mean[p * 64 + s]) + '\ndifference: ' + fmt(v) : ''));
    d.querySelector('.scale').innerHTML = scaleHTML(D);
    const what = st.q === 'wl' ? DATA.unit : 'draw log-odds minus the mean of the two wins’';
    d.querySelector('.cap').textContent = what + ', ' + (st.view === 'value' ? 'against the same square empty' : 'minus the mean leaf (weighted by the leaves’ positions)') + '. Blue favours white (or a draw), red black (or a decision). The scale ends at the 98th percentile of all leaves’ values (' + fmt(D) + ').';
  };
  const qs = d.querySelector('[data-role=q]');
  if (l.dr) seg(qs, 'Board', [['wl', 'Win − loss'], ['dr', 'Draw']], 'wl', v => { st.q = v; draw(); }); else qs.remove();
  seg(d.querySelector('[data-role=view]'), 'Show', [['value', 'Value'], ['diff', 'Minus the mean leaf']], 'value', v => { st.view = v; draw(); });
  draw();
}
document.querySelectorAll('details[data-r]').forEach(d => { const go = () => { if (d.open && !d.dataset.drawn) { d.dataset.drawn = 1; routerBoards(d); } }; d.addEventListener('toggle', go); go(); });
document.querySelectorAll('details[data-l]').forEach(d => { const go = () => { if (d.open && !d.dataset.drawn) { d.dataset.drawn = 1; leafBoards(d); } }; d.addEventListener('toggle', go); go(); });
"""


def render(m, res, cfg, lc, flat_name, games_note, secs):
    global LC
    LC = lc
    nodes, model, N = res["nodes"], res["model"], res["N"]
    hist = res["history"] or {}
    since = hist.get("since", {})
    reach = sorted(res["depth"], key=lambda x: (res["depth"][x], x))
    nleaves = sum(1 for i in reach if nodes[i]["kind"] == "leaf")
    nrouters = len(reach) - nleaves
    is_dag = m["prefix"] == "bdd"
    where = (f"experiment {m['name']}" if m["kind"] == "experiment" else ("the global league" if m["kind"] == "global"
                                                                         else os.path.dirname(m["path"])))
    words = recipe_words(train_args(lc)) if lc else []
    field = collections.Counter(l.get("recipe") or l.get("archetype") or l["line"] for l in cfg.get("lines", [])
                                if not l.get("observer"))
    # an experiment's observers (2026-11-12): they train on the field's games and never play
    observers = collections.Counter(l.get("recipe") or l.get("archetype") or l["line"] for l in cfg.get("lines", [])
                                    if l.get("observer"))
    date = time.strftime("%Y-%m-%d %H:%M", time.localtime(os.path.getmtime(m["path"])))
    t = region(res["total"], res["total_ply"])
    title = f"{short_line(m['line'], cfg)} @ v{m['version']}"
    intro = (f"A {'DAG' if is_dag else 'tree'} of {len(reach)} nodes: {nrouters} router{'s' if nrouters != 1 else ''} "
             f"and {nleaves} lea{'ves' if nleaves != 1 else 'f'}"
             + (f", {len(model['sets'])} sibling set{'s' if len(model['sets']) != 1 else ''} from router splits"
                if model["sets"] else "") + f"; {model['split_number']} splits in all. ")
    if words:
        intro += "Recipe: " + "; ".join(words) + ". "
    intro += (f"Version {m['version']} of line {esc(m['line'])} in {esc(where)}, saved {date}"
              + (f"; trained by {esc(os.path.basename(lc.get('train_binary') or ''))}" if lc.get("train_binary") else "")
              + ("; an observer: it trains on the field's games each cycle and never plays" if lc.get("observer")
                 else "") + ". ")
    if field:
        intro += ("The field" + (f" (entries at ply {cfg.get('entry_ply')})" if cfg.get("entry_ply") else "") + ": "
                  + ", ".join(f"{esc(k)}{f' ×{v}' if v > 1 else ''}" for k, v in field.items()) + ". ")
    if observers:
        intro += ("Observers (they train on the field's games and never play): "
                  + ", ".join(f"{esc(k)}{f' ×{v}' if v > 1 else ''}" for k, v in observers.items()) + ". ")
    intro += (f"Read on {N:,} positions of {res['ngames']:,} games ({esc(games_note)})"
              + (f"; the flat model is {esc(flat_name)}" if flat_name else "") + ".")
    tiles = [("Positions", f"{N:,}", f"{res['ngames']:,} games"),
             ("Nodes", f"{len(reach)}", f"{nrouters} routers, {nleaves} leaves"
              + (f", {len(model['sets'])} sibling sets" if model["sets"] else ""))]
    if t:
        tiles.append(("Log-loss", fnum(t["ll"]), f"flat {fnum(t['fll'])}" if res["flat"] else "no flat comparison"))
        if t["wll"] is not None:
            tiles.append(("Winner log-loss", fnum(t["wll"]), f"flat {fnum(t['fwll'])}" if res["flat"] else ""))
    if hist:
        tiles.append(("Versions read", f"{hist['dated']}", f"of {hist['total']}, to date the nodes"))
    tile_html = "".join(f"<div class=tile><div class=k>{esc(a)}</div><div class=v>{esc(b)}</div><div class=s>{esc(c)}"
                        f"</div></div>" for a, b, c in tiles)
    fl = findings(m, res, since, flat_name)
    # board data
    routers_js, leaves_js = {}, {}
    for i in reach:
        nd = nodes[i]
        if nd["kind"] == "branch":
            ri = res["rinfo"][i]
            emp = [board_vals(ri["emp_llr"][k], True) for k in range(len(nd["children"]))]
            r = dict(children=nd["children"], kinds=[nodes[c]["kind"] for c in nd["children"]], emp=emp, empD=domain(emp))
            if ri["learned"] is not None:
                ln = [board_vals(ri["learned"][k], True) for k in range(len(nd["children"]))]
                r.update(learned=ln, learnedD=domain(ln))
            routers_js[i] = r
        else:
            li = res["linfo"][i]
            if li["pst"]:
                leaves_js[i] = dict(wl=board_vals(lambda p, s, P=li["pst"]: P[p][s], False),
                                    dr=board_vals(lambda p, s, P=li["pst_dr"]: P[p][s], False) if li["pst_dr"] else None)
    meanleaf, leafD = {}, {}
    for q in ("wl", "dr"):
        ws = [(res["node_a"][l][A_N], v[q]) for l, v in leaves_js.items() if v.get(q) and res["node_a"].get(l)]
        tw = sum(w for w, _ in ws)
        if tw:
            meanleaf[q] = [round(sum(w * v[j] for w, v in ws) / tw, 3) for j in range(768)]
            leafD[q + "value"] = domain([v for _, v in ws])
            leafD[q + "diff"] = domain([[a - b for a, b in zip(v, meanleaf[q])] for _, v in ws])
    unit = ("white-win minus black-win log-odds" if any(nodes[l]["cl"]["kind"] == "lr" for l in leaves_js)
            else "the latent's mean (positive favours white)")
    data = dict(routers=routers_js, leaves=leaves_js, meanleaf=meanleaf, leafD=leafD, unit=unit)
    blob = json.dumps(data, separators=(",", ":")).replace("</", "<\\/")
    sections = []
    for i in reach:
        if nodes[i]["kind"] == "branch":
            sections.append(router_section(m, res, i, since, hist, flat_name))
        else:
            sections.append(leaf_section(m, res, i, since, hist))
    ps = res["pathstats"]
    method = (
        "<h2>How this page was made</h2><p class=note>Routes and predictions are exact: "
        "<code>chessai-treeview node-dump</code> loads the model as play does (the same routing, path cells and "
        "probabilities as the league's binaries) and writes every position's path, leaf and probabilities; "
        "the page's figures sum them. A router's children: the positions it sent to each. <i>Routed apart by</i>: "
        "mutual information (nats) between a feature (a square's contents, side to move, castling rights, en "
        "passant) and the child picked. <i>Its counts</i>: the same under the router's own naive-Bayes counts, i.e. "
        "its training targets (for error routing, which child predicted best; for cluster routing, where it sent "
        "positions). Heat maps: log of a piece-square's share in a child's positions over its share in the router's "
        "other positions (30 positions or more), or the counts' log-likelihood ratio. Leaves: weight boards are the "
        "classifier's weights for a piece on a square against the same square empty (LR: white-win minus "
        "black-win log-odds, and the draw log-odds against the mean of the two wins'; probit: the latent's mean); "
        "effective piece values average them over the squares the piece stood on in the leaf's positions. "
        "Similarity: the leaves' own log-odds (without path cells) on a sample of a fifth of the positions. "
        "<i>Since</i>: the first version from which the node has had its role without a break (same kind, its "
        "training counters never falling; a router split restarts its central router), over "
        f"{hist.get('dated', 0)} of the line's {hist.get('total', 0)} versions. Error windows: the means of its "
        "closed training windows, as the split tests read them (their standard errors are not in model files). "
        "Phase: non-pawn material of both sides, full board ≥ 52, middlegame 26–51, endgame ≤ 25."
        + (" Path context: <code>chessai-treeview path-stats</code> on the same games." if ps else "")
        + (" Path parameters: <code>chessai-treeview2 path-detail</code> on the same games (its predictions with the "
           "cells are bit for bit the league binary's); “neutral” is a node's classifier alone, without its path "
           "table; dropping one cell leaves out that cell's term and keeps the others'. Standard errors are paired "
           "over positions, which share a game's result, so they flatter; the top line also gives one clustered "
           "by game." if res.get("pathdetail") else "")
        + f"</p><p class=small>{esc(m['path'])} · {N:,} positions · made {time.strftime('%Y-%m-%d %H:%M')} in "
        f"{secs:.0f} s by leagues/treeview.py</p>")
    legend = ("<div class=legend><span><span class=sw style='background:var(--q1)'></span>full board</span>"
              "<span><span class=sw style='background:var(--q3)'></span>middlegame</span><span><span class=sw "
              "style='background:var(--q5)'></span>endgame</span><span style='margin-left:12px'><span class=sw "
              "style='background:var(--res-w)'></span>white wins</span><span><span class=sw "
              "style='background:var(--res-b)'></span>black wins</span><span><span class=sw "
              "style='background:var(--res-d)'></span>draws</span></div>")
    body = (f"<main><div class=top><div><h1>{esc(title)}</h1><p class=sub>{esc(where)} · {esc(m['line'])} · node report"
            f"</p></div>"
            f"<button id=theme data-mode=auto>Theme: auto</button></div><p>{intro}</p>{pd_model_line(res)}"
            f"<div class=tiles>{tile_html}</div>"
            + (f"<ul class=findings>{''.join(f'<li>{x}</li>' for x in fl)}</ul>" if fl else "")
            + f"<h2>The graph</h2><p class=note>Boxes: R routers, L leaves; the bar and the first figure, the share of "
              f"positions; v, the version since which the node has had its role. Edges: thickness by share of all "
              f"positions, label by share of the router's. "
            + ("Green dashed frames: sibling sets (a router split's copies); a green badge: the number of parents of "
               "a shared node. " if is_dag else "") + "Click a node for its section.</p>"
            + f"<div class=graph>{graph_svg(res, since)}</div>"
            + (f"<h4>Growth</h4>{growth_svg(hist)}" if hist else "")
            + f"<h2>Nodes</h2>{legend}" + "".join(sections)
            + (f"<h2>Path context</h2><pre class=raw>{esc(ps['text'])}</pre>" if ps else "")
            + method + "</main>")
    return ("<!doctype html><html lang=en><head><meta charset=utf-8><meta name=viewport "
            "content='width=device-width,initial-scale=1'>"
            f"<title>{esc(title)} · nodes</title><style>{page_css()}</style></head><body>{body}<div id=tip></div>"
            f'<script type="application/json" id="tv-data">{blob}</script><script>{JS_PAGE}</script></body></html>')


def safe(s):
    return re.sub(r"[^A-Za-z0-9_.@+=-]", "_", s)


def line_parts(line):
    s = line[:-4] if line.endswith(".loc") else line
    rep = None
    if "#" in s:
        s, rep = s.rsplit("#", 1)
    toks = s.split("+")
    return toks[0], toks[1:], rep


def short_line(line, cfg):
    """A line's name without the options every line of its recipe in the league shares, its replicate as -rN:
    D-P-Ne-ctx+split-limit=4+...+router-split-z=30+router-split-limit=4#1.loc -> D-P-Ne-ctx+router-split-z=30-r1."""
    base, opts, rep = line_parts(line)
    peers = [set(line_parts(l["line"])[1]) for l in cfg.get("lines", []) if line_parts(l["line"])[0] == base]
    common = set.intersection(*peers) if peers else set()
    return base + "".join("+" + o for o in opts if o not in common) + (f"-r{rep}" if rep else "")


def default_out(m, cfg):
    pre = f"{m['name']}_" if m["kind"] == "experiment" else ""
    return os.path.join(OUTDIR, safe(f"{pre}{short_line(m['line'], cfg)}@{m['version']}") + ".html")


def main(argv=None):
    global QUIET, HISTORY_N, WAIT
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0], formatter_class=argparse.RawDescriptionHelpFormatter,
                                 epilog="\n".join(__doc__.split("\n")[2:]))
    ap.add_argument("model")
    ap.add_argument("--out")
    ap.add_argument("--games", nargs="+")
    ap.add_argument("--games-from")
    ap.add_argument("--positions", type=int, default=200000)
    ap.add_argument("--flat")
    ap.add_argument("--history", type=int, default=HISTORY_N)
    ap.add_argument("--wait", type=int, default=WAIT)
    ap.add_argument("--text", action="store_true")
    ap.add_argument("--quiet", action="store_true")
    a = ap.parse_args(argv)
    QUIET, HISTORY_N, WAIT = a.quiet, max(2, a.history), a.wait
    t0 = time.time()
    m = resolve(a.model)
    cfg, lc = league_info(m)
    if a.games:
        games = [os.path.abspath(g) for g in a.games]
        games_note = f"{len(games)} directories given"
    else:
        src = m["league"]
        if a.games_from:
            src = os.path.abspath(a.games_from) if os.sep in a.games_from else os.path.join(LEAGUES, "experiments", a.games_from)
        elif src and not round_source(src) and not iter_snapshots(os.path.join(CACHE, "games", os.path.basename(src))) \
                and not running(src):
            src = fallback_league(m, cfg, lc)
        if not src:
            sys.exit("no games: give --games or --games-from")
        games = [d for d, _ in snapshot_rounds(src, a.positions)]
        rn = sorted(int(os.path.basename(d)[5:]) for d in games)
        games_note = (f"{os.path.basename(src)}'s rounds {rn[0]}-{rn[-1]}" if rn else "") + \
                     ("" if src == m["league"] else " (the model's league has no games of its own)")
    if not games:
        sys.exit("no games found")
    flat = None if a.flat == "none" else (resolve_flat(a.flat) if a.flat else (default_flat(m, cfg, lc) if m["league"] else None))
    log(f"{a.model}: {m['path']}; games: {games_note}; flat: {flat[1] if flat else None}")
    res = analyse(m, games, flat)
    flat_name = flat[1] if flat else None
    if a.text:
        print(text_summary(m, res, flat_name))
    out = a.out or default_out(m, cfg)
    os.makedirs(os.path.dirname(os.path.abspath(out)), exist_ok=True)
    page = render(m, res, cfg, lc, flat_name, games_note, time.time() - t0)
    tmp = out + ".tmp"
    with open(tmp, "w") as f:
        f.write(page)
    os.replace(tmp, out)
    log(f"wrote {out} ({len(page) / 1e6:.2f} MB) in {time.time() - t0:.0f}s")
    return out


if __name__ == "__main__":
    main()
