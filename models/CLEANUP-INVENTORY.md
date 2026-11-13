# Cleanup inventory

> **TIER 1 RUN 2026-11-12 21:38-21:43 (the user approved tier 1 at ~21:18).** Lists regenerated first (refs.py, exp_breakdown.py, make_report.py; verify_lists.py: 0 violations): 1a 74 dirs and 1c 22,449 entries deleted as listed; 1b deleted 139 of 165 dirs: the caches of the 13 cap-ended runs that may be extended (std-v1-p1-s1/-s2, treeonly-p1 and its 4 arms, fponly-p1-coldtrain/-t002, grow-p1, flat-p2-w3, flat-p3-w3f1, flat-p3-w4) KEPT; the deleted list is inventory/tier1b_exp_caches.deleted-2026-11-12.txt. df: 370 -> 356 GB used (btrfs shared extents: much of 1c shared data with experiments). Tiers 2 and 3 untouched; regenerate their lists again before any run.

Built 2026-11-12 21:22 by the inventory fork. **Read-only: nothing was deleted, moved or renamed.** Scratch work, scripts and every list are in `models/from-scratch/inventory/`. GB = 10^9 bytes. "Size" is allocated size (du); "excl." is btrfs exclusive, which is what deleting frees when copies share extents (reflinks). Library snapshots and arena model copies are reflinks, so deleting an original they share frees nothing and loses nothing.

Disk: /home 1021.43 GB, 397.04 GB used, 618.53 GB free.

## 1. Totals by category

| Area | Size GB | Excl. GB | Status | Contents |
|---|---:|---:|---|---|
| leagues/experiments | 123.63 |  | 88 finished, 4 running or created | unreferenced versions 61.24, records (tour/, cold/) 53.21 of which elo_history.csv 34.86, caches 2.36, kept versions 4.07, pools 1.03, logs 0.77; running runs 0.73 |
| cycles/runs | 78.37 | 76.09 | stopped 11-07 | the first cycle system: 93 lines, ~170,000 version files, states |
| cycles/champ | 5.48 | 2.99 | stopped 11-09 | the champion run: staged cycles, owncal, records |
| models/from-scratch/fastbin | 157.13 | ~33.39 | build scratch | speedup builds: byte-identity dry runs, sources, bench |
| models/from-scratch/twin | 46.96 | 44.04 | 11-09 study | twin-league study (league/ holds most of it) |
| models/from-scratch/offline-ply | 20.98 | 20.98 | 11-11 study | offline ply study: pools/ (20 model files are arena sources) |
| models/from-scratch/drift | 11.85 | 11.85 | 11-10 study | drift study |
| models/from-scratch (the rest) | 76.88 | ~62.14 | mixed | old studies, *-src copies, build dirs, notes, bin/ |
| leagues/arenas/.owncal-cache | 10.44 | 9.10 | live cache | arena own-calibration games, ~11 MB per entry |
| leagues/arenas/diag | 3.51 | 0.76 | results | diagnostic arenas: configs, reflinked model copies, tours |
| selfplay | 1.68 | 1.16 | old | pre-league self-play and the old leagues' records |
| games | 0.44 | 0.44 | mixed | held-out game sets |
| leagues-smoke | 0.32 | 0.29 | old | smoke and price-calibration runs |
| leagues/global | 0.17 | 0.09 | live | the global ladder (210 champions) |
| leagues/library | 0.15 |  | keep | the model library: snapshots |
| rs | 0.56 |  | source | Rust source and its target/ |

fastbin, twin and offline-ply: btrfs exclusive figures are in section 3 where measured; dry runs are often reflink copies of experiments, so their exclusive share can be far below their size.

## 2. Library state

- `leagues/library/index.csv` (built 11-12 19:54): 481 rows over 89 experiments; 1420 snapshots in `snapshots.csv`; `leagues/library/` is 0.15 GB (reflinks of the originals). Library models are never deleted and no list below touches them.
- **Stale.** 0 finished experiments are missing: none. 0 have interim rows from before they finished: .
- Not yet eligible (running or created): grow-l-p1c, std-v1-p2-s1, std-v1-p2-s2, std-v1-p2-x3.
- **Step 0 before any deletion:** rerun `leagues/library/build_index.py` (adds snapshots, never deletes), then regenerate the lists (section 6). The finals it would add are already kept as line finals, so the lists should barely move, but the rebuilt index is the record of what stays.

## 3. Delete candidates, by tier

Each tier's list file holds literal absolute paths, one per line. "Frees" is the listed allocated size; section 1 and the per-root tables give btrfs exclusive figures where measured.

### Tier 1: scratch and regenerable (safest)

| List | What | Items | Frees GB | Why |
|---|---|---:|---:|---|
| `tier1a_rust_targets.txt` | Rust build dirs (target/) under models/from-scratch | 74 dirs | 7.47 | cargo rebuilds them; the bin/ binaries are separate copies. Source (src/, Cargo.*) stays. Skipped as in use: models/from-scratch/fastbin/hybrid19-src/target. |
| `tier1b_exp_caches.txt` | Caches of finished experiments (scan-cache/, owncal/) | 165 dirs | 1.82 | Rebuilt on demand; the runs are done or stopped. |
| `tier1c_build_scratch.txt` | Build and verification scratch (fastbin build dirs, *-build dirs, listed scratch) | 22,449 entries | 33.54 | Byte-identity replays, dry runs and selftests of builds that are deployed and written up in fastbin/PERF-LEDGER.md and the dirs' summary.txt. Notes, scripts, patches, binaries, backups and small (< 64 KiB) result files stay, as do the bench inputs (calsets, trainset, fx, nogames) and anything touched in the last 3 hours. Entries are files or whole subdirs whose every file is a candidate. |

### Tier 2: unreferenced old models and data

| List | What | Items | Frees GB | Why |
|---|---|---:|---:|---|
| `tier2a_versions_all.txt` | Experiment versions, unreferenced (all) | 262,908 files | 61.24 | Every version file of a finished experiment that no keep reason names. Kept: each line's final and champion versions, the library snapshots' originals, arena sources, the panels, pending reads and warm starts. Sampled 97.5% exclusive, so it frees about its size. |
| `tier2a_versions_thin50.txt` | Experiment versions, unreferenced, keeping every 50th version | 257,784 files | 60.15 (alt.) | Alternative to the line above: also keeps v0, v50, v100 ... of every line as a lineage trail. |
| `tier2b_cycles_versions_all.txt` | Old cycle system (cycles/runs) versions (all but finals and referenced) | 173,233 files | 67.34 | The first run (stopped 11-07) and its 93 lines. Kept: each line's final version and the 27 version files that arena configs name. |
| `tier2b_cycles_versions_thin50.txt` | Old cycle system versions, keeping every 50th version | 169,815 files | 66.01 (alt.) | Alternative to the line above. |
| `tier2c_cycles_states_files.txt` | Old cycle system: superseded top-level .state files | 2,883 files | 2.81 | Each line keeps its latest state (resume or warm start), 54 files, 0.09 GB. |
| `tier2c_cycles_states_dirs.txt` | Old cycle system: per-cycle state dirs (cycle-states/, reverted/) | 143 dirs | 11.00 | Snapshots of every cycle's states and the versions that cycles reverted. |
| `tier2d_champ_run.txt` | Champion run (cycles/champ): staged cycles, owncal, backups | 324 entries | 2.99 | Stopped 11-09 after cycle 75. Its cycle reports, cycles.csv, costs.csv, logs and dashboard stay. Frees capped at cycles/champ's btrfs exclusive size. |
| `tier2e_fromscratch_studies.txt` | Old studies in models/from-scratch (unreferenced data and models) | 124,003 entries | 120.55 | Results are in the notes (NOTES.md, summary.txt, analysis.txt), which stay with every script and small result file; games, models and states go whatever their size. The 26 arena-referenced files (offline-ply/models, matched-depth) stay. |
| `tier2f_games_heldout.txt` | Unused held-out game sets (games/) | 6 dirs | 0.39 | No spec, arena config or default names them. |
| `tier2g_exp_pools.txt` | Game pools of done experiments (replaypool, frozenref, pool, topuppool, selfpool) | 3 dirs | 0.13 | Training pools; a done run never reads them again. Stopped runs' pools are under Unsure. |
| `tier2h_selfplay_old.txt` | Old self-play leagues no dashboard reads (selfplay/league, diag-tan, session) | 455 entries | 0.47 | Pre-cycle-system data (11-05/11-06). selfplay/league-champ and league-cycles are read by the old runs' dashboards: Unsure. |
| `tier2i_owncal_superseded.txt` | Arena own-calibration cache entries for superseded binaries | 350 dirs | 3.95 | Each is ~11 MB of calibration games keyed by (model, held-out set, binary). Only a rerun of an old arena pinned to that binary reads one, and it would recompute it. Kept: chessai-hybrid16 and -hybrid18 entries and every entry made since the scan. |

### Tier 3: compress or archive, do not delete

| List | What | Items | Frees GB | Why |
|---|---|---:|---:|---|
| `tier3a_elo_history_zstd.txt` | elo_history.csv of finished experiments: compress with zstd -3 | 100 files | 34.85 | No leagues/ tool reads it; only the done t01check scripts did. zstd -3 measured 10.2x on 300 MB of error-targets-probit (zstd -19 --long 17.5x but ~100x slower). |
| `tier3b_before_backups.txt` | *.before-* backups of source and notes: archive into one tar.zst | 185 files | 0.03 | Pre-deploy copies (kept so far as rollback points). Archive, verify the archive, then remove. The backups in leagues/library and leagues/specs are left out (protected trees). |


**Build and verification scratch (fastbin build dirs, *-build dirs, listed scratch), by root.** Listed = allocated size of the listed files; root excl. = btrfs exclusive size of the whole root where measured (a dry run that reflinks an experiment frees only this); kept = what stays.

| Root | Entries | Listed GB | Root excl. GB | Kept GB |
|---|---:|---:|---:|---:|
| `models/from-scratch/fastbin/hybrid16` | 984 | 64.71 | 0.78 | 1.12 |
| `models/from-scratch/fastbin/hybrid8` | 155 | 16.81 | 2.90 | 0.03 |
| `models/from-scratch/fastbin/hybrid9` | 98 | 15.87 | 0.81 | 0.02 |
| `models/from-scratch/fastbin/errdag4` | 542 | 13.73 | 2.84 | 0.17 |
| `models/from-scratch/fastbin/hybrid14` | 769 | 11.63 | 1.81 | 0.15 |
| `models/from-scratch/fastbin/review-1112a` | 35 | 11.46 | 0.19 | 0.09 |
| `models/from-scratch/fastbin/lrdecay1` | 331 | 5.64 | 5.64 | 0.01 |
| `models/from-scratch/fastbin/hybrid6` | 1,177 | 3.71 | 3.32 | 0.01 |
| `models/from-scratch/filter-build` | 5,992 | 3.63 | 1.45 | 0.15 |
| `models/from-scratch/fastbin/hybrid17` | 78 | 2.30 | 0.22 | 0.01 |
| `models/from-scratch/fastbin/grow5` | 17 | 1.56 | 1.56 | 0.01 |
| `models/from-scratch/openings-build` | 2,055 | 1.55 | 1.39 | 0.03 |
| `models/from-scratch/fastbin/grow6` | 46 | 1.41 | 1.41 | 0.01 |
| `models/from-scratch/fastbin/hybrid18` | 170 | 1.20 | 0.38 | 0.02 |
| `leagues/arenas/diag/cohort-fp-p2/owncal` | 85 | 0.93 |  | 0.00 |
| `models/from-scratch/fastbin/grow2` | 414 | 0.90 | 0.90 | 0.01 |
| `models/from-scratch/fastbin/grow1` | 307 | 0.83 | 0.83 | 0.01 |
| `leagues/experiments/replay-p2/replaypool` | 49 | 0.82 |  | 0.00 |
| `models/from-scratch/fastbin/grow4` | 304 | 0.80 | 0.80 | 0.01 |
| `models/from-scratch/fastbin/review-1111b` | 370 | 0.80 | 0.79 | 0.13 |
| `models/from-scratch/fastbin/grow3` | 292 | 0.79 | 0.79 | 0.01 |
| `models/from-scratch/fastbin/hybrid15` | 261 | 0.74 | 0.75 | 0.12 |
| `models/from-scratch/refwindow` | 3,733 | 0.73 | 0.72 | 0.02 |
| `models/from-scratch/fastbin/review-1111c` | 26 | 0.64 | 0.28 | 0.01 |
| `models/from-scratch/observers-build` | 1,891 | 0.25 | 0.26 | 0.02 |
| `models/from-scratch/coldgames-build` | 1,892 | 0.25 | 0.26 | 0.02 |
| `models/from-scratch/fastbin/grow7` | 178 | 0.19 |  | 0.58 |
| `models/from-scratch/fastbin/p3diff` | 41 | 0.16 | 0.06 | 0.00 |
| `models/from-scratch/fastbin/review-1112b` | 32 | 0.10 | 0.10 | 0.02 |
| `models/from-scratch/fastbin/hybrid10` | 2 | 0.07 |  | 0.00 |
| 9 smaller roots | 123 | 0.31 | | 0.28 |

**Old studies in models/from-scratch (unreferenced data and models), by root.** Listed = allocated size of the listed files; root excl. = btrfs exclusive size of the whole root where measured (a dry run that reflinks an experiment frees only this); kept = what stays.

| Root | Entries | Listed GB | Root excl. GB | Kept GB |
|---|---:|---:|---:|---:|
| `models/from-scratch/twin` | 687 | 47.47 | 44.04 | 0.15 |
| `models/from-scratch/offline-ply` | 1,928 | 24.99 | 20.98 | 0.01 |
| `models/from-scratch/drift` | 340 | 12.19 | 11.85 | 0.01 |
| `models/from-scratch/movegen` | 28,587 | 2.42 | 1.87 | 0.47 |
| `models/from-scratch/spdrop` | 52 | 1.97 | 1.60 | 0.00 |
| `models/from-scratch/modelopt` | 484 | 1.67 | 1.52 | 0.13 |
| `models/from-scratch/T-L-LLs-s21` | 1,000 | 1.30 | 1.30 | 0.00 |
| `models/from-scratch/T-L-Le-s21` | 1,000 | 1.27 | 1.27 | 0.00 |
| `models/from-scratch/degeneracy` | 73 | 1.26 | 1.22 | 0.11 |
| `models/from-scratch/polyak` | 171 | 1.23 | 0.96 | 0.00 |
| `models/from-scratch/batch28` | 6,060 | 1.22 | 1.23 | 0.00 |
| `models/from-scratch/T-L-NLs-s21` | 1,000 | 1.14 | 1.14 | 0.00 |
| `models/from-scratch/leaguecal` | 103 | 1.09 | 0.55 | 0.00 |
| `models/from-scratch/trigger` | 807 | 1.03 | 1.03 | 0.00 |
| `models/from-scratch/T-L-Ne-s21` | 1,000 | 1.01 | 1.01 | 0.00 |
| `models/from-scratch/T-L-Ne-soft-s21` | 1,000 | 0.99 | 0.99 | 0.00 |
| `models/from-scratch/T-L-NNs-s21` | 1,000 | 0.99 | 0.99 | 0.00 |
| `models/from-scratch/T-L-Nc-s21` | 1,000 | 0.98 | 0.98 | 0.00 |
| `models/from-scratch/F-FM-s21` | 1,000 | 0.96 | 0.96 | 0.00 |
| `models/from-scratch/T-L-Ne-reset-s21` | 1,000 | 0.96 | 0.96 | 0.00 |
| `models/from-scratch/target` | 470 | 0.94 | 0.94 | 0.00 |
| `models/from-scratch/routercmp` | 83 | 0.94 | 0.93 | 0.03 |
| `models/from-scratch/lranchor` | 929 | 0.92 | 0.91 | 0.11 |
| `models/from-scratch/T-L-Ne-merge-s21-r2000` | 2,001 | 0.90 | 0.90 | 0.00 |
| `models/from-scratch/lrprobit` | 143 | 0.85 | 0.17 | 0.00 |
| `models/from-scratch/resid` | 1,408 | 0.85 | 0.74 | 0.23 |
| `models/from-scratch/solo` | 37 | 0.79 | 0.36 | 0.03 |
| `models/from-scratch/errdag` | 180 | 0.75 | 0.62 | 0.01 |
| `models/from-scratch/std-v1` | 2,033 | 0.72 |  | 0.03 |
| `models/from-scratch/optimism` | 1 | 0.62 | 0.47 | 0.00 |
| 96 smaller roots | 68,426 | 17.86 | | 3.67 |

**Champion run (cycles/champ): staged cycles, owncal, backups, by root.** Listed = allocated size of the listed files; root excl. = btrfs exclusive size of the whole root where measured (a dry run that reflinks an experiment frees only this); kept = what stays.

| Root | Entries | Listed GB | Root excl. GB | Kept GB |
|---|---:|---:|---:|---:|
| `cycles/champ/staged-c45` | 162 | 3.55 |  | 0.06 |
| `cycles/champ/owncal` | 154 | 1.65 |  | 0.04 |
| `cycles/champ/backup-20261109-035014` | 4 | 0.13 |  | 0.00 |
| `cycles/champ/backup-20261109-014732` | 4 | 0.12 |  | 0.00 |

**Own-calibration entries by binary** (tier 2i): chessai-hybrid13 1.08, chessai-hybrid17 0.95, chessai-hybrid9 0.57, chessai-hybrid10 0.35, chessai-hybrid15 0.31, chessai-hybrid8 0.27, chessai-errdag4 0.26, chessai-hybrid5 0.07, chessai-hybrid11 0.04, chessai-hybrid4gfx 0.04.

**Tier totals** (alternatives excluded): tier 1 42.83 GB, tier 2 270.87 GB (with the thin-50 alternatives instead: 268.45 GB), tier 3 about 34.88 GB.

**Biggest single roots, by estimated frees:** `models/from-scratch/twin` 44.04 GB; `models/from-scratch/offline-ply` 20.98 GB; `models/from-scratch/drift` 11.85 GB; `models/from-scratch/fastbin/lrdecay1` 5.64 GB; `models/from-scratch/fastbin/hybrid6` 3.32 GB; `models/from-scratch/fastbin/hybrid8` 2.90 GB; `models/from-scratch/fastbin/errdag4` 2.84 GB; `models/from-scratch/movegen` 1.87 GB.

## 4. Keep list (reasons)

- **Referenced paths: 7,580.** By reason (a path can have several): library 1,645; library snapshot 1,420; line final/current 749; library index row 707; line champion 485; arena spec ladder-2.json 481; arena ladder/ladder-2: entry source 323; arena ladder/ladder-2: own copy 323; arena diag/global-all-champions-p1: entry source 210; arena spec global-all-champions-p1.json 210; arena diag/global-all-champions-p1: own copy 210; arena diag/cohort-fp-p2: entry source 85; arena spec cohort-fp-p2.json 85; arena diag/cohort-fp-p2: own copy 85.
- **Kept whole while their reads run:** fponly-p1-open (0.32 GB), grow-dag-p1 (3.94 GB), treeonly-p1-open (1.29 GB).
- **Running or created experiments, whole:** grow-l-p1c (0.00 GB), std-v1-p2-s1 (0.22 GB), std-v1-p2-s2 (0.23 GB), std-v1-p2-x3 (0.27 GB).
- **Every finished experiment's line finals and champions**, its logs, lines.csv, specs, notes and the small tour files (head_to_head, ratings); the big tour files are tier 3 or Unsure.
- **leagues/library** (never deleted), **leagues/global** (the global ladder's 210 champions), **every arena dir** (configs, reflinked model copies and tours are the results' evidence; diag is 0.76 GB exclusive), and the own-calibration entries of chessai-hybrid16 and -hybrid18.
- **games/heldout-lr-d2-s555 and games/heldout-cnbp-d2-s555**: leagues.py's default held-out sets and every spec's.
- **models/from-scratch:** bin/ (binaries), every *-src dir (source; only its target/ is tier 1a), theory, theory-2, global11-drafts (read scripts and results), t01check (evidence, 6 MB), inventory (this), treeview-cache (the dashboard reads its pages and game snapshots), fastbin's bench inputs (calsets, trainset, fx, nogames), PERF-LEDGER.md and every summary.txt, and in use now: rateevery-build, ply4-prep, fastbin/hybrid19-src.
- **Source, notes, specs** (leagues/*.py, *.md, leagues/specs, rs/, c_src, py_src, .git): never candidates.
- **Old cycle system:** each line's final version and latest state, the 27 arena-referenced versions, cycle reports, cycles.csv, costs.csv and logs.

## 5. Unsure (what decides it)

| Item | Size GB | Decides it |
|---|---:|---|
| games.csv and game_cpu.csv of finished experiments | 12.30 | arena.py, leagues.py, dashboard.py, make_ladder.py, calibrate.py and the read scripts open them; compressing needs zstd-aware readers, deleting loses the dashboard's run pages. Keep unless the old runs' pages can go. |
| selfplay/league-champ, selfplay/league-cycles | 1.17 | Read by the old runs' dashboards (cycles/champ/dashboard.py, linked from the main dashboard). Delete only if those pages may go stale. |
| Pools of the stopped runs (cluster-error, lr-settings, nb-settings, nb-settings-no-random3, probit-widen-p1) | 0.00 | Whether any stopped run might resume. Their versions are in tier 2a either way (finals kept). |
| leagues-smoke | 0.29 excl. | Price-calibration evidence (prices.csv was fitted from it) or selftest scratch: the price notes decide. |
| rs/target | 0.56 (rs/) | Rust build output, but rs/ is off-limits to us: your call. |
| models/from-scratch/treeview | 0.41 | The tree viewer's development dir; its data is in tier 2e only if treeview.py no longer needs it. |

## 6. Proposed commands (NOT run)

**Order matters for what is freed.** The fastbin dry runs (tier 1c) are reflinks of experiments' big files (elo_history.csv, versions): alone they free only their exclusive share (~32 GB of 163 GB listed), and while they exist, compressing elo_history.csv (tier 3a) or deleting versions (2a) frees less. Run 1c first, then 2a and 3a; together they free more than the sum of the separate estimates.

Nothing below has been run. Rules: confirm each tier first; library files, bin/ binaries, source and notes are in no list; every list holds literal absolute paths frozen at the time above. Review a list (`wc -l`, `head`, `grep -c`) before its line.

```bash
# Step 0: rebuild the library index, then regenerate every list (the lists shrink or stay; nothing new is added
# without a fresh keep check)
cd /home/iv/dev/chessai
python3 leagues/library/build_index.py
python3 models/from-scratch/inventory/refs.py && python3 models/from-scratch/inventory/exp_breakdown.py \
  && python3 models/from-scratch/inventory/make_report.py
```

**Tier 1: scratch and regenerable (safest)**

```bash
# tier1a_rust_targets: 74 dirs, frees 7.47 GB
xargs -d '\n' -a /home/iv/dev/chessai/models/from-scratch/inventory/tier1a_rust_targets.txt rm -rf --
# tier1b_exp_caches: 165 dirs, frees 1.82 GB
xargs -d '\n' -a /home/iv/dev/chessai/models/from-scratch/inventory/tier1b_exp_caches.txt rm -rf --
# tier1c_build_scratch: 22,449 entries, frees 33.54 GB
xargs -d '\n' -a /home/iv/dev/chessai/models/from-scratch/inventory/tier1c_build_scratch.txt rm -rf --
```

**Tier 2: unreferenced old models and data**

```bash
# tier2a_versions_all: 262,908 files, frees 61.24 GB
xargs -d '\n' -a /home/iv/dev/chessai/models/from-scratch/inventory/tier2a_versions_all.txt rm -f --
# tier2a_versions_thin50: 257,784 files, frees 60.15 GB  (ALTERNATIVE to the _all list above)
xargs -d '\n' -a /home/iv/dev/chessai/models/from-scratch/inventory/tier2a_versions_thin50.txt rm -f --
# tier2b_cycles_versions_all: 173,233 files, frees 67.34 GB
xargs -d '\n' -a /home/iv/dev/chessai/models/from-scratch/inventory/tier2b_cycles_versions_all.txt rm -f --
# tier2b_cycles_versions_thin50: 169,815 files, frees 66.01 GB  (ALTERNATIVE to the _all list above)
xargs -d '\n' -a /home/iv/dev/chessai/models/from-scratch/inventory/tier2b_cycles_versions_thin50.txt rm -f --
# tier2c_cycles_states_files: 2,883 files, frees 2.81 GB
xargs -d '\n' -a /home/iv/dev/chessai/models/from-scratch/inventory/tier2c_cycles_states_files.txt rm -f --
# tier2c_cycles_states_dirs: 143 dirs, frees 11.00 GB
xargs -d '\n' -a /home/iv/dev/chessai/models/from-scratch/inventory/tier2c_cycles_states_dirs.txt rm -rf --
# tier2d_champ_run: 324 entries, frees 2.99 GB
xargs -d '\n' -a /home/iv/dev/chessai/models/from-scratch/inventory/tier2d_champ_run.txt rm -rf --
# tier2e_fromscratch_studies: 124,003 entries, frees 120.55 GB
xargs -d '\n' -a /home/iv/dev/chessai/models/from-scratch/inventory/tier2e_fromscratch_studies.txt rm -rf --
# tier2f_games_heldout: 6 dirs, frees 0.39 GB
xargs -d '\n' -a /home/iv/dev/chessai/models/from-scratch/inventory/tier2f_games_heldout.txt rm -rf --
#   /home/iv/dev/chessai/games/heldout
#   /home/iv/dev/chessai/games/heldout-cnbp-d2-s556
#   /home/iv/dev/chessai/games/heldout-d2play
#   /home/iv/dev/chessai/games/heldout-parent-jitter-s600
#   /home/iv/dev/chessai/games/heldout-parent-jitter-s601
#   /home/iv/dev/chessai/games/heldout-t01
# tier2g_exp_pools: 3 dirs, frees 0.13 GB
xargs -d '\n' -a /home/iv/dev/chessai/models/from-scratch/inventory/tier2g_exp_pools.txt rm -rf --
#   /home/iv/dev/chessai/leagues/experiments/diversity-p2/frozenref
#   /home/iv/dev/chessai/leagues/experiments/diversity-p2c/frozenref
#   /home/iv/dev/chessai/leagues/experiments/diversity-p2d/frozenref
# tier2h_selfplay_old: 455 entries, frees 0.47 GB
xargs -d '\n' -a /home/iv/dev/chessai/models/from-scratch/inventory/tier2h_selfplay_old.txt rm -rf --
# tier2i_owncal_superseded: 350 dirs, frees 3.95 GB
xargs -d '\n' -a /home/iv/dev/chessai/models/from-scratch/inventory/tier2i_owncal_superseded.txt rm -rf --
```

**Tier 3: compress or archive, do not delete**

```bash
# tier3a_elo_history_zstd: 100 files, 38.72 GB -> ~3.87 GB (zstd keeps each original until its .zst is written)
xargs -d '\n' -a /home/iv/dev/chessai/models/from-scratch/inventory/tier3a_elo_history_zstd.txt -n1 zstd -q -3 -T4 --rm --
# tier3b_before_backups: 185 files, 0.03 GB
tar --zstd -cf /home/iv/dev/chessai/models/from-scratch/inventory/before-backups-20261112.tar.zst -T /home/iv/dev/chessai/models/from-scratch/inventory/tier3b_before_backups.txt && tar --zstd -tf /home/iv/dev/chessai/models/from-scratch/inventory/before-backups-20261112.tar.zst | wc -l   # expect 185
xargs -d '\n' -a /home/iv/dev/chessai/models/from-scratch/inventory/tier3b_before_backups.txt rm -f --
```

After each tier: `btrfs filesystem du -s` the parent dirs and `df -h /home` to check what it freed.

