package com.lkjmc.velocity;

import net.kyori.adventure.text.Component;

public final class VelocityTabListAdapter {
    public Component header(int onlinePlayers) {
        return Component.text("lkjmc players: " + onlinePlayers);
    }

    public Component footer() {
        return Component.text("managed by lkjmc");
    }
}
