package com.lkjmc.paper;

import com.lkjmc.common.BuildInfo;

public final class PaperBuildInfo {
    private PaperBuildInfo() {
    }

    public static BuildInfo common() {
        return BuildInfo.common();
    }

    public static String component() {
        return "platforms:jvm:paper";
    }
}
