#!/usr/bin/env python3
"""Create the exact bounded, redacted CI evidence closure and checksum index."""
import argparse,hashlib,json,os,re,sys
from pathlib import Path
from fd_tree import Limits,walk,visit_file
URL=re.compile(rb'(?i)[a-z][a-z0-9+.-]*://[^\s"\']+'); VALUE=re.compile(rb'(?i)((?:password|token|secret|credential)\s*[:=]\s*)\S+')
LIMITS={'build.log':131072,'verify.log':131072,'compose-config.yaml':262144,'lane.json':65536,
 'cleanup.json':16384,'build.exit':32,'run.exit':32,'artifact-manifest.json':1048576,'artifact-manifest.json.sha256':256}
RAW=('build.log','verify.log','lane.json','cleanup.json','build.exit','run.exit'); RELEASE=('artifact-manifest.json','artifact-manifest.json.sha256')
TREE_LIMITS=Limits(max_entries=10000,max_files=10000,max_bytes=2*1024**3,max_file_bytes=1024**3,max_depth=16)
def fail(message): raise RuntimeError(message)
def read_fd(fd,size):
 os.lseek(fd,0,os.SEEK_SET); data=b''
 while chunk:=os.read(fd,min(65536,size+1-len(data))):
  data+=chunk
  if len(data)>size: fail('input changed size while reading')
 if len(data)!=size: fail('input size differs')
 return data
def redact(value): return VALUE.sub(rb'\1<redacted>',URL.sub(b'<redacted-url>',value))
def private_write(root,name,data):
 if name in LIMITS and len(data)>LIMITS[name]: fail(f'oversized evidence: {name}')
 fd=os.open(name,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_CLOEXEC|os.O_NOFOLLOW,0o600,dir_fd=root)
 try:
  view=memoryview(data)
  while view: view=view[os.write(fd,view):]
  os.fsync(fd)
 finally: os.close(fd)
def collect(root,needed,exact):
 values={}
 def visitor(fd,item):
  if item.path in needed: values[item.path]=read_fd(fd,item.size)
 entries=walk(root,visitor,TREE_LIMITS); paths={item.path for item in entries}
 if exact and paths!=set(needed): fail(f'input closure differs: {sorted(paths)}')
 if set(values)!=set(needed): fail(f'missing required inputs: {sorted(set(needed)-set(values))}')
 return values
def main():
 parser=argparse.ArgumentParser(); parser.add_argument('--output',required=True); parser.add_argument('--raw',required=True)
 parser.add_argument('--compose',required=True); parser.add_argument('--release',required=True); args=parser.parse_args()
 out=Path(args.output)
 try: os.mkdir(out,0o700)
 except FileExistsError: fail(f'refusing existing output: {out}')
 out_fd=os.open(out,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW|os.O_CLOEXEC)
 try:
  raw=collect(Path(args.raw),RAW,True); release=collect(Path(args.release),RELEASE,False)
  for name in RAW:
   data=redact(raw[name])[-LIMITS[name]:] if name.endswith('.log') else raw[name]
   private_write(out_fd,name,data)
  private_write(out_fd,'compose-config.yaml',redact_file(Path(args.compose))[-LIMITS['compose-config.yaml']:])
  for name in RELEASE: private_write(out_fd,name,release[name])
  entries=[]
  def index_file(fd,item):
   data=read_fd(fd,item.size); entries.append({'maxSize':LIMITS[item.path],'path':item.path,
    'sha256':hashlib.sha256(data).hexdigest(),'size':item.size})
  walked=walk(out,index_file,TREE_LIMITS)
  if {item.path for item in walked}!=set(LIMITS): fail('evidence closure differs before indexing')
  index={'schemaVersion':1,'entries':sorted(entries,key=lambda x:x['path']),
   'selfExcluded':'artifact-index.json is omitted to avoid recursive hashing'}
  private_write(out_fd,'artifact-index.json',(json.dumps(index,indent=2,sort_keys=True)+'\n').encode())
 finally: os.close(out_fd)
 print(f'ok operations-evidence files={len(LIMITS)+1}')
def redact_file(path):
 value=[]
 visit_file(path,lambda fd,item:value.append(read_fd(fd,item.size)),TREE_LIMITS)
 return redact(value[0])
if __name__=='__main__':
 try: main()
 except Exception as error: print(f'operations evidence failed: {error}',file=sys.stderr); sys.exit(1)
