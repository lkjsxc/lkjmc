#!/usr/bin/env python3
"""Atomically create a private manifest for the independently derived release."""
import argparse,hashlib,json,os,sys,tempfile
from pathlib import Path
from release_inventory import commit,expected,fail

def publish(path,payload):
 path.parent.mkdir(parents=True,exist_ok=True)
 fd,name=tempfile.mkstemp(prefix='.'+path.name+'.',dir=path.parent)
 try:
  os.fchmod(fd,0o600)
  with os.fdopen(fd,'wb') as stream:
   stream.write(payload); stream.flush(); os.fsync(stream.fileno())
  os.replace(name,path)
  parent=os.open(path.parent,os.O_RDONLY|os.O_DIRECTORY)
  try: os.fsync(parent)
  finally: os.close(parent)
 except Exception:
  try: os.close(fd)
  except OSError: pass
  Path(name).unlink(missing_ok=True); raise

def main():
 parser=argparse.ArgumentParser()
 parser.add_argument('--output',required=True)
 parser.add_argument('--release-root',required=True)
 args=parser.parse_args(); out=Path(args.output).resolve(); release=Path(args.release_root).resolve()
 if out.parent!=release or out.name!='artifact-manifest.json': fail('manifest must be release-root/artifact-manifest.json')
 sidecar=out.with_suffix(out.suffix+'.sha256')
 if out.exists() or sidecar.exists(): fail('refusing existing manifest or sidecar')
 version=commit(); data=expected(release,version)
 payload=(json.dumps(data,indent=2,sort_keys=True)+'\n').encode()
 old=os.umask(0o077)
 try:
  publish(out,payload)
  publish(sidecar,(hashlib.sha256(payload).hexdigest()+'  '+out.name+'\n').encode())
 finally: os.umask(old)
 print(f'ok artifact-manifest commit={version} artifacts={len(data["artifacts"])} contracts={len(data["contracts"])}')
if __name__=='__main__':
 try: main()
 except Exception as error:
  print(f'artifact manifest failed: {error}',file=sys.stderr); sys.exit(1)
