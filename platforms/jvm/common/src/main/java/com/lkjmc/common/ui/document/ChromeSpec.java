package com.lkjmc.common.ui.document;

public record ChromeSpec(
    String info,
    boolean back,
    boolean refresh,
    boolean close,
    boolean mainMenu
) {
    public static ChromeSpec empty() {
        return new ChromeSpec(null, false, false, true, false);
    }
}
