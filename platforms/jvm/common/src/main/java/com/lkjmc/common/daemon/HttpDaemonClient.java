package com.lkjmc.common.daemon;

import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.time.Duration;
import java.util.Optional;
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
            .POST(HttpRequest.BodyPublishers.ofString(DaemonJson.encodeRequest(request), StandardCharsets.UTF_8));
        token.ifPresent(value -> builder.header("Authorization", "Bearer " + value));
        return client.sendAsync(builder.build(), HttpResponse.BodyHandlers.ofString())
            .thenApply(response -> DaemonJson.decodeResponse(request.requestId(), response.body()))
            .exceptionally(error -> DaemonJson.error(
                request.requestId(), "daemon.http_failed", error.getMessage(), true
            ));
    }
}
