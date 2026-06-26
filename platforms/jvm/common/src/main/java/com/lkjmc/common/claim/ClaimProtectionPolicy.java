package com.lkjmc.common.claim;

public final class ClaimProtectionPolicy {
    private ClaimProtectionPolicy() {}

    public static ClaimDecision decide(
        ClaimSnapshot snapshot,
        String playerUuid,
        boolean operator,
        ClaimChunk chunk,
        ClaimEventKind event
    ) {
        if (event == null) {
            return ClaimDecision.allow();
        }
        return snapshot.decide(playerUuid, operator, chunk);
    }
}
