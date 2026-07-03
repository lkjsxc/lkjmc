package com.lkjmc.paper;

import com.lkjmc.common.menu.ItemSpec;
import com.lkjmc.common.menu.ItemVisualRole;
import com.lkjmc.common.menu.MenuAction;
import com.lkjmc.common.menu.MenuId;
import com.lkjmc.common.menu.MenuSpec;
import com.lkjmc.common.menu.SlotSpec;
import java.time.Duration;
import java.util.ArrayList;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;

final class StaleMenuCache {
    private static final long TTL_MILLIS = Duration.ofMinutes(2).toMillis();
    private final ConcurrentHashMap<String, Entry> entries = new ConcurrentHashMap<>();

    void remember(UUID player, MenuSpec spec) {
        entries.put(key(player, spec.id()), new Entry(spec, System.currentTimeMillis()));
    }

    MenuSpec fallback(UUID player, MenuId id, String code, MenuSpec unavailable) {
        var entry = entries.get(key(player, id));
        if (entry == null || System.currentTimeMillis() - entry.loadedAt() > TTL_MILLIS) {
            return unavailable;
        }
        var slots = new ArrayList<SlotSpec>();
        for (var slot : entry.spec().slots()) {
            if (slot.slot() != 4) { slots.add(slot); }
        }
        slots.add(new SlotSpec(4, new ItemSpec("CLOCK", "menu.stale-data",
            java.util.List.of("menu.stale-data.lore", "literal:" + code), ItemVisualRole.INFO),
            MenuAction.none()));
        return new MenuSpec(entry.spec().id(), entry.spec().title(), entry.spec().size(), slots);
    }

    private static String key(UUID player, MenuId id) {
        return player + ":" + id.value();
    }

    private record Entry(MenuSpec spec, long loadedAt) {}
}
