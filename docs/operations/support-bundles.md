# Support bundles

## Purpose

Define deterministic, bounded diagnostic collection without retaining or
printing secrets.

## Status

planned

## Collection contract

`lkjmc support bundle --output PATH` requests the daemon to create an archive at
an operator-selected local path. The web support view can request and inspect
the same manifest but never renders archive contents or sensitive values.
Collection is allowlisted to:

- sanitized daemon status and readiness;
- bounded metric export;
- at most 500 recent structured events;
- explicitly named regular diagnostic files below configured roots.

Collection has byte, file, event, and elapsed-time caps. It rejects symlinks,
special files, traversal, and destinations outside the requested parent. Files
are ordered by archive name. The manifest records schema version, UTC creation
time, local source, truncation facts, sizes, and SHA-256 hashes.

## Confidentiality

Redaction scans keys and text case-insensitively for bearer tokens,
authorization, cookies, CSRF, forwarding and RCON secrets, session and profile
fields, database and arbitrary URLs, passwords, and user-info credentials.
Known-value canaries cover every class. Redaction occurs before hashing or
archiving, and a final byte scan covers every retained member and the manifest.
A detected canary fails collection and removes the temporary archive.

The archive is written to a private temporary regular file with mode `0600`,
synced, renamed atomically in the destination directory, and left private.
Existing destinations are not overwritten. Generated credentials, redacted
source values, and archive member contents are never printed.

## Failure and proof

Any read, cap, timeout, redaction, hash, sync, or rename error is a typed
non-success and removes temporary output. `support-bundle-pass` checks stable
member order and hashes; `secret-canary-pass` verifies known-positive detection
and a clean final archive. Guarded external logs remain absent unless their
adapter was really available.
