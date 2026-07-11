#!/usr/bin/env python3
from __future__ import annotations

import os
from pathlib import Path
import socket
import sys
import tempfile
import threading
import time
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).parent))
from lab_boundaries import _psql, disposable_postgres_url, postgres_real  # noqa: E402
from lab_harness import Lab, http_post_tcp, http_post_unix  # noqa: E402
from lab_outcomes import Blocked  # noqa: E402
from lab_probes import run  # noqa: E402


class LabHarnessTest(unittest.TestCase):
    def setUp(self) -> None:
        self.artifact_root = tempfile.TemporaryDirectory()
        self.previous = os.environ.get("LKJMC_LAB_ARTIFACT_ROOT")
        os.environ["LKJMC_LAB_ARTIFACT_ROOT"] = self.artifact_root.name

    def tearDown(self) -> None:
        if self.previous is None:
            os.environ.pop("LKJMC_LAB_ARTIFACT_ROOT", None)
        else:
            os.environ["LKJMC_LAB_ARTIFACT_ROOT"] = self.previous
        self.artifact_root.cleanup()

    def test_cleanup_probe_proves_held_tcp_unix_and_process_teardown(self) -> None:
        self.assertEqual("PASS", run("isolation-cleanup"))

    def test_close_releases_actual_listener_socket_and_child(self) -> None:
        lab = Lab("cleanup")
        root, tcp = lab.root, lab.listen_tcp()
        port = int(tcp.getsockname()[1])
        path = lab.listen_unix("held.sock")
        child = lab.start("child", [sys.executable, "-u", "-c", "import time; time.sleep(60)"])
        with socket.create_connection(("127.0.0.1", port), timeout=2):
            pass
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as unix:
            unix.connect(str(path))
        lab.close()
        self.assertFalse(root.exists())
        self.assertFalse(path.exists())
        self.assertIsNotNone(child.poll())
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
            listener.bind(("127.0.0.1", port))

    def test_tcp_and_unix_http_use_real_local_sockets(self) -> None:
        lab = Lab("sockets")
        listeners: list[socket.socket] = []
        try:
            tcp = lab.listen_tcp()
            listeners.append(tcp)
            thread = self._reply_once(tcp)
            self.assertEqual((200, '{"ok":true}'), http_post_tcp(tcp.getsockname()[1], "{}", "tcp-token"))
            unix_path = lab.listen_unix("daemon.sock")
            unix = lab.listeners[-1][0]
            listeners.append(unix)
            unix_thread = self._reply_once(unix)
            self.assertEqual((200, '{"ok":true}'), http_post_unix(unix_path, "{}"))
            thread.join(2)
            unix_thread.join(2)
            self.assertFalse(thread.is_alive() or unix_thread.is_alive())
        finally:
            lab.close()

    def test_record_redacts_unregistered_uri_credentials_and_queries(self) -> None:
        lab = Lab("adversarial-redaction")
        values = [
            "http userinfo", "http token tail", "https userinfo", "https password tail",
            "custom userinfo", "custom authorization tail", "postgres userinfo",
            "postgres%20token", "nested password", "nested token tail", "basic sentinel",
            "bearer sentinel", "combined token tail", "combined key tail",
        ]
        try:
            artifact = lab.record("adversarial.log", "\n".join([
                f"http://lab:{values[0]} @lab.invalid/path?token={values[1]}&safe=1",
                f"https://lab:{values[2]} @lab.invalid/path?password={values[3]}#fragment",
                f"custom+lab://lab:{values[4]} @lab.invalid/path?authorization={values[5]}#fragment",
                f"postgresql://lab:{values[6]} @127.0.0.1/db?api_key={values[7]}&safe=1",
                f'{{"nested":{{"password":"{values[8]}","endpoint":"https://lab/n?token={values[9]}&safe=1"}}}}',
                f"Authorization: Basic {values[10]}", f"authorization: Bearer {values[11]}",
                f"https://lab.invalid/callback?token={values[12]}&mode=public&key={values[13]}#done",
            ]))
            text = artifact.read_text(encoding="utf-8")
            self.assertTrue(artifact.is_file())
            self.assertTrue(all(value not in text for value in values))
            self.assertIn("Authorization: <redacted>", text)
            self.assertIn("authorization: <redacted>", text)
            self.assertIn("#fragment", text)
        finally:
            lab.close()
            self.assertFalse(lab.root.exists())

    def test_postgres_policy_and_quiet_psql_command(self) -> None:
        self.assertTrue(disposable_postgres_url("postgresql://lab:pw@127.0.0.1:5432/lkjmc_lab_disposable"))
        for value in ["postgres://lab:pw@db.example/lkjmc_lab_disposable", "postgres://lab:pw@localhost/postgres", "postgres://lab:pw@localhost/lkjmc_lab_test?host=db.example"]:
            self.assertFalse(disposable_postgres_url(value))
        fake = _FakeLab()
        with patch.dict(os.environ, {"LKJMC_LAB_POSTGRES_URL": "postgres://lab:pw@localhost/lkjmc_lab_test"}):
            self.assertEqual("1\n", _psql(fake, "query", "SELECT 1"))
        self.assertTrue({"--quiet", "--tuples-only", "--no-align", "ON_ERROR_STOP=1"}.issubset(fake.command))
        unconfirmed = _FakeLab()
        with patch("lab_boundaries.shutil.which", return_value=None), patch.dict(
            os.environ, {"LKJMC_LAB_POSTGRES_URL": "postgres://lab:pw@localhost/lkjmc_lab_test"}, clear=True
        ):
            with self.assertRaises(Blocked):
                postgres_real(unconfirmed)
        self.assertFalse(hasattr(unconfirmed, "command"))
        unsafe = _FakeLab()
        env = {"LKJMC_LAB_POSTGRES_URL": "postgres://lab:secret@db.example/lkjmc_lab_test", "LKJMC_LAB_POSTGRES_DISPOSABLE": "1"}
        with patch("lab_boundaries.shutil.which", return_value=None), patch.dict(os.environ, env, clear=True):
            with self.assertRaises(Blocked):
                postgres_real(unsafe)
        self.assertFalse(hasattr(unsafe, "command"))

    def _reply_once(self, listener: socket.socket) -> threading.Thread:
        def reply() -> None:
            connection, _ = listener.accept()
            with connection:
                request = b""
                while b"\r\n\r\n" not in request:
                    chunk = connection.recv(4096)
                    if not chunk:
                        return
                    request += chunk
                connection.sendall(b'HTTP/1.1 200 OK\r\nContent-Length: 11\r\nConnection: close\r\n\r\n{"ok":true}')
        thread = threading.Thread(target=reply)
        thread.start()
        return thread


class _FakeLab:
    def run(self, _label: str, command: list[str], _timeout: int) -> tuple[int, str]:
        self.command = command
        return 0, "1\n"


if __name__ == "__main__":
    unittest.main()
