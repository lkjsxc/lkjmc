package com.lkjmc.common.sync;

import java.io.IOException;
import java.net.InetSocketAddress;
import java.net.ServerSocket;
import java.net.Socket;
import java.net.URI;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Duration;
import java.util.concurrent.TimeUnit;

final class HarnessDaemon implements AutoCloseable {
    private final Path root;
    private final String databaseUrl;
    private final int port;
    private final Path socket;
    private final Path log;
    private Process process;

    HarnessDaemon(Path root, String databaseUrl) throws Exception {
        this.root = root;
        this.databaseUrl = databaseUrl;
        try (ServerSocket available = new ServerSocket(0, 1, java.net.InetAddress.getLoopbackAddress())) {
            port = available.getLocalPort();
        }
        socket = Files.createTempFile("lkjmc-sync-", ".sock");
        Files.deleteIfExists(socket);
        log = Files.createTempFile("lkjmc-sync-daemon-", ".log");
        start();
    }

    URI endpoint() {
        return URI.create("http://127.0.0.1:" + port);
    }

    void start() throws Exception {
        if (process != null && process.isAlive()) {
            throw new IllegalStateException("daemon already running");
        }
        ProcessBuilder builder = new ProcessBuilder(
                root.resolve("target/debug/lkjmc-daemon").toString(),
                "--socket", socket.toString(), "--http", "127.0.0.1:" + port,
                "--database-url", databaseUrl, "--data-root", root.resolve("tmp/sync-data").toString(),
                "--log-root", root.resolve("tmp/sync-logs").toString(),
                "--jar-root", root.resolve("tmp/sync-jars").toString());
        builder.directory(root.toFile()).redirectErrorStream(true).redirectOutput(ProcessBuilder.Redirect.appendTo(log.toFile()));
        process = builder.start();
        long deadline = System.nanoTime() + Duration.ofSeconds(10).toNanos();
        while (System.nanoTime() < deadline) {
            if (!process.isAlive()) {
                throw new IllegalStateException("daemon exited: " + logText());
            }
            try (Socket probe = new Socket()) {
                probe.connect(new InetSocketAddress("127.0.0.1", port), 100);
                return;
            } catch (IOException unavailable) {
                Thread.sleep(25);
            }
        }
        throw new IllegalStateException("daemon did not listen: " + logText());
    }

    void stop() throws Exception {
        if (process == null || !process.isAlive()) {
            return;
        }
        process.destroy();
        if (!process.waitFor(8, TimeUnit.SECONDS)) {
            process.destroyForcibly();
            if (!process.waitFor(2, TimeUnit.SECONDS)) {
                throw new IllegalStateException("daemon did not stop");
            }
        }
        Files.deleteIfExists(socket);
    }

    String logText() throws IOException {
        return Files.readString(log);
    }

    @Override
    public void close() throws Exception {
        stop();
        Files.deleteIfExists(socket);
        Files.deleteIfExists(log);
    }
}
