package com.lkjmc.velocity;

import com.lkjmc.common.permission.PermissionNodes;
import com.velocitypowered.api.command.CommandSource;
import com.velocitypowered.api.proxy.Player;
import java.util.HashMap;
import java.util.Map;
import java.util.function.BiPredicate;

final class VelocityCommandPrincipal {
    private VelocityCommandPrincipal() {}

    static Map<String, Object> body(
        CommandSource source,
        String command,
        Map<String, Object> body,
        BiPredicate<CommandSource, String> permissionCheck
    ) {
        var values = new HashMap<String, Object>(body);
        values.put("platformPermission", permissionCheck.test(source, permission(command)));
        if (source instanceof Player player) {
            values.put("principalKind", "minecraft-player");
            values.put("principalId", player.getUniqueId().toString());
            values.put("principalName", player.getUsername());
        }
        return values;
    }

    private static String permission(String command) {
        return switch (command) {
            case "status", "doctor" -> PermissionNodes.ADMIN_STATUS;
            case "config.reload" -> PermissionNodes.ADMIN_RELOAD;
            case "instance.list" -> PermissionNodes.ADMIN_INSTANCE_LIST;
            case "instance.create" -> PermissionNodes.ADMIN_INSTANCE_CREATE;
            case "instance.start" -> PermissionNodes.ADMIN_INSTANCE_START;
            case "instance.stop" -> PermissionNodes.ADMIN_INSTANCE_STOP;
            case "instance.restart" -> PermissionNodes.ADMIN_INSTANCE_RESTART;
            case "instance.delete" -> PermissionNodes.ADMIN_INSTANCE_DELETE;
            default -> PermissionNodes.ADMIN_STATUS;
        };
    }
}
