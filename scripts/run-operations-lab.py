#!/usr/bin/env python3
"""Run the eight required disposable A-OPS lanes without retries."""
import hashlib,json,os,re,secrets,shutil,subprocess,sys,tarfile,tempfile,time
from pathlib import Path
from fd_tree import Limits,handoff,walk
TREE_LIMITS=Limits(max_entries=200000,max_files=100000,max_bytes=3*1024**3,max_file_bytes=2*1024**3,max_depth=32)
PROBES=("clean-clone-compose","restore-boot-pass","installer-rerun-pass","artifact-provenance-pass","toolchain-acquisition-pass","verification-evidence-pass","fault-lab-pass","ci-compose-retained")
URL=re.compile(r'(?i)[a-z][a-z0-9+.-]*://[^\s"\']+')
SECRET=re.compile(r'(?i)((?:password|token|secret|credential)\s*[=:]\s*)\S+')
def redact(s,canary): return SECRET.sub(r'\1<redacted>',URL.sub('<redacted-url>',s.replace(canary,'<redacted-canary>')))
def fd_digest(fd):
 value=hashlib.sha256(); os.lseek(fd,0,os.SEEK_SET)
 while chunk:=os.read(fd,65536): value.update(chunk)
 return value.hexdigest()
def digest(p):
 values=[]; from fd_tree import visit_file
 visit_file(p,lambda fd,item:values.append(fd_digest(fd)),TREE_LIMITS); return values[0]
class Lab:
 def __init__(self,root,commit,seed,canary):
  self.root=root; self.commit=commit; self.seed=seed; self.canary=canary; self.lanes=[]; self.n=0
 def lane(self,probe,commands,cwd,env=None):
  records=[]; artifacts=[]; failed=False; lane_dir=self.root/'raw'/probe
  lane_dir.mkdir(parents=True); lane_dir.chmod(0o700)
  for argv,timeout in commands:
   self.n+=1; start=time.monotonic()
   try:
    done=subprocess.run(argv,cwd=cwd,env=env,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,timeout=timeout)
    code=done.returncode; output=done.stdout
   except (OSError,subprocess.TimeoutExpired) as e:
    code=124 if isinstance(e,subprocess.TimeoutExpired) else 127; output=str(e)
   log=lane_dir/f'{self.n:02d}.log'
   log.write_text(redact(f'exit={code} seconds={time.monotonic()-start:.3f}\n{output}',self.canary)[-65536:]); log.chmod(0o600)
   artifacts.append({'path':str(log.relative_to(self.root)),'sha256':digest(log)})
   records.append({'argv':[redact(x,self.canary) for x in argv],'exit':code})
   if code: failed=True; break
  handoff(lane_dir,os.getuid(),os.getgid(),TREE_LIMITS); known={item['path'] for item in artifacts}; found=[]
  walk(lane_dir,lambda fd,item:found.append((item.path,fd_digest(fd))),TREE_LIMITS)
  for relative,sha256 in found:
   path=f'raw/{probe}/{relative}'
   if path not in known: artifacts.append({'path':path,'sha256':sha256})
  self.lanes.append({'probe':probe,'status':'fail' if failed else 'pass','commands':records,'skips':[],'artifacts':artifacts})
  if failed: raise RuntimeError(f'{probe} failed; retained={lane_dir}')
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
 root=Path(tempfile.mkdtemp(prefix='lkjmc-a-ops-')); root.chmod(0o700)
 clone=root/'source'; independent=root/'independent-source'; clone.mkdir(); independent.mkdir()
 archive=root/'context.tar'; subprocess.run(('git','archive','--format=tar','-o',str(archive),commit),cwd=repo,check=True); archive.chmod(0o600)
 with tarfile.open(archive) as tf: tf.extractall(clone,filter='data')
 with tarfile.open(archive) as tf: tf.extractall(independent,filter='data')
 (clone/'.env').write_text('TOKEN='+canary+'\n'); (independent/'.env').write_text('TOKEN='+canary+'\n')
 raw_root=root/'raw'; raw_root.mkdir(mode=0o700)
 lab=Lab(root,commit,seed,canary); base=['docker','compose','-f',str(clone/'docker-compose.yml')]
 independent_base=['docker','compose','-f',str(independent/'docker-compose.yml')]; projects=[]; failure=None
 try:
  clean=[]
  for rep,compose in ((1,base),(2,independent_base)):
   project=f'lkjmcaops{seed}{rep}{secrets.token_hex(3)}'; projects.append(project); fresh=compose+['--project-name',project]
   clean += [(fresh+['--profile','verify','build','--no-cache','verify'],3600),(fresh+['--profile','verify','run','--rm','verify'],3600),(fresh+['down','-v','--remove-orphans','--rmi','local'],300)]
  lab.lane(PROBES[0],clean,clone)
  project=f'lkjmcaopsrestore{secrets.token_hex(4)}'; projects.append(project); cmd=base+['--project-name',project]
  lab.lane(PROBES[1],[(cmd+['--profile','verify','build','verify'],3600),(cmd+['up','-d','postgres'],300),(cmd+['run','--rm','--no-deps','-v',f'{root}/raw:/evidence','-e',f'LKJMC_SOURCE_COMMIT={commit}','-e','LKJMC_RESTORE_EVIDENCE_DIR=/evidence/restore-boot-pass/restore','verify','sh','scripts/operations-restore-drill.sh'],3600),(cmd+['down','-v','--remove-orphans'],300)],clone)
  artifact='cargo build --locked --release -p lkjmc-cli -p lkjmc-daemon -p lkjmc-discord; ./gradlew --no-daemon --no-build-cache shadowJar; scripts/operations-artifact-install-drill.sh /evidence/installer-rerun-pass/release'
  lab.lane(PROBES[2],[(cmd+['run','--rm','--no-deps','-v',f'{root}/raw:/evidence','-e',f'LKJMC_SOURCE_COMMIT={commit}','-e',f'LKJMC_SECRET_CANARY={canary}','verify','sh','-ec',artifact],3600)],clone)
  provenance='scripts/verify-artifact-manifest.py --manifest /evidence/installer-rerun-pass/release/artifact-manifest.json --release-root /evidence/installer-rerun-pass/release'
  actual=cmd+['run','--rm','--no-deps','-v',f'{root}/raw:/evidence','-e',f'LKJMC_SOURCE_COMMIT={commit}','verify','sh','-ec',provenance]
  lab.lane(PROBES[3],[(actual,300),(('python3','scripts/check-operations.py','--probe',PROBES[3],'--mutations'),300)],clone)
  tools='rustc --version; cargo --version; java -version; ./gradlew --no-daemon --version; dpkg-query -W build-essential python3 ca-certificates curl unzip postgresql-client-14; cargo metadata --locked --no-deps --format-version=1 >/dev/null'
  actual=cmd+['run','--rm','--no-deps','verify','sh','-ec',tools]
  lab.lane(PROBES[4],[(actual,300),(('python3','scripts/check-operations.py','--probe',PROBES[4],'--mutations'),300)],clone)
  evidence=root/'raw'/PROBES[5]/'compose.json'
  actual=('python3','scripts/ci-compose-evidence.py','--log',str(root/'raw'/PROBES[0]/'05.log'),'--exit','0','--build-exit','0','--output',str(evidence),'--commit',commit)
  lab.lane(PROBES[5],[(actual,300),(('python3','scripts/check-operations.py','--probe',PROBES[5],'--mutations'),300)],clone)
  fault='scripts/check-data-workflows.py --all; scripts/check-network-adoption.py --all; scripts/check-process-runtime.sh; scripts/check-safe-ops.py --probe atomic-download-faults; scripts/check-safe-ops.py --probe partial-final-files-zero'
  env=os.environ|{'LKJMC_OPS_SEED':str(seed)}; image_tar=root/'images.tar'
  save=('sh','-ec',f'image=$(docker image inspect --format "{{{{.Id}}}}" {project}-verify); docker image save "$image" -o {image_tar}; chmod 0600 {image_tar}')
  audit=(sys.executable,'scripts/audit-saved-image.py','--path',str(image_tar))
  lab.lane(PROBES[6],[(cmd+['up','-d','postgres'],300),(cmd+['run','--rm','--no-deps','-e','LKJMC_STORE_TEST_DATABASE_URL=postgres://lkjmc:lkjmc-dev@postgres:5432/lkjmc','verify','sh','-ec',fault],3600),(save,600),(audit,300),(cmd+['down','-v','--remove-orphans','--rmi','local'],300)],clone,env)
  evidence=root/'raw'/PROBES[7]/'lane.json'
  actual=('python3','scripts/ci-compose-evidence.py','--log',str(root/'raw'/PROBES[0]/'02.log'),'--exit','0','--build-exit','0','--output',str(evidence),'--commit',commit)
  lab.lane(PROBES[7],[(actual,300),(('python3','scripts/check-operations.py','--probe',PROBES[7],'--mutations'),300),(cmd+['down','-v','--remove-orphans','--rmi','local'],300)],clone)
 except Exception as e: failure=e
 cleanup=clean_projects(base,projects)
 if failure or cleanup['status']!='pass':
  print(f'operations lab failed: {failure or "cleanup"}; cleanup={cleanup}; raw={root}',file=sys.stderr); return 1
 scan=(str(clone/'scripts/scan-secrets.py'),'--canary',canary,'--path',str(archive),'--path',str(root/'raw'),'--path',str(image_tar))
 if subprocess.run(scan,cwd=clone,stdout=subprocess.DEVNULL,stderr=subprocess.PIPE).returncode:
  print(f'operations lab failed: full secret scan rejected retained closure; raw={root}',file=sys.stderr); return 1
 archive.unlink(); image_tar.unlink(); shutil.rmtree(clone); shutil.rmtree(independent)
 if [x['probe'] for x in lab.lanes]!=list(PROBES) or any(x['status']!='pass' for x in lab.lanes):
  print(f'operations lab failed: exact probes did not pass; raw={root}',file=sys.stderr); return 1
 indexed=[item for lane in lab.lanes for item in lane['artifacts']]; paths=[item['path'] for item in indexed]; actual=[]
 walk(root/'raw',lambda fd,item:actual.append(item.path),TREE_LIMITS)
 actual={'raw/'+path for path in actual}
 if len(paths)!=len(set(paths)) or set(paths)!=actual or any(digest(root/item['path'])!=item['sha256'] for item in indexed):
  print(f'operations lab failed: retained artifact index is not exact; raw={root}',file=sys.stderr); return 1
 evidence={'schemaVersion':1,'commit':commit,'seed':seed,'lanes':lab.lanes,'cleanup':cleanup}; old=os.umask(0o077)
 try: output.write_text(json.dumps(evidence,indent=2,sort_keys=True)+'\n')
 finally: os.umask(old)
 print(f'ok operations-lab evidence={output} raw={root}'); return 0
if __name__=='__main__': sys.exit(main())
