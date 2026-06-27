package com.lkjmc.common.menu;

public record MailMenuEntry(String id, String senderName, String body, boolean read) {
    public MailMenuEntry {
        if (id == null || id.isBlank()) {
            throw new IllegalArgumentException("mail id is required");
        }
        senderName = senderName == null || senderName.isBlank() ? "unknown" : senderName;
        body = body == null ? "" : body;
    }
}
