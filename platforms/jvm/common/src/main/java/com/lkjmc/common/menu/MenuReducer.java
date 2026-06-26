package com.lkjmc.common.menu;

import java.util.List;

public final class MenuReducer {
    private MenuReducer() {}

    public static MenuDecision click(MenuSpec spec, MenuState state, MenuClick click) {
        if (!click.topInventory() || click.slot() < 0) {
            return new MenuDecision(List.of());
        }
        return spec.slots().stream()
            .filter(slot -> slot.slot() == click.slot())
            .findFirst()
            .map(slot -> decide(slot, click.actionKey()))
            .orElseGet(() -> unknownOrEmpty(click.actionKey()));
    }

    private static MenuDecision unknownOrEmpty(String actionKey) {
        if (actionKey == null) {
            return new MenuDecision(List.of());
        }
        return new MenuDecision(List.of(new MenuEffect.SendMessage("menu.error.unknown-action")));
    }

    private static MenuDecision decide(SlotSpec slot, String actionKey) {
        var expected = MenuAction.key(slot.action());
        if (actionKey != null && !actionKey.equals(expected)) {
            return new MenuDecision(List.of(new MenuEffect.SendMessage("menu.error.unknown-action")));
        }
        if (slot.item().inert()) {
            return new MenuDecision(List.of());
        }
        return new MenuDecision(effects(slot.action()));
    }

    private static List<MenuEffect> effects(MenuAction action) {
        return switch (action) {
            case MenuAction.None ignored -> List.of();
            case MenuAction.Open open -> List.of(new MenuEffect.OpenMenu(open.menuId()));
            case MenuAction.Command command -> List.of(new MenuEffect.RunCommand(command.command()));
            case MenuAction.Close ignored -> List.of(new MenuEffect.CloseMenu());
            case MenuAction.Refresh ignored -> List.of(new MenuEffect.Refresh());
            case MenuAction.Disabled disabled -> List.of(new MenuEffect.SendMessage(disabled.reasonKey()));
        };
    }
}
