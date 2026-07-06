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
    private static final int ATTEMPTS_PER_BATCH = 4;

    private final LkjmcPaperPlugin plugin;

    RandomTeleportSearch(LkjmcPaperPlugin plugin) {
        this.plugin = plugin;
    }

    void find(Player player, RandomTeleportQuote quote, Consumer<Optional<Location>> done) {
        var world = targetWorld(player, quote);
        var maxAttempts = Math.max(0, quote.maxAttempts());
        if (world.isEmpty() || maxAttempts == 0) {
            done.accept(Optional.empty());
            return;
        }
        attempt(player, world.get(), quote, new SearchState(origin(player, world.get()), maxAttempts, 0, done));
    }

    private void attempt(Player player, World world, RandomTeleportQuote quote, SearchState state) {
        if (state.attemptsUsed() >= state.maxAttempts() || !player.isOnline()) {
            state.done().accept(Optional.empty());
            return;
        }
        var candidate = candidate(state.origin(), quote);
        var blockX = candidate.getBlockX();
        var blockZ = candidate.getBlockZ();
        var chunkX = Math.floorDiv(blockX, 16);
        var chunkZ = Math.floorDiv(blockZ, 16);
        var next = state.withAttempts(state.attemptsUsed() + 1);
        world.getChunkAtAsync(chunkX, chunkZ, true).whenComplete((chunk, error) -> {
            if (error != null) {
                continueAfterFailure(player, world, quote, next, chunkX, chunkZ, true);
                return;
            }
            plugin.scheduler().runRegion(world, chunkX, chunkZ, () -> {
                if (!player.isOnline()) {
                    state.done().accept(Optional.empty());
                    return;
                }
                var safe = safeAt(world, blockX, blockZ);
                if (safe.isPresent()) {
                    state.done().accept(safe);
                } else {
                    continueAfterFailure(player, world, quote, next, chunkX, chunkZ, false);
                }
            });
        });
    }

    private void continueAfterFailure(Player player, World world, RandomTeleportQuote quote,
                                      SearchState state, int chunkX, int chunkZ, boolean forceYield) {
        if (state.attemptsUsed() >= state.maxAttempts() || !player.isOnline()) {
            state.done().accept(Optional.empty());
            return;
        }
        if (forceYield || state.attemptsUsed() % ATTEMPTS_PER_BATCH == 0) {
            plugin.scheduler().runRegion(world, chunkX, chunkZ, () -> attempt(player, world, quote, state));
        } else {
            attempt(player, world, quote, state);
        }
    }

    private Optional<World> targetWorld(Player player, RandomTeleportQuote quote) {
        var environment = switch (quote.targetEnvironment()) {
            case "nether" -> World.Environment.NETHER;
            case "the_end" -> World.Environment.THE_END;
            default -> World.Environment.NORMAL;
        };
        if (player.getWorld().getEnvironment() == environment) {
            return Optional.of(player.getWorld());
        }
        return plugin.getServer().getWorlds().stream().filter(world -> world.getEnvironment() == environment).findFirst();
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

    private record SearchState(Location origin, int maxAttempts, int attemptsUsed,
                               Consumer<Optional<Location>> done) {
        SearchState withAttempts(int attempts) {
            return new SearchState(origin, maxAttempts, attempts, done);
        }
    }
}
