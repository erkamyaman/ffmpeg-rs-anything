#!/usr/bin/env bash
# ffmpeg's own libavutil.a (its configure and make, -O3) against the Rust ports, from one C
# program. Run prepare.sh first. Prints the machine, then the table; run it twice on a quiet
# machine and compare.
set -euo pipefail
HERE=$(cd "$(dirname "$0")/.." && pwd)
WORK=${WORK:-$HERE/work}
FF=$WORK/ffmpeg
cargo build --release --quiet --manifest-path "$HERE/bench/rust-all/Cargo.toml" \
  --target-dir "$WORK/bench-target"
cc -O3 -I"$FF" -o "$WORK/bench" "$HERE/bench/bench.c" "$FF/libavutil/libavutil.a" \
  "$WORK/bench-target/release/librust_all.a" -lm -lpthread
if command -v sysctl > /dev/null && sysctl -n machdep.cpu.brand_string > /dev/null 2>&1; then
  echo "$(sysctl -n machdep.cpu.brand_string), $(sysctl -n hw.ncpu) cores, $(uname -sr)"
else
  echo "$(grep -m1 'model name' /proc/cpuinfo | cut -d: -f2), $(nproc) cores, $(uname -sr)"
fi
echo "$(cc --version | head -1); $(rustc --version)"
"$WORK/bench"
