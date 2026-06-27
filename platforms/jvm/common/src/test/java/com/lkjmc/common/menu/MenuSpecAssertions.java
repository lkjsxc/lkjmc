package com.lkjmc.common.menu;

import static org.junit.jupiter.api.Assertions.assertEquals;

final class MenuSpecAssertions {
    private MenuSpecAssertions() {}

    static MenuAction actionAt(MenuSpec spec, int slot) {
        return slotAt(spec, slot).action();
    }

    static void assertSlot(MenuSpec spec, int slot, String key) {
        assertEquals(key, slotAt(spec, slot).item().nameKey());
    }

    static SlotSpec slotAt(MenuSpec spec, int slot) {
        return spec.slots().stream().filter(value -> value.slot() == slot).findFirst().orElseThrow();
    }
}
