package com.lkjmc.common.menu;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

public record MenuRouteStack(List<MenuRoute> entries) {
    public MenuRouteStack {
        var copy = new ArrayList<>(entries == null || entries.isEmpty()
            ? List.of(rootRoute()) : entries);
        if (!isRoot(copy.get(0))) {
            copy.add(0, rootRoute());
        }
        entries = List.copyOf(copy);
    }

    public static MenuRoute rootRoute() {
        return new MenuRoute(new MenuId("root"));
    }

    public static MenuRouteStack root() {
        return new MenuRouteStack(List.of(rootRoute()));
    }

    public MenuRoute last() {
        return entries.get(entries.size() - 1);
    }

    public MenuRouteStack push(MenuRoute route) {
        return pushDistinct(route, false);
    }

    public MenuRouteStack pushDistinct(MenuRoute route) {
        return pushDistinct(route, true);
    }

    public Optional<MenuRoute> previous() {
        return entries.size() < 2 ? Optional.empty() : Optional.of(entries.get(entries.size() - 2));
    }

    public MenuRouteStack pop() {
        return entries.size() <= 1 ? this : new MenuRouteStack(entries.subList(0, entries.size() - 1));
    }

    public MenuRouteStack popOrRoot() {
        return entries.size() <= 1 ? root() : pop();
    }

    public MenuRouteStack replaceTop(MenuRoute route) {
        var next = route == null ? rootRoute() : route;
        if (isRoot(next)) {
            return root();
        }
        var copy = new ArrayList<>(entries);
        copy.set(copy.size() - 1, next);
        return new MenuRouteStack(copy);
    }

    private MenuRouteStack pushDistinct(MenuRoute route, boolean distinct) {
        var next = route == null ? rootRoute() : route;
        if (isRoot(next)) {
            return root();
        }
        if (distinct && last().equals(next)) {
            return this;
        }
        var copy = new ArrayList<>(entries);
        copy.add(next);
        return new MenuRouteStack(copy);
    }

    private static boolean isRoot(MenuRoute route) {
        return route != null && route.id().value().equals("root");
    }
}
