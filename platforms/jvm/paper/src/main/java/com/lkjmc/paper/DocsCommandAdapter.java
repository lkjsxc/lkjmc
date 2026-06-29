package com.lkjmc.paper;

import com.lkjmc.common.docs.DocBundle;
import com.lkjmc.common.docs.DocPaginator;
import com.lkjmc.common.docs.DocPath;
import com.lkjmc.common.i18n.MessageRenderer;
import java.util.ArrayDeque;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
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
    private final ConcurrentHashMap<UUID, ArrayDeque<String>> back = new ConcurrentHashMap<>();
    private DocBundle bundle;

    public DocsCommandAdapter(LkjmcPaperPlugin plugin, MessageRenderer renderer) {
        this.plugin = plugin;
        this.renderer = renderer;
        this.key = new NamespacedKey(plugin, "docs-action");
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (!(sender instanceof Player player)) {
            sender.sendMessage("players only");
            return true;
        }
        if (args.length > 1 && args[0].equalsIgnoreCase("search")) {
            open(player, "search:" + String.join(" ", java.util.Arrays.copyOfRange(args, 1, args.length)), true);
            return true;
        }
        open(player, args.length == 1 ? "file:" + args[0] + ":0" : "dir:", true);
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
        if (action.equals("home")) open(player, "dir:", true);
        else if (action.equals("back")) open(player, pop(player, route), false);
        else if (action.equals("search")) player.sendMessage(text(player, "docs.search.prompt", Map.of()));
        else if (action.startsWith("external:")) external(player, action.substring(9));
        else open(player, action, true);
    }

    private void open(Player player, String route, boolean push) {
        try {
            var docs = bundle();
            if (push) back.computeIfAbsent(player.getUniqueId(), id -> new ArrayDeque<>()).push(route);
            if (route.startsWith("file:")) file(player, docs, route);
            else if (route.startsWith("links:")) links(player, docs, route);
            else if (route.startsWith("search:")) search(player, docs, route.substring(7));
            else dir(player, docs, route.substring(4));
        } catch (RuntimeException error) {
            player.sendMessage(text(player, "docs.error", Map.of()));
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
        chrome(inv, "dir:" + path, false, false, true);
        player.openInventory(inv);
    }

    private void file(Player player, DocBundle docs, String route) {
        var parts = route.substring(5).split(":", 2);
        var file = docs.file(parts[0]).orElseThrow();
        var page = DocPaginator.page(file, parts.length == 2 ? Integer.parseInt(parts[1]) : 0, 38);
        var inv = Bukkit.createInventory(new DocsMenuHolder(route), 54, "Docs: " + file.title());
        inv.setItem(22, item(Material.WRITABLE_BOOK, file.path() + " " + (page.page() + 1) + "/" + page.pageCount(), "", page.lines()));
        chrome(inv, route, page.page() > 0, page.page() + 1 < page.pageCount(), true);
        inv.setItem(52, item(Material.OAK_SIGN, text(player, "docs.links", Map.of()), "links:" + file.path() + ":" + page.page(), List.of()));
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
        chrome(inv, route, false, false, true);
        player.openInventory(inv);
    }

    private void search(Player player, DocBundle docs, String query) {
        var inv = Bukkit.createInventory(new DocsMenuHolder("search:" + query), 54, "Docs search: " + query);
        var results = docs.search(query);
        for (var i = 0; i < Math.min(ENTRY.size(), results.size()); i++) {
            var file = results.get(i);
            inv.setItem(ENTRY.get(i), item(Material.BOOK, file.title(), "file:" + file.path() + ":0", List.of(file.path())));
        }
        chrome(inv, "search:" + query, false, false, true);
        player.openInventory(inv);
    }

    private void chrome(org.bukkit.inventory.Inventory inv, String route, boolean prev, boolean next, boolean search) {
        inv.setItem(45, item(Material.COMPASS, "Home", "home", List.of()));
        inv.setItem(49, item(Material.ARROW, "Back", "back", List.of()));
        if (prev) inv.setItem(48, item(Material.ARROW, "Previous", page(route, -1), List.of()));
        if (next) inv.setItem(50, item(Material.ARROW, "Next", page(route, 1), List.of()));
        if (search) inv.setItem(53, item(Material.SPYGLASS, "Search", "search", List.of("/docs search <query>")));
    }

    private String page(String route, int delta) {
        var idx = route.lastIndexOf(':');
        return route.substring(0, idx + 1) + (Integer.parseInt(route.substring(idx + 1)) + delta);
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

    private String pop(Player player, String current) {
        var stack = back.get(player.getUniqueId());
        if (stack == null || stack.size() < 2) return parent(current);
        stack.pop();
        return stack.peek();
    }

    private String parent(String route) {
        if (!route.startsWith("file:")) return "dir:";
        var path = route.substring(5).split(":", 2)[0];
        return "dir:" + (path.contains("/") ? path.substring(0, path.lastIndexOf('/')) : "");
    }

    private DocBundle bundle() {
        if (bundle == null) bundle = DocBundle.load(getClass().getResourceAsStream("/lkjmc-docs-bundle.json"));
        return bundle;
    }

    private String text(Player player, String key, Map<String, String> values) {
        return renderer.render(player.locale().toLanguageTag(), key, values);
    }
}
