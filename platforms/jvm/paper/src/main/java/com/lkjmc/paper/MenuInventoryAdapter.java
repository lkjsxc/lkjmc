package com.lkjmc.paper;

import com.lkjmc.common.i18n.LocaleResolver;
import com.lkjmc.common.i18n.MessageCatalog;
import com.lkjmc.common.menu.MenuClick;
import com.lkjmc.common.menu.DynamicMenus;
import com.lkjmc.common.menu.MenuId;
import com.lkjmc.common.menu.MenuReducer;
import com.lkjmc.common.menu.MenuRegistry;
import com.lkjmc.common.menu.MenuRoute;
import com.lkjmc.common.menu.MenuState;
import com.lkjmc.common.menu.StandardMenus;
import java.util.Optional;
import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.inventory.InventoryClickEvent;
import org.bukkit.event.inventory.InventoryCloseEvent;
import org.bukkit.event.inventory.InventoryDragEvent;

public final class MenuInventoryAdapter implements Listener {
    private final LkjmcPaperPlugin plugin;
    private final MessageCatalog catalog;
    private final LocaleResolver resolver;
    private final MenuRegistry registry = StandardMenus.registry();
    private final MenuSessionStore sessions = new MenuSessionStore();
    private final MenuMetadataCodec metadata;
    private final MenuInventoryRenderer renderer;
    private final MenuEffectExecutor effects;
    private final MenuDataGateway data;

    public MenuInventoryAdapter(LkjmcPaperPlugin plugin, MessageCatalog catalog, LocaleResolver resolver,
                                InventorySyncService sync) {
        this.plugin = plugin;
        this.catalog = catalog;
        this.resolver = resolver;
        this.metadata = new MenuMetadataCodec(plugin);
        this.renderer = new MenuInventoryRenderer(catalog, new MenuItemFactory(catalog, metadata));
        this.effects = new MenuEffectExecutor(plugin, catalog, resolver, plugin.daemon(), this, sync);
        this.data = new MenuDataGateway(plugin.daemon());
    }

    public void openRoot(Player player) {
        render(player, registry.require(new MenuId("root")), sessions.openRoot(player));
    }

    public void open(Player player, MenuId id) {
        open(player, new MenuRoute(id));
    }

    public void open(Player player, MenuRoute route) {
        var spec = registry.find(route.id()).orElse(StandardMenus.root());
        render(player, spec, sessions.openRoute(player, new MenuRoute(spec.id(), route.params())));
    }

    void openPrevious(Player player) {
        var state = sessions.back(player);
        render(player, registry.find(state.current()).orElse(StandardMenus.root()), state);
    }

    void refresh(Player player) {
        var state = sessions.refresh(player);
        render(player, registry.find(state.current()).orElse(StandardMenus.root()), state);
    }

    @EventHandler
    public void onClick(InventoryClickEvent event) {
        var holder = holder(event);
        if (holder.isEmpty()) {
            return;
        }
        event.setCancelled(true);
        if (!(event.getWhoClicked() instanceof Player player)) {
            return;
        }
        var item = event.getCurrentItem();
        var decoded = metadata.read(item);
        var action = decoded == null && metadata.hasAny(item) ? "malformed" : null;
        var state = sessions.state(player).orElse(holder.get().state());
        var spec = registry.require(holder.get().menuId());
        var decision = MenuReducer.click(spec, state, new MenuClick(event.getRawSlot(), decoded, action, true));
        decision.effects().forEach(effect -> effects.execute(player, effect));
    }

    @EventHandler
    public void onDrag(InventoryDragEvent event) {
        if (event.getView().getTopInventory().getHolder() instanceof MenuInventoryHolder) {
            var topSize = event.getView().getTopInventory().getSize();
            if (event.getRawSlots().stream().anyMatch(slot -> slot < topSize)) {
                event.setCancelled(true);
            }
        }
    }

    @EventHandler
    public void onClose(InventoryCloseEvent event) {
        if (event.getInventory().getHolder() instanceof MenuInventoryHolder holder
            && event.getPlayer() instanceof Player player) {
            sessions.clearIfSession(player, holder.state().sessionId());
        }
    }

    private Optional<MenuInventoryHolder> holder(InventoryClickEvent event) {
        var top = event.getView().getTopInventory();
        if (event.getRawSlot() < 0 || event.getRawSlot() >= top.getSize()) {
            return Optional.empty();
        }
        return top.getHolder() instanceof MenuInventoryHolder holder ? Optional.of(holder) : Optional.empty();
    }

    private void render(Player player, com.lkjmc.common.menu.MenuSpec spec, MenuState state) {
        player.openInventory(renderer.render(locale(player), spec, state));
        if (spec.id().value().equals("server-list")) {
            loadServers(player, state);
        }
    }

    private void loadServers(Player player, MenuState state) {
        data.servers(player).whenComplete((servers, error) -> {
            if (error != null) {
                return;
            }
            plugin.scheduler().runPlayer(player, () -> sessions.state(player)
                .filter(current -> current.sessionId().equals(state.sessionId()))
                .ifPresent(current -> {
                    var refreshed = sessions.refresh(player);
                    player.openInventory(renderer.render(locale(player), DynamicMenus.serverList(servers), refreshed));
                }));
        });
    }

    private String locale(Player player) {
        return resolver.resolve(Optional.of(player.locale().toLanguageTag()));
    }
}
