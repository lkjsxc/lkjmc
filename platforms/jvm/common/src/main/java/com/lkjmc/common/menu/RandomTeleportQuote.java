package com.lkjmc.common.menu;

public record RandomTeleportQuote(
    boolean enabled,
    boolean canAfford,
    long costPoints,
    long balance,
    long cooldownRemainingSeconds,
    int minRadius,
    int maxRadius,
    int maxAttempts
) {
    public boolean available() {
        return enabled && canAfford && cooldownRemainingSeconds <= 0;
    }
}
