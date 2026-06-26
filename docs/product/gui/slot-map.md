# Slot map

## Purpose

This document defines stable slot assignments for inventory menus.

## Rules

- Root menu uses slot `4` for info.
- Root category entries use `19..25`.
- Back is slot `49` in 54-slot menus.
- Previous page is slot `46`.
- Next page is slot `47`.
- Page info is slot `48`.
- Close or refresh is slot `50` where applicable.
- Decorative borders never overwrite functional slots.
- Language menu includes English and Japanese from the first implementation.

## Border slots

Default 54-slot border slots are top row `0..8`, bottom row `45..53` except
functional controls, left column `9,18,27,36`, and right column `17,26,35,44`.
Functional slots win over border slots.

## Target status

Java common validates slot bounds, rejects duplicate functional slots, expands
inert borders, and tests root, settings, language, confirmation, and pagination
slot contracts. Paper hotbar guardrails use player hotbar index `8`, not raw
view slot `8`.
