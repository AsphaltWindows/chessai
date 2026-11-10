#!/bin/bash
# Runs the training/tournament cycles one after another, until a file named STOP appears in this directory (checked
# between cycles; the loop removes it when it stops on it) or a cycle fails. Start it detached, from cycles/:
#     setsid nohup ./loop.sh >> loop.log 2>&1 < /dev/null &
# Stop it cleanly after the current cycle with:  touch /home/iv/dev/chessai/cycles/STOP
# Don't edit this file while the loop runs.
cd "$(dirname "$0")" || exit 1
echo "loop started $(date '+%F %T') (pid $$)"
while true; do
    if [ -e STOP ]; then
        rm -f STOP
        echo "STOP found: loop stopped $(date '+%F %T')"
        exit 0
    fi
    python3 -u cycle.py run --cycles 1
    code=$?
    if [ $code -ne 0 ]; then
        echo "cycle failed (exit $code): loop stopped $(date '+%F %T')"
        exit $code
    fi
done
