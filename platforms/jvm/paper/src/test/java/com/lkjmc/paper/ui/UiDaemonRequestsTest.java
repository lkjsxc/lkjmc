package com.lkjmc.paper.ui;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertInstanceOf;

import java.util.Map;
import org.junit.jupiter.api.Test;

final class UiDaemonRequestsTest {
    @Test
    void explicitEulaConfirmationIsEncodedAsBoolean() {
        var body = UiDaemonRequests.body(UiTestFixtures.player(), "adventure.purchase",
            Map.of("acceptMinecraftEula", "true"));

        assertEquals(Boolean.TRUE, assertInstanceOf(Boolean.class, body.get("acceptMinecraftEula")));
    }
}
