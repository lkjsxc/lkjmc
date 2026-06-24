package com.lkjmc.common.transfer;

import java.nio.charset.StandardCharsets;
import java.util.Optional;
import java.util.UUID;

public final class ProfileTransferMessages {
    public static final String CHANNEL = "lkjmc:profile";

    private ProfileTransferMessages() {}

    public static byte[] saveRequest(UUID requestId) {
        return ("save:" + requestId).getBytes(StandardCharsets.UTF_8);
    }

    public static byte[] saved(UUID requestId) {
        return ("saved:" + requestId).getBytes(StandardCharsets.UTF_8);
    }

    public static byte[] transferRequest(String server) {
        return ("transfer:" + server).getBytes(StandardCharsets.UTF_8);
    }

    public static Optional<String> parseText(String prefix, byte[] bytes) {
        var text = new String(bytes, StandardCharsets.UTF_8);
        if (!text.startsWith(prefix + ":")) {
            return Optional.empty();
        }
        return Optional.of(text.substring(prefix.length() + 1));
    }

    public static Optional<UUID> parse(String prefix, byte[] bytes) {
        var text = new String(bytes, StandardCharsets.UTF_8);
        if (!text.startsWith(prefix + ":")) {
            return Optional.empty();
        }
        try {
            return Optional.of(UUID.fromString(text.substring(prefix.length() + 1)));
        } catch (IllegalArgumentException error) {
            return Optional.empty();
        }
    }
}
