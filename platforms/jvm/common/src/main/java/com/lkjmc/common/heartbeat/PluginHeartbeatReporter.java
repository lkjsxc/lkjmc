package com.lkjmc.common.heartbeat;

import java.io.IOException;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.LinkOption;
import java.nio.file.Path;
import java.nio.file.attribute.PosixFileAttributes;
import java.nio.file.attribute.PosixFilePermission;
import java.time.Duration;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicReference;
import java.util.function.Consumer;

/** One bounded, empty-body readiness heartbeat for a single plugin process. */
public final class PluginHeartbeatReporter implements AutoCloseable {
    static final String ENDPOINT_ENV = "LKJMC_HEARTBEAT_ENDPOINT";
    static final String CREDENTIAL_FILE_ENV = "LKJMC_HEARTBEAT_CREDENTIAL_FILE";
    static final String INSTANCE_ID_ENV = "LKJMC_INSTANCE_ID";
    private static final Duration DEFAULT_INTERVAL = Duration.ofSeconds(10);
    private static final Duration DEFAULT_TIMEOUT = Duration.ofSeconds(3);
    private static final int MAX_CREDENTIAL_BYTES = 512;

    private final String instanceId;
    private final URI endpoint;
    private final Path credentialFile;
    private final Duration interval;
    private final Duration timeout;
    private final Consumer<String> diagnosticSink;
    private final ScheduledExecutorService worker;
    private final HttpClient http;
    private final AtomicBoolean started = new AtomicBoolean();
    private final AtomicBoolean closed = new AtomicBoolean();
    private final AtomicReference<Boolean> healthy = new AtomicReference<>();

    public static Optional<PluginHeartbeatReporter> fromEnvironment(
            Map<String, String> environment, Consumer<String> diagnosticSink) {
        return fromEnvironment(
                environment, diagnosticSink, DEFAULT_INTERVAL, DEFAULT_TIMEOUT);
    }

    static Optional<PluginHeartbeatReporter> fromEnvironment(
            Map<String, String> environment,
            Consumer<String> diagnosticSink,
            Duration interval,
            Duration timeout) {
        String endpoint = environment.get(ENDPOINT_ENV);
        String credentialFile = environment.get(CREDENTIAL_FILE_ENV);
        String instanceId = environment.get(INSTANCE_ID_ENV);
        if (endpoint == null && credentialFile == null && instanceId == null) {
            return Optional.empty();
        }
        if (blank(endpoint) || blank(credentialFile) || blank(instanceId)) {
            throw new IllegalStateException("heartbeat endpoint, credential file, and instance id are required");
        }
        return Optional.of(new PluginHeartbeatReporter(
                instanceId, URI.create(endpoint), Path.of(credentialFile), interval, timeout,
                diagnosticSink));
    }

    private PluginHeartbeatReporter(
            String instanceId,
            URI endpoint,
            Path credentialFile,
            Duration interval,
            Duration timeout,
            Consumer<String> diagnosticSink) {
        if (!instanceId.matches("[A-Za-z0-9._-]{1,96}")) {
            throw new IllegalArgumentException("invalid heartbeat instance id");
        }
        if (!safeEndpoint(endpoint)) {
            throw new IllegalArgumentException("heartbeat endpoint must be exact loopback HTTP path");
        }
        if (!credentialFile.isAbsolute()) {
            throw new IllegalArgumentException("heartbeat credential file must be absolute");
        }
        if (!positive(interval) || !positive(timeout)) {
            throw new IllegalArgumentException("positive heartbeat interval and timeout required");
        }
        this.instanceId = instanceId;
        this.endpoint = endpoint;
        this.credentialFile = credentialFile;
        this.interval = interval;
        this.timeout = timeout;
        this.diagnosticSink = diagnosticSink;
        this.worker = Executors.newSingleThreadScheduledExecutor(task -> {
            Thread thread = new Thread(task, "lkjmc-heartbeat-" + instanceId);
            thread.setDaemon(true);
            return thread;
        });
        this.http = HttpClient.newBuilder()
                .connectTimeout(timeout.compareTo(Duration.ofSeconds(2)) < 0
                        ? timeout : Duration.ofSeconds(2))
                .build();
    }

    public void start() {
        if (closed.get()) throw new IllegalStateException("heartbeat reporter closed");
        if (started.compareAndSet(false, true)) {
            worker.scheduleWithFixedDelay(
                    this::heartbeat,
                    0,
                    Math.max(1, interval.toMillis()),
                    TimeUnit.MILLISECONDS);
        }
    }

    private void heartbeat() {
        if (closed.get()) return;
        try {
            String credential = credential();
            HttpRequest request = HttpRequest.newBuilder(endpoint)
                    .timeout(timeout)
                    .header("Authorization", "Bearer " + credential)
                    .POST(HttpRequest.BodyPublishers.noBody())
                    .build();
            HttpResponse<Void> response = http.send(request, HttpResponse.BodyHandlers.discarding());
            if (response.statusCode() != 204) {
                state(false, "status=" + response.statusCode());
                return;
            }
            state(true, "accepted");
        } catch (InterruptedException interrupted) {
            Thread.currentThread().interrupt();
            if (!closed.get()) state(false, "interrupted");
        } catch (IOException | RuntimeException failure) {
            state(false, failure.getClass().getSimpleName());
        }
    }

    private String credential() throws IOException {
        Path parent = credentialFile.getParent();
        PosixFileAttributes parentAttributes = Files.readAttributes(
                parent, PosixFileAttributes.class, LinkOption.NOFOLLOW_LINKS);
        if (!parentAttributes.isDirectory() || parentAttributes.permissions().stream()
                .anyMatch(permission -> permission != PosixFilePermission.OWNER_READ
                        && permission != PosixFilePermission.OWNER_WRITE
                        && permission != PosixFilePermission.OWNER_EXECUTE)) {
            throw new IOException("credential parent metadata is invalid");
        }
        PosixFileAttributes attributes = Files.readAttributes(
                credentialFile, PosixFileAttributes.class, LinkOption.NOFOLLOW_LINKS);
        if (!attributes.isRegularFile() || insecure(attributes.permissions())) {
            throw new IOException("credential file metadata is invalid");
        }
        long size = attributes.size();
        if (size < 1 || size > MAX_CREDENTIAL_BYTES) {
            throw new IOException("credential size is invalid");
        }
        String value = Files.readString(credentialFile, StandardCharsets.UTF_8).strip();
        if (value.isEmpty() || value.length() > MAX_CREDENTIAL_BYTES
                || value.chars().anyMatch(Character::isWhitespace)) {
            throw new IOException("credential format is invalid");
        }
        return value;
    }

    private void state(boolean available, String detail) {
        Boolean previous = healthy.getAndSet(available);
        if (previous != null && previous == available) return;
        String outcome = available ? "active" : "unavailable";
        try {
            diagnosticSink.accept("lkjmc heartbeat " + outcome + " instance=" + instanceId
                    + " detail=" + detail);
        } catch (RuntimeException ignored) {
            // Diagnostics must not stop future heartbeat attempts.
        }
    }

    @Override
    public void close() {
        if (closed.compareAndSet(false, true)) {
            worker.shutdownNow();
            http.shutdownNow();
        }
    }

    public boolean awaitClosed(Duration wait) throws InterruptedException {
        long deadline = System.nanoTime() + wait.toNanos();
        if (!worker.awaitTermination(Math.max(0, wait.toNanos()), TimeUnit.NANOSECONDS)) {
            return false;
        }
        Duration remaining = Duration.ofNanos(Math.max(0, deadline - System.nanoTime()));
        return http.awaitTermination(remaining);
    }

    private static boolean insecure(Set<PosixFilePermission> permissions) {
        return !permissions.contains(PosixFilePermission.OWNER_READ)
                || permissions.stream().anyMatch(permission -> switch (permission) {
                    case GROUP_READ, GROUP_WRITE, GROUP_EXECUTE,
                            OTHERS_READ, OTHERS_WRITE, OTHERS_EXECUTE -> true;
                    default -> false;
                });
    }

    private static boolean safeEndpoint(URI value) {
        String host = value.getHost();
        return "http".equals(value.getScheme())
                && host != null
                && (host.equalsIgnoreCase("localhost") || host.equals("127.0.0.1")
                    || host.equals("::1"))
                && value.getPort() > 0
                && "/plugin/v1/heartbeat".equals(value.getPath())
                && value.getRawQuery() == null
                && value.getRawFragment() == null
                && value.getUserInfo() == null;
    }

    private static boolean positive(Duration value) {
        return value != null && !value.isZero() && !value.isNegative();
    }

    private static boolean blank(String value) {
        return value == null || value.isBlank();
    }
}
