package com.lkjsxc.lkjmc.paper;

import com.google.gson.JsonObject;
import com.lkjsxc.lkjmc.common.*;
import java.nio.file.Path;
import java.util.*;
import org.bukkit.entity.*;
import org.bukkit.event.*;
import org.bukkit.event.block.*;
import org.bukkit.event.entity.*;
import org.bukkit.event.inventory.*;
import org.bukkit.event.player.*;
import org.bukkit.plugin.messaging.PluginMessageListener;

/** The last combat check runs on the same native tick thread as PvP damage. */
public final class DepartureGate implements Listener, PluginMessageListener {
  private final PaperContext ctx;
  private final SignedBridge bridge;
  private final Map<UUID, JsonObject> leaving = new HashMap<>();
  private final Map<UUID, Long> seen = new HashMap<>();

  public DepartureGate(PaperContext ctx, Path key) throws Exception {
    this.ctx = ctx;
    bridge = new SignedBridge(key);
    var messenger = ctx.plugin().getServer().getMessenger();
    messenger.registerIncomingPluginChannel(ctx.plugin(), SignedBridge.CHANNEL, this);
    messenger.registerOutgoingPluginChannel(ctx.plugin(), SignedBridge.CHANNEL);
  }

  public boolean leaving(UUID player) {
    JsonObject request = leaving.get(player);
    if (request == null) return false;
    if (request.get("expires_at").getAsLong() < System.currentTimeMillis()) {
      leaving.remove(player);
      return false;
    }
    return true;
  }

  @Override
  public void onPluginMessageReceived(String channel, Player player, byte[] bytes) {
    try {
      JsonObject request = bridge.decode(bytes);
      UUID nonce = CoreClient.uuid(request, "nonce");
      if (!CoreClient.uuid(request, "native_uuid").equals(player.getUniqueId())
          || !CoreClient.uuid(ctx.session(player.getUniqueId()), "session_id")
              .equals(CoreClient.uuid(request, "session_id"))) return;
      if (request.get("op").getAsString().equals("release")) {
        JsonObject old = leaving.get(player.getUniqueId());
        if (old != null && CoreClient.uuid(old, "nonce").equals(nonce))
          leaving.remove(player.getUniqueId());
        return;
      }
      if (!request.get("op").getAsString().equals("prepare")) return;
      seen.values().removeIf(expiry -> expiry < System.currentTimeMillis());
      if (seen.putIfAbsent(nonce, request.get("expires_at").getAsLong()) != null) return;
      String error = null;
      if (player.isDead()) error = "復活してから移動してください。";
      else if (ctx.inCombat(player.getUniqueId())) error = "PvP直後は30秒間サーバーを移動できません。";
      else if (ctx.mustIsolate(player.getUniqueId()) || ctx.quarantined(player.getLocation()))
        error = "持ち物・建物の保存が終わってから移動してください。";
      else if (leaving(player.getUniqueId())) error = "すでに移動を準備しています。";
      if (error == null) {
        leaving.put(player.getUniqueId(), request);
        player.closeInventory();
        try {
          WorldDurability.flush(List.of(), List.of(player));
        } catch (Exception e) {
          leaving.remove(player.getUniqueId());
          throw e;
        }
      }
      JsonObject response = request.deepCopy();
      response.addProperty("op", "result");
      response.addProperty("allowed", error == null);
      if (error != null) response.addProperty("reason", error);
      player.sendPluginMessage(ctx.plugin(), SignedBridge.CHANNEL, bridge.encode(response));
    } catch (Exception e) {
      ctx.plugin()
          .getLogger()
          .warning("Rejected departure frame or failed save: " + e.getMessage());
    }
  }

  @EventHandler
  public void quit(PlayerQuitEvent e) {
    leaving.remove(e.getPlayer().getUniqueId());
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void move(PlayerMoveEvent e) {
    if (leaving(e.getPlayer().getUniqueId()) && e.hasChangedPosition()) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void teleport(PlayerTeleportEvent e) {
    if (leaving(e.getPlayer().getUniqueId())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void damage(EntityDamageEvent e) {
    if (e.getEntity() instanceof Player p && leaving(p.getUniqueId())) e.setCancelled(true);
    if (e instanceof EntityDamageByEntityEvent hit) {
      Entity source = hit.getDamager();
      if (source instanceof Projectile projectile
          && projectile.getShooter() instanceof Entity shooter) source = shooter;
      if (source instanceof Player p && leaving(p.getUniqueId())) e.setCancelled(true);
    }
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void interact(PlayerInteractEvent e) {
    if (leaving(e.getPlayer().getUniqueId())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void interactEntity(PlayerInteractEntityEvent e) {
    if (leaving(e.getPlayer().getUniqueId())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void place(BlockPlaceEvent e) {
    if (leaving(e.getPlayer().getUniqueId())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void breakBlock(BlockBreakEvent e) {
    if (leaving(e.getPlayer().getUniqueId())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void click(InventoryClickEvent e) {
    if (leaving(e.getWhoClicked().getUniqueId())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void drag(InventoryDragEvent e) {
    if (leaving(e.getWhoClicked().getUniqueId())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void pickup(EntityPickupItemEvent e) {
    if (leaving(e.getEntity().getUniqueId())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void drop(PlayerDropItemEvent e) {
    if (leaving(e.getPlayer().getUniqueId())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void swap(PlayerSwapHandItemsEvent e) {
    if (leaving(e.getPlayer().getUniqueId())) e.setCancelled(true);
  }
}
