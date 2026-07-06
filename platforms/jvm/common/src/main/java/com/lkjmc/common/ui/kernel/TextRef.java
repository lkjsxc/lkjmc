package com.lkjmc.common.ui.kernel;

import java.util.Map;

public sealed interface TextRef permits TextRef.Key, TextRef.Literal {
    static TextRef key(String key) {
        return new Key(key, Map.of());
    }

    static TextRef key(String key, Map<String, String> args) {
        return new Key(key, args);
    }

    static TextRef literal(String value) {
        return new Literal(value);
    }

    record Key(String key, Map<String, String> args) implements TextRef {
        public Key {
            if (key == null || key.isBlank()) {
                throw new IllegalArgumentException("text key is required");
            }
            args = Map.copyOf(args == null ? Map.of() : args);
        }
    }

    record Literal(String value) implements TextRef {
        public Literal {
            value = value == null ? "" : value;
        }
    }
}
