package com.lkjmc.common.player;

import java.util.regex.Pattern;

public final class HomeNamePolicy {
    private static final Pattern SAFE_NAME = Pattern.compile("[A-Za-z0-9_-]{1,32}");

    private HomeNamePolicy() {}

    public static boolean isValid(String name) {
        return name != null && SAFE_NAME.matcher(name).matches();
    }
}
