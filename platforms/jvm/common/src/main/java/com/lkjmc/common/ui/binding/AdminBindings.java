package com.lkjmc.common.ui.binding;

import java.util.List;

public final class AdminBindings {
    private AdminBindings() {}

    public static List<MenuBinding> bindings() {
        return List.of(
            new AdminServersBinding(), new AdminServerDetailBinding(),
            new AdminCreateKindBinding(), new AdminCreateTemplateBinding(),
            new AdminSectionBinding("admin-config", "status"),
            new AdminSectionBinding("admin-economy", "player.shop.list"),
            new AdminSectionBinding("admin-moderation", "player.report.list"),
            new AdminSectionBinding("admin-security", "admin.role.list", "security.daemon-token.status"),
            new AdminAuditBinding(), new AdminSectionBinding("admin-web", "status"));
    }
}
