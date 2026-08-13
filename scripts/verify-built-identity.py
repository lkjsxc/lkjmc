#!/usr/bin/env python3
"""Verify copied release artifacts expose the exact source identity."""
import argparse
import json
import re
import subprocess
import sys
import zipfile
from pathlib import Path

from release_inventory import commit, fail, workspace_package_value


def run(path, *arguments):
    result = subprocess.run(
        (str(path), *arguments), capture_output=True, text=True, timeout=10, check=False)
    if result.returncode:
        fail(f"identity command failed: {Path(path).name}")
    return result.stdout.strip()


def manifest_fields(raw):
    fields = {}
    name = None
    for line in raw.decode("utf-8").replace("\r\n", "\n").split("\n"):
        if line.startswith(" ") and name is not None:
            fields[name] += line[1:]
        elif ": " in line:
            name, value = line.split(": ", 1)
            fields[name] = value
        elif line:
            fail("malformed JAR manifest")
    return fields


def verify_binary(path, component, version, source_commit, json_output=False):
    if json_output:
        value = json.loads(run(path, "--json", "version"))
        expected = {"schemaVersion": 1, "version": version, "commit": source_commit}
        if any(value.get(name) != expected_value for name, expected_value in expected.items()):
            fail(f"wrong embedded identity: {path.name}")
        if value.get("dirty") is not False:
            fail(f"release artifact does not report a clean build: {path.name}")
    else:
        output = run(path, "--version")
        match = re.fullmatch(
            rf"{re.escape(component)} {re.escape(version)} commit=([0-9a-f]{{40}}) dirty=false",
            output)
        if not match or match.group(1) != source_commit:
            fail(f"wrong embedded identity: {path.name}")


def verify_jar(path, version, license_id, source_commit):
    with zipfile.ZipFile(path) as jar:
        fields = manifest_fields(jar.read("META-INF/MANIFEST.MF"))
        expected = {
            "Implementation-Version": version,
            "Bundle-License": license_id,
            "LKJMC-Build-Commit": source_commit,
        }
        if any(fields.get(name) != value for name, value in expected.items()):
            fail(f"wrong JAR identity: {path.name}")
        if fields.get("LKJMC-Build-Dirty") != "false":
            fail(f"release JAR does not report a clean build: {path.name}")
        expected_runtime = "\t".join((version, license_id, source_commit, "false"))
        runtime = run("java", "-cp", str(path), "com.lkjmc.common.LkjmcBuildInfo")
        if runtime != expected_runtime:
            fail(f"compiled JVM identity differs: {path.name}")
        if path.name == "lkjmc-paper.jar":
            plugin = jar.read("plugin.yml").decode("utf-8")
            if f"version: '{version}'" not in plugin:
                fail("Paper descriptor version differs")
        elif path.name == "lkjmc-velocity.jar":
            descriptor = json.loads(jar.read("velocity-plugin.json"))
            if descriptor.get("version") != version:
                fail("Velocity descriptor version differs")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", required=True)
    args = parser.parse_args()
    source = Path(args.source).resolve()
    version = workspace_package_value("version")
    license_id = workspace_package_value("license")
    source_commit = commit()
    verify_binary(source / "lkjmc", "lkjmc", version, source_commit, True)
    verify_binary(source / "lkjmc-daemon", "lkjmc-daemon", version, source_commit)
    verify_binary(source / "lkjmc-discord", "lkjmc-discord", version, source_commit)
    for name in ("lkjmc-common.jar", "lkjmc-paper.jar", "lkjmc-velocity.jar"):
        verify_jar(source / name, version, license_id, source_commit)
    print(f"ok built-identity version={version} commit={source_commit}")


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(f"built identity verification failed: {error}", file=sys.stderr)
        sys.exit(1)
