package com.lkjmc.common.daemon;

import com.google.gson.JsonObject;
import java.util.Optional;
import java.util.UUID;

public record DaemonResponse(UUID requestId, boolean ok, JsonObject body, Optional<DaemonError> error) {
    public DaemonResponse {
        if (requestId == null) {
            throw new IllegalArgumentException("request id is required");
        }
        body = body == null ? new JsonObject() : body.deepCopy();
        error = error == null ? Optional.empty() : error;
    }
}
