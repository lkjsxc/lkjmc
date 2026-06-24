package com.lkjmc.common.daemon;

import java.util.Map;
import java.util.UUID;

public record DaemonRequest(UUID requestId, DaemonActor actor, String command, Map<String, Object> body) {
    public DaemonRequest {
        if (requestId == null || actor == null || command == null || command.isBlank()) {
            throw new IllegalArgumentException("request id, actor, and command are required");
        }
        body = Map.copyOf(body == null ? Map.of() : body);
    }
}
