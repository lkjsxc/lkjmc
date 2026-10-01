#!/usr/bin/env python3
"""Isolated local development database. Never connects to the production host."""
import argparse
import json
import os
from pathlib import Path
import secrets
import subprocess

ROOT = Path(__file__).resolve().parent.parent
LOCAL = ROOT / '.local'
NAME = 'lkjmc-rebuild-dev-postgres'
PORT = 16543

def run(*args, **kwargs):
    return subprocess.run(args, check=True, **kwargs)

def config():
    LOCAL.mkdir(mode=0o700, exist_ok=True)
    path = LOCAL / 'dev.json'
    if not path.exists():
        data = {'database_url': f'postgres://lkjmc:{secrets.token_hex(32)}@127.0.0.1:{PORT}/lkjmc_rebuild',
                'session_secret': secrets.token_hex(32)}
        fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        with os.fdopen(fd, 'w') as f:
            json.dump(data, f)
    return json.loads(path.read_text())

def database():
    cfg = config()
    check = subprocess.run(['docker','inspect',NAME], capture_output=True, text=True)
    if check.returncode == 0:
        container = json.loads(check.stdout)[0]
        if container['Config']['Labels'].get('com.lkjmc.scope') != 'rebuild-dev':
            raise SystemExit('Container name belongs to another application')
        run('docker','start',NAME)
    else:
        run('docker','pull','postgres:18')
        password = cfg['database_url'].split(':',2)[2].split('@')[0]
        secret = LOCAL / 'postgres-password'
        fd = os.open(secret, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600)
        with os.fdopen(fd,'w') as f:
            f.write(password)
        run('docker','run','-d','--name',NAME,'--label','com.lkjmc.scope=rebuild-dev',
            '-p',f'127.0.0.1:{PORT}:5432','-e','POSTGRES_USER=lkjmc',
            '-e','POSTGRES_DB=lkjmc_rebuild','-e','POSTGRES_PASSWORD_FILE=/run/secrets/password',
            '-v',f'{secret}:/run/secrets/password:ro','-v','lkjmc-rebuild-dev-pg18:/var/lib/postgresql',
            '--health-cmd','pg_isready -U lkjmc -d lkjmc_rebuild','--health-interval','2s',
            '--health-retries','30','postgres:18')
    print(f'Development DB: 127.0.0.1:{PORT}; credentials remain in .local/dev.json')

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('command',choices=['db','api','test','migrate','account','dev-session','credential','register-server'])
    args,extra=parser.parse_known_args()
    if args.command=='db':
        database(); return
    cfg=config()
    env=os.environ.copy()
    env.update({'DATABASE_URL':cfg['database_url'],'LKJMC_BIND':'127.0.0.1:18091',
                'LKJMC_PUBLIC_URL':'http://127.0.0.1:18091','LKJMC_STORAGE':str(LOCAL/'storage'),
                'LKJMC_DEVELOPMENT':'true','RUST_LOG':'lkjmc_core=debug,tower_http=info',
                'PATH':f'{Path.home()}/.cargo/bin:'+env.get('PATH','')})
    dump=LOCAL/'pg-client/root/usr/lib/postgresql/18/bin/pg_dump'
    if dump.exists():env['LKJMC_PG_DUMP']=str(dump)
    command=['cargo','test','--workspace',*extra] if args.command=='test' else ['cargo','run','-p','lkjmc-core','--',args.command,*extra]
    run(*command,cwd=ROOT,env=env)

if __name__=='__main__': main()
