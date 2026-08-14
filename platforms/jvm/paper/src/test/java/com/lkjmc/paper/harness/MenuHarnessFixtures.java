package com.lkjmc.paper.harness;

import com.lkjmc.common.docs.DocBundle;
import com.lkjmc.common.i18n.MessageCatalog;
import com.lkjmc.common.menu.MenuBundle;
import com.lkjmc.common.menu.MenuRenderer;

final class MenuHarnessFixtures {
    final MenuBundle bundle = MenuBundle.fromResource();
    final MenuRenderer renderer = new MenuRenderer(
            MessageCatalog.fromResources("en", "en", "ja"),
            DocBundle.load(MenuHarnessFixtures.class.getResourceAsStream("/lkjmc-docs-bundle.json")));
}
