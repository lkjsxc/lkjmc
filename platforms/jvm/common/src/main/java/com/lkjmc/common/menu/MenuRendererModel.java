package com.lkjmc.common.menu;

import java.util.List;

public record MenuRendererModel(MenuSpec spec, MenuState state, List<SlotSpec> slots) {
    public MenuRendererModel {
        if (spec == null || state == null) {
            throw new IllegalArgumentException("renderer spec and state are required");
        }
        slots = List.copyOf(slots == null ? spec.slots() : slots);
    }
}
