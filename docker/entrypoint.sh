#!/bin/sh
# Container entrypoint for the dnsmasqweb image.
#
# Starts dnsmasq in the foreground (as a background child of this shell, which is
# then replaced by dnsmasqweb via exec) so the systemctl shim can reload it with
# SIGHUP, then runs the web service. tini is PID 1 and reaps the children.

set -eu

config_file="${DNSMASQWEB_CONFIG:-/etc/dnsmasq.conf}"
backup_dir="${DNSMASQWEB_BACKUP_DIR:-/var/backups/dnsmasqweb}"
credentials_dir="${DNSMASQWEB_CREDENTIALS_FILE:-/var/lib/dnsmasqweb/password.hash}"
credentials_dir=$(dirname "$credentials_dir")

# Ensure runtime directories exist and the backups dir stays private. This also
# covers empty volume mounts that shadow the directories created at image build.
mkdir -p "$backup_dir" "$credentials_dir"
chmod 0700 "$backup_dir"

# The web service requires the config file to exist before it can load it.
if [ ! -f "$config_file" ]; then
    : > "$config_file"
fi

# Start dnsmasq if it is not already running (e.g. it was started by the
# systemctl shim before this entrypoint reached this point).
if ! pgrep -x dnsmasq >/dev/null 2>&1; then
    /usr/sbin/dnsmasq --keep-in-foreground \
        --conf-file="$config_file" \
        --pid-file=/run/dnsmasq.pid &
fi

exec /usr/local/bin/dnsmasqweb "$@"
