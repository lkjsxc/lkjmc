#!/usr/bin/env python3
"""Convert one retained Compose verifier result to bounded JSON evidence."""
import argparse,json,re,sys
from pathlib import Path
p=argparse.ArgumentParser(); p.add_argument('--log',required=True); p.add_argument('--exit',type=int,required=True); p.add_argument('--build-exit',type=int,required=True); p.add_argument('--output',required=True); p.add_argument('--commit',required=True); a=p.parse_args()
text=Path(a.log).read_text(errors='replace')[-131072:]
text=re.sub(r'(?i)[a-z][a-z0-9+.-]*://[^\s"\']+','<redacted-url>',text)
match=re.search(r'ok verify-full ran=(\S+) skipped=(\S+)',text)
ran=[] if not match or match.group(1)=='none' else match.group(1).split(',')
skipped=[] if not match or match.group(2)=='none' else match.group(2).split(',')
status='pass' if a.build_exit==0 and a.exit==0 and match else 'fail'
data={"schemaVersion":1,"commit":a.commit,"lane":{"probe":"ci-compose-retained","status":status,
 "commands":[{"argv":["docker","compose","--profile","verify","build","--no-cache","verify"],"exit":a.build_exit},{"argv":["docker","compose","--profile","verify","run","--rm","verify"],"exit":a.exit}],
 "ran":ran,"skips":skipped}}
out=Path(a.output); out.parent.mkdir(parents=True,exist_ok=True); out.write_text(json.dumps(data,indent=2,sort_keys=True)+'\n')
print(f'ok ci-compose-evidence status={status} ran={len(ran)} skipped={len(skipped)}')
sys.exit(status!='pass')
