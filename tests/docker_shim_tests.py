"""Portable lifecycle tests: python3 tests/docker_shim_tests.py.

Run the actual shim with sandboxed paths, a fake daemon and a /proc fixture.
Real Linux /proc, dnsmasq and container logging still need the Docker smoke test.
"""

import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time
import unittest


REPO = Path(__file__).resolve().parents[1]


class ShimTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="dnsmasq-shim-")
        self.root = Path(self.temp.name)
        self.config = self.root / "dnsmasq.conf"
        self.pidfile = self.root / "dnsmasq.pid"
        self.active = self.root / "active.conf"
        self.daemon = self.root / "fake-dnsmasq"
        self.log = self.root / "daemon.log"
        self.config.write_text("old-address\n")
        self.daemon.write_text("""#!/bin/sh
for arg in "$@"; do
    case "$arg" in
        --conf-file=*) config=${arg#--conf-file=} ;;
        --pid-file=*) pidfile=${arg#--pid-file=} ;;
    esac
done
content=$(cat "$config")
[ "$content" != fail ] || exit 1
mkdir -p "$TEST_ROOT/proc/$$"
echo dnsmasq > "$TEST_ROOT/proc/$$/comm"
printf '%s\\000' "$0" "$@" > "$TEST_ROOT/proc/$$/cmdline"
trap 'rm -f "$TEST_ROOT/proc/$$/comm" "$TEST_ROOT/proc/$$/cmdline"' EXIT
if [ "$content" = stubborn ]; then
    trap '' TERM
else
    trap 'exit 0' TERM
fi
trap 'echo HUP >> "$TEST_ROOT/signals"' HUP
echo "$$" > "$pidfile"
echo "$$" >> "$TEST_ROOT/started"
cp "$config" "$TEST_ROOT/active.conf"
echo started
while :; do sleep 0.1; done
""")
        self.daemon.chmod(0o755)
        shim = (REPO / "docker/systemctl").read_text()
        shim = shim.replace("/usr/sbin/dnsmasq", str(self.daemon))
        shim = shim.replace("/run/dnsmasq.pid", str(self.pidfile))
        shim = shim.replace("/proc/1/fd/1", str(self.log))
        shim = shim.replace("/proc/1/fd/2", str(self.log))
        # macOS has no /proc; the fake daemon publishes only its own identity.
        shim = shim.replace("/proc/", str(self.root / "proc") + "/")
        self.shim = self.root / "systemctl"
        self.shim.write_text(shim)
        self.shim.chmod(0o755)
        self.env = dict(os.environ, TEST_ROOT=str(self.root),
                        DNSMASQWEB_CONFIG=str(self.config),
                        PATH=str(self.root) + os.pathsep + os.environ["PATH"])

    def tearDown(self):
        # Every PID was recorded by our fixture in its private directory.
        started = self.root / "started"
        if started.exists():
            for pid in started.read_text().splitlines():
                try:
                    args = subprocess.run(["ps", "-p", pid, "-o", "args="],
                                          capture_output=True, text=True).stdout
                    if str(self.daemon) in args:
                        os.kill(int(pid), signal.SIGKILL)
                except ProcessLookupError:
                    pass
        self.temp.cleanup()

    def run_shim(self, action, success=True, timeout=10):
        result = subprocess.run(["sh", str(self.shim), action, "dnsmasq"],
                                env=self.env, capture_output=True, text=True,
                                timeout=timeout)
        if success:
            self.assertEqual(result.returncode, 0, result.stderr)
        else:
            self.assertNotEqual(result.returncode, 0)
        return result

    def test_restart_reads_new_config_and_releases_capture_pipes(self):
        self.run_shim("start", timeout=4)
        old_pid = self.pidfile.read_text()
        self.config.write_text("new-address\n")
        self.run_shim("restart", timeout=8)
        self.assertNotEqual(old_pid, self.pidfile.read_text())
        self.assertEqual(self.active.read_text(), "new-address\n")
        self.assertIn("started", self.log.read_text())
        self.run_shim("stop")
        self.run_shim("is-active", success=False)

    def test_reload_does_not_apply_main_config(self):
        self.run_shim("reload", success=False)
        self.run_shim("start")
        old_pid = self.pidfile.read_text()
        self.config.write_text("new-address\n")
        self.run_shim("reload")
        time.sleep(0.2)
        self.assertEqual(old_pid, self.pidfile.read_text())
        self.assertEqual(self.active.read_text(), "old-address\n")
        self.assertIn("HUP", (self.root / "signals").read_text())
        self.run_shim("force-reload")
        self.assertEqual(self.active.read_text(), "new-address\n")

    def test_try_restart_does_not_start_inactive_service(self):
        self.run_shim("try-restart")
        self.assertFalse(self.pidfile.exists())
        self.run_shim("restart")
        old_pid = self.pidfile.read_text()
        self.run_shim("try-restart")
        self.assertNotEqual(old_pid, self.pidfile.read_text())

    def test_stop_timeout_does_not_start_a_second_daemon(self):
        self.config.write_text("stubborn\n")
        self.run_shim("start")
        old_pid = self.pidfile.read_text()
        self.config.write_text("new-address\n")
        result = self.run_shim("restart", success=False)
        self.assertIn("did not stop", result.stderr)
        self.assertEqual(old_pid, self.pidfile.read_text())
        self.assertEqual(len((self.root / "started").read_text().splitlines()), 1)

    def test_failed_start_allows_restoring_previous_config(self):
        self.run_shim("start")
        self.config.write_text("fail\n")
        result = self.run_shim("restart", success=False)
        self.assertIn("failed to start", result.stderr)
        self.run_shim("is-active", success=False)
        self.config.write_text("old-address\n")
        self.run_shim("restart")
        self.assertEqual(self.active.read_text(), "old-address\n")

    def test_stale_pid_and_unrelated_process_are_not_signalled(self):
        with subprocess.Popen(["sleep", "30"]) as unrelated:
            try:
                for pid in ["invalid", "0", "1", str(unrelated.pid)]:
                    self.pidfile.write_text(pid)
                    self.run_shim("is-active", success=False)
                    self.run_shim("stop")
                    self.assertIsNone(unrelated.poll())
                self.run_shim("start")
                self.assertNotEqual(int(self.pidfile.read_text()), unrelated.pid)
            finally:
                unrelated.terminate()

    def test_missing_config_fails_without_creating_it(self):
        self.config.unlink()
        self.run_shim("start", success=False)
        self.assertFalse(self.config.exists())

    def test_entrypoint_stops_on_daemon_start_failure(self):
        web = self.root / "dnsmasqweb"
        web.write_text('#!/bin/sh\ntouch "$TEST_ROOT/web-started"\n')
        web.chmod(0o755)
        entrypoint = self.root / "entrypoint.sh"
        entrypoint.write_text((REPO / "docker/entrypoint.sh").read_text()
                              .replace("/usr/local/bin/systemctl", str(self.shim))
                              .replace("/usr/local/bin/dnsmasqweb", str(web)))
        env = dict(self.env, DNSMASQWEB_BACKUP_DIR=str(self.root / "backups"),
                   DNSMASQWEB_CREDENTIALS_FILE=str(self.root / "credentials/password.hash"))
        self.config.write_text("fail\n")
        result = subprocess.run(["sh", str(entrypoint)], env=env,
                                capture_output=True, text=True, timeout=5)
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse((self.root / "web-started").exists())
        self.config.write_text("old-address\n")
        subprocess.run(["sh", str(entrypoint)], env=env, capture_output=True,
                       text=True, check=True, timeout=5)
        self.assertTrue((self.root / "web-started").exists())

    def test_pid_of_dnsmasq_using_another_config_is_not_signalled(self):
        other_config = self.root / "other.conf"
        other_pidfile = self.root / "other.pid"
        other_config.write_text("other-address\n")
        with subprocess.Popen([str(self.daemon), f"--conf-file={other_config}",
                               f"--pid-file={other_pidfile}"], env=self.env,
                              stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL) as other:
            try:
                for _ in range(50):
                    if other_pidfile.exists():
                        break
                    time.sleep(0.02)
                self.assertTrue(other_pidfile.exists())
                self.pidfile.write_text(str(other.pid))
                self.run_shim("stop")
                self.assertIsNone(other.poll())
                self.run_shim("is-active", success=False)
            finally:
                other.terminate()


if __name__ == "__main__":
    unittest.main()
