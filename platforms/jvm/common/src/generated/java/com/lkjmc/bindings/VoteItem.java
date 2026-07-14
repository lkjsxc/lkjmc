package com.lkjmc.bindings;

public record VoteItem(
        String id,
        String titleKey,
        String url
) {
    public VoteItem {
        java.util.Objects.requireNonNull(id, "id");
        if (id.isBlank()) throw new IllegalArgumentException("id");
        java.util.Objects.requireNonNull(titleKey, "titleKey");
        if (titleKey.isBlank()) throw new IllegalArgumentException("titleKey");
        java.util.Objects.requireNonNull(url, "url");
        if (url.isBlank()) throw new IllegalArgumentException("url");
    }
}
