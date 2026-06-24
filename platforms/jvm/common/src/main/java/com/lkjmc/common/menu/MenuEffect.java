package com.lkjmc.common.menu;

public sealed interface MenuEffect permits MenuEffect.OpenMenu, MenuEffect.RunCommand, MenuEffect.Refresh {
    record OpenMenu(MenuId menuId) implements MenuEffect {}
    record RunCommand(String command) implements MenuEffect {}
    record Refresh() implements MenuEffect {}
}
