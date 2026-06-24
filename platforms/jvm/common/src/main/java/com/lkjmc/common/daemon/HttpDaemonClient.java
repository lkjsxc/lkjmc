package com.lkjmc.common.daemon;

import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.time.Duration;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;

public final class HttpDaemonClient implements DaemonClient {
    private final HttpClient client;
    private final URI endpoint;
    private final Optional<String> token;

    public HttpDaemonClient(URI endpoint, Optional<String> token) {
        this.client = HttpClient.newHttpClient();
        this.endpoint = endpoint;
        this.token = token == null ? Optional.empty() : token;
    }

    public static Optional<HttpDaemonClient> fromEnv() {
        var url = System.getenv("LKJMC_DAEMON_HTTP_URL");
        var token = Optional.ofNullable(System.getenv("LKJMC_DAEMON_HTTP_TOKEN"));
        if (url == null || url.isBlank() || token.isEmpty() || token.get().isBlank()) {
            return Optional.empty();
        }
        return Optional.of(new HttpDaemonClient(URI.create(url), token));
    }

    @Override
    public CompletableFuture<DaemonResponse> send(DaemonRequest request) {
        var builder = HttpRequest.newBuilder(endpoint)
            .timeout(Duration.ofSeconds(5))
            .header("Content-Type", "application/json")
            .POST(HttpRequest.BodyPublishers.ofString(encode(request), StandardCharsets.UTF_8));
        token.ifPresent(value -> builder.header("Authorization", "Bearer " + value));
        return client.sendAsync(builder.build(), HttpResponse.BodyHandlers.ofString())
            .thenApply(response -> decode(request.requestId(), response.body()))
            .exceptionally(error -> new DaemonResponse(
                request.requestId(),
                false,
                Map.of(),
                Optional.of(new DaemonError("daemon.http_failed", error.getMessage(), true))
            ));
    }

    private static String encode(DaemonRequest request) {
        return "{\"requestId\":\"" + request.requestId() + "\",\"actor\":{\"kind\":\""
            + escape(request.actor().kind()) + "\",\"name\":\"" + escape(request.actor().name())
            + "\"},\"command\":\"" + escape(request.command()) + "\",\"body\":"
            + encodeMap(request.body()) + "}";
    }

    private static String encodeMap(Map<String, Object> values) {
        var builder = new StringBuilder("{");
        var first = true;
        for (var entry : values.entrySet()) {
            if (!first) {
                builder.append(',');
            }
            first = false;
            builder.append('"').append(escape(entry.getKey())).append("\":");
            builder.append(encodeValue(entry.getValue()));
        }
        return builder.append('}').toString();
    }

    private static String encodeValue(Object value) {
        if (value == null) {
            return "null";
        }
        if (value instanceof Number || value instanceof Boolean) {
            return value.toString();
        }
        if (value instanceof Map<?, ?> map) {
            var values = new java.util.LinkedHashMap<String, Object>();
            map.forEach((key, item) -> values.put(key.toString(), item));
            return encodeMap(values);
        }
        return "\"" + escape(value.toString()) + "\"";
    }

    private static DaemonResponse decode(UUID requestId, String body) {
        var ok = body.contains("\"ok\":true");
        var message = ok ? Optional.<DaemonError>empty() : Optional.of(new DaemonError(
            extract(body, "code").orElse("daemon.error"),
            extract(body, "message").orElse(body),
            false
        ));
        return new DaemonResponse(requestId, ok, Map.of("raw", body), message);
    }

    private static Optional<String> extract(String json, String key) {
        var needle = "\"" + key + "\":\"";
        var start = json.indexOf(needle);
        if (start < 0) {
            return Optional.empty();
        }
        var valueStart = start + needle.length();
        var end = json.indexOf('"', valueStart);
        return end < 0 ? Optional.empty() : Optional.of(json.substring(valueStart, end));
    }

    private static String escape(String value) {
        return value.replace("\\", "\\\\").replace("\"", "\\\"");
    }
}
