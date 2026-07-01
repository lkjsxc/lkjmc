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

    @Test
    void resolvesRouteParentsDeterministically() {
        assertEquals("dir:", DocRoute.parent("dir:"));
        assertEquals("dir:", DocRoute.parent("dir:a"));
        assertEquals("dir:a", DocRoute.parent("dir:a/b"));
        assertEquals("dir:", DocRoute.parent("file:a.md:0"));
        assertEquals("dir:a", DocRoute.parent("file:a/b.md:2"));
        assertEquals("dir:a", DocRoute.parent("links:a/b.md:2"));
        assertEquals("dir:", DocRoute.parent("search:anything"));
        assertEquals("dir:", DocRoute.parent("unknown"));
    }

    @Test
    void clampsRoutePages() {
        assertEquals("file:a.md:0", DocRoute.page("file:a.md:0", -1, 3));
        assertEquals("file:a.md:2", DocRoute.page("file:a.md:2", 1, 3));
        assertEquals("file:a.md:0", DocRoute.page("file:a.md:not-a-page", 1, 1));
    }

    @Test
    void filePageControlsStayAdjacentToContent() {
        assertEquals(21, DocBrowserLayout.FILE_PREVIOUS_SLOT);
        assertEquals(22, DocBrowserLayout.FILE_CONTENT_SLOT);
        assertEquals(23, DocBrowserLayout.FILE_NEXT_SLOT);
        assertTrue(DocBrowserLayout.isFileReadingSlot(21));
        assertTrue(DocBrowserLayout.isFileReadingSlot(22));
        assertTrue(DocBrowserLayout.isFileReadingSlot(23));
    }
}
