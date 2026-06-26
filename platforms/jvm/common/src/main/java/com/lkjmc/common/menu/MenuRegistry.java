package com.lkjmc.common.menu;

import java.util.Collection;
import java.util.Map;
import java.util.Optional;
import java.util.stream.Collectors;

public record MenuRegistry(Map<MenuId, MenuSpec> menus) {
    public MenuRegistry(Collection<MenuSpec> specs) {
        this(specs.stream().collect(Collectors.toUnmodifiableMap(MenuSpec::id, spec -> spec)));
    }

    public Optional<MenuSpec> find(MenuId id) {
        return Optional.ofNullable(menus.get(id));
    }

    public MenuSpec require(MenuId id) {
        return find(id).orElseThrow(() -> new IllegalArgumentException("unknown menu: " + id.value()));
    }
}
