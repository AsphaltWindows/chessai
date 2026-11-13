#!/bin/bash
# Cold read across experiments, one arena per ply (11-11 ~22:15): waits for WAITFOR's "experiment done", writes the
# arenas (make_cross_read.py), runs them one after another (ply order as given) and prints the pooled head-to-heads
# of the given tag pairs at each version and ply (h2h_btg.py; analyst 6.44's read rule).
#     cross_read.sh WAITFOR NAME VERSIONS PLIES "PAIRS" TAG=EXPERIMENT:REGEX [...]
# PAIRS: space-separated A:B tag pairs, without the version (each is read at every version), e.g. "rfp:pfp rtr:ptr".
set -u
cd /home/iv/dev/chessai
WAIT=$1; NAME=$2; VERS=$3; PLIES=$4; PAIRS=$5; shift 5
E=leagues/experiments/$WAIT
while ! tail -1 $E/repeat.log | grep -q "experiment done"; do sleep 60; done
tail -1 $E/repeat.log
python3 leagues/specs/arenas/make_cross_read.py "$NAME" "$VERS" "$PLIES" "$@" || exit 1
for p in ${PLIES//,/ }; do
    A=/home/iv/dev/chessai/leagues/arenas/diag/$NAME-p$p
    python3 leagues/arena.py create leagues/specs/arenas/$NAME-p$p.json 2>&1 | tail -1
    echo "ply $p start $(date +%T)"
    python3 -u leagues/arena.py run $A > $A/run.out 2>&1
    echo "ply $p end $(date +%T): $(tail -1 $A/run.out | cut -c1-150)"
    for v in ${VERS//,/ }; do
        s=$([ "$v" = final ] && echo F || echo "$v")
        for pr in $PAIRS; do
            python3 leagues/specs/arenas/h2h_btg.py $A/tour ${pr%%:*}$s ${pr##*:}$s
        done
    done
done
