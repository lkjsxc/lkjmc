#!/usr/bin/env python3
"""Run the eight required disposable A-OPS lanes without retries."""
import hashlib,json,os,re,secrets,subprocess,sys,tarfile,tempfile,time
from pathlib import Path
PROBES=("clean-clone-compose","restore-boot-pass","installer-rerun-pass","artifact-provenance-pass","toolchain-acquisition-pass","verification-evidence-pass","fault-lab-pass","ci-compose-retained")
URL=re.compile(r'(?i)[a-z][a-z0-9+.-]*://[^\s"\']+')
SECRET=re.compile(r'(?i)((?:password|token|secret|credential)\s*[=:]\s*)\S+')
def redact(s,canary): return SECRET.sub(r'\1<redacted>',URL.sub('<redacted-url>',s.replace(canary,'<redacted-canary>')))
def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def contains(path,needle):
 prior=b''
 with path.open('rb') as source:
  while chunk:=source.read(65536):
   block=prior+chunk
   if needle in block: return True
   prior=block[-max(0,len(needle)-1):]
 return False
class Lab:
 def __init__(self,root,commit,seed,canary):
  self.root=root; self.commit=commit; self.seed=seed; self.canary=canary; self.lanes=[]; self.n=0
 def lane(self,probe,commands,cwd,env=None):
  records=[]; artifacts=[]; lane_dir=self.root/'raw'/probe
  lane_dir.mkdir(parents=True); lane_dir.chmod(0o700)
  for argv,timeout in commands:
   self.n+=1; start=time.monotonic()
   try:
    done=subprocess.run(argv,cwd=cwd,env=env,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,timeout=timeout)
    code=done.returncode; output=done.stdout
   except (OSError,subprocess.TimeoutExpired) as e:
    code=124 if isinstance(e,subprocess.TimeoutExpired) else 127; output=str(e)
   log=lane_dir/f'{self.n:02d}.log'
   log.write_text(redact(f'exit={code} seconds={time.monotonic()-start:.3f}\n{output}',self.canary)[-65536:])
   artifacts.append({'path':str(log.relative_to(self.root)),'sha256':digest(log)})
   records.append({'argv':[redact(x,self.canary) for x in argv],'exit':code})
   if code:
    self.lanes.append({'probe':probe,'status':'fail','commands':records,'skips':[],'artifacts':artifacts})
    raise RuntimeError(f'{probe} failed')
  self.lanes.append({'probe':probe,'status':'pass','commands':records,'skips':[],'artifacts':artifacts})
def remaining(project):
 filters=(('ps','-aq'),('network','ls','-q'),('volume','ls','-q'),('image','ls','-q'))
 found=[]
 for args in filters:
  done=subprocess.run(('docker',)+args+('--filter',f'label=com.docker.compose.project={project}'),text=True,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
  if done.returncode or done.stdout.strip(): found.append('/'.join(args[:2]))
 return found
def clean_projects(base,projects):
 failed=[]
 for project in projects:
  if not remaining(project): continue
  command=base+['--project-name',project,'down','-v','--remove-orphans','--rmi','local']
  if subprocess.run(command,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,timeout=300).returncode: failed.append(project)
 residual={project:remaining(project) for project in projects}
 residual={key:value for key,value in residual.items() if value}
 return {'status':'pass' if not failed and not residual else 'fail','ownedResourcesRemaining':sum(map(len,residual.values())),'failedProjects':sorted(set(failed)),'residual':residual}
def main():
 import argparse
 ap=argparse.ArgumentParser(); ap.add_argument('--output',default='/tmp/a-ops-evidence.json'); a=ap.parse_args(); output=Path(a.output)
 repo=Path(subprocess.check_output(('git','rev-parse','--show-toplevel'),text=True).strip())
 dirty=subprocess.run(('git','status','--porcelain'),cwd=repo,text=True,stdout=subprocess.PIPE,check=True).stdout
 if dirty: raise SystemExit('worktree, including untracked files, must be clean')
 if output.exists(): raise SystemExit(f'refusing existing output: {output}')
 commit=subprocess.check_output(('git','rev-parse','HEAD'),cwd=repo,text=True).strip()
 seed=int(os.environ.get('LKJMC_OPS_SEED','20260714')); canary='a-ops-'+secrets.token_hex(24)
 root=Path(tempfile.mkdtemp(prefix='lkjmc-a-ops-')); root.chmod(0o700); clone=root/'source'; clone.mkdir()
 archive=root/'source.tar'; subprocess.run(('git','archive','--format=tar','-o',str(archive),commit),cwd=repo,check=True)
 with tarfile.open(archive) as tf: tf.extractall(clone,filter='data')
 archive.unlink(); (clone/'.env').write_text('TOKEN='+canary+'\n'); lab=Lab(root,commit,seed,canary)
 base=['docker','compose','-f',str(clone/'docker-compose.yml')]; projects=[]; failure=None
 try:
  clean=[]
  for rep in (1,2):
   project=f'lkjmcaops{seed}{rep}{secrets.token_hex(3)}'; projects.append(project); cmd=base+['--project-name',project]
   clean += [(cmd+['--profile','verify','build','--no-cache','verify'],3600),(cmd+['--profile','verify','run','--rm','verify'],3600),(cmd+['down','-v','--remove-orphans','--rmi','local'],300)]
  lab.lane(PROBES[0],clean,clone)
  project=f'lkjmcaopsrestore{secrets.token_hex(4)}'; projects.append(project); cmd=base+['--project-name',project]
  lab.lane(PROBES[1],[(cmd+['--profile','verify','build','verify'],3600),(cmd+['up','-d','postgres'],300),(cmd+['run','--rm','--no-deps','-v',f'{root}/raw:/evidence','-e',f'LKJMC_SOURCE_COMMIT={commit}','verify','sh','scripts/operations-restore-drill.sh'],3600),(cmd+['down','-v','--remove-orphans'],300)],clone)
  artifact='cargo build --locked --release -p lkjmc-cli -p lkjmc-daemon; ./gradlew --no-daemon --no-build-cache shadowJar; scripts/operations-artifact-install-drill.sh /evidence/release'
  lab.lane(PROBES[2],[(cmd+['run','--rm','--no-deps','-v',f'{root}/raw:/evidence','-e',f'LKJMC_SOURCE_COMMIT={commit}','-e',f'LKJMC_SECRET_CANARY={canary}','verify','sh','-ec',artifact],3600),(cmd+['down','-v','--remove-orphans','--rmi','local'],300)],clone)
  for probe in PROBES[3:6]: lab.lane(probe,[(('python3','scripts/check-operations.py','--probe',probe,'--mutations'),300)],clone)
  fault='scripts/check-data-workflows.py --all; scripts/check-network-adoption.py --all; scripts/check-process-runtime.sh; scripts/check-safe-ops.py --probe atomic-download-faults; scripts/check-safe-ops.py --probe partial-final-files-zero'
  env=os.environ|{'LKJMC_OPS_SEED':str(seed)}
  lab.lane(PROBES[6],[(cmd+['up','-d','postgres'],300),(cmd+['run','--rm','--no-deps','-e','LKJMC_STORE_TEST_DATABASE_URL=postgres://lkjmc:lkjmc-dev@postgres:5432/lkjmc','verify','sh','-ec',fault],3600),(cmd+['down','-v','--remove-orphans','--rmi','local'],300)],clone,env)
  lab.lane(PROBES[7],[(('python3','scripts/check-operations.py','--probe',PROBES[7],'--mutations'),300)],clone)
 except Exception as e: failure=e
 cleanup=clean_projects(base,projects)
 if failure or cleanup['status']!='pass':
  print(f'operations lab failed: {failure or "cleanup"}; cleanup={cleanup}; raw={root}',file=sys.stderr); return 1
 try: leaked=[str(p) for p in (root/'raw').rglob('*') if p.is_file() and contains(p,canary.encode())]
 except OSError as e: print(f'operations lab failed: retained evidence unreadable: {e}; raw={root}',file=sys.stderr); return 1
 if leaked: print(f'operations lab failed: credential canary leaked; raw={root}',file=sys.stderr); return 1
 if [x['probe'] for x in lab.lanes]!=list(PROBES) or any(x['status']!='pass' for x in lab.lanes):
  print(f'operations lab failed: exact probes did not pass; raw={root}',file=sys.stderr); return 1
 evidence={'schemaVersion':1,'commit':commit,'seed':seed,'lanes':lab.lanes,'cleanup':cleanup}; old=os.umask(0o077)
 try: output.write_text(json.dumps(evidence,indent=2,sort_keys=True)+'\n')
 finally: os.umask(old)
 print(f'ok operations-lab evidence={output} raw={root}'); return 0
if __name__=='__main__': sys.exit(main())
