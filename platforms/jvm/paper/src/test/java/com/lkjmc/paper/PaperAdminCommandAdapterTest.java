package com.lkjmc.paper;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.lkjmc.common.command.CommandPlatform;
import com.lkjmc.common.command.LkjmcCommandTree;
import com.lkjmc.common.permission.PermissionNodes;
import java.lang.reflect.Proxy;
import java.util.Set;
import java.util.UUID;
import org.bukkit.entity.Player;
import java.util.ArrayList;
import org.bukkit.command.CommandSender;
import org.junit.jupiter.api.Test;

final class PaperAdminCommandAdapterTest {
    @Test
    void incompleteServerBranchRendersProductUsage() {
        var sender = sender(Set.of());
        new PaperAdminCommandAdapter(null).handle(sender.proxy(), new String[] {"server"});

        assertEquals(1, sender.messages().size());
        assertEquals("usage: /lkjmc server list|start|stop|restart|create|delete", sender.messages().get(0));
        assertFalse(sender.messages().get(0).contains("position"));
    }

    @Test
    void malformedRestartRendersProductUsage() {
        var sender = sender(Set.of());
        new PaperAdminCommandAdapter(null).handle(sender.proxy(), new String[] {"restart", "warn", "soon"});

        assertEquals("usage: /lkjmc restart warn <seconds>", sender.messages().get(0));
        assertFalse(sender.messages().get(0).contains("position"));
    }

    @Test
    void missingPermissionNamesProductPermission() {
        var sender = sender(Set.of());
        new PaperAdminCommandAdapter(null).handle(sender.proxy(), new String[] {"status"});

        assertEquals("no permission: " + PermissionNodes.ADMIN_STATUS, sender.messages().get(0));
    }

    @Test
    void paper_instance_create_body_omits_eula_acceptance() {
        var parsed = LkjmcCommandTree.parse(CommandPlatform.PAPER,
            java.util.List.of("server", "create", "hub", "paper-survival"));

        assertTrue(parsed.success());
        assertFalse(PaperAdminCommandAdapter.instanceCreateBody(parsed.invocation())
            .containsKey("acceptMinecraftEula"));
    }

    @Test
    void paper_adventure_body_omits_eula_acceptance() {
        var player = proxy(Player.class, (ignored, method, args) -> switch (method.getName()) {
            case "getUniqueId" -> UUID.fromString("00000000-0000-0000-0000-000000000001");
            case "getName" -> "Alex";
            default -> fallback(method.getReturnType());
        });

        var body = new PaperAdminCommandAdapter(null).adventureBody(player, "end-expedition");

        assertFalse(body.containsKey("acceptMinecraftEula"));
    }

    private static TestSender sender(Set<String> permissions) {
        var messages = new ArrayList<String>();
        var proxy = proxy(CommandSender.class, (ignored, method, args) -> switch (method.getName()) {
            case "sendMessage" -> {
                messages.add(String.valueOf(args[0]));
                yield null;
            }
            case "hasPermission" -> permissions.contains((String) args[0]);
            default -> fallback(method.getReturnType());
        });
        return new TestSender(proxy, messages);
    }

    @SuppressWarnings("unchecked")
    private static <T> T proxy(Class<T> type, java.lang.reflect.InvocationHandler handler) {
        return (T) Proxy.newProxyInstance(type.getClassLoader(), new Class<?>[] {type}, handler);
    }

    private static Object fallback(Class<?> type) {
        if (type.equals(boolean.class)) return false;
        if (type.equals(int.class)) return 0;
        if (type.equals(void.class)) return null;
        return null;
    }

    private record TestSender(CommandSender proxy, ArrayList<String> messages) {}
}
