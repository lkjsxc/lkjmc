package com.lkjmc.common.docs;

public final class DocPaginator {
    private static final int LINES_PER_PAGE = 10;

    private DocPaginator() {}

    public static DocPage page(DocFile file, int page, int width) {
        var wrapped = DocLineWrapper.wrap(file.lines(), width);
        var pages = Math.max(1, (wrapped.size() + LINES_PER_PAGE - 1) / LINES_PER_PAGE);
        var current = Math.max(0, Math.min(page, pages - 1));
        var start = current * LINES_PER_PAGE;
        var end = Math.min(wrapped.size(), start + LINES_PER_PAGE);
        return new DocPage(file.path(), current, pages, wrapped.subList(start, end));
    }
}
