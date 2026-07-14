package com.lkjmc.bindings;

import java.util.List;

public record MenuPayload(
        List<ShopItem> shop,
        List<KitItem> kits,
        List<VoteItem> votes,
        List<PluginItem> plugins
) implements DomainPayload {
    public MenuPayload {
        shop = List.copyOf(shop);
        kits = List.copyOf(kits);
        votes = List.copyOf(votes);
        plugins = List.copyOf(plugins);
    }
}
