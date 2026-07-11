package com.lkjmc.common.ui.kernel;

public sealed interface UiEffect permits UiEffect.LoadData, UiEffect.SendDaemon,
    UiEffect.RunCommand, UiEffect.Transfer, UiEffect.Message, UiEffect.PromptText,
    UiEffect.CloseInventory {
    record LoadData(DaemonRequestPlan plan, UiRequest request) implements UiEffect {
        public LoadData(DaemonRequestPlan plan) { this(plan, UiRequest.none()); }
    }
    record SendDaemon(DaemonRequestPlan plan, TextRef ok, TextRef fail,
                      boolean refreshOnOk, UiRequest request) implements UiEffect {
        public SendDaemon(DaemonRequestPlan plan, TextRef ok, TextRef fail, boolean refreshOnOk) {
            this(plan, ok, fail, refreshOnOk, UiRequest.none());
        }
    }
    record RunCommand(String command) implements UiEffect {}
    record Transfer(String serverId) implements UiEffect {}
    record Message(TextRef text) implements UiEffect {}
    record PromptText(TextRef prompt, String commandPrefix) implements UiEffect {}
    record CloseInventory() implements UiEffect {}
}
