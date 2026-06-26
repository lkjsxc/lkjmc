# Catalog

## Purpose

This document defines localization catalog rules and verification.

## Paths

- `config/locales/en.json`
- `config/locales/ja.json`
- `platforms/jvm/common/src/main/resources/locales/en.json`
- `platforms/jvm/common/src/main/resources/locales/ja.json`

## Rules

- English and Japanese leaf key sets must match in both repository config and
  JVM bundled resources.
- Player-visible features add English and Japanese messages in the same change.
- Fallback chain is player locale, network default, then English.
- Message keys are stable dotted identifiers.
- MiniMessage is used for Minecraft components when formatting is required.
- Localized sentence fragments are not concatenated.

## Source owners

Java common loads the bundled catalogs through `MessageCatalog`. Product config
catalogs are deployment defaults and must stay key-compatible with bundled
resources.

## Verification

`scripts/check-locales.py` compares the four JSON catalogs. Java common tests
also verify bundled English and Japanese key parity.
