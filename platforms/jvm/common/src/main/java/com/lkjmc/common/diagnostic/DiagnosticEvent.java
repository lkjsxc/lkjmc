package com.lkjmc.common.diagnostic;

import com.google.gson.annotations.SerializedName;
import java.time.Instant;
import java.util.Map;
import java.util.Set;
import java.util.UUID;

public record DiagnosticEvent(
        String timestamp, Severity severity, Component component, EventKind eventKind,
        UUID requestId, UUID operationId, UUID correlationId,
        String actorKind, String actorName, Surface surface, Outcome outcome,
        String errorClass, Map<String, Object> attributes, String source, String serverId) {
    private static final Set<String> ATTRIBUTE_KEYS = Set.of(
            "command", "serverId", "route", "runtime", "fault", "queue", "reason",
            "migration", "retention", "bundle", "transport", "source");

    public DiagnosticEvent {
        bounded("timestamp", timestamp, 48); bounded("actorKind", actorKind, 32);
        bounded("actorName", actorName, 96); bounded("source", source, 64);
        bounded("serverId", serverId, 96);
        if (severity == null || component == null || eventKind == null || surface == null || outcome == null)
            throw new IllegalArgumentException("closed diagnostic enum required");
        if (errorClass != null) bounded("errorClass", errorClass, 64);
        attributes = Map.copyOf(attributes);
        if (attributes.size() > 12 || !ATTRIBUTE_KEYS.containsAll(attributes.keySet()))
            throw new IllegalArgumentException("diagnostic attributes are not bounded");
        attributes.forEach((key, value) -> {
            if (!(value == null || value instanceof Boolean || value instanceof Number
                    || value instanceof String text && text.length() <= 128 && safe(text)))
                throw new IllegalArgumentException("diagnostic attribute is not scalar: " + key);
        });
    }

    public static DiagnosticEvent local(String serverId, EventKind kind, Outcome outcome,
            Map<String, Object> attributes) {
        UUID operationId = UUID.randomUUID();
        return new DiagnosticEvent(Instant.now().toString(), Severity.INFO, Component.JVM, kind,
                null, operationId, operationId, "plugin", serverId,
                serverId.equals("velocity") ? Surface.VELOCITY : Surface.PAPER,
                outcome, null, attributes, "jvm-local", serverId);
    }

    private static void bounded(String name, String value, int maximum) {
        if (value == null || value.isBlank() || value.length() > maximum || !safe(value))
            throw new IllegalArgumentException(name + " is not bounded or is sensitive");
    }

    private static boolean safe(String value) {
        String lower = value.toLowerCase(java.util.Locale.ROOT);
        return !lower.contains("://") && !lower.contains("bearer ")
                && !lower.contains("password=") && !lower.contains("secret=")
                && !lower.contains("token=") && !lower.contains("cookie=")
                && !lower.contains("csrf=") && !lower.contains("-canary");
    }

    public enum Severity { @SerializedName("debug") DEBUG, @SerializedName("info") INFO,
        @SerializedName("warn") WARN, @SerializedName("error") ERROR }
    public enum Component { @SerializedName("jvm") JVM }
    public enum EventKind { @SerializedName("jvm_diagnostic") JVM_DIAGNOSTIC,
        @SerializedName("sync_diagnostic") SYNC_DIAGNOSTIC,
        @SerializedName("runtime_diagnostic") RUNTIME_DIAGNOSTIC }
    public enum Surface { @SerializedName("paper") PAPER, @SerializedName("velocity") VELOCITY }
    public enum Outcome { @SerializedName("succeeded") SUCCEEDED, @SerializedName("failed") FAILED,
        @SerializedName("degraded") DEGRADED, @SerializedName("cancelled") CANCELLED }
}
