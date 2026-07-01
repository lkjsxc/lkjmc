package com.lkjmc.common.menu;

import static org.junit.jupiter.api.Assertions.assertEquals;

import java.util.Map;
import org.junit.jupiter.api.Test;

final class MenuActionPayloadTest {
    @Test
    void encodesValuesInSortedOrder() {
        var payload = new MenuActionPayload(Map.of("reason", "safe now", "force", "true", "id", "alpha"));
        assertEquals("force=true&id=alpha&reason=safe+now", payload.value());
    }

    @Test
    void decodesEscapedMultiFieldValues() {
        var payload = new MenuActionPayload("id=alpha&reason=safe+now&force=true");
        assertEquals(Map.of("id", "alpha", "reason", "safe now", "force", "true"), payload.values());
    }

    @Test
    void keepsEmptyPayloadStable() {
        assertEquals("", MenuActionPayload.EMPTY.value());
        assertEquals(Map.of(), new MenuActionPayload("").values());
    }
}
