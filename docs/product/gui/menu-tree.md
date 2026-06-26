# Menu tree

## Purpose

This contract defines the reachable player menu hierarchy.

## Root layout

- Slot `4`: network and player info panel.
- Slot `19`: Network and servers.
- Slot `20`: Travel.
- Slot `21`: Claims.
- Slot `22`: Economy.
- Slot `23`: Social.
- Slot `24`: Profile.
- Slot `25`: Settings.
- Slot `49`: inert on root.
- Slot `50`: Close.

## Required menus

- Root.
- Network status and servers.
- Player profile summary.
- Homes.
- Warps.
- Teleport requests.
- Claims.
- Economy and points.
- Shop list and shop detail.
- Kits.
- Daily reward.
- Votes.
- Mail.
- Reports for staff when permitted.
- Settings.
- Language selector.
- Confirmation pages for destructive actions.

## Navigation

Every non-root menu has a deterministic back path toward root. Dense lists use
stable ordering and pagination. Picker menus may expose manual refresh. Menus
refresh after state changes, but background reopen loops are disallowed.

## Disabled features

If a command-backed feature cannot be represented as a GUI action, it renders a
disabled item with an exact reason. It must not register a fake click action.
