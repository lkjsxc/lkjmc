package com.lkjmc.common.docs;

import java.net.URI;
import java.util.ArrayDeque;
import java.util.Optional;

public final class DocPath {
    private DocPath() {}

    public static Optional<String> normalize(String input) {
        if (input == null || input.isBlank()) return Optional.of("");
        var value = input.replace('\\', '/').split("#", 2)[0];
        if (value.startsWith("/") || absolute(value)) return Optional.empty();
        var stack = new ArrayDeque<String>();
        for (var part : value.split("/+")) {
            if (part.isBlank() || part.equals(".")) continue;
            if (part.equals("..")) {
                if (stack.isEmpty()) return Optional.empty();
                stack.removeLast();
            } else {
                stack.addLast(part);
            }
        }
        return Optional.of(String.join("/", stack));
    }

    public static Optional<String> resolve(String fromFile, String target) {
        if (target == null || target.startsWith("#")) return normalize(fromFile);
        if (absolute(target)) return Optional.empty();
        var base = fromFile == null || !fromFile.contains("/") ? "" : fromFile.substring(0, fromFile.lastIndexOf('/'));
        return normalize(base.isBlank() ? target : base + "/" + target);
    }

    private static boolean absolute(String value) {
        try {
            return URI.create(value).isAbsolute();
        } catch (IllegalArgumentException error) {
            return true;
        }
    }
}
