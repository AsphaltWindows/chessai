# EXTEND fork handoff (written 2026-11-12 22:16; DONE 23:00, see summary.txt)

Directive (coordinator, ~21:25): build an exact `python3 leagues/leagues.py extend LEAGUE_DIR --cycles N` for
experiments that ended at their cap; prove it byte-identical to a fresh run with the new cap; repro live runs with
the old vs new driver; swap atomically (backup leagues/leagues.py.before-extend); docs; DO NOT launch any extension.
Update 22:15: extend std-v1-p1-s1/-s2 by 200, flat-p3-w4, flat-p3-w3f1 and treeonly-p1 by 100, skip the rest; live
runs explore-t015-p1 and probit-field-p1 (batch7) need the repro too; don't touch pairs-src/ or residual-p2/.

## Done
- Work copy leagues/.leagues-extendwip.py, md5 b89c6a4c603f77c68a054758e0642cb9, built by patch_extend.py from the
  live leagues.py (1315b8ef...): additive only (9 hunks). Off-path unless config.json has cap_history.
- extend_test on an earlier build passed (t/extend_test.out: 184 items identical, 7 crownings undone, 9 refusals).
- repro std-v1-p2-s1 K5 on the new driver: 91 items identical (repro/repro.out).
- Dry runs of all 13 candidates pass, read-only (dryrun-candidates.txt). caps.py / caps.txt: plateau Monte Carlo.
- leagues/library/build_index.py and LIBRARY.md (`extending` reason; backups .before-extend), tested in libtest/.
- leagues/dashboard.py swapped (ca402631... -> cf871526..., backup dashboard.py.before-extend): cap history line.
  Servers not restarted.
- FORMATS.md (backup .before-extend): extend/cap-<C>/ entry and the "Cap history" bullet. BUILD.md: backup only.

## Running (background, this session)
- bcny0ub7l: repro x3 K5, then s2 K3 -> repro/repro.out.
- bf73fu275: run_tests.sh: extend_test alone (t2/, t2.out) and the full selftest (full/, full.out) on the work copy;
  tested copy full/leagues-extendwip-tested.py, md5 in full-wip.md5.
- bgc2b0ts4: repro explore-t015-p1 (passed) and probit-field-p1 K5 -> repro7/; b635zr4as: residual-p2 K5 -> repro8/
  (live since 22:19; the repro waits for its cycle 5). swap_extend.py SELFTEST_SUMMARY does the swap and the docs.

## Left
1. All tests and repros pass (else fix, rebuild with patch_extend.py, re-test).
2. Swap: work copy md5 == tested md5; live still 1315b8ef; cp -p leagues.py leagues.py.before-extend; copy the work
   copy to a temp file in leagues/ and os.replace onto leagues.py between cycles; record time + md5; quick selftest
   on the swapped file; `leagues.py status` on a live run (read-only).
3. FORMATS.md Changes entry (newest first); BUILD.md section "Extend (2026-11-12 ~21:25-...)".
4. summary.txt (+ the <25-line report): command, proof, swap, caveats, the five extensions' new caps.
Never: launch an extension, touch the live runs, edit rs/ or models/from-scratch/bin/*, commit, delete without OK.

## Done at 23:00
All tests and the six live-run repros passed; swapped in at 22:56:00 (leagues.py b89c6a4c..., backup
leagues.py.before-extend 1315b8ef...); quick selftest after the swap passed; FORMATS.md Changes and BUILD.md "Extend"
written; summary.txt has the report. No extension launched.
