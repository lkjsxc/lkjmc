package com.lkjmc.common.menu;

public record VoteMenuEntry(String id, String titleKey, String url) {
    public VoteMenuEntry {
        if (id == null || id.isBlank()) {
            throw new IllegalArgumentException("vote id is required");
        }
        if (titleKey == null || titleKey.isBlank()) {
            titleKey = id;
        }
        url = url == null ? "" : url;
    }
}
