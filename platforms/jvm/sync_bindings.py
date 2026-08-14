"""Generate sync records after checking their canonical Rust wire sources."""
import re
from binding_codegen import enum_source, record_source, require, write
from sync_schema import PAYLOAD_KEYS, PAYLOADS, RECORDS, VARIANTS

SNAPSHOTS = {name: payload.removesuffix("Payload") + "Snapshot"
             for name, payload in PAYLOADS.items()}


def _camel(name):
    parts = name.split("_")
    return parts[0] + "".join(part.title() for part in parts[1:])


def verify_canonical(root):
    payload = (root / "crates/lkjmc-store/src/sync/payload.rs").read_text(encoding="utf-8")
    domains = re.findall(r'^        "([a-z]+)" => [a-z]+\(client', payload, re.MULTILINE)
    require(set(domains) == set(PAYLOADS) and len(domains) == 6, "canonical sync domains drifted")
    literals = {"[]", "lkjmc-profile-one", "en"}
    for domain, expected in PAYLOAD_KEYS.items():
        section = re.search(rf"fn {domain}\(.*?(?=\nfn )", payload, re.DOTALL)
        require(section is not None, f"canonical {domain} payload missing")
        query = re.search(r'"select (.*?)"\s*,\s*&\[', section.group(0), re.DOTALL)
        require(query is not None, f"canonical {domain} query missing")
        found = {value for value in re.findall(r"'([A-Za-z][A-Za-z0-9-]*)'", query.group(1))
                 if value not in literals}
        require(found == expected, f"canonical {domain} fields drifted: {sorted(found ^ expected)}")
    transport = (root / "crates/lkjmc-daemon/src/transport/sync.rs").read_text(encoding="utf-8")
    found = set(re.findall(r'"([A-Za-z][A-Za-z0-9]*)"\s*:', transport))
    expected = {"result", "domain", "key", "revision", "generatedAt", "credentialRevision",
                "payload", "reason", "cursor", "activeFloor", "changes", "feedRevision",
                "error", "code"}
    require(found == expected, f"canonical sync result fields drifted: {sorted(found ^ expected)}")
    profile = (root / "crates/lkjmc-core/src/profile_envelope.rs").read_text(encoding="utf-8")
    body = re.search(r"pub struct ProfileEnvelope \{(.*?)\n\}", profile, re.DOTALL)
    require(body is not None, "canonical profile envelope missing")
    fields = {_camel(name) for name in re.findall(r"pub ([a-z_]+):", body.group(1))}
    require(fields == set(RECORDS["ProfileEnvelope"]), "canonical profile envelope fields drifted")


def generate(root, output, contract):
    verify_canonical(root)
    known = set(RECORDS) | set(VARIANTS) | {"GameMode"}
    for interface, variants in VARIANTS.items():
        parent = " extends DomainPayload" if interface in PAYLOADS.values() else ""
        write(output, interface, f"public sealed interface {interface}{parent} permits {', '.join(variants)} {{}}")
    direct = [payload for payload in PAYLOADS.values() if payload not in VARIANTS]
    write(output, "DomainPayload", "public sealed interface DomainPayload permits "
          + ", ".join(direct + list(VARIANTS)) + " {}")
    for name, fields in RECORDS.items():
        interface = next((owner for owner, variants in VARIANTS.items() if name in variants), "")
        if not interface and name in direct:
            interface = "DomainPayload"
        write(output, name, record_source(name, fields, known, interface))
    write(output, "GameMode", "import com.google.gson.annotations.SerializedName;\n\npublic enum GameMode {\n"
          + "    @SerializedName(\"survival\") SURVIVAL,\n"
          + "    @SerializedName(\"creative\") CREATIVE,\n"
          + "    @SerializedName(\"adventure\") ADVENTURE,\n"
          + "    @SerializedName(\"spectator\") SPECTATOR\n}")
    snapshots = list(SNAPSHOTS.values())
    write(output, "TypedSnapshot", "public sealed interface TypedSnapshot extends SyncResponse permits "
          + ", ".join(snapshots) + " {\n    String domain(); String key(); long revision();\n"
          + "    java.time.Instant generatedAt(); long credentialRevision(); DomainPayload payload();\n}")
    for domain, payload in PAYLOADS.items():
        fields = {"domain": "String", "key": "String", "revision": "long",
                  "generatedAt": "Instant", "credentialRevision": "long", "payload": payload}
        write(output, SNAPSHOTS[domain], record_source(SNAPSHOTS[domain], fields, known, "TypedSnapshot"))
    write(output, "SnapshotUnavailable", record_source("SnapshotUnavailable",
          {"domain": "String", "key": "String", "credentialRevision": "long", "reason": "String"},
          known, "SyncResponse"))
    write(output, "FeedResponse", record_source("FeedResponse", {"cursor": "long", "activeFloor": "long",
          "credentialRevision": "long", "changes": "List<FeedChange>"}, known, "SyncResponse"))
    write(output, "ReloadRequired", record_source("ReloadRequired", {"cursor": "long",
          "activeFloor": "long", "credentialRevision": "long"}, known, "SyncResponse"))
    write(output, "SyncUnavailable", record_source("SyncUnavailable", {"error": "SyncErrorBody"},
          known, "SyncResponse"))
    write(output, "SyncResponse", "public sealed interface SyncResponse permits TypedSnapshot, "
          + "SnapshotUnavailable, FeedResponse, ReloadRequired, SyncUnavailable {}")
    write(output, "SyncDomain", "public enum SyncDomain {\n    "
          + ",\n    ".join(name.upper() for name in PAYLOADS) + "\n}")
    requests = contract["requests"]
    write(output, "SyncRequest", "public sealed interface SyncRequest permits SnapshotRequest, FeedRequest {}")
    write(output, "SnapshotRequest", record_source("SnapshotRequest", requests["snapshot"], known, "SyncRequest"))
    write(output, "FeedRequest", record_source("FeedRequest", requests["feed"], known, "SyncRequest"))
