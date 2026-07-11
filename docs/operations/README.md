# Operations

## Purpose

This area owns install, autosuspend, verification, quickstart, and smoke-check
operator contracts.


## Status

implemented

## Table of contents

- [Autosuspend](autosuspend.md)
- [Backup and restore](backup-restore.md)
- [Continuous integration](continuous-integration.md)
- [Daemon HTTP auth operations](daemon-http-auth.md)
- [Install](install.md)
- [Kubernetes runtime](kubernetes-runtime.md)
- [Lifecycle and recovery](lifecycle-recovery.md)
- [Quickstart](quickstart/README.md)
- [Release integrity](release-integrity.md)
- [Smoke checks](smoke-checks.md)
- [Web control](web-control.md)
- [Verification](verification.md)

## Contract

Operations must be idempotent and truthful. Checks must not report product
success for behavior that is not implemented.
