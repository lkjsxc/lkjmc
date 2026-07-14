package com.lkjmc.paper;

import com.lkjmc.common.docs.DocBundle;
import com.lkjmc.common.i18n.MessageCatalog;
import com.lkjmc.common.menu.MenuBundle;
import com.lkjmc.common.menu.MenuFrame;
import com.lkjmc.common.menu.MenuRenderer;
import com.lkjmc.common.menu.MenuResult;
import com.lkjmc.common.menu.MenuTypes;
import com.lkjmc.common.runtime.JvmPluginRuntime;
import java.util.Map;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicLong;
import org.bukkit.Bukkit;
import org.bukkit.Material;
import org.bukkit.NamespacedKey;
import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.inventory.InventoryClickEvent;
import org.bukkit.event.inventory.InventoryCloseEvent;
import org.bukkit.event.player.PlayerLocaleChangeEvent;
import org.bukkit.event.player.PlayerQuitEvent;
import org.bukkit.inventory.Inventory;
import org.bukkit.inventory.InventoryHolder;
import org.bukkit.inventory.ItemStack;
import org.bukkit.persistence.PersistentDataType;

public final class PaperMenuAdapter implements Listener {
    private final LkjmcPaperPlugin plugin;
    private final PaperMenuSnapshots snapshots;
    private final DocBundle docs;
    private final MenuBundle bundle;
    private final MenuRenderer renderer;
    private final NamespacedKey metadataKey;
    private final AtomicLong sessions = new AtomicLong();
    private final Set<UUID> replacing = ConcurrentHashMap.newKeySet();
    private final MenuResponseOwnership<PaperMenuProtocolAdapter> ownership;

    PaperMenuAdapter(LkjmcPaperPlugin plugin, JvmPluginRuntime runtime) {
        this.plugin = plugin; snapshots = new PaperMenuSnapshots(runtime);
        docs = DocBundle.load(PaperMenuAdapter.class.getResourceAsStream("/lkjmc-docs-bundle.json"));
        bundle = MenuBundle.fromResource();
        var messages = MessageCatalog.fromResources("en", "en", "ja");
        renderer = new MenuRenderer(bundle, messages, docs);
        metadataKey = new NamespacedKey(plugin, "menu_metadata");
        ownership = new MenuResponseOwnership<>(new PaperSchedulerBridge(plugin));
    }

    public void openRoot(Player player) { open(player, "root", Map.of()); }
    public void openSearch(Player player, String query) {
        open(player, "docs-search", Map.of("query", query));
    }
    public void openPath(Player player, String path) {
        if (docs.file(path).isPresent()) open(player, "docs-file", Map.of("path", path, "page", "0"));
        else openSearch(player, path);
    }
    public void openDocs(Player player) { open(player, "docs-directory", Map.of("path", "docs")); }

    @EventHandler
    public void onClick(InventoryClickEvent event) {
        if (!(event.getWhoClicked() instanceof Player player)
                || !(event.getView().getTopInventory().getHolder() instanceof Holder holder)) return;
        event.setCancelled(true);
        if (event.getRawSlot() < 0 || event.getRawSlot() >= holder.frame().size()) return;
        var slot = holder.frame().bySlot().get(event.getRawSlot());
        if (slot == null || event.getCurrentItem() == null || !encoded(slot.metadata()).equals(
                event.getCurrentItem().getItemMeta().getPersistentDataContainer()
                        .get(metadataKey, PersistentDataType.STRING))) {
            player.sendMessage(renderer.failure(locale(player), MenuTypes.Failure.UNKNOWN_ACTION)); return;
        }
        var adapter = ownership.active(player.getUniqueId()).orElse(null);
        if (adapter == null || !sameFrame(adapter, holder.frame())) return;
        apply(player, adapter, adapter.click(slot.metadata(), slot.action(), false));
    }

    @EventHandler
    public void onClose(InventoryCloseEvent event) {
        UUID playerId = event.getPlayer().getUniqueId();
        if (replacing.contains(playerId) || !(event.getInventory().getHolder() instanceof Holder holder)) return;
        ownership.active(playerId).filter(adapter -> sameFrame(adapter, holder.frame()))
                .ifPresent(ignored -> ownership.invalidate(playerId));
    }

    @EventHandler public void onQuit(PlayerQuitEvent event) { ownership.invalidate(event.getPlayer().getUniqueId()); }

    @EventHandler
    public void onLocale(PlayerLocaleChangeEvent event) {
        UUID playerId = event.getPlayer().getUniqueId();
        if (ownership.active(playerId).isEmpty()) return;
        ownership.invalidate(playerId);
        if (event.getPlayer().getOpenInventory().getTopInventory().getHolder() instanceof Holder)
            event.getPlayer().closeInventory();
    }

    public void disable() { ownership.disable(); }

    private void open(Player player, String route, Map<String, String> params) {
        snapshots.subscribe(player.getUniqueId());
        var adapter = new PaperMenuProtocolAdapter(bundle, renderer);
        var result = adapter.open(sessions.incrementAndGet(), route, params, locale(player), snapshots.view(player.getUniqueId()));
        if (result instanceof MenuResult.Rendered value) {
            var frame = value.frame();
            ownership.open(player.getUniqueId(), adapter, frame.session(), locale(player),
                    frame.route(), frame.request());
            render(player, frame);
        } else if (result instanceof MenuResult.Failed value) player.sendMessage(value.message());
    }

    private void apply(Player player, PaperMenuProtocolAdapter adapter, MenuResult result) {
        switch (result) {
            case MenuResult.Rendered value -> {
                capture(player, adapter, value.frame()); render(player, value.frame());
            }
            case MenuResult.Closed ignored -> {
                ownership.invalidate(player.getUniqueId()); player.closeInventory();
            }
            case MenuResult.Failed value -> player.sendMessage(value.message());
            case MenuResult.Pending value -> pending(player, adapter, value.request());
            case MenuResult.Ignored ignored -> { }
        }
    }

    private void pending(Player player, PaperMenuProtocolAdapter adapter, long request) {
        var frame = adapter.frame();
        var token = ownership.advance(player.getUniqueId(), adapter, frame.session(), locale(player),
                frame.route(), request);
        ownership.onEntity(token, owned -> complete(token, owned)).exceptionally(failure -> {
            ownership.invalidate(token); return null;
        });
    }

    private void complete(MenuResponseOwnership.Token token, PaperMenuProtocolAdapter adapter) {
        var player = Bukkit.getPlayer(token.playerId());
        if (player == null || !player.isOnline() || !locale(player).equals(token.locale())
                || !sameToken(adapter, token)) {
            ownership.invalidate(token.playerId()); return;
        }
        var result = adapter.response(token.request(), snapshots.view(player.getUniqueId()));
        if (ownership.current(token).filter(value -> value == adapter).isEmpty()) return;
        apply(player, adapter, result);
    }

    private void capture(Player player, PaperMenuProtocolAdapter adapter, MenuFrame frame) {
        ownership.advance(player.getUniqueId(), adapter, frame.session(), locale(player),
                frame.route(), frame.request());
    }

    private boolean sameToken(PaperMenuProtocolAdapter adapter, MenuResponseOwnership.Token token) {
        var frame = adapter.frame();
        return frame.session() == token.session() && frame.request() == token.request()
                && frame.route().equals(token.route());
    }

    private boolean sameFrame(PaperMenuProtocolAdapter adapter, MenuFrame expected) {
        var current = adapter.frame();
        return current.session() == expected.session() && current.request() == expected.request()
                && current.route().equals(expected.route())
                && current.renderRevision() == expected.renderRevision();
    }

    private void render(Player player, MenuFrame frame) {
        var inventory = Bukkit.createInventory(new Holder(frame), frame.size(), frame.title());
        for (var slot : frame.slots()) inventory.setItem(slot.index(), item(slot));
        replacing.add(player.getUniqueId());
        try { player.openInventory(inventory); }
        finally { replacing.remove(player.getUniqueId()); }
    }

    private ItemStack item(MenuFrame.Slot slot) {
        Material material = Material.matchMaterial(slot.material());
        var item = new ItemStack(material == null ? Material.BARRIER : material);
        var meta = item.getItemMeta(); meta.setDisplayName(slot.name());
        if (!slot.lore().isEmpty()) meta.setLore(slot.lore());
        meta.getPersistentDataContainer().set(metadataKey, PersistentDataType.STRING, encoded(slot.metadata()));
        item.setItemMeta(meta); return item;
    }

    private String encoded(MenuFrame.Metadata value) {
        return String.join("|", value.route(), Long.toString(value.session()), Long.toString(value.request()),
                Long.toString(value.renderRevision()), Integer.toString(value.slot()), value.action().name());
    }

    private String locale(Player player) { return player.locale().toLanguageTag(); }
    private record Holder(MenuFrame frame) implements InventoryHolder {
        @Override public Inventory getInventory() { return null; }
    }
}
