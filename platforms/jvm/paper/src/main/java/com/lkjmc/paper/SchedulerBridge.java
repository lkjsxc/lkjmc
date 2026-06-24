package com.lkjmc.paper;

import java.time.Duration;
import org.bukkit.entity.Player;

public interface SchedulerBridge {
    void runPlayer(Player player, Runnable task);

    void runAsync(Runnable task);

    void runAsyncRepeating(Runnable task, Duration initialDelay, Duration period);

    void cancelAll();
}
