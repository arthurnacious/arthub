#!/usr/bin/env bash
# Download the CC0 source textures (Poly Haven) the game room is baked from
# into art/textures/<asset>/. Sources are not committed; the baked KTX2 files
# in assets/room/tex/ are. Re-run, then `just bake-textures`, to change them.
set -euo pipefail
cd "$(dirname "$0")/.."

# asset  resolution
SETS=(
  "dark_wooden_planks 2k"
  "red_brick_03 1k"
  "painted_plaster_wall 1k"
  "velour_velvet 2k"
  "dark_wood 1k"
  "brown_leather 1k"
)

for entry in "${SETS[@]}"; do
  set -- $entry; id=$1 res=$2; dir="art/textures/$id"
  mkdir -p "$dir"
  json=$(curl -sf "https://api.polyhaven.com/files/$id")
  for map in Diffuse nor_gl Rough; do
    url=$(python3 -c "import json,sys; d=json.loads(sys.argv[1]); print(d['$map']['$res']['jpg']['url'])" "$json" 2>/dev/null || \
          python3 -c "import json,sys; d=json.loads(sys.argv[1]); print(d['$map']['$res']['png']['url'])" "$json")
    ext=${url##*.}
    [[ -f "$dir/$map.$ext" ]] || curl -sfL -o "$dir/$map.$ext" "$url"
  done
  curl -sf "https://api.polyhaven.com/info/$id" > "$dir/info.json"
  echo "$id ($res): $(ls "$dir" | tr '\n' ' ')"
done
