package com.lkjmc.paper.harness;

import com.lkjmc.common.menu.MenuAction;
import com.lkjmc.common.menu.MenuFrame;
import com.lkjmc.common.menu.MenuResult;
import com.lkjmc.common.menu.MenuTypes;
import com.lkjmc.paper.MenuSessionOwnership;
import com.lkjmc.paper.PaperMenuProtocolAdapter;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;
import java.util.UUID;
import java.util.zip.ZipFile;

public final class MenuProbeRunner {
    private static final List<String> NAMES = List.of("all-routes-selected-engine",
            "golden-state-matrix", "no-unintended-close", "local-menu-daemon-independent",
            "locale-parity-quality", "protocol-menu-pass", "old-menu-engine-absent");
    private final MenuHarnessFixtures fixtures = new MenuHarnessFixtures();
    private Path paperJar;

    public static void main(String[] args) throws Exception {
        var runner = new MenuProbeRunner();
        runner.paperJar = Path.of(args[0]);
        String output = System.getProperty("lkjmc.menu.goldens.write");
        if (output != null) {
            new MenuGoldenSet(runner.fixtures).write(Path.of(output));
            return;
        }
        String selected = System.getProperty("lkjmc.menu.probe", "all");
        if (!selected.equals("all") && !NAMES.contains(selected)) {
            throw new IllegalArgumentException("unknown menu probe");
        }
        for (String name : NAMES) {
            if (selected.equals("all") || selected.equals(name)) {
                runner.run(name);
                System.out.println(name + "=PASS");
            }
        }
    }

    private void run(String name) throws Exception {
        switch (name) {
            case "all-routes-selected-engine" -> allRoutes();
            case "golden-state-matrix" -> new MenuGoldenSet(fixtures).verify();
            case "no-unintended-close" -> noClose();
            case "local-menu-daemon-independent" -> localOnly();
            case "locale-parity-quality" -> locale();
            case "protocol-menu-pass" -> protocol();
            case "old-menu-engine-absent" -> oldEngine();
            default -> throw new AssertionError(name);
        }
    }

    private void allRoutes() {
        require(fixtures.bundle.routes().size() == 5, "route count");
        for (var route : fixtures.bundle.routes()) {
            var result = adapter().open(1, route.id(), params(route.parameterMap()), "en");
            require(result instanceof MenuResult.Rendered, "route not rendered: " + route.id());
            require(((MenuResult.Rendered) result).frame().route().equals(route.id()), "route mismatch");
        }
    }

    private void noClose() {
        var adapter = adapter();
        var root = rendered(adapter.open(2, "root", Map.of(), "en"));
        var docs = adapter.click(root.bySlot().get(15).metadata(), root.bySlot().get(15).action());
        require(docs instanceof MenuResult.Rendered, "navigation closed inventory");
        var backFrame = ((MenuResult.Rendered) docs).frame();
        var back = backFrame.bySlot().get(49);
        require(adapter.click(back.metadata(), back.action()) instanceof MenuResult.Rendered,
                "back closed inventory");
        var current = adapter.frame();
        var close = current.bySlot().get(26);
        require(adapter.click(close.metadata(), close.action()) instanceof MenuResult.Closed,
                "close not explicit");
    }

    private void localOnly() {
        var root = rendered(adapter().open(3, "root", Map.of(), "en"));
        require(root.slots().stream().anyMatch(slot -> slot.lore().stream()
                .anyMatch(line -> line.contains("/lkjmc status"))), "command guidance absent");
        var docs = rendered(adapter().open(4, "docs-directory", Map.of(), "en"));
        require(docs.slots().stream().anyMatch(slot -> slot.material().equals("BOOK")),
                "bundled docs absent");
        fixtures.bundle.routes().forEach(route -> {
            require(!route.chrome().refresh(), "remote refresh exposed");
            require(route.confirmation() == null, "mutation confirmation exposed");
            if (route.dynamic() != null) {
                require(route.dynamic().binding().name().startsWith("DOCS_"),
                        "non-doc dynamic binding exposed");
            }
        });
    }

    private void locale() {
        var en = rendered(adapter().open(5, "root", Map.of(), "en"));
        var ja = rendered(adapter().open(5, "root", Map.of(), "ja"));
        require(!en.title().equals(ja.title()), "locale not selected");
        for (var frame : List.of(en, ja)) {
            require(!frame.title().isBlank() && !frame.title().contains("<"), "color-only title");
            frame.slots().forEach(slot -> require(
                    !slot.name().isBlank() && !slot.name().contains("<"), "bad label"));
        }
    }

    private void protocol() {
        var adapter = adapter();
        var root = rendered(adapter.open(6, "root", Map.of(), "en"));
        var row = root.bySlot().get(15);
        var missing = adapter.click(row.metadata(), new MenuAction.Navigate("docs-file", Map.of()));
        require(missing instanceof MenuResult.Failed failed
                && failed.failure() == MenuTypes.Failure.MISSING_PARAMETER,
                "missing dynamic parameters admitted");
        require(adapter.click(row.metadata(), row.action()) instanceof MenuResult.Rendered,
                "failed navigation corrupted session");
        require(((MenuResult.Failed) adapter.click(row.metadata(), row.action())).failure()
                == MenuTypes.Failure.STALE_RENDER, "old row accepted");
        var japanese = rendered(adapter().open(7, "docs-search", Map.of("query", "menu"), "ja"));
        require(!japanese.title().isBlank(), "locale sequence failed");
        docsActionGraph();
        directBack();
        ownership();
    }

    private void docsActionGraph() {
        var adapter = adapter();
        var directory = rendered(adapter.open(8, "docs-directory", Map.of(), "en"));
        var fileRow = directory.slots().stream()
                .filter(slot -> slot.lore().contains("docs/product/gui/README.md"))
                .findFirst().orElseThrow();
        var file = rendered(adapter.click(fileRow.metadata(), fileRow.action()));
        require(file.route().equals("docs-file"), "directory did not open file");
        var linksRow = file.bySlot().get(51);
        require(linksRow != null, "file links action absent");
        var links = rendered(adapter.click(linksRow.metadata(), linksRow.action()));
        require(links.route().equals("docs-links"), "file did not open links");
        var target = links.slots().stream()
                .filter(slot -> slot.action().type() == MenuTypes.ActionType.NAVIGATE)
                .findFirst().orElseThrow();
        require(rendered(adapter.click(target.metadata(), target.action())).route().equals("docs-file"),
                "in-corpus link did not open file");
    }

    private void directBack() {
        var cases = List.of(
                Map.entry("docs-directory", Map.<String, String>of()),
                Map.entry("docs-file", Map.of("path", "docs/product/gui/README.md", "page", "0")),
                Map.entry("docs-links", Map.of("path", "docs/product/gui/README.md", "page", "0")),
                Map.entry("docs-search", Map.of("query", "menu")));
        for (var item : cases) {
            var adapter = adapter();
            var opened = rendered(adapter.open(9, item.getKey(), item.getValue(), "en"));
            var back = opened.bySlot().get(49);
            require(back != null && adapter.click(back.metadata(), back.action()) instanceof MenuResult.Rendered,
                    "direct route Back failed: " + item.getKey());
        }
    }

    private void ownership() {
        var owners = new MenuSessionOwnership<Object>();
        var player = UUID.fromString("00000000-0000-0000-0000-000000000179");
        var first = new Object();
        owners.open(player, first);
        require(owners.active(player).orElseThrow() == first, "menu owner missing");
        owners.advance(player, first);
        var replacement = new Object();
        owners.open(player, replacement);
        boolean staleRejected = false;
        try {
            owners.advance(player, first);
        } catch (IllegalStateException expected) {
            staleRejected = true;
        }
        require(staleRejected, "replaced adapter retained ownership");
        owners.invalidate(player);
        require(owners.active(player).isEmpty(), "closed menu retained ownership");
        owners.open(player, replacement);
        owners.disable();
        require(owners.activeOwners() == 0, "ownership leaked after shutdown");
    }

    private void oldEngine() throws Exception {
        try (var jar = new ZipFile(paperJar.toFile())) {
            var names = jar.stream().map(java.util.zip.ZipEntry::getName).toList();
            require(names.contains("com/lkjmc/paper/PaperMenuAdapter.class"), "selected adapter absent");
            require(names.contains("com/lkjmc/paper/PaperMenuProtocolAdapter.class"), "protocol adapter absent");
            require(names.contains("lkjmc-menu-bundle.json"), "compiled menu bundle absent");
            var removed = List.of(
                    "com/lkjmc/paper/PaperMenuSnapshots.class",
                    "com/lkjmc/common/menu/MenuSnapshotView.class",
                    "com/lkjmc/common/menu/MenuAction$Mutation.class",
                    "com/lkjmc/bindings/MenuSnapshot.class",
                    "com/lkjmc/bindings/MenuPayload.class",
                    "com/lkjmc/bindings/ShopItem.class",
                    "com/lkjmc/bindings/KitItem.class",
                    "com/lkjmc/bindings/VoteItem.class",
                    "com/lkjmc/bindings/PluginItem.class");
            require(removed.stream().noneMatch(names::contains),
                    "removed remote or mutation menu code packaged");
        }
    }

    private PaperMenuProtocolAdapter adapter() {
        return new PaperMenuProtocolAdapter(fixtures.bundle, fixtures.renderer);
    }

    private static MenuFrame rendered(MenuResult result) {
        require(result instanceof MenuResult.Rendered, "render expected");
        return ((MenuResult.Rendered) result).frame();
    }

    private static Map<String, String> params(Map<String, Boolean> fields) {
        var values = new TreeMap<String, String>();
        fields.forEach((key, required) -> {
            if (required) {
                values.put(key, switch (key) {
                    case "page" -> "0";
                    case "path" -> "docs/product/gui/README.md";
                    case "query" -> "menu";
                    default -> "sample";
                });
            }
        });
        return values;
    }

    private static void require(boolean condition, String message) {
        if (!condition) throw new AssertionError(message);
    }
}
