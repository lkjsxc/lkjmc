package com.lkjmc.paper;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.lkjmc.bindings.ClaimChunk;
import com.lkjmc.bindings.ClaimSnapshot;
import com.lkjmc.bindings.PermissionSnapshot;
import java.time.Instant;
import java.util.Set;
import java.util.UUID;
import org.junit.jupiter.api.Test;

final class FreshAuthorityMutationTest {
    @Test
    void staleOrRevisionMutatedSnapshotsNeverAuthorize() {
        Instant now = Instant.now();
        var adapter = new FreshAuthorityAdapter();
        var current = new PermissionSnapshot(now.plusSeconds(2), Set.of("build"), "player:a", 4);
        assertTrue(adapter.permits(current, "player:a", "build", 4, now));
        assertFalse(adapter.permits(current, "player:a", "build", 3, now));
        var stale = new PermissionSnapshot(now.minusSeconds(1), Set.of("build"), "player:a", 4);
        assertFalse(adapter.permits(stale, "player:a", "build", 4, now));
    }

    @Test
    void changedClaimOwnerIsDenied() {
        Instant now = Instant.now();
        UUID owner = UUID.randomUUID();
        var snapshot = new ClaimSnapshot(Set.of(new ClaimChunk(1, 2, owner, "world")),
                now.plusSeconds(2), 8);
        var adapter = new FreshAuthorityAdapter();
        assertTrue(adapter.owns(snapshot, "world", 1, 2, owner, 8, now));
        assertFalse(adapter.owns(snapshot, "world", 1, 2, UUID.randomUUID(), 8, now));
        assertFalse(adapter.owns(snapshot, "world", 1, 2, owner, 7, now));
    }
}
