package com.lkjmc.paper;

import static org.junit.jupiter.api.Assertions.assertEquals;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.lkjmc.common.command.CommandCompletionContext;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.daemon.DaemonResponse;
import com.lkjmc.common.permission.PermissionNodes;
import com.lkjmc.common.permission.PermissionSnapshotCache;
import com.lkjmc.common.permission.PrincipalIdentity;
import java.lang.reflect.Proxy;
import java.util.List;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.Player;
import org.junit.jupiter.api.Test;

final class PaperLkjmcTabCompleterTest {
    @Test
    void suggestsRootTreeWithPermissions() {
        var completer = new PaperLkjmcTabCompleter(() -> context());
        var sender = sender(PermissionNodes.ADMIN_STATUS, PermissionNodes.ADMIN_INSTANCE_LIST, PermissionNodes.USER_ADVENTURE);
        assertEquals(List.of("adventure", "config", "doctor", "server", "status"), completer.onTabComplete(sender, null, "lkjmc",
            new String[] {""}));
    }

    @Test
    void suggestsSharedServerTreeWithPermissions() {
        var completer = new PaperLkjmcTabCompleter(() -> context());
        var sender = sender(PermissionNodes.ADMIN_INSTANCE_LIST, PermissionNodes.ADMIN_INSTANCE_START);
        assertEquals(List.of("list", "start"), completer.onTabComplete(sender, null, "lkjmc",
            new String[] {"server", ""}));
        assertEquals(List.of("hub"), completer.onTabComplete(sender, null, "lkjmc",
            new String[] {"server", "start", ""}));
    }

    @Test
    void cachedGrantEnablesCompletionWithoutPlatformPermission() {
        var id = UUID.fromString("00000000-0000-0000-0000-000000000123");
        var identity = new PrincipalIdentity("minecraft-player", id.toString(), "Alex");
        var cache = new PermissionSnapshotCache(new GrantDaemon(PermissionNodes.ADMIN_INSTANCE_START),
            "paper-plugin", "test");
        cache.refresh(identity).join();
        var completer = new PaperLkjmcTabCompleter(() -> context(), cache);

        assertEquals(List.of("start"), completer.onTabComplete(player(id), null, "lkjmc",
            new String[] {"server", ""}));
    }

    private static CommandCompletionContext context() {
        return new CommandCompletionContext(List.of("hub"), List.of(), List.of("paper"));
    }

    private static CommandSender sender(String... permissions) {
        var allowed = java.util.Set.of(permissions);
        return proxy(CommandSender.class, (proxy, method, args) -> switch (method.getName()) {
            case "hasPermission" -> allowed.contains((String) args[0]);
            default -> fallback(method.getReturnType());
        });
    }

    private static Player player(UUID id) {
        return proxy(Player.class, (proxy, method, args) -> switch (method.getName()) {
            case "getUniqueId" -> id;
            case "getName" -> "Alex";
            case "hasPermission", "isOp" -> false;
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

    private static final class GrantDaemon implements DaemonClient {
        private final String permission;

        private GrantDaemon(String permission) {
            this.permission = permission;
        }

        @Override
        public CompletableFuture<DaemonResponse> send(DaemonRequest request) {
            var body = new JsonObject();
            var permissions = new JsonArray();
            permissions.add(permission);
            body.add("permissions", permissions);
            return CompletableFuture.completedFuture(new DaemonResponse(
                request.requestId(), true, body, Optional.empty()));
        }
    }
}
