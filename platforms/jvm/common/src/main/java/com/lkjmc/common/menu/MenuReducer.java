package com.lkjmc.common.menu;

import java.util.List;

public final class MenuReducer {
    private MenuReducer() {}

    public static MenuDecision click(MenuSpec spec, MenuState state, MenuClick click) {
        return spec.slots().stream()
            .filter(slot -> slot.slot() == click.slot())
            .findFirst()
            .map(slot -> new MenuDecision(effects(slot.action())))
            .orElseGet(() -> new MenuDecision(List.of(new MenuEffect.Refresh())));
    }

    private static List<MenuEffect> effects(MenuAction action) {
        return switch (action) {
            case MenuAction.None ignored -> List.of();
            case MenuAction.Open open -> List.of(new MenuEffect.OpenMenu(open.menuId()));
            case MenuAction.Command command -> List.of(new MenuEffect.RunCommand(command.command()));
        };
    }
}
