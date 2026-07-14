package com.lkjmc.bindings;

public record ProfileSlot(
        int slot,
        ProfileItem item
) {
    public ProfileSlot {
        java.util.Objects.requireNonNull(item, "item");
    }
}
