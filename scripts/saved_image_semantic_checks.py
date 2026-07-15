#!/usr/bin/env python3
"""Deterministic Docker-save audit and secret-scan falsifiers."""
import hashlib,io,json,os,subprocess,sys,tarfile,tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
AUDIT=(sys.executable,str(ROOT/'scripts/audit-saved-image.py'),'--path')
SCAN=(sys.executable,str(ROOT/'scripts/scan-secrets.py'),'--canary','fixture-'+('c'*40),'--path')
MAX_ARCHIVE=2*1024*1024*1024

def require(ok,message):
 if not ok: raise RuntimeError(message)
def command(argv,ok=True,capture=False):
 done=subprocess.run(tuple(map(str,argv)),cwd=ROOT,text=True,stdout=subprocess.PIPE if capture else subprocess.DEVNULL,stderr=subprocess.DEVNULL)
 require((done.returncode==0)==ok,f'command outcome differs: {argv}')
 return done.stdout if capture else ''
def tar_payload(secret=False):
 raw=io.BytesIO()
 with tarfile.open(fileobj=raw,mode='w') as archive:
  data=(b'TOKEN='+b'z'*40+b'\n') if secret else b'fixture layer\n'
  item=tarfile.TarInfo('app/value'); item.size=len(data); item.mode=0o600; item.mtime=0
  archive.addfile(item,io.BytesIO(data))
 return raw.getvalue()
def config_bytes(diff_id,architecture):
 value={'architecture':architecture,'os':'linux','rootfs':{'diff_ids':['sha256:'+diff_id],'type':'layers'}}
 return json.dumps(value,separators=(',',':'),sort_keys=True).encode()
def add_file(archive,name,data):
 item=tarfile.TarInfo(name); item.size=len(data); item.mode=0o600; item.mtime=0
 archive.addfile(item,io.BytesIO(data))
def write_image(path,mutation='valid',secret=False):
 layer=tar_payload(secret); layer_digest=hashlib.sha256(layer).hexdigest(); layer_name='blobs/sha256/'+layer_digest
 configs=[]
 for arch in ('amd64','arm64'):
  raw=config_bytes(layer_digest,arch); digest=hashlib.sha256(raw).hexdigest()
  configs.append(('blobs/sha256/'+digest,raw,arch))
 manifest=[{'Config':name,'Layers':[layer_name],'RepoTags':[f'fixture:{arch}']} for name,_,arch in configs]
 with tarfile.open(path,mode='w') as archive:
  add_file(archive,'manifest.json',json.dumps(manifest,separators=(',',':'),sort_keys=True).encode())
  for name,raw,_ in configs: add_file(archive,name,raw)
  if mutation!='missing': add_file(archive,layer_name,layer)
  if mutation=='duplicate': add_file(archive,layer_name,b'conflicting layer')
  if mutation=='extra': add_file(archive,'hidden/value',b'undeclared')
  if mutation=='traversal': add_file(archive,'../escape',b'bad')
  if mutation in ('symlink','device'):
   item=tarfile.TarInfo('special')
   if mutation=='symlink': item.type=tarfile.SYMTYPE; item.linkname='manifest.json'
   else: item.type=tarfile.CHRTYPE; item.devmajor=1; item.devminor=3
   archive.addfile(item)
def check():
 with tempfile.TemporaryDirectory(prefix='lkjmc-saved-image-check-') as raw:
  root=Path(raw); valid=root/'valid.tar'; write_image(valid)
  first=command((*AUDIT,valid),capture=True); second=command((*AUDIT,valid),capture=True)
  require(first==second and 'images=2 layerReferences=2 layers=1' in first,'shared layer output is not deterministic')
  for mutation in ('missing','duplicate','extra','traversal','symlink','device'):
   path=root/f'{mutation}.tar'; write_image(path,mutation=mutation); command((*AUDIT,path),ok=False)
  oversized=root/'oversized.tar'
  with oversized.open('wb') as output: output.truncate(MAX_ARCHIVE+1)
  command((*AUDIT,oversized),ok=False)
  hidden=root/'hidden.tar'; write_image(hidden,secret=True)
  command((*AUDIT,hidden)); command((*SCAN,hidden),ok=False)
if __name__=='__main__': check(); print('ok saved-image-semantic-checks')
