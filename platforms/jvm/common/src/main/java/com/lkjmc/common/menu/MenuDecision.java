package com.lkjmc.common.menu;

import java.util.List;

public record MenuDecision(List<MenuEffect> effects, MenuFailure failure) {
    public MenuDecision(List<MenuEffect> effects) {
        this(effects, null);
    }

    public MenuDecision {
        effects = List.copyOf(effects == null ? List.of() : effects);
    }
}
