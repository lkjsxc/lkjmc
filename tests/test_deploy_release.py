#!/usr/bin/env python3
"""Deterministic safety regressions for immutable release publication and update boundaries."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import stat
import subprocess
import sys
import tempfile
import time
from types import SimpleNamespace
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]


def load_module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


DEPLOY = load_module("deploy_release", ROOT / "scripts/deploy-release.py")
FENCE_CHECK = load_module("deployment_fence_check", ROOT / "scripts/deployment-fence-check.py")


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_release(root, artifacts, commit="a" * 40):
    source = root / "source"
    source.mkdir(parents=True)
    items = []
    for name, kind, payload in artifacts:
        path = source / name
        path.write_bytes(payload)
        path.chmod(0o700 if kind == "binary" else 0o600)
        items.append({
            "component": name,
            "kind": kind,
            "path": name,
            "provenance": f"pinned build at {commit}",
            "sha256": digest(path),
            "size": len(payload),
            "source": f"fixture/{name}",
        })
    data = {"schemaVersion": 1, "commit": commit, "artifacts": items}
    manifest = root / "artifact-manifest.json"
    manifest.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    manifest.chmod(0o600)
    value = digest(manifest)
    sidecar = root / "artifact-manifest.json.sha256"
    sidecar.write_text(f"{value}  artifact-manifest.json\n", encoding="ascii")
    sidecar.chmod(0o600)
    return value


class DeployReleaseTest(unittest.TestCase):
    def test_anchored_release_requires_the_exact_packaged_update_closure(self):
        with tempfile.TemporaryDirectory(prefix="lkjmc-deploy-release-") as raw:
            root = Path(raw)
            artifacts = [
                (name, kind, f"exact-{name}\n".encode())
                for name, kind in DEPLOY.EXPECTED_ARTIFACTS.items()
            ]
            manifest_digest = write_release(root, artifacts)
            release = DEPLOY.load_anchored_release(root, manifest_digest)
            self.assertEqual(release["commit"], "a" * 40)
            self.assertEqual(set(release["artifacts"]), set(DEPLOY.EXPECTED_ARTIFACTS))

            (root / "source/lkjmc-paper.jar").write_bytes(b"substituted\n")
            with self.assertRaisesRegex(DEPLOY.DeployError, "release artifact differs"):
                DEPLOY.load_anchored_release(root, manifest_digest)

    def test_portable_installer_publishes_config_metadata_and_stable_noop(self):
        with tempfile.TemporaryDirectory(prefix="lkjmc-portable-install-") as raw:
            base = Path(raw)
            first = base / "first"
            first.mkdir()
            first_digest = write_release(first, [
                ("tool", "binary", b"tool-one\n"),
                ("plugin.jar", "jar", b"jar-one\n"),
                ("daemon.service", "config", b"unit-one\n"),
            ])
            target = base / "installed"

            def command(release, manifest_digest):
                arguments = [
                    sys.executable,
                    str(ROOT / "scripts/install_artifacts.py"),
                    "--manifest", str(release / "artifact-manifest.json"),
                    "--manifest-sha256", manifest_digest,
                    "--source", str(release / "source"),
                    "--root", str(target),
                ]
                if os.geteuid() == 0:
                    arguments += [
                        "--scope", "system",
                        "--service-uid", "0",
                        "--service-gid", str(os.getegid()),
                    ]
                else:
                    arguments += ["--scope", "user"]
                return arguments

            subprocess.run(command(first, first_digest), cwd=ROOT, check=True, capture_output=True, text=True)
            self.assertEqual((target / "share/daemon.service").read_bytes(), b"unit-one\n")
            self.assertTrue((target / "meta/artifact-manifest.json").is_file())
            before = (target / "bin/tool").stat()
            subprocess.run(command(first, first_digest), cwd=ROOT, check=True, capture_output=True, text=True)
            after = (target / "bin/tool").stat()
            self.assertEqual((before.st_ino, before.st_mtime_ns), (after.st_ino, after.st_mtime_ns))

            changed = base / "changed"
            changed.mkdir()
            changed_digest = write_release(changed, [
                ("tool", "binary", b"tool-two\n"),
                ("plugin.jar", "jar", b"jar-two\n"),
                ("daemon.service", "config", b"unit-two\n"),
            ], commit="b" * 40)
            result = subprocess.run(
                command(changed, changed_digest),
                cwd=ROOT,
                env=os.environ | {"LKJMC_INSTALL_FAULT": "after-publish"},
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual((target / "bin/tool").read_bytes(), b"tool-one\n")
            self.assertEqual((target / "share/daemon.service").read_bytes(), b"unit-one\n")
            self.assertFalse(any(path.name.startswith((".lkjmc-stage-", ".lkjmc-rollback-")) for path in base.iterdir()))

            committed = subprocess.run(
                command(changed, changed_digest),
                cwd=ROOT,
                env=os.environ | {"LKJMC_INSTALL_FAULT": "after-commit"},
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertNotEqual(committed.returncode, 0)
            self.assertEqual((target / "bin/tool").read_bytes(), b"tool-two\n")
            rollbacks = [path for path in base.iterdir() if path.name.startswith(".lkjmc-rollback-")]
            self.assertEqual(len(rollbacks), 1)
            self.assertEqual((rollbacks[0] / "bin/tool").read_bytes(), b"tool-one\n")

    def test_self_consistent_fake_bytes_are_not_accepted_as_a_postgres_backup(self):
        with tempfile.TemporaryDirectory(prefix="lkjmc-backup-proof-") as raw:
            root = Path(raw)
            dump = root / "lkjmc.dump"
            manifest = Path(str(dump) + ".manifest")
            metadata = Path(str(dump) + ".metadata.json")
            checks = Path(str(dump) + ".sha256")
            dump.write_bytes(b"database\n")
            manifest.write_bytes(b"manifest\n")
            marker = [{"version": 53, "name": "menu removal", "checksum": "c" * 64}]
            metadata.write_text(json.dumps({
                "schemaVersion": 1,
                "sourceCommit": "d" * 40,
                "migrationMarker": json.dumps(marker, separators=(",", ":"), sort_keys=True),
                "dumpSha256": digest(dump),
                "manifestSha256": digest(manifest),
            }) + "\n", encoding="utf-8")
            checks.write_text(
                "".join(f"{digest(path)}  {path.name}\n" for path in (dump, manifest, metadata)),
                encoding="ascii",
            )
            for path in (dump, manifest, metadata, checks):
                path.chmod(0o600)
                os.utime(path, (time.time(), time.time()))

            old_root = DEPLOY.BACKUP_ROOT
            try:
                DEPLOY.BACKUP_ROOT = root
                with self.assertRaises(DEPLOY.DeployError):
                    DEPLOY.verify_backup(dump, "d" * 40, marker, 3600)
            finally:
                DEPLOY.BACKUP_ROOT = old_root

    def test_root_execution_inputs_reject_service_owned_or_group_writable_metadata(self):
        self.assertTrue(DEPLOY.root_owned_safe(SimpleNamespace(st_uid=0, st_mode=stat.S_IFREG | 0o750)))
        self.assertFalse(DEPLOY.root_owned_safe(SimpleNamespace(st_uid=999, st_mode=stat.S_IFREG | 0o700)))
        self.assertFalse(DEPLOY.root_owned_safe(SimpleNamespace(st_uid=0, st_mode=stat.S_IFREG | 0o770)))

    def test_trusted_commands_match_the_current_distribution_safety(self):
        commands = (
            DEPLOY.SYSTEMCTL,
            DEPLOY.RUNUSER,
            DEPLOY.PSQL,
            DEPLOY.PGRESTORE,
            DEPLOY.PGREP,
            DEPLOY.PYTHON,
        )
        for command in commands:
            with self.subTest(command=command):
                try:
                    resolved = DEPLOY.trusted_command(command)
                except DEPLOY.DeployError as error:
                    message = str(error)
                    self.assertTrue(
                        "missing or unreadable required command" in message
                        or "ownership or mode is unsafe" in message,
                        message,
                    )
                else:
                    self.assertEqual(resolved, command.resolve(strict=True))

    def test_trusted_commands_reject_unapproved_or_out_of_root_targets(self):
        with self.assertRaisesRegex(DEPLOY.DeployError, "unexpected required command"):
            DEPLOY.trusted_command(Path("/usr/bin/true"))

        command = Path("/usr/bin/true")
        try:
            DEPLOY.TRUSTED_COMMAND_TARGET_ROOTS[command] = ()
            with self.assertRaisesRegex(DEPLOY.DeployError, "outside its allowed target roots"):
                DEPLOY.trusted_command(command)
        finally:
            del DEPLOY.TRUSTED_COMMAND_TARGET_ROOTS[command]

    def test_trusted_commands_reject_unsafe_ancestry_type_mode_and_symlink_owner(self):
        with tempfile.TemporaryDirectory(prefix="lkjmc-command-trust-") as raw:
            root = Path(raw)
            command = root / "command"
            command.write_text("#!/bin/sh\nexit 0\n", encoding="ascii")
            command.chmod(0o755)
            DEPLOY.TRUSTED_COMMAND_TARGET_ROOTS[command] = (root,)
            try:
                with self.assertRaisesRegex(DEPLOY.DeployError, "directory ownership or mode is unsafe"):
                    DEPLOY.trusted_command(command)
            finally:
                del DEPLOY.TRUSTED_COMMAND_TARGET_ROOTS[command]

            link = root / "untrusted-link"
            link.symlink_to("/usr/bin/true")
            if os.geteuid() == 0:
                os.chown(link, 65534, 65534, follow_symlinks=False)
            DEPLOY.TRUSTED_COMMAND_TARGET_ROOTS[link] = (Path("/usr/bin"),)
            try:
                with mock.patch.object(DEPLOY, "require_root_ancestry"):
                    with self.assertRaisesRegex(DEPLOY.DeployError, "symlink is not root-owned"):
                        DEPLOY.trusted_command(link)
            finally:
                del DEPLOY.TRUSTED_COMMAND_TARGET_ROOTS[link]

        hosts = Path("/etc/hosts")
        DEPLOY.TRUSTED_COMMAND_TARGET_ROOTS[hosts] = (Path("/etc"),)
        try:
            with self.assertRaisesRegex(DEPLOY.DeployError, "ownership or executable mode is unsafe"):
                DEPLOY.trusted_command(hosts)
        finally:
            del DEPLOY.TRUSTED_COMMAND_TARGET_ROOTS[hosts]

        directory = Path("/usr/bin")
        DEPLOY.TRUSTED_COMMAND_TARGET_ROOTS[directory] = (Path("/usr"),)
        try:
            with self.assertRaisesRegex(DEPLOY.DeployError, "is not a regular file"):
                DEPLOY.trusted_command(directory)
        finally:
            del DEPLOY.TRUSTED_COMMAND_TARGET_ROOTS[directory]

        command = Path("/usr/bin/true")
        DEPLOY.TRUSTED_COMMAND_TARGET_ROOTS[command] = (Path("/usr/bin"),)
        changed = SimpleNamespace(
            st_dev=-1,
            st_ino=-1,
            st_mode=stat.S_IFREG | 0o755,
            st_uid=0,
        )
        try:
            with mock.patch.object(DEPLOY, "regular", return_value=changed):
                with self.assertRaisesRegex(DEPLOY.DeployError, "identity changed during validation"):
                    DEPLOY.trusted_command(command)
        finally:
            del DEPLOY.TRUSTED_COMMAND_TARGET_ROOTS[command]

    def test_database_environment_is_data_not_root_shell_input(self):
        with tempfile.TemporaryDirectory(prefix="lkjmc-daemon-env-") as raw:
            path = Path(raw) / "daemon.env"
            path.write_text(
                "LKJMC_DATABASE_URL=postgres://lkjmc:opaque@127.0.0.1:5432/lkjmc\n",
                encoding="ascii",
            )
            path.chmod(0o600)
            value, _ = DEPLOY.database_url_from_environment(path)
            self.assertEqual(value, "postgres://lkjmc:opaque@127.0.0.1:5432/lkjmc")
            path.write_text(
                "LKJMC_DATABASE_URL=postgres://lkjmc:opaque@127.0.0.1:5432/lkjmc\n"
                "touch /root/service-user-code-ran\n",
                encoding="ascii",
            )
            with self.assertRaisesRegex(DEPLOY.DeployError, "one newline-terminated assignment"):
                DEPLOY.database_url_from_environment(path)

    def test_global_deployment_lock_rejects_a_concurrent_invocation(self):
        with tempfile.TemporaryDirectory(prefix="lkjmc-deploy-lock-") as raw:
            old_lock = DEPLOY.LOCK
            try:
                DEPLOY.LOCK = Path(raw) / "deploy.lock"
                with DEPLOY.deployment_lock():
                    with self.assertRaisesRegex(DEPLOY.DeployError, "global lock"):
                        with DEPLOY.deployment_lock():
                            pass
            finally:
                DEPLOY.LOCK = old_lock

    def test_effective_eula_value_must_not_be_overridden_by_later_false(self):
        self.assertEqual(DEPLOY.effective_eula("# accepted\neula=true\n"), "true")
        self.assertEqual(DEPLOY.effective_eula("eula=true\neula=false\n"), "false")
        self.assertEqual(DEPLOY.effective_eula("eula=true=invalid\n"), "true=invalid")
        self.assertIsNone(DEPLOY.effective_eula("# eula=true\n"))

    def test_topology_validator_rejects_duplicate_instances_and_listeners(self):
        network = {
            "instances": [
                {"id": "hub", "kind": "folia", "listener": "hub-java", "desiredState": "running", "owner": "lkjmc-daemon", "assetIds": ["folia"]},
                {"id": "proxy", "kind": "velocity", "listener": "proxy-java", "desiredState": "running", "owner": "lkjmc-daemon", "assetIds": ["velocity"]},
                {"id": "survival", "kind": "folia", "listener": "survival-java", "desiredState": "running", "owner": "lkjmc-daemon", "assetIds": ["folia"]},
            ],
            "listeners": [
                {"id": "hub-java", "bindHost": "127.0.0.1", "port": 25566, "protocol": "java-tcp", "publicHosts": []},
                {"id": "proxy-java", "bindHost": "0.0.0.0", "port": 25591, "protocol": "java-tcp", "publicHosts": ["lkjsxc.com"]},
                {"id": "survival-java", "bindHost": "127.0.0.1", "port": 25567, "protocol": "java-tcp", "publicHosts": []},
            ],
            "routes": [{"id": "default", "listener": "proxy-java", "target": "hub", "fallbacks": ["survival"]}],
            "capabilities": {"runtime": "local-process", "mountedConfig": True, "mountedSecrets": True, "mountedAssets": True},
            "auth": {"onlineMode": True},
            "forwarding": {"mode": "modern"},
        }
        self.assertEqual(len(DEPLOY.validate_network_topology(network)), 3)
        duplicated = json.loads(json.dumps(network))
        duplicated["instances"].append(dict(duplicated["instances"][1]))
        with self.assertRaises(DEPLOY.DeployError):
            DEPLOY.validate_network_topology(duplicated)
        duplicated = json.loads(json.dumps(network))
        duplicated["listeners"][2] = dict(duplicated["listeners"][1])
        with self.assertRaises(DEPLOY.DeployError):
            DEPLOY.validate_network_topology(duplicated)

    def test_binary_rollback_is_forbidden_for_changed_or_unknown_migration_state(self):
        before = [{"version": 52}]
        self.assertTrue(DEPLOY.binary_rollback_allowed(before, [{"version": 52}]))
        self.assertFalse(DEPLOY.binary_rollback_allowed(before, [{"version": 52}, {"version": 53}]))
        self.assertFalse(DEPLOY.binary_rollback_allowed(before, None))

    def test_withdrawn_checkout_installer_fails_without_mutation(self):
        with tempfile.TemporaryDirectory(prefix="lkjmc-withdrawn-install-") as raw:
            marker = Path(raw) / "unchanged"
            marker.write_text("before\n", encoding="utf-8")
            result = subprocess.run(
                (ROOT / "scripts/install.sh",),
                cwd=raw,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("checkout-based lkjmc installer has been withdrawn", result.stderr)
            self.assertEqual(marker.read_text(encoding="utf-8"), "before\n")

    def test_restart_helper_refuses_to_invent_eula_acceptance(self):
        helper = (ROOT / "packaging/lkjmc-bootstrap-after-start").read_text(encoding="utf-8")
        self.assertIn("no existing Minecraft EULA acceptance record", helper)
        self.assertNotIn("printf 'eula=true", helper)
        self.assertNotIn('echo "eula=true', helper)
        self.assertIn('value=substr($0, separator + 1)', helper)
        self.assertNotIn('value=$2', helper)

    def test_privileged_fence_check_consumes_exactly_one_start_permit(self):
        with tempfile.TemporaryDirectory(prefix="lkjmc-fence-check-") as raw:
            root = Path(raw)
            root.chmod(0o700)
            fence = root / "deployment-fence.json"
            permit = root / "start-permit"
            self.assertEqual(
                FENCE_CHECK.check(
                    fence, permit, expected_uid=os.geteuid(), trusted_root=root),
                "unfenced",
            )
            fence.write_text(json.dumps({
                "schemaVersion": 1,
                "fromCommit": "a" * 40,
                "toCommit": "b" * 40,
                "stateDirectory": "/var/lib/private/lkjmc-deployments/" + "b" * 40,
                "backup": "/var/backups/lkjmc/pre-update/lkjmc.dump",
                "rollbackSnapshot": "pre-update",
            }) + "\n", encoding="utf-8")
            fence.chmod(0o600)
            permit.write_bytes(b"lkjmc-deploy-start-permit\n")
            permit.chmod(0o400)
            self.assertEqual(
                FENCE_CHECK.check(
                    fence, permit, expected_uid=os.geteuid(), trusted_root=root),
                "permitted-once",
            )
            self.assertFalse(permit.exists())
            with self.assertRaisesRegex(FENCE_CHECK.FenceError, "blocks service start"):
                FENCE_CHECK.check(
                    fence, permit, expected_uid=os.geteuid(), trusted_root=root)
            fence.unlink()
            permit.write_bytes(b"lkjmc-deploy-start-permit\n")
            permit.chmod(0o400)
            with self.assertRaisesRegex(FENCE_CHECK.FenceError, "without a deployment fence"):
                FENCE_CHECK.check(
                    fence, permit, expected_uid=os.geteuid(), trusted_root=root)

    def test_systemd_uses_one_privileged_reset_fence_check(self):
        unit = (ROOT / "packaging/lkjmc-daemon.service").read_text(encoding="utf-8")
        dropin = (ROOT / "packaging/lkjmc-deployment-fence.conf").read_text(encoding="utf-8")
        self.assertIn("ExecStartPre=+/opt/lkjmc/releases/current/bin/lkjmc-deployment-fence-check", unit)
        self.assertIn("ExecStartPre=\n", dropin)
        self.assertIn("ExecStartPre=+@LKJMC_FENCE_CHECKER@", dropin)

    def test_effective_systemd_fence_check_counts_commands_not_argv_repetitions(self):
        checker = "/opt/lkjmc/releases/" + "a" * 40 + "/bin/lkjmc-deployment-fence-check"
        effective = (
            f"ExecStartPre={{ path={checker} ; argv[]={checker} ; "
            "ignore_errors=no ; start_time=[n/a] ; stop_time=[n/a] ; pid=0 ; status=0/0 }}\n"
        )
        self.assertEqual(DEPLOY.effective_exec_start_pre_paths(effective), [checker])
        duplicate = effective.rstrip() + (
            " { path=/usr/bin/false ; argv[]=/usr/bin/false ; ignore_errors=no ; "
            "start_time=[n/a] ; stop_time=[n/a] ; pid=0 ; status=0/0 }\n"
        )
        self.assertEqual(
            DEPLOY.effective_exec_start_pre_paths(duplicate),
            [checker, "/usr/bin/false"],
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
