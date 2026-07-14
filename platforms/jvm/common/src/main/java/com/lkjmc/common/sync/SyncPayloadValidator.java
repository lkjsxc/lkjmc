package com.lkjmc.common.sync;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import java.time.Instant;
import java.util.Set;
import java.util.UUID;
import java.util.function.Consumer;

public final class SyncPayloadValidator {
    private static final int MAX_ROWS = 10_000;
    private SyncPayloadValidator() {}

    public static boolean valid(SyncKey key, JsonElement payload) {
        try {
            JsonObject value = object(payload);
            switch (key.domain()) {
                case "permissions" -> permissions(key, value);
                case "claims" -> claims(value);
                case "menus" -> menus(value);
                case "profiles" -> profiles(key, value);
                case "presence" -> presence(key, value);
                case "routing" -> routing(value);
                case "settings" -> settings(key, value);
                default -> throw new IllegalArgumentException("unknown domain");
            }
            return true;
        } catch (RuntimeException invalid) {
            return false;
        }
    }

    private static void permissions(SyncKey key, JsonObject value) {
        exact(value, "principalKind", "principalId", "grants", "permissions");
        text(value, "principalKind", 1, 32);
        text(value, "principalId", 1, 256);
        require((value.get("principalKind").getAsString() + ":"
                + value.get("principalId").getAsString()).equals(key.key()));
        array(value, "grants", row -> {
            exact(row, "id", "roleId", "expiresAt");
            uuid(row, "id");
            identifier(row, "roleId");
            nullableInstant(row, "expiresAt");
        });
        strings(value, "permissions", 1, 128);
    }

    private static void claims(JsonObject value) {
        exact(value, "chunks");
        array(value, "chunks", row -> {
            exact(row, "claimId", "ownerUuid", "ownerName", "name", "worldName",
                    "chunkX", "chunkZ", "trusts");
            uuid(row, "claimId"); uuid(row, "ownerUuid");
            text(row, "ownerName", 1, 64); text(row, "name", 1, 64);
            text(row, "worldName", 1, 128); integer(row, "chunkX", -30_000_000, 30_000_000);
            integer(row, "chunkZ", -30_000_000, 30_000_000);
            array(row, "trusts", trust -> {
                exact(trust, "uuid", "name"); uuid(trust, "uuid"); text(trust, "name", 1, 64);
            });
        });
    }

    private static void menus(JsonObject value) {
        exact(value, "shop", "kits", "votes", "plugins");
        array(value, "shop", row -> {
            exact(row, "id", "titleKey", "pricePoints", "metadata");
            identifier(row, "id"); text(row, "titleKey", 1, 128);
            integer(row, "pricePoints", 0, Long.MAX_VALUE); object(row.get("metadata"));
        });
        array(value, "kits", row -> {
            exact(row, "id", "titleKey", "rewardPoints", "cooldownHours");
            identifier(row, "id"); text(row, "titleKey", 1, 128);
            integer(row, "rewardPoints", 0, Long.MAX_VALUE);
            integer(row, "cooldownHours", 0, Long.MAX_VALUE);
        });
        array(value, "votes", row -> {
            exact(row, "id", "titleKey", "url"); identifier(row, "id");
            text(row, "titleKey", 1, 128); text(row, "url", 1, 2048);
        });
        array(value, "plugins", row -> {
            exact(row, "id", "displayName", "platforms"); identifier(row, "id");
            text(row, "displayName", 1, 128); strings(row, "platforms", 1, 32);
        });
    }

    private static void profiles(SyncKey key, JsonObject value) {
        if (value.has("profile")) {
            exact(value, "playerUuid", "scope", "profile");
            require(value.get("profile").isJsonNull());
        } else {
            exact(value, "playerUuid", "scope", "profileRevision", "schema", "sha256", "envelope");
            integer(value, "profileRevision", 1, Long.MAX_VALUE);
            require("lkjmc-profile-one".equals(value.get("schema").getAsString()));
            require(value.get("sha256").getAsString().matches("[0-9A-Fa-f]{64}"));
            object(value.get("envelope"));
        }
        uuid(value, "playerUuid"); text(value, "scope", 1, 64);
        require(key.key().equals(value.get("playerUuid").getAsString() + ":"
                + value.get("scope").getAsString()));
    }

    private static void presence(SyncKey key, JsonObject value) {
        if (value.has("available")) {
            exact(value, "instanceId", "available");
            require(!value.get("available").getAsBoolean());
        } else {
            exact(value, "instanceId", "playerCount", "maxPlayers", "ready",
                    "lastHeartbeatAt", "suspendReason");
            nullableInteger(value, "playerCount", 0, 1_000_000);
            nullableInteger(value, "maxPlayers", 0, 1_000_000);
            require(value.get("ready").isJsonPrimitive()
                    && value.get("ready").getAsJsonPrimitive().isBoolean());
            Instant.parse(value.get("lastHeartbeatAt").getAsString());
            nullableText(value, "suspendReason", 256);
        }
        text(value, "instanceId", 1, 128);
        require(key.key().equals(value.get("instanceId").getAsString()));
    }

    private static void routing(JsonObject value) {
        exact(value, "instances");
        array(value, "instances", row -> {
            exact(row, "id", "kind", "desiredState", "observedState", "healthy", "ready",
                    "playerCount", "ports");
            identifier(row, "id"); identifier(row, "kind"); identifier(row, "desiredState");
            nullableText(row, "observedState", 64); nullableBoolean(row, "healthy");
            nullableBoolean(row, "ready"); nullableInteger(row, "playerCount", 0, 1_000_000);
            array(row, "ports", port -> {
                exact(port, "port", "purpose"); integer(port, "port", 1, 65535);
                text(port, "purpose", 1, 64);
            });
        });
    }

    private static void settings(SyncKey key, JsonObject value) {
        exact(value, "playerUuid", "language", "menuEnabled", "hudEnabled", "tipsEnabled", "privacy");
        uuid(value, "playerUuid"); require(key.key().equals(value.get("playerUuid").getAsString()));
        require(value.get("language").getAsString().matches("[a-z]{2}(?:-[A-Z]{2})?"));
        bool(value, "menuEnabled"); bool(value, "hudEnabled");
        bool(value, "tipsEnabled"); object(value.get("privacy"));
    }

    private static JsonObject object(JsonElement value) {
        require(value != null && value.isJsonObject()); return value.getAsJsonObject();
    }
    private static void exact(JsonObject value, String... names) {
        require(value.keySet().equals(Set.of(names)));
    }
    private static void array(JsonObject value, String name, Consumer<JsonObject> check) {
        JsonArray rows = value.getAsJsonArray(name); require(rows != null && rows.size() <= MAX_ROWS);
        rows.forEach(row -> check.accept(object(row)));
    }
    private static void strings(JsonObject value, String name, int min, int max) {
        JsonArray rows = value.getAsJsonArray(name); require(rows != null && rows.size() <= MAX_ROWS);
        rows.forEach(row -> { require(row.isJsonPrimitive() && row.getAsJsonPrimitive().isString());
            int size = row.getAsString().length(); require(size >= min && size <= max); });
    }
    private static void text(JsonObject value, String name, int min, int max) {
        JsonElement item = value.get(name); require(item != null && item.isJsonPrimitive()
                && item.getAsJsonPrimitive().isString());
        int size = item.getAsString().length(); require(size >= min && size <= max);
    }
    private static void nullableText(JsonObject value, String name, int max) {
        if (!value.get(name).isJsonNull()) text(value, name, 0, max);
    }
    private static void uuid(JsonObject value, String name) { UUID.fromString(value.get(name).getAsString()); }
    private static void identifier(JsonObject value, String name) {
        text(value, name, 1, 128); require(value.get(name).getAsString().matches("[A-Za-z0-9._:-]+"));
    }
    private static void integer(JsonObject value, String name, long min, long max) {
        JsonElement item = value.get(name); require(item.isJsonPrimitive() && item.getAsJsonPrimitive().isNumber());
        long number = item.getAsLong(); require(number >= min && number <= max
                && Double.isFinite(item.getAsDouble()) && item.getAsDouble() == number);
    }
    private static void nullableInteger(JsonObject value, String name, long min, long max) {
        if (!value.get(name).isJsonNull()) integer(value, name, min, max);
    }
    private static void bool(JsonObject value, String name) {
        JsonElement item = value.get(name);
        require(item.isJsonPrimitive() && item.getAsJsonPrimitive().isBoolean());
    }
    private static void nullableBoolean(JsonObject value, String name) {
        JsonElement item = value.get(name); require(item.isJsonNull()
                || item.isJsonPrimitive() && item.getAsJsonPrimitive().isBoolean());
    }
    private static void nullableInstant(JsonObject value, String name) {
        if (!value.get(name).isJsonNull()) Instant.parse(value.get(name).getAsString());
    }
    private static void require(boolean condition) { if (!condition) throw new IllegalArgumentException(); }
}
