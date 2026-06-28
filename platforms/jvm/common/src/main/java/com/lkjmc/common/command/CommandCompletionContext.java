package com.lkjmc.common.command;

import java.util.List;

public record CommandCompletionContext(
    List<String> serverIds,
    List<String> playerNames,
    List<String> templates
) {
    public CommandCompletionContext {
        serverIds = serverIds == null ? List.of() : List.copyOf(serverIds);
        playerNames = playerNames == null ? List.of() : List.copyOf(playerNames);
        templates = templates == null ? List.of() : List.copyOf(templates);
    }

    public static CommandCompletionContext empty() {
        return new CommandCompletionContext(List.of(), List.of(), List.of());
    }
}
