# AGENTS.md

## Project Overview

`dnsmasqweb` is a single-binary Rust web UI for managing a narrow dnsmasq static DNS surface:

- `address=`
- `host-record=`
- `cname=`
- `server=`

Unknown dnsmasq directives, comments, and blank lines must be preserved. Do not broaden the managed directive set unless explicitly requested.

## Architecture

- Backend binary: Axum + Tokio, built with Cargo feature `ssr`.
- Frontend: Leptos CSR, built to WASM with Trunk using Cargo feature `csr`.
- Frontend/backend calls: Leptos server functions in `src/server_fns.rs`.
- Server function implementation calls shared backend services in `src/server/services.rs`.
- The final backend binary embeds the generated `dist/` frontend assets.

Build order matters:

```bash
env -u NO_COLOR trunk build --release --no-default-features --features csr
cargo build --release --bin dnsmasqweb --features ssr
```

## Cargo Features

- `ssr`: server-side binary, Axum, Tokio, dnsmasq/systemd operations, Leptos server functions.
- `csr`: browser-side WASM frontend, Leptos UI, browser storage APIs.
- Default feature is `ssr`.

Do not use the old `server` feature name. Leptos server function macros expect the server-side feature to be named `ssr`.

## Module Conventions

- Do not add `mod.rs`; use modern module files like `src/server.rs` plus `src/server/*.rs`.
- Keep dnsmasq parsing/rendering in `src/config/*`.
- Keep backend side effects in server/storage/dnsmasq modules, not UI modules.
- Keep UI-only state out of API/config models. For example, `EditableRecord<T>` row IDs are frontend-only and must not be serialized into dnsmasq config.

## API Boundary

Use Leptos server functions for frontend/backend calls. Do not add a parallel hand-written REST client unless explicitly requested.

- Server function declarations live in `src/server_fns.rs`.
- Shared backend logic lives in `src/server/services.rs`.
- Axum routes dynamically mount registered server function paths in `src/server/routes.rs`.

Authentication currently uses an in-memory bcrypt password hash and in-memory session tokens. Browser session tokens are stored in `localStorage`.

## Frontend Notes

- SCSS entry point is `style/main.scss`; split styles live under `style/base`, `style/layout`, `style/components`, and `style/pages`. Trunk compiles this entry through `data-trunk rel="scss"` in `index.html`.
- `index.html` is the Trunk entry and WASM bootstrap shell.
- i18n is intentionally lightweight and implemented in `src/i18n.rs`; avoid introducing a full i18n framework unless requested.
- Dynamic editable record lists should use keyed Leptos `<For/>` with stable UI-only IDs.

## Validation Commands

Run these after meaningful changes:

```bash
cargo fmt --all --check
cargo check --bin dnsmasqweb --features ssr
cargo check --bin dnsmasqweb-frontend --features csr --target wasm32-unknown-unknown --no-default-features
cargo test --tests --features ssr
cargo clippy --all-targets --features ssr -- -D warnings
cargo clippy --bin dnsmasqweb-frontend --features csr --target wasm32-unknown-unknown --no-default-features -- -D warnings
env -u NO_COLOR trunk build --release --no-default-features --features csr
cargo build --release --bin dnsmasqweb --features ssr
```

Use `env -u NO_COLOR` with Trunk because some environments set `NO_COLOR=1`, which can confuse Trunk's CLI parsing.

## Release Workflow

GitHub Actions builds Linux musl artifacts for:

- `x86_64-unknown-linux-musl`
- `aarch64-unknown-linux-musl`

Release builds are tag-triggered. Keep the workflow minimal unless there is a concrete release need.

## Safety Rules

- Never drop unknown dnsmasq lines.
- Always validate managed records before writing.
- Always run `dnsmasq --test --conf-file=...` against a temp file before replacing the real config.
- Preserve backup behavior before config replacement.
- Do not use panicking `unwrap()`/`expect()` in runtime paths.
