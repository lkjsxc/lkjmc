package com.lkjmc.common.menu;

public sealed interface MenuEffect permits MenuEffect.OpenRoute, MenuEffect.OpenPrevious,
    MenuEffect.CloseMenu, MenuEffect.RefreshRoute, MenuEffect.RunPlayerCommand,
    MenuEffect.SendDaemonCommand, MenuEffect.TransferPlayer, MenuEffect.SendMessage,
    MenuEffect.PromptText, MenuEffect.RenderLoadingThenRun, MenuEffect.Noop {

    record OpenRoute(MenuRoute route) implements MenuEffect {}
    record OpenPrevious() implements MenuEffect {}
    record CloseMenu() implements MenuEffect {}
    record RefreshRoute() implements MenuEffect {}
    record RunPlayerCommand(String command) implements MenuEffect {}
    record SendDaemonCommand(String command, MenuActionPayload body) implements MenuEffect {}
    record TransferPlayer(String serverId) implements MenuEffect {}
    record SendMessage(String key) implements MenuEffect {}
    record PromptText(String promptKey, String commandPrefix) implements MenuEffect {}
    record RenderLoadingThenRun(MenuEffect effect) implements MenuEffect {}
    record Noop() implements MenuEffect {}
}
