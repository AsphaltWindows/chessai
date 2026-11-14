# Drift check (2026-11-07)

Is the downward rating drift of retired league entries (`selfplay/league-cycles`) justified, or should it be reversed?
Everything here is outside the league: no game was added to the league, and the loop wasn't touched.

- `snapshot-rr210/`: the league's files after round-robin 210 (end of cycle 40's tournament), with their md5s.
- `part1.py` -> `part1.out`, `drift.csv`: the free check, no games.
  - Residuals of current-cycle entries' games by opponent category, under the current fit.
  - The favourite's residual by predicted score.
  - Drift (rating now minus rating after the entry's last round-robin, from `elo_history.csv`) against the share of
    games with long-lived opponents, raw and within retirement batches.
- `select.txt`, `players.txt`: the replay test's 24 retired ply-2 entries and its panel of 8 current ply-2 entries.
  - The retired entries are idle 25 or more round-robins: 3 from each of 8 retirement batches.
  - Specs are copied exactly from the snapshot's `retired.txt` and `players.txt`.
- `games-pilot.txt`, `games-rest.txt`: the games, `T WHITE BLACK`.
  - 22 per pair, both colours, t 0-10.
  - Played by `chessai-probit replay-games` (the tournament's policy: log-odds-softmax at 0.1, `--use-probs`, seed
    20261107) at nice 19 with 3 threads.
- `pilot.csv`, `rest.csv`, `games/`: the results.
- `part2.py` -> `part2.out`: per-entry and per-batch scores against the predictions under "now" and "at the last game".
  It also fits alpha (rating = now + alpha * (last - now)), with and without a common offset delta.
- `part2b.py` -> `part2b.out`: the same per panel member, and alpha leaving each panel member out.

Cost: 22 CPU-s pilot plus 287 CPU-s for the remaining 4,032 games, and about 12 CPU-s of analysis (about 5.4 CPU-min).

Control (the coordinator's, same day): `control.py` (setup, analyze) -> `control.out`.
- Entries still playing at round-robin 210 play the same panel with the identical setup, 22 games per pair: 1,056 games.
  - Frozen: MoE-c-v373-2, T-L-LLs-c-v651-2 and F-N-c-v1691-2.
  - Current: F-L-d2jit-c-v3943-2, T-L-Ne-c-v1425-2 and T-L-Le-c-v1176-2.
- Files: `players-control.txt` (part2's players file with the 6 appended), `games-control.txt`, `control.csv`,
  `games-control/`.
- The league's head-to-head for the same pairs in cycles 36-40 comes from the snapshot's `games.csv`.
- Cost: 98 CPU-s for the games and about 2 CPU-s of analysis.
- Results:
  - The control entries scored 51.3% against a predicted 49.4%, 1.9 points above (SE 1.6). The league head-to-head
    for those pairs was 49.1% against a predicted 49.0%.
  - Part2's retired entries scored 5.5 points below prediction (SE 0.9).
  - So the replay setup isn't biased, and the retired shortfall is real.

F-L-d2jit-c's freeze (the coordinator's, same day): `fl.py` (setup, analyze) -> `fl.out`.
- Was the rule's comparison with its idle peak biased? The setup is the same as the control's.
  - Its retired ply-2 versions v3348 (the peak), v3711 and v1651 played the panel, 22 games per pair.
  - Each also played v3943-2 (current when frozen), 88 games per pair: 792 games.
- Files: `players-fl.txt`, `games-fl.txt`, `fl.csv`, `games-fl/`.
- A joint ML fit (the panel held at the snapshot fit) puts the 3 old versions on average level with v3943-2:
  0 ± 36 Elo (95%), where the fit has them 20 above. In head-to-heads they scored 49.4% in 264 games, against a
  predicted 52.9%.
- Cost: 53 CPU-s.

`snapshot-rr210/` (60 MB) and `games/` (82 MB) can be deleted once the results are no longer needed.
