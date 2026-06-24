# Slot map

## Purpose

This document defines stable slot assignments for inventory menus.

## Rules

- Root menu uses slot `4` for info.
- Back is slot `49` in 54-slot menus.
- Previous page is slot `46`.
- Next page is slot `47`.
- Page info is slot `48`.
- Close or refresh is slot `50` where applicable.
- Decorative borders never overwrite functional slots.
- Language menu includes English and Japanese from the first implementation.

## Current status

Java common validates slot bounds, rejects duplicate functional slots, and tests
root, settings, language, confirmation, and pagination slot contracts. Paper
hotbar guardrails cover the initial menu entry item. Decorative border expansion
is still a later polish slice.
