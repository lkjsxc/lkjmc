package com.lkjmc.common.actionbar;

public record ActionBarSnapshot(
    boolean hudEnabled,
    long playtimeSeconds,
    long balance,
    String serverId,
    long serverPlayerCount,
    long networkOnlineCount,
    boolean dailyAvailable,
    long randomTeleportCooldownSeconds
) {}
