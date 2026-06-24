package com.lkjmc.common.menu;

public record NavigationPolicy(int backSlot, int previousSlot, int nextSlot, int pageInfoSlot) {
    public static NavigationPolicy standard54() {
        return new NavigationPolicy(49, 46, 47, 48);
    }
}
