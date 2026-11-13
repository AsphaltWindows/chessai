#!/usr/bin/env python3
"""Arenas: fixed, already-trained models play a tournament and get rated. No lines, no training, no versions
(leagues/ARENA.md). An arena lives in leagues/arenas/<name>/ and is made from a spec.

    python3 leagues/arena.py create SPEC.json               make leagues/arenas/<name>/ (or the spec's "dir")
    python3 leagues/arena.py run NAME|DIR [--rounds N] [--threads T]
                                                            own calibration games, then round-robins until the stop
                                                            rule holds (or N more round-robins)
    python3 leagues/arena.py status NAME|DIR [--vs PLAYER]  the ratings; --vs: one player's head-to-head results
    python3 leagues/arena.py fit NAME|DIR [--write] [--top N]
                                                            the fit report of any arena: the Bradley-Terry residuals
                                                            (and, with a pinned anchor, the pinned ratings)

An entry is a saved experiment version, "<experiment>:<line>@<version>" (its model and the calibration it had
there), or a model file with its kind. Each plays at the plies listed (default: ply 2 only, the ply ratings are
quoted at), beside the random references. The tournament binary and its files are the experiments' (tour/:
games.csv, ratings.csv, head_to_head.csv, ...), so the experiments' tools read an arena's tour/ as they read theirs.

A spec with "anchor": {"mean_of": [players] | "field"} (11-12, the global ladder; ARENA.md, Pinned anchor) has no
random references: the mean of the listed players (or of every player) is pinned at Elo 0, the ratings and their
95% intervals relative to it are pinned.csv, the stop rule reads those intervals, and each chunk writes the fit
report (fit.md, residuals.csv). An entry's "ply" (one ply) or "plies" may differ between entries: a mixed-ply arena.

This script only reads leagues.py (its helpers); it never changes a league.
"""
import argparse
import filecmp
import hashlib
import json
import math
import os
import re
import shutil
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import leagues as LG  # noqa: E402  (helpers only: nothing in it runs on import)

ARENAS = os.path.join(HERE, "arenas")
EXPERIMENTS = os.path.join(HERE, "experiments")
DEFAULT_BINARY = os.path.join(LG.BIN, "chessai-hybrid7")
NAME_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9_.+=#@-]*$")
VERSION_RE = re.compile(r"^(?P<exp>[^:]+):(?P<line>.+)@(?P<v>\d+)$")
CAL_RE = re.compile(r"^calibration on (?P<path>\S+): (?P<params>\S+) \((?P<n>\d+) positions\)$")
DEFAULTS = {
    "seed": 1, "nice": 15, "threads": 2, "jobs": 2,
    "binary": DEFAULT_BINARY, "flags": ["--reports-last", "--sparse-fit"],
    "references": ["random", "random-1", "random-2", "random-3"],
    "plies": [2],
    "calibration": LG.DEFAULTS["calibration"],  # for file entries; experiment entries use their experiment's
    "owncal_binary": None,  # for file entries (None: "binary")
    "keep_own_games": False,  # False: a run removes the own games when it ends (the next run restores or makes them)
    "owncal_cache": True,  # False: no shared cache; every run that needs the own games makes them (as before 11-11)
    "stop": {"ci95": 20.0, "chunk_rounds": 10, "min_rounds": 10, "max_rounds": 200},
}
CALCHECK_FIELDS = ["entry", "source", "expected", "arena", "identical"]
# Own calibration games are a deterministic function of the model, its kind, the calibration settings, the held-out
# games and the binary that makes them (own_cache_key), so arenas share them in one cache (ARENA.md, Calibration).
# CHESSAI_OWNCAL_CACHE moves it (for tests).
OWNCAL_CACHE = os.environ.get("CHESSAI_OWNCAL_CACHE") or os.path.join(ARENAS, ".owncal-cache")
OWNCAL_CACHE_CAP = 20e9  # bytes (apparent sizes); past it the entries used longest ago go first
ELO_UNIT = 400 / math.log(10)  # Elo per natural unit of the Bradley-Terry fit (the binary's ELO)
PINNED_FIELDS = ["rank", "player", "ply", "elo", "ci95", "se", "games", "score", "reference", "binary_elo",
                 "binary_ci95", "label"]
RESIDUAL_FIELDS = ["a", "b", "ply_a", "ply_b", "games", "wins", "losses", "draws", "observed", "predicted", "z",
                   "dev_elo", "se_elo"]


def arena_dir(name_or_dir):
    d = name_or_dir if os.sep in name_or_dir or os.path.isdir(name_or_dir) else os.path.join(ARENAS, name_or_dir)
    return os.path.abspath(d)


def repo_path(p):
    return p if os.path.isabs(p) else os.path.join(LG.REPO, p)


def expected_calibration(log_path, owncal_path):
    """The calibration the experiment's tournament fitted on a version's own games, as its log printed it
    ("params (N positions)"), or None."""
    if not os.path.exists(log_path):
        return None
    want = f"calibration on {owncal_path}: "
    with open(log_path, errors="replace") as f:
        for s in f:
            if s.startswith(want):
                m = CAL_RE.match(s.rstrip("\n"))
                if m and m["path"] == owncal_path:
                    return f"{m['params']} ({m['n']} positions)"
    return None


def resolve_entry(spec, e, adir):
    """One spec entry, resolved: name, kind, plies, model (the arena's copy), source, md5, heldout path, the
    calibration settings and binary of its own games, and (experiment versions) the calibration it had there."""
    if "ply" in e:  # one ply for this entry (a mixed-ply arena gives each entry its own)
        if "plies" in e:
            sys.exit(f"entry {e}: give ply or plies, not both")
        plies = [int(e["ply"])]
    else:
        plies = [int(x) for x in e.get("plies", spec["plies"])]
    if "version" in e:
        m = VERSION_RE.match(e["version"])
        if not m:
            sys.exit(f"entry {e}: version must be <experiment>:<line>@<version>")
        exp, line, v = m["exp"], m["line"], int(m["v"])
        xdir = exp if os.path.isabs(exp) else os.path.join(EXPERIMENTS, exp)
        xcfg = LG.read_json(os.path.join(xdir, "config.json"))
        if not xcfg:
            sys.exit(f"entry {e}: no experiment config in {xdir}")
        lc = next((x for x in xcfg["lines"] if x["line"] == line), None)
        if lc is None:
            sys.exit(f"entry {e}: {exp} has no line {line}")
        kind = lc["kind"]
        src = os.path.join(xdir, "runs", line, f"{kind}{v}.model")
        cal = xcfg["calibration"]
        held = (cal.get("heldout") or {}).get(lc.get("heldout")) if lc.get("heldout") else None
        name = e.get("name", f"{line}@{v}")
        out = dict(name=name, kind=kind, source=e["version"], source_model=src, heldout=held,
                   calibration={k: cal[k] for k in ("games", "depth", "seed", "policy", "temperature")},
                   owncal_binary=xcfg["tournament"]["binary"],
                   expected_calibration=expected_calibration(os.path.join(xdir, "tour", "tournament.log"),
                                                             os.path.join(xdir, "owncal", LG.own_key(line, v))))
    elif "file" in e:
        if "kind" not in e or "name" not in e:
            sys.exit(f"entry {e}: a file entry needs kind and name")
        src = os.path.abspath(repo_path(e["file"]))
        cal = spec["calibration"]
        heldout = e.get("heldout")
        held = (cal.get("heldout") or {}).get(heldout) if heldout else None
        if heldout and not held:
            sys.exit(f"entry {e}: no held-out games named {heldout!r} in the calibration settings")
        out = dict(name=e["name"], kind=e["kind"], source=e["file"], source_model=src, heldout=held,
                   calibration={k: cal[k] for k in ("games", "depth", "seed", "policy", "temperature")},
                   owncal_binary=spec["owncal_binary"] or spec["binary"], expected_calibration=None)
    else:
        sys.exit(f"entry {e}: needs version or file")
    if not NAME_RE.match(out["name"]):
        sys.exit(f"entry {e}: bad name {out['name']!r}")
    if not os.path.exists(out["source_model"]):
        sys.exit(f"entry {e}: no model file {out['source_model']}")
    out["md5"] = LG.file_md5(out["source_model"])
    out["model"] = os.path.join(adir, "models", f"{out['name']}.{out['kind']}.model")
    out["plies"] = plies
    for k in ("label", "positions", "note"):
        if k in e:
            out[k] = e[k]
    return out


def check_anchor(raw, entries):
    """The spec's pinned anchor, checked: {"mean_of": "field" | [player names]} (a player is <entry name>-<ply>).
    None without one."""
    anchor = raw.get("anchor")
    if anchor is None:
        return None
    if not isinstance(anchor, dict) or "mean_of" not in anchor or set(anchor) - {"mean_of", "note"}:
        sys.exit('anchor: must be {"mean_of": "field" or [player names]} (and an optional "note")')
    players = [f"{e['name']}-{p}" for e in entries for p in e["plies"]]
    mo = anchor["mean_of"]
    if mo == "field":
        return anchor
    if not isinstance(mo, list) or not mo or not all(isinstance(x, str) for x in mo):
        sys.exit('anchor: mean_of must be "field" or a non-empty list of player names (<entry name>-<ply>)')
    if len(set(mo)) != len(mo):
        sys.exit(f"anchor: players listed twice: {sorted(x for x in set(mo) if mo.count(x) > 1)}")
    unknown = [x for x in mo if x not in players]
    if unknown:
        sys.exit(f"anchor: not players of this arena: {unknown} (a player is <entry name>-<ply>)")
    return anchor


def cmd_create(spec_path):
    raw = LG.read_json(spec_path)
    spec = LG.deep_merge(DEFAULTS, raw)
    if raw.get("anchor") is not None:  # the pinned anchor replaces the random references (ARENA.md, Pinned anchor)
        if raw.get("references"):
            sys.exit("anchor: the pinned reference set replaces the random references; leave references out or []")
        spec["references"] = []
    if not spec.get("name") or not NAME_RE.match(spec["name"]):
        sys.exit("the spec needs a name")
    for k in ("binary", "owncal_binary"):
        if spec[k]:
            spec[k] = os.path.abspath(repo_path(spec[k]))
            if not os.access(spec[k], os.X_OK):
                sys.exit(f"{k}: no binary {spec[k]}")
    adir = os.path.abspath(repo_path(spec["dir"])) if spec.get("dir") else os.path.join(ARENAS, spec["name"])
    if os.path.exists(adir):
        sys.exit(f"{adir} exists")
    for r in spec["references"]:
        if r not in LG.REFERENCE_PLY:
            sys.exit(f"unknown reference {r}")
    entries = [resolve_entry(spec, e, adir) for e in spec["entries"]]
    names = [e["name"] for e in entries]
    if len(set(names)) != len(names):
        sys.exit(f"duplicate entry names: {sorted(n for n in set(names) if names.count(n) > 1)}")
    anchor = check_anchor(raw, entries)
    os.makedirs(os.path.join(adir, "models"))
    os.makedirs(os.path.join(adir, "tour"))
    for e in entries:
        shutil.copy2(e["source_model"], e["model"])
        if LG.file_md5(e["model"]) != e["md5"]:
            sys.exit(f"{e['name']}: copy differs from {e['source_model']}")
    cfg = {k: spec[k] for k in ("name", "seed", "nice", "threads", "jobs", "binary", "flags", "references", "stop",
                                "keep_own_games", "owncal_cache")}
    if anchor is not None:  # only then: a spec without one makes the config it always made
        cfg["anchor"] = anchor
    cfg.update(kind="arena", created=LG.now(), description=spec.get("description", ""), entries=entries,
               spec=os.path.abspath(spec_path))
    LG.write_json(os.path.join(adir, "config.json"), cfg)
    LG.write_json(os.path.join(adir, "state.json"), dict(status="created", rounds=0, updated=LG.now(), chunks=[]))
    if anchor is None:
        print(f"created {adir}: {len(entries)} entries, {sum(len(e['plies']) for e in entries)} players besides "
              f"{len(cfg['references'])} references")
    else:
        mo = anchor["mean_of"]
        plies = sorted({p for e in entries for p in e["plies"]})
        print(f"created {adir}: {len(entries)} entries, {sum(len(e['plies']) for e in entries)} players at plies "
              f"{','.join(map(str, plies))}, no random references; anchor: the mean of "
              + ("every player" if mo == "field" else f"{len(mo)} players") + " at Elo 0")


class Arena:
    def __init__(self, path):
        self.dir = arena_dir(path)
        self.cfg = LG.read_json(self.p("config.json"))
        if not self.cfg or self.cfg.get("kind") != "arena":
            sys.exit(f"{self.dir}: not an arena")
        self.state = LG.read_json(self.p("state.json"), {})
        self.cache_hits = []  # entries whose own games came from the shared cache (make_own)

    def p(self, *parts):
        return os.path.join(self.dir, *parts)

    def save(self):
        self.state["updated"] = LG.now()
        LG.write_json(self.p("state.json"), self.state)

    def nice(self):
        return ["nice", "-n", str(self.cfg["nice"])]

    def log(self, msg):
        s = f"{LG.now()} {msg}"
        print(s, flush=True)
        with open(self.p("arena.log"), "a") as f:
            f.write(s + "\n")

    def owncal(self, e):
        return self.p("owncal", e["name"])

    def player_spec(self, e):
        return f"cal:{self.owncal(e)}:{e['kind']}:{e['model']}"

    def players(self):
        """[(name, ply, spec, entry or None)] in a fixed order: the references, then each entry at its plies."""
        out = [(r, LG.REFERENCE_PLY[r], "rand", None) for r in self.cfg["references"]]
        for e in self.cfg["entries"]:
            for ply in e["plies"]:
                out.append((f"{e['name']}-{ply}", ply, self.player_spec(e), e))
        return out


def own_done(a, e):
    d = a.owncal(e)
    return os.path.isdir(d) and all(os.path.exists(os.path.join(d, f)) for f in LG.GAME_FILES)


def own_play(e, model=None, heldout=None):
    """The player spec the own games are made with (the entry's held-out calibration, if any)."""
    model, heldout = model or e["model"], heldout or e["heldout"]
    return f"cal:{heldout}:{e['kind']}:{model}" if e["heldout"] else f"{e['kind']}:{model}"


def own_args(e, play, out):
    """The arguments (after the binary) of the `arena` command that makes an entry's own games."""
    o = e["calibration"]
    return ["arena", play, play, "--use-probs", "--games", str(o["games"]), "--a-policy", o["policy"],
            "--b-policy", o["policy"], "--temperature", str(o["temperature"]), "--a-depth", str(o["depth"]),
            "--b-depth", str(o["depth"]), "--seed", str(o["seed"]), "--out", out]


_MD5 = {}


def md5_of(path):
    """md5 of a file, memoised on its size, mtime and inode; of a directory, md5 over its files' names and md5s."""
    if os.path.isdir(path):
        h = hashlib.md5()
        for f in sorted(os.listdir(path)):
            if os.path.isfile(os.path.join(path, f)):
                h.update(f"{f}\0{md5_of(os.path.join(path, f))}\n".encode())
        return h.hexdigest()
    st = os.stat(path)
    sig = (os.path.abspath(path), st.st_size, st.st_mtime_ns, st.st_ino)
    if sig not in _MD5:
        _MD5[sig] = LG.file_md5(path)
    return _MD5[sig]


def own_cache_key(e):
    """(key, parts): the own games' cache key and everything it is made of, which is everything the games depend on:
    the `arena` command with the paths left out (kind, settings, flags), the model's md5 (the arena's copy), the
    held-out games' md5 (None without) and the own-games binary's md5. The games don't depend on the paths."""
    parts = dict(args=own_args(e, own_play(e, "<model>", "<heldout>"), "<out>"), model_md5=md5_of(e["model"]),
                 heldout_md5=md5_of(e["heldout"]) if e["heldout"] else None, binary_md5=md5_of(e["owncal_binary"]))
    return hashlib.sha256(json.dumps(parts, sort_keys=True).encode()).hexdigest()[:32], parts


def copy_games(src, dst):
    """Copies the game files of directory src into directory dst, reflinked where the filesystem can."""
    subprocess.run(["cp", "--reflink=auto", *[os.path.join(src, f) for f in LG.GAME_FILES], dst + "/"], check=True,
                   capture_output=True)


def cache_restore(key, dest):
    """Copies the cached own games of `key` into `dest`, a new directory, and checks each copy's size and md5
    against the entry's manifest (key.json). Returns the manifest, or None if the cache doesn't hold them whole."""
    src = os.path.join(OWNCAL_CACHE, key)
    try:
        man = LG.read_json(os.path.join(src, "key.json"))
        if not man or man.get("key") != key:
            return None
        os.makedirs(dest)
        copy_games(src, dest)
        for f in LG.GAME_FILES:
            p = os.path.join(dest, f)
            os.chmod(p, 0o644)
            if os.path.getsize(p) != man["files"][f]["size"] or LG.file_md5(p) != man["files"][f]["md5"]:
                raise ValueError(f"{p}: not the cached file's size and md5")
        os.utime(src)  # its last use, for cache_trim
        return man
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError):
        shutil.rmtree(dest, ignore_errors=True)
        return None


def cache_store(a, e, key, parts, games, cpu):
    """Puts own games just made (directory `games`) in the cache under `key`, read-only, with a manifest of their
    sizes and md5s. If the key is there already (another run made them meanwhile), compares the bytes instead: they
    must be equal, the games being deterministic in the key; if not, logs it and removes the cache's entry."""
    os.makedirs(OWNCAL_CACHE, exist_ok=True)
    final = os.path.join(OWNCAL_CACHE, key)
    if not os.path.isdir(final):
        tmp = os.path.join(OWNCAL_CACHE, f".tmp-{key}-{os.getpid()}-{id(e)}")
        shutil.rmtree(tmp, ignore_errors=True)
        os.makedirs(tmp)
        copy_games(games, tmp)
        files = {f: dict(size=os.path.getsize(os.path.join(tmp, f)), md5=LG.file_md5(os.path.join(tmp, f)))
                 for f in LG.GAME_FILES}
        for f in LG.GAME_FILES:
            os.chmod(os.path.join(tmp, f), 0o444)
        LG.write_json(os.path.join(tmp, "key.json"), dict(key=key, parts=parts, files=files, made=LG.now(),
                                                          arena=a.dir, entry=e["name"], cpu_s=round(cpu, 2)))
        try:
            os.rename(tmp, final)
            cache_trim()
            return
        except OSError:  # stored meanwhile: compare
            shutil.rmtree(tmp, ignore_errors=True)
    if not all(filecmp.cmp(os.path.join(games, f), os.path.join(final, f), shallow=False) for f in LG.GAME_FILES):
        a.log(f"WARNING: own games of {e['name']} differ from the cache's {key}; the cache entry is removed")
        shutil.rmtree(final, ignore_errors=True)


def cache_trim(cap=OWNCAL_CACHE_CAP):
    """While the cache holds more than `cap` bytes, removes the entries used longest ago (a directory's mtime: made
    or last restored); and leftovers of interrupted stores older than a day."""
    entries = []
    for name in os.listdir(OWNCAL_CACHE):
        p = os.path.join(OWNCAL_CACHE, name)
        try:
            st = os.stat(p)
            if name.startswith(".tmp-"):
                if time.time() - st.st_mtime > 86400:
                    shutil.rmtree(p, ignore_errors=True)
                continue
            entries.append((st.st_mtime, sum(os.path.getsize(os.path.join(p, f)) for f in os.listdir(p)), p))
        except OSError:
            continue
    total = sum(size for _, size, _ in entries)
    for _, size, p in sorted(entries):
        if total <= cap:
            break
        shutil.rmtree(p, ignore_errors=True)
        total -= size


def make_own(a, e, threads):
    """The entry's own calibration games, as the experiments make them (leagues.py make_own): the model plays itself
    at the calibration depth, log-odds softmax, its held-out calibration, the settings' seed; the tournament then
    fits its calibration on them. Deterministic, so an experiment version's games (and calibration) come out as
    they did there; and so, with owncal_cache (the default), they come from the shared cache when any arena made
    them before (cache_restore), and games made here go into it (cache_store). Returns the CPU s spent."""
    root = a.p("owncal")
    os.makedirs(root, exist_ok=True)
    final, tmp = a.owncal(e), os.path.join(root, f".tmp-{e['name']}")
    if own_done(a, e):
        return 0.0
    play = own_play(e)
    record = dict(play=play, model=e["model"], md5=e["md5"], binary=e["owncal_binary"], settings=e["calibration"])
    cached = a.cfg.get("owncal_cache", True)
    if cached:
        key, parts = own_cache_key(e)
        record["cache_key"] = key
    LG.write_json(os.path.join(root, e["name"] + ".json"), record)
    shutil.rmtree(tmp, ignore_errors=True)
    shutil.rmtree(final, ignore_errors=True)
    log = os.path.join(root, e["name"] + ".arena")
    if cached:
        man = cache_restore(key, tmp)
        if man:
            os.rename(tmp, final)
            with open(log, "a") as f:
                f.write(f"== own games of {e['name']} {LG.now()}: from the cache, {key} (made {man['made']} by "
                        f"{man['arena']} for {man['entry']}, {man['cpu_s']} CPU s)\n")
            a.cache_hits.append(e["name"])
            return 0.0
    cmd = a.nice() + [e["owncal_binary"], *own_args(e, play, tmp)]
    code, cpu, _, _ = LG.run_measured(cmd, log, f"own games of {e['name']}", env=LG.threads_env(threads))
    if code != 0 or not all(os.path.exists(os.path.join(tmp, f)) for f in LG.GAME_FILES):
        shutil.rmtree(tmp, ignore_errors=True)
        raise RuntimeError(f"own games of {e['name']} failed (exit {code}); see owncal/{e['name']}.arena")
    os.rename(tmp, final)
    if cached:
        cache_store(a, e, key, parts, final, cpu)
    return cpu


def rounds_done(tour):
    """Complete round-robins in tour/games.csv (the binary appends a round-robin's games when it ends)."""
    p = os.path.join(tour, "games.csv")
    if not os.path.exists(p):
        return 0
    last = -1
    with open(p) as f:
        next(f, None)
        for s in f:
            if s.endswith("\n"):
                last = max(last, int(s.split(",", 1)[0]))
    return last + 1


def clean_partial(tour, done):
    """Drops game_cpu.csv rows of a round-robin an interruption cut short (games.csv has only whole ones)."""
    p = os.path.join(tour, "game_cpu.csv")
    if not os.path.exists(p):
        return
    with open(p) as f:
        lines = f.readlines()
    keep = lines[:1] + [s for s in lines[1:] if s.endswith("\n") and int(s.split(",", 1)[0]) < done]
    if len(keep) != len(lines):
        LG.atomic_write(p, "".join(keep))


def read_ratings(tour):
    return {r["player"]: r for r in LG.read_csv(os.path.join(tour, "ratings.csv"))}


def entry_cis(a, ratings):
    """{player: ci95 or inf} of the entries' players (unrated players count as inf)."""
    out = {}
    for name, _, _, e in a.players():
        if e is None:
            continue
        r = ratings.get(name)
        try:
            out[name] = float(r["ci95"]) if r and r.get("elo") not in (None, "") else math.inf
        except ValueError:
            out[name] = math.inf
    return out


def expected_score(d):
    """Bradley-Terry expected score of a player rated d Elo above its opponent."""
    return 1 / (1 + 10 ** (-d / 400))


def invert(m):
    """Inverse of a square matrix (list of rows), Gauss-Jordan with partial pivoting (numpy isn't installed)."""
    n = len(m)
    a = [list(row) + [1.0 if i == j else 0.0 for j in range(n)] for i, row in enumerate(m)]
    for c in range(n):
        p = max(range(c, n), key=lambda r: abs(a[r][c]))
        if abs(a[p][c]) < 1e-300:
            raise ValueError("singular matrix")
        a[c], a[p] = a[p], a[c]
        piv = a[c][c]
        rc = [v / piv for v in a[c]]
        a[c] = rc
        for r in range(n):
            f = a[r][c]
            if r != c and f:
                a[r] = [x - f * y for x, y in zip(a[r], rc)]
    return [row[n:] for row in a]


def h2h_counts(tour):
    """{(player, opponent): (points, games, wins, losses, draws)} from tour/head_to_head.csv (both orders listed)."""
    out = {}
    for r in LG.read_csv(os.path.join(tour, "head_to_head.csv")):
        w, l, d = int(r["wins"]), int(r["losses"]), int(r["draws"])
        if w + l + d:
            out[(r["player"], r["opponent"])] = (w + d / 2, w + l + d, w, l, d)
    return out


def contrast_errors(elo, h2h, ref):
    """Standard errors (Elo) of each rated player's rating minus the mean of the players `ref`, from the information
    matrix of the fit at the ratings `elo` ({player: Elo}, rated players only) over the games in `h2h` between rated
    players. As the binary's errors (mean_relative_errors: the information plus 1/m everywhere, inverted): for a
    contrast c (sum 0), c' inverse c is c' I+ c, the variance under any constraint on the mean; with ref = every
    rated player it gives the binary's own errors."""
    names = sorted(elo)
    m = len(names)
    idx = {n: i for i, n in enumerate(names)}
    info = [[1.0 / m] * m for _ in range(m)]
    for (x, y), (_, n, *_rest) in h2h.items():
        if x in idx and y in idx:
            p = expected_score(elo[x] - elo[y])
            w = n * p * (1 - p) / (ELO_UNIT * ELO_UNIT)
            info[idx[x]][idx[y]] -= w
            info[idx[x]][idx[x]] += w
    inv = invert(info)
    rs = [idx[r] for r in ref]
    k = len(rs)
    row = [sum(inv[i][j] for j in rs) for i in range(m)]
    tot = sum(row[j] for j in rs)
    return {n: math.sqrt(max(inv[i][i] - 2 * row[i] / k + tot / (k * k), 0.0)) for n, i in idx.items()}


def player_plies(a):
    return {name: ply for name, ply, _, _ in a.players()}


def bt_ml(names, h2h):
    """Pure maximum-likelihood Bradley-Terry Elo of `names` from their games among themselves, as the binary's ml_fit
    (Newton's method from 0; the rated group is the players linked both ways by "scored against" results to the one
    with the most games); relative to the group's mean. Players outside the group are left out. For the fit report
    of arenas whose binary rated nobody (a random anchor that never scores)."""
    n = len(names)
    idx = {x: i for i, x in enumerate(names)}
    pts = [[0.0] * n for _ in range(n)]
    cnt = [[0.0] * n for _ in range(n)]
    for (x, y), (p, g, *_rest) in h2h.items():
        if x in idx and y in idx:
            pts[idx[x]][idx[y]] += p
            cnt[idx[x]][idx[y]] += g
    if not n:
        return {}
    root = max(range(n), key=lambda i: (sum(cnt[i]), -i))

    def reach(fwd):
        seen, stack = {root}, [root]
        while stack:
            i = stack.pop()
            for j in range(n):
                if (pts[i][j] if fwd else pts[j][i]) > 0 and j not in seen:
                    seen.add(j)
                    stack.append(j)
        return seen

    grp = sorted(reach(True) & reach(False))
    m = len(grp)
    var = [g for g in grp if g != root]
    pos = {g: k for k, g in enumerate(var)}
    x = {g: 0.0 for g in grp}

    def logsig(d):  # log of the logistic, without overflow
        return -math.log1p(math.exp(-d)) if d >= 0 else d - math.log1p(math.exp(d))

    def loglik(x):
        return sum(pts[i][j] * logsig(x[i] - x[j]) for i in grp for j in grp if pts[i][j] > 0)

    cur = loglik(x)
    for _ in range(200):
        k = len(var)
        if not k:
            break
        grad = [0.0] * k
        info = [[0.0] * k for _ in range(k)]
        for ai, i in enumerate(grp):
            for j in grp[ai + 1:]:
                games = cnt[i][j]
                if not games:
                    continue
                p = 1 / (1 + math.exp(-max(min(x[i] - x[j], 700.0), -700.0)))
                gr, w = pts[i][j] - games * p, games * p * (1 - p)
                if i in pos:
                    grad[pos[i]] += gr
                    info[pos[i]][pos[i]] += w
                if j in pos:
                    grad[pos[j]] -= gr
                    info[pos[j]][pos[j]] += w
                if i in pos and j in pos:
                    info[pos[i]][pos[j]] -= w
                    info[pos[j]][pos[i]] -= w
        inv = invert(info)
        step = [sum(inv[r][c] * grad[c] for c in range(k)) for r in range(k)]
        scale = 1.0
        while True:
            trial = {g: x[g] + (scale * step[pos[g]] if g in pos else 0.0) for g in grp}
            val = loglik(trial)
            if val >= cur - 1e-12 * abs(cur):
                x, cur = trial, val
                break
            scale /= 2
            if scale < 1e-12:
                break
        if max(abs(scale * s) for s in step) < 1e-10 or scale < 1e-12:
            break
    mean = sum(x.values()) / m
    return {names[g]: (x[g] - mean) * ELO_UNIT for g in grp}


def fit_ratings(a):
    """({player: Elo} the fit report uses, source): the binary's ratings (ratings.csv) when they rate every player
    of the arena's entries; otherwise (a random anchor that never scores leaves them unrated) the same pure
    maximum-likelihood fit made here over the entries' games among themselves."""
    ratings = read_ratings(a.p("tour"))
    refs = set(a.cfg["references"])
    names = [n for n in player_plies(a) if n not in refs]
    elo = {p: float(r["elo"]) for p, r in ratings.items() if r.get("elo") not in (None, "") and p not in refs}
    if all(n in elo for n in names):
        return elo, "the binary's ratings"
    return bt_ml(names, h2h_counts(a.p("tour"))), ("refitted here without the random references (the binary rated "
                                                  f"{len(elo)} of {len(names)} entries' players)")


def pinned_table(a):
    """The pinned anchor's view of the fit (ARENA.md, Pinned anchor): for every player, its Elo minus the mean of the
    reference players' and the 95% interval of that difference. Returns (rows by rating, note); a player the binary
    leaves unrated (or every player, if a reference player is unrated) gets no Elo and an infinite interval."""
    tour = a.p("tour")
    ratings = read_ratings(tour)
    elo = {p: float(r["elo"]) for p, r in ratings.items() if r.get("elo") not in (None, "")}
    plies = player_plies(a)
    mo = a.cfg["anchor"]["mean_of"]
    ref = [n for n in plies if n not in a.cfg["references"]] if mo == "field" else list(mo)
    missing = [r for r in ref if r not in elo]
    note = ""
    errs, zero = {}, None
    h2h = h2h_counts(tour) if os.path.exists(os.path.join(tour, "head_to_head.csv")) else {}
    if missing:
        note = f"anchor undefined: reference players unrated: {', '.join(missing[:5])}" + (" ..." if len(missing) > 5
                                                                                             else "")
    elif elo:
        zero = sum(elo[r] for r in ref) / len(ref)
        errs = contrast_errors(elo, h2h, ref)
    played = {}  # games and points from head_to_head.csv, for the players ratings.csv leaves out (unrated)
    for (x, _), (pts, n, *_rest) in h2h.items():
        g, s = played.get(x, (0, 0.0))
        played[x] = (g + n, s + pts)
    entry_of = {name: e for name, _, _, e in a.players()}
    rows = []
    for name, ply in plies.items():
        r = ratings.get(name, {})
        rated = zero is not None and name in elo
        g, s = played.get(name, (0, 0.0))
        games = r.get("games") or (str(g) if g else "")
        score = r.get("score") or (f"{100 * s / g:.2f}" if g else "")
        rows.append(dict(player=name, ply=ply, elo=round(elo[name] - zero, 1) if rated else "",
                         ci95=round(1.96 * errs[name], 1) if rated else "", se=round(errs[name], 2) if rated else "",
                         games=games, score=score, reference="yes" if name in ref else "",
                         binary_elo=r.get("elo", ""), binary_ci95=r.get("ci95", ""),
                         label=(entry_of.get(name) or {}).get("label", "")))
    rows.sort(key=lambda x: -x["elo"] if x["elo"] != "" else math.inf)
    for i, x in enumerate(rows):
        x["rank"] = i + 1 if x["elo"] != "" else ""
    return rows, note


def pinned_cis(a):
    """{player: ci95 or inf} on the pinned anchor, for the stop rule."""
    rows, _ = pinned_table(a)
    return {r["player"]: (float(r["ci95"]) if r["ci95"] != "" else math.inf) for r in rows}


def stop_cis(a, tour):
    return pinned_cis(a) if a.cfg.get("anchor") else entry_cis(a, read_ratings(tour))


def residuals(a):
    """The Bradley-Terry fit's residuals per pair of rated players (random references left out), lower ply first:
    observed and predicted score, z (the binomial standardised residual; draws make it a little conservative), and the
    pair's Elo deviation from the fit (linearised: ELO (s - p) / (p (1 - p))) with its standard error.
    Returns (pairs, number of rated players, the ratings' source)."""
    tour = a.p("tour")
    elo, source = fit_ratings(a)
    plies = player_plies(a)
    out = []
    for (x, y), (pts, n, w, l, d) in h2h_counts(tour).items():
        if x not in elo or y not in elo:
            continue
        if (plies[x], x) > (plies[y], y):  # each pair once, the lower ply (then name) first
            continue
        p = expected_score(elo[x] - elo[y])
        pq = max(p * (1 - p), 1e-12)  # a predicted certainty (thousands of Elo apart) would divide by zero
        s = pts / n
        # the pair's Elo deviation from the fit, linearised (delta method): unbiased under the model, dev / se = z
        out.append(dict(a=x, b=y, ply_a=plies[x], ply_b=plies[y], games=n, wins=w, losses=l, draws=d,
                        observed=round(s, 4), predicted=round(p, 4), z=round((s - p) / math.sqrt(pq / n), 3),
                        dev_elo=round(ELO_UNIT * (s - p) / pq, 1),
                        se_elo=round(ELO_UNIT / math.sqrt(n * pq), 1)))
    return out, len(elo), source


def residual_summary(pairs, rated):
    """chi2/dof of the z (dof: pairs minus the fit's rated - 1 parameters; about 1 if the fit holds, less with many
    draws); the pairs' Elo deviations weighted by 1/se^2: their mean (of the first player of each pair; it means
    something only for a group of cross-ply pairs, where the first is the lower ply) and the excess SD beyond their
    noise, DerSimonian-Laird (an estimate of the non-transitive spread; lopsided pairs with few games, whose se is
    huge, weigh next to nothing). The weighted mean is sum n (s - p) / sum n p (1 - p): the maximum-likelihood fit
    makes sum n (s - p) zero over each player's games, so over all of one ply's cross-ply pairs too; with two plies
    the cross-ply mean is 0 by construction, with more it shows how the plies' offsets fail to add up."""
    k = len(pairs)
    if not k:
        return dict(pairs=0, games=0)
    dof = max(k - (rated - 1), 1)
    w = [1 / max(p["se_elo"], 1e-9) ** 2 for p in pairs]
    sw = sum(w)
    mean = sum(wi * p["dev_elo"] for wi, p in zip(w, pairs)) / sw
    q = sum(wi * (p["dev_elo"] - mean) ** 2 for wi, p in zip(w, pairs))
    c = sw - sum(wi * wi for wi in w) / sw
    tau2 = max((q - (k - 1)) / c, 0.0) if c > 0 else 0.0
    return dict(pairs=k, games=sum(p["games"] for p in pairs),
                chi2_dof=round(sum(p["z"] ** 2 for p in pairs) / dof, 2),
                rms_z=round(math.sqrt(sum(p["z"] ** 2 for p in pairs) / k), 2),
                excess_sd=round(math.sqrt(tau2), 1), mean_dev=round(mean, 1) + 0.0)  # + 0.0: no "-0.0"


def fit_report(a, top=15):
    """The fit report as text (fit.md): the pinned anchor (if any), the residual summary, by ply pair when the arena
    has several plies, and the `top` largest deviations by |z|. Returns (text, pairs, pinned rows or None)."""
    pairs, rated, source = residuals(a)
    s = residual_summary(pairs, rated)
    st = a.state
    lines = [f"# Fit report: {a.cfg['name']}, {st.get('rounds', 0)} round-robins", ""]
    pinned = None
    if a.cfg.get("anchor"):
        pinned, note = pinned_table(a)
        mo = a.cfg["anchor"]["mean_of"]
        lines.append("Anchor: the mean of " + ("every player" if mo == "field" else f"{len(mo)} reference players")
                     + " at Elo 0 (pinned.csv; no random player)." + (f" {note.upper()}." if note else ""))
        lines.append("")
    if not s["pairs"]:
        lines.append("No rated pairs yet.")
        return "\n".join(lines) + "\n", pairs, pinned
    lines += [f"Bradley-Terry residuals over {s['pairs']} pairs of {rated} rated players ({s['games']} games; random "
              f"references left out; ratings: {source}): chi2/dof {s['chi2_dof']} (about 1 if the fit holds within "
              f"noise, less with many draws), rms z {s['rms_z']}; Elo deviation per pair: excess SD {s['excess_sd']} "
              f"beyond noise (the non-transitive spread).", ""]
    groups = {}
    for p in pairs:
        groups.setdefault((p["ply_a"], p["ply_b"]), []).append(p)
    if len(groups) > 1:
        lines += ["## By ply pair", "",
                  "Mean dev, cross-ply pairs only: the lower-ply player's Elo deviation, weighted by 1/se^2 (> 0: the "
                  "lower ply scores more against the higher than the ratings predict). With two plies it is 0 by "
                  "construction (the fit's score equations, all players rated, no random); with more, each ply's "
                  "cross-ply groups balance (weighted by their total 1/se^2) and their spread shows the plies' "
                  "offsets failing to add up.", "",
                  "| plies | pairs | games | rms z | excess SD | mean dev |", "|---|---:|---:|---:|---:|---:|"]
        for (pa, pb), g in sorted(groups.items()):
            gs = residual_summary(g, rated)
            lines.append(f"| {pa}-{pb} | {gs['pairs']} | {gs['games']} | {gs['rms_z']} | {gs['excess_sd']} | "
                         + (f"{gs['mean_dev']} |" if pa != pb else "- |"))
        lines.append("")
    big = sorted(pairs, key=lambda p: -abs(p["z"]))[:top]
    lines += [f"## The {len(big)} largest deviations by |z|", "",
              "| a | b | games | observed | predicted | z | dev Elo | se |", "|---|---|---:|---:|---:|---:|---:|---:|"]
    for p in big:
        lines.append(f"| {p['a']} | {p['b']} | {p['games']} | {p['observed']:.3f} | {p['predicted']:.3f} | "
                     f"{p['z']:.2f} | {p['dev_elo']:.0f} | {p['se_elo']:.0f} |")
    return "\n".join(lines) + "\n", pairs, pinned


def write_fit_files(a):
    """fit.md and residuals.csv, and pinned.csv with a pinned anchor, in the arena's directory."""
    text, pairs, pinned = fit_report(a)
    LG.atomic_write(a.p("fit.md"), text)
    LG.write_csv(a.p("residuals.csv"), RESIDUAL_FIELDS, pairs)
    if pinned is not None:
        LG.write_csv(a.p("pinned.csv"), PINNED_FIELDS, pinned)


def check_calibrations(a):
    """Compares the calibration the arena's tournament fitted on each entry's own games with the one the entry's
    experiment fitted (both as their tournament logs print them): calibration.csv."""
    got = {}
    log = a.p("tour", "tournament.log")
    if os.path.exists(log):
        with open(log, errors="replace") as f:
            for s in f:
                m = CAL_RE.match(s.rstrip("\n"))
                if m:
                    got.setdefault(m["path"], f"{m['params']} ({m['n']} positions)")
    rows = []
    for e in a.cfg["entries"]:
        arena = got.get(a.owncal(e))
        exp = e.get("expected_calibration")
        rows.append(dict(entry=e["name"], source=e["source"], expected=exp or "", arena=arena or "",
                         identical="" if not exp or not arena else ("yes" if exp == arena else "NO")))
    LG.write_csv(a.p("calibration.csv"), CALCHECK_FIELDS, rows)
    return rows


def games_in_csv(tour):
    p = os.path.join(tour, "games.csv")
    if not os.path.exists(p):
        return 0
    with open(p) as f:
        next(f, None)
        return sum(1 for s in f if s.endswith("\n"))


def tables_current(tour):
    """Whether the tables count every game in games.csv: a run cut short keeps the tables of its last chunk.
    (head_to_head.csv lists every player's games; ratings.csv leaves out the unrated.)"""
    h = LG.read_csv(os.path.join(tour, "head_to_head.csv"))
    return sum(int(x["wins"]) + int(x["losses"]) + int(x["draws"]) for x in h) == 2 * games_in_csv(tour)


def prepare(a):
    """What the tournament needs to load the field: the missing own games, players.txt and retired.txt."""
    cfg, tour = a.cfg, a.p("tour")
    todo = [e for e in cfg["entries"] if not own_done(a, e)]
    if todo:
        a.state["status"] = "calibration"
        a.save()
        a.log(f"own calibration games of {len(todo)} entries")
        per = max(1, cfg["threads"] // max(1, cfg["jobs"]))
        a.cache_hits.clear()
        with ThreadPoolExecutor(max(1, cfg["jobs"])) as ex:
            cpu = sum(ex.map(lambda e: make_own(a, e, per), todo))
        if cfg.get("owncal_cache", True):
            a.log(f"own calibration games: {len(a.cache_hits)} from the shared cache, {len(todo) - len(a.cache_hits)} "
                  f"made ({cpu:.1f} CPU s)")
    head = "# NAME DEPTH SPEC: written by leagues/arena.py\n"
    LG.atomic_write(os.path.join(tour, "players.txt"), head + "".join(f"{n} {d} {s}\n" for n, d, s, _ in a.players()))
    if not os.path.exists(os.path.join(tour, "retired.txt")):
        LG.atomic_write(os.path.join(tour, "retired.txt"), "# An arena's field is fixed: nobody retires.\n")


def play_tournament(a, target, title):
    """The experiments' tournament command, without --save-games (nothing trains on an arena's games), to `target`
    round-robins in all. With target = the round-robins played, the binary only rewrites the tables."""
    cfg, tour = a.cfg, a.p("tour")
    # With a pinned anchor there is no random player: the binary rates relative to the rated players' mean, and
    # pinned_table re-centres on the reference set. Without one, the experiments' command as always.
    anchor = [] if cfg.get("anchor") else ["--anchor", LG.ANCHOR]
    cmd = a.nice() + [cfg["binary"], "tournament", "players.txt", ".", "--tournaments", str(target), "--use-probs",
                      *anchor, "--seed", str(cfg["seed"]), "--no-catch-up", "--retired", "retired.txt",
                      "--prior-draws", "0", *cfg["flags"], "--game-cpu"]
    a.log(title)
    code, cpu, wall, _ = LG.run_measured(cmd, os.path.join(tour, "tournament.log"), title,
                                         env=LG.threads_env(cfg["threads"]), cwd=tour)
    new = rounds_done(tour)
    if code != 0 or new != target:
        a.state["status"] = "failed"
        a.save()
        sys.exit(f"tournament exit {code}, {new} round-robins (wanted {target}); see {tour}/tournament.log")
    return cpu, wall


def drop_own_games(a):
    """Removes the own games once a run ends, keeping their records (<name>.json, <name>.arena): they are 10-15 MB
    an entry, and the next run that has games to play takes them from the shared cache (owncal_cache) or makes them
    again, the same."""
    root, freed = a.p("owncal"), 0
    for e in a.cfg["entries"]:
        d = a.owncal(e)
        if os.path.isdir(d):
            freed += sum(os.path.getsize(os.path.join(d, f)) for f in os.listdir(d))
            shutil.rmtree(d)
    if os.path.isdir(root):
        for f in os.listdir(root):
            if f.startswith(".tmp-"):
                shutil.rmtree(os.path.join(root, f), ignore_errors=True)
    if freed:
        a.log(f"own games removed ({freed / 1e6:.0f} MB); the next run "
              + ("takes them from the shared cache or makes them again" if a.cfg.get("owncal_cache", True)
                 else "makes them again"))


def cmd_run(path, more_rounds=None, threads=None):
    a = Arena(path)
    cfg, tour = a.cfg, a.p("tour")
    if threads:  # the games are the same at any thread count
        cfg["threads"] = threads
    stop = cfg["stop"]
    done = rounds_done(tour)
    clean_partial(tour, done)
    ready = False
    if done and not tables_current(tour):
        prepare(a)
        ready = True
        play_tournament(a, done, f"tables of round-robins 0-{done - 1} (a run cut short left older ones)")
        if cfg.get("anchor"):
            write_fit_files(a)
    goal = done + more_rounds if more_rounds else None
    while True:
        cis = stop_cis(a, tour) if done else {}
        worst = max(cis.values()) if cis else math.inf
        if goal is None:
            if done >= stop["min_rounds"] and worst <= stop["ci95"]:
                a.state["status"] = "done"
                a.state["stopped"] = f"every entry's ci95 <= {stop['ci95']} after {done} round-robins"
                break
            if done >= stop["max_rounds"]:
                a.state["status"] = "done"
                a.state["stopped"] = f"max_rounds ({stop['max_rounds']}); widest ci95 {worst:.1f}"
                break
            target = min(done + stop["chunk_rounds"], stop["max_rounds"])
        else:
            if done >= goal:
                a.state["status"] = "paused"
                a.state["stopped"] = f"{more_rounds} more round-robins played (to {done}); widest ci95 {worst:.1f}"
                break
            target = min(done + stop["chunk_rounds"], goal)
        if not ready:
            prepare(a)
            ready = True
        a.state["status"] = "running"
        a.save()
        cpu, wall = play_tournament(a, target, f"round-robins {done}-{target - 1} ({len(a.players())} players)")
        cis = stop_cis(a, tour)
        worst = max(cis.values()) if cis else math.inf
        a.state.setdefault("chunks", []).append(dict(rounds=target, cpu_s=round(cpu, 1), wall_s=round(wall, 1),
                                                     widest_ci95=None if math.isinf(worst) else round(worst, 1),
                                                     at=LG.now()))
        a.state["rounds"] = done = target
        a.save()
        if cfg.get("anchor"):  # the pinned ratings and the fit report after each chunk
            write_fit_files(a)
        if not a.state.get("calibration_checked"):
            rows = check_calibrations(a)
            a.state["calibration_checked"] = LG.now()
            bad = [r["entry"] for r in rows if r["identical"] == "NO"]
            same = sum(r["identical"] == "yes" for r in rows)
            a.log(f"calibrations: {same} identical to their experiment's"
                  + (f"; DIFFERENT: {', '.join(bad)}" if bad else "")
                  + f"; {sum(r['identical'] == '' for r in rows)} without a record to compare")
    a.state["rounds"] = done
    a.save()
    if not cfg.get("keep_own_games"):
        drop_own_games(a)
    a.log(f"{a.state['status']}: {a.state['stopped']}")
    cmd_status(path)


def elo_of(score):
    if score <= 0:
        return -math.inf
    if score >= 1:
        return math.inf
    return -400 * math.log10(1 / score - 1)


def score_interval(w, l, d, z=1.96):
    """Score and its 95% Wilson interval, a draw counting as half a win (so with draws the interval is a little wide:
    a draw varies less than half a win does). A perfect or a zero score still gets a finite bound on the other side.
    Returns (score, low, high)."""
    n = w + l + d
    s, k = (w + d / 2) / n, z * z / n
    c, half = (s + k / 2) / (1 + k), z * math.sqrt(s * (1 - s) / n + k / (4 * n)) / (1 + k)
    lo, hi = c - half, c + half
    return s, (0.0 if lo < 1e-12 else lo), (1.0 if hi > 1 - 1e-12 else hi)


def h2h_rows(a):
    return LG.read_csv(a.p("tour", "head_to_head.csv"))


def print_pinned(a):
    """status's table for an arena with a pinned anchor: Elo relative to the reference players' mean."""
    rows, note = pinned_table(a)
    mo = a.cfg["anchor"]["mean_of"]
    print("  anchor: the mean of " + ("every player" if mo == "field" else f"{len(mo)} reference players (*)")
          + " at Elo 0; ci95 relative to it" + (f"; {note.upper()}" if note else ""))
    print(f"  {'player':44s} {'ply':>3s} {'elo':>7s} {'ci95':>6s} {'games':>6s} {'score':>6s}  label")
    for r in rows:
        sc = f"{float(r['score']):.1f}%" if r["score"] else ""
        star = "*" if r["reference"] and mo != "field" else " "
        print(f" {star}{r['player']:44s} {r['ply']:>3d} {str(r['elo']):>7s} {str(r['ci95']):>6s} "
              f"{r['games']:>6s} {sc:>6s}  {r['label']}")
    pairs, rated, _ = residuals(a)
    s = residual_summary(pairs, rated)
    if s["pairs"]:
        print(f"  fit: chi2/dof {s['chi2_dof']}, excess SD {s['excess_sd']} Elo over {s['pairs']} pairs "
              f"(fit.md: by ply pair and the largest deviations)")


def cmd_status(path, vs=None):
    a = Arena(path)
    ratings = read_ratings(a.p("tour"))
    st = a.state
    print(f"arena {a.cfg['name']}: {st.get('status')}, {st.get('rounds', 0)} round-robins"
          + (f" ({st['stopped']})" if st.get("stopped") else ""))
    if a.cfg.get("description"):
        print(f"  {a.cfg['description']}")
    if a.cfg.get("anchor"):
        print_pinned(a)
        if vs:
            print_vs(a, vs)
        return
    rows = []
    for name, ply, _, e in a.players():
        r = ratings.get(name, {})
        rows.append((float(r["elo"]) if r.get("elo") else -math.inf, name, r, e))
    rows.sort(key=lambda x: -x[0])
    print(f"  {'player':40s} {'elo':>7s} {'ci95':>6s} {'games':>6s} {'score':>6s} {'positions':>12s}  label")
    for elo, name, r, e in rows:
        pos = f"{e['positions']:,}" if e and isinstance(e.get("positions"), int) else ""
        label = (e or {}).get("label", "reference" if e is None else "")
        sc = f"{float(r['score']):.1f}%" if r.get("score") else ""  # ratings.csv scores are percentages
        print(f"  {name:40s} {r.get('elo', ''):>7s} {r.get('ci95', ''):>6s} {r.get('games', ''):>6s} {sc:>6s} "
              f"{pos:>12s}  {label}")
    cal = LG.read_csv(a.p("calibration.csv"))
    if cal:
        same = sum(r["identical"] == "yes" for r in cal)
        bad = [r["entry"] for r in cal if r["identical"] == "NO"]
        print(f"  calibration: {same} of {sum(r['identical'] != '' for r in cal)} experiment versions identical to "
              f"their experiment's" + (f"; DIFFERENT: {', '.join(bad)}" if bad else ""))
    if vs:
        print_vs(a, vs)


def print_vs(a, vs):
    print(f"\n  {vs} against each player (direct games; 95% interval):")
    for r in h2h_rows(a):
        if r["player"] != vs:
            continue
        w, l, d = int(r["wins"]), int(r["losses"]), int(r["draws"])
        n = w + l + d
        if not n:
            continue
        s, lo, hi = score_interval(w, l, d)
        print(f"    vs {r['opponent']:38s} +{w} -{l} ={d} ({n}) score {100 * s:5.1f}% -> "
              f"{elo_of(s):6.0f} [{elo_of(lo):.0f}, {elo_of(hi):.0f}]")


def cmd_fit(path, write=False, top=15):
    """The fit report of any arena (fit.md's text): with --write also fit.md, residuals.csv (and pinned.csv with a
    pinned anchor) in its directory. Arenas without a pinned anchor get the residuals only (their random references
    left out); their files are not touched unless --write."""
    a = Arena(path)
    text, pairs, pinned = fit_report(a, top)
    print(text, end="")
    if write:
        write_fit_files(a)
        print(f"wrote {a.p('fit.md')}, {a.p('residuals.csv')}" + (f", {a.p('pinned.csv')}" if pinned is not None
                                                                   else ""))


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    s = sub.add_parser("create")
    s.add_argument("spec")
    s = sub.add_parser("run")
    s.add_argument("arena")
    s.add_argument("--rounds", type=int, help="play this many more round-robins, whatever the stop rule says")
    s.add_argument("--threads", type=int, help="threads for this run (default: the config's)")
    s = sub.add_parser("status")
    s.add_argument("arena")
    s.add_argument("--vs", help="a player's direct results against every other player")
    s = sub.add_parser("fit")
    s.add_argument("arena")
    s.add_argument("--write", action="store_true", help="also write fit.md, residuals.csv (and pinned.csv)")
    s.add_argument("--top", type=int, default=15, help="the largest deviations listed (default 15)")
    args = ap.parse_args()
    if args.cmd == "create":
        cmd_create(args.spec)
    elif args.cmd == "run":
        cmd_run(args.arena, args.rounds, args.threads)
    elif args.cmd == "fit":
        cmd_fit(args.arena, args.write, args.top)
    else:
        cmd_status(args.arena, args.vs)


if __name__ == "__main__":
    main()
