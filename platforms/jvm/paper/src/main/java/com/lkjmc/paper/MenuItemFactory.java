package com.lkjmc.paper;

import com.lkjmc.common.i18n.MessageCatalog;
import com.lkjmc.common.menu.ItemSpec;
import com.lkjmc.common.menu.MenuAction;
import com.lkjmc.common.menu.MenuMetadata;
import com.lkjmc.common.menu.MenuState;
import org.bukkit.Material;
import org.bukkit.inventory.ItemStack;

final class MenuItemFactory {
    private final MessageCatalog catalog;
    private final MenuMetadataCodec codec;

    MenuItemFactory(MessageCatalog catalog, MenuMetadataCodec codec) {
        this.catalog = catalog;
        this.codec = codec;
    }

    ItemStack item(String locale, MenuState state, int slot, ItemSpec spec, MenuAction action) {
        var material = Material.matchMaterial(spec.material());
        var item = new ItemStack(material == null ? Material.STONE : material);
        var meta = item.getItemMeta();
        meta.setDisplayName(render(locale, spec.nameKey()));
        meta.setLore(spec.loreKeys().stream().map(key -> render(locale, key)).toList());
        codec.write(meta, MenuMetadata.of(state.current(), state.route(), slot, action,
            state.sessionId(), state.renderEpoch(), spec.inert()));
        item.setItemMeta(meta);
        return item;
    }

    private String render(String locale, String key) {
        return key.startsWith("literal:") ? key.substring("literal:".length()) : catalog.render(locale, key);
    }
}
