#!/usr/bin/env python3
"""Fail closed on generated canaries and credential values in complete trees."""
import argparse,os,re,stat,sys,tarfile,tempfile,zipfile
from pathlib import Path,PurePosixPath
URL=re.compile(rb'(?i)\b(?:postgres(?:ql)?|https?|mysql)://[^\s/:"\']+:[^\s/@"\']+@[^\s"\']+')
BEARER=re.compile(rb'(?i)\bBearer[ \t]+[A-Za-z0-9._~+/=-]{12,}')
ASSIGN=re.compile(rb'(?i)\b(?:password|token|secret|credential|api[_-]?key)\b[ \t]*[:=][ \t]*["\']?([A-Za-z0-9._~+/=-]{16,})')
SAFE_VALUES=(b'lkjmc-dev',b'example-password',b'<redacted>',b'[redacted]')
SOURCE_FIXTURES=('docs/research/','/tests/','src/tests/','_tests.rs','test_lab_harness.py',
 'support/redaction.rs','support/daemon_config.rs','commands/doctor_api.rs',
 'observability/validation.rs','assets/server_download.rs','assets/download_io_tests.rs')
def fail(message): raise RuntimeError(message)
def fixture_path(label):
 value=label.replace('\\','/')
 return any(marker in value for marker in SOURCE_FIXTURES)
def findings(data,label,canaries):
 if fixture_path(label):
  for value in (b'password',b'secret',b'pass',b'obs-token-canary',b'pw'):
   data=data.replace(value,b'')
 for value in SAFE_VALUES: data=data.replace(value,b'')
 found=[]
 for canary in canaries:
  if canary in data: found.append('generated canary')
 if URL.search(data): found.append('credential URL')
 if BEARER.search(data): found.append('bearer credential')
 if ASSIGN.search(data): found.append('credential assignment')
 return found
def safe_member(name):
 path=PurePosixPath(name)
 return bool(path.parts) and not path.is_absolute() and '..' not in path.parts
def scan_archive(path,label,canaries,depth):
 if depth>5: fail(f'archive nesting exceeds limit: {label}')
 with tempfile.TemporaryDirectory(prefix='lkjmc-secret-scan-') as raw:
  root=Path(raw); os.chmod(root,0o700)
  if zipfile.is_zipfile(path):
   with zipfile.ZipFile(path) as archive:
    for item in archive.infolist():
     if not safe_member(item.filename): fail(f'unsafe zip member: {label}')
     mode=item.external_attr>>16
     if stat.S_ISLNK(mode): fail(f'symlink zip member: {label}')
    archive.extractall(root)
  elif tarfile.is_tarfile(path):
   with tarfile.open(path) as archive:
    for item in archive.getmembers():
     if not safe_member(item.name): fail(f'unsafe tar member: {label}')
     if (item.issym() or item.islnk()) and not safe_member(item.linkname): fail(f'unsafe tar link: {label}')
    archive.extractall(root,filter='data')
  else: return False
  scan_tree(root,label+'!',canaries,depth+1)
 return True
def scan_file(path,label,canaries,depth):
 mode=path.lstat().st_mode
 if not stat.S_ISREG(mode) or path.is_symlink(): fail(f'nonregular scan input: {label}')
 if scan_archive(path,label,canaries,depth): return
 data=path.read_bytes(); found=findings(data,label,canaries)
 if found: fail(f'{label}: {", ".join(found)}')
def scan_tree(path,label,canaries,depth=0):
 if path.is_symlink(): fail(f'symlink scan root: {label}')
 if path.is_file(): scan_file(path,label,canaries,depth); return
 if not path.is_dir(): fail(f'missing scan root: {path}')
 for child in sorted(path.rglob('*')):
  if child.is_symlink(): continue
  if child.is_file(): scan_file(child,f'{label}/{child.relative_to(path)}',canaries,depth)
def main():
 parser=argparse.ArgumentParser(); parser.add_argument('--path',action='append',required=True)
 parser.add_argument('--canary',action='append',required=True); args=parser.parse_args()
 canaries=[]
 for value in args.canary:
  raw=value.encode()
  if len(raw)<32: fail('generated canary must be at least 32 bytes')
  canaries.append(raw)
 for value in args.path:
  path=Path(value).resolve(); scan_tree(path,path.name,canaries)
 print(f'ok secret-scan roots={len(args.path)} canaries={len(canaries)}')
if __name__=='__main__':
 try: main()
 except Exception as error:
  print(f'secret scan failed: {error}',file=sys.stderr); sys.exit(1)
