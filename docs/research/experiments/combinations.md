# E-SYNTHESIS combination register

## Purpose

Assess all 31 required research interactions from candidate
`3a0aa47ce4e29d17656d2ba2973ea673e0788db6` without adopting an experiment or
product path. Each row has distinct named evidence, outcome, and hard-gate
conclusion; `BLOCKED` is never a capability pass.

## Method

The [evidence runner](../runs/e_synthesis_20260712_evidence.py) checks that
historic replay paths resolve at the source tip and executes each named `Run`
attempt. Its [raw manifest](../runs/e-synthesis-20260712-raw-manifest.json)
contains the per-row compatibility hashes and exact attempt exits. A replay
reference is a compatibility check, not a fresh execution or live claim.

## Assessments

| ID | Interaction | Distinct evidence attempt | Outcome and hard-gate conclusion |
| --- | --- | --- | --- |
| C01 | contract + consumers | E-CONTRACT replay; CT config run | REJECTED; real strict consumers remain required. |
| C02 | config + paths + reload | CT config and paths runs | REJECTED; no downstream reload observation. |
| C03 | catalog + width + aliases | CT index, width, alias runs | REJECTED; no migration compatibility proof. |
| C04 | network doc + desired state | CT network run; E-NETWORK replay | REJECTED; no durable controller loop. |
| C05 | executor + journal + effect | E-CONTROL replay | BLOCKED; idempotent post-launch effect gate. |
| C06 | reconcile event + observation | CP reconcile run; E-OBS replay | BLOCKED; attested observer event missing. |
| C07 | domain split + runtime | CP split run; E-RUNTIME replay | REJECTED; daemon ordering unproved. |
| C08 | workflow + journal + ack | E-DATA replay; JV ack guard | BLOCKED; attested nonblocking arrival missing. |
| C09 | adventure + notification | DW adventure and notify runs | REJECTED; no player workflow or freshness. |
| C10 | runtime history + observer | DW runtime; CP reconcile guards | BLOCKED; no independent provenance. |
| C11 | changelog + delta + retention | three DW local runs | REJECTED; no durable reconnect recovery. |
| C12 | audit integrity + retention | DW audit and retention runs | REJECTED; export/restore evidence missing. |
| C13 | pool fairness + clock | DW fairness and clock runs | REJECTED; no durable contention workload. |
| C14 | cache + invalidation + policy | E-SECURITY replay; DW notify run | REJECTED; no production ownership/freshness. |
| C15 | principal + transfer ack | SE principal; JV ack guards | BLOCKED; trusted session and arrival missing. |
| C16 | root retirement + provider | SE root and opaque-provider runs | REJECTED; no provider lifecycle boundary. |
| C17 | dependency + artifact manifest | SE dependency run; E-OPS replay | REJECTED; immutable acquisition remains open. |
| C18 | JVM repair + transfer | E-JVM replay; JV ack guard | BLOCKED; no adapter or client outcome. |
| C19 | Folia ownership + shutdown | two JV guarded attempts | BLOCKED; no platform runtime proof. |
| C20 | menu + protocol client | E-MENU replay; Folia guard | BLOCKED; resource is not client/scheduler proof. |
| C21 | plan + runtime + Kubernetes | E-NETWORK replay | BLOCKED; no authorized apply/observe/rollback. |
| C22 | network doc + controller | CT network; CP split runs | REJECTED; no desired/observed effect loop. |
| C23 | event + metric + history | E-OBS replay; DW runtime guard | BLOCKED; synthetic/local IDs are insufficient. |
| C24 | CI lanes + quality checks | OP CI run; E-QUALITY replay | REJECTED; local checks are not CI ownership. |
| C25 | cutover + restore + rollback | OP cutover guard; E-DATA replay | BLOCKED; PostgreSQL rollback drill absent. |
| C26 | incidents + support redaction | OP incidents guard; E-OBS replay | BLOCKED; authorized corpus absent. |
| C27 | plugin load + flake ban | QV plugin guard; flake run | BLOCKED; no client load harness. |
| C28 | clock + coverage + mutation | QV clock/map runs; E-QUALITY replay | REJECTED; metadata cannot replace owner proof. |
| C29 | Bedrock UX + degraded network | PX Bedrock and network guards | BLOCKED; no supported client recovery. |
| C30 | feature prune + adventure | PX corpus guard; DW adventure run | BLOCKED; no value corpus/player outcome. |
| C31 | principal + transfer + recovery | SE/JV/PX guarded attempts | BLOCKED; all three attestation gates compound. |

## Boundary

These are evidence attempts and interaction assessments, not required product
runs, adoption approvals, controller work, or a substitute for a live harness.
The [synthesis decision](../decisions/e-synthesis-20260712.md) retains the
resulting no-adoption and reconsideration boundaries.
