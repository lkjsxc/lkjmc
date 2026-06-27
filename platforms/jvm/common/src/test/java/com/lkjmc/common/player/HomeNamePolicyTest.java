package com.lkjmc.common.player;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

final class HomeNamePolicyTest {
    @Test
    void acceptsCommandSafeHomeNames() {
        assertTrue(HomeNamePolicy.isValid("base"));
        assertTrue(HomeNamePolicy.isValid("Base_01-west"));
    }

    @Test
    void rejectsNamesThatCannotRoundTripThroughCommands() {
        assertFalse(HomeNamePolicy.isValid(""));
        assertFalse(HomeNamePolicy.isValid("bad name"));
        assertFalse(HomeNamePolicy.isValid("slash/name"));
        assertFalse(HomeNamePolicy.isValid("a".repeat(33)));
    }
}
