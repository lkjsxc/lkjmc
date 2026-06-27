# Menu tree

## Purpose

This contract defines the reachable player menu hierarchy.

## Root layout

- Slot `4`: network and player info.
- Slot `19`: Network and servers.
- Slot `20`: Travel.
- Slot `21`: Claims.
- Slot `22`: Economy.
- Slot `23`: Social.
- Slot `24`: Profile and progression.
- Slot `25`: Settings.
- Slot `31`: Staff tools when permitted; otherwise empty or disabled by owner.
- Slot `40`: Temporary adventures only as a disabled future item until live.
- Slot `50`: Close.

## Required surfaces

Root links to network, travel, claims, economy, social, profile, and settings.
Dynamic child surfaces cover servers, homes, warps, teleport requests, claims,
shop, kits, daily reward, votes, mail, reports, language, and personal settings
when daemon-backed data and actions exist.

## Navigation

Back is route-based. Non-root menus preserve a route stack and return to the
previous route, not always root. Dense lists use stable ordering, deterministic
pagination, and explicit refresh.

## Future adventures

Temporary adventures may appear only as a disabled item with an exact inactive
reason until daemon-managed temporary instances, purchase atomicity, transfer,
and cleanup work end to end.
