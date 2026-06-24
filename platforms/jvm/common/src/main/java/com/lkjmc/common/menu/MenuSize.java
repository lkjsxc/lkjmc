package com.lkjmc.common.menu;

public record MenuSize(int slots) {
    public MenuSize {
        if (slots <= 0 || slots % 9 != 0 || slots > 54) {
            throw new IllegalArgumentException("menu size must be 9..54 and divisible by 9");
        }
    }
}
