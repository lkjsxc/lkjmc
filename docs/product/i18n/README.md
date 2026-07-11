# I18n

## Purpose

This area owns localization files, fallback behavior, and message completeness.


## Status

implemented

## Table of contents

- [Catalog](catalog.md)

## Contract

English and Japanese catalogs must be complete for each player-visible feature
introduced in the same change.

## Outcome, journey, and evidence boundary

A player receives catalog-backed copy in their persisted language, platform
locale, network default, or English fallback order. Missing or invalid catalog
data must fail checks or use the documented key fallback, never a newly invented
adapter sentence. Catalog parity and strict rendering checks prove bundled
resources; they do not prove a player's external client locale preference.
