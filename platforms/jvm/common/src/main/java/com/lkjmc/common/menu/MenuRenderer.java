package com.lkjmc.common.menu;

import com.lkjmc.bindings.ClaimSnapshot;
import com.lkjmc.bindings.MenuSnapshot;
import com.lkjmc.bindings.SettingsSnapshot;
import com.lkjmc.common.docs.DocBundle;
import com.lkjmc.common.docs.DocPaginator;
import com.lkjmc.common.i18n.MessageCatalog;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;

public final class MenuRenderer {
    private static final int[] INTERIOR = {19,20,21,22,23,24,25,28,29,30,31,32,33,34,37,38,39,40,41,42,43};
    private final MenuBundle bundle;
    private final MessageCatalog messages;
    private final DocBundle docs;

    public MenuRenderer(MenuBundle bundle, MessageCatalog messages, DocBundle docs) {
        this.bundle = bundle; this.messages = messages; this.docs = docs;
    }

    public MenuFrame render(String routeId, Map<String, String> params, String locale,
                            MenuSnapshotView snapshots, long session, long request, long revision) {
        var route = bundle.route(routeId);
        route.params().stream().filter(MenuRoute.Param::required).forEach(item -> {
            if (!params.containsKey(item.name())) throw new IllegalArgumentException("missing parameter " + item.name());
        });
        var slots = new ArrayList<MenuFrame.Slot>();
        for (var source : route.slots()) {
            slots.add(slot(source.slot(), source.material(), text(locale, source.nameKey(), Map.of()),
                    source.loreKeys().stream().map(key -> text(locale, key, Map.of())).toList(),
                    source.role(), source.action(), route, session, request, revision));
        }
        if (route.dynamic() != null) renderDynamic(route, params, locale, snapshots,
                session, request, revision, slots);
        renderChrome(route, params, locale, session, request, revision, slots);
        return new MenuFrame(route.id(), text(locale, route.titleKey(), Map.of()), route.size(),
                session, request, revision, slots);
    }

    public String failure(String locale, MenuTypes.Failure failure) {
        String key = switch (failure) {
            case STALE_RENDER, STALE_RESPONSE -> "menu.error.stale-epoch";
            case BUSY_SESSION -> "menu.loading";
            case PERMISSION_DENIED, UNATTESTED -> "menu.denied";
            case DEPENDENCY_UNAVAILABLE -> "daemon.unavailable";
            default -> "menu.error.unknown-action";
        };
        return text(locale, key, Map.of());
    }

    private void renderDynamic(MenuRoute route, Map<String, String> params, String locale,
                               MenuSnapshotView views, long session, long request, long revision,
                               List<MenuFrame.Slot> slots) {
        if (route.dynamic().binding().name().startsWith("DOCS_")) {
            renderDocs(route, params, locale, session, request, revision, slots); return;
        }
        var unavailable = route.dependencies().stream().map(item -> views.entry(item.domain()))
                .filter(item -> item.freshness() == MenuTypes.Freshness.UNAVAILABLE).findFirst();
        if (unavailable.isPresent()) {
            String key = route.dependencies().stream().anyMatch(item -> item.domain() == MenuTypes.Domain.PERMISSIONS)
                    ? "menu.denied" : "daemon.unavailable";
            slots.add(inert(22, "BARRIER", text(locale, key, Map.of()), route, session, request, revision)); return;
        }
        boolean stale = route.dependencies().stream().map(item -> views.entry(item.domain()))
                .anyMatch(item -> item.freshness() == MenuTypes.Freshness.STALE);
        if (stale) slots.add(inert(8, "CLOCK", text(locale, "menu.stale.warning", Map.of("code", "stale")),
                route, session, request, revision));
        if (route.dynamic().binding() == MenuTypes.Binding.SHOP) renderShop(route, locale, views,
                stale, session, request, revision, slots);
        else if (route.dynamic().binding() == MenuTypes.Binding.KITS) renderKits(route, locale, views,
                stale, session, request, revision, slots);
        else if (route.dynamic().binding() == MenuTypes.Binding.VOTES) renderVotes(route, locale, views,
                session, request, revision, slots);
        else if (route.dynamic().binding() == MenuTypes.Binding.CLAIMS) renderClaims(route, locale, views,
                session, request, revision, slots);
        else if (route.dynamic().binding() == MenuTypes.Binding.SETTINGS) renderSettings(route, locale, views,
                session, request, revision, slots);
        else renderRevision(route, locale, views, session, request, revision, slots);
    }

    private void renderShop(MenuRoute route, String locale, MenuSnapshotView views, boolean stale,
                            long session, long request, long revision, List<MenuFrame.Slot> slots) {
        var value = views.snapshot(MenuTypes.Domain.MENUS).filter(MenuSnapshot.class::isInstance)
                .map(MenuSnapshot.class::cast).orElse(null);
        if (value == null || value.payload().shop().isEmpty()) { empty(route, locale, session, request, revision, slots); return; }
        int index = 0;
        for (var item : value.payload().shop()) {
            var action = stale ? none() : new MenuAction.Mutation(MenuTypes.Operation.SHOP_PURCHASE, "menu.action.shop-purchase");
            slots.add(slot(INTERIOR[index++], "EMERALD", text(locale, item.titleKey(), Map.of()),
                    List.of("Points: " + item.pricePoints()), stale ? MenuTypes.Role.DISABLED : MenuTypes.Role.ACTION,
                    action, route, session, request, revision));
            if (index == INTERIOR.length) break;
        }
    }

    private void renderKits(MenuRoute route, String locale, MenuSnapshotView views, boolean stale,
                            long session, long request, long revision, List<MenuFrame.Slot> slots) {
        var value = views.snapshot(MenuTypes.Domain.MENUS).filter(MenuSnapshot.class::isInstance)
                .map(MenuSnapshot.class::cast).orElse(null);
        if (value == null || value.payload().kits().isEmpty()) { empty(route, locale, session, request, revision, slots); return; }
        int index = 0;
        for (var item : value.payload().kits()) {
            var action = stale ? none() : new MenuAction.Mutation(MenuTypes.Operation.KIT_CLAIM, "menu.action.kit-claim");
            slots.add(slot(INTERIOR[index++], "CHEST", text(locale, item.titleKey(), Map.of()), List.of(),
                    stale ? MenuTypes.Role.DISABLED : MenuTypes.Role.ACTION, action, route, session, request, revision));
            if (index == INTERIOR.length) break;
        }
    }

    private void renderVotes(MenuRoute route, String locale, MenuSnapshotView views,
                             long session, long request, long revision, List<MenuFrame.Slot> slots) {
        var value = views.snapshot(MenuTypes.Domain.MENUS).filter(MenuSnapshot.class::isInstance)
                .map(MenuSnapshot.class::cast).orElse(null);
        if (value == null || value.payload().votes().isEmpty()) { empty(route, locale, session, request, revision, slots); return; }
        int index = 0;
        for (var item : value.payload().votes()) {
            slots.add(inert(INTERIOR[index++], "PAPER", text(locale, item.titleKey(), Map.of()), route, session, request, revision));
            if (index == INTERIOR.length) break;
        }
    }

    private void renderClaims(MenuRoute route, String locale, MenuSnapshotView views, long session,
                              long request, long revision, List<MenuFrame.Slot> slots) {
        var value = views.snapshot(MenuTypes.Domain.CLAIMS).filter(ClaimSnapshot.class::isInstance)
                .map(ClaimSnapshot.class::cast).orElse(null);
        if (value == null || value.payload().chunks().isEmpty()) {
            empty(route, locale, session, request, revision, slots); return;
        }
        int index = 0;
        for (var claim : value.payload().chunks()) {
            var action = new MenuAction.Navigate("claim-detail", Map.of("claimId", claim.claimId().toString()));
            slots.add(slot(INTERIOR[index++], "GOLDEN_SHOVEL", claim.name(), List.of(claim.worldName()),
                    MenuTypes.Role.NAVIGATION, action, route, session, request, revision));
            if (index == INTERIOR.length) break;
        }
    }

    private void renderSettings(MenuRoute route, String locale, MenuSnapshotView views, long session,
                                long request, long revision, List<MenuFrame.Slot> slots) {
        views.snapshot(MenuTypes.Domain.SETTINGS).filter(SettingsSnapshot.class::isInstance)
                .map(SettingsSnapshot.class::cast).ifPresent(value -> slots.add(inert(22, "COMPARATOR",
                        text(locale, "menu.settings.title", Map.of()), route, session, request, revision)));
    }

    private void renderDocs(MenuRoute route, Map<String, String> params, String locale, long session,
                            long request, long revision, List<MenuFrame.Slot> slots) {
        slots.addAll(new DocsRouteRenderer(docs, messages).render(route, params, locale, session, request, revision));
    }

    private void renderRevision(MenuRoute route, String locale, MenuSnapshotView views, long session,
                                long request, long revision, List<MenuFrame.Slot> slots) {
        long source = route.dependencies().stream().mapToLong(item -> views.entry(item.domain()).revision()).max().orElse(0);
        String key = route.chrome().infoKey() == null ? route.titleKey() : route.chrome().infoKey();
        slots.add(slot(22, "PAPER", text(locale, key, Map.of()), List.of("Revision " + source),
                MenuTypes.Role.INFO, none(), route, session, request, revision));
    }

    private void empty(MenuRoute route, String locale, long session, long request, long revision,
                       List<MenuFrame.Slot> slots) {
        String key = route.dynamic().emptyNameKey() == null ? route.titleKey() : route.dynamic().emptyNameKey();
        slots.add(inert(22, "PAPER", text(locale, key, Map.of()), route, session, request, revision));
    }

    private void renderChrome(MenuRoute route, Map<String, String> params, String locale, long session,
                              long request, long revision, List<MenuFrame.Slot> slots) {
        if (route.chrome().infoKey() != null) slots.add(inert(4, "PAPER", text(locale, route.chrome().infoKey(), Map.of()), route, session, request, revision));
        if (route.chrome().mainMenu()) slots.add(slot(45, "COMPASS", text(locale, "menu.main-menu", Map.of()), List.of(), MenuTypes.Role.NAVIGATION,
                new MenuAction.Navigate("root", Map.of()), route, session, request, revision));
        if (route.chrome().back()) slots.add(slot(49, "ARROW", text(locale, "menu.back", Map.of()), List.of(), MenuTypes.Role.NAVIGATION,
                new MenuAction.Simple(MenuTypes.ActionType.BACK), route, session, request, revision));
        if (route.chrome().refresh()) slots.add(slot(50, "CLOCK", text(locale, "menu.refresh", Map.of()), List.of(), MenuTypes.Role.ACTION,
                new MenuAction.Simple(MenuTypes.ActionType.REFRESH), route, session, request, revision));
        if (route.chrome().close()) slots.add(slot(route.size() == 27 ? 26 : 53, "BARRIER", text(locale, "menu.close", Map.of()), List.of(), MenuTypes.Role.NAVIGATION,
                new MenuAction.Simple(MenuTypes.ActionType.CLOSE), route, session, request, revision));
    }

    private MenuFrame.Slot inert(int index, String material, String name, MenuRoute route,
                                 long session, long request, long revision) {
        return slot(index, material, name, List.of(), MenuTypes.Role.INFO, none(), route, session, request, revision);
    }
    private MenuFrame.Slot slot(int index, String material, String name, List<String> lore,
                                MenuTypes.Role role, MenuAction action, MenuRoute route,
                                long session, long request, long revision) {
        return MenuFrame.slot(index, material, name, lore, role, action, route.id(), session, request, revision);
    }
    private static MenuAction none() { return new MenuAction.Simple(MenuTypes.ActionType.NONE); }
    private String text(String locale, String key, Map<String, String> placeholders) {
        String value = key.startsWith("literal:") ? key.substring(8) : messages.render(locale, key);
        for (var item : placeholders.entrySet()) value = value.replace("{" + item.getKey() + "}", item.getValue());
        value = value.replaceAll("<[^>]+>", "").trim(); return value.isBlank() ? " " : value;
    }
}
