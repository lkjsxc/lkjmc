package com.lkjsxc.lkjmc.paper;

import com.google.gson.*;
import com.lkjsxc.lkjmc.common.*;
import io.papermc.paper.event.player.AsyncPlayerSpawnLocationEvent;
import java.nio.file.*;
import java.util.*;
import java.util.concurrent.*;
import java.util.concurrent.atomic.AtomicInteger;
import org.bukkit.*;
import org.bukkit.block.Block;
import org.bukkit.entity.Player;
import org.bukkit.event.*;
import org.bukkit.event.block.*;
import org.bukkit.event.entity.*;
import org.bukkit.event.inventory.*;
import org.bukkit.event.player.*;
import org.bukkit.generator.ChunkGenerator;

/**
 * Every fallback resolves to a private holding cell; the living world's default spawn is never
 * used.
 */
public final class SpawnPolicy implements Listener {
  private final PaperContext ctx;
  private final Journal states;
  private final World living, holding;
  private final Map<UUID, JsonObject> players = new ConcurrentHashMap<>();
  private final Set<UUID> searching = ConcurrentHashMap.newKeySet();
  private final Set<UUID> approvedTeleports = ConcurrentHashMap.newKeySet();
  private final AtomicInteger slots = new AtomicInteger();
  private static final Set<Material> HAZARDS =
      Set.of(
          Material.LAVA,
          Material.WATER,
          Material.POWDER_SNOW,
          Material.MAGMA_BLOCK,
          Material.CACTUS,
          Material.CAMPFIRE,
          Material.SOUL_CAMPFIRE,
          Material.FIRE,
          Material.SOUL_FIRE,
          Material.SWEET_BERRY_BUSH);

  public SpawnPolicy(PaperContext ctx, World living, World holding) throws Exception {
    this.ctx = ctx;
    this.living = living;
    this.holding = holding;
    Path directory = ctx.plugin().getDataFolder().toPath().resolve("player-locations");
    states = new Journal(directory);
    try (var files = Files.list(directory)) {
      for (Path file : files.filter(p -> p.getFileName().toString().endsWith(".json")).toList()) {
        UUID id = UUID.fromString(file.getFileName().toString().replace(".json", ""));
        JsonObject value = states.read(id).orElseThrow();
        players.put(id, value);
        slots.accumulateAndGet(value.get("cell").getAsInt() + 1, Math::max);
      }
    }
    holding.setAutoSave(true);
    holding.setGameRule(GameRule.DO_MOB_SPAWNING, false);
    holding.setGameRule(GameRule.DO_DAYLIGHT_CYCLE, false);
    holding.setGameRule(GameRule.DO_WEATHER_CYCLE, false);
    holding.setTime(6000);
    holding.setDifficulty(Difficulty.PEACEFUL);
  }

  private synchronized JsonObject state(UUID id) throws Exception {
    JsonObject existing = players.get(id);
    if (existing != null) return existing.deepCopy();
    var value =
        CoreClient.object(
            "cell", slots.getAndIncrement(), "phase", "needs_random", "reason", "first_join");
    states.write(id, value);
    players.put(id, value);
    return value.deepCopy();
  }

  private synchronized void save(UUID id, JsonObject value) throws Exception {
    states.write(id, value);
    players.put(id, value.deepCopy());
  }

  public synchronized void reloadIdentity(UUID id) throws Exception {
    if (Bukkit.getPlayer(id) != null)
      throw new IllegalStateException("Cannot replace an online spawn state");
    players.remove(id);
    Optional<JsonObject> saved = states.read(id);
    if (saved.isPresent()) {
      players.put(id, saved.get());
      slots.accumulateAndGet(saved.get().get("cell").getAsInt() + 1, Math::max);
    }
  }

  public static JsonObject location(Location loc) {
    return CoreClient.object(
        "world",
        loc.getWorld().getName(),
        "x",
        loc.getX(),
        "y",
        loc.getY(),
        "z",
        loc.getZ(),
        "yaw",
        loc.getYaw(),
        "pitch",
        loc.getPitch());
  }

  public Location decode(JsonObject point) {
    World world = Bukkit.getWorld(point.get("world").getAsString());
    if (world == null) return null;
    return new Location(
        world,
        point.get("x").getAsDouble(),
        point.get("y").getAsDouble(),
        point.get("z").getAsDouble(),
        point.get("yaw").getAsFloat(),
        point.get("pitch").getAsFloat());
  }

  public Location cell(UUID id) throws Exception {
    int index = state(id).get("cell").getAsInt();
    if (index >= 1_000_000) throw new IllegalStateException("Holding capacity exhausted");
    int x = (index % 1000) * 1024 + 512, z = (index / 1000) * 1024 + 512;
    return ctx.main(
        () -> {
          for (int dx = -2; dx <= 2; dx++)
            for (int dz = -2; dz <= 2; dz++) {
              holding.getBlockAt(x + dx, 100, z + dz).setType(Material.BEDROCK, false);
              for (int dy = 101; dy <= 104; dy++)
                holding
                    .getBlockAt(x + dx, dy, z + dz)
                    .setType(
                        Math.abs(dx) == 2 || Math.abs(dz) == 2 ? Material.GLASS : Material.AIR,
                        false);
            }
          return new Location(holding, x + .5, 101, z + .5);
        });
  }

  @EventHandler(priority = EventPriority.HIGHEST)
  public void firstAppearance(AsyncPlayerSpawnLocationEvent event) {
    UUID id = event.getConnection().getProfile().getId();
    try {
      JsonObject state = state(id);
      if (!ctx.mustIsolate(id)
          && state.get("phase").getAsString().equals("known")
          && state.has("location")) {
        Location destination = ctx.main(() -> decode(state.getAsJsonObject("location")));
        if (destination != null
            && !destination.getWorld().equals(holding)
            && ctx.main(
                () ->
                    allowedWorld(destination.getWorld())
                        && AdventureTransactions.permits(ctx, id, destination.getWorld())
                        && !ctx.quarantined(destination))) {
          event.setSpawnLocation(destination);
          return;
        }
        if (state.has("location")
            && state
                .getAsJsonObject("location")
                .get("world")
                .getAsString()
                .startsWith("adventure_")) {
          state.addProperty("phase", "needs_return");
          state.addProperty("reason", "adventure_closed");
          state.remove("location");
          save(id, state);
        }
      }
      // Resolving the first point may involve generation. Appearance happens only after this event.
      event.setSpawnLocation(cell(id));
    } catch (Exception e) {
      ctx.plugin().getLogger().severe("Unable to isolate joining player: " + e.getMessage());
      event
          .getConnection()
          .disconnect(
              ctx.text(
                  event.getConnection().getProfile().getId(),
                  "Your starting area could not be prepared. Wait a moment and reconnect."));
    }
  }

  private boolean allowedWorld(World world) {
    if (world.getName().startsWith("adventure_")) {
      String id = world.getName().substring("adventure_".length());
      boolean active = false;
      for (JsonElement entry : ctx.projection().getAsJsonArray("adventures")) {
        JsonObject a = entry.getAsJsonObject();
        if (a.get("id").getAsString().equals(id)
            && a.get("state").getAsString().equals("active")
            && java.time.Instant.parse(a.get("expires_at").getAsString())
                .isAfter(java.time.Instant.now())) active = true;
      }
      if (!active) return false;
    }
    for (JsonElement element : ctx.projection().getAsJsonArray("worlds")) {
      JsonObject item = element.getAsJsonObject();
      if (item.get("name").getAsString().equals(world.getName()))
        return item.get("enabled").getAsBoolean()
            && !item.get("kind").getAsString().equals("holding");
    }
    return false;
  }

  @EventHandler(priority = EventPriority.MONITOR, ignoreCancelled = true)
  public void spawnSet(com.destroystokyo.paper.event.player.PlayerSetSpawnEvent event) {
    try {
      JsonObject state = state(event.getPlayer().getUniqueId());
      Location at = event.getLocation();
      if (at == null) state.remove("respawn_binding");
      else {
        JsonObject claim = ctx.spawnClaim(at);
        state.add(
            "respawn_binding",
            CoreClient.object(
                "location",
                location(at),
                "invalidated",
                false,
                "claim_id",
                claim == null ? null : claim.get("id"),
                "claim_job",
                claim == null ? null : claim.get("job_id")));
      }
      save(event.getPlayer().getUniqueId(), state);
    } catch (Exception e) {
      failClosed(event.getPlayer(), e);
    }
  }

  public void invalidateRespawns(WorldLocks.Box box) throws Exception {
    for (UUID id : List.copyOf(players.keySet())) {
      JsonObject state = state(id);
      JsonObject binding = state.getAsJsonObject("respawn_binding");
      if (binding == null || binding.get("invalidated").getAsBoolean()) continue;
      Location at = decode(binding.getAsJsonObject("location"));
      if (box.contains(at)) {
        binding.addProperty("invalidated", true);
        save(id, state);
      }
    }
  }

  private boolean validRespawn(Player player, Location destination) {
    if (!ctx.mayRespawn(player.getUniqueId(), destination)) return false;
    try {
      JsonObject binding = state(player.getUniqueId()).getAsJsonObject("respawn_binding");
      if (binding == null) return true;
      if (binding.get("invalidated").getAsBoolean()) return false;
      Location original = decode(binding.getAsJsonObject("location"));
      if (!ctx.mayRespawn(player.getUniqueId(), original)) return false;
      if (binding.has("claim_id") && !binding.get("claim_id").isJsonNull()) {
        JsonObject claim = ctx.spawnClaim(original);
        if (claim == null
            || !claim.get("id").equals(binding.get("claim_id"))
            || !claim.get("job_id").equals(binding.get("claim_job"))) return false;
      }
      return true;
    } catch (Exception e) {
      return false;
    }
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void portal(PlayerPortalEvent event) {
    if (event.getFrom().getWorld().equals(holding)
        || ctx.inCombat(event.getPlayer().getUniqueId())) {
      event.setCancelled(true);
      return;
    }
    if (event.getCause() == PlayerTeleportEvent.TeleportCause.END_PORTAL) {
      if (event.getFrom().getWorld().getEnvironment() == World.Environment.THE_END) {
        teleported(event);
        return;
      }
      World end = Bukkit.getWorld("living_the_end");
      if (end == null) {
        event.setCancelled(true);
        return;
      }
      event.setTo(new Location(end, 100.5, 50, 0.5));
      return;
    }
    if (event.getCause() == PlayerTeleportEvent.TeleportCause.NETHER_PORTAL) {
      boolean returning = event.getFrom().getWorld().getEnvironment() == World.Environment.NETHER;
      World target = returning ? living : Bukkit.getWorld("living_nether");
      if (target == null) {
        event.setCancelled(true);
        return;
      }
      Location from = event.getFrom();
      double scale = returning ? 8 : 0.125;
      event.setTo(
          new Location(
              target,
              Math.clamp(from.getX() * scale, -29_999_872, 29_999_872),
              Math.clamp(from.getY(), target.getMinHeight() + 8, target.getMaxHeight() - 8),
              Math.clamp(from.getZ() * scale, -29_999_872, 29_999_872),
              from.getYaw(),
              from.getPitch()));
    }
  }

  @EventHandler(priority = EventPriority.MONITOR)
  public void joined(PlayerJoinEvent event) {
    Player player = event.getPlayer();
    if (player.getWorld().equals(holding)) {
      isolate(player);
      if (!ctx.mustIsolate(player.getUniqueId())) {
        try {
          if (CoreClient.string(state(player.getUniqueId()), "reason", "")
              .equals("adventure_closed"))
            Bukkit.getScheduler()
                .runTask(
                    ctx.plugin(),
                    () -> {
                      try {
                        returnFromEnd(player, "adventure_closed");
                      } catch (Exception e) {
                        failClosed(player, e);
                      }
                    });
          else search(player.getUniqueId());
        } catch (Exception e) {
          failClosed(player, e);
        }
      }
    } else {
      player.setInvulnerable(false);
      if (players.get(player.getUniqueId()).has("pending_used")) search(player.getUniqueId());
    }
  }

  private void isolate(Player player) {
    player.setInvulnerable(true);
    for (Player other : Bukkit.getOnlinePlayers())
      if (!other.equals(player)) {
        player.hidePlayer(ctx.plugin(), other);
        other.hidePlayer(ctx.plugin(), player);
      }
    player.sendMessage(
        ctx.text(
            player.getUniqueId(),
            "Preparing a safe starting area. You are alone in this waiting area."));
  }

  @EventHandler(priority = EventPriority.MONITOR)
  public void death(PlayerDeathEvent event) {
    UUID id = event.getEntity().getUniqueId();
    try {
      JsonObject state = state(id);
      state.addProperty("phase", "needs_random");
      state.addProperty("reason", "death");
      state.remove("location");
      save(id, state);
    } catch (Exception e) {
      failClosed(event.getEntity(), e);
    }
  }

  @EventHandler(priority = EventPriority.HIGHEST)
  public void respawn(PlayerRespawnEvent event) {
    Player player = event.getPlayer();
    try {
      if ((event.isBedSpawn() || event.isAnchorSpawn())
          && !event.isMissingRespawnBlock()
          && allowedWorld(event.getRespawnLocation().getWorld())
          && validRespawn(player, event.getRespawnLocation())) {
        remember(player.getUniqueId(), event.getRespawnLocation());
        return;
      }
      JsonObject state = state(player.getUniqueId());
      state.addProperty("phase", "needs_random");
      state.addProperty(
          "reason",
          event.getRespawnReason() == PlayerRespawnEvent.RespawnReason.END_PORTAL
              ? "end_portal"
              : "death");
      state.remove("location");
      save(player.getUniqueId(), state);
      event.setRespawnLocation(cell(player.getUniqueId()));
      Bukkit.getScheduler()
          .runTask(
              ctx.plugin(),
              () -> {
                isolate(player);
                search(player.getUniqueId());
              });
    } catch (Exception e) {
      // This player already owns a cell from its login. Never share an emergency spawn.
      int index = players.get(player.getUniqueId()).get("cell").getAsInt();
      event.setRespawnLocation(
          new Location(holding, (index % 1000) * 1024 + 512.5, 101, (index / 1000) * 1024 + 512.5));
      failClosed(player, e);
    }
  }

  @EventHandler(priority = EventPriority.MONITOR)
  public void quit(PlayerQuitEvent event) {
    Player player = event.getPlayer();
    if (player.isDead() || player.getWorld().equals(holding)) return;
    try {
      remember(player.getUniqueId(), player.getLocation());
    } catch (Exception e) {
      ctx.plugin().getLogger().severe("Unable to save last position: " + e.getMessage());
    }
  }

  public void remember(UUID id, Location location) throws Exception {
    if (location.getWorld().equals(holding)) return;
    JsonObject state = state(id);
    state.addProperty("phase", "known");
    state.add("location", location(location));
    save(id, state);
  }

  public void invalidateWorld(String name) throws Exception {
    for (UUID id : List.copyOf(players.keySet())) {
      JsonObject value = state(id);
      if (value.has("location")
          && value.getAsJsonObject("location").get("world").getAsString().equals(name)) {
        value.addProperty("phase", "needs_return");
        value.addProperty("reason", "adventure_closed");
        value.remove("location");
        save(id, value);
      }
    }
  }

  public void returnFromEnd(Player player, String reason) throws Exception {
    Location bed = player.getRespawnLocation();
    if (bed != null
        && allowedWorld(bed.getWorld())
        && validRespawn(player, bed)
        && bed.getBlock().isPassable()
        && bed.clone().add(0, 1, 0).getBlock().isPassable()) teleport(player, bed);
    else fallback(player, reason);
    WorldDurability.flush(List.of(), List.of(player));
  }

  public void search(UUID nativeId) {
    if (!searching.add(nativeId)) return;
    ctx.async(
        () -> {
          try {
            JsonObject session = ctx.session(nativeId);
            UUID account = CoreClient.uuid(session, "account_id");
            JsonObject state = state(nativeId);
            if (state.has("pending_used")) {
              ctx.core()
                  .post(
                      "/internal/v1/spawn/resolve",
                      CoreClient.object(
                          "id",
                          state.get("pending_used").getAsString(),
                          "account_id",
                          account,
                          "state",
                          "used"));
              state.remove("pending_used");
              save(nativeId, state);
            }
            if (CoreClient.string(state, "phase", "").equals("known") && state.has("location")) {
              JsonObject known = state.getAsJsonObject("location");
              boolean resumed =
                  ctx.main(
                      () -> {
                        Location saved = decode(known);
                        if (saved == null
                            || !allowedWorld(saved.getWorld())
                            || !AdventureTransactions.permits(ctx, nativeId, saved.getWorld()))
                          return false;
                        if (ctx.quarantined(saved))
                          throw new IllegalStateException(
                              "Waiting for building and land saves to finish.");
                        Player player = Bukkit.getPlayer(nativeId);
                        if (player != null && player.getWorld().equals(holding))
                          teleport(player, saved);
                        return true;
                      });
              if (resumed) return;
            }
            Location destination = null;
            JsonObject reservation = null;
            for (int attempt = 0; attempt < 16; attempt++) {
              reservation =
                  ctx.core()
                      .post(
                          "/internal/v1/spawn/reserve",
                          CoreClient.object(
                              "account_id",
                              account,
                              "reason",
                              CoreClient.string(state, "reason", "recovery")));
              int x = reservation.get("x").getAsInt(), z = reservation.get("z").getAsInt();
              living
                  .getChunkAtAsync(Math.floorDiv(x, 16), Math.floorDiv(z, 16), true)
                  .get(60, TimeUnit.SECONDS);
              destination = ctx.main(() -> safeSurface(x, z));
              if (destination != null) break;
              ctx.core()
                  .post(
                      "/internal/v1/spawn/resolve",
                      CoreClient.object(
                          "id",
                          reservation.get("id").getAsString(),
                          "account_id",
                          account,
                          "state",
                          "rejected"));
            }
            if (destination == null)
              throw new IllegalStateException("Still searching for safe ground.");
            String spawnId = reservation.get("id").getAsString();
            ctx.core()
                .post(
                    "/internal/v1/spawn/resolve",
                    CoreClient.object(
                        "id",
                        spawnId,
                        "account_id",
                        account,
                        "state",
                        "ready",
                        "y",
                        destination.getBlockY()));
            state.add("location", location(destination));
            state.addProperty("phase", "known");
            state.addProperty("pending_used", spawnId);
            save(nativeId, state);
            ctx.core()
                .post(
                    "/internal/v1/spawn/resolve",
                    CoreClient.object("id", spawnId, "account_id", account, "state", "used"));
            state.remove("pending_used");
            save(nativeId, state);
            Location target = destination;
            ctx.main(
                () -> {
                  Player player = Bukkit.getPlayer(nativeId);
                  if (player != null && player.getWorld().equals(holding)) {
                    teleport(player, target);
                    player.sendMessage(
                        ctx.text(
                            player.getUniqueId(),
                            "You have arrived at your first starting point."));
                  }
                  return null;
                });
          } catch (Exception e) {
            ctx.plugin().getLogger().warning("Spawn remains isolated: " + e.getMessage());
            Bukkit.getScheduler()
                .runTaskLater(
                    ctx.plugin(),
                    () -> {
                      Player player = Bukkit.getPlayer(nativeId);
                      if (player != null && player.getWorld().equals(holding)) search(nativeId);
                    },
                    100);
          } finally {
            searching.remove(nativeId);
          }
        });
  }

  private Location safeSurface(int x, int z) {
    int y = living.getHighestBlockYAt(x, z, HeightMap.MOTION_BLOCKING_NO_LEAVES) + 1;
    if (y <= living.getMinHeight() + 1 || y + 2 >= living.getMaxHeight()) return null;
    for (int dx = -1; dx <= 1; dx++)
      for (int dz = -1; dz <= 1; dz++) {
        Block ground = living.getBlockAt(x + dx, y - 1, z + dz);
        Block feet = living.getBlockAt(x + dx, y, z + dz),
            head = living.getBlockAt(x + dx, y + 1, z + dz);
        if (!ground.getType().isSolid()
            || HAZARDS.contains(ground.getType())
            || !feet.isPassable()
            || !head.isPassable()
            || feet.isLiquid()
            || head.isLiquid()
            || HAZARDS.contains(feet.getType())
            || HAZARDS.contains(head.getType())) return null;
      }
    Location target = new Location(living, x + .5, y, z + .5);
    return living.getWorldBorder().isInside(target) ? target : null;
  }

  public void teleport(Player player, Location destination) throws Exception {
    if (!allowedWorld(destination.getWorld())
        || !AdventureTransactions.permits(ctx, player.getUniqueId(), destination.getWorld()))
      throw new IllegalArgumentException("You cannot travel to this world.");
    approvedTeleports.add(player.getUniqueId());
    try {
      if (!player.teleport(destination, PlayerTeleportEvent.TeleportCause.PLUGIN))
        throw new IllegalStateException("Travel was cancelled.");
      player.setInvulnerable(false);
      remember(player.getUniqueId(), destination);
      player.saveData();
      for (Player other : Bukkit.getOnlinePlayers())
        if (!other.getWorld().equals(holding)) {
          player.showPlayer(ctx.plugin(), other);
          other.showPlayer(ctx.plugin(), player);
        }
    } finally {
      approvedTeleports.remove(player.getUniqueId());
    }
  }

  public void fallback(Player player, String reason) throws Exception {
    JsonObject state = state(player.getUniqueId());
    state.addProperty("phase", "needs_random");
    state.addProperty("reason", reason);
    state.remove("location");
    save(player.getUniqueId(), state);
    approvedTeleports.add(player.getUniqueId());
    try {
      player.teleport(cell(player.getUniqueId()), PlayerTeleportEvent.TeleportCause.PLUGIN);
    } finally {
      approvedTeleports.remove(player.getUniqueId());
    }
    isolate(player);
    search(player.getUniqueId());
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void moved(PlayerMoveEvent event) {
    if (event.getPlayer().getWorld().equals(holding)
        && event.hasChangedPosition()
        && !approvedTeleports.contains(event.getPlayer().getUniqueId())) event.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void teleported(PlayerTeleportEvent event) {
    if (approvedTeleports.contains(event.getPlayer().getUniqueId())) return;
    if (event.getTo() != null
        && !AdventureTransactions.permits(
            ctx, event.getPlayer().getUniqueId(), event.getTo().getWorld())) {
      event.setCancelled(true);
      return;
    }
    if (ctx.inCombat(event.getPlayer().getUniqueId())) {
      event.setCancelled(true);
      return;
    }
    if (event.getPlayer().getWorld().equals(holding)) {
      event.setCancelled(true);
      return;
    }
    if (event.getCause() == PlayerTeleportEvent.TeleportCause.END_PORTAL
        && event.getFrom().getWorld().getEnvironment() == World.Environment.THE_END) {
      Location bed = event.getPlayer().getRespawnLocation();
      if (bed != null
          && allowedWorld(bed.getWorld())
          && validRespawn(event.getPlayer(), bed)
          && bed.getBlock().isPassable()
          && bed.clone().add(0, 1, 0).getBlock().isPassable()) {
        event.setTo(bed);
        try {
          remember(event.getPlayer().getUniqueId(), bed);
        } catch (Exception e) {
          event.setCancelled(true);
          failClosed(event.getPlayer(), e);
        }
      } else {
        event.setCancelled(true);
        try {
          fallback(event.getPlayer(), "end_portal");
        } catch (Exception e) {
          failClosed(event.getPlayer(), e);
        }
      }
    }
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void damage(EntityDamageEvent event) {
    if (event.getEntity() instanceof Player p && p.getWorld().equals(holding))
      event.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void drop(PlayerDropItemEvent event) {
    if (event.getPlayer().getWorld().equals(holding)) event.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void interact(PlayerInteractEvent event) {
    if (event.getPlayer().getWorld().equals(holding)) event.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void inventory(InventoryClickEvent event) {
    if (event.getWhoClicked().getWorld().equals(holding)) event.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void drag(InventoryDragEvent event) {
    if (event.getWhoClicked().getWorld().equals(holding)) event.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void breakBlock(BlockBreakEvent event) {
    if (event.getBlock().getWorld().equals(holding)) event.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void placeBlock(BlockPlaceEvent event) {
    if (event.getBlock().getWorld().equals(holding)) event.setCancelled(true);
  }

  private void failClosed(Player player, Exception e) {
    ctx.plugin().getLogger().severe("Spawn state failure: " + e.getMessage());
    player.kick(
        ctx.text(
            player.getUniqueId(),
            "Your starting point could not be saved safely. Contact an administrator."));
  }

  public static final class VoidGenerator extends ChunkGenerator {
    @Override
    public boolean shouldGenerateNoise() {
      return false;
    }

    @Override
    public boolean shouldGenerateSurface() {
      return false;
    }

    @Override
    public boolean shouldGenerateCaves() {
      return false;
    }

    @Override
    public boolean shouldGenerateDecorations() {
      return false;
    }

    @Override
    public boolean shouldGenerateMobs() {
      return false;
    }

    @Override
    public boolean shouldGenerateStructures() {
      return false;
    }
  }
}
