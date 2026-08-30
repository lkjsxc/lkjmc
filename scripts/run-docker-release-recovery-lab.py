#!/usr/bin/env python3
"""Canonical operator entrypoint for the disposable Docker recovery lab."""
from __future__ import annotations

import argparse
from pathlib import Path
import sys


ROOT = Path(__file__).resolve().parents[1]
SUPPORT = ROOT / "tests" / "docker_release_recovery"
sys.path.insert(0, str(SUPPORT))

from docker_lab import (  # noqa: E402
    LabError,
    execute,
    execute_full_matrix,
    extract_transport_zip,
    new_project,
    private_json,
    prepare_input_descriptor,
    publish_evidence_packet,
    validate_input_descriptor,
    validate_project,
)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "mode",
        choices=(
            "extract-transport",
            "fixture-consent-gate",
            "full-matrix",
            "input-check",
            "preflight",
            "prepare-inputs",
            "systemd-probe",
        ),
    )
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--project", type=validate_project, default=None)
    parser.add_argument("--transport-zip", type=Path)
    parser.add_argument("--artifact-dir", type=Path)
    parser.add_argument("--input-descriptor", type=Path)
    parser.add_argument("--accept-minecraft-eula", action="store_true")
    parser.add_argument("--input-root", type=Path)
    parser.add_argument("--baseline-artifact-id", type=int)
    parser.add_argument("--baseline-commit")
    parser.add_argument("--target-artifact-id", type=int)
    parser.add_argument("--target-commit")
    arguments = parser.parse_args()
    project = arguments.project or new_project()
    output = arguments.output.absolute()
    try:
        output.relative_to(ROOT)
    except ValueError:
        pass
    else:
        parser.error("private lab evidence must remain outside the repository")
    preparation_arguments = (
        arguments.input_root,
        arguments.baseline_artifact_id,
        arguments.baseline_commit,
        arguments.target_artifact_id,
        arguments.target_commit,
    )
    if arguments.mode != "prepare-inputs" and any(value is not None for value in preparation_arguments):
        parser.error("input preparation arguments are only valid for prepare-inputs")
    if arguments.mode == "prepare-inputs":
        if arguments.input_root is None or arguments.baseline_artifact_id is None \
                or arguments.baseline_commit is None:
            parser.error("prepare-inputs requires --input-root, --baseline-artifact-id, and --baseline-commit")
        if (arguments.target_artifact_id is None) != (arguments.target_commit is None):
            parser.error("prepare-inputs target artifact ID and commit must be supplied together")
        if arguments.transport_zip is not None or arguments.artifact_dir is not None \
                or arguments.input_descriptor is not None:
            parser.error("prepare-inputs received an unrelated input argument")
        try:
            observation = prepare_input_descriptor(
                arguments.input_root,
                baseline_artifact_id=arguments.baseline_artifact_id,
                baseline_commit=arguments.baseline_commit,
                target_artifact_id=arguments.target_artifact_id,
                target_commit=arguments.target_commit,
                accept_minecraft_eula=arguments.accept_minecraft_eula,
            )
            result = {
                "schemaVersion": 1,
                "mode": arguments.mode,
                "status": "PASS",
                "observation": observation,
            }
            code = 0
        except (LabError, OSError, ValueError) as error:
            result = {
                "schemaVersion": 1,
                "mode": arguments.mode,
                "status": "BLOCKED" if "unavailable" in str(error) else "FAILED",
                "error": str(error),
            }
            code = 2 if result["status"] == "BLOCKED" else 1
    elif arguments.mode == "extract-transport":
        if arguments.transport_zip is None or arguments.artifact_dir is None:
            parser.error("extract-transport requires --transport-zip and --artifact-dir")
        try:
            observation = extract_transport_zip(arguments.transport_zip, arguments.artifact_dir)
            result = {
                "schemaVersion": 1,
                "mode": arguments.mode,
                "status": "PASS",
                "observation": observation,
            }
            code = 0
        except LabError as error:
            result = {"schemaVersion": 1, "mode": arguments.mode, "status": "FAILED", "error": str(error)}
            code = 1
    elif arguments.mode == "input-check":
        if arguments.input_descriptor is None:
            parser.error("input-check requires --input-descriptor")
        try:
            observation = validate_input_descriptor(arguments.input_descriptor)
            blockers = []
            if observation["target"] is None:
                blockers.append("exact target release input is not yet available")
            if observation["minecraftEulaAccepted"] is not True and not arguments.accept_minecraft_eula:
                blockers.append("explicit Minecraft EULA acceptance is absent")
            result = {
                "schemaVersion": 1,
                "mode": arguments.mode,
                "status": "BLOCKED" if blockers else "PASS",
                "blockers": blockers,
                "observation": observation,
            }
            code = 2 if blockers else 0
        except (LabError, OSError, ValueError) as error:
            result = {"schemaVersion": 1, "mode": arguments.mode, "status": "FAILED", "error": str(error)}
            code = 1
    elif arguments.mode == "full-matrix":
        if arguments.transport_zip is not None or arguments.artifact_dir is not None:
            parser.error("Docker modes do not accept artifact transport arguments")
        if arguments.input_descriptor is None:
            parser.error("full-matrix requires --input-descriptor")
        try:
            code, result = execute_full_matrix(
                arguments.input_descriptor,
                project,
                output.parent / f".{project}-work",
                accept_minecraft_eula=arguments.accept_minecraft_eula,
            )
        except (LabError, OSError, ValueError) as error:
            result = {
                "schemaVersion": 1,
                "mode": arguments.mode,
                "project": project,
                "status": "BLOCKED" if "EULA" in str(error) or "target release" in str(error) else "FAILED",
                "error": str(error),
            }
            code = 2 if result["status"] == "BLOCKED" else 1
    else:
        if arguments.transport_zip is not None or arguments.artifact_dir is not None \
                or arguments.accept_minecraft_eula:
            parser.error("Docker modes do not accept artifact transport arguments")
        if arguments.mode == "fixture-consent-gate" and arguments.input_descriptor is None:
            parser.error("fixture-consent-gate requires --input-descriptor")
        if arguments.mode != "fixture-consent-gate" and arguments.input_descriptor is not None:
            parser.error("only fixture-consent-gate accepts --input-descriptor")
        code, result = execute(
            arguments.mode,
            project,
            input_descriptor=arguments.input_descriptor,
        )
    try:
        packet = publish_evidence_packet(output, result)
    except (LabError, OSError) as error:
        safe = {
            "schemaVersion": 1,
            "mode": arguments.mode,
            "project": project,
            "status": "FAILED",
            "error": "private evidence publication failed; rejected details were not retained",
        }
        try:
            private_json(output, safe)
        except (LabError, OSError):
            pass
        print(f"FAILED project={project} evidence-write={error}", file=sys.stderr)
        return 1
    project_text = f" project={project}" if arguments.mode in {
        "fixture-consent-gate",
        "full-matrix",
        "preflight",
        "prepare-inputs",
        "systemd-probe",
    } else ""
    print(f"{result['status']}{project_text} evidence={output} index={packet['index']}")
    return code


if __name__ == "__main__":
    raise SystemExit(main())
