package com.lkjmc.common.sync;

import com.google.gson.JsonParser;
import java.util.UUID;
import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class SyncPayloadValidatorTest {
    private static final UUID PLAYER = UUID.fromString("4a1f2b5c-2a1e-4c7a-8b6d-111111111111");
    private static final SyncKey KEY = new SyncKey("settings", PLAYER.toString());

    @Test
    void settings_require_typed_exact_bounded_payload() {
        assertTrue(valid("{\"playerUuid\":\"" + PLAYER + "\",\"language\":\"en\","
                + "\"menuEnabled\":true,\"hudEnabled\":false,\"tipsEnabled\":true,\"privacy\":{}}"));
        assertFalse(valid("\"settings\""));
        assertFalse(valid("{\"playerUuid\":\"" + PLAYER + "\",\"language\":\"en\"}"));
        assertFalse(valid("{\"playerUuid\":\"" + PLAYER + "\",\"language\":\"en\","
                + "\"menuEnabled\":true,\"hudEnabled\":false,\"tipsEnabled\":true,"
                + "\"privacy\":{},\"authority\":true}"));
        assertFalse(valid("{\"playerUuid\":\"" + PLAYER + "\",\"language\":\"invalid-language\","
                + "\"menuEnabled\":true,\"hudEnabled\":false,\"tipsEnabled\":true,\"privacy\":{}}"));
    }

    @Test
    void routing_rejects_missing_ports_and_invalid_bounds() {
        SyncKey routing = new SyncKey("routing", "network");
        assertTrue(SyncPayloadValidator.valid(routing, JsonParser.parseString(
                "{\"instances\":[{\"id\":\"hub\",\"kind\":\"paper\",\"desiredState\":\"running\","
                + "\"observedState\":null,\"healthy\":null,\"ready\":true,\"playerCount\":9,"
                + "\"ports\":[{\"port\":25565,\"purpose\":\"minecraft\"}]}]}")));
        assertFalse(SyncPayloadValidator.valid(routing, JsonParser.parseString(
                "{\"instances\":[{\"id\":\"hub\",\"kind\":\"paper\",\"desiredState\":\"running\","
                + "\"observedState\":null,\"healthy\":null,\"ready\":true,\"playerCount\":-1,\"ports\":[]}]}")));
    }

    private static boolean valid(String json) {
        return SyncPayloadValidator.valid(KEY, JsonParser.parseString(json));
    }
}
