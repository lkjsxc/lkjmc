package com.lkjmc.common.menu;

public record Pagination(int page, int pageSize, int totalItems) {
    public Pagination {
        if (page < 0 || pageSize <= 0 || totalItems < 0) {
            throw new IllegalArgumentException("invalid pagination");
        }
    }

    public int clampedPage() {
        return Math.min(page, Math.max(0, pageCount() - 1));
    }

    public int pageCount() {
        return Math.max(1, (int) Math.ceil(totalItems / (double) pageSize));
    }

    public boolean hasNext() {
        return clampedPage() + 1 < pageCount();
    }

    public boolean hasPrevious() {
        return clampedPage() > 0;
    }

    public PageWindow window() {
        int first = clampedPage() * pageSize;
        return new PageWindow(first, Math.min(totalItems, first + pageSize), totalItems);
    }
}
