package com.lkjsxc.lkjmc.paper;

import java.util.*;
import org.bukkit.*;
import org.bukkit.block.Block;
import org.bukkit.block.BlockFace;
import org.bukkit.entity.FallingBlock;
import org.bukkit.event.*;
import org.bukkit.event.block.*;
import org.bukkit.event.entity.EntityChangeBlockEvent;
import org.bukkit.event.world.StructureGrowEvent;
import org.bukkit.persistence.PersistentDataType;

/** Chunk-owned bitsets are saved alongside the blocks they describe, from day one. */
public final class Provenance implements Listener {
  private final NamespacedKey key;
  private final NamespacedKey fallingKey;

  public Provenance(org.bukkit.plugin.Plugin plugin) {
    key = new NamespacedKey(plugin, "built-v1");
    fallingKey = new NamespacedKey(plugin, "falling-built-v1");
  }

  private int index(Block block) {
    return ((block.getY() - block.getWorld().getMinHeight()) * 256)
        + ((block.getZ() & 15) * 16)
        + (block.getX() & 15);
  }

  private BitSet read(Chunk chunk) {
    byte[] bytes = chunk.getPersistentDataContainer().get(key, PersistentDataType.BYTE_ARRAY);
    return bytes == null ? new BitSet() : BitSet.valueOf(bytes);
  }

  public boolean built(Block block) {
    return read(block.getChunk()).get(index(block));
  }

  public void mark(Block block, boolean value) {
    BitSet bits = read(block.getChunk());
    bits.set(index(block), value);
    block
        .getChunk()
        .getPersistentDataContainer()
        .set(key, PersistentDataType.BYTE_ARRAY, bits.toByteArray());
  }

  @EventHandler(priority = EventPriority.MONITOR, ignoreCancelled = true)
  public void placed(BlockPlaceEvent event) {
    mark(event.getBlockPlaced(), true);
  }

  @EventHandler(priority = EventPriority.MONITOR, ignoreCancelled = true)
  public void multiple(BlockMultiPlaceEvent event) {
    event.getReplacedBlockStates().forEach(state -> mark(state.getBlock(), true));
  }

  @EventHandler(priority = EventPriority.MONITOR, ignoreCancelled = true)
  public void broken(BlockBreakEvent event) {
    mark(event.getBlock(), false);
  }

  @EventHandler(priority = EventPriority.MONITOR, ignoreCancelled = true)
  public void burned(BlockBurnEvent event) {
    mark(event.getBlock(), false);
  }

  @EventHandler(priority = EventPriority.MONITOR, ignoreCancelled = true)
  public void faded(BlockFadeEvent event) {
    mark(event.getBlock(), false);
  }

  @EventHandler(priority = EventPriority.MONITOR, ignoreCancelled = true)
  public void grown(BlockGrowEvent event) {
    mark(event.getBlock(), false);
  }

  @EventHandler(priority = EventPriority.MONITOR, ignoreCancelled = true)
  public void spread(BlockSpreadEvent event) {
    mark(event.getBlock(), false);
  }

  @EventHandler(priority = EventPriority.MONITOR, ignoreCancelled = true)
  public void tree(StructureGrowEvent event) {
    event.getBlocks().forEach(state -> mark(state.getBlock(), false));
  }

  @EventHandler(priority = EventPriority.MONITOR, ignoreCancelled = true)
  public void exploded(BlockExplodeEvent event) {
    event.blockList().forEach(block -> mark(block, false));
  }

  @EventHandler(priority = EventPriority.MONITOR, ignoreCancelled = true)
  public void entityExploded(org.bukkit.event.entity.EntityExplodeEvent event) {
    event.blockList().forEach(block -> mark(block, false));
  }

  private void move(List<Block> blocks, BlockFace direction) {
    Map<Block, Boolean> values = new LinkedHashMap<>();
    for (Block block : blocks) values.put(block, built(block));
    values.keySet().forEach(block -> mark(block, false));
    values.forEach((block, built) -> mark(block.getRelative(direction), built));
  }

  @EventHandler(priority = EventPriority.MONITOR, ignoreCancelled = true)
  public void extend(BlockPistonExtendEvent event) {
    move(event.getBlocks(), event.getDirection());
    mark(event.getBlock().getRelative(event.getDirection()), built(event.getBlock()));
  }

  @EventHandler(priority = EventPriority.MONITOR, ignoreCancelled = true)
  public void retract(BlockPistonRetractEvent event) {
    mark(event.getBlock().getRelative(event.getDirection()), false);
    move(event.getBlocks(), event.getDirection().getOppositeFace());
  }

  @EventHandler(priority = EventPriority.MONITOR, ignoreCancelled = true)
  public void falling(EntityChangeBlockEvent event) {
    if (event.getEntity() instanceof FallingBlock entity) {
      if (event.getTo().isAir()) {
        entity
            .getPersistentDataContainer()
            .set(fallingKey, PersistentDataType.BYTE, (byte) (built(event.getBlock()) ? 1 : 0));
        mark(event.getBlock(), false);
      } else {
        mark(
            event.getBlock(),
            entity
                    .getPersistentDataContainer()
                    .getOrDefault(fallingKey, PersistentDataType.BYTE, (byte) 0)
                == 1);
      }
    } else mark(event.getBlock(), false);
  }
}
