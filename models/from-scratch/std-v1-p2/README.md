# std-v1-p2: the ply-2 rung of training standard v1 (prep, 2026-11-12)

Prepared by the ply-2 prep fork, following analyst 6.95. Nothing is launched; the launch waits on decision 2 (the
user's call). The rung is described in models/STANDARD.md, section "The ply-2 rung".

## Specs and scripts
- **Specs:** leagues/specs/standard/std-v1-p2-{s1,s2,x3}.DRAFT.json, written by prep/make_drafts.py from the ply-1 spec.
- **Start versions:** leagues/specs/standard/fill_p2_starts.py fills the warm drafts' start versions once std-v1-p1-sN
  is done. It runs the driver's create checks and writes std-v1-p2-sN.json. It needs the warm2 driver: a model entry's
  start_reset and train_binary.
- **Read:** leagues/specs/standard/std_p2_read.py, a draft.
  - `make early|final` writes the arena spec.
  - `read early|final` gives the stranger fit, per recipe and pooled, warm against scratch.
  - `selftest` checks the fit against a simulated truth.

## prep/
- **Driver patch:** patch_warm2.py, the patch for leagues/.leagues-warm2wip.py.
- **Selftests:** selftest-quick.out and selftest-full.out.
- **Repro:** repro_warm2.py, the 5-cycle byte-identity re-runs of std-v1-p1-s1 and -s2, in repro/ and repro2/
  (repro2/ is the final work copy).
- **Freezing:** freeze_start.py, which freezes a live run's current versions as a warm-start source.
- **Swap:** swap_warm2.py, the atomic swap with backups and the docs edits.
- **STANDARD.md.before-p2:** the standard before the ply-2 section.

## proof/
- **Source:** starts/std-v1-p1-s1@340, frozen from the live run.
- **Proof run:** w2proof.json and proof.sh. This is s1's draft warm from the frozen copy, at ply 2, with 240 games a
  cycle for 5 cycles. The kept/ directory holds cycle 1's training set.
- **Replay checks:** replicate_w2.py and replicate.txt, with 35 checks and 0 failures.
- **Smoke run:** x3smoke.json, x3's draft run for 5 cycles at ply 2.
- **Scratch root:** root/, which holds both runs.

The binary is models/from-scratch/bin/chessai-grow7; see fastbin/grow7/summary.txt.
