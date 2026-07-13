#!/usr/bin/env python3
import re

RUST_STRING = re.compile(
    r'r(?P<h>#{0,16})"(?P<raw>.*?)"(?P=h)|(?:b)?"(?P<quoted>(?:\\.|[^"\\])*)"',
    re.S,
)
SQL_WRITE = re.compile(
    r"\b(insert\s+into|update|delete\s+from)\s+([a-z_][a-z0-9_{}.]*)",
    re.I,
)
NETWORK_TYPE_CALL = re.compile(
    r"\b(?:TcpStream|TcpListener|TcpSocket|UdpSocket|UnixStream|UnixListener|"
    r"UnixDatagram|Socket)::\s*(?:accept|bind|connect|connect_timeout|from_std|listen|"
    r"new|new_v4|new_v6|pair)\b"
)
NETWORK_RECEIVER_CALL = re.compile(
    r"\b(?:listener|socket|stream|datagram|[a-z_][a-z0-9_]*_"
    r"(?:listener|socket|stream|datagram)[a-z0-9_]*)\s*\.\s*"
    r"(?:accept|bind|connect|listen|recv_from|send_to)\s*\(", re.I,
)
EFFECT_MARKERS = {
    "process": ("std::process::Command", "process::Command", "Command::new"),
    "filesystem": (
        "std::fs::", "fs::write", "fs::remove", "File::create", "OpenOptions",
    ),
}
NETWORK_MARKERS = ("reqwest::", "ureq::", "hyper::Client", "lookup_host(")


def without_rust_comments(source):
    pattern = re.compile(
        r'(?P<comment>/\*.*?\*/|//[^\n]*)|'
        r'(?P<string>r(?P<h>#{0,16})".*?"(?P=h)|(?:b)?"(?:\\.|[^"\\])*")',
        re.S,
    )
    return pattern.sub(
        lambda found: " " * len(found.group(0)) if found.group("comment") else found.group(0),
        source,
    )


def decode_rust_string(value):
    decoded, index = [], 0
    escapes = {"n": "\n", "r": "\r", "t": "\t", '"': '"', "\\": "\\"}
    while index < len(value):
        if value[index] != "\\" or index + 1 == len(value):
            decoded.append(value[index]); index += 1
            continue
        following = value[index + 1]
        if following == "\n":
            index += 2
            while index < len(value) and value[index] in " \t\r\n": index += 1
            continue
        decoded.append(escapes.get(following, following)); index += 2
    return "".join(decoded)


def mask_sql_noncode(sql):
    output = list(sql)
    index, block_depth = 0, 0
    while index < len(sql):
        if block_depth:
            if sql.startswith("/*", index): block_depth += 1; width = 2
            elif sql.startswith("*/", index): block_depth -= 1; width = 2
            else: width = 1
            for offset in range(width):
                if sql[index + offset] != "\n": output[index + offset] = " "
            index += width; continue
        if sql.startswith("--", index):
            end = sql.find("\n", index)
            end = len(sql) if end < 0 else end
            output[index:end] = " " * (end - index); index = end; continue
        if sql.startswith("/*", index):
            output[index:index + 2] = "  "; block_depth = 1; index += 2; continue
        if sql[index] in "'\"":
            quote, end = sql[index], index + 1
            while end < len(sql):
                if sql[end] == quote and end + 1 < len(sql) and sql[end + 1] == quote:
                    end += 2; continue
                if sql[end] == quote: end += 1; break
                end += 1
            for offset in range(index, end):
                if sql[offset] != "\n": output[offset] = " "
            index = end; continue
        dollar = re.match(r"\$[A-Za-z_0-9]*\$", sql[index:])
        if dollar:
            marker = dollar.group(0)
            end = sql.find(marker, index + len(marker))
            end = len(sql) if end < 0 else end + len(marker)
            for offset in range(index, end):
                if sql[offset] != "\n": output[offset] = " "
            index = end; continue
        index += 1
    return "".join(output)


def sql_writes(body):
    writes = []
    for literal in RUST_STRING.finditer(without_rust_comments(body)):
        value = literal.group("raw")
        if value is None: value = decode_rust_string(literal.group("quoted"))
        for statement in mask_sql_noncode(value).split(";"):
            stripped = statement.lstrip()
            if not re.match(r"(?i)(?:insert\b|update\b|delete\b|with\b)", stripped): continue
            found = SQL_WRITE.search(stripped)
            if found:
                verb, table = found.groups()
                kind = {"insert": "insert-into", "delete": "delete-from"}.get(
                    verb.lower().split()[0], "update"
                )
                writes.append(f"sql:{kind}:{table.lower()}")
    return writes


def effect_kinds(clean):
    effects = {
        kind for kind, markers in EFFECT_MARKERS.items() if any(marker in clean for marker in markers)
    }
    network = any(marker in clean for marker in NETWORK_MARKERS)
    network = network or bool(NETWORK_TYPE_CALL.search(clean) or NETWORK_RECEIVER_CALL.search(clean))
    if network: effects.add("network")
    return effects
