# Orchestration architecture

## Purpose

This area owns desired state, observed state, reconciliation, and process
runtime behavior.

## Table of contents

- [Desired state](desired-state.md)
- [Process runtime](process-runtime.md)

## Contract

Instance create writes desired state. The daemon reconciles desired state into
real process effects.
