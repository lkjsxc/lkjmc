# Shop catalog

## Purpose

This document defines the conservative default point shop catalog.

## Current status

The shop lists daemon items and delivers supported `minecraft-item` metadata.
The default catalog below is seeded by `lkjmc shop seed-defaults`, and core tests
validate that seeded buy prices exceed configured sell values.

## Sell rates

| Material | Points per item |
|---|---:|
| `COBBLESTONE` | 1 |
| `STONE` | 2 |
| `DIRT` | 1 |
| `GRAVEL` | 1 |
| `SAND` | 1 |
| `OAK_LOG` | 5 |
| `SPRUCE_LOG` | 5 |
| `COAL` | 8 |
| `COPPER_INGOT` | 10 |
| `REDSTONE` | 8 |

## Buy defaults

| Item id | Material | Amount | Price |
|---|---|---:|---:|
| `block-cobblestone-64` | `COBBLESTONE` | 64 | 96 |
| `block-stone-64` | `STONE` | 64 | 192 |
| `block-glass-32` | `GLASS` | 32 | 240 |
| `wood-oak-log-32` | `OAK_LOG` | 32 | 240 |
| `food-bread-16` | `BREAD` | 16 | 160 |
| `food-cooked-beef-16` | `COOKED_BEEF` | 16 | 420 |
| `food-golden-carrot-8` | `GOLDEN_CARROT` | 8 | 640 |
| `utility-torch-64` | `TORCH` | 64 | 256 |
| `utility-arrow-64` | `ARROW` | 64 | 384 |
| `utility-ender-pearl-4` | `ENDER_PEARL` | 4 | 1000 |
| `mineral-iron-ingot-8` | `IRON_INGOT` | 8 | 960 |
| `mineral-gold-ingot-8` | `GOLD_INGOT` | 8 | 1120 |
| `redstone-redstone-32` | `REDSTONE` | 32 | 384 |
| `redstone-repeater-8` | `REPEATER` | 8 | 640 |
| `decor-name-tag-1` | `NAME_TAG` | 1 | 1500 |
| `transport-saddle-1` | `SADDLE` | 1 | 1800 |

## Exclusions

Do not seed diamonds, netherite, elytra, shulker boxes, spawners, command-only
items, or progression-breaking items unless an owner explicitly configures them.
