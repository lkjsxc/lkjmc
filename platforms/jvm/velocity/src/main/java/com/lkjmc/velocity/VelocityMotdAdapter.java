package com.lkjmc.velocity;

import net.kyori.adventure.text.Component;

public final class VelocityMotdAdapter {
    public Component render(String motd) {
        return Component.text(motd == null || motd.isBlank() ? "lkjmc" : motd);
    }
}
