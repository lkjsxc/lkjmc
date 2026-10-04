#!/usr/bin/env python3
"""Real Core/browser and offline protocol acceptance against explicit test data.

Never provisions a database, resets a world, or stops a foreign listener. CI
creates its own database first. Production identity/Bedrock are separate checks.
"""
import argparse
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import time
import urllib.parse
import urllib.request
import uuid

ROOT = Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--browser', action='store_true')
    parser.add_argument('--protocol', action='store_true')
    parser.add_argument('--expeditions', action='store_true', help='Run the direct Paper expedition crash/recovery scenarios')
    args = parser.parse_args()
    assert args.browser or args.protocol or args.expeditions
    local = ROOT / '.local'
    config = json.loads((local / 'dev.json').read_text())
    url = urllib.parse.urlparse(config['database_url'])
    assert config['scope'] == 'isolated-protocol-test'
    assert url.hostname in ('localhost', '127.0.0.1', '::1') and url.port == 16543
    assert url.username == 'lkjmc' and not url.query and not url.fragment
    database = url.path[1:]
    import re
    assert re.fullmatch(r'lkjmc_test_protocol_[a-z0-9_]{8,48}', database)
    assert config['test_database'] == database
    for port in (18091, 25691, 25692, 25693):
        with socket.socket() as probe:
            probe.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            probe.bind(('127.0.0.1', port))
    tag = uuid.uuid4().hex[:12]
    evidence = local / 'ux' / ('real-' + tag)
    evidence.mkdir(parents=True, mode=0o700)
    env = os.environ.copy()
    env.update(DATABASE_URL=config['database_url'], LKJMC_BIND='127.0.0.1:18091',
               LKJMC_PUBLIC_URL='http://127.0.0.1:18091', LKJMC_DEVELOPMENT='true',
               LKJMC_GAME_ADDRESS='127.0.0.1:25693' if args.protocol or args.browser else '127.0.0.1:25691',
               LKJMC_STORAGE=str(local / 'storage'), LKJMC_WEB=str(ROOT / 'web/dist'),
               LKJMC_TEST_EVIDENCE=str(evidence.relative_to(ROOT)),
               CARGO_NET_OFFLINE='true')
    target = Path(env.get('CARGO_TARGET_DIR', ROOT / 'target'))
    binary = Path(env.get('LKJMC_TEST_CORE', target / 'debug/lkjmc-core'))
    assert binary.is_file()
    core = None
    def run(name, argv, cwd=ROOT):
        with (evidence / (name + '.log')).open('w') as log:
            result = subprocess.run(argv, cwd=cwd, env=env, stdout=log, stderr=subprocess.STDOUT)
        print(name + ': ' + str(result.returncode), flush=True)
        assert result.returncode == 0, 'Inspect ' + str(evidence / (name + '.log'))
    try:
        run('game-setup', ['python3', 'scripts/game_dev.py', 'network-setup' if args.protocol or args.browser else 'setup'])
        if args.browser:
            account = subprocess.check_output([str(binary), 'account', 'Browser ' + tag, '--admin'], cwd=ROOT, env=env, text=True).strip()
            uuid.UUID(account)
            session = evidence / 'browser-session.json'
            run('session', [str(binary), 'dev-session', account, str(session)])
            env['LKJMC_TEST_SESSION'] = str(session)
        with (evidence / 'core.log').open('w') as log:
            core = subprocess.Popen([str(binary), 'api'], cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT)
            deadline = time.monotonic() + 30
            while True:
                assert core.poll() is None, 'The test Core exited; inspect its log'
                try:
                    urllib.request.urlopen('http://127.0.0.1:18091/health/ready', timeout=1).close()
                    break
                except Exception:
                    assert time.monotonic() < deadline, 'Test Core did not become ready'
                    time.sleep(.2)
            if args.browser:
                run('browser-integration', ['./node_modules/.bin/playwright', 'test', '--project=integration', '--workers=1'], ROOT / 'web')
            if args.protocol:
                run('game-protocol', ['node', 'tests/game/network.mjs'])
            # The protocol release lane includes the temporary-world lifecycle.
            # Its direct Paper fixture runs only after the owned network fixture
            # has stopped, using the same isolated DB and task-owned world roots.
            if args.protocol or args.expeditions:
                if args.protocol or args.browser:
                    run('expedition-setup', ['python3', 'scripts/game_dev.py', 'setup'])
                run('game-expeditions', ['node', 'tests/game/adventure.mjs'])
        (evidence / 'result.json').write_text(json.dumps({'database': database, 'browser': args.browser, 'offline_protocol': args.protocol, 'expeditions': args.protocol or args.expeditions, 'passed': True}) + '\n')
        print('Evidence: ' + str(evidence), flush=True)
    finally:
        if core and core.poll() is None:
            core.send_signal(signal.SIGINT)
            try:
                core.wait(timeout=15)
            except subprocess.TimeoutExpired:
                core.terminate()
                core.wait(timeout=15)


if __name__ == '__main__':
    main()
