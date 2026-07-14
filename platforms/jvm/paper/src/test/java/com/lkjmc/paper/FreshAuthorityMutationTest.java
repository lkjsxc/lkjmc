package com.lkjmc.paper;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.lkjmc.bindings.*;
import java.time.Instant;
import java.util.List;
import java.util.UUID;
import org.junit.jupiter.api.Test;

final class FreshAuthorityMutationTest {
    @Test
    void futureOrRevisionMutatedSnapshotsNeverAuthorize() {
        Instant now = Instant.now();
        var adapter = new FreshAuthorityAdapter();
        var payload = new PermissionPayload("player", "a", List.of(), List.of("build"));
        var current = new PermissionSnapshot("permissions", "player:a", 4,
                now.minusSeconds(1), 1, payload);
        assertTrue(adapter.permits(current, "player:a", "build", 4, now));
        assertFalse(adapter.permits(current, "player:a", "build", 3, now));
        var future = new PermissionSnapshot("permissions", "player:a", 4,
                now.plusSeconds(1), 1, payload);
        assertFalse(adapter.permits(future, "player:a", "build", 4, now));
    }

    @Test
    void changedClaimOwnerIsDenied() {
        Instant now = Instant.now();
        UUID owner = UUID.randomUUID();
        var chunk = new ClaimChunk(UUID.randomUUID(), owner, "Owner", "Base", "world",
                1, 2, List.of());
        var snapshot = new ClaimSnapshot("claims", "survival", 8, now.minusSeconds(1), 1,
                new ClaimPayload(List.of(chunk)));
        var adapter = new FreshAuthorityAdapter();
        assertTrue(adapter.owns(snapshot, "world", 1, 2, owner, 8, now));
        assertFalse(adapter.owns(snapshot, "world", 1, 2, UUID.randomUUID(), 8, now));
        assertFalse(adapter.owns(snapshot, "world", 1, 2, owner, 7, now));
    }
}
