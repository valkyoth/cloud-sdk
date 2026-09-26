#!/usr/bin/env python3
"""Actual child-process timeout and source-worker failure regressions."""

import hashlib
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import patch

import cratesio_source_fetch as fetch
from cratesio_source_error import SourceLockError


class DeadlineTests(unittest.TestCase):
    def test_slow_real_http_read_is_killed_and_reaped(self):
        # Real HTTPResponse.read() would otherwise wait for all trickled bytes.
        worker = r'''
import http.client, io, runpy, socket, sys, threading, time
from pathlib import Path
from unittest.mock import patch
sys.path.insert(0, str(Path(sys.argv[1]).parent))
marker = Path(sys.argv[2])
left, right = socket.socketpair()
def send():
    right.sendall(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 80\r\n\r\n")
    for _ in range(80):
        right.sendall(b"x")
        time.sleep(0.1)
threading.Thread(target=send, daemon=True).start()
response = http.client.HTTPResponse(left)
response.begin()
response.geturl = lambda: "https://example.test/source"
read = response.read
def marked_read(amount):
    marker.write_text("reading", encoding="ascii")
    return read(amount)
response.read = marked_read
class Opener:
    def open(self, *args, **kwargs): return response
with patch("urllib.request.build_opener", return_value=Opener()):
    sys.argv = [sys.argv[1], "--worker"]
    runpy.run_path(sys.argv[0], run_name="__main__")
'''
        real_run = subprocess.run
        real_popen = subprocess.Popen
        children = []

        def tracked_popen(*args, **kwargs):
            process = real_popen(*args, **kwargs)
            children.append(process)
            return process

        def run(command, **kwargs):
            self.assertEqual(command, [sys.executable, "-E", "-s", str(Path(fetch.__file__).resolve()), "--worker"])
            return real_run([sys.executable, "-c", worker, command[3], str(marker)], **kwargs)

        source = dict(id="fixture", url="https://example.test/source",
                      final_url="https://example.test/source", redirects=[],
                      accept="application/json", media_type="application/json", max_bytes=80)
        started = time.monotonic()
        with tempfile.TemporaryDirectory() as directory, \
                patch.object(fetch.subprocess, "run", side_effect=run), \
                patch.object(fetch.subprocess, "Popen", side_effect=tracked_popen):
            marker = Path(directory) / "read-started"
            with self.assertRaisesRegex(SourceLockError, "deadline exceeded"):
                fetch.run_worker(source, timeout=1.0)
            self.assertEqual(marker.read_text(encoding="ascii"), "reading")
        self.assertLess(time.monotonic() - started, 4.0)
        self.assertEqual(len(children), 1)
        self.assertIsNotNone(children[0].returncode)
        self.assertNotEqual(children[0].returncode, 0)

    def test_worker_failure_and_size_are_fail_closed(self):
        for error in (OSError(), subprocess.CalledProcessError(1, [])):
            with patch.object(fetch.subprocess, "run", side_effect=error):
                with self.assertRaisesRegex(SourceLockError, "retrieval failed"):
                    fetch.run_worker({"max_bytes": 4})
        with patch.object(fetch.subprocess, "run", return_value=subprocess.CompletedProcess([], 0, b"12345")):
            with self.assertRaisesRegex(SourceLockError, "size bound"):
                fetch.run_worker({"max_bytes": 4})

    def test_observation_and_digest_validation_share_the_deadline(self):
        source = dict(id="fixture", max_bytes=4, size_bytes=4,
                      sha256=hashlib.sha256(b"1234").hexdigest())
        with patch.object(fetch, "run_worker", return_value=b"1234") as worker:
            self.assertEqual(fetch.observe_source(source), b"1234")
            self.assertEqual(fetch.fetch_source(source), b"1234")
            self.assertEqual(worker.call_count, 2)
        with patch.object(fetch, "run_worker", return_value=b"4321"):
            with self.assertRaisesRegex(SourceLockError, "digest or size changed"):
                fetch.fetch_source(source)


if __name__ == "__main__":
    unittest.main()
