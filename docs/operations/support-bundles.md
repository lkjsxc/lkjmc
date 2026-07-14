# Support bundles

## Purpose

Define deterministic, bounded diagnostic collection without retaining or
printing secrets.

## Status

implemented

## Collection contract

`lkjmc support bundle --output PATH` requests the daemon to create an archive at
an operator-selected local path. The web support view can request and inspect
the same manifest but never renders archive contents or sensitive values.
Collection is allowlisted to:

- sanitized daemon status and readiness;
- bounded metric export;
- at most 500 recent structured events;
- explicitly named regular diagnostic files below configured roots.

Collection has byte, file, event, and one seven-second monotonic deadline shared
by status, readiness, metrics, PostgreSQL, file, redaction, hashing, archive,
sync, and publication phases. No phase starts without remaining budget;
PostgreSQL statement deadlines use that remainder. Collection cooperatively
stops between bounded reads and archive members. It rejects symlinks, FIFOs,
devices, other special files, traversal, and destinations outside the approved
lexical parent. Files are ordered by archive name. The manifest records schema
version, UTC creation time, local source, truncation facts, sizes, and SHA-256
hashes.

## Confidentiality

Redaction scans keys and text case-insensitively for bearer tokens,
authorization, cookies, CSRF, forwarding and RCON secrets, session and profile
fields, database and arbitrary URLs, passwords, and user-info credentials.
Known-value canaries cover every class. Redaction occurs before hashing or
archiving, and a final byte scan covers every retained member and the manifest.
A detected canary fails collection and removes the temporary archive.

Every destination-parent component must be a real directory, never a symlink,
and its canonical path must equal the approved normalized lexical parent. The
existing target must be absent; symlinks and non-regular targets fail closed.
The archive is created relative to a verified parent directory handle as a
private no-follow temporary regular file with mode `0600`, synced, published
atomically without overwrite by a same-directory hard link, and left private.
Generated credentials, redacted source values, and archive contents are never
printed.

## Failure and proof

Any read, cap, timeout, redaction, hash, sync, or publication error is a typed
non-success and removes temporary and partial output. Timeout returns within the
seven-second bound plus small scheduler tolerance. `support-bundle-pass` checks
stable member order, hashes, parent/target symlink attacks, FIFO rejection, slow
fault cleanup, and elapsed bounds; `secret-canary-pass` verifies known-positive
detection and a clean final archive. Guarded external logs remain absent unless
their adapter was really available.
