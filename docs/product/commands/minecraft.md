# Minecraft commands

## Paper/Folia

`/menu` and `/docs` are backend-local presentation paths. They do not own desired state,
routing, PostgreSQL mutation, or proxy transfers.

## Velocity

Velocity owns `/lkjmc`:

- `/lkjmc` prints the supported forms;
- `/lkjmc status` runs bounded asynchronous pings for every configured backend;
- `/lkjmc server <instance-id>` requests the player's transfer through Velocity;
- completion offers the current typed backend IDs, not a source-coded list.

Unknown IDs, non-player sources, missing registrations, timeouts, and platform connection outcomes
produce distinct feedback. A successful connection-request future is not evidence that an
independent observer saw the same player arrive.

No per-backend shortcut, generic daemon command, or admin/economy/gameplay tree is part of this
supported proxy surface.

## Evidence boundary

Rust/Java tests prove parsing, dynamic completion, bounded callbacks, and connection-request
classification. Actual command text, completion, status, and transfer still require an authorized
online-mode player and remain outside current player evidence.
