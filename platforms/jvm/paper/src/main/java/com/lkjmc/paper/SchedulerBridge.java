package com.lkjmc.paper;

import org.bukkit.entity.Player;

public interface SchedulerBridge {
    void runPlayer(Player player, Runnable task);

    void runAsync(Runnable task);

    void cancelAll();
}
