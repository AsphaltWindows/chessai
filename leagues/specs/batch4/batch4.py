#!/usr/bin/env python3
"""Batch 4 (the user, 2026-11-10 17:15): every experiment so far rerun under the new experiment defaults, no random
players, no ply-1 entries, run until the champions plateau. Each original ORIG gets ORIG-new; the arenas that put both
versions' champions together (with random players) are built separately (leagues/arenas/).

    python3 leagues/specs/batch4/batch4.py specs [NAME]  write specs/batch4/ORIG-new.json from the originals and the
                                                         extras' specs (NAME: only that one, an original or an extra)
    python3 leagues/specs/batch4/batch4.py check [ORIG]  check the specs against the originals (nothing written)
    python3 leagues/specs/batch4/batch4.py create ORIG   create experiments/ORIG-new (checked first and after)
    python3 leagues/specs/batch4/batch4.py state [NAME]  rewrite experiments/batch4-queue.txt (NAME: the one running)
    python3 leagues/specs/batch4/batch4.py items | status-of NAME | create-item NAME   for queue.sh: the queue's
                                                         experiments in order, one's status, create one
    python3 leagues/specs/batch4/batch4.py order | status ORIG   the reruns' originals; ORIG-new's status (the
                                                         first queue.sh, started 17:41, calls these)

What a rerun keeps from its original: the lines (models, recipes, options, every argument), the seed and train
seed, the calibration, champion_se and the prices version. What changes:
- references [], ply1_entries false, plateau {"cycles": 50, "min_cycles": 100}, cycles 1000 (the cap);
- rounds_per_cycle (and min_rounds): the original's mean games with a model at ply 2 per cycle (totals.csv) over the
  games of one round of the new field, two ply-2 entries per line (2n(2n-1) for n lines), rounded;
- binaries, today's exact ones: tournament chessai-hybrid9; train chessai-hybrid6, except lines on chessai-target or
  chessai-trigger (kept) and search-label lines (--label-depth: kept, lr-settings' F-L+label-depth=2...); self-play
  binaries kept (no newer self-play binary has been checked byte-identical).
`leagues.py create` checks each spec flag against its recipe row's binary; as in models/from-scratch/target/league/
create_et.py, create gives the rows of models whose original lines ran chessai-target or chessai-trigger that binary
in memory, runs the engine's own create, and then sets every line's binaries as above in config.json (before cycle
1; atomic write). The spec's "batch4" key (the engine ignores it) records the original and the binaries.

The queue (leagues/specs/batch4/queue.sh) runs the reruns one at a time in ORDER, then the EXTRA experiments;
selfplay-depth is held (its spec is written, it isn't started); nb-settings-no-random3 is covered by nb-settings-new
(the same design once its references are gone: same lines, seeds, rounds and cycles).

Extras (not reruns: no original, no pair arena), specs/batch4/NAME.json:
- small-field-x4 (the user, 11-10 ~18:00: "One more thing for us to test in the queue is the same idea but with 4 of
  each all differently seeded"): small-field's two models (F-L; T-L-Tc-merge at single-split's split limit 1 and
  threshold 1M), four replicates each through the engine's "replicates" (replicate r trains with seed + cycle +
  100003 x (r - 1)), under batch 4's conditions, with a new experiment seed (11) and rounds for small-field's games
  with a model at ply 2 per cycle (2n(2n-1) a round, n = 8). Train binary hybrid6 on every line, as in small-field."""
import copy
import csv
import importlib.machinery
import importlib.util
import json
import os
import sys

ROOT = "/home/iv/dev/chessai/leagues"
BIN = "/home/iv/dev/chessai/models/from-scratch/bin"
HERE = os.path.join(ROOT, "specs", "batch4")
QUEUE_TXT = os.path.join(ROOT, "experiments", "batch4-queue.txt")
TOURNAMENT = os.path.join(BIN, "chessai-hybrid9")
# The tournament binaries a rerun may name: hybrid9 from 2026-11-10 ~19:50, hybrid8 before (the same games and
# own-calibration games, byte for byte: models/from-scratch/fastbin/hybrid9/summary.txt).
TOURNAMENTS = {os.path.join(BIN, "chessai-hybrid8"), TOURNAMENT}
TRAIN = os.path.join(BIN, "chessai-hybrid6")
KEEP_TRAIN = {"chessai-target", "chessai-trigger"}
PLATEAU = {"cycles": 50, "min_cycles": 100}
CAP = 1000
ORIGINAL_SPECS = {  # the originals, chronologically (RESULTS.md section 7)
    "lr-settings": "batch1/1a-lr-settings.json", "probit-settings": "batch1/1b-probit-settings.json",
    "nb-settings": "batch1/1c-nb-settings.json", "nb-settings-no-random3": "batch1/1c-nb-settings-no-random3.json",
    "selfplay-depth": "batch1/3-selfplay-depth.json", "leaf-sources": "batch1/4-leaf-sources.json",
    "lr-settings-2": "batch1/1a2-lr-settings-2.json", "nb-settings-2": "batch1/1c2-nb-settings-2.json",
    "leaf-sources-2": "batch1/4b-leaf-sources-2.json", "single-split": "batch2/1-single-split.json",
    "single-split-probit": "batch2/2-single-split-probit.json", "cluster-error": "batch2/3-cluster-error.json",
    "split-growth": "batch3/7-split-growth.json", "split-growth-probit": "batch3/8-split-growth-probit.json",
    "error-targets": "batch3/9-error-targets.json", "error-targets-probit": "batch3/10-error-targets-probit.json",
    "residual-leaves": "batch3/11-residual-leaves.json", "trigger-gradient": "batch3/14-trigger-gradient.json",
    "small-field": "batch3/15-small-field.json"}
ORDER = ["lr-settings", "probit-settings", "nb-settings", "leaf-sources", "lr-settings-2", "nb-settings-2",
         "leaf-sources-2", "single-split", "single-split-probit", "cluster-error", "split-growth",
         "split-growth-probit", "error-targets", "error-targets-probit", "residual-leaves", "trigger-gradient",
         "small-field"]
HELD = ["selfplay-depth"]  # spec written; not started (the coordinator, relaying the user)
COVERED = {"nb-settings-no-random3": "nb-settings"}  # the same design without references
ROUNDS_FROM = {"nb-settings-no-random3": "nb-settings"}  # it runs as nb-settings-new
EXTRA = {"small-field-x4": "small-field"}  # queued after the reruns: name -> the experiment whose models it reuses
EXTRA_SEED = {"small-field-x4": 11}  # a seed no other experiment or prepared spec uses
REPLICATES = 4


def load_engine():
    path = os.environ.get("BATCH4_LEAGUES", os.path.join(ROOT, "leagues.py"))
    loader = importlib.machinery.SourceFileLoader("leagues", path)
    spec = importlib.util.spec_from_loader("leagues", loader)
    mod = importlib.util.module_from_spec(spec)
    sys.modules["leagues"] = mod
    loader.exec_module(mod)
    if not hasattr(mod, "batch4_options"):
        sys.exit(f"{path}: an engine without the batch-4 options")
    return mod


LG = load_engine()


def orig_dir(orig):
    return os.path.join(ROOT, "experiments", orig)


def items():
    """The queue's experiments in order: the reruns (ORIG-new), then the extras."""
    return [new_name(o) for o in ORDER] + list(EXTRA)


def extra_spec_path(name):
    return os.path.join(HERE, name + ".json")


def new_name(orig):
    return orig + "-new"


def spec_path(orig):
    return os.path.join(HERE, new_name(orig) + ".json")


def original(orig):
    cfg = LG.read_json(os.path.join(orig_dir(orig), "config.json"))
    spec = LG.read_json(os.path.join(ROOT, "specs", ORIGINAL_SPECS[orig]))
    totals = LG.read_csv(os.path.join(orig_dir(orig), "totals.csv"))
    return cfg, spec, totals


def rounds_for(orig):
    """(rounds, mean ply-2 games per cycle, n lines): the original's mean games with a model at ply 2 per cycle over
    one round of the new field (2n entries, 2n(2n-1) games), rounded, at least 1."""
    src = ROUNDS_FROM.get(orig, orig)
    cfg, _, totals = original(src)
    n = len(cfg["lines"])
    mean = sum(int(r["ply2_games"]) for r in totals) / len(totals)
    return max(1, round(mean / (2 * n * (2 * n - 1)))), mean, n


def rerun_binaries(lc):
    """(train binary, self-play binary) of a rerun line, from the original line."""
    tb = lc["train_binary"]
    if os.path.basename(tb) not in KEEP_TRAIN and "--label-depth" not in lc["train_args"]:
        tb = TRAIN
    return tb, lc.get("selfplay_binary")


def make_spec(orig):
    cfg, spec, _ = original(orig)
    rounds, mean, n = rounds_for(orig)
    out = {k: v for k, v in spec.items() if k not in ("name", "references", "cycles", "rounds_per_cycle", "min_rounds",
                                                      "tournament")}
    out["name"] = new_name(orig)
    tour = dict(spec.get("tournament") or {})
    tour["binary"] = TOURNAMENT
    out.update(references=[], ply1_entries=False, plateau=dict(PLATEAU), cycles=CAP, rounds_per_cycle=rounds,
               min_rounds=rounds, tournament=tour, prices_version=cfg["prices"]["version"])
    if cfg.get("seed") != LG.DEFAULTS["seed"] or "seed" in spec:
        out["seed"] = cfg["seed"]
    if cfg["train"] != LG.DEFAULTS["train"] or "train" in spec:
        out["train"] = cfg["train"]
    binaries = {lc["line"]: dict(zip(("train", "selfplay"), (os.path.basename(b) if b else None
                                                            for b in rerun_binaries(lc)))) for lc in cfg["lines"]}
    note = (f"Batch 4: {orig} rerun without random players or ply-1 entries until the champions plateau. "
            f"Rounds: {mean:.0f} games with a model at ply 2 per cycle in the original over {2 * n * (2 * n - 1)} "
            f"per round ({n} lines, two ply-2 entries each).")
    if orig in COVERED:
        note += f" Covered by {new_name(COVERED[orig])} (the same design once the references are gone): not run."
    if orig in HELD:
        note += " Held: written, not started."
    out["batch4"] = dict(original=orig, note=note, binaries=binaries,
                         rounds_from=dict(original=ROUNDS_FROM.get(orig, orig), ply2_games_per_cycle=round(mean, 2),
                                          lines=n, games_per_round=2 * n * (2 * n - 1)))
    return out


def spec_binaries(spec):
    """{line: (train binary, self-play binary or None)}: the binaries the spec records (fixed when it was written)."""
    return {line: (os.path.join(BIN, b["train"]), os.path.join(BIN, b["selfplay"]) if b["selfplay"] else None)
            for line, b in spec["batch4"]["binaries"].items()}


def patched_recipes(orig, spec):
    """recipes.txt's rows, those of models whose lines the spec puts on chessai-target or chessai-trigger given that
    binary (in memory), so create checks their flags against it."""
    cfg, _, _ = original(orig)
    sb = spec_binaries(spec)
    special = {}
    for lc in cfg["lines"]:
        b = sb[lc["line"]][0]
        if os.path.basename(b) in KEEP_TRAIN:
            special.setdefault(lc["base_model"], set()).add(b)
    for m, bs in special.items():
        if len(bs) > 1:
            sys.exit(f"{orig}: model {m} runs on several special binaries: {sorted(bs)}")
    rows = ENGINE_LOAD_RECIPES()
    return [dict(r, train_binary=next(iter(special[r["model"]]))) if r["model"] in special else r for r in rows]


ENGINE_LOAD_RECIPES = LG.load_recipes


def build_config(orig, spec):
    """make_config of the rerun's spec (recipes patched as above), then every line's binaries set."""
    LG.load_recipes = lambda path=None: patched_recipes(orig, spec)
    try:
        ldir, cfg = LG.make_config(spec)
    finally:
        LG.load_recipes = ENGINE_LOAD_RECIPES
    set_binaries(cfg, spec)
    return ldir, cfg


def set_binaries(cfg, spec):
    sb = spec_binaries(spec)
    for lc in cfg["lines"]:
        tb, spb = sb[lc["line"]]
        lc["train_binary"] = tb
        if lc.get("selfplay_binary"):
            lc["selfplay_binary"] = spb


SAME_TOP = ("kind", "seed", "nice", "threads", "jobs", "champion_se", "calibration", "train", "selfplay",
            "global_source", "wait_poll_s", "prices", "selfplay_budget_s", "patience_cycles", "anchors")


def check(orig, cfg, spec):
    """Problems of a rerun's config against its original's: [] when it keeps everything it should (the lines but
    their binaries, which are the spec's, seeds, calibration, champion_se, prices, ...) and has the batch-4 keys."""
    ocfg, _, _ = original(orig)
    rf = spec["batch4"]["rounds_from"]
    rounds = max(1, round(rf["ply2_games_per_cycle"] / rf["games_per_round"]))
    p = []
    if rf["games_per_round"] != 2 * len(ocfg["lines"]) * (2 * len(ocfg["lines"]) - 1):
        p.append("the spec's games per round")
    sbin = spec_binaries(spec)
    if set(sbin) != {lc["line"] for lc in ocfg["lines"]}:
        p.append("the spec's binaries name other lines")
    if [lc["line"] for lc in cfg["lines"]] != [lc["line"] for lc in ocfg["lines"]]:
        p.append("line names or order")
    for a, b in zip(ocfg["lines"], cfg["lines"]):
        tb, sb = sbin.get(a["line"], (None, None))
        if (os.path.basename(tb or ""), os.path.basename(sb or "")) != tuple(
                os.path.basename(x or "") for x in rerun_binaries(a)):
            p.append(f"{a['line']}: the spec's binaries aren't the rule's for the original line")
        for k in sorted(set(a) | set(b)):
            want = tb if k == "train_binary" else sb if k == "selfplay_binary" else a.get(k)
            if b.get(k) != want:
                p.append(f"{a['line']} {k}: {a.get(k)!r} -> {b.get(k)!r}")
    for k in SAME_TOP:
        if ocfg.get(k) != cfg.get(k):
            p.append(f"{k}: {json.dumps(ocfg.get(k))[:80]} -> {json.dumps(cfg.get(k))[:80]}")
    for k in ("flags", "game_cpu"):
        if ocfg["tournament"].get(k) != cfg["tournament"].get(k):
            p.append(f"tournament {k}")
    want = dict(name=new_name(orig), references=[], ply1_entries=False, plateau=PLATEAU, cycles=CAP,
                rounds_per_cycle=rounds, min_rounds=rounds, rating_zero=LG.entry_name(ocfg["lines"][0]["line"], 0, 2))
    for k, v in want.items():
        if cfg.get(k) != v:
            p.append(f"{k}: {cfg.get(k)!r}, want {v!r}")
    if cfg["tournament"]["binary"] not in TOURNAMENTS:
        p.append("tournament binary")
    extra = set(cfg) - set(ocfg) - {"ply1_entries", "plateau", "rating_zero"}
    if extra:
        p.append(f"keys the original hasn't: {sorted(extra)}")
    missing = set(ocfg) - set(cfg)
    if missing:
        p.append(f"keys the original has: {sorted(missing)}")
    return p


def make_extra_spec(name):
    """An extra's spec: its model source's spec with every model item replicated, a new seed and batch 4's
    conditions; rounds as for a rerun, from the source's games with a model at ply 2 per cycle."""
    like = EXTRA[name]
    cfg, spec, totals = original(like)
    models = [dict({"model": m} if isinstance(m, str) else m, replicates=REPLICATES) for m in spec["models"]]
    n = len(cfg["lines"]) * REPLICATES
    mean = sum(int(r["ply2_games"]) for r in totals) / len(totals)
    rounds = max(1, round(mean / (2 * n * (2 * n - 1))))
    out = {k: v for k, v in spec.items() if k not in ("name", "models", "seed", "train", "references", "cycles",
                                                      "rounds_per_cycle", "min_rounds", "tournament")}
    tour = dict(spec.get("tournament") or {})
    tour["binary"] = TOURNAMENT
    seed = EXTRA_SEED[name]
    out.update(name=name, models=models, seed=seed, train=dict(seed=seed), references=[], ply1_entries=False,
               plateau=dict(PLATEAU), cycles=CAP, rounds_per_cycle=rounds, min_rounds=rounds, tournament=tour,
               prices_version=cfg["prices"]["version"])
    _, c = LG.make_config(out)  # the line names (these flags are in the recipes' own binary too)
    binaries = {lc["line"]: dict(train=os.path.basename(TRAIN), selfplay=None) for lc in c["lines"]}
    note = (f"Batch 4 extra (the user, 11-10 ~18:00): {like}'s models, {REPLICATES} replicates each (replicate r "
            f"trains with seed + cycle + {LG.REPLICATE_SEED_STEP} x (r - 1)), without random players or ply-1 entries, "
            f"until the champions plateau; seed {seed}. Rounds: {mean:.0f} games with a model at ply 2 per cycle in "
            f"{like} over {2 * n * (2 * n - 1)} per round ({n} lines, two ply-2 entries each).")
    out["batch4"] = dict(original=None, like=like, note=note, binaries=binaries,
                         rounds_from=dict(original=like, ply2_games_per_cycle=round(mean, 2), lines=n,
                                          games_per_round=2 * n * (2 * n - 1)))
    return out


def build_extra_config(spec):
    ldir, cfg = LG.make_config(spec)
    set_binaries(cfg, spec)
    return ldir, cfg


def seeds_in_use(exclude):
    """{seed: [experiments]} of every experiment's config and every spec under leagues/specs but exclude's."""
    out = {}
    for d in sorted(os.listdir(os.path.join(ROOT, "experiments"))):
        c = LG.read_json(os.path.join(ROOT, "experiments", d, "config.json"))
        if c and c.get("name") != exclude:
            out.setdefault(c.get("seed"), []).append(d)
            out.setdefault((c.get("train") or {}).get("seed"), []).append(d + " (train)")
    for base, _, files in os.walk(os.path.join(ROOT, "specs")):
        for f in sorted(files):
            if f.endswith(".json"):
                s = LG.read_json(os.path.join(base, f)) or {}
                if isinstance(s, dict) and s.get("kind") == "experiment" and s.get("name") != exclude:
                    for x in (s.get("seed"), (s.get("train") or {}).get("seed")):
                        if x is not None:
                            out.setdefault(x, []).append(os.path.relpath(os.path.join(base, f), ROOT))
    return out


def check_extra(name, cfg, spec):
    """Problems of an extra's config: every line must be a replicate of its model source's line with the same
    recipe and arguments (only the names, replicate and seed offset differ), the top level as the source's but the
    seeds, which must be the extra's own and used nowhere else, and batch 4's keys."""
    like = EXTRA[name]
    ocfg, _, _ = original(like)
    rf = spec["batch4"]["rounds_from"]
    rounds = max(1, round(rf["ply2_games_per_cycle"] / rf["games_per_round"]))
    n = len(ocfg["lines"]) * REPLICATES
    p = []
    if rf["lines"] != n or rf["games_per_round"] != 2 * n * (2 * n - 1):
        p.append("the spec's lines or games per round")
    sbin = spec_binaries(spec)
    if set(sbin) != {lc["line"] for lc in cfg["lines"]}:
        p.append("the spec's binaries name other lines")
    base = {lc["model"]: lc for lc in ocfg["lines"]}
    reps = {}
    for lc in cfg["lines"]:
        mid, _, r = lc["model"].rpartition("#")
        b = base.get(mid)
        if b is None or not r.isdigit():
            p.append(f"{lc['line']}: not a replicate of a line of {like}")
            continue
        reps.setdefault(mid, []).append(int(r))
        if (lc["replicate"], lc["seed_offset"], lc["line"]) != (int(r), int(r) - 1, f"{lc['model']}.loc"):
            p.append(f"{lc['line']}: replicate, seed offset or name")
        if lc["line"] in sbin and sbin[lc["line"]][0] != b["train_binary"]:
            p.append(f"{lc['line']}: the spec's train binary isn't {like}'s ({b['train_binary']})")
        for k in sorted(set(b) | set(lc)):
            if k in ("line", "model", "label", "replicate", "seed_offset"):
                continue
            want = sbin.get(lc["line"], (None,))[0] if k == "train_binary" else b.get(k)
            if lc.get(k) != want:
                p.append(f"{lc['line']} {k}: {b.get(k)!r} -> {lc.get(k)!r}")
    if sorted(reps) != sorted(base) or any(sorted(v) != list(range(1, REPLICATES + 1)) for v in reps.values()):
        p.append(f"replicates: {reps}")
    for k in SAME_TOP:
        if k not in ("seed", "train") and ocfg.get(k) != cfg.get(k):
            p.append(f"{k}: {json.dumps(ocfg.get(k))[:80]} -> {json.dumps(cfg.get(k))[:80]}")
    seed = EXTRA_SEED[name]
    if cfg["seed"] != seed or cfg["train"] != dict(ocfg["train"], seed=seed):
        p.append(f"seeds {cfg['seed']} {cfg['train']}")
    used = seeds_in_use(name).get(seed)
    if used:
        p.append(f"seed {seed} is also used by {used}")
    for k in ("flags", "game_cpu"):
        if ocfg["tournament"].get(k) != cfg["tournament"].get(k):
            p.append(f"tournament {k}")
    want = dict(name=name, references=[], ply1_entries=False, plateau=PLATEAU, cycles=CAP, rounds_per_cycle=rounds,
                min_rounds=rounds, rating_zero=LG.entry_name(cfg["lines"][0]["line"], 0, 2))
    for k, v in want.items():
        if cfg.get(k) != v:
            p.append(f"{k}: {cfg.get(k)!r}, want {v!r}")
    if cfg["tournament"]["binary"] not in TOURNAMENTS:
        p.append("tournament binary")
    extra = set(cfg) - set(ocfg) - {"ply1_entries", "plateau", "rating_zero"}
    if extra:
        p.append(f"keys the source hasn't: {sorted(extra)}")
    missing = set(ocfg) - set(cfg)
    if missing:
        p.append(f"keys the source has: {sorted(missing)}")
    return p


def cmd_create_extra(name, spec_file=None):
    spec_file = spec_file or extra_spec_path(name)
    spec = LG.read_json(spec_file)
    ldir, cfg = build_extra_config(spec)
    p = check_extra(name, cfg, spec)
    if p:
        sys.exit(f"{name}: the spec doesn't make the planned experiment:\n  " + "\n  ".join(p))
    if os.path.exists(os.path.join(ldir, "config.json")):
        sys.exit(f"{ldir} already has a league")
    LG.cmd_create(spec_file)
    cpath = os.path.join(ldir, "config.json")
    made = LG.read_json(cpath)
    before = copy.deepcopy(made)
    set_binaries(made, spec)
    changed = [lc["line"] for a, lc in zip(before["lines"], made["lines"]) if a != lc]
    LG.write_json(cpath, made)  # atomic; before cycle 1 (version 0 is made in cycle 1 by the line's train binary)
    p = check_extra(name, LG.read_json(cpath), spec)
    if p:
        sys.exit(f"{name}: the written config differs from the plan:\n  " + "\n  ".join(p))
    L = LG.League(ldir)
    LG.write_lines_csv(L)
    L.log(f"batch 4: train binary {os.path.basename(TRAIN)} set on {len(changed)} lines; checked against "
          f"{EXTRA[name]} ({REPLICATES} replicates of each line)")
    print(f"{ldir}: created; checked against {EXTRA[name]}")


def cmd_create_item(name):
    if name in EXTRA:
        return cmd_create_extra(name)
    orig = name[:-len("-new")] if name.endswith("-new") else None
    if orig not in ORDER:
        sys.exit(f"{name}: not in the queue")
    return cmd_create(orig)


def cmd_specs(only=None):
    if only and only not in ORIGINAL_SPECS and only not in EXTRA:
        sys.exit(f"{only}: neither an original nor an extra")
    os.makedirs(HERE, exist_ok=True)
    for orig in ([only] if only in ORIGINAL_SPECS else [] if only else ORIGINAL_SPECS):
        path = spec_path(orig)
        if os.path.exists(os.path.join(ROOT, "experiments", new_name(orig), "config.json")):
            print(f"{new_name(orig)}: created already; its spec stays")
            continue
        LG.write_json(path, make_spec(orig))
        print(path)
    for name in ([only] if only in EXTRA else [] if only else EXTRA):
        if os.path.exists(os.path.join(ROOT, "experiments", name, "config.json")):
            print(f"{name}: created already; its spec stays")
            continue
        LG.write_json(extra_spec_path(name), make_extra_spec(name))
        print(extra_spec_path(name))


def cmd_check(only=None):
    bad = 0
    print(f"{'original':24s} {'lines':>5s} {'rounds':>6s} {'ply-2 games':>11s} {'new rounds':>10s} "
          f"{'new games':>9s}  binaries (train; self-play kept)")
    for orig in ([only] if only and only in ORIGINAL_SPECS else [] if only else ORIGINAL_SPECS):
        spec = LG.read_json(spec_path(orig))
        _, cfg = build_config(orig, spec)
        p = check(orig, cfg, spec)
        ocfg, _, _ = original(orig)
        rf = spec["batch4"]["rounds_from"]
        rounds, mean, n = cfg["rounds_per_cycle"], rf["ply2_games_per_cycle"], rf["lines"]
        tb = {}
        for lc in cfg["lines"]:
            tb[os.path.basename(lc["train_binary"])] = tb.get(os.path.basename(lc["train_binary"]), 0) + 1
        sb = sorted({os.path.basename(lc["selfplay_binary"]) for lc in cfg["lines"] if lc.get("selfplay_binary")})
        print(f"{orig:24s} {n:5d} {ocfg['rounds_per_cycle']:6d} {mean:11.0f} {rounds:10d} "
              f"{rounds * 2 * n * (2 * n - 1):9d}  {', '.join(f'{k} x{v}' for k, v in tb.items())}"
              + (f"; {', '.join(sb)}" if sb else "") + ("" if not p else "  PROBLEMS"))
        for x in p:
            print("    " + x)
        bad += bool(p)
    for name in ([] if only and only not in EXTRA else [only] if only else EXTRA):
        spec = LG.read_json(extra_spec_path(name))
        _, cfg = build_extra_config(spec)
        p = check_extra(name, cfg, spec)
        rf = spec["batch4"]["rounds_from"]
        tb = sorted({os.path.basename(lc["train_binary"]) for lc in cfg["lines"]})
        print(f"{name:24s} {rf['lines']:5d} {'-':>6s} {rf['ply2_games_per_cycle']:11.0f} {cfg['rounds_per_cycle']:10d} "
              f"{cfg['rounds_per_cycle'] * rf['games_per_round']:9d}  {', '.join(tb)} x{len(cfg['lines'])}; seed "
              f"{cfg['seed']}; replicates of {EXTRA[name]}" + ("" if not p else "  PROBLEMS"))
        for x in p:
            print("    " + x)
        bad += bool(p)
    if bad:
        sys.exit(f"{bad} specs with problems")


def cmd_create(orig):
    if orig in COVERED or orig in HELD:
        sys.exit(f"{orig}: {'covered by ' + new_name(COVERED[orig]) if orig in COVERED else 'held'}; not created")
    spec = LG.read_json(spec_path(orig))
    ldir, cfg = build_config(orig, spec)
    p = check(orig, cfg, spec)
    if p:
        sys.exit(f"{orig}: the spec doesn't keep the original's design:\n  " + "\n  ".join(p))
    if os.path.exists(os.path.join(ldir, "config.json")):
        sys.exit(f"{ldir} already has a league")
    LG.load_recipes = lambda path=None: patched_recipes(orig, spec)
    try:
        LG.cmd_create(spec_path(orig))
    finally:
        LG.load_recipes = ENGINE_LOAD_RECIPES
    cpath = os.path.join(ldir, "config.json")
    made = LG.read_json(cpath)
    before = copy.deepcopy(made)
    set_binaries(made, spec)
    changed = [lc["line"] for a, lc in zip(before["lines"], made["lines"]) if a != lc]
    LG.write_json(cpath, made)  # atomic; before cycle 1 (version 0 is made in cycle 1 by the line's train binary)
    p = check(orig, LG.read_json(cpath), spec)
    if p:
        sys.exit(f"{orig}: the written config differs from the original's design:\n  " + "\n  ".join(p))
    L = LG.League(ldir)
    LG.write_lines_csv(L)
    L.log(f"batch 4: binaries set on {len(changed)} lines (train {os.path.basename(TRAIN)} unless chessai-target, "
          f"chessai-trigger or search labels; self-play kept); checked against {orig}")
    print(f"{ldir}: created; checked against {orig}")


def status_of(orig):
    return status_of_name(new_name(orig))


def status_of_name(name):
    st = LG.read_json(os.path.join(ROOT, "experiments", name, "state.json"))
    return (st or {}).get("status"), (st or {}).get("cycle", 0), st


def cmd_state(running=None, note=""):
    """experiments/batch4-queue.txt: the queue's order and each rerun's state (done, running, next, waiting, held,
    covered), with its cycle, the last crowning and the stop reason."""
    out = [f"# Batch-4 queue (leagues/specs/batch4/queue.sh): one experiment at a time, in this order (the reruns, "
           f"then small-field-x4). Updated {LG.now()}.",
           "# Columns: position | experiment | state | cycle | last crowning | note. States: done, running, next,",
           "# waiting, halted (stopped or failed: the queue stops), held (spec only), covered (another rerun stands in)."]
    if note:
        out.append(f"# {note}")
    nxt = None
    rows = []
    for i, name in enumerate(items(), 1):
        status, cycle, st = status_of_name(name)
        last = (st or {}).get("last_crown_cycle", "")
        detail = ""
        if status == "done":
            state = "done"
            ev = [r for r in LG.read_csv(os.path.join(ROOT, "experiments", name, "events.csv")) if r["event"] == "done"]
            detail = ev[-1]["detail"] if ev else ""
        elif (running and name in (running, new_name(running))) or status in ("running", "waiting"):
            state = "running"
        elif status in ("stopped", "failed") and cycle > 0:
            state = "halted"
            detail = f"status {status}"
        elif nxt is None:
            state = nxt = "next"
        else:
            state = "waiting"
        rows.append(f"{i} | {name} | {state} | {cycle} | {last} | {detail}")
    for orig in HELD:
        rows.append(f"- | {new_name(orig)} | held | | | spec {os.path.relpath(spec_path(orig), ROOT)}; not started")
    for orig, by in COVERED.items():
        rows.append(f"- | {new_name(orig)} | covered | | | by {new_name(by)} (same design without references)")
    LG.atomic_write(QUEUE_TXT, "\n".join(out + rows) + "\n")


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    cmd = sys.argv[1]
    if cmd == "specs":
        cmd_specs(sys.argv[2] if len(sys.argv) > 2 else None)
    elif cmd == "check":
        cmd_check(sys.argv[2] if len(sys.argv) > 2 else None)
    elif cmd == "create":
        cmd_create(sys.argv[2])
    elif cmd == "order":
        print(" ".join(ORDER))
    elif cmd == "status":
        print(status_of(sys.argv[2])[0] or "")
    elif cmd == "items":
        print(" ".join(items()))
    elif cmd == "status-of":
        print(status_of_name(sys.argv[2])[0] or "")
    elif cmd == "create-item":
        cmd_create_item(sys.argv[2])
    elif cmd == "state":
        cmd_state(sys.argv[2] if len(sys.argv) > 2 and sys.argv[2] else None,
                  sys.argv[3] if len(sys.argv) > 3 else "")
    else:
        sys.exit(__doc__)


if __name__ == "__main__":
    main()
