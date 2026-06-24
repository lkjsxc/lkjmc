package com.lkjmc.common.menu;

import java.util.List;

public record MenuDecision(List<MenuEffect> effects) {
    public MenuDecision {
        effects = List.copyOf(effects == null ? List.of() : effects);
    }
}
