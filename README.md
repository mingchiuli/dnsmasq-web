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

At runtime, the server serves frontend assets from `site/` next to the binary
when present, or from `target/site` during local builds. Set `LEPTOS_SITE_ROOT`
to override the asset directory.

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
```

For production, bind to `127.0.0.1` or a private/VPN address.

On first browser access, set the admin password in the UI. The bcrypt password
hash is stored in `DNSMASQWEB_CREDENTIALS_FILE` using file mode `0600`; session
tokens remain in server memory. The browser receives the session token in a
`HttpOnly` `SameSite=Lax` cookie, which expires after 24 hours and becomes invalid
after the service restarts. The persisted password continues to apply after a
restart.

## Permissions

The process needs permission to write the dnsmasq config file, create backups,
create and update the credentials file, and run:

```text
/usr/sbin/dnsmasq --test --conf-file=...
systemctl reload dnsmasq
systemctl restart dnsmasq
```

Use `--dnsmasq-bin` and `--service` if your paths or service name differ.

The config path must resolve to a regular file. Symbolic links are supported and
remain in place; the linked file is replaced atomically. Config replacement
preserves Unix mode, owner, group, and extended attributes, synchronizes the
temporary file and parent directory, and cleans up temporary files on failure.

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
