#!/usr/bin/env python3
"""Fail closed on generated canaries and credential values in complete trees."""
import argparse,os,re,stat,sys,tarfile,tempfile,zipfile
from pathlib import Path,PurePosixPath
URL=re.compile(rb'(?i)\b(?:postgres(?:ql)?|https?|mysql)://([A-Za-z0-9._~+%-]+):([A-Za-z0-9._~+!$&()*,:=%-]{8,})@[A-Za-z0-9]')
PRINTF=re.compile(rb'(?:%[A-Za-z])+')
BEARER=re.compile(rb'(?<![A-Za-z0-9/])Bearer[ \t]+[A-Za-z0-9._~+/=-]{12,}')
ASSIGN=re.compile(rb'(?i)\b(?:password|token|secret|credential|api[_-]?key)\b[ \t]*[:=][ \t]*["\']([A-Za-z0-9._~+/=-]{16,})')
ENV_ASSIGN=re.compile(rb'\b(?:PASSWORD|TOKEN|SECRET|CREDENTIAL|API_KEY)=([A-Za-z0-9._~+/=-]{16,})')
SAFE_VALUES=(b'lkjmc-dev',b'example-password',b'<redacted>',b'[redacted]')
SAFE_URL_VALUES=(b'BadPass',b'opensesame',b'password')
SOURCE_FIXTURES=('docs/research/','/tests/','/src/test/','src/tests/','_tests.rs','test_lab_harness.py','/scripts/check-',
 'support/redaction.rs','support/daemon_config.rs','commands/doctor_api.rs',
 'observability/validation.rs','assets/server_download.rs','assets/download_io_tests.rs',
 'support/http_auth.rs','operations_semantic_checks.py')
def fail(message): raise RuntimeError(message)
def fixture_path(label):
 value=label.replace('\\','/')
 return any(marker in value for marker in SOURCE_FIXTURES)
def findings(data,label,canaries):
 if label.endswith('.rs') and b'#[cfg(test)]' in data: data=data.split(b'#[cfg(test)]',1)[0]
 found=[]
 for canary in canaries:
  if canary in data: found.append('generated canary')
 if fixture_path(label): return found
 for value in SAFE_VALUES: data=data.replace(value,b'')
 urls=(match for match in URL.finditer(data) if not all(PRINTF.fullmatch(value) for value in match.groups()) and not any(value in SAFE_URL_VALUES for value in match.groups()))
 if next(urls,None): found.append('credential URL')
 bearers=(match for match in BEARER.finditer(data) if match.end()==len(data) or data[match.end()]!=0x20)
 if next(bearers,None): found.append('bearer credential')
 if ASSIGN.search(data) or ENV_ASSIGN.search(data): found.append('credential assignment')
 return found
def safe_member(name):
 path=PurePosixPath(name)
 return bool(path.parts) and not path.is_absolute() and '..' not in path.parts
def scan_archive(path,label,canaries,depth):
 if depth>5: fail(f'archive nesting exceeds limit: {label}')
 with tempfile.TemporaryDirectory(prefix='lkjmc-secret-scan-') as raw:
  root=Path(raw); os.chmod(root,0o700)
  if zipfile.is_zipfile(path):
   try:
    with zipfile.ZipFile(path) as archive:
     for item in archive.infolist():
      if not safe_member(item.filename): fail(f'unsafe zip member: {label}')
      mode=item.external_attr>>16
      if stat.S_ISLNK(mode) and depth==0: fail(f'symlink zip member: {label}')
      if stat.S_ISLNK(mode): continue
      archive.extract(item,root)
   except zipfile.BadZipFile:
    if PurePosixPath(label).suffix.lower() in ('.jar','.zip'): fail(f'invalid zip archive: {label}')
    return False
  elif tarfile.is_tarfile(path):
   with tarfile.open(path) as archive:
    for item in archive.getmembers():
     if not safe_member(item.name): fail(f'unsafe tar member: {label}')
     if (item.issym() or item.islnk()) and depth==0: fail(f'symlink tar member: {label}')
    def nested_filter(item,target):
     if item.issym() or item.islnk(): return None
     return tarfile.data_filter(item,target)
    archive.extractall(root,filter=nested_filter)
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
