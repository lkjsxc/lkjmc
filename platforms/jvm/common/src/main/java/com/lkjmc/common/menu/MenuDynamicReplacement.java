package com.lkjmc.common.menu;

public final class MenuDynamicReplacement {
    private MenuDynamicReplacement() {}

    public static boolean accepts(MenuState current, MenuState pending) {
        return current != null && pending != null
            && current.sessionId().equals(pending.sessionId())
            && current.renderEpoch() == pending.renderEpoch()
            && current.route().equals(pending.route());
    }
}
