package com.lkjmc.velocity;

import static org.junit.jupiter.api.Assertions.assertEquals;

import net.kyori.adventure.text.Component;
import org.junit.jupiter.api.Test;

final class LocalVelocityPresentationTest {
    @Test
    void motdUsesFallbackOnlyForBlankInput() {
        var adapter = new VelocityMotdAdapter();
        assertEquals(Component.text("lkjmc"), adapter.render(null));
        assertEquals(Component.text("lkjmc"), adapter.render("  "));
        assertEquals(Component.text("local network"), adapter.render("local network"));
    }

    @Test
    void tabListTextUsesOnlyTheLocalPlayerCount() {
        var adapter = new VelocityTabListAdapter(null);
        assertEquals(Component.text("lkjmc players: 3"), adapter.header(3));
        assertEquals(Component.text("managed by lkjmc"), adapter.footer());
    }
}
