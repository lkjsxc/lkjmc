# E-SYNTHESIS gap-evidence run 2026-07-12

## Purpose

Record bounded evidence attempts for every former `no ` inventory row at source
candidate `3a0aa47ce4e29d17656d2ba2973ea673e0788db6`. This is research-only: an
exit-zero local fixture is not a capability pass or product adoption.

## Inputs and limits

The runner hashes the candidate's eight catalog-source files per attempted ID.
It owns only `tmp/e-synthesis-20260712/<ID>` and writes no product path, config,
controller record, secret, client request, cluster request, or adapter action.
The committed [raw manifest](e-synthesis-20260712-raw-manifest.json) records
ID, command, exit, source path/hash, result, and guarded rerun command.

## Command and result

```sh
python3 docs/research/runs/e_synthesis_20260712_evidence.py --write
python3 docs/research/runs/e_synthesis_20260712_evidence.py --check
```

The first command exited `0`: 34 attempts were recorded, with 22 safe local
fixtures exiting `0` and 12 guarded external attempts exiting `2` for missing
prerequisites. The check command exited `0` against the committed manifest.
External guard exit `2` is `BLOCKED`, never a client, Bedrock, attestation, or
Kubernetes capability result. If every guard is supplied, an individual rerun
exits `3` as `REJECTED`; this docs-only harness intentionally starts no external
client.

## Individual rerun

Run one exact command from that ID's manifest record, for example:

```sh
python3 docs/research/runs/e_synthesis_20260712_evidence.py --id CT-CONFIG-DIR
python3 docs/research/runs/e_synthesis_20260712_evidence.py --id PX-BEDROCK-UX
```

The latter records its guard failure without reading or printing any endpoint or
credential value. The complete interaction assessment is the 31-row
[combination register](../experiments/combinations.md).

## Boundary

The run rejects insufficient local evidence or records a blocked prerequisite;
it does not select a controller, workflow, Java adapter, client, secret
provider, or operational process. A future owner proposal must use its own
contract and approved live harness before any reconsideration.
