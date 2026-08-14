package com.lkjmc.paper;

import com.lkjmc.common.menu.MenuAction;
import com.lkjmc.common.menu.MenuBundle;
import com.lkjmc.common.menu.MenuController;
import com.lkjmc.common.menu.MenuFrame;
import com.lkjmc.common.menu.MenuRenderer;
import com.lkjmc.common.menu.MenuResult;
import java.util.Map;

public final class PaperMenuProtocolAdapter {
    private final MenuController controller;

    public PaperMenuProtocolAdapter(MenuBundle bundle, MenuRenderer renderer) {
        controller = new MenuController(bundle, renderer);
    }

    public MenuResult open(long session, String route, Map<String, String> params, String locale) {
        return controller.open(session, route, params, locale);
    }

    public MenuResult click(MenuFrame.Metadata metadata, MenuAction action) {
        return controller.click(metadata, action);
    }

    public MenuFrame frame() { return controller.frame(); }
}
