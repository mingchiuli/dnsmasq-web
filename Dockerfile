# syntax=docker/dockerfile:1

# Builds the standalone dnsmasqweb binary (frontend assets embedded) and packages
# it together with dnsmasq into a single image. See README.md "Docker".

FROM rust:1.96-bookworm AS builder

# rust-toolchain.toml is intentionally not copied into the builder so rustup does
# not install the extra components it lists (clippy, rust-analyzer, rust-src).
# The rust:1.96-bookworm tag pins the Rust version instead.
RUN rustup target add wasm32-unknown-unknown
RUN cargo install cargo-leptos --version 0.3.6 --locked

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY assets ./assets
COPY style ./style

# Download dependencies as its own layer so source changes do not re-fetch.
RUN cargo fetch

# Generate the hydrated WASM frontend into target/site (site-root in Cargo.toml).
RUN cargo leptos build --release --frontend-only

# Build the SSR server binary with the frontend assets embedded via rust-embed
# (EmbeddedAssets reads target/site/).
ENV LEPTOS_OUTPUT_NAME=dnsmasqweb
RUN cargo build --release --bin dnsmasqweb \
    --no-default-features --features ssr,embedded-assets

FROM debian:bookworm-slim

# dnsmasq: the daemon itself and the binary used for `dnsmasq --test` validation.
# procps: pgrep/pkill used by the systemctl shim.
# curl: HTTP probe for healthchecks users define (e.g. in Docker Compose).
# tini: init/process reaper so signals reach the foreground processes.
RUN apt-get update && apt-get install -y --no-install-recommends \
        dnsmasq procps curl tini ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/dnsmasqweb /usr/local/bin/dnsmasqweb
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
COPY docker/systemctl /usr/local/bin/systemctl
RUN chmod +x /usr/local/bin/dnsmasqweb /usr/local/bin/entrypoint.sh /usr/local/bin/systemctl

ENV DNSMASQWEB_CONFIG=/etc/dnsmasq.conf \
    DNSMASQWEB_BACKUP_DIR=/var/backups/dnsmasqweb \
    DNSMASQWEB_CREDENTIALS_FILE=/var/lib/dnsmasqweb/password.hash \
    DNSMASQWEB_LISTEN=0.0.0.0:8080 \
    DNSMASQWEB_DNSMASQ_BIN=/usr/sbin/dnsmasq \
    DNSMASQWEB_SERVICE=dnsmasq

EXPOSE 8080 53/tcp 53/udp

ENTRYPOINT ["tini", "--", "/usr/local/bin/entrypoint.sh"]
