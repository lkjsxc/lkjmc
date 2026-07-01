package com.lkjmc.paper;

import static org.junit.jupiter.api.Assertions.assertEquals;

import org.bukkit.Material;
import org.junit.jupiter.api.Test;

final class DocsCommandAdapterTest {
    @Test
    void mainMenuReturnUsesNetherStar() {
        assertEquals(Material.NETHER_STAR, DocsCommandAdapter.MAIN_MENU_MATERIAL);
    }
}
