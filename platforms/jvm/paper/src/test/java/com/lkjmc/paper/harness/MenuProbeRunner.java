package com.lkjmc.paper.harness;

import com.lkjmc.common.menu.MenuFrame;
import com.lkjmc.common.menu.MenuResult;
import com.lkjmc.common.menu.MenuTypes;
import com.lkjmc.paper.MenuResponseOwnership;
import com.lkjmc.paper.PaperMenuProtocolAdapter;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;
import java.util.UUID;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.zip.ZipFile;

public final class MenuProbeRunner {
    private static final List<String> NAMES = List.of("all-routes-selected-engine",
            "golden-state-matrix", "no-unintended-close", "daemon-outage-truthful",
            "locale-parity-quality", "protocol-menu-pass", "old-menu-engine-absent");
    private final MenuHarnessFixtures fixtures = new MenuHarnessFixtures();
    private Path paperJar;

    public static void main(String[] args) throws Exception {
        var runner = new MenuProbeRunner(); runner.paperJar = Path.of(args[0]);
        String output = System.getProperty("lkjmc.menu.goldens.write");
        if (output != null) { new MenuGoldenSet(runner.fixtures).write(Path.of(output)); return; }
        String selected = System.getProperty("lkjmc.menu.probe", "all");
        if (!selected.equals("all") && !NAMES.contains(selected)) throw new IllegalArgumentException("unknown menu probe");
        for (String name : NAMES) if (selected.equals("all") || selected.equals(name)) {
            runner.run(name); System.out.println(name + "=PASS");
        }
    }

    private void run(String name) throws Exception {
        switch (name) {
            case "all-routes-selected-engine" -> allRoutes();
            case "golden-state-matrix" -> new MenuGoldenSet(fixtures).verify();
            case "no-unintended-close" -> noClose();
            case "daemon-outage-truthful" -> outage();
            case "locale-parity-quality" -> locale();
            case "protocol-menu-pass" -> protocol();
            case "old-menu-engine-absent" -> oldEngine();
            default -> throw new AssertionError(name);
        }
    }

    private void allRoutes() {
        require(fixtures.bundle.routes().size() == 62, "route count");
        for (var route : fixtures.bundle.routes()) {
            var adapter = adapter();
            var result = adapter.open(1, route.id(), params(route.parameterMap()), "en",
                    fixtures.views(MenuTypes.Freshness.UNAVAILABLE));
            require(result instanceof MenuResult.Rendered, "route not rendered: " + route.id());
            require(((MenuResult.Rendered) result).frame().route().equals(route.id()), "route mismatch");
        }
    }

    private void noClose() {
        var adapter = adapter();
        var root = rendered(adapter.open(2, "root", Map.of(), "en", fixtures.views(MenuTypes.Freshness.UNAVAILABLE)));
        var network = adapter.click(root.bySlot().get(19).metadata(), root.bySlot().get(19).action(), false);
        require(network instanceof MenuResult.Rendered, "navigation closed inventory");
        var backFrame = ((MenuResult.Rendered) network).frame();
        var back = backFrame.bySlot().get(49);
        require(adapter.click(back.metadata(), back.action(), false) instanceof MenuResult.Rendered, "back closed inventory");
        var current = adapter.frame(); var close = current.bySlot().get(53);
        require(adapter.click(close.metadata(), close.action(), false) instanceof MenuResult.Closed, "close not explicit");
    }

    private void outage() {
        var adapter = adapter();
        var shop = rendered(adapter.open(3, "shop", Map.of(), "en",
                fixtures.views(MenuTypes.Freshness.UNAVAILABLE)));
        require(shop.slots().stream().anyMatch(slot -> slot.name().equals("Daemon is unavailable.")), "outage hidden");
        adapter = adapter();
        var docs = rendered(adapter.open(4, "docs-directory", Map.of("path", "docs"), "en",
                fixtures.views(MenuTypes.Freshness.UNAVAILABLE)));
        require(docs.slots().stream().anyMatch(slot -> slot.material().equals("BOOK")), "local docs lost in outage");
    }

    private void locale() {
        var en = rendered(adapter().open(5, "root", Map.of(), "en", fixtures.views(MenuTypes.Freshness.CURRENT)));
        var ja = rendered(adapter().open(5, "root", Map.of(), "ja", fixtures.views(MenuTypes.Freshness.CURRENT)));
        require(!en.title().equals(ja.title()), "locale not selected");
        for (var frame : List.of(en, ja)) {
            require(!frame.title().isBlank() && !frame.title().contains("<"), "color-only title");
            frame.slots().forEach(slot -> require(!slot.name().isBlank() && !slot.name().contains("<"), "bad label"));
        }
    }

    private void protocol() {
        var adapter = adapter();
        var root = rendered(adapter.open(6, "root", Map.of(), "en", fixtures.views(MenuTypes.Freshness.CURRENT)));
        var row = root.bySlot().get(20);
        require(adapter.click(row.metadata(), row.action(), false) instanceof MenuResult.Rendered, "click navigation");
        require(((MenuResult.Failed) adapter.click(row.metadata(), row.action(), false)).failure()
                == MenuTypes.Failure.STALE_RENDER, "old row accepted");
        adapter = adapter();
        var shop = rendered(adapter.open(7, "shop", Map.of(), "en", fixtures.views(MenuTypes.Freshness.UNAVAILABLE)));
        var refresh = shop.bySlot().get(50);
        var pending = (MenuResult.Pending) adapter.click(refresh.metadata(), refresh.action(), false);
        require(((MenuResult.Failed) adapter.click(refresh.metadata(), refresh.action(), false)).failure()
                == MenuTypes.Failure.BUSY_SESSION, "repeated click admitted");
        require(((MenuResult.Failed) adapter.response(pending.request() + 1, fixtures.views(MenuTypes.Freshness.CURRENT))).failure()
                == MenuTypes.Failure.STALE_RESPONSE, "stale response admitted");
        require(adapter.response(pending.request(), fixtures.views(MenuTypes.Freshness.STALE)) instanceof MenuResult.Rendered,
                "matching response lost");
        var japanese = rendered(adapter().open(8, "docs-search", Map.of("query", "menu"), "ja",
                fixtures.views(MenuTypes.Freshness.UNAVAILABLE)));
        require(!japanese.title().isBlank(), "locale sequence failed");
        adversarialOwnership();
    }

    private void adversarialOwnership() {
        var hops = new HarnessFakes.PaperHops();
        hops.hold = true;
        var ownership = new MenuResponseOwnership<Object>(hops);
        var player = UUID.fromString("00000000-0000-0000-0000-000000000179");
        var mutations = new AtomicInteger();

        var firstAdapter = new Object();
        var opened = ownership.open(player, firstAdapter, 9, "en", "root", 0);
        var pending = ownership.advance(player, firstAdapter, 9, "en", "root", 1);
        require(pending.adapterInstance() == opened.adapterInstance()
                && pending.generation() > opened.generation(), "generation did not advance");
        ownership.onEntity(pending, ignored -> mutations.incrementAndGet());
        ownership.invalidate(player); // delayed response after close
        hops.release();

        var oldAdapter = new Object();
        var old = ownership.open(player, oldAdapter, 10, "en", "root", 1);
        hops.hold = true;
        ownership.onEntity(old, ignored -> mutations.incrementAndGet());
        var reopened = ownership.open(player, new Object(), 11, "en", "network", 0);
        require(reopened.adapterInstance() != old.adapterInstance()
                && reopened.generation() > old.generation(), "reopen reused ownership");
        hops.release();

        var disconnect = ownership.advance(player, ownership.active(player).orElseThrow(),
                11, "en", "network", 1);
        hops.hold = true;
        ownership.onEntity(disconnect, ignored -> mutations.incrementAndGet());
        ownership.invalidate(player); // disconnect
        hops.release();

        var localeAdapter = new Object();
        var oldLocale = ownership.open(player, localeAdapter, 12, "en", "docs-directory", 1);
        hops.hold = true;
        ownership.onEntity(oldLocale, ignored -> mutations.incrementAndGet());
        ownership.invalidate(player); // locale change
        ownership.open(player, new Object(), 13, "ja", "docs-directory", 0);
        hops.release();

        var shutdown = ownership.advance(player, ownership.active(player).orElseThrow(),
                13, "ja", "docs-directory", 1);
        hops.hold = true;
        ownership.onEntity(shutdown, ignored -> mutations.incrementAndGet());
        ownership.disable();
        hops.release();

        require(mutations.get() == 0, "stale completion mutated UI or chat");
        require(ownership.activeOwners() == 0, "ownership leaked after shutdown");
        require(hops.entity.get() == 5 && hops.global.get() == 0
                && hops.region.get() == 0 && hops.async.get() == 0,
                "menu completion did not use only entity ownership");
    }

    private void oldEngine() throws Exception {
        try (var jar = new ZipFile(paperJar.toFile())) {
            var names = jar.stream().map(java.util.zip.ZipEntry::getName).toList();
            require(names.contains("com/lkjmc/paper/PaperMenuAdapter.class"), "selected adapter absent");
            require(names.contains("com/lkjmc/paper/PaperMenuProtocolAdapter.class"), "protocol adapter absent");
            String oldLocalEngine = "Local" + "Docs" + "Menu";
            require(names.stream().noneMatch(name -> name.contains(oldLocalEngine) || name.contains("com/lkjmc/common/ui/")),
                    "old menu engine packaged");
            require(names.contains("lkjmc-menu-bundle.json"), "compiled menu bundle absent");
        }
    }

    private PaperMenuProtocolAdapter adapter() { return new PaperMenuProtocolAdapter(fixtures.bundle, fixtures.renderer); }
    private static MenuFrame rendered(MenuResult result) {
        require(result instanceof MenuResult.Rendered, "render expected"); return ((MenuResult.Rendered) result).frame();
    }
    private static Map<String, String> params(Map<String, Boolean> fields) {
        var values = new TreeMap<String, String>();
        fields.forEach((key, required) -> { if (required) values.put(key, switch (key) {
            case "page" -> "0"; case "path" -> "docs/product/gui/README.md"; case "query" -> "menu"; default -> "sample";
        }); });
        return values;
    }
    private static void require(boolean condition, String message) { if (!condition) throw new AssertionError(message); }
}
