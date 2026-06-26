package com.lkjmc.common.claim;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.JsonParser;
import org.junit.jupiter.api.Test;

final class ClaimProtectionPolicyTest {
    @Test
    void denies_mutating_events_for_strangers() {
        var body = JsonParser.parseString("""
            {"chunks":[{"claimId":"c1","ownerUuid":"owner","ownerName":"Owner","name":"base",
            "instanceId":"survival","worldName":"world","chunkX":1,"chunkZ":2,"trusts":[]}]}
            """).getAsJsonObject();
        var snapshot = ClaimSnapshot.fromDaemonBody(body);
        var chunk = new ClaimChunk("survival", "world", 1, 2);
        assertFalse(ClaimProtectionPolicy.decide(snapshot, "stranger", false, chunk, ClaimEventKind.BREAK).allowed());
        assertFalse(ClaimProtectionPolicy.decide(snapshot, "stranger", false, chunk, ClaimEventKind.PLACE).allowed());
        assertFalse(ClaimProtectionPolicy.decide(snapshot, "stranger", false, chunk, ClaimEventKind.INTERACT).allowed());
        assertTrue(ClaimProtectionPolicy.decide(snapshot, "stranger", true, chunk, ClaimEventKind.BREAK).allowed());
    }
}
