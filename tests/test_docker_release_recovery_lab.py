from __future__ import annotations

import json
import os
from pathlib import Path
import stat
import sys
import tempfile
import unittest
from unittest import mock
import zipfile


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tests" / "docker_release_recovery"))

from docker_lab import (  # noqa: E402
    EXPECTED_CAPABILITIES,
    LabError,
    PROJECT_LABEL,
    PURPOSE_LABEL,
    _host_command,
    endpoint_class,
    extract_transport_zip,
    object_list_commands,
    private_json,
    publish_evidence_packet,
    validate_compose_model,
    validate_container_inspect,
    validate_project,
    verify_owned,
)
from fixture import config_value  # noqa: E402


PROJECT = "lkjmcdrr-12345678-abcdef1234"
CONTAINER = f"{PROJECT}-host"


def compose_model() -> dict:
    return {
        "services": {
            "host": {
                "cap_add": sorted(EXPECTED_CAPABILITIES),
                "cap_drop": ["ALL"],
                "cgroup": "private",
                "command": ["/usr/local/libexec/lkjmc-drr-systemd-entrypoint"],
                "labels": {PROJECT_LABEL: PROJECT, PURPOSE_LABEL: "systemd-host"},
                "networks": {"lab": None},
                "security_opt": ["apparmor=unconfined", "no-new-privileges:false"],
                "tmpfs": ["/run", "/run/lock", "/sys/fs/cgroup", "/tmp"],
            }
        },
        "networks": {"lab": {"internal": True, "labels": {PROJECT_LABEL: PROJECT}}},
    }


def container_inspect() -> dict:
    return {
        "Id": "a" * 64,
        "Name": f"/{CONTAINER}",
        "Config": {
            "Cmd": ["/usr/local/libexec/lkjmc-drr-systemd-entrypoint"],
            "Labels": {PROJECT_LABEL: PROJECT, PURPOSE_LABEL: "systemd-host"},
        },
        "HostConfig": {
            "Binds": None,
            "CapAdd": sorted(EXPECTED_CAPABILITIES),
            "CapDrop": ["ALL"],
            "CgroupnsMode": "private",
            "Devices": [],
            "Memory": 805306368,
            "NanoCpus": 2000000000,
            "NetworkMode": f"{PROJECT}_lab",
            "PidMode": "",
            "PidsLimit": 512,
            "PortBindings": {},
            "Privileged": False,
            "SecurityOpt": ["apparmor=unconfined", "no-new-privileges:false"],
            "Tmpfs": {"/run": "rw", "/run/lock": "rw", "/sys/fs/cgroup": "rw", "/tmp": "rw"},
        },
        "Mounts": [],
        "NetworkSettings": {"Ports": {}},
        "State": {"Pid": 1234, "StartedAt": "2026-08-30T00:00:00Z"},
    }


class DockerReleaseRecoveryLabTest(unittest.TestCase):
    def test_host_consumer_preserves_github_auth_location_and_reports_failure(self) -> None:
        with mock.patch.dict(
            os.environ,
            {"GH_CONFIG_DIR": "/private/github-config", "HOME": "/private/operator-home"},
            clear=True,
        ):
            result = _host_command(
                (
                    sys.executable,
                    "-c",
                    "import os; print(os.environ['GH_CONFIG_DIR']); print(os.environ['HOME'])",
                )
            )
        self.assertEqual(result.stdout.splitlines(), ["/private/github-config", "/private/operator-home"])
        with self.assertRaisesRegex(LabError, r"exit 7: .*exact safe diagnostic"):
            _host_command(
                (
                    sys.executable,
                    "-c",
                    "import sys; print('exact safe diagnostic', file=sys.stderr); raise SystemExit(7)",
                )
            )

    def test_project_identity_and_endpoint_classification_fail_closed(self) -> None:
        self.assertEqual(validate_project(PROJECT), PROJECT)
        for value in ("lkjmc", "lkjmcdrr-short", "lkjmcdrr-UPPERCASE-123", "../lkjmcdrr-12345678"):
            with self.assertRaises(LabError):
                validate_project(value)
        self.assertEqual(endpoint_class("unix:///var/run/docker.sock"), "local-default-unix")
        self.assertEqual(endpoint_class("unix:///run/user/1001/docker.sock"), "local-other-unix")
        self.assertEqual(endpoint_class("ssh://operator@example"), "ssh-remote")
        self.assertEqual(endpoint_class("tcp://example:2376"), "nonlocal-or-unknown")

    def test_compose_boundary_rejects_privilege_ports_namespaces_mounts_and_caps(self) -> None:
        self.assertEqual(validate_compose_model(compose_model(), PROJECT)["ports"], [])
        mutations = (
            ("privileged", True),
            ("network_mode", "host"),
            ("pid", "host"),
            ("ports", [{"published": "25591", "target": 25591}]),
            ("volumes", [{"source": "/", "target": "/host"}]),
            ("devices", [{"source": "/dev/kvm", "target": "/dev/kvm"}]),
            ("cgroup", "host"),
            ("cap_add", ["SYS_ADMIN", "SYS_PTRACE"]),
        )
        for key, value in mutations:
            with self.subTest(key=key):
                model = compose_model()
                model["services"]["host"][key] = value
                with self.assertRaises(LabError):
                    validate_compose_model(model, PROJECT)

    def test_effective_container_rejects_public_or_unowned_state(self) -> None:
        observation = validate_container_inspect(container_inspect(), PROJECT, CONTAINER)
        self.assertEqual(observation["ports"], [])
        mutations = (
            ("Privileged", True),
            ("NetworkMode", "host"),
            ("PidMode", "host"),
            ("CgroupnsMode", "host"),
            ("PortBindings", {"25591/tcp": [{"HostPort": "25591"}]}),
            ("Binds", ["/:/host"]),
        )
        for key, value in mutations:
            with self.subTest(key=key):
                inspect = container_inspect()
                inspect["HostConfig"][key] = value
                with self.assertRaises(LabError):
                    validate_container_inspect(inspect, PROJECT, CONTAINER)
        inspect = container_inspect()
        inspect["Config"]["Labels"][PROJECT_LABEL] = "lkjmcdrr-87654321-fedcba9876"
        with self.assertRaises(LabError):
            validate_container_inspect(inspect, PROJECT, CONTAINER)

    def test_cleanup_enumeration_and_ownership_are_exact_label_scoped(self) -> None:
        commands = object_list_commands(PROJECT)
        for command in commands.values():
            self.assertIn(f"label={PROJECT_LABEL}={PROJECT}", command)
        verify_owned({"Labels": {PROJECT_LABEL: PROJECT}}, PROJECT, "networks")
        verify_owned({"Config": {"Labels": {PROJECT_LABEL: PROJECT}}}, PROJECT, "images")
        with self.assertRaises(LabError):
            verify_owned({"Labels": {PROJECT_LABEL: "lkjmcdrr-87654321-fedcba9876"}}, PROJECT, "volumes")

    def test_private_evidence_is_exclusive_owner_only_json(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory) / "private"
            parent.mkdir(mode=0o700)
            output = parent / "result.json"
            private_json(output, {"schemaVersion": 1, "status": "PASS"})
            self.assertEqual(stat.S_IMODE(output.stat().st_mode), 0o600)
            self.assertEqual(json.loads(output.read_text())["status"], "PASS")
            with self.assertRaises(FileExistsError):
                private_json(output, {"status": "FAILED"})
            os.chmod(parent, 0o755)
            with self.assertRaises(LabError):
                private_json(parent / "second.json", {"status": "PASS"})

    def test_transport_extraction_accepts_only_the_three_private_regular_members(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            transport = parent / "artifact.zip"
            members = {
                "lkjmc-release.tar": b"archive",
                "lkjmc-release.tar.sha256": b"digest  lkjmc-release.tar\n",
                "release-handoff.json": b"{}\n",
            }
            with zipfile.ZipFile(transport, "w") as archive:
                for name, data in members.items():
                    info = zipfile.ZipInfo(name)
                    info.external_attr = (stat.S_IFREG | 0o600) << 16
                    archive.writestr(info, data)
            os.chmod(transport, 0o600)
            output = parent / "outer"
            result = extract_transport_zip(transport, output)
            self.assertEqual({item["path"] for item in result["outerFiles"]}, set(members))
            self.assertTrue(all(stat.S_IMODE(path.stat().st_mode) == 0o600 for path in output.iterdir()))
            with self.assertRaises(LabError):
                extract_transport_zip(transport, output)

    def test_transport_extraction_rejects_traversal_and_link_members(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            for label, bad_name, mode in (
                ("traversal", "../release-handoff.json", stat.S_IFREG | 0o600),
                ("link", "release-handoff.json", stat.S_IFLNK | 0o777),
            ):
                with self.subTest(label=label):
                    transport = parent / f"{label}.zip"
                    with zipfile.ZipFile(transport, "w") as archive:
                        values = (
                            ("lkjmc-release.tar", stat.S_IFREG | 0o600),
                            ("lkjmc-release.tar.sha256", stat.S_IFREG | 0o600),
                            (bad_name, mode),
                        )
                        for name, member_mode in values:
                            info = zipfile.ZipInfo(name)
                            info.external_attr = member_mode << 16
                            archive.writestr(info, b"value")
                    os.chmod(transport, 0o600)
                    with self.assertRaises(LabError):
                        extract_transport_zip(transport, parent / f"{label}-out")

    def test_lab_entrypoint_and_support_are_not_product_release_artifacts(self) -> None:
        inventory = json.loads((ROOT / "config" / "release-artifacts.json").read_text())
        sources = {item["source"] for item in inventory["artifacts"]}
        self.assertNotIn("scripts/run-docker-release-recovery-lab.py", sources)
        self.assertFalse(any(source.startswith("tests/docker_release_recovery/") for source in sources))

    def test_fixture_configuration_is_the_exact_private_supported_topology(self) -> None:
        assets = {
            "folia": {"sha256": "1" * 64},
            "velocity": {"sha256": "2" * 64},
        }
        value = config_value(Path("/etc/lkjmc/database.secret"), assets)
        network = value["network"]
        self.assertEqual(
            {item["id"]: (item["kind"], item["listener"], item["assetIds"]) for item in network["instances"]},
            {
                "hub": ("folia", "hub-java", ["folia-server"]),
                "proxy": ("velocity", "proxy-java", ["velocity-server"]),
                "survival": ("folia", "survival-java", ["folia-server"]),
            },
        )
        self.assertEqual(
            {item["id"]: (item["bindHost"], item["port"]) for item in network["listeners"]},
            {
                "hub-java": ("127.0.0.1", 25566),
                "proxy-java": ("0.0.0.0", 25591),
                "survival-java": ("127.0.0.1", 25567),
            },
        )
        self.assertTrue(network["auth"]["onlineMode"])
        self.assertEqual(network["routes"][0]["fallbacks"], ["survival"])
        self.assertEqual(
            network["capabilities"],
            {
                "mountedAssets": True,
                "mountedConfig": True,
                "mountedSecrets": True,
                "runtime": "local-process",
            },
        )
        self.assertTrue(value["plugins"]["lkjmc"]["enabled"])
        for plugin in ("viaversion", "viabackwards", "geyser", "floodgate"):
            self.assertEqual(value["plugins"][plugin]["mode"], "disabled")

    def test_runtime_image_is_source_free_and_has_no_implicit_postgres_cluster(self) -> None:
        dockerfile = (ROOT / "tests/docker_release_recovery/Dockerfile").read_text()
        self.assertIn("create_main_cluster = false", dockerfile)
        self.assertIn('test -z "$(pg_lsclusters --no-header)"', dockerfile)
        self.assertNotIn("COPY ../../", dockerfile)
        self.assertNotIn("cargo", dockerfile.lower())
        self.assertNotIn("gradle", dockerfile.lower())

    def test_fixture_checks_explicit_consent_before_product_mutation(self) -> None:
        source = (ROOT / "tests/docker_release_recovery/fixture.py").read_text()
        start = source.index("def prepare(expected_commit")
        end = source.index("\ndef encode_varint", start)
        body = source[start:end]
        consent = body.index("explicit Minecraft EULA acceptance is absent")
        for mutation in ("require_fresh_host()", 'run(["groupadd"', "database_setup(database_password)"):
            self.assertLess(consent, body.index(mutation))

    def test_evidence_packet_is_private_scanned_and_exactly_indexed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            output = parent / "result.json"
            result = {"schemaVersion": 1, "mode": "unit", "status": "PASS"}
            packet = publish_evidence_packet(output, result)
            index = json.loads(Path(packet["index"]).read_text())
            self.assertEqual(index["entries"][0]["path"], output.name)
            self.assertEqual(index["entries"][0]["sha256"], packet["resultSha256"])
            self.assertEqual(stat.S_IMODE(output.stat().st_mode), 0o600)
            self.assertEqual(stat.S_IMODE(Path(packet["index"]).stat().st_mode), 0o600)
            self.assertEqual(json.loads(output.read_text())["evidence"]["secretScan"], "PASS")

    def test_evidence_packet_rejects_credentials_without_retaining_bytes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            output = parent / "result.json"
            result = {
                "schemaVersion": 1,
                "status": "FAILED",
                "diagnostic": "postgres://operator:this-is-a-real-looking-secret-value@127.0.0.1/db",
            }
            with self.assertRaises(LabError):
                publish_evidence_packet(output, result)
            self.assertFalse(output.exists())
            self.assertFalse(output.with_name(f"{output.name}.index.json").exists())


if __name__ == "__main__":
    unittest.main()
