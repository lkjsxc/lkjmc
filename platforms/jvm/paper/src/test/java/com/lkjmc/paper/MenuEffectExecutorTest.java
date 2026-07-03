package com.lkjmc.paper;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;

import com.lkjmc.common.transfer.ProfileTransferMessages;
import java.util.concurrent.atomic.AtomicReference;
import org.bukkit.Location;
import org.junit.jupiter.api.Test;

final class MenuEffectExecutorTest {
    @Test
    void transferChannelEmitsProfileTransferRequest() {
        var sent = new AtomicReference<byte[]>();
        new ProfileTransferChannel((player, message) -> sent.set(message)).transfer(null, "hub");
        assertArrayEquals(ProfileTransferMessages.transferRequest("hub"), sent.get());
    }

    @Test
    void homeLocationIsNestedDaemonShape() {
        var payload = MenuEffectExecutor.homeLocation(new Location(null, 1.5, 64.0, -2.25, 90.0f, 10.0f));
        assertEquals("world", payload.get("world"));
        assertEquals(1.5, payload.get("x"));
        assertEquals(64.0, payload.get("y"));
        assertEquals(-2.25, payload.get("z"));
        assertEquals(90.0, payload.get("yaw"));
        assertEquals(10.0, payload.get("pitch"));
    }
}
