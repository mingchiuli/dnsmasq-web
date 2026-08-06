# dnsmasq-web

Small Rust web UI for managing a limited dnsmasq static DNS surface:

- `address=`
- `host-record=`
- `cname=`
- `server=`

Unknown directives, comments, and blank lines are preserved. They can still be
edited from the raw config editor.

The optional `dnsmasqweb managed records` block is parsed as a single explicit
BEGIN/END region. Comments, blank lines, and unknown directives inside that
region are preserved when structured records are saved. Unmatched, nested, or
duplicate managed blocks are rejected instead of being rewritten ambiguously.

## Scope

This project is intended for small deployments where dnsmasq runs directly on
the same Linux host, typically as a systemd service, and only a narrow static DNS
editing UI is needed.

Good fits include home lab gateways, small office DNS hosts, VPN DNS nodes, and
appliance-like machines that manage a local dnsmasq config file.

It is not a full DNS management platform, container orchestration layer, or
replacement for dnsmasq itself. The process needs local access to the config
file and permission to test and reload the dnsmasq service.

## Build

```bash
cargo install cargo-leptos --locked
rustup toolchain install 1.96.0 --component clippy,rustfmt --target wasm32-unknown-unknown
cargo leptos build --release
```

`cargo-leptos` builds the hydrated WASM frontend into `target/site` and the SSR
server binary.

The release binary is:

```text
target/release/dnsmasqweb
```

Normal local builds serve frontend assets from `site/` next to the binary when
present, or from `target/site`. Set `LEPTOS_SITE_ROOT` to override the asset
directory.

Official release binaries include the generated frontend assets and do not need
a separate `site/` directory. They still check the configured site directory
first, so an existing `LEPTOS_SITE_ROOT` can override an embedded asset. To build
the same standalone binary locally, generate the frontend first and then enable
`embedded-assets`:

```bash
cargo leptos build --release --frontend-only
LEPTOS_OUTPUT_NAME=dnsmasqweb cargo build --release --bin dnsmasqweb \
  --no-default-features --features ssr,embedded-assets
```

## Run

```bash
./dnsmasqweb \
  --config /etc/dnsmasq.conf \
  --backup-dir /var/backups/dnsmasqweb \
  --credentials-file /var/lib/dnsmasqweb/password.hash \
  --listen 127.0.0.1:8080
```

Options can also be set with environment variables:

```text
DNSMASQWEB_CONFIG
DNSMASQWEB_BACKUP_DIR
DNSMASQWEB_CREDENTIALS_FILE
DNSMASQWEB_LISTEN
DNSMASQWEB_DNSMASQ_BIN
DNSMASQWEB_SERVICE
DNSMASQWEB_DNSMASQ_TEST_TIMEOUT_SECS
DNSMASQWEB_SYSTEMCTL_TIMEOUT_SECS
DNSMASQWEB_MAX_BACKUPS
```

dnsmasq validation commands time out after 10 seconds by default; systemctl
status and restart commands time out after 30 seconds. Up to 50 backups are kept
after a successful save or restore. Set `DNSMASQWEB_MAX_BACKUPS=0` to keep all
backups. Failed transactions retain their rollback backup, and a cleanup failure
does not turn a successful config update into a failed one.

For production, bind to `127.0.0.1` or a private/VPN address.

On first browser access, set the admin password in the UI. The bcrypt password
hash is stored in `DNSMASQWEB_CREDENTIALS_FILE` using file mode `0600`; session
tokens remain in server memory. The browser receives the session token in a
`HttpOnly` `SameSite=Lax` cookie, which expires after 24 hours and becomes invalid
after the service restarts. The persisted password continues to apply after a
restart.

Login is limited per peer IP to 10 attempts in each 60-second window, and a
successful login resets that peer's window. Server function request bodies are
limited to 2 MiB.

## Permissions

The process needs permission to write the dnsmasq config file, create backups,
create and update the credentials file, and run:

```text
/usr/sbin/dnsmasq --test --conf-file=...
systemctl is-active dnsmasq
systemctl restart dnsmasq
```

Use `--dnsmasq-bin` and `--service` if your paths or service name differ.

The config path must resolve to a regular file. Symbolic links are supported and
remain in place; the linked file is replaced atomically. Config replacement
preserves Unix mode, owner, group, and extended attributes, synchronizes the
temporary file and parent directory, and cleans up temporary files on failure.
Backups are created as private regular files in a `0700` directory; backup
symbolic links and non-regular files are rejected.

Each editor load includes a content revision. Saving is rejected if the config
changed after it was loaded, including changes made while the dnsmasq validation
command was running. The editor keeps unsaved input so it can be reviewed before
refreshing.

## Systemd

```ini
[Unit]
Description=dnsmasq-web
After=network.target

[Service]
ExecStart=/usr/local/bin/dnsmasqweb \
  --config /etc/dnsmasq.conf \
  --backup-dir /var/backups/dnsmasqweb \
  --credentials-file /var/lib/dnsmasqweb/password.hash \
  --listen 127.0.0.1:8080
Restart=always

[Install]
WantedBy=multi-user.target
```
