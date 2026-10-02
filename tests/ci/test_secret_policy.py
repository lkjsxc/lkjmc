import copy
import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('secret_policy', ROOT / 'ops/ci/secret_policy.py')
policy_module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(policy_module)

class SecretPolicy(unittest.TestCase):
    def setUp(self):
        self.policy = json.loads((ROOT / 'ops/ci/secret-policy.json').read_text())
        self.findings = [{'File': name, 'StartLine': line, 'EndLine': line, 'RuleID': 'generic-api-key',
            'Commit': item['historical_commit']} for name,item in self.policy['manifests'].items() for line in item['lines']]

    def test_reviewed_dependency_checksums_do_not_disable_secret_detection(self):
        self.assertEqual(policy_module.classify(self.findings,self.policy,ROOT,False)['unclassified'],0)
        for mutation in ['duplicate','missing','new-file','changed-rule','changed-line']:
            changed=copy.deepcopy(self.findings)
            if mutation=='duplicate':changed.append(changed[0])
            elif mutation=='missing':changed.pop()
            elif mutation=='new-file':changed[0]['File']='production-credentials.json'
            elif mutation=='changed-rule':changed[0]['RuleID']='another-rule'
            else:changed[0]['StartLine']+=100
            with self.subTest(mutation=mutation),self.assertRaises(AssertionError):
                policy_module.classify(changed,self.policy,ROOT,False)

    def test_changed_blob_or_history_coordinate_is_refused(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)
            for name in self.policy['manifests']:
                (root/name).parent.mkdir(parents=True,exist_ok=True)
                shutil.copyfile(ROOT/name,root/name)
            with (root/self.findings[0]['File']).open('ab') as output:
                output.write(b'\n')
            with self.assertRaises(AssertionError):policy_module.classify(self.findings,self.policy,root,False)
        changed=copy.deepcopy(self.findings)
        changed[0]['Commit']='0'*40
        with self.assertRaises(AssertionError):policy_module.classify(changed,self.policy,ROOT,True)

if __name__=='__main__':unittest.main()
