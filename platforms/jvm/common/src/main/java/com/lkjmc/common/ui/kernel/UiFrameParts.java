package com.lkjmc.common.ui.kernel;

import com.lkjmc.common.ui.document.DocumentAction;
import com.lkjmc.common.ui.document.ItemRole;
import com.lkjmc.common.ui.document.MenuDocument;
import com.lkjmc.common.ui.document.RegionCatalog;
import com.lkjmc.common.ui.document.StaticSlot;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;

final class UiFrameParts {
    private UiFrameParts() {}

    static FrameSlot entry(int slot, EntryView entry, MenuRoute route, boolean stale) {
        var action = stale && entry.action() instanceof DocumentAction.Daemon
            ? new DocumentAction.Message("menu.stale.action-disabled") : entry.action();
        var role = stale && entry.action() instanceof DocumentAction.Daemon ? ItemRole.DISABLED : entry.role();
        var inert = role.inertByRole() || action instanceof DocumentAction.None;
        return inert ? FrameSlot.inert(slot, entry.material(), entry.name(), entry.lore(), role)
            : FrameSlot.action(slot, entry.material(), entry.name(), entry.lore(), role, action, route.params());
    }

    static FrameSlot staticSlot(StaticSlot slot, MenuRoute route) {
        var lore = slot.lore().stream().map(TextRef::key).toList();
        if (slot.inert()) {
            return FrameSlot.inert(slot.slot(), slot.material(), TextRef.key(slot.name()), lore, slot.role());
        }
        return FrameSlot.action(slot.slot(), slot.material(), TextRef.key(slot.name()), lore, slot.role(),
            slot.action(), route.params());
    }

    static void chrome(Map<Integer, FrameSlot> slots, MenuDocument doc, UiModel model) {
        var info = infoLines(model.phase());
        if (doc.chrome().info() != null) {
            put(slots, FrameSlot.inert(4, "BOOK", TextRef.key(doc.chrome().info()), info, ItemRole.INFO));
        }
        if (doc.chrome().mainMenu()) {
            put(slots, action(45, "NETHER_STAR", "menu.main-menu", List.of("menu.main-menu.lore"),
                new DocumentAction.Open("root", Map.of()), model.route()));
        }
        if (doc.chrome().back()) {
            var parent = doc.id().startsWith("docs-");
            put(slots, action(49, "ARROW", parent ? "menu.parent" : "menu.back",
                List.of(parent ? "menu.parent.lore" : "menu.back.lore"), new DocumentAction.Back(), model.route()));
        }
        if (doc.chrome().refresh() && doc.data() != null && doc.data().source() == MenuDocument.Source.DAEMON) {
            put(slots, action(50, "CLOCK", "menu.refresh", List.of("menu.refresh.lore"),
                new DocumentAction.Refresh(), model.route()));
        }
        if (doc.chrome().close()) {
            put(slots, action(doc.size() == 27 ? 26 : 53, "BARRIER", "menu.close", List.of(),
                new DocumentAction.Close(), model.route()));
        }
        if (doc.size() == 54) {
            border(slots, doc.theme());
        }
    }

    static List<TextRef> infoLines(RoutePhase phase) {
        return switch (phase) {
            case RoutePhase.Loaded loaded -> viewInfo(loaded.view());
            case RoutePhase.Stale stale -> append(viewInfo(stale.view()), TextRef.key("menu.stale.warning",
                Map.of("code", shortCode(stale.diagnosticCode()))));
            case RoutePhase.Diagnostic diagnostic -> List.of(diagnosticTitle(diagnostic.code()),
                diagnosticHint(diagnostic.code()));
            case RoutePhase.Loading loading -> List.of();
            case RoutePhase.Empty empty -> List.of();
            case RoutePhase.Denied denied -> List.of();
            case RoutePhase.Static statik -> List.of();
        };
    }

    static List<TextRef> viewInfo(RouteView view) {
        return switch (view) {
            case RouteView.ListView list -> list.infoLines();
            case RouteView.DetailView detail -> detail.infoLines();
            case RouteView.CustomView custom -> custom.infoLines();
        };
    }

    static void center(Map<Integer, FrameSlot> slots, MenuDocument doc, String material, String name,
                       List<String> lore, ItemRole role) {
        put(slots, FrameSlot.inert(centerSlot(doc), material, TextRef.key(name),
            lore.stream().map(TextRef::key).toList(), role));
    }

    static int centerSlot(MenuDocument doc) {
        var region = region(doc);
        return region.get(region.size() / 2);
    }

    static List<Integer> region(MenuDocument doc) {
        if (doc.list() != null && RegionCatalog.exists(doc.list().region())) {
            return RegionCatalog.require(doc.list().region());
        }
        if (doc.size() == 27) {
            return RegionCatalog.require("confirm-pair");
        }
        return RegionCatalog.require("detail-band");
    }

    static void pageControls(Map<Integer, FrameSlot> slots, MenuRoute route, Pagination page) {
        put(slots, pageControl(46, "previous", page.hasPrevious(), route));
        put(slots, FrameSlot.inert(47, "PAPER", TextRef.key("menu.page.info",
            Map.of("page", Integer.toString(page.clampedPage() + 1), "pages", Integer.toString(page.pageCount()))),
            List.of(), ItemRole.INFO));
        put(slots, pageControl(48, "next", page.hasNext(), route));
    }

    static TextRef diagnosticTitle(String code) {
        return diagnosticRef(code, "title");
    }

    static TextRef diagnosticHint(String code) {
        return diagnosticRef(code, "hint");
    }

    static void put(Map<Integer, FrameSlot> slots, FrameSlot slot) {
        slots.put(slot.slot(), slot);
    }

    private static FrameSlot pageControl(int slot, String direction, boolean enabled, MenuRoute route) {
        var name = TextRef.key("menu.page." + direction);
        if (!enabled) {
            return FrameSlot.inert(slot, "ARROW", name, List.of(), ItemRole.DISABLED);
        }
        return FrameSlot.action(slot, "ARROW", name, List.of(), ItemRole.NAVIGATION,
            new DocumentAction.Page(direction), route.params());
    }

    private static FrameSlot action(int slot, String material, String name, List<String> lore,
                                    DocumentAction action, MenuRoute route) {
        return FrameSlot.action(slot, material, TextRef.key(name), lore.stream().map(TextRef::key).toList(),
            ItemRole.NAVIGATION, action, route.params());
    }

    private static TextRef diagnosticRef(String code, String suffix) {
        if (code != null && code.startsWith("menu.decode.")) {
            return TextRef.key("diagnostic.menu.decode." + suffix, Map.of("route", code.substring(12)));
        }
        return TextRef.key("diagnostic." + (code == null ? "daemon.command_failed" : code) + "." + suffix);
    }

    private static void border(Map<Integer, FrameSlot> slots, String theme) {
        for (var slot : borderSlots()) {
            slots.putIfAbsent(slot, FrameSlot.inert(slot, themeMaterial(theme), TextRef.key("menu.decorative"),
                List.of(), ItemRole.DECORATION));
        }
    }

    private static List<Integer> borderSlots() {
        var slots = new ArrayList<Integer>();
        for (int i = 0; i <= 8; i++) { slots.add(i); }
        for (int i = 45; i <= 53; i++) { slots.add(i); }
        slots.addAll(List.of(9, 18, 27, 36, 17, 26, 35, 44));
        return slots;
    }

    private static String themeMaterial(String theme) {
        return switch (theme) {
            case "root" -> "LIGHT_BLUE_STAINED_GLASS_PANE";
            case "network" -> "CYAN_STAINED_GLASS_PANE";
            case "travel" -> "GREEN_STAINED_GLASS_PANE";
            case "claims" -> "LIME_STAINED_GLASS_PANE";
            case "economy" -> "YELLOW_STAINED_GLASS_PANE";
            case "social" -> "PURPLE_STAINED_GLASS_PANE";
            case "profile" -> "ORANGE_STAINED_GLASS_PANE";
            case "settings" -> "LIGHT_GRAY_STAINED_GLASS_PANE";
            case "staff", "danger" -> "RED_STAINED_GLASS_PANE";
            case "adventure" -> "MAGENTA_STAINED_GLASS_PANE";
            case "docs" -> "BROWN_STAINED_GLASS_PANE";
            default -> "GRAY_STAINED_GLASS_PANE";
        };
    }

    private static List<TextRef> append(List<TextRef> lines, TextRef extra) {
        var next = new ArrayList<>(lines);
        next.add(extra);
        return List.copyOf(next);
    }

    private static String shortCode(String code) {
        if (code == null || code.isBlank()) {
            return "unknown";
        }
        var index = code.lastIndexOf('.');
        return index < 0 ? code : code.substring(index + 1);
    }
}
