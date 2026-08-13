#!/usr/bin/env python3
"""Executable falsifiers for operations acquisition, provenance, install, and scanning."""
import hashlib,json,os,shutil,stat,subprocess,tarfile,tempfile,zipfile
from pathlib import Path
from saved_image_semantic_checks import check as saved_image_mutations
from operations_evidence_mutations import check as evidence_mutations
ROOT=Path(__file__).resolve().parents[1]
def require(ok,message):
 if not ok: raise RuntimeError(message)
def command_env(env):
 result=(os.environ if env is None else env).copy()
 result.pop('LKJMC_SOURCE_COMMIT',None); result.pop('LKJMC_BUILD_NONCE',None)
 return result
def run(argv,env=None,ok=True,cwd=ROOT):
 done=subprocess.run(tuple(map(str,argv)),cwd=cwd,env=command_env(env),stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
 require((done.returncode==0)==ok,f'command outcome differs: {argv}')
def gradle_mutations():
 with tempfile.TemporaryDirectory(prefix='lkjmc-gradle-check-') as raw:
  root=Path(raw); repo=root/'repo'; (repo/'gradle/wrapper').mkdir(parents=True); shutil.copy2(ROOT/'gradlew',repo/'gradlew')
  launcher='gradle-1.2.3/bin/gradle'; archive=root/'gradle.zip'
  with zipfile.ZipFile(archive,'w') as value:
   info=zipfile.ZipInfo(launcher); info.external_attr=(stat.S_IFREG|0o755)<<16; value.writestr(info,b'#!/bin/sh\nexit 0\n')
  digest=hashlib.sha256(archive.read_bytes()).hexdigest(); props=repo/'gradle/wrapper/gradle-wrapper.properties'
  def write(checksum): props.write_text(f'distributionUrl=https\\://example.invalid/gradle-1.2.3-bin.zip\ndistributionSha256Sum={checksum}\n')
  tools=root/'tools'; tools.mkdir(); curl=tools/'curl'; curl.write_text('#!/bin/sh\nfor last do :; done\ncp "$FAKE_ZIP" "$last"\n'); curl.chmod(0o700)
  env=os.environ|{'PATH':str(tools)+':'+os.environ['PATH'],'FAKE_ZIP':str(archive),'GRADLE_USER_HOME':str(root/'home')}
  write('1'*64); run((repo/'gradlew','--version'),env,False)
  write('0'*64); run((repo/'gradlew','--version'),env,False)
  write(digest); run((repo/'gradlew','--version'),env)
  cached=next((root/'home').rglob('distribution.zip')); cached.write_bytes(b'corrupt'); run((repo/'gradlew','--version'),env,False)
  shutil.rmtree(root/'home'); write(digest); run((repo/'gradlew','--version'),env)
  installed=next((root/'home').rglob('gradle')); installed.write_text('#!/bin/sh\nexit 7\n'); installed.chmod(0o700)
  run((repo/'gradlew','--version'),env,False)
def release_fixture(root):
 fixture=root/'fixture'; release=root/'release'; source=release/'source'; source.mkdir(parents=True)
 ignored=shutil.ignore_patterns('.git','target','build','tmp','.pi','.gradle','.gradle-bootstrap','__pycache__','*.pyc')
 shutil.copytree(ROOT,fixture,ignore=ignored)
 run(('git','init','-q','-b','fixture'),cwd=fixture)
 run(('git','config','user.name','lkjmc fixture'),cwd=fixture)
 run(('git','config','user.email','fixture@lkjmc.invalid'),cwd=fixture)
 run(('git','add','.'),cwd=fixture); run(('git','commit','-q','-m','fixture'),cwd=fixture)
 contract=json.loads((fixture/'config/release-artifacts.json').read_text())
 for item in contract['artifacts']:
  path=source/item['destination']; path.write_bytes((item['component']+'\n').encode()); path.chmod(0o700 if item['kind']=='binary' else 0o600)
 run((fixture/'scripts/artifact-manifest.py','--release-root',release,'--output',release/'artifact-manifest.json'),cwd=fixture)
 return release,fixture
def rewrite_manifest(release,change):
 path=release/'artifact-manifest.json'; data=json.loads(path.read_text()); change(data); raw=(json.dumps(data,indent=2,sort_keys=True)+'\n').encode(); path.write_bytes(raw)
 (release/'artifact-manifest.json.sha256').write_text(hashlib.sha256(raw).hexdigest()+'  artifact-manifest.json\n')
def provenance_mutations():
 with tempfile.TemporaryDirectory(prefix='lkjmc-provenance-check-') as raw:
  baseline,fixture=release_fixture(Path(raw)); verifier=fixture/'scripts/verify-artifact-manifest.py'
  run((verifier,'--release-root',baseline,'--manifest',baseline/'artifact-manifest.json'),cwd=fixture)
  changes=(lambda d:d['artifacts'].pop(),lambda d:d['artifacts'].append(dict(d['artifacts'][0],path='extra')),
   lambda d:d['artifacts'].append(d['artifacts'][0]),lambda d:d['artifacts'].__setitem__(0,dict(d['artifacts'][0],path='../escape')))
  for number,change in enumerate(changes):
   release=Path(raw)/f'mutation-{number}'; shutil.copytree(baseline,release); rewrite_manifest(release,change)
   run((verifier,'--release-root',release,'--manifest',release/'artifact-manifest.json'),ok=False,cwd=fixture)
  extra=Path(raw)/'extra'; shutil.copytree(baseline,extra); (extra/'source/undeclared').write_text('extra')
  run((verifier,'--release-root',extra,'--manifest',extra/'artifact-manifest.json'),ok=False,cwd=fixture)
def installer_mutations():
 with tempfile.TemporaryDirectory(prefix='lkjmc-install-check-') as raw:
  root=Path(raw); release,fixture=release_fixture(root); install=root/'installed'; script=fixture/'scripts/install-artifacts.sh'
  if os.geteuid()==0: scope=('system','--service-uid','0','--service-gid','42424')
  else: scope=('user',)
  command=(script,'--scope',*scope,'--manifest',release/'artifact-manifest.json','--source',release/'source','--root',install)
  run(command,cwd=fixture); target=install/'bin/lkjmc'; first=(target.stat().st_ino,target.stat().st_mtime_ns,target.stat().st_uid,target.stat().st_gid,stat.S_IMODE(target.stat().st_mode))
  run(command,cwd=fixture); require(first==(target.stat().st_ino,target.stat().st_mtime_ns,target.stat().st_uid,target.stat().st_gid,stat.S_IMODE(target.stat().st_mode)),'identical install changed metadata')
  old=hashlib.sha256(target.read_bytes()).hexdigest(); changed=root/'changed'; shutil.copytree(release,changed)
  (changed/'source/lkjmc').write_bytes(b'changed\n'); (changed/'artifact-manifest.json').unlink(); (changed/'artifact-manifest.json.sha256').unlink()
  run((fixture/'scripts/artifact-manifest.py','--release-root',changed,'--output',changed/'artifact-manifest.json'),cwd=fixture)
  update=(script,'--scope',*scope,'--manifest',changed/'artifact-manifest.json','--source',changed/'source','--root',install)
  run(update,os.environ|{'LKJMC_INSTALL_FAULT':'status'},False,cwd=fixture); require(hashlib.sha256(target.read_bytes()).hexdigest()==old,'status rollback failed')
  run(update,cwd=fixture); require(hashlib.sha256(target.read_bytes()).hexdigest()!=old,'changed update missing')
def scan_mutations():
 with tempfile.TemporaryDirectory(prefix='lkjmc-scan-check-') as raw:
  root=Path(raw); scanner=ROOT/'scripts/scan-secrets.py'; canary='scan-'+('a'*40); safe=root/'safe'
  safe.write_bytes(b'password token tokenFile databaseUrl\npostgres://\xc0\x01:\xc0@\xc0\npostgres://%s:%s@127.0.0.1/db\nhttp://brzuser:BadPass@brz.example.invalid/repo\nBearer authentication problem\nbearer authorization:password=secret=token=\n'); safe.chmod(0o600)
  run((scanner,'--canary',canary,'--path',safe))
  leak=root/'leak'; leak.write_text('postgres://user:actual-credential@db.invalid/name\n'); leak.chmod(0o600); run((scanner,'--canary',canary,'--path',leak),ok=False)
  leak.write_text('Authorization: Bearer '+('z'*40)+'\n'); run((scanner,'--canary',canary,'--path',leak),ok=False)
  leak.write_text(canary); run((scanner,'--canary',canary,'--path',leak),ok=False)
  archive=root/'layer.tar'
  with tarfile.open(archive,'w') as value: value.add(leak,arcname='layer/leak')
  archive.chmod(0o600); run((scanner,'--canary',canary,'--path',archive),ok=False)
  linked=root/'linked-layer.tar'
  with tarfile.open(linked,'w') as value:
   item=tarfile.TarInfo('bin/tool'); item.type=tarfile.SYMTYPE; item.linkname='/usr/bin/tool'; value.addfile(item)
  outer=root/'image.tar'
  with tarfile.open(outer,'w') as value: value.add(linked,arcname='layer/blob')
  outer.chmod(0o600); run((scanner,'--canary',canary,'--path',outer))
def check(probe):
 if probe=='toolchain-acquisition-pass': gradle_mutations()
 elif probe=='artifact-provenance-pass': provenance_mutations()
 elif probe=='installer-rerun-pass': installer_mutations()
 elif probe=='clean-clone-compose': scan_mutations()
 elif probe=='ci-compose-retained': scan_mutations(); saved_image_mutations(); evidence_mutations()
