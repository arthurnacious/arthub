#!/usr/bin/env bash
# Draw the pool-ball texture atlas: a 4×4 grid of equirectangular ball maps
# (2048×1024, 512×256 per ball; ball N at column N%4, row N/4). Each ball has
# its number twice, on opposite sides of the equator. Stripes 9–15 are a
# coloured band on white. Then `just bake-textures` compresses it.
set -euo pipefail
cd "$(dirname "$0")/.."
FONT=/usr/share/fonts/noto/NotoSans-Bold.ttf
mkdir -p art
COLORS=("#f2efe4" "#f7c41a" "#1d3fb8" "#d81f1a" "#5a1d86" "#f46a14" "#14713a" "#7a1414" "#111111")

args=(-size 2048x1024 xc:black)
for n in $(seq 0 15); do
  x=$(( (n % 4) * 512 )); y=$(( (n / 4) * 256 ))
  if (( n == 0 )); then base=${COLORS[0]}; band=""
  elif (( n <= 8 )); then base=${COLORS[$n]}; band=""
  else base="#f2efe4"; band=${COLORS[$((n - 8))]}; fi
  args+=(-fill "$base" -draw "rectangle $x,$y $((x + 511)),$((y + 255))")
  [[ -n "$band" ]] && args+=(-fill "$band" -draw "rectangle $x,$((y + 82)) $((x + 511)),$((y + 173))")
  if (( n > 0 )); then
    for cx in $((x + 128)) $((x + 384)); do
      args+=(-fill "#f6f3ea" -draw "circle $cx,$((y + 128)) $((cx + 34)),$((y + 128))")
      args+=(-fill "#111111" -font "$FONT" -pointsize $(( n > 9 ? 34 : 40 )) -gravity NorthWest \
             -annotate "+$((cx - (n > 9 ? 20 : 12)))+$((y + (n > 9 ? 110 : 107)))" "$n")
    done
  else
    # The cue ball's red spot, as on practice balls.
    for cx in $((x + 128)) $((x + 384)); do
      args+=(-fill "#c02020" -draw "circle $cx,$((y + 128)) $((cx + 7)),$((y + 128))")
    done
  fi
done
magick "${args[@]}" art/balls.png
echo "art/balls.png: $(magick identify -format '%wx%h' art/balls.png)"
