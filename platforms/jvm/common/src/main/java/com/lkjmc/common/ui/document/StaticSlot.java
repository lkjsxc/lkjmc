package com.lkjmc.common.ui.document;

import java.util.List;

public record StaticSlot(
    int slot,
    String material,
    String name,
    List<String> lore,
    ItemRole role,
    DocumentAction action
) {
    public StaticSlot {
        if (slot < 0) {
            throw new IllegalArgumentException("slot must be non-negative");
        }
        if (material == null || material.isBlank()) {
            throw new IllegalArgumentException("material is required");
        }
        if (name == null || name.isBlank()) {
            throw new IllegalArgumentException("name is required");
        }
        lore = List.copyOf(lore == null ? List.of() : lore);
        role = role == null ? ItemRole.ACTION : role;
        action = action == null ? DocumentAction.none() : action;
    }

    public boolean inert() {
        return role.inertByRole() || action instanceof DocumentAction.None;
    }
}
