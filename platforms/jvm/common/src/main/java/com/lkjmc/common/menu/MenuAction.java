package com.lkjmc.common.menu;

import java.util.Map;

public sealed interface MenuAction permits MenuAction.Navigate, MenuAction.Simple {
    MenuTypes.ActionType type();

    record Navigate(String route, Map<String, String> params) implements MenuAction {
        public Navigate {
            if (route == null || route.isBlank()) throw new IllegalArgumentException("route required");
            params = Map.copyOf(params);
        }
        @Override public MenuTypes.ActionType type() { return MenuTypes.ActionType.NAVIGATE; }
    }

    record Simple(MenuTypes.ActionType type) implements MenuAction {
        public Simple {
            if (type == null || type == MenuTypes.ActionType.NAVIGATE) {
                throw new IllegalArgumentException("simple action required");
            }
        }
    }
}
