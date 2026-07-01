package com.lkjmc.paper;

import com.lkjmc.common.docs.DocBrowserLayout;
import com.lkjmc.common.docs.DocBundle;
import com.lkjmc.common.docs.DocPaginator;
import com.lkjmc.common.docs.DocPath;
import com.lkjmc.common.docs.DocRoute;
import com.lkjmc.common.i18n.MessageRenderer;
import java.util.List;
import java.util.Map;
import java.util.function.Consumer;
import net.kyori.adventure.text.Component;
import net.kyori.adventure.text.event.ClickEvent;
import org.bukkit.Bukkit;
import org.bukkit.Material;
import org.bukkit.NamespacedKey;
import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.inventory.InventoryClickEvent;
import org.bukkit.inventory.ItemStack;
import org.bukkit.persistence.PersistentDataType;

public final class DocsCommandAdapter implements CommandExecutor, Listener {
    private static final List<Integer> ENTRY = List.of(10,11,12,13,14,15,16,19,20,21,22,23,24,25,28,29,30,31,32,33,34);
    private final LkjmcPaperPlugin plugin;
    private final MessageRenderer renderer;
    private final NamespacedKey key;
    private final Consumer<Player> mainMenu;
    private DocBundle bundle;

    public DocsCommandAdapter(LkjmcPaperPlugin plugin, MessageRenderer renderer, Consumer<Player> mainMenu) {
        this.plugin = plugin;
        this.renderer = renderer;
        this.key = new NamespacedKey(plugin, "docs-action");
        this.mainMenu = mainMenu;
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (!(sender instanceof Player player)) {
            sender.sendMessage("players only");
            return true;
        }
        if (args.length > 1 && args[0].equalsIgnoreCase("search")) {
            open(player, "search:" + String.join(" ", java.util.Arrays.copyOfRange(args, 1, args.length)));
            return true;
        }
        open(player, args.length == 1 ? "file:" + args[0] + ":0" : "dir:");
        return true;
    }

    @EventHandler
    public void onClick(InventoryClickEvent event) {
        if (!(event.getInventory().getHolder() instanceof DocsMenuHolder holder)) return;
        event.setCancelled(true);
        if (!(event.getWhoClicked() instanceof Player player) || event.getCurrentItem() == null) return;
        var action = event.getCurrentItem().getItemMeta().getPersistentDataContainer().get(key, PersistentDataType.STRING);
        if (action == null) return;
        handle(player, holder.route(), action);
    }

    private void handle(Player player, String route, String action) {
        if (action.equals("main-menu")) mainMenu.accept(player);
        else if (action.equals("parent")) open(player, DocRoute.parent(route));
        else if (action.equals("search")) player.sendMessage(text(player, "docs.search.prompt", Map.of()));
        else if (action.startsWith("external:")) external(player, action.substring(9));
        else open(player, action);
    }

    private void open(Player player, String route) {
        try {
            var docs = bundle();
            var safe = route == null || route.isBlank() ? "dir:" : route;
            if (safe.startsWith("file:")) file(player, docs, safe);
            else if (safe.startsWith("links:")) links(player, docs, safe);
            else if (safe.startsWith("search:")) search(player, docs, safe.substring(7));
            else if (safe.startsWith("dir:")) dir(player, docs, safe.substring(4));
            else dir(player, docs, "");
        } catch (RuntimeException error) {
            player.sendMessage(text(player, "docs.error", Map.of()));
            dir(player, bundle(), "");
        }
    }

    private void dir(Player player, DocBundle docs, String path) {
        var inv = Bukkit.createInventory(new DocsMenuHolder("dir:" + path), 54, "Docs: " + (path.isBlank() ? "/" : path));
        var index = 0;
        for (var child : docs.children(path)) {
            if (index >= ENTRY.size()) break;
            var full = path.isBlank() ? child : path + "/" + child;
            var dir = child.endsWith("/");
            inv.setItem(ENTRY.get(index++), item(dir ? Material.CHEST : Material.BOOK, child, dir ? "dir:" + full.substring(0, full.length() - 1) : "file:" + full + ":0", List.of(full)));
        }
        chrome(player, inv, "dir:" + path, false, false, true);
        player.openInventory(inv);
    }

    private void file(Player player, DocBundle docs, String route) {
        var parts = route.substring(5).split(":", 2);
        var file = docs.file(parts[0]).orElseThrow();
        var page = DocPaginator.page(file, parts.length == 2 ? DocRoute.parsePage(parts[1]) : 0, 38);
        var currentRoute = "file:" + file.path() + ":" + page.page();
        var inv = Bukkit.createInventory(new DocsMenuHolder(currentRoute), 54, "Docs: " + file.title());
        inv.setItem(DocBrowserLayout.FILE_PREVIOUS_SLOT, pageItem(player, currentRoute, "docs.previous", "docs.previous.disabled", -1, page.page() > 0));
        inv.setItem(DocBrowserLayout.FILE_CONTENT_SLOT, item(Material.WRITABLE_BOOK, file.path() + " " + (page.page() + 1) + "/" + page.pageCount(), "", page.lines()));
        inv.setItem(DocBrowserLayout.FILE_NEXT_SLOT, pageItem(player, currentRoute, "docs.next", "docs.next.disabled", 1, page.page() + 1 < page.pageCount()));
        chrome(player, inv, currentRoute, false, false, true);
        inv.setItem(DocBrowserLayout.LINKS_SLOT, item(Material.OAK_SIGN, text(player, "docs.links", Map.of()), "links:" + file.path() + ":" + page.page(), List.of()));
        player.openInventory(inv);
    }

    private void links(Player player, DocBundle docs, String route) {
        var path = route.substring(6).split(":", 2)[0];
        var file = docs.file(path).orElseThrow();
        var inv = Bukkit.createInventory(new DocsMenuHolder(route), 54, "Links: " + file.title());
        for (var i = 0; i < Math.min(ENTRY.size(), file.links().size()); i++) {
            var link = file.links().get(i);
            var external = link.target().startsWith("http://") || link.target().startsWith("https://");
            var target = external ? "external:" + link.target() : DocPath.resolve(path, link.target()).filter(p -> docs.file(p).isPresent()).map(p -> "file:" + p + ":0").orElse("");
            inv.setItem(ENTRY.get(i), item(external ? Material.PAPER : Material.MAP, link.text(), target, List.of(link.target())));
        }
        chrome(player, inv, route, false, false, true);
        player.openInventory(inv);
    }

    private void search(Player player, DocBundle docs, String query) {
        var inv = Bukkit.createInventory(new DocsMenuHolder("search:" + query), 54, "Docs search: " + query);
        var results = docs.search(query);
        for (var i = 0; i < Math.min(ENTRY.size(), results.size()); i++) {
            var file = results.get(i);
            inv.setItem(ENTRY.get(i), item(Material.BOOK, file.title(), "file:" + file.path() + ":0", List.of(file.path())));
        }
        chrome(player, inv, "search:" + query, false, false, true);
        player.openInventory(inv);
    }

    private void chrome(Player player, org.bukkit.inventory.Inventory inv, String route, boolean prev, boolean next, boolean search) {
        inv.setItem(DocBrowserLayout.MAIN_MENU_SLOT, item(Material.COMPASS, text(player, "docs.main-menu", Map.of()), "main-menu", List.of()));
        var parent = DocRoute.hasParent(route) ? "parent" : "";
        inv.setItem(DocBrowserLayout.PARENT_SLOT, item(Material.ARROW, text(player, "docs.parent", Map.of()), parent,
            parent.isBlank() ? List.of(text(player, "docs.parent.disabled", Map.of())) : List.of()));
        if (prev) inv.setItem(48, item(Material.ARROW, text(player, "docs.previous", Map.of()), page(route, -1), List.of()));
        if (next) inv.setItem(50, item(Material.ARROW, text(player, "docs.next", Map.of()), page(route, 1), List.of()));
        if (search) inv.setItem(DocBrowserLayout.SEARCH_SLOT, item(Material.SPYGLASS, text(player, "docs.search", Map.of()), "search", List.of("/docs search <query>")));
    }

    private String page(String route, int delta) {
        return DocRoute.page(route, delta, 9999);
    }

    private ItemStack pageItem(Player player, String route, String titleKey, String disabledKey, int delta, boolean enabled) {
        if (enabled) return item(Material.ARROW, text(player, titleKey, Map.of()), page(route, delta), List.of());
        return item(Material.GRAY_DYE, text(player, titleKey, Map.of()), "", List.of(text(player, disabledKey, Map.of())));
    }

    private ItemStack item(Material material, String title, String action, List<String> lore) {
        var item = new ItemStack(material);
        item.editMeta(meta -> {
            meta.setDisplayName(title);
            meta.setLore(lore.stream().limit(10).toList());
            if (!action.isBlank()) meta.getPersistentDataContainer().set(key, PersistentDataType.STRING, action);
        });
        return item;
    }

    private void external(Player player, String url) {
        player.sendMessage(Component.text(url).clickEvent(ClickEvent.openUrl(url)));
        player.sendMessage("Copy: " + url);
    }

    private DocBundle bundle() {
        if (bundle == null) bundle = DocBundle.load(getClass().getResourceAsStream("/lkjmc-docs-bundle.json"));
        return bundle;
    }

    private String text(Player player, String key, Map<String, String> values) {
        return renderer.render(player.locale().toLanguageTag(), key, values);
    }
}
