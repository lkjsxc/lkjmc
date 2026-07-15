#!/usr/bin/env python3
"""Convert one retained Compose verifier result to bounded JSON evidence."""
import argparse,json,os,re,sys
from pathlib import Path
from fd_tree import Limits,visit_file
p=argparse.ArgumentParser(); p.add_argument('--log',required=True); p.add_argument('--exit',type=int,required=True); p.add_argument('--build-exit',type=int,required=True); p.add_argument('--output',required=True); p.add_argument('--commit',required=True); a=p.parse_args()
values=[]
def read(fd,item):
 os.lseek(fd,max(0,item.size-131072),os.SEEK_SET); values.append(os.read(fd,min(item.size,131072)).decode(errors='replace'))
visit_file(Path(a.log),read,Limits(max_bytes=2*1024**3,max_file_bytes=2*1024**3))
text=re.sub(r'(?i)[a-z][a-z0-9+.-]*://[^\s"\']+','<redacted-url>',values[0])
match=re.search(r'ok verify-full ran=(\S+) skipped=(\S+)',text)
ran=[] if not match or match.group(1)=='none' else match.group(1).split(',')
skipped=[] if not match or match.group(2)=='none' else match.group(2).split(',')
status='pass' if a.build_exit==0 and a.exit==0 and match else 'fail'
data={"schemaVersion":1,"commit":a.commit,"lane":{"probe":"ci-compose-retained","status":status,
 "commands":[{"argv":["docker","compose","--profile","verify","build","--no-cache","verify"],"exit":a.build_exit},{"argv":["docker","compose","--profile","verify","run","--rm","verify"],"exit":a.exit}],
 "ran":ran,"skips":skipped}}
out=Path(a.output); raw=(json.dumps(data,indent=2,sort_keys=True)+'\n').encode()
fd=os.open(out,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_CLOEXEC|os.O_NOFOLLOW,0o600)
try:
 view=memoryview(raw)
 while view: view=view[os.write(fd,view):]
 os.fsync(fd)
finally: os.close(fd)
print(f'ok ci-compose-evidence status={status} ran={len(ran)} skipped={len(skipped)}')
sys.exit(status!='pass')
