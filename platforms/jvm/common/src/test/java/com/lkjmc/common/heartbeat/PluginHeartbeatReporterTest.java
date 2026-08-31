package com.lkjmc.common.heartbeat;

import static org.junit.jupiter.api.Assertions.*;

import com.sun.net.httpserver.HttpExchange;
import com.sun.net.httpserver.HttpServer;
import java.io.IOException;
import java.net.InetSocketAddress;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.attribute.PosixFilePermission;
import java.time.Duration;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicReference;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

final class PluginHeartbeatReporterTest {
    @TempDir Path root;

    @Test
    void sendsOnlyAnEmptyAuthenticatedLoopbackPost() throws Exception {
        String credential = "MiXeD-Case_heartbeat-token";
        Path credentialFile = root.resolve("hub.secret");
        writeCredential(credentialFile, credential + "\n");
        CountDownLatch accepted = new CountDownLatch(1);
        AtomicReference<String> method = new AtomicReference<>();
        AtomicReference<String> authorization = new AtomicReference<>();
        AtomicReference<byte[]> body = new AtomicReference<>();
        HttpServer server = server(exchange -> {
            method.set(exchange.getRequestMethod());
            authorization.set(exchange.getRequestHeaders().getFirst("Authorization"));
            body.set(exchange.getRequestBody().readAllBytes());
            exchange.sendResponseHeaders(204, -1);
            exchange.close();
            accepted.countDown();
        });
        List<String> diagnostics = new CopyOnWriteArrayList<>();
        PluginHeartbeatReporter reporter = reporter(
                server, credentialFile, diagnostics, Duration.ofSeconds(1));
        try {
            reporter.start();
            assertTrue(accepted.await(2, TimeUnit.SECONDS));
            assertEquals("POST", method.get());
            assertEquals("Bearer " + credential, authorization.get());
            assertArrayEquals(new byte[0], body.get());
            await(() -> diagnostics.stream().anyMatch(value -> value.contains("heartbeat active")));
            assertTrue(diagnostics.stream().noneMatch(value -> value.contains(credential)));
        } finally {
            reporter.close();
            server.stop(0);
        }
    }

    @Test
    void sendsAConfiguredBoundedJsonObservationWithoutLoggingIt() throws Exception {
        String credential = "velocity-heartbeat-token";
        Path credentialFile = root.resolve("edge-gateway.secret");
        writeCredential(credentialFile, credential);
        String payload = "{\"registrations\":[{\"instanceId\":\"quartz-world\","
                + "\"connectHost\":\"127.0.0.1\",\"connectPort\":25566,"
                + "\"registered\":true}]}";
        CountDownLatch accepted = new CountDownLatch(1);
        AtomicReference<String> contentType = new AtomicReference<>();
        AtomicReference<String> body = new AtomicReference<>();
        HttpServer server = server(exchange -> {
            contentType.set(exchange.getRequestHeaders().getFirst("Content-Type"));
            body.set(new String(exchange.getRequestBody().readAllBytes(), StandardCharsets.UTF_8));
            exchange.sendResponseHeaders(204, -1);
            exchange.close();
            accepted.countDown();
        });
        List<String> diagnostics = new CopyOnWriteArrayList<>();
        int port = server.getAddress().getPort();
        PluginHeartbeatReporter reporter = PluginHeartbeatReporter.fromEnvironment(
                        Map.of(
                                PluginHeartbeatReporter.INSTANCE_ID_ENV, "edge-gateway",
                                PluginHeartbeatReporter.ENDPOINT_ENV,
                                "http://127.0.0.1:" + port + "/plugin/v1/heartbeat",
                                PluginHeartbeatReporter.CREDENTIAL_FILE_ENV,
                                credentialFile.toString()),
                        diagnostics::add,
                        () -> payload,
                        Duration.ofSeconds(1),
                        Duration.ofSeconds(1))
                .orElseThrow();
        try {
            reporter.start();
            assertTrue(accepted.await(2, TimeUnit.SECONDS));
            assertEquals("application/json", contentType.get());
            assertEquals(payload, body.get());
            assertTrue(diagnostics.stream().noneMatch(value ->
                    value.contains(credential) || value.contains("quartz-world")));
        } finally {
            reporter.close();
            server.stop(0);
        }
    }

    @Test
    void unavailableDaemonIsRetriedWithoutQueueGrowthOrSecretLogging() throws Exception {
        String credential = "retry-heartbeat-token";
        Path credentialFile = root.resolve("survival.secret");
        writeCredential(credentialFile, credential);
        AtomicInteger requests = new AtomicInteger();
        CountDownLatch retried = new CountDownLatch(2);
        HttpServer server = server(exchange -> {
            int status = requests.incrementAndGet() == 1 ? 503 : 204;
            exchange.getRequestBody().readAllBytes();
            exchange.sendResponseHeaders(status, -1);
            exchange.close();
            retried.countDown();
        });
        List<String> diagnostics = new CopyOnWriteArrayList<>();
        PluginHeartbeatReporter reporter = reporter(
                server, credentialFile, diagnostics, Duration.ofMillis(20));
        try {
            reporter.start();
            assertTrue(retried.await(2, TimeUnit.SECONDS));
            await(() -> diagnostics.stream().anyMatch(value -> value.contains("heartbeat active")));
            assertTrue(diagnostics.stream().anyMatch(value -> value.contains("heartbeat unavailable")));
            assertTrue(diagnostics.stream().noneMatch(value -> value.contains(credential)));
        } finally {
            reporter.close();
            server.stop(0);
        }
    }

    @Test
    void closeInterruptsAndJoinsAnInflightHeartbeat() throws Exception {
        Path credentialFile = root.resolve("proxy.secret");
        writeCredential(credentialFile, "shutdown-heartbeat-token");
        CountDownLatch requestStarted = new CountDownLatch(1);
        CountDownLatch releaseServer = new CountDownLatch(1);
        HttpServer server = server(exchange -> {
            requestStarted.countDown();
            try {
                releaseServer.await(2, TimeUnit.SECONDS);
            } catch (InterruptedException interrupted) {
                Thread.currentThread().interrupt();
            }
            exchange.close();
        });
        PluginHeartbeatReporter reporter = reporter(
                server, credentialFile, new CopyOnWriteArrayList<>(), Duration.ofSeconds(1));
        try {
            reporter.start();
            assertTrue(requestStarted.await(2, TimeUnit.SECONDS));
            reporter.close();
            assertTrue(reporter.awaitClosed(Duration.ofSeconds(2)));
            assertTrue(Thread.getAllStackTraces().keySet().stream().noneMatch(thread ->
                    thread.isAlive() && thread.getName().equals("lkjmc-heartbeat-hub")));
        } finally {
            releaseServer.countDown();
            reporter.close();
            server.stop(0);
        }
    }

    @Test
    void partialOrNonLoopbackConfigurationFailsClosed() {
        assertThrows(IllegalStateException.class, () ->
                PluginHeartbeatReporter.fromEnvironment(
                        Map.of(PluginHeartbeatReporter.INSTANCE_ID_ENV, "hub"), ignored -> {}));
        assertThrows(IllegalArgumentException.class, () ->
                PluginHeartbeatReporter.fromEnvironment(
                        Map.of(
                                PluginHeartbeatReporter.INSTANCE_ID_ENV, "hub",
                                PluginHeartbeatReporter.ENDPOINT_ENV,
                                "http://example.com:8765/plugin/v1/heartbeat",
                                PluginHeartbeatReporter.CREDENTIAL_FILE_ENV,
                                root.resolve("hub.secret").toString()),
                        ignored -> {}));
    }

    private void writeCredential(Path path, String credential) throws IOException {
        Files.writeString(path, credential, StandardCharsets.UTF_8);
        Files.setPosixFilePermissions(path, Set.of(
                PosixFilePermission.OWNER_READ,
                PosixFilePermission.OWNER_WRITE));
    }

    private PluginHeartbeatReporter reporter(
            HttpServer server,
            Path credentialFile,
            List<String> diagnostics,
            Duration interval) {
        int port = server.getAddress().getPort();
        return PluginHeartbeatReporter.fromEnvironment(
                        Map.of(
                                PluginHeartbeatReporter.INSTANCE_ID_ENV, "hub",
                                PluginHeartbeatReporter.ENDPOINT_ENV,
                                "http://127.0.0.1:" + port + "/plugin/v1/heartbeat",
                                PluginHeartbeatReporter.CREDENTIAL_FILE_ENV,
                                credentialFile.toString()),
                        diagnostics::add,
                        interval,
                        Duration.ofSeconds(1))
                .orElseThrow();
    }

    private HttpServer server(Handler handler) throws IOException {
        HttpServer server = HttpServer.create(new InetSocketAddress("127.0.0.1", 0), 8);
        server.createContext("/plugin/v1/heartbeat", exchange -> handler.handle(exchange));
        server.start();
        return server;
    }

    private static void await(Check check) throws InterruptedException {
        long deadline = System.nanoTime() + Duration.ofSeconds(2).toNanos();
        while (!check.value() && System.nanoTime() < deadline) Thread.sleep(5);
        assertTrue(check.value(), "condition did not become true before deadline");
    }

    @FunctionalInterface
    private interface Handler {
        void handle(HttpExchange exchange) throws IOException;
    }

    @FunctionalInterface
    private interface Check {
        boolean value();
    }
}
