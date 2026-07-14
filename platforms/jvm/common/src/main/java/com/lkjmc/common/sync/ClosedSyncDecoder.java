package com.lkjmc.common.sync;

import com.google.gson.JsonObject;
import com.lkjmc.bindings.*;
import java.time.Instant;
import java.util.List;
import java.util.UUID;

public final class ClosedSyncDecoder {
    private final StrictRecordReader records = new StrictRecordReader();

    public SyncResponse decode(JsonObject body) {
        try {
            StrictRecordReader.require(body != null && body.has("result")
                    && body.get("result").isJsonPrimitive()
                    && body.get("result").getAsJsonPrimitive().isString());
            return switch (body.get("result").getAsString()) {
                case "snapshot" -> snapshot(body);
                case "changes" -> feed(body);
                case "reload-required" -> reload(body);
                case "unavailable" -> unavailable(body);
                default -> throw StrictRecordReader.invalid();
            };
        } catch (RuntimeException invalid) {
            if (invalid instanceof IllegalArgumentException argument
                    && "invalid sync response".equals(argument.getMessage())) throw argument;
            throw StrictRecordReader.invalid();
        }
    }

    private TypedSnapshot snapshot(JsonObject body) {
        StrictRecordReader.exact(body, "result", "domain", "key", "revision", "generatedAt",
                "credentialRevision", "payload");
        String domain = text(body, "domain");
        String key = text(body, "key");
        long revision = positive(body, "revision");
        long credential = positive(body, "credentialRevision");
        Instant generated = Instant.parse(text(body, "generatedAt"));
        var payload = body.get("payload");
        TypedSnapshot result = switch (domain) {
            case "permissions" -> new PermissionSnapshot(domain, key, revision, generated, credential,
                    records.read(payload, PermissionPayload.class));
            case "claims" -> new ClaimSnapshot(domain, key, revision, generated, credential,
                    records.read(payload, ClaimPayload.class));
            case "menus" -> new MenuSnapshot(domain, key, revision, generated, credential,
                    records.read(payload, MenuPayload.class));
            case "profiles" -> new ProfileSnapshot(domain, key, revision, generated, credential,
                    profile(payload.getAsJsonObject()));
            case "presence" -> new PresenceSnapshot(domain, key, revision, generated, credential,
                    presence(payload.getAsJsonObject()));
            case "routing" -> new RoutingSnapshot(domain, key, revision, generated, credential,
                    records.read(payload, RoutingPayload.class));
            case "settings" -> new SettingsSnapshot(domain, key, revision, generated, credential,
                    records.read(payload, SettingsPayload.class));
            default -> throw StrictRecordReader.invalid();
        };
        validateIdentity(result);
        return result;
    }

    private ProfilePayload profile(JsonObject payload) {
        if (payload.has("profile")) {
            StrictRecordReader.exact(payload, "playerUuid", "scope", "profile");
            StrictRecordReader.require(payload.get("profile").isJsonNull());
            return new ProfileMissing(UUID.fromString(text(payload, "playerUuid")), text(payload, "scope"));
        }
        ProfileAvailable value = records.read(payload, ProfileAvailable.class);
        StrictRecordReader.require("lkjmc-profile-one".equals(value.schema())
                && value.schema().equals(value.envelope().schema())
                && value.sha256().matches("[0-9a-fA-F]{64}") && value.profileRevision() > 0);
        return value;
    }

    private PresencePayload presence(JsonObject payload) {
        if (payload.has("available")) {
            StrictRecordReader.exact(payload, "instanceId", "available");
            StrictRecordReader.require(payload.get("available").isJsonPrimitive()
                    && payload.get("available").getAsJsonPrimitive().isBoolean()
                    && !payload.get("available").getAsBoolean());
            return new PresenceMissing(text(payload, "instanceId"));
        }
        return records.read(payload, PresenceAvailable.class);
    }

    private FeedResponse feed(JsonObject body) {
        StrictRecordReader.exact(body, "result", "cursor", "activeFloor", "credentialRevision", "changes");
        FeedResponse result = records.read(withoutResult(body), FeedResponse.class);
        StrictRecordReader.require(result.cursor() >= 0 && result.activeFloor() >= 0
                && result.credentialRevision() > 0);
        long previous = 0;
        for (FeedChange change : result.changes()) {
            StrictRecordReader.require(change.feedRevision() > previous
                    && change.feedRevision() <= result.cursor() && change.revision() > 0
                    && SyncKey.validDomain(change.domain()));
            previous = change.feedRevision();
        }
        return result;
    }

    private ReloadRequired reload(JsonObject body) {
        StrictRecordReader.exact(body, "result", "cursor", "activeFloor", "credentialRevision");
        ReloadRequired result = records.read(withoutResult(body), ReloadRequired.class);
        StrictRecordReader.require(result.cursor() >= 0 && result.activeFloor() >= 0
                && result.credentialRevision() > 0);
        return result;
    }

    private SyncResponse unavailable(JsonObject body) {
        if (body.has("error")) {
            StrictRecordReader.exact(body, "result", "error");
            return new SyncUnavailable(records.read(body.get("error"), SyncErrorBody.class));
        }
        StrictRecordReader.exact(body, "result", "domain", "key", "credentialRevision", "reason");
        return new SnapshotUnavailable(text(body, "domain"), text(body, "key"),
                positive(body, "credentialRevision"), text(body, "reason"));
    }

    private void validateIdentity(TypedSnapshot snapshot) {
        String expected = switch (snapshot.payload()) {
            case PermissionPayload value -> value.principalKind() + ":" + value.principalId();
            case ProfileAvailable value -> value.playerUuid() + ":" + value.scope();
            case ProfileMissing value -> value.playerUuid() + ":" + value.scope();
            case PresenceAvailable value -> value.instanceId();
            case PresenceMissing value -> value.instanceId();
            case SettingsPayload value -> value.playerUuid().toString();
            default -> snapshot.key();
        };
        StrictRecordReader.require(expected.equals(snapshot.key()));
    }

    private JsonObject withoutResult(JsonObject body) {
        JsonObject copy = body.deepCopy();
        copy.remove("result");
        return copy;
    }

    private String text(JsonObject body, String name) {
        StrictRecordReader.require(body.get(name).isJsonPrimitive()
                && body.get(name).getAsJsonPrimitive().isString()
                && !body.get(name).getAsString().isBlank());
        return body.get(name).getAsString();
    }

    private long positive(JsonObject body, String name) {
        long value = body.get(name).getAsLong();
        StrictRecordReader.require(value > 0 && body.get(name).getAsDouble() == value);
        return value;
    }
}
