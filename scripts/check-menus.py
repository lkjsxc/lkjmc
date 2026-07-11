#!/usr/bin/env python3
from pathlib import Path
import json, re, sys

M=Path('contracts/menus'); EN=Path('config/locales/en.json'); CMD=Path('contracts/commands.json'); RD=Path('docs/product/gui/routes'); SCHEMA=Path('contracts/menus.schema.json')
KINDS={'static','list','detail','confirm','custom'}; THEMES={'root','network','travel','claims','economy','social','profile','settings','staff','adventure','danger','docs'}
ROLES={'info','action','navigation','decoration','disabled','success','danger'}; QUOTES={'random-teleport-nether-confirm','random-teleport-end-confirm'}
REASONS={'deletes-durable-state','overwrites-named-durable-state','creates-durable-world-state','writes-named-durable-state','stops-server','forceful-server-mutation','starts-durable-resources','starts-temporary-infrastructure','affects-other-players','changes-moderation-state','paid-dimension-change'}
REG={'interior-28':{10,11,12,13,14,15,16,19,20,21,22,23,24,25,28,29,30,31,32,33,34,37,38,39,40,41,42,43},'interior-21':{19,20,21,22,23,24,25,28,29,30,31,32,33,34,37,38,39,40,41,42,43},'filter-row':{10,11,12,13,14,15,16},'detail-band':{20,21,22,23,24},'confirm-pair':{11,15}}
BORDER=set(range(9))|set(range(45,54))|{9,18,27,36,17,26,35,44}; P_RE=re.compile(r'@param\.([A-Za-z0-9_-]+)'); ID_RE=re.compile(r'^[a-z0-9]+(?:-[a-z0-9]+)*$')


def e(es,p,f,m): es.append(f'{p}:{f}: {m}')
def j(p,es):
    try: return json.loads(p.read_text(encoding='utf-8'))
    except Exception as ex: e(es,p,'json',str(ex)); return None

def validate_schema(value,schema,path,errors,root):
    if '$ref' in schema: return validate_schema(value,root['$defs'][schema['$ref'].rsplit('/',1)[-1]],path,errors,root)
    kinds=schema.get('type'); kinds=[kinds] if isinstance(kinds,str) else kinds
    checks={'object':lambda x:isinstance(x,dict),'array':lambda x:isinstance(x,list),
            'string':lambda x:isinstance(x,str),'integer':lambda x:isinstance(x,int) and not isinstance(x,bool),
            'boolean':lambda x:isinstance(x,bool),'null':lambda x:x is None}
    if kinds and not any(checks[k](value) for k in kinds): e(errors,path,'schema',f'expected {kinds}'); return
    if 'enum' in schema and value not in schema['enum']: e(errors,path,'schema','value outside enum')
    if isinstance(value,str):
        if len(value)<schema.get('minLength',0): e(errors,path,'schema','string too short')
        if 'pattern' in schema and not re.match(schema['pattern']+'$',value): e(errors,path,'schema','pattern mismatch')
    if isinstance(value,int) and not isinstance(value,bool):
        if value<schema.get('minimum',value) or value>schema.get('maximum',value): e(errors,path,'schema','number out of range')
    if isinstance(value,list) and 'items' in schema:
        for i,item in enumerate(value): validate_schema(item,schema['items'],f'{path}[{i}]',errors,root)
    if isinstance(value,dict):
        props=schema.get('properties',{})
        for key in schema.get('required',[]):
            if key not in value: e(errors,path,'schema',f'missing {key}')
        if schema.get('additionalProperties') is False:
            for key in value:
                if key not in props: e(errors,path,'schema',f'unknown {key}')
        for key,item in value.items():
            if key in props: validate_schema(item,props[key],f'{path}.{key}',errors,root)

def pnames(d): return {p.get('name') for p in d.get('params',[]) if isinstance(p,dict)}
def reqs(d): return {p.get('name') for p in d.get('params',[]) if isinstance(p,dict) and p.get('required')}
def vals(v):
    if isinstance(v,dict):
        for x in v.values(): yield from vals(x)
    elif isinstance(v,list):
        for x in v: yield from vals(x)
    else: yield v

def cslots(d):
    c=d.get('chrome') or {}; s={4} if c.get('info') else set()
    if d.get('size')==27: return s|({26} if c.get('close') else set())
    for key,slot in [('mainMenu',45),('back',49),('refresh',50),('close',53)]:
        if c.get(key): s.add(slot)
    if (d.get('list') or {}).get('pagination'): s|={46,47,48}
    return s

def locales(d,p,en,es):
    refs=[('title',d.get('title')),('chrome.info',(d.get('chrome') or {}).get('info'))]
    li=d.get('list') or {}; refs.append(('list.emptyName',li.get('emptyName'))); refs += [(f'list.emptyLore[{i}]',x) for i,x in enumerate(li.get('emptyLore',[]))]
    for i,s in enumerate(d.get('static',[])):
        refs.append((f'static[{i}].name',s.get('name'))); refs += [(f'static[{i}].lore[{k}]',x) for k,x in enumerate(s.get('lore',[]))]
        a=s.get('action') or {}; refs += [(f'static[{i}].action.{k}',a.get(k)) for k in ['ok','fail','prompt','key'] if a.get(k)]
    for f,k in refs:
        if k and not str(k).startswith('literal:') and k not in en: e(es,p,f,f'missing locale key {k}')

def shape(d,p,es):
    for k in ['id','kind','title','theme','size','params','parent','chrome','static']:
        if k not in d: e(es,p,k,'missing field')
    rid=d.get('id'); data=d.get('data'); kind=d.get('kind')
    if rid!=p.stem: e(es,p,'id','must equal filename')
    if not isinstance(rid,str) or not ID_RE.match(rid): e(es,p,'id','must be lowercase kebab-case')
    if kind not in KINDS: e(es,p,'kind','unknown menu kind')
    if d.get('theme') not in THEMES: e(es,p,'theme','unknown menu theme')
    if d.get('size') not in {27,54}: e(es,p,'size','must be 27 or 54')
    seen=set()
    for i,x in enumerate(d.get('params',[])):
        name=x.get('name') if isinstance(x,dict) else None
        if not name or name in seen: e(es,p,f'params[{i}]','must be unique named objects')
        seen.add(name)
    if kind=='static' and data is not None: e(es,p,'data','static documents must not declare data')
    if kind in {'list','detail'} and not data: e(es,p,'data','list and detail documents require data')
    if kind=='confirm':
        if d.get('size')!=27: e(es,p,'size','confirm documents must be 27 slots')
        if d.get('confirmation') not in REASONS: e(es,p,'confirmation','unknown confirmation reason')
        if data and rid not in QUOTES: e(es,p,'data','only quote confirmation routes may bind data')
        if not data and {s.get('slot') for s in d.get('static',[])}!=REG['confirm-pair']: e(es,p,'static','confirm documents must use slots 11 and 15')
    if data:
        if data.get('source') not in {'daemon','local'}: e(es,p,'data.source','must be daemon or local')
        if not data.get('binding'): e(es,p,'data.binding','missing binding id')
    if bool((d.get('chrome') or {}).get('refresh')) != bool(data and data.get('source')=='daemon'): e(es,p,'chrome.refresh','must match daemon data binding presence')

def slots(d,p,es):
    seen=set(); controls=cslots(d); region=set(); li=d.get('list') or {}
    for f in ['region','reserved']:
        name=li.get(f)
        if name and name not in REG: e(es,p,f'list.{f}','unknown region name')
        elif name: region |= REG[name]
    if region & controls: e(es,p,'list.region','region overlaps chrome slots')
    for i,spec in enumerate(d.get('static',[])):
        s=spec.get('slot'); f=f'static[{i}].slot'
        if not isinstance(s,int) or s<0 or s>=d.get('size',0): e(es,p,f,'slot out of bounds')
        if s in seen: e(es,p,f,'duplicate static slot')
        seen.add(s)
        if s in controls: e(es,p,f,'collides with declared chrome')
        if d.get('size')==54 and s in BORDER: e(es,p,f,'functional slots must leave borders to chrome')
        if s in region: e(es,p,f,'collides with list region')
        if spec.get('role') not in ROLES: e(es,p,f'static[{i}].role','unknown slot role')
        if spec.get('role') in {'info','decoration'} and (spec.get('action') or {}).get('type')!='none': e(es,p,f'static[{i}].action','inert roles require none action')

def actions(d,p,docs,cmds,es):
    src=pnames(d)
    for i,s in enumerate(d.get('static',[])):
        a=s.get('action') or {}; typ=a.get('type')
        if typ not in {'open','back','close','refresh','command','daemon','input','message','none'}: e(es,p,f'static[{i}].action.type','unknown action type')
        if typ=='open':
            t=docs.get(a.get('route'))
            if not t: e(es,p,f'static[{i}].action.route','unknown route')
            else:
                passed=set((a.get('params') or {}).keys())
                for k in sorted(passed-pnames(t)): e(es,p,f'static[{i}].action.params.{k}','target does not declare param')
                for k in sorted(reqs(t)-passed): e(es,p,f'static[{i}].action.params.{k}','required target param missing')
        if typ=='daemon' and a.get('command') not in cmds: e(es,p,f'static[{i}].action.command','daemon command lacks paper surface')
        if 'acceptMinecraftEula' in (a.get('body') or {}): e(es,p,f'static[{i}].action.body','EULA acceptance is generated only')
        if a.get('eulaAcceptance'):
            informed = (d.get('id') == 'adventures-end-confirm' and typ == 'daemon'
                and a.get('command') == 'adventure.purchase'
                and d.get('confirmation') == 'starts-temporary-infrastructure'
                and d.get('title') == 'menu.adventures.end.eula.title'
                and (d.get('chrome') or {}).get('info') == 'menu.adventures.end.eula.info'
                and s.get('name') == 'menu.adventures.end.eula.accept'
                and s.get('lore') == ['menu.adventures.end.eula.accept.lore'])
            if not informed: e(es,p,f'static[{i}].action.eulaAcceptance','requires informed End Expedition action')
        for v in list(vals(a.get('body',{}))) + list((a.get('params') or {}).values()) + list(vals(a.get('args',{}))):
            if isinstance(v,str):
                for tok in P_RE.findall(v):
                    if tok not in src: e(es,p,f'static[{i}].action.body',f'undeclared param token {tok}')

def data_cmds(d,p,cmds,es):
    for i,c in enumerate((d.get('data') or {}).get('commands',[])):
        if c not in cmds: e(es,p,f'data.commands[{i}]','daemon command lacks paper surface')

def reach(docs,paths,es):
    g={r:set() for r in docs}
    for r,d in docs.items():
        par=d.get('parent')
        if r=='root' and par is not None: e(es,paths[r],'parent','root parent must be null')
        if r!='root':
            if par not in docs: e(es,paths[r],'parent','unknown parent route')
            else: g[par].add(r)
        for s in d.get('static',[]):
            a=s.get('action') or {}
            if a.get('type')=='open' and a.get('route') in docs: g[r].add(a['route'])
    seen=set(); stack=['root'] if 'root' in docs else []
    while stack:
        r=stack.pop(); seen.add(r); stack += sorted(g[r]-seen)
    for r in sorted(set(docs)-seen): e(es,paths[r],'id','route is not reachable from root')

def doc_parity(ids,es):
    found=set()
    if not RD.exists(): es.append(f'{RD}:README.md: run scripts/generate-menu-docs.py'); return
    for p in RD.glob('*.md'):
        if p.name!='README.md': found |= set(re.findall(r'contracts/menus/([a-z0-9-]+)\.json',p.read_text(encoding='utf-8')))
    for r in sorted(ids-found): es.append(f'{RD}:routes: missing generated route {r}')
    for r in sorted(found-ids): es.append(f'{RD}:routes: stale generated route {r}')

def main():
    es=[]; en=j(EN,es) or {}; ci=(j(CMD,es) or {}).get('commands',[]); cmds={c['name'] for c in ci if 'paper' in c.get('surfaces',[])}
    schema=j(SCHEMA,es) if SCHEMA.is_file() else None
    if not isinstance(schema,dict): es.append(f'{SCHEMA}: missing menu schema')
    docs={}; paths={}
    for p in sorted(M.glob('*.json')):
        if p.name == 'README.json':
            continue
        d=j(p,es)
        if not isinstance(d,dict): continue
        if d.get('id') in docs: e(es,p,'id','duplicate route id')
        docs[d.get('id')]=d; paths[d.get('id')]=p
        if schema: validate_schema(d,schema,p,es,schema)
        shape(d,p,es); slots(d,p,es); locales(d,p,en,es); data_cmds(d,p,cmds,es)
    for r,d in docs.items(): actions(d,paths[r],docs,cmds,es)
    eula = [a for s in docs.get('adventures-end-confirm',{}).get('static',[]) if (a:=s.get('action') or {}).get('eulaAcceptance')]
    if len(eula) != 1: e(es,M,'eulaAcceptance','requires exactly one informed action')
    if 'root' not in docs: es.append(f'{M}:root: missing root route')
    reach(docs,paths,es); doc_parity(set(docs),es)
    if es: print('\n'.join(es)); return 1
    print('ok check-menus'); return 0
if __name__=='__main__': sys.exit(main())
