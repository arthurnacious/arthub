# Arthub task runner. `just --list` for everything.

# Build the room (a no-op if unchanged), then run the site with live reload.
# After editing crates/game_room, run `just build-room` in another terminal;
# it swaps the new bundle in under the running server.
dev: build-room
    cargo leptos watch

# Like dev, but reachable from phones etc. on the network (allow ports 3000-3001).
dev-lan: build-room
    LEPTOS_SITE_ADDR=0.0.0.0:3000 cargo leptos watch

# Build the game room WASM bundle into assets/room/bundle/.
build-room:
    ./scripts/build-room.sh

# Fast unoptimized room build for iterating in the browser.
build-room-dev:
    PROFILE=dev ./scripts/build-room.sh

# Run the game room natively.
room *args:
    cargo run -p game_room --release -- {{args}}

# Screenshot every hotspot, play a break, measure fps (writes to target/tour/).
tour:
    ARTHUB_TOUR=target/tour cargo run -p game_room --release

# Production build of everything.
build: build-room
    cargo leptos build --release

test:
    cargo test -p bridge -p pool_physics -p content -p contact -p server --features content/parse,contact/server
    cargo test -p game_room --lib

check:
    cargo fmt --all --check
    cargo clippy -p server -p game_room -p bridge -p pool_physics -p content -p contact -p texbake --all-targets --features content/parse,contact/server -- -D warnings
    cargo clippy -p site --features hydrate --target wasm32-unknown-unknown -- -D warnings
    cargo clippy -p game_room --lib --target wasm32-unknown-unknown --target-dir target/room -- -D warnings

# Download the CC0 source textures (art/, not committed).
fetch-textures:
    ./scripts/fetch-textures.sh

# Bake art/ into compressed KTX2 in assets/room/tex/ (BC + ETC2, mipmapped).
bake-textures:
    ./scripts/make-ball-atlas.sh
    cargo run --release -p texbake
    test ! -d target/site || (mkdir -p target/site/room && rm -rf target/site/room/tex && cp -r assets/room/tex target/site/room/tex)

# Send one real email through the configured provider (reads .env).
test-email:
    cargo run -q -p server --bin test_email

# Both portraits from one photo (room painting + home ASCII): just portrait photo.jpg
portrait photo:
    ./scripts/make-portrait.sh {{photo}}
