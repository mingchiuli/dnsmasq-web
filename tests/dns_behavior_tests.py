"""Real DNS checks: DNSMASQ_BIN=/path/to/dnsmasq python3 tests/dns_behavior_tests.py.

Only uses temporary configs, loopback high ports, and a controlled local upstream.
"""

import contextlib
import os
from pathlib import Path
import shutil
import socket
import struct
import subprocess
import tempfile
import threading
import time
import unittest


DNSMASQ = os.environ.get("DNSMASQ_BIN") or shutil.which("dnsmasq")


def query(port, name, kind):
    question = b"".join(bytes([len(label)]) + label.encode() for label in name.split("."))
    packet = struct.pack("!6H", 42, 0x100, 1, 0, 0, 0) + question + b"\0" + struct.pack("!2H", kind, 1)
    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as client:
        client.settimeout(0.5)
        client.sendto(packet, ("127.0.0.1", port))
        return client.recv(4096)


def skip_name(packet, offset):
    while packet[offset]:
        if packet[offset] & 0xC0 == 0xC0:
            return offset + 2
        offset += 1 + packet[offset]
    return offset + 1


def addresses(packet):
    _, flags, questions, answers, _, _ = struct.unpack("!6H", packet[:12])
    if flags & 15:
        raise AssertionError(f"DNS error code: {flags & 15}")
    offset = 12
    for _ in range(questions):
        offset = skip_name(packet, offset) + 4
    result = []
    for _ in range(answers):
        offset = skip_name(packet, offset)
        kind, _, _, size = struct.unpack("!HHIH", packet[offset:offset + 10])
        offset += 10
        if kind in (1, 28):
            result.append(socket.inet_ntop(socket.AF_INET if kind == 1 else socket.AF_INET6,
                                          packet[offset:offset + size]))
        offset += size
    return result


class Upstream:
    def __init__(self):
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.socket.bind(("127.0.0.1", 0))
        self.socket.settimeout(0.1)
        self.port = self.socket.getsockname()[1]
        self.queries = []
        self.done = threading.Event()
        self.thread = threading.Thread(target=self.serve, daemon=True)
        self.thread.start()

    def serve(self):
        while not self.done.is_set():
            try:
                packet, peer = self.socket.recvfrom(4096)
            except socket.timeout:
                continue
            end = skip_name(packet, 12)
            kind = struct.unpack("!H", packet[end:end + 2])[0]
            self.queries.append(kind)
            ip = "203.0.113.10" if kind == 1 else "2001:db8::10"
            value = socket.inet_pton(socket.AF_INET if kind == 1 else socket.AF_INET6, ip)
            response = packet[:2] + struct.pack("!5H", 0x8180, 1, 1, 0, 0)
            response += packet[12:end + 4]
            response += b"\xc0\x0c" + struct.pack("!HHIH", kind, 1, 0, len(value)) + value
            self.socket.sendto(response, peer)

    def close(self):
        self.done.set()
        self.thread.join(timeout=2)
        self.socket.close()


@unittest.skipUnless(DNSMASQ, "set DNSMASQ_BIN to a dnsmasq executable")
class DnsBehaviorTests(unittest.TestCase):
    def setUp(self):
        self.upstream = Upstream()

    def tearDown(self):
        self.upstream.close()

    @contextlib.contextmanager
    def dnsmasq(self, records):
        with tempfile.TemporaryDirectory(prefix="dnsmasq-dns-test-") as directory:
            with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as port_socket:
                port_socket.bind(("127.0.0.1", 0))
                port = port_socket.getsockname()[1]
            config = Path(directory) / "dnsmasq.conf"
            config.write_text(f"no-resolv\nno-hosts\nbind-interfaces\nlisten-address=127.0.0.1\n"
                              f"port={port}\nserver=127.0.0.1#{self.upstream.port}\n" + records)
            subprocess.run([DNSMASQ, "--test", f"--conf-file={config}"], check=True,
                           capture_output=True, timeout=5)
            with (Path(directory) / "dnsmasq.log").open("w+") as log:
                process = subprocess.Popen([DNSMASQ, "--no-daemon", f"--conf-file={config}"],
                                           stdin=subprocess.DEVNULL, stdout=log, stderr=log)
                try:
                    for _ in range(20):
                        if process.poll() is not None:
                            log.seek(0)
                            self.fail(log.read())
                        try:
                            query(port, "app.example.test", 1)
                            break
                        except socket.timeout:
                            time.sleep(0.05)
                    else:
                        self.fail("dnsmasq did not become ready")
                    yield port
                finally:
                    process.terminate()
                    try:
                        process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait(timeout=5)

    def test_ipv4_only_without_local_rule_forwards_aaaa(self):
        with self.dnsmasq("address=/app.example.test/10.10.0.1\n") as port:
            self.assertEqual(addresses(query(port, "app.example.test", 1)), ["10.10.0.1"])
            self.assertEqual(addresses(query(port, "app.example.test", 28)), ["2001:db8::10"])
            self.assertIn(28, self.upstream.queries)

    def test_local_rule_blocks_upstream_for_domain_and_subdomains(self):
        for directive in ("server", "local"):
            with self.subTest(directive=directive):
                self.upstream.queries.clear()
                with self.dnsmasq(f"address=/app.example.test/10.10.0.1\n{directive}=/app.example.test/\n") as port:
                    for name in ("app.example.test", "child.app.example.test"):
                        self.assertEqual(addresses(query(port, name, 1)), ["10.10.0.1"])
                        self.assertEqual(addresses(query(port, name, 28)), [])
                    self.assertEqual(self.upstream.queries, [])
                    self.assertEqual(addresses(query(port, "other.example.test", 28)), ["2001:db8::10"])
                    self.assertEqual(self.upstream.queries, [28])

    def test_dual_stack_answers_both_families_locally(self):
        with self.dnsmasq("address=/app.example.test/10.10.0.1\n"
                          "address=/app.example.test/fd00::1\nserver=/app.example.test/\n") as port:
            self.assertEqual(addresses(query(port, "app.example.test", 1)), ["10.10.0.1"])
            self.assertEqual(addresses(query(port, "app.example.test", 28)), ["fd00::1"])
            self.assertEqual(self.upstream.queries, [])


if __name__ == "__main__":
    unittest.main()
