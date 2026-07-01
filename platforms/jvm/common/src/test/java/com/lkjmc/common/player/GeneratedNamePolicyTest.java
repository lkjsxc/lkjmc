package com.lkjmc.common.player;

import static org.junit.jupiter.api.Assertions.assertEquals;

import java.util.Set;
import org.junit.jupiter.api.Test;

final class GeneratedNamePolicyTest {
    @Test
    void homesUseDuplicateFreeNumberedNames() {
        assertEquals("home", GeneratedNamePolicy.nextNumbered("home", Set.of()));
        assertEquals("home-2", GeneratedNamePolicy.nextNumbered("home", Set.of("home")));
        assertEquals("home-3", GeneratedNamePolicy.nextNumbered("home", Set.of("home", "home-2")));
    }

    @Test
    void generatedBaseIsSafe() {
        assertEquals("my-home", GeneratedNamePolicy.nextNumbered("My Home!", Set.of()));
    }
}
