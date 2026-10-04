"""The same complete stable message/parameter contract must ship to both frontends."""
import importlib.util
import json
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[2]
SLOT = re.compile(r'\{([A-Za-z_0-9]+)\}')
SPEC = importlib.util.spec_from_file_location('message_migration', ROOT / 'scripts/migrate_messages.py')
MIGRATION = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MIGRATION)


class Languages(unittest.TestCase):
    def test_complete_catalogs_match_declared_keys_and_parameters(self):
        registry = json.loads((ROOT / 'locales/languages.json').read_text())
        contracts = json.loads((ROOT / 'locales/messages.json').read_text())
        self.assertEqual(registry['default'], 'en')
        codes = [entry['code'] for entry in registry['languages']]
        self.assertEqual(len(set(codes)), len(codes))
        self.assertEqual(set(codes), {'en', 'ja'})
        for language in codes:
            catalog = json.loads((ROOT / 'locales' / (language + '.json')).read_text())
            self.assertEqual(set(catalog), set(contracts), language)
            for key, translated in catalog.items():
                self.assertRegex(key, r'^[a-z][a-z0-9_]*(?:\.[a-z0-9_]+)+$', key)
                self.assertTrue(translated.strip(), key)
                self.assertEqual(sorted(set(SLOT.findall(translated))), contracts[key]['params'], key)
                self.assertNotIn('{}', translated, key)

    def test_migration_never_translates_data_arguments(self):
        mapping = {'Hello {0}': 'text.hello', 'Teams': 'text.teams', 'Open': 'text.open'}
        source = 't("Hello {0}", "Teams"); tr(p, "Hello {0}", "Open"); t(choice === "Open" ? "Teams" : "Open");'
        expected = 't("text.hello", "Teams"); tr(p, "text.hello", "Open"); t(choice === "Open" ? "text.teams" : "text.open");'
        migrated = MIGRATION.migrate_source(source, mapping)
        self.assertEqual(migrated, expected)
        self.assertEqual(MIGRATION.migrate_source(migrated, mapping), expected)

    def test_rust_dynamic_errors_keep_parameters(self):
        source = 'Error::invalid(format!("Enter 1–{max} characters."))'
        migrated = MIGRATION.migrate_source(source, {'Enter 1–{max} characters.': 'text.enter'})
        self.assertEqual(migrated, 'Error::invalid(crate::system_message::SystemMessage::new("text.enter").with("max", max))')
        source = 'Error::conflict(format!("Remaining: {}", 2000 - used))'
        migrated = MIGRATION.migrate_source(source, {'Remaining: {}': 'text.remaining'})
        self.assertEqual(migrated, 'Error::conflict(crate::system_message::SystemMessage::new("text.remaining").with("0", 2000 - used))')

    def test_frontend_literals_are_ids(self):
        contracts = json.loads((ROOT / 'locales/messages.json').read_text())
        sources = [*ROOT.glob('web/src/*.ts'), *ROOT.glob('web/src/*.tsx'), *ROOT.glob('plugins/**/src/main/java/**/*.java')]
        for source in sources:
            if source.name in {'i18n.ts', 'Messages.java'}: continue
            text = source.read_text()
            for match in MIGRATION.CALL.finditer(text):
                fragment = text[match.end():MIGRATION.call_end(text, match.end())]
                args = MIGRATION.argument_spans(fragment)
                index = 0 if match.group().lstrip().startswith(('t(', 't (', 'message')) else 1
                if index >= len(args): continue
                left, right = args[index]
                expression = fragment[left:right].strip()
                literal = re.fullmatch(MIGRATION.LITERAL, expression)
                if literal:
                    key = json.loads(literal.group())
                    self.assertIn(key, contracts, f'{source.relative_to(ROOT)}: {key}')

    def test_runtime_does_not_reverse_lookup_prose(self):
        for path in ['web/src/i18n.ts', 'plugins/common/src/main/java/com/lkjsxc/lkjmc/common/Messages.java']:
            source = (ROOT / path).read_text()
            self.assertNotIn('ERROR_KEYS', source)
            self.assertNotIn('errorMessages', source)
            self.assertNotRegex(source, r'Object\.entries\(japanese\)')
