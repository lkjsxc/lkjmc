package com.lkjsxc.lkjmc.paper;

import com.google.gson.*;
import com.lkjsxc.lkjmc.common.Journal;
import com.sk89q.worldedit.bukkit.BukkitAdapter;
import com.sk89q.worldedit.math.BlockVector3;
import com.sk89q.worldguard.WorldGuard;
import com.sk89q.worldguard.protection.flags.*;
import com.sk89q.worldguard.protection.regions.*;
import java.util.*;
import org.bukkit.*;
import org.bukkit.block.Block;
import org.bukkit.entity.*;
import org.bukkit.event.*;
import org.bukkit.event.block.*;
import org.bukkit.event.entity.EntityDamageByEntityEvent;
import org.bukkit.event.entity.EntityPlaceEvent;
import org.bukkit.event.inventory.InventoryMoveItemEvent;
import org.bukkit.event.player.PlayerInteractEntityEvent;
import org.bukkit.event.world.PortalCreateEvent;
import org.bukkit.inventory.Inventory;
import org.bukkit.persistence.PersistentDataType;

/** The database is authoritative; UUID membership is materialized in WorldGuard. */
public final class ClaimProtection implements Listener {
  private final PaperContext ctx;
  private String applied = "";
  private final Set<UUID> releasing = new HashSet<>();
  private final NamespacedKey entityOwner;

  public ClaimProtection(PaperContext ctx) {
    this.ctx = ctx;
    entityOwner = new NamespacedKey(ctx.plugin(), "placed-by-account");
  }

  public void release(UUID id) throws Exception {
    releasing.add(id);
    applied = "";
    apply();
  }

  public void apply() throws Exception {
    JsonArray claims = ctx.projection().getAsJsonArray("claims");
    if (claims == null) return;
    String digest = Journal.digest(claims) + releasing;
    if (digest.equals(applied)) return;
    for (World world : Bukkit.getWorlds()) {
      var manager =
          WorldGuard.getInstance()
              .getPlatform()
              .getRegionContainer()
              .get(BukkitAdapter.adapt(world));
      if (manager == null)
        throw new IllegalStateException(
            "WorldGuard region manager is unavailable: " + world.getName());
      Set<String> present = new HashSet<>();
      for (JsonElement element : claims) {
        JsonObject claim = element.getAsJsonObject();
        UUID id = UUID.fromString(claim.get("id").getAsString());
        if (releasing.contains(id) || !worldId(world).equals(claim.get("world_id").getAsString()))
          continue;
        String name = "lkjmc_" + id.toString().replace("-", "");
        present.add(name);
        var region =
            new ProtectedCuboidRegion(
                name,
                BlockVector3.at(
                    claim.get("min_x").getAsInt() * 16,
                    world.getMinHeight(),
                    claim.get("min_z").getAsInt() * 16),
                BlockVector3.at(
                    claim.get("max_x").getAsInt() * 16 + 15,
                    world.getMaxHeight() - 1,
                    claim.get("max_z").getAsInt() * 16 + 15));
        if (!claim.get("native_uuid").isJsonNull())
          region.getOwners().addPlayer(UUID.fromString(claim.get("native_uuid").getAsString()));
        for (JsonElement member : claim.getAsJsonArray("members")) {
          JsonObject m = member.getAsJsonObject();
          if (m.get("can_build").getAsBoolean() && !m.get("native_uuid").isJsonNull())
            region.getMembers().addPlayer(UUID.fromString(m.get("native_uuid").getAsString()));
        }
        region.setPriority(100);
        region.setFlag(Flags.PVP, StateFlag.State.DENY);
        region.setFlag(Flags.CREEPER_EXPLOSION, StateFlag.State.DENY);
        region.setFlag(Flags.WITHER_DAMAGE, StateFlag.State.DENY);
        region.setFlag(Flags.GHAST_FIREBALL, StateFlag.State.DENY);
        region.setFlag(Flags.ENDER_BUILD, StateFlag.State.DENY);
        region.setFlag(Flags.TNT, StateFlag.State.DENY);
        region.setFlag(Flags.FIRE_SPREAD, StateFlag.State.DENY);
        region.setFlag(Flags.LAVA_FIRE, StateFlag.State.DENY);
        manager.addRegion(region);
      }
      for (String name : new ArrayList<>(manager.getRegions().keySet()))
        if (name.startsWith("lkjmc_") && !present.contains(name)) manager.removeRegion(name);
      manager.save();
    }
    applied = digest;
  }

  public String worldId(World world) {
    for (JsonElement entry : ctx.projection().getAsJsonArray("worlds"))
      if (entry.getAsJsonObject().get("name").getAsString().equals(world.getName()))
        return entry.getAsJsonObject().get("id").getAsString();
    return "";
  }

  public JsonObject claim(Block block) {
    String world = worldId(block.getWorld());
    for (JsonElement element : ctx.projection().getAsJsonArray("claims")) {
      JsonObject c = element.getAsJsonObject();
      int x = block.getX() >> 4, z = block.getZ() >> 4;
      if (c.get("world_id").getAsString().equals(world)
          && x >= c.get("min_x").getAsInt()
          && x <= c.get("max_x").getAsInt()
          && z >= c.get("min_z").getAsInt()
          && z <= c.get("max_z").getAsInt()) return c;
    }
    return null;
  }

  public boolean canBuild(UUID account, Block block) {
    JsonObject c = claim(block);
    if (c == null) return false;
    if (c.get("owner").getAsString().equals(account.toString())) return true;
    for (JsonElement element : c.getAsJsonArray("members")) {
      JsonObject m = element.getAsJsonObject();
      if (m.get("account_id").getAsString().equals(account.toString())
          && m.get("can_build").getAsBoolean()) return true;
    }
    return false;
  }

  private boolean sameOwnership(Block from, Block to) {
    JsonObject a = claim(from), b = claim(to);
    return a == null ? b == null : b != null && a.get("owner").equals(b.get("owner"));
  }

  private UUID account(Player player) {
    try {
      return UUID.fromString(ctx.session(player.getUniqueId()).get("account_id").getAsString());
    } catch (Exception e) {
      return new UUID(0, 0);
    }
  }

  @EventHandler(priority = EventPriority.MONITOR, ignoreCancelled = true)
  public void entityPlaced(EntityPlaceEvent event) {
    if (event.getPlayer() != null)
      event
          .getEntity()
          .getPersistentDataContainer()
          .set(entityOwner, PersistentDataType.STRING, account(event.getPlayer()).toString());
  }

  private boolean authorizedInventory(Inventory inventory) {
    Location location = inventory.getLocation();
    if (location == null || claim(location.getBlock()) == null) return true;
    if (inventory.getHolder() instanceof Entity entity) {
      String owner =
          entity.getPersistentDataContainer().get(entityOwner, PersistentDataType.STRING);
      return owner != null && canBuild(UUID.fromString(owner), location.getBlock());
    }
    return true;
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void hopper(InventoryMoveItemEvent event) {
    Location from = event.getSource().getLocation(), to = event.getDestination().getLocation();
    if (from != null
        && to != null
        && (!sameOwnership(from.getBlock(), to.getBlock())
            || !authorizedInventory(event.getSource())
            || !authorizedInventory(event.getDestination()))) event.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void useEntity(PlayerInteractEntityEvent event) {
    Block at = event.getRightClicked().getLocation().getBlock();
    if (!(event.getRightClicked() instanceof Player)
        && claim(at) != null
        && !canBuild(account(event.getPlayer()), at)) event.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void damageEntity(EntityDamageByEntityEvent event) {
    if (event.getEntity() instanceof Player) return;
    Entity source = event.getDamager();
    Player player =
        source instanceof Player p
            ? p
            : source instanceof Projectile projectile && projectile.getShooter() instanceof Player p
                ? p
                : null;
    Block at = event.getEntity().getLocation().getBlock();
    if (player != null && claim(at) != null && !canBuild(account(player), at))
      event.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void portalCreated(PortalCreateEvent event) {
    for (var state : event.getBlocks())
      if (claim(state.getBlock()) != null
          && (!(event.getEntity() instanceof Player p)
              || !canBuild(account(p), state.getBlock()))) {
        event.setCancelled(true);
        break;
      }
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void fluid(BlockFromToEvent e) {
    if (!sameOwnership(e.getBlock(), e.getToBlock())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void extend(BlockPistonExtendEvent e) {
    if (!sameOwnership(e.getBlock(), e.getBlock().getRelative(e.getDirection())))
      e.setCancelled(true);
    for (Block b : e.getBlocks())
      if (!sameOwnership(e.getBlock(), b) || !sameOwnership(b, b.getRelative(e.getDirection())))
        e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void retract(BlockPistonRetractEvent e) {
    for (Block b : e.getBlocks())
      if (!sameOwnership(e.getBlock(), b)
          || !sameOwnership(b, b.getRelative(e.getDirection().getOppositeFace())))
        e.setCancelled(true);
  }
}
