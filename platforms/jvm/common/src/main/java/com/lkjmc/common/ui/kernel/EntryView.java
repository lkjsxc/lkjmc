package com.lkjmc.common.ui.kernel;

import com.lkjmc.common.ui.document.DocumentAction;
import com.lkjmc.common.ui.document.ItemRole;
import java.util.List;

public record EntryView(
    String material,
    TextRef name,
    List<TextRef> lore,
    ItemRole role,
    DocumentAction action
) {
    public EntryView {
        if (material == null || material.isBlank()) {
            throw new IllegalArgumentException("entry material is required");
        }
        if (name == null) {
            throw new IllegalArgumentException("entry name is required");
        }
        lore = List.copyOf(lore == null ? List.of() : lore);
        role = role == null ? ItemRole.ACTION : role;
        action = action == null ? DocumentAction.none() : action;
    }
}
