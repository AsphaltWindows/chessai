#!/bin/bash
# Stage-1 cold read of a ply-2 experiment (11-11): waits for "experiment done", writes the read (make_stage_read.py),
# runs it and prints the contrasts against the flat tag at each version.
#     stage_read.sh EXPERIMENT VERSIONS PLIES FLATTAG TAG=REGEX [TAG=REGEX ...]
# e.g. stage_read.sh base-lr-p2 300,final 1,2,3 fl fl=^F-L dag=^D-L tree=^T-L
set -u
cd /home/iv/dev/chessai
EXP=$1; VERS=$2; PLIES=$3; BASE=$4; shift 4
E=leagues/experiments/$EXP
while ! tail -1 $E/repeat.log | grep -q "experiment done"; do sleep 60; done
tail -1 $E/repeat.log
python3 leagues/specs/arenas/make_stage_read.py "$EXP" "$VERS" "$PLIES" "$@" || exit 1
python3 leagues/arena.py create leagues/specs/arenas/$EXP-stage.json 2>&1 | tail -1
A=/home/iv/dev/chessai/leagues/arenas/diag/$EXP-stage
date +%T
python3 -u leagues/arena.py run $A > $A/run.out 2>&1
date +%T
tail -2 $A/run.out | cut -c1-200
for v in ${VERS//,/ }; do
    t=$BASE$([ "$v" = final ] && echo F || echo "$v")
    echo "== contrasts against $t"
    python3 leagues/specs/arenas/arm_btg.py $A/tour $t
done
# 11-11 ~21:10 (analyst 6.44): the verdict of each contrast is its pooled head-to-head at the same version and ply
# (h2h_btg.py); BT over a field that mixes versions or training fields can invert it.
for v in ${VERS//,/ }; do
    s=$([ "$v" = final ] && echo F || echo "$v")
    for tr in "$@"; do
        t=${tr%%=*}
        [ "$t" = "$BASE" ] && continue
        python3 leagues/specs/arenas/h2h_btg.py $A/tour $t$s $BASE$s
    done
done
