package com.lkjmc.common.config;

public record RuntimeConfigValidation(boolean valid, String code) {
    public static RuntimeConfigValidation ok() {
        return new RuntimeConfigValidation(true, "schema.valid");
    }

    public static RuntimeConfigValidation invalid(String code) {
        return new RuntimeConfigValidation(false, code);
    }
}
