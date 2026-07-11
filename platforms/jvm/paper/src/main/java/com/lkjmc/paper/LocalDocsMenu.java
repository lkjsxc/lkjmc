package com.lkjmc.paper;

import com.lkjmc.common.docs.DocBundle;
import com.lkjmc.common.docs.DocPaginator;
import java.util.List;
import org.bukkit.Bukkit;
import org.bukkit.Material;
import org.bukkit.NamespacedKey;
import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.inventory.InventoryClickEvent;
import org.bukkit.inventory.Inventory;
import org.bukkit.inventory.InventoryHolder;
import org.bukkit.inventory.ItemStack;
import org.bukkit.persistence.PersistentDataType;

public final class LocalDocsMenu implements Listener {
    private static final int PREVIOUS = 46;
    private static final int NEXT = 48;
    private static final int ROOT = 49;
    private static final int CLOSE = 53;
    private final DocBundle docs;
    private final NamespacedKey action;

    LocalDocsMenu(LkjmcPaperPlugin plugin) {
        this.docs = DocBundle.load(LocalDocsMenu.class.getResourceAsStream("/lkjmc-docs-bundle.json"));
        this.action = new NamespacedKey(plugin, "local_docs_action");
    }

    public void openRoot(Player player) {
        openList(player, docs.files(), "lkjmc documentation");
    }

    public void openSearch(Player player, String query) {
        openList(player, docs.search(query), "documentation search");
    }

    public void openPath(Player player, String path) {
        docs.file(path).ifPresentOrElse(file -> openFile(player, file.path(), 0), () -> openSearch(player, path));
    }

    @EventHandler
    public void onClick(InventoryClickEvent event) {
        if (!(event.getWhoClicked() instanceof Player player)
            || !(event.getView().getTopInventory().getHolder() instanceof View view)) return;
        event.setCancelled(true);
        var item = event.getCurrentItem();
        if (item == null || !item.hasItemMeta()) return;
        var value = item.getItemMeta().getPersistentDataContainer().get(action, PersistentDataType.STRING);
        if (value == null) return;
        if (value.equals("root")) openRoot(player);
        else if (value.equals("close")) player.closeInventory();
        else if (value.equals("previous")) openFile(player, view.path(), view.page() - 1);
        else if (value.equals("next")) openFile(player, view.path(), view.page() + 1);
        else if (value.startsWith("file:")) openFile(player, value.substring(5), 0);
    }

    private void openList(Player player, List<com.lkjmc.common.docs.DocFile> files, String title) {
        var inventory = Bukkit.createInventory(new View("", 0), 54, title);
        for (int index = 0; index < Math.min(45, files.size()); index++) {
            var file = files.get(index);
            inventory.setItem(index, item(Material.BOOK, file.title(), "file:" + file.path()));
        }
        inventory.setItem(CLOSE, item(Material.BARRIER, "Close", "close"));
        player.openInventory(inventory);
    }

    private void openFile(Player player, String path, int page) {
        var file = docs.file(path);
        if (file.isEmpty()) {
            openRoot(player);
            return;
        }
        var content = DocPaginator.page(file.get(), page, 36);
        var inventory = Bukkit.createInventory(new View(path, content.page()), 54, file.get().title());
        for (int index = 0; index < content.lines().size(); index++) {
            inventory.setItem(19 + index, item(Material.PAPER, content.lines().get(index), ""));
        }
        if (content.page() > 0) inventory.setItem(PREVIOUS, item(Material.ARROW, "Previous", "previous"));
        if (content.page() + 1 < content.pageCount()) inventory.setItem(NEXT, item(Material.ARROW, "Next", "next"));
        inventory.setItem(ROOT, item(Material.BOOK, "Documentation", "root"));
        inventory.setItem(CLOSE, item(Material.BARRIER, "Close", "close"));
        player.openInventory(inventory);
    }

    private ItemStack item(Material material, String name, String value) {
        var item = new ItemStack(material);
        var meta = item.getItemMeta();
        meta.setDisplayName(name == null || name.isBlank() ? " " : name);
        if (!value.isBlank()) meta.getPersistentDataContainer().set(action, PersistentDataType.STRING, value);
        item.setItemMeta(meta);
        return item;
    }

    private record View(String path, int page) implements InventoryHolder {
        @Override public Inventory getInventory() { return null; }
    }
}
