#!/usr/bin/env python3
"""Training/tournament cycles (the user's design, 2026-11-06; see README.md).

A cycle: (1) self-play trains every qualified lineage for the same CPU budget, measured in the loop itself:
each job first plays a measuring segment (its next rounds, at least 5% of the budget), then the rounds that fill the
budget at the CPU per game it measured; (2) each lineage's newest version
replaces its entries in the cycles' league (config league_dir: selfplay/league-cycles), at plies 1 and 2; the old
ones retire (they stop playing, but their games stay in the fit); (3) the whole current field plays fresh double round-robins; (4) each
lineage's new version is compared with its predecessor in that fit, by their 2-ply entries, which decides who trains
next cycle (a dormant lineage trains again once its newest version beats its best earlier one, in that fit, by more
than its ±); (5) every lineage with the own-games step trains once more on its entries' games from that tournament and
keeps the result only if it scores at least 50% against the model it started from (the gate). Cycle 0 makes the
league (the random players random, random-1 and random-2) and records every lineage as new.

    cycle.py [--config FILE] status                show the lineages, their state and the next cycle's plan
    cycle.py [--config FILE] measure [--force]     measure the CPU cost per game of every recipe into costs.csv
    cycle.py [--config FILE] setup-league          make the league the cycles' (once; safe to repeat): archive the
                                                   players league_setup removes with their games, add its new
                                                   players, note the new rules in players.txt and refit
    cycle.py [--config FILE] refit                 rewrite the league's reports with the cycles' rating fit (plays
                                                   nothing)
    cycle.py [--config FILE] init [--force]        cycle 0: setup-league, record the league's ratings, and run the
                                                   own-games step on the games saved from its last round-robin
                                                   (own_games.first_pool; one is played if none are saved there)
    cycle.py [--config FILE] run [--cycles N]      run N cycles (default 1)
    cycle.py [--config FILE] verify NAME... [--rounds R | --cycle C]
                                                   replay a continued run's last R rounds, or the lineages'
                                                   self-play of cycle C (from cycles.csv; cycle 2 on), with the
                                                   lineage's binary and compare the models byte for byte
    cycle.py selftest                              test the qualification rules, the gate's arithmetic, the league
                                                   updates and the saved-games reader
"""
import argparse
import csv
import datetime
import fcntl
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

HERE = os.path.dirname(os.path.abspath(__file__))
LADDER_HEADER = "round,version,opponent,wins,losses,draws,score,elo,elo_low,elo_high\n"
OUTCOMES = ("white_wins.games", "black_wins.games", "draws.games")
TREE_KINDS = {"bdt", "bdd", "ht", "tan"}  # resume and train --start from {kind}N.state, not the model file (TAN's
# state holds its pair counts, which its model file lacks)
STEPS = {"own"}
MEASURE_MIN, MEASURE_AIM = 0.05, 0.06  # a self-play job's measuring segment: at least 5% of its budget (aims at 6%)
CHUNK = 0.22  # then chunks of about this share of the budget, each re-planned from the last one's cost (the user)
DRIVER_FLAGS = {"--rounds", "--games", "--threads", "--resume", "--init"}  # set by the driver, never by the registry
NOT_COST = {"--seed", "--no-games", "--vs-previous", "--use-probs", "--model", "--depth", "--policy", "--temperature",
            "--label-depth", "--label-result"}
SELFPLAY_ONLY = {"--model", "--seed", "--rounds", "--games", "--policy", "--temperature", "--threads", "--vs-previous",
                 "--no-games", "--depth", "--round-forget", "--age-forget", "--resume", "--keep-states", "--init"}
LABEL_FLAGS = {"--label-depth", "--label-result", "--label-model"}  # the own step's train takes them too (the user)
TOURNAMENT_FLAGS = ["--no-catch-up", "--retired", "--save-games"]  # the cycles need all of them
LOG_FIELDS = ["cycle", "date", "lineage", "status", "version", "games", "rounds", "round_size", "cpu_s", "wall_s", "cost_ms", "segment_rounds", "start_version", "end_version",
              "entries", "elo", "ci95", "prev_elo", "peak_elo", "peak_version", "pre_dormancy_peak_elo",
              "pre_dormancy_peak_version", "rel_elo", "strikes", "decision", "own_games", "own_positions",
              "own_cpu_s", "gate_wins", "gate_losses", "gate_draws", "gate_score", "gate_elo", "gate_ci95",
              "gate_cpu_s", "gate", "next_version"]
COST_FIELDS = ["signature", "cpu_per_game", "games", "rounds", "threads", "cpu_s", "wall_s", "binary", "models",
               "date", "loadavg"]
NOTE_START, NOTE_END = "# Since cycles/cycle.py setup-league", "# Don't run ./run.sh any more"
MODELS_HEADER = "name,archetype,variant,games,moves,train_depth,labels,init,rounds,seed,calibrated,run_dir,script\n"
RETIRED_HEADER = ("# Retired players (cycles/cycle.py): NAME DEPTH SPEC, as in players.txt. They play no more games, "
                  "but their games\n# stay in games.csv and in the fit (tournament --retired).\n")


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


def plan(cfg, costs, lin, st):
    """The next cycle's self-play for a prepared lineage, or (None, reason). The job measures its own cost when it
    runs (see next_step): the plan's rounds are only an estimate, from the lineage's last measured cost or else the
    cost table, for the status, the first measuring segment's size and the dashboard's ETA."""
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
    rounds = max(1, int(cfg.budget_cpu_s / cost / size + 0.5))
    v = st["version"] or 0
    if v == 0 or (lin["kind"] == "moe" and v < 2):
        rounds = max(rounds, 2 - v)  # --resume needs two versions: the model and its opponent
    base = ["nice", "-n", str(cfg.nice), lin["binary"], "self-play", lin["run_dir"], *lin["tokens"],
            "--games", str(size), "--threads", str(cfg.threads_per_job)]
    init = ["--init", lin["source"]] if lin["source_type"] == "init" else []
    return dict(rounds=rounds, size=size, games=rounds * size, start=v, cost=cost, cost_source=source, base=base,
                init=init), None


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
    """A lineage's self-play job for this cycle, one command at a time. `p` is the job in state.json (the caller
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
    run_dir, kind, size, budget = lin["run_dir"], lin["kind"], p["size"], float(cfg.budget_cpu_s)
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


def lineage_peak(ratings, depths, depth, name, current, ref, upto=None):
    """A lineage's best earlier version in this fit: (rel, version, entries) of the highest-rated of its versions by
    their
    entries <name>-v<N>-<ply> at ply `depth`, retired ones included (they stay in the fit), but not the current
    version (`current`: its entries); None if none is rated. rel is Elo minus `ref`, the references' mean in this
    fit, as for the lineage's own rating, so that references added later (fixed anchors, say) count here too. With
    `upto`, only versions numbered up to it count (a dormant lineage's bar: its peak from before it went dormant)."""
    pattern = re.compile(re.escape(name) + r"-v(\d+)-\d+")
    skip = {int(m.group(1)) for m in map(pattern.fullmatch, current) if m}
    versions = {}
    for player in ratings:
        m = pattern.fullmatch(player)
        if m and depths.get(player) == depth and int(m.group(1)) not in skip and (
                upto is None or int(m.group(1)) <= upto):
            versions.setdefault(int(m.group(1)), []).append(player)
    best = None
    for v in sorted(versions):
        rel = rating_of(ratings, versions[v])[0] - ref
        if best is None or rel > best[0]:
            best = (rel, v, versions[v])
    return best


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


def peak_compare(ratings, depths, depth, name, entries, ref, factor, upto=None):
    """A lineage's current version (its `entries`) against its peak (lineage_peak), in this fit: a dict with gap
    (its rel minus the peak's), se (that difference's standard error from the fit's covariance, `factor`; without
    one, the two entries' ± combined as if independent), and the peak's version, rel and Elo. None if the current
    version or every earlier one is unrated at ply `depth`. `upto`: see lineage_peak (a dormant lineage's bar)."""
    now = rating_at(ratings, entries or [], depths, depth)
    peak = lineage_peak(ratings, depths, depth, name, entries or [], ref, upto)
    if not now or not peak:
        return None
    mine = [n for n in entries or [] if depths.get(n) == depth and n in ratings]
    peak_r = rating_of(ratings, peak[2])
    se = contrast_se(factor, mine, peak[2]) if factor and mine else math.hypot(now[1], peak_r[1]) / 1.96
    return dict(gap=now[0] - ref - peak[0], se=se, version=peak[1], rel=peak[0], elo=peak_r[0], upto=upto)


def se_multipliers(cfg):
    """(dormancy_se, requalify_se): how many standard errors below its peak strike a qualified lineage, and how
    many above it re-qualify a dormant one."""
    return float(cfg.raw.get("dormancy_se", 2)), float(cfg.raw.get("requalify_se", 2))


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
    files = lambda r: ", ".join(os.path.basename(r[k]) for k in ("players", "games", "model_rows") if k in r)
    removed = "; ".join(f"{r['what']} ({files(r)})" for r in cfg.league_setup.get("remove", []))
    plies = " and ".join(map(str, cfg.tournament["entry_depths"]))
    text = (f"{NOTE_START[2:]} ({now()[:10]}), the training/tournament cycles (cycles/README.md) run this league, "
            f"and these rules replace those above. Each cycle plays fresh round-robins of the players listed here "
            f"(tool flag --no-catch-up): a newcomer plays from its first cycle on, and nobody catches up on earlier "
            f"round-robins. The fit is pure maximum likelihood with random at 0 (--prior-draws 0). A cycle lineage's "
            f"entries are <lineage>-v<N>-<ply>, at plies {plies}; a new version's entries replace the old ones, which "
            f"move to retired.txt: they play no more, but their games stay in games.csv and in the fit (--retired). "
            + (f"Removed with their games, archived and never deleted: {removed}. " if removed else "")
            + "To add a reference model: append its lines here and its row to models.csv; it joins the next cycle.")
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
                    f.write(f"# Removed from the league by cycles/cycle.py setup-league ({now()}): {rule['what']}. "
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
        atomic_write(players_path, f"# A league for the training/tournament cycles, made by cycles/cycle.py "
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
    lacks = [f for f in TOURNAMENT_FLAGS if not supports(b, "tournament", f)]
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
    atomic_write(os.path.join(league, "pid.txt"), f"{os.getpid()}\n")
    cmd = ["nice", "-n", str(cfg.nice), cfg.abs(t["binary"]), "tournament", "players.txt", ".", "--tournaments",
           str(tournaments), "--use-probs", "--anchor", "random", "--seed", str(t["seed"]), "--no-catch-up",
           "--retired", retired_path(cfg), "--prior-draws", "0"]
    if save:
        os.makedirs(save, exist_ok=True)
        cmd += ["--save-games", save]
    t0 = time.time()
    with open(os.path.join(league, "tournament.log"), "a") as log:
        log.write(f"start {now()}: cycles/cycle.py: {shlex.join(cmd)}\n")
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
    return state["lineages"].setdefault(name, dict(
        status="new", strikes=0, prepared=False, version=None, entry_version=None, field_version=None, entries=None,
        prev_entries=None, elo=None, ci=None, rel=None, elo_cycle=None, dormant_rel=None))


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


# ---------------------------------------------------------------- qualification

def decide(st, trained, now, prev, rel, patience, dormancy_se, requalify_se, cmp=None, version=None):
    """The user's rules (from 2026-11-06 ~21:00). Each lineage's current version is compared with its peak, the best
    of its earlier versions in the same (newest) fit: `cmp` (peak_compare: the gap and its standard error), or None
    if no earlier version is rated. A qualified lineage more than dormancy_se standard errors below its peak gets a
    strike, and `patience` strikes in a row make it dormant; otherwise its strikes reset (a new peak, or within
    range of the old one). A dormant lineage re-qualifies when it beats its peak by more than requalify_se
    standard errors. The standard error is the difference's, from the fit's covariance, which carries the shift a
    whole cycle shares; the predecessor test before it struck lineages on such shifts (9 went dormant in cycle 5).
    `now` and `prev`: (mean Elo, mean ±) of the lineage's entries and of its predecessor's, in the same fit; the
    predecessor is reported only. `rel`: now's Elo minus the references' mean. A dormant lineage's `cmp` is
    against its peak from before it went dormant (versions up to st["dormant_version"], the version that went
    dormant, which going dormant records as `version`), so versions its own steps make while dormant never raise
    its bar and small gains add up (the user, ~21:35). Returns the decision; updates st (status, strikes,
    dormant_rel, dormant_version)."""
    pred = f"; predecessor {now[0] - prev[0]:+.0f}" if now and prev else ""
    if st["status"] == "dormant":
        if rel is None:
            return "dormant"
        if cmp is None:
            return "dormant (no earlier version rated)"
        half = requalify_se * cmp["se"]
        before = f" before dormancy at v{cmp['upto']}," if cmp.get("upto") is not None else ""
        at = f"{cmp['gap']:+.0f} ± {half:.0f} vs peak v{cmp['version']}{before} at {cmp['rel']:.0f} vs refs"
        if cmp["gap"] > half:
            st["status"], st["strikes"], st["dormant_rel"], st["dormant_version"] = "qualified", 0, None, None
            return f"re-qualified ({at})"
        return f"dormant ({at}; needs > {half:+.0f})"
    if not trained:
        return "not trained"
    if now is None:
        st["status"], st["strikes"] = "qualified", 0
        return "qualified (no rating)"
    if cmp is None:
        st["status"], st["strikes"] = "qualified", 0
        return "qualified (first rating)" if prev is None else f"qualified (no earlier version rated{pred})"
    half = dormancy_se * cmp["se"]
    if cmp["gap"] >= -half:
        st["status"], st["strikes"] = "qualified", 0
        if cmp["gap"] > 0:
            return f"new peak ({cmp['gap']:+.0f} ± {half:.0f} over v{cmp['version']}{pred})"
        return f"ok ({cmp['gap']:+.0f} ± {half:.0f} vs peak v{cmp['version']}{pred})"
    st["strikes"] += 1
    at = f"{cmp['gap']:+.0f} ± {half:.0f} vs peak v{cmp['version']}{pred}"
    if st["strikes"] >= patience:
        st["status"], st["dormant_rel"], st["dormant_version"] = "dormant", rel, version
        return f"dormant (strike {st['strikes']}; {at})"
    st["status"] = "qualified"
    return f"strike {st['strikes']} ({at})"


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


def gate_match(cfg, lin, post, pre, cycle):
    """post vs pre at the gate's depth, calibrated like the league entries. Returns (w, l, d, cpu) or (None, why)."""
    g = cfg.gate
    cmd = ["nice", "-n", str(cfg.nice), own_binary(cfg, lin), "arena", spec(lin, post), spec(lin, pre),
           "--use-probs", "--games", str(g["games"]), "--a-depth", str(g["depth"]), "--b-depth", str(g["depth"]),
           "--seed", str(int(g["seed"]) + cycle)]
    code, cpu, _, out = run_measured(cmd, lin["run_dir"] + ".log", f"cycle {cycle} gate")
    m = re.search(r"^A: \+(\d+) -(\d+) =(\d+)", out, re.M)
    if code != 0 or not m:
        return None, f"gate failed (exit {code})"
    return (*map(int, m.groups()), cpu), None


def own_step(cfg, lin, st, games, cycle, row):
    """Trains the lineage's newest version once on `games`; keeps the result as its next version if it scores at
    least the gate's threshold against the version it started from."""
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
    result, why = gate_match(cfg, lin, trained, os.path.join(run_dir, f"{kind}{n}.model"), cycle)
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


def own_opponents(cfg, state):
    """Whose games against a lineage's entries the own-games step trains on: with own_games.opponents "lineages"
    (the user's choice), only the lineages' entries, its own included, so that the random players (and any other
    player that never trains) only measure; with "all", everyone still in the league. The lineages' entries are all
    those that played in the cycle's tournament, so they include the one a freeze at this cycle's decision has
    just taken out of the field (`frozen_retired`; a retired lineage keeps its entries): the user, 2026-11-07, as
    cycles 15 and 21 left those games out."""
    if cfg.own_games.get("opponents", "lineages") == "all":
        return set(league_depths(cfg))
    return {e for s in state["lineages"].values() for e in (s["entries"] or []) + (s.get("frozen_retired") or [])}


def own_steps(cfg, state, lineages, pool, cycle, rows, players, done):
    """The own-games step for every lineage with the switch on whose entries played in this tournament, except those
    in `done` (this cycle's, before an interruption)."""
    pool_games, opponents = saved_games(pool), own_opponents(cfg, state)
    for lin in lineages:
        st, row = lineage_state(state, lin["name"]), rows[lin["name"]]
        if lin["name"] in done or not own_on(cfg, lin) or not st["entries"]:
            continue
        games = games_of(pool_games, st["entries"], opponents)
        if not games or own_step_skipped(cfg, st):
            continue
        why = prepare(cfg, lin, st, players)
        if why:
            row["gate"] = f"step skipped: {why}"
        else:
            own_step(cfg, lin, st, games, cycle, row)
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


def write_summary(cfg, cycle, rows, extra, rel_refs=None):
    fmt = lambda x, f="{:+.0f}": "" if x in (None, "") else f.format(float(x))
    lines = [f"# Cycle {cycle}", ""]
    p = pooled(rows)
    if p:
        lines.append(f"**Own-games step, pooled over the lineages: {pooled_text(p)}.**")
        with open(cfg.file("log_csv")) as f:
            everything = pooled(list(csv.DictReader(f)))  # this cycle's rows are already in it
        lines += ["", f"All cycles so far: {pooled_text(everything)}."]
    elif any(own_on(cfg, l) for l in load_registry(cfg)):
        lines.append("**Own-games step: no gates this cycle.**")
    if any(r.get("status") == "frozen" for r in rows):
        extra = list(extra) + ["Frozen (the user): a frozen lineage keeps one entry in the field, at one ply, as a fixed "
                               "reference; it neither self-plays nor takes own steps or gates and gets no decision. Its "
                               "Elo is that entry's (at its ply: cycles.csv's entries), its peak the usual all-time one."]
    lines += ["", *extra, "",
              f"Elo: the lineage's {cfg.tournament['qualify_depth']}-ply entry (its entries at other plies play for "
              "information and as opponents); previous: its predecessor's, in the same fit; vs refs: Elo minus the "
              + (f"mean Elo of {', '.join(rel_refs)} in this fit" if rel_refs else "fixed references' mean") +
              ". Only players that stay in the field link one cycle's fit to the next, and through cycle 4 those "
              "were the random players alone (every lineage entered a new version each cycle), so comparisons "
              "across cycles are weak. From cycle 5, random-3 and random-4 (random moves searched 3 and 4 plies "
              "deep) play every cycle as references too and link the fits better; they stay out of that mean, so "
              "that vs refs keeps its meaning.", "",
              f"Decisions: each lineage's current version is compared with its peak, the best of its earlier "
              f"versions at ply {cfg.tournament['qualify_depth']} in this fit, retired ones included (peak and peak "
              f"version; Elo scale). A qualified lineage more than {se_multipliers(cfg)[0]:g} standard errors of that "
              f"difference below its peak gets a strike, and {cfg.patience} in a row make it dormant; otherwise its "
              "strikes reset (new peak: above it; ok: within range). A dormant lineage doesn't self-play but keeps "
              "its own-games step and gate, and re-qualifies when its current version beats its peak from before it "
              "went dormant (in parentheses: the best of its versions up to the one that went dormant, so versions "
              f"made while dormant never raise that bar) by more than {se_multipliers(cfg)[1]:g} standard errors. The ± in a decision is that many standard errors, from "
              "the fit's covariance, which carries the shift a whole cycle shares. The predecessor (previous) is "
              "shown for information only; until the peak rule replaced it (see README.md), a new version below "
              "its predecessor got a strike.",
              "",
              "| lineage | status | version | self-play | CPU s | Elo | ± | previous | peak | peak version "
              "| vs refs | decision | own games (positions) | gate | next |",
              "|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---|---|---|---:|"]
    for r in rows:
        sp = f"{r['rounds']} x {r['round_size']}" if r.get("rounds") else ""
        own = f"{r['own_games']} ({r['own_positions']})" if r.get("own_games") not in (None, "") else ""
        gate = r.get("gate") or ""
        if str(r.get("gate_wins", "")) != "":
            gate = (f"+{r['gate_wins']} -{r['gate_losses']} ={r['gate_draws']}, "
                    f"{100 * float(r['gate_score']):.1f}%, {fmt(r['gate_elo'])} ± {fmt(r['gate_ci95'], '{:.0f}')}: "
                    f"{r['gate']}")
        pre = r.get("pre_dormancy_peak_version") not in (None, "")  # decided while dormant: its bar in ()
        peak = fmt(r.get("peak_elo"), "{:.0f}") + (f" ({fmt(r.get('pre_dormancy_peak_elo'), '{:.0f}')})" if pre else "")
        peak_v = (f"v{r['peak_version']}" if r.get("peak_version") not in (None, "") else "") + \
            (f" (v{r['pre_dormancy_peak_version']})" if pre else "")
        lines.append(f"| {r['lineage']} | {r.get('status', '')} | {r.get('version') or ''} | {sp} "
                     f"| {fmt(r.get('cpu_s'), '{:.0f}')} | {fmt(r.get('elo'), '{:.0f}')} | {fmt(r.get('ci95'), '{:.0f}')} "
                     f"| {fmt(r.get('prev_elo'), '{:.0f}')} | {peak} | {peak_v} "
                     f"| {fmt(r.get('rel_elo'), '{:.0f}')} | {r.get('decision', '')} "
                     f"| {own} | {gate} | {r.get('next_version') or ''} |")
    atomic_write(os.path.join(cfg.file("log_dir"), f"cycle-{cycle:03d}.md"), "\n".join(lines) + "\n")


# ---------------------------------------------------------------- the freeze/retire rule

GOING_DORMANT = re.compile(r"dormant \((?:[^;()]*?,\s*)?strike \d")  # a decision that made a lineage dormant
FREEZE_RULE = dict(after_dormant=10, milestone_step=100, milestone_window=10, reference_window=50, exempt=[])


def freeze_rule_cfg(cfg):
    """config.json's freeze_rule over the defaults (the user's rule, 2026-11-06)."""
    r = dict(FREEZE_RULE)
    r.update(cfg.raw.get("freeze_rule") or {})
    return r


def went_dormant(rows):
    """The cycle a lineage went dormant, counted as the dashboard counts it: its latest decision that made it dormant,
    else the first of its latest run of dormant rows; None if neither. rows: {cycle: cycles.csv row}."""
    went = [c for c, r in rows.items() if GOING_DORMANT.match(str(r.get("decision") or ""))]
    if went:
        return max(went)
    c0 = None
    for c in sorted(rows, reverse=True):
        if rows[c].get("status") != "dormant":
            break
        c0 = c
    return c0


def round_robin_end(state, cycle):
    """The league's round-robin count at the end of a cycle's tournament (elo_history.csv's tournament column):
    recorded in state.json's round_robin_ends from the rule's first decision on; before, 5, 10 and 20 for cycles 1-3
    and 5c+10 from cycle 4 (5 a cycle)."""
    got = (state.get("round_robin_ends") or {}).get(str(cycle))
    return int(got) if got is not None else {1: 5, 2: 10, 3: 20}.get(cycle, 5 * cycle + 10)


def read_elo_history(league, tournaments):
    """{tournament: {player: Elo}} from elo_history.csv, for these round-robin counts only."""
    want, out = {int(t) for t in tournaments}, {}
    p = os.path.join(league, "elo_history.csv")
    if not want or not os.path.exists(p):
        return out
    with open(p, newline="") as f:
        for r in csv.DictReader(f):
            if int(r["tournament"]) in want:
                out.setdefault(int(r["tournament"]), {})[r["player"]] = float(r["elo"])
    return out


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


def rule_text(i, window):
    """The rule's outcome for cycles.csv's decision and the notes."""
    a, b = i["span"][0], i["span"][-1]
    if i["outcome"]:
        ply, m, avg = i["outcome"]
        return f"frozen at ply {ply} (avg {avg:.1f} over cycles {a}–{b}; milestone {m})"
    avgs = ", ".join(f"ply {p} {v:.1f}" for p, v in sorted(i["avg"].items()))
    return f"retired (avg {avgs or 'none'}; no untaken milestone within {window:g})"


def rule_survey(cfg, state, cycle, names, cur_rows=None, ratings=None):
    """The freeze/retire rule (the user, 2026-11-06) at `cycle`'s decision (cur_rows: its decided rows; None between
    cycles, `cycle` then being the last logged one). For each dormant lineage in `names`: the cycle it went dormant
    (c0), the cycles it has spent dormant since (n), its entries' mean Elo per ply over the last after_dormant of
    them, each cycle's as measured then (elo_history.csv at the cycle's last round-robin; this cycle's: `ratings`),
    whether the rule fires now (n >= after_dormant, not exempt, not fired before in this dormant spell) and what it
    would do (outcome; lineages due earlier claim first). And the milestones taken: by a frozen lineage's claim, or
    by a reference player (state's references) nearest to it within reference_window in the newest fit."""
    rc = freeze_rule_cfg(cfg)
    after, step, win = rc["after_dormant"], rc["milestone_step"], rc["milestone_window"]
    league = cfg.file("league_dir")
    if ratings is None:
        ratings = read_ratings(league)
    by = {}
    if "log_csv" in cfg.raw and os.path.exists(cfg.file("log_csv")):
        with open(cfg.file("log_csv"), newline="") as f:
            for r in csv.DictReader(f):
                if int(r["cycle"]) <= cycle:
                    by.setdefault(r["lineage"], {})[int(r["cycle"])] = r
    for name, r in (cur_rows or {}).items():
        by.setdefault(name, {})[cycle] = r
    taken = {}
    for ref in state.get("references") or []:
        if ref in ratings:
            m = nearest_milestone(ratings[ref][0], step)
            if abs(ratings[ref][0] - m) <= rc["reference_window"]:
                taken.setdefault(m, []).append(ref)
    for name, st in sorted(state["lineages"].items()):
        if st["status"] == "frozen" and st.get("claim") is not None:
            taken.setdefault(int(st["claim"]), []).append((st.get("entries") or [name])[0])
    info = {}
    for name in names:
        st = state["lineages"].get(name)
        if not st or st["status"] != "dormant":
            continue
        c0 = went_dormant(by.get(name, {}))
        if c0 is None:
            continue
        info[name] = dict(c0=c0, n=cycle - c0, applies_after=c0 + after, exempt=name in rc["exempt"],
                          span=list(range(max(c0 + 1, cycle - after + 1), cycle + 1)),
                          done=(st.get("rule") or {}).get("dormant_since") == c0)
    hist = read_elo_history(league, {round_robin_end(state, c) for i in info.values() for c in i["span"]
                                     if not (c == cycle and cur_rows is not None)})
    for name, i in info.items():
        got = {}
        for c in i["span"]:
            r = by[name].get(c)
            elo = ({n: x[0] for n, x in ratings.items()} if c == cycle and cur_rows is not None
                   else hist.get(round_robin_end(state, c), {}))
            for e in ((r or {}).get("entries") or "").split():
                if e in elo:
                    got.setdefault(e.rsplit("-", 1)[1], []).append(elo[e])
        i["avg"] = {p: sum(v) / len(v) for p, v in sorted(got.items())}
        i["fire"] = i["n"] >= after and not i["exempt"] and not i["done"]
    claimed = set(taken)
    waiting = [i for i in info.values() if not (i["fire"] or i["exempt"] or i["done"])]
    groups = [[(n, i["avg"]) for n, i in sorted(info.items()) if i["fire"]]] + \
        [[(n, i["avg"]) for n, i in sorted(info.items()) if i in waiting and i["applies_after"] == a]
         for a in sorted({i["applies_after"] for i in waiting})]
    out = resolve_claims(groups, claimed, step, win)
    for name, i in info.items():
        i["outcome"] = out.get(name)
        i["would"] = ("exempt" if i["exempt"] else "done this dormant spell" if i["done"] else
                      f"freeze {i['outcome'][0]} at {i['outcome'][1]}" if i["outcome"] else "retire")
    return dict(lineages=info, taken=taken, cfg=rc, cycle=cycle)


def rule_preview_lines(survey, markdown):
    """The rule's preview: each dormant lineage's count, running averages and what the rule would do now, and the
    milestones taken."""
    rc, info = survey["cfg"], survey["lineages"]
    taken = ", ".join(f"{m} {', '.join(t)}" for m, t in sorted(survey["taken"].items())) or "none"
    head = (f"Freeze/retire rule (the user): a lineage dormant {rc['after_dormant']} cycles is frozen at the ply "
            f"whose mean Elo over them is within {rc['milestone_window']:g} of an untaken multiple of "
            f"{rc['milestone_step']:g} (the closer ply; a tie, ply 2; closest lineage first), else retired. "
            f"Exempt: {', '.join(rc['exempt']) or 'none'}.")
    fmt = lambda i, p: f"{i['avg'][p]:.1f}" if p in i["avg"] else "-"
    span = lambda i, dash: f"{i['span'][0]}{dash}{i['span'][-1]}" if i["span"] else "-"  # none yet: went dormant now
    order = sorted(((n, i) for n, i in info.items() if not (markdown and i["fire"])),  # the note: fired ones above
                   key=lambda kv: (kv[1]["applies_after"], kv[0]))
    if markdown:
        lines = ["", f"**{head}** Dormant lineages, their running mean Elo per ply and what the rule would do now "
                 "(lineages due earlier claim first):", ""]
        if order:
            lines += ["| lineage | dormant | applies after | cycles | ply 1 | ply 2 | would |",
                      "|---|---:|---:|---|---:|---:|---|"]
            lines += [f"| {n} | {i['n']} of {rc['after_dormant']} | cycle {i['applies_after']} | "
                      f"{span(i, '–')} | {fmt(i, '1')} | {fmt(i, '2')} | {i['would']} |"
                      for n, i in order]
        else:
            lines.append("No lineage is dormant.")
        return lines + ["", f"Milestones taken (multiple: takers): {taken}.", ""]
    lines = [head, f"  {'lineage':24} {'dormant':>8} {'applies':>9} {'cycles':>7} {'ply 1':>7} {'ply 2':>7}  would"]
    lines += [f"  {n:24} {str(i['n']) + ' of ' + str(rc['after_dormant']):>8} {'after ' + str(i['applies_after']):>9} "
              f"{span(i, '-'):>7} {fmt(i, '1'):>7} {fmt(i, '2'):>7}  {i['would']}"
              for n, i in order]
    return lines + [f"  milestones taken: {taken}"]


def rule_refresh(cfg, state):
    """The freeze/retire rule's previews and milestones in state.json, recomputed between cycles after a status
    command (freeze, unfreeze, retire, unretire, wake), as of the last decision. Nothing fires here."""
    survey = rule_survey(cfg, state, state["cycle"], registry_names(cfg))
    for i in survey["lineages"].values():
        i["fire"] = False
    rule_apply(cfg, state, survey, {}, state["cycle"], [], 0)


def rule_apply(cfg, state, survey, rows, cycle, extra, n_players):
    """Applies the rule's outcomes for the lineages due at this decision, in state.json and their rows: a frozen one
    keeps the entry at its ply (cmd_freeze's per-entry freeze) and claims its milestone; a retired one gets status
    "retired" (the driver skips it; `unretire` brings it back). Records the preview and the milestones. Returns the
    entries to move to retired.txt (the own phase moves them, so the next round-robin excludes them)."""
    rc, retire, said = survey["cfg"], [], []
    for name, i in sorted(survey["lineages"].items()):
        st = state["lineages"][name]
        if not i["fire"]:
            st["rule_preview"] = dict(n=i["n"], of=rc["after_dormant"], applies_after=i["applies_after"],
                                      cycles=[i["span"][0], i["span"][-1]] if i["span"] else [],
                                      avg={p: round(v, 1) for p, v in i["avg"].items()}, would=i["would"],
                                      exempt=i["exempt"])
            continue
        keep = f"{name}-v{st['field_version']}-{i['outcome'][0]}" if i["outcome"] else None
        if keep and keep not in (st["entries"] or []):  # never expected: the entry that played this cycle
            extra.append(f"Freeze/retire rule: {name} not applied: no entry {keep} among {st['entries']}.")
            continue
        text = rule_text(i, rc["milestone_window"])
        st["rule"] = dict(cycle=cycle, dormant_since=i["c0"], cycles=[i["span"][0], i["span"][-1]],
                          avg={p: round(v, 1) for p, v in i["avg"].items()}, text=text,
                          outcome=f"freeze {i['outcome'][0]} at {i['outcome'][1]}" if keep else "retire")
        st.pop("rule_preview", None)
        if keep:
            other = [e for e in st["entries"] if e != keep]
            st["frozen_from"], st["status"], st["frozen_retired"], st["entries"] = st["status"], "frozen", other, [keep]
            st["claim"], st["frozen_by"] = i["outcome"][1], "rule"
            survey["taken"].setdefault(i["outcome"][1], []).append(keep)
            retire += other
        else:
            st["retired_from"], st["status"], st["retired_cycle"], st["retired_by"] = st["status"], "retired", cycle, "rule"
            st["retired_entries"] = list(st["entries"] or [])
            retire += st["retired_entries"]
        rows[name]["status"] = st["status"]
        rows[name]["decision"] = f"{text}; {rows[name].get('decision') or ''}".rstrip("; ")
        said.append(f"{name} {text}")
    for name, st in state["lineages"].items():
        if st["status"] != "dormant" or name not in survey["lineages"]:
            st.pop("rule_preview", None)
    state["milestones"] = {str(m): ", ".join(t) for m, t in sorted(survey["taken"].items())}
    if said:
        rr, after = cfg.tournament["round_robins_per_cycle"], n_players - len(set(retire))
        extra.append(f"Freeze/retire rule, after {rc['after_dormant']} cycles dormant: " + "; ".join(said) +
                     f". {len(set(retire))} entries leave the field for retired.txt (their games stay in the fit): "
                     f"{n_players} players become {after}, {rr * n_players * (n_players - 1):,} games a cycle "
                     f"become {rr * after * (after - 1):,}.")
    return sorted(set(retire))


# ---------------------------------------------------------------- commands

def lock(cfg):
    f = open(cfg.file("state") + ".lock", "w")
    try:
        fcntl.flock(f, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except OSError:
        sys.exit("another cycle.py holds the lock")
    return f


def cmd_migrate_strikes(cfg, dry_run):
    """Once, when the peak rule replaced the predecessor test: recomputes each qualified lineage's strikes from the
    newest fit under the peak rule (1 if its current version is more than dormancy_se standard errors below its
    peak, else 0), since its old strikes counted drops against predecessors. Dormant lineages stay dormant (the
    user: "Let them stay dormant"); the rule re-qualifies them. Prints the table; writes state.json unless
    dry_run. Between cycles only."""
    held = None if dry_run else lock(cfg)
    state = load_state(cfg)
    if not state or state.get("current"):
        sys.exit("no state, or a cycle is under way: migrate between cycles")
    league = cfg.file("league_dir")
    ratings, depths, q = read_ratings(league), league_depths(cfg), cfg.tournament["qualify_depth"]
    ref = references_mean(ratings, rel_references(state))
    try:
        factor = fit_factor(league, ratings)
    except (OSError, ValueError):
        factor = None
    k = se_multipliers(cfg)[0]
    print(f"strikes under the peak rule, from the fit after cycle {state['cycle']} (the difference's ± is {k:g} "
          f"standard errors{' from the covariance' if factor else ', entries ± combined'}):")
    changed = 0
    for lin in load_registry(cfg):
        st = state["lineages"].get(lin["name"])
        if not st:
            continue
        cmp = peak_compare(ratings, depths, q, lin["name"], st["entries"], ref, factor)
        old = st["strikes"]
        if st["status"] != "dormant":
            st["strikes"] = 1 if cmp and cmp["gap"] < -k * cmp["se"] else 0
        changed += st["strikes"] != old
        gap = f"{cmp['gap']:+.0f} ± {k * cmp['se']:.0f} vs peak v{cmp['version']}" if cmp else "no earlier version"
        print(f"  {lin['name']:18} {st['status']:10} strikes {old} -> {st['strikes']}  {gap}")
    if not dry_run:
        state["strikes_migrated"] = dict(cycle=state["cycle"], date=now(), dormancy_se=k)
        save_state(cfg, state)
    print(f"{changed} changed" + (" (dry run: nothing written)" if dry_run else "; state.json written"))
    return held


def dormancy_versions(cfg):
    """{lineage: (cycle, version)} of each lineage's latest turn to dormant in cycles.csv: the row whose status is
    dormant after a row (or none) that wasn't; its version is the one rated then, the version that went dormant."""
    went, last = {}, {}
    with open(cfg.file("log_csv"), newline="") as f:
        for r in sorted(csv.DictReader(f), key=lambda r: int(r["cycle"])):
            if r["status"] == "dormant" and last.get(r["lineage"]) != "dormant":
                went[r["lineage"]] = (int(r["cycle"]), int(r["version"]) if r["version"] else None)
            last[r["lineage"]] = r["status"]
    return went


def cmd_migrate_dormant(cfg, dry_run):
    """Once, when a dormant lineage's bar became its peak from before it went dormant: records each dormant
    lineage's dormant_version (cycles.csv: the version of its latest turn to dormant) and re-checks it on the
    newest fit against that bar; above it by more than requalify_se standard errors, it re-qualifies now, so that
    none loses a cycle to the timing. Prints the table; writes state.json unless dry_run. Between cycles only."""
    held = None if dry_run else lock(cfg)
    state = load_state(cfg)
    if not state or state.get("current"):
        sys.exit("no state, or a cycle is under way: migrate between cycles")
    league = cfg.file("league_dir")
    ratings, depths, q = read_ratings(league), league_depths(cfg), cfg.tournament["qualify_depth"]
    ref = references_mean(ratings, rel_references(state))
    try:
        factor = fit_factor(league, ratings)
    except (OSError, ValueError):
        factor = None
    k, went = se_multipliers(cfg)[1], dormancy_versions(cfg)
    since = {}  # {lineage: {dormancy cycle: [versions current in the cycles after it]}}
    with open(cfg.file("log_csv"), newline="") as f:
        for r in csv.DictReader(f):
            c0 = went.get(r["lineage"], (None,))[0]
            if c0 is not None and int(r["cycle"]) > c0 and r["version"]:
                since.setdefault(r["lineage"], {}).setdefault(c0, []).append(int(r["version"]))
    print(f"dormant lineages against their peak from before dormancy, in the fit after cycle {state['cycle']} (the "
          f"difference's ± is {k:g} standard errors{' from the covariance' if factor else ', entries ± combined'}):")
    woke = 0
    for lin in load_registry(cfg):
        st = state["lineages"].get(lin["name"])
        if not st:
            continue
        if st["status"] != "dormant":
            st["dormant_version"] = None
            continue
        c, v = went.get(lin["name"], (None, None))
        if v is None:
            sys.exit(f"{lin['name']}: no turn to dormant in cycles.csv")
        st["dormant_version"] = v
        # every version that was current in a cycle since it went dormant, and the current one (the user: wake
        # everyone the rule would have woken), each against the bar in this fit
        tried = sorted(set(since.get(lin["name"], {}).get(c, [])) | {st["field_version"]})
        best = None
        for w in tried:
            cmp = peak_compare(ratings, depths, q, lin["name"], [f"{lin['name']}-v{w}-{q}"], ref, factor, v)
            if cmp and (best is None or cmp["gap"] - k * cmp["se"] > best[1]["gap"] - k * best[1]["se"]):
                best = (w, cmp)
        wake = best is not None and best[1]["gap"] > k * best[1]["se"]
        if wake:
            st["status"], st["strikes"], st["dormant_rel"], st["dormant_version"] = "qualified", 0, None, None
            woke += 1
        gap = (f"best v{best[0]}: {best[1]['gap']:+.0f} ± {k * best[1]['se']:.0f} vs peak v{best[1]['version']} at "
               f"{best[1]['elo']:.0f}" if best else "no earlier version rated")
        print(f"  {lin['name']:18} dormant since cycle {c} at v{v}, now v{st['field_version']}, tried "
              f"{','.join(f'v{w}' for w in tried)}: {gap} -> {'RE-QUALIFIED' if wake else 'stays dormant'}")
    if not dry_run:
        state["dormancy_migrated"] = dict(cycle=state["cycle"], date=now(), requalify_se=k)
        save_state(cfg, state)
    print(f"{woke} re-qualified" + (" (dry run: nothing written)" if dry_run else "; state.json written"))
    return held


def own_step_skipped(cfg, st):
    """A frozen or retired lineage never takes the own-games step; a dormant one does unless own_games.dormant is
    off."""
    return st["status"] in ("frozen", "retired") or (st["status"] == "dormant" and not cfg.own_games.get("dormant", True))


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


def cmd_freeze(cfg, name, ply=None, unfreeze=False):
    """freeze NAME PLY (the user, 2026-11-06): the lineage keeps ONE entry in the field, its current version's at
    PLY, as a fixed reference, and its other entries retire (to retired.txt: their games stay in the fit). It stops
    self-play, own steps, gates and decisions; cycles.csv still logs its rating (its frozen entry's) and all-time
    peak, as "frozen". Its status before is kept as frozen_from, the retired entries as frozen_retired, its strikes
    and dormant_version as they were. unfreeze NAME: the retired entries come back to the field and the lineage to
    frozen_from (a dormant lineage then faces its pre-dormancy bar again). Between cycles only (touch STOP)."""
    held = lock(cfg)
    state = load_state(cfg)
    if not state or state.get("current"):
        sys.exit("no state, or a cycle is under way: between cycles only (touch STOP, wait for the loop to stop)")
    if name not in registry_names(cfg) or name not in state["lineages"]:
        sys.exit(f"{name} is not a lineage in lineages.txt and state.json")
    st = state["lineages"][name]
    listed = {n for n, _, _ in read_players(cfg.file("league_dir"))}
    if not unfreeze:
        if st["status"] == "frozen":
            sys.exit(f"{name} is already frozen ({' '.join(st['entries'])})")
        keep = f"{name}-v{st['field_version']}-{ply}"
        if ply not in cfg.tournament["entry_depths"] or keep not in (st["entries"] or []) or keep not in listed:
            sys.exit(f"{name}: no entry {keep} in the field to freeze (entries: {' '.join(st['entries'] or [])})")
        other = [e for e in st["entries"] if e != keep]
        moved = update_league(cfg, other, [], [])
        st["frozen_from"], st["status"], st["frozen_retired"], st["entries"] = st["status"], "frozen", other, [keep]
        print(f"{name}: frozen after cycle {state['cycle']} (was {st['frozen_from']}): {keep} stays in the field; "
              f"{moved} retired ({' '.join(other) or 'none'})")
    else:
        if st["status"] != "frozen":
            sys.exit(f"{name} is not frozen ({st['status']})")
        back = st.pop("frozen_retired", None) or []
        st.pop("claim", None), st.pop("frozen_by", None)  # the milestone is free again; st["rule"] stays (no re-firing)
        moved = unretire_players(cfg, back)
        st["entries"] = sorted(set(st["entries"]) | set(back), key=lambda e: int(e.rsplit("-", 1)[1]))
        st["status"] = st.pop("frozen_from", None) or "dormant"
        print(f"{name}: unfrozen after cycle {state['cycle']}, back to {st['status']} with {' '.join(st['entries'])} "
              f"({moved} back from retired.txt; strikes {st['strikes']}"
              + (f", dormant since v{st.get('dormant_version')}" if st["status"] == "dormant" else "") + ")")
    rule_refresh(cfg, state)
    save_state(cfg, state)
    return held


def cmd_retire(cfg, name):
    """retire NAME, after taking its line out of lineages.txt (or commenting it out): its entries move from the
    league's players.txt to retired.txt, so its games stay in the fit; state.json keeps it with status "retired" and
    retired_cycle (its last cycle). The driver only runs lineages.txt's lineages, so it is never planned, stepped,
    decided or logged again. Between cycles only."""
    held = lock(cfg)
    state = load_state(cfg)
    if not state or state.get("current"):
        sys.exit("no state, or a cycle is under way: between cycles only (touch STOP, wait for the loop to stop)")
    if name in registry_names(cfg):
        sys.exit(f"{name} is still in lineages.txt: take its line out first")
    st = state["lineages"].get(name)
    if not st:
        sys.exit(f"no lineage {name} in state.json")
    listed = [n for n, _, _ in read_players(cfg.file("league_dir")) if n.startswith(name + "-v")]
    entries = sorted(set(st["entries"] or []) | set(listed))
    moved = update_league(cfg, entries, [], [])
    if st["status"] != "retired":
        st["retired_from"], st["status"], st["retired_cycle"] = st["status"], "retired", state["cycle"]
    rule_refresh(cfg, state)
    save_state(cfg, state)
    print(f"{name}: retired after cycle {st['retired_cycle']}; {moved} entries moved to retired.txt "
          f"({' '.join(entries) or 'none'}); its games stay in the fit")
    return held


def cmd_unretire(cfg, name):
    """unretire NAME: a retired lineage's entries come back from retired.txt to the field and it gets back the status
    it had (retired_from: dormant, for the freeze/retire rule's). Its line must be in lineages.txt (the rule leaves
    it there; put back a line taken out for `retire`). The rule's record (rule) stays, so it doesn't fire again in
    the same dormant spell. Between cycles only."""
    held = lock(cfg)
    state = load_state(cfg)
    if not state or state.get("current"):
        sys.exit("no state, or a cycle is under way: between cycles only (touch STOP, wait for the loop to stop)")
    st = state["lineages"].get(name)
    if not st or st["status"] != "retired":
        sys.exit(f"{name} is not a retired lineage in state.json" + (f" ({st['status']})" if st else ""))
    if name not in registry_names(cfg):
        sys.exit(f"{name} is not in lineages.txt: put its line back first")
    back = st.pop("retired_entries", None) or list(st["entries"] or [])
    moved = unretire_players(cfg, back)
    st["entries"] = sorted(set(st["entries"] or []) | set(back), key=lambda e: int(e.rsplit("-", 1)[1]))
    st["status"] = st.pop("retired_from", None) or "dormant"
    st.pop("retired_by", None), st.pop("retired_cycle", None)
    rule_refresh(cfg, state)
    save_state(cfg, state)
    print(f"{name}: unretired after cycle {state['cycle']}, back to {st['status']} with {' '.join(st['entries'])} "
          f"({moved} back from retired.txt)")
    return held


def cmd_wake(cfg, name):
    """wake NAME (the user, 2026-11-07): a dormant lineage becomes qualified by hand, as re-qualifying makes it:
    strikes 0, no dormant_version (no pre-dormancy bar) and no dormant_rel, and it self-plays from the next cycle.
    It undid the old dormancy rule (through cycle 8 a lineage got a strike when its Elo dropped against its
    predecessor's, and 2 in a row made it dormant): the lineages whose dormancy began then were woken after cycle 15.
    A frozen or retired lineage needs `unfreeze` or `unretire` first. Its dormancy is kept in state.json as `woken`
    (the cycle after which it woke, its strikes and dormant_version), and a freeze/retire rule record moves to
    `rule_undone`, so the rule fires again only in a later dormant spell. Between cycles only."""
    held = lock(cfg)
    state = load_state(cfg)
    if not state or state.get("current"):
        sys.exit("no state, or a cycle is under way: between cycles only (touch STOP, wait for the loop to stop)")
    if name not in registry_names(cfg) or name not in state["lineages"]:
        sys.exit(f"{name} is not a lineage in lineages.txt and state.json")
    st = state["lineages"][name]
    if st["status"] != "dormant":
        sys.exit(f"{name} is {st['status']}, not dormant" +
                 (f": {'unfreeze' if st['status'] == 'frozen' else 'unretire'} it first"
                  if st["status"] in ("frozen", "retired") else ""))
    st["woken"] = dict(after_cycle=state["cycle"], strikes=st.get("strikes"), dormant_version=st.get("dormant_version"))
    st["status"], st["strikes"], st["dormant_rel"], st["dormant_version"] = "qualified", 0, None, None
    if st.get("rule"):
        st["rule_undone"] = dict(st.pop("rule"), undone_after_cycle=state["cycle"])
    rule_refresh(cfg, state)
    save_state(cfg, state)
    w = st["woken"]
    print(f"{name}: woken after cycle {state['cycle']}: qualified, strikes 0, no dormant_version (was dormant, "
          f"strikes {w['strikes']}, dormant since v{w['dormant_version']}); self-plays from cycle {state['cycle'] + 1}"
          + ("; its freeze/retire rule record moved to rule_undone" if st.get("rule_undone", {}).get(
              "undone_after_cycle") == state["cycle"] else ""))
    return held


def cmd_status(cfg):
    lineages, costs, state = load_registry(cfg), load_costs(cfg), load_state(cfg)
    t = cfg.tournament
    print(f"cycle {state['cycle'] if state else '- (run init)'}; budget {cfg.budget_cpu_s} CPU s per lineage; round "
          f"sizes " + ", ".join(f"d{d} {round_size(cfg, costs, d) or '?'}" for d in rule_depths(lineages)))
    print(f"league {cfg.league_dir}; tournament: {t['round_robins_per_cycle']} round-robins per cycle, entries at "
          f"plies {t['entry_depths']}, qualification at ply {t['qualify_depth']}: "
          f"{tournament_unavailable(cfg) or 'ready'}; own-games step: {own_unavailable(cfg) or 'ready'}")
    print(f"{'lineage':14} {'status':10} {'ver':>5} {'elo':>6} {'next self-play':42} recipe")
    for lin in lineages:
        st = dict(lineage_state(state or {"lineages": {}}, lin["name"]))
        if st["version"] is None and lin["source_type"] == "continue":
            st["version"] = last_version(lin["source"], lin["kind"])
        p, why = plan(cfg, costs, lin, st)
        nxt = st["status"] if st["status"] in ("dormant", "frozen", "retired") else (
            f"~{p['rounds']} x {p['size']} games ({p['cost'] * 1000:.2f} ms/game {p['cost_source']})" if p else why)
        elo = f"{st['elo']:.0f}" if st["elo"] is not None else "-"
        steps = "+own " if own_on(cfg, lin) else ""
        ver = st["version"] if st["version"] is not None else "-"
        print(f"{lin['name']:14} {st['status']:10} {ver:>5} {elo:>6} {nxt:42} {steps}{lin['sig']}")
    if state:  # the freeze/retire rule's preview: as of the last decision (the cycle under way's, once made)
        cur = state.get("current") or {}
        decided = cur.get("phase") in ("own", "log")
        print("\n".join(rule_preview_lines(rule_survey(cfg, state, cur["cycle"] if decided else state["cycle"],
                                                        [l["name"] for l in lineages], cur["rows"] if decided else None),
                                            markdown=False)))


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


def cmd_init(cfg, force):
    lk = lock(cfg)
    old = load_state(cfg)
    if old and not force:
        sys.exit("state.json exists: cycle 0 is recorded (init --force redoes it, before cycle 1 only)")
    if old and (old["cycle"] > 0 or old.get("current")):
        sys.exit("cycles have run: init --force only redoes cycle 0")
    lineages = load_registry(cfg)
    extra = setup_league(cfg)
    league = cfg.file("league_dir")
    players = read_players(league)
    state, rows = {"cycle": 0, "lineages": {}}, {}
    for lin in lineages:
        st = lineage_state(state, lin["name"])
        if lin["source_type"] == "continue":
            st["entries"] = [e[0] for e in match_entries(players, lin["source"])]
    lineage_entries = {e for s in state["lineages"].values() for e in s["entries"] or []}
    state["references"] = [n for n, _, _ in players if n not in lineage_entries]
    extra.append(f"The field: {len(players)} players, {len(lineage_entries)} of them lineage entries; the "
                 f"{len(state['references'])} others are the fixed references (players that never train).")
    pool = cfg.abs(cfg.own_games["first_pool"]) if cfg.own_games.get("first_pool") else None
    if pool and lineage_entries and any(own_on(cfg, l) for l in lineages):
        why = own_unavailable(cfg)
        games = games_of(saved_games(pool), lineage_entries, own_opponents(cfg, state))
        if why or not games:
            extra.append(f"No own-games step in cycle 0: {why or f'no usable games in {cfg.rel(pool)}'}.")
            pool = None
        else:
            extra.append(f"Own-games pool: the games saved in {cfg.rel(pool)}, of which {len(games)} count.")
    else:
        pool = None
    ratings, depths = read_ratings(league), league_depths(cfg)
    q = cfg.tournament["qualify_depth"]
    ref = references_mean(ratings, rel_references(state))
    for lin in lineages:
        st = lineage_state(state, lin["name"])
        now_r = rating_at(ratings, st["entries"] or [], depths, q)
        if now_r:
            st["elo"], st["ci"], st["rel"], st["elo_cycle"] = now_r[0], now_r[1], now_r[0] - ref, 0
        rows[lin["name"]] = dict(
            cycle=0, date=now(), lineage=lin["name"], status=st["status"], entries=" ".join(st["entries"] or []),
            elo=st["elo"], ci95=st["ci"], rel_elo=st["rel"],
            decision="entry" + ("" if os.path.exists(lin["binary"]) else " (waiting for binary)"))
    save_state(cfg, state)
    if pool:
        own_steps(cfg, state, lineages, pool, 0, rows, players, [])
    for lin in lineages:
        st = lineage_state(state, lin["name"])
        rows[lin["name"]]["version"] = st["field_version"]
        rows[lin["name"]]["next_version"] = st["version"]
    save_state(cfg, state)
    append_log(cfg, rows.values())
    write_summary(cfg, 0, list(rows.values()), [f"Cycle 0 ({now()}): the league before any cycle. Every lineage "
                                                "qualifies on entry."] + extra)
    print(f"cycle 0 recorded: {len(rows)} lineages, {sum(r['elo'] is not None for r in rows.values())} with a league "
          f"rating; see {cfg.file('log_dir')}/cycle-000.md")
    lk.close()


def cmd_run(cfg, cycles):
    lk = lock(cfg)
    for _ in range(cycles):
        run_cycle(cfg)
    lk.close()


def start_cycle(cfg, state, lineages, costs):
    """The next cycle's plan: its log rows and its self-play jobs."""
    cycle = state["cycle"] + 1
    players = read_players(cfg.file("league_dir"))
    print(f"cycle {cycle} ({now()})", flush=True)
    cur = dict(cycle=cycle, phase="self-play", rows={}, jobs={}, started=[], done=[], dormant=[], changed=[],
               league=None, moved=None, target=None, extra=[], pool=None, own_done=[])
    for lin in lineages:
        name, st = lin["name"], lineage_state(state, lin["name"])
        if st["status"] == "retired":  # retired by the freeze/retire rule: no row, no entries (unretire NAME)
            continue
        row = cur["rows"][name] = dict(cycle=cycle, lineage=name, decision="")
        if st["status"] == "frozen":  # no self-play, own step, gate or decision; its entries stay in the field
            row["decision"] = "frozen"
            continue
        if st["status"] == "dormant":
            cur["dormant"].append(name)
            continue
        if not os.path.exists(lin["binary"]):
            row["decision"] = "waiting for binary"
            continue
        why = prepare(cfg, lin, st, players)
        if why:
            row["decision"] = why
            continue
        p, why = plan(cfg, costs, lin, st)
        if not p:
            row["decision"] = why
            continue
        cur["jobs"][name] = p
        keep_start_states(lin, p["start"])
        print(f"  {name}: from v{p['start']}, about {p['rounds']} x {p['size']} games ({p['cost_source']} "
              f"{1000 * p['cost']:.2f} ms CPU/game)", flush=True)
    return cur


def new_entries(cfg, state, lineages, cur):
    """The field: each lineage's newest version replaces its entries, at the entry plies. Updates the lineages'
    state in memory."""
    adds, models, retire = [], [], []
    for lin in lineages:
        name, st = lin["name"], lineage_state(state, lin["name"])
        if not st["prepared"] or not st["version"] or st["version"] <= st["field_version"]:
            continue
        v, kind = st["version"], lin["kind"]
        path = os.path.join(lin["run_dir"], f"{kind}{v}.model")
        entries = [f"{name}-v{v}-{d}" for d in cfg.tournament["entry_depths"]]
        adds += [f"{e} {d} {spec(lin, path)}" for e, d in zip(entries, cfg.tournament["entry_depths"])]
        moves, labels = describe(lin)
        if lin["source_type"] == "continue":
            start = f"continues {cfg.rel(lin['source'])} from v{st['entry_version']}"
        else:
            start = "scratch" if lin["source_type"] == "scratch" else f"from {cfg.rel(lin['source'])}"
        p = cur["jobs"].get(name)
        models.append(dict(
            name=f"{name}-v{v}", archetype=lin["archetype"],
            variant=f"cycle lineage {name}" + (" (own-games step)" if own_on(cfg, lin) else ""),
            games="own (vN vs vN-1)", moves=moves, train_depth=lin["depth"], labels=labels, init=start,
            rounds=f"v{v} (cycle {cur['cycle']}" + (f": {cur['rows'][name].get('rounds') or p['rounds']}x{p['size']})"
                                                    if p else ")"),
            seed=lin["seed"], calibrated="yes" if lin["heldout"] else "no", run_dir=cfg.rel(lin["run_dir"]),
            script="cycles/cycle.py"))
        retire += st["entries"] or []
        st["prev_entries"], st["entries"], st["field_version"] = st["entries"] or [], entries, v
        cur["changed"].append(name)
    cur["league"] = dict(retire=sorted(set(retire)), add_lines=adds, add_models=models)


def run_cycle(cfg):
    """One cycle, in phases saved in state.json, so that an interrupted cycle resumes where it stopped."""
    state = load_state(cfg)
    if not state:
        sys.exit("no state.json: run init first")
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
    cycle, rows = cur["cycle"], cur["rows"]
    registry = load_registry(cfg)
    lineages = [l for l in registry if l["name"] in rows]  # lineages added since wait for the next cycle
    league = cfg.file("league_dir")

    if cur["phase"] == "self-play":  # 1. self-play for the qualified lineages
        by_name = {l["name"]: l for l in lineages}
        cut = sorted(set(cur["started"]) - set(cur["done"]))  # running when the driver stopped
        if cut:
            cur["extra"].append(f"Self-play of {', '.join(cut)} was interrupted and resumed; its CPU time leaves out "
                                f"the command that was cut short.")

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
            print(f"  finished {name}: v{p['start']} -> v{v}, {v - p['start']} x {p['size']} games, cpu {cpu:.0f} s, "
                  f"wall {wall:.0f} s" + (f"; measured {1000 * m['cost']:.2f} ms CPU/game over {m['rounds']} rounds"
                                          if m else "") + (f"; FAILED: {p['failed']}" if p.get("failed") else ""),
                  flush=True)

        run_jobs(cfg, [(n, stepper(n), by_name[n]["run_dir"] + ".log") for n in cur["jobs"]
                       if n in by_name and n not in cur["done"]], cycle, started, finished)
        for name in cur["done"]:
            lin, st, row, p = by_name[name], lineage_state(state, name), rows[name], cur["jobs"][name]
            st["version"] = last_version(lin["run_dir"], lin["kind"])
            if row.pop("code", 0) != 0 or st["version"] != p.get("end"):
                row["decision"] = f"self-play failed (last version v{st['version']}); see {lin['run_dir']}.log"
        new_entries(cfg, state, lineages, cur)
        cur["phase"] = "tournament"
        save_state(cfg, state)

    if cur["phase"] == "tournament":  # 2. the field, then fresh round-robins of all of it; games saved for the step
        own_any = any(own_on(cfg, l) for l in lineages)
        why = own_unavailable(cfg) if own_any else None
        wait_league(cfg)
        if cur["moved"] is None:
            cur["moved"] = update_league(cfg, **cur["league"])
            cur["extra"].append(f"League: retired {cur['moved']} players to retired.txt (their games stay in the fit), "
                                f"added {len(cur['league']['add_lines'])}.")
            save_state(cfg, state)
        if cur["target"] is None:  # fixed once, so that a resumed tournament plays only what is left
            cur["target"] = done_round_robins(league) + cfg.tournament["round_robins_per_cycle"]
            save_state(cfg, state)
        cur["pool"] = pool_dir(cfg, cycle) if own_any and not why else None
        secs = run_tournament(cfg, cur["target"], cur["pool"])
        n = len(read_players(league))
        cur["extra"].append(f"Tournament: {cfg.tournament['round_robins_per_cycle']} double round-robins of the "
                            f"{n} active players ({cfg.tournament['round_robins_per_cycle'] * n * (n - 1)} games) "
                            f"in {secs / 60:.1f} min" + (f", {len(saved_games(cur['pool']))} games saved." if cur["pool"]
                                                         else "."))
        if why:
            cur["extra"].append(f"Own-games step skipped: {why}.")
        cur["phase"] = "decide"
        save_state(cfg, state)

    if cur["phase"] == "decide":  # 3. qualification: each lineage's current version against its peak, in this fit
        ratings, depths, q = read_ratings(league), league_depths(cfg), cfg.tournament["qualify_depth"]
        ref = references_mean(ratings, rel_references(state))
        try:  # the standard errors of differences, for the peaks
            factor, why = fit_factor(league, ratings), "its information matrix is not positive definite"
        except (OSError, ValueError) as e:
            factor, why = None, f"games.csv could not be read ({e})"
        if factor is None:
            cur["extra"].append(f"Peaks: no standard errors from the fit ({why}), so the differences with the peaks "
                                "have the two entries' ± combined instead.")
        k_dorm, k_req = se_multipliers(cfg)
        for lin in lineages:
            name, st, row = lin["name"], lineage_state(state, lin["name"]), rows[lin["name"]]
            now_r = (rating_of(ratings, st["entries"] or []) if st["status"] == "frozen"  # its one entry, any ply
                     else rating_at(ratings, st["entries"] or [], depths, q))
            rel = now_r[0] - ref if now_r else None
            cmp = peak_compare(ratings, depths, q, name, st["entries"], ref, factor)  # every lineage: all-time peak
            row["peak_elo"], row["peak_version"] = (cmp["elo"], cmp["version"]) if cmp else (None, None)
            if st["status"] == "frozen" and not cmp:  # its one entry may be at another ply: the peak alone
                pk = lineage_peak(ratings, depths, q, name, st["entries"] or [], ref)
                if pk:
                    row["peak_elo"], row["peak_version"] = rating_of(ratings, pk[2])[0], pk[1]
            if st["status"] == "dormant":  # its bar: the peak from before it went dormant, in columns of its own
                if st.get("dormant_version") is not None:
                    cmp = peak_compare(ratings, depths, q, name, st["entries"], ref, factor, st["dormant_version"])
                row["pre_dormancy_peak_elo"], row["pre_dormancy_peak_version"] = \
                    (cmp["elo"], cmp["version"]) if cmp else (None, None)
            if name in cur["dormant"] or name in cur["changed"]:
                prev = rating_at(ratings, st["prev_entries"] or [], depths, q) if name in cur["changed"] else None
                row["prev_elo"] = prev[0] if prev else None
                row["decision"] = decide(st, name in cur["changed"], now_r, prev, rel, cfg.patience, k_dorm, k_req,
                                         cmp, st["field_version"])
            elif not row["decision"]:
                row["decision"] = "not trained"
            if now_r:
                st["elo"], st["ci"], st["rel"], st["elo_cycle"] = now_r[0], now_r[1], rel, cycle
            row.update(date=now(), status=st["status"], version=st["field_version"], strikes=st["strikes"],
                       entries=" ".join(st["entries"] or []), elo=now_r[0] if now_r else None,
                       ci95=now_r[1] if now_r else None, rel_elo=rel)
        state.setdefault("round_robin_ends", {})[str(cycle)] = cur["target"]  # elo_history's end of this cycle
        survey = rule_survey(cfg, state, cycle, [l["name"] for l in lineages], rows, ratings)
        cur["rule_retire"] = rule_apply(cfg, state, survey, rows, cycle, cur["extra"], len(read_players(league)))
        cur["extra"] += rule_preview_lines(survey, markdown=True)
        cur["phase"] = "own"
        save_state(cfg, state)

    if cur["phase"] == "own":  # 4. the own-games step and its gate
        if cur["pool"]:
            own_steps(cfg, state, lineages, cur["pool"], cycle, rows, read_players(league), cur["own_done"])
        if cur.get("rule_retire"):  # the freeze/retire rule's: to retired.txt before the next round-robin
            moved = update_league(cfg, cur["rule_retire"], [], [])
            print(f"  freeze/retire rule: {moved} entries to retired.txt ({' '.join(cur['rule_retire'])})", flush=True)
        cur["phase"] = "log"
        save_state(cfg, state)

    # 5. The logs.
    for lin in lineages:
        rows[lin["name"]]["next_version"] = lineage_state(state, lin["name"])["version"]
    ordered = [rows[l["name"]] for l in lineages]
    if cycle not in logged_cycles(cfg):
        append_log(cfg, ordered)
    write_summary(cfg, cycle, ordered, rel_refs=rel_references(state), extra=
                  [f"{now()}: budget {cfg.budget_cpu_s} CPU s per qualified lineage, measured in the loop: each "
                   f"job measured its CPU per game over its first rounds (at least {100 * MEASURE_MIN:.0f}% of the "
                   f"budget), then the rest in chunks of about {100 * CHUNK:.0f}% of it, each re-planned from the last one's "
                   f"CPU per game until the budget was spent; round sizes "
                   + ", ".join(f"d{d} {round_size(cfg, costs, d)}" for d in rule_depths(lineages)) + "."]
                  + cur["extra"])
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
    def run(seq, patience=2, k=2.0, se=10.0):
        """seq: (trained, elo) per cycle; trained makes a new version, otherwise the current one is rated again.
        Every version keeps its latest rating, as in one fit; refs' mean at 0; every difference's SE `se`."""
        st = dict(status="new", strikes=0, dormant_rel=None)
        out, elos = [], []
        for trained, elo in seq:
            if trained or not elos:
                elos.append(None)
            elos[-1] = elo
            prev = (elos[-2], 5.0) if trained and len(elos) > 1 else None
            earlier = [(e, v + 1) for v, e in enumerate(elos[:-1])]
            best = max(earlier) if earlier else None
            c = dict(gap=elo - best[0], se=se, version=best[1], rel=best[0], elo=best[0]) if best else None
            d = decide(st, trained, (elo, 5.0), prev, elo, patience, k, k, c)
            out.append((st["status"], st["strikes"], d.split(" (")[0]))
        return out
    # half-width 2 x 10: first rating, a new peak (130), within 20 of it (ok), 25 below (strike), 18 below resets,
    # strike, strike 2 -> dormant; rated again +10 over the peak (v2's 130) it stays dormant, +21 re-qualifies it
    # (the current version is not part of its own peak); then a new peak over v7.
    got = run([(True, 100), (True, 130), (True, 115), (True, 105), (True, 112), (True, 100), (True, 95),
               (False, 140), (False, 151), (True, 160)])
    want = [("qualified", 0, "qualified"), ("qualified", 0, "new peak"), ("qualified", 0, "ok"),
            ("qualified", 1, "strike 1"), ("qualified", 0, "ok"), ("qualified", 1, "strike 1"),
            ("dormant", 2, "dormant"), ("dormant", 2, "dormant"), ("qualified", 0, "re-qualified"),
            ("qualified", 0, "new peak")]
    assert got == want, f"\n got  {got}\n want {want}"
    # rising against the predecessor doesn't help while far below the peak; patience 1: one strike is dormant
    got = run([(True, 200), (True, 120), (True, 150)], patience=1)
    assert got == [("qualified", 0, "qualified"), ("dormant", 1, "dormant"), ("dormant", 1, "dormant")], got
    got = run([(True, 200), (True, 120), (True, 150)])
    assert got == [("qualified", 0, "qualified"), ("qualified", 1, "strike 1"), ("dormant", 2, "dormant")], got
    c = lambda gap, se, v=3, rel=100.0: dict(gap=gap, se=se, version=v, rel=rel, elo=rel + 50)
    st = dict(status="qualified", strikes=1, dormant_rel=None)
    d = decide(st, True, (150.0, 20.0), (140.0, 20.0), 100.0, 2, 2.0, 2.0, c(-40.0, 20.0))
    assert d == "ok (-40 ± 40 vs peak v3; predecessor +10)" and st["strikes"] == 0, d
    d = decide(st, True, (150.0, 20.0), (100.0, 20.0), 100.0, 2, 2.0, 2.0, c(-41.0, 20.0))
    assert d == "strike 1 (-41 ± 40 vs peak v3; predecessor +50)" and st["status"] == "qualified", d
    d = decide(st, True, (150.0, 20.0), (160.0, 20.0), 100.0, 2, 2.0, 2.0, c(-41.0, 20.0))
    assert d == "dormant (strike 2; -41 ± 40 vs peak v3; predecessor -10)" and st["dormant_rel"] == 100.0, d
    st = dict(status="qualified", strikes=1, dormant_rel=None)
    d = decide(st, True, (150.0, 20.0), (170.0, 20.0), 100.0, 2, 2.0, 2.0, c(5.0, 20.0))
    assert d == "new peak (+5 ± 40 over v3; predecessor -20)" and st["strikes"] == 0, d
    assert decide(st, True, (150.0, 20.0), None, 100.0, 2, 1.0, 2.0, c(-21.0, 20.0)) == "strike 1 (-21 ± 20 vs peak v3)"
    st = dict(status="qualified", strikes=1, dormant_rel=None)  # no earlier version rated: no strike
    assert decide(st, True, (150.0, 20.0), None, 100.0, 2, 2.0, 2.0, None) == "qualified (first rating)"
    assert st["strikes"] == 0
    # dormant: re-qualifies only above requalify_se standard errors over the peak
    st = dict(status="dormant", strikes=2, dormant_rel=60.0)
    d = decide(st, True, (110.0, 20.0), (90.0, 20.0), 110.0, 2, 2.0, 2.0, c(40.0, 20.0, 3, 70.0))
    assert d == "dormant (+40 ± 40 vs peak v3 at 70 vs refs; needs > +40)" and st["status"] == "dormant", d
    d = decide(st, True, (110.0, 20.0), (90.0, 20.0), 110.0, 2, 2.0, 2.0, c(41.0, 20.0, 3, 69.0))
    assert d == "re-qualified (+41 ± 40 vs peak v3 at 69 vs refs)" and st == dict(status="qualified", strikes=0,
                                                                                   dormant_rel=None,
                                                                                   dormant_version=None), (d, st)
    st = dict(status="dormant", strikes=2, dormant_rel=60.0)
    assert decide(st, False, (500.0, 20.0), None, 500.0, 2, 2.0, 2.0, None) == "dormant (no earlier version rated)"
    assert decide(st, False, None, None, None, 2, 2.0, 2.0, c(1.0, 1.0)) == "dormant" and st["status"] == "dormant"
    st = dict(status="new", strikes=0, dormant_rel=None)  # not trained, not dormant: keeps its status
    assert decide(st, False, (5.0, 10.0), None, 5.0, 2, 2.0, 2.0) == "not trained" and st["status"] == "new"
    # a dormant lineage's bar is its peak from before it went dormant (v2 at 140; it went dormant at v3): its own
    # steps crawl up in steps under 2 SE (14 here) and it re-qualifies once the total clears the bar by 2 SE, though
    # every step is within range of the version before; versions made while dormant never raise the bar.
    rt, dp = {}, {}
    st = dict(status="dormant", strikes=2, dormant_rel=0.0, dormant_version=3)
    got = []
    for v, elo in enumerate([100, 140, 120, 128, 136, 144, 150, 155], start=1):
        rt[f"L-v{v}-2"], dp[f"L-v{v}-2"] = (float(elo), 9.8, 9), 2
        if v <= 3:
            continue
        pc = peak_compare(rt, dp, 2, "L", [f"L-v{v}-2"], 0.0, None, st["dormant_version"] if st["status"] ==
                          "dormant" else None)
        assert st["status"] != "dormant" or pc["version"] == 2, pc  # never raised by v4.. while dormant
        got.append(decide(st, True, (float(elo), 9.8), None, float(elo), 2, 2.0, 2.0, pc, v).split(" (")[0])
    assert got == ["dormant"] * 4 + ["re-qualified"], got  # +15 over v2 at v8, each step under 14
    assert st["dormant_version"] is None, st
    pc = peak_compare(rt, dp, 2, "L", ["L-v8-2"], 0.0, None)  # the old bar: v7, made while dormant (+5 only)
    assert pc["version"] == 7 and pc["gap"] == 5.0, pc
    d = decide(dict(status="dormant", strikes=2, dormant_rel=0.0, dormant_version=3), True, (155.0, 9.8), None,
               155.0, 2, 2.0, 2.0, peak_compare(rt, dp, 2, "L", ["L-v8-2"], 0.0, None, 3), 8)
    assert d.startswith("re-qualified (+15 ± 14 vs peak v2 before dormancy at v3, at 140 vs refs"), d
    # the current version is the one that went dormant (every gate reverted): the bar excludes it
    assert peak_compare(rt, dp, 2, "L", ["L-v3-2"], 0.0, None, 3)["version"] == 2
    # sleep, wake, sleep: going dormant records the version, re-qualifying clears it, the next sleep records anew
    st = dict(status="qualified", strikes=1, dormant_rel=None)
    decide(st, True, (100.0, 9.8), None, 100.0, 2, 2.0, 2.0, c(-50.0, 10.0), 5)
    assert st["status"] == "dormant" and st["dormant_version"] == 5, st
    decide(st, True, (100.0, 9.8), None, 100.0, 2, 2.0, 2.0, c(30.0, 10.0), 6)
    assert st["status"] == "qualified" and st["dormant_version"] is None, st
    for v in (7, 9):
        decide(st, True, (100.0, 9.8), None, 100.0, 2, 2.0, 2.0, c(-50.0, 10.0), v)
    assert st["status"] == "dormant" and st["dormant_version"] == 9, st
    # peak_compare: the peak excludes the current version; without the covariance, the ± combined (95% -> SE)
    rt = {"L-v1-2": (200.0, 39.2, 9), "L-v2-2": (150.0, 39.2, 9), "L-v3-2": (180.0, 19.6, 9), "L-v3-1": (90.0, 9.8, 9)}
    dp = {"L-v1-2": 2, "L-v2-2": 2, "L-v3-2": 2, "L-v3-1": 1}
    pc = peak_compare(rt, dp, 2, "L", ["L-v3-1", "L-v3-2"], 50.0, None)
    assert pc["version"] == 1 and pc["elo"] == 200.0 and abs(pc["gap"] + 20) < 1e-9, pc
    assert abs(pc["se"] - math.hypot(10, 20)) < 1e-9 and peak_compare(rt, dp, 2, "L", ["L-v1-2"], 0.0, None)["version"] == 3
    assert peak_compare(rt, dp, 2, "M", ["M-v1-2"], 0.0, None) is None
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
    # the peak: the lineage's best earlier version at the qualifying ply, retired or not, never the current one,
    # nor another lineage's whose name extends this one's; relative to the references' mean.
    ratings.update({"x-v1-2": (250.0, 20.0, 1), "x-v3-2": (280.0, 20.0, 1), "x-v4-2": (400.0, 20.0, 1),
                    "x-v4-1": (500.0, 20.0, 1), "x-merge-v9-2": (900.0, 20.0, 1), "x-v5-1": (999.0, 20.0, 1)})
    depths.update({"x-v1-2": 2, "x-v3-2": 2, "x-v4-2": 2, "x-v4-1": 1, "x-merge-v9-2": 2, "x-v5-1": 1})
    assert lineage_peak(ratings, depths, 2, "x", ["x-v4-1", "x-v4-2"], 50.0) == (250.0, 2, ["x-v2-2"])
    assert lineage_peak(ratings, depths, 2, "x", ["x-v2-1", "x-v2-2"], 50.0) == (350.0, 4, ["x-v4-2"])
    assert lineage_peak(ratings, depths, 1, "x", ["x-v5-1"], 0.0) == (500.0, 4, ["x-v4-1"])
    assert lineage_peak(ratings, depths, 2, "y", [], 0.0) is None
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
    oc = lambda dormant: types.SimpleNamespace(own_games=dict(dormant=dormant))
    assert own_step_skipped(oc(True), dict(status="frozen")) and own_step_skipped(oc(False), dict(status="dormant"))
    assert not own_step_skipped(oc(True), dict(status="dormant")) and not own_step_skipped(oc(True), dict(status="qualified"))
    tr = lambda line: model_args(dict(flags=parse_flags(line.split())))  # the own step's train keeps the labels
    assert tr("--model lr --use-probs --seed 21 --vs-previous --no-games --policy log-odds-softmax --temperature 0.3 "
              "--depth 4 --label-depth 2 --label-result 0.5") == ["--use-probs", "--label-depth", "2", "--label-result", "0.5"]
    assert tr("--model bdt --leaf cnbp --router cnbp --seed 21 --vs-previous --no-games --depth 2 --use-probs") == [
        "--leaf", "cnbp", "--router", "cnbp", "--use-probs"]
    assert rule_depths([dict(depth=0), dict(depth=4)]) == [0, 1, 2, 3, 4] and rule_depths([dict(depth=2)]) == [0, 1, 2, 3]
    with tempfile.TemporaryDirectory() as d:  # freeze, unfreeze and retire on a scratch root
        league = os.path.join(d, "league")
        os.makedirs(league)
        with open(os.path.join(league, "players.txt"), "w") as f:
            f.write("random 0 rand\nA-v3-1 1 s\nA-v3-2 2 s\nB-v7-1 1 s\nB-v7-2 2 s\n")
        with open(os.path.join(league, "models.csv"), "w") as f:
            f.write("name,archetype\nrandom,random\n")
        reg = os.path.join(d, "lineages.txt")
        line = lambda n: f"{n} | X | bin | scratch | lr | own | --model lr --seed 21\n"
        with open(reg, "w") as f:
            f.write(line("A") + line("B"))
        cfg = Config.__new__(Config)
        cfg.root = d
        cfg.raw = dict(league_dir=league, state=os.path.join(d, "state.json"), registry=reg, heldout={},
                       runs_dir=os.path.join(d, "runs"), tournament=dict(retired=os.path.join(league, "retired.txt"),
                                                                       entry_depths=[1, 2]))
        st0 = lambda v, s: dict(status=s, strikes=2, dormant_version=v, field_version=v)
        state = dict(cycle=10, lineages={"A": dict(st0(3, "dormant"), entries=["A-v3-1", "A-v3-2"]),
                                         "B": dict(st0(7, "dormant"), entries=["B-v7-1", "B-v7-2"])})
        save_state(cfg, state)
        assert registry_names(cfg) == ["A", "B"]
        if True:
            import contextlib, io as _io
            with contextlib.redirect_stdout(_io.StringIO()):
                cmd_freeze(cfg, "A", 1).close()  # one entry stays, at ply 1; the other retires
                s = load_state(cfg)["lineages"]["A"]
                assert s["status"] == "frozen" and s["frozen_from"] == "dormant" and s["dormant_version"] == 3, s
                assert s["entries"] == ["A-v3-1"] and s["frozen_retired"] == ["A-v3-2"], s
                cfg.raw["own_games"] = dict(opponents="lineages")  # the frozen-out entry played: its games count
                assert own_opponents(cfg, load_state(cfg)) == {"A-v3-1", "A-v3-2", "B-v7-1", "B-v7-2"}
                assert [n for n, _, _ in read_players(league)] == ["random", "A-v3-1", "B-v7-1", "B-v7-2"]
                assert [n for n, _, _ in read_entries(os.path.join(league, "retired.txt"))] == ["A-v3-2"]
                cmd_freeze(cfg, "A", unfreeze=True).close()  # both plies back in the field
                s = load_state(cfg)["lineages"]["A"]
                assert s["status"] == "dormant" and "frozen_from" not in s and s["strikes"] == 2, s
                assert s["entries"] == ["A-v3-1", "A-v3-2"] and "frozen_retired" not in s, s
                assert sorted(n for n, _, _ in read_players(league)) == ["A-v3-1", "A-v3-2", "B-v7-1", "B-v7-2",
                                                                         "random"]
                assert [n for n, _, _ in read_entries(os.path.join(league, "retired.txt"))] == []
                cmd_freeze(cfg, "A", 2).close()
                assert load_state(cfg)["lineages"]["A"]["entries"] == ["A-v3-2"]
                cmd_freeze(cfg, "A", unfreeze=True).close()
                with open(reg, "w") as f:
                    f.write(line("A") + "# retired: " + line("B"))
                for _ in range(2):
                    cmd_retire(cfg, "B").close()
            s = load_state(cfg)["lineages"]["B"]
            assert s["status"] == "retired" and s["retired_cycle"] == 10, s
            assert sorted(n for n, _, _ in read_players(league)) == ["A-v3-1", "A-v3-2", "random"]
            assert [n for n, _, _ in read_entries(os.path.join(league, "retired.txt"))] == ["B-v7-1", "B-v7-2"]
            with contextlib.redirect_stdout(_io.StringIO()):
                try:
                    cmd_unretire(cfg, "B")
                    raise AssertionError("unretired without its line in lineages.txt")
                except SystemExit as e:
                    assert "lineages.txt" in str(e), e
                with open(reg, "w") as f:
                    f.write(line("A") + line("B"))
                cmd_unretire(cfg, "B").close()
            s = load_state(cfg)["lineages"]["B"]
            assert s["status"] == "dormant" and "retired_from" not in s and s["entries"] == ["B-v7-1", "B-v7-2"], s
            assert sorted(n for n, _, _ in read_players(league)) == ["A-v3-1", "A-v3-2", "B-v7-1", "B-v7-2", "random"]
            assert [n for n, _, _ in read_entries(os.path.join(league, "retired.txt"))] == []
            with contextlib.redirect_stdout(_io.StringIO()):  # wake: dormant -> qualified, as re-qualifying does
                s = load_state(cfg)
                s["lineages"]["B"]["rule"] = dict(dormant_since=5)
                save_state(cfg, s)
                cmd_wake(cfg, "B").close()
                s = load_state(cfg)["lineages"]["B"]
                assert s["status"] == "qualified" and s["strikes"] == 0 and s["dormant_version"] is None, s
                assert s["woken"] == dict(after_cycle=10, strikes=2, dormant_version=7) and "rule" not in s, s
                assert s["rule_undone"] == dict(dormant_since=5, undone_after_cycle=10), s
                for who, why in (("B", "not dormant"), ("A", None)):
                    if who == "A":
                        cmd_freeze(cfg, "A", 1).close()
                    try:
                        cmd_wake(cfg, who)
                        raise AssertionError(f"woke {who}")
                    except SystemExit as e:
                        assert (why or "unfreeze it first") in str(e), e
                cmd_freeze(cfg, "A", unfreeze=True).close()
    # the freeze/retire rule (the user, 2026-11-06): today's case, a same-group conflict, the fallback, retire-both
    rc = lambda groups, taken=(0, 200, 300, 400): resolve_claims(groups, set(taken), 100, 10)
    got = rc([[("Nc", {"1": 805.2, "2": 1492.5}), ("Ne", {"1": 799.4, "2": 1490.0}), ("d2mix", {"1": 837.0, "2": 1782.1})]])
    assert got == {"Ne": (1, 800, 799.4), "Nc": (2, 1500, 1492.5), "d2mix": None}, got
    assert rc([[("X", {"1": 805.0, "2": 1495.0})]]) == {"X": (2, 1500, 1495.0)}  # a tie: ply 2
    assert rc([[("X", {"1": 790.0, "2": 1489.0})]]) == {"X": (1, 800, 790.0)}  # within 10: <= 10
    assert rc([[("X", {"1": 789.9, "2": 1489.0})]]) == {"X": None}  # neither within 10: retires
    assert rc([[("X", {"1": 210.0})]]) == {"X": None} and rc([[("X", {"1": 189.9})]]) == {"X": None}
    assert rc([[("A", {"1": 1203.0})], [("B", {"1": 1201.0})]]) == {"A": (1, 1200, 1203.0), "B": None}  # A due first
    assert rc([[("A", {"1": 1203.0}), ("B", {"1": 1201.0, "2": 1595.0})]]) == {"B": (1, 1200, 1201.0),
                                                                              "A": None}
    assert nearest_milestone(-30, 100) == 0 and nearest_milestone(850, 100) == 900 and nearest_milestone(849.9, 100) == 800
    rows = {4: dict(status="qualified", decision="strike 1 (-26)"), 5: dict(status="dormant", decision="dormant (-51, "
            "strike 2; -77 vs peak v500)"), 6: dict(status="dormant", decision="dormant (+24 ± 95 vs peak v500)")}
    assert went_dormant(rows) == 5 and went_dormant({**rows, 7: dict(status="dormant", decision="dormant (strike 2; "
                                                                     "-94 ± 58 vs peak v855)")}) == 7
    assert went_dormant({6: dict(status="qualified"), 7: dict(status="dormant"), 8: dict(status="dormant")}) == 7
    assert went_dormant({8: dict(status="qualified")}) is None
    assert [round_robin_end({}, c) for c in (1, 2, 3, 4, 12)] == [5, 10, 20, 30, 70]
    assert round_robin_end({"round_robin_ends": {"15": 86}}, 15) == 86
    # self-play in chunks (the user, 2026-11-07): each re-planned from the last one's cost, so drift is corrected
    def simulate(cost, size, start=0, restart_at=None, overhead=0.0, budget=240.0):
        """A job against a fake run: a command plays rounds at cost(n) CPU s per game. With restart_at, the driver
        dies during that command after 2 of its rounds (their CPU spent, unrecorded) and the job restarts."""
        with tempfile.TemporaryDirectory() as d:
            for n in range(1, start + 1):
                open(os.path.join(d, f"lr{n}.model"), "w").close()
            cf = Config.__new__(Config)
            cf.raw = dict(budget_cpu_s=budget)
            ln, p = dict(run_dir=d, kind="lr"), dict(rounds=1, size=size, start=start, cost=cost(start + 1),
                                                   base=["sp"], init=[])
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
        cf.raw = dict(budget_cpu_s=240.0)
        ln, p = dict(run_dir=d, kind="probit"), dict(rounds=1, size=1000, start=0, cost=0.0024, base=["sp"], init=[])
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
    print("selftest: qualification rules, gate arithmetic, lineage ratings, saved games, league updates, removals, "
          "trimming, freezing, retiring, unretiring, waking, the freeze/retire rule, self-play chunks, the own step's "
          "labels and probit's model files OK")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--config", default=os.path.join(HERE, "config.json"))
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("status")
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
    s = sub.add_parser("migrate-dormant")
    s.add_argument("--dry-run", action="store_true")
    s = sub.add_parser("migrate-strikes")
    s.add_argument("--dry-run", action="store_true")
    for c in ("freeze", "unfreeze", "retire", "unretire", "wake"):
        s = sub.add_parser(c)
        s.add_argument("name")
        if c == "freeze":
            s.add_argument("ply", type=int)
    a = ap.parse_args()
    if a.cmd == "selftest":
        return selftest()
    cfg = Config(a.config)
    if a.cmd == "status":
        cmd_status(cfg)
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
    elif a.cmd == "migrate-dormant":
        cmd_migrate_dormant(cfg, a.dry_run)
    elif a.cmd == "migrate-strikes":
        cmd_migrate_strikes(cfg, a.dry_run)
    elif a.cmd in ("freeze", "unfreeze"):
        cmd_freeze(cfg, a.name, getattr(a, "ply", None), a.cmd == "unfreeze")
    elif a.cmd == "retire":
        cmd_retire(cfg, a.name)
    elif a.cmd == "unretire":
        cmd_unretire(cfg, a.name)
    elif a.cmd == "wake":
        cmd_wake(cfg, a.name)
    elif a.cmd == "verify":
        sys.exit(0 if (verify_cycle(cfg, a.names, a.cycle) if a.cycle is not None else
                       cmd_verify(cfg, a.names, a.rounds)) else 1)


if __name__ == "__main__":
    main()
