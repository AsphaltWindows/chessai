#!/usr/bin/env python3
"""Writes leagues/recipes.txt from the live league's registry, read-only: one row per experiment model (a recipe plus
its self-play settings), each naming its training recipe. The global league uses the distinct recipes; an
experiment picks models. Source: the lineages active at cycle SNAPSHOT of cycles/champ/cycles.csv, their flags in
cycles/champ/lineages.txt and their own-step binaries in cycles/champ/config.json (own_games). Nothing live is
written.

    python3 leagues/make_recipes.py [--check]
"""
import csv
import json
import os
import re
import shlex
import sys

ROOT = "/home/iv/dev/chessai"
CHAMP = os.path.join(ROOT, "cycles/champ")
OUT = os.path.join(ROOT, "leagues/recipes.txt")
# old binary -> its rebuild (the same games and models), applied in this order: the fast rebuilds, then the
# fixed-point one (hybrid4g -> hybrid4gfx)
MAPS = [os.path.join(ROOT, "leagues/binaries-fast.txt"), os.path.join(ROOT, "leagues/binaries-fx.txt")]
SNAPSHOT = 64  # the design's count: 70 active lineages at cycle 64 -> 54 distinct training recipes
# Lineages taken beyond the snapshot: the default flat NB and probit leaves, which the planned experiments use
# (F-N-c retired at cycle 57; F-P-c has been frozen since cycle 37). Their rows are models F-N and F-P.
EXTRA = ["F-N-c", "F-P-c"]
# flags of self-play only (the live driver's SELFPLAY_ONLY): dropped from the training recipe
SELFPLAY_ONLY = {"--model", "--seed", "--rounds", "--games", "--policy", "--temperature", "--threads", "--vs-previous",
                 "--no-games", "--depth", "--round-forget", "--age-forget", "--resume", "--keep-states", "--init"}
# lineage-name parts that only name self-play settings or a run's history, not training
DROP = {"d0soft", "d1jit", "d1play", "d2jit", "d2play", "d4jit", "solo", "new", "agdiv"}
MIX = {"d2mix", "d4mix"}  # 50/50 result + depth-2 search labels: the training recipe keeps them, as "mix"


def parse_flags(tokens):
    out, i = [], 0
    while i < len(tokens):
        t = tokens[i]
        if i + 1 < len(tokens) and not tokens[i + 1].startswith("--"):
            out.append((t, tokens[i + 1]))
            i += 2
        else:
            out.append((t, True))
            i += 1
    return out


def train_args(flags):
    """The recipe's options for `train` (the live own step's model_args): the flags minus the self-play-only ones,
    labels kept."""
    out = []
    for k, v in flags:
        if k not in SELFPLAY_ONLY:
            out += [k] if v is True else [k, v]
    return out


def train_binary_of(name, selfplay_binary, own):
    """The binary that trains the lineage's recipe on league games: the live own step's (own_games.binary_by_lineage,
    else own_games.binary). A solo lineage (self-play only) has no own step: it takes its non-solo twin's, else its
    self-play binary when that isn't the default chessai-cyc5 (own_games.binary is chessai-cyc5 plus train labels)."""
    bbl = own.get("binary_by_lineage", {})
    for n in (name, name.replace("-solo", "")):
        if n in bbl:
            return bbl[n]
    return own["binary"] if os.path.basename(selfplay_binary) == "chessai-cyc5" else selfplay_binary


def repo_rel(token):
    """A binary named in binaries-fast.txt as a repo-relative path (a bare name is in models/from-scratch/bin)."""
    p = token if "/" in token else os.path.join("models/from-scratch/bin", token)
    return os.path.relpath(os.path.join(ROOT, p), ROOT)


def file_md5(path):
    import hashlib
    with open(path, "rb") as f:
        return hashlib.md5(f.read()).hexdigest()


def read_binary_map(path):
    """{old binary: new binary} (repo-relative) from a binaries-*.txt file: its table rows
    `old | old md5 | new | new md5 | ...` (both md5s are checked against the files), or lines `old -> new`. Other
    lines (`#` comments, binary lists, by-row lists) are skipped. {} without the file."""
    out = {}
    if not os.path.exists(path):
        return out
    with open(path) as f:
        for line in f:
            line = line.split("#", 1)[0].strip()
            if not line:
                continue
            fields = [x.strip() for x in line.split("|")]
            if (len(fields) >= 4 and "chessai" in fields[0] and "chessai" in fields[2]
                    and all(re.fullmatch(r"[0-9a-f]{32}", fields[i]) for i in (1, 3))):
                pairs = [(fields[0], fields[1]), (fields[2], fields[3])]
            elif len(fields) == 1 and re.fullmatch(r"\S*chessai\S*\s*->\s*\S*chessai\S*", line):
                pairs = [(x.strip(), None) for x in line.split("->")]
            else:
                continue
            for b, md5 in pairs:
                full = os.path.join(ROOT, repo_rel(b))
                if not os.access(full, os.X_OK):
                    sys.exit(f"{path}: {b} is not an executable")
                if md5 and file_md5(full) != md5:
                    sys.exit(f"{path}: {b} has md5 {file_md5(full)}, not {md5}")
            out[repo_rel(pairs[0][0])] = repo_rel(pairs[1][0])
    return out


def recipe_name(lineage):
    parts = lineage[:-2].split("-") if lineage.endswith("-c") else lineage.split("-")
    return "-".join("mix" if p in MIX else p for p in parts if p not in DROP)


def main():
    with open(os.path.join(CHAMP, "config.json")) as f:
        cfg = json.load(f)
    own = cfg["own_games"]
    active = []
    with open(os.path.join(CHAMP, "cycles.csv")) as f:
        for r in csv.DictReader(f):
            if r["cycle"] == str(SNAPSHOT) and r["status"] == "active":
                active.append(r["lineage"])
    n_snapshot = len(active)
    active += [x for x in EXTRA if x not in active]
    reg = {}
    with open(os.path.join(CHAMP, "lineages.txt")) as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            p = [x.strip() for x in line.split(" | ")]
            if len(p) == 7:
                reg[p[0]] = dict(archetype=p[1], binary=p[2], heldout=p[4], tokens=shlex.split(p[6]))
    rows = []
    for name in sorted(active):
        if name not in reg:
            sys.exit(f"{name}: not in lineages.txt")
        r = reg[name]
        flags = parse_flags(r["tokens"])
        f = dict(flags)
        targs = train_args(flags)
        rows.append(dict(lineage=name, archetype=r["archetype"], kind=f["--model"], seed=f["--seed"],
                         train_binary=train_binary_of(name, r["binary"], own),
                         selfplay_binary=r["binary"], train_args=targs, selfplay_args=r["tokens"],
                         heldout=r["heldout"]))
    # recipes: the same model kind, seed, training flags and train binary
    recipes = {}
    for r in rows:
        key = (r["kind"], r["seed"], tuple(r["train_args"]), r["train_binary"])
        recipes.setdefault(key, []).append(r)
    for key, members in recipes.items():
        if len({m["heldout"] for m in members}) > 1:
            sys.exit(f"recipe {key}: members differ in their held-out set")
    names = {}
    for key, members in recipes.items():
        cands = sorted({recipe_name(m["lineage"]) for m in members}, key=lambda s: (len(s), s))
        names[key] = cands[0]
    # Recipes that would share a name differ in a training flag (the d2jit trees' smaller --split-threshold): the
    # one with the most common value keeps the name, the others get a suffix naming their value (-st100k).
    by_name = {}
    for key, n in names.items():
        by_name.setdefault(n, []).append(key)
    for n, keys in by_name.items():
        if len(keys) < 2:
            continue
        flags = [dict(parse_flags(list(k[2]))) for k in keys]
        differ = sorted({f for d in flags for f in d if len({str(x.get(f)) for x in flags}) > 1})
        if differ != ["--split-threshold"]:
            sys.exit(f"recipe names clash: {n} differ in {differ}")
        for key, d in zip(keys, flags):
            v = int(d["--split-threshold"])
            if v != 5000000:
                names[key] = f"{n}-st{v // 1000000}M" if v % 1000000 == 0 else f"{n}-st{v // 1000}k"
    clash = [n for n in names.values() if list(names.values()).count(n) > 1]
    if clash:
        sys.exit(f"recipe names clash: {sorted(set(clash))}")
    # models: a recipe plus its self-play settings (duplicate settings collapse)
    models = {}
    for key, members in sorted(recipes.items(), key=lambda kv: names[kv[0]]):
        for m in sorted(members, key=lambda m: (len(m["lineage"]), m["lineage"])):
            sp = tuple(m["selfplay_args"])
            mk = (names[key], sp)
            if mk in models:
                models[mk]["sources"].append(m["lineage"])
                continue
            base = m["lineage"][:-2] if m["lineage"].endswith("-c") else m["lineage"]
            for d in ("-solo", "-new", "-agdiv"):
                base = base.replace(d, "")
            models[mk] = dict(m, model=base, recipe=names[key], sources=[m["lineage"]])
    mnames = [m["model"] for m in models.values()]
    dup = sorted({n for n in mnames if mnames.count(n) > 1})
    if dup:
        sys.exit(f"model names clash: {dup}")
    maps = [(p, read_binary_map(p)) for p in MAPS]
    chains = {}  # live binary -> [its rebuilds in turn]
    for m in models.values():
        for k in ("train_binary", "selfplay_binary"):
            b = os.path.relpath(os.path.join(ROOT, m[k]), ROOT)
            chain = [b]
            for _, mp in maps:
                if chain[-1] in mp:
                    chain.append(mp[chain[-1]])
            if len(chain) > 1:
                chains[b] = chain
            m[k] = chain[-1]
    lines = [
        "# Recipes and models of the new league system (written by leagues/make_recipes.py from the live registry,",
        f"# lineages active at cycle {SNAPSHOT}; edit by hand to add more). One row per model, fields separated by ' | ':",
        "#   model | recipe | archetype | kind | seed | train_binary | selfplay_binary | train_args | selfplay_args | sources"
        " | heldout",
        "# - model: the name an experiment uses; recipe: the training recipe, the name the global league uses (rows with",
        "#   the same recipe train identically; their self-play settings differ).",
        "# - train_args: the options of `train` (as the live own step's: the flags minus the self-play-only ones, labels",
        "#   kept); the driver adds --model KIND --seed SEED --rounds 1 [--start ...].",
        "# - selfplay_args: the full self-play recipe (as lineages.txt), without --rounds/--games/--threads/--resume/--init.",
        "# - sources: the live lineages this row came from.",
        "# - heldout: the held-out set (config calibration.heldout) its own calibration games are played with, as the",
        "#   live league's (lineages.txt's heldout field); - for none.",
        f"# Also taken, though not active at cycle {SNAPSHOT}: {', '.join(EXTRA)} (make_recipes.py's EXTRA: the "
        "default flat",
        "#   NB and probit leaves, models F-N and F-P).",
    ]
    if chains:
        lines.append("# Binaries: rebuilds that play the same games and write the same models replace the live ones ("
                     + ", then ".join(os.path.relpath(p, ROOT) for p, mp in maps if mp) + "):")
        lines += [f"#   {' -> '.join(chains[b])}" for b in sorted(chains)]
    for m in sorted(models.values(), key=lambda m: (m["recipe"], m["model"])):
        lines.append(" | ".join([m["model"], m["recipe"], m["archetype"], m["kind"], m["seed"],
                                 os.path.relpath(os.path.join(ROOT, m["train_binary"]), ROOT),
                                 os.path.relpath(os.path.join(ROOT, m["selfplay_binary"]), ROOT),
                                 shlex.join(m["train_args"]) or "-", shlex.join(m["selfplay_args"]),
                                 " ".join(m["sources"]), m["heldout"]]))
    text = "\n".join(lines) + "\n"
    print(f"{n_snapshot} active lineages at cycle {SNAPSHOT} and {len(active) - n_snapshot} extra -> {len(recipes)} "
          f"recipes, {len(models)} models")
    if "--check" in sys.argv:
        return
    tmp = OUT + ".tmp"
    with open(tmp, "w") as f:
        f.write(text)
    os.replace(tmp, OUT)
    print(f"wrote {OUT}")


if __name__ == "__main__":
    main()
