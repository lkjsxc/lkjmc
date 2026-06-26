# Bootstrap planner

## Purpose

This target contract defines the pure planning model for playable bootstrap.

## Inputs

The planner receives a `BootstrapRequest`, a `DesiredNetwork`, and
`BootstrapFacts`. Facts include database availability, daemon HTTP readiness,
installed binaries, existing instances, known assets, port observations, and
filesystem observations.

## Output

The planner returns a `BootstrapPlan` with ordered effects, safe rollback
effects, diagnostics, and a typed outcome such as blocked, ready to apply, or
already converged.

## Rules

- Missing EULA acceptance blocks Paper and Folia start effects.
- PostgreSQL absence blocks playable bootstrap.
- Missing daemon HTTP token plans secure token-file generation.
- Missing Velocity forwarding secret plans secure secret generation.
- Missing server jars plan server asset sync effects.
- Missing `lkjmc` plugin jars plan build and asset registration effects.
- Unverified ViaVersion or ViaBackwards assets are withdrawn in auto mode.
- Unverified Geyser or Floodgate assets withdraw Bedrock in auto mode.
- Backend port conflicts allocate from the configured range and update configs.
- Managed instance drift is reconciled idempotently.
- Unmanaged directory conflicts block rather than overwrite.
- Plugin-only changes plan affected restarts without unrelated rewrites.

## Idempotency

A plan may be empty only when facts prove the desired network, assets, plugins,
secrets, ports, and running processes already match the desired state.
