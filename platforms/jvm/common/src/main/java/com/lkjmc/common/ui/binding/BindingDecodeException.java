package com.lkjmc.common.ui.binding;

public final class BindingDecodeException extends RuntimeException {
    private final String code;

    public BindingDecodeException(String code) {
        super(code);
        this.code = code == null || code.isBlank() ? "menu.decode.unknown" : code;
    }

    public String code() {
        return code;
    }
}
