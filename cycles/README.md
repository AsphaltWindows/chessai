# Training/tournament cycles

`cycle.py` runs the user's cycle system (designed 2026-11-06, full reset ~15:45). Every lineage (a model kind plus
its training recipe) gets the same compute per cycle while it keeps improving, and a league of its own,
`selfplay/league-cycles`, is the measuring stick. Python 3 standard library only.

The old league, `selfplay/league` (6 round-robins, 100,620 games), is not used or touched: it is the archive of the
old models.

## The field

- **Random players.** These are the league's only players that never train, and its fixed references:
  - `random` plays uniformly random moves at ply 0 and is pinned at Elo 0.
  - `random-1` and `random-2` search 1 and 2 plies on a uniform evaluation, through the same player code as the
    models. So they take a mate in one, and `random-2` also avoids allowing one.
  - Head to head (1000 games each, seed 1): `random-1` scores 73.9% (+181) against `random`, `random-2` 76.1%
    (+202) against `random`, and `random-2` 73.5% (+177) against `random-1`.
  - From cycle 5, `random-3` and `random-4` search 3 and 4 plies the same way. Through cycle 4 only the random
    players stayed from one cycle's field to the next (every lineage entered a new version each cycle), and the
    field scores about 99% against them, so the cycles' fits were linked by a few dozen non-wins per cycle (roughly
    ±80 Elo). The deeper random players score real results against the field and play every cycle, which links the
    fits better:
    - `random-3` scored 19-29% against 1-ply entries rated 790-1040, and 67.5% against `random-2` (60 games each);
      about 0.04-0.08 CPU-s per game.
    - `random-4` scored 40% against F-L-c-v836-1, 5% against F-L-c-v836-2 (1 win in 20) and 72.5% against
      `random-3`; 0.6-1.2 CPU-s per game, its search dominating.
  - **Vs refs** (`rel` in `state.json`, "vs refs" in the reports) stays the 2-ply Elo minus the mean of `random`,
    `random-1` and `random-2` (`state["rel_references"]`), so it means the same before and after cycle 5. All five
    are references (`state["references"]`: players that never train), but `random-3` and `random-4` stay out of
    that mean. The fit itself can still shift at cycle 5, because the new players add evidence.
- **54 lineages** (30 through cycle 5, 31 in cycles 6-9, 36 in cycle 10, 37 in cycle 11, 38 in cycle 12, 39 in cycles
  13-15, 40 in cycles 16-18, 41 in cycles 19-22, 44 in cycles 23 and 24, 45 in cycle 25, 48 in cycles 26-34, 51 in
  cycles 35-37, 52 in cycle 38, 53 in cycle 39), all from scratch with seed 21, so they differ only in their recipe
  (`lineages.txt`). From cycle 15 the freeze/retire rule freezes or retires the ones that stay dormant (see the
  decision step):
  - 16 non-tree lineages:
    - F-L-c, F-L-d0soft-c, F-L-d1jit-c, F-L-d1play-c, F-L-d2play-c, F-L-d2mix-c and F-L-d2jit-c;
    - F-N-c, MoE-c and FM-c;
    - from cycle 11, F-N-d2jit-c (the user, 2026-11-06): F-N-c's recipe plus `--depth 2`, one factor, named like
      F-L-d2jit-c (F-L-c plus `--depth 2`);
    - from cycle 16, MoE-d2jit-c (the user, 2026-11-07): MoE-c's recipe plus `--depth 2`, one factor, the same way.
      MoE-c gains on tournament games and drops on its own depth-0 self-play: cycle 13 took it from v300 at 1744 to
      v336 at 1368 at ply 2, and from 1013 to 858 at ply 1. A ply-2 drop about 2.4 times the ply-1 drop points at
      search exploiting it, so the question is whether self-play by depth-2 search fixes that;
    - TAN-c from cycle 6 (the user: "Let's just throw some basic version of TAN in"): `--model tan --tan-m 200000`,
      a tree-augmented naive Bayes that backs off to CNBP, and otherwise F-N-c's recipe, so TAN-c against F-N-c is
      one factor. Like a tree, it resumes and starts its own step from its training state (`tanN.state`, 8.6 MB,
      which holds the pair counts).
    - from cycle 25, F-P-c (the user, 2026-11-07): `--model probit`, an online Bayesian ordinal probit regression over
      LR's features, trained by assumed-density filtering, and otherwise F-L-c's recipe, so F-P-c against F-L-c is one
      factor. It runs `chessai-probit` (see *The tool*). Its file `probitN.model` holds all its training state, so it
      resumes and starts its own step from it, as LR does. In the probit agent's run of the recipe its v10 matched
      F-L-c's lr10 at depth 2 (2 Elo, -30 to 34) and its v30 beat lr30 by 80 Elo (48 to 112), 400 games each,
      calibrated on the LR held-out set as in the league.
    - from cycle 38, F-P-tau0-c (the user, 2026-11-07, the probit agent's proposal): F-P-c's line plus
      `--probit-tau 0`, one factor: the posterior's dynamics noise tau is 0 instead of 1e-5. Each position adds tau^2
      to the variance of every weight it uses, so at 1e-5 the weights' variance floors at about 2 tau, which caps the
      filter's memory at about one round (about 200k positions), and F-P-c's v1646 already sat at that floor; at tau 0
      the variance keeps shrinking like 1/n, so the model averages over all its data. In the agent's check
      (`models/from-scratch/probit/tau/summary.txt`: from scratch, seed 21, 1000 games a round, each variant at the
      same cumulative CPU as a tau 1e-5 baseline; depth-2 arenas of 2 x 1000 games, calibrated on the LR held-out set)
      tau 0 beat the baseline by 73 Elo at its v100, 48 at v200 and 85 (71 to 99) at v300, and tau 1e-6 by 48, 47 and
      63, while 1e-4 lost by about 420 and 1e-3 by 510-560. The model files store tau, so its own step's
      `train --start` keeps it at 0. The risk: at tau 0 the model keeps stiffening over the league's 1000+ rounds, so
      late own steps may barely move it; tau 1e-6 is the hedge, and the league decides.
    - from cycle 39, F-P-tau0-d2jit-c (the user, 2026-11-07: "You don't need to run compute matched arenas for so much
      of CPU time may as well throw them into the tournament league and see how they do against a more balanced
      field"): F-P-tau0-c's line plus `--depth 2`, one factor: self-play by depth-2 search, the classifier-by-depth
      question for probit, as F-L-d2jit-c and F-N-d2jit-c ask it for LR and NB. On cycle 15's league games, probit
      trained on ply-2 games beat the same model trained on ply-1 games by 174 Elo at ply 2 (LR by 201, while NB's was
      203 below); in pure self-play from scratch at equal CPU (tau 1e-5), probit at depth 2 against depth 1, at ply 2,
      went from 231 below to 53 and 51 below at 450, 900 and 1800 CPU-s, closing fastest of the three classifiers
      (`models/from-scratch/depth-train/`, and `models/IDEAS.md`, "Look-ahead training by classifier"). Tau 0 rather
      than 1e-5 because at depth 2 the model sees about 20-50 times fewer positions a cycle, so tau 1e-5's memory of
      about 200k positions would be under a cycle's data.
  - The 9 BDT recipes, each twice:
    - `<X>-merge-c` uses `--revise 0`, so it merges failed branches.
    - `<X>-c` drops `--revise` but keeps `--reset-leaves inf`. It still trains and logs the shadows, at the same
      cost as its twin, but it never merges and never resets (the models are byte-identical to runs with no
      revise flags at all).
    - So each pair differs in exactly one factor: merging.
  - From cycle 10, 5 more T-N trees, merge versions only (the user, 2026-11-06), each one factor (its routers)
    from an existing T-N merge tree: T-N-Le-merge-c (LR router; vs T-N-Ne-merge-c), T-N-Lc-merge-c (LR router; vs
    T-N-Nc-merge-c), T-N-LLs-merge-c (LR routers; vs T-N-NNs-merge-c), T-N-NLs-merge-c (LR error router; vs
    T-N-NNs-merge-c) and T-N-LNs-merge-c (LR cluster router with an NB error router, a new combination; vs
    T-N-NNs-merge-c).
  - From cycle 12, T-N-NNs-merge-d2jit-c (the user, 2026-11-06): T-N-NNs-merge-c's recipe plus `--depth 2`, with
    `--split-threshold` scaled to its data (the user). A leaf's split check comes every that many training
    positions, and the first check never splits (it compares a window's error with the previous window's). At
    depth 2 the tree trains about 0.85M positions a cycle (240 CPU-s at the measured 85.7 ms a game, about 2,800
    games, at about 302 plies a game from scratch) against T-N-NNs-merge-c's 13.1M (cycles 9 and 10: 55 and 35
    rounds of 1,000 games at about 292 plies a game): 5,000,000 x 0.85 / 13.1 = 323k, rounded to 300,000. At
    5,000,000 it would have stayed one leaf for 5-10 cycles. A check run at 300,000 split at its second check,
    after about 600k positions (56 rounds of 40 games), as soon as the rule allows.
  - From cycle 13, F-L-d4mix-c (the user, 2026-11-06): F-L-d2mix-c's recipe with `--depth 4` instead of 2, its one
    factor: self-play at depth 4, trained on depth-2 look-ahead labels mixed half and half with results. A depth-4
    game costs about 2 CPU-s on the mature models of the LR reference run (2,260 ms measured by the deploy) and about
    4 from scratch, whose opening games run about 350 plies. So its 240 CPU-s buy only about 60-120 games a cycle,
    roughly 15-20k positions, in rounds of 10 games (the round-size rule's floor): the small training set is what the
    user wants to test. Its own step labels its tournament games the same way (see the own-games step).
  - From cycle 19, T-L-Nc-fresh-c (the user, 2026-11-07): T-L-Nc-merge-c's recipe with another merge test, its one
    factor, on `chessai-resid4` (see *The tool*). Its shadow test scores each self-play round's positions by the
    model as it starts the round, before training on them (`--revise-on fresh`), rather than each example as
    training reaches the delegator (the stream), and a branch merges back into its shadow only when it is worse by
    more than 2 standard errors (`--revise 2`). T-L-Nc-merge-c tests on the stream at z 0; T-L-Nc-c never merges.
    The windows are the stream's (`--split-threshold 5000000`, about 15-17 rounds of 1000 games), so until its first
    split it trains as T-L-Nc-merge-c did (byte-identical models over 3 rounds from scratch). Its own step needs its
    binary too (see the own-games step).
  - From cycle 23, T-L-Ne-fresh-c, T-L-Le-fresh-c and T-N-Nc-fresh-c (the user, 2026-11-07): T-L-Ne-merge-c's,
    T-L-Le-merge-c's and T-N-Nc-merge-c's recipes with T-L-Nc-fresh-c's merge test (`--revise 2 --revise-on fresh`) in
    place of the stream's at z 0, the one factor, on `chessai-resid4`. Offline, on league games
    (`models/from-scratch/offline-c15/`, split threshold 100k, held-out log-loss), fresh scored 0.6206 against the
    stream's 0.6199 for T-L-Ne, 0.6220 against 0.6309 for T-L-Le, and 0.7169 (7 leaves) against 0.7328 for T-N-Nc
    (without `--leaf-share`), whose stream test merged its tree back to one leaf; T-N-Le's fresh twin did no better
    (0.7342 against 0.7328) and isn't in. Until its first split each trains as its twin does (byte-identical models
    over 2 rounds from scratch). Their own steps need their binary too (see the own-games step).
  - From cycle 26, T-L-Ne-pre-c and T-L-Le-pre-c (the user, 2026-11-07: "train the Error router from the beginning so
    that when the split happens it's not as random"): T-L-Ne-c's and T-L-Le-c's recipes plus `--pre-split`, the one
    factor, on `chessai-presplit` (see *The tool*). Before each split check a leaf trains its would-be split, the
    children and the router, in the shadow over the window, and a split installs those trained candidates. Offline,
    without merging (the presplit agent), the held-out log-loss went from 0.6242 to 0.6219 for T-L-Ne and from 0.6257
    to 0.6246 for T-L-Le, with far less lopsided splits (no near-empty children). Until a tree's first split each
    plays and trains as its twin (byte-identical models over 2 rounds from scratch, and through v4 at a 300k split
    threshold, where both trees split at v5). While a leaf can still split (`--split-limit`), its training costs about
    twice as much (1.3-1.4 times a game in the checks), and the state files are about 1.5 MB against 0.3. Their own
    steps need their binary too (see the own-games step).
  - From cycle 26, D-L-Nc-c (the user, 2026-11-07), a new archetype, D-L-Nc: the user's DAG, on `chessai-dag` (see
    *The tool*), with `--branch-factor 9 --dag-leaves 3 --dag-window 50000`, LR leaves and a CNBP cluster router
    (`--router cnbp --route cluster`), and otherwise the trees' recipe without `--reset-leaves` (`chessai-dag` refuses
    `inf`). It is one LR leaf until the root's first split check (every 5,000,000 training positions); then the root
    becomes a CNBP cluster router over 9 cells that point to 3 shared LR leaves, copies of the root's (cell c to leaf
    c mod 3), and a cell is pointed to another leaf when that leaf's fresh loss on the cell is lower by 2 standard
    errors over a window of 50,000 judged positions. Its model files are BDT files with repeated children, which the
    tournament plays as trees. Until the first split check it trains as its line without the `--dag-*` options
    (byte-identical models over 2 rounds from scratch; at a 300k threshold through v1, the DAG forming at v2). The DAG
    agent expects a mostly fixed mapping (no re-pointing in 20 self-play rounds), and a game to cost about 1.4 times
    T-L-Nc's once the DAG forms (the 9-way router at every evaluation and each round scored by all 3 leaves). Its own
    step needs its binary too.
  - From cycle 35, T-L-Le-merge-d2jit-c, T-L-NLs-d2jit-c and T-N-Nc-merge-d2jit-c (the user, 2026-11-07):
    T-L-Le-merge-c's, T-L-NLs-c's and T-N-Nc-merge-c's lines plus `--depth 2`, with split thresholds of 100,000,
    100,000 and 200,000 scaled to their data as for T-N-NNs-merge-d2jit-c: 5,000,000 x (positions a cycle at depth 2)
    / (the twin's at depth 0) gave 109k, 138k and 194k (the numbers are in `lineages.txt`), and both T-L trees got
    100,000 so the pair differs only in its recipe. The question is whether look-ahead helps trees with LR leaves and
    not those with CNBP leaves: at ply 2 F-L-d2jit-c peaked at 1818 against F-L-c's 1717, while F-N-d2jit-c (1265
    against 1477) and T-N-NNs-merge-d2jit-c (1175 against 1472) peaked below their twins. The twins: T-L-Le-merge-c,
    the best T-L by peak (1704; retired by the rule after cycle 26, its rating stays in the fit; this note first
    called it a real tree, but it was mostly one leaf, see T-L-NLs-merge-d2jit-c below), T-L-NLs-c, the original T-L
    recipe with the lowest peak (1591, 11 leaves), and T-N-Nc-merge-c, the best active T-N (1453, 7 leaves at v1832).
    Check runs from scratch in rounds of 40 games: the two T-L trees, the same single LR leaf until then, split at
    their second check, at v18 after 204,781 positions, and T-N-Nc-merge-d2jit-c at its third, at v57 after 606,071,
    each within about its first cycle's data (T-N-Nc-merge-d2jit-c's run played 1.28M positions in 122 rounds for 253
    CPU-s, about one cycle's budget); T-L-Le-merge-d2jit-c merged back to one leaf at v28 and T-N-Nc-merge-d2jit-c at
    v76, while T-L-NLs-d2jit-c kept its 3 leaves through v40.
  - From cycle 40, T-L-NLs-merge-d2jit-c (the user, 2026-11-07): T-L-NLs-d2jit-c's line plus `--revise 0` where
    T-L-NLs-merge-c's line has it, at the same split threshold of 100,000, so the depth-2 pair differs only in
    merging. The user reasons that merging beats not merging, which holds in 8 of the 9 tree pairs by peak (T-L-LLs
    about level). At depth 0 T-L-NLs-merge-c reverted 7 of its 8 splits, kept the 8th (from v746) and peaked at 1658
    at v1121 with about 7 leaves, while T-L-NLs-c, which doesn't merge, peaked at 1585 (1591 in the fit used above);
    T-L-NLs-d2jit-c has kept the split it made at v18, and by v211 its model file is 12.3 times the size of its
    single-leaf v17's. So the pair tests whether the merge rule keeps any of its depth-2 splits. A correction to the
    cycle-35 note: T-L-Le-merge-c reverted 25 of its 26 splits, each after about 20 versions, and was one leaf in 71%
    of its versions, and its peak (v2051) came 2 versions into a split; T-L-Nc-merge-c (22 of 22 splits reverted),
    T-L-Ne-merge-c (12 of 12) and T-L-NNs-merge-c (11 of 12) all peaked as a single leaf; and at depth 2
    T-L-Le-merge-d2jit-c had reverted all 5 of its splits and was one leaf at v297. Checked before the deploy, on
    `chessai-cyc5`: from scratch at depth 2 its first 6 versions equal T-L-NLs-d2jit-c's check run's, as nothing
    merges before a split; at a split threshold of 20,000 it split at v3 after 40,937 positions, as T-L-NLs-d2jit-c's
    line does, and merged back at v5, and 4 rounds and `--resume` to 8 gave the same files as 8 rounds in one run; its
    own step through the driver trained and gated 1,000 games at depth 2 (kept); and a round-robin with its entries
    played all 30 games on `chessai-probit`.
  - 2 HT lineages: T-L-H-c and T-N-H-c.
- **Entries.** A lineage's newest version plays at plies 1 and 2, as `<lineage>-v<N>-1` and `<lineage>-v<N>-2`.
- **Size.** From cycle 40, with 54 lineages, 6 of them frozen (one entry each) and 22 retired: 63 players and 3,906
  games per double round-robin, so 19,530 games per cycle (5 round-robins). In cycle 39, with 53 lineages, 6 of them
  frozen and 22 retired: 61 players and 3,660 games per round-robin, 18,300 games. In cycle 38, with 52 lineages, 6 of
  them frozen and 22 retired: 59 players and 3,422 games per round-robin, 17,110 games. In cycles 35-37, with 51
  lineages (6 frozen and 19 retired in cycle 35, 6 and 21 in cycle 37): 63, 59 and 59 players, so 19,530, 17,110 and
  17,110 games a cycle. In cycles 26-34, with 48 lineages (5 frozen and 5 retired in cycle 26, 6 and 19 in cycle 34):
  86, 81, 71, 69, 63, 63, 61, 59 and 57 players, so 36,550, 32,400, 24,850, 23,460, 19,530, 19,530, 18,300, 17,110 and
  15,960 games a cycle. In cycle 25, with 45 lineages, 5 of them frozen (one entry each) and 5 retired: 80 players and
  6,320 games per round-robin, 31,600 games. In cycles 23 and 24, with 44 lineages, 2 of them frozen and 2 retired: 87
  players and 7,482 games per round-robin, 37,410 games. In cycle 22, with 41 lineages, 2 of them frozen: 85 players
  and 7,140 games per round-robin, 35,700 games. In cycles 19-21, with all 41 in: 87 players and 7,482 games per
  round-robin, 37,410 games. In cycles 16-18: 85 players and 7,140 games per round-robin, 35,700 games. In cycles
  13-15: 83 players and 6,806 games per round-robin, 34,030 games. In cycle 12: 81 players and 6,480 games per
  round-robin, 32,400 games. In cycle 11: 79 players and 6,162 games per round-robin, 30,810 games. In cycle 10: 77
  players and 5,852 games per round-robin, 29,260 games. In cycles 6-9: 67 players and 4,422 games per round-robin,
  22,110 games. In cycle 5: 65 players, 20,800 games. Before: 63 players and 3,906 games per round-robin, 39,060 games
  per cycle in cycles 3 and 4 (10 round-robins) and 19,530 in cycles 1 and 2 (5).

## A cycle

1. **Self-play.**
   - Every qualified lineage trains for `budget_cpu_s`: 240 CPU seconds from cycle 3, half a machine-minute on the 8
     cores (480 in cycles 1 and 2; the user halved it and doubled the tournament on 2026-11-06). Each job
     measures its own CPU per game in the loop, under the loop's real load, then plays the rounds that fill the
     budget (see *Budget and round sizes*).
   - It resumes its run exactly (`--resume`, same seed and flags).
   - At most `jobs` (2) run at a time, each with `--threads 4`, at nice 10.
2. **The field.**
   - A lineage with a new version enters it at plies 1 and 2.
   - Its previous entries move from `players.txt` to `retired.txt`. They play no more games, but their games stay in
     `games.csv` and in the fit.
3. **The tournament.** The league tool plays `round_robins_per_cycle` fresh double round-robins of all the active
   players: 5 from cycle 5 (the user cut them back), 10 in cycles 3 and 4, 5 in cycles 1 and 2.
   - `bin/chessai-probit tournament players.txt . --tournaments T --use-probs --anchor random --seed 1 --no-catch-up
     --retired retired.txt --prior-draws 0 --save-games cycles/pools/cycle-NNN`, where T is the round-robins already
     played plus `round_robins_per_cycle`; `bin/chessai-cyc5` through cycle 24 (see *The tool*). Then `league.py`
     writes `league.md`.
   - **No catch-up:** a newcomer plays from its first cycle on, and nobody replays earlier round-robins.
   - **Pure maximum likelihood**, with no prior and no virtual draws, so Elo doesn't inflate when the field and its
     strengths stay the same.
     - `random` is pinned at 0, and a full Newton fit is run.
     - The rated players are those connected to `random` both ways by "scored against" links. Anyone else is listed
       as unrated in `ratings.md`.
     - The ± comes from the information matrix.
4. **Qualification**, by the 2-ply entry, in this tournament's fit. From cycle 9 the **peak rule** decides (the
   user, 2026-11-06):
   - Each lineage's current version is compared with its **peak**: the best vs refs among its earlier versions at
     ply 2, each as rated in the current fit. The report shows it as `peak` and `peak version`, `cycles.csv` as
     `peak_elo` and `peak_version`, on the Elo scale. Retired versions stay in the fit, so the comparison stays
     within one fit, and with the references' mean subtracted from both sides, references added later count
     automatically. The current version is not part of its own peak.
   - The ± is `dormancy_se` standard errors of the difference, from the fit's covariance: 1.5 from cycle 16's
     decisions, 2 through cycle 15 (the user, 2026-11-07). `cycle.py` rebuilds the fit's information matrix from
     `games.csv` at the fit's ratings (the tool's model: Bradley-Terry, draws as half points, `random` pinned at 0);
     that reproduces `ratings.csv`'s ± exactly. If it can't (an unreadable `games.csv`, a matrix that isn't
     positive definite), it combines the two entries' ± and the report says so. The difference's standard error is
     about 30-50, so 1.5 SE is about ±45-75 (2 SE was ±60-100), more than one entry's ±, because it carries the
     shift a whole cycle shares.
   - A qualified lineage more than that below its peak gets a strike (`strike 1 (-X ± R vs peak vN; predecessor
     ±Y)`), and `patience` (2) in a row make it dormant (`dormant (strike 2; ...)`). Otherwise its strikes reset:
     `new peak (+X ± R over vN; ...)` above the peak, `ok (-X ± R vs peak vN; ...)` within range. A lineage with no
     earlier rated version (a new one) qualifies with no strike.
   - A dormant lineage doesn't self-play but keeps its own-games step and gate. From cycle 10 it re-qualifies when
     its current version beats its **peak from before it went dormant** by more than `requalify_se` standard errors
     (1.0 from cycle 16's decisions, 2 through cycle 15; the user, 2026-11-06 and 2026-11-07): the best of its
     versions at ply 2 up to the one that went dormant (`dormant_version` in `state.json`, set when it goes
     dormant), in the same fit, the current version excluded. Versions its own steps make while dormant never
     raise that bar, so small gains add up and it can crawl out: `re-qualified (+X ± R vs peak vN before dormancy
     at vD, at P vs refs)`; dormant rows show the gap (`dormant (+X ± R vs peak vN before dormancy at vD, at P vs
     refs; needs > +R)`). `cycle.py wake NAME` does the same by hand (see below).
   - The report's `peak` and `peak version` stay the all-time peak for every lineage. For a lineage decided while
     dormant (its re-qualified row included) the bar its decision used follows in parentheses, e.g. `1601 (1522)`
     and `v816 (v541)`; `cycles.csv` has it as `pre_dormancy_peak_elo` and `pre_dormancy_peak_version`, blank for
     the rest.
   - The predecessor (`previous`, `prev_elo`) is information only.
   - **History.**
     - Cycles 1-8 struck a lineage whose new version rated below its predecessor in the same fit. Across cycles
       that difference carries the shift a whole cycle shares (one lineage against its predecessor ±108 on cycle
       4's fit), so a shift of the field struck many lineages at once: in cycle 5 (mean change -25) 9 went dormant
       and 13 got a strike, while a same-field replay (all 30 cycle-4 versions against cycle 5's field) found a
       mean change of -2, with the sign agreeing for only 16 of 30. Replayed on cycle 6's fit (rebuilt from
       `games.csv`), the peak rule would have kept qualified the 4 lineages that went dormant there, 7-29 below
       their peaks; on cycle 7's, its 12 strikes become 2.
     - At the switch, after cycle 8, `cycle.py migrate-strikes` recomputed the qualified lineages' strikes from
       cycle 8's fit under the peak rule (1 if more than 2 SE below the peak, else 0; 9 changed, the table
       is in `deploy.log`). The dormant lineages stayed dormant (the user: "Let them stay dormant"), to re-qualify
       by the rule.
     - Re-qualification compared with the lineage's low point when it went dormant (`dormant_rel`, still recorded)
       through cycle 4: a flat lineage's ratings regress upwards from that low, so it re-qualified by chance (about
       19% the next cycle and 51% within 5, in the coordinator's simulation). Cycle 5 compared with the peak, by
       `requalify_margin` (1) times the current entry's ±; cycles 6-8 by 1.96 standard errors of the difference;
       cycle 9 by 2 standard errors from the all-time peak, which the versions made while dormant could raise.
     - At the switch to the pre-dormancy bar, after cycle 9, `cycle.py migrate-dormant` applied it retroactively
       (the user): each dormant lineage got its `dormant_version` from `cycles.csv` (the version of its latest turn
       to dormant), and every version current in a cycle since then was checked against that bar in cycle 9's fit
       (the newest, with the most games). 2 cleared it and re-qualified for cycle 10, training from their current
       versions: F-L-d0soft-c (v1619, +119 ± 68 vs peak v1457; dormant since cycle 6 at v1617) and T-L-NNs-merge-c
       (v818, +90 ± 71 vs peak v815; dormant since cycle 5 at v815). Per-cycle fits, rebuilt from `games.csv` as a
       check, wake the same 2 and no other; there T-L-NNs-merge-c would have woken in cycle 6 and F-L-d0soft-c in
       cycle 8. The table is in `deploy.log`.
     - After cycle 15 the old rule's effects were undone (the user, 2026-11-07), by `cycle.py wake` in the deploy's
       `migrate` file (the commands' output is in `deploy.log`). Every lineage whose dormancy began at one of its
       decisions woke: qualified, strikes 0, no `dormant_version`, self-playing from cycle 16. Those are the 4 the
       freeze/retire rule took at cycle 15's decision, all dormant since cycle 5, first unretired (F-L-d2mix-c,
       T-N-Nc-c and T-N-Ne-merge-c) or unfrozen (T-N-Nc-merge-c, frozen at ply 1 at 800), so their 7 entries came
       back to the field and 800 is free again; and F-L-d2play-c and T-L-Ne-merge-c (dormant since cycle 6) and
       T-L-H-c, T-L-NLs-merge-c, T-L-Ne-c and T-N-NNs-c (cycle 8). The 12 lineages that went dormant under the peak
       rule (cycles 11-14) stay dormant.
     - The same deploy lowered `dormancy_se` from 2 to 1.5 and `requalify_se` from 2 to 1.0 (the user), for cycle
       16's decisions on. Earlier strikes and wakes were not recomputed.
   - **Freezing and retiring.** From cycle 15's decision an automatic rule freezes or retires the lineages that
     stay dormant (the user, 2026-11-06; it replaced the plan of a hand-picked ladder of fixed references). Four
     `cycle.py` commands do it by hand, and a fifth, `wake`, ends a dormancy; all run between cycles only (touch
     `STOP`, wait for the loop to stop, run the command, restart the loop as below; or put the commands in a staged
     deploy's `migrate` file, one a line).
     - **The rule** (`config.json`'s `freeze_rule`), at each decision, after the lineages' decisions. A lineage
       dormant 10 cycles in a row (`after_dormant`), counted from the decision that made it dormant as the
       dashboard counts them ("dormant (N)"; waking resets the count), is settled at the decision of cycle went+10:
       - Its average per ply: the mean, over those 10 cycles, of its entry's Elo at that ply at the end of each
         cycle's tournament (`elo_history.csv` at the cycle's last round-robin), whatever version was current then.
         At ply 2 that is the mean of `cycles.csv`'s `elo`.
       - Milestones: the multiples of 100 (`milestone_step`). One is taken by a frozen lineage's claim (`claim` in
         `state.json`, kept however its rating drifts later) or by a reference player (`random` to `random-4`)
         nearest to it within 50 (`reference_window`) in the newest fit: now 0, 200, 300 and 400.
       - A ply is eligible if its average is within 10 (`milestone_window`, inclusive) of its nearest milestone and
         that milestone is untaken. The lineage freezes at the eligible ply closer to its milestone (a tie: ply 2)
         and claims it; its other entry retires, as `freeze` does. With neither ply eligible it retires: both entries
         go to `retired.txt`, its status becomes `retired` (`retired_by` "rule"), its line stays in `lineages.txt`
         and the driver skips it.
       - Lineages due at the same decision settle closest first. One that loses its milestone falls back to its
         other ply if that is eligible and untaken, and retires otherwise.
       - The entries leave the field in the own phase, before the next cycle's round-robins. `cycles.csv` logs the
         new status, and the decision starts with the outcome: "frozen at ply 1 (avg 799.4 over cycles 6–15;
         milestone 800); dormant (...)" or "retired (avg ply 1 837.0, ply 2 1782.1; no untaken milestone within
         10); dormant (...)". The cycle's summary adds the change in field size and games per cycle. `state.json`
         keeps the lineage's `rule` (cycles, averages, outcome, the cycle it went dormant), a `rule_preview` on each
         dormant lineage and the `milestones` taken, with their takers.
       - Preview: `cycle.py status` and every `cycle-NNN.md` list each dormant lineage's count (N of 10), its
         running averages per ply and what the rule would do now (lineages due earlier claim first), and the
         milestones taken.
       - Exemptions: `exempt`, empty. No exemption for F-L-d2mix-c (the user: "i would have liked to freeze it,
         but if it can't maintain the 1800 level it's not worth it").
     - `freeze NAME PLY`: the lineage keeps ONE entry in the field, its current version's at PLY (1 or 2), as a
       fixed reference; its other entry moves to the league's `retired.txt`, so its games stay in the fit (never
       both plies). Status `frozen`, with `frozen_from` (the status before) and `frozen_retired` in `state.json`;
       its strikes and `dormant_version` stay as they were. No self-play, own step, gate or decision. `cycles.csv`
       still logs it every cycle: status and decision `frozen`, its entry's Elo (at that entry's ply, which the
       `entries` column gives), the all-time peak, no pre-dormancy columns.
     - `unfreeze NAME`: the retired entry comes back to the field and the lineage to the status it was frozen from
       (a dormant one faces its pre-dormancy bar again). A claim by the rule is released.
     - `retire NAME`, for a lineage that fills no slot, after taking its line out of `lineages.txt` (comment it out
       to keep the recipe): its entries move to `retired.txt`, its games stay in the fit, and `state.json` keeps it
       with status `retired`, `retired_cycle` and `retired_from`. The driver never runs it again.
     - `unretire NAME`: a retired lineage's entries come back from `retired.txt` to the field, and it gets back the
       status it was retired from. Its line must be in `lineages.txt` (the rule leaves it there; put back one taken
       out for `retire`). The rule doesn't fire again in the same dormant spell.
     - `wake NAME` (from cycle 16, the user): a dormant lineage becomes qualified, as re-qualifying makes it:
       strikes 0, no `dormant_version` (so no pre-dormancy bar) and no `dormant_rel`; it self-plays from the next
       cycle. A frozen or retired lineage needs `unfreeze` or `unretire` first. `state.json` keeps the dormancy as
       `woken` (the cycle after which it woke, its strikes and `dormant_version`), and a rule record moves to
       `rule_undone`, so the rule fires again only in a later dormant spell. After cycle 15 it undid the old rule
       (see History).
5. **The own-games step**, for every lineage whose `steps` field is `own` (all of them), dormant ones included.
   - The lineage trains once on the games its entries played in this tournament against lineage entries, its own
     other entry included: `train SUB OUT --start <newest model> --rounds 1 --seed 21 <model options>`. Trees and
     TAN-c start from their `.state` file.
   - **Labels** (from cycle 13, the user): the model options include the recipe's labels (`--label-depth`,
     `--label-result`, `--label-model`), so F-L-d2mix-c and F-L-d4mix-c train their own games on depth-2 look-ahead
     labels mixed half and half with results, searched by the model the step starts from and calibrated on these
     games, as their self-play labels its games. Through cycle 12 the driver dropped those options (train took none)
     and every own step trained on results. Offline on cycles 5-9, F-L-d2mix-c's own steps with its labels beat those
     on results by 35 ± 12 Elo, and 3 of the 4 gates of cycles 5-8 would have passed. The label search costs about
     12-15 CPU-s a cycle for each of the two, outside the budget (the user). A lineage with labels skips the step,
     logged, rather than train without them if the binary can't label. The step's train and gate run
     `bin/chessai-cyc5-labels` (see *The tool*); every other lineage's step is byte for byte what it was.
   - **A lineage's own binary** (from cycle 19): `own_games.binary_by_lineage` in `config.json` names the binary of a
     lineage's own step, its train and its gate; the others run `own_games.binary`. T-L-Nc-fresh-c's runs
     `chessai-resid4`, and from cycle 23 so do T-L-Ne-fresh-c's, T-L-Le-fresh-c's and T-N-Nc-fresh-c's: their trees'
     states record `--revise-on fresh`, which `chessai-cyc5-labels` refuses as an unknown flag, and `chessai-resid4`
     refuses a state without it. Their train reads the step's round of linked game directories and scores the round
     fresh before training on it, as self-play does. From cycle 25 F-P-c's runs `chessai-probit`, the one binary with
     `--model probit`, and from cycle 38 so does F-P-tau0-c's and from cycle 39 F-P-tau0-d2jit-c's. From cycle 26
     T-L-Ne-pre-c's and T-L-Le-pre-c's run `chessai-presplit` and D-L-Nc-c's `chessai-dag`: their trees' states record
     `--pre-split` and the DAG's cells, and `chessai-cyc5-labels` has neither option.
   - Games against the random players don't count: they only measure.
   - That's 226 games per round-robin from cycle 40 (218 in cycle 39, 210 in cycle 38, 226, 210 and 210 in cycles
     35-37, 318, 298, 258, 250, 226, 226, 218, 210 and 202 in cycles 26-34, 294 in cycle 25, 322 in cycles 23 and 24
     and in cycles 19-21, 314 in cycle 22 and in cycles 16-18, 306 in cycles 13-15, 298 in cycle 12, 290 in cycle 11,
     282 in cycle 10, 242 in cycles 6-9, 234 before): about 1,130 per cycle per lineage at 5 round-robins (1,090 in
     cycle 39; 1,050 in cycle 38; 1,130, 1,050 and 1,050 in cycles 35-37; 1,590, 1,490, 1,290, 1,250, 1,130, 1,130,
     1,090, 1,050 and 1,010 in cycles 26-34; 1,470 in cycle 25; 1,610 in cycle 23, 1,550 in cycle 24; 1,570 in cycles
     16-18, 21 and 22; 1,530 in cycles 13 and 14, 1,510 in cycle 15; 1,490 in cycle 12; 1,450 in cycle 11; 1,410 in
     cycle 10; 1,210 in cycles 6-9; 1,170 in cycles 1, 2 and 5; 2,340 in cycles 3 and 4). In cycles 15, 21 and 24 the
     own steps were 20 games short for each entry the rule's freezes took out of the field at that cycle's decision:
     the step took its opponents from the lineages' entries after the decision, and a frozen lineage keeps only one.
     That was unintended (the user: every lineage entry that played counts), and from cycle 25 the step's opponents
     include the entries a freeze has just taken out (`frozen_retired` in `state.json`). Replaying cycle 21's own
     steps of F-L-c and F-N-c with the fix changed only the game set: 1,610 games instead of 1,570, the 40 against the
     two frozen-out entries, with the same commands; the old code gave cycle 21's games, positions, gates and models
     byte for byte.
   - **Gate:** the result plays the model it started from in `arena`.
     - 1000 games, both sides at depth 2, calibrated like the league entries, seed 1 + cycle.
     - The result becomes the next version only if it scores at least 50%. Otherwise it is moved to
       `runs/<lineage>/reverted/`.
   - Each cycle's summary opens with the gates pooled over all lineages, for that cycle and for all cycles so far.
     This is the step's one-factor measurement.

**Cycle 0** (`init`) makes the new league with the 3 random players and records every lineage as new (qualified on
entry). There is no own-games step in cycle 0 (`own_games.first_pool` is empty): cycle 1 starts with self-play.

## Running it

    cd /home/iv/dev/chessai/cycles
    python3 cycle.py selftest
    python3 cycle.py status
    python3 cycle.py init                                   # once: the league, cycle 0
    setsid nohup ./loop.sh >> loop.log 2>&1 < /dev/null &   # cycles until stopped

- **The loop.** `loop.sh` runs `cycle.py run --cycles 1` again and again. It stops when a file named `STOP` appears
  in this directory or when a cycle fails.
  - The first loop started on 2026-11-06 at 16:15 (pid 8204). It was stopped after cycle 1 to swap in the driver
    that measures the budget in the loop.
  - The second loop started on 2026-11-06 at 17:25:53 (pid 31566), 9 s after the first stopped. It was stopped
    after cycle 2 to swap in the cycle-3 config (budget 240, 10 round-robins).
  - The third loop started on 2026-11-06 at 18:24:12 (pid 46938), 14 s after the second stopped. It was stopped
    after cycle 4 to deploy cycle 5's changes from `staged-c5/`: `chessai-cyc5`, 5 round-robins, `random-3` and
    `random-4`, and the peak rule for re-qualification.
  - The fourth loop started on 2026-11-06 at 19:33:45 (pid 66586), 19 s after the third stopped (19:33:26). It was
    stopped after cycle 5 to deploy cycle 6's changes from `staged-c6/`: re-qualification by the 95% range of the
    difference with the peak, and TAN-c.
  - The fifth loop started on 2026-11-06 at 20:09:01 (pid 80066), 21 s after the fourth stopped (20:08:40). It was
    stopped after cycle 8 to deploy cycle 9's changes from `staged-c8/`: the peak rule for strikes, the peak
    columns and the strike migration.
  - The sixth loop started on 2026-11-06 at 21:31:20 (pid 95816), 22 s after the fifth stopped (21:30:58). It was
    stopped after cycle 9 to deploy cycle 10's changes from `staged-c9/`: the 5 new T-N merge trees, and the
    pre-dormancy bar for re-qualifying with its retroactive migration.
  - The seventh loop started on 2026-11-06 at 21:52:39 (pid 101823), 19 s after the sixth stopped (21:52:20). It
    was stopped after cycle 10 to deploy cycle 11's change from `staged-c10/`: F-N-d2jit-c.
  - The eighth loop started on 2026-11-06 at 22:20:31 (pid 122030), 18 s after the seventh stopped (22:20:13). It was
    stopped after cycle 11 to deploy cycle 12's change from `staged-c11/`: T-N-NNs-merge-d2jit-c.
  - The ninth loop started on 2026-11-06 at 22:51:41 (pid 137984), 24 s after the eighth stopped (22:51:17). It was
    stopped after cycle 12 to deploy cycle 13's changes from `staged-c12/`: the own step's labels and F-L-d4mix-c.
  - The tenth loop started on 2026-11-06 at 23:22:40 (pid 148040), 35 s after the ninth stopped (23:22:05). It was
    stopped after cycle 14 to deploy cycle 15's change from `staged-c14/`: the automatic freeze/retire rule.
  - The eleventh loop started on 2026-11-07 at 00:25:16 (pid 167912), 16 s after the tenth stopped (00:25:00). It
    was stopped after cycle 15 to deploy cycle 16's changes from `staged-c15/`: self-play in re-planned chunks,
    MoE-d2jit-c, the undo of the old dormancy rule and the new thresholds (`dormancy_se` 1.5, `requalify_se` 1.0).
  - The twelfth loop started on 2026-11-07 at 01:02:27 (pid 190999), 8 min 50 s after the eleventh stopped
    (00:53:37): the deploy waited for the undo (the user). It was stopped after cycle 18 to deploy cycle 19's change
    from `staged-c18/`: T-L-Nc-fresh-c.
  - The thirteenth loop started on 2026-11-07 at 02:36:27 (pid 212993), 24 s after the twelfth stopped (02:36:03). It
    was stopped after cycle 22 to deploy cycle 23's change from `staged-c22/`: T-L-Ne-fresh-c, T-L-Le-fresh-c and
    T-N-Nc-fresh-c.
  - The fourteenth loop started on 2026-11-07 at 04:09:03 (pid 231296), 21 s after the thirteenth stopped (04:08:42).
    It was stopped after cycle 24 to deploy cycle 25's changes from `staged-c24/`: F-P-c, the tournament on
    `chessai-probit` and the own step's opponents fix.
  - The fifteenth loop started on 2026-11-07 at 05:08:25 (pid 263025), 4 min 4 s after the fourteenth stopped
    (05:04:21): the first deploy run took a watcher's shell for a running driver and stopped without deploying, and a
    re-run deployed once the watcher was stopped. It was stopped after cycle 25 to deploy cycle 26's changes from
    `staged-c25/`: T-L-Ne-pre-c, T-L-Le-pre-c and D-L-Nc-c.
  - The sixteenth loop started on 2026-11-07 at 05:33:42 (pid 268851), 25 s after the fifteenth stopped (05:33:17). It
    was stopped after cycle 34 to deploy cycle 35's change from `staged-c34/`: T-L-Le-merge-d2jit-c, T-L-NLs-d2jit-c
    and T-N-Nc-merge-d2jit-c.
  - The seventeenth loop started on 2026-11-07 at 09:23:33 (pid 297069), 40 s after the sixteenth stopped (09:22:53).
    It was stopped after cycle 37 to deploy cycle 38's change from `staged-c37/`: F-P-tau0-c.
  - The eighteenth loop started on 2026-11-07 at 10:49:39 (pid 314615), 9 s after the seventeenth stopped (10:49:30).
    It was stopped after cycle 38 to deploy cycle 39's change from `staged-c38/`: F-P-tau0-d2jit-c.
  - The nineteenth loop started on 2026-11-07 at 11:14:54 (pid 319038), 14 s after the eighteenth stopped (11:14:40).
    It was stopped after cycle 39 to deploy cycle 40's change from `staged-c39/`: T-L-NLs-merge-d2jit-c.
  - The current loop started on 2026-11-07 at 11:40:58 (pid 324307), 23 s after the nineteenth stopped (11:40:35).
- **Watching.** Use `tail -f loop.log` and `python3 cycle.py status`, which is read-only and safe while the loop
  runs. Each cycle writes its summary to `cycle-NNN.md`.
- **Dashboard.** `dashboard.py` serves a live page at http://127.0.0.1:8765 (this machine only), refreshed every
  30 s. Each panel has an anchor: `#now`, `#chart`, `#standings`, `#compare`, `#lineages`, `#head-to-head`, `#elo`,
  `#gates`, `#history`; a jump lands just below the bar at the top.
  - **Now:** the phase, each self-play job against its budget, the tournament's games and its log's last line
    (ratings without "+"), the own-games steps and gates, and the cycle's notes (the freeze/retire rule's preview as
    a table). A job between two of its commands (its measuring segment, then its chunks) still shows as running, and
    its CPU so far includes `cut_cpu`;
  - **The bar at the top** stays in view as the page scrolls. "Filters" opens or closes the factor chips (open at
    first; closed, the bar is one line); beside it the filters in force, each with ✕, how many of the lineages they
    show (e.g. "17 of 54 lineages") and "clear all"; at the right, colour and group by with its key. While any
    lineage is highlighted, a second line lists them, each with ✕, with "show in the chart" and "clear the
    highlight";
  - **Elo by cycle** (the user, 2026-11-07), under Now: one line per lineage the factor filters show, coloured by
    the colour factor, at 2-ply (1-ply on its toggle; one league play depth ticked in the bar sets it), from the same
    per-cycle points as Elo over cycles. A frozen lineage's line is dotted from the cycle it froze at the ply of the
    entry it kept; × ends a retired lineage's line at its last cycle, and a frozen one's at the ply whose entry it
    retired at the freeze. A ring marks each line's peak (Standings' Peak Elo; "Peaks" hides them); the random
    players can be reference lines. Under the chart, a key and the lineages grouped by the colour factor, each with
    its latest Elo, and a table view.
    - Highlight: click a line, its name in the list, or the lineage anywhere on the page (a Standings, lineages-table
      or gate row, a factor-comparison cell, a small multiple); again to un-highlight. Highlighted lines are drawn
      thick with their points, 95% whiskers and end labels, the others faded; their rows elsewhere are tinted in
      their colour. A small multiple's series or name also scrolls to the chart, and its title highlights all its
      lineages the filters show.
    - Hovering, or the arrow keys once the chart has focus (up and down change lines, Enter highlights), give a
      point's lineage, Elo ±, cycle, version and ply, the cycle's decision, and the lineage's status and peak.
    - It replaced the pick-any comparison chart above the small multiples (its picker and colour-each-lineage are
      gone; the filters and the highlight do that work now);
  - **Factors:** each lineage has one value for each of seven factors, read from its `lineages.txt` line's flags, with
    the binary's defaults where a flag is absent (a retired lineage without its line, from its name; "unclassified"
    where neither settles it, never a guess):
    - Architecture: F flat (`--model lr`, `cnbp`, `probit`, `tan` or `fm`), T tree (`--model bdt`, or `ht`), D DAG
      (`--dag-leaves`), M MoE (`--model moe`).
    - Leaf evaluator: L, N, P, TAN or FM: a flat model's `--model`, a tree's or DAG's `--leaf` (cnbp by default); an
      MoE's experts are L.
    - Router, as in the names: `--router`'s letter (cnbp by default), then c or e (`--route cluster` or `error`, error
      by default); with `--route-switch`, the error router's letter (`--error-router`, else the router's) and s: Nc,
      Ne, Lc, Le, NNs, NLs, LNs, LLs. An HT's is H, an MoE's "gate (soft)", a flat model's none.
    - Split management: merge (`--revise`), fresh (`--revise-on fresh`), pre-split (`--pre-split`), else none.
    - Self-play depth: `--depth` (0 by default).
    - Training labels: result, or e.g. "mix: d2 search 50%" (`--label-depth 2 --label-result 0.5`).
    - Self-play move selection: jitter10 (the default), or e.g. "softmax 0.3" (`--policy log-odds-softmax
      --temperature 0.3`).
    - Hyperparameters are not factors: the Lineages panel's variant column gives those that differ from the field's
      usual recipe (split threshold 5M, split limit 5, forget factor 1, branch factor 3, leaf share 0, 2 experts, TAN
      m 5, τ 1e-5, seed 21; a DAG's leaves and window always), e.g. "split threshold 300k", "τ 0", "4 experts".
    - Filters, in the bar at the top: a row of chips per factor. Values of one factor add up (or), factors narrow
      down (and); each chip counts the lineages it would show given the other factors' ticks. League play depth
      (ply 1, ply 2) is per entry, not per lineage: it filters the Standings rows and the head-to-head entries. The
      filters apply to every panel; the random players always stay, and pooled rows still pool every lineage.
    - Colour and group by, in the same bar: any factor (leaf evaluator at first). Every lineage's marks take its
      value's colour: a fixed slot per value in the factor's order; grey past the eighth, for unclassified, and for
      router H, gate (soft) or none and split none. The gate rows and the chart's list of lineages are grouped by
      it (the Standings are not: one table, the user, 2026-11-07). Marker shapes stay: a ring for a merging tree
      (merge or fresh), a diamond for an HT, a triangle for a DAG, a square for a random player;
  - **Standings:** the current fit (`ratings.csv`), with the ±, one table ranked by Elo across every lineage (no
    sections; the user, 2026-11-07). Absolute ratings carry no "+"; only recent changes do (the change column, and
    each cycle's gate results in Gate history). Clicking a row highlights its lineage in the chart at the top.
    - Change: over the cycle, per lineage and ply: the row's Elo now minus the lineage's rating at that ply at the
      end of the previous cycle's tournament, whatever version it was then, i.e. the last segment of its line on Elo
      over cycles (`cycles.csv` at ply 2, the league's `elo_history.csv` at ply 1, the same full fit). A version
      carried over shows its own drift; a new one, its change from the version it replaced; a random player, its own
      drift. E.g. "▲ +58" or "▼ −9"; "new" only for a lineage's first cycle at that ply. The previous cycle is the one
      before the cycle whose round-robins made today's fit: while cycle N's round-robins run, the change so far in N;
      during N's self-play, N−1's whole change. Hovering it gives both ratings, e.g. "v1754: 1798.6 now; the lineage
      at the end of cycle 11: v1651 at 1837.3 → ▼ −38.7", "k of 5 round-robins into cycle N" while a tournament is
      partial, and the version's story ("v750: the same model since cycle 5", or "v1754 replaced v1651 in cycle 12"
      and how they compare in today's fit).
    - Filter by status: a checkbox per status (qualified, dormant and reference; frozen and retired once a row has
      them), each with its count of lineages (random players for reference). None is ticked at first, and every row
      shows; ticking statuses shows only their rows, and unticking them all shows every row again. The ranks stay as
      they are. The factor filters hide the rows of the lineages, and the play depths, they leave out; a note at the
      right counts what shows, e.g. "showing 17 of 54 lineages, 39 of 113 rows".
    - Factor columns, at the right end after the interval bars: each row's lineage's value of each factor (arch,
      leaf, router, split, self-play depth, labels, move selection), the letter alone where a value has a name
      (hover for it, e.g. "T (tree)"), the factor the page colours by in bold; empty for the random players.
    - Status: a dormant lineage shows the cycles it has spent dormant, counting the one under way, since its latest
      going-dormant decision (`cycles.csv`'s "dormant (strike 2; ..." or, in older rows, "dormant (-43, strike 2;
      ..."), e.g. "dormant (7)"; hovering it gives "dormant since cycle 5 (at v750)" and the wake gap.
    - The freeze/retire rule on dormant rows: the status hover adds the countdown ("dormant 9 of 10: the
      freeze/retire rule applies after cycle 15"), each ply's average of its end-of-cycle ratings over its dormant
      cycles so far, and what the rule would do if it applied now. These come from `state.json`'s `rule_preview` once
      `cycle.py` writes it; until then the dashboard computes them the same way (each cycle's rating as measured then,
      the dormant lineages resolved jointly, earlier deadline first, then the closest).
    - Frozen and retired: a frozen lineage shows "frozen", or "frozen (800)" when the rule froze it at the milestone
      it claimed (hover: its averages and the cycles averaged). A retired lineage's last entries, and a frozen
      lineage's entry retired at the freeze, keep their rows, muted and ranked "–" so the active ranks don't shift:
      "retired (cN)" or "retired (frozen)", with their own drift as the change (0, never "new", when the entry has no
      rating of its own at the end of the previous cycle; the user, 2026-11-07) and no "new peak"; the hover gives the
      rule's averages, or "retired by hand". The rule's entries count as out from its decision on (`state.json`'s
      `current.rule_retire`), though the own phase moves them to `retired.txt` a few minutes later; a lineage it
      retired shows "retired" in that cycle's gate column, since it takes no own-games step.
    - Under the table, the milestones taken: each multiple of 100 held by a reference player (its nearest, within
      50) or claimed by a frozen lineage, from `state.json`'s `milestones` once `cycle.py` writes them; a claim
      freed since by an unfreeze is left out.
    - Peak Elo and Peak version (the user, 2026-11-07) give the lineage's highest rating at the row's ply as
      measured at the completion of a cycle, and the version that played in that cycle: at the qualifying ply
      `cycles.csv`'s `elo` (at the decision), at the other the league's `elo_history.csv` at that cycle's last
      round-robin (`cycle.py`'s `round_robin_end`). The cycle under way counts once decided (`state.json`'s
      rows), never the running fit, so mid-tournament the row's Elo can be above its peak (the chart keeps its live
      point). E.g. F-L-d2mix-c's 1807 is v750 as measured in cycle 7. Hovering a peak gives the cycle it was
      measured in, that version's rating in today's fit and the rule's comparison, the best earlier version in
      today's fit (what `cycles.csv` logs as `peak_version` and `peak_elo`). A "new peak" tag marks a peak set by
      the latest completed cycle (not a lineage's first cycle at that ply, nor a retired row's).
    - Ply gain: each model's ply-2 entry's Elo minus its ply-1 entry's, on the ply-2 row, ± the 95% range from the
      fit's covariance (`cycle.py`'s `fit_factor` and `contrast_se`, computed in the background for each new fit,
      a few seconds of CPU; until then the previous fit's, or right after a restart the two ±s combined as if
      independent, and the hover says which). Clicking its
      header sorts by it (again to reverse, Elo to go back); the order is kept in the browser.
    - Dormant rows add, in parentheses, the same peak and its version over the cycles up to the one whose
      decision made the lineage dormant (its latest going-dormant decision), at either ply, e.g. TAN-c (dormant
      from cycle 14's decision) "1287 (1287)" and "v175 (v175)". The rule's wake bar is in the status and peak
      hovers, labelled as the rule's: the lineage's best version up to `state.json`'s `dormant_version`, the
      current one excluded, in today's fit, which it must beat by more than `requalify_se` standard errors
      (`config.json`) to wake, with the gap in words as `cycle.py` computes it. The texts quoting the rule's
      standard errors (a strike, waking) take `dormancy_se` and `requalify_se` from `config.json`.
    - When `cycles.csv`, or the cycle under way in `state.json`, has `peak_elo` and `peak_version` (and
      `pre_dormancy_peak_elo` and `pre_dormancy_peak_version` on dormant rows) from the same fit, the panel checks
      them against the rule's comparison it computes (the best earlier version and the rule's wake bar, in the
      peak hover) and marks any difference with ≠ on the Peak version.
    - On a narrow window the table scrolls sideways inside its panel;
  - **Factor comparison:** pick a factor. Each row is a set of lineages that agree on every other factor, a column per
    value; comparing architectures also lets the router and the split management vary, as a flat model has neither.
    - A cell gives each lineage's peak Elo and Elo now (as in Standings) at the ply picked, its variant and, unless
      qualified, its status; past four lineages it folds.
    - The difference column sets each value against the reference (the factor's default, or the one picked), by peak
      Elo or Elo now: for the pair from the two cells with the fewest other differences, then the best of each, e.g.
      "107 above ± 78" or "221 below ± 66" (± the two ±s combined as if independent), with "also differs: split
      threshold 300k vs 5M" where the pair differs otherwise. A row without the reference sets its values against its
      first one.
    - A summary gives, per value, the rows where it is above or below the reference and the median difference;
  - **Lineages:** one row per lineage (`lineages.txt`'s, then the retired ones without a line): its archetype, its
    value for each factor, its variant, status, peak Elo and Elo now. Hovering a name gives its line; clicking a
    heading sorts by it. A count above the table gives the lineages the factor filters show, e.g. "Showing 17 of 54
    lineages (Leaf evaluator: N (NB))";
  - **Head to head:** +wins −losses =draws for every pair in the current field, in Standings order, shaded by score.
    Hovering a cell gives the games, the score, and the score the current fit expects. A switch counts by
    lineage@ply over all cycles instead (random players included).
    - A cycle selector shows one cycle's tournament, newest cycle first. The table holds that tournament's players
      (that cycle's versions at plies 1 and 2, and the reference players then active), ordered by their ratings in
      the current fit, and counts that cycle's games only.
    - While a cycle's tournament is running, the selector shows its games so far, marked partial.
    - Round-robins are matched to cycles by the counts in the `cycle-NNN.md` summaries.
    - Clicking a row gives that player's record per opponent and cycle.
    - It is built from `games.csv`, read incrementally;
  - **Elo over cycles, by archetype:** each lineage's 2-ply Elo in small multiples: the F-L variants, the other flat
    models and the MoE, one per tree or DAG archetype (its non-merge, merge, fresh, pre-split and depth-2 lineages
    together), the HTs together, each titled by what varies in it. A dashed vertical line marks where reference
    players joined the field. Clicking a series or a name highlights the lineage in the chart at the top and scrolls
    there; clicking a panel's title highlights all of its lineages the filters show.
    - Each lineage has a point for every cycle it played, rated as measured then: a finished cycle's from its
      decision (`cycles.csv`'s elo and ci95 at 2-ply; at 1-ply the league's `elo_history.csv` at the cycle's last
      round-robin, which keeps no ±), the cycle under way's live from `ratings.csv` once its fit has a round-robin of
      that cycle. A version that keeps playing (a dormant lineage's) gets a smaller point each later cycle, so its
      line moves as the league re-rates it. Hovering a point gives its version and that version's rating in today's
      fit.
    - In the small multiples, a dashed line is a lineage that self-plays at depth 2 (`--depth 2` in `lineages.txt`),
      e.g. F-N-d2jit-c next to F-N-c;
  - **Gate history:** each own-games step's gate, by lineage and cycle, the lineages grouped by their value of the
    factor the page groups by (the retired ones last).
    - Each cell gives the score and Elo ± 95% against the model before the step, marked ✓ kept or ↺ reverted.
      Hovering it gives +wins −losses =draws, the own games and positions trained on, and the CPU.
    - A pooled row per cycle repeats that cycle's summary line, and a chart plots it.
    - The last two columns give each lineage's kept steps out of its gates, and the sum of the kept steps' Elo.
      That sum is biased up, since each step was kept because it scored at least 50%.
    - While a cycle's own-games steps are running, the panel shows its gates so far, from `state.json`;
  - **Cycle history:** each cycle's times, the CPU per lineage against that cycle's budget, and the disk use; a frozen
    and a retired count once a cycle has one. Retired lineages, by hand or by the rule (whose line stays in
    `lineages.txt`), keep their history in Elo over cycles and the gates table (its Retired group).

  The factor filters (and whether the bar shows them), colour and grouping, the highlight, the chart's ply, peaks
  and reference lines, the comparison's choices, the lineages table's sort, the Standings sort and status filter, the
  head-to-head scope (or cycle) and the open player are kept in the browser (localStorage), so the refresh doesn't
  reset them. The page only reads (no locks, no writes), parses a file only when it changes, and runs at nice 19. It
  runs as pid 353738; stop it with `kill 353738` and start it with:

      setsid nohup nice -n 19 python3 dashboard.py >> dashboard.log 2>&1 < /dev/null &
- **Stopping cleanly:** `touch STOP`. The loop finishes the cycle under way, removes `STOP` and exits.
- **Stopping now:** `kill -- -PGID`, the loop's process group, which takes the driver and its jobs too. Then check
  for partial files as after a crash.
- **Resuming.** A cycle saves its progress in `state.json` phase by phase: self-play, tournament, decisions,
  own-games step, logs. Starting the loop (or `run`) again resumes where it stopped:
  - Self-play jobs resume from their last complete version. A job left running by an interrupted driver is waited
    for first.
  - A job keeps its measured cost and its parts, which are in `state.json` as soon as each command ends. A job cut
    short during its segment measures again, from its last saved version. One cut short during a chunk re-plans
    from its parts, counting the rounds the cut command saved at the cost then known (`cut_cpu`).
  - The tournament plays only what is missing up to the cycle's target.
  - The own-games step skips the lineages it has done.
  - The rows that self-play logged for a round cut short (`ladder.csv`, `results.txt`) are trimmed before the job
    starts again.
  - Smoke tests killed the driver with kill -9 mid self-play (also during a measuring segment, and after it) and
    resumed. The models, `ladder.csv` and `results.txt` were byte-identical to an uninterrupted run.
- **After a machine crash.** The loop doesn't restart by itself; restart it with the same command.
  - btrfs can lose the last ~30 s of writes, and models are written directly, not through a temporary file. So
    first run `find runs ../selfplay/league-cycles -size 0`.
  - A zero-length model makes that lineage's resume fail ("self-play failed" in the summary), or makes the
    tournament stop the loop. Repair it before restarting.
- A lock (`state.json.lock`) stops two drivers at once.

## Time and disk per cycle

- **Cycle 1** took 71 min (16:15-17:26): self-play 64 min, the tournament 1.5 min, the own-games steps and gates
  4.5 min (1,900 CPU-s of gates).
  - Its self-play was planned from `costs.csv`, measured on the quiet machine, and used 5.57 CPU-hours instead of 4:
    411-1,143 CPU-s per lineage, 1.39 times the budget on average. The non-tree lineages used 1.59-2.38 times it,
    the trees 0.86-1.86 times.
  - Under the loop's own load a ply costs 1.5-1.7 times its quiet-machine CPU time. The machine's 4 P-cores and 4
    LP E-cores share one power budget, the cache and the memory, and 2 jobs of 4 threads also run on the slower
    E-cores. The games were about as long as measured (-4% to +7%).
- **Cycle 2** measured its cost in the loop (see *Budget and round sizes*) and used 4.17 CPU-hours, 1.04 times the
  budget on average. The non-tree lineages used 0.97-1.06 times it. The trees used 0.75-1.46 times it, because a
  tree's CPU per game changes after its measuring segment: T-N-Nc-c's rose from 3.2 to 4.7 ms, T-N-NNs-merge-c's
  fell from 3.2 to 2.4 ms. The cycle took 58 min: 52 min of self-play (4.8 CPU-s per wall second), 1 min of
  tournament and 5.6 min of own-games steps and gates.
- **From cycle 3** the budget is 240 CPU-s and the tournament 10 round-robins: about 2.1 CPU-hours of self-play
  (26 min at cycle 2's pace), 2 min of tournament and 6 min of own-games steps and gates, so about 34 min a cycle.
  Later cycles take about the same or less, as lineages go dormant.
- **From cycle 5** the tournament is 5 round-robins of 65 players (20,800 games). `random-4`'s games cost 0.6-1.2
  CPU-s each (about 640 a cycle, ~8 CPU-min) and `random-3`'s 0.04-0.08 (~0.7 CPU-min), so the tournament's CPU is
  about what 10 round-robins cost before, and `random-4`'s slow games can make a tail at the end of the schedule.
  Cycle 5's took 2.2 min: 22-36 s per round-robin, against 13-16 s for cycle 4's 3,906 games.
- **From cycle 6** TAN-c adds its 240 CPU-s of self-play (2.99 ms CPU per game, measured by the deploy with the
  loop stopped, so about 80 rounds of 1000 games in its first cycle), its own step and gate, and 2 players to the
  tournament (22,110 games).
- **From cycle 10** the 5 new T-N trees add 5 x 240 CPU-s of self-play (about 3 ms CPU per game from scratch,
  measured by the deploy), their own steps and gates, and 10 players to the tournament (29,260 games). The 2 lineages
  the migration woke add 2 x 240 CPU-s of self-play while they stay qualified.
- **From cycle 11** F-N-d2jit-c adds 240 CPU-s of self-play at depth 2 (30.5 ms CPU per game, measured by the
  deploy on the mature models of the CNBP reference run, so about 197 rounds of 40 games), its own step and gate,
  and 2 players to the tournament (30,810 games).
- **From cycle 12** T-N-NNs-merge-d2jit-c adds 240 CPU-s of self-play at depth 2 (86.1 ms CPU per game from
  scratch, measured by the deploy, so about 70 rounds of 40 games), its own step and gate, and 2 players to the
  tournament (32,400 games).
- **From cycle 13** F-L-d4mix-c adds 240 CPU-s of self-play at depth 4 (2,260 ms CPU per game, measured by the deploy
  on the LR reference run's mature models, so about 11 rounds of 10 games; more per game from scratch), its own step
  and gate, and 2 players to the tournament (34,030 games). The label search of the own steps of F-L-d2mix-c and
  F-L-d4mix-c adds about 25-30 CPU-s a cycle, outside the budget.
- **From cycle 16** MoE-d2jit-c adds 240 CPU-s of self-play at depth 2 (152.2 ms CPU per game, measured by the
  deploy on the mature models of the MoE reference run, so about 39 rounds of 40 games; about 235 ms a game from
  scratch, 3.4 times LR at depth 2 under the same load), its own step and gate, and 2 players to the tournament. Its
  gate plays 1000 games at depth 2 like every gate, which for MoE costs about 3.4 times an LR's (243 CPU-s from
  scratch), outside the budget. With the 10 lineages woken after cycle 15 (see the decision step), cycle 16 has 28
  self-play jobs, about 6,720 CPU-s (cycle 15: 17 jobs, 4,701 CPU-s), and 85 players (35,700 games).
- **From cycle 19** T-L-Nc-fresh-c adds 240 CPU-s of self-play (1.66 ms CPU per game from scratch, measured by
  the deploy, so about 145 rounds of 1000 games; T-L-Nc-merge-c's recipe cost the same from scratch within noise,
  2.29 against 2.53 ms in one side-by-side run). Once its tree has delegators the fresh test adds a scoring pass
  over each round's positions, and with fewer merges its tree can grow larger than T-L-Nc-merge-c's, so a game
  may cost more, as T-L-Nc-c's do (4-7 ms against 1.6-2.8): the budget then buys fewer games. Its own step and gate
  (about 40 CPU-s at depth 2 from scratch), and 2 players to the tournament (37,410 games).
- **From cycle 23** T-L-Ne-fresh-c, T-L-Le-fresh-c and T-N-Nc-fresh-c add 3 x 240 CPU-s of self-play (2.78, 2.91 and
  2.97 ms CPU per game from scratch, measured by the deploy, so about 86, 83 and 81 rounds of 1000 games; side by side
  with their twins' recipes from scratch, 2 rounds of 1000 games cost 5.73-5.75 CPU-s against 5.61-5.66 under the same
  load), their own steps and gates (about 40 CPU-s each at depth 2 from scratch), and 6 players to the tournament:
  with T-N-H-c and T-N-NNs-merge-c retired by the rule after cycle 22, 87 players (37,410 games). As with
  T-L-Nc-fresh-c, once a tree has delegators the fresh test adds a scoring pass over each round's positions.
- **From cycle 25** F-P-c adds 240 CPU-s of self-play (0.99 ms CPU per game, measured by the deploy on the probit
  agent's v29 and v30, so about 242 rounds of 1000 games; the agent measured 2.1-2.5 ms a game at depth 0 against LR's
  2.9-3.1), its own step and gate (47 CPU-s at depth 2 in the check from v30), and 2 players to the tournament: 80
  players (31,600 games), with F-L-d1jit-c, T-N-Lc-merge-c and TAN-c retired, MoE-c and T-L-LLs-c frozen at ply 2 and
  T-N-LNs-merge-c at ply 1 by the rule after cycle 24. The tournament plays the same games on `chessai-probit` as on
  `chessai-cyc5`.
- **From cycle 26** T-L-Ne-pre-c, T-L-Le-pre-c and D-L-Nc-c add 3 x 240 CPU-s of self-play (2.16, 2.27 and 1.65 ms CPU
  per game from scratch, measured by the deploy, so about 111, 106 and 146 rounds of 1000 games, the DAG's while it is
  one leaf: once it forms, a game costs about 1.4 times T-L-Nc's and the budget buys fewer), their own steps and gates
  (40-70 CPU-s each at depth 2 in the checks, 102 for the formed DAG), and 6 players to the tournament: 86 players
  (36,550 games).
- **From cycle 35** T-L-Le-merge-d2jit-c, T-L-NLs-d2jit-c and T-N-Nc-merge-d2jit-c add 3 x 240 CPU-s of self-play
  (83.6, 80.8 and 87.2 ms CPU per game from scratch at depth 2, measured by the deploy, so about 72, 74 and 69 rounds
  of 40 games; a depth-2 game of the twins' grown models costs 56.5, 116.8 and 87.5 ms), their own steps and gates
  (46, 81 and 40 CPU-s in the checks, nearly all of it the gate's 1,000 games at depth 2), and 6 players to the
  tournament: 63 players (19,530 games).
- **From cycle 38** F-P-tau0-c adds 240 CPU-s of self-play (1.13 ms CPU per game, measured by the deploy on the probit
  reference run's v29 and v30, so about 212 rounds of 1000 games), its own step and gate (the gate's 1,000 games at
  depth 2 took 45 CPU-s in the check from v330), and 2 players to the tournament: 59 players (17,110 games), with
  T-L-NLs-c retired by the rule after cycle 37.
- **From cycle 39** F-P-tau0-d2jit-c adds 240 CPU-s of self-play (26.54 ms CPU per game at depth 2, measured by the
  deploy on the probit reference run's v29 and v30, so about 226 rounds of 40 games; 52 ms from scratch in the check),
  its own step and gate (the gate's 1,000 games at depth 2 took 52 CPU-s in the check from v6), and 2 players to the
  tournament: 61 players (18,300 games).
- **From cycle 40** T-L-NLs-merge-d2jit-c adds 240 CPU-s of self-play (79.89 ms CPU per game from scratch at depth 2,
  measured by the deploy, so about 75 rounds of 40 games; 72 ms over 6 rounds in the check), its own step and gate
  (the gate's 1,000 games at depth 2 took 45 CPU-s in the check from v6), and 2 players to the tournament: 63 players
  (19,530 games).
- **Disk:** every version is kept, about 2-3 GB of models per cycle. The saved games take about 330 MB (17 KB per
  game) and are deleted after the cycle unless `own_games.keep_pools` is set.

## Budget and round sizes

- **The budget is measured in the loop** (from cycle 2 on). Each self-play job:
  1. plays a measuring segment: the run's next rounds, sized for about 6% of the budget and accepted at 5% (12 CPU-s
     at 240; even the depth-2 recipes measure several rounds).
     If a command falls short, more rounds follow.
  2. takes its CPU per game from the segment (the segment's CPU time over its games);
  3. plays the rest in chunks with an exact resume (`--resume --rounds V`), from cycle 16 (the user, 2026-11-07).
     Before each chunk it re-plans: the rounds left are `int(budget left / (cost × round size) + 0.5)`, where the
     budget left is `budget_cpu_s` minus the CPU the job has used so far, and the cost is the last chunk's own CPU
     per game (the segment's, for the first chunk). A chunk is about 22% of the budget (`CHUNK`) or the rounds
     left, if fewer. The job stops when no round is left, so it overshoots or falls short by at most the drift
     within its last chunk plus half a round.
     - Through cycle 15 one plan, made after the segment, ran the rest as one command. Drift in the cost was
       never corrected, and jobs used 153-467 CPU-s in cycles 9-13. In cycle 13: F-L-d4mix-c 165 (its
       from-scratch first round cost 5.25 s a game, the later ones about 2.8), T-N-NLs-merge-c 421, T-N-LLs-merge-c
       1.48 times the budget, T-N-Le-merge-c 1.38, T-L-Le-merge-c 0.75, the depth-2 tree 0.74.
     - Chunks cost nothing measurable. A resumed command that plays one game takes 11-18 ms of CPU, model, opponent
       and the largest tree states included, and resume is exact. Played in 2 chunks, 4 rounds of T-N-NLs-merge-c
       and of F-L-c gave byte-identical models, states, `ladder.csv` and `results.txt` to 1 command.
  - Expensive recipes simply get fewer games. The cost covers everything self-play does: games, training and the
    trees' shadows.
  - The cost, the chunks (each part logs its rounds and CPU per game) and the planned end go into `state.json` as
    soon as they are known (see *Resuming*).
  - `cycles.csv` records each job's cost, segment rounds, start and end versions and actual CPU time, so any cycle
    can be replayed: `verify NAME... --cycle C`.
  - `cycles.csv`'s cost is the segment's. The chunks correct for games that get dearer or cheaper within the
    cycle (a tree as it splits, a model leaving its from-scratch start, a change of load).
- **`costs.csv`** holds the CPU seconds per game of each recipe's signature (the model kind, depth, move choice,
  labels and every model option), measured once on the quiet machine (`measure`; `--force` re-measures). It now only
  sizes the plan in `status`, the first measuring segment and the dashboard's ETA. Once a lineage has measured
  itself in a cycle, its own last cost replaces the table for that.
- **Round size:** `games_per_round(d) = 1000 × cost(LR d0) / cost(LR dD)` from `costs.csv`, rounded to 10 and capped
  at 1000, and never below 10. That gives d0 1000, d1 860, d2 40, d3 10 and, from cycle 13, d4 10 (the deploy
  measured LR at depth 4 for F-L-d4mix-c), the same every cycle.
- A cycle has at least 1 round. A MoE's first cycle has at least 2 (it needs 2 versions to resume).
- Cycle 1 ran on the old rule, with rounds from `costs.csv`: 346-537 for the LR and CNBP lineages, 181-250 for the
  trees, 99 for MoE and 78 for FM.
- The own-games step and the gate are not charged to the budget. Their CPU time is logged separately.

## Commands

    python3 cycle.py status            # lineages, state, next cycle's plan; whether the own-games step can run
    python3 cycle.py measure           # fill costs.csv (only the signatures it lacks; --force re-measures)
    python3 cycle.py init              # cycle 0 (makes the league if needed); --force redoes it before cycle 1
    python3 cycle.py run --cycles N    # N cycles (the loop runs 1 at a time)
    python3 cycle.py refit             # rewrite the league's reports with the cycles' fit; plays nothing
    python3 cycle.py setup-league      # make or adjust the league only (init does it)
    python3 cycle.py verify NAME ...   # replay a continued run's last rounds and compare the models byte for byte
    python3 cycle.py verify NAME ... --cycle C   # replay the lineages' self-play of cycle C (cycle 2 on), likewise
    python3 cycle.py freeze NAME PLY   # between cycles; also unfreeze NAME, retire NAME, unretire NAME, wake NAME
    python3 cycle.py selftest

## Files

- `lineages.txt`: the registry, 7 fields per lineage (name, archetype, binary, source, heldout, steps, flags), as
  its header explains.
  - Adding a lineage is one line; it joins at the next cycle.
  - Sources can be `scratch` (all of them now), `continue:<run>` or `init:<model>`.
  - To drop a lineage, delete its line: its last entries stay in the league.
- `config.json`: the settings.
  - Paths: `league_dir` and the others.
  - Budget, jobs, threads and nice.
  - The tournament: binary, 5 round-robins (from cycle 5; 10 in cycles 3 and 4), entry plies [1, 2],
    qualification at ply 2, seed, `retired.txt`.
  - `league_setup.add_players`: the random players.
  - Patience and margin.
  - The own-games step: lineage opponents only, dormant ones included, no first pool.
  - The gate: 1000 games, depth 2, 50%.
  - `league_setup` can also archive players (`remove`, `backup`). The final design doesn't use that.
- `costs.csv`: the cost table for plans, in CPU seconds per game (play plus train), with the date and load average.
- `state.json`: each lineage's status, strikes, version, version in the field, entries and last Elo, and while it
  is dormant the version that went dormant (`dormant_version`, from cycle 10). While a cycle
  runs, it also holds that cycle's phase and plan. `references` lists the players that never train (the 5 random
  players from cycle 5); `rel_references` the 3 whose mean vs refs subtracts.
- `cycles.csv`: append-only, one row per lineage per cycle:
  - self-play: games, rounds, round size, CPU and wall seconds, the CPU per game measured in the loop (`cost_ms`, in
    ms), the measuring segment's rounds, and the start and end versions (these four from cycle 2 on);
  - the 2-ply entry's Elo, ±, the predecessor's Elo, the peak's Elo and version (`peak_elo`, `peak_version`, from
    cycle 9; blank without an earlier rated version), from cycle 10 for a lineage decided while dormant the bar it
    was decided against (`pre_dormancy_peak_elo`, `pre_dormancy_peak_version`), the Elo relative to the
    references, strikes and the decision;
  - the own-games step: games, positions and CPU seconds;
  - the gate: wins, losses, draws, score, Elo, ±, CPU seconds, and kept or reverted;
  - the next version.
- `cycle-NNN.md`: each cycle's summary, opening with the pooled gates. From cycle 12 its table writes absolute
  ratings (Elo, previous, peak, vs refs) without a sign; differences (the gaps in the decisions, the gate's Elo) keep
  theirs (the user). Earlier summaries keep their signs. `cycle.py status` and the league's `league.md` follow
  the same rule.
- `runs/<lineage>/`: the lineage's run, with every version's model, the trees' last 2 states, `ladder.csv`,
  `results.txt` and `reverted/`.
  - A tree's `cycle-states/` keeps (as hard links) the states of the 2 versions each cycle starts from, so that
    `verify --cycle` can replay it.
- `runs/<lineage>.log`: its self-play, own-games step and gate output.
- `pools/cycle-NNN/`: a cycle's saved tournament games, as `round<t>/<white>~<black>/`.
- `loop.sh` and `loop.log`: the loop and its output.
- `dashboard.py` and `dashboard.log`: the dashboard and its output.
- `staged-c5/`: what went live after cycle 4: `cycle.py` (the peak rule, `rel_references`), `config.json` (5
  round-robins, `chessai-cyc5`), `lineages.txt` (`chessai-cyc5`), and a `deploy.py` that also adds `random-3` and
  `random-4` to the league's `players.txt` and to `state.json` (backups in `backup-<time>/`, with `state.json` and
  the league's `players.txt`). `models.csv` needs no row for them: `league.py` maps a player to its model by
  dropping the ply suffix, and `random` has its row.
- `staged-c39/`: what went live after cycle 39: `lineages.txt` (T-L-NLs-merge-d2jit-c, whose comment corrects cycle
  35's on T-L-Le-merge-c), by staged-c38's `deploy.py`, which first checked the md5s of `chessai-cyc5`,
  `chessai-cyc5-labels` and `chessai-probit` and would have left the loop stopped had any step failed; its `measure`
  took the new lineage's cost from scratch (79.89 ms a game at depth 2). `note_c39.py` adds the note to
  `cycle-040.md`.
- `staged-c38/`: what went live after cycle 38: `lineages.txt` (F-P-tau0-d2jit-c) and `config.json` (its
  `own_games.binary_by_lineage` entry, `chessai-probit`), by staged-c37's `deploy.py`, which first checked
  `chessai-probit`'s md5 and would have left the loop stopped had any step failed; its `measure` took
  F-P-tau0-d2jit-c's cost on the probit reference run's v29 and v30 at depth 2 (26.54 ms a game). `note_c38.py` adds
  the note to `cycle-039.md`.
- `staged-c37/`: what went live after cycle 37: `lineages.txt` (F-P-tau0-c) and `config.json` (its
  `own_games.binary_by_lineage` entry, `chessai-probit`), by staged-c34's `deploy.py` with `chessai-probit`'s md5
  check, which would have left the loop stopped had any step failed; its `measure` took F-P-tau0-c's cost on the
  probit reference run's v29 and v30 (1.13 ms a game). `note_c37.py` adds the note to `cycle-038.md`.
- `staged-c34/`: what went live after cycle 34: `lineages.txt` (T-L-Le-merge-d2jit-c, T-L-NLs-d2jit-c and
  T-N-Nc-merge-d2jit-c), by staged-c25's `deploy.py` with this deploy's binaries, which first checked the md5s of
  `chessai-cyc5`, `chessai-cyc5-labels` and `chessai-probit` and would have left the loop stopped had any step failed;
  its `measure` took their costs from scratch (83.6, 80.8 and 87.2 ms a game at depth 2). `note_c34.py` adds the note
  to `cycle-035.md`.
- `staged-c25/`: what went live after cycle 25: `lineages.txt` (T-L-Ne-pre-c, T-L-Le-pre-c and D-L-Nc-c) and
  `config.json` (their `own_games.binary_by_lineage` entries), by `deploy.py`, whose busy check now matches the loop,
  the driver and self-play jobs by what they run rather than by text in their command lines (the coordinator's
  suggestion after staged-c24's first run). It first checked the md5s of `chessai-presplit`, `chessai-dag` and
  `chessai-probit` and would have left the loop stopped had any step failed; its `measure` took their costs from
  scratch (2.16, 2.27 and 1.65 ms a game). `note_c25.py` adds the note to `cycle-026.md`.
- `staged-c24/`: what went live after cycle 24: `lineages.txt` (F-P-c), `config.json` (the tournament binary
  `chessai-probit`, F-P-c's `own_games.binary_by_lineage` entry and `measure.reference_runs.probit`) and `cycle.py`
  (the own step's opponents include the entries a freeze at the decision takes out; the selftest's probit case), by
  the same `deploy.py`, which first checked `chessai-probit`'s md5 and would have left the loop stopped had any step
  failed. Its first run (05:06:32) didn't deploy: its busy check took a watcher's shell in `cycles/`, whose command
  line named the driver, for a running driver; a re-run deployed at 05:08:19 once the watcher was stopped. Its
  `measure` took F-P-c's cost (0.99 ms a game). `note_c24.py` adds the note to `cycle-025.md`.
- `staged-c22/`: what went live after cycle 22: `lineages.txt` (T-L-Ne-fresh-c, T-L-Le-fresh-c and T-N-Nc-fresh-c) and
  `config.json` (their `own_games.binary_by_lineage` entries), by the same `deploy.py` (which would have left the loop
  stopped had any step failed), whose `measure` took their costs from scratch (2.78, 2.91 and 2.97 ms a game).
  `note_c22.py` adds the note to `cycle-023.md`.
- `staged-c18/`: what went live after cycle 18: `lineages.txt` (T-L-Nc-fresh-c), `cycle.py` (a lineage's own-step
  binary) and `config.json` (`own_games.binary_by_lineage`), by the same `deploy.py` (restarting the loop on the
  old files if anything had failed), whose `measure` took T-L-Nc-fresh-c's cost from scratch (1.66 ms a
  game). `note_c18.py` adds the note to `cycle-019.md`.
- `staged-c15/`: what went live after cycle 15: `cycle.py` (self-play in re-planned chunks; `wake`),
  `lineages.txt` (MoE-d2jit-c) and `config.json` (`dormancy_se` 1.5, `requalify_se` 1.0), by the same `deploy.py`,
  which ran the `migrate` file (the undo of the old dormancy rule), would have left the loop stopped had any step
  failed, and whose `measure` took MoE's cost at depth 2 (152.2 ms a game). `note_c15.py` adds the note to
  `cycle-016.md`.
- `staged-c14/`: what went live after cycle 14: `cycle.py` (the automatic freeze/retire rule, its preview and
  `unretire`) and `config.json` (`freeze_rule`), by the same `deploy.py`. `note_c14.py` adds the note to
  `cycle-015.md`.
- `staged-c12/`: what went live after cycle 12: `cycle.py` (the own step keeps the recipe's labels; the round-size
  rule at depth 4), `config.json` (`own_games.binary` `chessai-cyc5-labels`; `measure.games_by_depth` 4: 10) and
  `lineages.txt` (F-L-d4mix-c), by the same `deploy.py`, whose `measure` took the LR cost at depth 4 and
  F-L-d4mix-c's. `note_c12.py` adds the note to `cycle-013.md`.
- `staged-c11/`: what went live after cycle 11: `lineages.txt` (T-N-NNs-merge-d2jit-c) and `cycle.py` (the
  `freeze`, `unfreeze` and `retire` commands and the frozen status, unused; unsigned absolute ratings in
  `cycle-NNN.md` and `status`) and `league.py` (unsigned absolute Elo in `league.md`), by a `deploy.py` that can
  run `cycle.py` commands from its `migrate` file and backs up the league's `retired.txt` too; its `measure` took
  the tree's cost. `note_c11.py` adds the note to `cycle-012.md`.
- `staged-c10/`: what went live after cycle 10: `lineages.txt` (F-N-d2jit-c), by the same `deploy.py`, whose
  `measure` then took its cost. `note_c10.py` adds the note to `cycle-011.md`.
- `staged-c9/`: what went live after cycle 9: `cycle.py` (the pre-dormancy bar, its columns,
  `migrate-dormant`) and `lineages.txt` (the 5 new T-N merge trees), by the same `deploy.py`, which also ran
  `cycle.py migrate-dormant` (its file `migrate` says `dormant`) and whose `measure` then took the new trees'
  costs. `note_c9.py` adds the note to `cycle-010.md`.
- `staged-c8/`: what went live after cycle 8: `cycle.py` (the peak rule for strikes, the peak columns,
  `migrate-strikes`) and `config.json` (`dormancy_se` 2 and `requalify_se` 2 instead of `requalify_margin`), by the
  same `deploy.py`, which also ran `cycle.py migrate-strikes` (its file `migrate` says `strikes`). `note_c8.py`
  adds the note to `cycle-009.md`.
- `staged-c6/`: what went live after cycle 5: `cycle.py` (re-qualification by the difference's 95% range, "tan"
  in `TREE_KINDS`) and `lineages.txt` (TAN-c), by the same `deploy.py` with its league step off; its `measure`
  then took TAN-c's cost. `note_c5.py` in `staged-c5/` and `note_c6.py` here add each cycle's "Changes from this
  cycle" to its report.
- `staged-c3/`: the config swapped in after cycle 2 (budget 240, 10 round-robins), by the same `deploy.py`.
- `staged-pre-tan/`: the files swapped in after cycle 1 (`deploy.py` did it and logged to `deploy.log`); the old ones
  are in `backup-<time>/`.
- `staged/`: the same set plus TAN-c (on `chessai-cyc4`), held while TAN's smoothing was studied; superseded by
  `staged-c6/`.
- `league.py`: writes the league's `league.md`, with each player's status (lineage, retired, random) and Elo by ply.
- `selfplay/league-cycles/`:
  - `players.txt`, `retired.txt` and `models.csv`;
  - `games.csv`: every game, retired players' included;
  - `ratings.csv`, `ratings.md`, `head_to_head.md` (active players), `elo_history.csv` (with `elo_history.key`, which
    caches unchanged prefixes) and `league.md`;
  - `tournament.log`.
- `.target/`: the build directory of `bin/chessai-cyc2` (78 MB). It can be deleted.
- `chessai-cyc5-src.tar.gz`: the source `bin/chessai-cyc5` was built from (rs/'s Cargo files and src/).
- `chessai-cyc5-labels-src.tar.gz`: the source `bin/chessai-cyc5-labels` was built from (`chessai-cyc5`'s, with
  `src/main.rs` changed).

## The tool

From cycle 5 the cycles run `models/from-scratch/bin/chessai-cyc5`, built 2026-11-06 18:52 from the uncommitted
rs/ working tree (its source is saved in `chessai-cyc5-src.tar.gz`). It is `chessai-cyc2` plus two changes:
- the TAN agent's TAN code (`--model tan` only; TAN-c uses it from cycle 6). Its TAN path is `chessai-cyc4`'s: TAN-c's
  recipe gives byte-identical versions 1-10 on both, and resumed with 2 threads the same as straight with 1;
- the merge trees' switch fix (rs/src/delegation.rs). When a delegator switches to error routing (`--route-switch`),
  its revision windows start afresh. So a `--revise` tree judges the switched branch against its shadow on a full
  window of error-routed examples before it can merge it. If the router window and the revision window end on the
  same example, the switch is decided first.
  - Before, the merge check came on the very next example and judged the cluster-routed window: T-L-LLs-merge-c
    and T-L-NLs-merge-c merged right after every one of their switches (9 and 5 in cycles 1-4).
  - So the recipes of T-L-LLs-merge-c, T-L-NLs-merge-c, T-L-NNs-merge-c and T-N-NNs-merge-c change from cycle 5.
  - Every other lineage trains byte for byte as on `chessai-cyc2`. That was checked on the cycle-4 start states of
    all 30 lineages: resumed self-play (2 and 4 rounds), a tournament, the own-games step and the gate's arena.
    A non-merge tree only logs and saves restarted windows.
  - `verify --cycle C` replays with the lineage's binary in `lineages.txt`, now `chessai-cyc5`. For the 4 merge
    lineages, a cycle up to 4 that had a switch only replays with `chessai-cyc2` (a `--config` whose lineages file
    points there).

From cycle 13 the own step's train and gate run `models/from-scratch/bin/chessai-cyc5-labels`, built 2026-11-06 22:40
from `chessai-cyc5`'s source (`chessai-cyc5-labels-src.tar.gz`; only `src/main.rs` differs, and rs/ doesn't have the
change). Self-play and the tournament still run `chessai-cyc5`. It adds:
- `train --label-depth D --label-result W [--label-model SPEC]` (LR only): each position's label is the outcome a
  depth-D search by the start model (or the fixed model SPEC) expects, mixed with the game's result at weight W and
  calibrated on the round's games, as self-play labels its games. The positions come from the games' moves
  (`human.games`, which `tournament --save-games` writes) and are checked against the saved ones; they are trained in
  the same order as without labels.
- `replay-games`: replays chosen tournament games exactly (the test agent's check of the old own steps, and of this
  change).
- Everything else is `chessai-cyc5`'s: without labels, every lineage's own-step train (36 lineages, models and
  states) and the gate's arena (8 kinds) give the same output on both. F-L-d2mix-c's cycle-8 own step through the new
  driver gives the test agent's look-ahead model byte for byte (gate 415-360-225, kept), and its result-label train
  gives the logged cycle-8 model.

From cycle 19 T-L-Nc-fresh-c's self-play, own-step train and gate run `models/from-scratch/bin/chessai-resid4` (md5
a872a8123f568fe6794b6d86f3af2316), and from cycle 23 T-L-Ne-fresh-c's, T-L-Le-fresh-c's and T-N-Nc-fresh-c's: the
residual agent's build of `chessai-cyc5-labels`' source plus `models/from-scratch/resid/resid4.patch` (source in
`models/from-scratch/resid-src/`). It adds residual leaves (`--leaf-residual` and its options, unused here),
`--revise-on stream|fresh` and `--log-fresh-gains`, in self-play and in train (fresh needs rounds with game ids:
self-play's, or train's round of game directories). With them off its output is byte-identical to
`chessai-cyc5-labels`' (the residual agent's check, self-play and offline train). Its model files are plain trees
(they don't record the merge test), which the tournament's `chessai-cyc5` plays. Checked on a scratch root before the
deploy: fresh self-play with `--no-games` (it scores the round in memory) and with merges; a resume from a fresh state
byte-identical to one command; `--log-fresh-gains` leaving every file unchanged; a `chessai-cyc5` tournament with its
entries saving the games; and an own step through the driver on them, train and a 1000-game gate at depth 2. Checked
again before cycle 23's deploy, for its three twins: fresh self-play with error routing (T-L-Ne-fresh-c,
T-L-Le-fresh-c) and with `--leaf-share` (T-N-Nc-fresh-c), merging at a low split threshold (20k: 3, 2 and 4 merges in
6 rounds of 500 games), and models byte-identical to their twins' over 2 rounds from scratch at the real threshold; a
resume byte-identical to one command; a `chessai-cyc5` tournament with their entries; and an own step through the
driver on each (`own_games.binary_by_lineage`), train and a 1000-game gate at depth 2.

From cycle 25 the tournament, and F-P-c's self-play, own-step train and gate, run
`models/from-scratch/bin/chessai-probit` (md5 576db2c21f7997fe09bcd1ac790ffd77), the probit agent's build of
`chessai-resid4`'s source plus `models/from-scratch/probit/probit.patch` (source in `models/from-scratch/probit-src/`;
an unmodified build is byte-identical to `chessai-resid4`). It adds `--model probit` and its `--probit-*` options
(their defaults here). The agent checked it against `chessai-resid4` for every other kind
(`models/from-scratch/probit/identity/summary.txt`: eval-loss, a tournament, self-play with resume and training
states, and train). Checked against `chessai-cyc5`, the tournament binary through cycle 24, before the deploy: it
replayed cycle 22's first round-robin (85 players: lr, cnbp, bdt, ht, tan, moe, fm and the random players at plies
0-4, calibrated as in the league) with every game's result and length as in `games.csv`; it replayed the 1,002 games
of cycle 23's three fresh twins (`chessai-resid4` trees) with the same results and saved games, byte for byte, as
`chessai-cyc5` and the cycle's pool; and it rewrites the league's ratings, head-to-head tables and Elo history byte
for byte. `chessai-cyc5` can't load a probit entry ("unknown model kind"). F-P-c's self-play in chunks (3 rounds, then
`--resume` to 6) gives the agent's straight run byte for byte, and its own step through the driver trains
`--start probitN.model` and gates with `cal:...:probit:` entries.

From cycle 38 F-P-tau0-c runs `chessai-probit` the same way, with `--probit-tau 0`. Checked before the deploy, on
`chessai-probit` at nice 19 and 2 threads: from scratch, 6 rounds of 1000 games gave the probit agent's tau-0 run
(`models/from-scratch/probit/tau/tau0`) byte for byte, with tau 0 in every model file; 3 rounds and `--resume` to 6
gave the same files as one run; its own step through the driver, from the agent's v330 on saved league games, trained
with `--start` and `--probit-tau 0` and gated 1,000 games at depth 2 (55.4%, kept), and the kept v331 still has tau 0,
with lower weight variances; and a round-robin as the driver runs it, with its entries (v6 and the agent's v330 at
plies 1 and 2), F-P-c's and two random players, played every game (56 games, all saved).

From cycle 39 F-P-tau0-d2jit-c runs it the same way at depth 2. Checked before the deploy, on `chessai-probit` at nice
19 and 2 threads: from scratch at depth 2, 6 rounds of 40 games ran (52 ms a game), with tau 0 in every model file; 3
rounds and `--resume` to 6 gave the same files as one run; its own step through the driver, from its v6 on saved
league games, trained with `--start` and `--probit-tau 0` and gated 1,000 games at depth 2 (kept), and the kept v7
still has tau 0; and a round-robin as the driver runs it, with its entries (v6 at plies 1 and 2), F-P-tau0-c's,
F-P-c's and two random players, played every game (56 games, all saved).

From cycle 26 T-L-Ne-pre-c and T-L-Le-pre-c run `models/from-scratch/bin/chessai-presplit` (md5
ba083fc7908338108792279e58b9b3f9) for self-play and for their own steps' train and gate: the presplit agent's build of
`chessai-resid4`'s source plus `models/from-scratch/presplit/presplit.patch` (source in
`models/from-scratch/presplit-src/`). It adds `--pre-split` (and `--pre-split-fresh`, not in the recipes); without
them its output is `chessai-resid4`'s (the agent's check: offline models, states and logs, and 16 rounds of self-play,
every file). The candidates live in the state files (`bdtN.state`), and the model files are plain trees. D-L-Nc-c runs
`models/from-scratch/bin/chessai-dag` (md5 036be3c53c69c84025c55b807c3753a5) the same way: the DAG agent's build of
`chessai-resid4`'s source plus `models/from-scratch/dag/dag.patch` (source in `models/from-scratch/dag-src/`), which
adds `--dag-leaves`, `--dag-window`, `--dag-z` and `--dag-reseed` and gives `chessai-resid4`'s output for every
existing option (offline models, states and logs, and 6 rounds of T-L-Nc's self-play with the log). Checked before the
deploy, from scratch against each new line's twin on `chessai-cyc5` (T-L-Ne-c's and T-L-Le-c's lines; for the DAG, its
line without the `--dag-*` options): the same games and models over 2 rounds of 1000 games; at a 300k split threshold,
rounds of 500 games, the same models up to the first split (through v4 for the pre-split trees, whose first splits
came at v5 in both runs, and through v1 for the DAG, which formed at v2); at 100k, a run in two chunks, 3 rounds and
`--resume` to 6, byte-identical to one run, with promoted candidates and the formed DAG; a round-robin of their
entries (exact and 100k, calibrated) with the same games, saved games and reports on `chessai-probit` as on
`chessai-cyc5`; and each new lineage's own step through the driver, train and a 1000-game gate at depth 2 on its
binary.

`chessai-cyc2`, which ran cycles 2-4, is built from rs/ (2026-11-06 15:49). A clean rebuild is byte-identical.
- Self-play is identical to `chessai-cyc`, and so are its outputs in the old mode.
- New tournament flags:
  - `--no-catch-up`;
  - `--retired FILE`;
  - `--prior-draws 0` for pure maximum likelihood. The default of 1 is the old fit.
  - `--save-games DIR`.
- New in the player: random players at depth 1 and up.

Don't rebuild or overwrite it while cycles run.
