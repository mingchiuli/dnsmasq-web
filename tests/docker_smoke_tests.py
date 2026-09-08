"""After docker build -t dnsmasqweb:test ., run:
DNSMASQWEB_TEST_IMAGE=dnsmasqweb:test python3 tests/docker_smoke_tests.py

Uses a disposable container, private config, and a randomly published loopback port.
"""

import os
import subprocess
import time
import unittest

from dns_behavior_tests import addresses, query


IMAGE = os.environ.get("DNSMASQWEB_TEST_IMAGE")
CONFIG = ("port=1053\nno-resolv\nno-hosts\n"
          "address=/app.example.test/10.10.0.1\nserver=/app.example.test/\n")


@unittest.skipUnless(IMAGE, "set DNSMASQWEB_TEST_IMAGE to the built image")
class DockerSmokeTests(unittest.TestCase):
    def setUp(self):
        # tini stays PID 1 and reaps the shim's background processes. Its log
        # descriptors match the production entrypoint's /proc/1/fd setup.
        self.container = subprocess.check_output(
            ["docker", "run", "--detach", "--publish", "127.0.0.1::1053/udp",
             "--env", "DNSMASQWEB_CONFIG=/tmp/test.conf", "--entrypoint", "tini",
             IMAGE, "--", "sleep", "300"], text=True).strip()
        self.addCleanup(subprocess.run, ["docker", "rm", "--force", self.container],
                        check=True, capture_output=True, timeout=15)
        mapping = self.exec_docker("port", self.container, "1053/udp").stdout.strip()
        self.port = int(mapping.rsplit(":", 1)[1])
        self.write_config(CONFIG)

    def exec_docker(self, *args, **kwargs):
        return subprocess.run(["docker", *args], capture_output=True, text=True,
                              timeout=15, check=True, **kwargs)

    def write_config(self, content):
        self.exec_docker("exec", "-i", self.container, "tee", "/tmp/test.conf", input=content)

    def shim(self, action, success=True):
        result = subprocess.run(["docker", "exec", self.container, "systemctl", action, "dnsmasq"],
                                capture_output=True, text=True, timeout=12)
        if success:
            self.assertEqual(result.returncode, 0, result.stderr)
        else:
            self.assertNotEqual(result.returncode, 0)
        return result

    def pid(self):
        return self.exec_docker("exec", self.container, "cat", "/run/dnsmasq.pid").stdout.strip()

    def test_restart_loads_addresses_and_local_only_rules(self):
        self.shim("start")
        initial_pid = self.pid()
        self.assertEqual(addresses(query(self.port, "app.example.test", 1)), ["10.10.0.1"])
        self.assertEqual(addresses(query(self.port, "app.example.test", 28)), [])
        changed = CONFIG.replace("10.10.0.1", "10.10.0.2") + "address=/app.example.test/fd00::1\n"
        self.write_config(changed)
        self.shim("reload")
        self.assertEqual(self.pid(), initial_pid)
        self.assertEqual(addresses(query(self.port, "app.example.test", 1)), ["10.10.0.1"])
        self.shim("restart")
        self.assertNotEqual(self.pid(), initial_pid)
        self.assertEqual(addresses(query(self.port, "app.example.test", 1)), ["10.10.0.2"])
        self.assertEqual(addresses(query(self.port, "app.example.test", 28)), ["fd00::1"])

    def test_failure_can_be_followed_by_backup_restore(self):
        self.shim("start")
        self.write_config("invalid-directive-for-test\n")
        self.shim("restart", success=False)
        self.shim("is-active", success=False)
        self.write_config(CONFIG)
        self.shim("restart")
        self.assertEqual(addresses(query(self.port, "app.example.test", 1)), ["10.10.0.1"])

    def test_stop_timeout_reports_failure_without_starting_another_process(self):
        self.shim("start")
        pid = self.pid()
        self.exec_docker("exec", self.container, "kill", "-STOP", pid)
        try:
            result = self.shim("restart", success=False)
            self.assertIn("did not stop", result.stderr)
            self.assertEqual(self.pid(), pid)
        finally:
            self.exec_docker("exec", self.container, "kill", "-CONT", pid)
        time.sleep(1)
        self.shim("restart")


if __name__ == "__main__":
    unittest.main()
