package com.lkjmc.common.daemon;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.io.OutputStreamWriter;
import java.net.InetAddress;
import java.net.ServerSocket;
import java.net.Socket;
import java.net.URI;
import java.nio.charset.StandardCharsets;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import org.junit.jupiter.api.Test;

final class HttpDaemonClientBoundaryTest {
    @Test
    void sends_a_real_loopback_tcp_request() throws Exception {
        var requestId = UUID.randomUUID();
        try (var server = new ServerSocket(0, 1, InetAddress.getLoopbackAddress())) {
            var seenAuthorization = new CompletableFuture<String>();
            var worker = new Thread(() -> reply(server, requestId, seenAuthorization));
            worker.start();
            var client = new HttpDaemonClient(
                URI.create("http://127.0.0.1:" + server.getLocalPort()),
                java.util.Optional.of("boundary-token")
            );
            var request = new DaemonRequest(requestId, new DaemonActor("paper", "lab"), "doctor", Map.of());
            var response = client.send(request).get(5, TimeUnit.SECONDS);
            assertTrue(response.ok());
            assertEquals(requestId, response.requestId());
            assertEquals("Bearer boundary-token", seenAuthorization.get(5, TimeUnit.SECONDS));
            worker.join(5000);
        }
    }

    private static void reply(ServerSocket server, UUID requestId, CompletableFuture<String> authorization) {
        try (Socket socket = server.accept();
             var input = new BufferedReader(new InputStreamReader(socket.getInputStream(), StandardCharsets.UTF_8));
             var output = new OutputStreamWriter(socket.getOutputStream(), StandardCharsets.UTF_8)) {
            String line;
            while ((line = input.readLine()) != null && !line.isEmpty()) {
                if (line.regionMatches(true, 0, "Authorization:", 0, "Authorization:".length())) {
                    authorization.complete(line.substring("Authorization:".length()).trim());
                }
            }
            var body = "{\"requestId\":\"" + requestId + "\",\"ok\":true,\"body\":{}}";
            output.write("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: "
                + body.getBytes(StandardCharsets.UTF_8).length + "\r\nConnection: close\r\n\r\n" + body);
            output.flush();
            authorization.complete("missing");
        } catch (Exception error) {
            authorization.completeExceptionally(error);
        }
    }
}
