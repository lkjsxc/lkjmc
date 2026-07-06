package com.lkjmc.paper.ui;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;

import com.lkjmc.common.ui.document.DocumentAction;
import com.lkjmc.common.ui.document.ItemRole;
import com.lkjmc.common.ui.kernel.FrameSlot;
import com.lkjmc.common.ui.kernel.MenuRoute;
import com.lkjmc.common.ui.kernel.TextRef;
import java.util.List;
import java.util.Map;
import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.TextComponent;
import org.bukkit.Material;
import org.junit.jupiter.api.Test;

final class UiRendererTest {
    @Test
    void mapsFrameSlotToMaterialComponentsAndStampedMetadata() {
        var renderer = new UiRenderer(UiTestFixtures.docs(), new UiMetadataCodec("lkjmc"), UiTestFixtures.text());
        var slot = FrameSlot.action(4, "DIAMOND", TextRef.key("item.name"),
            List.of(TextRef.key("item.lore")), ItemRole.ACTION,
            new DocumentAction.Command("say hello"), Map.of()).stamped(new MenuRoute("root"), "s1", 7);

        var item = renderer.mapped("en", slot);

        assertEquals(Material.DIAMOND, item.material());
        assertEquals("Open", plain(item.name()));
        assertEquals(List.of("Lore"), item.lore().stream().map(UiRendererTest::plain).toList());
        assertFalse(item.inert());
        assertEquals("command:say hello", item.metadata().actionKey());
        assertEquals("s1", item.metadata().sessionId());
        assertEquals(7, item.metadata().epoch());
    }

    private static String plain(Component component) {
        var value = new StringBuilder();
        append(component, value);
        return value.toString();
    }

    private static void append(Component component, StringBuilder value) {
        if (component instanceof TextComponent text) {
            value.append(text.content());
        }
        component.children().forEach(child -> append(child, value));
    }
}
