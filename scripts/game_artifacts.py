#!/usr/bin/env python3
"""Fetch pinned, hash-checked compatibility candidates into the local-only test rig."""
import hashlib,json,urllib.request
from pathlib import Path

ROOT=Path(__file__).resolve().parent.parent
TARGET=ROOT/'.local'/'artifacts'
EXTRA={
 'viaversion':('https://cdn.modrinth.com/data/P1OZGk5p/versions/FaishMnD/ViaVersion-5.12.0.jar','sha512','2dfe562109179f08685dc84a66aedf7c810592223b5287b6355ba1e741d1737d015ca0c444b4bc01e5db2a4b10549d5028bf2e3f3337219ded709e44b579a4d8'),
 'viabackwards':('https://cdn.modrinth.com/data/NpvuJQoq/versions/SxGhdsPK/ViaBackwards-5.12.0.jar','sha512','dba076b3283eb5987e3d37a63b57802e7946cf8a2d3b6f68ff57ddbc4eb648e2084decd212fef2cd0cde5187bc6565959c31b496bfb38000e67e3ffab4c290b3'),
 'worldedit':('https://cdn.modrinth.com/data/1u6JkXh5/versions/F5ea2ov3/worldedit-bukkit-7.4.5.jar','sha512','a383492fac6bfb4d43a257dfa7b5fc076aae503a71151b463de4fe80e6f3d5fc11209eaf4097baa115f3febf0adc40ca0a1ecda227b8439b429d0a4ba3a63a4f'),
 'worldguard':('https://cdn.modrinth.com/data/DKY9btbd/versions/TtfwTyi6/worldguard-bukkit-7.0.19.jar','sha512','e9ad7ad53c93a07a7d5c3af3d844a677abbe894de4d7ff69fc03397daae5e8532217d57de23a3cddbe0d94ae81ee7acde6c78384b9596aa1eb3001bb37087d41')
}
def main():
 TARGET.mkdir(parents=True,exist_ok=True)
 versions=json.loads((ROOT/'ops/component-candidates.json').read_text())
 artifacts={key:(versions[key]['url'],'sha256',versions[key]['sha256']) for key in ['paper','velocity']}
 artifacts.update(EXTRA)
 for name,(url,algorithm,expected) in artifacts.items():
  file=TARGET/(name+'.jar')
  if file.exists() and hashlib.new(algorithm,file.read_bytes()).hexdigest()==expected:continue
  request=urllib.request.Request(url,headers={'User-Agent':'lkjmc/0.1 (https://lkjmc.lkjsxc.com)'})
  with urllib.request.urlopen(request,timeout=90) as response:data=response.read()
  if hashlib.new(algorithm,data).hexdigest()!=expected:raise RuntimeError('Artifact checksum mismatch: '+name)
  temporary=file.with_suffix('.part');temporary.write_bytes(data);temporary.replace(file)
  print(name+': verified '+str(len(data))+' bytes')
if __name__=='__main__':main()
