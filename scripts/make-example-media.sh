#!/usr/bin/env bash
# Generate the placeholder media for the example projects in content/.
# Delete the examples (content/projects/*.md with `example = true` and their
# assets/media/work/<slug>/ folders) once you've added real work.
# Needs ImageMagick 7 (`magick`) and ffmpeg with libx264.
set -euo pipefail
cd "$(dirname "$0")/.."

FONT_BLACK=/usr/share/fonts/noto/NotoSans-Black.ttf
FONT_MONO=/usr/share/fonts/noto/NotoSansMono-Regular.ttf

art() { # out w h seed c1 c2 big small
  local out=$1 w=$2 h=$3 seed=$4 c1=$5 c2=$6 big=$7 small=$8
  local pt=$(( w / 11 ))
  magick -size "${w}x${h}" -seed "$seed" "plasma:${c1}-${c2}" -blur 0x28 -modulate 75,150 \
    \( -size "${w}x${h}" xc:none -stroke 'rgba(255,255,255,0.07)' -strokewidth 2 \
       -draw "$(for x in $(seq 0 80 "$w"); do printf 'line %d,0 %d,%d ' "$x" "$x" "$h"; done)" \
       -draw "$(for y in $(seq 0 80 "$h"); do printf 'line 0,%d %d,%d ' "$y" "$w" "$y"; done)" \) -composite \
    -fill 'rgba(10,10,10,0.45)' -draw "rectangle 0,$(( h * 7 / 10 )) $w,$h" \
    -font "$FONT_BLACK" -pointsize "$pt" -fill '#f2f1ec' -gravity southwest -annotate "+$(( w / 22 ))+$(( h / 9 ))" "$big" \
    -font "$FONT_MONO" -pointsize $(( pt / 5 )) -fill '#e4dfd1' -annotate "+$(( w / 22 + 6 ))+$(( h / 22 ))" "$small" \
    -quality 82 "$out"
}

video() { # out seed c0 c1 title
  ffmpeg -loglevel error -y -f lavfi \
    -i "gradients=s=960x600:c0=$3:c1=$4:c2=#0a0a0a:n=3:speed=0.015:duration=6:rate=30:seed=$2" \
    -vf "drawtext=fontfile=$FONT_BLACK:text='$5':fontcolor=0xf2f1ec:fontsize=88:x=48:y=h-170,drawtext=fontfile=$FONT_MONO:text='EXAMPLE PREVIEW LOOP':fontcolor=0xe4dfd1:fontsize=22:x=52:y=h-62" \
    -c:v libx264 -crf 30 -preset slow -pix_fmt yuv420p -movflags +faststart -an "$1"
}

project() { # slug seed c1 c2 TITLE
  local slug=$1 seed=$2 c1=$3 c2=$4 title=$5 d="assets/media/work/$1"
  mkdir -p "$d"
  art "$d/cover.jpg"      1600 1000 "$seed"         "$c1" "$c2" "$title" "EXAMPLE PROJECT · REPLACE WITH YOUR WORK"
  art "$d/process-1.webp" 1600 1000 $(( seed + 1 )) "$c2" "$c1" "PROCESS 01" "EXAMPLE · SKETCHES AND WIREFRAMES"
  art "$d/process-2.webp" 1200 1500 $(( seed + 2 )) "$c1" "#0a0a0a" "BUILD" "EXAMPLE · WORK IN PROGRESS"
  art "$d/outcome.webp"   1600 900  $(( seed + 3 )) "$c2" "#0a0a0a" "OUTCOME" "EXAMPLE · FINAL RESULT"
  video "$d/preview.mp4" "$seed" "$c1" "$c2" "$title"
}

project neon-city     11 '#ff2e88' '#3a0ca3' 'NEON CITY'
project render-farm   23 '#22e6ff' '#023047' 'RENDER FARM'
project pocket-ledger 37 '#ffb020' '#5f0f40' 'POCKET LEDGER'
project signal-mesh   41 '#8c5cff' '#0b3d2e' 'SIGNAL MESH'

mkdir -p assets/media/about
magick -size 1200x1500 -seed 7 plasma:'#2a2a28-#0a0a0a' -blur 0x30 \
  -fill '#1b1b1a' -draw 'circle 600,560 600,800' -draw 'ellipse 600,1500 420,520 180,360' \
  -font "$FONT_MONO" -pointsize 34 -fill '#e4dfd1' -gravity south -annotate +0+60 'PORTRAIT PLACEHOLDER · 1200×1500' \
  -quality 82 assets/media/about/portrait.jpg

du -sh assets/media/work/* assets/media/about
