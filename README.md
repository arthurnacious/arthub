# ARTHUB

Portfolio of Arthurnacious Monethi. Rust end to end: a Leptos site (SSR + hydration) served by a NestForge/Axum backend, and a Bevy game room compiled to WebAssembly.

## Run it

```sh
rustup target add wasm32-unknown-unknown   # once
cargo install cargo-leptos just --locked   # once
cp .env.example .env                       # optional: email settings for the contact form
just dev                                   # builds the room if needed, then http://127.0.0.1:3000
```

| command | does |
|---|---|
| `just dev` | room bundle (no-op if unchanged) + `cargo leptos watch` |
| `just build` | production build: room bundle + `cargo leptos build --release` |
| `just test` | every crate's tests, including the server's per-module route tests |
| `just check` | fmt, clippy (native and wasm targets) |
| `just build-room` | rebuild the game room WASM (safe while the site is running) |
| `just room` | run the game room natively |
| `just test-email` | send one real email through the configured provider |
| `just fetch-textures` / `just bake-textures` | refresh the room's PBR textures |
| `just portrait photo.jpg` | regenerate both portraits: the low-poly painting in the room and the ASCII one on the home page |

## Layout

```
crates/
  site/          Leptos app: pages, components. Built natively with `ssr`
                 (into the server) and as wasm with `hydrate`.
  server/        The backend, as NestForge modules (below).
  content/       content/ → compiled-in data (build-time loader + validator).
  contact/       Contact form rules (shared with the browser) + spam checks + mail.
  game_room/     Bevy room (separate wasm bundle, lazy-loaded).
  pool_physics/  Pool simulation, tested natively.
  bridge/        Site ↔ room event types.
  texbake/       Offline texture baker (KTX2, BC + ETC2).
content/         Projects, about, experience, skills (see CONTENT.md)
assets/          Served at the site root: fonts, media, room bundle + textures
style/           main.css
```

## Backend modules

The server is a [NestForge](https://github.com/vernonthedev/nestforge) app (NestJS-style modules and dependency injection on Axum) with the Leptos router mounted alongside. Shared services are providers resolved from the container, not globals.

| module | routes | provides |
|---|---|---|
| `CoreModule` | `GET /healthz` | `AppConfig` (env), logging |
| `ContentModule` | `GET /api/content/{projects, projects/{slug}, skills, experience, about}` | `ContentRepository` |
| `ContactModule` | `POST /api/contact`, `GET /api/contact/token` | `ContactService` (validation, spam checks, mailer) |
| `RoomModule` | `GET /api/room/featured`, `/room/bundle/*` (pre-compressed) | |
| `SeoModule` | `GET /robots.txt`, `GET /sitemap.xml` (built from content) | |
| `AuthModule`, `AdminModule` | (empty, for the dashboard) | |

Everything else (every page, Leptos server functions at `POST /api/{fn}`, static files) is the Leptos router in `crates/server/src/app.rs`. To add a module: create `crates/server/src/<name>/mod.rs` with a `#[module(...)]`, add it to `AppModule`'s `imports`. NestForge's route macro wants an OpenAPI schema for every handler's return type; return `core::JsonBody` / `core::RawResponse` (or `String`) rather than raw `Json<Value>`/`Response`.

## Deploying

One binary plus the `target/site` folder.

```sh
just build                                   # → target/release/server + target/site/
```

Run it with:

| variable | |
|---|---|
| `LEPTOS_ENV=PROD` | production mode: secure cookies; refuses to start without email config |
| `LEPTOS_SITE_ADDR=0.0.0.0:8080` | listen address |
| `LEPTOS_SITE_ROOT=site` | path to the copied `target/site` folder |
| `LEPTOS_OUTPUT_NAME=arthub` | must match the build |
| `CONTACT_TO`, `CONTACT_FROM`, + SMTP or API key | see `.env.example` |
| `TRUST_PROXY=1` | behind Fly/Railway/nginx, so per-IP rate limits see real clients |
| `SITE_URL=https://your.domain` | public origin for canonical links, share images and the sitemap |

Or as a container (the image builds the room, the site and the server, then ships only the binary and `site/`):

```sh
docker build -t arthub .
docker run -p 8080:8080 --env-file .env -e SITE_URL=https://your.domain arthub
```

Health check: `GET /healthz` → `200 ok`. The server binds the address itself (it doesn't use NestForge's `listen()`, which is 127.0.0.1-only) and passes client addresses through for rate limiting. Serve `/pkg/*`, `/room/bundle/*` and `/media/*` with long cache lifetimes if a CDN fronts it; the room bundle already ships `.br`/`.gz` siblings.

## Design tokens

All in `:root` in `style/main.css`. `--accent` is a neutral chalk off-white used as an inverted fill. Change that one value to re-skin the site. Neon tones (`--neon-pink`, `--neon-cyan`) are reserved for the game room.

## Cursor states

Add `data-cursor="view" | "drag" | "shoot" | "link"` to any element. Links and buttons get `link` automatically. The cursor only runs for fine pointers, and the trailing ring snaps to the pointer instead of easing under `prefers-reduced-motion`.
