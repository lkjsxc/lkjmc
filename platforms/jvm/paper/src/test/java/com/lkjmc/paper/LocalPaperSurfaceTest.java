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
    void curatedDocumentationLoadsWithoutExternalData() throws Exception {
        try (var resource = PaperMenuAdapter.class.getResourceAsStream("/lkjmc-docs-bundle.json")) {
            assertNotNull(resource);
            var docs = DocBundle.load(resource);
            assertEquals(11, docs.files().size());
            assertTrue(docs.file("docs/product/gui/README.md").isPresent());
            assertFalse(docs.file("AGENTS.md").isPresent());
        }
    }

    @Test
    void pluginMetadataRegistersOnlyLocalCommands() throws Exception {
        try (var resource = LocalPaperSurfaceTest.class.getResourceAsStream("/plugin.yml")) {
            assertNotNull(resource);
            var metadata = new String(resource.readAllBytes(), StandardCharsets.UTF_8);
            assertTrue(metadata.contains("version: '0.1.0-alpha.1'"));
            assertFalse(metadata.contains("0.0.0"));
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
