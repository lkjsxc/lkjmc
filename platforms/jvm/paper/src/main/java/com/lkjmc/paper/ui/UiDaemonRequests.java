package com.lkjmc.paper.ui;

import com.google.gson.JsonObject;
import com.lkjmc.common.daemon.DaemonActor;
import com.lkjmc.common.daemon.DaemonRequest;
import com.lkjmc.common.daemon.DaemonResponse;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.CompletionException;
import org.bukkit.Location;
import org.bukkit.entity.Player;

final class UiDaemonRequests {
    private UiDaemonRequests() {}

    static DaemonRequest request(Player player, String command, Map<String, String> values) {
        return new DaemonRequest(UUID.randomUUID(), new DaemonActor("paper-plugin", player.getName()),
            command, body(player, command, values));
    }

    static Map<String, Object> body(Player player, String command, Map<String, String> values) {
        var body = new HashMap<String, Object>();
        body.put("playerUuid", player.getUniqueId().toString());
        body.put("name", player.getName());
        body.put("playerName", player.getName());
        for (var entry : (values == null ? Map.<String, String>of() : values).entrySet()) {
            put(body, entry.getKey(), resolve(player, entry.getValue()));
        }
        if ("player.home.set".equals(command)) {
            body.put("serverId", instanceId());
            body.put("location", homeLocation(player.getLocation()));
        }
        if (command != null && command.startsWith("instance.") && body.containsKey("serverId")) {
            body.putIfAbsent("id", body.get("serverId"));
        }
        return Map.copyOf(body);
    }

    static JsonObject merge(List<DaemonResponse> responses) {
        var merged = new JsonObject();
        for (var response : responses) {
            for (var entry : response.body().entrySet()) {
                merged.add(entry.getKey(), entry.getValue().deepCopy());
            }
        }
        return merged;
    }

    static String diagnostic(Throwable error) {
        var cause = error instanceof CompletionException && error.getCause() != null
            ? error.getCause() : error;
        return cause == null ? "daemon.command_failed" : map(cause.getMessage());
    }

    static String diagnostic(DaemonResponse response) {
        if (response == null) {
            return "daemon.http_failed";
        }
        return response.error().map(value -> map(value.code())).orElse("daemon.command_failed");
    }

    static Map<String, Object> homeLocation(Location location) {
        return Map.of(
            "world", location.getWorld() == null ? "world" : location.getWorld().getName(),
            "x", location.getX(),
            "y", location.getY(),
            "z", location.getZ(),
            "yaw", (double) location.getYaw(),
            "pitch", (double) location.getPitch()
        );
    }

    private static void put(Map<String, Object> body, String key, String value) {
        body.put("homeName".equals(key) ? "home" : key, typed(value));
    }

    private static Object typed(String value) {
        if ("true".equalsIgnoreCase(value) || "false".equalsIgnoreCase(value)) {
            return Boolean.parseBoolean(value);
        }
        return value;
    }

    private static String resolve(Player player, String value) {
        if ("@player.uuid".equals(value)) {
            return player.getUniqueId().toString();
        }
        if ("@player.name".equals(value)) {
            return player.getName();
        }
        if ("@instance.id".equals(value)) {
            return instanceId();
        }
        return value == null ? "" : value;
    }

    private static String map(String source) {
        if (source == null || source.isBlank()) {
            return "daemon.command_failed";
        }
        return switch (source) {
            case "command.unknown" -> "daemon.command_unknown";
            case "daemon.invalid_json" -> "daemon.http_failed";
            case "database.not_configured" -> "daemon.database_not_configured";
            case "database.error" -> "daemon.database_unavailable";
            default -> source.startsWith("daemon.") || source.startsWith("menu.") ? source
                : source.contains("permission") ? "menu.permission_denied" : "daemon.command_failed";
        };
    }

    private static String instanceId() {
        return System.getenv().getOrDefault("LKJMC_INSTANCE_ID", "paper");
    }
}
