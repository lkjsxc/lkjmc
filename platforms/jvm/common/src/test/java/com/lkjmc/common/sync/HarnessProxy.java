package com.lkjmc.common.sync;

import com.sun.net.httpserver.HttpExchange;
import com.sun.net.httpserver.HttpServer;
import java.io.IOException;
import java.net.InetSocketAddress;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.time.Duration;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;

final class HarnessProxy implements AutoCloseable {
    private final URI daemon;
    private final HttpServer server;
    private final HttpClient client = HttpClient.newBuilder().connectTimeout(Duration.ofSeconds(1)).build();
    private final ExecutorService executor;
    private final AtomicInteger active = new AtomicInteger();
    private final AtomicInteger maximum = new AtomicInteger();
    private final AtomicInteger snapshots = new AtomicInteger();
    private final AtomicBoolean dropSnapshot = new AtomicBoolean();
    private final Object holdMonitor = new Object();
    private volatile boolean holdAll;
    private volatile boolean holdNext;
    private volatile CountDownLatch captured = new CountDownLatch(0);
    private volatile String lastSnapshot = "none";

    HarnessProxy(URI daemon) throws IOException {
        this.daemon = daemon;
        server = HttpServer.create(new InetSocketAddress("127.0.0.1", 0), 32);
        executor = Executors.newCachedThreadPool(runnable -> {
            Thread thread = new Thread(runnable, "sync-harness-proxy");
            thread.setDaemon(true);
            return thread;
        });
        server.setExecutor(executor);
        server.createContext("/sync/", this::handle);
        server.start();
    }

    URI endpoint() {
        return URI.create("http://127.0.0.1:" + server.getAddress().getPort());
    }

    int maximumConcurrent() { return maximum.get(); }
    int snapshotRequests() { return snapshots.get(); }
    String lastSnapshot() { return lastSnapshot; }
    void dropOneSnapshot() { dropSnapshot.set(true); }

    void holdNextSnapshot() {
        captured = new CountDownLatch(1);
        holdNext = true;
    }

    boolean awaitCaptured(Duration timeout) throws InterruptedException {
        return captured.await(timeout.toMillis(), TimeUnit.MILLISECONDS);
    }

    void holdAllSnapshots(boolean value) {
        holdAll = value;
        if (!value) {
            release();
        }
    }

    void release() {
        synchronized (holdMonitor) {
            holdNext = false;
            holdMonitor.notifyAll();
        }
    }

    private void handle(HttpExchange exchange) throws IOException {
        int current = active.incrementAndGet();
        maximum.accumulateAndGet(current, Math::max);
        try {
            boolean snapshot = exchange.getRequestURI().getPath().endsWith("/snapshot");
            if (snapshot) snapshots.incrementAndGet();
            if (snapshot && dropSnapshot.compareAndSet(true, false)) {
                reply(exchange, 503, new byte[0]);
                return;
            }
            byte[] requestBody = exchange.getRequestBody().readNBytes(1024 * 1024 + 1);
            HttpRequest.Builder builder = HttpRequest.newBuilder(daemon.resolve(exchange.getRequestURI()))
                    .timeout(Duration.ofSeconds(3))
                    .header("Content-Type", "application/json")
                    .POST(HttpRequest.BodyPublishers.ofByteArray(requestBody));
            String authorization = exchange.getRequestHeaders().getFirst("Authorization");
            if (authorization != null) {
                builder.header("Authorization", authorization);
            }
            HttpResponse<byte[]> response = client.send(builder.build(), HttpResponse.BodyHandlers.ofByteArray());
            if (snapshot) {
                lastSnapshot = response.statusCode() + ":" + new String(response.body(), java.nio.charset.StandardCharsets.UTF_8);
            }
            if (snapshot && holdNext) {
                captured.countDown();
                awaitRelease();
            } else if (snapshot && holdAll) {
                awaitRelease();
            }
            reply(exchange, response.statusCode(), response.body());
        } catch (InterruptedException interrupted) {
            Thread.currentThread().interrupt();
            reply(exchange, 503, new byte[0]);
        } catch (Exception unavailable) {
            reply(exchange, 503, new byte[0]);
        } finally {
            active.decrementAndGet();
            exchange.close();
        }
    }

    private void awaitRelease() throws InterruptedException {
        synchronized (holdMonitor) {
            while (holdNext || holdAll) {
                holdMonitor.wait(1000);
            }
        }
    }

    private static void reply(HttpExchange exchange, int status, byte[] body) throws IOException {
        exchange.getResponseHeaders().set("Content-Type", "application/json");
        exchange.sendResponseHeaders(status, body.length);
        exchange.getResponseBody().write(body);
    }

    @Override
    public void close() throws InterruptedException {
        release();
        holdAll = false;
        server.stop(0);
        executor.shutdownNow();
        executor.awaitTermination(2, TimeUnit.SECONDS);
    }
}
