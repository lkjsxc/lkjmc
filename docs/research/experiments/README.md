# Research experiments

## Purpose

This index holds falsifiable hypotheses; it does not authorize experimental
commands, adapters, downloads, or process management on the shared product.

## Table of contents

- [E-CONTRACT contract-source experiment](e-contract-20260711.md)
- [E-CONTROL execution comparison](e-control.md)
- [E-CONTROL disposable harness](e-control/README.md)
- [E-DATA durable workflow](e-data-20260711.md)
- [E-JVM isolated adapter comparison](e-jvm-20260711.md)
- [E-OBS observability candidates](e-obs.md)
- [E-OPS reproducibility hypothesis](e-ops-20260711.md)
- [E-PRODUCT bounded journey hypothesis](e-product-20260711.md)
- [E-QUALITY quality-technique hypothesis](e-quality-20260711.md)
- [E-RUNTIME lifecycle coordinators](e-runtime-20260711.md)
- [E-SECURITY credential experiment hypothesis](e-security-20260711.md)
- [Hypothesis template](hypothesis-template.md)

## Required evidence

A hypothesis names baseline evidence, a smallest real vertical slice, invariants,
workload, faults, measurements, and allowed worktree writes. Commit it before
experiment code. A pure-core slice needs a real-adapter integration run before
adoption.

## Decision boundary

A planned experiment is not shipped behavior. Use [runs](../runs/README.md) for
observations and [decisions](../decisions/README.md) for a reviewed disposition.
