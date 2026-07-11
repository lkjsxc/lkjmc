package com.lkjmc.common.command;

import java.util.List;

public record CommandCompletionContext(
    List<String> serverIds,
    List<String> playerNames,
    List<String> templates,
    List<String> roleIds,
    List<String> adventureIds,
    List<String> shopItemIds,
    List<String> kitIds,
    List<String> voteIds,
    List<String> principalHints
) {
    public CommandCompletionContext {
        serverIds = copy(serverIds);
        playerNames = copy(playerNames);
        templates = copy(templates);
        roleIds = copy(roleIds);
        adventureIds = copy(adventureIds);
        shopItemIds = copy(shopItemIds);
        kitIds = copy(kitIds);
        voteIds = copy(voteIds);
        principalHints = copy(principalHints);
    }

    public CommandCompletionContext(List<String> serverIds, List<String> playerNames, List<String> templates) {
        this(serverIds, playerNames, templates, List.of("owner", "operator", "moderator", "support", "builder"),
            List.of("end-expedition"), List.of(), List.of(), List.of(), List.of());
    }

    public static CommandCompletionContext empty() {
        return new CommandCompletionContext(List.of(), List.of(), List.of());
    }

    private static List<String> copy(List<String> values) {
        return values == null ? List.of() : List.copyOf(values);
    }
}
