package com.lkjmc.common.menu;

public record PageWindow(int firstIndex, int lastExclusive, int totalItems) {
    public PageWindow {
        if (firstIndex < 0 || lastExclusive < firstIndex || totalItems < lastExclusive) {
            throw new IllegalArgumentException("invalid page window");
        }
    }
}
