package com.lkjmc.common.menu;

public record RandomTeleportQuote(
    String profileId,
    String targetEnvironment,
    boolean confirmationRequired,
    boolean enabled,
    boolean canAfford,
    long costPoints,
    long balance,
    long cooldownRemainingSeconds,
    int minRadius,
    int maxRadius,
    int maxAttempts
) {
    public RandomTeleportQuote(boolean enabled, boolean canAfford, long costPoints, long balance,
                               long cooldownRemainingSeconds, int minRadius, int maxRadius, int maxAttempts) {
        this("overworld", "normal", costPoints > 0, enabled, canAfford, costPoints, balance,
            cooldownRemainingSeconds, minRadius, maxRadius, maxAttempts);
    }

    public RandomTeleportQuote {
        profileId = profileId == null || profileId.isBlank() ? "overworld" : profileId;
        targetEnvironment = targetEnvironment == null || targetEnvironment.isBlank() ? "normal" : targetEnvironment;
    }

    public boolean available() {
        return enabled && canAfford && cooldownRemainingSeconds <= 0;
    }
}
