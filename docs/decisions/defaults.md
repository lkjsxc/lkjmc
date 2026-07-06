# Default assumptions

## Purpose

This document records assumptions made because no interactive planning mode is
available.

## Assumptions

- The default install creates config and templates, then waits for explicit
  instance creation or start commands.
- Player sync includes inventory-like data and game mode by policy; location is
  disabled unless a profile opts in.
- Random teleport uses shipped profile definitions; availability is decided
  by the selected profile and its configured cost, cooldown, and bounds.
- In-game admins may manage instances only with explicit admin permissions.
- Custom servers use RCON when configured, otherwise stdin and process signals.
- Loopback HTTP for plugins is enabled only with a local token.
- The hotbar menu item is opt-in per player or template.
- Japanese messages use compact game UI language.
- The installer prefers native PostgreSQL packages on Ubuntu-like hosts.
- Real Minecraft smoke downloads require explicit environment flags.
