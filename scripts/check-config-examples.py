#!/usr/bin/env python3
import json
import sys
from pathlib import Path

EXPECTED = {
    'installRoot', 'configRoot', 'dataRoot', 'logRoot', 'socketPath',
    'database', 'network', 'jars', 'daemonHttp', 'assets', 'plugins', 'runtime',
}
LEGACY = {'paths', 'http'}
DATABASE = {'host', 'port', 'database', 'user', 'secretFile', 'poolSize'}
NETWORK = {
    'name', 'defaultLocale', 'fallbackServer', 'onlineMode',
    'velocityForwarding', 'forwardingSecretFile', 'javaEntry', 'bedrockEntry',
}
JAVA_ENTRY = {'bindHost', 'port', 'publicHosts', 'preferredPublicHost'}
BEDROCK_ENTRY = {'mode', 'host', 'port'}
JARS = {'root', 'defaultChannel', 'userAgent'}
DAEMON_HTTP = {'enabled', 'address', 'tokenFile'}
ASSETS = {'root', 'serverChannel', 'pluginChannel', 'userAgent', 'downloadTimeoutSeconds'}
RUNTIME = {
    'adapter', 'defaultJavaMemoryMb', 'proxyJavaMemoryMb',
    'stopTimeoutSeconds', 'portRangeStart', 'portRangeEnd', 'kubernetes',
}
PLUGIN_INSTALL = {'mode', 'installOn'}
FLOODGATE = {'mode', 'installOn', 'backendApi'}


def check_keys(errors, path, data, expected, *, exact=True):
    keys = set(data)
    unknown = keys - expected
    missing = expected - keys if exact else set()
    for key in sorted(unknown):
        errors.append(f'{path}: unknown key {key}')
    for key in sorted(missing):
        errors.append(f'{path}: missing key {key}')


def check_daemon(path: Path) -> list[str]:
    errors = []
    try:
        data = json.loads(path.read_text())
    except json.JSONDecodeError as error:
        return [f'{path}: invalid json: {error}']
    if not isinstance(data, dict):
        return [f'{path}: expected json object']
    check_keys(errors, str(path), data, EXPECTED)
    for key in LEGACY & set(data):
        errors.append(f'{path}: legacy top-level key {key}')
    database = data.get('database', {})
    if isinstance(database, dict):
        check_keys(errors, f'{path}:database', database, DATABASE)
        if 'url' in database:
            errors.append(f'{path}:database: legacy key url')
    else:
        errors.append(f'{path}: database must be object')
    network = data.get('network', {})
    if isinstance(network, dict):
        check_keys(errors, f'{path}:network', network, NETWORK)
        if isinstance(network.get('javaEntry'), dict):
            check_keys(errors, f'{path}:network.javaEntry', network['javaEntry'], JAVA_ENTRY)
        if isinstance(network.get('bedrockEntry'), dict):
            check_keys(errors, f'{path}:network.bedrockEntry', network['bedrockEntry'], BEDROCK_ENTRY)
    else:
        errors.append(f'{path}: network must be object')
    nested = [
        ('jars', JARS, True),
        ('daemonHttp', DAEMON_HTTP, True),
        ('assets', ASSETS, True),
        ('runtime', RUNTIME, False),
    ]
    for key, expected, exact in nested:
        value = data.get(key, {})
        if isinstance(value, dict):
            check_keys(errors, f'{path}:{key}', value, expected, exact=exact)
        else:
            errors.append(f'{path}: {key} must be object')
    plugins = data.get('plugins', {})
    if isinstance(plugins, dict):
        for name in ('viaversion', 'viabackwards', 'geyser'):
            if isinstance(plugins.get(name), dict):
                check_keys(errors, f'{path}:plugins.{name}', plugins[name], PLUGIN_INSTALL)
        if isinstance(plugins.get('floodgate'), dict):
            check_keys(errors, f'{path}:plugins.floodgate', plugins['floodgate'], FLOODGATE)
        if isinstance(plugins.get('lkjmc'), dict):
            check_keys(errors, f'{path}:plugins.lkjmc', plugins['lkjmc'], {'enabled'})
    else:
        errors.append(f'{path}: plugins must be object')
    return errors


def main() -> int:
    files = sorted(Path('config/defaults').glob('*.json.example'))
    errors = []
    if not files:
        errors.append('config examples: no json examples found')
    for path in files:
        errors.extend(check_daemon(path))
    if errors:
        print('\n'.join(errors))
        return 1
    print('ok check-config-examples')
    return 0


if __name__ == '__main__':
    sys.exit(main())
