package com.lkjmc.common.ui.kernel;

@FunctionalInterface
public interface UiIds {
    String nextSessionId();

    static UiIds constant(String value) {
        return () -> value;
    }
}
