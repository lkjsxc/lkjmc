# Connection diagnostics

## Purpose

This contract defines structured diagnostics for Java hostname entry.

## Diagnostic path

A diagnostic run for `HOST` and `PORT` must decompose the client path:

1. Resolve `_minecraft._tcp.HOST` SRV records.
2. Resolve A and AAAA records for `HOST` and any SRV target.
3. Select the effective Minecraft target: SRV target and port when SRV exists,
   otherwise `HOST:PORT`.
4. Compare resolved addresses with an expected LAN address when configured.
5. Check TCP reachability for the effective target and safe fallback targets.
6. Run a Java status ping using the original hostname in the handshake.
7. When configured, run the same ping against a direct IP comparison target.
8. Classify split-horizon mistakes, SRV target drift, missing target address
   records, different SRV ports, IPv6-only failure, loopback-only proxy binds,
   backend unavailability, and missing Velocity backend routes.
9. Emit exact next actions.

## CLI surfaces

```sh
lkjmc network diagnose lkjsxc.com --port 25565
lkjmc network diagnose lkjsxc.com --json
lkjmc bootstrap doctor --host lkjsxc.com
lkjmc bootstrap status --json
```

Human output must be compact and actionable. JSON output must include the input
host, configured port, SRV result, resolved addresses, selected target, TCP
checks, status ping result, comparison target when present, classifications, and
next actions.

## Severity

- `ok`: the step succeeded.
- `info`: the step is optional or useful context, such as no SRV on port
  `25565`.
- `warning`: the hostname path differs from direct IP but may still work.
- `blocking`: the selected target is unreachable or cannot be resolved.

## Common remediation

- A record points to the wrong LAN IP: update local DNS for the public host.
- SRV points elsewhere: update or remove `_minecraft._tcp.<host>`.
- SRV target has no address record: add A or AAAA for the target.
- IPv6 fails while IPv4 works: fix the AAAA route or remove the bad AAAA.
- TCP works but status ping fails: inspect Velocity bind, forwarding, and logs.
- Host route exists for a missing backend: fix `forced-hosts` or the backend id.
