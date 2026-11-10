#!/bin/bash
# Runs a league's cycles one after another, one driver process per cycle (leagues.py run LEAGUE --cycles 1), until a
# file named STOP appears in the league's directory (the driver checks it between cycles and while an experiment
# waits for global rounds, and removes it when it stops on it), a cycle fails, or an experiment is done.
# Start it detached:
#     setsid nohup /home/iv/dev/chessai/leagues/repeat.sh LEAGUE_DIR >> LEAGUE_DIR/repeat.log 2>&1 < /dev/null &
# Stop it after the current cycle:  python3 /home/iv/dev/chessai/leagues/leagues.py stop LEAGUE_DIR
# The nice level, threads and jobs come from the league's config.json (the driver applies them to its children).
here="$(cd "$(dirname "$0")" && pwd)"
dir="$(cd "${1:?usage: repeat.sh LEAGUE_DIR}" && pwd)" || exit 1
echo "repeat started $(date '+%F %T') (pid $$) for $dir"
while true; do
    python3 -u "$here/leagues.py" run "$dir" --cycles 1
    code=$?
    case $code in
        0) ;;
        3) echo "STOP found: stopped $(date '+%F %T')"; exit 0 ;;
        4) echo "experiment done $(date '+%F %T')"; exit 0 ;;
        *) echo "cycle failed (exit $code): stopped $(date '+%F %T')"; exit $code ;;
    esac
done
