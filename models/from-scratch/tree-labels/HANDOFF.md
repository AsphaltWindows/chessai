# tree-labels build fork (started 2026-11-13 02:47; analyst 6.113)

Goal: --label-depth / --label-result / --label-model for bdt and bdd (trees, DAGs), byte-identical when off;
Part B a moving labeller (another line's current version); Part C the labels-tree-p2 draft (not launched).

Layout (all on disk here):
- src/            copy of fastbin/grow7-src (grow7 = hybrid6 on capped trees, = grow6 on grow trees/DAGs), patched:
                  main.rs only (bail now allows bdt/bdd; Bdt/Bdd branches call train_rounds_labelled; that function
                  gained a scores_rounds branch (observe_round/train_in_game/end_round with game ids) and refuses
                  needs_games models; help text). Build: `nice cargo build --release -j 3` in src/.
- cand/chessai-treelab  FINAL build md5 e7d31b0e81f03afb6c40ff1f4bb29df0 (help text + comment edits after build1;
                  build1 df571926... moved to cand/build1, its partial identity to id/p1-build1). Identity suite
                  restarted 03:04 on the final (JOBS=2; dl-std-grow7 left running from the first suite, .done by hand).
- id_p1_tl.py / id_p1_tl.txt / id/p1   identity vs grow7's stored runs (fastbin/grow7/id/p1) + tlneg-std/dl-std
                  on both binaries (their stored runs are incomplete now) + fp-lab3 (flat labels vs hybrid6).
- resume_check.sh exact resume (2 rounds = 1 + resume with s' = s ^ sm(1) ^ sm(0)) and 1 vs 3 threads. NOT RUN YET.
- smoke/          s1-* (label-result 1 == no labels: SAME), t-own/t-fix, d-own/d-fix (own search == --label-model
                  start model: SAME, tree and DAG). smoke/league/: drv.py (WIP leagues.py + temp recipes), recipes.txt.
- draft/write_draft.py  writes leagues/specs/batch7/labels-tree-p2.DRAFT.json (cost sentence as argv[1]).

Part B (leagues/.leagues-labellinewip.py, not swapped yet): label_model_refs / label_lines_option (create) /
label_line_args (train_line). Quick selftest 246 pass (selftest/quick.out). 446 configs, 1,136 lines: no
--label-model anywhere -> train cmd unchanged (same list object). Base md5 of leagues.py b89c6a4c... (check before
swap; backup leagues.py.before-labelline).
Recipes: leagues/.make_recipes-treelabwip.py adds F-P-lab and T-P-Ne-cap (HAND); not swapped yet (make_recipes.py
md5 62c57f08..., recipes.txt 86c26b36...; the pairs3 fork may add F-P-S7v3 too: re-diff before swapping).

Done: inert/inert_check.py: 100 experiments, 511 lines: train commands IDENTICAL old vs WIP (03:07).
Written, not run: cli_checks.sh (A-D label-result 1 == none incl. scores-rounds tree, grow tree, DAG; E own == fixed;
G refusals), resume_check.sh, swap.sh (md5-guarded atomic swaps + quick selftest), draft/labels-tree-read.sh.
Next: identity + unit tests (bfcppeqve) -> cli_checks + resume_check -> install
bin/chessai-treelab (cp -n) -> resume_check -> league smoke (smoke/league) -> swap leagues.py + recipes -> draft.

04:08 DONE: identity 11/11 SAME (id_p1_tl.txt), unit tests 248+1, cli_checks 0 failures, resume_check 0 failures,
bin/chessai-treelab installed (e7d31b0e...), league smoke OK, full selftest 246, swap.sh run 04:08:01 (leagues.py
e98aa828..., make_recipes.py da2be44d..., recipes.txt +F-P-lab +T-P-Ne-cap), BUILD.md section, draft written and
validated (draftcheck/), summary.txt complete. Nothing left running.
