# The model library

Built 2026-11-12 ~10:00 at the user's request: "We should have a library of fully-plateaud models to choose from
for future tests in arenas, so that we don't have to retrain them".

The library is a catalogue of every trained line's final model. Each entry records whether its run plateaued, the
ply it trained at, its set-up, its seed and its lineage. Each file has a reflinked snapshot. Arenas pick their
entries from it, so a new question costs an arena (CPU-minutes) instead of a retrain (CPU-hours).

**Never delete anything under leagues/library/.** Library models are kept through every cleanup. As with any
deletion, ask the user first. The snapshots are btrfs reflinks of the experiments' files, so they cost nothing
while the originals exist.

## Files

| File | What it is |
|---|---|
| `index.csv` | One row per line of every experiment that ran (leagues/experiments), at its final version. The global league adds one row per line for its best champion in the 11-12 all-champions arena, plus a second row when the last champion differs. 393 rows, 254 of them plateaued. grow-l-p1c never ran and is skipped. |
| `snapshots.csv` | Every snapshotted file with its md5: the index rows plus extra versions, such as the cohort read's v279, v299 and finals. |
| `holdouts.csv` | The holdouts (columns `id,reason`; added 2026-11-12 for the dashboard's Models page, `/m/`): models kept for their interest beside the fully plateaued ones. An id is an index id (every row with it), `<id>@<version>` or `<experiment>:<line>@<version>`; a version the index doesn't have shows as its own row. Curated by hand: no script writes it. |
| `models/<experiment>/` | The snapshot, in the experiment's own layout: `config.json`, `lines.csv`, `runs/<line>/<kind><v>.model`. arena.py can read an entry from it directly. |
| `build_index.py` | Rebuilds the index, adds missing snapshots and re-checks md5s. `--extra EXP:LINE@V` snapshots more versions. It never deletes. |
| `library.py` | `list`, `tags` and `spec` choose arena entries; `smoke` is the load test. Usage is in its docstring. |
| `cohort_read.py` | The first use: the cohort-variance read (see models/RESULTS.md section 1, 11-12). |

## Columns of index.csv

**Identity**
- `id`: a short tag, safe in arena names: the experiment name without punctuation, `_`, then a letter in line order, e.g. `fponlyp2b_a`.
- `experiment`, `line`, `version`.
- `version_role`: final, best champion, last champion, or best and last champion.

**The model**
- `model`: the naming-scheme code with options.
- `archetype`, e.g. F-P or T-P-Ne.
- `kind`: lr, probit, cnbp, bdt or bdd.
- `line_status`: active or retired at the end.

**How the run ended**
- `train_ply`: the experiment's entry ply. Old-system runs without one get it from their tournament plies: `2`, or `mixed 1,2` when lines played at plies 1 and 2 next to random players. Self-play lines show their self-play depth.
- `end_reason`: `plateau` (the plateau rule ended the run), `cap` (the cycle cap), `manual-stop` (a STOP file) or `killed` (the global league). `extending` (2026-11-12): a run that reached its cap and runs on after `leagues.py extend LEAGUE --cycles N`. Until it ends again its row stays its final at the old cap (`end_cycle` and `cap` the old cap; `note` says so). When it ends, its new final is indexed with the true reason, and the `note` names the old final, whose snapshot stays.
- `plateaued`: `yes` only for `plateau`. **This is what "fully plateaued" means here.**
- `end_cycle`, `floor` (min_cycles), `cap`, `plateau_window`.
- `last_champion`: the line's last crowned version. The in-run gate at T 0.1 judged that one best.

**The set-up**
- `field`: the composition, e.g. `F-P x4`, or `F-P x2, T-P-Ne+... x2`.
- `field_class`: `pure` (one archetype) or `mixed`.
- `setup_group`: runs whose specs match apart from name, description, seed and tournament binary are seed replicates of one set-up. There are five pairs:
  - fponly-p2 / fponly-p2b
  - grow-p1b / grow-p1b2
  - probit-widen-p1b / probit-widen-p1c
  - flat-p2 / flat-p2-b
  - flat-p3 / flat-p3-b
- `seed`.
- `games_per_cycle`, if configured, and `rounds_per_cycle`.
- `train_games_per_cycle`: the median of the line's last 100 cycles, from cycles.csv.
- `self_games`, `ply1_entries`.
- `random_players`: the number of random references in the field (old system).
- `frozen_references`, `reference_window`, `replay`.
- `warm_start`: the spec's `start`.

**Lineage**
- `lineage` lists:
  - versions shared with another run: md5-identical versions among v1-v20, e.g. diversity-p2c/p2d to v9, fponly-p2/-720 at v1, probit-late-p2/replay-p2 at v1;
  - frozen references.

**Binaries and files**
- `train_binary`, `train_args`, `train_overrides`, `tournament_binary`.
- `spec`.
- `source_model` and `md5` (the experiment's file), and `snapshot`.
- `loads_hybrid17`: the smoke test.

## Choosing arena entries

```
python3 leagues/library/library.py list --archetype '^F-P$' --train-ply '^2$' --plateaued yes
python3 leagues/library/library.py tags --experiment '^fponly-p2b$' --versions final,final-20   # for make_split_read.py
python3 leagues/library/library.py spec my-read 2 ci15 --archetype '^F-P$' --train-ply '^2$' --plateaued yes \
    --versions final           # writes leagues/specs/arenas/my-read.json on chessai-hybrid17
```

- **Entry names.** Entries are named `ID<v>.L<k>@<v>`, so arm_panel.py, h2h_pool.py and h2h_btg.py group them by `ID<v>`.
- **Where entries are read from.** By default (`--source experiment`) they come from leagues/experiments, where arena.py checks each entry's calibration against its experiment's log. With `--source library` they come from the snapshot, which is the fallback if an experiment's directory is ever gone. The models and calibration settings are identical, but arena.py can't check the calibration (no tournament log in the snapshot).
- **Line-name limits.** `tags` covers only lines named `...#k`, because make_split_read.py needs the `#k`. The global league's lines and old single lines (`F-P.loc`) need `spec`.
- **Anchor (11-12 ~11:30).** `spec` writes no random player by default: the mean of every entry is pinned at Elo 0
  (`"anchor": {"mean_of": "field"}`; ARENA.md, Pinned anchor), so `ciX` stops. `--anchor random` writes the spec as
  before (one random reference; fixed round-robins only). cohort_read.py calls `write_spec` without an anchor, so its
  specs stay as before.
- **The global ladder** (one arena of promising finals, each at its own ply, on the standing panels' scale) has its
  own builder, leagues/specs/arenas/make_ladder.py, which uses this file's `rows`, `entry` and `replicate` (ARENA.md,
  The ladder builder).

## Load check (11-12)

All 22 (kind, training binary) combinations in the library load and play in chessai-hybrid17 (`library.py smoke`):
- bdd: errdag4, grow3, hybrid15.
- bdt: dag-q9, grow1, grow4, grow5, hybrid6, target, trigger.
- cnbp: hybrid4gfx, hybrid6.
- lr: grow4, grow5, hybrid4gfx, hybrid6.
- probit: grow1, grow3, grow4, hybrid4gfx, hybrid6, weight1.

Every row's `loads_hybrid17` says which smoke test covers it.

## Cautions

- **Rate entries only at their training ply.** A model's strength depends strongly on the ply it trained at (11-12 ply-3 and ply-2 panel reads).
- **Old-system runs** (11-09/11-10; `train_ply` `mixed 1,2` or a self-play depth, `random_players` 3-4) trained in fields mixed across plies and including random players. Use them as history, not as set-up comparisons.
- **The global league's champions** come from a co-evolved field with champion gating and no plateau rule. Don't pool them with the experiments' plateaued lines.
- **No ci stop with a random anchor.** Arenas whose only reference is one random player can't use a ci stop. The
  tournament rates no entry, because none is linked both ways to a random anchor that loses every game, so `ciX`
  ran on to 200 round-robins (found 11-12 on cohort-fp-p2). Since 11-12 ~11:30 `library.py spec` and
  make_split_read.py write the pinned anchor by default, where `ciX` stops; with `--anchor random` (`spec`) or
  `--random-anchor` (make_split_read.py) they still refuse `ciX`.

## First use: the cohort read (11-12)

`cohort_read.py` holds the design and the analysis; the result is in models/RESULTS.md section 1 (11-12 ~10:50).
- Plateaued F-Ps of one set-up are near-interchangeable: lines of a run agree within 0-7; seeds within 5 at ply 2
  and 16-20 at ply 1.
- At their finals, F-Ps from different set-ups land within ~20 of each other.
- The 11-10 run leaf-sources-new is the outlier (-245).

