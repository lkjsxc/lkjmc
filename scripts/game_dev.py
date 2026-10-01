#!/usr/bin/env python3
"""Private loopback Paper rig; never reuse its offline-mode configuration in production."""
import argparse,json,os,shutil,subprocess,sys,uuid
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
LOCAL=ROOT/'.local'
RIG=LOCAL/'game'
JAVA=LOCAL/'toolchains'/'jdk-25.0.4.1+1'/'bin'/'java'
def cli(*args):subprocess.run([sys.executable,str(ROOT/'scripts/dev.py'),*args],cwd=ROOT,check=True)
def setup():
 RIG.mkdir(parents=True,exist_ok=True,mode=0o700)
 state=RIG/'ids.json'
 if not state.exists():
  ids={'official':str(uuid.uuid4()),'lobby':str(uuid.uuid4())}
  with open(state,'x') as f:json.dump(ids,f)
 ids=json.loads(state.read_text())
 for role,port in [('official',25691),('lobby',25692)]:
  root=RIG/role;plugins=root/'plugins';(plugins/'Lkjmc').mkdir(parents=True,exist_ok=True)
  credential=RIG/(role+'-token')
  if not credential.exists():
   cli('register-server',ids[role],role+' development','--kind',role,'--version','26.2','--address',f'127.0.0.1:{port}')
   cli('credential',role+' development',role,'--server',ids[role],str(credential))
  shutil.copy2(ROOT/'plugins/paper/build/libs/lkjmc-paper.jar',plugins/'lkjmc-paper.jar')
  for name in ['worldedit','worldguard','viaversion','viabackwards']:shutil.copy2(LOCAL/'artifacts'/(name+'.jar'),plugins/(name+'.jar'))
  shutil.copy2(LOCAL/'artifacts'/'paper.jar',root/'paper.jar')
  primary='holding' if role=='official' else 'lobby'
  (root/'eula.txt').write_text('eula=true\n')
  (root/'server.properties').write_text(f'server-ip=127.0.0.1\nserver-port={port}\nonline-mode=false\nenforce-secure-profile=false\nlevel-name={primary}\nallow-nether=false\nview-distance=4\nsimulation-distance=4\nspawn-protection=0\nmax-players=40\nmax-tick-time=120000\nenable-rcon=false\n')
  (root/'bukkit.yml').write_text(f'settings:\n  allow-end: false\nworlds:\n  {primary}:\n    generator: Lkjmc:void\n')
  (plugins/'Lkjmc'/'config.yml').write_text(f'core-url: http://127.0.0.1:18091\ncredential-file: {credential}\nserver-id: {ids[role]}\nrole: {role}\n')
 proxy=RIG/'proxy-token'
 if not proxy.exists():cli('credential','private test session fixture','proxy',str(proxy))
 print('Loopback-only Paper rigs prepared. These are development worlds.')
def main():
 parser=argparse.ArgumentParser();parser.add_argument('action',choices=['setup','official','lobby']);args=parser.parse_args()
 if args.action=='setup':setup();return
 root=RIG/args.action
 os.chdir(root)
 os.execv(str(JAVA),[str(JAVA),'-Xms512M','-Xmx4G','-XX:ActiveProcessorCount=4','-jar','paper.jar','--nogui'])
if __name__=='__main__':main()
