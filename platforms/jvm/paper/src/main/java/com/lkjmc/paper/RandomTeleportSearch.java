package com.lkjmc.paper;

import com.lkjmc.common.menu.RandomTeleportQuote;
import java.util.Optional;
import java.util.concurrent.ThreadLocalRandom;
import java.util.function.Consumer;
import org.bukkit.HeightMap;
import org.bukkit.Location;
import org.bukkit.Material;
import org.bukkit.World;
import org.bukkit.block.Block;
import org.bukkit.entity.Player;

final class RandomTeleportSearch {
    private final LkjmcPaperPlugin plugin;

    RandomTeleportSearch(LkjmcPaperPlugin plugin) {
        this.plugin = plugin;
    }

    void find(Player player, RandomTeleportQuote quote, Consumer<Optional<Location>> done) {
        var world = targetWorld(quote).orElse(player.getWorld());
        attempt(player, world, quote, 0, done);
    }

    private void attempt(Player player, World world, RandomTeleportQuote quote, int attempt,
                         Consumer<Optional<Location>> done) {
        if (attempt >= quote.maxAttempts() || !player.isOnline()) {
            done.accept(Optional.empty());
            return;
        }
        var candidate = candidate(origin(player, world), quote);
        var blockX = candidate.getBlockX();
        var blockZ = candidate.getBlockZ();
        var chunkX = Math.floorDiv(blockX, 16);
        var chunkZ = Math.floorDiv(blockZ, 16);
        world.getChunkAtAsync(chunkX, chunkZ, true).whenComplete((chunk, error) -> {
            if (error != null) {
                attempt(player, world, quote, attempt + 1, done);
                return;
            }
            plugin.scheduler().runRegion(world, chunkX, chunkZ, () -> {
                var safe = safeAt(world, blockX, blockZ);
                if (safe.isPresent()) {
                    done.accept(safe);
                } else {
                    attempt(player, world, quote, attempt + 1, done);
                }
            });
        });
    }

    private Optional<World> targetWorld(RandomTeleportQuote quote) {
        var environment = switch (quote.targetEnvironment()) {
            case "nether" -> World.Environment.NETHER;
            case "the_end" -> World.Environment.THE_END;
            default -> World.Environment.NORMAL;
        };
        return plugin.getServer().getWorlds().stream()
            .filter(world -> world.getEnvironment() == environment)
            .findFirst();
    }

    private static Location origin(Player player, World world) {
        var base = player.getLocation();
        return new Location(world, base.getX(), base.getY(), base.getZ(), base.getYaw(), base.getPitch());
    }

    private static Location candidate(Location origin, RandomTeleportQuote quote) {
        var random = ThreadLocalRandom.current();
        var angle = random.nextDouble(0.0, Math.PI * 2.0);
        var radius = random.nextDouble(quote.minRadius(), quote.maxRadius() + 1.0);
        var x = origin.getX() + Math.cos(angle) * radius;
        var z = origin.getZ() + Math.sin(angle) * radius;
        return new Location(origin.getWorld(), x, origin.getY(), z, origin.getYaw(), origin.getPitch());
    }

    private static Optional<Location> safeAt(World world, int x, int z) {
        var y = world.getHighestBlockYAt(x, z, HeightMap.MOTION_BLOCKING_NO_LEAVES);
        if (y <= world.getMinHeight() + 1 || y >= world.getMaxHeight() - 2) {
            return Optional.empty();
        }
        var floor = world.getBlockAt(x, y - 1, z);
        var feet = world.getBlockAt(x, y, z);
        var head = world.getBlockAt(x, y + 1, z);
        var target = new Location(world, x + 0.5, y, z + 0.5);
        if (!world.getWorldBorder().isInside(target) || !floor.getType().isSolid()) {
            return Optional.empty();
        }
        if (!feet.isPassable() || !head.isPassable() || hazardous(floor) || hazardous(feet)) {
            return Optional.empty();
        }
        return adjacentHazard(world, x, y, z) ? Optional.empty() : Optional.of(target);
    }

    private static boolean adjacentHazard(World world, int x, int y, int z) {
        return hazardous(world.getBlockAt(x + 1, y - 1, z))
            || hazardous(world.getBlockAt(x - 1, y - 1, z))
            || hazardous(world.getBlockAt(x, y - 1, z + 1))
            || hazardous(world.getBlockAt(x, y - 1, z - 1));
    }

    private static boolean hazardous(Block block) {
        var type = block.getType();
        return type == Material.LAVA || type == Material.FIRE || type == Material.SOUL_FIRE
            || type == Material.MAGMA_BLOCK || type == Material.CACTUS || type == Material.POWDER_SNOW;
    }
}
