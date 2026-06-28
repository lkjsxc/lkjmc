package com.lkjmc.paper;

import static org.junit.jupiter.api.Assertions.assertEquals;

import org.bukkit.Material;
import org.junit.jupiter.api.Test;

final class HotbarMenuTokenServiceTest {
    @Test
    void slotEightTokenUsesNetherStarAndPersistentMarker() {
        assertEquals(8, HotbarMenuTokenService.SLOT);
        assertEquals(Material.NETHER_STAR, HotbarMenuTokenService.TOKEN_MATERIAL);
        assertEquals("menu_item", HotbarMenuTokenService.MARKER_KEY);
    }
}
