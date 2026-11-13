#!/usr/bin/env python3
"""Set the training binary of every line of a created experiment before its first cycle (written 2026-11-10 ~22:45
for the ply-1 experiments of this folder; optional).

    python3 leagues/specs/ply1/train_binary.py LDIR [BINARY]   LDIR: an experiment or (since 11-11) a global league;
                                                                BINARY: a name in models/from-scratch/bin or a path
                                                                (default chessai-hybrid6)
    python3 leagues/specs/ply1/train_binary.py --check LDIR [BINARY]   the same checks, nothing written
    python3 leagues/specs/ply1/train_binary.py --from NAME [--check] LDIR [BINARY]   only the lines whose training
                                                                binary's file name is NAME (11-11: dag-lr-p1, whose
                                                                DAG lines keep chessai-dag-q9)

A plain `leagues.py create` gives each line its recipe's training binary (recipes.txt: chessai-hybrid4gfx for F-L,
T-L-Tc-merge and T-L-Nc-merge). chessai-hybrid6 trains the same models byte for byte with ~0.6 of the training CPU
(models/from-scratch/fastbin/PERF-LEDGER.md); batch4.py gives it to the batch-4 experiments (small-field-x4
included), and it was swapped into the older runs while they ran. This does the same for an experiment created
without batch4.py: it refuses a league past cycle 0 (version 0 is made in cycle 1 by the line's training binary) or
one a driver holds, checks that the binary's `train` accepts every line's training arguments, writes config.json
atomically, rewrites lines.csv and logs the change. Only the training binary changes: self-play binaries stay."""
import importlib.machinery
import importlib.util
import os
import sys

ROOT = "/home/iv/dev/chessai/leagues"
BIN = "/home/iv/dev/chessai/models/from-scratch/bin"


def load_engine():
    loader = importlib.machinery.SourceFileLoader("leagues", os.path.join(ROOT, "leagues.py"))
    spec = importlib.util.spec_from_loader("leagues", loader)
    mod = importlib.util.module_from_spec(spec)
    sys.modules["leagues"] = mod
    loader.exec_module(mod)
    return mod


LG = load_engine()


def main(argv):
    src = None
    if argv[:1] == ["--from"] and len(argv) >= 2:
        src, argv = argv[1], argv[2:]
    check_only = argv[:1] == ["--check"]
    args = argv[1:] if check_only else argv
    if not 1 <= len(args) <= 2:
        sys.exit(__doc__)
    ldir = os.path.abspath(args[0])
    name = args[1] if len(args) == 2 else "chessai-hybrid6"
    binary = name if os.sep in name else os.path.join(BIN, name)
    binary = os.path.abspath(binary)
    if not os.access(binary, os.X_OK):
        sys.exit(f"{binary}: no such binary")
    cpath = os.path.join(ldir, "config.json")
    cfg = LG.read_json(cpath)
    st = LG.read_json(os.path.join(ldir, "state.json"))
    if not cfg or not st:
        sys.exit(f"{ldir}: no league there (config.json and state.json)")
    if cfg.get("kind") not in ("experiment", "global"):  # global leagues too since 2026-11-11 ~14:00 (the six)
        sys.exit(f"{ldir}: an experiment or a global league only")
    if st.get("cycle") != 0 or st.get("status") != "created" or st.get("current"):
        sys.exit(f"{ldir}: cycle {st.get('cycle')}, status {st.get('status')}: the training binary can only change "
                 "before cycle 1")
    lines = [lc for lc in cfg["lines"] if src is None or os.path.basename(lc["train_binary"]) == src]
    flags = LG.binary_flags(binary, "train")[0]
    bad = sorted({f for lc in lines for f, _ in LG.parse_flags(lc["train_args"]) if f not in flags})
    if bad:
        sys.exit(f"{os.path.basename(binary)} train doesn't accept {', '.join(bad)}")
    changed = [lc["line"] for lc in lines if lc["train_binary"] != binary]
    if check_only or not changed:
        print(f"{ldir}: {len(changed)} of {len(cfg['lines'])} lines would change to {os.path.basename(binary)}"
              + (" (checked; nothing written)" if check_only else "; nothing to do"))
        return
    lock = LG.FileLock(os.path.join(ldir, ".lock"), block=False)
    try:
        lock.__enter__()
    except OSError:
        sys.exit(f"{ldir}: a driver holds the league; set the binary before it starts")
    try:
        before = {lc["line"]: os.path.basename(lc["train_binary"]) for lc in lines}
        for lc in lines:
            lc["train_binary"] = binary
        LG.write_json(cpath, cfg)  # atomic
        L = LG.League(ldir)
        LG.write_lines_csv(L)
        was = ", ".join(sorted(set(before.values())))
        L.log(f"training binary {os.path.basename(binary)} set on {len(changed)} lines (was {was}; "
              f"specs/ply1/train_binary.py, before cycle 1)")
    finally:
        lock.__exit__(None, None, None)
    print(f"{ldir}: training binary {os.path.basename(binary)} on {len(changed)} lines")


if __name__ == "__main__":
    main(sys.argv[1:])
