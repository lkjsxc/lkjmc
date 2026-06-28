package com.lkjmc.common.command;

@FunctionalInterface
public interface CommandPermissionChecker {
    boolean has(String permission);
}
