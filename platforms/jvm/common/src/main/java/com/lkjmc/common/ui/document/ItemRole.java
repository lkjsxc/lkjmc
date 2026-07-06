package com.lkjmc.common.ui.document;

public enum ItemRole {
    INFO,
    ACTION,
    NAVIGATION,
    DECORATION,
    DISABLED,
    SUCCESS,
    DANGER,
    WARNING,
    LOADING;

    public static ItemRole parse(String value) {
        return value == null ? ACTION : ItemRole.valueOf(value.toUpperCase().replace('-', '_'));
    }

    public boolean inertByRole() {
        return this == INFO || this == DECORATION;
    }
}
