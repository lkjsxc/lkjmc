package com.lkjmc.common.docs;

import java.util.List;

public record DocFile(String path, String title, List<String> lines, List<DocLink> links) {
    public DocFile {
        lines = List.copyOf(lines == null ? List.of() : lines);
        links = List.copyOf(links == null ? List.of() : links);
    }
}
