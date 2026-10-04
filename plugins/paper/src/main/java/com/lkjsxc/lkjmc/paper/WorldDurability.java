package com.lkjsxc.lkjmc.paper;

import java.nio.channels.FileChannel;
import java.nio.file.*;
import java.util.*;
import org.bukkit.*;
import org.bukkit.entity.Player;

public final class WorldDurability {
  private WorldDurability() {}

  /** World UUIDs and generation settings must be durable before Core registers them. */
  public static void flushWorldIdentities(Collection<World> worlds) throws Exception {
    if (!Bukkit.isPrimaryThread())
      throw new IllegalStateException("world identity flush must run on the server thread");
    if (!Bukkit.dispatchCommand(Bukkit.getConsoleSender(), "minecraft:save-all flush"))
      throw new IllegalStateException("Minecraft world identity flush failed");
    Set<Path> files = new HashSet<>();
    for (World world : worlds) metadata(world, files);
    force(files);
  }

  public static void flush(Collection<Chunk> chunks, Collection<Player> players) throws Exception {
    if (!Bukkit.isPrimaryThread())
      throw new IllegalStateException("world flush must run on the server thread");
    players.forEach(Player::saveData);
    if (!chunks.isEmpty()
        && !Bukkit.dispatchCommand(Bukkit.getConsoleSender(), "minecraft:save-all flush"))
      throw new IllegalStateException("Minecraft flush command failed");
    Set<Path> files = new HashSet<>();
    for (Chunk chunk : chunks) {
      metadata(chunk.getWorld(), files);
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
    force(files);
  }

  private static void metadata(World world, Set<Path> files) throws Exception {
    Path data = world.getWorldPath().resolve("data");
    Path identity = data.resolve("paper/metadata.dat");
    if (!Files.isRegularFile(identity, LinkOption.NOFOLLOW_LINKS))
      throw new IllegalStateException("Native world identity was not saved: " + world.getName());
    // Paper 26.2 stores UUID, seed/settings and other world metadata separately.
    try (var paths = Files.walk(data, 4)) {
      for (Path file :
          paths
              .filter(
                  p ->
                      Files.isRegularFile(p, LinkOption.NOFOLLOW_LINKS)
                          && p.getFileName().toString().endsWith(".dat"))
              .toList()) files.add(file);
    }
    files.add(Bukkit.getServer().getLevelDirectory().resolve("level.dat"));
  }

  private static void force(Collection<Path> files) throws Exception {
    Set<Path> directories = new HashSet<>();
    Path level = Bukkit.getServer().getLevelDirectory().toAbsolutePath().normalize();
    for (Path file : files)
      if (Files.exists(file)) {
        try (FileChannel channel = FileChannel.open(file, StandardOpenOption.WRITE)) {
          channel.force(true);
        }
        for (Path parent = file.toAbsolutePath().normalize().getParent();
            parent != null && parent.startsWith(level);
            parent = parent.getParent()) directories.add(parent);
      }
    for (Path directory : directories)
      try (FileChannel parent = FileChannel.open(directory, StandardOpenOption.READ)) {
        parent.force(true);
      }
  }
}
