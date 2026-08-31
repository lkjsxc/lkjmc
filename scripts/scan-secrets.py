#!/usr/bin/env python3
"""Fail closed on canaries and credential values in descriptor-safe closures."""
import argparse,os,re,stat,sys,tarfile,tempfile,zipfile
from pathlib import Path,PurePosixPath
from fd_tree import Limits,walk,visit_file
URL=re.compile(rb'(?i)\b(?:postgres(?:ql)?|https?|mysql)://([A-Za-z0-9._~+%-]+):([A-Za-z0-9._~+!$&()*,:=%-]{8,})@[A-Za-z0-9]')
PRINTF=re.compile(rb'(?:%[A-Za-z])+'); BEARER=re.compile(rb'(?<![A-Za-z0-9/])Bearer[ \t]+[A-Za-z0-9._~+/=-]{12,}')
ASSIGN=re.compile(rb'(?i)\b(?:password|token|secret|credential|api[_-]?key)\b[ \t]*[:=][ \t]*["\']([A-Za-z0-9._~+/=-]{16,})')
ENV_ASSIGN=re.compile(rb'\b(?:PASSWORD|TOKEN|SECRET|CREDENTIAL|API_KEY)=([A-Za-z0-9._~+/=-]{16,})')
SAFE_VALUES=(b'lkjmc-dev',b'example-password',b'<redacted>',b'[redacted]'); SAFE_URL_VALUES=(b'BadPass',b'opensesame',b'password')
SOURCE_FIXTURES=('docs/research/','/tests/','/src/test/','src/tests/','_tests.rs','test_lab_harness.py','/scripts/check-',
 'support/redaction.rs','support/daemon_config.rs','commands/doctor_api.rs','observability/validation.rs','assets/server_download.rs',
 'assets/download_io_tests.rs','support/http_auth.rs')
LIMITS=Limits(max_entries=200000,max_files=100000,max_bytes=3*1024**3,max_file_bytes=2*1024**3,max_depth=32)
def fail(message): raise RuntimeError(message)
def fixture_path(label): return any(marker in label.replace('\\','/') for marker in SOURCE_FIXTURES)
def findings(data,label,canaries):
 if label.endswith('.rs') and b'#[cfg(test)]' in data: data=data.split(b'#[cfg(test)]',1)[0]
 found=[]
 for canary in canaries:
  if canary in data: found.append('generated canary')
 if fixture_path(label): return found
 for value in SAFE_VALUES: data=data.replace(value,b'')
 urls=(m for m in URL.finditer(data) if not all(PRINTF.fullmatch(v) for v in m.groups()) and not any(v in SAFE_URL_VALUES for v in m.groups()))
 if next(urls,None): found.append('credential URL')
 bearers=(m for m in BEARER.finditer(data) if m.end()==len(data) or data[m.end()]!=0x20)
 if next(bearers,None): found.append('bearer credential')
 if ASSIGN.search(data) or ENV_ASSIGN.search(data): found.append('credential assignment')
 return found
def safe_name(name,label):
 value=PurePosixPath(name)
 if not value.parts or value.is_absolute() or '..' in value.parts or str(value)!=name.rstrip('/'): fail(f'unsafe archive member: {label}')
 return value
def write_member(root,name,source,size):
 target=root.joinpath(*name.parts); target.parent.mkdir(parents=True,exist_ok=True,mode=0o700)
 for parent in (target.parent,*target.parent.parents):
  if parent==root.parent: break
  os.chmod(parent,0o700)
 fd=os.open(target,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
 written=0
 try:
  while chunk:=source.read(65536):
   written+=len(chunk)
   if written>size: fail(f'archive member grew: {name}')
   view=memoryview(chunk)
   while view: view=view[os.write(fd,view):]
  if written!=size: fail(f'archive member size changed: {name}')
  os.fsync(fd)
 finally: os.close(fd)
def extract_zip(stream,root,label,canaries,depth):
 stream.seek(0)
 try: archive=zipfile.ZipFile(stream)
 except zipfile.BadZipFile: return False
 with archive:
  infos=archive.infolist(); names=set(); total=0
  for item in infos:
   name=safe_name(item.filename,label); mode=item.external_attr>>16; kind=stat.S_IFMT(mode); legacy_dir=mode==0o177777 and item.is_dir()
   if item.filename in names: fail(f'duplicate archive member: {label}/{item.filename}')
   names.add(item.filename); total+=item.file_size
   if len(name.parts)>LIMITS.max_depth or len(names)>LIMITS.max_entries or total>LIMITS.max_bytes: fail(f'archive limit exceeded: {label}')
   if kind==stat.S_IFLNK:
    if depth==0: fail(f'archive link member: {label}/{item.filename}')
   elif not legacy_dir and (kind not in (0,stat.S_IFREG,stat.S_IFDIR) or (kind!=0 and (kind==stat.S_IFDIR)!=item.is_dir())): fail(f'archive special member: {label}/{item.filename}')
  for item in infos:
   name=safe_name(item.filename,label); target=root.joinpath(*name.parts)
   if stat.S_IFMT(item.external_attr>>16)==stat.S_IFLNK:
    with archive.open(item) as source: target_data=source.read(65537)
    if len(target_data)>65536: fail(f'oversized archive link: {label}/{item.filename}')
    found=findings(target_data,f'{label}/{item.filename}',canaries)
    if found: fail(f'{label}/{item.filename}: {", ".join(found)}')
   elif item.is_dir(): target.mkdir(parents=True,exist_ok=True,mode=0o700); os.chmod(target,0o700)
   else:
    with archive.open(item) as source: write_member(root,name,source,item.file_size)
 return True
def extract_tar(stream,root,label,canaries,depth):
 stream.seek(0)
 try: archive=tarfile.open(fileobj=stream,mode='r:*')
 except tarfile.ReadError: return False
 with archive:
  members=archive.getmembers(); names=set(); total=0
  for item in members:
   name=safe_name(item.name,label)
   if item.name in names: fail(f'duplicate archive member: {label}/{item.name}')
   names.add(item.name); total+=item.size
   if len(name.parts)>LIMITS.max_depth or len(names)>LIMITS.max_entries or total>LIMITS.max_bytes: fail(f'archive limit exceeded: {label}')
   if item.issym() or item.islnk():
    if depth==0: fail(f'archive link member: {label}/{item.name}')
    found=findings(item.linkname.encode(),f'{label}/{item.name}',canaries)
    if found: fail(f'{label}/{item.name}: {", ".join(found)}')
   elif not (item.isdir() or item.isreg()): fail(f'archive special member: {label}/{item.name}')
  for item in members:
   name=safe_name(item.name,label); target=root.joinpath(*name.parts)
   if item.issym() or item.islnk(): continue
   if item.isdir(): target.mkdir(parents=True,exist_ok=True,mode=0o700); os.chmod(target,0o700)
   else:
    source=archive.extractfile(item)
    if source is None: fail(f'unreadable archive member: {label}/{item.name}')
    with source: write_member(root,name,source,item.size)
 return True
def scan_bytes(fd,label,canaries):
 os.lseek(fd,0,os.SEEK_SET); prior=b''; overlap=max([512,*map(len,canaries)])
 while chunk:=os.read(fd,65536):
  block=prior+chunk; found=findings(block,label,canaries)
  if found: fail(f'{label}: {", ".join(found)}')
  prior=block[-overlap:]
def scan_fd(fd,label,canaries,depth):
 if depth>5: fail(f'archive nesting exceeds limit: {label}')
 with os.fdopen(os.dup(fd),'rb',closefd=True) as stream:
  with tempfile.TemporaryDirectory(prefix='lkjmc-secret-scan-') as raw:
   root=Path(raw); os.chmod(root,0o700)
   archived=extract_zip(stream,root,label,canaries,depth) or extract_tar(stream,root,label,canaries,depth)
   if archived:
    walk(root,lambda child,item:scan_fd(child,f'{label}!/{item.path}',canaries,depth+1),LIMITS); return
 if PurePosixPath(label).suffix.lower() in ('.jar','.zip','.tar'): fail(f'invalid archive: {label}')
 scan_bytes(fd,label,canaries)
def scan_path(path,label,canaries):
 before=os.lstat(path)
 if stat.S_ISDIR(before.st_mode): walk(path,lambda fd,item:scan_fd(fd,f'{label}/{item.path}',canaries,0),LIMITS)
 else: visit_file(path,lambda fd,item:scan_fd(fd,label,canaries,0),LIMITS)
def main():
 parser=argparse.ArgumentParser(); parser.add_argument('--path',action='append',required=True); parser.add_argument('--canary',action='append',required=True); args=parser.parse_args()
 canaries=[]
 for value in args.canary:
  raw=value.encode()
  if len(raw)<32: fail('generated canary must be at least 32 bytes')
  canaries.append(raw)
 for value in args.path:
  path=Path(value).absolute(); scan_path(path,path.name,canaries)
 print(f'ok secret-scan roots={len(args.path)} canaries={len(canaries)}')
if __name__=='__main__':
 try: main()
 except Exception as error: print(f'secret scan failed: {error}',file=sys.stderr); sys.exit(1)
