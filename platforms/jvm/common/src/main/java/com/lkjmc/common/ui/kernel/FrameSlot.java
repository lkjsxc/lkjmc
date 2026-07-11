package com.lkjmc.common.ui.kernel;

import com.lkjmc.common.ui.document.DocumentAction;
import com.lkjmc.common.ui.document.ItemRole;
import java.util.List;
import java.util.Map;

public record FrameSlot(
    int slot,
    String material,
    TextRef name,
    List<TextRef> lore,
    ItemRole role,
    MenuMetadata metadata,
    boolean inert
) {
    public FrameSlot {
        if (slot < 0 || slot >= 54) {
            throw new IllegalArgumentException("frame slot out of range");
        }
        if (material == null || material.isBlank()) {
            throw new IllegalArgumentException("frame material is required");
        }
        if (name == null) {
            throw new IllegalArgumentException("frame name is required");
        }
        lore = List.copyOf(lore == null ? List.of() : lore);
        role = role == null ? ItemRole.ACTION : role;
    }

    public static FrameSlot inert(int slot, String material, TextRef name, List<TextRef> lore,
                                  ItemRole role) {
        return new FrameSlot(slot, material, name, lore, role, null, true);
    }

    public static FrameSlot action(int slot, String material, TextRef name, List<TextRef> lore,
                                   ItemRole role, DocumentAction action, Map<String, String> routeParams) {
        return new FrameSlot(slot, material, name, lore, role,
            MenuMetadata.template(slot, action, routeParams), false);
    }

    FrameSlot disabled(String reasonKey) {
        return new FrameSlot(slot, material, name, lore, ItemRole.DISABLED,
            MenuMetadata.template(slot, "disabled:" + reasonKey, Map.of("type", "disabled", "key", reasonKey)), false);
    }

    public FrameSlot stamped(MenuRoute route, String sessionId, long epoch) {
        if (inert) {
            return this;
        }
        var next = metadata == null
            ? MenuMetadata.template(slot, "none", Map.of())
            : metadata;
        return new FrameSlot(slot, material, name, lore, role, next.stamp(route, sessionId, epoch), false);
    }
}
