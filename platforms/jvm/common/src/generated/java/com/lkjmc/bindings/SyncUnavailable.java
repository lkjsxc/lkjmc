package com.lkjmc.bindings;

public record SyncUnavailable(
        SyncErrorBody error
) implements SyncResponse {
    public SyncUnavailable {
        java.util.Objects.requireNonNull(error, "error");
    }
}
