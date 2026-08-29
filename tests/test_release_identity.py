#!/usr/bin/env python3
"""Executable regressions for source and artifact build identity."""
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile
import unittest
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def run(arguments, cwd, env=None, ok=True):
    result = subprocess.run(
        tuple(map(str, arguments)), cwd=cwd, env=env, text=True,
        stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=180)
    if (result.returncode == 0) != ok:
        raise AssertionError(
            f"unexpected exit {result.returncode}: {' '.join(map(str, arguments))}\n{result.stdout}")
    return result.stdout.strip()


def init_repo(path):
    run(("git", "init", "-q", "-b", "main"), path)
    run(("git", "config", "user.name", "lkjmc test"), path)
    run(("git", "config", "user.email", "test@lkjmc.invalid"), path)


def commit_all(path, message="fixture"):
    run(("git", "add", "."), path)
    run(("git", "commit", "-q", "-m", message), path)
    return run(("git", "rev-parse", "HEAD"), path)


class ReleaseIdentityTest(unittest.TestCase):
    def test_build_script_never_caches_a_false_clean_claim_and_tracks_linked_ref(self):
        with tempfile.TemporaryDirectory(prefix="lkjmc-build-identity-") as raw:
            repo = Path(raw) / "repo"
            package = repo / "crates/app"
            (package / "src").mkdir(parents=True)
            shutil.copy2(ROOT / "crates/lkjmc-core/build.rs", package / "build.rs")
            (repo / "Cargo.toml").write_text(
                '[workspace]\nmembers = ["crates/app"]\nresolver = "2"\n')
            (package / "Cargo.toml").write_text(
                '[package]\nname = "identity-app"\nversion = "1.0.0"\nedition = "2021"\n')
            (package / "src/main.rs").write_text(
                'fn main() { println!("{} {}", env!("LKJMC_BUILD_COMMIT"), '
                'env!("LKJMC_BUILD_DIRTY")); }\n')
            (repo / "README.md").write_text("identity fixture\n")
            (repo / ".gitignore").write_text("/target/\n")
            run(("cargo", "generate-lockfile", "-q"), repo)
            init_repo(repo)
            first = commit_all(repo)
            target = Path(raw) / "target"
            env = os.environ | {"CARGO_TARGET_DIR": str(target)}
            self.assertEqual(run(("cargo", "run", "-q", "-p", "identity-app"), repo, env),
                             f"{first} unknown")

            (repo / "README.md").write_text("tracked edit\n")
            self.assertEqual(run(("cargo", "run", "-q", "-p", "identity-app"), repo, env),
                             f"{first} unknown")
            (repo / "untracked.txt").write_text("untracked\n")
            self.assertEqual(run(("cargo", "run", "-q", "-p", "identity-app"), repo, env),
                             f"{first} unknown")

            missing_nonce = run(("cargo", "check", "-q", "-p", "identity-app"), repo,
                                env | {"LKJMC_SOURCE_COMMIT": first}, ok=False)
            self.assertIn("requires LKJMC_BUILD_NONCE", missing_nonce)
            dirty = run(("cargo", "check", "-q", "-p", "identity-app"), repo,
                        env | {"LKJMC_SOURCE_COMMIT": first,
                               "LKJMC_BUILD_NONCE": "a" * 32}, ok=False)
            self.assertIn("requires a clean worktree", dirty)

            run(("git", "checkout", "--", "README.md"), repo)
            (repo / "untracked.txt").unlink()
            clean = run(("cargo", "run", "-q", "-p", "identity-app"), repo,
                        env | {"LKJMC_SOURCE_COMMIT": first,
                               "LKJMC_BUILD_NONCE": "b" * 32})
            self.assertEqual(clean, f"{first} false")

            export = Path(raw) / "export"
            export.mkdir()
            archive = Path(raw) / "source.tar"
            run(("git", "archive", "-o", archive, "HEAD"), repo)
            with tarfile.open(archive) as source:
                source.extractall(export, filter="data")
            gitless = run(("cargo", "check", "-q", "-p", "identity-app"), export,
                          os.environ | {"CARGO_TARGET_DIR": str(Path(raw) / "gitless-target"),
                                        "LKJMC_SOURCE_COMMIT": first,
                                        "LKJMC_BUILD_NONCE": "c" * 32}, ok=False)
            self.assertIn("requires a Git checkout", gitless)

            worktree = Path(raw) / "linked"
            run(("git", "worktree", "add", "-q", "-b", "identity-move", worktree), repo)
            linked_target = Path(raw) / "linked-target"
            linked_env = os.environ | {"CARGO_TARGET_DIR": str(linked_target)}
            self.assertEqual(
                run(("cargo", "run", "-q", "-p", "identity-app"), worktree, linked_env),
                f"{first} unknown")
            (worktree / "README.md").write_text("new linked commit\n")
            second = commit_all(worktree, "move linked ref")
            self.assertNotEqual(first, second)
            self.assertEqual(
                run(("cargo", "run", "-q", "-p", "identity-app"), worktree, linked_env),
                f"{second} unknown")

    def test_gradle_exact_identity_requires_clean_git_and_nonce(self):
        with tempfile.TemporaryDirectory(prefix="lkjmc-gradle-identity-") as raw:
            project = Path(raw)
            shutil.copy2(ROOT / "build.gradle.kts", project / "build.gradle.kts")
            (project / "settings.gradle.kts").write_text('rootProject.name = "identity-fixture"\n')
            (project / "Cargo.toml").write_text(
                '[workspace]\nresolver = "2"\n\n[workspace.package]\n'
                'version = "0.1.0-alpha.1"\nlicense = "Apache-2.0"\n')
            (project / ".gitignore").write_text("/.gradle/\n")
            commit = "e" * 40
            exact = os.environ | {"LKJMC_SOURCE_COMMIT": commit,
                                  "LKJMC_BUILD_NONCE": "a" * 32}
            gitless = run((ROOT / "gradlew", "--no-daemon", "-q", "-p", project, "help"),
                          project, exact, ok=False)
            self.assertIn("requires a Git checkout", gitless)

            init_repo(project)
            commit = commit_all(project)
            (project / "untracked.txt").write_text("dirty\n")
            dirty = run((ROOT / "gradlew", "--no-daemon", "-q", "-p", project, "help"),
                        project, os.environ | {"LKJMC_SOURCE_COMMIT": commit,
                                               "LKJMC_BUILD_NONCE": "b" * 32}, ok=False)
            self.assertIn("requires a clean worktree", dirty)
            (project / "untracked.txt").unlink()
            clean = run((ROOT / "gradlew", "--no-daemon", "-q", "-p", project, "help"),
                        project, os.environ | {"LKJMC_SOURCE_COMMIT": commit,
                                               "LKJMC_BUILD_NONCE": "c" * 32})
            self.assertIn("Welcome to Gradle", clean)

    def test_export_must_match_trusted_git_bundle(self):
        with tempfile.TemporaryDirectory(prefix="lkjmc-source-bundle-") as raw:
            root = Path(raw)
            repo = root / "repo"
            repo.mkdir()
            init_repo(repo)
            (repo / "tracked.txt").write_text("first\n")
            first = commit_all(repo, "first")
            (repo / "tracked.txt").write_text("canonical\n")
            commit = commit_all(repo, "second")
            bundle = root / "source.bundle"
            output = run((ROOT / "scripts/create-source-git-bundle.sh", bundle, commit), repo)
            self.assertIn("ref=refs/bundles/lkjmc-source", output)
            self.assertEqual(
                run(("git", "bundle", "list-heads", bundle), repo),
                f"{commit} refs/bundles/lkjmc-source")
            archive = root / "source.tar"
            run(("git", "archive", "-o", archive, "HEAD"), repo)

            imported = root / "imported"
            imported.mkdir()
            run(("git", "init", "-q"), imported)
            run(("git", "bundle", "verify", bundle), imported)
            run(("git", "fetch", "-q", bundle, "refs/bundles/lkjmc-source"), imported)
            run(("git", "checkout", "-q", "--detach", "FETCH_HEAD"), imported)
            self.assertEqual(run(("git", "rev-parse", "HEAD"), imported), commit)
            run(("git", "fsck", "--full", "--strict", "--no-dangling"), imported)

            clean = root / "clean"
            clean.mkdir()
            with tarfile.open(archive) as source:
                source.extractall(clean, filter="data")
            output = run((ROOT / "scripts/attach-source-git.sh", bundle), clean,
                         os.environ | {"LKJMC_SOURCE_COMMIT": commit})
            self.assertIn(commit, output)

            changed = root / "changed"
            changed.mkdir()
            with tarfile.open(archive) as source:
                source.extractall(changed, filter="data")
            (changed / "tracked.txt").write_text("substituted\n")
            output = run((ROOT / "scripts/attach-source-git.sh", bundle), changed,
                         os.environ | {"LKJMC_SOURCE_COMMIT": commit}, ok=False)
            self.assertIn("differs from bundled Git object", output)
            self.assertFalse((changed / ".git").exists())

            shallow = root / "shallow"
            run(("git", "clone", "-q", "--depth", "1", repo.as_uri(), shallow), root)
            shallow_output = run(
                (ROOT / "scripts/create-source-git-bundle.sh", root / "shallow.bundle", commit),
                shallow, ok=False)
            self.assertIn("requires complete non-shallow history", shallow_output)

            source_ref = "refs/bundles/lkjmc-source"
            run(("git", "update-ref", source_ref, commit, ""), shallow)
            incomplete = root / "incomplete.bundle"
            run(("git", "bundle", "create", incomplete, source_ref), shallow)
            run(("git", "update-ref", "-d", source_ref, commit), shallow)
            incomplete_export = root / "incomplete-export"
            incomplete_export.mkdir()
            with tarfile.open(archive) as source:
                source.extractall(incomplete_export, filter="data")
            run((ROOT / "scripts/attach-source-git.sh", incomplete), incomplete_export,
                os.environ | {"LKJMC_SOURCE_COMMIT": commit}, ok=False)
            self.assertFalse((incomplete_export / ".git").exists())

            unexpected_ref = "refs/bundles/unexpected"
            run(("git", "update-ref", source_ref, commit, ""), repo)
            run(("git", "update-ref", unexpected_ref, first, ""), repo)
            extra = root / "extra.bundle"
            run(("git", "bundle", "create", extra, source_ref, unexpected_ref), repo)
            run(("git", "update-ref", "-d", source_ref, commit), repo)
            run(("git", "update-ref", "-d", unexpected_ref, first), repo)
            extra_export = root / "extra-export"
            extra_export.mkdir()
            with tarfile.open(archive) as source:
                source.extractall(extra_export, filter="data")
            output = run((ROOT / "scripts/attach-source-git.sh", extra), extra_export,
                         os.environ | {"LKJMC_SOURCE_COMMIT": commit}, ok=False)
            self.assertIn("advertised refs differ", output)
            self.assertFalse((extra_export / ".git").exists())

    def test_release_build_ignores_ambient_outputs(self):
        with tempfile.TemporaryDirectory(prefix="lkjmc-release-isolation-") as raw:
            root = Path(raw)
            repo = root / "repo"
            (repo / "scripts").mkdir(parents=True)
            (repo / "config").mkdir()
            shutil.copy2(ROOT / "scripts/build-release.sh", repo / "scripts/build-release.sh")
            os.chmod(repo / "scripts/build-release.sh", 0o755)
            contract = json.loads((ROOT / "config/release-artifacts.json").read_text())
            (repo / "config/release-artifacts.json").write_text(json.dumps(contract) + "\n")
            for name in ("verify-built-identity.py", "artifact-manifest.py",
                         "verify-artifact-manifest.py"):
                path = repo / "scripts" / name
                path.write_text("#!/bin/sh\nexit 0\n")
                path.chmod(0o755)
            (repo / "scripts/verify-built-identity.py").write_text(
                "#!/bin/sh\nset -eu\n"
                "if [ \"${LKJMC_TEST_REPLACE_OUTPUT:-0}\" = 1 ]; then "
                "test \"$1\" = --source; out=$(dirname \"$2\"); "
                "mv \"$out\" \"$out-original\"; mkdir \"$out\"; "
                "printf 'preserve\\n' >\"$out/replacement\"; exit 1; fi\n"
                "exit 0\n")
            (repo / "scripts/verify-built-identity.py").chmod(0o755)
            (repo / "gradlew").write_text(
                "#!/bin/sh\nset -eu\n"
                "for module in common paper velocity; do "
                "mkdir -p platforms/jvm/$module/build/libs; "
                "printf 'fresh-gradle-%s\\n' \"$module\" >"
                "platforms/jvm/$module/build/libs/$module-all.jar; done\n")
            (repo / "gradlew").chmod(0o755)
            (repo / ".gitignore").write_text("/target/\n**/build/\n")
            dynamic_sources = {
                item["source"] for item in contract["artifacts"]
                if item["source"].startswith("target/release/") or "/build/libs/" in item["source"]
            }
            for item in contract["artifacts"]:
                if item["source"] in dynamic_sources:
                    continue
                path = repo / item["source"]
                path.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(ROOT / item["source"], path)
                if item["kind"] == "binary":
                    path.chmod(0o755)
            init_repo(repo)
            commit_all(repo)

            for item in contract["artifacts"]:
                if item["source"] not in dynamic_sources:
                    continue
                path = repo / item["source"]
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("ambient-substitution\n")
                if item["kind"] == "binary":
                    path.chmod(0o755)
            tools = root / "tools"
            tools.mkdir()
            cargo = tools / "cargo"
            cargo.write_text(
                "#!/bin/sh\nset -eu\nmkdir -p target/release\n"
                "for name in lkjmc lkjmc-daemon lkjmc-discord; do "
                "printf 'fresh-cargo-%s\\n' \"$name\" >target/release/$name; "
                "chmod 755 target/release/$name; done\n")
            cargo.chmod(0o755)
            source_link = root / "source-link"
            source_link.symlink_to(repo, target_is_directory=True)
            escaped = run((repo / "scripts/build-release.sh", source_link / "release"), repo,
                          os.environ | {"PATH": f"{tools}:{os.environ['PATH']}"}, ok=False)
            self.assertIn("outside the source checkout", escaped)
            self.assertFalse((repo / "release").exists())

            replaced = root / "replaced-release"
            output = run((repo / "scripts/build-release.sh", replaced), repo,
                         os.environ | {"PATH": f"{tools}:{os.environ['PATH']}",
                                       "LKJMC_TEST_REPLACE_OUTPUT": "1"}, ok=False)
            self.assertIn("refusing cleanup of replaced release output", output)
            self.assertEqual((replaced / "replacement").read_text(), "preserve\n")

            release = root / "release"
            run((repo / "scripts/build-release.sh", release), repo,
                os.environ | {"PATH": f"{tools}:{os.environ['PATH']}"})
            for item in contract["artifacts"]:
                output = release / "source" / item["destination"]
                if item["source"] in dynamic_sources:
                    content = output.read_text()
                    self.assertTrue(content.startswith("fresh-"), item["destination"])
                    self.assertNotIn("ambient", content)
                else:
                    self.assertEqual(output.read_bytes(), (repo / item["source"]).read_bytes())

    def test_compiled_jvm_identity_must_match_manifest(self):
        scripts = str(ROOT / "scripts")
        if scripts not in sys.path:
            sys.path.insert(0, scripts)
        spec = importlib.util.spec_from_file_location(
            "verify_built_identity", ROOT / "scripts/verify-built-identity.py")
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        commit = "d" * 40
        with tempfile.TemporaryDirectory(prefix="lkjmc-jvm-identity-") as raw:
            root = Path(raw)
            source = root / "src/com/lkjmc/common"
            source.mkdir(parents=True)
            classes = root / "classes"
            classes.mkdir()

            def build_jar(dirty):
                java = source / "LkjmcBuildInfo.java"
                java.write_text(
                    "package com.lkjmc.common; public final class LkjmcBuildInfo {"
                    'public static final String VERSION="0.1.0-alpha.1";'
                    'public static final String LICENSE="Apache-2.0";'
                    f'public static final String COMMIT="{commit}";'
                    f'public static final String DIRTY="{dirty}";'
                    "public static void main(String[] a){System.out.println(VERSION+\"\\t\"+"
                    "LICENSE+\"\\t\"+COMMIT+\"\\t\"+DIRTY);}}\n")
                shutil.rmtree(classes)
                classes.mkdir()
                run(("javac", "-d", classes, java), root)
                jar = root / "lkjmc-common.jar"
                with zipfile.ZipFile(jar, "w") as output:
                    output.writestr("META-INF/MANIFEST.MF",
                        "Manifest-Version: 1.0\r\n"
                        "Implementation-Version: 0.1.0-alpha.1\r\n"
                        "Bundle-License: Apache-2.0\r\n"
                        f"LKJMC-Build-Commit: {commit}\r\n"
                        "LKJMC-Build-Dirty: false\r\n\r\n")
                    output.write(classes / "com/lkjmc/common/LkjmcBuildInfo.class",
                                 "com/lkjmc/common/LkjmcBuildInfo.class")
                return jar

            module.verify_jar(build_jar("false"), "0.1.0-alpha.1", "Apache-2.0", commit)
            with self.assertRaisesRegex(RuntimeError, "compiled JVM identity differs"):
                module.verify_jar(build_jar("corrupt"), "0.1.0-alpha.1", "Apache-2.0", commit)

    def test_release_root_comparison_rejects_every_tree_difference(self):
        with tempfile.TemporaryDirectory(prefix="lkjmc-release-compare-") as raw:
            root = Path(raw)
            first = root / "first"
            source = first / "source"
            source.mkdir(parents=True)
            first.chmod(0o700)
            source.chmod(0o700)
            (first / "artifact-manifest.json").write_bytes(b"manifest\n")
            (first / "artifact-manifest.json").chmod(0o600)
            (source / "tool").write_bytes(b"tool\n")
            (source / "tool").chmod(0o700)
            comparator = ROOT / "scripts/compare-release-roots.py"

            def copy(name):
                target = root / name
                shutil.copytree(first, target)
                return target

            matching = copy("matching")
            self.assertIn("release-roots-reproducible", run((comparator, first, matching), ROOT))

            changed = copy("changed")
            (changed / "source/tool").write_bytes(b"changed\n")
            self.assertIn("release roots differ", run((comparator, first, changed), ROOT, ok=False))

            mode = copy("mode")
            (mode / "source/tool").chmod(0o600)
            self.assertIn("release roots differ", run((comparator, first, mode), ROOT, ok=False))

            extra = copy("extra")
            (extra / "empty").mkdir(mode=0o700)
            self.assertIn("release roots differ", run((comparator, first, extra), ROOT, ok=False))

            linked = copy("linked")
            (linked / "source/link").symlink_to("tool")
            self.assertIn("symlink", run((comparator, first, linked), ROOT, ok=False))


if __name__ == "__main__":
    unittest.main(verbosity=2)
