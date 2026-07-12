# High-cost research ideas

## Purpose

Imported costly alternatives. They are preserved as untested research, not
roadmap commitments or supported product capabilities.

## Catalog evidence

Source: supplied `experiments/catalog/high-cost.md`. Each needs a bounded real
slice where locally feasible and a retained rejection or adoption decision.

## Candidates

- `HC-EVENT-ALL` all-domain event sourcing; `HC-MULTI-DAEMON` active replicas.
- `HC-MULTI-REGION` distant database regions; `HC-KUBE-OPERATOR` custom resources.
- `HC-BROWSER-SPA` separate browser app; `HC-PLUGIN-SDK` public extensions.
- `HC-LANGUAGE-REWRITE` language replacement; `HC-EMBEDDED-STORE` embedded truth.
- `HC-MESSAGE-BROKER` external delivery broker; `HC-GRAPHQL` graph surface.
- `HC-WASM-RULES` sandboxed policies; `HC-AI-OPERATOR` automated remediation.
- `HC-PUBLIC-CONTROL` public API; `HC-MOBILE-CLIENT` mobile operator app.
- `HC-PREDICT-WAKE` predictive starts; `HC-WORLD-SERVICE` remote world storage.

## Bounded evidence

[E-HC-CONTROL](../decisions/e-hc-control-20260712.md) rejects its tested event,
lease, and broker paths; its Kubernetes attempt is external proof pending.
[E-HC-SURFACE](../decisions/e-hc-surface-20260712.md) rejects browser, graph,
extension, and public-control adoption; its mobile lane is blocked. [E-HC-
PLATFORM](../decisions/e-hc-platform-20260712.md) retains no adoption for the
language, embedded-store, and Wasm slices, while remote-world proof is blocked.
[E-HC-AUTOMATION](../decisions/e-hc-automation-20260712.md) rejects offline
operator and predictive-wake adoption; multi-region proof remains pending.

These are bounded, overlapping observations, not competing catalog-wide
selections. Any untested or externally blocked row remains no adoption; a
negative result in one slice does not assert current product behavior.

## High-cost and external prerequisites

A diagram is insufficient: execute one representative real path and measure
integration and operational cost. Cluster/operator runs need an authorized
throwaway namespace; public-control testing stays local; remote-world and
multi-region runs need controlled endpoints and network shaping; AI runs use
recorded incidents with no mutation authority. For unavailable access, record
the attempt, missing prerequisite, runnable harness, and exact rerun command.

## Decision boundary

Reject only with measured infrastructure, security, operational, latency, or
one-network-fit evidence. Reconsider event sourcing with a proven operation
journal need, brokers beyond PostgreSQL/HTTP bounds, and controllers beyond
measured adapter reconciliation limits.
