#!/usr/bin/env python3
"""Calibrates the price formula and writes a new prices version: rows in leagues/prices.csv and a report,
leagues/prices/<version>.md. Re-runnable: each run adds a version (a version's rows never change).

    python3 leagues/calibrate.py SPEC.json                 measure, fit, write the version
    python3 leagues/calibrate.py SPEC.json --fit-only      refit from the measurements already in the spec's work dir
    python3 leagues/calibrate.py SPEC.json --measure-only  measure (what isn't measured yet) and stop
    python3 leagues/calibrate.py SPEC.json --check         fit; the report goes to the work dir, no version is added

The formula: each side of a game costs us_per_game plus its moves times us_per_ply, the constants of its
(context, kind, depth). Context `tournament` prices games the tournament binary plays (a reference is kind `random` at
its ply); context `self-play` prices the games a line's self-play binary plays and trains on (both sides are the
line's model at its self-play depth, so a round of G games and P plies costs 2 G us_per_game + P us_per_ply). The
per-game term stands for the opening's dearer plies (more legal moves to search than in an endgame): a per-ply
price alone underprices short games and overprices long ones (P0: 0.58 and 1.27 of the measured CPU).
The constants are fitted on measurements:
- tournament: per-game thread CPU (`tournament --game-cpu`) of a calibration tournament of the spec's players at
  plies 1 and 2 (models copied from run dirs, such as the live league's, or untrained ones made as the driver makes
  version 0), and of any league dirs the spec includes (their tour/game_cpu.csv). The model: a game's CPU = the sum
  over its sides of a(side) + moves x c(side), fitted by least squares weighted 1/predicted (iterated from
  1/measured; a term that comes out negative is held at 0 for that iteration), which keeps the sum of prices equal to
  the sum of measured CPU. A side with fewer than MIN_GAMES games or no positive per-ply term counts as unmeasured.
- self-play: short self-play runs (from scratch, or resumed from a copy of a run dir's last two versions), one
  thread each: generation CPU (the process's CPU split by the wall time of its phases: playing, training,
  labelling), as a ratio to the run's price at its kind's tournament constants (ply 1 for self-play depths 0 and 1,
  ply 2 deeper).
Constants the measurements don't cover are derived, and the report says how: a tournament constant of an unmeasured
kind from the mean of the measured kinds at that ply (weighted by moves); a self-play constant from the tournament's
of the same kind times the self-play/tournament ratio measured for that kind and depth, else the ratio pooled over
the kinds measured at that depth, else interpolated geometrically between measured depths of the same tournament ply
(a depth above every measured one is not derived). Self-play constants are written for depths 0-4 of every recipe
kind where they can be derived (a sweep may change the depth); a recipe's own self-play depth must be.

Spec (JSON):
  {"version": "P1", "note": "...", "work": DIR, "nice": 19, "threads": 2,
   "tournament": {"players": [{"name": N, "from": RUN_DIR, "version": V (default: the newest settled)}
                              | {"name": N, "model": PATH, "kind": K}
                              | {"name": N, "recipe": RECIPE}   (its untrained version 0)
                              ... each may give "heldout" (lr or cnbp; default: the kind's recipe's)],
                  "plies": [1, 2], "round_robins": 1, "seed": 1, "binary": PATH (default: the leagues' tournament
                  binary), "split": true (one round robin per ply), "repeats": 1 (the same round robins played
                  again, interleaved, to average the CPU noise; the report shows it)},
   "include": [LEAGUE_DIR, ...],
   "selfplay": [{"name": N, "model": RECIPES_MODEL, "games": G, "rounds": R, "from": RUN_DIR (optional: resume
                 from its newest settled version), "depth": D (optional: the model's self-play at --depth D)}],
   "selfplay_fit_stages": ["mature"] (optional: each kind and depth's self-play ratio is fitted on its runs of
                 these stages where it has some, and only checked on the others; default: all runs),
   "refresh_costs": true (optional: also append leagues/costs.csv rows "lr d0 jitter10" to "lr d4 jitter10" from
                 the version's lr self-play prices, so new leagues' self-play round sizes follow them)}
A tree version copied from a run dir (a kind that keeps states) is taken only with its own and its previous
version's states, all four files older than 30 s.
"""
import argparse
import datetime
import json
import os
import re
import shutil
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import leagues as lg  # noqa: E402

REPORTS = os.path.join(lg.HERE, "prices")
STAGES = ("untrained", "early", "mature")
MIN_GAMES = 20  # a tournament side with fewer games in the fit counts as unmeasured
SP_DEPTHS = range(5)  # the self-play depths priced for every recipe kind
LENGTHS = ((100, "short (under 100 plies)"), (300, "mid (100-299 plies)"), (10 ** 9, "long (300 plies or more)"))


# ---------------------------------------------------------------- inputs

def models_in(run_dir):
    """{kind: sorted versions} of a run dir's <kind><N>.model files."""
    out = {}
    for f in os.listdir(run_dir):
        m = re.fullmatch(r"([a-z]+)(\d+)\.model", f)
        if m:
            out.setdefault(m.group(1), []).append(int(m.group(2)))
    return {k: sorted(v) for k, v in out.items()}


def settled_version(run_dir, version=None, min_age=180):
    """(kind, N) of a run dir: N given, or its newest version whose files are older than min_age s (not being
    written), with version N-1 present (self-play's --resume needs the opponent) and, for a kind that keeps states,
    the states of both."""
    kinds = models_in(run_dir)
    if not kinds:
        sys.exit(f"{run_dir}: no <kind><N>.model files")
    kind = max(kinds, key=lambda k: len(kinds[k]))
    vs = kinds[kind]
    if version is not None:
        if version not in vs:
            sys.exit(f"{run_dir}: no {kind}{version}.model")
        return kind, version
    now = time.time()
    # a kind that keeps states (trees) resumes from the states of both versions. A live run keeps only its last two
    # states (and a few older ones), so the newest pair is taken once its files are 30 s old (a version's files are
    # written once, at its end); the copy checks they didn't change.
    states = any(re.fullmatch(re.escape(kind) + r"\d+\.state", f) for f in os.listdir(run_dir))
    age = min(min_age, 30) if states else min_age
    for v in reversed(vs):
        if states and not all(os.path.exists(os.path.join(run_dir, f"{kind}{n}.state")) for n in (v - 1, v)):
            continue
        files = [os.path.join(run_dir, f"{kind}{n}.{e}") for n in (v - 1, v) for e in ("model", "state")]
        if v - 1 in vs and all(now - os.path.getmtime(f) > age for f in files if os.path.exists(f)):
            return kind, v
    sys.exit(f"{run_dir}: no settled version" + (" with the states of it and the version before" if states else ""))


def recipe_by(recipes, key, value):
    for r in recipes:
        if r[key] == value:
            return r
    return None


def heldout_dir(name, kind, recipes):
    key = name or (recipe_by(recipes, "kind", kind) or {}).get("heldout")
    return lg.DEFAULTS["calibration"]["heldout"].get(key) if key else None


def prepare_players(spec, work, recipes, nice):
    """Each tournament player's model in work/models, made or copied once: [dict(name, kind, stage, model, spec)]."""
    t = spec["tournament"]
    mdir = os.path.join(work, "models")
    os.makedirs(mdir, exist_ok=True)
    out = []
    for p in t["players"]:
        name = p["name"]
        if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.+=-]*", name):
            sys.exit(f"player name {name!r}: letters, digits and _ . + = - only")
        if "recipe" in p:
            row = recipe_by(recipes, "recipe", p["recipe"])
            if not row:
                sys.exit(f"{p['recipe']}: not a recipe")
            kind, stage = row["kind"], "untrained"
            target = os.path.join(mdir, f"{name}.{kind}.model")
            if not os.path.exists(target):
                lg.make_version0(row["train_binary"], kind, row["train_args"], 1, target, os.path.join(work, ".nogames"),
                                 os.path.join(work, f".v0-{name}"), os.path.join(work, "make.log"), nice)
            held = heldout_dir(p.get("heldout") or row["heldout"], kind, recipes)
            source = f"version 0 of recipe {p['recipe']}"
        else:
            if "from" in p:
                kind, v = settled_version(p["from"], p.get("version"))
                src = os.path.join(p["from"], f"{kind}{v}.model")
                source = f"{src} (version {v})"
            else:
                kind, src, source = p["kind"], p["model"], p["model"]
                if os.path.exists(src + ".source"):  # a calibration work dir's copy: what it was copied from
                    with open(src + ".source") as f:
                        source = f"{f.read().strip()}, via {src}"
            stage = p.get("stage", "mature")
            target = os.path.join(mdir, f"{name}.{kind}.model")
            label = target + ".source"  # what was copied (a run dir's newest version moves on)
            if not os.path.exists(target):
                shutil.copyfile(src, target + ".tmp")
                os.replace(target + ".tmp", target)
                lg.atomic_write(label, source + "\n")
            elif os.path.exists(label):
                with open(label) as f:
                    source = f.read().strip()
            held = heldout_dir(p.get("heldout"), kind, recipes)
        sp = f"cal:{held}:{kind}:{target}" if held else f"{kind}:{target}"
        out.append(dict(name=name, kind=kind, stage=stage, model=target, spec=sp, source=source,
                        bytes=os.path.getsize(target)))
    return out


def repeat_suffix(k):
    return f"-r{k}" if k > 1 else ""


def tournament_dirs(spec, work):
    """[(dir, plies)]: one round robin of every player at each ply (spec "split": true, the default: the additive
    fit needs no cross-ply games, and the cheap ply-1 games aren't lost among the dear ones), or one of all plies.
    With "repeats": n (default 1) the same round robins (same seed, same games) are played n times, interleaved
    (ply 1, ply 2, ply 1 again, ...), so the fit averages the CPU noise (core types, the machine's load) and the
    report can show it; repeat k > 1 plays in <dir>-r<k>."""
    t = spec["tournament"]
    plies = t.get("plies", [1, 2])
    reps = range(1, int(t.get("repeats", 1)) + 1)
    if t.get("split", True):
        return [(os.path.join(work, f"tour-p{p}{repeat_suffix(k)}"), [p]) for k in reps for p in plies]
    return [(os.path.join(work, f"tour{repeat_suffix(k)}"), plies) for k in reps]


TOUR_DIR_RE = re.compile(r"tour(-p\d+)?(-r(?P<rep>\d+))?")


def run_tournament(spec, work, players, nice):
    return sum(run_one_tournament(spec, tour, plies, players, nice) for tour, plies in tournament_dirs(spec, work))


def run_one_tournament(spec, tour, plies, players, nice):
    t = spec["tournament"]
    os.makedirs(tour, exist_ok=True)
    rows = [f"{r} {d} rand" for r, d in lg.REFERENCE_PLY.items()]
    for p in players:
        for ply in plies:
            rows.append(f"{p['name']}.p{ply} {ply} {p['spec']}")
    lg.atomic_write(os.path.join(tour, "players.txt"), "# NAME DEPTH SPEC: written by leagues/calibrate.py\n"
                    + "\n".join(rows) + "\n")
    if os.path.exists(os.path.join(tour, "games.csv")):
        print(f"{tour}: the calibration tournament was played already")
        return 0.0
    binary = t.get("binary") or lg.DEFAULTS["tournament"]["binary"]
    cmd = nice + [binary, "tournament", "players.txt", ".", "--tournaments", str(t.get("round_robins", 1)),
                  "--use-probs", "--anchor", "random", "--seed", str(t.get("seed", 1)), "--no-catch-up",
                  "--prior-draws", "0", *lg.DEFAULTS["tournament"]["flags"], "--game-cpu"]
    code, cpu, wall, out = lg.run_measured(cmd, os.path.join(tour, "tournament.log"), "calibration tournament",
                                           env=lg.threads_env(spec.get("threads", 2)), cwd=tour)
    if code != 0:
        sys.exit(f"calibration tournament failed (exit {code}); see {tour}/tournament.log")
    print(f"calibration tournament {os.path.basename(tour)}: {len(rows)} players, {cpu:.1f} CPU s, {wall:.1f} s wall")
    return cpu


def run_selfplay(item, work, recipes, nice):
    """One self-play measurement (kept in work/sp/<name>.json once done)."""
    rec_path = os.path.join(work, "sp", item["name"] + ".json")
    if os.path.exists(rec_path):
        return lg.read_json(rec_path)
    row = recipe_by(recipes, "model", item["model"])
    if not row:
        sys.exit(f"{item['model']}: not a model in recipes.txt")
    d = os.path.join(work, "sp", item["name"])
    shutil.rmtree(d, ignore_errors=True)
    os.makedirs(d)
    kind, args = row["kind"], row["selfplay_args"]
    if "depth" in item:  # the row's self-play at another search depth
        args = lg.apply_overrides(args, [("--depth", str(int(item["depth"])))])
    depth = lg.selfplay_depth(dict(selfplay_args=args))
    games, rounds = int(item.get("games", 10)), int(item.get("rounds", 1))
    resume, start, stage, source = [], 0, "untrained", "from scratch"
    if item.get("from"):
        k2, v = settled_version(item["from"], item.get("version"))
        if k2 != kind:
            sys.exit(f"{item['from']}: models of kind {k2}, not {kind}")
        # --resume finds the last of the versions saved from 1 on: versions v-1 and v are copied as 1 and 2
        for n, as_n in ((v - 1, 1), (v, 2)):
            for e in ("model", "state"):
                f = os.path.join(item["from"], f"{kind}{n}.{e}")
                if os.path.exists(f):
                    st = os.stat(f)
                    shutil.copyfile(f, os.path.join(d, f"{kind}{as_n}.{e}"))
                    st2 = os.stat(f)
                    if (st.st_size, st.st_mtime) != (st2.st_size, st2.st_mtime) or \
                            os.path.getsize(os.path.join(d, f"{kind}{as_n}.{e}")) != st.st_size:
                        sys.exit(f"{f} changed while it was copied; run the calibration again")
        with open(os.path.join(item["from"], "ladder.csv")) as f:
            header = f.readline()
        lg.atomic_write(os.path.join(d, "ladder.csv"), header)
        resume, start, stage, source = ["--resume"], 2, "mature", f"{item['from']} version {v}"
    elif kind == "moe":
        rounds = max(rounds, 2)
    cmd = nice + [row["selfplay_binary"], "self-play", d, *args, "--games", str(games), "--threads", "1",
                  "--rounds", str(start + rounds), *resume]
    code, cpu, wall, out = lg.run_measured(cmd, os.path.join(work, "sp", item["name"] + ".log"),
                                           f"self-play {item['name']}", env=lg.threads_env(1))
    if code != 0:
        sys.exit(f"self-play {item['name']} failed (exit {code}); see {d}.log")
    found = [lg.sp_round_of(m) for m in lg.SP_ROUND_RE.finditer(out)]
    plies = [n for _, _, n in found]
    play_s, train_s, labels_s = lg.selfplay_times(out)
    sizes = []  # the model file each round played with (None: not saved, as version 0 from scratch)
    for n in range(start, start + len(plies)):
        f = os.path.join(d, f"{kind}{n}.model")
        sizes.append(os.path.getsize(f) if os.path.exists(f) else None)
    rec = dict(name=item["name"], model=item["model"], kind=kind, depth=depth, binary=row["selfplay_binary"],
               stage=stage, source=source, games=games, rounds=len(plies), round_games=[g for _, g, _ in found],
               round_model_bytes=sizes, plies=sum(plies), cpu_s=cpu,
               play_s=play_s, train_s=train_s, labels_s=labels_s,
               gen_cpu_s=lg.generation_cpu(cpu, play_s, train_s, labels_s, 1), wall_s=wall)
    lg.write_json(rec_path, rec)
    shutil.rmtree(d, ignore_errors=True)
    print(f"self-play {item['name']}: {rec['plies']} plies, generation {rec['gen_cpu_s']:.2f} CPU s")
    return rec


# ---------------------------------------------------------------- the tournament fit

def tournament_games(work, players, include):
    """[dict(w, b: (kind, depth); stage_w, stage_b; bytes_w, bytes_b (model file sizes; None for references); mw, mb;
    cpu; plies; src; game: (round robin, index, white, black, plies) and rep: its repeat, for calibration games,
    else None)] of the calibration tournaments and the included leagues' games that have a measured CPU."""
    out = []
    srcs = []
    if players:
        sides = {f"{p['name']}.p{ply}": ((p["kind"], ply), p["stage"], p["bytes"]) for p in players
                 for ply in (1, 2, 3)}
        for d in sorted(x for x in os.listdir(work) if TOUR_DIR_RE.fullmatch(x)):
            rep = TOUR_DIR_RE.fullmatch(d).group("rep")
            srcs.append(("calibration tournament" + (f", repeat {rep}" if rep else ""), os.path.join(work, d),
                         sides, None, (re.sub(r"-r\d+$", "", d), int(rep or 1))))
    for d in include or []:
        cfg = lg.read_json(os.path.join(d, "config.json"))
        kinds = {lc["line"]: lc["kind"] for lc in cfg["lines"]}
        srcs.append((os.path.relpath(d, lg.REPO), os.path.join(d, "tour"), None, (d, kinds), None))
    size = {}

    def model_bytes(d, kind, line, v):
        key = (d, line, v)
        if key not in size:
            f = os.path.join(d, "runs", line, f"{kind}{v}.model")
            size[key] = os.path.getsize(f) if os.path.exists(f) else None
        return size[key]

    for label, tour, sides, league, rep in srcs:
        if not os.path.exists(os.path.join(tour, "game_cpu.csv")):
            continue
        for i, r in enumerate(lg.game_rows(tour, 0, 10 ** 9)):
            if r["cpu_s"] is None:
                continue
            sw, sb = [], []
            for name, acc in ((r["white"], sw), (r["black"], sb)):
                if name in lg.REFERENCE_PLY:
                    acc += [("random", lg.REFERENCE_PLY[name]), "reference", None]
                elif sides is not None:
                    acc += list(sides[name])
                else:
                    d, kinds = league
                    line, v, _, ply = lg.parse_entry(name)
                    acc += [(kinds[line], ply), "untrained" if v == 0 else "early",
                            model_bytes(d, kinds[line], line, v)]
            mw, mb = lg.moves_of(r["plies"])
            out.append(dict(w=sw[0], b=sb[0], stage_w=sw[1], stage_b=sb[1], bytes_w=sw[2], bytes_b=sb[2], mw=mw,
                            mb=mb, cpu=r["cpu_s"], plies=int(r["plies"]), src=label,
                            game=(rep[0], i, r["white"], r["black"], r["plies"]) if rep else None,
                            rep=rep[1] if rep else None))
    return out


def design(g):
    """A tournament game's terms: {(side, "game"): games, (side, "ply"): moves} of its two sides."""
    d = {}
    for side, mv in ((g["w"], g["mw"]), (g["b"], g["mb"])):
        d[(side, "game")] = d.get((side, "game"), 0) + 1
        d[(side, "ply")] = d.get((side, "ply"), 0) + mv
    return d


def predicted(c, g):
    return sum(v * c[k] for k, v in design(g).items())


def solve(rows, keys, fixed, weights):
    """Weighted least squares for the terms `keys` (fixed ones held): min sum w x (cpu - pred)^2."""
    idx = {k: i for i, k in enumerate(keys)}
    n = len(keys)
    m = [[0.0] * (n + 1) for _ in range(n)]
    for g, w in zip(rows, weights):
        d = design(g)
        y = g["cpu"] - sum(v * fixed[k] for k, v in d.items() if k in fixed)
        a = [(idx[k], v) for k, v in d.items() if k in idx and v]
        for i, vi in a:
            for j, vj in a:
                m[i][j] += w * vi * vj
            m[i][n] += w * vi * y
    for c in range(n):
        p = max(range(c, n), key=lambda r: abs(m[r][c]))
        m[c], m[p] = m[p], m[c]
        if abs(m[c][c]) < 1e-30:
            continue
        for r in range(n):
            if r != c and m[r][c]:
                f = m[r][c] / m[c][c]
                for k in range(c, n + 1):
                    m[r][k] -= f * m[c][k]
    return {k: (m[i][n] / m[i][i] if abs(m[i][i]) > 1e-30 else 0.0) for k, i in idx.items()}


def fit_tournament(rows):
    """{(kind, depth): (CPU s per side per game, CPU s per move)}: least squares weighted 1/predicted, iterated from
    1/measured; each iteration starts with every term free and holds the ones that come out negative at 0. At the
    end the weights are 1/predicted, so the prices' sum is the measured sum."""
    keys = sorted({k for g in rows for k in design(g)})
    weights = [1.0 / max(1e-6, g["cpu"]) for g in rows]
    c = {}
    for _ in range(60):
        fixed = {}
        while True:
            sol = solve(rows, [k for k in keys if k not in fixed], fixed, weights)
            neg = [k for k, v in sol.items() if v < 0]
            if not neg:
                break
            fixed.update(dict.fromkeys(neg, 0.0))
        c = dict(sol)
        c.update(fixed)
        new_w = [1.0 / max(1e-6, predicted(c, g)) for g in rows]
        done = max(abs(a - b) / b for a, b in zip(new_w, weights)) < 1e-6
        weights = new_w
        if done:
            break
    return {side: (c[(side, "game")], c[(side, "ply")]) for side in sorted({k[0] for k in keys})}


# ---------------------------------------------------------------- derived rows, the version, the report

def recipe_keys(recipes):
    """(needed, extra): the price keys recipes.txt's rows need (references, every kind at plies 1 and 2, each row's
    self-play depth), and the other self-play depths of SP_DEPTHS, written when they can be derived."""
    need = {("tournament", "random", d) for d in lg.REFERENCE_PLY.values()}
    for r in recipes:
        need |= {("tournament", r["kind"], 1), ("tournament", r["kind"], 2)}
        need.add(("self-play", r["kind"], lg.selfplay_depth(dict(selfplay_args=r["selfplay_args"]))))
    extra = {("self-play", r["kind"], d) for r in recipes for d in SP_DEPTHS} - need
    return need, extra


def tournament_ref_depth(d):
    """The tournament ply a self-play depth is priced from: 1 for depths 0 and 1, 2 deeper."""
    return 1 if d <= 1 else 2


def sp_games(r):
    """A self-play measurement's games (records before round_games: games per round x rounds)."""
    return sum(r["round_games"]) if "round_games" in r else r["games"] * r["rounds"]


def interpolate_kappa(kappa, depth):
    """(ratio, how) at a self-play depth from {depth: ratio}: measured, else geometric between the measured depths
    around it with the same tournament ply, else the nearest such depth below 3; None if none applies."""
    group = sorted(d for d in kappa if tournament_ref_depth(d) == tournament_ref_depth(depth))
    if depth in kappa:
        return kappa[depth], f"measured at depth {depth}"
    lo = [d for d in group if d < depth]
    hi = [d for d in group if d > depth]
    if lo and hi:
        a, b = lo[-1], hi[0]
        k = kappa[a] * (kappa[b] / kappa[a]) ** ((depth - a) / (b - a))
        return k, f"geometric between depths {a} and {b}"
    if depth <= 2 and group and max(group) <= 2:
        d = min(group, key=lambda x: (abs(x - depth), x))
        return kappa[d], f"nearest measured depth {d}"
    return None


def build_rows(version, note, tour_c, tour_counts, sp_recs, recipes, fit_stages=None):
    """(prices.csv rows, {(kind, depth): self-play ratio}, {depth: pooled ratio}, [keys not derived], {names of the
    self-play runs fitted}): measured rows, then derived ones for the keys recipes.txt needs and the extra self-play
    depths that can be derived. fit_stages: the stages a (kind, depth)'s self-play ratio is fitted on when it has
    runs of them (the spec's "selfplay_fit_stages", for instance ["mature"]: the stage experiments spend most cycles
    in); its other runs are only reported. None: every run."""
    need, extra = recipe_keys(recipes)
    rows = {}
    measured = {}
    for side, (a, c) in tour_c.items():
        games, moves = tour_counts.get(side, (0, 0))
        if c > 0 and games >= MIN_GAMES:
            measured[side] = (a, c)
            rows[("tournament",) + side] = dict(a=a, c=c, source="measured: tournament --game-cpu", games=games,
                                                plies=moves, note="")
    mean_t = {}
    for (kind, depth), (a, c) in measured.items():
        if kind != "random":
            n = tour_counts[(kind, depth)][1]
            x = mean_t.setdefault(depth, [0.0, 0.0, 0])
            x[0] += a * n
            x[1] += c * n
            x[2] += n

    def tour_const(kind, depth):
        if (kind, depth) in measured:
            return measured[(kind, depth)]
        if depth not in mean_t:
            sys.exit(f"can't price tournament {kind} {depth}: no measured model kind at ply {depth}")
        a, c, n = mean_t[depth]
        return a / n, c / n

    for key in sorted(k for k in need if k[0] == "tournament" and k not in rows):
        a, c = tour_const(key[1], key[2])
        why = "too few games" if tour_counts.get(key[1:], (0, 0))[0] else "kind not measured"
        if key[1:] in tour_c and tour_counts.get(key[1:], (0, 0))[0] >= MIN_GAMES:
            why = "fit gave no positive per-ply term"
        rows[key] = dict(a=a, c=c, source=f"derived: mean of measured kinds at ply {key[2]}", games=0, plies=0,
                         note=why)
    # self-play: measured generation CPU / the runs' price at the kind's tournament constants
    groups = {}
    for r in sp_recs:
        groups.setdefault((r["kind"], r["depth"]), []).append(r)
    sp, fit_runs, fit_by = {}, set(), {}
    for key, runs in groups.items():
        chosen = [r for r in runs if fit_stages and r["stage"] in fit_stages] or runs
        fit_by[key] = sorted({r["stage"] for r in chosen})
        for r in chosen:
            fit_runs.add(r["name"])
            a, c = tour_const(r["kind"], tournament_ref_depth(r["depth"]))
            x = sp.setdefault(key, [0.0, 0.0, 0, 0])
            x[0] += r["gen_cpu_s"]
            x[1] += 2 * sp_games(r) * a + r["plies"] * c
            x[2] += sp_games(r)
            x[3] += r["plies"]
    kappa_kd = {k: x[0] / x[1] for k, x in sp.items() if x[1] > 0}
    pooled = {}
    for (kind, depth), x in sp.items():
        y = pooled.setdefault(depth, [0.0, 0.0])
        y[0] += x[0]
        y[1] += x[1]
    kappa_d = {d: a / b for d, (a, b) in pooled.items() if b > 0}
    missing = []
    for key in sorted({("self-play",) + k for k in sp} | {k for k in need | extra if k[0] == "self-play"}):
        _, kind, depth = key
        ref = tournament_ref_depth(depth)
        if (kind, depth) in kappa_kd:
            k = kappa_kd[(kind, depth)]
            x = sp[(kind, depth)]
            src_ = (f"measured: self-play runs ({', '.join(fit_by[(kind, depth)])}), ratio {k:.3f} to tournament "
                    f"{kind} ply {ref}")
            games, plies, why = x[2], x[3], ""
        else:
            got = interpolate_kappa(kappa_d, depth)
            if got is None:
                if key in need:
                    sys.exit(f"can't price {' '.join(map(str, key))}: no self-play measurement at depth {depth} "
                             f"(or around it, at tournament ply {ref})")
                missing.append(key)
                continue
            k, how = got
            own = sorted(d for (kk, d) in kappa_kd if kk == kind and tournament_ref_depth(d) == ref)
            if own:  # the kind's own ratio at its nearest measured depth, moved along the pooled depth profile
                d = min(own, key=lambda x: (abs(x - depth), x))
                k = kappa_kd[(kind, d)] * k / kappa_d[d]
                how = f"{kind}'s ratio at depth {d} x the pooled ratios' change from depth {d} ({how})"
            else:
                how += ", pooled over kinds"
            src_ = f"derived: tournament {kind} ply {ref} x self-play ratio {k:.3f} ({how})"
            games, plies, why = 0, 0, "kind or depth not measured in self-play"
        a, c = tour_const(kind, ref)
        rows[key] = dict(a=a * k, c=c * k, source=src_, games=games, plies=plies, note=why)
    out = [dict(version=version, context=k[0], kind=k[1], depth=k[2], us_per_game=f"{v['a'] * 1e6:.4f}",
                us_per_ply=f"{v['c'] * 1e6:.4f}", source=v["source"], games=v["games"], plies=v["plies"],
                note=v["note"] or note) for k, v in sorted(rows.items())]
    return out, kappa_kd, kappa_d, missing, fit_runs


def ratio(p, m):
    return f"{p / m:.2f}" if m > 0 else ""


SIZES = ((50_000, "under 50 KB"), (150_000, "50-150 KB"), (400_000, "150-400 KB"), (1_000_000, "400 KB-1 MB"),
         (10 ** 12, "1 MB or more"))


def size_bucket(nb):
    return next(label for top, label in SIZES if nb < top)


def length_of(plies):
    return next(label for top, label in LENGTHS if plies < top)


def repeats_report(games, us):
    """The report's section on repeated round robins (none without repeats): how much the same games' CPU moves
    between repeats, and how much each repeat alone would move the constants."""
    reps = {}
    for g in games:
        if g.get("game"):
            reps.setdefault(g["game"], {})[g["rep"]] = g["cpu"]
    nrep = max((r for v in reps.values() for r in v), default=1)
    if nrep < 2:
        return []
    lines = ["", "## Repeats: the same games measured again", "",
             "Each repeat replays the round robins with the same seed. Games are matched by round robin, order, "
             "players and plies. Per game: CPU of repeat k / repeat 1 (median, 10th and 90th percentiles); total: "
             "the matched games' summed CPU, repeat k / repeat 1.", "",
             "| round robin | repeat | games matched / played | per game: median | 10th pct | 90th pct | total |",
             "|---|---:|---:|---:|---:|---:|---:|"]
    for base in sorted({k[0] for k in reps}):
        n1 = sum(1 for k, v in reps.items() if k[0] == base and 1 in v)
        for rk in range(2, nrep + 1):
            pairs = [(v[1], v[rk]) for k, v in reps.items() if k[0] == base and 1 in v and rk in v and v[1] > 0]
            if not pairs:
                lines.append(f"| {base} | {rk} | 0 / {n1} | | | | |")
                continue
            rs = sorted(b / a for a, b in pairs)
            q = [rs[min(len(rs) - 1, int(f * len(rs)))] for f in (0.5, 0.1, 0.9)]
            tot = sum(b for _, b in pairs) / sum(a for a, _ in pairs)
            lines.append(f"| {base} | {rk} | {len(pairs)} / {n1} | {q[0]:.2f} | {q[1]:.2f} | {q[2]:.2f} | "
                         f"{tot:.2f} |")
    # the constants fit on each repeat alone, as the price of a 60-move side relative to the version's
    fits = {rk: fit_tournament([g for g in games if g.get("rep") == rk]) for rk in range(1, nrep + 1)}
    lines += ["", "Fit on one repeat's games alone: the price of a side of 60 moves, relative to the version's "
              "constants.", "", "| kind | depth | " + " | ".join(f"repeat {rk}" for rk in fits) + " |",
              "|---|---:|" + "---:|" * len(fits)]
    for side in sorted({k for f in fits.values() for k in f}):
        a, c = us[lg.price_key("tournament", *side)]
        base = a + 60 * c
        cells = [f"{(f[side][0] + 60 * f[side][1]) * 1e6 / base:.2f}" if side in f and base > 0 else ""
                 for f in fits.values()]
        lines.append(f"| {side[0]} | {side[1]} | " + " | ".join(cells) + " |")
    return lines


def costs_rows(version, rows, sp_recs):
    """Rows `lr d<d> jitter10` (d = 0-4) for leagues/costs.csv, which the self-play round-size rule (leagues.py
    round_size) reads: the version's self-play price of one lr game at depth d, at the length of the mature lr runs'
    games (plies per game; a depth without a mature run takes the nearest measured depth's, the lower on a tie)."""
    us = lg.prices_from_rows(rows)
    lengths = {}
    for r in sp_recs:
        if r["kind"] == "lr" and r["stage"] == "mature" and sp_games(r):
            x = lengths.setdefault(r["depth"], [0, 0])
            x[0] += r["plies"]
            x[1] += sp_games(r)
    out = []
    binary = os.path.relpath((recipe_by(lg.load_recipes(), "model", "F-L") or {}).get("selfplay_binary", ""), lg.REPO)
    for d in SP_DEPTHS:
        key = lg.price_key("self-play", "lr", d)
        if key not in us or not lengths:
            continue
        near = min(lengths, key=lambda x: (abs(x - d), x))
        plies = lengths[near][0] / lengths[near][1]
        a, c = us[key]
        out.append(dict(signature=f"lr d{d} jitter10", cpu_per_game=f"{(2 * a + plies * c) / 1e6:.6f}", games="",
                        rounds="", threads=1, cpu_s="", wall_s="", binary=binary,
                        models=f"prices {version}: lr self-play at depth {d}, {plies:.0f} plies a game (the mature "
                               f"runs' at depth {near}); priced, not measured",
                        date=datetime.datetime.now().strftime("%Y-%m-%d %H:%M"), loadavg=""))
    return out


def measured_cpu(work, sp_recs):
    """(tournament CPU s, self-play CPU s) of the measurements in the work dir, however many runs made them: the
    calibration tournaments' logged CPU and the self-play records' (version 0 models and fits not counted)."""
    tour = 0.0
    for d in sorted(x for x in os.listdir(work) if TOUR_DIR_RE.fullmatch(x)):
        log = os.path.join(work, d, "tournament.log")
        if os.path.exists(log):
            with open(log) as f:
                tour += sum(float(m.group(1)) for m in
                            re.finditer(r"^== calibration tournament: exit 0, cpu ([0-9.]+) s", f.read(), re.M))
    return tour, sum(r["cpu_s"] for r in sp_recs)


def report(version, spec, rows, games, sp_recs, kappas, cal_cpu, players, missing):
    us = lg.prices_from_rows(rows)

    def priced(g):
        return lg.game_price(us, g["w"], g["b"], g["plies"])

    lines = [f"# Prices {version}", "",
             f"Calibrated {datetime.datetime.now():%Y-%m-%d %H:%M} by leagues/calibrate.py from "
             f"{os.path.abspath(spec['_path'])}. {spec.get('note', '')}".rstrip(), "",
             "A game's price: each side's `us_per_game` plus its moves times its `us_per_ply` (CPU microseconds; the "
             "side's context, kind and search depth), summed over the two sides. A self-play round of G games and "
             "P plies: 2 G `us_per_game` plus P `us_per_ply` (both sides are the line's model). "
             f"Calibration CPU: {sum(cal_cpu):.0f} s (tournaments {cal_cpu[0]:.0f} s, self-play {cal_cpu[1]:.0f} s).",
             "", "## Constants", "",
             "| context | kind | depth | us per game | us per ply | source | games | plies |",
             "|---|---|---:|---:|---:|---|---:|---:|"]
    for r in rows:
        lines.append(f"| {r['context']} | {r['kind']} | {r['depth']} | {float(r['us_per_game']):.1f} | "
                     f"{float(r['us_per_ply']):.2f} | {r['source']} | {r['games']} | {r['plies']} |")
    if missing:
        lines += ["", "Not priced (no measured self-play depth around them): "
                  + ", ".join(" ".join(map(str, k)) for k in missing) + "."]
    if players:
        lines += ["", "## Calibration players", "", "| player | kind | stage | model bytes | source |",
                  "|---|---|---|---:|---|"]
        lines += [f"| {p['name']} | {p['kind']} | {p['stage']} | {p['bytes']} | {p['source']} |" for p in players]
    # errors: share-weighted by side class and stage
    by = {}
    for g in games:
        pg = priced(g)
        if pg <= 0:
            continue
        for side, mv, stage in ((g["w"], g["mw"], g["stage_w"]), (g["b"], g["mb"], g["stage_b"])):
            a, c = us[lg.price_key("tournament", *side)]
            share = (a + mv * c) / 1e6 / pg
            for key in ((side, stage), (side, "all")):
                x = by.setdefault(key, [0.0, 0.0, 0])
                x[0] += share * pg
                x[1] += share * g["cpu"]
                x[2] += 1
    sizes = {}
    for g in games:
        pg = priced(g)
        if pg <= 0:
            continue
        for side, mv, nb in ((g["w"], g["mw"], g["bytes_w"]), (g["b"], g["mb"], g["bytes_b"])):
            if nb is None:
                continue
            a, c = us[lg.price_key("tournament", *side)]
            share = (a + mv * c) / 1e6 / pg
            x = sizes.setdefault((side, size_bucket(nb)), [0.0, 0.0, 0])
            x[0] += share * pg
            x[1] += share * g["cpu"]
            x[2] += 1
    lines += ["", "## Tournament: priced / measured CPU by kind, depth and model size", "",
              "As below, by the model file's size: the price has no size term, so a ratio that falls with size "
              "means bigger models cost more per ply than the kind's constants say. Cells: ratio (games).", "",
              "| kind | depth | " + " | ".join(label for _, label in SIZES) + " |",
              "|---|---:|" + "---:|" * len(SIZES)]
    for side in sorted({k[0] for k in sizes}):
        cells = []
        for _, label in SIZES:
            x = sizes.get((side, label))
            cells.append(f"{ratio(x[0], x[1])} ({x[2]})" if x else "")
        lines.append(f"| {side[0]} | {side[1]} | " + " | ".join(cells) + " |")
    lines += ["", "## Tournament: priced / measured CPU by kind, depth and stage", "",
              "Each game's measured CPU is split between its sides in proportion to their prices. Stages: untrained "
              "(version 0), early (a league's trained versions), mature (copied from run dirs), reference. Cells: "
              "ratio (games).", "",
              "| kind | depth | all | untrained | early | mature | reference |", "|---|---:|---:|---:|---:|---:|---:|"]
    for side in sorted({k[0] for k in by}):
        cells = []
        for stage in ("all",) + STAGES + ("reference",):
            x = by.get((side, stage))
            cells.append(f"{ratio(x[0], x[1])} ({x[2]})" if x else "")
        lines.append(f"| {side[0]} | {side[1]} | " + " | ".join(cells) + " |")
    # by game stage x length, and source
    grp = {}
    for g in games:
        st = [s for s in (g["stage_w"], g["stage_b"]) if s != "reference"]
        gs = min(st, key=STAGES.index) if st else "reference"
        for key in (("stage x length", f"{gs}, {length_of(g['plies'])}"), ("stage", gs),
                    ("length", length_of(g["plies"])), ("source", g["src"]), ("all", "")):
            x = grp.setdefault(key, [0.0, 0.0, 0])
            x[0] += priced(g)
            x[1] += g["cpu"]
            x[2] += 1
    lines += ["", "## Tournament: priced / measured CPU of game groups", "",
              "A game's stage is its least trained model side's.", "",
              "| group | value | games | measured CPU s | priced s | priced / measured |",
              "|---|---|---:|---:|---:|---:|"]
    order = {"all": 0, "stage": 1, "length": 2, "stage x length": 3, "source": 4}
    for k in sorted(grp, key=lambda k: (order[k[0]], k[1])):
        p, m, n = grp[k]
        lines.append(f"| {k[0]} | {k[1]} | {n} | {m:.2f} | {p:.2f} | {ratio(p, m)} |")
    lines += repeats_report(games, us)
    if sp_recs:
        kappa_kd, kappa_d, fit_runs = kappas
        lines += ["", "## Self-play runs: priced / measured generation CPU", "",
                  "A run marked `fit` sets its kind and depth's self-play ratio"
                  + (f" (the spec fits on {', '.join(spec['selfplay_fit_stages'])} runs where a kind and depth has "
                     "them)" if spec.get("selfplay_fit_stages") else "") + "; the others are only checked.", "",
                  "| run | model | kind | depth | stage | source | games | plies | generation CPU s | priced s | ratio "
                  "| fit |", "|---|---|---|---:|---|---|---:|---:|---:|---:|---:|---|"]
        stage_sum = {}
        for r in sp_recs:
            p = lg.selfplay_round_price(us, r["kind"], r["depth"], sp_games(r), r["plies"])
            src = r.get("source", "")
            src = src.replace(lg.REPO + "/", "") if isinstance(src, str) else src
            lines.append(f"| {r['name']} | {r['model']} | {r['kind']} | {r['depth']} | {r['stage']} | {src} | "
                         f"{sp_games(r)} | {r['plies']} | {r['gen_cpu_s']:.2f} | {p:.2f} | "
                         f"{ratio(p, r['gen_cpu_s'])} | {'fit' if r['name'] in fit_runs else ''} |")
            for key in ((r["kind"], r["depth"], r["stage"]), (r["kind"], r["depth"], "all"), ("all", "", r["stage"]),
                        ("all", "", "all")):
                x = stage_sum.setdefault(key, [0.0, 0.0, 0])
                x[0] += p
                x[1] += r["gen_cpu_s"]
                x[2] += 1
        lines += ["", "## Self-play: priced / measured generation CPU by kind, depth and stage", "",
                  "Cells: ratio (runs).", "", "| kind | depth | untrained | mature | all |", "|---|---:|---:|---:|---:|"]
        for kind, depth in sorted({(k[0], k[1]) for k in stage_sum if k[0] != "all"}) + [("all", "")]:
            cells = []
            for stage in ("untrained", "mature", "all"):
                x = stage_sum.get((kind, depth, stage))
                cells.append(f"{ratio(x[0], x[1])} ({x[2]})" if x else "")
            lines.append(f"| {kind} | {depth} | " + " | ".join(cells) + " |")
        lines += ["", "Self-play/tournament ratios (generation CPU / price at the kind's tournament constants): "
                  + ", ".join(f"{k} depth {d}: {v:.3f}" for (k, d), v in sorted(kappa_kd.items()))
                  + "; pooled by depth: " + ", ".join(f"{d}: {v:.3f}" for d, v in sorted(kappa_d.items())) + "."]
    return "\n".join(lines) + "\n"


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("spec")
    ap.add_argument("--fit-only", action="store_true", help="refit from the measurements already in the work dir")
    ap.add_argument("--measure-only", action="store_true", help="make the measurements; fit and write nothing")
    ap.add_argument("--check", action="store_true",
                    help="fit and write the report to the work dir only (<version>-check.md); no version is added")
    args = ap.parse_args()
    with open(args.spec) as f:
        spec = json.load(f)
    spec["_path"] = args.spec
    version = spec["version"]
    if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.-]*", version):
        sys.exit(f"bad version {version!r}")
    have, _ = lg.read_prices()
    if version in have and not (args.check or args.measure_only):
        sys.exit(f"{lg.PRICES} already has version {version}; versions never change, give a new one")
    work = os.path.abspath(spec["work"])
    os.makedirs(work, exist_ok=True)
    nice = ["nice", "-n", str(spec.get("nice", 19))]
    recipes = lg.load_recipes()
    cal_cpu = 0.0
    players = []
    if spec.get("tournament", {}).get("players"):
        players = prepare_players(spec, work, recipes, nice)
        if not args.fit_only:
            cal_cpu += run_tournament(spec, work, players, nice)
    sp_recs = []
    for item in spec.get("selfplay", []):
        rec_path = os.path.join(work, "sp", item["name"] + ".json")
        if args.fit_only and not os.path.exists(rec_path):
            continue
        os.makedirs(os.path.join(work, "sp"), exist_ok=True)
        had = os.path.exists(rec_path)
        rec = run_selfplay(item, work, recipes, nice)
        if not had:
            cal_cpu += rec["cpu_s"]
        sp_recs.append(rec)
    if args.measure_only:
        print(f"measurements in {work}; calibration CPU {cal_cpu:.1f} s")
        return
    games = tournament_games(work, players, spec.get("include"))
    if not games:
        sys.exit("no tournament games with measured CPU to fit")
    tour_c = fit_tournament(games)
    counts = {}
    for g in games:
        for side, mv in ((g["w"], g["mw"]), (g["b"], g["mb"])):
            x = counts.setdefault(side, [0, 0])
            x[0] += 1
            x[1] += mv
    rows, kappa_kd, kappa_d, missing, fit_runs = build_rows(version, spec.get("note", ""), tour_c,
                                                            {k: tuple(v) for k, v in counts.items()}, sp_recs,
                                                            recipes, spec.get("selfplay_fit_stages"))
    rep = report(version, spec, rows, games, sp_recs, (kappa_kd, kappa_d, fit_runs), measured_cpu(work, sp_recs),
                 players, missing)
    if args.check:
        path = os.path.join(work, f"{version}-check.md")
        lg.atomic_write(path, rep)
        print(f"check: report in {path}; no version added")
        return
    os.makedirs(REPORTS, exist_ok=True)
    lg.atomic_write(os.path.join(REPORTS, f"{version}.md"), rep)
    exists = os.path.exists(lg.PRICES)
    text = "" if exists else lg.csv_text(lg.PRICES_FIELDS, [])
    with open(lg.PRICES, "a") as f:
        f.write(text + lg.csv_text(lg.PRICES_FIELDS, rows).split("\n", 1)[1])
    print(f"prices {version}: {len(rows)} rows added to {lg.PRICES}; report {os.path.join(REPORTS, version + '.md')}")
    if spec.get("refresh_costs"):  # the round-size rule follows the version's self-play prices
        crows = costs_rows(version, rows, sp_recs)
        fields = list(lg.read_csv(lg.COSTS)[0].keys()) if os.path.exists(lg.COSTS) else list(crows[0].keys())
        with open(lg.COSTS, "a") as f:
            f.write(lg.csv_text(fields, crows).split("\n", 1)[1])
        costs = lg.load_costs()
        sizes = ", ".join(f"depth {d}: {lg.round_size(1000, costs, d)}" for d in SP_DEPTHS)
        print(f"{len(crows)} rows added to {lg.COSTS}; self-play games per round (anchor 1000): {sizes}")


if __name__ == "__main__":
    main()
