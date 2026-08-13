package com.lkjmc.common;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

final class LkjmcBuildInfoTest {
    @Test
    void exposesCanonicalReleaseIdentity() {
        assertEquals("0.1.0-alpha.1", LkjmcBuildInfo.VERSION);
        assertNotEquals("0.0.0", LkjmcBuildInfo.VERSION);
        assertEquals("Apache-2.0", LkjmcBuildInfo.LICENSE);
        assertTrue(LkjmcBuildInfo.COMMIT.equals("unknown")
                || LkjmcBuildInfo.COMMIT.matches("[0-9a-f]{40}"));
        assertTrue(LkjmcBuildInfo.DIRTY.matches("false|unknown"));
    }
}
