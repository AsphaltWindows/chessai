#!/usr/bin/env python3
"""Writes league.md, a league's table: one row per model, with its status, archetype and recipe (models.csv) and the
ratings of its players at each search depth from the tournament tool's ratings.csv, sorted by the depth-2 Elo; then
the best model of each archetype, and the retired models. The ratings are the tool's: maximum likelihood over all the
games in games.csv. Statuses:
- lineage: the newest version of a training/tournament cycle lineage (cycles/README.md);
- retired: an older lineage version (retired.txt): it plays no more, but its games count;
- other: any other model, which plays but never trains (the cycles' league has none);
- random: the random players (rand), the reference.

    league.py [DIR]     DIR defaults to the current directory

For the cycles' league (selfplay/league-cycles); it also reads a league without retired.txt, such as the old one.
"""
import csv
import datetime
import os
import re
import sys

D = sys.argv[1] if len(sys.argv) > 1 else "."


def path(name):
    return os.path.join(D, name)


def roster(name):
    """name -> (model, depth) in file order. A player's model is its name without the depth suffix (-2, or -d2 for
    the old league's Nov 5 players)."""
    out = {}
    if os.path.exists(path(name)):
        for line in open(path(name)):
            line = line.strip()
            if line and not line.startswith("#"):
                player, depth, _ = line.split()
                out[player] = (re.sub(r"-d?%s$" % depth, "", player) if depth != "0" else player, int(depth))
    return out


players, retired = roster("players.txt"), roster("retired.txt")
models = {r["name"]: r for r in csv.DictReader(open(path("models.csv")))}
ratings = {}
if os.path.exists(path("ratings.csv")):
    ratings = {r["player"]: r for r in csv.DictReader(open(path("ratings.csv")))}
tournaments = games = 0
if os.path.exists(path("games.csv")):
    with open(path("games.csv")) as f:
        next(f)
        for line in f:
            games += 1
            tournaments = max(tournaments, int(line.split(",", 1)[0]) + 1)

warnings = []
rows = {}  # model -> {depth: rating row}
for name, (model, depth) in list(players.items()) + list(retired.items()):
    if model not in models:
        warnings.append(f"player {name}: model {model} is not in models.csv")
    if name in ratings:
        rows.setdefault(model, {})[depth] = ratings[name]
    elif name in players:
        warnings.append(f"player {name} has no rating yet")
for name in ratings:
    if name not in players and name not in retired:
        warnings.append(f"ratings.csv has {name}, who is in neither players.txt nor retired.txt")
for model in models:
    if model not in {m for m, _ in players.values()} | {m for m, _ in retired.values()}:
        warnings.append(f"models.csv has {model}, who has no players")

depths = sorted({d for _, d in players.values() if d > 0}) or [1, 2]
key_depth = 2 if 2 in depths else depths[-1]
active = list(dict.fromkeys(m for m, d in players.values() if d in depths))
gone = [m for m in dict.fromkeys(m for m, _ in retired.values()) if m not in active]


def status(model):
    if model in gone:
        return "retired"
    if model == "random" or all(players[p][1] == 0 or models.get(model, {}).get("archetype") == "random"
                                for p in players if players[p][0] == model):
        return "random"
    return "lineage" if models.get(model, {}).get("script") == "cycles/cycle.py" else "other"


def elo(model, depth):
    r = rows.get(model, {}).get(depth)
    return float(r["elo"]) if r else None


def cell(model, depth):
    r = rows.get(model, {}).get(depth)
    return f"{float(r['elo']):.0f} ±{float(r['ci95']):.0f}" if r else "–"


def by_rating(ms):
    return sorted(ms, key=lambda m: -elo(m, key_depth) if elo(m, key_depth) is not None else float("inf"))


cols = ["archetype", "variant", "games", "moves", "train_depth", "labels", "init", "rounds", "seed", "calibrated"]
heads = ["archetype", "variant", "trained on", "moves", "train depth", "labels", "init", "rounds x games", "seed", "cal"]


def table(ms, first=1):
    out = ["| # | model | status | " + " | ".join(heads) + " | " + " | ".join(f"d{d}" for d in depths)
           + " | games each |", "|" + "---|" * (len(heads) + len(depths) + 4)]
    for i, m in enumerate(ms, first):
        info = models.get(m, {})
        n = next((r["games"] for r in rows.get(m, {}).values()), "–")
        out.append(f"| {i} | {m} | {status(m)} | " + " | ".join(str(info.get(c, "?")) or " " for c in cols)
                   + " | " + " | ".join(cell(m, d) for d in depths) + f" | {n} |")
    return out


out = [
    "# League table",
    "",
    "Every player meets every other in double round-robins, at depths " + ", ".join(map(str, depths)) + " (random "
    "also at 0). The tournament tool rates them relative to random (0) by maximum likelihood over ALL the games in",
    "games.csv; its ratings.md has the ± details. Status: lineage = the newest version of a training/tournament cycle",
    "lineage (cycles/README.md); retired = an older version (retired.txt), which plays no more but whose games count;",
    "random = the random players. NEVER DELETE games.csv. Recipes from models.csv; written by league.py.",
    "",
    f"Updated {datetime.datetime.now():%Y-%m-%d %H:%M}: {tournaments} round-robins, {games} games, "
    f"{len(players)} players, {len(active)} models in play, {len(gone)} retired.",
    "",
] + table(by_rating(active))

out += ["", "## Best per archetype (models in play)", "",
        "| archetype | models | " + " | ".join(f"best at d{d}" for d in depths) + " |", "|---|---|" + "---|" * len(depths)]
archetypes = {}
for m in by_rating(active):
    archetypes.setdefault(models.get(m, {}).get("archetype", "?"), []).append(m)
for a, ms in sorted(archetypes.items(), key=lambda a: -max((elo(m, key_depth) or -1e9) for m in a[1])):
    best = []
    for d in depths:
        rated = [m for m in ms if elo(m, d) is not None]
        best.append(f"{max(rated, key=lambda m: elo(m, d))} {max(elo(m, d) for m in rated):.0f}" if rated else "–")
    out.append(f"| {a} | {len(ms)} | " + " | ".join(best) + " |")
if gone:
    out += ["", "## Retired", ""] + table(by_rating(gone))
if warnings:
    out += ["", "## Warnings", ""] + [f"- {w}" for w in warnings]
open(path("league.md"), "w").write("\n".join(out) + "\n")
print(f"league.md: {len(active)} models in play, {len(gone)} retired, {tournaments} round-robins, {games} games, "
      f"{len(warnings)} warnings")
