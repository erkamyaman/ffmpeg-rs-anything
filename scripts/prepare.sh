#!/usr/bin/env bash
# Clones ffmpeg n7.1, configures it, builds libavutil.a, and assembles the tree the ports were
# recorded from: ffmpeg's config.h, libavutil's headers, the .c files the ported units need to
# link, and the one-shot wrappers. Nothing from ffmpeg is copied into this repository.
set -euo pipefail
HERE=$(cd "$(dirname "$0")/.." && pwd)
WORK=${WORK:-$HERE/work}
TAG=n7.1
COMMIT=b08d7969c550a804a59511c7b83f2dd8cc0499b8
FF=$WORK/ffmpeg
TREE=$WORK/tree

mkdir -p "$WORK"
if [ ! -d "$FF" ]; then
  git clone --quiet --depth 1 --branch "$TAG" https://github.com/FFmpeg/FFmpeg.git "$FF"
fi
test "$(git -C "$FF" rev-parse HEAD)" = "$COMMIT" || { echo "expected ffmpeg $COMMIT"; exit 1; }

if [ ! -f "$FF/libavutil/libavutil.a" ]; then
  (cd "$FF" && ./configure --disable-programs --disable-doc --disable-avdevice --disable-avcodec \
    --disable-avformat --disable-swresample --disable-swscale --disable-postproc \
    --disable-avfilter --disable-network --disable-autodetect > "$WORK/configure.log" \
    && make -j"$(getconf _NPROCESSORS_ONLN)" libavutil/libavutil.a > "$WORK/make.log")
fi

rm -rf "$TREE"
mkdir -p "$TREE/libavutil"
cp "$FF/config.h" "$FF/config_components.h" "$TREE/"
(cd "$FF/libavutil" && find . -name '*.h' | while read -r h; do
  mkdir -p "$TREE/libavutil/$(dirname "$h")" && cp "$h" "$TREE/libavutil/$h"; done)
(cd "$FF" && find compat -name '*.h' | while read -r h; do
  mkdir -p "$TREE/$(dirname "$h")" && cp "$h" "$TREE/$h"; done)
for c in adler32 base64 crc md5 sha murmur3 avstring parseutils rational mathematics \
         bprint eval random_seed time log mem reverse; do
  cp "$FF/libavutil/$c.c" "$TREE/libavutil/"
done
cp "$HERE"/wrappers/*.c "$TREE/libavutil/"
echo "ffmpeg $TAG ($COMMIT) at $FF; tree at $TREE"
