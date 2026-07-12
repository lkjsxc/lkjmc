# E-HC-SURFACE research decision 2026-07-12

## Purpose

Preserve the candidate dispositions from a local safety spike. This decision
adopts no browser application, graph layer, extension SDK, public endpoint, or
mobile API.

## Evidence and review status

Baseline owner evidence is the private `/web` adapter and daemon transport.
The variant is the [hypothesis](../experiments/e-hc-surface-20260712.md),
local harness `acd5e38`, and [run](../runs/e-hc-surface-20260712.md), based on
`4b9357a8e1a7949e0ebfe59c16af5196554f46cc`. No independent reviewer or
external system was available for this narrow local run. The summary proves
only 12 local HTTP assertions, a mixed browser result, and a blocked mobile
gate. Firefox completed the required POST-and-response semantic observation;
Chrome did not. It is not product, public-network, cross-browser, or
mobile-client proof.

## Dispositions

| Candidate | Disposition | Evidence-backed reason |
| --- | --- | --- |
| `HC-BROWSER-SPA` | reject | Firefox semantic evidence passed, Chrome is blocked, so the aggregate is `MIXED`, not cross-browser support. It also duplicates private presentation without session or workflow proof. |
| `HC-GRAPHQL` | reject | The constrained graph model resolved two fixed projections and denied other inputs. It is not a GraphQL service, has no unique need, and adds parser, authorization, and cost-control work. |
| `HC-PLUGIN-SDK` | reject | `exec_module` ran `third.party.status` with host authority. That unsandboxed extension execution is explicitly rejected; shape validation is not isolation, signing, revocation, or compatibility. |
| `HC-PUBLIC-CONTROL` | reject | The 403, 413, and zero-mutation 405 results are local denial threat-test observations only, not a safe public-control basis. |
| `HC-MOBILE-CLIENT` | blocked | The mobile-neutral endpoint stayed unregistered. The missing-evidence attempt exited 2; no reviewed unique consumer requirement exists beyond the graph projection. |

## Combination findings

The local browser task has Firefox-only semantic proof and a `MIXED` aggregate;
Chrome remains blocked. Even a cross-browser pass would leave a second client,
protocol, and authorization boundary without unique value. The loaded extension makes it
worse because unsandboxed `exec_module` code runs with host authority before a
projection can be validated. The fail-closed public path supports only
non-adoption; opening it would invalidate the zero-mutation observation.

Keep the current private authenticated `/web` owner contract unchanged. Do not
route any result into the daemon, controller, state matrix, configuration,
plugin, browser asset bundle, mobile client, or deployment. No owner or state
document changes are warranted for this research-only decision.

## Reconsideration and next step

Reconsider browser or graph work only with a measured, distinct operator task,
independent security review, session/CSRF/authorization evidence, accessibility
and browser support evidence, and a proven advantage over private `/web`.
Reconsider extensions only after an independently evaluated isolation,
capability, provenance, lifecycle, revocation, and compatibility design.

Reconsider public control only after a separate approved front-door threat
model and controlled external test; this decision authorizes neither. Reconsider
mobile only with reviewed evidence of a unique mobile capability, a real client
or neutral-consumer harness, contract/versioning proof, and privacy/security
review. The next executable step is an independent verifier rerunning
`harness.py --self-test` in a fresh owned root, then repeating the missing-file
mobile command before any synthesis discussion.
