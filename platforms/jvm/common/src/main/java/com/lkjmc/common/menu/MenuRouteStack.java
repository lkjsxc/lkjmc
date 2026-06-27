package com.lkjmc.common.menu;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

public record MenuRouteStack(List<MenuRoute> entries) {
    public MenuRouteStack {
        entries = List.copyOf(entries == null ? List.of() : entries);
    }

    public static MenuRouteStack root() {
        return new MenuRouteStack(List.of(new MenuRoute(new MenuId("root"))));
    }

    public MenuRouteStack push(MenuRoute route) {
        var copy = new ArrayList<>(entries);
        copy.add(route);
        return new MenuRouteStack(copy);
    }

    public Optional<MenuRoute> previous() {
        return entries.size() < 2 ? Optional.empty() : Optional.of(entries.get(entries.size() - 2));
    }

    public MenuRouteStack pop() {
        if (entries.size() <= 1) {
            return this;
        }
        return new MenuRouteStack(entries.subList(0, entries.size() - 1));
    }
}
