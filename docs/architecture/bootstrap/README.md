# Bootstrap architecture

## Purpose

This directory owns the target architecture for turning a clean installation
into a playable managed network.

## Table of contents

- [Desired network](desired-network.md)
- [Effects](effects.md)
- [Planner](planner.md)
- [Rollback](rollback.md)

## Contract

Bootstrap decisions belong in pure Rust core. Daemon adapters gather facts,
apply effects, record steps, and report diagnostics without faking completed
work.
