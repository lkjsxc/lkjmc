#!/usr/bin/env python3
"""Fail closed while auditing a bounded Docker save archive."""
import argparse,gzip,hashlib,json,stat,sys,tarfile
from pathlib import Path,PurePosixPath
MAX_ARCHIVE=2*1024*1024*1024
MAX_EXPANDED=4*1024*1024*1024
MAX_METADATA=4*1024*1024
MAX_MEMBERS=4096
MAX_IMAGES=128
MAX_LAYER_REFS=8192
SHA256='sha256:'

def fail(message): raise RuntimeError(message)
def canonical(name):
 path=PurePosixPath(name)
 return bool(path.parts) and not path.is_absolute() and '..' not in path.parts and str(path)==name and len(name)<=512
def object_no_duplicates(pairs):
 result={}
 for key,value in pairs:
  if key in result: fail(f'duplicate JSON key: {key}')
  result[key]=value
 return result
def digest_value(value):
 if not isinstance(value,str) or not value.startswith(SHA256): fail('invalid sha256 digest')
 raw=value[len(SHA256):]
 if len(raw)!=64 or any(c not in '0123456789abcdef' for c in raw): fail('invalid sha256 digest')
 return raw
def blob_name(value): return 'blobs/sha256/'+digest_value(value)
def read_bytes(archive,item,limit):
 if item.size>limit: fail(f'oversized metadata member: {item.name}')
 source=archive.extractfile(item)
 if source is None: fail(f'unreadable member: {item.name}')
 data=source.read(limit+1)
 if len(data)!=item.size: fail(f'truncated member: {item.name}')
 return data
def read_json(archive,item,limit=MAX_METADATA):
 try: return json.loads(read_bytes(archive,item,limit),object_pairs_hook=object_no_duplicates)
 except RuntimeError: raise
 except Exception: fail(f'invalid JSON member: {item.name}')
def hash_bytes(data): return hashlib.sha256(data).hexdigest()
def hash_member(archive,item):
 source=archive.extractfile(item); digest=hashlib.sha256(); total=0
 if source is None: fail(f'unreadable member: {item.name}')
 while chunk:=source.read(1024*1024): digest.update(chunk); total+=len(chunk)
 if total!=item.size: fail(f'truncated member: {item.name}')
 return digest.hexdigest()
def path_digest(name):
 parts=PurePosixPath(name).parts
 if len(parts)==3 and parts[:2]==('blobs','sha256') and len(parts[2])==64: return digest_value(SHA256+parts[2])
 if len(parts)==1 and name.endswith('.json') and len(name)==69: return digest_value(SHA256+name[:-5])
 return None
class HashingReader:
 def __init__(self,source,prefix,digest): self.source=source; self.prefix=prefix; self.digest=digest
 def read(self,size=-1):
  if self.prefix:
   if size<0: data=self.prefix+self.source.read(); self.prefix=b''
   else:
    data=self.prefix[:size]; self.prefix=self.prefix[size:]
    if len(data)<size: data+=self.source.read(size-len(data))
  else: data=self.source.read(size)
  self.digest.update(data); return data
 def close(self): pass

def audit_layer(archive,item,expected):
 source=archive.extractfile(item)
 if source is None: fail(f'unreadable layer: {item.name}')
 first=source.read(2); raw=hashlib.sha256(); expanded=hashlib.sha256(); total=0
 reader=HashingReader(source,first,raw)
 stream=gzip.GzipFile(fileobj=reader) if first==b'\x1f\x8b' else reader
 try:
  while chunk:=stream.read(1024*1024):
   total+=len(chunk)
   if total>MAX_EXPANDED: fail('expanded image layers exceed bound')
   expanded.update(chunk)
 except RuntimeError: raise
 except Exception: fail(f'invalid compressed layer: {item.name}')
 if stream is not reader: reader.read()
 declared=path_digest(item.name)
 if declared is not None and raw.hexdigest()!=declared: fail(f'layer blob digest mismatch: {item.name}')
 if expanded.hexdigest()!=expected: fail(f'layer content digest mismatch: {item.name}')
 return total

def main():
 parser=argparse.ArgumentParser(); parser.add_argument('--path',required=True); args=parser.parse_args()
 path=Path(args.path); mode=path.lstat().st_mode
 if path.is_symlink() or not stat.S_ISREG(mode): fail('saved image is not a regular file')
 size=path.stat().st_size
 if not 0<size<=MAX_ARCHIVE: fail(f'saved image size outside bound: {size}')
 with tarfile.open(path,mode='r:') as archive:
  listed=archive.getmembers()
  if len(listed)>MAX_MEMBERS: fail('saved image member count exceeds bound')
  members={}; directories=set(); file_bytes=0
  for item in listed:
   if not canonical(item.name): fail(f'unsafe saved image member: {item.name}')
   if item.name in members or item.name in directories: fail(f'duplicate conflicting member: {item.name}')
   if item.isdir(): directories.add(item.name); continue
   if item.type not in (tarfile.REGTYPE,tarfile.AREGTYPE) or item.sparse is not None: fail(f'nonregular saved image member: {item.name}')
   if item.size<0 or item.size>MAX_ARCHIVE: fail(f'oversized saved image member: {item.name}')
   file_bytes+=item.size
   if file_bytes>MAX_ARCHIVE: fail('saved image members exceed bound')
   members[item.name]=item
  manifest=members.get('manifest.json')
  if manifest is None: fail('bounded Docker manifest missing')
  data=read_json(archive,manifest)
  if not isinstance(data,list) or not data or len(data)>MAX_IMAGES: fail('invalid Docker manifest')
  expected={'manifest.json'}; configs=set(); layer_expect={}; refs=0
  for image in data:
   if not isinstance(image,dict) or set(image)!= {'Config','RepoTags','Layers'}: fail('invalid Docker manifest entry')
   config=image['Config']; layers=image['Layers']; tags=image['RepoTags']
   if not isinstance(config,str) or not canonical(config) or not isinstance(layers,list): fail('invalid Docker manifest entry')
   if tags is not None and (not isinstance(tags,list) or any(not isinstance(tag,str) for tag in tags)): fail('invalid Docker repository tags')
   if config not in members: fail(f'declared image config missing: {config}')
   expected.add(config); configs.add(config); config_data=read_json(archive,members[config])
   declared=path_digest(config)
   if declared is None or hash_bytes(read_bytes(archive,members[config],MAX_METADATA))!=declared: fail(f'image config digest mismatch: {config}')
   try: diff_ids=config_data['rootfs']['diff_ids']
   except (KeyError,TypeError): fail(f'invalid image config: {config}')
   if not isinstance(diff_ids,list) or len(diff_ids)!=len(layers): fail(f'image config layer count mismatch: {config}')
   refs+=len(layers)
   if refs>MAX_LAYER_REFS: fail('saved image layer reference count exceeds bound')
   for name,diff_id in zip(layers,diff_ids):
    if not isinstance(name,str) or not canonical(name): fail('invalid declared image layer')
    digest=digest_value(diff_id)
    if name in layer_expect and layer_expect[name]!=digest: fail(f'conflicting shared layer digest: {name}')
    layer_expect[name]=digest; expected.add(name)
  if not layer_expect: fail('Docker manifest declares no layers')
  for name in layer_expect:
   if name not in members: fail(f'declared image layer missing: {name}')
  hashed=set()
  def verify_blob(name):
   if name in hashed: return
   item=members.get(name)
   if item is None: fail(f'declared OCI member missing: {name}')
   declared=path_digest(name)
   if declared is None or hash_member(archive,item)!=declared: fail(f'OCI blob digest mismatch: {name}')
   hashed.add(name)
  def walk(descriptor,depth=0):
   if depth>8 or not isinstance(descriptor,dict): fail('invalid OCI descriptor')
   name=blob_name(descriptor.get('digest'))
   if not isinstance(descriptor.get('size'),int) or descriptor['size']<0: fail('invalid OCI descriptor size')
   item=members.get(name)
   if item is None or item.size!=descriptor['size']: fail(f'OCI descriptor size mismatch: {name}')
   expected.add(name); media=descriptor.get('mediaType','')
   if name not in layer_expect: verify_blob(name)
   if media.endswith('image.index.v1+json') or media.endswith('manifest.list.v2+json'):
    value=read_json(archive,item)
    if not isinstance(value,dict) or not isinstance(value.get('manifests'),list): fail(f'invalid OCI index: {name}')
    for child in value['manifests']: walk(child,depth+1)
   elif media.endswith('image.manifest.v1+json') or media.endswith('manifest.v2+json'):
    value=read_json(archive,item); children=[]
    if not isinstance(value,dict): fail(f'invalid OCI manifest: {name}')
    if 'config' in value: children.append(value['config'])
    children.extend(value.get('layers',value.get('blobs',[])))
    for child in children: walk(child,depth+1)
  oci_roots={'index.json','oci-layout'}
  present=oci_roots&set(members)
  if present and present!=oci_roots: fail('incomplete OCI metadata')
  if present:
   expected|=oci_roots
   layout=read_json(archive,members['oci-layout'])
   if layout!={'imageLayoutVersion':'1.0.0'}: fail('invalid OCI layout')
   index=read_json(archive,members['index.json'])
   if not isinstance(index,dict) or not isinstance(index.get('manifests'),list): fail('invalid OCI root index')
   for descriptor in index['manifests']: walk(descriptor)
  if 'repositories' in members:
   read_json(archive,members['repositories']); expected.add('repositories')
  extras=set(members)-expected
  if extras: fail('unreferenced saved image member: '+sorted(extras)[0])
  allowed_dirs={str(parent) for name in expected for parent in PurePosixPath(name).parents if str(parent)!='.'}
  extras=directories-allowed_dirs
  if extras: fail('unreferenced saved image directory: '+sorted(extras)[0])
  layer_bytes=sum(members[name].size for name in layer_expect); expanded=0
  for name in sorted(layer_expect):
   expanded+=audit_layer(archive,members[name],layer_expect[name])
   if expanded>MAX_EXPANDED: fail('expanded image layers exceed bound')
 print(f'ok saved-image-audit archiveBytes={size} images={len(data)} layerReferences={refs} layers={len(layer_expect)} layerBytes={layer_bytes} expandedLayerBytes={expanded}')
if __name__=='__main__':
 try: main()
 except Exception as error:
  print(f'saved image audit failed: {error}',file=sys.stderr); sys.exit(1)
