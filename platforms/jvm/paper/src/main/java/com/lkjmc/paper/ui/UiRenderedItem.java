package com.lkjmc.paper.ui;

import com.lkjmc.common.ui.kernel.MenuMetadata;
import java.util.List;
import net.kyori.adventure.text.Component;
import org.bukkit.Material;

record UiRenderedItem(
    Material material,
    Component name,
    List<Component> lore,
    MenuMetadata metadata,
    boolean inert
) {
    UiRenderedItem {
        lore = List.copyOf(lore == null ? List.of() : lore);
    }
}
