#!/bin/bash
# Cold read of an error-DAG experiment at its end (11-11): waits for "experiment done", fills the cold-read DRAFT's
# FINAL with the lines' final version, runs the arena and the arm contrasts (arm_btg.py), then copies the run's last
# pool rounds into round0/, round1/, ... (the layout path-stats reads) and runs path-stats on each DAG line's final
# model: sibling sets, cells, non-home arrivals at the leaves, how much the path tables move log-odds and routing.
#     dag_read.sh EXPERIMENT DRAFT_SPEC BASE_TAG BINARY
set -u
cd /home/iv/dev/chessai
EXP=$1; DRAFT=$2; BASE=$3; BIN=$4
E=leagues/experiments/$EXP
while ! tail -1 $E/repeat.log | grep -q "experiment done"; do sleep 60; done
tail -1 $E/repeat.log
OUT=leagues/specs/arenas/$EXP-cold.json
python3 - "$EXP" "$DRAFT" "$OUT" <<'EOF'
import csv, json, sys, os
exp, draft, out = sys.argv[1:4]
rows = list(csv.DictReader(open(f'/home/iv/dev/chessai/leagues/experiments/{exp}/lines.csv')))
vs = {r['version'] for r in rows}
assert len(vs) == 1, vs
v = vs.pop()
s = json.load(open(draft))
txt = json.dumps(s).replace('@FINAL', f'@{v}').replace('vFINAL', f'v{v}')
s = json.loads(txt)
s['description'] = s['description'].replace('DRAFT (11-11): ', '').replace(
    'Before creating, replace FINAL with the experiment\'s last version (every line has the same). ', '')
assert 'FINAL' not in json.dumps(s['entries'])
assert not os.path.exists(out), out
json.dump(s, open(out, 'w'), indent=1)
print('final version', v, [e['name'] for e in s['entries']])
EOF
python3 leagues/arena.py create $OUT 2>&1 | tail -1
A=/home/iv/dev/chessai/leagues/arenas/diag/$EXP-cold
date +%T
python3 -u leagues/arena.py run $A > $A/run.out 2>&1
date +%T
tail -2 $A/run.out | cut -c1-200
python3 leagues/specs/arenas/arm_btg.py $A/tour $BASE
# structural read on the run's last pool rounds: path-stats reads round-format directories (white_wins.games,
# black_wins.games, draws.games side by side), so each pool round's game directories are merged into one part
S=/home/iv/dev/chessai/models/from-scratch/errdag/final-$EXP
mkdir -p $S/parts
k=0
for r in $(ls -d $E/pool/round* | sort -V); do
    o=$S/parts/p$k; mkdir -p $o
    for c in white_wins black_wins draws; do cat $r/*/$c.games > $o/$c.games 2>/dev/null; done
    k=$((k+1))
done
echo "pool rounds merged: $k"
for d in $E/runs/D-*.loc; do
    m=$(ls -t "$d"/*.model | head -1)
    echo "== $(basename "$d") $(basename "$m"): $(head -1 "$m")"
    nice -n 15 $BIN path-stats "bdd:$m" $S/parts/p* --use-probs 2>&1 | grep -v "^node .* router member - cells \[\]" | cut -c1-220
done
