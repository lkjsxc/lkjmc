package com.lkjmc.common.ui.binding;

import com.google.gson.JsonObject;
import com.lkjmc.common.docs.DocBundle;
import com.lkjmc.common.docs.DocFile;
import com.lkjmc.common.docs.DocPaginator;
import com.lkjmc.common.docs.DocPath;
import com.lkjmc.common.ui.document.DocumentAction;
import com.lkjmc.common.ui.document.ItemRole;
import com.lkjmc.common.ui.kernel.EntryView;
import com.lkjmc.common.ui.kernel.FrameSlot;
import com.lkjmc.common.ui.kernel.RouteView;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;

public final class DocsBindings {
    private DocsBindings() {}

    public static List<MenuBinding> bindings() {
        return List.of(new Directory(), new File(), new Links(), new Search());
    }

    private static DocBundle docs(BindingContext ctx, String binding) {
        if (ctx.local().docs() == null) { throw Jsons.fail(binding); }
        return ctx.local().docs();
    }

    private static EntryView child(String parent, String child) {
        var dir = child.endsWith("/");
        var full = parent == null || parent.isBlank() ? child : parent + "/" + child;
        var path = dir ? full.substring(0, full.length() - 1) : full;
        return Views.entry(dir ? "CHEST" : "BOOK", Views.lit(child), List.of(Views.lit(path)),
            ItemRole.NAVIGATION, dir ? Views.open("docs-directory", Map.of("path", path))
                : Views.open("docs-file", Map.of("path", path, "page", "0")));
    }

    private static EntryView result(DocFile file) {
        return Views.entry("BOOK", Views.lit(file.title()), List.of(Views.lit(file.path())),
            ItemRole.NAVIGATION, Views.open("docs-file", Map.of("path", file.path(), "page", "0")));
    }

    private static final class Directory extends BasicBinding {
        Directory() { super("docs-directory", "local", List.of()); }
        @Override public BindingResult decode(JsonObject body, BindingContext ctx) {
            var path = ctx.param("path").orElse("");
            var entries = docs(ctx, id()).children(path).stream().map(child -> child(path, child)).toList();
            return entries.isEmpty() ? BindingResult.empty()
                : Views.data(new RouteView.ListView(entries, List.of(Views.lit(path)), List.of(search(ctx))));
        }
    }

    private static final class Search extends BasicBinding {
        Search() { super("docs-search", "local", List.of()); }
        @Override public BindingResult decode(JsonObject body, BindingContext ctx) {
            var query = ctx.param("query").orElse("");
            var entries = docs(ctx, id()).search(query).stream().map(DocsBindings::result).toList();
            return entries.isEmpty() ? BindingResult.empty()
                : Views.data(new RouteView.ListView(entries, List.of(Views.lit(query)), List.of(search(ctx))));
        }
    }

    private static final class File extends BasicBinding {
        File() { super("docs-file", "local", List.of()); }
        @Override public BindingResult decode(JsonObject body, BindingContext ctx) {
            var path = ctx.param("path").orElse("");
            var pageNumber = parse(ctx.param("page").orElse("0"));
            var file = docs(ctx, id()).file(path);
            if (file.isEmpty()) { return BindingResult.empty(); }
            var page = DocPaginator.page(file.get(), pageNumber, 38);
            var slots = new ArrayList<FrameSlot>();
            slots.add(pageSlot(21, "docs.previous", page.page() > 0, path, page.page() - 1, ctx));
            slots.add(Views.slot(22, "WRITABLE_BOOK", Views.lit(file.get().title()),
                page.lines().stream().map(Views::lit).toList(), ItemRole.INFO,
                new DocumentAction.None(), ctx.params()));
            slots.add(pageSlot(23, "docs.next", page.page() + 1 < page.pageCount(), path, page.page() + 1, ctx));
            if (!file.get().links().isEmpty()) {
                slots.add(Views.keyedSlot(52, "OAK_SIGN", "docs.links", ItemRole.NAVIGATION,
                    Views.open("docs-links", Map.of("path", path, "page", Integer.toString(page.page()))), ctx.params()));
            }
            return Views.data(new RouteView.CustomView("docs-file", slots,
                List.of(Views.lit(file.get().path()), Views.lit((page.page() + 1) + "/" + page.pageCount()))));
        }

        private FrameSlot pageSlot(int slot, String key, boolean enabled, String path, int page,
                                   BindingContext ctx) {
            var action = enabled ? Views.open("docs-file", Map.of("path", path, "page", Integer.toString(page)))
                : Views.disabled(key + ".disabled");
            return Views.keyedSlot(slot, enabled ? "ARROW" : "GRAY_DYE", key,
                enabled ? ItemRole.NAVIGATION : ItemRole.DISABLED, action, ctx.params());
        }
    }

    private static final class Links extends BasicBinding {
        Links() { super("docs-links", "local", List.of()); }
        @Override public BindingResult decode(JsonObject body, BindingContext ctx) {
            var path = ctx.param("path").orElse("");
            var file = docs(ctx, id()).file(path);
            if (file.isEmpty() || file.get().links().isEmpty()) { return BindingResult.empty(); }
            var entries = file.get().links().stream().map(link -> {
                var external = link.target().startsWith("http://") || link.target().startsWith("https://");
                var action = external ? new DocumentAction.Message("docs.external-link", Map.of("url", link.target()))
                    : DocPath.resolve(path, link.target()).filter(p -> docs(ctx, id()).file(p).isPresent())
                        .<DocumentAction>map(p -> Views.open("docs-file", Map.of("path", p, "page", "0")))
                        .orElseGet(() -> Views.disabled("menu.docs.links.empty"));
                return Views.entry(external ? "PAPER" : "MAP", Views.lit(link.text()),
                    List.of(Views.lit(link.target())), external ? ItemRole.ACTION : ItemRole.NAVIGATION, action);
            }).toList();
            return Views.data(new RouteView.ListView(entries, List.of(Views.lit(path))));
        }
    }

    private static FrameSlot search(BindingContext ctx) {
        return Views.keyedSlot(16, "SPYGLASS", "docs.search", ItemRole.NAVIGATION,
            new DocumentAction.Input("docs.search.prompt", "docs search"), ctx.params());
    }

    private static int parse(String value) {
        try { return Math.max(0, Integer.parseInt(value)); } catch (NumberFormatException error) { return 0; }
    }
}
