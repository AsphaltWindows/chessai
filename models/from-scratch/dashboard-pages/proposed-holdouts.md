# Proposed holdouts for leagues/library/holdouts.csv

Written 2026-11-12 ~17:00 with the dashboard's Models page (`/m/`). holdouts.csv holds only its header: nothing
below is in it yet. To hold one out, copy its `id,reason` line into the file. The page re-reads the file at every
refresh. An id is an index id (every row with it), `<id>@<version>`, or `<experiment>:<line>@<version>`. A version
the index doesn't have shows as its own row, marked "holdout version".

Ladder figures are from ladder-1 (done; Elo ± 95%, each player at the ply shown). Not plateaued means the page hides
the model unless it is a holdout.

## A. Strong runs that hit their cycle cap (end cap, so hidden today)

```
flatp3w4_a,"W4: F-L warm-started from flat-p3@199, trained at ply 4; cap 100; ladder #1, 2503.9 ±88 at ply 4 (brief: 173 over its start at ply 4)"
flatp3w3f1_a,"F-L warm-started from flat-p3@199, trained at ply 3; cap 100; ladder #2, 2196.1 ±58.7 at ply 3"
flatp2w3_a,"F-L warm-started from flat-p2@244, trained at ply 3; cap 100; ladder #4, 2165.5 ±57.4 at ply 3"
```

- Optional: their sibling lines `flatp3w4_b`..`_d`, `flatp3w3f1_b`..`_d` and `flatp2w3_b`..`_d`. They have the same
  set-up and are not on the ladder.
- Not proposed:
  - grow-p1 (cap 400): its v300s sit mid-pack at ply 1 (856.7, 837.6). They are already panel members (C).
  - treeonly-p1 (cap 400): the uniform-field tree run. Uniform tree minus mixed tree was −14, not significant.
  - probit-widen-p1: a manual stop at cycle 66.

## B. The old global league's last champions (killed by the user, no plateau rule, so hidden today)

The Elo figures are from the all-champions arena (ply 1, T 0.02), in index.csv's note.

```
global_a,"global league last champion F-L.p1 v538; all-champions arena 893 ±19.1"
global_b,"global league last champion F-P.p1 v463; all-champions arena 874 ±18.9"
global_c,"global league last champion T-P-Nc-merge-fresh.p1 v354; all-champions arena 912 ±19.3"
global_d,"global league last champion T-L-Nc-merge-fresh.p1 v581; all-champions arena 969 ±20.2"
global_e,"global league last champion D-P-Ne-ctx-z30.p1 v5375; all-champions arena 999 ±20.8"
global_f@754,"global league last champion D-P-Ne-ctx-z20.p1 v754; all-champions arena 928 ±19.5"
global_g,"global league last champion D-L-Ne-ctx-z160.p1 v5626; all-champions arena 999 ±20.8"
global_h,"global league last champion D-L-Ne-ctx-z120.p1 v638; all-champions arena 940 ±19.7"
```

- Optional: `global_f@513`, that line's best champion (942 ±19.7).
- Plain `global_f` holds out both of its rows.

## C. The standing stranger panels (verdict reads use these exact versions; they are not the lines' finals)

The ply-1 panels at v300 are used by cold-read.sh and t002-read.sh (arm_panel.py). **None of these v300 versions has
a library snapshot.** If their experiments' runs/ are cleaned up, the panels are lost. `build_index.py --extra
EXP:LINE@300` snapshots each one; the page notes the missing snapshot in each row's tooltip.

```
errorroutingp1_a@300,"ply-1 F-P stranger panel (rfp), v300"
errorroutingp1_b@300,"ply-1 F-P stranger panel (rfp), v300"
errordagprobitp1_a@300,"ply-1 F-P stranger panel (efp), v300"
errordagprobitp1_b@300,"ply-1 F-P stranger panel (efp), v300"
growp1_a@300,"ply-1 F-P stranger panel (gfp), v300"
growp1_b@300,"ply-1 F-P stranger panel (gfp), v300"
growp1b_a@300,"ply-1 F-P stranger panel (hfp), v300"
growp1b_b@300,"ply-1 F-P stranger panel (hfp), v300"
errorroutingp1_c@300,"ply-1 tree stranger panel (rtr), v300"
errorroutingp1_d@300,"ply-1 tree stranger panel (rtr), v300"
errordagprobitp1_c@300,"ply-1 tree stranger panel (etr), v300"
errordagprobitp1_d@300,"ply-1 tree stranger panel (etr), v300"
growp1_c@300,"ply-1 tree stranger panel (gtr), v300"
growp1_d@300,"ply-1 tree stranger panel (gtr), v300"
growp1b_c@300,"ply-1 tree stranger panel (htr, T-P-Ne-grow), v300"
growp1b_d@300,"ply-1 tree stranger panel (htr, T-P-Ne-grow), v300"
```

probit-p3-read.sh's ply-3 panel adds the following. flat-p3 and flat-p3-b at v199 are plateaued finals and show
anyway; flat-p3-w4 v99 is in A. These versions do have snapshots (the cohort read's v299):

```
fponlyp2_a@299,"ply-3 panel x4 (fponly-p2 F-P at v299; final is v320)"
fponlyp2_b@299,"ply-3 panel x4 (fponly-p2 F-P at v299)"
fponlyp2_c@299,"ply-3 panel x4 (fponly-p2 F-P at v299)"
fponlyp2_d@299,"ply-3 panel x4 (fponly-p2 F-P at v299)"
fponlyp2b_a@299,"ply-3 panel x4b (fponly-p2b F-P at v299; final is v352)"
fponlyp2b_b@299,"ply-3 panel x4b (fponly-p2b F-P at v299)"
fponlyp2b_c@299,"ply-3 panel x4b (fponly-p2b F-P at v299)"
fponlyp2b_d@299,"ply-3 panel x4b (fponly-p2b F-P at v299)"
baseprobitp2_a@299,"ply-3 panel bfp (base-probit-p2 F-P at v299; final is v599)"
baseprobitp2_b@299,"ply-3 panel bfp (base-probit-p2 F-P at v299)"
```

## D. Ladder players at versions other than their finals (the depth anchors and ply-2 entries)

Their finals are plateaued and show anyway. The ladder rates these exact versions, so holding them keeps ladder-1
reproducible. The F-P versions have snapshots; **the T-P-Ne ones don't** (same remedy as C).

```
replayp2_a@299,"ladder depth anchor F-P v299: 2033.4 at ply 3, 1539.8 at ply 2, 639.5 at ply 1"
replayp2_c@299,"ladder depth anchor T-P-Ne v299: 1916.2 at ply 3, 1544.6 at ply 2, 621.5 at ply 1"
diversityp2_a@299,"ladder ply-2 entry F-P v299: 1668.1"
diversityp2_c@299,"ladder ply-2 entry T-P-Ne v299: 1616.9"
probitlatep2_a@299,"ladder ply-2 entry F-P v299: 1621.5"
probitlatep2_c@299,"ladder ply-2 entry T-P-Ne v299: 1613.4"
```

The error-routing-p1, error-dag-probit-p1, grow-p1 and grow-p1b v300 entries on the ladder are already in C.

## E. After the index is rebuilt

The index dates from 11:56. probit-p3 ended at 14:50 by the plateau rule ("no new champion since cycle 267"), but
the index has it at v64 with end unknown. treeonly-p1-coldrate and -coldtrain are listed the same way (v113). The
t002 and cold-games arms, filtered-p1-twin and the open runs are missing. `python3 leagues/library/build_index.py`
brings it up to date. After that:

- probit-p3's arms (p3tr, p3fp; the ply-3/ply-4 reads use v150 and v301) show as fully plateaued finals. Their v150
  is not a final: hold it out (`probitp3_a@150` ...) if the v150 reads stay in use.
