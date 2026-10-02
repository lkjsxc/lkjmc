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

def checked_path(value):
 if not isinstance(value,str) or not value or len(value)>240 or '\\' in value or any(ord(c)<32 for c in value):raise ValueError('Invalid target path')
 path=PurePosixPath(value)
 if path.is_absolute() or any(p in ('','.','..') or p.startswith('.') for p in value.split('/')):raise ValueError('Invalid target path')
 target=ROOT.joinpath(*path.parts)
 if not target.resolve().is_relative_to(ROOT.resolve()) or target.resolve()==ROOT.resolve():raise ValueError('Target leaves the server directory')
 if path.parts[0].startswith('lkjmc-') or value=='eula.txt':raise ValueError('System-managed file')
 return target

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
 if systemctl('is-active','lkjmc-game',check=False).returncode==0:raise ValueError('Stop the game before changing files')

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
''')
 systemctl('daemon-reload');os.sync()
 return {'configured':True,'server_id':request['server_id']}

def install(request):
 server_stopped();job=str(uuid.UUID(request['job_id']));receipt=receipt_path(job)
 digest=hashlib.sha256(json.dumps(request,sort_keys=True).encode()).hexdigest()
 prior=json.loads(receipt.read_text()) if receipt.exists() else None
 if prior:
  if prior['digest']!=digest:raise ValueError('Install request changed during recovery')
  if prior['phase']=='committed':
   target=checked_path(request['path']);discard(target.with_name('.lkjmc-before-'+job))
   return prior['result']
 source=CONTROL/'incoming'/str(uuid.UUID(request['artifact_id']))
 with open(source,'rb') as f:actual=hashlib.file_digest(f,'sha256').hexdigest()
 if actual!=request['sha256']:raise ValueError('Uploaded artifact checksum mismatch')
 target=checked_path(request['path']);target.parent.mkdir(parents=True,exist_ok=True)
 staged=target.with_name('.lkjmc-stage-'+job);backup=target.with_name('.lkjmc-before-'+job)
 if not prior:
  if staged.exists():
   if staged.is_dir():shutil.rmtree(staged)
   else:staged.unlink()
  if request['kind']=='world':
   staged.mkdir();extract_world(source,staged,min(int(request['storage_mib'])*1048576,shutil.disk_usage(ROOT).free-134217728))
  else:
   shutil.copyfile(source,staged)
   with open(staged,'rb') as f:os.fsync(f.fileno())
  owner(staged);os.sync()
  prior={'digest':digest,'phase':'prepared'};atomic(receipt,prior)
 if staged.exists():
  if target.exists() and not backup.exists():os.rename(target,backup);os.sync()
  if target.exists():raise ValueError('Install destination changed during recovery')
  os.rename(staged,target);os.sync()
 elif not target.exists():raise ValueError('Prepared install has no destination or staged data')
 result={'effect':'committed','sha256':actual,'path':request['path']}
 atomic(receipt,{'digest':digest,'phase':'committed','result':result})
 # Retain the displaced tree until its replacement and receipt are durable.
 if backup.exists():
  discard(backup)
 return result

def console(request):
 job=str(uuid.UUID(request['job_id']));receipt=receipt_path(job);line=request['line'];data=(line+'\n').encode()
 if not line or len(data)>4096 or any(c in line for c in '\r\n\0'):raise ValueError('Invalid console line')
 digest=hashlib.sha256(data).hexdigest()
 if receipt.exists():
  prior=json.loads(receipt.read_text())
  if prior['digest']!=digest:raise ValueError('Console request changed')
  if prior['phase']=='committed':return prior['result']
  return {'effect':'uncertain','message':'Console delivery may have occurred; do not automatically resend this line.'}
 fd=os.open(FIFO,os.O_WRONLY|os.O_NONBLOCK|os.O_NOFOLLOW)
 try:
  if not stat.S_ISFIFO(os.fstat(fd).st_mode):raise ValueError('Console endpoint is not a pipe')
  atomic(receipt,{'phase':'prepared','digest':digest})
  if os.write(fd,data)!=len(data):raise ValueError('Console delivery was partial')
 finally:os.close(fd)
 result={'effect':'committed','delivery':'sent','message':'コンソールへ送信しました。コマンドの実行結果はログで確認してください。'}
 atomic(receipt,{'phase':'committed','digest':digest,'result':result});return result

def run():
 config=json.loads(CONFIG.read_text());java=f'/opt/lkjmc/java/{config["java"]}/bin/java'
 if not (ROOT/'server.jar').is_file():raise ValueError('server.jar is not installed')
 if FIFO.exists():FIFO.unlink()
 os.mkfifo(FIFO,0o600);fd=os.open(FIFO,os.O_RDWR)
 child=subprocess.Popen([java,f'-Xms256M',f'-Xmx{config["heap_mib"]}M','-jar','server.jar','nogui'],cwd=ROOT,stdin=subprocess.PIPE,text=True,bufsize=1)
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

def main():
 action=sys.argv[1]
 if action=='run':sys.exit(run())
 if os.geteuid()!=0 or Path('/etc/lkjmc-guest-image').read_text().strip()!='rebuild-v1':raise ValueError('Not an lkjmc guest control context')
 request=json.load(sys.stdin)
 if 'job_id' in request:os.environ['LKJMC_GUEST_JOB']=str(uuid.UUID(request['job_id']))
 if action=='bootstrap':result=bootstrap(request)
 elif action=='install':result=install(request)
 elif action=='console':result=console(request)
 elif action=='start':systemctl('start','lkjmc-game');result={'starting':True}
 elif action=='stop':
  systemctl('stop','lkjmc-game');os.sync()
  status=systemctl('show','lkjmc-game','--property=Result','--value').stdout.strip()
  if status not in ('success','exit-code'):raise ValueError('Game did not stop cleanly; inspect the logs')
  result={'stopped':True,'service_result':status}
 elif action=='status':result={'active':systemctl('is-active','lkjmc-game',check=False).returncode==0}
 elif action=='logs':
  result={'lines':[line[:4096] for line in subprocess.run(['journalctl','-u','lkjmc-game','-n','200','--output=cat','--no-pager'],capture_output=True,text=True,check=True).stdout.splitlines()[-200:]]}
 else:raise ValueError('Unknown guest operation')
 print(json.dumps(result,ensure_ascii=False))
if __name__=='__main__':
 try:main()
 except Exception as error:
  # Only a rejected install before its durable prepare record can be declared effect-free.
  # Keep bootstrap, stop and arbitrary console interruptions for reconciliation.
  effect='uncertain'
  if sys.argv[1]=='install':
   try:
    # The job UUID is recovered from an explicit environment set in main, never a path from the archive.
    job=os.environ.get('LKJMC_GUEST_JOB')
    if job and not receipt_path(job).exists():effect='none'
   except Exception:pass
  print(json.dumps({'error':str(error),'effect':effect},ensure_ascii=False))
