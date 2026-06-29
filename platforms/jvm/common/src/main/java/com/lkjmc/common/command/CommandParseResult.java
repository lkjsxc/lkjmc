package com.lkjmc.common.command;

public record CommandParseResult(
    CommandInvocation invocation,
    CommandParseFailureKind failureKind,
    String usage
) {
    public CommandParseResult {
        if ((invocation == null) == (failureKind == null)) {
            throw new IllegalArgumentException("parse result must be success or failure");
        }
        usage = usage == null ? "" : usage;
    }

    public static CommandParseResult success(CommandInvocation invocation) {
        return new CommandParseResult(invocation, null, invocation.spec().usage());
    }

    public static CommandParseResult failure(CommandParseFailureKind kind, String usage) {
        if (kind == null) {
            throw new IllegalArgumentException("failure kind is required");
        }
        return new CommandParseResult(null, kind, usage);
    }

    public boolean success() {
        return invocation != null;
    }
}
