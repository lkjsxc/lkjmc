package com.lkjmc.common.menu;

import com.lkjmc.bindings.ClaimSnapshot;
import com.lkjmc.bindings.MenuSnapshot;
import com.lkjmc.bindings.PermissionSnapshot;
import com.lkjmc.bindings.SettingsSnapshot;
import com.lkjmc.bindings.TypedSnapshot;
import java.util.EnumMap;
import java.util.Map;
import java.util.Optional;

public final class MenuSnapshotView {
    private final Map<MenuTypes.Domain, Entry> entries;

    public MenuSnapshotView(Map<MenuTypes.Domain, Entry> entries) {
        this.entries = Map.copyOf(entries);
    }

    public static MenuSnapshotView unavailable() { return new MenuSnapshotView(Map.of()); }

    public static MenuSnapshotView of(MenuTypes.Freshness freshness, TypedSnapshot... snapshots) {
        var values = new EnumMap<MenuTypes.Domain, Entry>(MenuTypes.Domain.class);
        for (var snapshot : snapshots) {
            var domain = domain(snapshot);
            values.put(domain, new Entry(freshness, snapshot.revision(), snapshot));
        }
        return new MenuSnapshotView(values);
    }

    public MenuSnapshotView withLocalDocs() {
        var values = new EnumMap<MenuTypes.Domain, Entry>(MenuTypes.Domain.class);
        values.putAll(entries);
        values.put(MenuTypes.Domain.LOCAL_DOCS,
                new Entry(MenuTypes.Freshness.CURRENT, 1, null));
        return new MenuSnapshotView(values);
    }

    public Entry entry(MenuTypes.Domain domain) {
        return entries.getOrDefault(domain,
                new Entry(MenuTypes.Freshness.UNAVAILABLE, 0, null));
    }

    public Optional<TypedSnapshot> snapshot(MenuTypes.Domain domain) {
        return Optional.ofNullable(entry(domain).snapshot());
    }

    public boolean hasCurrentCapability(String capability) {
        var entry = entry(MenuTypes.Domain.PERMISSIONS);
        return entry.freshness() == MenuTypes.Freshness.CURRENT
                && entry.snapshot() instanceof PermissionSnapshot value
                && value.payload().permissions().contains(capability);
    }

    private static MenuTypes.Domain domain(TypedSnapshot value) {
        if (value instanceof MenuSnapshot) return MenuTypes.Domain.MENUS;
        if (value instanceof PermissionSnapshot) return MenuTypes.Domain.PERMISSIONS;
        if (value instanceof ClaimSnapshot) return MenuTypes.Domain.CLAIMS;
        if (value instanceof SettingsSnapshot) return MenuTypes.Domain.SETTINGS;
        return MenuTypes.Domain.valueOf(value.domain().toUpperCase(java.util.Locale.ROOT));
    }

    public record Entry(MenuTypes.Freshness freshness, long revision,
                        TypedSnapshot snapshot) {
        public Entry {
            if (freshness == null || revision < 0) throw new IllegalArgumentException("invalid view");
        }
    }
}
