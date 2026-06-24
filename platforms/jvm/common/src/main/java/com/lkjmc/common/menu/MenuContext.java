package com.lkjmc.common.menu;

import java.util.Map;

public record MenuContext(String locale, Map<String, String> values) {
    public MenuContext {
        values = Map.copyOf(values == null ? Map.of() : values);
    }
}
