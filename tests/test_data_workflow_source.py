#!/usr/bin/env python3
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
import data_workflow_checks as checks
from data_workflow_source import sql_writes


class DataWorkflowSourceTests(unittest.TestCase):
    def assert_unclassified(self, source_text, symbol):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / "crates/lkjmc-store/src/example.rs"
            source.parent.mkdir(parents=True)
            source.write_text(source_text, encoding="utf-8")
            config = root / "config/data-workflows.json"
            config.parent.mkdir()
            config.write_text(
                json.dumps({"classifications": [], "schema": "lkjmc-data-workflows-two"}),
                encoding="utf-8",
            )
            with mock.patch.object(checks, "ROOT", root):
                errors = checks.inventory_errors()
            expected = (
                "unclassified multiwrite/effect: "
                f"crates/lkjmc-store/src/example.rs::{symbol}"
            )
            self.assertIn(expected, errors)

    def test_batch_execute_two_statements_is_rejected(self):
        self.assert_unclassified(
            'fn escaped(c: &mut C) { c.batch_execute('
            '"INSERT INTO first VALUES (1); UPDATE second SET value = 2;"); }',
            "escaped",
        )

    def test_batch_sql_whitespace_comments_and_read_only_text(self):
        body = r'''fn batch(c: &mut C) { c.batch_execute(r#"
            SELECT 'UPDATE ignored SET value = 1;';
            -- DELETE FROM ignored
            /* INSERT INTO ignored VALUES (1); */
            INSERT /* space */ INTO first VALUES (1);

            /* between statements */ UPDATE second SET value = 2;
        "#); }'''
        self.assertEqual(
            sql_writes(body),
            ["sql:insert-into:first", "sql:update:second"],
        )
        self.assert_unclassified(body, "batch")

    def test_normal_string_comment_newline_separates_writes(self):
        self.assert_unclassified(
            'fn escaped(c: &mut C) { c.batch_execute('
            '"INSERT INTO first VALUES (1); -- ignored UPDATE nope\\n '
            'DELETE FROM second;"); }',
            "escaped",
        )

    def test_std_tcp_listener_bind_is_rejected(self):
        self.assert_unclassified(
            'fn escaped() { let _ = std::net::TcpListener::bind("127.0.0.1:0"); }',
            "escaped",
        )

    def test_qualified_and_imported_tcp_listeners_are_rejected(self):
        self.assert_unclassified(
            'use std::net::TcpListener;\n'
            'fn imported() { let _ = TcpListener::bind("127.0.0.1:0"); }\n'
            'fn qualified() { let _ = tokio::net::TcpListener::bind("x").await; }',
            "imported",
        )
        self.assert_unclassified(
            'fn qualified() { let _ = tokio::net::TcpListener::bind("x").await; }',
            "qualified",
        )

    def test_listener_accept_and_common_socket_constructors_are_rejected(self):
        cases = (
            ('fn accept_one(listener: L) { listener.accept(); }', "accept_one"),
            ('fn udp() { UdpSocket::bind("127.0.0.1:0"); }', "udp"),
            ('fn unix() { UnixStream::connect("socket"); }', "unix"),
            ('fn socket() { TcpSocket::new_v4(); }', "socket"),
        )
        for source, symbol in cases:
            with self.subTest(symbol=symbol): self.assert_unclassified(source, symbol)

    def test_existing_effect_and_multiwrite_controls_are_rejected(self):
        cases = (
            (
                'fn direct(c: &mut C) { c.execute("insert into one values (1)"); '
                'c.execute("update two set value = 2"); }',
                "direct",
            ),
            (
                'fn one(c: &mut C) { c.execute("insert into one values (1)"); }\n'
                'fn two(c: &mut C) { c.execute("delete from two"); }\n'
                'fn nested(c: &mut C) { one(c); two(c); }',
                "nested",
            ),
            ('fn process() { std::process::Command::new("false"); }', "process"),
            ('fn filesystem() { std::fs::write("x", b"x"); }', "filesystem"),
            ('fn stream() { TcpStream::connect("127.0.0.1:9"); }', "stream"),
        )
        for source, symbol in cases:
            with self.subTest(symbol=symbol): self.assert_unclassified(source, symbol)


if __name__ == "__main__":
    unittest.main()
