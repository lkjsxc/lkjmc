package com.lkjmc.common.menu;

import java.util.HashSet;
import java.util.List;

public record MenuSpec(MenuId id, MenuTitle title, MenuSize size, List<SlotSpec> slots) {
    public MenuSpec {
        if (id == null || title == null || size == null) {
            throw new IllegalArgumentException("menu id, title, and size are required");
        }
        slots = List.copyOf(slots == null ? List.of() : slots);
        var occupied = new HashSet<Integer>();
        for (SlotSpec slot : slots) {
            if (slot.slot() >= size.slots()) {
                throw new IllegalArgumentException("slot exceeds menu size: " + slot.slot());
            }
            if (!occupied.add(slot.slot())) {
                throw new IllegalArgumentException("duplicate menu slot: " + slot.slot());
            }
        }
    }
}
