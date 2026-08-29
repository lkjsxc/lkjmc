#!/usr/bin/env python3
"""Deterministic Docker-save audit and secret-scan falsifiers."""
import hashlib,io,json,os,subprocess,sys,tarfile,tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
AUDIT=(sys.executable,str(ROOT/'scripts/audit-saved-image.py'),'--path')
SCAN=(sys.executable,str(ROOT/'scripts/scan-secrets.py'),'--canary','fixture-'+('c'*40),'--path')
MAX_ARCHIVE=2*1024*1024*1024
CREATED='2026-08-29T00:00:00Z'
EPOCH='1970-01-01T00:00:00Z'
ZERO_CONFIG={'Hostname':'','AttachStdin':False,'Env':None,'Cmd':None,'Volumes':None,'Labels':None}

def require(ok,message):
 if not ok: raise RuntimeError(message)
def command(argv,ok=True,capture=False):
 done=subprocess.run(tuple(map(str,argv)),cwd=ROOT,text=True,stdout=subprocess.PIPE if capture else subprocess.DEVNULL,stderr=subprocess.PIPE)
 require((done.returncode==0)==ok,f'command outcome differs: {argv}; stderr={done.stderr.strip()}')
 return done.stdout if capture else ''
def tar_payload(label,secret=False):
 raw=io.BytesIO()
 with tarfile.open(fileobj=raw,mode='w') as archive:
  data=(b'TOKEN='+b'z'*40+b'\n') if secret else f'fixture layer {label}\n'.encode()
  item=tarfile.TarInfo('app/value'); item.size=len(data); item.mode=0o600; item.mtime=0
  archive.addfile(item,io.BytesIO(data))
 return raw.getvalue()
def config_bytes(diff_ids,architecture):
 value={'architecture':architecture,'config':{'Cmd':['/bin/true']},'created':CREATED,'os':'linux',
        'rootfs':{'diff_ids':['sha256:'+item for item in diff_ids],'type':'layers'}}
 return json.dumps(value,separators=(',',':'),sort_keys=True).encode()
def add_file(archive,name,data):
 item=tarfile.TarInfo(name); item.size=len(data); item.mode=0o600; item.mtime=0
 archive.addfile(item,io.BytesIO(data))
def json_bytes(value): return json.dumps(value,separators=(',',':'),sort_keys=True).encode()
def go_json(value):
 raw=json.dumps(value,ensure_ascii=False,separators=(',',':'),allow_nan=False)
 return raw.replace('&','\\u0026').replace('<','\\u003c').replace('>','\\u003e').replace('\u2028','\\u2028').replace('\u2029','\\u2029')
def legacy_identifier(value,chain_id,top):
 fields={key:item for key,item in value.items() if key!='id'}; fields['layer_id']=chain_id
 if 'parent' in fields: fields['parent']='sha256:'+fields['parent']
 if not top: fields.pop('os',None)
 encoded='{'+','.join(go_json(key)+':'+go_json(fields[key]) for key in sorted(fields))+'}'
 return hashlib.sha256(encoded.encode()).hexdigest()
def legacy_chain(diff_ids,architecture,mutation=None):
 result=[]; chain_id=''; parent=None
 for number,diff_id in enumerate(diff_ids):
  chain_id='sha256:'+diff_id if not chain_id else 'sha256:'+hashlib.sha256((chain_id+' sha256:'+diff_id).encode()).hexdigest()
  top=number==len(diff_ids)-1
  value={'created':CREATED,'container_config':dict(ZERO_CONFIG),'config':dict(ZERO_CONFIG)|{'Cmd':['/bin/true']},'architecture':architecture,'os':'linux'} if top else {'created':EPOCH,'container_config':dict(ZERO_CONFIG),'os':'linux'}
  if parent is not None: value['parent']=parent
  if top and mutation=='legacy-parent': value['parent']='1'*64
  if top and mutation=='legacy-field': value['unexpected']='value'
  if top and mutation=='legacy-config': value['config']={'Cmd':['/bin/false']}
  if top and mutation=='legacy-config-extra': value['config']['Image']='unexpected'
  if top and mutation=='legacy-config-schema': value['config']['Unexpected']=''
  if top and mutation=='legacy-container-config': value['container_config']={'Cmd':['/bin/false']}
  if not top and mutation=='legacy-intermediate': value['container_config']={'Cmd':['/bin/false']}
  if top and mutation=='legacy-type': value['architecture']=[]
  identifier=legacy_identifier(value,chain_id,top); value['id']=identifier
  if top and mutation=='legacy-parent-null': value['parent']=None
  if top and mutation=='legacy-id': value['id']='2'*64
  raw=go_json(value).encode(); digest=hashlib.sha256(raw).hexdigest()
  if top and mutation=='legacy-digest': digest='3'*64
  result.append((identifier,'blobs/sha256/'+digest,raw,top)); parent=identifier
 return result
def descriptor(name,data,media,platform=None):
 result={'mediaType':media,'digest':'sha256:'+name.split('/')[-1],'size':len(data)}
 if platform is not None: result['platform']=platform
 return result
def write_image(path,mutation='valid',secret=False):
 layers=[]
 for label in ('one','two'):
  raw=tar_payload(label,secret and label=='one'); digest=hashlib.sha256(raw).hexdigest()
  layers.append((digest,'blobs/sha256/'+digest,raw))
 layer_by_digest={digest:(name,raw) for digest,name,raw in layers}
 specs=(('amd64',(layers[0][0],)),('arm64',(layers[0][0],)),('amd64',(layers[0][0],layers[1][0])))
 configs=[]
 legacy_by_id={}
 for number,(arch,diff_ids) in enumerate(specs):
  raw=config_bytes(diff_ids,arch); digest=hashlib.sha256(raw).hexdigest()
  configs.append(('blobs/sha256/'+digest,raw,arch,diff_ids))
  selected_mutation=mutation if number==2 and mutation.startswith('legacy-') and mutation not in ('legacy-missing','legacy-extra') else None
  for identifier,name,legacy_raw,top in legacy_chain(diff_ids,arch,selected_mutation):
   if mutation=='legacy-missing' and number==2 and top: continue
   legacy_by_id[identifier]=(name,legacy_raw)
 if mutation=='legacy-extra':
  for identifier,name,legacy_raw,_ in legacy_chain((layers[0][0],),'s390x'): legacy_by_id[identifier]=(name,legacy_raw)
 legacy=list(legacy_by_id.values())
 source={'mediaType':'application/vnd.oci.image.layer.v1.tar+gzip','size':23,'digest':'sha256:'+hashlib.sha256(b'compressed-provenance').hexdigest(),
         'annotations':{'org.opencontainers.image.title':'fixture-layer'}}
 manifest=[{'Config':name,'Layers':[layer_by_digest[item][0] for item in diff_ids],'RepoTags':[f'fixture:{number}'],
            'LayerSources':{'sha256:'+item:dict(source) for item in diff_ids}} for number,(name,_,_,diff_ids) in enumerate(configs)]
 manifest[1]['Parent']='sha256:'+configs[0][0].split('/')[-1]
 first_diff=specs[0][1][0]
 if mutation=='source-unknown': manifest[0]['LayerSources']={'sha256:'+'1'*64:dict(source)}
 if mutation=='source-digest': manifest[0]['LayerSources']['sha256:'+first_diff]['digest']='sha512:'+'2'*64
 if mutation=='source-size': manifest[0]['LayerSources']['sha256:'+first_diff]['size']=MAX_ARCHIVE+1
 if mutation=='source-url': manifest[0]['LayerSources']['sha256:'+first_diff]['urls']=['https://mutable.invalid/layer']
 if mutation=='parent-missing': manifest[1]['Parent']='sha256:'+'3'*64
 if mutation=='manifest-field': manifest[0]['Unexpected']='value'
 oci_manifests=[]; selected=[]
 for name,raw,arch,diff_ids in configs:
  value={'schemaVersion':2,'mediaType':'application/vnd.oci.image.manifest.v1+json',
         'config':descriptor(name,raw,'application/vnd.oci.image.config.v1+json'),
         'layers':[descriptor(layer_by_digest[item][0],layer_by_digest[item][1],'application/vnd.oci.image.layer.v1.tar') for item in diff_ids]}
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
  for name,raw,_,_ in configs: add_file(archive,name,raw)
  for number,(_,name,raw) in enumerate(layers):
   if mutation!='missing' or number!=0: add_file(archive,name,raw)
  add_file(archive,'oci-layout',json_bytes({'imageLayoutVersion':'1.0.0'})); add_file(archive,'index.json',index)
  if mutation!='oci-root-missing': add_file(archive,upstream_name,upstream)
  for number,(name,raw) in enumerate(oci_manifests):
   if mutation!='oci-selected-missing' or number!=1: add_file(archive,name,raw)
  for name,raw in legacy: add_file(archive,name,raw)
  if mutation=='duplicate': add_file(archive,layers[0][1],b'conflicting layer')
  if mutation=='extra': add_file(archive,'hidden/value',b'undeclared')
  if mutation=='traversal': add_file(archive,'../escape',b'bad')
  if mutation in ('symlink','device'):
   item=tarfile.TarInfo('special')
   if mutation=='symlink': item.type=tarfile.SYMTYPE; item.linkname='manifest.json'
   else: item.type=tarfile.CHRTYPE; item.devmajor=1; item.devminor=3
   archive.addfile(item)
def check():
 require(legacy_identifier({'created':CREATED,'container_config':{'Env':['A=<x>&','雪'],'Cmd':['sh','-c','true']},'architecture':'amd64','os':'linux','id':'0'*64},'sha256:'+'1'*64,True)=='37aa81fa10e8a096055c7da1917336dcd703a30ff4a375a5739115a8fb67c2ae','legacy top ID differs from Go encoding/json')
 require(legacy_identifier({'created':EPOCH,'container_config':{},'os':'linux','id':'0'*64},'sha256:'+'2'*64,False)=='7fce4950f1580b0427c9710935b1bb53086255244be5fa5d53b31b96864d82f6','legacy base ID differs from Go encoding/json')
 require(legacy_identifier({'created':CREATED,'container_config':{},'config':{'Cmd':['/bin/true']},'architecture':'amd64','os':'linux','parent':'3'*64,'id':'0'*64},'sha256:'+'4'*64,True)=='a0e6391b7c5d6d6a26e30ffecfbdb367faba2ba29301270ae4a9023ab67862fd','legacy parent ID differs from Go encoding/json')
 with tempfile.TemporaryDirectory(prefix='lkjmc-saved-image-check-') as raw:
  root=Path(raw); valid=root/'valid.tar'; write_image(valid)
  first=command((*AUDIT,valid),capture=True); second=command((*AUDIT,valid),capture=True)
  require(first==second and 'images=3 layerReferences=4 layers=2 legacyConfigs=4' in first,'shared layer output is not deterministic')
  for mutation in ('missing','duplicate','extra','traversal','symlink','device','source-unknown','source-digest','source-size','source-url','parent-missing','manifest-field','oci-size','oci-root-missing','oci-selected-missing','legacy-missing','legacy-extra','legacy-id','legacy-parent','legacy-parent-null','legacy-field','legacy-config','legacy-config-extra','legacy-config-schema','legacy-container-config','legacy-intermediate','legacy-type','legacy-digest'):
   path=root/f'{mutation}.tar'; write_image(path,mutation=mutation); command((*AUDIT,path),ok=False)
  oversized=root/'oversized.tar'
  with oversized.open('wb') as output: output.truncate(MAX_ARCHIVE+1)
  command((*AUDIT,oversized),ok=False)
  hidden=root/'hidden.tar'; write_image(hidden,secret=True)
  command((*AUDIT,hidden)); command((*SCAN,hidden),ok=False)
if __name__=='__main__': check(); print('ok saved-image-semantic-checks')
