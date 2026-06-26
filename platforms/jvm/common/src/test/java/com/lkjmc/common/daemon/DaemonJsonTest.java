package com.lkjmc.common.daemon;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.JsonParser;
import java.net.URI;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.TimeUnit;
import org.junit.jupiter.api.Test;

final class DaemonJsonTest {
    private static final UUID REQUEST_ID = UUID.fromString("00000000-0000-0000-0000-000000000001");

    @Test
    void encodesNestedRequestValues() {
        var request = new DaemonRequest(REQUEST_ID, new DaemonActor("paper-plugin", "hub"), "test.command", Map.of(
            "text", "quote \" and slash \\",
            "flag", true,
            "count", 3,
            "nested", Map.of("world", "world", "x", 12.5),
            "list", List.of("a", "b")
        ));
        var root = JsonParser.parseString(DaemonJson.encodeRequest(request)).getAsJsonObject();
        assertEquals("test.command", root.get("command").getAsString());
        assertEquals("quote \" and slash \\", root.getAsJsonObject("body").get("text").getAsString());
        assertTrue(root.getAsJsonObject("body").get("flag").getAsBoolean());
        assertEquals(3, root.getAsJsonObject("body").get("count").getAsInt());
        assertEquals(12.5, root.getAsJsonObject("body").getAsJsonObject("nested").get("x").getAsDouble());
        assertEquals(2, root.getAsJsonObject("body").getAsJsonArray("list").size());
    }

    @Test
    void decodesSuccessAndErrorBodies() {
        var success = DaemonJson.decodeResponse(REQUEST_ID,
            "{\"requestId\":\"00000000-0000-0000-0000-000000000002\",\"ok\":true,\"body\":{\"found\":true}}"
        );
        assertTrue(success.ok());
        assertTrue(DaemonJson.bool(success.body(), "found"));
        assertEquals(UUID.fromString("00000000-0000-0000-0000-000000000002"), success.requestId());

        var failure = DaemonJson.decodeResponse(REQUEST_ID,
            "{\"ok\":false,\"error\":{\"code\":\"database.error\",\"message\":\"no\",\"retryable\":true}}"
        );
        assertFalse(failure.ok());
        assertEquals("database.error", failure.error().orElseThrow().code());
        assertTrue(failure.error().orElseThrow().retryable());
    }

    @Test
    void invalidJsonBecomesDaemonError() {
        var response = DaemonJson.decodeResponse(REQUEST_ID, "not json");
        assertFalse(response.ok());
        assertEquals("daemon.invalid_json", response.error().orElseThrow().code());
    }

    @Test
    void httpExceptionBecomesRetryableError() throws Exception {
        var client = new HttpDaemonClient(URI.create("http://127.0.0.1:1/"), Optional.of("token"));
        var response = client.send(new DaemonRequest(
            REQUEST_ID, new DaemonActor("test", "test"), "status", Map.of()
        )).get(8, TimeUnit.SECONDS);
        assertFalse(response.ok());
        assertEquals("daemon.http_failed", response.error().orElseThrow().code());
        assertTrue(response.error().orElseThrow().retryable());
    }
}
