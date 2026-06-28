package com.lkjmc.common.command;

import java.util.Map;

public record CommandInvocation(CommandSpec spec, Map<String, String> arguments) {
    public CommandInvocation {
        if (spec == null) {
            throw new IllegalArgumentException("command spec is required");
        }
        arguments = arguments == null ? Map.of() : Map.copyOf(arguments);
    }

    public String argument(String name) {
        return arguments.getOrDefault(name, "");
    }
}
