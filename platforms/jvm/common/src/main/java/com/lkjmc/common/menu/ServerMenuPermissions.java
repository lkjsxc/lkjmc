package com.lkjmc.common.menu;

public record ServerMenuPermissions(boolean canStart, boolean canStop) {
    public static ServerMenuPermissions none() {
        return new ServerMenuPermissions(false, false);
    }
}
