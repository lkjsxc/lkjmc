#!/usr/bin/env python3
"""Verified no-op or atomic artifact-tree publication with rollback."""
import argparse,hashlib,json,os,shutil,stat,subprocess,sys,tempfile
from pathlib import Path

def fail(message): raise RuntimeError(message)
def fsync_dir(path):
 fd=os.open(path,os.O_RDONLY|os.O_DIRECTORY)
 try: os.fsync(fd)
 finally: os.close(fd)
def metadata(data,manifest):
 return (json.dumps({'commit':data['commit'],'manifestSha256':hashlib.sha256(manifest.read_bytes()).hexdigest(),'schemaVersion':1},sort_keys=True,separators=(',',':'))+'\n').encode()
def expected_files(data):
 result={}
 for item in data['artifacts']:
  folder='jars' if item['kind']=='jar' else 'bin'; mode=0o640 if item['kind']=='jar' else 0o750
  result[f'{folder}/{item["path"]}']=(item['sha256'],item['size'],mode)
 return result
def valid_tree(root,data,manifest,uid,gid,dir_mode,meta_mode):
 if root.is_symlink() or not root.is_dir(): return False
 expected=expected_files(data); expected['.lkjmc-install.json']=(hashlib.sha256(metadata(data,manifest)).hexdigest(),len(metadata(data,manifest)),meta_mode)
 actual={str(p.relative_to(root)) for p in root.rglob('*') if p.is_file() and not p.is_symlink()}
 if actual!=set(expected) or any(p.is_symlink() for p in root.rglob('*')): return False
 for directory in (root,root/'bin',root/'jars'):
  value=directory.stat()
  if not directory.is_dir() or stat.S_IMODE(value.st_mode)!=dir_mode or (value.st_uid,value.st_gid)!=(uid,gid): return False
 for rel,(digest,size,mode) in expected.items():
  path=root/rel; value=path.stat()
  if not stat.S_ISREG(value.st_mode) or (value.st_uid,value.st_gid)!=(uid,gid): return False
  raw=path.read_bytes()
  if len(raw)!=size or hashlib.sha256(raw).hexdigest()!=digest or stat.S_IMODE(value.st_mode)!=mode: return False
 return True
def stage_tree(stage,source,data,manifest,uid,gid,dir_mode,meta_mode):
 os.mkdir(stage/'bin',dir_mode); os.mkdir(stage/'jars',dir_mode)
 for item in data['artifacts']:
  src=source/item['path']; folder='jars' if item['kind']=='jar' else 'bin'; mode=0o640 if item['kind']=='jar' else 0o750
  destination=stage/folder/item['path']
  with src.open('rb') as incoming, destination.open('xb') as outgoing:
   shutil.copyfileobj(incoming,outgoing); outgoing.flush(); os.fsync(outgoing.fileno())
  os.chmod(destination,mode); os.chown(destination,uid,gid)
 with (stage/'.lkjmc-install.json').open('xb') as output:
  output.write(metadata(data,manifest)); output.flush(); os.fsync(output.fileno())
 os.chmod(stage/'.lkjmc-install.json',meta_mode); os.chown(stage/'.lkjmc-install.json',uid,gid)
 for directory in (stage/'bin',stage/'jars',stage):
  os.chmod(directory,dir_mode); os.chown(directory,uid,gid); fsync_dir(directory)
def main():
 parser=argparse.ArgumentParser()
 parser.add_argument('--scope',required=True,choices=('system','user','rootless')); parser.add_argument('--manifest',required=True)
 parser.add_argument('--root',required=True); parser.add_argument('--source',required=True)
 parser.add_argument('--service-uid',type=int); parser.add_argument('--service-gid',type=int); args=parser.parse_args()
 current=os.geteuid()
 if args.scope=='system':
  if current!=0 or args.service_uid is None or args.service_gid is None: fail('system scope requires root and numeric service UID/GID')
  uid=0; gid=args.service_gid; dir_mode=0o750; meta_mode=0o640
 else:
  if current==0: fail(f'{args.scope} scope refuses root')
  uid=current; gid=os.getegid(); dir_mode=0o700; meta_mode=0o600
 manifest=Path(args.manifest).resolve(); release=manifest.parent; source=Path(args.source).resolve(); root=Path(os.path.abspath(args.root))
 if source!=release/'source' or root==Path('/') or '..' in Path(args.root).parts or root.is_symlink(): fail('unsafe release source or install root')
 subprocess.run((sys.executable,str(Path(__file__).with_name('verify-artifact-manifest.py')),'--manifest',str(manifest),'--release-root',str(release)),check=True)
 data=json.loads(manifest.read_bytes())
 if valid_tree(root,data,manifest,uid,gid,dir_mode,meta_mode):
  print(f'ok artifact-install scope={args.scope} root={root} result=no-op version={data["commit"]}'); return
 parent=root.parent; parent.mkdir(parents=True,exist_ok=True)
 stage=Path(tempfile.mkdtemp(prefix='.lkjmc-stage-',dir=parent)); os.chmod(stage,0o700)
 rollback=Path(tempfile.mkdtemp(prefix='.lkjmc-rollback-',dir=parent)); rollback.rmdir(); prior=False; published=False
 try:
  stage_tree(stage,source,data,manifest,uid,gid,dir_mode,meta_mode)
  if os.environ.get('LKJMC_INSTALL_FAULT')=='after-stage': fail('injected failure after stage')
  if root.exists(): os.replace(root,rollback); prior=True
  os.replace(stage,root); published=True; fsync_dir(parent)
  if os.environ.get('LKJMC_INSTALL_FAULT') in ('after-publish','validation','status'): fail('injected post-publish validation failure')
  if not valid_tree(root,data,manifest,uid,gid,dir_mode,meta_mode): fail('post-publish validation failed')
  if prior: shutil.rmtree(rollback)
  fsync_dir(parent)
 except Exception:
  if published and root.exists(): shutil.rmtree(root)
  if prior and rollback.exists(): os.replace(rollback,root)
  fsync_dir(parent); raise
 finally:
  if stage.exists(): shutil.rmtree(stage)
  if rollback.exists() and not prior: shutil.rmtree(rollback)
 print(f'ok artifact-install scope={args.scope} root={root} result=updated version={data["commit"]}')
if __name__=='__main__':
 try: main()
 except Exception as error:
  print(f'artifact install failed: {error}',file=sys.stderr); sys.exit(1)
