package com.lkjmc.bindings;

import java.util.UUID;

public record ProfileAvailable(
        UUID playerUuid,
        String scope,
        long profileRevision,
        String schema,
        String sha256,
        ProfileEnvelope envelope
) implements ProfilePayload {
    public ProfileAvailable {
        java.util.Objects.requireNonNull(playerUuid, "playerUuid");
        java.util.Objects.requireNonNull(scope, "scope");
        if (scope.isBlank()) throw new IllegalArgumentException("scope");
        if (profileRevision < 0) throw new IllegalArgumentException("profileRevision");
        java.util.Objects.requireNonNull(schema, "schema");
        if (schema.isBlank()) throw new IllegalArgumentException("schema");
        java.util.Objects.requireNonNull(sha256, "sha256");
        if (sha256.isBlank()) throw new IllegalArgumentException("sha256");
        java.util.Objects.requireNonNull(envelope, "envelope");
    }
}
