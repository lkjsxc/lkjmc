package com.lkjmc.common.command;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.lkjmc.common.permission.PermissionNodes;
import java.util.List;
import org.junit.jupiter.api.Test;

final class LkjmcCommandTreeTest {
    private static final String ROOT = "/lkjmc status|doctor|server|admin|config|security|economy|adventure|reload|restart";

    @Test
    void parsesDocumentedPaperCommandsWithoutPlatformErrors() {
        assertTarget(CommandPlatform.PAPER, List.of("status"), "status");
        assertTarget(CommandPlatform.PAPER, List.of("doctor"), "doctor");
        assertTarget(CommandPlatform.PAPER, List.of("server", "list"), "instance.list");
        assertTarget(CommandPlatform.PAPER, List.of("server", "start", "hub"), "instance.start");
        assertTarget(CommandPlatform.PAPER, List.of("server", "delete", "smp", "confirm"), "instance.delete");
        assertTarget(CommandPlatform.PAPER, List.of("config", "reload"), "config.reload");
        assertTarget(CommandPlatform.PAPER, List.of("admin", "role", "list"), "admin.role.list");
        assertTarget(CommandPlatform.PAPER, List.of("admin", "inspect", "minecraft-player:abc"), "admin.principal.inspect");
        assertTarget(CommandPlatform.PAPER, List.of("security", "daemon-token", "status"), "security.daemon-token.status");
        assertTarget(CommandPlatform.PAPER, List.of("adventure", "start", "resource-rush"), "adventure.purchase");
    }

    @Test
    void returnsTypedFailuresWithUsage() {
        assertFailure(CommandPlatform.PAPER, List.of(), CommandParseFailureKind.EMPTY_ROOT, ROOT);
        assertFailure(CommandPlatform.PAPER, List.of("server"), CommandParseFailureKind.INCOMPLETE_BRANCH,
            "/lkjmc server list|start|stop|restart|create|delete");
        assertFailure(CommandPlatform.PAPER, List.of("server", "start"), CommandParseFailureKind.MISSING_ARGUMENT,
            "/lkjmc server list|start|stop|restart|create|delete");
        assertFailure(CommandPlatform.PAPER, List.of("restart", "warn", "soon"),
            CommandParseFailureKind.MALFORMED_ARGUMENT, "/lkjmc restart warn <seconds>");
        assertFailure(CommandPlatform.PAPER, List.of("send", "Alex", "hub"),
            CommandParseFailureKind.UNSUPPORTED_PLATFORM, ROOT);
        assertFailure(CommandPlatform.PAPER, List.of("wat"), CommandParseFailureKind.UNKNOWN_LITERAL, ROOT);
    }

    @Test
    void rejectsIncompleteDestructiveSyntaxWithUsage() {
        assertFailure(CommandPlatform.PAPER, List.of("server", "delete", "smp"),
            CommandParseFailureKind.INCOMPLETE_BRANCH, "/lkjmc server delete <server> confirm");
    }

    @Test
    void velocityOwnsProxyTransferCommands() {
        assertTarget(CommandPlatform.VELOCITY, List.of("send", "Alex", "hub"), "proxy.send");
        assertTarget(CommandPlatform.VELOCITY, List.of("temporary", "send", "Alex", "end-1"), "temporary.send");
        assertTarget(CommandPlatform.VELOCITY, List.of("wake", "send", "Alex", "hub"), "wake.send");
        assertFalse(LkjmcCommandTree.parse(CommandPlatform.PAPER, List.of("send", "Alex", "hub")).success());
    }

    @Test
    void completionsArePermissionFilteredAndContextAware() {
        var context = new CommandCompletionContext(List.of("hub", "smp"), List.of("Alex"), List.of("paper"),
            List.of("owner"), List.of("resource-rush"), List.of(), List.of(), List.of(), List.of("minecraft-player:abc"));
        assertEquals(List.of("config", "doctor", "status"), LkjmcCommandTree.suggest(
            CommandPlatform.PAPER, List.of(""), permission -> permission.endsWith("status"), context));
        assertEquals(List.of("list", "start"), LkjmcCommandTree.suggest(
            CommandPlatform.PAPER, List.of("server", ""), permission -> permission.equals(PermissionNodes.ADMIN_INSTANCE_LIST)
                || permission.equals(PermissionNodes.ADMIN_INSTANCE_START), context));
        assertEquals(List.of("hub", "smp"), LkjmcCommandTree.suggest(
            CommandPlatform.VELOCITY, List.of("server", "start", ""), permission -> true, context));
        assertEquals(List.of("confirm"), LkjmcCommandTree.suggest(
            CommandPlatform.PAPER, List.of("server", "delete", "smp", ""), permission -> true, context));
        assertEquals(List.of("resource-rush"), LkjmcCommandTree.suggest(
            CommandPlatform.PAPER, List.of("adventure", "start", ""), permission -> true, context));
    }

    private static void assertTarget(CommandPlatform platform, List<String> args, String target) {
        var result = LkjmcCommandTree.parse(platform, args);
        assertTrue(result.success());
        assertEquals(target, result.invocation().spec().target());
    }

    private static void assertFailure(CommandPlatform platform, List<String> args,
                                      CommandParseFailureKind kind, String usage) {
        var result = LkjmcCommandTree.parse(platform, args);
        assertFalse(result.success());
        assertEquals(kind, result.failureKind());
        assertEquals(usage, result.usage());
    }
}
