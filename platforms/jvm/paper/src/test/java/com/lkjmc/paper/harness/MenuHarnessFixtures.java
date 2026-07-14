package com.lkjmc.paper.harness;

import com.lkjmc.common.docs.DocBundle;
import com.lkjmc.common.i18n.MessageCatalog;
import com.lkjmc.common.menu.MenuBundle;
import com.lkjmc.common.menu.MenuRenderer;
import com.lkjmc.common.menu.MenuSnapshotView;
import com.lkjmc.common.menu.MenuTypes;
import java.util.EnumMap;

final class MenuHarnessFixtures {
    final MenuBundle bundle = MenuBundle.fromResource();
    final MenuRenderer renderer = new MenuRenderer(bundle,
            MessageCatalog.fromResources("en", "en", "ja"),
            DocBundle.load(MenuHarnessFixtures.class.getResourceAsStream("/lkjmc-docs-bundle.json")));

    MenuSnapshotView views(MenuTypes.Freshness freshness) {
        var entries = new EnumMap<MenuTypes.Domain, MenuSnapshotView.Entry>(MenuTypes.Domain.class);
        for (var domain : MenuTypes.Domain.values()) {
            if (domain != MenuTypes.Domain.LOCAL_DOCS)
                entries.put(domain, new MenuSnapshotView.Entry(freshness, freshness == MenuTypes.Freshness.UNAVAILABLE ? 0 : 7, null));
        }
        return new MenuSnapshotView(entries).withLocalDocs();
    }
}
