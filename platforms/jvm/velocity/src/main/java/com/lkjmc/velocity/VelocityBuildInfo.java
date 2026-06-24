package com.lkjmc.velocity;

import com.lkjmc.common.BuildInfo;

public final class VelocityBuildInfo {
    private VelocityBuildInfo() {
    }

    public static BuildInfo common() {
        return BuildInfo.common();
    }

    public static String component() {
        return "platforms:jvm:velocity";
    }
}
