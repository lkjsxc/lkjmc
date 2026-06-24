package com.lkjmc.common;

public record BuildInfo(String component) {
    public static BuildInfo common() {
        return new BuildInfo("platforms:jvm:common");
    }
}
