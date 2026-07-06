package com.lkjmc.paper.ui;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.lkjmc.common.ui.kernel.MenuMetadata;
import com.lkjmc.common.ui.kernel.MenuRoute;
import java.lang.reflect.Proxy;
import java.util.HashMap;
import java.util.Map;
import org.bukkit.NamespacedKey;
import org.bukkit.persistence.PersistentDataContainer;
import org.junit.jupiter.api.Test;

final class UiMetadataCodecTest {
    @Test
    void roundTripsMetadataThroughPersistentDataContainer() {
        var codec = new UiMetadataCodec("lkjmc");
        var pdc = pdc();
        var metadata = new MenuMetadata(new MenuRoute("shop", Map.of("category", "blocks & tools")),
            Map.of("category", "blocks & tools"), 13, "daemon:player.shop.purchase",
            Map.of("type", "daemon", "body.itemId", "stone/slab"), "session-1", 42);

        codec.write(pdc, metadata);

        assertTrue(codec.hasAny(pdc));
        assertEquals(metadata, codec.read(pdc));
    }

    @Test
    void malformedContainerReturnsNull() {
        var codec = new UiMetadataCodec("lkjmc");
        var pdc = pdc();
        pdc.set(new NamespacedKey("lkjmc", "ui_marker"), org.bukkit.persistence.PersistentDataType.BYTE, (byte) 1);
        assertNull(codec.read(pdc));
    }

    private static PersistentDataContainer pdc() {
        var values = new HashMap<NamespacedKey, Object>();
        return (PersistentDataContainer) Proxy.newProxyInstance(
            PersistentDataContainer.class.getClassLoader(), new Class<?>[] {PersistentDataContainer.class},
            (proxy, method, args) -> switch (method.getName()) {
                case "set" -> { values.put((NamespacedKey) args[0], args[2]); yield null; }
                case "get" -> values.get(args[0]);
                case "has" -> values.containsKey(args[0]);
                case "remove" -> { values.remove(args[0]); yield null; }
                case "isEmpty" -> values.isEmpty();
                case "getKeys" -> values.keySet();
                default -> UiTestFixtures.fallback(method.getReturnType());
            });
    }
}
