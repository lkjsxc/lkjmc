package com.lkjmc.paper;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;

import java.lang.reflect.Proxy;
import java.util.ArrayList;
import org.bukkit.command.CommandSender;
import org.junit.jupiter.api.Test;

final class PaperAdminCommandAdapterTest {
    @Test
    void incompleteServerBranchRendersProductUsage() {
        var sender = sender();
        new PaperAdminCommandAdapter(null).handle(sender.proxy(), new String[] {"server"});

        assertEquals(1, sender.messages().size());
        assertEquals("usage: /lkjmc server list|start|stop|restart|create|delete", sender.messages().get(0));
        assertFalse(sender.messages().get(0).contains("position"));
    }

    private static TestSender sender() {
        var messages = new ArrayList<String>();
        var proxy = proxy(CommandSender.class, (ignored, method, args) -> switch (method.getName()) {
            case "sendMessage" -> {
                messages.add(String.valueOf(args[0]));
                yield null;
            }
            case "hasPermission" -> true;
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
