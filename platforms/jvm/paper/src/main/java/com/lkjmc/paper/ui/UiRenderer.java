package com.lkjmc.paper.ui;

import com.lkjmc.common.ui.document.MenuDocumentSet;
import com.lkjmc.common.ui.kernel.FrameSlot;
import com.lkjmc.common.ui.kernel.UiFrame;
import com.lkjmc.common.ui.kernel.UiModel;
import com.lkjmc.common.ui.kernel.UiView;
import org.bukkit.Bukkit;
import org.bukkit.Material;
import org.bukkit.entity.Player;
import org.bukkit.inventory.Inventory;
import org.bukkit.inventory.ItemStack;

public class UiRenderer implements UiSessionService.Renderer {
    private final MenuDocumentSet documents;
    private final UiMetadataCodec metadata;
    private final UiText text;

    public UiRenderer(MenuDocumentSet documents, UiMetadataCodec metadata, UiText text) {
        this.documents = documents;
        this.metadata = metadata;
        this.text = text;
    }

    @Override
    public void render(Player player, String locale, UiModel model) {
        apply(player, locale, UiView.frame(documents, model), model);
    }

    void apply(Player player, String locale, UiFrame frame, UiModel model) {
        var current = current(player);
        if (sameSessionAndSize(current, model, frame)) {
            fill(current, locale, frame);
            return;
        }
        var holder = new UiInventoryHolder(model.sessionId(), model.route(), frame.size());
        var inventory = Bukkit.createInventory(holder, frame.size(), text.title(locale, frame.title()));
        holder.attach(inventory);
        fill(inventory, locale, frame);
        player.openInventory(inventory);
    }

    UiRenderedItem mapped(String locale, FrameSlot slot) {
        return new UiRenderedItem(material(slot.material()), text.itemName(locale, slot.name(), slot.role()),
            text.lore(locale, slot.lore(), slot.role()), slot.metadata(), slot.inert());
    }

    private ItemStack item(String locale, FrameSlot slot) {
        var mapped = mapped(locale, slot);
        var item = new ItemStack(mapped.material());
        var meta = item.getItemMeta();
        if (meta != null) {
            meta.displayName(mapped.name());
            meta.lore(mapped.lore());
            if (!mapped.inert() && mapped.metadata() != null) {
                metadata.write(meta, mapped.metadata());
            }
            item.setItemMeta(meta);
        }
        return item;
    }

    private void fill(Inventory inventory, String locale, UiFrame frame) {
        for (var slot = 0; slot < frame.size(); slot++) {
            inventory.setItem(slot, null);
        }
        for (var slot : frame.slots()) {
            if (slot.slot() < frame.size()) {
                inventory.setItem(slot.slot(), item(locale, slot));
            }
        }
    }

    private Inventory current(Player player) {
        var view = player.getOpenInventory();
        if (view == null) {
            return null;
        }
        var top = view.getTopInventory();
        return top != null && top.getHolder() instanceof UiInventoryHolder ? top : null;
    }

    private boolean sameSessionAndSize(Inventory current, UiModel model, UiFrame frame) {
        if (current == null || !(current.getHolder() instanceof UiInventoryHolder holder)) {
            return false;
        }
        return holder.sessionId().equals(model.sessionId())
            && holder.route().equals(model.route())
            && current.getSize() == frame.size();
    }

    private static Material material(String name) {
        var material = Material.matchMaterial(name == null ? "" : name);
        return material == null ? Material.STONE : material;
    }
}
