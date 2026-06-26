package com.lkjmc.common.menu;

public sealed interface MenuEffect permits MenuEffect.OpenMenu, MenuEffect.RunCommand, MenuEffect.CloseMenu, MenuEffect.Refresh, MenuEffect.SendMessage {
    record OpenMenu(MenuId menuId) implements MenuEffect {}
    record RunCommand(String command) implements MenuEffect {}
    record CloseMenu() implements MenuEffect {}
    record Refresh() implements MenuEffect {}
    record SendMessage(String key) implements MenuEffect {}
}
