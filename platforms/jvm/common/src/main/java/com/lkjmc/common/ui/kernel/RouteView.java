package com.lkjmc.common.ui.kernel;

import java.util.List;

public sealed interface RouteView permits RouteView.ListView, RouteView.DetailView,
    RouteView.CustomView {
    record ListView(List<EntryView> entries, List<TextRef> infoLines,
                    List<FrameSlot> reservedSlots) implements RouteView {
        public ListView(List<EntryView> entries, List<TextRef> infoLines) {
            this(entries, infoLines, List.of());
        }
        public ListView {
            entries = List.copyOf(entries == null ? List.of() : entries);
            infoLines = List.copyOf(infoLines == null ? List.of() : infoLines);
            reservedSlots = List.copyOf(reservedSlots == null ? List.of() : reservedSlots);
        }
    }

    record DetailView(List<FrameSlot> slots, List<TextRef> infoLines) implements RouteView {
        public DetailView {
            slots = List.copyOf(slots == null ? List.of() : slots);
            infoLines = List.copyOf(infoLines == null ? List.of() : infoLines);
        }
    }

    record CustomView(String name, List<FrameSlot> slots, List<TextRef> infoLines) implements RouteView {
        public CustomView {
            if (name == null || name.isBlank()) {
                throw new IllegalArgumentException("custom view name is required");
            }
            slots = List.copyOf(slots == null ? List.of() : slots);
            infoLines = List.copyOf(infoLines == null ? List.of() : infoLines);
        }
    }
}
