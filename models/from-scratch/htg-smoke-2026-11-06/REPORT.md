# Model-gain split tree: 150-round trial (2026-11-06)

## Setup

- **Binary and recipe.** Binary: `models/from-scratch/bin/chessai-htg`. Recipe: F-L-s21's (LR, jitter10 at depth 0, result labels, 1000 games per round, version N vs N-1, seed 21), with `--model ht --leaf lr --ht-criterion model-gain` and the default `--ht-gain-threshold 1e-4 --ht-lambda 10 --ht-sample 1.0`. 150 rounds at nice 10. The leaf is checked every round.
- **Split rule.** Games are split by hash into halves A and B. For every test "feature f == v" at a leaf, the Newton gain is ½·Σ[G_yes²/(H_yes+λ) + G_no²/(H_no+λ) − G²/(H+λ)], computed from the leaf's per-weight gradient and Hessian sums. The best test is picked on A. The leaf splits only if A's yes and no steps, applied to B's games, lower B's loss more than A's unsplit step does, by more than 1e-4 nats per position.
- **Timing baseline.** A plain LR run (`F-L-timing-s21`) with the same binary, seed and recipe ran alongside under the same load.

## Results

1. **No split in 150 checks.** With no split the tree is a flat LR, so the run is F-L-s21 exactly: `results.txt` is identical for all 150 rounds. The building agent also found all 150 saved leaves byte-identical to F-L-s21's `lr*.model`, then deleted them.

2. **The best test's gains are noise.** Root checks every 10 rounds:

   | Round | Best test on A | Positions per half | A gain per position | B gain per position | A total | B total | Formula recomputed on B |
   |---:|---|---:|---:|---:|---:|---:|---:|
   | 0 | h6 white pawn | 0.3M | 1.01e-01 | −1.44e-01 | 33K | −47K | 5.23e-02 |
   | 10 | h4 empty | 1.8M | 2.23e-02 | −4.22e-02 | 40K | −74K | 1.69e-02 |
   | 20 | e7 black pawn | 3.2M | 9.22e-03 | −1.83e-02 | 29K | −56K | 1.30e-02 |
   | 30 | a6 empty | 4.5M | 7.00e-03 | −1.12e-02 | 32K | −50K | 6.04e-03 |
   | 40 | a6 empty | 5.9M | 5.20e-03 | −7.49e-03 | 31K | −43K | 4.05e-03 |
   | 50 | h7 black pawn | 7.3M | 4.28e-03 | −6.33e-03 | 31K | −45K | 2.88e-03 |
   | 60 | a6 empty | 8.5M | 3.59e-03 | −5.31e-03 | 31K | −45K | 2.94e-03 |
   | 70 | a6 empty | 9.8M | 3.19e-03 | −4.54e-03 | 31K | −44K | 2.45e-03 |
   | 80 | h2 white pawn | 11.1M | 2.92e-03 | −4.12e-03 | 32K | −45K | 2.64e-03 |
   | 90 | h1 empty | 12.4M | 2.57e-03 | −3.39e-03 | 32K | −42K | 2.15e-03 |
   | 100 | h1 empty | 13.7M | 2.40e-03 | −3.36e-03 | 33K | −46K | 2.13e-03 |
   | 110 | b1 white knight | 15.0M | 2.08e-03 | −2.99e-03 | 31K | −44K | 2.30e-03 |
   | 120 | h2 white pawn | 16.2M | 1.89e-03 | −2.22e-03 | 31K | −36K | 1.86e-03 |
   | 130 | h2 white pawn | 17.5M | 1.73e-03 | −1.85e-03 | 30K | −32K | 1.68e-03 |
   | 140 | h2 white pawn | 18.7M | 1.60e-03 | −1.66e-03 | 30K | −31K | 1.53e-03 |

   - A's total gain stays at 30–33K nats while the positions grow 10× (rounds 10 to 140). A real effect would grow in proportion to the data; a constant total is what fitting noise gives.
   - B's gain is negative in every check and ends at about minus A's (−31K against +30K nats). That is the out-of-sample cost of fitting noise.
   - The gain formula recomputed on B's own data is positive and about as large as A's gain. That is why it can't serve as the check.
   - The least negative B gain of any check was −1.54e-3 nats per position (h2 white pawn, round 141, 18.9M positions per half). The threshold is +1e-4.
   - Round labels follow `analyse.py`: a check is labelled with the round whose log line it follows, so its sums include the next round's games too. The positions column is exact.

3. **Which tests won.** Over all 150 checks:
   - h2 white pawn 41, a6 empty 34, b1 white knight 16, h1 empty 13, h4 empty 9.
   - h7 black pawn 5; h3 white pawn, e7 black pawn and f4 white pawn 4 each.
   - a3 white pawn, a6 white pawn and h5 black pawn 3 each.
   - No king square was the best or the runner-up test in any check.

4. **Cost.** Under the same load:

   | Run | Play (s per round) | Train (s per round) | Total (s per round) |
   |---|---:|---:|---:|
   | T-L-HG (model gain) | 1.90 | 6.04 | 7.94 |
   | F-L, same binary, alongside | 1.85 | 2.03 | 3.88 |

   Training costs 3.0× LR and a whole round 2.0×. Most of the extra is the pair sums, about 2,500 updates per position. Per 50-round block, training was 6.57, 5.71 and 5.85 s against LR's 2.22, 1.94 and 1.94 s.

## What it means

- **No single test helps.** With up to 19M positions per half (about 75,000 games), no single "piece on square" test lets an LR leaf do better on games it wasn't fitted on.
- **The detection floor falls with data.** The noise total is roughly constant at about 31K nats, so the floor per position falls as 1/n.
  - At 150 rounds the floor is about 1.7e-3 nats per position; at 1000 rounds it would be about 2.5e-4.
  - So a 1000-round run could detect effects about 7× smaller.
  - That run would cost about 2× F-L per round, roughly 20 minutes on an idle machine.
- **No king-square conditioning.** It did not discover conditioning on a king's square. In this run, nothing about the king came close.

## Caveats

- **The gain is a ranking score.** The diagonal gain counts the residual an example's ~71 weights share once per weight, so "nats per position" is a ranking score, not a real change in log-loss.
- **Early rounds stay in the statistics.** The sums accumulate from the leaf's creation, so early rounds, played by weaker models, stay in them.
- **Three deliberate changes from the spec**, by the building agent:
  - B's gain applies A's steps to B's games, instead of recomputing the formula on B. The recomputed formula is positive even on noise, as column 8 shows.
  - Sums are 64-bit floats. In 32-bit, sums past about a million positions lose most of each new term.
  - Gradients come from the live leaf. Gradients from a frozen per-round copy inflated A's gains above 1 nat per position and split on noise in round 1.
- **Tests not rerun.** The building agent reported all 60 tests passing. They weren't rerun here, because another agent is editing `rs/` now.

## Files (this directory)

- `T-L-HG-s21.log` and `F-L-timing-s21.log`: the two runs' logs, copied from the scratchpad.
- `analyse.py`: the analysis script. Its paths point at the scratchpad copies.
- `analysis.txt`: the script's full output.
