package com.lkjmc.paper.ui;

import com.lkjmc.common.ui.kernel.MenuRoute;
import com.lkjmc.paper.SchedulerBridge;
import org.bukkit.entity.Player;

public final class UiEntrypoints {
    private final SchedulerBridge scheduler;
    private final UiSessionService sessions;

    public UiEntrypoints(SchedulerBridge scheduler, UiSessionService sessions) {
        this.scheduler = scheduler;
        this.sessions = sessions;
    }

    public void openRoot(Player player) {
        scheduler.runPlayer(player, () -> sessions.openRoot(player));
    }

    public void openHotbar(Player player) {
        openRoot(player);
    }

    public void openDeep(Player player, MenuRoute route) {
        scheduler.runPlayer(player, () -> sessions.openFromRoot(player, route));
    }
}
