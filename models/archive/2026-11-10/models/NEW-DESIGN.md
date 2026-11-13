# New league design (discussion started 2026-11-09 ~11:45)

Status: design only. Nothing here changes the live league (cycles/champ/, loop pid 919919). The user, 11-09 ~13:25:
"I don't mean from the current league but in the new design".

## The user's aims
- Get the most out of the compute. Generating games is the most expensive part.
- The questions are separate:
  - A: which model gets strongest with self-play;
  - B: which gets strongest given the same dataset;
  - C: whether higher-quality data is worth it, and from what point (the d2jit lines);
  - D: which models generate the best training data, possibly pair by pair.
- Patience ignoring the training a lineage gets from tournament games was an oversight; it slowed the league.
- Offline games are too biased to give good results.

## Decided by the user
1. The global league provides two things only: the practical-strength ranking and high-quality games. Everything else
   is answered in other environments. (~11:50)
2. No gating ("get rid of all gating"). (~13:25)
3. The global league has no self-play and no gating: its players train on league games only. (~13:30)
4. Small experiment leagues, spun up as needed, are where models are tried before the global league. (~13:35) The
   user renamed them from "incubators" to "experiments" (~14:20).
5. Structure of the global league (~13:50), similar to the current one:
   - Ply 2: each model's current version and its champion. The champion rules stay as in the current league.
   - Ply 1: models are used to freeze 100-point intervals. The first ply-1 model to break an interval has a frozen copy
     brought in and locked in as the interval's representative. Ply 1 keeps the frozen models cheap.
   - If a frozen model drops below or rises above its interval "long enough" (the duration is mine to set), the
     interval becomes available. The next ply-1 model to reach the threshold takes its place, and the old frozen model
     is removed from the league. The new holder's current version is frozen (like a champion), and the model keeps
     competing as it updates.
   - Training: at the end of every cycle, all models train on the same set of games. The set is every game in which at
     least one model played at ply 2; random-2 doesn't count. Then the ratings are recomputed, and the cycle repeats.
   - Versions are G.<champion>.<cycle>, plus F for a frozen ply-1 anchor.
     - The major (champion) number increments when a new champion is crowned.
     - The cycle number increments every cycle and never resets.
   - New models enter the global league fully untrained.
6. The anchor duration, set by me as asked: an anchor gives up its interval after 5 straight cycles more than 25 Elo
   outside it.
   - The margin is the part that matters. A one-cycle rating is about ±19 at 95%, so the first entry to cross a line
     was usually flattered by noise that cycle. A fresh anchor likely settles 10-20 below its line (estimate).
   - Real drift is small: unchanged ply-2 entries moved about 6 Elo over cycles 30-60.
7. No global tournaments. (~14:20)
8. Cycles stay at 5 double round-robins, as today ("or even more if you'd like"). The user dropped one round-robin
   a cycle as too much overhead (~14:40). So the champion test stays as today: the current version alone, at 2 SE.
9. Nothing launches yet: the experiments get designed first ("in some ways more important"). (~14:20)
10. An experiment is essentially a league, and experiments replace today's "offline" games (head-to-head tests,
    twins, panels). (~14:55) The user has more ideas for experiments, to come.

## My readings and defaults (told to the user ~14:05; the user may correct them)
- Anchor lock-in: the first current ply-1 entry whose rating lands in an empty interval; ties go to the higher rating.
  - A model that jumps several intervals in one cycle fills only the one it lands in.
  - Lower intervals are filled by later models climbing from untrained.
- A game counts for training if at least one side is a model (not a random player) at ply 2. That is about 84% of a
  field built like today's. Each model trains with its own recipe (labels, rates, splits), in a fixed game order, so
  runs stay repeatable.
- Versions:
  - a new model enters as G.0.0, which is also its first champion;
  - a crowned version takes the new major (G.0.5 becomes G.1.5), and later versions keep it (G.1.6, ...);
  - the cycle number alone identifies a version, so records key on it.
- Experiments: models enter the global league untrained, so an experiment graduates a recipe, not a trained model.
- Patience: 20 cycles without a new champion. Today:
  - the gaps between a lineage's champions have median 3 and maximum 34, and 94% are within 20 (n=366);
  - the 1,920 CPU s of patience comes to 32-38 cycles.
- Finished lineages retire. The ply-1 anchors replace today's ply-2 freezes at milestones. There are no revivals,
  since there are no global tournaments.
- Calibration stays own: 600 games of each version against itself at ply 1, about 2 CPU s a version, never trained on.
  Fitting the calibration on the shared games instead needs a check first: calibrating on the wrong positions has
  biased ratings before (league-calibration-confound).
- References: random, random-1, random-2 and random-3 stay (random pins 0). ref-d4mix43-1 goes; the anchors cover it.

## Cycle length (settled ~14:40)
- 5 double round-robins a cycle, as today. A version's rating is about ±19 at 95%, and a promotion needs a measured
  lead of about 25-30 Elo (2 SE of the difference).
- My pick is to stay at 5. More rounds would sharpen each version's rating a little, but every training step would
  wait longer, and at 5 the per-cycle overheads (fit, calibration, loading models) are already small.
- The rest stays as before:
  - patience 20 cycles;
  - an anchor gives up its interval after 5 straight cycles more than 25 Elo outside it;
  - anchor lock-in uses the version's own ply-1 rating;
  - calibration costs about 2 CPU s a version, roughly 1.5% of a cycle with 54 models and 4% with 20.
- Withdrawn with the one-round-robin idea: the champion test on the mean of the last 5 versions, and the durations
  scaled by 5.

## Calibration (explained to the user ~14:30)
- League play is a softmax at T 0.1 over each move's log-odds. Models differ in how confident their numbers are: an
  overconfident model plays cold and an underconfident one plays hot, and colder play won at T 0.1 for every model
  tested.
- The effect was about 132 Elo per unit of ln(slope ratio): 90-113 Elo between some families
  (league-calibration-confound).
- Calibration fits one slope per version on 600 of its own ply-1 games, so that its percentages come true on its own
  play. A held-out set from other games was off by factors of 0.55-2.03.
- Ways to drop those games, all untested:
  - fit on the version's own league games (but it needs a calibration before it has played);
  - a scale-free policy that picks by rank.

## Data on hand
- The live league deletes each cycle's game pool after its own steps (own_games.keep_pools false). Only results are
  kept (games.csv). 13.4M games have been played in cycles 1-65.
- One cycle's pool is about 4 GB; the disk has 654 GB free.
- The user, ~14:45: "we don't need to keep the league's games it's ok". Then, ~14:50: "maybe we could keep just the
  league's latest batch of games" (tentative).
  - Design: each cycle's batch (the games everyone trains on) stays on disk until the next cycle's batch replaces it.
    That is one batch, about 2-4 GB.
  - Use (my proposal): "shadow" recipes train on each batch as it arrives without playing in the global league, and
    are rated in their own small league against anchors.
    - They see nearly the data an untrained entrant would (the entrant's own games are a few % of the batch).
    - So they preview global-league results and answer question B on the league's own data. They could be the
      graduation test.
    - Cost a cycle: training (~50 CPU s per average recipe at 54 models) plus a few hundred rating games, against the
      5,000-7,000 games a cycle that a global-league seat adds.
  - Live league: no. The user, ~15:00: "we don't need the current league's games". The live driver is unchanged.
    Keeping the latest batch applies to the new design only.

## Experiments (the user's spec, ~15:10)
- An experiment is a small league that runs for a set number of cycles. Nobody enters halfway.
- To create one, pick a set of models and at least one of three options. Each model runs under every chosen option.
  Each (model, option) pair has three entries: a ply-1 competitor, a ply-2 competitor and a ply-2 champion.
  - Option 1, self-play: a solo version that trains every cycle on its own self-play.
  - Option 2, local training: trains every cycle on all the games of that cycle's experiment tournament.
  - Option 3, global training (uncommon): trains every cycle on the latest global-league games.
- Fairness: in every cycle, each version trains on roughly the same CPU's worth of games.
- Each cycle:
  1. If global training is on, check for a new global batch. Copy the fewest full rounds whose generation CPU covers 5
     or more experiment rounds, with their CPU. Set the number of experiment rounds to match that CPU. If global
     training is off, play 5 rounds.
  2. From the rounds and the competitors, set the self-play CPU for each self-play version.
  3. Play the rounds: new Elo, plus the local-training data.
  4. Train:
     - the global versions on the copied rounds;
     - the local versions on the local games;
     - the self-play versions on their own scheduled self-play.
  5. Repeat.
- Terms: "round" means one double round-robin. "Train every round" in the spec reads as every cycle.

## Experiment decisions (the user, ~15:20: "all of that sounds right but i would prefer to not sample let's just take a full round")
- Global option: copy full rounds, no sampling. That is the fewest full rounds covering 5 or more experiment rounds.
- Local and global versions train only on games in which a model plays at ply 2 (the global league's rule).
- Accepted:
  - all versions start untrained;
  - the champion test is the global league's;
  - no patience;
  - the four random players are the references;
  - calibration is per version.
- Accepted: self-play runs after the tournament. Its budget is the measured CPU of the games the local versions train
  on, counting generation only, not training. An experiment ahead of the global league waits and never reuses a
  round. The global league records each round's CPU.
- Build note: to price subsets exactly (the ply-2 games of a round), both leagues should record CPU per game.
- Until the global league runs, experiments can use only self-play and local training.

## My readings, consequences and questions (told to the user ~15:15)
- Self-play is the costly option. Each self-play version gets as much CPU as the whole experiment tournament, so 4
  self-play models cost 4 times the tournament.
- A full global round is a big unit, because games per round grow with the square of the entries.
  - Example: a 54-recipe global league has about 180 entries, and a 4-model x 3-option experiment has 40. One global
    round is then about 20 experiment rounds.
  - So the global option forces about 20 rounds a cycle. A run of a set number of cycles then costs about 8 times as
    much as the same experiment without that option (each cycle carries about 8 times the data).
  - Q1: full rounds, or a random sample of a round's games? A sample keeps the experiment near 5 rounds. My pick: the
    sample.
- Q2: do local and global versions train on all games, or only on games with a model at ply 2 (the global league's
  rule)? My pick: the ply-2 rule for both, so the options differ only in where the games come from.
- Defaults:
  - all versions start untrained;
  - the champion test is the global league's;
  - no patience, since the experiment has a fixed length;
  - the four random players are the references;
  - calibration is per version, on its own games.
- Self-play runs after the tournament, so its budget can be the tournament's measured CPU instead of an estimate.
  It counts game generation only, not training, to match the other options.
- If an experiment runs ahead of the global league, it waits for new rounds and never reuses one.
- The global league must record each round's CPU.
- Precision note (not raised with the user): in a 28-entry experiment, 5 rounds give each entry about 270 games a
  cycle, about ±55 at 95%. Conclusions have to pool versions over cycles.

## Costs (estimates for mature models)
- Basis:
  - today's 30 ms a tournament game (cycle 51 measured) and 85 positions a game;
  - own-step training costs per position;
  - each model has 3 entries, plus about 15 anchors and 4 random players.
- Correction to the ~13:40 message: about 6,100 of the 12,700 CPU s quoted for "every game" came from F-L-d2mix-c and
  F-L-d4mix-c. They label with their own depth-2 search, at about 150 µs a position, 30 times a plain LR. Without
  them the mean is 4.6 µs a position per model.
- Once self-play settings are dropped (--depth, --policy, --temperature, --vs-previous, --no-games, steps), today's 70
  active lineages are 54 distinct training recipes. For example, 11 F-L variants become one plain LR, and 4 F-P-tau0
  variants become one.

| models | games a cycle | CPU s a cycle (without the search-label recipe) |
|---|---|---|
| 20 | ~31,000 | ~1,100 |
| 40 | ~96,000 | ~4,100 |
| 54 | ~163,000 | ~7,700 (~9,400 with it) |
| today's league | 245,000 | ~12,000 |

Training grows with the cube of the number of models; the tournament grows with the square.

## Open
- Experiments: the user's spec is above (~15:10). Still open:
  - anchors shared with the global league (for comparing scales);
  - what graduates.
  Each experiment needs:
  - its question and its one factor;
  - its data rule;
  - its field, with anchors for a shared Elo scale;
  - its budget and stop;
  - what it graduates.
- Launch population: deferred, nothing launches yet.
- The CPU split between the global league and the experiments.
- Recipe names: today's names encode self-play settings such as d2jit.
- Early cycles: games between untrained models may be long; not priced.
- The other rules up for review (11-09 ~13:15 list):
  - self-play-only data and zero knowledge;
  - search as a last resort;
  - hard routing;
  - error routing as the direction;
  - no new error-router merges;
  - no NB leaves or experts;
  - no purely tournament-trained lineages (overturned for the global league by 3);
  - never cut lineages;
  - one factor per experiment, with the round robin as the judge;
  - about 60 active lineages.

## Build (started 2026-11-09 ~15:05 (leagues/ created 15:10), on the user's go: "Have agents build this experiment league and new tournament league")
- Engine fork: the driver for both league kinds, under /home/iv/dev/chessai/leagues/.
  - It writes leagues/FORMATS.md first, then BUILD.md (progress notes).
  - Smoke runs go on disk in /home/iv/dev/chessai/leagues-smoke/.
  - It doesn't launch any real league.
- Dashboard fork: leagues/dashboard.py and leagues/DASHBOARD.md, on port 8766 for development.
  - Pages: a home page listing every league, the global league, and each experiment.
  - The live dashboard (8765) is untouched; switching over is the user's call.
- The live league is untouched by both.

## After the build (the user, ~16:10)
- Dashboard:
  - old runs get just a link to the live dashboard, not a mounted archive;
  - Standings show the last completed cycle;
  - CPU totals keep the rating fit, in its own column;
  - it stays on 8766 until the user decides.
- Budgets use a fixed price formula instead of measured CPU ("we should price number of games with a fixed formula").
  So experiments replay exactly, like the global league. Measured CPU is still recorded, for information.
- Experiment versions are E.<S|L|G>.<champion>.<cycle>, with S self-play, L local, G global; for example E.S.2.17.
- Defaults accepted without objection:
  - anchors start at 100;
  - an experiment's last cycle trains nothing;
  - calibration games are played with the held-out calibration;
  - the global option carries its data mismatch like self-play;
  - leagues-smoke is kept until launch.
- Sent to the engine and dashboard agents (~16:10). Still open:
  - experiments sharing the global league's anchors;
  - the CPU split;
  - Experiment 1's model list;
  - a free budget for self-play-only experiments.
- Free budget (the user, ~16:25: "let's go 120 seconds per cycle"). An experiment with only self-play lines gives each
  line a fixed self-play budget of 120 priced CPU s per cycle (a spec field). Experiments with local or global lines
  keep the tie. Sent to the engine agent.
  - What 120 s buys, from mature live-league F-L self-play (CPU per game including its training):
    - depth 0 (F-L-c): about 1.8 ms a game, so about 67,000-73,000 games;
    - depth 1: about 2.0 ms, about 60,000;
    - depth 2: about 61 ms, about 2,000;
    - depth 4: about 2.0-2.6 s, about 50.
  - Early cycles give fewer games, because untrained models play longer games.
- Speed-ups before launch (the user, ~16:35: "Yeah we should update the binaries with optimizations. Also, we should
  take a look at some optimizations we could do for the base models").
  - Fast-binaries agent: rebuilds the 15 recipe binaries with hybrid4's exact fast paths under new names, with
    byte-identical games and models. Outputs: leagues/binaries-fast.txt and models/from-scratch/fastbin/summary.txt.
  - Model-optimization agent:
    - profiles each model kind;
    - writes a costed list of speed-ups, each marked exact or not;
    - prototypes fixed-point incremental sums on flat LR;
    - output: models/from-scratch/modelopt/summary.txt.
    Nothing is deployed; the user chooses.
  - Once the fast binaries are ready, the engine agent switches recipes.txt to them and recalibrates the price
    formula.

## Initial experiments: brainstorm (2026-11-09 ~16:30)
- The user: "start from scratch with regard to the fundamental leaf classifiers, as this will give us structured data
  potentially along various parameters". That means LR, probit and NB (CNBP) first. Routers and trees on the winning
  leaves come later (stage 2), then mixtures.
- Settings available per classifier (hybrid4 self-play --help):
  - LR: --lr-rate (0.05), --round-forget and --age-forget;
  - probit: --probit-tau (1e-5), -prior (0.3), -beta (1), -alpha and -per-game, -eps (0.5);
  - CNBP: --alpha (Laplace, 1) and forgetting;
  - all: --depth, --policy and --temperature, --games, --label-depth and --label-result.
- Proposed stage-1 slate. Costs are rough: 25 ms a tournament game, 120 s free budget, 100 cycles. Each experiment runs
  its default line twice, with different seeds.

  | # | experiment | lines | options | CPU s a cycle | CPU-h |
  |---|---|---|---|---|---|
  | 1 | leaf type x data source: LR, probit tau 0, NB | 7 | self-play + local | ~360 | ~10 |
  | 2 | data quality: LR, probit, NB x self-play depth 0/1/2/4 | 13 | self-play | ~1,800 | ~50 |
  | 3 | LR step size 0.0125 / 0.025 / 0.05 / 0.1 | 5 | self-play | ~650 | ~18 |
  | 4 | probit tau 0 / 1e-6 / 1e-5 / 1e-4 | 5 | self-play | ~650 | ~18 |
  | 5 | NB smoothing 0.1 / 1 / 10, plus forgetting | 5 | self-play | ~650 | ~18 |
  | 6 | LR exploration: jitter 10/20, softmax T 0.1/0.3 | 5 | self-play | ~650 | ~18 |
  | 7 | LR labels: results vs 50/50 search labels | 3 | self-play | ~400 | ~11 |

- Needs:
  - variants, sweeps and replicates in experiment specs (sent to the engine agent ~16:30);
  - for experiment 7, label-search CPU counted as generation (asked the user).
- Questions asked:
  - Are these the right parameters? Should FM and TAN join?
  - Run 1-2 now with defaults, or sweep first (3-5)?
  - What happens to the live league while experiments run?
- The user (~16:40): "for a lot of these we can go for local rather self-play". Revised slate (proposed):

  | # | experiment | lines | options | CPU s a cycle | cycles | CPU-h |
  |---|---|---|---|---|---|---|
  | 1 | leaf settings in one league: LR step 0.0125/0.025/0.05/0.1 + search labels; probit tau 0/1e-6/1e-5/1e-4; NB smoothing 0.1/1/10 + forgetting | 13 | local | ~350 | 200 | ~20 |
  | 2 | leaf type x data source: LR, probit tau 0, NB (+ a seed copy of LR self-play) | 7 | self-play + local | ~360 | 100 | ~10 |
  | 3 | data quality: LR, probit, NB x self-play depth 0/1/2/4 (+ a copy) | 13 | self-play | ~1,800 | 100 | ~50 |

  - The self-play exploration policy is deferred.
  - Self-play depth and policy must stay self-play, because their factor is how the self-play games are made.
  - In local lines, a replicate only reshuffles the same positions (train --seed shuffles each round's positions,
    rs/src/main.rs:1586 and 1596). LR and probit come out slightly different (order noise). NB is pure counting
    (cnbp.rs:152), so an NB replicate is the same model exactly. The noise floor comes from the error bars, the shape of
    each sweep curve, or a rerun with another tournament seed.
  - Total ~80 CPU-h (was ~140). Experiment 1 also compares LR, probit and NB on the same data; experiment 2 keeps the
    self-play vs league-games question.
  - Search labels in local mode cost training time only (about 100 s a cycle); no budget-fairness issue.
- The user (16:48): "We can keep FM and TAN out for now." Stage 1 is LR, probit and NB only.
- The user (16:48) asked why experiment 2 has both self-play and local. My answer:
  - It's the only head-to-head of the two data sources: same leaves, equal CPU, one field (Q3, the league-data
    confound). Its local half repeats experiment 1; the new part is the contrast.
  - The self-play side depends on depth. An S line's budget is the price of the local games, ~60 s a cycle in a 7-line
    field: ~40,000 games at depth 0 or ~1,800 at depth 2 (fast binaries), against ~2,500 local games. The recipes
    differ too: F-L and F-P-tau0 default to depth 0, while NB exists only at depth 2 (F-N-d2jit).
  - Proposed: run experiment 2 after experiment 3, with each leaf's best self-play depth from 3, or drop it (~10 CPU-h).
- The user (17:03): "We should definitely run experiment 3 first. Probably alongside 1." First batch: experiments 1 and 3
  together. Experiment 2 (self-play at each leaf's best depth from 3, plus local) comes after 3.
- The user (17:03): "also run a depth 0 Leaves only at default setting with local and self-play. That we could compare
  effect of self-play to cross-pollination of games for leaves." Experiment 4:
  - LR, probit and NB at default settings (LR rate 0.05; probit tau 1e-5; NB alpha 1), self-play at depth 0 with the
    default policy, options self-play + local, plus a second-seed copy of the LR self-play line: 7 lines, ~350 CPU s a
    cycle, ~10 CPU-h over 100 cycles. It doesn't depend on 3, so proposed to start with 1 and 3 (~80 CPU-h together).
  - The three recipes are the live league's default flat leaves, all at depth 0: F-L-c (active), F-P-c (probit, default
    tau 1e-5, status frozen) and F-N-c (NB, retired by finish at cycle 57). The user caught that F-N-c is depth 0; I
    had said NB existed only at depth 2, because leagues/recipes.txt only takes lineages active at cycle 64
    (make_recipes.py SNAPSHOT), so F-N-c and F-P-c are missing from it. Asked the engine agent to add them (~17:05); added 17:11 as rows F-N and F-P (make_recipes.py EXTRA list, other
    rows unchanged), checked end to end in a 2-cycle mini experiment 4 (40 CPU s).
    Tau 0 did better in the live league; experiment 1's sweep covers both.
  - Caveat: at equal CPU (~60 s a cycle per self-play line) an LR line at depth 0 gets ~40,000 one-ply games, while
    the local lines get ~2,500 games that each have a 2-ply side. The contrast mixes cross-pollination with search
    depth and volume. Experiment 2 gives the matched one: at depth 2 a self-play line gets ~1,800 games a cycle. If 3
    finds depth 0 best, run 2 at depth 2 anyway.
  - The engine doesn't merge identical lines: F-L and F-L-d2jit have the same train args, so in one experiment with the
    local option they'd make the same local line twice (same seed). Avoid listing two depths of one leaf with local.
- The user (17:08): "test different leaf settings independently of each other. I don't think we want them to learn on
  games of other leaf models." Experiment 1 is split into one local league per leaf family:

  | # | experiment | lines | local games a cycle | CPU s a cycle | CPU-h, 500 cycles |
  |---|---|---|---|---|---|
  | 1a | LR: step 0.0125 / 0.025 / 0.05 / 0.1, plus search labels at 0.05 | 5 | ~1,350 | ~67 | ~9 |
  | 1b | probit: tau 0 / 1e-6 / 1e-5 / 1e-4 | 4 | ~920 | ~36 | ~5 |
  | 1c | NB: alpha 0.1 / 1 / 10, plus forgetting at 1 | 4 | ~920 | ~36 | ~5 |

  - Within a family, settings still learn from each other's games (that is local); fully separate needs self-play.
  - Data per line per cycle falls 6-8x against the combined league (~7,700 games); cost falls more (n^2 tournament):
    ~140 CPU s a cycle for the three against ~350. Proposed 500 cycles: ~19 CPU-h, each line 460k-675k games.
  - Families aren't comparable with each other (different fields and games). Experiments 2 and 4 mix leaves on
    purpose: cross-pollination between leaves is their factor.
  - First batch: 1a-1c, 3 and 4 (~80 CPU-h). Experiment 2 after 3.
- Fast binaries done (~17:15, models/from-scratch/fastbin/summary.txt, leagues/binaries-fast.txt): the 15 recipe
  binaries become 5 (chessai-hybrid4g for 57 rows; dag-q9, presplit-q9, bal-q9, bddtrig2-q9 for the side branches),
  byte-identical in every check (self-play, training, version 0, arenas, forced tree splits). New CPU / old: depth 0
  flat 0.74-0.87, depth 1 0.53-0.62, depth 2 flat 0.45-0.59, trees 0.54-0.76, depth 4 0.62-0.64, training unchanged,
  depth-2 label search 0.58. My spot check (17:17): F-N d0, F-L d2, F-P d0, 3 rounds of 40 games from scratch,
  identical; F-L d2 5.4 s against 10.0 s. Engine told to switch recipes, re-run the replay test and write P1 (17:18).
- Launch prep (17:34):
  - The user asked whether we're ready to shut down the old league and start the experiments. Touching
    cycles/champ/STOP was denied by the permission classifier ("interfere with workloads"); the user runs it
    (! touch /home/iv/dev/chessai/cycles/champ/STOP; the loop stops after its current cycle).
  - The user's "great ok" taken as a yes to the fixed-point binary before launch: the fast-binaries agent builds
    chessai-hybrid4gfx (hybrid4g + fxlr2's changes) and runs the identity suite on its 57 rows; then the engine switches
    recipes and calibrates prices on it.
  - 1c drops forgetting: train has no --round-forget/--age-forget (self-play only). 1c = NB alpha 0.1 / 1 / 10.
  - Cleanup: dashboard 8766 restarted on the real root (leagues/, "No leagues yet"), pid 1206555; leagues-smoke (364 MB)
    to be deleted after the engine's P1 run there.
  - The user (17:33): T-L-Ne-share3-c reached 1889.9 (cycle 71, v394, champion). A fork builds a leaf view
    (models/from-scratch/leafview/share3.html).
- Engine (17:46): recipes switched to the fast binaries (replay identical: games, ratings, 128 model files). P1 prices written
  (leagues/prices/P1.md; default for new leagues). Open: trees need a model-size term (bdt/bdd priced at 0.50-2.14 of
  their cost by kind alone; proposed log2(model bytes / 20 KB) per-move term; needs sizes recorded, a format change);
  self-play mispriced by maturity (mature F-L d2 at 0.60, trees from scratch ~2). P2 comes with the fixed-point binary.
- Cleanup (17:46): leagues-smoke's test leagues deleted (~347 MB); kept pricecal-P0, pricecal-P1, their measure logs 1-2 and
  equalisation.py (17 MB). pricecal-P1.measure3.log was deleted by mistake (pattern missed it).
- The user (17:54): "I'll let the live league run for a little longer. Let's scale the self-play compute times per cycle
  down by a factor of 2 for our planned experiments."
  - Self-play-only experiments (3): selfplay_budget_s 60 (was 120), set in the specs.
  - Self-play + local experiments (4, later 2) keep the equal-CPU rule (each self-play line gets the local games' price);
    halving there would make a 1:2 comparison. Experiment 4's self-play is small (~4 CPU-h at 100 cycles).
  - Re-costed with P1 prices (flat-only tournament games ~11 ms, not the 25 ms assumed): 1a ~4.7, 1b ~1.8, 1c ~1.3 CPU-h
    (500 cycles); 3 ~25 (was ~46 at 120 s); 4 ~5. First batch ~37 CPU-h, before the fixed-point binary.
  - P1 prices: 60 s buys F-L ~45 rounds of 1,000 games at depth 0, ~75 rounds of 40 at depth 2, a few at depth 4
    (mature depth-2 runs cost ~1.7x their price until P2).
  - Tree size term: on hold until the user says yes (no effect on stage 1). P2 calibrates self-play on mature models.
- Fixed-point binary ready (18:05): chessai-hybrid4gfx (md5 a3e6d4e6...), 465/465 identity checks byte-identical against
  hybrid4g (incl. calibrated arenas and a 27-player tournament with the league's flags); my spot check identical.
  New CPU / old: LR self-play d0 0.74, d2 0.39; NB d0 0.67, d2 0.36; probit self-play ~1.0 (gains only in calibrated
  play, d2 0.70); label training 0.29; trees/mixtures self-play unchanged, calibrated d2 0.93-0.97; F-L-d1jit 1.07 (its
  depth-1 jitter policy needs float predictions too). Engine told to switch, replay, calibrate P2 (mature self-play)
  and draft the five specs in a scratch root.
- Engine (18:47): batch 1 ready to launch.
  - Recipes switched to chessai-hybrid4gfx (57 rows; the 8 side-branch rows keep their q9 binaries). Replay identical
    against a hybrid4g reference (25 global and 201 experiment items), 30 s of CPU against 53 s.
  - P2 prices (leagues/prices/P2.md) are the default: the same 7992 tournament games as P1, self-play fitted on mature
    runs. P2/P1 tournament CPU 0.99 (ply 1), 0.93 (ply 2). From scratch, priced/measured self-play by depth 0/1/2/4:
    lr 1.10/0.74/1.17/1.31, probit 1.12/1.03/1.43/1.52, cnbp 0.92/0.74/0.90/1.10. So early in experiment 3, against
    depth 0, depth-1 lines get up to ~1.5x the real CPU and deep probit lines ~a quarter less. Fades as lines mature;
    measured CPU is recorded beside priced (cycles.csv), so the depth comparison can be checked at real CPU.
    Trees from scratch priced 2.35 (bdt) and 4.99 (bdd) of measured: the size term; no effect on batch 1 (flat only).
  - Specs: leagues-smoke/batch1/specs/ (root leagues/; `leagues.py create <spec>`), each run 2 cycles without errors in
    leagues-smoke/batch1/mini. CPU-h at cycle-2 rates: 1a 4.0, 1b 3.7, 1c 2.0 (500 cycles x 5 rounds; 100 x 25:
    3.5/3.2/1.6), 3 25-28 (13 lines, 60 priced s, 100 cycles), 4 5.3 (8 lines, 100 cycles). ~40 CPU-h, about 5 h of
    the 8-core machine. Driver defaults nice 10, threads 4, jobs 2. Budgets are priced, so results don't depend on load.
  - Seeds come from the spec (league seed, cycle, replicate offset), not the root: the launched first two cycles should
    match the mini run. Compare at launch, then delete mini (385 MB).
  - My 17:46 cleanup deleted the engine's replay references mid-run; it re-ran them (110 s).
  - Open: 1a-1c length (500 x 5 recommended: retrain every 5 rounds, 500 rated versions; or 100 x 25); the user stops
    the live league; then create and launch all five. Unchanged unless the user objects: experiment 4 equal CPU, size
    term waits for tree experiments.
- 1a-1c length (18:55): stays 500 cycles x 5 rounds, as the specs have it. The user asked what was needed; nothing is,
  unless they prefer 100 x 25. Remaining before launch: the user stops the live league.
- Repo committed and pushed (19:30; the user asked): 3c1b42f Rust (rs/src, compiles clean), b8dcc32 cycle system (code,
  configs, cycle reports, cyc5 source tarballs), e0d3f24 new league engine (code, docs, prices P0-P2, recipes) with the
  batch-1 specs copied to leagues/specs/batch1/ (launch from there: leagues.py create leagues/specs/batch1/<spec>.json).
  .gitignore keeps league data out. Not in git (models/** ignored): models/*.md (236 KB), analysis scripts (5.3 MB),
  38 binary source dirs (29.2 MB, incl. hybrid4gfx-src 870 KB, which batch 1 runs on); offered to the user.
- Hoeffding trees closed (19:34). The user: "I think that they're a bit too deterministic for a game like chess." No HT
  lines in new-league experiments (T-L-H and T-N-H stay in recipes.txt as catalogue rows only). The live T-L-H-c and
  T-N-H-c keep running until the live league stops. Ledger row added (models/LEDGER.md, Trees: when to split).
- LAUNCH (20:10). The user stopped the live league (STOP; it stopped cleanly after cycle 75, the global-tournament
  cycle, at 20:06:49; no deploy pending, no league processes left). Batch 1 created from leagues/specs/batch1/ and
  started, one repeat.sh each (pids 1264295-1264299, driver defaults nice 10, threads 4, jobs 2):
  leagues/experiments/{lr-settings, probit-settings, nb-settings, selfplay-depth, leaf-sources}. Stop one with
  `python3 leagues/leagues.py stop leagues/experiments/NAME`. Launch check running: cycles 1-2 against the 2-cycle
  check run (leagues-smoke/batch1/mini): models v0-v2 md5, tournament games, cycles.csv rows without measured columns.
- Launch check (20:22, leagues-smoke/batch1/launchcheck.txt): lr-settings, probit-settings, nb-settings and
  leaf-sources are identical to the 2-cycle check run (models v0-v2, games, cycles.csv rows), though the check ran at
  nice 19, 2 threads, 1 job and the real runs at nice 10, 4 threads, 2 jobs. selfplay-depth differs as expected: its
  check used a reduced spec (4 priced s, no depth-4 lines); its 10 shared v0 models match. Check run deleted (385 MB).
- NB local stalls in its own league (20:31, the user: "the F-N test is going terribly, as somewhat expected"). nb-settings
  at cycle 92: alpha 0.1/1/10 all flat at 250-380 at ply 2 (refs random-2 241, random-3 352); champions still v1-v2.
  Same recipe in leaf-sources: F-N.loc 1,168 at cycle 17 on about the same total local games (~53k vs ~52k), F-N.sp
  1,323 (15,000 self-play games a cycle). LR bootstraps in its own league (lr-settings 1,440-1,730 at cycle 57). NB
  learns from better players' games or from self-play volume, but can't bootstrap from an NB-only field. 1c can't
  rank alphas. Proposed to the user: stop 1c (saves ~1.6 CPU-h); NB stays in experiments 3 and 4.
  - The user keeps 1c running (20:33): "maybe they'll eventually break away from the randoms".
- 1c copy without random-2 (20:48, the user: "run another copy of that experiment but without the random 2 reference",
  "don't stop this one"): leagues/experiments/nb-settings-no-random2 (spec leagues/specs/batch1/1c-nb-settings-no-random2.json,
  references random, random-1, random-3; otherwise identical: alpha 0.1/1/10, local, 500 x 5). repeat.sh pid 1284839.
  The original nb-settings keeps running. The engine has one training rule ("a model at ply 2"), so the copy can't
  keep a reference in the field while leaving its games out of training.
  - Aborted (20:50): the user meant random-3, not random-2. nb-settings-no-random2 was stopped after its cycle 2 and,
    as the user said, deleted with its spec. In its place: leagues/experiments/nb-settings-no-random3 (spec
    leagues/specs/batch1/1c-nb-settings-no-random3.json; references random, random-1, random-2). repeat.sh pid 1285826.
- The user (21:13): "For LR you chose to test 2 different things, which isn't good because local training causes a lot of
  cross-pollination." 1a mixes step size (4 rates) and search labels (1 line at rate 0.05) in one local league. The
  label line is the strongest (1,734 at cycle 57, against 1,440-1,580), and its games are in every rate line's training
  set. Checked the rest: 1b and 1c sweep one flag each; experiment 3's lines differ only in --depth (plus #2's seed) and
  train on their own games; experiment 4 mixes leaf types on purpose. Proposed: stop lr-settings and split it into two
  local experiments, the step sizes only, and F-L at 0.05 against F-L with search labels.
- Per-family experiments evened out (21:15). The user: "kill both the LR and both NB experiments and remove the labeling
  model for LR and add NB with another alpha value". Also: "NB has 1 less model, which makes it so there are relatively
  more games against random per round". Measured share of qualifying games against random references: LR 5 lines 30%
  (1,350 a cycle), probit 4 lines 35% (920), NB 3 lines 42% (570).
  - Stopped (STOP, after their current cycle, 21:15): lr-settings at cycle 170, nb-settings at cycle 262,
    nb-settings-no-random3. Their folders are kept.
  - Started: lr-settings-2 (spec 1a2: --lr-rate 0.0125/0.025/0.05/0.1, no label line; pid 1299641) and nb-settings-2
    (spec 1c2: --alpha 0.1/1/10/100, 100 added as the next log step; pid 1299640). Local, 500 x 5, default references.
  - All three per-family experiments now have 4 lines and the same 4 references: 16 players, ~35% of qualifying
    games against random, the same field shape. The search-label test is out. It could return as its own 4-line
    sweep (label weight).
  - The user on look-ahead (search) labels: "obviously it's a good technique but it's basically equivalent to
    self-play with n+1 depth from given position. It also isn't fair wrt to CPU usage." Measured in lr-settings: the
    label line's training took a median 3.57 s a cycle, against 0.28-0.29 s for the plain lines (12x). That extra ~3.3
    s a cycle was unpriced: experiments price games only, not training. If labels come back, price the label search
    like self-play (charged to the line's budget) and compare them with deeper self-play at equal CPU.
  - Correction: the 16:4x note "search labels in local mode cost training time only; no budget-fairness issue" was
    wrong. Training CPU is unpriced, so a label line gets free compute (12x a plain line's training CPU in lr-settings).
- The user asked why leaf-sources has two LR lines but one probit and one NB. The plan (17:03) was 7 lines: one line per
  leaf and source, plus a second-seed copy of the LR self-play line only, to measure noise. The spec replicated F-L
  under both options, so 8 lines run, including a local LR replicate, which only reshuffles the same games. Side effect:
  4 of 8 lines are LR, so the local lines' training games are LR-heavy (the field-shape issue the user raised for NB).
  The LR copies agree within ~15 (local 1,699/1,702, self-play 1,607/1,622 at cycle 36).
  - The user (21:34): "Just start another 1 with only 1 line per leaf." leaf-sources-2 (spec
    leagues/specs/batch1/4b-leaf-sources-2.json: F-L, F-P, F-N, each under self-play and local, no replicate; 6 lines,
    100 cycles), repeat.sh pid 1309055. The original leaf-sources (8 lines, cycle 68 of 100) keeps running.
- The user (21:39), on the extra lines: "you inserted additional models into experiments which kind of upset the
  balance of the models doing local-training." Rule for future batches: an experiment has only the lines the user
  asked for. Replicates and extra conditions are proposed separately and, if wanted, run as their own experiment. In a
  local experiment every line's games are every local line's training data, so one extra line shifts all of them.
  - Batch 1 had two such insertions: 1a's search-label line, whose strong games fed every rate line, and 4's LR
    replicate pair, which made the field LR-heavy. Both are rerun without them (lr-settings-2, leaf-sources-2).
  - The one other extra is selfplay-depth's second F-L depth-2 line. That experiment is self-play only, so the line
    touches the rating field but no line's training. Kept unless the user says otherwise.
  - The original leaf-sources, if it runs to 100, differs from leaf-sources-2 only by the extra LR pair. Its F-P.loc
    and F-N.loc against leaf-sources-2's then show how much one extra pair moves the local lines.
- The user (~21:45): new experiment, "an F-L and single split T-L-Xe trees with all the various base models that we
  can conceivably use for the error router", then "local-training only".
  - Routers the binaries accept: LR, NB (cnbp) and TAN. Probit is leaves-only: it is ordinal over the three results,
    and a tree's children have no order. FM isn't a router in any build, but its code already handles any number of
    classes. The user: "let's leave probit out of routing". An FM router needs a new build (fork, ~21:58).
  - Single split: --split-limit 1 (every live tree recipe passes 5; the binary's default is 20). --split-threshold
    100,000 (live 5,000,000: a local line sees ~100-220k positions a cycle, so 5M would put the first split check 50-100
    cycles in). Tested offline on held-out games: the tree splits once, in its second round, and keeps its leaves. The
    trees are identical until then (same games, same seeds), so they all split at the same moment.
  - Lines: F-L, T-L-Le, T-L-Ne, T-L-Te (tan-m 200000, as the live T-L-Te) and T-L-Fe. Local, 500 cycles x 5 rounds.
    5 lines put ~30% of qualifying games against random. All launch together once the FM binary passes its identity
    checks, and the spec sets the tournament binary to the new build, which has to load FM-router trees.
  - Open: 2 or 3 children per split. 2 was proposed so a probit router would be a plain binary probit. With probit out,
    3 matches every live tree.
  - The user (~21:58-22:00): 3 children per split. FM is out of routing too, "although we should still hook it up for
    the future": the FM-router fork goes on, for later. Then: "Let's also add T-L-Lc-merge T-L-Nc-merge T-L-Tc-merge"
    (cluster routing with merge, which the user allows for cluster routers), and "let's add more cycles to this test":
    1000 cycles.
  - Offline test (held-out rounds of ~170k positions, split threshold 100k): single-split cluster trees with merge flip.
    A merge undoes the split once it loses to the unsplit shadow; that frees the one allowed split, so the tree splits
    again at the next check. NB and TAN routers flipped every round for 6 rounds; LR merged twice, then kept its split.
    TAN at tan-m 200000 matched NB's merge gains to 3-4 digits: near-NB until its counts grow well past 200k.
  - Taken (my recommendations, not objected to): split threshold 1,000,000 for every tree line, so a split gets ~5-10
    cycles before it is judged and the error trees split around cycle 10; tan-m 200000, as the live T-L-Te.
  - New recipe rows, by hand: T-L-Lc-merge and T-L-Tc-merge are T-L-Nc-merge with --router lr, and with --router tan
    --tan-m 200000. make_recipes.py now has a HAND list, kept on a re-run (the regenerated file matched the old one
    exactly, plus these rows). The FM fork adds T-L-Fe to the same list.
  - LAUNCHED 22:01: single-split (spec leagues/specs/batch2/1-single-split.json).
    - Lines: F-L; T-L-Le, T-L-Ne and T-L-Te (error routing, no merge); T-L-Lc-merge, T-L-Nc-merge and T-L-Tc-merge
      (cluster routing, merge). Every tree has --split-limit 1 --split-threshold 1000000 and 3 children.
    - Local only, 1000 cycles x 5 rounds, repeat.sh pid 1324482.
    - From cycle 2 the field has 25 players, so 3,000 games a cycle, ~23% of qualifying games against random.
    - The engine reads config.json every cycle, so the cycle count can probably be raised before the end (untested).
- The user (22:10): "Let's start the same tree test but for probit leaves." single-split-probit (spec
  leagues/specs/batch2/2-single-split-probit.json): F-P plus T-P-Le, T-P-Ne, T-P-Te, T-P-Lc-merge, T-P-Nc-merge and
  T-P-Tc-merge. These are the single-split rows with --leaf probit; everything else is as single-split. 1000 cycles,
  launched 22:15, repeat.sh pid 1332165.
  - Offline check first (held-out rounds, threshold 100k): probit-leaf error trees split once and kept 3 leaves. The
    cluster-merge ones flipped like the LR ones. A probit-leaf tree played an arena against rand.
  - The six T-P rows are in make_recipes.py's HAND list, after the fork's T-L-Fe row.
  - single-split ran ~70 s a cycle alone (cycle 12 by 22:15). With both running the load is ~26 on 8 cores.
- I found that the cluster-merge lines split by absolute error, the binary's default: the live T-L-Nc-merge recipe
  never set --error. The error-routed lines use --error log, adopted in the ledger after a test (T-L-Ne log beat abs);
  abs vs log was never tested on cluster trees. In single-split the error trees split at cycle 12 and the cluster
  trees hadn't by cycle 15. For a cluster tree, --error changes only the split check; merges use --trial-error, the
  same for every line.
- The user (22:25): "create a new experiment with both F-L and F-P and put all the cluster trees in there a version
  with --error and a version without --error". cluster-error (spec leagues/specs/batch2/3-cluster-error.json).
  - Lines: F-L, F-P, and the six cluster-merge trees (T-L and T-P, routers L, N, T), each once without --error (abs)
    and once with --error log. 14 lines, all single split (limit 1, threshold 1M), 3 children, local, 1000 cycles.
  - From cycle 2 the field has 46 players, so ~10,350 games a cycle, ~13% of qualifying games against random.
  - Launched 22:26, repeat.sh pid 1338340. single-split and single-split-probit keep running with their abs-split
    cluster lines unless the user says otherwise.
- Dashboard (the user, ~22:20): green ▲ and red ▼ beside Elo changes: the by-model grid, Standings' Change column and
  "Against its champion", the Lines gap, and global anchors. There are no arrows on option gaps (neither direction is
  good or bad). Done by a fork; dashboard restarted (pid 1337882).
- leaf-sources finished (100 of 100) at ~22:25.
- chessai-hybrid5 is the tournament binary of the three tree experiments (2026-11-10, the hybrid5 fork; report
  models/from-scratch/fastbin/hybrid5/summary.txt). It plays calibrated trees and DAGs in exact fixed point, with the
  same games, ratings, calibration games, models and states as chessai-hybrid4gfx. Each experiment was stopped after a
  cycle (leagues.py stop); only config.json's tournament.binary changed; then repeat.sh was restarted.
  - single-split: after cycle 111; restarted 00:53:37, repeat.sh pid 1404770 (was 1324482). Cycle 112 done 00:54:24.
  - single-split-probit: after cycle 122; restarted 00:56:51, pid 1406175 (was 1332165). Cycle 123 done 00:57:47.
  - cluster-error: after cycle 39; restarted 00:58:26, pid 1407148 (was 1338340). Cycle 40 done 01:01:03.
  - The flat-only experiments keep chessai-hybrid4gfx: flat models play the same code in both, so there is no gain.
- chessai-hybrid6 is the train binary of the three tree experiments (2026-11-10, the training-speedup fork; report
  models/from-scratch/fastbin/hybrid6/summary.txt). It trains with the same models, states and output as
  chessai-hybrid4gfx at about 0.6 of the training CPU, and plays as chessai-hybrid5. Each experiment was stopped after
  a cycle (leagues.py stop); only config.json's train_binary changed, on every line; then repeat.sh was restarted.
  - single-split: after cycle 502; restarted 05:19:58, repeat.sh pid 1506075 (was 1404770). Cycle 503 done 05:20:39.
  - single-split-probit: after cycle 511; restarted 05:19:58, pid 1506083 (was 1406175). Cycle 512 done 05:20:40.
  - cluster-error: after cycle 174; restarted 05:20:41, pid 1506455 (was 1407148). Cycle 175 done 05:22:12.
  - selfplay-depth keeps chessai-hybrid4gfx: it ends ~07:00, and its self-play binary trains too.
