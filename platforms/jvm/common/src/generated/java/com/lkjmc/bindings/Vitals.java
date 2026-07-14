package com.lkjmc.bindings;

public record Vitals(
        double health,
        int food,
        double saturation,
        int air
) {
    public Vitals {
    }
}
