# dnsmasqweb

Rust + Leptos + Axum based dnsmasq configuration UI for a narrow, maintainable
static DNS surface:

- `address=`
- `host-record=`
- `cname=`
- `server=`

Unknown directives, comments and blank lines are preserved. They are visible in
the raw editor, but are not exposed in the classified forms.

## Scope

dnsmasqweb is intended for hosts that already run dnsmasq directly on the
machine, typically as a systemd service, and only need a small web UI for editing
the local dnsmasq config, testing it, backing it up, and reloading dnsmasq.

It is a good fit for non-container deployments such as home lab gateways, small
office DNS hosts, WireGuard/VPN DNS nodes, or appliance-like Linux machines
where dnsmasq owns `/etc/dnsmasq.conf`.

It is not designed to be a container orchestration layer, a full DNS management
platform, or a replacement for running dnsmasq itself. The server process expects
local filesystem access to the config file and permission to execute
`dnsmasq --test` plus `systemctl reload/restart dnsmasq`.

The frontend calls backend operations through Leptos server functions mounted on
Axum, so configuration actions are defined as Rust functions instead of a
separate hand-written REST client.

## Build

Local release build:

```bash
cargo install trunk --locked
rustup target add wasm32-unknown-unknown
env -u NO_COLOR trunk build --release --no-default-features --features csr
cargo build --release --bin dnsmasqweb --features ssr
```

This builds the Leptos WASM frontend into `dist/`, then embeds that directory
into the Axum server binary:

```text
target/release/dnsmasqweb
```

Run the commands in this order. The server binary embeds files from `dist/`, so
building the server before `trunk build` will not include the latest frontend
assets.

Trunk normally downloads the matching `wasm-bindgen` CLI automatically. In a
restricted network, install the CLI version requested by Trunk once and rerun the
build.

## GitHub Release

Push a tag to build release artifacts:

```bash
git tag v1.0.0
git push origin v1.0.0
```

The GitHub Actions workflow builds Linux musl `x86_64` and `aarch64` tarballs
and uploads them to the GitHub Release.

## Run

```bash
./dnsmasqweb \
  --config /etc/dnsmasq.conf \
  --backup-dir /var/backups/dnsmasqweb \
  --listen 10.10.0.1:8080
```

Recommended production setup is to bind only to `127.0.0.1` or the WireGuard
address, not a public interface.

On first browser access after the service starts, set the admin password in the
web UI. The bcrypt password hash and issued session tokens are kept only in
server memory. Login tokens are valid for 24 hours, are stored by the browser in
`localStorage`, and become invalid immediately when the service restarts.

## Systemd

```ini
[Unit]
Description=dnsmasqweb
After=network.target

[Service]
ExecStart=/usr/local/bin/dnsmasqweb \
  --config /etc/dnsmasq.conf \
  --backup-dir /var/backups/dnsmasqweb \
  --listen 10.10.0.1:8080
Restart=always

[Install]
WantedBy=multi-user.target
```

The service needs permission to write `/etc/dnsmasq.conf` and run:

```text
/usr/sbin/dnsmasq --test --conf-file=...
systemctl reload dnsmasq
systemctl restart dnsmasq
```
