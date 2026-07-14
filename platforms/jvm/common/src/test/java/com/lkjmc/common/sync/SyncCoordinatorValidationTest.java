package com.lkjmc.common.sync;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.net.URI;
import java.time.Duration;
import java.util.UUID;
import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

class SyncCoordinatorValidationTest {
    @Test
    void malformed_newer_responses_advance_neither_cache_nor_cursor() throws Exception {
        UUID player = UUID.fromString("4a1f2b5c-2a1e-4c7a-8b6d-111111111111");
        SyncKey key = new SyncKey("settings", player.toString());
        SyncConfig config = new SyncConfig(URI.create("http://127.0.0.1:9"), "test-credential",
                Duration.ofMillis(50), Duration.ofHours(1), Duration.ofMinutes(1),
                2, 1, 2, 100_000, 100_000);
        SyncCoordinator coordinator = new SyncCoordinator(config);
        coordinator.subscribe(key);
        try {
            coordinator.applySnapshot(key, snapshot(key, 1, settings(player, "en")));
            assertEquals(1, coordinator.view(key).orElseThrow().revision());
            JsonObject malformed = snapshot(key, 2, JsonParser.parseString("\"settings\"").getAsJsonPrimitive());
            assertThrows(IllegalArgumentException.class, () -> coordinator.applySnapshot(key, malformed));
            assertEquals(1, coordinator.view(key).orElseThrow().revision());
            JsonObject feed = JsonParser.parseString(
                    "{\"result\":\"changes\",\"cursor\":-1,\"activeFloor\":1,"
                    + "\"credentialRevision\":1,\"changes\":[]}").getAsJsonObject();
            assertThrows(IllegalArgumentException.class, () -> coordinator.applyFeed(feed));
            assertEquals(0, coordinator.checkpoint());
        } finally {
            coordinator.close();
            coordinator.awaitClosed(Duration.ofSeconds(1));
        }
    }

    private static JsonObject snapshot(SyncKey key, long revision, com.google.gson.JsonElement payload) {
        JsonObject body = new JsonObject();
        body.addProperty("result", "snapshot"); body.addProperty("domain", key.domain());
        body.addProperty("key", key.key()); body.addProperty("revision", revision);
        body.addProperty("generatedAt", "2026-07-14T00:00:00Z");
        body.addProperty("credentialRevision", 1); body.add("payload", payload);
        return body;
    }

    private static JsonObject settings(UUID player, String language) {
        return JsonParser.parseString("{\"playerUuid\":\"" + player + "\",\"language\":\""
                + language + "\",\"menuEnabled\":true,\"hudEnabled\":true,"
                + "\"tipsEnabled\":true,\"privacy\":{}}").getAsJsonObject();
    }
}
