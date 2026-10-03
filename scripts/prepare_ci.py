#!/usr/bin/env python3
"""Assemble a CI image context from explicit public inputs and tracked source.

Does not copy development databases, local accounts, home caches, management
files, or any credentials. The resulting directory is reviewed before building.
"""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

root = Path(__file__).resolve().parent.parent
inputs = root / '.local/ci-input'
context = root / '.local/ci-context'
assert not context.exists()
assert not subprocess.check_output(['git','status','--porcelain'],cwd=root)
commit = subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()
context.mkdir(mode=0o700)

def copy(source,target):
    assert source.is_file() and not source.is_symlink()
    target.parent.mkdir(parents=True,exist_ok=True)
    shutil.copyfile(source,target)
    target.chmod(source.stat().st_mode & 0o777)

for name in ['Dockerfile','toolchains.lock.json','install-toolchains.py','check.py','checkout.sh',
             'gitleaks.toml','empty.gitleaksignore','secret-policy.json','secret_policy.py','ux-inputs.lock.json']:
    copy(root/'ops/ci'/name,context/name)
tools=json.loads((context/'toolchains.lock.json').read_text())['tools']
for name,pin in tools.items():
    source=inputs/'toolchains'/name
    assert hashlib.file_digest(source.open('rb'),'sha256').hexdigest()==pin['sha256']
    copy(source,context/'toolchains'/name)
copy(inputs/'gitleaks',context/'gitleaks')
assert hashlib.file_digest((context/'gitleaks').open('rb'),'sha256').hexdigest()=='88f91962aa2f93ac6ab281d553b9e125f5197bbbce38f9f2437f7299c32e5509'
for path in (inputs/'vendor').rglob('*'):
    assert path.is_file() or path.is_dir()
    assert not path.is_symlink()
shutil.copytree(inputs/'vendor',context/'vendor')
files=subprocess.check_output(['git','ls-files','-z'],cwd=root).split(b'\0')
for raw in files:
    if not raw:continue
    name=raw.decode()
    if name in ['web/package.json','web/package-lock.json','tests/game/package.json','tests/game/package-lock.json'] or name.startswith('plugins/') and (
            name.endswith(('/build.gradle.kts','/settings.gradle.kts','/gradle.lockfile','/dependency-sha256.json'))):
        copy(root/name,context/'dependencies'/name)
ux=json.loads((context/'ux-inputs.lock.json').read_text())
assert ux['schema']==1 and ux['playwright']=='1.63.0' and ux['chromium_headless_revision']=='1243'
sources={'browser':Path.home()/'.cache/ms-playwright/chromium_headless_shell-1243',
         'paper-runtime':root/'.local/game/official'}
for name,pin in ux['files'].items():
    group,relative=name.split('/',1)
    assert group in sources and '..' not in Path(relative).parts and not Path(relative).is_absolute()
    source=sources[group]/relative
    assert hashlib.file_digest(source.open('rb'),'sha256').hexdigest()==pin['sha256']
    assert source.stat().st_mode&0o777==pin['mode']
    copy(source,context/'ux-public'/name)
subprocess.run(['python3','scripts/game_artifacts.py','--offline'],cwd=root,check=True)
for name in ['paper','velocity','worldedit','worldguard','viaversion','viabackwards','geyser','floodgate']:
    copy(root/'.local/artifacts'/(name+'.jar'),context/'game-artifacts'/(name+'.jar'))
upload=inputs/'upload-artifact'
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=upload,text=True).strip()=='c6a3b2bd78b3985e4b2f15397fec357f0fd808de'
assert not subprocess.check_output(['git','status','--porcelain'],cwd=upload)
for raw in subprocess.check_output(['git','ls-files','-z','dist','action.yml','LICENSE'],cwd=upload).split(b'\0'):
    if raw:copy(upload/raw.decode(),context/'upload-artifact'/raw.decode())
manifest={path.relative_to(context).as_posix():hashlib.file_digest(path.open('rb'),'sha256').hexdigest()
          for path in sorted(context.rglob('*')) if path.is_file()}
receipt={'schema':1,'source_commit':commit,'files':manifest}
(context/'context-manifest.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({'source_commit':commit,'files':len(manifest),'context_manifest_sha256':hashlib.file_digest((context/'context-manifest.json').open('rb'),'sha256').hexdigest()}))
