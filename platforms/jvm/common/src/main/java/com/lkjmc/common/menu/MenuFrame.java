package com.lkjmc.common.menu;

import java.util.List;
import java.util.Map;

public record MenuFrame(String route, String title, int size, long session,
                        long renderRevision, List<Slot> slots) {
    public MenuFrame { slots = List.copyOf(slots); }

    public record Slot(int index, String material, String name, List<String> lore,
                       MenuTypes.Role role, MenuAction action, Metadata metadata) {
        public Slot { lore = List.copyOf(lore); }
    }

    public record Metadata(String route, long session, long renderRevision,
                           int slot, MenuTypes.ActionType action) {}

    public static Slot slot(int index, String material, String name, List<String> lore,
                            MenuTypes.Role role, MenuAction action, String route,
                            long session, long revision) {
        var metadata = new Metadata(route, session, revision, index, action.type());
        return new Slot(index, material, name, lore, role, action, metadata);
    }

    public Map<Integer, Slot> bySlot() {
        var result = new java.util.TreeMap<Integer, Slot>();
        slots.forEach(slot -> result.put(slot.index(), slot));
        return Map.copyOf(result);
    }
}
