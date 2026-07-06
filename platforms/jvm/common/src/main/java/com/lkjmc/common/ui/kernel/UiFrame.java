package com.lkjmc.common.ui.kernel;

import java.util.List;

public record UiFrame(TextRef title, int size, List<FrameSlot> slots) {
    public UiFrame {
        if (title == null) {
            throw new IllegalArgumentException("frame title is required");
        }
        if (size <= 0 || size % 9 != 0) {
            throw new IllegalArgumentException("invalid frame size");
        }
        slots = List.copyOf(slots == null ? List.of() : slots);
    }
}
