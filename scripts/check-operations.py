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
"clean-clone-compose":[("scripts/run-operations-lab.py","git","archive","independent-source","--no-cache","scan-secrets.py"),(".dockerignore","**/.env.*"),("docker-compose.yml","RUST_TEST_THREADS: \"4\"")],
"restore-boot-pass":[("scripts/backup-postgres.sh","pg_export_snapshot","pg_current_wal_lsn","jsonb_agg","--no-align --tuples-only","lsnSha256","schemaSha256","migrationSha256"),("scripts/restore-postgres.sh","marker!=canonical","restore target is not a fresh database"),("scripts/operations-restore-drill.sh","doctor","cleanup; cleanup")],
"installer-rerun-pass":[("scripts/install_artifacts.py","result=no-op","service-gid","os.replace","post-publish validation failed"),("scripts/operations-artifact-install-drill.sh","unnamed_gid=42424","fingerprint","LKJMC_INSTALL_FAULT=status","changed release was not published"),("scripts/operations_semantic_checks.py","st_mtime_ns","identical install changed metadata")],
"artifact-provenance-pass":[("scripts/release_inventory.py","release artifact closure differs from contract","tracked_contract_paths","image_items"),("scripts/verify-artifact-manifest.py","independently derived release closure","fullmatch"),("scripts/run-operations-lab.py","retained artifact index is not exact")],
"toolchain-acquisition-pass":[("Dockerfile","@sha256:","cargo fetch --locked","dpkg-query -W"),("gradlew","distributionUrl","distributionSha256Sum","mktemp","verify_zip","validate_dist"),("rust-toolchain.toml","channel = \"1.97.0\""),("scripts/install-support.sh","20a06e644b0d9bd2fbdbfd52d42540bdde820ea7df86e92e533c073da0cdd43c")],
"verification-evidence-pass":[("scripts/run-operations-lab.py","commands","skips","set(paths)!=actual"),("scripts/verify-full.sh","ran=%s skipped=%s"),("scripts/ci-compose-evidence.py","ci-compose-retained")],
"fault-lab-pass":[("scripts/run-operations-lab.py","check-network-adoption.py","check-process-runtime.sh","check-data-workflows.py","atomic-download-faults","partial-final-files-zero")],
"ci-compose-retained":[(".github/workflows/verify.yml","LKJMC_SCAN_CANARY","context.tar","docker image save","compose-config.raw","if: always() && steps.secret-scan.outcome == 'success'","Upload only safe failure marker"),("scripts/prepare-operations-evidence.py","LIMITS","artifact-index.json"),("scripts/scan-secrets.py","credential URL","generated canary")],
}
def require(ok,message):
 if not ok: raise RuntimeError(message)
def source(path,overrides): return overrides.get(path,(ROOT/path).read_text())
def verify(probe,overrides={}):
 for path,*markers in RULES[probe]:
  text=source(path,overrides)
  for marker in markers: require(marker in text,f'{probe}: {path} missing {marker}')
 if probe=='toolchain-acquisition-pass':
  docker=source('Dockerfile',overrides); compose=source('docker-compose.yml',overrides)
  refs=re.findall(r'\b(?:FROM|image:)\s+([^\s]+)',docker+'\n'+compose)
  refs=[value for value in refs if value not in {'toolchain','rust-deps','gradle-deps','verify'}]
  require(len(refs)>=3 and all('@sha256:' in value for value in refs),'unpinned container reference')
  require('command -v gradle' not in source('gradlew',overrides),'launcher bypasses verified distribution')
  apt=re.search(r'apt-get install(.+?)&& dpkg-query',docker,re.S)
  require(apt is not None and not re.search(r'\b[\w.+-]+=[0-9]',apt.group(1)),'invented exact apt package pin')
 if probe=='clean-clone-compose': require('target/' in source('.dockerignore',overrides),'target may enter context')
 if probe=='verification-evidence-pass':
  runner=source('scripts/run-operations-lab.py',overrides)
  for name in PROBES: require(name in runner,f'runner omits {name}')
 if probe=='ci-compose-retained':
  workflow=source('.github/workflows/verify.yml',overrides)
  scan=workflow.index('id: secret-scan'); success=workflow.index('Upload scanned operations evidence'); failure=workflow.index('Upload only safe failure marker')
  require(scan<success and scan<failure,'upload is not gated after scan')
  require('path: ${{ runner.temp }}/operations-evidence/*' in workflow,'bounded evidence upload missing')
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
