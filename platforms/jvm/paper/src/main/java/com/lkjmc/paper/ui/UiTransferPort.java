package com.lkjmc.paper.ui;

import org.bukkit.entity.Player;

@FunctionalInterface
public interface UiTransferPort {
    void transfer(Player player, String target);
}
