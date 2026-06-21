# dnsmasq-web

Small Rust web UI for managing a limited dnsmasq static DNS surface:

- `address=`
- `host-record=`
- `cname=`
- `server=`

Unknown directives, comments, and blank lines are preserved. They can still be
edited from the raw config editor.

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
cargo install trunk --locked
rustup toolchain install 1.96.0 --component clippy,rustfmt --target wasm32-unknown-unknown
env -u NO_COLOR trunk build --release --no-default-features --features csr
cargo build --release --bin dnsmasqweb --features ssr
```

Build the frontend first. The server binary embeds the generated `dist/`
frontend assets.

The release binary is:

```text
target/release/dnsmasqweb
```

## Run

```bash
./dnsmasqweb \
  --config /etc/dnsmasq.conf \
  --backup-dir /var/backups/dnsmasqweb \
  --listen 127.0.0.1:8080
```

Options can also be set with environment variables:

```text
DNSMASQWEB_CONFIG
DNSMASQWEB_BACKUP_DIR
DNSMASQWEB_LISTEN
DNSMASQWEB_DNSMASQ_BIN
DNSMASQWEB_SERVICE
```

For production, bind to `127.0.0.1` or a private/VPN address.

On first browser access after startup, set the admin password in the UI. The
password hash and session tokens are kept in server memory. The browser receives
the session token in a `HttpOnly` `SameSite=Lax` cookie, which expires after 24
hours and becomes invalid after the service restarts.

## Permissions

The process needs permission to write the dnsmasq config file, create backups,
and run:

```text
/usr/sbin/dnsmasq --test --conf-file=...
systemctl reload dnsmasq
systemctl restart dnsmasq
```

Use `--dnsmasq-bin` and `--service` if your paths or service name differ.

## Systemd

```ini
[Unit]
Description=dnsmasq-web
After=network.target

[Service]
ExecStart=/usr/local/bin/dnsmasqweb \
  --config /etc/dnsmasq.conf \
  --backup-dir /var/backups/dnsmasqweb \
  --listen 127.0.0.1:8080
Restart=always

[Install]
WantedBy=multi-user.target
```
