package com.lkjmc.common.docs;

import java.util.ArrayList;
import java.util.List;

public final class DocLineWrapper {
    private DocLineWrapper() {}

    public static List<String> wrap(List<String> raw, int width) {
        var lines = new ArrayList<String>();
        for (var line : raw) {
            if (line == null || line.isBlank()) {
                lines.add("");
                continue;
            }
            var remaining = line.stripTrailing();
            while (visibleWidth(remaining) > width) {
                var cut = cut(remaining, width);
                lines.add(remaining.substring(0, cut).stripTrailing());
                remaining = remaining.substring(cut).stripLeading();
            }
            lines.add(remaining);
        }
        return List.copyOf(lines);
    }

    public static int visibleWidth(String text) {
        var width = 0;
        for (var index = 0; index < text.length();) {
            var cp = text.codePointAt(index);
            width += cp > 0x2E80 ? 2 : 1;
            index += Character.charCount(cp);
        }
        return width;
    }

    private static int cut(String value, int width) {
        var lastSpace = -1;
        var used = 0;
        for (var index = 0; index < value.length();) {
            var cp = value.codePointAt(index);
            if (Character.isWhitespace(cp)) lastSpace = index;
            var next = used + (cp > 0x2E80 ? 2 : 1);
            if (next > width) return lastSpace > 0 ? lastSpace : index;
            used = next;
            index += Character.charCount(cp);
        }
        return value.length();
    }
}
