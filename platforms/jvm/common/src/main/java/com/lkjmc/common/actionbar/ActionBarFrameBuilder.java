package com.lkjmc.common.actionbar;

import java.util.ArrayList;
import java.util.List;

public final class ActionBarFrameBuilder {
    private ActionBarFrameBuilder() {}

    public static List<ActionBarFrame> passive(ActionBarSnapshot snapshot, long nowMillis, long ttlMillis) {
        var frames = new ArrayList<ActionBarFrame>();
        if (snapshot.dailyAvailable()) {
            frames.add(frame(5, "Daily ready", "daily", nowMillis, ttlMillis));
        }
        if (snapshot.randomTeleportCooldownSeconds() > 0) {
            frames.add(frame(4, "RTP " + snapshot.randomTeleportCooldownSeconds() + "s", "rtp", nowMillis, ttlMillis));
        }
        frames.add(frame(1, passiveText(snapshot), "passive:" + snapshot.serverId() + ":" + snapshot.balance(), nowMillis, ttlMillis));
        return List.copyOf(frames);
    }

    public static String passiveText(ActionBarSnapshot snapshot) {
        var points = snapshot.balance() >= 0 ? " · Points " + snapshot.balance() : "";
        return "Play " + ActionBarFormatter.playtime(snapshot.playtimeSeconds())
            + points + " · " + safe(snapshot.serverId())
            + " · Online " + snapshot.serverPlayerCount() + "/" + snapshot.networkOnlineCount();
    }

    private static ActionBarFrame frame(int priority, String text, String key, long now, long ttl) {
        return new ActionBarFrame(priority, text, key, now + ttl);
    }

    private static String safe(String value) {
        return value == null || value.isBlank() ? "server" : value;
    }
}
