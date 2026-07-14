package com.lkjmc.bindings;

public record PotionEffect(
        String id,
        int amplifier,
        long durationTicks,
        boolean ambient,
        boolean particles,
        boolean icon
) {
    public PotionEffect {
        java.util.Objects.requireNonNull(id, "id");
        if (id.isBlank()) throw new IllegalArgumentException("id");
    }
}
