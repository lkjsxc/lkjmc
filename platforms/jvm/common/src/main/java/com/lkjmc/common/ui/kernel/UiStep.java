package com.lkjmc.common.ui.kernel;

import java.util.List;

public record UiStep(UiModel model, List<UiEffect> effects) {
    public UiStep {
        if (model == null) {
            throw new IllegalArgumentException("model is required");
        }
        effects = List.copyOf(effects == null ? List.of() : effects);
    }
}
