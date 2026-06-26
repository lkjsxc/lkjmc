package com.lkjmc.common.claim;

import com.google.gson.JsonObject;
import com.lkjmc.common.daemon.DaemonJson;
import java.util.Collection;
import java.util.HashSet;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;

public record ClaimSnapshot(Map<ClaimChunk, ClaimRecord> records) {
    public ClaimSnapshot {
        records = Map.copyOf(records == null ? Map.of() : records);
    }

    public static ClaimSnapshot empty() {
        return new ClaimSnapshot(Map.of());
    }

    public static ClaimSnapshot fromDaemonBody(JsonObject body) {
        var records = new ConcurrentHashMap<ClaimChunk, ClaimRecord>();
        DaemonJson.array(body, "chunks").ifPresent(chunks -> {
            for (var element : chunks) {
                if (!element.isJsonObject()) {
                    continue;
                }
                var object = element.getAsJsonObject();
                var chunk = new ClaimChunk(
                    DaemonJson.string(object, "instanceId").orElse("instance"),
                    DaemonJson.string(object, "worldName").orElse("world"),
                    DaemonJson.integer(object, "chunkX").orElse(0L).intValue(),
                    DaemonJson.integer(object, "chunkZ").orElse(0L).intValue()
                );
                records.put(chunk, record(object, chunk));
            }
        });
        return new ClaimSnapshot(records);
    }

    public Collection<ClaimRecord> all() {
        return records.values();
    }

    public Optional<ClaimRecord> at(ClaimChunk chunk) {
        return Optional.ofNullable(records.get(chunk));
    }

    public ClaimDecision decide(String playerUuid, boolean operator, ClaimChunk chunk) {
        if (operator) {
            return ClaimDecision.allow();
        }
        var claim = at(chunk);
        if (claim.isEmpty()) {
            return ClaimDecision.allow();
        }
        var record = claim.get();
        if (record.ownerUuid().equals(playerUuid) || record.trustedUuids().contains(playerUuid)) {
            return ClaimDecision.allow();
        }
        return ClaimDecision.deny(record);
    }

    public Optional<ClaimRecord> ownerClaimByName(String ownerUuid, String name) {
        var key = name == null ? "" : name.toLowerCase();
        return records.values().stream()
            .filter(record -> record.ownerUuid().equals(ownerUuid))
            .filter(record -> record.name().toLowerCase().equals(key))
            .findFirst();
    }

    private static ClaimRecord record(JsonObject object, ClaimChunk chunk) {
        return new ClaimRecord(
            DaemonJson.string(object, "claimId").orElse(""),
            DaemonJson.string(object, "ownerUuid").orElse(""),
            DaemonJson.string(object, "ownerName").orElse(""),
            DaemonJson.string(object, "name").orElse(""),
            chunk,
            trusts(object)
        );
    }

    private static Set<String> trusts(JsonObject object) {
        var values = new HashSet<String>();
        DaemonJson.array(object, "trusts").ifPresent(trusts -> {
            for (var item : trusts) {
                if (item.isJsonObject()) {
                    DaemonJson.string(item.getAsJsonObject(), "uuid").ifPresent(values::add);
                }
            }
        });
        return values;
    }
}
