package com.lkjsxc.lkjmc.paper;

import com.google.gson.*;
import com.lkjsxc.lkjmc.common.*;
import java.util.*;
import org.bukkit.*;
import org.bukkit.block.Block;
import org.bukkit.entity.*;
import org.bukkit.event.*;
import org.bukkit.event.block.*;
import org.bukkit.event.entity.*;
import org.bukkit.event.hanging.*;
import org.bukkit.event.inventory.*;
import org.bukkit.event.player.*;
import org.bukkit.event.world.PortalCreateEvent;
import org.bukkit.inventory.Inventory;

/** World transactions retain a physical quarantine until Core confirms the durable receipt. */
public final class WorldLocks implements Listener {
  public record Box(String world, int minX, int minY, int minZ, int maxX, int maxY, int maxZ) {
    public boolean contains(Location p) {
      return p != null
          && p.getWorld().getName().equals(world)
          && p.getX() >= minX
          && p.getX() < (double) maxX + 1
          && p.getY() >= minY
          && p.getY() < (double) maxY + 1
          && p.getZ() >= minZ
          && p.getZ() < (double) maxZ + 1;
    }

    public JsonObject json() {
      return CoreClient.object(
          "world", world, "min_x", minX, "min_y", minY, "min_z", minZ, "max_x", maxX, "max_y", maxY,
          "max_z", maxZ);
    }

    public static Box read(JsonObject j) {
      return new Box(
          j.get("world").getAsString(),
          j.get("min_x").getAsInt(),
          j.get("min_y").getAsInt(),
          j.get("min_z").getAsInt(),
          j.get("max_x").getAsInt(),
          j.get("max_y").getAsInt(),
          j.get("max_z").getAsInt());
    }

    public Set<Chunk> chunks() {
      World w = Objects.requireNonNull(Bukkit.getWorld(world));
      Set<Chunk> result = new LinkedHashSet<>();
      for (int x = minX >> 4; x <= maxX >> 4; x++)
        for (int z = minZ >> 4; z <= maxZ >> 4; z++) result.add(w.getChunkAt(x, z));
      return result;
    }
  }

  private final PaperContext ctx;
  private final ClaimProtection claims;
  private final Map<UUID, Box> local = new java.util.concurrent.ConcurrentHashMap<>();

  public WorldLocks(PaperContext ctx, ClaimProtection claims) {
    this.ctx = ctx;
    this.claims = claims;
  }

  public void hold(UUID job, Box box) {
    local.put(job, box);
    for (Chunk c : box.chunks()) c.addPluginChunkTicket(ctx.plugin());
  }

  public void release(UUID job) {
    Box box = local.remove(job);
    if (box == null) return;
    for (Chunk c : box.chunks())
      if (local.values().stream()
          .noneMatch(
              b ->
                  b.world.equals(box.world)
                      && c.getX() >= b.minX >> 4
                      && c.getX() <= b.maxX >> 4
                      && c.getZ() >= b.minZ >> 4
                      && c.getZ() <= b.maxZ >> 4)) c.removePluginChunkTicket(ctx.plugin());
  }

  public boolean locked(Location point) {
    if (local.values().stream().anyMatch(b -> b.contains(point))) return true;
    if (point == null) return false;
    JsonObject claim = claims.claim(point.getBlock());
    if (claim == null) return false;
    if (Set.of("transferring", "releasing", "pending").contains(claim.get("state").getAsString()))
      return true;
    JsonArray assets = ctx.projection().getAsJsonArray("assets");
    if (assets != null)
      for (JsonElement e : assets) {
        JsonObject a = e.getAsJsonObject();
        if (a.has("locked_claim_id") && claim.get("id").equals(a.get("locked_claim_id")))
          return true;
      }
    return false;
  }

  private boolean locked(Block b) {
    return b != null && locked(b.getLocation());
  }

  private boolean locked(Inventory i) {
    return i != null
        && (locked(i.getLocation())
            || i.getHolder() instanceof Entity e && locked(e.getLocation())
            || i instanceof org.bukkit.inventory.MerchantInventory m
                && m.getMerchant() instanceof Entity e
                && locked(e.getLocation()));
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void place(BlockPlaceEvent e) {
    if (locked(e.getBlock())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void breakBlock(BlockBreakEvent e) {
    if (locked(e.getBlock())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void interact(PlayerInteractEvent e) {
    if (locked(e.getClickedBlock())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void interactEntity(PlayerInteractEntityEvent e) {
    if (locked(e.getRightClicked().getLocation())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void damage(EntityDamageEvent e) {
    if (locked(e.getEntity().getLocation())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void hang(HangingBreakEvent e) {
    if (locked(e.getEntity().getLocation())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void hangingPlace(HangingPlaceEvent e) {
    if (locked(e.getEntity().getLocation())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void armor(PlayerArmorStandManipulateEvent e) {
    if (locked(e.getRightClicked().getLocation())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void inventoryOpen(InventoryOpenEvent e) {
    if (locked(e.getInventory())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void inventoryClick(InventoryClickEvent e) {
    if (locked(e.getView().getTopInventory())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void inventoryDrag(InventoryDragEvent e) {
    if (locked(e.getView().getTopInventory())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void hopper(InventoryMoveItemEvent e) {
    if (locked(e.getSource()) || locked(e.getDestination())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void pickup(InventoryPickupItemEvent e) {
    if (locked(e.getInventory()) || locked(e.getItem().getLocation())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void entityPickup(EntityPickupItemEvent e) {
    if (locked(e.getEntity().getLocation()) || locked(e.getItem().getLocation()))
      e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void itemDrop(EntityDropItemEvent e) {
    if (locked(e.getEntity().getLocation())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void itemSpawn(ItemSpawnEvent e) {
    if (locked(e.getLocation())) e.setCancelled(true);
  }

  private void bucket(PlayerBucketEvent e) {
    if (locked(e.getBlock()) || locked(e.getBlockClicked().getRelative(e.getBlockFace())))
      e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void bucketEmpty(PlayerBucketEmptyEvent e) {
    bucket(e);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void bucketFill(PlayerBucketFillEvent e) {
    bucket(e);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void flow(BlockFromToEvent e) {
    if (locked(e.getBlock()) || locked(e.getToBlock())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void physics(BlockPhysicsEvent e) {
    if (locked(e.getBlock()) || locked(e.getSourceBlock())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void extend(BlockPistonExtendEvent e) {
    if (locked(e.getBlock())
        || e.getBlocks().stream()
            .anyMatch(b -> locked(b) || locked(b.getRelative(e.getDirection()))))
      e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void retract(BlockPistonRetractEvent e) {
    if (locked(e.getBlock())
        || e.getBlocks().stream()
            .anyMatch(b -> locked(b) || locked(b.getRelative(e.getDirection().getOppositeFace()))))
      e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void explosion(EntityExplodeEvent e) {
    e.blockList().removeIf(this::locked);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void blockExplosion(BlockExplodeEvent e) {
    e.blockList().removeIf(this::locked);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void burn(BlockBurnEvent e) {
    if (locked(e.getBlock())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void ignite(BlockIgniteEvent e) {
    if (locked(e.getBlock())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void fade(BlockFadeEvent e) {
    if (locked(e.getBlock())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void form(BlockFormEvent e) {
    if (locked(e.getBlock())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void grow(BlockGrowEvent e) {
    if (locked(e.getBlock())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void spread(BlockSpreadEvent e) {
    if (locked(e.getBlock()) || locked(e.getSource())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void structure(org.bukkit.event.world.StructureGrowEvent e) {
    if (e.getBlocks().stream().anyMatch(b -> locked(b.getLocation()))) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void change(EntityChangeBlockEvent e) {
    if (locked(e.getBlock())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void entityMove(io.papermc.paper.event.entity.EntityMoveEvent e) {
    if (locked(e.getFrom()) || locked(e.getTo())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void entityTeleport(EntityTeleportEvent e) {
    if (locked(e.getFrom()) || locked(e.getTo())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void move(PlayerMoveEvent e) {
    if (locked(e.getTo()) && !locked(e.getFrom())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void teleport(PlayerTeleportEvent e) {
    if (locked(e.getTo())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void portal(PortalCreateEvent e) {
    if (e.getBlocks().stream().anyMatch(b -> locked(b.getLocation()))) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void dispense(BlockDispenseEvent e) {
    if (locked(e.getBlock())
        || locked(
            e.getBlock()
                .getRelative(
                    ((org.bukkit.block.data.Directional) e.getBlock().getBlockData()).getFacing())))
      e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void breed(EntityBreedEvent e) {
    if (locked(e.getEntity().getLocation())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void transform(EntityTransformEvent e) {
    if (locked(e.getEntity().getLocation())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void cook(BlockCookEvent e) {
    if (locked(e.getBlock())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void craft(CrafterCraftEvent e) {
    if (locked(e.getBlock())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void fuel(FurnaceBurnEvent e) {
    if (locked(e.getBlock())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void brew(BrewEvent e) {
    if (locked(e.getBlock())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void brewFuel(BrewingStandFuelEvent e) {
    if (locked(e.getBlock())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void restock(VillagerReplenishTradeEvent e) {
    if (locked(e.getEntity().getLocation())) e.setCancelled(true);
  }

  @EventHandler(priority = EventPriority.HIGHEST, ignoreCancelled = true)
  public void creature(CreatureSpawnEvent e) {
    if (e.getSpawnReason() != CreatureSpawnEvent.SpawnReason.CUSTOM && locked(e.getLocation()))
      e.setCancelled(true);
  }
}
