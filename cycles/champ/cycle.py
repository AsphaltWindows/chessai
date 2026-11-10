#!/usr/bin/env python3
"""Training/tournament cycles under the champion rules (the user's design, 2026-11-07; see README.md).

A fresh ladder (config league_dir: selfplay/league-champ) for the lineages of the first run, whose files stay as its
record. The lineages keep their run directories, so their versions keep counting up.

Cycle 1 seeds the ladder and trains nothing: each lineage enters with its latest version at plies 1 and 2 and with its
peak from the first run's league (its best ply-2 Elo there, retired entries included) at ply 2; the field plays its
round-robins; the better of the two at ply 2 becomes the lineage's first champion, and the other leaves the field (its
games stay in the fit). A lineage whose champion is the old peak resumes from a copy of it, as its next version.

Every later cycle: (1) self-play trains each active lineage for an equal share of a fixed CPU budget per cycle
(cycle_budget_cpu_s split evenly among the active lineages; what a job over- or underspends, whole rounds being the
self-play tool's unit, carries to its next cycle), each job measuring its CPU per game as it runs; (2) each lineage's
newest version replaces its entries at plies 1 and 2, and its champion keeps its ply-2 entry; (3) the whole field
plays fresh double round-robins (round_robins_per_cycle, a constant 5); (4) the champion test: the current version at
ply 2 becomes the champion when it beats the champion by champion_se standard errors of their difference, from the
fit's covariance; a lineage whose self-play since its last new champion reaches patience_cpu_s CPU seconds finishes:
its champion is frozen at an untaken multiple of 100 within 10 of its ply-2 Elo (it keeps playing as a fixed player
and is never checked again), else retired (it leaves the field); (5) every active lineage trains once more on its
current version's games of that tournament against the lineages' entries and keeps the result only if it scores at
least the gate's threshold against the model it started from. Every global_every cycles the retired champions rejoin
the round-robins (the global tournament): one that scores revive_se standard errors above what its rating before
them predicts is revived and trains again from its champion; one within 10 of an untaken milestone is frozen there;
the others leave again. A new lineage enters after its first training cycle, its first version being its champion.
A solo lineage (steps - in lineages.txt) skips (5): it trains on its own self-play only, and its row's gate in the log
says "solo: no own step"; its entries play and are rated, and the other lineages' steps train on their games.

    cycle.py [--config FILE] status                show the lineages, their champions and the next cycle's shares
    cycle.py [--config FILE] dry-run [--cycles N]  the seeding cycle's field, the schedule and every lineage's resume
                                                   point, from the files as they are now; writes nothing
    cycle.py [--config FILE] measure [--force]     measure the CPU cost per game of every recipe into costs.csv
    cycle.py [--config FILE] setup-league          make the league (once; safe to repeat)
    cycle.py [--config FILE] refit                 rewrite the league's reports (plays nothing)
    cycle.py [--config FILE] init [--force]        make the league and state.json: the seeding plan (each lineage's
                                                   latest version and old peak) as cycle 0
    cycle.py [--config FILE] run [--cycles N]      run N cycles (default 1); the first is the seeding cycle
    cycle.py [--config FILE] freeze NAME           by hand: the lineage's champion stays in the field as a fixed
                                                   player, and it stops training (unfreeze NAME: back to training)
    cycle.py [--config FILE] retire NAME           by hand, after taking its line out of lineages.txt: its entries
                                                   leave the field (unretire NAME brings them back)
    cycle.py [--config FILE] verify NAME... [--rounds R | --cycle C]
                                                   replay a lineage's self-play and compare the models byte for byte
    cycle.py selftest                              test the rules, the budget split, the standard errors, the gate's
                                                   arithmetic, the league updates and the saved-games reader
"""


import argparse
import csv
import datetime
import fcntl
import hashlib
import io
import json
import math
import os
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
import time
from concurrent.futures import ThreadPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))
LADDER_HEADER = "round,version,opponent,wins,losses,draws,score,elo,elo_low,elo_high\n"
OUTCOMES = ("white_wins.games", "black_wins.games", "draws.games")
TREE_KINDS = {"bdt", "bdd", "ht", "tan"}  # resume and train --start from {kind}N.state, not the model file (TAN's
# state holds its pair counts, which its model file lacks)
STEPS = {"own"}
SOLO = "solo: no own step"  # the log row's gate of a solo lineage (steps - in lineages.txt: self-play only)
MEASURE_MIN, MEASURE_AIM = 0.05, 0.06  # a self-play job's measuring segment: at least 5% of its budget (aims at 6%)
CHUNK = 0.22  # then chunks of about this share of the budget, each re-planned from the last one's cost (the user)
DRIVER_FLAGS = {"--rounds", "--games", "--threads", "--resume", "--init"}  # set by the driver, never by the registry
NOT_COST = {"--seed", "--no-games", "--vs-previous", "--use-probs", "--model", "--depth", "--policy", "--temperature",
            "--label-depth", "--label-result"}
SELFPLAY_ONLY = {"--model", "--seed", "--rounds", "--games", "--policy", "--temperature", "--threads", "--vs-previous",
                 "--no-games", "--depth", "--round-forget", "--age-forget", "--resume", "--keep-states", "--init"}
LABEL_FLAGS = {"--label-depth", "--label-result", "--label-model"}  # the own step's train takes them too (the user)
TOURNAMENT_FLAGS = ["--no-catch-up", "--retired", "--save-games"]  # the cycles need all of them
LOG_FIELDS = ["cycle", "date", "lineage", "status", "version", "games", "rounds", "round_size", "cpu_s", "wall_s",
              "cost_ms", "segment_rounds", "start_version", "end_version", "budget_cpu_s", "carry_cpu_s",
              "patience_cpu_s", "entries", "elo", "ci95", "rel_elo", "champion_version", "champion_elo",
              "champion_ci95", "diff", "diff_se", "decision", "note", "revive_games", "revive_score",
              "revive_expected", "revive_z", "own_games", "own_positions", "own_cpu_s", "gate_wins", "gate_losses",
              "gate_draws", "gate_score", "gate_elo", "gate_ci95", "gate_cpu_s", "gate", "next_version"]
TOTALS_FIELDS = ["cycle", "date", "kind", "active", "frozen", "retired", "trained", "field", "round_robins", "games",
                 "tournament_min", "budget_cpu_s", "share_cpu_s", "selfplay_cpu_s", "own_cpu_s", "gate_cpu_s",
                 "new_champions", "finished_frozen", "finished_retired", "revived", "global_frozen", "left_again"]
COST_FIELDS = ["signature", "cpu_per_game", "games", "rounds", "threads", "cpu_s", "wall_s", "binary", "models",
               "date", "loadavg"]
NOTE_START, NOTE_END = "# The champion rules (cycles/champ/cycle.py)", "# Don't run ./run.sh here"
MODELS_HEADER = "name,archetype,variant,games,moves,train_depth,labels,init,rounds,seed,calibrated,run_dir,script\n"
RETIRED_HEADER = ("# Retired players (cycles/champ/cycle.py): NAME DEPTH SPEC, as in players.txt. They play no more "
                  "games,\n# but their games stay in games.csv and in the fit (tournament --retired).\n")


def now():
    return f"{datetime.datetime.now():%Y-%m-%d %H:%M}"


class Config:
    def __init__(self, path):
        self.path = os.path.abspath(path)
        with open(self.path) as f:
            self.raw = json.load(f)
        self.root = self.raw["root"]

    def __getattr__(self, key):
        try:
            return self.__dict__["raw"][key]
        except KeyError:
            raise AttributeError(key)

    def abs(self, p):
        return p if os.path.isabs(p) else os.path.join(self.root, p)

    def file(self, key):
        return self.abs(self.raw[key])

    def rel(self, p):
        r = os.path.relpath(p, self.root)
        return p if r.startswith("..") else r


# ---------------------------------------------------------------- registry

def parse_flags(tokens):
    """[(flag, value or True)] in order; a flag takes the next token as its value unless that is another flag."""
    out, i = [], 0
    while i < len(tokens):
        t = tokens[i]
        if not t.startswith("--"):
            raise ValueError(f"unexpected token {t!r}")
        if i + 1 < len(tokens) and not tokens[i + 1].startswith("--"):
            out.append((t, tokens[i + 1]))
            i += 2
        else:
            out.append((t, True))
            i += 1
    return out


def signature(kind, flags):
    """The recipe's cost key: model kind, search depth, move choice, labels and every other model option."""
    f = dict(flags)
    policy = f.get("--policy", "wld-score-jitter10")
    if "softmax" in policy:
        move = policy.replace("log-odds-", "").replace("wld-score-", "wld-") + str(f.get("--temperature", "0.01"))
    else:
        move = {"wld-score-jitter10": "jitter10"}.get(policy, policy)
    sig = f"{kind} d{int(f.get('--depth', 0))} {move}"
    if "--label-depth" in f:
        sig += f" labels-d{f['--label-depth']}-r{f.get('--label-result', '0')}"
    rest = sorted(k if v is True else f"{k} {v}" for k, v in flags if k not in NOT_COST)
    return " ".join([sig] + rest)


def load_registry(cfg):
    lineages = []
    with open(cfg.file("registry")) as fh:
        lines = fh.readlines()
    for n, line in enumerate(lines, 1):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        where = f"{cfg.registry}:{n}"
        parts = [p.strip() for p in line.split(" | ")]
        if len(parts) != 7:
            sys.exit(f"{where}: want 7 fields separated by ' | ', got {len(parts)}")
        name, archetype, binary, source, heldout, steps, flag_text = parts
        if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9+.-]*", name):
            sys.exit(f"{where}: bad lineage name {name!r}")
        tokens = shlex.split(flag_text)
        try:
            flags = parse_flags(tokens)
        except ValueError as e:
            sys.exit(f"{where}: {name}: {e}")
        bad = DRIVER_FLAGS & {k for k, _ in flags}
        if bad:
            sys.exit(f"{where}: {name}: the driver sets {', '.join(sorted(bad))}")
        f = dict(flags)
        if "--model" not in f or "--seed" not in f:
            sys.exit(f"{where}: {name}: give --model and --seed")
        step_set = set() if steps == "-" else {s.strip() for s in steps.split(",")}
        if step_set - STEPS:
            sys.exit(f"{where}: {name}: unknown steps {', '.join(sorted(step_set - STEPS))}")
        stype, _, spath = source.partition(":")
        if stype not in ("continue", "scratch", "init") or (stype != "scratch") != bool(spath):
            sys.exit(f"{where}: {name}: bad source {source!r}")
        held = None if heldout == "-" else cfg.abs(cfg.heldout.get(heldout, heldout))
        kind = f["--model"]
        lineages.append(dict(
            name=name, archetype=archetype, binary=cfg.abs(binary), source_type=stype,
            source=cfg.abs(spath) if spath else None, heldout=held, steps=step_set, tokens=tokens, flags=flags,
            kind=kind, depth=int(f.get("--depth", 0)), seed=f["--seed"], sig=signature(kind, flags),
            run_dir=os.path.join(cfg.file("runs_dir"), name)))
    names = [l["name"] for l in lineages]
    dup = {x for x in names if names.count(x) > 1}
    if dup:
        sys.exit(f"duplicate lineages: {', '.join(sorted(dup))}")
    return lineages


def own_on(cfg, lin):
    return cfg.own_games.get("enabled", True) and "own" in lin["steps"]


def describe(lin):
    """models.csv's moves and labels columns for a lineage."""
    f = dict(lin["flags"])
    policy = f.get("--policy", "wld-score-jitter10")
    moves = "jitter10" if policy == "wld-score-jitter10" else (
        f"softmax{f.get('--temperature', '0.01')}" if policy == "log-odds-softmax" else policy)
    if "--label-depth" in f:
        r = float(f.get("--label-result", 0))
        labels = f"{round(100 * r)}/{round(100 * (1 - r))} result + d{f['--label-depth']} search"
    else:
        labels = "result"
    return moves, labels


def model_args(lin):
    """The recipe's options for the own step's train: the self-play flags minus the self-play-only ones. Its labels
    (LABEL_FLAGS) stay, so that train labels the own games as self-play labels its games (the user, 2026-11-06)."""
    out = []
    for k, v in lin["flags"]:
        if k not in SELFPLAY_ONLY:
            out += [k] if v is True else [k, v]
    return out


def spec(lin, path):
    """A league or arena spec for a model file of the lineage: calibrated on its held-out set if it has one."""
    return f"cal:{lin['heldout']}:{lin['kind']}:{path}" if lin["heldout"] else f"{lin['kind']}:{path}"


# ---------------------------------------------------------------- costs and plans

def load_costs(cfg):
    costs = {}
    p = cfg.file("costs")
    if os.path.exists(p):
        with open(p) as f:
            for r in csv.DictReader(f):
                costs[r["signature"]] = float(r["cpu_per_game"])  # the last measurement of a signature wins
    return costs


def rule_depths(lineages):
    """The depths the round-size rule measures LR at: 0-3 and any deeper lineage's."""
    return sorted({0, 1, 2, 3} | {lin["depth"] for lin in lineages})


def round_size(cfg, costs, depth):
    """Games per round at a training depth: a round costs about what a round of round_anchor_games costs at depth 0
    (the user's rule), from LR's costs with jitter10; never more than round_anchor_games."""
    anchor = cfg.round_anchor_games
    if depth == 0:
        return anchor
    c0, cd = costs.get("lr d0 jitter10"), costs.get(f"lr d{depth} jitter10")
    if not c0 or not cd:
        return None
    return min(anchor, max(10, int(round(anchor * c0 / cd / 10.0)) * 10))


def last_version(run_dir, kind):
    """The last version --resume would find: versions 1, 2, ... while the file exists."""
    n = 0
    while run_dir and os.path.exists(os.path.join(run_dir, f"{kind}{n + 1}.model")):
        n += 1
    return n


def plan(cfg, costs, lin, st, budget):
    """The next cycle's self-play for a prepared lineage with this CPU budget (its share of the cycle's, plus its
    carry), or (None, reason). The job measures its own cost when it runs (see next_step): the plan's rounds are only
    an estimate, from the lineage's last measured cost or else the cost table, for the status, the first measuring
    segment's size and the dashboard's ETA."""
    if not os.path.exists(lin["binary"]):
        return None, "waiting for binary"
    size = round_size(cfg, costs, lin["depth"])
    if not size:
        return None, f"no LR costs for depth {lin['depth']} (run measure)"
    cost, source = st.get("cost"), "measured"
    if not cost:
        cost, source = costs.get(lin["sig"]), "table"
    if not cost:
        return None, f"no cost for '{lin['sig']}' (run measure)"
    rounds = max(1, int(budget / cost / size + 0.5))
    v = st["version"] or 0
    if v == 0 or (lin["kind"] == "moe" and v < 2):
        rounds = max(rounds, 2 - v)  # --resume needs two versions: the model and its opponent
    base = ["nice", "-n", str(cfg.nice), lin["binary"], "self-play", lin["run_dir"], *lin["tokens"],
            "--games", str(size), "--threads", str(cfg.threads_per_job)]
    init = ["--init", lin["source"]] if lin["source_type"] == "init" else []
    return dict(rounds=rounds, size=size, games=rounds * size, start=v, cost=cost, cost_source=source, base=base,
                init=init, budget=round(budget, 2)), None


def keep_start_states(lin, v):
    """Hard-links a tree's training states of versions v and v-1, where its cycle starts, into
    runs/<lineage>/cycle-states/, so that `verify --cycle` can replay the cycle later: self-play keeps only the
    last two states."""
    if lin["kind"] not in TREE_KINDS or not v:
        return
    keep = os.path.join(lin["run_dir"], "cycle-states")
    for n in (v - 1, v):
        a, b = (os.path.join(lin["run_dir"], f"{lin['kind']}{n}.state"),
                os.path.join(keep, f"{lin['kind']}{n}.state"))
        if n >= 1 and os.path.exists(a) and not os.path.exists(b):
            os.makedirs(keep, exist_ok=True)
            try:
                os.link(a, b)
            except OSError:
                shutil.copy2(a, b)


def next_step(cfg, lin, p, result):
    """A lineage's self-play job for this cycle, one command at a time, for its CPU budget p["budget"] (its share of
    the cycle's budget plus its carry; "the budget" below). `p` is the job in state.json (the caller
    saves it after every call); `result` is None before the job's first command (also after a restart), else
    (exit code, CPU s, wall s) of the last one.
    First the measuring segment: the run's next rounds, enough for MEASURE_MIN of the budget (more commands if one
    falls short); its CPU per game is p["measured"]. Then the rest in chunks of about CHUNK of the budget (the user,
    2026-11-07: a job's cost per game drifts as it trains, so one plan made after the segment over- or undershot the
    budget). Before each chunk, the rounds left are int(budget left / (cost x size) + 0.5): the budget left is the
    budget minus the CPU the job has used (its parts, plus the rounds a command cut short by a driver restart saved,
    at the cost then known: p["cut_cpu"]), the cost the last chunk's own (the segment's for the first). The job
    stops when that is 0; p["end"] is the planned end, the last version once it is over, p["total"] its rounds from
    the cycle's start version. Resume is exact, so the chunks change only how many rounds are played. A job
    interrupted in its segment measures again. Each chunk's part logs its rounds and CPU per game. Returns the next
    command, or None when the job is over (p["failed"] says why if it failed)."""
    if "base" not in p:  # planned by the driver before measuring segments: its one command
        if result is None:
            return None if p.get("parts") else p["cmd"]
        p.setdefault("parts", []).append(["run", result[0], round(result[1], 2), round(result[2], 2)])
        return None
    run_dir, kind, size, budget = lin["run_dir"], lin["kind"], p["size"], float(p["budget"])
    v = last_version(run_dir, kind)
    covered = p.get("covered", p["start"])  # the last version a recorded command reached
    if result is None:
        if v > covered:  # rounds saved by a command the driver's restart cut short: their CPU, estimated
            p["cut_cpu"] = round(p.get("cut_cpu", 0.0) + (v - covered) * (p.get("cost_now") or p["cost"]) * size, 2)
        p["covered"] = v
    else:
        code, cpu, wall = result
        part = [p.get("step"), code, round(cpu, 2), round(wall, 2)]
        if p.get("step") == "chunk":
            part += [v - covered, round(cpu / ((v - covered) * size), 7) if v > covered else None]
        p.setdefault("parts", []).append(part)
        p["covered"] = v
        if code != 0:
            p["failed"] = f"self-play exited with {code}"
            return None
        if p.get("step") == "measure" and "measured" not in p:
            seg = p["segment"]
            seg["cpu"] += cpu
            seg["rounds"] = v - seg["from"]
            if seg["rounds"] < 1:
                p["failed"] = "the measuring segment saved no version"
                return None
            if seg["cpu"] >= MEASURE_MIN * budget:
                cost = seg["cpu"] / (seg["rounds"] * size)
                p["measured"] = dict(cost=cost, rounds=seg["rounds"], cpu=round(seg["cpu"], 2), at=v)
                p["cost_now"] = cost
        elif p.get("step") == "chunk":
            if v <= covered:
                p["failed"] = "a chunk saved no version"
                return None
            p["cost_now"] = max(cpu, 1e-6) / ((v - covered) * size)
    if "measured" in p:
        used = sum(x[2] for x in p.get("parts", [])) + p.get("cut_cpu", 0.0)
        left = max(0, int((budget - used) / (p["cost_now"] * size) + 0.5))
        p["end"], p["total"] = v + left, v + left - p["start"]
        if left == 0:
            return None
        p["step"] = "chunk"
        return p["base"] + ["--rounds", str(v + min(left, max(1, int(CHUNK * budget / (p["cost_now"] * size) + 0.5)))),
                            "--resume"]
    fresh = v == 0 or (kind == "moe" and v < 2)  # --resume needs a model and its opponent
    if result is None or p.get("step") != "measure":  # a new segment; a cut-short one is measured again
        p["segment"] = {"from": 0 if fresh else v, "cpu": 0.0, "rounds": 0}
    seg = p["segment"]
    est = seg["cpu"] / (seg["rounds"] * size) if seg["rounds"] else p["cost"]
    m = max(1, math.ceil((MEASURE_AIM * budget - seg["cpu"]) / (est * size)))
    p["step"] = "measure"
    if fresh:
        return p["base"] + ["--rounds", str(max(2 if kind == "moe" else 1, v + m))] + p["init"]
    return p["base"] + ["--rounds", str(v + m), "--resume"]


# ---------------------------------------------------------------- processes

def cmdline(pid):
    try:
        with open(f"/proc/{pid}/cmdline", "rb") as f:
            return f.read().replace(b"\0", b" ").decode(errors="replace")
    except OSError:
        return ""


def source_busy(path):
    """Is a self-play run still writing this run directory? (Arenas and tournaments only read it.)"""
    me, path = os.getpid(), path.rstrip("/")
    pattern = re.compile(r"\bself-play " + re.escape(path) + r"/?( |$)")
    return any(p.isdigit() and int(p) != me and pattern.search(cmdline(p)) for p in os.listdir("/proc"))


def ancestors():
    out, pid = set(), os.getpid()
    while pid > 1 and pid not in out:
        out.add(pid)
        try:
            with open(f"/proc/{pid}/stat") as f:
                pid = int(f.read().rsplit(")", 1)[1].split()[1])
        except (OSError, ValueError, IndexError):
            break
    return out


def league_busy(cfg):
    """A process using the league, or None: the tournament tool, or a shell running one of the league's scripts
    (run.sh, ...), started in the league's directory or naming it."""
    league = cfg.file("league_dir")
    scripts = {f for f in os.listdir(league) if f.endswith(".sh")}
    mine = ancestors()
    for p in os.listdir("/proc"):
        if not p.isdigit() or int(p) in mine:
            continue
        c = cmdline(p)
        try:
            cwd = os.readlink(f"/proc/{p}/cwd")
        except OSError:
            cwd = ""
        if not c or (cwd != league and league not in c):
            continue
        toks = c.split()
        script = os.path.basename(toks[0]) in ("bash", "sh") and any(os.path.basename(t) in scripts for t in toks[1:])
        if script or " tournament " in c + " ":
            return f"pid {p}: {c.strip()[:160]}"
    return None


def wait_league(cfg):
    said = False
    while True:
        busy = league_busy(cfg)
        if not busy:
            return
        if not said:
            print(f"league busy ({busy}); waiting", flush=True)
            said = True
        time.sleep(60)


def run_measured(cmd, log_path, title):
    """Runs cmd, appending its output to log_path. Returns (exit code, CPU s, wall s, output)."""
    with open(log_path, "a") as f:
        f.write(f"== {title} {now()}: {shlex.join(cmd)}\n")
        f.flush()
        t0 = time.time()
        proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        out = proc.stdout.read()
        _, status, usage = os.wait4(proc.pid, 0)
        proc.returncode = code = os.waitstatus_to_exitcode(status)
        cpu, wall = usage.ru_utime + usage.ru_stime, time.time() - t0
        f.write(out)
        f.write(f"== {title}: exit {code}, cpu {cpu:.1f} s, wall {wall:.1f} s\n")
    return code, cpu, wall, out


def run_jobs(cfg, jobs, cycle, on_start=None, on_done=None):
    """Runs jobs, at most cfg.jobs at a time. A job is (name, step, log): step(None) gives its first command and
    step((exit code, CPU s, wall s)) the next one as each finishes, in the same slot, until it gives None. Calls
    on_start(name) as a job starts and on_done(name) as it ends. Returns the names done."""
    pending, running, done = list(jobs), {}, []

    def launch(name, cmd, step, log):
        run_dir = cmd[cmd.index("self-play") + 1]
        while source_busy(run_dir):  # a job left running by an interrupted driver
            time.sleep(30)
        v = last_version(run_dir, cmd[cmd.index("--model") + 1]) if "--resume" in cmd else 0
        for side in ("ladder.csv", "results.txt"):  # self-play appends a round's rows before saving its model:
            if os.path.exists(os.path.join(run_dir, side)):  # an interrupted round's rows go, as it is replayed
                trim_side_file(os.path.join(run_dir, side), v)
        f = open(log, "a")
        f.write(f"== cycle {cycle} self-play {now()}: {shlex.join(cmd)}\n")
        f.flush()
        proc = subprocess.Popen(cmd, stdout=f, stderr=subprocess.STDOUT)
        running[proc.pid] = (name, proc, f, time.time(), step, log)
        return proc.pid

    while pending or running:
        while pending and len(running) < cfg.jobs:
            name, step, log = pending.pop(0)
            cmd = step(None)
            if cmd is None:
                done.append(name)
                if on_done:
                    on_done(name)
                continue
            pid = launch(name, cmd, step, log)
            print(f"  started {name} (pid {pid})", flush=True)
            if on_start:
                on_start(name)
        if not running:
            continue
        pid, status, usage = os.wait4(-1, 0)
        if pid not in running:
            continue
        name, proc, f, t0, step, log = running.pop(pid)
        proc.returncode = code = os.waitstatus_to_exitcode(status)
        cpu, wall = usage.ru_utime + usage.ru_stime, time.time() - t0
        f.write(f"== cycle {cycle} self-play: exit {code}, cpu {cpu:.1f} s, wall {wall:.1f} s\n")
        f.close()
        cmd = step((code, cpu, wall))
        if cmd is not None:
            launch(name, cmd, step, log)
            continue
        done.append(name)
        if on_done:
            on_done(name)
    return done


_HELP = {}


def supports(binary, sub, flag):
    if not os.path.exists(binary):
        return False
    if (binary, sub) not in _HELP:
        r = subprocess.run([binary, sub, "--help"], capture_output=True, text=True)
        _HELP[binary, sub] = r.stdout + r.stderr
    return flag in _HELP[binary, sub]


# ---------------------------------------------------------------- the league's files

def read_entries(path):
    """[(name, depth, spec)] of a players.txt or retired.txt, in order."""
    out = []
    if os.path.exists(path):
        with open(path) as f:
            for line in f:
                s = line.strip()
                if s and not s.startswith("#"):
                    name, depth, sp = s.split()
                    out.append((name, int(depth), sp))
    return out


def read_players(league):
    return read_entries(os.path.join(league, "players.txt"))


def model_of(name, depth):
    """league.py's rule: a player's model is its name without the depth suffix (-2, or -d2 for Nov 5's players)."""
    return re.sub(r"-d?%s$" % depth, "", name) if depth != 0 else name


def match_entries(players, source):
    """The league players that are versions of the run in `source`: their model file is in that directory."""
    prefix = source.rstrip("/") + "/"
    return [(n, d, s) for n, d, s in players if s.rsplit(":", 1)[-1].startswith(prefix)]


def read_ratings(league):
    p = os.path.join(league, "ratings.csv")
    if not os.path.exists(p):
        return {}
    with open(p) as f:
        return {r["player"]: (float(r["elo"]), float(r["ci95"]), int(r["games"])) for r in csv.DictReader(f)}


def rating_of(ratings, names):
    """The mean Elo and mean ± of these players, or None if none is rated."""
    rs = [ratings[n] for n in names if n in ratings]
    if not rs:
        return None
    return sum(r[0] for r in rs) / len(rs), sum(r[1] for r in rs) / len(rs)


def rating_at(ratings, names, depths, depth):
    """The rating of a lineage's entries at one ply (their mean: one model can have two entries at a ply)."""
    return rating_of(ratings, [n for n in names if depths.get(n) == depth])


def references_mean(ratings, refs):
    """The mean Elo of the fixed references (state["references"]) rated in this fit, which makes lineage ratings
    comparable across fits."""
    rs = [ratings[n][0] for n in refs if n in ratings]
    return sum(rs) / len(rs) if rs else 0.0


def rel_references(state):
    """The players whose mean Elo rel ("vs refs") subtracts: state["rel_references"] if set, else the references.
    From cycle 5 it is the original three (random, random-1, random-2), so that rel keeps its meaning when
    references join (random-3 and random-4 from cycle 5: they play every cycle and link the fits, but stay out of
    this mean)."""
    return state.get("rel_references") or state["references"]


ANCHOR = "random"  # the tournament's --anchor, pinned at Elo 0


def fit_factor(league, ratings, anchor=ANCHOR):
    """The league fit's information matrix (Elo units) at its ratings, Cholesky-factored, with each rated player's
    row (the anchor has none: it is pinned): from every game in games.csv between rated players, retired ones
    included. The tournament tool's fit is the same (Bradley-Terry, draws as half points, the anchor at 0;
    ratings.csv's ± comes from this matrix), so contrast_se gives a difference of ratings its standard error from
    the fit's covariance. None if the matrix is not positive definite."""
    names = [p for p in ratings if p != anchor]
    index = {p: i for i, p in enumerate(names)}
    count = {}
    with open(os.path.join(league, "games.csv")) as f:
        next(f, None)
        for line in f:
            _, white, black, _ = line.split(",", 3)
            if white in ratings and black in ratings and white != black:
                key = (white, black) if white < black else (black, white)
                count[key] = count.get(key, 0) + 1
    k = len(names)
    info = [[0.0] * k for _ in range(k)]
    for (a, b), n in count.items():
        p = 1 / (1 + 10 ** (-(ratings[a][0] - ratings[b][0]) / 400))
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
        d = row_j[j] - sum(v * v for v in row_j[:j])
        if not d > 0:
            return None
        row_j[j] = d = math.sqrt(d)
        for i in range(j + 1, k):
            row_i = info[i]
            row_i[j] = (row_i[j] - sum(x * y for x, y in zip(row_i[:j], row_j[:j]))) / d
    return info, index


def contrast_se(factor, a, b):
    """The standard error (Elo) of the mean rating of players `a` minus that of players `b`, in the fit that
    `factor` (fit_factor) came from: sqrt(e' I^-1 e) = |L^-1 e|."""
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


def contrast_ses(league, ratings, pairs, anchor=ANCHOR):
    """Each pair's standard error, as contrast_se gives it from fit_factor (the fit's covariance: the inverse of its
    information matrix at its ratings, from every game in games.csv between rated players, retired ones included, the
    anchor pinned), but by eliminating the players that are in no pair from the sparse matrix first, fewest opponents
    first, and factoring only the small dense rest. Exact (Gaussian elimination is the Cholesky factorization in
    another order), and quick: an entry that played one cycle's field only adds nothing new when it is eliminated,
    since that field played itself, so the work grows with the games rather than with the cube of the players. pairs:
    [(players a, players b)], each a difference of the mean rating of a and that of b. Returns the list of standard
    errors, or None if the matrix is not positive definite."""
    w2 = (math.log(10) / 400) ** 2
    count = {}
    with open(os.path.join(league, "games.csv")) as f:
        next(f, None)
        for line in f:
            _, white, black, _ = line.split(",", 3)
            if white != black and white in ratings and black in ratings:
                key = (white, black) if white < black else (black, white)
                count[key] = count.get(key, 0) + 1
    a_ = {}
    for (a, b), n in count.items():
        p = 1 / (1 + 10 ** (-(ratings[a][0] - ratings[b][0]) / 400))
        w = n * p * (1 - p) * w2
        for x in (a, b):
            if x != anchor:
                row = a_.setdefault(x, {})
                row[x] = row.get(x, 0.0) + w
        if a != anchor and b != anchor:
            a_[a][b] = a_[a].get(b, 0.0) - w
            a_[b][a] = a_[b].get(a, 0.0) - w
    keep = sorted({x for a, b in pairs for x in list(a) + list(b) if x in a_})
    kept = set(keep)
    for x in sorted((x for x in a_ if x not in kept), key=lambda x: (len(a_[x]), x)):
        row = a_.pop(x)
        d = row.pop(x)
        if not d > 0:
            return None
        nb = list(row.items())
        for u, a_ux in nb:
            ru = a_[u]
            del ru[x]
            f = a_ux / d
            for v, a_xv in nb:
                ru[v] = ru.get(v, 0.0) - f * a_xv
    k = len(keep)
    lower = [[a_[p].get(q, 0.0) for q in keep] for p in keep]
    for j in range(k):  # Cholesky, in place, as fit_factor's
        row_j = lower[j]
        d = row_j[j] - sum(v * v for v in row_j[:j])
        if not d > 0:
            return None
        row_j[j] = d = math.sqrt(d)
        for i in range(j + 1, k):
            row_i = lower[i]
            row_i[j] = (row_i[j] - sum(x * y for x, y in zip(row_i[:j], row_j[:j]))) / d
    factor = (lower, {p: i for i, p in enumerate(keep)})
    return [contrast_se(factor, a, b) for a, b in pairs]


def league_depths(cfg):
    """{player: ply} of the league's active and retired players."""
    return {n: d for n, d, _ in read_players(cfg.file("league_dir")) + read_entries(retired_path(cfg))}


def done_round_robins(league):
    p = os.path.join(league, "games.csv")
    if not os.path.exists(p):
        return 0
    t = -1
    with open(p) as f:
        next(f, None)
        for line in f:
            t = max(t, int(line.split(",", 1)[0]))
    return t + 1


def atomic_write(path, text):
    tmp = path + ".tmp"
    with open(tmp, "w") as f:
        f.write(text)
    os.replace(tmp, path)


def league_note(cfg):
    """The cycles' rules for the league, for the header of its players.txt."""
    plies = " and ".join(map(str, cfg.tournament["entry_depths"]))
    q = cfg.tournament["qualify_depth"]
    text = (f"{NOTE_START[2:]} ({now()[:10]}), the training/tournament cycles under the champion rules "
            f"(cycles/champ/README.md) run this league. Each cycle plays fresh round-robins of the players listed here "
            f"(tool flag --no-catch-up): a newcomer plays from its first cycle on, and nobody catches up on earlier "
            f"round-robins. The fit is pure maximum likelihood with random at 0 (--prior-draws 0). A lineage's "
            f"entries are <lineage>-v<N>-<ply>: its current version at plies {plies}, and its champion at ply {q} "
            f"when that is another version. Players that leave the field move to retired.txt: they play no more, but "
            f"their games stay in games.csv and in the fit (--retired). The references never train. ")
    lines, line = [], "#"
    for word in text.split():
        if len(line) + 1 + len(word) > 118:
            lines.append(line)
            line = "#"
        line += " " + word
    return lines + [line, NOTE_END + " (it would catch players up and refit with the old prior)."]


def note_league(cfg):
    """Writes the cycles' rules into the header of the league's players.txt (replacing an older note), and makes
    retired.txt."""
    league = cfg.file("league_dir")
    if not os.path.exists(retired_path(cfg)):
        atomic_write(retired_path(cfg), RETIRED_HEADER)
    p = os.path.join(league, "players.txt")
    with open(p) as f:
        lines = f.readlines()
    start = next((i for i, l in enumerate(lines) if l.startswith(NOTE_START)), None)
    if start is not None:
        end = next(i for i in range(start, len(lines)) if lines[i].startswith(NOTE_END))
        undated = lambda ls: [re.sub(r" \(\d{4}-\d\d-\d\d\)", "", l.rstrip("\n")) for l in ls]
        if undated(lines[start:end + 1]) == undated(league_note(cfg)):
            return False  # the same rules
        del lines[start:end + 1]
    else:
        start = 0
        while start < len(lines) and lines[start].startswith("#"):
            start += 1
    lines[start:start] = [l + "\n" for l in league_note(cfg)]
    atomic_write(p, "".join(lines))
    return True


def move_rows(path, archive, chosen, key=None):
    """Moves the rows of a CSV file for which chosen(fields) holds to `archive`, never losing one: the archive (made
    with the same header if missing) is written before the file. `key` (fields -> key) keeps a rerun from archiving a
    row twice. Returns the number of rows moved."""
    with open(path) as f:
        header, *rows = [l if l.endswith("\n") else l + "\n" for l in f.read().splitlines()]
    split = lambda row: next(csv.reader([row]))
    out = [r for r in rows if chosen(split(r))]
    if not out:
        return 0
    have = []
    if os.path.exists(archive):
        with open(archive) as f:
            have = [l if l.endswith("\n") else l + "\n" for l in f.read().splitlines()][1:]
    seen = {key(split(r)) for r in have} if key else set()
    atomic_write(archive, header + "".join(have + [r for r in out if not key or key(split(r)) not in seen]))
    atomic_write(path, header + "".join(r for r in rows if not chosen(split(r))))
    return len(out)


def remove_players(cfg):
    """Archives the players each league_setup.remove rule picks (by ply or by model), in order, with their games
    (each game goes to the first archive whose players it involves) and, for whole models, their models.csv rows.
    Safe to repeat. Returns lines for the log."""
    league = cfg.file("league_dir")
    notes, players_path = [], os.path.join(league, "players.txt")
    for rule in cfg.league_setup.get("remove", []):
        with open(players_path) as f:
            lines = [l if l.endswith("\n") else l + "\n" for l in f.read().splitlines()]

        def picked(line):
            s = line.split()
            if not s or s[0].startswith("#"):
                return False
            return int(s[1]) in rule.get("depths", []) or model_of(s[0], int(s[1])) in rule.get("models", [])

        archive = cfg.abs(rule["players"])
        archived = {n for n, _, _ in read_entries(archive)}
        gone = [l for l in lines if picked(l)]
        if gone:
            with open(archive, "a") as f:
                if f.tell() == 0:
                    f.write(f"# Removed from the league by cycles/champ/cycle.py setup-league ({now()}): {rule['what']}. "
                            f"Their games are in {os.path.basename(rule['games'])}.\n")
                f.writelines(l for l in gone if l.split()[0] not in archived)
        names = archived | {l.split()[0] for l in gone}
        games = move_rows(os.path.join(league, "games.csv"), cfg.abs(rule["games"]),
                          lambda r: r[1] in names or r[2] in names, key=lambda r: tuple(r[:3]))
        rows = 0
        if "model_rows" in rule:
            rows = move_rows(os.path.join(league, "models.csv"), cfg.abs(rule["model_rows"]),
                             lambda r: r[0] in rule.get("models", []), key=lambda r: r[0])
        if gone:
            atomic_write(players_path, "".join(l for l in lines if not picked(l)))
        if gone or games or rows:
            notes.append(f"Removed {rule['what']}: {len(gone)} players to {os.path.basename(archive)}, {games} games "
                         f"to {os.path.basename(rule['games'])}"
                         + (f", {rows} models.csv rows to {os.path.basename(rule['model_rows'])}." if rows else "."))
    return notes


def setup_league(cfg):
    """Makes the league the cycles' (safe to repeat): copies it to league_setup.backup if that doesn't exist yet;
    archives the players each league_setup.remove rule picks (by ply or by model), in order, with their games (each
    game goes to the first archive whose players it involves) and, for whole models, their models.csv rows; adds
    league_setup.add_players; notes the rules in players.txt; refits. A league_dir without players.txt becomes a new
    league of random players. Returns lines for the log."""
    league, setup = cfg.file("league_dir"), cfg.league_setup
    why = tournament_unavailable(cfg)
    if why:
        sys.exit(f"can't set up the league: {why}")
    notes, players_path = [], os.path.join(league, "players.txt")
    if not os.path.exists(players_path):
        os.makedirs(league, exist_ok=True)
        atomic_write(players_path, f"# A league for the training/tournament cycles under the champion rules, made by "
                                   f"cycles/champ/cycle.py "
                                   f"({now()}): NAME DEPTH SPEC per player.\nrandom 0 rand\n")
        atomic_write(os.path.join(league, "models.csv"), MODELS_HEADER + "random,random,uniformly random moves "
                     "(searching on a uniform evaluation at ply 1 and up),-,random,0,-,-,-,-,no,-,-\n")
        notes.append(f"Made a new league in {cfg.rel(league)}.")
    busy = league_busy(cfg)
    if busy:
        sys.exit(f"the league is busy ({busy})")
    backup = cfg.abs(setup["backup"]) if setup.get("backup") else None
    if backup and not os.path.exists(backup):
        shutil.copytree(league, backup + ".tmp", symlinks=True)
        os.rename(backup + ".tmp", backup)
        notes.append(f"Copied the league to {cfg.rel(backup)} before changing it.")
    notes += remove_players(cfg)
    listed = {n for n, _, _ in read_players(league)}
    new = [l for l in cfg.league_setup.get("add_players", []) if l.split()[0] not in listed]
    if own_cal(cfg):  # a calibrated reference (ref-d4mix43-1) plays calibrated on its own games, like the entries
        new = [own_reference(cfg, l) for l in new]
    if new:
        with open(players_path) as f:
            text = f.read()
        atomic_write(players_path, text + ("" if text.endswith("\n") else "\n") + "".join(l + "\n" for l in new))
        notes.append(f"Added {', '.join(l.split()[0] for l in new)}.")
    if note_league(cfg):
        notes.append("Wrote the cycles' rules into the header of the league's players.txt.")
    done = done_round_robins(league)
    if done:
        secs = run_tournament(cfg, done)
        notes.append(f"Refitted the league's {done} round-robins (pure maximum likelihood, random at 0) in "
                     f"{secs:.0f} s.")
    return notes


def retired_path(cfg):
    return cfg.abs(cfg.tournament["retired"])


def update_league(cfg, retire, add_lines, add_models):
    """Moves the players in `retire` from players.txt to retired.txt (their games stay in games.csv), appends
    `add_lines` to players.txt and `add_models` rows to models.csv. Safe to repeat after an interruption."""
    league = cfg.file("league_dir")
    retire = set(retire)
    with open(os.path.join(league, "players.txt")) as f:
        lines = f.read().splitlines(keepends=True)
    kept, gone = [], []
    for line in lines:
        s = line.split()
        if s and not s[0].startswith("#") and s[0] in retire:
            gone.append(line if line.endswith("\n") else line + "\n")
        else:
            kept.append(line)
    if kept and not kept[-1].endswith("\n"):
        kept[-1] += "\n"
    listed = {l.split()[0] for l in kept if l.split() and not l.startswith("#")}
    kept += [l + "\n" for l in add_lines if l.split()[0] not in listed]  # already there if this is a rerun
    rp = retired_path(cfg)
    already = {n for n, _, _ in read_entries(rp)}
    with open(rp, "a") as f:  # first: a player is never in neither file
        if f.tell() == 0:
            f.write(RETIRED_HEADER)
        f.writelines(l for l in gone if l.split()[0] not in already)
    atomic_write(os.path.join(league, "players.txt"), "".join(kept))

    mpath = os.path.join(league, "models.csv")
    with open(mpath) as f:
        rows = list(csv.DictReader(f))
    fields = list(rows[0].keys())
    names = {r["name"] for r in rows}
    new = [{k: r.get(k, "") for k in fields} for r in add_models if r["name"] not in names]
    if new:
        buf = io.StringIO()
        w = csv.DictWriter(buf, fieldnames=fields, lineterminator="\n")
        w.writeheader()
        w.writerows(rows + new)
        atomic_write(mpath, buf.getvalue())
    return len(gone)


def tournament_unavailable(cfg):
    b = cfg.abs(cfg.tournament["binary"])
    if not os.path.exists(b):
        return f"{cfg.rel(b)} doesn't exist yet"
    lacks = [f for f in TOURNAMENT_FLAGS + list(cfg.tournament.get("flags", [])) if not supports(b, "tournament", f)]
    return f"{cfg.rel(b)} tournament lacks {', '.join(lacks)}" if lacks else None


def run_tournament(cfg, tournaments, save=None):
    """Plays round-robins of the current field (never catching anyone up) until `tournaments` are in games.csv,
    saving the games in `save` if given; then rewrites league.md. With `tournaments` already played, it plays nothing
    and only rewrites the reports."""
    league, t = cfg.file("league_dir"), cfg.tournament
    wait_league(cfg)
    why = tournament_unavailable(cfg)
    if why:
        sys.exit(f"can't run the cycles' tournament: {why}")
    note_league(cfg)
    if own_cal(cfg):  # made before the round-robins (own_field); here for any other call, such as refit
        own_games_for(cfg, field_own_keys(cfg), workers=cfg.jobs * cfg.threads_per_job)
    atomic_write(os.path.join(league, "pid.txt"), f"{os.getpid()}\n")
    cmd = ["nice", "-n", str(cfg.nice), cfg.abs(t["binary"]), "tournament", "players.txt", ".", "--tournaments",
           str(tournaments), "--use-probs", "--anchor", "random", "--seed", str(t["seed"]), "--no-catch-up",
           "--retired", retired_path(cfg), "--prior-draws", "0"] + list(t.get("flags", []))
    if save:
        os.makedirs(save, exist_ok=True)
        cmd += ["--save-games", save]
    t0 = time.time()
    with open(os.path.join(league, "tournament.log"), "a") as log:
        log.write(f"start {now()}: cycles/champ/cycle.py: {shlex.join(cmd)}\n")
        log.flush()
        r = subprocess.run(cmd, cwd=league, stdout=log, stderr=subprocess.STDOUT)
        if r.returncode != 0:
            log.write(f"tournament failed {now()}\n")
            sys.exit(f"tournament failed (exit {r.returncode}); see {league}/tournament.log")
        log.flush()
        subprocess.run([sys.executable, cfg.file("league_py"), league], stdout=log, stderr=subprocess.STDOUT)
        log.write(f"done {now()}\n")
    return time.time() - t0


# ---------------------------------------------------------------- saved games

def pool_dir(cfg, cycle):
    return os.path.join(cfg.abs(cfg.own_games["pools_dir"]), f"cycle-{cycle:03d}")


def saved_games(pool):
    """[(t, white, black, dir)] of the games `tournament --save-games` wrote: pool/round{t}/{white}~{black}/."""
    out = []
    if not pool or not os.path.isdir(pool):
        return out
    for r in os.listdir(pool):
        m = re.fullmatch(r"round(\d+)", r)
        if not m:
            continue
        for g in os.listdir(os.path.join(pool, r)):
            white, sep, black = g.partition("~")  # names can't contain ~ (the tool refuses them with --save-games)
            path = os.path.join(pool, r, g)
            if sep and os.path.isfile(os.path.join(path, OUTCOMES[0])):
                out.append((int(m.group(1)), white, black, path))
    return sorted(out)


def games_of(pool_games, entries, opponents):
    """The saved games in which any of `entries` played against one of `opponents`, each once."""
    entries, opponents = set(entries), set(opponents)
    return [g for g in pool_games if (g[1] in entries and g[2] in opponents) or (g[2] in entries and g[1] in opponents)]


def count_positions(game_dir):
    n = 0
    for name in OUTCOMES:
        with open(os.path.join(game_dir, name), "rb") as f:
            n += sum(1 for line in f if line.strip())
    return n


def own_unavailable(cfg):
    """Why the own-games step can't run, or None."""
    b = cfg.abs(cfg.own_games["binary"])
    if not os.path.exists(b):
        return f"{cfg.rel(b)} doesn't exist yet"
    if not supports(b, "train", "--start"):
        return f"{cfg.rel(b)} lacks train --start"
    return None


# ---------------------------------------------------------------- state

def load_state(cfg):
    p = cfg.file("state")
    if not os.path.exists(p):
        return None
    with open(p) as f:
        return json.load(f)


def save_state(cfg, state):
    atomic_write(cfg.file("state"), json.dumps(state, indent=1, sort_keys=True) + "\n")


def lineage_state(state, name):
    """A lineage's record in state.json (README.md, "Records"): status active, frozen or retired; version (its run's
    last), field_version and entries (its current version's entries in the field), champion (version, entry, since,
    elo, ci), patience_cpu (self-play CPU s since its last new champion), carry_cpu (budget carried to the next
    cycle)."""
    return state["lineages"].setdefault(name, dict(
        status="active", prepared=False, version=None, field_version=None, entries=None, prev_entries=None,
        champion=None, patience_cpu=0.0, carry_cpu=0.0, elo=None, ci=None, rel=None, elo_cycle=None))


def trim_side_file(path, start):
    """Drops from a copied ladder.csv or results.txt the rounds from `start` on, which the lineage plays itself."""
    name = os.path.basename(path)
    if name not in ("ladder.csv", "results.txt"):
        return
    with open(path) as f:
        lines = f.readlines()
    keep = []
    for i, line in enumerate(lines):
        m = re.match(r"(\d+)," if name == "ladder.csv" else r"round (\d+)\b", line)
        if (m and int(m.group(1)) >= start) or (name == "ladder.csv" and i > 0 and not m):
            continue
        keep.append(line if line.endswith("\n") else line + "\n")
    if keep != lines:
        atomic_write(path, "".join(keep))


def prepare(cfg, lin, st, players):
    """Makes the lineage's own run directory. Returns None, or the reason it can't train yet."""
    if st["prepared"]:
        return None
    run_dir, kind = lin["run_dir"], lin["kind"]
    if lin["source_type"] == "continue" and last_version(run_dir, kind):  # made already (in the first run, say):
        st["version"] = last_version(run_dir, kind)  # never rebuilt from its source
        st["prepared"] = True
        return None
    if lin["source_type"] == "continue":
        src = lin["source"]
        k = last_version(src, kind)
        if source_busy(src):
            k -= 1  # the newest file may still be being written: enter at the one before
        if k < (2 if kind == "moe" else 1):
            return f"no complete {kind} versions in {cfg.rel(src)} yet"
        if os.path.exists(run_dir):
            shutil.rmtree(run_dir)  # an unfinished earlier preparation, or init --force
        os.makedirs(run_dir)
        versioned = re.compile(re.escape(kind) + r"(\d+)\.(model|state)$")
        for f in sorted(os.listdir(src)):
            a, b = os.path.join(src, f), os.path.join(run_dir, f)
            m = versioned.match(f)
            if m:
                if 1 <= int(m.group(1)) <= k:
                    try:
                        os.link(a, b)  # never written again: self-play only writes newer versions
                    except OSError:
                        shutil.copy2(a, b)
            elif os.path.isfile(a):
                shutil.copy2(a, b)  # ladder.csv, results.txt, logs: self-play appends to some of them
                trim_side_file(b, k)
        if not os.path.exists(os.path.join(run_dir, "ladder.csv")):
            with open(os.path.join(run_dir, "ladder.csv"), "w") as f:
                f.write(LADDER_HEADER)  # --resume appends to it
        st["version"] = st["entry_version"] = st["field_version"] = k
        if st["entries"] is None:
            st["entries"] = [e[0] for e in match_entries(players, src)]
    else:
        os.makedirs(run_dir, exist_ok=True)
        st["version"] = st["entry_version"] = st["field_version"] = last_version(run_dir, kind)
        if st["entries"] is None:
            st["entries"] = []
    st["prepared"] = True
    return None


# ---------------------------------------------------------------- the champion rules

MILESTONES = dict(step=100, window=10, reference_window=50)


def milestones_cfg(cfg):
    """config.json's milestones over the defaults: a finishing champion is frozen at the multiple of `step` nearest
    its ply-2 Elo if within `window` and untaken; a reference takes the multiple nearest it within
    `reference_window`."""
    m = dict(MILESTONES)
    m.update(cfg.raw.get("milestones") or {})
    return m


def cycle_kind(cfg, state, cycle):
    """"seeding" (the first cycle: no training), "global" (every global_every cycles: the retired champions rejoin
    the round-robins) or "normal"."""
    if not state.get("seeded"):
        return "seeding"
    every = int(cfg.raw.get("global_every") or 0)
    return "global" if every > 0 and cycle % every == 0 else "normal"


def finite(r):
    """A rating (Elo, ±, games) that the fit determined."""
    return r is not None and math.isfinite(r[0])


def model_path(lin, v):
    return os.path.join(lin["run_dir"], f"{lin['kind']}{v}.model")


def budgets(total, carries):
    """The self-play budget split (the user, 2026-11-07): the cycle's `total` CPU s evenly among the lineages that
    self-play this cycle, each share plus the lineage's carry. carries: {name: carry}. Returns (share, {name:
    budget})."""
    share = total / len(carries) if carries else 0.0
    return share, {n: share + c for n, c in carries.items()}


def too_small(budget, cost, size):
    """A lineage skips this cycle's self-play when its budget is under half a round at its cost (the self-play tool
    plays whole rounds); the budget then carries over whole, so it plays next cycle with about twice its share."""
    return budget < 0.5 * cost * size


def settle(st, budget, used, failed=False):
    """A cycle's self-play accounting for a lineage: its patience counts the CPU it used; its carry is its budget
    minus that (the whole budget when it skipped the cycle; nothing after a failed job, which banks no budget)."""
    st["patience_cpu"] = round((st.get("patience_cpu") or 0.0) + used, 1)
    st["carry_cpu"] = 0.0 if failed else round(budget - used, 2)


def champion_test(cur, champ, se, k):
    """The champion test (the user, 2026-11-07): (new champion?, note). cur and champ: the current version's and the
    champion's ply-2 entries' (Elo, ±) in this fit; se: the standard error of their difference from the fit's
    covariance. The current version becomes the champion when it is above the champion by at least k of them."""
    diff = cur[0] - champ[0]
    z = diff / se if se > 0 else 0.0
    new = se > 0 and diff >= k * se
    return new, diff, z


def revival(games, entry, elo0, ci0, ratings):
    """The global tournament's revival statistic for a retired champion's entry (the user asked for actual minus
    expected score in standard errors; this is the exact one): over its games in this cycle's round-robins (games:
    [(white, black, result)]), its points S (draws half) minus the points E expected from its rating before them,
    elo0 (the previous cycle's fit), and each opponent's rating in this cycle's fit, over the standard error of S - E:
    the games' binomial variance V = sum p(1 - p) at those expectations (draws make the true variance smaller, so
    this errs on the side of not reviving), plus the variance of E from the uncertainty of elo0, (V ln10/400)^2
    (ci0/1.96)^2. Returns dict(games, score, expected, z)."""
    s_ = e_ = v_ = 0.0
    n = 0
    for white, black, result in games:
        if entry not in (white, black) or white == black:
            continue
        opp = black if white == entry else white
        if not finite(ratings.get(opp)):
            continue
        p = 1 / (1 + 10 ** ((ratings[opp][0] - elo0) / 400))
        s = {"1-0": 1.0, "0-1": 0.0}.get(result, 0.5)
        s_ += s if white == entry else 1 - s
        e_ += p
        v_ += p * (1 - p)
        n += 1
    var = v_ + (v_ * math.log(10) / 400 * ci0 / 1.96) ** 2
    z = (s_ - e_) / math.sqrt(var) if var > 0 else 0.0
    return dict(games=n, score=round(s_, 1), expected=round(e_, 2), z=round(z, 3))


def cycle_games(league, start, entries):
    """[(white, black, result)] of games.csv's round-robins from `start` on that involve any of `entries`."""
    entries, out = set(entries), []
    p = os.path.join(league, "games.csv")
    with open(p) as f:
        next(f, None)
        for line in f:
            t, white, black, result, _ = line.rstrip("\n").split(",", 4)
            if int(t) >= start and (white in entries or black in entries):
                out.append((white, black, result))
    return out


def taken_milestones(cfg, state, ratings):
    """{milestone: [holders]}: each frozen lineage's claim, and each reference player's nearest multiple within
    reference_window in this fit."""
    mc, taken = milestones_cfg(cfg), {}
    for ref in state.get("references") or []:
        if finite(ratings.get(ref)):
            m = nearest_milestone(ratings[ref][0], mc["step"])
            if abs(ratings[ref][0] - m) <= mc["reference_window"]:
                taken.setdefault(m, []).append(ref)
    for name, st in sorted(state["lineages"].items()):
        if st["status"] == "frozen" and st.get("claim") is not None:
            taken.setdefault(int(st["claim"]), []).append((st.get("champion") or {}).get("entry") or name)
    return taken


def link_or_copy(a, b):
    try:
        os.link(a, b)
    except OSError:
        shutil.copy2(a, b)


def same_file(a, b):
    if not (os.path.exists(a) and os.path.exists(b)):
        return False
    if os.path.samefile(a, b):
        return True
    with open(a, "rb") as x, open(b, "rb") as y:
        return x.read() == y.read()


def state_file(lin, v):
    """A tree's training state of version v: in its run directory, or kept in cycle-states/; None if neither."""
    for d in (lin["run_dir"], os.path.join(lin["run_dir"], "cycle-states")):
        p = os.path.join(d, f"{lin['kind']}{v}.state")
        if os.path.exists(p):
            return p
    return None


def keep_champion_state(lin, v):
    """A champion's training state (trees: {kind}{v}.state) is hard-linked into cycle-states/, where self-play never
    deletes it, so that a revival can resume from it."""
    if lin["kind"] not in TREE_KINDS:
        return
    a = os.path.join(lin["run_dir"], f"{lin['kind']}{v}.state")
    b = os.path.join(lin["run_dir"], "cycle-states", f"{lin['kind']}{v}.state")
    if os.path.exists(a) and not os.path.exists(b):
        os.makedirs(os.path.dirname(b), exist_ok=True)
        link_or_copy(a, b)


def copy_as_next(lin, v):
    """Training resumes from version v (a champion: the seeding's old peak, or a revived lineage's): its model file,
    and a tree's training state, become the run's next version, so that self-play's --resume and the own step's
    --start continue from it (the next round plays it against the run's last version, its predecessor by number).
    Safe to repeat: when the run's last version already is v or a copy of it, nothing is copied. Returns the version
    training resumes from, or raises OSError if a file is missing."""
    kind, run_dir = lin["kind"], lin["run_dir"]
    last = last_version(run_dir, kind)
    if last == v:
        return v
    src = model_path(lin, v)
    st_src = state_file(lin, v) if kind in TREE_KINDS else None
    if not os.path.exists(src) or (kind in TREE_KINDS and not st_src):
        raise OSError(f"{lin['name']}: v{v}'s {'state' if os.path.exists(src) else 'model'} file is missing")
    if same_file(model_path(lin, last), src) and (st_src is None or same_file(state_file(lin, last) or "", st_src)):
        return last  # copied before an interruption
    if kind in TREE_KINDS:  # a tree resumes from the states of its last two versions, in the run directory itself
        prev = state_file(lin, last)
        if not prev:
            raise OSError(f"{lin['name']}: v{last}'s state file is missing")
        if os.path.dirname(prev) != run_dir:
            link_or_copy(prev, os.path.join(run_dir, f"{kind}{last}.state"))
    n = last + 1
    if st_src:  # the state first: --resume scans the models
        link_or_copy(st_src, os.path.join(run_dir, f"{kind}{n}.state"))
    link_or_copy(src, model_path(lin, n))
    return n


def old_peak(ratings, depths, name, depth):
    """A lineage's best version at ply `depth` in the first run's league (its fit's ratings and plies, retired
    entries included): (version, entry, Elo, ±), or None. Ratings the fit left undetermined don't count."""
    pattern = re.compile(re.escape(name) + r"-v(\d+)-(\d+)")
    best = None
    for player, r in ratings.items():
        m = pattern.fullmatch(player)
        if m and depths.get(player) == depth and finite(r) and (best is None or r[0] > best[2]):
            best = (int(m.group(1)), player, r[0], r[1])
    return best


def match_stats(w, l, d):
    """Score, Elo and the 95% half-width of a match, as the arena computes them (selfplay::score_match)."""
    n = w + l + d
    s = (w + d / 2) / n
    var = (w * (1 - s) ** 2 + l * s ** 2 + d * (0.5 - s) ** 2) / n
    margin = 1.96 * math.sqrt(var / n)
    elo = lambda x: -400 * math.log10(1 / min(max(x, 1e-6), 1 - 1e-6) - 1)
    return s, elo(s), (elo(s + margin) - elo(s - margin)) / 2


# ---------------------------------------------------------------- the own-games step and its gate

def own_binary(cfg, lin):
    """The binary of a lineage's own step, its train and its gate: own_games.binary_by_lineage's entry for it, else
    own_games.binary (the user, 2026-11-07: T-L-Nc-fresh-c's train needs --revise-on, which only its own binary,
    chessai-resid4, has; every other lineage's step is unchanged)."""
    return cfg.abs(cfg.own_games.get("binary_by_lineage", {}).get(lin["name"]) or cfg.own_games["binary"])


LEAGUE_POLICY = dict(policy="log-odds-softmax", temperature=0.1)  # the league's: the tournament binary's defaults
# (tournament --help), which run_tournament leaves as they are


def league_policy_args():
    """The arena's flags that make both sides play as the league's entries do (LEAGUE_POLICY). Without them the
    arena plays its own default, wld-score-jitter10: self-play's policy, not the league's (the gate bug, fixed
    2026-11-08)."""
    p = LEAGUE_POLICY
    return ["--a-policy", p["policy"], "--b-policy", p["policy"], "--temperature", str(p["temperature"])]


def gate_cmd(cfg, lin, a, b, cycle):
    """The gate's arena: `a` against `b` at the gate's depth, games and seed, both in the league's policy."""
    g = cfg.gate
    return ["nice", "-n", str(cfg.nice), own_binary(cfg, lin), "arena", a, b,
            "--use-probs", "--games", str(g["games"]), "--a-depth", str(g["depth"]), "--b-depth", str(g["depth"]),
            "--seed", str(int(g["seed"]) + cycle), *league_policy_args()]


def gate_match(cfg, lin, post, pre, cycle, specs=None):
    """post vs pre at the gate's depth, calibrated like the league entries (`specs`, with calibration "own") and
    playing the league's policy. Returns (w, l, d, cpu) or (None, why)."""
    a, b = specs or (spec(lin, post), spec(lin, pre))
    cmd = gate_cmd(cfg, lin, a, b, cycle)
    code, cpu, _, out = run_measured(cmd, lin["run_dir"] + ".log", f"cycle {cycle} gate")
    m = re.search(r"^A: \+(\d+) -(\d+) =(\d+)", out, re.M)
    if code != 0 or not m:
        return None, f"gate failed (exit {code})"
    return (*map(int, m.groups()), cpu), None


def own_step(cfg, lin, st, games, cycle, row, own_stats=None):
    """Trains the lineage's newest version once on `games`; keeps the result as its next version if it scores at
    least the gate's threshold against the version it started from. With calibration "own", both sides of the gate
    are calibrated on their own games, as in the league: the trained model's are made first and deleted after the
    gate (a kept version is followed by self-play's, which enters the field; if it enters itself, its own games are
    made again, the same)."""
    name, kind, run_dir, n = lin["name"], lin["kind"], lin["run_dir"], st["version"]
    binary, args = own_binary(cfg, lin), model_args(lin)
    if not os.path.exists(binary):
        row["gate"] = f"step skipped: {cfg.rel(binary)} doesn't exist"
        print(f"  {name}: own step skipped: {cfg.rel(binary)} doesn't exist", flush=True)
        return
    if LABEL_FLAGS & set(args) and not supports(binary, "train", "--label-depth"):  # never without the recipe's labels
        row["gate"] = f"step skipped: {cfg.rel(binary)} lacks train --label-depth"
        print(f"  {name}: own step skipped: {cfg.rel(binary)} can't label like the recipe", flush=True)
        return
    work = os.path.join(cfg.file("runs_dir"), f".own-{name}")
    shutil.rmtree(work, ignore_errors=True)
    rnd = os.path.join(work, "games", "round0")
    os.makedirs(rnd)
    positions = 0
    for t, white, black, path in games:
        os.symlink(path, os.path.join(rnd, f"{t:03d}~{white}~{black}"))  # train reads linked game dirs in name order
        positions += count_positions(path)
    start = os.path.join(run_dir, f"{kind}{n}.{'state' if kind in TREE_KINDS else 'model'}")
    out = os.path.join(work, "out")
    cmd = ["nice", "-n", str(cfg.nice), binary, "train", os.path.join(work, "games"), out,
           "--model", kind, "--rounds", "1", "--seed", lin["seed"], "--start", start, *args]
    code, cpu, _, text = run_measured(cmd, run_dir + ".log", f"cycle {cycle} own games")
    m = re.search(r"^round 0: (\d+) positions", text, re.M)
    row.update(own_games=len(games), own_positions=int(m.group(1)) if m else positions, own_cpu_s=f"{cpu:.1f}")
    trained = os.path.join(out, f"{kind}1.model")
    if code != 0 or not os.path.exists(trained):
        last = (text.strip().splitlines() or [""])[-1][:120]
        row["gate"] = f"step failed (exit {code}: {last})"
        shutil.rmtree(work, ignore_errors=True)
        return
    specs, post_key = None, f".post-{name}"
    if own_cal(cfg):
        pre = os.path.join(run_dir, f"{kind}{n}.model")
        register_own(cfg, post_key, spec(lin, trained), trained)
        specs = (own_spec_of(cfg, post_key, kind, trained), league_spec(cfg, lin, n, pre))
        try:
            made, _, own_cpu = own_games_for(cfg, [post_key, f"{name}-v{n}"], threads=None)
        except OSError as e:
            row["gate"] = f"step failed ({e})"
            shutil.rmtree(work, ignore_errors=True)
            return
        if own_stats is not None:
            own_stats["made"] += made
            own_stats["cpu_s"] += own_cpu
    result, why = gate_match(cfg, lin, trained, os.path.join(run_dir, f"{kind}{n}.model"), cycle, specs)
    if specs:
        own_drop(cfg, post_key)
    if result is None:
        row["gate"] = why
        shutil.rmtree(work, ignore_errors=True)
        return
    w, l, d, gate_cpu = result
    s, elo, ci = match_stats(w, l, d)
    keep = s >= cfg.gate["threshold"]
    row.update(gate_wins=w, gate_losses=l, gate_draws=d, gate_score=f"{s:.4f}", gate_elo=f"{elo:.1f}",
               gate_ci95=f"{ci:.1f}", gate_cpu_s=f"{gate_cpu:.1f}", gate="kept" if keep else "reverted")
    files = [(trained, f"{kind}{n + 1}.model")]
    if os.path.exists(os.path.join(out, f"{kind}1.state")):
        files.append((os.path.join(out, f"{kind}1.state"), f"{kind}{n + 1}.state"))
    if keep:
        for a, b in reversed(files):  # the state first: --resume scans the models
            os.replace(a, os.path.join(run_dir, b))
        st["version"] = n + 1
    else:  # kept aside for the record; --resume only looks at the run directory itself
        os.makedirs(os.path.join(run_dir, "reverted"), exist_ok=True)
        for a, b in files:
            os.replace(a, os.path.join(run_dir, "reverted", f"cycle{cycle:03d}-{b}"))
    shutil.rmtree(work, ignore_errors=True)
    print(f"  {name}: own games {len(games)} ({row['own_positions']} positions, {cpu:.0f} s); gate +{w} -{l} ={d}, "
          f"{100 * s:.1f}% -> {row['gate']}", flush=True)


# ---------------------------------------------------------------- calibration on each version's own play (an option)

OWN_CAL_DEFAULTS = dict(dir="cycles/champ/owncal", seed_dirs=[], games=600, depth=1, seed=5,
                        policy="log-odds-softmax", temperature=0.1)
OWN_FILES = ("white_wins.games", "black_wins.games", "draws.games", "human.games")
POLICY_SHOWN = {"log-odds-softmax": "LogOddsSoftmax"}  # how the arena's first line names a policy


def own_cal(cfg):
    """True with config "calibration": "own" (off by default): every league entry and both sides of every gate are
    then calibrated on their own version's play, `cal:OWN:KIND:PATH` with OWN that version's own games (own_games_for),
    instead of on the lineage's held-out set ("heldout", the default: `cal:HELDOUT:KIND:PATH`)."""
    mode = cfg.raw.get("calibration", "heldout")
    if mode not in ("heldout", "own"):
        sys.exit(f"{cfg.path}: calibration is {mode!r}; it must be heldout or own")
    return mode == "own"


def own_cal_cfg(cfg):
    return {**OWN_CAL_DEFAULTS, **(cfg.raw.get("own_calibration") or {})}


def own_root(cfg):
    return cfg.abs(own_cal_cfg(cfg)["dir"])


def own_spec_of(cfg, key, kind, path):
    """The spec of a model calibrated on its own games, own_root/KEY (the tool fits the calibration as it loads)."""
    return f"cal:{os.path.join(own_root(cfg), key)}:{kind}:{path}"


def own_key_of(spec_text, root):
    """The own-games key of a spec calibrated on own games under root, else None."""
    prefix = f"cal:{root}/"
    return spec_text[len(prefix):].split(":", 1)[0] if spec_text.startswith(prefix) else None


def file_md5(path):
    h = hashlib.md5()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def register_own(cfg, key, play, path):
    """Records how version KEY's own games are made, once, in own_root/KEY.json: the spec they are played with (the
    version's held-out league spec, `play`), its model file and the file's md5. A record that no longer matches (the
    file changed) is replaced, and the games made from it are deleted."""
    root = own_root(cfg)
    os.makedirs(root, exist_ok=True)
    p = os.path.join(root, key + ".json")
    rec = dict(play=play, model=path, md5=file_md5(path))
    if os.path.exists(p):
        with open(p) as f:
            if json.load(f) == rec:
                return
        shutil.rmtree(os.path.join(root, key), ignore_errors=True)
    atomic_write(p, json.dumps(rec, indent=1) + "\n")


def league_spec(cfg, lin, v, path=None):
    """The spec a lineage's version plays with in the league: calibrated on the lineage's held-out set, or with
    calibration "own" on the version's own games (recorded here, made before the version plays)."""
    path = path or model_path(lin, v)
    if not own_cal(cfg):
        return spec(lin, path)
    key = f"{lin['name']}-v{v}"
    register_own(cfg, key, spec(lin, path), path)
    return own_spec_of(cfg, key, lin["kind"], path)


def own_games_cmd(cfg, play, out, binary=None):
    """The recipe: the version plays itself with its held-out league spec, at the league's policy (log-odds softmax at
    T 0.1), own_calibration.games games at ply own_calibration.depth, seed own_calibration.seed."""
    o = own_cal_cfg(cfg)
    return ["nice", "-n", str(cfg.nice), binary or cfg.abs(cfg.tournament["binary"]), "arena", play, play,
            "--use-probs", "--games", str(o["games"]), "--a-policy", o["policy"], "--b-policy", o["policy"],
            "--temperature", str(o["temperature"]), "--a-depth", str(o["depth"]), "--b-depth", str(o["depth"]),
            "--seed", str(o["seed"]), "--out", out]


def own_header(cfg, play):
    """The line the arena prints for these own games: a seed directory's games are reused only when its log has it."""
    o = own_cal_cfg(cfg)
    shown = POLICY_SHOWN.get(o["policy"])
    side = f"{play} ({shown}, depth {o['depth']})"
    return f"arena: A = {side}, B = {side}, {o['games']} games, seed {o['seed']}" if shown else None


def seed_match(cfg, key, play):
    """A seed directory's own games of KEY made by this recipe (own_calibration.seed_dirs, in order; each KEY/ with
    its KEY.arena log, as models/from-scratch/leaguecal/games/ has them): its path, or None. The log's first line
    must show the same spec, policy, ply, number of games and seed (not the temperature: a seed directory must have
    been made at own_calibration.temperature), and the model file must be older than the log."""
    head, model = own_header(cfg, play), play.rsplit(":", 1)[-1]
    for sd in own_cal_cfg(cfg)["seed_dirs"]:
        src = os.path.join(cfg.abs(sd), key)
        log = src + ".arena"
        if not (head and os.path.exists(log) and all(os.path.exists(os.path.join(src, f)) for f in OWN_FILES)):
            continue
        if os.path.getmtime(model) > os.path.getmtime(log):  # rewritten since: its games may not be this file's
            continue
        with open(log) as f:
            text = f.read()
        if head in text.splitlines() and re.search(r"^A: \+\d+ -\d+ =\d+", text, re.M):
            return src
    return None


def make_own(cfg, key, rec, threads=1):
    """Version KEY's own games, into own_root/KEY: linked from a seed directory when one has them, else played.
    Returns ("reused" or "made", CPU s), or raises OSError."""
    root = own_root(cfg)
    final, tmp = os.path.join(root, key), os.path.join(root, f".tmp-{key}")
    shutil.rmtree(tmp, ignore_errors=True)
    src = seed_match(cfg, key, rec["play"])
    if src:
        os.makedirs(tmp)
        for f in OWN_FILES:
            link_or_copy(os.path.join(src, f), os.path.join(tmp, f))
        shutil.copy2(src + ".arena", final + ".arena")
        os.rename(tmp, final)
        return "reused", 0.0
    cmd = own_games_cmd(cfg, rec["play"], tmp)
    env = dict(os.environ, RAYON_NUM_THREADS=str(threads)) if threads else None
    proc = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, env=env)
    out = proc.stdout.read()
    _, status, usage = os.wait4(proc.pid, 0)
    code = os.waitstatus_to_exitcode(status)
    atomic_write(final + ".arena", f"# {now()}: {shlex.join(cmd)}\n{out}")
    if code != 0 or not all(os.path.exists(os.path.join(tmp, f)) for f in OWN_FILES):
        shutil.rmtree(tmp, ignore_errors=True)
        raise OSError(f"own games of {key} failed (exit {code}); see {cfg.rel(final)}.arena")
    os.rename(tmp, final)
    return "made", usage.ru_utime + usage.ru_stime


def own_games_for(cfg, keys, workers=1, threads=1):
    """Makes the own games of each of `keys` that lacks them, from its KEY.json, `workers` at a time with `threads`
    threads each (None: the default). The games are seeded, so a version's are the same whenever they are made, and
    any thread count gives the same. Returns (made, reused, CPU s)."""
    root = own_root(cfg)
    todo = []
    for key in sorted(set(keys)):
        with open(os.path.join(root, key + ".json")) as f:
            rec = json.load(f)
        if os.path.isdir(os.path.join(root, key)):
            if file_md5(rec["model"]) == rec["md5"]:
                continue
            sys.exit(f"{rec['model']} changed since its own games were made (md5 {rec['md5']})")
        todo.append((key, rec))
    if not todo:
        return 0, 0, 0.0
    with ThreadPoolExecutor(max(1, workers)) as ex:
        done = list(ex.map(lambda kr: make_own(cfg, kr[0], kr[1], threads), todo))
    return (sum(1 for how, _ in done if how == "made"), sum(1 for how, _ in done if how == "reused"),
            sum(cpu for _, cpu in done))


def field_own_keys(cfg):
    """The own-games keys of the players listed in players.txt (the field; retired players' models aren't loaded)."""
    root = own_root(cfg)
    return [k for k in (own_key_of(s, root) for _, _, s in read_players(cfg.file("league_dir"))) if k]


def own_gc(cfg):
    """Deletes the own games of versions not in the field (and any left half-made): they come back the same if a
    version returns (a retired champion at a global tournament). The KEY.json and KEY.arena records stay. Returns
    the number of directories deleted."""
    root = own_root(cfg)
    if not os.path.isdir(root):
        return 0
    keep, n = set(field_own_keys(cfg)), 0
    for name in sorted(os.listdir(root)):
        p = os.path.join(root, name)
        if os.path.isdir(p) and name not in keep:
            shutil.rmtree(p)
            n += 1
    return n


def own_reference(cfg, line):
    """A league_setup.add_players line with calibration "own": a calibrated model's spec (cal:HELDOUT:KIND:PATH)
    becomes its own-games spec, keyed by the player's model name (ref-d4mix43 for ref-d4mix43-1)."""
    name, depth, s = line.split()
    parts = s.split(":")
    if parts[0] != "cal" or len(parts) != 4:
        return line
    key = model_of(name, int(depth))
    register_own(cfg, key, s, parts[3])
    return f"{name} {depth} {own_spec_of(cfg, key, parts[2], parts[3])}"


def own_dry_lines(cfg, lineages, plan):
    """The dry run's account of calibration "own" for the seeding field: each version's own games, found in a seed
    directory (made by this recipe) or to be made. Reads only."""
    o, by = own_cal_cfg(cfg), {l["name"]: l for l in lineages}
    found, make = [], []
    for p in plan:
        lin = by[p["name"]]
        for v in sorted({v for v in (p["latest"], p["peak"]) if v}):  # 0: no version yet, not in the field
            key = f"{lin['name']}-v{v}"
            (found if seed_match(cfg, key, spec(lin, model_path(lin, v))) else make).append(key)
    refs = [l for l in cfg.league_setup.get("add_players", []) if l.split()[2].startswith("cal:")]
    return [f"Calibration \"own\": each version's {o['games']} own games at ply {o['depth']} (seed {o['seed']}), in "
            f"{cfg.rel(own_root(cfg))}. The seeding field's {len(found) + len(make)} versions: {len(found)} found in "
            f"{', '.join(cfg.rel(cfg.abs(s)) for s in o['seed_dirs']) or 'no seed directory'} (made by this recipe), "
            f"{len(make)} to make (1-3 CPU s each), plus {len(refs)} calibrated reference(s).",
            *(f"  to make: {k}" for k in make)]


def own_drop(cfg, key):
    """Deletes KEY's own games and their records."""
    root = own_root(cfg)
    shutil.rmtree(os.path.join(root, key), ignore_errors=True)
    for ext in (".json", ".arena"):
        if os.path.exists(os.path.join(root, key + ext)):
            os.remove(os.path.join(root, key + ext))


def own_disk_mb(cfg):
    """MB of own games in own_root, counting files linked from a seed directory once more (their space is shared)."""
    root, total = own_root(cfg), 0
    for d, _, files in os.walk(root):
        total += sum(os.path.getsize(os.path.join(d, f)) for f in files)
    return total / 1e6


def own_field(cfg):
    """Before the round-robins, with calibration "own": deletes the own games of versions that left the field, then
    makes those of the field that lack them, jobs x threads_per_job at a time on one thread each. Returns the
    numbers for the cycle's notes."""
    deleted = own_gc(cfg)
    keys = field_own_keys(cfg)
    t0 = time.time()
    made, reused, cpu = own_games_for(cfg, keys, workers=cfg.jobs * cfg.threads_per_job, threads=1)
    return dict(field=len(set(keys)), made=made, reused=reused, cpu_s=round(cpu, 1), wall_s=round(time.time() - t0, 1),
                deleted=deleted, mb=round(own_disk_mb(cfg)))


def own_opponents(cfg, state, field):
    """Whose games against a lineage's current entries its own step trains on: everyone who played this cycle's
    round-robins (`field`, recorded as they began) except the references, so that games against champions (its own,
    the other lineages', the frozen ones and, in a global cycle, the retired ones) count and games against the random
    players (and any other reference) only measure; with own_games.opponents "all", the references too."""
    if cfg.own_games.get("opponents", "lineages") == "all":
        return set(field)
    return set(field) - set(state.get("references") or [])


def own_steps(cfg, state, lineages, pool, cycle, rows, players, done, opponents, own_stats=None):
    """The own-games step for every active lineage with the switch on whose current entries played in this
    tournament, except those in `done` (this cycle's, before an interruption). A lineage that finished this cycle, or
    whose champion is a copy waiting to train (seeding, revival), has no current entries and takes no step. Nor does
    a solo lineage (steps - in the registry, the switch on for the others): its row's gate says SOLO instead."""
    pool_games = saved_games(pool)
    for lin in lineages:
        st, row = lineage_state(state, lin["name"]), rows[lin["name"]]
        if lin["name"] in done or st["status"] != "active" or not st["entries"]:
            continue
        if not own_on(cfg, lin):
            if cfg.own_games.get("enabled", True) and not lin["steps"]:
                row["gate"] = SOLO
            continue
        games = games_of(pool_games, st["entries"], opponents)
        if not games:
            continue
        why = prepare(cfg, lin, st, players)
        if why:
            row["gate"] = f"step skipped: {why}"
        else:
            own_step(cfg, lin, st, games, cycle, row, own_stats)
        done.append(lin["name"])
        save_state(cfg, state)


# ---------------------------------------------------------------- logs

def append_log(cfg, rows):
    p = cfg.file("log_csv")
    if os.path.exists(p):
        with open(p, newline="") as f:
            head = next(csv.reader(f), None)
        if head and head != LOG_FIELDS:  # new columns: rewrite the file once with them, keeping every row
            with open(p, newline="") as f:
                old = list(csv.DictReader(f))
            buf = io.StringIO()
            w = csv.DictWriter(buf, fieldnames=LOG_FIELDS, lineterminator="\n", extrasaction="ignore")
            w.writeheader()
            w.writerows({k: r.get(k) or "" for k in LOG_FIELDS} for r in old)
            atomic_write(p, buf.getvalue())
    new = not os.path.exists(p)
    with open(p, "a", newline="") as f:
        w = csv.DictWriter(f, fieldnames=LOG_FIELDS, lineterminator="\n")
        if new:
            w.writeheader()
        for r in rows:
            w.writerow({k: "" if r.get(k) is None else r.get(k) for k in LOG_FIELDS})


def logged_cycles(cfg):
    p = cfg.file("log_csv")
    if not os.path.exists(p):
        return set()
    with open(p) as f:
        return {int(r["cycle"]) for r in csv.DictReader(f)}


def pooled(rows):
    """The gates' pooled result: (gates, kept, w, l, d) or None."""
    gs = [r for r in rows if str(r.get("gate_wins", "")) != ""]
    if not gs:
        return None
    total = lambda k: sum(int(r[k]) for r in gs)
    return len(gs), sum(r["gate"] == "kept" for r in gs), total("gate_wins"), total("gate_losses"), \
        total("gate_draws")


def pooled_text(p):
    n, kept, w, l, d = p
    s, elo, ci = match_stats(w, l, d)
    return (f"{n} gate{'s' if n != 1 else ''}, +{w} -{l} ={d}, score {100 * s:.1f}%, Elo {elo:+.1f} ± {ci:.1f}; "
            f"kept {kept} of {n}")


def gap_words(diff):
    """A difference of ratings in words (no plus signs on gaps)."""
    if diff in (None, ""):
        return ""
    d = float(diff)
    return f"{abs(d):.0f} {'above' if d >= 0 else 'below'}"


def write_summary(cfg, cycle, kind, rows, extra, rel_refs=None):
    def fmt(x, f="{:.0f}"):
        """x formatted, "" for none; never "-0" (a carry of -0.3 shows as 0)."""
        if x in (None, ""):
            return ""
        t = f.format(float(x))
        return t[1:] if re.fullmatch(r"-0(\.0*)?", t) else t
    title = {"seeding": " (seeding)", "global": " (global tournament)"}.get(kind, "")
    lines = [f"# Cycle {cycle}{title}", ""]
    p = pooled(rows)
    if p:
        lines.append(f"**Own-games step, pooled over the lineages: {pooled_text(p)}.**")
        with open(cfg.file("log_csv")) as f:
            everything = pooled(list(csv.DictReader(f)))  # this cycle's rows are already in it
        lines += ["", f"All cycles so far: {pooled_text(everything)}."]
    elif kind != "seeding":
        lines.append("**Own-games step: no gates this cycle.**")
    k = float(cfg.raw.get("champion_se", 2.0))
    lines += ["", *extra, "",
              f"Elo: each entry's in this fit (pure maximum likelihood, random at 0); vs refs: Elo minus the mean of "
              f"{', '.join(rel_refs or [])} in this fit. The champion test compares the current version's ply-"
              f"{cfg.tournament['qualify_depth']} entry with the champion's: it becomes the champion when it is at "
              f"least {k:g} standard errors of their difference above it (from the fit's covariance). Patience: "
              f"self-play CPU s since the lineage's last new champion, of {cfg.raw.get('patience_cpu_s', 0):g}; at "
              "that the lineage finishes, its champion frozen at an untaken milestone within "
              f"{milestones_cfg(cfg)['window']:g} of its ply-2 Elo, else retired.", "",
              "| lineage | status | version | self-play | CPU s | budget | carry | patience | Elo | ± | champion | "
              "champion Elo | current vs champion | decision | note | own games (positions) | gate | next |",
              "|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---|---|---|---|---|---:|"]
    for r in rows:
        sp = f"{r['rounds']} x {r['round_size']}" if r.get("rounds") not in (None, "") else ""
        own = f"{r['own_games']} ({r['own_positions']})" if r.get("own_games") not in (None, "") else ""
        gate = r.get("gate") or ""
        if str(r.get("gate_wins", "")) not in ("", "None"):
            gate = (f"+{r['gate_wins']} -{r['gate_losses']} ={r['gate_draws']}, {100 * float(r['gate_score']):.1f}%, "
                    f"{float(r['gate_elo']):+.0f} ± {fmt(r['gate_ci95'])}: {r['gate']}")
        vs = (f"{gap_words(r['diff'])}, {float(r['diff']) / float(r['diff_se']):.2f} SE"
              if r.get("diff") not in (None, "") and r.get("diff_se") not in (None, "") and float(r["diff_se"]) > 0
              else "")
        if r.get("revive_z") not in (None, ""):
            vs = (f"{r['revive_score']} of {r['revive_games']} vs {r['revive_expected']} expected, "
                  f"{float(r['revive_z']):.2f} SE")
        champ = f"v{r['champion_version']}" if r.get("champion_version") not in (None, "") else ""
        lines.append(f"| {r['lineage']} | {r.get('status', '')} | {r.get('version') or ''} | {sp} "
                     f"| {fmt(r.get('cpu_s'))} | {fmt(r.get('budget_cpu_s'))} | {fmt(r.get('carry_cpu_s'))} "
                     f"| {fmt(r.get('patience_cpu_s'))} | {fmt(r.get('elo'))} | {fmt(r.get('ci95'))} | {champ} "
                     f"| {fmt(r.get('champion_elo'))} | {vs} | {r.get('decision', '')} | {r.get('note') or ''} "
                     f"| {own} | {gate} | {r.get('next_version') or ''} |")
    atomic_write(os.path.join(cfg.file("log_dir"), f"cycle-{cycle:03d}.md"), "\n".join(lines) + "\n")


def append_totals(cfg, row):
    """totals.csv: one row per cycle (TOTALS_FIELDS), written once."""
    p = cfg.file("totals_csv")
    if os.path.exists(p):
        with open(p, newline="") as f:
            if any(r["cycle"] == str(row["cycle"]) for r in csv.DictReader(f)):
                return
    new = not os.path.exists(p)
    with open(p, "a", newline="") as f:
        w = csv.DictWriter(f, fieldnames=TOTALS_FIELDS, lineterminator="\n")
        if new:
            w.writeheader()
        w.writerow({k: "" if row.get(k) is None else row.get(k) for k in TOTALS_FIELDS})


def nearest_milestone(x, step):
    return int(math.floor(x / step + 0.5)) * step


def resolve_claims(groups, taken, step, window):
    """The rule's outcomes. groups: lists of (name, {ply: average Elo}), in the order they come due (the lineages due
    at the same decision form one group); taken: the milestones already taken (claims, references), updated here.
    Within a group, closest first (a tie: ply 2 first, then by name): a lineage freezes at a ply whose average is
    within `window` of its nearest milestone and that milestone is untaken, so one that loses its milestone to a
    closer lineage falls back to its other ply; one with neither retires. {name: (ply, milestone, average) or None}"""
    out = {}
    for group in groups:
        opts = []
        for name, avg in group:
            out[name] = None
            for ply, a in avg.items():
                m = nearest_milestone(a, step)
                if abs(a - m) <= window:
                    opts.append((abs(a - m), -int(ply), name, int(ply), m, a))
        for _, _, name, ply, m, a in sorted(opts):
            if out[name] is None and m not in taken:
                out[name] = (ply, m, a)
                taken.add(m)
    return out


# ---------------------------------------------------------------- commands

def lock(cfg):
    f = open(cfg.file("state") + ".lock", "w")
    try:
        fcntl.flock(f, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except OSError:
        sys.exit("another cycle.py holds the lock")
    return f


def registry_names(cfg):
    return [l["name"] for l in load_registry(cfg)]


def unretire_players(cfg, names):
    """Moves these players back from retired.txt to the league's players.txt (players.txt first: a player is never
    in neither file). Returns how many moved."""
    league, rp = cfg.file("league_dir"), retired_path(cfg)
    with open(rp) as f:
        lines = f.read().splitlines(keepends=True)
    back = [l if l.endswith("\n") else l + "\n" for l in lines if l.split() and not l.startswith("#")
            and l.split()[0] in names]
    with open(os.path.join(league, "players.txt")) as f:
        text = f.read()
    listed = {l.split()[0] for l in text.splitlines() if l.split() and not l.startswith("#")}
    add = [l for l in back if l.split()[0] not in listed]
    if add:
        atomic_write(os.path.join(league, "players.txt"), text + ("" if text.endswith("\n") or not text else "\n")
                     + "".join(add))
    atomic_write(rp, "".join(l for l in lines if not (l.split() and not l.startswith("#") and l.split()[0] in names)))
    return len(back)


def between_cycles(cfg):
    """The lock and the state, for a command that runs between cycles only."""
    held = lock(cfg)
    state = load_state(cfg)
    if not state or state.get("current"):
        sys.exit("no state, or a cycle is under way: between cycles only (touch STOP, wait for the loop to stop)")
    return held, state


def field_entries(st):
    """A lineage's entries in the field: its current version's and its champion's (once, if the same)."""
    ch = (st.get("champion") or {}).get("entry")
    if st["status"] == "retired":
        return []
    if st["status"] == "frozen":
        return [ch] if ch else list(st["entries"] or [])
    return list(st["entries"] or []) + ([ch] if ch and ch not in (st["entries"] or []) else [])


def cmd_freeze(cfg, name, unfreeze=False):
    """freeze NAME (by hand): an active lineage stops training and its champion stays in the field as a fixed
    player (no milestone: frozen_by "hand"); its current version's entries leave the field (retired.txt: their games
    stay in the fit). unfreeze NAME: a frozen lineage trains again from its latest version, with a fresh patience (a
    lineage the rules froze had used all of its own); a milestone it held is free again. Between cycles only."""
    held, state = between_cycles(cfg)
    if name not in registry_names(cfg) or name not in state["lineages"]:
        sys.exit(f"{name} is not a lineage in lineages.txt and state.json")
    st = state["lineages"][name]
    if not unfreeze:
        if st["status"] != "active" or not st.get("champion"):
            sys.exit(f"{name} is {st['status']}" + ("" if st.get("champion") else ", with no champion yet"))
        ch = st["champion"]["entry"]
        other = [e for e in st["entries"] or [] if e != ch]
        moved = update_league(cfg, other, [], [])
        st.update(status="frozen", frozen_by="hand", frozen_cycle=state["cycle"], entries=[])
        print(f"{name}: frozen after cycle {state['cycle']}: champion {ch} stays in the field; {moved} entries "
              f"retired ({' '.join(other) or 'none'})")
    else:
        if st["status"] != "frozen":
            sys.exit(f"{name} is not frozen ({st['status']})")
        for k in ("frozen_by", "frozen_cycle", "claim"):
            st.pop(k, None)
        st.update(status="active", patience_cpu=0.0, carry_cpu=0.0)
        st["field_version"] = st["version"]  # its next self-play's version enters; the champion stays
        print(f"{name}: unfrozen after cycle {state['cycle']}: trains again from v{st['version']}; champion "
              f"{st['champion']['entry']} stays")
    taken = taken_milestones(cfg, state, read_ratings(cfg.file("league_dir")))
    state["milestones"] = {str(m): ", ".join(t) for m, t in sorted(taken.items())}
    save_state(cfg, state)
    return held


def cmd_retire(cfg, name):
    """retire NAME (by hand), after taking its line out of lineages.txt (or commenting it out): its entries leave
    the field (retired.txt: their games stay in the fit), and it never rejoins a global tournament (retired_by
    "hand"). Between cycles only."""
    held, state = between_cycles(cfg)
    if name in registry_names(cfg):
        sys.exit(f"{name} is still in lineages.txt: take its line out first")
    st = state["lineages"].get(name)
    if not st:
        sys.exit(f"no lineage {name} in state.json")
    listed = [n for n, _, _ in read_players(cfg.file("league_dir")) if n.startswith(name + "-v")]
    entries = sorted(set(field_entries(st)) | set(listed))
    moved = update_league(cfg, entries, [], [])
    if st["status"] != "retired" or st.get("retired_by") != "hand":
        st.update(retired_from=st["status"], status="retired", retired_by="hand", retired_cycle=state["cycle"],
                  entries=[])
    save_state(cfg, state)
    print(f"{name}: retired by hand after cycle {state['cycle']}; {moved} entries moved to retired.txt "
          f"({' '.join(entries) or 'none'}); its games stay in the fit")
    return held


def cmd_unretire(cfg, name):
    """unretire NAME: a retired lineage (by hand or by the rules) trains again from its latest version, with a fresh
    patience, its champion back in the field; its line must be in lineages.txt. Between cycles only."""
    held, state = between_cycles(cfg)
    st = state["lineages"].get(name)
    if not st or st["status"] != "retired":
        sys.exit(f"{name} is not a retired lineage in state.json" + (f" ({st['status']})" if st else ""))
    if name not in registry_names(cfg):
        sys.exit(f"{name} is not in lineages.txt: put its line back first")
    back = [st["champion"]["entry"]] if st.get("champion") else []
    moved = unretire_players(cfg, back)
    for k in ("retired_from", "retired_by", "retired_cycle"):
        st.pop(k, None)
    st.update(status="active", entries=[], field_version=st["version"], patience_cpu=0.0, carry_cpu=0.0)
    save_state(cfg, state)
    print(f"{name}: unretired after cycle {state['cycle']}: trains again from v{st['version']} ({moved} back from "
          f"retired.txt)")
    return held


def cmd_status(cfg):
    lineages, costs, state = load_registry(cfg), load_costs(cfg), load_state(cfg)
    t = cfg.tournament
    state = state or {"cycle": 0, "lineages": {}}
    active = [l for l in lineages if lineage_state(state, l["name"])["status"] == "active"]
    share = cfg.cycle_budget_cpu_s / max(1, len(active))
    print(f"cycle {state['cycle']} ({'seeded' if state.get('seeded') else 'next: the seeding cycle'}); self-play "
          f"{cfg.cycle_budget_cpu_s} CPU s a cycle, {share:.0f} each for {len(active)} active lineages; patience "
          f"{cfg.raw.get('patience_cpu_s')} CPU s; champion test {cfg.raw.get('champion_se')} SE; global tournament "
          f"every {cfg.raw.get('global_every')} cycles")
    print(f"league {cfg.league_dir}; {t['round_robins_per_cycle']} round-robins per cycle: "
          f"{tournament_unavailable(cfg) or 'ready'}; own-games step: {own_unavailable(cfg) or 'ready'}")
    if own_cal(cfg):
        print(f"calibration: own (each version on its own games, in {cfg.rel(own_root(cfg))})")
    print(f"{'lineage':24} {'status':8} {'ver':>5} {'champion':>9} {'elo':>6} {'patience':>8} {'carry':>6}  next")
    for lin in lineages:
        st = lineage_state(state, lin["name"])
        ch = st.get("champion") or {}
        nxt = ""
        if st["status"] == "active":
            p, why = plan(cfg, costs, lin, st, share + st.get("carry_cpu", 0.0))
            nxt = (f"~{p['rounds']} x {p['size']} games ({p['cost'] * 1000:.2f} ms/game {p['cost_source']})" if p
                   else why)
        elif st["status"] == "frozen":
            nxt = f"frozen ({st.get('frozen_by')}" + (f", milestone {st['claim']})" if st.get("claim") is not None
                                                     else ")")
        else:
            nxt = f"retired ({st.get('retired_by')})"
        ver = st["version"] if st["version"] is not None else "-"
        elo = f"{ch['elo']:.0f}" if ch.get("elo") is not None else "-"
        print(f"{lin['name']:24} {st['status']:8} {ver:>5} {('v' + str(ch['version'])) if ch else '-':>9} {elo:>6} "
              f"{st.get('patience_cpu', 0):8.0f} {st.get('carry_cpu', 0):6.0f}  {nxt}")
    if state.get("milestones"):
        print("milestones taken: " + "; ".join(f"{m} {t}" for m, t in sorted(state["milestones"].items(),
                                                                              key=lambda kv: int(kv[0]))))


def measure_one(cfg, sig, binary, src, kind, tokens, depth, threads):
    m = cfg.measure
    games = int(m["games_by_kind"].get(kind, m["games_by_depth"][str(depth)]))
    rounds = int(m["rounds"])
    tmp = os.path.join(cfg.abs(m["dir"]), re.sub(r"[^A-Za-z0-9.-]+", "_", sig)[:80])
    shutil.rmtree(tmp, ignore_errors=True)
    os.makedirs(tmp)
    models = "scratch"
    if src:
        k = last_version(src, kind)
        shutil.copy2(os.path.join(src, f"{kind}{k - 1}.model"), os.path.join(tmp, f"{kind}1.model"))
        shutil.copy2(os.path.join(src, f"{kind}{k}.model"), os.path.join(tmp, f"{kind}2.model"))
        with open(os.path.join(tmp, "ladder.csv"), "w") as f:
            f.write(LADDER_HEADER)
        extra, models = ["--resume", "--rounds", str(2 + rounds)], f"{cfg.rel(src)} v{k - 1},v{k}"
    else:
        extra = ["--rounds", str(rounds)]
    cmd = ["nice", "-n", str(cfg.nice), binary, "self-play", tmp, *tokens, "--games", str(games),
           "--threads", str(threads), *extra]
    load = os.getloadavg()[0]
    code, cpu, wall, out = run_measured(cmd, os.path.join(cfg.abs(m["dir"]), "measure.log"), f"measure {sig}")
    played = len(re.findall(r"^round \d+: \+", out, re.M))
    if code != 0 or played != rounds:
        sys.exit(f"measuring '{sig}' failed:\n{out[-1500:]}")
    shutil.rmtree(tmp)
    return dict(signature=sig, cpu_per_game=f"{cpu / (games * rounds):.6f}", games=games, rounds=rounds,
                threads=threads, cpu_s=f"{cpu:.1f}", wall_s=f"{wall:.1f}", binary=cfg.rel(binary), models=models,
                date=now(), loadavg=f"{(load + os.getloadavg()[0]) / 2:.1f}")


def cmd_measure(cfg, force):
    lineages, costs = load_registry(cfg), load_costs(cfg)
    m = cfg.measure
    os.makedirs(cfg.abs(m["dir"]), exist_ok=True)
    todo = {}
    ref_lr = cfg.abs(m["reference_runs"]["lr"])
    for d in rule_depths(lineages):  # the round-size rule
        tokens = ["--model", "lr", "--use-probs", "--seed", "21", "--vs-previous", "--no-games"] + (
            ["--depth", str(d)] if d else [])
        todo[f"lr d{d} jitter10"] = (cfg.abs(m["round_rule_binary"]), ref_lr, "lr", tokens, d)
    for lin in lineages:
        if not os.path.exists(lin["binary"]):
            continue
        src = None
        if lin["source_type"] == "continue" and not source_busy(lin["source"]):
            src = lin["source"]
        elif lin["kind"] in m["reference_runs"]:
            src = cfg.abs(m["reference_runs"][lin["kind"]])
        entry = (lin["binary"], src, lin["kind"], list(lin["tokens"]), lin["depth"])
        if src and src == lin["source"]:
            todo[lin["sig"]] = entry  # the lineage's own models beat a reference run's
        else:
            todo.setdefault(lin["sig"], entry)
    p = cfg.file("costs")
    for sig, (binary, src, kind, tokens, depth) in todo.items():
        if sig in costs and not force:
            continue
        print(f"measuring {sig} ...", flush=True)
        row = measure_one(cfg, sig, binary, src, kind, tokens, depth, cfg.threads_per_job)
        new = not os.path.exists(p)
        with open(p, "a", newline="") as f:
            w = csv.DictWriter(f, fieldnames=COST_FIELDS, lineterminator="\n")
            if new:
                w.writeheader()
            w.writerow(row)
        print(f"  {float(row['cpu_per_game']) * 1000:.2f} ms CPU per game ({row['cpu_s']} s for {row['rounds']} x "
              f"{row['games']} games, {row['wall_s']} s wall, load {row['loadavg']})", flush=True)


def cmd_refit(cfg):
    lk = lock(cfg)
    busy = league_busy(cfg)
    if busy:
        sys.exit(f"the league is busy ({busy})")
    done = done_round_robins(cfg.file("league_dir"))
    secs = run_tournament(cfg, done)
    print(f"rewrote the league's reports for its {done} round-robins with the cycles' fit in {secs:.0f} s")
    lk.close()


def cmd_setup(cfg):
    lk = lock(cfg)
    state = load_state(cfg)
    if state and state.get("current"):
        sys.exit(f"cycle {state['current']['cycle']} is under way: finish it first (run)")
    for line in setup_league(cfg):
        print(line)
    lk.close()


def old_league(cfg):
    """The first run's league (seed_from.league_dir): its fit's ratings and every player's ply, retired ones
    included."""
    old = cfg.abs(cfg.raw["seed_from"]["league_dir"])
    depths = {n: d for n, d, _ in read_entries(os.path.join(old, "players.txt")) +
              read_entries(os.path.join(old, "retired.txt"))}
    return read_ratings(old), depths


def seeding_plan(cfg, lineages):
    """Each lineage's seeding: its latest version (its run's last) and its old peak (its best ply-2 Elo in the first
    run's league, retired entries included), and the files each resume point needs, with whether each exists: from
    the latest, its model (and a tree's states of it and the version before, which --resume reads); from the peak,
    its model and a tree's state (copied as the next version) and the latest's state (the version before the
    copy)."""
    ratings, depths = old_league(cfg)
    q = cfg.tournament["qualify_depth"]
    out = []
    for lin in lineages:
        kind, run_dir, name = lin["kind"], lin["run_dir"], lin["name"]
        last = last_version(run_dir, kind)
        pk = old_peak(ratings, depths, name, q) if last else None
        tree = kind in TREE_KINDS
        f_latest, f_peak = {}, {}
        if last:
            f_latest[model_path(lin, last)] = os.path.exists(model_path(lin, last))
            if tree:
                for v in (last, last - 1):
                    if v >= 1:
                        p = os.path.join(run_dir, f"{kind}{v}.state")
                        f_latest[p] = os.path.exists(p)
        if pk and pk[0] != last:
            f_peak[model_path(lin, pk[0])] = os.path.exists(model_path(lin, pk[0]))
            if tree:
                p = state_file(lin, pk[0]) or os.path.join(run_dir, "cycle-states", f"{kind}{pk[0]}.state")
                f_peak[p] = os.path.exists(p)
                p = state_file(lin, last) or os.path.join(run_dir, f"{kind}{last}.state")
                f_peak[p] = os.path.exists(p)
        lo = ratings.get(f"{name}-v{last}-{q}") if last else None
        out.append(dict(name=name, kind=kind, latest=last, latest_old=lo[:2] if finite(lo) else None,
                        peak=pk[0] if pk else None, peak_entry=pk[1] if pk else None,
                        peak_elo=round(pk[2], 1) if pk else None, peak_ci=round(pk[3], 1) if pk else None,
                        files_latest=f_latest, files_peak=f_peak))
    return out


def seeding_field(cfg, plan, references):
    """The seeding cycle's field: (entries, players)."""
    q, depths = cfg.tournament["qualify_depth"], cfg.tournament["entry_depths"]
    entries = []
    for p in plan:
        if p["latest"]:
            entries += [f"{p['name']}-v{p['latest']}-{d}" for d in depths]
            if p["peak"] and p["peak"] != p["latest"]:
                entries.append(f"{p['name']}-v{p['peak']}-{q}")
    return entries, list(references) + entries


def copy_reference_files(cfg):
    """league_setup.copy_files {to: from}: the reference models' files, copied once (a reference never trains, so its
    file is its own, apart from the runs). Returns lines for the log."""
    notes = []
    for to, src in (cfg.league_setup.get("copy_files") or {}).items():
        a, b = cfg.abs(src), cfg.abs(to)
        if os.path.exists(b):
            if not same_file(a, b):
                sys.exit(f"{cfg.rel(b)} exists and differs from {cfg.rel(a)}")
            continue
        os.makedirs(os.path.dirname(b), exist_ok=True)
        shutil.copy2(a, b)
        notes.append(f"Copied {cfg.rel(a)} to {cfg.rel(b)}.")
    return notes


def plan_lines(cfg, plan, markdown):
    """The seeding plan as a table: each lineage's latest version and old peak, and its resume points' files."""
    q = cfg.tournament["qualify_depth"]
    fmt = lambda r: f"{r[0]:.0f} ± {r[1]:.0f}" if r else "-"
    files = lambda d: (", ".join(f"{os.path.relpath(p, cfg.file('runs_dir'))}{'' if ok else ' MISSING'}"
                                 for p, ok in d.items()) or "-")
    rows = []
    for p in plan:
        if not p["latest"]:
            rows.append((p["name"], "-", "-", "-", "-", "no version yet: enters after its first training cycle", "-"))
            continue
        same = p["peak"] == p["latest"]
        rows.append((p["name"], f"v{p['latest']}", fmt(p["latest_old"]),
                     f"v{p['peak']}" if p["peak"] else "-",
                     fmt((p["peak_elo"], p["peak_ci"])) if p["peak"] else "-",
                     files(p["files_latest"]),
                     "the latest (one candidate)" if same else files(p["files_peak"])))
    head = ("lineage", "latest", f"latest's ply-{q} Elo (first run)", "old peak", f"peak's ply-{q} Elo (first run)",
            "resume from the latest: files", f"resume from the peak (copied as the next version): files")
    if markdown:
        return [f"| {' | '.join(head)} |", "|" + "---|" * len(head)] + [f"| {' | '.join(r)} |" for r in rows]
    return ["  ".join(head)] + ["  ".join(r) for r in rows]


def cmd_init(cfg, force):
    """Cycle 0: the league (random, and league_setup's players: the other references) and state.json, with each
    lineage's seeding plan (seeding_plan), written to cycle-000.md. No games."""
    lk = lock(cfg)
    old = load_state(cfg)
    if old and not force:
        sys.exit("state.json exists: cycle 0 is recorded (init --force redoes it, before cycle 1 only)")
    if old and (old["cycle"] > 0 or old.get("current")):
        sys.exit("cycles have run: init --force only redoes cycle 0")
    lineages = load_registry(cfg)
    plan = seeding_plan(cfg, lineages)
    missing = [(p["name"], f) for p in plan for d in (p["files_latest"], p["files_peak"]) for f, ok in d.items()
               if not ok]
    if missing:
        sys.exit("files missing for the seeding: " + "; ".join(f"{n}: {cfg.rel(f)}" for n, f in missing))
    extra = copy_reference_files(cfg)
    extra += setup_league(cfg)
    league = cfg.file("league_dir")
    if cfg.league_setup.get("add_models"):
        rows = cfg.league_setup["add_models"]
        if own_cal(cfg):  # a calibrated reference plays on its own games (own_reference)
            rows = [dict(r, calibrated="own") if r.get("calibrated") == "yes" else r for r in rows]
        update_league(cfg, [], [], rows)
    names = [n for n, _, _ in read_players(league)]
    refs = list(cfg.raw.get("references") or names)
    if set(refs) - set(names):
        sys.exit(f"references not in the league: {', '.join(sorted(set(refs) - set(names)))}")
    state = dict(cycle=0, seeded=False, references=refs, rel_references=list(cfg.raw.get("rel_references") or refs),
                 milestones={}, lineages={})
    if own_cal(cfg):
        state["calibration"] = "own"
    for lin, p in zip(lineages, plan):
        st = lineage_state(state, lin["name"])
        st["version"], st["prepared"] = p["latest"], bool(p["latest"])
        if p["latest"]:
            st["seeding"] = {k: p[k] for k in ("latest", "latest_old", "peak", "peak_entry", "peak_elo", "peak_ci")}
    save_state(cfg, state)
    entries, field = seeding_field(cfg, plan, refs)
    rr = cfg.tournament["round_robins_per_cycle"]
    write_lines = [f"# Cycle 0: the seeding plan ({now()})", "", *extra, "",
                   f"The seeding cycle (cycle 1) trains nothing: {len(field)} players ({len(refs)} references, "
                   f"{len(entries)} lineage entries) play {rr} double round-robins, {rr * len(field) * (len(field) - 1):,} "
                   f"games. Each lineage enters with its latest version at plies "
                   f"{' and '.join(map(str, cfg.tournament['entry_depths']))} and its old peak at ply "
                   f"{cfg.tournament['qualify_depth']}; the better of the two at that ply becomes its first champion.",
                   "", *plan_lines(cfg, plan, markdown=True)]
    atomic_write(os.path.join(cfg.file("log_dir"), "cycle-000.md"), "\n".join(write_lines) + "\n")
    print(f"cycle 0 recorded: {len(plan)} lineages, {sum(1 for p in plan if p['latest'])} with versions; the seeding "
          f"field {len(field)} players; see {cfg.file('log_dir')}/cycle-000.md")
    lk.close()


def schedule_lines(cfg, cycles):
    """The schedule of cycles 1..cycles: which kind each is and what it does."""
    every = int(cfg.raw.get("global_every") or 0)
    rr = cfg.tournament["round_robins_per_cycle"]
    out = [f"cycle 1: seeding. No self-play, own steps or gates. {rr} double round-robins of the seeding field; the "
           f"better of each lineage's latest version and old peak at ply {cfg.tournament['qualify_depth']} becomes its "
           f"first champion, the other leaves the field; a lineage whose champion is its old peak gets a copy of it as "
           f"its next version, to train from."]
    normal = [c for c in range(2, cycles + 1) if not (every and c % every == 0)]
    glob = [c for c in range(2, cycles + 1) if every and c % every == 0]
    spans, start = [], None
    for c in normal:
        if start is None:
            start = prev = c
        elif c == prev + 1:
            prev = c
        else:
            spans.append((start, prev))
            start = prev = c
    if start is not None:
        spans.append((start, prev))
    text = (f"self-play ({cfg.cycle_budget_cpu_s:g} CPU s split evenly among the active lineages, each share plus "
            f"its carry), new entries, {rr} double round-robins of the field, the champion test "
            f"({cfg.raw.get('champion_se', 2.0):g} SE), patience ({cfg.raw.get('patience_cpu_s'):g} CPU s of self-play "
            f"since the last new champion), finishing (frozen at an untaken milestone or retired), own steps and "
            f"gates ({cfg.gate['games']} games at depth {cfg.gate['depth']}, kept at {cfg.gate['threshold']:g})")
    for a, b in spans:
        out.append(f"cycles {a}-{b}: {text}." if a != b else f"cycle {a}: {text}.")
    for c in glob:
        out.append(f"cycle {c}: global tournament: as the others, and every retired champion rejoins the "
                   f"round-robins at ply {cfg.tournament['qualify_depth']}; revived at {cfg.raw.get('revive_se', 2.0):g} "
                   f"SE above its expected score (it trains again from the next cycle), else frozen at an untaken "
                   f"milestone within {milestones_cfg(cfg)['window']:g}, else out again.")
    return out


def cmd_dry_run(cfg, cycles):
    """What the run would do, from the files as they are now; writes nothing: the seeding field (entries and games),
    the schedule and each lineage's resume points with their files."""
    lineages = load_registry(cfg)
    plan = seeding_plan(cfg, lineages)
    refs = list(cfg.raw.get("references") or [])
    entries, field = seeding_field(cfg, plan, refs)
    rr = cfg.tournament["round_robins_per_cycle"]
    q = cfg.tournament["qualify_depth"]
    two = sum(1 for p in plan if p["latest"] and p["peak"] and p["peak"] != p["latest"])
    print(f"dry run ({now()}): {len(lineages)} lineages in {cfg.rel(cfg.file('registry'))}, "
          f"{sum(1 for p in plan if p['latest'])} with versions; the first run's league "
          f"{cfg.raw['seed_from']['league_dir']} for the old peaks; tournament binary "
          f"{cfg.tournament['binary']}: {tournament_unavailable(cfg) or 'ready'}")
    print(f"\nThe seeding field: {len(field)} players: {len(refs)} references ({', '.join(refs)}) and "
          f"{len(entries)} lineage entries ({sum(1 for p in plan if p['latest'])} latest versions at plies "
          f"{' and '.join(map(str, cfg.tournament['entry_depths']))}, {two} old peaks at ply {q}; "
          f"{sum(1 for p in plan if p['latest'] and p['peak'] == p['latest'])} lineages' latest is their peak). "
          f"{rr} double round-robins: {rr * len(field) * (len(field) - 1):,} games.")
    print("\nThe schedule:")
    for line in schedule_lines(cfg, cycles):
        print("  " + line)
    print("\nResume points (from the latest if it wins the seeding, else from a copy of the old peak):")
    for line in plan_lines(cfg, plan, markdown=False):
        print("  " + line)
    missing = [(p["name"], f) for p in plan for d in (p["files_latest"], p["files_peak"]) for f, ok in d.items()
               if not ok]
    print(f"\nFiles missing: {len(missing)}" + "".join(f"\n  {n}: {cfg.rel(f)}" for n, f in missing))
    print("\nSeeding field entries:\n  " + "\n  ".join(field))
    if own_cal(cfg):
        print("\n" + "\n".join(own_dry_lines(cfg, lineages, plan)))


def cmd_run(cfg, cycles):
    lk = lock(cfg)
    for _ in range(cycles):
        run_cycle(cfg)
    lk.close()


def rejoins(st):
    """A retired champion that rejoins the global tournament: retired by the rules (finished, or out again after a
    global tournament), not by hand."""
    return st["status"] == "retired" and st.get("champion") and st.get("retired_by") in ("finish", "global")


def start_cycle(cfg, state, lineages, costs):
    """The next cycle's plan: its log rows, the self-play split and its jobs."""
    cycle = state["cycle"] + 1
    kind = cycle_kind(cfg, state, cycle)
    players = read_players(cfg.file("league_dir"))
    print(f"cycle {cycle}{ {'seeding': ' (seeding)', 'global': ' (global tournament)'}.get(kind, '')} ({now()})",
          flush=True)
    cur = dict(cycle=cycle, kind=kind, phase="self-play", rows={}, jobs={}, skipped={}, started=[], done=[],
               changed=[], league=None, moved=None, rejoined=None, rejoin={}, target=None, start_rr=None, field=None,
               extra=[], pool=None, own_done=[], retire=[], share=None, tournament={})
    able = {}
    for lin in lineages:
        name, st = lin["name"], lineage_state(state, lin["name"])
        if st["status"] == "retired" and not (kind == "global" and rejoins(st)):
            continue  # out of the field: no row
        row = cur["rows"][name] = dict(cycle=cycle, lineage=name, decision="", note="")
        if st["status"] != "active" or kind == "seeding":
            continue
        if not os.path.exists(lin["binary"]):
            row["decision"] = "waiting for binary"
            continue
        why = prepare(cfg, lin, st, players)
        if why:
            row["decision"] = why
            continue
        p, why = plan(cfg, costs, lin, st, 1.0)  # can it self-play at all? (the budget comes next)
        if not p:
            row["decision"] = why
            continue
        able[name] = lin
    share, budget = budgets(float(cfg.cycle_budget_cpu_s),
                            {n: float(lineage_state(state, n).get("carry_cpu") or 0.0) for n in able})
    cur["share"] = round(share, 2)
    if able:
        cur["extra"].append(f"Self-play: {cfg.cycle_budget_cpu_s:g} CPU s split evenly among {len(able)} lineages, "
                            f"{share:.1f} CPU s each, plus each one's carry.")
    for name, lin in able.items():
        st, row = lineage_state(state, name), cur["rows"][name]
        b = budget[name]
        row["budget_cpu_s"] = round(b, 1)
        p, why = plan(cfg, costs, lin, st, max(b, 0.0))
        if too_small(b, p["cost"], p["size"]):
            cur["skipped"][name] = round(b, 2)
            row["note"] = (f"no self-play: budget {b:.0f} CPU s, under half a round ({p['cost'] * p['size']:.0f} s "
                           f"at {1000 * p['cost']:.2f} ms a game); it carries over")
            print(f"  {name}: no self-play this cycle (budget {b:.0f} CPU s, a round {p['cost'] * p['size']:.0f} s)",
                  flush=True)
            continue
        cur["jobs"][name] = p
        keep_start_states(lin, p["start"])
        print(f"  {name}: from v{p['start']}, about {p['rounds']} x {p['size']} games for {b:.0f} CPU s "
              f"({p['cost_source']} {1000 * p['cost']:.2f} ms CPU/game)", flush=True)
    return cur


def entry_line(cfg, lin, v, d):
    return f"{lin['name']}-v{v}-{d} {d} {league_spec(cfg, lin, v)}"


def model_row(cfg, lin, v, cur, what):
    """models.csv's row for version v of a lineage."""
    moves, labels = describe(lin)
    p = cur["jobs"].get(lin["name"])
    r = cur["rows"].get(lin["name"]) or {}
    return dict(name=f"{lin['name']}-v{v}", archetype=lin["archetype"],
                variant=f"cycle lineage {lin['name']} ({what})", games="own (vN vs vN-1)", moves=moves,
                train_depth=lin["depth"], labels=labels, init=lin["source_type"],
                rounds=f"v{v} (cycle {cur['cycle']}" + (f": {r.get('rounds') or p['rounds']}x{p['size']})" if p else ")"),
                seed=lin["seed"], calibrated="own" if own_cal(cfg) else "yes" if lin["heldout"] else "no",
                run_dir=cfg.rel(lin["run_dir"]), script="cycles/champ/cycle.py")


def new_entries(cfg, state, lineages, cur):
    """The field's changes for this cycle (cur["league"]). Seeding: each lineage's latest version at the entry plies
    and its old peak at the qualifying ply. Later: each active lineage's newest version enters at the entry plies and
    its entries replace the current ones, except the champion's, which stays (at the qualifying ply only)."""
    depths, q = cfg.tournament["entry_depths"], cfg.tournament["qualify_depth"]
    adds, models, retire = [], [], []
    for lin in lineages:
        name, st = lin["name"], lineage_state(state, lin["name"])
        if cur["kind"] == "seeding":
            sd = st.get("seeding")
            if not sd:
                continue
            last, peak = sd["latest"], sd.get("peak")
            adds += [entry_line(cfg, lin, last, d) for d in depths]
            models.append(model_row(cfg, lin, last, cur, "latest version, seeding"))
            if peak and peak != last:
                adds.append(entry_line(cfg, lin, peak, q))
                models.append(model_row(cfg, lin, peak, cur, "old peak, seeding"))
            st["entries"], st["field_version"] = [f"{name}-v{last}-{d}" for d in depths], last
            cur["changed"].append(name)
            continue
        if st["status"] != "active" or not st["prepared"] or not st["version"] or \
                st["version"] <= (st["field_version"] or 0):
            continue
        v = st["version"]
        adds += [entry_line(cfg, lin, v, d) for d in depths]
        models.append(model_row(cfg, lin, v, cur, "trained"))
        champ = (st.get("champion") or {}).get("entry")
        retire += [e for e in st["entries"] or [] if e != champ]
        st["prev_entries"], st["entries"], st["field_version"] = st["entries"] or [], \
            [f"{name}-v{v}-{d}" for d in depths], v
        cur["changed"].append(name)
    cur["league"] = dict(retire=sorted(set(retire)), add_lines=adds, add_models=models)


def decide_cycle(cfg, state, lineages, cur):
    """The cycle's decisions, in state.json (saved by the caller with the phase) and the rows: the seeding's first
    champions, or the champion test, patience and finishing, and in a global cycle the revivals. The entries that
    leave the field go to cur["retire"] (moved after the own steps). Copies a champion's files as its lineage's next
    version where training resumes from it. Safe to repeat after an interruption: the state on disk is the one from
    before, and the copies are skipped when made."""
    league, q, depths = cfg.file("league_dir"), cfg.tournament["qualify_depth"], cfg.tournament["entry_depths"]
    cycle, kind, rows = cur["cycle"], cur["kind"], cur["rows"]
    ratings = read_ratings(league)
    ref = references_mean(ratings, rel_references(state))
    k, mc = float(cfg.raw.get("champion_se", 2.0)), milestones_cfg(cfg)
    patience = float(cfg.raw.get("patience_cpu_s") or math.inf)
    rating = lambda e: ratings[e] if e and finite(ratings.get(e)) else None
    retire, finishing, leaving = [], [], []
    for row in rows.values():  # what the earlier phases said (a failed self-play, say) moves to the note
        if row.get("decision"):
            row["note"], row["decision"] = f"{row['decision']}. {row.get('note') or ''}".strip(), ""

    def show(row, st):  # a row's ratings: its current version's at the qualifying ply, and its champion's
        e = f"{row['lineage']}-v{st['field_version']}-{q}" if st["entries"] else None
        r = rating(e)
        if r:
            st["elo"], st["ci"], st["rel"], st["elo_cycle"] = r[0], r[1], r[0] - ref, cycle
        row.update(version=st["field_version"] if st["entries"] else "", entries=" ".join(field_entries(st)),
                   elo=round(r[0], 1) if r else None, ci95=round(r[1], 1) if r else None,
                   rel_elo=round(r[0] - ref, 1) if r else None)
        ch = st.get("champion")
        if ch:
            cr = rating(ch["entry"])
            ch["elo"], ch["ci"] = (round(cr[0], 1), round(cr[1], 1)) if cr else (None, None)
            row.update(champion_version=ch["version"], champion_elo=ch["elo"], champion_ci95=ch["ci"])
        row["patience_cpu_s"] = round(st.get("patience_cpu") or 0.0, 1)

    if kind == "seeding":
        for lin in lineages:
            name, st, row = lin["name"], lineage_state(state, lin["name"]), rows[lin["name"]]
            sd = st.get("seeding")
            if not sd:
                row["decision"] = "waits for a first version"
                continue
            last, peak = sd["latest"], sd.get("peak")
            el, ep = f"{name}-v{last}-{q}", (f"{name}-v{peak}-{q}" if peak and peak != last else None)
            rl, rp = rating(el), rating(ep)
            win = peak if ep and rp and (not rl or rp[0] > rl[0]) else last
            if win == peak and win != last:
                try:
                    v = copy_as_next(lin, peak)
                except OSError as e:
                    win, v = last, None
                    row["note"] = f"the old peak won but can't be resumed ({e}); the latest stays champion. "
            if win != last:
                retire += [f"{name}-v{last}-{d}" for d in depths]
                st.update(version=v, field_version=v, entries=[])
                row["decision"] = "first champion: old peak"
            else:
                if ep:
                    retire.append(ep)
                row["decision"] = "first champion: latest"
            st["champion"] = dict(version=win, entry=f"{name}-v{win}-{q}", since=cycle)
            st["patience_cpu"], st["carry_cpu"] = 0.0, 0.0
            keep_champion_state(lin, win)
            f = lambda r: f"{r[0]:.0f} ± {r[1]:.0f}" if r else "unrated"
            row["note"] = (row.get("note") or "") + (
                f"latest v{last} {f(rl)}" + (f", old peak v{peak} {f(rp)} (first run: {sd['peak_elo']:.0f})"
                                             if ep else " (also its old peak)") +
                (f"; trains from v{v}, a copy of v{peak}" if win != last else ""))
            show(row, st)
            played = [f"{name}-v{last}-{d}" for d in depths] + ([ep] if ep else [])
            row.update(version=last, entries=" ".join(played), elo=round(rl[0], 1) if rl else None,
                       ci95=round(rl[1], 1) if rl else None, rel_elo=round(rl[0] - ref, 1) if rl else None)
        state["seeded"] = True
    else:
        pairs = {}
        for lin in lineages:
            st = lineage_state(state, lin["name"])
            ch = st.get("champion")
            if st["status"] == "active" and ch and st["entries"]:
                e = f"{lin['name']}-v{st['field_version']}-{q}"
                if e != ch["entry"] and rating(e) and rating(ch["entry"]):
                    pairs[lin["name"]] = ([e], [ch["entry"]])
        ses = {}
        if pairs:
            t0 = time.process_time()
            got = contrast_ses(league, ratings, list(pairs.values()))
            how = f"from the fit's covariance ({time.process_time() - t0:.1f} s)"
            if got is None:
                got = [math.hypot(ratings[a[0]][1], ratings[b[0]][1]) / 1.96 for a, b in pairs.values()]
                how = "from the two entries' ± combined (the fit's information matrix is not positive definite)"
            ses = dict(zip(pairs, got))
            cur["extra"].append(f"Champion test: {len(pairs)} comparisons, their standard errors {how}.")
        for lin in lineages:
            name, st, row = lin["name"], lineage_state(state, lin["name"]), rows[lin["name"]]
            if st["status"] == "frozen":
                row["decision"] = row["decision"] or "frozen"
                show(row, st)
                continue
            if st["status"] != "active":
                continue  # a retired champion back for the global tournament: below
            e = f"{name}-v{st['field_version']}-{q}" if st["entries"] else None
            ch = st.get("champion")
            if ch is None:
                if rating(e):
                    st["champion"] = dict(version=st["field_version"], entry=e, since=cycle)
                    st["patience_cpu"] = 0.0
                    keep_champion_state(lin, st["field_version"])
                    row["decision"] = "first champion"
                else:
                    row["decision"] = row["decision"] or "waits for a first version"
            elif name in pairs:
                new, diff, z = champion_test(rating(e), rating(ch["entry"]), ses[name], k)
                row.update(diff=round(diff, 1), diff_se=round(ses[name], 1))
                if new:
                    retire.append(ch["entry"])
                    st["champion"] = dict(version=st["field_version"], entry=e, since=cycle)
                    st["patience_cpu"] = 0.0
                    keep_champion_state(lin, st["field_version"])
                    row["decision"] = "new champion"
                    row["note"] = (row.get("note") or "") + f"{diff:.0f} above v{ch['version']}, {z:.2f} SE"
                else:
                    row["decision"] = "champion kept"
                    row["note"] = (row.get("note") or "") + (f"{gap_words(diff)} v{ch['version']}, {z:.2f} SE "
                                                             f"(a new champion needs {k:g})")
            else:
                row["decision"] = row["decision"] or "champion kept"
                row["note"] = (row.get("note") or "") + ("its current version is the champion" if e == ch["entry"]
                                                         else "no rated current version this cycle")
            show(row, st)
            if st.get("champion") and row["decision"] not in ("first champion", "new champion") and \
                    (st.get("patience_cpu") or 0.0) >= patience:
                finishing.append(name)
        if cur.get("rejoin"):  # the global tournament: revival
            games = cycle_games(league, cur["start_rr"], [b["entry"] for b in cur["rejoin"].values()])
            rs = float(cfg.raw.get("revive_se", 2.0))
            by = {l["name"]: l for l in lineages}
            for name, b in sorted(cur["rejoin"].items()):
                st, row = lineage_state(state, name), rows[name]
                rv = revival(games, b["entry"], b["elo0"], b["ci0"], ratings) if b.get("elo0") is not None else None
                if rv:
                    row.update(revive_games=rv["games"], revive_score=rv["score"], revive_expected=rv["expected"],
                               revive_z=rv["z"])
                if rv and rv["z"] >= rs and name in by:
                    try:
                        v = copy_as_next(by[name], st["champion"]["version"])
                    except OSError as e:
                        row["note"] = f"not revived: {e}"
                        leaving.append(name)
                        continue
                    for x in ("retired_by", "retired_cycle", "retired_from"):
                        st.pop(x, None)
                    st.update(status="active", version=v, field_version=v, entries=[], patience_cpu=0.0,
                              carry_cpu=0.0)
                    st.setdefault("revived", []).append(cycle)
                    row["decision"] = "revived"
                    row["note"] = (f"{rv['score']:g} points in {rv['games']} games vs {rv['expected']:.1f} expected "
                                   f"from its {b['elo0']:.0f} before, {rv['z']:.2f} SE; trains from v{v}"
                                   + (f", a copy of v{st['champion']['version']}" if v != st["champion"]["version"]
                                      else ""))
                else:
                    leaving.append(name)
                    if rv:
                        row["note"] = (f"{rv['score']:g} points in {rv['games']} games vs {rv['expected']:.1f} "
                                       f"expected, {rv['z']:.2f} SE (revival needs {rs:g})")
                show(row, st)
        taken = taken_milestones(cfg, state, ratings)
        group = []
        for name in finishing + leaving:
            r = rating(state["lineages"][name]["champion"]["entry"])
            group.append((name, {str(q): r[0]} if r else {}))
        got = resolve_claims([group], set(taken), mc["step"], mc["window"])
        for name in finishing + leaving:
            st, row = state["lineages"][name], rows[name]
            ch = st["champion"]
            how = "finish" if name in finishing else "global"
            if got.get(name):
                _, m, a = got[name]
                retire += [x for x in st["entries"] or [] if x != ch["entry"]]
                for x in ("retired_by", "retired_cycle", "retired_from"):
                    st.pop(x, None)
                st.update(status="frozen", frozen_by=how, frozen_cycle=cycle, claim=m, entries=[])
                taken.setdefault(m, []).append(ch["entry"])
                row["decision"] = "finished-frozen" if how == "finish" else "frozen at the global tournament"
                row["note"] = ((row.get("note") or "") + "; " if row.get("note") else "") + \
                    f"champion v{ch['version']} at {a:.1f}: milestone {m}"
            elif how == "finish":
                retire += [x for x in st["entries"] or []] + [ch["entry"]]
                st.update(status="retired", retired_by="finish", retired_cycle=cycle, entries=[])
                row["decision"] = "finished-retired"
                r = rating(ch["entry"])
                row["note"] = (f"champion v{ch['version']} at {r[0]:.1f}: " if r else f"champion v{ch['version']}: ") \
                    + f"no untaken milestone within {mc['window']:g}"
            else:
                retire.append(ch["entry"])
                row["decision"] = "out again"
            if how == "finish":
                row["note"] = f"patience {st.get('patience_cpu', 0):.0f} of {patience:g} CPU s; " + row["note"]
        state["milestones"] = {str(m): ", ".join(t) for m, t in sorted(taken.items())}
    for lin in lineages:
        rows[lin["name"]]["status"] = lineage_state(state, lin["name"])["status"]
        rows[lin["name"]]["date"] = now()
    cur["retire"] = sorted(set(retire))


def run_cycle(cfg):
    """One cycle, in phases saved in state.json, so that an interrupted cycle resumes where it stopped."""
    state = load_state(cfg)
    if not state:
        sys.exit("no state.json: run init first")
    if own_cal(cfg) != (state.get("calibration") == "own"):
        sys.exit(f"config calibration is {cfg.raw.get('calibration', 'heldout')}, but this league was set up with "
                 f"{state.get('calibration', 'heldout')} (state.json): mixing them would mix how its entries play")
    costs = load_costs(cfg)
    if not state.get("current"):
        why = tournament_unavailable(cfg)
        if why:
            sys.exit(f"can't run a cycle: {why}")
        state["current"] = start_cycle(cfg, state, load_registry(cfg), costs)
        save_state(cfg, state)
    else:
        print(f"cycle {state['current']['cycle']}: resuming at {state['current']['phase']}", flush=True)
    cur = state["current"]
    cycle, rows, kind = cur["cycle"], cur["rows"], cur["kind"]
    registry = load_registry(cfg)
    lineages = [l for l in registry if l["name"] in rows]  # lineages added since wait for the next cycle
    league = cfg.file("league_dir")

    if cur["phase"] == "self-play":  # 1. self-play for the active lineages, each for its budget
        by_name = {l["name"]: l for l in lineages}
        cut = sorted(set(cur["started"]) - set(cur["done"]))  # running when the driver stopped
        if cut:
            cur["extra"].append(f"Self-play of {', '.join(cut)} was interrupted and resumed; its CPU time counts the "
                                f"rounds the cut command saved at the cost then known.")

        def started(name):
            if name not in cur["started"]:
                cur["started"].append(name)
            save_state(cfg, state)

        def stepper(name):
            lin, p = by_name[name], cur["jobs"][name]

            def step(result):
                cmd = next_step(cfg, lin, p, result)
                save_state(cfg, state)  # the measured cost and the total, the moment they are decided
                return cmd
            return step

        def finished(name):
            p, row, lin = cur["jobs"][name], rows[name], by_name[name]
            v = last_version(lin["run_dir"], lin["kind"])
            parts = p.get("parts", [])
            cpu, wall, m = sum(x[2] for x in parts), sum(x[3] for x in parts), p.get("measured") or {}
            row.update(rounds=v - p["start"], round_size=p["size"], games=(v - p["start"]) * p["size"],
                       cpu_s=f"{cpu:.1f}", wall_s=f"{wall:.1f}", cost_ms=f"{1000 * m['cost']:.4f}" if m else "",
                       segment_rounds=m.get("rounds", ""), start_version=p["start"], end_version=v,
                       code=1 if p.get("failed") else 0)
            if m:
                lineage_state(state, name)["cost"] = m["cost"]
            cur["done"].append(name)
            save_state(cfg, state)
            print(f"  finished {name}: v{p['start']} -> v{v}, {v - p['start']} x {p['size']} games, cpu {cpu:.0f} s "
                  f"of {p['budget']:.0f}, wall {wall:.0f} s" + (f"; measured {1000 * m['cost']:.2f} ms CPU/game over "
                                                                f"{m['rounds']} rounds" if m else "")
                  + (f"; FAILED: {p['failed']}" if p.get("failed") else ""), flush=True)

        run_jobs(cfg, [(n, stepper(n), by_name[n]["run_dir"] + ".log") for n in cur["jobs"]
                       if n in by_name and n not in cur["done"]], cycle, started, finished)
        for name in cur["done"]:
            lin, st, row, p = by_name[name], lineage_state(state, name), rows[name], cur["jobs"][name]
            st["version"] = last_version(lin["run_dir"], lin["kind"])
            failed = row.pop("code", 0) != 0 or st["version"] != p.get("end")
            if failed:
                row["decision"] = f"self-play failed (last version v{st['version']}); see {lin['run_dir']}.log"
            used = sum(x[2] for x in p.get("parts", [])) + p.get("cut_cpu", 0.0)
            settle(st, p["budget"], used, failed)  # once: saved with the phase change
            row.update(budget_cpu_s=round(p["budget"], 1), carry_cpu_s=st["carry_cpu"])
        for name, b in cur["skipped"].items():
            st = lineage_state(state, name)
            settle(st, b, 0.0)
            rows[name].update(budget_cpu_s=round(b, 1), carry_cpu_s=st["carry_cpu"])
        new_entries(cfg, state, lineages, cur)
        cur["phase"] = "tournament"
        save_state(cfg, state)

    if cur["phase"] == "tournament":  # 2. the field, then fresh round-robins of all of it; games saved for the step
        own_any = kind != "seeding" and any(own_on(cfg, l) for l in lineages)
        why = own_unavailable(cfg) if own_any else None
        wait_league(cfg)
        if cur["moved"] is None:
            cur["moved"] = update_league(cfg, **cur["league"])
            cur["extra"].append(f"League: {cur['moved']} players to retired.txt (their games stay in the fit), "
                                f"{len(cur['league']['add_lines'])} added.")
            save_state(cfg, state)
        if kind == "global" and cur["rejoined"] is None:  # the retired champions, with their ratings before
            before = read_ratings(league)
            for lin in lineages:
                st = lineage_state(state, lin["name"])
                if rejoins(st):
                    e = st["champion"]["entry"]
                    r = before.get(e)
                    cur["rejoin"][lin["name"]] = dict(entry=e, elo0=round(r[0], 2) if finite(r) else None,
                                                      ci0=round(r[1], 2) if finite(r) else None)
            cur["rejoined"] = unretire_players(cfg, [b["entry"] for b in cur["rejoin"].values()])
            cur["extra"].append(f"Global tournament: {cur['rejoined']} retired champions rejoin the round-robins.")
            save_state(cfg, state)
        if cur["target"] is None:  # fixed once, so that a resumed tournament plays only what is left
            cur["start_rr"] = done_round_robins(league)
            cur["target"] = cur["start_rr"] + cfg.tournament["round_robins_per_cycle"]
            cur["field"] = [n for n, _, _ in read_players(league)]
            save_state(cfg, state)
        if own_cal(cfg) and cur.get("own_cal") is None:  # every entry's own games, before it plays
            o = cur["own_cal"] = own_field(cfg)
            cur["extra"].append(f"Own calibration: the field's {o['field']} versions calibrated on their own games; "
                                f"{o['made']} made ({o['cpu_s']:.0f} CPU s, {o['wall_s']:.0f} s), {o['reused']} "
                                f"reused from a seed directory, {o['deleted']} deleted (versions that left the "
                                f"field); {o['mb']:,} MB in {cfg.rel(own_root(cfg))}.")
            save_state(cfg, state)
        cur["pool"] = pool_dir(cfg, cycle) if own_any and not why else None
        secs = run_tournament(cfg, cur["target"], cur["pool"])
        n, rr = len(cur["field"]), cfg.tournament["round_robins_per_cycle"]
        cur["tournament"] = dict(players=n, games=rr * n * (n - 1), minutes=round(secs / 60, 2))
        cur["extra"].append(f"Tournament: {rr} double round-robins of the {n} players ({rr * n * (n - 1):,} games) in "
                            f"{secs / 60:.1f} min" + (f", {len(saved_games(cur['pool'])):,} games saved." if cur["pool"]
                                                      else "."))
        if why:
            cur["extra"].append(f"Own-games step skipped: {why}.")
        cur["phase"] = "decide"
        save_state(cfg, state)

    if cur["phase"] == "decide":  # 3. the champion test, patience, finishing; seeding's first champions; revivals
        decide_cycle(cfg, state, lineages, cur)
        state.setdefault("round_robin_ends", {})[str(cycle)] = cur["target"]  # elo_history's end of this cycle
        cur["phase"] = "own"
        save_state(cfg, state)

    if cur["phase"] == "own":  # 4. the own-games step and its gate; then the field's departures
        if cur["pool"]:
            own_steps(cfg, state, lineages, cur["pool"], cycle, rows, read_players(league), cur["own_done"],
                      own_opponents(cfg, state, cur["field"]),
                      cur.setdefault("own_gates", dict(made=0, cpu_s=0.0)) if own_cal(cfg) else None)
            if own_cal(cfg) and cur["own_gates"]["made"]:
                g = cur["own_gates"]
                cur["extra"].append(f"Own calibration in the gates: {g['made']} trained versions' own games made "
                                    f"before their gates ({g['cpu_s']:.0f} CPU s), deleted after them.")
        if cur["retire"]:  # before the next round-robin
            moved = update_league(cfg, cur["retire"], [], [])
            cur["extra"].append(f"Leaving the field: {moved} entries to retired.txt ({' '.join(cur['retire'])}).")
        cur["phase"] = "log"
        save_state(cfg, state)

    # 5. The logs.
    for lin in lineages:
        st = lineage_state(state, lin["name"])
        rows[lin["name"]]["next_version"] = st["version"]
        if rows[lin["name"]].get("patience_cpu_s") in (None, ""):
            rows[lin["name"]]["patience_cpu_s"] = round(st.get("patience_cpu") or 0.0, 1)
    ordered = [rows[l["name"]] for l in lineages]
    if cycle not in logged_cycles(cfg):
        append_log(cfg, ordered)
    statuses = [lineage_state(state, n)["status"] for n in state["lineages"]]
    dec = [r.get("decision") for r in ordered]
    num = lambda k: sum(float(r[k]) for r in ordered if r.get(k) not in (None, ""))
    t = cur.get("tournament") or {}
    append_totals(cfg, dict(
        cycle=cycle, date=now(), kind=kind, active=statuses.count("active"), frozen=statuses.count("frozen"),
        retired=statuses.count("retired"), trained=len(cur["jobs"]), field=t.get("players"),
        round_robins=cfg.tournament["round_robins_per_cycle"], games=t.get("games"), tournament_min=t.get("minutes"),
        budget_cpu_s=cfg.cycle_budget_cpu_s if kind != "seeding" else 0, share_cpu_s=cur.get("share"),
        selfplay_cpu_s=round(num("cpu_s"), 1), own_cpu_s=round(num("own_cpu_s"), 1),
        gate_cpu_s=round(num("gate_cpu_s"), 1),
        new_champions=sum(1 for d in dec if d == "new champion" or str(d).startswith("first champion")),
        finished_frozen=dec.count("finished-frozen"), finished_retired=dec.count("finished-retired"),
        revived=dec.count("revived"), global_frozen=dec.count("frozen at the global tournament"),
        left_again=dec.count("out again")))
    write_summary(cfg, cycle, kind, ordered, rel_refs=rel_references(state), extra=[
        f"{now()}: " + ("the seeding cycle: no training." if kind == "seeding" else
                        f"self-play budget {cfg.cycle_budget_cpu_s:g} CPU s, {cur.get('share') or 0:.1f} for each of "
                        f"{len(cur['jobs']) + len(cur['skipped'])} lineages plus its carry; each job measured its CPU "
                        f"per game over its first rounds (at least {100 * MEASURE_MIN:.0f}% of its budget), then played "
                        f"the rest in chunks of about {100 * CHUNK:.0f}% of it, each re-planned from the last one's "
                        f"CPU per game.")] + cur["extra"])
    pool = cur["pool"]
    state["cycle"] = cycle
    del state["current"]
    save_state(cfg, state)
    if pool and not cfg.own_games.get("keep_pools", False):
        shutil.rmtree(pool, ignore_errors=True)
    print(f"cycle {cycle} done: {len(cur['done'])} trained, {len(cur['changed'])} new in the field; see "
          f"{cfg.file('log_dir')}/cycle-{cycle:03d}.md", flush=True)


def verify_cycle(cfg, names, cycle):
    """Replays each lineage's self-play of `cycle` from its cycles.csv row (start and end versions, round size) with
    the lineage's binary, in a scratch directory beside the runs, and compares the models byte for byte. A tree
    needs the training state of its start version, which only the last two versions keep."""
    lineages = {l["name"]: l for l in load_registry(cfg)}
    with open(cfg.file("log_csv"), newline="") as f:
        rows = {r["lineage"]: r for r in csv.DictReader(f) if r.get("cycle") == str(cycle)}
    ok = True
    for name in names:
        lin, r = lineages.get(name), rows.get(name)
        if not lin or not r or not r.get("start_version") or not r.get("end_version"):
            print(f"{name}: no self-play record with start and end versions in cycle {cycle}", flush=True)
            ok = False
            continue
        start, end, size, kind, src = (int(r["start_version"]), int(r["end_version"]), int(r["round_size"]),
                                       lin["kind"], lin["run_dir"])
        need = [f"{kind}{n}.model" for n in range(1, start + 1)]
        if kind in TREE_KINDS and start:
            need += [f"{kind}{start}.state"] + ([f"{kind}{start - 1}.state"] if start > 1 else [])
        where = {x: next((os.path.join(dd, x) for dd in (src, os.path.join(src, "cycle-states"))
                          if os.path.exists(os.path.join(dd, x))), None) for x in need}
        missing = [x for x in need if not where[x]]
        if missing:
            print(f"{name}: can't replay cycle {cycle}: {missing[0]} is no longer kept", flush=True)
            ok = False
            continue
        tmp = os.path.join(cfg.file("runs_dir"), f".verify-{name}")
        shutil.rmtree(tmp, ignore_errors=True)
        os.makedirs(tmp)
        for x in need:
            os.link(where[x], os.path.join(tmp, x))
        with open(os.path.join(tmp, "ladder.csv"), "w") as f:
            f.write(LADDER_HEADER)
        cmd = ["nice", "-n", str(cfg.nice), lin["binary"], "self-play", tmp, *lin["tokens"], "--games", str(size),
               "--rounds", str(end), "--threads", str(cfg.threads_per_job)]
        cmd += ["--resume"] if start else (["--init", lin["source"]] if lin["source_type"] == "init" else [])
        t0 = time.time()
        res = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        same = []
        for n in range(start + 1, end + 1):
            a, b = os.path.join(tmp, f"{kind}{n}.model"), os.path.join(src, f"{kind}{n}.model")
            if os.path.exists(a) and os.path.exists(b):
                with open(a, "rb") as x, open(b, "rb") as y:
                    same.append(x.read() == y.read())
        good = res.returncode == 0 and len(same) == end - start and all(same)
        ok &= good
        print(f"{name}: cycle {cycle}, v{start} -> v{end} ({size} games per round) replayed in "
              f"{time.time() - t0:.0f} s: " + ("identical" if good else f"DIFFERENT (exit {res.returncode}, "
                                               f"{sum(same)}/{end - start} identical)\n" + res.stdout[-800:]),
              flush=True)
        shutil.rmtree(tmp)
    return ok


def cmd_verify(cfg, names, rounds):
    """Replays the last `rounds` rounds of a continued run with the lineage's binary and compares the models."""
    lineages = {l["name"]: l for l in load_registry(cfg)}
    ok = True
    for name in names:
        lin = lineages[name]
        src, kind = lin["source"], lin["kind"]
        k = last_version(src, kind)
        start = k - rounds
        games = 1000
        log = src.rstrip("/") + ".log"
        if os.path.exists(log):
            with open(log) as f:
                m = re.search(r"rounds of (\d+) games", f.read(4000))
            games = int(m.group(1)) if m else games
        tmp = os.path.join(cfg.file("runs_dir"), f".verify-{name}")
        shutil.rmtree(tmp, ignore_errors=True)
        os.makedirs(tmp)
        for n in range(1, start + 1):
            os.link(os.path.join(src, f"{kind}{n}.model"), os.path.join(tmp, f"{kind}{n}.model"))
        with open(os.path.join(tmp, "ladder.csv"), "w") as f:
            f.write(LADDER_HEADER)
        cmd = ["nice", "-n", str(cfg.nice), lin["binary"], "self-play", tmp, *lin["tokens"], "--games", str(games),
               "--rounds", str(k), "--threads", str(cfg.threads_per_job), "--resume"]
        t0 = time.time()
        r = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        same = []
        for n in range(start + 1, k + 1):
            a = os.path.join(tmp, f"{kind}{n}.model")
            if os.path.exists(a):
                with open(a, "rb") as x, open(os.path.join(src, f"{kind}{n}.model"), "rb") as y:
                    same.append(x.read() == y.read())
        good = r.returncode == 0 and len(same) == rounds and all(same)
        ok &= good
        print(f"{name}: v{start} -> v{k} ({games} games per round) replayed in {time.time() - t0:.0f} s with "
              f"{os.path.basename(lin['binary'])}: " + ("identical" if good else
                                                        f"DIFFERENT (exit {r.returncode}, {sum(same)}/{rounds} "
                                                        f"identical)\n" + r.stdout[-800:]), flush=True)
        shutil.rmtree(tmp)
    return ok


def selftest():
    # the champion test: k standard errors of the difference, from the fit's covariance
    assert champion_test((150.0, 20.0), (100.0, 20.0), 25.0, 2.0) == (True, 50.0, 2.0)
    assert champion_test((149.0, 20.0), (100.0, 20.0), 25.0, 2.0)[0] is False
    assert champion_test((90.0, 20.0), (100.0, 20.0), 5.0, 2.0) == (False, -10.0, -2.0)
    assert champion_test((150.0, 20.0), (100.0, 20.0), 0.0, 2.0)[0] is False  # no standard error: no decision
    # the revival statistic: 10 wins against opponents rated as the champion was before (expected 5 points), with
    # the binomial variance 2.5 and the rating's (ci 19.6: SE 10) in E's variance; draws half; others' games ignored
    games = [("R", f"o{i}", "1-0") if i % 2 else (f"o{i}", "R", "0-1") for i in range(10)] + [("o1", "o2", "1-0")]
    rt = {f"o{i}": (1000.0, 20.0, 10) for i in range(10)}
    rv = revival(games, "R", 1000.0, 19.6, rt)
    var = 2.5 + (2.5 * math.log(10) / 400 * 10) ** 2
    assert rv["games"] == 10 and rv["score"] == 10 and abs(rv["z"] - round(5 / math.sqrt(var), 3)) < 1e-9, rv
    rv = revival([("R", "o1", "1/2-1/2"), ("o2", "R", "1/2-1/2"), ("R", "x", "1-0")], "R", 1000.0, 0.0, rt)
    assert rv == dict(games=2, score=1.0, expected=1.0, z=0.0), rv  # an unrated opponent's game doesn't count
    # milestones: ply 2 only, closest first (a tie by name), the references' within 50 and the frozen claims taken
    rc = lambda group, taken=(0, 200, 300, 400): resolve_claims([group], set(taken), 100, 10)
    assert rc([("B", {"2": 1503.0}), ("A", {"2": 1497.0})]) == {"A": (2, 1500, 1497.0), "B": None}
    assert rc([("B", {"2": 1502.0}), ("A", {"2": 1497.0})]) == {"B": (2, 1500, 1502.0), "A": None}
    assert rc([("A", {"2": 1489.9})]) == {"A": None} and rc([("A", {"2": 410.0})]) == {"A": None}
    assert rc([("A", {})]) == {"A": None}  # unrated: retired
    import types
    cf = types.SimpleNamespace(raw={})
    stt = dict(references=["random", "random-1", "ref-1"], lineages={
        "F": dict(status="frozen", claim=1500, champion=dict(entry="F-v9-2")), "G": dict(status="active", claim=900)})
    tk = taken_milestones(cf, stt, {"random": (0.0, 1.0, 9), "random-1": (151.0, 9.0, 9), "ref-1": (452.0, 9.0, 9)})
    assert tk == {0: ["random"], 200: ["random-1"], 500: ["ref-1"], 1500: ["F-v9-2"]}, tk  # within 50 of 500
    # the budget split: an even share each, plus each lineage's carry
    assert budgets(3600.0, {"a": 0.0, "b": 10.0, "c": -20.0}) == (1200.0, {"a": 1200.0, "b": 1210.0, "c": 1180.0})
    assert budgets(3600.0, {}) == (0.0, {}) and too_small(49.9, 0.1, 1000) and not too_small(50.0, 0.1, 1000)
    # contrast_ses: exactly fit_factor's and contrast_se's numbers, on a league of three cycles' fields (references
    # and a champion every cycle, a long-lived entry in two, one-cycle entries), its pairs among any players
    with tempfile.TemporaryDirectory() as d:
        fields = [["random", "r1", "r2", "C", "L", "a1", "a2"], ["random", "r1", "r2", "C", "L", "b1", "b2"],
                  ["random", "r1", "r2", "C", "c1", "c2"]]
        res = ["1-0", "0-1", "1/2-1/2"]
        lines = ["tournament,white,black,result,plies"]
        for t, field in enumerate(fields):
            for i, w in enumerate(field):
                for j, b in enumerate(field):
                    if w != b:
                        lines.append(f"{t},{w},{b},{res[(i * 7 + j * 3 + t) % 3]},50")
        with open(os.path.join(d, "games.csv"), "w") as f:
            f.write("\n".join(lines) + "\n")
        names = sorted({p for fl in fields for p in fl})
        rt = {p: (0.0 if p == "random" else 37.0 * (i % 5) - 40.0, 20.0, 10) for i, p in enumerate(names)}
        pairs = [(["c1"], ["C"]), (["b2"], ["L"]), (["a1", "a2"], ["random"]), (["L"], ["r1"]), (["c2"], ["a1"])]
        fac = fit_factor(d, rt)
        want = [contrast_se(fac, a, b) for a, b in pairs]
        got = contrast_ses(d, rt, pairs)
        assert all(abs(g - w) < 1e-9 * w for g, w in zip(got, want)), (got, want)
        # copying a champion as the run's next version: an LR run, then a tree (its state too, from cycle-states/)
        run = os.path.join(d, "runs", "X")
        os.makedirs(os.path.join(run, "cycle-states"))
        for v in range(1, 6):
            with open(os.path.join(run, f"lr{v}.model"), "w") as f:
                f.write(f"lr {v}\n")
        lin = dict(name="X", kind="lr", run_dir=run)
        assert copy_as_next(lin, 5) == 5 and last_version(run, "lr") == 5  # the latest: nothing to copy
        assert copy_as_next(lin, 3) == 6 and open(os.path.join(run, "lr6.model")).read() == "lr 3\n"
        assert copy_as_next(lin, 3) == 6 and last_version(run, "lr") == 6  # a rerun copies nothing
        tree = dict(name="T", kind="bdt", run_dir=run)
        for v in range(1, 6):
            with open(os.path.join(run, f"bdt{v}.model"), "w") as f:
                f.write(f"bdt {v}\n")
        for v, where in ((3, "cycle-states"), (4, "cycle-states"), (5, "")):
            with open(os.path.join(run, where, f"bdt{v}.state"), "w") as f:
                f.write(f"state {v}\n")
        try:
            copy_as_next(tree, 2)
            raise AssertionError("copied a tree without its state")
        except OSError as e:
            assert "state" in str(e), e
        assert copy_as_next(tree, 3) == 6 and open(os.path.join(run, "bdt6.state")).read() == "state 3\n"
        assert open(os.path.join(run, "bdt6.model")).read() == "bdt 3\n" and copy_as_next(tree, 3) == 6
        assert copy_as_next(tree, 4) == 7 and os.path.exists(os.path.join(run, "bdt6.state"))
        keep_champion_state(tree, 7)  # the champion's state kept where self-play never deletes it
        assert open(os.path.join(run, "cycle-states", "bdt7.state")).read() == "state 4\n"
        os.remove(os.path.join(run, "bdt7.state"))  # self-play keeps the last two states only
        assert state_file(tree, 7).endswith(os.path.join("cycle-states", "bdt7.state"))
        assert copy_as_next(tree, 5) == 8  # resumes from the states of v8 and v7, both in the run directory
        assert os.path.exists(os.path.join(run, "bdt7.state")) and open(os.path.join(run, "bdt8.state")).read() == \
            "state 5\n"
    # the old peak: the best rated version at the ply, of this lineage only
    rt = {"X-v1-2": (100.0, 9.0, 1), "X-v3-2": (150.0, 9.0, 1), "X-v3-1": (500.0, 9.0, 1),
          "X-merge-v9-2": (900.0, 9.0, 1), "X-v4-2": (float("nan"), float("nan"), 1)}
    dp = {"X-v1-2": 2, "X-v3-2": 2, "X-v3-1": 1, "X-merge-v9-2": 2, "X-v4-2": 2}
    assert old_peak(rt, dp, "X", 2) == (3, "X-v3-2", 150.0, 9.0) and old_peak(rt, dp, "Y", 2) is None
    assert field_entries(dict(status="active", entries=["X-v5-1", "X-v5-2"], champion=dict(entry="X-v3-2"))) == [
        "X-v5-1", "X-v5-2", "X-v3-2"]
    assert field_entries(dict(status="active", entries=["X-v5-1", "X-v5-2"], champion=dict(entry="X-v5-2"))) == [
        "X-v5-1", "X-v5-2"]
    assert field_entries(dict(status="frozen", entries=[], champion=dict(entry="X-v3-2"))) == ["X-v3-2"]
    assert field_entries(dict(status="retired", entries=[], champion=dict(entry="X-v3-2"))) == []
    # the gate's arithmetic matches the arena's (selfplay::score_match; F-L-s21's ladder row 999).
    s, elo, ci = match_stats(48, 37, 915)
    assert abs(s - 0.5055) < 1e-4 and abs(elo - 3.8) < 0.05 and abs(ci - 6.3) < 0.1, (s, elo, ci)
    assert match_stats(10, 10, 0)[:2] == (0.5, 0.0)
    # lineage ratings: means over the entries; the references are the players that aren't lineage entries.
    ratings = {"random": (0.0, 9.0, 1), "a-1": (100.0, 10.0, 1), "x-v2-1": (200.0, 20.0, 1),
               "x-v2-2": (300.0, 30.0, 1), "x-v1-1": (50.0, 10.0, 1)}
    assert rating_of(ratings, ["x-v2-1", "x-v2-2", "gone"]) == (250.0, 25.0) and rating_of(ratings, ["gone"]) is None
    assert references_mean(ratings, ["random", "a-1", "unrated"]) == 50.0
    depths = {"x-v2-1": 1, "x-v2-2": 2, "x-v1-1": 1}
    assert rating_at(ratings, ["x-v2-1", "x-v2-2"], depths, 2) == (300.0, 30.0)
    assert rating_at(ratings, ["x-v1-1"], depths, 2) is None
    with tempfile.TemporaryDirectory() as d:
        # differences' standard errors from the fit: random-b (b scores 75%) and b-c (50%), 100 games each, at the
        # fit's ratings. Each link's variance is ELO^2 / (n p (1 - p)), and a chain's add up.
        b = 400 * math.log10(3)
        with open(os.path.join(d, "games.csv"), "w") as f:
            f.write("tournament,white,black,result,plies\n" + "0,random,b,0-1,9\n" * 100 + "0,c,b,1-0,9\n" * 100)
        fr = {"random": (0.0, 1.0, 100), "b": (b, 1.0, 200), "c": (b, 1.0, 100), "unrated": (5.0, 1.0, 0)}
        fac = fit_factor(d, {k: v for k, v in fr.items() if k != "unrated"})
        elo2 = (400 / math.log(10)) ** 2
        for a_, b_, var in [(["b"], ["random"], elo2 / 18.75), (["c"], ["b"], elo2 / 25),
                            (["c"], ["random"], elo2 / 18.75 + elo2 / 25), (["random"], ["c"], elo2 / 18.75 + elo2 / 25),
                            (["b", "c"], ["random"], elo2 / 18.75 + elo2 / 100)]:
            assert abs(contrast_se(fac, a_, b_) ** 2 - var) < 1e-6 * var, (a_, b_, contrast_se(fac, a_, b_) ** 2, var)
        # the saved-games reader, and each lineage's games across its entries.
        for t, w, b, lines in [(0, "a-2", "b-1", 3), (1, "b-1", "a-2", 2), (0, "c", "b-1", 1)]:
            g = os.path.join(d, "pool", f"round{t}", f"{w}~{b}")
            os.makedirs(g)
            for i, name in enumerate(OUTCOMES):
                with open(os.path.join(g, name), "w") as f:
                    f.write("1 2 3\n" * (lines if i == 0 else 0))
        os.makedirs(os.path.join(d, "pool", "round0", "junk"))
        games = saved_games(os.path.join(d, "pool"))
        assert [g[:3] for g in games] == [(0, "a-2", "b-1"), (0, "c", "b-1"), (1, "b-1", "a-2")], games
        assert sum(count_positions(g[3]) for g in games) == 6
        assert [g[:3] for g in games_of(games, ["a-2", "b-1"], ["a-2", "b-1", "c"])] == [g[:3] for g in games]
        assert [g[:3] for g in games_of(games, ["c"], ["b-1"])] == [(0, "c", "b-1")]
        # only games against the opponents given, which include the lineage's own entries
        assert [g[:3] for g in games_of(games, ["a-2"], ["b-1"])] == [(0, "a-2", "b-1"), (1, "b-1", "a-2")]
        assert games_of(games, ["b-1"], ["a-2"]) == games_of(games, ["a-2"], ["b-1"])
        assert games_of(games, ["c"], ["a-2"]) == []
        # the league update: retired players move to retired.txt, games and model rows stay; reruns change nothing.
        league = os.path.join(d, "league")
        os.makedirs(league)
        with open(os.path.join(league, "players.txt"), "w") as f:
            f.write("# header\nrandom 0 rand\nref-1 1 lr:r.model\nx-v1-1 1 lr:x1.model\nx-v1-2 2 lr:x1.model\n")
        with open(os.path.join(league, "models.csv"), "w") as f:
            f.write("name,archetype\nrandom,random\nref,F-L\nx-v1,F-L\n")
        cfg = Config.__new__(Config)
        cfg.raw = dict(league_dir=league, tournament=dict(retired=os.path.join(league, "retired.txt")))
        cfg.root = d
        for _ in range(2):
            moved = update_league(cfg, ["x-v1-1", "x-v1-2"], ["x-v2-1 1 lr:x2.model", "x-v2-2 2 lr:x2.model"],
                                  [dict(name="x-v2", archetype="F-L")])
        assert [n for n, _, _ in read_players(league)] == ["random", "ref-1", "x-v2-1", "x-v2-2"]
        assert [n for n, _, _ in read_entries(os.path.join(league, "retired.txt"))] == ["x-v1-1", "x-v1-2"]
        with open(os.path.join(league, "models.csv")) as f:
            assert [r["name"] for r in csv.DictReader(f)] == ["random", "ref", "x-v1", "x-v2"]
        assert moved == 0  # the second time round there was nothing left to move
        # the league setup's removals: each game to the first archive whose players it involves, nothing lost, and
        # a rerun changes nothing.
        with open(os.path.join(league, "players.txt"), "w") as f:
            f.write("# header\nrandom 0 rand\na-1 1 s\na-2 2 s\na-3 3 s\noff-1 1 s\noff-2 2 s\noff-3 3 s\n"
                    "dup-1 1 s\nkeep-2 2 s\n")
        with open(os.path.join(league, "models.csv"), "w") as f:
            f.write("name,archetype\nrandom,random\na,F-L\noff,F-L\ndup,F-L\nkeep,F-L\n")
        names = [n for n, _, _ in read_players(league)]
        rows = [f"{t},{w},{b},1-0,10\n" for t in range(2) for w in names for b in names if w != b]
        with open(os.path.join(league, "games.csv"), "w") as f:
            f.write("tournament,white,black,result,plies\n" + "".join(rows))
        arc = lambda name: os.path.join(league, name)
        cfg.raw["league_setup"] = dict(remove=[
            dict(what="3-ply", depths=[3], players=arc("p3.txt"), games=arc("g3.csv")),
            dict(what="offline", models=["off"], players=arc("po.txt"), games=arc("go.csv"), model_rows=arc("mo.csv")),
            dict(what="duplicate", models=["dup"], players=arc("pd.txt"), games=arc("gd.csv"), model_rows=arc("md.csv"))])
        for _ in range(2):
            remove_players(cfg)
        assert [n for n, _, _ in read_players(league)] == ["random", "a-1", "a-2", "keep-2"]
        assert [n for n, _, _ in read_entries(arc("po.txt"))] == ["off-1", "off-2"]
        read = lambda name: open(arc(name)).read().splitlines()[1:]
        assert sorted(read("games.csv") + read("g3.csv") + read("go.csv") + read("gd.csv")) == sorted(
            r.strip() for r in rows)
        assert all("-3" in r for r in read("g3.csv")) and all("off" in r and "-3" not in r for r in read("go.csv"))
        assert len(read("games.csv")) == 2 * 4 * 3 and len(read("gd.csv")) == 2 * 2 * 4
        assert read("models.csv") == ["random,random", "a,F-L", "keep,F-L"] and read("mo.csv") == ["off,F-L"]
        # the side-file trimming.
        p = os.path.join(d, "ladder.csv")
        with open(p, "w") as f:
            f.write(LADDER_HEADER + "1,1,0,1,1,1,0.5,0,-1,1\n2,2,1,1,1,1,0.5,0,-1,1\n3,3")
        trim_side_file(p, 2)
        with open(p) as f:
            assert f.read() == LADDER_HEADER + "1,1,0,1,1,1,0.5,0,-1,1\n"
        p = os.path.join(d, "results.txt")
        with open(p, "w") as f:
            f.write("round 0 white wins: 1\nround 1 white wins: 2\nround 2 white")
        trim_side_file(p, 1)
        with open(p) as f:
            assert f.read() == "round 0 white wins: 1\n"
    import types
    ob = types.SimpleNamespace(own_games=dict(binary="b/default", binary_by_lineage={"X": "b/x"}), abs=lambda q: "/r/" + q)
    assert own_binary(ob, dict(name="X")) == "/r/b/x" and own_binary(ob, dict(name="Y")) == "/r/b/default"
    ob.own_games.pop("binary_by_lineage")
    assert own_binary(ob, dict(name="X")) == "/r/b/default"
    tr = lambda line: model_args(dict(flags=parse_flags(line.split())))  # the own step's train keeps the labels
    assert tr("--model lr --use-probs --seed 21 --vs-previous --no-games --policy log-odds-softmax --temperature 0.3 "
              "--depth 4 --label-depth 2 --label-result 0.5") == ["--use-probs", "--label-depth", "2", "--label-result", "0.5"]
    assert tr("--model bdt --leaf cnbp --router cnbp --seed 21 --vs-previous --no-games --depth 2 --use-probs") == [
        "--leaf", "cnbp", "--router", "cnbp", "--use-probs"]
    assert rule_depths([dict(depth=0), dict(depth=4)]) == [0, 1, 2, 3, 4] and rule_depths([dict(depth=2)]) == [0, 1, 2, 3]
    # self-play in chunks (the user, 2026-11-07): each re-planned from the last one's cost, so drift is corrected
    def simulate(cost, size, start=0, restart_at=None, overhead=0.0, budget=240.0):
        """A job against a fake run: a command plays rounds at cost(n) CPU s per game. With restart_at, the driver
        dies during that command after 2 of its rounds (their CPU spent, unrecorded) and the job restarts."""
        with tempfile.TemporaryDirectory() as d:
            for n in range(1, start + 1):
                open(os.path.join(d, f"lr{n}.model"), "w").close()
            cf = Config.__new__(Config)
            cf.raw = {}
            ln, p = dict(run_dir=d, kind="lr"), dict(rounds=1, size=size, start=start, cost=cost(start + 1),
                                                   base=["sp"], init=[], budget=budget)
            spent, cmds, result = 0.0, 0, None
            while True:
                cmd = next_step(cf, ln, p, result)
                if cmd is None:
                    return p, spent, cmds
                cmds += 1
                assert cmds < 40, p
                v, target = last_version(d, "lr"), int(cmd[cmd.index("--rounds") + 1])
                cut = restart_at == cmds and target - v >= 3
                cpu = overhead
                for n in range(v + 1, v + 3 if cut else target + 1):
                    cpu += cost(n) * size
                    open(os.path.join(d, f"lr{n}.model"), "w").close()
                spent += cpu
                result = None if cut else (0, cpu, cpu)
    falls = lambda n: 5.25 if n == 1 else 2.8  # F-L-d4mix-c from scratch in cycle 13: 165 s by the one-plan rule
    p, spent, cmds = simulate(falls, 10)
    assert abs(spent - 240) <= 0.5 * 28 and cmds <= 8 and p["end"] == p["start"] + p["total"] == 8, (spent, cmds, p)
    assert all(len(x) == 6 and x[4] >= 1 and x[5] > 0 for x in p["parts"] if x[0] == "chunk"), p["parts"]
    rises = lambda n: 0.002 + 0.00012 * (n - 300)  # a growing tree: 2 ms a game rising to about 6 (one plan: 1.6x)
    p, spent, cmds = simulate(rises, 1000, start=300)
    last = p["parts"][-1]
    assert abs(spent - 240) <= 0.5 * rises(300 + p["total"]) * 1000 + last[4] * 0.00012 * 1000 * last[4], (spent, p)
    assert cmds <= 8 and p["parts"][0][0] == "measure" and last[0] == "chunk", (cmds, p["parts"])
    p, spent, cmds = simulate(rises, 1000, start=300, restart_at=3)  # a restart mid-chunk: its rounds at the estimate
    assert p["cut_cpu"] > 0 and abs(spent - 240) <= 15, (spent, p)
    p, spent, cmds = simulate(lambda n: 0.0004, 1000, overhead=0.5)  # flat LR: about 600 rounds, few commands
    assert abs(spent - 240) <= 1 and cmds <= 7, (spent, cmds, p["parts"])
    # a probit lineage (F-P-c, the user, 2026-11-07): plain probitN.model files like lr's, which hold all its
    # training state (not a tree kind: no .state files): its job starts from scratch without --resume, then resumes
    # from the last probitN.model, and its own step trains --start from that model file.
    with tempfile.TemporaryDirectory() as d:
        cf = Config.__new__(Config)
        cf.raw = {}
        ln, p = dict(run_dir=d, kind="probit"), dict(rounds=1, size=1000, start=0, cost=0.0024, base=["sp"], init=[],
                                                     budget=240.0)
        cmds, result = [], None
        while True:
            cmd = next_step(cf, ln, p, result)
            if cmd is None:
                break
            cmds.append(cmd)
            assert len(cmds) < 20, cmds
            v, target = last_version(d, "probit"), int(cmd[cmd.index("--rounds") + 1])
            for n in range(v + 1, target + 1):
                open(os.path.join(d, f"probit{n}.model"), "w").close()
            result = (0, 2.4 * (target - v), 2.4 * (target - v))
        assert len(cmds) > 1 and "--resume" not in cmds[0] and all("--resume" in c for c in cmds[1:]), cmds
        assert last_version(d, "probit") == p["end"] and abs(2.4 * p["end"] - 240) <= 2.4, p
    assert "probit" not in TREE_KINDS and signature("probit", parse_flags(
        "--model probit --use-probs --seed 21 --vs-previous --no-games".split())) == "probit d0 jitter10"
    # the budget's round granularity (the user, 2026-11-07): a lineage whose round costs 2.6 shares skips a cycle
    # while its budget is under half a round and carries it; one whose rounds cost a tenth of a share plays its
    # budget each cycle. Over 40 cycles each spends its share a cycle, to within a round over the 40.
    for round_cost in (2.6 * 63.0, 6.3):
        st, spent = dict(patience_cpu=0.0, carry_cpu=0.0), 0.0
        for c in range(40):
            b = budgets(63.0 * 57, {n: (st["carry_cpu"] if n == "x" else 0.0) for n in [f"l{i}" for i in range(56)]
                                    + ["x"]})[1]["x"]
            if too_small(b, round_cost / 1000, 1000):
                settle(st, b, 0.0)
                continue
            p, used, _ = simulate(lambda n: round_cost / 1000, 1000, start=1 + c, budget=b)
            spent += used
            settle(st, b, used)
        assert abs(spent / 40 - 63.0) <= round_cost / 40 + 0.5, (round_cost, spent / 40)
        assert st["patience_cpu"] == round(spent, 1) or abs(st["patience_cpu"] - spent) < 0.1 * 40, st
    st = dict(patience_cpu=100.0, carry_cpu=5.0)
    settle(st, 70.0, 75.5)
    assert st == dict(patience_cpu=175.5, carry_cpu=-5.5), st
    settle(st, 70.0, 12.0, failed=True)
    assert st == dict(patience_cpu=187.5, carry_cpu=0.0), st
    # calibration "own" (off by default): the specs and records, the recipe, a seed directory's games, the field's
    # clean-up and a gate's post games
    with tempfile.TemporaryDirectory() as d:
        cpath = os.path.join(d, "config.json")

        def config(**raw):
            with open(cpath, "w") as f:
                json.dump(dict(root=d, nice=19, league_dir="league", tournament=dict(binary="bin/t"), **raw), f)
            return Config(cpath)
        off, on = config(), config(calibration="own", own_calibration=dict(dir="own", seed_dirs=["seed"]))
        assert not own_cal(off) and not own_cal(config(calibration="heldout")) and own_cal(on)
        try:
            own_cal(config(calibration="mine"))
            assert False, "an unknown calibration must stop the driver"
        except SystemExit:
            pass
        run, root = os.path.join(d, "runs", "A"), os.path.join(d, "own")
        os.makedirs(run)
        for v in (1, 2, 9):
            with open(os.path.join(run, f"lr{v}.model"), "w") as f:
                f.write(f"lr {v}\n")
        lin = dict(name="A", kind="lr", run_dir=run, heldout=os.path.join(d, "ho"))
        p1, p2 = model_path(lin, 1), model_path(lin, 2)
        assert league_spec(off, lin, 2) == spec(lin, p2) and not os.path.exists(root)  # off: as before, no files
        s2 = league_spec(on, lin, 2)
        assert s2 == f"cal:{root}/A-v2:lr:{p2}" and own_key_of(s2, root) == "A-v2"
        assert own_key_of(spec(lin, p2), root) is None
        with open(os.path.join(root, "A-v2.json")) as f:
            assert json.load(f) == dict(play=spec(lin, p2), model=p2, md5=file_md5(p2))
        os.makedirs(os.path.join(root, "A-v2"))
        assert league_spec(on, lin, 2) == s2 and os.path.isdir(os.path.join(root, "A-v2"))  # the same record: kept
        assert own_games_for(on, ["A-v2"]) == (0, 0, 0.0)  # made already
        with open(p2, "a") as f:
            f.write("changed\n")
        try:
            own_games_for(on, ["A-v2"])
            assert False, "a changed model file must stop the driver"
        except SystemExit:
            pass
        league_spec(on, lin, 2)  # a new record for the changed file: its old games are deleted
        assert not os.path.exists(os.path.join(root, "A-v2"))
        assert own_games_cmd(on, "P", "OUT") == ["nice", "-n", "19", os.path.join(d, "bin/t"), "arena", "P", "P",
                                                 "--use-probs", "--games", "600", "--a-policy", "log-odds-softmax",
                                                 "--b-policy", "log-odds-softmax", "--temperature", "0.1",
                                                 "--a-depth", "1", "--b-depth", "1", "--seed", "5", "--out", "OUT"]
        # a seed directory's games are reused (linked) only if its log shows this recipe and the same spec
        play1 = spec(lin, p1)
        src = os.path.join(d, "seed", "A-v1")
        os.makedirs(src)
        for name in OWN_FILES:
            with open(os.path.join(src, name), "w") as f:
                f.write(name)
        for head, ok in [(own_header(on, play1).replace("seed 5", "seed 6"), False),
                         (own_header(on, spec(lin, p2)), False), (own_header(on, play1), True)]:
            with open(src + ".arena", "w") as f:
                f.write(f"# cmd\n{head}\nA: +200 -150 =250, score 54.2%\n")
            assert (seed_match(on, "A-v1", play1) == src) == ok, head
        t = os.path.getmtime(src + ".arena")
        os.utime(p1, (t + 10, t + 10))
        assert seed_match(on, "A-v1", play1) is None  # the model file rewritten after its seed games: not reused
        os.utime(p1, (t - 10, t - 10))
        s1 = league_spec(on, lin, 1)
        assert own_games_for(on, ["A-v1"]) == (0, 1, 0.0)  # reused, not played
        assert all(os.path.samefile(os.path.join(src, n), os.path.join(root, "A-v1", n)) for n in OWN_FILES)
        # a calibrated reference plays on its own games too, keyed by its model name; others are left alone
        ref = os.path.join(d, "ref.model")
        with open(ref, "w") as f:
            f.write("ref\n")
        assert own_reference(on, f"ref-x-1 1 cal:H:lr:{ref}") == f"ref-x-1 1 cal:{root}/ref-x:lr:{ref}"
        assert own_reference(on, "random-4 0 random") == "random-4 0 random"
        # the field's clean-up: own games of versions not in players.txt go (half-made ones too); records stay
        os.makedirs(os.path.join(d, "league"))
        with open(os.path.join(d, "league", "players.txt"), "w") as f:
            f.write(f"A-v1-1 1 {s1}\nA-v1-2 2 {s1}\nrandom 0 random\n")
        for k in ("A-v2", ".tmp-A-v9", ".post-B"):
            os.makedirs(os.path.join(root, k))
        assert field_own_keys(on) == ["A-v1", "A-v1"] and own_gc(on) == 3
        assert sorted(n for n in os.listdir(root) if os.path.isdir(os.path.join(root, n))) == ["A-v1"]
        assert os.path.exists(os.path.join(root, "A-v2.json")) and os.path.exists(os.path.join(root, "A-v1.arena"))
        # a gate's post: its own games and records go after the gate
        trained = os.path.join(d, "trained.model")
        with open(trained, "w") as f:
            f.write("lr post\n")
        register_own(on, ".post-A", spec(lin, trained), trained)
        os.makedirs(os.path.join(root, ".post-A"))
        with open(os.path.join(root, ".post-A.arena"), "w") as f:
            f.write("log\n")
        own_drop(on, ".post-A")
        assert not any(n.startswith(".post-A") for n in os.listdir(root)), os.listdir(root)
    # the gate plays the league's policy (LEAGUE_POLICY, the tournament binary's defaults), not the arena's default
    # (wld-score-jitter10, self-play's): the gate bug, fixed 2026-11-08
    with tempfile.TemporaryDirectory() as d:
        cpath = os.path.join(d, "config.json")
        with open(cpath, "w") as f:
            json.dump(dict(root=d, nice=19, gate=dict(games=100, depth=2, threshold=0.5, seed=1),
                           own_games=dict(binary="bin/own", binary_by_lineage={"B": "bin/b"})), f)
        gc = Config(cpath)
        for name, binary in (("A", "bin/own"), ("B", "bin/b")):
            assert gate_cmd(gc, dict(name=name), "P", "Q", 4) == [
                "nice", "-n", "19", os.path.join(d, binary), "arena", "P", "Q", "--use-probs", "--games", "100",
                "--a-depth", "2", "--b-depth", "2", "--seed", "5", "--a-policy", "log-odds-softmax", "--b-policy",
                "log-odds-softmax", "--temperature", "0.1"], gate_cmd(gc, dict(name=name), "P", "Q", 4)
        assert LEAGUE_POLICY == dict(policy=OWN_CAL_DEFAULTS["policy"], temperature=OWN_CAL_DEFAULTS["temperature"])
    # a solo lineage (steps -): no own step, its row's gate says so; an active lineage with the step on and no games
    # this cycle, and a frozen solo one, are left alone; with own_games.enabled off for all, nobody is marked solo
    sc = types.SimpleNamespace(own_games=dict(enabled=True))
    for enabled, want in ((True, {"S": dict(gate=SOLO), "O": {}, "X": {}}), (False, {"S": {}, "O": {}, "X": {}})):
        sc.own_games["enabled"] = enabled
        sst = dict(lineages={"S": dict(status="active", entries=["S-v1-1"]),
                             "O": dict(status="active", entries=["O-v1-1"]),
                             "X": dict(status="frozen", entries=["X-v1-1"])})
        srows, sdone = {"S": {}, "O": {}, "X": {}}, []
        own_steps(sc, sst, [dict(name="S", steps=set()), dict(name="O", steps={"own"}), dict(name="X", steps=set())],
                  None, 7, srows, [], sdone, {"O-v1-1", "S-v1-1"})
        assert srows == want and sdone == [], (enabled, srows, sdone)
    print("selftest: the champion test, the revival statistic, milestones, the budget split and its carry, the "
          "standard errors from the fit's covariance, copying a champion to resume from, the old peak, the gate's "
          "arithmetic, lineage ratings, saved games, league updates, removals, trimming, self-play chunks, the own "
          "step's labels, probit's model files, calibration \"own\", the gate's policy and solo lineages OK")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--config", default=os.path.join(HERE, "config.json"))
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("status")
    s = sub.add_parser("dry-run")
    s.add_argument("--cycles", type=int, default=50)
    s = sub.add_parser("measure")
    s.add_argument("--force", action="store_true")
    sub.add_parser("setup-league")
    sub.add_parser("refit")
    s = sub.add_parser("init")
    s.add_argument("--force", action="store_true")
    s = sub.add_parser("run")
    s.add_argument("--cycles", type=int, default=1)
    s = sub.add_parser("verify")
    s.add_argument("names", nargs="+")
    s.add_argument("--rounds", type=int, default=1)
    s.add_argument("--cycle", type=int, help="replay the lineages' self-play of this cycle, from cycles.csv")
    sub.add_parser("selftest")
    for c in ("freeze", "unfreeze", "retire", "unretire"):
        s = sub.add_parser(c)
        s.add_argument("name")
    a = ap.parse_args()
    if a.cmd == "selftest":
        return selftest()
    cfg = Config(a.config)
    if a.cmd == "status":
        cmd_status(cfg)
    elif a.cmd == "dry-run":
        cmd_dry_run(cfg, a.cycles)
    elif a.cmd == "measure":
        cmd_measure(cfg, a.force)
    elif a.cmd == "setup-league":
        cmd_setup(cfg)
    elif a.cmd == "refit":
        cmd_refit(cfg)
    elif a.cmd == "init":
        cmd_init(cfg, a.force)
    elif a.cmd == "run":
        cmd_run(cfg, a.cycles)
    elif a.cmd in ("freeze", "unfreeze"):
        cmd_freeze(cfg, a.name, a.cmd == "unfreeze")
    elif a.cmd == "retire":
        cmd_retire(cfg, a.name)
    elif a.cmd == "unretire":
        cmd_unretire(cfg, a.name)
    elif a.cmd == "verify":
        sys.exit(0 if (verify_cycle(cfg, a.names, a.cycle) if a.cycle is not None else
                       cmd_verify(cfg, a.names, a.rounds)) else 1)


if __name__ == "__main__":
    main()
