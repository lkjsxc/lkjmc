# Locale coverage

## Purpose

This document maps locale catalogs to checked documentation.

## Source owners

- Repository English config: `config/locales/en.json`.
- Repository Japanese config: `config/locales/ja.json`.
- JVM English resources: `platforms/jvm/common/src/main/resources/locales/en.json`.
- JVM Japanese resources: `platforms/jvm/common/src/main/resources/locales/ja.json`.

## Checked docs

- Catalog contract: [../product/i18n/catalog.md](../product/i18n/catalog.md).

## Rule

English and Japanese leaf key sets must match. Player-visible features must add
both languages before the command, menu, or event feedback is registered.
