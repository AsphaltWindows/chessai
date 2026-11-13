#!/bin/bash
# Batch 4's queue: one experiment at a time, in batch4.py's items order (the reruns ORIG-new in ORDER, then the extras:
# small-field-x4), beside whatever else runs. When an experiment's turn comes it is created from its spec (batch4.py
# create-item: a rerun is checked against its original, an extra against the experiment whose models it replicates)
# and run by leagues/repeat.sh until the driver says "experiment done" (exit 4: the plateau rule or the 1000-cycle
# cap); then the next starts. The queue stops (and says so in experiments/batch4-queue.txt) when an experiment ends
# any other way (STOP, a failed cycle, a failed create), or between experiments when experiments/batch4-queue.STOP
# exists (it doesn't stop the running experiment: use leagues.py stop for that). selfplay-depth is held and
# nb-settings-no-random3 is covered (batch4.py). An experiment already done is skipped, so a restarted queue goes on
# where it was.
# Start it detached:
#   setsid nohup /home/iv/dev/chessai/leagues/specs/batch4/queue.sh >> /home/iv/dev/chessai/leagues/experiments/batch4-queue.log 2>&1 < /dev/null &
# --after PID: first wait for that process (an earlier queue.sh, which runs the reruns) to exit, then go on only if
# every rerun is done (otherwise say so and exit: the earlier queue stopped, and something needs a look). This copy
# was started so on 2026-11-10 ~18:45, after the first queue (pid 1809879, 17:41), which runs the 17 reruns from the
# first version of this script (it expanded its order when it started), to add small-field-x4 at the end.
L=/home/iv/dev/chessai/leagues
B="$L/specs/batch4/batch4.py"
STOPQ="$L/experiments/batch4-queue.STOP"
echo "queue started $(date '+%F %T') (pid $$) $*"
if [ "$1" = "--after" ]; then
    wp="${2:?usage: queue.sh [--after PID]}"
    echo "waiting for pid $wp (the earlier queue) to exit $(date '+%F %T')"
    while kill -0 "$wp" 2> /dev/null && tr '\0' ' ' < "/proc/$wp/cmdline" 2> /dev/null | grep -q "batch4/queue.sh"; do
        sleep 60
    done
    echo "pid $wp exited $(date '+%F %T')"
    for o in $(python3 "$B" order); do
        if [ "$(python3 "$B" status "$o")" != "done" ]; then
            echo "$o-new isn't done after the earlier queue exited: this queue stops without starting anything $(date '+%F %T')"
            python3 "$B" state "" "The earlier queue (pid $wp) exited with $o-new not done; the follow-on queue (pid $$) stopped without starting anything $(date '+%F %T')."
            exit 1
        fi
    done
fi
for x in $(python3 "$B" items); do
    d="$L/experiments/$x"
    [ "$(python3 "$B" status-of "$x")" = "done" ] && continue
    if [ -f "$STOPQ" ]; then
        rm -f "$STOPQ"
        echo "batch4-queue.STOP found before $x: the queue stops $(date '+%F %T')"
        python3 "$B" state "" "The queue stopped before $x (batch4-queue.STOP), $(date '+%F %T')."
        exit 0
    fi
    if pgrep -f "[r]epeat.sh $d\$" > /dev/null; then
        echo "$x already has a repeat.sh: waiting for it $(date '+%F %T')"
        while pgrep -f "[r]epeat.sh $d\$" > /dev/null; do python3 "$B" state "$x" > /dev/null 2>&1; sleep 30; done
    else
        if [ ! -f "$d/config.json" ]; then
            if ! python3 "$B" create-item "$x"; then
                echo "create $x failed: the queue stops $(date '+%F %T')"
                python3 "$B" state "" "Creating $x failed; the queue stopped $(date '+%F %T')."
                exit 1
            fi
        fi
        echo "start $x $(date '+%F %T')"
        python3 "$B" state "$x"
        "$L/repeat.sh" "$d" >> "$d/repeat.log" 2>&1 < /dev/null &
        rp=$!
        while kill -0 "$rp" 2> /dev/null; do python3 "$B" state "$x" > /dev/null 2>&1; sleep 15; done
        wait "$rp"
        echo "repeat.sh of $x exited (code $?) $(date '+%F %T')"
    fi
    st=$(python3 "$B" status-of "$x")
    python3 "$B" state ""
    if [ "$st" != "done" ]; then
        echo "$x ended with status '$st', not done: the queue stops $(date '+%F %T')"
        python3 "$B" state "" "$x ended with status '$st'; the queue stopped $(date '+%F %T')."
        exit 1
    fi
    echo "$x done $(date '+%F %T')"
done
python3 "$B" state "" "Every experiment in the queue is done ($(date '+%F %T'))."
echo "queue finished $(date '+%F %T')"
