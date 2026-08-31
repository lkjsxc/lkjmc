# Domain entry

## Purpose

This contract defines how Java players enter the network by a hostname such as
`lkjsxc.com` and how operators diagnose differences between hostname and direct
IP entry.


## Status

implemented

## Address concepts

- Bind host: the local interface Velocity listens on, commonly `0.0.0.0`.
- Java port: the TCP port Velocity listens on, commonly `25565`.
- Public hosts: hostnames players type into the Java client.
- Preferred public host: the hostname shown in status and
  player-facing diagnostics.
- Public socket display: the preferred public host with the Java port, or a
  local-only fallback when no public host is configured.
- SRV expectation: optional operator intent for `_minecraft._tcp.<host>` target
  and port.
- Host routing policy: optional Velocity `forced-hosts` entries for declared
  public hosts when host-specific backend routing is desired.

## Default behavior

Direct IP entry remains allowed by default. Public host routing may route known
hostnames to the fallback backend, but it must not deny direct-IP comparison
unless a future owner doc explicitly adds a deny policy.

When a public host resolves to the same machine as a working direct IP, Java
entry should work through the public host. If it does not, diagnostics must show
where hostname entry diverges from direct IP entry.

## DNS shape

For a server on the default Java port, this local DNS data is sufficient:

```text
lkjsxc.com.  A    192.168.1.2
```

An explicit SRV record is also valid and recommended for clarity:

```text
_minecraft._tcp.lkjsxc.com. SRV 0 0 25565 lkjsxc.com.
```

If no SRV exists and the port is `25565`, absence of SRV is informational, not a
failure. If SRV exists, clients use its target and port.

## Operator output

Bootstrap status and bootstrap apply must render
the effective public socket. With `lkjsxc.com` configured for port `25565`, the
compact Java line is:

```text
java: lkjsxc.com:25565
```

They must not print `127.0.0.1` when a public host is configured.
