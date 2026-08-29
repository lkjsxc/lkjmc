#!/usr/bin/env python3
"""Fail-closed operations contract checker with executable mutations."""
import argparse,re,subprocess,sys
from pathlib import Path
from operations_semantic_checks import check as semantic_check
ROOT=Path(__file__).resolve().parents[1]
PROBES=("clean-clone-compose","restore-boot-pass","installer-rerun-pass",
 "artifact-provenance-pass","toolchain-acquisition-pass","verification-evidence-pass",
 "fault-lab-pass","ci-compose-retained")
RULES={
"clean-clone-compose":[("scripts/run-operations-lab.py","git","archive","create-source-git-bundle.sh","independent-source","--no-cache","scan-secrets.py"),(".dockerignore","**/.env.*"),("docker-compose.yml","RUST_TEST_THREADS: \"4\"")],
"restore-boot-pass":[("scripts/backup-postgres.sh","pg_export_snapshot","pg_current_wal_lsn","jsonb_agg","--no-align --tuples-only","lsnSha256","schemaSha256","migrationSha256"),("scripts/restore-postgres.sh","marker!=canonical","restore target is not a fresh database"),("scripts/operations-restore-drill.sh","doctor","cleanup; cleanup","private-artifact-handoff.py","LKJMC_RESTORE_EVIDENCE_DIR")],
"installer-rerun-pass":[("scripts/install_artifacts.py","result=no-op","service-gid","os.replace","post-publish validation failed"),("scripts/deploy-release.py","TRUSTED_COMMAND_TARGET_ROOTS","MAX_COMMAND_SYMLINKS","symlink is not root-owned","outside its allowed target roots","identity changed during validation"),("scripts/operations-artifact-install-drill.sh","evidence_owner","setpriv","unnamed_gid=42424","fingerprint","LKJMC_INSTALL_FAULT=status","changed release was not published"),("scripts/operations_semantic_checks.py","st_mtime_ns","identical install changed metadata")],
"artifact-provenance-pass":[("Dockerfile","FROM gradle-deps AS compact-input","WORKDIR /","rm -rf /opt/gradle /workspace","/home/gradle/.gradle/wrapper/dists/lkjmc","/usr/share/doc /usr/share/info /usr/share/man","FROM scratch AS compact-toolchain","COPY --from=compact-input / /","FROM compact-toolchain AS verify","COPY . /workspace","test -x /workspace/scripts/verify-full.sh","test -x /workspace/scripts/attach-source-git.sh","test -x /workspace/scripts/compare-release-roots.py","test -x /workspace/scripts/release_archive.py"),(".github/workflows/verify.yml","scripts/build-release.sh /release-parent/release","scripts/build-release.sh /tmp/release-rebuild/release","scripts/compare-release-roots.py","pack_release /release-parent/release-handoff","pack_release /tmp/release-rebuild/release-handoff","scripts/release_archive.py extract"),("build.gradle.kts","isPreserveFileTimestamps = false","isReproducibleFileOrder = true"),("scripts/release_inventory.py","release artifact closure differs from contract","tracked_contract_paths","image_items","release provenance requires a clean Git checkout","cargo_lock_packages","unsupported Cargo.lock format version"),("scripts/build-release.sh","git worktree add","LKJMC_BUILD_NONCE","fresh built artifact","release output parent must not be group/other writable","refusing cleanup of replaced release output"),("scripts/compare-release-roots.py","from fd_tree import Limits, walk","walk(root, digest, LIMITS)","release roots must be distinct","release roots differ"),("scripts/release_archive.py","ARCHIVE_FORMAT = \"posix-ustar-uncompressed\"","tarfile.USTAR_FORMAT","RENAME_NOREPLACE","archive member order differs","outer artifact file closure differs","extract_inspection","independent manifest verifier","independent built-identity verifier"),("scripts/create-source-git-bundle.sh","refs/bundles/lkjmc-source","complete non-shallow history","bundle verify","source bundle advertised refs differ"),("scripts/attach-source-git.sh","refs/bundles/lkjmc-source","git bundle list-heads","git bundle verify","exported source differs from bundled Git object"),("scripts/verify-built-identity.py","compiled JVM identity differs","does not report a clean build"),("tests/test_release_identity.py","linked ref","ambient-substitution","source-link","compiled JVM identity differs","release-roots-reproducible","derived without cargo resolution"),("tests/test_release_archive.py","two_packs_are_identical","raw_header_path_and_type_mutations_fail","cleanup_refuses_replacement_inode","outer_missing_extra_wrong_descriptor_and_wrong_mode_fail"),("scripts/verify-artifact-manifest.py","independently derived release closure","fullmatch"),("scripts/operations_semantic_checks.py","shutil.copytree(ROOT,fixture","git','commit','-q','-m','fixture"),("scripts/run-operations-lab.py","retained artifact index is not exact")],
"toolchain-acquisition-pass":[("Dockerfile","@sha256:","cargo fetch --locked","dpkg-query -W"),("scripts/check-operations.py","external_container_refs","unparsed Docker FROM instruction","unparsed Compose image reference","normalized!='scratch'","unpinned container reference"),("gradlew","distributionUrl","distributionSha256Sum","mktemp","verify_zip","validate_dist"),("rust-toolchain.toml","channel = \"1.97.0\"")],
"verification-evidence-pass":[("scripts/run-operations-lab.py","commands","skips","set(paths)!=actual"),("scripts/verify-full.sh","ran=%s skipped=%s"),("scripts/ci-compose-evidence.py","ci-compose-retained")],
"fault-lab-pass":[("scripts/run-operations-lab.py","check-network-adoption.py","check-process-runtime.sh","check-data-workflows.py","atomic-download-faults","partial-final-files-zero","docker image inspect","audit-saved-image.py")],
"ci-compose-retained":[(".github/workflows/verify.yml","fetch-depth: 0","create-source-git-bundle.sh","LKJMC_SCAN_CANARY","context.tar","docker image inspect","docker image save","audit-saved-image.py","private-artifact-handoff.py","compose-config.raw","if: always() && steps.secret-scan.outcome == 'success'","Upload only safe failure marker","RELEASE_ARTIFACT_NAME: lkjmc-release-${{ github.sha }}-run-${{ github.run_id }}-attempt-${{ github.run_attempt }}","if: success() && steps.secret-scan.outcome == 'success'","Upload canonical release artifact","path: ${{ runner.temp }}/release-parent/release-handoff/*","verify-release-artifact:","needs: verify-compose","actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093","artifact-ids: ${{ needs.verify-compose.outputs.release-artifact-id }}","scripts/release_archive.py consume","Retain bounded consumer receipt separately","retention-days: 30"),("scripts/audit-saved-image.py","MAX_ARCHIVE","manifest.json","MANIFEST_OPTIONAL","LEGACY_CONFIG_FIELDS","legacy Docker config chain does not match image layers","Docker layer source","allow_missing","Docker image config missing from OCI closure","declared image layer missing","duplicate conflicting member","unreferenced saved image member","conflicting shared layer digest"),("scripts/saved_image_semantic_checks.py","images=3 layerReferences=4 layers=2 legacyConfigs=4","source-url","oci-selected-missing","legacy-missing","legacy-extra","legacy-type","legacy-digest","missing","duplicate","hidden.tar"),("scripts/fd_tree.py","O_NOFOLLOW","traversal race","root crossing","permission violation","count overflow","byte overflow","depth overflow","visit_directory"),("scripts/prepare-operations-evidence.py","walk(","input closure differs","artifact-index.json"),("scripts/scan-secrets.py","walk(","archive special member","credential URL","generated canary","[A-Za-z0-9._~+%-]+","(?<![A-Za-z0-9/])Bearer"),("scripts/operations_evidence_mutations.py","setpriv","ordinary","index is not exact")],
}
RULES["ci-compose-retained"].append((".github/workflows/verify.yml",
 "permissions:","actions: read","merge-multiple: true","Independently verify outer artifact-service digest",
 "artifact-transport.zip","observed==expected_digest","metadata.get('expired') is False"))
def require(ok,message):
 if not ok: raise RuntimeError(message)
def source(path,overrides): return overrides.get(path,(ROOT/path).read_text())
def external_container_refs(docker,compose):
 stages=set(); refs=[]
 pattern=re.compile(r'^\s*FROM\s+(?:--platform=\S+\s+)?(\S+)(?:\s+AS\s+([A-Za-z0-9_.-]+))?\s*$',re.I|re.M)
 docker_lines=re.findall(r'^\s*FROM\b[^\n]*$',docker,re.I|re.M); parsed=pattern.findall(docker)
 require(len(parsed)==len(docker_lines),'unparsed Docker FROM instruction')
 for base,alias in parsed:
  normalized=base.lower()
  if normalized!='scratch' and normalized not in stages: refs.append(base)
  if alias:
   normalized_alias=alias.lower(); require(normalized_alias not in stages,'duplicate Docker stage name')
   stages.add(normalized_alias)
 compose_lines=re.findall(r'^\s*image\s*:[^\n]*$',compose,re.M)
 compose_refs=re.findall(r'^\s*image\s*:\s*([^\s#]+)\s*(?:#.*)?$',compose,re.M)
 require(len(compose_refs)==len(compose_lines),'unparsed Compose image reference')
 return refs+compose_refs
def verify(probe,overrides={}):
 for path,*markers in RULES[probe]:
  text=source(path,overrides)
  for marker in markers: require(marker in text,f'{probe}: {path} missing {marker}')
 if probe=='toolchain-acquisition-pass':
  docker=source('Dockerfile',overrides); compose=source('docker-compose.yml',overrides)
  refs=external_container_refs(docker,compose)
  require(len(refs)>=3 and all(re.fullmatch(r'[^@\s]+@sha256:[0-9a-f]{64}',value) for value in refs),'unpinned container reference')
  require('command -v gradle' not in source('gradlew',overrides),'launcher bypasses verified distribution')
  apt=re.search(r'apt-get install(.+?)&& dpkg-query',docker,re.S)
  require(apt is not None and not re.search(r'\b[\w.+-]+=[0-9]',apt.group(1)),'invented exact apt package pin')
 if probe=='clean-clone-compose': require('target/' in source('.dockerignore',overrides),'target may enter context')
 if probe=='artifact-provenance-pass':
  require('chmod +x /workspace' not in source('Dockerfile',overrides),'verifier image mutates exported source modes')
 if probe=='verification-evidence-pass':
  runner=source('scripts/run-operations-lab.py',overrides)
  for name in PROBES: require(name in runner,f'runner omits {name}')
 if probe=='ci-compose-retained':
  workflow=source('.github/workflows/verify.yml',overrides)
  scan=workflow.index('id: secret-scan'); success=workflow.index('Upload scanned operations evidence'); failure=workflow.index('Upload only safe failure marker')
  require(scan<success and scan<failure,'upload is not gated after scan')
  require('path: ${{ runner.temp }}/operations-evidence/*' in workflow,'bounded evidence upload missing')
  cleanup=workflow.index('Clean Compose resources'); release_scan=workflow.index('Scan context release archive image layers and retained evidence')
  release_upload=workflow.index('Upload canonical release artifact'); consumer=workflow.index('\n  verify-release-artifact:')
  require(cleanup<release_scan<release_upload<consumer,'release upload ordering is not acceptance-gated')
  upload_step=workflow[workflow.rfind('\n      - name:',0,release_upload):consumer]
  require("if: success() && steps.secret-scan.outcome == 'success'" in upload_step,'release upload is not success gated')
  require('always()' not in upload_step,'release upload has an unconditional always path')
  require('path: ${{ runner.temp }}/release-parent/release-handoff/*' in upload_step,'release upload closure differs')
  require('/release-parent/release/*' not in upload_step,'unpacked release root is uploaded')
  consumer_text=workflow[consumer:]
  require('needs: verify-compose' in consumer_text,'release consumer does not require producer success')
  for forbidden in ('build-release.sh','cargo build','cargo metadata','gradlew','docker compose'):
   require(forbidden not in consumer_text,f'release consumer rebuilds or resolves outputs: {forbidden}')
  pins=re.findall(r'^\s*uses:\s*[^@\s]+@([^\s]+)\s*$',workflow,re.M)
  require(pins and all(re.fullmatch(r'[0-9a-f]{40}',pin) for pin in pins),'workflow action reference is not immutable')
def mutations(probes):
 count=0
 for probe in probes:
  for path,*markers in RULES[probe]:
   for marker in markers:
    original=(ROOT/path).read_text(); changed=original.replace(marker,'')
    require(changed!=original,f'mutation marker absent: {path}:{marker}')
    try: verify(probe,{path:changed})
    except RuntimeError: count+=1
    else: raise RuntimeError(f'mutation survived: {probe}:{path}:{marker}')
  if probe=='toolchain-acquisition-pass':
   docker=(ROOT/'Dockerfile').read_text()
   for old,new in (('FROM scratch AS compact-toolchain','FROM debian:latest AS compact-toolchain'),
                   ('FROM compact-toolchain AS verify','FROM debian:latest AS verify')):
    changed=docker.replace(old,new); require(changed!=docker,f'mutation marker absent: Dockerfile:{old}')
    try: verify(probe,{'Dockerfile':changed})
    except RuntimeError: count+=1
    else: raise RuntimeError(f'mutation survived: {probe}:Dockerfile:{old}')
  semantic_check(probe); count+=1
 print(f'ok operations-mutations rejected={count}')
def main():
 parser=argparse.ArgumentParser(); parser.add_argument('--probe',choices=PROBES); parser.add_argument('--all',action='store_true'); parser.add_argument('--mutations',action='store_true'); args=parser.parse_args()
 probes=PROBES if args.all else ((args.probe,) if args.probe else ())
 if not probes: parser.error('choose --probe or --all')
 try:
  for probe in probes: verify(probe)
  if args.mutations: mutations(probes)
 except Exception as error:
  print(f'operations check failed: {error}',file=sys.stderr); return 1
 print('ok check-operations probes='+','.join(probes)); return 0
if __name__=='__main__': sys.exit(main())
