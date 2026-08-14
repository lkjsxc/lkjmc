package com.lkjmc.common.menu;

import com.lkjmc.common.docs.DocBundle;
import com.lkjmc.common.i18n.MessageCatalog;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Map;

public final class MenuRenderer {
    private final MessageCatalog messages;
    private final DocBundle docs;

    public MenuRenderer(MessageCatalog messages, DocBundle docs) {
        this.messages = messages;
        this.docs = docs;
    }

    public MenuFrame render(MenuRoute route, Map<String, String> params, String locale,
                            long session, long revision) {
        var slots = new ArrayList<MenuFrame.Slot>();
        if (route.dynamic() != null) {
            var rendered = new DocsRouteRenderer(docs, messages).render(
                    route, params, locale, session, revision);
            slots.addAll(rendered);
        }
        renderStatic(route, params, locale, session, revision, slots);
        chrome(route, locale, session, revision, slots);
        slots.sort(Comparator.comparingInt(MenuFrame.Slot::index));
        return new MenuFrame(route.id(), text(locale, route.titleKey(), params), route.size(),
                session, revision, slots);
    }

    public String failure(String locale, MenuTypes.Failure failure) {
        return text(locale, switch (failure) {
            case STALE_RENDER -> "menu.error.stale-epoch";
            case UNKNOWN_ACTION -> "menu.error.unknown-action";
            case UNKNOWN_ROUTE, MISSING_PARAMETER -> "menu.error.route-mismatch";
        }, Map.of());
    }

    private void renderStatic(MenuRoute route, Map<String, String> params, String locale,
                              long session, long revision, List<MenuFrame.Slot> slots) {
        for (var source : route.slots()) {
            var action = resolve(source.action(), params);
            slots.add(MenuFrame.slot(source.slot(), source.material(),
                    text(locale, source.nameKey(), params), source.loreKeys().stream()
                            .map(key -> text(locale, key, params)).toList(), source.role(), action,
                    route.id(), session, revision));
        }
    }

    private void chrome(MenuRoute route, String locale, long session, long revision,
                        List<MenuFrame.Slot> slots) {
        if (route.chrome().infoKey() != null) {
            slots.add(slot(4, "NETHER_STAR", text(locale, route.chrome().infoKey(), Map.of()),
                    MenuTypes.Role.INFO, none(), route, session, revision));
        }
        if (route.chrome().mainMenu()) {
            slots.add(slot(45, "COMPASS", text(locale, "menu.main-menu", Map.of()),
                    MenuTypes.Role.NAVIGATION, new MenuAction.Navigate("root", Map.of()),
                    route, session, revision));
        }
        if (route.chrome().back()) {
            slots.add(slot(49, "ARROW", text(locale, "menu.back", Map.of()),
                    MenuTypes.Role.NAVIGATION, new MenuAction.Simple(MenuTypes.ActionType.BACK),
                    route, session, revision));
        }
        if (route.chrome().close()) {
            int index = route.size() == 27 ? 26 : 53;
            slots.add(slot(index, "BARRIER", text(locale, "menu.close", Map.of()),
                    MenuTypes.Role.NAVIGATION, new MenuAction.Simple(MenuTypes.ActionType.CLOSE),
                    route, session, revision));
        }
    }

    private static MenuFrame.Slot slot(int index, String material, String name,
                                       MenuTypes.Role role, MenuAction action, MenuRoute route,
                                       long session, long revision) {
        return MenuFrame.slot(index, material, name, List.of(), role, action,
                route.id(), session, revision);
    }

    private static MenuAction none() {
        return new MenuAction.Simple(MenuTypes.ActionType.NONE);
    }

    private static MenuAction resolve(MenuAction action, Map<String, String> params) {
        if (!(action instanceof MenuAction.Navigate navigate)) return action;
        var resolved = new java.util.LinkedHashMap<String, String>();
        navigate.params().forEach((key, value) -> resolved.put(key,
                value.startsWith("$") ? params.getOrDefault(value.substring(1), "") : value));
        return new MenuAction.Navigate(navigate.route(), resolved);
    }

    private String text(String locale, String key, Map<String, String> params) {
        String value = key.startsWith("literal:") ? key.substring("literal:".length())
                : messages.render(locale, key);
        for (var item : params.entrySet()) {
            value = value.replace("{" + item.getKey() + "}", item.getValue());
        }
        value = value.replaceAll("<[^>]+>", "").trim();
        return value.isBlank() ? " " : value;
    }
}
