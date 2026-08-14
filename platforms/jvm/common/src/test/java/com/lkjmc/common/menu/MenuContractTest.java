package com.lkjmc.common.menu;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.ByteArrayInputStream;
import java.nio.charset.StandardCharsets;
import java.util.Set;
import java.util.stream.Collectors;
import org.junit.jupiter.api.Test;

final class MenuContractTest {
    @Test
    void loadsOnlyTheLocalMenuAndDocsRoutes() {
        var bundle = MenuBundle.fromResource();
        assertEquals(Set.of("root", "docs-directory", "docs-file", "docs-links", "docs-search"),
                bundle.routes().stream().map(MenuRoute::id).collect(Collectors.toSet()));
        for (var route : bundle.routes()) {
            assertTrue(!route.chrome().refresh());
            assertNull(route.confirmation());
            assertTrue(route.dependencies().stream().allMatch(dependency ->
                    dependency.domain() == MenuTypes.Domain.LOCAL_DOCS
                            && dependency.scope() == MenuTypes.Scope.LOCAL));
            if (route.dynamic() != null) {
                assertTrue(route.dynamic().binding().name().startsWith("DOCS_"));
            }
            route.slots().forEach(slot -> assertTrue(
                    slot.action() instanceof MenuAction.Navigate
                            || slot.action() instanceof MenuAction.Simple));
        }
    }

    @Test
    void rejectsMissingNavigationTargetParameters() throws Exception {
        String mutated = resource().replaceFirst("\\\"route\\\":\\\"docs-directory\\\"",
                "\\\"route\\\":\\\"docs-file\\\"");
        assertThrows(IllegalArgumentException.class, () -> load(mutated));
    }

    @Test
    void rejectsMutationActions() throws Exception {
        String source = resource();
        String mutated = source.replaceFirst("\\\"type\\\":\\\"NONE\\\"",
                "\\\"type\\\":\\\"MUTATION\\\",\\\"operation\\\":\\\"STATUS_VIEW\\\"," +
                        "\\\"capability\\\":\\\"menu.action.status-view\\\"");
        assertThrows(IllegalArgumentException.class, () -> load(mutated));
    }

    @Test
    void rejectsChromeActionsInAuthoredSlots() throws Exception {
        String mutated = resource().replaceFirst("\\\"type\\\":\\\"NONE\\\"",
                "\\\"type\\\":\\\"BACK\\\"");
        assertThrows(IllegalArgumentException.class, () -> load(mutated));
    }

    @Test
    void rejectsGenericActionMembers() throws Exception {
        String source = resource();
        String mutated = source.replaceFirst("\\\"type\\\":\\\"NONE\\\"",
                "\\\"type\\\":\\\"NONE\\\",\\\"body\\\":{}");
        assertThrows(IllegalArgumentException.class, () -> load(mutated));
    }

    private static String resource() throws Exception {
        try (var input = MenuContractTest.class.getResourceAsStream("/lkjmc-menu-bundle.json")) {
            return new String(input.readAllBytes(), StandardCharsets.UTF_8);
        }
    }

    private static MenuBundle load(String source) {
        return MenuBundle.load(new ByteArrayInputStream(source.getBytes(StandardCharsets.UTF_8)));
    }
}
