package com.lkjmc.common.menu;

public sealed interface MenuAction permits MenuAction.None, MenuAction.Open, MenuAction.Command, MenuAction.Close, MenuAction.Refresh, MenuAction.Disabled {
    static MenuAction none() {
        return new None();
    }

    static String key(MenuAction action) {
        return switch (action) {
            case None ignored -> "inert";
            case Open open -> "open:" + open.menuId().value();
            case Command command -> "command:" + command.command();
            case Close ignored -> "close";
            case Refresh ignored -> "refresh";
            case Disabled disabled -> "disabled:" + disabled.reasonKey();
        };
    }

    record None() implements MenuAction {}
    record Open(MenuId menuId) implements MenuAction {}
    record Command(String command) implements MenuAction {}
    record Close() implements MenuAction {}
    record Refresh() implements MenuAction {}
    record Disabled(String reasonKey) implements MenuAction {}
}
