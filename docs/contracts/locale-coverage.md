# Locale coverage

## Purpose

This document maps locale catalogs to checked documentation.

## Source owners

- Repository English config: `config/locales/en.json`.
- Repository Japanese config: `config/locales/ja.json`.

## Checked docs

- Catalog contract: [../product/i18n/catalog.md](../product/i18n/catalog.md).

## Rule

English and Japanese key sets must match in `config/locales/`. The JVM common
build bundles those same files into jar resources at build time. Player-visible
features must add both languages before the command, menu, or event feedback is
registered, and Java references to locale keys must resolve to committed keys.
