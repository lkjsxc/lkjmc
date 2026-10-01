package com.lkjsxc.lkjmc.paper;

import com.google.gson.*;
import com.lkjsxc.lkjmc.common.*;
import java.time.Instant;
import java.util.*;
import java.util.concurrent.*;
import org.bukkit.*;
import org.bukkit.block.data.Ageable;
import org.bukkit.entity.*;
import org.bukkit.event.*;
import org.bukkit.event.block.*;
import org.bukkit.event.entity.EntityDamageByEntityEvent;
import org.bukkit.event.player.*;

/** Official-only events, durable retry IDs, and immediate local combat restrictions. */
public final class GameEvents implements Listener, AutoCloseable {
  private final PaperContext ctx;
  private final Journal outbox;
  private final Map<UUID, Long> combat = new ConcurrentHashMap<>();
  private final Map<UUID, Double> walking = new HashMap<>();
  private final ScheduledExecutorService sender = Executors.newSingleThreadScheduledExecutor();

  public GameEvents(PaperContext ctx) throws Exception {
    this.ctx = ctx;
    outbox = new Journal(ctx.plugin().getDataFolder().toPath().resolve("event-outbox"));
    sender.scheduleWithFixedDelay(this::drain, 1, 2, TimeUnit.SECONDS);
  }

  public boolean inCombat(UUID id) {
    return combat.getOrDefault(id, 0L) > System.nanoTime();
  }

  private boolean survival(Player player) {
    return player.getGameMode() == GameMode.SURVIVAL
        && !player.getWorld().getName().equals("holding");
  }

  private void record(Player player, String kind, JsonObject payload) {
    try {
      JsonObject session = ctx.session(player.getUniqueId());
      UUID id = UUID.randomUUID();
      JsonObject event =
          CoreClient.object(
              "id",
              id,
              "account_id",
              session.get("account_id").getAsString(),
              "session_id",
              session.get("session_id").getAsString(),
              "occurred_at",
              Instant.now().toString(),
              "kind",
              kind,
              "payload",
              payload);
      sender.execute(
          () -> {
            try {
              outbox.write(id, CoreClient.object("id", id, "phase", "pending", "event", event));
            } catch (Exception e) {
              ctx.plugin()
                  .getLogger()
                  .log(java.util.logging.Level.SEVERE, "Cannot save game event", e);
              Bukkit.getScheduler()
                  .runTask(
                      ctx.plugin(),
                      () ->
                          player.kick(
                              net.kyori.adventure.text.Component.text("実績を安全に保存できません。再接続してください。")));
            }
          });
    } catch (Exception e) {
      ctx.plugin().getLogger().warning("Cannot identify official event: " + e.getMessage());
    }
  }

  private void drain() {
    try {
      for (JsonObject row : outbox.unfinished()) {
        ctx.core().post("/internal/v1/game/event", row.getAsJsonObject("event"));
        row.addProperty("phase", "committed");
        outbox.write(CoreClient.uuid(row, "id"), row);
      }
    } catch (Exception e) {
      ctx.plugin().getLogger().warning("Official event awaits retry: " + e.getMessage());
    }
  }

  @EventHandler(priority = EventPriority.MONITOR, ignoreCancelled = true)
  public void placed(BlockPlaceEvent e) {
    if (survival(e.getPlayer()))
      record(e.getPlayer(), "block.placed", CoreClient.object("amount", 1));
  }

  @EventHandler(priority = EventPriority.MONITOR, ignoreCancelled = true)
  public void crop(BlockBreakEvent e) {
    if (survival(e.getPlayer())
        && e.getBlock().getBlockData() instanceof Ageable crop
        && crop.getAge() == crop.getMaximumAge())
      record(e.getPlayer(), "crop.harvest", CoreClient.object("amount", 1));
  }

  @EventHandler(priority = EventPriority.MONITOR, ignoreCancelled = true)
  public void walk(PlayerMoveEvent e) {
    if (!survival(e.getPlayer())
        || !e.hasChangedPosition()
        || !e.getFrom().getWorld().equals(e.getTo().getWorld())) return;
    double moved = e.getFrom().distance(e.getTo());
    if (moved > 10) return;
    UUID id = e.getPlayer().getUniqueId();
    double total = walking.getOrDefault(id, 0.0) + moved;
    if (total >= 10) {
      int amount = (int) Math.min(1000, total);
      record(e.getPlayer(), "walk.distance", CoreClient.object("amount", amount));
      total -= amount;
    }
    walking.put(id, total);
  }

  @EventHandler(priority = EventPriority.MONITOR, ignoreCancelled = true)
  public void hit(EntityDamageByEntityEvent e) {
    if (!(e.getEntity() instanceof Player target) || e.getFinalDamage() <= 0) return;
    Entity source = e.getDamager();
    Player attacker =
        source instanceof Player p
            ? p
            : source instanceof Projectile projectile && projectile.getShooter() instanceof Player p
                ? p
                : null;
    if (attacker == null || attacker.equals(target)) return;
    long until = System.nanoTime() + TimeUnit.SECONDS.toNanos(30);
    combat.put(attacker.getUniqueId(), until);
    combat.put(target.getUniqueId(), until);
    try {
      record(
          attacker,
          "combat",
          CoreClient.object(
              "target", ctx.session(target.getUniqueId()).get("account_id").getAsString()));
    } catch (Exception error) {
      ctx.plugin().getLogger().warning("Combat event needs reconciliation: " + error.getMessage());
    }
  }

  @Override
  public void close() {
    sender.shutdown();
  }
}
