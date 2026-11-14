# Champion-rule simulation (2026-11-07)

Stage 1 of the cycle-rule redesign (the coordinator's): the champion rule simulated over cycles 1-46 from the existing
ratings and cycles.csv. No games were played, and the live loop, league files and lineages.txt weren't touched.

- `snap/`: read-only copies taken at the start of cycle 47, with md5s in `snap/MD5`.
  - From `selfplay/league-cycles`: ratings.csv (the fit after round-robin 240, cycle 46), games.csv, players.txt,
    retired.txt, elo_history.csv and .key.
  - From `cycles/`: cycles.csv, state.json, config.json, lineages.txt, and `md/cycle-*.md`, the source of each
    tournament's minutes.
- `sim.py` writes `report.md` and `results.json`. It runs in about 2 s.
  - `sim.py --trace LINEAGE` also prints that lineage's comparisons (gap/SE per cycle) for P8 k2 in each variant.
- Grid: patience 6, 8 and 10 by champion_se 1.5 and 2. Each setting runs in three variants:
  - `fit`: the fit's ratings as they are.
  - `tilt`: old champions lowered by 30, the drift check's tilt (`../drift-check`).
  - `washout`: a champion's rating moves toward its expected true strength as its games pile up. The expectation is
    shrunk toward the lineage's local level (a symmetric window of 3 cycles) by tau^2/(tau^2 + se^2). Tau, about 25,
    comes robustly from consecutive versions.

## Approximations

- Versions are the actual ones. Under the rule, lineages that were dormant would have self-played. The simulation
  uses their own-step versions, which change less.
- After a lineage's actual end, when the rule froze or retired it, there are no versions. The simulation then assumes
  no new champion; those finishes are marked `*`.
- Ratings come from the fit after cycle 46 rather than each day's fit. Under the `fit` variant, a champion's rating
  stays the one its own games gave it.
- The SE of a difference combines independent SEs from the fit's ci95. The live covariance SEs are 0.99 of that.
  - The current version's SE uses its games up to that cycle.
  - The champion's SE uses its games while it was current, plus one entry's games for each cycle after that.
- Milestones: the champion's ply-2 rating, as the variant has it, within 10 of an untaken multiple of 100.
  - The references take 0, 200, 300 and 400.
  - Claims go in finishing order, the closest first within a cycle.
- Self-play costs the actual CPU-s, or 240 in a cycle where the lineage didn't self-play.
- A gate costs the lineage's actual gate CPU-s, or its median. At 500 and 250 games it costs half and a quarter.
- Round-robins have `rrs * K * (K-1)` games, where K counts:
  - the references;
  - 3 entries per active lineage, or 2 in its first cycle;
  - 1 per frozen champion.
- Round-robin minutes scale the actual minutes by the ply-2 entries' games, with a ply-1 game side at 1/20 of a ply-2
  side (costs.csv: LR self-play at d1 against d2).
  - random-3 and random-4 add 0.043 s per game side. That figure comes from the jump when they joined in cycle 5.
  - The other references are left out.
- The refresh at cycle 25 is 250 games per finished champion, as ply-2 against ply-2.
- The own step's new training data isn't simulated: only the current version's games count, and champions are
  opponents.
