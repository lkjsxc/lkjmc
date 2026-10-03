#!/usr/bin/env python3
"""Baked CI entrypoint: isolated PostgreSQL, offline checks, verified release.

Only the summary and explicit release payload leave the job. Diagnostics and
temporary database files stay in ephemeral storage.
"""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import time
from secret_policy import classify

root = Path.cwd()
assert os.getuid() != 0, 'CI must run without root'
assert not any(os.path.lexists(p) for p in ['/var/run/docker.sock', '/var/lib/incus/unix.socket',
    '/root/forgejo-restore-20260906T232532'])
commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
assert commit == os.environ['CI_COMMIT']
assert not subprocess.check_output(['git', 'status', '--porcelain'])
assert not (root / '.local/ci').exists()
local = root / '.local/ci'
reports = local / 'reports'
reports.mkdir(parents=True, mode=0o700)
private = local / 'private'
private.mkdir(mode=0o700)
summary = {'schema': 1, 'commit': commit, 'status': 'running', 'checks': [],
           'production_acceptance': False, 'started': time.time()}
env = os.environ.copy()
env.update({'CARGO_HOME': str(local / 'cargo'), 'CARGO_NET_OFFLINE': 'true', 'CARGO_BUILD_JOBS': '2',
    'GRADLE_USER_HOME': str(local / 'gradle'), 'npm_config_cache': str(local / 'npm'),
    'LKJMC_PG_DUMP': '/usr/lib/postgresql/18/bin/pg_dump', 'RUST_TEST_THREADS': '4'})
env.update(PLAYWRIGHT_BROWSERS_PATH='/opt/ci/browsers', LKJMC_TEST_HEAP_MIB='640', LKJMC_TEST_PROXY_HEAP_MIB='384', LKJMC_TEST_PSQL='/usr/lib/postgresql/18/bin/psql')
for key in ['CI_JOB_TOKEN', 'ACTIONS_RUNTIME_TOKEN', 'GITHUB_TOKEN', 'FORGEJO_TOKEN']:
    env.pop(key, None)

def run(name, argv, **kwargs):
    with (private / (name + '.log')).open('wb') as log:
        result = subprocess.run(argv, env=env, stdout=log, stderr=subprocess.STDOUT, **kwargs)
    summary['checks'].append({'name': name, 'passed': result.returncode == 0})
    if result.returncode:
        raise RuntimeError(name + ' failed; inspect the private job diagnostic')
    print(name + ': passed', flush=True)

def scan(name, source, historical):
    report = private / (name + '.json')
    args = ['gitleaks', 'git' if historical else 'dir', '--config=/opt/ci/lkjmc/gitleaks.toml',
        '--gitleaks-ignore-path=/opt/ci/lkjmc/empty.gitleaksignore', '--ignore-gitleaks-allow',
        '--redact=100', '--report-format=json', '--report-path=' + str(report)]
    if historical:
        args.append('--log-opts=--all')
    with (private / (name + '.log')).open('wb') as log:
        result = subprocess.run(args + [str(source)], env=env, stdout=log, stderr=subprocess.STDOUT)
    assert result.returncode in [0, 1] and report.is_file(), 'Secret scanner did not finish'
    proof = classify(json.loads(report.read_text()),
        json.loads(Path('/opt/ci/lkjmc/secret-policy.json').read_text()), source, historical)
    summary['checks'].append({'name': name, 'passed': True, **proof})
    print(name + ': passed (reviewed public dependency checksums only)', flush=True)

database = None
try:
    scan('secret-history', root, True)
    tracked = private / 'tracked'
    tracked.mkdir()
    # Scan only committed source; test DBs and CI tokens never enter this tree.
    names = subprocess.check_output(['git', 'ls-files', '-z']).split(b'\0')
    for raw in names:
        if not raw:
            continue
        path = Path(os.fsdecode(raw))
        assert not path.is_absolute() and '..' not in path.parts
        source = root / path
        assert source.is_file() and not source.is_symlink()
        target = tracked / path
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)
    scan('secret-source', tracked, False)
    (local / 'cargo').mkdir()
    (local / 'cargo/config.toml').write_text('[source.crates-io]\nreplace-with="ci-vendor"\n[source.ci-vendor]\ndirectory="/opt/ci/vendor"\n[net]\noffline=true\n')
    shutil.copytree('/opt/ci/npm-cache', local / 'npm')
    shutil.copytree('/opt/ci/gradle-cache', local / 'gradle')
    shutil.copytree('/opt/ci/game-artifacts', root / '.local/artifacts')
    run('rust-format', ['cargo', 'fmt', '--all', '--check'])
    run('guest-helper', ['python3', '-m', 'unittest', 'discover', '-s', 'tests/guest', '-v'])
    run('secret-policy', ['python3', '-m', 'unittest', 'discover', '-s', 'tests/ci', '-v'])
    database = tempfile.TemporaryDirectory(prefix='lkjmc-ci-pg-')
    pg = Path(database.name)
    run('postgres-init', ['initdb', '-D', str(pg / 'data'), '-A', 'trust', '-U', 'ci', '--no-locale', '--encoding=UTF8'])
    run('postgres-start', ['pg_ctl', '-D', str(pg / 'data'), '-l', str(pg / 'server.log'),
        '-o', '-h 127.0.0.1 -p 16543 -k ' + str(pg), '-w', 'start'])
    env['DATABASE_URL'] = 'postgres://ci@127.0.0.1:16543/postgres'
    run('rust-and-postgres', ['cargo', 'test', '--workspace', '--locked', '--offline'])
    run('release-build', ['python3', 'scripts/release.py', 'build', '--output', str(local / 'release')])
    payloads = list((local / 'release').glob('*.tar.gz'))
    assert len(payloads) == 1 and payloads[0].name == 'lkjmc-' + commit + '.tar.gz'
    run('release-verify', ['python3', 'scripts/release.py', 'verify', str(payloads[0])])
    run('web-state', ['npm', 'run', 'test:state', '--prefix', 'web'])
    run('browser-fixture', ['./node_modules/.bin/playwright', 'test', '--project=fixture', '--workers=1'], cwd=root / 'web')
    run('protocol-scope', ['node', '--test', 'tests/game/scope.test.mjs'])
    run('protocol-dependencies', ['npm', 'ci', '--offline', '--ignore-scripts', '--prefix', 'tests/game'])
    run('protocol-adapter', [env['LKJMC_GRADLE'], '--no-daemon', '--max-workers=2', '-p', 'plugins', ':test-fixture:jar', '--offline'])
    test_database = 'lkjmc_test_protocol_ci_' + commit[:12]
    run('protocol-database', ['psql', env['DATABASE_URL'], '-X', '-v', 'ON_ERROR_STOP=1', '-c', 'CREATE ROLE lkjmc LOGIN SUPERUSER', '-c', 'CREATE DATABASE ' + test_database + ' OWNER lkjmc'])
    (root / '.local/dev.json').write_text(json.dumps({'scope':'isolated-protocol-test', 'test_database':test_database, 'database_url':'postgres://lkjmc@127.0.0.1:16543/' + test_database}))
    (root / '.local/dev.json').chmod(0o600)
    for role in ('official', 'lobby'):
        shutil.copytree('/opt/ci/paper-runtime', root / '.local/game' / role, dirs_exist_ok=True)
    run('browser-integration', ['python3', 'scripts/ux_verify.py', '--browser'])
    run('game-protocol', ['python3', 'scripts/ux_verify.py', '--protocol'])
    summary['release'] = {'name': payloads[0].name, 'bytes': payloads[0].stat().st_size,
        'sha256': hashlib.file_digest(payloads[0].open('rb'), 'sha256').hexdigest()}
    summary['status'] = 'passed'
except Exception as exc:
    summary['status'] = 'failed'
    summary['error'] = str(exc)
    raise
finally:
    if database:
        subprocess.run(['pg_ctl', '-D', str(Path(database.name) / 'data'), '-m', 'immediate', 'stop'],
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        database.cleanup()
    summary['finished'] = time.time()
    (reports / 'acceptance.json').write_text(json.dumps(summary, indent=2) + '\n')
