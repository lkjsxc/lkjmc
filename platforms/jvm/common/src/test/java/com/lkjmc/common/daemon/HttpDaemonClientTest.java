package com.lkjmc.common.daemon;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
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
}
