# Operations

## Purpose

This area owns install, autosuspend, verification, quickstart, and smoke-check
operator contracts.

## Table of contents

- [Autosuspend](autosuspend.md)
- [Daemon HTTP auth operations](daemon-http-auth.md)
- [Install](install.md)
- [Kubernetes runtime](kubernetes-runtime.md)
- [Quickstart](quickstart/README.md)
- [Smoke checks](smoke-checks.md)
- [Web control](web-control.md)
- [Verification](verification.md)

## Contract

Operations must be idempotent and truthful. Checks must not report product
success for behavior that is not implemented.
