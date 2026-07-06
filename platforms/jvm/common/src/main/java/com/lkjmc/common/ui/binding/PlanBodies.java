package com.lkjmc.common.ui.binding;

import java.util.LinkedHashMap;
import java.util.Map;

final class PlanBodies {
    private PlanBodies() {}

    static Map<String, String> forBinding(String id, BindingContext ctx) {
        var body = new LinkedHashMap<String, String>();
        switch (id) {
            case "homes", "settings", "daily", "party", "profile",
                "achievements", "achievement-directory", "achievement-detail" -> player(body, ctx);
            case "home-detail" -> {
                player(body, ctx);
                ctx.param("home").ifPresent(value -> body.put("home", value));
            }
            case "claims" -> body.put("ownerUuid", ctx.playerUuid());
            case "claim-detail" -> ctx.param("claimId").ifPresent(value -> body.put("claimId", value));
            case "mail" -> {
                player(body, ctx);
                body.put("limit", "14");
            }
            case "reports", "report-detail", "admin-moderation" -> body.put("limit", "14");
            case "random-teleport" -> {
                player(body, ctx);
                body.put("profileId", ctx.param("profileId").orElse("overworld"));
                ctx.param("serverId").ifPresent(value -> body.put("serverId", value));
            }
            case "shop", "kits", "votes", "warps", "adventures", "admin-audit",
                "admin-config", "admin-economy", "admin-security", "admin-web" -> { }
            case "server-list", "admin-servers" -> {
                body.put("principalKind", "minecraft-player");
                body.put("principalId", ctx.playerUuid());
                body.put("principalName", ctx.playerName());
                body.put("platformPermission", Boolean.toString(ctx.permissions().listServers()));
            }
            case "admin-server-detail" -> ctx.param("serverId").ifPresent(value -> body.put("id", value));
            case "admin-server-create-kind" -> create(body, "paper", "paper-survival", ctx);
            case "admin-server-create-template" -> {
                var kind = ctx.param("kind").orElse("paper");
                create(body, kind, template(kind), ctx);
            }
            case "teleports" -> {
                player(body, ctx);
                body.put("name", ctx.playerName());
                body.put("sourceInstance", ctx.param("serverId").orElse("unknown"));
            }
            default -> body.putAll(ctx.params());
        }
        return Map.copyOf(body);
    }

    private static void player(Map<String, String> body, BindingContext ctx) {
        body.put("playerUuid", ctx.playerUuid());
        if (!ctx.playerName().isBlank()) {
            body.put("name", ctx.playerName());
        }
    }

    private static void create(Map<String, String> body, String kind, String template,
                               BindingContext ctx) {
        body.put("kind", kind);
        body.put("template", template);
        body.put("id", template + "-001");
        body.put("acceptMinecraftEula", "true");
        ctx.params().forEach(body::putIfAbsent);
    }

    private static String template(String kind) {
        return "velocity".equals(kind) ? "velocity-modern" : "paper-survival";
    }
}
