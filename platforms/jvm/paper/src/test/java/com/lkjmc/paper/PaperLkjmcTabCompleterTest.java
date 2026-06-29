package com.lkjmc.paper;

import static org.junit.jupiter.api.Assertions.assertEquals;

import com.lkjmc.common.command.CommandCompletionContext;
import com.lkjmc.common.permission.PermissionNodes;
import java.lang.reflect.Proxy;
import java.util.List;
import org.bukkit.command.CommandSender;
import org.junit.jupiter.api.Test;

final class PaperLkjmcTabCompleterTest {
    @Test
    void suggestsRootTreeWithPermissions() {
        var completer = new PaperLkjmcTabCompleter(() -> new CommandCompletionContext(
            List.of("hub"), List.of(), List.of("paper")));
        var sender = sender(PermissionNodes.ADMIN_STATUS, PermissionNodes.ADMIN_INSTANCE_LIST);
        assertEquals(List.of("doctor", "server", "status"), completer.onTabComplete(sender, null, "lkjmc",
            new String[] {""}));
    }

    @Test
    void suggestsSharedServerTreeWithPermissions() {
        var completer = new PaperLkjmcTabCompleter(() -> new CommandCompletionContext(
            List.of("hub"), List.of(), List.of("paper")));
        var sender = sender(PermissionNodes.ADMIN_INSTANCE_LIST, PermissionNodes.ADMIN_INSTANCE_START);
        assertEquals(List.of("list", "start"), completer.onTabComplete(sender, null, "lkjmc",
            new String[] {"server", ""}));
        assertEquals(List.of("hub"), completer.onTabComplete(sender, null, "lkjmc",
            new String[] {"server", "start", ""}));
    }

    private static CommandSender sender(String... permissions) {
        var allowed = java.util.Set.of(permissions);
        return proxy(CommandSender.class, (proxy, method, args) -> switch (method.getName()) {
            case "hasPermission" -> allowed.contains((String) args[0]);
            default -> fallback(method.getReturnType());
        });
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
}
