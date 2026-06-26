package com.lkjmc.common.claim;

import java.util.Optional;

public record ClaimDecision(boolean allowed, Optional<ClaimRecord> claim) {
    public ClaimDecision {
        claim = claim == null ? Optional.empty() : claim;
    }

    public static ClaimDecision allow() {
        return new ClaimDecision(true, Optional.empty());
    }

    public static ClaimDecision deny(ClaimRecord claim) {
        return new ClaimDecision(false, Optional.of(claim));
    }
}
