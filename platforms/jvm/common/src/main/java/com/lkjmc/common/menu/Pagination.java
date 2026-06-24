package com.lkjmc.common.menu;

public record Pagination(int page, int pageSize, int totalItems) {
    public Pagination {
        if (page < 0 || pageSize <= 0 || totalItems < 0) {
            throw new IllegalArgumentException("invalid pagination");
        }
    }

    public boolean hasNext() {
        return (page + 1) * pageSize < totalItems;
    }

    public boolean hasPrevious() {
        return page > 0;
    }
}
