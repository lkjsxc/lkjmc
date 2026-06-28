package com.lkjmc.velocity;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.JsonObject;
import org.junit.jupiter.api.Test;

final class VelocityServerRegistryTest {
    @Test
    void registrationHintControlsManagedServers() {
        var instance = new JsonObject();
        instance.addProperty("proxyRegistration", true);
        assertTrue(VelocityServerRegistry.shouldRegister(instance));
        instance.addProperty("proxyRegistration", false);
        assertFalse(VelocityServerRegistry.shouldRegister(instance));
    }
}
