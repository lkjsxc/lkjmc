"""Classify exact reviewed dependency-checksum findings; reject everything else."""
import hashlib
import json
from pathlib import Path
import subprocess

def classify(findings, policy, root, historical):
    assert policy['schema'] == 1
    expected = {(name, line) for name, spec in policy['manifests'].items() for line in spec['lines']}
    seen = set()
    for finding in findings:
        name, line = finding['File'], finding['StartLine']
        if not historical:
            path = Path(name)
            if path.is_absolute():
                name = path.relative_to(root).as_posix()
        spec = policy['manifests'].get(name)
        assert spec is not None and (name, line) in expected
        assert (name, line) not in seen and finding['RuleID'] == 'generic-api-key'
        assert finding['StartLine'] == finding['EndLine']
        if historical:
            assert finding['Commit'] == spec['historical_commit']
            data = subprocess.check_output(['git', 'show', finding['Commit'] + ':' + name], cwd=root)
        else:
            source = root / name
            assert not source.is_symlink()
            data = source.read_bytes()
        assert hashlib.sha256(data).hexdigest() == spec['sha256']
        seen.add((name, line))
    assert seen == expected, 'Scanner result differs from the reviewed baseline'
    return {'findings': len(findings), 'reviewed_public_dependency_checksums': len(seen), 'unclassified': 0}
