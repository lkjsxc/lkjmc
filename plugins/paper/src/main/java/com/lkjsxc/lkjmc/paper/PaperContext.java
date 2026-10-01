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

  JsonObject session(UUID nativeId) throws Exception;

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
