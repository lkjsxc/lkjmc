package com.lkjmc.common.claim;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.JsonParser;
import org.junit.jupiter.api.Test;

final class ClaimSnapshotTest {
    @Test
    void parsesSnapshotAndDecidesAccess() {
        var body = JsonParser.parseString("""
            {"chunks":[{"claimId":"c1","ownerUuid":"owner","ownerName":"Owner","name":"base",
            "instanceId":"survival","worldName":"world","chunkX":1,"chunkZ":2,
            "trusts":[{"uuid":"friend","name":"Friend"}]}]}
            """).getAsJsonObject();
        var snapshot = ClaimSnapshot.fromDaemonBody(body);
        var chunk = new ClaimChunk("survival", "world", 1, 2);
        assertTrue(snapshot.decide("owner", false, chunk).allowed());
        assertTrue(snapshot.decide("friend", false, chunk).allowed());
        assertTrue(snapshot.decide("stranger", true, chunk).allowed());
        assertFalse(snapshot.decide("stranger", false, chunk).allowed());
        assertTrue(snapshot.ownerClaimByName("owner", "BASE").isPresent());
    }
}
