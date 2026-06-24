package com.lkjmc.velocity;

import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.format.NamedTextColor;

public final class VelocityMessages {
    private VelocityMessages() {}

    public static Component ok(String text) {
        return Component.text(text, NamedTextColor.GREEN);
    }

    public static Component error(String text) {
        return Component.text(text, NamedTextColor.RED);
    }
}
