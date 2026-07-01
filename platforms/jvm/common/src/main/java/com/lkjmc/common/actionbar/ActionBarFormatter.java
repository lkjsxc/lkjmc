package com.lkjmc.common.actionbar;

public final class ActionBarFormatter {
    private ActionBarFormatter() {}

    public static String playtime(long seconds) {
        var minutes = Math.max(0, seconds) / 60;
        if (minutes < 60) {
            return minutes + "m";
        }
        var hours = minutes / 60;
        var remainder = minutes % 60;
        if (hours >= 100) {
            return hours + "h";
        }
        return hours + "h " + String.format("%02dm", remainder);
    }
}
