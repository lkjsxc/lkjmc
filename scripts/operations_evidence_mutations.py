#!/usr/bin/env python3
"""Executable falsifiers for evidence traversal, handoff, and exact closure."""
import json,os,shutil,stat,subprocess,sys,tempfile
from pathlib import Path
from unittest.mock import patch
from fd_tree import Limits,walk
ROOT=Path(__file__).resolve().parents[1]; CANARY='mutation-'+('c'*40)
RAW=('build.log','verify.log','lane.json','cleanup.json','build.exit','run.exit')
def require(value,message):
 if not value: raise RuntimeError(message)
def private_dir(path): path.mkdir(parents=True,exist_ok=True); path.chmod(0o700)
def private_file(path,data=b'ok\n'): path.write_bytes(data); path.chmod(0o600)
def command(argv,ok=True,unprivileged=False):
 values=list(map(str,argv))
 if unprivileged and os.geteuid()==0: values=['setpriv','--reuid=65534','--regid=65534','--clear-groups','--',*values]
 done=subprocess.run(values,cwd=ROOT,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
 require((done.returncode==0)==ok,f'command outcome differs: {values[:3]}')
def give_to_unprivileged(root):
 if os.geteuid()!=0: return
 for directory,names,files in os.walk(root):
  os.chown(directory,65534,65534)
  for name in names+files: os.chown(Path(directory)/name,65534,65534,follow_symlinks=False)
def prepare_fixture(root):
 raw=root/'raw'; release=root/'release'; private_dir(raw); private_dir(release)
 for name in RAW: private_file(raw/name,b'{}\n' if name.endswith('.json') else b'0\n')
 private_file(release/'artifact-manifest.json',b'{}\n'); private_file(release/'artifact-manifest.json.sha256',b'0'*64+b'\n')
 compose=root/'compose'; private_file(compose,b'services: {}\n'); return raw,release,compose
def prepare_command(root,output):
 return (sys.executable,ROOT/'scripts/prepare-operations-evidence.py','--output',output,
  '--raw',root/'raw','--release',root/'release','--compose',root/'compose')
def closure_mutations():
 with tempfile.TemporaryDirectory(prefix='lkjmc-evidence-') as value:
  root=Path(value); root.chmod(0o700); prepare_fixture(root); output=root/'out'
  command(prepare_command(root,output)); index=json.loads((output/'artifact-index.json').read_text())
  actual={name for name in os.listdir(output) if stat.S_ISREG(os.lstat(output/name).st_mode)}
  require({item['path'] for item in index['entries']}|{'artifact-index.json'}==actual,'new evidence index is not exact')
 for number in range(3):
  with tempfile.TemporaryDirectory(prefix='lkjmc-unreadable-') as value:
   root=Path(value); root.chmod(0o700); raw,_,_=prepare_fixture(root)
   if number==0: raw.chmod(0)
   elif number==1:
    hidden=raw/'ordinary'; private_dir(hidden); private_file(hidden/'.hidden',CANARY.encode()); hidden.chmod(0)
   else: (raw/'build.log').chmod(0)
   give_to_unprivileged(root); command(prepare_command(root,root/'out'),ok=False,unprivileged=True)
def expect_walk_failure(root,limits=Limits(),visitor=lambda fd,item:None):
 try: walk(root,visitor,limits)
 except RuntimeError: return
 raise RuntimeError('walker mutation survived')
def traversal_mutations():
 with tempfile.TemporaryDirectory(prefix='lkjmc-walk-') as value:
  root=Path(value); root.chmod(0o700); private_file(root/'file')
  walk(root,lambda fd,item:None)
  link=root/'link'; link.symlink_to('file'); expect_walk_failure(root); link.unlink()
  fifo=root/'fifo'; os.mkfifo(fifo,0o600); expect_walk_failure(root); fifo.unlink()
  (root/'file').chmod(0o640); expect_walk_failure(root); (root/'file').chmod(0o600)
  expect_walk_failure(root,Limits(max_entries=0)); expect_walk_failure(root,Limits(max_bytes=1,max_file_bytes=1))
  deep=root/'deep'; private_dir(deep); private_file(deep/'nested'); expect_walk_failure(root,Limits(max_depth=0)); shutil.rmtree(deep)
  created=False
  def race(fd,item):
   nonlocal created
   if not created: private_file(root/'raced'); created=True
  expect_walk_failure(root,visitor=race); (root/'raced').unlink()
  real_stat=os.stat
  def crossing(path,*args,**kwargs):
   value=real_stat(path,*args,**kwargs)
   if kwargs.get('dir_fd') is not None and path=='file':
    fields=list(value); fields[stat.ST_DEV]=value.st_dev+1; return os.stat_result(fields)
   return value
  with patch('fd_tree.os.stat',crossing): expect_walk_failure(root)
def scan_unreadable_canary():
 with tempfile.TemporaryDirectory(prefix='lkjmc-canary-') as value:
  root=Path(value); root.chmod(0o700); hidden=root/'ordinary'; private_dir(hidden); private_file(hidden/'.hidden',CANARY.encode()); hidden.chmod(0)
  give_to_unprivileged(root); command((sys.executable,ROOT/'scripts/scan-secrets.py','--canary',CANARY,'--path',root),ok=False,unprivileged=True)
def check(): traversal_mutations(); scan_unreadable_canary(); closure_mutations()
if __name__=='__main__': check(); print('ok operations-evidence-mutations')
