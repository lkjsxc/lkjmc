# Catalog

## Purpose

This document defines localization catalog rules and verification.


## Status

implemented

## Paths

The repository has one committed source per language:

- `config/locales/en.json`
- `config/locales/ja.json`

The JVM common build copies these files into jar resources under `locales/` at
`processResources` time. Do not commit generated locale resource copies under
platform source trees.

## Rules

- English and Japanese key sets must match.
- Every locale value must be a string; nested objects are rejected.
- Player-visible features add English and Japanese messages in the same change.
- Japanese values must not leave English prose untranslated; `check-locales.py`
  permits only documented ASCII command labels, IDs, URLs, and decorative text.
- Wake-and-join, web, token rotation, Kubernetes diagnostics, and End Expedition
  shop copy use stable keys before code references them.
- Fallback chain is persisted player language, platform locale, network default,
  then English.
- Message keys are stable dotted identifiers.
- MiniMessage is used for Minecraft components when formatting is required.
- Every catalog value must parse with strict MiniMessage; escape literal angle
  brackets in examples such as command placeholders.
- Item names are rendered through the shared helper with `<!italic>` so custom
  names do not inherit Minecraft's default italic style.
- Localized sentence fragments are not concatenated.

## Source owners

Java common loads bundled catalogs through `MessageCatalog` with Gson parsing and
normalizes `en_US`, `en-US`, `ja_JP`, and `ja-JP` to supported language ids.
`MiniMessageText` owns strict Adventure MiniMessage rendering for component
surfaces. Product config catalogs are deployment defaults and the build-time jar
source.

## Verification

`scripts/check-locales.py` checks `config/locales/` key parity, Japanese ASCII
quality, and Java key references across common, Paper, and Velocity sources.
Java common tests verify bundled English and Japanese key parity, Gson parsing
behavior, and strict MiniMessage parsing for every bundled value.
