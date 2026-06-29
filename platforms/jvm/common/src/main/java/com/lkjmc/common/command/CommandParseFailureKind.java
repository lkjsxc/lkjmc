package com.lkjmc.common.command;

public enum CommandParseFailureKind {
    EMPTY_ROOT,
    INCOMPLETE_BRANCH,
    UNKNOWN_LITERAL,
    MISSING_ARGUMENT,
    MALFORMED_ARGUMENT,
    UNSUPPORTED_PLATFORM
}
