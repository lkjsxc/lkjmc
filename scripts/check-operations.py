#!/usr/bin/env python3
"""Fail-closed operations contract checker with exact probes and mutations."""
import argparse, json, re, subprocess, sys, tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
PROBES=("clean-clone-compose","restore-boot-pass","installer-rerun-pass",
 "artifact-provenance-pass","toolchain-acquisition-pass","verification-evidence-pass",
 "fault-lab-pass","ci-compose-retained")
RULES={
"clean-clone-compose":[("scripts/run-operations-lab.py","git", "archive"),("scripts/run-operations-lab.py","--no-cache"),(".dockerignore","**/.env.*")],
"restore-boot-pass":[("scripts/backup-postgres.sh","pg_export_snapshot","pg_current_wal_lsn","jsonb_agg","--no-align --tuples-only","lsnSha256","schemaSha256","migrationSha256"),("scripts/restore-postgres.sh","marker!=canonical","restore target is not a fresh database"),("scripts/operations-restore-drill.sh","doctor","cleanup; cleanup")],
"installer-rerun-pass":[("scripts/install-artifacts.sh","scope refuses root"),("scripts/install-artifacts.sh","after-publish"),("scripts/check-installer.sh","run_install /tmp/install-2.log")],
"artifact-provenance-pass":[("scripts/artifact-manifest.py","commit"),("scripts/artifact-manifest.py","components"),("scripts/artifact-manifest.py","secret-shaped artifact"),("scripts/run-operations-lab.py","sha256sum --check artifact-manifest.json.sha256")],
"toolchain-acquisition-pass":[("Dockerfile","@sha256:","cargo fetch --locked","dpkg-query -W"),("gradle/wrapper/gradle-wrapper.properties","distributionSha256Sum="),("rust-toolchain.toml","channel = \"1.97.0\""),("scripts/install-support.sh","20a06e644b0d9bd2fbdbfd52d42540bdde820ea7df86e92e533c073da0cdd43c"),("scripts/run-operations-lab.py","cargo metadata --locked --no-deps")],
"verification-evidence-pass":[("scripts/run-operations-lab.py","commands","ci-compose-evidence.py"),("scripts/run-operations-lab.py","skips"),("scripts/verify-full.sh","ran=%s skipped=%s")],
"fault-lab-pass":[("scripts/run-operations-lab.py","check-network-adoption.py"),("scripts/run-operations-lab.py","check-process-runtime.sh"),("scripts/run-operations-lab.py","check-data-workflows.py")],
"ci-compose-retained":[(".github/workflows/verify.yml","--no-cache","if: always()","operations-evidence","COMPOSE_PROJECT_NAME","ownedResourcesRemaining"),("scripts/run-operations-lab.py","PROBES[7]/'lane.json'")],
}
def require(ok,msg):
 if not ok: raise RuntimeError(msg)
def source(path,overrides): return overrides.get(path,(ROOT/path).read_text())
def verify(probe,overrides={}):
 for rule in RULES[probe]:
  path,*markers=rule; text=source(path,overrides)
  for marker in markers: require(marker in text,f'{probe}: {path} missing {marker}')
 if probe=='toolchain-acquisition-pass':
  docker=source('Dockerfile',overrides); compose=source('docker-compose.yml',overrides)
  refs=re.findall(r'\b(?:FROM|image:)\s+([^\s]+)',docker+'\n'+compose)
  refs=[x for x in refs if x not in {'toolchain','rust-deps','gradle-deps','verify'}]
  require(len(refs)>=3 and all('@sha256:' in x for x in refs), 'unpinned container reference')
  apt=re.search(r'apt-get install(.+?)&& dpkg-query',docker,re.S)
  require(apt is not None and not re.search(r'\b[\w.+-]+=[0-9]',apt.group(1)), 'invented exact apt package pin')
 if probe=='clean-clone-compose':
  require('target/' in source('.dockerignore',overrides), 'target may enter context')
 if probe=='verification-evidence-pass':
  runner=source('scripts/run-operations-lab.py',overrides)
  for name in PROBES: require(name in runner,f'runner omits {name}')
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
 print(f'ok operations-mutations rejected={count}')
def main():
 ap=argparse.ArgumentParser(); ap.add_argument('--probe',choices=PROBES); ap.add_argument('--all',action='store_true'); ap.add_argument('--mutations',action='store_true'); a=ap.parse_args()
 probes=PROBES if a.all else ((a.probe,) if a.probe else ())
 if not probes: ap.error('choose --probe or --all')
 try:
  for probe in probes: verify(probe)
  if a.mutations: mutations(probes)
 except (OSError,RuntimeError,subprocess.CalledProcessError,json.JSONDecodeError) as e:
  print(f'operations check failed: {e}',file=sys.stderr); return 1
 print('ok check-operations probes='+','.join(probes)); return 0
if __name__=='__main__': sys.exit(main())
