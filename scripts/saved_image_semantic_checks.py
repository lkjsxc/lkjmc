#!/usr/bin/env python3
"""Deterministic Docker-save audit and secret-scan falsifiers."""
import hashlib,io,json,os,subprocess,sys,tarfile,tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
AUDIT=(sys.executable,str(ROOT/'scripts/audit-saved-image.py'),'--path')
SCAN=(sys.executable,str(ROOT/'scripts/scan-secrets.py'),'--canary','fixture-'+('c'*40),'--path')
MAX_ARCHIVE=2*1024*1024*1024

def require(ok,message):
 if not ok: raise RuntimeError(message)
def command(argv,ok=True,capture=False):
 done=subprocess.run(tuple(map(str,argv)),cwd=ROOT,text=True,stdout=subprocess.PIPE if capture else subprocess.DEVNULL,stderr=subprocess.DEVNULL)
 require((done.returncode==0)==ok,f'command outcome differs: {argv}')
 return done.stdout if capture else ''
def tar_payload(secret=False):
 raw=io.BytesIO()
 with tarfile.open(fileobj=raw,mode='w') as archive:
  data=(b'TOKEN='+b'z'*40+b'\n') if secret else b'fixture layer\n'
  item=tarfile.TarInfo('app/value'); item.size=len(data); item.mode=0o600; item.mtime=0
  archive.addfile(item,io.BytesIO(data))
 return raw.getvalue()
def config_bytes(diff_id,architecture):
 value={'architecture':architecture,'os':'linux','rootfs':{'diff_ids':['sha256:'+diff_id],'type':'layers'}}
 return json.dumps(value,separators=(',',':'),sort_keys=True).encode()
def add_file(archive,name,data):
 item=tarfile.TarInfo(name); item.size=len(data); item.mode=0o600; item.mtime=0
 archive.addfile(item,io.BytesIO(data))
def json_bytes(value): return json.dumps(value,separators=(',',':'),sort_keys=True).encode()
def descriptor(name,data,media,platform=None):
 result={'mediaType':media,'digest':'sha256:'+name.split('/')[-1],'size':len(data)}
 if platform is not None: result['platform']=platform
 return result
def write_image(path,mutation='valid',secret=False):
 layer=tar_payload(secret); layer_digest=hashlib.sha256(layer).hexdigest(); layer_name='blobs/sha256/'+layer_digest
 configs=[]
 for arch in ('amd64','arm64'):
  raw=config_bytes(layer_digest,arch); digest=hashlib.sha256(raw).hexdigest()
  configs.append(('blobs/sha256/'+digest,raw,arch))
 source={'mediaType':'application/vnd.oci.image.layer.v1.tar+gzip','size':23,'digest':'sha256:'+hashlib.sha256(b'compressed-provenance').hexdigest(),
         'annotations':{'org.opencontainers.image.title':'fixture-layer'}}
 manifest=[{'Config':name,'Layers':[layer_name],'RepoTags':[f'fixture:{arch}'],
            'LayerSources':{'sha256:'+layer_digest:dict(source)}} for name,_,arch in configs]
 manifest[1]['Parent']='sha256:'+configs[0][0].split('/')[-1]
 if mutation=='source-unknown': manifest[0]['LayerSources']={'sha256:'+'1'*64:dict(source)}
 if mutation=='source-digest': manifest[0]['LayerSources']['sha256:'+layer_digest]['digest']='sha512:'+'2'*64
 if mutation=='source-size': manifest[0]['LayerSources']['sha256:'+layer_digest]['size']=MAX_ARCHIVE+1
 if mutation=='source-url': manifest[0]['LayerSources']['sha256:'+layer_digest]['urls']=['https://mutable.invalid/layer']
 if mutation=='parent-missing': manifest[1]['Parent']='sha256:'+'3'*64
 if mutation=='manifest-field': manifest[0]['Unexpected']='value'
 oci_manifests=[]; selected=[]
 for name,raw,arch in configs:
  value={'schemaVersion':2,'mediaType':'application/vnd.oci.image.manifest.v1+json',
         'config':descriptor(name,raw,'application/vnd.oci.image.config.v1+json'),
         'layers':[descriptor(layer_name,layer,'application/vnd.oci.image.layer.v1.tar')]}
  encoded=json_bytes(value); blob='blobs/sha256/'+hashlib.sha256(encoded).hexdigest(); oci_manifests.append((blob,encoded))
  selected.append(descriptor(blob,encoded,'application/vnd.oci.image.manifest.v1+json',{'architecture':arch,'os':'linux'}))
 missing=b'not retained'; selected.append({'mediaType':'application/vnd.oci.image.manifest.v1+json',
  'digest':'sha256:'+hashlib.sha256(missing).hexdigest(),'size':len(missing),'platform':{'architecture':'s390x','os':'linux'}})
 if mutation=='oci-size': selected[0]['size']+=1
 upstream=json_bytes({'schemaVersion':2,'mediaType':'application/vnd.oci.image.index.v1+json','manifests':selected})
 upstream_name='blobs/sha256/'+hashlib.sha256(upstream).hexdigest()
 index=json_bytes({'schemaVersion':2,'mediaType':'application/vnd.oci.image.index.v1+json',
                   'manifests':[descriptor(upstream_name,upstream,'application/vnd.oci.image.index.v1+json')]})
 with tarfile.open(path,mode='w') as archive:
  add_file(archive,'manifest.json',json_bytes(manifest))
  for name,raw,_ in configs: add_file(archive,name,raw)
  if mutation!='missing': add_file(archive,layer_name,layer)
  add_file(archive,'oci-layout',json_bytes({'imageLayoutVersion':'1.0.0'})); add_file(archive,'index.json',index)
  if mutation!='oci-root-missing': add_file(archive,upstream_name,upstream)
  for number,(name,raw) in enumerate(oci_manifests):
   if mutation!='oci-selected-missing' or number!=1: add_file(archive,name,raw)
  if mutation=='duplicate': add_file(archive,layer_name,b'conflicting layer')
  if mutation=='extra': add_file(archive,'hidden/value',b'undeclared')
  if mutation=='traversal': add_file(archive,'../escape',b'bad')
  if mutation in ('symlink','device'):
   item=tarfile.TarInfo('special')
   if mutation=='symlink': item.type=tarfile.SYMTYPE; item.linkname='manifest.json'
   else: item.type=tarfile.CHRTYPE; item.devmajor=1; item.devminor=3
   archive.addfile(item)
def check():
 with tempfile.TemporaryDirectory(prefix='lkjmc-saved-image-check-') as raw:
  root=Path(raw); valid=root/'valid.tar'; write_image(valid)
  first=command((*AUDIT,valid),capture=True); second=command((*AUDIT,valid),capture=True)
  require(first==second and 'images=2 layerReferences=2 layers=1' in first,'shared layer output is not deterministic')
  for mutation in ('missing','duplicate','extra','traversal','symlink','device','source-unknown','source-digest','source-size','source-url','parent-missing','manifest-field','oci-size','oci-root-missing','oci-selected-missing'):
   path=root/f'{mutation}.tar'; write_image(path,mutation=mutation); command((*AUDIT,path),ok=False)
  oversized=root/'oversized.tar'
  with oversized.open('wb') as output: output.truncate(MAX_ARCHIVE+1)
  command((*AUDIT,oversized),ok=False)
  hidden=root/'hidden.tar'; write_image(hidden,secret=True)
  command((*AUDIT,hidden)); command((*SCAN,hidden),ok=False)
if __name__=='__main__': check(); print('ok saved-image-semantic-checks')
