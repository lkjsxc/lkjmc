# Java compatibility

## Purpose

This contract defines optional Java protocol compatibility plugins.

## Default policy

ViaVersion and ViaBackwards are auto-enabled for Paper and Folia backends when
hash-verified compatible files are available from Modrinth. The default install
location is the backend.

## Dependency

ViaBackwards requires ViaVersion. If ViaVersion is unavailable or fails hash
verification, the planner withdraws both and reports a non-blocking diagnostic in
auto mode.

## Forwarding compatibility

The default network keeps Velocity modern forwarding. ProtocolSupport is not
installed with modern forwarding. Operators who need a conflicting forwarding
mode must use a future deliberate profile rather than mutating the playable
default.

## Status

implemented

## Bootstrap status

Bootstrap status reports each compatibility plugin as installed, withdrawn, or
not requested, including the target instance and diagnostic reason.
