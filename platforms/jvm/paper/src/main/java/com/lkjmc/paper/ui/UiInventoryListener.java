package com.lkjmc.paper.ui;

import com.lkjmc.common.ui.kernel.UiMsg;
import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.Listener;
import org.bukkit.event.inventory.InventoryClickEvent;
import org.bukkit.event.inventory.InventoryCloseEvent;
import org.bukkit.event.inventory.InventoryDragEvent;

public final class UiInventoryListener implements Listener {
    @FunctionalInterface
    public interface TokenGuard {
        boolean protect(InventoryClickEvent event);
        static TokenGuard none() { return event -> false; }
    }

    private final UiSessionService sessions;
    private final UiMetadataCodec metadata;
    private final TokenGuard tokenGuard;

    public UiInventoryListener(UiSessionService sessions, UiMetadataCodec metadata) {
        this(sessions, metadata, TokenGuard.none());
    }

    public UiInventoryListener(UiSessionService sessions, UiMetadataCodec metadata, TokenGuard tokenGuard) {
        this.sessions = sessions;
        this.metadata = metadata;
        this.tokenGuard = tokenGuard == null ? TokenGuard.none() : tokenGuard;
    }

    @EventHandler
    public void onClick(InventoryClickEvent event) {
        var top = event.getView().getTopInventory();
        if (!(top.getHolder() instanceof UiInventoryHolder)) {
            return;
        }
        if (!(event.getWhoClicked() instanceof Player player)) {
            return;
        }
        if (event.getRawSlot() < 0 || event.getRawSlot() >= top.getSize()) {
            if (tokenGuard.protect(event)) {
                event.setCancelled(true);
            }
            return;
        }
        event.setCancelled(true);
        var item = event.getCurrentItem();
        var decoded = metadata.read(item);
        var malformed = decoded == null && metadata.hasAny(item);
        sessions.dispatch(player, new UiMsg.Clicked(event.getRawSlot(), decoded, malformed));
    }

    @EventHandler
    public void onDrag(InventoryDragEvent event) {
        var top = event.getView().getTopInventory();
        if (top.getHolder() instanceof UiInventoryHolder) {
            var size = top.getSize();
            if (event.getRawSlots().stream().anyMatch(slot -> slot >= 0 && slot < size)) {
                event.setCancelled(true);
            }
        }
    }

    @EventHandler
    public void onClose(InventoryCloseEvent event) {
        if (event.getInventory().getHolder() instanceof UiInventoryHolder holder
            && event.getPlayer() instanceof Player player) {
            sessions.close(player, holder.sessionId());
        }
    }
}
