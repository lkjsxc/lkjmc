package com.lkjmc.paper;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.lkjmc.common.docs.DocBundle;
import java.nio.charset.StandardCharsets;
import org.junit.jupiter.api.Test;

final class LocalPaperSurfaceTest {
    @Test
    void bundledDocumentationLoadsWithoutExternalData() throws Exception {
        try (var resource = LocalDocsMenu.class.getResourceAsStream("/lkjmc-docs-bundle.json")) {
            assertNotNull(resource);
            var docs = DocBundle.load(resource);
            assertFalse(docs.files().isEmpty());
            assertTrue(docs.file("docs/README.md").isPresent());
        }
    }

    @Test
    void pluginMetadataRegistersOnlyLocalCommands() throws Exception {
        try (var resource = LocalPaperSurfaceTest.class.getResourceAsStream("/plugin.yml")) {
            assertNotNull(resource);
            var metadata = new String(resource.readAllBytes(), StandardCharsets.UTF_8);
            assertTrue(metadata.contains("commands:\n  menu:\n"));
            assertTrue(metadata.contains("\n  docs:\n"));
            assertFalse(metadata.contains("\n  lkjmc:"));
        }
    }

    @Test
    void documentationTokenUsesTheLastHotbarSlot() {
        assertEquals(8, HotbarMenuTokenService.SLOT);
    }
}
