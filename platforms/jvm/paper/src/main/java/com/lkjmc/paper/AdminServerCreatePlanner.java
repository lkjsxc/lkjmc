package com.lkjmc.paper;

import com.google.gson.JsonObject;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonClient;
import com.lkjmc.common.daemon.DaemonRequest;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import org.bukkit.entity.Player;

final class AdminServerCreatePlanner {
    private final Optional<DaemonClient> daemon;

    AdminServerCreatePlanner(Optional<DaemonClient> daemon) {
        this.daemon = daemon == null ? Optional.empty() : daemon;
    }

    CompletableFuture<Plan> plan(Player player, String id, String kind, String template) {
        if (daemon.isEmpty()) {
            return CompletableFuture.failedFuture(MenuDataException.missingDaemon());
        }
        var body = Map.<String, Object>of("id", id, "kind", kind, "template", template, "acceptMinecraftEula", true);
        var request = new DaemonRequest(UUID.randomUUID(), new DaemonActor("paper-plugin", player.getName()),
            "instance.create.plan", body);
        return daemon.get().send(request).thenApply(response -> {
            if (!response.ok()) {
                throw MenuDataException.response("instance.create.plan", response);
            }
            return from(response.body());
        });
    }

    private static Plan from(JsonObject body) {
        var startable = body.has("startable") && body.get("startable").getAsBoolean();
        var text = startable && body.has("createPlan") ? body.get("createPlan").toString()
            : diagnosticText(body);
        return new Plan(startable, text);
    }

    private static String diagnosticText(JsonObject body) {
        var diagnostic = body.has("diagnostic") && body.get("diagnostic").isJsonObject()
            ? body.getAsJsonObject("diagnostic") : null;
        if (diagnostic == null && body.has("diagnostics") && body.get("diagnostics").isJsonArray()
            && body.getAsJsonArray("diagnostics").size() > 0
            && body.getAsJsonArray("diagnostics").get(0).isJsonObject()) {
            diagnostic = body.getAsJsonArray("diagnostics").get(0).getAsJsonObject();
        }
        if (diagnostic == null) return body.has("diagnostics") ? body.get("diagnostics").toString() : "";
        var message = string(diagnostic, "message").orElse(diagnostic.toString());
        return string(diagnostic, "suggestedCommand").map(command -> message + " Suggested: " + command)
            .orElse(message);
    }

    private static Optional<String> string(JsonObject object, String key) {
        return object.has(key) && object.get(key).isJsonPrimitive()
            ? Optional.of(object.get(key).getAsString()) : Optional.empty();
    }

    record Plan(boolean startable, String diagnostics) {}
}
