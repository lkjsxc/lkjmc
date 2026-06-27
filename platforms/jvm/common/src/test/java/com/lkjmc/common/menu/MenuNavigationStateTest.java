package com.lkjmc.common.menu;

import static org.junit.jupiter.api.Assertions.assertEquals;

import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;

final class MenuNavigationStateTest {
    @Test
    void rootResetCreatesRootOnlyStack() {
        var state = open(root(), "travel").openRoot("s2", 2);
        assertRoute(state, "root");
        assertPath(state, "root");
    }

    @Test
    void openingChildPushesOnceAndCurrentRouteDoesNotDuplicate() {
        var state = open(root(), "travel");
        assertPath(state, "root", "travel");
        var same = state.openRoute(route("travel"), "s2", 2);
        assertPath(same, "root", "travel");
    }

    @Test
    void backAtRootRepairsToRoot() {
        var state = root().back("s2", 2);
        assertRoute(state, "root");
        assertPath(state, "root");
    }

    @Test
    void refreshAndDynamicReplacementPreserveRouteStack() {
        var state = open(open(root(), "travel"), "homes");
        var refreshed = state.refresh("s3", 3);
        var replaced = refreshed.replaceDynamic("s4", 4);
        assertRoute(replaced, "homes");
        assertPath(replaced, "root", "travel", "homes");
    }

    @Test
    void homesBackPathDoesNotLoop() {
        var state = open(open(root(), "travel"), "homes");
        state = state.back("s4", 4);
        assertRoute(state, "travel");
        state = state.back("s5", 5);
        assertRoute(state, "root");
        assertPath(state, "root");
    }

    @Test
    void teleportPickerBackPathDoesNotLoop() {
        var state = open(open(open(root(), "travel"), "teleports"), "teleport-picker");
        state = state.back("s5", 5);
        assertRoute(state, "teleports");
        state = state.back("s6", 6);
        assertRoute(state, "travel");
        state = state.back("s7", 7);
        assertRoute(state, "root");
    }

    @Test
    void claimConfirmBackPathDoesNotLoop() {
        var detail = new MenuRoute(new MenuId("claim-detail"), Map.of("name", "base", "chunkCount", "2"));
        var confirm = new MenuRoute(new MenuId("claim-confirm"), Map.of("name", "base"));
        var state = root().openRoute(route("claims"), "s2", 2).openRoute(detail, "s3", 3)
            .openRoute(confirm, "s4", 4);
        state = state.back("s5", 5);
        assertRoute(state, "claim-detail");
        state = state.back("s6", 6);
        assertRoute(state, "claims");
        state = state.back("s7", 7);
        assertRoute(state, "root");
    }

    @Test
    void reportConfirmBackPathDoesNotLoop() {
        var detail = new MenuRoute(new MenuId("report-detail"), Map.of("reportId", "r1"));
        var confirm = new MenuRoute(new MenuId("report-confirm"), Map.of("reportId", "r1", "action", "resolve"));
        var state = root().openRoute(route("social"), "s2", 2).openRoute(route("reports"), "s3", 3)
            .openRoute(detail, "s4", 4).openRoute(confirm, "s5", 5);
        state = state.back("s6", 6);
        assertRoute(state, "report-detail");
    }

    private static MenuNavigationState root() {
        return MenuNavigationState.initial().openRoot("s1", 1);
    }

    private static MenuNavigationState open(MenuNavigationState state, String id) {
        return state.openRoute(route(id), "s", state.renderEpoch() + 1);
    }

    private static MenuRoute route(String id) {
        return new MenuRoute(new MenuId(id));
    }

    private static void assertRoute(MenuNavigationState state, String id) {
        assertEquals(id, state.route().id().value());
        assertEquals(state.route(), state.routeStack().last());
    }

    private static void assertPath(MenuNavigationState state, String... ids) {
        assertEquals(List.of(ids), state.routeStack().entries().stream().map(route -> route.id().value()).toList());
    }
}
