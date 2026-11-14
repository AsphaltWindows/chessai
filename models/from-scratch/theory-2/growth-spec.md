# Growth rule: builder specification

Analyst, 2026-11-11 ~20:15. Consolidates theory-2 notes 6.36-6.39. Self-contained: a builder needs this file and
the source (rs/src/delegation.rs, bdt.rs, bdd.rs; the errdag/hybrid tree for the router rule with path context).
Line numbers below refer to rs/src/delegation.rs unless said otherwise.

Purpose. Today a BDT leaf splits when a 1M-example window's mean error exceeds the previous window's by any amount
(SplitRule::Worse, split_z 0), until --split-limit; a BDD delegator (router) splits on a rise of --router-split-z
standard errors. Neither measures benefit; the leaf rule is a coin flip once a leaf has converged and the cap is
the only brake; the router rule is a drift detector that fires early or never. The new rule: a node becomes
eligible when it has STAGNATED (and, for routers, when everything below it has), splits are rationed at a CONSTANT
RATE per unit of training data, and a BENEFIT test decides which eligible node splits and whether the split stays.
Growth never stops by a count; it stops only when nothing earns a split.

Two phases, built and tested separately. Phase 1: leaves in BDT trees (T-P-Ne, T-L-Ne; also the leaves of BDDs,
since BDD reuses Graph::check_leaf_split). Phase 2: routers in BDD DAGs (D-x-Ne-ctx).

Identity requirement (both phases). With the new flags absent, training, model files and state files are
byte-identical to hybrid15 / hybrid6 / errdag4 on any stream. Every new field is allocated, updated or consulted
only when its rule is on. Prove it with the existing identity harness (the hybrid14 checks) on a league pool and
on the offline-ply pools before any other test.

## 0. Shared definitions

Notation: W = --split-threshold (1M in the recipes); B = --branch-factor (3); n_v = node v's lifetime examples
(num_trained_total); N = examples through the root (the stream).

0.1 Windows and blocks. Every node already keeps a window of its own examples (num_trained_last, error_last,
error_sq_last, error_prev, error_prev_se2; close_window at 571). Add, used only under the new rules, block
accumulators of the same shape as GainWindow (433-475): block length b = W / 20 (50,000 at W 1M; integer
division; the last partial block of a window is dropped), count of finished blocks K_b, sum and sum of squares of
block means. At a window close the window's statistics are
    mean_w  = block_sum / K_b
    se_w    = sqrt( max(0, block_sum_sq / K_b - mean_w^2) / (K_b - 1) )           (K_b = 20 normally)
These replace the per-example SE for every test in this spec (a game's positions are correlated; the per-example
SE understates the noise by a factor of 2-3, see the comment at 254-256). For a leaf the error is its prediction's
error by --error on the examples that score it (as node.record does today); for a delegator it is its routing
error (the chosen child's error, as branches record today).

0.2 Stagnation of one node. Keep per node, new fields: prev_mean, prev_se (the last closed window's statistics;
"none" before the first close) and stall_streak (u32). At a close with statistics (mean_w, se_w):
    if prev exists:
        improvement = prev_mean - mean_w
        bar         = Z_S * sqrt(prev_se^2 + se_w^2)           (Z_S = --grow-stall-z, default 2.0)
        stall_streak = stall_streak + 1  if improvement <= bar  else 0
    prev := (mean_w, se_w)
A node is STAGNANT iff stall_streak >= K_S (--grow-stall-windows, default 2). A fresh node therefore needs three
closed windows (3W of its own examples) before it can be stagnant. A rising error counts as not improving.

0.3 Fresh starts. A node made by a split (a leaf copy, a router copy, a shadow) copies PARAMETERS from its source
(leaf copies forgotten by --forget-factor and perturbed by --split-noise as today; router copies forgotten as
today) and starts with EMPTY window statistics: num_trained_last 0, error_last 0, error_sq_last 0, error_prev -1,
error_prev_se2 0, block accumulators zero, prev none, stall_streak 0, backoff fields zero. The split node itself
(a leaf that became a delegator, a delegator that became a router) also restarts: forget_window plus the same
resets. This is what new_node (2236) and split_delegator (bddtrig bdd.rs) do today for the old fields; extend it
to the new ones. Nothing inherits a parent's windows.

0.4 Subtree stall (eligibility of a node that has descendants; for a leaf it is plain stagnation). Define
recursively, evaluated only when needed (at a scheduling point, 0.6), O(subtree) each time:
    stalled(v) = stagnant(v) AND ( v is a leaf  OR  sum over children c of share(c) * [stalled(c)] >= S )
    share(c)   = n_c / sum over children c' of n_c'       (lifetime routing shares; 0/0 counts as not stalled)
with S = --grow-subtree-share, default 0.9. A DAG child with several parents is evaluated once per evaluation (memo
by index). The lifetime share is the chosen weighting: exact, cheap, and the small leaf that never settles is also
the one that barely moves its ancestors' routing targets.

0.5 Credits (the rate). Graph-level fields, in the state: credits (u32), next_credit_at (u64), credits_spent,
credits_refunded (u64, for the log). After every root example: while N >= next_credit_at { credits = min(credits +
1, C); next_credit_at += D }. D = --grow-credit (positions; default 5,000,000), C = --grow-credit-cap (default 2).
A credit is spent when a leaf trial starts or a router split is made; a failed leaf trial refunds one (credits =
min(credits + 1, C)). Credits only ration; waiting never consumes them. D sets the steady-state mean examples per
leaf (about D / (B - 1)): 5M gives 2.5M per leaf; 10M is the lean setting.

0.6 Scheduling point and order at a window close of node u (num_trained_last >= W). This replaces the body of
check_leaf_split (797) and check_delegator_split (bdd.rs) when the respective new rule is on:
    1. close u's window: compute (mean_w, se_w) from the blocks, update prev and stall_streak (0.2); record the
       model's and routers' running means over the window as today (window_model / window_routers).
    2. open u's new window (open_window, plus zeroed blocks).
    3. if u is on trial, nothing more here: trial_step (1286) ends the trial after W examples through u, keeping or
       undoing it by 1.3 / 2.4.
    4. SCHEDULE: if credits >= 1 and trials_in_flight < T (--grow-trials, default 2):
       a. phase 2 only: candidates R = routers v with stalled(v), not on trial, not in back-off, whose shadow gain
          clears both bars (2.3). If R is non-empty, split the one with the largest (gain - MDL bar) (ties: larger
          n_v, then lower index); spend a credit; stop.
       b. phase 1: candidates L = leaves v with stalled(v), not on trial, not in back-off, not made by a split
          still on trial. If L is empty, stop (the credit waits). Else pick the v maximising
              excess(v) = mean_w(v) - model_mean(v)
          where model_mean(v) = self.model_errors.mean_since(v.window_model) over v's last closed window (the
          StallHigh statistic, 810); ties: larger n_v, then lower index. Start its split on trial (1.2); spend a
          credit.
    5. return as today (Some(split) for leaves; the "check was due" flag for delegators).
    Only ONE split per scheduling point. Scheduling happens only at window closes, so its O(nodes) cost is paid
    once per W examples of some node, never per example.

0.7 MDL bar. A split that adds parameters must gain at least their description length per example:
    bar_mdl(k, n) = M * k * ln(n) / (2 n)        (nats per example; M = --grow-mdl, default 1.0; 0 disables)
k = number of free parameters the split adds, n = n_v of the split node at the time of judgement. Parameter
counts, by a new NodeModel::num_params(): Logreg / probit = class_num * width (weights per class, logreg.rs 25-28,
intercepts included if stored); Cnbp = class_num * total_cat_vals + class_num (counts and class totals); TAN as
its count of stored conditionals; FM as its weights. Leaf split: k = (B - 1) * d_leaf + d_router of the new
delegator's router. Router split: k = (m - 1) * d_router for an unsharing by m arrival paths, B * d_router +
d_top for a cluster-shadow split. Report the numbers in the log; the exact count only scales the bar.

0.8 Back-off. Per node: backoff_windows (u32, 0 initially) and eligible_after_close (u64, compared with the node's
count of closed windows, closes). After an undone trial of leaf v: backoff_windows = max(1, 2 * backoff_windows);
eligible_after_close = closes + backoff_windows; stall_streak = 0. v is "in back-off" while closes <
eligible_after_close. A kept split clears nothing (the node is a delegator now). Routers back off the same way
after a shadow test that fired but whose split was then judged worse (2.5); otherwise routers have no trials.

0.9 Node count. After every split, log "nodes N leaves L routers R splits S credits c" and, if N >= --grow-warn-
nodes (default 55), one warning line: hybrid15's fixed-point play path is exact up to 64 nodes and falls back to
float (exact, hybrid14's speed) above. Nodes = 1 + 3 * splits at B 3 (a leaf split adds B nodes; a router unsharing
adds m - 1; a cluster-shadow router split adds B).

0.10 Tie-breaks and determinism. Every statistic is a sum over the node's examples in the fixed training order;
every choice is a total order (the stated key, then larger n_v, then lower node index); credits are integers;
block boundaries are counted in the node's own examples. Two runs with the same stream, seed and flags produce
the same files. No randomness is added anywhere; --split-noise stays the only noise and uses the existing RNG in
the existing order (the trial's children are made exactly as split_leaf makes them today).

0.11 Logging (stderr, one line each, like the trial and revision lines today): window close with the window's
mean, block SE, improvement, bar, streak; every scheduling decision (candidates, chosen node, excess, credit
count); trial start, trial end with gain, block SE, both bars and the verdict; credit accrual and refund; back-
off; router shadow gains at every router close (phase 2).

## 1. Phase 1: the leaf rule (BDT; BDD leaves)

1.1 Flags. --split-rule grow (new SplitRule variant; the rule's switch); --grow-credit D (u64 positions, default
5000000); --grow-credit-cap C (default 2); --grow-stall-z (default 2.0); --grow-stall-windows (default 2);
--grow-trial-z (default 2.0); --grow-mdl (default 1.0); --grow-subtree-share (default 0.9; leaves ignore it);
--grow-trials (default 2); --grow-warn-nodes (default 55). --split-limit stays a hard cap as today (default 20 in
main.rs 517, 4 in the recipes): grow recipes pass --split-limit 1000000. --split-z, --split-trial, --revise and
--reset-leaves are not used by grow and must be absent (error if given with grow). --split-threshold keeps its
meaning (W). --trial-error must be the training objective (log-loss) for grow; error if Winner.

1.2 Trial start (at 0.6 step 4b). Call split_leaf (1040-1065) exactly as today with a Trial: the leaf becomes a
delegator over B fresh copies (forgotten, noised), the shadow is a forgotten copy of the unsplit leaf, trial.made
lists the new nodes. Add to Trial: block accumulators over the per-example difference (split_error - shadow_error)
with block length b, as 0.1; keep today's n, sum, sum_sq for the log. The trial runs for W examples through the
node (trial_step, 1286-1300: the shadow trains on every example the node sees and both are scored by
--trial-error on the examples that score). The node's own window statistics restart at the split (0.3).

1.3 Trial end (end_trial, 1340, when trial.examples >= W). With gain = -(mean of block means of split - shadow),
se_g from the blocks (0.1), n = n_v:
    keep iff gain > Z_T * se_g  AND  gain > bar_mdl(k, n),   Z_T = --grow-trial-z, k from 0.7
Kept: the split stays, nothing else changes (credits were spent at the start); the children keep their windows
(they started fresh at the split and have each seen about W/B examples). Undone: exactly today's undo (the shadow
takes the node's place, the made nodes go to dead, split_number -= 1), then refund one credit and apply 0.8.

1.4 Model file. Unchanged format: the BDT text format (bdt.rs to_text) with the current split number and nodes
(train.py in offline-ply reads lines h+4 and h+5). A model saved mid-trial contains the split as if kept (today's
rule, 377-378: "a model saved during a test keeps the split"), nothing of the trial; hybrid15/16 play it as it
is. The league's exact resume uses the .state, where the trial continues; a model file reloaded without its state
keeps the split and has no trial, as today.

1.5 State file (.state, write_state 1618-1680 / read_state). Append, inside the same records and only when
--split-rule grow is on (a state written by grow is read only by grow; the params string at the top already makes
mismatched params an error): per node: block partial sum, block count in the current block, K_b, block_sum,
block_sum_sq, prev_mean, prev_se, prev_exists, stall_streak, closes, backoff_windows, eligible_after_close; per
trial: its block accumulators; graph: credits, next_credit_at, credits_spent, credits_refunded, trials_in_flight
(derivable, but write it). Resume must reproduce the run byte for byte (extend the existing resume test).

1.6 Unit tests (synthetic streams; feed Graph directly with hand-made examples and labels, or drive node.record
with injected error sequences where the test is about statistics):
    a. Identity: grow off, a 5M-example stream: model and state byte-identical to the previous binary's.
    b. Window statistics: 20 blocks of known means give the asserted mean_w and se_w; a window of 19.5 blocks
       drops the partial block.
    c. Stagnation: injected window means 1.0, 0.8, 0.6, 0.59, 0.585, 0.58 with equal se 0.01: stall_streak 0, 0,
       0, 1, 2, 3; stagnant from the fifth close; a rise (0.58 -> 0.60) keeps the streak growing; a fall of 3 se
       resets it.
    d. Credits: D 1000, C 2, a 5000-example stream with nothing eligible: credits 2 (capped), next_credit_at
       6000; a spend then a refund restores 2; a spend with credits 0 is impossible (the scheduler does not run).
    e. Scheduler choice: three stagnant leaves with excess 0.02, 0.05, 0.05 and n 10, 20, 30: the leaf with 0.05
       and n 30 is chosen; with equal n, the lower index.
    f. Trial verdict: a stream where the children can lower the error by exactly g per example (two label
       regimes separable by one feature, the shadow cannot) is kept iff g > max(Z_T se_g, bar_mdl); test both
       sides of each bar separately (set --grow-mdl 0 to isolate Z_T, Z_T 0 to isolate the MDL bar).
    g. Undo bookkeeping: after an undone trial the node is a leaf with the shadow's parameters, dead lists the made
       nodes, split_number is back, one credit is refunded, backoff_windows 1 then 2 then 4 over three failures,
       and the node is not a candidate until closes >= eligible_after_close.
    h. One split per scheduling point; trials_in_flight never exceeds T; a node made by a split on trial is never
       a candidate.
    i. Rate: a long stationary stream with every leaf stagnant and every trial passing (synthetic gain far above
       the bars) makes exactly one kept split per D examples after the first, never two within D.
    j. Model file written mid-trial loads in the old loader and plays; the state resumes the trial.

## 2. Phase 2: the router rule (BDD with path context; error-routed delegators)

2.1 Flags. --router-split-rule grow (switch; the old --router-split-z rule stays the default and is unchanged);
--grow-router-z (default 2.0); --grow-router-shadow {auto, path, cluster} (default auto); the shared flags of 1.1
(credit, cap, stall, mdl, subtree share, warn). Scope: delegators that route by error (D-x-Ne). Cluster-routed
delegators are out of scope for phase 2 (their examples do not score every child).

2.2 Shadow partition. For a delegator v with router R_v over children c_1..c_B:
    - path: if v has m >= 2 arrival paths (the path-context cells identify the arrival path p(x) of every example
      at routing time), the partition is p(x).
    - cluster: otherwise a shadow cluster router Q_v (a Cnbp over B classes trained label-free by hard EM exactly
      as a cluster-routed delegator trains its router, including the hash warm-up of the first W/5 examples,
      delegation.rs 55-61) over v's examples; the partition is Q_v's class q(x).
    - auto: path where m >= 2, else cluster.
    Shadow routing copies S_1..S_m (m = B for cluster): clones of R_v's parameters at the shadow's start (no
    forgetting), each trained with v's error-routing target on the examples whose partition index is i, exactly
    as R_v trains on all of them. The shadows start when v first closes a window and persist (no parameter reset
    after a window; only the gain window restarts).

2.3 Gain. On every example x through v that scores (the same examples that train R_v's error target), with the
children's errors e_1..e_B already computed for the target:
    gain(x) = e[child chosen by R_v](x) - e[child chosen by S_{i(x)}](x)
accumulated in a GainWindow with blocks of b examples; at v's close: gain_w, se_gw as 0.1. Fires iff
    stalled(v)  AND  gain_w > Z_R * se_gw  AND  gain_w > bar_mdl(k, n_v),   k from 0.7.

2.4 The split (at 0.6 step 4a). Path case: UNSHARE v by arrival path: for each arrival path i, a new node v_i with
S_i's parameters as its router, v's children as its children (shared), v's error-router state copied; the
upstream node of path i now points to v_i instead of v; v is removed (dead) if no path remains; node count +
(m - 1). No top router is needed. Cluster case: as split_delegator today, except the top router is Q_v (already
trained) and the B new delegators are S_1..S_B (already trained); node count + B. In both cases the new nodes
start with fresh windows (0.3), the shadows of v are dropped, and the new nodes get their own shadows at their
first close. Spend a credit. No trial: the evidence is in hand.

2.5 Post-split check (cheap safeguard, replaces a trial): the new nodes' routing error over their first closed
window, pooled by example count, is compared with v's last window mean; if worse by more than Z_R * se (blocks),
log a warning and apply back-off to the new nodes (they cannot split again for backoff windows); the split is NOT
undone (undoing shared structure is not safe). Report the count of such warnings in the run's summary.

2.6 Model file. Unchanged BDD text format (bdd.rs save; "pathcontext" header as today). Shadows, gain windows and
credits are never in the model file; a model saved at any moment is the current structure, playable by hybrid15/
16. State: append per delegator (only under the grow router rule) the partition kind, Q_v's state (cluster case),
each S_i's state, the gain window with blocks, closes, back-off fields; the graph fields of 1.5 are shared.

2.7 Unit tests.
    a. Identity: --router-split-rule absent, a path-context DAG stream: byte-identical to errdag4 / hybrid15.
    b. Partition choice: a node with one arrival path gets a cluster shadow; after an unsharing above it, m >= 2
       and the next shadow start uses paths.
    c. Gain fires only on benefit: a synthetic stream where the best child depends on the arrival path (path A:
       child 1 is best, path B: child 2) makes gain_w > 0 and the split fires at the first close where stalled(v)
       holds; the same stream with the best child independent of the path keeps gain_w within 1 se of 0 over 10
       windows and never fires.
    d. Subtree stall: a router whose leaves are still improving (injected falling errors) is not a candidate
       although its own routing error is flat; it becomes one when leaves holding >= 90% of its examples are
       stagnant; a tiny never-settling leaf (share 0.05) does not block it.
    e. Unsharing: parents point to distinct copies, the children are shared (same indices), node count + (m - 1),
       the copies' routers equal the shadows' parameters at the split.
    f. Routers before leaves: with a router candidate and a leaf candidate at the same scheduling point, the router
       splits and the leaf waits for the next credit.
    g. The 55-node warning appears exactly when the count crosses 55.

## 3. Offline tests on the discriminator's pools (models/from-scratch/offline-ply, 21.6M train each)

Phase 1: arm G = T-P-Ne flags with --split-rule grow --split-limit 1000000, D 5M and D 2.5M, on P1 and P2, one
pass, same order as arms A/B/F. Measures: kept and undone splits with versions, leaf count by version, held-out
loss against the pool's flat and against A, the rate (kept splits per positions) against the 1/D line.
Prediction (theory-2 6.37): D 5M: root split at ~5M (v25), two level-1 trials from ~15M, 5-7 leaves kept by 21.6M
with one trial open; D 2.5M: 7-9 leaves, none at level 2; every level-1 trial passes on P1 (gains 0.01-0.1
against a bar ~0.004 at n 3M), 1-2 of 3 on P2; held-out within 0.003 of A on both pools. Phase 2: arm H =
D-P-Ne-ctx with --router-split-rule grow (auto) beside the grow leaf rule, D 5M, on P1 and P2: 0-1 router splits
on P1 (cluster-shadow at a level-1 router at the earliest ~27M after its leaves formed, so most likely 0 within
21.6M), 0 on P2; held-out within 0.001 of the tree arm G.

## 4. League test for phase 1 (recommended)

Field at PLY 1 (the ply where trees have a measured gain, ~100 over F-P, and where the rule's timing shows in Elo;
at ply 2 every tree rule ties flat so far): F-P x2 (comparator); T-P-Ne current rule (worse / z 0, --split-limit
4) x2; T-P-Ne --split-rule grow, D 5M, --split-limit 1000000 x2. One factor: the leaf rule (the cap goes with it,
since grow has none). Shared train seed as usual, floor 300, cap 400, cold reads at v150 and the final version at
plies 1-2, T 0.02; a second seed only if the contrast exceeds 20. About 1 CPU-h.
Prediction: grow minus current -15 to 10 at v150 (grow reaches 9 leaves at ~v115 against v100-150 for the current
rule, with the second level placed later) and -10 to 15 at v300; grow at 9-11 leaves at v300 with 0-2 undone
trials, none kept at the third level; both tree arms 70-120 over F-P at ply 1 and 40-80 at ply 2. The result that
matters is not the Elo contrast but the record: splits at a steady rate, placed where the model is worst, with the
third level refused at ply 1 where it has been copies in every run. If grow is 20 or more below the current rule
at v300, the stagnation gate is too slow for the field's drift (K_S 2 at W 1M) and the next run tries K_S 1.
