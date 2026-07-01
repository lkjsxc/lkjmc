package com.lkjmc.common.menu;

import java.util.List;

public record ShopView(long balance, String category, List<ShopMenuEntry> entries) {
    public ShopView {
        category = category == null || category.isBlank() ? "all" : category;
        entries = entries == null ? List.of() : List.copyOf(entries);
    }
}
