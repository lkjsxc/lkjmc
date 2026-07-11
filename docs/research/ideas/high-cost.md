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
