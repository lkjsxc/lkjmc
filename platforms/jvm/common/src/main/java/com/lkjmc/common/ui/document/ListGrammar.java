package com.lkjmc.common.ui.document;

import java.util.List;

public record ListGrammar(
    String region,
    String reserved,
    boolean pagination,
    String emptyName,
    List<String> emptyLore
) {
    public ListGrammar {
        if (region == null || region.isBlank()) {
            throw new IllegalArgumentException("list region is required");
        }
        emptyLore = List.copyOf(emptyLore == null ? List.of() : emptyLore);
    }
}
