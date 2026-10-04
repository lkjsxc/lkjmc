#!/usr/bin/python3
"""Installed inside each tenant VM. No host/Core credentials enter this guest."""
import hashlib,json,os,pwd,shutil,signal,stat,subprocess,sys,tarfile,threading,uuid,zipfile
from pathlib import Path,PurePosixPath

ROOT=Path('/srv/lkjmc')
CONTROL=Path('/var/lib/lkjmc')
CONFIG=Path('/etc/lkjmc/server.json')
FIFO=Path('/run/lkjmc-game/console')

def atomic(path,value):
 path.parent.mkdir(parents=True,exist_ok=True)
 temporary=path.with_name('.tmp-'+str(uuid.uuid4()))
 with open(temporary,'x') as f:
  json.dump(value,f,ensure_ascii=False);f.flush();os.fsync(f.fileno())
 os.replace(temporary,path)
 fd=os.open(path.parent,os.O_RDONLY)
 try:os.fsync(fd)
 finally:os.close(fd)

def extract_world(source,destination,limit):
 """Extract a data archive into a new directory, rejecting links, devices and zip bombs."""
 total=0;count=0
 def member(name,size):
  nonlocal total,count
  while name.startswith('./'):name=name[2:]
  name=name.rstrip('/')
  if name=='.':return None
  if not name:return None
  p=PurePosixPath(name)
  if len(name.encode())>240 or len(p.parts)>32:raise ValueError('World archive paths exceed the 240-byte or 32-level limit')
  if p.is_absolute() or '\\' in name or any(x in ('','.','..') for x in name.split('/')):raise ValueError('Unsafe archive path')
  total+=size;count+=1
  if size<0 or total>limit or count>200000:raise ValueError('Expanded world exceeds available storage')
  target=destination.joinpath(*p.parts)
  if not target.resolve().is_relative_to(destination.resolve()):raise ValueError('Archive escapes its directory')
  return target
 def write(target,stream,size):
  target.parent.mkdir(parents=True,exist_ok=True)
  with open(target,'xb') as out:
   remaining=size
   while remaining:
    data=stream.read(min(1048576,remaining))
    if not data:raise ValueError('Truncated archive')
    out.write(data);remaining-=len(data)
   out.flush();os.fsync(out.fileno())
 if zipfile.is_zipfile(source):
  # Bound central-directory allocation before ZipFile constructs ZipInfo objects.
  with open(source,'rb') as stream:
   stream.seek(0,os.SEEK_END);stream.seek(max(0,stream.tell()-65577));tail=stream.read(65577)
  end=tail.rfind(b'PK\x05\x06')
  if end<0 or len(tail)<end+22:raise ValueError('Invalid ZIP directory')
  record=struct.unpack('<4s4H2LH',tail[end:end+22])
  if record[4]==65535 or record[5]>8*1024*1024 or tail[max(0,end-20):end-16]==b'PK\x06\x07':raise ValueError('ZIP64 or oversized ZIP metadata is unsupported; use a bounded tar archive')
  with zipfile.ZipFile(source) as archive:
   for entry in archive.infolist():
    mode=entry.external_attr>>16
    if stat.S_ISLNK(mode) or mode&0o170000 not in (0,stat.S_IFDIR,stat.S_IFREG):raise ValueError('Archive links and special files are forbidden')
    target=member(entry.filename,0 if entry.is_dir() else entry.file_size)
    if target is None:continue
    if entry.is_dir():target.mkdir(parents=True,exist_ok=True)
    else:
     with archive.open(entry) as stream:write(target,stream,entry.file_size)
 else:
  with tarfile.open(source,'r:*') as archive:
   for entry in archive:
    if not entry.isdir() and not entry.isfile():raise ValueError('Archive links and special files are forbidden')
    target=member(entry.name,0 if entry.isdir() else entry.size)
    if target is None:continue
    if entry.isdir():target.mkdir(parents=True,exist_ok=True)
    else:
     with archive.extractfile(entry) as stream:write(target,stream,entry.size)
 if not (destination/'level.dat').is_file():
  roots=list(destination.iterdir())
  if len(roots)!=1 or not roots[0].is_dir() or not (roots[0]/'level.dat').is_file():raise ValueError('World archive must contain level.dat at its root')
  nested=roots[0]
  for child in list(nested.iterdir()):os.rename(child,destination/child.name)
  nested.rmdir()
 return total

def systemctl(*args,check=True):return subprocess.run(['systemctl',*args],check=check,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,timeout=150)
def owner(path):
 account=pwd.getpwnam('lkjmc-game')
 os.chown(path,account.pw_uid,account.pw_gid,follow_symlinks=False)
 if path.is_dir():
  for parent,dirs,files in os.walk(path,followlinks=False):
   for name in dirs+files:os.chown(Path(parent)/name,account.pw_uid,account.pw_gid,follow_symlinks=False)
def receipt_path(job):return CONTROL/'receipts'/(str(uuid.UUID(job))+'.json')
def discard(path):
 if path.is_symlink() or path.is_file():path.unlink()
 elif path.is_dir():shutil.rmtree(path)
def server_stopped():
 result=systemctl('show','lkjmc-game','--property=ActiveState,SubState,MainPID,ControlPID,ControlGroup',check=True)
 values=dict(line.split('=',1) for line in result.stdout.splitlines() if '=' in line)
 if values.get('ActiveState') not in ('inactive','failed') or values.get('SubState') not in ('dead','failed') or values.get('MainPID')!='0' or values.get('ControlPID')!='0':raise ValueError('Stop the game completely before changing files (including activating or stopping processes)')
 group=values.get('ControlGroup')
 if group:
  if not group.startswith('/') or '..' in group.split('/'):raise ValueError('Invalid game process group')
  events=Path('/sys/fs/cgroup'+group)/'cgroup.events'
  if events.exists() and 'populated 0' not in events.read_text().splitlines():raise ValueError('Game processes are still running; wait for a complete stop')

def runtime_configuration(java,memory,software):
 # The service runs as lkjmc-game, while bootstrap runs as root with umask 077.
 # This file contains only public launch settings. Credentials retain their
 # own private modes; changing the directory never changes those files.
 CONFIG.parent.mkdir(parents=True,exist_ok=True)
 if CONFIG.parent.is_symlink() or CONFIG.is_symlink():raise ValueError('Linked runtime configuration is forbidden')
 os.chmod(CONFIG.parent,0o755)
 if not CONFIG.exists():atomic(CONFIG,{'java':java,'heap_mib':max(256,memory-512),'software':software})
 os.chmod(CONFIG,0o644)

def bootstrap(request):
 if Path('/etc/lkjmc-guest-image').read_text().strip()!='rebuild-v1':raise ValueError('Wrong guest image')
 if os.geteuid()!=0:raise ValueError('Guest bootstrap requires the Incus guest agent')
 java=int(request['java']);memory=int(request['memory_mib'])
 if java not in (8,17,21,25) or memory<512:raise ValueError('Invalid runtime limits')
 if not Path(f'/opt/lkjmc/java/{java}/bin/java').is_file():raise ValueError('Selected Java runtime is absent from the pinned image')
 try:pwd.getpwnam('lkjmc-game')
 except KeyError:subprocess.run(['useradd','--system','--home-dir',str(ROOT),'--shell','/usr/sbin/nologin','lkjmc-game'],check=True)
 ROOT.mkdir(parents=True,exist_ok=True);CONTROL.mkdir(parents=True,exist_ok=True)
 for folder in ['incoming','receipts']: (CONTROL/folder).mkdir(exist_ok=True)
 os.chmod(CONTROL,0o700);Path(__file__).parent.chmod(0o755);Path(__file__).chmod(0o755)
 runtime_configuration(java,memory,request['software'])
 (ROOT/'eula.txt').write_text('eula=true\n')
 if not (ROOT/'server.properties').exists():
  (ROOT/'server.properties').write_text('server-port=25565\nonline-mode='+('false' if request['software']=='paper' else 'true')+'\nenforce-secure-profile=false\nenable-rcon=false\nmax-players=40\nview-distance=8\nsimulation-distance=6\n')
 if request['software']=='paper':
  (ROOT/'config').mkdir(exist_ok=True)
  secret=request['forwarding_secret']
  if len(secret)<32 or not all(c in '0123456789abcdef' for c in secret):raise ValueError('Invalid forwarding key')
  (ROOT/'config/paper-global.yml').write_text(f'proxies:\n  velocity:\n    enabled: true\n    online-mode: true\n    secret: "{secret}"\n')
 owner(ROOT)
 Path('/etc/systemd/system/lkjmc-game.service').write_text('''[Unit]
Description=lkjmc Minecraft instance
After=network-online.target
Wants=network-online.target
[Service]
Type=simple
User=lkjmc-game
Group=lkjmc-game
WorkingDirectory=/srv/lkjmc
RuntimeDirectory=lkjmc-game
RuntimeDirectoryMode=0700
ExecStart=/usr/local/lib/lkjmc/guest.py run
KillMode=mixed
TimeoutStopSec=120
Restart=no
UMask=0077
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/srv/lkjmc /run/lkjmc-game
LimitNOFILE=65536
[Install]
WantedBy=multi-user.target
''')
 systemctl('daemon-reload');systemctl('disable','lkjmc-game');os.sync()
 return {'configured':True,'server_id':request['server_id']}

# Guest filesystem boundary. All tenant file opens are rooted dirfd operations.
# Linux openat2 resolves the WHOLE path beneath the original root on each open;
# never fall back to Path.resolve followed by a pathname open.
import contextlib,ctypes,datetime,gzip,re,time,fcntl,struct
MAX_TEXT=65536
MAX_ENTRIES=256
MAX_SCAN=4096
MAX_LOG_BYTES=262144
MAX_LOG_EXPANDED=8*1024*1024
MAX_FILE=1024*1024*1024
PASSIVE=('logs','files','file_read')

MANAGED_PATHS=json.loads(Path(__file__).with_name('managed-paths.json').read_text())
def protected(value):
 value=value.lower()
 return any(value==p or value.startswith(p+'/') for p in MANAGED_PATHS)

def validate_path(value,root=False,internal=False):
 if root and value=='':return value
 if not isinstance(value,str) or not value or len(value.encode())>240 or '\\' in value or any(ord(c)<32 or ord(c)==127 for c in value):raise ValueError('Invalid relative path (maximum 240 bytes)')
 if value.startswith('/') or any(p in ('','.','..') or p.startswith('.') for p in value.split('/')):raise ValueError('Invalid relative path')
 if not internal and protected(value):raise ValueError('Managed configuration, authentication files and credentials are protected')
 return value

class OpenHow(ctypes.Structure):
 _fields_=[('flags',ctypes.c_uint64),('mode',ctypes.c_uint64),('resolve',ctypes.c_uint64)]
libc=ctypes.CDLL(None,use_errno=True)

def rooted_open(root,path,flags=os.O_RDONLY,mode=0):
 how=OpenHow(flags|os.O_NOFOLLOW|os.O_CLOEXEC|(0 if flags & os.O_PATH else os.O_NONBLOCK),mode,0x08|0x04|0x02|0x01) # BENEATH, NO_SYMLINKS, NO_MAGICLINKS, NO_XDEV
 fd=libc.syscall(437,root,os.fsencode(path or '.'),ctypes.byref(how),ctypes.sizeof(how))
 if fd<0:
  e=ctypes.get_errno()
  raise OSError(e,'Unsafe, missing, or inaccessible server path')
 return fd

@contextlib.contextmanager
def root_fd():
 # Anchor ROOT itself without following any ancestor symlink.
 fd=os.open('/',os.O_RDONLY|os.O_DIRECTORY|os.O_CLOEXEC)
 try:
  for part in ROOT.parts[1:]:
   nxt=os.open(part,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW|os.O_CLOEXEC,dir_fd=fd);os.close(fd);fd=nxt
  yield fd
 finally:os.close(fd)

@contextlib.contextmanager
def opened(root,path,flags=os.O_RDONLY):
 # O_PATH cannot activate a device or block on a FIFO. Reopen only the checked
 # kernel-held inode, never the tenant pathname after its type check.
 probe=rooted_open(root,path,os.O_PATH | (flags & os.O_DIRECTORY))
 fd=None
 try:
  info=os.fstat(probe)
  if not stat.S_ISDIR(info.st_mode) and (not stat.S_ISREG(info.st_mode) or info.st_nlink!=1):raise ValueError('Links and special files are forbidden')
  fd=os.open('/proc/self/fd/'+str(probe),flags|os.O_CLOEXEC|os.O_NONBLOCK)
  current=os.fstat(fd)
  if (info.st_dev,info.st_ino)!=(current.st_dev,current.st_ino) or (stat.S_ISREG(current.st_mode) and current.st_nlink!=1):raise ValueError('File identity changed during open')
  yield fd
 finally:
  if fd is not None:os.close(fd)
  os.close(probe)

def regular(fd,limit):
 info=os.fstat(fd)
 if not stat.S_ISREG(info.st_mode) or info.st_nlink!=1:raise ValueError('Only single-link regular files are supported; links and special files are forbidden')
 if info.st_size>limit:raise ValueError('File exceeds the bounded read size')
 return info

def read_bytes(root,path,limit):
 with opened(root,path) as fd:
  before=regular(fd,limit);parts=[];total=0
  while True:
   chunk=os.read(fd,min(65536,limit+1-total))
   if not chunk:break
   parts.append(chunk);total+=len(chunk)
   if total>limit:raise ValueError('File grew beyond the read limit')
  after=regular(fd,limit)
  if (before.st_size,before.st_mtime_ns,before.st_ctime_ns)!=(after.st_size,after.st_mtime_ns,after.st_ctime_ns):raise ValueError('File changed while reading; retry')
  return b''.join(parts)

def checked_path(value):
 validate_path(value)
 with root_fd() as root:
  parent,_,name=value.rpartition('/')
  try:
   with opened(root,parent,os.O_RDONLY|os.O_DIRECTORY):pass
   with opened(root,value) as fd:
    info=os.fstat(fd)
    if not stat.S_ISDIR(info.st_mode):regular(fd,MAX_FILE)
  except FileNotFoundError:pass
  except OSError as e:raise ValueError('Links and unsafe paths are forbidden') from e
 return ROOT/value # Compatibility only. Effect operations do not use this pathname.

def files(request):
 path=validate_path(request.get('path',''),root=True);entries=[];truncated=False;scanned=0
 with root_fd() as root,opened(root,path,os.O_RDONLY|os.O_DIRECTORY) as directory:
  with os.scandir(directory) as stream:
   for entry in stream:
    scanned+=1
    if scanned>MAX_SCAN or len(entries)>=MAX_ENTRIES:truncated=True;break
    target='/'.join(filter(None,(path,entry.name)))
    try:
     validate_path(target)
     with opened(root,target) as fd:
      info=os.fstat(fd)
      if stat.S_ISDIR(info.st_mode):kind='directory'
      else:regular(fd,MAX_FILE);kind='file'
     entries.append({'name':entry.name,'path':target,'kind':kind,'bytes':info.st_size if kind=='file' else None,'modified_at':datetime.datetime.fromtimestamp(info.st_mtime,datetime.timezone.utc).isoformat()})
    except (OSError,ValueError):continue
 return {'path':path,'entries':sorted(entries,key=lambda e:(e['kind']!='directory',e['name'])),'truncated':truncated}

def file_read(request):
 path=validate_path(request['path'])
 with root_fd() as root:data=read_bytes(root,path,MAX_TEXT)
 try:text=data.decode('utf-8')
 except UnicodeDecodeError:raise ValueError('This file is not UTF-8 text; use an artifact workflow for binary files')
 if '\0' in text:raise ValueError('Binary files cannot be read as text')
 return {'path':path,'text':text,'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()}

def snapshot(root,path,tree=False):
 """Bounded content identity; no links, mount traversal, devices or unbounded trees."""
 total=0;count=0;digest=hashlib.sha256()
 def walk(name):
  nonlocal total,count
  count+=1
  if count>200000:raise ValueError('Too many world entries')
  with opened(root,name) as fd:
   info=os.fstat(fd)
   if stat.S_ISDIR(info.st_mode):
    if not tree:raise ValueError('A single regular file is required; directories cannot be deleted')
    names=[]
    with os.scandir(fd) as stream:
     for entry in stream:
      names.append(entry.name)
      if len(names)>200000-count:raise ValueError('Too many world entries')
    digest.update(b'D'+name[len(path):].encode()+b'\0')
    for child in sorted(names):walk(name+'/'+child)
   else:
    regular(fd,MAX_FILE)
    digest.update(b'F'+name[len(path):].encode()+b'\0')
    h=hashlib.sha256()
    while True:
     chunk=os.read(fd,65536)
     if not chunk:break
     total+=len(chunk)
     if total>MAX_FILE:raise ValueError('Content exceeds the 1 GiB verification bound')
     h.update(chunk)
    regular(fd,MAX_FILE);digest.update(h.digest())
    return info,h.hexdigest()
   return info,None
 try:info,content=walk(path)
 except FileNotFoundError:return None
 # Directory hashes must be independent of the stage/destination name.
 # walk uses relative entry names through the normalized wrapper below.
 return {'dev':info.st_dev,'ino':info.st_ino,'kind':'directory' if stat.S_ISDIR(info.st_mode) else 'file','sha256':content or digest.hexdigest()}


def same_snapshot(root,path,expected,tree=False):
 actual=snapshot(root,path,tree)
 return actual==expected

def rename_new(parent,source,target):
 rc=libc.renameat2(parent,os.fsencode(source),parent,os.fsencode(target),1) # RENAME_NOREPLACE
 if rc:
  e=ctypes.get_errno();raise OSError(e,'Destination already exists or changed; refresh and retry')

def remove_tree(root,path):
 # Only a verified, displaced install tree is eligible; never called by explorer delete.
 with opened(root,path,os.O_RDONLY|os.O_DIRECTORY) as directory:
  with os.scandir(directory) as stream:names=[entry.name for entry in stream]
  for name in names:
   child=path+'/'+name
   with opened(root,child) as fd:info=os.fstat(fd)
   if stat.S_ISDIR(info.st_mode):remove_tree(root,child)
   else:
    if not stat.S_ISREG(info.st_mode) or info.st_nlink!=1:raise ValueError('Displaced world contains unsafe entries')
    os.unlink(name,dir_fd=directory)
  os.fsync(directory)
 parent,_,name=path.rpartition('/')
 with opened(root,parent,os.O_RDONLY|os.O_DIRECTORY) as directory:os.rmdir(name,dir_fd=directory);os.fsync(directory)

def mutation(request,action,internal=False,receipt_extra=None):
 request=dict(request);reconcile_only=request.pop('reconcile_only',False)
 server_stopped()
 path=validate_path(request['path'],internal=internal);job=str(uuid.UUID(request['job_id']))
 receipt=receipt_path(job);digest=hashlib.sha256(json.dumps({'action':action,'request':request,'context':receipt_extra},sort_keys=True).encode()).hexdigest()
 prior=json.loads(receipt.read_text()) if receipt.exists() else None
 if reconcile_only and prior is None:raise ValueError('Authorization expired before preparation; no effect to reconcile')
 if prior and prior['digest']!=digest:raise ValueError('Request changed during recovery; do not reuse a job ID')
 parent,_,name=path.rpartition('/');stage='.lkjmc-stage-'+job;backup='.lkjmc-before-'+job
 staged='/'.join(filter(None,(parent,stage)));displaced='/'.join(filter(None,(parent,backup)))
 tree=action=='install' and request['kind']=='world'
 with root_fd() as root,opened(root,parent,os.O_RDONLY|os.O_DIRECTORY) as directory:
  identity=[os.fstat(directory).st_dev,os.fstat(directory).st_ino]
  if prior and prior['parent']!=identity:raise ValueError('Destination directory changed during recovery')
  if not prior:
   before=snapshot(root,path,tree)
   if tree and before is not None and snapshot(root,path+'/level.dat') is None:raise ValueError('A world archive may replace only an existing world directory or create a new one')
   if action=='directory_create' and before is not None:raise ValueError('Destination already exists')
   if action in ('file_write','file_delete') or action=='install' and not tree:
    if name in ('level.dat','level.dat_old','session.lock') or name.endswith(('.mca','.mcr','.dat')):raise ValueError('Use a complete world archive to replace world data; individual world data files are protected')
   if action in ('file_write','file_delete'):
    expected=request.get('expected_sha256')
    if expected is not None and (not isinstance(expected,str) or not re.fullmatch('[0-9a-f]{64}',expected)):raise ValueError('Invalid expected SHA-256')
    if (before is None and expected is not None) or (before is not None and before['sha256']!=expected):raise ValueError('File changed or already exists; read it again before saving')
    if action=='file_delete' and (before is None or expected is None):raise ValueError('Delete requires the current file SHA-256')
   if action!='file_delete':
    # A pre-prepare leftover is not authoritative. Fail closed, never follow/remove it.
    try:os.stat(stage,dir_fd=directory,follow_symlinks=False)
    except FileNotFoundError:pass
    else:raise ValueError('An unprepared staging file exists; host reconciliation is required')
    if action=='directory_create' or tree:
     os.mkdir(stage,0o700,dir_fd=directory)
     if tree:
      source=CONTROL/'incoming'/str(uuid.UUID(request['artifact_id']))
      with incoming(source) as f:
       regular(f.fileno(),MAX_FILE)
       if hashlib.file_digest(f,'sha256').hexdigest()!=request['sha256']:raise ValueError('Uploaded artifact checksum mismatch')
      extract_world(source,Path('/proc/self/fd')/str(directory)/stage,min(int(request['storage_mib'])*1048576,MAX_FILE,shutil.disk_usage(ROOT).free-134217728))
     owner(Path('/proc/self/fd')/str(directory)/stage)
     os.sync() # Include every newly extracted subdirectory before the prepare receipt.
    else:
     fd=os.open(stage,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600,dir_fd=directory)
     try:
      with os.fdopen(fd,'wb') as out:
       if action=='install':
        source=CONTROL/'incoming'/str(uuid.UUID(request['artifact_id']))
        with incoming(source) as stream:
         regular(stream.fileno(),MAX_FILE);h=hashlib.sha256();total=0
         while True:
          chunk=stream.read(65536)
          if not chunk:break
          total+=len(chunk)
          if total>MAX_FILE:raise ValueError('Artifact exceeds 1 GiB')
          out.write(chunk);h.update(chunk)
         if h.hexdigest()!=request['sha256']:raise ValueError('Uploaded artifact checksum mismatch')
       else:
        data=request['text'].encode('utf-8')
        if len(data)>MAX_TEXT or b'\0' in data:raise ValueError('UTF-8 text is limited to 64 KiB without NUL')
        out.write(data)
       out.flush();os.fsync(out.fileno())
      owner(Path('/proc/self/fd')/str(directory)/stage)
     except BaseException:raise
    after=snapshot(root,staged,tree or action=='directory_create')
   else:after=None
   os.fsync(directory)
   prior={'phase':'prepared','digest':digest,'parent':identity,'before':before,'after':after,**(receipt_extra or {})}
   atomic(receipt,prior)
  expected=prior['after'];before=prior['before']
  actual=snapshot(root,path,tree or action=='directory_create')
  if prior['phase']!='committed':
   if actual!=expected:
    if actual!=before and not (tree and actual is None and snapshot(root,displaced,True)==before):raise ValueError('Destination changed after prepare; reconciliation required')
    server_stopped() # Recheck immediately before the namespace effect.
    if action=='file_delete':os.unlink(name,dir_fd=directory)
    else:
     if snapshot(root,staged,tree or action=='directory_create')!=expected:raise ValueError('Prepared bytes changed; refusing to commit')
     if tree and actual is not None:rename_new(directory,name,backup);os.fsync(directory)
     if before is None or tree or action=='directory_create':rename_new(directory,stage,name)
     else:os.replace(stage,name,src_dir_fd=directory,dst_dir_fd=directory)
    os.fsync(directory)
   if snapshot(root,path,tree or action=='directory_create')!=expected:raise ValueError('Effect verification failed')
   result={'path':path,'effect':'committed'}
   if expected is not None:result['sha256']=request['sha256'] if tree else expected['sha256']
   prior={**prior,'phase':'committed','result':result};atomic(receipt,prior)
  elif actual!=expected:raise ValueError('The committed destination changed; refusing to report an old success')
  if tree and snapshot(root,displaced,True) is not None:
   if not prior.get('cleanup_started'):
    if snapshot(root,displaced,True)!=before:raise ValueError('Displaced world changed; preserve it for reconciliation')
    prior={**prior,'cleanup_started':True};atomic(receipt,prior)
   remove_tree(root,displaced)
  return prior['result']

@contextlib.contextmanager
def incoming(source):
 fd=os.open(source,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)
 with os.fdopen(fd,'rb') as stream:
  regular(fd,MAX_FILE)
  yield stream

def install(request):return mutation(request,'install')

def verify_managed_auth(properties,paper):
 # Interpret only a deliberately small, unambiguous subset of Java properties
 # and YAML mappings. Complex/aliased auth configuration requires reconciliation.
 props={}
 for line in properties.splitlines():
  line=line.strip()
  if not line or line.startswith(('#','!')):continue
  if line.endswith('\\'):raise ValueError('Continued authentication properties are not supported for OP verification')
  pair=re.split(r'\s*[:=]\s*|\s+',line,maxsplit=1);key=pair[0];value=pair[1] if len(pair)>1 else ''
  if '\\' in key:raise ValueError('Escaped authentication property keys are not supported')
  if key in ('online-mode','enable-rcon'):
   if key in props:raise ValueError('Duplicate authentication properties require reconciliation')
   props[key]=value
 if props!={'online-mode':'false','enable-rcon':'false'}:raise ValueError('Managed authentication properties changed; reconcile them before OP')
 stack=[];seen=set();values={}
 for line in paper.splitlines():
  if not line.strip() or line.lstrip().startswith('#'):continue
  if '\t' in line:raise ValueError('Ambiguous proxy YAML indentation')
  match=re.fullmatch(r'( *)([A-Za-z0-9_-]+):(?: +(.*))?',line)
  if not match:
   if line.lstrip().startswith('<<:'):raise ValueError('Inherited proxy YAML is not supported')
   if not line.startswith(' ') or (stack and stack[0][1]=='proxies'):raise ValueError('Complex proxy YAML requires reconciliation')
   continue
  indent=len(match[1]);key=match[2];value=(match[3] or '').strip()
  while stack and stack[-1][0]>=indent:stack.pop()
  path=tuple(v[1] for v in stack)+(key,)
  if path[0]=='proxies':
   if path in seen:raise ValueError('Duplicate proxy YAML keys require reconciliation')
   seen.add(path)
   if path in (('proxies',),('proxies','velocity')) and value and not value.startswith('#'):raise ValueError('Proxy YAML must use explicit mappings')
   if len(path)==3 and path[:2]==('proxies','velocity'):
    scalar=value.split(' #',1)[0].strip()
    if key=='secret' and len(scalar)>=2 and scalar[0]==scalar[-1] and scalar[0] in ('"',"'"):scalar=scalar[1:-1]
    values[key]=scalar
  stack.append((indent,key))
 if values.get('enabled')!='true' or values.get('online-mode')!='true' or not re.fullmatch('[0-9a-f]{32,}',values.get('secret','')):raise ValueError('Proxy UUID ownership is not proven by the managed configuration')

def native_operator(request):
 server_stopped()
 if not isinstance(request.get('operator'),bool):raise ValueError('Operator must be a boolean')
 identity=request['identity'];native=str(uuid.UUID(identity['uuid']));name=identity['name']
 if not re.fullmatch('[A-Za-z0-9_]{3,16}',name):raise ValueError('Invalid verified Java name')
 if json.loads(CONFIG.read_text())['software']!='paper':raise ValueError('Only custom Paper native OP is supported')
 with root_fd() as root:
  # Protected properties and the current explicit Velocity mapping must prove UUID forwarding.
  properties=read_bytes(root,'server.properties',MAX_TEXT).decode()
  paper=read_bytes(root,'config/paper-global.yml',MAX_TEXT).decode()
  verify_managed_auth(properties,paper)
  receipt=receipt_path(request['job_id'])
  # Freeze the exact original content in the first receipt. Recovery must use the
  # original precondition/text even after ops.json has already been renamed.
  if receipt.exists():
   prior=json.loads(receipt.read_text());saved=prior.get('operator_request')
   if saved is None or saved['intent']!=request:raise ValueError('OP intent changed during recovery')
   write=saved['write']
  else:
   try:data=read_bytes(root,'ops.json',MAX_TEXT);ops=json.loads(data);expected=hashlib.sha256(data).hexdigest()
   except FileNotFoundError:ops=[];expected=None
   if not isinstance(ops,list) or len(ops)>1000:raise ValueError('Invalid or oversized ops.json')
   for entry in ops:
    if not isinstance(entry,dict) or not isinstance(entry.get('name'),str):raise ValueError('Invalid ops.json entry')
    if entry['name'].lower()==name.lower() and str(uuid.UUID(entry['uuid']))!=native:raise ValueError('Minecraft name belongs to another UUID in ops.json; reconcile identity first')
   ops=[e for e in ops if str(uuid.UUID(e['uuid']))!=native]
   if request['operator']:ops.append({'uuid':native,'name':name,'level':4,'bypassesPlayerLimit':False})
   write={'job_id':request['job_id'],'path':'ops.json','text':json.dumps(ops,ensure_ascii=False,indent=2)+'\n','expected_sha256':expected}
 result=mutation(write,'file_write',internal=True,receipt_extra={'operator_request':{'intent':request,'write':write}})
 with root_fd() as root:
  applied=json.loads(read_bytes(root,'ops.json',MAX_TEXT))
  found=[e for e in applied if str(uuid.UUID(e['uuid']))==native]
  if bool(found)!=request['operator'] or (found and (len(found)!=1 or found[0]['name']!=name or found[0]['level']!=4)):raise ValueError('Native OP effect was not verified')
 return {**result,'member':request['member'],'operator':request['operator'],'native_uuid':native,'effective':'next_start','message':'Minecraft OP is saved and becomes effective at the next game start.'}

def log_date(value):
 if not isinstance(value,str) or not re.fullmatch(r'\d{4}-\d{2}-\d{2}',value):raise ValueError('Choose a UTC date in YYYY-MM-DD format')
 datetime.date.fromisoformat(value);return value

def logs(request):
 date=request.get('date')
 if date is not None:log_date(date)
 dates=set();selected=[];truncated=False;scanned=0
 if any(zone not in ('UTC','GMT') for zone in time.tzname):raise ValueError('Guest log timezone is not proven UTC; historical date selection is unavailable')
 with root_fd() as root:
  try:
   with opened(root,'logs',os.O_RDONLY|os.O_DIRECTORY) as directory,os.scandir(directory) as stream:
    for entry in stream:
     scanned+=1
     if scanned>MAX_SCAN:truncated=True;break
     match=re.fullmatch(r'(\d{4}-\d{2}-\d{2})-(\d+)\.log(?:\.gz)?',entry.name)
     if entry.name!='latest.log' and not match:continue
     try:
      with opened(root,'logs/'+entry.name) as fd:info=regular(fd,MAX_FILE)
      day=log_date(match[1]) if match else datetime.datetime.fromtimestamp(info.st_mtime,datetime.timezone.utc).date().isoformat()
      dates.add(day)
      if date==day or (date is None and entry.name=='latest.log'):selected.append((entry.name,day))
     except (OSError,ValueError):continue
  except FileNotFoundError:raise ValueError('No Minecraft logs exist yet; the game has not produced logs')
  if not selected:return {'lines':[],'date':date,'dates':sorted(dates,reverse=True)[:366],'timezone':'UTC','available':False,'message':'No log is available for this date' if date else 'No latest log is available; choose an archived date','truncated':truncated or len(dates)>366}
  output=b'';expanded=0;compressed=0
  for name,day in sorted(selected,key=lambda pair:(pair[0]=='latest.log',int(re.search(r'-(\d+)\.log',pair[0])[1]) if pair[0]!='latest.log' else 0))[:128]:
   with opened(root,'logs/'+name) as fd:
    regular(fd,MAX_FILE)
    with os.fdopen(os.dup(fd),'rb') as raw:
     # Seek to a bounded tail for latest/uncompressed data; gzip is streamed with
     # a strict aggregate expansion budget. No date ever becomes a shell argument.
     if name.endswith('.gz'):
      size=os.fstat(fd).st_size
      if compressed+size>MAX_LOG_EXPANDED:truncated=True;break
      compressed+=size
      stream=gzip.GzipFile(fileobj=raw)
      with stream:
       while expanded<MAX_LOG_EXPANDED:
        chunk=stream.read(min(65536,MAX_LOG_EXPANDED-expanded))
        if not chunk:break
        expanded+=len(chunk);output=(output+chunk)[-MAX_LOG_BYTES:]
       if expanded>=MAX_LOG_EXPANDED:truncated=True;break
     else:
      size=os.fstat(fd).st_size
      start=max(0,size-min(MAX_LOG_BYTES,MAX_LOG_EXPANDED-expanded));raw.seek(start)
      chunk=raw.read(min(MAX_LOG_BYTES,MAX_LOG_EXPANDED-expanded));expanded+=len(chunk);output=(output+chunk)[-MAX_LOG_BYTES:]
      truncated|=start>0
  lines=output.decode('utf-8',errors='replace').splitlines()
  truncated|=len(lines)>200 or len(output)>=MAX_LOG_BYTES or len(selected)>128 or any(len(line)>1024 for line in lines)
  # 200*1024 characters bounds JSON even with escaped control characters.
  result={'lines':[line[:1024] for line in lines[-200:]],'date':date,'dates':sorted(dates,reverse=True)[:366],'timezone':'UTC','truncated':truncated or len(dates)>366}
  while len(json.dumps(result,ensure_ascii=False).encode())>MAX_LOG_BYTES:
   result['lines'].pop(0);result['truncated']=True
  return result

def console(request):
 job=str(uuid.UUID(request['job_id']));receipt=receipt_path(job);line=request['line'];data=(line+'\n').encode()
 if not line or len(data)>4096 or any(c in line for c in '\r\n\0'):raise ValueError('Invalid console line')
 digest=hashlib.sha256(data).hexdigest()
 if receipt.exists():
  prior=json.loads(receipt.read_text())
  if prior['digest']!=digest:raise ValueError('Console request changed')
  if prior['phase']=='committed':return prior['result']
  if prior.get('job_id')!=job or prior.get('authorized_at')!=request['authorized_at']:raise ValueError('Console authorization receipt changed; reconcile without resending')
  return {'effect':'uncertain','prepared_receipt':True,'job_id':job,'command_sha256':digest,'authorized_at':prior['authorized_at'],'prepared_lease_token':prior['lease_token']}
 lease=str(uuid.UUID(request['lease_token']));authorized=request['authorized_at']
 if not isinstance(authorized,str) or not authorized:raise ValueError('Console authorization is missing')
 fd=os.open(FIFO,os.O_WRONLY|os.O_NONBLOCK|os.O_NOFOLLOW)
 try:
  if not stat.S_ISFIFO(os.fstat(fd).st_mode):raise ValueError('Console endpoint is not a pipe')
  atomic(receipt,{'phase':'prepared','digest':digest,'job_id':job,'authorized_at':authorized,'lease_token':lease})
  if os.write(fd,data)!=len(data):raise ValueError('Console delivery was partial')
 finally:os.close(fd)
 result={'effect':'committed','delivery':'sent','job_id':job,'command_sha256':digest,'authorized_at':authorized,'prepared_lease_token':lease}
 atomic(receipt,{'phase':'committed','digest':digest,'result':result});return result

def inspection(request,opening):
 server_stopped()
 marker=CONTROL/'inspection.json';session=str(uuid.UUID(request['id']))
 if marker.exists() and json.loads(marker.read_text())['id']!=session:raise ValueError('Inspection session changed; reconcile before shutdown')
 if opening:
  if systemctl('is-enabled','lkjmc-game',check=False).stdout.strip() not in ('disabled','static'):raise ValueError('Game autostart must be disabled')
  atomic(marker,{'id':session})
 else:
  marker.unlink(missing_ok=True);os.sync()
 return {'guest_ready':True,'game_stopped':True}

def run():
 config=json.loads(CONFIG.read_text());java=f'/opt/lkjmc/java/{config["java"]}/bin/java'
 if not (ROOT/'server.jar').is_file():raise ValueError('server.jar is not installed')
 if FIFO.exists():FIFO.unlink()
 os.mkfifo(FIFO,0o600);fd=os.open(FIFO,os.O_RDWR)
 child=subprocess.Popen([java,'-Duser.timezone=UTC',f'-Xms256M',f'-Xmx{config["heap_mib"]}M','-jar','server.jar','nogui'],cwd=ROOT,stdin=subprocess.PIPE,text=True,bufsize=1)
 lock=threading.Lock()
 def send(line):
  with lock:
   if child.poll() is None:child.stdin.write(line+'\n');child.stdin.flush()
 def reader():
  with os.fdopen(fd) as stream:
   for line in stream:
    try:send(line.rstrip('\n'))
    except (BrokenPipeError,OSError):break
 threading.Thread(target=reader,daemon=True).start()
 signal.signal(signal.SIGTERM,lambda *_:send('stop'))
 signal.signal(signal.SIGINT,lambda *_:send('stop'))
 code=child.wait();FIFO.unlink(missing_ok=True);return code

@contextlib.contextmanager
def guest_lock():
 CONTROL.mkdir(parents=True,exist_ok=True)
 fd=os.open(CONTROL/'operations.lock',os.O_RDWR|os.O_CREAT|os.O_NOFOLLOW|os.O_NONBLOCK,0o600)
 try:
  regular(fd,1024)
  try:fcntl.flock(fd,fcntl.LOCK_EX|fcntl.LOCK_NB)
  except BlockingIOError:raise ValueError('Another guest operation is still running; wait and retry')
  yield
 finally:os.close(fd)

def main():
 action=sys.argv[1]
 if action=='run':sys.exit(run())
 if os.geteuid()!=0 or Path('/etc/lkjmc-guest-image').read_text().strip()!='rebuild-v1':raise ValueError('Not an lkjmc guest control context')
 request=json.load(sys.stdin)
 if 'job_id' in request:os.environ['LKJMC_GUEST_JOB']=str(uuid.UUID(request['job_id']))
 with guest_lock():result=dispatch(action,request)
 print(json.dumps(result,ensure_ascii=False))
def dispatch(action,request):
 if action=='bootstrap':result=bootstrap(request)
 elif action=='install':result=install(request)
 elif action=='console':result=console(request)
 elif action=='files':result=files(request)
 elif action=='file_read':result=file_read(request)
 elif action in ('file_write','file_delete','directory_create'):result=mutation(request,action)
 elif action=='operator':result=native_operator(request)
 elif action=='start':
  if (CONTROL/'inspection.json').exists():raise ValueError('Close file inspection before starting Minecraft')
  systemctl('start','lkjmc-game');result={'starting':True}
 elif action=='inspection_release':
  server_stopped();(CONTROL/'inspection.json').unlink(missing_ok=True);os.sync();result={'released':True}
 elif action in ('inspection_open','inspection_close'):
  result=inspection(request,action=='inspection_open')
 elif action=='stop':
  systemctl('stop','lkjmc-game');os.sync()
  status=systemctl('show','lkjmc-game','--property=Result','--value').stdout.strip()
  if status not in ('success','exit-code'):raise ValueError('Game did not stop cleanly; inspect the logs')
  result={'stopped':True,'service_result':status}
 elif action=='inspection_ready':
  server_stopped()
  enabled=systemctl('is-enabled','lkjmc-game',check=False).stdout.strip()
  if enabled not in ('disabled','static'):raise ValueError('Game autostart must be disabled by a reviewed guest upgrade')
  result={'guest_ready':True,'game_stopped':True}
 elif action=='status':result={'active':systemctl('is-active','lkjmc-game',check=False).returncode==0}
 elif action=='logs':result=logs(request)
 else:raise ValueError('Unknown guest operation')
 return result

if __name__=='__main__':
 try:main()
 except Exception as error:
  # Only a rejected mutation before its durable prepare record can be declared effect-free.
  # Keep bootstrap, stop and arbitrary console interruptions for reconciliation.
  effect='uncertain'
  if sys.argv[1] in PASSIVE:effect='none'
  if sys.argv[1] in ('install','file_write','file_delete','directory_create','operator'):
   try:
    # The job UUID is recovered from an explicit environment set in main, never a path from the archive.
    job=os.environ.get('LKJMC_GUEST_JOB')
    if job and not receipt_path(job).exists():effect='none'
   except Exception:pass
  message=error.strerror if isinstance(error,OSError) and error.strerror else str(error)
  print(json.dumps({'error':message,'effect':effect},ensure_ascii=False))
  if sys.argv[1]=='run':sys.exit(1)
