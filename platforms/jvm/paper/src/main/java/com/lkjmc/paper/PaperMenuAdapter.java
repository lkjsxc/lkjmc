package com.lkjmc.paper;

import com.lkjmc.common.docs.DocBundle;
import com.lkjmc.common.i18n.MessageCatalog;
import com.lkjmc.common.menu.MenuBundle;
import com.lkjmc.common.menu.MenuFrame;
import com.lkjmc.common.menu.MenuRenderer;
import com.lkjmc.common.menu.MenuResult;
import com.lkjmc.common.menu.MenuSnapshotView;
import com.lkjmc.common.menu.MenuTypes;
import com.lkjmc.common.runtime.JvmPluginRuntime;
import com.lkjmc.common.sync.SyncKey;
import java.time.Duration;
import java.time.Instant;
import java.util.EnumMap;
import java.util.List;
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
import org.bukkit.inventory.Inventory;
import org.bukkit.inventory.InventoryHolder;
import org.bukkit.inventory.ItemStack;
import org.bukkit.persistence.PersistentDataType;

public final class PaperMenuAdapter implements Listener {
    private final LkjmcPaperPlugin plugin;
    private final JvmPluginRuntime runtime;
    private final DocBundle docs;
    private final MenuBundle bundle;
    private final MenuRenderer renderer;
    private final NamespacedKey metadataKey;
    private final AtomicLong sessions = new AtomicLong();
    private final Map<UUID, PaperMenuProtocolAdapter> players = new ConcurrentHashMap<>();
    private final Set<UUID> replacing = ConcurrentHashMap.newKeySet();

    PaperMenuAdapter(LkjmcPaperPlugin plugin, JvmPluginRuntime runtime) {
        this.plugin = plugin; this.runtime = runtime;
        docs = DocBundle.load(PaperMenuAdapter.class.getResourceAsStream("/lkjmc-docs-bundle.json"));
        bundle = MenuBundle.fromResource();
        var messages = MessageCatalog.fromResources("en", "en", "ja");
        renderer = new MenuRenderer(bundle, messages, docs);
        metadataKey = new NamespacedKey(plugin, "menu_metadata");
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
        var adapter = players.get(player.getUniqueId());
        if (adapter == null) return;
        apply(player, adapter, adapter.click(slot.metadata(), slot.action(), false));
    }

    @EventHandler
    public void onClose(InventoryCloseEvent event) {
        UUID player = event.getPlayer().getUniqueId();
        if (!replacing.contains(player)) players.remove(player);
    }

    private void open(Player player, String route, Map<String, String> params) {
        subscribe(player);
        var adapter = new PaperMenuProtocolAdapter(bundle, renderer);
        players.put(player.getUniqueId(), adapter);
        apply(player, adapter, adapter.open(sessions.incrementAndGet(), route, params,
                locale(player), snapshots(player)));
    }

    private void apply(Player player, PaperMenuProtocolAdapter adapter, MenuResult result) {
        switch (result) {
            case MenuResult.Rendered value -> render(player, value.frame());
            case MenuResult.Closed ignored -> player.closeInventory();
            case MenuResult.Failed value -> player.sendMessage(value.message());
            case MenuResult.Pending value -> new PaperSchedulerBridge(plugin).mainOrGlobal(() ->
                    apply(player, adapter, adapter.response(value.request(), snapshots(player))));
            case MenuResult.Ignored ignored -> { }
        }
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

    private void subscribe(Player player) {
        String id = player.getUniqueId().toString();
        runtime.subscribe(List.of(new SyncKey("menus", "global"), new SyncKey("permissions", id),
                new SyncKey("claims", id), new SyncKey("settings", id), new SyncKey("profiles", id),
                new SyncKey("routing", "network"), new SyncKey("presence", "global")));
    }

    private MenuSnapshotView snapshots(Player player) {
        var values = new EnumMap<MenuTypes.Domain, MenuSnapshotView.Entry>(MenuTypes.Domain.class);
        runtime.coordinator().ifPresent(coordinator -> {
            String id = player.getUniqueId().toString();
            for (var key : List.of(new SyncKey("menus", "global"), new SyncKey("permissions", id),
                    new SyncKey("claims", id), new SyncKey("settings", id), new SyncKey("profiles", id),
                    new SyncKey("routing", "network"), new SyncKey("presence", "global"))) {
                coordinator.view(key).ifPresent(value -> {
                    var freshness = Duration.between(value.receivedAt(), Instant.now()).compareTo(Duration.ofSeconds(30)) > 0
                            ? MenuTypes.Freshness.STALE : MenuTypes.Freshness.CURRENT;
                    values.put(MenuTypes.Domain.valueOf(key.domain().toUpperCase(java.util.Locale.ROOT)),
                            new MenuSnapshotView.Entry(freshness, value.revision(), value.value()));
                });
            }
        });
        return new MenuSnapshotView(values).withLocalDocs();
    }

    private String locale(Player player) { return player.locale().toLanguageTag(); }
    private record Holder(MenuFrame frame) implements InventoryHolder {
        @Override public Inventory getInventory() { return null; }
    }
}
