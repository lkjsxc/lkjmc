package com.lkjmc.common.menu;

public sealed interface MenuAction permits MenuAction.None, MenuAction.OpenRoute,
    MenuAction.Back, MenuAction.Close, MenuAction.RefreshRoute,
    MenuAction.RunPlayerCommand, MenuAction.DaemonCommand, MenuAction.Transfer,
    MenuAction.Confirm, MenuAction.Disabled, MenuAction.TextInput,
    MenuAction.Select, MenuAction.Purchase, MenuAction.Toggle {

    static MenuAction none() { return new None(); }

    static String key(MenuAction action) {
        return switch (action) {
            case None ignored -> "none";
            case OpenRoute open -> "open:" + open.route().id().value();
            case Back ignored -> "back";
            case Close ignored -> "close";
            case RefreshRoute ignored -> "refresh";
            case RunPlayerCommand command -> "command:" + command.command();
            case DaemonCommand command -> "daemon:" + command.command();
            case Transfer transfer -> "transfer:" + transfer.serverId();
            case Confirm confirm -> "confirm:" + confirm.route().id().value();
            case Disabled disabled -> "disabled:" + disabled.reasonKey();
            case TextInput input -> "input:" + input.commandPrefix();
            case Select select -> "select:" + select.payload().value();
            case Purchase purchase -> "purchase:" + purchase.payload().value();
            case Toggle toggle -> "toggle:" + toggle.settingKey();
        };
    }

    record None() implements MenuAction {}
    record OpenRoute(MenuRoute route) implements MenuAction {}
    record Back() implements MenuAction {}
    record Close() implements MenuAction {}
    record RefreshRoute() implements MenuAction {}
    record RunPlayerCommand(String command) implements MenuAction {}
    record DaemonCommand(String command, MenuActionPayload body) implements MenuAction {}
    record Transfer(String serverId) implements MenuAction {}
    record Confirm(MenuRoute route, MenuAction confirmAction) implements MenuAction {}
    record Disabled(String reasonKey) implements MenuAction {}
    record TextInput(String promptKey, String commandPrefix) implements MenuAction {}
    record Select(MenuActionPayload payload) implements MenuAction {}
    record Purchase(MenuActionPayload payload) implements MenuAction {}
    record Toggle(String settingKey) implements MenuAction {}
}
