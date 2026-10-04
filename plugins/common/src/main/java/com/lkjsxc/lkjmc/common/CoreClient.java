package com.lkjsxc.lkjmc.common;

import com.google.gson.*;
import java.net.URI;
import java.net.http.*;
import java.nio.file.*;
import java.time.Duration;
import java.util.*;
import java.util.concurrent.*;

/** The credential belongs to this trusted adapter. It is never sent in plugin messages. */
public final class CoreClient implements AutoCloseable {
  public static final Gson JSON = new GsonBuilder().disableHtmlEscaping().create();
  private final HttpClient http;
  private final URI origin;
  private final String token;

  public CoreClient(URI origin, Path tokenFile) throws Exception {
    if (!Set.of("http", "https").contains(origin.getScheme())
        || origin.getQuery() != null
        || origin.getUserInfo() != null) throw new IllegalArgumentException("Invalid core origin");
    if (!origin.getScheme().equals("https")
        && !Set.of("127.0.0.1", "localhost").contains(origin.getHost()))
      throw new IllegalArgumentException("Non-local core connections require HTTPS");
    this.origin = origin;
    this.token = Files.readString(tokenFile).trim();
    if (token.length() < 32) throw new IllegalArgumentException("Missing service credential");
    http =
        HttpClient.newBuilder()
            .connectTimeout(Duration.ofSeconds(10))
            .followRedirects(HttpClient.Redirect.NEVER)
            .build();
  }

  public JsonObject get(String path) throws Exception {
    return request("GET", path, null);
  }

  public JsonObject post(String path, JsonObject value) throws Exception {
    return request("POST", path, value);
  }

  public JsonObject post(String path, JsonArray value) throws Exception {
    return request("POST", path, value);
  }

  private JsonObject request(String method, String path, JsonElement data) throws Exception {
    if (!path.startsWith("/internal/v1/"))
      throw new IllegalArgumentException("Not an internal endpoint");
    var builder =
        HttpRequest.newBuilder(origin.resolve(path))
            .timeout(Duration.ofSeconds(20))
            .header("Authorization", "Bearer " + token)
            .header("Content-Type", "application/json")
            .header("User-Agent", "lkjmc-adapter/0.1");
    builder.method(
        method,
        data == null
            ? HttpRequest.BodyPublishers.noBody()
            : HttpRequest.BodyPublishers.ofString(JSON.toJson(data)));
    var response = http.send(builder.build(), HttpResponse.BodyHandlers.ofString());
    JsonObject body;
    try {
      body = JsonParser.parseString(response.body()).getAsJsonObject();
    } catch (RuntimeException e) {
      throw new CoreFailure(response.statusCode(), "Could not read the shared service response.");
    }
    if (response.statusCode() / 100 != 2) {
      SystemMessage message =
          body.has("error")
              ? SystemMessage.parse(body.getAsJsonObject("error").get("message"))
              : SystemMessage.unknown("http-" + response.statusCode());
      throw new CoreFailure(response.statusCode(), message);
    }
    return body;
  }

  public JsonObject command(JsonObject session, JsonObject command, UUID requestId)
      throws Exception {
    var body = new JsonObject();
    body.add("account_id", session.get("account_id"));
    body.add("session_id", session.get("session_id"));
    body.addProperty("request_id", requestId.toString());
    body.add("command", command);
    return post("/internal/v1/game/command", body);
  }

  public void ack(
      JsonObject job, String state, JsonObject result, JsonObject progress, String error)
      throws Exception {
    var body = new JsonObject();
    body.add("lease_token", job.get("lease_token"));
    body.addProperty("state", state);
    body.add("result", result == null ? new JsonObject() : result);
    JsonObject safeProgress = progress == null ? new JsonObject() : progress.deepCopy();
    if (safeProgress.has("message") && safeProgress.get("message").isJsonPrimitive())
      safeProgress.add(
          "message", SystemMessage.decode(safeProgress.get("message").getAsString()).json());
    body.add("progress", safeProgress);
    if (error != null) body.addProperty("error", SystemMessage.decode(error).toString());
    post("/internal/v1/jobs/" + job.get("id").getAsString() + "/ack", body);
  }

  public static JsonObject object(Object... values) {
    JsonObject out = new JsonObject();
    for (int i = 0; i < values.length; i += 2)
      out.add((String) values[i], JSON.toJsonTree(values[i + 1]));
    return out;
  }

  public static UUID uuid(JsonObject value, String key) {
    return UUID.fromString(value.get(key).getAsString());
  }

  public static String string(JsonObject value, String key, String fallback) {
    return value.has(key) && !value.get(key).isJsonNull() ? value.get(key).getAsString() : fallback;
  }

  public void close() {
    http.close();
  }

  public static final class CoreFailure extends Exception {
    public final int status;

    public final SystemMessage systemMessage;

    public CoreFailure(int status, String diagnostic) {
      this(status, SystemMessage.unknown(diagnostic));
    }

    public CoreFailure(int status, SystemMessage message) {
      super(message.toString());
      this.status = status;
      this.systemMessage = message;
    }
  }
}
