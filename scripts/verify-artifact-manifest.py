#!/usr/bin/env python3
"""Verify manifest, sidecar, and independently derived release closure."""
import argparse,hashlib,json,re,stat,sys
from pathlib import Path
from release_inventory import commit,expected,fail

def regular(path,label):
 try: mode=path.lstat().st_mode
 except FileNotFoundError: fail(f'missing {label}')
 if not stat.S_ISREG(mode) or path.is_symlink(): fail(f'{label} is not a regular file')
def main():
 parser=argparse.ArgumentParser()
 parser.add_argument('--manifest',required=True)
 parser.add_argument('--release-root',required=True)
 args=parser.parse_args(); manifest=Path(args.manifest).resolve(); release=Path(args.release_root).resolve()
 if manifest.parent!=release or manifest.name!='artifact-manifest.json': fail('unsafe manifest location')
 sidecar=manifest.with_suffix(manifest.suffix+'.sha256')
 regular(manifest,'manifest'); regular(sidecar,'manifest sidecar')
 raw=manifest.read_bytes(); line=sidecar.read_text(encoding='ascii')
 match=re.fullmatch(r'([0-9a-f]{64})  artifact-manifest\.json\n',line)
 if not match or match.group(1)!=hashlib.sha256(raw).hexdigest(): fail('manifest checksum sidecar differs')
 data=json.loads(raw)
 if data!=expected(release,commit()): fail('manifest differs from independently derived release closure')
 print(f'ok artifact-manifest-verified commit={data["commit"]} artifacts={len(data["artifacts"])} contracts={len(data["contracts"])}')
if __name__=='__main__':
 try: main()
 except Exception as error:
  print(f'artifact manifest verification failed: {error}',file=sys.stderr); sys.exit(1)
