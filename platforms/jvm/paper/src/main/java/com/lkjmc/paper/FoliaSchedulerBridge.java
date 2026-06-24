package com.lkjmc.paper;

import io.papermc.paper.threadedregions.scheduler.ScheduledTask;
import java.time.Duration;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.TimeUnit;
import org.bukkit.entity.Player;
import org.bukkit.plugin.Plugin;

public final class FoliaSchedulerBridge implements SchedulerBridge {
    private final Plugin plugin;
    private final List<ScheduledTask> tasks = new ArrayList<>();

    public FoliaSchedulerBridge(Plugin plugin) {
        this.plugin = plugin;
    }

    @Override
    public void runPlayer(Player player, Runnable task) {
        tasks.add(player.getScheduler().run(plugin, ignored -> task.run(), null));
    }

    @Override
    public void runAsync(Runnable task) {
        tasks.add(plugin.getServer().getAsyncScheduler().runNow(plugin, ignored -> task.run()));
    }

    @Override
    public void runAsyncRepeating(Runnable task, Duration initialDelay, Duration period) {
        tasks.add(plugin.getServer().getAsyncScheduler().runAtFixedRate(
            plugin,
            ignored -> task.run(),
            initialDelay.toMillis(),
            period.toMillis(),
            TimeUnit.MILLISECONDS
        ));
    }

    @Override
    public void cancelAll() {
        for (ScheduledTask task : List.copyOf(tasks)) {
            task.cancel();
        }
        tasks.clear();
    }
}
