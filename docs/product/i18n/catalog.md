# Catalog

## Purpose

This document defines localization catalog rules.

## Paths

- `config/locales/en.json`
- `config/locales/ja.json`
- `platforms/jvm/common/src/main/resources/locales/en.json`
- `platforms/jvm/common/src/main/resources/locales/ja.json`

## Rules

- Fallback chain is player locale, network default, then English.
- Missing keys fail verification once catalog checks exist.
- Message keys are stable dotted identifiers.
- MiniMessage is used for Minecraft components when formatting is required.
- Localized sentence fragments are not concatenated.

## Current status

English and Japanese JSON catalogs exist in repository config and JVM common
resources. Java common loads flat string catalogs, resolves player locale to
network default to English, renders placeholders, and tests that bundled English
and Japanese keys match.
