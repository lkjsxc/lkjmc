# Third-party plugin policy

## Purpose

This target contract defines default decisions for ViaVersion, ViaBackwards,
Geyser, and Floodgate.

## Java compatibility

ViaVersion and ViaBackwards are enabled in auto mode for Paper and Folia
backends when Modrinth provides hash-verified compatible files. ViaBackwards
requires ViaVersion. If ViaVersion cannot be verified, both are withdrawn and
Java play continues without compatibility plugins.

The default install location is the backend because upstream guidance favors
backend installation for compatibility. ProtocolSupport is not installed with
modern Velocity forwarding.

## Bedrock entry

Geyser and Floodgate are enabled in auto mode for the Velocity proxy when the
GeyserMC API provides hash-verified files and UDP `19132` can be bound or
published. Geyser uses Floodgate auth only when Floodgate is installed and its
key material exists.

## Floodgate keys

Floodgate `key.pem` is never logged, committed, or copied broadly. If backend
API access is requested, key copy is a deliberate private backend effect with
restrictive permissions. If the key appears only after first plugin boot,
bootstrap reports a multi-phase status rather than pretending the key exists.

## Withdrawal rules

Auto-mode third-party features may withdraw with non-blocking diagnostics. An
operator-requested enabled feature may block when safety or verification fails.
