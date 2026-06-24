package com.lkjmc.common.menu;

import java.util.List;

public final class StandardMenus {
    private static final ItemSpec INFO = new ItemSpec("COMPASS", "menu.root.info", List.of());
    private static final ItemSpec SETTINGS = new ItemSpec("COMPARATOR", "menu.settings.title", List.of());
    private static final ItemSpec LANGUAGE = new ItemSpec("BOOK", "menu.language.title", List.of());
    private static final ItemSpec SERVER = new ItemSpec("GRASS_BLOCK", "menu.server-list.title", List.of());
    private static final ItemSpec ENGLISH = new ItemSpec("PAPER", "language.english", List.of());
    private static final ItemSpec JAPANESE = new ItemSpec("PAPER", "language.japanese", List.of());
    private static final ItemSpec CONFIRM = new ItemSpec("LIME_WOOL", "menu.confirm.yes", List.of());
    private static final ItemSpec CANCEL = new ItemSpec("RED_WOOL", "menu.confirm.no", List.of());

    private StandardMenus() {}

    public static MenuSpec root() {
        return new MenuSpec(
            new MenuId("root"),
            new MenuTitle("menu.root.title"),
            new MenuSize(54),
            List.of(
                new SlotSpec(4, INFO, MenuAction.none()),
                new SlotSpec(20, SERVER, new MenuAction.Open(new MenuId("server-list"))),
                new SlotSpec(22, SETTINGS, new MenuAction.Open(new MenuId("settings")))
            )
        );
    }

    public static MenuSpec serverList() {
        return new MenuSpec(
            new MenuId("server-list"),
            new MenuTitle("menu.server-list.title"),
            new MenuSize(54),
            List.of(
                new SlotSpec(10, SERVER, new MenuAction.Open(new MenuId("server-detail"))),
                new SlotSpec(46, pageItem("previous"), new MenuAction.Command("page previous")),
                new SlotSpec(47, pageItem("next"), new MenuAction.Command("page next")),
                new SlotSpec(48, pageItem("info"), MenuAction.none()),
                new SlotSpec(49, back(), new MenuAction.Open(new MenuId("root")))
            )
        );
    }

    public static MenuSpec serverDetail() {
        return new MenuSpec(
            new MenuId("server-detail"),
            new MenuTitle("menu.server-detail.title"),
            new MenuSize(54),
            List.of(new SlotSpec(49, back(), new MenuAction.Open(new MenuId("server-list"))))
        );
    }

    public static MenuSpec settings() {
        return new MenuSpec(
            new MenuId("settings"),
            new MenuTitle("menu.settings.title"),
            new MenuSize(54),
            List.of(
                new SlotSpec(22, LANGUAGE, new MenuAction.Open(new MenuId("language"))),
                new SlotSpec(49, back(), new MenuAction.Open(new MenuId("root")))
            )
        );
    }

    public static MenuSpec language() {
        return new MenuSpec(
            new MenuId("language"),
            new MenuTitle("menu.language.title"),
            new MenuSize(54),
            List.of(
                new SlotSpec(20, ENGLISH, new MenuAction.Command("lang en")),
                new SlotSpec(24, JAPANESE, new MenuAction.Command("lang ja")),
                new SlotSpec(49, back(), new MenuAction.Open(new MenuId("settings")))
            )
        );
    }

    public static MenuSpec confirmation(ConfirmationSpec spec) {
        return new MenuSpec(
            spec.id(),
            new MenuTitle(spec.messageKey()),
            new MenuSize(27),
            List.of(
                new SlotSpec(11, CONFIRM, spec.confirmAction()),
                new SlotSpec(15, CANCEL, new MenuAction.Open(new MenuId("root")))
            )
        );
    }

    public static NavigationPolicy navigation() {
        return NavigationPolicy.standard54();
    }

    private static ItemSpec back() {
        return new ItemSpec("ARROW", "menu.back", List.of());
    }

    private static ItemSpec pageItem(String key) {
        return new ItemSpec("ARROW", "menu.page." + key, List.of());
    }
}
