package com.lkjmc.paper;

import com.lkjmc.common.scheduler.PaperScheduler;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionStage;
import org.bukkit.Bukkit;
import org.bukkit.World;
import org.bukkit.entity.Player;
import org.bukkit.plugin.Plugin;

public final class PaperSchedulerBridge implements PaperScheduler {
    private final Plugin plugin;
    private final boolean folia;

    public PaperSchedulerBridge(Plugin plugin) {
        this.plugin = plugin;
        this.folia = classPresent("io.papermc.paper.threadedregions.RegionizedServer");
    }

    @Override
    public CompletionStage<Void> mainOrGlobal(Runnable action) {
        return submit(done -> {
            if (folia) plugin.getServer().getGlobalRegionScheduler().execute(plugin, done);
            else Bukkit.getScheduler().runTask(plugin, done);
        }, action);
    }

    @Override
    public CompletionStage<Void> entity(UUID playerId, Runnable action) {
        Player player = Bukkit.getPlayer(playerId);
        if (player == null) return CompletableFuture.failedFuture(new IllegalStateException("player unavailable"));
        return submit(done -> {
            if (folia) player.getScheduler().execute(plugin, done, null, 1L);
            else Bukkit.getScheduler().runTask(plugin, done);
        }, action);
    }

    @Override
    public CompletionStage<Void> region(String worldName, int chunkX, int chunkZ, Runnable action) {
        World world = Bukkit.getWorld(worldName);
        if (world == null) return CompletableFuture.failedFuture(new IllegalStateException("world unavailable"));
        return submit(done -> {
            if (folia) plugin.getServer().getRegionScheduler().execute(plugin, world, chunkX, chunkZ, done);
            else Bukkit.getScheduler().runTask(plugin, done);
        }, action);
    }

    @Override
    public CompletionStage<Void> async(Runnable action) {
        return submit(done -> Bukkit.getScheduler().runTaskAsynchronously(plugin, done), action);
    }

    public boolean folia() {
        return folia;
    }

    private CompletionStage<Void> submit(
            java.util.function.Consumer<Runnable> dispatcher,
            Runnable action) {
        CompletableFuture<Void> result = new CompletableFuture<>();
        try {
            dispatcher.accept(() -> {
                try {
                    action.run();
                    result.complete(null);
                } catch (RuntimeException failure) {
                    result.completeExceptionally(failure);
                }
            });
        } catch (RuntimeException failure) {
            result.completeExceptionally(failure);
        }
        return result;
    }

    private boolean classPresent(String name) {
        try {
            Class.forName(name, false, plugin.getClass().getClassLoader());
            return true;
        } catch (ClassNotFoundException absent) {
            return false;
        }
    }
}
