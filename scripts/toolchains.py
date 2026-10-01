#!/usr/bin/env python3
"""Fetch public build tools and Minecraft artifact metadata with checksums.

Artifacts stay outside Git. Server component pins are reviewed separately before deployment.
"""
import argparse
import hashlib
import json
from pathlib import Path
import tarfile
import urllib.request
import zipfile

ROOT=Path(__file__).resolve().parent.parent
CACHE=ROOT/'.local'/'toolchains'
UA='lkjmc/0.1 (https://lkjmc.lkjsxc.com)'

def fetch(url):
    with urllib.request.urlopen(urllib.request.Request(url,headers={'User-Agent':UA}),timeout=90) as response:
        return response.read()

def document(url): return json.loads(fetch(url))

def download(url,checksum,target):
    if target.exists() and hashlib.sha256(target.read_bytes()).hexdigest()==checksum: return
    data=fetch(url)
    if hashlib.sha256(data).hexdigest()!=checksum: raise ValueError('Artifact checksum mismatch')
    target.parent.mkdir(parents=True,exist_ok=True)
    target.write_bytes(data)

def java():
    assets=document('https://api.adoptium.net/v3/assets/latest/25/hotspot?architecture=x64&heap_size=normal&image_type=jdk&jvm_impl=hotspot&os=linux&vendor=eclipse')
    asset=assets[0];package=asset['binary']['package'];archive=CACHE/package['name']
    download(package['link'],package['checksum'],archive)
    with tarfile.open(archive) as tar:
        top=tar.getmembers()[0].name.split('/')[0]
        if not (CACHE/top/'bin/java').exists(): tar.extractall(CACHE,filter='data')
    print(json.dumps({'java_home':str(CACHE/top),'version':asset['version']['semver'],'sha256':package['checksum']}))

def gradle():
    asset=document('https://services.gradle.org/versions/current')
    checksum=fetch(asset['checksumUrl']).decode().strip()
    target=CACHE/f'gradle-{asset["version"]}-bin.zip'
    download(asset['downloadUrl'],checksum,target)
    with zipfile.ZipFile(target) as archive:
        for info in archive.infolist():
            path=Path(info.filename)
            if path.is_absolute() or '..' in path.parts: raise ValueError('Unsafe tool archive')
        if not (CACHE/f'gradle-{asset["version"]}'/'bin/gradle').exists():archive.extractall(CACHE)
    binary=CACHE/f'gradle-{asset["version"]}'/'bin/gradle';binary.chmod(0o755)
    print(json.dumps({'gradle':str(binary),'version':asset['version'],'sha256':checksum}))

def paper(project,version):
    base=f'https://fill.papermc.io/v3/projects/{project}'
    data=document(base)
    versions=[v for group in data['versions'].values() for v in group]
    if version:versions=[version]
    for v in versions[:10]:
        if project=='paper' and any(s in v for s in ['-pre','-rc']):continue
        builds=document(f'{base}/versions/{v}/builds')
        stable=[b for b in builds if b['channel']=='STABLE']
        latest=stable[0] if stable else (builds[0] if project=='velocity' else None)
        if latest:
            print(json.dumps({'project':project,'version':v,'build':latest['id'],'channel':latest['channel'],'download':latest['downloads']['server:default']},indent=2));return
    raise ValueError('No stable build found; do not substitute an experimental production build')

def main():
    parser=argparse.ArgumentParser();parser.add_argument('component',choices=['java','gradle','paper','velocity']);parser.add_argument('--version');args=parser.parse_args();CACHE.mkdir(parents=True,exist_ok=True)
    if args.component=='java':java()
    elif args.component=='gradle':gradle()
    else:paper(args.component,args.version)
if __name__=='__main__':main()
