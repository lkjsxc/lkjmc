package com.lkjmc.paper;

import io.papermc.paper.threadedregions.scheduler.ScheduledTask;
import java.time.Duration;
import java.util.concurrent.TimeUnit;
import java.util.function.Consumer;
import org.bukkit.World;
import org.bukkit.entity.Player;
import org.bukkit.plugin.Plugin;

public final class FoliaSchedulerBridge implements SchedulerBridge {
    private final Plugin plugin;
    private final SchedulerTaskRegistry tasks = new SchedulerTaskRegistry();

    public FoliaSchedulerBridge(Plugin plugin) {
        this.plugin = plugin;
    }

    @Override
    public void runPlayer(Player player, Runnable task) {
        tasks.track(player.getScheduler().run(plugin, oneShot(task), null));
    }

    @Override
    public void runPlayerLater(Player player, Runnable task, Duration delay) {
        long ticks = Math.max(1, delay.toMillis() / 50);
        tasks.track(player.getScheduler().runDelayed(plugin, oneShot(task), null, ticks));
    }

    @Override
    public void runAsync(Runnable task) {
        tasks.track(plugin.getServer().getAsyncScheduler().runNow(plugin, oneShot(task)));
    }

    @Override
    public void runGlobal(Runnable task) {
        tasks.track(plugin.getServer().getGlobalRegionScheduler().run(plugin, oneShot(task)));
    }

    @Override
    public void runRegion(World world, int chunkX, int chunkZ, Runnable task) {
        tasks.track(plugin.getServer().getRegionScheduler().run(plugin, world, chunkX, chunkZ, oneShot(task)));
    }

    @Override
    public void runAsyncRepeating(Runnable task, Duration initialDelay, Duration period) {
        tasks.track(plugin.getServer().getAsyncScheduler().runAtFixedRate(
            plugin,
            ignored -> task.run(),
            initialDelay.toMillis(),
            period.toMillis(),
            TimeUnit.MILLISECONDS
        ));
    }

    @Override
    public void cancelAll() {
        tasks.cancelAll();
    }

    private Consumer<ScheduledTask> oneShot(Runnable task) {
        return scheduled -> {
            try {
                task.run();
            } finally {
                tasks.complete(scheduled);
            }
        };
    }
}
