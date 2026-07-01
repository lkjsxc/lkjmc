package com.lkjmc.common.docs;

import java.util.Set;

public final class DocBrowserLayout {
    public static final int FILE_PREVIOUS_SLOT = 21;
    public static final int FILE_CONTENT_SLOT = 22;
    public static final int FILE_NEXT_SLOT = 23;
    public static final int MAIN_MENU_SLOT = 45;
    public static final int PARENT_SLOT = 49;
    public static final int LINKS_SLOT = 52;
    public static final int SEARCH_SLOT = 53;

    private static final Set<Integer> FILE_READING_SLOTS = Set.of(
        FILE_PREVIOUS_SLOT, FILE_CONTENT_SLOT, FILE_NEXT_SLOT);

    private DocBrowserLayout() {
    }

    public static boolean isFileReadingSlot(int slot) {
        return FILE_READING_SLOTS.contains(slot);
    }
}
