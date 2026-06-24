package com.lkjmc.common.menu;

public sealed interface MenuAction permits MenuAction.None, MenuAction.Open, MenuAction.Command {
    static MenuAction none() {
        return new None();
    }

    record None() implements MenuAction {}
    record Open(MenuId menuId) implements MenuAction {}
    record Command(String command) implements MenuAction {}
}
