#!/usr/bin/env python3
import hashlib
import json
import socket
import struct
import sys
import time
import uuid


def encode_varint(value):
    out = bytearray()
    value &= 0xFFFFFFFF
    while True:
        byte = value & 0x7F
        value >>= 7
        if value:
            out.append(byte | 0x80)
        else:
            out.append(byte)
            return bytes(out)


def read_varint(read_byte):
    value = 0
    for shift in range(0, 35, 7):
        raw = read_byte()
        if not raw:
            raise EOFError("connection closed")
        byte = raw[0]
        value |= (byte & 0x7F) << shift
        if byte & 0x80 == 0:
            return value
    raise ValueError("varint too large")


def read_varint_from(data, offset=0):
    value = 0
    for shift in range(0, 35, 7):
        byte = data[offset]
        offset += 1
        value |= (byte & 0x7F) << shift
        if byte & 0x80 == 0:
            return value, offset
    raise ValueError("varint too large")


def read_exact(sock, size):
    data = bytearray()
    while len(data) < size:
        chunk = sock.recv(size - len(data))
        if not chunk:
            raise EOFError("connection closed")
        data.extend(chunk)
    return bytes(data)


def read_packet(sock, compression=False):
    size = read_varint(lambda: sock.recv(1))
    body = read_exact(sock, size)
    if compression:
        data_len, offset = read_varint_from(body)
        if data_len:
            import zlib
            body = zlib.decompress(body[offset:])
        else:
            body = body[offset:]
    packet_id, offset = read_varint_from(body)
    return packet_id, body[offset:]


def pack_string(value):
    raw = value.encode("utf-8")
    return encode_varint(len(raw)) + raw


def unpack_string(data):
    size, offset = read_varint_from(data)
    return data[offset:offset + size].decode("utf-8", "replace")


def send_packet(sock, packet_id, body=b""):
    payload = encode_varint(packet_id) + body
    sock.sendall(encode_varint(len(payload)) + payload)


def handshake(sock, host, port, protocol, state):
    body = (
        encode_varint(protocol)
        + pack_string(host)
        + struct.pack(">H", port)
        + encode_varint(state)
    )
    send_packet(sock, 0, body)


def open_socket(host, port):
    sock = socket.create_connection((host, port), timeout=10)
    sock.settimeout(10)
    return sock


def status(host, port):
    with open_socket(host, port) as sock:
        handshake(sock, host, port, 47, 1)
        send_packet(sock, 0)
        packet_id, body = read_packet(sock)
        if packet_id != 0:
            raise RuntimeError(f"unexpected status packet {packet_id}")
        doc = json.loads(unpack_string(body))
        print(doc["version"]["protocol"])


def offline_uuid(name):
    digest = bytearray(hashlib.md5(("OfflinePlayer:" + name).encode("utf-8")).digest())
    digest[6] = (digest[6] & 0x0F) | 0x30
    digest[8] = (digest[8] & 0x3F) | 0x80
    print(uuid.UUID(bytes=bytes(digest)))


def login_body(name, player_uuid, variant):
    if variant == "uuid":
        return pack_string(name) + uuid.UUID(player_uuid).bytes
    if variant == "name":
        return pack_string(name)
    if variant == "optional_uuid":
        return pack_string(name) + b"\x00\x01" + uuid.UUID(player_uuid).bytes
    if variant == "optional_none":
        return pack_string(name) + b"\x00\x00"
    raise ValueError(variant)


def login_start(host, port, protocol, name, player_uuid, variant):
    sock = open_socket(host, port)
    handshake(sock, host, port, protocol, 2)
    send_packet(sock, 0, login_body(name, player_uuid, variant))
    return sock


def attempt_login(host, port, protocol, name, player_uuid, mode, expected, variant):
    deadline = time.time() + 20
    compression = False
    with login_start(host, port, protocol, name, player_uuid, variant) as sock:
        while time.time() < deadline:
            packet_id, body = read_packet(sock, compression)
            if packet_id == 0:
                reason = unpack_string(body)
                if mode == "deny" and expected in reason:
                    print("ok banned login denied")
                    return True
                raise RuntimeError(f"login denied: {reason}")
            if packet_id == 3:
                compression = True
                continue
            if packet_id == 2:
                if mode == "accept":
                    print("ok login accepted")
                    return True
                raise RuntimeError("login was accepted")
    raise RuntimeError(f"login {mode} timed out")


def expect_login(host, port, protocol, name, player_uuid, mode, expected=""):
    failures = []
    variants = ("name",) if protocol < 759 else ("uuid", "name", "optional_uuid", "optional_none")
    for variant in variants:
        try:
            if attempt_login(host, port, protocol, name, player_uuid, mode, expected, variant):
                return
        except EOFError as error:
            failures.append(f"{variant}: {error}")
    raise RuntimeError("login attempts closed: " + "; ".join(failures))


def main():
    if len(sys.argv) < 2:
        raise SystemExit("usage: minecraft_login_probe.py status|offline-uuid|accept|deny ...")
    command = sys.argv[1]
    if command == "status" and len(sys.argv) == 4:
        status(sys.argv[2], int(sys.argv[3]))
    elif command == "offline-uuid" and len(sys.argv) == 3:
        offline_uuid(sys.argv[2])
    elif command == "accept" and len(sys.argv) == 7:
        expect_login(sys.argv[2], int(sys.argv[3]), int(sys.argv[4]), sys.argv[5], sys.argv[6], "accept")
    elif command == "deny" and len(sys.argv) == 8:
        expect_login(sys.argv[2], int(sys.argv[3]), int(sys.argv[4]), sys.argv[5], sys.argv[6], "deny", sys.argv[7])
    else:
        raise SystemExit("invalid arguments")


if __name__ == "__main__":
    main()
