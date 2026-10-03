"""Keep supported locales and runtime message placeholders consistent."""
import json
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[2]


class Languages(unittest.TestCase):
    def test_catalog_covers_both_frontends_and_preserves_placeholders(self):
        registry = json.loads((ROOT / 'locales/languages.json').read_text())
        self.assertEqual(registry['default'], 'en')
        self.assertEqual(len({entry['code'] for entry in registry['languages']}), len(registry['languages']))
        for language in registry['languages']:
            if language['code'] == 'en':
                continue
            catalog = json.loads((ROOT / 'locales' / (language['code'] + '.json')).read_text())
            for english, translated in catalog.items():
                self.assertTrue(translated, english)
                self.assertEqual(sorted(re.findall(r'\{[a-zA-Z_0-9]*\}', english)),
                                 sorted(re.findall(r'\{[a-zA-Z_0-9]*\}', translated)), english)
            for source in (ROOT / 'web/src').glob('*.tsx'):
                for literal in re.findall(r'\bt\(("(?:[^"\\]|\\.)*")', source.read_text()):
                    self.assertIn(json.loads(literal), catalog, str(source))
            source = ROOT / 'plugins/paper/src/main/java/com/lkjsxc/lkjmc/paper/GameMenus.java'
            for literal in re.findall(r'\btr\(\w+,\s*("(?:[^"\\]|\\.)*")', source.read_text()):
                self.assertIn(json.loads(literal), catalog)
