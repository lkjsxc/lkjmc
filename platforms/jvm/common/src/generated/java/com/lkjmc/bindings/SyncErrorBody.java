package com.lkjmc.bindings;

public record SyncErrorBody(
        String code
) {
    public SyncErrorBody {
        java.util.Objects.requireNonNull(code, "code");
        if (code.isBlank()) throw new IllegalArgumentException("code");
    }
}
