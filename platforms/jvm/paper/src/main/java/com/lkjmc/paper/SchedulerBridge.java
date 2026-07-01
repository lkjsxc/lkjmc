package com.lkjmc.paper;

import java.time.Duration;
import org.bukkit.World;
import org.bukkit.entity.Player;

public interface SchedulerBridge {
    void runPlayer(Player player, Runnable task);

    default void runPlayerLater(Player player, Runnable task, Duration delay) {
        runPlayer(player, task);
    }

    void runAsync(Runnable task);

    default void runRegion(World world, int chunkX, int chunkZ, Runnable task) {
        runAsync(task);
    }

    void runAsyncRepeating(Runnable task, Duration initialDelay, Duration period);

    void cancelAll();
}
