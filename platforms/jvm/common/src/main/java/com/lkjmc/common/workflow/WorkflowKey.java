package com.lkjmc.common.workflow;

import java.util.UUID;

public record WorkflowKey(
        UUID operationId,
        UUID sessionId,
        UUID playerId,
        long profileRevision,
        long fence,
        UUID correlationId) {
    public WorkflowKey {
        if (operationId == null || sessionId == null || playerId == null || correlationId == null) {
            throw new IllegalArgumentException("complete workflow identity is required");
        }
        if (profileRevision <= 0 || fence <= 0) {
            throw new IllegalArgumentException("positive profile revision and fence are required");
        }
    }
}
