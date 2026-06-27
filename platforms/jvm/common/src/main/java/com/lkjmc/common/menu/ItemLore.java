package com.lkjmc.common.menu;

import java.util.List;

public record ItemLore(List<String> keys) {
    public ItemLore {
        keys = List.copyOf(keys == null ? List.of() : keys);
    }
}
