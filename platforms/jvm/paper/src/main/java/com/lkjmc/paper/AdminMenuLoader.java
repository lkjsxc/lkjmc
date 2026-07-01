package com.lkjmc.paper;

import com.lkjmc.common.menu.AdminDynamicMenus;
import com.lkjmc.common.menu.AdminMenuPermissions;
import com.lkjmc.common.menu.MenuId;
import com.lkjmc.common.menu.MenuSpec;
import com.lkjmc.common.permission.PermissionNodes;
import com.lkjmc.common.permission.PrincipalIdentity;
import java.util.Optional;
import org.bukkit.entity.Player;

final class AdminMenuLoader {
    private final LkjmcPaperPlugin plugin;

    AdminMenuLoader(LkjmcPaperPlugin plugin) {
        this.plugin = plugin;
    }

    Optional<MenuSpec> load(Player player, MenuId id) {
        var permissions = permissions(player);
        return switch (id.value()) {
            case "admin" -> Optional.of(AdminDynamicMenus.dashboard(permissions));
            case "admin-servers" -> Optional.of(AdminDynamicMenus.servers(permissions));
            case "admin-config" -> Optional.of(AdminDynamicMenus.config(permissions));
            case "admin-security" -> Optional.of(AdminDynamicMenus.security(permissions));
            case "admin-economy" -> Optional.of(AdminDynamicMenus.economy(permissions));
            case "admin-moderation" -> Optional.of(AdminDynamicMenus.moderation(permissions));
            case "admin-audit" -> Optional.of(AdminDynamicMenus.audit(permissions));
            case "admin-web" -> Optional.of(AdminDynamicMenus.web(permissions));
            default -> Optional.empty();
        };
    }

    AdminMenuPermissions permissions(Player player) {
        return new AdminMenuPermissions(
            allowed(player, PermissionNodes.ADMIN_STATUS),
            allowed(player, PermissionNodes.ADMIN_RELOAD),
            allowed(player, PermissionNodes.ADMIN_ADMIN),
            allowed(player, PermissionNodes.ADMIN_ECONOMY),
            allowed(player, PermissionNodes.ADMIN_ANNOUNCE),
            allowed(player, PermissionNodes.ADMIN_REPORTS),
            allowed(player, PermissionNodes.ADMIN_WARN),
            allowed(player, PermissionNodes.ADMIN_BAN),
            allowed(player, PermissionNodes.ADMIN_MUTE),
            allowed(player, PermissionNodes.ADMIN_CLAIM),
            allowed(player, PermissionNodes.ADMIN_INSTANCE_LIST),
            allowed(player, PermissionNodes.ADMIN_INSTANCE_CREATE),
            allowed(player, PermissionNodes.ADMIN_INSTANCE_START),
            allowed(player, PermissionNodes.ADMIN_INSTANCE_STOP),
            allowed(player, PermissionNodes.ADMIN_INSTANCE_RESTART),
            allowed(player, PermissionNodes.ADMIN_INSTANCE_DELETE));
    }

    private boolean allowed(Player player, String permission) {
        var platform = player.hasPermission(permission) || player.isOp();
        return plugin.adminGrants().decide(identity(player), permission, platform, player.isOp()).allowed();
    }

    private PrincipalIdentity identity(Player player) {
        return new PrincipalIdentity("minecraft-player", player.getUniqueId().toString(), player.getName());
    }
}
