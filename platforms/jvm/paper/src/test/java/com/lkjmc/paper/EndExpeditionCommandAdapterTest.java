package com.lkjmc.paper;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;

import java.lang.reflect.Proxy;
import java.util.UUID;
import org.bukkit.entity.Player;
import org.junit.jupiter.api.Test;

final class EndExpeditionCommandAdapterTest {
    @Test
    void direct_start_forwards_an_unconfirmed_end_purchase() {
        var player = (Player) Proxy.newProxyInstance(getClass().getClassLoader(), new Class<?>[] {Player.class},
            (ignored, method, args) -> switch (method.getName()) {
                case "getUniqueId" -> UUID.fromString("00000000-0000-0000-0000-000000000001");
                case "getName" -> "Alex";
                default -> null;
            });

        var body = EndExpeditionCommandAdapter.purchaseBody(player, false);

        assertEquals("adventure.end.purchase", EndExpeditionCommandAdapter.purchaseCommand());
        assertFalse(body.containsKey("acceptMinecraftEula"));
    }
}
