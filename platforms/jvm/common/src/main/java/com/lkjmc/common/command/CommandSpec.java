package com.lkjmc.common.command;

import java.util.List;
import java.util.Set;

public record CommandSpec(
    List<String> path,
    String permission,
    CommandSenderKind senderKind,
    String usage,
    String summaryKey,
    String target,
    Set<CommandPlatform> platforms
) {
    public CommandSpec {
        path = List.copyOf(path);
        platforms = Set.copyOf(platforms);
        if (path.isEmpty()) {
            throw new IllegalArgumentException("command path is required");
        }
        if (permission == null || permission.isBlank()) {
            throw new IllegalArgumentException("permission is required");
        }
    }

    public boolean supports(CommandPlatform platform) {
        return platforms.contains(platform);
    }

    public static boolean isArgument(String token) {
        return token.startsWith("<") && token.endsWith(">");
    }
}
