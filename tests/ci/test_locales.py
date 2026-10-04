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

    def test_generated_types_match_shared_contract(self):
        manifest = json.loads((ROOT / 'locales/messages.json').read_text())
        self.assertEqual((ROOT / 'web/src/messages.generated.ts').read_text(), MIGRATION.generated_types(manifest))

    def test_migration_never_translates_data_arguments(self):
        mapping = {'Hello {0}': 'text.hello', 'Teams': 'text.teams', 'Open': 'text.open'}
        source = 't("Hello {0}", "Teams"); tr(p, "Hello {0}", "Open"); t(choice === "Open" ? "Teams" : "Open");'
        expected = 't("text.hello", "Teams"); tr(p, "text.hello", "Open"); t(choice === "Open" ? "text.teams" : "text.open");'
        migrated = MIGRATION.migrate_source(source, mapping)
        self.assertEqual(migrated, expected)
        self.assertEqual(MIGRATION.migrate_source(migrated, mapping), expected)

    def test_unknown_literals_remain_exact_and_self_constructors_migrate(self):
        unknown = 'Error::invalid("内部の日本語診断")'
        self.assertEqual(MIGRATION.migrate_source(unknown, {}), unknown)
        source = 'Self::conflict("Known error")'
        self.assertEqual(MIGRATION.migrate_source(source, {'Known error': 'error.known'}), 'Self::conflict("error.known")')

    def test_migrated_routes_preserve_formatting_and_non_label_data(self):
        source = 'const group = second ?? "friends"; const tabs = { home: ["overview", "text.home"] };'
        self.assertEqual(MIGRATION.migrate_source(source, {'Home': 'text.home'}, route_labels=True), source)

    def test_predictable_worker_rejections_are_structured(self):
        mapping = {'Action unavailable': 'error.unavailable'}
        java = 'throw new IllegalArgumentException("Action unavailable");'
        self.assertEqual(MIGRATION.migrate_source(java, mapping, java_errors=True),
                         'throw new IllegalArgumentException(com.lkjsxc.lkjmc.common.SystemMessage.of("error.unavailable").toString());')
        split = 'throw new IllegalArgumentException("Action " + "unavailable");'
        self.assertEqual(MIGRATION.migrate_source(split, mapping, java_errors=True),
                         'throw new IllegalArgumentException(com.lkjsxc.lkjmc.common.SystemMessage.of("error.unavailable").toString());')
        dynamic = 'throw new IllegalArgumentException("Action unavailable" + playerName);'
        self.assertEqual(MIGRATION.migrate_source(dynamic, mapping, java_errors=True), dynamic)
        diagnostic = 'throw new IllegalStateException("Raw I/O diagnosis");'
        self.assertEqual(MIGRATION.migrate_source(diagnostic, mapping, java_errors=True), diagnostic)
        rust = 'json!({"rejected":"Action unavailable","effect":"none"})'
        self.assertEqual(MIGRATION.migrate_source(rust, mapping, host_context=True),
                         'json!({"rejected":crate::system_message::SystemMessage::new("error.unavailable"),"effect":"none"})')

    def test_game_helper_aliases_migrate_templates_and_preserve_raw_components(self):
        source = 'text(player,"  [Join]"); tell(player,"Choose a world"); notice(s,job,phase,"Arrived at {0}.","Choose a world"); Component.text("Choose a world");'
        mapping = {'  [Join]':'text.join', 'Choose a world':'text.choose', 'Arrived at {0}.':'text.arrived'}
        expected = 'text(player,"text.join"); tell(player,"text.choose"); notice(s,job,phase,"text.arrived","Choose a world"); Component.text("Choose a world");'
        self.assertEqual(MIGRATION.migrate_source(source, mapping, game_helpers=True), expected)

    def test_system_message_factories_migrate_only_the_id(self):
        source = 'SystemMessage.of("Arrived at {0}.", "Choose a world");'
        self.assertEqual(MIGRATION.migrate_source(source, {'Arrived at {0}.': 'text.arrived', 'Choose a world': 'text.choose'}),
                         'SystemMessage.of("text.arrived", "Choose a world");')
        source = 'GuestFailure::system("Unavailable", true)'
        self.assertEqual(MIGRATION.migrate_source(source, {'Unavailable': 'error.unavailable'}),
                         'GuestFailure::system("error.unavailable", true)')

    def test_system_producer_slots_reject_prose_without_classifying_player_content(self):
        preview = 'preview.addProperty("message", clear ? "Clear area" : "Blocked area");'
        self.assertTrue(MIGRATION.system_producer_issues(preview, 'BuildingTransactions.java'))
        envelope = 'preview.add("message", SystemMessage.of(clear ? "text.clear" : "text.blocked").json());'
        self.assertEqual(MIGRATION.system_producer_issues(envelope, 'BuildingTransactions.java'), [])
        self.assertTrue(MIGRATION.system_producer_issues('public Waiting(String message) {}', 'IdentityTransactions.java'))
        self.assertEqual(MIGRATION.system_producer_issues('public Waiting(SystemMessage message) {}', 'IdentityTransactions.java'), [])
        self.assertTrue(MIGRATION.system_producer_issues('response.addProperty("reason", error);', 'DepartureGate.java'))
        self.assertEqual(MIGRATION.system_producer_issues('response.add("reason", error.json());', 'DepartureGate.java'), [])
        self.assertTrue(MIGRATION.system_producer_issues('preview.get("message").getAsString();', 'GameMenus.java'))
        self.assertEqual(MIGRATION.system_producer_issues('Messages.render(language, preview.get("message"));', 'GameMenus.java'), [])
        raw_content = 'state.addProperty("reason", "adventure_closed"); Component.text(payload.get("reason").getAsString());'
        self.assertEqual(MIGRATION.system_producer_issues(raw_content, 'SpawnPolicy.java'), [])
        self.assertEqual(MIGRATION.system_producer_issues(raw_content, 'LkjmcProxy.java'), [])
        self.assertTrue(MIGRATION.system_producer_issues('GuestFailure { message: "Unavailable".into(), no_effect: true }', 'worker.rs'))
        self.assertEqual(MIGRATION.system_producer_issues('GuestFailure::system("error.unavailable", true)', 'worker.rs'), [])
        self.assertEqual(MIGRATION.system_producer_issues('GuestFailure { message: error.chars().take(2000).collect(), no_effect: true }', 'incus.rs'), [])

    def test_live_game_producers_use_system_envelopes(self):
        folder = ROOT / 'plugins/paper/src/main/java/com/lkjsxc/lkjmc/paper'
        for name in ['BuildingTransactions.java', 'IdentityTransactions.java', 'DepartureGate.java', 'GameMenus.java']:
            self.assertEqual(MIGRATION.system_producer_issues((folder / name).read_text(), name), [], name)
        folder = ROOT / 'crates/agent/src'
        for name in ['worker.rs', 'incus.rs', 'inspection.rs']:
            self.assertEqual(MIGRATION.system_producer_issues((folder / name).read_text(), name), [], name)

    def test_rust_dynamic_errors_keep_parameters(self):
        source = 'Error::invalid(format!("Enter 1–{max} characters."))'
        migrated = MIGRATION.migrate_source(source, {'Enter 1–{max} characters.': 'text.enter'})
        self.assertEqual(migrated, 'Error::invalid(crate::system_message::SystemMessage::new("text.enter").with("max", max))')
        source = 'Error::conflict(format!("Remaining: {}", 2000 - used))'
        migrated = MIGRATION.migrate_source(source, {'Remaining: {}': 'text.remaining'})
        self.assertEqual(migrated, 'Error::conflict(crate::system_message::SystemMessage::new("text.remaining").with("0", 2000 - used))')

    def test_display_and_host_producer_literals_are_ids(self):
        contracts = json.loads((ROOT / 'locales/messages.json').read_text())
        sources = [*ROOT.glob('web/src/*.ts'), *ROOT.glob('web/src/*.tsx'), *ROOT.glob('plugins/**/src/main/java/**/*.java'), *ROOT.glob('crates/agent/src/**/*.rs')]
        for source in sources:
            if source.name in {'i18n.ts', 'Messages.java'}: continue
            text = source.read_text()
            for match in MIGRATION.localization_calls(text, source.name in {'LkjmcProxy.java', 'LkjmcPaper.java'}):
                fragment = text[match.end():MIGRATION.call_end(text, match.end())]
                args = MIGRATION.argument_spans(fragment)
                index = MIGRATION.template_index(match.group())
                if index >= len(args): continue
                left, right = args[index]
                expression = fragment[left:right].strip()
                literal = re.fullmatch(MIGRATION.LITERAL, expression)
                if literal:
                    key = json.loads(literal.group())
                    self.assertTrue(key in contracts, f'{source.relative_to(ROOT)}: {key}')

    def test_runtime_does_not_reverse_lookup_prose(self):
        for path in ['web/src/i18n.ts', 'plugins/common/src/main/java/com/lkjsxc/lkjmc/common/Messages.java']:
            source = (ROOT / path).read_text()
            self.assertNotIn('ERROR_KEYS', source)
            self.assertNotIn('errorMessages', source)
            self.assertNotRegex(source, r'Object\.entries\(japanese\)')
