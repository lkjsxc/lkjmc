package com.lkjmc.paper;

import com.lkjmc.common.menu.MenuId;
import com.lkjmc.common.menu.MenuRoute;
import com.lkjmc.common.menu.MenuRouteStack;
import com.lkjmc.common.menu.MenuState;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import org.bukkit.entity.Player;

final class MenuSessionStore {
    private final Map<UUID, MenuState> states = new HashMap<>();

    MenuState openRoot(Player player) {
        var route = new MenuRoute(new MenuId("root"));
        return put(player, route, new MenuRouteStack(List.of(route)), nextEpoch(player));
    }

    MenuState openRoute(Player player, MenuRoute route) {
        var previous = states.get(player.getUniqueId());
        var stack = previous == null ? new MenuRouteStack(List.of(route)) : previous.routeStack().push(route);
        return put(player, route, stack, nextEpoch(player));
    }

    MenuState refresh(Player player) {
        var current = states.get(player.getUniqueId());
        if (current == null) {
            return openRoot(player);
        }
        return put(player, current.route(), current.routeStack(), current.renderEpoch() + 1);
    }

    MenuState back(Player player) {
        var current = states.get(player.getUniqueId());
        if (current == null) {
            return openRoot(player);
        }
        var route = current.routeStack().previous().orElse(new MenuRoute(new MenuId("root")));
        return put(player, route, current.routeStack().pop(), current.renderEpoch() + 1);
    }

    Optional<MenuState> state(Player player) {
        return Optional.ofNullable(states.get(player.getUniqueId()));
    }

    void clearIfSession(Player player, String sessionId) {
        var current = states.get(player.getUniqueId());
        if (current != null && current.sessionId().equals(sessionId)) {
            states.remove(player.getUniqueId());
        }
    }

    private long nextEpoch(Player player) {
        return states.getOrDefault(player.getUniqueId(), new MenuState(new MenuId("root"), 0)).renderEpoch() + 1;
    }

    private MenuState put(Player player, MenuRoute route, MenuRouteStack stack, long epoch) {
        var state = new MenuState(route, stack, 0, UUID.randomUUID().toString(), epoch);
        states.put(player.getUniqueId(), state);
        return state;
    }
}
