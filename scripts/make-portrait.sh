#!/usr/bin/env bash
# Make both portraits from one photo:
#   assets/room/portrait.jpg     low-poly painting hanging in the game room
#   crates/site/ascii/*.txt      ASCII portrait in the home hero (+ lens layer)
#   ./scripts/make-portrait.sh path/to/photo.jpg
set -euo pipefail
cd "$(dirname "$0")/.."
SRC="$1"
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT
magick "$SRC" -alpha off -gravity center -extent "%[fx:min(w,h)]x%[fx:min(w,h)]" "$TMP/sq.png"

# Low-poly: lift contrast a little, then triangulate (dense facets on the
# eyes/mouth, loose ones on the background).
magick "$TMP/sq.png" -resize 768x768 -modulate 100,115 -sigmoidal-contrast 3x45% -depth 8 "rgb:$TMP/in.rgb"
python3 -I scripts/lowpoly.py "$TMP/in.rgb" "$TMP/out.rgb" 768 5000 7 0.04 0.012 0.07
magick -size 768x768 -depth 8 "rgb:$TMP/out.rgb" -strip -quality 88 assets/room/portrait.jpg
test ! -d target/site/room || cp assets/room/portrait.jpg target/site/room/portrait.jpg
echo "→ assets/room/portrait.jpg"

# ASCII: 96 x 58 characters (a mono cell is 0.6 x 1 em, so that's square).
C=96; R=58
magick "$TMP/sq.png" -resize 460x460 -colorspace HSL -channel G -separate +channel -blur 0x2 -resize ${C}x${R}! -depth 8 "gray:$TMP/sat.gray"
magick "$TMP/sq.png" -resize 460x460 -colorspace gray -blur 0x2 -resize ${C}x${R}! -depth 8 "gray:$TMP/lum.gray"
magick "$TMP/sq.png" -resize 460x460 -colorspace gray -clahe 40x40%+128+2 -unsharp 0x2+1+0 -resize ${C}x${R}! -depth 8 "gray:$TMP/det.gray"
python3 -I scripts/ascii.py "$TMP/lum.gray" "$TMP/sat.gray" "$TMP/det.gray" $C $R 1.0 classic > crates/site/ascii/portrait.txt
python3 -I scripts/ascii.py "$TMP/lum.gray" "$TMP/sat.gray" "$TMP/det.gray" $C $R 1.0 block > crates/site/ascii/portrait-lens.txt
echo "→ crates/site/ascii/portrait.txt, portrait-lens.txt"
