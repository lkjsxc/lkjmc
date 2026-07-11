package com.lkjmc.common.daemon;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.util.Map;
import java.util.Optional;
import org.junit.jupiter.api.Test;

final class HttpDaemonClientTest {
    @Test
    void directTokenWins() throws Exception {
        var file = Files.createTempFile("lkjmc-token", ".txt");
        Files.writeString(file, "file-token\n", StandardCharsets.UTF_8);
        var token = HttpDaemonClient.tokenFrom(Optional.of(" direct-token "), Optional.of(file.toString()));
        assertEquals(Optional.of("direct-token"), token);
        Files.deleteIfExists(file);
    }

    @Test
    void readsTokenFileWhenDirectTokenMissing() throws Exception {
        var file = Files.createTempFile("lkjmc-token", ".txt");
        Files.writeString(file, "file-token\n", StandardCharsets.UTF_8);
        var token = HttpDaemonClient.tokenFrom(Optional.empty(), Optional.of(file.toString()));
        assertEquals(Optional.of("file-token"), token);
        Files.deleteIfExists(file);
    }

    @Test
    void missingTokenReturnsEmpty() {
        var token = HttpDaemonClient.tokenFrom(Optional.of(" "), Optional.of("/missing/lkjmc-token"));
        assertTrue(token.isEmpty());
    }

    @Test
    void credentialSnapshotDoesNotRereadTokenFile() throws Exception {
        var file = Files.createTempFile("lkjmc-token", ".txt");
        Files.writeString(file, "old\n", StandardCharsets.UTF_8);
        var snapshot = HttpDaemonClient.tokenFrom(Optional.empty(), Optional.of(file.toString()));
        var client = new HttpDaemonClient(java.net.URI.create("http://127.0.0.1:9"), snapshot);
        Files.writeString(file, "new\n", StandardCharsets.UTF_8);
        assertEquals(Optional.of("old"), client.currentToken());
        Files.deleteIfExists(file);
    }

    @Test
    void classifiesDaemonHttpConfiguration() {
        assertEquals("daemon.not_configured", DaemonHttpConfigStatus.from(Map.of(), path -> Optional.empty()).code());
        assertEquals("daemon.token_missing", DaemonHttpConfigStatus.from(
            Map.of("LKJMC_DAEMON_HTTP_URL", "http://127.0.0.1:8765"), path -> Optional.empty()).code());
        assertEquals("daemon.token_unreadable", DaemonHttpConfigStatus.from(Map.of(
            "LKJMC_DAEMON_HTTP_URL", "http://127.0.0.1:8765",
            "LKJMC_DAEMON_HTTP_TOKEN_FILE", "/missing"), path -> Optional.empty()).code());
        assertTrue(DaemonHttpConfigStatus.from(Map.of(
            "LKJMC_DAEMON_HTTP_URL", "http://127.0.0.1:8765",
            "LKJMC_DAEMON_HTTP_TOKEN", "secret"), path -> Optional.empty()).configured());
    }
}
