package com.lkjmc.paper;

import com.lkjmc.common.menu.MenuNavigationState;
import com.lkjmc.common.menu.MenuRoute;
import com.lkjmc.common.menu.MenuState;
import java.util.HashMap;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import org.bukkit.entity.Player;

final class MenuSessionStore {
    private final Map<UUID, MenuState> states = new HashMap<>();

    MenuState openRoot(Player player) {
        return put(player, MenuNavigationState.initial().openRoot(newSession(), nextEpoch(player)));
    }

    MenuState openRoute(Player player, MenuRoute route) {
        var current = MenuNavigationState.from(states.get(player.getUniqueId()));
        return put(player, current.openRoute(route, newSession(), nextEpoch(player)));
    }

    MenuState refresh(Player player) {
        var current = states.get(player.getUniqueId());
        if (current == null) {
            return openRoot(player);
        }
        return put(player, MenuNavigationState.from(current).refresh(newSession(), current.renderEpoch() + 1));
    }

    MenuState replaceDynamic(Player player) {
        var current = states.get(player.getUniqueId());
        if (current == null) {
            return openRoot(player);
        }
        return put(player, MenuNavigationState.from(current).replaceDynamic(newSession(), current.renderEpoch() + 1));
    }

    MenuState back(Player player) {
        var current = states.get(player.getUniqueId());
        if (current == null) {
            return openRoot(player);
        }
        return put(player, MenuNavigationState.from(current).back(newSession(), current.renderEpoch() + 1));
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
        return states.getOrDefault(player.getUniqueId(), MenuNavigationState.initial().toMenuState())
            .renderEpoch() + 1;
    }

    private String newSession() {
        return UUID.randomUUID().toString();
    }

    private MenuState put(Player player, MenuNavigationState navigation) {
        var state = navigation.toMenuState();
        states.put(player.getUniqueId(), state);
        return state;
    }
}
