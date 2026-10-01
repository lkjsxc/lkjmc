package com.lkjsxc.lkjmc.floodgate;

import com.google.gson.*;
import com.lkjsxc.lkjmc.common.CoreClient;
import java.net.URI;
import java.nio.file.*;
import java.util.UUID;
import java.util.concurrent.*;
import org.geysermc.floodgate.api.link.*;
import org.geysermc.floodgate.util.LinkedPlayer;

/** Floodgate database extension. Core confirms both identities and chooses native data first. */
public final class CoreLinks implements PlayerLink {
  private CoreClient core;
  private final ExecutorService io = Executors.newVirtualThreadPerTaskExecutor();

  @Override
  public void load() {
    try {
      Path file = Path.of(System.getProperty("lkjmc.linkConfig", "plugins/lkjmc/config.json"));
      JsonObject config = JsonParser.parseString(Files.readString(file)).getAsJsonObject();
      core =
          new CoreClient(
              URI.create(config.get("core_url").getAsString()),
              Path.of(config.get("credential_file").getAsString()));
    } catch (Exception e) {
      throw new IllegalStateException("lkjmc local identity source could not be configured", e);
    }
  }

  @Override
  public CompletableFuture<LinkedPlayer> getLinkedPlayer(UUID bedrockId) {
    return CompletableFuture.supplyAsync(
        () -> {
          try {
            JsonElement result = core.get("/internal/v1/game/linked/" + bedrockId).get("link");
            if (result == null || result.isJsonNull()) return null;
            JsonObject link = result.getAsJsonObject();
            return LinkedPlayer.of(
                link.get("java_username").getAsString(),
                CoreClient.uuid(link, "java_uuid"),
                bedrockId);
          } catch (Exception e) {
            throw new CompletionException("Cannot verify local identity link", e);
          }
        },
        io);
  }

  @Override
  public CompletableFuture<Boolean> isLinkedPlayer(UUID playerId) {
    return getLinkedPlayer(playerId).thenApply(link -> link != null);
  }

  private <T> CompletableFuture<T> refused() {
    return CompletableFuture.failedFuture(
        new IllegalStateException(
            "Use the lkjmc identity confirmation and native-data selection flow"));
  }

  @Override
  public CompletableFuture<Void> linkPlayer(UUID bedrock, UUID java, String username) {
    return refused();
  }

  @Override
  public CompletableFuture<Void> unlinkPlayer(UUID id) {
    return refused();
  }

  @Override
  public CompletableFuture<?> createLinkRequest(UUID id, String username, String other) {
    return refused();
  }

  @Override
  public CompletableFuture<LinkRequestResult> verifyLinkRequest(
      UUID id, String username, String other, String code) {
    return refused();
  }

  @Override
  public String getName() {
    return "lkjmc";
  }

  @Override
  public boolean isEnabled() {
    return core != null;
  }

  @Override
  public long getVerifyLinkTimeout() {
    return 0;
  }

  @Override
  public boolean isAllowLinking() {
    return false;
  }

  @Override
  public void stop() {
    io.shutdown();
    if (core != null) core.close();
  }
}
