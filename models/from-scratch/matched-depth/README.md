# matched-depth: self-play models matched to the depth-4 lines' positions

Made 2026-11-10 for the arena `matched-depth` (leagues/arenas/matched-depth/, spec
leagues/specs/arenas/matched-depth.json): how do the depth-4 self-play models do against flat LR and probit
models of the other self-play depths trained on about as many positions?

Targets (the selfplay-depth experiment's final versions, both at cycle 99):

| target | model | rounds | positions |
|---|---|---|---|
| F-L+depth=4.sp@99 | runs/F-L+depth=4.sp/lr99.model = sp/lr836.model | 836 | 1,649,603 |
| F-P+depth=4.sp@99 | runs/F-P+depth=4.sp/probit99.model = sp/probit368.model | 368 | 788,565 |

The files here are not new training. They are copies of the selfplay-depth lines' own round models
(leagues/experiments/selfplay-depth/runs/<line>/sp/<kind><k>.model). That directory keeps every round's model,
and model k is trained on rounds 0..k-1. So each file is exactly the experiment's early round: identical by md5
to its source (models.csv), with nothing to replay.

| file | rounds | games | positions | of target |
|---|---|---|---|---|
| F-L+depth=0.sp.round@5.lr.model | 5 | 5,000 | 1,511,085 | 0.916 |
| F-L+depth=0.sp.round@6.lr.model | 6 | 6,000 | 1,798,531 | 1.090 |
| F-L+depth=1.sp.round@14.lr.model | 14 | 10,500 | 1,682,882 | 1.020 |
| F-P+depth=0.sp.round@2.probit.model | 2 | 2,000 | 635,717 | 0.806 |
| F-P+depth=0.sp.round@3.probit.model | 3 | 3,000 | 910,791 | 1.155 |
| F-P+depth=1.sp.round@7.probit.model | 7 | 5,250 | 791,676 | 1.004 |

A depth-0 round is 1,000 games (about 290k positions), so no depth-0 round lands near either target. The rounds
on each side are both here. Depth 1 lands within 2%. The depth-2 matches are the experiment's own versions,
F-L+depth=2#1.sp@2 (1,613,734), F-L+depth=2#2.sp@2 (1,601,001) and F-P+depth=2.sp@2 (852,185), so the arena
takes them from the experiment.

Self-play settings of the source lines, all with chessai-hybrid4gfx: `self-play --model lr|probit --use-probs
--seed 21 --vs-previous --no-games --depth D`, with 1,000 games a round at depth 0 and 750 at depth 1.
Calibration as in the experiment: held-out games
games/heldout-lr-d2-s555/round0 for both kinds, and own games (600 at depth 1, log-odds softmax T 0.1, seed 5).
