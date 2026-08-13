package com.lkjmc.velocity;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import org.junit.jupiter.api.Test;

final class VelocityPluginMetadataTest {
    @Test
    void descriptorUsesCanonicalReleaseVersion() throws Exception {
        try (var resource = getClass().getResourceAsStream("/velocity-plugin.json")) {
            assertNotNull(resource);
            var metadata = new String(resource.readAllBytes(), StandardCharsets.UTF_8);
            assertTrue(metadata.contains("\"version\":\"0.1.0-alpha.1\""));
            assertFalse(metadata.contains("0.0.0"));
        }
    }
}
