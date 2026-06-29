package com.lkjmc.common.docs;

import java.util.List;

public record DocPage(String path, int page, int pageCount, List<String> lines) {
    public DocPage {
        lines = List.copyOf(lines == null ? List.of() : lines);
    }
}
