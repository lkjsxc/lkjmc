#!/usr/bin/env python3
"""Create a private commit-tied release and component inventory."""
import argparse, hashlib, json, os, re, stat, subprocess, sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
SECRET_NAME = re.compile(r'(^|[._-])(secret|token|password|cookie|credential)([._-]|$)', re.I)

def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def git(*args): return subprocess.check_output(('git',)+args,cwd=ROOT,text=True).strip()
def fail(message): raise RuntimeError(message)
def item(path, kind, source, provenance):
    mode=path.stat().st_mode
    if not stat.S_ISREG(mode) or path.is_symlink(): fail(f'not a regular artifact: {path}')
    data=path.read_bytes()
    if SECRET_NAME.search(path.name): fail(f'secret-shaped artifact: {path}')
    canary=os.environ.get('LKJMC_SECRET_CANARY','').encode()
    if canary and canary in data: fail(f'credential canary in artifact: {path}')
    return {"component":path.stem,"kind":kind,"path":str(path),"provenance":provenance,
            "sha256":hashlib.sha256(data).hexdigest(),"size":len(data),"source":source}
def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--output',required=True); ap.add_argument('artifacts',nargs='+')
    args=ap.parse_args(); out=Path(args.output)
    supplied=os.environ.get('LKJMC_SOURCE_COMMIT')
    if (ROOT/'.git').exists():
        if subprocess.run(('git','diff','--quiet','HEAD','--'),cwd=ROOT).returncode: fail('tracked worktree is dirty')
        commit=git('rev-parse','HEAD'); tracked=set(git('ls-files').splitlines())
        if supplied and supplied != commit: fail('supplied commit differs from checkout')
    else:
        if not supplied or not re.fullmatch(r'[0-9a-f]{40}',supplied): fail('gitless export requires LKJMC_SOURCE_COMMIT')
        commit=supplied
        tracked={'Cargo.lock','rust-toolchain.toml','gradle/wrapper/gradle-wrapper.properties'}
        tracked.update(str(p.relative_to(ROOT)) for root in ('config','contracts') for p in (ROOT/root).rglob('*') if p.is_file())
    artifacts=[]; seen=set()
    for value in args.artifacts:
        path=Path(value)
        if not path.is_absolute(): path=(ROOT/path).resolve()
        if path in seen: fail(f'duplicate artifact: {value}')
        seen.add(path); kind='jar' if path.suffix=='.jar' else 'binary'
        artifacts.append(item(path,kind,'committed source',f'pinned build at {commit}'))
    if not any(x['kind']=='binary' for x in artifacts) or not any(x['kind']=='jar' for x in artifacts):
        fail('release inventory requires binaries and jars')
    contracts=[]
    for rel in sorted(tracked):
        if rel.startswith(('config/','contracts/')) or rel in ('Cargo.lock','rust-toolchain.toml','gradle/wrapper/gradle-wrapper.properties'):
            path=ROOT/rel
            if path.is_file(): contracts.append(item(path,'config-contract',rel,'git object '+commit))
    images=[]
    for rel in ('Dockerfile','docker-compose.yml'):
        text=(ROOT/rel).read_text()
        for name,digest in re.findall(r'([\w./:-]+)@sha256:([0-9a-f]{64})',text):
            images.append({"component":name,"digest":"sha256:"+digest,"source":rel})
    if len(images)<3: fail('expected pinned Rust, Gradle, and PostgreSQL images')
    metadata=json.loads(subprocess.check_output(
        ('cargo','metadata','--locked','--format-version=1'),cwd=ROOT,text=True))
    components=[{"ecosystem":"cargo","name":p['name'],"version":p['version'],
                 "source":p.get('source') or 'workspace'} for p in metadata['packages']]
    gradle=(ROOT/'gradle/wrapper/gradle-wrapper.properties').read_text()
    m=re.search(r'gradle-([0-9.]+)-bin.zip',gradle)
    components.append({"ecosystem":"gradle","name":"gradle","version":m.group(1),"source":"verified distribution"})
    data={"schemaVersion":1,"commit":commit,"artifacts":sorted(artifacts,key=lambda x:x['path']),
          "components":sorted(components,key=lambda x:(x['ecosystem'],x['name'],x['version'])),
          "contracts":contracts,"images":sorted(images,key=lambda x:(x['component'],x['source']))}
    payload=(json.dumps(data,indent=2,sort_keys=True)+'\n').encode()
    out.parent.mkdir(parents=True,exist_ok=True); old=os.umask(0o077)
    try:
        if out.exists(): fail(f'refusing existing manifest: {out}')
        out.write_bytes(payload); out.with_suffix(out.suffix+'.sha256').write_text(hashlib.sha256(payload).hexdigest()+'  '+out.name+'\n')
    finally: os.umask(old)
    print(f'ok artifact-manifest commit={commit} artifacts={len(artifacts)} components={len(components)}')
if __name__=='__main__':
    try: main()
    except (OSError,RuntimeError,subprocess.CalledProcessError,KeyError,ValueError) as e:
        print(f'artifact manifest failed: {e}',file=sys.stderr); sys.exit(1)
