package com.lkjmc.common.menu;

public record ConfirmationSpec(MenuId id, String messageKey, MenuAction confirmAction) {
    public ConfirmationSpec {
        if (id == null || messageKey == null || messageKey.isBlank()) {
            throw new IllegalArgumentException("confirmation id and message key are required");
        }
        if (confirmAction == null) {
            confirmAction = MenuAction.none();
        }
    }
}
