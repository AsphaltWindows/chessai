#!/usr/bin/env python3
"""A read-only live dashboard for the training/tournament cycles (see README.md).

    setsid nohup nice -n 19 python3 dashboard.py >> dashboard.log 2>&1 < /dev/null &
    python3 dashboard.py --once > page.html        # render once to stdout, for testing

It serves http://127.0.0.1:8765 (localhost only) and renders every page from the cycle files and /proc, so it shows
the loop live. The page reloads its content every 30 s. It never writes into cycles/ or the league, takes no locks,
and parses each file only when it changes (games.csv incrementally, from where it left off), at nice 19. A file
caught half-written keeps its last good reading. Python 3 standard library only.
"""
import argparse
import csv
import datetime
import html
import io
import json
import math
import os
import re
import sys
import threading
import time
import traceback
import urllib.parse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

HERE = os.path.dirname(os.path.abspath(__file__))
CLK = os.sysconf("SC_CLK_TCK")
REFRESH_S = 30
DU_EVERY_S = 600
CORES_PER_JOB = 2.8  # measured CPU/wall of a 4-thread self-play job; only for the ETA before any job has run

FAMILIES = [  # the marker shapes (shape_of picks one from the facets): key, label, colour variable, shape; a
    # lineage's colour is its value's for the factor the page colours by (--lc on its row or group; grey without)
    ("fl", "flat", "--lc", "dot"),
    ("other", "other", "--lc", "dot"),
    ("plain", "tree", "--lc", "dot"),
    ("merge", "merging tree (merge or fresh)", "--lc", "ring"),
    ("dag", "DAG", "--lc", "triangle"),
    ("ht", "HT (router H)", "--lc", "diamond"),
    ("random", "Random players (references)", "--c-ref", "square"),
]
FAMILY = {k: (label, colour, shape) for k, label, colour, shape in FAMILIES}
PHASES = ["self-play", "tournament", "decide", "own", "log"]
PHASE_LABEL = {"self-play": "self-play", "tournament": "tournament", "decide": "qualification",
               "own": "own-games steps and gates", "log": "writing the logs"}

LOCK = threading.Lock()
_cache, _errors = {}, {}
_du = {"at": 0.0, "total": None, "per": {}, "running": False}


# ---------------------------------------------------------------- reading files

def load(path, parse, binary=False):
    """parse(contents) of the file at `path`, re-read only when its size, mtime or inode changes. On any error,
    the last good value (or None); the error is noted for the page. A missing file gives None."""
    try:
        st = os.stat(path)
    except OSError:
        _cache.pop(path, None)
        _errors.pop(path, None)
        return None
    sig = (st.st_mtime_ns, st.st_size, st.st_ino)
    hit = _cache.get(path)
    if hit and hit[0] == sig:
        return hit[1]
    try:
        if binary:
            with open(path, "rb") as f:
                data = f.read()
        else:
            with open(path, encoding="utf-8", errors="replace") as f:
                data = f.read()
        value = parse(data)
    except Exception as e:  # half-written or malformed: keep the last good reading
        _errors[path] = f"{type(e).__name__}: {e}"
        return hit[1] if hit else None
    _errors.pop(path, None)
    _cache[path] = (sig, value)
    return value


def complete_lines(text):
    """The text without a trailing partial line (a file caught while being appended to)."""
    return text if text.endswith("\n") or not text else text[:text.rfind("\n") + 1]


def parse_csv(required):
    def parse(text):
        if not text.strip():
            raise ValueError("empty")
        rows = list(csv.DictReader(io.StringIO(complete_lines(text))))
        if rows and not set(required) <= set(rows[0]):
            raise ValueError(f"missing columns {sorted(set(required) - set(rows[0]))}")
        return [r for r in rows if all(r.get(k) is not None for k in required)]
    return parse


def parse_ratings(text):
    if not text.endswith("\n"):
        raise ValueError("being written")
    rows = parse_csv(["player", "elo", "ci95", "games"])(text)
    out = {}
    for r in rows:
        out[r["player"]] = dict(elo=float(r["elo"]), ci=float(r["ci95"]), games=int(float(r["games"])),
                                score=r.get("score"), retired=r.get("retired") == "yes")
    return out


def parse_entries(text):
    out = []
    for line in text.splitlines():
        s = line.split(None, 2)
        if len(s) >= 2 and not s[0].startswith("#"):
            out.append((s[0], int(s[1]) if s[1].isdigit() else None, s[2] if len(s) > 2 else ""))
    return out


RETIRED_LINE = re.compile(r"#\s*retired\b(.*?):\s+(\S+ \| .+)$")  # the comment may hold a time


def parse_registry(text):
    """lineages.txt's lineages in order, then its retired ones: lines commented out as "# retired after cycle N (...):
    <the line>", with former=True and retired_after=N (None when the comment names no cycle)."""
    out, former = [], {}
    for line in text.splitlines():
        line = line.strip()
        if line.startswith("#"):
            m = RETIRED_LINE.match(line)
            try:
                l = registry_entry(m.group(2)) if m else None
            except ValueError:  # a comment that only looks like one
                l = None
            if l:
                c = re.search(r"\bcycle (\d+)", m.group(1))
                former[l["name"]] = dict(l, former=True, retired_after=int(c.group(1)) if c else None)
        elif line:
            l = registry_entry(line)
            if l:
                out.append(l)
    names = {l["name"] for l in out}
    return out + [l for n, l in former.items() if n not in names]


def registry_entry(line):
    """One lineages.txt line as a dict, or None if it isn't one."""
    parts = [p.strip() for p in line.split(" | ")]
    if len(parts) != 7:
        return None
    if True:
        name, arch, binary, source, heldout, steps, flags = parts
        toks = flags.split()
        kind = toks[toks.index("--model") + 1] if "--model" in toks[:-1] else "?"
        depth = int(toks[toks.index("--depth") + 1]) if "--depth" in toks[:-1] else 0
        if kind == "ht":
            fam = "ht"
        elif arch.startswith("D-") or "--dag-leaves" in toks:
            fam = "dag"
        elif kind in ("bdt", "bdd"):
            fam = "merge" if "--revise" in toks else "plain"
        elif arch.startswith("F-L"):
            fam = "fl"
        else:
            fam = "other"
        return dict(name=name, arch=arch, kind=kind, depth=depth, family=fam, own="own" in steps.split(","),
                    binary=binary, flags=toks)


def guess_lineage(name, known):
    """A lineage state.json calls retired whose line lineages.txt no longer has, even commented out: its archetype is
    the longest known one its name starts with; a tree's family is merge if its name says so (-merge or -fresh: a
    --revise tree), otherwise the
    archetype's family; its depth is the one in its name (d2 in F-L-d2jit-c), else 0."""
    arch = max((l["arch"] for l in known if name.startswith(l["arch"] + "-")), key=len, default=None)
    if arch is None:
        return None
    sib = [l for l in known if l["arch"] == arch]
    tree = any(l["family"] in ("plain", "merge") for l in sib)
    m = re.search(r"-d(\d+)", name[len(arch):])
    return dict(name=name, arch=arch, kind="bdt" if tree else sib[0]["kind"], depth=int(m.group(1)) if m else 0,
                family=("merge" if re.search(r"-(merge|fresh)\b", name[len(arch):]) else "plain") if tree
                else sib[0]["family"],
                own=False, former=True, retired_after=None)


def former_lineages(state, reg):
    """The retired lineages, for their history: lineages.txt's "# retired ..." lines, then any other lineage state.json
    calls retired (guessed from its name). retired_after: state.json's retired_cycle, else the comment's cycle."""
    sts = state.get("lineages") or {}
    out = [l for l in reg if l.get("former")]
    have = {l["name"] for l in reg}
    for name, st in sts.items():
        if isinstance(st, dict) and st.get("status") == "retired" and name not in have:
            g = guess_lineage(name, reg)
            if g:
                out.append(g)
    res = []
    for l in out:  # copies: reg is the cached parse
        st = sts.get(l["name"])
        c = num(st.get("retired_cycle")) if isinstance(st, dict) else None
        res.append(dict(l, retired_after=int(c)) if c is not None else l)
    return res


def parse_models_csv(text):
    """name -> the cycle its version entered the field, from the league's models.csv ("v424 (cycle 1: ...)")."""
    out = {}
    for r in csv.DictReader(io.StringIO(complete_lines(text))):
        m = re.search(r"\(cycle (\d+)", r.get("rounds") or "")
        if m and r.get("name"):
            out[r["name"]] = int(m.group(1))
    return out


def parse_loop_log(text):
    starts, events = {}, []
    for line in text.splitlines():
        m = re.match(r"cycle (\d+) \((\d{4}-\d\d-\d\d \d\d:\d\d)\)", line)
        if m:
            starts.setdefault(int(m.group(1)), datetime.datetime.strptime(m.group(2), "%Y-%m-%d %H:%M").timestamp())
        if re.match(r"(loop started|STOP found|cycle failed|cycle \d+: resuming|cycle \d+ done)", line):
            events.append(line.strip())
    return dict(starts=starts, events=events[-6:])


def parse_cycle_md(text):
    m = re.search(r"Tournament: .*? in ([\d.]+) min", text)
    g = re.search(r"^\*\*Own-games step, pooled over the lineages: (.*?)\*\*", text, re.M)
    b = re.search(r"budget ([\d.]+) CPU s per qualified lineage", text)
    r = re.search(r"Tournament: (\d+) double round-robins", text)
    return dict(tournament_min=float(m.group(1)) if m else None, pooled=g.group(1) if g else None,
                budget=float(b.group(1)) if b else None, round_robins=int(r.group(1)) if r else None)


def parse_elo_history(text):
    """The league's Elo history (elo_history.csv): {t: {player: Elo}}, the fit after t round-robins (ratings.csv is
    the newest one)."""
    out = {}
    for r in parse_csv(["tournament", "player", "elo"])(text):
        t, e = num(r.get("tournament")), num(r.get("elo"))
        if t is not None and e is not None:
            out.setdefault(int(t), {})[r["player"]] = e
    return out


def unsign(line):
    """A tool's line with its absolute ratings unsigned ("top: X +1836" -> "top: X 1836", " -12" -> " −12"): a sign
    before a number after a space."""
    return re.sub(r"(?<=\s)-(?=\d)", "−", re.sub(r"(?<=\s)\+(?=\d)", "", line))


def tail(path, nbytes=8192):
    try:
        with open(path, "rb") as f:
            f.seek(0, 2)
            size = f.tell()
            f.seek(max(0, size - nbytes))
            return f.read().decode("utf-8", "replace")
    except OSError:
        return ""


class GameCounter:
    """Games per tournament in games.csv, read incrementally: only the bytes appended since the last call."""

    def __init__(self):
        self.key, self.offset, self.counts, self.col = None, 0, {}, None

    def __call__(self, path):
        try:
            st = os.stat(path)
        except OSError:
            self.key, self.offset, self.counts = None, 0, {}
            return {}
        if self.key != (st.st_dev, st.st_ino) or st.st_size < self.offset:
            self.key, self.offset, self.counts, self.col = (st.st_dev, st.st_ino), 0, {}, None
        if st.st_size == self.offset:
            return self.counts
        with open(path, "rb") as f:
            f.seek(self.offset)
            while True:
                chunk = f.read(1 << 22)
                if not chunk:
                    break
                end = chunk.rfind(b"\n")
                if end < 0:
                    break  # a partial line: wait for the rest
                for line in chunk[:end].split(b"\n"):
                    if self.col is None:
                        head = line.decode("utf-8", "replace").split(",")
                        self.col = head.index("tournament") if "tournament" in head else 0
                        continue
                    field = line.split(b",", self.col + 1)
                    if len(field) > self.col and field[self.col].isdigit():
                        t = int(field[self.col])
                        self.counts[t] = self.counts.get(t, 0) + 1
                self.offset += end + 1
                f.seek(self.offset)
        return self.counts


count_games = GameCounter()


# ---------------------------------------------------------------- processes and disk

def boot_time():
    with open("/proc/stat") as f:
        for line in f:
            if line.startswith("btime"):
                return int(line.split()[1])
    return 0


BTIME = boot_time()


def processes():
    """[{pid, args, cpu, start, cwd}] of the loop, the driver and the chessai jobs."""
    out = []
    for pid in os.listdir("/proc"):
        if not pid.isdigit():
            continue
        try:
            with open(f"/proc/{pid}/cmdline", "rb") as f:
                args = [a.decode("utf-8", "replace") for a in f.read().split(b"\0") if a]
            if not args or not any(k in a for a in args[:2] for k in ("loop.sh", "cycle.py", "chessai")):
                continue
            with open(f"/proc/{pid}/stat") as f:
                fields = f.read().rsplit(")", 1)[1].split()
            try:
                cwd = os.readlink(f"/proc/{pid}/cwd")
            except OSError:
                cwd = ""
            out.append(dict(pid=int(pid), args=args, cwd=cwd, state=fields[0],
                            cpu=(int(fields[11]) + int(fields[12])) / CLK, start=BTIME + int(fields[19]) / CLK))
        except (OSError, IndexError, ValueError):
            continue
    return out


def last_version(run_dir, kind, lo=0):
    """The newest version N with {kind}1..N.model all present (versions are written in order): a galloping search
    from `lo`, a few stats instead of hundreds."""
    def ok(n):
        return n <= 0 or os.path.exists(os.path.join(run_dir, f"{kind}{n}.model"))
    if not ok(lo):
        lo = 0
    step = 1
    while ok(lo + step):
        lo += step
        step *= 2
    hi = lo + step  # ok(lo), not ok(hi)
    while hi - lo > 1:
        mid = (lo + hi) // 2
        lo, hi = (mid, hi) if ok(mid) else (lo, mid)
    return lo


def du_worker(runs_dir):
    """Disk used by each lineage's run directory, refreshed every DU_EVERY_S seconds in the background."""
    while True:
        per = {}
        try:
            for e in os.scandir(runs_dir):
                if e.is_dir(follow_symlinks=False):
                    total, stack = 0, [e.path]
                    while stack:
                        d = stack.pop()
                        try:
                            for x in os.scandir(d):
                                if x.is_dir(follow_symlinks=False):
                                    stack.append(x.path)
                                elif x.is_file(follow_symlinks=False):
                                    total += x.stat(follow_symlinks=False).st_blocks * 512
                        except OSError:
                            pass
                    per[e.name] = total
            _du.update(per=per, total=sum(per.values()), at=time.time())
        except OSError:
            pass
        time.sleep(DU_EVERY_S)


# ---------------------------------------------------------------- formatting

esc = html.escape


def fmt_dur(s):
    if s is None or s != s:
        return "—"
    s = int(round(s))
    if s < 90:
        return f"{s} s"
    m = s // 60
    if m < 90:
        return f"{m} min"
    return f"{m // 60} h {m % 60:02d} min"


def fmt_cpu(s):
    return f"{s:,.0f} CPU-s" if s < 3600 else f"{s / 3600:.1f} CPU-hours"


def fmt_clock(t):
    return datetime.datetime.fromtimestamp(t).strftime("%H:%M") if t else "—"


def fmt_bytes(n):
    if n is None:
        return "—"
    for unit in ("B", "KB", "MB", "GB", "TB"):
        if n < 1000 or unit == "TB":
            return f"{n:.0f} {unit}" if unit in ("B", "KB") or n >= 100 else f"{n:.1f} {unit}"
        n /= 1000


def signed(x, digits=0):
    """A recent change (the Standings change column, a cycle's gate result): with its sign."""
    return "—" if x is None else f"{x:+.{digits}f}".replace("-", "−")


def plain(x, digits=0):
    """An absolute number (a rating, an axis tick, a gain): no "+", a real minus sign."""
    if x is None:
        return "—"
    x = 0.0 if round(x, digits) == 0 else x
    return f"{x:.{digits}f}".replace("-", "−")


def num(x, default=None):
    try:
        return float(x)
    except (TypeError, ValueError):
        return default


def nice_ticks(lo, hi, n=5):
    if hi <= lo:
        hi = lo + 1
    raw = (hi - lo) / max(1, n)
    mag = 10 ** math.floor(math.log10(raw))
    step = min((m * mag for m in (1, 2, 2.5, 5, 10) if m * mag >= raw), default=10 * mag)
    first = math.ceil(lo / step) * step
    return [first + i * step for i in range(int((hi - first) / step) + 1)]


def fam_colour(fam):
    """A marker's colour: the lineage's (--lc), grey where none is set; a random player's own."""
    colour = FAMILY.get(fam, FAMILY["random"])[1]
    return "var(--lc, var(--muted))" if colour == "--lc" else f"var({colour})"


def marker(fam, x, y, r=4.0, extra=""):
    shape = FAMILY.get(fam, FAMILY["random"])[2]
    c = fam_colour(fam)
    if shape == "ring":
        return (f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{r + 1:.1f}" fill="var(--surface)"/>'
                f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{r - 0.5:.1f}" fill="var(--surface)" stroke="{c}" '
                f'stroke-width="2"{extra}/>')
    if shape == "diamond":
        d = r + 1.2
        pts = f"{x:.1f},{y - d:.1f} {x + d:.1f},{y:.1f} {x:.1f},{y + d:.1f} {x - d:.1f},{y:.1f}"
        return f'<polygon points="{pts}" fill="{c}" stroke="var(--surface)" stroke-width="2"{extra}/>'
    if shape == "triangle":
        d = r + 1.6
        pts = f"{x:.1f},{y - d:.1f} {x + d * 0.92:.1f},{y + d * 0.6:.1f} {x - d * 0.92:.1f},{y + d * 0.6:.1f}"
        return f'<polygon points="{pts}" fill="{c}" stroke="var(--surface)" stroke-width="2"{extra}/>'
    if shape == "square":
        return (f'<rect x="{x - r + 0.5:.1f}" y="{y - r + 0.5:.1f}" width="{2 * r - 1:.1f}" height="{2 * r - 1:.1f}" '
                f'rx="1" fill="{c}" stroke="var(--surface)" stroke-width="2"{extra}/>')
    return f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{r:.1f}" fill="{c}" stroke="var(--surface)" stroke-width="2"{extra}/>'


def swatch(fam, attrs=""):
    return f'<svg class="sw" width="14" height="14" aria-hidden="true"{attrs}>{marker(fam, 7, 7)}</svg>'


DASH = "3 2"  # a lineage that self-plays at depth 2 or more, in the small multiples


def line_key(fam, dashed=False):
    """A small-multiples legend sample: the family's marker on a short line, dashed for a lineage that self-plays at
    depth 2 or more (--depth in lineages.txt)."""
    da = f' stroke-dasharray="{DASH}"' if dashed else ""
    return (f'<svg class="sw" width="30" height="14" aria-hidden="true"><line x1="1" x2="29" y1="7" y2="7" '
            f'stroke="{fam_colour(fam)}" stroke-width="2"{da}/>{marker(fam, 15, 7)}</svg>')


# ---------------------------------------------------------------- facets: each lineage's value for each factor

UNCL = "unclassified"
FACTORS = [  # key, label, the values' order (the chips and the colour slots follow it), the comparison's reference
    ("arch", "Architecture", ["F", "T", "D", "M"], "F"),
    ("leaf", "Leaf evaluator", ["L", "N", "P", "TAN", "FM"], "L"),
    ("router", "Router", ["Nc", "Ne", "Lc", "Le", "NNs", "NLs", "LNs", "LLs", "H", "gate (soft)", "none"], "Ne"),
    ("split", "Split management", ["none", "merge", "fresh", "pre-split"], "none"),
    ("depth", "Self-play depth", [], "0"),
    ("labels", "Training labels", ["result"], "result"),
    ("policy", "Self-play move selection", ["jitter10"], "jitter10"),
]
FX = {k: (label, order, ref) for k, label, order, ref in FACTORS}
FX_BY = "leaf"  # what the page colours and groups by until the user picks
FX_NAMES = {"arch": {"F": "flat", "T": "tree", "D": "DAG", "M": "MoE"},  # shown beside the letter
            "leaf": {"L": "LR", "N": "NB", "P": "probit"}}
FX_GREY = {"router": ("H", "gate (soft)", "none"), "split": ("none",)}  # no colour of their own (nor unclassified)
FX_LINKED = {"arch": ["router", "split"]}  # comparing architectures lets these vary with it (a flat model has none)
FX_MODEL = {"lr": "L", "cnbp": "N", "probit": "P", "tan": "TAN", "fm": "FM"}  # a flat --model, a tree's --leaf
FX_LETTER = {"cnbp": "N", "lr": "L"}  # a router's classifier
FX_ALWAYS = ("--use-probs", "--vs-previous", "--no-games")  # every line has these; the variant says if one lacks one
FX_ERROR = {"c": "abs", "e": "log", "s": "log"}  # the field's --error, by routing approach
FX_ARCH = re.compile(r"(?:F-([LNP])|(MoE)|(FM)|(TAN)|([TD])-([LN])-(H|[NL]{1,2}[ces]))")


def fx_flags(toks):
    """lineages.txt's flags as {flag: value}: a flag followed by another flag, or by nothing, is a switch (True)."""
    out, i = {}, 0
    while i < len(toks):
        t = toks[i]
        if t.startswith("--") and i + 1 < len(toks) and not toks[i + 1].startswith("--"):
            out[t] = toks[i + 1]
            i += 2
            continue
        if t.startswith("--"):
            out[t] = True
        else:
            out.setdefault("", []).append(t)  # a stray value: the variant lists it
        i += 1
    return out


def fx_float(x):
    try:
        return float(x)
    except (TypeError, ValueError):
        return None


def fx_router(m):
    """A BDT's or a DAG's router, as in the lineage names: --router's letter (cnbp by default), then the approach, c or
    e (--route cluster or error, error by default); with --route-switch, the error router's letter (--error-router, the
    router's by default) and s."""
    r1 = FX_LETTER.get(m.get("--router", "cnbp"))
    route = m.get("--route", "error")
    if r1 is None:
        return UNCL
    if "--route-switch" not in m:
        return r1 + {"cluster": "c", "error": "e"}[route] if route in ("cluster", "error") else UNCL
    r2 = FX_LETTER.get(m.get("--error-router", m.get("--router", "cnbp")))
    return r1 + r2 + "s" if r2 and route == "cluster" else UNCL


def fx_split(m):
    """A BDT's split management: merge (--revise), fresh (--revise with --revise-on fresh), pre-split (--pre-split),
    else none."""
    rev, pre, on = "--revise" in m, "--pre-split" in m, m.get("--revise-on")
    if not (rev or pre or on):
        return "none"
    if pre and not (rev or on):
        return "pre-split"
    if rev and not pre:
        return "fresh" if on == "fresh" else "merge" if on in (None, "stream") else UNCL
    return UNCL


def facets(l):
    """A lineage's value for each factor of FACTORS, the single source of the facets: read from its lineages.txt line's
    flags, with the binary's defaults where a flag is absent; for a retired lineage without its line, from its name
    (fx_by_name). "unclassified" wherever neither settles it, never a guess."""
    if l.get("flags") is None:
        return fx_by_name(l)
    f = {k: UNCL for k, *_ in FACTORS}
    m = fx_flags(l["flags"])
    kind, dag = m.get("--model"), "--dag-leaves" in m
    if kind in FX_MODEL and not dag:
        f.update(arch="F", leaf=FX_MODEL[kind], router="none", split="none")
    elif kind == "moe" and not dag:
        f.update(arch="M", leaf="L", router="gate (soft)", split="none")  # its experts are logistic regressions
    elif kind == "ht" and not dag:  # a Hoeffding tree: no model routes, its feature tests do
        f.update(arch="T", leaf=FX_MODEL.get(m.get("--leaf", "cnbp"), UNCL), router="H",
                 split="none" if fx_split(m) == "none" else UNCL)
    elif kind == "bdt":
        f.update(arch="D" if dag else "T", leaf=FX_MODEL.get(m.get("--leaf", "cnbp"), UNCL), router=fx_router(m),
                 split=fx_split(m))
    dp = m.get("--depth", "0")
    if isinstance(dp, str) and dp.isdigit():
        f["depth"] = str(int(dp))
    ld, r = m.get("--label-depth"), fx_float(m.get("--label-result", "0"))  # r: the game result's weight
    if ld is None:
        f["labels"] = "result"
    elif isinstance(ld, str) and ld.isdigit() and r is not None and 0 <= r < 1:
        f["labels"] = f"d{ld} search" if r == 0 else f"mix: d{ld} search {100 * (1 - r):g}%"
    pol, t = m.get("--policy", "wld-score-jitter10"), fx_float(m.get("--temperature", "0.01"))
    if pol == "wld-score-jitter10":
        f["policy"] = "jitter10"
    elif pol == "log-odds-softmax" and t is not None:
        f["policy"] = f"softmax {t:g}"
    return f


def fx_by_name(l):
    """A retired lineage known only by its name: its archetype's letters (F-L, MoE, FM, TAN, T-N-NNs, T-L-H, D-L-Nc)
    give its architecture, leaf and router; the rest of its name its split management (-merge, -fresh, -pre, else
    none) and self-play depth (-dN, else 0), and the defaults (result labels, jitter10) when it has no -dN or says
    jit (d2jit). Anything else unclassified (d0soft, d2play, d2mix: the temperature and the mix aren't in the name)."""
    f = {k: UNCL for k, *_ in FACTORS}
    arch, name = l.get("arch") or "", l["name"]
    m, rest = FX_ARCH.fullmatch(arch), name[len(arch):]
    if not m:
        return f
    if m.group(5):
        f.update(arch=m.group(5), leaf=m.group(6), router=m.group(7),
                 split=next((s for s, w in (("merge", "merge"), ("fresh", "fresh"), ("pre-split", "pre"))
                             if re.search(rf"-{w}\b", rest)), "none"))
    else:
        f.update(arch="M" if m.group(2) else "F", leaf="L" if m.group(2) else m.group(1) or m.group(3) or m.group(4),
                 router="gate (soft)" if m.group(2) else "none", split="none")
    dp = re.search(r"-d(\d+)", rest)
    f["depth"] = dp.group(1) if dp else "0"
    if not dp or "jit" in rest:
        f.update(labels="result", policy="jitter10")
    return f


def fx_num(x):
    """A hyperparameter's value, short: 5000000 is 5M, 300000 300k, 1e-05 1e-5."""
    v = fx_float(x)
    if v is None:
        return str(x)
    if math.isinf(v):
        return "inf" if v > 0 else "-inf"
    for div, suf in ((1e6, "M"), (1e3, "k")):
        if abs(v) >= div and float(f"{v / div:g}") * div == v:
            return f"{v / div:g}{suf}"
    return re.sub(r"e([-+])0*(\d)", lambda mm: "e" + ("-" if mm.group(1) == "-" else "") + mm.group(2), f"{v:g}")


def fx_same(a, b):
    x, y = fx_float(a), fx_float(b)
    return b is not None and (x == y if x is not None and y is not None else str(a) == str(b))


def fx_variant(l, f):
    """A lineage's hyperparameters, beside its facets and not facets themselves: for each that applies to it, the value
    in effect (its flag's, else the binary's default), shown in the variant text when it differs from the field's usual
    recipe (split threshold 5M, split limit 5, forget factor 1, branch factor 3, leaf share 0, 2 experts, TAN m 5,
    τ 1e-5, seed 21; a DAG's leaves and window always). Flags this doesn't know come last, always shown.
    [dict(k=label, v=value, t=text, show=bool)]."""
    if l.get("flags") is None:
        return []
    m, out = fx_flags(l["flags"]), []
    used = {"--model", "--leaf", "--router", "--error-router", "--route", "--route-switch", "--revise", "--revise-on",
            "--pre-split", "--dag-leaves", "--depth", "--label-depth", "--label-result", "--policy", "--temperature"}

    def knob(flag, k, default, recipe, fmt=None, always=False):
        used.add(flag)
        v = m.get(flag, default)
        if v is None:
            return
        s = "on" if v is True else fx_num(v)
        out.append(dict(k=k, v=s, t=fmt(s) if fmt else f"{k} {s}", show=always or not fx_same(v, recipe)))

    arch, router = f["arch"], f["router"]
    if arch in ("T", "D") and router != "H":  # a BDT or a DAG
        st = m.get("--split-threshold", "2000")
        knob("--split-threshold", "split threshold", "2000", "5000000")
        knob("--split-limit", "split limit", "20", "5")
        knob("--forget-factor", "forget factor", "10", "1")
        knob("--branch-factor", "branch factor", "2", "3")
        knob("--leaf-share", "leaf share", "0", "0")
        knob("--error", "error measure", "abs", FX_ERROR.get(router[-1:]) if router != UNCL else None)
        if "--route-switch" in m:
            knob("--route-switch", "switch z", None, "3")
        if arch == "T":
            knob("--reset-leaves", "reset leaves", "inf", "inf")  # absent: never; inf: compare and log only
        if "--revise" in m:
            knob("--revise", "revise z", None, "2" if f["split"] == "fresh" else "0")
        if "--pre-split-fresh" in m:
            knob("--pre-split-fresh", "pre-split fresh z", None, None, always=True)
        if arch == "D":
            knob("--dag-leaves", "DAG leaves", None, None, always=True)
            bf, stv = fx_float(m.get("--branch-factor", "2")), fx_float(st)
            knob("--dag-window", "DAG window", str(stv / bf) if bf and stv else None, None, always=True)
            knob("--dag-z", "DAG z", "2", "2")
    if arch == "M":
        knob("--moe-experts", "experts", "2", "2", fmt=lambda s: f"{s} experts")
    if f["leaf"] == "TAN":
        knob("--tan-m", "TAN m", "5", "5")
    if f["leaf"] == "P":
        knob("--probit-tau", "τ", "1e-5", "1e-5")
    knob("--seed", "seed", None, "21")
    for flag in FX_ALWAYS:
        used.add(flag)
        if flag not in m:
            out.append(dict(k=f"no {flag}", v="", t=f"no {flag}", show=True))
    for flag, v in m.items():
        if flag not in used:
            s = "" if v is True else " ".join(v) if isinstance(v, list) else str(v)
            out.append(dict(k=flag or "stray values", v=s, t=f"{flag} {s}".strip(), show=True))
    return out


def fx(d):
    """{lineage: (facets, variant)} for lineages.txt's lineages and the retired ones, worked out once per page."""
    if getattr(d, "_fx", None) is None:
        d._fx = {}
        for l in d.registry + d.former:
            f = facets(l)
            d._fx[l["name"]] = (f, fx_variant(l, f))
    return d._fx


def fx_values(d, k):
    """A factor's values among the lineages: in FACTORS order, others after it (depth by number), unclassified last."""
    order = FX[k][1]
    have = {f[k] for f, _ in fx(d).values()}
    return sorted(have, key=lambda v: (v == UNCL, order.index(v) if v in order else len(order),
                                       int(v) if v.isdigit() else 0, v))


def fx_colours(d, k):
    """{value: colour} for a factor: the categorical slots in the values' order, fixed, never cycled; grey for
    unclassified, for the values FX_GREY names (no router model, no split management) and past the eighth (Other)."""
    cache = d.__dict__.setdefault("_fxc", {})
    if k not in cache:
        vals = [v for v in fx_values(d, k) if v != UNCL and v not in FX_GREY.get(k, ())]
        cache[k] = {v: f"var(--k{vals.index(v) + 1})" if v in vals and vals.index(v) < 8 else "var(--muted)"
                    for v in fx_values(d, k)}
    return cache[k]


def fx_label(k, v):
    """A value as the page shows it: a letter with its name (F (flat), L (LR))."""
    n = FX_NAMES.get(k, {}).get(v)
    return f"{v} ({n})" if n else v


def lin_of(d, name):
    """An entry's lineage, or the name itself for a lineage."""
    return name if name in d.lin else d.split_entry(name)[0]


def lc_attr(d, name):
    """The attributes that colour a lineage's (or an entry's) marks: its value's colour for the default factor; the
    page script recolours every data-lc by the user's pick."""
    lin = lin_of(d, name)
    e = fx(d).get(lin)
    col = fx_colours(d, FX_BY).get(e[0][FX_BY]) if e else None
    return f' data-lc="{esc(lin)}" style="--lc:{col or "var(--muted)"}"'


def shape_of(d, name):
    """A lineage's (or entry's) marker: a square for a random player, a diamond for an HT (router H), a triangle for a
    DAG, a ring for a merging tree (merge or fresh), else a dot."""
    if name in d.refs or name.startswith("random"):
        return "random"
    e = fx(d).get(lin_of(d, name))
    if not e:
        return d.family(name)
    f = e[0]
    return ("ht" if f["router"] == "H" else "dag" if f["arch"] == "D" else
            "merge" if f["split"] in ("merge", "fresh") else "plain")


def sw(d, name):
    """A lineage's (or entry's) swatch: its marker in its value's colour."""
    s = shape_of(d, name)
    return swatch(s, "" if s == "random" else lc_attr(d, name))


def fx_dot(colour):
    return f'<svg class="sw" width="14" height="14" aria-hidden="true"><circle cx="7" cy="7" r="4.5" fill="{colour}"/></svg>'


def fx_legend(d, refs=False, extra=""):
    """The colour key, one per factor (the page script shows the one picked; without it, the default's), the grey
    values together; then the markers in use."""
    out = []
    for k, label, *_ in FACTORS:
        cols = fx_colours(d, k)
        vals = fx_values(d, k)
        items = [f'<span>{fx_dot(cols[v])}{esc(fx_label(k, v))}</span>' for v in vals if cols[v] != "var(--muted)"]
        grey = [fx_label(k, v) for v in vals if cols[v] == "var(--muted)"]
        if grey:
            items.append(f'<span>{fx_dot("var(--muted)")}{esc(", ".join(grey))}</span>')
        out.append(f'<div class="legend fxl" data-fx="{k}"{"" if k == FX_BY else " hidden"}><span class="fxl-h">'
                   f'{esc(label)}:</span>{"".join(items)}</div>')
    have = {shape_of(d, l["name"]) for l in d.registry + d.former}
    marks = [f'<span>{swatch(s)}{esc(t)}</span>' for s, t in (("merge", "merging tree (merge or fresh)"),
                                                               ("ht", "HT (router H)"), ("dag", "DAG")) if s in have]
    if refs:
        marks.append(f'<span>{swatch("random")}random players (references)</span>')
    if marks or extra:
        out.append('<div class="legend fxs"><span class="fxl-h">Markers:</span>' + "".join(marks) + extra + "</div>")
    return "".join(out)


def fx_varies(d, names):
    """What varies among some lineages: the factors with more than one value, then the variant's hyperparameters."""
    fs = [fx(d)[n] for n in names if n in fx(d)]
    out = [label.lower() for k, label, *_ in FACTORS if len({f[k] for f, _ in fs}) > 1]
    keys = list(dict.fromkeys(x["k"] for _, var in fs for x in var))
    kv = [k for k in keys if len({next((x["v"] for x in var if x["k"] == k), None) for _, var in fs}) > 1]
    return ", ".join(out) + (("; " if out else "") + "variant: " + ", ".join(kv) if kv else "")


def fx_scores(d):
    """{lineage: {"peak": {ply: [Elo, ±, version, cycle]}, "now": {ply: [Elo, ±, entry, games]}}}: peak, the lineage's
    highest rating at that ply at a completed cycle (the Peak Elo column's; at 1-ply the league's Elo history, which
    keeps no ±); now, its newest entry at that ply in today's fit, the field's, else its last retired one's."""
    if getattr(d, "_fxs", None) is not None:
        return d._fxs
    out = {}
    for lin, per in peak_points(d).items():
        for p, s in per.items():
            if s:
                top = max(s, key=lambda pt: pt["elo"])
                out.setdefault(lin, {"peak": {}, "now": {}})["peak"][str(p)] = [
                    round(top["elo"], 1), None if top["ci"] is None else round(top["ci"], 1), top["v"], top["c"]]
    infield = field_names(d)
    for pool in ([n for n, _, _ in d.players if n in infield], list(gone_entries(d))):
        best = {}
        for n in pool:
            lin, v, p = d.split_entry(n)
            if v is not None and n in d.ratings and ((lin, p) not in best or v > best[(lin, p)][0]):
                best[(lin, p)] = (v, n)
        for (lin, p), (v, n) in best.items():
            now = out.setdefault(lin, {"peak": {}, "now": {}})["now"]
            if str(p) not in now:
                r = d.ratings[n]
                now[str(p)] = [round(r["elo"], 1), round(r["ci"], 1), n, r["games"]]
    d._fxs = out
    return out


def fx_data(d):
    """The facets for the page script: the factors (values, reference, colours, labels), the entries' plies, and per
    lineage (lineages.txt's order, then the retired ones): [name, its values, variant text, {hyperparameter: value},
    status, retired, peaks, Elo now, archetype]."""
    scores = fx_scores(d)
    facs = []
    for k, label, _, ref in FACTORS:
        vals, cols = fx_values(d, k), fx_colours(d, k)
        facs.append([k, label, vals, ref, [cols[v] for v in vals], [fx_label(k, v) for v in vals]])
    lins = []
    for l in d.registry + d.former:
        f, var = fx(d)[l["name"]]
        sc = scores.get(l["name"]) or {"peak": {}, "now": {}}
        lins.append([l["name"], [f[k] for k, *_ in FACTORS], ", ".join(x["t"] for x in var if x["show"]),
                     {x["k"]: x["v"] for x in var}, status_of(d, l["name"]), int(is_retired(d, l["name"])),
                     sc["peak"], sc["now"], l["arch"]])
    plies = [str(p) for p in (d.cfg.get("tournament") or {}).get("entry_depths") or [1, 2]]
    return dict(by=FX_BY, ply=str(d.qualify_ply), plies=plies, factors=facs, linked=FX_LINKED, lins=lins)


def meter(frac, label=""):
    """A thin progress track, full at 1."""
    frac = max(0.0, min(1.0, frac or 0.0))
    return f'<span class="meter" title="{esc(label)}"><span class="fill" style="width:{100 * frac:.1f}%"></span></span>'


def budget_meter(value, budget):
    """CPU against the budget: the track spans max(value, budget); blue up to the budget, the serious status
    colour beyond it, and a tick where the budget ends."""
    value = max(0.0, value or 0.0)
    span = max(value, budget) or 1.0
    inb, over = min(value, budget) / span, max(0.0, value - budget) / span
    out = f'<span class="meter"><span class="fill" style="width:{100 * inb:.1f}%"></span>'
    if over:
        out += (f'<span class="over" style="left:{100 * inb:.1f}%;width:{100 * over:.1f}%"></span>'
                f'<span class="tick" style="left:{100 * budget / span:.1f}%"></span>')
    return out + "</span>"


# ---------------------------------------------------------------- the data

class Data:
    """Everything one page shows, read once per request."""

    def __init__(self, config_path):
        self.now = time.time()
        self.config_path = os.path.abspath(config_path)
        self.base = os.path.dirname(self.config_path)
        self.cfg = load(self.config_path, json.loads) or {}
        root = self.cfg.get("root", os.path.dirname(HERE))
        self.abs = lambda p: p if os.path.isabs(p) else os.path.join(root, p)
        f = lambda k, d: self.abs(self.cfg.get(k, d))
        self.root = root
        self.state_path = f("state", "cycles/state.json")
        self.league = f("league_dir", "selfplay/league-cycles")
        self.runs_dir = f("runs_dir", "cycles/runs")
        self.log_dir = f("log_dir", "cycles")
        self.state = load(self.state_path, json.loads) or {}
        reg = load(f("registry", "cycles/lineages.txt"), parse_registry) or []
        self.registry = [l for l in reg if not l.get("former")]  # the lineages the loop runs
        self.former = former_lineages(self.state, reg)  # retired ones, kept for their history
        self.lin = {l["name"]: l for l in self.former + self.registry}
        self.cycles_rows = load(f("log_csv", "cycles/cycles.csv"), parse_csv(["cycle", "lineage"])) or []
        self.ratings = load(os.path.join(self.league, "ratings.csv"), parse_ratings) or {}
        self.players = load(os.path.join(self.league, "players.txt"), parse_entries) or []
        self.retired = load(os.path.join(self.league, "retired.txt"), parse_entries) or []
        self.entry_cycle = load(os.path.join(self.league, "models.csv"), parse_models_csv) or {}
        self.elo_hist = load(os.path.join(self.league, "elo_history.csv"), parse_elo_history) or {}
        self.loop_log = load(os.path.join(self.base, "loop.log"), parse_loop_log) or dict(starts={}, events=[])
        self.stop_pending = os.path.exists(os.path.join(self.base, "STOP"))
        self.budget = num(self.cfg.get("budget_cpu_s"), 480.0)
        self.slots = int(self.cfg.get("jobs", 2))
        t = self.cfg.get("tournament", {})
        self.rr = int(t.get("round_robins_per_cycle", 5))
        self.qualify_ply = int(t.get("qualify_depth", 2))
        self.cur = self.state.get("current") or None
        self.refs = self.state.get("references") or [n for n, _, _ in self.players if n.startswith("random")]
        self.procs = processes()
        self.loop_proc = next((p for p in self.procs if any(a.endswith("loop.sh") for a in p["args"][:2])
                               and os.path.realpath(p["cwd"]) == os.path.realpath(self.base)), None)
        self.driver = next((p for p in self.procs if any(a.endswith("cycle.py") for a in p["args"][:3])
                            and "run" in p["args"]), None)
        self.jobs_running = {}
        for p in self.procs:
            a = p["args"]
            if "self-play" in a and a.index("self-play") + 1 < len(a):
                d = a[a.index("self-play") + 1]
                if os.path.dirname(os.path.realpath(d)) == os.path.realpath(self.runs_dir):
                    self.jobs_running[os.path.basename(d)] = p
        self.tournament_proc = next((p for p in self.procs if "tournament" in p["args"][1:3]), None)
        self.own_proc = next((p for p in self.procs if len(p["args"]) > 1 and p["args"][1] in ("train", "arena")
                              and any(self.runs_dir in a for a in p["args"])), None)
        self.history = self.read_history()

    def family(self, name):
        if name in self.refs or name.startswith("random"):
            return "random"
        lin = self.lin.get(name) or self.lin.get(self.split_entry(name)[0])
        return lin["family"] if lin else "other"

    @staticmethod
    def split_entry(name):
        m = re.fullmatch(r"(.+)-v(\d+)-(\d+)", name)
        return (m.group(1), int(m.group(2)), int(m.group(3))) if m else (name, None, None)

    def read_history(self):
        """Per logged cycle: its rows, start, end, phase times and counts."""
        by = {}
        for r in self.cycles_rows:
            c = int(num(r.get("cycle"), -1))
            if c >= 0:
                by.setdefault(c, []).append(r)
        out = []
        for c in sorted(by):
            rows = by[c]
            md_path = os.path.join(self.log_dir, f"cycle-{c:03d}.md")
            md = load(md_path, parse_cycle_md) or {}
            try:
                end = os.stat(md_path).st_mtime
            except OSError:
                end = None
            start = self.loop_log["starts"].get(c)
            walls = [num(r.get("wall_s")) for r in rows if num(r.get("wall_s")) is not None]
            free, makespan = [0.0] * max(1, self.slots), 0.0
            for w in walls:  # the driver's queue: each job takes the first free slot, in registry order
                i = free.index(min(free))
                free[i] += w
                makespan = max(makespan, free[i])
            tmin = md.get("tournament_min")
            took = end - start if end and start else None
            own = took - makespan - 60 * tmin if took and tmin is not None and walls else None
            gates = [r.get("gate") for r in rows if r.get("gate")]
            cpu = [num(r.get("cpu_s")) for r in rows if num(r.get("cpu_s")) is not None]
            out.append(dict(
                cycle=c, rows=rows, start=start, end=end, took=took, selfplay=makespan if walls else None,
                tournament=60 * tmin if tmin is not None else None, own=own if own is None or own > 0 else 0.0,
                trained=sum(1 for r in rows if num(r.get("games"))), cpu=sum(cpu) if cpu else None,
                budget=md.get("budget") or self.budget, round_robins=md.get("round_robins"),
                qualified=sum(1 for r in rows if r.get("status") == "qualified"),
                dormant=sum(1 for r in rows if r.get("status") == "dormant"),
                frozen=sum(1 for r in rows if "frozen" in (r.get("status"), r.get("decision"))),
                retired=sum(1 for r in rows if r.get("status") == "retired"),
                kept=sum(1 for g in gates if g == "kept"), gates=sum(1 for g in gates if g in ("kept", "reverted")),
                pooled=md.get("pooled"), summary=bool(md)))
        return out


# ---------------------------------------------------------------- panel 1: now

def job_rows(d):
    """The current cycle's self-play jobs: (name, plan, row, status, cpu, wall, rounds done). A job runs one command
    at a time (its measuring segment, then chunks); between two, for a moment, none runs: while the driver runs and a
    slot is free, a job with parts so far is still running. Its CPU so far: its parts', the running command's, and
    cut_cpu (rounds a command cut short by a driver restart saved), as the driver counts its budget."""
    cur = d.cur
    out = []
    gap = d.driver is not None and len(d.jobs_running) < d.slots
    for name, p in (cur.get("jobs") or {}).items():
        row = (cur.get("rows") or {}).get(name, {})
        lin = d.lin.get(name, {})
        parts = [x for x in p.get("parts") or [] if isinstance(x, list) and len(x) > 3]
        if name in (cur.get("done") or []):
            out.append((name, p, row, "done", num(row.get("cpu_s")), num(row.get("wall_s")),
                        int(num(row.get("rounds"), p.get("rounds", 0)))))
        elif name in d.jobs_running or (parts and gap and not p.get("failed")):
            pr = d.jobs_running.get(name)
            k = last_version(os.path.join(d.runs_dir, name), lin.get("kind", "lr"), int(p.get("start", 0)))
            out.append((name, p, row, "running",
                        (pr["cpu"] if pr else 0.0) + sum(num(x[2], 0) for x in parts) + num(p.get("cut_cpu"), 0),
                        (d.now - pr["start"] if pr else 0.0) + sum(num(x[3], 0) for x in parts),
                        k - int(p.get("start", 0))))
        else:
            out.append((name, p, row, "pending", None, None, 0))
    return out


def estimates(d):
    """(tournament seconds, own-games seconds) for this cycle, from earlier cycles when there are any."""
    t = [h["tournament"] for h in d.history if h["tournament"]]
    o = [h["own"] for h in d.history if h["own"]]
    n = len(d.players) or 63
    games = d.rr * n * (n - 1)
    own_n = sum(1 for l in d.registry if l["own"] and not is_frozen(d, l["name"]) and not is_retired(d, l["name"]))
    return (t[-1] * games / max(1, t and games or 1) if False else (t[-1] if t else 60.0 * games / 19530),
            o[-1] if o else 12.0 * own_n)


def panel_now(d):
    cur, out = d.cur, []
    alive = d.loop_proc is not None
    if alive:
        loop_v = '<span class="ok">●</span> running'
        loop_s = f"loop pid {d.loop_proc['pid']}" + (f" · driver pid {d.driver['pid']}" if d.driver else "")
    else:
        loop_v = '<span class="bad">✕</span> not running'
        loop_s = ("stopped mid-cycle: start the loop again to resume" if cur else
                  "start it with ./loop.sh (see README)")
    stop_v = ('<span class="warn">▲</span> pending' if d.stop_pending else "none")
    stop_s = "the loop stops after this cycle" if d.stop_pending else "touch cycles/STOP to stop between cycles"
    if cur:
        cycle, phase = cur.get("cycle"), cur.get("phase", "?")
        start = d.loop_log["starts"].get(cycle)
    else:
        cycle, phase, start = d.state.get("cycle"), None, None
    jobs = job_rows(d) if cur else []
    tour_s, own_s = estimates(d)
    eta, prog = None, ""
    if cur and phase in PHASES:
        remaining = 0.0
        i = PHASES.index(phase)
        if phase == "self-play":
            fin = [j for j in jobs if j[3] == "done"]
            planned = lambda p: d.budget if "base" in p else num(p.get("cost"), 0) * num(p.get("games"), 0)
            done_plan = sum(planned(j[1]) for j in fin)
            ratio = sum(j[4] or 0 for j in fin) / done_plan if done_plan else 1.0
            left = sum(planned(j[1]) * ratio for j in jobs if j[3] == "pending")
            left += sum(max(0.0, planned(j[1]) * ratio - (j[4] or 0)) for j in jobs if j[3] == "running")
            used = [(j[4], j[5]) for j in jobs if j[3] in ("done", "running") and j[4] and j[5]]
            rate = (sum(c for c, _ in used) / sum(w for _, w in used) if used else CORES_PER_JOB) * d.slots
            remaining += left / rate
            done_n = sum(1 for j in jobs if j[3] == "done")
            prog = f"{done_n} of {len(jobs)} jobs done, {sum(1 for j in jobs if j[3] == 'running')} running"
        if i <= 1:
            remaining += tour_s
        if phase == "tournament":
            counts = count_games(os.path.join(d.league, "games.csv"))
            target = cur.get("target") or 0
            n = len(d.players)
            want = d.rr * n * (n - 1)
            got = sum(v for t, v in counts.items() if t >= target - d.rr)
            remaining -= tour_s * min(1.0, got / want) if want else 0
            prog = f"{got:,} of {want:,} games"
        if i <= 3:
            now_rows = cur.get("rows") or {}
            own_lins = [l for l in d.registry if l["own"] and l["name"] in now_rows
                        and not is_frozen(d, l["name"], now_rows.get(l["name"]))
                        and (now_rows.get(l["name"]) or {}).get("status") != "retired"]  # frozen or retired: none
            done_own = len(cur.get("own_done") or [])
            remaining += own_s * (1 - (done_own / len(own_lins) if own_lins and phase == "own" else 0))
            if phase == "own":
                prog = f"{done_own} of {len(own_lins)} lineages"
        eta = d.now + max(0.0, remaining)
    tiles = [
        ("Loop", loop_v, loop_s),
        ("STOP file", stop_v, stop_s),
        ("Cycle", esc(str(cycle)) if cycle is not None else "—",
         (PHASE_LABEL.get(phase, phase) + (f": {prog}" if prog else "")) if cur else
         ("between cycles" if alive else "no cycle running")),
        ("Elapsed", fmt_dur(d.now - start) if start else "—", f"started {fmt_clock(start)}" if start else ""),
        ("Ends about", fmt_clock(eta) if eta else "—", f"in {fmt_dur(eta - d.now)} (rough)" if eta else ""),
    ]
    out.append('<div class="tiles">' + "".join(
        f'<div class="tile"><div class="tl">{esc(a)}</div><div class="tv">{b}</div><div class="ts">{esc(c)}</div>'
        f"</div>" for a, b, c in tiles) + "</div>")
    if not cur:
        if d.history:
            h = d.history[-1]
            out.append(f'<p class="note">Last finished: cycle {h["cycle"]} (see cycles/cycle-{h["cycle"]:03d}.md).</p>')
        return "".join(out)

    # self-play jobs
    if jobs:
        running = [j for j in jobs if j[3] == "running"]
        done = [j for j in jobs if j[3] == "done"]
        pending = [j for j in jobs if j[3] == "pending"]
        if phase == "self-play":
            used = sum(j[4] or 0 for j in jobs if j[3] != "pending")
            want = d.budget * len(jobs)
            out.append(f'<div class="bar-line"><span>Self-play CPU so far</span>{meter(used / want if want else 0)}'
                       f'<span class="num">{fmt_cpu(used)} of {fmt_cpu(want)} budgeted '
                       f'({len(done)}/{len(jobs)} jobs done)</span></div>')
        if running:
            out.append('<h3>Running</h3><table><tr><th>lineage</th><th>rounds</th><th></th><th class="r">CPU-s</th>'
                       f'<th>vs {d.budget:.0f} budget</th><th class="r">wall</th><th class="r">cores</th></tr>')
            for name, p, row, _, cpu, wall, k in running:
                n = int(p.get("total") or p.get("rounds", 0))
                how = ("" if "base" not in p else " <span class='muted'>(measuring)</span>" if "end" not in p
                       else " <span class='muted'>(measured)</span>")
                out.append(f"<tr><td>{sw(d, name)} {esc(name)}{how}</td><td class='r'>{k} of "
                           f"{'~' if 'base' in p and 'end' not in p else ''}{n}</td>"
                           f"<td>{meter(k / n if n else 0)}</td><td class='r'>{cpu:.0f}</td>"
                           f"<td>{budget_meter(cpu, d.budget)}</td><td class='r'>{fmt_dur(wall)}</td>"
                           f"<td class='r'>{cpu / wall if wall else 0:.1f}</td></tr>")
            out.append("</table>")
        if done:
            out.append(f'<details id="done-jobs" open><summary>Finished ({len(done)})</summary><table><tr>'
                       '<th>lineage</th><th class="r">rounds × games</th><th class="r">CPU-s</th>'
                       f'<th>vs {d.budget:.0f}</th><th class="r">wall</th><th class="r">ms CPU/game</th>'
                       '<th class="r">measured or planned</th></tr>')
            for name, p, row, _, cpu, wall, k in done:
                games = num(row.get("games"), num(p.get("games"), 0))
                over = (cpu / d.budget - 1) if cpu and cpu > d.budget else None
                flag = f' <span class="warn">▲</span> {100 * over:.0f}%' if over and over > 0.1 else ""
                out.append(f"<tr><td>{sw(d, name)} {esc(name)}</td>"
                           f"<td class='r'>{k} × {int(num(p.get('size'), 0))}</td><td class='r'>{cpu or 0:.0f}{flag}</td>"
                           f"<td>{budget_meter(cpu, d.budget)}</td><td class='r'>{fmt_dur(wall)}</td>"
                           f"<td class='r'>{1000 * (cpu or 0) / games if games else 0:.2f}</td>"
                           f"<td class='r'>{num(row.get('cost_ms')) or 1000 * num(p.get('cost'), 0):.2f}</td></tr>")
            out.append("</table></details>")
        if pending:
            out.append('<p class="note">Waiting: ' + ", ".join(esc(j[0]) for j in pending) + ".</p>")
    if phase == "tournament":
        counts = count_games(os.path.join(d.league, "games.csv"))
        target = cur.get("target") or 0
        n = len(d.players)
        want = d.rr * n * (n - 1)
        got = sum(v for t, v in counts.items() if t >= target - d.rr)
        last = [l for l in tail(os.path.join(d.league, "tournament.log")).splitlines() if l.startswith("tournament ")]
        out.append(f'<div class="bar-line"><span>Tournament</span>{meter(got / want if want else 0)}'
                   f'<span class="num">{got:,} of {want:,} games ({d.rr} double round-robins of {n} players)</span></div>')
        if last:
            out.append(f'<p class="note">{esc(unsign(last[-1][:200]))}</p>')
    own_rows = [r for r in (cur.get("rows") or {}).values() if r.get("gate")]
    if phase in ("own", "log") or own_rows:
        out.append(own_table(d, own_rows))
        if d.own_proc and phase == "own":
            a = " ".join(d.own_proc["args"])
            m = re.search(r"\.own-([A-Za-z0-9+.-]+)", a) or re.search(re.escape(d.runs_dir) + r"/([A-Za-z0-9+.-]+)/", a)
            what = "training on its own games" if d.own_proc["args"][1] == "train" else "gate match"
            out.append(f'<p class="note">Now: {esc(m.group(1) if m else "?")}, {what} '
                       f'({fmt_dur(d.now - d.own_proc["start"])}).</p>')
    if cur.get("extra"):
        out.append(notes_html(cur["extra"]))
    return "".join(out)


def md_inline(t):
    """A note's text as HTML: escaped, with markdown's **bold** and `code`."""
    t = esc(t)
    return re.sub(r"`([^`]+)`", r"<code>\1</code>", re.sub(r"\*\*(.+?)\*\*", r"<b>\1</b>", t))


def notes_html(lines):
    """The cycle's notes (cycle.py's current.extra): one list item a line, except the markdown it writes for the
    cycle's .md, which shows as such: blank lines skipped, **bold**, and a pipe table (the freeze/retire rule's
    preview) as a table."""
    out, items, i, lines = [], [], 0, [str(x) for x in lines]

    def flush():
        if items:
            out.append('<ul class="notes">' + "".join(f"<li>{x}</li>" for x in items) + "</ul>")
            items.clear()
    while i < len(lines):
        t = lines[i].strip()
        if not t.startswith("|"):
            if t:
                items.append(md_inline(t))
            i += 1
            continue
        rows = []
        while i < len(lines) and lines[i].strip().startswith("|"):
            rows.append([c.strip() for c in lines[i].strip().strip("|").split("|")])
            i += 1
        right = []
        if len(rows) > 1 and all(re.fullmatch(r":?-+:?", c) for c in rows[1]):
            right = [c.endswith(":") for c in rows[1]]
            del rows[1]
        cls = lambda j: ' class="r"' if j < len(right) and right[j] else ""
        flush()
        out.append('<div class="tscroll"><table class="mdt">' + "".join(
            "<tr>" + "".join(f"<{tag}{cls(j)}>{md_inline(c)}</{tag}>" for j, c in enumerate(r)) + "</tr>"
            for k, r in enumerate(rows) for tag in ["th" if k == 0 else "td"]) + "</table></div>")
    flush()
    return "".join(out)


def own_table(d, rows):
    if not rows:
        return '<p class="note">No own-games step finished yet in this cycle.</p>'
    out = ['<h3>Own-games steps and gates</h3><table><tr><th>lineage</th><th class="r">own games</th>'
           '<th class="r">positions</th><th class="r">gate +/−/=</th><th class="r">score</th><th class="r">Elo</th>'
           '<th>result</th></tr>']
    order = {l["name"]: i for i, l in enumerate(d.registry)}
    for r in sorted(rows, key=lambda r: order.get(r.get("lineage"), 99)):
        g = r.get("gate", "")
        mark = ('<span class="ok">✓</span> kept' if g == "kept" else '<span class="bad">✕</span> reverted'
                if g == "reverted" else esc(g))
        score = num(r.get("gate_score"))
        out.append(f"<tr><td>{sw(d, r.get('lineage', ''))} {esc(r.get('lineage', ''))}</td>"
                   f"<td class='r'>{esc(str(r.get('own_games', '')))}</td>"
                   f"<td class='r'>{esc(str(r.get('own_positions', '')))}</td>"
                   f"<td class='r'>{esc(str(r.get('gate_wins', '')))}/{esc(str(r.get('gate_losses', '')))}/"
                   f"{esc(str(r.get('gate_draws', '')))}</td>"
                   f"<td class='r'>{'' if score is None else f'{100 * score:.1f}%'}</td>"
                   f"<td class='r'>{signed(num(r.get('gate_elo')))} ± {num(r.get('gate_ci95'), 0):.0f}</td>"
                   f"<td>{mark}</td></tr>")
    kept = sum(1 for r in rows if r.get("gate") == "kept")
    out.append(f"</table><p class='note'>{kept} of {len(rows)} kept so far.</p>")
    return "".join(out)


# ---------------------------------------------------------------- panel 2: standings

def status_of(d, lineage):
    st = (d.state.get("lineages") or {}).get(lineage)
    l = d.lin.get(lineage) or {}
    if l.get("former") or (isinstance(st, dict) and st.get("status") == "retired"):
        c = l.get("retired_after")
        if c is None and isinstance(st, dict) and num(st.get("retired_cycle")) is not None:
            c = int(num(st["retired_cycle"]))
        if c is None and isinstance(st, dict) and isinstance(st.get("rule"), dict) and st.get("retired_by") == "rule":
            c = int(num(st["rule"].get("cycle"), -1)) if num(st["rule"].get("cycle")) is not None else None
        return f"retired after cycle {c}" if c is not None else "retired"
    if not st:
        return "—"
    s = st.get("status", "?")
    if s == "qualified" and st.get("strikes"):
        return f"qualified, strike {st['strikes']}"
    return s


def is_frozen(d, name, row=None):
    """Whether a lineage is frozen: by its row of the cycle under way when that has a status or decision (a frozen
    one's say "frozen" from the cycle's start), else by state.json."""
    if isinstance(row, dict) and (row.get("status") or row.get("decision")):
        return "frozen" in (row.get("status"), row.get("decision"))
    st = (d.state.get("lineages") or {}).get(name)
    return isinstance(st, dict) and st.get("status") == "frozen"


def frozen_since(d, lineage):
    """The first cycle of the lineage's latest unbroken run of frozen rows (cycles.csv, then the cycle under way), or
    None."""
    rows = {int(num(r.get("cycle"), -1)): r for r in d.cycles_rows if r.get("lineage") == lineage}
    cur = d.cur or {}
    if isinstance(cur.get("cycle"), int) and isinstance((cur.get("rows") or {}).get(lineage), dict):
        rows[cur["cycle"]] = cur["rows"][lineage]
    since = None
    for c in sorted(rows, reverse=True):
        if "frozen" not in (rows[c].get("status"), rows[c].get("decision")):
            break
        since = c
    return since


def frozen_note(d, lineage):
    """The frozen badge's hover: since when, what it was, the entry it keeps in the field and the one retired while
    frozen, and what unfreezing restores."""
    st = (d.state.get("lineages") or {}).get(lineage) or {}
    since = frozen_since(d, lineage)
    head = f"{lineage}: frozen" + (f" since cycle {since}" if since is not None else "")
    if st.get("frozen_from"):
        head += f" (it was {st['frozen_from']} when frozen)"
    out = [head + "."]
    kept_e, gone = list(st.get("entries") or []), list(st.get("frozen_retired") or [])
    if kept_e:
        out.append(f"In the field: {', '.join(kept_e)}" + (f"; retired while frozen: {', '.join(gone)}" if gone else "")
                   + ".")
    out.append("No self-play, own-games step, gate or qualification decision while frozen; its entry keeps playing "
               "every tournament, so its rating stays current.")
    kept = []
    if num(st.get("strikes")):
        n = int(num(st["strikes"]))
        kept.append(f"{n} strike{'s' if n != 1 else ''}")
    if st.get("dormant_version") is not None:
        kept.append(f"the version it went dormant at, v{st['dormant_version']}")
    if kept:
        out.append(f"Kept for when it's unfrozen: {' and '.join(kept)}" + (" (and its retired entry comes back)" if gone
                                                                            else "") + ".")
    return "\n".join(out)


def predecessor(d, lineage, version, ply):
    best = None
    for name, r in d.ratings.items():
        lin, v, p = d.split_entry(name)
        if lin == lineage and p == ply and v is not None and v < version and (best is None or v > best[0]):
            best = (v, r)
    return best


PEAK_HEAD = ("the lineage's highest rating at the row's ply as measured at the completion of a cycle, and the version "
             "that played that cycle: completed cycles only (cycles.csv's Elo at the decision at the qualifying ply, "
             "the league's Elo history at the cycle's last round-robin at the other; the cycle under way counts once "
             "it has decided, never the running fit mid-tournament). In parentheses on dormant rows: the same peak "
             "over the cycles up to the one whose decision made it dormant. ")
PEAK_NOTE = PEAK_HEAD + ("The rule compares a version with the best earlier version in today's fit instead (in the "
                         "hover): a lineage gets a strike when it's more than {dse} below that, and a dormant one "
                         "wakes when it beats it by more than {rse}.")
# from the cycle that keeps dormant_version (the dormant rule changed)
PEAK_NOTE_DORMANT = PEAK_HEAD + ("The rule compares a version with the best earlier version in today's fit instead (in "
                                 "the hover): a lineage gets a strike when it's more than {dse} below that. The "
                                 "rule's wake bar (in the hover, not the parentheses): its best version up to the one "
                                 "that went dormant, this one excluded, in today's fit; it must beat this by more than "
                                 "{rse} to wake.")


def se_words(d, key):
    """config.json's dormancy_se (a strike) or requalify_se (waking) in words: "1.5 standard errors"."""
    k = float(num(d.cfg.get(key), 2.0))
    return f"{k:g} standard error" + ("" if k == 1 else "s")


def peak_lines(lin, ply, pt):
    """A Peak cell's hover: the completed cycle's point it is (its cycle, and how that cycle measured it) and that
    version's rating in today's fit."""
    how = ("live, today's fit" if pt["live"] else
           f"at that cycle's decision, {pt.get('src') or 'cycles.csv'}" if pt["ci"] is not None
           else "after that cycle's last round-robin, the league's Elo history")
    out = [f"{lin}: peak at ply {ply} {plain(pt['elo'], 1)}" + (f" ± {pt['ci']:.0f}" if pt["ci"] is not None else "")
           + f", v{pt['v']} as measured in cycle {pt['c']} ({how})"]
    if pt.get("today") is not None:
        out.append(f"v{pt['v']} in today's fit: {plain(pt['today'], 1)}"
                   + (f" ± {pt['today_ci']:.0f}" if pt.get("today_ci") is not None else "")
                   + ("" if pt.get("field") else " (retired)"))
    else:
        out.append(f"v{pt['v']}: not rated at ply {ply} in today's fit")
    return out


def entry_plies(d):
    """{player: ply} from players.txt and retired.txt, as cycle.py's league_depths reads them."""
    out = {n: p for n, p, _ in d.retired}
    out.update({n: p for n, p, _ in d.players})
    return out


def ply_versions(d, plies):
    """{lineage: {version: (Elo, ±)}} of the rated versions at the qualifying ply in the current fit, retired ones
    included (they stay in the fit). A version's rating is its entries' mean there (one model can have two entries
    at a ply), as in cycle.py's lineage_peak; a player missing from both files takes the ply in its name."""
    by = {}
    for name, r in d.ratings.items():
        lin, v, p = d.split_entry(name)
        if v is not None and plies.get(name, p) == d.qualify_ply:
            by.setdefault(lin, {}).setdefault(v, []).append(r)
    return {lin: {v: (sum(r["elo"] for r in rs) / len(rs), sum(r["ci"] for r in rs) / len(rs))
                  for v, rs in vs.items()} for lin, vs in by.items()}


def earlier_peak(versions, version):
    """(version, Elo, ±) of the best of a lineage's versions before `version`, or None if none is rated; a tie keeps
    the earlier version, as cycle.py's lineage_peak does."""
    best = None
    for v in sorted(versions or {}):
        if v >= version:
            break
        if best is None or versions[v][0] > best[1]:
            best = (v, *versions[v])
    return best


def dormant_cap(d, lineage):
    """The version a dormant lineage went dormant at (state.json's dormant_version): its peak from before then is its
    best version at the qualifying ply numbered up to that one, the current version's entries still skipped, as in
    cycle.py. None for a qualified lineage, or before cycle.py kept the field."""
    st = (d.state.get("lineages") or {}).get(lineage) or {}
    if st.get("status") != "dormant" or st.get("dormant_version") is None:
        return None
    try:
        return int(st["dormant_version"])
    except (TypeError, ValueError):
        return None


def version_of(x):
    """A logged peak version ("816", "v816", 816) as an int, or None."""
    s = str(x if x is not None else "").strip().lstrip("v")
    v = num(s)
    return int(v) if v is not None and v == int(v) else None


def logged_peaks(d, plies, versions):
    """Each lineage's latest logged peaks, from cycles.csv or from the cycle under way once it has decided
    (state.json), next to this fit's: {lineage: dict(cycle, src, version, elo, pre, same, agrees, how)}. version and
    elo: peak_version and peak_elo, the all-time peak; pre: (pre_dormancy_peak_version, pre_dormancy_peak_elo) when
    logged (dormant rows), else None. same: that cycle's rows are from this same fit (each row's own Elo is this
    fit's for its entries), so they should match mine: the best version before the row's, and the best one up to the
    version it went dormant at (state.json's dormant_version). agrees is None when they are from another fit; how
    says which cut the pre-dormancy peak was checked with."""
    rows = [(int(num(r.get("cycle"), -1)), "cycles.csv", r) for r in d.cycles_rows]
    if d.cur and d.cur.get("cycle") is not None:
        rows += [(int(d.cur["cycle"]), "state.json", r) for r in (d.cur.get("rows") or {}).values()]
    fit = {}  # (src, cycle): every row of it that has an Elo has this fit's
    for c, src, r in rows:
        e = num(r.get("elo"))
        if e is not None:
            ents = {n: plies.get(n, d.split_entry(n)[2]) for n in str(r.get("entries") or "").split()}
            at = (d.qualify_ply if d.qualify_ply in ents.values() else  # a frozen row's: its kept entry's ply
                  next(iter(ents.values())) if len(set(ents.values())) == 1 else None)
            names = [n for n, p in ents.items() if n in d.ratings and p == at]
            now = sum(d.ratings[n]["elo"] for n in names) / len(names) if names else None
            fit[(src, c)] = fit.get((src, c), True) and now is not None and abs(now - e) < 0.051
    out = {}
    for c, src, r in rows:
        lin, pv, pe = r.get("lineage"), version_of(r.get("peak_version")), num(r.get("peak_elo"))
        qv, qe = version_of(r.get("pre_dormancy_peak_version")), num(r.get("pre_dormancy_peak_elo"))
        if not lin or (pv is None and pe is None) or c < out.get(lin, {}).get("cycle", -1):
            continue
        same, ver = fit.get((src, c), False), num(r.get("version"))
        agrees, how = None, ""
        if same and ver is not None:
            vl, ver = versions.get(lin), int(ver)
            hit = lambda pk, v_, e_: pk is not None and pk[0] == v_ and e_ is not None and abs(pk[1] - e_) < 0.051
            frozen = r.get("decision") == "frozen"  # a frozen cycle's row; the rule's decision row logs the dormant check
            agrees = hit(earlier_peak(vl, 10 ** 9 if frozen else ver), pv, pe)  # a frozen row: the all-time peak
            if qv is not None or qe is not None:
                dv = dormant_cap(d, lin)
                if dv is not None:
                    how = f"up to v{dv}"
                    agrees = agrees and hit(earlier_peak(vl, min(ver, dv + 1)), qv, qe)
                elif qv is not None:  # it re-qualified in that decide, and state.json has cleared the version it went
                    how = f"up to v{qv} or later"  # dormant at: its peak then is still its best version up to its own
                    agrees = agrees and hit(earlier_peak(vl, qv + 1), qv, qe)
                else:
                    agrees = False
        out[lin] = dict(cycle=c, src=src, version=pv, elo=pe, pre=(qv, qe) if qv is not None or qe is not None else None,
                        same=same, agrees=agrees, how=how, frozen=r.get("decision") == "frozen")
    return out


def fit_cycles(d):
    """(the cycle today's fit ends with, {cycle: t of its last round-robin in the Elo history}), from the league's Elo
    history (t: the fit after t round-robins) and the round-robins each cycle played. Without the history: the newest
    cycle with a logged Elo, or the cycle under way once it has decided."""
    if getattr(d, "_fit_cycles", None) is not None:
        return d._fit_cycles
    cof = cycle_of_fn(d)
    last = {}
    for t in sorted(d.elo_hist):
        if t >= 1:
            last[cof(t - 1)] = t
    if last:
        res = (cof(max(last.values()) - 1), last)
    else:
        logged = [h["cycle"] for h in d.history if any(num(r.get("elo")) is not None for r in h["rows"])]
        c = max(logged) if logged else None
        if d.cur and d.cur.get("phase") in ("decide", "own", "log") and isinstance(d.cur.get("cycle"), int):
            c = d.cur["cycle"]
        res = (c, last)
    d._fit_cycles = res
    return res


def cycle_points(d):
    """{lineage: {ply: [point, ...]}}, in cycle order: each lineage's rating in every cycle it played. A point is a
    dict: c (the cycle), v (the version that played it), elo and ci (the 95% half-range, None when not kept), live
    (the cycle under way, from today's fit), since (the first cycle of that version), decision (that cycle's), and
    today, today_ci, today_games, field: that version's entries at that ply in today's fit, and whether one is in
    the field.
    A finished cycle's rating is its cycles.csv row's elo and ci95, the fit at that cycle's decision, at the ply the
    row's entries say (the qualifying ply; a frozen row's kept entry's own); the other ply's comes from the league's
    Elo history at that cycle's last round-robin, which keeps no ±. The cycle under way gets today's fit
    (ratings.csv) for the lineage's entries in the field, once that fit has a round-robin of the cycle."""
    if getattr(d, "_points", None) is not None:
        return d._points
    plies = entry_plies(d)
    ply = lambda n: plies.get(n, d.split_entry(n)[2])
    fit_c, last = fit_cycles(d)
    found = {}  # lineage -> ply -> cycle -> point

    def add(lin, p, c, v, elo, ci, live, decision, into=None):
        if p is not None and v is not None and elo is not None:
            (found if into is None else into).setdefault(lin, {}).setdefault(p, {})[c] = dict(
                c=c, v=v, elo=elo, ci=ci, live=live, decision=decision or "")

    def from_row(r, c, into=None):  # a decided cycle's row: cycles.csv's, or the cycle under way's once decided
        lin, names = r.get("lineage"), str(r.get("entries") or "").split()
        if c < 1 or not lin or not names:
            return
        by = {}
        for n in names:
            by.setdefault(ply(n), []).append(n)
        own = d.qualify_ply if d.qualify_ply in by else next(iter(by)) if len(by) == 1 else None
        e, ci = num(r.get("elo")), num(r.get("ci95"))
        vs = {d.split_entry(n)[1] for n in names} - {None}
        v = int(num(r.get("version"))) if num(r.get("version")) is not None else min(vs) if len(vs) == 1 else None
        hist = d.elo_hist.get(last.get(c)) or {}
        for p, ns in by.items():
            if p == own and e is not None:
                add(lin, p, c, v, e, ci, False, r.get("decision"), into)
            else:
                got = [hist[n] for n in ns if n in hist]
                if got:
                    add(lin, p, c, v, sum(got) / len(got), None, False, r.get("decision"), into)

    for r in d.cycles_rows:
        from_row(r, int(num(r.get("cycle"), -1)))
    cur = d.cur or {}
    decided = {}  # the cycle under way once it has decided, at its decision's values: the Peak's (the chart stays live)
    if isinstance(cur.get("cycle"), int) and cur.get("phase") in ("own", "log"):
        for lin, r in (cur.get("rows") or {}).items():
            if isinstance(r, dict):
                from_row(dict(r, lineage=r.get("lineage") or lin), cur["cycle"], decided)
    cur = d.cur or {}
    if isinstance(cur.get("cycle"), int) and fit_c == cur["cycle"]:  # under way, and today's fit has some of it
        C, rows, field = cur["cycle"], cur.get("rows") or {}, {}
        for n, p, _ in d.players:
            lin, v, p_ = d.split_entry(n)
            if v is not None and n in d.ratings and not is_ref(d, n):
                field.setdefault((lin, p if p is not None else p_), {}).setdefault(v, []).append(d.ratings[n])
        for (lin, p), by_v in field.items():
            if C in found.get(lin, {}).get(p, {}):
                continue  # cycles.csv has it already
            v = max(by_v)
            rs = by_v[v]
            row = rows.get(lin) if isinstance(rows.get(lin), dict) else {}
            add(lin, p, C, v, sum(x["elo"] for x in rs) / len(rs), sum(x["ci"] for x in rs) / len(rs), True,
                row.get("decision"))
    today = {}
    for n, r in d.ratings.items():
        lin, v, _ = d.split_entry(n)
        if v is not None:
            today.setdefault((lin, v, ply(n)), []).append((n, r))
    infield = field_names(d)

    def dress(pt, lin, p):
        rs = today.get((lin, pt["v"], p)) or []
        pt.update(today=sum(r["elo"] for _, r in rs) / len(rs) if rs else None,
                  today_ci=sum(r["ci"] for _, r in rs) / len(rs) if rs else None,
                  today_games=sum(r["games"] for _, r in rs) if rs else None,
                  field=any(n in infield for n, _ in rs))
    out = {}
    for lin, per in found.items():
        for p, by_c in per.items():
            s, since, prev = [], None, None
            for c in sorted(by_c):
                pt = by_c[c]
                if since is None or pt["v"] != prev:
                    since = c
                prev = pt["v"]
                pt["since"] = since
                dress(pt, lin, p)
                s.append(pt)
            out.setdefault(lin, {})[p] = s
    peaks = {}  # the completed cycles' points only, the cycle under way at its decision's values once decided
    for lin in set(out) | set(decided):
        for p in set(out.get(lin, {})) | set(decided.get(lin, {})):
            s = [pt for pt in out.get(lin, {}).get(p, []) if not pt["live"]]
            for c, dp in sorted((decided.get(lin, {}).get(p) or {}).items()):
                s = [pt for pt in s if pt["c"] != c]
                dp.update(since=s[-1]["since"] if s and s[-1]["v"] == dp["v"] else c, src="state.json")
                dress(dp, lin, p)
                s.append(dp)
            peaks.setdefault(lin, {})[p] = s
    d._points, d._peaks = out, peaks
    return out


def peak_points(d):
    """{lineage: {ply: [point, ...]}}, in cycle order: the points the Peak columns count, completed cycles only:
    cycles.csv's, and the cycle under way at its decision's values once it has decided (state.json), never the
    running fit (the chart's live point)."""
    cycle_points(d)
    return d._peaks


def point_tip(lin, ply, pt):
    """A per-cycle point's hover: the version, its rating that cycle, and that version's in today's fit."""
    val = f"Elo {plain(pt['elo'])}" + (f" ± {pt['ci']:.0f}" if pt["ci"] is not None else "")
    src = ("live, today's fit (ratings.csv)" if pt["live"] else
           "at that cycle's decision (cycles.csv)" if pt["ci"] is not None else
           "after that cycle's last round-robin (the league's Elo history, which keeps no ±)")
    out = [f"{lin} v{pt['v']}, ply {ply}, cycle {pt['c']}" + (" (under way)" if pt["live"] else ""), f"{val}, {src}",
           f"the same version since cycle {pt['since']}" if pt["since"] < pt["c"] else f"v{pt['v']}'s first cycle"]
    if not pt["live"]:
        out.append(f"v{pt['v']} in today's fit: {plain(pt['today'])} ± {pt['today_ci']:.0f}"
                   + ("" if pt["field"] else " (retired)") if pt["today"] is not None else
                   f"v{pt['v']} at ply {ply}: not in today's fit")
    if pt["decision"]:
        out.append(f"cycle {pt['c']} decision: {pt['decision']}")
    return "\n".join(out)


_cov = {"sig": None, "factor": None, "good_sig": None, "busy": False, "memo": {}}  # factor: the newest good one
_cov_lock = threading.Lock()
ANCHOR = "random"  # the tournament's --anchor, pinned at Elo 0 (as in cycle.py)


def fit_factor(league, ratings, rounds=None, anchor=ANCHOR):
    """cycle.py's fit_factor, for a difference's standard error: the league fit's information matrix (Elo units) at
    its ratings ({player: Elo}), Cholesky-factored, a row per rated player but the anchor (pinned), from the games in
    games.csv between rated players, retired ones included; with `rounds`, only the first that many round-robins
    (the ones in the fit, while a tournament is under way). None if it is not positive definite."""
    names = [p for p in ratings if p != anchor]
    index = {p: i for i, p in enumerate(names)}
    count = {}
    with open(os.path.join(league, "games.csv"), encoding="utf-8", errors="replace") as f:
        next(f, None)
        for line in f:
            parts = line.split(",", 3)
            if len(parts) < 4 or not line.endswith("\n"):
                continue
            t, white, black = parts[0], parts[1], parts[2]
            if rounds is not None:
                try:
                    if int(t) >= rounds:
                        continue
                except ValueError:
                    continue
            if white in ratings and black in ratings and white != black:
                key = (white, black) if white < black else (black, white)
                count[key] = count.get(key, 0) + 1
    k = len(names)
    info = [[0.0] * k for _ in range(k)]
    for (a, b), n in count.items():
        p = 1 / (1 + 10 ** (-(ratings[a] - ratings[b]) / 400))
        w = n * p * (1 - p) * (math.log(10) / 400) ** 2
        ia, ib = index.get(a), index.get(b)
        for i in (ia, ib):
            if i is not None:
                info[i][i] += w
        if ia is not None and ib is not None:
            info[ia][ib] -= w
            info[ib][ia] -= w
    for j in range(k):  # Cholesky, in place: the lower triangle becomes L, with info = L L'
        row_j = info[j]
        dd = row_j[j] - sum(v * v for v in row_j[:j])
        if not dd > 0:
            return None
        row_j[j] = dd = math.sqrt(dd)
        for i in range(j + 1, k):
            row_i = info[i]
            row_i[j] = (row_i[j] - sum(x * y for x, y in zip(row_i[:j], row_j[:j]))) / dd
    return info, index


def contrast_se(factor, a, b):
    """cycle.py's contrast_se: the standard error (Elo) of the mean rating of players `a` minus that of players `b`,
    in the fit `factor` came from: sqrt(e' I^-1 e) = |L^-1 e|."""
    lower, index = factor
    e = [0.0] * len(lower)
    for names, sign in ((a, 1.0), (b, -1.0)):
        for n in names:
            if n in index:
                e[index[n]] += sign / len(names)
    y = []
    for i, row in enumerate(lower):
        y.append((e[i] - sum(x * v for x, v in zip(row[:i], y))) / row[i])
    return math.sqrt(sum(v * v for v in y))


def cov_factor(d):
    """(factor, fresh): fit_factor for today's ratings.csv, computed in a background thread when the fit changes (a few
    seconds of CPU). Until it is ready, or if it fails, the newest earlier fit's (fresh False); None if none."""
    hit = _cache.get(os.path.join(d.league, "ratings.csv"))
    if not d.ratings or not hit or hit[1] is not d.ratings:
        return None, False
    with _cov_lock:
        good, fresh = _cov["factor"] or None, _cov["good_sig"] == hit[0]
        if fresh or _cov["sig"] == hit[0] or _cov["busy"]:  # ready; failed; or one is being computed
            return good, fresh
        _cov["busy"], _cov["sig"] = True, hit[0]
    rounds = max(d.elo_hist) if d.elo_hist else None
    ratings = {n: r["elo"] for n, r in d.ratings.items()}
    threading.Thread(target=_cov_worker, args=(hit[0], d.league, ratings, rounds), daemon=True).start()
    return good, False


def _cov_worker(sig, league, ratings, rounds):
    try:
        f = fit_factor(league, ratings, rounds)
    except Exception:
        traceback.print_exc()
        f = None
    with _cov_lock:
        if f:
            _cov.update(factor=f, good_sig=sig, memo={})
        _cov["busy"] = False


def diff_se(d, a, b):
    """(se, fresh): the standard error of entries a's mean Elo minus entries b's from the fit's covariance
    (contrast_se); fresh False when it is an earlier fit's, today's being computed. (None, False) when there is none,
    or it lacks one of the entries (contrast_se would drop it)."""
    f, fresh = cov_factor(d)
    if not f or not a or not b or any(n not in f[1] and n != ANCHOR for n in list(a) + list(b)):
        return None, False
    key = (tuple(a), tuple(b))
    with _cov_lock:
        if _cov["factor"] is not f:
            return None, False
        hit = _cov["memo"].get(key)
    if hit is None:
        hit = contrast_se(f, a, b)
        with _cov_lock:
            if _cov["factor"] is f:
                _cov["memo"][key] = hit
    return hit, fresh


def entries_at(d, plies, lin, v, p):
    """The rated entries of a lineage's version at a ply."""
    return sorted(n for n in d.ratings if d.split_entry(n)[:2] == (lin, v) and plies.get(n, d.split_entry(n)[2]) == p)


def wake_gap(d, lin, v, plies, versions):
    """A dormant lineage's version against the bar it must clear to wake, in today's fit: (gap, half, bar version,
    how), gap its Elo minus the bar's (its best version up to the one it went dormant at, as cycle.py's
    peak_compare; before cycle.py kept that version, its all-time peak) and half requalify_se standard errors of the
    difference (from the fit's covariance, else the two ±s combined as if independent, as cycle.py does without it).
    None if either is unrated at the qualifying ply."""
    q, dv = d.qualify_ply, dormant_cap(d, lin)
    bar = earlier_peak(versions.get(lin), min(v, dv + 1) if dv is not None else v)
    now = (versions.get(lin) or {}).get(v)
    if bar is None or now is None:
        return None
    k, kw = num(d.cfg.get("requalify_se"), 2.0), se_words(d, "requalify_se")
    se, fresh = diff_se(d, entries_at(d, plies, lin, v, q), entries_at(d, plies, lin, bar[0], q))
    how = f"{kw}, from the {'' if fresh else 'previous '}fit's covariance"
    if se is None:
        se = math.hypot(now[1], bar[2]) / 1.96
        how = f"{kw}, the two ±s combined as if independent (the fit's covariance is being computed)"
    return now[0] - bar[1], k * se, bar[0], how


def gap_words(g, what):
    """A gap in words: "15 above v591", "23 below v591"."""
    return f"{abs(g):.0f} {'above' if g >= 0 else 'below'} {what}"


def prev_round(d):
    """(T, t, {player: Elo}): the round-robin today's fit (ratings.csv) ends with, the one before it, and each player's
    Elo in the fit after that one. From the league's Elo history, whose elo column is that same full fit after every
    round-robin (its running column, the classic running Elo, is not used). ratings.csv is written just before the
    history, so for a moment it can be a round-robin ahead of it (or, read before it, behind). T is None without a
    history; t is None before a second round-robin."""
    if getattr(d, "_prev_round", None) is not None:
        return d._prev_round
    H, res = d.elo_hist, (None, None, {})
    if H and d.ratings:
        def same(t):
            row = H.get(t) or {}
            common = [n for n in row if n in d.ratings]
            return bool(common) and all(abs(row[n] - d.ratings[n]["elo"]) < 0.051 for n in common)
        top = max(H)
        T = next((t for t in (top, top - 1) if same(t)), top + 1)
        res = (T, T - 1 if T - 1 in H else None, H.get(T - 1, {}))
    d._prev_round = res
    return res


def cycle_base(d):
    """The Change column's baseline: (c, k, n, B, tb, {player: Elo}). c: the cycle whose round-robins made today's
    fit (fit_cycles, as the peaks' "new"), k of its n round-robins in the fit; B: the cycle before it, tb its last
    round-robin, and each player's Elo in the fit after it (the league's Elo history, the full fit). c is None
    without a history; B and tb are None for the first cycle."""
    if getattr(d, "_cycle_base", None) is not None:
        return d._cycle_base
    res = (None, 0, d.rr, None, None, {})
    c, last = fit_cycles(d)
    if c is not None and last:
        earlier = [x for x in last if x < c]
        B = max(earlier) if earlier else None
        tb = last[B] if B is not None else None
        k = max(last.values()) - (tb or 0)
        res = (c, k, d.rr, B, tb, d.elo_hist.get(tb, {}) if tb is not None else {})
    d._cycle_base = res
    return res


def ordinal(n):
    return f"{n}{'th' if 10 <= n % 100 <= 20 else {1: 'st', 2: 'nd', 3: 'rd'}.get(n % 10, 'th')}"


def rr_words(d, t):
    """The league's round-robin t in words: "round-robin 64 (cycle 11's 4th)"."""
    cof = cycle_of_fn(d)
    c = cof(t - 1)
    return f"round-robin {t} (cycle {c}'s {ordinal(sum(1 for u in range(t) if cof(u) == c))})"


# a lineage's turn to dormant: "dormant (strike 2; ..." (older rows: "dormant (-43, strike 2; ..."); a dormant one's
# later rows say "dormant (+15 ± 80 vs peak ...; needs > +80)"
GOING_DORMANT = re.compile(r"dormant \((?:[^;()]*?,\s*)?strike \d")


def dormancy(d, lineage):
    """(cycle, version, n) for a dormant lineage: the cycle of its latest going-dormant decision (cycles.csv's, then
    the cycle under way's), the version that went dormant then, and the cycles it has spent dormant since, counting
    the one under way (between cycles, the last one). Without such a decision: the first of its latest run of dormant
    rows, as cycle.py's dormancy_versions. None if neither."""
    rows = {int(num(r["cycle"])): r for r in d.cycles_rows
            if r.get("lineage") == lineage and num(r.get("cycle")) is not None}
    cur = d.cur or {}
    cc = cur.get("cycle") if isinstance(cur.get("cycle"), int) else None
    if cc is not None and isinstance((cur.get("rows") or {}).get(lineage), dict):
        rows[cc] = cur["rows"][lineage]
    went = [c for c, r in rows.items() if GOING_DORMANT.match(str(r.get("decision") or ""))]
    c0 = max(went) if went else None
    if c0 is None:
        for c in sorted(rows, reverse=True):
            if rows[c].get("status") != "dormant":
                break
            c0 = c
    if c0 is None:
        return None
    v = num(rows[c0].get("version"))
    now = cc if cc is not None else num(d.state.get("cycle"))
    return c0, None if v is None else int(v), None if now is None else int(now) - c0


# the freeze/retire rule (cycle.py, from cycle 15): config.json "freeze_rule"; state.json "milestones",
# "references" and, per lineage, "claim", "frozen_by", "retired_by", "rule" (once fired), "rule_preview" (dormant)
def rule_conf(d):
    """(dormant cycles in a row before the rule applies, milestone step, window, reference window, exempt lineages)."""
    c = (d.cfg or {}).get("freeze_rule")
    c = c if isinstance(c, dict) else {}
    return (int(num(c.get("after_dormant"), 10)), int(num(c.get("milestone_step"), 100)),
            float(num(c.get("milestone_window"), 10)), float(num(c.get("reference_window"), 50)),
            set(c.get("exempt") or []))


def is_retired(d, lin):
    """Retired: by hand (its line commented out of lineages.txt) or by the rule (state.json; its line stays)."""
    st = (d.state.get("lineages") or {}).get(lin)
    return bool((d.lin.get(lin) or {}).get("former")) or (isinstance(st, dict) and st.get("status") == "retired")


def nearest_ms(x, step):
    """The multiple of the step nearest x, halves up (cycle.py's nearest_milestone)."""
    return int(math.floor(x / step + 0.5)) * step


def milestones(d, derive=False):
    """The milestones taken, {m: [(who, how)]}: state.json's "milestones" (cycle.py's, as of its last decision), else
    (or with derive) derived as cycle.py does: the multiple nearest each reference player (state.json's references)
    within the reference window in today's fit, then each frozen lineage's claim."""
    N, step, win, rwin, _ = rule_conf(d)
    refs = list(d.state.get("references") or []) or sorted(n for n, _, _ in d.players if d.family(n) == "random")
    lins = d.state.get("lineages") or {}

    def how(who):
        if who in refs:
            return f"reference at {plain(d.ratings[who]['elo'])}" if who in d.ratings else "reference"
        r = (lins.get(d.split_entry(who)[0]) or {}).get("rule")
        return "frozen" + (f" after cycle {r['cycle']}" if isinstance(r, dict) and r.get("cycle") is not None else "")
    out = {}
    stored = d.state.get("milestones")
    if isinstance(stored, dict) and stored and not derive:
        for m, v in stored.items():
            for who in str(v).split(", "):
                st = lins.get(d.split_entry(who)[0]) or {}
                if who not in refs and (st.get("status") != "frozen" or num(st.get("claim")) != num(m)):
                    continue  # unfrozen since: the claim is free again (state.json keeps it until the next decision)
                out.setdefault(int(num(m)), []).append((who, how(who)))
        return out
    for n in refs:
        if n in d.ratings:
            e = d.ratings[n]["elo"]
            m = nearest_ms(e, step)
            if abs(e - m) <= rwin:
                out.setdefault(m, []).append((n, how(n)))
    for lin, st in sorted(lins.items()):
        if isinstance(st, dict) and st.get("status") == "frozen" and num(st.get("claim")) is not None:
            who = (st.get("entries") or [lin])[0]
            out.setdefault(int(num(st["claim"])), []).append((who, how(who)))
    return out


def cycles_span(cyc, k=None):
    """("cycles 6–10", 5) for the cycles averaged: every cycle, or [first, last] as cycle.py stores them."""
    cyc = sorted(int(c) for c in cyc or [])
    if not cyc:
        return "its dormant cycles", k or 0
    k = k if k is not None else len(cyc) if len(cyc) != 2 else cyc[1] - cyc[0] + 1
    return (f"cycle {cyc[0]}" if cyc[0] == cyc[-1] else f"cycles {cyc[0]}–{cyc[-1]}"), k


def rr_end(d, c):
    """The league's round-robin count at the end of cycle c's tournament: state.json's round_robin_ends (recorded at
    each decision from the rule's first), else 5, 10 and 20 for cycles 1-3 and 5c+10 from cycle 4 (cycle.py's)."""
    got = (d.state.get("round_robin_ends") or {}).get(str(c))
    return int(got) if got is not None else {1: 5, 2: 10, 3: 20}.get(c, 5 * c + 10)


def rule_previews(d):
    """cycle.py's rule survey, for the preview when state.json has none, as of the last decision (the cycle under
    way's once made, else state.json's cycle): {lineage: (n, applies_after, {ply: average}, [cycles], would)} for the
    dormant lineages. Averages: its entries' Elo per ply over the last after_dormant cycles of its dormant spell, each
    as measured then (elo_history.csv at the cycle's last round-robin, for that cycle's cycles.csv entries; the decided
    cycle under way's, today's fit). Resolved as cycle.py does: the lineages due now first, then by when they come
    due, closest first within each (a tie: ply 2, then the name), each claim taking its milestone; exempt and
    done-this-spell lineages claim nothing."""
    if getattr(d, "_rule_prev", None) is not None:
        return d._rule_prev
    N, step, win, rwin, exempt = rule_conf(d)
    cur = d.cur or {}
    decided = cur.get("phase") in ("own", "log") and isinstance(cur.get("cycle"), int)
    C = cur["cycle"] if decided else int(num(d.state.get("cycle"), 0))
    rows = {}
    for r in d.cycles_rows:
        c = num(r.get("cycle"))
        if c is not None and int(c) <= C:
            rows.setdefault(r.get("lineage"), {})[int(c)] = r
    if decided:
        for lin, r in (cur.get("rows") or {}).items():
            if isinstance(r, dict):
                rows.setdefault(lin, {})[C] = r
    taken, info = set(milestones(d, derive=True)), {}
    for lin, st in sorted((d.state.get("lineages") or {}).items()):
        if not isinstance(st, dict) or st.get("status") != "dormant":
            continue
        rl = rows.get(lin, {})
        went = [c for c, r in rl.items() if GOING_DORMANT.match(str(r.get("decision") or ""))]
        c0 = max(went) if went else None
        if c0 is None:
            for c in sorted(rl, reverse=True):
                if rl[c].get("status") != "dormant":
                    break
                c0 = c
        if c0 is None:
            continue
        span, got = list(range(max(c0 + 1, C - N + 1), C + 1)), {}
        for c in span:
            elo = ({n: x["elo"] for n, x in d.ratings.items()} if decided and c == C else
                   d.elo_hist.get(rr_end(d, c), {}))
            for e in str((rl.get(c) or {}).get("entries") or "").split():
                if e in elo:
                    got.setdefault(int(e.rsplit("-", 1)[1]), []).append(elo[e])
        r = st.get("rule") if isinstance(st.get("rule"), dict) else {}
        ex, done = lin in exempt, r.get("dormant_since") == c0
        info[lin] = dict(n=C - c0, after=c0 + N, span=span, avg={p: sum(v) / len(v) for p, v in sorted(got.items())},
                         exempt=ex, done=done, fire=C - c0 >= N and not ex and not done)
    waiting = [l for l, i in info.items() if not (i["fire"] or i["exempt"] or i["done"])]
    groups = ([[l for l, i in info.items() if i["fire"]]]
              + [[l for l in waiting if info[l]["after"] == a] for a in sorted({info[l]["after"] for l in waiting})])
    out = {}
    for group in groups:
        opts = []
        for lin in sorted(group):
            out[lin] = None
            for p, a in info[lin]["avg"].items():
                m = nearest_ms(a, step)
                if abs(a - m) <= win:
                    opts.append((abs(a - m), -p, lin, p, m))
        for _, _, lin, p, m in sorted(opts):
            if out[lin] is None and m not in taken:
                out[lin] = (p, m)
                taken.add(m)
    res = {}
    for lin, i in info.items():
        o = out.get(lin)
        res[lin] = (i["n"], i["after"], i["avg"], i["span"],
                    "exempt" if i["exempt"] else "done this dormant spell" if i["done"] else
                    f"freeze {o[0]} at {o[1]}" if o else "retire")
    d._rule_prev = res
    return res


def rule_avg_words(avg, taken, step):
    """ "ply 2: 1752.1, 47.9 from 1800; ply 1: 812.0, 12.0 from 800 (taken by random-4)" """
    out = []
    for p, a in sorted(avg.items(), key=lambda x: -int(x[0])):
        a = float(a)
        m = nearest_ms(a, step)
        by = taken.get(m)
        out.append(f"ply {p}: {plain(a, 1)}, {abs(a - m):.1f} from {m}"
                   + (f" (taken by {', '.join(w for w, _ in by)})" if by else ""))
    return "; ".join(out)


def act_words(act, two_plies=True):
    """cycle.py's "freeze 2 at 1500" / "retire" (or a computed (act, ply, m)), in words."""
    if isinstance(act, (tuple, list)):
        act = f"freeze {act[1]} at {act[2]}" if act[0] == "freeze" else "retire"
    m = re.fullmatch(r"freeze (\d+) at (\d+)", str(act or "").strip())
    if m:
        return f"freeze ply {m.group(1)}, claiming {m.group(2)}" + (", and retire the other ply" if two_plies else "")
    return "retire the lineage" if str(act).strip() == "retire" else str(act)


def rule_lines(d, lin, n, c0):
    """The dormant hover's lines on the freeze/retire rule: the countdown, each ply's average of its end-of-cycle
    ratings over its dormant cycles so far, and what the rule would do if it applied now (cycle.py's preview, else
    computed the same way)."""
    N, step, win, rwin, exempt = rule_conf(d)
    st = (d.state.get("lineages") or {}).get(lin) or {}
    pv = st.get("rule_preview") if isinstance(st.get("rule_preview"), dict) else None
    if lin in exempt or (pv or {}).get("exempt"):
        return [f"Exempt from the freeze/retire rule (config.json freeze_rule.exempt)."]
    r = st.get("rule")
    if isinstance(r, dict) and r.get("dormant_since") == c0:
        return [f"The freeze/retire rule fired in this dormant spell (cycle {r.get('cycle')}) and was undone by hand: "
                f"it won't fire again until the lineage wakes."]
    after = int(num((pv or {}).get("applies_after"), c0 + N))
    lines = [f"dormant {n} of {N}: the freeze/retire rule applies after cycle {after}" if n < N else
             f"dormant {n} of {N}: the freeze/retire rule applies at this cycle's decision (cycle {after})" if n == N
             else f"dormant {n}, past the rule's {N}: it applies at the next decision"]
    taken = {m: ws for m, ws in milestones(d).items()}
    if pv:
        avg = {int(p): float(a) for p, a in (pv.get("avg") or {}).items()}
        span, k = cycles_span(pv.get("cycles"))
        would, src = pv.get("would"), ", cycle.py's preview"
    else:
        x = rule_previews(d).get(lin)
        avg, (span, k) = (x[2], cycles_span(x[3], len(x[3]))) if x else ({}, ("", 0))
        would, src = (x[4] if x else None), ""
    if would in ("exempt", "done this dormant spell"):
        would = None
    if not avg:
        lines.append("No dormant cycle has ended yet: no averages so far.")
        return lines
    lines.append(f"Averages of its end-of-cycle ratings so far ({span}, {k} of {N}{src}): "
                 + rule_avg_words(avg, taken, step) + ".")
    if would is not None:
        lines.append(f"If the rule applied now, it would {act_words(would, len(avg) > 1)}.")
    return lines


def rule_note(d, lin):
    """The rule's decision for a lineage it froze or retired ("Frozen by the freeze/retire rule after cycle 15, ...:
    it claimed 800. Averages ..."), else None: frozen or retired by hand."""
    N, step, win, rwin, _ = rule_conf(d)
    st = (d.state.get("lineages") or {}).get(lin) or {}
    r = st.get("rule")
    frozen = st.get("status") == "frozen" and st.get("frozen_by") == "rule"
    if not isinstance(r, dict) or not (frozen or (st.get("status") == "retired" and st.get("retired_by") == "rule")):
        return None
    span, k = cycles_span(r.get("cycles"))
    when = (f" after cycle {r['cycle']}" if r.get("cycle") is not None else "") + f", its {k or N} dormant cycles in a row ({span})"
    taken = {m: [x for x in ws if not x[0].startswith(lin + "-v")] for m, ws in milestones(d).items()}
    av = rule_avg_words({int(p): float(a) for p, a in (r.get("avg") or {}).items()},
                        {m: ws for m, ws in taken.items() if ws}, step)
    if frozen:
        claim = num(st.get("claim"))
        return (f"Frozen by the freeze/retire rule{when}" + (f": it claimed {int(claim)}" if claim is not None else "")
                + f". Averages of its end-of-cycle ratings: {av}.")
    return (f"retired by the freeze/retire rule{when}: no ply's average was within {win:g} of an untaken multiple "
            f"of {step}. Averages of its end-of-cycle ratings: {av}.")


def frozen_label(d, lin):
    """ "frozen (800)" with the milestone the rule claimed for it, else "frozen"."""
    c = num(((d.state.get("lineages") or {}).get(lin) or {}).get("claim"))
    return f"frozen ({int(c)})" if c is not None else "frozen"


def milestone_note(d):
    """The line under Standings: the milestones taken, and by whom."""
    ms = milestones(d)
    if not ms:
        return ""
    N, step, win, rwin, _ = rule_conf(d)
    items = [f"<b>{m}</b> " + ", ".join(f"{esc(w)} <span class='muted'>({esc(how)})</span>" for w, how in ws)
             for m, ws in sorted(ms.items())]
    tip = (f"The freeze/retire rule: a lineage dormant {N} cycles in a row averages each ply's end-of-cycle ratings "
           f"over them. A ply within {win:g} of an untaken multiple of {step} freezes there, claiming it (the closer "
           f"ply if both qualify, ply 2 on a tie), and the other ply retires; if neither qualifies, both retire. "
           f"Taken: a frozen lineage's claim, or the multiple nearest a reference player within {rwin:g}."
           + (" As of cycle.py's last decision, less any claim freed since by an unfreeze."
              if isinstance(d.state.get("milestones"), dict) and
              d.state.get("milestones") else " Derived from today's fit (cycle.py has not run the rule yet)."))
    return f'<p class="note ms" data-tip="{esc(tip)}">Milestones taken: ' + " · ".join(items) + '.</p>'


def leaving(d):
    """The entries the freeze/retire rule took out at this cycle's decision (cycle.py's current.rule_retire): the own
    phase moves them from players.txt to retired.txt, before the next round-robin, so they are out already."""
    return set((d.cur or {}).get("rule_retire") or [])


def field_names(d):
    """The field: players.txt's entries, less those the rule is about to move out."""
    return {n for n, _, _ in d.players} - leaving(d)


def gone_entries(d):
    """Standings rows for entries out of the field: a retired lineage's last entries (state.json's entries, which
    cycle.py's retire keeps, else its last cycles.csv row's) and a frozen lineage's entries retired at the freeze
    (frozen_retired). {name: (status, hover)}."""
    infield, out = field_names(d), {}
    for lin, st in (d.state.get("lineages") or {}).items():
        if not isinstance(st, dict):
            continue
        if st.get("status") == "retired" or (d.lin.get(lin) or {}).get("former"):
            ents = list(st.get("entries") or [])
            if not ents:
                rows = [r for r in d.cycles_rows if r.get("lineage") == lin and str(r.get("entries") or "").strip()]
                ents = str(max(rows, key=lambda r: num(r.get("cycle"), -1))["entries"]).split() if rows else []
            m = re.search(r"(\d+)$", status_of(d, lin))
            c = int(m.group(1)) if m else None
            for n in ents:
                if n not in infield:
                    rn = rule_note(d, lin)
                    out[n] = (f"retired (c{c})" if c is not None else "retired",
                              (f"{lin}: {rn} It plays no more" if rn else
                               f"{lin}: retired by hand" + (f" after cycle {c}" if c is not None else "")
                               + "; it plays no more")
                              + ". Its games stay in the fit, so its Elo can still drift with refits.")
        elif st.get("status") == "frozen":
            since, kept = frozen_since(d, lin), ", ".join(st.get("entries") or []) or "one entry"
            for n in st.get("frozen_retired") or []:
                if n not in infield:
                    rn = rule_note(d, lin)
                    out[n] = ("retired (frozen)",
                              f"{lin} is frozen" + (f" since cycle {since}" if since is not None else "")
                              + f": it keeps {kept} in the field, and {n} was retired at the freeze; it plays no "
                                "more. Its games stay in the fit, so its Elo can still drift with refits; unfreezing "
                                "brings it back." + ("\n" + rn if rn else ""))
    return out


STATUS_ORDER = ["qualified", "dormant", "frozen", "retired", "reference"]  # the filter's order
STATUS_ALWAYS = {"qualified", "dormant", "reference"}  # the others show once a row has them


def status_key(status):
    """A Standings row's status for the filter: its first word (qualified with or without strikes, dormant (7),
    frozen, retired, reference), else "other"."""
    m = re.match(r"[a-z]+", status or "")
    return m.group(0) if m else "other"


STD_FX_HEAD = {"arch": "arch", "leaf": "leaf", "router": "router", "split": "split", "depth": "self-play depth",
               "labels": "labels", "policy": "move selection"}  # the Standings' factor columns' headings, short


def std_fx_head():
    """The Standings' factor columns' headings, a column per factor; the hover names the factor."""
    return "".join(f'<th class="fxh" data-fxk="{k}" data-tip="{esc(label)}: the lineage\'s value (the Lineages table '
                   f'has them all, with the variant); the bar at the top filters by it">{esc(STD_FX_HEAD.get(k, label))}'
                   '</th>' for k, label, *_ in FACTORS)


def std_fx_cells(d, name, ref):
    """A Standings row's factor cells: its lineage's value of each factor, the letter alone where the value has a name
    (hover for it); empty for a random player."""
    e = None if ref else fx(d).get(lin_of(d, name))
    out = []
    for k, *_ in FACTORS:
        if not e:
            out.append(f'<td class="fxv" data-fxk="{k}"></td>')
            continue
        v = e[0][k]
        shown, lab = ("?" if v == UNCL else v), fx_label(k, v)
        grey = v == UNCL or v in FX_GREY.get(k, ())
        out.append(f'<td class="fxv{" muted" if grey else ""}" data-fxk="{k}"'
                   + (f' data-tip="{esc(lab)}"' if lab != shown else "") + f'>{esc(shown)}</td>')
    return "".join(out)


def status_filter(groups):
    """The Standings filter: a checkbox per status with its count. groups: {status: (set of lineages, rows)}."""
    keys = ([k for k in STATUS_ORDER if k in STATUS_ALWAYS or k in groups]
            + sorted(k for k in groups if k not in STATUS_ORDER))
    out = []
    for k in keys:
        lins, nrows = groups.get(k, (set(), 0))
        n = len(lins)
        what = (f"{n} random player{'s' if n != 1 else ''}" if k == "reference" else
                f"{n} lineage{'s' if n != 1 else ''}, {nrows} row{'s' if nrows != 1 else ''}")
        out.append(f'<label data-tip="{esc(what)}: tick to show only these; none ticked shows all; the ranks stay '
                   f'as they are"><input type="checkbox" data-st="{esc(k)}"{"" if nrows else " disabled"}> {esc(k)} '
                   f'({n})</label>')
    total = sum(g[1] for g in groups.values())
    return ('<div class="stf" role="group" aria-label="Filter the rows by status"><span class="muted">Filter by '
            'status:</span>' + "".join(out) + f'<span class="muted stf-n">showing all {total} rows</span></div>')


def panel_standings(d):
    infield = field_names(d)
    active = [(n, ply) for n, ply, _ in d.players if n in infield]
    if not d.ratings:
        return ('<p class="note">No ratings yet: the league has played no games. The first ratings appear when '
                'cycle 1\'s tournament finishes.</p>' + fx_legend(d, refs=True))
    gone = {n: g for n, g in gone_entries(d).items() if n in d.ratings}  # muted rows, no rank
    rated = sorted([(n, ply) for n, ply in active if n in d.ratings] + [(n, d.split_entry(n)[2]) for n in gone],
                   key=lambda x: -d.ratings[x[0]]["elo"])
    unrated = [n for n, ply in active if n not in d.ratings]
    if not rated:
        return '<p class="note">None of the current players has a rating yet.</p>'
    lo = min(d.ratings[n]["elo"] - d.ratings[n]["ci"] for n, _ in rated)
    hi = max(d.ratings[n]["elo"] + d.ratings[n]["ci"] for n, _ in rated)
    pad = 0.04 * (hi - lo or 100)
    lo, hi = lo - pad, hi + pad
    W, H = 300, 18
    sx = lambda e: 6 + (W - 12) * (e - lo) / (hi - lo)
    ticks = nice_ticks(lo, hi, 5)
    axis = "".join(f'<text x="{sx(t):.1f}" y="12" text-anchor="middle">{plain(t)}</text>' for t in ticks)
    grid = "".join(f'<line x1="{sx(t):.1f}" x2="{sx(t):.1f}" y1="0" y2="{H}" class="grid"/>' for t in ticks)
    q, plies = d.qualify_ply, entry_plies(d)
    versions = ply_versions(d, plies)
    logged = logged_peaks(d, plies, versions)
    pts, ppts = cycle_points(d), peak_points(d)
    fit_c = fit_cycles(d)[0]
    cur_ = d.cur or {}
    last_done = (cur_["cycle"] if isinstance(cur_.get("cycle"), int) and cur_.get("phase") in ("own", "log") else
                 max((int(num(r.get("cycle"), -1)) for r in d.cycles_rows), default=None))  # the latest completed cycle
    dorm_rule = any(isinstance(s, dict) and "dormant_version" in s for s in (d.state.get("lineages") or {}).values())
    rse_w = se_words(d, "requalify_se")
    note = (PEAK_NOTE_DORMANT if dorm_rule else PEAK_NOTE).format(ply=q, dse=se_words(d, "dormancy_se"), rse=rse_w)
    chg_note = ("Change over the cycle, per lineage and ply: the row's Elo now minus the lineage's rating at that ply "
                "at the end of the previous cycle's tournament, whatever version it was then (the last segment of its "
                "line on Elo over cycles). A version carried over shows its own drift; a new one, its change from the "
                "version it replaced. While a cycle's round-robins run, the change so far in it; during the next "
                "cycle's self-play, that cycle's whole change. new: the lineage's first cycle at that ply. Hover a "
                "change for both ratings and the version's story.")
    T = prev_round(d)[0]
    fc, k_rr, n_rr, B, tb, base_h = cycle_base(d)
    partial = fc is not None and k_rr < n_rr and fc == (d.cur or {}).get("cycle")
    gain_note = (f"Ply gain: how much the model gains from the extra ply, its ply-{q} entry's Elo minus its "
                 f"ply-{q - 1} entry's, for the same version, in today's fit; ± is the 95% range of that difference "
                 f"from the fit's covariance (both entries play the same tournaments, so it is tighter than their "
                 f"two ±s combined). Click to sort by it.")
    out = [fx_legend(d, refs=True),
           '<div class="tscroll" id="standings-wrap" data-sk="left"><table class="standings"><tr><th class="r">#</th>'
           '<th>entry</th><th class="r">ply</th><th>lineage</th>'
           '<th class="r">version</th><th class="r">games</th><th>status</th>'
           f'<th class="r hlp" data-tip="{esc(chg_note)}">change</th>'
           '<th class="r" data-sortcol="elo"><button type="button" class="sortb" data-sort="elo" '
           'data-tip="Sort by Elo (the default order)">Elo<span class="sarr"></span></button></th>'
           f'<th class="r">±95%</th><th class="r pk" data-tip="{esc(note)}">Peak Elo</th>'
           f'<th class="r pk" data-tip="{esc(note)}">Peak version</th>'
           f'<th class="r" data-sortcol="gain"><button type="button" class="sortb" data-sort="gain" '
           f'data-tip="{esc(gain_note)}">Ply gain<span class="sarr"></span></button></th>'
           f'<th><svg width="{W}" height="16" class="axis">{axis}</svg></th>'
           + std_fx_head() +
           "</tr>"]
    gain_how, groups, frozen_seen = set(), {}, set()
    rank = j = 0
    for i, (name, ply) in enumerate(rated, 1):
        r = d.ratings[name]
        if name in gone:  # no rank: sorted after the active entry above it
            j += 1
            rank_s, rank_v = "–", f"{rank}.{j:02d}"
        else:
            rank, j = rank + 1, 0
            rank_s = rank_v = str(rank)
        fam = shape_of(d, name)
        lin, v, p = d.split_entry(name)
        ref = fam == "random"
        new, ever, story, status_tip, shown = True, [], "", "", ""
        if ref:
            lin_s, v_s, status = "—", "—", "reference"
            if name == ANCHOR:
                story = "The anchor: pinned at 0 in every fit."
        else:
            lin_s, v_s, status = lin, f"v{v}", status_of(d, lin)
            ever = sorted({pt["c"] for pp in pts.get(lin, {}).values() for pt in pp if pt["v"] == v})
            earlier = [c for c in ever if fit_c is not None and c < fit_c]
            new = not earlier
            bpt = next((pt for pt in pts.get(lin, {}).get(ply, []) if pt["c"] == B), None) if B is not None else None
            if earlier:  # the version's story, for the change's hover
                story = f"v{v}: the same model since cycle {ever[0]}, re-rated as the league plays on."
            elif bpt and bpt["v"] != v:
                story = (f"v{v} replaced v{bpt['v']} "
                         + ("this cycle" if fc == (d.cur or {}).get("cycle") else f"in cycle {fc}") + "."
                         + (f" In today's fit it is {gap_words(r['elo'] - bpt['today'], f'v' + str(bpt['v']))} "
                            f"({plain(bpt['today'])})." if bpt.get("today") is not None else ""))
            else:
                pv = predecessor(d, lin, v, p)
                story = (f"v{v} is new" + (f" in cycle {fit_c}" if fit_c is not None else "") + ": "
                         + (f"{gap_words(r['elo'] - pv[1]['elo'], f'v{pv[0]}')}, the lineage's previous version at "
                            f"ply {p} ({plain(pv[1]['elo'], 1)} in today's fit)." if pv else
                            "the lineage's first version."))
            if name in gone:  # a retired lineage's entry, or a frozen lineage's retired one
                status, status_tip = gone[name]
                story = f"{name}: retired, so the change is its own drift."
            if status == "dormant":
                dz = dormancy(d, lin)
                if dz:
                    c0, v0, n = dz
                    shown = "dormant" if n is None else f"dormant ({n})"
                    lines = [f"dormant since cycle {c0}" + ("" if v0 is None else f" (at v{v0})")]
                    if n == 0:
                        lines.append("It went dormant at this cycle's decision: no cycle spent dormant yet.")
                    elif n:
                        lines.append(f"{n} cycle{'s' if n != 1 else ''} spent dormant: "
                                     + (f"cycle {c0 + 1}" if n == 1 else f"cycles {c0 + 1}–{c0 + n}") + ".")
                    if n is not None:
                        lines += rule_lines(d, lin, n, c0)
                    status_tip = "\n".join(lines)
        # the baseline: the lineage's point at this ply at the end of cycle B, whatever version; a random player's own
        own = ref or name in gone  # random players and retired entries: their own drift
        if own:
            e0, bv = base_h.get(name), None
        else:
            e0, bv = (bpt["elo"], bpt["v"]) if bpt else (None, None)
        who = name if ref else f"v{v}"
        head = f"{who}: {plain(r['elo'], 1)} now" + (f", {k_rr} of {n_rr} round-robins into cycle {fc}" if partial else "")
        if fc is None:
            delta, chg_tip = "", story
        elif e0 is None and name in gone:  # the user: a retired row isn't new; it plays no more games, so 0
            delta = "0"
            chg_tip = (head + "; retired, so it plays no more games"
                       + ("" if B is None else
                          f"; it has no rating of its own at the end of cycle {B}"
                          + (f" (it first played in cycle {ever[0]})" if ever else "") + ", so no drift to show")
                       + ": 0.")
        elif e0 is None:
            delta = "new"
            chg_tip = (head + ("; the league's first cycle" if B is None else
                               f"; not rated at the end of cycle {B}" if own else
                               f"; the lineage has no rating at ply {ply} at the end of cycle {B}: its first cycle")
                       + ".")
        else:
            dv = r["elo"] - e0
            ch = "no change" if round(dv, 1) == 0 else ("▲ " if dv > 0 else "▼ ") + signed(dv, 1)
            delta = ("0" if round(dv) == 0 else
                     ('<span class="ok">▲</span> ' if dv > 0 else '<span class="bad">▼</span> ') + signed(dv))
            chg_tip = head + (f"; {plain(e0, 1)} at the end of cycle {B} → {ch}" if own else
                              f"; the lineage at the end of cycle {B}: v{bv} at {plain(e0, 1)} → {ch}")
        if story and fc is not None:
            chg_tip += "\n" + story
        chg_attr = f' data-tip="{esc(chg_tip)}"' if chg_tip else ""
        x0, x1, xm = sx(r["elo"] - r["ci"]), sx(r["elo"] + r["ci"]), sx(r["elo"])
        tip = f"{name}: Elo {plain(r['elo'])} ± {r['ci']:.0f} (95%), {r['games']} games"
        svg = (f'<svg width="{W}" height="{H}" data-tip="{esc(tip)}">{grid}'
               f'<line x1="{x0:.1f}" x2="{x1:.1f}" y1="{H / 2}" y2="{H / 2}" stroke="{fam_colour(fam)}" '
               f'stroke-width="2" stroke-linecap="round" opacity="0.55"/>{marker(fam, xm, H / 2)}</svg>')
        cls = ' class="ref"' if ref else ' class="ret"' if name in gone else ""
        peak, gain, attrs = "<td></td><td></td>", "<td></td>", f' data-rank="{rank_v}"'
        series = (ppts.get(lin, {}).get(ply) or []) if not ref and v is not None else []
        top = max(series, key=lambda pt: pt["elo"]) if series else None  # the peak: completed cycles only
        dz0 = dormancy(d, lin) if not ref and v is not None and status == "dormant" else None
        c0 = dz0[0] if dz0 else None  # the cycle whose decision made it dormant: the parentheses' last
        before = [pt for pt in series if c0 is not None and pt["c"] <= c0]
        bpk = max(before, key=lambda pt: pt["elo"]) if before else None
        par = ([] if c0 is None else
               [f"In parentheses, its peak at ply {ply} from before it went dormant (cycles up to {c0}, whose "
                f"decision made it dormant): " + (f"{plain(bpk['elo'], 1)}, v{bpk['v']} as measured in cycle "
                                                  f"{bpk['c']}." if bpk else "none, no completed cycle at this ply.")])
        pe_s = (f' <span class="pkd">({plain(bpk["elo"])})</span>' if bpk else
                ' <span class="pkd">(—)</span>' if c0 is not None else "")
        pv_s = (f' <span class="pkd">(v{bpk["v"]})</span>' if bpk else
                ' <span class="pkd">(—)</span>' if c0 is not None else "")
        tag = ('<span class="npk">new peak</span> ' if top is not None and top["c"] == last_done
               and name not in gone and any(pt["c"] < top["c"] for pt in series) else "")  # not its first
        if top is not None and ply != q:
            t = esc("\n".join([PEAK_HEAD.strip(), ""] + peak_lines(lin, ply, top) + par))
            peak = (f'<td class="r" data-tip="{t}">{plain(top["elo"])}{pe_s}</td>'
                    f'<td class="r" data-tip="{t}">{tag}v{top["v"]}{pv_s}</td>')
        elif not ref and v is not None and ply != q:  # its first cycle is under way: nothing completed yet
            t = esc(f"{PEAK_HEAD.strip()}\n\n{lin}: no completed cycle at ply {ply} yet")
            peak = f'<td data-tip="{t}"></td><td data-tip="{t}"></td>'
        if not ref and v is not None and ply == q:
            pk, lg = earlier_peak(versions.get(lin), v), logged.get(lin)  # pk: the best earlier version, the rule's
            dv = dormant_cap(d, lin)
            pre = earlier_peak(versions.get(lin), min(v, dv + 1)) if dv is not None else None  # the rule's wake bar
            wake = wake_gap(d, lin, v, plies, versions) if status == "dormant" else None
            wake_s = ""
            if wake:
                g, half, bv, how = wake
                wake_s = (f"The rule's wake bar: {gap_words(g, f'v{bv}')}, its best version"
                          + (" from before dormancy" if dv is not None else "")
                          + f" in today's fit, ± {half:.0f} ({how}); it needs more than {half:.0f} above to wake.")
                status_tip = (status_tip + "\n" if status_tip else f"{lin} is dormant. ") + wake_s
            if top is None:
                t = esc(f"{note}\n\n{lin}: no completed cycle at ply {q} yet" + (f"\n\n{wake_s}" if wake_s else ""))
                peak = f'<td data-tip="{t}"></td><td data-tip="{t}"></td>'
            else:
                gap = r["elo"] - pk[1] if pk else None
                tip = [note, ""] + ([wake_s, ""] if wake_s else []) + peak_lines(lin, q, top) + par + [
                    (f"The rule's comparison, the best earlier version in today's fit: v{pk[0]} at "
                     f"{plain(pk[1], 1)} ± {pk[2]:.0f}; this entry (v{v}) at {plain(r['elo'], 1)}: "
                     f"{gap_words(gap, 'it')}.") if pk else
                    f"The rule's comparison: no earlier version rated at ply {q} in today's fit."]
                if status == "frozen":
                    tip[-1] += " Not applied while frozen: no qualification decision."
                elif name in gone:
                    tip[-1] += " Not applied: retired, it plays no more."
                if dv is not None:
                    ksd = f"{float(num(d.cfg.get('requalify_se'), 2)):g}"
                    upto = (f"up to v{dv}, the version that went dormant" if v > dv else
                            f"before v{dv}, the version that went dormant (this entry)")
                    tip += ["", f"The rule's wake bar: to wake, it must beat this by more than {ksd} SE. It is the "
                                f"lineage's best version {upto}, in today's fit, retired ones included; versions its "
                                f"own steps made since don't count."]
                    tip.append(f"the rule's wake bar: v{pre[0]} at {plain(pre[1], 1)} ± {pre[2]:.0f} (95%); "
                               f"this entry: {gap_words(r['elo'] - pre[1], 'it')}" if pre else
                               f"the rule's wake bar: no version {upto.split(',')[0]} rated at ply {q}")
                flag = ""
                if lg:
                    lv = lambda v_, e_: ((f"v{v_}" if v_ is not None else "no version") +
                                         (f" at {plain(e_, 1)}" if e_ is not None else ""))
                    at = lv(lg["version"], lg["elo"]) + (f"; the rule's wake bar {lv(*lg['pre'])}" if lg["pre"] else "")
                    head = (f"{lg['src']}, cycle {lg['cycle']}: its "
                            f"{'all-time peak (frozen)' if lg.get('frozen') else 'best earlier version'} {at}")
                    if not lg["same"]:
                        tip.append(f"{head}, from that cycle's fit")
                    elif lg["agrees"]:
                        tip.append(f"{head}, from this same fit: agrees")
                    else:
                        tip.append(f"{head}, from this same fit: DIFFERS")
                        flag = ' <span class="warn">≠</span>'
                t = esc("\n".join(tip))
                peak = (f'<td class="r" data-tip="{t}">{plain(top["elo"])}{pe_s}</td>'
                        f'<td class="r" data-tip="{t}">{tag}v{top["v"]}{pv_s}{flag}</td>')
            hi_n, lo_n = entries_at(d, plies, lin, v, q), entries_at(d, plies, lin, v, q - 1)
            if hi_n and lo_n:
                mean = lambda ns, k: sum(d.ratings[n][k] for n in ns) / len(ns)
                e2, c2, e1, c1 = mean(hi_n, "elo"), mean(hi_n, "ci"), mean(lo_n, "elo"), mean(lo_n, "ci")
                g, hyp = e2 - e1, math.hypot(c2, c1)
                se, fresh = diff_se(d, hi_n, lo_n)
                if se is not None:
                    half, how = 1.96 * se, (("from the fit's covariance" if fresh else
                                             "from the previous fit's covariance, today's being computed")
                                            + f"; the two ±s combined as if independent would give ± {hyp:.0f}")
                    gain_how.add("cov" if fresh else "prev")
                else:
                    half, how = hyp, "the two ±s combined as if independent: the fit's covariance is being computed"
                    gain_how.add("hypot")
                ent_s = lambda ns: ", ".join(n + ("" if n in infield else ", retired") for n in ns)
                t = (f"{lin} v{v}, today's fit:\nply {q}: {plain(e2, 1)} ± {c2:.0f} ({ent_s(hi_n)})\n"
                     f"ply {q - 1}: {plain(e1, 1)} ± {c1:.0f} ({ent_s(lo_n)})\n"
                     f"ply gain {plain(g, 1)} ± {half:.1f} (95%, {how})")
                gain = (f'<td class="r" data-tip="{esc(t)}">{plain(g)} <span class="muted">± {half:.0f}</span>'
                        f'</td>')
                attrs += f' data-gain="{g:.1f}"'
        if not ref and status == "frozen":
            rn = rule_note(d, lin)
            status_h = (f'<span class="frz" data-tip="{esc(frozen_note(d, lin) + (chr(10) + rn if rn else ""))}">'
                        f'{esc(frozen_label(d, lin))}</span>')
            frozen_seen.add(lin)
        else:
            status_h = (f'<span class="hlp" data-tip="{esc(status_tip)}">{esc(shown or status)}</span>' if status_tip
                        else esc(shown or status))
        key = status_key(status)
        g = groups.setdefault(key, [set(), 0])
        g[0].add(name if ref else lin)
        g[1] += 1
        attrs += f' data-st="{esc(key)}" data-ply="{ply}"' + ("" if ref else lc_attr(d, name))
        out.append(f"<tr{cls}{attrs}><td class='r'>{rank_s}</td><td>{swatch(fam)} {esc(name)}</td><td class='r'>{ply}</td>"
                   f"<td>{esc(lin_s)}</td><td class='r'>{esc(v_s)}</td><td class='r'>{r['games']:,}</td>"
                   f"<td>{status_h}</td><td class='r'{chg_attr}>{delta}</td><td class='r'>{plain(r['elo'])}</td>"
                   f"<td class='r'>{r['ci']:.0f}</td>{peak}{gain}<td>{svg}</td>{std_fx_cells(d, name, ref)}</tr>")
    out.append("</table></div>")
    out.insert(1, status_filter(groups))
    out.append(milestone_note(d))
    if unrated:
        out.append(f'<p class="note">Not rated yet ({len(unrated)}): new entries that have played no games, or not '
                   f'enough to be tied to random both ways: {", ".join(esc(n) for n in unrated[:70])}.</p>')
    how = ("from the fit's covariance" if gain_how == {"cov"} else
           "from the previous fit's covariance until today's is computed (a few seconds after each new fit)"
           if gain_how == {"prev"} else
           "the two ±s combined as if independent, until the fit's covariance is computed (a few seconds after "
           "each new fit)" if gain_how == {"hypot"} else "from the fit's covariance where computed (hover a value)")
    after = f", after {rr_words(d, T)}" if T is not None else ""
    out.append(f'<p class="note">Ratings from the league\'s current fit{esc(after)}: '
               f'{esc(os.path.basename(d.league))}/ratings.csv, pure maximum likelihood, random at 0; ± is the 95% '
               f'interval. Change: over the cycle, per lineage and ply, the row\'s Elo now minus the lineage\'s '
               f'rating at that ply at the end of the previous cycle\'s tournament'
               + (f' (cycle {B}\'s, after round-robin {tb})' if B is not None else '')
               + f', whatever version it was then: the last segment of its line on Elo over cycles (cycles.csv at ply '
               f'{q}, the league\'s elo_history.csv at ply {q - 1}). A version carried over shows its own drift, a '
               f'new one its change from the version it replaced; a random player, its own drift. While a cycle\'s '
               f'round-robins run, the change so far in it' + (f' ({k_rr} of {n_rr} in)' if partial else '')
               + f'; during the next cycle\'s self-play, that cycle\'s whole change. new: the lineage\'s first '
               f'cycle at that ply. Hover a change for both ratings and the version\'s story. Status: a dormant lineage shows the cycles it has spent dormant since its latest '
               f'going-dormant decision, counting the one under way; hover it for when, and at which version. '
               f'Peak Elo and Peak version: the lineage\'s highest rating at the row\'s ply as measured at the '
               f'completion of a cycle, and the version that played it: completed cycles only (cycles.csv\'s Elo '
               f'at the decision at the qualifying ply, the league\'s Elo history at the cycle\'s last round-robin '
               f'at the other; the cycle under way counts once it has decided, never the running fit), so the '
               f'row\'s Elo can be above it mid-tournament; hover a peak for that cycle and the version\'s rating '
               f'today. "new peak": the peak is the latest completed cycle\'s (not a lineage\'s first). On '
               f'dormant rows, in parentheses: the same peak over the cycles up to the one whose decision made it '
               f'dormant. The rule compares a version with the best earlier version in today\'s fit instead, '
               f'which is what cycles.csv logs as peak_version and peak_elo (in the hover).'
               + (f' A dormant lineage wakes when it beats its best version up to the one that went dormant '
                  f'(state.json\'s dormant_version), this one excluded, in today\'s fit, by more than {rse_w}: '
                  f'the rule\'s wake bar, in the status and peak hovers.' if dorm_rule else '')
               + f' Ply gain: the model\'s ply-{q} entry\'s Elo minus its ply-{q - 1} entry\'s, on the ply-{q} row, '
                 f'± {how}; click its header to sort by it (again to reverse, Elo to go back). Filter by status: '
                 f'tick statuses to show only their rows; none ticked shows every row; the ranks stay as they are, '
                 f'and the choice is kept in the browser. The factor filters in the bar at the top hide the '
                 f'entries of the lineages they leave out (and those at a play depth they leave out; never the '
                 f'random players); the count at the right of the status filter says how many lineages and rows '
                 f'show. One table, ranked by Elo across every lineage; the factor columns give each lineage\'s '
                 f'value of each factor (as in the Lineages table, which adds the variant), the one the page colours '
                 f'by in bold. Click a row to highlight its lineage in the chart at the top (again to un-highlight).'
               + (' Retired rows (muted, rank –): the last entries of a retired lineage, "retired (cN)" for retired '
                  'after cycle N, and the entry of a frozen lineage retired at the freeze, "retired (frozen)". They '
                  'play no more, but their games stay in the fit, so their Elo can still drift with refits; their '
                  'change is their own drift (0, never "new", when the entry has no rating of its own at the end of '
                  'the previous cycle), and they take no rank, so the ranks of the active entries do not shift.'
                  if gone else '')
               + (' A frozen lineage (the badge) keeps one entry in the field and gets no self-play, own-games step, '
                  'gate or qualification decision; its peaks show as for any lineage.' if frozen_seen else '')
               + '</p>')
    same = {lin: lg for lin, lg in logged.items() if lg["same"]}
    if same:
        bad = sorted(lin for lin, lg in same.items() if not lg["agrees"])
        srcs = " and ".join(sorted({f"{lg['src']} (cycle {lg['cycle']})" for lg in same.values()}))
        cols = "peak_version and peak_elo" + (" (and pre_dormancy_peak_version and pre_dormancy_peak_elo on dormant "
                                              "rows)" if any(lg["pre"] for lg in same.values()) else "")
        text = (f"The best earlier versions (the rule's comparison, in the peak hovers) checked against the {cols} "
                f"that {esc(srcs)} logged from this same fit: they "
                f"agree for {len(same) - len(bad)} of {len(same)} lineage{'s' if len(same) != 1 else ''}.")
        if bad:
            text += (f' <span class="warn">They differ for {esc(", ".join(bad))}</span> (≠ in the table where the '
                     f'lineage is in it; hover for both).')
        out.append(f'<p class="note">{text}</p>')
    elif logged:
        c = max(lg["cycle"] for lg in logged.values())
        out.append(f'<p class="note">The peaks logged up to cycle {c} come from earlier fits than this one, so they '
                   f'are not compared with these; hover a peak for its logged value.</p>')
    return "".join(out)


# ---------------------------------------------------------------- panel 3: Elo over cycles

def panel_elo(d):
    ply = d.qualify_ply
    series = {lin: per[ply] for lin, per in cycle_points(d).items() if lin in d.lin and per.get(ply)}
    if not series:
        return ('<p class="note">No rated lineage versions yet: each lineage\'s versions appear here once they have '
                'played a tournament.</p>')
    refs = {n: d.ratings[n] for n in d.refs if n in d.ratings}
    allp = [pt for s in series.values() for pt in s]
    lo = min(min(pt["elo"] - (pt["ci"] or 0) for pt in allp), min((r["elo"] for r in refs.values()), default=0))
    hi = max(pt["elo"] + (pt["ci"] or 0) for pt in allp)
    pad = 0.05 * (hi - lo or 100)
    lo, hi = lo - pad, hi + pad
    cycles = sorted({pt["c"] for pt in allp})
    c0, c1 = cycles[0], cycles[-1]
    if c0 == c1:
        c0, c1 = c0 - 0.5, c1 + 0.5
    else:
        c0, c1 = c0 - 0.3, c1 + 0.3
    W, H, L, R, T, B = 300, 190, 44, 10, 10, 26
    sx = lambda c: L + (W - L - R) * (c - c0) / (c1 - c0)
    sy = lambda e: T + (H - T - B) * (hi - e) / (hi - lo)
    yt = nice_ticks(lo, hi, 4)
    step = max(1, math.ceil(len(range(int(math.ceil(c0)), int(c1) + 1)) / 8))
    xt = [c for c in range(int(math.ceil(c0)), int(c1) + 1) if (c - int(math.ceil(c0))) % step == 0]

    joins = [(c, ns) for c, ns in ref_joins(d) if c0 < c - 0.5 < c1]

    def frame():
        g = "".join(f'<line x1="{L}" x2="{W - R}" y1="{sy(t):.1f}" y2="{sy(t):.1f}" class="grid"/>'
                    f'<text x="{L - 6}" y="{sy(t) + 3.5:.1f}" text-anchor="end">{plain(t)}</text>'
                    for t in yt)
        g += "".join(f'<text x="{sx(c):.1f}" y="{H - 8}" text-anchor="middle">{c}</text>' for c in xt)
        g += f'<line x1="{L}" x2="{W - R}" y1="{H - B}" y2="{H - B}" class="axisline"/>'
        for n, r in refs.items():  # the random players, as hairline references
            g += (f'<line x1="{L}" x2="{W - R}" y1="{sy(r["elo"]):.1f}" y2="{sy(r["elo"]):.1f}" class="refline" '
                  f'data-tip="{esc(n)}: Elo {plain(r["elo"])} in today\'s fit"/>')
        for c, ns in joins:  # reference players that joined later: the cycles' ratings are linked better since
            g += (f'<line x1="{sx(c - 0.5):.1f}" x2="{sx(c - 0.5):.1f}" y1="{T}" y2="{H - B}" class="ec-join" '
                  f'data-tip="{esc(", ".join(ns))} joined the field in cycle {c}"/>')
        return g

    def draw(lins, title):
        body, items = [], []
        for lin in lins:
            s = series.get(lin)
            l = d.lin[lin]
            fam, deep = shape_of(d, lin), l.get("depth", 0) >= 2
            colour = fam_colour(fam)
            key = line_key(fam, deep)
            if not s:
                items.append(f'<span data-s="{esc(lin)}" class="li muted"{lc_attr(d, lin)}>{key}{esc(lin)} '
                             f'(no rating yet)</span>')
                continue
            g = [f'<g data-s="{esc(lin)}"{lc_attr(d, lin)}>']
            if len(s) > 1:
                path = " ".join(f"{'M' if i == 0 else 'L'}{sx(pt['c']):.1f},{sy(pt['elo']):.1f}"
                                for i, pt in enumerate(s))
                g.append(f'<path d="{path}" fill="none" stroke="transparent" stroke-width="12"/>'
                         f'<path d="{path}" fill="none" stroke="{colour}" stroke-width="2" stroke-linejoin="round" '
                         + (f'stroke-dasharray="{DASH}"/>' if deep else 'stroke-linecap="round"/>'))
            for pt in s:
                x, y = sx(pt["c"]), sy(pt["elo"])
                tip = esc(point_tip(lin, ply, pt))
                if pt["ci"] is not None:
                    g.append(f'<line x1="{x:.1f}" x2="{x:.1f}" y1="{sy(pt["elo"] - pt["ci"]):.1f}" '
                             f'y2="{sy(pt["elo"] + pt["ci"]):.1f}" stroke="{colour}" stroke-width="1.5" '
                             f'opacity="0.5"/>')
                g.append(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="12" fill="transparent" data-tip="{tip}"/>'
                         + marker(fam, x, y, r=4.0 if pt["since"] == pt["c"] else 2.5, extra=f' data-tip="{tip}"'))
            g.append("</g>")
            body.append("".join(g))
            last = s[-1]
            gone = f" · {status_of(d, lin)}" if is_retired(d, lin) or is_frozen(d, lin) else ""
            items.append(f'<span data-s="{esc(lin)}" class="li" tabindex="0"{lc_attr(d, lin)}>{key}{esc(lin)} '
                         f'<span class="muted">{plain(last["elo"])}{" live" if last["live"] else ""}{esc(gone)}'
                         f'</span></span>')
        return (f'<figure class="mult"><figcaption>{esc(title)}</figcaption>'
                f'<svg width="{W}" height="{H}" role="img" aria-label="{esc(title)}">{frame()}{"".join(body)}</svg>'
                f'<div class="items">{"".join(items)}</div></figure>')

    shown = d.registry + [l for l in d.former if l["name"] in series]  # retired ones keep their history
    panels = [draw(names, title) for title, names in mult_panels(d, shown)]
    table = ['<details id="elo-table"><summary>Table view</summary><table><tr><th>lineage</th>'
             + "".join(f"<th class='r'>cycle {c}</th>" for c in cycles) + "</tr>"]
    for l in shown:
        s = {pt["c"]: pt for pt in series.get(l["name"], [])}
        gone = f" <span class='muted'>({esc(status_of(d, l['name']))})</span>" if is_retired(d, l["name"]) else ""
        table.append(f"<tr{lc_attr(d, l['name'])}><td>{swatch(shape_of(d, l['name']))} {esc(l['name'])}{gone}</td>"
                     + "".join(
            f"<td class='r'>{plain(s[c]['elo'])}" + (f" ± {s[c]['ci']:.0f}" if s[c]["ci"] is not None else "")
            + f" <span class='muted'>v{s[c]['v']}{' live' if s[c]['live'] else ''}</span></td>"
            if c in s else "<td></td>" for c in cycles) + "</tr>")
    table.append("</table></details>")
    return (fx_legend(d, extra=deep_key(shown))
            + f'<p class="note">Each lineage\'s rating in every cycle it played, by its ply-{ply} entry, as '
              f'measured at that cycle\'s decision (cycles.csv), and for the cycle under way live from today\'s fit '
              f'once its tournament has begun. A smaller marker is the same version as the cycle before, re-rated: '
              f'a version that keeps playing (a dormant lineage\'s, say) moves as the league plays on. Whiskers are '
              f'the 95% ranges and hairlines mark the random players in today\'s fit. Each cycle\'s fit adds that '
              f'cycle\'s games to all the earlier ones, random at 0, so a step from one cycle to the next mixes the '
              f'lineage\'s own change with the whole field\'s re-rating; versions of different cycles meet only '
              f'through the players their tournaments share (through cycle 4, the random players alone).'
              f'{join_note(d)} Hover a point for its version and that version\'s rating in today\'s fit, or a '
              f'series or a name to pick it out; click a series or a name to highlight the lineage in the chart at '
              f'the top and go there, or a panel\'s title for all of its lineages the filters show.</p>'
            + '<div class="grid-mult">' + "".join(panels) + "</div>" + "".join(table))


def mult_panels(d, shown):
    """The small multiples' panels, [(title, lineages)]: F-L's variants, then the other flat models and the MoE, then
    one per tree or DAG archetype of lineages.txt (its order), the HTs together; each titled by what varies in it."""
    groups = {}
    for l in shown:
        f = fx(d)[l["name"]][0]
        key = (("F-L" if l["arch"] == "F-L" else "flat") if f["arch"] in ("F", "M") else
               "HT" if f["router"] == "H" else l["arch"] if f["arch"] in ("T", "D") else UNCL)
        groups.setdefault(key, []).append(l["name"])
    out = []
    for key in ["F-L", "flat"] + [k for k in groups if k not in ("F-L", "flat", "HT", UNCL)] + ["HT", UNCL]:
        names = groups.get(key)
        if not names:
            continue
        arches = ", ".join(dict.fromkeys(d.lin[n]["arch"] for n in names))
        title = ("F-L variants" if key == "F-L" else f"Other flat models and MoE ({arches})" if key == "flat" else
                 f"HT ({arches})" if key == "HT" else "Unclassified" if key == UNCL else key)
        vary = fx_varies(d, names) if key not in ("flat", UNCL) else ""
        out.append((title + (f": {vary}" if vary else ""), names))
    return out


def deep_key(lins):
    """The legend's entry for the dashed lines, when any lineage self-plays at depth 2 or more."""
    deep = sorted({l.get("depth", 0) for l in lins if l.get("depth", 0) >= 2})
    if not deep:
        return ""
    what = " or ".join(str(x) for x in deep)
    return (f'<span><svg class="sw" width="30" height="14" aria-hidden="true"><line x1="1" x2="29" y1="7" y2="7" '
            f'stroke="var(--ink2)" stroke-width="2" stroke-dasharray="{DASH}"/></svg>dashed line: self-plays at depth '
            f'{what} (--depth in lineages.txt)</span>')


# ---------------------------------------------------------------- panel 4: history

def panel_history(d):
    out = []
    if d.history:
        fz, rt = any(h["frozen"] for h in d.history), any(h.get("retired") for h in d.history)
        out.append('<table><tr><th class="r">cycle</th><th>started</th><th class="r">took</th>'
                   '<th class="r">self-play</th><th class="r">tournament</th><th class="r">own + gates</th>'
                   '<th class="r">trained</th><th class="r">qualified</th><th class="r">dormant</th>'
                   + ('<th class="r">frozen</th>' if fz else '') + ('<th class="r">retired</th>' if rt else '') +
                   '<th class="r">gates kept</th><th class="r">self-play CPU</th></tr>')
        for h in d.history:
            want = h["budget"] * h["trained"] if h["trained"] else None
            cpu = (f"{h['cpu'] / 3600:.2f} h" + (f" <span class='muted'>({100 * h['cpu'] / want:.0f}% of budget)"
                                                 f"</span>" if want else "")) if h["cpu"] else "—"
            out.append(
                f"<tr><td class='r'>{h['cycle']}</td><td>{fmt_clock(h['start'])}</td><td class='r'>{fmt_dur(h['took'])}</td>"
                f"<td class='r'>{fmt_dur(h['selfplay'])}</td><td class='r'>{fmt_dur(h['tournament'])}</td>"
                f"<td class='r'>{fmt_dur(h['own'])}</td><td class='r'>{h['trained']}</td>"
                f"<td class='r'>{h['qualified']}</td><td class='r'>{h['dormant']}</td>"
                + (f"<td class='r'>{h['frozen']}</td>" if fz else "") + (f"<td class='r'>{h['retired']}</td>" if rt else "") +
                f"<td class='r'>{h['kept']} of {h['gates']}</td><td class='r'>{cpu}</td></tr>")
        out.append('</table><p class="note">Self-play time is the job queue\'s makespan, from each job\'s wall time; '
                   'own + gates is the rest of the cycle after the tournament. Cycle 0 is the start, with nothing '
                   'trained.' + (' Frozen: lineages frozen that cycle, with no self-play, own-games step, gate or '
                                 'qualification decision; their kept entry still plays the tournament.' if fz else '')
                   + '</p>')
    else:
        out.append('<p class="note">No cycle logged yet.</p>')
    # CPU per lineage: the running cycle's jobs, or the last logged cycle's
    rows, label, budget = [], "", d.budget  # each cycle against its own budget: the running one's is config.json's
    if d.cur and d.cur.get("jobs"):
        for name, p, row, status, cpu, wall, k in job_rows(d):
            rows.append((name, cpu, status))
        label = f"cycle {d.cur.get('cycle')} (under way)"
    else:
        for h in reversed(d.history):
            got = [(r["lineage"], num(r.get("cpu_s")), "done") for r in h["rows"] if num(r.get("cpu_s")) is not None]
            if got:
                rows, label, budget = got, f"cycle {h['cycle']}", h["budget"]
                break
    if rows:
        top = max([budget * 1.25] + [c for _, c, _ in rows if c])
        W, L, R, rowh = 820, 150, 130, 20
        H = rowh * len(rows) + 30
        sx = lambda v: L + (W - L - R) * v / top
        ticks = nice_ticks(0, top, 6)
        g = [f'<line x1="{sx(t):.1f}" x2="{sx(t):.1f}" y1="0" y2="{H - 24}" class="grid"/>'
             f'<text x="{sx(t):.1f}" y="{H - 8}" text-anchor="middle">{t:,.{0 if float(t).is_integer() else 1}f}</text>'
             for t in ticks]
        for i, (name, cpu, status) in enumerate(rows):
            y = i * rowh + 4
            fam = shape_of(d, name)
            g.append(f'<g transform="translate(0,{y})"><text x="{L - 22}" y="12" text-anchor="end">{esc(name)}</text>'
                     f'<g transform="translate({L - 12},8)"{"" if fam == "random" else lc_attr(d, name)}>'
                     f'{marker(fam, 0, 0, r=3.5)}</g>')
            if cpu:
                inb = min(cpu, budget)
                tip = f"{name}: {cpu:.0f} CPU-s ({status}), {100 * cpu / budget:.0f}% of the {budget:.0f} budget"
                g.append(f'<rect x="{L}" y="2" width="{max(1.0, sx(inb) - L):.1f}" height="12" rx="2" '
                         f'class="{"barrun" if status == "running" else "bar"}" data-tip="{esc(tip)}"/>')
                if cpu > budget:
                    g.append(f'<rect x="{sx(budget) + 1:.1f}" y="2" width="{max(1.0, sx(cpu) - sx(budget) - 1):.1f}" '
                             f'height="12" rx="2" class="barover" data-tip="{esc(tip)}"/>')
                over = 100 * (cpu / budget - 1)
                txt = f"{cpu:,.0f}" + (f" ▲ {over:.0f}%" if over > 10 else "") + (" (running)" if status == "running" else "")
                g.append(f'<text x="{sx(cpu) + 4:.1f}" y="12" class="val">{esc(txt)}</text>')
            else:
                g.append(f'<text x="{L + 2}" y="12" class="muted">{esc(status)}</text>')
            g.append("</g>")
        g.append(f'<line x1="{sx(budget):.1f}" x2="{sx(budget):.1f}" y1="0" y2="{H - 24}" class="budget"/>')
        out.append(f'<h3>Self-play CPU per lineage, {esc(label)}</h3><p class="note">The bar beyond the vertical '
                   f'line at {budget:.0f} CPU-s (the budget) is the overspend; ▲ marks more than 10% over.</p>'
                   f'<svg width="{W}" height="{H}" class="cpu" role="img" aria-label="CPU per lineage">{"".join(g)}</svg>')
    # disk
    try:
        vfs = os.statvfs(d.runs_dir if os.path.isdir(d.runs_dir) else d.root)
        free, total = vfs.f_bavail * vfs.f_frsize, vfs.f_blocks * vfs.f_frsize
    except OSError:
        free = total = None
    du = _du["total"]
    big = sorted(_du["per"].items(), key=lambda x: -x[1])[:5]
    out.append('<h3>Disk</h3><div class="tiles">'
               f'<div class="tile"><div class="tl">Models (cycles/runs)</div><div class="tv">{fmt_bytes(du)}</div>'
               f'<div class="ts">{"measured " + fmt_clock(_du["at"]) + ", every 10 min" if _du["at"] else "measuring…"}'
               f'</div></div>'
               f'<div class="tile"><div class="tl">Free disk</div><div class="tv">{fmt_bytes(free)}</div>'
               f'<div class="ts">of {fmt_bytes(total)}</div></div></div>')
    if big:
        out.append('<p class="note">Largest: ' + ", ".join(f"{esc(n)} {fmt_bytes(b)}" for n, b in big) + ".</p>")
    return "".join(out)


# ---------------------------------------------------------------- panel: gate history

GATE_MARK = {"kept": ("✓", "gk", "kept"), "reverted": ("↺", "gr", "reverted")}


def match_stats(w, l, dr):
    """Score, Elo and the 95% half-width of a match, as cycle.py (and the arena) compute them."""
    n = w + l + dr
    s = (w + dr / 2) / n
    var = (w * (1 - s) ** 2 + l * s ** 2 + dr * (0.5 - s) ** 2) / n
    margin = 1.96 * math.sqrt(var / n)
    elo = lambda x: -400 * math.log10(1 / min(max(x, 1e-6), 1 - 1e-6) - 1)
    return s, elo(s), (elo(s + margin) - elo(s - margin)) / 2


def fmt_s(x):
    """Seconds as the logs give them: whole above 10, tenths below."""
    return f"{x:,.0f}" if x >= 10 else f"{x:.1f}"


def gate_wld(r):
    """A row's gate match as (wins, losses, draws), or None if it had none."""
    x = [num(r.get(k)) for k in ("gate_wins", "gate_losses", "gate_draws")]
    return tuple(int(v) for v in x) if None not in x and sum(x) > 0 else None


def gate_pool(rows):
    """The gates pooled as each cycle's summary pools them: (gates, kept, wins, losses, draws), or None."""
    gs = [(x, r.get("gate")) for x, r in ((gate_wld(r), r) for r in rows) if x]
    if not gs:
        return None
    return (len(gs), sum(1 for _, g in gs if g == "kept")) + tuple(sum(x[i] for x, _ in gs) for i in range(3))


def pool_text(p, sign=True):
    """A pool in the summaries' words: "30 gates, +W −L =D, score S%, Elo E ± C; kept K of 30". sign: a cycle's
    pool, a change that cycle ("+"); the pool of all cycles has none."""
    n, kept, w, l, dr = p
    s, elo, ci = match_stats(w, l, dr)
    return (f"{n} gate{'s' if n != 1 else ''}, +{w} −{l} ={dr}, score {100 * s:.1f}%, "
            f"Elo {signed(elo, 1) if sign else plain(elo, 1)} ± "
            f"{ci:.1f}; kept {kept} of {n}")


def gate_history(d):
    """[(cycle, {lineage: row}, partial)]: each logged cycle's rows from cycles.csv, then the cycle under way's from
    state.json (as the Now panel reads them) once its own-games steps have begun."""
    out = [(h["cycle"], {r.get("lineage"): r for r in h["rows"]}, False) for h in d.history if h["cycle"] >= 1]
    cur = d.cur
    if cur and isinstance(cur.get("cycle"), int) and cur["cycle"] not in {c for c, _, _ in out}:
        rows = {k: v for k, v in (cur.get("rows") or {}).items() if isinstance(v, dict)}
        if cur.get("phase") in ("own", "log") or any(r.get("gate") for r in rows.values()):
            out.append((cur["cycle"], rows, True))
    return out


def gate_cell(d, lin, c, r, partial, thr):
    """One lineage's gate in one cycle: the score and Elo, marked kept or reverted (an icon as well as a colour)."""
    if r is None:
        return f'<td class="gc" data-tip="{esc(f"{lin}: not in the league in cycle {c}")}"></td>'
    g = str(r.get("gate") or "")
    x = gate_wld(r)
    cur = d.cur or {}
    dormant = r.get("status") == "dormant" or (partial and lin in (cur.get("dormant") or []))
    frozen = "frozen" in (r.get("status"), r.get("decision"))
    head = f"{lin}, cycle {c}" + (" (dormant)" if dormant else " (frozen)" if frozen else "")
    zz = '<span class="gz">dormant</span>' if dormant else '<span class="gz">frozen</span>' if frozen else ""
    og, op, oc, gc = (num(r.get(k)) for k in ("own_games", "own_positions", "own_cpu_s", "gate_cpu_s"))
    if x and g in GATE_MARK:
        w, l, dr = x
        s, elo, ci = match_stats(w, l, dr)
        s, elo, ci = num(r.get("gate_score"), s), num(r.get("gate_elo"), elo), num(r.get("gate_ci95"), ci)
        icon, cls, word = GATE_MARK[g]
        v, nv = num(r.get("version")), num(r.get("next_version"))
        if g == "kept":
            what = "kept: the step stays" + (f" (v{v:.0f} → v{nv:.0f})" if v is not None and nv not in (None, v)
                                             else "")
        else:
            what = (f"reverted: it scored under {100 * thr:g}%, so the step was undone"
                    + (f" (stays v{v:.0f})" if v is not None else ""))
        tip = [f"{head}: {what}",
               f"+{w} −{l} ={dr} in {w + l + dr:,} games against the model from before the step: score "
               f"{100 * s:.2f}%, Elo {signed(elo, 1)} ± {ci:.1f}"]
        if og is not None:
            tip.append(f"trained one pass on {og:,.0f} own games" + (f" ({op:,.0f} positions)" if op is not None else ""))
        cpu = [f"gate {fmt_s(gc)} CPU-s" if gc is not None else "", f"training {fmt_s(oc)} CPU-s" if oc is not None else ""]
        if any(cpu):
            tip.append(", ".join(x for x in cpu if x))
        pct = f"{100 * s:.1f}"
        if float(pct) == round(100 * thr, 1) and (s >= thr) != (float(pct) >= 100 * thr):
            pct = f"{100 * s:.2f}"  # rounds onto the threshold from below: show the side it fell on
        return (f'<td class="gc {cls}" data-tip="{esc(chr(10).join(tip))}"><span class="gi" aria-hidden="true">{icon}'
                f'</span><span class="sr">{word} </span>{pct}%<br><span class="ge">{signed(elo)} ± {ci:.0f}'
                f'</span>{zz}</td>')
    if g:  # the step or its gate failed, or was skipped: the driver's own words
        word = "skipped" if "skipped" in g else "failed"
        return (f'<td class="gc gf" data-tip="{esc(head + ": " + g)}"><span class="gi" aria-hidden="true">!</span>'
                f'{word}{zz}</td>')
    if r.get("status") == "retired":  # retired at this decision by the freeze/retire rule: no step, nothing missing
        tip = head + ": retired at this cycle’s decision, so no own-games step or gate"
        return f'<td class="gc" data-tip="{esc(tip)}"><span class="frz">retired</span></td>'
    if frozen:  # no own-games step or gate while frozen: nothing missing
        return (f'<td class="gc" data-tip="{esc(head + ": no own-games step or gate while frozen")}">'
                f'<span class="frz">frozen</span></td>')
    if partial and cur.get("phase") == "own" and d.lin.get(lin, {}).get("own") and lin not in (cur.get("own_done") or []):
        return f'<td class="gc" data-tip="{esc(head + ": its own-games step hasn’t run yet")}">…{zz}</td>'
    why = "the own-games step is off for this lineage" if lin in d.lin and not d.lin[lin]["own"] else "no own-games step"
    return f'<td class="gc" data-tip="{esc(head + ": " + why)}"><span class="muted">—</span>{zz}</td>'


def pool_cell(what, p, partial, sign=True):
    """A pool as a cell: `what` names it ("Cycle 3", "All cycles")."""
    if not p:
        return (f'<td class="gc" data-tip="{esc(what)}: no gates{" yet" if partial else ""}"><span class="muted">no '
                f'gates{" yet" if partial else ""}</span></td>')
    n, kept, w, l, dr = p
    s, elo, ci = match_stats(w, l, dr)
    tip = f"{what}{' so far' if partial else ''}, pooled over the lineages: {pool_text(p, sign)}"
    return (f'<td class="gc gp" data-tip="{esc(tip)}">{n} gate{"s" if n != 1 else ""}, kept {kept}<br>'
            f'<span class="ge">+{w} −{l} ={dr}</span><br>{100 * s:.1f}%, {signed(elo, 1) if sign else plain(elo, 1)} '
            f'± {ci:.1f}</td>')


def gate_chart(pts):
    """Pooled gate Elo, ±95%, per cycle: pts = [(cycle, pool, partial)]. One series: its caption names it."""
    vals = [(c, *match_stats(*p[2:]), p, part) for c, p, part in pts]
    lo = min([0.0] + [e - ci for _, _, e, ci, _, _ in vals])
    hi = max([0.0] + [e + ci for _, _, e, ci, _, _ in vals])
    pad = 0.08 * (hi - lo or 40)
    lo, hi = lo - pad, hi + pad
    cs = [v[0] for v in vals]
    c0, c1 = (cs[0] - 0.6, cs[-1] + 0.6) if len(cs) == 1 else (cs[0] - 0.45, cs[-1] + 0.45)
    W, H, L, R, T, B = 450, 176, 44, 64, 10, 30
    sx = lambda c: L + (W - L - R) * (c - c0) / (c1 - c0)
    sy = lambda e: T + (H - T - B) * (hi - e) / (hi - lo)
    g = []
    for t in nice_ticks(lo, hi, 4):
        g.append(f'<line x1="{L}" x2="{W - R}" y1="{sy(t):.1f}" y2="{sy(t):.1f}" class="{"zero" if t == 0 else "grid"}"/>'
                 f'<text x="{L - 6}" y="{sy(t) + 3.5:.1f}" text-anchor="end">{plain(t)}</text>')
    if 0 not in nice_ticks(lo, hi, 4):
        g.append(f'<line x1="{L}" x2="{W - R}" y1="{sy(0):.1f}" y2="{sy(0):.1f}" class="zero"/>')
    g.append(f'<text x="{W - R + 4}" y="{sy(0) + 3.5:.1f}">50% score</text>')
    step = max(1, math.ceil(len(cs) / 12))
    for i, c in enumerate(cs):
        if i % step == 0 or i == len(cs) - 1:
            g.append(f'<text x="{sx(c):.1f}" y="{H - 12}" text-anchor="middle">{c}</text>')
    g.append(f'<text x="{(L + W - R) / 2:.1f}" y="{H - 1}" text-anchor="middle">cycle</text>')
    for c, s, e, ci, p, part in vals:
        x, y = sx(c), sy(e)
        tip = f"Cycle {c}{' so far' if part else ''}: {pool_text(p)}"
        g.append(f'<line x1="{x:.1f}" x2="{x:.1f}" y1="{sy(e - ci):.1f}" y2="{sy(e + ci):.1f}" class="wh"/>'
                 f'<circle cx="{x:.1f}" cy="{y:.1f}" r="4.5" class="pt{" part" if part else ""}"/>'
                 f'<circle cx="{x:.1f}" cy="{y:.1f}" r="13" fill="transparent" data-tip="{esc(tip)}"/>')
    c, s, e, ci, p, part = vals[-1]
    g.append(f'<text x="{sx(c) + 9:.1f}" y="{sy(e) + 3.5:.1f}" class="val">{signed(e, 1)}</text>')
    return (f'<svg width="{W}" height="{H}" role="img" aria-label="Pooled gate Elo per cycle">{"".join(g)}</svg>')


def panel_gates(d):
    gcfg = d.cfg.get("gate") or {}
    thr = num(gcfg.get("threshold"), 0.5)
    dormant_too = (d.cfg.get("own_games") or {}).get("dormant", True)
    hist = gate_history(d)
    any_frozen = (any(is_frozen(d, l["name"]) for l in d.registry)
                  or any("frozen" in (r.get("status"), r.get("decision")) for _, rows, _ in hist for r in rows.values()))
    paren = ", ".join(x for x in ("dormant ones too" if dormant_too else "", "frozen ones not" if any_frozen else "") if x)
    intro = (f'<p class="note">Each cycle\'s own-games step trains every lineage{f" ({paren})" if paren else ""} '
             f'one pass on its entries\' tournament games. A gate match of {int(num(gcfg.get("games"), 1000)):,} games '
             f'at depth {int(num(gcfg.get("depth"), 2))} against the model from before the step keeps the step if it '
             f'scores at least {100 * thr:g}%, and reverts it otherwise.</p>')
    if not any(gate_wld(r) or r.get("gate") for _, rows, _ in hist for r in rows.values()):
        return intro + '<p class="note">No gates yet: the first come at the end of cycle 1.</p>'
    pools = [(c, gate_pool(rows.values()), part) for c, rows, part in hist]
    every = gate_pool([r for _, rows, _ in hist for r in rows.values()])
    live = next((c for c, _, part in hist if part), None)
    chart = gate_chart([(c, p, part) for c, p, part in pools if p])
    so_far = f", with cycle {live}'s gates so far" if live is not None else ""
    top = (f'<div class="gates-top"><figure class="gchart"><figcaption>Pooled gate result per cycle: Elo against the '
           f'model before the step, ±95%</figcaption>{chart}</figure><div>{intro}'
           f'<p class="note"><b>All cycles{esc(so_far)}:</b> {esc(pool_text(every, False)) if every else "no gates"}.</p>'
           '<p class="note">Each cycle\'s point pools every lineage\'s gate games, kept or reverted (the pooled row '
           'below, and the first line of each cycles/cycle-NNN.md)'
           + ('; a hollow point is the cycle under way, so far' if live is not None else "") + '.</p></div></div>')
    # rows: the lineages by their value of the factor the page groups by (here the default's; the page script
    # regroups them by the user's pick), in lineages.txt order; the retired ones, then any no longer there, at the end
    seen = {lin for _, rows, _ in hist for lin in rows if lin}
    active, cols = [l["name"] for l in d.registry if not is_retired(d, l["name"])], fx_colours(d, FX_BY)
    groups = [(cols[v], f"{FX[FX_BY][0]}: {fx_label(FX_BY, v)}", [n for n in active if fx(d)[n][0][FX_BY] == v])
              for v in fx_values(d, FX_BY)]
    groups = [g for g in groups if g[2]]
    retired = [l["name"] for l in d.registry + d.former if is_retired(d, l["name"]) and l["name"] in seen]
    if retired:
        groups.append((None, "Retired", retired))
    gone = sorted(seen - {l["name"] for l in d.registry} - set(retired))
    if gone:
        groups.append((None, "No longer in lineages.txt", gone))
    ncol = len(hist) + 3
    head = "".join(f'<th class="r">cycle {c}' + ('<br><span class="gpart">so far</span>' if part else "") + "</th>"
                   for c, _, part in hist)
    out = [top, '<div class="gates-wrap" id="gates-wrap" data-sk="1"><table class="gates"><thead><tr>'
                f'<th class="corner">lineage</th>{head}<th class="r">all cycles:<br>kept</th>'
                f'<th class="r gsum">sum of kept steps<br><span class="gpart">(biased up: each was kept because it '
                f'scored ≥{100 * thr:g}%)</span></th></tr></thead><tbody>',
           '<tr class="gpool"><th>all lineages, pooled</th>'
           + "".join(pool_cell(f"Cycle {c}", p, part) for c, p, part in pools)
           + pool_cell("All cycles" + so_far, every, False, sign=False) + '<td class="gc"></td></tr>']
    for col, label, names in groups:
        out.append(f'<tr class="gfam{" fx-h" if col else ""}"><th colspan="{ncol}"><span class="gfl">'
                   f'{fx_dot(col) if col else ""} {esc(label)}</span></th></tr>')
        fixed = "" if col else ' class="gfx"'
        for lin in names:
            cells, kept, total, elo_sum = [], 0, 0, 0.0
            for c, rows, part in hist:
                r = rows.get(lin)
                cells.append(gate_cell(d, lin, c, r, part, thr))
                if r is not None and gate_wld(r) and r.get("gate") in GATE_MARK:
                    total += 1
                    if r.get("gate") == "kept":
                        kept += 1
                        elo_sum += num(r.get("gate_elo"), match_stats(*gate_wld(r))[1])
            if total:
                summ = (f'<td class="gc">{kept} of {total}</td><td class="gc" data-tip="{esc(lin)}: the {kept} kept '
                        f'steps\' gate Elo, summed (each was kept because it scored ≥{100 * thr:g}%, so this '
                        f'overstates what they added)">{plain(elo_sum) if kept else "—"}</td>')
            else:
                summ = '<td class="gc"><span class="muted">no gates yet</span></td><td class="gc"></td>'
            tip = f' data-tip="{esc(lin)}: {esc(status_of(d, lin))}"' if lin in retired else ""
            out.append(f'<tr{lc_attr(d, lin)}{fixed}><th{tip}>{swatch(shape_of(d, lin))} {esc(lin)}</th>'
                       f'{"".join(cells)}{summ}</tr>')
    out.append("</tbody></table></div>")
    out.append(f'<p class="note"><span class="gkey gk"><span class="gi">✓</span> kept</span> '
               f'<span class="gkey gr"><span class="gi">↺</span> reverted: the step was undone</span> '
               '<span class="gkey gf"><span class="gi">!</span> the step or its gate failed or was skipped</span> '
               '— no gate that cycle · blank: not in the league that cycle · <i>dormant</i>: the lineage was dormant '
               'that cycle' + (' · <span class="frz">frozen</span>: frozen that cycle, so no own-games step or gate'
                               if any_frozen else '') + '. Each cell is the trained model\'s score and Elo ± 95% against the model before the step; hover '
               'it for +wins −losses =draws, the own games and positions it trained on, and the gate\'s CPU. The '
               'rows fall into a group per value of the factor the page groups by (the retired lineages last), and '
               'the factor filters hide the lineages they leave out; the pooled rows still pool every lineage. Click '
               'a lineage\'s name to highlight it in the chart at the top.</p>')
    return "".join(out)


# ---------------------------------------------------------------- panel: head to head

class PairCounter:
    """Each pair's results per cycle, from games.csv, read incrementally like GameCounter: only the bytes appended
    since the last call. pairs[(a, b)] with a < b: {cycle: [a's wins, b's wins, draws]}. A round-robin's cycle comes
    from `cycle_of` when its games are read. `version` changes whenever the table does."""

    def __init__(self):
        self.version = 0
        self.reset(None)

    def reset(self, key):
        self.key, self.offset, self.cols, self.pairs = key, 0, None, {}
        self.version += 1

    def __call__(self, path, cycle_of):
        try:
            st = os.stat(path)
        except OSError:
            if self.key is not None:
                self.reset(None)
            return self
        if self.key != (st.st_dev, st.st_ino) or st.st_size < self.offset:
            self.reset((st.st_dev, st.st_ino))
        if st.st_size == self.offset:
            return self
        with open(path, "rb") as f:
            f.seek(self.offset)
            while True:
                chunk = f.read(1 << 22)
                if not chunk:
                    break
                end = chunk.rfind(b"\n")
                if end < 0:
                    break  # a partial line: wait for the rest
                for line in chunk[:end].decode("utf-8", "replace").split("\n"):
                    field = line.rstrip("\r").split(",")
                    if self.cols is None:
                        names = ("tournament", "white", "black", "result")
                        self.cols = [field.index(k) for k in names] if set(names) <= set(field) else [0, 1, 2, 3]
                        continue
                    try:
                        t, w, b, res = (field[i] for i in self.cols)
                        t = int(t)
                    except (IndexError, ValueError):
                        continue
                    k = 0 if res == "1-0" else 1 if res == "0-1" else 2 if res.startswith("1/2") else None
                    if k is None or w == b:
                        continue
                    if w > b:  # stored in the pair's sorted order
                        w, b, k = b, w, (1 - k if k < 2 else 2)
                    self.pairs.setdefault((w, b), {}).setdefault(cycle_of(t), [0, 0, 0])[k] += 1
                self.offset += end + 1
                f.seek(self.offset)
        self.version += 1
        return self


count_pairs = PairCounter()
_h2h_cache = {}


def cycle_of_fn(d):
    """Round-robin index -> cycle. Each logged cycle played the round-robins its summary counts, in order; later
    ones belong to a logged cycle whose summary isn't written yet, else to the cycle under way (between cycles, they
    follow config.json's count)."""
    bounds, t0, last, unsummed = [], 0, 0, None
    for h in d.history:
        n = h.get("round_robins") or 0
        if n:
            bounds.append((t0, t0 + n, h["cycle"]))
            t0 += n
        elif h["cycle"] >= 1 and not h.get("summary") and unsummed is None:
            unsummed = h["cycle"]
        last = max(last, h["cycle"])
    cur = d.cur.get("cycle") if d.cur else None
    nxt = unsummed if unsummed is not None else cur

    def f(t):
        for a, b, c in bounds:
            if a <= t < b:
                return c
        return nxt if nxt is not None else last + 1 + (t - t0) // max(1, d.rr)
    return f


def h2h_pairs(d):
    return count_pairs(os.path.join(d.league, "games.csv"), cycle_of_fn(d))


def h2h_version(d):
    sig = lambda p: (lambda s: f"{s.st_mtime_ns}:{s.st_size}")(os.stat(p)) if os.path.exists(p) else "-"
    pc = h2h_pairs(d)
    cur = f"{d.cur.get('cycle')}-{d.cur.get('phase')}" if d.cur else "-"
    return (f"{pc.version}.{sig(os.path.join(d.league, 'ratings.csv'))}.{sig(os.path.join(d.league, 'players.txt'))}"
            f".{cur}")


def is_ref(d, name):
    return name in d.refs or name.startswith("random")


def scope_cycle(scope):
    """The cycle of a one-cycle scope ("c3"), or None."""
    m = re.fullmatch(r"c(\d+)", scope or "")
    return int(m.group(1)) if m else None


def h2h_cycles(d):
    """The cycles with games, newest first."""
    return sorted({c for by in h2h_pairs(d).pairs.values() for c in by}, reverse=True)


def h2h_partial(d, c):
    """Whether cycle c's tournament is still under way: (games so far, about how many it plays), or None."""
    if not d.cur or d.cur.get("cycle") != c or d.cur.get("phase") not in ("self-play", "tournament"):
        return None
    n = sum(sum(by[c]) for by in h2h_pairs(d).pairs.values() if c in by)
    p = len(d.players)
    return n, d.rr * p * (p - 1)


def h2h_key(d, name, scope):
    """The row a player's games count in: itself (current field), or its lineage at its ply (all cycles)."""
    if scope != "lineage" or is_ref(d, name):
        return name
    lin, v, ply = d.split_entry(name)
    return f"{lin}@{ply}" if v is not None else name


def h2h_short(d, key):
    """A column label: the lineage without its -c and the ply; the random players as they are."""
    if is_ref(d, key):
        return key
    if "@" in key:
        lin, ply = key.rsplit("@", 1)
    else:
        lin, v, ply = d.split_entry(key)
        if v is None:
            return key
    return f"{lin[:-2] if lin.endswith('-c') else lin}@{ply}"


def expected(ra, rb):
    return 1.0 / (1.0 + 10.0 ** ((rb - ra) / 400.0))


def standings_order(d):
    active = [n for n, _, _ in d.players]
    rated = sorted([n for n in active if n in d.ratings], key=lambda n: -d.ratings[n]["elo"])
    return rated + [n for n in active if n not in d.ratings]


def h2h_rows(d, scope):
    """The crosstable's players, in Standings order (all cycles: each lineage@ply where its current entry stands,
    then any other in name order; one cycle: the players of its tournament by their ratings in the current fit)."""
    c = scope_cycle(scope)
    if c is not None:
        names = {x for (a, b), by in h2h_pairs(d).pairs.items() if c in by for x in (a, b)}
        rated = sorted((n for n in names if n in d.ratings), key=lambda n: (-d.ratings[n]["elo"], n))
        return rated + sorted(n for n in names if n not in d.ratings)
    order, seen = [], set()
    for n in standings_order(d):
        k = h2h_key(d, n, scope)
        if k not in seen:
            seen.add(k)
            order.append(k)
    if scope == "lineage":
        rest = set()
        for a, b in h2h_pairs(d).pairs:
            rest.update((h2h_key(d, a, scope), h2h_key(d, b, scope)))
        order += sorted(rest - seen)
    return order


def h2h_cells(d, scope, keys):
    """(row, column) -> [wins, losses, draws, expected-score sum, games with an expected score], from the row's
    side, over all cycles (one cycle's only, in a one-cycle scope); only pairs among `keys`."""
    want, cells, only = set(keys), {}, scope_cycle(scope)
    for (a, b), by in h2h_pairs(d).pairs.items():
        if only is not None:
            if only not in by:
                continue
            by = {only: by[only]}
        ka, kb = h2h_key(d, a, scope), h2h_key(d, b, scope)
        if ka == kb or ka not in want or kb not in want:
            continue
        w = sum(x[0] for x in by.values())
        l = sum(x[1] for x in by.values())
        dr = sum(x[2] for x in by.values())
        ra, rb = d.ratings.get(a), d.ratings.get(b)
        e = expected(ra["elo"], rb["elo"]) if ra and rb else None
        n = w + l + dr
        for key, (x, y, s) in (((ka, kb), (w, l, e)), ((kb, ka), (l, w, None if e is None else 1 - e))):
            c = cells.setdefault(key, [0, 0, 0, 0.0, 0])
            c[0] += x
            c[1] += y
            c[2] += dr
            if s is not None:
                c[3] += s * n
                c[4] += n
    return cells


def score_class(s):
    dv = s - 0.5
    m = abs(dv)
    step = 0 if m <= 0.02 else 1 if m <= 0.08 else 2 if m <= 0.20 else 3 if m <= 0.35 else 4
    return "s-z" if step == 0 else f"s-{'p' if dv > 0 else 'n'}{step}"


def wld(w, l, dr):
    return f"+{w}−{l}={dr}"


H2H_SCALE = [("s-n4", "<15%"), ("s-n3", "15–30"), ("s-n2", "30–42"), ("s-n1", "42–48"), ("s-z", "48–52"),
             ("s-p1", "52–58"), ("s-p2", "58–70"), ("s-p3", "70–85"), ("s-p4", ">85%")]


def h2h_live(d):
    """The cycle whose tournament is under way and has games, or None."""
    c = d.cur.get("cycle") if d.cur else None
    return c if c is not None and h2h_partial(d, c) and c in h2h_cycles(d) else None


def h2h_meta(d):
    """For the page script: the cycles its selector offers, and the one still under way."""
    live = h2h_live(d)
    return (f'<span class="h2h-meta" hidden data-cycles="{",".join(map(str, h2h_cycles(d)))}" '
            f'data-partial="{"" if live is None else live}"></span>')


def h2h_table(d, scope):
    keys = h2h_rows(d, scope)
    if not h2h_pairs(d).pairs:
        return '<p class="note">No games yet: the first appear during cycle 1\'s tournament.</p>' + h2h_meta(d)
    only = scope_cycle(scope)
    if only is not None and not keys:
        return f'<p class="note">Cycle {only} has no games.</p>' + h2h_meta(d)
    cells = h2h_cells(d, scope, keys)
    keys = [k for k in keys if any((k, j) in cells for j in keys)] or keys
    head = "".join(f'<th class="ch" title="{esc(k)}"><span>{esc(h2h_short(d, k))}</span></th>' for k in keys)
    part = h2h_partial(d, only) if only is not None else None
    out = [h2h_meta(d)]
    if part:
        out.append(f'<p class="note h2h-partial"><b>Partial:</b> cycle {only}\'s tournament is under way, '
                   f'{part[0]:,} of about {part[1]:,} games played so far.</p>')
    out += ['<div class="h2h-scale"><span class="lbl">Cell colour, the row player\'s score:</span>'
            + "".join(f'<span class="{c}">{esc(t)}</span>' for c, t in H2H_SCALE) + "</div>",
           f'<div class="h2h-wrap"><table class="h2h"><thead><tr><th class="corner">row vs column</th>{head}</tr>'
           '</thead><tbody>']
    for i in keys:
        tds = []
        for j in keys:
            c = cells.get((i, j))
            if i == j:
                tds.append('<td class="dg"></td>')
            elif not c or not (c[0] + c[1] + c[2]):
                tds.append("<td></td>")
            else:
                w, l, dr, es, en = c
                s = (w + dr / 2) / (w + l + dr)
                e = round(1000 * es / en) if en else -1
                tds.append(f'<td class="{score_class(s)}" data-h="{w} {l} {dr} {e}">{wld(w, l, dr)}</td>')
        tds = "".join(tds)
        out.append(f'<tr><th><button type="button" data-p="{esc(i)}">{esc(i)}</button></th>{tds}</tr>')
    out.append("</tbody></table></div>")
    what = ("each row and column is a lineage at one ply, all its versions together (retired ones included), and "
            "each random player" if scope == "lineage" else
            f"the players of cycle {only}'s tournament (that cycle's versions at plies 1 and 2, and the reference "
            f"players then active), by their ratings in the current fit, over cycle {only}'s games only"
            if only is not None else
            "each row and column is an entry of the current field, over all the games the two have played")
    out.append(f'<p class="note">{len(keys)} players: {what}. A cell is the row player\'s +wins −losses =draws '
               "against the column player; blank: never met. Hover a cell for the games, the score and the expected "
               "score from the current fit (" + ("averaged over the games, each by the two versions' ratings"
                                                 if scope == "lineage" else "by the two ratings")
               + "); click a row's name for its results by cycle" + (" (all its games)" if only is not None else "")
               + "; the factor filters at the top hide the rows and columns of the entries they leave out"
               + ".</p>")
    return "".join(out)


def h2h_player(d, scope, name):
    """One player's results against each opponent, by cycle (its scope's rows and opponents)."""
    keys = h2h_rows(d, scope)
    pos = {k: i for i, k in enumerate(keys)}
    per, cyc = {}, set()
    for (a, b), by in h2h_pairs(d).pairs.items():
        ka, kb = h2h_key(d, a, scope), h2h_key(d, b, scope)
        if ka == kb or name not in (ka, kb):
            continue
        me_a = ka == name
        opp = kb if me_a else ka
        ra, rb = d.ratings.get(a), d.ratings.get(b)
        e = expected(ra["elo"], rb["elo"]) if ra and rb else None
        if e is not None and not me_a:
            e = 1 - e
        rec = per.setdefault(opp, {"by": {}, "es": 0.0, "en": 0})
        for c, (x, y, dr) in by.items():
            w, l = (x, y) if me_a else (y, x)
            t = rec["by"].setdefault(c, [0, 0, 0])
            t[0] += w
            t[1] += l
            t[2] += dr
            cyc.add(c)
            if e is not None:
                rec["es"] += e * (w + l + dr)
                rec["en"] += w + l + dr
    if not per:
        return f'<p class="note">{esc(name)} has no games.</p>'
    cycles = sorted(cyc)
    opps = sorted(per, key=lambda k: (pos.get(k, len(pos)), -d.ratings[k]["elo"] if k in d.ratings else 0, k))
    head = "".join(f'<th class="r">cycle {c}</th>' for c in cycles)
    out = [f'<h3>{esc(name)}: results by cycle <button type="button" class="h2h-close">close</button></h3>'
           f'<div class="h2h-wrap h2hp-wrap"><table class="h2hp"><thead><tr><th>opponent</th>{head}'
           '<th class="r">total</th><th class="r">games</th><th class="r">score</th><th class="r">expected</th>'
           '<th class="r">over / under</th></tr></thead><tbody>']
    tw = tl = td_ = 0
    tes = ten = 0.0
    for o in opps:
        rec = per[o]
        w = sum(x[0] for x in rec["by"].values())
        l = sum(x[1] for x in rec["by"].values())
        dr = sum(x[2] for x in rec["by"].values())
        n = w + l + dr
        s = (w + dr / 2) / n
        tw, tl, td_ = tw + w, tl + l, td_ + dr
        tes, ten = tes + rec["es"], ten + rec["en"]
        e = rec["es"] / rec["en"] if rec["en"] else None
        cols = "".join(f"<td class='r'>{wld(*rec['by'][c]) if c in rec['by'] else ''}</td>" for c in cycles)
        ex = f"{100 * e:.1f}%" if e is not None else "—"
        ou = plain(100 * (s - e), 1) if e is not None else "—"
        out.append(f"<tr><td>{esc(o)}</td>{cols}<td class='r'><span class='chip {score_class(s)}'>{wld(w, l, dr)}"
                   f"</span></td><td class='r'>{n}</td><td class='r'>{100 * s:.1f}%</td><td class='r'>{ex}</td>"
                   f"<td class='r'>{ou}</td></tr>")
    n = tw + tl + td_
    s = (tw + td_ / 2) / n
    e = tes / ten if ten else None
    tot = "".join(f"<td class='r'>{wld(*[sum(per[o]['by'].get(c, [0, 0, 0])[i] for o in opps) for i in range(3)])}"
                  "</td>" for c in cycles)
    out.append(f"<tr class='tot'><td>all opponents</td>{tot}<td class='r'>{wld(tw, tl, td_)}</td><td class='r'>{n}</td>"
               f"<td class='r'>{100 * s:.1f}%</td><td class='r'>{f'{100 * e:.1f}%' if e is not None else '—'}</td>"
               f"<td class='r'>{plain(100 * (s - e), 1) if e is not None else '—'}</td></tr>")
    out.append("</tbody></table></div><p class=\"note\">Each cell is this player's +wins −losses =draws against the "
               "opponent in that cycle's tournament. Over / under: the score minus the expected score from the "
               "current fit, in percentage points.</p>")
    return "".join(out)


def h2h_fragment(config_path, scope, name=None):
    scope = scope if scope in ("lineage", "field") or scope_cycle(scope) is not None else "field"
    with LOCK:
        d = Data(config_path)
        v = h2h_version(d)
        key = (scope, name, v)
        if key not in _h2h_cache:
            if len(_h2h_cache) > 64:
                _h2h_cache.clear()
            _h2h_cache[key] = h2h_player(d, scope, name) if name else h2h_table(d, scope)
        return _h2h_cache[key]


def panel_h2h(d):
    live = h2h_live(d)
    opts = "".join(f'<option value="c{c}">Cycle {c}{" (partial)" if c == live else ""}</option>' for c in h2h_cycles(d))
    return (f'<div id="h2h" data-keep="1" data-v="{esc(h2h_version(d))}"><div class="h2h-bar"><span>Scope:</span>'
            '<button type="button" data-scope="field" aria-pressed="true">Current field (entries)</button>'
            f'<select data-cycsel aria-label="One cycle\'s tournament"><option value="">One cycle…</option>{opts}'
            '</select>'
            '<button type="button" data-scope="lineage" aria-pressed="false">All cycles (lineage@ply)</button></div>'
            '<div class="h2h-table"><p class="note">Loading…</p></div><div id="h2h-player"></div>'
            '<noscript><p class="note">The head-to-head tables need JavaScript.</p></noscript></div>')


# ---------------------------------------------------------------- panel 1a: Elo by cycle, every lineage the filters show

def ref_joins(d):
    """[(cycle, [reference players])] for the reference players whose first games came after the first cycle's,
    with the cycle they joined in: the Elo charts mark it, as the cycles' fits are linked better from then on."""
    first = {}
    for (a, b), by in h2h_pairs(d).pairs.items():
        for x in (a, b):
            if is_ref(d, x) and by:
                first[x] = min(first.get(x, min(by)), min(by))
    if not first:
        return []
    start = min(c for by in h2h_pairs(d).pairs.values() for c in by)
    out = {}
    for n, c in first.items():
        if c > start:
            out.setdefault(c, []).append(n)
    return sorted((c, sorted(ns)) for c, ns in out.items())


def join_note(d):
    joins = ref_joins(d)
    if not joins:
        return ""
    what = "; ".join(f"{', '.join(ns)} in cycle {c}" for c, ns in joins)
    return (f" A dashed line marks reference players joining ({esc(what)}): they play every cycle from then on, "
            "which links the cycles' ratings better, so the level can step there.")


def elo_cmp(d):
    """The chart at the top: a kept container that the page script fills (controls, chart, key, the lineages' list,
    table view), and the data as JSON. Per lineage and ply, a point per cycle it played (see cycle_points): [cycle,
    version, Elo, ± (null when not kept), live, first cycle of that version, that version's Elo, ± and games in today's
    fit (null if unrated), whether it is in the field, that cycle's decision]; pk, each lineage's peak per ply
    ([cycle, version, Elo], as fx_scores, Standings' Peak Elo); fz, a frozen lineage's [the cycle it froze from
    (frozen_since), the plies of the entries it kept]."""
    pts = {}
    for lin, per in cycle_points(d).items():
        if lin not in d.lin:
            continue
        for p, s in per.items():
            pts.setdefault(lin, {})[str(p)] = [
                [pt["c"], pt["v"], round(pt["elo"], 1), None if pt["ci"] is None else round(pt["ci"], 1),
                 int(pt["live"]), pt["since"], None if pt["today"] is None else round(pt["today"], 1),
                 None if pt["today_ci"] is None else round(pt["today_ci"], 1), pt["today_games"], int(pt["field"]),
                 pt["decision"]] for pt in s]
    shown = d.registry + [l for l in d.former if l["name"] in pts]  # retired ones keep their history
    pk = {lin: {p: [x[3], x[2], x[0]] for p, x in sc["peak"].items()} for lin, sc in fx_scores(d).items()
          if lin in pts and sc["peak"]}
    fz = {}  # a frozen lineage: [the cycle it froze from, the plies of the entries it kept (state.json)]
    for l in shown:
        c = None if is_retired(d, l["name"]) else frozen_since(d, l["name"])
        if c is not None:
            st = (d.state.get("lineages") or {}).get(l["name"]) or {}
            kept = sorted({str(d.split_entry(e)[2]) for e in st.get("entries") or []
                           if isinstance(e, str) and d.split_entry(e)[2] is not None})
            fz[l["name"]] = [c, kept]
    data = dict(ply=d.qualify_ply, pk=pk, fz=fz,
                lins=[[l["name"], shape_of(d, l["name"]), status_of(d, l["name"])]
                      + (["retired"] if is_retired(d, l["name"]) else []) for l in shown],
                pts=pts,
                refs=[[n, round(d.ratings[n]["elo"], 1), round(d.ratings[n]["ci"], 1)] for n in d.refs if n in d.ratings],
                joins=[[c, ", ".join(ns)] for c, ns in ref_joins(d)])
    js = json.dumps(data, separators=(",", ":")).replace("</", "<\\/")
    return ('<div id="bc" data-keep="1" data-v="1"><p class="note">Loading the chart…</p>'
            '<noscript><p class="note">The chart needs JavaScript; the small multiples in Elo over cycles, by '
            'archetype, and their table view don\'t.</p></noscript></div>'
            f'<script type="application/json" id="elo-data">{js}</script>')


def panel_chart(d):
    return (elo_cmp(d) + '<p class="note">One line per lineage that the factor filters in the bar at the top show, '
            'coloured by its value of the factor picked there (Colour and group by); the list under the chart groups '
            'them by it. A line runs through the lineage\'s rating in every cycle it played, as measured that cycle: a '
            'finished cycle\'s at its decision (cycles.csv; at 1-ply the league\'s Elo history, which keeps no ±), '
            'the cycle under way\'s live from today\'s fit once its tournament has begun, as in Elo over cycles '
            'below. A version that keeps playing gets a point every cycle, so a dormant or frozen lineage\'s line '
            'moves as the league re-rates it; a frozen one\'s is dotted from the cycle it froze (at the ply of the '
            'entry it kept; the other entry retired then, so its line ends with ×), and a retired one\'s ends with × at '
            'its last cycle. A ring marks each line\'s peak, its highest Elo at a completed '
            'cycle as in Standings (Peaks hides them). Click a line, its name in the list, or the lineage anywhere on '
            'the page (a Standings, lineages-table or gate row, a small multiple) to highlight it: drawn thick, with '
            'its points, 95% whiskers and an end label, the others faded; click again to un-highlight, or use the '
            'bar at the top. Point at the chart, or focus it and use the arrow keys (up and down change lines, Enter '
            'highlights), for a point\'s version, Elo ± and the cycle\'s decision. 2-ply at first; ticking one '
            'league play depth in the bar sets the chart\'s too. The ply, the peaks and the reference lines stay in '
            'this browser, as do the filters and the highlight.' + join_note(d) + '</p>')


FX_CSS = r"""
.fxbar{position:sticky;top:0;z-index:6;margin:8px -24px 0;padding:5px 24px 6px;background:var(--plane);
 border-bottom:1px solid var(--ring);font-size:12.5px;color:var(--ink2)}
.fxbar:empty{display:none}
.fxb-row{display:flex;flex-wrap:wrap;gap:4px 12px;align-items:center}
.fxb-tog{display:inline-flex;align-items:center;gap:5px;font:inherit;font-size:12.5px;font-weight:600;
 border:1px solid var(--axis);background:var(--surface);color:var(--ink);border-radius:6px;padding:2px 10px 2px 8px;
 min-height:26px;cursor:pointer}
.fxb-tog.on{border-color:var(--ink)}
.fxb-tog:focus-visible{outline:2px solid var(--ink2);outline-offset:2px}
.fxb-car{display:inline-block;width:10px;color:var(--ink2)}
.fxb-cg{display:inline-flex;flex-wrap:wrap;gap:4px 10px;align-items:center;margin-left:auto}
.fxbar select,.fxc-bar select{font:inherit;font-size:12.5px;border:1px solid var(--axis);background:var(--surface);
 color:var(--ink);border-radius:6px;padding:2px 6px;min-height:26px;cursor:pointer}
.fx-bl{font-weight:600}
.fx-key,.fx-act{display:inline-flex;flex-wrap:wrap;gap:2px 10px;align-items:center}
.fx-key>span{display:inline-flex;align-items:center;gap:4px;color:var(--ink)}
.fx-act .fx-n{color:var(--ink);font-weight:600}
#fx.fx-chips{display:grid;grid-template-columns:repeat(auto-fill,minmax(500px,1fr));gap:0 28px;margin:6px 0 0;
 max-height:40vh;overflow:auto}
#fx.fx-chips[hidden]{display:none}
#fx .fx-row{padding:2px 0}
#fx .fx-row.fx-entry{border-top:0}
#fx .fx-k{width:150px;font-size:12px}
#fx .fx-chip{min-height:22px;font-size:12px;padding:0 9px}
.fx-pill{display:inline-flex;align-items:center;gap:2px;border:1px solid var(--axis);border-radius:12px;
 padding:0 2px 0 8px;background:var(--surface);color:var(--ink)}
.fx-pill button,.fx-clr,.fx-clrall{font:inherit;font-size:12px;border:0;background:none;color:var(--ink2);cursor:pointer;
 padding:1px 6px;border-radius:10px}
.fx-pill button:hover,.fx-clr:hover,.fx-clrall:hover{background:var(--grid);color:var(--ink)}
.fx-dot{display:inline-block;width:10px;height:10px;border-radius:50%;flex:none;vertical-align:middle;
 background:var(--lc,var(--muted))}
.fx-dot[hidden]{display:none}
#fx{margin:0 0 6px}
.fx-row{display:flex;flex-wrap:wrap;gap:4px 6px;align-items:center;padding:4px 0;border-bottom:1px solid var(--grid)}
.fx-row.fx-entry{border-top:1px dashed var(--axis)}
.fx-k{width:210px;flex:none;font-size:12.5px;font-weight:600;color:var(--ink2)}
.fx-chip{display:inline-flex;align-items:center;gap:5px;font:inherit;font-size:12.5px;border:1px solid var(--axis);
 border-radius:14px;padding:1px 10px;min-height:26px;background:var(--surface);color:var(--ink);cursor:pointer}
.fx-chip:hover{border-color:var(--ink2)}
.fx-chip[aria-pressed="true"]{background:var(--ink);color:var(--surface);border-color:var(--ink)}
.fx-chip .n{color:var(--muted);font-variant-numeric:tabular-nums;font-size:11.5px}
.fx-chip[aria-pressed="true"] .n{color:var(--surface);opacity:.75}
.fx-chip.zero{opacity:.45}
.fx-chip:focus-visible,.fx-clr:focus-visible,.fx-clrall:focus-visible,.fx-pill button:focus-visible{
 outline:2px solid var(--ink2);outline-offset:2px}
.fx-foot{grid-column:1/-1;font-size:12px;color:var(--muted);margin:4px 0 0}
.fx-out{display:none!important}
table.standings th.fxh{white-space:normal;max-width:68px;line-height:1.2;vertical-align:bottom;cursor:help}
table.standings td.fxv{font-size:12.5px;white-space:nowrap}
table.standings tr:not(.ret) td.fxv.muted{color:var(--muted)}
table.standings th.fx-by,table.standings tr:not(.ret) td.fx-by{color:var(--ink);font-weight:600}
.stf .stf-n.on,.lt-n.on{color:var(--ink);font-weight:600}
.lt-bar{display:flex;flex-wrap:wrap;gap:6px 12px;align-items:center;margin:0 0 6px;font-size:12.5px;color:var(--ink2)}
.legend .fxl-h{font-weight:600;color:var(--ink2)}
.legend.fxl[hidden]{display:none}
table.gates tr.gfam .fx-dot{margin-right:1px}
.fxc-bar{display:flex;flex-wrap:wrap;gap:6px;align-items:center;margin:0 0 8px;font-size:12.5px;color:var(--ink2)}
.fxc-bar button{font:inherit;font-size:12.5px;border:1px solid var(--axis);background:var(--surface);color:var(--ink);
 border-radius:6px;padding:2px 10px;min-height:26px;cursor:pointer}
.fxc-bar button[aria-pressed="true"]{background:var(--ink);color:var(--surface);border-color:var(--ink)}
table.fxc{font-size:12.5px}
table.fxc th{vertical-align:bottom;white-space:nowrap}
table.fxc td{vertical-align:top;white-space:normal;padding:5px 8px}
table.fxc td.cfg{max-width:240px;color:var(--ink)}
table.fxc td.cfg .muted{font-size:11.5px;margin-top:2px}
table.fxc td.cell{min-width:170px}
.fxc-l{display:flex;flex-wrap:wrap;align-items:center;gap:0 6px;line-height:1.45}
.fxc-l b{font-weight:600}
.fxc-e{font-variant-numeric:tabular-nums;color:var(--ink2)}
.fxc-t{font-size:11.5px;color:var(--ink2);background:var(--plane);border:1px solid var(--grid);border-radius:4px;
 padding:0 4px}
.fxc-s{font-size:11.5px;color:var(--muted);font-style:italic}
table.fxc details summary{margin:2px 0;font-size:12px}
td.fxc-d{min-width:210px}
td.fxc-d>div{margin:0 0 4px;font-variant-numeric:tabular-nums}
.fxc-d .pm,.fxc-d .pr{color:var(--muted);font-size:11.5px}
.fxc-d .pr{white-space:nowrap}
.fxc-d .also{display:block;font-size:11.5px;color:var(--ink2)}
table.lintab th{vertical-align:bottom;max-width:110px}
table.lintab th .sortb{text-align:left}
table.lintab td.lt-w{white-space:normal;min-width:110px;max-width:180px}
table.lintab td.muted,table.lintab tr.ret td{color:var(--muted)}
#app>section[id]{scroll-margin-top:calc(var(--fxbar-h,48px) + 8px)}
.fxb-hi{margin-top:5px}
.fxb-hi[hidden]{display:none}
.fxb-hp{padding-left:6px;gap:5px}
table.standings tr.fx-hi>td,table.lintab tr.fx-hi>td,#elo-table tr.fx-hi>td{
 background:color-mix(in srgb,var(--lc,var(--muted)) 14%,var(--surface))}
table.gates tr.fx-hi>th{background:color-mix(in srgb,var(--lc,var(--muted)) 18%,var(--surface))}
table.standings tr.fx-hi>td:first-child,table.lintab tr.fx-hi>td:first-child,#elo-table tr.fx-hi>td:first-child,
table.gates tr.fx-hi>th{box-shadow:inset 3px 0 0 var(--lc,var(--ink2))}
table.standings tr[data-lc],table.lintab tbody tr[data-lc],#elo-table tr[data-lc],table.gates tr[data-lc]>th{cursor:pointer}
.mult [data-s],.mult figcaption,.mult .items .li{cursor:pointer}
.mult g.fx-hi path:not([stroke="transparent"]){stroke-width:3.5}
.mult .items .li.fx-hi{color:var(--ink);font-weight:600}
.fxc-l{cursor:pointer;border-radius:4px}
.fxc-l.fx-hi{background:color-mix(in srgb,var(--lc,var(--muted)) 14%,transparent);box-shadow:inset 3px 0 0 var(--lc)}
"""

FX_JS = r"""
// ---------------------------------------------------------------- facets: filter, colour and group by factor
// (first in the script: the Standings and the chart read this state as they start). open: whether the bar at the
// top shows the filter chips; hi: the highlighted lineages
var FXS = {on: {}, by: null, open: true, hi: []};
try { Object.assign(FXS, JSON.parse(localStorage.getItem('cycles.fx') || '{}')); } catch (err) {}
if (!FXS.on || typeof FXS.on !== 'object' || Array.isArray(FXS.on)) FXS.on = {};
FXS.hi = Array.isArray(FXS.hi) ? [...new Set(FXS.hi.filter(x => typeof x === 'string'))] : [];
delete FXS.sec;  // the Standings' sections, gone: one table ranked by Elo
delete FXS.ec;  // the old comparison chart's colour pick
var FXD = null, FXT = null, FXL = new Map(), FXI = {}, fxCmpSig = null;
var FXC = {f: 'depth', ref: '', m: 'peak', ply: ''};
try { Object.assign(FXC, JSON.parse(localStorage.getItem('cycles.fxcmp') || '{}')); } catch (err) {}
var LT = {col: '', dir: 1};
try { Object.assign(LT, JSON.parse(localStorage.getItem('cycles.lintab') || '{}')); } catch (err) {}
function fxSave() { try { localStorage.setItem('cycles.fx', JSON.stringify(FXS)); } catch (err) {} }
function fxLoad() {  // read the facets the page carries; true when they changed
  const src = document.getElementById('fx-data');
  if (!src || src.textContent === FXT) return false;
  let data;
  try { data = JSON.parse(src.textContent); } catch (err) { return false; }
  FXD = data;
  FXT = src.textContent;
  FXL = new Map(FXD.lins.map(l => [l[0], l]));
  FXI = {};
  FXD.factors.forEach((f, i) => { FXI[f[0]] = i; });
  if (FXI[FXS.by] === undefined) FXS.by = FXD.by;
  return true;
}
fxLoad();
function fxF(k) { return FXD && FXI[k] !== undefined ? FXD.factors[FXI[k]] : null; }
function fxVal(n, k) { const l = FXL.get(n); return l && FXI[k] !== undefined ? l[1][FXI[k]] : 'unclassified'; }
function fxColour(k, v) { const f = fxF(k), i = f ? f[2].indexOf(v) : -1; return i >= 0 ? f[4][i] : 'var(--muted)'; }
function fxLabelOf(k, v) { const f = fxF(k), i = f ? f[2].indexOf(v) : -1; return i >= 0 ? f[5][i] : v; }
function fxLinColour(n) { return fxColour(FXS.by, fxVal(n, FXS.by)); }
function fxActive() {  // {factor: Set of ticked values} for the factors with ticks the data still has; ply: the entries'
  const out = {};
  if (!FXD) return out;
  FXD.factors.forEach(f => {
    const on = (FXS.on[f[0]] || []).filter(v => f[2].includes(v));
    if (on.length) out[f[0]] = new Set(on);
  });
  const p = (FXS.on.ply || []).filter(v => FXD.plies.includes(v));
  if (p.length) out.ply = new Set(p);
  return out;
}
function fxPass(n, act, skip) {  // whether lineage n shows: for each factor with ticks (but skip), its value is ticked
  act = act || fxActive();
  for (const k in act) if (k !== skip && k !== 'ply' && !act[k].has(fxVal(n, k))) return false;
  return true;
}
function fxPassEntry(n, ply, act) {  // an entry of lineage n at that ply (n null: a random player, always shown)
  act = act || fxActive();
  return n === null || (fxPass(n, act) && (!act.ply || ply == null || act.ply.has(String(ply))));
}
function fxKey(key) {  // a head-to-head row's [lineage, ply]: an entry (LIN-vN-P) or a lineage at a ply (LIN@P)
  const m = /^(.+)@(\d+)$/.exec(key) || /^(.+)-v\d+-(\d+)$/.exec(key);
  return m ? [m[1], m[2]] : [FXL.has(key) ? key : null, null];
}
function fxElo(x) { const r = Math.round(x); return (r < 0 ? '−' : '') + Math.abs(r); }
function fxWords(d) { const r = Math.round(Math.abs(d)); return r === 0 ? 'level' : r + (d > 0 ? ' above' : ' below'); }
function fxDesc(act, skipPly) {  // the filters in force, in words
  act = act || fxActive();
  const rows = FXD ? fxRows() : [];
  return Object.keys(act).filter(k => !(skipPly && k === 'ply')).map(k => {
    const r = rows.find(x => x.k === k);
    return r ? r.label + ': ' + [...act[k]].map(v => r.labels[r.vals.indexOf(v)]).join(' or ') : k;
  }).join('; ');
}

// the highlight: lineages picked out in the chart at the top by a click on a line, a name in its list, or the lineage
// anywhere on the page (a Standings, gate or lineages-table row, a small multiple); tinted wherever they appear
function fxHiList() { return FXS.hi.filter(n => !FXD || FXL.has(n)); }
function fxHiSet(list) {
  FXS.hi = [...new Set(list)];
  fxSave();
  fxHiMark();
  bcDraw();
}
function fxHiToggle(n) { const h = fxHiList(); fxHiSet(h.includes(n) ? h.filter(x => x !== n) : h.concat([n])); }
function fxHiAdd(ns) { fxHiSet(fxHiList().concat(ns)); }
function fxHiClear() { fxHiSet([]); }
function fxHiMark() {  // the rows and marks of the highlighted lineages, and the bar's list of them
  const hi = new Set(fxHiList());
  document.querySelectorAll('[data-lc]').forEach(x => x.classList.toggle('fx-hi', hi.has(x.dataset.lc)));
  const row = document.querySelector('#fxbar .fxb-hi');
  if (!row) return;
  row.textContent = '';
  row.hidden = !hi.size;
  if (!hi.size) return;
  const list = [...hi], MAX = 10;
  row.append(ecEl('span', {class: 'fx-bl'}, 'Highlighted (' + list.length + ')'));
  list.slice(0, MAX).forEach(n => {
    const p = ecEl('span', {class: 'fx-pill fxb-hp'}), dot = ecEl('span', {class: 'fx-dot', 'aria-hidden': 'true'});
    dot.style.background = fxLinColour(n);
    p.append(dot, document.createTextNode(n + (fxPass(n) ? '' : ' (the filters hide it)')),
             ecEl('button', {type: 'button', 'data-fxhi': n, title: 'Un-highlight ' + n, 'aria-label': 'Un-highlight ' + n}, '✕'));
    row.append(p);
  });
  if (list.length > MAX) row.append(ecEl('span', {class: 'muted'}, 'and ' + (list.length - MAX) + ' more'));
  row.append(ecEl('button', {type: 'button', class: 'fx-clrall', 'data-fxhigo': '1'}, 'show in the chart'),
             ecEl('button', {type: 'button', class: 'fx-clrall', 'data-fxhiclr': '1'}, 'clear the highlight'));
}

// the chips: a row per factor, a toggle per value with the lineages it would show given the other rows' ticks
function fxRows() {
  const rows = FXD.factors.map(f => ({k: f[0], label: f[1], vals: f[2], labels: f[5]}));
  rows.push({k: 'ply', label: 'League play depth (entries)', vals: FXD.plies, labels: FXD.plies.map(p => 'ply ' + p),
             entry: true});
  return rows;
}
function fxBuild() {
  const box = document.getElementById('fx');
  if (!box || !FXD) return;
  const rows = fxRows(), key = JSON.stringify(rows.map(r => [r.k, r.vals, r.labels]));
  if (box.dataset.built === key) return;
  box.textContent = '';
  rows.forEach(r => {
    const row = ecEl('div', {class: 'fx-row' + (r.entry ? ' fx-entry' : ''), role: 'group', 'aria-label': r.label});
    row.append(ecEl('span', {class: 'fx-k'}, r.label));
    r.vals.forEach((v, i) => {
      const b = ecEl('button', {type: 'button', class: 'fx-chip', 'data-fxk': r.k, 'data-fxv': v, 'aria-pressed': 'false'});
      b.append(ecEl('span', {class: 'fx-dot', 'aria-hidden': 'true'}), ecEl('span', {}, r.labels[i]), ecEl('span', {class: 'n'}));
      row.append(b);
    });
    row.append(ecEl('button', {type: 'button', class: 'fx-clr', 'data-fxclr': r.k, hidden: ''}, 'clear'));
    box.append(row);
  });
  box.append(ecEl('div', {class: 'fx-foot'}, 'Values in a row add up (or), rows narrow down (and); a count is how many '
                  + 'lineages the value would show with the other rows’ ticks. League play depth picks entries, not '
                  + 'lineages (one ticked sets the chart’s ply); the random players always stay.'));
  box.dataset.built = key;
}
function fxSync(act) {
  const box = document.getElementById('fx');
  if (box && FXD) {
    box.querySelectorAll('.fx-chip').forEach(b => {
      const k = b.dataset.fxk, v = b.dataset.fxv, on = !!(act[k] && act[k].has(v)), dot = b.querySelector('.fx-dot');
      b.setAttribute('aria-pressed', String(on));
      if (k === 'ply') {
        b.title = (on ? 'Ticked: ' : '') + 'the entries that play at ' + v + '-ply, in Standings and head to head';
      } else {
        const n = FXD.lins.filter(l => l[1][FXI[k]] === v && fxPass(l[0], act, k)).length;
        b.querySelector('.n').textContent = n;
        b.classList.toggle('zero', !n && !on);
        b.title = (on ? 'Ticked: ' : '') + n + ' lineage' + (n !== 1 ? 's' : '') + ' with this value'
                  + (Object.keys(act).some(x => x !== k) ? ' among those the other ticks show' : '');
      }
      dot.hidden = k !== FXS.by;
      dot.style.background = k === FXS.by ? fxColour(k, v) : '';
    });
    box.querySelectorAll('[data-fxclr]').forEach(c => {
      c.hidden = c.dataset.fxclr === '*' ? !Object.keys(act).length : !act[c.dataset.fxclr];
    });
  }
  fxBar(act);
}
// the bar that stays at the top: the filters (a toggle opens the chips), the ones in force, colour and group by with
// its key; the page's anchors scroll to just below it
function fxBarBuild() {
  const bar = document.getElementById('fxbar');
  if (!bar || !FXD) return;
  const key = JSON.stringify(FXD.factors.map(f => [f[0], f[1]]));
  if (bar.dataset.built === key) return;
  bar.textContent = '';
  const top = ecEl('div', {class: 'fxb-row'});
  const tog = ecEl('button', {type: 'button', class: 'fxb-tog', 'data-fxopen': '1', 'aria-controls': 'fx',
                              'aria-expanded': 'true', title: 'Show or hide the filter chips'});
  tog.append(ecEl('span', {class: 'fxb-car', 'aria-hidden': 'true'}), ecEl('span', {class: 'fxb-tl'}, 'Filters'));
  const sel = ecEl('select', {'data-fxby': '1', 'aria-label': 'Colour and group by'});
  FXD.factors.forEach(f => sel.append(new Option(f[1], f[0])));
  const cg = ecEl('span', {class: 'fxb-cg'});
  cg.append(ecEl('span', {class: 'fx-bl', title: 'Colours every lineage\'s marks by its value of this factor, and '
                                                  + 'groups the gate rows and the chart\'s list of lineages'},
                 'Colour and group by'), sel, ecEl('span', {class: 'fx-key'}));
  top.append(tog, ecEl('span', {class: 'fx-act'}), cg);
  bar.append(top, ecEl('div', {class: 'fxb-row fxb-hi', hidden: ''}), ecEl('div', {id: 'fx', class: 'fx-chips'}));
  bar.dataset.built = key;
  if (!bar.dataset.ro && window.ResizeObserver) {  // anchors land below the bar, whatever its height
    bar.dataset.ro = '1';
    new ResizeObserver(() => document.documentElement.style.setProperty('--fxbar-h', bar.offsetHeight + 'px')).observe(bar);
  }
}
function fxBar(act) {
  const bar = document.getElementById('fxbar');
  if (!bar || !FXD || !bar.dataset.built) return;
  bar.querySelector('[data-fxby]').value = FXS.by;
  const tog = bar.querySelector('[data-fxopen]'), nf = Object.keys(act).length;
  tog.setAttribute('aria-expanded', String(!!FXS.open));
  tog.querySelector('.fxb-car').textContent = FXS.open ? '▾' : '▸';
  tog.querySelector('.fxb-tl').textContent = 'Filters' + (nf ? ' (' + nf + ')' : '');
  tog.classList.toggle('on', nf > 0);
  const chips = document.getElementById('fx');
  if (chips) chips.hidden = !FXS.open;
  const keyEl = bar.querySelector('.fx-key'), f = fxF(FXS.by), greys = [];
  keyEl.textContent = '';
  f[2].forEach((v, i) => {
    if (f[4][i] === 'var(--muted)') { greys.push(f[5][i]); return; }
    const s = ecEl('span'), dot = ecEl('span', {class: 'fx-dot'});
    dot.style.background = f[4][i];
    s.append(dot, document.createTextNode(f[5][i]));
    keyEl.append(s);
  });
  if (greys.length) {
    const s = ecEl('span'), dot = ecEl('span', {class: 'fx-dot'});
    dot.style.background = 'var(--muted)';
    s.append(dot, document.createTextNode(greys.join(', ')));
    keyEl.append(s);
  }
  const out = bar.querySelector('.fx-act'), ks = Object.keys(act);
  out.textContent = '';
  if (!ks.length) { out.append(ecEl('span', {class: 'muted'}, 'no filter: all ' + FXD.lins.length + ' lineages')); return; }
  const rows = fxRows();
  ks.forEach(k => {
    const r = rows.find(x => x.k === k), p = ecEl('span', {class: 'fx-pill'});
    p.append(document.createTextNode(r.label + ': ' + [...act[k]].map(v => r.labels[r.vals.indexOf(v)]).join(', ')),
             ecEl('button', {type: 'button', 'data-fxclr': k, title: 'Clear this filter', 'aria-label': 'Clear the ' + r.label + ' filter'}, '✕'));
    out.append(p);
  });
  const n = FXD.lins.filter(l => fxPass(l[0], act)).length;
  out.append(ecEl('b', {class: 'fx-n'}, n + ' of ' + FXD.lins.length + ' lineages'),
             ecEl('button', {type: 'button', class: 'fx-clrall', 'data-fxclr': '*'}, 'clear all'));
}
document.addEventListener('click', e => {
  const t = e.target.closest ? e.target : e.target.parentElement;
  if (!t) return;
  const chip = t.closest('.fx-chip');
  if (chip) {
    const k = chip.dataset.fxk, v = chip.dataset.fxv, cur = new Set(FXS.on[k] || []);
    if (cur.has(v)) cur.delete(v); else cur.add(v);
    if (cur.size) FXS.on[k] = [...cur]; else delete FXS.on[k];
    fxSave();
    fxApply();
    return;
  }
  const clr = t.closest('[data-fxclr]');
  if (clr) {
    if (clr.dataset.fxclr === '*') FXS.on = {}; else delete FXS.on[clr.dataset.fxclr];
    fxSave();
    fxApply();
    return;
  }
  if (t.closest('[data-fxopen]')) { FXS.open = !FXS.open; fxSave(); fxBar(fxActive()); return; }
  const hb = t.closest('[data-fxhi], [data-fxhigo], [data-fxhiclr]');
  if (hb) {
    if (hb.dataset.fxhi) fxHiToggle(hb.dataset.fxhi);
    else if (hb.dataset.fxhigo) bcReveal();
    else fxHiClear();
    return;
  }
  // a lineage anywhere else: a small multiple's series or name highlights it and goes to the chart (a panel's title
  // takes all of its lineages the filters show); a row or mark elsewhere toggles its highlight. Not on controls, nor
  // in the bar, the chart or head to head (their own clicks), nor when the click ends a text selection
  if (t.closest('a, button, input, select, textarea, summary, label, #fxbar, #bc, #h2h')) return;
  if (String(window.getSelection ? window.getSelection() : '')) return;
  const m = t.closest('.mult');
  if (m) {
    const sr = t.closest('[data-s]');
    if (sr) { fxHiAdd([sr.dataset.s]); bcReveal(); return; }
    if (t.closest('figcaption')) {
      const ns = [...new Set([...m.querySelectorAll('[data-s]')].map(x => x.dataset.s))].filter(n => fxPass(n));
      if (ns.length) { fxHiAdd(ns); bcReveal(); }
    }
    return;
  }
  const lc = t.closest('[data-lc]');
  if (lc) fxHiToggle(lc.dataset.lc);
});
document.addEventListener('keydown', e => {  // a small multiple's name, focused: Enter or Space as a click
  if (e.key !== 'Enter' && e.key !== ' ') return;
  const li = e.target.closest && e.target.closest('.mult .li[data-s]');
  if (!li) return;
  e.preventDefault();
  fxHiAdd([li.dataset.s]);
  bcReveal();
});
document.addEventListener('change', e => {
  const t = e.target;
  if (!t.matches) return;
  if (t.matches('[data-fxby]')) { FXS.by = t.value; fxSave(); fxApply(); }
  else if (t.matches('#fxcmp [data-fxcf]')) { FXC.f = t.value; FXC.ref = ''; fxCmpSave(); fxCmp(); }
  else if (t.matches('#fxcmp [data-fxcref]')) { FXC.ref = t.value; fxCmpSave(); fxCmp(); }
});

// everything the facets touch: colours, legends, the small multiples, Standings, gates, head to head, the lineages
// table, the factor comparison, the highlight and the chart
function fxApply() {
  if (!FXD) return;
  const act = fxActive();
  fxSync(act);
  document.querySelectorAll('[data-lc]').forEach(x => x.style.setProperty('--lc', fxLinColour(x.dataset.lc)));
  document.querySelectorAll('.fxl[data-fx]').forEach(x => { x.hidden = x.dataset.fx !== FXS.by; });
  document.querySelectorAll('.mult').forEach(m => {
    let any = false;
    m.querySelectorAll('[data-s]').forEach(x => {
      const ok = fxPass(x.dataset.s, act);
      x.classList.toggle('fx-out', !ok);
      any = any || ok;
    });
    m.classList.toggle('fx-out', !any);
  });
  document.querySelectorAll('#elo-table tr[data-lc]').forEach(r => r.classList.toggle('fx-out', !fxPass(r.dataset.lc, act)));
  document.querySelectorAll('table.standings [data-fxk]').forEach(c => c.classList.toggle('fx-by', c.dataset.fxk === FXS.by));
  stSort();
  fxGates(act);
  fxH2h(act);
  fxLinTab(act);
  fxCmp();
  fxHiMark();
  bcRefresh();
}
function fxRefresh() { fxLoad(); fxBarBuild(); fxBuild(); fxApply(); }

// the gates: the lineages' rows in a group per value of the factor the page groups by, then the fixed groups
// (retired, no longer in lineages.txt); a group the filters empty hides
function fxGates(act) {
  const t = document.querySelector('table.gates');
  if (!t || !FXD) return;
  const body = t.tBodies[0], k = FXS.by, f = fxF(k);
  body.querySelectorAll('tr.fx-h').forEach(r => r.remove());
  const rows = [...body.querySelectorAll('tr[data-lc]:not(.gfx)')], groups = new Map();
  rows.forEach(r => { const v = fxVal(r.dataset.lc, k); if (!groups.has(v)) groups.set(v, []); groups.get(v).push(r); });
  const at = body.querySelector('tr.gfam'), ncol = t.tHead.rows[0].cells.length;
  const pos = v => { const i = f[2].indexOf(v); return i < 0 ? 1e9 : i; };
  [...groups.keys()].sort((a, b) => pos(a) - pos(b)).forEach(v => {
    const h = ecEl('tr', {class: 'gfam fx-h'}), th = ecEl('th', {colspan: String(ncol)}), s = ecEl('span', {class: 'gfl'});
    const dot = ecEl('span', {class: 'fx-dot'});
    dot.style.background = fxColour(k, v);
    s.append(dot, document.createTextNode(' ' + f[1] + ': ' + fxLabelOf(k, v)));
    th.append(s);
    h.append(th);
    body.insertBefore(h, at);
    groups.get(v).forEach(r => body.insertBefore(r, at));
  });
  let head = null, n = 0;
  const close = () => { if (head) head.classList.toggle('fx-out', !n); };
  [...body.rows].forEach(r => {
    if (r.classList.contains('gfam')) { close(); head = r; n = 0; return; }
    if (!r.dataset.lc) return;
    const ok = fxPass(r.dataset.lc, act);
    r.classList.toggle('fx-out', !ok);
    if (ok) n++;
  });
  close();
}
// head to head: the rows and columns of the entries the filters leave out
function fxH2h(act) {
  const t = document.querySelector('#h2h table.h2h');
  if (!t || !FXD) return;
  act = act || fxActive();
  const show = key => { const [n, p] = fxKey(key); return fxPassEntry(n, p, act); };
  const head = t.tHead.rows[0], cols = [...head.cells].map((th, i) => i === 0 || show(th.title));
  [...head.cells].forEach((th, i) => th.classList.toggle('fx-out', !cols[i]));
  [...t.tBodies[0].rows].forEach(tr => {
    const b = tr.querySelector('button[data-p]'), on = !b || show(b.dataset.p);
    tr.classList.toggle('fx-out', !on);
    if (on) [...tr.cells].forEach((c, i) => c.classList.toggle('fx-out', !cols[i]));
  });
}
// the lineages table: sorted by a column (numbers high first, then reversed, then lineages.txt's order), filtered
function fxLinTab(act) {
  const t = document.querySelector('table.lintab');
  if (!t) return;
  act = act || fxActive();
  const body = t.tBodies[0], rows = [...body.rows], hs = [...t.tHead.rows[0].cells];
  const ci = LT.col ? hs.findIndex(c => c.dataset.lcol === LT.col) : -1, num = ci >= 0 && hs[ci].dataset.num === '1';
  const val = r => (r.cells[ci] ? r.cells[ci].dataset.v || '' : '');
  rows.sort((a, b) => {
    if (ci >= 0) {
      const x = val(a), y = val(b);
      if ((x === '') !== (y === '')) return x === '' ? 1 : -1;
      const c = num ? (+x) - (+y) : x.localeCompare(y, undefined, {numeric: true});
      if (c) return LT.dir * c;
    }
    return a.dataset.i - b.dataset.i;
  });
  rows.forEach(r => body.appendChild(r));
  let n = 0;
  rows.forEach(r => { const ok = fxPass(r.dataset.lc, act); r.classList.toggle('fx-out', !ok); if (ok) n++; });
  hs.forEach(c => {
    const on = !!LT.col && c.dataset.lcol === LT.col, a = c.querySelector('.sarr'), b = c.querySelector('button');
    c.setAttribute('aria-sort', on ? (LT.dir > 0 ? 'ascending' : 'descending') : 'none');
    if (b) b.setAttribute('aria-pressed', String(on));
    if (a) a.textContent = on ? (LT.dir > 0 ? ' ▲' : ' ▼') : '';
  });
  const c = document.querySelector('.lt-n'), f = fxDesc(act, true);
  if (c) {
    c.textContent = n === rows.length && !f ? 'All ' + n + ' lineages'
                    : 'Showing ' + n + ' of ' + rows.length + ' lineages' + (f ? ' (' + f + ')' : '');
    c.classList.toggle('on', n !== rows.length);
  }
}
document.addEventListener('click', e => {
  const b = e.target.closest ? e.target.closest('table.lintab button[data-lsort]') : null;
  if (!b) return;
  const col = b.dataset.lsort, first = b.closest('th').dataset.num === '1' ? -1 : 1;
  if (LT.col !== col) { LT.col = col; LT.dir = first; }
  else if (LT.dir === first) LT.dir = -first;
  else { LT.col = ''; LT.dir = 1; }
  try { localStorage.setItem('cycles.lintab', JSON.stringify(LT)); } catch (err) {}
  fxLinTab();
});

// the factor comparison: rows of lineages that agree on every other factor, a column per value of the one compared
function fxCmpSave() { try { localStorage.setItem('cycles.fxcmp', JSON.stringify(FXC)); } catch (err) {} }
function fxCmpPly() { return FXD.plies.includes(FXC.ply) ? FXC.ply : FXD.ply; }
function fxScore(l, ply) { return (FXC.m === 'now' ? l[7] : l[6])[ply || fxCmpPly()] || null; }
function fxM(l) { const s = fxScore(l); return s ? s[0] : null; }
function fxMci(l) { const s = fxScore(l); return s && s[1] != null ? s[1] : null; }
function fxCmpRows(k) {
  const act = fxActive(), linked = (FXD.linked || {})[k] || [];
  const keyF = FXD.factors.map(f => f[0]).filter(g => g !== k && !linked.includes(g));
  const rows = new Map(), skipped = [];
  FXD.lins.forEach((l, i) => {
    if (!fxPass(l[0], act)) return;
    const v = l[1][FXI[k]], key = keyF.map(g => l[1][FXI[g]]);
    if (v === 'unclassified' || key.includes('unclassified')) { skipped.push(l[0]); return; }
    const ks = key.join('\u0001');
    if (!rows.has(ks)) rows.set(ks, {key, cells: new Map(), first: i});
    const r = rows.get(ks);
    if (!r.cells.has(v)) r.cells.set(v, []);
    r.cells.get(v).push(l);
  });
  return {keyF, linked, rows: [...rows.values()].filter(r => r.cells.size > 1).sort((a, b) => a.first - b.first), skipped};
}
function fxDiffs(a, b, linked) {  // what else differs between lineages a and b: the factors let vary, then the variant
  const out = [], other = q => q.startsWith('--') || q.startsWith('no --') || q === 'stray values';
  linked.forEach(g => {
    const x = a[1][FXI[g]], y = b[1][FXI[g]];
    if (x !== y) out.push({t: fxF(g)[1].toLowerCase() + ' ' + x + ' vs ' + y,
                           shown: !['none', 'unclassified'].includes(x) && !['none', 'unclassified'].includes(y)});
  });
  const ka = a[3], kb = b[3];
  Object.keys(ka).forEach(q => {
    if (q in kb) { if (ka[q] !== kb[q]) out.push({t: q + ' ' + ka[q] + ' vs ' + kb[q], shown: true}); }
    else if (other(q)) out.push({t: (q + ' ' + ka[q]).trim() + ' (' + a[0] + ' only)', shown: true});
  });
  Object.keys(kb).forEach(q => {
    if (!(q in ka) && other(q)) out.push({t: (q + ' ' + kb[q]).trim() + ' (' + b[0] + ' only)', shown: true});
  });
  return out;
}
function fxPair(as, bs, linked) {  // the pair to compare: the fewest other differences, then the best of each cell
  let best = null;
  as.forEach(a => bs.forEach(b => {
    const ma = fxM(a), mb = fxM(b);
    if (ma == null || mb == null || a === b) return;
    const diff = fxDiffs(a, b, linked), s = [diff.length, -ma, -mb];
    if (!best || s[0] < best.s[0] || (s[0] === best.s[0] && (s[1] < best.s[1] || (s[1] === best.s[1] && s[2] < best.s[2]))))
      best = {a, b, diff, s};
  }));
  return best;
}
function fxCfg(r, keyF) {  // the rest of a row's configuration, in words
  const m = {};
  keyF.forEach((g, i) => { m[g] = r.key[i]; });
  const out = [];
  if (m.arch !== undefined) out.push(fxLabelOf('arch', m.arch));
  if (m.leaf !== undefined) out.push('leaf ' + fxLabelOf('leaf', m.leaf));
  if (m.router !== undefined && m.router !== 'none') out.push(m.router === 'gate (soft)' ? 'soft gate' : 'router ' + m.router);
  if (m.split !== undefined && (m.split !== 'none' || ['T', 'D'].includes(m.arch))) out.push('split ' + m.split);
  if (m.depth !== undefined) out.push('self-play depth ' + m.depth);
  if (m.labels !== undefined) out.push(m.labels + ' labels');
  if (m.policy !== undefined) out.push(m.policy);
  return out.join(' · ');
}
function fxCell(ls, ply) {
  const td = ecEl('td', {class: 'cell'}), key = l => { const x = fxM(l); return x == null ? -1e9 : x; };
  const sorted = ls.slice().sort((a, b) => key(b) - key(a));
  const add = (into, l) => {
    const pk = l[6][ply], nw = l[7][ply], d = ecEl('div', {class: 'fxc-l', 'data-lc': l[0]});
    d.style.setProperty('--lc', fxLinColour(l[0]));
    d.append(ecEl('span', {class: 'fx-dot'}), ecEl('b', {}, l[0]),
             ecEl('span', {class: 'fxc-e'}, 'peak ' + (pk ? fxElo(pk[0]) : '—') + ' · now ' + (nw ? fxElo(nw[0]) : '—')));
    if (l[2]) d.append(ecEl('span', {class: 'fxc-t'}, l[2]));
    if (!/^qualified/.test(l[4])) d.append(ecEl('span', {class: 'fxc-s'}, l[4]));
    d.title = l[0] + ' (' + l[8] + ')\npeak at ' + ply + '-ply: ' + (pk ? fxElo(pk[0]) + (pk[1] != null ? ' ± ' + Math.round(pk[1]) : '')
              + ', v' + pk[2] + ' in cycle ' + pk[3] : 'no completed cycle yet') + '\nnow: '
              + (nw ? fxElo(nw[0]) + ' ± ' + Math.round(nw[1]) + ', ' + nw[2] + ', ' + nw[3] + ' games' : 'not rated')
              + '\nstatus: ' + l[4] + (l[2] ? '\nvariant: ' + l[2] : '');
    into.append(d);
  };
  const top = sorted.length > 4 ? sorted.slice(0, 3) : sorted;
  top.forEach(l => add(td, l));
  if (sorted.length > top.length) {
    const det = ecEl('details');
    det.append(ecEl('summary', {}, 'and ' + (sorted.length - top.length) + ' more'));
    sorted.slice(top.length).forEach(l => add(det, l));
    td.append(det);
  }
  return td;
}
function fxCmp() {
  const box = document.getElementById('fxcmp');
  if (!box || !FXD) return;
  if (FXI[FXC.f] === undefined) FXC.f = 'depth';
  const fkey = JSON.stringify([FXD.factors.map(f => [f[0], f[1]]), FXD.plies]);
  if (box.dataset.built !== fkey) {
    box.textContent = '';
    const bar = ecEl('div', {class: 'fxc-bar'}), fs = ecEl('select', {'data-fxcf': '1', 'aria-label': 'The factor to compare'});
    FXD.factors.forEach(f => fs.append(new Option(f[1], f[0])));
    bar.append(ecEl('span', {}, 'Factor'), fs, ecEl('span', {}, 'against'),
               ecEl('select', {'data-fxcref': '1', 'aria-label': 'The value the differences are against'}),
               ecEl('span', {class: 'ec-gap'}), ecEl('span', {}, 'by'));
    [['peak', 'peak Elo'], ['now', 'Elo now']].forEach(([m, t]) =>
      bar.append(ecEl('button', {type: 'button', 'data-fxcm': m, 'aria-pressed': 'false'}, t)));
    bar.append(ecEl('span', {}, 'at'));
    FXD.plies.slice().reverse().forEach(p =>
      bar.append(ecEl('button', {type: 'button', 'data-fxcp': p, 'aria-pressed': 'false'}, p + '-ply')));
    box.append(bar, ecEl('div', {class: 'fxc-out'}));
    box.dataset.built = fkey;
    fxCmpSig = null;
  }
  const sig = FXT + '\u0002' + JSON.stringify([FXC, FXS.on, FXS.by]);
  if (sig === fxCmpSig) return;
  fxCmpSig = sig;
  const k = FXC.f, f = fxF(k), ply = fxCmpPly(), res = fxCmpRows(k), rows = res.rows;
  box.querySelector('[data-fxcf]').value = k;
  box.querySelectorAll('[data-fxcm]').forEach(b => b.setAttribute('aria-pressed', String(b.dataset.fxcm === FXC.m)));
  box.querySelectorAll('[data-fxcp]').forEach(b => b.setAttribute('aria-pressed', String(b.dataset.fxcp === ply)));
  const cols = f[2].filter(v => rows.some(r => r.cells.has(v)));
  const ref = cols.includes(FXC.ref) ? FXC.ref : cols.includes(f[3]) ? f[3] : cols[0];
  const rs = box.querySelector('[data-fxcref]');
  rs.textContent = '';
  cols.forEach(v => rs.append(new Option(fxLabelOf(k, v) + (v === f[3] ? ' (default)' : ''), v)));
  rs.value = ref || '';
  rs.disabled = !cols.length;
  const out = box.querySelector('.fxc-out'), what = (FXC.m === 'now' ? 'Elo now' : 'peak Elo') + ' at ' + ply + '-ply';
  out.textContent = '';
  if (!rows.length) {
    out.append(ecEl('p', {class: 'note'}, 'No two lineages' + (Object.keys(fxActive()).length ? ' that the filters show' : '')
                    + ' differ in ' + f[1].toLowerCase() + ' alone' + (res.linked.length ? ' (with what varies with it)' : '') + '.'));
    return;
  }
  const t = ecEl('table', {class: 'fxc'}), thead = ecEl('thead'), hr = ecEl('tr'), tb = ecEl('tbody'), sums = new Map();
  hr.append(ecEl('th', {}, 'the rest of the configuration'));
  cols.forEach(v => {
    const th = ecEl('th');
    if (k === FXS.by) { const dot = ecEl('span', {class: 'fx-dot'}); dot.style.background = fxColour(k, v); th.append(dot, document.createTextNode(' ')); }
    th.append(document.createTextNode(fxLabelOf(k, v) + (v === ref ? ' (reference)' : '')));
    hr.append(th);
  });
  hr.append(ecEl('th', {}, 'difference against ' + fxLabelOf(k, ref) + ', ' + what));
  thead.append(hr);
  rows.forEach(r => {
    const tr = ecEl('tr'), cfg = ecEl('td', {class: 'cfg'});
    cfg.append(document.createTextNode(fxCfg(r, res.keyF)),
               ecEl('div', {class: 'muted'}, [...new Set([...r.cells.values()].flat().map(l => l[8]))].join(', ')));
    tr.append(cfg);
    cols.forEach(v => tr.append(fxCell(r.cells.get(v) || [], ply)));
    const base = r.cells.has(ref) ? ref : cols.find(v => r.cells.has(v)), dtd = ecEl('td', {class: 'fxc-d'});
    cols.filter(v => v !== base && r.cells.has(v)).forEach(v => {
      const p = fxPair(r.cells.get(v), r.cells.get(base), res.linked), line = ecEl('div');
      const lead = cols.length > 2 || base !== ref ? fxLabelOf(k, v) + ' against ' + fxLabelOf(k, base) + ': ' : '';
      if (!p) { line.append(ecEl('span', {class: 'muted'}, lead + 'not rated at ' + ply + '-ply yet')); dtd.append(line); return; }
      const dd = fxM(p.a) - fxM(p.b), ca = fxMci(p.a), cb = fxMci(p.b), pm = ca != null && cb != null ? Math.hypot(ca, cb) : null;
      line.append(document.createTextNode(lead), ecEl('b', {}, fxWords(dd)));
      if (pm != null) line.append(ecEl('span', {class: 'pm'}, ' ± ' + Math.round(pm)));
      if (r.cells.get(v).length > 1 || r.cells.get(base).length > 1) line.append(ecEl('span', {class: 'pr'}, ' ' + p.a[0] + ' vs ' + p.b[0]));
      const also = p.diff.filter(x => x.shown).map(x => x.t);
      if (also.length) line.append(ecEl('span', {class: 'also'}, 'also differs: ' + also.join('; ')));
      line.title = p.a[0] + ': ' + fxElo(fxM(p.a)) + (ca != null ? ' ± ' + Math.round(ca) : '') + '\n' + p.b[0] + ': '
                   + fxElo(fxM(p.b)) + (cb != null ? ' ± ' + Math.round(cb) : '') + '\ndifference ' + fxElo(dd)
                   + (pm != null ? ' ± ' + Math.round(pm) + ' (95%, the two ±s combined as if independent)' : '');
      dtd.append(line);
      if (base === ref) { if (!sums.has(v)) sums.set(v, []); sums.get(v).push(dd); }
    });
    tr.append(dtd);
    tb.append(tr);
  });
  t.append(thead, tb);
  const wrap = ecEl('div', {class: 'tscroll'});
  wrap.append(t);
  out.append(wrap);
  const parts = [];
  cols.filter(v => v !== ref && sums.has(v)).forEach(v => {
    const ds = sums.get(v).slice().sort((a, b) => a - b), n = ds.length;
    const med = n % 2 ? ds[(n - 1) / 2] : (ds[n / 2 - 1] + ds[n / 2]) / 2;
    const up = ds.filter(x => Math.round(x) > 0).length, down = ds.filter(x => Math.round(x) < 0).length;
    parts.push(fxLabelOf(k, v) + ' against ' + fxLabelOf(k, ref) + ', ' + n + ' row' + (n > 1 ? 's' : '') + ': above in ' + up
               + ', below in ' + down + (n - up - down ? ', level in ' + (n - up - down) : '') + '; median ' + fxWords(med));
  });
  if (parts.length) {
    const s = ecEl('p', {class: 'note'});
    s.append(ecEl('b', {}, 'Summary, by ' + what + ': '), document.createTextNode(parts.join('. ') + '.'));
    out.append(s);
  }
  if (res.skipped.length) out.append(ecEl('p', {class: 'note'}, 'Left out as unclassified: ' + res.skipped.join(', ') + '.'));
}
document.addEventListener('click', e => {
  const b = e.target.closest ? e.target.closest('#fxcmp button[data-fxcm], #fxcmp button[data-fxcp]') : null;
  if (!b) return;
  if (b.dataset.fxcm) FXC.m = b.dataset.fxcm; else FXC.ply = b.dataset.fxcp;
  fxCmpSave();
  fxCmp();
});
"""

EC_CSS = r"""

.viz-root{--k1:#2a78d6;--k2:#eb6834;--k3:#1baf7a;--k4:#eda100;--k5:#e87ba4;--k6:#008300;--k7:#6250d6;--k8:#e34948}
@media (prefers-color-scheme:dark){:root:where(:not([data-theme="light"])) .viz-root{--k1:#3987e5;--k2:#d95926;
 --k3:#199e70;--k4:#c98500;--k5:#d55181;--k6:#008300;--k7:#9085e9;--k8:#e66767}}
#bc{margin:0 0 4px}
.ec-bar{display:flex;flex-wrap:wrap;gap:6px;align-items:center;margin:0 0 8px;font-size:12.5px;color:var(--ink2)}
.ec-bar button{font:inherit;font-size:12.5px;border:1px solid var(--axis);background:var(--surface);color:var(--ink);
 border-radius:6px;padding:2px 10px;min-height:26px;cursor:pointer}
.ec-bar button[aria-pressed="true"]{background:var(--ink);color:var(--surface);border-color:var(--ink)}
.ec-bar button[hidden]{display:none}
.ec-bar button:disabled{cursor:default;opacity:.55}
.ec-bar .bc-n.on{color:var(--ink);font-weight:600}
.ec-gap{width:10px}
.bc-hin{color:var(--ink);font-weight:600}
.bc-n{margin-left:auto;color:var(--ink2)}
.ec-k{flex:none;vertical-align:middle}
.ec-chart{min-height:40px}.ec-chart svg{display:block}
.ec-chart svg:focus{outline:none}.ec-chart svg:focus-visible{outline:2px solid var(--ink2);outline-offset:2px}
.bc-s .ln{stroke-width:1.25;stroke-opacity:.85}
.bc-s.on .ln,.bc-s.hov .ln{stroke-opacity:1}
.bc-s.on .ln{stroke-width:3.25}
.bc-s.hov .ln{stroke-width:2.75}
.bc-s .ln.fz{stroke-dasharray:.1 3.6}
.bc-s.hov .ln.fz{stroke-dasharray:.1 5.5}
.bc-s.on .ln.fz{stroke-dasharray:.1 6.5}
.bc-s .rt{stroke-width:1.75;fill:none}
.bc-s .pk{fill:var(--surface);stroke-width:1.5}
.bc-s.on .pk{stroke-width:2.25}
.ec-chart svg.bc-hl .bc-s:not(.on):not(.hov){opacity:.13}
.ec-chart svg.bc-hv .bc-s:not(.hov):not(.on),.ec-chart svg.bc-hv .ec-lab:not(.hov){opacity:.25}
.ec-chart svg.bc-hl.bc-hv .bc-s.on:not(.hov){opacity:.55}
.bc-ov{cursor:pointer}
.ec-lab text{fill:var(--ink);font-size:12px;paint-order:stroke;stroke:var(--surface);stroke-width:3px;
 stroke-linejoin:round}
.ec-lab text .ec-lv{fill:var(--ink2)}
.ec-reflab{fill:var(--muted);paint-order:stroke;stroke:var(--surface);stroke-width:3px;stroke-linejoin:round}
.bc-key{display:flex;flex-wrap:wrap;gap:4px 18px;font-size:12px;color:var(--ink2);margin:4px 0 6px}
.bc-key span{display:inline-flex;align-items:center;gap:5px}
.bc-legend{margin:2px 0 6px}
.bc-g{margin:0 0 6px}
.bc-gh{display:flex;align-items:center;gap:6px;font-size:12px;font-weight:600;color:var(--ink2);margin:4px 0 1px}
.bc-gl{display:grid;grid-template-columns:repeat(auto-fill,minmax(230px,1fr));gap:1px 12px}
.bc-li{display:flex;align-items:center;gap:6px;font:inherit;font-size:12.5px;text-align:left;border:1px solid transparent;
 border-radius:4px;background:none;color:var(--ink);padding:1px 6px;min-height:24px;cursor:pointer;min-width:0}
.bc-li:hover{background:var(--plane)}
.bc-li[aria-pressed="true"]{border-color:var(--ink2);background:var(--plane);font-weight:600}
.bc-li:focus-visible{outline:2px solid var(--ink2);outline-offset:1px}
.bc-li .bc-ln{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.bc-li .bc-e{margin-left:auto;color:var(--muted);font-variant-numeric:tabular-nums;font-weight:400}
.bc-li .bc-t{font-size:11px;color:var(--muted);font-weight:400}
.ec-tbl table{font-size:12px}
#ectip{position:fixed;z-index:9;pointer-events:none;background:var(--surface);color:var(--ink);border:1px solid var(--ring);
 border-radius:6px;padding:6px 8px;font-size:12.5px;max-width:440px;box-shadow:0 2px 8px rgba(0,0,0,.15)}
#ectip .v{font-size:14px;font-weight:600;font-variant-numeric:tabular-nums}
#ectip .m{color:var(--ink2)}
#ectip .bc-tn{display:flex;align-items:center;gap:6px;font-size:13px}
#ectip .bc-hint{margin-top:4px;font-size:11.5px;color:var(--muted)}
"""

EC_JS = r"""

// ---------------------------------------------------------------- Elo by cycle: every lineage the filters show
const EC_NS = 'http://www.w3.org/2000/svg';
const BC = {ply: '2', refs: false, peaks: true};
try { Object.assign(BC, JSON.parse(localStorage.getItem('cycles.bigchart') || '{}')); } catch (err) {}
if (!['1', '2'].includes(BC.ply)) BC.ply = '2';
let ecD = null, ecText = null, bcG = null, bcKey = null, bcRaf = 0, bcLins = new Map(), bcHovN = null, bcSig = null;
const ecTip = document.createElement('div');
ecTip.id = 'ectip';
ecTip.hidden = true;
document.querySelector('.viz-root').appendChild(ecTip);
const ecCtx = document.createElement('canvas').getContext('2d');

function bcSave() { try { localStorage.setItem('cycles.bigchart', JSON.stringify(BC)); } catch (err) {} }
function bcPinned() {  // the play depth the filters pin (exactly one ticked), or null
  const a = fxActive();
  return a.ply && a.ply.size === 1 ? [...a.ply][0] : null;
}
function bcPly() { return bcPinned() || BC.ply; }
function ecEl(tag, attrs, text) {
  const e = document.createElement(tag);
  for (const k in attrs || {}) e.setAttribute(k, attrs[k]);
  if (text !== undefined) e.textContent = text;
  return e;
}
function ecSv(tag, attrs, text) {
  const e = document.createElementNS(EC_NS, tag);
  for (const k in attrs || {}) e.setAttribute(k, attrs[k]);
  if (text !== undefined) e.textContent = text;
  return e;
}
function ecNum(x) {
  const r = Math.round(x);
  return (r < 0 ? '−' : '') + Math.abs(r);
}
function ecKeySvg(colour, dot, w) {  // dot: dotted, as a frozen lineage's line from its freeze on
  const k = ecSv('svg', {width: String(w || 18), height: '10', class: 'ec-k', 'aria-hidden': 'true'});
  const ln = ecSv('line', {x1: '2', x2: String((w || 18) - 2), y1: '5', y2: '5', stroke: colour, 'stroke-width': '2.5',
                           'stroke-linecap': 'round'});
  if (dot) ln.setAttribute('stroke-dasharray', '0.1 4.5');
  k.append(ln);
  return k;
}
function ecTicks(lo, hi, n) {
  if (hi <= lo) hi = lo + 1;
  const raw = (hi - lo) / Math.max(1, n), mag = Math.pow(10, Math.floor(Math.log10(raw)));
  const step = [1, 2, 2.5, 5, 10].map(m => m * mag).find(s => s >= raw) || 10 * mag;
  const out = [];
  for (let t = Math.ceil(lo / step) * step; t <= hi + 1e-9; t += step) out.push(Math.round(t * 1000) / 1000);
  return out;
}
// end labels: keep each at its line's end, pushed apart just enough not to overlap
function ecPlace(items, top, bottom, gap) {
  items.sort((a, b) => a.y - b.y);
  let prev = -Infinity;
  items.forEach(it => { it.ly = Math.max(it.y, prev + gap, top); prev = it.ly; });
  let next = Infinity;
  for (let i = items.length - 1; i >= 0; i--) { items[i].ly = Math.min(items[i].ly, next - gap, bottom); next = items[i].ly; }
  prev = -Infinity;
  items.forEach(it => { it.ly = Math.max(it.ly, prev + gap, top); prev = it.ly; });
}
function bcStatus(n) { const l = bcLins.get(n); return l ? l[2] : ''; }
function bcRetired(n) { const l = bcLins.get(n); return !!(l && l[3] === 'retired'); }
function bcFrozen(n) { const c = ecD && ecD.fz ? ecD.fz[n] : null; return c ? c[0] : null; }  // the freeze's cycle
function bcDots(n) {  // the cycle its line is dotted from at this ply: the freeze's, where it kept its entry
  const c = ecD && ecD.fz ? ecD.fz[n] : null;
  return c && (!c[1].length || c[1].includes(bcPly())) ? c[0] : null;
}
function bcPeak(n) { const p = ecD && ecD.pk ? (ecD.pk[n] || {})[bcPly()] : null; return p || null; }
function bcPts(n) { return ((ecD.pts[n] || {})[bcPly()]) || []; }
function bcShown() {  // the lineages the filters show: by the colour factor's values, then in lineages.txt's order
  const f = fxF(FXS.by), pos = v => { const i = f ? f[2].indexOf(v) : -1; return i < 0 ? 1e9 : i; };
  return ecD.lins.map((l, i) => [l[0], i]).filter(x => fxPass(x[0]))
    .sort((a, b) => pos(fxVal(a[0], FXS.by)) - pos(fxVal(b[0], FXS.by)) || a[1] - b[1]).map(x => x[0]);
}
function bcBuild(box) {
  box.textContent = '';
  const bar = ecEl('div', {class: 'ec-bar'});
  bar.append(ecEl('span', {}, 'Ply:'));
  [['2', '2-ply'], ['1', '1-ply']].forEach(([k, t]) =>
    bar.append(ecEl('button', {type: 'button', 'data-bcply': k, 'aria-pressed': 'false'}, t)));
  bar.append(ecEl('span', {class: 'ec-gap'}),
             ecEl('button', {type: 'button', 'data-bcpk': '1', 'aria-pressed': 'false',
                             title: 'A ring on each line at its peak: its highest Elo at a completed cycle, as in Standings'},
                  'Peaks'),
             ecEl('button', {type: 'button', 'data-bcrefs': '1', 'aria-pressed': 'false'}, 'Random players as reference lines'),
             ecEl('span', {class: 'ec-gap'}), ecEl('span', {class: 'bc-hin'}),
             ecEl('button', {type: 'button', 'data-bcclr': '1', hidden: ''}, 'clear the highlight'),
             ecEl('span', {class: 'bc-n'}));
  const tbl = ecEl('details', {class: 'ec-tbl'});
  tbl.append(ecEl('summary', {}, 'Table view of the chart'), ecEl('div', {class: 'ec-tb'}));
  const key = ecEl('div', {class: 'bc-key'});
  const kv = (svg, text) => { const s = ecEl('span'); s.append(svg, document.createTextNode(text)); return s; };
  const sym = (draw) => { const s = ecSv('svg', {width: '22', height: '12', class: 'ec-k', 'aria-hidden': 'true'}); draw(s); return s; };
  key.append(
    kv(sym(s => s.append(ecSv('line', {x1: '2', x2: '20', y1: '6', y2: '6', stroke: 'var(--ink2)', 'stroke-width': '1.5'}))),
       'a lineage’s Elo at each cycle it played'),
    kv(sym(s => s.append(ecSv('line', {x1: '2', x2: '20', y1: '6', y2: '6', stroke: 'var(--ink2)', 'stroke-width': '2',
                                      'stroke-linecap': 'round', 'stroke-dasharray': '0.1 4'}))),
       'dotted from the cycle a lineage froze: its kept version, re-rated'),
    kv(sym(s => s.append(ecSv('path', {d: 'M7,2 L15,10 M7,10 L15,2', stroke: 'var(--ink2)', 'stroke-width': '1.75',
                                      fill: 'none'}))),
       'the end: a retired lineage’s last cycle, or the entry a frozen one retired at its freeze'),
    kv(sym(s => s.append(ecSv('circle', {cx: '11', cy: '6', r: '4', fill: 'var(--surface)', stroke: 'var(--ink2)',
                                        'stroke-width': '1.5'}))), 'peak'),
    kv(sym(s => s.append(ecSv('line', {x1: '2', x2: '20', y1: '6', y2: '6', stroke: 'var(--ink2)', 'stroke-width': '3.5'}))),
       'highlighted (click a line or a name; again to un-highlight)'));
  box.append(bar, ecEl('div', {class: 'ec-chart'}), key, ecEl('div', {class: 'bc-legend'}), tbl);
}
function bcSync(names, drawn) {
  const box = document.getElementById('bc');
  const pin = bcPinned();
  box.querySelectorAll('[data-bcply]').forEach(b => {
    b.setAttribute('aria-pressed', String(b.dataset.bcply === bcPly()));
    b.disabled = !!pin;
    b.title = pin ? 'Set by the League play depth filter in the bar at the top (' + pin + '-ply ticked)' : '';
  });
  box.querySelector('[data-bcpk]').setAttribute('aria-pressed', String(!!BC.peaks));
  box.querySelector('[data-bcrefs]').setAttribute('aria-pressed', String(!!BC.refs));
  const hi = fxHiList(), shown = hi.filter(n => drawn.some(s => s.n === n)), gone = hi.length - shown.length;
  box.querySelector('.bc-hin').textContent = hi.length ? 'Highlighted: ' + hi.length
    + (gone ? ' (' + gone + ' not drawn: the filters or the ply)' : '') : '';
  box.querySelector('[data-bcclr]').hidden = !hi.length;
  const act = fxActive(), f = fxDesc(act, true), all = FXD ? FXD.lins.length : ecD.lins.length;
  const pass = FXD ? FXD.lins.filter(l => fxPass(l[0], act)).length : names.length, miss = pass - drawn.length;
  const n = box.querySelector('.bc-n');
  n.textContent = (f ? pass + ' of ' + all + ' lineages (' + f + ')' : 'all ' + all + ' lineages') + ': ' + drawn.length
                  + ' line' + (drawn.length !== 1 ? 's' : '') + (miss > 0 ? ', ' + miss + ' without a ' + bcPly()
                  + '-ply rating' : '');
  n.classList.toggle('on', !!f);
}
function bcLegend(ser) {
  const box = document.querySelector('#bc .bc-legend');
  const focus = document.activeElement && document.activeElement.closest && document.activeElement.closest('#bc .bc-li');
  const keep = focus ? focus.dataset.ln : null;
  box.textContent = '';
  const f = fxF(FXS.by), groups = new Map();
  ser.forEach(s => { const v = fxVal(s.n, FXS.by); if (!groups.has(v)) groups.set(v, []); groups.get(v).push(s); });
  groups.forEach((ss, v) => {
    const name = (f ? f[1] : 'Value') + ': ' + fxLabelOf(FXS.by, v);
    const g = ecEl('div', {class: 'bc-g', role: 'group', 'aria-label': name}), h = ecEl('div', {class: 'bc-gh'});
    const dot = ecEl('span', {class: 'fx-dot'});
    dot.style.background = fxColour(FXS.by, v);
    h.append(dot, document.createTextNode(name + ' (' + ss.length + ')'));
    const list = ecEl('div', {class: 'bc-gl'});
    ss.forEach(s => {
      const last = s.pts.length ? s.pts[s.pts.length - 1] : null, pk = bcPeak(s.n);
      const b = ecEl('button', {type: 'button', class: 'bc-li', 'data-ln': s.n, 'aria-pressed': String(s.on)});
      b.append(ecKeySvg(s.col, s.fz != null), ecEl('span', {class: 'bc-ln'}, s.n));
      if (s.ret) b.append(ecEl('span', {class: 'bc-t'}, 'retired'));
      else if (s.frz != null) b.append(ecEl('span', {class: 'bc-t'}, 'frozen'));
      b.append(ecEl('span', {class: 'bc-e'}, last ? ecNum(last[2]) : '—'));
      b.title = s.n + ' (' + (bcStatus(s.n) || '—') + ')\n' + (last ? 'latest: ' + ecNum(last[2])
        + (last[3] != null ? ' ± ' + Math.round(last[3]) : '') + ', v' + last[1] + ' in cycle ' + last[0]
        + (last[4] ? ' (under way, live)' : '') : 'no rating at ' + bcPly() + '-ply yet')
        + (pk ? '\npeak: ' + ecNum(pk[2]) + ', v' + pk[1] + ' in cycle ' + pk[0] : '')
        + '\n' + (s.on ? 'click to un-highlight' : 'click to highlight');
      list.append(b);
    });
    g.append(h, list);
    box.append(g);
  });
  if (keep) { const b = box.querySelector('.bc-li[data-ln="' + CSS.escape(keep) + '"]'); if (b) b.focus(); }
}
function bcDraw() {
  const box = document.getElementById('bc');
  if (!box || !ecD || box.dataset.built !== '1') return;
  const chart = box.querySelector('.ec-chart'), hadFocus = bcG && document.activeElement === bcG.svg;
  bcSig = bcSigNow();
  bcHide();
  bcG = null;
  chart.textContent = '';
  const names = bcShown(), hiSet = new Set(fxHiList());
  const ser = names.map(n => ({n, pts: bcPts(n), col: fxLinColour(n), fz: bcDots(n), frz: bcFrozen(n), ret: bcRetired(n),
                                on: hiSet.has(n)}));
  ser.forEach(s => { s.cut = s.ret || (s.frz != null && s.fz == null); });  // its line ends: × at its last point
  const drawn = ser.filter(s => s.pts.length);
  bcSync(names, drawn);
  bcLegend(ser);
  bcTable(drawn);
  if (!drawn.length) {
    chart.append(ecEl('p', {class: 'note'}, names.length ? 'None of the lineages the filters show has a rated version at '
                                                         + bcPly() + '-ply yet.' : 'The factor filters show no lineage.'));
    return;
  }
  // x: every cycle with a rated version at this ply (the axis stays put as the filters change); y: the lines drawn
  let cmin = Infinity, cmax = -Infinity, lo = Infinity, hi = -Infinity;
  Object.values(ecD.pts).forEach(per => (per[bcPly()] || []).forEach(p => { cmin = Math.min(cmin, p[0]); cmax = Math.max(cmax, p[0]); }));
  drawn.forEach(s => s.pts.forEach(p => { lo = Math.min(lo, p[2] - (p[3] || 0)); hi = Math.max(hi, p[2] + (p[3] || 0)); }));
  const refs = BC.refs ? ecD.refs : [], refText = r => r[0] + ' ' + ecNum(r[1]);
  refs.forEach(r => { lo = Math.min(lo, r[1]); hi = Math.max(hi, r[1]); });
  const pad = 0.04 * ((hi - lo) || 100);
  lo -= pad; hi += pad;
  const lastOf = s => s.pts[s.pts.length - 1], hiDrawn = drawn.filter(s => s.on);
  ecCtx.font = '600 12px system-ui, -apple-system, "Segoe UI", sans-serif';
  const labText = s => s.n + '  ' + ecNum(lastOf(s)[2]);
  let R = Math.min(300, Math.max(16, ...hiDrawn.filter(s => lastOf(s)[0] === cmax).map(s => ecCtx.measureText(labText(s)).width + 26)));
  ecCtx.font = '11px system-ui, -apple-system, "Segoe UI", sans-serif';
  refs.forEach(r => { R = Math.max(R, ecCtx.measureText(refText(r)).width + 16); });
  const W = Math.max(480, Math.floor(chart.clientWidth || 900)), H = 470, L = 50, T = 12, B = 40, PR = W - R, PB = H - B;
  const c0 = cmin === cmax ? cmin - 0.5 : cmin - 0.25, c1 = cmin === cmax ? cmax + 0.5 : cmax + 0.25;
  const sx = c => L + (PR - L) * (c - c0) / (c1 - c0), sy = e => T + (PB - T) * (hi - e) / (hi - lo);
  const svg = ecSv('svg', {width: W, height: H, tabindex: '0', class: hiDrawn.length ? 'bc-hl' : '',
    'aria-label': 'Elo by cycle of ' + drawn.length + ' lineages at ' + bcPly() + '-ply. Arrow keys step through a '
                  + 'line’s points (up and down change lines), Enter highlights the line.'});
  ecTicks(lo, hi, 7).forEach(t => {
    svg.append(ecSv('line', {x1: L, x2: PR, y1: sy(t), y2: sy(t), class: 'grid'}),
               ecSv('text', {x: L - 6, y: sy(t) + 3.5, 'text-anchor': 'end'}, t ? ecNum(t) : '0'));
  });
  const every = Math.max(1, Math.ceil((cmax - cmin + 1) / Math.max(2, Math.floor((PR - L) / 40))));
  for (let c = cmin; c <= cmax; c++)
    if ((c - cmin) % every === 0) svg.append(ecSv('text', {x: sx(c), y: PB + 16, 'text-anchor': 'middle'}, String(c)));
  svg.append(ecSv('line', {x1: L, x2: PR, y1: PB, y2: PB, class: 'axisline'}),
             ecSv('text', {x: (L + PR) / 2, y: PB + 33, 'text-anchor': 'middle'}, 'cycle'));
  refs.forEach(r => svg.append(ecSv('line', {x1: L, x2: PR, y1: sy(r[1]), y2: sy(r[1]), class: 'refline'})));
  (ecD.joins || []).filter(j => j[0] > cmin && j[0] <= cmax).forEach(j => {
    const x = (sx(j[0] - 1) + sx(j[0])) / 2, tip = j[1] + ' joined the field in cycle ' + j[0] +
      ': from then on they play every cycle, which links the cycles’ ratings better';
    svg.append(ecSv('line', {x1: x, x2: x, y1: T, y2: PB, class: 'ec-join', 'data-tip': tip}),
               ecSv('text', {x: x + 4, y: T + 9, class: 'ec-join-t'}, j[1] + ' join'));
  });
  const pts = [], ends = [];
  drawn.filter(s => !s.on).concat(hiDrawn).forEach(s => {
    const g = ecSv('g', {class: 'bc-s' + (s.on ? ' on' : ''), 'data-ln': s.n});
    const xy = s.pts.map(p => [sx(p[0]), sy(p[2])]);
    const path = (from, to, dash) => {
      const q = xy.slice(from, to);
      if (q.length < 2) return;
      const e = ecSv('path', {d: q.map((v, i) => (i ? 'L' : 'M') + v[0].toFixed(1) + ',' + v[1].toFixed(1)).join(' '),
        class: 'ln', fill: 'none', stroke: s.col, 'stroke-linejoin': 'round', 'stroke-linecap': 'round'});
      if (dash) e.classList.add('fz');
      g.append(e);
    };
    const fi = s.fz == null ? -1 : s.pts.findIndex(p => p[0] >= s.fz);  // frozen: dotted from the freeze on
    if (fi < 0) path(0, xy.length, false);
    else { path(0, fi + 1, false); path(fi, xy.length, true); }
    if (xy.length === 1) g.append(ecSv('circle', {cx: xy[0][0], cy: xy[0][1], r: '2.5', fill: s.col, class: 'bc-dot'}));
    if (s.on) {
      s.pts.forEach((p, i) => {
        if (p[3] != null) g.append(ecSv('line', {x1: xy[i][0], x2: xy[i][0], y1: sy(p[2] - p[3]), y2: sy(p[2] + p[3]),
                                                 stroke: s.col, 'stroke-width': '1.5', opacity: '.4'}));
        g.append(ecSv('circle', {cx: xy[i][0], cy: xy[i][1], r: p[5] < p[0] ? '2.25' : '3.5', fill: s.col,
                                 stroke: 'var(--surface)', 'stroke-width': '1.25'}));
      });
    }
    if (s.cut) {
      const [x, y] = xy[xy.length - 1];
      g.append(ecSv('path', {d: 'M' + (x - 4) + ',' + (y - 4) + ' L' + (x + 4) + ',' + (y + 4) + ' M' + (x - 4) + ','
                                + (y + 4) + ' L' + (x + 4) + ',' + (y - 4), class: 'rt', stroke: s.col}));
    }
    const pk = BC.peaks ? bcPeak(s.n) : null;
    if (pk) g.append(ecSv('circle', {cx: sx(pk[0]), cy: sy(pk[2]), r: s.on ? '5.5' : '4', class: 'pk', stroke: s.col}));
    svg.append(g);
    s.pts.forEach((p, i) => pts.push({s, p, i, x: xy[i][0], y: xy[i][1]}));
    if (s.on) ends.push({s, x: xy[xy.length - 1][0], y: xy[xy.length - 1][1], c: lastOf(s)[0]});
  });
  // the random players' labels sit at their lines' right ends, placed with the highlighted lines' labels
  const refEnds = refs.map(r => ({r, x: PR, y: sy(r[1]), c: cmax}));
  const byEnd = new Map();
  ends.concat(refEnds).forEach(e => { if (!byEnd.has(e.c)) byEnd.set(e.c, []); byEnd.get(e.c).push(e); });
  byEnd.forEach(items => ecPlace(items, T + 4, PB + 4, 15));
  refEnds.forEach(e => {
    const g = ecSv('g', {class: 'ec-rl'});
    if (Math.abs(e.ly - e.y) > 1)
      g.append(ecSv('polyline', {points: PR + ',' + e.y + ' ' + (PR + 4) + ',' + e.ly, fill: 'none', class: 'refline'}));
    g.append(ecSv('text', {x: PR + 6, y: e.ly + 4, class: 'ec-reflab'}, refText(e.r)));
    svg.append(g);
  });
  ends.forEach(e => {
    const g = ecSv('g', {class: 'ec-lab', 'data-ln': e.s.n}), x = e.x + 6, lx = e.x + 13;
    if (Math.abs(e.ly - e.y) > 1)
      g.append(ecSv('polyline', {points: x + ',' + e.y + ' ' + (lx - 3) + ',' + e.ly + ' ' + lx + ',' + e.ly,
        fill: 'none', stroke: e.s.col, 'stroke-width': '1'}));
    const t = ecSv('text', {x: lx + 2, y: e.ly + 4});
    t.append(ecSv('tspan', {}, e.s.n), ecSv('tspan', {class: 'ec-lv', dx: '6'}, ecNum(lastOf(e.s)[2])));
    g.append(t);
    svg.append(g);
  });
  const ring = ecSv('circle', {r: '7', fill: 'none', 'stroke-width': '2', visibility: 'hidden'});
  const ov = ecSv('rect', {x: L - 12, y: 0, width: W - L + 12, height: PB + 8, fill: 'transparent', class: 'bc-ov'});
  svg.append(ring, ov);
  chart.append(svg);
  bcG = {svg, ring, pts, sx, sy};
  ov.addEventListener('pointermove', e => {
    const r = svg.getBoundingClientRect(), q = bcNear(e.clientX - r.left, e.clientY - r.top);
    if (q) { bcKey = {n: q.s.n, i: q.i}; bcShow(q, e.clientX, e.clientY); } else bcHide();
  });
  ov.addEventListener('pointerleave', bcHide);
  ov.addEventListener('click', e => {
    const r = svg.getBoundingClientRect(), q = bcNear(e.clientX - r.left, e.clientY - r.top);
    if (q) fxHiToggle(q.s.n);
  });
  svg.addEventListener('focus', () => {
    if (!bcKey || !pts.some(q => q.s.n === bcKey.n)) {
      const first = hiDrawn[0] || drawn.slice().sort((a, b) => lastOf(b)[2] - lastOf(a)[2])[0];
      bcKey = {n: first.n, i: first.pts.length - 1};
    }
    bcKeyShow();
  });
  svg.addEventListener('blur', bcHide);
  svg.addEventListener('keydown', bcKeyDown);
  if (hadFocus) svg.focus();
}
function bcNear(mx, my) {  // the point nearest the pointer, if within reach; the highlighted lines' first
  if (!bcG) return null;
  let best = null, bd = 18;
  const dim = bcG.svg.classList.contains('bc-hl');
  bcG.pts.forEach(q => {
    const d = Math.hypot(q.x - mx, q.y - my) + (dim && !q.s.on ? 6 : 0);
    if (d < bd) { bd = d; best = q; }
  });
  return best;
}
function bcShow(q, cx, cy) {
  const g = bcG, p = q.p, n = q.s.n, pk = bcPeak(n);
  g.ring.setAttribute('cx', q.x);
  g.ring.setAttribute('cy', q.y);
  g.ring.setAttribute('stroke', q.s.col);
  g.ring.setAttribute('visibility', 'visible');
  bcHov(n);
  ecTip.textContent = '';
  const head = ecEl('div', {class: 'bc-tn'});
  head.append(ecKeySvg(q.s.col, q.s.fz != null && p[0] >= q.s.fz), ecEl('b', {}, n));
  ecTip.append(head,
               ecEl('div', {class: 'v'}, ecNum(p[2]) + (p[3] != null ? ' ± ' + Math.round(p[3]) : '') + ' Elo'),
               ecEl('div', {}, 'cycle ' + p[0] + ' · v' + p[1] + ' · ' + bcPly() + '-ply'
                               + (p[4] ? ' (under way: live, today’s fit)'
                                  : p[3] == null ? ' (the Elo history: no ± kept)' : '')),
               ecEl('div', {class: 'm'}, (p[5] < p[0] ? 'the same version since cycle ' + p[5]
                                                      : 'v' + p[1] + '’s first cycle')
                                         + (p[4] ? '' : ' · today: ' + (p[6] == null ? 'not rated'
                                            : ecNum(p[6]) + ' ± ' + Math.round(p[7]) + ', ' + p[8] + ' games'))),
               ...(p[10] ? [ecEl('div', {class: 'm'}, 'cycle ' + p[0] + ' decision: ' + p[10])] : []),
               ecEl('div', {class: 'm'}, 'now: ' + (bcStatus(n) || '—')
                                         + (q.s.frz != null ? ' · frozen from cycle ' + q.s.frz + (q.s.fz == null
                                            ? ' (it kept its other entry; this one retired then)' : '') : '')
                                         + (pk ? ' · peak ' + ecNum(pk[2]) + ' (v' + pk[1] + ', cycle ' + pk[0] + ')' : '')),
               ecEl('div', {class: 'm bc-hint'}, (q.s.on ? 'click to un-highlight' : 'click to highlight')));
  ecTip.hidden = false;
  const w = ecTip.offsetWidth, h = ecTip.offsetHeight;
  let x = cx + 16, y = cy + 16;
  if (x + w > innerWidth - 8) x = Math.max(8, cx - 16 - w);
  if (y + h > innerHeight - 8) y = Math.max(8, cy - 16 - h);
  ecTip.style.left = x + 'px';
  ecTip.style.top = y + 'px';
}
function bcHide() {
  ecTip.hidden = true;
  if (bcG) bcG.ring.setAttribute('visibility', 'hidden');
  bcHov(null);
}
function bcHov(n) {  // a line picked out while pointed at (in the chart or the legend): the others dim
  bcHovN = n;
  if (!bcG) return;
  bcG.svg.classList.toggle('bc-hv', !!n);
  bcG.svg.querySelectorAll('.bc-s, .ec-lab').forEach(x => x.classList.toggle('hov', x.dataset.ln === n));
}
function bcKeyShow() {
  if (!bcG || !bcKey) return;
  const line = bcG.pts.filter(q => q.s.n === bcKey.n);
  if (!line.length) return;
  bcKey.i = Math.max(0, Math.min(line.length - 1, bcKey.i));
  const q = line[bcKey.i], r = bcG.svg.getBoundingClientRect();
  bcShow(q, r.left + q.x, r.top + q.y);
}
function bcKeyDown(e) {
  if (!bcG || !bcKey) return;
  if (e.key === 'Escape') { bcHide(); return; }
  if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); fxHiToggle(bcKey.n); return; }
  const line = bcG.pts.filter(q => q.s.n === bcKey.n);
  if (e.key === 'ArrowLeft' || e.key === 'ArrowRight') {
    e.preventDefault();
    bcKey.i += e.key === 'ArrowLeft' ? -1 : 1;
  } else if (e.key === 'ArrowUp' || e.key === 'ArrowDown') {  // the next line above or below at this cycle
    e.preventDefault();
    const cur = line[Math.max(0, Math.min(line.length - 1, bcKey.i))];
    const at = bcG.pts.filter(q => q.p[0] === cur.p[0]).sort((a, b) => a.y - b.y || (a.s.n < b.s.n ? -1 : 1));
    const k = at.indexOf(cur) + (e.key === 'ArrowUp' ? -1 : 1);
    if (k >= 0 && k < at.length) bcKey = {n: at[k].s.n, i: at[k].i};
  } else return;
  bcKeyShow();
}
function bcTable(ser) {
  const out = document.querySelector('#bc .ec-tb');
  if (!out) return;
  out.textContent = '';
  if (!ser.length) { out.append(ecEl('p', {class: 'note'}, 'No rated versions to show.')); return; }
  const cyc = [...new Set(ser.flatMap(s => s.pts.map(p => p[0])))].sort((a, b) => a - b);
  const t = ecEl('table'), hr = ecEl('tr');
  hr.append(ecEl('th', {}, 'lineage'));
  cyc.forEach(c => hr.append(ecEl('th', {class: 'r'}, 'cycle ' + c)));
  hr.append(ecEl('th', {}, 'lineage now'));
  t.append(hr);
  ser.forEach(s => {
    const tr = ecEl('tr'), by = new Map(s.pts.map(p => [p[0], p])), name = ecEl('td');
    name.append(ecKeySvg(s.col, s.fz != null), document.createTextNode(' ' + s.n));
    tr.append(name);
    cyc.forEach(c => {
      const p = by.get(c), td = ecEl('td', {class: 'r'});
      if (p) {
        td.append(document.createTextNode(ecNum(p[2]) + (p[3] != null ? ' ± ' + Math.round(p[3]) : '') + ' '),
                  ecEl('span', {class: 'muted'}, 'v' + p[1] + (p[4] ? ', live' : '')));
        td.title = (p[5] < p[0] ? 'the same version since cycle ' + p[5] : 'v' + p[1] + '’s first cycle')
                   + (p[6] == null ? '' : '; today: ' + ecNum(p[6]) + ' ± ' + Math.round(p[7]))
                   + (p[10] ? '; cycle ' + p[0] + ' decision: ' + p[10] : '');
      }
      tr.append(td);
    });
    tr.append(ecEl('td', {}, bcStatus(s.n) || '—'));
    t.append(tr);
  });
  out.append(t);
}
function bcSigNow() {  // what the chart shows depends on: its data, the facets, filters, colour, highlight, choices
  return [ecText, FXT, JSON.stringify([FXS.on, FXS.by, fxHiList(), BC, bcPly()])].join('\u0002');
}
function bcRefresh() {  // (re)draw when the data or anything it shows changed
  const box = document.getElementById('bc'), src = document.getElementById('elo-data');
  if (!box || !src) return;
  const text = src.textContent;
  if (text !== ecText) {
    let data;
    try { data = JSON.parse(text); } catch (err) { return; }
    ecD = data;
    ecText = text;
    bcLins = new Map(ecD.lins.map(l => [l[0], l]));
  }
  if (!ecD) return;
  if (box.dataset.built !== '1') { bcBuild(box); box.dataset.built = '1'; bcSig = null; }
  if (bcSigNow() !== bcSig || !box.querySelector('.ec-chart svg, .ec-chart p')) bcDraw();
}
function bcReveal() {  // bring the chart into view, just below the bar that stays at the top
  const sec = document.getElementById('chart');
  if (!sec) return;
  const bar = document.getElementById('fxbar'), h = bar ? bar.getBoundingClientRect().height : 0;
  const quiet = window.matchMedia && matchMedia('(prefers-reduced-motion: reduce)').matches;
  window.scrollTo({top: Math.max(0, sec.getBoundingClientRect().top + window.scrollY - h - 8),
                   behavior: quiet ? 'auto' : 'smooth'});
}
document.addEventListener('click', e => {
  const b = e.target.closest && e.target.closest('#bc button');
  if (!b || !ecD) return;
  if (b.dataset.bcply) { if (!bcPinned()) { BC.ply = b.dataset.bcply; bcSave(); bcDraw(); } }
  else if (b.dataset.bcpk) { BC.peaks = !BC.peaks; bcSave(); bcDraw(); }
  else if (b.dataset.bcrefs) { BC.refs = !BC.refs; bcSave(); bcDraw(); }
  else if (b.dataset.bcclr) fxHiClear();
  else if (b.dataset.ln) fxHiToggle(b.dataset.ln);
});
document.addEventListener('pointerover', e => {
  const t = e.target.closest && e.target.closest('#bc .bc-li');
  if (t) bcHov(t.dataset.ln);
});
document.addEventListener('pointerout', e => {
  const t = e.target.closest && e.target.closest('#bc .bc-li');
  if (t && !(e.relatedTarget && t.contains(e.relatedTarget))) bcHov(null);
});
window.addEventListener('resize', () => { cancelAnimationFrame(bcRaf); bcRaf = requestAnimationFrame(bcDraw); });
bcRefresh();
"""


# ---------------------------------------------------------------- the page

CSS = """
.viz-root{color-scheme:light;--surface:#fcfcfb;--plane:#f9f9f7;--ink:#0b0b0b;--ink2:#52514e;--muted:#898781;
 --grid:#e1e0d9;--axis:#c3c2b7;--c-fl:#2a78d6;--c-other:#eb6834;--c-tree:#1baf7a;--c-ref:#898781;--good:#0ca30c;
 --good-text:#006300;--warning:#fab219;--serious:#ec835a;--critical:#d03b3b;--ring:rgba(11,11,11,.10);--track:#cde2fb;
 --fill:#2a78d6}
@media (prefers-color-scheme:dark){:root:where(:not([data-theme="light"])) .viz-root{color-scheme:dark;
 --surface:#1a1a19;--plane:#0d0d0d;--ink:#f0efec;--ink2:#c3c2b7;--grid:#2c2c2a;--axis:#383835;--c-fl:#3987e5;
 --c-other:#d95926;--c-tree:#199e70;--good-text:#0ca30c;--ring:rgba(255,255,255,.10);--track:#184f95;--fill:#3987e5}}
body{margin:0;background:var(--plane);font:14px/1.45 system-ui,-apple-system,"Segoe UI",sans-serif}
.viz-root{color:var(--ink);background:var(--plane);padding:16px 24px 40px;min-height:100vh;box-sizing:border-box}
h1{font-size:20px;margin:0 0 2px}h2{font-size:16px;margin:0 0 10px}h3{font-size:14px;margin:14px 0 6px}
.sub,.note,.muted,.ts,.tl{color:var(--ink2)}.note{font-size:12.5px;margin:6px 0}.muted{color:var(--muted)}
section{background:var(--surface);border:1px solid var(--ring);border-radius:8px;padding:14px 16px;margin:14px 0}
.tiles{display:flex;flex-wrap:wrap;gap:10px}.tile{border:1px solid var(--ring);border-radius:8px;padding:8px 12px;
 min-width:150px}.tl{font-size:12px}.tv{font-size:22px;font-weight:600}.ts{font-size:12px}
table{border-collapse:collapse;font-variant-numeric:tabular-nums;font-size:13px}
th{font-weight:600;color:var(--ink2);text-align:left;padding:3px 8px;border-bottom:1px solid var(--axis)}
td{padding:2px 8px;border-bottom:1px solid var(--grid);white-space:nowrap}td.r,th.r{text-align:right}
tr.ref td{color:var(--ink2)}
table.standings tr.ret td{color:var(--muted)}table.standings tr.ret svg{opacity:.45}
.npk{font-size:11px;font-weight:600;color:var(--good-text);background:var(--gk);border-radius:8px;padding:0 6px;white-space:nowrap}
table.standings th.pk{cursor:help}.pkd{color:var(--ink2)}.tscroll{overflow-x:auto}.hlp{cursor:help}
.viz-root{--frz:#d6e6fa;--frz-ink:#104281}
@media (prefers-color-scheme:dark){:root:where(:not([data-theme="light"])) .viz-root{--frz:#184f95;--frz-ink:#e6effb}}
.frz{font-size:11px;font-weight:600;color:var(--frz-ink);background:var(--frz);border-radius:8px;padding:0 6px;
 white-space:nowrap}
.stf{display:flex;flex-wrap:wrap;gap:4px 14px;align-items:center;font-size:12.5px;margin:2px 0 8px}
.stf label{display:inline-flex;gap:5px;align-items:center;cursor:pointer;color:var(--ink2)}
.stf input{margin:0;accent-color:var(--ink2)}.ms{cursor:help}.stf .stf-n{margin-left:auto}
.sortb{font:inherit;font-weight:600;color:var(--ink2);background:none;border:0;padding:0;cursor:pointer;
 text-decoration:underline dotted;text-underline-offset:3px}.sortb[aria-pressed="true"]{color:var(--ink)}
.sortb:focus-visible{outline:2px solid var(--ink2);outline-offset:2px}
.ok{color:var(--good)}.bad{color:var(--critical)}.warn{color:var(--serious)}
.meter{display:inline-block;position:relative;width:120px;height:8px;border-radius:4px;background:var(--track);
 vertical-align:middle;overflow:hidden}.meter .fill{position:absolute;left:0;top:0;bottom:0;background:var(--fill);
 border-radius:4px}.meter .over{position:absolute;top:0;bottom:0;background:var(--serious);border-radius:0 4px 4px 0}
.meter .tick{position:absolute;top:-2px;bottom:-2px;width:2px;margin-left:-1px;background:var(--surface)}
.bar-line{display:flex;align-items:center;gap:10px;margin:8px 0}.bar-line .meter{width:280px}
svg text{fill:var(--muted);font:11px system-ui,-apple-system,"Segoe UI",sans-serif}
svg .val{fill:var(--ink2)}svg.cpu text{fill:var(--ink2)}
.grid{stroke:var(--grid);stroke-width:1}.axisline{stroke:var(--axis);stroke-width:1}
.refline{stroke:var(--c-ref);stroke-width:1;opacity:.6}
.ec-join{stroke:var(--ink2);stroke-width:1;stroke-dasharray:3 3;opacity:.8}.ec-join-t{font-size:10.5px;fill:var(--ink2)}.budget{stroke:var(--ink2);stroke-width:1.5}
.bar{fill:var(--muted)}.barrun{fill:var(--axis)}.barover{fill:var(--serious)}
.legend{display:flex;flex-wrap:wrap;gap:4px 16px;font-size:12.5px;color:var(--ink2);margin:4px 0 10px}
.legend span,.items span{display:inline-flex;align-items:center;gap:5px}.sw{vertical-align:middle}
.grid-mult{display:flex;flex-wrap:wrap;gap:12px}
.mult{margin:0;border:1px solid var(--ring);border-radius:8px;padding:6px 8px;width:300px}
.mult figcaption{font-size:12.5px;font-weight:600;color:var(--ink2);margin-bottom:2px}
.items{display:flex;flex-wrap:wrap;gap:2px 10px;font-size:12px;color:var(--ink2);margin-top:4px}
.items .li{cursor:default}.dim{opacity:.15}
details summary{cursor:pointer;color:var(--ink2);font-size:13px;margin:8px 0}
ul.notes{margin:6px 0;padding-left:18px;color:var(--ink2);font-size:12.5px}
table.mdt{font-size:12.5px;margin:2px 0 8px 18px}table.mdt td{color:var(--ink2)}
#tip{position:fixed;z-index:9;pointer-events:none;background:var(--surface);color:var(--ink);border:1px solid var(--ring);
 border-radius:6px;padding:6px 8px;font-size:12.5px;white-space:pre-line;box-shadow:0 2px 8px rgba(0,0,0,.15)}
#stale{color:var(--critical)}
h2 a.anchor{color:inherit;text-decoration:none}h2 a.anchor:hover{text-decoration:underline}
.viz-root{--dz:#f0efec;--dp1:#b7d3f6;--dp2:#86b6ef;--dp3:#5598e7;--dp4:#2a78d6;--dn1:#fac0ba;--dn2:#f2958e;
 --dn3:#e76762;--dn4:#d2383a;--dlo:#0b0b0b;--dhi:#ffffff}
@media (prefers-color-scheme:dark){:root:where(:not([data-theme="light"])) .viz-root{--dz:#383835;--dp1:#104281;
 --dp2:#1c5cab;--dp3:#2a78d6;--dp4:#5598e7;--dn1:#7e1519;--dn2:#a72528;--dn3:#d2383a;--dn4:#e76762;--dlo:#ffffff;
 --dhi:#0b0b0b}}
.s-z{background:var(--dz);color:var(--ink)}
.s-p1{background:var(--dp1);color:var(--dlo)}.s-p2{background:var(--dp2);color:var(--dlo)}
.s-p3{background:var(--dp3);color:var(--dlo)}.s-p4{background:var(--dp4);color:var(--dhi)}
.s-n1{background:var(--dn1);color:var(--dlo)}.s-n2{background:var(--dn2);color:var(--dlo)}
.s-n3{background:var(--dn3);color:var(--dlo)}.s-n4{background:var(--dn4);color:var(--dhi)}
.h2h-bar{display:flex;flex-wrap:wrap;gap:6px;align-items:center;margin:2px 0 8px;font-size:12.5px;color:var(--ink2)}
.h2h-bar button,.h2h-close{font:inherit;font-size:12.5px;border:1px solid var(--axis);background:var(--surface);
 color:var(--ink);border-radius:6px;padding:2px 10px;cursor:pointer}
.h2h-bar button[aria-pressed="true"]{background:var(--ink);color:var(--surface);border-color:var(--ink)}
.h2h-close{margin-left:8px;font-weight:400}
.h2h-bar select{font:inherit;font-size:12.5px;border:1px solid var(--axis);background:var(--surface);color:var(--ink);
 border-radius:6px;padding:2px 6px;cursor:pointer}
.h2h-bar select.on{background:var(--ink);color:var(--surface);border-color:var(--ink)}
.h2h-partial{margin:0 0 6px}
.h2h-scale{display:flex;flex-wrap:wrap;align-items:center;gap:2px;font-size:11.5px;color:var(--ink2);margin:0 0 8px}
.h2h-scale .lbl{margin-right:6px}.h2h-scale span[class^="s-"]{padding:1px 6px;border-radius:3px;min-width:34px;
 text-align:center;font-variant-numeric:tabular-nums}
.h2h-wrap{overflow:auto;max-height:78vh;border:1px solid var(--ring);border-radius:6px;background:var(--surface)}
table.h2h{border-collapse:separate;border-spacing:0;font-size:11px}
table.h2h th,table.h2h td{padding:1px 4px;border-bottom:1px solid var(--surface);border-right:1px solid var(--surface)}
table.h2h thead th{position:sticky;top:0;z-index:2;background:var(--surface);vertical-align:bottom;padding:4px 1px;
 border-bottom:1px solid var(--axis)}
table.h2h thead th.ch{text-align:center}table.h2h thead th.ch span{writing-mode:vertical-rl;transform:rotate(180deg);
 white-space:nowrap;font-weight:500;display:inline-block}
table.h2h tbody th{position:sticky;left:0;z-index:1;background:var(--surface);text-align:left;white-space:nowrap;
 font-weight:500;border-right:1px solid var(--axis)}
table.h2h thead th.corner{left:0;z-index:3;text-align:left;font-weight:400;color:var(--muted);vertical-align:bottom}
table.h2h tbody th button{font:inherit;color:var(--ink);background:none;border:0;padding:0;cursor:pointer;
 text-align:left}
table.h2h tbody th button:hover,table.h2h tbody th button:focus-visible{text-decoration:underline}
table.h2h td{text-align:center;min-width:40px}table.h2h td.dg{background:var(--plane)}
table.h2h tr.sel th{background:var(--grid)}table.h2h tr.sel td{outline:1px solid var(--ink2);outline-offset:-1px}
table.h2h td[data-h]:hover{outline:2px solid var(--ink);outline-offset:-2px}
#h2h-player h3{display:flex;align-items:center}
.h2hp-wrap{max-height:60vh}table.h2hp thead th{position:sticky;top:0;background:var(--surface);z-index:1}
table.h2hp tr.tot td{font-weight:600;border-top:1px solid var(--axis)}
.chip{display:inline-block;padding:0 6px;border-radius:3px}
.sr{position:absolute;width:1px;height:1px;overflow:hidden;clip-path:inset(50%);white-space:nowrap}
.viz-root{--gk:rgba(12,163,12,.13);--gr:rgba(208,59,59,.15)}
@media (prefers-color-scheme:dark){:root:where(:not([data-theme="light"])) .viz-root{--gk:rgba(12,163,12,.18);
 --gr:rgba(208,59,59,.23)}}
.gates-top{display:flex;flex-wrap:wrap;gap:4px 28px;align-items:flex-start}.gates-top>div{max-width:560px}
figure.gchart{margin:0}figure.gchart figcaption{font-size:12.5px;font-weight:600;color:var(--ink2);margin:6px 0 2px}
.gchart .pt{fill:var(--fill);stroke:var(--surface);stroke-width:2}.gchart .pt.part{fill:var(--surface);stroke:var(--fill)}
.gchart .wh{stroke:var(--fill);stroke-width:2;stroke-linecap:round;opacity:.5}.gchart .zero{stroke:var(--ink2);stroke-width:1}
.gates-wrap{position:relative;overflow:auto;max-height:80vh;border:1px solid var(--ring);border-radius:6px;
 background:var(--surface);margin-top:10px}
table.gates{border-collapse:separate;border-spacing:0;font-size:12px}
table.gates th,table.gates td{padding:3px 8px;border-bottom:2px solid var(--surface);border-right:2px solid var(--surface)}
table.gates thead th{position:sticky;top:0;z-index:2;background:var(--surface);border-bottom:1px solid var(--axis);
 vertical-align:bottom;text-align:right;white-space:nowrap;font-weight:600}
table.gates thead th.corner{left:0;z-index:3;text-align:left}
table.gates th.gsum{white-space:normal;min-width:150px;max-width:180px;font-weight:600}
table.gates tbody th{position:sticky;left:0;z-index:1;background:var(--surface);text-align:left;white-space:nowrap;
 font-weight:500;border-right:1px solid var(--axis)}
table.gates td.gc{text-align:right;line-height:1.3;vertical-align:top;white-space:nowrap}
table.gates td.gk,.gkey.gk{background:var(--gk)}table.gates td.gr,.gkey.gr{background:var(--gr)}
table.gates .gi,.gkey .gi{font-weight:700;margin-right:4px}
.gk .gi{color:var(--good-text)}.gr .gi{color:var(--critical)}.gf .gi{color:var(--serious)}
table.gates .ge{color:var(--ink2);font-size:11.5px}table.gates .gz{display:block;font-size:10.5px;color:var(--muted);
 font-style:italic}
table.gates tr.gfam th{position:static;color:var(--ink2);font-weight:600;padding-top:10px;
 border-bottom:1px solid var(--grid);border-right:0}
table.gates tr.gfam .gfl{position:sticky;left:8px;display:inline-flex;align-items:center;gap:5px}
table.gates tr.gpool th,table.gates tr.gpool td{background:var(--plane);border-bottom:1px solid var(--axis)}
table.gates tr:hover td.gc:not(.gk):not(.gr){background:var(--plane)}
.gpart{font-weight:400;color:var(--muted)}
.gkey{display:inline-block;padding:0 6px;border-radius:3px;margin-right:6px;color:var(--ink)}
"""

JS = """
const tip = document.getElementById('tip');
function hl(el) {
  const m = el && el.closest ? el.closest('.mult') : null;
  document.querySelectorAll('.mult').forEach(x => { if (x !== m) x.querySelectorAll('.dim').forEach(y => y.classList.remove('dim')); });
  if (!m) return;
  const s = el.closest('[data-s]');
  m.querySelectorAll('[data-s]').forEach(x => x.classList.toggle('dim', !!s && x.dataset.s !== s.dataset.s));
}
function showTip(el, x, y) {
  const t = el && el.closest ? el.closest('[data-tip]') : null;
  if (!t) { tip.hidden = true; return; }
  tip.textContent = t.getAttribute('data-tip');
  tip.hidden = false;
  if (x !== undefined) { tip.style.left = (x + 14) + 'px'; tip.style.top = (y + 14) + 'px'; }
}
document.addEventListener('pointerover', e => { hl(e.target); showTip(e.target, e.clientX, e.clientY); });
document.addEventListener('pointermove', e => { if (!tip.hidden) { tip.style.left = (e.clientX + 14) + 'px'; tip.style.top = (e.clientY + 14) + 'px'; } });
document.addEventListener('focusin', e => { hl(e.target); const r = e.target.getBoundingClientRect(); showTip(e.target, r.left, r.bottom); });
document.addEventListener('pointerleave', () => { tip.hidden = true; hl(null); });
setInterval(async () => {
  try {
    const r = await fetch(location.pathname, {cache: 'no-store'});
    if (!r.ok) throw new Error(r.status);
    const doc = new DOMParser().parseFromString(await r.text(), 'text/html');
    const app = doc.getElementById('app');
    if (!app) return;
    const open = new Map([...document.querySelectorAll('details[id]')].map(x => [x.id, x.open]));
    const keep = [...document.querySelectorAll('[data-keep]')];
    const sk = new Map([...document.querySelectorAll('[data-sk][id]')].map(x =>
      [x.id, [x.scrollTop, x.scrollLeft, x.scrollLeft + x.clientWidth >= x.scrollWidth - 2]]));
    document.getElementById('app').replaceWith(document.adoptNode(app));
    keep.forEach(k => { const slot = document.getElementById(k.id); if (slot && slot !== k) { k.dataset.v = slot.dataset.v; slot.replaceWith(k); } });
    fxRefresh();  // the facets: colours, filters, the Standings' sections and order, the comparison
    skRestore(sk);
    open.forEach((v, id) => { const x = document.getElementById(id); if (x) x.open = v; });
    document.getElementById('stale').hidden = true;
    h2hLoad(false);
    bcRefresh();
  } catch (err) { document.getElementById('stale').hidden = false; }
}, %d);
function skRestore(sk) {
  document.querySelectorAll('[data-sk][id]').forEach(x => {
    const at = sk && sk.get(x.id);
    x.scrollTop = at ? at[0] : 0;
    const right = x.dataset.sk !== 'left';  // gates: start at the right end (the newest cycles), and stay there
    x.scrollLeft = at && !(right && at[2]) ? at[1] : right ? x.scrollWidth : 0;
  });
}
skRestore(null);
const ST = {sort: 'elo', only: []};
try { ST.sort = localStorage.getItem('cycles.standings.sort') || 'elo'; } catch (err) {}
try {
  localStorage.removeItem('cycles.standings.hide');  // the old untick-to-hide filter: its state must not carry over
  const o = JSON.parse(localStorage.getItem('cycles.standings.only') || '[]');
  ST.only = Array.isArray(o) ? o.map(String) : [];
} catch (err) {}
function stOnly() {  // the ticked statuses that have rows now (none: no filter)
  const have = new Set([...document.querySelectorAll('table.standings tr[data-st]')].map(r => r.dataset.st));
  return ST.only.filter(s => have.has(s));
}
function stFilter() {  // the ticked statuses' rows (every row when none is ticked) of the entries the factor filters
  // show (the random players always); the ranks stay as they are; the count says how many lineages and rows show
  const only = stOnly(), rows = [...document.querySelectorAll('table.standings tr[data-st]')], act = fxActive();
  document.querySelectorAll('.stf input[data-st]').forEach(b => { b.checked = only.includes(b.dataset.st); });
  rows.forEach(r => {
    r.hidden = (only.length > 0 && !only.includes(r.dataset.st))
               || (!!r.dataset.lc && !fxPassEntry(r.dataset.lc, r.dataset.ply, act));
  });
  const shown = rows.filter(r => !r.hidden), lins = rs => new Set(rs.filter(r => r.dataset.lc).map(r => r.dataset.lc)).size;
  const c = document.querySelector('.stf .stf-n');
  if (c) {
    const part = shown.length < rows.length;
    c.textContent = part ? 'showing ' + lins(shown) + ' of ' + lins(rows) + ' lineages, ' + shown.length + ' of '
                           + rows.length + ' rows' : 'all ' + lins(rows) + ' lineages, ' + rows.length + ' rows';
    c.classList.toggle('on', part);
  }
}
document.addEventListener('change', e => {
  const b = e.target.closest ? e.target.closest('.stf input[data-st]') : null;
  if (!b) return;
  ST.only = stOnly().filter(s => s !== b.dataset.st).concat(b.checked ? [b.dataset.st] : []);
  try { localStorage.setItem('cycles.standings.only', JSON.stringify(ST.only)); } catch (err) {}
  stFilter();
});
function stSort() {  // the Standings rows by Elo (their rank), or by ply gain (blank ones last, then by rank): one
  // table across every lineage, the random players among them
  const t = document.querySelector('table.standings');
  if (!t) return;
  const rows = [...t.querySelectorAll('tr[data-rank]')];
  if (!rows.length) return stFilter();
  const g = r => (r.dataset.gain === undefined ? null : +r.dataset.gain);
  rows.sort((a, b) => {
    if (ST.sort !== 'elo') {
      const x = g(a), y = g(b);
      if (x === null && y !== null) return 1;
      if (y === null && x !== null) return -1;
      if (x !== null && x !== y) return ST.sort === 'gain-asc' ? x - y : y - x;
    }
    return a.dataset.rank - b.dataset.rank;
  });
  const body = rows[0].parentNode;
  rows.forEach(r => body.appendChild(r));
  t.querySelectorAll('th[data-sortcol]').forEach(th => {
    const on = th.dataset.sortcol === 'elo' ? ST.sort === 'elo' : ST.sort.startsWith(th.dataset.sortcol);
    th.setAttribute('aria-sort', !on ? 'none' : ST.sort === 'gain-asc' ? 'ascending' : 'descending');
    const b = th.querySelector('button'), a = th.querySelector('.sarr');
    if (b) b.setAttribute('aria-pressed', String(on));
    if (a) a.textContent = on && ST.sort !== 'elo' ? (ST.sort === 'gain-asc' ? ' ▲' : ' ▼') : '';
  });
  stFilter();
}
document.addEventListener('click', e => {
  const b = e.target.closest ? e.target.closest('table.standings button[data-sort]') : null;
  if (!b) return;
  ST.sort = b.dataset.sort === 'elo' ? 'elo' : ST.sort === 'gain-desc' ? 'gain-asc' : 'gain-desc';
  try { localStorage.setItem('cycles.standings.sort', ST.sort); } catch (err) {}
  stSort();
});
stSort();
const H2H = {scope: 'field', player: null};
try { Object.assign(H2H, JSON.parse(localStorage.getItem('cycles.h2h') || '{}')); } catch (err) {}
function h2hSave() { try { localStorage.setItem('cycles.h2h', JSON.stringify(H2H)); } catch (err) {} }
async function h2hGet(url) { const r = await fetch(url, {cache: 'no-store'}); if (!r.ok) throw new Error(r.status); return r.text(); }
function h2hMark() {
  document.querySelectorAll('table.h2h tbody tr').forEach(tr => {
    const b = tr.querySelector('button[data-p]');
    tr.classList.toggle('sel', !!b && b.dataset.p === H2H.player);
  });
}
async function h2hPlayer(scroll) {
  const out = document.getElementById('h2h-player');
  if (!out) return;
  if (!H2H.player) { out.innerHTML = ''; h2hMark(); return; }
  try {
    const html = await h2hGet('h2h/player?scope=' + encodeURIComponent(H2H.scope) + '&name=' + encodeURIComponent(H2H.player));
    const old = out.querySelector('.h2hp-wrap'), who = H2H.scope + '|' + H2H.player;
    const at = old && out.dataset.p === who ? [old.scrollTop, old.scrollLeft] : null;
    out.innerHTML = html;
    out.dataset.p = who;
    const now = out.querySelector('.h2hp-wrap');
    if (at && now) { now.scrollTop = at[0]; now.scrollLeft = at[1]; }
    h2hMark();
    if (scroll) out.scrollIntoView({block: 'nearest'});
  } catch (err) {}
}
function h2hSel(box) {
  const sel = box.querySelector('select[data-cycsel]'), meta = box.querySelector('.h2h-meta');
  if (!sel) return;
  const sig = meta ? meta.dataset.cycles + '|' + (meta.dataset.partial || '') : null;
  if (meta && sel.dataset.sig !== sig) {
    sel.dataset.sig = sig;
    sel.querySelectorAll('option[value^="c"]').forEach(o => o.remove());
    meta.dataset.cycles.split(',').filter(Boolean).forEach(c =>
      sel.append(new Option('Cycle ' + c + (c === meta.dataset.partial ? ' (partial)' : ''), 'c' + c)));
  }
  const cyc = /^c[0-9]+$/.test(H2H.scope) ? H2H.scope : '';
  if (cyc && !sel.querySelector('option[value="' + cyc + '"]')) sel.append(new Option('Cycle ' + cyc.slice(1), cyc));
  sel.value = cyc;
  sel.classList.toggle('on', !!cyc);
}
async function h2hLoad(force) {
  const box = document.getElementById('h2h');
  if (!box) return;
  box.querySelectorAll('[data-scope]').forEach(b => b.setAttribute('aria-pressed', String(b.dataset.scope === H2H.scope)));
  h2hSel(box);
  const want = box.dataset.v + '|' + H2H.scope;
  if (!force && box.dataset.loaded === want) return;
  try {
    const html = await h2hGet('h2h?scope=' + encodeURIComponent(H2H.scope));
    const old = box.querySelector('.h2h-table .h2h-wrap'), at = old && !force ? [old.scrollTop, old.scrollLeft] : null;
    box.querySelector('.h2h-table').innerHTML = html;
    const now = box.querySelector('.h2h-table .h2h-wrap');
    if (at && now) { now.scrollTop = at[0]; now.scrollLeft = at[1]; }
    box.dataset.loaded = want;
    h2hSel(box);
    h2hMark();
    fxH2h();
    await h2hPlayer(false);
  } catch (err) {}
}
document.addEventListener('click', e => {
  const t = e.target.closest ? e.target : e.target.parentElement;
  const s = t.closest('#h2h [data-scope]');
  if (s) {
    if (H2H.scope !== s.dataset.scope) { H2H.scope = s.dataset.scope; H2H.player = null; h2hSave(); h2hLoad(true); }
    return;
  }
  const p = t.closest('#h2h button[data-p]');
  if (p) { H2H.player = p.dataset.p; h2hSave(); h2hPlayer(true); return; }
  if (t.closest('#h2h .h2h-close')) { H2H.player = null; h2hSave(); h2hPlayer(false); }
});
document.addEventListener('change', e => {
  const sel = e.target.closest && e.target.closest('#h2h select[data-cycsel]');
  if (!sel) return;
  const v = sel.value || 'field';
  if (H2H.scope !== v) { H2H.scope = v; H2H.player = null; h2hSave(); h2hLoad(true); }
});
document.addEventListener('pointerover', e => {
  const td = e.target.closest && e.target.closest('table.h2h td[data-h]');
  if (!td || td.hasAttribute('data-tip')) return;
  const row = td.parentElement.querySelector('th').textContent.trim();
  const col = td.closest('table').tHead.rows[0].cells[td.cellIndex].title;
  const [w, l, dr, e3] = td.dataset.h.split(' ').map(Number);
  const n = w + l + dr, s = (w + dr / 2) / n;
  let tx = `${row} vs ${col}\\n+${w} \u2212${l} =${dr}: ${n} games, score ${(100 * s).toFixed(1)}%%`;
  if (e3 >= 0) {
    const ex = e3 / 1000, dd = 100 * (s - ex);
    tx += `\\nexpected ${(100 * ex).toFixed(1)}%% from the current fit: ${dd >= 0 ? 'over' : 'under'} by ${Math.abs(dd).toFixed(1)} points`;
  } else tx += '\\nno expected score: one of them is not rated';
  td.setAttribute('data-tip', tx);
}, true);
h2hLoad(true);
""" % (REFRESH_S * 1000)


def fx_script(d):
    """The facets for the page script (fx_data), as JSON."""
    try:
        js = json.dumps(fx_data(d), separators=(",", ":")).replace("</", "<\\/")
    except Exception:
        traceback.print_exc()
        return (f'<p class="bad">The factor filters failed to load: '
                f'{esc(traceback.format_exc().splitlines()[-1])}</p>')
    return (f'<script type="application/json" id="fx-data">{js}</script><noscript><p class="note">The factor '
            f'filters and the colour pick need JavaScript; without it the page colours by '
            f'{esc(FX[FX_BY][0].lower())}.</p></noscript>')


def panel_compare(d):
    return ('<div id="fxcmp" data-keep="1" data-v="1"><p class="note">Loading the factor comparison…</p><noscript><p '
            'class="note">The factor comparison needs JavaScript.</p></noscript></div>'
            '<p class="note">Pick a factor: each row is a set of lineages that agree on every other factor and differ in '
            'this one, a column per value (comparing architectures also lets the router and the split management vary, '
            'as a flat model has neither); the variant (hyperparameters) may differ too. A cell gives each lineage\'s '
            'peak Elo (its highest at a completed cycle, as in Standings) and its Elo now (its newest entry in today\'s '
            'fit) at the ply picked, its variant and, unless qualified, its status; past four lineages a cell folds. The '
            'difference column sets each value against the reference (the factor\'s default unless you pick one), by '
            'peak or by Elo now: for the pair of lineages from the two cells with the fewest other differences, then '
            'the best of each; ± is the two ±s combined as if independent, and "also differs" names the pair\'s other '
            'differences. A row without the reference sets its values against its first one. The summary counts the '
            'rows where each value is above or below the reference, with the median difference. The factor filters at '
            'the top narrow the lineages; the choices stay in this browser.</p>')


def panel_lineages(d):
    q, scores = str(d.qualify_ply), fx_scores(d)
    short = {"arch": "architecture", "leaf": "leaf", "router": "router", "split": "split", "depth": "self-play depth",
             "labels": "labels", "policy": "move selection"}  # the headings, short: the table must fit
    cols = ([("name", "lineage", 0), ("archetype", "archetype", 0)] + [(k, short.get(k, label), int(k == "depth"))
                                                                        for k, label, *_ in FACTORS]
            + [("variant", "variant", 0), ("status", "status", 0), ("peak", f"peak Elo, {q}‑ply", 1),
               ("now", f"Elo now, {q}‑ply", 1)])
    head = "".join(f'<th{" class=" + chr(34) + "r" + chr(34) if n else ""} data-lcol="{k}" data-num="{n}"'
                   + (f' title="{esc(FX[k][0])}"' if k in FX else "") + '><button '
                   f'type="button" class="sortb" data-lsort="{k}">{esc(label)}<span class="sarr"></span></button></th>'
                   for k, label, n in cols)
    rows = []
    for i, l in enumerate(d.registry + d.former):
        name = l["name"]
        f, var = fx(d)[name]
        sc = scores.get(name) or {"peak": {}, "now": {}}
        line = (f'{l.get("binary", "")} {" ".join(l["flags"])}' if l.get("flags") is not None else
                "no lineages.txt line: the facets come from the name")
        tds = [f'<td data-v="{esc(name)}" data-tip="{esc(line)}">{swatch(shape_of(d, name))} {esc(name)}</td>',
               f'<td data-v="{esc(l["arch"])}">{esc(l["arch"])}</td>']
        for k, *_ in FACTORS:
            v = f[k]
            grey = v == UNCL or v in FX_GREY.get(k, ())
            tds.append(f'<td data-v="{esc("" if v == UNCL else v)}"{" class=" + chr(34) + "muted" + chr(34) if grey else ""}>'
                       f'{esc(fx_label(k, v))}</td>')
        vt = ", ".join(x["t"] for x in var if x["show"])
        st = status_of(d, name)
        tds += [f'<td class="lt-w" data-v="{esc(vt)}">{esc(vt)}</td>', f'<td data-v="{esc(st)}">{esc(st)}</td>']
        pk, nw = sc["peak"].get(q), sc["now"].get(q)
        tds.append(f'<td class="r" data-v="{pk[0]}" data-tip="{esc(name)}: v{pk[2]} at the end of cycle {pk[3]}'
                   + (f", ± {pk[1]:.0f}" if pk[1] is not None else "") + f'">{plain(pk[0])} <span class="muted">v{pk[2]}'
                   f'</span></td>' if pk else '<td class="r" data-v=""></td>')
        tds.append(f'<td class="r" data-v="{nw[0]}" data-tip="{esc(nw[2])}: {nw[3]:,} games in today\'s fit">'
                   f'{plain(nw[0])} <span class="muted">± {nw[1]:.0f}</span></td>' if nw else '<td class="r" data-v=""></td>')
        cls = ' class="ret"' if is_retired(d, name) else ""
        rows.append(f'<tr data-i="{i}"{lc_attr(d, name)}{cls}>{"".join(tds)}</tr>')
    unc = sorted(n for n, (f, _) in fx(d).items() if UNCL in f.values())
    return ('<div class="lt-bar"><span class="lt-n"></span></div>'
            '<div class="tscroll"><table class="lintab"><thead><tr>' + head + '</tr></thead><tbody>' + "".join(rows)
            + '</tbody></table></div><p class="note">One row per lineage (lineages.txt\'s, '
            'then the retired ones without a line): its archetype, its value for each factor (hover a name for its '
            'line), its variant (the hyperparameters that differ from the field\'s usual recipe: split threshold 5M, '
            'split limit 5, forget factor 1, branch factor 3, leaf share 0, 2 experts, TAN m 5, τ 1e-5, seed 21; a '
            'DAG\'s leaves and window always), its status, and its peak Elo and Elo now as in Standings. Click a '
            'heading to sort (again to reverse, a third time for lineages.txt\'s order), a row to highlight the '
            'lineage in the chart at the top. A lineage\'s factors come '
            'from its lineages.txt line\'s flags (with the binary\'s defaults where a flag is absent; for a retired '
            'lineage without its line, from its name). The filters in the bar at the top tick values of them: values '
            'of one factor add up (or), factors narrow down (and), and a value\'s count is how many lineages it would '
            'show with the other factors\' ticks; league play depth is per entry, not per lineage (Standings, head to '
            'head). They narrow every panel: Standings, the factor comparison, this table, head to head, Elo over '
            'cycles and the gates (the random players always stay; pooled rows still pool every lineage). Colour and '
            'group by, in the same bar, colours every lineage\'s marks by its value of that factor and groups the '
            'gate rows and the chart\'s list of lineages. The choices stay in this browser.'
            + (f' Unclassified (neither the flags nor the name settle a value): {esc(", ".join(unc))}.' if unc else "")
            + '</p>')


def section(title, body_fn, d, sid):
    try:
        body = body_fn(d)
    except Exception:
        traceback.print_exc()
        body = f'<p class="bad">This panel failed to render: {esc(traceback.format_exc().splitlines()[-1])}</p>'
    return f'<section id="{sid}"><h2><a class="anchor" href="#{sid}">{esc(title)}</a></h2>{body}</section>'


def render(config_path):
    with LOCK:
        try:
            d = Data(config_path)
        except Exception:
            traceback.print_exc()
            return (f"<!doctype html><meta charset=utf-8><meta http-equiv=refresh content={REFRESH_S}>"
                    f"<pre>dashboard: {esc(traceback.format_exc())}</pre>")
        stamp = datetime.datetime.fromtimestamp(d.now).strftime("%Y-%m-%d %H:%M:%S")
        errs = [f"{os.path.relpath(p, d.root)}: {e}" for p, e in _errors.items()]
        app = (f'<div id="app"><h1>Training/tournament cycles</h1><div class="sub">{esc(stamp)} · updates every '
               f'{REFRESH_S} s · <span id="stale" hidden>the dashboard server is not responding; showing the last '
               f'page</span></div>'
               + (f'<p class="note">Kept the last good reading of: {esc("; ".join(errs))}</p>' if errs else "")
               + '<div id="fxbar" class="fxbar" data-keep="1" data-v="1"></div>' + fx_script(d)
               + section("Now", panel_now, d, "now")
               + section("Elo by cycle: every lineage the filters show", panel_chart, d, "chart")
               + section("Standings: the current field", panel_standings, d, "standings")
               + section("Factor comparison", panel_compare, d, "compare")
               + section("Lineages", panel_lineages, d, "lineages")
               + section("Head to head", panel_h2h, d, "head-to-head")
               + section("Elo over cycles, by archetype", panel_elo, d, "elo") + section("Gate history", panel_gates, d, "gates")
               + section("Cycle history", panel_history, d, "history") + "</div>")
        return ("<!doctype html><html lang=en><head><meta charset=utf-8><title>Cycles</title>"
                f"<meta name=viewport content='width=device-width'><style>{CSS}{EC_CSS}{FX_CSS}</style></head>"
                f'<body><div class="viz-root">{app}<div id="tip" hidden></div></div>'
                f"<noscript><meta http-equiv=refresh content={REFRESH_S}></noscript><script>{FX_JS}{JS}{EC_JS}fxRefresh();</script>"
                "</body></html>")


class Handler(BaseHTTPRequestHandler):
    config_path = None

    def do_GET(self):
        path, _, query = self.path.partition("?")
        if path in ("/h2h", "/h2h/player"):
            q = urllib.parse.parse_qs(query)
            try:
                body = h2h_fragment(self.config_path, q.get("scope", ["field"])[0],
                                    q.get("name", [""])[0] if path == "/h2h/player" else None)
            except Exception:
                traceback.print_exc()
                body = f'<p class="bad">The head-to-head table failed: {esc(traceback.format_exc().splitlines()[-1])}</p>'
            body = body.encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Cache-Control", "no-store")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if self.path.split("?")[0] not in ("/", "/index.html"):
            self.send_response(404 if self.path != "/favicon.ico" else 204)
            self.end_headers()
            return
        body = render(self.config_path).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Cache-Control", "no-store")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, fmt, *args):
        pass  # no line per request


def warm(config_path):
    """Render once at start, so the file caches and the fit's covariance are ready for the first visit."""
    try:
        render(config_path)
    except Exception:
        traceback.print_exc()


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--config", default=os.path.join(HERE, "config.json"))
    ap.add_argument("--port", type=int, default=8765)
    ap.add_argument("--once", action="store_true", help="render the page once to stdout and exit")
    args = ap.parse_args()
    if os.nice(0) < 19:
        os.nice(19 - os.nice(0))
    cfg = load(os.path.abspath(args.config), json.loads) or {}
    root = cfg.get("root", os.path.dirname(HERE))
    runs = cfg.get("runs_dir", "cycles/runs")
    runs = runs if os.path.isabs(runs) else os.path.join(root, runs)
    if args.once:
        du_once = threading.Thread(target=du_worker, args=(runs,), daemon=True)
        du_once.start()
        time.sleep(0.5)
        sys.stdout.write(render(args.config))
        return
    threading.Thread(target=du_worker, args=(runs,), daemon=True).start()
    Handler.config_path = args.config
    server = ThreadingHTTPServer(("127.0.0.1", args.port), Handler)
    server.daemon_threads = True
    threading.Thread(target=warm, args=(args.config,), daemon=True).start()
    print(f"{datetime.datetime.now():%Y-%m-%d %H:%M:%S} dashboard on http://127.0.0.1:{args.port} "
          f"(pid {os.getpid()}, nice {os.nice(0)}, config {args.config})", flush=True)
    server.serve_forever()


if __name__ == "__main__":
    main()
