package com.lkjmc.common.menu;

import java.util.List;
import java.util.Map;

public record MenuRoute(
        String id,
        MenuTypes.RouteKind kind,
        String titleKey,
        MenuTypes.Theme theme,
        int size,
        List<Param> params,
        String parent,
        List<Dependency> dependencies,
        Chrome chrome,
        List<SourceSlot> slots,
        Dynamic dynamic,
        String confirmation) {
    public MenuRoute {
        if (id == null || id.isBlank() || titleKey == null || titleKey.isBlank()
                || kind == null || theme == null || (size != 27 && size != 54)) {
            throw new IllegalArgumentException("invalid route");
        }
        params = List.copyOf(params); dependencies = List.copyOf(dependencies);
        slots = List.copyOf(slots);
    }

    public record Param(String name, boolean required) {}
    public record Dependency(MenuTypes.Domain domain, MenuTypes.Scope scope) {}
    public record Chrome(String infoKey, boolean back, boolean refresh,
                         boolean close, boolean mainMenu) {}
    public record SourceSlot(int slot, String material, String nameKey,
                             List<String> loreKeys, MenuTypes.Role role, MenuAction action) {
        public SourceSlot { loreKeys = List.copyOf(loreKeys); }
    }
    public record Dynamic(MenuTypes.Binding binding, String region,
                          String emptyNameKey, List<String> emptyLoreKeys) {
        public Dynamic { emptyLoreKeys = List.copyOf(emptyLoreKeys); }
    }

    public Map<String, Boolean> parameterMap() {
        var values = new java.util.LinkedHashMap<String, Boolean>();
        params.forEach(item -> values.put(item.name(), item.required()));
        return Map.copyOf(values);
    }
}
