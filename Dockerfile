# Production image: one server binary plus the built site folder.
#
#   docker build -t arthub .
#   docker run -p 8080:8080 --env-file .env -e SITE_URL=https://your.domain arthub
#
# Building needs ~6 GB of RAM (the Bevy room and the Leptos server are both
# large compiles); CARGO_BUILD_JOBS keeps peak memory down.

FROM rust:1-bookworm AS build
ARG CARGO_BUILD_JOBS=4
ENV CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS}
RUN apt-get update \
 && apt-get install -y --no-install-recommends binaryen \
 && rm -rf /var/lib/apt/lists/*
RUN rustup target add wasm32-unknown-unknown \
 && cargo install cargo-leptos --version 0.3.11 --locked
WORKDIR /app
# wasm-bindgen's CLI must match the crate version in Cargo.lock.
COPY Cargo.toml Cargo.lock ./
RUN cargo install wasm-bindgen-cli --locked --version \
    "$(grep -A1 '^name = "wasm-bindgen"$' Cargo.lock | sed -n 's/version = "\(.*\)"/\1/p')"
COPY . .
RUN ./scripts/build-room.sh \
 && cargo leptos build --release

FROM debian:bookworm-slim
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --system --home /app arthub
WORKDIR /app
COPY --from=build /app/target/release/server /app/server
COPY --from=build /app/target/site /app/site
USER arthub
ENV LEPTOS_ENV=PROD \
    LEPTOS_OUTPUT_NAME=arthub \
    LEPTOS_SITE_ROOT=site \
    LEPTOS_SITE_ADDR=0.0.0.0:8080 \
    RUST_LOG=info
EXPOSE 8080
CMD ["/app/server"]
