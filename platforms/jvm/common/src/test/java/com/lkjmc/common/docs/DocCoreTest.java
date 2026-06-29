package com.lkjmc.common.docs;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import org.junit.jupiter.api.Test;

final class DocCoreTest {
    @Test
    void rejectsTraversalAndAbsolutePaths() {
        assertTrue(DocPath.normalize("../AGENTS.md").isEmpty());
        assertTrue(DocPath.normalize("/etc/passwd").isEmpty());
        assertEquals("docs/README.md", DocPath.normalize("docs//./README.md").orElseThrow());
    }

    @Test
    void wrapsAndPaginatesTenLines() {
        var file = new DocFile("README.md", "Readme", List.of(
            "one two three four five six seven eight nine ten eleven",
            "", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j"), List.of());

        var page = DocPaginator.page(file, 0, 10);

        assertTrue(page.lines().size() <= 10);
        assertTrue(page.pageCount() > 1);
    }
}
