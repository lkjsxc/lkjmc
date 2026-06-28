package com.lkjmc.common.command;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.lkjmc.common.permission.PermissionNodes;
import java.util.List;
import org.junit.jupiter.api.Test;

final class LkjmcCommandTreeTest {
    @Test
    void parsesDocumentedPaperCommandsWithoutPlatformErrors() {
        assertTarget(CommandPlatform.PAPER, List.of("status"), "status");
        assertTarget(CommandPlatform.PAPER, List.of("doctor"), "doctor");
        assertTarget(CommandPlatform.PAPER, List.of("server", "list"), "instance.list");
        assertTarget(CommandPlatform.PAPER, List.of("server", "start", "hub"), "instance.start");
        assertTarget(CommandPlatform.PAPER, List.of("server", "stop", "hub"), "instance.stop");
        assertTarget(CommandPlatform.PAPER, List.of("server", "restart", "hub"), "instance.restart");
        assertTarget(CommandPlatform.PAPER, List.of("server", "create", "smp", "paper"), "instance.create");
        assertTarget(CommandPlatform.PAPER, List.of("server", "delete", "smp", "confirm"), "instance.delete");
    }

    @Test
    void rejectsIncompleteDestructiveSyntaxWithUsage() {
        assertTrue(LkjmcCommandTree.parse(CommandPlatform.PAPER, List.of("server", "delete", "smp")).isEmpty());
        assertEquals("/lkjmc server delete <server> confirm",
            LkjmcCommandTree.usage(CommandPlatform.PAPER, List.of("server", "delete", "smp")));
    }

    @Test
    void velocityOwnsProxyTransferCommands() {
        assertTarget(CommandPlatform.VELOCITY, List.of("send", "Alex", "hub"), "proxy.send");
        assertTarget(CommandPlatform.VELOCITY, List.of("temporary", "send", "Alex", "end-1"), "temporary.send");
        assertTarget(CommandPlatform.VELOCITY, List.of("wake", "send", "Alex", "hub"), "wake.send");
        assertTrue(LkjmcCommandTree.parse(CommandPlatform.PAPER, List.of("send", "Alex", "hub")).isEmpty());
    }

    @Test
    void completionsArePermissionFilteredAndContextAware() {
        var context = new CommandCompletionContext(List.of("hub", "smp"), List.of("Alex"), List.of("paper"));
        assertEquals(List.of("doctor", "status"), LkjmcCommandTree.suggest(
            CommandPlatform.PAPER, List.of(""), permission -> permission.endsWith("status"), context));
        assertEquals(List.of("list", "start"), LkjmcCommandTree.suggest(
            CommandPlatform.PAPER, List.of("server", ""), permission -> permission.equals(PermissionNodes.ADMIN_INSTANCE_LIST)
                || permission.equals(PermissionNodes.ADMIN_INSTANCE_START), context));
        assertEquals(List.of("hub", "smp"), LkjmcCommandTree.suggest(
            CommandPlatform.VELOCITY, List.of("server", "start", ""), permission -> true, context));
        assertEquals(List.of("confirm"), LkjmcCommandTree.suggest(
            CommandPlatform.PAPER, List.of("server", "delete", "smp", ""), permission -> true, context));
    }

    private static void assertTarget(CommandPlatform platform, List<String> args, String target) {
        assertEquals(target, LkjmcCommandTree.parse(platform, args).orElseThrow().spec().target());
    }
}
