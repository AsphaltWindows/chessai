# New league design: the decisions in force

The design the user started on 2026-11-09 ~11:45, trimmed to what is decided and still holds. Updated 2026-11-10
~21:10 (the ply ladder; the global league: a thunderdome, one ply per model, two entries, training on games of
players at least as strong or a little weaker; G lines: a fixed Elo cutoff).

- Status: the engine is built (leagues/). Experiments run (NEXT-EXPERIMENTS.md). The global league has not been
  launched.
- How to create and run leagues: leagues/BUILD.md. Files and formats: leagues/FORMATS.md. Arenas: leagues/ARENA.md.
- The full discussion log is archived: archive/2026-11-10/models/NEW-DESIGN.md. It holds the readings told to the
  user, the cost estimates, the stage-1 brainstorm, and the build, launch and batch notes with their times and pids.
  The parts that no longer hold are listed at the end.

## The user's aims

- Get the most out of the compute. Generating games is the most expensive part.
- Keep the questions separate:
  - A: which model gets strongest with self-play;
  - B: which gets strongest given the same dataset;
  - C: whether higher-quality data (deeper search) is worth it, and from what point;
  - D: which models generate the best training data, possibly pair by pair.
- Offline games are too biased to give good results. Experiments replace them (head-to-head tests, twins, panels).
- Architecture questions are B questions, so routing experiments are local (every line trains on the experiment's
  games).
- **The ultimate goal** (the user, 11-10 ~20:15): an evaluation function strong enough to drive an alpha-beta search
  deeper than ply 1 or 2. **The plan (the ply ladder):** most initial experiments at ply 1, then "upgrade" the ideas
  that hold to ply 2. Evidence that ply-1 verdicts don't always carry to ply 2: RESULTS section 6.
  - **No more ply-2 experiments for now** (the user, 11-10 ~22:15): "apart from the test we're doing (the result of
    which is probably very evident now, as it's becoming quite obvious that the random players and players of
    unrelated plies should simply not be present in the experiments) we really have no reason to run any more ply 2
    experiments at the moment". Batch 4 ends after leaf-sources-new; experiments run at ply 1 (every entry, its
    training games and its champion test at ply 1; the engine's entries'-ply option, being built).

## The global league (decided; the 11-10 design built 2026-11-11, not launched)

1. It provides two things only: the practical-strength ranking and high-quality games. Everything else is answered
   elsewhere.
2. No gating, no self-play: its players train on league games only.
3. No global tournaments.
4. Structure (the user, 11-10 ~20:30):
   - Every model enters with the ply it plays at, and plays at that ply against everyone, whatever their plies. It has
     two entries only: its current version and its champion. The champion test is the current version alone, at 2 SE
     of the fit. No model plays at a second ply: a model whose training favoured one ply playing at the other "just
     doesn't make sense" (the user). An upgrade enters the same recipe as a new model with ply 2.
   - "The league is a thunderdome" (the user, 11-10 ~20:50): any model at any ply, everyone against everyone. Mixed-ply
     training data is fine here; ply-1 models will train on games with ply-2 models. Clean one-factor comparisons are
     the experiments' job.
   - Training (the user, 11-10 ~21:10): each model trains on the cycle's games in which both players are at least as
     strong as it, or at most a little weaker: both rated at or above its current version's Elo minus a small margin
     (the margin is under Open). Each model trains with its own recipe, in a fixed game order, so runs repeat exactly. No self-play
     top-up for the leaders (the user, ~21:25): they train on few games, and the count shows who led ("we'll be able
     to tell who lead the pack by how few games they were trained on"), so each version's training-game count is
     recorded and shown.
   - Versions G.<champion>.<cycle>. A new model enters as G.0.0 (its first champion) and untrained.
5. Cycles are 5 double round-robins. Variable-length cycles (the user, 2026-11-11 ~20:10: "we should just have
   variable length cycles based on the number of participants in the league"; built ~20:40, OFF until the user
   sets G): with config.json `games_per_entry` G, each cycle plays ceil(G / (2 (N - 1))) double round-robins for
   its field of N entries (references, and each live model's current and champion), so every entry plays at
   least G games whatever the league's size. A cycle keeps the rounds it planned if G changes mid-cycle. Pooling
   the champion test over cycles was rejected (the user: "You can't pool across cycles, because we train once
   per cycle and you're no longer comparing the elos of the same models").
   Games per cycle (the user, 2026-11-12 ~01:15: "equilibrate the amount of games towards some standard
   range/amount by changing the number of rounds per cycle based on the number of active participants", in
   experiments and global leagues, "because we want more stable elos and also a more consistent training batch
   size"; the rule analyst 6.53's; built 2026-11-12, for new experiments and the next global league, OFF unless a
   spec sets T): with `games_per_cycle` T each cycle plays R = round(T / (N (N - 1))) double round-robins, at least
   1 and at most 48 ("under T" when the cap binds), N every entry of the cycle's round-robins, recomputed each
   cycle. It replaces the fixed 5 rounds (rounds_per_cycle) and is exclusive with `games_per_entry`. Standard
   T 1584 (range 1200-2000): rating SE about 348 / sqrt(2T / N), 17 Elo at N 8, 21 at 12, 28 at 20. Self-games
   (entries x rounds) sit outside T. leagues/FORMATS.md config.json has the details.
6. Patience, revised 2026-11-11 ~13:45 (the user: "patience should probably also be scaled in some way by the total
   amount of models ... a global league with a small amount of participants is going to retire everyone quickly";
   my proposal, the user: "Ok that works"): patience counts training data, not cycles. Each cycle without a new
   champion moves a model (positions its current version trained on that cycle) / P* of the way to retirement, P* =
   20M positions. It retires at 100%; a new champion resets it to 0; no retirement within 5 cycles of the last
   crowning or entry; no revivals. Every model accrues it, the top one included (the user, ~13:50: "the top model
   still trains on its own positions ... maybe we should just get rid of the percentiles altogether"): no
   percentiles anywhere. Fewer models mean fewer games a cycle and so slower progress; the Elo window gives stronger
   models fewer games, so they accrue it slowly (a top model well ahead trains on a few thousand positions a cycle). P* from the ply-1 experiments: lines that later crowned again had
   gone up to 6.7M positions without a crowning in 90% of gaps, 17.7M in 99%, 37.8M at most. For the six at ply 1
   (~120 positions a game, ~1,300 games a cycle): ~160 cycles at the bottom, 300-400 at the median. The percentile
   rule is dropped from the engine. Superseded version:
   Patience (the user, 11-10 ~21:40, replacing a flat 20 cycles): each cycle without a new champion moves a model
   (1 - p)^2 / 25% x 5% of the way to retirement, where p is its current version's percentile among all model
   entries (current versions and champions, since it plays both and champions' Elo is more reliable: the user;
   references out; 1 the top, 0 the bottom). The current version, not the
   champion (the user, ~21:45), because its Elo is the one that selects the training games.
   It retires at 100%; a new champion resets it to 0. No revivals. (1 - p)^2 is about the share of games a model
   trains on when it learns only from players above it (25% for the median model), so a model retires at the old
   pace scaled by how much it trains. Patience in cycles = 5 / (1 - p)^2: 5 at the bottom, 9 at p = 0.25, 20 at the
   median, 80 at 0.75, 500 at 0.9, and never for the top model.
7. Calibration is per version, on 600 of its own ply-1 games, never trained on (about 2 CPU s a version). Fitting it
   on the shared games instead needs a check first (a fixed held-out calibration once biased ratings by ~132 Elo per
   unit of ln(slope ratio)).
8. References: random, random-1, random-2 and random-3 (random pins 0).
9. Nothing launches until the experiments are designed (the user, 11-09 ~14:20).

## Experiments (decided; running)

- **What one is.** A small league that runs until its champions plateau (since 11-10; before, a set number of
  cycles); nobody enters halfway. Pick models and at least one option; each model runs under every chosen option.
  Each (model, option) pair has two entries: a ply-2 competitor (the current version) and a ply-2 champion, one
  entry when they are the same version (before 11-10 also a ply-1 competitor).
  - Self-play (S): a solo version trains every cycle on its own self-play.
  - Local (L): trains every cycle on all the games of that cycle's experiment tournament.
  - Global (G, uncommon): trains every cycle on the latest global-league games. Until the global league runs, only S
    and L exist. A G line sets a fixed Elo cutoff for the games it trains on (the user, 11-10 ~20:55: "when we run
    experiments and specify global training we should be able to specify ELO game cut off"). That's how an
    experiment picks the level of its steady stream. Drift doesn't matter here (the user, ~21:10): the global league
    should progress slowly, like the champion run near its end, so game quality hardly changes during one
    experiment. Details under Open.
- **Each cycle:** (1) with the global option, copy the fewest full global rounds whose generation CPU covers 5 or
  more experiment rounds, and set the experiment's rounds to match; otherwise play the spec's rounds. (2) Set each
  self-play line's budget. (3) Play the rounds. (4) Train: global lines on the copied rounds, local lines on the local
  games, self-play lines on their own games. Repeat.
- **Training games:** local and global lines train only on games in which a model plays at ply 2.
- **Fairness:** every line trains on about the same CPU's worth of games a cycle. Self-play runs after the
  tournament, and its budget is the priced generation CPU of the games the local lines train on (training CPU is not
  counted). Experiments with self-play lines only get a fixed budget: 60 priced CPU s a line a cycle (the user halved
  120 on 11-09 17:54).
- **Prices:** a fixed price formula, not measured CPU, so experiments replay exactly (the user, 11-09 ~16:10).
  Default prices: leagues/prices/P2.md. Measured CPU is recorded beside them.
- **Defaults:** every version starts untrained; the champion test is the global league's; no patience; the four
  random players were the references until 11-10 (below); anchors start at 100; the last cycle trains nothing; an
  experiment ahead of the global league waits and never reuses a round.
- **The user's experiment defaults of 2026-11-10 (~17:00)** (engine options: leagues/FORMATS.md, config.json):
  - No random players in experiments. Random players belong in arenas (no training happens there), in the global
    league, or in an experiment only on request. Playing them wastes compute and adds noise that a run aimed at the
    strongest model wouldn't take on.
  - No ply-1 entries.
  - Relative strength only inside an experiment: its ratings are relative to a fixed entry (the first line's
    untrained version, `rating_zero`). Absolute placement comes afterwards, from arenas of the champions (with
    random players), which are much cheaper than pinning every cycle to random.
  - Champions remain the gate. The user: engines such as lc0 and Stockfish run gating tests before each release;
    the champion test is ours. Not gating would make sense once an architecture is trusted; with this much noise
    we have to know when to cut our losses.
  - Run until the champions plateau: an experiment ends when no line has crowned a new champion for 50 cycles, not
    before cycle 100, at most 1000 cycles. The user: "who knows if any of those results are even valuable. If it
    runs for 1000 more runs maybe everything would reverse."
  - **Defaults added 11-10 ~22:00 (the user), for experiments started from now on:**
    - Self-games: whenever local training is involved, every entry also plays itself once a round. The games train
      (local lines, and the global league's pool), never rate (a game against yourself carries no rating
      information). They count in the priced games that set self-play lines' budgets, so mixed self-play and local
      experiments stay fair (to be checked in the build).
    - Plateau window scaled by the number of lines n: K = round(50 x sqrt(2 / n)) cycles without a new champion
      (50 at 2 lines, 35 at 4, 25 at 8, 19 at 14), from cycle 100, cap 1000. Reasoning and simulation under Open.
  - Batch 4 reruns every earlier experiment this way, one at a time, and arenas then rate each original's and
    rerun's champions together (NEXT-EXPERIMENTS.md).
- **Versions:** E.<S|L|G>.<champion>.<cycle>, for example E.S.2.17.

## Rules in force for experiments

From batches 1-3 (the user's corrections on 11-09 and 11-10):
1. **One factor per comparison**, against a named comparator. Seeds count as a factor. In a local experiment, lines
   without replicates share the train seed.
2. **Only the lines the user asks for.** In a local experiment every line's games are every local line's training
   data, so one extra line shifts all of them. Replicates and extra conditions are proposed separately.
3. **Field shape.** The share of training games against the random references depends on the number of lines n:
   8n / (C(3n+4, 2) - C(n+4, 2)): 42% (n=3), 35% (4), 30% (5), 26% (6), 23% (7), 13% (14). Per-family experiments
   use 4 lines. (Batches 1-3. Without references and ply-1 entries, from 11-10, every game trains: 2n(2n-1) games
   a round.)
4. **Same tree rules unless the rule is the factor:** split limit, threshold, children, error measure, forget factor,
   merge setting.
5. **Unpriced compute is a factor.** Training CPU is not priced; recipes that spend extra training compute (search
   labels: 12×) get it free. Price it or keep it out.
6. **Noise.** One version swings ~17 Elo (SD), windows of versions ~30. Compare pooled windows and replicate gaps
   under ~40 in a second window.
7. **Measure current against current** (direct games between current ply-2 versions). Version windows count
   champions.
8. **Check that a compound model is still compound:** routing shares and leaf distinctness at checkpoints.

Model rules:
- Stage 1 is the leaf classifiers alone: LR, probit and NB. FM and TAN are out of stage 1 (TAN routers are in use
  since batch 2).
- Routers: LR, NB (CNBP) and TAN. Probit and FM are out of routing (the user, 11-09); an FM router is built for
  later.
- Hoeffding trees are closed (the user, 11-09). NB as a predictor (leaves, experts) is abandoned (11-08); NB routers
  and gates stay.
- Merges: never for error routers (the user, 11-08). Cluster routers may merge (--revise 0).
- Trees: 3 children, split threshold 1,000,000, error trees with --error log. Split limit 1 so far; limits will rise
  (split-growth runs limit 4) and designs must scale linearly with size.
- Search labels stay out unless their search is priced like self-play.
- Binaries: new builds under new names; never overwrite models/from-scratch/bin. A binary swaps into a running
  experiment only byte-identical and between cycles (leagues/BUILD.md). Speedups: PERF-LEDGER.md.

Ground rules (memory research-constraints): self-play only, zero knowledge, no hand features, search as a last
resort, hard routing.

## Open

- **The global league under the ply ladder** (11-10 ~20:15-21:10). Decided by the user (sections above): one ply per
  model and two entries per model; the league is a thunderdome; each model trains on games where both players are at
  least as strong as it or a little weaker; G lines in experiments set a fixed Elo cutoff. Open:
  - The margin ("at most a little bit weaker"). In the champion run's last round robin, with both players rated at or
    above the model's Elo minus the margin, the median model keeps 14% of the games at a margin of 15, 18% at 30, 20%
    at 50 and 24% at 100. The strongest tenth keep 2%, 4%, 8% and 19%, and the weakest models 35-45%
    (fs/analysis-2026-11-10/champply/window.py). New, untrained models train on nearly every game between models.
  - Patience (rule 6): a variant scales by the model's actual training share that cycle instead of (1 - p)^2: it also
    counts the margin's extra games and its current-against-champion games (in the champion run's last round robin
    the median model kept 14-20%, not 25%). Needs an engine change (patience_cycles is global only, so no running
    experiment uses it).
  - The G cutoff's details (my proposal): both players rated at or above the cutoff; ratings from the global league's
    fit for that cycle, random pinned at 0; the fairness count after the cut, so a high cutoff copies more global
    rounds and may wait for them.
  - The global league as a steady stream of games for experiments (the user, ~20:40): the G option made the default
    for architecture questions. Every line learns from games of the same, steady quality; the games are generated
    once for all experiments; an experiment's own games only rate its lines. Experiments that care about ply don't
    use global data (the user, ~21:00): "You can imagine that there's some model that's as good or better at ply 1 as
    another model at ply 2, I find it hard to believe that the ply 1's games would be worse training games for ply 2
    models in an experiment than the weaker ply 2 model's games." Risk: strong games rarely contain the positions a
    weak model reaches, or the positions after blunders that a search must still score (PATH-FORWARD (c)). It could
    be tried before the league exists, on the champion run's recorded games (15.8M, both plies).
  - Calibration: on own ply-1 games for every model (today), or at the model's own ply (~12x the CPU).
  - From the 20:15 proposal: entry after the ply-2 upgrade test; ply-1 screening experiments need an engine option,
    byte-identical for existing configs.
- **The plateau window against the number of lines** (decided ~22:00, see Experiments; the reasoning:). The stop needs K cycles in which no line crowns, so n and K act as a product: at a plateau each line
  still crowns now and then (rate q), and a quiet window has probability about (1 - q)^(nK). (Batch 4 set its rounds
  to keep the originals' games a cycle, so there entries also play fewer games as lines are added and q rises a little
  more; with the rounds fixed instead they would play more. The scaling holds with every entry's games held equal,
  sim_fixed_entry.py: K = 50 stops 2 / 4 / 8 / 14 fast-learning lines at 232 / 284 / 364 / 476.) Simulation (fs/analysis-2026-11-10/plateau/;
  exponential curves, version noise 17, the 2-SE test, G = 2,400): with K = 50 the median stop is cycle 233 / 280 /
  364 / 634 for 2 / 4 / 8 / 14 fast-learning lines, with 1.5 / 0.5 / 0.1 / 0 Elo still to gain; slow learners stop at
  391 / 492 / 647 / 971 with 19 / 8 / 2 / 0.2 left (45% of 14-line runs reach the cap). Equal gain left at the stop
  needs K = K2 x sqrt(2 / n): with K2 = 50, 35 / 25 / 19 for 4 / 8 / 14 lines (K2 = 100 for slow learners: 71 / 50 /
  38). The user (~22:05): "everything that hasn't started should follow the new standard", so the queued batch-4 reruns and small-field-x4 would get it; then (~22:15) no more ply-2 experiments for now, so batch 4 ends after leaf-sources-new and the defaults start with the ply-1 experiments (K = 41 / 35 / 32 / 29 / 27 / 25 / 19 for 3 / 4 / 5 / 6 / 7 / 8 / 14 lines); leaf-sources-new, running, keeps K = 50.
- **Self-games in the round robin** (decided ~22:00 for experiments with local training, see Experiments; the
  global league too). Each entry also plays itself. A game against yourself carries
  no rating information, so it can't move the fit; it is pure training data. Under the training rule it qualifies for
  the model and every model below it, so it helps the leaders most. Cost: one game per entry a round adds 1/(E - 1)
  games (1% with 100 entries, 5% with 20, 33% in a 2-line experiment). Needs stochastic play (seeded softmax: yes)
  and a tournament program that lets a player meet itself. In experiments it changes every line's data, so it would
  be a factor (an option, off in running ones).
- **Replay of earlier cycles' games** (option built 11-11 ~22:00; its test replay-p2 is drafted, not launched;
  analyst 6.46 (i)). The offline order and mix checks showed the probit and LR learners follow their most recent
  data, so a pure ply-2 field ends on its newest, narrowest games; a stationary mix was best (the interleaved P2x/P2
  flat beat the P2-only flat by 186 at ply 2; trees beat flats on stationary data and lose under a switch). With
  `"replay": {"share": S, "window": W}` each cycle's local set adds round(S / (1 - S) x its own games) games sampled
  uniformly from the own games of the W cycles before (leagues/FORMATS.md, config.json `replay`). Self-play data
  only: the replayed games are the run's own. It changes every local line's data, so it is a factor (an option,
  off in running experiments); replay-p2 (share 0.5, window 50) is probit-late-p2 plus replay, read head-to-head at
  equal versions.
- Experiments sharing the global league's anchors, for one Elo scale.
- What an experiment graduates (a recipe, since models enter the global league untrained), and the test for it.
- The launch population and the CPU split between the global league and experiments.
- Recipe names: today's names encode old self-play settings such as d2jit.
- Prices: early cycles of untrained models (long games) are not priced right; a tree size term (log2 of model bytes
  over 20 KB a move) is on hold until the user says yes.
- Keeping the latest global batch on disk (the user, tentative: "maybe we could keep just the league's latest batch
  of games"; about 2-4 GB), and "shadow" recipes that train on each batch without playing, rated against anchors (my
  proposal).
- Calibration without own games (untested): fit on the version's own league games, or a scale-free policy that picks
  by rank.
- The rules up for review (the user's 11-09 ~13:15 list): self-play-only data and zero knowledge; search as a last
  resort; hard routing; error routing as the direction; no new error-router merges; no NB leaves or experts; never
  cut lineages; one factor per experiment with the round robin as the judge; about 60 active lineages.

## Theory, in brief

models/from-scratch/theory/summary.txt (2026-11-10 ~01:10; RESULTS.md section 10 lists the items):
- Best, Soft and Balanced router targets are inconsistent for routed log-loss. Consistent targets: the pairwise gap,
  fractional labels, weighted all-pairs (built in bin/chessai-target; running in error-targets).
- Every error router is an evaluator in disguise: its target is an outcome probability. LR routers show it; NB
  routers route like label-free clustering.
- After a split consistent targets are silent, so symmetry must be broken without labels.
- Additive (residual) nodes are the only growth operation with a monotone guarantee (T3).
- The right split trigger is the size of the leaf's mean gradient over candidate regions (T4), not stalling.
- A leaf split breaks even at about a million positions per leaf (conjecture).

## Superseded (kept in the archive)

- The stage-1 brainstorm slates (the 7-experiment and 3-experiment tables, the combined 13-line local league): replaced
  by batch 1's per-family experiments (RESULTS.md section 7).
- The cost table for the global league (estimates from the old league's cycle 51).
- "Search labels in local mode cost training time only": wrong, training CPU is unpriced.
- One round-robin a cycle and a champion test on the mean of the last 5 versions: withdrawn (the user, 11-09 ~14:40).
- The 120 s free self-play budget: halved to 60 s.
- Ply-2 entries with frozen ply-1 anchors, and every model training on the same games: replaced on 11-10 ~20:30 by
  one ply per model (two entries) and training on games between players at least as strong or a little weaker
  (~21:10).
- The old league's data notes (game pools deleted each cycle; "we don't need the current league's games").
- Build, launch and batch notes with pids and times: leagues/BUILD.md, RESULTS.md section 7, the archive.

## The next global league's end rule (the user, 2026-11-12 ~11:08)

"for the global league, I think as discussed we need to increase the number of rounds as the number of remaining
competitors decreases. Alternatively maybe, we could just never retire the league and have it shut down just like
experiments do after there hasn't been any new champions for a while. I think that's actually the better option
probably."
- No retirement of lines. The league stops on the experiments' plateau rule: no new champion for a window scaled by
  the number of lines, after a floor, with a cycle cap as backstop. Crownings (and so the stop) run on the cold
  (T 0.02) ratings, the user's 10:54 rating decision.
- games_per_cycle stays in the spec; with no retirement the field size stays constant.
- Before launch (from the first league): cap the rating history (it grew ~20 GB a day).
- Lineup, ply and timing: update 8 (analyst 6.71/6.83), awaiting the user.

## The global league's new purpose (the user, 2026-11-12 ~11:11; "Yeah, I think that's a better future for it")

"what we should do with the global league is not train anyone in it at all. and simply have it be an arena that runs
occasionally with all of our promising models where the models run at the ply that's specified for them and that
helps us establish a global ELO ladder. [...] But in addition, I would also like to consider something like the
current global league but an experiment, basically where each model only trains on games from models of the same or
higher elo. It's a less controlled environment but maybe one from which we could glean insights rather than hard
conclusions."

**(1) The global ladder** (analyst 6.85): an occasional arena, no training. Players are (model, depth): each promising
library final at its specified ply (default its training ply), one entry per model, 8-16 per ply; a few anchors may
also enter at their ply +-1 to calibrate the depth axis, labelled. No random player: the mean of a FIXED reference set
(the standing panels per ply) is pinned at zero, so the scale persists across runs and matches the experiments' panel
reads. 10 double round-robins at ~40 entries (~780 games an entry, SE ~12, +-25 on a contrast), ~1.5-2 CPU-h (the
ply-4 side ~0.8); re-run after each batch of library additions. It ranks DEPLOYED players (evaluator plus search); it
cannot rank evaluators apart from depth or give a transitive cross-ply scale (models swap ~380 between plies; kind
pairs carry 40-70 of matchup), so each run reports the fit's residual SD and its largest pair deviations: within a
ply a ranking, across plies a map with known folds.

**(2) The Elo-filtered multi-kind experiment** (the first global league as an experiment; analyst 6.85):
- Rule: a game enters line L's training set only if every player in it is rated at or above R_L - 50 on the COLD
  (T 0.02) ratings (50 = ~1.5 SE of a ~100-game cold set). The top line trains on its own and near-equals' games,
  topped up to the standard with self-games scaled per line by its kept share (logged).
- Ply 1; update 8's six (F-L, F-P, T-P-Ne-grow bar on, T-P-Ne capped, T-L-Ne capped forget 10, optional T-L-Ne-grow
  forget 10; a router-split DAG seat if grow-dag-p1 passes); the user's 11:08 end rule (no retirement; plateau stop on
  the scaled window, crowning bar 2.5 SE on the cold set, a cap as backstop); per-cycle history kept only inside the
  scaled window.
- Insight sought: the field's dynamics under upward-only data flow (flats train mostly on the trees' games, trees on
  tree-only games): do weaker kinds catch up from their betters, is the ranking stable, what co-adaptation does under
  a filter. Predictions vs an unfiltered mixed twin (standing ply-1 panel): flats 10-30 better, trees 0-40 better,
  ranking unchanged (trees over flats 60-110 against the mixed panel, 40-70 of it matchup); ply 2 within 30.
- WAITS for the cold-games twins' verdict (on T 0.1 ratings it is the biased filter already run: the reference
  window in diversity-p2c/p2d, no detectable effect).

**Builds:** arena.py: per-entry ply, the pinned reference-set anchor (no random; also fixes the ciX stop), the
residual report, a ladder spec builder from the library. leagues.py: the Elo filter on each line's training set
(generalising the reference window), per-line self-games top-up with the kept share logged, the end rule with cold
crowning, history kept only inside the window.

**The ladder's anchor (the user, 11-12 ~11:55): "for the global ladder we should add the random players back in."**
The ladder's field includes random, random-1, random-2 and random-3 (plies 0-3) and random is Elo 0 (arena.py's
default anchor), not the pinned panel mean. make_ladder.py's default (--anchor random; --anchor panels keeps the
pinned design). Observers are left out of the ladder's picks unless --include'd. First run: ladder-1 (11:57, 44
players, 10 double round-robins; models/from-scratch/global11-drafts/ladder-1.sh -> ladder-1.out).

**SUPERSEDED BY THE USER (11-12 ~13:22-13:24):** "You should cut a model's own games against significantly weaker
opponents. I don't think we should use an averaged rating, instead we should just play more games per round so that
the standard error is lower." / "both players need to be at least the current model's strength - standard error based
on last round." The rule: a game (own games included) trains line L iff both players are rated at or above R_L -
SE_L, R_L the current version's cold Elo in this cycle's fit (no averaging), SE_L its standard error there; unrated
players count (the old global league's qualify rule, margin = SE). Cold games per cycle raised to share 1.0 (~1,620
in the 10-entry field, SE ~17 against ~35). Crownings at the default 2.0 SE. Whether the cold games train: asked.

**The filter's rule, refined (analyst 6.87, 11-12 ~12:33; under the user's ~11:50 autonomy):** a line's OWN games
always count, whoever the opponent (on-policy data is worth ~45, 6.86; a filter must not remove it); a game between
OTHER players enters L's set iff both players' smoothed ratings are at or above R_L - 50. Every rating the filter uses
is smoothed: the running mean of the entry's last 5 versions' cold ratings (SE ~25 a rating; a single version's rests
on ~100 cold games, ci95 ~+-115, which moved the floor by the margin each cycle). This departs from the letter of the
user's rule ("each model only trains on games from models of the same or higher elo") for a line's own games: told
the user. Both filtered-p1 and its unfiltered twin run (6.87 (B)), after the cold-games verdict.
