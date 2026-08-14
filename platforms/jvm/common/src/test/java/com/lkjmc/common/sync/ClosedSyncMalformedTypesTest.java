package com.lkjmc.common.sync;

import static org.junit.jupiter.api.Assertions.assertThrows;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;

final class ClosedSyncMalformedTypesTest {
    private static final String PLAYER = "4a1f2b5c-2a1e-4c7a-8b6d-111111111111";
    private static final List<String> ENVELOPE = List.of(
            "result", "domain", "key", "revision", "generatedAt", "credentialRevision", "payload");
    private final ClosedSyncDecoder decoder = new ClosedSyncDecoder();

    @Test
    void everyEnvelopeFieldAndSixDomainPayloadsRejectWrongJsonKinds() {
        domains().forEach((domain, encoded) -> {
            JsonObject valid = snapshot(domain, encoded[0], encoded[1]);
            for (String field : ENVELOPE) {
                JsonObject malformed = valid.deepCopy();
                malformed.add(field, wrong(malformed.get(field)));
                assertThrows(IllegalArgumentException.class, () -> decoder.decode(malformed),
                        domain + " envelope " + field);
            }
            JsonObject payload = valid.getAsJsonObject("payload");
            for (String field : List.copyOf(payload.keySet())) {
                JsonObject malformed = valid.deepCopy();
                JsonObject changed = malformed.getAsJsonObject("payload");
                changed.add(field, wrong(changed.get(field)));
                assertThrows(IllegalArgumentException.class, () -> decoder.decode(malformed),
                        domain + " payload " + field);
            }
        });
    }

    @Test
    void revisionAndCursorNumbersAreIntegralNonnegativeAndInRange() {
        JsonObject valid = snapshot("settings", PLAYER, domains().get("settings")[1]);
        for (String invalid : List.of("\"2\"", "true", "2.5", "0", "-1",
                "9223372036854775808")) {
            JsonObject malformed = valid.deepCopy();
            malformed.add("revision", JsonParser.parseString(invalid));
            assertThrows(IllegalArgumentException.class, () -> decoder.decode(malformed), invalid);
        }
        String feed = "{\"result\":\"changes\",\"cursor\":%s,\"activeFloor\":0,"
                + "\"credentialRevision\":1,\"changes\":[]}";
        for (String invalid : List.of("\"2\"", "true", "2.5", "-1",
                "9223372036854775808")) {
            JsonObject malformed = JsonParser.parseString(feed.formatted(invalid)).getAsJsonObject();
            assertThrows(IllegalArgumentException.class, () -> decoder.decode(malformed), invalid);
        }
    }

    private static Map<String, String[]> domains() {
        return Map.of(
                "permissions", row("player:a", "{\"principalKind\":\"player\","
                        + "\"principalId\":\"a\",\"grants\":[],\"permissions\":[]}"),
                "claims", row("survival", "{\"chunks\":[]}"),
                "profiles", row(PLAYER + ":profile", "{\"playerUuid\":\"" + PLAYER
                        + "\",\"scope\":\"profile\",\"profile\":null}"),
                "presence", row("hub", "{\"instanceId\":\"hub\",\"available\":false}"),
                "routing", row("network", "{\"instances\":[]}"),
                "settings", row(PLAYER, "{\"playerUuid\":\"" + PLAYER
                        + "\",\"language\":\"en\",\"menuEnabled\":true,"
                        + "\"hudEnabled\":true,\"tipsEnabled\":true,\"privacy\":{}}"));
    }

    private static String[] row(String key, String payload) { return new String[] {key, payload}; }

    private static JsonObject snapshot(String domain, String key, String payload) {
        return JsonParser.parseString("{\"result\":\"snapshot\",\"domain\":\"" + domain
                + "\",\"key\":\"" + key + "\",\"revision\":1,"
                + "\"generatedAt\":\"2026-07-14T00:00:00Z\",\"credentialRevision\":1,"
                + "\"payload\":" + payload + "}").getAsJsonObject();
    }

    private static JsonElement wrong(JsonElement value) {
        if (value.isJsonArray() || value.isJsonObject()) return JsonParser.parseString("\"wrong\"");
        if (value.isJsonNull()) return JsonParser.parseString("false");
        if (value.getAsJsonPrimitive().isString()) return JsonParser.parseString("false");
        return JsonParser.parseString("\"wrong\"");
    }
}
