# E-CONTRACT bounded run 2026-07-11

## Purpose

Record a reproducible isolated comparison of contract sources without changing a
product contract, consumer, JVM adapter, or Discord registration.

## Identity

- Experiment: [E-CONTRACT hypothesis](../experiments/e-contract-20260711.md).
- Run: `E-CONTRACT-RUN-20260711`; task base: `d20e5e5`.
- Candidate commit: `a6dcf8eef4cc539ff6b50ecab6a5b2fea77ce2d6`.
- Worktree: `/tmp/pi-agent-535f3a0b-ed13-405-0b4a8021` (isolated detached HEAD).
- Result: this run records no product result commit; the following decision is
  a research-only commit.

## Environment and command

Host was Linux with Python 3.12.3, rustc 1.96.0, and javac 21.0.11. The locked
workspace dependencies supplied the daemon and Discord tests. No database,
network, process-managed server, JVM plugin, or external service was started.

```sh
REPO=/tmp/pi-agent-535f3a0b-ed13-405-0b4a8021
OUT=/tmp/lkjmc-e-contract-d20e5e5
rm -rf "$OUT"
cd /tmp
python3 "$REPO/docs/research/experiments/e-contract-20260711.py" --output "$OUT"
```

The command exited `0`. It used fixed seed `20260711`, one valid body plus an
unknown-field and wrong-type fault per command, and three shard repetitions.
The harness compiles the isolated Rust descriptor and generated Rust/Java files;
it does not generate into a product source root.

## Results

| Probe or observation | Result | Evidence |
| --- | --- | --- |
| three contract candidates | PASS | hand, Rust descriptor, and neutral shapes agreed and rejected faults |
| payload drift | PASS | current generic schema accepts both fault classes; all candidates reject them |
| shard repeatability | PASS | 15 files; three identical `be00a664…0316` tree hashes |
| generated bindings | PASS | stale mutation differs; rustc test and javac 21 compilation exit `0` |
| handler coverage | PASS | 137 registry names equal 137 registrations; daemon test passed |
| menu schema boundary | PASS, bounded | four static local routes validate; no runtime menu consumption claim |
| all applicable surfaces | BLOCKED | transfer and purchase lack CLI/web mappings; Java and Discord remain withdrawn |
| combinations | PASS, non-adoption | five requested source/shard/adapter/error/menu combinations ran |

Raw output remains at `/tmp/lkjmc-e-contract-d20e5e5/`; `result.json` SHA-256
is `e91443731cd4e0ef6525198e3a8617c5ef21703e273749587c4c04398a0a5997`.
It contains exact commands, exit codes, logs, source coverage, generated hashes,
and withdrawal checks. `jvm-containment.log` and `discord-withdrawal.log` both
record exit `0`; the latter ran four withdrawal tests.

## Faults, cleanup, and deviations

The stale generated Java file was deliberately altered only under `OUT` and its
hash mismatch was detected. The generic request schema was inspected as an
object with neither `properties` nor `additionalProperties`; it accepts the two
fault classes by JSON Schema defaults. No secret input existed. The raw root is
retained for replay; no service or external resource needs cleanup. Git status
was clean before the run and no product path was written.

## Interpretation boundary

These are experiment observations. The [decision](../decisions/e-contract-20260711.md)
selects no product implementation and does not weaken F-SAFETY.
