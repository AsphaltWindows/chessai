# chessai

This is research on chess evaluation functions learned by self-play from zero knowledge. The project began in 2019 as C and Python code. In November 2026 it was rewritten in Rust and run as a large set of experiments.

The question was whether a model that runs only part of itself on each position can evaluate chess as well as one that runs all of itself:

- **A routed model** is a tree or DAG of small classifiers that sends each position down one path to a single specialist leaf.
- **A flat model** is one logistic regression or probit over the raw board, so it uses every weight on every position.

Everything is learned from self-play over the raw board:

- **Inputs:** 70 values: the piece on each square, the side to move, the en passant file and the castling rights.
- **No outside knowledge:** no hand-made features, engine games or engine labels.
- **Shallow search:** every model is judged inside a fixed 1–4 ply search.

## Outcome

No routed model beat the flat ones.

- **Structure:** the best trees and DAGs matched the flats, but none led them by the 30 Elo the pre-registered rules required.
- **Training regime:** the search depth a model trains at, its labels and its data moved strength by hundreds of Elo. Structure moved it by tens.
- **Leela data:** a last experiment trained the same model types on 50 million positions valued by Leela, the way Stockfish trains its network. Every Leela-trained tree and DAG lost to the self-play models of its type, and to the flat model trained on the same data.

| Write-up | What it holds |
|---|---|
| [models/RESULTS.md](models/RESULTS.md) | Every experiment and study, grouped by topic; each topic opens with where it stands |
| [models/NEXT-EXPERIMENTS.md](models/NEXT-EXPERIMENTS.md) | The dated research log, newest first, with every verdict and the rule it was judged by |
| [models/STANDARD.md](models/STANDARD.md) | The training standard: the best known settings per model recipe and search depth |
| [models/IDEAS.md](models/IDEAS.md) | Open, queued, tried and rejected ideas |
| [models/README.md](models/README.md) | A guide to the research notes and their archive |

The summary report ("chessai: final report") and a catalogue of every avenue explored ("chessai: avenues explored") are docs in the owner's claude.ai account.

## Layout

| Path | Contents |
|---|---|
| `rs/` | The Rust engine (crate `chessai`), one command-line program covering the steps below. |
| `leagues/` | The experiment system used from 9 November 2026, with docs in BUILD.md, FORMATS.md, ARENA.md and DASHBOARD.md. |
| `cycles/` | The earlier league drivers, the cycle system and the champion run: code, configs, lineages and cycle reports. |
| `models/` | The research notes (`*.md`) and their archive. Models, binaries and study folders on disk are not tracked. |
| `c_src/`, `py_src/` | The original C and Python implementation. |
| `games/`, `selfplay/` | Data folders. They are not tracked. |

The Rust engine (`rs/`) does the following:
- encodes the board, generates moves and searches;
- holds the models: naive Bayes (CNBP), LR, trees (BDT), DAGs (BDD), factorisation machines, TAN, Hoeffding trees and mixtures;
- runs self-play, tournaments and calibration.

The experiment system (`leagues/`) has these parts:

| File or folder | Role |
|---|---|
| `leagues.py` | Self-play experiments with champions and plateau rules |
| `arena.py` | Round-robin arenas |
| `recipes.txt` | Model recipes and their training arguments |
| `specs/` | Every experiment and arena spec |
| `dashboard.py` | The dashboard and the Play page |
| `library/` | An index of plateaued final models |

## Code that is not in git

`rs/` holds the engine as committed on 9 November 2026. Later versions were built as source copies under `models/from-scratch/<study>/`, and their binaries sit in `models/from-scratch/bin/`. Those later versions add:
- probit leaves;
- growth rules;
- DAG path context and router splits;
- search labels and pairwise features;
- the exact fixed-point speedups;
- the Leela converter.

The newest copy is `models/from-scratch/leela/src`. `.gitignore` excludes `models/**` to keep data and models out of the repository, so these copies and each study's write-up (`models/from-scratch/<study>/summary.txt`) exist only on the local disk.

## Running

```
cd rs && cargo build --release          # builds target/release/chessai
./target/release/chessai --help         # self-play, train, arena, tournament, calibration, eval-loss, ...
python3 leagues/dashboard.py            # the dashboard on 127.0.0.1:8766 (needs the local league files)
```

The experiment system calls the binaries named in `leagues/recipes.txt` from `models/from-scratch/bin/`, which is not tracked.

## Data on disk

A cleanup on 13 November 2026 deleted every game and every intermediate model. It kept:
- the library models;
- each experiment's final models and final champions;
- all results and stats.

None of these files are in git.
