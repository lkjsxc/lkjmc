package com.lkjmc.common.daemon;

import java.util.Map;
import java.util.Optional;
import java.util.UUID;

public record DaemonResponse(UUID requestId, boolean ok, Map<String, Object> body, Optional<DaemonError> error) {
    public DaemonResponse {
        body = Map.copyOf(body == null ? Map.of() : body);
        error = error == null ? Optional.empty() : error;
    }
}
