package com.lkjsxc.lkjmc.paper;

import com.google.gson.JsonObject;
import com.lkjsxc.lkjmc.common.CoreClient;
import java.util.List;
import java.util.UUID;
import org.bukkit.Location;
import org.bukkit.World;
import org.bukkit.entity.Player;

/** Per-participant origins survive entry, logout, world deletion and return crashes. */
final class ExpeditionReturns {
  private final PaperContext ctx;
  private final SpawnPolicy spawns;
  private final World living;

  ExpeditionReturns(PaperContext ctx, SpawnPolicy spawns, World living) {
    this.ctx = ctx;
    this.spawns = spawns;
    this.living = living;
  }

  JsonObject capture(Player player, UUID expedition, UUID entryJob) throws Exception {
    if (player.getWorld().getName().startsWith("adventure_"))
      throw new IllegalArgumentException("Return to the SMP before entering an expedition again.");
    if (!spawns.allowedWorld(player.getWorld()))
      throw new IllegalArgumentException("Enter an expedition from a playable SMP world.");
    JsonObject context =
        CoreClient.object(
            "expedition_id",
            expedition,
            "entry_job",
            entryJob,
            "origin",
            SpawnPolicy.location(player.getLocation()),
            "phase",
            "inside");
    JsonObject state = spawns.state(player.getUniqueId());
    state.add("expedition_return", context);
    spawns.save(player.getUniqueId(), state);
    return context.deepCopy();
  }

  boolean entryApplied(Player player, UUID job, UUID expedition) throws Exception {
    JsonObject context = spawns.state(player.getUniqueId()).getAsJsonObject("expedition_return");
    return context != null
        && CoreClient.string(context, "entry_job", "").equals(job.toString())
        && player.getWorld().getName().equals("adventure_" + expedition);
  }

  boolean entryRecorded(UUID player, UUID job) throws Exception {
    JsonObject context = spawns.state(player).getAsJsonObject("expedition_return");
    return context != null
        && CoreClient.string(context, "entry_job", "").equals(job.toString())
        && context.has("entry_applied")
        && context.get("entry_applied").getAsBoolean();
  }

  void recordEntry(Player player, UUID job) throws Exception {
    JsonObject state = spawns.state(player.getUniqueId());
    JsonObject context = state.getAsJsonObject("expedition_return");
    if (context == null || !CoreClient.string(context, "entry_job", "").equals(job.toString()))
      throw new IllegalStateException("The expedition entry origin record has changed.");
    context.addProperty("entry_applied", true);
    spawns.save(player.getUniqueId(), state);
  }

  boolean pending(UUID player) throws Exception {
    JsonObject state = spawns.state(player);
    JsonObject context = state.getAsJsonObject("expedition_return");
    return context != null && CoreClient.string(context, "phase", "").equals("return_pending");
  }

  void invalidate(JsonObject state, String world) {
    JsonObject context = state.getAsJsonObject("expedition_return");
    // A pre-upgrade run has no captured origin. Preserve its existing native
    // journals and explicitly use the safe bed/spawn fallback exactly once.
    if (context == null) {
      context =
          CoreClient.object(
              "expedition_id",
              world.substring("adventure_".length()),
              "migrated_origin_unknown",
              true);
      state.add("expedition_return", context);
    }
    context.addProperty("phase", "return_pending");
  }

  Location destination(Player player) throws Exception {
    JsonObject context = spawns.state(player.getUniqueId()).getAsJsonObject("expedition_return");
    if (context != null && context.has("origin")) {
      Location origin = spawns.decode(context.getAsJsonObject("origin"));
      if (safe(player, origin)) return origin;
    }
    Location bed = player.getRespawnLocation();
    if (bed != null && spawns.validRespawn(player, bed)) {
      if (safe(player, bed)) return bed;
      // A configured bed point can refer to the occupied bed block itself.
      // Resolve a safe adjacent standing position without modifying the world.
      for (int dx = -2; dx <= 2; dx++)
        for (int dz = -2; dz <= 2; dz++) {
          Location beside = bed.clone();
          beside.setX(bed.getBlockX() + dx + .5);
          beside.setZ(bed.getBlockZ() + dz + .5);
          if (safe(player, beside)) return beside;
        }
    }
    Location spawn = living.getSpawnLocation();
    if (safe(player, spawn)) return spawn;
    // Search only near the established SMP spawn. Expiry must never allocate
    // a new random starting point or move someone ten kilometres away.
    for (int radius = 0; radius <= 64; radius += 8)
      for (int dx = -radius; dx <= radius; dx += 8)
        for (int dz = -radius; dz <= radius; dz += 8) {
          if (Math.max(Math.abs(dx), Math.abs(dz)) != radius) continue;
          Location candidate = spawns.safeSurface(spawn.getBlockX() + dx, spawn.getBlockZ() + dz);
          if (safe(player, candidate)) return candidate;
        }
    throw new IllegalStateException("Waiting for a safe expedition return point in the SMP.");
  }

  private boolean safe(Player player, Location at) {
    return at != null
        && !at.getWorld().getName().startsWith("adventure_")
        && spawns.allowedWorld(at.getWorld())
        && spawns.safeStanding(at)
        && ctx.mayRespawn(player.getUniqueId(), at);
  }

  void returnToSmp(Player player) throws Exception {
    JsonObject state = spawns.state(player.getUniqueId());
    JsonObject context = state.getAsJsonObject("expedition_return");
    if (context == null) {
      String name = player.getWorld().getName();
      if (!name.startsWith("adventure_"))
        throw new IllegalArgumentException("You are already outside the expedition.");
      invalidate(state, name);
      context = state.getAsJsonObject("expedition_return");
    }
    context.addProperty("phase", "return_pending");
    state.addProperty("phase", "needs_return");
    state.addProperty("reason", "adventure_closed");
    spawns.save(player.getUniqueId(), state);
    Location at = destination(player);
    spawns.teleport(player, at);
    WorldDurability.flush(List.of(), List.of(player));
    state = spawns.state(player.getUniqueId());
    state.getAsJsonObject("expedition_return").addProperty("phase", "returned");
    spawns.save(player.getUniqueId(), state);
  }
}
