package com.lkjmc.common.menu;

import com.lkjmc.common.docs.DocBundle;
import com.lkjmc.common.docs.DocFile;
import com.lkjmc.common.docs.DocPaginator;
import com.lkjmc.common.i18n.MessageCatalog;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;

final class DocsRouteRenderer {
    private static final int[] ROWS = {
        19, 20, 21, 22, 23, 24, 25, 28, 29, 30, 31, 32, 33, 34, 37, 38, 39, 40, 41, 42, 43
    };
    private final DocBundle docs;
    private final MessageCatalog messages;

    DocsRouteRenderer(DocBundle docs, MessageCatalog messages) {
        this.docs = docs;
        this.messages = messages;
    }

    List<MenuFrame.Slot> render(MenuRoute route, Map<String, String> params, String locale,
                                long session, long revision) {
        var slots = new ArrayList<MenuFrame.Slot>();
        switch (route.dynamic().binding()) {
            case DOCS_DIRECTORY -> files(route, docs.files(), locale, session, revision, slots);
            case DOCS_SEARCH -> files(route, docs.search(params.getOrDefault("query", "")), locale,
                    session, revision, slots);
            case DOCS_FILE -> file(route, params, locale, session, revision, slots);
            case DOCS_LINKS -> links(route, params, locale, session, revision, slots);
        }
        if (slots.isEmpty()) {
            String key = route.dynamic().emptyNameKey() == null
                    ? "menu.docs.search.empty" : route.dynamic().emptyNameKey();
            slots.add(inert(route, 22, "PAPER", localized(locale, key), session, revision));
        }
        return slots;
    }

    private void files(MenuRoute route, List<DocFile> files, String locale, long session,
                       long revision, List<MenuFrame.Slot> slots) {
        int index = 0;
        for (var file : files) {
            var action = new MenuAction.Navigate("docs-file",
                    Map.of("path", file.path(), "page", "0"));
            slots.add(slot(route, ROWS[index++], "BOOK", file.title(), List.of(file.path()),
                    MenuTypes.Role.NAVIGATION, action, session, revision));
            if (index == ROWS.length) break;
        }
    }

    private void file(MenuRoute route, Map<String, String> params, String locale, long session,
                      long revision, List<MenuFrame.Slot> slots) {
        var value = docs.file(params.get("path"));
        if (value.isEmpty()) return;
        int requested;
        try {
            requested = Integer.parseInt(params.getOrDefault("page", "0"));
        } catch (NumberFormatException failure) {
            requested = 0;
        }
        var page = DocPaginator.page(value.get(), requested, 36);
        for (int index = 0; index < page.lines().size(); index++) {
            slots.add(inert(route, ROWS[index], "PAPER", page.lines().get(index), session, revision));
        }
        if (page.page() > 0) {
            slots.add(slot(route, 46, "ARROW", localized(locale, "docs.previous"), List.of(),
                    MenuTypes.Role.NAVIGATION, pageAction(value.get().path(), page.page() - 1),
                    session, revision));
        }
        if (page.page() + 1 < page.pageCount()) {
            slots.add(slot(route, 48, "ARROW", localized(locale, "docs.next"), List.of(),
                    MenuTypes.Role.NAVIGATION, pageAction(value.get().path(), page.page() + 1),
                    session, revision));
        }
        if (!value.get().links().isEmpty()) {
            slots.add(slot(route, 51, "LECTERN", localized(locale, "docs.links"), List.of(),
                    MenuTypes.Role.NAVIGATION,
                    new MenuAction.Navigate("docs-links", Map.of(
                            "path", value.get().path(), "page", Integer.toString(page.page()))),
                    session, revision));
        }
    }

    private void links(MenuRoute route, Map<String, String> params, String locale, long session,
                       long revision, List<MenuFrame.Slot> slots) {
        var source = docs.file(params.get("path"));
        if (source.isEmpty()) return;
        int index = 0;
        for (var link : source.get().links()) {
            var target = target(source.get().path(), link.target());
            MenuAction action = target == null
                    ? new MenuAction.Simple(MenuTypes.ActionType.NONE)
                    : new MenuAction.Navigate("docs-file", Map.of("path", target, "page", "0"));
            slots.add(slot(route, ROWS[index++], "PAPER", link.text(), List.of(),
                    target == null ? MenuTypes.Role.INFO : MenuTypes.Role.NAVIGATION,
                    action, session, revision));
            if (index == ROWS.length) break;
        }
    }

    private String target(String source, String raw) {
        if (raw == null || raw.contains(":") || raw.startsWith("/") || raw.startsWith("#")) return null;
        String withoutAnchor = raw.split("#", 2)[0];
        Path parent = Path.of(source).getParent();
        String candidate = (parent == null ? Path.of(withoutAnchor) : parent.resolve(withoutAnchor))
                .normalize().toString().replace('\\', '/');
        return docs.file(candidate).isPresent() ? candidate : null;
    }

    private MenuAction pageAction(String path, int page) {
        return new MenuAction.Navigate("docs-file",
                Map.of("path", path, "page", Integer.toString(page)));
    }

    private MenuFrame.Slot inert(MenuRoute route, int index, String material, String name,
                                 long session, long revision) {
        return slot(route, index, material, name, List.of(), MenuTypes.Role.INFO,
                new MenuAction.Simple(MenuTypes.ActionType.NONE), session, revision);
    }

    private MenuFrame.Slot slot(MenuRoute route, int index, String material, String name,
                                List<String> lore, MenuTypes.Role role, MenuAction action,
                                long session, long revision) {
        String clean = name.replaceAll("<[^>]+>", "").trim();
        return MenuFrame.slot(index, material, clean.isBlank() ? " " : clean, lore, role, action,
                route.id(), session, revision);
    }

    private String localized(String locale, String key) {
        return messages.render(locale, key).replaceAll("<[^>]+>", "").trim();
    }
}
