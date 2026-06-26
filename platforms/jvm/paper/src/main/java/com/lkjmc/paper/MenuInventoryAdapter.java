package com.lkjmc.paper;

import com.lkjmc.common.i18n.LocaleResolver;
import com.lkjmc.common.i18n.MessageCatalog;
import com.lkjmc.common.menu.ItemSpec;
import com.lkjmc.common.menu.MenuClick;
import com.lkjmc.common.menu.MenuEffect;
import com.lkjmc.common.menu.MenuId;
import com.lkjmc.common.menu.MenuReducer;
import com.lkjmc.common.menu.MenuRegistry;
import com.lkjmc.common.menu.MenuSpec;
import com.lkjmc.common.menu.MenuState;
import com.lkjmc.common.menu.MenuAction;
import com.lkjmc.common.menu.StandardMenus;
import java.util.HashMap;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import org.bukkit.Bukkit;
import org.bukkit.Material;
import org.bukkit.NamespacedKey;
import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.inventory.InventoryClickEvent;
import org.bukkit.event.inventory.InventoryCloseEvent;
import org.bukkit.event.inventory.InventoryDragEvent;
import org.bukkit.inventory.ItemStack;
import org.bukkit.persistence.PersistentDataType;

public final class MenuInventoryAdapter implements Listener {
    private final LkjmcPaperPlugin plugin;
    private final MessageCatalog catalog;
    private final LocaleResolver resolver;
    private final MenuRegistry registry = StandardMenus.registry();
    private final Map<UUID, MenuState> sessions = new HashMap<>();
    private final NamespacedKey menuKey;
    private final NamespacedKey slotKey;
    private final NamespacedKey actionKey;
    private final NamespacedKey inertKey;

    public MenuInventoryAdapter(LkjmcPaperPlugin plugin, MessageCatalog catalog, LocaleResolver resolver) {
        this.plugin = plugin;
        this.catalog = catalog;
        this.resolver = resolver;
        this.menuKey = new NamespacedKey(plugin, "menu_id");
        this.slotKey = new NamespacedKey(plugin, "menu_slot");
        this.actionKey = new NamespacedKey(plugin, "menu_action");
        this.inertKey = new NamespacedKey(plugin, "menu_inert");
    }

    public void openRoot(Player player) {
        open(player, new MenuId("root"));
    }

    public void open(Player player, MenuId id) {
        var spec = registry.find(id).orElse(StandardMenus.root());
        open(player, spec);
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
        var spec = registry.require(holder.get().menuId());
        var item = event.getCurrentItem();
        var key = action(item);
        if (key == null && item != null && spec.slots().stream().anyMatch(slot -> slot.slot() == event.getRawSlot())) {
            key = "unknown";
        }
        var decision = MenuReducer.click(spec, state(player, spec), new MenuClick(event.getRawSlot(), key, true));
        decision.effects().forEach(effect -> execute(player, effect));
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
        if (event.getInventory().getHolder() instanceof MenuInventoryHolder) {
            sessions.remove(event.getPlayer().getUniqueId());
        }
    }

    private Optional<MenuInventoryHolder> holder(InventoryClickEvent event) {
        var top = event.getView().getTopInventory();
        if (event.getRawSlot() < 0 || event.getRawSlot() >= top.getSize()) {
            return Optional.empty();
        }
        if (top.getHolder() instanceof MenuInventoryHolder holder) {
            return Optional.of(holder);
        }
        return Optional.empty();
    }

    private MenuState state(Player player, MenuSpec spec) {
        return sessions.getOrDefault(player.getUniqueId(), new MenuState(spec.id(), 0));
    }

    private void open(Player player, MenuSpec spec) {
        var locale = locale(player);
        var sessionId = UUID.randomUUID();
        var holder = new MenuInventoryHolder(spec.id(), sessionId);
        var title = catalog.render(locale, spec.title().key());
        var inventory = Bukkit.createInventory(holder, spec.size().slots(), title);
        holder.attach(inventory);
        for (var slot : spec.slots()) {
            inventory.setItem(slot.slot(), item(locale, spec.id(), slot.slot(), slot.item(), slot.action()));
        }
        sessions.put(player.getUniqueId(), new MenuState(spec.id(), 0));
        player.openInventory(inventory);
    }

    private ItemStack item(String locale, MenuId menuId, int slot, ItemSpec spec, MenuAction action) {
        var material = Material.matchMaterial(spec.material());
        var item = new ItemStack(material == null ? Material.STONE : material);
        var meta = item.getItemMeta();
        meta.setDisplayName(catalog.render(locale, spec.nameKey()));
        meta.setLore(spec.loreKeys().stream().map(key -> catalog.render(locale, key)).toList());
        var pdc = meta.getPersistentDataContainer();
        pdc.set(menuKey, PersistentDataType.STRING, menuId.value());
        pdc.set(slotKey, PersistentDataType.INTEGER, slot);
        pdc.set(actionKey, PersistentDataType.STRING, MenuAction.key(action));
        if (spec.inert()) {
            pdc.set(inertKey, PersistentDataType.BYTE, (byte) 1);
        }
        item.setItemMeta(meta);
        return item;
    }

    private String action(ItemStack item) {
        if (item == null || !item.hasItemMeta()) {
            return null;
        }
        return item.getItemMeta().getPersistentDataContainer().get(actionKey, PersistentDataType.STRING);
    }

    private void execute(Player player, MenuEffect effect) {
        switch (effect) {
            case MenuEffect.OpenMenu open -> open(player, open.menuId());
            case MenuEffect.CloseMenu ignored -> player.closeInventory();
            case MenuEffect.Refresh ignored -> open(player, state(player, StandardMenus.root()).current());
            case MenuEffect.RunCommand command -> { player.closeInventory(); player.performCommand(command.command()); }
            case MenuEffect.SendMessage message -> player.sendMessage(catalog.render(locale(player), message.key()));
        }
    }

    private String locale(Player player) {
        return resolver.resolve(Optional.of(player.locale().toLanguageTag()));
    }
}
