#!/bin/bash
# Train the C and Rust classifiers on the same games directory, check the
# saved models and predictions are identical, and compare speed.
#
# Usage: rs/bench/compare_classifiers.sh GAMES_DIR [OUT_DIR]
set -euo pipefail

GAMES_DIR=$1
OUT=${2:-$(mktemp -d)}
HERE=$(cd "$(dirname "$0")" && pwd)
C_SRC="$HERE/../../c_src/models"
mkdir -p "$OUT"

clang -O3 -o "$OUT/classifier_bench" "$HERE/classifier_bench.c" "$C_SRC/cnbp/cnbp.c" "$C_SRC/bdt/bdt.c" -lm
cargo build --release --quiet --manifest-path "$HERE/../Cargo.toml"
RUST="$HERE/../target/release/chessai"

# compare NAME REPS USE_PROBS [BRANCH_FACTOR SPLIT_THRESHOLD SPLIT_LIMIT FORGET_FACTOR]
compare() {
    local name=$1 reps=$2 use_probs=$3
    local tag="$name-p$use_probs${4:+-b$4-t$5-l$6-f$7}"
    local flag=$([ "$use_probs" = 1 ] && echo --use-probs || true)
    local rust_args=()
    [ $# -gt 3 ] && rust_args=(--branch-factor "$4" --split-threshold "$5" --split-limit "$6" --forget-factor "$7")

    echo "== $tag"
    # bdt.c prints a line for every split; keep only the summary.
    "$OUT/classifier_bench" "$name" "$GAMES_DIR" "$OUT/c-$tag.model" "$OUT/c-$tag.preds" "$reps" 1.0 "$use_probs" "${@:4}" \
        | grep -v "^Splitting Node"
    "$RUST" "bench-$name" "$GAMES_DIR" "$OUT/rs-$tag.model" "$OUT/rs-$tag.preds" --alpha 1.0 --reps "$reps" $flag "${rust_args[@]}"

    if cmp -s "$OUT/c-$tag.model" "$OUT/rs-$tag.model"; then
        echo "model files: identical"
    else
        echo "model files: DIFFER"
    fi

    python3 - "$OUT/c-$tag.preds" "$OUT/rs-$tag.preds" <<'EOF'
import sys
c = [list(map(float, l.split())) for l in open(sys.argv[1])]
r = [list(map(float, l.split())) for l in open(sys.argv[2])]
assert len(c) == len(r), f"{len(c)} vs {len(r)} predictions"
pairs = [(a, b) for lc, lr in zip(c, r) for a, b in zip(lc, lr)]
exact = sum(a == b for a, b in pairs)
worst = max(abs(a - b) for a, b in pairs)
print(f"predictions: {exact}/{len(pairs)} bit-identical, max abs difference {worst:.3g}")
EOF
}

compare cnbp 20 0
compare cnbp 20 1
compare bdt 3 0 2 2000 20 10
compare bdt 3 1 2 2000 20 10
compare bdt 3 0 3 1000 50 2
