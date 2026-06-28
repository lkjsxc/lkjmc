package com.lkjmc.common.daemon;

import java.io.IOException;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
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
        var status = DaemonHttpConfigStatus.fromEnv();
        if (!status.configured()) {
            return Optional.empty();
        }
        var url = System.getenv("LKJMC_DAEMON_HTTP_URL");
        var token = tokenFrom(
            Optional.ofNullable(System.getenv("LKJMC_DAEMON_HTTP_TOKEN")),
            Optional.ofNullable(System.getenv("LKJMC_DAEMON_HTTP_TOKEN_FILE"))
        );
        if (token.isEmpty()) {
            return Optional.empty();
        }
        return Optional.of(new HttpDaemonClient(URI.create(url), token));
    }

    static Optional<String> tokenFrom(Optional<String> direct, Optional<String> tokenFile) {
        var directToken = direct.map(String::trim).filter(value -> !value.isBlank());
        if (directToken.isPresent()) {
            return directToken;
        }
        return tokenFile.map(String::trim)
            .filter(value -> !value.isBlank())
            .flatMap(HttpDaemonClient::readTokenFile)
            .map(String::trim)
            .filter(value -> !value.isBlank());
    }

    private static Optional<String> readTokenFile(String tokenFile) {
        try {
            return Optional.of(Files.readString(Path.of(tokenFile), StandardCharsets.UTF_8));
        } catch (IOException error) {
            return Optional.empty();
        }
    }

    @Override
    public CompletableFuture<DaemonResponse> send(DaemonRequest request) {
        var builder = HttpRequest.newBuilder(endpoint)
            .timeout(Duration.ofSeconds(5))
            .header("Content-Type", "application/json")
            .POST(HttpRequest.BodyPublishers.ofString(DaemonJson.encodeRequest(request), StandardCharsets.UTF_8));
        token.ifPresent(value -> builder.header("Authorization", "Bearer " + value));
        return client.sendAsync(builder.build(), HttpResponse.BodyHandlers.ofString())
            .thenApply(response -> decodeHttp(request, response))
            .exceptionally(error -> DaemonJson.error(
                request.requestId(), "daemon.http_failed", error.getMessage(), true
            ));
    }

    private static DaemonResponse decodeHttp(DaemonRequest request, HttpResponse<String> response) {
        var status = response.statusCode();
        if (status == 401 || status == 403) {
            return DaemonJson.error(request.requestId(), "daemon.auth_failed", "daemon authentication failed", false);
        }
        if (status < 200 || status >= 300) {
            return DaemonJson.error(request.requestId(), "daemon.http_failed", "http status " + status, true);
        }
        return DaemonJson.decodeResponse(request.requestId(), response.body());
    }
}
