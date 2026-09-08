#!/bin/sh
# Container entrypoint for the dnsmasqweb image.
#
# Starts dnsmasq through the same checked lifecycle used when applying config,
# then runs the web service. tini is PID 1 and reaps the background children.

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

# set -e prevents the web service from starting if dnsmasq cannot start.
/usr/local/bin/systemctl start dnsmasq

exec /usr/local/bin/dnsmasqweb "$@"
