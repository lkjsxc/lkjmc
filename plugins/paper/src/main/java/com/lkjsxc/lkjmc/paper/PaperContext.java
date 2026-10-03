package com.lkjsxc.lkjmc.paper;

import com.google.gson.JsonObject;
import com.lkjsxc.lkjmc.common.CoreClient;
import java.util.UUID;
import java.util.concurrent.*;
import org.bukkit.*;
import org.bukkit.plugin.java.JavaPlugin;

public interface PaperContext {
  JavaPlugin plugin();

  CoreClient core();

  UUID serverId();

  JsonObject projection();

  void refreshProjection() throws Exception;

  default boolean inCombat(UUID nativeId) {
    return false;
  }

  default boolean departing(UUID nativeId) {
    return false;
  }

  default void assignPetOwner(org.bukkit.entity.Tameable pet, UUID owner) {
    throw new IllegalStateException("Official pet ownership is not ready");
  }

  JsonObject session(UUID nativeId) throws Exception;

  default net.kyori.adventure.text.Component text(UUID player, String message) {
    String language = "en";
    try {
      language = CoreClient.string(session(player), "language", "en");
    } catch (Exception ignored) {
    }
    return net.kyori.adventure.text.Component.text(
        com.lkjsxc.lkjmc.common.Messages.error(language, message));
  }

  default boolean mustIsolate(UUID nativeId) {
    return false;
  }

  default boolean quarantined(Location location) {
    return false;
  }

  default JsonObject spawnClaim(Location location) {
    return null;
  }

  default boolean mayRespawn(UUID player, Location location) {
    return !quarantined(location);
  }

  void async(Runnable work);

  default <T> T main(Callable<T> action) throws Exception {
    if (Bukkit.isPrimaryThread()) return action.call();
    try {
      return Bukkit.getScheduler().callSyncMethod(plugin(), action).get(45, TimeUnit.SECONDS);
    } catch (ExecutionException e) {
      if (e.getCause() instanceof Exception cause) throw cause;
      throw e;
    }
  }

  default boolean official() {
    return plugin().getConfig().getString("role", "official").equals("official");
  }
}
