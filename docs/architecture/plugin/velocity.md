# Velocity plugin

## Purpose

Velocity owns authenticated proxy player identity, the small network command surface, and the final server-connection effect.

## Implemented responsibilities

The plugin provides:

- the proxy MOTD and tab-list presentation;
- `/lkjmc` help;
- asynchronous `/lkjmc status` pings for the configured `hub` and `survival` registrations;
- Brigadier completion for `status`, `server`, `hub`, and `survival`;
- `/lkjmc server <hub|survival>` through Velocity's connection-request API; and
- one Java-common read-only sync coordinator when a scoped credential is configured.

The command callback never waits for a ping, database, network response, or transfer. Status probes have a three-second deadline and eight-request admission bound. Transfers have a five-second deadline and a 32-request admission bound. A timeout releases player feedback but retains its admission slot until the underlying Velocity future actually settles, so abandoned network work cannot exceed those bounds. Completion exposes only the two current network targets. Velocity reports successful, already-connected, in-progress, cancelled, disconnected, timeout, unregistered, and invalid-target outcomes distinctly without claiming arrival before the platform connection future completes.

Player identity comes from Velocity's `Player`; production runs in online mode with modern forwarding. The command performs no daemon mutation and the only effect is an authenticated player's own connection request to one fixed local backend registration.

## Withdrawn responsibilities

`/hub`, arbitrary send and wake commands, profile saves or application, moderation decisions, and dynamic server registration remain withdrawn. Cached routing and grant views are not proxy authority. The older unattested workflow transfer adapter is not a command caller and must not be exposed as successful behavior.

## Lifecycle and verification

The command and both listeners are registered once per runtime and unregistered on replacement or shutdown. Closing the command suppresses late callback feedback while in-flight platform futures release their admission permits.

Focused Gradle tests execute the actual Brigadier tree, completion, both registered-server status probes, every Velocity transfer status, exceptional failure, console denial, invalid-target denial, pending-future return, timeout feedback, ninth/33rd admission denial, underlying-settlement permit retention, close suppression, and 100 lifecycle replacement cycles. Containment keeps the reviewed Velocity source set explicit and rejects the withdrawn generic command/daemon client frameworks.

## Forwarding target

The production proxy uses online mode and modern player information forwarding with a private `forwarding.secret` file. Backend listeners remain loopback-only.
