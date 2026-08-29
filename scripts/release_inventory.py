#!/usr/bin/env python3
"""Pure release-closure derivation shared by generation and verification."""
import hashlib,json,os,re,stat,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
HEX=re.compile(r'[0-9a-f]{64}')
SECRET_NAME=re.compile(r'(^|[._-])(secret|token|password|cookie|credential)([._-]|$)',re.I)
def fail(message): raise RuntimeError(message)
def workspace_package_value(name):
 in_package=False
 for raw in (ROOT/'Cargo.toml').read_text().splitlines():
  line=raw.strip()
  if line.startswith('[') and line.endswith(']'): in_package=line=='[workspace.package]'
  elif in_package:
   match=re.fullmatch(rf'{re.escape(name)}\s*=\s*"([^"]+)"',line)
   if match: return match.group(1)
 fail(f'missing workspace.package {name}')
def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def git(*args): return subprocess.check_output(('git',)+args,cwd=ROOT,text=True).strip()
def commit():
 supplied=os.environ.get('LKJMC_SOURCE_COMMIT','')
 inside=subprocess.run(('git','rev-parse','--is-inside-work-tree'),cwd=ROOT,text=True,
  stdout=subprocess.PIPE,stderr=subprocess.DEVNULL).stdout.strip()=='true'
 if inside:
  if git('status','--porcelain=v1','--untracked-files=normal'): fail('worktree is dirty')
  value=git('rev-parse','HEAD')
  if supplied and supplied!=value: fail('supplied commit differs from checkout')
  return value
 fail('release provenance requires a clean Git checkout')
def release_contract():
 data=json.loads((ROOT/'config/release-artifacts.json').read_text())
 if set(data)!={'schemaVersion','artifacts'} or data['schemaVersion']!=1: fail('invalid release contract schema')
 expected=[]; destinations=set(); sources=set()
 for value in data['artifacts']:
  if set(value)!={'component','destination','kind','source'}: fail('invalid release artifact contract fields')
  destination=Path(value['destination']); source=Path(value['source'])
  if destination.name!=value['destination'] or source.is_absolute() or '..' in source.parts: fail('unsafe release contract path')
  if value['kind'] not in ('binary','jar','config') or ((destination.suffix=='.jar')!=(value['kind']=='jar')): fail('release kind differs from destination')
  if destination.name in destinations or str(source) in sources: fail('duplicate release contract path')
  destinations.add(destination.name); sources.add(str(source)); expected.append(value)
 return sorted(expected,key=lambda x:x['destination'])
def tracked_contract_paths():
 paths=[]
 for parent in ('config','contracts'):
  paths.extend(p.relative_to(ROOT) for p in (ROOT/parent).rglob('*') if p.is_file() and not p.is_symlink())
 fixed=['Cargo.lock','rust-toolchain.toml','gradle/wrapper/gradle-wrapper.properties','Dockerfile','docker-compose.yml','Cargo.toml','settings.gradle.kts','build.gradle.kts']
 paths.extend(Path(value) for value in fixed)
 paths.extend(p.relative_to(ROOT) for p in (ROOT/'crates').glob('*/Cargo.toml'))
 paths.extend(p.relative_to(ROOT) for p in (ROOT/'platforms/jvm').glob('*/build.gradle.kts'))
 return sorted(set(paths),key=str)
def regular(path,label):
 try: mode=path.lstat().st_mode
 except FileNotFoundError: fail(f'missing {label}: {path}')
 if not stat.S_ISREG(mode) or path.is_symlink(): fail(f'{label} is not regular: {path}')
def artifact_items(release_root,version):
 source=release_root/'source'; regular_files=[]
 if source.is_symlink() or not source.is_dir(): fail('release source directory missing')
 for path in source.iterdir():
  regular(path,'release artifact'); regular_files.append(path.name)
 expected=release_contract(); names={x['destination'] for x in expected}
 if set(regular_files)!=names or len(regular_files)!=len(names): fail('release artifact closure differs from contract')
 canary=os.environ.get('LKJMC_SECRET_CANARY','').encode()
 items=[]
 for value in expected:
  path=source/value['destination']; raw=path.read_bytes()
  if SECRET_NAME.search(path.name): fail(f'secret-shaped artifact: {path.name}')
  if canary and canary in raw: fail(f'credential canary in artifact: {path.name}')
  items.append({'component':value['component'],'kind':value['kind'],'path':value['destination'],
   'provenance':f'pinned build at {version}','sha256':hashlib.sha256(raw).hexdigest(),
   'size':len(raw),'source':value['source']})
 return items
def contract_items(version):
 items=[]
 for rel in tracked_contract_paths():
  path=ROOT/rel; regular(path,'contract')
  items.append({'path':str(rel),'sha256':sha(path),'size':path.stat().st_size,
   'provenance':'git object '+version})
 return items
def image_items():
 items=[]
 for rel in ('Dockerfile','docker-compose.yml'):
  for name,digest in re.findall(r'([\w./:-]+)@sha256:([0-9a-f]{64})',(ROOT/rel).read_text()):
   items.append({'component':name,'digest':'sha256:'+digest,'source':rel})
 if len(items)<3: fail('expected pinned Rust, Gradle, and PostgreSQL images')
 return sorted(items,key=lambda x:(x['source'],x['component'],x['digest']))
def cargo_lock_packages():
 raw=(ROOT/'Cargo.lock').read_text(encoding='utf-8')
 if len(raw)>4*1024*1024: fail('Cargo.lock exceeds component inventory limit')
 blocks=re.split(r'(?m)^\[\[package\]\]\s*$',raw)
 header=blocks.pop(0)
 header_lines=[line.strip() for line in header.splitlines()
  if line.strip() and not line.lstrip().startswith('#')]
 if header_lines!=['version = 4']: fail('unsupported Cargo.lock format version')
 values=[]; string=r'("(?:[^"\\]|\\.)*")'
 for block in blocks:
  if re.search(r'(?m)^\s*\[',block): fail('unsupported Cargo.lock table')
  keys=re.findall(r'(?m)^([A-Za-z][A-Za-z0-9_-]*)\s*=',block)
  allowed={'name','version','source','checksum','dependencies','replace'}
  if set(keys)-allowed or len(keys)!=len(set(keys)): fail('unsupported or duplicate Cargo.lock package field')
  fields={}
  for name in ('name','version','source'):
   matches=re.findall(rf'(?m)^{name}\s*=\s*{string}\s*$',block)
   if len(matches)>1 or (name!='source' and len(matches)!=1): fail(f'invalid Cargo.lock package {name}')
   if matches:
    try: fields[name]=json.loads(matches[0])
    except json.JSONDecodeError: fail(f'invalid Cargo.lock package {name}')
  source=fields.get('source','workspace')
  if not all(isinstance(fields.get(name),str) and fields[name] for name in ('name','version')) or not isinstance(source,str) or not source:
   fail('invalid Cargo.lock package identity')
  values.append((fields['name'],fields['version'],source))
 if not values or len(values)!=len(set(values)): fail('Cargo.lock package closure is empty or duplicate')
 return values
def component_items():
 items=[{'ecosystem':'cargo','name':name,'version':version,'source':source}
  for name,version,source in cargo_lock_packages()]
 props=(ROOT/'gradle/wrapper/gradle-wrapper.properties').read_text(); match=re.search(r'gradle-([0-9.]+)-bin.zip',props)
 if not match: fail('Gradle version missing')
 items.append({'ecosystem':'gradle','name':'gradle','version':match.group(1),'source':'verified distribution'})
 return sorted(items,key=lambda x:(x['ecosystem'],x['name'],x['version'],x['source']))
def expected(release_root,version):
 return {'schemaVersion':1,'commit':version,'artifacts':artifact_items(release_root,version),
  'components':component_items(),'contracts':contract_items(version),'images':image_items()}
