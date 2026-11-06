#!/usr/bin/env bash
# Build the Bevy game room for the browser into assets/room/bundle/.
#
# assets/ is cargo-leptos's assets-dir, so every `cargo leptos build/watch`
# copies it to target/site/ and the site serves it at /room/bundle/, which is
# where the site's loader fetches it (see crates/site/src/room.rs).
#
# Safe to run while the site is being served: everything is produced in a
# staging folder and swapped in file by file with atomic renames, so the live
# bundle is never missing or half-written. If the compiled wasm hasn't changed
# since the last run, the slow post-processing is skipped.
set -euo pipefail
cd "$(dirname "$0")/.."

PROFILE="${PROFILE:-wasm-release}"
OUT=assets/room/bundle
LIVE=target/site/room/bundle   # cargo-leptos's copy, if a site build exists
TARGET_DIR=target/room
STAGE="$TARGET_DIR/stage"

# wasm-bindgen CLI must match the wasm-bindgen crate version in Cargo.lock.
WB_VERSION=$(grep -A1 '^name = "wasm-bindgen"$' Cargo.lock | sed -n 's/version = "\(.*\)"/\1/p')

find_tool() { # name, fallback glob under ~/.cache/cargo-leptos
  if command -v "$1" >/dev/null && [[ "$1" != wasm-bindgen || "$("$1" --version)" == *"$WB_VERSION"* ]]; then
    command -v "$1"
  else
    compgen -G "$HOME/.cache/cargo-leptos/$2" | head -1 || true
  fi
}
WASM_BINDGEN=$(find_tool wasm-bindgen "wasm-bindgen-$WB_VERSION/*/*/wasm-bindgen")
WASM_OPT=$(find_tool wasm-opt "wasm-opt-*/*/*/bin/wasm-opt")
if [[ -z "$WASM_BINDGEN" ]]; then
  echo "wasm-bindgen $WB_VERSION not found: cargo install wasm-bindgen-cli --version $WB_VERSION" >&2
  exit 1
fi

echo "→ cargo build (profile $PROFILE)"
cargo build -p game_room --lib --target wasm32-unknown-unknown \
  --profile "$PROFILE" --target-dir "$TARGET_DIR"

PROFILE_DIR=$([[ "$PROFILE" == dev ]] && echo debug || echo "$PROFILE")
WASM="$TARGET_DIR/wasm32-unknown-unknown/$PROFILE_DIR/game_room.wasm"
STAMP="$(sha256sum "$WASM" | cut -d' ' -f1)-$PROFILE"

if [[ -f "$OUT/.source" && "$(cat "$OUT/.source")" == "$STAMP" && -f "$OUT/game_room_bg.wasm" ]]; then
  echo "→ room unchanged, bundle is current"
else
  rm -rf "$STAGE" && mkdir -p "$STAGE"

  echo "→ wasm-bindgen $WB_VERSION"
  "$WASM_BINDGEN" --target web --no-typescript --out-dir "$STAGE" "$WASM"

  if [[ -n "$WASM_OPT" && "$PROFILE" != dev ]]; then
    echo "→ wasm-opt -Oz"
    "$WASM_OPT" -Oz --enable-bulk-memory --enable-nontrapping-float-to-int \
      --enable-sign-ext --enable-reference-types --enable-multivalue --enable-mutable-globals \
      "$STAGE/game_room_bg.wasm" -o "$STAGE/game_room_bg.wasm"
  else
    echo "! wasm-opt skipped"
  fi

  # The loader reads this to show true progress even when the response is
  # compressed (Content-Length is then the compressed size).
  printf '{"wasm_bytes":%d}\n' "$(stat -c%s "$STAGE/game_room_bg.wasm")" > "$STAGE/manifest.json"

  # Pre-compressed copies for the server to hand out as-is.
  for f in "$STAGE"/*.wasm "$STAGE"/*.js; do
    brotli -f -q 11 -o "$f.br" "$f"
    gzip -f -k -9 "$f"
  done
  echo "$STAMP" > "$STAGE/.source"
fi

# Swap the staged files into place: copy beside the target, then rename over
# it (atomic on one filesystem). The .source stamp goes last so an
# interrupted swap is redone next time.
publish() { # from_dir to_dir
  local from=$1 to=$2 f
  mkdir -p "$to"
  for f in $(cd "$from" && ls -A | grep -vx '.source'; echo .source); do
    [[ -f "$from/$f" ]] || continue
    cp "$from/$f" "$to/.$f.tmp" && mv -f "$to/.$f.tmp" "$to/$f"
  done
  # Drop files that are no longer part of the bundle.
  for f in $(cd "$to" && ls -A); do
    [[ -e "$from/$f" || "$f" == .*.tmp ]] || rm -f "$to/$f"
  done
}

if [[ -d "$STAGE" && -f "$STAGE/.source" && "$(cat "$STAGE/.source")" == "$STAMP" ]]; then
  publish "$STAGE" "$OUT"
fi
# A running `cargo leptos watch` doesn't pick up a changed asset folder
# reliably, so keep its served copy in step too.
if [[ -d target/site ]]; then
  publish "$OUT" "$LIVE"
fi

raw=$(stat -c%s "$OUT/game_room_bg.wasm")
br=$(stat -c%s "$OUT/game_room_bg.wasm.br")
awk -v r="$raw" -v b="$br" 'BEGIN { printf "room wasm: %.2f MB raw, %.2f MB brotli\n", r/1048576, b/1048576 }'
