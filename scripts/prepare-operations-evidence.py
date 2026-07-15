#!/usr/bin/env python3
"""Create the exact bounded, redacted CI evidence closure and checksum index."""
import argparse,hashlib,json,os,re,shutil,stat,sys
from pathlib import Path
URL=re.compile(r'(?i)[a-z][a-z0-9+.-]*://[^\s"\']+')
VALUE=re.compile(r'(?i)((?:password|token|secret|credential)\s*[:=]\s*)\S+')
LIMITS={'build.log':131072,'verify.log':131072,'compose-config.yaml':262144,
 'lane.json':65536,'cleanup.json':16384,'build.exit':32,'run.exit':32,
 'artifact-manifest.json':1048576,'artifact-manifest.json.sha256':256}
FILES=tuple(LIMITS)
def fail(message): raise RuntimeError(message)
def redact(value): return VALUE.sub(r'\1<redacted>',URL.sub('<redacted-url>',value))
def private_write(path,data):
 fd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
 with os.fdopen(fd,'wb') as stream: stream.write(data); stream.flush(); os.fsync(stream.fileno())
def main():
 parser=argparse.ArgumentParser(); parser.add_argument('--output',required=True); parser.add_argument('--raw',required=True)
 parser.add_argument('--compose',required=True); parser.add_argument('--release',required=True); args=parser.parse_args()
 out=Path(args.output); raw=Path(args.raw); release=Path(args.release); out.mkdir(parents=True,exist_ok=True); os.chmod(out,0o700)
 for name in ('lane.json','cleanup.json','build.exit','run.exit'):
  shutil.copyfile(raw/name,out/name); os.chmod(out/name,0o600)
 for name in ('build.log','verify.log'):
  text=(raw/name).read_text(errors='replace'); private_write(out/name,redact(text)[-LIMITS[name]:].encode())
 compose=redact(Path(args.compose).read_text(errors='replace'))
 private_write(out/'compose-config.yaml',compose[-LIMITS['compose-config.yaml']:].encode())
 for name in ('artifact-manifest.json','artifact-manifest.json.sha256'):
  shutil.copyfile(release/name,out/name); os.chmod(out/name,0o600)
 actual={p.name for p in out.iterdir() if p.is_file()}
 if actual!=set(FILES): fail(f'evidence closure differs before indexing: {sorted(actual)}')
 entries=[]
 for name in FILES:
  path=out/name; mode=path.lstat().st_mode
  if not stat.S_ISREG(mode) or path.is_symlink() or path.stat().st_size>LIMITS[name]: fail(f'invalid or oversized evidence: {name}')
  entries.append({'maxSize':LIMITS[name],'path':name,'sha256':hashlib.sha256(path.read_bytes()).hexdigest(),'size':path.stat().st_size})
 index={'schemaVersion':1,'entries':sorted(entries,key=lambda x:x['path']),
  'selfExcluded':'artifact-index.json is omitted to avoid recursive hashing'}
 private_write(out/'artifact-index.json',(json.dumps(index,indent=2,sort_keys=True)+'\n').encode())
 print(f'ok operations-evidence files={len(entries)+1}')
if __name__=='__main__':
 try: main()
 except Exception as error:
  print(f'operations evidence failed: {error}',file=sys.stderr); sys.exit(1)
