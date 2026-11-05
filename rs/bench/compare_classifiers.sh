#!/bin/bash
# Train the C and Rust CNBP on the same games directory, check the saved
# models and predictions are identical, and compare speed.
#
# Usage: rs/bench/compare_cnbp.sh GAMES_DIR [OUT_DIR]
set -euo pipefail

GAMES_DIR=$1
OUT=${2:-$(mktemp -d)}
HERE=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$OUT"

clang -O3 -o "$OUT/cnbp_bench" "$HERE/cnbp_bench.c" "$HERE/../../c_src/models/cnbp/cnbp.c" -lm
cargo build --release --quiet --manifest-path "$HERE/../Cargo.toml"
RUST="$HERE/../target/release/chessai"

for use_probs in 0 1; do
    echo "== use_probs=$use_probs"
    flag=$([ $use_probs = 1 ] && echo --use-probs || true)
    "$OUT/cnbp_bench" "$GAMES_DIR" "$OUT/c$use_probs.model" "$OUT/c$use_probs.preds" 1.0 $use_probs 20
    "$RUST" bench-cnbp "$GAMES_DIR" "$OUT/rs$use_probs.model" "$OUT/rs$use_probs.preds" --alpha 1.0 --reps 20 $flag

    if cmp -s "$OUT/c$use_probs.model" "$OUT/rs$use_probs.model"; then
        echo "model files: identical"
    else
        echo "model files: DIFFER"
    fi

    python3 - "$OUT/c$use_probs.preds" "$OUT/rs$use_probs.preds" <<'EOF'
import sys
c = [list(map(float, l.split())) for l in open(sys.argv[1])]
r = [list(map(float, l.split())) for l in open(sys.argv[2])]
assert len(c) == len(r), f"{len(c)} vs {len(r)} predictions"
pairs = [(a, b) for lc, lr in zip(c, r) for a, b in zip(lc, lr)]
exact = sum(a == b for a, b in pairs)
worst = max(abs(a - b) for a, b in pairs)
print(f"predictions: {exact}/{len(pairs)} bit-identical, max abs difference {worst:.3g}")
EOF
done
