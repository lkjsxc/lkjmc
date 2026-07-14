package com.lkjmc.bindings;

import java.util.List;

public record ClaimPayload(
        List<ClaimChunk> chunks
) implements DomainPayload {
    public ClaimPayload {
        chunks = List.copyOf(chunks);
    }
}
