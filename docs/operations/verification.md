# Verification

## Purpose

This document defines current and target verification gates.

## Current gates

```sh
./scripts/check-lines.py
./scripts/check-docs.py
./scripts/verify.sh
```

Successful current output is exactly one success line per quiet check.

## Target compose gate

```sh
docker compose -f docker-compose.yml -f docker-compose.verify.yml run --rm verify
```

The compose gate is not implemented yet.
