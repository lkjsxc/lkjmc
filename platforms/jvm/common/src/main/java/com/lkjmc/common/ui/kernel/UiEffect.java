package com.lkjmc.common.ui.kernel;

public sealed interface UiEffect permits UiEffect.LoadData, UiEffect.SendDaemon,
    UiEffect.RunCommand, UiEffect.Transfer, UiEffect.Message, UiEffect.PromptText,
    UiEffect.CloseInventory {
    record LoadData(DaemonRequestPlan plan) implements UiEffect {}
    record SendDaemon(DaemonRequestPlan plan, TextRef ok, TextRef fail,
                      boolean refreshOnOk) implements UiEffect {}
    record RunCommand(String command) implements UiEffect {}
    record Transfer(String serverId) implements UiEffect {}
    record Message(TextRef text) implements UiEffect {}
    record PromptText(TextRef prompt, String commandPrefix) implements UiEffect {}
    record CloseInventory() implements UiEffect {}
}
