package com.lkjsxc.lkjmc.paper;

import java.nio.channels.FileChannel;
import java.nio.file.*;
import java.util.*;
import org.bukkit.*;
import org.bukkit.entity.Player;

public final class WorldDurability {
  private WorldDurability() {}

  public static void flush(Collection<Chunk> chunks, Collection<Player> players) throws Exception {
    if (!Bukkit.isPrimaryThread())
      throw new IllegalStateException("world flush must run on the server thread");
    players.forEach(Player::saveData);
    if (!chunks.isEmpty()
        && !Bukkit.dispatchCommand(Bukkit.getConsoleSender(), "minecraft:save-all flush"))
      throw new IllegalStateException("Minecraft flush command failed");
    Set<Path> files = new HashSet<>();
    for (Chunk chunk : chunks) {
      Path directory = chunk.getWorld().getWorldPath();
      String region =
          "r." + Math.floorDiv(chunk.getX(), 32) + "." + Math.floorDiv(chunk.getZ(), 32) + ".mca";
      for (String part : List.of("region", "entities", "poi"))
        files.add(directory.resolve(part).resolve(region));
    }
    Path playerRoot = Bukkit.getServer().getLevelDirectory().resolve("players/data");
    for (Player player : players) {
      Path playerFile = playerRoot.resolve(player.getUniqueId() + ".dat");
      if (!Files.isRegularFile(playerFile))
        throw new IllegalStateException("Native inventory was not saved: " + player.getUniqueId());
      files.add(playerFile);
    }
    for (Path file : files)
      if (Files.exists(file)) {
        try (FileChannel channel = FileChannel.open(file, StandardOpenOption.WRITE)) {
          channel.force(true);
        }
        try (FileChannel parent = FileChannel.open(file.getParent(), StandardOpenOption.READ)) {
          parent.force(true);
        }
      }
  }
}
