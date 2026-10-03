#!/usr/bin/env python3
"""Private loopback Paper rig; never reuse its offline-mode configuration in production."""
import argparse,json,os,secrets,shutil,subprocess,sys,uuid
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
LOCAL=ROOT/'.local'
RIG=LOCAL/'game'
JAVA=Path(os.environ.get('JAVA_HOME',LOCAL/'toolchains'/'jdk-25.0.4.1+1'))/'bin'/'java'
def cli(*args):subprocess.run([sys.executable,str(ROOT/'scripts/dev.py'),*args],cwd=ROOT,check=True)
def setup(network=False):
 RIG.mkdir(parents=True,exist_ok=True,mode=0o700)
 state=RIG/'ids.json'
 if not state.exists():
  ids={'official':str(uuid.uuid4()),'lobby':str(uuid.uuid4())}
  with open(state,'x') as f:json.dump(ids,f)
 ids=json.loads(state.read_text())
 for name in ['forwarding-secret','departure-key']:
  key=RIG/name
  if not key.exists():
   with os.fdopen(os.open(key,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600),'w') as f:f.write(secrets.token_hex(32))
 for role,port in [('official',25691),('lobby',25692)]:
  root=RIG/role;plugins=root/'plugins';(plugins/'Lkjmc').mkdir(parents=True,exist_ok=True)
  credential=RIG/(role+'-token')
  if not credential.exists():
   cli('register-server',ids[role],role+' development','--kind',role,'--version','26.2','--address',f'127.0.0.1:{port}')
   cli('credential',role+' development',role,'--server',ids[role],str(credential))
  shutil.copy2(ROOT/'plugins/paper/build/libs/lkjmc-paper.jar',plugins/'lkjmc-paper.jar')
  if role=='official':shutil.copy2(ROOT/'plugins/test-fixture/build/libs/lkjmc-test-fixture.jar',plugins/'lkjmc-test-fixture.jar')
  for name in ['worldedit','worldguard','viaversion','viabackwards']:shutil.copy2(LOCAL/'artifacts'/(name+'.jar'),plugins/(name+'.jar'))
  shutil.copy2(LOCAL/'artifacts'/'paper.jar',root/'paper.jar')
  primary='holding' if role=='official' else 'lobby'
  (root/'eula.txt').write_text('eula=true\n')
  (root/'server.properties').write_text(f'server-ip=127.0.0.1\nserver-port={port}\nonline-mode=false\nenforce-secure-profile=false\nlevel-name={primary}\nallow-nether=false\nview-distance=4\nsimulation-distance=4\nspawn-protection=0\nmax-players=40\nmax-tick-time=120000\nenable-rcon=false\n')
  (root/'bukkit.yml').write_text(f'settings:\n  allow-end: false\nworlds:\n  {primary}:\n    generator: Lkjmc:void\n')
  (plugins/'Lkjmc'/'config.yml').write_text(f'core-url: http://127.0.0.1:18091\ncredential-file: {credential}\nserver-id: {ids[role]}\nrole: {role}\ndeparture-key-file: {RIG/"departure-key"}\nbedrock-compatible: {str(network).lower()}\n')
  (root/'config').mkdir(exist_ok=True)
  (root/'config/paper-global.yml').write_text(f'_version: 31\nproxies:\n  velocity:\n    enabled: {str(network).lower()}\n    online-mode: false\n    secret: "{(RIG/"forwarding-secret").read_text()}"\n')
 proxy=RIG/'proxy-token'
 if not proxy.exists():cli('credential','private test session fixture','proxy',str(proxy))
 if network:
  root=RIG/'proxy';plugins=root/'plugins';(plugins/'lkjmc').mkdir(parents=True,exist_ok=True)
  shutil.copy2(LOCAL/'artifacts/velocity.jar',root/'velocity.jar')
  shutil.copy2(ROOT/'plugins/proxy/build/libs/lkjmc-velocity.jar',plugins/'lkjmc-velocity.jar')
  for name in ['floodgate','geyser']:shutil.copy2(LOCAL/'artifacts'/(name+'.jar'),plugins/(name+'.jar'))
  (plugins/'floodgate').mkdir(exist_ok=True)
  shutil.copy2(ROOT/'plugins/floodgate-link/build/libs/floodgate-lkjmc-database.jar',plugins/'floodgate/floodgate-lkjmc-database.jar')
  (plugins/'floodgate/config.yml').write_text('''key-file-name: key.pem
username-prefix: "."
replace-spaces: true
player-link:
  enabled: true
  require-link: false
  enable-own-linking: true
  allowed: false
  type: lkjmc
  enable-global-linking: false
metrics:
  enabled: false
config-version: 3
''')
  (plugins/'Geyser-Velocity').mkdir(exist_ok=True)
  (plugins/'Geyser-Velocity/config.yml').write_text('''bedrock:
  address: 127.0.0.1
  port: 25693
java:
  auth-type: floodgate
config-version: 4
''')
  shutil.copy2(RIG/'forwarding-secret',root/'forwarding.secret')
  (plugins/'lkjmc/config.json').write_text(json.dumps({'core_url':'http://127.0.0.1:18091','credential_file':str(proxy),'departure_key_file':str(RIG/'departure-key'),'lobby_id':ids['lobby']}))
  (root/'velocity.toml').write_text('''config-version = "2.7"
bind = "127.0.0.1:25693"
motd = "lkjmc private integration test"
show-max-players = 40
online-mode = false
force-key-authentication = false
player-info-forwarding-mode = "MODERN"
forwarding-secret-file = "forwarding.secret"
[servers]
try = []
[forced-hosts]
[advanced]
connection-timeout = 3000
read-timeout = 10000
bungee-plugin-message-channel = false
''')
 print('Loopback-only Paper rigs prepared. These are development worlds.')
def main():
 parser=argparse.ArgumentParser();parser.add_argument('action',choices=['setup','network-setup','official','lobby','proxy']);args=parser.parse_args()
 if args.action in ['setup','network-setup']:setup(args.action=='network-setup');return
 root=RIG/args.action
 os.chdir(root)
 native=root/'native-tmp';native.mkdir(mode=0o700,exist_ok=True)
 # Keep executable native libraries in this private test workspace. CI's
 # system /tmp retains its noexec boundary.
 flags=['-Dlkjmc.testFaults=true','-Dlkjmc.testOffline=true','-Djava.io.tmpdir='+str(native)]
 heap=int(os.environ.get('LKJMC_TEST_PROXY_HEAP_MIB' if args.action=='proxy' else 'LKJMC_TEST_HEAP_MIB','2048'));assert 384<=heap<=4096
 os.execv(str(JAVA),[str(JAVA),*flags,'-Xms128M',f'-Xmx{heap}M','-XX:ActiveProcessorCount=4','-jar','velocity.jar' if args.action=='proxy' else 'paper.jar',*([] if args.action=='proxy' else ['--nogui'])])
if __name__=='__main__':main()
