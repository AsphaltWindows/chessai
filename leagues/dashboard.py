#!/usr/bin/env python3
"""The leagues' dashboard: a read-only web page over the new league system's files (FORMATS.md, beside this file).

    nice -n 19 python3 dashboard.py [--root DIR] [--port 8766] [--once PATH]

/ lists every league of the root (default: this file's directory); /global/ is the global league; /x/<name>/ an
experiment. The champion run and the first run are links to their own dashboard (http://127.0.0.1:8765/ and its
/first/ page), which this server never loads. It reads files and /proc only and writes nothing. Standard library only.
DASHBOARD.md has the pages and what each reads.
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
import shlex
import sys
import time
import traceback
import urllib.parse
from functools import cached_property
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

HERE = os.path.dirname(os.path.abspath(__file__))
CFG = {"root": HERE, "old": "http://127.0.0.1:8765/"}  # old: the champion run's dashboard, linked (not loaded)
CLK = os.sysconf("SC_CLK_TCK")
REFRESH_S = 30
_cache, _errors = {}, {}


# ---------------------------------------------------------------- reading files (read-only, cached by signature)

def load(path, parse):
    """parse(text) of the file at `path`, read again only when its mtime, size or inode changes. A missing file gives
    None; a file that fails to parse (caught half-written) gives the last good value, and the error is noted."""
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
        with open(path, encoding="utf-8", errors="replace") as f:
            value = parse(f.read())
    except Exception as e:
        _errors[path] = f"{type(e).__name__}: {e}"
        return hit[1] if hit else None
    _errors.pop(path, None)
    _cache[path] = (sig, value)
    return value


def complete_lines(text):
    """The text without a trailing partial line (a file caught while being appended to)."""
    return text if not text or text.endswith("\n") else text[:text.rfind("\n") + 1]


def parse_csv(text):
    return list(csv.DictReader(io.StringIO(complete_lines(text))))


def parse_json(text):
    return json.loads(text)


def tail(path, nbytes=8192):
    """The last `nbytes` of a file, from its first whole line."""
    try:
        with open(path, "rb") as f:
            f.seek(0, 2)
            size = f.tell()
            f.seek(max(0, size - nbytes))
            text = f.read().decode("utf-8", "replace")
        return text if size <= nbytes else text[text.find("\n") + 1:]
    except OSError:
        return ""


def listdir(path):
    try:
        return sorted(os.listdir(path))
    except OSError:
        return []


def boot_time():
    try:
        with open("/proc/stat") as f:
            for line in f:
                if line.startswith("btime"):
                    return int(line.split()[1])
    except OSError:
        pass
    return 0


BTIME = boot_time()


def proc_info(pid):
    """{pid, args, state, cpu, start} of a live process (cpu: its own user and system seconds), or None."""
    try:
        pid = int(pid)
        with open(f"/proc/{pid}/cmdline", "rb") as f:
            args = [a.decode("utf-8", "replace") for a in f.read().split(b"\0") if a]
        with open(f"/proc/{pid}/stat") as f:
            fields = f.read().rsplit(")", 1)[1].split()
        return dict(pid=pid, args=args, state=fields[0], cpu=(int(fields[11]) + int(fields[12])) / CLK,
                    start=BTIME + int(fields[19]) / CLK)
    except (OSError, IndexError, ValueError, TypeError):
        return None


# ---------------------------------------------------------------- formatting

esc = html.escape


def num(x, default=None):
    try:
        v = float(x)
        return v if math.isfinite(v) else default
    except (TypeError, ValueError):
        return default


def inum(x, default=None):
    v = num(x)
    return int(v) if v is not None else default


def price_txt(s):
    """Priced CPU seconds, worded (FORMATS.md, prices.csv: deterministic, the measure budgets use): "4.4 priced
    CPU-s"."""
    return cpu_txt(s) if s is None else cpu_txt(s).replace("CPU-", "priced CPU-")


def fmt_dur(s):
    if s is None:
        return "—"
    s = int(round(s))
    if s < 90:
        return f"{s} s"
    m = s // 60
    if m < 90:
        return f"{m} min"
    h = m // 60
    return f"{h} h {m % 60:02d} min" if h < 48 else f"{h // 24} d {h % 24} h"


def cpu_txt(s):
    """CPU seconds, short: 840 CPU-s, 14 CPU-min, 2.3 CPU-h."""
    if s is None:
        return "—"
    if s == 0:
        return "0 CPU-s"
    if abs(s) < 1:
        return f"{s:.2g} CPU-s"
    if abs(s) < 10:
        return f"{s:.1f} CPU-s"
    if s < 600:
        return f"{s:,.0f} CPU-s"
    if s < 36000:
        return f"{s / 60:,.0f} CPU-min"
    return f"{s / 3600:,.1f} CPU-h"


def plain(x, digits=0):
    """An absolute number (a rating, a gap): no "+", a real minus sign."""
    if x is None:
        return "—"
    x = 0.0 if round(x, digits) == 0 else x
    return f"{x:,.{digits}f}".replace("-", "−")


def signed(x, digits=0):
    """A recent change, with its sign: the only place a "+" goes."""
    if x is None:
        return "—"
    if round(x, digits) == 0:
        return "0"
    return f"{x:+,.{digits}f}".replace("-", "−")


def gap_words(g, what):
    """A gap in words, no sign: "12 above G.1.9", "23 below G.1.9", "level with G.1.9"."""
    if g is None:
        return "—"
    r = round(g)
    return f"level with {what}" if r == 0 else f"{abs(r)} {'above' if r > 0 else 'below'} {what}"


def moved_words(d):
    if d is None:
        return "—"
    r = round(d)
    return "0" if r == 0 else f"{'up' if r > 0 else 'down'} {abs(r)}"


def ptime(s):
    """A FORMATS.md time stamp (local "YYYY-MM-DD HH:MM:SS") as epoch seconds, or None."""
    if s in (None, ""):
        return None
    try:
        return time.mktime(datetime.datetime.strptime(str(s)[:19], "%Y-%m-%d %H:%M:%S").timetuple())
    except ValueError:
        try:
            return datetime.datetime.fromisoformat(str(s)).timestamp()
        except ValueError:
            return None


def ago(t):
    if t is None:
        return ""
    d = time.time() - t
    if d < -60:
        return f"in {fmt_dur(-d)}"
    return "just now" if d < 60 else f"{fmt_dur(d)} ago"


def tstamp(s):
    """A time stamp, short ("15:12" today, "Nov 8 15:12" this year), the full stamp and how long ago in its tooltip."""
    t = ptime(s)
    if t is None:
        return esc(str(s)) if s not in (None, "") else "—"
    dt, now = datetime.datetime.fromtimestamp(t), datetime.datetime.now()
    short = (dt.strftime("%H:%M") if dt.date() == now.date() else
             f"{dt:%b} {dt.day} {dt:%H:%M}" if dt.year == now.year else dt.strftime("%Y-%m-%d %H:%M"))
    return f'<span class="hlp" data-tip="{esc(str(s))} · {esc(ago(t))}">{esc(short)}</span>'


def meter(frac, tip=""):
    frac = max(0.0, min(1.0, frac or 0.0))
    t = f' data-tip="{esc(tip)}"' if tip else ""
    return f'<span class="meter"{t}><span class="fill" style="width:{100 * frac:.1f}%"></span></span>'


def budget_meter(value, budget, tip=""):
    """CPU against a budget: blue up to the budget, the serious colour beyond it, a tick where the budget ends."""
    value, budget = max(0.0, value or 0.0), max(0.0, budget or 0.0)
    span = max(value, budget) or 1.0
    inb, over = min(value, budget) / span, max(0.0, value - budget) / span
    t = f' data-tip="{esc(tip)}"' if tip else ""
    out = f'<span class="meter"{t}><span class="fill" style="width:{100 * inb:.1f}%"></span>'
    if over:
        out += (f'<span class="over" style="left:{100 * inb:.1f}%;width:{100 * over:.1f}%"></span>'
                f'<span class="tick" style="left:{100 * budget / span:.1f}%"></span>')
    return out + "</span>"


def tile(label, value, sub="", tip=""):
    t = f' data-tip="{esc(tip)}"' if tip else ""
    return (f'<div class="tile"{t}><div class="tl">{esc(label)}</div><div class="tv">{value}</div>'
            f'<div class="ts">{sub}</div></div>')


def th(label, key=None, kind="num", cls="", tip=""):
    """A table heading; with `key`, a sort button (table.st, the cells' data-sv)."""
    c = f' class="{cls}"' if cls else ""
    t = f' data-tip="{esc(tip)}"' if tip else ""
    if not key:
        return f"<th{c}{t}>{esc(label)}</th>"
    return (f'<th{c}{t} data-sk="{esc(key)}" data-st="{kind}"><button type="button" class="sb" aria-pressed="false">'
            f'{esc(label)}</button><span class="sa"></span></th>')


def td(content, sv=None, cls="", tip="", raw=False, it=None):
    a = f' class="{cls}"' if cls else ""
    a += f' data-sv="{esc(str(sv))}"' if sv is not None else ""
    a += f' data-tip="{esc(tip)}"' if tip else ""
    a += f' data-it="{esc(it)}"' if it else ""
    return f"<td{a}>{content if raw else esc(str(content))}</td>"


def section(title, body_fn, ctx, sid):
    try:
        body = body_fn(ctx)
    except Exception:
        traceback.print_exc()
        body = f'<p class="bad">This panel failed to render: {esc(traceback.format_exc().splitlines()[-1])}</p>'
    return f'<section id="{sid}"><h2><a class="anchor" href="#{sid}">{esc(title)}</a></h2>{body}</section>'


def table(tid, heads, rows, cls="st", box=True):
    """A sortable table (heads: th() cells; rows: <tr> strings) in a scroll box with a sticky heading."""
    t = f'<table class="{cls}" id="{tid}"><thead><tr>{"".join(heads)}</tr></thead><tbody>{"".join(rows)}</tbody></table>'
    return f'<div class="tbox" id="box-{tid}" data-sk="1">{t}</div>' if box else f'<div class="tscroll">{t}</div>'


def count_note(tid, text=""):
    return f'<div class="tb"><span class="note">{text}</span><span class="cnt" data-cnt="{tid}"></span></div>'


STATUS_CLS = {"running": "p-run", "waiting": "p-wait", "done": "p-done", "failed": "p-bad", "stopped": "p-rtd",
              "created": "p-rtd", "active": "p-run", "retired": "p-rtd"}


def pill(word, tip="", cls=None):
    t = f' data-tip="{esc(tip)}"' if tip else ""
    return f'<span class="pill {cls or STATUS_CLS.get(word, "p-rtd")}{" hlp" if tip else ""}"{t}>{esc(word)}</span>'


def dot(status):
    return f'<span class="sdot s-{esc(status)}" aria-hidden="true"></span>'


def args_list(v):
    """A line's train or self-play arguments as a list: config.json has lists; a CSV cell holds them as text."""
    if v is None or v == "":
        return []
    if isinstance(v, list):
        return [str(x) for x in v]
    s = str(v).strip()
    if s.startswith("["):
        try:
            return [str(x) for x in json.loads(s)]
        except ValueError:
            pass
    try:
        return shlex.split(s)
    except ValueError:
        return s.split()


# ---------------------------------------------------------------- facets: each line's value for each factor
# As the champion run's dashboard classifies its lineages (cycles/champ/dashboard.py: facets), from a line's
# training flags with its kind as --model and the binaries' defaults where a flag is absent; from its archetype's name
# when the flags don't settle it; "unclassified" otherwise, never a guess.

UNCL = "unclassified"
FX_MODEL = {"lr": "L", "cnbp": "N", "probit": "P", "tan": "TAN", "fm": "FM"}  # a flat --model, a tree's --leaf
FX_LETTER = {"cnbp": "N", "lr": "L", "tan": "T"}  # a router's classifier
FX_MOE = {"moe": ("L", "gate (soft)"), "moenb": ("N", "gate (soft)"), "nbgate": ("L", "gate (NB)"),
          "nbmix": ("N", "gate (NB)")}
FX_ARCH = re.compile(r"(?:F-([LNP])|(MoE|M)(?:-([LNP]{1,2})\d*-([LN]))?|(FM)|(TAN)|([TD])-([LN])-(H|[NLT]{1,2}[ces]))")
FX_ORDER = {"arch": ["F", "T", "D", "M"], "leaf": ["L", "N", "P", "TAN", "FM"],
            "router": ["Nc", "Ne", "Lc", "Le", "Te", "NNs", "NLs", "LNs", "LLs", "H", "gate (soft)", "gate (NB)", "none"],
            "split": ["none", "merge", "fresh", "pre-split"], "option": ["self-play", "local", "global", "league"],
            "status": ["running", "waiting", "created", "stopped", "done", "failed", "active", "retired"],
            "kind": ["global", "experiment"], "depth": ["d0", "d1", "d2", "d3", "d4", "n/a"],
            "labels": ["result"], "policy": ["jitter10"]}
FX_NAMES = {"arch": {"F": "F flat", "T": "T tree", "D": "D DAG", "M": "M MoE"},
            "leaf": {"L": "L (LR)", "N": "N (NB)", "P": "P (probit)"}}
GREY = {"none", UNCL, "retired", "stopped", "created", "n/a", ""}


def fx_flags(toks):
    """Flags as {flag: value}: a flag followed by another flag, or by nothing, is a switch (True)."""
    out, i = {}, 0
    while i < len(toks):
        t = toks[i]
        if t.startswith("--") and i + 1 < len(toks) and not toks[i + 1].startswith("--"):
            out[t] = toks[i + 1]
            i += 2
            continue
        if t.startswith("--"):
            out[t] = True
        i += 1
    return out


def fx_router(m):
    r1 = FX_LETTER.get(m.get("--router", "cnbp"))
    route = m.get("--route", "error")
    if r1 is None:
        return UNCL
    if "--route-switch" not in m:
        return r1 + {"cluster": "c", "error": "e"}[route] if route in ("cluster", "error") else UNCL
    r2 = FX_LETTER.get(m.get("--error-router", m.get("--router", "cnbp")))
    return r1 + r2 + "s" if r2 and route == "cluster" else UNCL


def fx_split(m):
    rev, pre, on = "--revise" in m, "--pre-split" in m, m.get("--revise-on")
    if not (rev or pre or on):
        return "none"
    if pre and not (rev or on):
        return "pre-split"
    if rev and not pre:
        return "fresh" if on == "fresh" else "merge" if on in (None, "stream") else UNCL
    return UNCL


def fx_float(x):
    try:
        return float(x)
    except (TypeError, ValueError):
        return None


def line_facets(d):
    """{arch, leaf, router, split, labels, depth, policy} of a line definition (config.json's, with lines.csv's):
    the model from its kind and training arguments, the labels from its training arguments, the self-play depth and
    move selection from its self-play arguments ("n/a" for a line that doesn't self-play)."""
    f = dict(arch=UNCL, leaf=UNCL, router=UNCL, split=UNCL, labels=UNCL, depth="n/a", policy="n/a")
    toks = args_list(d.get("train_args"))
    own = fx_flags(toks)
    m = fx_flags((["--model", str(d["kind"])] if d.get("kind") else []) + toks)
    if "--model" in own:  # its own --model, if the arguments carry one
        m["--model"] = own["--model"]
    kind, dag = m.get("--model"), "--dag-leaves" in m
    if kind in FX_MODEL and not dag:
        f.update(arch="F", leaf=FX_MODEL[kind], router="none", split="none")
    elif kind in FX_MOE and not dag:
        f.update(arch="M", leaf=FX_MOE[kind][0], router=FX_MOE[kind][1], split="none")
    elif kind == "ht" and not dag:
        f.update(arch="T", leaf=FX_MODEL.get(m.get("--leaf", "cnbp"), UNCL), router="H",
                 split="none" if fx_split(m) == "none" else UNCL)
    elif kind in ("bdt", "bdd"):  # a tree; a DAG with --dag-leaves, or a BDD
        f.update(arch="D" if dag or kind == "bdd" else "T", leaf=FX_MODEL.get(m.get("--leaf", "cnbp"), UNCL),
                 router=fx_router(m), split=fx_split(m))
    if f["arch"] == UNCL:  # the archetype's name: F-L, T-L-Ne, D-L-Nc, M-L-L, MoE ...
        mm = FX_ARCH.fullmatch(str(d.get("archetype") or d.get("model") or ""))
        if mm and mm.group(7):
            f.update(arch=mm.group(7), leaf=mm.group(8), router=mm.group(9))
        elif mm and mm.group(2):
            f.update(arch="M", leaf=mm.group(3) or "L", router="gate (NB)" if mm.group(4) == "N" else "gate (soft)",
                     split="none")
        elif mm:
            f.update(arch="F", leaf=mm.group(1) or mm.group(5) or mm.group(6), router="none", split="none")
    ld, r = m.get("--label-depth"), fx_float(m.get("--label-result", "0"))  # r: the game result's weight
    if ld is None:
        f["labels"] = "result"
    elif isinstance(ld, str) and ld.isdigit() and r is not None and 0 <= r < 1:
        f["labels"] = f"d{ld} search" if r == 0 else f"mix: d{ld} search {100 * (1 - r):g}%"
    sp = fx_flags(args_list(d.get("selfplay_args")))
    if d.get("option") == "self-play" or (sp and d.get("option") in (None, "")):
        dp = sp.get("--depth", "0")
        f["depth"] = f"d{int(dp)}" if isinstance(dp, str) and dp.isdigit() else UNCL
        pol, t = sp.get("--policy", "wld-score-jitter10"), fx_float(sp.get("--temperature", "0.01"))
        f["policy"] = ("jitter10" if pol == "wld-score-jitter10" else
                       f"softmax {t:g}" if pol == "log-odds-softmax" and t is not None else UNCL)
    return f


def line_overrides(d):
    """A line's flag overrides as [(flag without its dashes, value text)] (FORMATS.md, experiment lines):
    config.json's train_overrides and selfplay_overrides, lists of [flag, value] (true for a bare flag, null for a
    removed one), else lines.csv's "flag=value" words."""
    out = []
    for k in ("train_overrides", "selfplay_overrides"):
        v, items = d.get(k), []
        if isinstance(v, list):
            items = [(str(x[0]), x[1] if len(x) > 1 else True) for x in v if isinstance(x, (list, tuple)) and x]
        elif isinstance(v, str) and v.strip():
            for w in v.split():
                fl, eq, val = w.partition("=")
                items.append((fl, val if eq else True))
        for fl, val in items:
            fl = fl.lstrip("-")
            if fl and fl not in [o[0] for o in out]:
                out.append((fl, "on" if val is True else "off" if val is None or val is False else str(val)))
    return out


def fx_order(key, vals):
    if key.startswith("ov:") or key == "replicate":  # a swept flag's values and replicates: numerically
        def k(v):
            x = fx_float(str(v).lstrip("#"))
            return (0, 0.0, "") if v == "recipe's" else (1, x, "") if x is not None else (2, 0.0, str(v))
        return sorted(vals, key=k)
    order = FX_ORDER.get(key)
    if not order:
        return vals
    return sorted(vals, key=lambda v: (order.index(v) if v in order else len(order), vals.index(v)))


def fx_data(items, factors, by, noun, keep=(), names=None):
    """The filter bar's data (LG.fx): items {it: {v: {factor: value or [values]}, name}}; factors [(key, label)]. A
    factor with a single value on the page is left out (unless in `keep`). Colours follow the values' order, grey for
    none, unclassified and the like. `names`: {factor: {value: label}} for values shown otherwise (model ids)."""
    out = []
    for key, label in factors:
        vals = []
        for it in items.values():
            v = it["v"].get(key)
            for x in (v if isinstance(v, list) else [v]):
                if x is not None and x not in vals:
                    vals.append(x)
        if not vals or (len(vals) < 2 and key not in keep):
            continue
        vals = fx_order(key, vals)
        cols, k = [], 0
        for x in vals:
            if x in GREY:
                cols.append("var(--muted)")
            else:
                cols.append(f"var(--k{k % 8 + 1})")
                k += 1
        out.append([key, label, vals, [(names or {}).get(key, {}).get(x) or FX_NAMES.get(key, {}).get(x, x)
                                        for x in vals], cols])
    keys = [f[0] for f in out]
    return dict(factors=out, items=items, by=by if by in keys else (keys[0] if keys else None), noun=noun)


# ---------------------------------------------------------------- the leagues

def fit_cpu(t):
    """A totals.csv row's CPU for the rating fit and tables: the tournament process's own CPU minus its games'
    (tournament_proc_cpu_s, since 2026-11-09 16:00), or None without it."""
    proc, games = num(t.get("tournament_proc_cpu_s")), num(t.get("tournament_cpu_s"))
    return max(0.0, proc - (games or 0)) if proc is not None else None


class Root:
    """A root: index.json and the leagues it lists, plus any league directory on disk it doesn't list."""

    def __init__(self, path):
        self.path = path
        self.index_path = os.path.join(path, "index.json")
        idx = load(self.index_path, parse_json)
        self.index = idx if isinstance(idx, dict) else {}
        ents = [e for e in (self.index.get("leagues") or []) if isinstance(e, dict)]
        seen = {os.path.normpath(e.get("dir") or "") for e in ents}
        disk = [("global", "global")] + [(n, os.path.join("experiments", n))
                                         for n in listdir(os.path.join(path, "experiments"))]
        for name, d in disk:
            if os.path.normpath(d) not in seen and os.path.isfile(os.path.join(path, d, "config.json")):
                ents.append({"name": name, "kind": "global" if d == "global" else "experiment", "dir": d,
                             "unlisted": True})
        self.leagues = [League(self, e) for e in ents]
        self.glob = next((l for l in self.leagues if l.is_global), None)
        self.exps = [l for l in self.leagues if not l.is_global]

    def experiment(self, name):
        return next((l for l in self.exps if l.name == name), None)


class League:
    """One league directory's files (each read through the cache once per page) and what the pages derive from
    them."""

    def __init__(self, root, ent):
        self.root, self.ent = root, ent
        self.kind = ent.get("kind") or "experiment"
        self.is_global = self.kind == "global"
        self.name = ent.get("name") or os.path.basename(ent.get("dir") or "") or "?"
        self.dir = os.path.join(root.path, ent.get("dir") or
                                ("global" if self.is_global else os.path.join("experiments", self.name)))
        self.href = "/global/" if self.is_global else "/x/" + urllib.parse.quote(self.name, safe="") + "/"
        self.pre = "G" if self.is_global else "E"
        self.title = "Global league" if self.is_global else f"Experiment {self.name}"
        self.charts = {}

    def p(self, *parts):
        return os.path.join(self.dir, *parts)

    def _csv(self, name):
        return load(self.p(name), parse_csv) or []

    def _json(self, name):
        v = load(self.p(name), parse_json)
        return v if isinstance(v, dict) else {}

    @cached_property
    def config(self):
        return self._json("config.json")

    @cached_property
    def state(self):
        return self._json("state.json")

    @cached_property
    def lines(self):
        return self._csv("lines.csv")

    @cached_property
    def cycles(self):
        return self._csv("cycles.csv")

    @cached_property
    def totals(self):
        return sorted(self._csv("totals.csv"), key=lambda r: inum(r.get("cycle"), 0))

    @cached_property
    def rounds(self):
        return self._csv("rounds.csv")

    @cached_property
    def events(self):
        return self._csv("events.csv")

    @cached_property
    def history(self):
        return self._csv("history.csv")

    @cached_property
    def anchors(self):
        return self._csv("anchors.csv")

    @cached_property
    def copied(self):
        return self._csv("copied.csv")

    @cached_property
    def batch(self):
        return self._json(os.path.join("batch", "batch.json"))

    @cached_property
    def errors(self):
        """Files of this league that failed to parse (the last good reading is shown meanwhile)."""
        return {os.path.relpath(p, self.dir): e for p, e in list(_errors.items()) if p.startswith(self.dir + os.sep)}

    @cached_property
    def by_line(self):
        """{line: its cycles.csv rows in cycle order}."""
        out = {}
        for r in self.cycles:
            out.setdefault(r.get("line") or "?", []).append(r)
        for rows in out.values():
            rows.sort(key=lambda r: inum(r.get("cycle"), 0))
        return out

    @cached_property
    def cyc_rows(self):
        """{(cycle, line): its cycles.csv row}."""
        return {(inum(r.get("cycle"), 0), r.get("line")): r for r in self.cycles}

    @cached_property
    def hist(self):
        """{cycle: its history.csv rows}."""
        out = {}
        for r in self.history:
            out.setdefault(inum(r.get("cycle"), 0), []).append(r)
        return out

    @cached_property
    def tot(self):
        """{cycle: its totals.csv row}."""
        return {inum(r.get("cycle"), 0): r for r in self.totals}

    @cached_property
    def done_cycle(self):
        """The last completed cycle: state.json's, else index.json's, else the records'."""
        for c in (self.state.get("cycle"), self.ent.get("cycle")):
            if inum(c) is not None:
                return inum(c)
        return max([inum(r.get("cycle"), 0) for r in self.totals] or [0])

    @cached_property
    def rated_cycle(self):
        return max(self.hist) if self.hist else None

    @cached_property
    def planned(self):
        c = self.config.get("cycles")
        return inum(c) if c is not None else inum(self.ent.get("cycles_planned"))

    @cached_property
    def line_defs(self):
        """{line: definition}: config.json's lines in their order, with lines.csv's latest status over them and
        state.json's per-line state under "state"."""
        out = {}
        for l in self.config.get("lines") or []:
            if isinstance(l, dict) and l.get("line"):
                out[l["line"]] = dict(l)
        for r in self.lines:
            if not r.get("line"):
                continue
            d = out.setdefault(r["line"], {})
            for k, v in r.items():
                if k in ("train_args", "selfplay_args", "train_overrides", "selfplay_overrides") \
                        and isinstance(d.get(k), list) and (d.get(k) or k.endswith("_overrides")):
                    continue  # config.json's list is the exact one
                if v not in (None, "") or k not in d:
                    d[k] = v
        for ln, s in (self.state.get("lines") or {}).items():
            if isinstance(s, dict):
                out.setdefault(ln, {"line": ln})["state"] = s
        for ln, rows in self.by_line.items():  # lines known only from their records
            out.setdefault(ln, {"line": ln, "model": rows[-1].get("model"), "option": rows[-1].get("option")})
        return out

    def line_status(self, ln):
        d = self.line_defs.get(ln, {})
        return d.get("status") or (d.get("state") or {}).get("status") or "active"

    @cached_property
    def labels(self):
        """{model id: its display label} (FORMATS.md, Names): index.json's labels, then config.json's and
        lines.csv's (an id like F-L+lr-rate=0.025#2 shows as "F-L lr-rate=0.025 #2")."""
        out = {m: lab for m, lab in zip(self.ent.get("models") or [], self.ent.get("labels") or []) if m and lab}
        for d in self.line_defs.values():
            if d.get("model") and d.get("label"):
                out[d["model"]] = d["label"]
        return out

    def mlabel(self, m):
        return self.labels.get(m) or m or ""

    def disp(self, ln):
        """A line's display name: its id with the model id replaced by the model's label, so
        F-L+lr-rate=0.025#2.loc shows as "F-L lr-rate=0.025 #2.loc"; other lines as they are."""
        m = (self.line_defs.get(ln) or {}).get("model") or ""
        lab = self.labels.get(m)
        return lab + ln[len(m):] if lab and lab != m and ln and ln.startswith(m) else (ln or "")

    @cached_property
    def prices(self):
        """The prices version the league runs on (config.json prices.version, index.json, totals.csv), or None for a
        league made before prices (2026-11-09 16:40)."""
        p = self.config.get("prices")
        v = p.get("version") if isinstance(p, dict) else None
        return v or self.ent.get("prices_version") or next(
            (r.get("prices_version") for r in reversed(self.totals) if r.get("prices_version")), None)

    @cached_property
    def sp_budget(self):
        """A self-play-only experiment's fixed priced budget per line per cycle (config.json selfplay_budget_s), or
        None."""
        v = self.config.get("selfplay_budget_s")
        return num(v if v is not None else self.ent.get("selfplay_budget_s"))

    @cached_property
    def priced(self):
        """Whether the league's records carry prices (totals.csv price_s; leagues made before 16:40 don't)."""
        return any(num(r.get("price_s")) is not None for r in self.totals) or \
            any(num(r.get("price_s")) is not None for r in self.rounds)

    @cached_property
    def ov_flags(self):
        """The flags any line overrides (experiments' variants and sweeps), in the order first seen."""
        out = []
        for d in self.line_defs.values():
            for fl, _ in line_overrides(d):
                if fl not in out:
                    out.append(fl)
        return out

    @cached_property
    def facets(self):
        out = {}
        for ln, d in self.line_defs.items():
            f = line_facets(d)
            f.update(line=ln, model=d.get("model") or ln, option=d.get("option") or "", status=self.line_status(ln),
                     base=d.get("base_model") or d.get("recipe") or d.get("model") or ln,
                     replicate=f"#{inum(d.get('replicate'), 1)}")
            ov = dict(line_overrides(d))
            for fl in self.ov_flags:
                f["ov:" + fl] = ov.get(fl, "recipe's")
            out[ln] = f
        return out

    @cached_property
    def live(self):
        """(status, note, proc): state.json's status (index.json's when state.json has none; the driver writes
        both, state.json more often) checked against /proc: a running or waiting league whose driver is gone shows
        as stopped."""
        src = "state.json" if self.state.get("status") else "index.json"
        st = self.state.get("status") or self.ent.get("status")
        pid = self.state.get("pid") or self.ent.get("pid")
        proc = proc_info(pid) if pid and st in (None, "running", "waiting") else None
        if proc and proc["state"] == "Z":
            proc = None
        note = []
        if self.ent.get("unlisted"):
            note.append("not in index.json")
        if st is None:
            st = "running" if proc else ("created" if not self.totals else "stopped")
        elif st in ("running", "waiting") and not proc:
            note.append(f"{src} says {st}, but its driver (pid {pid or 'none'}) is not running")
            st = "stopped"
        if os.path.exists(self.p("STOP")):
            note.append("a STOP file asks the driver to stop at the next safe point")
        return st, "; ".join(note), proc

    @cached_property
    def cpu(self):
        """CPU spent by this league: totals.csv's sums, plus the running cycle's rounds so far (rounds.csv)."""
        keys = ("tournament_cpu_s", "train_cpu_s", "selfplay_cpu_s", "cal_cpu_s")
        out = {k: sum(num(r.get(k), 0) for r in self.totals) for k in keys}
        out["fit"] = sum(fit_cpu(r) or 0 for r in self.totals)
        last = max([inum(r.get("cycle"), 0) for r in self.totals] or [0])
        out["running"] = sum(num(r.get("cpu_s"), 0) for r in self.rounds if inum(r.get("cycle"), 0) > last)
        out["total"] = sum(out[k] for k in keys) + out["fit"] + out["running"]
        out["global"] = sum(num(r.get("global_cpu_s"), 0) for r in self.totals)
        return out

    def cpu_tip(self):
        c = self.cpu
        parts = [f"tournament games {cpu_txt(c['tournament_cpu_s'])}"]
        if c["fit"]:
            parts.append(f"rating fits and tables {cpu_txt(c['fit'])}")
        parts.append(f"training {cpu_txt(c['train_cpu_s'])}")
        if c["selfplay_cpu_s"]:
            parts.append(f"self-play {cpu_txt(c['selfplay_cpu_s'])}")
        parts.append(f"calibration {cpu_txt(c['cal_cpu_s'])}")
        if c["running"]:
            parts.append(f"the running cycle's rounds so far {cpu_txt(c['running'])}")
        out = "CPU this league spent, measured (totals.csv): " + ", ".join(parts) + "."
        pr = sum(num(r.get("price_s"), 0) for r in self.totals)
        if self.priced:
            out += (f" The tournament games' price: {price_txt(pr)} (prices {self.prices or '?'}), the measure budgets "
                    "use.")
        if c["global"]:
            out += (f" Not counted: {cpu_txt(c['global'])} of copied global rounds, which the global league spent "
                    "(the experiment trains on them).")
        return out

    @cached_property
    def started(self):
        return self.totals[0].get("start") if self.totals else None

    @cached_property
    def finished(self):
        if self.live[0] != "done":
            return None
        ev = [e for e in self.events if e.get("event") == "done"]
        return ev[-1].get("date") if ev else (self.totals[-1].get("end") if self.totals else self.ent.get("updated"))

    def time_left(self):
        """Seconds an unfinished experiment still needs at its median cycle wall time (the running cycle's elapsed
        part taken off), or None."""
        c, N, st = self.done_cycle, self.planned, self.live[0]
        if not N or not self.wall_median or c >= N or st in ("done", "failed"):
            return None
        left = (N - c) * self.wall_median
        cur = self.state.get("current") if isinstance(self.state.get("current"), dict) else {}
        if st in ("running", "waiting") and cur.get("started"):
            left -= min(self.wall_median, max(0.0, time.time() - (ptime(cur.get("started")) or time.time())))
        return max(0.0, left)

    @cached_property
    def has_global(self):
        return any((d.get("option") == "global") for d in self.line_defs.values()) or bool(self.config.get("global_source"))

    def models_options(self):
        models = list(self.ent.get("models") or [])
        options = list(self.ent.get("options") or [])
        for d in self.line_defs.values():
            if d.get("model") and d["model"] not in models:
                models.append(d["model"])
            if d.get("option") and d["option"] not in options:
                options.append(d["option"])
        return models, fx_order("option", options)

    @cached_property
    def wall_median(self):
        """The median wall time of the last (up to) five cycles, or None."""
        w = sorted(x for x in (num(r.get("wall_s")) for r in self.totals[-5:]) if x)
        return w[len(w) // 2] if w else None

    def leader(self):
        """(line, elo, version name) of the line whose current ply-2 entry rates highest in the last cycle."""
        best = None
        for ln, rows in self.by_line.items():
            r = rows[-1]
            if inum(r.get("cycle")) != self.rated_cycle and self.rated_cycle is not None:
                continue
            e = num(r.get("elo2"))
            if e is not None and (best is None or e > best[1]):
                best = (ln, e, r.get("version_name") or "")
        return best


# ---------------------------------------------------------------- shared pieces of the pages

DEC_WORDS = {"first-champion": "first champion", "new-champion": "new champion", "champion-kept": "champion kept",
             "unrated": "unrated", "retired": "retired"}
PHASES = ["copy", "waiting", "calibration", "tournament", "batch", "decide", "train", "self-play", "records"]
# the order a cycle runs its phases (FORMATS.md, state.json): global, and experiments (copy and waiting with the
# global option, self-play with a self-play line)
GLOBAL_PHASES = ["calibration", "tournament", "batch", "decide", "train", "records"]
EXP_PHASES = ["copy", "calibration", "tournament", "decide", "train", "self-play", "records"]  # waiting: inside copy
ROLE_CLS = {"champion": "p-frz", "current+champion": "p-frz", "anchor": "p-rtd"}


def rounds_done(lg, cur):
    """How many of the running cycle's rounds are played: state.json's current.rounds (per-round stats once played),
    else rounds.csv's rows of that cycle."""
    r = cur.get("rounds")
    if isinstance(r, (list, dict)) and r:
        return len(r)
    cc = inum(cur.get("cycle"))
    return sum(1 for x in lg.rounds if inum(x.get("cycle")) == cc) if cc is not None else 0


def nav(root, here=None):
    """The navigation in the sticky top bar: Home, the global league, the experiments (the latest few as links, all of
    them in a list), and links to the champion run's dashboard."""
    cur = lambda href: ' aria-current="page"' if here == href else ""
    out = [f'<a href="/"{cur("/")}>Leagues</a>']
    if root.glob:
        g = root.glob
        out.append(f'<a href="/global/"{cur("/global/")} data-tip="the global league: {esc(g.live[0])}">'
                   f'{dot(g.live[0])}Global</a>')
    exps = sorted(root.exps, key=lambda l: (l.live[0] not in ("running", "waiting"),
                                            -(ptime(l.ent.get("updated")) or 0), l.name))
    show = exps[:6]
    mine = next((l for l in exps if l.href == here), None)
    if mine and mine not in show:
        show.append(mine)
    for l in show:
        c = l.done_cycle
        out.append(f'<a href="{l.href}"{cur(l.href)} data-tip="experiment {esc(l.name)}: {esc(l.live[0])}, cycle {c}'
                   f'{" of " + str(l.planned) if l.planned else ""}">{dot(l.live[0])}{esc(l.name)}</a>')
    if len(exps) > 3:
        opts = "".join(f'<option value="{l.href}"{" selected" if l.href == here else ""}>{esc(l.name)} · '
                       f'{esc(l.live[0])}</option>' for l in sorted(exps, key=lambda l: l.name))
        out.append(f'<select data-go aria-label="go to an experiment"{" class=on" if mine else ""}>'
                   f'<option value="">all {len(exps)} experiments…</option>{opts}</select>')
    if CFG["old"]:
        old = esc(CFG["old"])
        out.append(f'<span class="sep">·</span><a href="{old}" data-tip="the champion run (cycles/champ) on its own '
                   f'dashboard, {old}">Champion run ↗</a><a href="{old}first/" data-tip="the first run (cycles/) on the '
                   f'champion run\'s dashboard, {old}first/">First run ↗</a>')
    return '<nav class="lg">' + "".join(out) + "</nav>"


def page(root, title, here, body, data):
    data = dict(data, refresh=REFRESH_S * 1000)
    blob = json.dumps(data, separators=(",", ":")).replace("</", "<\\/")
    return ("<!doctype html><html lang=en><head><meta charset=utf-8>"
            "<meta name=viewport content='width=device-width,initial-scale=1'>"
            f"<title>{esc(title)}</title><style>{CSS}{CSS2}</style></head><body><div class=\"viz-root\">"
            f'<div class="top">{nav(root, here)}<div class="fxb" id="fxb"></div></div>'
            f'<div id="app">{body}<script type="application/json" id="lg-data">{blob}</script></div>'
            '<div id="tip" hidden></div></div>'
            f"<noscript><meta http-equiv=refresh content={REFRESH_S}></noscript><script>{JS}</script></body></html>")


def stale_note():
    return '<span id="stale" hidden> · not refreshed: the server didn\'t answer</span>'


def line_cell(ln, extra=""):
    return f'<span class="fx-dot"></span> {esc(ln)}{extra}'


def chart_bar(cid, plies=None, toggles=()):
    """The buttons above a chart: its plies, and toggles (key, label, tip)."""
    b = []
    for p, lab in plies or []:
        b.append(f'<button type="button" data-cset="ply" data-ply="{p}">{esc(lab)}</button>')
    for k, lab, tip in toggles:
        b.append(f'<button type="button" data-cset="{k}" aria-pressed="false" data-tip="{esc(tip)}">{esc(lab)}</button>')
    return f'<div class="cb" data-for="{cid}">{"".join(b)}</div>'


def legend(lg, last, cid):
    """The lines under a chart, one button each (its colour and dash, its name, `last[line]`): a click highlights
    it."""
    out = []
    for ln in lg.line_defs:
        out.append(f'<button type="button" data-it="{esc(ln)}"><svg class="sw" width="26" height="10" '
                   f'aria-hidden="true"><line x1="1" x2="25" y1="5" y2="5"/></svg><span class="nm">{esc(lg.disp(ln))}</span>'
                   f'<span class="e">{esc(last.get(ln, ""))}</span></button>')
    return f'<div class="lgl" data-for="{cid}">{"".join(out)}</div>'


def bars_svg(cats, stacks, names, colours, tips, h=150, xlab="cycle"):
    """A stacked bar chart drawn here: one bar per category (cats), stacks[i] the parts' values (CPU seconds), a
    tooltip per bar. The axis' unit follows the largest bar (CPU-s, CPU-min or CPU-h)."""
    n = len(cats)
    if not n:
        return '<p class="note">Nothing yet.</p>'
    top = max(sum(s) for s in stacks) or 1.0
    scale, unit = (1, "CPU-s") if top < 600 else (60, "CPU-min") if top < 36000 else (3600, "CPU-h")
    bw = max(3, min(22, 880 // n))
    gap = 1 if bw < 8 else 2
    ml, mb, mt = 52, 22, 8
    W, H = ml + n * (bw + gap) + 8, h
    ih = H - mb - mt
    raw = top / scale / 4
    mag = 10 ** math.floor(math.log10(raw)) if raw > 0 else 1
    step = next(m * mag for m in (1, 2, 2.5, 5, 10) if m * mag >= raw)
    out = [f'<svg class="bars" width="{W}" height="{H}" role="img" aria-label="CPU by {xlab}">']
    y = 0.0
    while y <= top / scale + 1e-9:
        yy = mt + ih - y * scale / top * ih
        out.append(f'<line class="grid" x1="{ml}" x2="{W - 4}" y1="{yy:.1f}" y2="{yy:.1f}"/>'
                   f'<text x="{ml - 6}" y="{yy + 4:.1f}" text-anchor="end">{y:g}</text>')
        y += step
    out.append(f'<text x="{ml - 6}" y="{H - 4}" text-anchor="end">{unit}</text>')
    every = max(1, round(n / 12))
    for i, (c, parts) in enumerate(zip(cats, stacks)):
        x = ml + i * (bw + gap)
        out.append(f'<g data-tip="{esc(tips[i])}">')
        base = mt + ih
        for v, col in zip(parts, colours):
            hh = (v or 0) / top * ih
            if hh > 0:
                out.append(f'<rect x="{x}" y="{base - hh:.1f}" width="{bw}" height="{hh:.1f}" fill="{col}"/>')
                base -= hh
        out.append(f'<rect class="hit" x="{x}" y="{mt}" width="{bw + gap}" height="{ih}"/></g>')
        if i % every == 0 or i == n - 1:
            out.append(f'<text x="{x + bw / 2:.1f}" y="{H - 6}" text-anchor="middle">{esc(str(c))}</text>')
    out.append("</svg>")
    key = "".join(f'<span class="key"><span class="sw" style="background:{c}"></span>{esc(nm)}</span>'
                  for nm, c in zip(names, colours))
    return f'<div class="keys">{key}</div><div class="tscroll">{"".join(out)}</div>'


# ---------------------------------------------------------------- Home

def home_page(root):
    n = len(root.leagues)
    run = sum(1 for l in root.leagues if l.live[0] in ("running", "waiting"))
    body = [f'<h1>Leagues</h1><p class="sub">{esc(root.path)} · {n} league{"s" if n != 1 else ""}'
            f'{f", {run} running" if run else ""} · index.json updated {tstamp(root.index.get("updated"))}'
            f'{stale_note()}</p>']
    if not root.leagues:
        body.append(f'<section><p class="note">No leagues yet: {esc(root.index_path)} '
                    f'{"lists none" if root.index else "does not exist"}. The engine (leagues.py) writes it when a '
                    'league is created.</p></section>')
    else:
        body.append(section("Leagues", panel_leagues, root, "leagues"))
        body.append(section("Recent events", panel_recent, root, "recent"))
    body.append(section("Earlier runs", panel_archives, root, "archives"))
    items = {}
    for l in root.leagues:
        models, options = l.models_options()
        items[l.href] = dict(v=dict(kind=l.kind, status=l.live[0], model=models, option=options), name=l.name)
    names = {}
    for l in root.leagues:
        names.update(l.labels)
    fx = fx_data(items, [("kind", "Kind"), ("status", "Status"), ("model", "Model"), ("option", "Option")], "status",
                 "leagues", keep=("status",), names={"model": names})
    return page(root, "Leagues", "/", "".join(body), dict(fxkey="home", fx=fx, charts={}))


def panel_leagues(root):
    heads = [th("League", "name", "text"), th("Kind", "kind", "text"), th("Status", "status", "text"),
             th("Now", "now", "text", tip="the driver's phase (state.json) and the running cycle's rounds"),
             th("Cycle", "cycle", tip="the last completed cycle, of the planned number (experiments)"),
             th("Lines", "lines", tip="models × options (experiments); lines active and retired (global)"),
             th("Started", "start", tip="cycle 1's start (totals.csv), else when the league was created"),
             th("Finished", "end", tip="done experiments: when they finished; running ones: ≈ when they should, at "
                                         "their median cycle time"),
             th("Prices", "prices", "text", tip="the prices version (leagues/prices.csv) whose constants the league "
                                                 "froze at create: its budgets are priced CPU"),
             th("CPU", "cpu", tip="what the league spent, measured: tournaments, training, self-play, calibration"),
             th("Leading line", "lead", tip="the line whose current version rates highest at ply 2 in the last "
                                             "cycle")]
    rows = []
    for l in sorted(root.leagues, key=lambda l: (not l.is_global, l.live[0] not in ("running", "waiting"),
                                                -(ptime(l.ent.get("updated")) or 0))):
        st, note, proc = l.live
        s = l.state
        cur = s.get("current") if isinstance(s.get("current"), dict) else {}
        phase = s.get("phase") or ""
        now = ""
        if st in ("running", "waiting") and phase:
            now = esc(phase)
            if phase == "tournament" and cur.get("rounds_planned"):
                now += f' {rounds_done(l, cur)}/{inum(cur.get("rounds_planned"))}'
            gw = s.get("global_wait")
            if phase == "waiting" and isinstance(gw, dict) and gw.get("since"):
                now += f' since {tstamp(gw.get("since"))}'
            if cur.get("cycle"):
                now = f'cycle {inum(cur.get("cycle"))}: ' + now
        c, N = l.done_cycle, l.planned
        cyc = (f'<span class="prog">{c} of {N} {meter(c / N if N else 0)}</span>' if N else str(c))
        models, options = l.models_options()
        nl = len(l.line_defs) or inum(l.ent.get("lines"), 0)
        if l.is_global:
            ret = sum(1 for ln in l.line_defs if l.line_status(ln) == "retired")
            lines = f"{nl - ret} active" + (f", {ret} retired" if ret else "")
            ltip = "models: " + ", ".join(models)
        else:
            lines = f"{len(models)} × {len(options)}: {nl} lines"
            ltip = f"models {', '.join(l.mlabel(m) for m in models)} × options {', '.join(options)}"
        lead = l.leader()
        fin, left = l.finished, l.time_left()
        if fin:
            fin_cell = td(tstamp(fin), ptime(fin), raw=True)
        elif left is not None and st in ("running", "waiting"):
            eta = time.time() + left
            fin_cell = td(f'<span class="muted">≈ {tstamp(datetime.datetime.fromtimestamp(eta).strftime("%Y-%m-%d %H:%M:%S"))}'
                          f'</span>', eta, raw=True, tip=f"projected: {N - c} cycle{'s' if N - c != 1 else ''} left at the median wall time")
        else:
            fin_cell = td("—", None)
        rows.append(
            f'<tr data-it="{esc(l.href)}">'
            + td(f'<span class="fx-dot"></span> <a href="{l.href}"><b>{esc(l.name)}</b></a>', l.name, raw=True)
            + td(l.kind) + td(pill(st, note), st, raw=True) + td(now or "—", raw=True)
            + td(cyc, c, raw=True, cls="r") + td(lines, nl, tip=ltip)
            + td(tstamp(l.started or l.ent.get("created")), ptime(l.started or l.ent.get("created")), raw=True)
            + fin_cell
            + td(l.prices or "—", l.prices or "", tip=(f"self-play budget {price_txt(l.sp_budget)} per line per cycle"
                                                       if l.sp_budget is not None else ""))
            + td(cpu_txt(l.cpu["total"]) if l.totals or l.rounds else "—", l.cpu["total"], cls="r", tip=l.cpu_tip())
            + td(f"{esc(l.disp(lead[0]))} {plain(lead[1])}" if lead else "—", lead[1] if lead else None, raw=True,
                 tip=f"{l.disp(lead[0])} {lead[2]}: Elo {plain(lead[1])} at ply 2 in cycle {l.rated_cycle}" if lead
                 else "")
            + "</tr>")
    return (count_note("t-leagues", "Each league's page is one click away; the bar at the top reaches every league "
                       "from any page.") + table("t-leagues", heads, rows, box=False))


EVENT_WORDS = {"entered": "entered", "first-champion": "first champion", "new-champion": "new champion",
               "retired": "retired", "anchor-locked": "anchor locked", "anchor-available": "anchor available",
               "anchor-replaced": "anchor replaced", "copied": "global rounds copied", "wait-start": "waiting",
               "wait-end": "wait over", "done": "done"}


def event_value(e):
    ev, v = e.get("event"), e.get("value")
    if v in (None, ""):
        return ""
    if ev == "new-champion" and num(v) is not None:
        return f"z {num(v):.2f}"
    if ev == "anchor-locked" and num(v) is not None:
        return f"[{inum(v)}, {inum(v) + 100})"
    return str(v)


def panel_recent(root):
    evs = []
    for l in root.leagues:
        for e in l.events:
            evs.append((ptime(e.get("date")) or 0, l, e))
    evs.sort(key=lambda x: -x[0])
    if not evs:
        return '<p class="note">No events yet.</p>'
    rows = []
    for t, l, e in evs[:40]:
        rows.append(f'<tr data-it="{esc(l.href)}">' + td(tstamp(e.get("date")), t, raw=True)
                    + td(f'<span class="fx-dot"></span> <a href="{l.href}#events">{esc(l.name)}</a>', l.name, raw=True)
                    + td(e.get("cycle") or "", inum(e.get("cycle")), cls="r") + td(e.get("line") or "")
                    + td(EVENT_WORDS.get(e.get("event"), e.get("event") or ""))
                    + td(e.get("version_name") or "", tip=e.get("entry") or "") + td(event_value(e))
                    + td(e.get("detail") or "", cls="wrap") + "</tr>")
    heads = [th("When", "t"), th("League", "lg", "text"), th("Cycle", "c", cls="r"), th("Line", "ln", "text"),
             th("Event", "ev", "text"), th("Version", "v", "text"), th("Value", "val", "text"), th("Detail")]
    return count_note("t-recent", f"The latest {min(40, len(evs))} of {len(evs)} events across the leagues "
                      "(events.csv).") + table("t-recent", heads, rows)


def listening(url):
    """True or False when url is a local http address: whether a socket listens on its port (/proc/net/tcp, so
    nothing connects to it); None for other addresses."""
    m = re.fullmatch(r"http://(127\.0\.0\.1|localhost)(?::(\d+))?(/.*)?", url or "")
    if not m:
        return None
    port = int(m.group(2) or 80)
    try:
        with open("/proc/net/tcp") as f:
            next(f)
            return any(r.split()[1].endswith(f":{port:04X}") and r.split()[3] == "0A" for r in f)
    except (OSError, StopIteration, IndexError):
        return None


def panel_archives(root):
    if not CFG["old"]:
        return '<p class="note">No link to the champion run\'s dashboard (--old-dashboard "").</p>'
    old, up = esc(CFG["old"]), listening(CFG["old"])
    return ('<ul class="notes">'
            f'<li><a href="{old}">The champion run</a> (cycles/champ, selfplay/league-champ) and '
            f'<a href="{old}first/">the first run</a> (cycles/, selfplay/league-cycles) are on the champion run\'s own '
            f'dashboard, {old}.'
            + (" It is not running now, so the links lead nowhere." if up is False else "") + "</li></ul>")


# ---------------------------------------------------------------- a league's page (global or experiment)

def league_page(root, lg):
    st, note, proc = lg.live
    c, N = lg.done_cycle, lg.planned
    models, options = lg.models_options()
    if lg.is_global:
        sub = f"{len(lg.line_defs)} lines · cycle {c}"
    else:
        ml = [lg.mlabel(m) for m in models]
        sub = ((f"{', '.join(ml) if len(ml) <= 6 else f'{len(ml)} models'} × {', '.join(options)}" if models
                else "no lines") + f" · cycle {c}" + (f" of {N}" if N else ""))
    sub = esc(sub)
    if lg.prices:
        sub += (f' · <span data-tip="the prices version (leagues/prices.csv) whose constants the league froze at '
                f'create: budgets are priced CPU, deterministic, so a league replays exactly">prices '
                f'{esc(lg.prices)}</span>')
    if lg.sp_budget is not None:
        sub += f" · self-play budget {esc(price_txt(lg.sp_budget))} per line per cycle"
    cur = lg.state.get("current") if isinstance(lg.state.get("current"), dict) else {}
    if st in ("running", "waiting") and cur.get("cycle"):
        sub += f" · cycle {inum(cur.get('cycle'))} {esc(lg.state.get('phase') or '')}"
    secs = [("now", "Now", panel_now)]
    if not lg.is_global:
        secs.append(("grid", "By model and option", panel_grid))
    secs += [("elo", "Elo by cycle", panel_elo), ("standings", "Standings", panel_standings),
             ("lines", "Lines", panel_lines)]
    if lg.is_global:
        secs.append(("anchors", "Anchors", panel_anchors))
    secs.append(("events", "Events", panel_events))
    if not lg.is_global:
        secs.append(("data", "Training data, priced CPU" if lg.priced else "Training data CPU", panel_data_cpu))
        if lg.has_global:
            secs.append(("copied", "Global rounds copied", panel_copied))
    secs += [("cycles", "Cycles", panel_cycles), ("rounds", "Rounds", panel_rounds)]
    if lg.is_global:
        secs.append(("batch", "Latest batch", panel_batch))
    secs.append(("reports", "Reports and log", panel_reports))
    toc = " · ".join(f'<a href="#{sid}">{esc(t)}</a>' for sid, t, _ in secs)
    errs = lg.errors
    body = [f'<h1>{esc(lg.title)} {pill(st, note)}</h1><p class="sub">{sub} · {esc(lg.dir)}{stale_note()}</p>'
            f'<p class="toc">{toc}</p>']
    if not os.path.isdir(lg.dir):
        body.append(f'<section><p class="bad">The league\'s directory {esc(lg.dir)} does not exist.</p></section>')
    if errs:
        body.append('<section><p class="warn">Files that failed to parse (their last good reading is shown): '
                    + "; ".join(f"{esc(k)}: {esc(v)}" for k, v in errs.items()) + "</p></section>")
    body += [section(t, fn, lg, sid) for sid, t, fn in secs]
    items = {ln: dict(v=dict(f), name=lg.disp(ln)) for ln, f in lg.facets.items()}
    if lg.is_global:
        facs = [("line", "Line"), ("status", "Status"), ("arch", "Architecture"), ("leaf", "Leaf evaluator"),
                ("router", "Router"), ("split", "Split management"), ("labels", "Training labels")]
        by = "line" if len(items) <= 8 else "arch"
        fx = fx_data(items, facs, by, "lines", keep=("line",))
    else:
        ovs = [("ov:" + fl, fl) for fl in lg.ov_flags]
        facs = ([("model", "Model")] + ovs + [("replicate", "Replicate"), ("base", "Base model"), ("option", "Option"),
                                              ("depth", "Self-play depth"), ("policy", "Self-play move selection"),
                                              ("arch", "Architecture"), ("leaf", "Leaf evaluator"), ("router", "Router"),
                                              ("split", "Split management"), ("labels", "Training labels")])
        # colour by a swept flag when there is one, else by option (several), else by model
        by = next((k for k, _ in ovs if len({f.get(k) for f in lg.facets.values()}) > 1), None) or \
            ("option" if len(options) > 1 else "model")
        fx = fx_data(items, facs, by, "lines", keep=("model", "option"), names={"model": lg.labels})
    return page(root, f"{lg.title} · leagues", lg.href, "".join(body),
                dict(fxkey="global" if lg.is_global else "experiment", fx=fx, charts=lg.charts))


def panel_now(lg):
    st, note, proc = lg.live
    s = lg.state
    cur = s.get("current") if isinstance(s.get("current"), dict) else {}
    phase = s.get("phase") or "—"
    running = st in ("running", "waiting")
    tiles = []
    ssub = []
    if proc:
        ssub.append(f'driver pid {proc["pid"]}, up {fmt_dur(time.time() - proc["start"])}')
    if note:
        ssub.append(esc(note))
    ssub.append(f"state.json {tstamp(s.get('updated'))}" if s.get("updated") else
                "state.json unreadable" if os.path.exists(lg.p("state.json")) else "no state.json yet")
    tiles.append(tile("Status", f'{pill(st, note)} <span class="ph">{esc(phase if running and phase != st else "")}'
                                f'</span>',
                      " · ".join(ssub)))
    c, N = lg.done_cycle, lg.planned
    csub = []
    if running and cur.get("cycle"):
        rp = inum(cur.get("rounds_planned"))
        csub.append(f'cycle {inum(cur.get("cycle"))} started {tstamp(cur.get("started"))}'
                    + (f", round {min(rounds_done(lg, cur) + 1, rp)} of {rp}" if rp and phase == "tournament"
                       else ""))
    if N:
        tiles.append(tile("Cycles done", f'{c} <span class="of">of {N}</span>',
                          meter(c / N, f"{c} of {N} planned cycles") + " " + " · ".join(csub)))
    else:
        tiles.append(tile("Cycles done", str(c), " · ".join(csub)))
    nl = len(lg.line_defs)
    ret = sum(1 for ln in lg.line_defs if lg.line_status(ln) == "retired")
    models, options = lg.models_options()
    tiles.append(tile("Lines", str(nl - ret) if lg.is_global else str(nl),
                      (f"{ret} retired" if ret else "none retired") if lg.is_global else
                      f"{len(models)} model{'s' if len(models) != 1 else ''} × {len(options)} option{'s' if len(options) != 1 else ''}"))
    if lg.rated_cycle is not None:
        h = lg.hist[lg.rated_cycle]
        na = sum(1 for r in h if r.get("role") == "anchor")
        nr = sum(1 for r in h if r.get("role") == "reference")
        tiles.append(tile("Field", str(len(h)), f"cycle {lg.rated_cycle}: " + (f"{na} anchors, " if lg.is_global
                                                                              else "") + f"{nr} references"))
    tiles.append(tile("CPU spent", cpu_txt(lg.cpu["total"]), "measured, all cycles"
                      + (", the running one's rounds included" if lg.cpu["running"] else ""), lg.cpu_tip()))
    if not lg.is_global and lg.priced:
        # the last cycle that trained (an experiment's last cycle trains nothing)
        t = next((r for r in reversed(lg.totals) if not (lg.planned and inum(r.get("cycle"), 0) >= lg.planned)),
                 lg.totals[-1] if lg.totals else {})
        if lg.sp_budget is not None:
            tiles.append(tile("Data budget", price_txt(lg.sp_budget), "per line per cycle, fixed (self-play only), "
                              "plus the line's carry", "config.json selfplay_budget_s: each self-play line generates "
                              "this much priced CPU of games per cycle, plus what it under- or overspent before"))
        elif num(t.get("data_price_target_s")) is not None:
            tiles.append(tile("Data budget", price_txt(num(t.get("data_price_target_s"))),
                              f"per line in cycle {t.get('cycle')}: the price of its qualifying games",
                              "totals.csv data_price_target_s: what each line trains on, the priced CPU of the "
                              "cycle's qualifying games; measured "
                              f"{cpu_txt(num(t.get('data_cpu_target_s')))} (information only)"))
    if lg.totals:
        t = lg.totals[-1]
        tiles.append(tile("Last cycle", fmt_dur(num(t.get("wall_s"))),
                          f'cycle {t.get("cycle")} ended {tstamp(t.get("end"))}'
                          + (f" · median of the last 5: {fmt_dur(lg.wall_median)}" if lg.wall_median else "")))
    left = lg.time_left()
    if left is not None:
        k = N - c
        over = running and cur.get("started") and left < 1
        tiles.append(tile("Time left", "any time now" if over else f"about {fmt_dur(left)}",
                          f"{k} cycle{'s' if k != 1 else ''} at the median wall time"
                          + ("; the running cycle is past it" if over else
                             ", less the running cycle's time so far" if running and cur.get("started")
                             else "" if running else ", once it runs again")))
    if lg.has_global:
        nc = len(lg.copied)
        w = sum(num(r.get("wait_s"), 0) for r in lg.totals)
        gp = sum(num(r.get("global_price_s"), 0) for r in lg.totals)
        gc = num(s.get("global_carry_s"))
        tiles.append(tile("Global rounds", str(nc), (f"copied, {price_txt(gp)} of games" if lg.priced else
                                                     f"copied, {cpu_txt(lg.cpu['global'])} of games")
                          + (f" · waited {fmt_dur(w)}" if w else "")
                          + (f" · global data {'leads' if gc > 0 else 'trails'} by {price_txt(abs(gc))}" if gc else ""),
                          f"measured {cpu_txt(lg.cpu['global'])}; state.json global_carry_s: the priced CPU by which "
                          "the global lines' qualifying data has led the local lines' so far (the copy step aims at "
                          "the cycle's local data minus it)" if lg.priced else ""))
    out = f'<div class="tiles">{"".join(tiles)}</div>'
    if running and phase in PHASES:
        steps = GLOBAL_PHASES if lg.is_global else [
            p for p in EXP_PHASES if (p != "copy" or lg.has_global)
            and (p != "self-play" or any(d.get("option") == "self-play" for d in lg.line_defs.values()))]
        at = steps.index("copy" if phase == "waiting" else phase) if ("copy" if phase == "waiting" else phase) \
            in steps else -1
        # an experiment's last cycle copies nothing and trains nothing (no later cycle would rate the versions)
        last = not lg.is_global and lg.planned and inum(cur.get("cycle"), 0) >= lg.planned
        skip = {"copy", "train", "self-play"} if last else set()
        lab = {"tournament": f"tournament {rounds_done(lg, cur)}/{inum(cur.get('rounds_planned'), 0)}",
               "copy": "copy: waiting for global rounds" if phase == "waiting" else "copy"}

        def cls(i, p):
            return (" class=on" if i == at else ' class=skip title="skipped: the experiment&#39;s last cycle trains '
                    'nothing"' if p in skip else " class=done" if i < at else "")
        out += ('<ol class="phases" aria-label="the cycle\'s phases">' + "".join(
            f'<li{cls(i, p)}>{esc(lab.get(p, p) if i == at else p)}</li>' for i, p in enumerate(steps)) + "</ol>")
        if last:
            out += ('<p class="note">The experiment\'s last planned cycle: it rates and decides, and copies, trains '
                    'and self-plays nothing.</p>')
    gw = s.get("global_wait")
    if running and isinstance(gw, dict) and gw.get("since"):
        out += (f'<p class="note">Waiting for unused global rounds since {tstamp(gw.get("since"))} '
                f'({fmt_dur(time.time() - (ptime(gw.get("since")) or time.time()))})'
                + (f', for cycle {esc(str(gw.get("cycle")))}' if gw.get("cycle") is not None else "") + '.</p>')
    if running and phase == "tournament" and cur.get("cycle"):
        cc = inum(cur.get("cycle"))
        rr = [r for r in lg.rounds if inum(r.get("cycle")) == cc]
        if rr:
            out += (f'<p class="note">Cycle {cc} so far: {len(rr)} rounds, '
                    f'{sum(inum(r.get("games"), 0) for r in rr):,} games, '
                    f'{cpu_txt(sum(num(r.get("cpu_s"), 0) for r in rr))}.</p>')
    log = tail(lg.p("driver.log"), 6000).splitlines()[-14:]
    if log:
        out += (f'<details id="log-{esc(lg.kind)}"><summary>driver.log, the last {len(log)} lines</summary>'
                f'<pre class="log">{esc(chr(10).join(log))}</pre></details>')
    return out


def panel_elo(lg):
    cid = "elo"
    series = {"2": [], "1": []}
    last = {}
    for ln, rows in lg.by_line.items():
        for ply, ek, ck in (("2", "elo2", "ci2"), ("1", "elo1", "ci1")):
            pts, rings, champ, prev = [], [], [], None
            for r in rows:
                c, e = inum(r.get("cycle")), num(r.get(ek))
                if c is None or e is None:
                    continue
                dec = r.get("decision") or ""
                tip = r.get('version_name') or ''
                if dec in ("new-champion", "first-champion", "retired", "unrated"):
                    tip += f" · {DEC_WORDS.get(dec, dec)}" + (f" as {r.get('new_name')}" if r.get("new_name") else "")
                p = [c, round(e), round(num(r.get(ck), 0)), tip]
                if prev is not None and c != prev + 1:
                    p.append(1)  # a gap: the line starts again
                pts.append(p)
                prev = c
                if dec == "new-champion":
                    rings.append([c, round(e)])
                ce = num(r.get("champion_elo"))
                if ply == "2" and ce is not None and dec not in ("first-champion", ""):
                    champ.append([c, round(ce), round(num(r.get("champion_ci"), 0)),
                                  f"champion {r.get('champion_name') or ''}"])
            if pts:
                series[ply].append(dict(it=ln, name=lg.disp(ln), pts=pts, rings=rings))
                if ply == "2":
                    last[ln] = plain(pts[-1][1])
            if champ:
                series[ply].append(dict(it=ln, name=f"{ln}: champion", champ=1, dash=1, pts=champ))
    if not series["2"] and not series["1"]:
        return '<p class="note">No cycle has finished yet: the chart starts with cycle 1\'s ratings.</p>'
    refs = []
    if lg.rated_cycle is not None:
        refs = [[round(num(r.get("elo"), 0), 1), r.get("entry") or ""] for r in lg.hist[lg.rated_cycle]
                if r.get("role") == "reference" and num(r.get("elo")) is not None]
    lg.charts[cid] = dict(plies=["2", "1"], ply="2", xlab="cycle", ylab="Elo", refs=refs, series=series, h=360,
                          dashBy=[] if lg.is_global else ["replicate", "model", "option"],
                          defaults=dict(rings=sum(1 for x in series["2"] if not x.get("champ")) <= 12))
    return (chart_bar(cid, [("2", "ply 2"), ("1", "ply 1")],
                      [("champ", "champions", "each line's champion at ply 2 before the cycle's decision, dashed"),
                       ("refs", "references", "the random players at plies 0-3, as rated in the last cycle"),
                       ("rings", "new champions", "a ring where a version was crowned (2 SE above its champion)")])
            + f'<div class="lgc" data-chart="{cid}"></div>' + legend(lg, last, cid)
            + '<p class="note">Each line\'s current version per cycle (cycles.csv), on the scale where random is 0; '
              'every cycle is its own fit. Hover for the version, click a line (or its name) to highlight it with its '
              '95% ±.</p>')


def panel_standings(lg):
    c = lg.rated_cycle
    if c is None:
        return '<p class="note">No ratings yet: the first appear when cycle 1 ends (history.csv).</p>'
    rows = sorted(lg.hist[c], key=lambda r: -(num(r.get("elo"), -1e9)))
    prevs = [x for x in lg.hist if x < c]
    pc = max(prevs) if prevs else None
    prev = {r.get("entry"): num(r.get("elo")) for r in lg.hist.get(pc, [])}
    prev_cur = {}  # (line, ply): the previous cycle's current entry's rating
    for r in lg.hist.get(pc, []):
        if (r.get("role") or "").startswith("current"):
            prev_cur[(r.get("line"), r.get("ply"))] = num(r.get("elo"))
    out = []
    for i, r in enumerate(rows, 1):
        ln, role, ply, e = r.get("line") or "", r.get("role") or "", r.get("ply") or "", num(r.get("elo"))
        ent = r.get("entry") or ""
        if ent in prev and prev[ent] is not None and e is not None:
            ch, cht = e - prev[ent], f"since cycle {pc}: the same entry's move"
        elif role.startswith("current") and prev_cur.get((ln, ply)) is not None and e is not None:
            ch, cht = e - prev_cur[(ln, ply)], f"against the line's previous version at ply {ply} in cycle {pc}"
        else:
            ch, cht = None, ""
        vs, vsv, vst = "", None, ""
        cr = lg.cyc_rows.get((c, ln))
        if cr and role.startswith("current") and ent == cr.get("entry2") and cr.get("diff"):
            d, z = num(cr.get("diff")), num(cr.get("z"))
            vs = gap_words(d, cr.get("champion_name") or "its champion") + (f", z {z:.1f}" if z is not None else "")
            vsv, vst = d, f"current@2 minus champion@2 before the decision; {DEC_WORDS.get(cr.get('decision'), '')}"
        rcls = ROLE_CLS.get(role)
        rtxt = pill(role, cls=rcls) if rcls else esc(role)
        attr = f' data-it="{esc(ln)}"' if ln and role != "reference" else ' class="ref"'
        out.append(f"<tr{attr}>" + td(i, i, cls="r") + td(line_cell(lg.disp(ln)) if ln else "", lg.disp(ln), raw=True,
                                                          tip=ln if ln != lg.disp(ln) else "")
                   + td(r.get("version_name") or ent, inum(r.get("version")), tip=ent) + td(ply, inum(ply), cls="r")
                   + td(rtxt, role, raw=True) + td(plain(e), e, cls="r")
                   + td(f"±{num(r.get('ci95'), 0):.0f}", num(r.get("ci95")), cls="r")
                   + td(signed(ch), ch, cls="r", tip=cht) + td(f"{inum(r.get('games'), 0):,}", inum(r.get("games")),
                                                               cls="r")
                   + td(vs, vsv, tip=vst) + "</tr>")
    heads = [th("#", "rk", cls="r"), th("Line", "line", "text"), th("Version", "ver", tip="the version as named "
                                                                    "while it played; the entry id in its tooltip"),
             th("Ply", "ply", cls="r"), th("Role", "role", "text"), th("Elo", "elo", cls="r"),
             th("±", "ci", cls="r", tip="the tournament's 95% interval"),
             th("Change", "ch", cls="r", tip="since the previous cycle: a current entry against the line's previous "
                                               "version at the same ply; any other entry, its own move"),
             th("Games", "g", cls="r"), th("Against its champion", "vs", tip="current ply-2 entries: the gap to the "
                                                                              "champion and its z")]
    running = lg.live[0] in ("running", "waiting") and lg.state.get("phase") not in (None, "idle")
    return (count_note("t-stand", f"Every entry in cycle {c}'s field (history.csv), by Elo"
                       + (" (a cycle is running: these are the last completed one's)" if running else "") + ".")
            + table("t-stand", heads, out))


def panel_lines(lg):
    pc = lg.config.get("patience_cycles")
    rows = []
    for ln, d in lg.line_defs.items():
        st = d.get("state") or {}
        hist = lg.by_line.get(ln, [])
        r = hist[-1] if hist else {}
        dec = r.get("decision") or ""
        e2, ci2 = num(r.get("elo2")), num(r.get("ci2"))
        if dec in ("new-champion", "first-champion"):
            ce = e2
        else:
            ce = num(r.get("champion_elo"))
        champ = d.get("champion_name") or r.get("new_name") or r.get("champion_name") or ""
        diff, z = num(r.get("diff")), num(r.get("z"))
        crowned = st.get("crowned") if isinstance(st.get("crowned"), list) else None
        ncrown = len(crowned) - 1 if crowned else sum(1 for x in hist if x.get("decision") == "new-champion")
        status = lg.line_status(ln)
        stt = pill(status) + (f' <span class="muted">cycle {esc(str(d.get("retired_cycle")))}</span>'
                              if status == "retired" and d.get("retired_cycle") not in (None, "") else "")
        letter = "" if lg.is_global else {"self-play": "S.", "local": "L.", "global": "G."}.get(d.get("option"), "")
        nxt = d.get("version_name") or (f"{lg.pre}.{letter}{st.get('major')}.{st.get('version')}" if st else "")
        if status == "retired":
            nxt = "—"
        args = " ".join(args_list(d.get("train_args")))
        sp = " ".join(args_list(d.get("selfplay_args")))
        recipe_tip = ((f"recipe {d.get('recipe')} (recipes.txt)\n" if d.get("recipe") else "")
                      + f"train: {d.get('train_binary') or ''} {args}".strip()
                      + (f"\nself-play: {d.get('selfplay_binary') or ''} {sp}" if sp else "")
                      + (f"\nown calibration on the {d.get('heldout')} held-out set" if d.get("heldout") else "")
                      + (f"\nself-play version {st.get('sp_version')}, carry "
                         + (price_txt(num(st.get("carry_s"))) if lg.priced else cpu_txt(num(st.get("carry_s"))))
                         + (f", mean round price {price_txt(num(st.get('sp_round_price_s')))}"
                            if num(st.get("sp_round_price_s")) is not None else "")
                         + (f", {inum(d.get('selfplay_games'))} games per round" if inum(d.get("selfplay_games"))
                            else "") if sp and st and st.get("sp_version") is not None else "")
                      + (f"\nmodel id {d.get('model')}, replicate {d.get('replicate')} (seed offset "
                         f"{d.get('seed_offset')})" if d.get("model") and d.get("model") != lg.mlabel(d.get("model"))
                         else ""))
        cells = [td(line_cell(lg.disp(ln)), lg.disp(ln), raw=True, tip=ln if ln != lg.disp(ln) else "")]
        if not lg.is_global:
            cells += [td(lg.mlabel(d.get("model")), tip=d.get("model") if lg.mlabel(d.get("model")) != d.get("model")
                         else ""), td(d.get("option") or "")]
        cells += [td(stt, status, raw=True), td(nxt, inum(d.get("version"))),
                  td((r.get("version_name") or "") + (f" · {plain(e2)} ±{ci2:.0f}" if e2 is not None else ""),
                     e2, tip=f"cycle {r.get('cycle')}: {r.get('entry2') or ''}" if r else ""),
                  td(champ + (f" · {plain(ce)}" if ce is not None else ""), ce,
                     tip="the champion now, and its ply-2 Elo in the last cycle"),
                  td(gap_words(diff, r.get("champion_name") or "the champion") if diff is not None else "—", diff,
                     tip="the last cycle's current@2 minus the champion@2 before the decision"),
                  td(f"{z:.2f}" if z is not None else "—", z, cls="r",
                     tip=f"crowned at z ≥ {lg.config.get('champion_se', 2)}"),
                  td(DEC_WORDS.get(dec, dec) or "—", dec)]
        if lg.is_global:
            p = inum(d.get("patience"), inum(st.get("patience")))
            cells.append(td((meter(p / pc if pc else 0) + f"{p} of {pc}") if p is not None and pc else "—", p,
                            raw=True, tip="cycles since its last new champion; retired at the limit"))
        cells += [td(ncrown, ncrown, cls="r", tip="new champions crowned (the first, untrained one not counted)"
                     + (": " + ", ".join(f"v{x}" for x in crowned) if crowned else "")),
                  td(d.get("entered_cycle") or "", inum(d.get("entered_cycle")), cls="r")]
        if not lg.is_global:
            dk = "data_price_s" if lg.priced else "data_cpu_s"
            rd = next((x for x in reversed(hist) if num(x.get(dk)) is not None), {})
            dc = num(rd.get(dk))
            cells.append(td(cpu_txt(dc) if dc is not None else "—", dc, cls="r",
                            tip=(f"cycle {rd.get('cycle')}, the last it trained: the "
                                 + ("priced CPU of the games it trained on; measured "
                                    f"{cpu_txt(num(rd.get('data_cpu_s')))} (information only)" if lg.priced else
                                    "generation CPU of the games it trained on")) if rd else "no training yet"))
        cells.append(td((args[:48] + "…" if len(args) > 48 else args) or "—", args, tip=recipe_tip, cls="code"))
        rows.append(f'<tr data-it="{esc(ln)}">' + "".join(cells) + "</tr>")
    heads = [th("Line", "line", "text")]
    if not lg.is_global:
        heads += [th("Model", "model", "text"), th("Option", "opt", "text")]
    heads += [th("Status", "st", "text"), th("Plays next", "next", tip="the version that plays the next cycle"),
              th("Last played", "e2", tip="the version that played the last cycle, with its ply-2 Elo"),
              th("Champion", "ce", tip="the champion now, with its ply-2 Elo in the last cycle"),
              th("Gap", "gap", tip="last cycle: current@2 against the champion before the decision"),
              th("z", "z", cls="r"), th("Decision", "dec", "text")]
    if lg.is_global:
        heads.append(th("Patience", "pat", tip=f"cycles since the line's last new champion, of {pc}"))
    heads += [th("Crowned", "cr", cls="r"), th("Entered", "ent", cls="r", tip="the cycle it first played")]
    if not lg.is_global:
        heads.append(th("Data, priced" if lg.priced else "Data CPU", "dc", cls="r",
                        tip="the games it trained on in its last training cycle"))
    heads.append(th("Recipe", "args", "text", tip="the training arguments; the binaries in the tooltip"))
    return count_note("t-lines", "One row per line (lines.csv, state.json, the line's last cycles.csv row).") + table(
        "t-lines", heads, rows)


def panel_grid(lg):
    """An experiment's model × option grid: each cell the line's current version and its last ply-2 rating."""
    models, options = lg.models_options()
    if not lg.line_defs:
        return '<p class="note">No lines (config.json lists none).</p>'
    cell = {(d.get("model"), d.get("option")): ln for ln, d in lg.line_defs.items()}
    best = {}
    for m in models:
        es = [(num(lg.by_line.get(cell.get((m, o)), [{}])[-1].get("elo2")), o) for o in options if cell.get((m, o))]
        es = [x for x in es if x[0] is not None]
        if len(es) > 1:
            best[m] = max(es)[1]
    head = "<tr><th>Model</th>" + "".join(f"<th>{esc(o)}</th>" for o in options) + "</tr>"
    body = []
    for m in models:
        tds = [f"<td><b>{esc(lg.mlabel(m))}</b></td>"]
        for o in options:
            ln = cell.get((m, o))
            if not ln:
                tds.append('<td class="cell muted">—</td>')
                continue
            rows = lg.by_line.get(ln, [])
            r = rows[-1] if rows else {}
            e, ci = num(r.get("elo2")), num(r.get("ci2"))
            pe = num(rows[-2].get("elo2")) if len(rows) > 1 else None
            ncr = sum(1 for x in rows if x.get("decision") == "new-champion")
            top = ' <span class="p-new" data-tip="the best option for this model in the last cycle">best</span>' \
                if best.get(m) == o else ""
            tds.append(f'<td class="cell" data-it="{esc(ln)}" data-fixed><div><span class="fx-dot"></span> '
                       f'{esc(lg.disp(ln))}{top}</div>'
                       + (f'<div class="e">{plain(e)} <span class="muted">±{ci or 0:.0f}</span>'
                          + (f' <span class="chg" data-tip="since the previous cycle">{signed(e - pe)}</span>'
                             if pe is not None else "") + "</div>" if e is not None else '<div class="muted">not '
                                                                                         'rated yet</div>')
                       + f'<div class="s">{esc(r.get("version_name") or "")} · {ncr} new champion'
                       f'{"s" if ncr != 1 else ""}</div></td>')
        body.append("<tr>" + "".join(tds) + "</tr>")
    gaps = option_gaps(lg, models, options, cell)
    return (f'<div class="tscroll"><table class="grid2"><thead>{head}</thead><tbody>{"".join(body)}</tbody></table>'
            f'</div>{gaps}{sweep_summary(lg)}<p class="note">The current version of each line at ply 2 in '
            f'{f"cycle {lg.rated_cycle}" if lg.rated_cycle is not None else "the last cycle (none yet)"}, its 95% ±, '
            'the change since the previous cycle and its new champions so far. Click a cell to highlight the line '
            'everywhere.</p>')


def sweep_summary(lg, window=5):
    """Experiments with variants or replicates (FORMATS.md 16:45): per setting (the base model and its flag
    overrides) and option, the replicates' ply-2 ratings in the last cycle, their mean and spread, and the mean over
    the last `window` cycles."""
    groups = {}
    for ln, d in lg.line_defs.items():
        f = lg.facets.get(ln) or {}
        setting = " ".join([str(f.get("base") or "")] + [f"{fl}={f['ov:' + fl]}" for fl in lg.ov_flags
                                                         if f.get("ov:" + fl) not in (None, "recipe's")])
        groups.setdefault((setting, d.get("option") or ""), []).append(ln)
    if not lg.ov_flags and all(len(v) < 2 for v in groups.values()):
        return ""
    rows = []
    for (setting, opt), lns in sorted(groups.items(), key=lambda kv: (
            [fx_float(lg.facets[kv[1][0]].get("ov:" + fl)) or 0 for fl in lg.ov_flags], kv[0][0],
            FX_ORDER["option"].index(kv[0][1]) if kv[0][1] in FX_ORDER["option"] else 9)):
        last, win, each = [], [], []
        for ln in sorted(lns, key=lambda x: inum(lg.line_defs[x].get("replicate"), 1)):
            rs = [r for r in lg.by_line.get(ln, []) if num(r.get("elo2")) is not None]
            rep_ = lg.facets[ln]["replicate"]
            if not rs:
                each.append(f"{rep_} —")
                continue
            e = num(rs[-1].get("elo2"))
            if lg.rated_cycle is None or inum(rs[-1].get("cycle")) == lg.rated_cycle:
                last.append(e)
            w = [num(r.get("elo2")) for r in rs[-window:]]
            win.append(sum(w) / len(w))
            each.append(f"{rep_} {plain(e)}")
        m = sum(last) / len(last) if last else None
        mw = sum(win) / len(win) if win else None
        spread = max(last) - min(last) if len(last) > 1 else None
        rows.append("<tr>" + td(setting) + td(opt) + td(len(lns), len(lns), cls="r")
                    + td(plain(m) if m is not None else "—", m, cls="r")
                    + td(f"{spread:.0f}" if spread is not None else "—", spread, cls="r")
                    + td(plain(mw) if mw is not None else "—", mw, cls="r")
                    + td(", ".join(each)) + "</tr>")
    heads = [th("Setting", "s", "text"), th("Option", "o", "text"), th("Replicates", "n"),
             th("Last cycle", "m", tip="the replicates' mean ply-2 rating in the last cycle"),
             th("Spread", "sp", tip="the highest replicate's rating minus the lowest's, last cycle"),
             th(f"Last {window} cycles", "mw", tip="each replicate's mean ply-2 rating over its last cycles, averaged"),
             th("Each replicate", "e", "text", tip="each replicate's ply-2 rating in its last rated cycle")]
    return ('<h3>By setting</h3>' + table("t-sweep", heads, rows, box=False)
            + '<p class="note">Lines grouped by setting (the base model and its flag overrides) and option, their '
              'replicates pooled. A single version is noisy, so compare settings over several cycles and '
              'replicates.</p>')


def option_gaps(lg, models, options, cell, window=5):
    """Per model, each pair of options: their current versions' ply-2 gap in the last cycle (the two ±s combined as
    if independent), its mean over the last `window` cycles, and in how many cycles each was ahead."""
    rows = []
    for m in models:
        for i, a in enumerate(options):
            for b in options[i + 1:]:
                la, lb = cell.get((m, a)), cell.get((m, b))
                if not la or not lb:
                    continue
                ea = {inum(r.get("cycle")): (num(r.get("elo2")), num(r.get("ci2"))) for r in lg.by_line.get(la, [])}
                eb = {inum(r.get("cycle")): (num(r.get("elo2")), num(r.get("ci2"))) for r in lg.by_line.get(lb, [])}
                both = sorted(c for c in ea if c in eb and ea[c][0] is not None and eb[c][0] is not None)
                if not both:
                    continue
                g = {c: eb[c][0] - ea[c][0] for c in both}  # the later option minus the earlier one
                c = both[-1]
                se = math.sqrt(((ea[c][1] or 0) / 1.96) ** 2 + ((eb[c][1] or 0) / 1.96) ** 2)
                win = [g[x] for x in both[-window:]]
                mean = sum(win) / len(win)
                ahead_b = sum(1 for x in both if g[x] > 0)
                rows.append("<tr>" + td(lg.mlabel(m)) + td(f"{b} against {a}")
                            + td(f"{esc(b)} {esc(gap_words(g[c], a))} <span class=\"muted\">±{1.96 * se:.0f}</span>",
                                 g[c], raw=True, tip=f"cycle {c}: {lg.disp(lb)} {plain(eb[c][0])} against "
                                                     f"{lg.disp(la)} "
                                                     f"{plain(ea[c][0])}; ± is 95%, the two combined as if independent")
                            + td(f"{esc(b)} {esc(gap_words(mean, a))}", mean, raw=True,
                                 tip=f"the mean gap over cycles {both[-len(win)]}-{both[-1]}")
                            + td(f"{esc(b)} {ahead_b} of {len(both)}, {esc(a)} {len(both) - ahead_b}", ahead_b,
                                 raw=True, tip="cycles in which each option's current version rated higher at ply 2")
                            + "</tr>")
    if not rows:
        return ""
    heads = [th("Model", "m", "text"), th("Options", "o", "text"), th("Last cycle", "g", tip="the gap at ply 2"),
             th(f"Last {window} cycles", "mg", tip="the mean gap"), th("Cycles ahead", "a")]
    return ('<h3>Gaps between options</h3>' + table("t-gaps", heads, rows, box=False)
            + '<p class="note">Single cycles are noisy: one version can rate well above or below the next. The mean '
              'over several cycles is the steadier comparison.</p>')


def anchor_svg(lo, hi, margin, elo, ci):
    """The anchor's rating against its interval: the interval dark, the margins light, a dot with its ±."""
    W, H = 150, 14
    a, b = lo - 2 * margin, hi + 2 * margin
    X = lambda v: (min(max(v, a), b) - a) / (b - a) * W
    out = [f'<svg width="{W}" height="{H}" class="anc" aria-hidden="true">',
           f'<rect x="{X(lo - margin):.1f}" y="4" width="{X(hi + margin) - X(lo - margin):.1f}" height="6" class="m"/>',
           f'<rect x="{X(lo):.1f}" y="4" width="{X(hi) - X(lo):.1f}" height="6" class="i"/>']
    if elo is not None:
        if ci:
            out.append(f'<line x1="{X(elo - ci):.1f}" x2="{X(elo + ci):.1f}" y1="7" y2="7" class="w"/>')
        out.append(f'<circle cx="{X(elo):.1f}" cy="7" r="3.5" class="d"/>')
    return "".join(out) + "</svg>"


def panel_anchors(lg):
    cfg = lg.config.get("anchors") or {}
    step, margin, outc = num(cfg.get("step"), 100), num(cfg.get("margin"), 25), inum(cfg.get("out_cycles"), 5)
    lowest = inum(cfg.get("lowest"), 100)
    by_lo = {inum(r.get("interval_lo")): r for r in lg.anchors if inum(r.get("interval_lo")) is not None}
    top = max(by_lo) if by_lo else None
    if lg.rated_cycle is not None:  # empty intervals up to the best current ply-1 entry
        e1 = [num(r.get("elo")) for r in lg.hist[lg.rated_cycle] if r.get("ply") == "1"
              and (r.get("role") or "").startswith("current") and num(r.get("elo")) is not None]
        if e1 and max(e1) >= lowest:
            top = max(top or 0, int(max(e1) // step * step))
    if top is None:
        return ('<p class="note">No anchors yet: the first locks when a current ply-1 entry rates in an empty '
                f'{step:g}-Elo interval from {lowest} up (anchors.csv).</p>')
    rows = []
    lo = int(top)
    while lo >= lowest:
        r = by_lo.get(lo)
        hi = lo + int(step)
        if not r:
            rows.append(f'<tr class="mut">' + td(f"[{lo}, {hi})", lo) + td("no anchor yet", cls="muted")
                        + "".join(td("") for _ in range(8)) + "</tr>")
            lo -= int(step)
            continue
        ln = r.get("line") or ""
        e, ci, le = num(r.get("elo")), num(r.get("ci95")), num(r.get("locked_elo"))
        inside = e is not None and lo - margin <= e < hi + margin
        streak = inum(r.get("outside_streak"), 0)
        avail = str(r.get("available")) in ("1", "true", "True")
        rows.append(
            f'<tr data-it="{esc(ln)}">' + td(f"[{lo}, {hi})", lo)
            + td(r.get("version_name") or "", inum(r.get("version")), tip=r.get("entry") or "")
            + td(line_cell(lg.disp(ln)), lg.disp(ln), raw=True)
            + td(r.get("locked_cycle") or "", inum(r.get("locked_cycle")), cls="r")
            + td(plain(le), le, cls="r")
            + td(f"{plain(e)} ±{ci or 0:.0f}" if e is not None else "not rated yet", e, cls="r" if e is not None
                 else "muted", tip="" if e is not None else "it joins the field in the cycle after it locks")
            + td(moved_words(e - le) if e is not None and le is not None else "—",
                 e - le if e is not None and le is not None else None, tip="since it locked")
            + td(anchor_svg(lo, hi, margin, e, ci), e, raw=True,
                 tip=f"its interval [{lo}, {hi}) dark, the {margin:g}-Elo margins light")
            + td((f"{streak} of {outc}" if streak else ("inside" if inside else "outside") if e is not None
                  else "—"), streak, cls="r", tip=f"cycles in a row outside its interval by more than {margin:g}")
            + td(pill("available", "it gave its interval up; it stays until a new anchor takes it", "p-wait")
                 if avail else pill("locked", cls="p-frz"), int(avail), raw=True) + "</tr>")
        lo -= int(step)
    heads = [th("Interval", "iv"), th("Anchor", "a", tip="the frozen ply-1 copy, with its entry id in the tooltip"),
             th("Line", "line", "text"), th("Locked", "lc", cls="r", tip="the cycle it locked"),
             th("Locked at", "le", cls="r"), th("Elo now", "e", cls="r"), th("Moved", "mv"),
             th("In its interval", "pos"), th("Outside", "out", cls="r"), th("State", "av")]
    ev = [e for e in lg.events if (e.get("event") or "").startswith("anchor-")]
    return (count_note("t-anchors", f"Frozen ply-1 anchors, one per {step:g}-Elo interval on the ply-1 scale "
                       f"(anchors.csv). One gives its interval up after {outc} cycles in a row outside it by more than "
                       f"{margin:g}; {len(ev)} anchor events so far.")
            + table("t-anchors", heads, rows))


def panel_events(lg):
    evs = sorted(lg.events, key=lambda e: (-(inum(e.get("cycle"), 0)), -(ptime(e.get("date")) or 0)))
    if not evs:
        return '<p class="note">No events yet (events.csv).</p>'
    counts = {}
    for e in evs:
        counts[e.get("event")] = counts.get(e.get("event"), 0) + 1
    rows = []
    for e in evs[:250]:
        ln = e.get("line") or ""
        cls = {"new-champion": "ev-new", "retired": "ev-ret", "done": "ev-done"}.get(e.get("event"), "")
        attr = f' data-it="{esc(ln)}"' if ln else ""
        rows.append(f'<tr{attr}{f" class={cls}" if cls else ""}>' + td(e.get("cycle") or "", inum(e.get("cycle")),
                                                                      cls="r")
                    + td(tstamp(e.get("date")), ptime(e.get("date")), raw=True)
                    + td(line_cell(lg.disp(ln)) if ln else "", lg.disp(ln), raw=True)
                    + td(EVENT_WORDS.get(e.get("event"), e.get("event") or ""), e.get("event"))
                    + td(e.get("version_name") or "", tip=e.get("entry") or "") + td(event_value(e))
                    + td(e.get("detail") or "", cls="wrap") + "</tr>")
    heads = [th("Cycle", "c", cls="r"), th("When", "t"), th("Line", "line", "text"), th("Event", "ev", "text"),
             th("Version", "v", "text"), th("Value", "val", "text"), th("Detail")]
    summary = ", ".join(f"{n} {EVENT_WORDS.get(k, k)}" for k, n in counts.items())
    return (count_note("t-events", f"Newest first (events.csv): {esc(summary)}"
                       + (f"; the latest 250 of {len(evs)} shown" if len(evs) > 250 else "") + ".")
            + table("t-events", heads, rows))


def panel_cycles(lg):
    if not lg.totals:
        return '<p class="note">No cycle has finished yet (totals.csv).</p>'
    sp = any(num(r.get("selfplay_cpu_s"), 0) for r in lg.totals)
    fit = any(fit_cpu(r) is not None for r in lg.totals)
    pr = lg.priced
    names = ["tournament games"] + (["rating fit"] if fit else []) + ["training"] + (["self-play"] if sp else []) \
        + ["calibration"]
    cols = ["var(--k1)"] + (["var(--k7)"] if fit else []) + ["var(--k2)"] + (["var(--k3)"] if sp else []) \
        + ["var(--k4)"]
    keys = ["tournament_cpu_s"] + (["fit"] if fit else []) + ["train_cpu_s"] + (["selfplay_cpu_s"] if sp else []) \
        + ["cal_cpu_s"]
    tot = lg.totals[-200:]
    stacks = [[(fit_cpu(r) or 0) if k == "fit" else num(r.get(k), 0) for k in keys] for r in tot]
    tips = [f"cycle {r.get('cycle')}: " + ", ".join(f"{nm} {cpu_txt(v)}" for nm, v in zip(names, s))
            + f"; wall {fmt_dur(num(r.get('wall_s')))}" for r, s in zip(tot, stacks)]
    chart = bars_svg([r.get("cycle") for r in tot], stacks, names, cols, tips)
    rows = []
    for r in reversed(lg.totals[-120:]):
        cpu = sum(num(r.get(k), 0) for k in ("tournament_cpu_s", "train_cpu_s", "selfplay_cpu_s", "cal_cpu_s")) \
            + (fit_cpu(r) or 0)
        rn = inum(r.get("rounds"), 0)
        tc = num(r.get("tournament_cpu_s"), 0)
        fc = fit_cpu(r)
        cells = [td(r.get("cycle"), inum(r.get("cycle")), cls="r"), td(tstamp(r.get("start")), ptime(r.get("start")),
                                                                      raw=True),
                 td(fmt_dur(num(r.get("wall_s"))), num(r.get("wall_s")), cls="r"),
                 td(r.get("field") or "", inum(r.get("field")), cls="r"), td(rn, rn, cls="r"),
                 td(f"{inum(r.get('games'), 0):,}", inum(r.get("games")), cls="r",
                    tip=f"{inum(r.get('ply2_games'), 0):,} of them qualify (a model at ply 2)"),
                 td(cpu_txt(tc), tc, cls="r", tip=f"the games' own CPU; qualifying games: "
                                                  f"{cpu_txt(num(r.get('ply2_cpu_s')))}")]
        if pr:
            cells.append(td(cpu_txt(num(r.get("price_s"))), num(r.get("price_s")), cls="r",
                            tip=f"the games' price at prices {r.get('prices_version') or lg.prices or '?'}; "
                                f"qualifying games: {price_txt(num(r.get('ply2_price_s')))}"))
        cells.append(td(cpu_txt(tc / rn) if rn else "—", tc / rn if rn else None, cls="r"))
        if fit:
            cells.append(td(cpu_txt(fc), fc, cls="r",
                            tip=f"the tournament process's own CPU ({cpu_txt(num(r.get('tournament_proc_cpu_s')))}) "
                                f"beyond its games': the rating fit and tables" if fc is not None else
                                "not recorded (no tournament_proc_cpu_s)"))
        cells.append(td(cpu_txt(num(r.get("train_cpu_s"))), num(r.get("train_cpu_s")), cls="r"))
        if sp:
            cells.append(td(cpu_txt(num(r.get("selfplay_cpu_s"))), num(r.get("selfplay_cpu_s")), cls="r",
                            tip=f"measured; generation: {cpu_txt(num(r.get('selfplay_gen_cpu_s')))}"
                                + (f"; priced generation: {price_txt(num(r.get('selfplay_price_s')))}"
                                   if num(r.get("selfplay_price_s")) is not None else "")))
        cells += [td(cpu_txt(num(r.get("cal_cpu_s"))), num(r.get("cal_cpu_s")), cls="r",
                     tip=f"{r.get('cal_made') or 0} calibrations made"),
                  td(cpu_txt(cpu), cpu, cls="r", tip="games, rating fit and tables, training, self-play and "
                                                         "calibration"),
                  td(r.get("new_champions") or "0", inum(r.get("new_champions")), cls="r")]
        if lg.is_global:
            cells += [td(r.get("retired") or "0", inum(r.get("retired")), cls="r"),
                      td(f"{r.get('anchors') or 0}", inum(r.get("anchors")), cls="r",
                         tip=f"locked {r.get('anchors_locked') or 0}, replaced {r.get('anchors_replaced') or 0}")]
        else:
            if lg.sp_budget is None:
                dt = num(r.get("data_price_target_s" if pr else "data_cpu_target_s"))
                cells.append(td(cpu_txt(dt), dt, cls="r",
                                tip=f"the priced CPU each line should train on this cycle; measured "
                                    f"{cpu_txt(num(r.get('data_cpu_target_s')))} (information only)" if pr else
                                "the generation CPU each line should train on this cycle"))
            if lg.has_global:
                cells += [td(r.get("global_rounds") or "0", inum(r.get("global_rounds")), cls="r",
                             tip=(f"{price_txt(num(r.get('global_price_s')))} of games, "
                                  f"{price_txt(num(r.get('global_ply2_price_s')))} qualifying; measured "
                                  f"{cpu_txt(num(r.get('global_cpu_s')))}" if pr else
                                  f"{cpu_txt(num(r.get('global_cpu_s')))} of games, "
                                  f"{cpu_txt(num(r.get('global_ply2_cpu_s')))} qualifying")),
                          td(fmt_dur(num(r.get("wait_s"))) if num(r.get("wait_s")) else "—", num(r.get("wait_s")),
                             cls="r")]
        rows.append("<tr>" + "".join(cells) + "</tr>")
    heads = [th("Cycle", "c", cls="r"), th("Started", "s"), th("Wall", "w", cls="r"), th("Field", "f", cls="r"),
             th("Rounds", "r", cls="r"), th("Games", "g", cls="r"), th("Tournament", "tc", cls="r",
                                                                       tip="the games' own CPU, measured")]
    if pr:
        heads.append(th("Priced", "pc", cls="r", tip="the tournament games' price (prices.csv constants frozen in the "
                                                     "league's config): deterministic, the measure budgets use"))
    heads.append(th("Per round", "pr", cls="r", tip="measured CPU per round"))
    if fit:
        heads.append(th("Rating fit", "fit", cls="r", tip="the tournament process's own CPU beyond its games: the "
                                                         "rating fit and tables (tournament_proc_cpu_s minus "
                                                         "tournament_cpu_s)"))
    heads.append(th("Training", "trc", cls="r"))
    if sp:
        heads.append(th("Self-play", "spc", cls="r"))
    heads += [th("Calibration", "cc", cls="r"), th("CPU", "cpu", cls="r"), th("New champions", "nc", cls="r")]
    if lg.is_global:
        heads += [th("Retired", "ret", cls="r"), th("Anchors", "an", cls="r")]
    else:
        if lg.sp_budget is None:
            heads.append(th("Data target", "dt", cls="r", tip="priced: what each line trains on, the price of the "
                                                              "cycle's qualifying games" if pr else ""))
        if lg.has_global:
            heads += [th("Global rounds", "gr", cls="r"), th("Waited", "wt", cls="r")]
    return (f'<h3>Measured CPU per cycle</h3>{chart}'
            + count_note("t-cycles", "One row per cycle, newest first (totals.csv"
                         + (f"; the latest 120 of {len(lg.totals)}" if len(lg.totals) > 120 else "")
                         + "). Measured CPU, what was spent: tournament (the games' own CPU), rating fit (the "
                         "tournament process's CPU beyond its games), training, self-play, calibration, and CPU, "
                         "everything." + (f" Priced: the games' price at prices {lg.prices or '?'}, deterministic, the "
                                          "measure budgets use" + (", as is the data target" if lg.sp_budget is None
                                                                    and not lg.is_global else "") + "."
                                          if pr else ""))
            + table("t-cycles", heads, rows))


def panel_rounds(lg):
    if not lg.rounds:
        return '<p class="note">No round played yet (rounds.csv).</p>'
    pr = lg.priced
    rr = sorted(lg.rounds, key=lambda r: (inum(r.get("cycle"), 0), inum(r.get("round"), 0)))
    show = rr[-250:]
    k, kq = ("price_s", "ply2_price_s") if pr else ("cpu_s", "ply2_cpu_s")
    stacks = [[num(r.get(kq), 0), max(0.0, num(r.get(k), 0) - num(r.get(kq), 0))] for r in show]
    tips = [f"cycle {r.get('cycle')} round {r.get('round')} (t {r.get('t')}): "
            + (f"{price_txt(num(r.get('price_s')))}, {price_txt(num(r.get('ply2_price_s')))} qualifying; measured "
               f"{cpu_txt(num(r.get('cpu_s')))}" if pr else cpu_txt(num(r.get("cpu_s"))))
            + f"; {inum(r.get('games'), 0):,} games ({inum(r.get('ply2_games'), 0):,} qualifying), "
            f"{r.get('players')} players, wall {fmt_dur(num(r.get('wall_s')))}" for r in show]
    chart = bars_svg([f"{r.get('cycle')}" for r in show], stacks, ["qualifying games (a model at ply 2)", "the rest"],
                     ["var(--k1)", "var(--track)"], tips, h=130, xlab="round")
    cs = sorted({inum(r.get("cycle"), 0) for r in rr})[-3:]
    rows = []
    for r in reversed([r for r in rr if inum(r.get("cycle"), 0) in cs]):
        g, cpu = inum(r.get("games"), 0), num(r.get("cpu_s"), 0)
        cells = [td(r.get("cycle"), inum(r.get("cycle")), cls="r"), td(r.get("round"), inum(r.get("round")), cls="r"),
                 td(r.get("t"), inum(r.get("t")), cls="r"), td(r.get("players"), inum(r.get("players")), cls="r"),
                 td(f"{g:,}", g, cls="r"), td(f"{inum(r.get('ply2_games'), 0):,}", inum(r.get("ply2_games")), cls="r")]
        if pr:
            cells += [td(cpu_txt(num(r.get("price_s"))), num(r.get("price_s")), cls="r"),
                      td(cpu_txt(num(r.get("ply2_price_s"))), num(r.get("ply2_price_s")), cls="r")]
        cells += [td(cpu_txt(cpu), cpu, cls="r"), td(cpu_txt(num(r.get("ply2_cpu_s"))), num(r.get("ply2_cpu_s")), cls="r"),
                  td(f"{cpu / g:.3f} s" if g else "—", cpu / g if g else None, cls="r"),
                  td(fmt_dur(num(r.get("wall_s"))), num(r.get("wall_s")), cls="r")]
        rows.append("<tr>" + "".join(cells) + "</tr>")
    heads = [th("Cycle", "c", cls="r"), th("Round", "r", cls="r"),
             th("t", "t", cls="r", tip="the tournament index in tour/games.csv"),
             th("Players", "p", cls="r"), th("Games", "g", cls="r"), th("Qualifying", "q", cls="r")]
    if pr:
        heads += [th("Priced", "pc", cls="r", tip="the round's games priced (prices.csv constants frozen in the "
                                                  "league's config): the measure budgets use"),
                  th("Qualifying priced", "pq", cls="r", tip="the qualifying games' price: what the experiments' data "
                                                             "budgets are made of")]
    heads += [th("Measured CPU", "cpu", cls="r", tip="the games' own CPU (tour/game_cpu.csv), not the fit"),
              th("Qualifying measured", "qc", cls="r"), th("Per game", "pg", cls="r", tip="measured CPU per game"),
              th("Wall", "w", cls="r")]
    what = "Priced CPU" if pr else "CPU"
    return (f'<h3>{what} per round{f" (the latest {len(show)} of {len(rr)})" if len(rr) > len(show) else ""}</h3>'
            f'{chart}'
            + count_note("t-rounds", f"The rounds of cycles {', '.join(map(str, cs))} (rounds.csv)"
                         + (f". Priced: the games' price at prices {lg.prices or '?'}, the measure every budget uses; "
                            "measured: the games' own CPU (tour/game_cpu.csv), for information." if pr else
                            ": the games' own CPU (tour/game_cpu.csv), not the fit."))
            + table("t-rounds", heads, rows))


def panel_data_cpu(lg):
    """Experiments: each line's training data per cycle against its budget, in priced CPU (cycles.csv data_price_s
    against totals.csv data_price_target_s; a self-play line against its selfplay_budget_s, the fixed budget in a
    self-play-only experiment), the measured CPU in the tooltips. Leagues made before prices show measured CPU."""
    lines = list(lg.line_defs)
    cyc = sorted({inum(r.get("cycle"), 0) for r in lg.cycles})
    if not cyc:
        return '<p class="note">No cycle has finished yet.</p>'
    pr = lg.priced
    fmt = price_txt if pr else cpu_txt
    vkey, tkey = ("data_price_s", "data_price_target_s") if pr else ("data_cpu_s", "data_cpu_target_s")
    fixed = lg.sp_budget if pr else None  # a self-play-only experiment's budget per line per cycle

    def target(c):
        return fixed if fixed is not None else num(lg.tot.get(c, {}).get(tkey))

    series, tgt = [], []
    for ln in lines:
        pts = []
        for r in lg.by_line.get(ln, []):
            v = num(r.get(vkey))
            if v is not None:
                pts.append([inum(r.get("cycle")), round(v, 2), 0, r.get("version_name") or ""])
        if pts:
            series.append(dict(it=ln, name=lg.disp(ln), pts=pts))
    for c in cyc:
        t = target(c)
        if t is not None:
            tgt.append([c, round(t, 2), 0, "the fixed budget" if fixed is not None else "the cycle's target"])
    if tgt:
        series.append(dict(name="budget" if fixed is not None else "target", dash=1, pts=tgt))
    lg.charts["data"] = dict(plies=["cpu"], ply="cpu", xlab="cycle", ylab="priced CPU-s" if pr else "CPU-s",
                             fmt="price" if pr else "cpu", series={"cpu": series}, h=240,
                             dashBy=["replicate", "model", "option"])
    ratio = {}
    rows = []
    for c in reversed(cyc):
        t = target(c)
        tm = num(lg.tot.get(c, {}).get("data_cpu_target_s"))
        cells = [td(c, c, cls="r"),
                 td(fmt(t) if t is not None else "—", t, cls="r",
                    tip="config.json selfplay_budget_s: fixed" if fixed is not None else
                    f"the qualifying games' price; measured {cpu_txt(tm)} (information only)" if pr else "")]
        for ln in lines:
            r = lg.cyc_rows.get((c, ln))
            if not r:
                cells.append(td("", it=ln))
                continue
            v = num(r.get(vkey))
            vm = num(r.get("data_cpu_s"))
            opt = r.get("option") or lg.line_defs.get(ln, {}).get("option")
            name = lg.disp(ln)
            if v is None and lg.planned and c >= lg.planned and not num(r.get("selfplay_gen_cpu_s")) \
                    and not num(r.get("train_cpu_s")):
                cells.append(td("—", None, tip=f"{name}, cycle {c}: the experiment's last cycle trains nothing", it=ln))
                continue
            if opt == "self-play":
                b, cy = num(r.get("selfplay_budget_s")), num(r.get("selfplay_carry_s"))
                carry = (f"; carry after the cycle {'−' if cy < 0 else ''}{fmt(abs(cy))} "
                         + ("(overspent: the next budget is that much smaller)" if cy < 0 else
                            "(underspent: added to the next budget)") if cy else "")
                if pr:
                    tip = (f"{name}, cycle {c}: self-play generation {price_txt(num(r.get('selfplay_price_s')))} of a "
                           f"{price_txt(b)} budget ({'the fixed budget' if fixed is not None else 'the target'} plus "
                           f"the carry); {inum(r.get('selfplay_rounds'), 0)} rounds, "
                           f"{inum(r.get('selfplay_games'), 0):,} games, {inum(r.get('selfplay_plies'), 0):,} plies; "
                           f"trains on {price_txt(v)} of games{carry}. Measured, for information: generation "
                           f"{cpu_txt(num(r.get('selfplay_gen_cpu_s')))}, the job's total "
                           f"{cpu_txt(num(r.get('selfplay_cpu_s')))}")
                else:
                    tip = (f"{name}, cycle {c}: self-play generation {cpu_txt(num(r.get('selfplay_gen_cpu_s')))} of a "
                           f"{cpu_txt(b)} budget (the cycle's target plus the carry); the job's total "
                           f"{cpu_txt(num(r.get('selfplay_cpu_s')))}; {r.get('selfplay_rounds') or 0} rounds, "
                           f"{inum(r.get('selfplay_games'), 0):,} games; trains on {cpu_txt(v)} of games{carry}")
                ref = b if b is not None else t
            else:
                tip = (f"{name}, cycle {c}: trains on {fmt(v)} of games ({inum(r.get('train_games'), 0):,} games, "
                       f"{inum(r.get('train_positions'), 0):,} positions) against a {fmt(t)} target; training "
                       f"{cpu_txt(num(r.get('train_cpu_s')))}" + (f". Measured, for information: the games "
                                                                 f"{cpu_txt(vm)}" if pr else ""))
                ref = t
            if v is not None and ref:
                ratio.setdefault(opt, []).append(v / ref)
            pct = f"{100 * v / ref:.0f}%" if v is not None and ref else "—"
            cells.append(td(budget_meter(v, ref) + pct if v is not None else "—", v / ref if v is not None and ref
                            else None, raw=True, tip=tip, it=ln))
        rows.append("<tr>" + "".join(cells) + "</tr>")
    heads = [th("Cycle", "c", cls="r"),
             th("Budget" if fixed is not None else "Target", "t", cls="r",
                tip="config.json selfplay_budget_s: each line's fixed priced budget per cycle" if fixed is not None
                else "totals.csv data_price_target_s: the qualifying games' price of the cycle's tournament" if pr
                else "totals.csv data_cpu_target_s: the qualifying games' CPU of the cycle's tournament")]
    for ln in lines:
        heads.append(f'<th data-it="{esc(ln)}" data-sk="{esc(ln)}" data-st="num"><button type="button" class="sb" '
                     f'aria-pressed="false"><span class="fx-dot"></span> {esc(lg.disp(ln))}</button>'
                     f'<span class="sa"></span></th>')
    summ = "; ".join(f"{o}: {100 * sum(v) / len(v):.0f}% on average" for o, v in
                     sorted(ratio.items(), key=lambda kv: FX_ORDER["option"].index(kv[0])
                     if kv[0] in FX_ORDER["option"] else 9))
    if pr:
        what = (f"The priced CPU of the games each line trained on per cycle (cycles.csv data_price_s, prices "
                f"{esc(lg.prices or '?')}), against "
                + ("the fixed budget per line (dashed; a line's own budget adds its carry)" if fixed is not None else
                   "the cycle's target (dashed): the price of its qualifying games"))
    else:
        what = ("The generation CPU of the games each line trained on per cycle (cycles.csv data_cpu_s), against the "
                "cycle's target (dashed)")
    return ('<div class="lgc" data-chart="data"></div>' + legend(lg, {}, "data")
            + f'<p class="note">{what}. Relative to its budget: {esc(summ) or "—"}.'
            + (" Measured CPU is in the tooltips, for information only." if pr else "") + '</p>'
            + count_note("t-data", "Per cycle, newest first: each line's data against "
                         + ("its budget" if fixed is not None else "the target; a self-play line's against its budget "
                            "(the target plus its carry)") + ". Past the tick, over.")
            + table("t-data", heads, rows))


def panel_copied(lg):
    src = lg.config.get("global_source") or ""
    g = lg.root.glob
    pr = lg.priced
    link = (f' (<a href="/global/">the global league</a>)' if g and src and os.path.realpath(src) ==
            os.path.realpath(g.dir) else "")
    rows = []
    for r in sorted(lg.copied, key=lambda r: (-(inum(r.get("cycle"), 0)), -(inum(r.get("t"), 0)))):
        cells = [td(r.get("cycle"), inum(r.get("cycle")), cls="r"),
                 td(r.get("global_cycle"), inum(r.get("global_cycle")), cls="r"), td(r.get("t"), inum(r.get("t")), cls="r"),
                 td(f"{inum(r.get('games'), 0):,}", inum(r.get("games")), cls="r"),
                 td(f"{inum(r.get('ply2_games'), 0):,}", inum(r.get("ply2_games")), cls="r")]
        if pr:
            cells += [td(cpu_txt(num(r.get("price_s"))), num(r.get("price_s")), cls="r"),
                      td(cpu_txt(num(r.get("ply2_price_s"))), num(r.get("ply2_price_s")), cls="r")]
        cells += [td(cpu_txt(num(r.get("cpu_s"))), num(r.get("cpu_s")), cls="r"),
                  td(cpu_txt(num(r.get("ply2_cpu_s"))), num(r.get("ply2_cpu_s")), cls="r"),
                  td(tstamp(r.get("copied_at")), ptime(r.get("copied_at")), raw=True)]
        rows.append("<tr>" + "".join(cells) + "</tr>")
    heads = [th("Cycle", "c", cls="r"), th("Global cycle", "gc", cls="r"), th("t", "t", cls="r"),
             th("Games", "g", cls="r"), th("Qualifying", "q", cls="r")]
    if pr:
        heads += [th("Priced", "pc", cls="r", tip="the round's games priced by this experiment's prices"),
                  th("Qualifying priced", "pq", cls="r", tip="what the global lines' data budget counts")]
    heads += [th("Measured CPU" if pr else "CPU", "cpu", cls="r"),
              th("Qualifying measured" if pr else "Qualifying CPU", "qc", cls="r"), th("Copied", "at")]
    waits, open_w = [], None
    for e in sorted(lg.events, key=lambda e: ptime(e.get("date")) or 0):
        if e.get("event") == "wait-start":
            open_w = e
        elif e.get("event") == "wait-end":
            t0 = ptime(open_w.get("date")) if open_w else None
            t1 = ptime(e.get("date"))
            waits.append((open_w, e, t1 - t0 if t0 and t1 else None))
            open_w = None
    wtxt = ""
    if waits or open_w:
        tot = sum(w[2] or 0 for w in waits)
        wtxt = (f'<p class="note">Waits for unused global rounds: {len(waits)} ended, {fmt_dur(tot)} in all'
                + (f'; waiting since {tstamp(open_w.get("date"))} (cycle {esc(open_w.get("cycle") or "")})'
                   if open_w else "") + ".</p>")
    used = lg.state.get("used_global_rounds") or []
    gc = num(lg.state.get("global_carry_s"))
    ctxt = ""
    if pr and gc is not None:
        ctxt = (f'<p class="note">Global carry: the global lines\' qualifying data has '
                + (f"led the local lines' by {esc(price_txt(gc))}" if gc > 0 else
                   f"trailed the local lines' by {esc(price_txt(-gc))}" if gc < 0 else "matched the local lines'")
                + " over the cycles so far (state.json global_carry_s); each copy aims at the cycle's local data "
                  "minus it.</p>")
    per = []
    for t in lg.totals:
        if num(t.get("global_ply2_price_s" if pr else "global_ply2_cpu_s")) is not None:
            per.append(f"cycle {t.get('cycle')}: "
                       + (f"{price_txt(num(t.get('global_ply2_price_s')))} qualifying against "
                          f"{price_txt(num(t.get('data_price_target_s')))} local" if pr else
                          f"{cpu_txt(num(t.get('global_ply2_cpu_s')))} qualifying against "
                          f"{cpu_txt(num(t.get('data_cpu_target_s')))} local"))
    ptxt = (f'<p class="note">Copied against the local data, per cycle: {esc("; ".join(per[-6:]))}.</p>'
            if per else "")
    return (f'<p class="note">From {esc(src) or "—"}{link}. {len(used)} global rounds used so far; a round is never '
            f'used twice (state.json).</p>' + ctxt + ptxt + wtxt
            + (count_note("t-copied", "One row per global round copied, newest first (copied.csv)"
                          + ("; prices are this experiment's, measured CPU is for information." if pr else "."))
               + table("t-copied", heads, rows) if rows else '<p class="note">No rounds copied yet (copied.csv).</p>'))


def panel_batch(lg):
    b = lg.batch
    if not b:
        return '<p class="note">No batch yet: the first is made right after cycle 1\'s tournament (batch/batch.json).</p>'
    rounds = [r for r in b.get("rounds") or [] if isinstance(r, dict)]
    pr = any(num(r.get("price_s")) is not None for r in rounds)
    bc = inum(b.get("cycle"))
    users = {}  # t: [experiments that used (bc, t)]
    for x in lg.root.exps:
        src = x.config.get("global_source")
        if not src or os.path.realpath(src) != os.path.realpath(lg.dir):
            continue
        for pair in x.state.get("used_global_rounds") or []:
            if isinstance(pair, list) and len(pair) == 2 and inum(pair[0]) == bc:
                users.setdefault(inum(pair[1]), []).append(x)
    g = sum(inum(r.get("games"), 0) for r in rounds)
    q = sum(inum(r.get("ply2_games"), 0) for r in rounds)
    cpu = sum(num(r.get("cpu_s"), 0) for r in rounds)
    qc = sum(num(r.get("ply2_cpu_s"), 0) for r in rounds)
    pc = sum(num(r.get("price_s"), 0) for r in rounds)
    qp = sum(num(r.get("ply2_price_s"), 0) for r in rounds)
    tiles = (tile("Cycle", str(bc) if bc is not None else "—", f"made {tstamp(b.get('made'))}"
                  + (f" · prices {esc(b.get('prices_version'))}" if b.get("prices_version") else ""))
             + tile("Rounds", str(len(rounds)), f"{g:,} games, {q:,} qualifying")
             + (tile("Priced", esc(price_txt(pc)), f"{esc(price_txt(qp))} qualifying",
                     "the global league's prices; an experiment prices what it copies with its own") if pr else "")
             + tile("Measured CPU" if pr else "CPU", cpu_txt(cpu), f"{cpu_txt(qc)} qualifying"))
    rows = []
    for r in rounds:
        t = inum(r.get("t"))
        us = users.get(t, [])
        cells = [td(t, t, cls="r"), td(f"{inum(r.get('games'), 0):,}", inum(r.get("games")), cls="r"),
                 td(f"{inum(r.get('ply2_games'), 0):,}", inum(r.get("ply2_games")), cls="r")]
        if pr:
            cells += [td(cpu_txt(num(r.get("price_s"))), num(r.get("price_s")), cls="r"),
                      td(cpu_txt(num(r.get("ply2_price_s"))), num(r.get("ply2_price_s")), cls="r")]
        cells += [td(cpu_txt(num(r.get("cpu_s"))), num(r.get("cpu_s")), cls="r"),
                  td(cpu_txt(num(r.get("ply2_cpu_s"))), num(r.get("ply2_cpu_s")), cls="r"),
                  td(", ".join(f'<a href="{x.href}#copied">{esc(x.name)}</a>' for x in us) or
                     '<span class="muted">none yet</span>', len(us), raw=True)]
        rows.append("<tr>" + "".join(cells) + "</tr>")
    heads = [th("t", "t", cls="r"), th("Games", "g", cls="r"), th("Qualifying", "q", cls="r")]
    if pr:
        heads += [th("Priced", "pc", cls="r"), th("Qualifying priced", "pq", cls="r")]
    heads += [th("Measured CPU" if pr else "CPU", "cpu", cls="r"),
              th("Qualifying measured" if pr else "Qualifying CPU", "qc", cls="r"),
              th("Used by", "u", tip="experiments whose state.json lists this round as used")]
    lock = os.path.exists(lg.p("batch.lock"))
    return (f'<div class="tiles">{tiles}</div>'
            + count_note("t-batch", "The latest complete batch (batch/batch.json): the rounds experiments copy from. "
                         "It is replaced as a whole right after each cycle's tournament (the batch phase)"
                         + (", under an fcntl lock on batch.lock" if lock else "") + ".")
            + table("t-batch", heads, rows, box=False))


def panel_reports(lg):
    reps = sorted((f for f in listdir(lg.dir) if re.fullmatch(r"cycle-\d+\.md", f)), reverse=True)
    out = []
    if reps:
        latest = reps[0]
        text = tail(lg.p(latest), 60000)
        out.append(f'<details id="rep-{esc(lg.kind)}" open><summary>{esc(latest)}, the latest report</summary>'
                   f'<pre class="log">{esc(text)}</pre></details>')
        links = " ".join(f'<a href="report/{esc(f[6:-3])}">{esc(f[6:-3])}</a>' for f in reps[:200])
        out.append(f'<p class="note">Every cycle\'s report, as text: {links}'
                   + (f" … ({len(reps)} in all)" if len(reps) > 200 else "") + "</p>")
    else:
        out.append('<p class="note">No cycle report yet (cycle-NNN.md).</p>')
    out.append('<p class="note"><a href="log">driver.log</a> (the last 256 KB, as text) · '
               + ('<a href="repeat-log">repeat.log</a> (the loop script) · ' if os.path.exists(lg.p("repeat.log"))
                  else "")
               + " · ".join(f'<a href="file/{f}">{f}</a>' for f in ("config.json", "state.json") if
                            os.path.exists(lg.p(f))) + "</p>")
    return "".join(out)


# ---------------------------------------------------------------- routes and the server

def not_found(root, what):
    return page(root, "Not found · leagues", None, f'<h1>Not found</h1><p class="note">{esc(what)}</p>'
                f'<p><a href="/">All leagues</a></p>', dict(fxkey="none", fx={}, charts={}))


def route(path, query=""):
    """(body, content type, status) for a path, or None for an unknown one."""
    root = Root(CFG["root"])
    if path in ("/", "/index.html"):
        return home_page(root), "text/html; charset=utf-8", 200
    m = re.fullmatch(r"/(?:global|x/([^/]+))/(.*)", path)
    if not m:
        return None
    lg = root.experiment(m.group(1)) if m.group(1) is not None else root.glob
    if lg is None:
        what = f"No experiment named {m.group(1)!r} in {root.path}." if m.group(1) else f"No global league in {root.path}."
        return not_found(root, what), "text/html; charset=utf-8", 404
    rest = m.group(2)
    if rest in ("", "index.html"):
        return league_page(root, lg), "text/html; charset=utf-8", 200
    if rest == "repeat-log" and not os.path.exists(lg.p("repeat.log")):
        return "no repeat.log (the loop script was not started for this league)\n", "text/plain; charset=utf-8", 404
    if rest in ("log", "repeat-log"):
        f = "driver.log" if rest == "log" else "repeat.log"
        return tail(lg.p(f), 256 * 1024) or f"(no {f} yet)\n", "text/plain; charset=utf-8", 200
    r = re.fullmatch(r"report/(\d+)", rest)
    if r:
        f = lg.p(f"cycle-{int(r.group(1)):03d}.md")
        if not os.path.exists(f):
            f = lg.p(f"cycle-{r.group(1)}.md")
        if os.path.exists(f):
            return tail(f, 4 * 1024 * 1024), "text/plain; charset=utf-8", 200
        return f"no report {os.path.basename(f)}\n", "text/plain; charset=utf-8", 404
    if rest in ("file/config.json", "file/state.json"):
        f = lg.p(rest[5:])
        return (tail(f, 4 * 1024 * 1024) or "{}"), "text/plain; charset=utf-8", 200
    return None


class Handler(BaseHTTPRequestHandler):
    def send(self, body, status=200, ctype="text/html; charset=utf-8"):
        data = body.encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", ctype)
        self.send_header("Cache-Control", "no-store")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def redirect(self, where):
        self.send_response(301)
        self.send_header("Location", where)
        self.send_header("Content-Length", "0")
        self.end_headers()

    def do_GET(self):
        path, _, query = self.path.partition("?")
        path = urllib.parse.unquote(path)
        try:
            if path == "/favicon.ico":
                return self.send("", 204)
            if re.fullmatch(r"/(global|x/[^/]+)", path):
                return self.redirect(urllib.parse.quote(path) + "/")
            got = route(path, query)
            if got is None:
                return self.send(not_found(Root(CFG["root"]), f"Nothing at {path}."), 404)
            body, ctype, status = got
            self.send(body, status, ctype)
        except (BrokenPipeError, ConnectionResetError):
            pass
        except Exception:
            traceback.print_exc()
            try:
                self.send(f"<!doctype html><meta charset=utf-8><pre>dashboard: {esc(traceback.format_exc())}</pre>",
                          500)
            except OSError:
                pass

    def log_message(self, fmt, *args):
        pass  # no line per request


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--root", default=HERE, help="the leagues' root: index.json, global/, experiments/ (default: "
                                                 "this file's directory)")
    ap.add_argument("--port", type=int, default=8766)
    ap.add_argument("--old-dashboard", default=CFG["old"], metavar="URL",
                    help="the champion run's dashboard, linked for the champion run and its /first/ page (default: "
                         "%(default)s; \"\" for no links)")
    ap.add_argument("--once", metavar="PATH", help="render the page at PATH (e.g. / or /global/) to stdout and exit")
    args = ap.parse_args()
    old = args.old_dashboard.strip()
    CFG.update(root=os.path.abspath(args.root), old=(old if old.endswith("/") else old + "/") if old else "")
    if os.nice(0) < 19:
        os.nice(19 - os.nice(0))
    if args.once:
        path, _, query = args.once.partition("?")
        got = route(path, query)
        body = got[0] if got else f"nothing at {path}\n"
        sys.stdout.write(body)
        return
    server = ThreadingHTTPServer(("127.0.0.1", args.port), Handler)
    server.daemon_threads = True
    print(f"{datetime.datetime.now():%Y-%m-%d %H:%M:%S} leagues dashboard on http://127.0.0.1:{args.port} "
          f"(pid {os.getpid()}, nice {os.nice(0)}, root {CFG['root']}, "
          f"links to the champion run's dashboard: {CFG['old'] or 'none'})", flush=True)
    server.serve_forever()


CSS2 = r"""
.p-wait{color:var(--ink);background:color-mix(in srgb,var(--warning) 32%,transparent)}
.p-done{color:var(--frz-ink);background:var(--frz)}
.sdot{display:inline-block;width:8px;height:8px;border-radius:50%;margin-right:6px;vertical-align:1px;
 background:var(--muted)}
.sdot.s-running{background:var(--good)}.sdot.s-waiting{background:var(--warning)}.sdot.s-done{background:var(--fill)}
.sdot.s-failed{background:var(--critical)}
h1 .pill{font-size:12px;vertical-align:3px;margin-left:6px}
.toc{font-size:12.5px;color:var(--ink2);margin:4px 0 0}.toc a{color:var(--ink2)}
.tile .of{font-size:14px;font-weight:400;color:var(--ink2)}.tile .ph{font-size:13px;font-weight:400}
.tile .meter{width:90px}
ol.phases{display:flex;flex-wrap:wrap;gap:4px;list-style:none;padding:0;margin:12px 0 4px;font-size:12px}
ol.phases li{border:1px solid var(--grid);border-radius:10px;padding:0 8px;color:var(--muted)}
ol.phases li.on{border-color:var(--ink);color:var(--ink);font-weight:600;background:var(--gk)}
ol.phases li.done{color:var(--ink2)}ol.phases li.done::before{content:"✓ ";color:var(--good)}
ol.phases li.skip{color:var(--muted);text-decoration:line-through;border-style:dashed}
.tb{display:flex;gap:12px;align-items:baseline;margin:2px 0 4px}.tb .note{margin:0}
.tbox{max-height:520px;overflow:auto;border-top:1px solid var(--grid)}
.tbox thead th{position:sticky;top:0;background:var(--surface);z-index:1}
td.code{font-family:ui-monospace,monospace;font-size:12px;color:var(--ink2)}
pre.log{font-size:12px;background:var(--plane);border:1px solid var(--grid);border-radius:6px;padding:8px;
 max-height:420px;overflow:auto;white-space:pre-wrap}
svg.anc rect.m{fill:var(--track);opacity:.5}svg.anc rect.i{fill:var(--track)}
svg.anc line.w{stroke:var(--ink2);stroke-width:1.5}svg.anc circle.d{fill:var(--fill);stroke:var(--surface);stroke-width:1}
svg.bars text{fill:var(--muted);font:11px system-ui,-apple-system,"Segoe UI",sans-serif}
svg.bars line.grid{stroke:var(--grid)}svg.bars rect.hit{fill:transparent}
svg.bars g:hover rect:not(.hit){opacity:.75}
.keys{display:flex;flex-wrap:wrap;gap:4px 14px;font-size:12px;color:var(--ink2);margin:0 0 4px}
.keys .sw{display:inline-block;width:10px;height:10px;border-radius:2px;margin-right:5px;vertical-align:-1px}
table.grid2 td.cell[data-it]{cursor:pointer}.fx-out{opacity:.28}table.grid2 .e{font-size:15px}table.grid2 .chg{font-size:12px;
 color:var(--ink2)}
tr.ev-new td:nth-child(4){color:var(--good-text);font-weight:600}tr.ev-ret td:nth-child(4){color:var(--ink2)}
th[data-it] .sb{display:inline-flex;align-items:center;gap:4px}
"""


# ---------------------------------------------------------------- the page's CSS and JS

CSS = r"""
.viz-root{color-scheme:light;--surface:#fcfcfb;--plane:#f9f9f7;--ink:#0b0b0b;--ink2:#52514e;--muted:#898781;
 --grid:#e1e0d9;--axis:#c3c2b7;--c-ref:#898781;--good:#0ca30c;--good-text:#006300;--warning:#fab219;
 --serious:#ec835a;--critical:#d03b3b;--ring:rgba(11,11,11,.10);--track:#cde2fb;--fill:#2a78d6;
 --k1:#2a78d6;--k2:#eb6834;--k3:#1baf7a;--k4:#eda100;--k5:#e87ba4;--k6:#008300;--k7:#6250d6;--k8:#e34948;
 --frz:#d6e6fa;--frz-ink:#104281;--rtd:#ecebe6;--rtd-ink:#52514e;--gk:rgba(12,163,12,.13);--gr:rgba(208,59,59,.15)}
@media (prefers-color-scheme:dark){:root:where(:not([data-theme="light"])) .viz-root{color-scheme:dark;
 --surface:#1a1a19;--plane:#0d0d0d;--ink:#f0efec;--ink2:#c3c2b7;--grid:#2c2c2a;--axis:#383835;--good-text:#0ca30c;
 --ring:rgba(255,255,255,.10);--track:#184f95;--fill:#3987e5;--k1:#3987e5;--k2:#d95926;--k3:#199e70;--k4:#c98500;
 --k5:#d55181;--k6:#008300;--k7:#9085e9;--k8:#e66767;--frz:#184f95;--frz-ink:#e6effb;--rtd:#2c2c2a;--rtd-ink:#c3c2b7;
 --gk:rgba(12,163,12,.18);--gr:rgba(208,59,59,.23)}}
body{margin:0;background:var(--plane);font:14px/1.45 system-ui,-apple-system,"Segoe UI",sans-serif}
.viz-root{color:var(--ink);background:var(--plane);padding:0 24px 40px;min-height:100vh;box-sizing:border-box}
a{color:inherit}
h1{font-size:20px;margin:10px 0 2px}h2{font-size:16px;margin:0 0 10px}h3{font-size:14px;margin:14px 0 6px}
.sub,.note,.muted,.ts,.tl{color:var(--ink2)}.note{font-size:12.5px;margin:6px 0}.muted{color:var(--muted)}
section{background:var(--surface);border:1px solid var(--ring);border-radius:8px;padding:14px 16px;margin:14px 0}
h2 a.anchor{color:inherit;text-decoration:none}h2 a.anchor:hover{text-decoration:underline}
#app>section[id]{scroll-margin-top:calc(var(--top-h,48px) + 8px)}
.tiles{display:flex;flex-wrap:wrap;gap:10px}.tile{border:1px solid var(--ring);border-radius:8px;padding:8px 12px;
 min-width:150px}.tl{font-size:12px}.tv{font-size:22px;font-weight:600}.ts{font-size:12px}
table{border-collapse:collapse;font-variant-numeric:tabular-nums;font-size:13px}
th{font-weight:600;color:var(--ink2);text-align:left;padding:3px 8px;border-bottom:1px solid var(--axis);vertical-align:bottom}
td{padding:2px 8px;border-bottom:1px solid var(--grid);white-space:nowrap}td.r,th.r{text-align:right}
td.wrap{white-space:normal;min-width:220px;color:var(--ink2)}
tr.ref td,tr.mut td{color:var(--muted)}
.tscroll{overflow-x:auto}.hlp{cursor:help}
.ok{color:var(--good)}.bad{color:var(--critical)}.warn{color:var(--serious)}
.pill{font-size:11px;font-weight:600;border-radius:8px;padding:0 6px;white-space:nowrap}
.p-run{color:var(--good-text);background:var(--gk)}.p-frz{color:var(--frz-ink);background:var(--frz)}
.p-rtd{color:var(--rtd-ink);background:var(--rtd)}.p-bad{color:var(--critical);background:var(--gr)}
.p-new{font-size:11px;font-weight:600;color:var(--good-text);background:var(--gk);border-radius:8px;padding:0 6px}
.meter{display:inline-block;position:relative;width:120px;height:8px;border-radius:4px;background:var(--track);
 vertical-align:middle;overflow:hidden}.meter .fill{position:absolute;left:0;top:0;bottom:0;background:var(--fill);
 border-radius:4px}.meter .over{position:absolute;top:0;bottom:0;background:var(--serious);border-radius:0 4px 4px 0}
.meter .tick{position:absolute;top:-2px;bottom:-2px;width:2px;margin-left:-1px;background:var(--surface)}
td .meter{width:70px;height:6px;margin-right:6px}
details summary{cursor:pointer;color:var(--ink2);font-size:13px;margin:8px 0}
ul.notes{margin:6px 0;padding-left:18px;color:var(--ink2);font-size:12.5px}
#tip{position:fixed;z-index:20;pointer-events:none;background:var(--surface);color:var(--ink);border:1px solid var(--ring);
 border-radius:6px;padding:6px 8px;font-size:12.5px;white-space:pre-line;max-width:480px;
 box-shadow:0 2px 8px rgba(0,0,0,.15)}
#stale{color:var(--critical)}
.sr{position:absolute;width:1px;height:1px;overflow:hidden;clip-path:inset(50%);white-space:nowrap}
/* the sticky top: the leagues' navigation and the filters */
.top{position:sticky;top:0;z-index:10;margin:0 -24px;padding:6px 24px 6px;background:var(--plane);
 border-bottom:1px solid var(--ring);font-size:12.5px;color:var(--ink2)}
nav.lg{display:flex;flex-wrap:wrap;gap:6px;align-items:center}
nav.lg a,nav.lg select{font:inherit;font-size:12.5px;padding:2px 10px;border:1px solid var(--axis);border-radius:6px;
 color:var(--ink);text-decoration:none;background:var(--surface);min-height:24px;box-sizing:border-box}
nav.lg a[aria-current]{background:var(--ink);color:var(--surface);border-color:var(--ink)}
nav.lg a:hover{text-decoration:underline}nav.lg .sep{color:var(--muted)}
nav.lg select{padding:2px 6px;cursor:pointer}nav.lg select.on{background:var(--ink);color:var(--surface);border-color:var(--ink)}
.fxb{display:flex;flex-wrap:wrap;gap:4px 12px;align-items:center;margin-top:5px}
.fxb:empty{display:none}
.fxb-tog,.cb button{display:inline-flex;align-items:center;gap:5px;font:inherit;font-size:12.5px;font-weight:600;
 border:1px solid var(--axis);background:var(--surface);color:var(--ink);border-radius:6px;padding:2px 10px;
 min-height:24px;cursor:pointer}
.fxb-tog.on{border-color:var(--ink)}
.fxb select{font:inherit;font-size:12.5px;border:1px solid var(--axis);background:var(--surface);color:var(--ink);
 border-radius:6px;padding:2px 6px;min-height:24px;cursor:pointer}
.fxb .fx-n{color:var(--ink);font-weight:600}
.fx-pill{display:inline-flex;align-items:center;gap:2px;border:1px solid var(--axis);border-radius:12px;
 padding:0 2px 0 8px;background:var(--surface);color:var(--ink)}
.fx-pill button,.fx-x{font:inherit;font-size:12px;border:0;background:none;color:var(--ink2);cursor:pointer;
 padding:1px 6px;border-radius:10px}
.fx-pill button:hover,.fx-x:hover{background:var(--grid);color:var(--ink)}
.fx-chips{display:grid;grid-template-columns:repeat(auto-fill,minmax(460px,1fr));gap:0 28px;margin:6px 0 2px;
 max-height:40vh;overflow:auto}
.fx-chips[hidden]{display:none}
.fx-row{display:flex;flex-wrap:wrap;gap:4px 6px;align-items:center;padding:2px 0;border-bottom:1px solid var(--grid)}
.fx-k{width:150px;flex:none;font-size:12px;font-weight:600;color:var(--ink2)}
.fx-chip{display:inline-flex;align-items:center;gap:5px;font:inherit;font-size:12px;border:1px solid var(--axis);
 border-radius:14px;padding:0 9px;min-height:22px;background:var(--surface);color:var(--ink);cursor:pointer}
.fx-chip:hover{border-color:var(--ink2)}
.fx-chip[aria-pressed="true"]{background:var(--ink);color:var(--surface);border-color:var(--ink)}
.fx-chip .n{color:var(--muted);font-size:11.5px}.fx-chip[aria-pressed="true"] .n{color:var(--surface);opacity:.75}
.fx-chip.zero{opacity:.45}
.fx-dot{display:inline-block;width:10px;height:10px;border-radius:50%;flex:none;vertical-align:middle;
 background:var(--lc,var(--muted))}
[data-it]{--lc:var(--muted)}
tr[data-it]{cursor:pointer}
tr.fx-hi>td{background:color-mix(in srgb,var(--lc,var(--muted)) 14%,var(--surface))}
tr.fx-hi>td:first-child{box-shadow:inset 3px 0 0 var(--lc,var(--ink2))}
.cnt{margin-left:auto;color:var(--ink2)}.cnt.on{color:var(--ink);font-weight:600}
th .sb{font:inherit;font-weight:600;color:var(--ink2);background:none;border:0;padding:0;cursor:pointer;
 text-decoration:underline dotted;text-underline-offset:3px;text-align:left}
th .sb[aria-pressed="true"]{color:var(--ink)}
/* charts */
.cb{display:flex;flex-wrap:wrap;gap:6px;align-items:center;margin:0 0 8px;font-size:12.5px;color:var(--ink2)}
.cb button{font-weight:400}
.cb button[aria-pressed="true"]{background:var(--ink);color:var(--surface);border-color:var(--ink)}
.lgc{min-height:40px}.lgc svg{display:block;overflow:visible}
.lgc text{fill:var(--muted);font:11px system-ui,-apple-system,"Segoe UI",sans-serif}
.lgc .grid{stroke:var(--grid);stroke-width:1}.lgc .axisline{stroke:var(--axis);stroke-width:1}
.lgc .refl{stroke:var(--c-ref);stroke-width:1;stroke-dasharray:4 3;opacity:.7}
.lgc .ln{fill:none;stroke-width:1.4;stroke-opacity:.85;stroke-linejoin:round;stroke-linecap:round}
.lgc .ln.champ{stroke-width:1.1;stroke-opacity:.65;stroke-dasharray:2 3}
.lgc g.s.on .ln{stroke-width:3.2;stroke-opacity:1}.lgc g.s.hov .ln{stroke-width:2.6;stroke-opacity:1}
.lgc svg.hl g.s:not(.on):not(.hov){opacity:.13}.lgc svg.hv g.s:not(.hov):not(.on){opacity:.3}
.lgc .rg{fill:var(--surface);stroke-width:1.6}
.lgc .pt{stroke:var(--surface);stroke-width:1.5}.lgc .wh{stroke-width:1.5;opacity:.55}
.lgc .lab{fill:var(--ink);font-size:12px;paint-order:stroke;stroke:var(--surface);stroke-width:3px}
.lgc .ov{fill:transparent;cursor:crosshair}
.lgl{display:grid;grid-template-columns:repeat(auto-fill,minmax(240px,1fr));gap:1px 12px;margin:6px 0 0}
.lgl button{display:flex;align-items:center;gap:6px;font:inherit;font-size:12.5px;text-align:left;
 border:1px solid transparent;border-radius:4px;background:none;color:var(--ink);padding:1px 6px;min-height:22px;
 cursor:pointer;min-width:0}
.lgl button:hover{background:var(--plane)}.lgl button.fx-hi{border-color:var(--ink2);background:var(--plane);font-weight:600}
.lgl .sw line{stroke-width:2.2}.lgl .sw{flex:none}
.lgl .nm{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.lgl .e{margin-left:auto;color:var(--muted);font-weight:400}
.lgl button[hidden]{display:none}
.bars rect.b{fill:var(--fill)}.bars rect.b2{fill:var(--k2)}.bars rect.b3{fill:var(--k3)}
/* experiments: the model x option grid */
table.grid2 td.cell{min-width:120px;vertical-align:top}
table.grid2 td.cell .e{font-weight:600}table.grid2 td.cell .s{font-size:11.5px;color:var(--ink2)}
.prog{display:inline-flex;align-items:center;gap:6px}
"""

JS = r"""
let LG = {};
function lgData() { const s = document.getElementById('lg-data'); try { LG = s ? JSON.parse(s.textContent) : {}; } catch (e) { LG = {}; } }
lgData();
const tip = document.getElementById('tip');
function tipAt(x, y) { tip.style.left = Math.min(x + 14, innerWidth - tip.offsetWidth - 8) + 'px'; tip.style.top = Math.min(y + 14, innerHeight - tip.offsetHeight - 8) + 'px'; }
function showTip(el, x, y) {
  const t = el && el.closest ? el.closest('[data-tip]') : null;
  if (!t) { if (!tip.dataset.chart) tip.hidden = true; return; }
  tip.textContent = t.getAttribute('data-tip'); tip.dataset.chart = ''; tip.hidden = false;
  if (x !== undefined) tipAt(x, y);
}
document.addEventListener('pointerover', e => showTip(e.target, e.clientX, e.clientY));
document.addEventListener('pointermove', e => { if (!tip.hidden) tipAt(e.clientX, e.clientY); });
document.addEventListener('focusin', e => { const r = e.target.getBoundingClientRect ? e.target.getBoundingClientRect() : null; if (r) showTip(e.target, r.left, r.bottom); });
document.addEventListener('pointerleave', () => { tip.hidden = true; tip.dataset.chart = ''; });

// ---------------------------------------------------------------- filters: one state per kind of page (LG.fxkey)
const FXS = {};
function fxKey() { return 'leagues.fx.' + (LG.fxkey || 'page'); }
function fxLoad() {
  for (const k of Object.keys(FXS)) delete FXS[k];
  Object.assign(FXS, {sel: {}, by: null, hi: null, open: false});
  try { Object.assign(FXS, JSON.parse(localStorage.getItem(fxKey()) || '{}')); } catch (e) {}
  if (!FXS.sel || typeof FXS.sel !== 'object') FXS.sel = {};
}
function fxSave() { try { localStorage.setItem(fxKey(), JSON.stringify(FXS)); } catch (e) {} }
function fxFactors() { return (LG.fx && LG.fx.factors) || []; }  // [key, label, values, labels, colours]
function fxItems() { return (LG.fx && LG.fx.items) || {}; }      // {item: {factor: value}, name}
function fxSel(k) {  // the ticked values of a factor that exist on this page
  const f = fxFactors().find(x => x[0] === k), s = FXS.sel[k];
  return f && Array.isArray(s) ? s.filter(v => f[2].includes(v)) : [];
}
function fxHas(val, v) { return Array.isArray(val) ? val.includes(v) : val === v; }  // a value or a list of values
function fxPass(it, skip) {
  const f = fxItems()[it];
  if (!f) return true;
  for (const [k] of fxFactors()) {
    if (k === skip) continue;
    const s = fxSel(k);
    if (s.length && !s.some(v => fxHas(f.v[k], v))) return false;
  }
  return true;
}
function fxBy() { const ks = fxFactors().map(f => f[0]); return ks.includes(FXS.by) ? FXS.by : (LG.fx && ks.includes(LG.fx.by) ? LG.fx.by : ks[0]); }
function fxColour(it) {
  const f = fxItems()[it], fac = fxFactors().find(x => x[0] === fxBy());
  if (!f || !fac) return 'var(--muted)';
  const val = f.v[fac[0]], i = fac[2].indexOf(Array.isArray(val) ? val[0] : val);
  return i < 0 ? 'var(--muted)' : fac[4][i];
}
function fxEsc(s) { return String(s).replace(/[&<>"]/g, c => ({'&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;'})[c]); }
function fxBar() {
  const bar = document.getElementById('fxb');
  if (!bar) return;
  const facs = fxFactors(), items = Object.keys(fxItems());
  if (!facs.length || !items.length) { bar.innerHTML = ''; return; }
  const act = facs.filter(f => fxSel(f[0]).length), by = fxBy();
  let h = '<button type="button" class="fxb-tog' + (act.length ? ' on' : '') + '" data-fxtog aria-expanded="' + !!FXS.open + '">'
        + (FXS.open ? '▾' : '▸') + ' Filters' + (act.length ? ' <span class="fx-n">' + act.length + '</span>' : '') + '</button>';
  act.forEach(f => fxSel(f[0]).forEach(v => {
    const lab = f[3][f[2].indexOf(v)];
    h += '<span class="fx-pill">' + fxEsc(f[1]) + ': ' + fxEsc(lab) + '<button type="button" data-fxrm="' + fxEsc(f[0]) + '" data-v="' + fxEsc(v) + '" aria-label="remove">×</button></span>';
  }));
  if (act.length) h += '<button type="button" class="fx-x" data-fxclear>clear all</button>';
  h += '<label>Colour by <select data-fxby>' + facs.map(f => '<option value="' + fxEsc(f[0]) + '"' + (f[0] === by ? ' selected' : '') + '>' + fxEsc(f[1]) + '</option>').join('') + '</select></label>';
  const hi = FXS.hi && fxItems()[FXS.hi] ? FXS.hi : null;
  if (hi) h += '<span class="fx-pill"><span class="fx-dot" style="--lc:' + fxColour(hi) + '"></span> ' + fxEsc(fxItems()[hi].name || hi) + '<button type="button" data-fxunhi aria-label="un-highlight">×</button></span>';
  const shown = items.filter(it => fxPass(it)).length;
  h += '<span class="cnt' + (shown < items.length ? ' on' : '') + '">' + (shown < items.length ? 'showing ' + shown + ' of ' + items.length : 'all ' + items.length) + ' ' + fxEsc((LG.fx && LG.fx.noun) || 'lines') + '</span>';
  h += '<div class="fx-chips"' + (FXS.open ? '' : ' hidden') + '>';
  facs.forEach(f => {
    h += '<div class="fx-row"><span class="fx-k">' + fxEsc(f[1]) + '</span>';
    f[2].forEach((v, i) => {
      const n = items.filter(it => fxHas(fxItems()[it].v[f[0]], v) && fxPass(it, f[0])).length, on = fxSel(f[0]).includes(v);
      h += '<button type="button" class="fx-chip' + (n ? '' : ' zero') + '" data-fxk="' + fxEsc(f[0]) + '" data-v="' + fxEsc(v) + '" aria-pressed="' + on + '">'
         + (f[0] === by ? '<span class="fx-dot" style="--lc:' + f[4][i] + '"></span>' : '') + fxEsc(f[3][i]) + ' <span class="n">' + n + '</span></button>';
    });
    h += '</div>';
  });
  bar.innerHTML = h + '</div>';
}
function fxApply() {
  document.querySelectorAll('[data-it]').forEach(el => {
    const it = el.dataset.it;
    if (el.hasAttribute('data-fixed')) el.classList.toggle('fx-out', !fxPass(it)); else el.hidden = !fxPass(it);
    el.style.setProperty('--lc', fxColour(it));
    el.classList.toggle('fx-hi', FXS.hi === it);
  });
  document.querySelectorAll('[data-cnt]').forEach(c => {  // a table's row count: data-cnt is its id
    const t = document.getElementById(c.dataset.cnt);
    if (!t) return;
    const rows = [...t.querySelectorAll('tbody tr')], shown = rows.filter(r => !r.hidden).length;
    c.textContent = shown < rows.length ? 'showing ' + shown + ' of ' + rows.length + ' rows' : rows.length + ' rows';
    c.classList.toggle('on', shown < rows.length);
  });
  fxBar();
  chartsDraw();
  const top = document.querySelector('.top');
  if (top) document.documentElement.style.setProperty('--top-h', top.offsetHeight + 'px');
}
function fxHi(it) { FXS.hi = FXS.hi === it ? null : it; fxSave(); fxApply(); }
document.addEventListener('click', e => {
  const t = e.target.closest ? e.target : e.target.parentElement;
  if (t.closest('[data-fxtog]')) { FXS.open = !FXS.open; fxSave(); fxBar(); return; }
  const c = t.closest('[data-fxk]');
  if (c) { const k = c.dataset.fxk, v = c.dataset.v, s = fxSel(k); FXS.sel[k] = s.includes(v) ? s.filter(x => x !== v) : s.concat([v]); fxSave(); fxApply(); return; }
  const rm = t.closest('[data-fxrm]');
  if (rm) { FXS.sel[rm.dataset.fxrm] = fxSel(rm.dataset.fxrm).filter(x => x !== rm.dataset.v); fxSave(); fxApply(); return; }
  if (t.closest('[data-fxclear]')) { FXS.sel = {}; fxSave(); fxApply(); return; }
  if (t.closest('[data-fxunhi]')) { FXS.hi = null; fxSave(); fxApply(); return; }
  if (t.closest('a,select,input,th,summary,.cb')) return;
  const r = t.closest('[data-it]');
  if (r && !r.closest('.top')) fxHi(r.dataset.it);
});
document.addEventListener('change', e => {
  const s = e.target.closest && e.target.closest('[data-fxby]');
  if (s) { FXS.by = s.value; fxSave(); fxApply(); }
  const g = e.target.closest && e.target.closest('select[data-go]');
  if (g && g.value) location.href = g.value;
});

// ---------------------------------------------------------------- sortable tables: th[data-sk] (num or text), cells' data-sv
const SORT = {};
try { Object.assign(SORT, JSON.parse(localStorage.getItem('leagues.sort') || '{}')); } catch (e) {}
function sortTable(t) {
  const st = SORT[t.id], body = t.tBodies[0];
  if (!body) return;
  const rows = [...body.rows];
  rows.forEach((r, i) => { if (r.dataset.o === undefined) r.dataset.o = i; });
  const ths = [...t.tHead.rows[0].cells];
  if (st) {
    const col = ths.findIndex(th => th.dataset.sk === st[0]);
    if (col >= 0) {
      const num = ths[col].dataset.st !== 'text', dir = st[1] === 'asc' ? 1 : -1;
      const val = r => { const c = r.cells[col]; const v = c ? (c.dataset.sv !== undefined ? c.dataset.sv : c.textContent.trim()) : ''; return v; };
      rows.sort((a, b) => {
        let x = val(a), y = val(b);
        if (num) { x = x === '' ? null : +x; y = y === '' ? null : +y;
          if (x === null || isNaN(x)) return (y === null || isNaN(y)) ? a.dataset.o - b.dataset.o : 1;
          if (y === null || isNaN(y)) return -1;
          return x === y ? a.dataset.o - b.dataset.o : dir * (x - y); }
        return x === y ? a.dataset.o - b.dataset.o : dir * (x < y ? -1 : 1);
      });
    }
  } else rows.sort((a, b) => a.dataset.o - b.dataset.o);
  rows.forEach(r => body.appendChild(r));
  ths.forEach(th => { const b = th.querySelector('.sb'); if (b) { const on = !!st && st[0] === th.dataset.sk; b.setAttribute('aria-pressed', String(on)); const a = th.querySelector('.sa'); if (a) a.textContent = on ? (st[1] === 'asc' ? ' ▲' : ' ▼') : ''; } });
}
function sortAll() { document.querySelectorAll('table.st[id]').forEach(sortTable); }
document.addEventListener('click', e => {
  const b = e.target.closest && e.target.closest('table.st th[data-sk] .sb');
  if (!b) return;
  const th = b.closest('th'), t = th.closest('table'), k = th.dataset.sk, st = SORT[t.id];
  SORT[t.id] = !st || st[0] !== k ? [k, th.dataset.st === 'text' ? 'asc' : 'desc'] : st[1] === 'desc' ? [k, 'asc'] : st[1] === 'asc' && th.dataset.st === 'text' ? [k, 'desc'] : null;
  if (!SORT[t.id]) delete SORT[t.id];
  try { localStorage.setItem('leagues.sort', JSON.stringify(SORT)); } catch (e) {}
  sortTable(t);
});

// ---------------------------------------------------------------- line charts: div.lgc[data-chart], LG.charts[id]
// {plies: [..], ply, xlab, ylab, refs: [[y, label]], series: {ply: [{it, name, dash, pts: [[x, y, ci, tip]], rings: [[x, y, tip]]}]}}
const NS = 'http://www.w3.org/2000/svg', CH = {};
try { Object.assign(CH, JSON.parse(localStorage.getItem('leagues.charts') || '{}')); } catch (e) {}
function sv(tag, a, text) { const e = document.createElementNS(NS, tag); for (const k in a || {}) e.setAttribute(k, a[k]); if (text !== undefined) e.textContent = text; return e; }
function ticks(lo, hi, n) {
  if (hi <= lo) hi = lo + 1;
  const raw = (hi - lo) / Math.max(1, n), mag = Math.pow(10, Math.floor(Math.log10(raw)));
  const step = [1, 2, 2.5, 5, 10].map(m => m * mag).find(s => s >= raw) || 10 * mag, out = [];
  for (let t = Math.ceil(lo / step) * step; t <= hi + 1e-9; t += step) out.push(Math.round(t * 1000) / 1000);
  return out;
}
function n0(x) { const r = Math.round(x); return (r < 0 ? '−' : '') + Math.abs(r); }
function cpuS(s) { return s < 10 ? s.toFixed(1) + ' CPU-s' : s < 600 ? Math.round(s) + ' CPU-s' : s < 36000 ? Math.round(s / 60) + ' CPU-min' : (s / 3600).toFixed(1) + ' CPU-h'; }
function chartPly(id, C) { const p = (CH[id] && CH[id].ply) || C.ply; return C.series[p] ? p : C.ply; }
function chartsDraw() { document.querySelectorAll('.lgc[data-chart]').forEach(chartDraw); }
function chartDraw(box) {
  const id = box.dataset.chart, C = LG.charts && LG.charts[id];
  if (!C) return;
  const ply = chartPly(id, C), bar = box.previousElementSibling;
  if (bar && bar.classList.contains('cb')) bar.querySelectorAll('[data-ply]').forEach(b => b.setAttribute('aria-pressed', String(b.dataset.ply === ply)));
  const opt = chartOpt(id), all = C.series[ply] || [];
  const ser = all.filter(s => (!s.it || fxPass(s.it)) && (!s.champ || opt.champ));
  const W = Math.max(320, box.clientWidth || 800), H = C.h || 340, m = {l: 52, r: 120, t: 10, b: 30};
  box.innerHTML = '';
  if (!ser.length) { box.innerHTML = '<p class="note">' + (all.length ? 'Nothing the filters show.' : 'No points yet.') + '</p>'; return; }
  let x0 = Infinity, x1 = -Infinity, y0 = Infinity, y1 = -Infinity;
  all.forEach(s => s.pts.forEach(p => { x0 = Math.min(x0, p[0]); x1 = Math.max(x1, p[0]); }));
  ser.forEach(s => s.pts.concat(s.rings || []).forEach(p => { y0 = Math.min(y0, p[1]); y1 = Math.max(y1, p[1]); }));
  if (opt.refs) (C.refs || []).forEach(r => { y0 = Math.min(y0, r[0]); y1 = Math.max(y1, r[0]); });
  if (!isFinite(y0)) { y0 = 0; y1 = 1; }
  if (x1 <= x0) { x0 -= 1; x1 += 1; }
  const pad = Math.max(10, (y1 - y0) * 0.06); y0 -= pad; y1 += pad;
  const X = x => m.l + (x - x0) / (x1 - x0) * (W - m.l - m.r), Y = y => m.t + (1 - (y - y0) / (y1 - y0)) * (H - m.t - m.b);
  const svg = sv('svg', {width: W, height: H, role: 'img', 'aria-label': (C.title || 'chart')});
  ticks(y0, y1, 6).forEach(t => { svg.append(sv('line', {class: 'grid', x1: m.l, x2: W - m.r, y1: Y(t), y2: Y(t)})); svg.append(sv('text', {x: m.l - 6, y: Y(t) + 4, 'text-anchor': 'end'}, n0(t))); });
  const xt = ticks(x0, x1, Math.min(10, Math.round(x1 - x0) || 1)).filter(t => Number.isInteger(t));
  xt.forEach(t => svg.append(sv('text', {x: X(t), y: H - m.b + 16, 'text-anchor': 'middle'}, String(t))));
  svg.append(sv('line', {class: 'axisline', x1: m.l, x2: W - m.r, y1: H - m.b, y2: H - m.b}));
  svg.append(sv('text', {x: W - m.r, y: H - 4, 'text-anchor': 'end'}, C.xlab || 'cycle'));
  if (opt.refs) (C.refs || []).forEach(r => { svg.append(sv('line', {class: 'refl', x1: m.l, x2: W - m.r, y1: Y(r[0]), y2: Y(r[0])})); svg.append(sv('text', {x: W - m.r + 4, y: Y(r[0]) + 4}, r[1])); });
  const hi = FXS.hi, gs = [], dashOf = chartDash(C);
  if (hi && ser.some(s => s.it === hi)) svg.classList.add('hl');
  ser.forEach(s => {
    const col = s.it ? fxColour(s.it) : 'var(--c-ref)', g = sv('g', {class: 's' + (s.it === hi ? ' on' : '')});
    let d = '';
    s.pts.forEach((p, i) => { d += (i && !p[4] ? 'L' : 'M') + X(p[0]).toFixed(1) + ' ' + Y(p[1]).toFixed(1); });
    g.append(sv('path', {class: 'ln' + (s.champ ? ' champ' : ''), d: d, stroke: col, 'stroke-dasharray': dashOf(s)}));
    if (s.pts.length === 1) g.append(sv('circle', {class: 'pt', cx: X(s.pts[0][0]), cy: Y(s.pts[0][1]), r: 3, fill: col}));
    if (opt.rings) (s.rings || []).forEach(p => g.append(sv('circle', {class: 'rg', cx: X(p[0]), cy: Y(p[1]), r: 4.5, stroke: col})));
    if (s.it === hi) {
      s.pts.forEach(p => { if (p[2]) g.append(sv('line', {class: 'wh', x1: X(p[0]), x2: X(p[0]), y1: Y(p[1] - p[2]), y2: Y(p[1] + p[2]), stroke: col})); g.append(sv('circle', {class: 'pt', cx: X(p[0]), cy: Y(p[1]), r: 3.2, fill: col})); });
      const e = s.pts[s.pts.length - 1];
      g.append(sv('text', {class: 'lab', x: X(e[0]) + 7, y: Y(e[1]) + 4}, (s.short || s.name) + ' ' + n0(e[1])));
    }
    svg.append(g); gs.push([s, g]);
  });
  const ov = sv('rect', {class: 'ov', x: m.l - 6, y: 0, width: W - m.l - m.r + 12, height: H - m.b});
  svg.append(ov);
  let cur = null;
  function near(ev) {
    const r = svg.getBoundingClientRect(), px = ev.clientX - r.left, py = ev.clientY - r.top;
    let best = null, bd = 26 * 26;
    gs.forEach(([s, g]) => s.pts.forEach(p => { const dx = X(p[0]) - px, dy = Y(p[1]) - py, dd = dx * dx + dy * dy; if (dd < bd) { bd = dd; best = [s, g, p]; } }));
    return best;
  }
  ov.addEventListener('pointermove', ev => {
    const b = near(ev);
    if (cur && (!b || cur[1] !== b[1])) cur[1].classList.remove('hov');
    svg.classList.toggle('hv', !!b);
    if (!b) { cur = null; tip.hidden = true; tip.dataset.chart = ''; return; }
    b[1].classList.add('hov'); cur = b;
    const p = b[2];
    tip.textContent = (b[0].name) + '\n' + (C.xlab || 'cycle') + ' ' + p[0] + ': ' + (C.fmt === 'cpu' || C.fmt === 'price' ? '' : (C.ylab || 'Elo') + ' ') + (C.fmt === 'cpu' ? cpuS(p[1]) : C.fmt === 'price' ? cpuS(p[1]).replace('CPU-', 'priced CPU-') : n0(p[1])) + (p[2] ? ' ± ' + Math.round(p[2]) : '') + (p[3] ? '\n' + p[3] : '');
    tip.dataset.chart = '1'; tip.hidden = false; tipAt(ev.clientX, ev.clientY);
  });
  ov.addEventListener('pointerleave', () => { if (cur) cur[1].classList.remove('hov'); cur = null; svg.classList.remove('hv'); tip.hidden = true; tip.dataset.chart = ''; });
  ov.addEventListener('click', ev => { const b = near(ev); if (b && b[0].it) fxHi(b[0].it); });
  box.append(svg);
  document.querySelectorAll('.lgl[data-for="' + id + '"] [data-it]').forEach(b => {
    const l = b.querySelector('line');
    if (l) { l.setAttribute('stroke', fxColour(b.dataset.it)); l.setAttribute('stroke-dasharray', dashOf({it: b.dataset.it})); }
  });
}
// a series' dash: from the first of C.dashBy that the page doesn't colour by (an experiment's model, else its option)
const DASH = ['', '7 4', '2 3', '10 3 2 3', '5 2 1 2'];
function chartDash(C) {
  const dk = (C.dashBy || []).find(k => k !== fxBy() && fxFactors().some(f => f[0] === k));
  const dv = dk ? fxFactors().find(f => f[0] === dk)[2] : [];
  return s => {
    if (s.dash) return '5 3';
    const f = dk && s.it ? fxItems()[s.it] : null;
    if (!f) return '';
    const i = dv.indexOf(f.v[dk]);
    return i > 0 ? DASH[i % DASH.length] : '';
  };
}
// a chart's toggles: the user's choice (CH), else the chart's default (C.defaults), else off
function chartOpt(id) { const C = (LG.charts && LG.charts[id]) || {}; return Object.assign({}, C.defaults || {}, CH[id] || {}); }
document.addEventListener('click', e => {
  const b = e.target.closest && e.target.closest('.cb button[data-cset]');
  if (!b) return;
  const id = b.closest('.cb').dataset.for, k = b.dataset.cset;
  const on = !!chartOpt(id)[k];
  CH[id] = CH[id] || {};
  if (k === 'ply') CH[id].ply = b.dataset.ply; else CH[id][k] = !on;
  if (k !== 'ply') b.setAttribute('aria-pressed', String(!on));
  try { localStorage.setItem('leagues.charts', JSON.stringify(CH)); } catch (e) {}
  const box = document.querySelector('.lgc[data-chart="' + id + '"]');
  if (box) chartDraw(box);
});
let rsz = 0;
addEventListener('resize', () => { cancelAnimationFrame(rsz); rsz = requestAnimationFrame(chartsDraw); });

// ---------------------------------------------------------------- refresh every LG.refresh ms: the page's #app, kept state
function boot() { lgData(); fxLoad(); sortAll(); fxApply(); document.querySelectorAll('.cb[data-for]').forEach(b => { const s = chartOpt(b.dataset.for); b.querySelectorAll('[data-cset]').forEach(x => { const k = x.dataset.cset; if (k !== 'ply') x.setAttribute('aria-pressed', String(!!s[k])); }); }); }
setInterval(async () => {
  if (!LG.refresh) return;
  try {
    const r = await fetch(location.pathname + location.search, {cache: 'no-store'});
    if (!r.ok) throw new Error(r.status);
    const doc = new DOMParser().parseFromString(await r.text(), 'text/html'), app = doc.getElementById('app');
    if (!app) return;
    const open = new Map([...document.querySelectorAll('details[id]')].map(x => [x.id, x.open]));
    const sk = new Map([...document.querySelectorAll('[data-sk][id]')].map(x => [x.id, [x.scrollTop, x.scrollLeft]]));
    document.getElementById('app').replaceWith(document.adoptNode(app));
    const nd = doc.getElementById('lg-data'), od = document.getElementById('lg-data');
    if (nd && od) od.textContent = nd.textContent;
    const nn = doc.querySelector('nav.lg'), on = document.querySelector('nav.lg');
    if (nn && on) on.replaceWith(document.adoptNode(nn));
    open.forEach((v, id) => { const x = document.getElementById(id); if (x) x.open = v; });
    sk.forEach((v, id) => { const x = document.getElementById(id); if (x) { x.scrollTop = v[0]; x.scrollLeft = v[1]; } });
    document.getElementById('stale').hidden = true;
    boot();
  } catch (err) { const s = document.getElementById('stale'); if (s) s.hidden = false; }
}, LG.refresh || 30000);
boot();
"""


if __name__ == "__main__":
    main()
