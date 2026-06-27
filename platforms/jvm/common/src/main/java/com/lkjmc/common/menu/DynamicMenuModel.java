package com.lkjmc.common.menu;

import java.util.Map;

public record DynamicMenuModel(Map<String, Object> values, boolean loading, String failureKey) {
    public DynamicMenuModel {
        values = Map.copyOf(values == null ? Map.of() : values);
    }

    public static DynamicMenuModel empty() {
        return new DynamicMenuModel(Map.of(), false, null);
    }
}
